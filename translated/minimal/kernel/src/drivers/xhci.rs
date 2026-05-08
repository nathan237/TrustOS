














use alloc::vec::Vec;
use alloc::vec;
use alloc::boxed::Box;
use alloc::string::String;
use core::ptr;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex;






#[repr(C)]
pub struct Qw {
    pub caplength: u8,          
    pub _reserved: u8,          
    pub hciversion: u16,        
    pub hcsparams1: u32,        
    pub hcsparams2: u32,        
    pub hcsparams3: u32,        
    pub hccparams1: u32,        
    pub dboff: u32,             
    pub rtsoff: u32,            
    pub hccparams2: u32,        
}

impl Qw {
    pub fn max_slots(&self) -> u8 {
        (self.hcsparams1 & 0xFF) as u8
    }
    
    pub fn qov(&self) -> u16 {
        ((self.hcsparams1 >> 8) & 0x7FF) as u16
    }
    
    pub fn max_ports(&self) -> u8 {
        ((self.hcsparams1 >> 24) & 0xFF) as u8
    }
    
    pub fn context_size(&self) -> usize {
        if (self.hccparams1 & (1 << 2)) != 0 { 64 } else { 32 }
    }
}


#[repr(C)]
pub struct Ags {
    pub usbcmd: u32,            
    pub usbsts: u32,            
    pub pagesize: u32,          
    pub _reserved1: [u32; 2],   
    pub dnctrl: u32,            
    pub crcr: u64,              
    pub _reserved2: [u32; 4],   
    pub dcbaap: u64,            
    pub config: u32,            
    
}


const BKE_: u32 = 1 << 0;
const ALD_: u32 = 1 << 1;
const DDJ_: u32 = 1 << 2;


const ZG_: u32 = 1 << 0;  
const BKF_: u32 = 1 << 11; 


#[repr(C)]
#[derive(Clone, Copy)]
pub struct Qx {
    pub portsc: u32,    
    pub portpmsc: u32,  
    pub portli: u32,    
    pub porthlpmc: u32, 
}


pub(crate) const AII_: u32 = 1 << 0;     
pub(crate) const PX_: u32 = 1 << 1;     
pub(crate) const PY_: u32 = 1 << 4;      
pub(crate) const EEJ_: u32 = 0xF << 5; 
pub(crate) const EEK_: u32 = 1 << 9;      
pub(crate) const BFN_: u32 = 0xF << 10; 
pub(crate) const BFM_: u32 = 1 << 17;    
pub(crate) const LP_: u32 = 1 << 21;    


const BIG_: u32 = 1;   
const BIH_: u32 = 2;    
const AJW_: u32 = 3;   
const AJX_: u32 = 4;  


#[repr(C, align(16))]
#[derive(Clone, Copy, Default)]
pub struct Trb {
    pub parameter: u64,
    pub status: u32,
    pub control: u32,
}

impl Trb {
    pub fn new() -> Self {
        Self { parameter: 0, status: 0, control: 0 }
    }
    
    pub fn link(next_ring_phys: u64) -> Self {
        Self {
            parameter: next_ring_phys,
            status: 0,
            control: (ZB_ << 10) | DH_,
        }
    }
    
    pub fn cer(&self) -> u8 {
        ((self.control >> 10) & 0x3F) as u8
    }
    
    pub fn qce(&self) -> bool {
        (self.control & DH_) != 0
    }
}


pub(crate) const ALA_: u32 = 1;
pub(crate) const ZC_: u32 = 2;
pub(crate) const AKZ_: u32 = 3;
pub(crate) const ZD_: u32 = 4;
pub(crate) const ZB_: u32 = 6;
pub(crate) const EMH_: u32 = 7;
pub(crate) const EMI_: u32 = 8;
pub(crate) const DCF_: u32 = 9;
pub(crate) const EMF_: u32 = 10;
pub(crate) const DCE_: u32 = 11;
pub(crate) const BJU_: u32 = 12;
pub(crate) const EMG_: u32 = 13;
pub(crate) const EML_: u32 = 14;
pub(crate) const EMJ_: u32 = 23;


pub(crate) const DCH_: u32 = 32;
pub(crate) const BJT_: u32 = 33;
pub(crate) const EMK_: u32 = 34;


pub(crate) const DH_: u32 = 1 << 0;
pub(crate) const FL_: u32 = 1 << 5;   


#[repr(C, align(64))]
pub struct Xh {
    pub trbs: [Trb; 256],
}


#[repr(C, align(64))]
#[derive(Clone, Copy)]
pub struct Yk {
    pub ring_base: u64,
    pub dxu: u16,
    pub _reserved: [u16; 3],
}


#[repr(C)]
pub struct Kz {
    pub iman: u32,      
    pub imod: u32,      
    pub erstsz: u32,    
    pub _reserved: u32,
    pub erstba: u64,    
    pub erdp: u64,      
}


#[repr(C, align(32))]
#[derive(Clone, Copy, Default)]
pub struct SlotContext {
    pub data: [u32; 8],
}


#[repr(C, align(32))]
#[derive(Clone, Copy, Default)]
pub struct EndpointContext {
    pub data: [u32; 8],
}


#[repr(C, align(64))]
pub struct Ol {
    pub slot: SlotContext,
    pub endpoints: [EndpointContext; 31],
}


#[repr(C, align(64))]
pub struct Axy {
    pub input_control: Alu,
    pub slot: SlotContext,
    pub endpoints: [EndpointContext; 31],
}

#[repr(C, align(32))]
#[derive(Clone, Copy, Default)]
pub struct Alu {
    pub drop_flags: u32,
    pub add_flags: u32,
    pub _reserved: [u32; 6],
}






#[derive(Clone, Debug)]
pub struct Jh {
    pub slot_id: u8,
    pub port: u8,
    pub speed: u8,
    pub vendor_id: u16,
    pub product_id: u16,
    pub class: u8,
    pub subclass: u8,
    pub protocol: u8,
    pub num_configs: u8,
    pub max_packet_size: u16,
    pub manufacturer: String,
    pub product: String,
}


pub struct An {
    pub base_phys: u64,
    pub base_virt: u64,
    pub cap_regs: *mut Qw,
    pub glc: *mut Ags,
    pub doorbell_base: u64,
    pub runtime_base: u64,
    
    
    pub dcbaa: Box<[u64; 256]>,
    pub fqt: u64,
    
    
    pub cmd_ring: Box<Xh>,
    pub cmd_ring_phys: u64,
    pub cmd_enqueue: usize,
    pub cmd_cycle: bool,
    
    
    pub event_ring: Box<[Trb; 256]>,
    pub event_ring_phys: u64,
    pub fvf: Box<[Yk; 1]>,
    pub fvg: u64,
    pub event_dequeue: usize,
    pub event_cycle: bool,
    
    
    pub device_contexts: [Option<Box<Ol>>; 256],
    
    
    pub devices: Vec<Jh>,
    
    pub max_slots: u8,
    pub max_ports: u8,
    pub context_size: usize,
    pub initialized: bool,
}



unsafe impl Send for An {}

pub(crate) static Ao: Mutex<Option<An>> = Mutex::new(None);
pub(crate) static Ah: AtomicBool = AtomicBool::new(false);





fn lc(virt: u64) -> u64 {
    let bz = crate::memory::hhdm_offset();
    virt.wrapping_sub(bz)
}

pub fn wk(phys: u64) -> u64 {
    let bz = crate::memory::hhdm_offset();
    phys.wrapping_add(bz)
}






pub fn init(bar0: u64) -> bool {
    if bar0 == 0 || bar0 == 0xFFFFFFFF {
        crate::serial_println!("[xHCI] Invalid BAR0");
        return false;
    }
    
    crate::serial_println!("[xHCI] Initializing controller at phys {:#x}", bar0);
    
    
    let base_virt = match crate::memory::yv(bar0, 0x4000) {
        Ok(v) => v,
        Err(e) => {
            crate::serial_println!("[xHCI] Failed to map MMIO: {}", e);
            return false;
        }
    };
    
    crate::serial_println!("[xHCI] Mapped to virt {:#x}", base_virt);
    
    
    let cap_regs = base_virt as *mut Qw;
    let cap = unsafe { &*cap_regs };
    
    let caplength = cap.caplength as u64;
    let version = cap.hciversion;
    let max_slots = cap.max_slots();
    let max_ports = cap.max_ports();
    let context_size = cap.context_size();
    
    crate::serial_println!("[xHCI] Version: {}.{}", version >> 8, version & 0xFF);
    crate::serial_println!("[xHCI] Max slots: {}, Max ports: {}, Context size: {}", 
        max_slots, max_ports, context_size);
    
    
    let nnj = base_virt + caplength;
    let glc = nnj as *mut Ags;
    
    
    let doorbell_base = base_virt + (cap.dboff as u64);
    let runtime_base = base_virt + (cap.rtsoff as u64);
    
    
    
    
    let jry = ((cap.hccparams1 >> 16) & 0xFFFF) as u64;
    if jry != 0 {
        let mut bzd = base_virt + (jry << 2);
        for _ in 0..32 {
            let elc = unsafe { core::ptr::read_volatile(bzd as *const u32) };
            let lnm = elc & 0xFF;
            let eva = (elc >> 8) & 0xFF;

            if lnm == 1 {
                
                crate::serial_println!("[xHCI] Found USBLEGSUP at offset {:#x}", bzd - base_virt);

                let kbz = (elc >> 16) & 1;
                if kbz != 0 {
                    crate::serial_println!("[xHCI] BIOS owns controller, requesting handoff...");

                    
                    unsafe { core::ptr::write_volatile(bzd as *mut u32, elc | (1 << 24)); }

                    
                    let mut ok = false;
                    for i in 0..1000u32 {
                        let v = unsafe { core::ptr::read_volatile(bzd as *const u32) };
                        if (v >> 16) & 1 == 0 {
                            ok = true;
                            crate::serial_println!("[xHCI] BIOS handoff complete ({}ms)", i);
                            break;
                        }
                        for _ in 0..10000 { core::hint::spin_loop(); }
                    }
                    if !ok {
                        crate::serial_println!("[xHCI] WARNING: BIOS handoff timed out, forcing");
                        let v = unsafe { core::ptr::read_volatile(bzd as *const u32) };
                        unsafe { core::ptr::write_volatile(bzd as *mut u32, (v & !(1u32 << 16)) | (1 << 24)); }
                    }

                    
                    
                    let kzv = (bzd + 4) as *mut u32;
                    unsafe { core::ptr::write_volatile(kzv, 0); }
                    crate::serial_println!("[xHCI] USB SMI disabled");
                } else {
                    crate::serial_println!("[xHCI] No BIOS ownership, handoff not needed");
                }
                break;
            }

            if eva == 0 { break; }
            bzd += (eva as u64) << 2;
        }
    }

    
    let op = unsafe { &mut *glc };
    if (op.usbsts & ZG_) == 0 {
        crate::serial_println!("[xHCI] Halting controller...");
        op.usbcmd &= !BKE_;
        
        
        for _ in 0..1000 {
            if (op.usbsts & ZG_) != 0 {
                break;
            }
            for _ in 0..10000 { core::hint::spin_loop(); }
        }
    }
    
    
    crate::serial_println!("[xHCI] Resetting controller...");
    op.usbcmd |= ALD_;
    
    
    for _ in 0..1000 {
        if (op.usbcmd & ALD_) == 0 && (op.usbsts & BKF_) == 0 {
            break;
        }
        for _ in 0..10000 { core::hint::spin_loop(); }
    }
    
    if (op.usbcmd & ALD_) != 0 || (op.usbsts & BKF_) != 0 {
        crate::serial_println!("[xHCI] Reset failed!");
        return false;
    }
    
    crate::serial_println!("[xHCI] Reset complete");
    
    
    let mut dcbaa = Box::new([0u64; 256]);
    let fqt = lc(dcbaa.as_ptr() as u64);
    
    
    let mut cmd_ring = Box::new(Xh { trbs: [Trb::new(); 256] });
    let cmd_ring_phys = lc(cmd_ring.trbs.as_ptr() as u64);
    
    
    cmd_ring.trbs[255] = Trb::link(cmd_ring_phys);
    
    
    let event_ring = Box::new([Trb::new(); 256]);
    let event_ring_phys = lc(event_ring.as_ptr() as u64);
    
    
    let mut fvf = Box::new([Yk {
        ring_base: event_ring_phys,
        dxu: 256,
        _reserved: [0; 3],
    }]);
    let fvg = lc(fvf.as_ptr() as u64);
    
    
    op.config = max_slots as u32;
    
    
    op.dcbaap = fqt;
    
    
    op.crcr = cmd_ring_phys | 1;
    
    
    let clh = (runtime_base + 0x20) as *mut Kz;
    let btn = unsafe { &mut *clh };
    
    btn.erstsz = 1;  
    btn.erstba = fvg;
    btn.erdp = event_ring_phys;
    btn.iman = 0;    
    btn.imod = 0;
    
    
    op.usbcmd = BKE_ | DDJ_;
    
    
    for _ in 0..1000 {
        if (op.usbsts & ZG_) == 0 {
            break;
        }
        for _ in 0..10000 { core::hint::spin_loop(); }
    }
    
    if (op.usbsts & ZG_) != 0 {
        crate::serial_println!("[xHCI] Failed to start controller");
        return false;
    }
    
    crate::serial_println!("[xHCI] Controller running");
    
    
    const CLH_: Option<Box<Ol>> = None;
    let device_contexts: [Option<Box<Ol>>; 256] = [CLH_; 256];
    
    
    let ar = An {
        base_phys: bar0,
        base_virt,
        cap_regs,
        glc,
        doorbell_base,
        runtime_base,
        dcbaa,
        fqt,
        cmd_ring,
        cmd_ring_phys,
        cmd_enqueue: 0,
        cmd_cycle: true,
        event_ring,
        event_ring_phys,
        fvf,
        fvg,
        event_dequeue: 0,
        event_cycle: true,
        device_contexts,
        devices: Vec::new(),
        max_slots,
        max_ports,
        context_size,
        initialized: true,
    };
    
    *Ao.lock() = Some(ar);
    Ah.store(true, Ordering::SeqCst);
    
    
    {
        let mut ctrl = Ao.lock();
        if let Some(c) = ctrl.as_mut() {
            juw(c);
        }
    }
    
    
    mpl(max_slots);
    
    
    lqw();
    
    
    oqf();
    
    true
}


fn lqw() {
    let mut ctrl = Ao.lock();
    let ar = match ctrl.as_mut() {
        Some(c) => c,
        None => return,
    };
    
    let buz = ar.base_virt + 
        (unsafe { &*ar.cap_regs }.caplength as u64) + 0x400;
    
    crate::serial_println!("[xHCI] Enumerating {} ports...", ar.max_ports);
    
    for port_num in 0..ar.max_ports {
        let dws = (buz + (port_num as u64 * 16)) as *mut Qx;
        let port = unsafe { &mut *dws };
        
        let portsc = port.portsc;
        
        
        if (portsc & AII_) != 0 {
            let speed = (portsc & BFN_) >> 10;
            let dzz = match speed {
                BIH_ => "Low (1.5 Mbps)",
                BIG_ => "Full (12 Mbps)",
                AJW_ => "High (480 Mbps)",
                AJX_ => "Super (5 Gbps)",
                _ => "Unknown",
            };
            
            crate::serial_println!("[xHCI] Port {}: Device connected, speed: {}", 
                port_num + 1, dzz);
            
            
            port.portsc = portsc | BFM_ | LP_;
            
            
            if (portsc & PX_) == 0 {
                crate::serial_println!("[xHCI] Port {}: Resetting...", port_num + 1);
                
                
                port.portsc = (portsc & !PX_) | PY_;
                
                
                for _ in 0..100 {
                    for _ in 0..100000 { core::hint::spin_loop(); }
                    let cne = port.portsc;
                    if (cne & PY_) == 0 && (cne & LP_) != 0 {
                        
                        port.portsc = cne | LP_;
                        break;
                    }
                }
                
                let lvn = port.portsc;
                if (lvn & PX_) != 0 {
                    crate::serial_println!("[xHCI] Port {}: Enabled after reset", port_num + 1);
                    
                    
                    ar.devices.push(Jh {
                        slot_id: 0,
                        port: port_num + 1,
                        speed: speed as u8,
                        vendor_id: 0,
                        product_id: 0,
                        class: 0,
                        subclass: 0,
                        protocol: 0,
                        num_configs: 0,
                        max_packet_size: 0,
                        manufacturer: String::new(),
                        product: String::new(),
                    });
                }
            } else {
                crate::serial_println!("[xHCI] Port {}: Already enabled", port_num + 1);
                
                ar.devices.push(Jh {
                    slot_id: 0,
                    port: port_num + 1,
                    speed: speed as u8,
                    vendor_id: 0,
                    product_id: 0,
                    class: 0,
                    subclass: 0,
                    protocol: 0,
                    num_configs: 0,
                    max_packet_size: 0,
                    manufacturer: String::new(),
                    product: String::new(),
                });
            }
        }
    }
    
    crate::serial_println!("[xHCI] Found {} connected devices", ar.devices.len());
}






pub(crate) fn eap(ar: &mut An, nq: Trb) {
    let idx = ar.cmd_enqueue;
    
    
    let mut cmd = nq;
    if ar.cmd_cycle {
        cmd.control |= DH_;
    } else {
        cmd.control &= !DH_;
    }
    
    ar.cmd_ring.trbs[idx] = cmd;
    
    
    ar.cmd_enqueue += 1;
    if ar.cmd_enqueue >= 255 {
        
        let gfn = (ZB_ << 10) | if ar.cmd_cycle { DH_ } else { 0 } | (1 << 1); 
        ar.cmd_ring.trbs[255].control = gfn;
        ar.cmd_ring.trbs[255].parameter = ar.cmd_ring_phys;
        ar.cmd_cycle = !ar.cmd_cycle;
        ar.cmd_enqueue = 0;
    }
    
    
    unsafe {
        let fu = ar.doorbell_base as *mut u32;
        ptr::write_volatile(fu, 0);
    }
}


fn fev(ar: &mut An) -> Option<(u8, u8, u64)> {
    for _ in 0..2_000_000u32 {
        let idx = ar.event_dequeue;
        let nq = ar.event_ring[idx];
        
        
        let phase = (nq.control & DH_) != 0;
        if phase == ar.event_cycle {
            
            ar.event_dequeue += 1;
            if ar.event_dequeue >= 256 {
                ar.event_dequeue = 0;
                ar.event_cycle = !ar.event_cycle;
            }
            
            
            let dou = ar.event_ring_phys + (ar.event_dequeue as u64 * 16);
            let clh = (ar.runtime_base + 0x20) as *mut Kz;
            unsafe {
                (*clh).erdp = dou | (1 << 3); 
            }
            
            let cer = (nq.control >> 10) & 0x3F;
            
            if cer == BJT_ {
                let byh = ((nq.status >> 24) & 0xFF) as u8;
                let slot_id = ((nq.control >> 24) & 0xFF) as u8;
                return Some((byh, slot_id, nq.parameter));
            }
            
            continue;
        }
        core::hint::spin_loop();
    }
    None
}


fn few(ar: &mut An) -> Option<(u8, u32, u8)> {
    for _ in 0..5_000_000u32 {
        let idx = ar.event_dequeue;
        let nq = ar.event_ring[idx];
        
        let phase = (nq.control & DH_) != 0;
        if phase == ar.event_cycle {
            ar.event_dequeue += 1;
            if ar.event_dequeue >= 256 {
                ar.event_dequeue = 0;
                ar.event_cycle = !ar.event_cycle;
            }
            
            let dou = ar.event_ring_phys + (ar.event_dequeue as u64 * 16);
            let clh = (ar.runtime_base + 0x20) as *mut Kz;
            unsafe {
                (*clh).erdp = dou | (1 << 3);
            }
            
            let cer = (nq.control >> 10) & 0x3F;
            
            if cer == DCH_ {
                let byh = ((nq.status >> 24) & 0xFF) as u8;
                let gzx = nq.status & 0xFFFFFF;
                let fuw = ((nq.control >> 16) & 0x1F) as u8;
                return Some((byh, gzx, fuw));
            }
            if cer == BJT_ {
                let byh = ((nq.status >> 24) & 0xFF) as u8;
                let slot_id = ((nq.control >> 24) & 0xFF) as u8;
                return Some((byh, 0, slot_id));
            }
            continue;
        }
        core::hint::spin_loop();
    }
    None
}






pub(crate) struct TransferRing {
    trbs: Box<[Trb; 256]>,
    pub(crate) phys: u64,
    pub(crate) enqueue: usize,
    pub(crate) cycle: bool,
}

impl TransferRing {
    pub(crate) fn new() -> Option<Self> {
        let trbs = Box::new([Trb::new(); 256]);
        let phys = lc(trbs.as_ptr() as u64);
        Some(Self { trbs, phys, enqueue: 0, cycle: true })
    }
    
    pub(crate) fn enqueue_trb(&mut self, mut nq: Trb) {
        if self.cycle {
            nq.control |= DH_;
        } else {
            nq.control &= !DH_;
        }
        self.trbs[self.enqueue] = nq;
        self.enqueue += 1;
        if self.enqueue >= 255 {
            
            let gfn = (ZB_ << 10) | if self.cycle { DH_ } else { 0 } | (1 << 1);
            self.trbs[255].control = gfn;
            self.trbs[255].parameter = self.phys;
            self.cycle = !self.cycle;
            self.enqueue = 0;
        }
    }
}



pub(crate) struct Aey {
    pub(crate) ep0: TransferRing,         
    pub(crate) interrupt_in: Option<TransferRing>, 
    pub(crate) interrupt_dci: u8,         
    pub(crate) bulk_in: Option<TransferRing>,    
    pub(crate) bulk_in_dci: u8,
    pub(crate) bulk_out: Option<TransferRing>,   
    pub(crate) bulk_out_dci: u8,
}





fn juw(ar: &mut An) {
    let cap = unsafe { &*ar.cap_regs };
    let hcsparams2 = cap.hcsparams2;
    
    let hi = ((hcsparams2 >> 21) & 0x1F) as u32;
    let lo = ((hcsparams2 >> 27) & 0x1F) as u32;
    let gjy = (hi << 5) | lo;
    
    if gjy == 0 {
        return;
    }
    
    crate::serial_println!("[xHCI] Allocating {} scratchpad buffers", gjy);
    
    
    let hfr = match crate::memory::frame::aan() {
        Some(aa) => aa,
        None => { crate::serial_println!("[xHCI] OOM for scratchpad array"); return; }
    };
    let jxs = wk(hfr) as *mut u64;
    
    
    for i in 0..gjy.min(512) as usize {
        if let Some(bcy) = crate::memory::frame::aan() {
            unsafe { ptr::write_volatile(jxs.add(i), bcy); }
        }
    }
    
    
    ar.dcbaa[0] = hfr;
}






pub(crate) static CD_: Mutex<Vec<Option<Aey>>> = Mutex::new(Vec::new());

fn mpl(max_slots: u8) {
    let mut rings = CD_.lock();
    rings.clear();
    for _ in 0..=max_slots {
        rings.push(None);
    }
}


fn lpt(ar: &mut An) -> Option<u8> {
    let nq = Trb {
        parameter: 0,
        status: 0,
        control: (DCF_ << 10),
    };
    
    eap(ar, nq);
    
    if let Some((ft, slot_id, _param)) = fev(ar) {
        if ft == 1 { 
            crate::serial_println!("[xHCI] Enable Slot → slot_id={}", slot_id);
            return Some(slot_id);
        }
        crate::serial_println!("[xHCI] Enable Slot failed: cc={}", ft);
    }
    None
}


fn jub(ar: &mut An, slot_id: u8, port_num: u8, speed: u8) -> bool {
    
    let hry = Box::new(Ol {
        slot: SlotContext::default(),
        endpoints: [EndpointContext::default(); 31],
    });
    let leb = lc(&*hry as *const _ as u64);
    ar.dcbaa[slot_id as usize] = leb;
    ar.device_contexts[slot_id as usize] = Some(hry);
    
    
    let hwj = match TransferRing::new() {
        Some(r) => r,
        None => { crate::serial_println!("[xHCI] OOM for EP0 ring"); return false; }
    };
    let lra = hwj.phys;
    
    
    let eqs = match crate::memory::frame::aan() {
        Some(aa) => aa,
        None => { crate::serial_println!("[xHCI] OOM for input context"); return false; }
    };
    let gcx = wk(eqs);
    let brh = ar.context_size;
    
    
    unsafe {
        let ckt = gcx as *mut u32;
        ptr::write_volatile(ckt.add(1), 0x3); 
    }
    
    
    let ots = gcx + brh as u64;
    let bcp = match speed as u32 {
        BIH_ => 8u16,
        BIG_ => 8,
        AJW_ => 64,
        AJX_ => 512,
        _ => 64,
    };
    
    unsafe {
        let slot = ots as *mut u32;
        
        let dzx = (speed as u32) << 20;
        let dlk = 1u32 << 27; 
        ptr::write_volatile(slot, dzx | dlk);
        
        ptr::write_volatile(slot.add(1), (port_num as u32) << 16);
    }
    
    
    let lqz = gcx + (2 * brh) as u64;
    unsafe {
        let fvc = lqz as *mut u32;
        
        let lrh = 4u32 << 3;
        let cuw = 3u32 << 1;
        let gih = (bcp as u32) << 16;
        ptr::write_volatile(fvc.add(1), cuw | lrh | gih);
        
        let pml = lra | 1; 
        ptr::write_volatile(fvc.add(2) as *mut u64, pml);
        
        ptr::write_volatile(fvc.add(4), 8);
    }
    
    
    {
        let mut rings = CD_.lock();
        if (slot_id as usize) < rings.len() {
            rings[slot_id as usize] = Some(Aey {
                ep0: hwj,
                interrupt_in: None,
                interrupt_dci: 0,
                bulk_in: None,
                bulk_in_dci: 0,
                bulk_out: None,
                bulk_out_dci: 0,
            });
        }
    }
    
    
    let nq = Trb {
        parameter: eqs,
        status: 0,
        control: (DCE_ << 10) | ((slot_id as u32) << 24),
    };
    
    eap(ar, nq);
    
    if let Some((ft, _sid, _param)) = fev(ar) {
        if ft == 1 {
            crate::serial_println!("[xHCI] Address Device slot {} → success", slot_id);
            crate::memory::frame::vk(eqs);
            return true;
        }
        crate::serial_println!("[xHCI] Address Device failed: cc={}", ft);
    }
    crate::memory::frame::vk(eqs);
    false
}






const ME_: u8 = 0x80;
const ZH_: u8 = 0x00;
const MG_: u8 = 0x00;
const MF_: u8 = 0x00;
const RE_: u8 = 0x01;

const RF_: u8 = 0x06;
const DDP_: u8 = 0x09;
const DDR_: u8 = 0x0B;
const DDQ_: u8 = 0x0A;

const DDL_: u8 = 0x01;
const BKG_: u8 = 0x02;
const ENF_: u8 = 0x03;
const DDN_: u8 = 0x04;
const DDM_: u8 = 0x05;
const END_: u8 = 0x21;
const ENE_: u8 = 0x22;


fn foj(
    ar: &mut An,
    slot_id: u8,
    bm_request_type: u8,
    b_request: u8,
    w_value: u16,
    w_index: u16,
    w_length: u16,
    hg: u64,
) -> Option<u32> {
    let mut rings = CD_.lock();
    let acp = rings.get_mut(slot_id as usize)?.as_mut()?;
    
    
    let setup_data = (bm_request_type as u64)
        | ((b_request as u64) << 8)
        | ((w_value as u64) << 16)
        | ((w_index as u64) << 32)
        | ((w_length as u64) << 48);
    
    let gum = Trb {
        parameter: setup_data,
        status: 8, 
        control: (ZC_ << 10) | (1 << 6) | (3 << 16), 
    };
    acp.ep0.enqueue_trb(gum);
    
    
    if w_length > 0 {
        let ejx = Trb {
            parameter: hg,
            status: w_length as u32,
            control: (AKZ_ << 10) | (1 << 16), 
        };
        acp.ep0.enqueue_trb(ejx);
    }
    
    
    let gwi = Trb {
        parameter: 0,
        status: 0,
        control: (ZD_ << 10) | FL_, 
    };
    acp.ep0.enqueue_trb(gwi);
    
    
    drop(rings);
    unsafe {
        let fu = (ar.doorbell_base + (slot_id as u64) * 4) as *mut u32;
        ptr::write_volatile(fu, 1); 
    }
    
    
    if let Some((ft, bjr, _ep)) = few(ar) {
        if ft == 1 || ft == 13 { 
            return Some(bjr);
        }
        crate::serial_println!("[xHCI] Control IN failed: cc={}", ft);
    }
    None
}


fn fok(
    ar: &mut An,
    slot_id: u8,
    bm_request_type: u8,
    b_request: u8,
    w_value: u16,
    w_index: u16,
) -> bool {
    let mut rings = CD_.lock();
    let acp = match rings.get_mut(slot_id as usize).and_then(|r| r.as_mut()) {
        Some(r) => r,
        None => return false,
    };
    
    let setup_data = (bm_request_type as u64)
        | ((b_request as u64) << 8)
        | ((w_value as u64) << 16)
        | ((w_index as u64) << 32);
    
    let gum = Trb {
        parameter: setup_data,
        status: 8,
        control: (ZC_ << 10) | (1 << 6), 
    };
    acp.ep0.enqueue_trb(gum);
    
    let gwi = Trb {
        parameter: 0,
        status: 0,
        control: (ZD_ << 10) | FL_ | (1 << 16), 
    };
    acp.ep0.enqueue_trb(gwi);
    
    drop(rings);
    unsafe {
        let fu = (ar.doorbell_base + (slot_id as u64) * 4) as *mut u32;
        ptr::write_volatile(fu, 1);
    }
    
    if let Some((ft, _, _)) = few(ar) {
        return ft == 1;
    }
    false
}






fn mcv(ar: &mut An, slot_id: u8, s: &mut Jh) -> bool {
    let hg = match crate::memory::frame::aan() {
        Some(aa) => aa,
        None => return false,
    };
    let kt = wk(hg) as *const u8;
    
    
    let result = foj(
        ar, slot_id,
        ME_ | MG_ | MF_,
        RF_,
        (DDL_ as u16) << 8,
        0, 18, hg,
    );
    
    if result.is_some() {
        unsafe {
            let pwt = ptr::read_unaligned(kt.add(2) as *const u16);
            s.class = *kt.add(4);
            s.subclass = *kt.add(5);
            s.protocol = *kt.add(6);
            s.max_packet_size = *kt.add(7) as u16;
            s.vendor_id = ptr::read_unaligned(kt.add(8) as *const u16);
            s.product_id = ptr::read_unaligned(kt.add(10) as *const u16);
            s.num_configs = *kt.add(17);
        }
        crate::serial_println!("[xHCI] Device: VID={:04X} PID={:04X} class={:02X}:{:02X}:{:02X}",
            s.vendor_id, s.product_id, s.class, s.subclass, s.protocol);
    }
    
    crate::memory::frame::vk(hg);
    result.is_some()
}



fn mct(ar: &mut An, slot_id: u8) 
    -> Option<Vec<(u8, u8, u8, u8, u8, u16, u8)>> 
{
    let hg = match crate::memory::frame::aan() {
        Some(aa) => aa,
        None => return None,
    };
    let kt = wk(hg) as *const u8;
    
    
    foj(
        ar, slot_id,
        ME_ | MG_ | MF_,
        RF_,
        (BKG_ as u16) << 8,
        0, 9, hg,
    )?;
    
    let bjq = unsafe { ptr::read_unaligned(kt.add(2) as *const u16) };
    let arx = bjq.min(4096);
    
    
    foj(
        ar, slot_id,
        ME_ | MG_ | MF_,
        RF_,
        (BKG_ as u16) << 8,
        0, arx, hg,
    )?;
    
    let kwv = unsafe { *kt.add(5) };
    let mut interfaces = Vec::new();
    
    
    let mut offset = 0usize;
    let mut byj = (0u8, 0u8, 0u8, 0u8); 
    
    while offset + 1 < arx as usize {
        let len = unsafe { *kt.add(offset) } as usize;
        let ldq = unsafe { *kt.add(offset + 1) };
        
        if len == 0 { break; }
        
        match ldq {
            DDN_ if len >= 9 => {
                let czk = unsafe { *kt.add(offset + 2) };
                let bmx = unsafe { *kt.add(offset + 5) };
                let gbq = unsafe { *kt.add(offset + 6) };
                let eqe = unsafe { *kt.add(offset + 7) };
                
                byj = (bmx, czk, gbq, eqe);
                
                if bmx == 0x03 {
                    crate::serial_println!("[xHCI]   HID interface {}: subclass={} protocol={} ({})",
                        czk, gbq, eqe,
                        match eqe { 1 => "keyboard", 2 => "mouse", _ => "other" });
                } else if bmx == 0x08 {
                    crate::serial_println!("[xHCI]   Mass Storage interface {}: subclass={:#x} protocol={:#x}",
                        czk, gbq, eqe);
                }
            }
            DDM_ if len >= 7 => {
                let bgj = unsafe { *kt.add(offset + 2) };
                let lrb = unsafe { *kt.add(offset + 3) };
                let hwl = unsafe { ptr::read_unaligned(kt.add(offset + 4) as *const u16) };
                let hwk = unsafe { *kt.add(offset + 6) };
                let hwm = lrb & 0x03;
                
                let bmx = byj.0;
                if bmx == 0x03 && hwm == 3 && (bgj & 0x80 != 0) {
                    
                    interfaces.push((
                        bmx, byj.1, byj.2, byj.3,
                        bgj, hwl & 0x7FF, hwk,
                    ));
                } else if bmx == 0x08 && hwm == 2 {
                    
                    interfaces.push((
                        bmx, byj.1, byj.2, byj.3,
                        bgj, hwl & 0x7FF, hwk,
                    ));
                }
            }
            _ => {}
        }
        
        offset += len;
    }
    
    
    if !interfaces.is_empty() {
        fok(
            ar, slot_id,
            ZH_ | MG_ | MF_,
            DDP_,
            kwv as u16,
            0,
        );
    }
    
    crate::memory::frame::vk(hg);
    
    if interfaces.is_empty() { None } else { Some(interfaces) }
}


fn kwx(
    ar: &mut An,
    slot_id: u8,
    port_num: u8,
    speed: u8,
    bgj: u8,
    bcp: u16,
    axr: u8,
) -> bool {
    let lrd = bgj & 0x0F;
    let ahu = (lrd * 2 + 1) as u8; 
    
    
    let eqv = match TransferRing::new() {
        Some(r) => r,
        None => return false,
    };
    let mqv = eqv.phys;
    
    
    let cle = match crate::memory::frame::aan() {
        Some(aa) => aa,
        None => return false,
    };
    let cav = wk(cle);
    let brh = ar.context_size;
    
    unsafe {
        let ckt = cav as *mut u32;
        
        ptr::write_volatile(ckt.add(1), 1 | (1u32 << ahu));
        
        
        let slot = (cav + brh as u64) as *mut u32;
        let dzx = (speed as u32) << 20;
        let dlk = (ahu as u32) << 27;
        ptr::write_volatile(slot, dzx | dlk);
        ptr::write_volatile(slot.add(1), (port_num as u32) << 16);
        
        
        let eln = (cav + ((1 + ahu as usize) * brh) as u64) as *mut u32;
        
        
        let pvt = match speed as u32 {
            AJW_ | AJX_ => axr.max(1) as u32,
            _ => {
                
                let frames = (axr as u32).max(1) * 8;
                let mut log2 = 0u32;
                let mut v = frames;
                while v > 1 { v >>= 1; log2 += 1; }
                log2 + 1
            }
        };
        ptr::write_volatile(eln, (pvt << 16));
        
        
        let lri = 7u32 << 3;
        let cuw = 3u32 << 1;
        let gih = (bcp as u32) << 16;
        ptr::write_volatile(eln.add(1), cuw | lri | gih);
        
        
        ptr::write_volatile(eln.add(2) as *mut u64, mqv | 1);
        
        
        ptr::write_volatile(eln.add(4), (bcp as u32) | ((bcp as u32) << 16));
    }
    
    
    {
        let mut rings = CD_.lock();
        if let Some(Some(acp)) = rings.get_mut(slot_id as usize) {
            acp.interrupt_in = Some(eqv);
            acp.interrupt_dci = ahu;
        }
    }
    
    
    let nq = Trb {
        parameter: cle,
        status: 0,
        control: (BJU_ << 10) | ((slot_id as u32) << 24),
    };
    
    eap(ar, nq);
    
    let success = if let Some((ft, _, _)) = fev(ar) {
        if ft == 1 {
            crate::serial_println!("[xHCI] Configure Endpoint slot {} DCI {} → success", slot_id, ahu);
            true
        } else {
            crate::serial_println!("[xHCI] Configure Endpoint failed: cc={}", ft);
            false
        }
    } else {
        false
    };
    
    crate::memory::frame::vk(cle);
    success
}


fn kww(
    ar: &mut An,
    slot_id: u8,
    port_num: u8,
    speed: u8,
    ep_in_addr: u8,
    ep_out_addr: u8,
    max_packet_in: u16,
    max_packet_out: u16,
) -> bool {
    let lrc = ep_in_addr & 0x0F;
    let dmk = (lrc * 2 + 1) as u8;  
    let lre = ep_out_addr & 0x0F;
    let dml = (lre * 2) as u8;     
    let ncu = dmk.max(dml);
    
    
    let hjd = match TransferRing::new() {
        Some(r) => r,
        None => return false,
    };
    let hje = match TransferRing::new() {
        Some(r) => r,
        None => return false,
    };
    let mom = hjd.phys;
    let nom = hje.phys;
    
    
    let cle = match crate::memory::frame::aan() {
        Some(aa) => aa,
        None => return false,
    };
    let cav = wk(cle);
    let brh = ar.context_size;
    
    unsafe {
        let ckt = cav as *mut u32;
        
        ptr::write_volatile(ckt.add(1), 1 | (1u32 << dmk) | (1u32 << dml));
        
        
        let slot = (cav + brh as u64) as *mut u32;
        let dzx = (speed as u32) << 20;
        let dlk = (ncu as u32) << 27;
        ptr::write_volatile(slot, dzx | dlk);
        ptr::write_volatile(slot.add(1), (port_num as u32) << 16);
        
        
        let fvd = (cav + ((1 + dmk as usize) * brh) as u64) as *mut u32;
        let cuw = 3u32 << 1;
        let lrf = 6u32 << 3;
        let ngl = (max_packet_in as u32) << 16;
        ptr::write_volatile(fvd.add(1), cuw | lrf | ngl);
        ptr::write_volatile(fvd.add(2) as *mut u64, mom | 1);
        ptr::write_volatile(fvd.add(4), max_packet_in as u32);
        
        
        let fve = (cav + ((1 + dml as usize) * brh) as u64) as *mut u32;
        let lrg = 2u32 << 3;
        let ngm = (max_packet_out as u32) << 16;
        ptr::write_volatile(fve.add(1), cuw | lrg | ngm);
        ptr::write_volatile(fve.add(2) as *mut u64, nom | 1);
        ptr::write_volatile(fve.add(4), max_packet_out as u32);
    }
    
    
    {
        let mut rings = CD_.lock();
        if let Some(Some(acp)) = rings.get_mut(slot_id as usize) {
            acp.bulk_in = Some(hjd);
            acp.bulk_in_dci = dmk;
            acp.bulk_out = Some(hje);
            acp.bulk_out_dci = dml;
        }
    }
    
    
    let nq = Trb {
        parameter: cle,
        status: 0,
        control: (BJU_ << 10) | ((slot_id as u32) << 24),
    };
    
    eap(ar, nq);
    
    let success = if let Some((ft, _, _)) = fev(ar) {
        if ft == 1 {
            crate::serial_println!("[xHCI] Bulk endpoints configured: slot {} IN_DCI={} OUT_DCI={}",
                slot_id, dmk, dml);
            true
        } else {
            crate::serial_println!("[xHCI] Configure bulk EPs failed: cc={}", ft);
            false
        }
    } else {
        false
    };
    
    crate::memory::frame::vk(cle);
    success
}






fn ooq(ar: &mut An, slot_id: u8, interface: u8) -> bool {
    fok(
        ar, slot_id,
        ZH_ | (1 << 5) | RE_, 
        DDR_,
        0, 
        interface as u16,
    )
}


fn opa(ar: &mut An, slot_id: u8, interface: u8) -> bool {
    fok(
        ar, slot_id,
        ZH_ | (1 << 5) | RE_,
        DDQ_,
        0, 
        interface as u16,
    )
}


fn oar(ar: &An, slot_id: u8, hg: u64, bcp: u16) -> bool {
    let mut rings = CD_.lock();
    let acp = match rings.get_mut(slot_id as usize).and_then(|r| r.as_mut()) {
        Some(r) => r,
        None => return false,
    };
    
    let eqv = match acp.interrupt_in.as_mut() {
        Some(r) => r,
        None => return false,
    };
    let ahu = acp.interrupt_dci;
    
    let nq = Trb {
        parameter: hg,
        status: bcp as u32,
        control: (ALA_ << 10) | FL_,
    };
    eqv.enqueue_trb(nq);
    
    
    drop(rings);
    unsafe {
        let fu = (ar.doorbell_base + (slot_id as u64) * 4) as *mut u32;
        ptr::write_volatile(fu, ahu as u32);
    }
    
    true
}









fn nyj(report: &[u8]) {
    if report.len() < 8 { return; }
    
    let pxc = report[0];
    
    
    for &keycode in &report[2..8] {
        if keycode == 0 { continue; }
        
        
        let ascii = mli(keycode, report[0]);
        if ascii != 0 {
            crate::keyboard::nzs(ascii);
        }
    }
}






fn nyk(report: &[u8]) {
    if report.len() < 3 { return; }
    
    let buttons = report[0];
    let dx = report[1] as i8 as i32;
    let ad = report[2] as i8 as i32;
    let scroll = if report.len() >= 4 { report[3] as i8 } else { 0 };
    
    crate::mouse::mqc(
        dx, ad,
        buttons & 1 != 0,
        buttons & 2 != 0,
        buttons & 4 != 0,
        scroll,
    );
}


fn mli(keycode: u8, modifiers: u8) -> u8 {
    let no = (modifiers & 0x22) != 0; 
    let hdh = (modifiers & 0x11) != 0;
    
    match keycode {
        
        0x04..=0x1D => {
            let base = b'a' + (keycode - 0x04);
            if no { base - 32 } else { base }
        }
        
        0x1E..=0x26 => {
            if no {
                match keycode {
                    0x1E => b'!', 0x1F => b'@', 0x20 => b'#', 0x21 => b'$',
                    0x22 => b'%', 0x23 => b'^', 0x24 => b'&', 0x25 => b'*',
                    0x26 => b'(',
                    _ => 0,
                }
            } else {
                b'1' + (keycode - 0x1E)
            }
        }
        0x27 => if no { b')' } else { b'0' },
        0x28 => b'\r',   
        0x29 => 0x1B,    
        0x2A => 0x08,    
        0x2B => b'\t',   
        0x2C => b' ',    
        0x2D => if no { b'_' } else { b'-' },
        0x2E => if no { b'+' } else { b'=' },
        0x2F => if no { b'{' } else { b'[' },
        0x30 => if no { b'}' } else { b']' },
        0x31 => if no { b'|' } else { b'\\' },
        0x33 => if no { b':' } else { b';' },
        0x34 => if no { b'"' } else { b'\'' },
        0x35 => if no { b'~' } else { b'`' },
        0x36 => if no { b'<' } else { b',' },
        0x37 => if no { b'>' } else { b'.' },
        0x38 => if no { b'?' } else { b'/' },
        _ => 0,
    }
}






fn oqf() {
    let mut ioo: Vec<(u8, u8, u8, u16, u16)> = Vec::new();
    
    {
        let mut ctrl = Ao.lock();
        let ar = match ctrl.as_mut() {
            Some(c) => c,
            None => return,
        };
        
        let gka = ar.devices.len();
        if gka == 0 {
            return;
        }
        
        crate::serial_println!("[xHCI] Setting up {} devices...", gka);
        
        
        for i in 0..gka {
            let port = ar.devices[i].port;
            let speed = ar.devices[i].speed;
            
            
            let slot_id = match lpt(ar) {
                Some(id) => id,
                None => {
                    crate::serial_println!("[xHCI] Failed to enable slot for port {}", port);
                    continue;
                }
            };
            
            ar.devices[i].slot_id = slot_id;
            
            
            if !jub(ar, slot_id, port, speed) {
                crate::serial_println!("[xHCI] Failed to address device on port {}", port);
                continue;
            }
            
            
            let mut s = ar.devices[i].clone();
            if !mcv(ar, slot_id, &mut s) {
                crate::serial_println!("[xHCI] Failed to get device descriptor for slot {}", slot_id);
                continue;
            }
            ar.devices[i] = s;
            
            
            if let Some(all_interfaces) = mct(ar, slot_id) {
                let mut iop: Option<(u8, u16)> = None;
                let mut ioq: Option<(u8, u16)> = None;
                
                for &(bmx, czk, subclass, protocol, bgj, bcp, axr) in &all_interfaces {
                    match bmx {
                        0x03 => {
                            
                            let _ = ooq(ar, slot_id, czk);
                            let _ = opa(ar, slot_id, czk);
                            
                            kwx(
                                ar, slot_id, port, speed,
                                bgj, bcp, axr,
                            );
                            
                            if ar.devices[i].class == 0 {
                                ar.devices[i].class = 0x03;
                                ar.devices[i].subclass = subclass;
                                ar.devices[i].protocol = protocol;
                            }
                            
                            crate::serial_println!("[xHCI] HID endpoint configured: slot {} EP {:#x} max_pkt {} interval {}",
                                slot_id, bgj, bcp, axr);
                        }
                        0x08 => {
                            
                            if bgj & 0x80 != 0 {
                                iop = Some((bgj, bcp));
                            } else {
                                ioq = Some((bgj, bcp));
                            }
                        }
                        _ => {}
                    }
                }
                
                
                if let (Some((in_addr, in_mps)), Some((out_addr, out_mps))) = (iop, ioq) {
                    if kww(ar, slot_id, port, speed, in_addr, out_addr, in_mps, out_mps) {
                        ioo.push((slot_id, in_addr, out_addr, in_mps, out_mps));
                        
                        if ar.devices[i].class == 0 {
                            ar.devices[i].class = 0x08;
                            ar.devices[i].subclass = 0x06;
                            ar.devices[i].protocol = 0x50;
                        }
                    }
                }
            }
        }
        
        crate::serial_println!("[xHCI] Device setup complete");
    }
    
    
    
    for (slot_id, in_addr, out_addr, in_mps, out_mps) in ioo {
        super::usb_storage::mpb(slot_id, in_addr, out_addr, in_mps, out_mps);
    }
}






pub fn kgd(slot_id: u8, ahu: u8, hg: u64, length: u32) -> bool {
    
    {
        let mut rings = CD_.lock();
        let acp = match rings.get_mut(slot_id as usize).and_then(|r| r.as_mut()) {
            Some(r) => r,
            None => return false,
        };
        
        let dq = match acp.bulk_out.as_mut() {
            Some(r) => r,
            None => return false,
        };
        
        let nq = Trb {
            parameter: hg,
            status: length,
            control: (ALA_ << 10) | FL_,
        };
        dq.enqueue_trb(nq);
    }
    
    
    let mut ctrl = Ao.lock();
    let ar = match ctrl.as_mut() {
        Some(c) => c,
        None => return false,
    };
    
    unsafe {
        let fu = (ar.doorbell_base + (slot_id as u64) * 4) as *mut u32;
        ptr::write_volatile(fu, ahu as u32);
    }
    
    if let Some((ft, _, _)) = few(ar) {
        return ft == 1 || ft == 13; 
    }
    false
}


pub fn kgc(slot_id: u8, ahu: u8, hg: u64, length: u32) -> Option<u32> {
    
    {
        let mut rings = CD_.lock();
        let acp = match rings.get_mut(slot_id as usize).and_then(|r| r.as_mut()) {
            Some(r) => r,
            None => return None,
        };
        
        let dq = match acp.bulk_in.as_mut() {
            Some(r) => r,
            None => return None,
        };
        
        let nq = Trb {
            parameter: hg,
            status: length,
            control: (ALA_ << 10) | FL_,
        };
        dq.enqueue_trb(nq);
    }
    
    
    let mut ctrl = Ao.lock();
    let ar = match ctrl.as_mut() {
        Some(c) => c,
        None => return None,
    };
    
    unsafe {
        let fu = (ar.doorbell_base + (slot_id as u64) * 4) as *mut u32;
        ptr::write_volatile(fu, ahu as u32);
    }
    
    if let Some((ft, residue, _)) = few(ar) {
        if ft == 1 || ft == 13 {
            
            return Some(length.saturating_sub(residue));
        }
    }
    None
}


pub fn qqr() {
    
    let mlh: Vec<(u8, u16, u8)> = {
        let ctrl = Ao.lock();
        match ctrl.as_ref() {
            Some(c) => c.devices.iter()
                .filter(|d| d.slot_id != 0 && d.class == 0x03)
                .map(|d| (d.slot_id, d.max_packet_size, d.protocol))
                .collect(),
            None => return,
        }
    };
    
    for &(slot_id, max_packet_size, protocol) in &mlh {
        let hg = match crate::memory::frame::aan() {
            Some(aa) => aa,
            None => continue,
        };
        let kt = wk(hg);
        let imr = max_packet_size.max(8);
        
        
        {
            let ctrl = Ao.lock();
            if let Some(ar) = ctrl.as_ref() {
                if !oar(ar, slot_id, hg, imr) {
                    crate::memory::frame::vk(hg);
                    continue;
                }
            } else {
                crate::memory::frame::vk(hg);
                continue;
            }
        }
        
        
        {
            let mut kzw = Ao.lock();
            if let Some(ar) = kzw.as_mut() {
                for _ in 0..50_000u32 {
                    let idx = ar.event_dequeue;
                    let nq = ar.event_ring[idx];
                    let phase = (nq.control & DH_) != 0;
                    if phase == ar.event_cycle {
                        ar.event_dequeue += 1;
                        if ar.event_dequeue >= 256 {
                            ar.event_dequeue = 0;
                            ar.event_cycle = !ar.event_cycle;
                        }
                        let erdp = ar.event_ring_phys + (ar.event_dequeue as u64 * 16);
                        let btn = (ar.runtime_base + 0x20) as *mut Kz;
                        unsafe { (*btn).erdp = erdp | (1 << 3); }
                        
                        let ft = ((nq.status >> 24) & 0xFF) as u8;
                        if ft == 1 || ft == 13 { 
                            let report = unsafe {
                                core::slice::from_raw_parts(kt as *const u8, imr as usize)
                            };
                            
                            match protocol {
                                1 => nyj(report),
                                2 => nyk(report),
                                _ => {}
                            }
                        }
                        break;
                    }
                    core::hint::spin_loop();
                }
            }
        }
        
        crate::memory::frame::vk(hg);
        return; 
    }
}






pub fn is_initialized() -> bool {
    Ah.load(Ordering::SeqCst)
}


pub fn aqg() -> usize {
    Ao.lock().as_ref().map(|c| c.devices.len()).unwrap_or(0)
}


pub fn adz() -> Vec<Jh> {
    Ao.lock().as_ref()
        .map(|c| c.devices.clone())
        .unwrap_or_default()
}


pub fn qhq(slot_id: u8) -> Option<Jh> {
    Ao.lock().as_ref()
        .and_then(|c| c.devices.iter().find(|d| d.slot_id == slot_id).cloned())
}
