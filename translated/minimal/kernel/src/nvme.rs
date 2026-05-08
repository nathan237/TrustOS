











use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex;





const BGB_: u64 = 0x00;     
const CRX_: u64 = 0x08;      
const CRD_: u64 = 0x0C;   
const EFV_: u64 = 0x10;   
const AIV_: u64 = 0x14;      
const BGC_: u64 = 0x1C;    
const CQY_: u64 = 0x24;     
const BGA_: u64 = 0x28;     
const BFZ_: u64 = 0x30;     


const ASR_: u64 = 0x1000;


const ABI_: u32 = 1 << 0;         
const BOR_: u32 = 0 << 4;    
const BOV_: u32 = 0 << 7;     
const BOQ_: u32 = 0 << 11;    
const BOU_: u32 = 6 << 16;    
const BOT_: u32 = 4 << 20;    


const ARO_: u32 = 1 << 0;      
const BSZ_: u32 = 1 << 1;      






const DGB_: u8 = 0x00;
const BLO_: u8 = 0x01;
const DGA_: u8 = 0x04;
const BLN_: u8 = 0x05;
const AAB_: u8 = 0x06;
const DGC_: u8 = 0x09;


const CFS_: u8 = 0x00;
const CFV_: u8 = 0x01;
const CFU_: u8 = 0x02;


const CEU_: u32 = 0x00;
const CET_: u32 = 0x01;
const CES_: u32 = 0x02;






#[derive(Clone, Copy, Default)]
#[repr(C)]
struct Di {
    
    cdw0: u32,
    
    nsid: u32,
    
    cdw2: u32,
    cdw3: u32,
    
    mptr: u64,
    
    prp1: u64,
    
    prp2: u64,
    
    cdw10: u32,
    cdw11: u32,
    cdw12: u32,
    cdw13: u32,
    cdw14: u32,
    cdw15: u32,
}

const _: () = assert!(core::mem::size_of::<Di>() == 64);


#[derive(Clone, Copy, Default)]
#[repr(C)]
struct Hl {
    
    lmt: u32,
    
    bza: u32,
    
    sq_head_sqid: u32,
    
    cid_status: u32,
}

const _: () = assert!(core::mem::size_of::<Hl>() == 16);

impl Hl {
    fn phase(&self) -> bool {
        self.cid_status & (1 << 16) != 0
    }

    fn status_code(&self) -> u16 {
        ((self.cid_status >> 17) & 0x7FF) as u16
    }

    fn qav(&self) -> u16 {
        (self.cid_status & 0xFFFF) as u16
    }
}


struct QueuePair {
    
    sq_virt: u64,
    
    sq_phys: u64,
    
    cq_virt: u64,
    
    cq_phys: u64,
    
    depth: u16,
    
    sq_tail: u16,
    
    cq_head: u16,
    
    cq_phase: bool,
    
    next_cid: u16,
    
    qid: u16,
}

impl QueuePair {
    
    fn new(qid: u16, depth: u16) -> Option<Self> {
        
        let sq_phys = crate::memory::frame::aan()?;
        let sq_virt = crate::memory::wk(sq_phys);
        
        
        let cq_phys = crate::memory::frame::aan()?;
        let cq_virt = crate::memory::wk(cq_phys);
        
        Some(Self {
            sq_virt,
            sq_phys,
            cq_virt,
            cq_phys,
            depth,
            sq_tail: 0,
            cq_head: 0,
            cq_phase: true,     
            next_cid: 0,
            qid,
        })
    }
    
    
    fn submit(&mut self, mut cmd: Di) -> u16 {
        let hkz = self.next_cid;
        self.next_cid = self.next_cid.wrapping_add(1);
        
        
        cmd.cdw0 = (cmd.cdw0 & 0x0000FFFF) | ((hkz as u32) << 16);
        
        
        let offset = self.sq_tail as usize * core::mem::size_of::<Di>();
        unsafe {
            let ptr = (self.sq_virt + offset as u64) as *mut Di;
            core::ptr::write_volatile(ptr, cmd);
        }
        
        
        self.sq_tail = (self.sq_tail + 1) % self.depth;
        
        hkz
    }
    
    
    fn poll_completion(&mut self) -> Option<Hl> {
        let offset = self.cq_head as usize * core::mem::size_of::<Hl>();
        let entry = unsafe {
            let ptr = (self.cq_virt + offset as u64) as *const Hl;
            core::ptr::read_volatile(ptr)
        };
        
        
        if entry.phase() == self.cq_phase {
            
            self.cq_head += 1;
            if self.cq_head >= self.depth {
                self.cq_head = 0;
                self.cq_phase = !self.cq_phase;
            }
            Some(entry)
        } else {
            None
        }
    }
}


#[derive(Clone)]
pub struct Ud {
    
    pub nsid: u32,
    
    pub size_lbas: u64,
    
    pub lba_size: u32,
}


struct Uc {
    
    bar_virt: u64,
    
    doorbell_stride: u32,
    
    admin: QueuePair,
    
    io: Option<QueuePair>,
    
    serial: String,
    
    model: String,
    
    ns1_size: u64,
    
    ns1_lba_size: u32,
    
    namespaces: Vec<Ud>,
    
    max_transfer_pages: u32,
}

impl Uc {
    
    
    #[inline]
    fn read32(&self, offset: u64) -> u32 {
        unsafe { core::ptr::read_volatile((self.bar_virt + offset) as *const u32) }
    }
    
    #[inline]
    fn write32(&self, offset: u64, value: u32) {
        unsafe { core::ptr::write_volatile((self.bar_virt + offset) as *mut u32, value) }
    }
    
    #[inline]
    fn qru(&self, offset: u64) -> u64 {
        let lo = self.read32(offset) as u64;
        let hi = self.read32(offset + 4) as u64;
        lo | (hi << 32)
    }
    
    #[inline]
    fn write64(&self, offset: u64, value: u64) {
        self.write32(offset, value as u32);
        self.write32(offset + 4, (value >> 32) as u32);
    }
    
    
    
    
    fn ring_sq_doorbell(&self, qid: u16, new_tail: u16) {
        let offset = ASR_ + (2 * qid as u64) * self.doorbell_stride as u64;
        self.write32(offset, new_tail as u32);
    }
    
    
    fn ring_cq_doorbell(&self, qid: u16, bum: u16) {
        let offset = ASR_ + (2 * qid as u64 + 1) * self.doorbell_stride as u64;
        self.write32(offset, bum as u32);
    }
    
    
    
    
    fn admin_cmd(&mut self, cmd: Di) -> Result<Hl, &'static str> {
        let jsl = self.admin.submit(cmd);
        self.ring_sq_doorbell(0, self.admin.sq_tail);
        
        
        for _ in 0..1_000_000u32 {
            if let Some(cqe) = self.admin.poll_completion() {
                
                self.ring_cq_doorbell(0, self.admin.cq_head);
                
                if cqe.status_code() != 0 {
                    crate::serial_println!("[NVMe] Admin cmd failed: status={:#x}", cqe.status_code());
                    return Err("NVMe admin command failed");
                }
                return Ok(cqe);
            }
            core::hint::spin_loop();
        }
        Err("NVMe admin command timeout")
    }
    
    
    fn io_cmd(&mut self, cmd: Di) -> Result<Hl, &'static str> {
        
        let sq_tail = {
            let io = self.io.as_mut().ok_or("NVMe I/O queue not initialized")?;
            let jsl = io.submit(cmd);
            io.sq_tail
        };
        self.ring_sq_doorbell(1, sq_tail);
        
        
        for _ in 0..10_000_000u32 {
            let io = self.io.as_mut().unwrap();
            if let Some(cqe) = io.poll_completion() {
                let cq_head = io.cq_head;
                self.ring_cq_doorbell(1, cq_head);
                
                if cqe.status_code() != 0 {
                    crate::serial_println!("[NVMe] I/O cmd failed: status={:#x}", cqe.status_code());
                    return Err("NVMe I/O command failed");
                }
                return Ok(cqe);
            }
            core::hint::spin_loop();
        }
        Err("NVMe I/O command timeout")
    }
    
    
    
    
    fn identify_controller(&mut self) -> Result<(), &'static str> {
        let hg = crate::memory::frame::aan()
            .ok_or("NVMe: OOM for identify buffer")?;
        let kt = crate::memory::wk(hg);
        
        let cmd = Di {
            cdw0: AAB_ as u32,
            prp1: hg,
            cdw10: CET_,
            ..Default::default()
        };
        
        self.admin_cmd(cmd)?;
        
        
        unsafe {
            let data = kt as *const u8;
            
            
            let mut jgu = [0u8; 20];
            core::ptr::copy_nonoverlapping(data.add(4), jgu.as_mut_ptr(), 20);
            self.serial = core::str::from_utf8(&jgu)
                .unwrap_or("?")
                .trim()
                .into();
            
            
            let mut akc = [0u8; 40];
            core::ptr::copy_nonoverlapping(data.add(24), akc.as_mut_ptr(), 40);
            self.model = core::str::from_utf8(&akc)
                .unwrap_or("?")
                .trim()
                .into();
            
            
            
            let ina = *data.add(77);
            self.max_transfer_pages = if ina == 0 { 256 } else { 1u32 << ina };
        }
        
        crate::memory::frame::vk(hg);
        Ok(())
    }
    
    
    fn identify_namespace_by_id(&mut self, nsid: u32) -> Result<(u64, u32), &'static str> {
        let hg = crate::memory::frame::aan()
            .ok_or("NVMe: OOM for identify namespace buffer")?;
        let kt = crate::memory::wk(hg);
        
        let cmd = Di {
            cdw0: AAB_ as u32,
            nsid,
            prp1: hg,
            cdw10: CEU_,
            ..Default::default()
        };
        
        self.admin_cmd(cmd)?;
        
        let (bii, lba_size) = unsafe {
            let data = kt as *const u8;
            
            
            let bii = core::ptr::read_unaligned(data as *const u64);
            
            
            let lww = *data.add(26) & 0x0F;
            
            
            
            let mxq = 128 + (lww as usize) * 4;
            let mxp = core::ptr::read_unaligned(data.add(mxq) as *const u32);
            let mxo = (mxp >> 16) & 0xFF;
            let aol = 1u32 << mxo;
            
            (bii, aol)
        };
        
        crate::memory::frame::vk(hg);
        Ok((bii, lba_size))
    }
    
    
    fn qkx(&mut self) -> Result<(), &'static str> {
        let (bii, lba_size) = self.identify_namespace_by_id(1)?;
        self.ns1_size = bii;
        self.ns1_lba_size = lba_size;
        Ok(())
    }
    
    
    
    fn enumerate_namespaces(&mut self) -> Result<(), &'static str> {
        let hg = crate::memory::frame::aan()
            .ok_or("NVMe: OOM for NS list buffer")?;
        let kt = crate::memory::wk(hg);
        
        let cmd = Di {
            cdw0: AAB_ as u32,
            nsid: 0, 
            prp1: hg,
            cdw10: CES_,
            ..Default::default()
        };
        
        let gjx: Vec<u32>;
        
        match self.admin_cmd(cmd) {
            Ok(_) => {
                
                
                let mut ids = Vec::new();
                unsafe {
                    let list = kt as *const u32;
                    for i in 0..1024 {
                        let nsid = core::ptr::read_volatile(list.add(i));
                        if nsid == 0 { break; }
                        ids.push(nsid);
                    }
                }
                gjx = ids;
            }
            Err(_) => {
                
                crate::serial_println!("[NVMe] Active NSID list not supported, using NS1 only");
                gjx = alloc::vec![1];
            }
        }
        
        crate::memory::frame::vk(hg);
        
        self.namespaces.clear();
        
        for &nsid in &gjx {
            match self.identify_namespace_by_id(nsid) {
                Ok((bii, lba_size)) => {
                    if bii > 0 {
                        let size_mb = (bii * lba_size as u64) / (1024 * 1024);
                        crate::serial_println!("[NVMe] NS{}: {} LBAs x {} B = {} MB",
                            nsid, bii, lba_size, size_mb);
                        self.namespaces.push(Ud {
                            nsid,
                            size_lbas: bii,
                            lba_size,
                        });
                        
                        if nsid == 1 {
                            self.ns1_size = bii;
                            self.ns1_lba_size = lba_size;
                        }
                    }
                }
                Err(e) => {
                    crate::serial_println!("[NVMe] Failed to identify NS{}: {}", nsid, e);
                }
            }
        }
        
        if self.namespaces.is_empty() {
            return Err("NVMe: no usable namespaces found");
        }
        
        Ok(())
    }
    
    
    
    
    fn create_io_cq(&mut self, qid: u16, cq_phys: u64, depth: u16) -> Result<(), &'static str> {
        let cmd = Di {
            cdw0: BLN_ as u32,
            prp1: cq_phys,
            
            cdw10: (qid as u32) | (((depth - 1) as u32) << 16),
            
            cdw11: 1,   
            ..Default::default()
        };
        
        self.admin_cmd(cmd)?;
        Ok(())
    }
    
    
    fn create_io_sq(&mut self, qid: u16, sq_phys: u64, cqid: u16, depth: u16) -> Result<(), &'static str> {
        let cmd = Di {
            cdw0: BLO_ as u32,
            prp1: sq_phys,
            
            cdw10: (qid as u32) | (((depth - 1) as u32) << 16),
            
            cdw11: 1 | ((cqid as u32) << 16),
            ..Default::default()
        };
        
        self.admin_cmd(cmd)?;
        Ok(())
    }
    
    
    
    
    
    
    fn build_prp2_scatter(&self, acg: &[u64]) -> Result<(u64, Option<u64>), &'static str> {
        if acg.len() <= 1 {
            
            Ok((0, None))
        } else if acg.len() == 2 {
            
            Ok((acg[1], None))
        } else {
            
            let etc = crate::memory::frame::aan()
                .ok_or("NVMe: OOM for PRP list")?;
            let mzj = crate::memory::wk(etc);
            
            let ck = acg.len() - 1; 
            if ck > 512 {
                crate::memory::frame::vk(etc);
                return Err("NVMe: transfer too large for single PRP list");
            }
            
            unsafe {
                let entries = mzj as *mut u64;
                for i in 0..ck {
                    core::ptr::write_volatile(entries.add(i), acg[i + 1]);
                }
            }
            
            Ok((etc, Some(etc)))
        }
    }
    
    
    fn read_lbas_scatter(&mut self, start_lba: u64, count: u16, acg: &[u64]) -> Result<(), &'static str> {
        let (prp2, prp_list_page) = self.build_prp2_scatter(acg)?;
        
        let cmd = Di {
            cdw0: CFU_ as u32,
            nsid: 1,
            prp1: acg[0],
            prp2,
            cdw10: start_lba as u32,
            cdw11: (start_lba >> 32) as u32,
            cdw12: (count - 1) as u32,
            ..Default::default()
        };
        
        let result = self.io_cmd(cmd);
        if let Some(phys) = prp_list_page {
            crate::memory::frame::vk(phys);
        }
        result?;
        Ok(())
    }
    
    
    fn write_lbas_scatter(&mut self, start_lba: u64, count: u16, acg: &[u64]) -> Result<(), &'static str> {
        let (prp2, prp_list_page) = self.build_prp2_scatter(acg)?;
        
        let cmd = Di {
            cdw0: CFV_ as u32,
            nsid: 1,
            prp1: acg[0],
            prp2,
            cdw10: start_lba as u32,
            cdw11: (start_lba >> 32) as u32,
            cdw12: (count - 1) as u32,
            ..Default::default()
        };
        
        let result = self.io_cmd(cmd);
        if let Some(phys) = prp_list_page {
            crate::memory::frame::vk(phys);
        }
        result?;
        Ok(())
    }
    
    
    fn flush(&mut self) -> Result<(), &'static str> {
        let cmd = Di {
            cdw0: CFS_ as u32,
            nsid: 1,
            ..Default::default()
        };
        self.io_cmd(cmd)?;
        Ok(())
    }
}





static Gs: Mutex<Option<Uc>> = Mutex::new(None);
static Ah: AtomicBool = AtomicBool::new(false);


pub fn is_initialized() -> bool {
    Ah.load(Ordering::Relaxed)
}


pub fn capacity() -> u64 {
    Gs.lock().as_ref().map(|c| c.ns1_size).unwrap_or(0)
}


pub fn lba_size() -> u32 {
    Gs.lock().as_ref().map(|c| c.ns1_lba_size).unwrap_or(512)
}


pub fn rk() -> Option<(String, String, u64, u32)> {
    let ctrl = Gs.lock();
    let c = ctrl.as_ref()?;
    Some((c.model.clone(), c.serial.clone(), c.ns1_size, c.ns1_lba_size))
}


pub fn mze() -> Vec<Ud> {
    Gs.lock().as_ref().map(|c| c.namespaces.clone()).unwrap_or_default()
}















pub fn init(go: &crate::pci::L) -> Result<(), &'static str> {
    crate::serial_println!("[NVMe] Initializing {:02X}:{:02X}.{} ({:04X}:{:04X})",
        go.bus, go.device, go.function,
        go.vendor_id, go.device_id);
    
    
    crate::pci::bzi(go);
    crate::pci::bzj(go);
    
    
    let cmd = crate::pci::vf(go.bus, go.device, go.function, 0x04);
    crate::pci::qj(go.bus, go.device, go.function, 0x04,
        (cmd | (1 << 10)) as u32); 
    
    
    let cgc = go.bar_address(0).ok_or("NVMe: no BAR0")?;
    if cgc == 0 {
        return Err("NVMe: BAR0 is zero");
    }
    
    
    let bar_virt = crate::memory::yv(cgc, 0x10000)?;
    
    crate::serial_println!("[NVMe] BAR0: phys={:#x}, virt={:#x}", cgc, bar_virt);
    
    
    let cap = {
        let lo = unsafe { core::ptr::read_volatile((bar_virt + BGB_) as *const u32) } as u64;
        let hi = unsafe { core::ptr::read_volatile((bar_virt + BGB_ + 4) as *const u32) } as u64;
        lo | (hi << 32)
    };
    
    let euo = (cap & 0xFFFF) as u16 + 1;  
    let ekv = ((cap >> 32) & 0xF) as u32; 
    let doorbell_stride = 4u32 << ekv;
    let eun = ((cap >> 48) & 0xF) as u32; 
    let pjo = ((cap >> 24) & 0xFF) as u32; 
    
    let vs = unsafe { core::ptr::read_volatile((bar_virt + CRX_) as *const u32) };
    let axz = (vs >> 16) & 0xFFFF;
    let ayh = (vs >> 8) & 0xFF;
    
    crate::serial_println!("[NVMe] Version: {}.{}, MQES={}, DSTRD={}, MPS_MIN={}KB, Timeout={}ms",
        axz, ayh, euo, ekv, 4 << eun, pjo * 500);
    
    
    let dxc = euo.min(64) as u16;
    
    
    let ft = unsafe { core::ptr::read_volatile((bar_virt + AIV_) as *const u32) };
    if ft & ABI_ != 0 {
        
        unsafe { core::ptr::write_volatile((bar_virt + AIV_) as *mut u32, ft & !ABI_) };
        
        
        for _ in 0..1_000_000u32 {
            let chv = unsafe { core::ptr::read_volatile((bar_virt + BGC_) as *const u32) };
            if chv & ARO_ == 0 {
                break;
            }
            core::hint::spin_loop();
        }
    }
    
    
    let admin = QueuePair::new(0, dxc)
        .ok_or("NVMe: OOM for admin queues")?;
    
    crate::serial_println!("[NVMe] Admin SQ phys={:#x}, CQ phys={:#x}, depth={}",
        admin.sq_phys, admin.cq_phys, dxc);
    
    
    
    let jxh = ((dxc - 1) as u32) | (((dxc - 1) as u32) << 16);
    unsafe {
        core::ptr::write_volatile((bar_virt + CQY_) as *mut u32, jxh);
        
        core::ptr::write_volatile((bar_virt + BGA_) as *mut u32, admin.sq_phys as u32);
        core::ptr::write_volatile((bar_virt + BGA_ + 4) as *mut u32, (admin.sq_phys >> 32) as u32);
        
        core::ptr::write_volatile((bar_virt + BFZ_) as *mut u32, admin.cq_phys as u32);
        core::ptr::write_volatile((bar_virt + BFZ_ + 4) as *mut u32, (admin.cq_phys >> 32) as u32);
    }
    
    
    unsafe {
        core::ptr::write_volatile((bar_virt + CRD_) as *mut u32, 0xFFFFFFFF);
    }
    
    
    let khu = ABI_ | BOR_ | BOV_ | BOQ_ | BOU_ | BOT_;
    unsafe {
        core::ptr::write_volatile((bar_virt + AIV_) as *mut u32, khu);
    }
    
    
    let mut ready = false;
    for _ in 0..5_000_000u32 {
        let chv = unsafe { core::ptr::read_volatile((bar_virt + BGC_) as *const u32) };
        if chv & BSZ_ != 0 {
            return Err("NVMe: Controller Fatal Status during enable");
        }
        if chv & ARO_ != 0 {
            ready = true;
            break;
        }
        core::hint::spin_loop();
    }
    
    if !ready {
        return Err("NVMe: Controller did not become ready");
    }
    
    crate::serial_println!("[NVMe] Controller enabled and ready");
    
    
    let mut ctrl = Uc {
        bar_virt,
        doorbell_stride,
        admin,
        io: None,
        serial: String::new(),
        model: String::new(),
        ns1_size: 0,
        ns1_lba_size: 512,
        namespaces: Vec::new(),
        max_transfer_pages: 256,
    };
    
    
    ctrl.identify_controller()?;
    crate::serial_println!("[NVMe] Model: '{}', Serial: '{}'", ctrl.model, ctrl.serial);
    
    
    ctrl.enumerate_namespaces()?;
    let total_mb: u64 = ctrl.namespaces.iter()
        .map(|ayq| (ayq.size_lbas * ayq.lba_size as u64) / (1024 * 1024))
        .sum();
    crate::serial_println!("[NVMe] {} namespace(s), total {} MB", ctrl.namespaces.len(), total_mb);
    
    
    let era = dxc;
    let gdh = QueuePair::new(1, era)
        .ok_or("NVMe: OOM for I/O queues")?;
    
    ctrl.create_io_cq(1, gdh.cq_phys, era)?;
    ctrl.create_io_sq(1, gdh.sq_phys, 1, era)?;
    ctrl.io = Some(gdh);
    
    crate::serial_println!("[NVMe] I/O queue pair created (depth={})", era);
    
    
    let nll = ctrl.namespaces.len();
    *Gs.lock() = Some(ctrl);
    Ah.store(true, Ordering::Release);
    
    crate::serial_println!("[NVMe] ✓ Driver initialized — {} namespace(s), {} MB NVMe storage available",
        nll, total_mb);
    
    Ok(())
}










pub fn read_sectors(start_lba: u64, count: usize, buffer: &mut [u8]) -> Result<(), &'static str> {
    let mut ctrl = Gs.lock();
    let ctrl = ctrl.as_mut().ok_or("NVMe: not initialized")?;
    
    let aol = ctrl.ns1_lba_size as usize;
    let total_bytes = count * aol;
    
    if buffer.len() < total_bytes {
        return Err("NVMe: buffer too small");
    }
    
    if start_lba + count as u64 > ctrl.ns1_size {
        return Err("NVMe: read past end of namespace");
    }
    
    
    
    let ggz = 128usize;
    let ggt = ggz * 4096;
    let ggx = (ggt / aol).max(1);
    
    let mut hb = start_lba;
    let mut offset = 0usize;
    let mut ck = count;
    
    while ck > 0 {
        let df = ck.min(ggx);
        let blb = df * aol;
        let boc = (blb + 4095) / 4096;
        
        
        let mut bgc: Vec<u64> = Vec::with_capacity(boc);
        for _ in 0..boc {
            match crate::memory::frame::aan() {
                Some(phys) => bgc.push(phys),
                None => {
                    
                    for aa in &bgc { crate::memory::frame::vk(*aa); }
                    return Err("NVMe: OOM for DMA read buffer");
                }
            }
        }
        
        
        ctrl.read_lbas_scatter(hb, df as u16, &bgc)?;
        
        
        let mut dke = blb;
        for (i, &bcy) in bgc.iter().enumerate() {
            let dll = dke.min(4096);
            let virt = crate::memory::wk(bcy);
            unsafe {
                core::ptr::copy_nonoverlapping(
                    virt as *const u8,
                    buffer[offset + i * 4096..].as_mut_ptr(),
                    dll,
                );
            }
            dke -= dll;
        }
        
        
        for aa in &bgc { crate::memory::frame::vk(*aa); }
        
        hb += df as u64;
        offset += blb;
        ck -= df;
    }
    
    Ok(())
}


pub fn write_sectors(start_lba: u64, count: usize, buffer: &[u8]) -> Result<(), &'static str> {
    let mut ctrl = Gs.lock();
    let ctrl = ctrl.as_mut().ok_or("NVMe: not initialized")?;
    
    let aol = ctrl.ns1_lba_size as usize;
    let total_bytes = count * aol;
    
    if buffer.len() < total_bytes {
        return Err("NVMe: buffer too small");
    }
    
    if start_lba + count as u64 > ctrl.ns1_size {
        return Err("NVMe: write past end of namespace");
    }
    
    let ggz = 128usize;
    let ggt = ggz * 4096;
    let ggx = (ggt / aol).max(1);
    
    let mut hb = start_lba;
    let mut offset = 0usize;
    let mut ck = count;
    
    while ck > 0 {
        let df = ck.min(ggx);
        let blb = df * aol;
        let boc = (blb + 4095) / 4096;
        
        
        let mut bgc: Vec<u64> = Vec::with_capacity(boc);
        for _ in 0..boc {
            match crate::memory::frame::aan() {
                Some(phys) => bgc.push(phys),
                None => {
                    for aa in &bgc { crate::memory::frame::vk(*aa); }
                    return Err("NVMe: OOM for DMA write buffer");
                }
            }
        }
        
        
        let mut dke = blb;
        for (i, &bcy) in bgc.iter().enumerate() {
            let dll = dke.min(4096);
            let virt = crate::memory::wk(bcy);
            unsafe {
                core::ptr::copy_nonoverlapping(
                    buffer[offset + i * 4096..].as_ptr(),
                    virt as *mut u8,
                    dll,
                );
            }
            dke -= dll;
        }
        
        
        ctrl.write_lbas_scatter(hb, df as u16, &bgc)?;
        
        
        for aa in &bgc { crate::memory::frame::vk(*aa); }
        
        hb += df as u64;
        offset += blb;
        ck -= df;
    }
    
    Ok(())
}


pub fn flush() -> Result<(), &'static str> {
    let mut ctrl = Gs.lock();
    let ctrl = ctrl.as_mut().ok_or("NVMe: not initialized")?;
    ctrl.flush()
}
