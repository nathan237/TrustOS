



use spin::Mutex;


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InputDeviceType {
    Ciw,
    Cix,
    Dki,
    Dkj,
    Dkh,
}


#[derive(Clone, Debug)]
pub struct Czv {
    pub ceb: InputDeviceType,
    pub j: &'static str,
    pub bfz: bool,
}


static KM_: Mutex<InputState> = Mutex::new(InputState::new());

struct InputState {
    lhf: bool,
    lmq: bool,
    hpl: Option<InputDeviceType>,
    hry: Option<InputDeviceType>,
}

impl InputState {
    const fn new() -> Self {
        InputState {
            lhf: false,
            lmq: false,
            hpl: None,
            hry: None,
        }
    }
}


pub fn init() {
    let mut g = KM_.lock();
    
    
    
    g.lhf = true;
    g.hpl = Some(InputDeviceType::Ciw);
    
    
    if crate::mouse::ky() {
        g.lmq = true;
        g.hry = Some(InputDeviceType::Cix);
    }
    
    crate::serial_println!("[INPUT] Keyboard: {:?}, Mouse: {:?}",
        g.hpl, g.hry);
}


pub fn oar() -> bool {
    KM_.lock().lhf
}


pub fn tms() -> bool {
    KM_.lock().lmq
}


pub fn hpl() -> Option<InputDeviceType> {
    KM_.lock().hpl
}


pub fn hry() -> Option<InputDeviceType> {
    KM_.lock().hry
}


pub fn zqk(yai: bool, ybj: bool) {
    let mut g = KM_.lock();
    
    
    
    
    
    
    
    
    
    
    
    
    let _ = g;  
}
