










use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::vec;
use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use super::{Dd, NetStats};
use crate::drivers::{Cw, Bb, DriverStatus, DriverCategory};
use crate::pci::L;






const JN_: u32 = 0x0000;        
const LS_: u32 = 0x0008;      
const BGD_: u32 = 0x0014;        


const BGE_: u32 = 0x00C0;         
const EFU_: u32 = 0x00D0;         
const AIX_: u32 = 0x00D8;         


const LR_: u32 = 0x0100;        
const CRI_: u32 = 0x2800;       
const CRH_: u32 = 0x2804;       
const CRK_: u32 = 0x2808;       
const CRJ_: u32 = 0x2810;         
const BGJ_: u32 = 0x2818;         


const BGK_: u32 = 0x0400;        
const CRS_: u32 = 0x0410;        
const CRP_: u32 = 0x3800;       
const CRO_: u32 = 0x3804;       
const CRR_: u32 = 0x3808;       
const CRQ_: u32 = 0x3810;         
const BGL_: u32 = 0x3818;         


const BGI_: u32 = 0x5400;        
const BGH_: u32 = 0x5404;        


const CRG_: u32 = 0x5200;         





const BTB_: u32 = 1 << 0;         
const ARP_: u32 = 1 << 5;       
const BTF_: u32 = 1 << 6;        
const ARQ_: u32 = 1 << 26;       


const AKC_: u32 = 1 << 1;       
const BIL_: u32 = 0xC0; 


const CQO_: u32 = 1 << 1;         
const CQR_: u32 = 1 << 2;        
const AIQ_: u32 = 1 << 3;        
const AIP_: u32 = 1 << 4;        
const CQP_: u32 = 0 << 6;   
const CQQ_: u32 = 0 << 8; 
const CQM_: u32 = 1 << 15;       
const CQN_: u32 = 0 << 16; 
const CQS_: u32 = 1 << 26;     


const DBH_: u32 = 1 << 1;         
const DBI_: u32 = 1 << 3;        
const DBG_: u32 = 4;        
const DBF_: u32 = 12;     
const DBJ_: u32 = 1 << 24;      


const DBK_: u8 = 1 << 0;    
const DBL_: u8 = 1 << 1;   
const DBM_: u8 = 1 << 3;     


const AKO_: u8 = 1 << 0;     


const CQT_: u8 = 1 << 0;     


const CER_: u32 = 1 << 2;         


const AIW_: u32 = 0x0018;    
const CRC_: u32 = 0x5B54;        
const CRB_: u32 = 0x0F00; 
const DON_: u32 = 0xE000; 
const BTD_: u32 = 1 << 11;    
const BTC_: u32 = 1 << 12;    
const BTE_: u32 = 1 << 31;   
const DLW_: u32 = 1 << 20; 
const DLX_: u32 = 1 << 15; 





const CC_: usize = 32;
const CX_: usize = 8;
const DV_: usize = 2048;


#[repr(C, align(16))]
#[derive(Clone, Copy)]
struct RxDesc {
    buffer_addr: u64,    
    length: u16,         
    checksum: u16,       
    status: u8,          
    errors: u8,          
    cqw: u16,        
}


#[repr(C, align(16))]
#[derive(Clone, Copy)]
struct TxDesc {
    buffer_addr: u64,    
    length: u16,         
    cso: u8,             
    cmd: u8,             
    status: u8,          
    blj: u8,             
    cqw: u16,        
}

impl Default for RxDesc {
    fn default() -> Self {
        Self {
            buffer_addr: 0,
            length: 0,
            checksum: 0,
            status: 0,
            errors: 0,
            cqw: 0,
        }
    }
}

impl Default for TxDesc {
    fn default() -> Self {
        Self {
            buffer_addr: 0,
            length: 0,
            cso: 0,
            cmd: 0,
            status: AKO_, 
            blj: 0,
            cqw: 0,
        }
    }
}





pub struct E1000Driver {
    status: DriverStatus,
    mmio_base: u64,
    mac: [u8; 6],
    is_ich: bool,       
    is_spt: bool,       
    
    
    rx_descs: Vec<RxDesc>,
    tx_descs: Vec<TxDesc>,
    rx_buffers: Vec<Vec<u8>>,
    tx_buffers: Vec<Vec<u8>>,
    
    
    rx_cur: usize,
    tx_cur: usize,
    
    
    tx_packets: AtomicU64,
    rx_packets: AtomicU64,
    tx_bytes: AtomicU64,
    rx_bytes: AtomicU64,
    tx_errors: AtomicU64,
    rx_errors: AtomicU64,
    
    
    link_up: AtomicBool,
    initialized: AtomicBool,
}

impl E1000Driver {
    pub fn new() -> Self {
        Self {
            status: DriverStatus::Unloaded,
            mmio_base: 0,
            mac: [0x52, 0x54, 0x00, 0xE1, 0x00, 0x00],
            is_ich: false,
            is_spt: false,
            rx_descs: Vec::new(),
            tx_descs: Vec::new(),
            rx_buffers: Vec::new(),
            tx_buffers: Vec::new(),
            rx_cur: 0,
            tx_cur: 0,
            tx_packets: AtomicU64::new(0),
            rx_packets: AtomicU64::new(0),
            tx_bytes: AtomicU64::new(0),
            rx_bytes: AtomicU64::new(0),
            tx_errors: AtomicU64::new(0),
            rx_errors: AtomicU64::new(0),
            link_up: AtomicBool::new(false),
            initialized: AtomicBool::new(false),
        }
    }
    
    
    fn read_reg(&self, offset: u32) -> u32 {
        if self.mmio_base == 0 {
            return 0;
        }
        let addr = (self.mmio_base + offset as u64) as *const u32;
        unsafe { read_volatile(addr) }
    }
    
    
    fn write_reg(&self, offset: u32, value: u32) {
        if self.mmio_base == 0 {
            return;
        }
        let addr = (self.mmio_base + offset as u64) as *mut u32;
        unsafe { write_volatile(addr, value) }
    }
    
    
    fn wk(phys: u64) -> u64 {
        const ED_: u64 = 0xFFFF_8000_0000_0000;
        phys + ED_
    }
    
    
    fn lc(virt: u64) -> u64 {
        const ED_: u64 = 0xFFFF_8000_0000_0000;
        if virt >= ED_ {
            virt - ED_
        } else {
            virt
        }
    }
    
    
    
    fn disable_ulp(&mut self) {
        if !self.is_spt { return; }
        crate::serial_println!("[E1000] Disabling ULP mode for SPT...");
        
        
        let kzy = self.read_reg(AIW_);
        self.write_reg(AIW_, kzy & !0x00000800); 
        
        
        for _ in 0..5000 {
            unsafe { core::arch::asm!("out dx, al", in("dx") 0x80u16, in("al") 0u8, options(nomem, nostack)); }
        }
    }
    
    
    fn reset(&mut self) {
        crate::serial_println!("[E1000] Resetting device (is_ich={}, is_spt={})...", self.is_ich, self.is_spt);
        
        
        self.write_reg(AIX_, 0xFFFFFFFF);
        
        
        self.write_reg(LR_, 0);
        self.write_reg(0x0400, 0x00000008); 
        
        
        let _ = self.read_reg(LS_);
        
        
        for _ in 0..10000 {
            unsafe { core::arch::asm!("out dx, al", in("dx") 0x80u16, in("al") 0u8, options(nomem, nostack)); }
        }
        
        
        if self.is_spt {
            self.disable_ulp();
        }
        
        
        
        
        let ctrl = self.read_reg(JN_);
        self.write_reg(JN_, (ctrl & !BTE_) | ARQ_);
        
        
        for _ in 0..25000 {
            unsafe { core::arch::asm!("out dx, al", in("dx") 0x80u16, in("al") 0u8, options(nomem, nostack)); }
        }
        
        
        for i in 0..500u32 {
            let val = self.read_reg(JN_);
            if val & ARQ_ == 0 {
                crate::serial_println!("[E1000] Reset cleared after {} polls", i);
                break;
            }
            
            for _ in 0..100 {
                unsafe { core::arch::asm!("out dx, al", in("dx") 0x80u16, in("al") 0u8, options(nomem, nostack)); }
            }
        }
        
        
        self.write_reg(AIX_, 0xFFFFFFFF);
        
        let _ = self.read_reg(BGE_);
        
        crate::serial_println!("[E1000] Reset complete");
    }
    
    
    fn read_mac(&mut self) {
        
        let cot = self.read_reg(BGI_);
        let dxf = self.read_reg(BGH_);
        
        if cot != 0 || dxf != 0 {
            self.mac[0] = (cot >> 0) as u8;
            self.mac[1] = (cot >> 8) as u8;
            self.mac[2] = (cot >> 16) as u8;
            self.mac[3] = (cot >> 24) as u8;
            self.mac[4] = (dxf >> 0) as u8;
            self.mac[5] = (dxf >> 8) as u8;
            return;
        }
        
        
        
        
        let (done_bit, addr_shift) = if self.is_ich { (1 << 1, 2) } else { (1 << 4, 8) };
        
        for i in 0..3u32 {
            self.write_reg(BGD_, 1 | (i << addr_shift));
            for _ in 0..1000 {
                let hvd = self.read_reg(BGD_);
                if hvd & done_bit != 0 {
                    let data = (hvd >> 16) as u16;
                    self.mac[i as usize * 2] = (data & 0xFF) as u8;
                    self.mac[i as usize * 2 + 1] = (data >> 8) as u8;
                    break;
                }
                core::hint::spin_loop();
            }
        }
        
        
        if self.mac == [0, 0, 0, 0, 0, 0] {
            self.mac = [0x52, 0x54, 0x00, 0x12, 0x34, 0x56];
        }
    }
    
    
    fn init_rx(&mut self) {
        crate::log_debug!("[E1000] Initializing RX ring ({} descriptors)", CC_);
        
        self.rx_descs = vec![RxDesc::default(); CC_];
        self.rx_buffers = Vec::with_capacity(CC_);
        
        for i in 0..CC_ {
            let buffer = vec![0u8; DV_];
            let phys_addr = Self::lc(buffer.as_ptr() as u64);
            self.rx_descs[i].buffer_addr = phys_addr;
            self.rx_descs[i].status = 0;
            self.rx_buffers.push(buffer);
        }
        
        let dmv = Self::lc(self.rx_descs.as_ptr() as u64);
        
        self.write_reg(CRI_, dmv as u32);
        self.write_reg(CRH_, (dmv >> 32) as u32);
        
        let dxu = (CC_ * core::mem::size_of::<RxDesc>()) as u32;
        self.write_reg(CRK_, dxu);
        
        self.write_reg(CRJ_, 0);
        self.write_reg(BGJ_, (CC_ - 1) as u32);
        
        self.rx_cur = 0;
    }
    
    
    fn init_tx(&mut self) {
        crate::log_debug!("[E1000] Initializing TX ring ({} descriptors)", CX_);
        
        self.tx_descs = vec![TxDesc::default(); CX_];
        self.tx_buffers = Vec::with_capacity(CX_);
        
        for i in 0..CX_ {
            self.tx_buffers.push(vec![0u8; DV_]);
            
            self.tx_descs[i].status = AKO_;
        }
        
        let dmv = Self::lc(self.tx_descs.as_ptr() as u64);
        
        self.write_reg(CRP_, dmv as u32);
        self.write_reg(CRO_, (dmv >> 32) as u32);
        
        let dxu = (CX_ * core::mem::size_of::<TxDesc>()) as u32;
        self.write_reg(CRR_, dxu);
        
        self.write_reg(CRQ_, 0);
        self.write_reg(BGL_, 0);
        
        self.tx_cur = 0;
    }
    
    
    fn enable_rx(&mut self) {
        let dxj = CQO_ | CQR_ | AIQ_ | AIP_ 
                 | CQP_ | CQQ_ | CQM_ 
                 | CQS_ | CQN_;
        self.write_reg(LR_, dxj);
    }
    
    
    fn enable_tx(&mut self) {
        self.write_reg(CRS_, 10 | (8 << 10) | (6 << 20));
        
        let pdq = DBH_ | DBI_ 
                 | (15 << DBG_) 
                 | (64 << DBF_) 
                 | DBJ_;
        self.write_reg(BGK_, pdq);
    }
    
    
    fn setup_link(&mut self) {
        let mut ctrl = self.read_reg(JN_);
        ctrl |= BTF_;  
        if self.is_spt {
            
            ctrl &= !(BTD_ | BTC_);
            ctrl |= ARP_; 
        } else {
            ctrl |= ARP_ | BTB_;
        }
        self.write_reg(JN_, ctrl);
        crate::serial_println!("[E1000] CTRL={:#010X}", self.read_reg(JN_));
        
        
        for i in 0..128 {
            self.write_reg(CRG_ + i * 4, 0);
        }
        
        
        for i in 0..200u32 {
            let status = self.read_reg(LS_);
            if status & AKC_ != 0 {
                self.link_up.store(true, Ordering::SeqCst);
                let speed = match (status & BIL_) >> 6 {
                    0 => 10, 1 => 100, _ => 1000,
                };
                crate::log!("[E1000] Link up at {} Mbps (after {} iterations)", speed, i + 1);
                return;
            }
            
            for _ in 0..1000 {
                unsafe { core::arch::asm!("out dx, al", in("dx") 0x80u16, in("al") 0u8, options(nomem, nostack)); }
            }
        }
        
        crate::log_warn!("[E1000] Link not detected - continuing anyway");
        self.link_up.store(true, Ordering::SeqCst);
    }
}

impl Cw for E1000Driver {
    fn info(&self) -> &Bb {
        &CA_
    }
    
    fn probe(&mut self, pci_device: &L) -> Result<(), &'static str> {
        self.status = DriverStatus::Loading;
        
        
        crate::pci::bzi(pci_device);
        crate::pci::bzj(pci_device);
        crate::serial_println!("[E1000] PCI bus mastering + memory space enabled");
        
        
        self.is_ich = matches!(pci_device.device_id,
            0x1049 | 0x104A | 0x104B | 0x104C | 0x104D |  
            0x10BD | 0x10BF | 0x10C0 | 0x10C2 | 0x10C3 |  
            0x10CB | 0x10CC | 0x10CD | 0x10CE |             
            0x10DE | 0x10DF | 0x10E5 |                       
            0x10EA | 0x10EB | 0x10EF | 0x10F0 | 0x10F5 |    
            0x153A | 0x153B |                                 
            0x15A0 | 0x15A1 | 0x15A2 | 0x15A3 |             
            0x15B7 | 0x15B8 | 0x15B9 |                       
            0x15D6 | 0x15D7 | 0x15D8 |                       
            0x15E3 |                                          
            0x0D4C | 0x0D4D | 0x0D4E | 0x0D4F               
        );
        
        
        self.is_spt = matches!(pci_device.device_id,
            0x15B7 | 0x15B8 | 0x15B9 |                       
            0x15D6 | 0x15D7 | 0x15D8 |                       
            0x15E3 |                                          
            0x0D4C | 0x0D4D | 0x0D4E | 0x0D4F               
        );
        
        let edp = if self.is_spt { "e1000e (SPT)" } else if self.is_ich { "e1000e (ICH)" } else { "e1000" };
        crate::log!("[E1000] Probing {:04X}:{:04X} ({})", pci_device.vendor_id, pci_device.device_id, edp);
        
        let bar0 = pci_device.bar_address(0).ok_or("No BAR0")?;
        if bar0 == 0 { return Err("BAR0 is zero"); }
        
        crate::serial_println!("[E1000] BAR0={:#x}, calling map_mmio...", bar0);
        crate::println!("    [e1000] map_mmio BAR0={:#X}...", bar0);
        
        
        const BVR_: usize = 128 * 1024;
        self.mmio_base = crate::memory::yv(bar0, BVR_)
            .map_err(|e| { crate::serial_println!("[E1000] map_mmio failed: {}", e); "Failed to map E1000 MMIO" })?;
        crate::serial_println!("[E1000] map_mmio returned {:#x}", self.mmio_base);
        crate::println!("    [e1000] map_mmio OK -> {:#X}", self.mmio_base);
        crate::log_debug!("[E1000] MMIO: phys={:#x} virt={:#x}", bar0, self.mmio_base);
        
        crate::println!("    [e1000] reset...");
        self.reset();
        crate::println!("    [e1000] read_mac...");
        self.read_mac();
        
        
        let cot = (self.mac[0] as u32) | ((self.mac[1] as u32) << 8)
                | ((self.mac[2] as u32) << 16) | ((self.mac[3] as u32) << 24);
        let dxf = (self.mac[4] as u32) | ((self.mac[5] as u32) << 8) | (1 << 31);
        self.write_reg(BGI_, cot);
        self.write_reg(BGH_, dxf);
        
        crate::println!("    [e1000] init_rx...");
        self.init_rx();
        crate::println!("    [e1000] init_tx...");
        self.init_tx();
        crate::println!("    [e1000] setup_link...");
        self.setup_link();
        crate::println!("    [e1000] enable_rx/tx...");
        self.enable_rx();
        self.enable_tx();
        
        self.initialized.store(true, Ordering::SeqCst);
        self.status = DriverStatus::Running;
        
        crate::log!("[E1000] MAC: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            self.mac[0], self.mac[1], self.mac[2], self.mac[3], self.mac[4], self.mac[5]);
        
        
        crate::serial_println!("[E1000] STATUS={:#010X} CTRL={:#010X} RCTL={:#010X} TCTL={:#010X}",
            self.read_reg(LS_), self.read_reg(JN_),
            self.read_reg(LR_), self.read_reg(0x0400));
        if self.is_spt {
            crate::serial_println!("[E1000] FWSM={:#010X} CTRL_EXT={:#010X} EXTCNF_CTRL={:#010X}",
                self.read_reg(CRC_), self.read_reg(AIW_),
                self.read_reg(CRB_));
        }
        
        Ok(())
    }
    
    fn start(&mut self) -> Result<(), &'static str> {
        self.status = DriverStatus::Running;
        Ok(())
    }
    
    fn stop(&mut self) -> Result<(), &'static str> {
        self.write_reg(LR_, 0);
        self.write_reg(BGK_, 0);
        self.write_reg(AIX_, 0xFFFFFFFF);
        self.status = DriverStatus::Suspended;
        Ok(())
    }
    
    fn status(&self) -> DriverStatus {
        self.status
    }
}

impl Dd for E1000Driver {
    fn mac_address(&self) -> [u8; 6] {
        self.mac
    }
    
    fn link_up(&self) -> bool {
        if self.mmio_base != 0 {
            let status = self.read_reg(LS_);
            status & AKC_ != 0
        } else {
            self.link_up.load(Ordering::Relaxed)
        }
    }
    
    fn cbj(&self) -> u32 {
        if self.mmio_base == 0 { return 0; }
        let status = self.read_reg(LS_);
        match (status & BIL_) >> 6 {
            0 => 10, 1 => 100, _ => 1000,
        }
    }
    
    fn send(&mut self, data: &[u8]) -> Result<(), &'static str> {
        if !self.initialized.load(Ordering::Relaxed) {
            return Err("Driver not initialized");
        }
        if data.len() > DV_ { return Err("Packet too large"); }
        if data.len() < 14 { return Err("Packet too small"); }
        
        let tx = self.tx_cur;
        
        
        let mut mz = 10000;
        while self.tx_descs[tx].status & AKO_ == 0 {
            mz -= 1;
            if mz == 0 {
                self.tx_errors.fetch_add(1, Ordering::Relaxed);
                return Err("TX timeout");
            }
            core::hint::spin_loop();
        }
        
        
        let buffer = &mut self.tx_buffers[tx];
        buffer[..data.len()].copy_from_slice(data);
        
        
        let phys_addr = Self::lc(buffer.as_ptr() as u64);
        self.tx_descs[tx].buffer_addr = phys_addr;
        self.tx_descs[tx].length = data.len() as u16;
        self.tx_descs[tx].cmd = DBK_ | DBL_ | DBM_;
        self.tx_descs[tx].status = 0;
        
        
        self.tx_cur = (self.tx_cur + 1) % CX_;
        self.write_reg(BGL_, self.tx_cur as u32);
        
        self.tx_packets.fetch_add(1, Ordering::Relaxed);
        self.tx_bytes.fetch_add(data.len() as u64, Ordering::Relaxed);
        
        Ok(())
    }
    
    fn receive(&mut self) -> Option<Vec<u8>> {
        if !self.initialized.load(Ordering::Relaxed) { return None; }
        
        let tx = self.rx_cur;
        let status = self.rx_descs[tx].status;
        
        if status & CQT_ == 0 { return None; }
        
        
        
        if self.rx_descs[tx].errors != 0 {
            self.rx_errors.fetch_add(1, Ordering::Relaxed);
            self.rx_descs[tx].status = 0;
            self.rx_cur = (self.rx_cur + 1) % CC_;
            return None;
        }
        
        let length = self.rx_descs[tx].length as usize;
        if length == 0 || length > DV_ {
            self.rx_descs[tx].status = 0;
            self.rx_cur = (self.rx_cur + 1) % CC_;
            return None;
        }
        
        let be = self.rx_buffers[tx][..length].to_vec();
        
        self.rx_descs[tx].status = 0;
        self.rx_descs[tx].length = 0;
        self.write_reg(BGJ_, tx as u32);
        self.rx_cur = (self.rx_cur + 1) % CC_;
        
        self.rx_packets.fetch_add(1, Ordering::Relaxed);
        self.rx_bytes.fetch_add(length as u64, Ordering::Relaxed);
        
        Some(be)
    }
    
    fn poll(&mut self) {
        if !self.initialized.load(Ordering::Relaxed) { return; }
        let icr = self.read_reg(BGE_);
        if icr & CER_ != 0 {
            let status = self.read_reg(LS_);
            self.link_up.store(status & AKC_ != 0, Ordering::SeqCst);
        }
    }
    
    fn stats(&self) -> NetStats {
        NetStats {
            tx_packets: self.tx_packets.load(Ordering::Relaxed),
            rx_packets: self.rx_packets.load(Ordering::Relaxed),
            tx_bytes: self.tx_bytes.load(Ordering::Relaxed),
            rx_bytes: self.rx_bytes.load(Ordering::Relaxed),
            tx_errors: self.tx_errors.load(Ordering::Relaxed),
            rx_errors: self.rx_errors.load(Ordering::Relaxed),
            tx_dropped: 0,
            rx_dropped: 0,
        }
    }
    
    fn jfi(&mut self, enabled: bool) -> Result<(), &'static str> {
        if !self.initialized.load(Ordering::Relaxed) { return Err("Not initialized"); }
        let mut dxj = self.read_reg(LR_);
        if enabled { dxj |= AIQ_ | AIP_; } 
        else { dxj &= !(AIQ_ | AIP_); }
        self.write_reg(LR_, dxj);
        Ok(())
    }
}

const CA_: Bb = Bb {
    name: "e1000",
    version: "1.0.0",
    author: "TrustOS Team",
    category: DriverCategory::Network,
    vendor_ids: &[
        (0x8086, 0x100E),  
        (0x8086, 0x100F),  
        (0x8086, 0x10D3),  
        (0x8086, 0x153A),  
        (0x8086, 0x153B),  
        (0x8086, 0x1533),  
        
        (0x8086, 0x15A0),  
        (0x8086, 0x15A1),  
        (0x8086, 0x15A2),  
        (0x8086, 0x15A3),  
        (0x8086, 0x15B7),  
        (0x8086, 0x15B8),  
        (0x8086, 0x15B9),  
        (0x8086, 0x15D6),  
        (0x8086, 0x15D7),  
        (0x8086, 0x15D8),  
        (0x8086, 0x15E3),  
        (0x8086, 0x0D4E),  
        (0x8086, 0x0D4F),  
        (0x8086, 0x0D4C),  
        (0x8086, 0x0D4D),  
        
        (0x8086, 0x1049),  
        (0x8086, 0x104A),  
        (0x8086, 0x104B),  
        (0x8086, 0x104C),  
        (0x8086, 0x104D),  
        (0x8086, 0x10BD),  
        (0x8086, 0x10BF),  
        (0x8086, 0x10C0),  
        (0x8086, 0x10C2),  
        (0x8086, 0x10C3),  
        (0x8086, 0x10CB),  
        (0x8086, 0x10CC),  
        (0x8086, 0x10CD),  
        (0x8086, 0x10CE),  
        (0x8086, 0x10DE),  
        (0x8086, 0x10DF),  
        (0x8086, 0x10E5),  
        (0x8086, 0x10EA),  
        (0x8086, 0x10EB),  
        (0x8086, 0x10EF),  
        (0x8086, 0x10F0),  
        (0x8086, 0x10F5),  
    ],
};

pub fn register() {
    crate::drivers::register(CA_, || {
        Box::new(E1000Driver::new())
    });
    crate::drivers::net::eyh(CA_, || {
        Box::new(E1000Driver::new())
    });
}
