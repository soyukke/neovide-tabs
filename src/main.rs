use std::{
    cell::RefCell,
    collections::{HashMap, hash_map::DefaultHasher},
    env, fs,
    hash::{Hash, Hasher},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::Command,
    rc::Rc,
    sync::mpsc::{self, Receiver},
    thread,
    time::{Duration, Instant},
};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use anyhow::{Context, Result, anyhow};
use libghostty_vt::{
    RenderState, Terminal, TerminalOptions,
    render::{CellIterator, CursorVisualStyle, RowIterator},
    style::RgbColor,
    terminal::ScrollViewport,
};
use macroquad::prelude::*;
use macroquad::{
    input::utils as input_utils,
    miniquad::{EventHandler, KeyMods},
};
use portable_pty::{Child, CommandBuilder, MasterPty, PtySize, native_pty_system};
use serde::Deserialize;

use neovide_tabs::{
    core::{
        KittyCellPosition, KittyGraphicsState, KittyGraphicsTracker, KittyImageFormat,
        KittyImageResource, KittyPlacementKey, KittyTransmission, PaneId, PaneLayout,
        SessionPaneState, SessionState, SessionTabState, SplitAxis,
    },
    scroll::SmoothScroll,
};

#[cfg(test)]
use neovide_tabs::core::{StoredPaneLayout, StoredSplitAxis};

const FONT_SIZE: u16 = 17;
const SCROLL_ROWS_PER_WHEEL_UNIT: f32 = 3.0;
const WHEEL_SETTLE_AFTER: Duration = Duration::from_millis(140);
const TAB_BAR_HEIGHT: f32 = 34.0;
const TAB_MIN_WIDTH: f32 = 118.0;
const TAB_MAX_WIDTH: f32 = 220.0;
const TAB_MENU_WIDTH: f32 = 182.0;
const TAB_MENU_PADDING: f32 = 5.0;
const TAB_MENU_ROW_HEIGHT: f32 = 26.0;
const TAB_MENU_SEPARATOR_HEIGHT: f32 = 7.0;
const TAB_MENU_RADIUS: f32 = 9.0;
const PANE_GAP: f32 = 2.0;
const MIN_PANE_COLS: f32 = 8.0;
const MIN_PANE_ROWS: f32 = 3.0;
const MAX_OUTPUT_SCROLL_ANIMATION_ROWS: usize = 12;
const OUTPUT_SCROLL_ANIMATION_FAR_LINES: usize = 1;
const CURSOR_ANIMATION_LENGTH: f32 = 0.150;
const CURSOR_SHORT_ANIMATION_LENGTH: f32 = 0.040;
const CURSOR_SNAP_EPSILON: f32 = 0.01;
const CURSOR_TRAIL_SIZE: f32 = 1.0;
const CURSOR_TRAIL_ALPHA: u8 = 145;
const ICON_SMALL_BYTES: usize = 16 * 16 * 4;
const ICON_MEDIUM_BYTES: usize = 32 * 32 * 4;
const ICON_BIG_BYTES: usize = 64 * 64 * 4;
const SESSION_FILE_NAME: &str = "session.toml";
const AGENT_NOTIFY_MIN_BUSY: Duration = Duration::from_secs(8);

fn window_conf() -> Conf {
    Conf {
        window_title: "neovide-tabs prototype".to_owned(),
        window_width: 1100,
        window_height: 720,
        high_dpi: true,
        sample_count: 1,
        icon: app_icon(),
        ..Default::default()
    }
}

fn app_icon() -> Option<macroquad::miniquad::conf::Icon> {
    let source = image::load_from_memory(include_bytes!("../assets/app-icon.png"))
        .ok()?
        .to_rgba8();

    Some(macroquad::miniquad::conf::Icon {
        small: icon_array::<ICON_SMALL_BYTES>(resize_icon_rgba(&source, 16))?,
        medium: icon_array::<ICON_MEDIUM_BYTES>(resize_icon_rgba(&source, 32))?,
        big: icon_array::<ICON_BIG_BYTES>(resize_icon_rgba(&source, 64))?,
    })
}

fn resize_icon_rgba(source: &image::RgbaImage, size: u32) -> Vec<u8> {
    let resized =
        image::imageops::resize(source, size, size, image::imageops::FilterType::Lanczos3);

    resized.into_raw()
}

fn icon_array<const N: usize>(bytes: Vec<u8>) -> Option<[u8; N]> {
    bytes.try_into().ok()
}

#[derive(Clone, Debug, Default, Deserialize)]
struct AppConfig {
    #[serde(default)]
    font: FontConfig,
    #[serde(default)]
    ui: UiConfig,
    #[serde(default)]
    notifications: NotificationConfig,
    #[serde(default)]
    keybindings: HashMap<String, String>,
}

impl AppConfig {
    fn load() -> Result<Self> {
        let Some(path) = config_path() else {
            return Ok(Self::default());
        };

        if !path.exists() {
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(&path)
            .with_context(|| format!("failed to read config {}", path.display()))?;
        toml::from_str(&contents)
            .with_context(|| format!("failed to parse config {}", path.display()))
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
struct FontConfig {
    latin: Option<String>,
    cjk: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct UiConfig {
    theme: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct NotificationConfig {
    agents: Option<bool>,
    osc: Option<bool>,
    status_files: Option<bool>,
    agent_min_busy_seconds: Option<u64>,
}

#[derive(Clone, Copy, Debug)]
struct NotificationSettings {
    agents: bool,
    osc: bool,
    status_files: bool,
    agent_min_busy: Duration,
}

impl NotificationSettings {
    fn from_config(config: &AppConfig) -> Self {
        Self {
            agents: config.notifications.agents.unwrap_or(true),
            osc: config.notifications.osc.unwrap_or(true),
            status_files: config.notifications.status_files.unwrap_or(true),
            agent_min_busy: config
                .notifications
                .agent_min_busy_seconds
                .map(Duration::from_secs)
                .unwrap_or(AGENT_NOTIFY_MIN_BUSY),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DesktopNotification {
    title: String,
    subtitle: Option<String>,
    body: String,
}

fn send_desktop_notification(notification: &DesktopNotification) {
    if env::var_os("NVTERM_DEBUG_NOTIFY").is_some() {
        eprintln!(
            "nvterm notify: title={:?} subtitle={:?} body={:?}",
            notification.title, notification.subtitle, notification.body
        );
    }

    #[cfg(target_os = "macos")]
    {
        let mut script = format!(
            "display notification {} with title {}",
            applescript_string(&notification.body),
            applescript_string(&notification.title),
        );
        if let Some(subtitle) = &notification.subtitle {
            script.push_str(" subtitle ");
            script.push_str(&applescript_string(subtitle));
        }
        let _ = Command::new("osascript").arg("-e").arg(script).spawn();
    }

    #[cfg(not(target_os = "macos"))]
    {
        let title = if let Some(subtitle) = &notification.subtitle {
            format!("{} - {}", notification.title, subtitle)
        } else {
            notification.title.clone()
        };
        let _ = Command::new("notify-send")
            .arg(title)
            .arg(&notification.body)
            .spawn();
    }
}

fn applescript_string(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

fn config_path() -> Option<PathBuf> {
    if let Some(path) = env::var_os("NVTERM_CONFIG").filter(|path| !path.is_empty()) {
        return Some(PathBuf::from(path));
    }

    if let Some(config_home) = env::var_os("XDG_CONFIG_HOME").filter(|path| !path.is_empty()) {
        return Some(PathBuf::from(config_home).join("neovide-tabs/config.toml"));
    }

    env::var_os("HOME")
        .filter(|path| !path.is_empty())
        .map(|home| {
            PathBuf::from(home)
                .join(".config")
                .join("neovide-tabs")
                .join("config.toml")
        })
}

fn session_path() -> Option<PathBuf> {
    if let Some(path) = env::var_os("NVTERM_SESSION").filter(|path| !path.is_empty()) {
        return Some(PathBuf::from(path));
    }

    if let Some(state_home) = env::var_os("XDG_STATE_HOME").filter(|path| !path.is_empty()) {
        return Some(
            PathBuf::from(state_home)
                .join("neovide-tabs")
                .join(SESSION_FILE_NAME),
        );
    }

    env::var_os("HOME")
        .filter(|path| !path.is_empty())
        .map(|home| {
            PathBuf::from(home)
                .join(".local")
                .join("state")
                .join("neovide-tabs")
                .join(SESSION_FILE_NAME)
        })
}

fn app_state_dir() -> Option<PathBuf> {
    if let Some(state_home) = env::var_os("XDG_STATE_HOME").filter(|path| !path.is_empty()) {
        return Some(PathBuf::from(state_home).join("neovide-tabs"));
    }

    env::var_os("HOME")
        .filter(|path| !path.is_empty())
        .map(|home| {
            PathBuf::from(home)
                .join(".local")
                .join("state")
                .join("neovide-tabs")
        })
}

fn agent_status_dir() -> Option<PathBuf> {
    if let Some(path) = env::var_os("NVTERM_AGENT_STATUS_DIR").filter(|path| !path.is_empty()) {
        return Some(PathBuf::from(path));
    }

    app_state_dir().map(|path| path.join("agents"))
}

fn agent_status_path(dir: Option<&Path>, pane_id: PaneId) -> Option<PathBuf> {
    dir.map(|dir| dir.join(format!("pane-{}.status.toml", pane_id.0)))
}

fn agent_shim_dir(pane_id: PaneId) -> Option<PathBuf> {
    if let Some(root) = env::var_os("NVTERM_AGENT_SHIM_ROOT").filter(|path| !path.is_empty()) {
        return Some(PathBuf::from(root).join(format!("pane-{}", pane_id.0)));
    }

    app_state_dir().map(|path| path.join("shims").join(format!("pane-{}", pane_id.0)))
}

#[macroquad::main(window_conf)]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("{error:?}");
        loop {
            clear_background(Color::from_rgba(18, 20, 24, 255));
            draw_text_ex(
                format!("{error:#}"),
                24.0,
                48.0,
                TextParams {
                    font_size: 20,
                    color: Color::from_rgba(245, 108, 108, 255),
                    ..Default::default()
                },
            );
            next_frame().await;
        }
    }

    std::process::exit(0);
}

async fn run() -> Result<()> {
    let config = AppConfig::load()?;
    let session_file = session_path();
    let agent_status_dir = agent_status_dir();
    let saved_session = SessionState::load(session_file.as_deref())?;
    let fonts = TerminalFonts::load(&config).await;
    let metrics = CellMetrics::from_font(fonts.metrics_font());
    let mut app = AppState::new(
        content_rect(),
        metrics,
        &config,
        saved_session.as_ref(),
        session_file,
        agent_status_dir,
    )?;
    let input_subscriber = input_utils::register_input_subscriber();
    let debug_input = env::var_os("NVTERM_DEBUG_INPUT").is_some();
    let debug_pty = env::var_os("NVTERM_DEBUG_PTY").is_some();
    let debug_scroll = env::var_os("NVTERM_DEBUG_SCROLL").is_some();

    app.save_session_if_dirty()?;

    loop {
        let content_rect = content_rect();
        app.resize_all(content_rect, metrics)?;

        if app.drain(debug_pty)? {
            app.save_session_if_dirty()?;
            break Ok(());
        }

        let input = collect_input(
            input_subscriber,
            app.input_context(),
            &app.keybindings,
            debug_input,
        );
        if app.handle_input(input, content_rect, metrics)? {
            app.save_session_if_dirty()?;
            break Ok(());
        }
        app.handle_mouse(content_rect, metrics)?;
        app.save_session_if_dirty()?;

        let dt = get_frame_time();
        app.draw(&fonts, content_rect, metrics, dt, debug_scroll)?;
        next_frame().await;
    }
}

#[derive(Clone, Copy, Debug)]
struct PanePlacement {
    id: PaneId,
    rect: Rect,
    viewport: Viewport,
}

#[derive(Clone, Debug, Default)]
struct PaneDrain {
    cwd_changed: bool,
    notifications: Vec<DesktopNotification>,
}

struct TerminalPane {
    id: PaneId,
    cwd: Option<PathBuf>,
    pty: PtySession,
    pty_replies: Rc<RefCell<Vec<u8>>>,
    terminal: Terminal<'static, 'static>,
    renderer: TerminalRenderer,
    osc: OscTracker,
    kitty: KittyGraphicsTracker,
    graphics: KittyGraphicsState,
    image_textures: TerminalImageTextureCache,
    agent_status: AgentStatusFileMonitor,
    agent_monitor: AgentMonitor,
    smooth_scroll: SmoothScroll,
    cursor_motion: CursorMotion,
    last_wheel_at: Option<Instant>,
    previous_rows: Option<Vec<Vec<CellView>>>,
    previous_scrollbar: Option<ScrollbarView>,
    viewport: Option<Viewport>,
}

impl TerminalPane {
    fn new(
        id: PaneId,
        viewport: Viewport,
        theme: TerminalTheme,
        cwd: Option<PathBuf>,
        agent_status_dir: Option<&Path>,
    ) -> Result<Self> {
        let cwd = spawn_cwd(cwd);
        let agent_status_path = agent_status_path(agent_status_dir, id);
        let pty = PtySession::spawn(
            viewport.size,
            cwd.as_deref(),
            id,
            agent_status_path.as_deref(),
        )?;
        let pty_replies = Rc::new(RefCell::new(Vec::<u8>::new()));
        let mut terminal = Terminal::new(TerminalOptions {
            cols: viewport.size.cols,
            rows: viewport.size.rows,
            max_scrollback: 100_000,
        })?;

        terminal.on_pty_write({
            let pty_replies = Rc::clone(&pty_replies);
            move |_term, data| {
                pty_replies.borrow_mut().extend_from_slice(data);
            }
        })?;

        let mut pane = Self {
            id,
            cwd,
            pty,
            pty_replies,
            terminal,
            renderer: TerminalRenderer::new()?,
            osc: OscTracker::new(),
            kitty: KittyGraphicsTracker::new(),
            graphics: KittyGraphicsState::new(),
            image_textures: TerminalImageTextureCache::new(),
            agent_status: AgentStatusFileMonitor::new(agent_status_path),
            agent_monitor: AgentMonitor::new(),
            smooth_scroll: SmoothScroll::new(),
            cursor_motion: CursorMotion::new(),
            last_wheel_at: None,
            previous_rows: None,
            previous_scrollbar: None,
            viewport: Some(viewport),
        };
        pane.apply_theme(theme)?;
        Ok(pane)
    }

    fn apply_theme(&mut self, theme: TerminalTheme) -> Result<()> {
        self.terminal
            .set_default_bg_color(Some(theme.background.rgb()))?
            .set_default_fg_color(Some(theme.foreground.rgb()))?
            .set_default_cursor_color(Some(theme.cursor.rgb()))?;
        Ok(())
    }

    fn resize(&mut self, viewport: Viewport) -> Result<()> {
        if self
            .viewport
            .is_some_and(|current| current.size == viewport.size)
        {
            return Ok(());
        }

        self.terminal.resize(
            viewport.size.cols,
            viewport.size.rows,
            viewport.metrics.cell_width.round() as u32,
            viewport.metrics.cell_height.round() as u32,
        )?;
        self.pty.resize(viewport.size)?;
        self.smooth_scroll.set_all_rows(0.0);
        self.cursor_motion.reset();
        self.previous_rows = None;
        self.previous_scrollbar = None;
        self.viewport = Some(viewport);
        Ok(())
    }

    fn drain(&mut self, debug_pty: bool) -> Result<PaneDrain> {
        if self.pty.has_exited()? {
            return Ok(PaneDrain {
                cwd_changed: false,
                notifications: Vec::new(),
            });
        }

        let events = drain_pty(
            &self.pty.rx,
            &mut self.terminal,
            &self.pty_replies,
            TerminalProtocolTrackers {
                osc: &mut self.osc,
                kitty: &mut self.kitty,
                graphics: &mut self.graphics,
            },
            debug_pty,
            Some(self.id),
        );
        let mut cwd_changed = false;
        if let Some(cwd) = events.cwd {
            cwd_changed = self.cwd.as_deref() != Some(cwd.as_path());
            self.cwd = Some(cwd);
        }
        if !self.pty_replies.borrow().is_empty() {
            let replies = std::mem::take(&mut *self.pty_replies.borrow_mut());
            self.pty.write_all(&replies)?;
        }

        Ok(PaneDrain {
            cwd_changed,
            notifications: events.notifications,
        })
    }

    fn write_all(&mut self, bytes: &[u8]) -> Result<()> {
        self.pty.write_all(bytes)
    }

    fn handle_mouse_wheel(&mut self) {
        handle_mouse_wheel(
            &mut self.terminal,
            &mut self.smooth_scroll,
            &mut self.last_wheel_at,
        );
    }

    fn has_running_agent(&self) -> bool {
        should_show_agent_spinner(
            self.agent_status.has_status(),
            self.agent_status.is_running(),
            self.agent_monitor.is_busy(),
        )
    }

    fn frame(
        &mut self,
        dt: f32,
        debug_scroll: bool,
        agent_min_busy: Duration,
        tab_title: &str,
        status_files: bool,
    ) -> Result<(
        TerminalFrame,
        Option<AnimatedCursor>,
        Vec<DesktopNotification>,
    )> {
        if self
            .last_wheel_at
            .is_some_and(|at| at.elapsed() >= WHEEL_SETTLE_AFTER)
        {
            self.smooth_scroll.settle_fractional_offset();
            self.last_wheel_at = None;
        }

        self.smooth_scroll.update(dt);

        let mut frame = self.renderer.collect(&mut self.terminal)?;
        frame.images = self.image_textures.frame_images(&self.graphics);
        let animated_cursor = self.cursor_motion.update(frame.cursor, dt);

        if self.last_wheel_at.is_none()
            && let (Some(previous_rows), Some(previous_scrollbar)) =
                (self.previous_rows.as_deref(), self.previous_scrollbar)
        {
            let shifted_rows = detect_output_scroll_rows(
                previous_rows,
                previous_scrollbar,
                &frame.rows,
                frame.scrollbar,
            );
            if shifted_rows != 0 {
                if debug_scroll {
                    eprintln!(
                        "nvterm scroll: pane={:?} output-shift rows={shifted_rows}",
                        self.id
                    );
                }
                self.smooth_scroll.on_screen_shift(shifted_rows);
            }
        }

        self.previous_rows = Some(frame.rows.clone());
        self.previous_scrollbar = Some(frame.scrollbar);
        let mut notifications = Vec::new();
        if status_files && let Some(notification) = self.agent_status.update(tab_title, self.id) {
            notifications.push(notification);
        }
        if let Some(notification) =
            self.agent_monitor
                .update(&frame, Instant::now(), agent_min_busy, tab_title, self.id)
        {
            notifications.push(notification);
        }
        Ok((frame, animated_cursor, notifications))
    }

    fn close(&mut self) {
        let _ = self.pty.kill();
    }
}

struct TerminalTab {
    title: String,
    panes: Vec<TerminalPane>,
    layout: PaneLayout,
    active_pane: PaneId,
    theme_index: usize,
}

impl TerminalTab {
    fn new(title: String, pane: TerminalPane, theme_index: usize) -> Self {
        let active_pane = pane.id;
        Self {
            title,
            panes: vec![pane],
            layout: PaneLayout::Leaf(active_pane),
            active_pane,
            theme_index,
        }
    }

    fn theme(&self) -> TerminalTheme {
        THEMES[self.theme_index]
    }

    fn active_pane_mut(&mut self) -> Option<&mut TerminalPane> {
        self.pane_mut(self.active_pane)
    }

    fn active_pane_cwd(&self) -> Option<PathBuf> {
        self.panes
            .iter()
            .find(|pane| pane.id == self.active_pane)
            .and_then(|pane| pane.cwd.clone())
    }

    fn session_state(&self) -> SessionTabState {
        SessionTabState {
            title: self.title.clone(),
            active_pane: self.active_pane.0,
            theme: self.theme().name.to_owned(),
            panes: self
                .panes
                .iter()
                .map(|pane| SessionPaneState {
                    id: pane.id.0,
                    cwd: pane.cwd.clone(),
                })
                .collect(),
            layout: self.layout.to_stored(),
        }
    }

    fn pane_mut(&mut self, pane_id: PaneId) -> Option<&mut TerminalPane> {
        self.panes.iter_mut().find(|pane| pane.id == pane_id)
    }

    fn pane_placements(&self, rect: Rect, metrics: CellMetrics) -> Vec<PanePlacement> {
        let mut rects = Vec::new();
        self.layout.collect_with(rect, &split_rect, &mut rects);
        rects
            .into_iter()
            .map(|(id, rect)| PanePlacement {
                id,
                rect,
                viewport: Viewport::from_rect(metrics, rect),
            })
            .collect()
    }

    fn resize_panes(&mut self, rect: Rect, metrics: CellMetrics) -> Result<()> {
        for placement in self.pane_placements(rect, metrics) {
            if let Some(pane) = self.pane_mut(placement.id) {
                pane.resize(placement.viewport)?;
            }
        }
        Ok(())
    }

    fn split_active(&mut self, pane: TerminalPane, axis: SplitAxis) {
        let new_id = pane.id;
        if self.layout.split_leaf(self.active_pane, new_id, axis) {
            self.panes.push(pane);
            self.active_pane = new_id;
        }
    }

    fn set_active_at(&mut self, pos: Vec2, rect: Rect, metrics: CellMetrics) {
        for placement in self.pane_placements(rect, metrics) {
            if rect_contains(placement.rect, pos) {
                self.active_pane = placement.id;
                return;
            }
        }
    }

    fn remove_pane(&mut self, pane_id: PaneId, kill: bool) {
        if let Some(idx) = self.panes.iter().position(|pane| pane.id == pane_id) {
            if kill {
                self.panes[idx].close();
            }
            self.panes.remove(idx);
        }

        let fallback = self
            .panes
            .first()
            .map(|pane| pane.id)
            .unwrap_or(self.active_pane);
        let old_layout = std::mem::replace(&mut self.layout, PaneLayout::Leaf(fallback));
        if let Some(layout) = old_layout.without_leaf(pane_id) {
            self.layout = layout;
        }
        if !self.panes.iter().any(|pane| pane.id == self.active_pane) {
            self.active_pane = self.layout.first_leaf().unwrap_or(fallback);
        }
    }

    fn close_all(&mut self) {
        for pane in &mut self.panes {
            pane.close();
        }
    }

    fn exited_panes(&mut self) -> Result<Vec<PaneId>> {
        let mut exited = Vec::new();
        for pane in &mut self.panes {
            if pane.pty.has_exited()? {
                exited.push(pane.id);
            }
        }
        Ok(exited)
    }

    fn apply_theme(&mut self) -> Result<()> {
        let theme = self.theme();
        for pane in &mut self.panes {
            pane.apply_theme(theme)?;
        }
        Ok(())
    }

    fn has_running_agent(&self) -> bool {
        self.panes.iter().any(TerminalPane::has_running_agent)
    }
}

#[derive(Clone, Debug)]
enum InputMode {
    Terminal,
    Rename {
        buffer: String,
        replace_on_type: bool,
    },
    Keybindings {
        capture: Option<AppCommand>,
    },
}

#[derive(Clone, Copy, Debug)]
struct TabContextMenu {
    tab_index: usize,
    pos: Vec2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TabMenuAction {
    Rename,
    Theme(usize),
}

struct AppState {
    tabs: Vec<TerminalTab>,
    active_tab: usize,
    next_tab_id: usize,
    next_pane_id: usize,
    input_mode: InputMode,
    tab_menu: Option<TabContextMenu>,
    keybindings: Vec<KeyBinding>,
    default_theme_index: usize,
    notifications: NotificationSettings,
    session_path: Option<PathBuf>,
    agent_status_dir: Option<PathBuf>,
    session_dirty: bool,
}

impl AppState {
    fn new(
        content_rect: Rect,
        metrics: CellMetrics,
        config: &AppConfig,
        saved_session: Option<&SessionState>,
        session_path: Option<PathBuf>,
        agent_status_dir: Option<PathBuf>,
    ) -> Result<Self> {
        let mut state = Self {
            tabs: Vec::new(),
            active_tab: 0,
            next_tab_id: 1,
            next_pane_id: 1,
            input_mode: InputMode::Terminal,
            tab_menu: None,
            keybindings: configured_keybindings(config)?,
            default_theme_index: configured_theme_index(config),
            notifications: NotificationSettings::from_config(config),
            session_path,
            agent_status_dir,
            session_dirty: true,
        };
        if let Some(session) = saved_session
            && state.restore_session(session, content_rect, metrics)?
        {
            return Ok(state);
        }
        state.new_tab(content_rect, metrics)?;
        Ok(state)
    }

    fn is_renaming(&self) -> bool {
        matches!(self.input_mode, InputMode::Rename { .. })
    }

    fn input_context(&self) -> InputContext {
        match self.input_mode {
            InputMode::Terminal => InputContext::Terminal,
            InputMode::Rename { .. } => InputContext::Rename,
            InputMode::Keybindings { capture: Some(_) } => InputContext::KeybindingCapture,
            InputMode::Keybindings { capture: None } => InputContext::Keybindings,
        }
    }

    fn is_keybindings_open(&self) -> bool {
        matches!(self.input_mode, InputMode::Keybindings { .. })
    }

    fn active_tab(&self) -> &TerminalTab {
        &self.tabs[self.active_tab]
    }

    fn active_tab_mut(&mut self) -> &mut TerminalTab {
        &mut self.tabs[self.active_tab]
    }

    fn alloc_pane_id(&mut self) -> PaneId {
        let id = PaneId(self.next_pane_id);
        self.next_pane_id += 1;
        id
    }

    fn mark_session_dirty(&mut self) {
        self.session_dirty = true;
    }

    fn save_session_if_dirty(&mut self) -> Result<()> {
        if !self.session_dirty {
            return Ok(());
        }

        self.session_state().save(self.session_path.as_deref())?;
        self.session_dirty = false;
        Ok(())
    }

    fn session_state(&self) -> SessionState {
        SessionState {
            active_tab: self.active_tab.min(self.tabs.len().saturating_sub(1)),
            tabs: self.tabs.iter().map(TerminalTab::session_state).collect(),
        }
    }

    fn restore_session(
        &mut self,
        session: &SessionState,
        content_rect: Rect,
        metrics: CellMetrics,
    ) -> Result<bool> {
        for tab in &session.tabs {
            if let Some(tab) = self.restore_tab(tab, content_rect, metrics)? {
                self.tabs.push(tab);
            }
        }

        if self.tabs.is_empty() {
            return Ok(false);
        }

        self.active_tab = session.active_tab.min(self.tabs.len() - 1);
        self.next_pane_id = self
            .tabs
            .iter()
            .flat_map(|tab| tab.panes.iter().map(|pane| pane.id.0))
            .max()
            .unwrap_or(0)
            + 1;
        self.next_tab_id = self
            .tabs
            .iter()
            .filter_map(|tab| session_title_number(&tab.title))
            .max()
            .unwrap_or(self.tabs.len())
            + 1;
        self.input_mode = InputMode::Terminal;
        Ok(true)
    }

    fn restore_tab(
        &self,
        state: &SessionTabState,
        content_rect: Rect,
        metrics: CellMetrics,
    ) -> Result<Option<TerminalTab>> {
        let theme_index = theme_index_by_name(&state.theme).unwrap_or(self.default_theme_index);
        let theme = THEMES[theme_index];
        let mut panes = Vec::new();
        let viewport = Viewport::from_rect(metrics, content_rect);

        for pane in &state.panes {
            let id = PaneId(pane.id);
            if id.0 == 0
                || panes
                    .iter()
                    .any(|existing: &TerminalPane| existing.id == id)
            {
                continue;
            }

            panes.push(TerminalPane::new(
                id,
                viewport,
                theme,
                pane.cwd.clone(),
                self.agent_status_dir.as_deref(),
            )?);
        }

        let Some(first_pane) = panes.first().map(|pane| pane.id) else {
            return Ok(None);
        };

        let pane_ids = panes.iter().map(|pane| pane.id).collect::<Vec<_>>();
        let mut layout = state.layout.to_runtime();
        if !layout.contains_only(&pane_ids) {
            layout = PaneLayout::Leaf(first_pane);
        }
        let active_pane = PaneId(state.active_pane);
        let active_pane = if pane_ids.contains(&active_pane) {
            active_pane
        } else {
            layout.first_leaf().unwrap_or(first_pane)
        };

        let mut tab = TerminalTab {
            title: state.title.clone(),
            panes,
            layout,
            active_pane,
            theme_index,
        };
        tab.resize_panes(content_rect, metrics)?;
        Ok(Some(tab))
    }

    fn new_tab(&mut self, content_rect: Rect, metrics: CellMetrics) -> Result<()> {
        let tab_id = self.next_tab_id;
        self.next_tab_id += 1;
        let pane_id = self.alloc_pane_id();
        let theme_index = if self.tabs.is_empty() {
            self.default_theme_index
        } else {
            self.active_tab().theme_index
        };
        let cwd = if self.tabs.is_empty() {
            None
        } else {
            self.active_tab().active_pane_cwd()
        };
        let viewport = Viewport::from_rect(metrics, content_rect);
        let pane = TerminalPane::new(
            pane_id,
            viewport,
            THEMES[theme_index],
            cwd,
            self.agent_status_dir.as_deref(),
        )?;
        let tab = TerminalTab::new(format!("session {tab_id}"), pane, theme_index);
        self.tabs.push(tab);
        self.active_tab = self.tabs.len() - 1;
        self.input_mode = InputMode::Terminal;
        self.mark_session_dirty();
        Ok(())
    }

    fn split_active(
        &mut self,
        axis: SplitAxis,
        content_rect: Rect,
        metrics: CellMetrics,
    ) -> Result<()> {
        let pane_id = self.alloc_pane_id();
        let theme = self.active_tab().theme();
        let cwd = self.active_tab().active_pane_cwd();
        let viewport = Viewport::from_rect(metrics, content_rect);
        let pane = TerminalPane::new(
            pane_id,
            viewport,
            theme,
            cwd,
            self.agent_status_dir.as_deref(),
        )?;
        self.active_tab_mut().split_active(pane, axis);
        self.active_tab_mut().resize_panes(content_rect, metrics)?;
        self.mark_session_dirty();
        Ok(())
    }

    fn resize_all(&mut self, content_rect: Rect, metrics: CellMetrics) -> Result<()> {
        for tab in &mut self.tabs {
            tab.resize_panes(content_rect, metrics)?;
        }
        Ok(())
    }

    fn drain(&mut self, debug_pty: bool) -> Result<bool> {
        let mut changed = false;
        let mut notifications = Vec::new();
        for tab in &mut self.tabs {
            for pane in &mut tab.panes {
                let drain = pane.drain(debug_pty)?;
                changed |= drain.cwd_changed;
                notifications.extend(drain.notifications);
            }
        }

        if self.notifications.osc {
            for notification in notifications {
                send_desktop_notification(&notification);
            }
        }

        for tab_idx in (0..self.tabs.len()).rev() {
            let exited = self.tabs[tab_idx].exited_panes()?;
            for pane_id in exited {
                changed = true;
                if self.tabs[tab_idx].panes.len() <= 1 {
                    self.tabs.remove(tab_idx);
                    break;
                }
                self.tabs[tab_idx].remove_pane(pane_id, false);
            }
        }

        if self.tabs.is_empty() {
            return Ok(true);
        }
        self.active_tab = self.active_tab.min(self.tabs.len() - 1);
        if changed {
            self.mark_session_dirty();
        }
        Ok(false)
    }

    fn handle_input(
        &mut self,
        input: TerminalInput,
        content_rect: Rect,
        metrics: CellMetrics,
    ) -> Result<bool> {
        if self.handle_keybinding_capture(&input) {
            return Ok(false);
        }

        if self.handle_tab_menu_input(&input) {
            return Ok(false);
        }

        if self.is_renaming() && self.handle_rename_input(&input) {
            return Ok(false);
        }

        let mut started_rename = false;
        for &command in &input.commands {
            if command == AppCommand::RenameSession {
                started_rename = true;
            }
            if self.handle_command(command, content_rect, metrics)? {
                return Ok(true);
            }
        }

        if started_rename && self.handle_rename_input(&input) {
            return Ok(false);
        }

        if !input.bytes.is_empty()
            && let Some(pane) = self.active_tab_mut().active_pane_mut()
        {
            pane.write_all(&input.bytes)?;
        }

        Ok(false)
    }

    fn handle_keybinding_capture(&mut self, input: &TerminalInput) -> bool {
        let InputMode::Keybindings {
            capture: Some(command),
        } = self.input_mode
        else {
            return false;
        };

        if input.binding_capture_cancelled {
            self.input_mode = InputMode::Keybindings { capture: None };
            return true;
        }

        if let Some(chord) = input.captured_chord {
            self.set_keybinding(command, chord);
            self.input_mode = InputMode::Keybindings { capture: None };
            return true;
        }

        true
    }

    fn set_keybinding(&mut self, command: AppCommand, chord: KeyChord) {
        set_keybinding(&mut self.keybindings, command, chord);
    }

    fn handle_tab_menu_input(&mut self, input: &TerminalInput) -> bool {
        if self.tab_menu.is_none() {
            return false;
        }

        if input.bytes == b"\x1b" {
            self.tab_menu = None;
            return true;
        }

        false
    }

    fn begin_rename_active_tab(&mut self) {
        self.begin_rename_tab(self.active_tab);
    }

    fn begin_rename_tab(&mut self, tab_idx: usize) {
        let Some(tab) = self.tabs.get(tab_idx) else {
            return;
        };
        let title = tab.title.clone();
        self.active_tab = tab_idx;
        self.tab_menu = None;
        self.input_mode = InputMode::Rename {
            buffer: title,
            replace_on_type: true,
        };
    }

    fn handle_rename_input(&mut self, input: &TerminalInput) -> bool {
        let InputMode::Rename {
            buffer,
            replace_on_type,
        } = &mut self.input_mode
        else {
            return false;
        };

        let mut committed_title = None;
        let mut finished = false;

        for event in &input.text_events {
            match event {
                TextInputEvent::Char(ch) if !ch.is_control() => {
                    if *replace_on_type {
                        buffer.clear();
                        *replace_on_type = false;
                    }
                    buffer.push(*ch);
                }
                TextInputEvent::Char(_) => {}
                TextInputEvent::Edit(TextEdit::Commit) => {
                    let title = buffer.trim();
                    committed_title = (!title.is_empty()).then(|| title.to_owned());
                    finished = true;
                    break;
                }
                TextInputEvent::Edit(TextEdit::Cancel) => {
                    finished = true;
                    break;
                }
                TextInputEvent::Edit(TextEdit::Backspace) => {
                    if *replace_on_type {
                        buffer.clear();
                        *replace_on_type = false;
                    } else {
                        buffer.pop();
                    }
                }
            }
        }

        if let Some(title) = committed_title {
            self.active_tab_mut().title = title;
            self.mark_session_dirty();
        }
        if finished {
            self.input_mode = InputMode::Terminal;
        }

        true
    }

    fn handle_command(
        &mut self,
        command: AppCommand,
        content_rect: Rect,
        metrics: CellMetrics,
    ) -> Result<bool> {
        self.tab_menu = None;
        match command {
            AppCommand::NewTab => self.new_tab(content_rect, metrics)?,
            AppCommand::SplitVertical => {
                self.split_active(SplitAxis::Vertical, content_rect, metrics)?
            }
            AppCommand::SplitHorizontal => {
                self.split_active(SplitAxis::Horizontal, content_rect, metrics)?
            }
            AppCommand::NextTab => self.next_tab(),
            AppCommand::PreviousTab => self.previous_tab(),
            AppCommand::CloseActive => return Ok(self.close_active()),
            AppCommand::RenameSession => self.begin_rename_active_tab(),
            AppCommand::CycleTheme => self.cycle_theme()?,
            AppCommand::ShowKeybindings => {
                self.input_mode = if self.is_keybindings_open() {
                    InputMode::Terminal
                } else {
                    InputMode::Keybindings { capture: None }
                };
            }
            AppCommand::DismissOverlay => {
                if matches!(
                    self.input_mode,
                    InputMode::Rename { .. } | InputMode::Keybindings { .. }
                ) {
                    self.input_mode = InputMode::Terminal;
                }
            }
        }

        Ok(false)
    }

    fn next_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.active_tab = (self.active_tab + 1) % self.tabs.len();
            self.mark_session_dirty();
        }
    }

    fn previous_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.active_tab = if self.active_tab == 0 {
                self.tabs.len() - 1
            } else {
                self.active_tab - 1
            };
            self.mark_session_dirty();
        }
    }

    fn close_active(&mut self) -> bool {
        if self.tabs.is_empty() {
            return true;
        }

        let active_tab = self.active_tab;
        if self.tabs[active_tab].panes.len() <= 1 {
            self.tabs[active_tab].close_all();
            self.tabs.remove(active_tab);
            if self.tabs.is_empty() {
                return true;
            }
            self.active_tab = active_tab.min(self.tabs.len() - 1);
            self.input_mode = InputMode::Terminal;
            self.mark_session_dirty();
            return false;
        }

        let pane_id = self.tabs[active_tab].active_pane;
        self.tabs[active_tab].remove_pane(pane_id, true);
        self.mark_session_dirty();
        false
    }

    fn cycle_theme(&mut self) -> Result<()> {
        let theme_index = (self.active_tab().theme_index + 1) % THEMES.len();
        self.set_tab_theme(self.active_tab, theme_index)
    }

    fn set_tab_theme(&mut self, tab_idx: usize, theme_index: usize) -> Result<()> {
        let Some(tab) = self.tabs.get_mut(tab_idx) else {
            return Ok(());
        };
        if theme_index >= THEMES.len() {
            return Ok(());
        }

        if tab.theme_index != theme_index {
            tab.theme_index = theme_index;
            tab.apply_theme()?;
            self.mark_session_dirty();
        }
        Ok(())
    }

    fn open_tab_context_menu(&mut self, tab_idx: usize, pos: Vec2) {
        if tab_idx >= self.tabs.len() {
            return;
        }
        if self.active_tab != tab_idx {
            self.active_tab = tab_idx;
            self.mark_session_dirty();
        }
        self.input_mode = InputMode::Terminal;
        self.tab_menu = Some(TabContextMenu {
            tab_index: tab_idx,
            pos,
        });
    }

    fn handle_mouse(&mut self, content_rect: Rect, metrics: CellMetrics) -> Result<()> {
        if self.handle_keybindings_mouse() {
            return Ok(());
        }

        if self.handle_tab_context_menu_mouse()? {
            return Ok(());
        }

        if is_mouse_button_pressed(MouseButton::Right) {
            let (x, y) = mouse_position();
            let pos = vec2(x, y);
            if y < TAB_BAR_HEIGHT
                && let Some(tab_idx) = tab_index_at(pos, self.tabs.len())
            {
                self.open_tab_context_menu(tab_idx, pos);
                return Ok(());
            }
            self.tab_menu = None;
        }

        if is_mouse_button_pressed(MouseButton::Left) {
            let (x, y) = mouse_position();
            let pos = vec2(x, y);
            if y < TAB_BAR_HEIGHT {
                if let Some(tab_idx) = tab_index_at(pos, self.tabs.len()) {
                    self.tab_menu = None;
                    if tab_idx != self.active_tab {
                        self.active_tab = tab_idx;
                        self.input_mode = InputMode::Terminal;
                        self.mark_session_dirty();
                    }
                }
            } else {
                let previous = self.active_tab().active_pane;
                self.active_tab_mut()
                    .set_active_at(pos, content_rect, metrics);
                if self.active_tab().active_pane != previous {
                    self.mark_session_dirty();
                }
            }
        }

        let (_, wheel_y) = mouse_wheel();
        if wheel_y != 0.0 {
            let (x, y) = mouse_position();
            let pos = vec2(x, y);
            let previous = self.active_tab().active_pane;
            self.active_tab_mut()
                .set_active_at(pos, content_rect, metrics);
            if self.active_tab().active_pane != previous {
                self.mark_session_dirty();
            }
            if let Some(pane) = self.active_tab_mut().active_pane_mut() {
                pane.handle_mouse_wheel();
            }
        }

        Ok(())
    }

    fn handle_tab_context_menu_mouse(&mut self) -> Result<bool> {
        let Some(menu) = self.tab_menu else {
            return Ok(false);
        };

        if is_mouse_button_pressed(MouseButton::Right) {
            let (x, y) = mouse_position();
            let pos = vec2(x, y);
            if y < TAB_BAR_HEIGHT
                && let Some(tab_idx) = tab_index_at(pos, self.tabs.len())
            {
                self.open_tab_context_menu(tab_idx, pos);
                return Ok(true);
            }
            self.tab_menu = None;
            return Ok(true);
        }

        if !is_mouse_button_pressed(MouseButton::Left) {
            return Ok(false);
        }

        let (x, y) = mouse_position();
        let pos = vec2(x, y);
        let rect = tab_context_menu_rect(menu);
        if let Some(action) = tab_menu_action_at(rect, pos) {
            self.run_tab_menu_action(menu.tab_index, action)?;
            self.tab_menu = None;
            return Ok(true);
        }

        self.tab_menu = None;
        Ok(rect_contains(rect, pos))
    }

    fn run_tab_menu_action(&mut self, tab_idx: usize, action: TabMenuAction) -> Result<()> {
        match action {
            TabMenuAction::Rename => self.begin_rename_tab(tab_idx),
            TabMenuAction::Theme(theme_index) => self.set_tab_theme(tab_idx, theme_index)?,
        }
        Ok(())
    }

    fn handle_keybindings_mouse(&mut self) -> bool {
        if !self.is_keybindings_open() {
            return false;
        }

        if is_mouse_button_pressed(MouseButton::Left) {
            let (x, y) = mouse_position();
            let pos = vec2(x, y);
            let panel = keybindings_panel_rect();

            if let Some(command) = keybinding_command_at(pos) {
                self.input_mode = InputMode::Keybindings {
                    capture: Some(command),
                };
            } else if !rect_contains(panel, pos) {
                self.input_mode = InputMode::Terminal;
            }
        }

        true
    }

    fn draw(
        &mut self,
        fonts: &TerminalFonts,
        content_rect: Rect,
        metrics: CellMetrics,
        dt: f32,
        debug_scroll: bool,
    ) -> Result<()> {
        clear_background(self.active_tab().theme().background.color());
        draw_tab_bar(self, fonts);

        let active_tab = self.active_tab;
        let agent_min_busy = self.notifications.agent_min_busy;
        let agent_notifications = self.notifications.agents;
        let agent_status_files = self.notifications.status_files;
        for tab_idx in 0..self.tabs.len() {
            let tab_title = self.tabs[tab_idx].title.clone();
            let theme = self.tabs[tab_idx].theme();
            let active_pane = self.tabs[tab_idx].active_pane;
            let placements = self.tabs[tab_idx].pane_placements(content_rect, metrics);
            for placement in placements {
                let is_active_tab = tab_idx == active_tab;
                let is_active_pane = is_active_tab && active_pane == placement.id;
                if let Some(pane) = self.tabs[tab_idx].pane_mut(placement.id) {
                    let (frame, animated_cursor, notifications) = pane.frame(
                        dt,
                        debug_scroll && is_active_tab,
                        agent_min_busy,
                        &tab_title,
                        agent_status_files,
                    )?;
                    if agent_notifications {
                        for notification in notifications {
                            send_desktop_notification(&notification);
                        }
                    }
                    if is_active_tab {
                        draw_frame(DrawFrameRequest {
                            frame: &frame,
                            viewport: placement.viewport,
                            rect: placement.rect,
                            fonts,
                            visual_offset_rows: pane.smooth_scroll.visual_offset_rows(),
                            animated_cursor,
                            active: is_active_pane,
                            theme,
                        });
                    }
                }
            }
        }

        draw_rename_overlay(self, fonts);
        draw_tab_context_menu(self, fonts);
        draw_keybindings_overlay(self, fonts);
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
struct TerminalTheme {
    name: &'static str,
    background: ThemeRgb,
    foreground: ThemeRgb,
    cursor: ThemeRgb,
    accent: ThemeRgb,
    inactive: ThemeRgb,
}

#[derive(Clone, Copy, Debug)]
struct ThemeRgb {
    r: u8,
    g: u8,
    b: u8,
}

impl ThemeRgb {
    const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    fn rgb(self) -> RgbColor {
        RgbColor {
            r: self.r,
            g: self.g,
            b: self.b,
        }
    }

    fn color(self) -> Color {
        Color::from_rgba(self.r, self.g, self.b, 255)
    }

    fn alpha(self, alpha: u8) -> Color {
        Color::from_rgba(self.r, self.g, self.b, alpha)
    }
}

const THEMES: [TerminalTheme; 5] = [
    TerminalTheme {
        name: "Graphite",
        background: ThemeRgb::new(0x14, 0x16, 0x1a),
        foreground: ThemeRgb::new(0xe6, 0xe1, 0xd9),
        cursor: ThemeRgb::new(0xff, 0xd0, 0x66),
        accent: ThemeRgb::new(0x7c, 0xb7, 0xff),
        inactive: ThemeRgb::new(0x2b, 0x2f, 0x38),
    },
    TerminalTheme {
        name: "Juniper",
        background: ThemeRgb::new(0x10, 0x18, 0x16),
        foreground: ThemeRgb::new(0xdf, 0xe7, 0xdd),
        cursor: ThemeRgb::new(0x9c, 0xe6, 0xa2),
        accent: ThemeRgb::new(0x6e, 0xd4, 0x8f),
        inactive: ThemeRgb::new(0x25, 0x35, 0x30),
    },
    TerminalTheme {
        name: "Harbor",
        background: ThemeRgb::new(0x11, 0x18, 0x22),
        foreground: ThemeRgb::new(0xe4, 0xe8, 0xee),
        cursor: ThemeRgb::new(0x83, 0xd8, 0xff),
        accent: ThemeRgb::new(0x4e, 0xc2, 0xe8),
        inactive: ThemeRgb::new(0x25, 0x33, 0x42),
    },
    TerminalTheme {
        name: "Rose",
        background: ThemeRgb::new(0x1b, 0x14, 0x17),
        foreground: ThemeRgb::new(0xef, 0xdf, 0xdf),
        cursor: ThemeRgb::new(0xff, 0x9b, 0xad),
        accent: ThemeRgb::new(0xec, 0x7f, 0x95),
        inactive: ThemeRgb::new(0x3a, 0x2a, 0x30),
    },
    TerminalTheme {
        name: "Paper",
        background: ThemeRgb::new(0xf1, 0xef, 0xe8),
        foreground: ThemeRgb::new(0x22, 0x24, 0x28),
        cursor: ThemeRgb::new(0x1e, 0x76, 0xd0),
        accent: ThemeRgb::new(0x2d, 0x7d, 0xb8),
        inactive: ThemeRgb::new(0xd9, 0xd4, 0xc8),
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AppCommand {
    NewTab,
    SplitVertical,
    SplitHorizontal,
    NextTab,
    PreviousTab,
    CloseActive,
    RenameSession,
    CycleTheme,
    ShowKeybindings,
    DismissOverlay,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct KeyBinding {
    command: AppCommand,
    chord: KeyChord,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct KeyChord {
    keycode: KeyCode,
    mods: BindingMods,
}

impl KeyChord {
    fn new(keycode: KeyCode, keymods: KeyMods) -> Self {
        Self {
            keycode,
            mods: BindingMods::from_keymods(keymods),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct BindingMods {
    shift: bool,
    ctrl: bool,
    alt: bool,
    logo: bool,
}

impl BindingMods {
    fn from_keymods(keymods: KeyMods) -> Self {
        Self {
            shift: keymods.shift,
            ctrl: keymods.ctrl,
            alt: keymods.alt,
            logo: keymods.logo,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CommandSpec {
    command: AppCommand,
    title: &'static str,
    detail: &'static str,
}

const COMMAND_SPECS: [CommandSpec; 9] = [
    CommandSpec {
        command: AppCommand::NewTab,
        title: "New tab",
        detail: "Create a fresh shell session",
    },
    CommandSpec {
        command: AppCommand::SplitVertical,
        title: "Split vertical",
        detail: "Open a pane on the right",
    },
    CommandSpec {
        command: AppCommand::SplitHorizontal,
        title: "Split horizontal",
        detail: "Open a pane below",
    },
    CommandSpec {
        command: AppCommand::NextTab,
        title: "Next tab",
        detail: "Move focus to the tab on the right",
    },
    CommandSpec {
        command: AppCommand::PreviousTab,
        title: "Previous tab",
        detail: "Move focus to the tab on the left",
    },
    CommandSpec {
        command: AppCommand::CloseActive,
        title: "Close active",
        detail: "Close the focused pane or tab",
    },
    CommandSpec {
        command: AppCommand::RenameSession,
        title: "Rename session",
        detail: "Edit the active tab title",
    },
    CommandSpec {
        command: AppCommand::CycleTheme,
        title: "Cycle color",
        detail: "Switch the active tab theme",
    },
    CommandSpec {
        command: AppCommand::ShowKeybindings,
        title: "Keybindings",
        detail: "Open this command surface",
    },
];

fn default_keybindings() -> Vec<KeyBinding> {
    vec![
        keybinding(AppCommand::NewTab, KeyCode::T, true, false, false, false),
        keybinding(
            AppCommand::SplitVertical,
            KeyCode::D,
            true,
            false,
            false,
            false,
        ),
        keybinding(
            AppCommand::SplitHorizontal,
            KeyCode::D,
            true,
            true,
            false,
            false,
        ),
        keybinding(
            AppCommand::NextTab,
            KeyCode::RightBracket,
            true,
            true,
            false,
            false,
        ),
        keybinding(
            AppCommand::PreviousTab,
            KeyCode::LeftBracket,
            true,
            true,
            false,
            false,
        ),
        keybinding(
            AppCommand::CloseActive,
            KeyCode::W,
            true,
            false,
            false,
            false,
        ),
        keybinding(
            AppCommand::RenameSession,
            KeyCode::R,
            true,
            false,
            false,
            false,
        ),
        keybinding(
            AppCommand::CycleTheme,
            KeyCode::K,
            true,
            false,
            false,
            false,
        ),
        keybinding(
            AppCommand::ShowKeybindings,
            KeyCode::Comma,
            true,
            false,
            false,
            false,
        ),
    ]
}

fn configured_keybindings(config: &AppConfig) -> Result<Vec<KeyBinding>> {
    let mut keybindings = default_keybindings();

    for key in config.keybindings.keys() {
        app_command_from_config_key(key)
            .ok_or_else(|| anyhow!("unknown keybinding action `{key}`"))?;
    }

    for spec in COMMAND_SPECS {
        let key = config_key_for_command(spec.command);
        let Some(value) = config.keybindings.get(key) else {
            continue;
        };
        let chord = parse_key_chord(value)
            .with_context(|| format!("invalid keybinding `{key}` = `{value}`"))?;
        set_keybinding(&mut keybindings, spec.command, chord);
    }

    Ok(keybindings)
}

fn configured_theme_index(config: &AppConfig) -> usize {
    let Some(theme_name) = config.ui.theme.as_deref() else {
        return 0;
    };

    theme_index_by_name(theme_name).unwrap_or(0)
}

fn theme_index_by_name(theme_name: &str) -> Option<usize> {
    THEMES
        .iter()
        .position(|theme| theme.name.eq_ignore_ascii_case(theme_name))
}

fn session_title_number(title: &str) -> Option<usize> {
    title.strip_prefix("session ")?.parse().ok()
}

fn set_keybinding(keybindings: &mut Vec<KeyBinding>, command: AppCommand, chord: KeyChord) {
    keybindings.retain(|binding| binding.command != command && binding.chord != chord);
    keybindings.push(KeyBinding { command, chord });
}

fn keybinding(
    command: AppCommand,
    keycode: KeyCode,
    logo: bool,
    shift: bool,
    ctrl: bool,
    alt: bool,
) -> KeyBinding {
    KeyBinding {
        command,
        chord: KeyChord {
            keycode,
            mods: BindingMods {
                shift,
                ctrl,
                alt,
                logo,
            },
        },
    }
}

fn app_command_from_config_key(key: &str) -> Option<AppCommand> {
    match key {
        "new_tab" => Some(AppCommand::NewTab),
        "split_vertical" => Some(AppCommand::SplitVertical),
        "split_horizontal" => Some(AppCommand::SplitHorizontal),
        "next_tab" => Some(AppCommand::NextTab),
        "previous_tab" => Some(AppCommand::PreviousTab),
        "close_active" => Some(AppCommand::CloseActive),
        "rename_session" => Some(AppCommand::RenameSession),
        "cycle_theme" => Some(AppCommand::CycleTheme),
        "show_keybindings" => Some(AppCommand::ShowKeybindings),
        _ => None,
    }
}

fn config_key_for_command(command: AppCommand) -> &'static str {
    match command {
        AppCommand::NewTab => "new_tab",
        AppCommand::SplitVertical => "split_vertical",
        AppCommand::SplitHorizontal => "split_horizontal",
        AppCommand::NextTab => "next_tab",
        AppCommand::PreviousTab => "previous_tab",
        AppCommand::CloseActive => "close_active",
        AppCommand::RenameSession => "rename_session",
        AppCommand::CycleTheme => "cycle_theme",
        AppCommand::ShowKeybindings => "show_keybindings",
        AppCommand::DismissOverlay => "dismiss_overlay",
    }
}

fn parse_key_chord(value: &str) -> Result<KeyChord> {
    let mut mods = BindingMods::default();
    let mut keycode = None;

    for raw_part in value.split('+') {
        let part = raw_part.trim();
        if part.is_empty() {
            continue;
        }

        match part.to_ascii_lowercase().as_str() {
            "cmd" | "command" | "logo" | "super" | "meta" => mods.logo = true,
            "shift" => mods.shift = true,
            "ctrl" | "control" => mods.ctrl = true,
            "alt" | "opt" | "option" => mods.alt = true,
            _ => {
                if keycode.is_some() {
                    return Err(anyhow!("multiple non-modifier keys"));
                }
                keycode = Some(parse_keycode(part).ok_or_else(|| anyhow!("unknown key `{part}`"))?);
            }
        }
    }

    Ok(KeyChord {
        keycode: keycode.ok_or_else(|| anyhow!("missing non-modifier key"))?,
        mods,
    })
}

fn parse_keycode(key: &str) -> Option<KeyCode> {
    match key.to_ascii_lowercase().as_str() {
        "space" => Some(KeyCode::Space),
        "," | "comma" => Some(KeyCode::Comma),
        "." | "period" => Some(KeyCode::Period),
        "/" | "slash" => Some(KeyCode::Slash),
        ";" | "semicolon" => Some(KeyCode::Semicolon),
        "'" | "apostrophe" | "quote" => Some(KeyCode::Apostrophe),
        "-" | "minus" => Some(KeyCode::Minus),
        "=" | "equal" | "equals" => Some(KeyCode::Equal),
        "[" | "leftbracket" | "left_bracket" => Some(KeyCode::LeftBracket),
        "]" | "rightbracket" | "right_bracket" => Some(KeyCode::RightBracket),
        "\\" | "backslash" => Some(KeyCode::Backslash),
        "`" | "grave" | "graveaccent" | "grave_accent" => Some(KeyCode::GraveAccent),
        "0" => Some(KeyCode::Key0),
        "1" => Some(KeyCode::Key1),
        "2" => Some(KeyCode::Key2),
        "3" => Some(KeyCode::Key3),
        "4" => Some(KeyCode::Key4),
        "5" => Some(KeyCode::Key5),
        "6" => Some(KeyCode::Key6),
        "7" => Some(KeyCode::Key7),
        "8" => Some(KeyCode::Key8),
        "9" => Some(KeyCode::Key9),
        "a" => Some(KeyCode::A),
        "b" => Some(KeyCode::B),
        "c" => Some(KeyCode::C),
        "d" => Some(KeyCode::D),
        "e" => Some(KeyCode::E),
        "f" => Some(KeyCode::F),
        "g" => Some(KeyCode::G),
        "h" => Some(KeyCode::H),
        "i" => Some(KeyCode::I),
        "j" => Some(KeyCode::J),
        "k" => Some(KeyCode::K),
        "l" => Some(KeyCode::L),
        "m" => Some(KeyCode::M),
        "n" => Some(KeyCode::N),
        "o" => Some(KeyCode::O),
        "p" => Some(KeyCode::P),
        "q" => Some(KeyCode::Q),
        "r" => Some(KeyCode::R),
        "s" => Some(KeyCode::S),
        "t" => Some(KeyCode::T),
        "u" => Some(KeyCode::U),
        "v" => Some(KeyCode::V),
        "w" => Some(KeyCode::W),
        "x" => Some(KeyCode::X),
        "y" => Some(KeyCode::Y),
        "z" => Some(KeyCode::Z),
        "esc" | "escape" => Some(KeyCode::Escape),
        "enter" | "return" => Some(KeyCode::Enter),
        "tab" => Some(KeyCode::Tab),
        "backspace" => Some(KeyCode::Backspace),
        "delete" | "del" => Some(KeyCode::Delete),
        "up" => Some(KeyCode::Up),
        "down" => Some(KeyCode::Down),
        "left" => Some(KeyCode::Left),
        "right" => Some(KeyCode::Right),
        "pageup" | "page_up" => Some(KeyCode::PageUp),
        "pagedown" | "page_down" => Some(KeyCode::PageDown),
        "home" => Some(KeyCode::Home),
        "end" => Some(KeyCode::End),
        _ => None,
    }
}

fn resolve_keybinding(
    keybindings: &[KeyBinding],
    keycode: KeyCode,
    keymods: KeyMods,
) -> Option<AppCommand> {
    let chord = KeyChord::new(keycode, keymods);
    keybindings
        .iter()
        .find(|binding| binding.chord == chord)
        .map(|binding| binding.command)
}

fn binding_for_command(keybindings: &[KeyBinding], command: AppCommand) -> Option<KeyBinding> {
    keybindings
        .iter()
        .copied()
        .find(|binding| binding.command == command)
}

fn format_binding(binding: Option<KeyBinding>) -> String {
    binding
        .map(|binding| format_chord(binding.chord))
        .unwrap_or_else(|| "Unassigned".to_owned())
}

fn format_chord(chord: KeyChord) -> String {
    let mut parts = Vec::new();
    if chord.mods.logo {
        parts.push("Cmd".to_owned());
    }
    if chord.mods.shift {
        parts.push("Shift".to_owned());
    }
    if chord.mods.alt {
        parts.push("Opt".to_owned());
    }
    if chord.mods.ctrl {
        parts.push("Ctrl".to_owned());
    }
    parts.push(keycode_label(chord.keycode).to_owned());
    parts.join("+")
}

fn keycode_label(keycode: KeyCode) -> &'static str {
    match keycode {
        KeyCode::Space => "Space",
        KeyCode::Comma => ",",
        KeyCode::Period => ".",
        KeyCode::Slash => "/",
        KeyCode::Semicolon => ";",
        KeyCode::Apostrophe => "'",
        KeyCode::Minus => "-",
        KeyCode::Equal => "=",
        KeyCode::LeftBracket => "[",
        KeyCode::RightBracket => "]",
        KeyCode::Backslash => "\\",
        KeyCode::GraveAccent => "`",
        KeyCode::Key0 => "0",
        KeyCode::Key1 => "1",
        KeyCode::Key2 => "2",
        KeyCode::Key3 => "3",
        KeyCode::Key4 => "4",
        KeyCode::Key5 => "5",
        KeyCode::Key6 => "6",
        KeyCode::Key7 => "7",
        KeyCode::Key8 => "8",
        KeyCode::Key9 => "9",
        KeyCode::A => "A",
        KeyCode::B => "B",
        KeyCode::C => "C",
        KeyCode::D => "D",
        KeyCode::E => "E",
        KeyCode::F => "F",
        KeyCode::G => "G",
        KeyCode::H => "H",
        KeyCode::I => "I",
        KeyCode::J => "J",
        KeyCode::K => "K",
        KeyCode::L => "L",
        KeyCode::M => "M",
        KeyCode::N => "N",
        KeyCode::O => "O",
        KeyCode::P => "P",
        KeyCode::Q => "Q",
        KeyCode::R => "R",
        KeyCode::S => "S",
        KeyCode::T => "T",
        KeyCode::U => "U",
        KeyCode::V => "V",
        KeyCode::W => "W",
        KeyCode::X => "X",
        KeyCode::Y => "Y",
        KeyCode::Z => "Z",
        KeyCode::Escape => "Esc",
        KeyCode::Enter | KeyCode::KpEnter => "Enter",
        KeyCode::Tab => "Tab",
        KeyCode::Backspace => "Backspace",
        KeyCode::Delete => "Delete",
        KeyCode::Up => "Up",
        KeyCode::Down => "Down",
        KeyCode::Left => "Left",
        KeyCode::Right => "Right",
        KeyCode::PageUp => "PageUp",
        KeyCode::PageDown => "PageDown",
        KeyCode::Home => "Home",
        KeyCode::End => "End",
        _ => "Key",
    }
}

fn is_modifier_key(keycode: KeyCode) -> bool {
    matches!(
        keycode,
        KeyCode::LeftShift
            | KeyCode::RightShift
            | KeyCode::LeftControl
            | KeyCode::RightControl
            | KeyCode::LeftAlt
            | KeyCode::RightAlt
            | KeyCode::LeftSuper
            | KeyCode::RightSuper
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TextEdit {
    Commit,
    Cancel,
    Backspace,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TextInputEvent {
    Char(char),
    Edit(TextEdit),
}

fn content_rect() -> Rect {
    Rect::new(
        0.0,
        TAB_BAR_HEIGHT,
        screen_width().max(1.0),
        (screen_height() - TAB_BAR_HEIGHT).max(1.0),
    )
}

fn split_rect(rect: Rect, axis: SplitAxis) -> (Rect, Rect) {
    match axis {
        SplitAxis::Vertical => {
            let width = ((rect.w - PANE_GAP).max(1.0)) * 0.5;
            (
                Rect::new(rect.x, rect.y, width, rect.h),
                Rect::new(rect.x + width + PANE_GAP, rect.y, width, rect.h),
            )
        }
        SplitAxis::Horizontal => {
            let height = ((rect.h - PANE_GAP).max(1.0)) * 0.5;
            (
                Rect::new(rect.x, rect.y, rect.w, height),
                Rect::new(rect.x, rect.y + height + PANE_GAP, rect.w, height),
            )
        }
    }
}

fn rect_contains(rect: Rect, pos: Vec2) -> bool {
    pos.x >= rect.x && pos.x <= rect.x + rect.w && pos.y >= rect.y && pos.y <= rect.y + rect.h
}

fn tab_width(tab_count: usize) -> f32 {
    if tab_count == 0 {
        return TAB_MAX_WIDTH;
    }
    (screen_width() / tab_count as f32).clamp(TAB_MIN_WIDTH, TAB_MAX_WIDTH)
}

fn tab_rect(index: usize, tab_count: usize) -> Rect {
    let width = tab_width(tab_count);
    Rect::new(index as f32 * width, 0.0, width, TAB_BAR_HEIGHT)
}

fn tab_index_at(pos: Vec2, tab_count: usize) -> Option<usize> {
    (0..tab_count).find(|idx| rect_contains(tab_rect(*idx, tab_count), pos))
}

fn draw_tab_bar(app: &AppState, fonts: &TerminalFonts) {
    let theme = app.active_tab().theme();
    let tab_font = fonts.metrics_font();
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        TAB_BAR_HEIGHT,
        ThemeRgb::new(0x0f, 0x10, 0x13).color(),
    );

    let count = app.tabs.len();
    for (idx, tab) in app.tabs.iter().enumerate() {
        draw_tab_bar_item(
            tab,
            tab_rect(idx, count),
            idx == app.active_tab,
            theme,
            tab_font,
        );
    }

    draw_tab_bar_theme_label(theme, tab_font);
}

fn draw_tab_bar_item(
    tab: &TerminalTab,
    rect: Rect,
    selected: bool,
    theme: TerminalTheme,
    font: Option<&Font>,
) {
    let bg = if selected {
        theme.background.color()
    } else {
        theme.inactive.alpha(190)
    };
    draw_rectangle(rect.x, rect.y, rect.w - 1.0, rect.h, bg);
    if selected {
        draw_rectangle(
            rect.x,
            rect.y + rect.h - 3.0,
            rect.w - 1.0,
            3.0,
            theme.accent.color(),
        );
    }

    let running_agent = tab.has_running_agent();
    let label_x = rect.x + if running_agent { 34.0 } else { 12.0 };
    set_scissor(Some(rect));
    if running_agent {
        draw_tab_agent_spinner(rect, selected, theme);
    }
    draw_text_ex(
        tab_label(&tab.title),
        label_x,
        rect.y + 22.0,
        TextParams {
            font,
            font_size: 15,
            color: if selected {
                theme.foreground.color()
            } else {
                Color::from_rgba(190, 194, 202, 255)
            },
            ..Default::default()
        },
    );
    set_scissor(None);
}

fn draw_tab_agent_spinner(rect: Rect, selected: bool, theme: TerminalTheme) {
    draw_loading_spinner(
        vec2(rect.x + 18.0, rect.y + 16.0),
        5.2,
        get_time() as f32 * 5.2,
        if selected {
            theme.accent.color()
        } else {
            Color::from_rgba(170, 198, 220, 255)
        },
    );
}

fn draw_tab_bar_theme_label(theme: TerminalTheme, font: Option<&Font>) {
    let measured = measure_text(theme.name, font, 14, 1.0);
    draw_text_ex(
        theme.name,
        screen_width() - measured.width - 14.0,
        22.0,
        TextParams {
            font,
            font_size: 14,
            color: theme.accent.color(),
            ..Default::default()
        },
    );
}

fn tab_label(title: &str) -> &str {
    title
}

fn tab_menu_item_count() -> usize {
    1 + THEMES.len()
}

fn tab_menu_height() -> f32 {
    TAB_MENU_PADDING * 2.0
        + TAB_MENU_ROW_HEIGHT * tab_menu_item_count() as f32
        + TAB_MENU_SEPARATOR_HEIGHT
}

fn tab_context_menu_rect(menu: TabContextMenu) -> Rect {
    let height = tab_menu_height();
    let min_x = 6.0;
    let min_y = TAB_BAR_HEIGHT + 2.0;
    let max_x = (screen_width() - TAB_MENU_WIDTH - 6.0).max(min_x);
    let max_y = (screen_height() - height - 6.0).max(min_y);
    Rect::new(
        menu.pos.x.clamp(min_x, max_x),
        menu.pos.y.max(min_y).min(max_y),
        TAB_MENU_WIDTH,
        height,
    )
}

fn tab_menu_item_rect(menu_rect: Rect, index: usize) -> Rect {
    let separator_offset = if index > 0 {
        TAB_MENU_SEPARATOR_HEIGHT
    } else {
        0.0
    };
    Rect::new(
        menu_rect.x + TAB_MENU_PADDING,
        menu_rect.y + TAB_MENU_PADDING + index as f32 * TAB_MENU_ROW_HEIGHT + separator_offset,
        menu_rect.w - TAB_MENU_PADDING * 2.0,
        TAB_MENU_ROW_HEIGHT,
    )
}

fn tab_menu_action_for_index(index: usize) -> Option<TabMenuAction> {
    if index == 0 {
        return Some(TabMenuAction::Rename);
    }
    let theme_index = index - 1;
    (theme_index < THEMES.len()).then_some(TabMenuAction::Theme(theme_index))
}

fn tab_menu_action_at(menu_rect: Rect, pos: Vec2) -> Option<TabMenuAction> {
    (0..tab_menu_item_count())
        .find(|idx| rect_contains(tab_menu_item_rect(menu_rect, *idx), pos))
        .and_then(tab_menu_action_for_index)
}

fn draw_tab_context_menu(app: &AppState, fonts: &TerminalFonts) {
    let Some(menu) = app.tab_menu else {
        return;
    };
    let Some(tab) = app.tabs.get(menu.tab_index) else {
        return;
    };

    let theme = app.active_tab().theme();
    let rect = tab_context_menu_rect(menu);
    let mouse = vec2(mouse_position().0, mouse_position().1);
    draw_tab_context_menu_background(rect);

    for idx in 0..tab_menu_item_count() {
        let row = tab_menu_item_rect(rect, idx);
        let hovered = rect_contains(row, mouse);
        let action = tab_menu_action_for_index(idx);
        let selected_theme =
            matches!(action, Some(TabMenuAction::Theme(theme_idx)) if theme_idx == tab.theme_index);
        draw_tab_context_menu_row(row, action, selected_theme, hovered, theme, fonts);
    }
}

fn draw_tab_context_menu_background(rect: Rect) {
    draw_menu_shadow(rect);
    draw_rounded_rect(rect, TAB_MENU_RADIUS, Color::from_rgba(255, 255, 255, 34));
    draw_rounded_rect(
        Rect::new(rect.x + 1.0, rect.y + 1.0, rect.w - 2.0, rect.h - 2.0),
        TAB_MENU_RADIUS - 1.0,
        Color::from_rgba(46, 46, 48, 244),
    );

    let separator_y = tab_menu_item_rect(rect, 0).y + TAB_MENU_ROW_HEIGHT + 3.0;
    draw_rectangle(
        rect.x + 11.0,
        separator_y,
        rect.w - 22.0,
        1.0,
        Color::from_rgba(255, 255, 255, 25),
    );
}

fn draw_tab_context_menu_row(
    row: Rect,
    action: Option<TabMenuAction>,
    selected_theme: bool,
    hovered: bool,
    theme: TerminalTheme,
    fonts: &TerminalFonts,
) {
    if hovered {
        draw_rounded_rect(
            Rect::new(row.x + 2.0, row.y + 1.0, row.w - 4.0, row.h - 2.0),
            5.0,
            Color::from_rgba(0, 122, 255, 230),
        );
    }

    match action {
        Some(TabMenuAction::Rename) => draw_tab_menu_rename_row(row, hovered, fonts),
        Some(TabMenuAction::Theme(theme_index)) => {
            let option = THEMES[theme_index];
            if selected_theme {
                draw_checkmark(
                    vec2(row.x + 12.0, row.y + 9.0),
                    if hovered {
                        Color::from_rgba(255, 255, 255, 245)
                    } else {
                        theme.accent.color()
                    },
                );
            }
            draw_tab_menu_theme_row(row, option, hovered, fonts);
        }
        None => {}
    }
}

fn draw_tab_menu_rename_row(row: Rect, hovered: bool, fonts: &TerminalFonts) {
    draw_text_ex(
        "Rename",
        row.x + 25.0,
        row.y + 18.0,
        TextParams {
            font: fonts.metrics_font(),
            font_size: 13,
            color: menu_text_color(hovered),
            ..Default::default()
        },
    );
}

fn draw_tab_menu_theme_row(row: Rect, option: TerminalTheme, hovered: bool, fonts: &TerminalFonts) {
    draw_rounded_rect(
        Rect::new(row.x + 29.0, row.y + 8.0, 10.0, 10.0),
        2.0,
        option.accent.color(),
    );
    draw_text_ex(
        option.name,
        row.x + 50.0,
        row.y + 18.0,
        TextParams {
            font: fonts.metrics_font(),
            font_size: 13,
            color: menu_text_color(hovered),
            ..Default::default()
        },
    );
}

fn menu_text_color(hovered: bool) -> Color {
    if hovered {
        Color::from_rgba(255, 255, 255, 250)
    } else {
        Color::from_rgba(232, 232, 234, 245)
    }
}

fn draw_menu_shadow(rect: Rect) {
    for idx in 0..4 {
        let spread = idx as f32 * 1.5;
        draw_rounded_rect(
            Rect::new(
                rect.x - spread * 0.5,
                rect.y + 3.0 + spread,
                rect.w + spread,
                rect.h + spread,
            ),
            TAB_MENU_RADIUS + spread * 0.4,
            Color::from_rgba(0, 0, 0, 32u8.saturating_sub(idx as u8 * 6)),
        );
    }
}

fn draw_rounded_rect(rect: Rect, radius: f32, color: Color) {
    let radius = radius.min(rect.w * 0.5).min(rect.h * 0.5).max(0.0);
    draw_rectangle(
        rect.x + radius,
        rect.y,
        rect.w - radius * 2.0,
        rect.h,
        color,
    );
    draw_rectangle(
        rect.x,
        rect.y + radius,
        rect.w,
        rect.h - radius * 2.0,
        color,
    );
    draw_circle(rect.x + radius, rect.y + radius, radius, color);
    draw_circle(rect.x + rect.w - radius, rect.y + radius, radius, color);
    draw_circle(rect.x + radius, rect.y + rect.h - radius, radius, color);
    draw_circle(
        rect.x + rect.w - radius,
        rect.y + rect.h - radius,
        radius,
        color,
    );
}

fn draw_checkmark(pos: Vec2, color: Color) {
    draw_line(pos.x, pos.y + 5.0, pos.x + 3.0, pos.y + 8.0, 1.5, color);
    draw_line(
        pos.x + 3.0,
        pos.y + 8.0,
        pos.x + 9.0,
        pos.y + 1.0,
        1.5,
        color,
    );
}

fn draw_loading_spinner(center: Vec2, radius: f32, phase: f32, color: Color) {
    let segments = 8;
    for idx in 0..segments {
        let t = idx as f32 / segments as f32;
        let angle = phase + t * std::f32::consts::TAU;
        let alpha = 0.20 + t * 0.72;
        let mut segment_color = color;
        segment_color.a *= alpha;
        let inner = radius * 0.48;
        let outer = radius;
        draw_line(
            center.x + angle.cos() * inner,
            center.y + angle.sin() * inner,
            center.x + angle.cos() * outer,
            center.y + angle.sin() * outer,
            1.8,
            segment_color,
        );
    }
}

fn draw_rename_overlay(app: &AppState, fonts: &TerminalFonts) {
    let InputMode::Rename { buffer, .. } = &app.input_mode else {
        return;
    };

    let theme = app.active_tab().theme();
    let width = (screen_width() * 0.46).clamp(300.0, 620.0);
    let rect = Rect::new(
        (screen_width() - width) * 0.5,
        TAB_BAR_HEIGHT + 14.0,
        width,
        42.0,
    );
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        ThemeRgb::new(0x18, 0x1a, 0x20).alpha(245),
    );
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, theme.accent.color());
    set_scissor(Some(rect));
    draw_text_ex(
        buffer,
        rect.x + 14.0,
        rect.y + 27.0,
        TextParams {
            font: fonts.for_text(buffer),
            font_size: 17,
            color: theme.foreground.color(),
            ..Default::default()
        },
    );
    set_scissor(None);
}

fn keybindings_panel_rect() -> Rect {
    let width = screen_width().clamp(620.0, 780.0);
    let row_height = 45.0;
    let height = 98.0 + COMMAND_SPECS.len() as f32 * row_height + 24.0;
    Rect::new(
        (screen_width() - width) * 0.5,
        (screen_height() - height) * 0.5,
        width,
        height,
    )
}

fn keybinding_row_rect(panel: Rect, index: usize) -> Rect {
    Rect::new(
        panel.x + 22.0,
        panel.y + 82.0 + index as f32 * 45.0,
        panel.w - 44.0,
        38.0,
    )
}

fn keybinding_command_at(pos: Vec2) -> Option<AppCommand> {
    let panel = keybindings_panel_rect();
    COMMAND_SPECS
        .iter()
        .enumerate()
        .find(|(idx, _)| rect_contains(keybinding_row_rect(panel, *idx), pos))
        .map(|(_, spec)| spec.command)
}

fn draw_keybindings_overlay(app: &AppState, fonts: &TerminalFonts) {
    let InputMode::Keybindings { capture } = app.input_mode else {
        return;
    };

    let theme = app.active_tab().theme();
    let panel = keybindings_panel_rect();
    draw_keybindings_panel(panel, theme);
    draw_keybindings_header(panel, capture, theme, fonts);

    for (idx, spec) in COMMAND_SPECS.iter().enumerate() {
        draw_keybinding_row(
            app,
            *spec,
            keybinding_row_rect(panel, idx),
            capture,
            theme,
            fonts,
        );
    }
}

fn draw_keybindings_panel(panel: Rect, theme: TerminalTheme) {
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        Color::from_rgba(0, 0, 0, 135),
    );
    draw_rectangle(
        panel.x,
        panel.y,
        panel.w,
        panel.h,
        ThemeRgb::new(0x12, 0x14, 0x19).alpha(246),
    );
    draw_rectangle(panel.x, panel.y, panel.w, 3.0, theme.accent.color());
    draw_rectangle_lines(
        panel.x + 0.5,
        panel.y + 0.5,
        panel.w - 1.0,
        panel.h - 1.0,
        1.0,
        theme.accent.alpha(120),
    );
}

fn draw_keybindings_header(
    panel: Rect,
    capture: Option<AppCommand>,
    theme: TerminalTheme,
    fonts: &TerminalFonts,
) {
    draw_text_ex(
        "Keybindings",
        panel.x + 24.0,
        panel.y + 34.0,
        TextParams {
            font: fonts.metrics_font(),
            font_size: 24,
            color: theme.foreground.color(),
            ..Default::default()
        },
    );

    let subtitle = if capture.is_some() {
        "Press a new shortcut. Esc cancels."
    } else {
        "Click a shortcut to change it. Esc closes."
    };
    draw_text_ex(
        subtitle,
        panel.x + 24.0,
        panel.y + 59.0,
        TextParams {
            font: fonts.metrics_font(),
            font_size: 14,
            color: Color::from_rgba(166, 174, 187, 255),
            ..Default::default()
        },
    );
}

fn draw_keybinding_row(
    app: &AppState,
    spec: CommandSpec,
    row: Rect,
    capture: Option<AppCommand>,
    theme: TerminalTheme,
    fonts: &TerminalFonts,
) {
    let selected = capture == Some(spec.command);
    let hovered = rect_contains(row, vec2(mouse_position().0, mouse_position().1));
    let row_bg = if selected {
        theme.accent.alpha(54)
    } else if hovered {
        theme.inactive.alpha(150)
    } else {
        ThemeRgb::new(0x1b, 0x1f, 0x27).alpha(170)
    };
    draw_rectangle(row.x, row.y, row.w, row.h, row_bg);
    if selected {
        draw_rectangle(row.x, row.y, 3.0, row.h, theme.accent.color());
    }

    draw_keybinding_row_text(spec, row, theme, fonts);
    let binding = if selected {
        "recording...".to_owned()
    } else {
        format_binding(binding_for_command(&app.keybindings, spec.command))
    };
    draw_keybinding_chip(&binding, row, selected, theme, fonts);
}

fn draw_keybinding_row_text(
    spec: CommandSpec,
    row: Rect,
    theme: TerminalTheme,
    fonts: &TerminalFonts,
) {
    draw_text_ex(
        spec.title,
        row.x + 14.0,
        row.y + 17.0,
        TextParams {
            font: fonts.metrics_font(),
            font_size: 15,
            color: theme.foreground.color(),
            ..Default::default()
        },
    );
    draw_text_ex(
        spec.detail,
        row.x + 14.0,
        row.y + 32.0,
        TextParams {
            font: fonts.metrics_font(),
            font_size: 11,
            color: Color::from_rgba(150, 158, 170, 255),
            ..Default::default()
        },
    );
}

fn draw_keybinding_chip(
    binding: &str,
    row: Rect,
    selected: bool,
    theme: TerminalTheme,
    fonts: &TerminalFonts,
) {
    let chip_width = measure_text(binding, fonts.metrics_font(), 14, 1.0).width + 28.0;
    let chip = Rect::new(
        row.x + row.w - chip_width - 10.0,
        row.y + 7.0,
        chip_width,
        24.0,
    );
    draw_rectangle(
        chip.x,
        chip.y,
        chip.w,
        chip.h,
        if selected {
            theme.accent.alpha(120)
        } else {
            ThemeRgb::new(0x0b, 0x0d, 0x11).alpha(220)
        },
    );
    draw_rectangle_lines(chip.x, chip.y, chip.w, chip.h, 1.0, theme.accent.alpha(110));
    draw_text_ex(
        binding,
        chip.x + 14.0,
        chip.y + 17.0,
        TextParams {
            font: fonts.metrics_font(),
            font_size: 14,
            color: if selected {
                theme.foreground.color()
            } else {
                theme.accent.color()
            },
            ..Default::default()
        },
    );
}

fn set_scissor(rect: Option<Rect>) {
    let scale = screen_dpi_scale();
    let clip = rect.map(|rect| {
        (
            (rect.x.max(0.0) * scale).round() as i32,
            (rect.y.max(0.0) * scale).round() as i32,
            (rect.w.max(0.0) * scale).round() as i32,
            (rect.h.max(0.0) * scale).round() as i32,
        )
    });

    // SAFETY: macroquad exposes scissor control only through its internal GL
    // handle. The handle is used immediately on the render thread and is not
    // stored across frames.
    unsafe {
        let mut gl = macroquad::window::get_internal_gl();
        gl.flush();
        gl.quad_gl.scissor(clip);
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct TerminalOscEvents {
    cwd: Option<PathBuf>,
    notifications: Vec<DesktopNotification>,
}

#[derive(Clone, Debug, Default)]
struct OscTracker {
    pending: Vec<u8>,
}

impl OscTracker {
    fn new() -> Self {
        Self::default()
    }

    fn push(&mut self, bytes: &[u8]) -> TerminalOscEvents {
        const MAX_PENDING_OSC_BYTES: usize = 4096;
        self.pending.extend_from_slice(bytes);
        let mut events = TerminalOscEvents::default();

        loop {
            let Some(start) = find_osc_start(&self.pending) else {
                trim_osc_pending(&mut self.pending);
                return events;
            };

            if start > 0 {
                self.pending.drain(..start);
            }

            let payload_start = if self.pending.starts_with(b"\x1b]") {
                2
            } else {
                1
            };
            let Some((payload_end, terminator_len)) = find_osc_end(&self.pending, payload_start)
            else {
                if self.pending.len() > MAX_PENDING_OSC_BYTES {
                    let keep_from = self.pending.len().saturating_sub(MAX_PENDING_OSC_BYTES);
                    self.pending.drain(..keep_from);
                }
                return events;
            };

            if let Some(event) = parse_osc_payload(&self.pending[payload_start..payload_end]) {
                match event {
                    TerminalOscEvent::Cwd(cwd) => events.cwd = Some(cwd),
                    TerminalOscEvent::Notification(notification) => {
                        events.notifications.push(notification);
                    }
                }
            }
            self.pending.drain(..payload_end + terminator_len);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum TerminalOscEvent {
    Cwd(PathBuf),
    Notification(DesktopNotification),
}

fn find_osc_start(bytes: &[u8]) -> Option<usize> {
    let esc = bytes.windows(2).position(|window| window == b"\x1b]");
    let c1 = bytes.iter().position(|byte| *byte == 0x9d);

    match (esc, c1) {
        (Some(esc), Some(c1)) => Some(esc.min(c1)),
        (Some(esc), None) => Some(esc),
        (None, Some(c1)) => Some(c1),
        (None, None) => None,
    }
}

fn find_osc_end(bytes: &[u8], start: usize) -> Option<(usize, usize)> {
    let mut idx = start;
    while idx < bytes.len() {
        if bytes[idx] == 0x07 {
            return Some((idx, 1));
        }
        if idx + 1 < bytes.len() && bytes[idx] == 0x1b && bytes[idx + 1] == b'\\' {
            return Some((idx, 2));
        }
        idx += 1;
    }
    None
}

fn trim_osc_pending(pending: &mut Vec<u8>) {
    if pending.last() == Some(&0x1b) {
        let Some(esc) = pending.pop() else {
            return;
        };
        pending.clear();
        pending.push(esc);
    } else {
        pending.clear();
    }
}

fn parse_osc_payload(payload: &[u8]) -> Option<TerminalOscEvent> {
    let text = std::str::from_utf8(payload).ok()?;
    if let Some(uri) = text.strip_prefix("7;") {
        return parse_file_uri_path(uri).map(TerminalOscEvent::Cwd);
    }
    if let Some(message) = text.strip_prefix("9;") {
        let body = message.trim();
        if !body.is_empty() {
            return Some(TerminalOscEvent::Notification(DesktopNotification {
                title: "neovide-tabs".to_owned(),
                subtitle: None,
                body: body.to_owned(),
            }));
        }
    }
    if let Some(rest) = text.strip_prefix("777;notify;") {
        let mut parts = rest.splitn(2, ';');
        let title = parts.next().unwrap_or("").trim();
        let body = parts.next().unwrap_or("").trim();
        if !title.is_empty() || !body.is_empty() {
            return Some(TerminalOscEvent::Notification(DesktopNotification {
                title: if title.is_empty() {
                    "neovide-tabs".to_owned()
                } else {
                    title.to_owned()
                },
                subtitle: None,
                body: body.to_owned(),
            }));
        }
    }

    None
}

fn parse_file_uri_path(uri: &str) -> Option<PathBuf> {
    let rest = uri.strip_prefix("file://")?;
    let path_start = rest.find('/')?;
    let decoded = percent_decode_utf8(&rest[path_start..])?;
    let path = PathBuf::from(decoded);
    path.is_absolute().then_some(path)
}

fn percent_decode_utf8(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut idx = 0;

    while idx < bytes.len() {
        if bytes[idx] == b'%' {
            let hi = *bytes.get(idx + 1)?;
            let lo = *bytes.get(idx + 2)?;
            decoded.push(hex_value(hi)? << 4 | hex_value(lo)?);
            idx += 3;
        } else {
            decoded.push(bytes[idx]);
            idx += 1;
        }
    }

    String::from_utf8(decoded).ok()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

struct TerminalProtocolTrackers<'a> {
    osc: &'a mut OscTracker,
    kitty: &'a mut KittyGraphicsTracker,
    graphics: &'a mut KittyGraphicsState,
}

fn drain_pty(
    rx: &Receiver<Vec<u8>>,
    terminal: &mut Terminal<'_, '_>,
    pty_replies: &Rc<RefCell<Vec<u8>>>,
    trackers: TerminalProtocolTrackers<'_>,
    debug_pty: bool,
    pane_id: Option<PaneId>,
) -> TerminalOscEvents {
    let mut events = TerminalOscEvents::default();
    while let Ok(bytes) = rx.try_recv() {
        if debug_pty {
            if let Some(pane_id) = pane_id {
                eprintln!("nvterm pty pane={pane_id:?}: {}", debug_bytes(&bytes));
            } else {
                eprintln!("nvterm pty: {}", debug_bytes(&bytes));
            }
        }

        let chunk_events = trackers.osc.push(&bytes);
        if chunk_events.cwd.is_some() {
            events.cwd = chunk_events.cwd;
        }
        events.notifications.extend(chunk_events.notifications);
        for command in trackers.kitty.push(&bytes) {
            let cell = KittyCellPosition {
                col: terminal.cursor_x().unwrap_or(0),
                row: terminal.cursor_y().unwrap_or(0),
            };
            let _ = trackers.graphics.apply_at(command, cell);
        }
        terminal.vt_write(&bytes);

        if !pty_replies.borrow().is_empty() {
            break;
        }
    }

    events
}

fn debug_bytes(bytes: &[u8]) -> String {
    const MAX_DEBUG_BYTES: usize = 240;
    let mut out = String::new();

    for byte in bytes.iter().copied().take(MAX_DEBUG_BYTES) {
        match byte {
            b'\n' => out.push_str("\\n"),
            b'\r' => out.push_str("\\r"),
            b'\t' => out.push_str("\\t"),
            0x20..=0x7e => out.push(byte as char),
            _ => {
                use std::fmt::Write as _;
                let _ = write!(out, "\\x{byte:02x}");
            }
        }
    }

    if bytes.len() > MAX_DEBUG_BYTES {
        out.push_str("...");
    }

    out
}

fn collect_input(
    input_subscriber: usize,
    context: InputContext,
    keybindings: &[KeyBinding],
    debug_input: bool,
) -> TerminalInput {
    let mut input = TerminalInput::new(debug_input, context, keybindings);
    input_utils::repeat_all_miniquad_input(&mut input, input_subscriber);
    clear_input_queue();
    input
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum InputContext {
    #[default]
    Terminal,
    Rename,
    Keybindings,
    KeybindingCapture,
}

#[derive(Debug)]
struct TerminalInput {
    bytes: Vec<u8>,
    commands: Vec<AppCommand>,
    text_chars: Vec<char>,
    text_edits: Vec<TextEdit>,
    text_events: Vec<TextInputEvent>,
    captured_chord: Option<KeyChord>,
    binding_capture_cancelled: bool,
    skip_next_chars: Vec<char>,
    keybindings: Vec<KeyBinding>,
    debug: bool,
    context: InputContext,
}

impl Default for TerminalInput {
    fn default() -> Self {
        Self::new(false, InputContext::Terminal, &default_keybindings())
    }
}

impl TerminalInput {
    fn new(debug: bool, context: InputContext, keybindings: &[KeyBinding]) -> Self {
        Self {
            bytes: Vec::new(),
            commands: Vec::new(),
            text_chars: Vec::new(),
            text_edits: Vec::new(),
            text_events: Vec::new(),
            captured_chord: None,
            binding_capture_cancelled: false,
            skip_next_chars: Vec::new(),
            keybindings: keybindings.to_vec(),
            debug,
            context,
        }
    }

    fn push_bytes(&mut self, label: &str, bytes: &[u8]) {
        if self.debug {
            eprintln!("nvterm input: {label} -> {bytes:?}");
        }

        self.bytes.extend_from_slice(bytes);
    }

    fn push_command(&mut self, command: AppCommand) {
        if self.debug {
            eprintln!("nvterm command: {command:?}");
        }

        self.commands.push(command);
    }

    fn push_text_edit(&mut self, edit: TextEdit) {
        if self.debug {
            eprintln!("nvterm text-edit: {edit:?}");
        }

        self.text_edits.push(edit);
        self.text_events.push(TextInputEvent::Edit(edit));
    }

    fn push_char(&mut self, ch: char) {
        let mut buf = [0; 4];
        self.push_bytes("char", ch.encode_utf8(&mut buf).as_bytes());
    }

    fn skip_char(&mut self, ch: char) {
        self.skip_next_chars.push(ch);
    }

    fn consume_skipped_char(&mut self, ch: char) -> bool {
        if let Some(pos) = self.skip_next_chars.iter().position(|skip| *skip == ch) {
            self.skip_next_chars.remove(pos);
            return true;
        }

        false
    }

    fn handle_keybinding_capture_key(&mut self, keycode: KeyCode, keymods: KeyMods) -> bool {
        if self.context != InputContext::KeybindingCapture {
            return false;
        }
        if keycode == KeyCode::Escape {
            self.binding_capture_cancelled = true;
        } else if !is_modifier_key(keycode) {
            self.captured_chord = Some(KeyChord::new(keycode, keymods));
        }
        true
    }

    fn handle_keybindings_overlay_key(&mut self, keycode: KeyCode, keymods: KeyMods) -> bool {
        if self.context != InputContext::Keybindings {
            return false;
        }
        if keycode == KeyCode::Escape {
            self.push_command(AppCommand::DismissOverlay);
            return true;
        }
        if let Some(command) = resolve_keybinding(&self.keybindings, keycode, keymods) {
            self.push_command(command);
        }
        true
    }

    fn handle_app_command_key(&mut self, keycode: KeyCode, keymods: KeyMods) -> bool {
        let Some(command) = resolve_keybinding(&self.keybindings, keycode, keymods) else {
            return false;
        };
        self.push_command(command);
        if command == AppCommand::RenameSession {
            self.context = InputContext::Rename;
        }
        if command == AppCommand::ShowKeybindings {
            self.context = InputContext::Keybindings;
        }
        true
    }

    fn handle_rename_key(&mut self, keycode: KeyCode) -> bool {
        if self.context != InputContext::Rename {
            return false;
        }
        match keycode {
            KeyCode::Enter | KeyCode::KpEnter => self.push_text_edit(TextEdit::Commit),
            KeyCode::Escape => self.push_text_edit(TextEdit::Cancel),
            KeyCode::Backspace => self.push_text_edit(TextEdit::Backspace),
            _ => {}
        }
        true
    }

    fn handle_terminal_control_key(&mut self, keycode: KeyCode, keymods: KeyMods) -> bool {
        if keycode == KeyCode::Tab {
            self.push_bytes("tab", b"\t");
            self.skip_char('\t');
            return true;
        }
        if keymods.ctrl || keymods.logo {
            return true;
        }

        match keycode {
            KeyCode::Enter | KeyCode::KpEnter => self.push_enter_key(),
            KeyCode::Backspace => self.push_backspace_key(),
            KeyCode::Escape => self.push_escape_key(),
            KeyCode::Up => self.push_bytes("up", b"\x1b[A"),
            KeyCode::Down => self.push_bytes("down", b"\x1b[B"),
            KeyCode::Right => self.push_bytes("right", b"\x1b[C"),
            KeyCode::Left => self.push_bytes("left", b"\x1b[D"),
            KeyCode::Home => self.push_bytes("home", b"\x1b[H"),
            KeyCode::End => self.push_bytes("end", b"\x1b[F"),
            KeyCode::Delete => self.push_bytes("delete", b"\x1b[3~"),
            KeyCode::PageUp => self.push_bytes("page-up", b"\x1b[5~"),
            KeyCode::PageDown => self.push_bytes("page-down", b"\x1b[6~"),
            _ => return false,
        }
        true
    }

    fn push_enter_key(&mut self) {
        self.push_bytes("enter", b"\r");
        self.skip_char('\r');
        self.skip_char('\n');
    }

    fn push_backspace_key(&mut self) {
        self.push_bytes("backspace", b"\x7f");
        self.skip_char('\u{8}');
        self.skip_char('\u{7f}');
    }

    fn push_escape_key(&mut self) {
        self.push_bytes("escape", b"\x1b");
        self.skip_char('\u{1b}');
    }
}

impl EventHandler for TerminalInput {
    fn update(&mut self) {}

    fn draw(&mut self) {}

    fn key_down_event(&mut self, keycode: KeyCode, keymods: KeyMods, _repeat: bool) {
        if self.handle_keybinding_capture_key(keycode, keymods)
            || self.handle_keybindings_overlay_key(keycode, keymods)
            || self.handle_app_command_key(keycode, keymods)
            || self.handle_rename_key(keycode)
        {
            return;
        }
        self.handle_terminal_control_key(keycode, keymods);
    }

    fn char_event(&mut self, character: char, keymods: KeyMods, _repeat: bool) {
        if matches!(
            self.context,
            InputContext::Keybindings | InputContext::KeybindingCapture
        ) {
            return;
        }

        if keymods.logo {
            return;
        }

        if is_appkit_function_key_char(character) {
            return;
        }

        if self.context == InputContext::Rename {
            if !character.is_control() {
                self.text_chars.push(character);
                self.text_events.push(TextInputEvent::Char(character));
            }
            return;
        }

        if self.consume_skipped_char(character) {
            return;
        }

        match character {
            '\t' => self.push_bytes("char-tab", b"\t"),
            '\n' | '\r' => self.push_bytes("char-enter", b"\r"),
            '\u{8}' | '\u{7f}' => self.push_bytes("char-backspace", b"\x7f"),
            '\u{1b}' => self.push_bytes("char-escape", b"\x1b"),
            ch if is_c0_control(ch) => self.push_bytes("char-control", &[ch as u8]),
            ch if !ch.is_control() => self.push_char(ch),
            _ => {}
        }
    }
}

#[cfg_attr(not(test), allow(dead_code))]
fn app_command_for_key(keycode: KeyCode, keymods: KeyMods) -> Option<AppCommand> {
    resolve_keybinding(&default_keybindings(), keycode, keymods)
}

fn is_c0_control(ch: char) -> bool {
    matches!(ch as u32, 0x01..=0x1f)
}

fn is_appkit_function_key_char(ch: char) -> bool {
    matches!(ch as u32, 0xf700..=0xf8ff)
}

fn handle_mouse_wheel(
    terminal: &mut Terminal<'_, '_>,
    smooth_scroll: &mut SmoothScroll,
    last_wheel_at: &mut Option<Instant>,
) {
    let (_, wheel_y) = mouse_wheel();
    if wheel_y == 0.0 {
        return;
    }

    let delta_rows = -wheel_y * SCROLL_ROWS_PER_WHEEL_UNIT;
    let requested_rows = smooth_scroll.consume_history_scroll_request(delta_rows);
    let terminal_rows = bounded_scroll_rows(
        requested_rows,
        terminal.scrollbar().ok().map(|scrollbar| ScrollbarView {
            top: scrollbar.offset,
            visible: scrollbar.len,
            total: scrollbar.total,
        }),
    );

    if terminal_rows != 0 {
        terminal.scroll_viewport(ScrollViewport::Delta(terminal_rows));
        smooth_scroll.animate_history_rows(terminal_rows);
    }
    *last_wheel_at = Some(Instant::now());
}

fn bounded_scroll_rows(requested_rows: isize, scrollbar: Option<ScrollbarView>) -> isize {
    let Some(scrollbar) = scrollbar else {
        return requested_rows;
    };

    if requested_rows == 0 {
        return 0;
    }

    if scrollbar.total <= scrollbar.visible {
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

fn scrollbar_is_at_bottom(scrollbar: ScrollbarView) -> bool {
    scrollbar.total <= scrollbar.visible
        || scrollbar.top.saturating_add(scrollbar.visible) >= scrollbar.total
}

fn detect_output_scroll_rows(
    previous_rows: &[Vec<CellView>],
    previous_scrollbar: ScrollbarView,
    current_rows: &[Vec<CellView>],
    current_scrollbar: ScrollbarView,
) -> isize {
    if !scrollbar_is_at_bottom(current_scrollbar) {
        return 0;
    }

    let row_shift = detect_upward_row_shift(previous_rows, current_rows);
    if row_shift != 0 {
        return row_shift;
    }

    let total_delta = current_scrollbar
        .total
        .saturating_sub(previous_scrollbar.total);
    cap_output_scroll_rows(total_delta, current_scrollbar.visible)
}

fn detect_upward_row_shift(previous: &[Vec<CellView>], current: &[Vec<CellView>]) -> isize {
    if previous.len() != current.len() || previous.len() < 2 {
        return 0;
    }

    if previous == current {
        return 0;
    }

    let max_shift = previous
        .len()
        .saturating_sub(1)
        .min(MAX_OUTPUT_SCROLL_ANIMATION_ROWS);
    for shift in 1..=max_shift {
        let comparable_rows = previous.len() - shift;
        if (0..comparable_rows).all(|row| previous[row + shift] == current[row]) {
            return shift as isize;
        }
    }

    0
}

fn cap_output_scroll_rows(rows: u64, visible_rows: u64) -> isize {
    let capped_rows = if visible_rows > 0 && rows > visible_rows {
        OUTPUT_SCROLL_ANIMATION_FAR_LINES as u64
    } else {
        rows.min(MAX_OUTPUT_SCROLL_ANIMATION_ROWS as u64)
    };

    capped_rows.min(isize::MAX as u64) as isize
}

struct DrawFrameRequest<'a> {
    frame: &'a TerminalFrame,
    viewport: Viewport,
    rect: Rect,
    fonts: &'a TerminalFonts,
    visual_offset_rows: f32,
    animated_cursor: Option<AnimatedCursor>,
    active: bool,
    theme: TerminalTheme,
}

fn draw_frame(request: DrawFrameRequest<'_>) {
    let DrawFrameRequest {
        frame,
        viewport,
        rect,
        fonts,
        visual_offset_rows,
        animated_cursor,
        active,
        theme,
    } = request;

    draw_rectangle(rect.x, rect.y, rect.w, rect.h, frame.background);
    set_scissor(Some(rect));

    let cell_width = viewport.metrics.cell_width;
    let cell_height = viewport.metrics.cell_height;
    let baseline = viewport.metrics.baseline;
    let y_offset = visual_offset_rows * cell_height;

    for (row_idx, row) in frame.rows.iter().enumerate() {
        let y = row_idx as f32 * cell_height + y_offset;
        if y > rect.h || y + cell_height < 0.0 {
            continue;
        }

        for (col_idx, cell) in row.iter().enumerate() {
            let x = col_idx as f32 * cell_width;
            if x > rect.w || x + cell_width < 0.0 {
                continue;
            }
            if let Some(bg) = cell.bg {
                draw_rectangle(
                    rect.x + x,
                    rect.y + y.floor(),
                    cell_width.ceil(),
                    cell_height.ceil(),
                    bg,
                );
            }

            if cell.text.is_empty() {
                continue;
            }

            draw_text_ex(
                &cell.text,
                rect.x + x,
                rect.y + y + baseline,
                TextParams {
                    font: fonts.for_text(&cell.text),
                    font_size: FONT_SIZE,
                    font_scale: 1.0,
                    color: cell.fg,
                    ..Default::default()
                },
            );
        }
    }

    draw_frame_images(frame, viewport, rect, y_offset);

    if let Some(cursor) = animated_cursor {
        draw_cursor(frame, viewport, rect, fonts, cursor, y_offset);
    }

    set_scissor(None);

    if active {
        draw_rectangle_lines(
            rect.x + 0.5,
            rect.y + 0.5,
            rect.w - 1.0,
            rect.h - 1.0,
            1.0,
            theme.accent.alpha(180),
        );
    }

    draw_scroll_indicator(frame, rect);
}

fn draw_frame_images(frame: &TerminalFrame, viewport: Viewport, rect: Rect, y_offset: f32) {
    for image in &frame.images {
        let dest_size = terminal_image_dest_size(
            image.pixel_width,
            image.pixel_height,
            image.columns,
            image.rows,
            viewport.metrics,
        );
        let x = rect.x + image.col as f32 * viewport.metrics.cell_width;
        let y = rect.y + image.row as f32 * viewport.metrics.cell_height + y_offset;

        if x > rect.x + rect.w
            || x + dest_size.x < rect.x
            || y > rect.y + rect.h
            || y + dest_size.y < rect.y
        {
            continue;
        }

        draw_texture_ex(
            &image.texture,
            x,
            y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(dest_size),
                ..Default::default()
            },
        );
    }
}

fn terminal_image_dest_size(
    pixel_width: u16,
    pixel_height: u16,
    columns: Option<u32>,
    rows: Option<u32>,
    metrics: CellMetrics,
) -> Vec2 {
    let natural_width = pixel_width.max(1) as f32;
    let natural_height = pixel_height.max(1) as f32;

    match (columns, rows) {
        (Some(columns), Some(rows)) => vec2(
            columns.max(1) as f32 * metrics.cell_width,
            rows.max(1) as f32 * metrics.cell_height,
        ),
        (Some(columns), None) => {
            let width = columns.max(1) as f32 * metrics.cell_width;
            vec2(width, width * natural_height / natural_width)
        }
        (None, Some(rows)) => {
            let height = rows.max(1) as f32 * metrics.cell_height;
            vec2(height * natural_width / natural_height, height)
        }
        (None, None) => vec2(natural_width, natural_height),
    }
}

fn draw_cursor(
    frame: &TerminalFrame,
    viewport: Viewport,
    rect: Rect,
    fonts: &TerminalFonts,
    cursor: AnimatedCursor,
    y_offset: f32,
) {
    let cell_width = viewport.metrics.cell_width;
    let cell_height = viewport.metrics.cell_height;
    let x = rect.x + cursor.target.x as f32 * cell_width;
    let y = rect.y + cursor.target.y as f32 * cell_height + y_offset;
    draw_cursor_trail(frame, viewport, rect, cursor, y_offset);

    match cursor.target.style {
        CursorVisualStyle::Bar => draw_rectangle(x, y, 2.0, cell_height, frame.cursor_color),
        CursorVisualStyle::Underline => {
            draw_rectangle(
                x,
                y + cell_height - 2.0,
                cell_width,
                2.0,
                frame.cursor_color,
            );
        }
        CursorVisualStyle::Block | CursorVisualStyle::BlockHollow => {
            draw_rectangle(
                x,
                y.floor(),
                cell_width.ceil(),
                cell_height.ceil(),
                frame.cursor_color,
            );

            if let Some(cell) = frame
                .rows
                .get(cursor.target.y as usize)
                .and_then(|row| row.get(cursor.target.x as usize))
                .filter(|cell| !cell.text.is_empty())
            {
                draw_text_ex(
                    &cell.text,
                    x,
                    y + viewport.metrics.baseline,
                    TextParams {
                        font: fonts.for_text(&cell.text),
                        font_size: FONT_SIZE,
                        font_scale: 1.0,
                        color: frame.background,
                        ..Default::default()
                    },
                );
            }
        }
        _ => draw_rectangle(
            x,
            y.floor(),
            cell_width.ceil(),
            cell_height.ceil(),
            frame.cursor_color,
        ),
    }
}

fn draw_cursor_trail(
    frame: &TerminalFrame,
    viewport: Viewport,
    rect: Rect,
    cursor: AnimatedCursor,
    y_offset: f32,
) {
    let cell_width = viewport.metrics.cell_width;
    let cell_height = viewport.metrics.cell_height;
    let target_x = rect.x + cursor.target.x as f32 * cell_width;
    let target_y = rect.y + cursor.target.y as f32 * cell_height + y_offset;
    let raw_tail_x = rect.x + cursor.tail_x * cell_width;
    let raw_tail_y = rect.y + cursor.tail_y * cell_height + y_offset;
    let tail_x = target_x + (raw_tail_x - target_x) * CURSOR_TRAIL_SIZE;
    let tail_y = target_y + (raw_tail_y - target_y) * CURSOR_TRAIL_SIZE;
    let distance = ((target_x - tail_x).powi(2) + (target_y - tail_y).powi(2)).sqrt();

    if distance < 0.75 {
        return;
    }

    let mut color = frame.cursor_color;
    color.a = CURSOR_TRAIL_ALPHA as f32 / 255.0;

    if let Some(rect) =
        cursor_trail_rect(tail_x, tail_y, target_x, target_y, cell_width, cell_height)
    {
        draw_rectangle(
            rect.x.floor(),
            rect.y.floor(),
            rect.width.ceil(),
            rect.height.ceil(),
            color,
        );
        return;
    }

    draw_line(
        tail_x + cell_width * 0.5,
        tail_y + cell_height * 0.5,
        target_x + cell_width * 0.5,
        target_y + cell_height * 0.5,
        cell_width.max(cell_height) * 0.72,
        color,
    );
}

fn cursor_trail_rect(
    tail_x: f32,
    tail_y: f32,
    target_x: f32,
    target_y: f32,
    cell_width: f32,
    cell_height: f32,
) -> Option<CursorTrailRect> {
    let dx = (target_x - tail_x).abs();
    let dy = (target_y - tail_y).abs();

    if dx < 0.75 && dy < 0.75 {
        return None;
    }

    if dy <= cell_height * 0.25 {
        return Some(CursorTrailRect {
            x: tail_x.min(target_x),
            y: target_y,
            width: dx + cell_width,
            height: cell_height,
        });
    }

    if dx <= cell_width * 0.25 {
        return Some(CursorTrailRect {
            x: target_x,
            y: tail_y.min(target_y),
            width: cell_width,
            height: dy + cell_height,
        });
    }

    None
}

fn draw_scroll_indicator(frame: &TerminalFrame, rect: Rect) {
    if frame.scrollbar.total == 0 || frame.scrollbar.visible >= frame.scrollbar.total {
        return;
    }

    let track_height = rect.h.max(1.0);
    let thumb_height = (frame.scrollbar.visible as f32 / frame.scrollbar.total as f32
        * track_height)
        .clamp(24.0, track_height);
    let max_top = track_height - thumb_height;
    let top = if frame.scrollbar.total > frame.scrollbar.visible {
        frame.scrollbar.top as f32 / (frame.scrollbar.total - frame.scrollbar.visible) as f32
            * max_top
    } else {
        0.0
    };

    let x = rect.x + rect.w - 3.0;
    draw_rectangle(
        x,
        rect.y + top,
        2.0,
        thumb_height,
        Color::from_rgba(230, 225, 217, 90),
    );
}

struct TerminalFonts {
    latin: Option<Font>,
    cjk: Option<Font>,
}

impl TerminalFonts {
    async fn load(config: &AppConfig) -> Self {
        Self {
            latin: load_latin_font(config).await,
            cjk: load_cjk_font(config).await,
        }
    }

    fn metrics_font(&self) -> Option<&Font> {
        self.latin.as_ref().or(self.cjk.as_ref())
    }

    fn for_text(&self, text: &str) -> Option<&Font> {
        if text.chars().any(is_cjk_candidate) {
            self.cjk.as_ref().or(self.latin.as_ref())
        } else {
            self.latin.as_ref().or(self.cjk.as_ref())
        }
    }
}

async fn load_latin_font(config: &AppConfig) -> Option<Font> {
    if let Some(path) = config.font.latin.as_deref()
        && let Some(font) = load_font_path(path).await
    {
        return Some(font);
    }

    for env_key in ["NVTERM_FONT", "NVTERM_LATIN_FONT"] {
        if let Some(font) = load_env_font(env_key).await {
            return Some(font);
        }
    }

    load_first_font(&[
        "/Users/soyukke/Library/Fonts/CaskaydiaCoveNerdFont-Regular.ttf",
        "/Users/soyukke/Library/Fonts/CascadiaCode-Regular.otf",
        "/System/Library/Fonts/Menlo.ttc",
        "/System/Library/Fonts/Supplemental/Menlo.ttc",
        "/System/Library/Fonts/SFNSMono.ttf",
        "/Library/Fonts/MesloLGS NF Regular.ttf",
        "/Library/Fonts/JetBrainsMono-Regular.ttf",
    ])
    .await
}

async fn load_cjk_font(config: &AppConfig) -> Option<Font> {
    if let Some(path) = config.font.cjk.as_deref()
        && let Some(font) = load_font_path(path).await
    {
        return Some(font);
    }

    load_first_font(&[
        "/Users/soyukke/Library/Fonts/HomeManager/opentype/noto-cjk/NotoSansCJK-VF.otf.ttc",
        "/System/Library/Fonts/Supplemental/AppleGothic.ttf",
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
        "/System/Library/Fonts/AppleSDGothicNeo.ttc",
        "/System/Library/Fonts/CJKSymbolsFallback.ttc",
    ])
    .await
}

async fn load_env_font(env_key: &str) -> Option<Font> {
    let path = env::var(env_key).ok()?;
    load_font_path(&path).await
}

async fn load_first_font(paths: &[&str]) -> Option<Font> {
    for path in paths {
        if let Some(font) = load_font_path(path).await {
            return Some(font);
        }
    }

    None
}

async fn load_font_path(path: &str) -> Option<Font> {
    if Path::new(path).exists() {
        load_ttf_font(path).await.ok()
    } else {
        None
    }
}

fn is_cjk_candidate(ch: char) -> bool {
    matches!(
        ch as u32,
        0x1100..=0x11ff
            | 0x2e80..=0x9fff
            | 0xac00..=0xd7af
            | 0xf900..=0xfaff
            | 0xff00..=0xffef
            | 0x20000..=0x3ffff
    )
}

struct TerminalRenderer {
    state: RenderState<'static>,
    rows: RowIterator<'static>,
    cells: CellIterator<'static>,
}

impl TerminalRenderer {
    fn new() -> Result<Self> {
        Ok(Self {
            state: RenderState::new()?,
            rows: RowIterator::new()?,
            cells: CellIterator::new()?,
        })
    }

    fn collect(&mut self, terminal: &mut Terminal<'static, '_>) -> Result<TerminalFrame> {
        let scrollbar = terminal.scrollbar()?;
        let snapshot = self.state.update(terminal)?;
        let colors = snapshot.colors()?;
        let background = color_from_rgb(colors.background);
        let cursor_color = color_from_rgb(colors.cursor.unwrap_or(colors.foreground));
        let cursor = if snapshot.cursor_visible()? {
            snapshot.cursor_viewport()?.map(|cursor| CursorView {
                x: cursor.x,
                y: cursor.y,
                style: snapshot
                    .cursor_visual_style()
                    .unwrap_or(CursorVisualStyle::Block),
            })
        } else {
            None
        };

        let mut rows = Vec::with_capacity(snapshot.rows()? as usize);
        let mut row_iter = self.rows.update(&snapshot)?;

        while let Some(row) = row_iter.next() {
            let mut cells = Vec::with_capacity(snapshot.cols()? as usize);
            let mut cell_iter = self.cells.update(row)?;

            while let Some(cell) = cell_iter.next() {
                let style = cell.style()?;
                let mut text = String::new();
                cell.graphemes_utf8(&mut text)?;

                if style.invisible {
                    text.clear();
                }

                let mut fg = cell.fg_color()?.unwrap_or(colors.foreground);
                let mut bg = cell.bg_color()?;

                if style.inverse {
                    let inverse_bg = fg;
                    fg = bg.unwrap_or(colors.background);
                    bg = Some(inverse_bg);
                }

                cells.push(CellView {
                    text,
                    fg: color_from_rgb(fg),
                    bg: bg.map(color_from_rgb).filter(|color| *color != background),
                });
            }

            rows.push(cells);
            row.set_dirty(false)?;
        }

        snapshot.set_dirty(libghostty_vt::render::Dirty::Clean)?;

        Ok(TerminalFrame {
            rows,
            images: Vec::new(),
            background,
            cursor_color,
            cursor,
            scrollbar: ScrollbarView {
                top: scrollbar.offset,
                visible: scrollbar.len,
                total: scrollbar.total,
            },
        })
    }
}

struct TerminalFrame {
    rows: Vec<Vec<CellView>>,
    images: Vec<TerminalFrameImage>,
    background: Color,
    cursor_color: Color,
    cursor: Option<CursorView>,
    scrollbar: ScrollbarView,
}

struct TerminalFrameImage {
    key: KittyPlacementKey,
    texture: Texture2D,
    col: u16,
    row: u16,
    columns: Option<u32>,
    rows: Option<u32>,
    pixel_width: u16,
    pixel_height: u16,
    z_index: i32,
}

#[derive(Default)]
struct TerminalImageTextureCache {
    textures: HashMap<u32, CachedTerminalImageTexture>,
}

impl TerminalImageTextureCache {
    fn new() -> Self {
        Self::default()
    }

    fn frame_images(&mut self, graphics: &KittyGraphicsState) -> Vec<TerminalFrameImage> {
        self.textures
            .retain(|image_id, _| graphics.image(*image_id).is_some());

        let mut images = Vec::new();
        for placement in graphics.placements() {
            let Some(resource) = graphics.image(placement.key.image_id) else {
                continue;
            };
            let Some(texture) = self.texture_for(resource) else {
                continue;
            };
            images.push(TerminalFrameImage {
                key: placement.key,
                texture: texture.texture.weak_clone(),
                col: placement.cell.col,
                row: placement.cell.row,
                columns: placement.columns,
                rows: placement.rows,
                pixel_width: texture.pixel_width,
                pixel_height: texture.pixel_height,
                z_index: placement.z_index,
            });
        }

        images.sort_by_key(|image| (image.z_index, image.key.image_id, image.key.placement_id));
        images
    }

    fn texture_for(
        &mut self,
        resource: &KittyImageResource,
    ) -> Option<&CachedTerminalImageTexture> {
        let signature = image_resource_signature(resource);
        if self
            .textures
            .get(&resource.id)
            .is_some_and(|cached| cached.signature == signature)
        {
            return self.textures.get(&resource.id);
        }

        let texture = decode_terminal_image_texture(resource, signature)?;
        self.textures.insert(resource.id, texture);
        self.textures.get(&resource.id)
    }
}

struct CachedTerminalImageTexture {
    signature: u64,
    texture: Texture2D,
    pixel_width: u16,
    pixel_height: u16,
}

fn decode_terminal_image_texture(
    resource: &KittyImageResource,
    signature: u64,
) -> Option<CachedTerminalImageTexture> {
    if !resource.complete || resource.format != Some(KittyImageFormat::Png) {
        return None;
    }
    if !matches!(
        resource.transmission,
        None | Some(KittyTransmission::Direct)
    ) {
        return None;
    }

    let rgba = image::load_from_memory(&resource.bytes).ok()?.to_rgba8();
    let pixel_width = u16::try_from(rgba.width()).ok()?;
    let pixel_height = u16::try_from(rgba.height()).ok()?;
    if pixel_width == 0 || pixel_height == 0 {
        return None;
    }

    let texture = Texture2D::from_rgba8(pixel_width, pixel_height, rgba.as_raw());
    texture.set_filter(FilterMode::Linear);
    Some(CachedTerminalImageTexture {
        signature,
        texture,
        pixel_width,
        pixel_height,
    })
}

fn image_resource_signature(resource: &KittyImageResource) -> u64 {
    let mut hasher = DefaultHasher::new();
    resource.format.hash(&mut hasher);
    resource.transmission.hash(&mut hasher);
    resource.complete.hash(&mut hasher);
    resource.bytes.hash(&mut hasher);
    hasher.finish()
}

#[derive(Clone, Debug, PartialEq)]
struct CellView {
    text: String,
    fg: Color,
    bg: Option<Color>,
}

#[derive(Clone, Copy, Debug)]
struct CursorView {
    x: u16,
    y: u16,
    style: CursorVisualStyle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AgentKind {
    Claude,
    Codex,
    Agent,
}

impl AgentKind {
    fn label(self) -> &'static str {
        match self {
            Self::Claude => "Claude Code",
            Self::Codex => "Codex",
            Self::Agent => "Agent",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AgentScreenState {
    Unknown,
    Idle,
    Busy,
    NeedsInput,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AgentScreenStatus {
    kind: Option<AgentKind>,
    state: AgentScreenState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AgentNotificationEvent {
    Finished,
    NeedsInput,
}

#[derive(Clone, Debug)]
struct AgentMonitor {
    state: AgentScreenState,
    kind: Option<AgentKind>,
    busy_since: Option<Instant>,
    notified_for_run: bool,
}

impl AgentMonitor {
    fn new() -> Self {
        Self {
            state: AgentScreenState::Unknown,
            kind: None,
            busy_since: None,
            notified_for_run: false,
        }
    }

    fn update(
        &mut self,
        frame: &TerminalFrame,
        now: Instant,
        min_busy: Duration,
        tab_title: &str,
        pane_id: PaneId,
    ) -> Option<DesktopNotification> {
        let text = terminal_frame_text(frame);
        let status = classify_agent_screen(&text);
        if let Some(kind) = status.kind {
            self.kind = Some(kind);
        }

        let state = if status.state == AgentScreenState::Unknown
            && self.state == AgentScreenState::Busy
            && self.kind.is_some()
        {
            AgentScreenState::Idle
        } else {
            status.state
        };

        let was_busy = self.state == AgentScreenState::Busy;
        let busy_enough = self
            .busy_since
            .is_some_and(|since| now.duration_since(since) >= min_busy);

        match state {
            AgentScreenState::Busy => {
                if self.state != AgentScreenState::Busy {
                    self.busy_since = Some(now);
                    self.notified_for_run = false;
                }
                self.state = AgentScreenState::Busy;
                None
            }
            AgentScreenState::NeedsInput => {
                self.state = AgentScreenState::NeedsInput;
                self.busy_since = None;
                if was_busy && busy_enough && !self.notified_for_run {
                    self.notified_for_run = true;
                    return self.notification(
                        AgentNotificationEvent::NeedsInput,
                        tab_title,
                        pane_id,
                    );
                }
                None
            }
            AgentScreenState::Idle => {
                self.state = AgentScreenState::Idle;
                self.busy_since = None;
                if was_busy && busy_enough && !self.notified_for_run {
                    self.notified_for_run = true;
                    return self.notification(AgentNotificationEvent::Finished, tab_title, pane_id);
                }
                None
            }
            AgentScreenState::Unknown => {
                self.state = AgentScreenState::Unknown;
                self.busy_since = None;
                self.notified_for_run = false;
                None
            }
        }
    }

    fn notification(
        &self,
        event: AgentNotificationEvent,
        tab_title: &str,
        pane_id: PaneId,
    ) -> Option<DesktopNotification> {
        let kind = self.kind?;
        let action = match event {
            AgentNotificationEvent::Finished => "finished",
            AgentNotificationEvent::NeedsInput => "needs input",
        };
        Some(DesktopNotification {
            title: "neovide-tabs".to_owned(),
            subtitle: Some(format!("{} · pane {}", tab_title, pane_id.0)),
            body: format!("{} {action}", kind.label()),
        })
    }

    fn is_busy(&self) -> bool {
        self.state == AgentScreenState::Busy
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
struct AgentStatusFile {
    state: String,
    summary: Option<String>,
    updated_token: Option<String>,
    updated_unix: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AgentStatusKey {
    state: String,
    summary: String,
    revision: String,
}

#[derive(Clone, Debug)]
struct AgentStatusFileMonitor {
    path: Option<PathBuf>,
    last_key: Option<AgentStatusKey>,
}

impl AgentStatusFileMonitor {
    fn new(path: Option<PathBuf>) -> Self {
        let last_key = path.as_deref().and_then(read_agent_status);
        Self { path, last_key }
    }

    fn update(&mut self, tab_title: &str, pane_id: PaneId) -> Option<DesktopNotification> {
        let path = self.path.as_deref()?;
        let Some(key) = read_agent_status(path) else {
            self.last_key = None;
            return None;
        };

        if self.last_key.as_ref() == Some(&key) {
            return None;
        }

        let should_notify = is_terminal_agent_status(&key.state);
        self.last_key = Some(key.clone());

        should_notify.then(|| agent_status_notification(&key, tab_title, pane_id))
    }

    fn is_running(&self) -> bool {
        self.last_key
            .as_ref()
            .is_some_and(|key| is_terminal_agent_running_status(&key.state))
    }

    fn has_status(&self) -> bool {
        self.last_key.is_some()
    }
}

fn read_agent_status(path: &Path) -> Option<AgentStatusKey> {
    let contents = fs::read_to_string(path).ok()?;
    let status = toml::from_str::<AgentStatusFile>(&contents).ok()?;
    let state = normalize_agent_status(&status.state)?;
    let summary = status.summary.unwrap_or_default().trim().to_owned();
    let revision = status
        .updated_token
        .filter(|value| !value.trim().is_empty())
        .or_else(|| status.updated_unix.map(|value| value.to_string()))
        .unwrap_or_default();
    Some(AgentStatusKey {
        state,
        summary,
        revision,
    })
}

fn normalize_agent_status(state: &str) -> Option<String> {
    let state = state.trim().to_ascii_lowercase().replace('-', "_");
    (!state.is_empty()).then_some(state)
}

fn is_terminal_agent_status(state: &str) -> bool {
    matches!(
        state,
        "done" | "complete" | "completed" | "success" | "blocked" | "failed" | "needs_input"
    )
}

fn is_terminal_agent_running_status(state: &str) -> bool {
    matches!(state, "running" | "busy" | "working")
}

fn should_show_agent_spinner(status_seen: bool, status_running: bool, screen_busy: bool) -> bool {
    if status_seen {
        status_running
    } else {
        screen_busy
    }
}

fn agent_status_notification(
    status: &AgentStatusKey,
    tab_title: &str,
    pane_id: PaneId,
) -> DesktopNotification {
    let action = match status.state.as_str() {
        "done" | "complete" | "completed" | "success" => "Agent done",
        "blocked" => "Agent blocked",
        "failed" => "Agent failed",
        "needs_input" => "Agent needs input",
        _ => "Agent updated",
    };
    let body = if status.summary.is_empty() {
        action.to_owned()
    } else {
        format!("{action}: {}", status.summary)
    };

    DesktopNotification {
        title: "neovide-tabs".to_owned(),
        subtitle: Some(format!("{} · pane {}", tab_title, pane_id.0)),
        body,
    }
}

fn terminal_frame_text(frame: &TerminalFrame) -> String {
    let mut text = String::new();
    for row in &frame.rows {
        for cell in row {
            text.push_str(&cell.text);
        }
        text.push('\n');
    }
    text
}

fn classify_agent_screen(text: &str) -> AgentScreenStatus {
    let lower = text.to_lowercase();
    let has_claude = lower.contains("claude");
    let has_codex = lower.contains("codex")
        || lower.contains("gpt-")
        || lower.contains("tokens")
        || lower.contains("xhigh effort")
        || lower.contains("thinking with");
    let needs_input = contains_any(
        &lower,
        &[
            "needs input",
            "needs your input",
            "waiting for your input",
            "do you want to proceed",
            "requires confirmation",
            "permission",
            "approve",
            "approval",
            "press enter to continue",
            "accept edits",
            "proceed?",
        ],
    );
    let busy = contains_any(
        &lower,
        &[
            "esc to interrupt",
            "ctrl+c to cancel",
            "thinking with",
            "thinking",
            "working",
            "running",
            "executing",
            "processing",
            "tool use",
        ],
    );
    let kind = if has_claude {
        Some(AgentKind::Claude)
    } else if has_codex {
        Some(AgentKind::Codex)
    } else if needs_input || busy {
        Some(AgentKind::Agent)
    } else {
        None
    };
    let state = if needs_input {
        AgentScreenState::NeedsInput
    } else if busy {
        AgentScreenState::Busy
    } else if kind.is_some() {
        AgentScreenState::Idle
    } else {
        AgentScreenState::Unknown
    };

    AgentScreenStatus { kind, state }
}

fn contains_any(text: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|pattern| text.contains(pattern))
}

#[derive(Clone, Copy, Debug)]
struct AnimatedCursor {
    target: CursorView,
    tail_x: f32,
    tail_y: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct CursorTrailRect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

#[derive(Clone, Copy, Debug, Default)]
struct CursorMotion {
    tail_x: f32,
    tail_y: f32,
    target_x: f32,
    target_y: f32,
    initialized: bool,
}

impl CursorMotion {
    fn new() -> Self {
        Self::default()
    }

    fn reset(&mut self) {
        self.initialized = false;
    }

    fn update(&mut self, cursor: Option<CursorView>, dt_seconds: f32) -> Option<AnimatedCursor> {
        let cursor = cursor?;
        let target_x = cursor.x as f32;
        let target_y = cursor.y as f32;

        if !self.initialized {
            self.tail_x = target_x;
            self.tail_y = target_y;
            self.target_x = target_x;
            self.target_y = target_y;
            self.initialized = true;
            return Some(AnimatedCursor {
                target: cursor,
                tail_x: self.tail_x,
                tail_y: self.tail_y,
            });
        }

        self.target_x = target_x;
        self.target_y = target_y;

        let visible_tail_x = self.tail_x;
        let visible_tail_y = self.tail_y;
        let length = cursor_animation_length(self.tail_x, self.tail_y, target_x, target_y);
        let alpha = animation_alpha(dt_seconds, length);
        self.tail_x += (target_x - self.tail_x) * alpha;
        self.tail_y += (target_y - self.tail_y) * alpha;

        if (target_x - self.tail_x).abs() < CURSOR_SNAP_EPSILON
            && (target_y - self.tail_y).abs() < CURSOR_SNAP_EPSILON
        {
            self.tail_x = target_x;
            self.tail_y = target_y;
        }

        Some(AnimatedCursor {
            target: cursor,
            tail_x: visible_tail_x,
            tail_y: visible_tail_y,
        })
    }
}

fn cursor_animation_length(tail_x: f32, tail_y: f32, target_x: f32, target_y: f32) -> f32 {
    let dx = (target_x - tail_x).abs();
    let dy = (target_y - tail_y).abs();

    if dy < CURSOR_SNAP_EPSILON && dx <= 2.0 {
        CURSOR_SHORT_ANIMATION_LENGTH
    } else {
        CURSOR_ANIMATION_LENGTH
    }
}

fn animation_alpha(dt_seconds: f32, animation_length: f32) -> f32 {
    if dt_seconds <= 0.0 || animation_length <= 0.0 {
        return 1.0;
    }

    1.0 - 0.05_f32.powf(dt_seconds / animation_length)
}

#[derive(Clone, Copy, Debug)]
struct ScrollbarView {
    top: u64,
    visible: u64,
    total: u64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct CellMetrics {
    cell_width: f32,
    cell_height: f32,
    baseline: f32,
}

impl CellMetrics {
    fn from_font(font: Option<&Font>) -> Self {
        let measured = measure_text("M", font, FONT_SIZE, 1.0);
        let cell_width = measured.width.ceil().max(8.0);
        let cell_height = (FONT_SIZE as f32 * 1.35).ceil();
        let baseline = (cell_height * 0.78).round();

        Self {
            cell_width,
            cell_height,
            baseline,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TerminalSize {
    cols: u16,
    rows: u16,
    pixel_width: u16,
    pixel_height: u16,
}

impl TerminalSize {
    fn pty_size(self) -> PtySize {
        PtySize {
            rows: self.rows,
            cols: self.cols,
            pixel_width: self.pixel_width,
            pixel_height: self.pixel_height,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Viewport {
    size: TerminalSize,
    metrics: CellMetrics,
}

impl Viewport {
    fn from_rect(metrics: CellMetrics, rect: Rect) -> Self {
        let width = rect.w.max(metrics.cell_width);
        let height = rect.h.max(metrics.cell_height);
        let cols = (width / metrics.cell_width).floor().max(MIN_PANE_COLS) as u16;
        let rows = (height / metrics.cell_height).floor().max(MIN_PANE_ROWS) as u16;

        Self {
            size: TerminalSize {
                cols,
                rows,
                pixel_width: width.round().clamp(1.0, u16::MAX as f32) as u16,
                pixel_height: height.round().clamp(1.0, u16::MAX as f32) as u16,
            },
            metrics,
        }
    }
}

fn install_agent_command_shims(dir: &Path) -> Result<()> {
    fs::create_dir_all(dir)
        .with_context(|| format!("failed to create agent shim dir {}", dir.display()))?;
    write_agent_command_shim(dir, "claude", "nvterm-claude")?;
    write_agent_command_shim(dir, "codex", "nvterm-codex")?;
    Ok(())
}

fn write_agent_command_shim(dir: &Path, command_name: &str, wrapper_name: &str) -> Result<()> {
    let wrapper = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("scripts")
        .join(wrapper_name);
    let target = dir.join(command_name);
    let contents = format!(
        "#!/usr/bin/env bash\nexec {} \"$@\"\n",
        shell_single_quote(&wrapper.to_string_lossy())
    );
    fs::write(&target, contents)
        .with_context(|| format!("failed to write agent shim {}", target.display()))?;
    set_executable(&target)?;
    Ok(())
}

fn shell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn set_executable(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        let mut permissions = fs::metadata(path)?.permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(path, permissions)?;
    }
    Ok(())
}

fn find_executable_in_path(name: &str) -> Option<PathBuf> {
    let path = env::var_os("PATH")?;
    let current_shim_dir = env::var_os("NVTERM_AGENT_SHIM_DIR").map(PathBuf::from);
    let shim_root = env::var_os("NVTERM_AGENT_SHIM_ROOT")
        .map(PathBuf::from)
        .or_else(|| app_state_dir().map(|path| path.join("shims")));

    env::split_paths(&path)
        .filter(|dir| {
            current_shim_dir.as_deref() != Some(dir.as_path())
                && !shim_root
                    .as_deref()
                    .is_some_and(|root| dir.starts_with(root))
        })
        .map(|dir| dir.join(name))
        .find(|candidate| is_executable_file(candidate))
}

fn is_executable_file(path: &Path) -> bool {
    let Ok(metadata) = fs::metadata(path) else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        metadata.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        true
    }
}

struct PtySession {
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    rx: Receiver<Vec<u8>>,
    child: Box<dyn Child + Send + Sync>,
}

impl PtySession {
    fn spawn(
        size: TerminalSize,
        cwd: Option<&Path>,
        pane_id: PaneId,
        agent_status_path: Option<&Path>,
    ) -> Result<Self> {
        let pty_system = native_pty_system();
        let pair = pty_system.openpty(size.pty_size())?;
        let mut cmd = CommandBuilder::new(default_shell());
        cmd.arg("-l");
        cmd.arg("-i");
        if let Some(cwd) = cwd {
            cmd.cwd(cwd);
            cmd.env("PWD", cwd);
        }
        let locale = terminal_locale();
        cmd.env_remove("LC_ALL");
        cmd.env("LANG", &locale);
        cmd.env("LC_CTYPE", &locale);
        cmd.env("TERM", "xterm-256color");
        cmd.env("COLORTERM", "truecolor");
        cmd.env("NVTERM_PROTO", "libghostty-vt");
        cmd.env("NVTERM_PANE_ID", pane_id.0.to_string());
        let real_claude = find_executable_in_path("claude");
        let real_codex = find_executable_in_path("codex");
        if let Some(shim_dir) = agent_shim_dir(pane_id)
            && install_agent_command_shims(&shim_dir).is_ok()
        {
            cmd.env("NVTERM_AGENT_SHIM_DIR", &shim_dir);
            if let Some(path) = real_claude {
                cmd.env("NVTERM_REAL_CLAUDE", path);
            }
            if let Some(path) = real_codex {
                cmd.env("NVTERM_REAL_CODEX", path);
            }
            if let Some(path) = env::var_os("PATH") {
                let mut paths = vec![shim_dir];
                paths.extend(env::split_paths(&path));
                if let Ok(joined) = env::join_paths(paths) {
                    cmd.env("PATH", joined);
                }
            }
        }
        if let Some(agent_status_path) = agent_status_path {
            if let Some(parent) = agent_status_path.parent() {
                let _ = fs::create_dir_all(parent);
                cmd.env("NVTERM_AGENT_STATUS_DIR", parent);
            }
            cmd.env("NVTERM_AGENT_STATUS_FILE", agent_status_path);
        }

        let child = pair.slave.spawn_command(cmd)?;
        let mut reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let mut buf = [0u8; 16 * 1024];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        if tx.send(buf[..n].to_vec()).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(Self {
            master: pair.master,
            writer,
            rx,
            child,
        })
    }

    fn has_exited(&mut self) -> Result<bool> {
        Ok(self.child.try_wait()?.is_some())
    }

    fn resize(&self, size: TerminalSize) -> Result<()> {
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

fn spawn_cwd(cwd: Option<PathBuf>) -> Option<PathBuf> {
    cwd.filter(|path| path.is_dir())
        .or_else(|| env::current_dir().ok())
}

fn default_shell() -> String {
    if let Some(shell) = env_shell("NVTERM_SHELL") {
        return shell;
    }

    if let Some(shell) = login_shell() {
        return shell;
    }

    if let Some(shell) = env_shell("SHELL") {
        return shell;
    }

    "/bin/zsh".to_owned()
}

fn env_shell(key: &str) -> Option<String> {
    env::var(key).ok().filter(|shell| !shell.is_empty())
}

fn login_shell() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        let user = env::var("USER").ok()?;
        let output = Command::new("/usr/bin/dscl")
            .args([".", "-read", &format!("/Users/{user}"), "UserShell"])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        String::from_utf8_lossy(&output.stdout)
            .lines()
            .find_map(|line| line.strip_prefix("UserShell: "))
            .map(str::trim)
            .filter(|shell| !shell.is_empty())
            .map(str::to_owned)
    }

    #[cfg(not(target_os = "macos"))]
    {
        None
    }
}

fn terminal_locale() -> String {
    env::var("LANG")
        .ok()
        .filter(|locale| locale.to_ascii_uppercase().contains("UTF-8"))
        .unwrap_or_else(|| "ja_JP.UTF-8".to_owned())
}

fn color_from_rgb(rgb: RgbColor) -> Color {
    Color::from_rgba(rgb.r, rgb.g, rgb.b, 255)
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use macroquad::{
        miniquad::{EventHandler, KeyMods},
        prelude::{KeyCode, Rect, WHITE, vec2},
    };

    use super::{
        AgentKind, AgentScreenState, AgentStatusFileMonitor, AppCommand, AppConfig, CellMetrics,
        CellView, CursorMotion, CursorTrailRect, CursorView, CursorVisualStyle, InputContext,
        KeyChord, OUTPUT_SCROLL_ANIMATION_FAR_LINES, PaneId, PaneLayout, ScrollbarView,
        SessionPaneState, SessionState, SessionTabState, SplitAxis, StoredPaneLayout,
        StoredSplitAxis, THEMES, TabMenuAction, TerminalInput, TextEdit, TextInputEvent,
        animation_alpha, app_command_for_key, applescript_string, binding_for_command,
        bounded_scroll_rows, classify_agent_screen, configured_keybindings,
        cursor_animation_length, cursor_trail_rect, default_keybindings, detect_output_scroll_rows,
        detect_upward_row_shift, format_binding, keybinding, parse_file_uri_path, parse_key_chord,
        percent_decode_utf8, read_agent_status, resolve_keybinding, session_title_number,
        should_show_agent_spinner, tab_label, tab_menu_action_at, tab_menu_item_rect,
        terminal_image_dest_size,
    };

    fn scrollbar(top: u64, visible: u64, total: u64) -> ScrollbarView {
        ScrollbarView {
            top,
            visible,
            total,
        }
    }

    fn row(text: &str) -> Vec<CellView> {
        vec![CellView {
            text: text.to_owned(),
            fg: WHITE,
            bg: None,
        }]
    }

    fn temp_agent_status_path(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "neovide-tabs-agent-status-{name}-{}.toml",
            std::process::id()
        ));
        let _ = fs::remove_file(&path);
        path
    }

    #[test]
    fn bounded_scroll_rows_ignores_missing_scrollback() {
        assert_eq!(bounded_scroll_rows(3, Some(scrollbar(0, 20, 20))), 0);
        assert_eq!(bounded_scroll_rows(-3, Some(scrollbar(0, 20, 20))), 0);
    }

    #[test]
    fn bounded_scroll_rows_stops_at_edges() {
        assert_eq!(bounded_scroll_rows(-3, Some(scrollbar(0, 10, 100))), 0);
        assert_eq!(bounded_scroll_rows(3, Some(scrollbar(90, 10, 100))), 0);
    }

    #[test]
    fn tab_label_omits_pane_count() {
        assert_eq!(tab_label("session 1"), "session 1");
    }

    #[test]
    fn tab_context_menu_hit_tests_actions() {
        let menu = Rect::new(20.0, 40.0, 176.0, 192.0);
        let rename = tab_menu_item_rect(menu, 0);
        let first_theme = tab_menu_item_rect(menu, 1);
        let last_theme = tab_menu_item_rect(menu, THEMES.len());

        assert_eq!(
            tab_menu_action_at(menu, vec2(rename.x + 4.0, rename.y + 4.0)),
            Some(TabMenuAction::Rename)
        );
        assert_eq!(
            tab_menu_action_at(menu, vec2(first_theme.x + 4.0, first_theme.y + 4.0)),
            Some(TabMenuAction::Theme(0))
        );
        assert_eq!(
            tab_menu_action_at(menu, vec2(last_theme.x + 4.0, last_theme.y + 4.0)),
            Some(TabMenuAction::Theme(THEMES.len() - 1))
        );
        assert_eq!(tab_menu_action_at(menu, vec2(0.0, 0.0)), None);
    }

    #[test]
    fn terminal_image_dest_size_uses_cell_dimensions_when_provided() {
        let metrics = CellMetrics {
            cell_width: 9.0,
            cell_height: 18.0,
            baseline: 14.0,
        };

        assert_eq!(
            terminal_image_dest_size(20, 10, Some(4), Some(2), metrics),
            vec2(36.0, 36.0)
        );
        assert_eq!(
            terminal_image_dest_size(20, 10, Some(4), None, metrics),
            vec2(36.0, 18.0)
        );
        assert_eq!(
            terminal_image_dest_size(20, 10, None, Some(2), metrics),
            vec2(72.0, 36.0)
        );
        assert_eq!(
            terminal_image_dest_size(20, 10, None, None, metrics),
            vec2(20.0, 10.0)
        );
    }

    #[test]
    fn bounded_scroll_rows_caps_partial_edge_movement() {
        assert_eq!(bounded_scroll_rows(-5, Some(scrollbar(2, 10, 100))), -2);
        assert_eq!(bounded_scroll_rows(5, Some(scrollbar(88, 10, 100))), 2);
    }

    #[test]
    fn bounded_scroll_rows_keeps_request_when_scrollbar_is_unavailable() {
        assert_eq!(bounded_scroll_rows(3, None), 3);
        assert_eq!(bounded_scroll_rows(-3, None), -3);
    }

    #[test]
    fn detects_strict_upward_row_shift() {
        let previous = vec![row("a"), row("b"), row("c"), row("d")];
        let current = vec![row("b"), row("c"), row("d"), row("e")];

        assert_eq!(detect_upward_row_shift(&previous, &current), 1);
    }

    #[test]
    fn detects_batched_upward_row_shift() {
        let previous = (0..16)
            .map(|idx| row(&format!("old-{idx}")))
            .collect::<Vec<_>>();
        let current = (5..21)
            .map(|idx| row(&format!("old-{idx}")))
            .collect::<Vec<_>>();

        assert_eq!(detect_upward_row_shift(&previous, &current), 5);
    }

    #[test]
    fn ignores_unrelated_row_changes() {
        let previous = vec![row("a"), row("b"), row("c"), row("d")];
        let current = vec![row("a"), row("x"), row("c"), row("d")];

        assert_eq!(detect_upward_row_shift(&previous, &current), 0);
    }

    #[test]
    fn detects_output_scroll_from_scrollbar_growth_when_rows_do_not_overlap() {
        let previous = vec![row("a"), row("b"), row("c"), row("d")];
        let current = vec![row("w"), row("x"), row("y"), row("z")];

        assert_eq!(
            detect_output_scroll_rows(
                &previous,
                scrollbar(10, 10, 20),
                &current,
                scrollbar(16, 10, 26),
            ),
            6
        );
    }

    #[test]
    fn caps_large_output_scroll_animation() {
        let previous = vec![row("a"), row("b"), row("c"), row("d")];
        let current = vec![row("w"), row("x"), row("y"), row("z")];

        assert_eq!(
            detect_output_scroll_rows(
                &previous,
                scrollbar(10, 4, 14),
                &current,
                scrollbar(100, 4, 104),
            ),
            OUTPUT_SCROLL_ANIMATION_FAR_LINES as isize
        );
    }

    #[test]
    fn ignores_output_scroll_detection_away_from_bottom() {
        let previous = vec![row("a"), row("b"), row("c"), row("d")];
        let current = vec![row("b"), row("c"), row("d"), row("e")];

        assert_eq!(
            detect_output_scroll_rows(
                &previous,
                scrollbar(10, 4, 20),
                &current,
                scrollbar(11, 4, 30),
            ),
            0
        );
    }

    #[test]
    fn terminal_input_sends_tab_from_key_event_once() {
        let mut input = TerminalInput::default();

        input.key_down_event(KeyCode::Tab, KeyMods::default(), false);
        input.char_event('\t', KeyMods::default(), false);

        assert_eq!(input.bytes, b"\t");
    }

    #[test]
    fn terminal_input_sends_tab_from_char_event() {
        let mut input = TerminalInput::default();

        input.char_event('\t', KeyMods::default(), false);

        assert_eq!(input.bytes, b"\t");
    }

    #[test]
    fn terminal_input_sends_ctrl_i_as_tab_from_char_event() {
        let mut input = TerminalInput::default();
        let keymods = KeyMods {
            ctrl: true,
            ..Default::default()
        };

        input.key_down_event(KeyCode::I, keymods, false);
        input.char_event('\t', keymods, false);

        assert_eq!(input.bytes, b"\t");
    }

    #[test]
    fn terminal_input_sends_ctrl_u_once_from_char_event() {
        let mut input = TerminalInput::default();
        let keymods = KeyMods {
            ctrl: true,
            ..Default::default()
        };

        input.key_down_event(KeyCode::U, keymods, false);
        input.char_event('\u{15}', keymods, false);

        assert_eq!(input.bytes, b"\x15");
    }

    #[test]
    fn terminal_input_does_not_synthesize_ctrl_letters_from_key_event() {
        let mut input = TerminalInput::default();
        let keymods = KeyMods {
            ctrl: true,
            ..Default::default()
        };

        input.key_down_event(KeyCode::U, keymods, false);

        assert!(input.bytes.is_empty());
    }

    #[test]
    fn terminal_input_drops_appkit_arrow_function_char_after_arrow_key() {
        let mut input = TerminalInput::default();

        input.key_down_event(KeyCode::Up, KeyMods::default(), false);
        input.char_event(char::from_u32(0xf700).unwrap(), KeyMods::default(), false);

        assert_eq!(input.bytes, b"\x1b[A");
    }

    #[test]
    fn terminal_input_drops_appkit_function_char_without_key_event() {
        let mut input = TerminalInput::default();

        input.char_event(char::from_u32(0xf701).unwrap(), KeyMods::default(), false);

        assert!(input.bytes.is_empty());
    }

    #[test]
    fn rename_input_drops_appkit_function_key_chars() {
        let mut input = TerminalInput::new(false, InputContext::Rename, &default_keybindings());

        input.char_event('a', KeyMods::default(), false);
        input.char_event(char::from_u32(0xf702).unwrap(), KeyMods::default(), false);

        assert_eq!(input.text_chars, vec!['a']);
    }

    #[test]
    fn terminal_input_keeps_non_appkit_private_use_chars() {
        let mut input = TerminalInput::default();
        let ch = char::from_u32(0xe0b0).unwrap();
        let mut expected = [0; 4];

        input.char_event(ch, KeyMods::default(), false);

        assert_eq!(input.bytes, ch.encode_utf8(&mut expected).as_bytes());
    }

    #[test]
    fn app_commands_require_logo_modifier() {
        assert_eq!(
            app_command_for_key(
                KeyCode::T,
                KeyMods {
                    logo: true,
                    ..Default::default()
                },
            ),
            Some(AppCommand::NewTab)
        );
        assert_eq!(app_command_for_key(KeyCode::T, KeyMods::default()), None);
    }

    #[test]
    fn app_commands_split_and_move_tabs() {
        assert_eq!(
            app_command_for_key(
                KeyCode::D,
                KeyMods {
                    logo: true,
                    ..Default::default()
                },
            ),
            Some(AppCommand::SplitVertical)
        );
        assert_eq!(
            app_command_for_key(
                KeyCode::D,
                KeyMods {
                    logo: true,
                    shift: true,
                    ..Default::default()
                },
            ),
            Some(AppCommand::SplitHorizontal)
        );
        assert_eq!(
            app_command_for_key(
                KeyCode::RightBracket,
                KeyMods {
                    logo: true,
                    shift: true,
                    ..Default::default()
                },
            ),
            Some(AppCommand::NextTab)
        );
    }

    #[test]
    fn terminal_input_captures_command_without_pty_bytes() {
        let mut input = TerminalInput::default();

        input.key_down_event(
            KeyCode::T,
            KeyMods {
                logo: true,
                ..Default::default()
            },
            false,
        );
        input.char_event(
            't',
            KeyMods {
                logo: true,
                ..Default::default()
            },
            false,
        );

        assert_eq!(input.commands, vec![AppCommand::NewTab]);
        assert!(input.bytes.is_empty());
    }

    #[test]
    fn terminal_input_captures_rename_text() {
        let mut input = TerminalInput::new(false, InputContext::Rename, &default_keybindings());

        input.char_event('a', KeyMods::default(), false);
        input.key_down_event(KeyCode::Backspace, KeyMods::default(), false);
        input.key_down_event(KeyCode::Enter, KeyMods::default(), false);

        assert_eq!(input.text_chars, vec!['a']);
        assert_eq!(
            input.text_edits,
            vec![TextEdit::Backspace, TextEdit::Commit]
        );
        assert_eq!(
            input.text_events,
            vec![
                TextInputEvent::Char('a'),
                TextInputEvent::Edit(TextEdit::Backspace),
                TextInputEvent::Edit(TextEdit::Commit),
            ]
        );
        assert!(input.bytes.is_empty());
    }

    #[test]
    fn rename_command_captures_following_text_in_same_batch() {
        let mut input = TerminalInput::default();
        let command_mods = KeyMods {
            logo: true,
            ..Default::default()
        };

        input.key_down_event(KeyCode::R, command_mods, false);
        input.char_event('b', KeyMods::default(), false);
        input.char_event('u', KeyMods::default(), false);
        input.key_down_event(KeyCode::Enter, KeyMods::default(), false);

        assert_eq!(input.commands, vec![AppCommand::RenameSession]);
        assert_eq!(
            input.text_events,
            vec![
                TextInputEvent::Char('b'),
                TextInputEvent::Char('u'),
                TextInputEvent::Edit(TextEdit::Commit),
            ]
        );
        assert!(input.bytes.is_empty());
    }

    #[test]
    fn keybindings_overlay_consumes_text_input() {
        let mut input =
            TerminalInput::new(false, InputContext::Keybindings, &default_keybindings());

        input.char_event('x', KeyMods::default(), false);
        input.key_down_event(KeyCode::Escape, KeyMods::default(), false);

        assert!(input.bytes.is_empty());
        assert_eq!(input.commands, vec![AppCommand::DismissOverlay]);
    }

    #[test]
    fn keybinding_capture_records_next_chord() {
        let mut input = TerminalInput::new(
            false,
            InputContext::KeybindingCapture,
            &default_keybindings(),
        );
        let mods = KeyMods {
            logo: true,
            shift: true,
            ..Default::default()
        };

        input.key_down_event(KeyCode::P, mods, false);
        input.char_event('P', mods, false);

        assert_eq!(input.captured_chord, Some(KeyChord::new(KeyCode::P, mods)));
        assert!(input.bytes.is_empty());
    }

    #[test]
    fn dynamic_keybindings_resolve_new_binding() {
        let bindings = vec![keybinding(
            AppCommand::NewTab,
            KeyCode::P,
            true,
            true,
            false,
            false,
        )];
        let mods = KeyMods {
            logo: true,
            shift: true,
            ..Default::default()
        };

        assert_eq!(
            resolve_keybinding(&bindings, KeyCode::P, mods),
            Some(AppCommand::NewTab)
        );
        assert_eq!(
            format_binding(binding_for_command(&bindings, AppCommand::NewTab)),
            "Cmd+Shift+P"
        );
    }

    #[test]
    fn parses_config_key_chord() {
        let chord = parse_key_chord("cmd+shift+]").unwrap();

        assert_eq!(chord.keycode, KeyCode::RightBracket);
        assert!(chord.mods.logo);
        assert!(chord.mods.shift);
        assert!(!chord.mods.ctrl);
        assert!(!chord.mods.alt);
    }

    #[test]
    fn config_overrides_keybinding_and_removes_conflict() {
        let config = toml::from_str::<AppConfig>(
            r#"
            [keybindings]
            new_tab = "cmd+shift+n"
            close_active = "cmd+shift+n"
            "#,
        )
        .unwrap();

        let bindings = configured_keybindings(&config).unwrap();
        let mods = KeyMods {
            logo: true,
            shift: true,
            ..Default::default()
        };

        assert_eq!(
            resolve_keybinding(&bindings, KeyCode::N, mods),
            Some(AppCommand::CloseActive)
        );
        assert_eq!(binding_for_command(&bindings, AppCommand::NewTab), None);
    }

    #[test]
    fn parses_file_uri_paths_from_osc7_payloads() {
        assert_eq!(
            parse_file_uri_path("file://host/Users/soyukke/dev/app%20space"),
            Some(PathBuf::from("/Users/soyukke/dev/app space"))
        );
        assert_eq!(
            parse_file_uri_path("file:///tmp/%E3%83%86%E3%82%B9%E3%83%88"),
            Some(PathBuf::from("/tmp/テスト"))
        );
        assert_eq!(parse_file_uri_path("https://example.com/tmp"), None);
    }

    #[test]
    fn osc7_tracker_handles_split_bel_and_st_terminated_sequences() {
        let mut tracker = super::OscTracker::new();

        let events = tracker.push(b"\x1b]7;file://host/Users/soy");
        assert_eq!(events.cwd, None);
        assert!(events.notifications.is_empty());

        let events = tracker.push(b"ukke/dev\x07");
        assert_eq!(events.cwd, Some(PathBuf::from("/Users/soyukke/dev")));
        assert!(events.notifications.is_empty());

        let events = tracker.push(b"prompt\x1b]7;file://host/tmp/project\x1b\\tail");
        assert_eq!(events.cwd, Some(PathBuf::from("/tmp/project")));
        assert!(events.notifications.is_empty());
    }

    #[test]
    fn osc_tracker_parses_notification_sequences() {
        let mut tracker = super::OscTracker::new();

        let events = tracker.push(b"\x1b]9;Build finished\x07");
        assert_eq!(events.notifications.len(), 1);
        assert_eq!(events.notifications[0].body, "Build finished");

        let events = tracker.push(b"\x1b]777;notify;Codex;Needs input\x1b\\");
        assert_eq!(events.notifications.len(), 1);
        assert_eq!(events.notifications[0].title, "Codex");
        assert_eq!(events.notifications[0].body, "Needs input");
    }

    #[test]
    fn reads_agent_status_files() {
        let path = temp_agent_status_path("read");
        fs::write(
            &path,
            r#"
state = "done"
summary = "all good"
updated_token = "abc"
updated_unix = 123
"#,
        )
        .unwrap();

        let status = read_agent_status(&path).unwrap();

        assert_eq!(status.state, "done");
        assert_eq!(status.summary, "all good");
        assert_eq!(status.revision, "abc");
        let _ = fs::remove_file(path);
    }

    #[test]
    fn agent_status_monitor_notifies_on_terminal_status_change() {
        let path = temp_agent_status_path("monitor");
        let mut monitor = AgentStatusFileMonitor::new(Some(path.clone()));

        assert!(monitor.update("work", PaneId(7)).is_none());

        fs::write(
            &path,
            r#"
state = "running"
summary = "started"
"#,
        )
        .unwrap();
        assert!(monitor.update("work", PaneId(7)).is_none());
        assert!(monitor.is_running());

        fs::write(
            &path,
            r#"
state = "done"
summary = "implemented and tested"
updated_token = "1"
"#,
        )
        .unwrap();
        let notification = monitor.update("work", PaneId(7)).unwrap();

        assert!(!monitor.is_running());
        assert_eq!(notification.title, "neovide-tabs");
        assert_eq!(notification.subtitle, Some("work · pane 7".to_owned()));
        assert_eq!(notification.body, "Agent done: implemented and tested");
        assert!(monitor.update("work", PaneId(7)).is_none());

        fs::write(
            &path,
            r#"
state = "done"
summary = "implemented and tested"
updated_token = "2"
"#,
        )
        .unwrap();
        assert!(monitor.update("work", PaneId(7)).is_some());
        let _ = fs::remove_file(path);
    }

    #[test]
    fn agent_spinner_prefers_explicit_status_over_screen_busy() {
        assert!(should_show_agent_spinner(false, false, true));
        assert!(should_show_agent_spinner(true, true, true));
        assert!(!should_show_agent_spinner(true, false, true));
        assert!(!should_show_agent_spinner(true, false, false));
    }

    #[test]
    fn classifies_agent_screens() {
        let busy = classify_agent_screen(
            "Working (3m 12s - esc to interrupt) 2.1k tokens thinking with xhigh effort",
        );
        assert_eq!(busy.kind, Some(AgentKind::Codex));
        assert_eq!(busy.state, AgentScreenState::Busy);

        let needs_input =
            classify_agent_screen("Claude Code requires confirmation. Do you want to proceed?");
        assert_eq!(needs_input.kind, Some(AgentKind::Claude));
        assert_eq!(needs_input.state, AgentScreenState::NeedsInput);
    }

    #[test]
    fn applescript_string_escapes_quotes_and_backslashes() {
        assert_eq!(
            applescript_string(r#"a "quoted" \ path"#),
            r#""a \"quoted\" \\ path""#
        );
    }

    #[test]
    fn percent_decode_rejects_incomplete_escape() {
        assert_eq!(
            percent_decode_utf8("/tmp/a%20b"),
            Some("/tmp/a b".to_owned())
        );
        assert_eq!(percent_decode_utf8("/tmp/%"), None);
    }

    #[test]
    fn pane_layout_roundtrips_to_session_layout() {
        let layout = PaneLayout::Split {
            axis: SplitAxis::Vertical,
            first: Box::new(PaneLayout::Leaf(PaneId(1))),
            second: Box::new(PaneLayout::Split {
                axis: SplitAxis::Horizontal,
                first: Box::new(PaneLayout::Leaf(PaneId(2))),
                second: Box::new(PaneLayout::Leaf(PaneId(3))),
            }),
        };

        let stored = layout.to_stored();

        assert_eq!(
            stored,
            StoredPaneLayout::Split {
                axis: StoredSplitAxis::Vertical,
                first: Box::new(StoredPaneLayout::Leaf { pane: 1 }),
                second: Box::new(StoredPaneLayout::Split {
                    axis: StoredSplitAxis::Horizontal,
                    first: Box::new(StoredPaneLayout::Leaf { pane: 2 }),
                    second: Box::new(StoredPaneLayout::Leaf { pane: 3 }),
                }),
            }
        );
        assert_eq!(stored.to_runtime(), layout);
    }

    #[test]
    fn session_state_serializes_tabs_panes_and_layout() {
        let state = SessionState {
            active_tab: 1,
            tabs: vec![SessionTabState {
                title: "work".to_owned(),
                active_pane: 2,
                theme: "Graphite".to_owned(),
                panes: vec![
                    SessionPaneState {
                        id: 1,
                        cwd: Some(PathBuf::from("/tmp/a")),
                    },
                    SessionPaneState {
                        id: 2,
                        cwd: Some(PathBuf::from("/tmp/b")),
                    },
                ],
                layout: StoredPaneLayout::Split {
                    axis: StoredSplitAxis::Vertical,
                    first: Box::new(StoredPaneLayout::Leaf { pane: 1 }),
                    second: Box::new(StoredPaneLayout::Leaf { pane: 2 }),
                },
            }],
        };

        let encoded = toml::to_string(&state).unwrap();
        let decoded = toml::from_str::<SessionState>(&encoded).unwrap();

        assert_eq!(decoded, state);
    }

    #[test]
    fn session_title_number_parses_default_titles() {
        assert_eq!(session_title_number("session 12"), Some(12));
        assert_eq!(session_title_number("work"), None);
    }

    #[test]
    fn pane_layout_splits_active_leaf() {
        let mut layout = PaneLayout::Leaf(PaneId(1));

        assert!(layout.split_leaf(PaneId(1), PaneId(2), SplitAxis::Vertical));

        assert_eq!(
            layout,
            PaneLayout::Split {
                axis: SplitAxis::Vertical,
                first: Box::new(PaneLayout::Leaf(PaneId(1))),
                second: Box::new(PaneLayout::Leaf(PaneId(2))),
            }
        );
    }

    #[test]
    fn pane_layout_remove_leaf_collapses_split() {
        let layout = PaneLayout::Split {
            axis: SplitAxis::Horizontal,
            first: Box::new(PaneLayout::Leaf(PaneId(1))),
            second: Box::new(PaneLayout::Leaf(PaneId(2))),
        };

        assert_eq!(
            layout.without_leaf(PaneId(1)),
            Some(PaneLayout::Leaf(PaneId(2)))
        );
    }

    #[test]
    fn cursor_motion_initializes_at_target() {
        let mut motion = CursorMotion::new();
        let cursor = CursorView {
            x: 4,
            y: 2,
            style: CursorVisualStyle::Block,
        };

        let animated = motion.update(Some(cursor), 1.0 / 60.0).unwrap();

        assert_eq!(animated.target.x, 4);
        assert_eq!(animated.target.y, 2);
        assert_eq!(animated.tail_x, 4.0);
        assert_eq!(animated.tail_y, 2.0);
    }

    #[test]
    fn cursor_motion_trails_after_jump() {
        let mut motion = CursorMotion::new();
        let first = CursorView {
            x: 0,
            y: 0,
            style: CursorVisualStyle::Block,
        };
        let second = CursorView {
            x: 0,
            y: 5,
            style: CursorVisualStyle::Block,
        };

        motion.update(Some(first), 1.0 / 60.0);
        let animated = motion.update(Some(second), 1.0 / 60.0).unwrap();

        assert_eq!(animated.target.y, 5);
        assert_eq!(animated.tail_y, 0.0);

        let animated = motion.update(Some(second), 1.0 / 60.0).unwrap();
        assert!(animated.tail_y > 0.0);
        assert!(animated.tail_y < 5.0);
    }

    #[test]
    fn short_horizontal_cursor_moves_use_short_animation_length() {
        assert_eq!(
            cursor_animation_length(2.0, 3.0, 3.0, 3.0),
            super::CURSOR_SHORT_ANIMATION_LENGTH
        );
        assert_eq!(
            cursor_animation_length(2.0, 3.0, 2.0, 4.0),
            super::CURSOR_ANIMATION_LENGTH
        );
    }

    #[test]
    fn animation_alpha_reaches_near_target_after_length() {
        let alpha = animation_alpha(
            super::CURSOR_ANIMATION_LENGTH,
            super::CURSOR_ANIMATION_LENGTH,
        );

        assert!(alpha > 0.94);
        assert!(alpha < 1.0);
    }

    #[test]
    fn cursor_trail_rect_spans_same_row_cells() {
        assert_eq!(
            cursor_trail_rect(10.0, 20.0, 50.0, 20.0, 10.0, 20.0),
            Some(CursorTrailRect {
                x: 10.0,
                y: 20.0,
                width: 50.0,
                height: 20.0,
            })
        );
    }

    #[test]
    fn cursor_trail_rect_spans_same_column_cells() {
        assert_eq!(
            cursor_trail_rect(30.0, 20.0, 30.0, 80.0, 10.0, 20.0),
            Some(CursorTrailRect {
                x: 30.0,
                y: 20.0,
                width: 10.0,
                height: 80.0,
            })
        );
    }

    #[test]
    fn cursor_trail_rect_leaves_diagonal_moves_to_line_trail() {
        assert_eq!(cursor_trail_rect(10.0, 20.0, 50.0, 80.0, 10.0, 20.0), None);
    }
}
