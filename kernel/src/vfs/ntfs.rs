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
//! - Read-only (no write support — planned, see memory/repo/ntfs_write_plan.md)
//! - No compressed file support ($DATA with compression / LZNT1)
//! - No encrypted file support (EFS)
//! - Reparse points are *detected* (tag + substitute name parsed for
//!   symlinks and junctions) but not transparently resolved by the VFS yet.
//!   Symlinks/junctions surface as `FileType::Symlink`.
//! - $Volume dirty flag IS detected at mount; future write paths must check
//!   `NtfsFs::is_dirty()` and refuse if set.

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::vec;
use alloc::sync::Arc;
use alloc::format;
use spin::Mutex;

use super::{FileOps, DirOps, FileSystem, FileType, Stat, DirEntry, VfsResult, VfsError, Ino};
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
const MFT_RECORD_VOLUME: u64 = 3;       // $Volume (holds dirty flag)
const MFT_RECORD_ROOT: u64 = 5;         // Root directory

/// Attribute type codes
const ATTR_STANDARD_INFORMATION: u32 = 0x10;
const ATTR_FILE_NAME: u32 = 0x30;
const ATTR_VOLUME_INFORMATION: u32 = 0x70;
const ATTR_DATA: u32 = 0x80;
const ATTR_INDEX_ROOT: u32 = 0x90;
const ATTR_INDEX_ALLOCATION: u32 = 0xA0;
const ATTR_BITMAP: u32 = 0xB0;
const ATTR_REPARSE_POINT: u32 = 0xC0;
const ATTR_END: u32 = 0xFFFFFFFF;

/// Common NTFS reparse tags. Microsoft tags have the high bit set and carry
/// no extra GUID. Third-party tags include a 16-byte GUID after the header.
const IO_REPARSE_TAG_MOUNT_POINT: u32 = 0xA000_0003; // Junctions / volume mount points
const IO_REPARSE_TAG_SYMLINK: u32     = 0xA000_000C; // NT symbolic links

/// $VOLUME_INFORMATION flags (bitfield in the 12th byte of the attribute value)
/// Bit 0x0001 = volume is dirty (set by Windows on unclean shutdown,
/// cleared after `chkdsk` or successful clean dismount).
/// Writing to a dirty NTFS volume risks data corruption because the journal
/// ($LogFile) has pending transactions that must be replayed by Windows first.
const VOLUME_FLAG_DIRTY: u16 = 0x0001;

/// File name namespace types
const FILE_NAME_POSIX: u8 = 0;
const FILE_NAME_WIN32: u8 = 1;
const FILE_NAME_DOS: u8 = 2;
const FILE_NAME_WIN32_AND_DOS: u8 = 3;

/// MFT record flags
const MFT_RECORD_IN_USE: u16 = 0x0001;
const MFT_RECORD_IS_DIRECTORY: u16 = 0x0002;

/// Index entry flags
const INDEX_ENTRY_SUBNODE: u32 = 0x01;
const INDEX_ENTRY_LAST: u32 = 0x02;

/// Sector size
const SECTOR_SIZE: usize = 512;

// ============================================================================
// NTFS On-Disk Structures
// ============================================================================

/// NTFS Boot Sector (BPB)
#[repr(C, packed)]
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
    num_heads: u16,             // 0x1A
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

impl NtfsBootSector {
    fn is_valid(&self) -> bool {
        self.oem_id == *NTFS_OEM_ID
    }

    fn cluster_size(&self) -> u32 {
        let bps = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(self.bytes_per_sector)) };
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
        let lcn = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(self.mft_lcn)) };
        lcn * self.cluster_size() as u64
    }
}

/// MFT Record Header
#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
struct MftRecordHeader {
    magic: u32,                 // 0x00: "FILE"
    update_seq_offset: u16,     // 0x04: Offset to update sequence
    update_seq_size: u16,       // 0x06: Size in words of update sequence
    log_seq_number: u64,        // 0x08: $LogFile sequence number
    sequence_number: u16,       // 0x10: Sequence number (for consistency)
    hard_link_count: u16,       // 0x12
    first_attr_offset: u16,     // 0x14: Offset to first attribute
    flags: u16,                 // 0x16: Flags (in-use, directory)
    used_size: u32,             // 0x18: Real size of record
    allocated_size: u32,        // 0x1C: Allocated size
    base_record: u64,           // 0x20: Base MFT record (0 if base)
    next_attr_id: u16,          // 0x28
}

/// Attribute Header (common part)
#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
struct AttrHeader {
    attr_type: u32,             // 0x00: Attribute type
    length: u32,                // 0x04: Total length including header
    non_resident: u8,           // 0x08: 0=resident, 1=non-resident
    name_length: u8,            // 0x09: Name length in UTF-16 chars
    name_offset: u16,           // 0x0A: Offset to name
    flags: u16,                 // 0x0C: Flags (compressed, encrypted, sparse)
    attr_id: u16,               // 0x0E: Attribute ID
}

/// Resident attribute specific header
#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
struct ResidentAttrHeader {
    value_length: u32,          // 0x10: Length of attribute value
    value_offset: u16,          // 0x14: Offset to value (from attr start)
    indexed_flag: u16,          // 0x16
}

/// Non-resident attribute specific header
#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
struct NonResidentAttrHeader {
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
#[derive(Clone, Copy)]
struct FileNameAttr {
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
#[derive(Clone, Copy)]
struct StdInfoAttr {
    creation_time: u64,
    modification_time: u64,
    mft_modification_time: u64,
    access_time: u64,
    file_attributes: u32,       // DOS attributes (readonly, hidden, system, etc)
    _padding: [u8; 4],
}

/// $INDEX_ROOT header
#[repr(C, packed)]
#[derive(Clone, Copy)]
struct IndexRootHeader {
    attr_type: u32,             // Attribute type indexed (0x30 for $FILE_NAME)
    collation_rule: u32,        // Collation rule
    index_block_size: u32,      // Size of index allocation entry
    clusters_per_index: u8,     // Clusters per index block
    _padding: [u8; 3],
}

/// Index Node Header (used in both INDEX_ROOT and INDEX_ALLOCATION)
#[repr(C, packed)]
#[derive(Clone, Copy)]
struct IndexNodeHeader {
    entries_offset: u32,        // Offset to first index entry (relative to this header)
    total_size: u32,            // Total size of index entries
    allocated_size: u32,        // Allocated size
    flags: u32,                 // 0x01 = has sub-nodes
}

/// Index Entry (variable-length)
#[repr(C, packed)]
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
#[derive(Clone, Copy)]
struct IndexBlockHeader {
    magic: u32,                 // "INDX"
    update_seq_offset: u16,
    update_seq_size: u16,
    log_seq_number: u64,
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
    index_alloc_runs: Vec<DataRun>,
    /// Reparse tag (0 if not a reparse point). See `IO_REPARSE_TAG_*`.
    reparse_tag: u32,
    /// Resolved substitute name (UTF-8) for symlinks/junctions.
    /// Empty if not a recognized reparse point or the tag carries no path.
    reparse_target: String,
}

// ============================================================================
// NTFS Filesystem Driver
// ============================================================================

/// NTFS filesystem instance (read-only)
pub struct NtfsFs {
    inner: Mutex<NtfsFsInner>,
    /// `$Volume` dirty flag captured at mount time.
    /// `true` means Windows did not cleanly dismount this volume
    /// (Fast Startup, hibernation, crash, or pending journal transactions).
    /// Future write paths MUST refuse if this is `true`.
    volume_dirty: bool,
}

impl NtfsFs {
    /// True if the volume's `$Volume` dirty flag was set at mount time.
    /// Write operations must refuse when this returns `true`.
    pub fn is_dirty(&self) -> bool {
        self.volume_dirty
    }
}

struct NtfsFsInner {
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

impl NtfsFsInner {
    /// Read the `$Volume` (MFT record 3) dirty flag.
    ///
    /// Returns `Ok(true)` if Windows last marked the volume dirty (unclean
    /// dismount, Fast Startup still active, hibernation, or pending
    /// `$LogFile` transactions). Returns `Ok(false)` if clean.
    /// Returns `Err(())` if the record cannot be read or the
    /// `$VOLUME_INFORMATION` attribute is missing/malformed.
    fn read_volume_dirty_flag(&self) -> Result<bool, ()> {
        let raw = self.read_mft_record_raw(MFT_RECORD_VOLUME)?;
        if raw.len() < 24 {
            return Err(());
        }

        // Validate FILE magic
        let magic = u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]);
        if magic != MFT_RECORD_MAGIC {
            return Err(());
        }

        // Walk attributes looking for $VOLUME_INFORMATION (always resident).
        let first_attr = u16::from_le_bytes([raw[20], raw[21]]) as usize;
        let used_size =
            u32::from_le_bytes([raw[24], raw[25], raw[26], raw[27]]) as usize;
        let limit = used_size.min(raw.len());

        let mut off = first_attr;
        while off + 8 <= limit {
            let atype = u32::from_le_bytes([
                raw[off], raw[off + 1], raw[off + 2], raw[off + 3],
            ]);
            let alen = u32::from_le_bytes([
                raw[off + 4], raw[off + 5], raw[off + 6], raw[off + 7],
            ]) as usize;
            if atype == ATTR_END || atype == 0 || alen < 16 || alen > limit - off {
                break;
            }
            if atype == ATTR_VOLUME_INFORMATION && off + 24 <= limit {
                // Resident attribute: value_offset at off+20, value at off+value_offset
                let value_offset =
                    u16::from_le_bytes([raw[off + 20], raw[off + 21]]) as usize;
                let value_start = off + value_offset;
                // $VOLUME_INFORMATION layout:
                //   0x00 u64 reserved
                //   0x08 u8  major_version
                //   0x09 u8  minor_version
                //   0x0A u16 flags  <-- here
                if value_start + 12 <= raw.len() {
                    let flags = u16::from_le_bytes([
                        raw[value_start + 10],
                        raw[value_start + 11],
                    ]);
                    return Ok((flags & VOLUME_FLAG_DIRTY) != 0);
                }
                return Err(());
            }
            off += alen;
        }
        Err(())
    }

    /// Read raw bytes from device at byte offset
    fn read_bytes(&self, byte_offset: u64, buf: &mut [u8]) -> Result<(), ()> {
        let sector_size = self.device.sector_size() as u64;
        let start_sector = byte_offset / sector_size;
        let offset_in_sector = (byte_offset % sector_size) as usize;

        let total_bytes = offset_in_sector + buf.len();
        let num_sectors = (total_bytes + sector_size as usize - 1) / sector_size as usize;

        let mut remaining = buf.len();
        let mut buf_offset = 0usize;
        let mut sector_buf = vec![0u8; sector_size as usize];

        for i in 0..num_sectors {
            self.device.read_sector(start_sector + i as u64, &mut sector_buf)?;

            let src_start = if i == 0 { offset_in_sector } else { 0 };
            let copy_len = (sector_size as usize - src_start).min(remaining);

            buf[buf_offset..buf_offset + copy_len]
                .copy_from_slice(&sector_buf[src_start..src_start + copy_len]);

            buf_offset += copy_len;
            remaining -= copy_len;
        }

        Ok(())
    }

    /// Read clusters from the disk
    fn read_clusters(&self, lcn: u64, count: u64, buf: &mut [u8]) -> Result<(), ()> {
        let byte_offset = lcn * self.cluster_size as u64;
        let byte_len = count as usize * self.cluster_size as usize;
        if buf.len() < byte_len {
            return Err(());
        }
        self.read_bytes(byte_offset, &mut buf[..byte_len])
    }

    /// Apply fixup array to an MFT record buffer
    fn apply_fixups(&self, buf: &mut [u8], record_size: usize) -> Result<(), ()> {
        if buf.len() < 6 {
            return Err(());
        }
        let update_seq_offset = u16::from_le_bytes([buf[4], buf[5]]) as usize;
        let update_seq_size = u16::from_le_bytes([buf[6], buf[7]]) as usize;

        if update_seq_size < 2 || update_seq_offset + update_seq_size * 2 > buf.len() {
            return Err(());
        }

        // First word is the expected signature
        let signature = u16::from_le_bytes([
            buf[update_seq_offset],
            buf[update_seq_offset + 1],
        ]);

        // Replace the last 2 bytes of each sector with the stored values
        let sector_size = self.bytes_per_sector as usize;
        for i in 1..update_seq_size {
            let sector_end = i * sector_size;
            if sector_end > record_size || sector_end < 2 {
                break;
            }
            let fixup_pos = sector_end - 2;

            // Verify the sector ends with the expected signature
            let stored = u16::from_le_bytes([buf[fixup_pos], buf[fixup_pos + 1]]);
            if stored != signature {
                return Err(()); // Fixup mismatch — corrupt record
            }

            // Replace with the saved original bytes
            let saved_offset = update_seq_offset + i * 2;
            if saved_offset + 1 < buf.len() {
                buf[fixup_pos] = buf[saved_offset];
                buf[fixup_pos + 1] = buf[saved_offset + 1];
            }
        }

        Ok(())
    }

    /// Read an MFT record by record number
    fn read_mft_record_raw(&self, record_num: u64) -> Result<Vec<u8>, ()> {
        let record_size = self.mft_record_size as usize;
        let mut buf = vec![0u8; record_size];

        // Calculate which VCN this record falls in
        let byte_offset_in_mft = record_num * record_size as u64;
        let vcn = byte_offset_in_mft / self.cluster_size as u64;
        let offset_in_cluster = (byte_offset_in_mft % self.cluster_size as u64) as usize;

        // Find the data run that contains this VCN
        let mut bytes_remaining = record_size;
        let mut buf_pos = 0;
        let mut current_vcn = vcn;
        let mut current_offset = offset_in_cluster;

        while bytes_remaining > 0 {
            // Find the run containing current_vcn
            let run = self.mft_data_runs.iter().find(|r| {
                current_vcn >= r.vcn_start && current_vcn < r.vcn_start + r.length
            });

            match run {
                Some(run) => {
                    let vcn_in_run = current_vcn - run.vcn_start;
                    let lcn = run.lcn + vcn_in_run;
                    let byte_offset = lcn * self.cluster_size as u64 + current_offset as u64;

                    let available_in_cluster = self.cluster_size as usize - current_offset;
                    let to_read = bytes_remaining.min(available_in_cluster);

                    self.read_bytes(byte_offset, &mut buf[buf_pos..buf_pos + to_read])?;

                    buf_pos += to_read;
                    bytes_remaining -= to_read;
                    current_offset = 0;
                    current_vcn += 1;
                }
                None => return Err(()),
            }
        }

        // Apply fixup array
        self.apply_fixups(&mut buf, record_size)?;

        // Verify magic
        let magic = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
        if magic != MFT_RECORD_MAGIC {
            return Err(());
        }

        Ok(buf)
    }

    /// Parse attributes from a raw MFT record buffer
    fn parse_mft_record(&self, record_num: u64, buf: &[u8]) -> Result<MftRecord, ()> {
        let header = unsafe {
            core::ptr::read_unaligned(buf.as_ptr() as *const MftRecordHeader)
        };

        let flags = header.flags;
        let is_directory = (flags & MFT_RECORD_IS_DIRECTORY) != 0;
        let first_attr = header.first_attr_offset as usize;
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
        let mut index_alloc_runs = Vec::new();
        let mut reparse_tag: u32 = 0;
        let mut reparse_target = String::new();

        let mut offset = first_attr;
        let limit = used_size.min(buf.len());

        while offset + 4 <= limit {
            let attr_type = u32::from_le_bytes([
                buf[offset], buf[offset + 1], buf[offset + 2], buf[offset + 3],
            ]);

            if attr_type == ATTR_END || attr_type == 0 {
                break;
            }

            if offset + 8 > limit {
                break;
            }

            let attr_len = u32::from_le_bytes([
                buf[offset + 4], buf[offset + 5], buf[offset + 6], buf[offset + 7],
            ]) as usize;

            if attr_len < 16 || attr_len > limit - offset {
                break;
            }

            let non_resident = buf[offset + 8];
            let name_length = buf[offset + 9] as usize;

            // We only care about unnamed attributes (the default data stream)
            let is_unnamed = name_length == 0;

            match attr_type {
                ATTR_STANDARD_INFORMATION if non_resident == 0 => {
                    // Resident $STANDARD_INFORMATION
                    if offset + 24 <= limit {
                        let val_len = u32::from_le_bytes([
                            buf[offset + 16], buf[offset + 17],
                            buf[offset + 18], buf[offset + 19],
                        ]) as usize;
                        let val_off = u16::from_le_bytes([
                            buf[offset + 20], buf[offset + 21],
                        ]) as usize;
                        let data_start = offset + val_off;
                        if val_len >= 48 && data_start + 48 <= buf.len() {
                            let si = unsafe {
                                core::ptr::read_unaligned(
                                    buf[data_start..].as_ptr() as *const StdInfoAttr
                                )
                            };
                            creation_time = si.creation_time;
                            modification_time = si.modification_time;
                            access_time = si.access_time;
                            file_attributes = si.file_attributes;
                        }
                    }
                }

                ATTR_FILE_NAME if non_resident == 0 => {
                    // Resident $FILE_NAME
                    if offset + 24 <= limit {
                        let val_len = u32::from_le_bytes([
                            buf[offset + 16], buf[offset + 17],
                            buf[offset + 18], buf[offset + 19],
                        ]) as usize;
                        let val_off = u16::from_le_bytes([
                            buf[offset + 20], buf[offset + 21],
                        ]) as usize;
                        let data_start = offset + val_off;
                        if val_len >= 66 && data_start + 66 <= buf.len() {
                            let fn_attr = unsafe {
                                core::ptr::read_unaligned(
                                    buf[data_start..].as_ptr() as *const FileNameAttr
                                )
                            };
                            let ns = fn_attr.namespace;
                            let name_chars = fn_attr.name_length as usize;
                            let name_start = data_start + 66;

                            // Pick the best namespace name:
                            // Win32 (1) > Win32+DOS (3) > POSIX (0) > DOS (2)
                            let priority = match ns {
                                FILE_NAME_WIN32 => 4,
                                FILE_NAME_WIN32_AND_DOS => 3,
                                FILE_NAME_POSIX => 2,
                                FILE_NAME_DOS => 1,
                                _ => 0,
                            };
                            let current_priority = best_namespace.map(|n| match n {
                                FILE_NAME_WIN32 => 4,
                                FILE_NAME_WIN32_AND_DOS => 3,
                                FILE_NAME_POSIX => 2,
                                FILE_NAME_DOS => 1,
                                _ => 0,
                            }).unwrap_or(0);

                            if priority > current_priority {
                                if name_start + name_chars * 2 <= buf.len() {
                                    file_name = decode_utf16le(
                                        &buf[name_start..name_start + name_chars * 2]
                                    );
                                    best_namespace = Some(ns);
                                    parent_ref = fn_attr.parent_ref & 0x0000FFFFFFFFFFFF;

                                    // Use file size from FILE_NAME if no $DATA yet
                                    if file_size == 0 {
                                        file_size = unsafe {
                                            core::ptr::read_unaligned(
                                                core::ptr::addr_of!(fn_attr.real_size)
                                            )
                                        };
                                    }
                                }
                            }
                        }
                    }
                }

                ATTR_DATA if is_unnamed => {
                    if non_resident == 0 {
                        // Resident $DATA — small file stored inside MFT record
                        data_resident = true;
                        if offset + 24 <= limit {
                            let val_len = u32::from_le_bytes([
                                buf[offset + 16], buf[offset + 17],
                                buf[offset + 18], buf[offset + 19],
                            ]) as usize;
                            let val_off = u16::from_le_bytes([
                                buf[offset + 20], buf[offset + 21],
                            ]) as usize;
                            let data_start = offset + val_off;
                            if data_start + val_len <= buf.len() {
                                resident_data = buf[data_start..data_start + val_len].to_vec();
                                file_size = val_len as u64;
                            }
                        }
                    } else {
                        // Non-resident $DATA — decode data runs
                        data_resident = false;
                        if offset + 64 <= limit {
                            let nr_header = unsafe {
                                core::ptr::read_unaligned(
                                    buf[offset + 16..].as_ptr() as *const NonResidentAttrHeader
                                )
                            };
                            file_size = nr_header.real_size;
                            let runs_offset = unsafe {
                                core::ptr::read_unaligned(
                                    core::ptr::addr_of!(nr_header.data_runs_offset)
                                )
                            } as usize;
                            let runs_start = offset + runs_offset;
                            if runs_start < offset + attr_len {
                                data_runs = decode_data_runs(
                                    &buf[runs_start..offset + attr_len]
                                );
                            }
                        }
                    }
                }

                ATTR_INDEX_ROOT if non_resident == 0 => {
                    // Resident $INDEX_ROOT
                    if offset + 24 <= limit {
                        let val_len = u32::from_le_bytes([
                            buf[offset + 16], buf[offset + 17],
                            buf[offset + 18], buf[offset + 19],
                        ]) as usize;
                        let val_off = u16::from_le_bytes([
                            buf[offset + 20], buf[offset + 21],
                        ]) as usize;
                        let data_start = offset + val_off;
                        if data_start + val_len <= buf.len() {
                            index_root_data = buf[data_start..data_start + val_len].to_vec();
                        }
                    }
                }

                ATTR_REPARSE_POINT if non_resident == 0 => {
                    // Resident reparse point. Layout (Microsoft tags):
                    //   u32 ReparseTag
                    //   u16 ReparseDataLength
                    //   u16 Reserved
                    //   <data> (symlink / mount-point buffer)
                    if offset + 24 <= limit {
                        let val_len = u32::from_le_bytes([
                            buf[offset + 16], buf[offset + 17],
                            buf[offset + 18], buf[offset + 19],
                        ]) as usize;
                        let val_off = u16::from_le_bytes([
                            buf[offset + 20], buf[offset + 21],
                        ]) as usize;
                        let data_start = offset + val_off;
                        if val_len >= 8 && data_start + 8 <= buf.len() {
                            let tag = u32::from_le_bytes([
                                buf[data_start], buf[data_start + 1],
                                buf[data_start + 2], buf[data_start + 3],
                            ]);
                            reparse_tag = tag;
                            // Header is 8 bytes (tag + len + reserved); buffer starts at +8.
                            // For symlink/mount-point we read SubstituteName.
                            let buf_off = match tag {
                                IO_REPARSE_TAG_MOUNT_POINT => 8,
                                IO_REPARSE_TAG_SYMLINK => 8,
                                _ => 0,
                            };
                            if buf_off != 0 {
                                let path_buf_hdr = data_start + buf_off;
                                // 4 u16 fields: SubstOff, SubstLen, PrintOff, PrintLen.
                                // Symlink adds a u32 Flags after them.
                                if path_buf_hdr + 8 <= buf.len()
                                    && path_buf_hdr + 8 <= data_start + val_len
                                {
                                    let subst_off = u16::from_le_bytes([
                                        buf[path_buf_hdr], buf[path_buf_hdr + 1],
                                    ]) as usize;
                                    let subst_len = u16::from_le_bytes([
                                        buf[path_buf_hdr + 2], buf[path_buf_hdr + 3],
                                    ]) as usize;
                                    let extra = if tag == IO_REPARSE_TAG_SYMLINK { 12 } else { 8 };
                                    let path_start = path_buf_hdr + extra + subst_off;
                                    let path_end = path_start + subst_len;
                                    if path_end <= buf.len()
                                        && path_end <= data_start + val_len
                                    {
                                        reparse_target =
                                            decode_utf16le(&buf[path_start..path_end]);
                                    }
                                }
                            }
                        }
                    }
                }

                ATTR_INDEX_ALLOCATION if non_resident != 0 => {
                    // Non-resident $INDEX_ALLOCATION
                    if offset + 64 <= limit {
                        let nr_header = unsafe {
                            core::ptr::read_unaligned(
                                buf[offset + 16..].as_ptr() as *const NonResidentAttrHeader
                            )
                        };
                        let runs_offset = unsafe {
                            core::ptr::read_unaligned(
                                core::ptr::addr_of!(nr_header.data_runs_offset)
                            )
                        } as usize;
                        let runs_start = offset + runs_offset;
                        if runs_start < offset + attr_len {
                            index_alloc_runs = decode_data_runs(
                                &buf[runs_start..offset + attr_len]
                            );
                        }
                    }
                }

                _ => {}
            }

            offset += attr_len;
        }

        Ok(MftRecord {
            record_number: record_num,
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
            index_alloc_runs,
            reparse_tag,
            reparse_target,
        })
    }

    /// Read and parse an MFT record by number
    fn read_mft_record(&self, record_num: u64) -> Result<MftRecord, ()> {
        let raw = self.read_mft_record_raw(record_num)?;
        self.parse_mft_record(record_num, &raw)
    }

    /// Read file data using data runs
    fn read_file_data(
        &self,
        record: &MftRecord,
        file_offset: u64,
        buf: &mut [u8],
    ) -> Result<usize, ()> {
        if file_offset >= record.file_size {
            return Ok(0);
        }

        let read_len = ((record.file_size - file_offset) as usize).min(buf.len());
        if read_len == 0 {
            return Ok(0);
        }

        if record.data_resident {
            // Resident data — copy directly from MFT record
            let start = file_offset as usize;
            let end = start + read_len;
            if end <= record.resident_data.len() {
                buf[..read_len].copy_from_slice(&record.resident_data[start..end]);
            } else {
                let avail = record.resident_data.len().saturating_sub(start);
                buf[..avail].copy_from_slice(&record.resident_data[start..start + avail]);
            }
            return Ok(read_len);
        }

        // Non-resident data — follow data runs
        let cluster_size = self.cluster_size as u64;
        let mut remaining = read_len;
        let mut buf_offset = 0usize;
        let mut offset = file_offset;

        while remaining > 0 {
            let vcn = offset / cluster_size;
            let offset_in_cluster = (offset % cluster_size) as usize;

            // Find the run containing this VCN
            let run = record.data_runs.iter().find(|r| {
                vcn >= r.vcn_start && vcn < r.vcn_start + r.length
            });

            match run {
                Some(run) if run.lcn > 0 => {
                    let vcn_offset = vcn - run.vcn_start;
                    let lcn = run.lcn + vcn_offset;
                    let byte_offset = lcn * cluster_size + offset_in_cluster as u64;

                    let available = cluster_size as usize - offset_in_cluster;
                    let to_read = remaining.min(available);

                    self.read_bytes(byte_offset, &mut buf[buf_offset..buf_offset + to_read])?;

                    buf_offset += to_read;
                    offset += to_read as u64;
                    remaining -= to_read;
                }
                Some(_) => {
                    // Sparse run — zero fill
                    let available = cluster_size as usize - offset_in_cluster;
                    let to_fill = remaining.min(available);
                    for b in &mut buf[buf_offset..buf_offset + to_fill] {
                        *b = 0;
                    }
                    buf_offset += to_fill;
                    offset += to_fill as u64;
                    remaining -= to_fill;
                }
                None => {
                    // Beyond data runs — zero fill rest
                    for b in &mut buf[buf_offset..buf_offset + remaining] {
                        *b = 0;
                    }
                    remaining = 0;
                }
            }
        }

        Ok(read_len)
    }

    /// Read clusters from data runs (for index allocation)
    fn read_from_runs(&self, runs: &[DataRun], vcn: u64, buf: &mut [u8]) -> Result<(), ()> {
        let run = runs.iter().find(|r| {
            vcn >= r.vcn_start && vcn < r.vcn_start + r.length
        });

        match run {
            Some(run) if run.lcn > 0 => {
                let vcn_offset = vcn - run.vcn_start;
                let lcn = run.lcn + vcn_offset;
                let clusters_needed = (buf.len() + self.cluster_size as usize - 1)
                    / self.cluster_size as usize;
                // Read cluster by cluster
                for i in 0..clusters_needed {
                    let byte_off = (lcn + i as u64) * self.cluster_size as u64;
                    let buf_start = i * self.cluster_size as usize;
                    let buf_end = (buf_start + self.cluster_size as usize).min(buf.len());
                    self.read_bytes(byte_off, &mut buf[buf_start..buf_end])?;
                }
                Ok(())
            }
            _ => Err(()),
        }
    }

    /// Parse directory entries from $INDEX_ROOT and $INDEX_ALLOCATION
    fn read_dir_entries(&self, record: &MftRecord) -> Result<Vec<(u64, String, bool)>, ()> {
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
                let end = (node_offset + total_size).min(ir_data.len());

                self.parse_index_entries(&ir_data[start..end], &mut entries);
            }
        }

        // Parse $INDEX_ALLOCATION (for large directories)
        if !record.index_alloc_runs.is_empty() {
            let index_block_size = self.index_block_size as usize;
            let clusters_per_block = (index_block_size + self.cluster_size as usize - 1)
                / self.cluster_size as usize;

            // Calculate total VCNs available
            let total_vcns: u64 = record.index_alloc_runs.iter()
                .map(|r| r.length)
                .sum();

            let mut vcn: u64 = 0;
            while vcn < total_vcns {
                let mut block_buf = vec![0u8; index_block_size];
                if self.read_from_runs(&record.index_alloc_runs, vcn, &mut block_buf).is_ok() {
                    // Apply fixups to the index block
                    let _ = self.apply_fixups(&mut block_buf, index_block_size);

                    let magic = u32::from_le_bytes([
                        block_buf[0], block_buf[1], block_buf[2], block_buf[3],
                    ]);
                    if magic == INDX_MAGIC {
                        // IndexNodeHeader starts at offset 0x18 in the INDX block
                        let node_off = 0x18;
                        if node_off + 16 <= block_buf.len() {
                            let eo = u32::from_le_bytes([
                                block_buf[node_off], block_buf[node_off + 1],
                                block_buf[node_off + 2], block_buf[node_off + 3],
                            ]) as usize;
                            let ts = u32::from_le_bytes([
                                block_buf[node_off + 4], block_buf[node_off + 5],
                                block_buf[node_off + 6], block_buf[node_off + 7],
                            ]) as usize;

                            let start = node_off + eo;
                            let end = (node_off + ts).min(block_buf.len());
                            if start < end {
                                self.parse_index_entries(&block_buf[start..end], &mut entries);
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
        let mut pos = 0;
        while pos + 16 <= data.len() {
            let entry_header = unsafe {
                core::ptr::read_unaligned(data[pos..].as_ptr() as *const IndexEntryHeader)
            };

            let entry_len = entry_header.entry_length as usize;
            let content_len = entry_header.content_length as usize;
            let flags = entry_header.flags;

            if entry_len < 16 || entry_len > data.len() - pos {
                break;
            }

            if (flags & INDEX_ENTRY_LAST) != 0 {
                break; // Last entry marker
            }

            if content_len >= 66 {
                // Content is a $FILE_NAME structure
                let content_start = pos + 16; // After IndexEntryHeader
                if content_start + content_len <= data.len() {
                    let fn_data = &data[content_start..content_start + content_len];
                    if fn_data.len() >= 66 {
                        let fn_attr = unsafe {
                            core::ptr::read_unaligned(
                                fn_data.as_ptr() as *const FileNameAttr
                            )
                        };

                        let ns = fn_attr.namespace;
                        // Skip DOS-only names
                        if ns != FILE_NAME_DOS {
                            let name_chars = fn_attr.name_length as usize;
                            let name_start = 66;
                            if name_start + name_chars * 2 <= fn_data.len() {
                                let name = decode_utf16le(
                                    &fn_data[name_start..name_start + name_chars * 2]
                                );

                                let mft_ref = entry_header.mft_reference & 0x0000FFFFFFFFFFFF;
                                let flags_val = unsafe {
                                    core::ptr::read_unaligned(
                                        core::ptr::addr_of!(fn_attr.flags)
                                    )
                                };
                                let is_dir = (flags_val & 0x10000000) != 0;

                                // Filter out special NTFS metadata files starting with $
                                if !name.starts_with('$') && !name.is_empty() {
                                    entries.push((mft_ref, name, is_dir));
                                }
                            }
                        }
                    }
                }
            }

            pos += entry_len;
        }
    }

    /// Lookup a name in a directory MFT record
    fn dir_lookup(&self, dir_record_num: u64, name: &str) -> Result<u64, ()> {
        let record = self.read_mft_record(dir_record_num)?;
        let entries = self.read_dir_entries(&record)?;
        for (mft_ref, entry_name, _is_dir) in &entries {
            if entry_name.eq_ignore_ascii_case(name) {
                return Ok(*mft_ref);
            }
        }
        Err(())
    }

    /// Determine FileType from MftRecord. Reparse points (NT symlinks and
    /// junctions) surface as `FileType::Symlink` even when the MFT record
    /// also has the directory flag set (junctions are dir-typed reparse
    /// points underneath).
    fn record_file_type(&self, record: &MftRecord) -> FileType {
        match record.reparse_tag {
            IO_REPARSE_TAG_SYMLINK | IO_REPARSE_TAG_MOUNT_POINT => return FileType::Symlink,
            _ => {}
        }
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
                if let Some(ch) = char::from_u32(cp) {
                    result.push(ch);
                }
                i += 2;
                continue;
            }
        }
        if let Some(ch) = char::from_u32(c as u32) {
            result.push(ch);
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
    record_num: u64,
    device: Arc<dyn BlockDevice>,
    cluster_size: u32,
    mft_record_size: u32,
    index_block_size: u32,
    mft_start_byte: u64,
    sectors_per_cluster: u8,
    bytes_per_sector: u16,
    mft_data_runs: Vec<DataRun>,
}

impl NtfsFile {
    fn make_inner(&self) -> NtfsFsInner {
        NtfsFsInner {
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

impl FileOps for NtfsFile {
    fn read(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        let inner = self.make_inner();
        let record = inner.read_mft_record(self.record_num)
            .map_err(|_| VfsError::IoError)?;
        inner.read_file_data(&record, offset, buf)
            .map_err(|_| VfsError::IoError)
    }

    fn write(&self, _offset: u64, _buf: &[u8]) -> VfsResult<usize> {
        Err(VfsError::ReadOnly)
    }

    fn stat(&self) -> VfsResult<Stat> {
        let inner = self.make_inner();
        let record = inner.read_mft_record(self.record_num)
            .map_err(|_| VfsError::IoError)?;
        Ok(Stat {
            ino: self.record_num,
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
struct NtfsDir {
    record_num: u64,
    device: Arc<dyn BlockDevice>,
    cluster_size: u32,
    mft_record_size: u32,
    index_block_size: u32,
    mft_start_byte: u64,
    sectors_per_cluster: u8,
    bytes_per_sector: u16,
    mft_data_runs: Vec<DataRun>,
}

impl NtfsDir {
    fn make_inner(&self) -> NtfsFsInner {
        NtfsFsInner {
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

impl DirOps for NtfsDir {
    fn lookup(&self, name: &str) -> VfsResult<Ino> {
        let inner = self.make_inner();
        inner.dir_lookup(self.record_num, name)
            .map_err(|_| VfsError::NotFound)
    }

    fn readdir(&self) -> VfsResult<Vec<DirEntry>> {
        let inner = self.make_inner();
        let record = inner.read_mft_record(self.record_num)
            .map_err(|_| VfsError::IoError)?;
        let entries = inner.read_dir_entries(&record)
            .map_err(|_| VfsError::IoError)?;

        Ok(entries.into_iter()
            .map(|(mft_ref, name, is_dir)| DirEntry {
                name,
                ino: mft_ref,
                file_type: if is_dir { FileType::Directory } else { FileType::Regular },
            })
            .collect())
    }

    fn create(&self, _name: &str, _file_type: FileType) -> VfsResult<Ino> {
        Err(VfsError::ReadOnly)
    }

    fn unlink(&self, _name: &str) -> VfsResult<()> {
        Err(VfsError::ReadOnly)
    }

    fn stat(&self) -> VfsResult<Stat> {
        let inner = self.make_inner();
        let record = inner.read_mft_record(self.record_num)
            .map_err(|_| VfsError::IoError)?;
        Ok(Stat {
            ino: self.record_num,
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
impl FileSystem for NtfsFs {
    fn name(&self) -> &str { "ntfs" }

    fn root_inode(&self) -> Ino { MFT_RECORD_ROOT }

    fn get_file(&self, ino: Ino) -> VfsResult<Arc<dyn FileOps>> {
        let inner = self.inner.lock();
        let record = inner.read_mft_record(ino).map_err(|_| VfsError::NotFound)?;
        if record.is_directory {
            return Err(VfsError::IsDirectory);
        }
        Ok(Arc::new(NtfsFile {
            record_num: ino,
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

    fn get_dir(&self, ino: Ino) -> VfsResult<Arc<dyn DirOps>> {
        let inner = self.inner.lock();
        let record = inner.read_mft_record(ino).map_err(|_| VfsError::NotFound)?;
        if !record.is_directory {
            return Err(VfsError::NotDirectory);
        }
        Ok(Arc::new(NtfsDir {
            record_num: ino,
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

    fn stat(&self, ino: Ino) -> VfsResult<Stat> {
        let inner = self.inner.lock();
        let record = inner.read_mft_record(ino).map_err(|_| VfsError::NotFound)?;
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
pub fn mount(device: Arc<dyn BlockDevice>) -> Result<Arc<NtfsFs>, &'static str> {
    // Read boot sector
    let mut boot_buf = [0u8; SECTOR_SIZE];
    device.read_sector(0, &mut boot_buf).map_err(|_| "Failed to read NTFS boot sector")?;

    // Verify NTFS signature
    let bpb = unsafe { core::ptr::read_unaligned(boot_buf.as_ptr() as *const NtfsBootSector) };
    if !bpb.is_valid() {
        return Err("Not an NTFS filesystem (bad OEM ID)");
    }

    // Also check for 0x55AA boot signature
    if boot_buf[510] != 0x55 || boot_buf[511] != 0xAA {
        return Err("Not an NTFS filesystem (bad boot signature)");
    }

    let cluster_size = bpb.cluster_size();
    let mft_record_size = bpb.mft_record_bytes();
    let index_block_size = bpb.index_block_bytes();
    let mft_start_byte = bpb.mft_start_byte();
    let bytes_per_sector = unsafe {
        core::ptr::read_unaligned(core::ptr::addr_of!(bpb.bytes_per_sector))
    };
    let sectors_per_cluster = bpb.sectors_per_cluster;

    crate::serial_println!("[NTFS] Detected: cluster_size={} mft_record={}B index_block={}B",
        cluster_size, mft_record_size, index_block_size);
    crate::serial_println!("[NTFS] MFT at byte offset 0x{:X}", mft_start_byte);

    // Read the $MFT record (record 0) directly from its known location
    // to get the data runs for the MFT itself
    let mut mft0_buf = vec![0u8; mft_record_size as usize];
    let sector_size = device.sector_size() as u64;
    let mft_start_sector = mft_start_byte / sector_size;
    let sectors_needed = (mft_record_size as u64 + sector_size - 1) / sector_size;

    let mut raw_buf = vec![0u8; (sectors_needed * sector_size) as usize];
    for i in 0..sectors_needed {
        device.read_sector(mft_start_sector + i, 
            &mut raw_buf[(i as usize * sector_size as usize)..((i + 1) as usize * sector_size as usize)])
            .map_err(|_| "Failed to read MFT record 0")?;
    }
    mft0_buf.copy_from_slice(&raw_buf[..mft_record_size as usize]);

    // Apply fixups to $MFT record
    {
        if mft0_buf.len() < 8 {
            return Err("MFT record too small");
        }
        let usa_offset = u16::from_le_bytes([mft0_buf[4], mft0_buf[5]]) as usize;
        let usa_size = u16::from_le_bytes([mft0_buf[6], mft0_buf[7]]) as usize;
        if usa_size >= 2 && usa_offset + usa_size * 2 <= mft0_buf.len() {
            let sig = u16::from_le_bytes([mft0_buf[usa_offset], mft0_buf[usa_offset + 1]]);
            let sector_sz = bytes_per_sector as usize;
            for i in 1..usa_size {
                let sec_end = i * sector_sz;
                if sec_end <= mft0_buf.len() && sec_end >= 2 {
                    let pos = sec_end - 2;
                    let stored = u16::from_le_bytes([mft0_buf[pos], mft0_buf[pos + 1]]);
                    if stored == sig {
                        let saved_off = usa_offset + i * 2;
                        if saved_off + 1 < mft0_buf.len() {
                            mft0_buf[pos] = mft0_buf[saved_off];
                            mft0_buf[pos + 1] = mft0_buf[saved_off + 1];
                        }
                    }
                }
            }
        }
    }

    // Verify MFT magic
    let magic = u32::from_le_bytes([mft0_buf[0], mft0_buf[1], mft0_buf[2], mft0_buf[3]]);
    if magic != MFT_RECORD_MAGIC {
        return Err("MFT record 0 has bad magic");
    }

    // Parse $DATA attribute from $MFT to get the MFT's own data runs
    let first_attr = u16::from_le_bytes([mft0_buf[20], mft0_buf[21]]) as usize;
    let used_size = u32::from_le_bytes([mft0_buf[24], mft0_buf[25], mft0_buf[26], mft0_buf[27]]) as usize;
    let mut mft_data_runs = Vec::new();

    let mut off = first_attr;
    let limit = used_size.min(mft0_buf.len());
    while off + 8 <= limit {
        let atype = u32::from_le_bytes([
            mft0_buf[off], mft0_buf[off + 1], mft0_buf[off + 2], mft0_buf[off + 3],
        ]);
        let alen = u32::from_le_bytes([
            mft0_buf[off + 4], mft0_buf[off + 5], mft0_buf[off + 6], mft0_buf[off + 7],
        ]) as usize;

        if atype == ATTR_END || atype == 0 || alen < 16 || alen > limit - off {
            break;
        }

        if atype == ATTR_DATA && off + 9 < limit && mft0_buf[off + 8] == 1 {
            // Non-resident $DATA — this is the MFT data runs
            let name_len = mft0_buf[off + 9] as usize;
            if name_len == 0 && off + 64 <= limit {
                let nr = unsafe {
                    core::ptr::read_unaligned(
                        mft0_buf[off + 16..].as_ptr() as *const NonResidentAttrHeader
                    )
                };
                let runs_off = unsafe {
                    core::ptr::read_unaligned(core::ptr::addr_of!(nr.data_runs_offset))
                } as usize;
                let runs_start = off + runs_off;
                if runs_start < off + alen {
                    mft_data_runs = decode_data_runs(&mft0_buf[runs_start..off + alen]);
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

    let inner = NtfsFsInner {
        device,
        cluster_size,
        mft_record_size,
        index_block_size,
        mft_start_byte,
        sectors_per_cluster,
        bytes_per_sector,
        mft_data_runs,
    };

    // Read $Volume dirty flag (MFT record 3, $VOLUME_INFORMATION attr).
    // If we can't read it, treat as dirty (safe default — refuse future RW).
    let volume_dirty = match inner.read_volume_dirty_flag() {
        Ok(d) => {
            if d {
                crate::serial_println!(
                    "[NTFS] ⚠ Volume is DIRTY (Fast Startup, hibernation, or unclean dismount)"
                );
                crate::serial_println!(
                    "[NTFS]   → Read-only access OK; write operations will be refused."
                );
            } else {
                crate::serial_println!("[NTFS] ✓ Volume is clean");
            }
            d
        }
        Err(_) => {
            crate::serial_println!(
                "[NTFS] ⚠ Could not read $Volume dirty flag — assuming dirty (safe default)"
            );
            true
        }
    };

    let fs = Arc::new(NtfsFs {
        inner: Mutex::new(inner),
        volume_dirty,
    });

    // Verify we can read the root directory
    {
        let inner = fs.inner.lock();
        match inner.read_mft_record(MFT_RECORD_ROOT) {
            Ok(root) => {
                if !root.is_directory {
                    return Err("MFT record 5 is not a directory");
                }
                crate::serial_println!("[NTFS] Root directory OK, reading entries...");
                match inner.read_dir_entries(&root) {
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

    crate::serial_println!(
        "[NTFS] Filesystem mounted successfully (read-only, dirty={})",
        fs.volume_dirty
    );
    Ok(fs)
}

/// Probe a block device to check if it's NTFS
pub fn probe(device: &dyn BlockDevice) -> bool {
    let mut boot_buf = [0u8; SECTOR_SIZE];
    if device.read_sector(0, &mut boot_buf).is_err() {
        return false;
    }
    // Check OEM ID "NTFS    "
    &boot_buf[3..11] == NTFS_OEM_ID
}

/// Try to auto-mount NTFS partitions from AHCI devices
pub fn try_mount_ntfs() -> Option<Arc<NtfsFs>> {
    use crate::drivers::partition::{parse_partition_table, PartitionType};
    use crate::drivers::ahci;
    use super::fat32::AhciBlockReader;

    let devices = ahci::list_devices();
    crate::serial_println!("[NTFS] Scanning {} AHCI devices for NTFS partitions", devices.len());

    for device in devices {
        let port = device.port_num;
        let total_sectors = device.sector_count;

        let read_fn = |sector: u64, buf: &mut [u8]| -> Result<(), &'static str> {
            ahci::read_sectors(port, sector, 1, buf).map(|_| ())
        };

        if let Ok(table) = parse_partition_table(read_fn, total_sectors) {
            for partition in &table.partitions {
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
