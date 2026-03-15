//! TrustFS - Simple Filesystem for TrustOS
//!
//! A simple filesystem designed for the virtio-blk device.
//! Inspired by FAT but simplified for educational purposes.
//!
//! Disk Layout:
//! - Sector 0: Superblock
//! - Sectors 1-16: Inode table (256 inodes)
//! - Sectors 17-32: Block bitmap
//! - Sectors 33+: Data blocks

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::sync::Arc;
use alloc::collections::BTreeMap;
use spin::RwLock;
use core::sync::atomic::{AtomicU64, Ordering};

use super::{
    FileSystem, FileOperations, DirectoryOperations, Stat, DirectoryEntry, FileType,
    Ino, VfsResult, VfsError
};
use super::fat32::BlockDevice;

// Compile-time constant — evaluated at compilation, zero runtime cost.
const SECTOR_SIZE: usize = 512;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const MAGIC: u32 = 0x54525553; // "TRUS"
const VERSION: u32 = 1;

// Compile-time constant — evaluated at compilation, zero runtime cost.
const SUPERBLOCK_SECTOR: u64 = 0;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const INODE_START_SECTOR: u64 = 1;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const INODE_SECTORS: u64 = 16;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const BITMAP_START_SECTOR: u64 = 17;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const BITMAP_SECTORS: u64 = 16;
/// Data starts after WAL area (sectors 33-96 reserved for WAL journal)
const DATA_START_SECTOR: u64 = 97;

// Compile-time constant — evaluated at compilation, zero runtime cost.
const MAXIMUM_INODES: usize = 256;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const INODES_PER_SECTOR: usize = SECTOR_SIZE / core::mem::size_of::<DiskInode>();

// Compile-time constant — evaluated at compilation, zero runtime cost.
const MAXIMUM_NAME_LENGTH: usize = 28;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const DIRECT_BLOCKS: usize = 12;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const INDIRECT_PTRS: usize = SECTOR_SIZE / 4; // 128 block pointers per indirect block
/// Max file blocks: 12 direct + 128 indirect + 128×128 double-indirect = 16524 (~8.25MB)
const MAXIMUM_FILE_BLOCKS: usize = DIRECT_BLOCKS + INDIRECT_PTRS + INDIRECT_PTRS * INDIRECT_PTRS;

/// On-disk superblock
#[repr(C)]
// #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone, Copy, Debug)]
struct Superblock {
    magic: u32,
    version: u32,
    total_blocks: u32,
    free_blocks: u32,
    total_inodes: u32,
    free_inodes: u32,
    block_size: u32,
    root_inode: u32,
    reserved: [u32; 8],
}

// Implementation block — defines methods for the type above.
impl Superblock {
    fn new(total_sectors: u64) -> Self {
        Self {
            magic: MAGIC,
            version: VERSION,
            total_blocks: (total_sectors - DATA_START_SECTOR) as u32,
            free_blocks: (total_sectors - DATA_START_SECTOR - 1) as u32, // -1 for root dir
            total_inodes: MAXIMUM_INODES as u32,
            free_inodes: (MAXIMUM_INODES - 1) as u32, // -1 for root
            block_size: SECTOR_SIZE as u32,
            root_inode: 1,
            reserved: [0; 8],
        }
    }
    
    fn is_valid(&self) -> bool {
        self.magic == MAGIC && self.version == VERSION
    }
}

/// On-disk inode structure (32 bytes)
#[repr(C)]
// #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone, Copy, Debug)]
struct DiskInode {
    mode: u16,          // File type + permissions
    nlink: u16,         // Number of links
    size: u32,          // File size in bytes
    blocks: u32,        // Number of blocks used
    atime: u32,         // Access time
    mtime: u32,         // Modification time
    direct: [u32; DIRECT_BLOCKS], // Direct block pointers (12 × 512B = 6KB)
    indirect: u32,      // Single indirect block pointer (+128 × 512B = 64KB)
    double_indirect: u32, // Double indirect block pointer (+128×128 × 512B = ~8MB)
}

// Trait implementation — fulfills a behavioral contract.
impl Default for DiskInode {
    fn default() -> Self {
        Self {
            mode: 0,
            nlink: 0,
            size: 0,
            blocks: 0,
            atime: 0,
            mtime: 0,
            direct: [0; DIRECT_BLOCKS],
            indirect: 0,
            double_indirect: 0,
        }
    }
}

// Implementation block — defines methods for the type above.
impl DiskInode {
    fn file_type(&self) -> FileType {
                // Pattern matching — Rust's exhaustive branching construct.
match (self.mode >> 12) & 0xF {
            0x4 => FileType::Directory,
            0x8 => FileType::Regular,
            0x2 => FileType::CharDevice,
            0x6 => FileType::BlockDevice,
            _ => FileType::Regular,
        }
    }
    
    fn is_directory(&self) -> bool {
        self.file_type() == FileType::Directory
    }
    
    fn set_type(&mut self, ft: FileType) {
        let type_bits = // Pattern matching — Rust's exhaustive branching construct.
match ft {
            FileType::Directory => 0x4,
            FileType::Regular => 0x8,
            FileType::CharDevice => 0x2,
            FileType::BlockDevice => 0x6,
            _ => 0x8,
        };
        self.mode = (self.mode & 0x0FFF) | (type_bits << 12);
    }
}

/// Directory entry on disk (32 bytes)
#[repr(C)]
// #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone, Copy)]
struct DiskDirectoryEntry {
    inode: u32,
    name: [u8; MAXIMUM_NAME_LENGTH],
}

// Implementation block — defines methods for the type above.
impl DiskDirectoryEntry {
    fn name_str(&self) -> &str {
        let end = self.name.iter().position(|&b| b == 0).unwrap_or(MAXIMUM_NAME_LENGTH);
        core::str::from_utf8(&self.name[..end]).unwrap_or("")
    }
}

/// In-memory inode cache
struct InodeCache {
    inodes: BTreeMap<Ino, DiskInode>,
}

// Implementation block — defines methods for the type above.
impl InodeCache {
    fn new() -> Self {
        Self {
            inodes: BTreeMap::new(),
        }
    }
}

/// TrustFS file operations
struct TrustFilesystemFile {
    fs: Arc<TrustFilesystemInner>,
    ino: Ino,
}

// Trait implementation — fulfills a behavioral contract.
impl FileOperations for TrustFilesystemFile {
    fn read(&self, offset: u64, buffer: &mut [u8]) -> VfsResult<usize> {
        self.fs.read_file(self.ino, offset, buffer)
    }
    
    fn write(&self, offset: u64, buffer: &[u8]) -> VfsResult<usize> {
        self.fs.write_file(self.ino, offset, buffer)
    }
    
    fn status(&self) -> VfsResult<Stat> {
        self.fs.status(self.ino)
    }
    
    fn truncate(&self, size: u64) -> VfsResult<()> {
        self.fs.truncate(self.ino, size)
    }
    
    fn sync(&self) -> VfsResult<()> {
        self.fs.sync()
    }
}

/// TrustFS directory operations
struct TrustFilesystemDirectory {
    fs: Arc<TrustFilesystemInner>,
    ino: Ino,
}

// Trait implementation — fulfills a behavioral contract.
impl DirectoryOperations for TrustFilesystemDirectory {
    fn lookup(&self, name: &str) -> VfsResult<Ino> {
        self.fs.lookup(self.ino, name)
    }
    
    fn readdir(&self) -> VfsResult<Vec<DirectoryEntry>> {
        self.fs.readdir(self.ino)
    }
    
    fn create(&self, name: &str, file_type: FileType) -> VfsResult<Ino> {
        self.fs.create(self.ino, name, file_type)
    }
    
    fn unlink(&self, name: &str) -> VfsResult<()> {
        self.fs.unlink(self.ino, name)
    }
    
    fn status(&self) -> VfsResult<Stat> {
        self.fs.status(self.ino)
    }
}

/// Inner TrustFS state (shared)
struct TrustFilesystemInner {
    superblock: RwLock<Superblock>,
    inode_cache: RwLock<InodeCache>,
    dirty: RwLock<bool>,
    backend: Arc<dyn BlockDevice>,
}

// Implementation block — defines methods for the type above.
impl TrustFilesystemInner {
    /// Read a sector (via block cache if available)
    fn read_sector(&self, sector: u64, buffer: &mut [u8; SECTOR_SIZE]) -> VfsResult<()> {
        if super::block_cache::cached_read(sector, buffer).is_ok() {
            return Ok(());
        }
        self.backend.read_sector(sector, buffer)
            .map_error(|_| VfsError::IoError)
    }
    
    /// Write a sector (via block cache if available)
    fn write_sector(&self, sector: u64, buffer: &[u8; SECTOR_SIZE]) -> VfsResult<()> {
        if super::block_cache::cached_write(sector, buffer).is_ok() {
            return Ok(());
        }
        self.backend.write_sector(sector, buffer)
            .map_error(|_| VfsError::IoError)
    }

    /// Write a sector through the WAL for crash safety
    fn write_sector_wal(&self, sector: u64, buffer: &[u8; SECTOR_SIZE]) -> VfsResult<()> {
        // Log in WAL, then write through to cache/disk
        let _ = super::wal::log_write(sector, buffer);
        self.write_sector(sector, buffer)
    }

    /// Flush any pending WAL transaction to disk
    fn flush_wal(&self) -> VfsResult<()> {
        let backend = &self.backend;
        let write_fn = |sector: u64, data: &[u8; SECTOR_SIZE]| -> Result<(), ()> {
            backend.write_sector(sector, data)
        };
        super::wal::commit(&write_fn).map_error(|_| VfsError::IoError)
    }
    
    /// Read an inode from disk
    fn read_inode(&self, ino: Ino) -> VfsResult<DiskInode> {
        // Check cache first
        {
            let cache = self.inode_cache.read();
            if let Some(inode) = cache.inodes.get(&ino) {
                return Ok(*inode);
            }
        }
        
        // Read from disk
        let sector = INODE_START_SECTOR + (ino as u64 / INODES_PER_SECTOR as u64);
        let offset = (ino as usize % INODES_PER_SECTOR) * core::mem::size_of::<DiskInode>();
        
        let mut buffer = [0u8; SECTOR_SIZE];
        self.read_sector(sector, &mut buffer)?;
        
        let inode_pointer = buffer[offset..].as_pointer() as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const DiskInode;
        let inode = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { *inode_pointer };
        
        // Cache it
        {
            let mut cache = self.inode_cache.write();
            cache.inodes.insert(ino, inode);
        }
        
        Ok(inode)
    }
    
    /// Write an inode to disk
    fn write_inode(&self, ino: Ino, inode: &DiskInode) -> VfsResult<()> {
        let sector = INODE_START_SECTOR + (ino as u64 / INODES_PER_SECTOR as u64);
        let offset = (ino as usize % INODES_PER_SECTOR) * core::mem::size_of::<DiskInode>();
        
        // Read-modify-write
        let mut buffer = [0u8; SECTOR_SIZE];
        self.read_sector(sector, &mut buffer)?;
        
        let inode_pointer = buffer[offset..].as_mut_pointer() as *mut DiskInode;
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { *inode_pointer = *inode; }
        
        self.write_sector(sector, &buffer)?;
        
        // Update cache
        {
            let mut cache = self.inode_cache.write();
            cache.inodes.insert(ino, *inode);
        }
        
        *self.dirty.write() = true;
        Ok(())
    }
    
    /// Allocate a new inode
    fn allocator_inode(&self) -> VfsResult<Ino> {
        let mut sb = self.superblock.write();
        if sb.free_inodes == 0 {
            return Err(VfsError::NoSpace);
        }
        
        // Find free inode (simple linear search)
        for ino in 1..MAXIMUM_INODES as u64 {
            let inode = self.read_inode(ino)?;
            if inode.nlink == 0 && inode.mode == 0 {
                sb.free_inodes -= 1;
                return Ok(ino);
            }
        }
        
        Err(VfsError::NoSpace)
    }
    
    /// Allocate a data block
    fn allocator_block(&self) -> VfsResult<u32> {
        let mut sb = self.superblock.write();
        if sb.free_blocks == 0 {
            return Err(VfsError::NoSpace);
        }
        
        // Read bitmap
        for bitmap_sector in 0..BITMAP_SECTORS {
            let mut buffer = [0u8; SECTOR_SIZE];
            self.read_sector(BITMAP_START_SECTOR + bitmap_sector, &mut buffer)?;
            
            for byte_index in 0..SECTOR_SIZE {
                if buffer[byte_index] != 0xFF {
                    // Found a byte with free bit
                    for bit in 0..8 {
                        if (buffer[byte_index] & (1 << bit)) == 0 {
                            // Mark as used
                            buffer[byte_index] |= 1 << bit;
                            self.write_sector(BITMAP_START_SECTOR + bitmap_sector, &buffer)?;
                            
                            sb.free_blocks -= 1;
                            let block = (bitmap_sector as u32 * SECTOR_SIZE as u32 * 8)
                                + (byte_index as u32 * 8)
                                + bit as u32;
                            return Ok(block);
                        }
                    }
                }
            }
        }
        
        Err(VfsError::NoSpace)
    }
    
    /// Free a data block (clear its bitmap bit)
    fn free_block(&self, block: u32) -> VfsResult<()> {
        let mut sb = self.superblock.write();
        
        let bitmap_sector = block as u64 / (SECTOR_SIZE as u64 * 8);
        let byte_index = (block as usize / 8) % SECTOR_SIZE;
        let bit = block as usize % 8;
        
        if bitmap_sector >= BITMAP_SECTORS {
            return Err(VfsError::InvalidData);
        }
        
        let mut buffer = [0u8; SECTOR_SIZE];
        self.read_sector(BITMAP_START_SECTOR + bitmap_sector, &mut buffer)?;
        
        // Clear the bit
        buffer[byte_index] &= !(1 << bit);
        self.write_sector(BITMAP_START_SECTOR + bitmap_sector, &buffer)?;
        
        sb.free_blocks += 1;
        Ok(())
    }
    
    /// Free all data blocks held by an inode (direct + indirect + double-indirect)
    fn free_inode_blocks(&self, inode: &DiskInode) -> VfsResult<()> {
        // Free direct blocks
        for i in 0..DIRECT_BLOCKS {
            if inode.direct[i] != 0 {
                self.free_block(inode.direct[i])?;
            }
        }
        
        // Free indirect block entries + the indirect block itself
        if inode.indirect != 0 {
            let mut ind_buffer = [0u8; SECTOR_SIZE];
            self.read_sector(DATA_START_SECTOR + inode.indirect as u64, &mut ind_buffer)?;
            let ptrs = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &*(ind_buffer.as_pointer() as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const [u32; INDIRECT_PTRS]) };
            
            for &ptr in ptrs.iter() {
                if ptr != 0 {
                    self.free_block(ptr)?;
                }
            }
            
            // Free the indirect block itself
            self.free_block(inode.indirect)?;
        }
        
        // Free double-indirect block entries + all second-level blocks + the top-level block
        if inode.double_indirect != 0 {
            let mut l1_buffer = [0u8; SECTOR_SIZE];
            self.read_sector(DATA_START_SECTOR + inode.double_indirect as u64, &mut l1_buffer)?;
            let l1_ptrs = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &*(l1_buffer.as_pointer() as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const [u32; INDIRECT_PTRS]) };
            
            for &l2_block in l1_ptrs.iter() {
                if l2_block != 0 {
                    let mut l2_buffer = [0u8; SECTOR_SIZE];
                    self.read_sector(DATA_START_SECTOR + l2_block as u64, &mut l2_buffer)?;
                    let l2_ptrs = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &*(l2_buffer.as_pointer() as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const [u32; INDIRECT_PTRS]) };
                    
                    for &data_block in l2_ptrs.iter() {
                        if data_block != 0 {
                            self.free_block(data_block)?;
                        }
                    }
                    
                    // Free the second-level indirect block
                    self.free_block(l2_block)?;
                }
            }
            
            // Free the top-level double-indirect block
            self.free_block(inode.double_indirect)?;
        }
        
        Ok(())
    }
    
    /// Resolve logical block index to physical block number
    fn resolve_block(&self, inode: &DiskInode, block_index: usize) -> VfsResult<u32> {
        if block_index < DIRECT_BLOCKS {
            Ok(inode.direct[block_index])
        } else if block_index < DIRECT_BLOCKS + INDIRECT_PTRS {
            if inode.indirect == 0 { return Ok(0); }
            let mut ind_buffer = [0u8; SECTOR_SIZE];
            self.read_sector(DATA_START_SECTOR + inode.indirect as u64, &mut ind_buffer)?;
            let ptrs = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &*(ind_buffer.as_pointer() as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const [u32; INDIRECT_PTRS]) };
            Ok(ptrs[block_index - DIRECT_BLOCKS])
        } else if block_index < MAXIMUM_FILE_BLOCKS {
            // Double indirect: two levels of indirection
            if inode.double_indirect == 0 { return Ok(0); }
            let di_offset = block_index - DIRECT_BLOCKS - INDIRECT_PTRS;
            let l1_index = di_offset / INDIRECT_PTRS;
            let l2_index = di_offset % INDIRECT_PTRS;
            // Read first-level table
            let mut l1_buffer = [0u8; SECTOR_SIZE];
            self.read_sector(DATA_START_SECTOR + inode.double_indirect as u64, &mut l1_buffer)?;
            let l1_ptrs = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &*(l1_buffer.as_pointer() as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const [u32; INDIRECT_PTRS]) };
            let l2_block = l1_ptrs[l1_index];
            if l2_block == 0 { return Ok(0); }
            // Read second-level table
            let mut l2_buffer = [0u8; SECTOR_SIZE];
            self.read_sector(DATA_START_SECTOR + l2_block as u64, &mut l2_buffer)?;
            let l2_ptrs = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &*(l2_buffer.as_pointer() as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const [u32; INDIRECT_PTRS]) };
            Ok(l2_ptrs[l2_index])
        } else {
            Err(VfsError::NoSpace) // File too large
        }
    }

    /// Write a block pointer into the indirect block table
    fn write_indirect_pointer(&self, inode: &mut DiskInode, index: usize, block_number: u32) -> VfsResult<()> {
        if inode.indirect == 0 {
            inode.indirect = self.allocator_block()?;
            let zero = [0u8; SECTOR_SIZE];
            self.write_sector(DATA_START_SECTOR + inode.indirect as u64, &zero)?;
        }
        let mut ind_buffer = [0u8; SECTOR_SIZE];
        self.read_sector(DATA_START_SECTOR + inode.indirect as u64, &mut ind_buffer)?;
        let ptrs = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &mut *(ind_buffer.as_mut_pointer() as *mut [u32; INDIRECT_PTRS]) };
        ptrs[index] = block_number;
        self.write_sector(DATA_START_SECTOR + inode.indirect as u64, &ind_buffer)
    }

    /// Write a block pointer into the double-indirect block table
    fn write_double_indirect_pointer(&self, inode: &mut DiskInode, di_offset: usize, block_number: u32) -> VfsResult<()> {
        let zero = [0u8; SECTOR_SIZE];
        // Allocate top-level double-indirect block if needed
        if inode.double_indirect == 0 {
            inode.double_indirect = self.allocator_block()?;
            self.write_sector(DATA_START_SECTOR + inode.double_indirect as u64, &zero)?;
        }
        let l1_index = di_offset / INDIRECT_PTRS;
        let l2_index = di_offset % INDIRECT_PTRS;
        // Read first-level table
        let mut l1_buffer = [0u8; SECTOR_SIZE];
        self.read_sector(DATA_START_SECTOR + inode.double_indirect as u64, &mut l1_buffer)?;
        let l1_ptrs = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &mut *(l1_buffer.as_mut_pointer() as *mut [u32; INDIRECT_PTRS]) };
        // Allocate second-level block if needed
        if l1_ptrs[l1_index] == 0 {
            l1_ptrs[l1_index] = self.allocator_block()?;
            self.write_sector(DATA_START_SECTOR + inode.double_indirect as u64, &l1_buffer)?;
            self.write_sector(DATA_START_SECTOR + l1_ptrs[l1_index] as u64, &zero)?;
        }
        let l2_block = l1_ptrs[l1_index];
        // Write pointer into second-level table
        let mut l2_buffer = [0u8; SECTOR_SIZE];
        self.read_sector(DATA_START_SECTOR + l2_block as u64, &mut l2_buffer)?;
        let l2_ptrs = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &mut *(l2_buffer.as_mut_pointer() as *mut [u32; INDIRECT_PTRS]) };
        l2_ptrs[l2_index] = block_number;
        self.write_sector(DATA_START_SECTOR + l2_block as u64, &l2_buffer)
    }

    /// Read file data (supports direct + indirect + double-indirect blocks, up to ~8MB)
    fn read_file(&self, ino: Ino, offset: u64, buffer: &mut [u8]) -> VfsResult<usize> {
        let inode = self.read_inode(ino)?;
        
        if offset >= inode.size as u64 {
            return Ok(0); // EOF
        }
        
        let to_read = core::cmp::minimum(buffer.len(), (inode.size as u64 - offset) as usize);
        let mut bytes_read = 0;
        let mut file_offset = offset as usize;
        
        while bytes_read < to_read {
            let block_index = file_offset / SECTOR_SIZE;
            let block_offset = file_offset % SECTOR_SIZE;
            
            let physical_block = self.resolve_block(&inode, block_index)?;
            if physical_block == 0 { break; }
            
            let mut sector_buffer = [0u8; SECTOR_SIZE];
            self.read_sector(DATA_START_SECTOR + physical_block as u64, &mut sector_buffer)?;
            
            let chunk_size = core::cmp::minimum(SECTOR_SIZE - block_offset, to_read - bytes_read);
            buffer[bytes_read..bytes_read + chunk_size]
                .copy_from_slice(&sector_buffer[block_offset..block_offset + chunk_size]);
            
            bytes_read += chunk_size;
            file_offset += chunk_size;
        }
        
        Ok(bytes_read)
    }
    
    /// Write file data (supports direct + indirect + double-indirect blocks, up to ~8MB)
    fn write_file(&self, ino: Ino, offset: u64, buffer: &[u8]) -> VfsResult<usize> {
        let mut inode = self.read_inode(ino)?;
        
        let mut bytes_written = 0;
        let mut file_offset = offset as usize;
        let maximum_blocks = MAXIMUM_FILE_BLOCKS;
        
        while bytes_written < buffer.len() {
            let block_index = file_offset / SECTOR_SIZE;
            let block_offset = file_offset % SECTOR_SIZE;
            
            if block_index >= maximum_blocks { break; }
            
            // Resolve or allocate block
            let physical_block = self.resolve_block(&inode, block_index)?;
            let physical_block = if physical_block == 0 {
                let new_block = self.allocator_block()?;
                inode.blocks += 1;
                if block_index < DIRECT_BLOCKS {
                    inode.direct[block_index] = new_block;
                } else if block_index < DIRECT_BLOCKS + INDIRECT_PTRS {
                    self.write_indirect_pointer(&mut inode, block_index - DIRECT_BLOCKS, new_block)?;
                } else {
                    self.write_double_indirect_pointer(&mut inode, block_index - DIRECT_BLOCKS - INDIRECT_PTRS, new_block)?;
                }
                new_block
            } else {
                physical_block
            };
            
            let sector = DATA_START_SECTOR + physical_block as u64;
            let chunk_size = core::cmp::minimum(SECTOR_SIZE - block_offset, buffer.len() - bytes_written);
            
            // Read-modify-write for partial blocks
            let mut sector_buffer = [0u8; SECTOR_SIZE];
            if block_offset > 0 || chunk_size < SECTOR_SIZE {
                self.read_sector(sector, &mut sector_buffer)?;
            }
            
            sector_buffer[block_offset..block_offset + chunk_size]
                .copy_from_slice(&buffer[bytes_written..bytes_written + chunk_size]);
            
            self.write_sector(sector, &sector_buffer)?;
            
            bytes_written += chunk_size;
            file_offset += chunk_size;
        }
        
        // Update size
        let new_size = core::cmp::maximum(inode.size, (offset + bytes_written as u64) as u32);
        if new_size != inode.size {
            inode.size = new_size;
            inode.mtime = (crate::logger::get_ticks() / 100) as u32;
            self.write_inode(ino, &inode)?;
        }
        
        Ok(bytes_written)
    }
    
    /// Lookup name in directory
    fn lookup(&self, directory_ino: Ino, name: &str) -> VfsResult<Ino> {
        let entries = self.readdir(directory_ino)?;
        for entry in entries {
            if entry.name == name {
                return Ok(entry.ino);
            }
        }
        Err(VfsError::NotFound)
    }
    
    /// Read directory entries
    fn readdir(&self, directory_ino: Ino) -> VfsResult<Vec<DirectoryEntry>> {
        let inode = self.read_inode(directory_ino)?;
        if !inode.is_directory() {
            return Err(VfsError::NotDirectory);
        }
        
        let mut entries = Vec::new();
        let entry_size = core::mem::size_of::<DiskDirectoryEntry>();
        let number_entries = inode.size as usize / entry_size;
        
        for i in 0..number_entries {
            let offset = (i * entry_size) as u64;
            let mut buffer = [0u8; 32]; // DiskDirEntry is 32 bytes
            self.read_file(directory_ino, offset, &mut buffer)?;
            
            let entry_pointer = buffer.as_pointer() as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const DiskDirectoryEntry;
            let disk_entry = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &*entry_pointer };
            
            if disk_entry.inode != 0 {
                let child_inode = self.read_inode(disk_entry.inode as Ino)?;
                entries.push(DirectoryEntry {
                    name: String::from(disk_entry.name_str()),
                    ino: disk_entry.inode as Ino,
                    file_type: child_inode.file_type(),
                });
            }
        }
        
        Ok(entries)
    }
    
    /// Create a new file/directory in parent
    fn create(&self, parent_ino: Ino, name: &str, file_type: FileType) -> VfsResult<Ino> {
        if name.len() > MAXIMUM_NAME_LENGTH {
            return Err(VfsError::InvalidPath);
        }
        
        // Check if exists
        if self.lookup(parent_ino, name).is_ok() {
            return Err(VfsError::AlreadyExists);
        }
        
        // Allocate new inode
        let new_ino = self.allocator_inode()?;
        let mut new_inode = DiskInode::default();
        new_inode.set_type(file_type);
        new_inode.nlink = 1;
        new_inode.mode |= 0o644; // rw-r--r--
        
        if file_type == FileType::Directory {
            new_inode.mode |= 0o111; // +x for directories
        }
        
        self.write_inode(new_ino, &new_inode)?;
        
        // Add entry to parent directory
        let mut entry = DiskDirectoryEntry {
            inode: new_ino as u32,
            name: [0; MAXIMUM_NAME_LENGTH],
        };
        let name_bytes = name.as_bytes();
        let copy_length = core::cmp::minimum(name_bytes.len(), MAXIMUM_NAME_LENGTH);
        entry.name[..copy_length].copy_from_slice(&name_bytes[..copy_length]);
        
        let parent_inode = self.read_inode(parent_ino)?;
        let offset = parent_inode.size as u64;
        
        let entry_bytes = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
            core::slice::from_raw_parts(
                &entry as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const DiskDirectoryEntry as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const u8,
                core::mem::size_of::<DiskDirectoryEntry>()
            )
        };
        
        self.write_file(parent_ino, offset, entry_bytes)?;
        
        Ok(new_ino)
    }
    
    /// Unlink (delete) a file from directory
    fn unlink(&self, parent_ino: Ino, name: &str) -> VfsResult<()> {
        let entries = self.readdir(parent_ino)?;
        let entry_size = core::mem::size_of::<DiskDirectoryEntry>();
        
        for (i, entry) in entries.iter().enumerate() {
            if entry.name == name {
                // Read the inode
                let mut inode = self.read_inode(entry.ino)?;
                
                // Check if directory is empty
                if inode.is_directory() && inode.size > 0 {
                    let children = self.readdir(entry.ino)?;
                    if !children.is_empty() {
                        return Err(VfsError::NotEmpty);
                    }
                }
                
                // Decrement link count
                inode.nlink = inode.nlink.saturating_sub(1);
                
                if inode.nlink == 0 {
                    // Free all data blocks held by this inode
                    if let Err(e) = self.free_inode_blocks(&inode) {
                        crate::log_warn!("[TRUSTFS] Warning: failed to free blocks for inode {}: {:?}", entry.ino, e);
                    }
                    // Free the inode (mark as unused)
                    inode.mode = 0;
                    inode.size = 0;
                    inode.blocks = 0;
                    inode.direct = [0; DIRECT_BLOCKS];
                    inode.indirect = 0;
                    
                    // Update superblock free inode count
                    {
                        let mut sb = self.superblock.write();
                        sb.free_inodes += 1;
                    }
                }
                
                self.write_inode(entry.ino, &inode)?;
                
                // Remove directory entry by writing zero inode
                let mut zero_entry = DiskDirectoryEntry {
                    inode: 0,
                    name: [0; MAXIMUM_NAME_LENGTH],
                };
                let offset = (i * entry_size) as u64;
                let entry_bytes = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
                    core::slice::from_raw_parts(
                        &zero_entry as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const DiskDirectoryEntry as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const u8,
                        entry_size
                    )
                };
                self.write_file(parent_ino, offset, entry_bytes)?;
                
                return Ok(());
            }
        }
        
        Err(VfsError::NotFound)
    }
    
    /// Truncate file to size
    fn truncate(&self, ino: Ino, size: u64) -> VfsResult<()> {
        let mut inode = self.read_inode(ino)?;
        let old_size = inode.size as u64;
        let new_size = size;
        
        if new_size < old_size {
            // Shrinking: free blocks beyond the new size
            let old_blocks = ((old_size + SECTOR_SIZE as u64 - 1) / SECTOR_SIZE as u64) as usize;
            let new_blocks = ((new_size + SECTOR_SIZE as u64 - 1) / SECTOR_SIZE as u64) as usize;
            
            for block_index in new_blocks..old_blocks {
                if block_index < DIRECT_BLOCKS {
                    if inode.direct[block_index] != 0 {
                        let _ = self.free_block(inode.direct[block_index]);
                        inode.direct[block_index] = 0;
                        inode.blocks = inode.blocks.saturating_sub(1);
                    }
                } else if block_index < DIRECT_BLOCKS + INDIRECT_PTRS {
                    if inode.indirect != 0 {
                        let mut ind_buffer = [0u8; SECTOR_SIZE];
                        self.read_sector(DATA_START_SECTOR + inode.indirect as u64, &mut ind_buffer)?;
                        let ptrs = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &mut *(ind_buffer.as_mut_pointer() as *mut [u32; INDIRECT_PTRS]) };
                        let index = block_index - DIRECT_BLOCKS;
                        if ptrs[index] != 0 {
                            let _ = self.free_block(ptrs[index]);
                            ptrs[index] = 0;
                            inode.blocks = inode.blocks.saturating_sub(1);
                            self.write_sector(DATA_START_SECTOR + inode.indirect as u64, &ind_buffer)?;
                        }
                    }
                }
            }
            
            // If no more indirect blocks used, free the indirect block itself
            if new_blocks <= DIRECT_BLOCKS && inode.indirect != 0 {
                let _ = self.free_block(inode.indirect);
                inode.indirect = 0;
            }
        }
        
        inode.size = new_size as u32;
        inode.mtime = (crate::logger::get_ticks() / 100) as u32;
        self.write_inode(ino, &inode)
    }
    
    /// Get file statistics
    fn status(&self, ino: Ino) -> VfsResult<Stat> {
        let inode = self.read_inode(ino)?;
        Ok(Stat {
            ino,
            file_type: inode.file_type(),
            size: inode.size as u64,
            blocks: inode.blocks as u64,
            block_size: SECTOR_SIZE as u32,
            mode: inode.mode as u32,
            uid: 0,
            gid: 0,
            atime: inode.atime as u64,
            mtime: inode.mtime as u64,
            ctime: 0,
        })
    }
    
    /// Sync filesystem to disk (flush WAL + cache + superblock)
    fn sync(&self) -> VfsResult<()> {
        // Flush WAL first
        self.flush_wal()?;
        // Then flush block cache
        let _ = super::block_cache::sync();
        // Write superblock
        let sb = self.superblock.read();
        let mut buffer = [0u8; SECTOR_SIZE];
        let sb_pointer = buffer.as_mut_pointer() as *mut Superblock;
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { *sb_pointer = *sb; }
        self.write_sector(SUPERBLOCK_SECTOR, &buffer)?;
        
        *self.dirty.write() = false;
        crate::log_debug!("[TrustFS] sync complete");
        Ok(())
    }
}

/// TrustFS public interface
pub struct TrustFs {
    inner: Arc<TrustFilesystemInner>,
}

// Implementation block — defines methods for the type above.
impl TrustFs {
    /// Create new TrustFS on a block device, format if needed
    pub fn new(backend: Arc<dyn BlockDevice>, capacity: u64) -> VfsResult<Self> {
        // Try to read existing superblock
        let mut buffer = [0u8; SECTOR_SIZE];
        backend.read_sector(SUPERBLOCK_SECTOR, &mut buffer)
            .map_error(|_| VfsError::IoError)?;
        
        let sb_pointer = buffer.as_pointer() as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const Superblock;
        let existing_sb = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { *sb_pointer };
        
        let superblock = if existing_sb.is_valid() {
            crate::log_debug!("[TrustFS] Found existing filesystem");
            existing_sb
        } else {
            crate::log!("[TrustFS] Formatting new filesystem...");
            Self::format_with(&*backend, capacity)?
        };
        
        // Replay WAL if previous shutdown was unclean
        let backend_ref = &*backend;
        let replay_read = |sector: u64, buffer: &mut [u8; SECTOR_SIZE]| -> Result<(), ()> {
            backend_ref.read_sector(sector, buffer).map_error(|_| ())
        };
        let replay_write = |sector: u64, data: &[u8; SECTOR_SIZE]| -> Result<(), ()> {
            backend_ref.write_sector(sector, data).map_error(|_| ())
        };
                // Pattern matching — Rust's exhaustive branching construct.
match super::wal::replay_if_needed(&replay_read, &replay_write) {
            Ok(0) => {},
            Ok(n) => crate::log!("[TrustFS] WAL replay: {} writes recovered", n),
            Err(_) => crate::log_warn!("[TrustFS] WAL replay failed"),
        }

        let inner = Arc::new(TrustFilesystemInner {
            superblock: RwLock::new(superblock),
            inode_cache: RwLock::new(InodeCache::new()),
            dirty: RwLock::new(false),
            backend,
        });
        
        Ok(Self { inner })
    }
    
    /// Format the disk with TrustFS using a given block device
    fn format_with(backend: &dyn BlockDevice, capacity: u64) -> VfsResult<Superblock> {
        let sb = Superblock::new(capacity);
        
        // Write superblock
        let mut buffer = [0u8; SECTOR_SIZE];
        let sb_pointer = buffer.as_mut_pointer() as *mut Superblock;
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { *sb_pointer = sb; }
        backend.write_sector(SUPERBLOCK_SECTOR, &buffer)
            .map_error(|_| VfsError::IoError)?;
        
        // Clear inode table
        let zero_buffer = [0u8; SECTOR_SIZE];
        for i in 0..INODE_SECTORS {
            backend.write_sector(INODE_START_SECTOR + i, &zero_buffer)
                .map_error(|_| VfsError::IoError)?;
        }
        
        // Clear bitmap
        for i in 0..BITMAP_SECTORS {
            backend.write_sector(BITMAP_START_SECTOR + i, &zero_buffer)
                .map_error(|_| VfsError::IoError)?;
        }
        
        // Create root directory (inode 1)
        let mut root_inode = DiskInode::default();
        root_inode.set_type(FileType::Directory);
        root_inode.nlink = 1;
        root_inode.mode |= 0o755;
        
        let inode_sector = INODE_START_SECTOR;
        let mut inode_buffer = [0u8; SECTOR_SIZE];
        let inode_pointer = inode_buffer[core::mem::size_of::<DiskInode>()..].as_mut_pointer() as *mut DiskInode;
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { *inode_pointer = root_inode; }
        // Actually, inode 1 is at offset 1*32 = 32
        let root_offset = core::mem::size_of::<DiskInode>(); // Skip inode 0
        let inode_pointer = inode_buffer[root_offset..].as_mut_pointer() as *mut DiskInode;
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { *inode_pointer = root_inode; }
        backend.write_sector(inode_sector, &inode_buffer)
            .map_error(|_| VfsError::IoError)?;
        
        crate::log!("[TrustFS] Formatted: {} blocks, {} inodes", sb.total_blocks, sb.total_inodes);
        
        Ok(sb)
    }
}

// Trait implementation — fulfills a behavioral contract.
impl FileSystem for TrustFs {
    fn name(&self) -> &str {
        "trustfs"
    }
    
    fn root_inode(&self) -> Ino {
        1
    }
    
    fn get_file(&self, ino: Ino) -> VfsResult<Arc<dyn FileOperations>> {
        let inode = self.inner.read_inode(ino)?;
        if inode.is_directory() {
            return Err(VfsError::IsDirectory);
        }
        Ok(Arc::new(TrustFilesystemFile {
            fs: Arc::clone(&self.inner),
            ino,
        }))
    }
    
    fn get_directory(&self, ino: Ino) -> VfsResult<Arc<dyn DirectoryOperations>> {
        let inode = self.inner.read_inode(ino)?;
        if !inode.is_directory() {
            return Err(VfsError::NotDirectory);
        }
        Ok(Arc::new(TrustFilesystemDirectory {
            fs: Arc::clone(&self.inner),
            ino,
        }))
    }
    
    fn status(&self, ino: Ino) -> VfsResult<Stat> {
        self.inner.status(ino)
    }
    
    fn sync(&self) -> VfsResult<()> {
        self.inner.sync()
    }
}
