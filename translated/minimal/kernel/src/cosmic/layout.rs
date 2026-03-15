



use super::{Rect, Size};


#[derive(Clone, Copy, Debug)]
pub struct Bzz {
    pub czx: f32,
    pub dtb: f32,
    pub dtg: f32,
    pub hrb: f32,
}

impl Bzz {
    pub fn new(uon: f32, hrh: f32, uok: f32, hra: f32) -> Self {
        Self {
            czx: uon,
            dtb: hrh,
            dtg: uok,
            hrb: hra,
        }
    }
    
    pub fn zso(aw: Size) -> Self {
        Self {
            czx: aw.z,
            dtb: aw.z,
            dtg: aw.ac,
            hrb: aw.ac,
        }
    }
    
    pub fn zbs(aw: Size) -> Self {
        Self {
            czx: 0.0,
            dtb: aw.z,
            dtg: 0.0,
            hrb: aw.ac,
        }
    }
    
    pub fn ztw() -> Self {
        Self {
            czx: 0.0,
            dtb: f32::Att,
            dtg: 0.0,
            hrb: f32::Att,
        }
    }
    
    pub fn yjw(&self, aw: Size) -> Size {
        Size::new(
            aw.z.am(self.czx).v(self.dtb),
            aw.ac.am(self.dtg).v(self.hrb),
        )
    }
}


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Crk {
    Oe,
    Eo,
    Wm,
    Uq,
}


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Dch {
    Oe,
    Eo,
    Wm,
    Cnh,
    Cng,
    Cni,
}


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cto {
    Oe,
    Eo,
    Wm,
    Uq,
}


#[derive(Clone, Copy, Debug, Default)]
pub struct EdgeInsets {
    pub qc: f32,
    pub hw: f32,
    pub abm: f32,
    pub fd: f32,
}

impl EdgeInsets {
    pub const fn xx(bn: f32) -> Self {
        Self {
            qc: bn,
            hw: bn,
            abm: bn,
            fd: bn,
        }
    }
    
    pub const fn wwy(cns: f32, dic: f32) -> Self {
        Self {
            qc: cns,
            abm: cns,
            fd: dic,
            hw: dic,
        }
    }
    
    pub const fn uyk(qc: f32, hw: f32, abm: f32, fd: f32) -> Self {
        Self { qc, hw, abm, fd }
    }
    
    pub fn dic(&self) -> f32 {
        self.fd + self.hw
    }
    
    pub fn cns(&self) -> f32 {
        self.qc + self.abm
    }
    
    pub fn yln(&self, ha: Rect) -> Rect {
        Rect::new(
            ha.b + self.fd,
            ha.c + self.qc,
            ha.z - self.dic(),
            ha.ac - self.cns(),
        )
    }
    
    pub fn inflate(&self, ha: Rect) -> Rect {
        Rect::new(
            ha.b - self.fd,
            ha.c - self.qc,
            ha.z + self.dic(),
            ha.ac + self.cns(),
        )
    }
}
