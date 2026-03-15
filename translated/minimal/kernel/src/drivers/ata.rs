




use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;
use crate::arch::Port;


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IdeChannel {
    Adx,
    Aeq,
}


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DrivePosition {
    Ake,
    Ams,
}


mod cmd {
    pub const Cfg: u8 = 0xEC;
    pub const CBK_: u8 = 0xA1;
    pub const CNM_: u8 = 0x20;
    pub const CNN_: u8 = 0x24;
    pub const DBQ_: u8 = 0x30;
    pub const DBR_: u8 = 0x34;
    pub const BLQ_: u8 = 0xE7;
    pub const BLR_: u8 = 0xEA;
}


mod status {
    pub const Bfm: u8 = 1 << 0;
    pub const Car: u8 = 1 << 3;
    pub const Caf: u8 = 1 << 5;
    pub const Bcl: u8 = 1 << 7;
}


const PB_: u16 = 0x1F0;
const AGP_: u16 = 0x3F6;
const WW_: u16 = 0x170;
const AHO_: u16 = 0x376;


#[derive(Clone, Debug)]
pub struct Ajc {
    pub channel: IdeChannel,
    pub qf: DrivePosition,
    pub brs: bool,
    pub gal: bool,
    pub gle: bool,
    pub agw: u64,
    pub model: String,
    pub serial: String,
}


pub struct Bjl {
    pub bzh: Vec<Ajc>,
    pub jr: bool,
}

static Bn: Mutex<Option<Bjl>> = Mutex::new(None);


fn iha(ar: u16) -> Result<(), &'static str> {
    let mut dma = Port::<u8>::new(ar + 7);
    
    for _ in 0..100_000 {
        let status = unsafe { dma.read() };
        
        if status == 0xFF {
            return Err("No drive");
        }
        
        if status & status::Bcl == 0 {
            if status & status::Bfm != 0 {
                return Err("Drive error");
            }
            if status & status::Caf != 0 {
                return Err("Drive fault");
            }
            return Ok(());
        }
        
        core::hint::hc();
    }
    
    Err("Timeout")
}


fn mqd(ar: u16) -> Result<(), &'static str> {
    let mut dma = Port::<u8>::new(ar + 7);
    
    for _ in 0..100_000 {
        let status = unsafe { dma.read() };
        
        if status & status::Bfm != 0 {
            return Err("Drive error");
        }
        
        if status & status::Bcl == 0 && status & status::Car != 0 {
            return Ok(());
        }
        
        core::hint::hc();
    }
    
    Err("Timeout waiting for DRQ")
}


fn ply(ron: u16) {
    let mut control = Port::<u8>::new(ron);
    unsafe {
        control.write(0x04);
        for _ in 0..1000 { core::hint::hc(); }
        control.write(0x00);
        for _ in 0..10000 { core::hint::hc(); }
    }
}


fn mdk(ar: u16, cbv: bool) {
    let mut sgv = Port::<u8>::new(ar + 6);
    unsafe {
        sgv.write(if cbv { 0xB0 } else { 0xA0 });
        for _ in 0..4 {
            let _ = Port::<u8>::new(ar + 7).read();
        }
    }
}


fn izi(ar: u16, xyo: u16, cbv: bool) -> Option<Ajc> {
    mdk(ar, cbv);
    
    
    unsafe {
        Port::<u8>::new(ar + 2).write(0);
        Port::<u8>::new(ar + 3).write(0);
        Port::<u8>::new(ar + 4).write(0);
        Port::<u8>::new(ar + 5).write(0);
    }
    
    
    unsafe {
        Port::<u8>::new(ar + 7).write(cmd::Cfg);
    }
    
    
    let status = unsafe { Port::<u8>::new(ar + 7).read() };
    if status == 0 {
        return None;
    }
    
    
    let mut gal = false;
    if iha(ar).is_err() {
        let udo = unsafe { Port::<u8>::new(ar + 4).read() };
        let udn = unsafe { Port::<u8>::new(ar + 5).read() };
        
        if udo == 0x14 && udn == 0xEB {
            gal = true;
            unsafe {
                Port::<u8>::new(ar + 7).write(cmd::CBK_);
            }
            if iha(ar).is_err() {
                return None;
            }
        } else {
            return None;
        }
    }
    
    
    if mqd(ar).is_err() {
        return None;
    }
    
    
    let mut f = [0u16; 256];
    let mut axr = Port::<u16>::new(ar);
    for a in 0..256 {
        f[a] = unsafe { axr.read() };
    }
    
    
    let gle = (f[83] & (1 << 10)) != 0;
    
    let agw = if gle {
        (f[100] as u64) |
        ((f[101] as u64) << 16) |
        ((f[102] as u64) << 32) |
        ((f[103] as u64) << 48)
    } else {
        (f[60] as u64) | ((f[61] as u64) << 16)
    };
    
    
    let mut model = String::new();
    for a in 27..47 {
        let od = f[a];
        let rw = ((od >> 8) & 0xFF) as u8;
        let tx = (od & 0xFF) as u8;
        if rw > 0x20 && rw < 0x7F { model.push(rw as char); }
        if tx > 0x20 && tx < 0x7F { model.push(tx as char); }
    }
    
    
    let mut serial = String::new();
    for a in 10..20 {
        let od = f[a];
        let rw = ((od >> 8) & 0xFF) as u8;
        let tx = (od & 0xFF) as u8;
        if rw > 0x20 && rw < 0x7F { serial.push(rw as char); }
        if tx > 0x20 && tx < 0x7F { serial.push(tx as char); }
    }
    
    let channel = if ar == PB_ { IdeChannel::Adx } else { IdeChannel::Aeq };
    let qf = if cbv { DrivePosition::Ams } else { DrivePosition::Ake };
    
    Some(Ajc {
        channel,
        qf,
        brs: true,
        gal,
        gle,
        agw,
        model: model.eke().into(),
        serial: serial.eke().into(),
    })
}


pub fn oem() -> bool {
    crate::serial_println!("[IDE] Probing IDE channels...");
    
    let mut bzh = Vec::new();
    
    
    ply(AGP_);
    
    if let Some(cef) = izi(PB_, AGP_, false) {
        crate::serial_println!("[IDE] Primary Master: {} ({} sectors)", 
            cef.model, cef.agw);
        bzh.push(cef);
    }
    
    if let Some(cef) = izi(PB_, AGP_, true) {
        crate::serial_println!("[IDE] Primary Slave: {} ({} sectors)", 
            cef.model, cef.agw);
        bzh.push(cef);
    }
    
    
    ply(AHO_);
    
    if let Some(cef) = izi(WW_, AHO_, false) {
        crate::serial_println!("[IDE] Secondary Master: {} ({} sectors)", 
            cef.model, cef.agw);
        bzh.push(cef);
    }
    
    if let Some(cef) = izi(WW_, AHO_, true) {
        crate::serial_println!("[IDE] Secondary Slave: {} ({} sectors)", 
            cef.model, cef.agw);
        bzh.push(cef);
    }
    
    let oal = !bzh.is_empty();
    
    *Bn.lock() = Some(Bjl {
        bzh,
        jr: oal,
    });
    
    crate::serial_println!("[IDE] Found {} drives", 
        Bn.lock().as_ref().map(|r| r.bzh.len()).unwrap_or(0));
    
    oal
}


fn nnv(channel: IdeChannel, cbv: bool) -> bool {
    let u = if cbv { DrivePosition::Ams } else { DrivePosition::Ake };
    Bn.lock().as_ref()
        .and_then(|r| r.bzh.iter().du(|bc| bc.channel == channel && bc.qf == u))
        .map(|bc| bc.gle)
        .unwrap_or(false)
}


pub fn ain(channel: IdeChannel, cbv: bool, qa: u64, az: u8, bi: &mut [u8]) -> Result<(), &'static str> {
    let ar = match channel {
        IdeChannel::Adx => PB_,
        IdeChannel::Aeq => WW_,
    };
    
    if bi.len() < (az as usize) * 512 {
        return Err("Buffer too small");
    }
    
    
    let jva = nnv(channel, cbv);
    
    mdk(ar, cbv);
    iha(ar)?;
    
    if jva {
        
        unsafe {
            
            Port::<u8>::new(ar + 2).write(0); 
            Port::<u8>::new(ar + 3).write((qa >> 24) as u8); 
            Port::<u8>::new(ar + 4).write((qa >> 32) as u8); 
            Port::<u8>::new(ar + 5).write((qa >> 40) as u8); 
            
            Port::<u8>::new(ar + 2).write(az); 
            Port::<u8>::new(ar + 3).write(qa as u8); 
            Port::<u8>::new(ar + 4).write((qa >> 8) as u8); 
            Port::<u8>::new(ar + 5).write((qa >> 16) as u8); 
            Port::<u8>::new(ar + 6).write(0x40 | (if cbv { 0x10 } else { 0 })); 
            Port::<u8>::new(ar + 7).write(cmd::CNN_);
        }
    } else {
        
        unsafe {
            Port::<u8>::new(ar + 2).write(az);
            Port::<u8>::new(ar + 3).write(qa as u8);
            Port::<u8>::new(ar + 4).write((qa >> 8) as u8);
            Port::<u8>::new(ar + 5).write((qa >> 16) as u8);
            Port::<u8>::new(ar + 6).write(0xE0 | (if cbv { 0x10 } else { 0 }) | ((qa >> 24) as u8 & 0x0F));
            Port::<u8>::new(ar + 7).write(cmd::CNM_);
        }
    }
    
    let mut axr = Port::<u16>::new(ar);
    let mut l = 0;
    
    for _ in 0..az {
        mqd(ar)?;
        
        for _ in 0..256 {
            let od = unsafe { axr.read() };
            bi[l] = (od & 0xFF) as u8;
            bi[l + 1] = ((od >> 8) & 0xFF) as u8;
            l += 2;
        }
    }
    
    Ok(())
}


pub fn bpi(channel: IdeChannel, cbv: bool, qa: u64, az: u8, bi: &[u8]) -> Result<(), &'static str> {
    let ar = match channel {
        IdeChannel::Adx => PB_,
        IdeChannel::Aeq => WW_,
    };
    
    if bi.len() < (az as usize) * 512 {
        return Err("Buffer too small");
    }
    
    let jva = nnv(channel, cbv);
    
    mdk(ar, cbv);
    iha(ar)?;
    
    if jva {
        unsafe {
            
            Port::<u8>::new(ar + 2).write(0); 
            Port::<u8>::new(ar + 3).write((qa >> 24) as u8);
            Port::<u8>::new(ar + 4).write((qa >> 32) as u8);
            Port::<u8>::new(ar + 5).write((qa >> 40) as u8);
            
            Port::<u8>::new(ar + 2).write(az);
            Port::<u8>::new(ar + 3).write(qa as u8);
            Port::<u8>::new(ar + 4).write((qa >> 8) as u8);
            Port::<u8>::new(ar + 5).write((qa >> 16) as u8);
            Port::<u8>::new(ar + 6).write(0x40 | (if cbv { 0x10 } else { 0 }));
            Port::<u8>::new(ar + 7).write(cmd::DBR_);
        }
    } else {
        unsafe {
            Port::<u8>::new(ar + 2).write(az);
            Port::<u8>::new(ar + 3).write(qa as u8);
            Port::<u8>::new(ar + 4).write((qa >> 8) as u8);
            Port::<u8>::new(ar + 5).write((qa >> 16) as u8);
            Port::<u8>::new(ar + 6).write(0xE0 | (if cbv { 0x10 } else { 0 }) | ((qa >> 24) as u8 & 0x0F));
            Port::<u8>::new(ar + 7).write(cmd::DBQ_);
        }
    }
    
    let mut axr = Port::<u16>::new(ar);
    let mut l = 0;
    
    for _ in 0..az {
        mqd(ar)?;
        
        for _ in 0..256 {
            let od = (bi[l] as u16) | ((bi[l + 1] as u16) << 8);
            unsafe { axr.write(od); }
            l += 2;
        }
    }
    
    
    unsafe {
        let hjz = if jva { cmd::BLR_ } else { cmd::BLQ_ };
        Port::<u8>::new(ar + 7).write(hjz);
    }
    iha(ar)?;
    
    Ok(())
}


pub fn jdq() -> Vec<Ajc> {
    Bn.lock().as_ref().map(|r| r.bzh.clone()).age()
}


pub fn ky() -> bool {
    Bn.lock().as_ref().map(|r| r.jr).unwrap_or(false)
}
