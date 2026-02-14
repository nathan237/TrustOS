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
    FileSystem, FileOps, DirOps, Stat, DirEntry, FileType,
    Ino, VfsResult, VfsError
};

const SECTOR_SIZE: usize = 512;
const MAGIC: u32 = 0x54525553; // "TRUS"
const VERSION: u32 = 1;

const SUPERBLOCK_SECTOR: u64 = 0;
const INODE_START_SECTOR: u64 = 1;
const INODE_SECTORS: u64 = 16;
const BITMAP_START_SECTOR: u64 = 17;
const BITMAP_SECTORS: u64 = 16;
const DATA_START_SECTOR: u64 = 33;

const MAX_INODES: usize = 256;
const INODES_PER_SECTOR: usize = SECTOR_SIZE / core::mem::size_of::<DiskInode>();

const MAX_NAME_LEN: usize = 28;
const DIRECT_BLOCKS: usize = 12;
const INDIRECT_PTRS: usize = SECTOR_SIZE / 4; // 128 block pointers per indirect block

/// On-disk superblock
#[repr(C)]
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

impl Superblock {
    fn new(total_sectors: u64) -> Self {
        Self {
            magic: MAGIC,
            version: VERSION,
            total_blocks: (total_sectors - DATA_START_SECTOR) as u32,
            free_blocks: (total_sectors - DATA_START_SECTOR - 1) as u32, // -1 for root dir
            total_inodes: MAX_INODES as u32,
            free_inodes: (MAX_INODES - 1) as u32, // -1 for root
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
}

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
        }
    }
}

impl DiskInode {
    fn file_type(&self) -> FileType {
        match (self.mode >> 12) & 0xF {
            0x4 => FileType::Directory,
            0x8 => FileType::Regular,
            0x2 => FileType::CharDevice,
            0x6 => FileType::BlockDevice,
            _ => FileType::Regular,
        }
    }
    
    fn is_dir(&self) -> bool {
        self.file_type() == FileType::Directory
    }
    
    fn set_type(&mut self, ft: FileType) {
        let type_bits = match ft {
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
#[derive(Clone, Copy)]
struct DiskDirEntry {
    inode: u32,
    name: [u8; MAX_NAME_LEN],
}

impl DiskDirEntry {
    fn name_str(&self) -> &str {
        let end = self.name.iter().position(|&b| b == 0).unwrap_or(MAX_NAME_LEN);
        core::str::from_utf8(&self.name[..end]).unwrap_or("")
    }
}

/// In-memory inode cache
struct InodeCache {
    inodes: BTreeMap<Ino, DiskInode>,
}

impl InodeCache {
    fn new() -> Self {
        Self {
            inodes: BTreeMap::new(),
        }
    }
}

/// TrustFS file operations
struct TrustFsFile {
    fs: Arc<TrustFsInner>,
    ino: Ino,
}

impl FileOps for TrustFsFile {
    fn read(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        self.fs.read_file(self.ino, offset, buf)
    }
    
    fn write(&self, offset: u64, buf: &[u8]) -> VfsResult<usize> {
        self.fs.write_file(self.ino, offset, buf)
    }
    
    fn stat(&self) -> VfsResult<Stat> {
        self.fs.stat(self.ino)
    }
    
    fn truncate(&self, size: u64) -> VfsResult<()> {
        self.fs.truncate(self.ino, size)
    }
    
    fn sync(&self) -> VfsResult<()> {
        self.fs.sync()
    }
}

/// TrustFS directory operations
struct TrustFsDir {
    fs: Arc<TrustFsInner>,
    ino: Ino,
}

impl DirOps for TrustFsDir {
    fn lookup(&self, name: &str) -> VfsResult<Ino> {
        self.fs.lookup(self.ino, name)
    }
    
    fn readdir(&self) -> VfsResult<Vec<DirEntry>> {
        self.fs.readdir(self.ino)
    }
    
    fn create(&self, name: &str, file_type: FileType) -> VfsResult<Ino> {
        self.fs.create(self.ino, name, file_type)
    }
    
    fn unlink(&self, name: &str) -> VfsResult<()> {
        self.fs.unlink(self.ino, name)
    }
    
    fn stat(&self) -> VfsResult<Stat> {
        self.fs.stat(self.ino)
    }
}

/// Inner TrustFS state (shared)
struct TrustFsInner {
    superblock: RwLock<Superblock>,
    inode_cache: RwLock<InodeCache>,
    dirty: RwLock<bool>,
}

impl TrustFsInner {
    /// Read a sector (via block cache if available)
    fn read_sector(&self, sector: u64, buf: &mut [u8; SECTOR_SIZE]) -> VfsResult<()> {
        if super::block_cache::cached_read(sector, buf).is_ok() {
            return Ok(());
        }
        crate::virtio_blk::read_sector(sector, buf)
            .map_err(|_| VfsError::IoError)
    }
    
    /// Write a sector (via block cache if available)
    fn write_sector(&self, sector: u64, buf: &[u8; SECTOR_SIZE]) -> VfsResult<()> {
        if super::block_cache::cached_write(sector, buf).is_ok() {
            return Ok(());
        }
        crate::virtio_blk::write_sector(sector, buf)
            .map_err(|_| VfsError::IoError)
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
        
        let mut buf = [0u8; SECTOR_SIZE];
        self.read_sector(sector, &mut buf)?;
        
        let inode_ptr = buf[offset..].as_ptr() as *const DiskInode;
        let inode = unsafe { *inode_ptr };
        
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
        let mut buf = [0u8; SECTOR_SIZE];
        self.read_sector(sector, &mut buf)?;
        
        let inode_ptr = buf[offset..].as_mut_ptr() as *mut DiskInode;
        unsafe { *inode_ptr = *inode; }
        
        self.write_sector(sector, &buf)?;
        
        // Update cache
        {
            let mut cache = self.inode_cache.write();
            cache.inodes.insert(ino, *inode);
        }
        
        *self.dirty.write() = true;
        Ok(())
    }
    
    /// Allocate a new inode
    fn alloc_inode(&self) -> VfsResult<Ino> {
        let mut sb = self.superblock.write();
        if sb.free_inodes == 0 {
            return Err(VfsError::NoSpace);
        }
        
        // Find free inode (simple linear search)
        for ino in 1..MAX_INODES as u64 {
            let inode = self.read_inode(ino)?;
            if inode.nlink == 0 && inode.mode == 0 {
                sb.free_inodes -= 1;
                return Ok(ino);
            }
        }
        
        Err(VfsError::NoSpace)
    }
    
    /// Allocate a data block
    fn alloc_block(&self) -> VfsResult<u32> {
        let mut sb = self.superblock.write();
        if sb.free_blocks == 0 {
            return Err(VfsError::NoSpace);
        }
        
        // Read bitmap
        for bitmap_sector in 0..BITMAP_SECTORS {
            let mut buf = [0u8; SECTOR_SIZE];
            self.read_sector(BITMAP_START_SECTOR + bitmap_sector, &mut buf)?;
            
            for byte_idx in 0..SECTOR_SIZE {
                if buf[byte_idx] != 0xFF {
                    // Found a byte with free bit
                    for bit in 0..8 {
                        if (buf[byte_idx] & (1 << bit)) == 0 {
                            // Mark as used
                            buf[byte_idx] |= 1 << bit;
                            self.write_sector(BITMAP_START_SECTOR + bitmap_sector, &buf)?;
                            
                            sb.free_blocks -= 1;
                            let block = (bitmap_sector as u32 * SECTOR_SIZE as u32 * 8)
                                + (byte_idx as u32 * 8)
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
        let byte_idx = (block as usize / 8) % SECTOR_SIZE;
        let bit = block as usize % 8;
        
        if bitmap_sector >= BITMAP_SECTORS {
            return Err(VfsError::InvalidData);
        }
        
        let mut buf = [0u8; SECTOR_SIZE];
        self.read_sector(BITMAP_START_SECTOR + bitmap_sector, &mut buf)?;
        
        // Clear the bit
        buf[byte_idx] &= !(1 << bit);
        self.write_sector(BITMAP_START_SECTOR + bitmap_sector, &buf)?;
        
        sb.free_blocks += 1;
        Ok(())
    }
    
    /// Free all data blocks held by an inode (direct + indirect)
    fn free_inode_blocks(&self, inode: &DiskInode) -> VfsResult<()> {
        // Free direct blocks
        for i in 0..DIRECT_BLOCKS {
            if inode.direct[i] != 0 {
                self.free_block(inode.direct[i])?;
            }
        }
        
        // Free indirect block entries + the indirect block itself
        if inode.indirect != 0 {
            let mut ind_buf = [0u8; SECTOR_SIZE];
            self.read_sector(DATA_START_SECTOR + inode.indirect as u64, &mut ind_buf)?;
            let ptrs = unsafe { &*(ind_buf.as_ptr() as *const [u32; INDIRECT_PTRS]) };
            
            for &ptr in ptrs.iter() {
                if ptr != 0 {
                    self.free_block(ptr)?;
                }
            }
            
            // Free the indirect block itself
            self.free_block(inode.indirect)?;
        }
        
        Ok(())
    }
    
    /// Resolve logical block index to physical block number
    fn resolve_block(&self, inode: &DiskInode, block_idx: usize) -> VfsResult<u32> {
        if block_idx < DIRECT_BLOCKS {
            Ok(inode.direct[block_idx])
        } else if block_idx < DIRECT_BLOCKS + INDIRECT_PTRS {
            if inode.indirect == 0 { return Ok(0); }
            let mut ind_buf = [0u8; SECTOR_SIZE];
            self.read_sector(DATA_START_SECTOR + inode.indirect as u64, &mut ind_buf)?;
            let ptrs = unsafe { &*(ind_buf.as_ptr() as *const [u32; INDIRECT_PTRS]) };
            Ok(ptrs[block_idx - DIRECT_BLOCKS])
        } else {
            Err(VfsError::NoSpace) // File too large
        }
    }

    /// Write a block pointer into the indirect block table
    fn write_indirect_ptr(&self, inode: &mut DiskInode, idx: usize, block_num: u32) -> VfsResult<()> {
        if inode.indirect == 0 {
            inode.indirect = self.alloc_block()?;
            // Zero out the new indirect block
            let zero = [0u8; SECTOR_SIZE];
            self.write_sector(DATA_START_SECTOR + inode.indirect as u64, &zero)?;
        }
        let mut ind_buf = [0u8; SECTOR_SIZE];
        self.read_sector(DATA_START_SECTOR + inode.indirect as u64, &mut ind_buf)?;
        let ptrs = unsafe { &mut *(ind_buf.as_mut_ptr() as *mut [u32; INDIRECT_PTRS]) };
        ptrs[idx] = block_num;
        self.write_sector(DATA_START_SECTOR + inode.indirect as u64, &ind_buf)
    }

    /// Read file data (supports direct + indirect blocks, up to ~70KB)
    fn read_file(&self, ino: Ino, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        let inode = self.read_inode(ino)?;
        
        if offset >= inode.size as u64 {
            return Ok(0); // EOF
        }
        
        let to_read = core::cmp::min(buf.len(), (inode.size as u64 - offset) as usize);
        let mut bytes_read = 0;
        let mut file_offset = offset as usize;
        
        while bytes_read < to_read {
            let block_idx = file_offset / SECTOR_SIZE;
            let block_offset = file_offset % SECTOR_SIZE;
            
            let phys_block = self.resolve_block(&inode, block_idx)?;
            if phys_block == 0 { break; }
            
            let mut sector_buf = [0u8; SECTOR_SIZE];
            self.read_sector(DATA_START_SECTOR + phys_block as u64, &mut sector_buf)?;
            
            let chunk_size = core::cmp::min(SECTOR_SIZE - block_offset, to_read - bytes_read);
            buf[bytes_read..bytes_read + chunk_size]
                .copy_from_slice(&sector_buf[block_offset..block_offset + chunk_size]);
            
            bytes_read += chunk_size;
            file_offset += chunk_size;
        }
        
        Ok(bytes_read)
    }
    
    /// Write file data (supports direct + indirect blocks, up to ~70KB)
    fn write_file(&self, ino: Ino, offset: u64, buf: &[u8]) -> VfsResult<usize> {
        let mut inode = self.read_inode(ino)?;
        
        let mut bytes_written = 0;
        let mut file_offset = offset as usize;
        let max_blocks = DIRECT_BLOCKS + INDIRECT_PTRS;
        
        while bytes_written < buf.len() {
            let block_idx = file_offset / SECTOR_SIZE;
            let block_offset = file_offset % SECTOR_SIZE;
            
            if block_idx >= max_blocks { break; }
            
            // Resolve or allocate block
            let phys_block = self.resolve_block(&inode, block_idx)?;
            let phys_block = if phys_block == 0 {
                let new_block = self.alloc_block()?;
                inode.blocks += 1;
                if block_idx < DIRECT_BLOCKS {
                    inode.direct[block_idx] = new_block;
                } else {
                    self.write_indirect_ptr(&mut inode, block_idx - DIRECT_BLOCKS, new_block)?;
                }
                new_block
            } else {
                phys_block
            };
            
            let sector = DATA_START_SECTOR + phys_block as u64;
            let chunk_size = core::cmp::min(SECTOR_SIZE - block_offset, buf.len() - bytes_written);
            
            // Read-modify-write for partial blocks
            let mut sector_buf = [0u8; SECTOR_SIZE];
            if block_offset > 0 || chunk_size < SECTOR_SIZE {
                self.read_sector(sector, &mut sector_buf)?;
            }
            
            sector_buf[block_offset..block_offset + chunk_size]
                .copy_from_slice(&buf[bytes_written..bytes_written + chunk_size]);
            
            self.write_sector(sector, &sector_buf)?;
            
            bytes_written += chunk_size;
            file_offset += chunk_size;
        }
        
        // Update size
        let new_size = core::cmp::max(inode.size, (offset + bytes_written as u64) as u32);
        if new_size != inode.size {
            inode.size = new_size;
            inode.mtime = (crate::logger::get_ticks() / 100) as u32;
            self.write_inode(ino, &inode)?;
        }
        
        Ok(bytes_written)
    }
    
    /// Lookup name in directory
    fn lookup(&self, dir_ino: Ino, name: &str) -> VfsResult<Ino> {
        let entries = self.readdir(dir_ino)?;
        for entry in entries {
            if entry.name == name {
                return Ok(entry.ino);
            }
        }
        Err(VfsError::NotFound)
    }
    
    /// Read directory entries
    fn readdir(&self, dir_ino: Ino) -> VfsResult<Vec<DirEntry>> {
        let inode = self.read_inode(dir_ino)?;
        if !inode.is_dir() {
            return Err(VfsError::NotDirectory);
        }
        
        let mut entries = Vec::new();
        let entry_size = core::mem::size_of::<DiskDirEntry>();
        let num_entries = inode.size as usize / entry_size;
        
        for i in 0..num_entries {
            let offset = (i * entry_size) as u64;
            let mut buf = [0u8; 32]; // DiskDirEntry is 32 bytes
            self.read_file(dir_ino, offset, &mut buf)?;
            
            let entry_ptr = buf.as_ptr() as *const DiskDirEntry;
            let disk_entry = unsafe { &*entry_ptr };
            
            if disk_entry.inode != 0 {
                let child_inode = self.read_inode(disk_entry.inode as Ino)?;
                entries.push(DirEntry {
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
        if name.len() > MAX_NAME_LEN {
            return Err(VfsError::InvalidPath);
        }
        
        // Check if exists
        if self.lookup(parent_ino, name).is_ok() {
            return Err(VfsError::AlreadyExists);
        }
        
        // Allocate new inode
        let new_ino = self.alloc_inode()?;
        let mut new_inode = DiskInode::default();
        new_inode.set_type(file_type);
        new_inode.nlink = 1;
        new_inode.mode |= 0o644; // rw-r--r--
        
        if file_type == FileType::Directory {
            new_inode.mode |= 0o111; // +x for directories
        }
        
        self.write_inode(new_ino, &new_inode)?;
        
        // Add entry to parent directory
        let mut entry = DiskDirEntry {
            inode: new_ino as u32,
            name: [0; MAX_NAME_LEN],
        };
        let name_bytes = name.as_bytes();
        let copy_len = core::cmp::min(name_bytes.len(), MAX_NAME_LEN);
        entry.name[..copy_len].copy_from_slice(&name_bytes[..copy_len]);
        
        let parent_inode = self.read_inode(parent_ino)?;
        let offset = parent_inode.size as u64;
        
        let entry_bytes = unsafe {
            core::slice::from_raw_parts(
                &entry as *const DiskDirEntry as *const u8,
                core::mem::size_of::<DiskDirEntry>()
            )
        };
        
        self.write_file(parent_ino, offset, entry_bytes)?;
        
        Ok(new_ino)
    }
    
    /// Unlink (delete) a file from directory
    fn unlink(&self, parent_ino: Ino, name: &str) -> VfsResult<()> {
        let entries = self.readdir(parent_ino)?;
        let entry_size = core::mem::size_of::<DiskDirEntry>();
        
        for (i, entry) in entries.iter().enumerate() {
            if entry.name == name {
                // Read the inode
                let mut inode = self.read_inode(entry.ino)?;
                
                // Check if directory is empty
                if inode.is_dir() && inode.size > 0 {
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
                let mut zero_entry = DiskDirEntry {
                    inode: 0,
                    name: [0; MAX_NAME_LEN],
                };
                let offset = (i * entry_size) as u64;
                let entry_bytes = unsafe {
                    core::slice::from_raw_parts(
                        &zero_entry as *const DiskDirEntry as *const u8,
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
            
            for block_idx in new_blocks..old_blocks {
                if block_idx < DIRECT_BLOCKS {
                    if inode.direct[block_idx] != 0 {
                        let _ = self.free_block(inode.direct[block_idx]);
                        inode.direct[block_idx] = 0;
                        inode.blocks = inode.blocks.saturating_sub(1);
                    }
                } else if block_idx < DIRECT_BLOCKS + INDIRECT_PTRS {
                    if inode.indirect != 0 {
                        let mut ind_buf = [0u8; SECTOR_SIZE];
                        self.read_sector(DATA_START_SECTOR + inode.indirect as u64, &mut ind_buf)?;
                        let ptrs = unsafe { &mut *(ind_buf.as_mut_ptr() as *mut [u32; INDIRECT_PTRS]) };
                        let idx = block_idx - DIRECT_BLOCKS;
                        if ptrs[idx] != 0 {
                            let _ = self.free_block(ptrs[idx]);
                            ptrs[idx] = 0;
                            inode.blocks = inode.blocks.saturating_sub(1);
                            self.write_sector(DATA_START_SECTOR + inode.indirect as u64, &ind_buf)?;
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
    fn stat(&self, ino: Ino) -> VfsResult<Stat> {
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
    
    /// Sync filesystem to disk (flush cache + superblock)
    fn sync(&self) -> VfsResult<()> {
        // Flush block cache first
        let _ = super::block_cache::sync();
        // Write superblock
        let sb = self.superblock.read();
        let mut buf = [0u8; SECTOR_SIZE];
        let sb_ptr = buf.as_mut_ptr() as *mut Superblock;
        unsafe { *sb_ptr = *sb; }
        self.write_sector(SUPERBLOCK_SECTOR, &buf)?;
        
        *self.dirty.write() = false;
        Ok(())
    }
}

/// TrustFS public interface
pub struct TrustFs {
    inner: Arc<TrustFsInner>,
}

impl TrustFs {
    /// Create new TrustFS, format if needed
    pub fn new() -> VfsResult<Self> {
        if !crate::virtio_blk::is_initialized() {
            return Err(VfsError::IoError);
        }
        
        let capacity = crate::virtio_blk::capacity();
        
        // Try to read existing superblock
        let mut buf = [0u8; SECTOR_SIZE];
        crate::virtio_blk::read_sector(SUPERBLOCK_SECTOR, &mut buf)
            .map_err(|_| VfsError::IoError)?;
        
        let sb_ptr = buf.as_ptr() as *const Superblock;
        let existing_sb = unsafe { *sb_ptr };
        
        let superblock = if existing_sb.is_valid() {
            crate::log_debug!("[TrustFS] Found existing filesystem");
            existing_sb
        } else {
            crate::log!("[TrustFS] Formatting new filesystem...");
            Self::format(capacity)?
        };
        
        let inner = Arc::new(TrustFsInner {
            superblock: RwLock::new(superblock),
            inode_cache: RwLock::new(InodeCache::new()),
            dirty: RwLock::new(false),
        });
        
        Ok(Self { inner })
    }
    
    /// Format the disk with TrustFS
    fn format(capacity: u64) -> VfsResult<Superblock> {
        let sb = Superblock::new(capacity);
        
        // Write superblock
        let mut buf = [0u8; SECTOR_SIZE];
        let sb_ptr = buf.as_mut_ptr() as *mut Superblock;
        unsafe { *sb_ptr = sb; }
        crate::virtio_blk::write_sector(SUPERBLOCK_SECTOR, &buf)
            .map_err(|_| VfsError::IoError)?;
        
        // Clear inode table
        let zero_buf = [0u8; SECTOR_SIZE];
        for i in 0..INODE_SECTORS {
            crate::virtio_blk::write_sector(INODE_START_SECTOR + i, &zero_buf)
                .map_err(|_| VfsError::IoError)?;
        }
        
        // Clear bitmap
        for i in 0..BITMAP_SECTORS {
            crate::virtio_blk::write_sector(BITMAP_START_SECTOR + i, &zero_buf)
                .map_err(|_| VfsError::IoError)?;
        }
        
        // Create root directory (inode 1)
        let mut root_inode = DiskInode::default();
        root_inode.set_type(FileType::Directory);
        root_inode.nlink = 1;
        root_inode.mode |= 0o755;
        
        let inode_sector = INODE_START_SECTOR;
        let mut inode_buf = [0u8; SECTOR_SIZE];
        let inode_ptr = inode_buf[core::mem::size_of::<DiskInode>()..].as_mut_ptr() as *mut DiskInode;
        unsafe { *inode_ptr = root_inode; }
        // Actually, inode 1 is at offset 1*32 = 32
        let root_offset = core::mem::size_of::<DiskInode>(); // Skip inode 0
        let inode_ptr = inode_buf[root_offset..].as_mut_ptr() as *mut DiskInode;
        unsafe { *inode_ptr = root_inode; }
        crate::virtio_blk::write_sector(inode_sector, &inode_buf)
            .map_err(|_| VfsError::IoError)?;
        
        crate::log!("[TrustFS] Formatted: {} blocks, {} inodes", sb.total_blocks, sb.total_inodes);
        
        Ok(sb)
    }
}

impl FileSystem for TrustFs {
    fn name(&self) -> &str {
        "trustfs"
    }
    
    fn root_inode(&self) -> Ino {
        1
    }
    
    fn get_file(&self, ino: Ino) -> VfsResult<Arc<dyn FileOps>> {
        let inode = self.inner.read_inode(ino)?;
        if inode.is_dir() {
            return Err(VfsError::IsDirectory);
        }
        Ok(Arc::new(TrustFsFile {
            fs: Arc::clone(&self.inner),
            ino,
        }))
    }
    
    fn get_dir(&self, ino: Ino) -> VfsResult<Arc<dyn DirOps>> {
        let inode = self.inner.read_inode(ino)?;
        if !inode.is_dir() {
            return Err(VfsError::NotDirectory);
        }
        Ok(Arc::new(TrustFsDir {
            fs: Arc::clone(&self.inner),
            ino,
        }))
    }
    
    fn stat(&self, ino: Ino) -> VfsResult<Stat> {
        self.inner.stat(ino)
    }
    
    fn sync(&self) -> VfsResult<()> {
        self.inner.sync()
    }
}
