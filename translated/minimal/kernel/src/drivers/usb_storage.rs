







use alloc::vec::Vec;
use alloc::vec;
use alloc::string::String;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use spin::Mutex;

use crate::vfs::fat32::Ak;






const DDK_: u8 = 0x08;

const DDS_: u8 = 0x06;

const DDO_: u8 = 0x50;


const BOP_: u32 = 0x43425355; 

const BTA_: u32 = 0x53425355; 


const APW_: u8 = 0x00;
const BOO_: u8 = 0x80;


const KN_: u8 = 0x00;
const DLU_: u8 = 0x01;
const DLV_: u8 = 0x02;


const CUN_: u8 = 0x00;
const CUM_: u8 = 0x03;
const CUJ_: u8 = 0x12;
const CUL_: u8 = 0x25;
const CUK_: u8 = 0x28;
const CUO_: u8 = 0x2A;


const H_: usize = 512;


const BCE_: usize = 128; 





#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Cbw {
    signature: u32,        
    tag: u32,              
    data_transfer_length: u32, 
    flags: u8,             
    lun: u8,               
    cb_length: u8,         
    cb: [u8; 16],          
}

impl Cbw {
    fn new(tag: u32, bjr: u32, it: u8, lun: u8, cmd: &[u8]) -> Self {
        let mut cb = [0u8; 16];
        let len = cmd.len().min(16);
        cb[..len].copy_from_slice(&cmd[..len]);
        Self {
            signature: BOP_,
            tag,
            data_transfer_length: bjr,
            flags: it,
            lun,
            cb_length: len as u8,
            cb,
        }
    }

    fn as_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(self as *const Self as *const u8, 31)
        }
    }
}





#[repr(C, packed)]
#[derive(Clone, Copy, Default)]
struct Csw {
    signature: u32,      
    tag: u32,            
    data_residue: u32,   
    status: u8,          
}

impl Csw {
    fn bsv(data: &[u8]) -> Option<Self> {
        if data.len() < 13 { return None; }
        let sig = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        if sig != BTA_ { return None; }
        Some(Self {
            signature: sig,
            tag: u32::from_le_bytes([data[4], data[5], data[6], data[7]]),
            data_residue: u32::from_le_bytes([data[8], data[9], data[10], data[11]]),
            status: data[12],
        })
    }
}






#[derive(Clone)]
pub struct Gc {
    pub slot_id: u8,
    pub lun: u8,
    pub bulk_in_dci: u8,     
    pub bulk_out_dci: u8,    
    pub max_packet_in: u16,
    pub max_packet_out: u16,
    pub block_count: u64,
    pub block_size: u32,
    pub vendor: String,
    pub product: String,
    pub ready: bool,
}


static GI_: Mutex<Vec<Gc>> = Mutex::new(Vec::new());
static DAZ_: AtomicU32 = AtomicU32::new(1);
static Ah: AtomicBool = AtomicBool::new(false);

fn nkh() -> u32 {
    DAZ_.fetch_add(1, Ordering::Relaxed)
}






fn bulk_out(slot_id: u8, ahu: u8, data: &[u8]) -> bool {
    
    let hg = match crate::memory::frame::aan() {
        Some(aa) => aa,
        None => return false,
    };
    let kt = super::xhci::wk(hg) as *mut u8;
    
    let len = data.len().min(4096);
    unsafe {
        core::ptr::copy_nonoverlapping(data.as_ptr(), kt, len);
    }

    let success = super::xhci::kgd(slot_id, ahu, hg, len as u32);
    crate::memory::frame::vk(hg);
    success
}


fn bulk_in(slot_id: u8, ahu: u8, buffer: &mut [u8], length: u32) -> Option<u32> {
    let hg = match crate::memory::frame::aan() {
        Some(aa) => aa,
        None => return None,
    };
    let kt = super::xhci::wk(hg) as *mut u8;

    let result = super::xhci::kgc(slot_id, ahu, hg, length);
    if let Some(transferred) = result {
        let mb = (transferred as usize).min(buffer.len());
        unsafe {
            core::ptr::copy_nonoverlapping(kt, buffer.as_mut_ptr(), mb);
        }
    }
    
    crate::memory::frame::vk(hg);
    result
}


fn kfy(slot_id: u8, ahu: u8, buffer: &mut [u8], length: u32) -> Option<u32> {
    let mut offset = 0usize;
    let mut ck = length;
    
    while ck > 0 {
        let df = ck.min(4096);
        let end = offset + df as usize;
        if end > buffer.len() { break; }

        match bulk_in(slot_id, ahu, &mut buffer[offset..end], df) {
            Some(transferred) => {
                offset += transferred as usize;
                ck -= df;
            }
            None => return None,
        }
    }
    Some(offset as u32)
}


fn kga(slot_id: u8, ahu: u8, data: &[u8]) -> bool {
    let mut offset = 0usize;
    while offset < data.len() {
        let end = (offset + 4096).min(data.len());
        if !bulk_out(slot_id, ahu, &data[offset..end]) {
            return false;
        }
        offset = end;
    }
    true
}







fn ddy(
    s: &Gc,
    cmd: &[u8],
    data_in: Option<&mut [u8]>,
    data_out: Option<&[u8]>,
) -> Result<u8, &'static str> {
    let tag = nkh();
    let direction;
    let bjr;
    
    if let Some(ref buf) = data_in {
        direction = BOO_;
        bjr = buf.len() as u32;
    } else if let Some(ref buf) = data_out {
        direction = APW_;
        bjr = buf.len() as u32;
    } else {
        direction = APW_;
        bjr = 0;
    }
    
    
    let kht = Cbw::new(tag, bjr, direction, s.lun, cmd);
    if !bulk_out(s.slot_id, s.bulk_out_dci, kht.as_bytes()) {
        return Err("CBW send failed");
    }
    
    
    if let Some(buf) = data_in {
        if bjr > 4096 {
            kfy(s.slot_id, s.bulk_in_dci, buf, bjr)
                .ok_or("Data IN failed")?;
        } else {
            bulk_in(s.slot_id, s.bulk_in_dci, buf, bjr)
                .ok_or("Data IN failed")?;
        }
    } else if let Some(buf) = data_out {
        if !kga(s.slot_id, s.bulk_out_dci, buf) {
            return Err("Data OUT failed");
        }
    }
    
    
    let mut hpd = [0u8; 13];
    bulk_in(s.slot_id, s.bulk_in_dci, &mut hpd, 13)
        .ok_or("CSW receive failed")?;
    
    let hpc = Csw::bsv(&hpd).ok_or("Invalid CSW")?;
    if hpc.tag != tag {
        return Err("CSW tag mismatch");
    }
    
    Ok(hpc.status)
}






fn olz(s: &mut Gc) -> bool {
    let cmd = [CUJ_, 0, 0, 0, 36, 0]; 
    let mut buf = [0u8; 36];
    
    match ddy(s, &cmd, Some(&mut buf), None) {
        Ok(KN_) => {
            
            let vendor = core::str::from_utf8(&buf[8..16])
                .unwrap_or("Unknown")
                .trim()
                .into();
            let product = core::str::from_utf8(&buf[16..32])
                .unwrap_or("Unknown")
                .trim()
                .into();
            s.vendor = vendor;
            s.product = product;
            crate::serial_println!("[USB-MS] INQUIRY: {} {}", s.vendor, s.product);
            true
        }
        Ok(status) => {
            crate::serial_println!("[USB-MS] INQUIRY failed: status={}", status);
            false
        }
        Err(e) => {
            crate::serial_println!("[USB-MS] INQUIRY error: {}", e);
            false
        }
    }
}


fn omc(s: &Gc) -> bool {
    let cmd = [CUN_, 0, 0, 0, 0, 0];
    matches!(ddy(s, &cmd, None, None), Ok(KN_))
}


fn oma(s: &mut Gc) -> bool {
    let cmd = [CUL_, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut buf = [0u8; 8];
    
    match ddy(s, &cmd, Some(&mut buf), None) {
        Ok(KN_) => {
            let mwo = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
            let block_size = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);
            s.block_count = mwo as u64 + 1;
            s.block_size = block_size;
            crate::serial_println!("[USB-MS] Capacity: {} blocks × {} bytes = {} MB",
                s.block_count, s.block_size,
                (s.block_count * s.block_size as u64) / (1024 * 1024));
            true
        }
        Ok(status) => {
            crate::serial_println!("[USB-MS] READ CAPACITY failed: status={}", status);
            false
        }
        Err(e) => {
            crate::serial_println!("[USB-MS] READ CAPACITY error: {}", e);
            false
        }
    }
}


fn jdr(s: &Gc, hb: u32, count: u16, buffer: &mut [u8]) -> bool {
    let cmd = [
        CUK_,
        0,
        (hb >> 24) as u8,
        (hb >> 16) as u8,
        (hb >> 8) as u8,
        hb as u8,
        0, 
        (count >> 8) as u8,
        count as u8,
        0, 
    ];
    
    matches!(ddy(s, &cmd, Some(buffer), None), Ok(KN_))
}


fn jds(s: &Gc, hb: u32, count: u16, buffer: &[u8]) -> bool {
    let cmd = [
        CUO_,
        0,
        (hb >> 24) as u8,
        (hb >> 16) as u8,
        (hb >> 8) as u8,
        hb as u8,
        0,
        (count >> 8) as u8,
        count as u8,
        0,
    ];
    
    matches!(ddy(s, &cmd, None, Some(buffer)), Ok(KN_))
}


fn omb(s: &Gc) -> Option<(u8, u8, u8)> {
    let cmd = [CUM_, 0, 0, 0, 18, 0];
    let mut buf = [0u8; 18];
    
    match ddy(s, &cmd, Some(&mut buf), None) {
        Ok(KN_) => {
            let oof = buf[2] & 0x0F;
            let fho = buf[12];
            let fhp = buf[13];
            Some((oof, fho, fhp))
        }
        _ => None,
    }
}






pub fn iib(class: u8, subclass: u8, protocol: u8) -> bool {
    class == DDK_ 
        && subclass == DDS_ 
        && protocol == DDO_
}



pub fn mpb(
    slot_id: u8,
    bulk_in_ep: u8,    
    bulk_out_ep: u8,   
    max_packet_in: u16,
    max_packet_out: u16,
) {
    let kfz = bulk_in_ep & 0x0F;
    let kgb = bulk_out_ep & 0x0F;
    let bulk_in_dci = kfz * 2 + 1;   
    let bulk_out_dci = kgb * 2;      
    
    crate::serial_println!("[USB-MS] Initializing mass storage: slot {} IN_DCI={} OUT_DCI={}",
        slot_id, bulk_in_dci, bulk_out_dci);
    
    let mut s = Gc {
        slot_id,
        lun: 0,
        bulk_in_dci,
        bulk_out_dci,
        max_packet_in,
        max_packet_out,
        block_count: 0,
        block_size: H_ as u32,
        vendor: String::new(),
        product: String::new(),
        ready: false,
    };
    
    
    olz(&mut s);
    
    
    for attempt in 0..5 {
        if omc(&s) {
            break;
        }
        
        if let Some((gvg, fho, fhp)) = omb(&s) {
            crate::serial_println!("[USB-MS] Sense: key={:#x} ASC={:#x} ASCQ={:#x}", gvg, fho, fhp);
        }
        if attempt < 4 {
            
            for _ in 0..100_000 { core::hint::spin_loop(); }
        }
    }
    
    
    if oma(&mut s) {
        s.ready = true;
    }
    
    GI_.lock().push(s);
    Ah.store(true, Ordering::Release);
    
    crate::serial_println!("[USB-MS] Mass storage device ready");
}






pub struct UsbBlockDevice {
    device_index: usize,
}

impl UsbBlockDevice {
    pub fn new(device_index: usize) -> Self {
        Self { device_index }
    }
}

impl Ak for UsbBlockDevice {
    fn read_sector(&self, dj: u64, buffer: &mut [u8]) -> Result<(), ()> {
        let devices = GI_.lock();
        let s = devices.get(self.device_index).ok_or(())?;
        if !s.ready { return Err(()); }
        
        let aov = s.block_size as usize;
        if buffer.len() < aov { return Err(()); }
        
        if jdr(s, dj as u32, 1, &mut buffer[..aov]) {
            Ok(())
        } else {
            Err(())
        }
    }
    
    fn write_sector(&self, dj: u64, buffer: &[u8]) -> Result<(), ()> {
        let devices = GI_.lock();
        let s = devices.get(self.device_index).ok_or(())?;
        if !s.ready { return Err(()); }
        
        let aov = s.block_size as usize;
        if buffer.len() < aov { return Err(()); }
        
        if jds(s, dj as u32, 1, &buffer[..aov]) {
            Ok(())
        } else {
            Err(())
        }
    }
    
    fn sector_size(&self) -> usize {
        let devices = GI_.lock();
        devices.get(self.device_index)
            .map(|s| s.block_size as usize)
            .unwrap_or(H_)
    }
}






pub fn sw() -> bool {
    Ah.load(Ordering::Acquire)
}


pub fn aqg() -> usize {
    GI_.lock().len()
}


pub fn adz() -> Vec<(String, u64, u32)> {
    GI_.lock().iter().map(|s| {
        let name = if s.vendor.is_empty() && s.product.is_empty() {
            alloc::format!("USB Storage (slot {})", s.slot_id)
        } else {
            alloc::format!("{} {}", s.vendor, s.product)
        };
        (name, s.block_count, s.block_size)
    }).collect()
}


pub fn read_sectors(device_index: usize, start_lba: u64, count: usize, buffer: &mut [u8]) -> Result<(), &'static str> {
    let devices = GI_.lock();
    let s = devices.get(device_index).ok_or("Invalid device index")?;
    if !s.ready { return Err("Device not ready"); }
    
    let block_size = s.block_size as usize;
    if buffer.len() < count * block_size {
        return Err("Buffer too small");
    }
    
    
    let mut hb = start_lba as u32;
    let mut offset = 0;
    let mut ck = count;
    
    while ck > 0 {
        let df = ck.min(BCE_);
        let nb = df * block_size;
        
        if !jdr(s, hb, df as u16, &mut buffer[offset..offset + nb]) {
            return Err("SCSI READ failed");
        }
        
        hb += df as u32;
        offset += nb;
        ck -= df;
    }
    
    Ok(())
}


pub fn write_sectors(device_index: usize, start_lba: u64, count: usize, buffer: &[u8]) -> Result<(), &'static str> {
    let devices = GI_.lock();
    let s = devices.get(device_index).ok_or("Invalid device index")?;
    if !s.ready { return Err("Device not ready"); }
    
    let block_size = s.block_size as usize;
    if buffer.len() < count * block_size {
        return Err("Buffer too small");
    }
    
    let mut hb = start_lba as u32;
    let mut offset = 0;
    let mut ck = count;
    
    while ck > 0 {
        let df = ck.min(BCE_);
        let nb = df * block_size;
        
        if !jds(s, hb, df as u16, &buffer[offset..offset + nb]) {
            return Err("SCSI WRITE failed");
        }
        
        hb += df as u32;
        offset += nb;
        ck -= df;
    }
    
    Ok(())
}


pub fn ibh(device_index: usize) -> Option<UsbBlockDevice> {
    let devices = GI_.lock();
    if device_index < devices.len() && devices[device_index].ready {
        Some(UsbBlockDevice::new(device_index))
    } else {
        None
    }
}
