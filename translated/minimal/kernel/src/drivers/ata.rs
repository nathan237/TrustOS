




use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;
use crate::arch::Port;


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IdeChannel {
    Primary,
    Secondary,
}


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DrivePosition {
    Master,
    Slave,
}


mod cmd {
    pub const Alf: u8 = 0xEC;
    pub const CEV_: u8 = 0xA1;
    pub const CQV_: u8 = 0x20;
    pub const CQW_: u8 = 0x24;
    pub const DFL_: u8 = 0x30;
    pub const DFM_: u8 = 0x34;
    pub const BOJ_: u8 = 0xE7;
    pub const BOK_: u8 = 0xEA;
}


mod status {
    pub const Yf: u8 = 1 << 0;
    pub const Aiq: u8 = 1 << 3;
    pub const Aig: u8 = 1 << 5;
    pub const Wq: u8 = 1 << 7;
}


const PZ_: u16 = 0x1F0;
const AIJ_: u16 = 0x3F6;
const YD_: u16 = 0x170;
const AJL_: u16 = 0x376;


#[derive(Clone, Debug)]
pub struct Pd {
    pub channel: IdeChannel,
    pub position: DrivePosition,
    pub present: bool,
    pub atapi: bool,
    pub lba48: bool,
    pub sector_count: u64,
    pub model: String,
    pub serial: String,
}


pub struct Aac {
    pub drives: Vec<Pd>,
    pub initialized: bool,
}

static Ao: Mutex<Option<Aac>> = Mutex::new(None);


fn eeb(base: u16) -> Result<(), &'static str> {
    let mut bjk = Port::<u8>::new(base + 7);
    
    for _ in 0..100_000 {
        let status = unsafe { bjk.read() };
        
        if status == 0xFF {
            return Err("No drive");
        }
        
        if status & status::Wq == 0 {
            if status & status::Yf != 0 {
                return Err("Drive error");
            }
            if status & status::Aig != 0 {
                return Err("Drive fault");
            }
            return Ok(());
        }
        
        core::hint::spin_loop();
    }
    
    Err("Timeout")
}


fn hca(base: u16) -> Result<(), &'static str> {
    let mut bjk = Port::<u8>::new(base + 7);
    
    for _ in 0..100_000 {
        let status = unsafe { bjk.read() };
        
        if status & status::Yf != 0 {
            return Err("Drive error");
        }
        
        if status & status::Wq == 0 && status & status::Aiq != 0 {
            return Ok(());
        }
        
        core::hint::spin_loop();
    }
    
    Err("Timeout waiting for DRQ")
}


fn jha(control_base: u16) {
    let mut control = Port::<u8>::new(control_base);
    unsafe {
        control.write(0x04);
        for _ in 0..1000 { core::hint::spin_loop(); }
        control.write(0x00);
        for _ in 0..10000 { core::hint::spin_loop(); }
    }
}


fn gtr(base: u16, slave: bool) {
    let mut llk = Port::<u8>::new(base + 6);
    unsafe {
        llk.write(if slave { 0xB0 } else { 0xA0 });
        for _ in 0..4 {
            let _ = Port::<u8>::new(base + 7).read();
        }
    }
}


fn eqb(base: u16, _control: u16, slave: bool) -> Option<Pd> {
    gtr(base, slave);
    
    
    unsafe {
        Port::<u8>::new(base + 2).write(0);
        Port::<u8>::new(base + 3).write(0);
        Port::<u8>::new(base + 4).write(0);
        Port::<u8>::new(base + 5).write(0);
    }
    
    
    unsafe {
        Port::<u8>::new(base + 7).write(cmd::Alf);
    }
    
    
    let status = unsafe { Port::<u8>::new(base + 7).read() };
    if status == 0 {
        return None;
    }
    
    
    let mut atapi = false;
    if eeb(base).is_err() {
        let mxm = unsafe { Port::<u8>::new(base + 4).read() };
        let mxl = unsafe { Port::<u8>::new(base + 5).read() };
        
        if mxm == 0x14 && mxl == 0xEB {
            atapi = true;
            unsafe {
                Port::<u8>::new(base + 7).write(cmd::CEV_);
            }
            if eeb(base).is_err() {
                return None;
            }
        } else {
            return None;
        }
    }
    
    
    if hca(base).is_err() {
        return None;
    }
    
    
    let mut data = [0u16; 256];
    let mut zu = Port::<u16>::new(base);
    for i in 0..256 {
        data[i] = unsafe { zu.read() };
    }
    
    
    let lba48 = (data[83] & (1 << 10)) != 0;
    
    let sector_count = if lba48 {
        (data[100] as u64) |
        ((data[101] as u64) << 16) |
        ((data[102] as u64) << 32) |
        ((data[103] as u64) << 48)
    } else {
        (data[60] as u64) | ((data[61] as u64) << 16)
    };
    
    
    let mut model = String::new();
    for i in 27..47 {
        let fx = data[i];
        let hw = ((fx >> 8) & 0xFF) as u8;
        let jf = (fx & 0xFF) as u8;
        if hw > 0x20 && hw < 0x7F { model.push(hw as char); }
        if jf > 0x20 && jf < 0x7F { model.push(jf as char); }
    }
    
    
    let mut serial = String::new();
    for i in 10..20 {
        let fx = data[i];
        let hw = ((fx >> 8) & 0xFF) as u8;
        let jf = (fx & 0xFF) as u8;
        if hw > 0x20 && hw < 0x7F { serial.push(hw as char); }
        if jf > 0x20 && jf < 0x7F { serial.push(jf as char); }
    }
    
    let channel = if base == PZ_ { IdeChannel::Primary } else { IdeChannel::Secondary };
    let position = if slave { DrivePosition::Slave } else { DrivePosition::Master };
    
    Some(Pd {
        channel,
        position,
        present: true,
        atapi,
        lba48,
        sector_count,
        model: model.trim_end().into(),
        serial: serial.trim_end().into(),
    })
}


pub fn igt() -> bool {
    crate::serial_println!("[IDE] Probing IDE channels...");
    
    let mut drives = Vec::new();
    
    
    jha(AIJ_);
    
    if let Some(drive) = eqb(PZ_, AIJ_, false) {
        crate::serial_println!("[IDE] Primary Master: {} ({} sectors)", 
            drive.model, drive.sector_count);
        drives.push(drive);
    }
    
    if let Some(drive) = eqb(PZ_, AIJ_, true) {
        crate::serial_println!("[IDE] Primary Slave: {} ({} sectors)", 
            drive.model, drive.sector_count);
        drives.push(drive);
    }
    
    
    jha(AJL_);
    
    if let Some(drive) = eqb(YD_, AJL_, false) {
        crate::serial_println!("[IDE] Secondary Master: {} ({} sectors)", 
            drive.model, drive.sector_count);
        drives.push(drive);
    }
    
    if let Some(drive) = eqb(YD_, AJL_, true) {
        crate::serial_println!("[IDE] Secondary Slave: {} ({} sectors)", 
            drive.model, drive.sector_count);
        drives.push(drive);
    }
    
    let idn = !drives.is_empty();
    
    *Ao.lock() = Some(Aac {
        drives,
        initialized: idn,
    });
    
    crate::serial_println!("[IDE] Found {} drives", 
        Ao.lock().as_ref().map(|c| c.drives.len()).unwrap_or(0));
    
    idn
}


fn htt(channel: IdeChannel, slave: bool) -> bool {
    let pos = if slave { DrivePosition::Slave } else { DrivePosition::Master };
    Ao.lock().as_ref()
        .and_then(|c| c.drives.iter().find(|d| d.channel == channel && d.position == pos))
        .map(|d| d.lba48)
        .unwrap_or(false)
}


pub fn read_sectors(channel: IdeChannel, slave: bool, hb: u64, count: u8, buffer: &mut [u8]) -> Result<(), &'static str> {
    let base = match channel {
        IdeChannel::Primary => PZ_,
        IdeChannel::Secondary => YD_,
    };
    
    if buffer.len() < (count as usize) * 512 {
        return Err("Buffer too small");
    }
    
    
    let fed = htt(channel, slave);
    
    gtr(base, slave);
    eeb(base)?;
    
    if fed {
        
        unsafe {
            
            Port::<u8>::new(base + 2).write(0); 
            Port::<u8>::new(base + 3).write((hb >> 24) as u8); 
            Port::<u8>::new(base + 4).write((hb >> 32) as u8); 
            Port::<u8>::new(base + 5).write((hb >> 40) as u8); 
            
            Port::<u8>::new(base + 2).write(count); 
            Port::<u8>::new(base + 3).write(hb as u8); 
            Port::<u8>::new(base + 4).write((hb >> 8) as u8); 
            Port::<u8>::new(base + 5).write((hb >> 16) as u8); 
            Port::<u8>::new(base + 6).write(0x40 | (if slave { 0x10 } else { 0 })); 
            Port::<u8>::new(base + 7).write(cmd::CQW_);
        }
    } else {
        
        unsafe {
            Port::<u8>::new(base + 2).write(count);
            Port::<u8>::new(base + 3).write(hb as u8);
            Port::<u8>::new(base + 4).write((hb >> 8) as u8);
            Port::<u8>::new(base + 5).write((hb >> 16) as u8);
            Port::<u8>::new(base + 6).write(0xE0 | (if slave { 0x10 } else { 0 }) | ((hb >> 24) as u8 & 0x0F));
            Port::<u8>::new(base + 7).write(cmd::CQV_);
        }
    }
    
    let mut zu = Port::<u16>::new(base);
    let mut offset = 0;
    
    for _ in 0..count {
        hca(base)?;
        
        for _ in 0..256 {
            let fx = unsafe { zu.read() };
            buffer[offset] = (fx & 0xFF) as u8;
            buffer[offset + 1] = ((fx >> 8) & 0xFF) as u8;
            offset += 2;
        }
    }
    
    Ok(())
}


pub fn write_sectors(channel: IdeChannel, slave: bool, hb: u64, count: u8, buffer: &[u8]) -> Result<(), &'static str> {
    let base = match channel {
        IdeChannel::Primary => PZ_,
        IdeChannel::Secondary => YD_,
    };
    
    if buffer.len() < (count as usize) * 512 {
        return Err("Buffer too small");
    }
    
    let fed = htt(channel, slave);
    
    gtr(base, slave);
    eeb(base)?;
    
    if fed {
        unsafe {
            
            Port::<u8>::new(base + 2).write(0); 
            Port::<u8>::new(base + 3).write((hb >> 24) as u8);
            Port::<u8>::new(base + 4).write((hb >> 32) as u8);
            Port::<u8>::new(base + 5).write((hb >> 40) as u8);
            
            Port::<u8>::new(base + 2).write(count);
            Port::<u8>::new(base + 3).write(hb as u8);
            Port::<u8>::new(base + 4).write((hb >> 8) as u8);
            Port::<u8>::new(base + 5).write((hb >> 16) as u8);
            Port::<u8>::new(base + 6).write(0x40 | (if slave { 0x10 } else { 0 }));
            Port::<u8>::new(base + 7).write(cmd::DFM_);
        }
    } else {
        unsafe {
            Port::<u8>::new(base + 2).write(count);
            Port::<u8>::new(base + 3).write(hb as u8);
            Port::<u8>::new(base + 4).write((hb >> 8) as u8);
            Port::<u8>::new(base + 5).write((hb >> 16) as u8);
            Port::<u8>::new(base + 6).write(0xE0 | (if slave { 0x10 } else { 0 }) | ((hb >> 24) as u8 & 0x0F));
            Port::<u8>::new(base + 7).write(cmd::DFL_);
        }
    }
    
    let mut zu = Port::<u16>::new(base);
    let mut offset = 0;
    
    for _ in 0..count {
        hca(base)?;
        
        for _ in 0..256 {
            let fx = (buffer[offset] as u16) | ((buffer[offset + 1] as u16) << 8);
            unsafe { zu.write(fx); }
            offset += 2;
        }
    }
    
    
    unsafe {
        let dpz = if fed { cmd::BOK_ } else { cmd::BOJ_ };
        Port::<u8>::new(base + 7).write(dpz);
    }
    eeb(base)?;
    
    Ok(())
}


pub fn eta() -> Vec<Pd> {
    Ao.lock().as_ref().map(|c| c.drives.clone()).unwrap_or_default()
}


pub fn is_initialized() -> bool {
    Ao.lock().as_ref().map(|c| c.initialized).unwrap_or(false)
}
