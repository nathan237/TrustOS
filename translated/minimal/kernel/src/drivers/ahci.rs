




use alloc::vec::Vec;
use alloc::vec;
use alloc::string::String;
use alloc::format;
use alloc::boxed::Box;
use spin::Mutex;
use core::ptr;






#[repr(u8)]
#[derive(Clone, Copy)]
pub enum FisType {
    Aee = 0x27,      
    Dfu = 0x34,      
    Cuk = 0x39, 
    Cul = 0x41,    
    Cud = 0x46,        
    Csa = 0x58,        
    Dej = 0x5F,    
    Cuh = 0xA1,     
}


#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct Abt {
    pub eqj: u8,   
    pub hvn: u8,   
    pub ro: u8,    
    pub srq: u8,   
    
    pub gkz: u8,       
    pub gla: u8,       
    pub glb: u8,       
    pub de: u8,     
    
    pub glc: u8,       
    pub gld: u8,       
    pub glf: u8,       
    pub srp: u8,   
    
    pub gdl: u8,     
    pub gdk: u8,     
    pub fkw: u8,        
    pub control: u8,    
    
    pub asi: [u8; 4],
}

impl Abt {
    pub const fn new() -> Self {
        Self {
            eqj: FisType::Aee as u8,
            hvn: 0,
            ro: 0,
            srq: 0,
            gkz: 0, gla: 0, glb: 0,
            de: 0,
            glc: 0, gld: 0, glf: 0,
            srp: 0,
            gdl: 0, gdk: 0,
            fkw: 0, control: 0,
            asi: [0; 4],
        }
    }
}


#[repr(C, packed)]
pub struct Cdo {
    pub eqj: u8,
    pub zft: u8,
    pub status: u8,
    pub zt: u8,
    pub gkz: u8, pub gla: u8, pub glb: u8,
    pub de: u8,
    pub glc: u8, pub gld: u8, pub glf: u8,
    pub iii: u8,
    pub gdl: u8, pub gdk: u8,
    pub fzp: [u8; 6],
}


#[repr(C, packed)]
pub struct Cdn {
    pub eqj: u8,
    pub zfs: u8,
    pub status: u8,
    pub zt: u8,
    pub gkz: u8, pub gla: u8, pub glb: u8,
    pub de: u8,
    pub glc: u8, pub gld: u8, pub glf: u8,
    pub iii: u8,
    pub gdl: u8, pub gdk: u8,
    pub fzp: u8,
    pub yoe: u8,
    pub asb: u16,
    pub jyk: [u8; 2],
}


#[repr(C, packed)]
pub struct Cdm {
    pub eqj: u8,
    pub zfr: u8,
    pub iii: [u8; 2],
    pub ymj: u64,
    pub fzp: u32,
    pub ymk: u32,
    pub ztn: u32,
    pub jyk: u32,
}


#[repr(C, align(256))]
pub struct Cex {
    pub ynv: Cdm,       
    pub iig: [u8; 4],
    pub zgt: Cdn,       
    pub msr: [u8; 12],
    pub zke: Cdo,          
    pub qdf: [u8; 4],
    pub zlu: [u8; 8],          
    pub ztv: [u8; 64],           
    pub asi: [u8; 0x60],    
}






#[repr(C)]
#[derive(Clone, Copy)]
pub struct HbaCmdHeader {
    
    pub flags: u16,
    
    pub hvv: u16,
    
    pub hvu: u32,
    
    pub hel: u64,
    
    pub asi: [u32; 4],
}

impl HbaCmdHeader {
    pub const fn new() -> Self {
        Self {
            flags: 0,
            hvv: 0,
            hvu: 0,
            hel: 0,
            asi: [0; 4],
        }
    }
}


#[repr(C)]
#[derive(Clone, Copy)]
pub struct Cey {
    
    pub kod: u64,
    
    pub asi: u32,
    
    
    pub koe: u32,
}



#[repr(C, align(128))]
pub struct Aca {
    
    pub aam: [u8; 64],
    
    pub yee: [u8; 16],
    
    pub asi: [u8; 48],
    
    pub gpp: [Cey; 8],
}


#[repr(C, align(1024))]
pub struct Bip {
    pub zk: [HbaCmdHeader; 32],
}






pub struct PortMemory {
    pub gcr: Box<Bip>,
    pub nuw: Box<Cex>,
    pub hdk: [Box<Aca>; 8],  
}

impl PortMemory {
    pub fn new() -> Self {
        Self {
            gcr: Box::new(Bip { zk: [HbaCmdHeader::new(); 32] }),
            nuw: Box::new(unsafe { core::mem::zeroed() }),
            hdk: core::array::nwe(|_| Box::new(unsafe { core::mem::zeroed() })),
        }
    }
}


#[repr(C)]
pub struct Acb {
    
    pub mh: u32,
    
    pub hln: u32,
    
    pub cyz: u32,
    
    pub akk: u32,
    
    pub gwl: u32,
    
    pub yhm: u32,
    
    pub yhn: u32,
    
    pub yow: u32,
    
    pub yov: u32,
    
    pub yhj: u32,
    
    pub ygq: u32,
    
    asi: [u8; 0x74],
    
    qdy: [u8; 0x60],
    
    pub xf: [Aiw; 32],
}


#[repr(C)]
#[derive(Clone, Copy)]
pub struct Aiw {
    
    pub rbc: u64,
    
    pub pq: u64,
    
    pub cyz: u32,
    
    pub hnr: u32,
    
    pub cmd: u32,
    
    iii: u32,
    
    pub jte: u32,
    
    pub sig: u32,
    
    pub ssts: u32,
    
    pub sctl: u32,
    
    pub whz: u32,
    
    pub wcf: u32,
    
    pub nc: u32,
    
    pub zov: u32,
    
    pub yqh: u32,
    
    fzp: [u32; 11],
    
    qdy: [u32; 4],
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AhciDeviceType {
    None,
    Qr,
    Bse,  
    Cmt,    
    Cja,      
}


#[derive(Debug, Clone)]
pub struct Agc {
    pub kg: u8,
    pub ceb: AhciDeviceType,
    pub agw: u64,
    pub model: String,
    pub serial: String,
}


pub struct Bbn {
    pub sm: u64,
    pub vd: u64,
    pub xf: Vec<Agc>,
    pub bnx: Vec<Option<PortMemory>>,
    pub jr: bool,
}

static Bn: Mutex<Option<Bbn>> = Mutex::new(None);


const CQA_: u32 = 0x00000101;
const CQB_: u32 = 0xEB140101;
const CQD_: u32 = 0xC33C0101;
const CQC_: u32 = 0x96690101;


const AVR_: u32 = 1 << 0;   
const AVQ_: u32 = 1 << 4;  
const BYY_: u32 = 1 << 14;  
const AVP_: u32 = 1 << 15;  


const BKK_: u8 = 0x25;
const BKL_: u8 = 0x35;
const BKJ_: u8 = 0xEC;
const BKI_: u8 = 0xEA;


const RD_: u8 = 0x80;
const RE_: u8 = 0x08;


const H_: usize = 512;


fn abw(ju: u64) -> u64 {
    let hp = crate::memory::lr();
    ju.nj(hp)
}


fn wum(port: &mut Aiw) {
    
    port.cmd &= !AVR_;
    
    
    port.cmd &= !AVQ_;
    
    
    for _ in 0..1000 {
        if (port.cmd & BYY_) == 0 && (port.cmd & AVP_) == 0 {
            break;
        }
        
        for _ in 0..1000 { core::hint::hc(); }
    }
}


fn wsp(port: &mut Aiw) {
    
    while (port.cmd & AVP_) != 0 {
        core::hint::hc();
    }
    
    
    port.cmd |= AVQ_;
    port.cmd |= AVR_;
}


fn iuq(port: &Aiw) -> Option<u32> {
    
    let cuj = port.wcf | port.nc;
    
    for a in 0..32 {
        if (cuj & (1 << a)) == 0 {
            return Some(a);
        }
    }
    None
}


pub fn init(gzp: u64) -> bool {
    if gzp == 0 || gzp == 0xFFFFFFFF {
        crate::serial_println!("[AHCI] Invalid BAR5 address");
        return false;
    }
    
    
    let iio = (gzp & !0xF) as u64;
    
    const AKF_: usize = 0x2000;  
    
    crate::serial_println!("[AHCI] Mapping MMIO at phys={:#x} size={:#x}", iio, AKF_);
    
    let jys = match crate::memory::bki(iio, AKF_) {
        Ok(ju) => ju,
        Err(aa) => {
            crate::serial_println!("[AHCI] Failed to map MMIO: {}", aa);
            return false;
        }
    };
    
    crate::serial_println!("[AHCI] Initializing at ABAR phys={:#x} virt={:#x}", iio, jys);
    
    let but = unsafe { &mut *(jys as *mut Acb) };
    
    
    let dk = but.gwl;
    let efb = (dk >> 16) & 0xFF;
    let efm = dk & 0xFF;
    crate::serial_println!("[AHCI] Version {}.{}", efb, efm);
    
    let akk = but.akk;
    let mh = but.mh;
    let uwg = ((mh >> 8) & 0x1F) + 1;
    let pey = (mh >> 31) & 1 != 0; 
    
    crate::serial_println!("[AHCI] {} ports implemented, {} command slots, 64-bit DMA: {}", 
        akk.ipi(), uwg, pey);
    
    
    but.hln |= 1 << 31;
    
    
    but.hln |= 1; 
    let mut pcr = 0u32;
    while but.hln & 1 != 0 && pcr < 1_000_000 {
        pcr += 1;
        core::hint::hc();
    }
    if but.hln & 1 != 0 {
        crate::serial_println!("[AHCI] HBA reset timeout");
        return false;
    }
    
    
    but.hln |= 1 << 31;
    
    let mut xf = Vec::new();
    let mut bnx: Vec<Option<PortMemory>> = (0..32).map(|_| None).collect();
    
    
    for a in 0..32 {
        if akk & (1 << a) != 0 {
            let port = unsafe { &mut *(but.xf.mw().add(a)) };
            
            let ssts = port.ssts;
            let rwk = ssts & 0x0F;
            let twn = (ssts >> 8) & 0x0F;
            
            if rwk == 3 && twn == 1 {
                let sig = port.sig;
                let ceb = match sig {
                    CQA_ => AhciDeviceType::Qr,
                    CQB_ => AhciDeviceType::Bse,
                    CQD_ => AhciDeviceType::Cmt,
                    CQC_ => AhciDeviceType::Cja,
                    _ => AhciDeviceType::None,
                };
                
                if ceb != AhciDeviceType::None {
                    crate::serial_println!("[AHCI] Port {}: {:?} device detected", a, ceb);
                    
                    
                    let mem = PortMemory::new();
                    
                    
                    wum(port);
                    
                    
                    let kht = abw(&*mem.gcr as *const _ as u64);
                    
                    
                    let hiz = abw(&*mem.nuw as *const _ as u64);
                    
                    
                    if !pey && (kht > 0xFFFF_FFFF || hiz > 0xFFFF_FFFF) {
                        crate::serial_println!("[AHCI] WARNING: Port {} DMA buffers above 4GB \
                            but controller lacks S64A! clb={:#x} fb={:#x}", a, kht, hiz);
                        
                        continue;
                    }
                    
                    port.rbc = kht;
                    port.pq = hiz;
                    
                    
                    port.cyz = 0xFFFFFFFF;
                    
                    
                    port.whz = 0xFFFFFFFF;
                    
                    
                    wsp(port);
                    
                    bnx[a] = Some(mem);
                    
                    xf.push(Agc {
                        kg: a as u8,
                        ceb,
                        agw: 0,
                        model: String::from("Unknown"),
                        serial: String::from("Unknown"),
                    });
                }
            }
        }
    }
    
    let lbd = !xf.is_empty();
    
    *Bn.lock() = Some(Bbn {
        sm: iio,
        vd: jys,
        xf,
        bnx,
        jr: lbd,
    });
    
    crate::serial_println!("[AHCI] Initialization {}", 
        if lbd { "complete" } else { "no devices" });
    
    lbd
}


pub fn nyj() -> u8 {
    Bn.lock().as_ref().map(|r| r.xf.len() as u8).unwrap_or(0)
}


pub fn bhh() -> Vec<Agc> {
    Bn.lock().as_ref().map(|r| r.xf.clone()).age()
}


pub fn ky() -> bool {
    Bn.lock().as_ref().map(|r| r.jr).unwrap_or(false)
}



pub fn eda(kg: u8) -> Result<u64, &'static str> {
    let mut db = Bn.lock();
    let df = db.as_mut().ok_or("AHCI not initialized")?;
    
    if !df.jr {
        return Err("AHCI not initialized");
    }
    
    let bnx = df.bnx[kg as usize].as_mut()
        .ok_or("Port memory not allocated")?;
    
    let but = unsafe { &mut *(df.vd as *mut Acb) };
    let port = unsafe { &mut *(but.xf.mw().add(kg as usize)) };
    
    
    port.cyz = 0xFFFFFFFF;
    
    
    let gk = iuq(port).ok_or("No free command slot")?;
    
    
    let bmk = &mut bnx.gcr.zk[gk as usize];
    
    
    bmk.flags = 5;  
    bmk.hvv = 1;  
    bmk.hvu = 0;
    
    let bgj = &mut *bnx.hdk[gk as usize];
    let ffc = abw(bgj as *const _ as u64);
    bmk.hel = ffc;
    
    
    unsafe {
        ptr::ahx(bgj as *mut Aca, 0, 1);
    }
    
    
    let mut oda = vec![0u8; 512];
    let hbt = abw(oda.fq() as u64);
    
    
    bgj.gpp[0].kod = hbt;
    bgj.gpp[0].koe = (512 - 1) | (1 << 31);  
    
    
    let aam = unsafe { &mut *(bgj.aam.mw() as *mut Abt) };
    aam.eqj = FisType::Aee as u8;
    aam.hvn = 0x80;  
    aam.ro = BKJ_;
    aam.de = 0;  
    aam.gdl = 0;
    aam.gdk = 0;
    aam.gkz = 0;
    aam.gla = 0;
    aam.glb = 0;
    aam.glc = 0;
    aam.gld = 0;
    aam.glf = 0;
    
    
    core::sync::atomic::cxt(core::sync::atomic::Ordering::SeqCst);
    
    
    let mut spin = 0u32;
    while (port.jte & ((RD_ | RE_) as u32)) != 0 && spin < 1_000_000 {
        spin += 1;
        core::hint::hc();
    }
    if spin >= 1_000_000 {
        return Err("Port busy timeout");
    }
    
    
    port.nc = 1 << gk;
    
    
    let mut aah = 0u32;
    loop {
        if (port.nc & (1 << gk)) == 0 {
            break;
        }
        if (port.cyz & (1 << 30)) != 0 {
            return Err("Task file error during IDENTIFY");
        }
        aah += 1;
        if aah > 10_000_000 {
            return Err("IDENTIFY command timeout");
        }
        core::hint::hc();
    }
    
    
    
    
    let aoh = unsafe { 
        core::slice::anh(oda.fq() as *const u16, 256) 
    };
    
    
    let udm = (aoh[83] & (1 << 10)) != 0;
    
    let agw = if udm {
        
        (aoh[100] as u64) |
        ((aoh[101] as u64) << 16) |
        ((aoh[102] as u64) << 32) |
        ((aoh[103] as u64) << 48)
    } else {
        
        (aoh[60] as u64) | ((aoh[61] as u64) << 16)
    };
    
    
    let mut model = String::new();
    for a in 27..47 {
        let d = aoh[a];
        let rw = ((d >> 8) & 0xFF) as u8;
        let tx = (d & 0xFF) as u8;
        if rw >= 0x20 && rw < 0x7F { model.push(rw as char); }
        if tx >= 0x20 && tx < 0x7F { model.push(tx as char); }
    }
    let model = String::from(model.em());
    
    
    let mut serial = String::new();
    for a in 10..20 {
        let d = aoh[a];
        let rw = ((d >> 8) & 0xFF) as u8;
        let tx = (d & 0xFF) as u8;
        if rw >= 0x20 && rw < 0x7F { serial.push(rw as char); }
        if tx >= 0x20 && tx < 0x7F { serial.push(tx as char); }
    }
    let serial = String::from(serial.em());
    
    crate::serial_println!("[AHCI] Port {}: {} sectors ({} MB), model: {}, serial: {}", 
        kg, agw, agw / 2048, model, serial);
    
    
    if let Some(luw) = df.xf.el().du(|ai| ai.kg == kg) {
        luw.agw = agw;
        luw.model = model;
        luw.serial = serial;
    }
    
    Ok(agw)
}


pub fn tri() {
    let vjy: Vec<u8> = {
        Bn.lock().as_ref()
            .map(|r| r.xf.iter().map(|ai| ai.kg).collect())
            .age()
    };
    
    for kg in vjy {
        if let Err(aa) = eda(kg) {
            crate::serial_println!("[AHCI] Failed to identify port {}: {}", kg, aa);
        }
    }
}






pub fn ain(kg: u8, qa: u64, az: u16, bi: &mut [u8]) -> Result<usize, &'static str> {
    if az == 0 || az > 128 {
        return Err("Invalid sector count (1-128)");
    }
    
    let bod = (az as usize) * H_;
    if bi.len() < bod {
        return Err("Buffer too small");
    }
    
    let mut db = Bn.lock();
    let df = db.as_mut().ok_or("AHCI not initialized")?;
    
    if !df.jr {
        return Err("AHCI not initialized");
    }
    
    
    let zfw = df.xf.iter().qf(|ai| ai.kg == kg)
        .ok_or("Port not found")?;
    
    let bnx = df.bnx[kg as usize].as_mut()
        .ok_or("Port memory not allocated")?;
    
    
    let but = unsafe { &mut *(df.vd as *mut Acb) };
    let port = unsafe { &mut *(but.xf.mw().add(kg as usize)) };
    
    
    port.cyz = 0xFFFFFFFF;
    
    
    let gk = iuq(port).ok_or("No free command slot")?;
    
    
    let bmk = &mut bnx.gcr.zk[gk as usize];
    
    
    
    
    bmk.flags = 5;  
    bmk.hvv = 1;  
    bmk.hvu = 0;  
    
    
    let bgj = &mut *bnx.hdk[gk as usize];
    let ffc = abw(bgj as *const _ as u64);
    bmk.hel = ffc;
    
    
    unsafe {
        ptr::ahx(bgj as *mut Aca, 0, 1);
    }
    
    
    
    let hbt = abw(bi.fq() as u64);
    bgj.gpp[0].kod = hbt;
    bgj.gpp[0].koe = ((bod - 1) as u32) | (1 << 31);  
    
    
    let aam = unsafe { &mut *(bgj.aam.mw() as *mut Abt) };
    aam.eqj = FisType::Aee as u8;
    aam.hvn = 0x80;  
    aam.ro = BKK_;
    
    
    aam.gkz = (qa & 0xFF) as u8;
    aam.gla = ((qa >> 8) & 0xFF) as u8;
    aam.glb = ((qa >> 16) & 0xFF) as u8;
    aam.de = 0x40;  
    aam.glc = ((qa >> 24) & 0xFF) as u8;
    aam.gld = ((qa >> 32) & 0xFF) as u8;
    aam.glf = ((qa >> 40) & 0xFF) as u8;
    
    
    aam.gdl = (az & 0xFF) as u8;
    aam.gdk = ((az >> 8) & 0xFF) as u8;
    
    
    core::sync::atomic::cxt(core::sync::atomic::Ordering::SeqCst);
    
    
    let mut spin = 0u32;
    while (port.jte & ((RD_ | RE_) as u32)) != 0 && spin < 1_000_000 {
        spin += 1;
        core::hint::hc();
    }
    if spin >= 1_000_000 {
        return Err("Port busy timeout");
    }
    
    
    port.nc = 1 << gk;
    
    
    let mut aah = 0u32;
    loop {
        
        if (port.nc & (1 << gk)) == 0 {
            break;
        }
        
        
        if (port.cyz & (1 << 30)) != 0 {
            return Err("Task file error");
        }
        
        aah += 1;
        if aah > 10_000_000 {
            return Err("Command timeout");
        }
        
        core::hint::hc();
    }
    
    
    if (port.cyz & (1 << 30)) != 0 {
        return Err("Task file error after completion");
    }
    
    Ok(bod)
}


pub fn bpi(kg: u8, qa: u64, az: u16, bi: &[u8]) -> Result<usize, &'static str> {
    if az == 0 || az > 128 {
        return Err("Invalid sector count (1-128)");
    }
    
    let bod = (az as usize) * H_;
    if bi.len() < bod {
        return Err("Buffer too small");
    }
    
    let mut db = Bn.lock();
    let df = db.as_mut().ok_or("AHCI not initialized")?;
    
    if !df.jr {
        return Err("AHCI not initialized");
    }
    
    let bnx = df.bnx[kg as usize].as_mut()
        .ok_or("Port memory not allocated")?;
    
    let but = unsafe { &mut *(df.vd as *mut Acb) };
    let port = unsafe { &mut *(but.xf.mw().add(kg as usize)) };
    
    port.cyz = 0xFFFFFFFF;
    
    let gk = iuq(port).ok_or("No free command slot")?;
    
    let bmk = &mut bnx.gcr.zk[gk as usize];
    
    
    bmk.flags = 5 | (1 << 6);  
    bmk.hvv = 1;
    bmk.hvu = 0;
    
    let bgj = &mut *bnx.hdk[gk as usize];
    let ffc = abw(bgj as *const _ as u64);
    bmk.hel = ffc;
    
    unsafe {
        ptr::ahx(bgj as *mut Aca, 0, 1);
    }
    
    let hbt = abw(bi.fq() as u64);
    bgj.gpp[0].kod = hbt;
    bgj.gpp[0].koe = ((bod - 1) as u32) | (1 << 31);
    
    let aam = unsafe { &mut *(bgj.aam.mw() as *mut Abt) };
    aam.eqj = FisType::Aee as u8;
    aam.hvn = 0x80;
    aam.ro = BKL_;
    
    aam.gkz = (qa & 0xFF) as u8;
    aam.gla = ((qa >> 8) & 0xFF) as u8;
    aam.glb = ((qa >> 16) & 0xFF) as u8;
    aam.de = 0x40;
    aam.glc = ((qa >> 24) & 0xFF) as u8;
    aam.gld = ((qa >> 32) & 0xFF) as u8;
    aam.glf = ((qa >> 40) & 0xFF) as u8;
    
    aam.gdl = (az & 0xFF) as u8;
    aam.gdk = ((az >> 8) & 0xFF) as u8;
    
    core::sync::atomic::cxt(core::sync::atomic::Ordering::SeqCst);
    
    let mut spin = 0u32;
    while (port.jte & ((RD_ | RE_) as u32)) != 0 && spin < 1_000_000 {
        spin += 1;
        core::hint::hc();
    }
    if spin >= 1_000_000 {
        return Err("Port busy timeout");
    }
    
    port.nc = 1 << gk;
    
    let mut aah = 0u32;
    loop {
        if (port.nc & (1 << gk)) == 0 {
            break;
        }
        
        if (port.cyz & (1 << 30)) != 0 {
            return Err("Task file error");
        }
        
        aah += 1;
        if aah > 10_000_000 {
            return Err("Command timeout");
        }
        
        core::hint::hc();
    }
    
    if (port.cyz & (1 << 30)) != 0 {
        return Err("Task file error after completion");
    }
    
    Ok(bod)
}


pub fn yra(kg: u8) -> Result<(), &'static str> {
    let mut db = Bn.lock();
    let df = db.as_mut().ok_or("AHCI not initialized")?;
    
    if !df.jr {
        return Err("AHCI not initialized");
    }
    
    let bnx = df.bnx[kg as usize].as_mut()
        .ok_or("Port memory not allocated")?;
    
    let but = unsafe { &mut *(df.vd as *mut Acb) };
    let port = unsafe { &mut *(but.xf.mw().add(kg as usize)) };
    
    port.cyz = 0xFFFFFFFF;
    
    let gk = iuq(port).ok_or("No free command slot")?;
    
    let bmk = &mut bnx.gcr.zk[gk as usize];
    bmk.flags = 5; 
    bmk.hvv = 0; 
    bmk.hvu = 0;
    
    let bgj = &mut *bnx.hdk[gk as usize];
    let ffc = abw(bgj as *const _ as u64);
    bmk.hel = ffc;
    
    unsafe {
        ptr::ahx(bgj as *mut Aca, 0, 1);
    }
    
    let aam = unsafe { &mut *(bgj.aam.mw() as *mut Abt) };
    aam.eqj = FisType::Aee as u8;
    aam.hvn = 0x80;
    aam.ro = BKI_;
    aam.de = 0x40;
    
    core::sync::atomic::cxt(core::sync::atomic::Ordering::SeqCst);
    
    
    let mut spin = 0u32;
    while (port.jte & ((RD_ | RE_) as u32)) != 0 && spin < 1_000_000 {
        spin += 1;
        core::hint::hc();
    }
    if spin >= 1_000_000 {
        return Err("Port busy timeout");
    }
    
    port.nc = 1 << gk;
    
    
    let mut aah = 0u32;
    loop {
        if (port.nc & (1 << gk)) == 0 {
            break;
        }
        if (port.cyz & (1 << 30)) != 0 {
            return Err("Task file error during flush");
        }
        aah += 1;
        if aah > 30_000_000 {
            return Err("Flush timeout");
        }
        core::hint::hc();
    }
    
    Ok(())
}





use crate::security::{StorageOperation, StorageSecurityError, Ik};
use crate::security::storage;





pub fn zme(
    kg: u8,
    qa: u64,
    az: u16,
    bi: &mut [u8],
    aod: u64,
) -> Result<usize, StorageError> {
    let disk = Ik(kg);
    let op = StorageOperation::Alx;
    
    
    storage::khe(disk, op, aod)
        .jd(StorageError::De)?;
    
    
    storage::gzg(aod, disk, op, true);
    
    
    ain(kg, qa, az, bi)
        .jd(StorageError::Xg)
}




pub fn zmf(
    kg: u8,
    qa: u64,
    az: u16,
    bi: &[u8],
    aod: u64,
) -> Result<usize, StorageError> {
    let disk = Ik(kg);
    let op = StorageOperation::Aof;
    
    
    match storage::khe(disk, op, aod) {
        Ok(()) => {}
        Err(aa) => {
            storage::gzg(aod, disk, op, false);
            return Err(StorageError::De(aa));
        }
    }
    
    
    storage::gzg(aod, disk, op, true);
    
    
    bpi(kg, qa, az, bi)
        .jd(StorageError::Xg)
}




pub fn zmd(
    kg: u8,
    aod: u64,
) -> Result<(), StorageError> {
    let disk = Ik(kg);
    let op = StorageOperation::Ajw;
    
    
    match storage::khe(disk, op, aod) {
        Ok(()) => {}
        Err(aa) => {
            storage::gzg(aod, disk, op, false);
            crate::log_warn!(
                "[AHCI] FORMAT DENIED: task {} tried to format disk {} without permission",
                aod, kg
            );
            return Err(StorageError::De(aa));
        }
    }
    
    crate::log_warn!("[AHCI] !!! FORMATTING DISK {} - ALL DATA WILL BE LOST !!!", kg);
    
    
    let hgf = kyv(kg).ok_or(StorageError::Xg("Port not found"))?;
    let axf = hgf.agw;
    
    if axf == 0 {
        return Err(StorageError::Xg("Unknown disk size"));
    }
    
    
    let xxj = [0u8; H_];
    
    
    let mut fiy = 0u64;
    while fiy < axf {
        bpi(kg, fiy, 1, &xxj)
            .jd(StorageError::Xg)?;
        fiy += 1;
        
        
        if fiy % 1000 == 0 {
            crate::log!("[AHCI] Format progress: {}/{} sectors", fiy, axf);
        }
    }
    
    storage::gzg(aod, disk, op, true);
    crate::log!("[AHCI] Disk {} formatted successfully ({} sectors)", kg, axf);
    
    Ok(())
}


#[derive(Debug)]
pub enum StorageError {
    
    De(StorageSecurityError),
    
    Xg(&'static str),
}

impl core::fmt::Display for StorageError {
    fn fmt(&self, bb: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::De(aa) => write!(bb, "Security: {}", aa),
            Self::Xg(aa) => write!(bb, "I/O: {}", aa),
        }
    }
}


pub fn kyv(kg: u8) -> Option<Agc> {
    let db = Bn.lock();
    let df = db.as_ref()?;
    df.xf.iter().du(|ai| ai.kg == kg).abn()
}
