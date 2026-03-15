
























use super::{Atg, Sw, hcr, nfh};
use super::stage2::Stage2Tables;
use super::trap_handler;
use super::vgic::VirtualGic;
use super::mmio_spy;

use core::sync::atomic::{AtomicBool, Ordering};


static ADG_: AtomicBool = AtomicBool::new(false);








pub fn tvi() {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        
        
        
        core::arch::asm!(
            "adr {tmp}, 2f",
            "msr vbar_el2, {tmp}",
            "isb",
            "b 3f",

            
            
            
            ".balign 2048",
            "2:",
            
            "b .",           
            ".balign 128",
            "b .",           
            ".balign 128",
            "b .",           
            ".balign 128",
            "b .",           
            ".balign 128",

            
            "b .",           
            ".balign 128",
            "b .",           
            ".balign 128",
            "b .",           
            ".balign 128",
            "b .",           
            ".balign 128",

            
            "b 4f",          
            ".balign 128",
            "b 5f",          
            ".balign 128",
            "b .",           
            ".balign 128",
            "b .",           
            ".balign 128",

            
            "b .",
            ".balign 128",
            "b .",
            ".balign 128",
            "b .",
            ".balign 128",
            "b .",

            
            "4:",
            "stp x29, x30, [sp, #-16]!",
            "bl {sync_handler}",
            "ldp x29, x30, [sp], #16",
            "eret",

            
            "5:",
            "stp x29, x30, [sp, #-16]!",
            "bl {irq_handler}",
            "ldp x29, x30, [sp], #16",
            "eret",

            "3:",

            gup = bd(reg) _,
            zqn = aaw sjv,
            yyt = aaw sju,
            options(nostack)
        );
    }
}


#[cfg(target_arch = "aarch64")]
#[no_mangle]
extern "C" fn sjv() {
    
    let esr: u64;
    let adt: u64;
    let esb: u64;

    unsafe {
        core::arch::asm!(
            "mrs {esr}, esr_el2",
            "mrs {far}, far_el2",
            "mrs {hpfar}, hpfar_el2",
            esr = bd(reg) esr,
            adt = bd(reg) adt,
            esb = bd(reg) esb,
            options(nomem, nostack)
        );
    }

    
    
    unsafe {
        let regs = &mut TT_.b;
        let hr = trap_handler::tld(esr, adt, esb, regs);

        match hr {
            trap_handler::TrapAction::Gw => {
                
                let npn: u64;
                core::arch::asm!("mrs {e}, elr_el2", aa = bd(reg) npn, options(nomem, nostack));
                let odh = if (esr >> 25) & 1 != 0 { 4u64 } else { 2u64 };
                core::arch::asm!("msr elr_el2, {e}", aa = in(reg) npn + odh, options(nomem, nostack));
            }
            trap_handler::TrapAction::Bhj => {
                
                
                
            }
            trap_handler::TrapAction::Auf => {
                
                
            }
            trap_handler::TrapAction::Ath => {
                ADG_.store(false, Ordering::Release);
            }
        }
    }
}


#[cfg(target_arch = "aarch64")]
#[no_mangle]
extern "C" fn sju() {
    unsafe {
        super::vgic::tjl(&mut BIH_);
    }
}


static mut TT_: Atg = Atg {
    b: [0; 31],
    wql: 0,
    bzm: 0,
    mgy: 0,
    wfb: 0,
    xnc: 0,
    xnd: 0,
    xbl: 0,
    ujj: 0,
    xqt: 0,
    snk: 0,
    sra: 0,
    tql: 0,
};


static mut BIH_: VirtualGic = VirtualGic::new();









pub fn ypm(config: &Sw) -> ! {
    
    if !super::fma() {
        panic!("ARM hypervisor requires EL2! Current EL is lower.");
    }

    
    let mut cuc = Stage2Tables::new(1); 

    
    cuc.ujs(config.hmd, config.hme);

    
    for &(ar, aw) in &config.iew {
        let cu = mmio_spy::eda(ar);
        cuc.guw(ar, aw, cu);
    }

    
    let lbp = nfh(config);

    
    #[cfg(target_arch = "aarch64")]
    unsafe {
        
        core::arch::asm!(
            "msr hcr_el2, {hcr}",
            "isb",
            hcr = in(reg) lbp,
            options(nomem, nostack)
        );

        
        let fbe = cuc.fbe();
        core::arch::asm!(
            "msr vttbr_el2, {vttbr}",
            "isb",
            fbe = in(reg) fbe,
            options(nomem, nostack)
        );

        
        
        let pyy: u64 = (24 << 0)     
                       | (0b01 << 6)   
                       | (0b01 << 8)   
                       | (0b01 << 10)  
                       | (0b11 << 12)  
                       | (0b00 << 14)  
                       | (1 << 31);    
        core::arch::asm!(
            "msr vtcr_el2, {vtcr}",
            "isb",
            pyy = in(reg) pyy,
            options(nomem, nostack)
        );
    }

    
    tvi();

    
    unsafe {
        BIH_.init();
    }

    
    unsafe {
        TT_.b[0] = config.ixj;    
        TT_.bzm = config.hma;  
        TT_.mgy = 0x3C5;              
    }

    ADG_.store(true, Ordering::Release);

    
    #[cfg(target_arch = "aarch64")]
    unsafe {
        core::arch::asm!(
            
            "msr elr_el2, {entry}",
            
            "msr spsr_el2, {spsr}",
            
            "mov x0, {dtb}",
            
            "mov x1, xzr",
            "mov x2, xzr",
            "mov x3, xzr",
            
            "tlbi vmalls12e1is",
            "dsb ish",
            "isb",
            
            "eret",
            bt = in(reg) config.hma,
            zpg = in(reg) 0x3C5u64,
            azq = in(reg) config.ixj,
            options(jhe)
        );
    }

    #[cfg(not(target_arch = "aarch64"))]
    loop {
        core::hint::hc();
    }
}


pub fn yzj() -> bool {
    ADG_.load(Ordering::Acquire)
}


pub fn tes() -> alloc::string::String {
    use alloc::format;
    let uov = mmio_spy::mmj();
    let wpv = mmio_spy::jty();

    let mut e = format!(
        "=== TrustOS EL2 Hypervisor Spy Report ===\n\
         MMIO accesses intercepted: {}\n\
         SMC calls intercepted: {}\n",
        uov, wpv
    );

    
    let cm = mmio_spy::nld();
    if !cm.is_empty() {
        e.t("\n--- Device Activity ---\n");
        for (j, exj, fbu) in &cm {
            e.t(&format!("  {:<20} R:{:<6} W:{}\n", j, exj, fbu));
        }
    }

    
    let fsj = mmio_spy::paq(10);
    if !fsj.is_empty() {
        e.t("\n--- Recent MMIO (newest first) ---\n");
        for aiz in &fsj {
            e.t(&format!("  {}\n", mmio_spy::svw(aiz)));
        }
    }

    
    let pll = mmio_spy::lyf(5);
    if !pll.is_empty() {
        e.t("\n--- Recent SMC Calls ---\n");
        for aiz in &pll {
            e.t(&format!("  {}\n", mmio_spy::nvs(aiz)));
        }
    }

    e
}
