





use core::sync::atomic::Ordering;
use alloc::vec::Vec;
use alloc::boxed::Box;
use crate::arch::Port;


pub mod status {
    pub const Or: u8 = 1;
    pub const Fl: u8 = 2;
    pub const HW_: u8 = 4;
    pub const MZ_: u8 = 8;
    pub const BRL_: u8 = 64;
    pub const Arw: u8 = 128;
}


pub mod cap_type {
    pub const AOU_: u8 = 1;
    pub const BBO_: u8 = 2;
    pub const AXK_: u8 = 3;
    pub const AQA_: u8 = 4;
    pub const DZE_: u8 = 5;
}


pub mod legacy_reg {
    pub const BRK_: u16 = 0x00;      
    pub const BSE_: u16 = 0x04;      
    pub const CMV_: u16 = 0x08;        
    pub const WM_: u16 = 0x0C;           
    pub const AGU_: u16 = 0x0E;         
    pub const CNB_: u16 = 0x10;         
    pub const EF_: u16 = 0x12;        
    pub const CCO_: u16 = 0x13;           
    
    pub const AFW_: u16 = 0x14;              
    pub const DVC_: u16 = 0x1A;           
}


pub mod desc_flags {
    pub const Akj: u16 = 1;       
    pub const Db: u16 = 2;      
    pub const Cyr: u16 = 4;   
}


#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct VirtqDesc {
    pub ag: u64,    
    pub len: u32,     
    pub flags: u16,   
    pub next: u16,    
}


#[repr(C)]
#[derive(Debug)]
pub struct Zk {
    pub flags: u16,
    pub w: u16,
    
}


#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Aob {
    pub ad: u32,    
    pub len: u32,   
}


#[repr(C)]
#[derive(Debug)]
pub struct Zl {
    pub flags: u16,
    pub w: u16,
    
}


pub struct Virtqueue {
    
    pub aw: u16,
    
    pub ki: u64,
    
    pub desc: *mut VirtqDesc,
    
    pub apk: *mut Zk,
    
    pub mr: *mut Zl,
    
    pub csa: u16,
    
    pub cyb: u16,
    
    pub dts: u16,
    
    pub buk: Vec<u16>,
}



unsafe impl Send for Virtqueue {}
unsafe impl Sync for Virtqueue {}

impl Virtqueue {
    
    pub fn nbh(art: u16) -> usize {
        let kpj = core::mem::size_of::<VirtqDesc>() * art as usize;
        let qlr = 6 + 2 * art as usize; 
        let fxx = 6 + 8 * art as usize;  
        
        
        let fcw = kpj;
        let dxe = ((fcw + qlr) + 4095) & !4095; 
        
        dxe + fxx
    }
    
    
    pub fn blx(&mut self) -> Option<u16> {
        if self.dts == 0 {
            return None;
        }
        
        let w = self.cyb;
        self.cyb = self.buk[w as usize];
        self.dts -= 1;
        Some(w)
    }
    
    
    pub fn ald(&mut self, w: u16) {
        self.buk[w as usize] = self.cyb;
        self.cyb = w;
        self.dts += 1;
    }
    
    
    pub unsafe fn gxv(&mut self, ale: u16) {
        let apk = &mut *self.apk;
        let fsy = (self.apk as *mut u8).add(4) as *mut u16;
        let w = apk.w;
        *fsy.add((w % self.aw) as usize) = ale;
        
        
        core::sync::atomic::cxt(Ordering::Release);
        
        apk.w = w.cn(1);
    }


    pub fn new(aw: u16) -> Result<Box<Self>, &'static str> {
        use alloc::alloc::{alloc_zeroed, Layout};
        
        let aay = Self::nbh(aw);
        let layout = Layout::bjy(aay, 4096)
            .jd(|_| "Invalid layout")?;
        
        let ptr = unsafe { alloc_zeroed(layout) };
        if ptr.abq() {
            return Err("Failed to allocate virtqueue");
        }
        
        let ki = ptr as u64; 
        
        
        let kpj = core::mem::size_of::<VirtqDesc>() * aw as usize;
        let fcw = kpj;
        let dxe = ((fcw + 6 + 2 * aw as usize) + 4095) & !4095;
        
        let desc = ptr as *mut VirtqDesc;
        let apk = unsafe { ptr.add(fcw) as *mut Zk };
        let mr = unsafe { ptr.add(dxe) as *mut Zl };
        
        
        let mut buk = alloc::vec::Vec::fc(aw as usize);
        for a in 0..aw {
            buk.push(a + 1);
        }
        if aw > 0 {
            buk[aw as usize - 1] = 0;
        }
        
        Ok(Box::new(Self {
            aw,
            ki,
            desc,
            apk,
            mr,
            csa: 0,
            cyb: 0,
            dts: aw,
            buk,
        }))
    }
    
    
    pub fn bwz(&mut self, w: u16, ag: u64, len: u32, flags: u16, next: u16) {
        unsafe {
            let desc = &mut *self.desc.add(w as usize);
            desc.ag = ag;
            desc.len = len;
            desc.flags = flags;
            desc.next = next;
        }
    }
    
    
    pub fn dmd(&mut self, ale: u16) {
        unsafe { self.gxv(ale) }
    }
    
    
    pub fn ixy(&self) -> bool {
        unsafe { 
            let mr = &*self.mr;
            core::sync::atomic::cxt(Ordering::Acquire);
            mr.w != self.csa
        }
    }
    
    
    pub fn jjp(&mut self) -> Option<(u32, u32)> {
        unsafe {
            let mr = &*self.mr;
            core::sync::atomic::cxt(Ordering::Acquire);
            
            if mr.w == self.csa {
                return None;
            }
            
            let fsy = (self.mr as *mut u8).add(4) as *mut Aob;
            let fhm = *fsy.add((self.csa % self.aw) as usize);
            self.csa = self.csa.cn(1);
            
            Some((fhm.ad, fhm.len))
        }
    }
}


pub struct VirtioDevice {
    
    pub agq: u16,
    
    pub bju: u32,
    
    pub ckb: u32,
}

impl VirtioDevice {
    
    pub fn new(agq: u16) -> Self {
        Self {
            agq,
            bju: 0,
            ckb: 0,
        }
    }
    
    
    pub fn vsp(&self) -> u8 {
        unsafe {
            let mut port = Port::<u8>::new(self.agq + legacy_reg::EF_);
            port.read()
        }
    }
    
    
    pub fn qac(&mut self, status: u8) {
        unsafe {
            let mut port = Port::<u8>::new(self.agq + legacy_reg::EF_);
            port.write(status);
        }
    }
    
    
    pub fn fzu(&mut self, fs: u8) {
        let cv = self.vsp();
        self.qac(cv | fs);
    }
    
    
    pub fn apa(&mut self) {
        self.qac(0);
    }
    
    
    pub fn pab(&mut self) -> u32 {
        unsafe {
            let mut port = Port::<u32>::new(self.agq + legacy_reg::BRK_);
            self.bju = port.read();
            self.bju
        }
    }
    
    
    pub fn pzx(&mut self, features: u32) {
        self.ckb = features;
        unsafe {
            let mut port = Port::<u32>::new(self.agq + legacy_reg::BSE_);
            port.write(features);
        }
    }
    
    
    pub fn mdl(&mut self, queue: u16) {
        unsafe {
            let mut port = Port::<u16>::new(self.agq + legacy_reg::AGU_);
            port.write(queue);
        }
    }
    
    
    pub fn kyw(&self) -> u16 {
        unsafe {
            let mut port = Port::<u16>::new(self.agq + legacy_reg::WM_);
            port.read()
        }
    }
    
    
    pub fn meu(&mut self, duh: u32) {
        unsafe {
            let mut port = Port::<u32>::new(self.agq + legacy_reg::CMV_);
            port.write(duh);
        }
    }
    
    
    pub fn jhj(&self, queue: u16) {
        unsafe {
            let mut port = Port::<u16>::new(self.agq + legacy_reg::CNB_);
            port.write(queue);
        }
    }
    
    
    pub fn zhv(&self) -> u8 {
        unsafe {
            let mut port = Port::<u8>::new(self.agq + legacy_reg::CCO_);
            port.read()
        }
    }
    
    
    pub fn vrj(&self, l: u16) -> u8 {
        unsafe {
            let mut port = Port::<u8>::new(self.agq + legacy_reg::AFW_ + l);
            port.read()
        }
    }
    
    
    pub fn vri(&self, l: u16) -> u16 {
        unsafe {
            let mut port = Port::<u16>::new(self.agq + legacy_reg::AFW_ + l);
            port.read()
        }
    }
    
    
    pub fn ozw(&self, l: u16) -> u32 {
        unsafe {
            let mut port = Port::<u32>::new(self.agq + legacy_reg::AFW_ + l);
            port.read()
        }
    }
}
