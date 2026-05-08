










use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;

fn sm(addr: u64) -> Option<u32> {
    if addr == 0 { return None; }
    unsafe {
        let ptr = addr as *const u32;
        Some(core::ptr::read_volatile(ptr))
    }
}


const UP_: u64 = 0x000;
const AEI_: u64 = 0x004;
const CAN_: u64 = 0x008;
const CAQ_: u64 = 0x100;
const CAR_: u64 = 0x200;
const CAO_: u64 = 0x300;
const DRA_: u64 = 0x400;
const DRC_: u64 = 0x800;
const DQZ_: u64 = 0xC00;


#[cfg(target_arch = "aarch64")]
const CAT_: &[(u64, u64, &str)] = &[
    (0x0800_0000, 0x0801_0000, "QEMU virt GICv2/v3"),
    (0x0804_0000, 0x0806_0000, "QEMU virt GICv3 (alt)"),
    (0xFF84_1000, 0xFF84_2000, "BCM2711 GIC-400"),
    (0x1780_0000, 0x1790_0000, "Snapdragon GIC (apps)"),
];


#[cfg(target_arch = "aarch64")]
fn lcl(base: u64) -> String {
    let mut out = String::new();
    
    if let Some(dfy) = sm(base + AEI_) {
        let mup = dfy & 0x1F;
        let kyo = (dfy >> 5) & 0x7;
        let omy = (dfy >> 10) & 1;
        let cmv = (mup + 1) * 32;
        
        out.push_str(&format!("  TYPER    = 0x{:08X}\n", dfy));
        out.push_str(&format!("    Max IRQs: {}  CPUs: {}  Security Extensions: {}\n",
            cmv, kyo + 1, omy == 1));
    }
    
    if let Some(iidr) = sm(base + CAN_) {
        let product = (iidr >> 24) & 0xFF;
        let dru = iidr & 0xFFF;
        let edp = (iidr >> 16) & 0xF;
        let revision = (iidr >> 12) & 0xF;
        
        let gcb = match dru {
            0x43B => "ARM",
            0x42 => "Broadcom",
            0x51 => "Qualcomm",
            _ => "Unknown",
        };
        
        out.push_str(&format!("  IIDR     = 0x{:08X}\n", iidr));
        out.push_str(&format!("    Implementer: {} (0x{:03X})  Product: 0x{:02X}  Rev: {}.{}\n",
            gcb, dru, product, edp, revision));
    }
    
    if let Some(ctlr) = sm(base + UP_) {
        let hvn = ctlr & 1;
        let hvo = (ctlr >> 1) & 1;
        let jxl = (ctlr >> 4) & 1; 
        let jxk = (ctlr >> 5) & 1; 
        
        out.push_str(&format!("  CTLR     = 0x{:08X}\n", ctlr));
        out.push_str(&format!("    Group0: {}  Group1: {}  ARE-S: {}  ARE-NS: {}\n",
            hvn == 1, hvo == 1, jxl == 1, jxk == 1));
        
        if hvn == 0 && hvo == 0 {
            out.push_str("    \x01Y[WARN] GIC Distributor disabled — all IRQs masked\x01W\n");
        }
    }
    
    
    out.push_str("\n  Enabled IRQs:\n");
    let mut dop = 0;
    for gi in 0..32u64 {
        if let Some(val) = sm(base + CAQ_ + gi * 4) {
            if val != 0 {
                for bf in 0..32 {
                    if val & (1 << bf) != 0 {
                        let irq = gi * 32 + bf;
                        let pending = sm(base + CAR_ + (irq / 32) * 4)
                            .map(|v| (v >> (irq % 32)) & 1 == 1)
                            .unwrap_or(false);
                        let active = sm(base + CAO_ + (irq / 32) * 4)
                            .map(|v| (v >> (irq % 32)) & 1 == 1)
                            .unwrap_or(false);
                        
                        let state = match (pending, active) {
                            (true, true) => "\x01R[ACTIVE+PENDING]",
                            (false, true) => "\x01Y[ACTIVE]",
                            (true, false) => "\x01Y[PENDING]",
                            (false, false) => "\x01G[ENABLED]",
                        };
                        
                        
                        if dop < 20 {
                            out.push_str(&format!("    IRQ {:>4}: {}\x01W\n", irq, state));
                        }
                        dop += 1;
                    }
                }
            }
        }
    }
    if dop > 20 {
        out.push_str(&format!("    ... and {} more enabled IRQs\n", dop - 20));
    }
    out.push_str(&format!("  Total enabled: {}\n", dop));
    
    out
}


pub fn jde() -> String {
    let mut output = String::new();
    
    #[cfg(target_arch = "aarch64")]
    {
        output.push_str("\x01C== TrustProbe: ARM GIC Topology Mapper ==\x01W\n\n");
        
        for &(dist_base, _rdist_base, name) in CAT_ {
            output.push_str(&format!("\x01YProbing {} @ 0x{:08X}\x01W\n", name, dist_base));
            
            if let Some(val) = sm(dist_base) {
                if val != 0xFFFFFFFF {
                    output.push_str(&format!("\x01G[FOUND]\x01W GIC Distributor\n"));
                    output.push_str(&lcl(dist_base));
                } else {
                    output.push_str(&format!("\x01R[NOT PRESENT]\x01W\n"));
                }
            }
            output.push_str("\n");
        }
        
        
        output.push_str("\x01Y--- IRQ Security Analysis ---\x01W\n");
        output.push_str("Checking for potential IRQ-based attack vectors:\n");
        output.push_str("  - Group 0 (Secure) IRQs visible from Normal World: ");
        
        
        let mek = 0x0800_0000u64; 
        if let Some(igroupr) = sm(mek + 0x080) {
            if igroupr != 0xFFFFFFFF {
                let jec = (!igroupr).count_ones();
                if jec > 0 {
                    output.push_str(&format!("\x01R{} found — potential hijack vector\x01W\n", jec));
                } else {
                    output.push_str("\x01GNone (properly configured)\x01W\n");
                }
            } else {
                output.push_str("Cannot read (access denied)\n");
            }
        } else {
            output.push_str("GIC not accessible\n");
        }
    }
    
    #[cfg(target_arch = "x86_64")]
    {
        output.push_str("\x01C== TrustProbe: x86 APIC Topology Mapper ==\x01W\n\n");
        
        
        let esf = 0xFEE0_0000u64;
        output.push_str(&format!("Local APIC @ 0x{:08X}:\n", esf));
        
        if let Some(id) = sm(esf + 0x20) {
            output.push_str(&format!("  ID      = 0x{:08X} (APIC ID: {})\n", id, (id >> 24) & 0xFF));
        }
        if let Some(tu) = sm(esf + 0x30) {
            let ndb = (tu >> 16) & 0xFF;
            output.push_str(&format!("  Version = 0x{:08X} (Max LVT: {})\n", tu, ndb));
        }
        if let Some(svr) = sm(esf + 0xF0) {
            let enabled = (svr >> 8) & 1;
            output.push_str(&format!("  SVR     = 0x{:08X} (Enabled: {})\n", svr, enabled == 1));
        }
        
        
        let erc = 0xFEC0_0000u64;
        output.push_str(&format!("\nI/O APIC @ 0x{:08X}:\n", erc));
        
        
        if let Some(_) = sm(erc) {
            output.push_str("  I/O APIC accessible\n");
            output.push_str("  (Indirect access via IOREGSEL/IOWIN)\n");
        }
    }
    
    #[cfg(target_arch = "riscv64")]
    {
        output.push_str("\x01C== TrustProbe: RISC-V PLIC Mapper ==\x01W\n\n");
        
        let gnk = 0x0C00_0000u64;
        output.push_str(&format!("PLIC @ 0x{:08X}:\n", gnk));
        
        
        
        
        if let Some(pending) = sm(gnk + 0x1000) {
            output.push_str(&format!("  Pending word 0: 0x{:08X} ({} pending)\n",
                pending, pending.count_ones()));
        }
        
        
        let mut hdx = 0;
        for i in 0..128u64 {
            if let Some(prio) = sm(gnk + i * 4) {
                if prio > 0 {
                    hdx += 1;
                }
            }
        }
        output.push_str(&format!("  Active sources (priority > 0): {}\n", hdx));
    }
    
    output
}
