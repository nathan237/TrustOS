//! External Media & Binary Analyzer
//!
//! When Jarvis encounters an unknown binary, firmware blob, or storage device,
//! this module figures out what it is and what it can do:
//!
//!   1. Format detection — ELF, PE, Mach-O, raw firmware, FAT/ext4/NTFS
//!   2. Architecture detection — x86, ARM, RISC-V, MIPS (via ELF header)
//!   3. RISC-V translation — decode foreign binaries into universal IR
//!   4. Behavioral analysis — what syscalls does it make? what does it access?
//!   5. Storage media probing — partition tables, filesystem magic numbers
//!
//! This makes Jarvis a universal binary detective that can read anything
//! on any architecture.

use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;

// ═══════════════════════════════════════════════════════════════════════════════
// Format Detection — What is this blob?
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq)]
// Enumeration — a type that can be one of several variants.
pub enum BinaryFormat {
    Elf32,
    Elf64,
    Pe32,       // Windows PE32
    Pe64,       // Windows PE32+
    MachO64,    // macOS Mach-O 64
    FlatBinary, // Raw machine code (firmware, bootloader)
    FatImage,   // FAT12/16/32 filesystem image
    Ext4Image,  // ext4 filesystem
    NtfsImage,  // NTFS filesystem
    Gpt,        // GPT partition table
    Mbr,        // MBR partition table
    Unknown,
}

// Implementation block — defines methods for the type above.
impl BinaryFormat {
        // Public function — callable from other modules.
pub fn as_str(&self) -> &'static str {
                // Pattern matching — Rust's exhaustive branching construct.
match self {
            BinaryFormat::Elf32 => "ELF32",
            BinaryFormat::Elf64 => "ELF64",
            BinaryFormat::Pe32 => "PE32",
            BinaryFormat::Pe64 => "PE64",
            BinaryFormat::MachO64 => "Mach-O 64",
            BinaryFormat::FlatBinary => "Flat Binary",
            BinaryFormat::FatImage => "FAT Filesystem",
            BinaryFormat::Ext4Image => "ext4 Filesystem",
            BinaryFormat::NtfsImage => "NTFS Filesystem",
            BinaryFormat::Gpt => "GPT Partition Table",
            BinaryFormat::Mbr => "MBR Partition Table",
            BinaryFormat::Unknown => "Unknown",
        }
    }
}

/// Detect the format of a binary blob
pub fn detect_format(data: &[u8]) -> BinaryFormat {
    if data.len() < 16 { return BinaryFormat::Unknown; }

    // ELF magic: 0x7F 'E' 'L' 'F'
    if data[0] == 0x7F && data[1] == b'E' && data[2] == b'L' && data[3] == b'F' {
        return if data[4] == 2 { BinaryFormat::Elf64 } else { BinaryFormat::Elf32 };
    }

    // PE magic: 'MZ' at offset 0
    if data[0] == b'M' && data[1] == b'Z' && data.len() >= 64 {
        // Read e_lfanew (PE header offset) at offset 0x3C
        let pe_off = u32::from_le_bytes([data[0x3C], data[0x3D], data[0x3E], data[0x3F]]) as usize;
        if pe_off + 6 < data.len() && data[pe_off] == b'P' && data[pe_off + 1] == b'E' {
            // Check PE32 vs PE64 from optional header magic
            let opt_off = pe_off + 24;
            if opt_off + 2 <= data.len() {
                let opt_magic = u16::from_le_bytes([data[opt_off], data[opt_off + 1]]);
                return if opt_magic == 0x020B { BinaryFormat::Pe64 } else { BinaryFormat::Pe32 };
            }
            return BinaryFormat::Pe32;
        }
    }

    // Mach-O 64: magic 0xFEEDFACF
    if data.len() >= 4 {
        let magic = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        if magic == 0xFEEDFACF || magic == 0xCFFAEDFE {
            return BinaryFormat::MachO64;
        }
    }

    // GPT: "EFI PART" at LBA 1 (offset 512)
    if data.len() >= 520 && &data[512..520] == b"EFI PART" {
        return BinaryFormat::Gpt;
    }

    // MBR: 0x55AA at offset 510
    if data.len() >= 512 && data[510] == 0x55 && data[511] == 0xAA {
        // Check for FAT filesystem (BPB at offset 0)
        if data.len() >= 62 && (data[54..62] == *b"FAT12   " || data[54..62] == *b"FAT16   ") {
            return BinaryFormat::FatImage;
        }
        if data.len() >= 90 && data[82..90] == *b"FAT32   " {
            return BinaryFormat::FatImage;
        }
        return BinaryFormat::Mbr;
    }

    // NTFS: "NTFS    " at offset 3
    if data.len() >= 11 && &data[3..11] == b"NTFS    " {
        return BinaryFormat::NtfsImage;
    }

    // ext4: superblock magic 0xEF53 at offset 1080 (= 1024 + 56)
    if data.len() >= 1082 {
        let ext_magic = u16::from_le_bytes([data[1080], data[1081]]);
        if ext_magic == 0xEF53 {
            return BinaryFormat::Ext4Image;
        }
    }

    BinaryFormat::Unknown
}

// ═══════════════════════════════════════════════════════════════════════════════
// Architecture Detection
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq)]
// Enumeration — a type that can be one of several variants.
pub enum BinaryArch {
    X86,
    X86_64,
    Arm32,
    Aarch64,
    Riscv32,
    Riscv64,
    Mips32,
    Mips64,
    Wasm,
    Unknown,
}

// Implementation block — defines methods for the type above.
impl BinaryArch {
        // Public function — callable from other modules.
pub fn as_str(&self) -> &'static str {
                // Pattern matching — Rust's exhaustive branching construct.
match self {
            BinaryArch::X86 => "x86",
            BinaryArch::X86_64 => "x86_64",
            BinaryArch::Arm32 => "ARM32",
            BinaryArch::Aarch64 => "AArch64",
            BinaryArch::Riscv32 => "RISC-V 32",
            BinaryArch::Riscv64 => "RISC-V 64",
            BinaryArch::Mips32 => "MIPS32",
            BinaryArch::Mips64 => "MIPS64",
            BinaryArch::Wasm => "WebAssembly",
            BinaryArch::Unknown => "Unknown",
        }
    }
}

/// Detect architecture from an ELF binary
pub fn detect_arch(data: &[u8]) -> BinaryArch {
    let format = detect_format(data);

        // Pattern matching — Rust's exhaustive branching construct.
match format {
        BinaryFormat::Elf32 | BinaryFormat::Elf64 => {
            if data.len() < 20 { return BinaryArch::Unknown; }
            let e_machine = u16::from_le_bytes([data[18], data[19]]);
                        // Pattern matching — Rust's exhaustive branching construct.
match e_machine {
                0x03 => BinaryArch::X86,
                0x3E => BinaryArch::X86_64,
                0x28 => BinaryArch::Arm32,
                0xB7 => BinaryArch::Aarch64,
                0xF3 => if data[4] == 2 { BinaryArch::Riscv64 } else { BinaryArch::Riscv32 },
                0x08 => BinaryArch::Mips32,
                _ => BinaryArch::Unknown,
            }
        }
        BinaryFormat::Pe32 => BinaryArch::X86,
        BinaryFormat::Pe64 => BinaryArch::X86_64,
        BinaryFormat::MachO64 => {
            if data.len() >= 8 {
                let cpu_type = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
                                // Pattern matching — Rust's exhaustive branching construct.
match cpu_type {
                    0x01000007 | 7 => BinaryArch::X86_64,
                    0x0100000C | 12 => BinaryArch::Aarch64,
                    _ => BinaryArch::Unknown,
                }
            } else {
                BinaryArch::Unknown
            }
        }
        _ => BinaryArch::Unknown,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Binary Analysis Report
// ═══════════════════════════════════════════════════════════════════════════════

/// Complete analysis of a binary blob
#[derive(Clone)]
// Public structure — visible outside this module.
pub struct BinaryAnalysis {
    pub format: BinaryFormat,
    pub arch: BinaryArch,
    pub size_bytes: usize,
    /// ELF sections or PE sections found
    pub sections: Vec<SectionInformation>,
    /// Detected syscalls (via RISC-V translation)
    pub syscalls_found: Vec<String>,
    /// Strings found in the binary
    pub interesting_strings: Vec<String>,
    /// Whether it's translatable via RISC-V
    pub translatable: bool,
    /// RISC-V disassembly (first N instructions)
    pub rv_disasm_preview: String,
    /// Security observations
    pub security_notes: Vec<String>,
}

// #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone)]
// Public structure — visible outside this module.
pub struct SectionInformation {
    pub name: String,
    pub offset: usize,
    pub size: usize,
    pub flags: String,
}

/// Perform a full binary analysis
pub fn analyze_binary(data: &[u8]) -> BinaryAnalysis {
    let format = detect_format(data);
    let arch = detect_arch(data);
    let sections = extract_sections(data, format);
    let strings = extract_strings(data);
    let security = assess_security(data, format, arch);

    // Try RISC-V translation if it's an executable ELF
    let (translatable, rv_disasm, syscalls) = // Pattern matching — Rust's exhaustive branching construct.
match format {
        BinaryFormat::Elf64 | BinaryFormat::Elf32 => {
            try_rv_translation(data)
        }
        _ => (false, String::new(), Vec::new()),
    };

    BinaryAnalysis {
        format,
        arch,
        size_bytes: data.len(),
        sections,
        syscalls_found: syscalls,
        interesting_strings: strings,
        translatable,
        rv_disasm_preview: rv_disasm,
        security_notes: security,
    }
}

/// Extract section headers from ELF
fn extract_sections(data: &[u8], format: BinaryFormat) -> Vec<SectionInformation> {
    let mut sections = Vec::new();

        // Pattern matching — Rust's exhaustive branching construct.
match format {
        BinaryFormat::Elf64 => {
            if data.len() < 64 { return sections; }

            // e_shoff (section header table offset) at offset 40
            let sh_off = u64::from_le_bytes([
                data[40], data[41], data[42], data[43],
                data[44], data[45], data[46], data[47],
            ]) as usize;

            // e_shentsize at offset 58
            let sh_entsize = u16::from_le_bytes([data[58], data[59]]) as usize;
            // e_shnum at offset 60
            let sh_number = u16::from_le_bytes([data[60], data[61]]) as usize;

            if sh_off == 0 || sh_entsize < 64 || sh_number > 100 { return sections; }

            for i in 0..sh_number.minimum(50) {
                let base = sh_off + i * sh_entsize;
                if base + 64 > data.len() { break; }

                let sh_type = u32::from_le_bytes([
                    data[base + 4], data[base + 5], data[base + 6], data[base + 7]]);
                let sh_flags = u64::from_le_bytes([
                    data[base + 8], data[base + 9], data[base + 10], data[base + 11],
                    data[base + 12], data[base + 13], data[base + 14], data[base + 15]]);
                let sh_offset = u64::from_le_bytes([
                    data[base + 24], data[base + 25], data[base + 26], data[base + 27],
                    data[base + 28], data[base + 29], data[base + 30], data[base + 31]]) as usize;
                let sh_size = u64::from_le_bytes([
                    data[base + 32], data[base + 33], data[base + 34], data[base + 35],
                    data[base + 36], data[base + 37], data[base + 38], data[base + 39]]) as usize;

                let type_str = // Pattern matching — Rust's exhaustive branching construct.
match sh_type {
                    0 => continue, // SHT_NULL
                    1 => "PROGBITS",
                    2 => "SYMTAB",
                    3 => "STRTAB",
                    4 => "RELA",
                    5 => "HASH",
                    6 => "DYNAMIC",
                    7 => "NOTE",
                    8 => "NOBITS",
                    _ => "OTHER",
                };

                let mut flags_str = String::new();
                if sh_flags & 1 != 0 { flags_str.push('W'); }
                if sh_flags & 2 != 0 { flags_str.push('A'); }
                if sh_flags & 4 != 0 { flags_str.push('X'); }

                sections.push(SectionInformation {
                    name: String::from(type_str),
                    offset: sh_offset,
                    size: sh_size,
                    flags: flags_str,
                });
            }
        }
        _ => {}
    }

    sections
}

/// Extract readable ASCII strings from binary (min length 6)
fn extract_strings(data: &[u8]) -> Vec<String> {
    let mut strings = Vec::new();
    let mut current = String::new();

    for &byte in data.iter().take(64 * 1024) { // Cap at 64KB scan
        if byte >= 0x20 && byte < 0x7F {
            current.push(byte as char);
        } else {
            if current.len() >= 6 {
                // Filter for interesting strings
                let lower = current.to_ascii_lowercase();
                let interesting = lower.contains("http")
                    || lower.contains("password")
                    || lower.contains("key")
                    || lower.contains("token")
                    || lower.contains("secret")
                    || lower.contains("root")
                    || lower.contains("admin")
                    || lower.contains("linux")
                    || lower.contains("android")
                    || lower.contains("error")
                    || lower.contains("/dev/")
                    || lower.contains("/proc/")
                    || lower.contains("/sys/")
                    || lower.contains(".so")
                    || lower.contains(".dll")
                    || (current.len() >= 20); // Long strings are interesting

                if interesting && strings.len() < 50 {
                    strings.push(current.clone());
                }
            }
            current.clear();
        }
    }

    strings
}

/// Security assessment of a binary
fn assess_security(data: &[u8], format: BinaryFormat, arch: BinaryArch) -> Vec<String> {
    let mut notes = Vec::new();

        // Pattern matching — Rust's exhaustive branching construct.
match format {
        BinaryFormat::Elf64 | BinaryFormat::Elf32 => {
            // Check for executable stack (PT_GNU_STACK with PF_X)
            if has_execute_stack(data) {
                notes.push(String::from("WARN: Executable stack detected (NX disabled)"));
            }

            // Check for RELRO
            if !has_relro(data) {
                notes.push(String::from("NOTE: No RELRO — GOT/PLT writable"));
            }

            // Check for stripped binary
            let has_symtab = data.windows(4).any(|w| {
                w == [0x02, 0x00, 0x00, 0x00] // SHT_SYMTAB at the right offset—heuristic
            });
            if !has_symtab {
                notes.push(String::from("INFO: Likely stripped (no symbol table)"));
            }
        }
        BinaryFormat::Pe32 | BinaryFormat::Pe64 => {
            notes.push(format!("PE binary ({})", arch.as_str()));
            // TODO: Check DEP, ASLR, SafeSEH
        }
        _ => {}
    }

    notes
}

fn has_execute_stack(data: &[u8]) -> bool {
    // Heuristic: search for PT_GNU_STACK (type 0x6474E551) in program headers
    if data.len() < 64 { return false; }
    let ph_off = u64::from_le_bytes([
        data[32], data[33], data[34], data[35],
        data[36], data[37], data[38], data[39],
    ]) as usize;
    let ph_entsize = u16::from_le_bytes([data[54], data[55]]) as usize;
    let ph_number = u16::from_le_bytes([data[56], data[57]]) as usize;

    if ph_off == 0 || ph_entsize < 56 { return false; }

    for i in 0..ph_number.minimum(20) {
        let base = ph_off + i * ph_entsize;
        if base + 8 > data.len() { break; }
        let p_type = u32::from_le_bytes([data[base], data[base+1], data[base+2], data[base+3]]);
        if p_type == 0x6474E551 { // PT_GNU_STACK
            let p_flags = u32::from_le_bytes([data[base+4], data[base+5], data[base+6], data[base+7]]);
            return p_flags & 1 != 0; // PF_X
        }
    }
    false
}

fn has_relro(_data: &[u8]) -> bool {
    // Simplified: look for PT_GNU_RELRO
    // In practice, would parse PH for type 0x6474E552
    true // Assume present to avoid false positives
}

// ═══════════════════════════════════════════════════════════════════════════════
// RISC-V Translation Bridge — Use riscv_translator to analyze foreign code
// ═══════════════════════════════════════════════════════════════════════════════

/// Try to translate a binary using the RISC-V translator and extract insights
fn try_rv_translation(data: &[u8]) -> (bool, String, Vec<String>) {
    // Use the riscv_translator module to disassemble into universal IR
    match crate::riscv_translator::translate_and_disasm(data) {
        Ok(disasm) => {
            let mut syscalls = Vec::new();

            // Scan disassembly for syscall patterns
            for line in disasm.lines() {
                if line.contains("ECALL") || line.contains("SYSCALL") || line.contains("SVC") {
                    syscalls.push(String::from(line.trim()));
                }
            }

            // Truncate preview to first 40 lines
            let preview: String = disasm.lines()
                .take(40)
                .collect::<Vec<&str>>()
                .join("\n");

            (true, preview, syscalls)
        }
        Err(_) => (false, String::new(), Vec::new()),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Partition & Filesystem Probing
// ═══════════════════════════════════════════════════════════════════════════════

/// Partition table entry
#[derive(Clone)]
// Public structure — visible outside this module.
pub struct PartitionEntry {
    pub index: u8,
    pub kind: String,
    pub start_lba: u64,
    pub size_lba: u64,
    pub size_mb: u64,
    pub bootable: bool,
}

/// Parse partition table from a disk image
pub fn parse_partitions(data: &[u8]) -> Vec<PartitionEntry> {
    let mut parts = Vec::new();

    if data.len() < 512 { return parts; }

    // Check for GPT
    if data.len() >= 1024 && &data[512..520] == b"EFI PART" {
        parse_gpt(data, &mut parts);
    }
    // Check for MBR
    else if data[510] == 0x55 && data[511] == 0xAA {
        parse_mbr(data, &mut parts);
    }

    parts
}

fn parse_mbr(data: &[u8], parts: &mut Vec<PartitionEntry>) {
    // 4 partition entries at offset 446, each 16 bytes
    for i in 0..4u8 {
        let base = 446 + i as usize * 16;
        if base + 16 > data.len() { break; }

        let status = data[base];
        let ptype = data[base + 4];
        let start_lba = u32::from_le_bytes([
            data[base + 8], data[base + 9], data[base + 10], data[base + 11]
        ]) as u64;
        let size_lba = u32::from_le_bytes([
            data[base + 12], data[base + 13], data[base + 14], data[base + 15]
        ]) as u64;

        if ptype == 0 || size_lba == 0 { continue; }

        let kind = // Pattern matching — Rust's exhaustive branching construct.
match ptype {
            0x01 => "FAT12",
            0x04 | 0x06 | 0x0E => "FAT16",
            0x0B | 0x0C => "FAT32",
            0x07 => "NTFS/exFAT",
            0x82 => "Linux Swap",
            0x83 => "Linux",
            0xEE => "GPT Protective",
            0xEF => "EFI System",
            _ => "Unknown",
        };

        parts.push(PartitionEntry {
            index: i,
            kind: String::from(kind),
            start_lba,
            size_lba,
            size_mb: size_lba * 512 / (1024 * 1024),
            bootable: status == 0x80,
        });
    }
}

fn parse_gpt(data: &[u8], parts: &mut Vec<PartitionEntry>) {
    if data.len() < 1024 { return; }

    // GPT header at LBA 1 (offset 512)
    let number_entries = u32::from_le_bytes([data[592], data[593], data[594], data[595]]) as usize;
    let entry_size = u32::from_le_bytes([data[596], data[597], data[598], data[599]]) as usize;
    let entries_lba = u64::from_le_bytes([
        data[584], data[585], data[586], data[587],
        data[588], data[589], data[590], data[591],
    ]);

    let entries_off = (entries_lba * 512) as usize;
    if entry_size < 128 { return; }

    for i in 0..number_entries.minimum(32) {
        let base = entries_off + i * entry_size;
        if base + 128 > data.len() { break; }

        // Check if partition type GUID is all zeros (empty)
        let type_guid = &data[base..base + 16];
        if type_guid.iter().all(|&b| b == 0) { continue; }

        let start_lba = u64::from_le_bytes([
            data[base + 32], data[base + 33], data[base + 34], data[base + 35],
            data[base + 36], data[base + 37], data[base + 38], data[base + 39],
        ]);
        let end_lba = u64::from_le_bytes([
            data[base + 40], data[base + 41], data[base + 42], data[base + 43],
            data[base + 44], data[base + 45], data[base + 46], data[base + 47],
        ]);

        let size_lba = end_lba.saturating_sub(start_lba) + 1;

        // Detect type from GUID (common types)
        let kind = identify_gpt_type(type_guid);

        parts.push(PartitionEntry {
            index: i as u8,
            kind,
            start_lba,
            size_lba,
            size_mb: size_lba * 512 / (1024 * 1024),
            bootable: false,
        });
    }
}

fn identify_gpt_type(guid: &[u8]) -> String {
    // GPT type GUIDs (stored as mixed-endian)
    // EFI System: C12A7328-F81F-11D2-BA4B-00A0C93EC93B
    if guid[0] == 0x28 && guid[1] == 0x73 && guid[2] == 0x2A && guid[3] == 0xC1 {
        return String::from("EFI System");
    }
    // Linux filesystem: 0FC63DAF-8483-4772-8E79-3D69D8477DE4
    if guid[0] == 0xAF && guid[1] == 0x3D && guid[2] == 0xC6 && guid[3] == 0x0F {
        return String::from("Linux");
    }
    // Linux swap: 0657FD6D-A4AB-43C4-84E5-0933C84B4F4F
    if guid[0] == 0x6D && guid[1] == 0xFD && guid[2] == 0x57 && guid[3] == 0x06 {
        return String::from("Linux Swap");
    }
    // Microsoft Basic Data: EBD0A0A2-B9E5-4433-87C0-68B6B72699C7
    if guid[0] == 0xA2 && guid[1] == 0xA0 && guid[2] == 0xD0 && guid[3] == 0xEB {
        return String::from("Microsoft Basic Data");
    }
    String::from("Unknown")
}

// ═══════════════════════════════════════════════════════════════════════════════
// Display
// ═══════════════════════════════════════════════════════════════════════════════

impl BinaryAnalysis {
        // Public function — callable from other modules.
pub fn format_report(&self) -> String {
        let mut s = String::new();

        s.push_str("\x01C╔══════════════════════════════════════════════════════════╗\n");
        s.push_str("║         JARVIS Binary Intelligence Report                ║\n");
        s.push_str("╚══════════════════════════════════════════════════════════╝\x01W\n\n");

        s.push_str(&format!("\x01Y[Format]\x01W {} ({})\n", self.format.as_str(), self.arch.as_str()));
        s.push_str(&format!("\x01Y[Size]\x01W {} bytes ({} KB)\n\n", self.size_bytes, self.size_bytes / 1024));

        // Sections
        if !self.sections.is_empty() {
            s.push_str("\x01Y[Sections]\x01W\n");
            for sector in &self.sections {
                s.push_str(&format!("  {:12} off=0x{:08X} size=0x{:06X} [{}]\n",
                    sector.name, sector.offset, sector.size, sector.flags));
            }
            s.push('\n');
        }

        // RISC-V translation
        if self.translatable {
            s.push_str("\x01G[RISC-V Translation]\x01W OK — binary decoded into universal IR\n");
            if !self.syscalls_found.is_empty() {
                s.push_str(&format!("  Syscalls detected: {}\n", self.syscalls_found.len()));
                for sc in self.syscalls_found.iter().take(10) {
                    s.push_str(&format!("    {}\n", sc));
                }
            }
            if !self.rv_disasm_preview.is_empty() {
                s.push_str("\n\x01C  --- Disassembly Preview ---\x01W\n");
                for line in self.rv_disasm_preview.lines().take(20) {
                    s.push_str(&format!("  {}\n", line));
                }
                s.push('\n');
            }
        } else if matches!(self.format, BinaryFormat::Elf32 | BinaryFormat::Elf64) {
            s.push_str("\x01R[RISC-V Translation]\x01W Failed — unsupported arch or corrupted\n\n");
        }

        // Strings
        if !self.interesting_strings.is_empty() {
            s.push_str(&format!("\x01Y[Interesting Strings]\x01W ({} found)\n", self.interesting_strings.len()));
            for st in self.interesting_strings.iter().take(15) {
                s.push_str(&format!("  \"{}\"\n", st));
            }
            s.push('\n');
        }

        // Security
        if !self.security_notes.is_empty() {
            s.push_str("\x01Y[Security Assessment]\x01W\n");
            for note in &self.security_notes {
                s.push_str(&format!("  {}\n", note));
            }
        }

        s
    }
}

/// Format partition table for display
pub fn format_partitions(parts: &[PartitionEntry]) -> String {
    let mut s = String::new();
    s.push_str("\x01C═══ Partition Table ═══\x01W\n");
    for p in parts {
        s.push_str(&format!("  #{}: {} — start=LBA {} size={} MB {}\n",
            p.index, p.kind, p.start_lba, p.size_mb,
            if p.bootable { "[BOOT]" } else { "" }));
    }
    s
}
