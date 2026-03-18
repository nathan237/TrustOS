//! ACPI Table Dumper — parse host ACPI tables for hardware debug
//!
//! Uses the kernel's parsed ACPI info to dump MADT, FADT, MCFG, HPET details.

use super::dbg_out;

/// Run ACPI table diagnostics
pub fn run(args: &[&str]) {
    dbg_out!("[ACPI] === ACPI Table Analysis ===");

    let _verbose = args.contains(&"-v") || args.contains(&"--verbose");

    // Get RSDP from our ACPI module
    if let Some(info) = crate::acpi::get_info() {
        dbg_out!("[ACPI] ACPI info available: {} CPUs, {} I/O APICs",
            info.cpu_count, info.io_apics.len());

        // Dump MADT info
        dump_madt_info(&info);

        // Dump FADT info
        dump_fadt_info();

        // Dump additional parsed table info
        dump_raw_tables();
    } else {
        dbg_out!("[ACPI] No ACPI info available");
        dump_raw_tables();
    }
}

fn dump_madt_info(info: &crate::acpi::AcpiInfo) {
    dbg_out!("[ACPI] ─── MADT (Multiple APIC Description Table) ───");
    dbg_out!("[ACPI] Local APIC address: 0x{:08X}", info.local_apic_addr);
    dbg_out!("[ACPI] CPU count: {} (detected via MADT)", info.cpu_count);

    for (i, lapic) in info.local_apics.iter().enumerate() {
        dbg_out!("[ACPI]   LAPIC #{}: processor_id={} apic_id={} enabled={} online_capable={}",
            i, lapic.processor_id, lapic.apic_id, lapic.enabled, lapic.online_capable);
    }

    for (i, ioapic) in info.io_apics.iter().enumerate() {
        dbg_out!("[ACPI]   I/O APIC #{}: id={} address=0x{:08X} GSI_base={}",
            i, ioapic.id, ioapic.address, ioapic.gsi_base);
    }

    for iso in &info.int_overrides {
        dbg_out!("[ACPI]   INT Override: source={} → GSI={} polarity={} trigger={}",
            iso.source, iso.gsi, iso.polarity, iso.trigger);
    }

    for nmi in &info.local_apic_nmis {
        dbg_out!("[ACPI]   LAPIC NMI: processor_uid={} polarity={} trigger={} LINT={}",
            nmi.processor_uid, nmi.polarity, nmi.trigger, nmi.lint);
    }
}

fn dump_fadt_info() {
    dbg_out!("[ACPI] ─── FADT (Fixed ACPI Description Table) ───");
    if let Some(info) = crate::acpi::get_info() {
        if let Some(ref fadt) = info.fadt {
            dbg_out!("[ACPI] PM1a control block: 0x{:08X}", fadt.pm1a_cnt_blk);
            if fadt.pm1b_cnt_blk != 0 {
                dbg_out!("[ACPI] PM1b control block: 0x{:08X}", fadt.pm1b_cnt_blk);
            }
            dbg_out!("[ACPI] PM Timer block: 0x{:08X}", fadt.pm_tmr_blk);
            dbg_out!("[ACPI] SCI interrupt: {}", fadt.sci_int);
            dbg_out!("[ACPI] SMI command port: 0x{:08X}", fadt.smi_cmd);
            dbg_out!("[ACPI] Century register: 0x{:02X}", fadt.century_reg);
            dbg_out!("[ACPI] Flags: 0x{:08X}", fadt.flags);
            let wbinvd = fadt.flags & 1 != 0;
            let proc_c1 = fadt.flags & (1 << 2) != 0;
            let pwr_button = fadt.flags & (1 << 5) != 0;
            let slp_button = fadt.flags & (1 << 6) != 0;
            let rtc_valid = fadt.flags & (1 << 7) != 0;
            let hw_reduced = fadt.is_hw_reduced();
            let reset_sup = fadt.supports_reset();
            dbg_out!("[ACPI]   WBINVD: {}  C1: {}  PWR_BTN: {}  SLP_BTN: {}  RTC: {}  HW_Reduced: {}  Reset: {}",
                wbinvd, proc_c1, pwr_button, slp_button, rtc_valid, hw_reduced, reset_sup);
        } else {
            dbg_out!("[ACPI] FADT not parsed");
        }
    } else {
        dbg_out!("[ACPI] FADT info not available (no ACPI)");
    }
}

fn dump_raw_tables() {
    dbg_out!("[ACPI] ─── Raw Table Enumeration ───");

    // Dump info from our parsed ACPI module
    if let Some(info) = crate::acpi::get_info() {
        // MCfg regions (PCIe ECAM)
        if !info.mcfg_regions.is_empty() {
            dbg_out!("[ACPI]   MCFG: {} PCIe config space region(s)", info.mcfg_regions.len());
            for (i, region) in info.mcfg_regions.iter().enumerate() {
                dbg_out!("[ACPI]     Region {}: segment={} bus {:02X}-{:02X} base=0x{:016X}",
                    i, region.segment, region.start_bus, region.end_bus, region.base_address);
            }
        }

        // HPET
        if let Some(ref hpet) = info.hpet {
            dbg_out!("[ACPI]   HPET: base=0x{:016X} comparators={} 64bit={}",
                hpet.base_address, hpet.num_comparators, hpet.counter_64bit);
        }

        // FADT summary
        if info.fadt.is_some() {
            dbg_out!("[ACPI]   FADT: present (parsed)");
        }
    } else {
        dbg_out!("[ACPI] No parsed ACPI data available");
    }
}
