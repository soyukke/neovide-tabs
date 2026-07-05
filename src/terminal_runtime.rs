use std::{
    cell::RefCell,
    env,
    io::{Read, Write},
    process::Command,
    rc::Rc,
    sync::mpsc::{self, Receiver},
    thread,
};

use anyhow::Result;
use libghostty_vt::{
    RenderState, Terminal, TerminalOptions,
    render::{CellIteration, CellIterator, CursorVisualStyle, RowIteration, RowIterator, Snapshot},
    style::{RgbColor, Underline},
    terminal::ScrollViewport,
};
use portable_pty::{Child, CommandBuilder, MasterPty, PtySize, native_pty_system};
use serde::Serialize;

use crate::neovide_render::{
    NeovideLine, NeovideRenderedWindowCache, NeovideRenderedWindowPlacement,
    NeovideRendererModelSnapshot, NeovideWindowDrawCommand, NeovideWindowKind,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TerminalGridSize {
    pub rows: u16,
    pub cols: u16,
    pub pixel_width: u16,
    pub pixel_height: u16,
}

impl TerminalGridSize {
    fn pty_size(self) -> PtySize {
        PtySize {
            rows: self.rows,
            cols: self.cols,
            pixel_width: self.pixel_width,
            pixel_height: self.pixel_height,
        }
    }
}

pub struct NativeTerminalRuntime {
    pty: RuntimePty,
    pty_replies: Rc<RefCell<Vec<u8>>>,
    terminal: Terminal<'static, 'static>,
    renderer: TerminalFrameRenderer,
    renderer_model: TerminalRendererModel,
    size: TerminalGridSize,
}

impl NativeTerminalRuntime {
    pub fn spawn(size: TerminalGridSize) -> Result<Self> {
        let pty = RuntimePty::spawn(size)?;
        let pty_replies = Rc::new(RefCell::new(Vec::new()));
        let mut terminal = Terminal::new(TerminalOptions {
            cols: size.cols,
            rows: size.rows,
            max_scrollback: 100_000,
        })?;

        terminal.on_pty_write({
            let pty_replies = Rc::clone(&pty_replies);
            move |_term, data| {
                pty_replies.borrow_mut().extend_from_slice(data);
            }
        })?;

        Ok(Self {
            pty,
            pty_replies,
            terminal,
            renderer: TerminalFrameRenderer::new()?,
            renderer_model: TerminalRendererModel::new(size),
            size,
        })
    }

    pub fn write_all(&mut self, bytes: &[u8]) -> Result<()> {
        self.pty.write_all(bytes)
    }

    pub fn resize(&mut self, size: TerminalGridSize) -> Result<()> {
        if self.size == size {
            return Ok(());
        }
        self.terminal.resize(
            size.cols,
            size.rows,
            size.pixel_width.into(),
            size.pixel_height.into(),
        )?;
        self.pty.resize(size)?;
        self.renderer_model.resize(size);
        self.size = size;
        Ok(())
    }

    pub fn drain(&mut self) -> Result<bool> {
        let mut changed = false;
        while let Ok(bytes) = self.pty.rx.try_recv() {
            self.terminal.vt_write(&bytes);
            changed = true;

            if !self.pty_replies.borrow().is_empty() {
                let replies = std::mem::take(&mut *self.pty_replies.borrow_mut());
                self.pty.write_all(&replies)?;
            }
        }
        Ok(changed)
    }

    pub fn scroll_delta(&mut self, requested_rows: isize) -> Result<isize> {
        let terminal_rows = bounded_scroll_rows(
            requested_rows,
            self.terminal.scrollbar().ok().map(ScrollbarView::from),
        );
        if terminal_rows != 0 {
            self.terminal
                .scroll_viewport(ScrollViewport::Delta(terminal_rows));
            self.renderer_model.record_scroll_delta(terminal_rows);
        }
        Ok(terminal_rows)
    }

    pub fn frame(&mut self) -> Result<TerminalFrameSnapshot> {
        self.renderer.collect(&mut self.terminal)
    }

    pub fn renderer_model(&mut self) -> Result<NeovideRendererModelSnapshot> {
        let frame = self.frame()?;
        Ok(self.renderer_model.snapshot(&frame))
    }

    pub fn advance_renderer_animations(&mut self, dt: f32) -> bool {
        self.renderer_model.advance_animations(dt)
    }

    pub fn has_active_renderer_animation(&self) -> bool {
        self.renderer_model.has_active_animation()
    }

    pub fn renderer_scroll_position(&self) -> f32 {
        self.renderer_model.scroll_position()
    }
}

impl Drop for NativeTerminalRuntime {
    fn drop(&mut self) {
        let _ = self.pty.kill();
    }
}

struct RuntimePty {
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    rx: Receiver<Vec<u8>>,
    child: Box<dyn Child + Send + Sync>,
}

impl RuntimePty {
    fn spawn(size: TerminalGridSize) -> Result<Self> {
        let pty_system = native_pty_system();
        let pair = pty_system.openpty(size.pty_size())?;
        let mut cmd = CommandBuilder::new(default_shell());
        configure_shell_command(&mut cmd);

        let child = pair.slave.spawn_command(cmd)?;
        let mut reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || read_pty_loop(&mut reader, tx));

        Ok(Self {
            master: pair.master,
            writer,
            rx,
            child,
        })
    }

    fn resize(&self, size: TerminalGridSize) -> Result<()> {
        self.master.resize(size.pty_size())
    }

    fn write_all(&mut self, bytes: &[u8]) -> Result<()> {
        self.writer.write_all(bytes)?;
        self.writer.flush()?;
        Ok(())
    }

    fn kill(&mut self) -> Result<()> {
        self.child.kill()?;
        Ok(())
    }
}

fn read_pty_loop(reader: &mut Box<dyn Read + Send>, tx: mpsc::Sender<Vec<u8>>) {
    let mut buffer = [0u8; 16 * 1024];
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(count) => {
                if tx.send(buffer[..count].to_vec()).is_err() {
                    break;
                }
            }
            Err(_) => break,
        }
    }
}

fn configure_shell_command(cmd: &mut CommandBuilder) {
    cmd.arg("-l");
    cmd.arg("-i");
    if let Ok(cwd) = env::current_dir() {
        cmd.cwd(&cwd);
        cmd.env("PWD", cwd);
    }
    let locale = terminal_locale();
    cmd.env_remove("LC_ALL");
    cmd.env("LANG", &locale);
    cmd.env("LC_CTYPE", &locale);
    cmd.env("TERM", "xterm-256color");
    cmd.env("COLORTERM", "truecolor");
    cmd.env("NVTERM_PROTO", "libghostty-vt");
}

struct TerminalFrameRenderer {
    state: RenderState<'static>,
    rows: RowIterator<'static>,
    cells: CellIterator<'static>,
}

struct TerminalRendererModel {
    window: NeovideRenderedWindowCache,
    pending_scroll_delta: isize,
}

const MAX_TERMINAL_SCROLL_DETECTION_ROWS: usize = 80;
const MIN_TERMINAL_SCROLL_MATCH_ROWS: usize = 4;
const MIN_TERMINAL_SCROLL_REGION_ROWS: usize = 2;
const MAX_TERMINAL_SCROLL_ANIMATION_ROWS: isize = 24;

impl TerminalRendererModel {
    fn new(size: TerminalGridSize) -> Self {
        Self {
            window: NeovideRenderedWindowCache::new(size.cols as usize, size.rows as usize),
            pending_scroll_delta: 0,
        }
    }

    fn resize(&mut self, size: TerminalGridSize) {
        self.apply_position(size.cols as usize, size.rows as usize);
    }

    fn record_scroll_delta(&mut self, rows: isize) {
        self.pending_scroll_delta += rows;
    }

    fn snapshot(&mut self, frame: &TerminalFrameSnapshot) -> NeovideRendererModelSnapshot {
        let height = frame.rows.len().max(1);
        let width = frame.rows.iter().map(Vec::len).max().unwrap_or(1).max(1);
        let inferred_scroll = self.infer_output_scroll(frame);
        let bottom_margin = terminal_bottom_margin(&frame.rows);
        self.apply_position(width, height);
        self.apply_viewport_margins(bottom_margin);
        for (row, cells) in frame.rows.iter().enumerate() {
            self.window.apply(&NeovideWindowDrawCommand::DrawLine {
                row,
                line: NeovideLine::from_cells(cells.clone()),
            });
        }
        if let Some(rows) = inferred_scroll {
            self.record_scroll_delta(rows);
        }
        self.apply_pending_scroll_delta();
        self.window.flush(1);

        NeovideRendererModelSnapshot {
            schema_version: 1,
            background: frame.background,
            cursor_color: frame.cursor_color,
            cursor: frame.cursor.clone(),
            scroll_hint: None,
            windows: vec![
                self.window
                    .snapshot(1, NeovideRenderedWindowPlacement::main(width, height)),
            ],
        }
    }

    fn advance_animations(&mut self, dt: f32) -> bool {
        self.window.advance_animation(dt)
    }

    fn has_active_animation(&self) -> bool {
        self.window.has_active_animation()
    }

    fn scroll_position(&self) -> f32 {
        self.window.scroll_position()
    }

    fn apply_position(&mut self, width: usize, height: usize) {
        self.window.apply(&NeovideWindowDrawCommand::Position {
            top: 0,
            left: 0,
            width,
            height,
            window_kind: NeovideWindowKind::Normal,
            zindex: 0,
            compindex: 0,
        });
    }

    fn apply_viewport_margins(&mut self, bottom: usize) {
        self.window
            .apply(&NeovideWindowDrawCommand::ViewportMargins {
                top: 0,
                bottom,
                left: 0,
                right: 0,
            });
    }

    fn apply_pending_scroll_delta(&mut self) {
        if self.pending_scroll_delta == 0 {
            return;
        }
        self.window.apply(&NeovideWindowDrawCommand::Viewport {
            scroll_delta: self.pending_scroll_delta,
        });
        self.pending_scroll_delta = 0;
    }

    fn infer_output_scroll(&self, frame: &TerminalFrameSnapshot) -> Option<isize> {
        if !scrollbar_is_at_bottom(&frame.scrollbar) {
            return None;
        }
        let previous = self.previous_rows(frame.rows.len())?;
        detect_terminal_output_scroll(&previous, &frame.rows)
    }

    fn previous_rows(&self, height: usize) -> Option<Vec<Vec<TerminalCellSnapshot>>> {
        let rows = (0..height)
            .map(|row| self.window.line(row).map(|line| line.cells.clone()))
            .collect::<Option<Vec<_>>>()?;
        Some(rows)
    }
}

fn detect_terminal_output_scroll(
    previous: &[Vec<TerminalCellSnapshot>],
    current: &[Vec<TerminalCellSnapshot>],
) -> Option<isize> {
    if previous.len() != current.len() || previous.len() < 2 || previous == current {
        return None;
    }
    let end_row = terminal_scrollable_end_row(current)?;
    let previous = &previous[..=end_row];
    let current = &current[..=end_row];
    best_terminal_scroll_shift(previous, current).map(|rows| {
        let limit = (current.len() as isize - 1).clamp(1, MAX_TERMINAL_SCROLL_ANIMATION_ROWS);
        rows.clamp(-limit, limit)
    })
}

fn best_terminal_scroll_shift(
    previous: &[Vec<TerminalCellSnapshot>],
    current: &[Vec<TerminalCellSnapshot>],
) -> Option<isize> {
    let max_shift = (previous.len() - 1).min(MAX_TERMINAL_SCROLL_DETECTION_ROWS);
    let mut best = None;
    for shift in 1..=max_shift {
        best = better_scroll_candidate(best, scroll_candidate(previous, current, shift, 1));
        best = better_scroll_candidate(best, scroll_candidate(previous, current, shift, -1));
    }
    best.map(|candidate| candidate.rows)
}

fn scroll_candidate(
    previous: &[Vec<TerminalCellSnapshot>],
    current: &[Vec<TerminalCellSnapshot>],
    shift: usize,
    direction: isize,
) -> Option<TerminalScrollCandidate> {
    let mut matched_rows = 0;
    let mut content_rows = 0;
    for row in 0..previous.len().saturating_sub(shift) {
        let previous_row = if direction > 0 { row + shift } else { row };
        let current_row = if direction > 0 { row } else { row + shift };
        if !terminal_rows_match(&previous[previous_row], &current[current_row]) {
            continue;
        }
        matched_rows += 1;
        if terminal_row_has_content(&previous[previous_row])
            || terminal_row_has_content(&current[current_row])
        {
            content_rows += 1;
        }
    }
    let required = MIN_TERMINAL_SCROLL_MATCH_ROWS.max(previous.len() / 4);
    (content_rows >= required && content_rows >= MIN_TERMINAL_SCROLL_REGION_ROWS).then_some(
        TerminalScrollCandidate {
            rows: direction * shift as isize,
            score: content_rows * 100 + matched_rows,
        },
    )
}

#[derive(Clone, Copy)]
struct TerminalScrollCandidate {
    rows: isize,
    score: usize,
}

fn better_scroll_candidate(
    current: Option<TerminalScrollCandidate>,
    next: Option<TerminalScrollCandidate>,
) -> Option<TerminalScrollCandidate> {
    match (current, next) {
        (Some(current), Some(next)) if current.score >= next.score => Some(current),
        (_, Some(next)) => Some(next),
        (current, None) => current,
    }
}

fn terminal_rows_match(
    previous: &[TerminalCellSnapshot],
    current: &[TerminalCellSnapshot],
) -> bool {
    if previous == current {
        return true;
    }
    let previous_text = row_text(previous);
    let current_text = row_text(current);
    if previous_text.chars().count() <= 8 || current_text.chars().count() <= 8 {
        return false;
    }
    let previous_body = scroll_row_body(&previous_text);
    let current_body = scroll_row_body(&current_text);
    previous_body == current_body && !previous_body.trim().is_empty()
}

fn terminal_row_has_content(row: &[TerminalCellSnapshot]) -> bool {
    let text = row_text(row);
    let body = scroll_row_body(&text).trim().to_owned();
    !body.is_empty() || (text.chars().count() <= 8 && !text.trim().is_empty())
}

fn terminal_scrollable_end_row(rows: &[Vec<TerminalCellSnapshot>]) -> Option<usize> {
    if rows.is_empty() {
        return None;
    }
    let fixed_tail_start = terminal_fixed_tail_start(rows);
    Some(fixed_tail_start.map_or(rows.len() - 1, |start| start.saturating_sub(1)))
}

fn terminal_bottom_margin(rows: &[Vec<TerminalCellSnapshot>]) -> usize {
    terminal_fixed_tail_start(rows).map_or(0, |start| rows.len().saturating_sub(start))
}

fn terminal_fixed_tail_start(rows: &[Vec<TerminalCellSnapshot>]) -> Option<usize> {
    if rows.len() < 4 {
        return None;
    }
    let first_candidate = rows.len().saturating_sub(8);
    (first_candidate..rows.len()).find(|row| terminal_row_looks_fixed(&rows[*row]))
}

fn terminal_row_looks_fixed(row: &[TerminalCellSnapshot]) -> bool {
    let colored_cells = row.iter().filter(|cell| cell.bg.is_some()).count();
    colored_cells >= 8.max(row.len() / 4)
}

fn scrollbar_is_at_bottom(scrollbar: &ScrollbarSnapshot) -> bool {
    scrollbar.total <= scrollbar.visible
        || scrollbar.top.saturating_add(scrollbar.visible) >= scrollbar.total
}

fn row_text(row: &[TerminalCellSnapshot]) -> String {
    row.iter().map(|cell| cell.text.as_str()).collect()
}

fn scroll_row_body(text: &str) -> String {
    if text.chars().count() <= 8 {
        return text.to_owned();
    }
    text.chars().skip(8).collect()
}

impl TerminalFrameRenderer {
    fn new() -> Result<Self> {
        Ok(Self {
            state: RenderState::new()?,
            rows: RowIterator::new()?,
            cells: CellIterator::new()?,
        })
    }

    fn collect(&mut self, terminal: &mut Terminal<'static, '_>) -> Result<TerminalFrameSnapshot> {
        let state = &mut self.state;
        let row_iter = &mut self.rows;
        let cell_iter = &mut self.cells;
        let scrollbar = terminal.scrollbar()?;
        let snapshot = state.update(terminal)?;
        let colors = snapshot.colors()?;
        let background = TerminalColor::from_rgb(colors.background);
        let cursor_color = TerminalColor::from_rgb(colors.cursor.unwrap_or(colors.foreground));
        let cursor = terminal_cursor(&snapshot)?;
        let rows = collect_rows(
            row_iter,
            cell_iter,
            &snapshot,
            colors.foreground,
            background,
        )?;
        snapshot.set_dirty(libghostty_vt::render::Dirty::Clean)?;

        Ok(TerminalFrameSnapshot {
            rows,
            background,
            cursor_color,
            cursor,
            scrollbar: ScrollbarSnapshot {
                top: scrollbar.offset,
                visible: scrollbar.len,
                total: scrollbar.total,
            },
        })
    }
}

fn collect_rows<'alloc>(
    rows: &mut RowIterator<'alloc>,
    cells: &mut CellIterator<'alloc>,
    snapshot: &Snapshot<'alloc, '_>,
    default_fg: RgbColor,
    background: TerminalColor,
) -> Result<Vec<Vec<TerminalCellSnapshot>>> {
    let mut output = Vec::with_capacity(snapshot.rows()? as usize);
    let mut row_iter = rows.update(snapshot)?;
    while let Some(row) = row_iter.next() {
        output.push(collect_cells(cells, row, default_fg, background)?);
        row.set_dirty(false)?;
    }
    Ok(output)
}

fn collect_cells<'alloc>(
    cells: &mut CellIterator<'alloc>,
    row: &RowIteration<'alloc, '_>,
    default_fg: RgbColor,
    background: TerminalColor,
) -> Result<Vec<TerminalCellSnapshot>> {
    let mut output = Vec::new();
    let mut cell_iter = cells.update(row)?;
    while let Some(cell) = cell_iter.next() {
        output.push(terminal_cell(cell, default_fg, background)?);
    }
    Ok(output)
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct TerminalFrameSnapshot {
    pub rows: Vec<Vec<TerminalCellSnapshot>>,
    pub background: TerminalColor,
    pub cursor_color: TerminalColor,
    pub cursor: Option<TerminalCursorSnapshot>,
    pub scrollbar: ScrollbarSnapshot,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct TerminalCellSnapshot {
    pub text: String,
    pub fg: TerminalColor,
    pub bg: Option<TerminalColor>,
    #[serde(default)]
    pub blend: u8,
    pub style: TerminalCellStyle,
}

#[derive(Clone, Copy, Debug, Default, Serialize, PartialEq, Eq, Hash)]
pub struct TerminalCellStyle {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
pub struct TerminalColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl TerminalColor {
    fn from_rgb(rgb: RgbColor) -> Self {
        Self {
            r: rgb.r,
            g: rgb.g,
            b: rgb.b,
        }
    }
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct TerminalCursorSnapshot {
    pub x: u16,
    pub y: u16,
    pub style: &'static str,
    pub cell_percentage: u8,
    pub blinkwait_ms: u64,
    pub blinkon_ms: u64,
    pub blinkoff_ms: u64,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ScrollbarSnapshot {
    pub top: u64,
    pub visible: u64,
    pub total: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ScrollbarView {
    top: u64,
    visible: u64,
    total: u64,
}

impl From<libghostty_vt::terminal::Scrollbar> for ScrollbarView {
    fn from(scrollbar: libghostty_vt::terminal::Scrollbar) -> Self {
        Self {
            top: scrollbar.offset,
            visible: scrollbar.len,
            total: scrollbar.total,
        }
    }
}

fn bounded_scroll_rows(requested_rows: isize, scrollbar: Option<ScrollbarView>) -> isize {
    let Some(scrollbar) = scrollbar else {
        return requested_rows;
    };
    if requested_rows == 0 || scrollbar.total <= scrollbar.visible {
        return 0;
    }

    let max_top = scrollbar.total.saturating_sub(scrollbar.visible);
    let available_rows = if requested_rows < 0 {
        scrollbar.top
    } else {
        max_top.saturating_sub(scrollbar.top)
    };
    let requested_abs = requested_rows.unsigned_abs().min(isize::MAX as usize) as u64;
    let moved_rows = requested_abs.min(available_rows).min(isize::MAX as u64) as isize;
    requested_rows.signum() * moved_rows
}

fn terminal_cursor(snapshot: &Snapshot<'_, '_>) -> Result<Option<TerminalCursorSnapshot>> {
    if !snapshot.cursor_visible()? {
        return Ok(None);
    }
    Ok(snapshot
        .cursor_viewport()?
        .map(|cursor| TerminalCursorSnapshot {
            x: cursor.x,
            y: cursor.y,
            style: cursor_style_name(
                snapshot
                    .cursor_visual_style()
                    .unwrap_or(CursorVisualStyle::Block),
            ),
            cell_percentage: 100,
            blinkwait_ms: 0,
            blinkon_ms: 0,
            blinkoff_ms: 0,
        }))
}

fn terminal_cell(
    cell: &CellIteration<'_, '_>,
    default_fg: RgbColor,
    background: TerminalColor,
) -> Result<TerminalCellSnapshot> {
    let style = cell.style()?;
    let mut text = String::new();
    cell.graphemes_utf8(&mut text)?;
    if style.invisible {
        text.clear();
    }

    let mut fg = cell.fg_color()?.unwrap_or(default_fg);
    let mut bg = cell.bg_color()?;
    if style.inverse {
        let inverse_bg = fg;
        fg = bg.unwrap_or(default_fg);
        bg = Some(inverse_bg);
    }

    Ok(TerminalCellSnapshot {
        text,
        fg: TerminalColor::from_rgb(fg),
        bg: bg
            .map(TerminalColor::from_rgb)
            .filter(|color| *color != background),
        blend: 0,
        style: TerminalCellStyle {
            bold: style.bold,
            italic: style.italic,
            underline: style.underline != Underline::None,
            strikethrough: style.strikethrough,
        },
    })
}

fn cursor_style_name(style: CursorVisualStyle) -> &'static str {
    match style {
        CursorVisualStyle::Bar => "bar",
        CursorVisualStyle::Underline => "underline",
        _ => "block",
    }
}

fn default_shell() -> String {
    env_shell("NVTERM_SHELL")
        .or_else(login_shell)
        .or_else(|| env_shell("SHELL"))
        .unwrap_or_else(|| "/bin/zsh".to_owned())
}

fn env_shell(key: &str) -> Option<String> {
    env::var(key).ok().filter(|shell| !shell.is_empty())
}

fn login_shell() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        macos_login_shell()
    }
    #[cfg(not(target_os = "macos"))]
    {
        None
    }
}

#[cfg(target_os = "macos")]
fn macos_login_shell() -> Option<String> {
    let user = env::var("USER").ok()?;
    let output = Command::new("/usr/bin/dscl")
        .args([".", "-read", &format!("/Users/{user}"), "UserShell"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let output = String::from_utf8(output.stdout).ok()?;
    output
        .lines()
        .next()?
        .split_whitespace()
        .last()
        .map(str::to_owned)
}

fn terminal_locale() -> String {
    env::var("LANG")
        .ok()
        .filter(|locale| locale.to_ascii_uppercase().contains("UTF-8"))
        .unwrap_or_else(|| "ja_JP.UTF-8".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renderer_collects_plain_text_frame() {
        let mut terminal = Terminal::new(TerminalOptions {
            cols: 8,
            rows: 3,
            max_scrollback: 100,
        })
        .unwrap();
        terminal.vt_write(b"hello");
        let frame = TerminalFrameRenderer::new()
            .unwrap()
            .collect(&mut terminal)
            .unwrap();

        let first_row = frame.rows.first().unwrap();
        let text = first_row
            .iter()
            .map(|cell| cell.text.as_str())
            .collect::<String>();
        assert!(text.starts_with("hello"));
        assert_eq!(frame.cursor.unwrap().x, 5);
    }

    #[test]
    fn scroll_rows_are_bounded_to_available_scrollback() {
        let scrollbar = ScrollbarView {
            top: 4,
            visible: 3,
            total: 10,
        };

        assert_eq!(bounded_scroll_rows(-20, Some(scrollbar)), -4);
        assert_eq!(bounded_scroll_rows(20, Some(scrollbar)), 3);
        assert_eq!(bounded_scroll_rows(0, Some(scrollbar)), 0);
    }

    #[test]
    fn terminal_renderer_model_exposes_single_skia_window() {
        let mut model = TerminalRendererModel::new(grid_size(2, 3));
        let snapshot = model.snapshot(&frame(&["abc", "def"]));

        assert_eq!(snapshot.windows.len(), 1);
        let window = &snapshot.windows[0];
        assert_eq!(window.width, 3);
        assert_eq!(window.height, 2);
        assert_eq!(window.lines[0].as_ref().unwrap().text, "abc");
        assert_eq!(snapshot.cursor.unwrap().x, 1);
    }

    #[test]
    fn terminal_renderer_model_records_history_scroll_animation() {
        let mut model = TerminalRendererModel::new(grid_size(3, 3));
        model.snapshot(&frame(&["111", "222", "333"]));

        model.record_scroll_delta(2);
        let snapshot = model.snapshot(&frame(&["333", "444", "555"]));

        assert_eq!(snapshot.windows[0].scroll_position, -2.0);
        assert!(model.has_active_animation());
        assert!(!model.advance_animations(0.3));
        assert!(!model.has_active_animation());
    }

    #[test]
    fn terminal_renderer_model_infers_vim_output_scroll_animation() {
        let mut model = TerminalRendererModel::new(grid_size(8, 16));
        model.snapshot(&frame_with_rows(vec![
            row("00000001 alpha"),
            row("00000002 beta"),
            row("00000003 gamma"),
            row("00000004 delta"),
            row("00000005 epsilon"),
            row("00000006 zeta"),
            row("00000007 eta"),
            status_row("vim status"),
        ]));

        let snapshot = model.snapshot(&frame_with_rows(vec![
            row("00000002 beta"),
            row("00000003 gamma"),
            row("00000004 delta"),
            row("00000005 epsilon"),
            row("00000006 zeta"),
            row("00000007 eta"),
            row("00000008 theta"),
            status_row("vim status"),
        ]));

        let window = &snapshot.windows[0];
        assert_eq!(window.scroll_position, -1.0);
        assert_eq!(window.viewport_margins.bottom, 1);
        assert_eq!(window.lines[7].as_ref().unwrap().text, "vim status");
    }

    fn frame(lines: &[&str]) -> TerminalFrameSnapshot {
        frame_with_rows(lines.iter().map(|line| row(line)).collect())
    }

    fn frame_with_rows(rows: Vec<Vec<TerminalCellSnapshot>>) -> TerminalFrameSnapshot {
        let visible = rows.len() as u64;
        TerminalFrameSnapshot {
            rows,
            background: TerminalColor { r: 0, g: 0, b: 0 },
            cursor_color: TerminalColor {
                r: 220,
                g: 220,
                b: 210,
            },
            cursor: Some(TerminalCursorSnapshot {
                x: 1,
                y: 0,
                style: "block",
                cell_percentage: 100,
                blinkwait_ms: 0,
                blinkon_ms: 0,
                blinkoff_ms: 0,
            }),
            scrollbar: ScrollbarSnapshot {
                top: 0,
                visible,
                total: visible,
            },
        }
    }

    fn row(text: &str) -> Vec<TerminalCellSnapshot> {
        text.chars()
            .map(|character| TerminalCellSnapshot {
                text: character.to_string(),
                fg: TerminalColor {
                    r: 220,
                    g: 220,
                    b: 210,
                },
                bg: None,
                blend: 0,
                style: TerminalCellStyle::default(),
            })
            .collect()
    }

    fn status_row(text: &str) -> Vec<TerminalCellSnapshot> {
        text.chars()
            .map(|character| TerminalCellSnapshot {
                text: character.to_string(),
                fg: TerminalColor {
                    r: 10,
                    g: 10,
                    b: 10,
                },
                bg: Some(TerminalColor {
                    r: 80,
                    g: 96,
                    b: 112,
                }),
                blend: 0,
                style: TerminalCellStyle::default(),
            })
            .collect()
    }

    fn grid_size(rows: u16, cols: u16) -> TerminalGridSize {
        TerminalGridSize {
            rows,
            cols,
            pixel_width: cols * 10,
            pixel_height: rows * 20,
        }
    }
}
