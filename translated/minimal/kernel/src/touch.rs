











use core::sync::atomic::{AtomicBool, AtomicI32, AtomicU8, AtomicU16, AtomicU32, AtomicU64, Ordering};






pub const HL_: usize = 10;


const DB_: usize = 64;
const ATL_: usize = DB_ - 1;






#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TouchPhase {
    
    Down = 0,
    
    Moved = 1,
    
    Up = 2,
    
    Cancelled = 3,
}

impl TouchPhase {
    fn atw(v: u8) -> Self {
        match v {
            0 => Self::Down,
            1 => Self::Moved,
            2 => Self::Up,
            3 => Self::Cancelled,
            _ => Self::Cancelled,
        }
    }
}


#[derive(Debug, Clone, Copy)]
pub struct TouchPoint {
    
    pub id: u16,
    
    pub x: i32,
    
    pub y: i32,
    
    pub pressure: u8,
    
    pub phase: TouchPhase,
    
    pub timestamp_us: u64,
}

impl Default for TouchPoint {
    fn default() -> Self {
        Self {
            id: 0,
            x: 0,
            y: 0,
            pressure: 0,
            phase: TouchPhase::Up,
            timestamp_us: 0,
        }
    }
}


#[derive(Debug, Clone)]
pub struct TouchState {
    
    pub points: [TouchPoint; HL_],
    
    pub count: u8,
    
    pub timestamp_us: u64,
}

impl Default for TouchState {
    fn default() -> Self {
        Self {
            points: [TouchPoint::default(); HL_],
            count: 0,
            timestamp_us: 0,
        }
    }
}


#[derive(Debug, Clone, Copy)]
pub struct Kv {
    pub point: TouchPoint,
}






static Ah: AtomicBool = AtomicBool::new(false);


static ASI_: AtomicBool = AtomicBool::new(false);


static JP_: AtomicU32 = AtomicU32::new(1280);
static JO_: AtomicU32 = AtomicU32::new(800);


static ASG_: AtomicU32 = AtomicU32::new(4096);
static ASH_: AtomicU32 = AtomicU32::new(4096);


static ZY_: AtomicU8 = AtomicU8::new(0);



struct AtomicTouchSlot {
    active: AtomicBool,
    id: AtomicU16,
    x: AtomicI32,
    y: AtomicI32,
    pressure: AtomicU8,
    phase: AtomicU8,
    timestamp_us: AtomicU64,
}

impl AtomicTouchSlot {
    const fn new() -> Self {
        Self {
            active: AtomicBool::new(false),
            id: AtomicU16::new(0),
            x: AtomicI32::new(0),
            y: AtomicI32::new(0),
            pressure: AtomicU8::new(0),
            phase: AtomicU8::new(TouchPhase::Up as u8),
            timestamp_us: AtomicU64::new(0),
        }
    }

    fn load(&self) -> TouchPoint {
        TouchPoint {
            id: self.id.load(Ordering::Relaxed),
            x: self.x.load(Ordering::Relaxed),
            y: self.y.load(Ordering::Relaxed),
            pressure: self.pressure.load(Ordering::Relaxed),
            phase: TouchPhase::atw(self.phase.load(Ordering::Relaxed)),
            timestamp_us: self.timestamp_us.load(Ordering::Relaxed),
        }
    }

    fn store(&self, point: &TouchPoint) {
        self.id.store(point.id, Ordering::Relaxed);
        self.x.store(point.x, Ordering::Relaxed);
        self.y.store(point.y, Ordering::Relaxed);
        self.pressure.store(point.pressure, Ordering::Relaxed);
        self.phase.store(point.phase as u8, Ordering::Relaxed);
        self.timestamp_us.store(point.timestamp_us, Ordering::Relaxed);
        self.active.store(point.phase != TouchPhase::Up && point.phase != TouchPhase::Cancelled, Ordering::Relaxed);
    }

    fn clear(&self) {
        self.active.store(false, Ordering::Relaxed);
        self.phase.store(TouchPhase::Up as u8, Ordering::Relaxed);
    }
}


static HY_: [AtomicTouchSlot; HL_] = [
    AtomicTouchSlot::new(), AtomicTouchSlot::new(),
    AtomicTouchSlot::new(), AtomicTouchSlot::new(),
    AtomicTouchSlot::new(), AtomicTouchSlot::new(),
    AtomicTouchSlot::new(), AtomicTouchSlot::new(),
    AtomicTouchSlot::new(), AtomicTouchSlot::new(),
];






struct Rx {
    
    
    ids: [AtomicU16; DB_],
    xs: [AtomicI32; DB_],
    ys: [AtomicI32; DB_],
    pressures: [AtomicU8; DB_],
    phases: [AtomicU8; DB_],
    timestamps: [AtomicU64; DB_],
    
    write_idx: AtomicU32,
    
    read_idx: AtomicU32,
}


macro_rules! ctr {
    ($type:ty, $val:expr, $ae:expr) => {{
        
        
        const Bm: $type = $val;
        [Bm; $ae]
    }};
}

static HB_: Rx = Rx {
    ids: ctr!(AtomicU16, AtomicU16::new(0), DB_),
    xs: ctr!(AtomicI32, AtomicI32::new(0), DB_),
    ys: ctr!(AtomicI32, AtomicI32::new(0), DB_),
    pressures: ctr!(AtomicU8, AtomicU8::new(0), DB_),
    phases: ctr!(AtomicU8, AtomicU8::new(0), DB_),
    timestamps: ctr!(AtomicU64, AtomicU64::new(0), DB_),
    write_idx: AtomicU32::new(0),
    read_idx: AtomicU32::new(0),
};

impl Rx {
    
    fn push(&self, point: &TouchPoint) {
        let w = self.write_idx.load(Ordering::Relaxed);
        let idx = (w as usize) & ATL_;

        self.ids[idx].store(point.id, Ordering::Relaxed);
        self.xs[idx].store(point.x, Ordering::Relaxed);
        self.ys[idx].store(point.y, Ordering::Relaxed);
        self.pressures[idx].store(point.pressure, Ordering::Relaxed);
        self.phases[idx].store(point.phase as u8, Ordering::Relaxed);
        self.timestamps[idx].store(point.timestamp_us, Ordering::Relaxed);

        
        self.write_idx.store(w.wrapping_add(1), Ordering::Release);
    }

    
    fn pop(&self) -> Option<Kv> {
        let r = self.read_idx.load(Ordering::Relaxed);
        let w = self.write_idx.load(Ordering::Acquire);

        if r == w {
            return None; 
        }

        let idx = (r as usize) & ATL_;
        let point = TouchPoint {
            id: self.ids[idx].load(Ordering::Relaxed),
            x: self.xs[idx].load(Ordering::Relaxed),
            y: self.ys[idx].load(Ordering::Relaxed),
            pressure: self.pressures[idx].load(Ordering::Relaxed),
            phase: TouchPhase::atw(self.phases[idx].load(Ordering::Relaxed)),
            timestamp_us: self.timestamps[idx].load(Ordering::Relaxed),
        };

        self.read_idx.store(r.wrapping_add(1), Ordering::Release);
        Some(Kv { point })
    }
}






pub fn init() {
    
    for slot in &HY_ {
        slot.clear();
    }
    ZY_.store(0, Ordering::Relaxed);
    Ah.store(true, Ordering::Relaxed);
    crate::serial_println!("[TOUCH] Touch subsystem initialized (max {} points)", HL_);
}


pub fn set_screen_size(width: u32, height: u32) {
    JP_.store(width, Ordering::Relaxed);
    JO_.store(height, Ordering::Relaxed);
}


pub fn qvs(aly: u32, aye: u32) {
    ASG_.store(aly, Ordering::Relaxed);
    ASH_.store(aye, Ordering::Relaxed);
}


pub fn qvr(present: bool) {
    ASI_.store(present, Ordering::Relaxed);
    if present {
        crate::serial_println!("[TOUCH] Touchscreen device detected");
    }
}


pub fn sw() -> bool {
    Ah.load(Ordering::Relaxed) && ASI_.load(Ordering::Relaxed)
}


pub fn is_initialized() -> bool {
    Ah.load(Ordering::Relaxed)
}


pub fn active_count() -> u8 {
    ZY_.load(Ordering::Relaxed)
}


pub fn get_state() -> TouchState {
    let mut state = TouchState::default();
    let mut count = 0u8;

    for slot in &HY_ {
        if slot.active.load(Ordering::Relaxed) && (count as usize) < HL_ {
            state.points[count as usize] = slot.load();
            count += 1;
        }
    }

    state.count = count;
    state.timestamp_us = crate::gui::engine::yy();
    state
}


pub fn bir() -> Option<Kv> {
    HB_.pop()
}


pub fn lhk<F: FnMut(Kv)>(mut f: F) {
    while let Some(bsj) = HB_.pop() {
        f(bsj);
    }
}









pub fn gcu(id: u16, gpv: u32, cox: u32, pressure: u8, phase: TouchPhase) {
    let screen_w = JP_.load(Ordering::Relaxed);
    let screen_h = JO_.load(Ordering::Relaxed);
    let lec = ASG_.load(Ordering::Relaxed).max(1);
    let led = ASH_.load(Ordering::Relaxed).max(1);

    
    let x = ((gpv as u64 * screen_w as u64) / lec as u64) as i32;
    let y = ((cox as u64 * screen_h as u64) / led as u64) as i32;

    mqb(id, x, y, pressure, phase);
}





pub fn mqb(id: u16, x: i32, y: i32, pressure: u8, phase: TouchPhase) {
    let screen_w = JP_.load(Ordering::Relaxed) as i32;
    let screen_h = JO_.load(Ordering::Relaxed) as i32;

    let point = TouchPoint {
        id,
        x: x.clamp(0, screen_w - 1),
        y: y.clamp(0, screen_h - 1),
        pressure,
        phase,
        timestamp_us: crate::gui::engine::yy(),
    };

    
    let ott = lwa(id, phase);
    if let Some(idx) = ott {
        HY_[idx].store(&point);
    }

    
    let mut count = 0u8;
    for slot in &HY_ {
        if slot.active.load(Ordering::Relaxed) {
            count += 1;
        }
    }
    ZY_.store(count, Ordering::Relaxed);

    
    HB_.push(&point);
}



fn lwa(id: u16, phase: TouchPhase) -> Option<usize> {
    
    for (i, slot) in HY_.iter().enumerate() {
        if slot.active.load(Ordering::Relaxed) && slot.id.load(Ordering::Relaxed) == id {
            return Some(i);
        }
    }

    
    if phase == TouchPhase::Down {
        for (i, slot) in HY_.iter().enumerate() {
            if !slot.active.load(Ordering::Relaxed) {
                return Some(i);
            }
        }
    }

    
    if phase == TouchPhase::Up || phase == TouchPhase::Cancelled {
        
        for (i, slot) in HY_.iter().enumerate() {
            if slot.id.load(Ordering::Relaxed) == id {
                return Some(i);
            }
        }
    }

    None
}








pub fn qer() -> Option<(i32, i32, bool)> {
    
    for slot in &HY_ {
        if slot.active.load(Ordering::Relaxed) {
            let x = slot.x.load(Ordering::Relaxed);
            let y = slot.y.load(Ordering::Relaxed);
            let phase = TouchPhase::atw(slot.phase.load(Ordering::Relaxed));
            let pressed = phase == TouchPhase::Down || phase == TouchPhase::Moved;
            return Some((x, y, pressed));
        }
    }
    None
}
