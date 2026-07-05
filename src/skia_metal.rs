#[derive(Clone, Copy, Debug)]
pub struct SkiaRenderGeometry {
    pub width: i32,
    pub height: i32,
    pub origin_x: f32,
    pub origin_y: f32,
    pub cell_width: f32,
    pub cell_height: f32,
}

#[cfg(target_os = "macos")]
mod platform {
    use super::SkiaRenderGeometry;
    use crate::{
        neovide_render::{NeovideRenderedWindowSnapshot, NeovideRendererModelSnapshot},
        neovide_text::{NeovideTextRenderer, TextGridGeometry},
        neovim_runtime::NativeNeovimRuntime,
        terminal_runtime::{
            NativeTerminalRuntime, TerminalCellSnapshot, TerminalColor, TerminalCursorSnapshot,
        },
    };
    use skia_safe::{
        Canvas, Color, ColorSpace, ColorType, Paint, PaintStyle, Rect, Surface, SurfaceProps,
        SurfacePropsFlags,
        gpu::{
            self, DirectContext, SurfaceOrigin,
            mtl::{BackendContext, TextureInfo},
            surfaces::wrap_backend_render_target,
        },
    };
    use std::{
        cmp::Ordering,
        ffi::c_void,
        time::{Duration, Instant},
    };

    const CURSOR_TRAIL_SECONDS: f32 = 0.16;
    const CURSOR_TRAIL_ALPHA: u8 = 145;
    const CURSOR_BODY_ALPHA: u8 = 199;
    const MAX_ANIMATION_DT: f32 = 1.0 / 30.0;

    pub struct NativeSkiaMetalRenderer {
        context: DirectContext,
        _backend: BackendContext,
        text_renderer: NeovideTextRenderer,
        cursor_trail: CursorTrailState,
        cursor_blink: CursorBlinkState,
        runtime_id: Option<usize>,
        last_frame_at: Option<Instant>,
        scroll_animation_active: bool,
    }

    impl NativeSkiaMetalRenderer {
        /// # Safety
        ///
        /// `device` and `command_queue` must point to live Metal protocol objects
        /// that outlive the renderer.
        pub unsafe fn new(device: *mut c_void, command_queue: *mut c_void) -> Option<Self> {
            if device.is_null() || command_queue.is_null() {
                return None;
            }
            // SAFETY: Swift passes live MTLDevice and MTLCommandQueue protocol object pointers.
            let backend = unsafe { BackendContext::new(device, command_queue) };
            let context = gpu::direct_contexts::make_metal(&backend, None)?;
            Some(Self {
                context,
                _backend: backend,
                text_renderer: NeovideTextRenderer::new(),
                cursor_trail: CursorTrailState::default(),
                cursor_blink: CursorBlinkState::default(),
                runtime_id: None,
                last_frame_at: None,
                scroll_animation_active: false,
            })
        }

        /// # Safety
        ///
        /// `texture` must point to the current drawable MTLTexture and remain valid
        /// until Skia has flushed this frame.
        pub unsafe fn render_nvim(
            &mut self,
            runtime: &mut NativeNeovimRuntime,
            texture: *mut c_void,
            geometry: SkiaRenderGeometry,
        ) -> bool {
            // SAFETY: `texture` is the current drawable texture pointer for this frame.
            let Some(mut surface) = (unsafe { self.surface(texture, geometry) }) else {
                return false;
            };
            self.reset_state_if_runtime_changed(runtime as *const NativeNeovimRuntime as usize);
            let dt = self.animation_dt();
            let model = runtime.renderer_model();
            self.cursor_trail.update(model.cursor.as_ref());
            self.cursor_blink.update(model.cursor.as_ref());
            draw_model(
                surface.canvas(),
                &mut self.text_renderer,
                &self.cursor_trail,
                self.cursor_blink.should_render(),
                &model,
                geometry,
            );
            self.scroll_animation_active =
                runtime.advance_renderer_animations(dt) || runtime.has_active_renderer_animation();
            self.context.flush_and_submit();
            true
        }

        /// # Safety
        ///
        /// `texture` must point to the current drawable MTLTexture and remain valid
        /// until Skia has flushed this frame.
        pub unsafe fn render_terminal(
            &mut self,
            runtime: &mut NativeTerminalRuntime,
            texture: *mut c_void,
            geometry: SkiaRenderGeometry,
        ) -> bool {
            // SAFETY: `texture` is the current drawable texture pointer for this frame.
            let Some(mut surface) = (unsafe { self.surface(texture, geometry) }) else {
                return false;
            };
            self.reset_state_if_runtime_changed(runtime as *const NativeTerminalRuntime as usize);
            let dt = self.animation_dt();
            let Ok(model) = runtime.renderer_model() else {
                return false;
            };
            self.cursor_trail.update(model.cursor.as_ref());
            self.cursor_blink.update(model.cursor.as_ref());
            draw_model(
                surface.canvas(),
                &mut self.text_renderer,
                &self.cursor_trail,
                self.cursor_blink.should_render(),
                &model,
                geometry,
            );
            self.scroll_animation_active =
                runtime.advance_renderer_animations(dt) || runtime.has_active_renderer_animation();
            self.context.flush_and_submit();
            true
        }

        pub fn needs_animation_frame(&self) -> bool {
            self.next_frame_delay_ms().is_some()
        }

        pub fn next_frame_delay_ms(&self) -> Option<u64> {
            if self.cursor_trail.needs_animation_frame() || self.scroll_animation_active {
                return Some(0);
            }
            self.cursor_blink.next_frame_delay_ms()
        }

        unsafe fn surface(
            &mut self,
            texture: *mut c_void,
            geometry: SkiaRenderGeometry,
        ) -> Option<Surface> {
            if texture.is_null() || geometry.width <= 0 || geometry.height <= 0 {
                return None;
            }
            // SAFETY: Swift passes the current drawable MTLTexture pointer for this frame.
            let texture_info = unsafe { TextureInfo::new(texture) };
            let backend = gpu::backend_render_targets::make_mtl(
                (geometry.width, geometry.height),
                &texture_info,
            );
            wrap_backend_render_target(
                &mut self.context,
                &backend,
                SurfaceOrigin::TopLeft,
                ColorType::BGRA8888,
                ColorSpace::new_srgb(),
                Some(surface_props()).as_ref(),
            )
        }

        fn reset_state_if_runtime_changed(&mut self, runtime_id: usize) {
            if self.runtime_id == Some(runtime_id) {
                return;
            }
            self.runtime_id = Some(runtime_id);
            self.cursor_trail.clear();
            self.cursor_blink.clear();
            self.last_frame_at = None;
            self.scroll_animation_active = false;
        }

        fn animation_dt(&mut self) -> f32 {
            let now = Instant::now();
            let dt = self
                .last_frame_at
                .replace(now)
                .map_or(0.0, |previous| now.duration_since(previous).as_secs_f32());
            dt.min(MAX_ANIMATION_DT)
        }
    }

    fn draw_model(
        canvas: &Canvas,
        text_renderer: &mut NeovideTextRenderer,
        cursor_trail: &CursorTrailState,
        cursor_visible: bool,
        model: &NeovideRendererModelSnapshot,
        geometry: SkiaRenderGeometry,
    ) {
        text_renderer.update_geometry(text_grid_geometry(geometry));
        canvas.clear(color(model.background));
        fill_content(canvas, model.background, geometry);
        for window in sorted_windows(&model.windows) {
            draw_window(canvas, text_renderer, window, geometry);
        }
        text_renderer.cleanup_font_cache();
        draw_cursor(canvas, cursor_trail, cursor_visible, model, geometry);
    }

    fn text_grid_geometry(geometry: SkiaRenderGeometry) -> TextGridGeometry {
        TextGridGeometry {
            origin_x: geometry.origin_x,
            origin_y: geometry.origin_y,
            cell_width: geometry.cell_width,
            cell_height: geometry.cell_height,
        }
    }

    fn fill_content(canvas: &Canvas, background: TerminalColor, geometry: SkiaRenderGeometry) {
        let rect = Rect::from_xywh(
            geometry.origin_x,
            geometry.origin_y,
            geometry.width as f32 - geometry.origin_x * 2.0,
            geometry.height as f32 - geometry.origin_y,
        );
        let mut paint = Paint::default();
        paint.set_color(color(background));
        canvas.draw_rect(rect, &paint);
    }

    fn sorted_windows(
        windows: &[NeovideRenderedWindowSnapshot],
    ) -> Vec<&NeovideRenderedWindowSnapshot> {
        let mut windows = windows
            .iter()
            .filter(|window| !window.hidden)
            .collect::<Vec<_>>();
        windows.sort_by(window_order);
        windows
    }

    fn window_order(
        left: &&NeovideRenderedWindowSnapshot,
        right: &&NeovideRenderedWindowSnapshot,
    ) -> Ordering {
        (left.zindex, left.compindex, left.grid_id).cmp(&(
            right.zindex,
            right.compindex,
            right.grid_id,
        ))
    }

    fn draw_window(
        canvas: &Canvas,
        text_renderer: &mut NeovideTextRenderer,
        window: &NeovideRenderedWindowSnapshot,
        geometry: SkiaRenderGeometry,
    ) {
        let inner = window.inner_row_range();
        draw_fixed_lines(canvas, text_renderer, window, 0..inner.start, geometry);
        draw_scrollable_lines(canvas, text_renderer, window, inner.clone(), geometry);
        draw_fixed_lines(
            canvas,
            text_renderer,
            window,
            inner.end..window.height,
            geometry,
        );
    }

    fn draw_fixed_lines(
        canvas: &Canvas,
        text_renderer: &mut NeovideTextRenderer,
        window: &NeovideRenderedWindowSnapshot,
        rows: std::ops::Range<usize>,
        geometry: SkiaRenderGeometry,
    ) {
        for row in rows {
            let Some(line) = window.lines.get(row).and_then(Option::as_ref) else {
                continue;
            };
            draw_line(
                canvas,
                text_renderer,
                line,
                window.top as f32 + row as f32,
                window,
                geometry,
            );
        }
    }

    fn draw_scrollable_lines(
        canvas: &Canvas,
        text_renderer: &mut NeovideTextRenderer,
        window: &NeovideRenderedWindowSnapshot,
        inner: std::ops::Range<usize>,
        geometry: SkiaRenderGeometry,
    ) {
        if inner.is_empty() {
            return;
        }

        canvas.save();
        canvas.clip_rect(
            scroll_clip_rect(window, inner.clone(), geometry),
            None,
            Some(false),
        );
        let floor = window.scroll_position.floor();
        let row_offset = floor - window.scroll_position;
        let signed_start = floor as isize;
        for inner_row in 0..=inner.len() {
            let Some(line) = window.scrollback_line(signed_start + inner_row as isize) else {
                continue;
            };
            let row = window.top as f32 + inner.start as f32 + inner_row as f32 + row_offset;
            draw_line(canvas, text_renderer, line, row, window, geometry);
        }
        canvas.restore();
    }

    fn draw_line(
        canvas: &Canvas,
        text_renderer: &mut NeovideTextRenderer,
        line: &crate::neovide_render::NeovideLine,
        row: f32,
        window: &NeovideRenderedWindowSnapshot,
        geometry: SkiaRenderGeometry,
    ) {
        for (col, cell) in line.cells.iter().take(window.width).enumerate() {
            draw_cell_background_if_needed(canvas, cell, row, window.left + col, geometry);
        }
        text_renderer.draw_line(canvas, &line.cells, row, window.left, window.width);
    }

    fn scroll_clip_rect(
        window: &NeovideRenderedWindowSnapshot,
        inner: std::ops::Range<usize>,
        geometry: SkiaRenderGeometry,
    ) -> Rect {
        Rect::from_xywh(
            geometry.origin_x + window.left as f32 * geometry.cell_width,
            geometry.origin_y + (window.top + inner.start) as f32 * geometry.cell_height,
            window.width as f32 * geometry.cell_width,
            inner.len() as f32 * geometry.cell_height,
        )
    }

    fn draw_cell_background_if_needed(
        canvas: &Canvas,
        cell: &TerminalCellSnapshot,
        row: f32,
        col: usize,
        geometry: SkiaRenderGeometry,
    ) {
        let x = geometry.origin_x + col as f32 * geometry.cell_width;
        let y = geometry.origin_y + row * geometry.cell_height;
        if let Some(background) = cell.bg {
            draw_cell_background(canvas, background, cell.blend, x, y, geometry);
        }
    }

    fn draw_cell_background(
        canvas: &Canvas,
        background: TerminalColor,
        blend: u8,
        x: f32,
        y: f32,
        geometry: SkiaRenderGeometry,
    ) {
        let mut paint = Paint::default();
        paint.set_color(color_with_blend(background, blend));
        canvas.draw_rect(
            Rect::from_xywh(x, y, geometry.cell_width, geometry.cell_height),
            &paint,
        );
    }

    fn draw_cursor(
        canvas: &Canvas,
        cursor_trail: &CursorTrailState,
        cursor_visible: bool,
        model: &NeovideRendererModelSnapshot,
        geometry: SkiaRenderGeometry,
    ) {
        let Some(cursor) = &model.cursor else {
            return;
        };
        if !cursor_visible {
            return;
        }
        let rect = cursor_rect(cursor.x as f32, cursor.y as f32, geometry);
        draw_cursor_trail(canvas, cursor_trail, rect, model.cursor_color, geometry);
        draw_cursor_body(canvas, cursor, rect, model.cursor_color);
    }

    fn draw_cursor_trail(
        canvas: &Canvas,
        cursor_trail: &CursorTrailState,
        target: Rect,
        color: TerminalColor,
        geometry: SkiaRenderGeometry,
    ) {
        let Some(tail) = cursor_trail.animated_tail() else {
            return;
        };
        let tail_rect = cursor_rect(tail.x, tail.y, geometry);
        let dx = target.left - tail_rect.left;
        let dy = target.top - tail_rect.top;
        let distance = dx.hypot(dy);
        if distance < 0.75 {
            return;
        }

        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_color(color_with_alpha(color, CURSOR_TRAIL_ALPHA));
        if let Some(rect) = cursor_trail_rect(tail_rect, target, geometry) {
            canvas.draw_rect(rect, &paint);
            return;
        }

        paint.set_style(PaintStyle::Stroke);
        paint.set_stroke_width(geometry.cell_width.max(geometry.cell_height) * 0.72);
        canvas.draw_line(
            (tail_rect.center_x(), tail_rect.center_y()),
            (target.center_x(), target.center_y()),
            &paint,
        );
    }

    fn draw_cursor_body(
        canvas: &Canvas,
        cursor: &TerminalCursorSnapshot,
        rect: Rect,
        color: TerminalColor,
    ) {
        let mut paint = Paint::default();
        paint.set_color(color_with_alpha(color, CURSOR_BODY_ALPHA));
        match cursor.style {
            "bar" => {
                canvas.draw_rect(
                    Rect::from_xywh(
                        rect.left,
                        rect.top,
                        cursor_thickness(cursor.cell_percentage, rect.width()),
                        rect.height(),
                    ),
                    &paint,
                );
            }
            "underline" => {
                let height = cursor_thickness(cursor.cell_percentage, rect.height());
                canvas.draw_rect(
                    Rect::from_xywh(rect.left, rect.bottom - height, rect.width(), height),
                    &paint,
                );
            }
            _ => {
                canvas.draw_rect(rect, &paint);
            }
        };
    }

    fn cursor_thickness(percentage: u8, size: f32) -> f32 {
        let fraction = percentage.clamp(1, 100) as f32 / 100.0;
        (size * fraction).clamp(1.0, size)
    }

    fn cursor_trail_rect(tail: Rect, target: Rect, geometry: SkiaRenderGeometry) -> Option<Rect> {
        let dx = (target.left - tail.left).abs();
        let dy = (target.top - tail.top).abs();
        if dx < 0.75 && dy < 0.75 {
            return None;
        }
        if dy <= geometry.cell_height * 0.25 {
            return Some(Rect::from_xywh(
                tail.left.min(target.left),
                target.top,
                dx + geometry.cell_width,
                geometry.cell_height,
            ));
        }
        if dx <= geometry.cell_width * 0.25 {
            return Some(Rect::from_xywh(
                target.left,
                tail.top.min(target.top),
                geometry.cell_width,
                dy + geometry.cell_height,
            ));
        }
        None
    }

    fn cursor_rect(x: f32, y: f32, geometry: SkiaRenderGeometry) -> Rect {
        Rect::from_xywh(
            geometry.origin_x + x * geometry.cell_width,
            geometry.origin_y + y * geometry.cell_height,
            geometry.cell_width,
            geometry.cell_height,
        )
    }

    fn surface_props() -> SurfaceProps {
        SurfaceProps::new_with_text_properties(
            SurfacePropsFlags::default(),
            skia_safe::PixelGeometry::RGBH,
            0.0,
            0.0,
        )
    }

    fn color(color: TerminalColor) -> Color {
        Color::from_argb(255, color.r, color.g, color.b)
    }

    fn color_with_alpha(color: TerminalColor, alpha: u8) -> Color {
        Color::from_argb(alpha, color.r, color.g, color.b)
    }

    fn color_with_blend(color: TerminalColor, blend: u8) -> Color {
        let alpha = 255_u16.saturating_mul(100_u16.saturating_sub(blend.min(100) as u16)) / 100;
        Color::from_argb(alpha as u8, color.r, color.g, color.b)
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    struct GridPoint {
        x: f32,
        y: f32,
    }

    impl GridPoint {
        fn from_cursor(cursor: &TerminalCursorSnapshot) -> Self {
            Self {
                x: cursor.x as f32,
                y: cursor.y as f32,
            }
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct CursorBlinkSignature {
        x: u16,
        y: u16,
        style: &'static str,
        cell_percentage: u8,
        blinkwait_ms: u64,
        blinkon_ms: u64,
        blinkoff_ms: u64,
    }

    impl CursorBlinkSignature {
        fn from_cursor(cursor: &TerminalCursorSnapshot) -> Self {
            Self {
                x: cursor.x,
                y: cursor.y,
                style: cursor.style,
                cell_percentage: cursor.cell_percentage,
                blinkwait_ms: cursor.blinkwait_ms,
                blinkon_ms: cursor.blinkon_ms,
                blinkoff_ms: cursor.blinkoff_ms,
            }
        }

        fn is_static(self) -> bool {
            self.blinkon_ms == 0 || self.blinkoff_ms == 0
        }

        fn delay_for(self, phase: CursorBlinkPhase) -> Duration {
            let millis = match phase {
                CursorBlinkPhase::Waiting => self.blinkwait_ms,
                CursorBlinkPhase::On => self.blinkon_ms,
                CursorBlinkPhase::Off => self.blinkoff_ms,
            };
            Duration::from_millis(millis)
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum CursorBlinkPhase {
        Waiting,
        On,
        Off,
    }

    impl CursorBlinkPhase {
        fn next(self) -> Self {
            match self {
                Self::Waiting | Self::Off => Self::On,
                Self::On => Self::Off,
            }
        }
    }

    #[derive(Default)]
    struct CursorBlinkState {
        phase: Option<CursorBlinkPhase>,
        transition_at: Option<Instant>,
        cursor: Option<CursorBlinkSignature>,
    }

    impl CursorBlinkState {
        fn update(&mut self, cursor: Option<&TerminalCursorSnapshot>) {
            self.update_at(Instant::now(), cursor);
        }

        fn update_at(&mut self, now: Instant, cursor: Option<&TerminalCursorSnapshot>) {
            let Some(cursor) = cursor.map(CursorBlinkSignature::from_cursor) else {
                self.clear();
                return;
            };
            if self.cursor != Some(cursor) {
                self.start(now, cursor);
            }
            if cursor.is_static() {
                self.phase = Some(CursorBlinkPhase::Waiting);
                self.transition_at = None;
                return;
            }
            self.advance(now, cursor);
        }

        fn clear(&mut self) {
            self.phase = None;
            self.transition_at = None;
            self.cursor = None;
        }

        fn should_render(&self) -> bool {
            !matches!(self.phase, Some(CursorBlinkPhase::Off))
        }

        fn next_frame_delay_ms(&self) -> Option<u64> {
            let deadline = self.transition_at?;
            let delay = deadline.saturating_duration_since(Instant::now());
            Some(delay.as_millis().min(u64::MAX as u128) as u64)
        }

        fn start(&mut self, now: Instant, cursor: CursorBlinkSignature) {
            self.cursor = Some(cursor);
            let phase = if cursor.blinkwait_ms > 0 {
                CursorBlinkPhase::Waiting
            } else {
                CursorBlinkPhase::On
            };
            self.phase = Some(phase);
            self.transition_at = (!cursor.is_static()).then_some(now + cursor.delay_for(phase));
        }

        fn advance(&mut self, now: Instant, cursor: CursorBlinkSignature) {
            let (Some(mut phase), Some(mut transition_at)) = (self.phase, self.transition_at)
            else {
                return;
            };
            if transition_at > now {
                return;
            }
            phase = phase.next();
            transition_at += cursor.delay_for(phase);
            if transition_at <= now {
                transition_at = now + cursor.delay_for(phase);
            }
            self.phase = Some(phase);
            self.transition_at = Some(transition_at);
        }
    }

    #[derive(Default)]
    struct CursorTrailState {
        start: Option<GridPoint>,
        target: Option<GridPoint>,
        started_at: Option<Instant>,
    }

    impl CursorTrailState {
        fn update(&mut self, cursor: Option<&TerminalCursorSnapshot>) {
            let Some(cursor) = cursor else {
                self.clear();
                return;
            };
            let next = GridPoint::from_cursor(cursor);
            if self.target.is_none() {
                self.target = Some(next);
                return;
            }
            if self.target == Some(next) {
                return;
            }
            self.start = self.animated_tail().or(self.target);
            self.target = Some(next);
            self.started_at = Some(Instant::now());
        }

        fn clear(&mut self) {
            self.start = None;
            self.target = None;
            self.started_at = None;
        }

        fn animated_tail(&self) -> Option<GridPoint> {
            let start = self.start?;
            let target = self.target?;
            let progress = self.progress()?;
            let eased = 1.0 - (1.0 - progress).powi(3);
            Some(GridPoint {
                x: start.x + (target.x - start.x) * eased,
                y: start.y + (target.y - start.y) * eased,
            })
        }

        fn needs_animation_frame(&self) -> bool {
            self.progress().is_some()
        }

        fn progress(&self) -> Option<f32> {
            let elapsed = self.started_at?.elapsed().as_secs_f32();
            (elapsed < CURSOR_TRAIL_SECONDS).then_some(elapsed / CURSOR_TRAIL_SECONDS)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn cursor_blink_waits_then_toggles_off_and_on() {
            let now = Instant::now();
            let cursor = cursor(0, 0, "bar", 25, 300, 200, 150);
            let mut blink = CursorBlinkState::default();

            blink.update_at(now, Some(&cursor));
            assert!(blink.should_render());

            blink.update_at(now + Duration::from_millis(300), Some(&cursor));
            assert!(blink.should_render());

            blink.update_at(now + Duration::from_millis(500), Some(&cursor));
            assert!(!blink.should_render());

            blink.update_at(now + Duration::from_millis(650), Some(&cursor));
            assert!(blink.should_render());
        }

        #[test]
        fn cursor_blink_static_when_on_or_off_duration_is_zero() {
            let now = Instant::now();
            let cursor = cursor(0, 0, "bar", 25, 300, 0, 150);
            let mut blink = CursorBlinkState::default();

            blink.update_at(now, Some(&cursor));

            assert!(blink.should_render());
            assert!(blink.next_frame_delay_ms().is_none());
        }

        #[test]
        fn cursor_blink_resets_visible_when_cursor_changes() {
            let now = Instant::now();
            let first_cursor = cursor(0, 0, "bar", 25, 0, 200, 150);
            let moved = cursor(1, 0, "bar", 25, 0, 200, 150);
            let mut blink = CursorBlinkState::default();
            blink.update_at(now, Some(&first_cursor));
            blink.update_at(now + Duration::from_millis(200), Some(&first_cursor));
            assert!(!blink.should_render());

            blink.update_at(now + Duration::from_millis(200), Some(&moved));

            assert!(blink.should_render());
        }

        fn cursor(
            x: u16,
            y: u16,
            style: &'static str,
            cell_percentage: u8,
            blinkwait_ms: u64,
            blinkon_ms: u64,
            blinkoff_ms: u64,
        ) -> TerminalCursorSnapshot {
            TerminalCursorSnapshot {
                x,
                y,
                style,
                cell_percentage,
                blinkwait_ms,
                blinkon_ms,
                blinkoff_ms,
            }
        }
    }
}

#[cfg(not(target_os = "macos"))]
mod platform {
    use super::SkiaRenderGeometry;
    use crate::{neovim_runtime::NativeNeovimRuntime, terminal_runtime::NativeTerminalRuntime};
    use std::ffi::c_void;

    pub struct NativeSkiaMetalRenderer;

    impl NativeSkiaMetalRenderer {
        pub unsafe fn new(_device: *mut c_void, _command_queue: *mut c_void) -> Option<Self> {
            None
        }

        pub unsafe fn render_nvim(
            &mut self,
            _runtime: &mut NativeNeovimRuntime,
            _texture: *mut c_void,
            _geometry: SkiaRenderGeometry,
        ) -> bool {
            false
        }

        pub unsafe fn render_terminal(
            &mut self,
            _runtime: &mut NativeTerminalRuntime,
            _texture: *mut c_void,
            _geometry: SkiaRenderGeometry,
        ) -> bool {
            false
        }

        pub fn needs_animation_frame(&self) -> bool {
            false
        }

        pub fn next_frame_delay_ms(&self) -> Option<u64> {
            None
        }
    }
}

pub use platform::NativeSkiaMetalRenderer;
