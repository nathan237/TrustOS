




use alloc::vec::Vec;
use alloc::vec;
use alloc::boxed::Box;
use alloc::collections::VecDeque;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex;
use crate::arch::Port;

use crate::virtio::{self, VirtioDevice, Virtqueue, VirtqDesc, desc_flags, status, legacy_reg};
use crate::pci::L;


pub mod features {
    pub const Ato: u32 = 1 << 0;           
    pub const DSH_: u32 = 1 << 1;     
    pub const Aay: u32 = 1 << 5;            
    pub const Awu: u32 = 1 << 6;            
    pub const DSJ_: u32 = 1 << 7;     
    pub const DSK_: u32 = 1 << 8;     
    pub const DSI_: u32 = 1 << 9;      
    pub const DSL_: u32 = 1 << 10;     
    pub const DTA_: u32 = 1 << 11;     
    pub const DTB_: u32 = 1 << 12;     
    pub const DSZ_: u32 = 1 << 13;      
    pub const DTC_: u32 = 1 << 14;      
    pub const DYO_: u32 = 1 << 15;     
    pub const Fz: u32 = 1 << 16;        
    pub const DMA_: u32 = 1 << 17;       
    pub const DLY_: u32 = 1 << 18;       
    pub const DLZ_: u32 = 1 << 19;     
    pub const DSG_: u32 = 1 << 21; 
}


#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Default)]
pub struct VirtioNetHdr {
    pub flags: u8,
    pub gso_type: u8,
    pub hdr_len: u16,
    pub gso_size: u16,
    pub csum_start: u16,
    pub csum_offset: u16,
}

impl VirtioNetHdr {
    pub const Z: usize = 10;
    
    pub fn new() -> Self {
        Self::default()
    }
}


#[repr(C)]
struct Nc {
    header: VirtioNetHdr,
    data: [u8; 1514],  
}


#[repr(C)]
struct Vn {
    header: VirtioNetHdr,
    data: [u8; 1514],
}


pub struct VirtioNet {
    
    device: VirtioDevice,
    
    rx_queue: Option<Box<Virtqueue>>,
    
    tx_queue: Option<Box<Virtqueue>>,
    
    mac: [u8; 6],
    
    link_up: bool,
    
    rx_buffers: Vec<Box<Nc>>,
    
    tx_buffers: Vec<Box<Vn>>,
    
    tx_pending: VecDeque<u16>,
}


static Cl: Mutex<Option<VirtioNet>> = Mutex::new(None);
static Ah: AtomicBool = AtomicBool::new(false);


static BGZ_: Mutex<VecDeque<Vec<u8>>> = Mutex::new(VecDeque::new());


static ALJ_: core::sync::atomic::AtomicU16 = core::sync::atomic::AtomicU16::new(0);

static ALK_: AtomicBool = AtomicBool::new(false);


static BEL_: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
static BEK_: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
static APP_: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
static APO_: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);

impl VirtioNet {
    
    pub fn new(go: &L) -> Result<Self, &'static str> {
        
        let bar0 = go.bar[0];
        if bar0 == 0 {
            return Err("BAR0 not configured");
        }
        
        
        let iobase = if bar0 & 1 == 1 {
            (bar0 & 0xFFFC) as u16  
        } else {
            return Err("MMIO not supported yet, need I/O port BAR");
        };
        
        crate::log_debug!("[virtio-net] I/O base: {:#X}", iobase);
        
        let mut device = VirtioDevice::new(iobase);
        
        
        device.reset();
        
        
        device.add_status(status::Gf);
        device.add_status(status::Cl);
        
        
        let device_features = device.read_device_features();
        crate::log_debug!("[virtio-net] Device features: {:#X}", device_features);
        
        
        let mut driver_features = 0u32;
        if device_features & features::Aay != 0 {
            driver_features |= features::Aay;
        }
        if device_features & features::Fz != 0 {
            driver_features |= features::Fz;
        }
        
        device.write_driver_features(driver_features);
        crate::log_debug!("[virtio-net] Driver features: {:#X}", driver_features);
        
        
        let mut mac = [0u8; 6];
        for i in 0..6 {
            mac[i] = device.read_config8(i as u16);
        }
        crate::log!("[virtio-net] MAC: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);
        
        
        let link_up = if driver_features & features::Fz != 0 {
            let nim = device.read_config16(6);
            nim & 1 != 0
        } else {
            true 
        };
        
        crate::log!("[virtio-net] Link: {}", if link_up { "UP" } else { "DOWN" });
        
        Ok(Self {
            device,
            rx_queue: None,
            tx_queue: None,
            mac,
            link_up,
            rx_buffers: Vec::new(),
            tx_buffers: Vec::new(),
            tx_pending: VecDeque::new(),
        })
    }
    
    
    pub fn setup_queues(&mut self) -> Result<(), &'static str> {
        
        self.device.select_queue(0);
        let gsf = self.device.get_queue_size();
        crate::log_debug!("[virtio-net] RX queue size: {}", gsf);
        
        if gsf == 0 {
            return Err("RX queue not available");
        }
        
        let rx_queue = self.alloc_queue(gsf)?;
        
        
        let bog = (rx_queue.phys_addr / 4096) as u32;
        self.device.set_queue_address(bog);
        
        self.rx_queue = Some(rx_queue);
        
        
        self.device.select_queue(1);
        let hag = self.device.get_queue_size();
        crate::log_debug!("[virtio-net] TX queue size: {}", hag);
        
        if hag == 0 {
            return Err("TX queue not available");
        }
        
        let tx_queue = self.alloc_queue(hag)?;
        
        let bog = (tx_queue.phys_addr / 4096) as u32;
        self.device.set_queue_address(bog);
        
        self.tx_queue = Some(tx_queue);
        
        Ok(())
    }
    
    
    fn alloc_queue(&mut self, size: u16) -> Result<Box<Virtqueue>, &'static str> {
        let total_size = Virtqueue::hjs(size);
        
        
        
        let layout = core::alloc::Layout::from_size_align(total_size, 4096)
            .map_err(|_| "Layout error")?;
        
        let ptr = unsafe { alloc::alloc::alloc_zeroed(layout) };
        if ptr.is_null() {
            return Err("Failed to allocate queue memory");
        }
        
        
        let ldp = ptr as *mut VirtqDesc;
        let cfz = core::mem::size_of::<VirtqDesc>() * size as usize;
        let jyp = unsafe { ptr.add(cfz) as *mut virtio::Kx };
        let bps = ((cfz + 6 + 2 * size as usize) + 4095) & !4095;
        let pqp = unsafe { ptr.add(bps) as *mut virtio::Ky };
        
        
        let virt_addr = ptr as u64;
        let hhdm_offset = crate::memory::hhdm_offset();
        let phys_addr = if virt_addr >= hhdm_offset {
            virt_addr - hhdm_offset
        } else {
            
            virt_addr
        };
        
        
        let mut free_list = vec![0u16; size as usize];
        for i in 0..(size - 1) {
            free_list[i as usize] = i + 1;
        }
        free_list[(size - 1) as usize] = 0xFFFF; 
        
        Ok(Box::new(Virtqueue {
            size,
            phys_addr,
            desc: ldp,
            avail: jyp,
            used: pqp,
            last_used_idx: 0,
            free_head: 0,
            num_free: size,
            free_list,
        }))
    }
    
    
    pub fn setup_rx_buffers(&mut self) -> Result<(), &'static str> {
        let queue = self.rx_queue.as_mut().ok_or("RX queue not initialized")?;
        
        
        let irj = (queue.size / 2).min(128) as usize; 
        
        for _ in 0..irj {
            let buffer = Box::new(Nc {
                header: VirtioNetHdr::new(),
                data: [0u8; 1514],
            });
            
            let tx = queue.alloc_desc().ok_or("No free descriptors")?;
            
            
            let buffer_ptr = &*buffer as *const Nc;
            let virt_addr = buffer_ptr as u64;
            let bz = crate::memory::hhdm_offset();
            let phys_addr = if virt_addr >= bz { virt_addr - bz } else { virt_addr };
            
            
            unsafe {
                let desc = &mut *queue.desc.add(tx as usize);
                desc.addr = phys_addr;
                desc.len = core::mem::size_of::<Nc>() as u32;
                desc.flags = desc_flags::Bh; 
                desc.next = 0;
                
                
                queue.add_available(tx);
            }
            
            self.rx_buffers.push(buffer);
        }
        
        crate::log_debug!("[virtio-net] {} RX buffers ready", irj);
        
        Ok(())
    }
    
    
    pub fn start(&mut self) -> Result<(), &'static str> {
        
        self.device.add_status(status::IQ_);
        
        
        self.device.notify_queue(0);
        
        crate::log!("[virtio-net] Device started");
        Ok(())
    }
    
    
    pub fn send(&mut self, data: &[u8]) -> Result<(), &'static str> {
        if data.len() > 1514 {
            return Err("Packet too large");
        }
        
        
        self.poll_tx_internal();
        
        let queue = self.tx_queue.as_mut().ok_or("TX queue not initialized")?;
        
        
        let mut buffer = Box::new(Vn {
            header: VirtioNetHdr::new(),
            data: [0u8; 1514],
        });
        
        
        buffer.data[..data.len()].copy_from_slice(data);
        
        
        let tx = queue.alloc_desc().ok_or("TX queue full")?;
        
        
        let buffer_ptr = &*buffer as *const Vn;
        let virt_addr = buffer_ptr as u64;
        let bz = crate::memory::hhdm_offset();
        let phys_addr = if virt_addr >= bz { virt_addr - bz } else { virt_addr };
        
        
        unsafe {
            let desc = &mut *queue.desc.add(tx as usize);
            desc.addr = phys_addr;
            desc.len = (VirtioNetHdr::Z + data.len()) as u32;
            desc.flags = 0; 
            desc.next = 0;
            
            queue.add_available(tx);
        }
        
        
        self.tx_buffers.push(buffer);
        self.tx_pending.push_back(tx);
        
        
        self.device.notify_queue(1);
        
        
        BEL_.fetch_add(1, Ordering::Relaxed);
        APP_.fetch_add(data.len() as u64, Ordering::Relaxed);
        
        Ok(())
    }
    
    
    pub fn poll_rx(&mut self) -> Option<Vec<u8>> {
        let queue = self.rx_queue.as_mut()?;
        
        unsafe {
            if !queue.has_used() {
                return None;
            }
            
            let used = queue.pop_used()?;
            
            
            let tx = used.0 as u16;
            let desc = &*queue.desc.add(tx as usize);
            
            
            let buffer_ptr = (crate::memory::hhdm_offset() + desc.addr) as *const Nc;
            let buffer = &*buffer_ptr;
            
            let atl = (used.1 as usize).saturating_sub(VirtioNetHdr::Z);
            if atl > 0 && atl <= 1514 {
                let be = buffer.data[..atl].to_vec();
                
                
                queue.add_available(tx);
                self.device.notify_queue(0);
                
                
                BEK_.fetch_add(1, Ordering::Relaxed);
                APO_.fetch_add(atl as u64, Ordering::Relaxed);
                
                return Some(be);
            }
            
            
            queue.add_available(tx);
            self.device.notify_queue(0);
        }
        
        None
    }
    
    
    fn poll_tx_internal(&mut self) {
        let queue = match self.tx_queue.as_mut() {
            Some(q) => q,
            None => return,
        };
        
        unsafe {
            while queue.has_used() {
                if let Some(used) = queue.pop_used() {
                    
                    queue.free_desc(used.0 as u16);
                    
                    
                    if let Some(pos) = self.tx_pending.iter().position(|&x| x == used.0 as u16) {
                        self.tx_pending.remove(pos);
                    }
                }
            }
        }
        
        
        while self.tx_buffers.len() > 16 && !self.tx_pending.is_empty() {
            self.tx_buffers.remove(0);
        }
    }
    
    
    pub fn poll_tx(&mut self) {
        self.poll_tx_internal();
    }
    
    
    pub fn mac(&self) -> [u8; 6] {
        self.mac
    }
    
    
    pub fn is_link_up(&self) -> bool {
        self.link_up
    }
    
    
    pub fn iobase(&self) -> u16 {
        self.device.iobase
    }
}




pub fn init(go: &L) -> Result<(), &'static str> {
    crate::log!("[virtio-net] Initializing...");
    
    let mut driver = VirtioNet::new(go)?;
    driver.setup_queues()?;
    driver.setup_rx_buffers()?;
    driver.start()?;
    
    
    ALJ_.store(driver.device.iobase, Ordering::SeqCst);
    
    Ah.store(true, Ordering::SeqCst);
    *Cl.lock() = Some(driver);
    
    
    let irq = go.interrupt_line;
    if irq > 0 && irq < 255 {
        crate::apic::eyz(irq, crate::apic::HZ_);
        crate::serial_println!("[virtio-net] IRQ {} routed to vector {}", irq, crate::apic::HZ_);
    }
    
    Ok(())
}


pub fn is_initialized() -> bool {
    Ah.load(Ordering::Relaxed)
}


pub fn aqt() -> Option<[u8; 6]> {
    Cl.lock().as_ref().map(|d| d.mac())
}


pub fn aha(data: &[u8]) -> Result<(), &'static str> {
    let mut driver = Cl.lock();
    let tz = driver.as_mut().ok_or("Driver not initialized")?;
    tz.send(data)
}


pub fn poll() {
    
    ALK_.store(false, Ordering::Relaxed);
    
    let mut driver = Cl.lock();
    if let Some(tz) = driver.as_mut() {
        
        tz.poll_tx();
        
        
        while let Some(be) = tz.poll_rx() {
            BGZ_.lock().push_back(be);
        }
    }
}


pub fn iyr() -> Option<Vec<u8>> {
    poll(); 
    BGZ_.lock().pop_front()
}


pub fn get_stats() -> (u64, u64, u64, u64) {
    (
        BEL_.load(Ordering::Relaxed),
        BEK_.load(Ordering::Relaxed),
        APP_.load(Ordering::Relaxed),
        APO_.load(Ordering::Relaxed),
    )
}



pub fn btc() {
    let iobase = ALJ_.load(Ordering::Relaxed);
    if iobase == 0 { return; }
    
    
    let isr: u8 = unsafe {
        let mut port = Port::<u8>::new(iobase + 0x13);
        port.read()
    };
    
    if isr & 1 != 0 {
        
        ALK_.store(true, Ordering::Release);
    }
}


pub fn irq_pending() -> bool {
    ALK_.load(Ordering::Relaxed)
}



pub fn opc(iobase: u16) {
    ALJ_.store(iobase, Ordering::SeqCst);
}




pub fn mhx() {
    btc();
}
