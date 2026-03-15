










use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;

fn akp(ag: u64) -> Option<u32> {
    if ag == 0 { return None; }
    unsafe {
        let ptr = ag as *const u32;
        Some(core::ptr::read_volatile(ptr))
    }
}


const TJ_: u64 = 0x000;
const ACS_: u64 = 0x004;
const BXH_: u64 = 0x008;
const BXK_: u64 = 0x100;
const BXL_: u64 = 0x200;
const BXI_: u64 = 0x300;
const DNG_: u64 = 0x400;
const DNI_: u64 = 0x800;
const DNF_: u64 = 0xC00;


#[cfg(target_arch = "aarch64")]
const BXN_: &[(u64, u64, &str)] = &[
    (0x0800_0000, 0x0801_0000, "QEMU virt GICv2/v3"),
    (0x0804_0000, 0x0806_0000, "QEMU virt GICv3 (alt)"),
    (0xFF84_1000, 0xFF84_2000, "BCM2711 GIC-400"),
    (0x1780_0000, 0x1790_0000, "Snapdragon GIC (apps)"),
];


#[cfg(target_arch = "aarch64")]
fn run(ar: u64) -> String {
    let mut bd = String::new();
    
    if let Some(gvg) = akp(ar + ACS_) {
        let tzx = gvg & 0x1F;
        let rps = (gvg >> 5) & 0x7;
        let wgh = (gvg >> 10) & 1;
        let fnw = (tzx + 1) * 32;
        
        bd.t(&format!("  TYPER    = 0x{:08X}\n", gvg));
        bd.t(&format!("    Max IRQs: {}  CPUs: {}  Security Extensions: {}\n",
            fnw, rps + 1, wgh == 1));
    }
    
    if let Some(hns) = akp(ar + BXH_) {
        let baj = (hns >> 24) & 0xFF;
        let odm = hns & 0xFFF;
        let xqs = (hns >> 16) & 0xF;
        let afe = (hns >> 12) & 0xF;
        
        let tsi = match odm {
            0x43B => "ARM",
            0x42 => "Broadcom",
            0x51 => "Qualcomm",
            _ => "Unknown",
        };
        
        bd.t(&format!("  IIDR     = 0x{:08X}\n", hns));
        bd.t(&format!("    Implementer: {} (0x{:03X})  Product: 0x{:02X}  Rev: {}.{}\n",
            tsi, odm, baj, xqs, afe));
    }
    
    if let Some(hen) = akp(ar + TJ_) {
        let npt = hen & 1;
        let npu = (hen >> 1) & 1;
        let qkh = (hen >> 4) & 1; 
        let qkg = (hen >> 5) & 1; 
        
        bd.t(&format!("  CTLR     = 0x{:08X}\n", hen));
        bd.t(&format!("    Group0: {}  Group1: {}  ARE-S: {}  ARE-NS: {}\n",
            npt == 1, npu == 1, qkh == 1, qkg == 1));
        
        if npt == 0 && npu == 0 {
            bd.t("    \x01Y[WARN] GIC Distributor disabled — all IRQs masked\x01W\n");
        }
    }
    
    
    bd.t("\n  Enabled IRQs:\n");
    let mut hhz = 0;
    for om in 0..32u64 {
        if let Some(ap) = akp(ar + BXK_ + om * 4) {
            if ap != 0 {
                for ga in 0..32 {
                    if ap & (1 << ga) != 0 {
                        let irq = om * 32 + ga;
                        let aln = akp(ar + BXL_ + (irq / 32) * 4)
                            .map(|p| (p >> (irq % 32)) & 1 == 1)
                            .unwrap_or(false);
                        let gh = akp(ar + BXI_ + (irq / 32) * 4)
                            .map(|p| (p >> (irq % 32)) & 1 == 1)
                            .unwrap_or(false);
                        
                        let g = match (aln, gh) {
                            (true, true) => "\x01R[ACTIVE+PENDING]",
                            (false, true) => "\x01Y[ACTIVE]",
                            (true, false) => "\x01Y[PENDING]",
                            (false, false) => "\x01G[ENABLED]",
                        };
                        
                        
                        if hhz < 20 {
                            bd.t(&format!("    IRQ {:>4}: {}\x01W\n", irq, g));
                        }
                        hhz += 1;
                    }
                }
            }
        }
    }
    if hhz > 20 {
        bd.t(&format!("    ... and {} more enabled IRQs\n", hhz - 20));
    }
    bd.t(&format!("  Total enabled: {}\n", hhz));
    
    bd
}


pub fn pgd() -> String {
    let mut an = String::new();
    
    #[cfg(target_arch = "aarch64")]
    {
        an.t("\x01C== TrustProbe: ARM GIC Topology Mapper ==\x01W\n\n");
        
        for &(kqf, ycb, j) in BXN_ {
            an.t(&format!("\x01YProbing {} @ 0x{:08X}\x01W\n", j, kqf));
            
            if let Some(ap) = akp(kqf) {
                if ap != 0xFFFFFFFF {
                    an.t(&format!("\x01G[FOUND]\x01W GIC Distributor\n"));
                    an.t(&run(kqf));
                } else {
                    an.t(&format!("\x01R[NOT PRESENT]\x01W\n"));
                }
            }
            an.t("\n");
        }
        
        
        an.t("\x01Y--- IRQ Security Analysis ---\x01W\n");
        an.t("Checking for potential IRQ-based attack vectors:\n");
        an.t("  - Group 0 (Secure) IRQs visible from Normal World: ");
        
        
        let tfm = 0x0800_0000u64; 
        if let Some(odg) = akp(tfm + 0x080) {
            if odg != 0xFFFFFFFF {
                let phi = (!odg).ipi();
                if phi > 0 {
                    an.t(&format!("\x01R{} found — potential hijack vector\x01W\n", phi));
                } else {
                    an.t("\x01GNone (properly configured)\x01W\n");
                }
            } else {
                an.t("Cannot read (access denied)\n");
            }
        } else {
            an.t("GIC not accessible\n");
        }
    }
    
    #[cfg(target_arch = "x86_64")]
    {
        an.t("\x01C== TrustProbe: x86 APIC Topology Mapper ==\x01W\n\n");
        
        
        let jci = 0xFEE0_0000u64;
        an.t(&format!("Local APIC @ 0x{:08X}:\n", jci));
        
        if let Some(ad) = akp(jci + 0x20) {
            an.t(&format!("  ID      = 0x{:08X} (APIC ID: {})\n", ad, (ad >> 24) & 0xFF));
        }
        if let Some(axh) = akp(jci + 0x30) {
            let uln = (axh >> 16) & 0xFF;
            an.t(&format!("  Version = 0x{:08X} (Max LVT: {})\n", axh, uln));
        }
        if let Some(bim) = akp(jci + 0xF0) {
            let iq = (bim >> 8) & 1;
            an.t(&format!("  SVR     = 0x{:08X} (Enabled: {})\n", bim, iq == 1));
        }
        
        
        let jau = 0xFEC0_0000u64;
        an.t(&format!("\nI/O APIC @ 0x{:08X}:\n", jau));
        
        
        if let Some(_) = akp(jau) {
            an.t("  I/O APIC accessible\n");
            an.t("  (Indirect access via IOREGSEL/IOWIN)\n");
        }
    }
    
    #[cfg(target_arch = "riscv64")]
    {
        an.t("\x01C== TrustProbe: RISC-V PLIC Mapper ==\x01W\n\n");
        
        let luj = 0x0C00_0000u64;
        an.t(&format!("PLIC @ 0x{:08X}:\n", luj));
        
        
        
        
        if let Some(aln) = akp(luj + 0x1000) {
            an.t(&format!("  Pending word 0: 0x{:08X} ({} pending)\n",
                aln, aln.ipi()));
        }
        
        
        let mut mtq = 0;
        for a in 0..128u64 {
            if let Some(vlk) = akp(luj + a * 4) {
                if vlk > 0 {
                    mtq += 1;
                }
            }
        }
        an.t(&format!("  Active sources (priority > 0): {}\n", mtq));
    }
    
    an
}
