








use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

use super::{Cf, Cj, Event, MouseEvent, Rect, Size, Color, CosmicRenderer, ButtonState, theme};






pub struct Vs {
    pub cu: String,
    pub jhu: Option<Cj>,
    pub amx: ButtonStyle,
    g: ButtonState,
    asy: bool,
    vn: bool,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ButtonStyle {
    Gc,
    Us,
    We,
    Text,
}

impl Vs {
    pub fn new(cu: &str) -> Self {
        Self {
            cu: String::from(cu),
            jhu: None,
            amx: ButtonStyle::Gc,
            g: ButtonState::M,
            asy: false,
            vn: false,
        }
    }
    
    pub fn jhu(mut self, fr: Cj) -> Self {
        self.jhu = Some(fr);
        self
    }
    
    pub fn amx(mut self, amx: ButtonStyle) -> Self {
        self.amx = amx;
        self
    }
    
    fn jut(&mut self) {
        self.g = if self.vn {
            ButtonState::Alg
        } else if self.asy {
            match self.amx {
                ButtonStyle::Us => ButtonState::Us,
                ButtonStyle::We => ButtonState::We,
                _ => ButtonState::Aiz,
            }
        } else {
            match self.amx {
                ButtonStyle::Us => ButtonState::Us,
                ButtonStyle::We => ButtonState::We,
                _ => ButtonState::M,
            }
        };
    }
}

impl Cf for Vs {
    fn aw(&self) -> Size {
        
        let idh = self.cu.len() as f32 * 8.0;
        Size::new(idh + 32.0, 36.0)
    }
    
    fn po(&self, renderer: &mut CosmicRenderer, eg: Rect) {
        renderer.sca(eg, &self.cu, self.g);
    }
    
    fn goi(&mut self, id: &Event, eg: Rect) -> Option<Cj> {
        match id {
            Event::Cp(MouseEvent::Fw { b, c }) => {
                self.asy = eg.contains(*b, *c);
                self.jut();
                None
            }
            Event::Cp(MouseEvent::Axb { b, c, .. }) => {
                if eg.contains(*b, *c) {
                    self.vn = true;
                    self.jut();
                }
                None
            }
            Event::Cp(MouseEvent::Release { b, c, .. }) => {
                if self.vn && eg.contains(*b, *c) {
                    self.vn = false;
                    self.jut();
                    return self.jhu;
                }
                self.vn = false;
                self.jut();
                None
            }
            _ => None,
        }
    }
}






pub struct Dy {
    pub text: String,
    pub s: Option<Color>,
    pub aw: LabelSize,
}

#[derive(Clone, Copy)]
pub enum LabelSize {
    Ew,
    M,
    Ht,
    Bul,
}

impl Dy {
    pub fn new(text: &str) -> Self {
        Self {
            text: String::from(text),
            s: None,
            aw: LabelSize::M,
        }
    }
    
    pub fn s(mut self, r: Color) -> Self {
        self.s = Some(r);
        self
    }
    
    pub fn zoo(mut self, e: LabelSize) -> Self {
        self.aw = e;
        self
    }
}

impl Cf for Dy {
    fn aw(&self) -> Size {
        let nk = match self.aw {
            LabelSize::Ew => 6.0,
            LabelSize::M => 8.0,
            LabelSize::Ht => 10.0,
            LabelSize::Bul => 14.0,
        };
        let ac = match self.aw {
            LabelSize::Ew => 14.0,
            LabelSize::M => 18.0,
            LabelSize::Ht => 24.0,
            LabelSize::Bul => 32.0,
        };
        Size::new(self.text.len() as f32 * nk, ac)
    }
    
    fn po(&self, renderer: &mut CosmicRenderer, eg: Rect) {
        
        
        let ab = theme();
        let s = self.s.unwrap_or(ab.dcp);
        
        
        renderer.ahj(
            super::Point::new(eg.b, eg.c + eg.ac - 2.0),
            super::Point::new(eg.b + eg.z, eg.c + eg.ac - 2.0),
            s.fbo(0.3),
            1.0,
        );
    }
    
    fn goi(&mut self, qbu: &Event, qbh: Rect) -> Option<Cj> {
        None 
    }
}






pub struct Bds {
    zf: Vec<Box<dyn Cf>>,
    pub ob: f32,
    pub aoa: f32,
    pub sz: Direction,
    pub cop: Option<Color>,
    pub avh: f32,
}

#[derive(Clone, Copy)]
pub enum Direction {
    On,
    Po,
}

impl Bds {
    pub fn new() -> Self {
        Self {
            zf: Vec::new(),
            ob: 12.0,
            aoa: 8.0,
            sz: Direction::On,
            cop: None,
            avh: 0.0,
        }
    }
    
    pub fn push<Cpy: Cf + 'static>(mut self, bsy: Cpy) -> Self {
        self.zf.push(Box::new(bsy));
        self
    }
    
    pub fn ob(mut self, ai: f32) -> Self {
        self.ob = ai;
        self
    }
    
    pub fn aoa(mut self, e: f32) -> Self {
        self.aoa = e;
        self
    }
    
    pub fn sz(mut self, bc: Direction) -> Self {
        self.sz = bc;
        self
    }
    
    pub fn cop(mut self, r: Color) -> Self {
        self.cop = Some(r);
        self
    }
    
    pub fn avh(mut self, m: f32) -> Self {
        self.avh = m;
        self
    }
}

impl Cf for Bds {
    fn aw(&self) -> Size {
        let mut z = 0.0f32;
        let mut ac = 0.0f32;
        
        for aeh in &self.zf {
            let e = aeh.aw();
            match self.sz {
                Direction::On => {
                    z = z.am(e.z);
                    ac += e.ac + self.aoa;
                }
                Direction::Po => {
                    z += e.z + self.aoa;
                    ac = ac.am(e.ac);
                }
            }
        }
        
        Size::new(
            z + self.ob * 2.0,
            ac + self.ob * 2.0 - self.aoa,
        )
    }
    
    fn po(&self, renderer: &mut CosmicRenderer, eg: Rect) {
        
        if let Some(ei) = self.cop {
            if self.avh > 0.0 {
                renderer.afp(eg, self.avh, ei);
            } else {
                renderer.ah(eg, ei);
            }
        }
        
        
        let mut l = self.ob;
        
        for aeh in &self.zf {
            let e = aeh.aw();
            let khl = match self.sz {
                Direction::On => {
                    let o = Rect::new(
                        eg.b + self.ob,
                        eg.c + l,
                        eg.z - self.ob * 2.0,
                        e.ac,
                    );
                    l += e.ac + self.aoa;
                    o
                }
                Direction::Po => {
                    let o = Rect::new(
                        eg.b + l,
                        eg.c + self.ob,
                        e.z,
                        eg.ac - self.ob * 2.0,
                    );
                    l += e.z + self.aoa;
                    o
                }
            };
            
            aeh.po(renderer, khl);
        }
    }
    
    fn goi(&mut self, id: &Event, eg: Rect) -> Option<Cj> {
        
        let mut l = self.ob;
        
        for aeh in &mut self.zf {
            let e = aeh.aw();
            let khl = match self.sz {
                Direction::On => {
                    let o = Rect::new(
                        eg.b + self.ob,
                        eg.c + l,
                        eg.z - self.ob * 2.0,
                        e.ac,
                    );
                    l += e.ac + self.aoa;
                    o
                }
                Direction::Po => {
                    let o = Rect::new(
                        eg.b + l,
                        eg.c + self.ob,
                        e.z,
                        eg.ac - self.ob * 2.0,
                    );
                    l += e.z + self.aoa;
                    o
                }
            };
            
            if let Some(fr) = aeh.goi(id, khl) {
                return Some(fr);
            }
        }
        
        None
    }
}






pub struct Biq {
    pub dq: String,
    pub wnf: bool,
    ja: bool,
}

impl Biq {
    pub fn new(dq: &str) -> Self {
        Self {
            dq: String::from(dq),
            wnf: true,
            ja: true,
        }
    }
    
    pub fn zmz(&mut self, bb: bool) {
        self.ja = bb;
    }
}

impl Cf for Biq {
    fn aw(&self) -> Size {
        Size::new(400.0, 40.0) 
    }
    
    fn po(&self, renderer: &mut CosmicRenderer, eg: Rect) {
        renderer.nnb(eg, &self.dq, self.ja);
    }
    
    fn goi(&mut self, id: &Event, eg: Rect) -> Option<Cj> {
        
        if let Event::Cp(MouseEvent::Axb { b, c, .. }) = id {
            let ask = 14.0;
            let kn = eg.c + (eg.ac - ask) / 2.0;
            
            
            let bdr = eg.b + eg.z - ask - 12.0;
            if *b >= bdr && *b <= bdr + ask &&
               *c >= kn && *c <= kn + ask {
                return Some(1); 
            }
            
            
            let bvj = bdr - ask - 8.0;
            if *b >= bvj && *b <= bvj + ask &&
               *c >= kn && *c <= kn + ask {
                return Some(2); 
            }
            
            
            let cso = bvj - ask - 8.0;
            if *b >= cso && *b <= cso + ask &&
               *c >= kn && *c <= kn + ask {
                return Some(3); 
            }
        }
        
        None
    }
}






pub struct Dz {
    pub ac: f32,
}

impl Dz {
    pub fn new() -> Self {
        Self { ac: 32.0 }
    }
}

impl Cf for Dz {
    fn aw(&self) -> Size {
        Size::new(1280.0, self.ac)
    }
    
    fn po(&self, renderer: &mut CosmicRenderer, eg: Rect) {
        renderer.nnl(eg);
    }
    
    fn goi(&mut self, qbu: &Event, qbh: Rect) -> Option<Cj> {
        None
    }
}
