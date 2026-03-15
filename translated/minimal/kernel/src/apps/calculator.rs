



use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;

use crate::ui::{
    Cf, WidgetState, Color, Theme, Rect, Point, Size,
    UiEvent, MouseEvent, MouseButton,
    Vs, Dy, bvo, cb,
};
use crate::drivers::virtio_gpu::GpuSurface;


#[derive(Clone, Copy, PartialEq)]
pub enum CalcButton {
    Bu(u8),      
    Add,            
    Ur,       
    To,       
    Sc,         
    Wn,         
    Aan,          
    Aao,     
    Zz,      
    Aay,        
    Akm,         
    Qk,        
    Avm,      
    Avq,      
    Avp,   
    Avn,    
}

impl CalcButton {
    fn cu(&self) -> &'static str {
        match self {
            CalcButton::Bu(0) => "0",
            CalcButton::Bu(1) => "1",
            CalcButton::Bu(2) => "2",
            CalcButton::Bu(3) => "3",
            CalcButton::Bu(4) => "4",
            CalcButton::Bu(5) => "5",
            CalcButton::Bu(6) => "6",
            CalcButton::Bu(7) => "7",
            CalcButton::Bu(8) => "8",
            CalcButton::Bu(9) => "9",
            CalcButton::Add => "+",
            CalcButton::Ur => "-",
            CalcButton::To => "x",
            CalcButton::Sc => "/",
            CalcButton::Wn => "=",
            CalcButton::Aan => "C",
            CalcButton::Aao => "CE",
            CalcButton::Zz => "<",
            CalcButton::Aay => ".",
            CalcButton::Akm => "+/-",
            CalcButton::Qk => "%",
            CalcButton::Avm => "M+",
            CalcButton::Avq => "M-",
            CalcButton::Avp => "MR",
            CalcButton::Avn => "MC",
            _ => "?",
        }
    }
}


pub struct Calculator {
    ad: u32,
    eg: Rect,
    g: WidgetState,
    
    
    display: String,
    xz: String,
    
    
    cv: f64,
    fre: f64,
    htr: Option<CalcButton>,
    daf: bool,
    
    
    memory: f64,
    
    
    gbv: Vec<(CalcButton, Rect)>,
    hnc: Option<usize>,
    gpr: Option<usize>,
    
    
    xpk: bool,
}

impl Calculator {
    pub fn new() -> Self {
        Self {
            ad: bvo(),
            eg: Rect::Dh,
            g: WidgetState::new(),
            display: String::from("0"),
            xz: String::new(),
            cv: 0.0,
            fre: 0.0,
            htr: None,
            daf: true,
            memory: 0.0,
            gbv: Vec::new(),
            hnc: None,
            gpr: None,
            xpk: true,
        }
    }
    
    
    fn udl(&mut self) {
        self.gbv.clear();
        
        let b = self.eg.b as u32 + 10;
        let c = self.eg.c as u32 + 90; 
        let bym = 60;
        let doq = 50;
        let qi = 8;
        
        
        let layout: [[CalcButton; 5]; 5] = [
            [CalcButton::Avn, CalcButton::Avp, CalcButton::Avm, CalcButton::Avq, CalcButton::Zz],
            [CalcButton::Aan, CalcButton::Aao, CalcButton::Qk, CalcButton::Sc, CalcButton::Bu(7)],
            [CalcButton::Bu(4), CalcButton::Bu(5), CalcButton::Bu(6), CalcButton::To, CalcButton::Bu(8)],
            [CalcButton::Bu(1), CalcButton::Bu(2), CalcButton::Bu(3), CalcButton::Ur, CalcButton::Bu(9)],
            [CalcButton::Akm, CalcButton::Bu(0), CalcButton::Aay, CalcButton::Add, CalcButton::Wn],
        ];
        
        
        let wsh: [[CalcButton; 4]; 5] = [
            [CalcButton::Aan, CalcButton::Aao, CalcButton::Zz, CalcButton::Sc],
            [CalcButton::Bu(7), CalcButton::Bu(8), CalcButton::Bu(9), CalcButton::To],
            [CalcButton::Bu(4), CalcButton::Bu(5), CalcButton::Bu(6), CalcButton::Ur],
            [CalcButton::Bu(1), CalcButton::Bu(2), CalcButton::Bu(3), CalcButton::Add],
            [CalcButton::Akm, CalcButton::Bu(0), CalcButton::Aay, CalcButton::Wn],
        ];
        
        for (bwv, br) in wsh.iter().cf() {
            for (adq, bmc) in br.iter().cf() {
                let bx = b + (adq as u32) * (bym + qi);
                let je = c + (bwv as u32) * (doq + qi);
                
                self.gbv.push((*bmc, Rect::new(
                    bx as i32, je as i32, bym, doq
                )));
            }
        }
    }
    
    fn nau(&self, b: i32, c: i32) -> Option<usize> {
        for (a, (_, ha)) in self.gbv.iter().cf() {
            if ha.contains(Point::new(b, c)) {
                return Some(a);
            }
        }
        None
    }
    
    fn tjb(&mut self, bmc: CalcButton) {
        match bmc {
            CalcButton::Bu(bc) => {
                if self.daf {
                    self.display = format!("{}", bc);
                    self.daf = false;
                } else if self.display.len() < 15 {
                    if self.display == "0" {
                        self.display = format!("{}", bc);
                    } else {
                        self.display.push((b'0' + bc) as char);
                    }
                }
                self.cv = self.display.parse().unwrap_or(0.0);
            }
            
            CalcButton::Aay => {
                if self.daf {
                    self.display = String::from("0.");
                    self.daf = false;
                } else if !self.display.contains('.') {
                    self.display.push('.');
                }
            }
            
            CalcButton::Aan => {
                self.display = String::from("0");
                self.xz.clear();
                self.cv = 0.0;
                self.fre = 0.0;
                self.htr = None;
                self.daf = true;
            }
            
            CalcButton::Aao => {
                self.display = String::from("0");
                self.cv = 0.0;
                self.daf = true;
            }
            
            CalcButton::Zz => {
                if self.display.len() > 1 {
                    self.display.pop();
                } else {
                    self.display = String::from("0");
                }
                self.cv = self.display.parse().unwrap_or(0.0);
            }
            
            CalcButton::Akm => {
                if self.cv != 0.0 {
                    self.cv = -self.cv;
                    if self.display.cj('-') {
                        self.display = String::from(&self.display[1..]);
                    } else {
                        self.display = format!("-{}", self.display);
                    }
                }
            }
            
            CalcButton::Qk => {
                self.cv = self.cv / 100.0;
                self.display = self.hkd(self.cv);
            }
            
            CalcButton::Add | CalcButton::Ur | CalcButton::To | CalcButton::Sc => {
                self.nro();
                self.htr = Some(bmc);
                self.fre = self.cv;
                self.daf = true;
                
                let uyp = match bmc {
                    CalcButton::Add => "+",
                    CalcButton::Ur => "-",
                    CalcButton::To => "×",
                    CalcButton::Sc => "÷",
                    _ => "",
                };
                self.xz = format!("{} {}", self.display, uyp);
            }
            
            CalcButton::Wn => {
                self.nro();
                self.xz.clear();
                self.htr = None;
                self.daf = true;
            }
            
            CalcButton::Avm => {
                self.memory += self.cv;
            }
            CalcButton::Avq => {
                self.memory -= self.cv;
            }
            CalcButton::Avp => {
                self.cv = self.memory;
                self.display = self.hkd(self.memory);
                self.daf = true;
            }
            CalcButton::Avn => {
                self.memory = 0.0;
            }
        }
    }
    
    fn nro(&mut self) {
        if let Some(op) = self.htr {
            let result = match op {
                CalcButton::Add => self.fre + self.cv,
                CalcButton::Ur => self.fre - self.cv,
                CalcButton::To => self.fre * self.cv,
                CalcButton::Sc => {
                    if self.cv != 0.0 {
                        self.fre / self.cv
                    } else {
                        f64::Lx
                    }
                }
                _ => self.cv,
            };
            
            self.cv = result;
            self.display = self.hkd(result);
        }
    }
    
    fn hkd(&self, bo: f64) -> String {
        if bo.ogj() {
            return String::from("Error");
        }
        if bo.yzk() {
            return String::from("Infinity");
        }
        
        
        
        let txs = bo == (bo as i64) as f64 && bo.gp() < 1e15;
        if txs {
            format!("{:.0}", bo)
        } else {
            let e = format!("{:.10}", bo);
            
            let e = e.bdd('0');
            let e = e.bdd('.');
            String::from(e)
        }
    }
    
    fn qur(&self, bmc: &CalcButton, theme: &Theme, apx: bool, eth: bool) -> Color {
        let ar = match bmc {
            CalcButton::Wn => theme.mm,
            CalcButton::Add | CalcButton::Ur | CalcButton::To | CalcButton::Sc => {
                Color::new(80, 80, 90, 255)
            }
            CalcButton::Aan | CalcButton::Aao => {
                Color::new(120, 60, 60, 255)
            }
            _ => theme.dop,
        };
        
        if eth {
            ar.cdz(20)
        } else if apx {
            ar.clh(15)
        } else {
            ar
        }
    }
}

impl Cf for Calculator {
    fn ad(&self) -> u32 { self.ad }
    fn eg(&self) -> Rect { self.eg }
    fn cbq(&mut self, eg: Rect) { 
        self.eg = eg;
        self.udl();
    }
    fn g(&self) -> WidgetState { self.g }
    fn cbr(&mut self, g: WidgetState) { self.g = g; }
    
    fn ctk(&self) -> Size {
        Size::new(290, 400)
    }
    
    fn ecj(&mut self, id: &UiEvent) -> bool {
        match id {
            UiEvent::Cp(MouseEvent::Fw { b, c }) => {
                self.hnc = self.nau(*b, *c);
                true
            }
            UiEvent::Cp(MouseEvent::Fm { b, c, bdp: MouseButton::Ap }) => {
                self.gpr = self.nau(*b, *c);
                true
            }
            UiEvent::Cp(MouseEvent::Ek { bdp: MouseButton::Ap, .. }) => {
                if let Some(w) = self.gpr {
                    if Some(w) == self.hnc {
                        let bmc = self.gbv[w].0;
                        self.tjb(bmc);
                    }
                }
                self.gpr = None;
                true
            }
            UiEvent::Cp(MouseEvent::Tf) => {
                self.hnc = None;
                self.gpr = None;
                true
            }
            _ => false
        }
    }
    
    fn tj(&self, surface: &mut GpuSurface, theme: &Theme) {
        if !self.g.iw { return; }
        
        let b = self.eg.b as u32;
        let c = self.eg.c as u32;
        let d = self.eg.z;
        let i = self.eg.ac;
        
        
        surface.afp(b, c, d, i, 12, theme.gay.lv());
        surface.mf(b, c, d, i, 12, theme.acu.lv());
        
        
        surface.ah(b + 1, c + 1, d - 2, 28, theme.ems.lv());
        cb(surface, b as i32 + 10, c as i32 + 6, "Calculator", theme.ebn.lv());
        
        
        let hgh = b + 10;
        let hgi = c + 35;
        let dgi = d - 20;
        let eaw = 50;
        surface.afp(hgh, hgi, dgi, eaw, 6, Color::new(30, 30, 35, 255).lv());
        
        
        if !self.xz.is_empty() {
            let spm = hgh as i32 + dgi as i32 - (self.xz.len() as i32 * 8) - 8;
            cb(surface, spm, hgi as i32 + 4, &self.xz, theme.ebn.lv());
        }
        
        
        let ryr = hgh as i32 + dgi as i32 - (self.display.len() as i32 * 12) - 10;
        let nlz = hgi as i32 + 22;
        
        
        for (a, r) in self.display.bw().cf() {
            let cx = ryr + (a as i32 * 12);
            let inh = alloc::string::String::from(r);
            cb(surface, cx, nlz, &inh, theme.bui.lv());
            cb(surface, cx + 1, nlz, &inh, theme.bui.lv());
        }
        
        
        if self.memory != 0.0 {
            surface.afp(hgh, hgi, 20, 16, 3, theme.mm.fbo(60).lv());
            cb(surface, hgh as i32 + 4, hgi as i32 + 2, "M", theme.mm.lv());
        }
        
        
        for (a, (bmc, ha)) in self.gbv.iter().cf() {
            let apx = self.hnc == Some(a);
            let eth = self.gpr == Some(a);
            
            let vp = self.qur(bmc, theme, apx, eth);
            
            surface.afp(
                ha.b as u32, ha.c as u32,
                ha.z, ha.ac,
                8,
                vp.lv()
            );
            
            
            surface.mf(
                ha.b as u32, ha.c as u32,
                ha.z, ha.ac,
                8,
                theme.acu.lv()
            );
            
            
            let cu = bmc.cu();
            let bda = cu.len() as i32 * 8;
            let wg = ha.b + (ha.z as i32 - bda) / 2;
            let sl = ha.c + (ha.ac as i32 - 16) / 2;
            
            let agx = if oh!(bmc, CalcButton::Wn) {
                Color::Zm
            } else {
                theme.bui
            };
            
            cb(surface, wg, sl, cu, agx.lv());
        }
    }
}


pub fn rqj() -> Calculator {
    let mut akz = Calculator::new();
    akz.cbq(Rect::new(100, 100, 290, 400));
    akz
}
