use std::collections::VecDeque;

use serde::Serialize;

use crate::terminal_runtime::{TerminalCellSnapshot, TerminalColor, TerminalCursorSnapshot};

pub const SCROLL_ANIMATION_LENGTH_SECONDS: f32 = 0.3;

// Neovide-derived rendering boundary.
//
// The command model, retained window line cache, and critically damped spring
// follow Neovide's MIT-licensed renderer architecture:
// https://github.com/neovide/neovide
// Copyright (c) Neovide Contributors.

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NeovideWindowDrawCommand {
    Position {
        top: usize,
        left: usize,
        width: usize,
        height: usize,
        window_kind: NeovideWindowKind,
        zindex: i64,
        compindex: i64,
    },
    DrawLine {
        row: usize,
        line: NeovideLine,
    },
    Scroll {
        top: usize,
        bottom: usize,
        left: usize,
        right: usize,
        rows: isize,
        cols: isize,
    },
    Clear,
    Show,
    Hide,
    Close,
    Viewport {
        scroll_delta: isize,
    },
    ViewportMargins {
        top: usize,
        bottom: usize,
        left: usize,
        right: usize,
    },
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NeovideWindowKind {
    Normal,
    Float,
    Message,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct NeovideLine {
    pub text: String,
    pub cells: Vec<TerminalCellSnapshot>,
}

impl NeovideLine {
    pub fn from_cells(cells: Vec<TerminalCellSnapshot>) -> Self {
        let text = cells.iter().map(|cell| cell.text.as_str()).collect();
        Self { text, cells }
    }
}

#[derive(Clone, Debug)]
pub struct NeovideRenderedWindowCache {
    height: usize,
    width: usize,
    lines: Vec<Option<NeovideLine>>,
    scrollback_lines: VecDeque<Option<NeovideLine>>,
    scroll_delta: isize,
    viewport_margins: NeovideViewportMargins,
    pub scroll_animation: CriticallyDampedSpringAnimation,
}

impl NeovideRenderedWindowCache {
    pub fn new(width: usize, height: usize) -> Self {
        let mut cache = Self {
            height: 0,
            width: 0,
            lines: Vec::new(),
            scrollback_lines: VecDeque::new(),
            scroll_delta: 0,
            viewport_margins: NeovideViewportMargins::default(),
            scroll_animation: CriticallyDampedSpringAnimation::new(),
        };
        cache.resize(width, height);
        cache
    }

    pub fn apply(&mut self, command: &NeovideWindowDrawCommand) {
        match command {
            NeovideWindowDrawCommand::Position { width, height, .. } => {
                self.resize(*width, *height);
            }
            NeovideWindowDrawCommand::DrawLine { row, line } => self.draw_line(*row, line.clone()),
            NeovideWindowDrawCommand::Scroll {
                top,
                bottom,
                left,
                right,
                rows,
                cols,
            } => self.scroll(*top, *bottom, *left, *right, *rows, *cols),
            NeovideWindowDrawCommand::Clear => self.clear(),
            NeovideWindowDrawCommand::Viewport { scroll_delta } => {
                self.scroll_delta = *scroll_delta;
            }
            NeovideWindowDrawCommand::ViewportMargins {
                top,
                bottom,
                left,
                right,
            } => {
                self.viewport_margins = NeovideViewportMargins {
                    top: *top,
                    bottom: *bottom,
                    left: *left,
                    right: *right,
                };
            }
            _ => {}
        }
    }

    pub fn flush(&mut self, far_lines: usize) {
        let inner_range = self.inner_row_range();
        let inner_height = inner_range.len();
        if self.scrollback_shape_changed(inner_height) {
            self.reset_scrollback();
            self.scroll_delta = 0;
            self.scroll_animation.reset();
            return;
        }

        let scroll_delta = self.scroll_delta;
        self.rotate_scrollback(scroll_delta);
        self.clone_inner_lines_to_scrollback(inner_range);
        if scroll_delta != 0 {
            self.scroll_animation.position = limited_scroll_offset(
                self.scroll_animation.position,
                scroll_delta,
                far_lines,
                inner_height,
            );
        }
        self.scroll_delta = 0;
    }

    pub fn advance_animation(&mut self, dt: f32) -> bool {
        self.scroll_animation
            .update(dt, SCROLL_ANIMATION_LENGTH_SECONDS)
    }

    pub fn has_active_animation(&self) -> bool {
        self.scroll_animation.position != 0.0
    }

    pub fn scroll_position(&self) -> f32 {
        self.scroll_animation.position
    }

    pub fn line(&self, row: usize) -> Option<&NeovideLine> {
        self.lines.get(row)?.as_ref()
    }

    pub fn snapshot(
        &self,
        grid_id: i64,
        placement: NeovideRenderedWindowPlacement,
    ) -> NeovideRenderedWindowSnapshot {
        NeovideRenderedWindowSnapshot {
            grid_id,
            top: placement.top,
            left: placement.left,
            width: placement.width,
            height: placement.height,
            window_kind: placement.window_kind,
            zindex: placement.zindex,
            compindex: placement.compindex,
            hidden: placement.hidden,
            scroll_position: self.scroll_animation.position,
            viewport_margins: self.viewport_margins,
            scrollback_zero_index: self.scrollback_zero_index(),
            scrollback_lines: self.scrollback_lines.iter().cloned().collect(),
            lines: self.lines.clone(),
        }
    }

    fn resize(&mut self, width: usize, height: usize) {
        let width = width.max(1);
        let height = height.max(1);
        if self.width == width && self.height == height {
            return;
        }

        self.width = width;
        self.height = height;
        self.lines.resize(self.height, None);
        self.scroll_delta = 0;
        self.scroll_animation.reset();
        self.reset_scrollback();
    }

    fn draw_line(&mut self, row: usize, line: NeovideLine) {
        if let Some(slot) = self.lines.get_mut(row) {
            *slot = Some(line);
        }
    }

    fn scroll(
        &mut self,
        top: usize,
        bottom: usize,
        left: usize,
        right: usize,
        rows: isize,
        cols: isize,
    ) {
        if top == 0 && bottom == self.height && left == 0 && right == self.width && cols == 0 {
            self.rotate_visible_rows(rows);
        }
    }

    fn rotate_visible_rows(&mut self, rows: isize) {
        if rows > 0 {
            for _ in 0..rows {
                self.lines.remove(0);
                self.lines.push(None);
            }
            return;
        }
        for _ in 0..rows.unsigned_abs() {
            self.lines.pop();
            self.lines.insert(0, None);
        }
    }

    fn clear(&mut self) {
        self.lines.fill(None);
        self.scroll_delta = 0;
        self.scroll_animation.reset();
        self.reset_scrollback();
    }

    fn reset_scrollback(&mut self) {
        let inner_range = self.inner_row_range();
        let inner_height = inner_range.len();
        self.scrollback_lines = VecDeque::from(vec![None; scrollback_len(inner_height)]);
        self.clone_inner_lines_to_scrollback(inner_range);
    }

    fn scrollback_shape_changed(&self, inner_height: usize) -> bool {
        self.scrollback_lines.len() != scrollback_len(inner_height)
    }

    fn clone_inner_lines_to_scrollback(&mut self, inner_range: std::ops::Range<usize>) {
        for (inner_row, source_row) in inner_range.enumerate() {
            let line = self.lines.get(source_row).cloned().flatten();
            self.set_scrollback_line(inner_row as isize, line);
        }
    }

    fn rotate_scrollback(&mut self, scroll_delta: isize) {
        if scroll_delta == 0 || self.scrollback_lines.is_empty() {
            return;
        }
        let len = self.scrollback_lines.len();
        if scroll_delta.unsigned_abs() >= len {
            for line in &mut self.scrollback_lines {
                *line = None;
            }
            return;
        }
        let rows = scroll_delta.unsigned_abs() % len;
        if scroll_delta > 0 {
            self.scrollback_lines.rotate_left(rows);
        } else {
            self.scrollback_lines.rotate_right(rows);
        }
    }

    fn set_scrollback_line(&mut self, signed_row: isize, line: Option<NeovideLine>) {
        let Some(index) = self.scrollback_index(signed_row) else {
            return;
        };
        if let Some(target) = self.scrollback_lines.get_mut(index) {
            *target = line;
        }
    }

    fn scrollback_index(&self, signed_row: isize) -> Option<usize> {
        scrollback_index(self.scrollback_lines.len(), signed_row)
    }

    fn scrollback_zero_index(&self) -> usize {
        self.scrollback_lines.len() / 2
    }

    fn inner_row_range(&self) -> std::ops::Range<usize> {
        let top = self.viewport_margins.top.min(self.height);
        let bottom_margin = self
            .viewport_margins
            .bottom
            .min(self.height.saturating_sub(top));
        top..self.height.saturating_sub(bottom_margin)
    }
}

fn limited_scroll_offset(
    current: f32,
    scroll_delta: isize,
    far_lines: usize,
    inner_height: usize,
) -> f32 {
    if inner_height == 0 {
        return 0.0;
    }
    let far_lines = far_lines.max(1).min(inner_height) as isize;
    let max_delta = inner_height as f32;
    if scroll_delta.unsigned_abs() > inner_height {
        return -(far_lines * scroll_delta.signum()) as f32;
    }
    (current - scroll_delta as f32).clamp(-max_delta, max_delta)
}

fn scrollback_len(inner_height: usize) -> usize {
    if inner_height == 0 {
        return 0;
    }
    inner_height * 2 + 1
}

fn scrollback_index(len: usize, signed_row: isize) -> Option<usize> {
    let zero = len / 2;
    let raw_index = zero.checked_add_signed(signed_row)?;
    (raw_index < len).then_some(raw_index)
}

#[derive(Clone, Debug)]
pub struct CriticallyDampedSpringAnimation {
    pub position: f32,
    velocity: f32,
}

impl CriticallyDampedSpringAnimation {
    pub fn new() -> Self {
        Self {
            position: 0.0,
            velocity: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32, animation_length: f32) -> bool {
        if animation_length <= dt {
            self.reset();
            return false;
        }
        if self.position == 0.0 {
            return false;
        }

        let omega = 4.0 / animation_length;
        let start = self.position;
        let velocity = self.position * omega + self.velocity;
        let decay = (-omega * dt).exp();

        self.position = (start + velocity * dt) * decay;
        self.velocity = decay * (-start * omega - velocity * dt * omega + velocity);
        if self.position.abs() < 0.01 {
            self.reset();
            return false;
        }
        true
    }

    pub fn reset(&mut self) {
        self.position = 0.0;
        self.velocity = 0.0;
    }
}

impl Default for CriticallyDampedSpringAnimation {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct NeovideRendererModelSnapshot {
    pub schema_version: u32,
    pub background: TerminalColor,
    pub cursor_color: TerminalColor,
    pub cursor: Option<TerminalCursorSnapshot>,
    pub scrollbar: Option<crate::terminal_runtime::ScrollbarSnapshot>,
    pub scroll_hint: Option<NeovideScrollHint>,
    pub windows: Vec<NeovideRenderedWindowSnapshot>,
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
pub struct NeovideScrollHint {
    pub start_row: usize,
    pub end_row: usize,
    pub start_col: usize,
    pub end_col: usize,
    pub rows: isize,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct NeovideRenderedWindowSnapshot {
    pub grid_id: i64,
    pub top: usize,
    pub left: usize,
    pub width: usize,
    pub height: usize,
    pub window_kind: NeovideWindowKind,
    pub zindex: i64,
    pub compindex: i64,
    pub hidden: bool,
    pub scroll_position: f32,
    pub viewport_margins: NeovideViewportMargins,
    pub scrollback_zero_index: usize,
    pub scrollback_lines: Vec<Option<NeovideLine>>,
    pub lines: Vec<Option<NeovideLine>>,
}

impl NeovideRenderedWindowSnapshot {
    pub fn scrollback_line(&self, signed_row: isize) -> Option<&NeovideLine> {
        let index = self.scrollback_zero_index.checked_add_signed(signed_row)?;
        self.scrollback_lines.get(index)?.as_ref()
    }

    pub fn inner_row_range(&self) -> std::ops::Range<usize> {
        let top = self.viewport_margins.top.min(self.height);
        let bottom_margin = self
            .viewport_margins
            .bottom
            .min(self.height.saturating_sub(top));
        top..self.height.saturating_sub(bottom_margin)
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, PartialEq, Eq)]
pub struct NeovideViewportMargins {
    pub top: usize,
    pub bottom: usize,
    pub left: usize,
    pub right: usize,
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
pub struct NeovideRenderedWindowPlacement {
    pub top: usize,
    pub left: usize,
    pub width: usize,
    pub height: usize,
    pub window_kind: NeovideWindowKind,
    pub zindex: i64,
    pub compindex: i64,
    pub hidden: bool,
}

impl NeovideRenderedWindowPlacement {
    pub fn main(width: usize, height: usize) -> Self {
        Self {
            top: 0,
            left: 0,
            width,
            height,
            window_kind: NeovideWindowKind::Normal,
            zindex: 0,
            compindex: 0,
            hidden: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal_runtime::{TerminalCellStyle, TerminalColor};

    #[test]
    fn rendered_window_keeps_line_cache_and_scrolls_like_neovide() {
        let mut window = NeovideRenderedWindowCache::new(3, 3);
        window.apply(&NeovideWindowDrawCommand::DrawLine {
            row: 0,
            line: NeovideLine::from_cells(row("aaa")),
        });
        window.apply(&NeovideWindowDrawCommand::DrawLine {
            row: 1,
            line: NeovideLine::from_cells(row("bbb")),
        });
        window.apply(&NeovideWindowDrawCommand::DrawLine {
            row: 2,
            line: NeovideLine::from_cells(row("ccc")),
        });

        window.apply(&NeovideWindowDrawCommand::Scroll {
            top: 0,
            bottom: 3,
            left: 0,
            right: 3,
            rows: 1,
            cols: 0,
        });

        assert_eq!(window.line(0).map(|line| line.text.as_str()), Some("bbb"));
        assert_eq!(window.line(1).map(|line| line.text.as_str()), Some("ccc"));
        assert!(window.line(2).is_none());
    }

    #[test]
    fn rendered_window_snapshot_exposes_scrollback_for_scroll_animation() {
        let mut window = NeovideRenderedWindowCache::new(3, 3);
        set_window_rows(&mut window, ["aaa", "bbb", "ccc"]);
        window.flush(1);

        window.apply(&NeovideWindowDrawCommand::Scroll {
            top: 0,
            bottom: 3,
            left: 0,
            right: 3,
            rows: 1,
            cols: 0,
        });
        window.apply(&NeovideWindowDrawCommand::DrawLine {
            row: 2,
            line: NeovideLine::from_cells(row("ddd")),
        });
        window.apply(&NeovideWindowDrawCommand::Viewport { scroll_delta: 1 });
        window.flush(1);

        let snapshot = window.snapshot(1, NeovideRenderedWindowPlacement::main(3, 3));

        assert_eq!(snapshot.scroll_position, -1.0);
        assert_eq!(
            snapshot.scrollback_line(-1).map(|line| line.text.as_str()),
            Some("aaa")
        );
        assert_eq!(
            snapshot.scrollback_line(0).map(|line| line.text.as_str()),
            Some("bbb")
        );
        assert_eq!(
            snapshot.scrollback_line(1).map(|line| line.text.as_str()),
            Some("ccc")
        );
        assert_eq!(
            snapshot.scrollback_line(2).map(|line| line.text.as_str()),
            Some("ddd")
        );
    }

    #[test]
    fn repeated_position_keeps_retained_scroll_animation() {
        let mut window = NeovideRenderedWindowCache::new(3, 3);
        set_window_rows(&mut window, ["aaa", "bbb", "ccc"]);
        window.flush(1);

        window.apply(&NeovideWindowDrawCommand::Viewport { scroll_delta: 1 });
        window.flush(1);
        window.apply(&NeovideWindowDrawCommand::Position {
            top: 0,
            left: 0,
            width: 3,
            height: 3,
            window_kind: NeovideWindowKind::Normal,
            zindex: 0,
            compindex: 0,
        });

        let snapshot = window.snapshot(1, NeovideRenderedWindowPlacement::main(3, 3));

        assert_eq!(snapshot.scroll_position, -1.0);
        assert_eq!(
            snapshot.scrollback_line(0).map(|line| line.text.as_str()),
            Some("aaa")
        );
    }

    #[test]
    fn viewport_margins_keep_fixed_rows_out_of_scrollback() {
        let mut window = NeovideRenderedWindowCache::new(3, 4);
        set_window_rows(&mut window, ["top", "aaa", "bbb", "bot"]);
        window.apply(&NeovideWindowDrawCommand::ViewportMargins {
            top: 1,
            bottom: 1,
            left: 0,
            right: 0,
        });
        window.flush(1);

        let snapshot = window.snapshot(1, NeovideRenderedWindowPlacement::main(3, 4));

        assert_eq!(snapshot.inner_row_range(), 1..3);
        assert!(snapshot.scrollback_line(-1).is_none());
        assert_eq!(
            snapshot.scrollback_line(0).map(|line| line.text.as_str()),
            Some("aaa")
        );
        assert_eq!(
            snapshot.scrollback_line(1).map(|line| line.text.as_str()),
            Some("bbb")
        );
        assert!(snapshot.scrollback_line(2).is_none());
    }

    #[test]
    fn spring_animation_converges_to_zero() {
        let mut animation = CriticallyDampedSpringAnimation::new();
        animation.position = 5.0;

        for _ in 0..60 {
            animation.update(1.0 / 60.0, 0.3);
        }

        assert_eq!(animation.position, 0.0);
    }

    #[test]
    fn rendered_window_snapshot_exposes_retained_lines() {
        let mut window = NeovideRenderedWindowCache::new(3, 2);
        window.apply(&NeovideWindowDrawCommand::DrawLine {
            row: 1,
            line: NeovideLine::from_cells(row("abc")),
        });

        let snapshot = window.snapshot(7, NeovideRenderedWindowPlacement::main(3, 2));

        assert_eq!(snapshot.grid_id, 7);
        assert_eq!(snapshot.width, 3);
        assert_eq!(snapshot.height, 2);
        assert_eq!(
            snapshot.lines[1].as_ref().map(|line| line.text.as_str()),
            Some("abc")
        );
    }

    fn row(text: &str) -> Vec<TerminalCellSnapshot> {
        text.chars()
            .map(|char| TerminalCellSnapshot {
                text: char.to_string(),
                fg: TerminalColor { r: 1, g: 2, b: 3 },
                bg: None,
                blend: 0,
                style: TerminalCellStyle::default(),
            })
            .collect()
    }

    fn set_window_rows<const N: usize>(window: &mut NeovideRenderedWindowCache, rows: [&str; N]) {
        for (row_index, text) in rows.into_iter().enumerate() {
            window.apply(&NeovideWindowDrawCommand::DrawLine {
                row: row_index,
                line: NeovideLine::from_cells(row(text)),
            });
        }
    }
}
