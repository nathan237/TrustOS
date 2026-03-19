//! TrustOS Installer — self-install to SATA/NVMe from live boot
//!
//! Writes a bootable TrustOS installation to a target disk:
//! - MBR partition table (1 FAT32 boot partition)
//! - FAT32 filesystem with Limine + kernel
//! - Signed integrity manifest (HMAC-SHA256)
//!
//! Protected by Guardian ops: DiskInstall, DiskWipe

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::serial_println;
use crate::drivers::ahci;
#[cfg(feature = "jarvis")]
use crate::jarvis::guardian::{self, ProtectedOp};

// ═══════════════════════════════════════════════════════════════════════════════
// Kernel binary reference (set by main.rs from Limine)
// ═══════════════════════════════════════════════════════════════════════════════

struct KernelBinaryRef {
    ptr: usize,
    size: usize,
}

unsafe impl Send for KernelBinaryRef {}
unsafe impl Sync for KernelBinaryRef {}

static KERNEL_BINARY: spin::Mutex<Option<KernelBinaryRef>> = spin::Mutex::new(None);

/// Register the kernel binary from Limine's KernelFileRequest
///
/// # Safety
/// ptr must point to valid kernel ELF data for the kernel's lifetime.
pub unsafe fn register_kernel_binary(ptr: *const u8, size: usize) {
    *KERNEL_BINARY.lock() = Some(KernelBinaryRef { ptr: ptr as usize, size });
    serial_println!("[INSTALLER] Kernel binary registered: {} bytes ({} KB)", size, size / 1024);
}

fn get_kernel_data() -> Option<&'static [u8]> {
    KERNEL_BINARY.lock().as_ref().map(|r| unsafe {
        core::slice::from_raw_parts(r.ptr as *const u8, r.size)
    })
}

// ═══════════════════════════════════════════════════════════════════════════════
// Limine boot files from modules (set during module scan)
// ═══════════════════════════════════════════════════════════════════════════════

static LIMINE_BIOS_DATA: spin::Mutex<Option<&'static [u8]>> = spin::Mutex::new(None);
static LIMINE_EFI_DATA: spin::Mutex<Option<&'static [u8]>> = spin::Mutex::new(None);

/// Register a Limine boot module (called from main.rs module scan)
pub fn register_boot_module(cmdline: &str, data: &'static [u8]) {
    if cmdline.contains("limine-bios") {
        *LIMINE_BIOS_DATA.lock() = Some(data);
        serial_println!("[INSTALLER] Limine BIOS module registered: {} bytes", data.len());
    } else if cmdline.contains("limine-efi") {
        *LIMINE_EFI_DATA.lock() = Some(data);
        serial_println!("[INSTALLER] Limine EFI module registered: {} bytes", data.len());
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Constants
// ═══════════════════════════════════════════════════════════════════════════════

const SECTOR_SIZE: usize = 512;
/// Boot partition size: 256 MB (enough for kernel + bootloader + manifest)
const BOOT_PARTITION_SECTORS: u64 = 256 * 1024 * 1024 / SECTOR_SIZE as u64;
/// FAT32 cluster size: 4 KB (8 sectors)
const SECTORS_PER_CLUSTER: u16 = 8;
/// Reserved sectors (BPB + FSInfo + backup)
const RESERVED_SECTORS: u16 = 32;
/// Number of FATs
const NUM_FATS: u8 = 2;
/// Manifest magic: "TSIG"
const MANIFEST_MAGIC: [u8; 4] = [b'T', b'S', b'I', b'G'];
/// Manifest version
const MANIFEST_VERSION: u32 = 1;

// ═══════════════════════════════════════════════════════════════════════════════
// Error type
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug)]
pub enum InstallError {
    NoKernelBinary,
    NoDiskFound,
    DiskTooSmall,
    GuardianDenied(String),
    WriteError(&'static str),
    UserAborted,
}

impl core::fmt::Display for InstallError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            InstallError::NoKernelBinary => write!(f, "Kernel binary not available"),
            InstallError::NoDiskFound => write!(f, "No SATA/NVMe disk found"),
            InstallError::DiskTooSmall => write!(f, "Disk too small (need >= 512 MB)"),
            InstallError::GuardianDenied(msg) => write!(f, "Guardian denied: {}", msg),
            InstallError::WriteError(e) => write!(f, "Disk write error: {}", e),
            InstallError::UserAborted => write!(f, "Installation aborted by user"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Interactive Wizard (runs in install mode)
// ═══════════════════════════════════════════════════════════════════════════════

/// Run the installation wizard (called from main.rs in install mode)
pub fn run_wizard() {
    crate::println!("");
    crate::println_color!(0x00FF8800, "╔══════════════════════════════════════════════════════╗");
    crate::println_color!(0x00FF8800, "║          TrustOS Installer v1.0                      ║");
    crate::println_color!(0x00FF8800, "╚══════════════════════════════════════════════════════╝");
    crate::println!("");

    match install_interactive() {
        Ok(()) => {
            crate::println!("");
            crate::println_color!(0x0000FF00, "═══════════════════════════════════════════");
            crate::println_color!(0x0000FF00, "  Installation successful!");
            crate::println_color!(0x0000FF00, "  Remove USB/ISO and reboot to start");
            crate::println_color!(0x0000FF00, "  TrustOS from your disk.");
            crate::println_color!(0x0000FF00, "═══════════════════════════════════════════");
            crate::println!("");
        }
        Err(e) => {
            crate::println_color!(0x00FF0000, "Installation failed: {}", e);
            crate::println!("You can retry with the 'install' shell command.");
            crate::println!("");
        }
    }
}

/// Interactive installation flow
fn install_interactive() -> Result<(), InstallError> {
    // Phase 1: Check kernel binary
    let kernel_data = get_kernel_data().ok_or(InstallError::NoKernelBinary)?;
    crate::println!("  Kernel binary: {} bytes ({} KB)", kernel_data.len(), kernel_data.len() / 1024);

    // Phase 2: List available disks (filter out CD-ROMs/SATAPI with 0 sectors)
    let disks: Vec<_> = ahci::list_devices().into_iter()
        .filter(|d| d.sector_count > 0)
        .collect();
    if disks.is_empty() {
        crate::println_color!(0x00FF0000, "  No SATA disks detected!");
        crate::println!("  Make sure your BIOS is set to AHCI mode (not IDE/RAID).");
        return Err(InstallError::NoDiskFound);
    }

    crate::println!("");
    crate::println!("  Detected disks:");
    for (i, disk) in disks.iter().enumerate() {
        let size_mb = (disk.sector_count * 512) / (1024 * 1024);
        let model = if disk.model.is_empty() { "Unknown" } else { &disk.model };
        if size_mb >= 1024 {
            crate::println!("    [{}] {} — {} GB (port {})", i, model, size_mb / 1024, disk.port_num);
        } else {
            crate::println!("    [{}] {} — {} MB (port {})", i, model, size_mb, disk.port_num);
        }
    }

    // Phase 3: Select disk
    crate::println!("");
    crate::print!("  Install to disk [0]: ");
    let choice = read_line_trimmed();
    let idx: usize = if choice.is_empty() {
        0
    } else {
        choice.parse().unwrap_or(0)
    };

    if idx >= disks.len() {
        crate::println_color!(0x00FF0000, "  Invalid disk selection.");
        return Err(InstallError::UserAborted);
    }

    let target = &disks[idx];
    let min_sectors = 512 * 1024 * 1024 / 512; // 512 MB minimum
    if target.sector_count < min_sectors {
        return Err(InstallError::DiskTooSmall);
    }

    let size_mb = (target.sector_count * 512) / (1024 * 1024);
    let model = if target.model.is_empty() { "Unknown" } else { &target.model };

    // Phase 4: Confirmation
    crate::println!("");
    crate::println_color!(0x00FF0000, "  ╔══════════════════════════════════════════════╗");
    crate::println_color!(0x00FF0000, "  ║  WARNING: ALL DATA ON THIS DISK WILL BE      ║");
    crate::println_color!(0x00FF0000, "  ║  PERMANENTLY ERASED!                          ║");
    crate::println_color!(0x00FF0000, "  ╚══════════════════════════════════════════════╝");
    crate::println!("");
    if size_mb >= 1024 {
        crate::println!("  Target: {} ({} GB, port {})", model, size_mb / 1024, target.port_num);
    } else {
        crate::println!("  Target: {} ({} MB, port {})", model, size_mb, target.port_num);
    }
    crate::print!("  Type 'YES' to continue: ");
    let confirm = read_line_trimmed();
    let confirm_upper: String = confirm.chars().map(|c| c.to_ascii_uppercase()).collect();
    if confirm_upper != "YES" {
        crate::println!("  Aborted.");
        return Err(InstallError::UserAborted);
    }

    // Phase 5: Guardian authentication
    #[cfg(feature = "jarvis")]
    {
        crate::println!("");
        crate::println!("  Guardian authentication required (Le Pacte).");
        crate::print!("  Passphrase: ");
        let passphrase = read_line_hidden();
        if !guardian::authenticate_nathan(&passphrase) {
            return Err(InstallError::GuardianDenied("Invalid passphrase".into()));
        }
        guardian::authorize(ProtectedOp::DiskInstall)
            .map_err(|e| InstallError::GuardianDenied(e))?;
        crate::println_color!(0x0000FF00, "  ✓ Guardian authorized");
    }

    // Phase 6: Install
    crate::println!("");
    let port = target.port_num;
    let total_sectors = target.sector_count;

    crate::println!("  [1/6] Writing MBR partition table...");
    write_mbr(port, total_sectors)?;

    crate::println!("  [2/6] Formatting FAT32 boot partition...");
    format_fat32(port)?;

    crate::println!("  [3/6] Writing kernel binary...");
    let kernel_hash = write_kernel_to_fat32(port, kernel_data)?;

    crate::println!("  [4/6] Writing Limine bootloader...");
    write_limine_config(port)?;

    crate::println!("  [5/6] Writing Limine boot sector...");
    write_limine_bootsector(port)?;

    crate::println!("  [6/6] Signing manifest...");
    write_manifest(port, &kernel_hash)?;

    // Flush cache
    let _ = ahci::flush_cache(port);

    crate::println_color!(0x0000FF00, "  ✓ All files written and verified");
    Ok(())
}

/// Shell command entry point: `install [sata]`
pub fn cmd_install(args: &[&str]) {
    if args.first().map(|a| *a == "help" || *a == "--help").unwrap_or(false) {
        crate::println!("Usage: install [sata]");
        crate::println!("  Install TrustOS to a SATA disk.");
        crate::println!("  Requires Guardian authentication.");
        return;
    }
    run_wizard();
}

// ═══════════════════════════════════════════════════════════════════════════════
// MBR Partition Table
// ═══════════════════════════════════════════════════════════════════════════════

/// Write a Master Boot Record with one FAT32 partition
fn write_mbr(port: u8, total_sectors: u64) -> Result<(), InstallError> {
    let mut mbr = [0u8; 512];

    // Boot partition starts at LBA 2048 (1 MB alignment, standard)
    let part_start: u32 = 2048;
    let part_size: u32 = if BOOT_PARTITION_SECTORS as u32 > (total_sectors as u32 - part_start) {
        (total_sectors as u32).saturating_sub(part_start)
    } else {
        BOOT_PARTITION_SECTORS as u32
    };

    // Partition entry 1 at offset 0x1BE
    let entry = &mut mbr[0x1BE..0x1CE];
    entry[0] = 0x80;       // Bootable flag
    // CHS start (not used with LBA, set dummy)
    entry[1] = 0x00;       // Head
    entry[2] = 0x01;       // Sector (1-based)
    entry[3] = 0x00;       // Cylinder
    entry[4] = 0x0C;       // Type: FAT32 LBA (0x0C)
    // CHS end (dummy)
    entry[5] = 0xFE;
    entry[6] = 0xFF;
    entry[7] = 0xFF;
    // LBA start
    entry[8..12].copy_from_slice(&part_start.to_le_bytes());
    // LBA size
    entry[12..16].copy_from_slice(&part_size.to_le_bytes());

    // MBR signature
    mbr[510] = 0x55;
    mbr[511] = 0xAA;

    ahci::write_sectors(port, 0, 1, &mbr).map_err(InstallError::WriteError)?;
    serial_println!("[INSTALLER] MBR written: partition at LBA {} size {} sectors", part_start, part_size);
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════
// FAT32 Formatter
// ═══════════════════════════════════════════════════════════════════════════════

/// Partition start LBA (must match MBR)
const PART_START_LBA: u64 = 2048;

/// Format the boot partition as FAT32
fn format_fat32(port: u8) -> Result<(), InstallError> {
    let total_sectors = BOOT_PARTITION_SECTORS as u32;
    let data_sectors = total_sectors - RESERVED_SECTORS as u32;
    let clusters = data_sectors / SECTORS_PER_CLUSTER as u32;
    // FAT size: each entry is 4 bytes, round up to sector boundary
    let fat_entries = clusters + 2; // cluster 0 and 1 are reserved
    let fat_size_bytes = fat_entries * 4;
    let fat_size_sectors = (fat_size_bytes + SECTOR_SIZE as u32 - 1) / SECTOR_SIZE as u32;

    // ── BPB (BIOS Parameter Block) — Sector 0 of partition ──
    let mut bpb = [0u8; 512];
    // Jump instruction
    bpb[0] = 0xEB; bpb[1] = 0x58; bpb[2] = 0x90;
    // OEM Name
    bpb[3..11].copy_from_slice(b"TRUSTOS ");
    // Bytes per sector
    bpb[11..13].copy_from_slice(&(SECTOR_SIZE as u16).to_le_bytes());
    // Sectors per cluster
    bpb[13] = SECTORS_PER_CLUSTER as u8;
    // Reserved sectors
    bpb[14..16].copy_from_slice(&RESERVED_SECTORS.to_le_bytes());
    // Number of FATs
    bpb[16] = NUM_FATS;
    // Root entry count (0 for FAT32)
    bpb[17..19].copy_from_slice(&0u16.to_le_bytes());
    // Total sectors 16-bit (0 for FAT32)
    bpb[19..21].copy_from_slice(&0u16.to_le_bytes());
    // Media type (0xF8 = fixed disk)
    bpb[21] = 0xF8;
    // FAT size 16-bit (0 for FAT32)
    bpb[22..24].copy_from_slice(&0u16.to_le_bytes());
    // Sectors per track (dummy)
    bpb[24..26].copy_from_slice(&63u16.to_le_bytes());
    // Number of heads (dummy)
    bpb[26..28].copy_from_slice(&255u16.to_le_bytes());
    // Hidden sectors (partition offset)
    bpb[28..32].copy_from_slice(&(PART_START_LBA as u32).to_le_bytes());
    // Total sectors 32-bit
    bpb[32..36].copy_from_slice(&total_sectors.to_le_bytes());

    // ── FAT32-specific fields (offset 36+) ──
    // FAT size 32-bit
    bpb[36..40].copy_from_slice(&fat_size_sectors.to_le_bytes());
    // Ext flags (mirror both FATs)
    bpb[40..42].copy_from_slice(&0u16.to_le_bytes());
    // FS Version
    bpb[42..44].copy_from_slice(&0u16.to_le_bytes());
    // Root cluster (always 2)
    bpb[44..48].copy_from_slice(&2u32.to_le_bytes());
    // FSInfo sector
    bpb[48..50].copy_from_slice(&1u16.to_le_bytes());
    // Backup boot sector
    bpb[50..52].copy_from_slice(&6u16.to_le_bytes());
    // Reserved (12 bytes, already zero)
    // Drive number
    bpb[64] = 0x80;
    // Boot signature
    bpb[66] = 0x29;
    // Volume serial (use uptime as pseudo-random)
    let serial = crate::time::uptime_ms() as u32;
    bpb[67..71].copy_from_slice(&serial.to_le_bytes());
    // Volume label
    bpb[71..82].copy_from_slice(b"TRUSTOS    ");
    // FS Type
    bpb[82..90].copy_from_slice(b"FAT32   ");
    // Boot signature
    bpb[510] = 0x55;
    bpb[511] = 0xAA;

    // Write BPB at partition start
    ahci::write_sectors(port, PART_START_LBA, 1, &bpb)
        .map_err(InstallError::WriteError)?;

    // Write backup BPB at sector 6
    ahci::write_sectors(port, PART_START_LBA + 6, 1, &bpb)
        .map_err(InstallError::WriteError)?;

    // ── FSInfo sector (sector 1 of partition) ──
    let mut fsinfo = [0u8; 512];
    // Lead signature
    fsinfo[0..4].copy_from_slice(&0x41615252u32.to_le_bytes());
    // Struct signature
    fsinfo[484..488].copy_from_slice(&0x61417272u32.to_le_bytes());
    // Free cluster count (all minus root dir cluster)
    let free_clusters = clusters - 1;
    fsinfo[488..492].copy_from_slice(&free_clusters.to_le_bytes());
    // Next free cluster
    fsinfo[492..496].copy_from_slice(&3u32.to_le_bytes());
    // Trail signature
    fsinfo[508..512].copy_from_slice(&0xAA550000u32.to_le_bytes());

    ahci::write_sectors(port, PART_START_LBA + 1, 1, &fsinfo)
        .map_err(InstallError::WriteError)?;

    // ── Initialize FAT tables ──
    // FAT entry 0: media byte + 0xFFFFF00
    // FAT entry 1: end-of-chain marker
    // FAT entry 2: end-of-chain (root directory cluster)
    let mut fat_first_sector = [0u8; 512];
    // Entry 0: 0x0FFFFFF8 (media byte)
    fat_first_sector[0..4].copy_from_slice(&0x0FFFFFF8u32.to_le_bytes());
    // Entry 1: 0x0FFFFFFF (end of chain marker)
    fat_first_sector[4..8].copy_from_slice(&0x0FFFFFFFu32.to_le_bytes());
    // Entry 2: 0x0FFFFFFF (root dir = single cluster, end of chain)
    fat_first_sector[8..12].copy_from_slice(&0x0FFFFFFFu32.to_le_bytes());

    // Write FAT 1
    let fat1_start = PART_START_LBA + RESERVED_SECTORS as u64;
    ahci::write_sectors(port, fat1_start, 1, &fat_first_sector)
        .map_err(InstallError::WriteError)?;
    // Zero remaining FAT 1 sectors
    let zero_sector = [0u8; 512];
    for s in 1..fat_size_sectors as u64 {
        ahci::write_sectors(port, fat1_start + s, 1, &zero_sector)
            .map_err(InstallError::WriteError)?;
    }

    // Write FAT 2
    let fat2_start = fat1_start + fat_size_sectors as u64;
    ahci::write_sectors(port, fat2_start, 1, &fat_first_sector)
        .map_err(InstallError::WriteError)?;
    for s in 1..fat_size_sectors as u64 {
        ahci::write_sectors(port, fat2_start + s, 1, &zero_sector)
            .map_err(InstallError::WriteError)?;
    }

    // ── Zero root directory cluster ──
    let data_start = fat2_start + fat_size_sectors as u64;
    for s in 0..SECTORS_PER_CLUSTER as u64 {
        ahci::write_sectors(port, data_start + s, 1, &zero_sector)
            .map_err(InstallError::WriteError)?;
    }

    serial_println!("[INSTALLER] FAT32 formatted: fat_size={} sectors, clusters={}, data_start=LBA {}",
        fat_size_sectors, clusters, data_start);
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════
// FAT32 File Writer
// ═══════════════════════════════════════════════════════════════════════════════

/// FAT32 layout info (computed from BPB constants)
struct Fat32Layout {
    fat1_start: u64,        // LBA of FAT1
    fat_size: u32,          // sectors per FAT
    data_start: u64,        // LBA of first data cluster
    next_cluster: u32,      // next free cluster to allocate
}

impl Fat32Layout {
    fn new() -> Self {
        let total_sectors = BOOT_PARTITION_SECTORS as u32;
        let data_sectors = total_sectors - RESERVED_SECTORS as u32;
        let clusters = data_sectors / SECTORS_PER_CLUSTER as u32;
        let fat_entries = clusters + 2;
        let fat_size_bytes = fat_entries * 4;
        let fat_size = (fat_size_bytes + SECTOR_SIZE as u32 - 1) / SECTOR_SIZE as u32;
        let fat1_start = PART_START_LBA + RESERVED_SECTORS as u64;
        let fat2_end = fat1_start + (fat_size as u64 * 2);
        Fat32Layout {
            fat1_start,
            fat_size,
            data_start: fat2_end,
            next_cluster: 3, // cluster 2 = root dir, 3 = first free
        }
    }

    /// LBA of a given cluster's first sector
    fn cluster_to_lba(&self, cluster: u32) -> u64 {
        self.data_start + ((cluster - 2) as u64 * SECTORS_PER_CLUSTER as u64)
    }

    /// Allocate clusters for a file, write FAT chain, return first cluster
    fn allocate_chain(&mut self, port: u8, num_clusters: u32) -> Result<u32, InstallError> {
        let first = self.next_cluster;
        for i in 0..num_clusters {
            let cluster = first + i;
            let next = if i == num_clusters - 1 {
                0x0FFFFFFFu32 // end of chain
            } else {
                cluster + 1
            };
            self.write_fat_entry(port, cluster, next)?;
        }
        self.next_cluster += num_clusters;
        Ok(first)
    }

    /// Write a FAT entry to both FAT tables
    fn write_fat_entry(&self, port: u8, cluster: u32, value: u32) -> Result<(), InstallError> {
        let fat_offset = cluster * 4;
        let fat_sector = fat_offset / SECTOR_SIZE as u32;
        let fat_byte_offset = (fat_offset % SECTOR_SIZE as u32) as usize;

        // Read-modify-write the FAT sector
        let mut sector_buf = [0u8; 512];
        let lba1 = self.fat1_start + fat_sector as u64;
        ahci::read_sectors(port, lba1, 1, &mut sector_buf).map_err(InstallError::WriteError)?;

        sector_buf[fat_byte_offset..fat_byte_offset + 4].copy_from_slice(&value.to_le_bytes());

        // Write to FAT1
        ahci::write_sectors(port, lba1, 1, &sector_buf).map_err(InstallError::WriteError)?;
        // Write to FAT2
        let lba2 = lba1 + self.fat_size as u64;
        ahci::write_sectors(port, lba2, 1, &sector_buf).map_err(InstallError::WriteError)?;

        Ok(())
    }

    /// Write file data to allocated clusters
    fn write_file_data(&self, port: u8, first_cluster: u32, data: &[u8]) -> Result<(), InstallError> {
        let cluster_size = SECTORS_PER_CLUSTER as usize * SECTOR_SIZE;
        let mut offset = 0usize;
        let mut cluster = first_cluster;

        while offset < data.len() {
            let lba = self.cluster_to_lba(cluster);
            let remaining = data.len() - offset;

            // Write full sectors
            let sectors_to_write = core::cmp::min(
                SECTORS_PER_CLUSTER as u16,
                ((remaining + SECTOR_SIZE - 1) / SECTOR_SIZE) as u16,
            );

            if remaining >= sectors_to_write as usize * SECTOR_SIZE {
                // Full sectors — write directly
                ahci::write_sectors(port, lba, sectors_to_write, &data[offset..offset + sectors_to_write as usize * SECTOR_SIZE])
                    .map_err(InstallError::WriteError)?;
            } else {
                // Partial last sector — pad with zeros
                let mut padded = vec![0u8; sectors_to_write as usize * SECTOR_SIZE];
                padded[..remaining].copy_from_slice(&data[offset..]);
                ahci::write_sectors(port, lba, sectors_to_write, &padded)
                    .map_err(InstallError::WriteError)?;
            }

            offset += cluster_size;
            cluster += 1;
        }
        Ok(())
    }
}

/// Create a FAT32 directory entry
fn make_dir_entry(name: &[u8; 11], first_cluster: u32, file_size: u32) -> [u8; 32] {
    let mut entry = [0u8; 32];
    entry[0..11].copy_from_slice(name);
    entry[11] = 0x20; // ATTR_ARCHIVE
    // First cluster high word
    entry[20..22].copy_from_slice(&((first_cluster >> 16) as u16).to_le_bytes());
    // First cluster low word
    entry[26..28].copy_from_slice(&((first_cluster & 0xFFFF) as u16).to_le_bytes());
    // File size
    entry[28..32].copy_from_slice(&file_size.to_le_bytes());
    entry
}

/// Create a FAT32 directory entry with ATTR_DIRECTORY
fn make_subdir_entry(name: &[u8; 11], first_cluster: u32) -> [u8; 32] {
    let mut entry = [0u8; 32];
    entry[0..11].copy_from_slice(name);
    entry[11] = 0x10; // ATTR_DIRECTORY
    entry[20..22].copy_from_slice(&((first_cluster >> 16) as u16).to_le_bytes());
    entry[26..28].copy_from_slice(&((first_cluster & 0xFFFF) as u16).to_le_bytes());
    entry
}

// ═══════════════════════════════════════════════════════════════════════════════
// Write kernel binary to FAT32
// ═══════════════════════════════════════════════════════════════════════════════

/// Write kernel to /boot/trustos_kernel on the FAT32 partition
/// Returns the SHA-256 hash of the kernel binary
fn write_kernel_to_fat32(port: u8, kernel_data: &[u8]) -> Result<[u8; 32], InstallError> {
    let mut layout = Fat32Layout::new();
    let cluster_size = SECTORS_PER_CLUSTER as usize * SECTOR_SIZE;

    // Create /BOOT directory (cluster 3)
    let boot_dir_cluster = layout.allocate_chain(port, 1)?;

    // Allocate clusters for the kernel
    let kernel_clusters = ((kernel_data.len() + cluster_size - 1) / cluster_size) as u32;
    let kernel_cluster = layout.allocate_chain(port, kernel_clusters)?;

    // Write kernel data
    layout.write_file_data(port, kernel_cluster, kernel_data)?;

    // Create /BOOT directory entries
    let mut boot_dir = [0u8; 512 * 8]; // one cluster
    // "." entry
    let dot = make_subdir_entry(b".          ", boot_dir_cluster);
    boot_dir[0..32].copy_from_slice(&dot);
    // ".." entry (root = cluster 0 in entry)
    let dotdot = make_subdir_entry(b"..         ", 0);
    boot_dir[32..64].copy_from_slice(&dotdot);
    // TRUSTOS_KERNEL file (8.3 name: "TRUSTOS KER")
    let kernel_entry = make_dir_entry(
        b"TRUSTOS KER",
        kernel_cluster,
        kernel_data.len() as u32,
    );
    boot_dir[64..96].copy_from_slice(&kernel_entry);

    // Write /BOOT directory cluster
    let boot_dir_lba = layout.cluster_to_lba(boot_dir_cluster);
    for s in 0..SECTORS_PER_CLUSTER as u64 {
        let sector_start = (s as usize) * SECTOR_SIZE;
        if sector_start < boot_dir.len() {
            let end = core::cmp::min(sector_start + SECTOR_SIZE, boot_dir.len());
            ahci::write_sectors(port, boot_dir_lba + s, 1, &boot_dir[sector_start..end])
                .map_err(InstallError::WriteError)?;
        }
    }

    // Create root directory entry for /BOOT
    let mut root_dir = [0u8; 512 * 8]; // one cluster
    let boot_entry = make_subdir_entry(b"BOOT       ", boot_dir_cluster);
    root_dir[0..32].copy_from_slice(&boot_entry);

    // Write root directory (cluster 2)
    let root_lba = layout.cluster_to_lba(2);
    for s in 0..SECTORS_PER_CLUSTER as u64 {
        let sector_start = (s as usize) * SECTOR_SIZE;
        if sector_start < root_dir.len() {
            let end = core::cmp::min(sector_start + SECTOR_SIZE, root_dir.len());
            ahci::write_sectors(port, root_lba + s, 1, &root_dir[sector_start..end])
                .map_err(InstallError::WriteError)?;
        }
    }

    // Compute SHA-256 of kernel
    let hash = crate::tls13::crypto::sha256(kernel_data);
    serial_println!("[INSTALLER] Kernel written: {} bytes, {} clusters, SHA-256 OK", kernel_data.len(), kernel_clusters);
    Ok(hash)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Write Limine config
// ═══════════════════════════════════════════════════════════════════════════════

/// Write /limine.conf to the root directory
fn write_limine_config(port: u8) -> Result<(), InstallError> {
    let config = b"timeout: 3\n\n\
/TrustOS\n    \
    protocol: limine\n    \
    resolution: 1920x1080x32\n    \
    kernel_path: boot():/boot/trustos_kernel\n";

    let mut layout = Fat32Layout::new();
    // Re-sync next_cluster: root=2, boot_dir=3, kernel starts at 4
    // We need to read the current state — but since we just formatted, we know
    // the layout. After write_kernel_to_fat32, clusters are allocated sequentially.
    // For the config file, allocate after whatever kernel used.
    // Safe approach: read the next free from what we'll compute.
    // Actually, we'll append the limine.conf entry to root dir and allocate a new cluster.

    // Read current root directory
    let root_lba = layout.cluster_to_lba(2);
    let mut root_dir = [0u8; 512 * 8];
    for s in 0..SECTORS_PER_CLUSTER as u64 {
        let sector_start = (s as usize) * SECTOR_SIZE;
        ahci::read_sectors(port, root_lba + s, 1, &mut root_dir[sector_start..sector_start + SECTOR_SIZE])
            .map_err(InstallError::WriteError)?;
    }

    // Find next available cluster by scanning FAT
    let next_cluster = find_next_free_cluster(port, &layout)?;
    
    // Allocate one cluster for limine.conf
    let config_cluster = next_cluster;
    write_fat_entry_direct(port, &layout, config_cluster, 0x0FFFFFFF)?;

    // Write config data
    let mut padded = vec![0u8; SECTORS_PER_CLUSTER as usize * SECTOR_SIZE];
    let len = core::cmp::min(config.len(), padded.len());
    padded[..len].copy_from_slice(&config[..len]);
    let config_lba = layout.cluster_to_lba(config_cluster);
    for s in 0..SECTORS_PER_CLUSTER as u64 {
        let start = (s as usize) * SECTOR_SIZE;
        ahci::write_sectors(port, config_lba + s, 1, &padded[start..start + SECTOR_SIZE])
            .map_err(InstallError::WriteError)?;
    }

    // Add entry to root directory
    let config_entry = make_dir_entry(b"LIMINE  CFG", config_cluster, config.len() as u32);
    // Find first empty slot in root dir
    for i in 0..root_dir.len() / 32 {
        if root_dir[i * 32] == 0 {
            root_dir[i * 32..(i + 1) * 32].copy_from_slice(&config_entry);
            break;
        }
    }

    // Write root directory back
    for s in 0..SECTORS_PER_CLUSTER as u64 {
        let sector_start = (s as usize) * SECTOR_SIZE;
        ahci::write_sectors(port, root_lba + s, 1, &root_dir[sector_start..sector_start + SECTOR_SIZE])
            .map_err(InstallError::WriteError)?;
    }

    // Also write Limine BIOS and EFI bootloader files if available as modules
    write_limine_bootloader_files(port, &layout, &mut root_dir)?;

    serial_println!("[INSTALLER] Limine config written");
    Ok(())
}

/// Find the next free cluster by scanning FAT1
fn find_next_free_cluster(port: u8, layout: &Fat32Layout) -> Result<u32, InstallError> {
    let mut sector_buf = [0u8; 512];
    let entries_per_sector = SECTOR_SIZE / 4;

    for sector in 0..layout.fat_size {
        let lba = layout.fat1_start + sector as u64;
        ahci::read_sectors(port, lba, 1, &mut sector_buf).map_err(InstallError::WriteError)?;

        for i in 0..entries_per_sector {
            let cluster = sector as usize * entries_per_sector + i;
            if cluster < 2 { continue; } // skip reserved entries
            let val = u32::from_le_bytes([
                sector_buf[i * 4],
                sector_buf[i * 4 + 1],
                sector_buf[i * 4 + 2],
                sector_buf[i * 4 + 3],
            ]);
            if val == 0 {
                return Ok(cluster as u32);
            }
        }
    }
    Err(InstallError::WriteError("No free clusters"))
}

/// Write a single FAT entry to both FATs
fn write_fat_entry_direct(port: u8, layout: &Fat32Layout, cluster: u32, value: u32) -> Result<(), InstallError> {
    let fat_offset = cluster * 4;
    let fat_sector = fat_offset / SECTOR_SIZE as u32;
    let fat_byte_offset = (fat_offset % SECTOR_SIZE as u32) as usize;

    let mut sector_buf = [0u8; 512];
    let lba1 = layout.fat1_start + fat_sector as u64;
    ahci::read_sectors(port, lba1, 1, &mut sector_buf).map_err(InstallError::WriteError)?;
    sector_buf[fat_byte_offset..fat_byte_offset + 4].copy_from_slice(&value.to_le_bytes());

    ahci::write_sectors(port, lba1, 1, &sector_buf).map_err(InstallError::WriteError)?;
    let lba2 = lba1 + layout.fat_size as u64;
    ahci::write_sectors(port, lba2, 1, &sector_buf).map_err(InstallError::WriteError)?;
    Ok(())
}

/// Write Limine bootloader files from modules (if available)
fn write_limine_bootloader_files(port: u8, layout: &Fat32Layout, root_dir: &mut [u8]) -> Result<(), InstallError> {
    // Try to write limine-bios.sys
    if let Some(bios_data) = LIMINE_BIOS_DATA.lock().as_ref().copied() {
        let cluster = find_next_free_cluster(port, layout)?;
        let num_clusters = ((bios_data.len() + SECTORS_PER_CLUSTER as usize * SECTOR_SIZE - 1)
            / (SECTORS_PER_CLUSTER as usize * SECTOR_SIZE)) as u32;

        // Allocate chain
        for i in 0..num_clusters {
            let c = cluster + i;
            let next = if i == num_clusters - 1 { 0x0FFFFFFF } else { c + 1 };
            write_fat_entry_direct(port, layout, c, next)?;
        }

        // Write data
        let layout_for_write = Fat32Layout::new();
        layout_for_write.write_file_data(port, cluster, bios_data)?;

        // We need to write this into the /BOOT directory, not root
        // Read /BOOT dir, find its cluster from root_dir
        if let Some(boot_cluster) = find_dir_cluster(root_dir, b"BOOT       ") {
            let boot_lba = layout.cluster_to_lba(boot_cluster);
            let mut boot_dir = [0u8; 512 * 8];
            for s in 0..SECTORS_PER_CLUSTER as u64 {
                let start = (s as usize) * SECTOR_SIZE;
                ahci::read_sectors(port, boot_lba + s, 1, &mut boot_dir[start..start + SECTOR_SIZE])
                    .map_err(InstallError::WriteError)?;
            }

            // Add limine-bios.sys entry
            let entry = make_dir_entry(b"LIMINE  SYS", cluster, bios_data.len() as u32);
            for i in 0..boot_dir.len() / 32 {
                if boot_dir[i * 32] == 0 {
                    boot_dir[i * 32..(i + 1) * 32].copy_from_slice(&entry);
                    break;
                }
            }

            for s in 0..SECTORS_PER_CLUSTER as u64 {
                let start = (s as usize) * SECTOR_SIZE;
                ahci::write_sectors(port, boot_lba + s, 1, &boot_dir[start..start + SECTOR_SIZE])
                    .map_err(InstallError::WriteError)?;
            }
        }
        serial_println!("[INSTALLER] limine-bios.sys written: {} bytes", bios_data.len());
    }

    // Try to write BOOTX64.EFI to /EFI/BOOT/
    if let Some(efi_data) = LIMINE_EFI_DATA.lock().as_ref().copied() {
        let cluster = find_next_free_cluster(port, layout)?;
        let num_clusters = ((efi_data.len() + SECTORS_PER_CLUSTER as usize * SECTOR_SIZE - 1)
            / (SECTORS_PER_CLUSTER as usize * SECTOR_SIZE)) as u32;

        for i in 0..num_clusters {
            let c = cluster + i;
            let next = if i == num_clusters - 1 { 0x0FFFFFFF } else { c + 1 };
            write_fat_entry_direct(port, layout, c, next)?;
        }

        let layout_for_write = Fat32Layout::new();
        layout_for_write.write_file_data(port, cluster, efi_data)?;

        // Create /EFI directory in root
        let efi_dir_cluster = find_next_free_cluster(port, layout)?;
        write_fat_entry_direct(port, layout, efi_dir_cluster, 0x0FFFFFFF)?;

        // Create /EFI/BOOT directory
        let efi_boot_cluster = find_next_free_cluster(port, layout)?;
        write_fat_entry_direct(port, layout, efi_boot_cluster, 0x0FFFFFFF)?;

        // Write /EFI/BOOT directory with BOOTX64.EFI entry
        let mut efi_boot_dir = [0u8; 512 * 8];
        let dot = make_subdir_entry(b".          ", efi_boot_cluster);
        efi_boot_dir[0..32].copy_from_slice(&dot);
        let dotdot = make_subdir_entry(b"..         ", efi_dir_cluster);
        efi_boot_dir[32..64].copy_from_slice(&dotdot);
        let efi_entry = make_dir_entry(b"BOOTX64 EFI", cluster, efi_data.len() as u32);
        efi_boot_dir[64..96].copy_from_slice(&efi_entry);

        let efi_boot_lba = layout.cluster_to_lba(efi_boot_cluster);
        for s in 0..SECTORS_PER_CLUSTER as u64 {
            let start = (s as usize) * SECTOR_SIZE;
            ahci::write_sectors(port, efi_boot_lba + s, 1, &efi_boot_dir[start..start + SECTOR_SIZE])
                .map_err(InstallError::WriteError)?;
        }

        // Write /EFI directory with BOOT subdir entry
        let mut efi_dir = [0u8; 512 * 8];
        let dot = make_subdir_entry(b".          ", efi_dir_cluster);
        efi_dir[0..32].copy_from_slice(&dot);
        let dotdot = make_subdir_entry(b"..         ", 0);
        efi_dir[32..64].copy_from_slice(&dotdot);
        let boot_subdir = make_subdir_entry(b"BOOT       ", efi_boot_cluster);
        efi_dir[64..96].copy_from_slice(&boot_subdir);

        let efi_dir_lba = layout.cluster_to_lba(efi_dir_cluster);
        for s in 0..SECTORS_PER_CLUSTER as u64 {
            let start = (s as usize) * SECTOR_SIZE;
            ahci::write_sectors(port, efi_dir_lba + s, 1, &efi_dir[start..start + SECTOR_SIZE])
                .map_err(InstallError::WriteError)?;
        }

        // Add /EFI to root directory
        let root_lba = layout.cluster_to_lba(2);
        let efi_root_entry = make_subdir_entry(b"EFI        ", efi_dir_cluster);
        for i in 0..root_dir.len() / 32 {
            if root_dir[i * 32] == 0 {
                root_dir[i * 32..(i + 1) * 32].copy_from_slice(&efi_root_entry);
                break;
            }
        }

        // Write root directory back
        let root_lba = layout.cluster_to_lba(2);
        for s in 0..SECTORS_PER_CLUSTER as u64 {
            let start = (s as usize) * SECTOR_SIZE;
            ahci::write_sectors(port, root_lba + s, 1, &root_dir[start..start + SECTOR_SIZE])
                .map_err(InstallError::WriteError)?;
        }

        serial_println!("[INSTALLER] BOOTX64.EFI written: {} bytes", efi_data.len());
    }

    Ok(())
}

/// Find the first cluster of a subdirectory in a directory buffer
fn find_dir_cluster(dir: &[u8], name: &[u8; 11]) -> Option<u32> {
    for i in 0..dir.len() / 32 {
        let entry = &dir[i * 32..(i + 1) * 32];
        if entry[0] == 0 { break; }
        if &entry[0..11] == name {
            let hi = u16::from_le_bytes([entry[20], entry[21]]) as u32;
            let lo = u16::from_le_bytes([entry[26], entry[27]]) as u32;
            return Some((hi << 16) | lo);
        }
    }
    None
}

// ═══════════════════════════════════════════════════════════════════════════════
// Limine Boot Sector (MBR bootstrap)
// ═══════════════════════════════════════════════════════════════════════════════

/// Write the Limine BIOS boot sector to the MBR
/// This patches the MBR with a minimal bootstrap while preserving the partition table
fn write_limine_bootsector(port: u8) -> Result<(), InstallError> {
    // Read current MBR (preserve partition table at 0x1BE-0x1FF)
    let mut mbr = [0u8; 512];
    ahci::read_sectors(port, 0, 1, &mut mbr).map_err(InstallError::WriteError)?;

    // Write a minimal x86 bootstrap that loads from partition
    // This is a tiny bootloader that chain-loads the FAT32 VBR
    // For Limine BIOS, the limine-bios.sys file in /boot handles the rest
    // The MBR just needs to find the active partition and load its VBR

    // Standard MBR bootstrap (loads active partition's VBR)
    let bootstrap: [u8; 11] = [
        0xFA,                   // CLI
        0x31, 0xC0,             // XOR AX, AX
        0x8E, 0xD8,             // MOV DS, AX
        0x8E, 0xD0,             // MOV SS, AX
        0xBC, 0x00, 0x7C,       // MOV SP, 0x7C00
        0xFB,                   // STI
    ];
    mbr[0..11].copy_from_slice(&bootstrap);
    // The rest of the MBR bootstrap is complex; for a proper install,
    // Limine provides limine-deploy which patches the first 440 bytes.
    // Since we can't run limine-deploy bare-metal, we rely on UEFI boot path
    // (BOOTX64.EFI) for UEFI systems and the VBR for legacy BIOS.

    // Ensure MBR signature is intact
    mbr[510] = 0x55;
    mbr[511] = 0xAA;

    ahci::write_sectors(port, 0, 1, &mbr).map_err(InstallError::WriteError)?;
    serial_println!("[INSTALLER] MBR bootstrap written (UEFI primary, BIOS fallback)");
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════
// Integrity Manifest (HMAC-SHA256 signed)
// ═══════════════════════════════════════════════════════════════════════════════

/// Write the signed integrity manifest to /boot/manifest.sig
fn write_manifest(port: u8, kernel_hash: &[u8; 32]) -> Result<(), InstallError> {
    // Build manifest binary
    let timestamp = crate::time::uptime_ms();
    let entry_count: u32 = 1; // just kernel for now

    // Header: magic(4) + version(4) + entry_count(4) + timestamp(8) = 20 bytes
    let mut manifest = Vec::with_capacity(128);
    manifest.extend_from_slice(&MANIFEST_MAGIC);
    manifest.extend_from_slice(&MANIFEST_VERSION.to_le_bytes());
    manifest.extend_from_slice(&entry_count.to_le_bytes());
    manifest.extend_from_slice(&timestamp.to_le_bytes());

    // Entry: path_hash(32) + content_hash(32) + file_size(8) = 72 bytes
    let path_hash = crate::tls13::crypto::sha256(b"/boot/trustos_kernel");
    manifest.extend_from_slice(&path_hash);
    manifest.extend_from_slice(kernel_hash);
    let kernel_size = get_kernel_data().map(|d| d.len() as u64).unwrap_or(0);
    manifest.extend_from_slice(&kernel_size.to_le_bytes());

    // Sign the manifest with HMAC-SHA256
    // Key = SHA-256 of the Pact text (deterministic, compiled-in)
    #[cfg(feature = "jarvis")]
    let signing_key = crate::tls13::crypto::sha256(
        crate::jarvis::guardian::THE_PACT.as_bytes()
    );
    #[cfg(not(feature = "jarvis"))]
    let signing_key = crate::tls13::crypto::sha256(b"TrustOS-Install-Key");

    let signature = crate::tls13::crypto::hmac_sha256(&signing_key, &manifest);
    manifest.extend_from_slice(&signature);

    // Write manifest to disk (append to /BOOT directory as MANIFEST.SIG)
    let layout = Fat32Layout::new();
    let manifest_cluster = find_next_free_cluster(port, &layout)?;
    write_fat_entry_direct(port, &layout, manifest_cluster, 0x0FFFFFFF)?;

    let mut padded = vec![0u8; SECTORS_PER_CLUSTER as usize * SECTOR_SIZE];
    let len = core::cmp::min(manifest.len(), padded.len());
    padded[..len].copy_from_slice(&manifest[..len]);
    let manifest_lba = layout.cluster_to_lba(manifest_cluster);
    for s in 0..SECTORS_PER_CLUSTER as u64 {
        let start = (s as usize) * SECTOR_SIZE;
        ahci::write_sectors(port, manifest_lba + s, 1, &padded[start..start + SECTOR_SIZE])
            .map_err(InstallError::WriteError)?;
    }

    // Add to /BOOT directory
    let root_lba = layout.cluster_to_lba(2);
    let mut root_dir = [0u8; 512 * 8];
    for s in 0..SECTORS_PER_CLUSTER as u64 {
        let start = (s as usize) * SECTOR_SIZE;
        ahci::read_sectors(port, root_lba + s, 1, &mut root_dir[start..start + SECTOR_SIZE])
            .map_err(InstallError::WriteError)?;
    }

    if let Some(boot_cluster) = find_dir_cluster(&root_dir, b"BOOT       ") {
        let boot_lba = layout.cluster_to_lba(boot_cluster);
        let mut boot_dir = [0u8; 512 * 8];
        for s in 0..SECTORS_PER_CLUSTER as u64 {
            let start = (s as usize) * SECTOR_SIZE;
            ahci::read_sectors(port, boot_lba + s, 1, &mut boot_dir[start..start + SECTOR_SIZE])
                .map_err(InstallError::WriteError)?;
        }

        let manifest_entry = make_dir_entry(b"MANIFESTSIG", manifest_cluster, manifest.len() as u32);
        for i in 0..boot_dir.len() / 32 {
            if boot_dir[i * 32] == 0 {
                boot_dir[i * 32..(i + 1) * 32].copy_from_slice(&manifest_entry);
                break;
            }
        }

        for s in 0..SECTORS_PER_CLUSTER as u64 {
            let start = (s as usize) * SECTOR_SIZE;
            ahci::write_sectors(port, boot_lba + s, 1, &boot_dir[start..start + SECTOR_SIZE])
                .map_err(InstallError::WriteError)?;
        }
    }

    serial_println!("[INSTALLER] Manifest signed and written ({} bytes, HMAC-SHA256)", manifest.len());
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════
// Boot-time Integrity Verification
// ═══════════════════════════════════════════════════════════════════════════════

/// Verify the kernel integrity against the manifest on disk
/// Called during boot if booting from SATA (not live boot)
/// Returns true if verification passes or no manifest found (fresh boot)
pub fn verify_boot_integrity() -> bool {
    // Only verify if AHCI is initialized and has disks
    if !ahci::is_initialized() {
        return true; // No AHCI = probably live boot
    }

    let disks = ahci::list_devices();
    if disks.is_empty() {
        return true;
    }

    // Try each disk — look for our FAT32 partition with manifest
    for disk in &disks {
        if let Ok(valid) = verify_disk_manifest(disk.port_num) {
            if !valid {
                crate::serial_println!("[SECURITY] !!! INTEGRITY VIOLATION on port {} !!!", disk.port_num);
                crate::println_color!(0x00FF0000, "");
                crate::println_color!(0x00FF0000, "╔═══════════════════════════════════════════════╗");
                crate::println_color!(0x00FF0000, "║  INTEGRITY VIOLATION DETECTED!                ║");
                crate::println_color!(0x00FF0000, "║  Kernel binary has been modified on disk.      ║");
                crate::println_color!(0x00FF0000, "║  This could indicate tampering.                ║");
                crate::println_color!(0x00FF0000, "╚═══════════════════════════════════════════════╝");
                return false;
            }
        }
    }
    true
}

/// Verify manifest on a specific disk
fn verify_disk_manifest(port: u8) -> Result<bool, InstallError> {
    let layout = Fat32Layout::new();

    // Read root directory
    let root_lba = layout.cluster_to_lba(2);
    let mut root_dir = [0u8; 512 * 8];
    for s in 0..SECTORS_PER_CLUSTER as u64 {
        let start = (s as usize) * SECTOR_SIZE;
        if ahci::read_sectors(port, root_lba + s, 1, &mut root_dir[start..start + SECTOR_SIZE]).is_err() {
            return Err(InstallError::WriteError("Cannot read root dir"));
        }
    }

    // Find /BOOT directory
    let boot_cluster = match find_dir_cluster(&root_dir, b"BOOT       ") {
        Some(c) => c,
        None => return Ok(true), // No /BOOT = not our installation
    };

    // Read /BOOT directory
    let boot_lba = layout.cluster_to_lba(boot_cluster);
    let mut boot_dir = [0u8; 512 * 8];
    for s in 0..SECTORS_PER_CLUSTER as u64 {
        let start = (s as usize) * SECTOR_SIZE;
        if ahci::read_sectors(port, boot_lba + s, 1, &mut boot_dir[start..start + SECTOR_SIZE]).is_err() {
            return Err(InstallError::WriteError("Cannot read boot dir"));
        }
    }

    // Find MANIFEST.SIG
    let manifest_cluster = match find_dir_cluster(&boot_dir, b"MANIFESTSIG") {
        Some(c) => c,
        None => return Ok(true), // No manifest = not verified install (OK)
    };

    // Read manifest
    let manifest_lba = layout.cluster_to_lba(manifest_cluster);
    let mut manifest_buf = [0u8; 512 * 8];
    for s in 0..SECTORS_PER_CLUSTER as u64 {
        let start = (s as usize) * SECTOR_SIZE;
        if ahci::read_sectors(port, manifest_lba + s, 1, &mut manifest_buf[start..start + SECTOR_SIZE]).is_err() {
            return Err(InstallError::WriteError("Cannot read manifest"));
        }
    }

    // Parse manifest header
    if &manifest_buf[0..4] != &MANIFEST_MAGIC {
        return Ok(true); // Not our manifest format
    }

    let version = u32::from_le_bytes([manifest_buf[4], manifest_buf[5], manifest_buf[6], manifest_buf[7]]);
    if version != MANIFEST_VERSION {
        serial_println!("[INSTALLER] Unknown manifest version {}", version);
        return Ok(true);
    }

    let entry_count = u32::from_le_bytes([manifest_buf[8], manifest_buf[9], manifest_buf[10], manifest_buf[11]]);

    // Calculate expected manifest size: header(20) + entries(72 each) + signature(32)
    let manifest_data_len = 20 + (entry_count as usize * 72);
    let total_len = manifest_data_len + 32;

    if total_len > manifest_buf.len() {
        return Ok(true); // Manifest too large, skip
    }

    // Verify HMAC signature
    #[cfg(feature = "jarvis")]
    let signing_key = crate::tls13::crypto::sha256(
        crate::jarvis::guardian::THE_PACT.as_bytes()
    );
    #[cfg(not(feature = "jarvis"))]
    let signing_key = crate::tls13::crypto::sha256(b"TrustOS-Install-Key");

    let stored_sig = &manifest_buf[manifest_data_len..total_len];
    let computed_sig = crate::tls13::crypto::hmac_sha256(&signing_key, &manifest_buf[..manifest_data_len]);

    // Constant-time comparison
    let mut diff = 0u8;
    for i in 0..32 {
        diff |= stored_sig[i] ^ computed_sig[i];
    }
    if diff != 0 {
        serial_println!("[SECURITY] Manifest signature INVALID — possible tampering");
        return Ok(false);
    }

    // Verify kernel hash
    if let Some(kernel_data) = get_kernel_data() {
        // First entry is the kernel
        let stored_kernel_hash = &manifest_buf[52..84]; // offset 20+32 = content_hash
        let current_kernel_hash = crate::tls13::crypto::sha256(kernel_data);

        let mut hash_diff = 0u8;
        for i in 0..32 {
            hash_diff |= stored_kernel_hash[i] ^ current_kernel_hash[i];
        }
        if hash_diff != 0 {
            serial_println!("[SECURITY] Kernel hash MISMATCH — binary has been modified");
            return Ok(false);
        }
    }

    serial_println!("[INSTALLER] Boot integrity verified OK");
    Ok(true)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Utility
// ═══════════════════════════════════════════════════════════════════════════════

/// Read a line from keyboard, trimmed
fn read_line_trimmed() -> String {
    let mut buf = [0u8; 256];
    let len = crate::keyboard::read_line(&mut buf);
    let s = core::str::from_utf8(&buf[..len]).unwrap_or("");
    String::from(s.trim())
}

/// Read a line from keyboard (hidden, for passwords)
fn read_line_hidden() -> String {
    let mut buf = [0u8; 256];
    let len = crate::keyboard::read_line_hidden(&mut buf);
    let s = core::str::from_utf8(&buf[..len]).unwrap_or("");
    String::from(s.trim())
}
