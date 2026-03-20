//! WOA Camera — Viewport into the game world
//!
//! Smooth lerp follow with look-ahead.
//! All coordinates are in internal resolution (320×200).

pub struct Camera {
    /// Camera position (top-left of viewport in world coords)
    pub x: i32,
    pub y: i32,
    /// Viewport dimensions
    pub vw: u32,
    pub vh: u32,
}

impl Camera {
    pub fn new(viewport_w: u32, viewport_h: u32) -> Self {
        Self {
            x: 0,
            y: 0,
            vw: viewport_w,
            vh: viewport_h,
        }
    }

    /// Smoothly follow a target position (center of viewport)
    /// lerp_factor: 0.0 = no movement, 1.0 = instant snap
    pub fn follow(&mut self, target_x: i32, target_y: i32) {
        let center_x = target_x - (self.vw / 2) as i32;
        let center_y = target_y - (self.vh / 2) as i32;

        // Lerp factor ~0.1 for smooth follow (fixed-point: 1/8)
        self.x += (center_x - self.x) / 8;
        self.y += (center_y - self.y) / 8;
    }

    /// Snap camera to exact position (no lerp)
    pub fn snap(&mut self, target_x: i32, target_y: i32) {
        self.x = target_x - (self.vw / 2) as i32;
        self.y = target_y - (self.vh / 2) as i32;
    }

    /// Clamp camera within world bounds
    pub fn clamp(&mut self, world_w: i32, world_h: i32) {
        self.x = self.x.clamp(0, (world_w - self.vw as i32).max(0));
        self.y = self.y.clamp(0, (world_h - self.vh as i32).max(0));
    }

    /// Convert world position to screen position
    #[inline]
    pub fn world_to_screen(&self, wx: i32, wy: i32) -> (i32, i32) {
        (wx - self.x, wy - self.y)
    }
}
