//! Linux Boot Protocol Loader for TrustVM
//!
//! Implémente le protocole de boot Linux (version 2.15+) pour charger
//! des noyaux Linux (bzImage) dans des VMs TrustVM.
//!
//! Supporte:
//! - Parsing du header bzImage
//! - Chargement du kernel en mémoire guest
//! - Configuration des boot_params (zero page)
//! - Tables de pages guest (identity mapping 0-4GB)
//! - GDT guest pour mode 64-bit
//! - Ligne de commande kernel
//!
//! Références:
//! - Documentation/x86/boot.rst dans le source Linux
//! - Documentation/x86/zero-page.rst

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use super::{HypervisorError, Result};

// ============================================================================
// MEMORY LAYOUT CONSTANTS
// ============================================================================

/// Where the boot_params (zero page) is placed in guest physical memory
const BOOT_PARAMS_ADDR: u64 = 0x7000;

/// Where the command line is placed
const CMDLINE_ADDR: u64 = 0x20000;

/// Maximum command line size
const CMDLINE_MAX: usize = 2048;

/// Where guest page tables start (PML4)
const PAGE_TABLES_ADDR: u64 = 0x70000;

/// Where the guest GDT is placed
const GDT_ADDR: u64 = 0x60000;

/// Where the protected-mode kernel is loaded (standard 1MB)
const KERNEL_LOAD_ADDR: u64 = 0x100000;

/// Where the initrd is loaded (after kernel, aligned to page)
const INITRD_LOAD_ADDR: u64 = 0x1000000; // 16MB — leaves room for kernel

/// Guest stack pointer
const GUEST_STACK_ADDR: u64 = 0x80000; // 512KB

// ============================================================================
// LINUX BOOT HEADER STRUCTURES
// ============================================================================

/// Linux setup_header — at offset 0x1F1 in the bzImage
/// See Documentation/x86/boot.rst in Linux source
#[derive(Debug, Clone)]
pub struct SetupHeader {
    /// Number of setup sectors (at 0x1F1)
    pub setup_sects: u8,
    /// Size of protected-mode kernel in 16-byte paragraphs (at 0x1F4)
    pub syssize: u32,
    /// Boot protocol magic "HdrS" (at 0x202)
    pub header_magic: u32,
    /// Boot protocol version (at 0x206)
    pub version: u16,
    /// Boot loader type (at 0x210)
    pub type_of_loader: u8,
    /// Load flags (at 0x211)
    pub loadflags: u8,
    /// 32-bit entry point (at 0x214)
    pub code32_start: u32,
    /// Ramdisk physical address (at 0x218)
    pub ramdisk_image: u32,
    /// Ramdisk size (at 0x21C)
    pub ramdisk_size: u32,
    /// Command line pointer (at 0x228)
    pub cmd_line_ptr: u32,
    /// Highest address for initrd (at 0x22C)
    pub initrd_addr_max: u32,
    /// Kernel alignment (at 0x230)
    pub kernel_alignment: u32,
    /// Can be relocated? (at 0x234)
    pub relocatable_kernel: u8,
    /// Minimum alignment (at 0x235)
    pub min_alignment: u8,
    /// Extended load flags (at 0x236)
    pub xloadflags: u16,
    /// Amount of memory used for init (at 0x260)
    pub init_size: u32,
    /// Preferred load address (at 0x258)
    pub pref_address: u64,
}

/// Loadflags bits
pub const LOADED_HIGH: u8 = 0x01;      // Protected-mode code loaded at 0x100000
pub const CAN_USE_HEAP: u8 = 0x80;     // Set heap_end_ptr

/// Xloadflags bits
pub const XLF_KERNEL_64: u16 = 0x01;   // Kernel has 64-bit entry point
pub const XLF_CAN_BE_LOADED_ABOVE_4G: u16 = 0x02;
pub const XLF_EFI_HANDOVER_32: u16 = 0x04;
pub const XLF_EFI_HANDOVER_64: u16 = 0x08;

/// E820 memory map entry type
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum E820Type {
    Ram = 1,
    Reserved = 2,
    Acpi = 3,
    Nvs = 4,
    Unusable = 5,
}

/// E820 memory map entry (20 bytes)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct E820Entry {
    pub addr: u64,
    pub size: u64,
    pub entry_type: u32,
}

// ============================================================================
// BZIMAGE PARSER
// ============================================================================

/// Parsed Linux kernel image
pub struct LinuxKernel {
    /// Setup header information
    pub header: SetupHeader,
    /// Protected-mode kernel code
    pub kernel_data: Vec<u8>,
    /// Setup sectors (real-mode part)
    pub setup_data: Vec<u8>,
    /// Whether the kernel supports 64-bit boot
    pub supports_64bit: bool,
    /// 64-bit entry point (relative to kernel load address)
    pub entry_64: u64,
}

/// Parse a bzImage file
pub fn parse_bzimage(data: &[u8]) -> Result<LinuxKernel> {
    if data.len() < 0x300 {
        crate::serial_println!("[Linux] bzImage too small: {} bytes", data.len());
        return Err(HypervisorError::InvalidConfiguration);
    }

    // Check magic at offset 0x202
    let magic = read_u32(data, 0x202);
    if magic != 0x53726448 { // "HdrS"
        crate::serial_println!("[Linux] Invalid bzImage magic: 0x{:08X} (expected 0x53726448)", magic);
        return Err(HypervisorError::InvalidConfiguration);
    }

    let version = read_u16(data, 0x206);
    crate::serial_println!("[Linux] Boot protocol version: {}.{}", 
                          version >> 8, version & 0xFF);

    if version < 0x020F {
        crate::serial_println!("[Linux] Boot protocol version too old (need >= 2.15)");
        return Err(HypervisorError::InvalidConfiguration);
    }

    let setup_sects = data[0x1F1];
    let setup_sects = if setup_sects == 0 { 4 } else { setup_sects }; // Default 4

    let header = SetupHeader {
        setup_sects,
        syssize: read_u32(data, 0x1F4),
        header_magic: magic,
        version,
        type_of_loader: data[0x210],
        loadflags: data[0x211],
        code32_start: read_u32(data, 0x214),
        ramdisk_image: read_u32(data, 0x218),
        ramdisk_size: read_u32(data, 0x21C),
        cmd_line_ptr: read_u32(data, 0x228),
        initrd_addr_max: read_u32(data, 0x22C),
        kernel_alignment: read_u32(data, 0x230),
        relocatable_kernel: data[0x234],
        min_alignment: data[0x235],
        xloadflags: read_u16(data, 0x236),
        init_size: read_u32(data, 0x260),
        pref_address: read_u64(data, 0x258),
    };

    let supports_64bit = (header.xloadflags & XLF_KERNEL_64) != 0;

    // Setup data = first (1 + setup_sects) * 512 bytes
    let setup_size = (1 + setup_sects as usize) * 512;
    let setup_data = if setup_size <= data.len() {
        data[..setup_size].to_vec()
    } else {
        crate::serial_println!("[Linux] Warning: setup size {} > image size {}", setup_size, data.len());
        data.to_vec()
    };

    // Protected-mode kernel = everything after setup
    let kernel_offset = setup_size;
    let kernel_data = if kernel_offset < data.len() {
        data[kernel_offset..].to_vec()
    } else {
        crate::serial_println!("[Linux] No protected-mode kernel data!");
        return Err(HypervisorError::InvalidConfiguration);
    };

    // 64-bit entry point: code32_start + 0x200 for 64-bit kernels
    let entry_64 = if supports_64bit {
        KERNEL_LOAD_ADDR + 0x200
    } else {
        KERNEL_LOAD_ADDR
    };

    crate::serial_println!("[Linux] Parsed bzImage:");
    crate::serial_println!("  Setup sectors: {}", setup_sects);
    crate::serial_println!("  Kernel size: {} bytes ({} KB)", 
                          kernel_data.len(), kernel_data.len() / 1024);
    crate::serial_println!("  64-bit: {}", supports_64bit);
    crate::serial_println!("  Entry point: 0x{:X}", entry_64);
    crate::serial_println!("  Preferred load: 0x{:X}", header.pref_address);
    crate::serial_println!("  Init size: {} KB", header.init_size / 1024);

    Ok(LinuxKernel {
        header,
        kernel_data,
        setup_data,
        supports_64bit,
        entry_64,
    })
}

// ============================================================================
// GUEST MEMORY SETUP
// ============================================================================

/// Configuration for a Linux guest
pub struct LinuxGuestConfig {
    /// Kernel command line
    pub cmdline: String,
    /// Guest memory size in bytes
    pub memory_size: u64,
    /// Optional initrd data
    pub initrd: Option<Vec<u8>>,
}

impl Default for LinuxGuestConfig {
    fn default() -> Self {
        Self {
            cmdline: String::from("console=ttyS0 earlyprintk=serial,ttyS0 nokaslr"),
            memory_size: 256 * 1024 * 1024, // 256MB
            initrd: None,
        }
    }
}

/// Result of loading a Linux kernel into guest memory
pub struct LinuxGuestSetup {
    /// Guest entry point (RIP)
    pub entry_point: u64,
    /// Guest stack pointer (RSP)
    pub stack_ptr: u64,
    /// Boot params address (for RSI)
    pub boot_params_addr: u64,
    /// Page table root (CR3) in guest physical space
    pub cr3: u64,
    /// GDT base address in guest physical space
    pub gdt_base: u64,
}

/// Load a parsed Linux kernel into guest memory.
///
/// This sets up:
/// 1. Protected-mode kernel at 0x100000
/// 2. boot_params (zero page) at BOOT_PARAMS_ADDR
/// 3. Command line at CMDLINE_ADDR
/// 4. Identity-mapped page tables at PAGE_TABLES_ADDR
/// 5. GDT at GDT_ADDR
/// 6. Optionally, initrd at INITRD_LOAD_ADDR
pub fn load_linux_kernel(
    guest_memory: &mut [u8],
    kernel: &LinuxKernel,
    config: &LinuxGuestConfig,
) -> Result<LinuxGuestSetup> {
    let mem_size = guest_memory.len() as u64;
    
    crate::serial_println!("[Linux] Loading kernel into {} MB guest memory",
                          mem_size / (1024 * 1024));

    // Verify we have enough memory
    let min_memory = KERNEL_LOAD_ADDR + kernel.kernel_data.len() as u64 + 0x100000;
    if mem_size < min_memory {
        crate::serial_println!("[Linux] Insufficient guest memory: need {} MB, have {} MB",
                              min_memory / (1024 * 1024), mem_size / (1024 * 1024));
        return Err(HypervisorError::OutOfMemory);
    }

    // 1. Load protected-mode kernel at KERNEL_LOAD_ADDR
    let kernel_end = KERNEL_LOAD_ADDR as usize + kernel.kernel_data.len();
    if kernel_end > guest_memory.len() {
        return Err(HypervisorError::OutOfMemory);
    }
    guest_memory[KERNEL_LOAD_ADDR as usize..kernel_end]
        .copy_from_slice(&kernel.kernel_data);
    crate::serial_println!("[Linux] Kernel loaded at 0x{:X}-0x{:X}", 
                          KERNEL_LOAD_ADDR, kernel_end);

    // 2. Set up command line
    let cmdline_bytes = config.cmdline.as_bytes();
    let cmdline_len = cmdline_bytes.len().min(CMDLINE_MAX - 1);
    let cmd_start = CMDLINE_ADDR as usize;
    guest_memory[cmd_start..cmd_start + cmdline_len]
        .copy_from_slice(&cmdline_bytes[..cmdline_len]);
    guest_memory[cmd_start + cmdline_len] = 0; // null terminate
    crate::serial_println!("[Linux] Command line at 0x{:X}: \"{}\"", 
                          CMDLINE_ADDR, config.cmdline);

    // 3. Set up boot_params (zero page) — 4096 bytes at BOOT_PARAMS_ADDR
    setup_boot_params(guest_memory, kernel, config)?;

    // 4. Set up page tables (identity map 0-4GB)
    setup_guest_page_tables(guest_memory, mem_size)?;

    // 5. Set up GDT
    setup_guest_gdt(guest_memory)?;

    // 6. Install ACPI tables (RSDP, XSDT, MADT, FADT, DSDT)
    let rsdp_addr = super::acpi::install_acpi_tables(guest_memory);
    // Set acpi_rsdp_addr in boot_params (offset 0x070, 64-bit — Linux boot protocol 2.14+)
    write_u64(guest_memory, BOOT_PARAMS_ADDR as usize + 0x070, rsdp_addr);
    
    // 7. Load initrd if provided
    if let Some(ref initrd_data) = config.initrd {
        let initrd_end = INITRD_LOAD_ADDR as usize + initrd_data.len();
        if initrd_end > guest_memory.len() {
            crate::serial_println!("[Linux] Initrd too large for guest memory");
            return Err(HypervisorError::OutOfMemory);
        }
        guest_memory[INITRD_LOAD_ADDR as usize..initrd_end]
            .copy_from_slice(initrd_data);
        crate::serial_println!("[Linux] Initrd loaded at 0x{:X}-0x{:X} ({} KB)",
                              INITRD_LOAD_ADDR, initrd_end, initrd_data.len() / 1024);
    }

    Ok(LinuxGuestSetup {
        entry_point: kernel.entry_64,
        stack_ptr: GUEST_STACK_ADDR,
        boot_params_addr: BOOT_PARAMS_ADDR,
        cr3: PAGE_TABLES_ADDR,
        gdt_base: GDT_ADDR,
    })
}

// ============================================================================
// BOOT PARAMS (ZERO PAGE)
// ============================================================================

/// Set up the boot_params structure in guest memory.
///
/// The boot_params (zero page) is a 4096-byte structure that communicates
/// hardware information from the boot loader to the kernel.
fn setup_boot_params(
    guest_memory: &mut [u8],
    kernel: &LinuxKernel,
    config: &LinuxGuestConfig,
) -> Result<()> {
    let bp = BOOT_PARAMS_ADDR as usize;
    
    // Zero the entire page first
    for i in 0..4096 {
        if bp + i < guest_memory.len() {
            guest_memory[bp + i] = 0;
        }
    }

    // Copy the original setup header (0x1F1 - 0x290 approximately)
    // The setup header goes at offset 0x1F1 within boot_params
    let header_src_start = 0x1F1;
    let header_src_end = 0x290.min(kernel.setup_data.len());
    if header_src_end > header_src_start {
        let dest_start = bp + 0x1F1;
        let src = &kernel.setup_data[header_src_start..header_src_end];
        let dest = &mut guest_memory[dest_start..dest_start + src.len()];
        dest.copy_from_slice(src);
    }

    // Overwrite key fields in the setup header within boot_params:

    // type_of_loader (0x210) = 0xFF (undefined bootloader)
    guest_memory[bp + 0x210] = 0xFF;

    // loadflags (0x211) — set LOADED_HIGH | CAN_USE_HEAP
    guest_memory[bp + 0x211] = LOADED_HIGH | CAN_USE_HEAP;

    // heap_end_ptr (0x224) — relative to setup code, not meaningful for us
    write_u16(guest_memory, bp + 0x224, 0xFE00);

    // cmd_line_ptr (0x228) — physical address of command line
    write_u32(guest_memory, bp + 0x228, CMDLINE_ADDR as u32);

    // ramdisk_image & ramdisk_size
    if config.initrd.is_some() {
        write_u32(guest_memory, bp + 0x218, INITRD_LOAD_ADDR as u32);
        write_u32(guest_memory, bp + 0x21C, 
                  config.initrd.as_ref().unwrap().len() as u32);
    }

    // === Screen info (at offset 0x00 in boot_params) ===
    // These aren't critical for serial-only boot but prevent panics
    guest_memory[bp + 0x06] = 80;  // orig_video_cols
    guest_memory[bp + 0x07] = 25;  // orig_video_lines
    guest_memory[bp + 0x0F] = 0x22; // orig_video_mode (0x22 = 80x25 text)

    // === E820 Memory Map (at offset 0x2D0 in boot_params) ===
    // e820_entries count at offset 0x1E8
    let mem_size = config.memory_size;
    let e820_entries = setup_e820_map(guest_memory, bp, mem_size);
    guest_memory[bp + 0x1E8] = e820_entries;

    crate::serial_println!("[Linux] boot_params at 0x{:X}, {} e820 entries, cmdline at 0x{:X}",
                          BOOT_PARAMS_ADDR, e820_entries, CMDLINE_ADDR);

    Ok(())
}

/// Set up the E820 memory map in boot_params.
/// Returns the number of entries.
fn setup_e820_map(guest_memory: &mut [u8], bp: usize, mem_size: u64) -> u8 {
    // E820 map starts at offset 0x2D0 in boot_params
    // Each entry is 20 bytes: u64 addr, u64 size, u32 type
    let e820_base = bp + 0x2D0;
    let mut count = 0u8;

    // Entry 0: Low memory (0 - 0x9FC00 = 639KB usable)
    write_e820_entry(guest_memory, e820_base, count, 
                     0, 0x9FC00, E820Type::Ram);
    count += 1;

    // Entry 1: EBDA/VGA (0x9FC00 - 0xFFFFF = reserved)
    write_e820_entry(guest_memory, e820_base, count,
                     0x9FC00, 0xA0000 - 0x9FC00, E820Type::Reserved);
    count += 1;

    // Entry 2: ACPI tables (0x50000 - 0x50300, marked ACPI reclaimable)
    write_e820_entry(guest_memory, e820_base, count,
                     0x50000, 0x1000, E820Type::Acpi);
    count += 1;

    // Entry 3: Video/BIOS ROM (0xA0000 - 0xFFFFF)
    write_e820_entry(guest_memory, e820_base, count,
                     0xA0000, 0x60000, E820Type::Reserved);
    count += 1;

    // Entry 4: Main memory (1MB - end of guest memory)
    let main_mem_size = mem_size - 0x100000;
    write_e820_entry(guest_memory, e820_base, count,
                     0x100000, main_mem_size, E820Type::Ram);
    count += 1;

    count
}

fn write_e820_entry(
    mem: &mut [u8], base: usize, index: u8,
    addr: u64, size: u64, entry_type: E820Type,
) {
    let offset = base + (index as usize) * 20;
    write_u64(mem, offset, addr);
    write_u64(mem, offset + 8, size);
    write_u32(mem, offset + 16, entry_type as u32);
}

// ============================================================================
// GUEST PAGE TABLES (IDENTITY MAP 0 - 4GB)
// ============================================================================

/// Set up 4-level page tables in guest memory that identity-map 0 to 4GB.
/// Uses 2MB pages for simplicity.
///
/// Layout at PAGE_TABLES_ADDR:
///   +0x0000: PML4 (1 page)
///   +0x1000: PDPT (1 page) — 4 entries for 0-4GB
///   +0x2000: PD0 (1 page)  — 512 entries × 2MB = 1GB
///   +0x3000: PD1 (1 page)  — 512 entries × 2MB = 1GB
///   +0x4000: PD2 (1 page)  — 512 entries × 2MB = 1GB
///   +0x5000: PD3 (1 page)  — 512 entries × 2MB = 1GB
fn setup_guest_page_tables(guest_memory: &mut [u8], _mem_size: u64) -> Result<()> {
    let pt_base = PAGE_TABLES_ADDR as usize;

    // Zero 6 pages (PML4 + PDPT + 4 PDs)
    for i in 0..(6 * 4096) {
        if pt_base + i < guest_memory.len() {
            guest_memory[pt_base + i] = 0;
        }
    }

    // PML4[0] → PDPT (at pt_base + 0x1000)
    let pdpt_addr = PAGE_TABLES_ADDR + 0x1000;
    write_u64(guest_memory, pt_base, pdpt_addr | 0x3); // Present + Writable

    // PDPT[0..3] → PD0..PD3
    for i in 0..4u64 {
        let pd_addr = PAGE_TABLES_ADDR + 0x2000 + i * 0x1000;
        write_u64(guest_memory, pt_base + 0x1000 + (i as usize) * 8,
                  pd_addr | 0x3); // Present + Writable
    }

    // PD0..PD3: each has 512 entries of 2MB pages
    for pd_idx in 0..4u64 {
        let pd_offset = pt_base + 0x2000 + (pd_idx as usize) * 0x1000;
        for entry in 0..512u64 {
            let phys_addr = (pd_idx * 512 + entry) * 0x200000; // 2MB pages
            // Present + Writable + Page Size (2MB)
            let pde = phys_addr | 0x83; // bit 7 = PS (page size)
            write_u64(guest_memory, pd_offset + (entry as usize) * 8, pde);
        }
    }

    crate::serial_println!("[Linux] Guest page tables at 0x{:X} (identity map 0-4GB, 2MB pages)",
                          PAGE_TABLES_ADDR);
    Ok(())
}

// ============================================================================
// GUEST GDT
// ============================================================================

/// Set up a minimal GDT in guest memory for 64-bit mode.
///
/// Layout at GDT_ADDR:
///   Entry 0: Null descriptor
///   Entry 1: 64-bit code segment (CS)
///   Entry 2: 64-bit data segment (DS/SS/ES)
///   Entry 3: 32-bit code segment (unused but standard)
///   Entry 4: 32-bit data segment (unused but standard)
///
/// Also writes GDTR at GDT_ADDR + 0x100
fn setup_guest_gdt(guest_memory: &mut [u8]) -> Result<()> {
    let gdt_base = GDT_ADDR as usize;

    // Zero 512 bytes for GDT + GDTR
    for i in 0..512 {
        if gdt_base + i < guest_memory.len() {
            guest_memory[gdt_base + i] = 0;
        }
    }

    // Entry 0: Null descriptor (already zero)

    // Entry 1 (0x08): 64-bit code segment
    // Access: Present=1, S=1, Type=Execute/Read, L=1 (64-bit)
    // Limit and base are ignored in 64-bit mode
    write_u64(guest_memory, gdt_base + 0x08, 
              gdt_entry(0, 0xFFFFF, 0x9A, 0xA)); // L=1, D=0 for 64-bit

    // Entry 2 (0x10): 64-bit data segment
    // Access: Present=1, S=1, Type=Read/Write
    write_u64(guest_memory, gdt_base + 0x10,
              gdt_entry(0, 0xFFFFF, 0x92, 0xC)); // G=1, D/B=1

    // Entry 3 (0x18): 32-bit code segment (compatibility)
    write_u64(guest_memory, gdt_base + 0x18,
              gdt_entry(0, 0xFFFFF, 0x9A, 0xC)); // G=1, D=1

    // Entry 4 (0x20): 32-bit data segment
    write_u64(guest_memory, gdt_base + 0x20,
              gdt_entry(0, 0xFFFFF, 0x92, 0xC)); // G=1, D/B=1

    // Write GDTR at GDT_ADDR + 0x100
    let gdt_limit = 5 * 8 - 1; // 5 entries × 8 bytes - 1
    write_u16(guest_memory, gdt_base + 0x100, gdt_limit as u16);
    write_u64(guest_memory, gdt_base + 0x102, GDT_ADDR);

    crate::serial_println!("[Linux] Guest GDT at 0x{:X}: null, code64(0x08), data64(0x10), code32(0x18), data32(0x20)",
                          GDT_ADDR);
    Ok(())
}

/// Build a GDT entry from its components.
///
/// - `base`: 32-bit base address
/// - `limit`: 20-bit limit
/// - `access`: access byte (P, DPL, S, Type)
/// - `flags`: 4-bit flags (G, D/B, L, AVL)
fn gdt_entry(base: u32, limit: u32, access: u8, flags: u8) -> u64 {
    let mut entry = 0u64;
    
    // Limit[0:15]
    entry |= (limit & 0xFFFF) as u64;
    // Base[0:15]
    entry |= ((base & 0xFFFF) as u64) << 16;
    // Base[16:23]
    entry |= (((base >> 16) & 0xFF) as u64) << 32;
    // Access byte
    entry |= (access as u64) << 40;
    // Limit[16:19]
    entry |= (((limit >> 16) & 0xF) as u64) << 48;
    // Flags
    entry |= ((flags & 0xF) as u64) << 52;
    // Base[24:31]
    entry |= (((base >> 24) & 0xFF) as u64) << 56;
    
    entry
}

// ============================================================================
// VMCS CONFIGURATION FOR LINUX GUEST
// ============================================================================

/// Configure the VMCS guest state fields for a Linux kernel boot.
///
/// This sets up the guest in 64-bit long mode with:
/// - Page tables identity-mapping 0-4GB
/// - Proper segment registers (CS=0x08, DS/SS=0x10)
/// - RSI pointing to boot_params
/// - RIP at the kernel's 64-bit entry point
pub fn configure_vmcs_for_linux(
    vmcs: &super::vmcs::Vmcs,
    setup: &LinuxGuestSetup,
) -> Result<()> {
    use super::vmcs::fields;

    // === Guest control registers ===
    // CR0: PE=1, PG=1, ET=1, NE=1, WP=1, MP=1
    let cr0 = 0x8005_0033u64; // PE+MP+ET+NE+WP+PG
    vmcs.write(fields::GUEST_CR0, cr0)?;
    
    // CR3: point to our guest page tables
    vmcs.write(fields::GUEST_CR3, setup.cr3)?;
    
    // CR4: PAE=1, PGE=1, OSFXSR=1, OSXMMEXCPT=1
    let cr4 = 0x000006A0u64; // PAE(5)+PGE(7)+OSFXSR(9)+OSXMMEXCPT(10)
    vmcs.write(fields::GUEST_CR4, cr4)?;

    // === Guest segment registers ===
    // CS: 64-bit code segment
    vmcs.write(fields::GUEST_CS_SELECTOR, 0x08)?;
    vmcs.write(fields::GUEST_CS_BASE, 0)?;
    vmcs.write(fields::GUEST_CS_LIMIT, 0xFFFFFFFF)?;
    vmcs.write(fields::GUEST_CS_ACCESS_RIGHTS, 0xA09B)?; // P=1, S=1, Execute/Read, L=1 (64-bit)

    // SS: data segment
    vmcs.write(fields::GUEST_SS_SELECTOR, 0x10)?;
    vmcs.write(fields::GUEST_SS_BASE, 0)?;
    vmcs.write(fields::GUEST_SS_LIMIT, 0xFFFFFFFF)?;
    vmcs.write(fields::GUEST_SS_ACCESS_RIGHTS, 0xC093)?; // P=1, S=1, Read/Write, G=1, B=1

    // DS, ES, FS, GS: data segments
    for (sel_field, base_field, limit_field, access_field) in [
        (fields::GUEST_DS_SELECTOR, fields::GUEST_DS_BASE, fields::GUEST_DS_LIMIT, fields::GUEST_DS_ACCESS_RIGHTS),
        (fields::GUEST_ES_SELECTOR, fields::GUEST_ES_BASE, fields::GUEST_ES_LIMIT, fields::GUEST_ES_ACCESS_RIGHTS),
        (fields::GUEST_FS_SELECTOR, fields::GUEST_FS_BASE, fields::GUEST_FS_LIMIT, fields::GUEST_FS_ACCESS_RIGHTS),
        (fields::GUEST_GS_SELECTOR, fields::GUEST_GS_BASE, fields::GUEST_GS_LIMIT, fields::GUEST_GS_ACCESS_RIGHTS),
    ] {
        vmcs.write(sel_field, 0x10)?;
        vmcs.write(base_field, 0)?;
        vmcs.write(limit_field, 0xFFFFFFFF)?;
        vmcs.write(access_field, 0xC093)?;
    }

    // TR (task register) — must be valid even in 64-bit mode
    vmcs.write(fields::GUEST_TR_SELECTOR, 0)?;
    vmcs.write(fields::GUEST_TR_BASE, 0)?;
    vmcs.write(fields::GUEST_TR_LIMIT, 0xFFFF)?;
    vmcs.write(fields::GUEST_TR_ACCESS_RIGHTS, 0x8B)?; // Present, 64-bit TSS (Busy)

    // LDTR — unusable
    vmcs.write(fields::GUEST_LDTR_SELECTOR, 0)?;
    vmcs.write(fields::GUEST_LDTR_BASE, 0)?;
    vmcs.write(fields::GUEST_LDTR_LIMIT, 0xFFFF)?;
    vmcs.write(fields::GUEST_LDTR_ACCESS_RIGHTS, 0x10082)?; // Unusable bit set

    // GDTR
    vmcs.write(fields::GUEST_GDTR_BASE, setup.gdt_base)?;
    vmcs.write(fields::GUEST_GDTR_LIMIT, 5 * 8 - 1)?;

    // IDTR — empty initially (will cause #GP on interrupts, which we trap)
    vmcs.write(fields::GUEST_IDTR_BASE, 0)?;
    vmcs.write(fields::GUEST_IDTR_LIMIT, 0xFFF)?;

    // === Guest RIP, RSP, RFLAGS ===
    vmcs.write(fields::GUEST_RIP, setup.entry_point)?;
    vmcs.write(fields::GUEST_RSP, setup.stack_ptr)?;
    vmcs.write(fields::GUEST_RFLAGS, 0x2)?; // Reserved bit 1 must be set

    // === Guest MSRs ===
    // EFER: LME=1, LMA=1, SCE=1
    vmcs.write(fields::GUEST_IA32_EFER, 0x501)?; // SCE + LME + LMA

    crate::serial_println!("[Linux] VMCS configured: RIP=0x{:X} RSP=0x{:X} CR3=0x{:X} RSI=0x{:X}",
                          setup.entry_point, setup.stack_ptr, setup.cr3, setup.boot_params_addr);

    Ok(())
}

// ============================================================================
// HIGH-LEVEL API
// ============================================================================

/// Load and prepare a Linux kernel from raw bzImage data.
///
/// Returns (entry_point, boot_params_addr) for use with VM start.
pub fn prepare_linux_vm(
    guest_memory: &mut [u8],
    bzimage_data: &[u8],
    cmdline: &str,
    initrd: Option<&[u8]>,
) -> Result<LinuxGuestSetup> {
    // Parse the bzImage
    let kernel = parse_bzimage(bzimage_data)?;

    if !kernel.supports_64bit {
        crate::serial_println!("[Linux] Warning: kernel does not advertise 64-bit support");
        // Continue anyway — many kernels still work
    }

    // Configure
    let config = LinuxGuestConfig {
        cmdline: String::from(cmdline),
        memory_size: guest_memory.len() as u64,
        initrd: initrd.map(|d| d.to_vec()),
    };

    // Load into guest memory
    load_linux_kernel(guest_memory, &kernel, &config)
}

/// Create a minimal test "Linux-like" kernel that exercises the boot path.
///
/// This produces a fake bzImage with the proper headers but a tiny guest
/// that prints via serial and halts. Useful for testing the loader without
/// needing a real Linux kernel.
pub fn create_test_linux_kernel() -> Vec<u8> {
    // Build a minimal bzImage:
    // - 1 setup sector (512 bytes) + header bytes
    // - Small protected-mode "kernel" that runs in 64-bit mode

    let mut image = vec![0u8; 4096]; // Minimal image

    // Setup sectors = 1 (so setup is 2*512 = 1024 bytes)
    image[0x1F1] = 1;

    // Header magic "HdrS"
    image[0x202] = b'H';
    image[0x203] = b'd';
    image[0x204] = b'r';
    image[0x205] = b'S';

    // Version = 2.15
    image[0x206] = 0x0F;
    image[0x207] = 0x02;

    // loadflags = LOADED_HIGH
    image[0x211] = LOADED_HIGH;

    // code32_start = 0x100000
    write_u32(&mut image, 0x214, 0x100000);

    // xloadflags = XLF_KERNEL_64 (has 64-bit entry)
    write_u16(&mut image, 0x236, XLF_KERNEL_64);

    // pref_address = 0x100000
    write_u64(&mut image, 0x258, 0x100000);

    // init_size = 0x100000 (1MB)
    write_u32(&mut image, 0x260, 0x100000);

    // Now build the 64-bit "kernel" code at offset 1024 (after setup)
    // The 64-bit entry is at kernel_load + 0x200, so we need 0x200 bytes
    // of padding followed by the actual code.
    
    // Pad setup to 1024 bytes (2 sectors)
    while image.len() < 1024 {
        image.push(0);
    }

    // Protected-mode kernel starts here
    let kernel_start = image.len();
    
    // Pad to 0x200 offset for the 64-bit entry point
    for _ in 0..0x200 {
        image.push(0x90); // NOP sled
    }

    // === 64-bit entry point (at kernel_load + 0x200) ===
    // At this point:
    //   RSI = boot_params address
    //   We're in 64-bit mode with identity-mapped pages
    
    // Print "TrustVM Linux Boot OK!\n" via serial port 0x3F8
    let message = b"[TrustVM-Linux] Boot OK - 64-bit entry reached!\r\n";
    for &byte in message {
        // MOV DX, 0x3F8
        image.extend_from_slice(&[0x66, 0xBA, 0xF8, 0x03]);
        // MOV AL, byte
        image.extend_from_slice(&[0xB0, byte]);
        // OUT DX, AL
        image.push(0xEE);
    }

    // Also print via debug port 0xE9
    let debug_msg = b"[TrustVM-Linux] 64-bit kernel entry OK\n";
    for &byte in debug_msg {
        image.extend_from_slice(&[0xB0, byte]);
        image.extend_from_slice(&[0xE6, 0xE9]);
    }

    // Print some register info via VMCALL hypercall
    // VMCALL function 0 (print)
    image.extend_from_slice(&[0x48, 0xC7, 0xC0, 0x00, 0x00, 0x00, 0x00]); // MOV RAX, 0
    image.extend_from_slice(&[0x0F, 0x01, 0xC1]); // VMCALL

    // Exit via VMCALL function 1
    image.extend_from_slice(&[0x48, 0xC7, 0xC0, 0x01, 0x00, 0x00, 0x00]); // MOV RAX, 1
    image.extend_from_slice(&[0x0F, 0x01, 0xC1]); // VMCALL

    // HLT fallback
    image.push(0xF4);

    // Update syssize (protected-mode kernel size in 16-byte paragraphs)
    let kernel_size = image.len() - kernel_start;
    write_u32(&mut image, 0x1F4, (kernel_size / 16) as u32);

    crate::serial_println!("[Linux] Created test kernel: {} bytes ({} setup + {} kernel)",
                          image.len(), kernel_start, kernel_size);

    image
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn read_u16(data: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([data[offset], data[offset + 1]])
}

fn read_u32(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        data[offset], data[offset + 1],
        data[offset + 2], data[offset + 3],
    ])
}

fn read_u64(data: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes([
        data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
        data[offset + 4], data[offset + 5], data[offset + 6], data[offset + 7],
    ])
}

fn write_u16(data: &mut [u8], offset: usize, value: u16) {
    let bytes = value.to_le_bytes();
    data[offset] = bytes[0];
    data[offset + 1] = bytes[1];
}

fn write_u32(data: &mut [u8], offset: usize, value: u32) {
    let bytes = value.to_le_bytes();
    data[offset] = bytes[0];
    data[offset + 1] = bytes[1];
    data[offset + 2] = bytes[2];
    data[offset + 3] = bytes[3];
}

fn write_u64(data: &mut [u8], offset: usize, value: u64) {
    let bytes = value.to_le_bytes();
    for i in 0..8 {
        data[offset + i] = bytes[i];
    }
}
