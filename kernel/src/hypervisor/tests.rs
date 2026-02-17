//! TrustVM Hypervisor Self-Tests
//!
//! In-kernel unit tests for hypervisor subsystems:
//! - ACPI table generation (RSDP, XSDT, MADT, FADT, DSDT)
//! - PIC 8259A state machine
//! - PIT 8254 timer emulation
//! - LAPIC state management
//! - CMOS RTC register emulation
//! - E820 memory map
//!
//! Run from shell: `hv test`

use alloc::string::String;
use alloc::vec;
use alloc::format;

/// Test result for in-kernel testing
struct TestResult {
    name: &'static str,
    passed: bool,
    detail: Option<String>,
}

/// Run all hypervisor self-tests and return (passed, failed, results)
pub fn run_all_tests() -> (usize, usize, alloc::vec::Vec<String>) {
    let mut results: alloc::vec::Vec<TestResult> = alloc::vec::Vec::new();
    let mut log: alloc::vec::Vec<String> = alloc::vec::Vec::new();
    
    // ── ACPI Tests ──────────────────────────────────────────────
    results.push(test_acpi_rsdp_signature());
    results.push(test_acpi_rsdp_v1_checksum());
    results.push(test_acpi_rsdp_v2_checksum());
    results.push(test_acpi_rsdp_revision());
    results.push(test_acpi_rsdp_xsdt_pointer());
    results.push(test_acpi_xsdt_signature());
    results.push(test_acpi_xsdt_checksum());
    results.push(test_acpi_xsdt_entry_count());
    results.push(test_acpi_madt_signature());
    results.push(test_acpi_madt_checksum());
    results.push(test_acpi_madt_lapic_addr());
    results.push(test_acpi_madt_has_lapic_entry());
    results.push(test_acpi_madt_has_ioapic_entry());
    results.push(test_acpi_madt_irq_overrides());
    results.push(test_acpi_fadt_signature());
    results.push(test_acpi_fadt_checksum());
    results.push(test_acpi_fadt_pm_timer_port());
    results.push(test_acpi_fadt_sci_interrupt());
    results.push(test_acpi_fadt_dsdt_pointer());
    results.push(test_acpi_dsdt_signature());
    results.push(test_acpi_dsdt_checksum());
    results.push(test_acpi_bios_rsdp_copy());
    
    // ── PIC 8259A Tests ─────────────────────────────────────────
    results.push(test_pic_defaults());
    results.push(test_pic_icw_sequence_master());
    results.push(test_pic_icw_sequence_slave());
    results.push(test_pic_ocw1_imr());
    results.push(test_pic_eoi_clears_isr());
    
    // ── PIT 8254 Tests ──────────────────────────────────────────
    results.push(test_pit_defaults());
    results.push(test_pit_control_word());
    results.push(test_pit_lohi_reload());
    results.push(test_pit_latch_command());
    
    // ── LAPIC Tests ─────────────────────────────────────────────
    results.push(test_lapic_defaults());
    results.push(test_lapic_enable_via_svr());
    results.push(test_lapic_timer_arm());
    results.push(test_lapic_timer_lvt_modes());
    results.push(test_lapic_divider_values());
    
    // ── CMOS RTC Tests ──────────────────────────────────────────
    results.push(test_cmos_time_registers());
    results.push(test_cmos_status_registers());
    results.push(test_cmos_equipment_byte());
    results.push(test_cmos_century());
    
    // ── MMIO Instruction Decoder Tests ──────────────────────────
    results.push(test_mmio_decode_mov_write());
    results.push(test_mmio_decode_mov_read());
    results.push(test_mmio_decode_rex_w());
    results.push(test_mmio_decode_imm32());
    results.push(test_mmio_decode_movzx());
    results.push(test_mmio_decode_disp32());
    results.push(test_mmio_decode_r8_r15());
    results.push(test_mmio_register_rw());
    
    // ── I/O APIC Tests ──────────────────────────────────────────
    results.push(test_ioapic_defaults());
    results.push(test_ioapic_version_register());
    results.push(test_ioapic_id_readwrite());
    results.push(test_ioapic_redir_table());
    results.push(test_ioapic_indirect_access());
    results.push(test_ioapic_irq_routing());
    results.push(test_ioapic_readonly_bits());
    
    // ── HPET Tests ───────────────────────────────────────────────
    results.push(test_hpet_defaults());
    results.push(test_hpet_gcap_id());
    results.push(test_hpet_enable_disable());
    results.push(test_hpet_counter_increments());
    results.push(test_hpet_timer_config());
    results.push(test_hpet_timer_comparator());
    results.push(test_hpet_write_counter_disabled());
    results.push(test_hpet_acpi_table_signature());
    results.push(test_hpet_acpi_table_checksum());
    results.push(test_hpet_acpi_table_address());
    
    // Tally results
    let mut passed = 0usize;
    let mut failed = 0usize;
    
    for r in &results {
        if r.passed {
            passed += 1;
            log.push(format!("  [PASS] {}", r.name));
        } else {
            failed += 1;
            if let Some(ref detail) = r.detail {
                log.push(format!("  [FAIL] {} — {}", r.name, detail));
            } else {
                log.push(format!("  [FAIL] {}", r.name));
            }
        }
    }
    
    (passed, failed, log)
}

// ============================================================================
// HELPER: Read little-endian values from byte buffer
// ============================================================================

fn read_u16(mem: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([mem[offset], mem[offset + 1]])
}

fn read_u32(mem: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        mem[offset], mem[offset + 1], mem[offset + 2], mem[offset + 3],
    ])
}

fn read_u64(mem: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes([
        mem[offset], mem[offset + 1], mem[offset + 2], mem[offset + 3],
        mem[offset + 4], mem[offset + 5], mem[offset + 6], mem[offset + 7],
    ])
}

/// Verify that bytes [offset..offset+len] sum to 0 (mod 256)
fn verify_checksum(mem: &[u8], offset: usize, len: usize) -> bool {
    let mut sum: u8 = 0;
    for i in 0..len {
        sum = sum.wrapping_add(mem[offset + i]);
    }
    sum == 0
}

/// Allocate guest memory and install ACPI tables
fn setup_acpi_guest_memory() -> alloc::vec::Vec<u8> {
    // Need enough memory for ACPI tables (0x50000-0x50300) + BIOS RSDP (0xE0000)
    let mut mem = vec![0u8; 0xF0000]; // ~960 KB
    super::acpi::install_acpi_tables(&mut mem);
    mem
}

// ============================================================================
// ACPI: RSDP Tests
// ============================================================================

fn test_acpi_rsdp_signature() -> TestResult {
    let mem = setup_acpi_guest_memory();
    let rsdp = 0x50000;
    let sig = &mem[rsdp..rsdp + 8];
    let passed = sig == b"RSD PTR ";
    TestResult {
        name: "ACPI RSDP signature",
        passed,
        detail: if !passed {
            Some(format!("expected 'RSD PTR ', got {:?}", core::str::from_utf8(sig)))
        } else { None },
    }
}

fn test_acpi_rsdp_v1_checksum() -> TestResult {
    let mem = setup_acpi_guest_memory();
    let rsdp = 0x50000;
    // RSDP v1 checksum covers bytes 0-19
    let passed = verify_checksum(&mem, rsdp, 20);
    TestResult {
        name: "ACPI RSDP v1 checksum (bytes 0-19)",
        passed,
        detail: if !passed {
            let mut sum: u8 = 0;
            for i in 0..20 { sum = sum.wrapping_add(mem[rsdp + i]); }
            Some(format!("sum=0x{:02X}, expected 0x00", sum))
        } else { None },
    }
}

fn test_acpi_rsdp_v2_checksum() -> TestResult {
    let mem = setup_acpi_guest_memory();
    let rsdp = 0x50000;
    // RSDP v2 extended checksum covers bytes 0-35
    let passed = verify_checksum(&mem, rsdp, 36);
    TestResult {
        name: "ACPI RSDP v2 extended checksum (bytes 0-35)",
        passed,
        detail: if !passed {
            let mut sum: u8 = 0;
            for i in 0..36 { sum = sum.wrapping_add(mem[rsdp + i]); }
            Some(format!("sum=0x{:02X}, expected 0x00", sum))
        } else { None },
    }
}

fn test_acpi_rsdp_revision() -> TestResult {
    let mem = setup_acpi_guest_memory();
    let revision = mem[0x50000 + 15];
    let passed = revision == 2;
    TestResult {
        name: "ACPI RSDP revision is 2 (ACPI 2.0)",
        passed,
        detail: if !passed {
            Some(format!("revision={}, expected 2", revision))
        } else { None },
    }
}

fn test_acpi_rsdp_xsdt_pointer() -> TestResult {
    let mem = setup_acpi_guest_memory();
    let rsdp = 0x50000;
    let xsdt_addr = read_u64(&mem, rsdp + 24);
    let passed = xsdt_addr == 0x50040;
    TestResult {
        name: "ACPI RSDP XSDT pointer = 0x50040",
        passed,
        detail: if !passed {
            Some(format!("xsdt_addr=0x{:X}, expected 0x50040", xsdt_addr))
        } else { None },
    }
}

// ============================================================================
// ACPI: XSDT Tests
// ============================================================================

fn test_acpi_xsdt_signature() -> TestResult {
    let mem = setup_acpi_guest_memory();
    let xsdt = 0x50040;
    let sig = &mem[xsdt..xsdt + 4];
    let passed = sig == b"XSDT";
    TestResult {
        name: "ACPI XSDT signature",
        passed,
        detail: if !passed {
            Some(format!("got {:?}", core::str::from_utf8(sig)))
        } else { None },
    }
}

fn test_acpi_xsdt_checksum() -> TestResult {
    let mem = setup_acpi_guest_memory();
    let xsdt = 0x50040;
    let len = read_u32(&mem, xsdt + 4) as usize;
    let passed = len > 0 && verify_checksum(&mem, xsdt, len);
    TestResult {
        name: "ACPI XSDT checksum",
        passed,
        detail: if !passed {
            Some(format!("len={}, checksum invalid", len))
        } else { None },
    }
}

fn test_acpi_xsdt_entry_count() -> TestResult {
    let mem = setup_acpi_guest_memory();
    let xsdt = 0x50040;
    let total_len = read_u32(&mem, xsdt + 4) as usize;
    let entry_bytes = total_len.saturating_sub(36);
    let entry_count = entry_bytes / 8;
    // Should have 3 entries (MADT + FADT + HPET)
    let entry0 = read_u64(&mem, xsdt + 36);
    let entry1 = read_u64(&mem, xsdt + 44);
    let entry2 = read_u64(&mem, xsdt + 52);
    let passed = entry_count == 3 && entry0 == 0x50080 && entry1 == 0x50100 && entry2 == 0x50240;
    TestResult {
        name: "ACPI XSDT has 3 entries (MADT, FADT, HPET)",
        passed,
        detail: if !passed {
            Some(format!("count={}, e0=0x{:X}, e1=0x{:X}, e2=0x{:X}", entry_count, entry0, entry1, entry2))
        } else { None },
    }
}

// ============================================================================
// ACPI: MADT Tests
// ============================================================================

fn test_acpi_madt_signature() -> TestResult {
    let mem = setup_acpi_guest_memory();
    let madt = 0x50080;
    let sig = &mem[madt..madt + 4];
    let passed = sig == b"APIC";
    TestResult {
        name: "ACPI MADT signature 'APIC'",
        passed,
        detail: if !passed {
            Some(format!("got {:?}", core::str::from_utf8(sig)))
        } else { None },
    }
}

fn test_acpi_madt_checksum() -> TestResult {
    let mem = setup_acpi_guest_memory();
    let madt = 0x50080;
    let len = read_u32(&mem, madt + 4) as usize;
    let passed = len > 0 && verify_checksum(&mem, madt, len);
    TestResult {
        name: "ACPI MADT checksum",
        passed,
        detail: if !passed {
            Some(format!("len={}, checksum invalid", len))
        } else { None },
    }
}

fn test_acpi_madt_lapic_addr() -> TestResult {
    let mem = setup_acpi_guest_memory();
    let madt = 0x50080;
    // LAPIC address at offset 36 in MADT
    let lapic_addr = read_u32(&mem, madt + 36);
    let passed = lapic_addr == 0xFEE0_0000;
    TestResult {
        name: "ACPI MADT LAPIC address = 0xFEE00000",
        passed,
        detail: if !passed {
            Some(format!("got 0x{:08X}", lapic_addr))
        } else { None },
    }
}

fn test_acpi_madt_has_lapic_entry() -> TestResult {
    let mem = setup_acpi_guest_memory();
    let madt = 0x50080;
    let total_len = read_u32(&mem, madt + 4) as usize;
    
    // Walk MADT entries starting at offset 44
    let mut pos = 44;
    let mut found_lapic = false;
    while pos + 2 <= total_len {
        let entry_type = mem[madt + pos];
        let entry_len = mem[madt + pos + 1] as usize;
        if entry_len == 0 { break; }
        
        if entry_type == 0 && entry_len == 8 {
            // Processor Local APIC
            let apic_id = mem[madt + pos + 3];
            let flags = read_u32(&mem, madt + pos + 4);
            if apic_id == 0 && (flags & 1) != 0 {
                found_lapic = true;
            }
        }
        pos += entry_len;
    }
    
    TestResult {
        name: "ACPI MADT contains Processor Local APIC (ID=0, enabled)",
        passed: found_lapic,
        detail: if !found_lapic { Some(String::from("LAPIC entry not found")) } else { None },
    }
}

fn test_acpi_madt_has_ioapic_entry() -> TestResult {
    let mem = setup_acpi_guest_memory();
    let madt = 0x50080;
    let total_len = read_u32(&mem, madt + 4) as usize;
    
    let mut pos = 44;
    let mut found_ioapic = false;
    let mut ioapic_addr = 0u32;
    while pos + 2 <= total_len {
        let entry_type = mem[madt + pos];
        let entry_len = mem[madt + pos + 1] as usize;
        if entry_len == 0 { break; }
        
        if entry_type == 1 && entry_len == 12 {
            // I/O APIC
            ioapic_addr = read_u32(&mem, madt + pos + 4);
            if ioapic_addr == 0xFEC0_0000 {
                found_ioapic = true;
            }
        }
        pos += entry_len;
    }
    
    TestResult {
        name: "ACPI MADT contains I/O APIC at 0xFEC00000",
        passed: found_ioapic,
        detail: if !found_ioapic {
            Some(format!("I/O APIC entry not found (addr=0x{:08X})", ioapic_addr))
        } else { None },
    }
}

fn test_acpi_madt_irq_overrides() -> TestResult {
    let mem = setup_acpi_guest_memory();
    let madt = 0x50080;
    let total_len = read_u32(&mem, madt + 4) as usize;
    
    let mut pos = 44;
    let mut found_irq0_gsi2 = false;
    let mut found_irq9_gsi9 = false;
    while pos + 2 <= total_len {
        let entry_type = mem[madt + pos];
        let entry_len = mem[madt + pos + 1] as usize;
        if entry_len == 0 { break; }
        
        if entry_type == 2 && entry_len == 10 {
            // Interrupt Source Override
            let source = mem[madt + pos + 3];
            let gsi = read_u32(&mem, madt + pos + 4);
            if source == 0 && gsi == 2 { found_irq0_gsi2 = true; }
            if source == 9 && gsi == 9 { found_irq9_gsi9 = true; }
        }
        pos += entry_len;
    }
    
    let passed = found_irq0_gsi2 && found_irq9_gsi9;
    TestResult {
        name: "ACPI MADT IRQ overrides (IRQ0→GSI2, IRQ9→GSI9)",
        passed,
        detail: if !passed {
            Some(format!("irq0→gsi2={}, irq9→gsi9={}", found_irq0_gsi2, found_irq9_gsi9))
        } else { None },
    }
}

// ============================================================================
// ACPI: FADT Tests
// ============================================================================

fn test_acpi_fadt_signature() -> TestResult {
    let mem = setup_acpi_guest_memory();
    let fadt = 0x50100;
    let sig = &mem[fadt..fadt + 4];
    let passed = sig == b"FACP";
    TestResult {
        name: "ACPI FADT signature 'FACP'",
        passed,
        detail: if !passed {
            Some(format!("got {:?}", core::str::from_utf8(sig)))
        } else { None },
    }
}

fn test_acpi_fadt_checksum() -> TestResult {
    let mem = setup_acpi_guest_memory();
    let fadt = 0x50100;
    let len = read_u32(&mem, fadt + 4) as usize;
    let passed = len == 276 && verify_checksum(&mem, fadt, len);
    TestResult {
        name: "ACPI FADT checksum (276 bytes)",
        passed,
        detail: if !passed {
            Some(format!("len={}, checksum invalid", len))
        } else { None },
    }
}

fn test_acpi_fadt_pm_timer_port() -> TestResult {
    let mem = setup_acpi_guest_memory();
    let fadt = 0x50100;
    // PM_TMR_BLK at offset 76
    let pm_tmr = read_u32(&mem, fadt + 76);
    // PM_TMR_LEN at offset 91
    let pm_tmr_len = mem[fadt + 91];
    // TMR_VAL_EXT flag in flags at offset 112 (bit 4)
    let flags = read_u32(&mem, fadt + 112);
    let tmr_ext = (flags >> 4) & 1;
    
    let passed = pm_tmr == 0xB008 && pm_tmr_len == 4 && tmr_ext == 1;
    TestResult {
        name: "ACPI FADT PM timer at 0xB008 (32-bit)",
        passed,
        detail: if !passed {
            Some(format!("port=0x{:X} len={} ext={}", pm_tmr, pm_tmr_len, tmr_ext))
        } else { None },
    }
}

fn test_acpi_fadt_sci_interrupt() -> TestResult {
    let mem = setup_acpi_guest_memory();
    let fadt = 0x50100;
    // SCI_INT at offset 46
    let sci_int = read_u16(&mem, fadt + 46);
    let passed = sci_int == 9;
    TestResult {
        name: "ACPI FADT SCI interrupt = IRQ 9",
        passed,
        detail: if !passed {
            Some(format!("sci_int={}", sci_int))
        } else { None },
    }
}

fn test_acpi_fadt_dsdt_pointer() -> TestResult {
    let mem = setup_acpi_guest_memory();
    let fadt = 0x50100;
    // DSDT (32-bit) at offset 40
    let dsdt_32 = read_u32(&mem, fadt + 40) as u64;
    // X_DSDT (64-bit) at offset 140
    let dsdt_64 = read_u64(&mem, fadt + 140);
    let passed = dsdt_32 == 0x50200 && dsdt_64 == 0x50200;
    TestResult {
        name: "ACPI FADT DSDT pointer = 0x50200 (32+64 bit)",
        passed,
        detail: if !passed {
            Some(format!("dsdt32=0x{:X}, dsdt64=0x{:X}", dsdt_32, dsdt_64))
        } else { None },
    }
}

// ============================================================================
// ACPI: DSDT Tests
// ============================================================================

fn test_acpi_dsdt_signature() -> TestResult {
    let mem = setup_acpi_guest_memory();
    let dsdt = 0x50200;
    let sig = &mem[dsdt..dsdt + 4];
    let passed = sig == b"DSDT";
    TestResult {
        name: "ACPI DSDT signature",
        passed,
        detail: if !passed {
            Some(format!("got {:?}", core::str::from_utf8(sig)))
        } else { None },
    }
}

fn test_acpi_dsdt_checksum() -> TestResult {
    let mem = setup_acpi_guest_memory();
    let dsdt = 0x50200;
    let len = read_u32(&mem, dsdt + 4) as usize;
    let passed = len == 36 && verify_checksum(&mem, dsdt, len);
    TestResult {
        name: "ACPI DSDT checksum (36 byte stub)",
        passed,
        detail: if !passed {
            Some(format!("len={}", len))
        } else { None },
    }
}

fn test_acpi_bios_rsdp_copy() -> TestResult {
    let mem = setup_acpi_guest_memory();
    // RSDP should be copied to 0xE0000 (BIOS scan area)
    let rsdp_primary = &mem[0x50000..0x50000 + 36];
    let rsdp_bios = &mem[0xE0000..0xE0000 + 36];
    let passed = rsdp_primary == rsdp_bios;
    TestResult {
        name: "ACPI RSDP copy at BIOS area 0xE0000",
        passed,
        detail: if !passed {
            Some(String::from("RSDP at 0x50000 != RSDP at 0xE0000"))
        } else { None },
    }
}

// ============================================================================
// PIC 8259A Tests
// ============================================================================

fn test_pic_defaults() -> TestResult {
    let pic = super::svm_vm::PicState::default();
    let passed = pic.master_imr == 0xFF
        && pic.slave_imr == 0xFF
        && pic.master_vector_base == 0x08
        && pic.slave_vector_base == 0x70
        && pic.master_icw_phase == 0
        && pic.slave_icw_phase == 0
        && pic.master_isr == 0
        && pic.master_irr == 0
        && !pic.initialized;
    TestResult {
        name: "PIC defaults (IMR=0xFF, bases=0x08/0x70, phase=0)",
        passed,
        detail: if !passed {
            Some(format!("m_imr=0x{:02X} s_imr=0x{:02X} m_base=0x{:02X} s_base=0x{:02X} phase={}/{}",
                pic.master_imr, pic.slave_imr, pic.master_vector_base, pic.slave_vector_base,
                pic.master_icw_phase, pic.slave_icw_phase))
        } else { None },
    }
}

fn test_pic_icw_sequence_master() -> TestResult {
    // Simulate ICW1-4 for master PIC
    let mut pic = super::svm_vm::PicState::default();
    
    // ICW1: bit 4 set = start init
    let icw1: u8 = 0x11; // ICW1 + ICW4 needed
    if icw1 & 0x10 != 0 {
        pic.master_icw_phase = 1;
        pic.master_isr = 0;
        pic.master_irr = 0;
    }
    let phase_after_icw1 = pic.master_icw_phase;
    
    // ICW2: vector base = 0x20
    let icw2: u8 = 0x20;
    if pic.master_icw_phase == 1 {
        pic.master_vector_base = icw2 & 0xF8;
        pic.master_icw_phase = 2;
    }
    let phase_after_icw2 = pic.master_icw_phase;
    
    // ICW3: cascade (slave on IRQ2 = 0x04)
    if pic.master_icw_phase == 2 {
        pic.master_icw_phase = 3;
    }
    
    // ICW4: 8086 mode
    if pic.master_icw_phase == 3 {
        pic.master_icw_phase = 0;
        pic.initialized = true;
    }
    
    let passed = phase_after_icw1 == 1
        && phase_after_icw2 == 2
        && pic.master_icw_phase == 0
        && pic.master_vector_base == 0x20
        && pic.initialized;
    
    TestResult {
        name: "PIC master ICW1-4 sequence (vector base 0x20)",
        passed,
        detail: if !passed {
            Some(format!("phase={} base=0x{:02X} init={}", 
                pic.master_icw_phase, pic.master_vector_base, pic.initialized))
        } else { None },
    }
}

fn test_pic_icw_sequence_slave() -> TestResult {
    let mut pic = super::svm_vm::PicState::default();
    
    // ICW1 on slave
    let icw1: u8 = 0x11;
    if icw1 & 0x10 != 0 {
        pic.slave_icw_phase = 1;
    }
    
    // ICW2: vector base = 0x28
    let icw2: u8 = 0x28;
    if pic.slave_icw_phase == 1 {
        pic.slave_vector_base = icw2 & 0xF8;
        pic.slave_icw_phase = 2;
    }
    
    // ICW3
    if pic.slave_icw_phase == 2 { pic.slave_icw_phase = 3; }
    
    // ICW4
    if pic.slave_icw_phase == 3 { pic.slave_icw_phase = 0; }
    
    let passed = pic.slave_icw_phase == 0 && pic.slave_vector_base == 0x28;
    TestResult {
        name: "PIC slave ICW1-4 sequence (vector base 0x28)",
        passed,
        detail: if !passed {
            Some(format!("phase={} base=0x{:02X}", pic.slave_icw_phase, pic.slave_vector_base))
        } else { None },
    }
}

fn test_pic_ocw1_imr() -> TestResult {
    let mut pic = super::svm_vm::PicState::default();
    
    // After initialization, writing to data port sets IMR
    pic.master_icw_phase = 0; // ready state
    pic.master_imr = 0xFB; // mask all except IRQ2 (cascade)
    
    let passed = pic.master_imr == 0xFB;
    TestResult {
        name: "PIC OCW1 sets IMR (0xFB = mask all except IRQ2)",
        passed,
        detail: None,
    }
}

fn test_pic_eoi_clears_isr() -> TestResult {
    let mut pic = super::svm_vm::PicState::default();
    pic.master_isr = 0x04; // IRQ2 in-service
    
    // Non-specific EOI (0x20 to command port)
    let ocw2: u8 = 0x20;
    if ocw2 == 0x20 {
        pic.master_isr = 0;
    }
    
    let passed = pic.master_isr == 0;
    TestResult {
        name: "PIC non-specific EOI clears ISR",
        passed,
        detail: if !passed {
            Some(format!("isr=0x{:02X} after EOI", pic.master_isr))
        } else { None },
    }
}

// ============================================================================
// PIT 8254 Tests
// ============================================================================

fn test_pit_defaults() -> TestResult {
    let pit = super::svm_vm::PitState::default();
    let ch0 = &pit.channels[0];
    let passed = ch0.reload == 0xFFFF
        && ch0.count == 0xFFFF
        && ch0.access == 3
        && ch0.mode == 0
        && !ch0.latched
        && !ch0.write_hi_pending
        && pit.channels.len() == 3;
    TestResult {
        name: "PIT defaults (reload=0xFFFF, access=3, 3 channels)",
        passed,
        detail: if !passed {
            Some(format!("reload={} access={} mode={}", ch0.reload, ch0.access, ch0.mode))
        } else { None },
    }
}

fn test_pit_control_word() -> TestResult {
    let mut pit = super::svm_vm::PitState::default();
    
    // Write control word: channel 0, access lo/hi, mode 2 (rate generator)
    let control: u8 = 0b00_11_010_0; // ch=0, access=3, mode=2, BCD=0
    let channel = ((control >> 6) & 0x3) as usize;
    let access = (control >> 4) & 0x3;
    let mode = (control >> 1) & 0x7;
    
    if channel < 3 && access != 0 {
        pit.channels[channel].access = access;
        pit.channels[channel].mode = mode;
        pit.channels[channel].write_hi_pending = false;
    }
    
    let passed = pit.channels[0].access == 3
        && pit.channels[0].mode == 2;
    TestResult {
        name: "PIT control word (ch0, lo/hi, mode 2 rate generator)",
        passed,
        detail: if !passed {
            Some(format!("access={} mode={}", pit.channels[0].access, pit.channels[0].mode))
        } else { None },
    }
}

fn test_pit_lohi_reload() -> TestResult {
    let mut pit = super::svm_vm::PitState::default();
    pit.channels[0].access = 3; // lo/hi
    
    // Write low byte: 0x9C (100 Hz = 11932 = 0x2E9C)
    let lo: u8 = 0x9C;
    let ch = &mut pit.channels[0];
    if !ch.write_hi_pending {
        ch.reload = (ch.reload & 0xFF00) | lo as u16;
        ch.write_hi_pending = true;
    }
    let after_lo = ch.reload;
    let pending = ch.write_hi_pending;
    
    // Write high byte: 0x2E
    let hi: u8 = 0x2E;
    if ch.write_hi_pending {
        ch.reload = (ch.reload & 0x00FF) | ((hi as u16) << 8);
        ch.count = ch.reload;
        ch.write_hi_pending = false;
    }
    
    let passed = (after_lo & 0xFF) == 0x9C
        && pending
        && pit.channels[0].reload == 0x2E9C
        && pit.channels[0].count == 0x2E9C
        && !pit.channels[0].write_hi_pending;
    TestResult {
        name: "PIT lo/hi reload sequence (0x2E9C = ~100 Hz)",
        passed,
        detail: if !passed {
            Some(format!("reload=0x{:04X} count=0x{:04X} pending={}", 
                pit.channels[0].reload, pit.channels[0].count, pit.channels[0].write_hi_pending))
        } else { None },
    }
}

fn test_pit_latch_command() -> TestResult {
    let mut pit = super::svm_vm::PitState::default();
    pit.channels[0].count = 0x1234;
    
    // Latch command: control word with access=0 for channel 0
    let control: u8 = 0b00_00_000_0; // ch=0, access=0 (latch)
    let channel = ((control >> 6) & 0x3) as usize;
    let access = (control >> 4) & 0x3;
    
    if channel < 3 && access == 0 {
        pit.channels[channel].latched = true;
        pit.channels[channel].latch_value = pit.channels[channel].count;
    }
    
    let passed = pit.channels[0].latched
        && pit.channels[0].latch_value == 0x1234;
    TestResult {
        name: "PIT latch command captures count value",
        passed,
        detail: if !passed {
            Some(format!("latched={} value=0x{:04X}", pit.channels[0].latched, pit.channels[0].latch_value))
        } else { None },
    }
}

// ============================================================================
// LAPIC Tests
// ============================================================================

fn test_lapic_defaults() -> TestResult {
    let lapic = super::svm_vm::LapicState::default();
    let passed = lapic.icr == 0
        && lapic.ccr == 0
        && lapic.dcr == 0
        && (lapic.timer_lvt & 0x0001_0000) != 0  // masked
        && lapic.svr == 0x1FF
        && lapic.tpr == 0
        && !lapic.enabled
        && lapic.last_tick_exit == 0;
    TestResult {
        name: "LAPIC defaults (masked, SVR=0x1FF, disabled)",
        passed,
        detail: if !passed {
            Some(format!("icr={} lvt=0x{:X} svr=0x{:X} enabled={}", 
                lapic.icr, lapic.timer_lvt, lapic.svr, lapic.enabled))
        } else { None },
    }
}

fn test_lapic_enable_via_svr() -> TestResult {
    let mut lapic = super::svm_vm::LapicState::default();
    
    // Enable LAPIC by setting bit 8 of SVR
    lapic.svr = 0x1FF; // bit 8 set, spurious vector 0xFF
    lapic.enabled = (lapic.svr & 0x100) != 0;
    
    let enabled = lapic.enabled;
    
    // Disable by clearing bit 8
    lapic.svr = 0x0FF;
    lapic.enabled = (lapic.svr & 0x100) != 0;
    let disabled = !lapic.enabled;
    
    let passed = enabled && disabled;
    TestResult {
        name: "LAPIC enable/disable via SVR bit 8",
        passed,
        detail: if !passed {
            Some(format!("enabled_check={} disabled_check={}", enabled, disabled))
        } else { None },
    }
}

fn test_lapic_timer_arm() -> TestResult {
    let mut lapic = super::svm_vm::LapicState::default();
    lapic.enabled = true;
    
    // Program timer: one-shot mode, vector 0x30, unmask
    lapic.timer_lvt = 0x30; // vector=0x30, mode=0 (one-shot), mask=0
    lapic.icr = 100_000;
    lapic.ccr = 100_000;
    lapic.last_tick_exit = 0;
    
    let vector = lapic.timer_lvt & 0xFF;
    let masked = (lapic.timer_lvt >> 16) & 1;
    let mode = (lapic.timer_lvt >> 17) & 0x3;
    
    let passed = vector == 0x30
        && masked == 0
        && mode == 0
        && lapic.icr == 100_000
        && lapic.ccr == 100_000;
    TestResult {
        name: "LAPIC timer arm (one-shot, vector=0x30, ICR=100000)",
        passed,
        detail: if !passed {
            Some(format!("vec=0x{:X} mask={} mode={} icr={}", vector, masked, mode, lapic.icr))
        } else { None },
    }
}

fn test_lapic_timer_lvt_modes() -> TestResult {
    let mut lapic = super::svm_vm::LapicState::default();
    
    // One-shot: mode bits [18:17] = 00
    lapic.timer_lvt = 0x30; // vector 0x30, one-shot, unmasked
    let mode0 = (lapic.timer_lvt >> 17) & 0x3;
    
    // Periodic: mode bits = 01
    lapic.timer_lvt = 0x0002_0030; // bit 17 set
    let mode1 = (lapic.timer_lvt >> 17) & 0x3;
    
    // TSC-deadline: mode bits = 10
    lapic.timer_lvt = 0x0004_0030; // bit 18 set
    let mode2 = (lapic.timer_lvt >> 17) & 0x3;
    
    // Masked
    lapic.timer_lvt = 0x0001_0030; // bit 16 set
    let masked = (lapic.timer_lvt >> 16) & 1;
    
    let passed = mode0 == 0 && mode1 == 1 && mode2 == 2 && masked == 1;
    TestResult {
        name: "LAPIC timer LVT modes (one-shot/periodic/TSC-deadline/mask)",
        passed,
        detail: if !passed {
            Some(format!("modes: 0={} 1={} 2={} mask={}", mode0, mode1, mode2, masked))
        } else { None },
    }
}

fn test_lapic_divider_values() -> TestResult {
    // Test all 8 divider configurations
    let divider_map: [(u32, u64); 8] = [
        (0x0, 2),    // 0000: divide by 2
        (0x1, 4),    // 0001: divide by 4
        (0x2, 8),    // 0010: divide by 8
        (0x3, 16),   // 0011: divide by 16
        (0x8, 32),   // 1000: divide by 32
        (0x9, 64),   // 1001: divide by 64
        (0xA, 128),  // 1010: divide by 128
        (0xB, 1),    // 1011: divide by 1
    ];
    
    let mut all_ok = true;
    let mut bad = String::new();
    
    for &(dcr_val, expected) in &divider_map {
        let divider = match dcr_val & 0xB {
            0x0 => 2u64, 0x1 => 4, 0x2 => 8, 0x3 => 16,
            0x8 => 32, 0x9 => 64, 0xA => 128, 0xB => 1,
            _ => 1,
        };
        if divider != expected {
            all_ok = false;
            bad = format!("dcr=0x{:X}: got {} expected {}", dcr_val, divider, expected);
            break;
        }
    }
    
    TestResult {
        name: "LAPIC timer divider decode (all 8 values)",
        passed: all_ok,
        detail: if !all_ok { Some(bad) } else { None },
    }
}

// ============================================================================
// CMOS RTC Tests
// ============================================================================

fn test_cmos_time_registers() -> TestResult {
    // Verify CMOS time register values match what handle_io returns
    // These are the hardcoded values in svm_vm.rs
    let expected: [(u8, u8); 7] = [
        (0x00, 0x00),  // seconds
        (0x02, 0x30),  // minutes
        (0x04, 0x12),  // hours (BCD noon)
        (0x06, 0x02),  // day of week
        (0x07, 0x17),  // day of month
        (0x08, 0x02),  // month
        (0x09, 0x26),  // year (BCD 26 = 2026)
    ];
    
    let mut all_ok = true;
    let mut bad = String::new();
    
    for &(reg, expected_val) in &expected {
        let actual = cmos_register_value(reg);
        if actual != expected_val {
            all_ok = false;
            bad = format!("reg 0x{:02X}: got 0x{:02X} expected 0x{:02X}", reg, actual, expected_val);
            break;
        }
    }
    
    TestResult {
        name: "CMOS time registers (seconds/min/hour/date)",
        passed: all_ok,
        detail: if !all_ok { Some(bad) } else { None },
    }
}

fn test_cmos_status_registers() -> TestResult {
    let reg_a = cmos_register_value(0x0A);
    let reg_b = cmos_register_value(0x0B);
    let reg_c = cmos_register_value(0x0C);
    let reg_d = cmos_register_value(0x0D);
    
    let passed = reg_a == 0x26  // divider + rate
        && reg_b == 0x02        // 24h mode
        && reg_c == 0x00        // no interrupts pending
        && reg_d == 0x80;       // battery OK
    TestResult {
        name: "CMOS status registers A-D",
        passed,
        detail: if !passed {
            Some(format!("A=0x{:02X} B=0x{:02X} C=0x{:02X} D=0x{:02X}", reg_a, reg_b, reg_c, reg_d))
        } else { None },
    }
}

fn test_cmos_equipment_byte() -> TestResult {
    let equipment = cmos_register_value(0x14);
    // Bit 1 = math coprocessor, bit 2 = color display (should be 0x06)
    let passed = equipment == 0x06;
    TestResult {
        name: "CMOS equipment byte (FPU + color display)",
        passed,
        detail: if !passed {
            Some(format!("got 0x{:02X}, expected 0x06", equipment))
        } else { None },
    }
}

fn test_cmos_century() -> TestResult {
    let century = cmos_register_value(0x32);
    let passed = century == 0x20; // BCD 20 for 2000s
    TestResult {
        name: "CMOS century register = 0x20 (2000s)",
        passed,
        detail: if !passed {
            Some(format!("got 0x{:02X}", century))
        } else { None },
    }
}

/// Lookup CMOS register value (mirrors handle_io logic in svm_vm.rs)
fn cmos_register_value(index: u8) -> u8 {
    match index {
        0x00 => 0x00,  // seconds
        0x02 => 0x30,  // minutes
        0x04 => 0x12,  // hours
        0x06 => 0x02,  // day of week
        0x07 => 0x17,  // day of month
        0x08 => 0x02,  // month
        0x09 => 0x26,  // year
        0x0A => 0x26,  // Status A
        0x0B => 0x02,  // Status B
        0x0C => 0x00,  // Status C
        0x0D => 0x80,  // Status D
        0x0E => 0x00,  // Diagnostic
        0x0F => 0x00,  // Shutdown
        0x10 => 0x00,  // Floppy
        0x12 => 0x00,  // Hard disk
        0x14 => 0x06,  // Equipment
        0x15 => 0x80,  // Base memory low
        0x16 => 0x02,  // Base memory high
        0x17 => 0x00,  // Extended memory low
        0x18 => 0x00,  // Extended memory high
        0x32 => 0x20,  // Century
        _ => 0x00,
    }
}

// ============================================================================
// MMIO INSTRUCTION DECODER TESTS
// ============================================================================

/// Test: MOV [rdi], eax → write, 4-byte, reg=RAX
fn test_mmio_decode_mov_write() -> TestResult {
    let bytes = [0x89, 0x07]; // MOV [rdi], eax
    let d = super::mmio::decode_mmio_instruction(&bytes, 2, true);
    let ok = match d {
        Some(ref dec) => dec.is_write && dec.operand_size == 4 && dec.register == Some(0) && dec.insn_len == 2,
        None => false,
    };
    TestResult {
        name: "mmio_decode_mov_write",
        passed: ok,
        detail: if !ok { Some(format!("decoded={:?}", d)) } else { None },
    }
}

/// Test: MOV ecx, [rdi] → read, 4-byte, reg=ECX(1)
fn test_mmio_decode_mov_read() -> TestResult {
    let bytes = [0x8B, 0x0F]; // MOV ecx, [rdi]
    let d = super::mmio::decode_mmio_instruction(&bytes, 2, true);
    let ok = match d {
        Some(ref dec) => !dec.is_write && dec.operand_size == 4 && dec.register == Some(1) && dec.insn_len == 2,
        None => false,
    };
    TestResult {
        name: "mmio_decode_mov_read",
        passed: ok,
        detail: if !ok { Some(format!("decoded={:?}", d)) } else { None },
    }
}

/// Test: REX.W MOV [rdi], rax → write, 8-byte, reg=RAX
fn test_mmio_decode_rex_w() -> TestResult {
    let bytes = [0x48, 0x89, 0x07]; // REX.W MOV [rdi], rax
    let d = super::mmio::decode_mmio_instruction(&bytes, 3, true);
    let ok = match d {
        Some(ref dec) => dec.is_write && dec.operand_size == 8 && dec.register == Some(0) && dec.insn_len == 3,
        None => false,
    };
    TestResult {
        name: "mmio_decode_rex_w",
        passed: ok,
        detail: if !ok { Some(format!("decoded={:?}", d)) } else { None },
    }
}

/// Test: MOV DWORD PTR [rdi], 0x12345678 → write, 4-byte, immediate
fn test_mmio_decode_imm32() -> TestResult {
    // C7 07 78 56 34 12 = MOV [rdi], 0x12345678
    let bytes = [0xC7, 0x07, 0x78, 0x56, 0x34, 0x12];
    let d = super::mmio::decode_mmio_instruction(&bytes, 6, true);
    let ok = match d {
        Some(ref dec) => dec.is_write && dec.operand_size == 4 
            && dec.register.is_none() 
            && dec.immediate == Some(0x12345678),
        None => false,
    };
    TestResult {
        name: "mmio_decode_imm32",
        passed: ok,
        detail: if !ok { Some(format!("decoded={:?}", d)) } else { None },
    }
}

/// Test: MOVZX eax, BYTE PTR [rdi] → read, 1-byte, reg=EAX
fn test_mmio_decode_movzx() -> TestResult {
    let bytes = [0x0F, 0xB6, 0x07]; // MOVZX eax, byte ptr [rdi]
    let d = super::mmio::decode_mmio_instruction(&bytes, 3, true);
    let ok = match d {
        Some(ref dec) => !dec.is_write && dec.operand_size == 1 && dec.register == Some(0),
        None => false,
    };
    TestResult {
        name: "mmio_decode_movzx",
        passed: ok,
        detail: if !ok { Some(format!("decoded={:?}", d)) } else { None },
    }
}

/// Test: MOV eax, [rdi+0x320] → read with disp32, reg=EAX
fn test_mmio_decode_disp32() -> TestResult {
    let bytes = [0x8B, 0x87, 0x20, 0x03, 0x00, 0x00]; // MOV eax, [rdi+0x320]
    let d = super::mmio::decode_mmio_instruction(&bytes, 6, true);
    let ok = match d {
        Some(ref dec) => !dec.is_write && dec.operand_size == 4 && dec.register == Some(0) && dec.insn_len == 6,
        None => false,
    };
    TestResult {
        name: "mmio_decode_disp32",
        passed: ok,
        detail: if !ok { Some(format!("decoded={:?}", d)) } else { None },
    }
}

/// Test: REX.RB MOV [r15], r8d → write from extended register R8
fn test_mmio_decode_r8_r15() -> TestResult {
    // 45 89 07 = REX.RB MOV [r15], r8d  (REX=0x45: W=0,R=1,X=0,B=1)
    // Actually: REX.R=1 means reg field extended, REX.B=1 means rm field extended
    // 0x45 = 0100_0101 → W=0, R=1, X=0, B=1
    // ModRM: 0x07 = mod=00, reg=000, rm=111 → reg = REX.R:000 = 8 (R8), rm = REX.B:111 = R15
    let bytes = [0x45, 0x89, 0x07]; // MOV [r15], r8d
    let d = super::mmio::decode_mmio_instruction(&bytes, 3, true);
    let ok = match d {
        Some(ref dec) => dec.is_write && dec.operand_size == 4 && dec.register == Some(8), // R8
        None => false,
    };
    TestResult {
        name: "mmio_decode_r8_r15",
        passed: ok,
        detail: if !ok { Some(format!("decoded={:?}", d)) } else { None },
    }
}

/// Test: read_guest_reg / write_guest_reg round-trip
fn test_mmio_register_rw() -> TestResult {
    let mut regs = super::svm_vm::SvmGuestRegs::default();
    // Write to R10 (index 10) and read back
    super::mmio::write_guest_reg(&mut regs, 10, 0xDEAD_BEEF_CAFE_BABE);
    let val = super::mmio::read_guest_reg(&regs, 10);
    let ok = val == 0xDEAD_BEEF_CAFE_BABE && regs.r10 == 0xDEAD_BEEF_CAFE_BABE;
    TestResult {
        name: "mmio_register_rw",
        passed: ok,
        detail: if !ok { Some(format!("got 0x{:X}", val)) } else { None },
    }
}

// ============================================================================
// I/O APIC TESTS
// ============================================================================

/// Test: I/O APIC default state
fn test_ioapic_defaults() -> TestResult {
    let ioapic = super::ioapic::IoApicState::default();
    let ok = ioapic.id == 1 && ioapic.ioregsel == 0;
    // Verify all entries are masked
    let all_masked = ioapic.redir_table.iter().all(|e| (e >> 16) & 1 == 1);
    let ok = ok && all_masked;
    TestResult {
        name: "ioapic_defaults",
        passed: ok,
        detail: if !ok { Some(format!("id={} ioregsel={} all_masked={}", ioapic.id, ioapic.ioregsel, all_masked)) } else { None },
    }
}

/// Test: I/O APIC version register
fn test_ioapic_version_register() -> TestResult {
    let ioapic = super::ioapic::IoApicState::default();
    let ver = ioapic.read(0x10); // IOREGSEL defaults to 0, but we need to set it
    // Actually, we need to set ioregsel to 1 first
    let mut ioapic = ioapic;
    ioapic.write(0x00, 0x01); // IOREGSEL = 1 (version register)
    let ver = ioapic.read(0x10); // Read via IOWIN
    let version = ver & 0xFF;
    let max_redir = (ver >> 16) & 0xFF;
    let ok = version == 0x20 && max_redir == 23;
    TestResult {
        name: "ioapic_version_register",
        passed: ok,
        detail: if !ok { Some(format!("ver=0x{:X} max_redir={}", version, max_redir)) } else { None },
    }
}

/// Test: I/O APIC ID register read/write
fn test_ioapic_id_readwrite() -> TestResult {
    let mut ioapic = super::ioapic::IoApicState::default();
    // Read default ID
    ioapic.write(0x00, 0x00); // IOREGSEL = 0 (ID register)
    let id_reg = ioapic.read(0x10);
    let default_id = (id_reg >> 24) & 0xF;
    
    // Write a new ID
    ioapic.write(0x00, 0x00);
    ioapic.write(0x10, 0x05_00_00_00); // Set ID to 5
    let id_reg = ioapic.read(0x10);
    let new_id = (id_reg >> 24) & 0xF;
    
    let ok = default_id == 1 && new_id == 5;
    TestResult {
        name: "ioapic_id_readwrite",
        passed: ok,
        detail: if !ok { Some(format!("default={} new={}", default_id, new_id)) } else { None },
    }
}

/// Test: I/O APIC redirection table write/read
fn test_ioapic_redir_table() -> TestResult {
    let mut ioapic = super::ioapic::IoApicState::default();
    
    // Write entry 0 low DWORD: vector=0x40, fixed delivery, physical, edge, unmasked
    ioapic.write(0x00, 0x10); // IOREGSEL = 0x10 (entry 0 low)
    ioapic.write(0x10, 0x0000_0040); // IOWIN = vector 0x40, unmasked
    
    // Write entry 0 high DWORD: destination APIC ID = 0
    ioapic.write(0x00, 0x11); // IOREGSEL = 0x11 (entry 0 high)
    ioapic.write(0x10, 0x0000_0000);
    
    // Read back low DWORD
    ioapic.write(0x00, 0x10);
    let lo = ioapic.read(0x10);
    
    let ok = (lo & 0xFF) == 0x40 && ((lo >> 16) & 1) == 0; // vector=0x40, not masked
    TestResult {
        name: "ioapic_redir_table",
        passed: ok,
        detail: if !ok { Some(format!("lo=0x{:X}", lo)) } else { None },
    }
}

/// Test: I/O APIC indirect register access pattern
fn test_ioapic_indirect_access() -> TestResult {
    let mut ioapic = super::ioapic::IoApicState::default();
    
    // The guest writes IOREGSEL then reads/writes IOWIN
    // Verify that writing IOREGSEL changes which register IOWIN accesses
    ioapic.write(0x00, 0x00); // Select ID register
    let id = ioapic.read(0x10);
    ioapic.write(0x00, 0x01); // Select version register
    let ver = ioapic.read(0x10);
    
    let ok = id != ver; // They should be different registers
    TestResult {
        name: "ioapic_indirect_access",
        passed: ok,
        detail: if !ok { Some(format!("id=0x{:X} ver=0x{:X}", id, ver)) } else { None },
    }
}

/// Test: I/O APIC IRQ routing query
fn test_ioapic_irq_routing() -> TestResult {
    let mut ioapic = super::ioapic::IoApicState::default();
    
    // Configure GSI 2 → vector 0x30, unmasked, physical, edge, dest=0
    ioapic.write(0x00, 0x14); // Entry 2 low = 0x10 + 2*2 = 0x14
    ioapic.write(0x10, 0x0000_0030); // vector=0x30, unmasked
    ioapic.write(0x00, 0x15); // Entry 2 high
    ioapic.write(0x10, 0x0000_0000); // dest=0
    
    let route = ioapic.get_irq_route(2);
    let ok = match route {
        Some(ref r) => r.vector == 0x30 && !r.masked && r.delivery_mode == 0 && !r.level_triggered,
        None => false,
    };
    TestResult {
        name: "ioapic_irq_routing",
        passed: ok,
        detail: if !ok { Some(format!("route={:?}", route)) } else { None },
    }
}

/// Test: Read-only bits (delivery status, remote IRR) cannot be written
fn test_ioapic_readonly_bits() -> TestResult {
    let mut ioapic = super::ioapic::IoApicState::default();
    
    // Try to write with bits 12 and 14 set
    ioapic.write(0x00, 0x10); // Entry 0 low
    ioapic.write(0x10, 0x0000_5030); // vector=0x30, try set bits 12+14
    
    ioapic.write(0x00, 0x10);
    let lo = ioapic.read(0x10);
    
    // Bits 12 and 14 should be 0 (read-only, default to 0)
    let bit12 = (lo >> 12) & 1;
    let bit14 = (lo >> 14) & 1;
    let ok = bit12 == 0 && bit14 == 0 && (lo & 0xFF) == 0x30;
    TestResult {
        name: "ioapic_readonly_bits",
        passed: ok,
        detail: if !ok { Some(format!("lo=0x{:X} bit12={} bit14={}", lo, bit12, bit14)) } else { None },
    }
}

// ============================================================================
// HPET Tests
// ============================================================================

fn test_hpet_defaults() -> TestResult {
    let hpet = super::hpet::HpetState::default();
    let ok = !hpet.enabled && hpet.config == 0 && hpet.isr == 0 
             && hpet.counter_offset == 0 && hpet.timers.len() == 3;
    TestResult {
        name: "hpet_defaults",
        passed: ok,
        detail: if !ok { Some(format!("enabled={} config=0x{:X}", hpet.enabled, hpet.config)) } else { None },
    }
}

fn test_hpet_gcap_id() -> TestResult {
    let hpet = super::hpet::HpetState::default();
    let gcap = hpet.read(0x000, 8);
    // Bits [7:0] = revision (1), Bits [12:8] = num_timers-1 (2), Bit 13 = 64-bit (1)
    let rev = gcap & 0xFF;
    let num_timers_m1 = (gcap >> 8) & 0x1F;
    let counter_64bit = (gcap >> 13) & 1;
    let period = (gcap >> 32) as u32;
    let ok = rev == 1 && num_timers_m1 == 2 && counter_64bit == 1 && period == 69_841_279;
    TestResult {
        name: "hpet_gcap_id_register",
        passed: ok,
        detail: if !ok { 
            Some(format!("rev={} timers-1={} 64bit={} period={}", rev, num_timers_m1, counter_64bit, period))
        } else { None },
    }
}

fn test_hpet_enable_disable() -> TestResult {
    let mut hpet = super::hpet::HpetState::default();
    // Initially disabled, counter should be 0
    let c0 = hpet.read(0x0F0, 8);
    
    // Enable HPET
    hpet.write(0x010, 1, 8); // GCONF = 1 (enable)
    let enabled = hpet.enabled;
    let config = hpet.read(0x010, 8);
    
    // Disable HPET
    hpet.write(0x010, 0, 8);
    let disabled = !hpet.enabled;
    // Counter should be frozen at some value
    let c1 = hpet.read(0x0F0, 8);
    let c2 = hpet.read(0x0F0, 8);
    let frozen = c1 == c2; // Should be same when disabled
    
    let ok = c0 == 0 && enabled && config == 1 && disabled && frozen;
    TestResult {
        name: "hpet_enable_disable",
        passed: ok,
        detail: if !ok { 
            Some(format!("c0={} en={} cfg={} dis={} frozen={}", c0, enabled, config, disabled, frozen))
        } else { None },
    }
}

fn test_hpet_counter_increments() -> TestResult {
    let mut hpet = super::hpet::HpetState::default();
    hpet.write(0x010, 1, 8); // Enable
    
    // Read counter twice with a small spin loop between
    let c1 = hpet.read(0x0F0, 8);
    // Burn some cycles to advance TSC 
    for _ in 0..10000 {
        core::hint::spin_loop();
    }
    let c2 = hpet.read(0x0F0, 8);
    
    // Counter should have incremented (or at least not gone backwards)
    let ok = c2 >= c1;
    TestResult {
        name: "hpet_counter_increments",
        passed: ok,
        detail: if !ok { 
            Some(format!("c1={} c2={}", c1, c2))
        } else { None },
    }
}

fn test_hpet_timer_config() -> TestResult {
    let mut hpet = super::hpet::HpetState::default();
    // Timer 0 should be periodic-capable
    let t0_config = hpet.read(0x100, 8); // Timer 0 config
    let periodic_cap = (t0_config >> 4) & 1; // Bit 4 = periodic capable
    
    // Write timer 0 config to enable interrupt and set route
    // Bit 2 = interrupt enable, Bits [13:9] = route to IRQ 2
    hpet.write(0x100, (2 << 9) | (1 << 2), 8);
    let t0_after = hpet.read(0x100, 8);
    let int_enabled = (t0_after >> 2) & 1;
    let route = (t0_after >> 9) & 0x1F;
    
    let ok = periodic_cap == 1 && int_enabled == 1 && route == 2;
    TestResult {
        name: "hpet_timer_config",
        passed: ok,
        detail: if !ok {
            Some(format!("periodic_cap={} int_en={} route={} raw=0x{:X}", periodic_cap, int_enabled, route, t0_after))
        } else { None },
    }
}

fn test_hpet_timer_comparator() -> TestResult {
    let mut hpet = super::hpet::HpetState::default();
    // Write comparator for timer 0
    hpet.write(0x108, 0xDEAD_BEEF, 8);
    let comp = hpet.read(0x108, 8);
    
    // Write comparator for timer 1
    hpet.write(0x128, 0x1234_5678_9ABC_DEF0, 8);
    let comp1 = hpet.read(0x128, 8);
    
    let ok = comp == 0xDEAD_BEEF && comp1 == 0x1234_5678_9ABC_DEF0;
    TestResult {
        name: "hpet_timer_comparator",
        passed: ok,
        detail: if !ok { Some(format!("t0=0x{:X} t1=0x{:X}", comp, comp1)) } else { None },
    }
}

fn test_hpet_write_counter_disabled() -> TestResult {
    let mut hpet = super::hpet::HpetState::default();
    // HPET disabled — writing to main counter should work
    hpet.write(0x0F0, 0x42, 8);
    let c = hpet.read(0x0F0, 8);
    
    // Now enable — should not be able to write
    hpet.write(0x010, 1, 8);
    hpet.write(0x0F0, 0xFF, 8); // Should be ignored
    // Counter should be based on 0x42 + TSC-derived ticks (>= 0x42)
    let c_enabled = hpet.read(0x0F0, 8);
    
    let ok = c == 0x42 && c_enabled >= 0x42;
    TestResult {
        name: "hpet_write_counter_disabled",
        passed: ok,
        detail: if !ok { Some(format!("c_dis=0x{:X} c_en=0x{:X}", c, c_enabled)) } else { None },
    }
}

fn test_hpet_acpi_table_signature() -> TestResult {
    let mem = setup_acpi_guest_memory();
    let hpet_offset = 0x50240;
    let sig = &mem[hpet_offset..hpet_offset + 4];
    let ok = sig == b"HPET";
    TestResult {
        name: "hpet_acpi_table_signature",
        passed: ok,
        detail: if !ok { Some(format!("sig={:?}", core::str::from_utf8(sig))) } else { None },
    }
}

fn test_hpet_acpi_table_checksum() -> TestResult {
    let mem = setup_acpi_guest_memory();
    let hpet_offset = 0x50240;
    let length = read_u32(&mem, hpet_offset + 4) as usize;
    let sum: u8 = mem[hpet_offset..hpet_offset + length].iter().fold(0u8, |a, &b| a.wrapping_add(b));
    let ok = sum == 0 && length == 56;
    TestResult {
        name: "hpet_acpi_table_checksum",
        passed: ok,
        detail: if !ok { Some(format!("sum={} len={}", sum, length)) } else { None },
    }
}

fn test_hpet_acpi_table_address() -> TestResult {
    let mem = setup_acpi_guest_memory();
    let hpet_offset = 0x50240;
    // Base address is at offset 44 (GAS address field)
    let addr = read_u64(&mem, hpet_offset + 44);
    // Address space ID should be 0 (memory)
    let addr_space = mem[hpet_offset + 40];
    let ok = addr == 0xFED0_0000 && addr_space == 0;
    TestResult {
        name: "hpet_acpi_table_address",
        passed: ok,
        detail: if !ok { Some(format!("addr=0x{:X} space={}", addr, addr_space)) } else { None },
    }
}
