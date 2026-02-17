//! Minimal ACPI Table Generator for TrustVM
//!
//! Generates the minimum ACPI tables required for Linux to discover
//! interrupt controllers and boot successfully:
//!
//! - RSDP (Root System Description Pointer) — at EBDA or BIOS ROM area
//! - XSDT (Extended System Description Table) — points to MADT + FADT
//! - MADT (Multiple APIC Description Table) — describes LAPIC + I/O APIC
//! - FADT (Fixed ACPI Description Table) — minimal, tells Linux about PM
//! - DSDT (Differentiated System Description Table) — empty AML stub
//!
//! All tables use ACPI revision 2 (64-bit pointers via XSDT).
//!
//! References:
//! - ACPI Specification 6.4, Chapter 5 (ACPI Software Programming Model)
//! - Linux source: arch/x86/boot/compressed/acpi.c, drivers/acpi/

/// Guest physical address where ACPI tables are placed
/// Sits between cmdline (0x20000) and GDT (0x60000)
const ACPI_TABLES_ADDR: u64 = 0x50000;

/// RSDP is also placed in the BIOS ROM scan area so Linux firmware scan finds it
const RSDP_BIOS_ADDR: u64 = 0xE0000;

/// RSDP size (ACPI 2.0 = 36 bytes)
const RSDP_SIZE: usize = 36;

/// Standard LAPIC base address
const LAPIC_PHYS_ADDR: u32 = 0xFEE0_0000;

/// Standard I/O APIC base address  
const IOAPIC_PHYS_ADDR: u32 = 0xFEC0_0000;

/// I/O APIC ID
const IOAPIC_ID: u8 = 1;

/// ACPI OEM ID (padded to 6 bytes)
const OEM_ID: &[u8; 6] = b"TRUST\0";

/// ACPI OEM Table ID (padded to 8 bytes)
const OEM_TABLE_ID: &[u8; 8] = b"TRUSTVM\0";

// ============================================================================
// PUBLIC API
// ============================================================================

/// Install minimal ACPI tables into guest memory.
///
/// Returns the physical address of the RSDP (for boot_params.acpi_rsdp_addr).
///
/// Memory layout at ACPI_TABLES_ADDR:
///   +0x000: RSDP (36 bytes)
///   +0x040: XSDT (36 + 8*2 = 52 bytes) — points to MADT and FADT
///   +0x080: MADT (44 + 8 + 12 = 64 bytes) — LAPIC #0 + I/O APIC
///   +0x100: FADT (276 bytes) — minimal fixed description table
///   +0x200: DSDT (36 bytes) — empty AML definition block
pub fn install_acpi_tables(guest_memory: &mut [u8]) -> u64 {
    let base = ACPI_TABLES_ADDR as usize;
    
    // Ensure the area is zeroed
    let acpi_size = 0x300; // 768 bytes is plenty
    for i in 0..acpi_size {
        if base + i < guest_memory.len() {
            guest_memory[base + i] = 0;
        }
    }
    
    let rsdp_addr = base;           // 0x50000
    let xsdt_addr = base + 0x040;   // 0x50040
    let madt_addr = base + 0x080;   // 0x50080
    let fadt_addr = base + 0x100;   // 0x50100
    let dsdt_addr = base + 0x200;   // 0x50200
    
    // 1. Build DSDT (empty AML stub — just a header)
    let dsdt_len = build_dsdt(guest_memory, dsdt_addr);
    
    // 2. Build MADT
    let madt_len = build_madt(guest_memory, madt_addr);
    
    // 3. Build FADT (points to DSDT)
    let fadt_len = build_fadt(guest_memory, fadt_addr, dsdt_addr as u64);
    
    // 4. Build XSDT (points to MADT and FADT)
    let xsdt_len = build_xsdt(guest_memory, xsdt_addr, &[
        madt_addr as u64,
        fadt_addr as u64,
    ]);
    
    // 5. Build RSDP (points to XSDT)
    build_rsdp(guest_memory, rsdp_addr, xsdt_addr as u64);
    
    // 6. Also copy RSDP to BIOS ROM scan area (0xE0000) for firmware-based discovery
    // Linux scans 0xE0000-0xFFFFF in 16-byte increments looking for "RSD PTR "
    let bios_rsdp = RSDP_BIOS_ADDR as usize;
    if bios_rsdp + RSDP_SIZE <= guest_memory.len() {
        // Use temp buffer to avoid overlapping borrow
        let mut rsdp_copy = [0u8; 36];
        rsdp_copy.copy_from_slice(&guest_memory[rsdp_addr..rsdp_addr + RSDP_SIZE]);
        guest_memory[bios_rsdp..bios_rsdp + RSDP_SIZE].copy_from_slice(&rsdp_copy);
    }
    
    crate::serial_println!("[ACPI] Tables installed at GPA 0x{:X}:", ACPI_TABLES_ADDR);
    crate::serial_println!("[ACPI]   RSDP: 0x{:X} (also at 0x{:X})", rsdp_addr, RSDP_BIOS_ADDR);
    crate::serial_println!("[ACPI]   XSDT: 0x{:X} ({} bytes, {} entries)", xsdt_addr, xsdt_len, 2);
    crate::serial_println!("[ACPI]   MADT: 0x{:X} ({} bytes)", madt_addr, madt_len);
    crate::serial_println!("[ACPI]   FADT: 0x{:X} ({} bytes)", fadt_addr, fadt_len);
    crate::serial_println!("[ACPI]   DSDT: 0x{:X} ({} bytes)", dsdt_addr, dsdt_len);
    
    // Return RSDP address (for boot_params)
    ACPI_TABLES_ADDR
}

// ============================================================================
// RSDP — Root System Description Pointer (ACPI 2.0)
// ============================================================================

/// Build RSDP v2 (36 bytes) at the given offset.
///
/// Layout:
///   [0..8]   Signature "RSD PTR "
///   [8]      Checksum (v1 portion, bytes 0-19)
///   [9..15]  OEM ID
///   [15]     Revision (2 for ACPI 2.0+)
///   [16..20] RSDT Address (0 — we use XSDT)
///   [20..24] Length (36 for v2)
///   [24..32] XSDT Address (64-bit)
///   [32]     Extended checksum (bytes 0-35)
///   [33..36] Reserved
fn build_rsdp(mem: &mut [u8], offset: usize, xsdt_phys: u64) {
    // Signature
    mem[offset..offset + 8].copy_from_slice(b"RSD PTR ");
    
    // OEM ID
    mem[offset + 9..offset + 15].copy_from_slice(OEM_ID);
    
    // Revision = 2 (ACPI 2.0)
    mem[offset + 15] = 2;
    
    // RSDT Address = 0 (we only use XSDT)
    write_u32(mem, offset + 16, 0);
    
    // Length = 36 (RSDP v2)
    write_u32(mem, offset + 20, 36);
    
    // XSDT Address (64-bit)
    write_u64(mem, offset + 24, xsdt_phys);
    
    // Compute v1 checksum (bytes 0-19)
    mem[offset + 8] = 0; // clear checksum field first
    let mut sum: u8 = 0;
    for i in 0..20 {
        sum = sum.wrapping_add(mem[offset + i]);
    }
    mem[offset + 8] = 0u8.wrapping_sub(sum);
    
    // Compute extended checksum (bytes 0-35)
    mem[offset + 32] = 0; // clear extended checksum field first
    let mut sum2: u8 = 0;
    for i in 0..36 {
        sum2 = sum2.wrapping_add(mem[offset + i]);
    }
    mem[offset + 32] = 0u8.wrapping_sub(sum2);
}

// ============================================================================
// XSDT — Extended System Description Table
// ============================================================================

/// Build XSDT at the given offset.
/// Header (36 bytes) + array of 64-bit physical pointers to other tables.
fn build_xsdt(mem: &mut [u8], offset: usize, table_addrs: &[u64]) -> usize {
    let entry_count = table_addrs.len();
    let total_len = 36 + entry_count * 8;
    
    // Signature
    mem[offset..offset + 4].copy_from_slice(b"XSDT");
    
    // Length
    write_u32(mem, offset + 4, total_len as u32);
    
    // Revision
    mem[offset + 8] = 1;
    
    // Checksum (will fill after)
    mem[offset + 9] = 0;
    
    // OEM ID
    mem[offset + 10..offset + 16].copy_from_slice(OEM_ID);
    
    // OEM Table ID
    mem[offset + 16..offset + 24].copy_from_slice(OEM_TABLE_ID);
    
    // OEM Revision
    write_u32(mem, offset + 24, 1);
    
    // Creator ID
    mem[offset + 28..offset + 32].copy_from_slice(b"TROS");
    
    // Creator Revision
    write_u32(mem, offset + 32, 1);
    
    // Entry array (64-bit pointers)
    for (i, &addr) in table_addrs.iter().enumerate() {
        write_u64(mem, offset + 36 + i * 8, addr);
    }
    
    // Compute checksum
    fixup_checksum(mem, offset, total_len);
    
    total_len
}

// ============================================================================
// MADT — Multiple APIC Description Table
// ============================================================================

/// Build MADT at the given offset.
///
/// Contains:
///   - MADT header (44 bytes) — includes LAPIC address + flags
///   - Processor Local APIC entry (8 bytes) — CPU 0
///   - I/O APIC entry (12 bytes) — I/O APIC 0
///   - Interrupt Source Override for IRQ0→GSI2 (10 bytes) — standard PC remap
///   - Interrupt Source Override for IRQ9→GSI9 (10 bytes) — ACPI SCI
fn build_madt(mem: &mut [u8], offset: usize) -> usize {
    // MADT has a standard ACPI header (36 bytes) + 
    // 4 bytes LAPIC address + 4 bytes flags = 44 byte header total
    
    let mut pos = offset;
    
    // === Standard ACPI table header ===
    mem[pos..pos + 4].copy_from_slice(b"APIC"); // Signature
    // Length — filled later
    pos += 4;
    let len_offset = pos;
    pos += 4; // skip length for now
    
    mem[offset + 8] = 4; // Revision (MADT rev 4 for ACPI 6.0)
    mem[offset + 9] = 0; // Checksum — filled later
    mem[offset + 10..offset + 16].copy_from_slice(OEM_ID);
    mem[offset + 16..offset + 24].copy_from_slice(OEM_TABLE_ID);
    write_u32(mem, offset + 24, 1); // OEM Revision
    mem[offset + 28..offset + 32].copy_from_slice(b"TROS"); // Creator ID
    write_u32(mem, offset + 32, 1); // Creator Revision
    
    pos = offset + 36;
    
    // === MADT-specific fields (after standard header) ===
    
    // Local APIC Address (32-bit physical)
    write_u32(mem, pos, LAPIC_PHYS_ADDR);
    pos += 4;
    
    // Flags: bit 0 = PCAT_COMPAT (dual-8259 legacy setup present)
    write_u32(mem, pos, 1); // Yes, we emulate 8259 PICs
    pos += 4;
    
    // === MADT Entry: Processor Local APIC (Type 0, Length 8) ===
    mem[pos] = 0;     // Type = Processor Local APIC
    mem[pos + 1] = 8; // Length
    mem[pos + 2] = 0; // ACPI Processor UID
    mem[pos + 3] = 0; // APIC ID = 0 (BSP)
    write_u32(mem, pos + 4, 1); // Flags: bit 0 = Processor Enabled
    pos += 8;
    
    // === MADT Entry: I/O APIC (Type 1, Length 12) ===
    mem[pos] = 1;      // Type = I/O APIC
    mem[pos + 1] = 12; // Length
    mem[pos + 2] = IOAPIC_ID; // I/O APIC ID
    mem[pos + 3] = 0;  // Reserved
    write_u32(mem, pos + 4, IOAPIC_PHYS_ADDR); // I/O APIC Address
    write_u32(mem, pos + 8, 0); // Global System Interrupt Base
    pos += 12;
    
    // === MADT Entry: Interrupt Source Override (Type 2, Length 10) ===
    // IRQ 0 (PIT timer) → GSI 2 (standard PC remap via IOAPIC)
    mem[pos] = 2;      // Type = Interrupt Source Override
    mem[pos + 1] = 10; // Length
    mem[pos + 2] = 0;  // Bus = ISA
    mem[pos + 3] = 0;  // Source = IRQ 0
    write_u32(mem, pos + 4, 2); // Global System Interrupt = 2
    write_u16(mem, pos + 8, 0); // Flags: conforms to bus spec
    pos += 10;
    
    // === MADT Entry: Interrupt Source Override (Type 2, Length 10) ===
    // IRQ 9 → GSI 9 (ACPI SCI, level-triggered active-low)
    mem[pos] = 2;      // Type = Interrupt Source Override
    mem[pos + 1] = 10; // Length
    mem[pos + 2] = 0;  // Bus = ISA
    mem[pos + 3] = 9;  // Source = IRQ 9
    write_u32(mem, pos + 4, 9); // Global System Interrupt = 9
    write_u16(mem, pos + 8, 0x000D); // Flags: active low, level-triggered
    pos += 10;
    
    // === MADT Entry: Local APIC NMI (Type 4, Length 6) ===
    // NMI connected to LINT1 on all processors
    mem[pos] = 4;     // Type = Local APIC NMI
    mem[pos + 1] = 6; // Length
    mem[pos + 2] = 0xFF; // ACPI Processor UID = 0xFF (all processors)
    write_u16(mem, pos + 3, 0x0005); // Flags: active high, edge-triggered
    mem[pos + 5] = 1; // Local APIC LINT# = 1
    pos += 6;
    
    let total_len = pos - offset;
    
    // Fill in length
    write_u32(mem, len_offset, total_len as u32);
    
    // Compute checksum
    fixup_checksum(mem, offset, total_len);
    
    total_len
}

// ============================================================================
// FADT — Fixed ACPI Description Table
// ============================================================================

/// Build a minimal FADT (rev 5) at the given offset.
///
/// Linux uses FADT to find:
/// - PM timer port (for calibration)
/// - SCI interrupt number
/// - ACPI enable/disable ports
/// - Pointer to DSDT
fn build_fadt(mem: &mut [u8], offset: usize, dsdt_phys: u64) -> usize {
    // FADT rev 5 is 276 bytes
    let total_len: usize = 276;
    
    // Signature
    mem[offset..offset + 4].copy_from_slice(b"FACP");
    write_u32(mem, offset + 4, total_len as u32);
    mem[offset + 8] = 5; // Revision 5
    mem[offset + 9] = 0; // Checksum — filled later
    mem[offset + 10..offset + 16].copy_from_slice(OEM_ID);
    mem[offset + 16..offset + 24].copy_from_slice(OEM_TABLE_ID);
    write_u32(mem, offset + 24, 1); // OEM Revision
    mem[offset + 28..offset + 32].copy_from_slice(b"TROS"); // Creator ID
    write_u32(mem, offset + 32, 1); // Creator Revision
    
    // FADT-specific fields:
    
    // FIRMWARE_CTRL (32-bit FACS pointer) — 0 (no FACS)
    write_u32(mem, offset + 36, 0);
    
    // DSDT (32-bit pointer)
    write_u32(mem, offset + 40, dsdt_phys as u32);
    
    // INT_MODEL (offset 45) — not used in ACPI 2.0+, set to 0
    mem[offset + 45] = 0;
    
    // Preferred PM Profile (offset 45) = 0 (unspecified)
    // SCI_INT (offset 46) = IRQ 9
    write_u16(mem, offset + 46, 9);
    
    // SMI_CMD (offset 48) = 0 (no SMI — we're a VM)
    write_u32(mem, offset + 48, 0);
    
    // ACPI_ENABLE (offset 52) = 0
    mem[offset + 52] = 0;
    // ACPI_DISABLE (offset 53) = 0
    mem[offset + 53] = 0;
    
    // PM1a_EVT_BLK (offset 56) — PM1a event register block
    write_u32(mem, offset + 56, 0xB000);
    
    // PM1b_EVT_BLK (offset 60) = 0
    write_u32(mem, offset + 60, 0);
    
    // PM1a_CNT_BLK (offset 64) — PM1a control register block
    write_u32(mem, offset + 64, 0xB004);
    
    // PM1b_CNT_BLK (offset 68) = 0
    write_u32(mem, offset + 68, 0);
    
    // PM2_CNT_BLK (offset 72) = 0
    write_u32(mem, offset + 72, 0);
    
    // PM_TMR_BLK (offset 76) — PM timer I/O port
    write_u32(mem, offset + 76, 0xB008);
    
    // GPE0_BLK (offset 80) = 0
    write_u32(mem, offset + 80, 0);
    // GPE1_BLK (offset 84) = 0
    write_u32(mem, offset + 84, 0);
    
    // PM1_EVT_LEN (offset 88) = 4
    mem[offset + 88] = 4;
    // PM1_CNT_LEN (offset 89) = 2
    mem[offset + 89] = 2;
    // PM2_CNT_LEN (offset 90) = 0
    mem[offset + 90] = 0;
    // PM_TMR_LEN (offset 91) = 4 (32-bit timer)
    mem[offset + 91] = 4;
    
    // GPE0_BLK_LEN (offset 92) = 0
    mem[offset + 92] = 0;
    // GPE1_BLK_LEN (offset 93) = 0
    mem[offset + 93] = 0;
    
    // Flags (offset 112) — important bits:
    //   bit 0: WBINVD (processor correctly supports WBINVD)
    //   bit 4: TMR_VAL_EXT (PM timer is 32-bit)
    //   bit 8: RESET_REG_SUP (platform supports reset via FADT)
    //   bit 20: HW_REDUCED_ACPI = 0 (we provide full ACPI)
    let fadt_flags: u32 = (1 << 0)  // WBINVD
                        | (1 << 4)  // TMR_VAL_EXT (32-bit PM timer)
                        | (1 << 8); // RESET_REG_SUP
    write_u32(mem, offset + 112, fadt_flags);
    
    // RESET_REG (offset 116) — Generic Address Structure (12 bytes)
    // Address space: System I/O (1), bit width: 8, offset: 0, access_size: byte
    mem[offset + 116] = 1; // Address space = System I/O
    mem[offset + 117] = 8; // Bit width
    mem[offset + 118] = 0; // Bit offset
    mem[offset + 119] = 1; // Access size = Byte
    write_u64(mem, offset + 120, 0xCF9); // Reset register address (standard x86)
    
    // RESET_VALUE (offset 128) = 0x06 (full reset)
    mem[offset + 128] = 0x06;
    
    // ARM_BOOT_ARCH (offset 129) = 0 (not ARM)
    write_u16(mem, offset + 129, 0);
    
    // FADT Minor Version (offset 131) = 1
    mem[offset + 131] = 1;
    
    // X_FIRMWARE_CTRL (offset 132, 64-bit) = 0
    write_u64(mem, offset + 132, 0);
    
    // X_DSDT (offset 140, 64-bit) — 64-bit DSDT pointer
    write_u64(mem, offset + 140, dsdt_phys);
    
    // X_PM1a_EVT_BLK (offset 148) — GAS for PM1a event block
    write_gas(mem, offset + 148, 1, 32, 0, 2, 0xB000); // I/O, 32-bit
    
    // X_PM1b_EVT_BLK (offset 160) = empty
    // X_PM1a_CNT_BLK (offset 172) — GAS for PM1a control block  
    write_gas(mem, offset + 172, 1, 16, 0, 2, 0xB004); // I/O, 16-bit
    
    // X_PM1b_CNT_BLK (offset 184) = empty
    // X_PM2_CNT_BLK (offset 196) = empty
    
    // X_PM_TMR_BLK (offset 208) — GAS for PM timer
    write_gas(mem, offset + 208, 1, 32, 0, 3, 0xB008); // I/O, 32-bit, DWORD access
    
    // Compute checksum
    fixup_checksum(mem, offset, total_len);
    
    total_len
}

// ============================================================================
// DSDT — Differentiated System Description Table (empty stub)
// ============================================================================

/// Build a minimal empty DSDT (just the standard ACPI header).
/// A real DSDT would contain AML bytecode describing the platform,
/// but an empty one is enough for Linux to proceed without ACPI panics.
fn build_dsdt(mem: &mut [u8], offset: usize) -> usize {
    let total_len: usize = 36; // Just the header, no AML
    
    mem[offset..offset + 4].copy_from_slice(b"DSDT");
    write_u32(mem, offset + 4, total_len as u32);
    mem[offset + 8] = 2; // Revision 2
    mem[offset + 9] = 0; // Checksum
    mem[offset + 10..offset + 16].copy_from_slice(OEM_ID);
    mem[offset + 16..offset + 24].copy_from_slice(OEM_TABLE_ID);
    write_u32(mem, offset + 24, 1);
    mem[offset + 28..offset + 32].copy_from_slice(b"TROS");
    write_u32(mem, offset + 32, 1);
    
    fixup_checksum(mem, offset, total_len);
    
    total_len
}

// ============================================================================
// HELPERS
// ============================================================================

/// Compute and fix the checksum byte at offset+9 so all bytes sum to 0.
fn fixup_checksum(mem: &mut [u8], offset: usize, len: usize) {
    mem[offset + 9] = 0;
    let mut sum: u8 = 0;
    for i in 0..len {
        sum = sum.wrapping_add(mem[offset + i]);
    }
    mem[offset + 9] = 0u8.wrapping_sub(sum);
}

/// Write a Generic Address Structure (GAS, 12 bytes) at the given offset.
fn write_gas(mem: &mut [u8], offset: usize, addr_space: u8, bit_width: u8, bit_offset: u8, access_size: u8, address: u64) {
    mem[offset] = addr_space;
    mem[offset + 1] = bit_width;
    mem[offset + 2] = bit_offset;
    mem[offset + 3] = access_size;
    write_u64(mem, offset + 4, address);
}

fn write_u16(mem: &mut [u8], offset: usize, value: u16) {
    let bytes = value.to_le_bytes();
    mem[offset] = bytes[0];
    mem[offset + 1] = bytes[1];
}

fn write_u32(mem: &mut [u8], offset: usize, value: u32) {
    let bytes = value.to_le_bytes();
    mem[offset..offset + 4].copy_from_slice(&bytes);
}

fn write_u64(mem: &mut [u8], offset: usize, value: u64) {
    let bytes = value.to_le_bytes();
    mem[offset..offset + 8].copy_from_slice(&bytes);
}

// ============================================================================
// UNIT TESTS (documentation / future test framework)
// ============================================================================
// These tests validate ACPI table generation on pure byte arrays.
// They can run when a custom test framework is added, or are exercised
// by the in-kernel `hv test` command via hypervisor::tests module.

#[cfg(test)]
mod tests {
    use super::*;

    fn read_u32_le(mem: &[u8], offset: usize) -> u32 {
        u32::from_le_bytes([mem[offset], mem[offset+1], mem[offset+2], mem[offset+3]])
    }
    fn read_u64_le(mem: &[u8], offset: usize) -> u64 {
        u64::from_le_bytes([
            mem[offset], mem[offset+1], mem[offset+2], mem[offset+3],
            mem[offset+4], mem[offset+5], mem[offset+6], mem[offset+7],
        ])
    }
    fn checksum_ok(mem: &[u8], offset: usize, len: usize) -> bool {
        let mut sum: u8 = 0;
        for i in 0..len { sum = sum.wrapping_add(mem[offset + i]); }
        sum == 0
    }

    #[test]
    fn test_rsdp_signature_and_checksums() {
        let mut mem = vec![0u8; 0xF0000];
        install_acpi_tables(&mut mem);
        assert_eq!(&mem[0x50000..0x50008], b"RSD PTR ");
        assert!(checksum_ok(&mem, 0x50000, 20), "RSDP v1 checksum");
        assert!(checksum_ok(&mem, 0x50000, 36), "RSDP v2 checksum");
        assert_eq!(mem[0x50000 + 15], 2, "RSDP revision");
    }

    #[test]
    fn test_xsdt_structure() {
        let mut mem = vec![0u8; 0xF0000];
        install_acpi_tables(&mut mem);
        let xsdt = 0x50040;
        assert_eq!(&mem[xsdt..xsdt+4], b"XSDT");
        let len = read_u32_le(&mem, xsdt + 4) as usize;
        assert!(checksum_ok(&mem, xsdt, len));
        assert_eq!(read_u64_le(&mem, xsdt + 36), 0x50080); // MADT
        assert_eq!(read_u64_le(&mem, xsdt + 44), 0x50100); // FADT
    }

    #[test]
    fn test_madt_lapic_and_ioapic() {
        let mut mem = vec![0u8; 0xF0000];
        install_acpi_tables(&mut mem);
        let madt = 0x50080;
        assert_eq!(&mem[madt..madt+4], b"APIC");
        let len = read_u32_le(&mem, madt + 4) as usize;
        assert!(checksum_ok(&mem, madt, len));
        assert_eq!(read_u32_le(&mem, madt + 36), 0xFEE0_0000);
    }

    #[test]
    fn test_fadt_fields() {
        let mut mem = vec![0u8; 0xF0000];
        install_acpi_tables(&mut mem);
        let fadt = 0x50100;
        assert_eq!(&mem[fadt..fadt+4], b"FACP");
        let len = read_u32_le(&mem, fadt + 4) as usize;
        assert_eq!(len, 276);
        assert!(checksum_ok(&mem, fadt, len));
        assert_eq!(read_u32_le(&mem, fadt + 76), 0xB008); // PM timer
    }

    #[test]
    fn test_dsdt_stub() {
        let mut mem = vec![0u8; 0xF0000];
        install_acpi_tables(&mut mem);
        let dsdt = 0x50200;
        assert_eq!(&mem[dsdt..dsdt+4], b"DSDT");
        let len = read_u32_le(&mem, dsdt + 4) as usize;
        assert_eq!(len, 36);
        assert!(checksum_ok(&mem, dsdt, len));
    }

    #[test]
    fn test_bios_rsdp_copy() {
        let mut mem = vec![0u8; 0xF0000];
        install_acpi_tables(&mut mem);
        assert_eq!(&mem[0x50000..0x50024], &mem[0xE0000..0xE0024]);
    }
}
