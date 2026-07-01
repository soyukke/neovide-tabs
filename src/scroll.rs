#[derive(Debug, Clone, Copy)]
pub struct SmoothScroll {
    visual_offset_rows: f32,
    target_offset_rows: f32,
    velocity_rows: f32,
    stiffness: f32,
    damping: f32,
    snap_epsilon: f32,
}

impl Default for SmoothScroll {
    fn default() -> Self {
        Self {
            visual_offset_rows: 0.0,
            target_offset_rows: 0.0,
            velocity_rows: 0.0,
            stiffness: 180.0,
            damping: 27.0,
            snap_epsilon: 0.002,
        }
    }
}

#[cfg_attr(not(test), allow(dead_code))]
impl SmoothScroll {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_target_rows(&mut self, target_offset_rows: f32) {
        self.target_offset_rows = target_offset_rows;
    }

    pub fn add_target_rows(&mut self, delta_rows: f32) {
        self.target_offset_rows += delta_rows;
    }

    pub fn set_all_rows(&mut self, offset_rows: f32) {
        self.target_offset_rows = offset_rows;
        self.visual_offset_rows = offset_rows;
        self.velocity_rows = 0.0;
    }

    pub fn visual_offset_rows(&self) -> f32 {
        self.visual_offset_rows
    }

    pub fn target_offset_rows(&self) -> f32 {
        self.target_offset_rows
    }

    pub fn is_animating(&self) -> bool {
        (self.target_offset_rows - self.visual_offset_rows).abs() > self.snap_epsilon
            || self.velocity_rows.abs() > self.snap_epsilon
    }

    pub fn update(&mut self, dt_seconds: f32) {
        if dt_seconds <= 0.0 {
            return;
        }

        let dt = dt_seconds.min(1.0 / 15.0);
        let displacement = self.target_offset_rows - self.visual_offset_rows;
        let acceleration = self.stiffness * displacement - self.damping * self.velocity_rows;

        self.velocity_rows += acceleration * dt;
        self.visual_offset_rows += self.velocity_rows * dt;

        if displacement.abs() < self.snap_epsilon && self.velocity_rows.abs() < self.snap_epsilon {
            self.visual_offset_rows = self.target_offset_rows;
            self.velocity_rows = 0.0;
        }
    }

    pub fn consume_history_scroll_request(&mut self, delta_rows: f32) -> isize {
        self.add_target_rows(delta_rows);

        let whole_rows = self.target_offset_rows.trunc();
        self.target_offset_rows -= whole_rows;

        whole_rows as isize
    }

    pub fn animate_history_rows(&mut self, rows: isize) {
        self.visual_offset_rows += rows as f32;
    }

    pub fn settle_fractional_offset(&mut self) {
        self.target_offset_rows = 0.0;
    }

    pub fn on_screen_shift(&mut self, shifted_rows: isize) {
        if shifted_rows == 0 {
            return;
        }

        self.visual_offset_rows += shifted_rows as f32;
        self.target_offset_rows = 0.0;
        self.velocity_rows = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::SmoothScroll;

    #[test]
    fn converges_to_target() {
        let mut scroll = SmoothScroll::new();
        scroll.set_target_rows(5.0);

        for _ in 0..180 {
            scroll.update(1.0 / 60.0);
        }

        assert!((scroll.visual_offset_rows() - 5.0).abs() < 0.01);
        assert!(!scroll.is_animating());
    }

    #[test]
    fn splits_fractional_history_scroll_from_terminal_rows() {
        let mut scroll = SmoothScroll::new();

        assert_eq!(scroll.consume_history_scroll_request(0.4), 0);
        assert_eq!(scroll.target_offset_rows(), 0.4);
        assert_eq!(scroll.visual_offset_rows(), 0.0);

        assert_eq!(scroll.consume_history_scroll_request(0.7), 1);
        assert!((scroll.target_offset_rows() - 0.1).abs() < 0.001);
        assert_eq!(scroll.visual_offset_rows(), 0.0);
    }

    #[test]
    fn animates_only_rows_that_the_terminal_applied() {
        let mut scroll = SmoothScroll::new();

        assert_eq!(scroll.consume_history_scroll_request(1.2), 1);
        scroll.animate_history_rows(0);
        assert_eq!(scroll.visual_offset_rows(), 0.0);

        scroll.animate_history_rows(1);
        assert!((scroll.visual_offset_rows() - 1.0).abs() < 0.001);
    }

    #[test]
    fn screen_shift_starts_from_row_delta_and_returns_to_zero() {
        let mut scroll = SmoothScroll::new();

        scroll.on_screen_shift(3);
        assert_eq!(scroll.target_offset_rows(), 0.0);
        assert_eq!(scroll.visual_offset_rows(), 3.0);

        for _ in 0..180 {
            scroll.update(1.0 / 60.0);
        }

        assert!(scroll.visual_offset_rows().abs() < 0.01);
    }
}
