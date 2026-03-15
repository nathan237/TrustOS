




use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use crate::arch::Port;
use spin::Mutex;


const BCS_: u16 = 0xCF8;
const BCT_: u16 = 0xCFC;


pub mod class {
    pub const Cot: u8 = 0x00;
    pub const FK_: u8 = 0x01;
    pub const Qa: u8 = 0x02;
    pub const Ji: u8 = 0x03;
    pub const Blx: u8 = 0x04;
    pub const Chb: u8 = 0x05;
    pub const Vl: u8 = 0x06;
    pub const CTI_: u8 = 0x07;
    pub const BKY_: u8 = 0x08;
    pub const Cfl: u8 = 0x09;
    pub const Cal: u8 = 0x0A;
    pub const Civ: u8 = 0x0B;
    pub const PJ_: u8 = 0x0C;
    pub const Bwi: u8 = 0x0D;
    pub const Cfm: u8 = 0x0E;
    pub const Clb: u8 = 0x0F;
    pub const Cbg: u8 = 0x10;
    pub const CTH_: u8 = 0x11;
}


pub mod storage {
    pub const Clc: u8 = 0x00;
    pub const Cff: u8 = 0x01;
    pub const Cww: u8 = 0x02;
    pub const Cze: u8 = 0x03;
    pub const Cje: u8 = 0x04;
    pub const Bxl: u8 = 0x05;
    pub const Cla: u8 = 0x06;
    pub const Dgj: u8 = 0x07;
    pub const Cht: u8 = 0x08;  
}


pub mod network {
    pub const Cbm: u8 = 0x00;
    pub const EIL_: u8 = 0x01;
    pub const Cws: u8 = 0x02;
    pub const Crf: u8 = 0x03;
    pub const Czi: u8 = 0x04;
    pub const Ddr: u8 = 0x06;
    pub const Bjb: u8 = 0x07;
}


pub mod bridge {
    pub const Ces: u8 = 0x00;
    pub const Cfr: u8 = 0x01;
    pub const Cut: u8 = 0x02;
    pub const Dbn: u8 = 0x03;
    pub const CIU_: u8 = 0x04;
    pub const Ddp: u8 = 0x05;
    pub const Dcu: u8 = 0x06;
    pub const Csj: u8 = 0x07;
    pub const Dfd: u8 = 0x08;
    pub const DZF_: u8 = 0x09;
    pub const DQZ_: u8 = 0x0A;
}


pub mod serial {
    pub const Cwu: u8 = 0x00;
    pub const Aos: u8 = 0x01;
    pub const Dhv: u8 = 0x02;
    pub const Any: u8 = 0x03;
    pub const Cwt: u8 = 0x04;
    pub const Cma: u8 = 0x05;
    pub const Bjb: u8 = 0x06;
    pub const Czf: u8 = 0x07;
}


pub mod usb {
    pub const Buz: u8 = 0x00;
    pub const Bnp: u8 = 0x10;
    pub const Ark: u8 = 0x20;
    pub const Bbe: u8 = 0x30;
}


#[derive(Debug, Clone)]
pub struct S {
    pub aq: u8,
    pub de: u8,
    pub gw: u8,
    pub ml: u16,
    pub mx: u16,
    pub ajz: u8,
    pub adl: u8,
    pub frg: u8,
    pub afe: u8,
    pub lbw: u8,
    pub esw: u8,
    pub jan: u8,
    pub bar: [u32; 6],
}

impl S {
    
    pub fn bpz(&self) -> &'static str {
        match self.ajz {
            class::Cot => "Unclassified",
            class::FK_ => "Mass Storage",
            class::Qa => "Network Controller",
            class::Ji => "Display Controller",
            class::Blx => "Multimedia",
            class::Chb => "Memory Controller",
            class::Vl => "Bridge",
            class::CTI_ => "Communication",
            class::BKY_ => "Peripheral",
            class::Cfl => "Input Device",
            class::Cal => "Docking Station",
            class::Civ => "Processor",
            class::PJ_ => "Serial Bus",
            class::Bwi => "Wireless",
            class::Cfm => "Intelligent I/O",
            class::Clb => "Satellite",
            class::Cbg => "Encryption",
            class::CTH_ => "Signal Processing",
            _ => "Unknown",
        }
    }
    
    
    pub fn bor(&self) -> &'static str {
        match (self.ajz, self.adl) {
            
            (class::FK_, storage::Cff) => "IDE Controller",
            (class::FK_, storage::Cla) => "SATA Controller",
            (class::FK_, storage::Cht) => "NVMe Controller",
            (class::FK_, storage::Cje) => "RAID Controller",
            (class::FK_, storage::Clc) => "SCSI Controller",
            (class::FK_, storage::Bxl) => "ATA Controller",
            
            
            (class::Qa, network::Cbm) => "Ethernet",
            (class::Qa, network::Bjb) => "InfiniBand",
            
            
            (class::Ji, 0x00) => "VGA Compatible",
            (class::Ji, 0x01) => "XGA Controller",
            (class::Ji, 0x02) => "3D Controller",
            
            
            (class::Vl, bridge::Ces) => "Host Bridge",
            (class::Vl, bridge::Cfr) => "ISA Bridge",
            (class::Vl, bridge::CIU_) => "PCI-to-PCI Bridge",
            
            
            (class::PJ_, serial::Any) => match self.frg {
                usb::Buz => "USB UHCI",
                usb::Bnp => "USB OHCI",
                usb::Ark => "USB 2.0 EHCI",
                usb::Bbe => "USB 3.0 xHCI",
                0xFE => "USB Device",
                _ => "USB Controller",
            },
            (class::PJ_, serial::Cma) => "SMBus",
            
            _ => "",
        }
    }
    
    
    pub fn cip(&self) -> &'static str {
        match self.ml {
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
    
    
    pub fn tyh(&self) -> bool {
        self.lbw & 0x80 != 0
    }
    
    
    pub fn cje(&self, index: usize) -> Option<u64> {
        if index >= 6 {
            return None;
        }
        
        let bar = self.bar[index];
        if bar == 0 {
            return None;
        }
        
        
        if bar & 1 == 0 {
            
            let gzq = (bar >> 1) & 0x3;
            match gzq {
                0 => Some((bar & 0xFFFFFFF0) as u64), 
                2 if index < 5 => {
                    
                    let afq = self.bar[index + 1] as u64;
                    Some(((afq << 32) | (bar & 0xFFFFFFF0) as u64))
                }
                _ => None,
            }
        } else {
            
            Some((bar & 0xFFFFFFFC) as u64)
        }
    }
    
    
    pub fn mxx(&self, index: usize) -> bool {
        if index >= 6 {
            return false;
        }
        self.bar[index] & 1 == 0
    }
    
    
    pub fn yfl(&self, index: usize) -> bool {
        if index >= 6 {
            return false;
        }
        self.bar[index] & 1 != 0
    }
}


static Ry: Mutex<Vec<S>> = Mutex::new(Vec::new());


static BSW_: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);

static SV_: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);

static ABS_: core::sync::atomic::AtomicU8 = core::sync::atomic::AtomicU8::new(0);

static ABR_: core::sync::atomic::AtomicU8 = core::sync::atomic::AtomicU8::new(0);


fn ttk() {
    if let Some(co) = crate::acpi::ani() {
        if let Some(fv) = co.eut.fv() {
            let ar = fv.bps;
            let aw = fv.aw() as usize;
            let cca = fv.cca;
            let cej = fv.cej;
            
            crate::serial_println!("[PCI] PCIe ECAM detected: base={:#x} size={:#x} buses={}-{}",
                ar, aw, cca, cej);
            
            match crate::memory::bki(ar, aw) {
                Ok(ju) => {
                    BSW_.store(ar, core::sync::atomic::Ordering::SeqCst);
                    SV_.store(ju, core::sync::atomic::Ordering::SeqCst);
                    ABS_.store(cca, core::sync::atomic::Ordering::SeqCst);
                    ABR_.store(cej, core::sync::atomic::Ordering::SeqCst);
                    crate::serial_println!("[PCI] PCIe ECAM mapped at virt={:#x}", ju);
                }
                Err(aa) => {
                    crate::serial_println!("[PCI] Failed to map ECAM: {} — using legacy PIO only", aa);
                }
            }
        }
    }
}


pub fn sij(aq: u8, de: u8, gw: u8, l: u16) -> Option<u32> {
    let ju = SV_.load(core::sync::atomic::Ordering::Relaxed);
    if ju == 0 { return None; }
    let ay = ABS_.load(core::sync::atomic::Ordering::Relaxed);
    let ci = ABR_.load(core::sync::atomic::Ordering::Relaxed);
    if aq < ay || aq > ci || de > 31 || gw > 7 || l > 4092 {
        return None;
    }
    let ag = ju
        + ((aq - ay) as u64) * (32 * 8 * 4096)
        + (de as u64) * (8 * 4096)
        + (gw as u64) * 4096
        + (l & 0xFFC) as u64;
    Some(unsafe { core::ptr::read_volatile(ag as *const u32) })
}


pub fn sik(aq: u8, de: u8, gw: u8, l: u16, bn: u32) -> bool {
    let ju = SV_.load(core::sync::atomic::Ordering::Relaxed);
    if ju == 0 { return false; }
    let ay = ABS_.load(core::sync::atomic::Ordering::Relaxed);
    let ci = ABR_.load(core::sync::atomic::Ordering::Relaxed);
    if aq < ay || aq > ci || de > 31 || gw > 7 || l > 4092 {
        return false;
    }
    let ag = ju
        + ((aq - ay) as u64) * (32 * 8 * 4096)
        + (de as u64) * (8 * 4096)
        + (gw as u64) * 4096
        + (l & 0xFFC) as u64;
    unsafe { core::ptr::write_volatile(ag as *mut u32, bn); }
    true
}



pub fn zex(ba: &S, l: u16) -> u32 {
    if let Some(ap) = sij(ba.aq, ba.de, ba.gw, l) {
        return ap;
    }
    
    if l < 256 {
        return aon(ba.aq, ba.de, ba.gw, l as u8);
    }
    0xFFFFFFFF 
}


pub fn zey(ba: &S, l: u16, bn: u32) {
    if sik(ba.aq, ba.de, ba.gw, l, bn) {
        return;
    }
    if l < 256 {
        aso(ba.aq, ba.de, ba.gw, l as u8, bn);
    }
}


pub fn yog() -> bool {
    SV_.load(core::sync::atomic::Ordering::Relaxed) != 0
}


pub fn aon(aq: u8, de: u8, gw: u8, l: u8) -> u32 {
    let re: u32 = 
        (1 << 31) |                       
        ((aq as u32) << 16) |            
        ((de as u32) << 11) |         
        ((gw as u32) << 8) |        
        ((l as u32) & 0xFC);         
    
    let mut fzv: Port<u32> = Port::new(BCS_);
    let mut axr: Port<u32> = Port::new(BCT_);
    
    unsafe {
        fzv.write(re);
        axr.read()
    }
}


pub fn aso(aq: u8, de: u8, gw: u8, l: u8, bn: u32) {
    let re: u32 = 
        (1 << 31) |
        ((aq as u32) << 16) |
        ((de as u32) << 11) |
        ((gw as u32) << 8) |
        ((l as u32) & 0xFC);
    
    let mut fzv: Port<u32> = Port::new(BCS_);
    let mut axr: Port<u32> = Port::new(BCT_);
    
    unsafe {
        fzv.write(re);
        axr.write(bn);
    }
}


pub fn byw(aq: u8, de: u8, gw: u8, l: u8) -> u16 {
    let bn = aon(aq, de, gw, l & 0xFC);
    ((bn >> ((l & 2) * 8)) & 0xFFFF) as u16
}


pub fn enw(aq: u8, de: u8, gw: u8, l: u8) -> u8 {
    let bn = aon(aq, de, gw, l & 0xFC);
    ((bn >> ((l & 3) * 8)) & 0xFF) as u8
}


pub fn yju(aq: u8, de: u8, gw: u8, l: u8, bn: u8) {
    let ciz = l & 0xFC;
    let acn = (l & 3) * 8;
    let aft = aon(aq, de, gw, ciz);
    let hs = !(0xFFu32 << acn);
    let new = (aft & hs) | ((bn as u32) << acn);
    aso(aq, de, gw, ciz, new);
}


pub fn yjt(aq: u8, de: u8, gw: u8, l: u8, bn: u16) {
    let ciz = l & 0xFC;
    let acn = (l & 2) * 8;
    let aft = aon(aq, de, gw, ciz);
    let hs = !(0xFFFFu32 << acn);
    let new = (aft & hs) | ((bn as u32) << acn);
    aso(aq, de, gw, ciz, new);
}


fn pgc(aq: u8, de: u8, gw: u8) -> Option<S> {
    let pyc = aon(aq, de, gw, 0x00);
    let ml = (pyc & 0xFFFF) as u16;
    
    if ml == 0xFFFF || ml == 0x0000 {
        return None;
    }
    
    let mx = ((pyc >> 16) & 0xFFFF) as u16;
    
    let inv = aon(aq, de, gw, 0x08);
    let afe = (inv & 0xFF) as u8;
    let frg = ((inv >> 8) & 0xFF) as u8;
    let adl = ((inv >> 16) & 0xFF) as u8;
    let ajz = ((inv >> 24) & 0xFF) as u8;
    
    let tnz = aon(aq, de, gw, 0x0C);
    let lbw = ((tnz >> 16) & 0xFF) as u8;
    
    let oew = aon(aq, de, gw, 0x3C);
    let esw = (oew & 0xFF) as u8;
    let jan = ((oew >> 8) & 0xFF) as u8;
    
    
    let mut bar = [0u32; 6];
    for a in 0..6 {
        bar[a] = aon(aq, de, gw, 0x10 + (a as u8 * 4));
    }
    
    Some(S {
        aq,
        de,
        gw,
        ml,
        mx,
        ajz,
        adl,
        frg,
        afe,
        lbw,
        esw,
        jan,
        bar,
    })
}


fn wdp(aq: u8, de: u8, ik: &mut Vec<S>) {
    if let Some(ba) = pgc(aq, de, 0) {
        let uqo = ba.tyh();
        ik.push(ba);
        
        if uqo {
            for gw in 1..8 {
                if let Some(ba) = pgc(aq, de, gw) {
                    ik.push(ba);
                }
            }
        }
    }
}


pub fn arx() -> Vec<S> {
    let mut ik = Vec::new();
    
    
    let mut ouy = false;
    for de in 0..32 {
        let test = aon(0, de, 0, 0);
        if test != 0xFFFFFFFF && test != 0x00000000 {
            ouy = true;
            break;
        }
    }
    
    if !ouy {
        crate::log_warn!("[PCI] No PCI bus detected - scanning anyway...");
    }
    
    
    
    
    let mut gmf: u8 = 0;
    
    for aq in 0..=255u8 {
        let mut nvu = false;
        for de in 0..32 {
            let cvu = ik.len();
            wdp(aq, de, &mut ik);
            if ik.len() > cvu {
                nvu = true;
                
                for ba in &ik[cvu..] {
                    if ba.ajz == class::Vl && ba.adl == 0x04 {
                        
                        let pgz = (crate::pci::aon(ba.aq, ba.de, ba.gw, 0x18) >> 8) as u8;
                        let ppi = (crate::pci::aon(ba.aq, ba.de, ba.gw, 0x18) >> 16) as u8;
                        if ppi > gmf {
                            gmf = ppi;
                        }
                        if pgz > gmf {
                            gmf = pgz;
                        }
                    }
                }
            }
        }
        
        
        
        if aq >= gmf && aq > 0 && !nvu {
            
            if aq > gmf + 2 {
                break;
            }
        }
    }
    
    ik
}


pub fn init() {
    
    ttk();
    
    let ik = arx();
    let az = ik.len();
    
    crate::log!("[PCI] Found {} devices:", az);
    
    for ba in &ik {
        let bor = ba.bor();
        if bor.is_empty() {
            crate::log!("[PCI]   {:02X}:{:02X}.{} {:04X}:{:04X} {} ({})",
                ba.aq, ba.de, ba.gw,
                ba.ml, ba.mx,
                ba.bpz(),
                ba.cip());
        } else {
            crate::log!("[PCI]   {:02X}:{:02X}.{} {:04X}:{:04X} {} - {} ({})",
                ba.aq, ba.de, ba.gw,
                ba.ml, ba.mx,
                ba.bpz(),
                bor,
                ba.cip());
        }
    }
    
    *Ry.lock() = ik;
}


pub fn fjm() -> Vec<S> {
    Ry.lock().clone()
}


pub fn ebq(ajz: u8) -> Vec<S> {
    Ry.lock().iter()
        .hi(|bc| bc.ajz == ajz)
        .abn()
        .collect()
}


pub fn yqq(ajz: u8, adl: u8) -> Vec<S> {
    Ry.lock().iter()
        .hi(|bc| bc.ajz == ajz && bc.adl == adl)
        .abn()
        .collect()
}


pub fn sta(ml: u16, mx: u16) -> Option<S> {
    Ry.lock().iter()
        .du(|bc| bc.ml == ml && bc.mx == mx)
        .abn()
}


pub fn yqr(ajz: u8) -> Option<S> {
    Ry.lock().iter()
        .du(|bc| bc.ajz == ajz)
        .abn()
}


pub fn fhp(ba: &S) {
    let ro = byw(ba.aq, ba.de, ba.gw, 0x04);
    let hsp = ro | 0x04; 
    aso(ba.aq, ba.de, ba.gw, 0x04, hsp as u32);
    crate::log_debug!("[PCI] Bus mastering enabled for {:02X}:{:02X}.{}", 
        ba.aq, ba.de, ba.gw);
}


pub fn fhq(ba: &S) {
    let ro = byw(ba.aq, ba.de, ba.gw, 0x04);
    let hsp = ro | 0x02; 
    aso(ba.aq, ba.de, ba.gw, 0x04, hsp as u32);
}


pub fn ypd(ba: &S) {
    let ro = byw(ba.aq, ba.de, ba.gw, 0x04);
    let hsp = ro | 0x01; 
    aso(ba.aq, ba.de, ba.gw, 0x04, hsp as u32);
}



pub fn ebr(ba: &S, cap_id: u8) -> Option<u8> {
    
    let status = byw(ba.aq, ba.de, ba.gw, 0x06);
    if status & (1 << 4) == 0 {
        return None; 
    }
    
    
    let mut cdq = enw(ba.aq, ba.de, ba.gw, 0x34);
    let mut bxs = 0u32;
    
    while cdq != 0 && bxs < 48 {
        let cap_type = enw(ba.aq, ba.de, ba.gw, cdq);
        if cap_type == cap_id {
            return Some(cdq);
        }
        cdq = enw(ba.aq, ba.de, ba.gw, cdq + 1);
        bxs += 1;
    }
    
    None
}



pub fn stp(ba: &S) -> Vec<(u8, u8, u8, u32, u32)> {
    let mut dr = Vec::new();
    
    
    let status = byw(ba.aq, ba.de, ba.gw, 0x06);
    if status & (1 << 4) == 0 {
        return dr;
    }
    
    let mut cdq = enw(ba.aq, ba.de, ba.gw, 0x34);
    let mut bxs = 0u32;
    
    while cdq != 0 && bxs < 48 {
        let cap_type = enw(ba.aq, ba.de, ba.gw, cdq);
        
        if cap_type == 0x09 { 
            
            
            
            
            
            
            
            
            
            let ind = enw(ba.aq, ba.de, ba.gw, cdq + 3);
            let bar = enw(ba.aq, ba.de, ba.gw, cdq + 4);
            let l = aon(ba.aq, ba.de, ba.gw, cdq + 8);
            let go = aon(ba.aq, ba.de, ba.gw, cdq + 12);
            
            dr.push((cdq, ind, bar, l, go));
        }
        
        cdq = enw(ba.aq, ba.de, ba.gw, cdq + 1);
        bxs += 1;
    }
    
    dr
}


pub fn vsd(ba: &S, kgl: u8) -> u32 {
    
    aon(ba.aq, ba.de, ba.gw, kgl + 16)
}


pub fn ywc() -> String {
    let ik = Ry.lock();
    
    let (mut storage, mut network, mut display, mut usb, mut nac) = (0, 0, 0, 0, 0);
    for bc in ik.iter() {
        match bc.ajz {
            class::FK_ => storage += 1,
            class::Qa => network += 1,
            class::Ji => display += 1,
            class::Vl => nac += 1,
            class::PJ_ if bc.adl == serial::Any => usb += 1,
            _ => {}
        }
    }
    
    format!(
        "PCI: {} devices (Storage:{}, Network:{}, Display:{}, USB:{}, Bridges:{})",
        ik.len(), storage, network, display, usb, nac
    )
}






pub mod cap_id {
    pub const Acu: u8 = 0x05;
    pub const Akb: u8 = 0x11;
    pub const Ddo: u8 = 0x10;
}



pub fn ook(rwg: u8) -> u32 {
    0xFEE0_0000 | ((rwg as u32) << 12)
}



pub fn ool(wj: u8) -> u32 {
    wj as u32 
}



pub fn sla(ba: &S, wj: u8) -> Option<u8> {
    let aqv = ebr(ba, cap_id::Acu)?;
    
    
    let efo = byw(ba.aq, ba.de, ba.gw, aqv + 2);
    let edt = (efo & (1 << 7)) != 0;
    
    
    let rrj = efo & !(1u16 << 0); 
    aso(ba.aq, ba.de, ba.gw, (aqv + 2) & 0xFC, 
        (aon(ba.aq, ba.de, ba.gw, (aqv + 2) & 0xFC) 
            & !(0xFFFF << (((aqv + 2) & 2) * 8)))
            | ((rrj as u32) << (((aqv + 2) & 2) * 8)));
    
    
    let ag = ook(0); 
    aso(ba.aq, ba.de, ba.gw, aqv + 4, ag);
    
    
    let bbj = if edt {
        
        aso(ba.aq, ba.de, ba.gw, aqv + 8, 0);
        aqv + 12
    } else {
        aqv + 8
    };
    
    let f = ool(wj);
    
    let xy = aon(ba.aq, ba.de, ba.gw, bbj & 0xFC);
    let acn = ((bbj & 2) * 8) as u32;
    let hs = !(0xFFFF << acn);
    let uty = (xy & hs) | ((f as u32) << acn);
    aso(ba.aq, ba.de, ba.gw, bbj & 0xFC, uty);
    
    
    let hsq = (efo & !(0x7 << 4)) | (1 << 0); 
    let rrg = aon(ba.aq, ba.de, ba.gw, (aqv + 2) & 0xFC);
    let fgd = ((aqv + 2) & 2) * 8;
    let rri = !(0xFFFF << fgd);
    let rrk = (rrg & rri as u32) | ((hsq as u32) << fgd);
    aso(ba.aq, ba.de, ba.gw, (aqv + 2) & 0xFC, rrk);
    
    
    let cmd = byw(ba.aq, ba.de, ba.gw, 0x04);
    aso(ba.aq, ba.de, ba.gw, 0x04, (cmd | (1 << 10)) as u32);
    
    crate::serial_println!("[PCI] MSI enabled for {:02X}:{:02X}.{} vector={} {}",
        ba.aq, ba.de, ba.gw, wj,
        if edt { "64-bit" } else { "32-bit" });
    
    Some(aqv)
}


pub fn slb(ba: &S, wj: u8) -> Option<u8> {
    let aqv = ebr(ba, cap_id::Akb)?;
    
    
    let efo = byw(ba.aq, ba.de, ba.gw, aqv + 2);
    let prp = (efo & 0x7FF) + 1;
    
    
    let pro = aon(ba.aq, ba.de, ba.gw, aqv + 4);
    let prm = (pro & 0x7) as usize;
    let xak = (pro & !0x7) as u64;
    
    
    let kbx = match ba.cje(prm) {
        Some(q) => q,
        None => {
            crate::serial_println!("[PCI] MSI-X: BAR{} not configured", prm);
            return None;
        }
    };
    
    
    let cig = kbx + xak;
    let xal = (prp as usize) * 16;
    let ejk = match crate::memory::bki(cig, xal.am(4096)) {
        Ok(p) => p,
        Err(aa) => {
            crate::serial_println!("[PCI] MSI-X: Failed to map table: {}", aa);
            return None;
        }
    };
    
    
    let nhv = aon(ba.aq, ba.de, ba.gw, (aqv + 2) & 0xFC);
    let fgd = ((aqv + 2) & 2) * 8;
    
    let usr = (efo | (1 << 15) | (1 << 14)) as u32;
    let bnm = (nhv & !(0xFFFF << fgd)) | (usr << fgd);
    aso(ba.aq, ba.de, ba.gw, (aqv + 2) & 0xFC, bnm);
    
    
    let ggi = ejk;
    unsafe {
        
        core::ptr::write_volatile(ggi as *mut u32, ook(0));
        
        core::ptr::write_volatile((ggi + 4) as *mut u32, 0);
        
        core::ptr::write_volatile((ggi + 8) as *mut u32, ool(wj));
        
        core::ptr::write_volatile((ggi + 12) as *mut u32, 0);
    }
    
    
    let xok = (efo | (1 << 15)) & !(1 << 14);
    let ssv = (nhv & !(0xFFFF << fgd)) | ((xok as u32) << fgd);
    aso(ba.aq, ba.de, ba.gw, (aqv + 2) & 0xFC, ssv);
    
    
    let cmd = byw(ba.aq, ba.de, ba.gw, 0x04);
    aso(ba.aq, ba.de, ba.gw, 0x04, (cmd | (1 << 10)) as u32);
    
    crate::serial_println!("[PCI] MSI-X enabled for {:02X}:{:02X}.{} vector={} table_size={}",
        ba.aq, ba.de, ba.gw, wj, prp);
    
    Some(aqv)
}


pub fn ypg(ba: &S, wj: u8) -> bool {
    if slb(ba, wj).is_some() {
        return true;
    }
    if sla(ba, wj).is_some() {
        return true;
    }
    false
}


pub fn yme(ba: &S) {
    if let Some(aqv) = ebr(ba, cap_id::Acu) {
        let efo = byw(ba.aq, ba.de, ba.gw, aqv + 2);
        let hsq = efo & !(1u16 << 0);
        let xy = aon(ba.aq, ba.de, ba.gw, (aqv + 2) & 0xFC);
        let acn = ((aqv + 2) & 2) * 8;
        let hs = !(0xFFFF << acn);
        aso(ba.aq, ba.de, ba.gw, (aqv + 2) & 0xFC,
            (xy & hs as u32) | ((hsq as u32) << acn));
    }
}


pub fn ywi(ba: &S) -> (bool, bool) {
    let uqg = ebr(ba, cap_id::Acu).is_some();
    let uqh = ebr(ba, cap_id::Akb).is_some();
    (uqg, uqh)
}








pub fn yfm(ba: &S, fda: usize) -> u64 {
    if fda >= 6 {
        return 0;
    }
    let doi = (0x10 + fda * 4) as u8;
    let evs = aon(ba.aq, ba.de, ba.gw, doi);
    
    if evs == 0 {
        return 0; 
    }
    
    let ogd = evs & 1 != 0;
    let edt = !ogd && ((evs >> 1) & 0x3) == 2;
    
    
    let cmd = byw(ba.aq, ba.de, ba.gw, 0x04);
    aso(ba.aq, ba.de, ba.gw, 0x04, (cmd & !0x03) as u32);
    
    
    aso(ba.aq, ba.de, ba.gw, doi, 0xFFFFFFFF);
    let bky = aon(ba.aq, ba.de, ba.gw, doi);
    
    aso(ba.aq, ba.de, ba.gw, doi, evs);
    
    let aw = if ogd {
        let hs = bky & 0xFFFFFFFC;
        if hs == 0 { 0 } else { ((!hs) + 1) as u64 & 0xFFFF }
    } else if edt && fda < 5 {
        let ikq = (0x10 + (fda + 1) * 4) as u8;
        let lqw = aon(ba.aq, ba.de, ba.gw, ikq);
        aso(ba.aq, ba.de, ba.gw, ikq, 0xFFFFFFFF);
        let lxy = aon(ba.aq, ba.de, ba.gw, ikq);
        aso(ba.aq, ba.de, ba.gw, ikq, lqw);
        
        let auh = ((lxy as u64) << 32) | (bky & 0xFFFFFFF0) as u64;
        if auh == 0 { 0 } else { (!auh).cn(1) }
    } else {
        let hs = bky & 0xFFFFFFF0;
        if hs == 0 { 0 } else { ((!hs) + 1) as u64 }
    };
    
    
    aso(ba.aq, ba.de, ba.gw, 0x04, cmd as u32);
    
    aw
}
