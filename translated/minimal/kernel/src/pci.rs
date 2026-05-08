




use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use crate::arch::Port;
use spin::Mutex;


const BEV_: u16 = 0xCF8;
const BEW_: u16 = 0xCFC;


pub mod class {
    pub const Arf: u8 = 0x00;
    pub const FZ_: u8 = 0x01;
    pub const Gr: u8 = 0x02;
    pub const Du: u8 = 0x03;
    pub const Abd: u8 = 0x04;
    pub const Amo: u8 = 0x05;
    pub const Jk: u8 = 0x06;
    pub const CWZ_: u8 = 0x07;
    pub const BNP_: u8 = 0x08;
    pub const Alk: u8 = 0x09;
    pub const Aik: u8 = 0x0A;
    pub const Anj: u8 = 0x0B;
    pub const QG_: u8 = 0x0C;
    pub const Agk: u8 = 0x0D;
    pub const All: u8 = 0x0E;
    pub const Aoq: u8 = 0x0F;
    pub const Aiy: u8 = 0x10;
    pub const CWY_: u8 = 0x11;
}


pub mod storage {
    pub const Aor: u8 = 0x00;
    pub const Ale: u8 = 0x01;
    pub const Avy: u8 = 0x02;
    pub const Axp: u8 = 0x03;
    pub const Ank: u8 = 0x04;
    pub const Agy: u8 = 0x05;
    pub const Aop: u8 = 0x06;
    pub const Bbl: u8 = 0x07;
    pub const Amx: u8 = 0x08;  
}


pub mod network {
    pub const Aje: u8 = 0x00;
    pub const EMA_: u8 = 0x01;
    pub const Avt: u8 = 0x02;
    pub const Asq: u8 = 0x03;
    pub const Axt: u8 = 0x04;
    pub const Bab: u8 = 0x06;
    pub const Zu: u8 = 0x07;
}


pub mod bridge {
    pub const Akx: u8 = 0x00;
    pub const Alq: u8 = 0x01;
    pub const Auf: u8 = 0x02;
    pub const Ays: u8 = 0x03;
    pub const CMD_: u8 = 0x04;
    pub const Azz: u8 = 0x05;
    pub const Azr: u8 = 0x06;
    pub const Ath: u8 = 0x07;
    pub const Baw: u8 = 0x08;
    pub const ECW_: u8 = 0x09;
    pub const DUT_: u8 = 0x0A;
}


pub mod serial {
    pub const Avv: u8 = 0x00;
    pub const Qy: u8 = 0x01;
    pub const Bcx: u8 = 0x02;
    pub const Qs: u8 = 0x03;
    pub const Avu: u8 = 0x04;
    pub const Apo: u8 = 0x05;
    pub const Zu: u8 = 0x06;
    pub const Axq: u8 = 0x07;
}


pub mod usb {
    pub const Afp: u8 = 0x00;
    pub const Abw: u8 = 0x10;
    pub const Rv: u8 = 0x20;
    pub const Wa: u8 = 0x30;
}


#[derive(Debug, Clone)]
pub struct L {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class_code: u8,
    pub subclass: u8,
    pub prog_if: u8,
    pub revision: u8,
    pub header_type: u8,
    pub interrupt_line: u8,
    pub interrupt_pin: u8,
    pub bar: [u32; 6],
}

impl L {
    
    pub fn class_name(&self) -> &'static str {
        match self.class_code {
            class::Arf => "Unclassified",
            class::FZ_ => "Mass Storage",
            class::Gr => "Network Controller",
            class::Du => "Display Controller",
            class::Abd => "Multimedia",
            class::Amo => "Memory Controller",
            class::Jk => "Bridge",
            class::CWZ_ => "Communication",
            class::BNP_ => "Peripheral",
            class::Alk => "Input Device",
            class::Aik => "Docking Station",
            class::Anj => "Processor",
            class::QG_ => "Serial Bus",
            class::Agk => "Wireless",
            class::All => "Intelligent I/O",
            class::Aoq => "Satellite",
            class::Aiy => "Encryption",
            class::CWY_ => "Signal Processing",
            _ => "Unknown",
        }
    }
    
    
    pub fn subclass_name(&self) -> &'static str {
        match (self.class_code, self.subclass) {
            
            (class::FZ_, storage::Ale) => "IDE Controller",
            (class::FZ_, storage::Aop) => "SATA Controller",
            (class::FZ_, storage::Amx) => "NVMe Controller",
            (class::FZ_, storage::Ank) => "RAID Controller",
            (class::FZ_, storage::Aor) => "SCSI Controller",
            (class::FZ_, storage::Agy) => "ATA Controller",
            
            
            (class::Gr, network::Aje) => "Ethernet",
            (class::Gr, network::Zu) => "InfiniBand",
            
            
            (class::Du, 0x00) => "VGA Compatible",
            (class::Du, 0x01) => "XGA Controller",
            (class::Du, 0x02) => "3D Controller",
            
            
            (class::Jk, bridge::Akx) => "Host Bridge",
            (class::Jk, bridge::Alq) => "ISA Bridge",
            (class::Jk, bridge::CMD_) => "PCI-to-PCI Bridge",
            
            
            (class::QG_, serial::Qs) => match self.prog_if {
                usb::Afp => "USB UHCI",
                usb::Abw => "USB OHCI",
                usb::Rv => "USB 2.0 EHCI",
                usb::Wa => "USB 3.0 xHCI",
                0xFE => "USB Device",
                _ => "USB Controller",
            },
            (class::QG_, serial::Apo) => "SMBus",
            
            _ => "",
        }
    }
    
    
    pub fn vendor_name(&self) -> &'static str {
        match self.vendor_id {
            0x8086 => "Intel",
            0x1022 => "AMD",
            0x10DE => "NVIDIA",
            0x1002 => "AMD/ATI",
            0x14E4 => "Broadcom",
            0x10EC => "Realtek",
            0x8087 => "Intel (Wireless)",
            0x1B4B => "Marvell",
            0x1969 => "Qualcomm Atheros",
            0x168C => "Qualcomm Atheros",
            0x1AF4 => "Red Hat (virtio)",
            0x1234 => "QEMU",
            0x15AD => "VMware",
            0x80EE => "VirtualBox",
            0x1AB8 => "Parallels",
            _ => "Unknown",
        }
    }
    
    
    pub fn is_multifunction(&self) -> bool {
        self.header_type & 0x80 != 0
    }
    
    
    pub fn bar_address(&self, index: usize) -> Option<u64> {
        if index >= 6 {
            return None;
        }
        
        let bar = self.bar[index];
        if bar == 0 {
            return None;
        }
        
        
        if bar & 1 == 0 {
            
            let bqj = (bar >> 1) & 0x3;
            match bqj {
                0 => Some((bar & 0xFFFFFFF0) as u64), 
                2 if index < 5 => {
                    
                    let high = self.bar[index + 1] as u64;
                    Some(((high << 32) | (bar & 0xFFFFFFF0) as u64))
                }
                _ => None,
            }
        } else {
            
            Some((bar & 0xFFFFFFFC) as u64)
        }
    }
    
    
    pub fn bar_is_memory(&self, index: usize) -> bool {
        if index >= 6 {
            return false;
        }
        self.bar[index] & 1 == 0
    }
    
    
    pub fn pyk(&self, index: usize) -> bool {
        if index >= 6 {
            return false;
        }
        self.bar[index] & 1 != 0
    }
}


static Hm: Mutex<Vec<L>> = Mutex::new(Vec::new());


static BVS_: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);

static NU_: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);

static ADI_: core::sync::atomic::AtomicU8 = core::sync::atomic::AtomicU8::new(0);

static ADH_: core::sync::atomic::AtomicU8 = core::sync::atomic::AtomicU8::new(0);


fn mpd() {
    if let Some(info) = crate::acpi::rk() {
        if let Some(first) = info.mcfg_regions.first() {
            let base = first.base_address;
            let size = first.size() as usize;
            let start_bus = first.start_bus;
            let end_bus = first.end_bus;
            
            crate::serial_println!("[PCI] PCIe ECAM detected: base={:#x} size={:#x} buses={}-{}",
                base, size, start_bus, end_bus);
            
            match crate::memory::yv(base, size) {
                Ok(virt) => {
                    BVS_.store(base, core::sync::atomic::Ordering::SeqCst);
                    NU_.store(virt, core::sync::atomic::Ordering::SeqCst);
                    ADI_.store(start_bus, core::sync::atomic::Ordering::SeqCst);
                    ADH_.store(end_bus, core::sync::atomic::Ordering::SeqCst);
                    crate::serial_println!("[PCI] PCIe ECAM mapped at virt={:#x}", virt);
                }
                Err(e) => {
                    crate::serial_println!("[PCI] Failed to map ECAM: {} — using legacy PIO only", e);
                }
            }
        }
    }
}


pub fn qhs() -> Option<u64> {
    let virt = NU_.load(core::sync::atomic::Ordering::Relaxed);
    if virt == 0 { None } else { Some(virt) }
}


pub fn cwz(bus: u8, device: u8, function: u8, offset: u16) -> Option<u32> {
    let virt = NU_.load(core::sync::atomic::Ordering::Relaxed);
    if virt == 0 { return None; }
    let start = ADI_.load(core::sync::atomic::Ordering::Relaxed);
    let end = ADH_.load(core::sync::atomic::Ordering::Relaxed);
    if bus < start || bus > end || device > 31 || function > 7 || offset > 4092 {
        return None;
    }
    let addr = virt
        + ((bus - start) as u64) * (32 * 8 * 4096)
        + (device as u64) * (8 * 4096)
        + (function as u64) * 4096
        + (offset & 0xFFC) as u64;
    Some(unsafe { core::ptr::read_volatile(addr as *const u32) })
}


pub fn huw(bus: u8, device: u8, function: u8, offset: u16, value: u32) -> bool {
    let virt = NU_.load(core::sync::atomic::Ordering::Relaxed);
    if virt == 0 { return false; }
    let start = ADI_.load(core::sync::atomic::Ordering::Relaxed);
    let end = ADH_.load(core::sync::atomic::Ordering::Relaxed);
    if bus < start || bus > end || device > 31 || function > 7 || offset > 4092 {
        return false;
    }
    let addr = virt
        + ((bus - start) as u64) * (32 * 8 * 4096)
        + (device as u64) * (8 * 4096)
        + (function as u64) * 4096
        + (offset & 0xFFC) as u64;
    unsafe { core::ptr::write_volatile(addr as *mut u32, value); }
    true
}



pub fn qqe(s: &L, offset: u16) -> u32 {
    if let Some(val) = cwz(s.bus, s.device, s.function, offset) {
        return val;
    }
    
    if offset < 256 {
        return ms(s.bus, s.device, s.function, offset as u8);
    }
    0xFFFFFFFF 
}


pub fn qqf(s: &L, offset: u16, value: u32) {
    if huw(s.bus, s.device, s.function, offset, value) {
        return;
    }
    if offset < 256 {
        qj(s.bus, s.device, s.function, offset as u8, value);
    }
}


pub fn lnl() -> bool {
    NU_.load(core::sync::atomic::Ordering::Relaxed) != 0
}


pub fn ms(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
    let address: u32 = 
        (1 << 31) |                       
        ((bus as u32) << 16) |            
        ((device as u32) << 11) |         
        ((function as u32) << 8) |        
        ((offset as u32) & 0xFC);         
    
    let mut ctj: Port<u32> = Port::new(BEV_);
    let mut zu: Port<u32> = Port::new(BEW_);
    
    unsafe {
        ctj.write(address);
        zu.read()
    }
}


pub fn qj(bus: u8, device: u8, function: u8, offset: u8, value: u32) {
    let address: u32 = 
        (1 << 31) |
        ((bus as u32) << 16) |
        ((device as u32) << 11) |
        ((function as u32) << 8) |
        ((offset as u32) & 0xFC);
    
    let mut ctj: Port<u32> = Port::new(BEV_);
    let mut zu: Port<u32> = Port::new(BEW_);
    
    unsafe {
        ctj.write(address);
        zu.write(value);
    }
}


pub fn vf(bus: u8, device: u8, function: u8, offset: u8) -> u16 {
    let value = ms(bus, device, function, offset & 0xFC);
    ((value >> ((offset & 2) * 8)) & 0xFFFF) as u16
}


pub fn yn(bus: u8, device: u8, function: u8, offset: u8) -> u8 {
    let value = ms(bus, device, function, offset & 0xFC);
    ((value >> ((offset & 3) * 8)) & 0xFF) as u8
}


pub fn qbb(bus: u8, device: u8, function: u8, offset: u8, value: u8) {
    let asw = offset & 0xFC;
    let no = (offset & 3) * 8;
    let qb = ms(bus, device, function, asw);
    let mask = !(0xFFu32 << no);
    let new = (qb & mask) | ((value as u32) << no);
    qj(bus, device, function, asw, new);
}


pub fn qba(bus: u8, device: u8, function: u8, offset: u8, value: u16) {
    let asw = offset & 0xFC;
    let no = (offset & 2) * 8;
    let qb = ms(bus, device, function, asw);
    let mask = !(0xFFFFu32 << no);
    let new = (qb & mask) | ((value as u32) << no);
    qj(bus, device, function, asw, new);
}


fn jdc(bus: u8, device: u8, function: u8) -> Option<L> {
    let jpy = ms(bus, device, function, 0x00);
    let vendor_id = (jpy & 0xFFFF) as u16;
    
    if vendor_id == 0xFFFF || vendor_id == 0x0000 {
        return None;
    }
    
    let device_id = ((jpy >> 16) & 0xFFFF) as u16;
    
    let eib = ms(bus, device, function, 0x08);
    let revision = (eib & 0xFF) as u8;
    let prog_if = ((eib >> 8) & 0xFF) as u8;
    let subclass = ((eib >> 16) & 0xFF) as u8;
    let class_code = ((eib >> 24) & 0xFF) as u8;
    
    let mkq = ms(bus, device, function, 0x0C);
    let header_type = ((mkq >> 16) & 0xFF) as u8;
    
    let iha = ms(bus, device, function, 0x3C);
    let interrupt_line = (iha & 0xFF) as u8;
    let interrupt_pin = ((iha >> 8) & 0xFF) as u8;
    
    
    let mut bar = [0u32; 6];
    for i in 0..6 {
        bar[i] = ms(bus, device, function, 0x10 + (i as u8 * 4));
    }
    
    Some(L {
        bus,
        device,
        function,
        vendor_id,
        device_id,
        class_code,
        subclass,
        prog_if,
        revision,
        header_type,
        interrupt_line,
        interrupt_pin,
        bar,
    })
}


fn okx(bus: u8, device: u8, devices: &mut Vec<L>) {
    if let Some(s) = jdc(bus, device, 0) {
        let nhc = s.is_multifunction();
        devices.push(s);
        
        if nhc {
            for function in 1..8 {
                if let Some(s) = jdc(bus, device, function) {
                    devices.push(s);
                }
            }
        }
    }
}


pub fn scan() -> Vec<L> {
    let mut devices = Vec::new();
    
    
    let mut iuf = false;
    for device in 0..32 {
        let test = ms(0, device, 0, 0);
        if test != 0xFFFFFFFF && test != 0x00000000 {
            iuf = true;
            break;
        }
    }
    
    if !iuf {
        crate::log_warn!("[PCI] No PCI bus detected - scanning anyway...");
    }
    
    
    
    
    let mut dbb: u8 = 0;
    
    for bus in 0..=255u8 {
        let mut hzs = false;
        for device in 0..32 {
            let bak = devices.len();
            okx(bus, device, &mut devices);
            if devices.len() > bak {
                hzs = true;
                
                for s in &devices[bak..] {
                    if s.class_code == class::Jk && s.subclass == 0x04 {
                        
                        let jdv = (crate::pci::ms(s.bus, s.device, s.function, 0x18) >> 8) as u8;
                        let jjk = (crate::pci::ms(s.bus, s.device, s.function, 0x18) >> 16) as u8;
                        if jjk > dbb {
                            dbb = jjk;
                        }
                        if jdv > dbb {
                            dbb = jdv;
                        }
                    }
                }
            }
        }
        
        
        
        if bus >= dbb && bus > 0 && !hzs {
            
            if bus > dbb + 2 {
                break;
            }
        }
    }
    
    devices
}


pub fn init() {
    
    mpd();
    
    let devices = scan();
    let count = devices.len();
    
    crate::log!("[PCI] Found {} devices:", count);
    
    for s in &devices {
        let subclass_name = s.subclass_name();
        if subclass_name.is_empty() {
            crate::log!("[PCI]   {:02X}:{:02X}.{} {:04X}:{:04X} {} ({})",
                s.bus, s.device, s.function,
                s.vendor_id, s.device_id,
                s.class_name(),
                s.vendor_name());
        } else {
            crate::log!("[PCI]   {:02X}:{:02X}.{} {:04X}:{:04X} {} - {} ({})",
                s.bus, s.device, s.function,
                s.vendor_id, s.device_id,
                s.class_name(),
                subclass_name,
                s.vendor_name());
        }
    }
    
    *Hm.lock() = devices;
}


pub fn aqs() -> Vec<L> {
    Hm.lock().clone()
}


pub fn bsp(class_code: u8) -> Vec<L> {
    Hm.lock().iter()
        .filter(|d| d.class_code == class_code)
        .cloned()
        .collect()
}


pub fn qfs(class_code: u8, subclass: u8) -> Vec<L> {
    Hm.lock().iter()
        .filter(|d| d.class_code == class_code && d.subclass == subclass)
        .cloned()
        .collect()
}


pub fn lvs(vendor_id: u16, device_id: u16) -> Option<L> {
    Hm.lock().iter()
        .find(|d| d.vendor_id == vendor_id && d.device_id == device_id)
        .cloned()
}


pub fn qft(class_code: u8) -> Option<L> {
    Hm.lock().iter()
        .find(|d| d.class_code == class_code)
        .cloned()
}


pub fn bzi(s: &L) {
    let command = vf(s.bus, s.device, s.function, 0x04);
    let dvb = command | 0x04; 
    qj(s.bus, s.device, s.function, 0x04, dvb as u32);
    crate::log_debug!("[PCI] Bus mastering enabled for {:02X}:{:02X}.{}", 
        s.bus, s.device, s.function);
}


pub fn bzj(s: &L) {
    let command = vf(s.bus, s.device, s.function, 0x04);
    let dvb = command | 0x02; 
    qj(s.bus, s.device, s.function, 0x04, dvb as u32);
}


pub fn qev(s: &L) {
    let command = vf(s.bus, s.device, s.function, 0x04);
    let dvb = command | 0x01; 
    qj(s.bus, s.device, s.function, 0x04, dvb as u32);
}



pub fn bsq(s: &L, cap_id: u8) -> Option<u8> {
    
    let status = vf(s.bus, s.device, s.function, 0x06);
    if status & (1 << 4) == 0 {
        return None; 
    }
    
    
    let mut qg = yn(s.bus, s.device, s.function, 0x34);
    let mut anc = 0u32;
    
    while qg != 0 && anc < 48 {
        let cap_type = yn(s.bus, s.device, s.function, qg);
        if cap_type == cap_id {
            return Some(qg);
        }
        qg = yn(s.bus, s.device, s.function, qg + 1);
        anc += 1;
    }
    
    None
}



pub fn lwc(s: &L) -> Vec<(u8, u8, u8, u32, u32)> {
    let mut caps = Vec::new();
    
    
    let status = vf(s.bus, s.device, s.function, 0x06);
    if status & (1 << 4) == 0 {
        return caps;
    }
    
    let mut qg = yn(s.bus, s.device, s.function, 0x34);
    let mut anc = 0u32;
    
    while qg != 0 && anc < 48 {
        let cap_type = yn(s.bus, s.device, s.function, qg);
        
        if cap_type == 0x09 { 
            
            
            
            
            
            
            
            
            
            let ehn = yn(s.bus, s.device, s.function, qg + 3);
            let bar = yn(s.bus, s.device, s.function, qg + 4);
            let offset = ms(s.bus, s.device, s.function, qg + 8);
            let length = ms(s.bus, s.device, s.function, qg + 12);
            
            caps.push((qg, ehn, bar, offset, length));
        }
        
        qg = yn(s.bus, s.device, s.function, qg + 1);
        anc += 1;
    }
    
    caps
}


pub fn ocv(s: &L, cap_offset: u8) -> u32 {
    
    ms(s.bus, s.device, s.function, cap_offset + 16)
}


pub fn qkg() -> String {
    let devices = Hm.lock();
    
    let (mut storage, mut network, mut display, mut usb, mut bridges) = (0, 0, 0, 0, 0);
    for d in devices.iter() {
        match d.class_code {
            class::FZ_ => storage += 1,
            class::Gr => network += 1,
            class::Du => display += 1,
            class::Jk => bridges += 1,
            class::QG_ if d.subclass == serial::Qs => usb += 1,
            _ => {}
        }
    }
    
    format!(
        "PCI: {} devices (Storage:{}, Network:{}, Display:{}, USB:{}, Bridges:{})",
        devices.len(), storage, network, display, usb, bridges
    )
}






pub mod cap_id {
    pub const Mp: u8 = 0x05;
    pub const Pl: u8 = 0x11;
    pub const Azy: u8 = 0x10;
}



pub fn ios(dest_apic_id: u8) -> u32 {
    0xFEE0_0000 | ((dest_apic_id as u32) << 12)
}



pub fn iot(vector: u8) -> u32 {
    vector as u32 
}



pub fn lpr(s: &L, vector: u8) -> Option<u8> {
    let wa = bsq(s, cap_id::Mp)?;
    
    
    let akd = vf(s.bus, s.device, s.function, wa + 2);
    let arf = (akd & (1 << 7)) != 0;
    
    
    let lac = akd & !(1u16 << 0); 
    qj(s.bus, s.device, s.function, (wa + 2) & 0xFC, 
        (ms(s.bus, s.device, s.function, (wa + 2) & 0xFC) 
            & !(0xFFFF << (((wa + 2) & 2) * 8)))
            | ((lac as u32) << (((wa + 2) & 2) * 8)));
    
    
    let addr = ios(0); 
    qj(s.bus, s.device, s.function, wa + 4, addr);
    
    
    let data_offset = if arf {
        
        qj(s.bus, s.device, s.function, wa + 8, 0);
        wa + 12
    } else {
        wa + 8
    };
    
    let data = iot(vector);
    
    let ku = ms(s.bus, s.device, s.function, data_offset & 0xFC);
    let no = ((data_offset & 2) * 8) as u32;
    let mask = !(0xFFFF << no);
    let njv = (ku & mask) | ((data as u32) << no);
    qj(s.bus, s.device, s.function, data_offset & 0xFC, njv);
    
    
    let giy = (akd & !(0x7 << 4)) | (1 << 0); 
    let kzx = ms(s.bus, s.device, s.function, (wa + 2) & 0xFC);
    let chw = ((wa + 2) & 2) * 8;
    let laa = !(0xFFFF << chw);
    let lad = (kzx & laa as u32) | ((giy as u32) << chw);
    qj(s.bus, s.device, s.function, (wa + 2) & 0xFC, lad);
    
    
    let cmd = vf(s.bus, s.device, s.function, 0x04);
    qj(s.bus, s.device, s.function, 0x04, (cmd | (1 << 10)) as u32);
    
    crate::serial_println!("[PCI] MSI enabled for {:02X}:{:02X}.{} vector={} {}",
        s.bus, s.device, s.function, vector,
        if arf { "64-bit" } else { "32-bit" });
    
    Some(wa)
}


pub fn lps(s: &L, vector: u8) -> Option<u8> {
    let wa = bsq(s, cap_id::Pl)?;
    
    
    let akd = vf(s.bus, s.device, s.function, wa + 2);
    let fci = (akd & 0x7FF) + 1;
    
    
    let jlh = ms(s.bus, s.device, s.function, wa + 4);
    let jlf = (jlh & 0x7) as usize;
    let pcv = (jlh & !0x7) as u64;
    
    
    let fib = match s.bar_address(jlf) {
        Some(a) => a,
        None => {
            crate::serial_println!("[PCI] MSI-X: BAR{} not configured", jlf);
            return None;
        }
    };
    
    
    let asj = fib + pcv;
    let pcw = (fci as usize) * 16;
    let bwf = match crate::memory::yv(asj, pcw.max(4096)) {
        Ok(v) => v,
        Err(e) => {
            crate::serial_println!("[PCI] MSI-X: Failed to map table: {}", e);
            return None;
        }
    };
    
    
    let hpg = ms(s.bus, s.device, s.function, (wa + 2) & 0xFC);
    let chw = ((wa + 2) & 2) * 8;
    
    let niw = (akd | (1 << 15) | (1 << 14)) as u32;
    let masked = (hpg & !(0xFFFF << chw)) | (niw << chw);
    qj(s.bus, s.device, s.function, (wa + 2) & 0xFC, masked);
    
    
    let cxg = bwf;
    unsafe {
        
        core::ptr::write_volatile(cxg as *mut u32, ios(0));
        
        core::ptr::write_volatile((cxg + 4) as *mut u32, 0);
        
        core::ptr::write_volatile((cxg + 8) as *mut u32, iot(vector));
        
        core::ptr::write_volatile((cxg + 12) as *mut u32, 0);
    }
    
    
    let pps = (akd | (1 << 15)) & !(1 << 14);
    let lvp = (hpg & !(0xFFFF << chw)) | ((pps as u32) << chw);
    qj(s.bus, s.device, s.function, (wa + 2) & 0xFC, lvp);
    
    
    let cmd = vf(s.bus, s.device, s.function, 0x04);
    qj(s.bus, s.device, s.function, 0x04, (cmd | (1 << 10)) as u32);
    
    crate::serial_println!("[PCI] MSI-X enabled for {:02X}:{:02X}.{} vector={} table_size={}",
        s.bus, s.device, s.function, vector, fci);
    
    Some(wa)
}


pub fn qey(s: &L, vector: u8) -> bool {
    if lps(s, vector).is_some() {
        return true;
    }
    if lpr(s, vector).is_some() {
        return true;
    }
    false
}


pub fn qda(s: &L) {
    if let Some(wa) = bsq(s, cap_id::Mp) {
        let akd = vf(s.bus, s.device, s.function, wa + 2);
        let giy = akd & !(1u16 << 0);
        let ku = ms(s.bus, s.device, s.function, (wa + 2) & 0xFC);
        let no = ((wa + 2) & 2) * 8;
        let mask = !(0xFFFF << no);
        qj(s.bus, s.device, s.function, (wa + 2) & 0xFC,
            (ku & mask as u32) | ((giy as u32) << no));
    }
}


pub fn qkn(s: &L) -> (bool, bool) {
    let ngq = bsq(s, cap_id::Mp).is_some();
    let ngr = bsq(s, cap_id::Pl).is_some();
    (ngq, ngr)
}








pub fn pyl(s: &L, bar_index: usize) -> u64 {
    if bar_index >= 6 {
        return 0;
    }
    let bku = (0x10 + bar_index * 4) as u8;
    let ccb = ms(s.bus, s.device, s.function, bku);
    
    if ccb == 0 {
        return 0; 
    }
    
    let czy = ccb & 1 != 0;
    let arf = !czy && ((ccb >> 1) & 0x3) == 2;
    
    
    let cmd = vf(s.bus, s.device, s.function, 0x04);
    qj(s.bus, s.device, s.function, 0x04, (cmd & !0x03) as u32);
    
    
    qj(s.bus, s.device, s.function, bku, 0xFFFFFFFF);
    let agx = ms(s.bus, s.device, s.function, bku);
    
    qj(s.bus, s.device, s.function, bku, ccb);
    
    let size = if czy {
        let mask = agx & 0xFFFFFFFC;
        if mask == 0 { 0 } else { ((!mask) + 1) as u64 & 0xFFFF }
    } else if arf && bar_index < 5 {
        let ege = (0x10 + (bar_index + 1) * 4) as u8;
        let gle = ms(s.bus, s.device, s.function, ege);
        qj(s.bus, s.device, s.function, ege, 0xFFFFFFFF);
        let gqk = ms(s.bus, s.device, s.function, ege);
        qj(s.bus, s.device, s.function, ege, gle);
        
        let xo = ((gqk as u64) << 32) | (agx & 0xFFFFFFF0) as u64;
        if xo == 0 { 0 } else { (!xo).wrapping_add(1) }
    } else {
        let mask = agx & 0xFFFFFFF0;
        if mask == 0 { 0 } else { ((!mask) + 1) as u64 }
    };
    
    
    qj(s.bus, s.device, s.function, 0x04, cmd as u32);
    
    size
}
