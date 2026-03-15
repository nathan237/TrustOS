































use alloc::vec::Vec;


pub mod req_type {
    pub const DAS_: u32 = 0;     
    pub const DAT_: u32 = 1;    
    pub const DAQ_: u32 = 4;  
    pub const DAR_: u32 = 8; 
}


pub mod status {
    pub const YI_: u8 = 0;
    pub const EJZ_: u8 = 1;
    pub const DAP_: u8 = 2;
}


pub mod device_status {
    pub const Or: u8 = 1;
    pub const Fl: u8 = 2;
    pub const HW_: u8 = 4;
    pub const MZ_: u8 = 8;
    pub const BRL_: u8 = 64;
    pub const Arw: u8 = 128;
}


pub mod features {
    pub const DAO_: u32 = 1 << 1;
    pub const DAN_: u32 = 1 << 2;
    pub const EJX_: u32 = 1 << 4;
    pub const EJY_: u32 = 1 << 5;
    pub const EJW_: u32 = 1 << 6;
    pub const DAM_: u32 = 1 << 9;
}


pub const H_: usize = 512;


#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct VirtqDesc {
    
    pub ag: u64,
    
    pub len: u32,
    
    pub flags: u16,
    
    pub next: u16,
}


#[derive(Debug, Clone)]
pub struct VirtioBlkState {
    
    pub bju: u32,
    
    pub cyj: u32,
    
    pub dky: u16,
    
    pub gqd: u32,
    
    pub art: u16,
    
    pub device_status: u8,
    
    pub czc: u8,
    
    pub gce: u64,
    
    pub gks: u16,
}

impl Default for VirtioBlkState {
    fn default() -> Self {
        Self {
            bju: features::DAO_
                           | features::DAN_
                           | features::DAM_,
            cyj: 0,
            dky: 0,
            gqd: 0,
            art: 128, 
            device_status: 0,
            czc: 0,
            gce: 64, 
            gks: 0,
        }
    }
}

impl VirtioBlkState {
    
    pub fn fc(wuq: usize) -> Self {
        let mut g = Self::default();
        g.gce = (wuq / H_) as u64;
        g
    }
    
    
    
    pub fn crq(&mut self, l: u16) -> u32 {
        match l {
            
            0x00 => self.bju,
            
            0x04 => self.cyj,
            
            0x08 => self.gqd,
            
            0x0C => self.art as u32,
            
            0x0E => self.dky as u32,
            
            0x12 => self.device_status as u32,
            
            0x13 => {
                let ap = self.czc as u32;
                self.czc = 0;
                ap
            }
            
            0x14 => (self.gce & 0xFFFF_FFFF) as u32,
            
            0x18 => ((self.gce >> 32) & 0xFFFF_FFFF) as u32,
            
            0x1C => 0x1000, 
            
            0x20 => 128,   
            _ => 0,
        }
    }
    
    
    
    
    pub fn edp(&mut self, l: u16, bn: u32) -> bool {
        match l {
            
            0x04 => {
                self.cyj = bn;
            }
            
            0x08 => {
                self.gqd = bn;
            }
            
            0x0E => {
                self.dky = bn as u16;
            }
            
            0x10 => {
                
                return true;
            }
            
            0x12 => {
                self.device_status = bn as u8;
                if bn == 0 {
                    
                    self.cyj = 0;
                    self.gqd = 0;
                    self.dky = 0;
                    self.czc = 0;
                    self.gks = 0;
                }
            }
            _ => {}
        }
        false
    }
    
    
    
    
    pub fn vmr(&mut self, fe: &mut [u8], storage: &mut [u8]) -> usize {
        if self.gqd == 0 || self.device_status & device_status::HW_ == 0 {
            return 0;
        }
        
        let lwr = (self.gqd as u64) * 4096; 
        let art = self.art as usize;
        
        
        
        
        
        let dpv = lwr as usize;
        let kbl = dpv + art * 16;
        let doe = (kbl + 1) & !1; 
        let xpp = doe + 4 + art * 2 + 2; 
        let gvv = (xpp + 4095) & !4095; 
        
        
        if doe + 2 >= fe.len() {
            return 0;
        }
        let kbn = u16::dj([
            fe[doe + 2],
            fe[doe + 3],
        ]);
        
        let mut oyd = 0usize;
        
        while self.gks != kbn {
            let jml = (self.gks as usize) % art;
            let fsx = doe + 4 + jml * 2;
            
            if fsx + 2 > fe.len() {
                break;
            }
            
            let and = u16::dj([
                fe[fsx],
                fe[fsx + 1],
            ]) as usize;
            
            
            let xpr = self.vms(fe, storage, dpv, and, art);
            
            
            if gvv + 4 >= fe.len() {
                break;
            }
            let pxq = u16::dj([
                fe[gvv + 2],
                fe[gvv + 3],
            ]);
            let igc = gvv + 4 + (pxq as usize % art) * 8;
            
            if igc + 8 <= fe.len() {
                
                let trh = (and as u32).ho();
                let udx = (xpr as u32).ho();
                fe[igc..igc + 4].dg(&trh);
                fe[igc + 4..igc + 8].dg(&udx);
                
                
                let utx = pxq.cn(1);
                let bf = utx.ho();
                fe[gvv + 2] = bf[0];
                fe[gvv + 3] = bf[1];
            }
            
            self.gks = self.gks.cn(1);
            oyd += 1;
            
            
            self.czc |= 1;
        }
        
        oyd
    }
    
    
    fn vms(
        &self,
        fe: &mut [u8],
        storage: &mut [u8],
        dpv: usize,
        nuu: usize,
        art: usize,
    ) -> usize {
        
        let dh = self.paa(fe, dpv, nuu);
        if dh.ag as usize + 16 > fe.len() {
            return 0;
        }
        
        
        let req_type = u32::dj([
            fe[dh.ag as usize],
            fe[dh.ag as usize + 1],
            fe[dh.ag as usize + 2],
            fe[dh.ag as usize + 3],
        ]);
        let jk = u64::dj([
            fe[dh.ag as usize + 8],
            fe[dh.ag as usize + 9],
            fe[dh.ag as usize + 10],
            fe[dh.ag as usize + 11],
            fe[dh.ag as usize + 12],
            fe[dh.ag as usize + 13],
            fe[dh.ag as usize + 14],
            fe[dh.ag as usize + 15],
        ]);
        
        
        let mut aeb = 0usize;
        let mut cv = nuu;
        let mut hfd: [(u64, u32, u16); 16] = [(0, 0, 0); 16];
        let mut dpp = 0usize;
        let mut cmy: Option<(u64, u32)> = None;
        let mut ofy = true;
        
        loop {
            let desc = self.paa(fe, dpv, cv);
            
            if ofy {
                ofy = false;
            } else if desc.flags & 2 != 0 {
                
                
                if desc.len == 1 {
                    cmy = Some((desc.ag, desc.len));
                } else {
                    if dpp < 16 {
                        hfd[dpp] = (desc.ag, desc.len, desc.flags);
                        dpp += 1;
                    }
                }
            } else {
                
                if dpp < 16 {
                    hfd[dpp] = (desc.ag, desc.len, desc.flags);
                    dpp += 1;
                }
            }
            
            if desc.flags & 1 == 0 {
                
                
                if cmy.is_none() && desc.len == 1 {
                    cmy = Some((desc.ag, desc.len));
                    if dpp > 0 {
                        dpp -= 1; 
                    }
                }
                break;
            }
            
            cv = desc.next as usize;
            if cv >= art {
                break;
            }
        }
        
        
        let vyh = match req_type {
            req_type::DAS_ => {
                
                let mut l = jk as usize * H_;
                for a in 0..dpp {
                    let (ag, len, _) = hfd[a];
                    let ag = ag as usize;
                    let len = len as usize;
                    if l + len <= storage.len() && ag + len <= fe.len() {
                        fe[ag..ag + len].dg(&storage[l..l + len]);
                        aeb += len;
                    }
                    l += len;
                }
                status::YI_
            }
            req_type::DAT_ => {
                
                let mut l = jk as usize * H_;
                for a in 0..dpp {
                    let (ag, len, _) = hfd[a];
                    let ag = ag as usize;
                    let len = len as usize;
                    if l + len <= storage.len() && ag + len <= fe.len() {
                        storage[l..l + len].dg(&fe[ag..ag + len]);
                        aeb += len;
                    }
                    l += len;
                }
                status::YI_
            }
            req_type::DAQ_ => {
                
                status::YI_
            }
            req_type::DAR_ => {
                
                let izg = b"trustos-virtio-blk\0";
                if let Some((ag, _, _)) = hfd.fv() {
                    let ag = *ag as usize;
                    let zg = izg.len().v(20);
                    if ag + zg <= fe.len() {
                        fe[ag..ag + zg].dg(&izg[..zg]);
                        aeb += zg;
                    }
                }
                status::YI_
            }
            _ => status::DAP_,
        };
        
        
        if let Some((ag, _)) = cmy {
            let ag = ag as usize;
            if ag < fe.len() {
                fe[ag] = vyh;
                aeb += 1;
            }
        }
        
        aeb
    }
    
    
    fn paa(&self, fe: &[u8], dpv: usize, index: usize) -> VirtqDesc {
        let l = dpv + index * 16;
        if l + 16 > fe.len() {
            return VirtqDesc::default();
        }
        
        VirtqDesc {
            ag: u64::dj([
                fe[l], fe[l + 1],
                fe[l + 2], fe[l + 3],
                fe[l + 4], fe[l + 5],
                fe[l + 6], fe[l + 7],
            ]),
            len: u32::dj([
                fe[l + 8], fe[l + 9],
                fe[l + 10], fe[l + 11],
            ]),
            flags: u16::dj([
                fe[l + 12], fe[l + 13],
            ]),
            next: u16::dj([
                fe[l + 14], fe[l + 15],
            ]),
        }
    }
}
















#[derive(Debug, Clone)]
pub struct VirtioConsoleState {
    
    pub bju: u32,
    
    pub cyj: u32,
    
    pub dky: u16,
    
    pub jkv: u32,
    
    pub gqe: u32,
    
    pub art: u16,
    
    pub device_status: u8,
    
    pub czc: u8,
    
    pub ec: u16,
    
    pub lk: u16,
    
    pub jfk: u32,
    
    pub gve: u16,
}

impl Default for VirtioConsoleState {
    fn default() -> Self {
        Self {
            bju: 0, 
            cyj: 0,
            dky: 0,
            jkv: 0,
            gqe: 0,
            art: 64,
            device_status: 0,
            czc: 0,
            ec: 80,
            lk: 25,
            jfk: 1,
            gve: 0,
        }
    }
}

impl VirtioConsoleState {
    
    
    pub fn crq(&mut self, l: u16) -> u32 {
        match l {
            0x00 => self.bju,
            0x04 => self.cyj,
            0x08 => {
                match self.dky {
                    0 => self.jkv,
                    1 => self.gqe,
                    _ => 0,
                }
            }
            0x0C => self.art as u32,
            0x0E => self.dky as u32,
            0x12 => self.device_status as u32,
            0x13 => {
                let ap = self.czc as u32;
                self.czc = 0;
                ap
            }
            
            0x14 => self.ec as u32,
            0x16 => self.lk as u32,
            0x18 => self.jfk,
            _ => 0,
        }
    }
    
    
    
    pub fn edp(&mut self, l: u16, bn: u32) -> bool {
        match l {
            0x04 => { self.cyj = bn; }
            0x08 => {
                match self.dky {
                    0 => self.jkv = bn,
                    1 => self.gqe = bn,
                    _ => {}
                }
            }
            0x0E => { self.dky = bn as u16; }
            0x10 => {
                
                let vpj = bn as u16;
                if vpj == 1 {
                    
                    return true;
                }
            }
            0x12 => {
                self.device_status = bn as u8;
                if bn == 0 {
                    
                    self.cyj = 0;
                    self.jkv = 0;
                    self.gqe = 0;
                    self.dky = 0;
                    self.czc = 0;
                    self.gve = 0;
                }
            }
            
            0x1C => {
                let bm = (bn & 0xFF) as u8;
                crate::serial_print!("{}", bm as char);
            }
            _ => {}
        }
        false
    }
    
    
    
    pub fn vmv(&mut self, fe: &[u8]) -> usize {
        if self.gqe == 0 || self.device_status & device_status::HW_ == 0 {
            return 0;
        }
        
        let lwr = (self.gqe as u64) * 4096;
        let art = self.art as usize;
        
        let dpv = lwr as usize;
        let kbl = dpv + art * 16;
        let doe = (kbl + 1) & !1;
        
        if doe + 4 > fe.len() {
            return 0;
        }
        
        let kbn = u16::dj([
            fe[doe + 2],
            fe[doe + 3],
        ]);
        
        let mut xv = 0usize;
        
        while self.gve != kbn {
            let jml = (self.gve as usize) % art;
            let fsx = doe + 4 + jml * 2;
            
            if fsx + 2 > fe.len() {
                break;
            }
            
            let and = u16::dj([
                fe[fsx],
                fe[fsx + 1],
            ]) as usize;
            
            
            let l = dpv + and * 16;
            if l + 16 <= fe.len() {
                let ag = u64::dj([
                    fe[l], fe[l + 1],
                    fe[l + 2], fe[l + 3],
                    fe[l + 4], fe[l + 5],
                    fe[l + 6], fe[l + 7],
                ]) as usize;
                let len = u32::dj([
                    fe[l + 8], fe[l + 9],
                    fe[l + 10], fe[l + 11],
                ]) as usize;
                
                
                if ag + len <= fe.len() {
                    for a in 0..len {
                        crate::serial_print!("{}", fe[ag + a] as char);
                    }
                    xv += len;
                }
            }
            
            self.gve = self.gve.cn(1);
            self.czc |= 1;
        }
        
        xv
    }
}
