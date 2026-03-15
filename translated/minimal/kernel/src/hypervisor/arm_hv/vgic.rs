





















use core::sync::atomic::{AtomicU32, AtomicBool, Ordering};


const AZT_: usize = 16;


pub mod gic_regs {
    
    pub const DQC_: u32 = 0;
    
    pub const DQB_: u32 = 1;
    
    pub const DQD_: u32 = 2;
    
    pub const DQE_: u32 = 3;
    
    pub const DQF_: u32 = 4;
    
    pub const DQG_: u32 = 5;
}


pub mod ich_hcr {
    
    pub const Cbf: u64 = 1 << 0;
    
    pub const Dka: u64 = 1 << 1;
    
    pub const Dbe: u64 = 1 << 2;
    
    pub const Dct: u64 = 1 << 3;
    
    pub const Dku: u64 = 1 << 4;
    
    pub const Dkv: u64 = 1 << 5;
    
    pub const Djd: u64 = 1 << 6;
    
    pub const Dje: u64 = 1 << 7;
    
    pub const DLF_: u64 = 27;
}


pub mod ich_lr {
    
    pub const EGU_: u64 = 0b00 << 62;   
    
    pub const CUD_: u64 = 0b01 << 60;
    
    pub const EGT_: u64 = 0b10 << 60;
    
    pub const EGV_: u64 = 0b11 << 60;
    
    pub const Cet: u64 = 1 << 63;
    
    pub const Cei: u64 = 1 << 60;

    
    pub fn qtk(eko: u32, jji: u32, abv: u8, avz: bool) -> u64 {
        let mut aad: u64 = 0;
        aad |= (eko as u64) & 0xFFFF_FFFF;    
        if avz {
            aad |= Cet;
            aad |= ((jji as u64) & 0x1FFF) << 32;  
        }
        aad |= ((abv as u64) & 0xFF) << 48;  
        aad |= CUD_;   
        aad |= Cei;          
        aad
    }
}


pub struct VirtualGic {
    
    fpi: u32,
    
    egk: [u32; 64],
    ewn: usize,
    
    jr: bool,
}


static BIG_: AtomicBool = AtomicBool::new(false);

impl VirtualGic {
    
    pub const fn new() -> Self {
        VirtualGic {
            fpi: 0,
            egk: [0; 64],
            ewn: 0,
            jr: false,
        }
    }

    
    
    
    pub fn init(&mut self) {
        #[cfg(target_arch = "aarch64")]
        {
            
            let mqb: u64;
            unsafe {
                core::arch::asm!(
                    "mrs {vtr}, ich_vtr_el2",
                    mqb = bd(reg) mqb,
                    options(nomem, nostack)
                );
            }

            
            self.fpi = ((mqb & 0x1F) + 1) as u32;
            if self.fpi > AZT_ as u32 {
                self.fpi = AZT_ as u32;
            }

            
            unsafe {
                
                core::arch::asm!(
                    "mrs {tmp}, icc_sre_el2",
                    "orr {tmp}, {tmp}, #0x1",   
                    "orr {tmp}, {tmp}, #0x8",   
                    "msr icc_sre_el2, {tmp}",
                    "isb",
                    gup = bd(reg) _,
                    options(nomem, nostack)
                );

                
                let lbp = ich_hcr::Cbf;
                core::arch::asm!(
                    "msr ich_hcr_el2, {val}",
                    "isb",
                    ap = in(reg) lbp,
                    options(nomem, nostack)
                );
            }

            self.jr = true;
            BIG_.store(true, Ordering::Release);
        }

        #[cfg(not(target_arch = "aarch64"))]
        {
            self.fpi = 4;
            self.jr = true;
            BIG_.store(true, Ordering::Release);
        }
    }

    
    pub fn ky(&self) -> bool {
        self.jr
    }

    
    pub fn zeb(&self) -> u32 {
        self.fpi
    }

    
    
    
    
    pub fn oes(&mut self, eko: u32, jji: u32, abv: u8) -> bool {
        #[cfg(target_arch = "aarch64")]
        {
            
            for a in 0..self.fpi {
                let uip = self.vsa(a);
                let g = (uip >> 60) & 0x3;
                if g == 0 {
                    
                    let aad = ich_lr::qtk(eko, jji, abv, true);
                    self.xvn(a, aad);
                    return true;
                }
            }

            
            if self.ewn < self.egk.len() {
                self.egk[self.ewn] = eko;
                self.ewn += 1;
            }
            false
        }

        #[cfg(not(target_arch = "aarch64"))]
        {
            let _ = (eko, jji, abv);
            false
        }
    }

    
    pub fn ymt(&mut self) {
        if self.ewn == 0 {
            return;
        }

        let mut ia = 0;
        for a in 0..self.ewn {
            let eko = self.egk[a];
            if !self.oes(eko, eko, 0xA0) {
                
                self.egk[ia] = eko;
                ia += 1;
            }
        }
        self.ewn = ia;
    }

    
    fn vsa(&self, w: u32) -> u64 {
        #[cfg(target_arch = "aarch64")]
        unsafe {
            let ap: u64;
            match w {
                0 => core::arch::asm!("mrs {v}, ich_lr0_el2", p = bd(reg) ap, options(nomem, nostack)),
                1 => core::arch::asm!("mrs {v}, ich_lr1_el2", p = bd(reg) ap, options(nomem, nostack)),
                2 => core::arch::asm!("mrs {v}, ich_lr2_el2", p = bd(reg) ap, options(nomem, nostack)),
                3 => core::arch::asm!("mrs {v}, ich_lr3_el2", p = bd(reg) ap, options(nomem, nostack)),
                4 => core::arch::asm!("mrs {v}, ich_lr4_el2", p = bd(reg) ap, options(nomem, nostack)),
                5 => core::arch::asm!("mrs {v}, ich_lr5_el2", p = bd(reg) ap, options(nomem, nostack)),
                6 => core::arch::asm!("mrs {v}, ich_lr6_el2", p = bd(reg) ap, options(nomem, nostack)),
                7 => core::arch::asm!("mrs {v}, ich_lr7_el2", p = bd(reg) ap, options(nomem, nostack)),
                8 => core::arch::asm!("mrs {v}, ich_lr8_el2", p = bd(reg) ap, options(nomem, nostack)),
                9 => core::arch::asm!("mrs {v}, ich_lr9_el2", p = bd(reg) ap, options(nomem, nostack)),
                10 => core::arch::asm!("mrs {v}, ich_lr10_el2", p = bd(reg) ap, options(nomem, nostack)),
                11 => core::arch::asm!("mrs {v}, ich_lr11_el2", p = bd(reg) ap, options(nomem, nostack)),
                12 => core::arch::asm!("mrs {v}, ich_lr12_el2", p = bd(reg) ap, options(nomem, nostack)),
                13 => core::arch::asm!("mrs {v}, ich_lr13_el2", p = bd(reg) ap, options(nomem, nostack)),
                14 => core::arch::asm!("mrs {v}, ich_lr14_el2", p = bd(reg) ap, options(nomem, nostack)),
                15 => core::arch::asm!("mrs {v}, ich_lr15_el2", p = bd(reg) ap, options(nomem, nostack)),
                _ => return 0,
            }
            ap
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            let _ = w;
            0
        }
    }

    
    fn xvn(&self, w: u32, ap: u64) {
        #[cfg(target_arch = "aarch64")]
        unsafe {
            match w {
                0 => core::arch::asm!("msr ich_lr0_el2, {v}", p = in(reg) ap, options(nomem, nostack)),
                1 => core::arch::asm!("msr ich_lr1_el2, {v}", p = in(reg) ap, options(nomem, nostack)),
                2 => core::arch::asm!("msr ich_lr2_el2, {v}", p = in(reg) ap, options(nomem, nostack)),
                3 => core::arch::asm!("msr ich_lr3_el2, {v}", p = in(reg) ap, options(nomem, nostack)),
                4 => core::arch::asm!("msr ich_lr4_el2, {v}", p = in(reg) ap, options(nomem, nostack)),
                5 => core::arch::asm!("msr ich_lr5_el2, {v}", p = in(reg) ap, options(nomem, nostack)),
                6 => core::arch::asm!("msr ich_lr6_el2, {v}", p = in(reg) ap, options(nomem, nostack)),
                7 => core::arch::asm!("msr ich_lr7_el2, {v}", p = in(reg) ap, options(nomem, nostack)),
                8 => core::arch::asm!("msr ich_lr8_el2, {v}", p = in(reg) ap, options(nomem, nostack)),
                9 => core::arch::asm!("msr ich_lr9_el2, {v}", p = in(reg) ap, options(nomem, nostack)),
                10 => core::arch::asm!("msr ich_lr10_el2, {v}", p = in(reg) ap, options(nomem, nostack)),
                11 => core::arch::asm!("msr ich_lr11_el2, {v}", p = in(reg) ap, options(nomem, nostack)),
                12 => core::arch::asm!("msr ich_lr12_el2, {v}", p = in(reg) ap, options(nomem, nostack)),
                13 => core::arch::asm!("msr ich_lr13_el2, {v}", p = in(reg) ap, options(nomem, nostack)),
                14 => core::arch::asm!("msr ich_lr14_el2, {v}", p = in(reg) ap, options(nomem, nostack)),
                15 => core::arch::asm!("msr ich_lr15_el2, {v}", p = in(reg) ap, options(nomem, nostack)),
                _ => {}
            }
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            let _ = (w, ap);
        }
    }
}


pub fn tjl(vgic: &mut VirtualGic) {
    #[cfg(target_arch = "aarch64")]
    {
        
        let esx: u64;
        unsafe {
            core::arch::asm!(
                "mrs {id}, icc_iar1_el1",
                ad = bd(reg) esx,
                options(nomem, nostack)
            );
        }

        let esx = esx as u32;

        
        if esx >= 1020 {
            return;
        }

        
        vgic.oes(esx, esx, 0xA0);

        
        unsafe {
            core::arch::asm!(
                "msr icc_eoir1_el1, {id}",
                ad = in(reg) esx as u64,
                options(nomem, nostack)
            );
        }

        
        unsafe {
            core::arch::asm!(
                "msr icc_dir_el1, {id}",
                ad = in(reg) esx as u64,
                options(nomem, nostack)
            );
        }
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        let _ = vgic;
    }
}
