





















use core::sync::atomic::{AtomicU32, AtomicBool, Ordering};


const BBV_: usize = 16;


pub mod gic_regs {
    
    pub const DTW_: u32 = 0;
    
    pub const DTV_: u32 = 1;
    
    pub const DTX_: u32 = 2;
    
    pub const DTY_: u32 = 3;
    
    pub const DTZ_: u32 = 4;
    
    pub const DUA_: u32 = 5;
}


pub mod ich_hcr {
    
    pub const Aix: u64 = 1 << 0;
    
    pub const Bee: u64 = 1 << 1;
    
    pub const Ayp: u64 = 1 << 2;
    
    pub const Azq: u64 = 1 << 3;
    
    pub const Bes: u64 = 1 << 4;
    
    pub const Bet: u64 = 1 << 5;
    
    pub const Bdn: u64 = 1 << 6;
    
    pub const Bdo: u64 = 1 << 7;
    
    pub const DOU_: u64 = 27;
}


pub mod ich_lr {
    
    pub const EKM_: u64 = 0b00 << 62;   
    
    pub const CXV_: u64 = 0b01 << 60;
    
    pub const EKL_: u64 = 0b10 << 60;
    
    pub const EKN_: u64 = 0b11 << 60;
    
    pub const Aky: u64 = 1 << 63;
    
    pub const Akt: u64 = 1 << 60;

    
    pub fn kfd(bwt: u32, pintid: u32, priority: u8, xc: bool) -> u64 {
        let mut lr: u64 = 0;
        lr |= (bwt as u64) & 0xFFFF_FFFF;    
        if xc {
            lr |= Aky;
            lr |= ((pintid as u64) & 0x1FFF) << 32;  
        }
        lr |= ((priority as u64) & 0xFF) << 48;  
        lr |= CXV_;   
        lr |= Akt;          
        lr
    }
}


pub struct VirtualGic {
    
    num_lrs: u32,
    
    pending_queue: [u32; 64],
    pending_count: usize,
    
    initialized: bool,
}


static BKN_: AtomicBool = AtomicBool::new(false);

impl VirtualGic {
    
    pub const fn new() -> Self {
        VirtualGic {
            num_lrs: 0,
            pending_queue: [0; 64],
            pending_count: 0,
            initialized: false,
        }
    }

    
    
    
    pub fn init(&mut self) {
        #[cfg(target_arch = "aarch64")]
        {
            
            let vtr: u64;
            unsafe {
                core::arch::asm!(
                    "mrs {vtr}, ich_vtr_el2",
                    vtr = out(reg) vtr,
                    options(nomem, nostack)
                );
            }

            
            self.num_lrs = ((vtr & 0x1F) + 1) as u32;
            if self.num_lrs > BBV_ as u32 {
                self.num_lrs = BBV_ as u32;
            }

            
            unsafe {
                
                core::arch::asm!(
                    "mrs {tmp}, icc_sre_el2",
                    "orr {tmp}, {tmp}, #0x1",   
                    "orr {tmp}, {tmp}, #0x8",   
                    "msr icc_sre_el2, {tmp}",
                    "isb",
                    tmp = out(reg) _,
                    options(nomem, nostack)
                );

                
                let hcr_val = ich_hcr::Aix;
                core::arch::asm!(
                    "msr ich_hcr_el2, {val}",
                    "isb",
                    val = in(reg) hcr_val,
                    options(nomem, nostack)
                );
            }

            self.initialized = true;
            BKN_.store(true, Ordering::Release);
        }

        #[cfg(not(target_arch = "aarch64"))]
        {
            self.num_lrs = 4;
            self.initialized = true;
            BKN_.store(true, Ordering::Release);
        }
    }

    
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    
    pub fn qpu(&self) -> u32 {
        self.num_lrs
    }

    
    
    
    
    pub fn inject_irq(&mut self, bwt: u32, pintid: u32, priority: u8) -> bool {
        #[cfg(target_arch = "aarch64")]
        {
            
            for i in 0..self.num_lrs {
                let nba = self.read_lr(i);
                let state = (nba >> 60) & 0x3;
                if state == 0 {
                    
                    let lr = ich_lr::kfd(bwt, pintid, priority, true);
                    self.write_lr(i, lr);
                    return true;
                }
            }

            
            if self.pending_count < self.pending_queue.len() {
                self.pending_queue[self.pending_count] = bwt;
                self.pending_count += 1;
            }
            false
        }

        #[cfg(not(target_arch = "aarch64"))]
        {
            let _ = (bwt, pintid, priority);
            false
        }
    }

    
    pub fn qdj(&mut self) {
        if self.pending_count == 0 {
            return;
        }

        let mut ck = 0;
        for i in 0..self.pending_count {
            let bwt = self.pending_queue[i];
            if !self.inject_irq(bwt, bwt, 0xA0) {
                
                self.pending_queue[ck] = bwt;
                ck += 1;
            }
        }
        self.pending_count = ck;
    }

    
    fn read_lr(&self, idx: u32) -> u64 {
        #[cfg(target_arch = "aarch64")]
        unsafe {
            let val: u64;
            match idx {
                0 => core::arch::asm!("mrs {v}, ich_lr0_el2", v = out(reg) val, options(nomem, nostack)),
                1 => core::arch::asm!("mrs {v}, ich_lr1_el2", v = out(reg) val, options(nomem, nostack)),
                2 => core::arch::asm!("mrs {v}, ich_lr2_el2", v = out(reg) val, options(nomem, nostack)),
                3 => core::arch::asm!("mrs {v}, ich_lr3_el2", v = out(reg) val, options(nomem, nostack)),
                4 => core::arch::asm!("mrs {v}, ich_lr4_el2", v = out(reg) val, options(nomem, nostack)),
                5 => core::arch::asm!("mrs {v}, ich_lr5_el2", v = out(reg) val, options(nomem, nostack)),
                6 => core::arch::asm!("mrs {v}, ich_lr6_el2", v = out(reg) val, options(nomem, nostack)),
                7 => core::arch::asm!("mrs {v}, ich_lr7_el2", v = out(reg) val, options(nomem, nostack)),
                8 => core::arch::asm!("mrs {v}, ich_lr8_el2", v = out(reg) val, options(nomem, nostack)),
                9 => core::arch::asm!("mrs {v}, ich_lr9_el2", v = out(reg) val, options(nomem, nostack)),
                10 => core::arch::asm!("mrs {v}, ich_lr10_el2", v = out(reg) val, options(nomem, nostack)),
                11 => core::arch::asm!("mrs {v}, ich_lr11_el2", v = out(reg) val, options(nomem, nostack)),
                12 => core::arch::asm!("mrs {v}, ich_lr12_el2", v = out(reg) val, options(nomem, nostack)),
                13 => core::arch::asm!("mrs {v}, ich_lr13_el2", v = out(reg) val, options(nomem, nostack)),
                14 => core::arch::asm!("mrs {v}, ich_lr14_el2", v = out(reg) val, options(nomem, nostack)),
                15 => core::arch::asm!("mrs {v}, ich_lr15_el2", v = out(reg) val, options(nomem, nostack)),
                _ => return 0,
            }
            val
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            let _ = idx;
            0
        }
    }

    
    fn write_lr(&self, idx: u32, val: u64) {
        #[cfg(target_arch = "aarch64")]
        unsafe {
            match idx {
                0 => core::arch::asm!("msr ich_lr0_el2, {v}", v = in(reg) val, options(nomem, nostack)),
                1 => core::arch::asm!("msr ich_lr1_el2, {v}", v = in(reg) val, options(nomem, nostack)),
                2 => core::arch::asm!("msr ich_lr2_el2, {v}", v = in(reg) val, options(nomem, nostack)),
                3 => core::arch::asm!("msr ich_lr3_el2, {v}", v = in(reg) val, options(nomem, nostack)),
                4 => core::arch::asm!("msr ich_lr4_el2, {v}", v = in(reg) val, options(nomem, nostack)),
                5 => core::arch::asm!("msr ich_lr5_el2, {v}", v = in(reg) val, options(nomem, nostack)),
                6 => core::arch::asm!("msr ich_lr6_el2, {v}", v = in(reg) val, options(nomem, nostack)),
                7 => core::arch::asm!("msr ich_lr7_el2, {v}", v = in(reg) val, options(nomem, nostack)),
                8 => core::arch::asm!("msr ich_lr8_el2, {v}", v = in(reg) val, options(nomem, nostack)),
                9 => core::arch::asm!("msr ich_lr9_el2, {v}", v = in(reg) val, options(nomem, nostack)),
                10 => core::arch::asm!("msr ich_lr10_el2, {v}", v = in(reg) val, options(nomem, nostack)),
                11 => core::arch::asm!("msr ich_lr11_el2, {v}", v = in(reg) val, options(nomem, nostack)),
                12 => core::arch::asm!("msr ich_lr12_el2, {v}", v = in(reg) val, options(nomem, nostack)),
                13 => core::arch::asm!("msr ich_lr13_el2, {v}", v = in(reg) val, options(nomem, nostack)),
                14 => core::arch::asm!("msr ich_lr14_el2, {v}", v = in(reg) val, options(nomem, nostack)),
                15 => core::arch::asm!("msr ich_lr15_el2, {v}", v = in(reg) val, options(nomem, nostack)),
                _ => {}
            }
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            let _ = (idx, val);
        }
    }
}


pub fn mhr(vgic: &mut VirtualGic) {
    #[cfg(target_arch = "aarch64")]
    {
        
        let intid: u64;
        unsafe {
            core::arch::asm!(
                "mrs {id}, icc_iar1_el1",
                id = out(reg) intid,
                options(nomem, nostack)
            );
        }

        let intid = intid as u32;

        
        if intid >= 1020 {
            return;
        }

        
        vgic.inject_irq(intid, intid, 0xA0);

        
        unsafe {
            core::arch::asm!(
                "msr icc_eoir1_el1, {id}",
                id = in(reg) intid as u64,
                options(nomem, nostack)
            );
        }

        
        unsafe {
            core::arch::asm!(
                "msr icc_dir_el1, {id}",
                id = in(reg) intid as u64,
                options(nomem, nostack)
            );
        }
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        let _ = vgic;
    }
}
