











use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::vec;
use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use super::{Dd, NetStats};
use crate::drivers::{Cw, Bb, DriverStatus, DriverCategory};
use crate::pci::L;





const CRE_: u32       = 0x00;  
const CRF_: u32       = 0x04;  
const CRT_: u32      = 0x20;  
const CRU_: u32   = 0x24;  
const XX_: u32         = 0x37;  
const CRV_: u32      = 0x38;  
const BGF_: u32         = 0x3C;  
const BGG_: u32         = 0x3E;  
const CRW_: u32   = 0x40;  
const AIY_: u32   = 0x44;  
const EFW_: u32         = 0x4C;  
const BFY_: u32      = 0x50;  
const EFT_: u32     = 0x52;  
const XY_: u32  = 0x6C;  
const CRN_: u32 = 0xDA;  
const CQZ_: u32        = 0xE0;  
const CRL_: u32       = 0xE4;  
const CRM_: u32    = 0xE8;  
const CRA_: u32 = 0xEC; 






const AQA_: u8 = 0x10;
const BPI_: u8 = 0x08;
const BPJ_: u8 = 0x04;


const DBZ_: u8 = 0x40;  


const CFH_: u16 = 0x0001;    
const CFJ_: u16 = 0x0004;    
const AZE_: u16 = 0x0020; 
const CFI_: u16 = 0x0010;
const CFG_: u16 = CFH_ | CFJ_ | AZE_ | CFI_;


const DCT_: u32 = 0x03 << 24;  
const DCS_: u32 = 0x07 << 8; 


const BGY_: u32 = 1 << 0;   
const CTL_: u32 = 1 << 1;   
const CTK_: u32  = 1 << 2;   
const CTJ_: u32  = 1 << 3;   
const EHB_: u32 = 1 << 7;  
const CTM_: u32 = 0x07 << 8; 
const CTN_: u32 = 0x07 << 13; 


const DKQ_: u16 = 1 << 6;
const BRN_: u16 = 1 << 5;
const BRM_: u16 = 1 << 3;


const BOX_: u8 = 0xC0;
const BOW_: u8   = 0x00;


const AIF_: u32 = 0x02;
const BFE_: u32 = 0x10;
const BFF_: u32 = 0x08;
const CMU_: u32 = 0x04;





const CC_: usize = 64;
const CX_: usize = 64;
const DV_: usize = 2048;


const NQ_: u32  = 1 << 31;  
const TV_: u32  = 1 << 30;  
const ACP_: u32   = 1 << 29;  
const ACQ_: u32   = 1 << 28;  


#[repr(C, align(16))]
#[derive(Clone, Copy)]
struct Descriptor {
    opts1: u32,  
    opts2: u32,  
    buf_lo: u32, 
    buf_hi: u32, 
}

impl Default for Descriptor {
    fn default() -> Self {
        Self {
            opts1: 0,
            opts2: 0,
            buf_lo: 0,
            buf_hi: 0,
        }
    }
}





pub struct Rtl8169Driver {
    status: DriverStatus,
    mmio_base: u64,
    mac: [u8; 6],

    
    rx_descs: Vec<Descriptor>,
    tx_descs: Vec<Descriptor>,
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

impl Rtl8169Driver {
    pub fn new() -> Self {
        Self {
            status: DriverStatus::Unloaded,
            mmio_base: 0,
            mac: [0x52, 0x54, 0x00, 0x81, 0x69, 0x00],
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

    

    fn read8(&self, offset: u32) -> u8 {
        if self.mmio_base == 0 { return 0; }
        let addr = (self.mmio_base + offset as u64) as *const u8;
        unsafe { read_volatile(addr) }
    }

    fn write8(&self, offset: u32, val: u8) {
        if self.mmio_base == 0 { return; }
        let addr = (self.mmio_base + offset as u64) as *mut u8;
        unsafe { write_volatile(addr, val); }
    }

    fn read16(&self, offset: u32) -> u16 {
        if self.mmio_base == 0 { return 0; }
        let addr = (self.mmio_base + offset as u64) as *const u16;
        unsafe { read_volatile(addr) }
    }

    fn write16(&self, offset: u32, val: u16) {
        if self.mmio_base == 0 { return; }
        let addr = (self.mmio_base + offset as u64) as *mut u16;
        unsafe { write_volatile(addr, val); }
    }

    fn read32(&self, offset: u32) -> u32 {
        if self.mmio_base == 0 { return 0; }
        let addr = (self.mmio_base + offset as u64) as *const u32;
        unsafe { read_volatile(addr) }
    }

    fn write32(&self, offset: u32, val: u32) {
        if self.mmio_base == 0 { return; }
        let addr = (self.mmio_base + offset as u64) as *mut u32;
        unsafe { write_volatile(addr, val); }
    }

    
    fn lc(virt: u64) -> u64 {
        const ED_: u64 = 0xFFFF_8000_0000_0000;
        if virt >= ED_ { virt - ED_ } else { virt }
    }

    
    fn reset(&self) {
        crate::log_debug!("[RTL8169] Resetting controller...");

        self.write8(XX_, AQA_);

        
        for _ in 0..10_000 {
            if self.read8(XX_) & AQA_ == 0 {
                crate::log_debug!("[RTL8169] Reset complete");
                return;
            }
            for _ in 0..1000 { core::hint::spin_loop(); }
        }

        crate::log_warn!("[RTL8169] Reset timeout — continuing anyway");
    }

    
    fn read_mac(&mut self) {
        let lo = self.read32(CRE_);
        let hi = self.read32(CRF_);

        self.mac[0] = (lo >> 0) as u8;
        self.mac[1] = (lo >> 8) as u8;
        self.mac[2] = (lo >> 16) as u8;
        self.mac[3] = (lo >> 24) as u8;
        self.mac[4] = (hi >> 0) as u8;
        self.mac[5] = (hi >> 8) as u8;

        
        if self.mac == [0; 6] {
            self.mac = [0x52, 0x54, 0x00, 0x12, 0x81, 0x69];
        }
    }

    
    fn unlock_config(&self) {
        self.write8(BFY_, BOX_);
    }

    
    fn lock_config(&self) {
        self.write8(BFY_, BOW_);
    }

    
    fn init_rx(&mut self) {
        crate::log_debug!("[RTL8169] Initializing RX ring ({} descriptors)", CC_);

        self.rx_descs = vec![Descriptor::default(); CC_];
        self.rx_buffers = Vec::with_capacity(CC_);

        for i in 0..CC_ {
            let buffer = vec![0u8; DV_];
            let phys = Self::lc(buffer.as_ptr() as u64);

            let mut flags = NQ_ | (DV_ as u32 & 0x3FFF);
            if i == CC_ - 1 {
                flags |= TV_; 
            }

            self.rx_descs[i].opts1 = flags;
            self.rx_descs[i].opts2 = 0;
            self.rx_descs[i].buf_lo = phys as u32;
            self.rx_descs[i].buf_hi = (phys >> 32) as u32;

            self.rx_buffers.push(buffer);
        }

        
        let ring_phys = Self::lc(self.rx_descs.as_ptr() as u64);
        self.write32(CRL_, ring_phys as u32);
        self.write32(CRM_, (ring_phys >> 32) as u32);

        self.rx_cur = 0;
    }

    
    fn init_tx(&mut self) {
        crate::log_debug!("[RTL8169] Initializing TX ring ({} descriptors)", CX_);

        self.tx_descs = vec![Descriptor::default(); CX_];
        self.tx_buffers = Vec::with_capacity(CX_);

        for i in 0..CX_ {
            let buffer = vec![0u8; DV_];

            let mut flags = 0u32;
            if i == CX_ - 1 {
                flags |= TV_; 
            }

            self.tx_descs[i].opts1 = flags;
            self.tx_descs[i].opts2 = 0;

            let phys = Self::lc(buffer.as_ptr() as u64);
            self.tx_descs[i].buf_lo = phys as u32;
            self.tx_descs[i].buf_hi = (phys >> 32) as u32;

            self.tx_buffers.push(buffer);
        }

        
        let ring_phys = Self::lc(self.tx_descs.as_ptr() as u64);
        self.write32(CRT_, ring_phys as u32);
        self.write32(CRU_, (ring_phys >> 32) as u32);

        self.tx_cur = 0;
    }

    
    fn enable(&mut self) {
        
        self.unlock_config();

        
        let kyj = BRM_ | BRN_;
        self.write16(CQZ_, kyj);

        
        self.write16(CRN_, DV_ as u16);

        
        self.write32(CRW_, DCT_ | DCS_);

        
        let dye = CTL_ | CTJ_ | CTK_ | CTM_ | CTN_;
        self.write32(AIY_, dye);

        
        self.write8(CRA_, 0x3F);

        
        self.lock_config();

        
        self.write8(XX_, BPI_ | BPJ_);

        
        self.write16(BGF_, CFG_);

        crate::log_debug!("[RTL8169] Controller enabled (RX+TX)");
    }

    
    fn check_link(&mut self) {
        let buu = self.read32(XY_);
        let up = buu & AIF_ != 0;
        self.link_up.store(up, Ordering::SeqCst);

        if up {
            let speed = if buu & BFE_ != 0 {
                1000
            } else if buu & BFF_ != 0 {
                100
            } else {
                10
            };
            crate::log!("[RTL8169] Link up at {} Mbps", speed);
        }
    }
}





impl Cw for Rtl8169Driver {
    fn info(&self) -> &Bb {
        &CA_
    }

    fn probe(&mut self, pci_device: &L) -> Result<(), &'static str> {
        self.status = DriverStatus::Loading;

        crate::log!("[RTL8169] Probing {:04X}:{:04X}", pci_device.vendor_id, pci_device.device_id);

        
        let bar0 = pci_device.bar_address(0).ok_or("No BAR0")?;
        if bar0 == 0 { return Err("BAR0 is zero"); }

        
        const CTE_: usize = 4096;
        self.mmio_base = crate::memory::yv(bar0, CTE_)
            .map_err(|e| {
                crate::serial_println!("[RTL8169] map_mmio failed: {}", e);
                "Failed to map RTL8169 MMIO"
            })?;

        crate::log_debug!("[RTL8169] MMIO: phys={:#x} virt={:#x}", bar0, self.mmio_base);

        
        self.reset();

        
        self.read_mac();

        
        self.init_rx();
        self.init_tx();

        
        self.enable();

        
        self.check_link();

        
        if !self.link_up.load(Ordering::Relaxed) {
            crate::log_warn!("[RTL8169] Link not detected — assuming up (QEMU mode)");
            self.link_up.store(true, Ordering::SeqCst);
        }

        self.initialized.store(true, Ordering::SeqCst);
        self.status = DriverStatus::Running;

        crate::log!("[RTL8169] MAC: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            self.mac[0], self.mac[1], self.mac[2], self.mac[3], self.mac[4], self.mac[5]);

        Ok(())
    }

    fn start(&mut self) -> Result<(), &'static str> {
        self.status = DriverStatus::Running;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), &'static str> {
        
        self.write8(XX_, 0);
        
        self.write16(BGF_, 0);
        self.status = DriverStatus::Suspended;
        Ok(())
    }

    fn status(&self) -> DriverStatus {
        self.status
    }
}





impl Dd for Rtl8169Driver {
    fn mac_address(&self) -> [u8; 6] {
        self.mac
    }

    fn link_up(&self) -> bool {
        if self.mmio_base != 0 {
            self.read32(XY_) & AIF_ != 0
        } else {
            self.link_up.load(Ordering::Relaxed)
        }
    }

    fn cbj(&self) -> u32 {
        if self.mmio_base == 0 { return 0; }
        let buu = self.read32(XY_);
        if buu & BFE_ != 0 { 1000 }
        else if buu & BFF_ != 0 { 100 }
        else if buu & CMU_ != 0 { 10 }
        else { 0 }
    }

    fn send(&mut self, data: &[u8]) -> Result<(), &'static str> {
        if !self.initialized.load(Ordering::Relaxed) {
            return Err("Driver not initialized");
        }
        if data.len() > DV_ { return Err("Packet too large"); }
        if data.len() < 14 { return Err("Packet too small"); }

        let idx = self.tx_cur;

        
        let mut mz = 10_000;
        while self.tx_descs[idx].opts1 & NQ_ != 0 {
            mz -= 1;
            if mz == 0 {
                self.tx_errors.fetch_add(1, Ordering::Relaxed);
                return Err("TX timeout — descriptor still owned by NIC");
            }
            core::hint::spin_loop();
        }

        
        let buffer = &mut self.tx_buffers[idx];
        buffer[..data.len()].copy_from_slice(data);

        
        let phys = Self::lc(buffer.as_ptr() as u64);
        self.tx_descs[idx].buf_lo = phys as u32;
        self.tx_descs[idx].buf_hi = (phys >> 32) as u32;

        
        let mut flags = NQ_ | ACP_ | ACQ_ | (data.len() as u32 & 0x3FFF);
        if idx == CX_ - 1 {
            flags |= TV_;
        }
        self.tx_descs[idx].opts1 = flags;
        self.tx_descs[idx].opts2 = 0;

        
        self.write8(CRV_, DBZ_);

        
        self.tx_cur = (self.tx_cur + 1) % CX_;

        self.tx_packets.fetch_add(1, Ordering::Relaxed);
        self.tx_bytes.fetch_add(data.len() as u64, Ordering::Relaxed);

        Ok(())
    }

    fn receive(&mut self) -> Option<Vec<u8>> {
        if !self.initialized.load(Ordering::Relaxed) { return None; }

        let idx = self.rx_cur;
        let opts1 = self.rx_descs[idx].opts1;

        
        if opts1 & NQ_ != 0 {
            return None;
        }

        
        if opts1 & (ACP_ | ACQ_) != (ACP_ | ACQ_) {
            
            self.rx_errors.fetch_add(1, Ordering::Relaxed);
            self.reclaim_rx(idx);
            return None;
        }

        
        let length = (opts1 & 0x3FFF) as usize;
        if length < 4 || length > DV_ {
            self.rx_errors.fetch_add(1, Ordering::Relaxed);
            self.reclaim_rx(idx);
            return None;
        }

        let aup = length - 4; 
        if aup == 0 {
            self.reclaim_rx(idx);
            return None;
        }

        
        let be = self.rx_buffers[idx][..aup].to_vec();

        
        self.reclaim_rx(idx);

        self.rx_packets.fetch_add(1, Ordering::Relaxed);
        self.rx_bytes.fetch_add(aup as u64, Ordering::Relaxed);

        Some(be)
    }

    fn poll(&mut self) {
        if !self.initialized.load(Ordering::Relaxed) { return; }

        
        let isr = self.read16(BGG_);
        if isr != 0 {
            self.write16(BGG_, isr); 
        }

        
        if isr & AZE_ != 0 {
            let buu = self.read32(XY_);
            self.link_up.store(buu & AIF_ != 0, Ordering::SeqCst);
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
        let mut dye = self.read32(AIY_);
        if enabled {
            dye |= BGY_; 
        } else {
            dye &= !BGY_;
        }
        self.write32(AIY_, dye);
        Ok(())
    }
}

impl Rtl8169Driver {
    
    fn reclaim_rx(&mut self, idx: usize) {
        let mut flags = NQ_ | (DV_ as u32 & 0x3FFF);
        if idx == CC_ - 1 {
            flags |= TV_;
        }
        self.rx_descs[idx].opts1 = flags;
        self.rx_descs[idx].opts2 = 0;
        self.rx_cur = (self.rx_cur + 1) % CC_;
    }
}





const CA_: Bb = Bb {
    name: "rtl8169",
    version: "1.0.0",
    author: "TrustOS Team",
    category: DriverCategory::Network,
    vendor_ids: &[
        (0x10EC, 0x8169),  
        (0x10EC, 0x8168),  
        (0x10EC, 0x8161),  
        (0x10EC, 0x8136),  
    ],
};

pub fn register() {
    crate::drivers::register(CA_, || {
        Box::new(Rtl8169Driver::new())
    });
    crate::drivers::net::eyh(CA_, || {
        Box::new(Rtl8169Driver::new())
    });
}


pub fn is_initialized() -> bool {
    
    crate::drivers::net::aoh()
}
