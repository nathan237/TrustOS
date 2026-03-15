//! Gesture Recognizer
//!
//! State-machine based gesture detection for multitouch input.
//! Converts raw touch events from `touch.rs` into high-level gestures.
//!
//! Supported gestures:
//! - **Tap**: contact <200ms, displacement <10px → left click
//! - **Long press**: contact >500ms, immobile → right click / context menu
//! - **Swipe L/R/U/D**: Δ > 50px, velocity > 200px/s → back/forward/launcher/notifications
//! - **Pinch** (2 fingers): distance change → zoom in/out
//! - **Two-finger scroll**: same direction → scroll
//! - **Three-finger swipe**: horizontal → switch app (Alt+Tab)

use crate::touch::{self, TouchEvent, TouchPhase, TouchPoint, MAX_TOUCH_POINTS};

// ═════════════════════════════════════════════════════════════════════════════
// Constants (from roadmap specifications)
// ═════════════════════════════════════════════════════════════════════════════

/// Maximum tap duration in microseconds (200ms)
const TAP_MAX_DURATION_US: u64 = 200_000;

/// Maximum displacement for a tap (10 pixels)
const TAP_MAX_DISPLACEMENT: i32 = 10;

/// Long-press threshold in microseconds (500ms)
const LONG_PRESS_THRESHOLD_US: u64 = 500_000;

/// Long-press maximum displacement (fingers must stay still)
const LONG_PRESS_MAX_DISPLACEMENT: i32 = 15;

/// Minimum swipe distance in pixels (50px)
const SWIPE_MIN_DISTANCE: i32 = 50;

/// Minimum swipe velocity in pixels/second (200px/s)
const SWIPE_MIN_VELOCITY: i32 = 200;

/// Edge zone for edge swipes (from screen border, in pixels)
const EDGE_ZONE_PX: i32 = 30;

/// Pinch minimum distance change to trigger (15 pixels)
const PINCH_MIN_DELTA: i32 = 15;

/// Two-finger scroll: minimum movement to trigger (8 pixels)
const SCROLL_MIN_DELTA: i32 = 8;

/// Double-tap window in microseconds (300ms)
const DOUBLE_TAP_WINDOW_US: u64 = 300_000;

/// Maximum distance between two taps for double-tap (30 pixels)
const DOUBLE_TAP_MAX_DISTANCE: i32 = 30;

// ═════════════════════════════════════════════════════════════════════════════
// Gesture types
// ═════════════════════════════════════════════════════════════════════════════

/// Direction of a swipe gesture
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwipeDirection {
    Left,
    Right,
    Up,
    Down,
}

/// Origin of an edge swipe
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeOrigin {
    /// Swipe from bottom edge (open app launcher)
    Bottom,
    /// Swipe from top edge (notification panel)
    Top,
    /// Swipe from left edge
    Left,
    /// Swipe from right edge
    Right,
}

/// Recognized gesture event
#[derive(Debug, Clone, Copy)]
pub enum GestureEvent {
    /// Single tap at (x, y) — maps to left click
    Tap { x: i32, y: i32 },

    /// Double tap at (x, y) — maps to double click
    DoubleTap { x: i32, y: i32 },

    /// Long press at (x, y) — maps to right click / context menu
    LongPress { x: i32, y: i32 },

    /// Swipe gesture with direction and velocity
    Swipe {
        direction: SwipeDirection,
        start_x: i32,
        start_y: i32,
        end_x: i32,
        end_y: i32,
        velocity: i32, // pixels per second
    },

    /// Edge swipe (finger started at screen edge)
    EdgeSwipe {
        origin: EdgeOrigin,
        progress: i32, // how far the finger has moved from edge
    },

    /// Pinch gesture (2 fingers)
    Pinch {
        center_x: i32,
        center_y: i32,
        /// Scale factor * 100 (100 = no change, >100 = zoom in, <100 = zoom out)
        scale: i32,
    },

    /// Two-finger scroll
    Scroll {
        delta_x: i32,
        delta_y: i32,
    },

    /// Three-finger horizontal swipe (app switcher)
    ThreeFingerSwipe {
        direction: SwipeDirection,
    },

    /// Touch down (finger placed) — for visual feedback
    TouchDown { x: i32, y: i32 },

    /// Touch move (finger dragging) — for cursor tracking
    TouchMove { x: i32, y: i32 },

    /// All fingers lifted
    TouchUp { x: i32, y: i32 },

    /// Drag gesture (long press then move)
    Drag {
        x: i32,
        y: i32,
        start_x: i32,
        start_y: i32,
    },
}

// ═════════════════════════════════════════════════════════════════════════════
// Finger tracking
// ═════════════════════════════════════════════════════════════════════════════

/// Tracked state per finger
#[derive(Clone, Copy)]
struct FingerTracker {
    active: bool,
    id: u16,
    /// Position when finger first touched
    start_x: i32,
    start_y: i32,
    /// Current position
    current_x: i32,
    current_y: i32,
    /// Timestamp of first contact
    start_time_us: u64,
    /// Timestamp of latest update
    last_time_us: u64,
    /// Max displacement from start (for tap/long-press detection)
    max_displacement: i32,
}

impl Default for FingerTracker {
    fn default() -> Self {
        Self {
            active: false,
            id: 0,
            start_x: 0,
            start_y: 0,
            current_x: 0,
            current_y: 0,
            start_time_us: 0,
            last_time_us: 0,
            max_displacement: 0,
        }
    }
}

impl FingerTracker {
    const fn new() -> Self {
        Self {
            active: false,
            id: 0,
            start_x: 0,
            start_y: 0,
            current_x: 0,
            current_y: 0,
            start_time_us: 0,
            last_time_us: 0,
            max_displacement: 0,
        }
    }

    fn displacement(&self) -> i32 {
        let dx = self.current_x - self.start_x;
        let dy = self.current_y - self.start_y;
        // Use Manhattan distance for speed (no sqrt needed in kernel)
        dx.abs() + dy.abs()
    }

    fn euclidean_sq(&self) -> i32 {
        let dx = self.current_x - self.start_x;
        let dy = self.current_y - self.start_y;
        dx * dx + dy * dy
    }

    fn duration_us(&self) -> u64 {
        self.last_time_us.saturating_sub(self.start_time_us)
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// Recognizer state machine
// ═════════════════════════════════════════════════════════════════════════════

/// Internal state of the gesture recognizer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RecogState {
    /// No active gesture
    Idle,
    /// One or more fingers down, waiting to classify
    Tracking,
    /// Possible long press (timer running)
    PossibleLongPress,
    /// Confirmed drag (long-press + move)
    Dragging,
    /// Two-finger gesture in progress (pinch or scroll)
    TwoFinger,
    /// Three-finger gesture in progress
    ThreeFinger,
}

/// The gesture recognizer engine
pub struct GestureRecognizer {
    state: RecogState,
    /// Per-finger tracking (up to MAX_TOUCH_POINTS)
    fingers: [FingerTracker; MAX_TOUCH_POINTS],
    /// Number of fingers currently active
    finger_count: u8,
    /// Screen dimensions for edge detection
    screen_width: i32,
    screen_height: i32,
    /// Last tap location and time (for double-tap detection)
    last_tap_x: i32,
    last_tap_y: i32,
    last_tap_time_us: u64,
    /// Whether a long-press was already fired (to avoid repeating)
    long_press_fired: bool,
    /// Initial two-finger distance (for pinch)
    initial_two_finger_dist: i32,
    /// Previous two-finger positions (for scroll delta)
    prev_two_finger_mid_x: i32,
    prev_two_finger_mid_y: i32,
}

impl GestureRecognizer {
    /// Create a new gesture recognizer
    pub const fn new(screen_width: i32, screen_height: i32) -> Self {
        Self {
            state: RecogState::Idle,
            fingers: [FingerTracker::new(); MAX_TOUCH_POINTS],
            finger_count: 0,
            screen_width,
            screen_height,
            last_tap_x: 0,
            last_tap_y: 0,
            last_tap_time_us: 0,
            long_press_fired: false,
            initial_two_finger_dist: 0,
            prev_two_finger_mid_x: 0,
            prev_two_finger_mid_y: 0,
        }
    }

    /// Update screen dimensions (e.g. on resize or rotation)
    pub fn set_screen_size(&mut self, width: i32, height: i32) {
        self.screen_width = width;
        self.screen_height = height;
    }

    /// Process a single touch event and return a gesture if recognized.
    ///
    /// Call this for each event from `touch::poll_event()`.
    /// May return multiple gestures — caller should call repeatedly or use `process_all`.
    pub fn process(&mut self, event: &TouchEvent) -> Option<GestureEvent> {
        let point = &event.point;

        match point.phase {
            TouchPhase::Down => self.on_finger_down(point),
            TouchPhase::Moved => self.on_finger_move(point),
            TouchPhase::Up | TouchPhase::Cancelled => self.on_finger_up(point),
        }
    }

    /// Process all pending touch events and collect gestures
    pub fn process_all(&mut self, gestures: &mut GestureBuffer) {
        while let Some(event) = touch::poll_event() {
            if let Some(g) = self.process(&event) {
                gestures.push(g);
            }
        }

        // Check for long-press timeout
        if let Some(g) = self.check_long_press() {
            gestures.push(g);
        }
    }

    /// Check for time-based gestures (long-press). Call each frame.
    pub fn check_long_press(&mut self) -> Option<GestureEvent> {
        if self.state == RecogState::Tracking
            && self.finger_count == 1
            && !self.long_press_fired
        {
            // Copy finger data to avoid borrow conflict with self.long_press_fired
            let (fx, fy, start_t, max_disp) = match self.find_active_finger() {
                Some(f) => (f.current_x, f.current_y, f.start_time_us, f.max_displacement),
                None => return None,
            };
            let now = crate::gui::engine::now_us();
            let duration = now.saturating_sub(start_t);

            if duration >= LONG_PRESS_THRESHOLD_US
                && max_disp < LONG_PRESS_MAX_DISPLACEMENT
            {
                self.long_press_fired = true;
                self.state = RecogState::PossibleLongPress;
                return Some(GestureEvent::LongPress {
                    x: fx,
                    y: fy,
                });
            }
        }
        None
    }

    // ─────────────────────────────────────────────────────────────────────
    // Internal: finger down
    // ─────────────────────────────────────────────────────────────────────

    fn on_finger_down(&mut self, point: &TouchPoint) -> Option<GestureEvent> {
        // Find a free slot
        let slot = self.find_free_slot()?;

        self.fingers[slot] = FingerTracker {
            active: true,
            id: point.id,
            start_x: point.x,
            start_y: point.y,
            current_x: point.x,
            current_y: point.y,
            start_time_us: point.timestamp_us,
            last_time_us: point.timestamp_us,
            max_displacement: 0,
        };

        self.finger_count += 1;

        // State transitions
        match self.finger_count {
            1 => {
                self.state = RecogState::Tracking;
                self.long_press_fired = false;
            }
            2 => {
                self.state = RecogState::TwoFinger;
                // Record initial distance for pinch
                self.initial_two_finger_dist = self.two_finger_distance();
                let (mx, my) = self.two_finger_midpoint();
                self.prev_two_finger_mid_x = mx;
                self.prev_two_finger_mid_y = my;
            }
            3 => {
                self.state = RecogState::ThreeFinger;
            }
            _ => {}
        }

        Some(GestureEvent::TouchDown {
            x: point.x,
            y: point.y,
        })
    }

    // ─────────────────────────────────────────────────────────────────────
    // Internal: finger move
    // ─────────────────────────────────────────────────────────────────────

    fn on_finger_move(&mut self, point: &TouchPoint) -> Option<GestureEvent> {
        // Update the finger tracker
        let slot = self.find_slot_by_id(point.id)?;
        let finger = &mut self.fingers[slot];
        finger.current_x = point.x;
        finger.current_y = point.y;
        finger.last_time_us = point.timestamp_us;
        let disp = finger.displacement();
        if disp > finger.max_displacement {
            finger.max_displacement = disp;
        }

        match self.state {
            RecogState::Tracking if self.finger_count == 1 => {
                // Single finger move — emit TouchMove for cursor tracking
                Some(GestureEvent::TouchMove {
                    x: point.x,
                    y: point.y,
                })
            }
            RecogState::PossibleLongPress => {
                // Long press was fired, now finger moved → drag
                let finger = &self.fingers[slot];
                self.state = RecogState::Dragging;
                Some(GestureEvent::Drag {
                    x: point.x,
                    y: point.y,
                    start_x: finger.start_x,
                    start_y: finger.start_y,
                })
            }
            RecogState::Dragging => {
                let finger = &self.fingers[slot];
                Some(GestureEvent::Drag {
                    x: point.x,
                    y: point.y,
                    start_x: finger.start_x,
                    start_y: finger.start_y,
                })
            }
            RecogState::TwoFinger => {
                self.handle_two_finger_move()
            }
            RecogState::ThreeFinger => {
                // Just track, gesture on lift
                None
            }
            _ => None,
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Internal: finger up
    // ─────────────────────────────────────────────────────────────────────

    fn on_finger_up(&mut self, point: &TouchPoint) -> Option<GestureEvent> {
        let slot = self.find_slot_by_id(point.id)
            .or_else(|| self.find_any_active_slot());

        let slot = match slot {
            Some(s) => s,
            None => {
                self.reset();
                return Some(GestureEvent::TouchUp { x: point.x, y: point.y });
            }
        };

        // Update final position
        self.fingers[slot].current_x = point.x;
        self.fingers[slot].current_y = point.y;
        self.fingers[slot].last_time_us = point.timestamp_us;

        let finger = self.fingers[slot];
        let gesture = match self.state {
            RecogState::Tracking if self.finger_count == 1 => {
                self.classify_single_finger_lift(&finger, point.timestamp_us)
            }
            RecogState::PossibleLongPress => {
                // Long-press already fired, just clean up
                Some(GestureEvent::TouchUp { x: point.x, y: point.y })
            }
            RecogState::Dragging => {
                Some(GestureEvent::TouchUp { x: point.x, y: point.y })
            }
            RecogState::TwoFinger if self.finger_count <= 2 => {
                // End of two-finger gesture
                Some(GestureEvent::TouchUp { x: point.x, y: point.y })
            }
            RecogState::ThreeFinger if self.finger_count <= 3 => {
                self.classify_three_finger_lift()
            }
            _ => {
                Some(GestureEvent::TouchUp { x: point.x, y: point.y })
            }
        };

        // Deactivate the finger
        self.fingers[slot].active = false;
        if self.finger_count > 0 {
            self.finger_count -= 1;
        }

        // Reset state when all fingers lifted
        if self.finger_count == 0 {
            self.state = RecogState::Idle;
        }

        gesture
    }

    // ─────────────────────────────────────────────────────────────────────
    // Gesture classification
    // ─────────────────────────────────────────────────────────────────────

    fn classify_single_finger_lift(&mut self, finger: &FingerTracker, now_us: u64) -> Option<GestureEvent> {
        let duration = now_us.saturating_sub(finger.start_time_us);
        let dx = finger.current_x - finger.start_x;
        let dy = finger.current_y - finger.start_y;
        let displacement = dx.abs() + dy.abs();

        // Check for TAP first (short duration, small displacement)
        if duration < TAP_MAX_DURATION_US && displacement < TAP_MAX_DISPLACEMENT {
            // Check for double-tap
            let since_last_tap = now_us.saturating_sub(self.last_tap_time_us);
            let tap_dist = (finger.current_x - self.last_tap_x).abs()
                + (finger.current_y - self.last_tap_y).abs();

            self.last_tap_x = finger.current_x;
            self.last_tap_y = finger.current_y;
            self.last_tap_time_us = now_us;

            if since_last_tap < DOUBLE_TAP_WINDOW_US && tap_dist < DOUBLE_TAP_MAX_DISTANCE {
                // Reset so triple-tap isn't recognized as another double
                self.last_tap_time_us = 0;
                return Some(GestureEvent::DoubleTap {
                    x: finger.current_x,
                    y: finger.current_y,
                });
            }

            return Some(GestureEvent::Tap {
                x: finger.current_x,
                y: finger.current_y,
            });
        }

        // Check for SWIPE (sufficient distance and velocity)
        if displacement >= SWIPE_MIN_DISTANCE {
            let duration_secs_x100 = (duration / 10_000).max(1) as i32; // centiseconds
            let velocity = (displacement * 100) / duration_secs_x100; // px/sec

            if velocity >= SWIPE_MIN_VELOCITY {
                // Check for EDGE swipe first
                if let Some(edge_gesture) = self.check_edge_swipe(finger, dx, dy) {
                    return Some(edge_gesture);
                }

                // Regular swipe — determine direction by dominant axis
                let direction = if dx.abs() > dy.abs() {
                    if dx > 0 { SwipeDirection::Right } else { SwipeDirection::Left }
                } else {
                    if dy > 0 { SwipeDirection::Down } else { SwipeDirection::Up }
                };

                return Some(GestureEvent::Swipe {
                    direction,
                    start_x: finger.start_x,
                    start_y: finger.start_y,
                    end_x: finger.current_x,
                    end_y: finger.current_y,
                    velocity,
                });
            }
        }

        // Default: just a touch up (was a slow drag or something intermediate)
        Some(GestureEvent::TouchUp {
            x: finger.current_x,
            y: finger.current_y,
        })
    }

    fn check_edge_swipe(&self, finger: &FingerTracker, dx: i32, dy: i32) -> Option<GestureEvent> {
        // Bottom edge → swipe up (open launcher)
        if finger.start_y >= self.screen_height - EDGE_ZONE_PX && dy < -SWIPE_MIN_DISTANCE {
            return Some(GestureEvent::EdgeSwipe {
                origin: EdgeOrigin::Bottom,
                progress: dy.abs(),
            });
        }

        // Top edge → swipe down (notification panel)
        if finger.start_y <= EDGE_ZONE_PX && dy > SWIPE_MIN_DISTANCE {
            return Some(GestureEvent::EdgeSwipe {
                origin: EdgeOrigin::Top,
                progress: dy.abs(),
            });
        }

        // Left edge → swipe right
        if finger.start_x <= EDGE_ZONE_PX && dx > SWIPE_MIN_DISTANCE {
            return Some(GestureEvent::EdgeSwipe {
                origin: EdgeOrigin::Left,
                progress: dx.abs(),
            });
        }

        // Right edge → swipe left
        if finger.start_x >= self.screen_width - EDGE_ZONE_PX && dx < -SWIPE_MIN_DISTANCE {
            return Some(GestureEvent::EdgeSwipe {
                origin: EdgeOrigin::Right,
                progress: dx.abs(),
            });
        }

        None
    }

    fn handle_two_finger_move(&mut self) -> Option<GestureEvent> {
        let (f0, f1) = self.get_two_active_fingers()?;

        let cur_dist = self.two_finger_distance();
        let dist_delta = cur_dist - self.initial_two_finger_dist;

        // Check PINCH (distance changed significantly)
        if dist_delta.abs() >= PINCH_MIN_DELTA {
            let (cx, cy) = self.two_finger_midpoint();
            // Scale: 100 = no change, proportional to distance ratio
            let scale = if self.initial_two_finger_dist > 0 {
                (cur_dist * 100) / self.initial_two_finger_dist.max(1)
            } else {
                100
            };
            return Some(GestureEvent::Pinch {
                center_x: cx,
                center_y: cy,
                scale,
            });
        }

        // Check TWO-FINGER SCROLL (both fingers moving in same direction)
        let (mx, my) = self.two_finger_midpoint();
        let scroll_dx = mx - self.prev_two_finger_mid_x;
        let scroll_dy = my - self.prev_two_finger_mid_y;

        if scroll_dx.abs() >= SCROLL_MIN_DELTA || scroll_dy.abs() >= SCROLL_MIN_DELTA {
            self.prev_two_finger_mid_x = mx;
            self.prev_two_finger_mid_y = my;
            return Some(GestureEvent::Scroll {
                delta_x: scroll_dx,
                delta_y: scroll_dy,
            });
        }

        None
    }

    fn classify_three_finger_lift(&self) -> Option<GestureEvent> {
        // Compute average horizontal displacement of all 3 fingers
        let mut total_dx = 0i32;
        let mut count = 0;
        for finger in &self.fingers {
            if finger.active {
                total_dx += finger.current_x - finger.start_x;
                count += 1;
            }
        }
        if count == 0 { return None; }

        let avg_dx = total_dx / count;

        if avg_dx.abs() >= SWIPE_MIN_DISTANCE {
            let direction = if avg_dx > 0 {
                SwipeDirection::Right
            } else {
                SwipeDirection::Left
            };
            Some(GestureEvent::ThreeFingerSwipe { direction })
        } else {
            Some(GestureEvent::TouchUp { x: 0, y: 0 })
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Helper methods
    // ─────────────────────────────────────────────────────────────────────

    fn find_free_slot(&self) -> Option<usize> {
        for (i, f) in self.fingers.iter().enumerate() {
            if !f.active {
                return Some(i);
            }
        }
        None
    }

    fn find_slot_by_id(&self, id: u16) -> Option<usize> {
        for (i, f) in self.fingers.iter().enumerate() {
            if f.active && f.id == id {
                return Some(i);
            }
        }
        None
    }

    fn find_any_active_slot(&self) -> Option<usize> {
        for (i, f) in self.fingers.iter().enumerate() {
            if f.active {
                return Some(i);
            }
        }
        None
    }

    fn find_active_finger(&self) -> Option<&FingerTracker> {
        self.fingers.iter().find(|f| f.active)
    }

    fn get_two_active_fingers(&self) -> Option<(usize, usize)> {
        let mut found = [0usize; 2];
        let mut count = 0;
        for (i, f) in self.fingers.iter().enumerate() {
            if f.active && count < 2 {
                found[count] = i;
                count += 1;
            }
        }
        if count == 2 {
            Some((found[0], found[1]))
        } else {
            None
        }
    }

    fn two_finger_distance(&self) -> i32 {
        if let Some((a, b)) = self.get_two_active_fingers() {
            let dx = self.fingers[a].current_x - self.fingers[b].current_x;
            let dy = self.fingers[a].current_y - self.fingers[b].current_y;
            // Integer square root approximation (good enough for gesture detection)
            isqrt((dx * dx + dy * dy) as u32) as i32
        } else {
            0
        }
    }

    fn two_finger_midpoint(&self) -> (i32, i32) {
        if let Some((a, b)) = self.get_two_active_fingers() {
            let mx = (self.fingers[a].current_x + self.fingers[b].current_x) / 2;
            let my = (self.fingers[a].current_y + self.fingers[b].current_y) / 2;
            (mx, my)
        } else {
            (0, 0)
        }
    }

    fn reset(&mut self) {
        self.state = RecogState::Idle;
        self.finger_count = 0;
        self.long_press_fired = false;
        for f in &mut self.fingers {
            f.active = false;
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// Gesture buffer (small fixed-size collection for per-frame gesture output)
// ═════════════════════════════════════════════════════════════════════════════

/// Fixed-size buffer for gestures produced in one frame
pub struct GestureBuffer {
    gestures: [Option<GestureEvent>; 8],
    count: usize,
}

impl GestureBuffer {
    pub const fn new() -> Self {
        Self {
            gestures: [None; 8],
            count: 0,
        }
    }

    pub fn push(&mut self, gesture: GestureEvent) {
        if self.count < 8 {
            self.gestures[self.count] = Some(gesture);
            self.count += 1;
        }
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn iter(&self) -> GestureIter<'_> {
        GestureIter {
            buf: self,
            idx: 0,
        }
    }

    pub fn clear(&mut self) {
        self.count = 0;
        self.gestures = [None; 8];
    }
}

pub struct GestureIter<'a> {
    buf: &'a GestureBuffer,
    idx: usize,
}

impl<'a> Iterator for GestureIter<'a> {
    type Item = &'a GestureEvent;
    fn next(&mut self) -> Option<Self::Item> {
        while self.idx < self.buf.count {
            let i = self.idx;
            self.idx += 1;
            if let Some(ref g) = self.buf.gestures[i] {
                return Some(g);
            }
        }
        None
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// Utility: integer square root
// ═════════════════════════════════════════════════════════════════════════════

/// Integer square root (Newton's method)
fn isqrt(n: u32) -> u32 {
    if n == 0 { return 0; }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}
