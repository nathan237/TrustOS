












use crate::touch::{self, Kv, TouchPhase, TouchPoint, HL_};






const DBB_: u64 = 200_000;


const DBA_: i32 = 10;


const CIE_: u64 = 500_000;


const CID_: i32 = 15;


const LX_: i32 = 50;


const CYL_: i32 = 200;


const UE_: i32 = 30;


const CMZ_: i32 = 15;


const BHF_: i32 = 8;


const BUZ_: u64 = 300_000;


const BUY_: i32 = 30;






#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwipeDirection {
    Left,
    Right,
    Up,
    Down,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeOrigin {
    
    Bottom,
    
    Top,
    
    Left,
    
    Right,
}


#[derive(Debug, Clone, Copy)]
pub enum GestureEvent {
    
    Tap { x: i32, y: i32 },

    
    DoubleTap { x: i32, y: i32 },

    
    LongPress { x: i32, y: i32 },

    
    Swipe {
        direction: SwipeDirection,
        start_x: i32,
        start_y: i32,
        awy: i32,
        doq: i32,
        velocity: i32, 
    },

    
    EdgeSwipe {
        origin: EdgeOrigin,
        progress: i32, 
    },

    
    Pinch {
        center_x: i32,
        center_y: i32,
        
        scale: i32,
    },

    
    Scroll {
        delta_x: i32,
        delta_y: i32,
    },

    
    ThreeFingerSwipe {
        direction: SwipeDirection,
    },

    
    TouchDown { x: i32, y: i32 },

    
    TouchMove { x: i32, y: i32 },

    
    TouchUp { x: i32, y: i32 },

    
    Drag {
        x: i32,
        y: i32,
        start_x: i32,
        start_y: i32,
    },
}






#[derive(Clone, Copy)]
struct FingerTracker {
    active: bool,
    id: u16,
    
    start_x: i32,
    start_y: i32,
    
    current_x: i32,
    current_y: i32,
    
    start_time_us: u64,
    
    last_time_us: u64,
    
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
        let ad = self.current_y - self.start_y;
        
        dx.abs() + ad.abs()
    }

    fn qfg(&self) -> i32 {
        let dx = self.current_x - self.start_x;
        let ad = self.current_y - self.start_y;
        dx * dx + ad * ad
    }

    fn qeh(&self) -> u64 {
        self.last_time_us.saturating_sub(self.start_time_us)
    }
}






#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RecogState {
    
    Idle,
    
    Tracking,
    
    PossibleLongPress,
    
    Dragging,
    
    TwoFinger,
    
    ThreeFinger,
}


pub struct GestureRecognizer {
    state: RecogState,
    
    fingers: [FingerTracker; HL_],
    
    finger_count: u8,
    
    screen_width: i32,
    screen_height: i32,
    
    last_tap_x: i32,
    last_tap_y: i32,
    last_tap_time_us: u64,
    
    long_press_fired: bool,
    
    initial_two_finger_dist: i32,
    
    prev_two_finger_mid_x: i32,
    prev_two_finger_mid_y: i32,
}

impl GestureRecognizer {
    
    pub const fn new(screen_width: i32, screen_height: i32) -> Self {
        Self {
            state: RecogState::Idle,
            fingers: [FingerTracker::new(); HL_],
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

    
    pub fn set_screen_size(&mut self, width: i32, height: i32) {
        self.screen_width = width;
        self.screen_height = height;
    }

    
    
    
    
    pub fn process(&mut self, event: &Kv) -> Option<GestureEvent> {
        let point = &event.point;

        match point.phase {
            TouchPhase::Down => self.on_finger_down(point),
            TouchPhase::Moved => self.on_finger_move(point),
            TouchPhase::Up | TouchPhase::Cancelled => self.on_finger_up(point),
        }
    }

    
    pub fn process_all(&mut self, gestures: &mut GestureBuffer) {
        while let Some(event) = touch::bir() {
            if let Some(g) = self.process(&event) {
                gestures.push(g);
            }
        }

        
        if let Some(g) = self.check_long_press() {
            gestures.push(g);
        }
    }

    
    pub fn check_long_press(&mut self) -> Option<GestureEvent> {
        if self.state == RecogState::Tracking
            && self.finger_count == 1
            && !self.long_press_fired
        {
            
            let (dg, hj, start_t, max_disp) = match self.find_active_finger() {
                Some(f) => (f.current_x, f.current_y, f.start_time_us, f.max_displacement),
                None => return None,
            };
            let cy = crate::gui::engine::yy();
            let yq = cy.saturating_sub(start_t);

            if yq >= CIE_
                && max_disp < CID_
            {
                self.long_press_fired = true;
                self.state = RecogState::PossibleLongPress;
                return Some(GestureEvent::LongPress {
                    x: dg,
                    y: hj,
                });
            }
        }
        None
    }

    
    
    

    fn on_finger_down(&mut self, point: &TouchPoint) -> Option<GestureEvent> {
        
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

        
        match self.finger_count {
            1 => {
                self.state = RecogState::Tracking;
                self.long_press_fired = false;
            }
            2 => {
                self.state = RecogState::TwoFinger;
                
                self.initial_two_finger_dist = self.two_finger_distance();
                let (cg, cr) = self.two_finger_midpoint();
                self.prev_two_finger_mid_x = cg;
                self.prev_two_finger_mid_y = cr;
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

    
    
    

    fn on_finger_move(&mut self, point: &TouchPoint) -> Option<GestureEvent> {
        
        let slot = self.find_slot_by_id(point.id)?;
        let oj = &mut self.fingers[slot];
        oj.current_x = point.x;
        oj.current_y = point.y;
        oj.last_time_us = point.timestamp_us;
        let uv = oj.displacement();
        if uv > oj.max_displacement {
            oj.max_displacement = uv;
        }

        match self.state {
            RecogState::Tracking if self.finger_count == 1 => {
                
                Some(GestureEvent::TouchMove {
                    x: point.x,
                    y: point.y,
                })
            }
            RecogState::PossibleLongPress => {
                
                let oj = &self.fingers[slot];
                self.state = RecogState::Dragging;
                Some(GestureEvent::Drag {
                    x: point.x,
                    y: point.y,
                    start_x: oj.start_x,
                    start_y: oj.start_y,
                })
            }
            RecogState::Dragging => {
                let oj = &self.fingers[slot];
                Some(GestureEvent::Drag {
                    x: point.x,
                    y: point.y,
                    start_x: oj.start_x,
                    start_y: oj.start_y,
                })
            }
            RecogState::TwoFinger => {
                self.handle_two_finger_move()
            }
            RecogState::ThreeFinger => {
                
                None
            }
            _ => None,
        }
    }

    
    
    

    fn on_finger_up(&mut self, point: &TouchPoint) -> Option<GestureEvent> {
        let slot = self.find_slot_by_id(point.id)
            .or_else(|| self.find_any_active_slot());

        let slot = match slot {
            Some(j) => j,
            None => {
                self.reset();
                return Some(GestureEvent::TouchUp { x: point.x, y: point.y });
            }
        };

        
        self.fingers[slot].current_x = point.x;
        self.fingers[slot].current_y = point.y;
        self.fingers[slot].last_time_us = point.timestamp_us;

        let oj = self.fingers[slot];
        let gesture = match self.state {
            RecogState::Tracking if self.finger_count == 1 => {
                self.classify_single_finger_lift(&oj, point.timestamp_us)
            }
            RecogState::PossibleLongPress => {
                
                Some(GestureEvent::TouchUp { x: point.x, y: point.y })
            }
            RecogState::Dragging => {
                Some(GestureEvent::TouchUp { x: point.x, y: point.y })
            }
            RecogState::TwoFinger if self.finger_count <= 2 => {
                
                Some(GestureEvent::TouchUp { x: point.x, y: point.y })
            }
            RecogState::ThreeFinger if self.finger_count <= 3 => {
                self.classify_three_finger_lift()
            }
            _ => {
                Some(GestureEvent::TouchUp { x: point.x, y: point.y })
            }
        };

        
        self.fingers[slot].active = false;
        if self.finger_count > 0 {
            self.finger_count -= 1;
        }

        
        if self.finger_count == 0 {
            self.state = RecogState::Idle;
        }

        gesture
    }

    
    
    

    fn classify_single_finger_lift(&mut self, oj: &FingerTracker, yy: u64) -> Option<GestureEvent> {
        let yq = yy.saturating_sub(oj.start_time_us);
        let dx = oj.current_x - oj.start_x;
        let ad = oj.current_y - oj.start_y;
        let displacement = dx.abs() + ad.abs();

        
        if yq < DBB_ && displacement < DBA_ {
            
            let otf = yy.saturating_sub(self.last_tap_time_us);
            let pdc = (oj.current_x - self.last_tap_x).abs()
                + (oj.current_y - self.last_tap_y).abs();

            self.last_tap_x = oj.current_x;
            self.last_tap_y = oj.current_y;
            self.last_tap_time_us = yy;

            if otf < BUZ_ && pdc < BUY_ {
                
                self.last_tap_time_us = 0;
                return Some(GestureEvent::DoubleTap {
                    x: oj.current_x,
                    y: oj.current_y,
                });
            }

            return Some(GestureEvent::Tap {
                x: oj.current_x,
                y: oj.current_y,
            });
        }

        
        if displacement >= LX_ {
            let lms = (yq / 10_000).max(1) as i32; 
            let velocity = (displacement * 100) / lms; 

            if velocity >= CYL_ {
                
                if let Some(edge_gesture) = self.check_edge_swipe(oj, dx, ad) {
                    return Some(edge_gesture);
                }

                
                let direction = if dx.abs() > ad.abs() {
                    if dx > 0 { SwipeDirection::Right } else { SwipeDirection::Left }
                } else {
                    if ad > 0 { SwipeDirection::Down } else { SwipeDirection::Up }
                };

                return Some(GestureEvent::Swipe {
                    direction,
                    start_x: oj.start_x,
                    start_y: oj.start_y,
                    awy: oj.current_x,
                    doq: oj.current_y,
                    velocity,
                });
            }
        }

        
        Some(GestureEvent::TouchUp {
            x: oj.current_x,
            y: oj.current_y,
        })
    }

    fn check_edge_swipe(&self, oj: &FingerTracker, dx: i32, ad: i32) -> Option<GestureEvent> {
        
        if oj.start_y >= self.screen_height - UE_ && ad < -LX_ {
            return Some(GestureEvent::EdgeSwipe {
                origin: EdgeOrigin::Bottom,
                progress: ad.abs(),
            });
        }

        
        if oj.start_y <= UE_ && ad > LX_ {
            return Some(GestureEvent::EdgeSwipe {
                origin: EdgeOrigin::Top,
                progress: ad.abs(),
            });
        }

        
        if oj.start_x <= UE_ && dx > LX_ {
            return Some(GestureEvent::EdgeSwipe {
                origin: EdgeOrigin::Left,
                progress: dx.abs(),
            });
        }

        
        if oj.start_x >= self.screen_width - UE_ && dx < -LX_ {
            return Some(GestureEvent::EdgeSwipe {
                origin: EdgeOrigin::Right,
                progress: dx.abs(),
            });
        }

        None
    }

    fn handle_two_finger_move(&mut self) -> Option<GestureEvent> {
        let (f0, f1) = self.get_two_active_fingers()?;

        let hpk = self.two_finger_distance();
        let lfx = hpk - self.initial_two_finger_dist;

        
        if lfx.abs() >= CMZ_ {
            let (cx, u) = self.two_finger_midpoint();
            
            let scale = if self.initial_two_finger_dist > 0 {
                (hpk * 100) / self.initial_two_finger_dist.max(1)
            } else {
                100
            };
            return Some(GestureEvent::Pinch {
                center_x: cx,
                center_y: u,
                scale,
            });
        }

        
        let (cg, cr) = self.two_finger_midpoint();
        let jdn = cg - self.prev_two_finger_mid_x;
        let jdo = cr - self.prev_two_finger_mid_y;

        if jdn.abs() >= BHF_ || jdo.abs() >= BHF_ {
            self.prev_two_finger_mid_x = cg;
            self.prev_two_finger_mid_y = cr;
            return Some(GestureEvent::Scroll {
                delta_x: jdn,
                delta_y: jdo,
            });
        }

        None
    }

    fn classify_three_finger_lift(&self) -> Option<GestureEvent> {
        
        let mut fde = 0i32;
        let mut count = 0;
        for oj in &self.fingers {
            if oj.active {
                fde += oj.current_x - oj.start_x;
                count += 1;
            }
        }
        if count == 0 { return None; }

        let hgf = fde / count;

        if hgf.abs() >= LX_ {
            let direction = if hgf > 0 {
                SwipeDirection::Right
            } else {
                SwipeDirection::Left
            };
            Some(GestureEvent::ThreeFingerSwipe { direction })
        } else {
            Some(GestureEvent::TouchUp { x: 0, y: 0 })
        }
    }

    
    
    

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
        let mut nj = [0usize; 2];
        let mut count = 0;
        for (i, f) in self.fingers.iter().enumerate() {
            if f.active && count < 2 {
                nj[count] = i;
                count += 1;
            }
        }
        if count == 2 {
            Some((nj[0], nj[1]))
        } else {
            None
        }
    }

    fn two_finger_distance(&self) -> i32 {
        if let Some((a, b)) = self.get_two_active_fingers() {
            let dx = self.fingers[a].current_x - self.fingers[b].current_x;
            let ad = self.fingers[a].current_y - self.fingers[b].current_y;
            
            dsw((dx * dx + ad * ad) as u32) as i32
        } else {
            0
        }
    }

    fn two_finger_midpoint(&self) -> (i32, i32) {
        if let Some((a, b)) = self.get_two_active_fingers() {
            let cg = (self.fingers[a].current_x + self.fingers[b].current_x) / 2;
            let cr = (self.fingers[a].current_y + self.fingers[b].current_y) / 2;
            (cg, cr)
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

    pub fn iter(&self) -> Sq<'_> {
        Sq {
            buf: self,
            idx: 0,
        }
    }

    pub fn clear(&mut self) {
        self.count = 0;
        self.gestures = [None; 8];
    }
}

pub struct Sq<'a> {
    buf: &'a GestureBuffer,
    idx: usize,
}

impl<'a> Iterator for Sq<'a> {
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






fn dsw(ae: u32) -> u32 {
    if ae == 0 { return 0; }
    let mut x = ae;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + ae / x) / 2;
    }
    x
}
