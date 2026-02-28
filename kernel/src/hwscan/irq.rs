//! Interrupt Controller Topology Mapper
//!
//! Maps the full interrupt routing topology of the system:
//!   - ARM: GICv2/GICv3 Distributor, Redistributors, CPU interfaces
//!   - x86: Local APIC, I/O APIC, MSI/MSI-X
//!   - RISC-V: PLIC (Platform-Level Interrupt Controller)
//!
//! Security relevance: Interrupts that are misconfigured can be
//! hijacked for privilege escalation. Hidden or undocumented IRQs
//! can reveal debug interfaces.

use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;

fn safe_read(addr: u64) -> Option<u32> {
    if addr == 0 { return None; }
    unsafe {
        let ptr = addr as *const u32;
        Some(core::ptr::read_volatile(ptr))
    }
}

/// GICv2/v3 Distributor register offsets
const GICD_CTLR: u64 = 0x000;
const GICD_TYPER: u64 = 0x004;
const GICD_IIDR: u64 = 0x008;
const GICD_ISENABLER0: u64 = 0x100;
const GICD_ISPENDR0: u64 = 0x200;
const GICD_ISACTIVER0: u64 = 0x300;
const GICD_IPRIORITYR0: u64 = 0x400;
const GICD_ITARGETSR0: u64 = 0x800;
const GICD_ICFGR0: u64 = 0xC00;

/// Known GIC base addresses
#[cfg(target_arch = "aarch64")]
const GIC_BASES: &[(u64, u64, &str)] = &[
    (0x0800_0000, 0x0801_0000, "QEMU virt GICv2/v3"),
    (0x0804_0000, 0x0806_0000, "QEMU virt GICv3 (alt)"),
    (0xFF84_1000, 0xFF84_2000, "BCM2711 GIC-400"),
    (0x1780_0000, 0x1790_0000, "Snapdragon GIC (apps)"),
];

/// Decode GIC Distributor
#[cfg(target_arch = "aarch64")]
fn decode_gic_distributor(base: u64) -> String {
    let mut out = String::new();
    
    if let Some(typer) = safe_read(base + GICD_TYPER) {
        let it_lines = typer & 0x1F;
        let cpu_number = (typer >> 5) & 0x7;
        let security_extn = (typer >> 10) & 1;
        let max_irqs = (it_lines + 1) * 32;
        
        out.push_str(&format!("  TYPER    = 0x{:08X}\n", typer));
        out.push_str(&format!("    Max IRQs: {}  CPUs: {}  Security Extensions: {}\n",
            max_irqs, cpu_number + 1, security_extn == 1));
    }
    
    if let Some(iidr) = safe_read(base + GICD_IIDR) {
        let product = (iidr >> 24) & 0xFF;
        let implementer = iidr & 0xFFF;
        let variant = (iidr >> 16) & 0xF;
        let revision = (iidr >> 12) & 0xF;
        
        let impl_name = match implementer {
            0x43B => "ARM",
            0x42 => "Broadcom",
            0x51 => "Qualcomm",
            _ => "Unknown",
        };
        
        out.push_str(&format!("  IIDR     = 0x{:08X}\n", iidr));
        out.push_str(&format!("    Implementer: {} (0x{:03X})  Product: 0x{:02X}  Rev: {}.{}\n",
            impl_name, implementer, product, variant, revision));
    }
    
    if let Some(ctlr) = safe_read(base + GICD_CTLR) {
        let enable_grp0 = ctlr & 1;
        let enable_grp1 = (ctlr >> 1) & 1;
        let are_s = (ctlr >> 4) & 1; // GICv3: Affinity Routing Enable, Secure
        let are_ns = (ctlr >> 5) & 1; // GICv3: Affinity Routing Enable, Non-secure
        
        out.push_str(&format!("  CTLR     = 0x{:08X}\n", ctlr));
        out.push_str(&format!("    Group0: {}  Group1: {}  ARE-S: {}  ARE-NS: {}\n",
            enable_grp0 == 1, enable_grp1 == 1, are_s == 1, are_ns == 1));
        
        if enable_grp0 == 0 && enable_grp1 == 0 {
            out.push_str("    \x01Y[WARN] GIC Distributor disabled — all IRQs masked\x01W\n");
        }
    }
    
    // Scan enabled IRQs
    out.push_str("\n  Enabled IRQs:\n");
    let mut enabled_count = 0;
    for bank in 0..32u64 {
        if let Some(val) = safe_read(base + GICD_ISENABLER0 + bank * 4) {
            if val != 0 {
                for bit in 0..32 {
                    if val & (1 << bit) != 0 {
                        let irq = bank * 32 + bit;
                        let pending = safe_read(base + GICD_ISPENDR0 + (irq / 32) * 4)
                            .map(|v| (v >> (irq % 32)) & 1 == 1)
                            .unwrap_or(false);
                        let active = safe_read(base + GICD_ISACTIVER0 + (irq / 32) * 4)
                            .map(|v| (v >> (irq % 32)) & 1 == 1)
                            .unwrap_or(false);
                        
                        let state = match (pending, active) {
                            (true, true) => "\x01R[ACTIVE+PENDING]",
                            (false, true) => "\x01Y[ACTIVE]",
                            (true, false) => "\x01Y[PENDING]",
                            (false, false) => "\x01G[ENABLED]",
                        };
                        
                        // Only show first 20 to avoid flood
                        if enabled_count < 20 {
                            out.push_str(&format!("    IRQ {:>4}: {}\x01W\n", irq, state));
                        }
                        enabled_count += 1;
                    }
                }
            }
        }
    }
    if enabled_count > 20 {
        out.push_str(&format!("    ... and {} more enabled IRQs\n", enabled_count - 20));
    }
    out.push_str(&format!("  Total enabled: {}\n", enabled_count));
    
    out
}

/// Main IRQ topology scan
pub fn scan_irq_topology() -> String {
    let mut output = String::new();
    
    #[cfg(target_arch = "aarch64")]
    {
        output.push_str("\x01C== TrustProbe: ARM GIC Topology Mapper ==\x01W\n\n");
        
        for &(dist_base, _rdist_base, name) in GIC_BASES {
            output.push_str(&format!("\x01YProbing {} @ 0x{:08X}\x01W\n", name, dist_base));
            
            if let Some(val) = safe_read(dist_base) {
                if val != 0xFFFFFFFF {
                    output.push_str(&format!("\x01G[FOUND]\x01W GIC Distributor\n"));
                    output.push_str(&decode_gic_distributor(dist_base));
                } else {
                    output.push_str(&format!("\x01R[NOT PRESENT]\x01W\n"));
                }
            }
            output.push_str("\n");
        }
        
        // Security analysis
        output.push_str("\x01Y--- IRQ Security Analysis ---\x01W\n");
        output.push_str("Checking for potential IRQ-based attack vectors:\n");
        output.push_str("  - Group 0 (Secure) IRQs visible from Normal World: ");
        
        // Test: can we read Group 0 configuration?
        let gic_base = 0x0800_0000u64; // QEMU default
        if let Some(igroupr) = safe_read(gic_base + 0x080) {
            if igroupr != 0xFFFFFFFF {
                let secure_irqs = (!igroupr).count_ones();
                if secure_irqs > 0 {
                    output.push_str(&format!("\x01R{} found — potential hijack vector\x01W\n", secure_irqs));
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
        
        // Local APIC (typically at 0xFEE00000)
        let lapic_base = 0xFEE0_0000u64;
        output.push_str(&format!("Local APIC @ 0x{:08X}:\n", lapic_base));
        
        if let Some(id) = safe_read(lapic_base + 0x20) {
            output.push_str(&format!("  ID      = 0x{:08X} (APIC ID: {})\n", id, (id >> 24) & 0xFF));
        }
        if let Some(ver) = safe_read(lapic_base + 0x30) {
            let max_lvt = (ver >> 16) & 0xFF;
            output.push_str(&format!("  Version = 0x{:08X} (Max LVT: {})\n", ver, max_lvt));
        }
        if let Some(svr) = safe_read(lapic_base + 0xF0) {
            let enabled = (svr >> 8) & 1;
            output.push_str(&format!("  SVR     = 0x{:08X} (Enabled: {})\n", svr, enabled == 1));
        }
        
        // I/O APIC (typically at 0xFEC00000)
        let ioapic_base = 0xFEC0_0000u64;
        output.push_str(&format!("\nI/O APIC @ 0x{:08X}:\n", ioapic_base));
        
        // I/O APIC uses indirect register access
        if let Some(_) = safe_read(ioapic_base) {
            output.push_str("  I/O APIC accessible\n");
            output.push_str("  (Indirect access via IOREGSEL/IOWIN)\n");
        }
    }
    
    #[cfg(target_arch = "riscv64")]
    {
        output.push_str("\x01C== TrustProbe: RISC-V PLIC Mapper ==\x01W\n\n");
        
        let plic_base = 0x0C00_0000u64;
        output.push_str(&format!("PLIC @ 0x{:08X}:\n", plic_base));
        
        // PLIC priority registers start at offset 0
        // PLIC pending bits at 0x1000
        // PLIC enable bits at 0x2000
        if let Some(pending) = safe_read(plic_base + 0x1000) {
            output.push_str(&format!("  Pending word 0: 0x{:08X} ({} pending)\n",
                pending, pending.count_ones()));
        }
        
        // Scan priority registers for active sources
        let mut active_sources = 0;
        for i in 0..128u64 {
            if let Some(prio) = safe_read(plic_base + i * 4) {
                if prio > 0 {
                    active_sources += 1;
                }
            }
        }
        output.push_str(&format!("  Active sources (priority > 0): {}\n", active_sources));
    }
    
    output
}
