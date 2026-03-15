







use alloc::vec::Vec;
use alloc::vec;
use alloc::string::String;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use spin::Mutex;

use crate::vfs::fat32::Bj;






const CZS_: u8 = 0x08;

const DAA_: u8 = 0x06;

const CZW_: u8 = 0x50;


const BLW_: u32 = 0x43425355; 

const BQH_: u32 = 0x53425355; 


const ANS_: u8 = 0x00;
const BLV_: u8 = 0x80;


const JT_: u8 = 0x00;
const DIH_: u8 = 0x01;
const DII_: u8 = 0x02;


const CQW_: u8 = 0x00;
const CQV_: u8 = 0x03;
const CQS_: u8 = 0x12;
const CQU_: u8 = 0x25;
const CQT_: u8 = 0x28;
const CQX_: u8 = 0x2A;


const H_: usize = 512;


const BAC_: usize = 128; 





#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Cbw {
    signature: u32,        
    ll: u32,              
    rtr: u32, 
    flags: u8,             
    hqn: u8,               
    qwp: u8,         
    aiv: [u8; 16],          
}

impl Cbw {
    fn new(ll: u32, dmq: u32, te: u8, hqn: u8, cmd: &[u8]) -> Self {
        let mut aiv = [0u8; 16];
        let len = cmd.len().v(16);
        aiv[..len].dg(&cmd[..len]);
        Self {
            signature: BLW_,
            ll,
            rtr: dmq,
            flags: te,
            hqn,
            qwp: len as u8,
            aiv,
        }
    }

    fn as_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::anh(self as *const Self as *const u8, 31)
        }
    }
}





#[repr(C, packed)]
#[derive(Clone, Copy, Default)]
struct Csw {
    signature: u32,      
    ll: u32,            
    rtn: u32,   
    status: u8,          
}

impl Csw {
    fn eca(f: &[u8]) -> Option<Self> {
        if f.len() < 13 { return None; }
        let sig = u32::dj([f[0], f[1], f[2], f[3]]);
        if sig != BQH_ { return None; }
        Some(Self {
            signature: sig,
            ll: u32::dj([f[4], f[5], f[6], f[7]]),
            rtn: u32::dj([f[8], f[9], f[10], f[11]]),
            status: f[12],
        })
    }
}






#[derive(Clone)]
pub struct Om {
    pub fw: u8,
    pub hqn: u8,
    pub dzi: u8,     
    pub enc: u8,    
    pub hrd: u16,
    pub hre: u16,
    pub hat: u64,
    pub py: u32,
    pub acs: String,
    pub baj: String,
    pub ack: bool,
}


static FT_: Mutex<Vec<Om>> = Mutex::new(Vec::new());
static CXH_: AtomicU32 = AtomicU32::new(1);
static Be: AtomicBool = AtomicBool::new(false);

fn uuo() -> u32 {
    CXH_.fetch_add(1, Ordering::Relaxed)
}






fn gbu(fw: u8, bms: u8, f: &[u8]) -> bool {
    
    let rg = match crate::memory::frame::azg() {
        Some(ai) => ai,
        None => return false,
    };
    let aak = super::xhci::auv(rg) as *mut u8;
    
    let len = f.len().v(4096);
    unsafe {
        core::ptr::copy_nonoverlapping(f.fq(), aak, len);
    }

    let vx = super::xhci::quo(fw, bms, rg, len as u32);
    crate::memory::frame::apt(rg);
    vx
}


fn fea(fw: u8, bms: u8, bi: &mut [u8], go: u32) -> Option<u32> {
    let rg = match crate::memory::frame::azg() {
        Some(ai) => ai,
        None => return None,
    };
    let aak = super::xhci::auv(rg) as *mut u8;

    let result = super::xhci::qun(fw, bms, rg, go);
    if let Some(ieu) = result {
        let zg = (ieu as usize).v(bi.len());
        unsafe {
            core::ptr::copy_nonoverlapping(aak, bi.mw(), zg);
        }
    }
    
    crate::memory::frame::apt(rg);
    result
}


fn qui(fw: u8, bms: u8, bi: &mut [u8], go: u32) -> Option<u32> {
    let mut l = 0usize;
    let mut ia = go;
    
    while ia > 0 {
        let jj = ia.v(4096);
        let ci = l + jj as usize;
        if ci > bi.len() { break; }

        match fea(fw, bms, &mut bi[l..ci], jj) {
            Some(ieu) => {
                l += ieu as usize;
                ia -= jj;
            }
            None => return None,
        }
    }
    Some(l as u32)
}


fn qul(fw: u8, bms: u8, f: &[u8]) -> bool {
    let mut l = 0usize;
    while l < f.len() {
        let ci = (l + 4096).v(f.len());
        if !gbu(fw, bms, &f[l..ci]) {
            return false;
        }
        l = ci;
    }
    true
}







fn grp(
    ba: &Om,
    cmd: &[u8],
    njn: Option<&mut [u8]>,
    njo: Option<&[u8]>,
) -> Result<u8, &'static str> {
    let ll = uuo();
    let sz;
    let dmq;
    
    if let Some(ref k) = njn {
        sz = BLV_;
        dmq = k.len() as u32;
    } else if let Some(ref k) = njo {
        sz = ANS_;
        dmq = k.len() as u32;
    } else {
        sz = ANS_;
        dmq = 0;
    }
    
    
    let qxa = Cbw::new(ll, dmq, sz, ba.hqn, cmd);
    if !gbu(ba.fw, ba.enc, qxa.as_bytes()) {
        return Err("CBW send failed");
    }
    
    
    if let Some(k) = njn {
        if dmq > 4096 {
            qui(ba.fw, ba.dzi, k, dmq)
                .ok_or("Data IN failed")?;
        } else {
            fea(ba.fw, ba.dzi, k, dmq)
                .ok_or("Data IN failed")?;
        }
    } else if let Some(k) = njo {
        if !qul(ba.fw, ba.enc, k) {
            return Err("Data OUT failed");
        }
    }
    
    
    let mut nhs = [0u8; 13];
    fea(ba.fw, ba.dzi, &mut nhs, 13)
        .ok_or("CSW receive failed")?;
    
    let nhr = Csw::eca(&nhs).ok_or("Invalid CSW")?;
    if nhr.ll != ll {
        return Err("CSW tag mismatch");
    }
    
    Ok(nhr.status)
}






fn wex(ba: &mut Om) -> bool {
    let cmd = [CQS_, 0, 0, 0, 36, 0]; 
    let mut k = [0u8; 36];
    
    match grp(ba, &cmd, Some(&mut k), None) {
        Ok(JT_) => {
            
            let acs = core::str::jg(&k[8..16])
                .unwrap_or("Unknown")
                .em()
                .into();
            let baj = core::str::jg(&k[16..32])
                .unwrap_or("Unknown")
                .em()
                .into();
            ba.acs = acs;
            ba.baj = baj;
            crate::serial_println!("[USB-MS] INQUIRY: {} {}", ba.acs, ba.baj);
            true
        }
        Ok(status) => {
            crate::serial_println!("[USB-MS] INQUIRY failed: status={}", status);
            false
        }
        Err(aa) => {
            crate::serial_println!("[USB-MS] INQUIRY error: {}", aa);
            false
        }
    }
}


fn wfa(ba: &Om) -> bool {
    let cmd = [CQW_, 0, 0, 0, 0, 0];
    oh!(grp(ba, &cmd, None, None), Ok(JT_))
}


fn wey(ba: &mut Om) -> bool {
    let cmd = [CQU_, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut k = [0u8; 8];
    
    match grp(ba, &cmd, Some(&mut k), None) {
        Ok(JT_) => {
            let ucj = u32::oa([k[0], k[1], k[2], k[3]]);
            let py = u32::oa([k[4], k[5], k[6], k[7]]);
            ba.hat = ucj as u64 + 1;
            ba.py = py;
            crate::serial_println!("[USB-MS] Capacity: {} blocks × {} bytes = {} MB",
                ba.hat, ba.py,
                (ba.hat * ba.py as u64) / (1024 * 1024));
            true
        }
        Ok(status) => {
            crate::serial_println!("[USB-MS] READ CAPACITY failed: status={}", status);
            false
        }
        Err(aa) => {
            crate::serial_println!("[USB-MS] READ CAPACITY error: {}", aa);
            false
        }
    }
}


fn pgv(ba: &Om, qa: u32, az: u16, bi: &mut [u8]) -> bool {
    let cmd = [
        CQT_,
        0,
        (qa >> 24) as u8,
        (qa >> 16) as u8,
        (qa >> 8) as u8,
        qa as u8,
        0, 
        (az >> 8) as u8,
        az as u8,
        0, 
    ];
    
    oh!(grp(ba, &cmd, Some(bi), None), Ok(JT_))
}


fn pgw(ba: &Om, qa: u32, az: u16, bi: &[u8]) -> bool {
    let cmd = [
        CQX_,
        0,
        (qa >> 24) as u8,
        (qa >> 16) as u8,
        (qa >> 8) as u8,
        qa as u8,
        0,
        (az >> 8) as u8,
        az as u8,
        0,
    ];
    
    oh!(grp(ba, &cmd, None, Some(bi)), Ok(JT_))
}


fn wez(ba: &Om) -> Option<(u8, u8, u8)> {
    let cmd = [CQV_, 0, 0, 0, 18, 0];
    let mut k = [0u8; 18];
    
    match grp(ba, &cmd, Some(&mut k), None) {
        Ok(JT_) => {
            let whs = k[2] & 0x0F;
            let kbf = k[12];
            let kbg = k[13];
            Some((whs, kbf, kbg))
        }
        _ => None,
    }
}






pub fn ogh(class: u8, adl: u8, protocol: u8) -> bool {
    class == CZS_ 
        && adl == DAA_ 
        && protocol == CZW_
}



pub fn ttj(
    fw: u8,
    quh: u8,    
    quk: u8,   
    hrd: u16,
    hre: u16,
) {
    let quj = quh & 0x0F;
    let qum = quk & 0x0F;
    let dzi = quj * 2 + 1;   
    let enc = qum * 2;      
    
    crate::serial_println!("[USB-MS] Initializing mass storage: slot {} IN_DCI={} OUT_DCI={}",
        fw, dzi, enc);
    
    let mut ba = Om {
        fw,
        hqn: 0,
        dzi,
        enc,
        hrd,
        hre,
        hat: 0,
        py: H_ as u32,
        acs: String::new(),
        baj: String::new(),
        ack: false,
    };
    
    
    wex(&mut ba);
    
    
    for kbj in 0..5 {
        if wfa(&ba) {
            break;
        }
        
        if let Some((mfz, kbf, kbg)) = wez(&ba) {
            crate::serial_println!("[USB-MS] Sense: key={:#x} ASC={:#x} ASCQ={:#x}", mfz, kbf, kbg);
        }
        if kbj < 4 {
            
            for _ in 0..100_000 { core::hint::hc(); }
        }
    }
    
    
    if wey(&mut ba) {
        ba.ack = true;
    }
    
    FT_.lock().push(ba);
    Be.store(true, Ordering::Release);
    
    crate::serial_println!("[USB-MS] Mass storage device ready");
}






pub struct UsbBlockDevice {
    cpy: usize,
}

impl UsbBlockDevice {
    pub fn new(cpy: usize) -> Self {
        Self { cpy }
    }
}

impl Bj for UsbBlockDevice {
    fn xr(&self, jk: u64, bi: &mut [u8]) -> Result<(), ()> {
        let ik = FT_.lock();
        let ba = ik.get(self.cpy).ok_or(())?;
        if !ba.ack { return Err(()); }
        
        let cbj = ba.py as usize;
        if bi.len() < cbj { return Err(()); }
        
        if pgv(ba, jk as u32, 1, &mut bi[..cbj]) {
            Ok(())
        } else {
            Err(())
        }
    }
    
    fn aby(&self, jk: u64, bi: &[u8]) -> Result<(), ()> {
        let ik = FT_.lock();
        let ba = ik.get(self.cpy).ok_or(())?;
        if !ba.ack { return Err(()); }
        
        let cbj = ba.py as usize;
        if bi.len() < cbj { return Err(()); }
        
        if pgw(ba, jk as u32, 1, &bi[..cbj]) {
            Ok(())
        } else {
            Err(())
        }
    }
    
    fn zn(&self) -> usize {
        let ik = FT_.lock();
        ik.get(self.cpy)
            .map(|ba| ba.py as usize)
            .unwrap_or(H_)
    }
}






pub fn anl() -> bool {
    Be.load(Ordering::Acquire)
}


pub fn cjx() -> usize {
    FT_.lock().len()
}


pub fn bhh() -> Vec<(String, u64, u32)> {
    FT_.lock().iter().map(|ba| {
        let j = if ba.acs.is_empty() && ba.baj.is_empty() {
            alloc::format!("USB Storage (slot {})", ba.fw)
        } else {
            alloc::format!("{} {}", ba.acs, ba.baj)
        };
        (j, ba.hat, ba.py)
    }).collect()
}


pub fn ain(cpy: usize, aag: u64, az: usize, bi: &mut [u8]) -> Result<(), &'static str> {
    let ik = FT_.lock();
    let ba = ik.get(cpy).ok_or("Invalid device index")?;
    if !ba.ack { return Err("Device not ready"); }
    
    let py = ba.py as usize;
    if bi.len() < az * py {
        return Err("Buffer too small");
    }
    
    
    let mut qa = aag as u32;
    let mut l = 0;
    let mut ia = az;
    
    while ia > 0 {
        let jj = ia.v(BAC_);
        let aal = jj * py;
        
        if !pgv(ba, qa, jj as u16, &mut bi[l..l + aal]) {
            return Err("SCSI READ failed");
        }
        
        qa += jj as u32;
        l += aal;
        ia -= jj;
    }
    
    Ok(())
}


pub fn bpi(cpy: usize, aag: u64, az: usize, bi: &[u8]) -> Result<(), &'static str> {
    let ik = FT_.lock();
    let ba = ik.get(cpy).ok_or("Invalid device index")?;
    if !ba.ack { return Err("Device not ready"); }
    
    let py = ba.py as usize;
    if bi.len() < az * py {
        return Err("Buffer too small");
    }
    
    let mut qa = aag as u32;
    let mut l = 0;
    let mut ia = az;
    
    while ia > 0 {
        let jj = ia.v(BAC_);
        let aal = jj * py;
        
        if !pgw(ba, qa, jj as u16, &bi[l..l + aal]) {
            return Err("SCSI WRITE failed");
        }
        
        qa += jj as u32;
        l += aal;
        ia -= jj;
    }
    
    Ok(())
}


pub fn tcx(cpy: usize) -> Option<UsbBlockDevice> {
    let ik = FT_.lock();
    if cpy < ik.len() && ik[cpy].ack {
        Some(UsbBlockDevice::new(cpy))
    } else {
        None
    }
}
