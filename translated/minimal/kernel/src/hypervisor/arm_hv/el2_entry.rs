
























use super::{Ss, Ic, hcr, hnf};
use super::stage2::Stage2Tables;
use super::trap_handler;
use super::vgic::VirtualGic;
use super::mmio_spy;

use core::sync::atomic::{AtomicBool, Ordering};


static AEW_: AtomicBool = AtomicBool::new(false);








pub fn mqr() {
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

            tmp = out(reg) _,
            sync_handler = sym el2_sync_entry,
            irq_handler = sym el2_irq_entry,
            options(nostack)
        );
    }
}


#[cfg(target_arch = "aarch64")]
#[no_mangle]
extern "C" fn el2_sync_entry() {
    
    let esr: u64;
    let far: u64;
    let hpfar: u64;

    unsafe {
        core::arch::asm!(
            "mrs {esr}, esr_el2",
            "mrs {far}, far_el2",
            "mrs {hpfar}, hpfar_el2",
            esr = out(reg) esr,
            far = out(reg) far,
            hpfar = out(reg) hpfar,
            options(nomem, nostack)
        );
    }

    
    
    unsafe {
        let regs = &mut UZ_.x;
        let action = trap_handler::mio(esr, far, hpfar, regs);

        match action {
            trap_handler::TrapAction::Handled => {
                
                let elr: u64;
                core::arch::asm!("mrs {e}, elr_el2", e = out(reg) elr, options(nomem, nostack));
                let il = if (esr >> 25) & 1 != 0 { 4u64 } else { 2u64 };
                core::arch::asm!("msr elr_el2, {e}", e = in(reg) elr + il, options(nomem, nostack));
            }
            trap_handler::TrapAction::ForwardSmc => {
                
                
                
            }
            trap_handler::TrapAction::InjectFault => {
                
                
            }
            trap_handler::TrapAction::GuestHalt => {
                AEW_.store(false, Ordering::Release);
            }
        }
    }
}


#[cfg(target_arch = "aarch64")]
#[no_mangle]
extern "C" fn el2_irq_entry() {
    unsafe {
        super::vgic::mhr(&mut BKO_);
    }
}


static mut UZ_: Ss = Ss {
    x: [0; 31],
    sp_el1: 0,
    elr_el1: 0,
    spsr_el1: 0,
    sctlr_el1: 0,
    ttbr0_el1: 0,
    ttbr1_el1: 0,
    tcr_el1: 0,
    mair_el1: 0,
    vbar_el1: 0,
    esr_el2: 0,
    far_el2: 0,
    hpfar_el2: 0,
};


static mut BKO_: VirtualGic = VirtualGic::new();









pub fn qfd(config: &Ic) -> ! {
    
    if !super::cll() {
        panic!("ARM hypervisor requires EL2! Current EL is lower.");
    }

    
    let mut azn = Stage2Tables::new(1); 

    
    azn.map_ram(config.guest_ram_base, config.guest_ram_size);

    
    for &(base, size) in &config.trapped_mmio {
        let label = mmio_spy::btg(base);
        azn.trap_mmio(base, size, label);
    }

    
    let hcr_val = hnf(config);

    
    #[cfg(target_arch = "aarch64")]
    unsafe {
        
        core::arch::asm!(
            "msr hcr_el2, {hcr}",
            "isb",
            hcr = in(reg) hcr_val,
            options(nomem, nostack)
        );

        
        let vttbr = azn.vttbr();
        core::arch::asm!(
            "msr vttbr_el2, {vttbr}",
            "isb",
            vttbr = in(reg) vttbr,
            options(nomem, nostack)
        );

        
        
        let vtcr: u64 = (24 << 0)     
                       | (0b01 << 6)   
                       | (0b01 << 8)   
                       | (0b01 << 10)  
                       | (0b11 << 12)  
                       | (0b00 << 14)  
                       | (1 << 31);    
        core::arch::asm!(
            "msr vtcr_el2, {vtcr}",
            "isb",
            vtcr = in(reg) vtcr,
            options(nomem, nostack)
        );
    }

    
    mqr();

    
    unsafe {
        BKO_.init();
    }

    
    unsafe {
        UZ_.x[0] = config.guest_dtb;    
        UZ_.elr_el1 = config.guest_entry;  
        UZ_.spsr_el1 = 0x3C5;              
    }

    AEW_.store(true, Ordering::Release);

    
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
            entry = in(reg) config.guest_entry,
            spsr = in(reg) 0x3C5u64,
            dtb = in(reg) config.guest_dtb,
            options(noreturn)
        );
    }

    #[cfg(not(target_arch = "aarch64"))]
    loop {
        core::hint::spin_loop();
    }
}


pub fn qmj() -> bool {
    AEW_.load(Ordering::Acquire)
}


pub fn mdv() -> alloc::string::String {
    use alloc::format;
    let nfo = mmio_spy::gzs();
    let otx = mmio_spy::fdl();

    let mut j = format!(
        "=== TrustOS EL2 Hypervisor Spy Report ===\n\
         MMIO accesses intercepted: {}\n\
         SMC calls intercepted: {}\n",
        nfo, otx
    );

    
    let stats = mmio_spy::hrz();
    if !stats.is_empty() {
        j.push_str("\n--- Device Activity ---\n");
        for (name, reads, writes) in &stats {
            j.push_str(&format!("  {:<20} R:{:<6} W:{}\n", name, reads, writes));
        }
    }

    
    let cpd = mmio_spy::iyt(10);
    if !cpd.is_empty() {
        j.push_str("\n--- Recent MMIO (newest first) ---\n");
        for rt in &cpd {
            j.push_str(&format!("  {}\n", mmio_spy::lxo(rt)));
        }
    }

    
    let jgq = mmio_spy::gqq(5);
    if !jgq.is_empty() {
        j.push_str("\n--- Recent SMC Calls ---\n");
        for rt in &jgq {
            j.push_str(&format!("  {}\n", mmio_spy::hzq(rt)));
        }
    }

    j
}
