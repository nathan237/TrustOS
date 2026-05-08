




use alloc::boxed::Box;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use spin::Mutex;

use crate::virtio::{VirtioDevice, Virtqueue, desc_flags, status};
use crate::pci::L;


pub const AT_: usize = 512;


pub mod features {
    pub const EKE_: u32 = 1 << 1;      
    pub const EJC_: u32 = 1 << 2;       
    pub const Awg: u32 = 1 << 4;      
    pub const Adf: u32 = 1 << 5;            
    pub const DHK_: u32 = 1 << 6;      
    pub const Ajv: u32 = 1 << 9;         
    pub const Bdw: u32 = 1 << 10;     
    pub const DKF_: u32 = 1 << 11;   
}


pub mod req_type {
    pub const Alj: u32 = 0;       
    pub const Amz: u32 = 1;      
    pub const Ajv: u32 = 4;    
    pub const Atw: u32 = 11; 
    pub const EOE_: u32 = 13;
}


pub mod blk_status {
    pub const Abx: u8 = 0;
    pub const Axm: u8 = 1;
    pub const Bei: u8 = 2;
}


#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct VirtioBlkReqHdr {
    pub req_type: u32,
    pub reserved: u32,
    pub dj: u64,
}

impl VirtioBlkReqHdr {
    pub const Z: usize = 16;
}


pub struct VirtioBlk {
    
    device: VirtioDevice,
    
    queue: Option<Box<Virtqueue>>,
    
    capacity: u64,
    
    sector_size: u32,
    
    read_only: bool,
}


static Cl: Mutex<Option<VirtioBlk>> = Mutex::new(None);
static Ah: AtomicBool = AtomicBool::new(false);


static BKR_: core::sync::atomic::AtomicU16 = core::sync::atomic::AtomicU16::new(0);

static RI_: AtomicBool = AtomicBool::new(false);


static Acy: AtomicU64 = AtomicU64::new(0);
static Agl: AtomicU64 = AtomicU64::new(0);
static KF_: AtomicU64 = AtomicU64::new(0);
static KG_: AtomicU64 = AtomicU64::new(0);

impl VirtioBlk {
    
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
        
        crate::log_debug!("[virtio-blk] I/O base: {:#X}", iobase);
        
        let mut device = VirtioDevice::new(iobase);
        
        
        device.reset();
        
        
        device.add_status(status::Gf);
        device.add_status(status::Cl);
        
        
        let device_features = device.read_device_features();
        crate::log_debug!("[virtio-blk] Device features: {:#X}", device_features);
        
        
        let read_only = (device_features & features::Adf) != 0;
        
        
        let driver_features = 0u32; 
        device.write_driver_features(driver_features);
        
        
        
        let fkt = device.read_config32(0) as u64;
        let fks = device.read_config32(4) as u64;
        let capacity = fkt | (fks << 32);
        
        crate::log!("[virtio-blk] Capacity: {} sectors ({} MB)", 
            capacity, (capacity * 512) / (1024 * 1024));
        
        if read_only {
            crate::log!("[virtio-blk] Device is read-only");
        }
        
        Ok(Self {
            device,
            queue: None,
            capacity,
            sector_size: 512,
            read_only,
        })
    }
    
    
    pub fn setup_queue(&mut self) -> Result<(), &'static str> {
        self.device.select_queue(0);
        let size = self.device.get_queue_size();
        crate::log_debug!("[virtio-blk] Queue size: {}", size);
        
        if size == 0 {
            return Err("Queue not available");
        }
        
        let queue = self.alloc_queue(size)?;
        
        
        let bog = (queue.phys_addr / 4096) as u32;
        self.device.set_queue_address(bog);
        
        self.queue = Some(queue);
        Ok(())
    }
    
    
    fn alloc_queue(&self, size: u16) -> Result<Box<Virtqueue>, &'static str> {
        Virtqueue::new(size)
    }
    
    
    pub fn start(&mut self) -> Result<(), &'static str> {
        self.device.add_status(status::IQ_);
        crate::log!("[virtio-blk] Device started");
        Ok(())
    }
    
    
    pub fn read_sectors(&mut self, start_sector: u64, count: usize, buffer: &mut [u8]) -> Result<(), &'static str> {
        if buffer.len() < count * AT_ {
            return Err("Buffer too small");
        }
        
        if start_sector + count as u64 > self.capacity {
            return Err("Read beyond device capacity");
        }
        
        
        for i in 0..count {
            self.read_one_sector(start_sector + i as u64, 
                &mut buffer[i * AT_..(i + 1) * AT_])?;
        }
        
        Acy.fetch_add(count as u64, Ordering::Relaxed);
        KF_.fetch_add((count * AT_) as u64, Ordering::Relaxed);
        
        Ok(())
    }
    
    
    fn read_one_sector(&mut self, dj: u64, buffer: &mut [u8]) -> Result<(), &'static str> {
        
        use alloc::boxed::Box;
        use alloc::vec;
        
        
        
        let total_size = VirtioBlkReqHdr::Z + AT_ + 1;
        let mut dma_buf = vec![0u8; total_size].into_boxed_slice();
        
        
        let header = VirtioBlkReqHdr {
            req_type: req_type::Alj,
            reserved: 0,
            dj,
        };
        unsafe {
            let gak = dma_buf.as_mut_ptr() as *mut VirtioBlkReqHdr;
            core::ptr::write(gak, header);
        }
        
        
        dma_buf[VirtioBlkReqHdr::Z + AT_] = 0xFF;
        
        
        let bz = crate::memory::hhdm_offset();
        let fso = dma_buf.as_ptr() as u64;
        let ali = fso - bz;
        
        let gaj = ali;
        let data_phys = ali + VirtioBlkReqHdr::Z as u64;
        let status_phys = ali + (VirtioBlkReqHdr::Z + AT_) as u64;
        
        let queue = self.queue.as_mut().ok_or("Queue not initialized")?;
        
        
        let su = queue.alloc_desc().ok_or("No free descriptor")?;
        let bro = queue.alloc_desc().ok_or("No free descriptor")?;
        let ave = queue.alloc_desc().ok_or("No free descriptor")?;
        
        
        queue.set_desc(su, gaj, VirtioBlkReqHdr::Z as u32, 
            desc_flags::Pn, bro);
        
        
        queue.set_desc(bro, data_phys, AT_ as u32,
            desc_flags::Bh | desc_flags::Pn, ave);
        
        
        queue.set_desc(ave, status_phys, 1, desc_flags::Bh, 0);
        
        
        queue.submit(su);
        
        
        let iobase = self.device.iobase;
        
        
        unsafe {
            let mut port = crate::arch::Port::<u16>::new(iobase + 0x10);
            port.write(0);
        }
        
        
        RI_.store(false, Ordering::Release);
        
        
        let queue = self.queue.as_mut().ok_or("Queue not initialized")?;
        
        
        let mut mz = 1_000_000u32;
        while !queue.has_used() && mz > 0 {
            if RI_.load(Ordering::Acquire) {
                break;
            }
            core::hint::spin_loop();
            mz -= 1;
        }
        
        if mz == 0 {
            queue.free_desc(su);
            queue.free_desc(bro);
            queue.free_desc(ave);
            return Err("Request timeout");
        }
        
        
        let jsu = queue.pop_used().ok_or("No completion")?;
        
        
        queue.free_desc(su);
        queue.free_desc(bro);
        queue.free_desc(ave);
        
        if dma_buf[VirtioBlkReqHdr::Z + AT_] != blk_status::Abx {
            return Err("Device error");
        }
        
        
        buffer.copy_from_slice(&dma_buf[VirtioBlkReqHdr::Z..VirtioBlkReqHdr::Z + AT_]);
        
        Ok(())
    }
    
    
    pub fn write_sectors(&mut self, start_sector: u64, count: usize, buffer: &[u8]) -> Result<(), &'static str> {
        if self.read_only {
            return Err("Device is read-only");
        }
        
        if buffer.len() < count * AT_ {
            return Err("Buffer too small");
        }
        
        if start_sector + count as u64 > self.capacity {
            return Err("Write beyond device capacity");
        }
        
        
        for i in 0..count {
            self.write_one_sector(start_sector + i as u64,
                &buffer[i * AT_..(i + 1) * AT_])?;
        }
        
        Agl.fetch_add(count as u64, Ordering::Relaxed);
        KG_.fetch_add((count * AT_) as u64, Ordering::Relaxed);
        
        Ok(())
    }
    
    
    fn write_one_sector(&mut self, dj: u64, buffer: &[u8]) -> Result<(), &'static str> {
        
        use alloc::vec;
        
        
        
        let total_size = VirtioBlkReqHdr::Z + AT_ + 1;
        let mut dma_buf = vec![0u8; total_size].into_boxed_slice();
        
        
        let header = VirtioBlkReqHdr {
            req_type: req_type::Amz,
            reserved: 0,
            dj,
        };
        unsafe {
            let gak = dma_buf.as_mut_ptr() as *mut VirtioBlkReqHdr;
            core::ptr::write(gak, header);
        }
        
        
        dma_buf[VirtioBlkReqHdr::Z..VirtioBlkReqHdr::Z + AT_]
            .copy_from_slice(&buffer[..AT_]);
        
        
        dma_buf[VirtioBlkReqHdr::Z + AT_] = 0xFF;
        
        
        let bz = crate::memory::hhdm_offset();
        let fso = dma_buf.as_ptr() as u64;
        let ali = fso - bz;
        
        let gaj = ali;
        let data_phys = ali + VirtioBlkReqHdr::Z as u64;
        let status_phys = ali + (VirtioBlkReqHdr::Z + AT_) as u64;
        
        let queue = self.queue.as_mut().ok_or("Queue not initialized")?;
        
        let su = queue.alloc_desc().ok_or("No free descriptor")?;
        let bro = queue.alloc_desc().ok_or("No free descriptor")?;
        let ave = queue.alloc_desc().ok_or("No free descriptor")?;
        
        
        queue.set_desc(su, gaj, VirtioBlkReqHdr::Z as u32,
            desc_flags::Pn, bro);
        
        
        queue.set_desc(bro, data_phys, AT_ as u32,
            desc_flags::Pn, ave);
        
        
        queue.set_desc(ave, status_phys, 1, desc_flags::Bh, 0);
        
        queue.submit(su);
        
        
        let iobase = self.device.iobase;
        
        
        unsafe {
            let mut port = crate::arch::Port::<u16>::new(iobase + 0x10);
            port.write(0);
        }
        
        
        RI_.store(false, Ordering::Release);
        
        
        let queue = self.queue.as_mut().ok_or("Queue not initialized")?;
        
        let mut mz = 1_000_000u32;
        while !queue.has_used() && mz > 0 {
            if RI_.load(Ordering::Acquire) {
                break;
            }
            core::hint::spin_loop();
            mz -= 1;
        }
        
        if mz == 0 {
            queue.free_desc(su);
            queue.free_desc(bro);
            queue.free_desc(ave);
            return Err("Request timeout");
        }
        
        let jsu = queue.pop_used().ok_or("No completion")?;
        
        queue.free_desc(su);
        queue.free_desc(bro);
        queue.free_desc(ave);
        
        if dma_buf[VirtioBlkReqHdr::Z + AT_] != blk_status::Abx {
            return Err("Device error");
        }
        
        Ok(())
    }
    
    
    pub fn capacity(&self) -> u64 {
        self.capacity
    }
    
    
    pub fn is_read_only(&self) -> bool {
        self.read_only
    }
}




pub fn init(go: &L) -> Result<(), &'static str> {
    crate::log!("[virtio-blk] Initializing...");
    
    let mut driver = VirtioBlk::new(go)?;
    driver.setup_queue()?;
    driver.start()?;
    
    
    BKR_.store(driver.device.iobase, Ordering::SeqCst);
    
    Ah.store(true, Ordering::SeqCst);
    *Cl.lock() = Some(driver);
    
    
    let irq = go.interrupt_line;
    if irq > 0 && irq < 255 {
        crate::apic::eyz(irq, crate::apic::HZ_);
        crate::serial_println!("[virtio-blk] IRQ {} routed to vector {}", irq, crate::apic::HZ_);
    }
    
    Ok(())
}


pub fn is_initialized() -> bool {
    Ah.load(Ordering::Relaxed)
}


pub fn capacity() -> u64 {
    Cl.lock().as_ref().map(|d| d.capacity()).unwrap_or(0)
}


pub fn is_read_only() -> bool {
    Cl.lock().as_ref().map(|d| d.is_read_only()).unwrap_or(true)
}


pub fn read_sectors(start: u64, count: usize, buffer: &mut [u8]) -> Result<(), &'static str> {
    let mut driver = Cl.lock();
    let tz = driver.as_mut().ok_or("Driver not initialized")?;
    tz.read_sectors(start, count, buffer)
}


pub fn write_sectors(start: u64, count: usize, buffer: &[u8]) -> Result<(), &'static str> {
    let mut driver = Cl.lock();
    let tz = driver.as_mut().ok_or("Driver not initialized")?;
    tz.write_sectors(start, count, buffer)
}


pub fn read_sector(dj: u64, buffer: &mut [u8; 512]) -> Result<(), &'static str> {
    read_sectors(dj, 1, buffer)
}


pub fn write_sector(dj: u64, buffer: &[u8; 512]) -> Result<(), &'static str> {
    write_sectors(dj, 1, buffer)
}


pub fn get_stats() -> (u64, u64, u64, u64) {
    (
        Acy.load(Ordering::Relaxed),
        Agl.load(Ordering::Relaxed),
        KF_.load(Ordering::Relaxed),
        KG_.load(Ordering::Relaxed),
    )
}



pub fn btc() {
    let iobase = BKR_.load(Ordering::Relaxed);
    if iobase == 0 { return; }
    
    
    let isr: u8 = unsafe {
        let mut port = crate::arch::Port::<u8>::new(iobase + 0x13);
        port.read()
    };
    
    if isr & 1 != 0 {
        
        RI_.store(true, Ordering::Release);
    }
}
