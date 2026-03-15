//! NTFS Read-Only Filesystem Driver
//!
//! Supports reading files and directories from NTFS-formatted volumes.
//! Implements the Master File Table (MFT), attribute parsing, data run
//! decoding, and B-tree index navigation for directory listing.
//!
//! NTFS structure:
//! - Boot sector (sector 0) with BPB
//! - Master File Table (MFT) — the core metadata structure
//! - Every file/directory is an MFT record with attributes
//! - File data stored via data runs (extent-like compressed cluster lists)
//! - Directories use B-tree indexes (INDEX_ROOT + INDEX_ALLOCATION)
//!
//! Limitations:
//! - Read-only (no write support)
//! - No compressed file support ($DATA with compression)
//! - No encrypted file support (EFS)
//! - No sparse file support
//! - No reparse points / symlinks

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::vec;
use alloc::sync::Arc;
use alloc::format;
use spin::Mutex;

use super::{FileOperations, DirectoryOperations, FileSystem, FileType, Stat, DirectoryEntry, VfsResult, VfsError, Ino};
use super::fat32::BlockDevice;

// ============================================================================
// NTFS Constants
// ============================================================================

/// MFT record magic: "FILE"
const MFT_RECORD_MAGIC: u32 = 0x454C4946; // "FILE" in little-endian

/// NTFS OEM ID at boot sector offset 3
const NTFS_OEM_ID: &[u8; 8] = b"NTFS    ";

/// Well-known MFT record numbers
const MFT_RECORD_MFT: u64 = 0;          // $MFT itself
const MFT_RECORD_ROOT: u64 = 5;         // Root directory

/// Attribute type codes
const ATTRIBUTE_STANDARD_INFORMATION: u32 = 0x10;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const ATTRIBUTE_FILE_NAME: u32 = 0x30;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const ATTRIBUTE_DATA: u32 = 0x80;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const ATTRIBUTE_INDEX_ROOT: u32 = 0x90;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const ATTRIBUTE_INDEX_ALLOCATION: u32 = 0xA0;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const ATTRIBUTE_BITMAP: u32 = 0xB0;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const ATTRIBUTE_END: u32 = 0xFFFFFFFF;

/// File name namespace types
const FILE_NAME_POSIX: u8 = 0;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const FILE_NAME_WIN32: u8 = 1;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const FILE_NAME_DOS: u8 = 2;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const FILE_NAME_WIN32_AND_DOS: u8 = 3;

/// MFT record flags
const MFT_RECORD_IN_USE: u16 = 0x0001;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const MFT_RECORD_IS_DIRECTORY: u16 = 0x0002;

/// Index entry flags
const INDEX_ENTRY_SUBNODE: u32 = 0x01;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const INDEX_ENTRY_LAST: u32 = 0x02;

/// Sector size
const SECTOR_SIZE: usize = 512;

// ============================================================================
// NTFS On-Disk Structures
// ============================================================================

/// NTFS Boot Sector (BPB)
#[repr(C, packed)]
// #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone, Copy)]
struct NtfsBootSector {
    jmp_boot: [u8; 3],          // 0x00: Jump instruction
    oem_id: [u8; 8],            // 0x03: "NTFS    "
    bytes_per_sector: u16,      // 0x0B
    sectors_per_cluster: u8,    // 0x0D
    _reserved1: [u8; 7],        // 0x0E
    media_descriptor: u8,       // 0x15
    _reserved2: [u8; 2],        // 0x16
    sectors_per_track: u16,     // 0x18
    number_heads: u16,             // 0x1A
    hidden_sectors: u32,        // 0x1C
    _reserved3: u32,            // 0x20
    _reserved4: u32,            // 0x24
    total_sectors: u64,         // 0x28
    mft_lcn: u64,               // 0x30: MFT start cluster
    mft_mirror_lcn: u64,        // 0x38: MFT mirror start cluster
    mft_record_size: i8,        // 0x40: MFT record size (clusters or -log2 bytes)
    _reserved5: [u8; 3],        // 0x41
    index_block_size: i8,       // 0x44: Index block size
    _reserved6: [u8; 3],        // 0x45
    volume_serial: u64,         // 0x48
}

// Implementation block — defines methods for the type above.
impl NtfsBootSector {
    fn is_valid(&self) -> bool {
        self.oem_id == *NTFS_OEM_ID
    }

    fn cluster_size(&self) -> u32 {
        let bps = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { core::ptr::read_unaligned(core::ptr::address_of!(self.bytes_per_sector)) };
        bps as u32 * self.sectors_per_cluster as u32
    }

    fn mft_record_bytes(&self) -> u32 {
        if self.mft_record_size > 0 {
            self.mft_record_size as u32 * self.cluster_size()
        } else {
            // Negative value means: 2^(-value) bytes
            1u32 << (-(self.mft_record_size as i32) as u32)
        }
    }

    fn index_block_bytes(&self) -> u32 {
        if self.index_block_size > 0 {
            self.index_block_size as u32 * self.cluster_size()
        } else {
            1u32 << (-(self.index_block_size as i32) as u32)
        }
    }

    fn mft_start_byte(&self) -> u64 {
        let lcn = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { core::ptr::read_unaligned(core::ptr::address_of!(self.mft_lcn)) };
        lcn * self.cluster_size() as u64
    }
}

/// MFT Record Header
#[repr(C, packed)]
// #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone, Copy, Debug)]
struct MftRecordHeader {
    magic: u32,                 // 0x00: "FILE"
    update_sequence_offset: u16,     // 0x04: Offset to update sequence
    update_sequence_size: u16,       // 0x06: Size in words of update sequence
    log_sequence_number: u64,        // 0x08: $LogFile sequence number
    sequence_number: u16,       // 0x10: Sequence number (for consistency)
    hard_link_count: u16,       // 0x12
    first_attribute_offset: u16,     // 0x14: Offset to first attribute
    flags: u16,                 // 0x16: Flags (in-use, directory)
    used_size: u32,             // 0x18: Real size of record
    allocated_size: u32,        // 0x1C: Allocated size
    base_record: u64,           // 0x20: Base MFT record (0 if base)
    next_attribute_id: u16,          // 0x28
}

/// Attribute Header (common part)
#[repr(C, packed)]
// #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone, Copy, Debug)]
struct AttributeHeader {
    attribute_type: u32,             // 0x00: Attribute type
    length: u32,                // 0x04: Total length including header
    non_resident: u8,           // 0x08: 0=resident, 1=non-resident
    name_length: u8,            // 0x09: Name length in UTF-16 chars
    name_offset: u16,           // 0x0A: Offset to name
    flags: u16,                 // 0x0C: Flags (compressed, encrypted, sparse)
    attribute_id: u16,               // 0x0E: Attribute ID
}

/// Resident attribute specific header
#[repr(C, packed)]
// #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone, Copy, Debug)]
struct ResidentAttributeHeader {
    value_length: u32,          // 0x10: Length of attribute value
    value_offset: u16,          // 0x14: Offset to value (from attr start)
    indexed_flag: u16,          // 0x16
}

/// Non-resident attribute specific header
#[repr(C, packed)]
// #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone, Copy, Debug)]
struct NonResidentAttributeHeader {
    lowest_vcn: u64,            // 0x10: Lowest VCN covered by this attribute
    highest_vcn: u64,           // 0x18: Highest VCN covered
    data_runs_offset: u16,      // 0x20: Offset to data runs
    compression_unit: u16,      // 0x22: Compression unit size
    _padding: u32,              // 0x24
    allocated_size: u64,        // 0x28: Allocated size on disk
    real_size: u64,             // 0x30: Actual data size
    initialized_size: u64,      // 0x38: Initialized data size
}

/// $FILE_NAME attribute content
#[repr(C, packed)]
// #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone, Copy)]
struct FileNameAttribute {
    parent_ref: u64,            // 0x00: MFT reference to parent directory
    creation_time: u64,         // 0x08
    modification_time: u64,     // 0x10
    mft_modification_time: u64, // 0x18
    access_time: u64,           // 0x20
    allocated_size: u64,        // 0x28
    real_size: u64,             // 0x30
    flags: u32,                 // 0x38
    reparse_or_ea: u32,        // 0x3C
    name_length: u8,            // 0x40: Name length in UTF-16 chars
    namespace: u8,              // 0x41: Namespace (POSIX, Win32, DOS, Win32+DOS)
    // Followed by name_length * 2 bytes of UTF-16LE name
}

/// $STANDARD_INFORMATION attribute (first 48 bytes)
#[repr(C, packed)]
// #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone, Copy)]
struct StdInformationAttribute {
    creation_time: u64,
    modification_time: u64,
    mft_modification_time: u64,
    access_time: u64,
    file_attributes: u32,       // DOS attributes (readonly, hidden, system, etc)
    _padding: [u8; 4],
}

/// $INDEX_ROOT header
#[repr(C, packed)]
// #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone, Copy)]
struct IndexRootHeader {
    attribute_type: u32,             // Attribute type indexed (0x30 for $FILE_NAME)
    collation_rule: u32,        // Collation rule
    index_block_size: u32,      // Size of index allocation entry
    clusters_per_index: u8,     // Clusters per index block
    _padding: [u8; 3],
}

/// Index Node Header (used in both INDEX_ROOT and INDEX_ALLOCATION)
#[repr(C, packed)]
// #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone, Copy)]
struct IndexNodeHeader {
    entries_offset: u32,        // Offset to first index entry (relative to this header)
    total_size: u32,            // Total size of index entries
    allocated_size: u32,        // Allocated size
    flags: u32,                 // 0x01 = has sub-nodes
}

/// Index Entry (variable-length)
#[repr(C, packed)]
// #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone, Copy)]
struct IndexEntryHeader {
    mft_reference: u64,         // MFT record reference
    entry_length: u16,          // Length of this entry
    content_length: u16,        // Length of the content ($FILE_NAME)
    flags: u32,                 // INDEX_ENTRY_SUBNODE | INDEX_ENTRY_LAST
}

/// INDEX_ALLOCATION block header ("INDX")
const INDX_MAGIC: u32 = 0x58444E49; // "INDX" in little-endian

#[repr(C, packed)]
// #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone, Copy)]
struct IndexBlockHeader {
    magic: u32,                 // "INDX"
    update_sequence_offset: u16,
    update_sequence_size: u16,
    log_sequence_number: u64,
    vcn: u64,                   // VCN of this index block
    // Followed by IndexNodeHeader at offset 0x18
}

// ============================================================================
// Data Run Decoding
// ============================================================================

/// A decoded data run (extent)
#[derive(Clone, Debug)]
struct DataRun {
    /// Virtual cluster number (offset from start of data)
    vcn_start: u64,
    /// Length in clusters
    length: u64,
    /// Logical cluster number on disk (0 = sparse/hole)
    lcn: u64,
}

/// Decode NTFS data runs from raw bytes
fn decode_data_runs(data: &[u8]) -> Vec<DataRun> {
    let mut runs = Vec::new();
    let mut offset = 0usize;
    let mut current_vcn: u64 = 0;
    let mut current_lcn: i64 = 0;

    while offset < data.len() {
        let header = data[offset];
        if header == 0 {
            break; // End of data runs
        }

        let length_size = (header & 0x0F) as usize;
        let offset_size = ((header >> 4) & 0x0F) as usize;
        offset += 1;

        if length_size == 0 || offset + length_size + offset_size > data.len() {
            break;
        }

        // Read run length (unsigned)
        let mut run_length: u64 = 0;
        for i in 0..length_size {
            run_length |= (data[offset + i] as u64) << (i * 8);
        }
        offset += length_size;

        // Read run offset (signed, relative to previous LCN)
        let mut run_offset: i64 = 0;
        if offset_size > 0 {
            for i in 0..offset_size {
                run_offset |= (data[offset + i] as i64) << (i * 8);
            }
            // Sign-extend
            let sign_bit = 1i64 << (offset_size * 8 - 1);
            if run_offset & sign_bit != 0 {
                run_offset |= !((1i64 << (offset_size * 8)) - 1);
            }
            offset += offset_size;

            current_lcn += run_offset;
        }

        let lcn = if offset_size == 0 {
            0 // Sparse run
        } else {
            current_lcn as u64
        };

        runs.push(DataRun {
            vcn_start: current_vcn,
            length: run_length,
            lcn,
        });

        current_vcn += run_length;
    }

    runs
}

// ============================================================================
// Parsed MFT Record
// ============================================================================

/// Parsed MFT record with extracted attributes
#[derive(Clone)]
struct MftRecord {
    /// MFT record number
    record_number: u64,
    /// Flags
    flags: u16,
    /// File name (best available: Win32 > Win32+DOS > POSIX > DOS)
    file_name: String,
    /// Parent directory MFT reference
    parent_ref: u64,
    /// File size (from $DATA real_size or $FILE_NAME real_size)
    file_size: u64,
    /// Is directory
    is_directory: bool,
    /// Timestamps (from $STANDARD_INFORMATION)
    creation_time: u64,
    modification_time: u64,
    access_time: u64,
    /// DOS file attributes
    file_attributes: u32,
    /// $DATA runs (for file reading)
    data_runs: Vec<DataRun>,
    /// $DATA is resident (small file stored inside MFT record)
    data_resident: bool,
    /// Resident data content (if data_resident)
    resident_data: Vec<u8>,
    /// $INDEX_ROOT content (for directories)
    index_root_data: Vec<u8>,
    /// $INDEX_ALLOCATION data runs (for directories with many entries)
    index_allocator_runs: Vec<DataRun>,
}

// ============================================================================
// NTFS Filesystem Driver
// ============================================================================

/// NTFS filesystem instance (read-only)
pub struct NtfsFilesystem {
    inner: Mutex<NtfsFilesystemInner>,
}

struct NtfsFilesystemInner {
    device: Arc<dyn BlockDevice>,
    cluster_size: u32,
    mft_record_size: u32,
    index_block_size: u32,
    mft_start_byte: u64,
    sectors_per_cluster: u8,
    bytes_per_sector: u16,
    /// Data runs for the $MFT itself (to locate MFT records beyond the first few)
    mft_data_runs: Vec<DataRun>,
}

// Implementation block — defines methods for the type above.
impl NtfsFilesystemInner {
    /// Read raw bytes from device at byte offset
    fn read_bytes(&self, byte_offset: u64, buffer: &mut [u8]) -> Result<(), ()> {
        let sector_size = self.device.sector_size() as u64;
        let start_sector = byte_offset / sector_size;
        let offset_in_sector = (byte_offset % sector_size) as usize;

        let total_bytes = offset_in_sector + buffer.len();
        let number_sectors = (total_bytes + sector_size as usize - 1) / sector_size as usize;

        let mut remaining = buffer.len();
        let mut buffer_offset = 0usize;
        let mut sector_buffer = vec![0u8; sector_size as usize];

        for i in 0..number_sectors {
            self.device.read_sector(start_sector + i as u64, &mut sector_buffer)?;

            let source_start = if i == 0 { offset_in_sector } else { 0 };
            let copy_length = (sector_size as usize - source_start).minimum(remaining);

            buffer[buffer_offset..buffer_offset + copy_length]
                .copy_from_slice(&sector_buffer[source_start..source_start + copy_length]);

            buffer_offset += copy_length;
            remaining -= copy_length;
        }

        Ok(())
    }

    /// Read clusters from the disk
    fn read_clusters(&self, lcn: u64, count: u64, buffer: &mut [u8]) -> Result<(), ()> {
        let byte_offset = lcn * self.cluster_size as u64;
        let byte_length = count as usize * self.cluster_size as usize;
        if buffer.len() < byte_length {
            return Err(());
        }
        self.read_bytes(byte_offset, &mut buffer[..byte_length])
    }

    /// Apply fixup array to an MFT record buffer
    fn apply_fixups(&self, buffer: &mut [u8], record_size: usize) -> Result<(), ()> {
        if buffer.len() < 6 {
            return Err(());
        }
        let update_sequence_offset = u16::from_le_bytes([buffer[4], buffer[5]]) as usize;
        let update_sequence_size = u16::from_le_bytes([buffer[6], buffer[7]]) as usize;

        if update_sequence_size < 2 || update_sequence_offset + update_sequence_size * 2 > buffer.len() {
            return Err(());
        }

        // First word is the expected signature
        let signature = u16::from_le_bytes([
            buffer[update_sequence_offset],
            buffer[update_sequence_offset + 1],
        ]);

        // Replace the last 2 bytes of each sector with the stored values
        let sector_size = self.bytes_per_sector as usize;
        for i in 1..update_sequence_size {
            let sector_end = i * sector_size;
            if sector_end > record_size || sector_end < 2 {
                break;
            }
            let fixup_position = sector_end - 2;

            // Verify the sector ends with the expected signature
            let stored = u16::from_le_bytes([buffer[fixup_position], buffer[fixup_position + 1]]);
            if stored != signature {
                return Err(()); // Fixup mismatch — corrupt record
            }

            // Replace with the saved original bytes
            let saved_offset = update_sequence_offset + i * 2;
            if saved_offset + 1 < buffer.len() {
                buffer[fixup_position] = buffer[saved_offset];
                buffer[fixup_position + 1] = buffer[saved_offset + 1];
            }
        }

        Ok(())
    }

    /// Read an MFT record by record number
    fn read_mft_record_raw(&self, record_number: u64) -> Result<Vec<u8>, ()> {
        let record_size = self.mft_record_size as usize;
        let mut buffer = vec![0u8; record_size];

        // Calculate which VCN this record falls in
        let byte_offset_in_mft = record_number * record_size as u64;
        let vcn = byte_offset_in_mft / self.cluster_size as u64;
        let offset_in_cluster = (byte_offset_in_mft % self.cluster_size as u64) as usize;

        // Find the data run that contains this VCN
        let mut bytes_remaining = record_size;
        let mut buffer_position = 0;
        let mut current_vcn = vcn;
        let mut current_offset = offset_in_cluster;

        while bytes_remaining > 0 {
            // Find the run containing current_vcn
            let run = self.mft_data_runs.iter().find(|r| {
                current_vcn >= r.vcn_start && current_vcn < r.vcn_start + r.length
            });

                        // Pattern matching — Rust's exhaustive branching construct.
match run {
                Some(run) => {
                    let vcn_in_run = current_vcn - run.vcn_start;
                    let lcn = run.lcn + vcn_in_run;
                    let byte_offset = lcn * self.cluster_size as u64 + current_offset as u64;

                    let available_in_cluster = self.cluster_size as usize - current_offset;
                    let to_read = bytes_remaining.minimum(available_in_cluster);

                    self.read_bytes(byte_offset, &mut buffer[buffer_position..buffer_position + to_read])?;

                    buffer_position += to_read;
                    bytes_remaining -= to_read;
                    current_offset = 0;
                    current_vcn += 1;
                }
                None => return Err(()),
            }
        }

        // Apply fixup array
        self.apply_fixups(&mut buffer, record_size)?;

        // Verify magic
        let magic = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
        if magic != MFT_RECORD_MAGIC {
            return Err(());
        }

        Ok(buffer)
    }

    /// Parse attributes from a raw MFT record buffer
    fn parse_mft_record(&self, record_number: u64, buffer: &[u8]) -> Result<MftRecord, ()> {
        let header = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
            core::ptr::read_unaligned(buffer.as_pointer() as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const MftRecordHeader)
        };

        let flags = header.flags;
        let is_directory = (flags & MFT_RECORD_IS_DIRECTORY) != 0;
        let first_attribute = header.first_attribute_offset as usize;
        let used_size = header.used_size as usize;

        let mut file_name = String::new();
        let mut best_namespace: Option<u8> = None;
        let mut parent_ref: u64 = 0;
        let mut file_size: u64 = 0;
        let mut creation_time: u64 = 0;
        let mut modification_time: u64 = 0;
        let mut access_time: u64 = 0;
        let mut file_attributes: u32 = 0;
        let mut data_runs = Vec::new();
        let mut data_resident = false;
        let mut resident_data = Vec::new();
        let mut index_root_data = Vec::new();
        let mut index_allocator_runs = Vec::new();

        let mut offset = first_attribute;
        let limit = used_size.minimum(buffer.len());

        while offset + 4 <= limit {
            let attribute_type = u32::from_le_bytes([
                buffer[offset], buffer[offset + 1], buffer[offset + 2], buffer[offset + 3],
            ]);

            if attribute_type == ATTRIBUTE_END || attribute_type == 0 {
                break;
            }

            if offset + 8 > limit {
                break;
            }

            let attribute_length = u32::from_le_bytes([
                buffer[offset + 4], buffer[offset + 5], buffer[offset + 6], buffer[offset + 7],
            ]) as usize;

            if attribute_length < 16 || attribute_length > limit - offset {
                break;
            }

            let non_resident = buffer[offset + 8];
            let name_length = buffer[offset + 9] as usize;

            // We only care about unnamed attributes (the default data stream)
            let is_unnamed = name_length == 0;

                        // Pattern matching — Rust's exhaustive branching construct.
match attribute_type {
                ATTRIBUTE_STANDARD_INFORMATION if non_resident == 0 => {
                    // Resident $STANDARD_INFORMATION
                    if offset + 24 <= limit {
                        let value_length = u32::from_le_bytes([
                            buffer[offset + 16], buffer[offset + 17],
                            buffer[offset + 18], buffer[offset + 19],
                        ]) as usize;
                        let value_off = u16::from_le_bytes([
                            buffer[offset + 20], buffer[offset + 21],
                        ]) as usize;
                        let data_start = offset + value_off;
                        if value_length >= 48 && data_start + 48 <= buffer.len() {
                            let si = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
                                core::ptr::read_unaligned(
                                    buffer[data_start..].as_pointer() as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const StdInformationAttribute
                                )
                            };
                            creation_time = si.creation_time;
                            modification_time = si.modification_time;
                            access_time = si.access_time;
                            file_attributes = si.file_attributes;
                        }
                    }
                }

                ATTRIBUTE_FILE_NAME if non_resident == 0 => {
                    // Resident $FILE_NAME
                    if offset + 24 <= limit {
                        let value_length = u32::from_le_bytes([
                            buffer[offset + 16], buffer[offset + 17],
                            buffer[offset + 18], buffer[offset + 19],
                        ]) as usize;
                        let value_off = u16::from_le_bytes([
                            buffer[offset + 20], buffer[offset + 21],
                        ]) as usize;
                        let data_start = offset + value_off;
                        if value_length >= 66 && data_start + 66 <= buffer.len() {
                            let fn_attribute = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
                                core::ptr::read_unaligned(
                                    buffer[data_start..].as_pointer() as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const FileNameAttribute
                                )
                            };
                            let ns = fn_attribute.namespace;
                            let name_chars = fn_attribute.name_length as usize;
                            let name_start = data_start + 66;

                            // Pick the best namespace name:
                            // Win32 (1) > Win32+DOS (3) > POSIX (0) > DOS (2)
                            let priority = // Pattern matching — Rust's exhaustive branching construct.
match ns {
                                FILE_NAME_WIN32 => 4,
                                FILE_NAME_WIN32_AND_DOS => 3,
                                FILE_NAME_POSIX => 2,
                                FILE_NAME_DOS => 1,
                                _ => 0,
                            };
                            let current_priority = best_namespace.map(|n| // Pattern matching — Rust's exhaustive branching construct.
match n {
                                FILE_NAME_WIN32 => 4,
                                FILE_NAME_WIN32_AND_DOS => 3,
                                FILE_NAME_POSIX => 2,
                                FILE_NAME_DOS => 1,
                                _ => 0,
                            }).unwrap_or(0);

                            if priority > current_priority {
                                if name_start + name_chars * 2 <= buffer.len() {
                                    file_name = decode_utf16le(
                                        &buffer[name_start..name_start + name_chars * 2]
                                    );
                                    best_namespace = Some(ns);
                                    parent_ref = fn_attribute.parent_ref & 0x0000FFFFFFFFFFFF;

                                    // Use file size from FILE_NAME if no $DATA yet
                                    if file_size == 0 {
                                        file_size = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
                                            core::ptr::read_unaligned(
                                                core::ptr::address_of!(fn_attribute.real_size)
                                            )
                                        };
                                    }
                                }
                            }
                        }
                    }
                }

                ATTRIBUTE_DATA if is_unnamed => {
                    if non_resident == 0 {
                        // Resident $DATA — small file stored inside MFT record
                        data_resident = true;
                        if offset + 24 <= limit {
                            let value_length = u32::from_le_bytes([
                                buffer[offset + 16], buffer[offset + 17],
                                buffer[offset + 18], buffer[offset + 19],
                            ]) as usize;
                            let value_off = u16::from_le_bytes([
                                buffer[offset + 20], buffer[offset + 21],
                            ]) as usize;
                            let data_start = offset + value_off;
                            if data_start + value_length <= buffer.len() {
                                resident_data = buffer[data_start..data_start + value_length].to_vec();
                                file_size = value_length as u64;
                            }
                        }
                    } else {
                        // Non-resident $DATA — decode data runs
                        data_resident = false;
                        if offset + 64 <= limit {
                            let number_header = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
                                core::ptr::read_unaligned(
                                    buffer[offset + 16..].as_pointer() as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const NonResidentAttributeHeader
                                )
                            };
                            file_size = number_header.real_size;
                            let runs_offset = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
                                core::ptr::read_unaligned(
                                    core::ptr::address_of!(number_header.data_runs_offset)
                                )
                            } as usize;
                            let runs_start = offset + runs_offset;
                            if runs_start < offset + attribute_length {
                                data_runs = decode_data_runs(
                                    &buffer[runs_start..offset + attribute_length]
                                );
                            }
                        }
                    }
                }

                ATTRIBUTE_INDEX_ROOT if non_resident == 0 => {
                    // Resident $INDEX_ROOT
                    if offset + 24 <= limit {
                        let value_length = u32::from_le_bytes([
                            buffer[offset + 16], buffer[offset + 17],
                            buffer[offset + 18], buffer[offset + 19],
                        ]) as usize;
                        let value_off = u16::from_le_bytes([
                            buffer[offset + 20], buffer[offset + 21],
                        ]) as usize;
                        let data_start = offset + value_off;
                        if data_start + value_length <= buffer.len() {
                            index_root_data = buffer[data_start..data_start + value_length].to_vec();
                        }
                    }
                }

                ATTRIBUTE_INDEX_ALLOCATION if non_resident != 0 => {
                    // Non-resident $INDEX_ALLOCATION
                    if offset + 64 <= limit {
                        let number_header = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
                            core::ptr::read_unaligned(
                                buffer[offset + 16..].as_pointer() as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const NonResidentAttributeHeader
                            )
                        };
                        let runs_offset = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
                            core::ptr::read_unaligned(
                                core::ptr::address_of!(number_header.data_runs_offset)
                            )
                        } as usize;
                        let runs_start = offset + runs_offset;
                        if runs_start < offset + attribute_length {
                            index_allocator_runs = decode_data_runs(
                                &buffer[runs_start..offset + attribute_length]
                            );
                        }
                    }
                }

                _ => {}
            }

            offset += attribute_length;
        }

        Ok(MftRecord {
            record_number: record_number,
            flags,
            file_name,
            parent_ref,
            file_size,
            is_directory,
            creation_time,
            modification_time,
            access_time,
            file_attributes,
            data_runs,
            data_resident,
            resident_data,
            index_root_data,
            index_allocator_runs,
        })
    }

    /// Read and parse an MFT record by number
    fn read_mft_record(&self, record_number: u64) -> Result<MftRecord, ()> {
        let raw = self.read_mft_record_raw(record_number)?;
        self.parse_mft_record(record_number, &raw)
    }

    /// Read file data using data runs
    fn read_file_data(
        &self,
        record: &MftRecord,
        file_offset: u64,
        buffer: &mut [u8],
    ) -> Result<usize, ()> {
        if file_offset >= record.file_size {
            return Ok(0);
        }

        let read_length = ((record.file_size - file_offset) as usize).minimum(buffer.len());
        if read_length == 0 {
            return Ok(0);
        }

        if record.data_resident {
            // Resident data — copy directly from MFT record
            let start = file_offset as usize;
            let end = start + read_length;
            if end <= record.resident_data.len() {
                buffer[..read_length].copy_from_slice(&record.resident_data[start..end]);
            } else {
                let avail = record.resident_data.len().saturating_sub(start);
                buffer[..avail].copy_from_slice(&record.resident_data[start..start + avail]);
            }
            return Ok(read_length);
        }

        // Non-resident data — follow data runs
        let cluster_size = self.cluster_size as u64;
        let mut remaining = read_length;
        let mut buffer_offset = 0usize;
        let mut offset = file_offset;

        while remaining > 0 {
            let vcn = offset / cluster_size;
            let offset_in_cluster = (offset % cluster_size) as usize;

            // Find the run containing this VCN
            let run = record.data_runs.iter().find(|r| {
                vcn >= r.vcn_start && vcn < r.vcn_start + r.length
            });

                        // Pattern matching — Rust's exhaustive branching construct.
match run {
                Some(run) if run.lcn > 0 => {
                    let vcn_offset = vcn - run.vcn_start;
                    let lcn = run.lcn + vcn_offset;
                    let byte_offset = lcn * cluster_size + offset_in_cluster as u64;

                    let available = cluster_size as usize - offset_in_cluster;
                    let to_read = remaining.minimum(available);

                    self.read_bytes(byte_offset, &mut buffer[buffer_offset..buffer_offset + to_read])?;

                    buffer_offset += to_read;
                    offset += to_read as u64;
                    remaining -= to_read;
                }
                Some(_) => {
                    // Sparse run — zero fill
                    let available = cluster_size as usize - offset_in_cluster;
                    let to_fill = remaining.minimum(available);
                    for b in &mut buffer[buffer_offset..buffer_offset + to_fill] {
                        *b = 0;
                    }
                    buffer_offset += to_fill;
                    offset += to_fill as u64;
                    remaining -= to_fill;
                }
                None => {
                    // Beyond data runs — zero fill rest
                    for b in &mut buffer[buffer_offset..buffer_offset + remaining] {
                        *b = 0;
                    }
                    remaining = 0;
                }
            }
        }

        Ok(read_length)
    }

    /// Read clusters from data runs (for index allocation)
    fn read_from_runs(&self, runs: &[DataRun], vcn: u64, buffer: &mut [u8]) -> Result<(), ()> {
        let run = runs.iter().find(|r| {
            vcn >= r.vcn_start && vcn < r.vcn_start + r.length
        });

                // Pattern matching — Rust's exhaustive branching construct.
match run {
            Some(run) if run.lcn > 0 => {
                let vcn_offset = vcn - run.vcn_start;
                let lcn = run.lcn + vcn_offset;
                let clusters_needed = (buffer.len() + self.cluster_size as usize - 1)
                    / self.cluster_size as usize;
                // Read cluster by cluster
                for i in 0..clusters_needed {
                    let byte_off = (lcn + i as u64) * self.cluster_size as u64;
                    let buffer_start = i * self.cluster_size as usize;
                    let buffer_end = (buffer_start + self.cluster_size as usize).minimum(buffer.len());
                    self.read_bytes(byte_off, &mut buffer[buffer_start..buffer_end])?;
                }
                Ok(())
            }
            _ => Err(()),
        }
    }

    /// Parse directory entries from $INDEX_ROOT and $INDEX_ALLOCATION
    fn read_directory_entries(&self, record: &MftRecord) -> Result<Vec<(u64, String, bool)>, ()> {
        let mut entries = Vec::new();

        // Parse $INDEX_ROOT (always present for directories)
        if record.index_root_data.len() >= 32 {
            let ir_data = &record.index_root_data;

            // IndexRootHeader is 16 bytes, followed by IndexNodeHeader (16 bytes)
            let node_offset = 16; // Skip IndexRootHeader
            if node_offset + 16 <= ir_data.len() {
                let entries_offset = u32::from_le_bytes([
                    ir_data[node_offset], ir_data[node_offset + 1],
                    ir_data[node_offset + 2], ir_data[node_offset + 3],
                ]) as usize;
                let total_size = u32::from_le_bytes([
                    ir_data[node_offset + 4], ir_data[node_offset + 5],
                    ir_data[node_offset + 6], ir_data[node_offset + 7],
                ]) as usize;

                let start = node_offset + entries_offset;
                let end = (node_offset + total_size).minimum(ir_data.len());

                self.parse_index_entries(&ir_data[start..end], &mut entries);
            }
        }

        // Parse $INDEX_ALLOCATION (for large directories)
        if !record.index_allocator_runs.is_empty() {
            let index_block_size = self.index_block_size as usize;
            let clusters_per_block = (index_block_size + self.cluster_size as usize - 1)
                / self.cluster_size as usize;

            // Calculate total VCNs available
            let total_vcns: u64 = record.index_allocator_runs.iter()
                .map(|r| r.length)
                .sum();

            let mut vcn: u64 = 0;
            while vcn < total_vcns {
                let mut block_buffer = vec![0u8; index_block_size];
                if self.read_from_runs(&record.index_allocator_runs, vcn, &mut block_buffer).is_ok() {
                    // Apply fixups to the index block
                    let _ = self.apply_fixups(&mut block_buffer, index_block_size);

                    let magic = u32::from_le_bytes([
                        block_buffer[0], block_buffer[1], block_buffer[2], block_buffer[3],
                    ]);
                    if magic == INDX_MAGIC {
                        // IndexNodeHeader starts at offset 0x18 in the INDX block
                        let node_off = 0x18;
                        if node_off + 16 <= block_buffer.len() {
                            let eo = u32::from_le_bytes([
                                block_buffer[node_off], block_buffer[node_off + 1],
                                block_buffer[node_off + 2], block_buffer[node_off + 3],
                            ]) as usize;
                            let ts = u32::from_le_bytes([
                                block_buffer[node_off + 4], block_buffer[node_off + 5],
                                block_buffer[node_off + 6], block_buffer[node_off + 7],
                            ]) as usize;

                            let start = node_off + eo;
                            let end = (node_off + ts).minimum(block_buffer.len());
                            if start < end {
                                self.parse_index_entries(&block_buffer[start..end], &mut entries);
                            }
                        }
                    }
                }
                vcn += clusters_per_block as u64;
            }
        }

        Ok(entries)
    }

    /// Parse index entries from a buffer
    fn parse_index_entries(
        &self,
        data: &[u8],
        entries: &mut Vec<(u64, String, bool)>,
    ) {
        let mut position = 0;
        while position + 16 <= data.len() {
            let entry_header = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
                core::ptr::read_unaligned(data[position..].as_pointer() as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const IndexEntryHeader)
            };

            let entry_length = entry_header.entry_length as usize;
            let content_length = entry_header.content_length as usize;
            let flags = entry_header.flags;

            if entry_length < 16 || entry_length > data.len() - position {
                break;
            }

            if (flags & INDEX_ENTRY_LAST) != 0 {
                break; // Last entry marker
            }

            if content_length >= 66 {
                // Content is a $FILE_NAME structure
                let content_start = position + 16; // After IndexEntryHeader
                if content_start + content_length <= data.len() {
                    let fn_data = &data[content_start..content_start + content_length];
                    if fn_data.len() >= 66 {
                        let fn_attribute = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
                            core::ptr::read_unaligned(
                                fn_data.as_pointer() as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const FileNameAttribute
                            )
                        };

                        let ns = fn_attribute.namespace;
                        // Skip DOS-only names
                        if ns != FILE_NAME_DOS {
                            let name_chars = fn_attribute.name_length as usize;
                            let name_start = 66;
                            if name_start + name_chars * 2 <= fn_data.len() {
                                let name = decode_utf16le(
                                    &fn_data[name_start..name_start + name_chars * 2]
                                );

                                let mft_ref = entry_header.mft_reference & 0x0000FFFFFFFFFFFF;
                                let flags_value = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
                                    core::ptr::read_unaligned(
                                        core::ptr::address_of!(fn_attribute.flags)
                                    )
                                };
                                let is_directory = (flags_value & 0x10000000) != 0;

                                // Filter out special NTFS metadata files starting with $
                                if !name.starts_with('$') && !name.is_empty() {
                                    entries.push((mft_ref, name, is_directory));
                                }
                            }
                        }
                    }
                }
            }

            position += entry_length;
        }
    }

    /// Lookup a name in a directory MFT record
    fn directory_lookup(&self, directory_record_number: u64, name: &str) -> Result<u64, ()> {
        let record = self.read_mft_record(directory_record_number)?;
        let entries = self.read_directory_entries(&record)?;
        for (mft_ref, entry_name, _is_directory) in &entries {
            if entry_name.eq_ignore_ascii_case(name) {
                return Ok(*mft_ref);
            }
        }
        Err(())
    }

    /// Determine FileType from MftRecord
    fn record_file_type(&self, record: &MftRecord) -> FileType {
        if record.is_directory {
            FileType::Directory
        } else {
            FileType::Regular
        }
    }
}

// ============================================================================
// UTF-16LE Decoding
// ============================================================================

/// Decode UTF-16LE bytes to a String
fn decode_utf16le(data: &[u8]) -> String {
    let mut chars = Vec::with_capacity(data.len() / 2);
    for chunk in data.chunks_exact(2) {
        let code_unit = u16::from_le_bytes([chunk[0], chunk[1]]);
        chars.push(code_unit);
    }

    // Decode UTF-16 to UTF-8
    let mut result = String::with_capacity(chars.len());
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c >= 0xD800 && c <= 0xDBFF && i + 1 < chars.len() {
            // Surrogate pair
            let hi = c;
            let lo = chars[i + 1];
            if lo >= 0xDC00 && lo <= 0xDFFF {
                let cp = 0x10000 + ((hi as u32 - 0xD800) << 10) + (lo as u32 - 0xDC00);
                if let Some(character) = char::from_u32(cp) {
                    result.push(character);
                }
                i += 2;
                continue;
            }
        }
        if let Some(character) = char::from_u32(c as u32) {
            result.push(character);
        }
        i += 1;
    }

    result
}

// ============================================================================
// NTFS Time Conversion
// ============================================================================

/// Convert NTFS timestamp (100ns intervals since 1601-01-01) to Unix epoch seconds
fn ntfs_time_to_unix(ntfs_time: u64) -> u64 {
    if ntfs_time == 0 {
        return 0;
    }
    // NTFS epoch is 1601-01-01, Unix epoch is 1970-01-01
    // Difference: 11644473600 seconds
    const NTFS_UNIX_DIFF: u64 = 11644473600;
    let seconds = ntfs_time / 10_000_000; // Convert 100ns to seconds
    seconds.saturating_sub(NTFS_UNIX_DIFF)
}

// ============================================================================
// VFS Integration
// ============================================================================

/// NTFS file handle (read-only)
struct NtfsFile {
    record_number: u64,
    device: Arc<dyn BlockDevice>,
    cluster_size: u32,
    mft_record_size: u32,
    index_block_size: u32,
    mft_start_byte: u64,
    sectors_per_cluster: u8,
    bytes_per_sector: u16,
    mft_data_runs: Vec<DataRun>,
}

// Implementation block — defines methods for the type above.
impl NtfsFile {
    fn make_inner(&self) -> NtfsFilesystemInner {
        NtfsFilesystemInner {
            device: self.device.clone(),
            cluster_size: self.cluster_size,
            mft_record_size: self.mft_record_size,
            index_block_size: self.index_block_size,
            mft_start_byte: self.mft_start_byte,
            sectors_per_cluster: self.sectors_per_cluster,
            bytes_per_sector: self.bytes_per_sector,
            mft_data_runs: self.mft_data_runs.clone(),
        }
    }
}

// Trait implementation — fulfills a behavioral contract.
impl FileOperations for NtfsFile {
    fn read(&self, offset: u64, buffer: &mut [u8]) -> VfsResult<usize> {
        let inner = self.make_inner();
        let record = inner.read_mft_record(self.record_number)
            .map_error(|_| VfsError::IoError)?;
        inner.read_file_data(&record, offset, buffer)
            .map_error(|_| VfsError::IoError)
    }

    fn write(&self, _offset: u64, _buffer: &[u8]) -> VfsResult<usize> {
        Err(VfsError::ReadOnly)
    }

    fn status(&self) -> VfsResult<Stat> {
        let inner = self.make_inner();
        let record = inner.read_mft_record(self.record_number)
            .map_error(|_| VfsError::IoError)?;
        Ok(Stat {
            ino: self.record_number,
            file_type: inner.record_file_type(&record),
            size: record.file_size,
            blocks: (record.file_size + 511) / 512,
            block_size: inner.cluster_size,
            mode: 0o444, // Read-only
            uid: 0,
            gid: 0,
            atime: ntfs_time_to_unix(record.access_time),
            mtime: ntfs_time_to_unix(record.modification_time),
            ctime: ntfs_time_to_unix(record.creation_time),
        })
    }
}

/// NTFS directory handle (read-only)
struct NtfsDirectory {
    record_number: u64,
    device: Arc<dyn BlockDevice>,
    cluster_size: u32,
    mft_record_size: u32,
    index_block_size: u32,
    mft_start_byte: u64,
    sectors_per_cluster: u8,
    bytes_per_sector: u16,
    mft_data_runs: Vec<DataRun>,
}

// Implementation block — defines methods for the type above.
impl NtfsDirectory {
    fn make_inner(&self) -> NtfsFilesystemInner {
        NtfsFilesystemInner {
            device: self.device.clone(),
            cluster_size: self.cluster_size,
            mft_record_size: self.mft_record_size,
            index_block_size: self.index_block_size,
            mft_start_byte: self.mft_start_byte,
            sectors_per_cluster: self.sectors_per_cluster,
            bytes_per_sector: self.bytes_per_sector,
            mft_data_runs: self.mft_data_runs.clone(),
        }
    }
}

// Trait implementation — fulfills a behavioral contract.
impl DirectoryOperations for NtfsDirectory {
    fn lookup(&self, name: &str) -> VfsResult<Ino> {
        let inner = self.make_inner();
        inner.directory_lookup(self.record_number, name)
            .map_error(|_| VfsError::NotFound)
    }

    fn readdir(&self) -> VfsResult<Vec<DirectoryEntry>> {
        let inner = self.make_inner();
        let record = inner.read_mft_record(self.record_number)
            .map_error(|_| VfsError::IoError)?;
        let entries = inner.read_directory_entries(&record)
            .map_error(|_| VfsError::IoError)?;

        Ok(entries.into_iterator()
            .map(|(mft_ref, name, is_directory)| DirectoryEntry {
                name,
                ino: mft_ref,
                file_type: if is_directory { FileType::Directory } else { FileType::Regular },
            })
            .collect())
    }

    fn create(&self, _name: &str, _file_type: FileType) -> VfsResult<Ino> {
        Err(VfsError::ReadOnly)
    }

    fn unlink(&self, _name: &str) -> VfsResult<()> {
        Err(VfsError::ReadOnly)
    }

    fn status(&self) -> VfsResult<Stat> {
        let inner = self.make_inner();
        let record = inner.read_mft_record(self.record_number)
            .map_error(|_| VfsError::IoError)?;
        Ok(Stat {
            ino: self.record_number,
            file_type: FileType::Directory,
            size: record.file_size,
            blocks: 0,
            block_size: inner.cluster_size,
            mode: 0o555, // Read + execute for directories
            uid: 0,
            gid: 0,
            atime: ntfs_time_to_unix(record.access_time),
            mtime: ntfs_time_to_unix(record.modification_time),
            ctime: ntfs_time_to_unix(record.creation_time),
        })
    }
}

/// FileSystem implementation for NTFS
impl FileSystem for NtfsFilesystem {
    fn name(&self) -> &str { "ntfs" }

    fn root_inode(&self) -> Ino { MFT_RECORD_ROOT }

    fn get_file(&self, ino: Ino) -> VfsResult<Arc<dyn FileOperations>> {
        let inner = self.inner.lock();
        let record = inner.read_mft_record(ino).map_error(|_| VfsError::NotFound)?;
        if record.is_directory {
            return Err(VfsError::IsDirectory);
        }
        Ok(Arc::new(NtfsFile {
            record_number: ino,
            device: inner.device.clone(),
            cluster_size: inner.cluster_size,
            mft_record_size: inner.mft_record_size,
            index_block_size: inner.index_block_size,
            mft_start_byte: inner.mft_start_byte,
            sectors_per_cluster: inner.sectors_per_cluster,
            bytes_per_sector: inner.bytes_per_sector,
            mft_data_runs: inner.mft_data_runs.clone(),
        }))
    }

    fn get_directory(&self, ino: Ino) -> VfsResult<Arc<dyn DirectoryOperations>> {
        let inner = self.inner.lock();
        let record = inner.read_mft_record(ino).map_error(|_| VfsError::NotFound)?;
        if !record.is_directory {
            return Err(VfsError::NotDirectory);
        }
        Ok(Arc::new(NtfsDirectory {
            record_number: ino,
            device: inner.device.clone(),
            cluster_size: inner.cluster_size,
            mft_record_size: inner.mft_record_size,
            index_block_size: inner.index_block_size,
            mft_start_byte: inner.mft_start_byte,
            sectors_per_cluster: inner.sectors_per_cluster,
            bytes_per_sector: inner.bytes_per_sector,
            mft_data_runs: inner.mft_data_runs.clone(),
        }))
    }

    fn status(&self, ino: Ino) -> VfsResult<Stat> {
        let inner = self.inner.lock();
        let record = inner.read_mft_record(ino).map_error(|_| VfsError::NotFound)?;
        let ft = inner.record_file_type(&record);
        Ok(Stat {
            ino,
            file_type: ft,
            size: record.file_size,
            blocks: (record.file_size + 511) / 512,
            block_size: inner.cluster_size,
            mode: if record.is_directory { 0o555 } else { 0o444 },
            uid: 0,
            gid: 0,
            atime: ntfs_time_to_unix(record.access_time),
            mtime: ntfs_time_to_unix(record.modification_time),
            ctime: ntfs_time_to_unix(record.creation_time),
        })
    }
}

// ============================================================================
// Mount / Probe
// ============================================================================

/// Mount an NTFS filesystem from a block device
pub fn mount(device: Arc<dyn BlockDevice>) -> Result<Arc<NtfsFilesystem>, &'static str> {
    // Read boot sector
    let mut boot_buffer = [0u8; SECTOR_SIZE];
    device.read_sector(0, &mut boot_buffer).map_error(|_| "Failed to read NTFS boot sector")?;

    // Verify NTFS signature
    let bpb = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { core::ptr::read_unaligned(boot_buffer.as_pointer() as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const NtfsBootSector) };
    if !bpb.is_valid() {
        return Err("Not an NTFS filesystem (bad OEM ID)");
    }

    // Also check for 0x55AA boot signature
    if boot_buffer[510] != 0x55 || boot_buffer[511] != 0xAA {
        return Err("Not an NTFS filesystem (bad boot signature)");
    }

    let cluster_size = bpb.cluster_size();
    let mft_record_size = bpb.mft_record_bytes();
    let index_block_size = bpb.index_block_bytes();
    let mft_start_byte = bpb.mft_start_byte();
    let bytes_per_sector = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        core::ptr::read_unaligned(core::ptr::address_of!(bpb.bytes_per_sector))
    };
    let sectors_per_cluster = bpb.sectors_per_cluster;

    crate::serial_println!("[NTFS] Detected: cluster_size={} mft_record={}B index_block={}B",
        cluster_size, mft_record_size, index_block_size);
    crate::serial_println!("[NTFS] MFT at byte offset 0x{:X}", mft_start_byte);

    // Read the $MFT record (record 0) directly from its known location
    // to get the data runs for the MFT itself
    let mut mft0_buffer = vec![0u8; mft_record_size as usize];
    let sector_size = device.sector_size() as u64;
    let mft_start_sector = mft_start_byte / sector_size;
    let sectors_needed = (mft_record_size as u64 + sector_size - 1) / sector_size;

    let mut raw_buffer = vec![0u8; (sectors_needed * sector_size) as usize];
    for i in 0..sectors_needed {
        device.read_sector(mft_start_sector + i, 
            &mut raw_buffer[(i as usize * sector_size as usize)..((i + 1) as usize * sector_size as usize)])
            .map_error(|_| "Failed to read MFT record 0")?;
    }
    mft0_buffer.copy_from_slice(&raw_buffer[..mft_record_size as usize]);

    // Apply fixups to $MFT record
    {
        if mft0_buffer.len() < 8 {
            return Err("MFT record too small");
        }
        let usa_offset = u16::from_le_bytes([mft0_buffer[4], mft0_buffer[5]]) as usize;
        let usa_size = u16::from_le_bytes([mft0_buffer[6], mft0_buffer[7]]) as usize;
        if usa_size >= 2 && usa_offset + usa_size * 2 <= mft0_buffer.len() {
            let sig = u16::from_le_bytes([mft0_buffer[usa_offset], mft0_buffer[usa_offset + 1]]);
            let sector_size = bytes_per_sector as usize;
            for i in 1..usa_size {
                let sector_end = i * sector_size;
                if sector_end <= mft0_buffer.len() && sector_end >= 2 {
                    let position = sector_end - 2;
                    let stored = u16::from_le_bytes([mft0_buffer[position], mft0_buffer[position + 1]]);
                    if stored == sig {
                        let saved_off = usa_offset + i * 2;
                        if saved_off + 1 < mft0_buffer.len() {
                            mft0_buffer[position] = mft0_buffer[saved_off];
                            mft0_buffer[position + 1] = mft0_buffer[saved_off + 1];
                        }
                    }
                }
            }
        }
    }

    // Verify MFT magic
    let magic = u32::from_le_bytes([mft0_buffer[0], mft0_buffer[1], mft0_buffer[2], mft0_buffer[3]]);
    if magic != MFT_RECORD_MAGIC {
        return Err("MFT record 0 has bad magic");
    }

    // Parse $DATA attribute from $MFT to get the MFT's own data runs
    let first_attribute = u16::from_le_bytes([mft0_buffer[20], mft0_buffer[21]]) as usize;
    let used_size = u32::from_le_bytes([mft0_buffer[24], mft0_buffer[25], mft0_buffer[26], mft0_buffer[27]]) as usize;
    let mut mft_data_runs = Vec::new();

    let mut off = first_attribute;
    let limit = used_size.minimum(mft0_buffer.len());
    while off + 8 <= limit {
        let atype = u32::from_le_bytes([
            mft0_buffer[off], mft0_buffer[off + 1], mft0_buffer[off + 2], mft0_buffer[off + 3],
        ]);
        let alen = u32::from_le_bytes([
            mft0_buffer[off + 4], mft0_buffer[off + 5], mft0_buffer[off + 6], mft0_buffer[off + 7],
        ]) as usize;

        if atype == ATTRIBUTE_END || atype == 0 || alen < 16 || alen > limit - off {
            break;
        }

        if atype == ATTRIBUTE_DATA && off + 9 < limit && mft0_buffer[off + 8] == 1 {
            // Non-resident $DATA — this is the MFT data runs
            let name_length = mft0_buffer[off + 9] as usize;
            if name_length == 0 && off + 64 <= limit {
                let nr = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
                    core::ptr::read_unaligned(
                        mft0_buffer[off + 16..].as_pointer() as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const NonResidentAttributeHeader
                    )
                };
                let runs_off = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
                    core::ptr::read_unaligned(core::ptr::address_of!(nr.data_runs_offset))
                } as usize;
                let runs_start = off + runs_off;
                if runs_start < off + alen {
                    mft_data_runs = decode_data_runs(&mft0_buffer[runs_start..off + alen]);
                }
            }
        }

        off += alen;
    }

    if mft_data_runs.is_empty() {
        return Err("Failed to parse $MFT data runs");
    }

    let total_mft_clusters: u64 = mft_data_runs.iter().map(|r| r.length).sum();
    crate::serial_println!("[NTFS] $MFT: {} data runs, {} clusters total",
        mft_data_runs.len(), total_mft_clusters);

    let fs = Arc::new(NtfsFilesystem {
        inner: Mutex::new(NtfsFilesystemInner {
            device,
            cluster_size,
            mft_record_size,
            index_block_size,
            mft_start_byte,
            sectors_per_cluster,
            bytes_per_sector,
            mft_data_runs,
        }),
    });

    // Verify we can read the root directory
    {
        let inner = fs.inner.lock();
                // Pattern matching — Rust's exhaustive branching construct.
match inner.read_mft_record(MFT_RECORD_ROOT) {
            Ok(root) => {
                if !root.is_directory {
                    return Err("MFT record 5 is not a directory");
                }
                crate::serial_println!("[NTFS] Root directory OK, reading entries...");
                                // Pattern matching — Rust's exhaustive branching construct.
match inner.read_directory_entries(&root) {
                    Ok(entries) => {
                        crate::serial_println!("[NTFS] Root has {} entries", entries.len());
                    }
                    Err(_) => {
                        crate::serial_println!("[NTFS] Warning: could not read root dir entries");
                    }
                }
            }
            Err(_) => return Err("Failed to read root directory"),
        }
    }

    crate::serial_println!("[NTFS] Filesystem mounted successfully (read-only)");
    Ok(fs)
}

/// Probe a block device to check if it's NTFS
pub fn probe(device: &dyn BlockDevice) -> bool {
    let mut boot_buffer = [0u8; SECTOR_SIZE];
    if device.read_sector(0, &mut boot_buffer).is_err() {
        return false;
    }
    // Check OEM ID "NTFS    "
    &boot_buffer[3..11] == NTFS_OEM_ID
}

/// Try to auto-mount NTFS partitions from AHCI devices
pub fn try_mount_ntfs() -> Option<Arc<NtfsFilesystem>> {
    use crate::drivers::partition::{parse_partition_table, PartitionType};
    use crate::drivers::ahci;
    use super::fat32::AhciBlockReader;

    let devices = ahci::list_devices();
    crate::serial_println!("[NTFS] Scanning {} AHCI devices for NTFS partitions", devices.len());

    for device in devices {
        let port = device.port_number;
        let total_sectors = device.sector_count;

        let read_fn = |sector: u64, buffer: &mut [u8]| -> Result<(), &'static str> {
            ahci::read_sectors(port, sector, 1, buffer).map(|_| ())
        };

        if let Ok(table) = parse_partition_table(read_fn, total_sectors) {
            for partition in &table.partitions {
                                // Pattern matching — Rust's exhaustive branching construct.
match partition.partition_type {
                    PartitionType::Ntfs | PartitionType::MicrosoftBasicData => {
                        crate::serial_println!("[NTFS] Found candidate partition at LBA {} ({})",
                            partition.start_lba, partition.size_human());

                        let reader = Arc::new(AhciBlockReader::new(
                            port as usize,
                            partition.start_lba,
                        ));

                        // Probe before full mount
                        if probe(&*reader) {
                                                        // Pattern matching — Rust's exhaustive branching construct.
match mount(reader) {
                                Ok(fs) => {
                                    crate::serial_println!("[NTFS] Mounted partition from port {} at LBA {}",
                                        port, partition.start_lba);
                                    return Some(fs);
                                }
                                Err(e) => {
                                    crate::serial_println!("[NTFS] Mount failed: {}", e);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    None
}
