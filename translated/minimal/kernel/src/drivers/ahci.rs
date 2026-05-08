




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
    RegH2D = 0x27,      
    RegD2H = 0x34,      
    DmaActivate = 0x39, 
    DmaSetup = 0x41,    
    Data = 0x46,        
    Bist = 0x58,        
    PioSetup = 0x5F,    
    DevBits = 0xA1,     
}


#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct Mc {
    pub fis_type: u8,   
    pub pmport_c: u8,   
    pub command: u8,    
    pub featurel: u8,   
    
    pub lba0: u8,       
    pub lba1: u8,       
    pub lba2: u8,       
    pub device: u8,     
    
    pub lba3: u8,       
    pub lba4: u8,       
    pub lba5: u8,       
    pub featureh: u8,   
    
    pub countl: u8,     
    pub counth: u8,     
    pub ckt: u8,        
    pub control: u8,    
    
    pub _reserved: [u8; 4],
}

impl Mc {
    pub const fn new() -> Self {
        Self {
            fis_type: FisType::RegH2D as u8,
            pmport_c: 0,
            command: 0,
            featurel: 0,
            lba0: 0, lba1: 0, lba2: 0,
            device: 0,
            lba3: 0, lba4: 0, lba5: 0,
            featureh: 0,
            countl: 0, counth: 0,
            ckt: 0, control: 0,
            _reserved: [0; 4],
        }
    }
}


#[repr(C, packed)]
pub struct Akf {
    pub fis_type: u8,
    pub pmport_i: u8,
    pub status: u8,
    pub error: u8,
    pub lba0: u8, pub lba1: u8, pub lba2: u8,
    pub device: u8,
    pub lba3: u8, pub lba4: u8, pub lba5: u8,
    pub _reserved0: u8,
    pub countl: u8, pub counth: u8,
    pub _reserved1: [u8; 6],
}


#[repr(C, packed)]
pub struct Ake {
    pub fis_type: u8,
    pub pmport_di: u8,
    pub status: u8,
    pub error: u8,
    pub lba0: u8, pub lba1: u8, pub lba2: u8,
    pub device: u8,
    pub lba3: u8, pub lba4: u8, pub lba5: u8,
    pub _reserved0: u8,
    pub countl: u8, pub counth: u8,
    pub _reserved1: u8,
    pub e_status: u8,
    pub wo: u16,
    pub _reserved2: [u8; 2],
}


#[repr(C, packed)]
pub struct Akd {
    pub fis_type: u8,
    pub pmport_dai: u8,
    pub _reserved0: [u8; 2],
    pub dma_buffer_id: u64,
    pub _reserved1: u32,
    pub dma_buffer_offset: u32,
    pub transfer_count: u32,
    pub _reserved2: u32,
}


#[repr(C, align(256))]
pub struct Akz {
    pub dsfis: Akd,       
    pub _pad0: [u8; 4],
    pub psfis: Ake,       
    pub _pad1: [u8; 12],
    pub rfis: Akf,          
    pub _pad2: [u8; 4],
    pub sdbfis: [u8; 8],          
    pub ufis: [u8; 64],           
    pub _reserved: [u8; 0x60],    
}






#[repr(C)]
#[derive(Clone, Copy)]
pub struct HbaCmdHeader {
    
    pub flags: u16,
    
    pub prdtl: u16,
    
    pub prdbc: u32,
    
    pub ctba: u64,
    
    pub _reserved: [u32; 4],
}

impl HbaCmdHeader {
    pub const fn new() -> Self {
        Self {
            flags: 0,
            prdtl: 0,
            prdbc: 0,
            ctba: 0,
            _reserved: [0; 4],
        }
    }
}


#[repr(C)]
#[derive(Clone, Copy)]
pub struct Ala {
    
    pub dba: u64,
    
    pub _reserved: u32,
    
    
    pub dbc_i: u32,
}



#[repr(C, align(128))]
pub struct Me {
    
    pub cfis: [u8; 64],
    
    pub acmd: [u8; 16],
    
    pub _reserved: [u8; 48],
    
    pub prdt: [Ala; 8],
}


#[repr(C, align(1024))]
pub struct Zq {
    pub headers: [HbaCmdHeader; 32],
}






pub struct PortMemory {
    pub cmd_list: Box<Zq>,
    pub fis: Box<Akz>,
    pub cmd_tables: [Box<Me>; 8],  
}

impl PortMemory {
    pub fn new() -> Self {
        Self {
            cmd_list: Box::new(Zq { headers: [HbaCmdHeader::new(); 32] }),
            fis: Box::new(unsafe { core::mem::zeroed() }),
            cmd_tables: core::array::from_fn(|_| Box::new(unsafe { core::mem::zeroed() })),
        }
    }
}


#[repr(C)]
pub struct Mf {
    
    pub cap: u32,
    
    pub ghc: u32,
    
    pub is: u32,
    
    pub pi: u32,
    
    pub vs: u32,
    
    pub ccc_ctl: u32,
    
    pub ccc_ports: u32,
    
    pub em_loc: u32,
    
    pub em_ctl: u32,
    
    pub cap2: u32,
    
    pub bohc: u32,
    
    _reserved: [u8; 0x74],
    
    _vendor: [u8; 0x60],
    
    pub ports: [Pc; 32],
}


#[repr(C)]
#[derive(Clone, Copy)]
pub struct Pc {
    
    pub clb: u64,
    
    pub fb: u64,
    
    pub is: u32,
    
    pub drt: u32,
    
    pub cmd: u32,
    
    _reserved0: u32,
    
    pub tfd: u32,
    
    pub sig: u32,
    
    pub ssts: u32,
    
    pub sctl: u32,
    
    pub serr: u32,
    
    pub sact: u32,
    
    pub ci: u32,
    
    pub sntf: u32,
    
    pub fbs: u32,
    
    _reserved1: [u32; 11],
    
    _vendor: [u32; 4],
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AhciDeviceType {
    None,
    Sata,
    Satapi,  
    Semb,    
    Pm,      
}


#[derive(Debug, Clone)]
pub struct Oa {
    pub port_num: u8,
    pub device_type: AhciDeviceType,
    pub sector_count: u64,
    pub model: String,
    pub serial: String,
}


pub struct Wf {
    pub base_addr: u64,
    pub virt_addr: u64,
    pub ports: Vec<Oa>,
    pub port_memory: Vec<Option<PortMemory>>,
    pub initialized: bool,
}

static Ao: Mutex<Option<Wf>> = Mutex::new(None);


const CTP_: u32 = 0x00000101;
const CTQ_: u32 = 0xEB140101;
const CTS_: u32 = 0xC33C0101;
const CTR_: u32 = 0x96690101;


const AXV_: u32 = 1 << 0;   
const AXU_: u32 = 1 << 4;  
const CCE_: u32 = 1 << 14;  
const AXT_: u32 = 1 << 15;  


const BMY_: u8 = 0x25;
const BMZ_: u8 = 0x35;
const BMX_: u8 = 0xEC;
const BMW_: u8 = 0xEA;


const RZ_: u8 = 0x80;
const SA_: u8 = 0x08;


const H_: usize = 512;


fn lc(virt: u64) -> u64 {
    let bz = crate::memory::hhdm_offset();
    virt.wrapping_sub(bz)
}


fn oxm(port: &mut Pc) {
    
    port.cmd &= !AXV_;
    
    
    port.cmd &= !AXU_;
    
    
    for _ in 0..1000 {
        if (port.cmd & CCE_) == 0 && (port.cmd & AXT_) == 0 {
            break;
        }
        
        for _ in 0..1000 { core::hint::spin_loop(); }
    }
}


fn owc(port: &mut Pc) {
    
    while (port.cmd & AXT_) != 0 {
        core::hint::spin_loop();
    }
    
    
    port.cmd |= AXU_;
    port.cmd |= AXV_;
}


fn emq(port: &Pc) -> Option<u32> {
    
    let azs = port.sact | port.ci;
    
    for i in 0..32 {
        if (azs & (1 << i)) == 0 {
            return Some(i);
        }
    }
    None
}


pub fn init(dih: u64) -> bool {
    if dih == 0 || dih == 0xFFFFFFFF {
        crate::serial_println!("[AHCI] Invalid BAR5 address");
        return false;
    }
    
    
    let eeu = (dih & !0xF) as u64;
    
    const ALZ_: usize = 0x2000;  
    
    crate::serial_println!("[AHCI] Mapping MMIO at phys={:#x} size={:#x}", eeu, ALZ_);
    
    let ffw = match crate::memory::yv(eeu, ALZ_) {
        Ok(virt) => virt,
        Err(e) => {
            crate::serial_println!("[AHCI] Failed to map MMIO: {}", e);
            return false;
        }
    };
    
    crate::serial_println!("[AHCI] Initializing at ABAR phys={:#x} virt={:#x}", eeu, ffw);
    
    let als = unsafe { &mut *(ffw as *mut Mf) };
    
    
    let version = als.vs;
    let axz = (version >> 16) & 0xFF;
    let ayh = version & 0xFF;
    crate::serial_println!("[AHCI] Version {}.{}", axz, ayh);
    
    let pi = als.pi;
    let cap = als.cap;
    let gjz = ((cap >> 8) & 0x1F) + 1;
    let jce = (cap >> 31) & 1 != 0; 
    
    crate::serial_println!("[AHCI] {} ports implemented, {} command slots, 64-bit DMA: {}", 
        pi.count_ones(), gjz, jce);
    
    
    als.ghc |= 1 << 31;
    
    
    als.ghc |= 1; 
    let mut jaj = 0u32;
    while als.ghc & 1 != 0 && jaj < 1_000_000 {
        jaj += 1;
        core::hint::spin_loop();
    }
    if als.ghc & 1 != 0 {
        crate::serial_println!("[AHCI] HBA reset timeout");
        return false;
    }
    
    
    als.ghc |= 1 << 31;
    
    let mut ports = Vec::new();
    let mut port_memory: Vec<Option<PortMemory>> = (0..32).map(|_| None).collect();
    
    
    for i in 0..32 {
        if pi & (1 << i) != 0 {
            let port = unsafe { &mut *(als.ports.as_mut_ptr().add(i)) };
            
            let ssts = port.ssts;
            let dmw = ssts & 0x0F;
            let ihp = (ssts >> 8) & 0x0F;
            
            if dmw == 3 && ihp == 1 {
                let sig = port.sig;
                let device_type = match sig {
                    CTP_ => AhciDeviceType::Sata,
                    CTQ_ => AhciDeviceType::Satapi,
                    CTS_ => AhciDeviceType::Semb,
                    CTR_ => AhciDeviceType::Pm,
                    _ => AhciDeviceType::None,
                };
                
                if device_type != AhciDeviceType::None {
                    crate::serial_println!("[AHCI] Port {}: {:?} device detected", i, device_type);
                    
                    
                    let mem = PortMemory::new();
                    
                    
                    oxm(port);
                    
                    
                    let flt = lc(&*mem.cmd_list as *const _ as u64);
                    
                    
                    let dpj = lc(&*mem.fis as *const _ as u64);
                    
                    
                    if !jce && (flt > 0xFFFF_FFFF || dpj > 0xFFFF_FFFF) {
                        crate::serial_println!("[AHCI] WARNING: Port {} DMA buffers above 4GB \
                            but controller lacks S64A! clb={:#x} fb={:#x}", i, flt, dpj);
                        
                        continue;
                    }
                    
                    port.clb = flt;
                    port.fb = dpj;
                    
                    
                    port.is = 0xFFFFFFFF;
                    
                    
                    port.serr = 0xFFFFFFFF;
                    
                    
                    owc(port);
                    
                    port_memory[i] = Some(mem);
                    
                    ports.push(Oa {
                        port_num: i as u8,
                        device_type,
                        sector_count: 0,
                        model: String::from("Unknown"),
                        serial: String::from("Unknown"),
                    });
                }
            }
        }
    }
    
    let fzv = !ports.is_empty();
    
    *Ao.lock() = Some(Wf {
        base_addr: eeu,
        virt_addr: ffw,
        ports,
        port_memory,
        initialized: fzv,
    });
    
    crate::serial_println!("[AHCI] Initialization {}", 
        if fzv { "complete" } else { "no devices" });
    
    fzv
}


pub fn ibt() -> u8 {
    Ao.lock().as_ref().map(|c| c.ports.len() as u8).unwrap_or(0)
}


pub fn adz() -> Vec<Oa> {
    Ao.lock().as_ref().map(|c| c.ports.clone()).unwrap_or_default()
}


pub fn is_initialized() -> bool {
    Ao.lock().as_ref().map(|c| c.initialized).unwrap_or(false)
}



pub fn btg(port_num: u8) -> Result<u64, &'static str> {
    let mut ctrl = Ao.lock();
    let ar = ctrl.as_mut().ok_or("AHCI not initialized")?;
    
    if !ar.initialized {
        return Err("AHCI not initialized");
    }
    
    let port_memory = ar.port_memory[port_num as usize].as_mut()
        .ok_or("Port memory not allocated")?;
    
    let als = unsafe { &mut *(ar.virt_addr as *mut Mf) };
    let port = unsafe { &mut *(als.ports.as_mut_ptr().add(port_num as usize)) };
    
    
    port.is = 0xFFFFFFFF;
    
    
    let slot = emq(port).ok_or("No free command slot")?;
    
    
    let ahq = &mut port_memory.cmd_list.headers[slot as usize];
    
    
    ahq.flags = 5;  
    ahq.prdtl = 1;  
    ahq.prdbc = 0;
    
    let aey = &mut *port_memory.cmd_tables[slot as usize];
    let chh = lc(aey as *const _ as u64);
    ahq.ctba = chh;
    
    
    unsafe {
        ptr::write_bytes(aey as *mut Me, 0, 1);
    }
    
    
    let mut ifn = vec![0u8; 512];
    let djz = lc(ifn.as_ptr() as u64);
    
    
    aey.prdt[0].dba = djz;
    aey.prdt[0].dbc_i = (512 - 1) | (1 << 31);  
    
    
    let cfis = unsafe { &mut *(aey.cfis.as_mut_ptr() as *mut Mc) };
    cfis.fis_type = FisType::RegH2D as u8;
    cfis.pmport_c = 0x80;  
    cfis.command = BMX_;
    cfis.device = 0;  
    cfis.countl = 0;
    cfis.counth = 0;
    cfis.lba0 = 0;
    cfis.lba1 = 0;
    cfis.lba2 = 0;
    cfis.lba3 = 0;
    cfis.lba4 = 0;
    cfis.lba5 = 0;
    
    
    core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
    
    
    let mut spin = 0u32;
    while (port.tfd & ((RZ_ | SA_) as u32)) != 0 && spin < 1_000_000 {
        spin += 1;
        core::hint::spin_loop();
    }
    if spin >= 1_000_000 {
        return Err("Port busy timeout");
    }
    
    
    port.ci = 1 << slot;
    
    
    let mut mz = 0u32;
    loop {
        if (port.ci & (1 << slot)) == 0 {
            break;
        }
        if (port.is & (1 << 30)) != 0 {
            return Err("Task file error during IDENTIFY");
        }
        mz += 1;
        if mz > 10_000_000 {
            return Err("IDENTIFY command timeout");
        }
        core::hint::spin_loop();
    }
    
    
    
    
    let um = unsafe { 
        core::slice::from_raw_parts(ifn.as_ptr() as *const u16, 256) 
    };
    
    
    let mxk = (um[83] & (1 << 10)) != 0;
    
    let sector_count = if mxk {
        
        (um[100] as u64) |
        ((um[101] as u64) << 16) |
        ((um[102] as u64) << 32) |
        ((um[103] as u64) << 48)
    } else {
        
        (um[60] as u64) | ((um[61] as u64) << 16)
    };
    
    
    let mut model = String::new();
    for i in 27..47 {
        let w = um[i];
        let hw = ((w >> 8) & 0xFF) as u8;
        let jf = (w & 0xFF) as u8;
        if hw >= 0x20 && hw < 0x7F { model.push(hw as char); }
        if jf >= 0x20 && jf < 0x7F { model.push(jf as char); }
    }
    let model = String::from(model.trim());
    
    
    let mut serial = String::new();
    for i in 10..20 {
        let w = um[i];
        let hw = ((w >> 8) & 0xFF) as u8;
        let jf = (w & 0xFF) as u8;
        if hw >= 0x20 && hw < 0x7F { serial.push(hw as char); }
        if jf >= 0x20 && jf < 0x7F { serial.push(jf as char); }
    }
    let serial = String::from(serial.trim());
    
    crate::serial_println!("[AHCI] Port {}: {} sectors ({} MB), model: {}, serial: {}", 
        port_num, sector_count, sector_count / 2048, model, serial);
    
    
    if let Some(port_info) = ar.ports.iter_mut().find(|aa| aa.port_num == port_num) {
        port_info.sector_count = sector_count;
        port_info.model = model;
        port_info.serial = serial;
    }
    
    Ok(sector_count)
}


pub fn mnl() {
    let nwf: Vec<u8> = {
        Ao.lock().as_ref()
            .map(|c| c.ports.iter().map(|aa| aa.port_num).collect())
            .unwrap_or_default()
    };
    
    for port_num in nwf {
        if let Err(e) = btg(port_num) {
            crate::serial_println!("[AHCI] Failed to identify port {}: {}", port_num, e);
        }
    }
}






pub fn read_sectors(port_num: u8, hb: u64, count: u16, buffer: &mut [u8]) -> Result<usize, &'static str> {
    if count == 0 || count > 128 {
        return Err("Invalid sector count (1-128)");
    }
    
    let aim = (count as usize) * H_;
    if buffer.len() < aim {
        return Err("Buffer too small");
    }
    
    let mut ctrl = Ao.lock();
    let ar = ctrl.as_mut().ok_or("AHCI not initialized")?;
    
    if !ar.initialized {
        return Err("AHCI not initialized");
    }
    
    
    let qqs = ar.ports.iter().position(|aa| aa.port_num == port_num)
        .ok_or("Port not found")?;
    
    let port_memory = ar.port_memory[port_num as usize].as_mut()
        .ok_or("Port memory not allocated")?;
    
    
    let als = unsafe { &mut *(ar.virt_addr as *mut Mf) };
    let port = unsafe { &mut *(als.ports.as_mut_ptr().add(port_num as usize)) };
    
    
    port.is = 0xFFFFFFFF;
    
    
    let slot = emq(port).ok_or("No free command slot")?;
    
    
    let ahq = &mut port_memory.cmd_list.headers[slot as usize];
    
    
    
    
    ahq.flags = 5;  
    ahq.prdtl = 1;  
    ahq.prdbc = 0;  
    
    
    let aey = &mut *port_memory.cmd_tables[slot as usize];
    let chh = lc(aey as *const _ as u64);
    ahq.ctba = chh;
    
    
    unsafe {
        ptr::write_bytes(aey as *mut Me, 0, 1);
    }
    
    
    
    let djz = lc(buffer.as_ptr() as u64);
    aey.prdt[0].dba = djz;
    aey.prdt[0].dbc_i = ((aim - 1) as u32) | (1 << 31);  
    
    
    let cfis = unsafe { &mut *(aey.cfis.as_mut_ptr() as *mut Mc) };
    cfis.fis_type = FisType::RegH2D as u8;
    cfis.pmport_c = 0x80;  
    cfis.command = BMY_;
    
    
    cfis.lba0 = (hb & 0xFF) as u8;
    cfis.lba1 = ((hb >> 8) & 0xFF) as u8;
    cfis.lba2 = ((hb >> 16) & 0xFF) as u8;
    cfis.device = 0x40;  
    cfis.lba3 = ((hb >> 24) & 0xFF) as u8;
    cfis.lba4 = ((hb >> 32) & 0xFF) as u8;
    cfis.lba5 = ((hb >> 40) & 0xFF) as u8;
    
    
    cfis.countl = (count & 0xFF) as u8;
    cfis.counth = ((count >> 8) & 0xFF) as u8;
    
    
    core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
    
    
    let mut spin = 0u32;
    while (port.tfd & ((RZ_ | SA_) as u32)) != 0 && spin < 1_000_000 {
        spin += 1;
        core::hint::spin_loop();
    }
    if spin >= 1_000_000 {
        return Err("Port busy timeout");
    }
    
    
    port.ci = 1 << slot;
    
    
    let mut mz = 0u32;
    loop {
        
        if (port.ci & (1 << slot)) == 0 {
            break;
        }
        
        
        if (port.is & (1 << 30)) != 0 {
            return Err("Task file error");
        }
        
        mz += 1;
        if mz > 10_000_000 {
            return Err("Command timeout");
        }
        
        core::hint::spin_loop();
    }
    
    
    if (port.is & (1 << 30)) != 0 {
        return Err("Task file error after completion");
    }
    
    Ok(aim)
}


pub fn write_sectors(port_num: u8, hb: u64, count: u16, buffer: &[u8]) -> Result<usize, &'static str> {
    if count == 0 || count > 128 {
        return Err("Invalid sector count (1-128)");
    }
    
    let aim = (count as usize) * H_;
    if buffer.len() < aim {
        return Err("Buffer too small");
    }
    
    let mut ctrl = Ao.lock();
    let ar = ctrl.as_mut().ok_or("AHCI not initialized")?;
    
    if !ar.initialized {
        return Err("AHCI not initialized");
    }
    
    let port_memory = ar.port_memory[port_num as usize].as_mut()
        .ok_or("Port memory not allocated")?;
    
    let als = unsafe { &mut *(ar.virt_addr as *mut Mf) };
    let port = unsafe { &mut *(als.ports.as_mut_ptr().add(port_num as usize)) };
    
    port.is = 0xFFFFFFFF;
    
    let slot = emq(port).ok_or("No free command slot")?;
    
    let ahq = &mut port_memory.cmd_list.headers[slot as usize];
    
    
    ahq.flags = 5 | (1 << 6);  
    ahq.prdtl = 1;
    ahq.prdbc = 0;
    
    let aey = &mut *port_memory.cmd_tables[slot as usize];
    let chh = lc(aey as *const _ as u64);
    ahq.ctba = chh;
    
    unsafe {
        ptr::write_bytes(aey as *mut Me, 0, 1);
    }
    
    let djz = lc(buffer.as_ptr() as u64);
    aey.prdt[0].dba = djz;
    aey.prdt[0].dbc_i = ((aim - 1) as u32) | (1 << 31);
    
    let cfis = unsafe { &mut *(aey.cfis.as_mut_ptr() as *mut Mc) };
    cfis.fis_type = FisType::RegH2D as u8;
    cfis.pmport_c = 0x80;
    cfis.command = BMZ_;
    
    cfis.lba0 = (hb & 0xFF) as u8;
    cfis.lba1 = ((hb >> 8) & 0xFF) as u8;
    cfis.lba2 = ((hb >> 16) & 0xFF) as u8;
    cfis.device = 0x40;
    cfis.lba3 = ((hb >> 24) & 0xFF) as u8;
    cfis.lba4 = ((hb >> 32) & 0xFF) as u8;
    cfis.lba5 = ((hb >> 40) & 0xFF) as u8;
    
    cfis.countl = (count & 0xFF) as u8;
    cfis.counth = ((count >> 8) & 0xFF) as u8;
    
    core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
    
    let mut spin = 0u32;
    while (port.tfd & ((RZ_ | SA_) as u32)) != 0 && spin < 1_000_000 {
        spin += 1;
        core::hint::spin_loop();
    }
    if spin >= 1_000_000 {
        return Err("Port busy timeout");
    }
    
    port.ci = 1 << slot;
    
    let mut mz = 0u32;
    loop {
        if (port.ci & (1 << slot)) == 0 {
            break;
        }
        
        if (port.is & (1 << 30)) != 0 {
            return Err("Task file error");
        }
        
        mz += 1;
        if mz > 10_000_000 {
            return Err("Command timeout");
        }
        
        core::hint::spin_loop();
    }
    
    if (port.is & (1 << 30)) != 0 {
        return Err("Task file error after completion");
    }
    
    Ok(aim)
}


pub fn qfx(port_num: u8) -> Result<(), &'static str> {
    let mut ctrl = Ao.lock();
    let ar = ctrl.as_mut().ok_or("AHCI not initialized")?;
    
    if !ar.initialized {
        return Err("AHCI not initialized");
    }
    
    let port_memory = ar.port_memory[port_num as usize].as_mut()
        .ok_or("Port memory not allocated")?;
    
    let als = unsafe { &mut *(ar.virt_addr as *mut Mf) };
    let port = unsafe { &mut *(als.ports.as_mut_ptr().add(port_num as usize)) };
    
    port.is = 0xFFFFFFFF;
    
    let slot = emq(port).ok_or("No free command slot")?;
    
    let ahq = &mut port_memory.cmd_list.headers[slot as usize];
    ahq.flags = 5; 
    ahq.prdtl = 0; 
    ahq.prdbc = 0;
    
    let aey = &mut *port_memory.cmd_tables[slot as usize];
    let chh = lc(aey as *const _ as u64);
    ahq.ctba = chh;
    
    unsafe {
        ptr::write_bytes(aey as *mut Me, 0, 1);
    }
    
    let cfis = unsafe { &mut *(aey.cfis.as_mut_ptr() as *mut Mc) };
    cfis.fis_type = FisType::RegH2D as u8;
    cfis.pmport_c = 0x80;
    cfis.command = BMW_;
    cfis.device = 0x40;
    
    core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
    
    
    let mut spin = 0u32;
    while (port.tfd & ((RZ_ | SA_) as u32)) != 0 && spin < 1_000_000 {
        spin += 1;
        core::hint::spin_loop();
    }
    if spin >= 1_000_000 {
        return Err("Port busy timeout");
    }
    
    port.ci = 1 << slot;
    
    
    let mut mz = 0u32;
    loop {
        if (port.ci & (1 << slot)) == 0 {
            break;
        }
        if (port.is & (1 << 30)) != 0 {
            return Err("Task file error during flush");
        }
        mz += 1;
        if mz > 30_000_000 {
            return Err("Flush timeout");
        }
        core::hint::spin_loop();
    }
    
    Ok(())
}





use crate::security::{StorageOperation, StorageSecurityError, Dn};
use crate::security::storage;





pub fn qvd(
    port_num: u8,
    hb: u64,
    count: u16,
    buffer: &mut [u8],
    task_id: u64,
) -> Result<usize, StorageError> {
    let disk = Dn(port_num);
    let op = StorageOperation::ReadSectors;
    
    
    storage::flh(disk, op, task_id)
        .map_err(StorageError::Security)?;
    
    
    storage::dib(task_id, disk, op, true);
    
    
    read_sectors(port_num, hb, count, buffer)
        .map_err(StorageError::Io)
}




pub fn qve(
    port_num: u8,
    hb: u64,
    count: u16,
    buffer: &[u8],
    task_id: u64,
) -> Result<usize, StorageError> {
    let disk = Dn(port_num);
    let op = StorageOperation::WriteSectors;
    
    
    match storage::flh(disk, op, task_id) {
        Ok(()) => {}
        Err(e) => {
            storage::dib(task_id, disk, op, false);
            return Err(StorageError::Security(e));
        }
    }
    
    
    storage::dib(task_id, disk, op, true);
    
    
    write_sectors(port_num, hb, count, buffer)
        .map_err(StorageError::Io)
}




pub fn qvc(
    port_num: u8,
    task_id: u64,
) -> Result<(), StorageError> {
    let disk = Dn(port_num);
    let op = StorageOperation::LowLevelFormat;
    
    
    match storage::flh(disk, op, task_id) {
        Ok(()) => {}
        Err(e) => {
            storage::dib(task_id, disk, op, false);
            crate::log_warn!(
                "[AHCI] FORMAT DENIED: task {} tried to format disk {} without permission",
                task_id, port_num
            );
            return Err(StorageError::Security(e));
        }
    }
    
    crate::log_warn!("[AHCI] !!! FORMATTING DISK {} - ALL DATA WILL BE LOST !!!", port_num);
    
    
    let dne = fyw(port_num).ok_or(StorageError::Io("Port not found"))?;
    let zp = dne.sector_count;
    
    if zp == 0 {
        return Err(StorageError::Io("Unknown disk size"));
    }
    
    
    let pwk = [0u8; H_];
    
    
    let mut cjt = 0u64;
    while cjt < zp {
        write_sectors(port_num, cjt, 1, &pwk)
            .map_err(StorageError::Io)?;
        cjt += 1;
        
        
        if cjt % 1000 == 0 {
            crate::log!("[AHCI] Format progress: {}/{} sectors", cjt, zp);
        }
    }
    
    storage::dib(task_id, disk, op, true);
    crate::log!("[AHCI] Disk {} formatted successfully ({} sectors)", port_num, zp);
    
    Ok(())
}


#[derive(Debug)]
pub enum StorageError {
    
    Security(StorageSecurityError),
    
    Io(&'static str),
}

impl core::fmt::Display for StorageError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Security(e) => write!(f, "Security: {}", e),
            Self::Io(e) => write!(f, "I/O: {}", e),
        }
    }
}


pub fn fyw(port_num: u8) -> Option<Oa> {
    let ctrl = Ao.lock();
    let ar = ctrl.as_ref()?;
    ar.ports.iter().find(|aa| aa.port_num == port_num).cloned()
}
