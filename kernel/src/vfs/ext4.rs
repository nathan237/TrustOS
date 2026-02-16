//! ext4 Read-Only Filesystem Driver
//!
//! Supports reading files and directories from ext4-formatted volumes.
//! Implements block group descriptors, inode reading, extent trees,
//! directory entry parsing, and VFS integration.
//!
//! Notable ext4 features handled:
//! - Extent-based block mapping (EXT4_EXTENTS_FL)
//! - 64-bit block numbers (for large volumes)
//! - Variable-size inodes (>= 128 bytes)
//! - Hash tree directories (falls back to linear scan)
//! - Large files (>2GB via i_size_high)

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::vec;
use alloc::sync::Arc;
use alloc::format;
use spin::Mutex;

use super::{FileOps, DirOps, FileSystem, FileType, Stat, DirEntry, VfsResult, VfsError, Ino};
use super::fat32::BlockDevice;

// ============================================================================
// ext4 On-Disk Structures
// ============================================================================

/// ext4 magic number
const EXT4_SUPER_MAGIC: u16 = 0xEF53;

/// Superblock is always at byte offset 1024
const SUPERBLOCK_OFFSET: u64 = 1024;

/// Inode flag: uses extents
const EXT4_EXTENTS_FL: u32 = 0x00080000;

/// Root inode number (always 2)
const EXT4_ROOT_INO: u32 = 2;

/// File type in directory entries
const EXT4_FT_UNKNOWN: u8 = 0;
const EXT4_FT_REG_FILE: u8 = 1;
const EXT4_FT_DIR: u8 = 2;
const EXT4_FT_CHRDEV: u8 = 3;
const EXT4_FT_BLKDEV: u8 = 4;
const EXT4_FT_FIFO: u8 = 5;
const EXT4_FT_SOCK: u8 = 6;
const EXT4_FT_SYMLINK: u8 = 7;

/// Inode mode bits
const S_IFMT: u16 = 0xF000;
const S_IFREG: u16 = 0x8000;
const S_IFDIR: u16 = 0x4000;
const S_IFLNK: u16 = 0xA000;
const S_IFCHR: u16 = 0x2000;
const S_IFBLK: u16 = 0x6000;

/// ext4 Superblock (partial — fields we need for read-only)
#[repr(C)]
#[derive(Clone, Copy)]
struct Ext4SuperBlock {
    s_inodes_count: u32,         // 0x00
    s_blocks_count_lo: u32,      // 0x04
    s_r_blocks_count_lo: u32,    // 0x08
    s_free_blocks_count_lo: u32, // 0x0C
    s_free_inodes_count: u32,    // 0x10
    s_first_data_block: u32,     // 0x14
    s_log_block_size: u32,       // 0x18 (block_size = 1024 << s_log_block_size)
    s_log_cluster_size: u32,     // 0x1C
    s_blocks_per_group: u32,     // 0x20
    s_clusters_per_group: u32,   // 0x24
    s_inodes_per_group: u32,     // 0x28
    s_mtime: u32,                // 0x2C
    s_wtime: u32,                // 0x30
    s_mnt_count: u16,            // 0x34
    s_max_mnt_count: u16,        // 0x36
    s_magic: u16,                // 0x38
    s_state: u16,                // 0x3A
    s_errors: u16,               // 0x3C
    s_minor_rev_level: u16,      // 0x3E
    s_lastcheck: u32,            // 0x40
    s_checkinterval: u32,        // 0x44
    s_creator_os: u32,           // 0x48
    s_rev_level: u32,            // 0x4C
    s_def_resuid: u16,           // 0x50
    s_def_resgid: u16,           // 0x52
    // -- EXT4_DYNAMIC_REV fields --
    s_first_ino: u32,            // 0x54
    s_inode_size: u16,           // 0x58
    s_block_group_nr: u16,       // 0x5A
    s_feature_compat: u32,       // 0x5C
    s_feature_incompat: u32,     // 0x60
    s_feature_ro_compat: u32,    // 0x64
    s_uuid: [u8; 16],            // 0x68
    s_volume_name: [u8; 16],     // 0x78
    _pad1: [u8; 168],            // 0x88..0x130
    // At offset 0x150 (336): s_blocks_count_hi (u32)
    // We read this separately if needed
}

/// ext4 Block Group Descriptor (32 bytes base, 64 bytes for 64-bit)
#[repr(C)]
#[derive(Clone, Copy)]
struct Ext4GroupDescBase {
    bg_block_bitmap_lo: u32,     // 0x00
    bg_inode_bitmap_lo: u32,     // 0x04
    bg_inode_table_lo: u32,      // 0x08
    bg_free_blocks_count_lo: u16,// 0x0C
    bg_free_inodes_count_lo: u16,// 0x0E
    bg_used_dirs_count_lo: u16,  // 0x10
    bg_flags: u16,               // 0x12
    bg_exclude_bitmap_lo: u32,   // 0x14
    bg_block_bitmap_csum_lo: u16,// 0x18
    bg_inode_bitmap_csum_lo: u16,// 0x1A
    bg_itable_unused_lo: u16,    // 0x1C
    bg_checksum: u16,            // 0x1E
    // 64-bit extension (if desc_size > 32)
    bg_block_bitmap_hi: u32,     // 0x20
    bg_inode_bitmap_hi: u32,     // 0x24
    bg_inode_table_hi: u32,      // 0x28
}

/// ext4 Inode (128 bytes base, may be larger)
#[repr(C)]
#[derive(Clone, Copy)]
struct Ext4Inode {
    i_mode: u16,                 // 0x00
    i_uid: u16,                  // 0x02
    i_size_lo: u32,              // 0x04
    i_atime: u32,                // 0x08
    i_ctime: u32,                // 0x0C
    i_mtime: u32,                // 0x10
    i_dtime: u32,                // 0x14
    i_gid: u16,                  // 0x16
    i_links_count: u16,          // 0x18
    i_blocks_lo: u32,            // 0x1A
    i_flags: u32,                // 0x20
    i_osd1: u32,                 // 0x24
    i_block: [u32; 15],          // 0x28 (60 bytes — extents or block map)
    i_generation: u32,           // 0x64
    i_file_acl_lo: u32,          // 0x68
    i_size_high: u32,            // 0x6C (upper 32 bits of size for regular files)
    _pad: [u8; 16],              // 0x70..0x80
}

/// Extent tree header (12 bytes)
#[repr(C)]
#[derive(Clone, Copy)]
struct Ext4ExtentHeader {
    eh_magic: u16,      // 0xF30A
    eh_entries: u16,     // Number of valid entries
    eh_max: u16,         // Capacity of entries
    eh_depth: u16,       // 0 = leaf, >0 = internal node
    eh_generation: u32,  // Generation (not used for read)
}

/// Extent leaf entry (12 bytes)
#[repr(C)]
#[derive(Clone, Copy)]
struct Ext4Extent {
    ee_block: u32,       // First file block covered
    ee_len: u16,         // Number of blocks covered
    ee_start_hi: u16,    // High 16 bits of physical block
    ee_start_lo: u32,    // Low 32 bits of physical block
}

/// Extent index entry (12 bytes, for internal nodes)
#[repr(C)]
#[derive(Clone, Copy)]
struct Ext4ExtentIdx {
    ei_block: u32,       // Covers file blocks from this offset
    ei_leaf_lo: u32,     // Low 32 bits of child node block
    ei_leaf_hi: u16,     // High 16 bits of child node block
    _pad: u16,
}

/// Ext4 directory entry (variable length)
#[repr(C)]
struct Ext4DirEntry2 {
    inode: u32,          // Inode number
    rec_len: u16,        // Total entry length
    name_len: u8,        // Name length
    file_type: u8,       // File type
    // name follows (name_len bytes)
}

// ============================================================================
// ext4 Filesystem Driver
// ============================================================================

/// ext4 filesystem instance (read-only)
pub struct Ext4Fs {
    inner: Mutex<Ext4FsInner>,
}

struct Ext4FsInner {
    block_size: u32,
    inode_size: u16,
    inodes_per_group: u32,
    blocks_per_group: u32,
    first_data_block: u32,
    group_desc_size: u32,
    total_groups: u32,
    device: Arc<dyn BlockDevice>,
}

impl Ext4FsInner {
    /// Read raw bytes from device by byte offset
    fn read_bytes(&self, byte_offset: u64, buf: &mut [u8]) -> Result<(), ()> {
        let sector_size = self.device.sector_size() as u64;
        let start_sector = byte_offset / sector_size;
        let offset_in_sector = (byte_offset % sector_size) as usize;
        
        // Calculate how many sectors we need
        let total_bytes = offset_in_sector + buf.len();
        let num_sectors = (total_bytes + sector_size as usize - 1) / sector_size as usize;
        
        let mut sector_buf = vec![0u8; sector_size as usize];
        let mut remaining = buf.len();
        let mut buf_offset = 0usize;
        
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
    
    /// Read a full block
    fn read_block(&self, block_num: u64, buf: &mut [u8]) -> Result<(), ()> {
        let byte_offset = block_num * self.block_size as u64;
        self.read_bytes(byte_offset, buf)
    }
    
    /// Get inode table block for a given block group
    fn get_inode_table_block(&self, group: u32) -> Result<u64, ()> {
        // Block Group Descriptor Table starts at block after superblock
        let gdt_block = if self.block_size == 1024 { 2 } else { 1 };
        let gdt_offset = gdt_block as u64 * self.block_size as u64
            + group as u64 * self.group_desc_size as u64;
        
        let mut desc_buf = [0u8; 64];
        let read_len = (self.group_desc_size as usize).min(64);
        self.read_bytes(gdt_offset, &mut desc_buf[..read_len])?;
        
        let lo = u32::from_le_bytes([desc_buf[8], desc_buf[9], desc_buf[10], desc_buf[11]]);
        let hi = if self.group_desc_size >= 64 {
            u32::from_le_bytes([desc_buf[0x28], desc_buf[0x29], desc_buf[0x2A], desc_buf[0x2B]])
        } else {
            0
        };
        
        Ok(lo as u64 | ((hi as u64) << 32))
    }
    
    /// Read an inode by number
    fn read_inode(&self, ino: u32) -> Result<Ext4Inode, ()> {
        if ino == 0 { return Err(()); }
        
        let group = (ino - 1) / self.inodes_per_group;
        let index = (ino - 1) % self.inodes_per_group;
        
        let inode_table = self.get_inode_table_block(group)?;
        let inode_offset = inode_table * self.block_size as u64
            + index as u64 * self.inode_size as u64;
        
        let mut buf = [0u8; 128];
        self.read_bytes(inode_offset, &mut buf)?;
        
        Ok(unsafe { core::ptr::read_unaligned(buf.as_ptr() as *const Ext4Inode) })
    }
    
    /// Get file size from inode (combines lo and hi)
    fn inode_size(&self, inode: &Ext4Inode) -> u64 {
        let lo = inode.i_size_lo as u64;
        let hi = if inode.i_mode & S_IFREG == S_IFREG {
            (inode.i_size_high as u64) << 32
        } else {
            0
        };
        lo | hi
    }
    
    /// Resolve extent tree to find physical block for a given logical block
    fn extent_lookup(&self, inode: &Ext4Inode, logical_block: u32) -> Result<u64, ()> {
        // The i_block field contains the extent tree root
        let raw = unsafe {
            core::slice::from_raw_parts(
                inode.i_block.as_ptr() as *const u8,
                60, // 15 * 4 bytes
            )
        };
        
        self.extent_search(raw, logical_block, 4) // max 4 levels of indirection
    }
    
    /// Search extent tree (recursive for internal nodes)
    fn extent_search(&self, data: &[u8], logical_block: u32, depth_limit: u32) -> Result<u64, ()> {
        if data.len() < 12 || depth_limit == 0 { return Err(()); }
        
        let header = unsafe { core::ptr::read_unaligned(data.as_ptr() as *const Ext4ExtentHeader) };
        
        if header.eh_magic != 0xF30A {
            return Err(()); // Invalid extent magic
        }
        
        let entries = header.eh_entries as usize;
        
        if header.eh_depth == 0 {
            // Leaf node: search extent entries
            for i in 0..entries {
                let offset = 12 + i * 12;
                if offset + 12 > data.len() { break; }
                
                let extent = unsafe {
                    core::ptr::read_unaligned(data[offset..].as_ptr() as *const Ext4Extent)
                };
                
                let start = extent.ee_block;
                let len = extent.ee_len as u32;
                
                if logical_block >= start && logical_block < start + len {
                    let phys_start = (extent.ee_start_hi as u64) << 32
                        | extent.ee_start_lo as u64;
                    let block_in_extent = (logical_block - start) as u64;
                    return Ok(phys_start + block_in_extent);
                }
            }
            Err(()) // Block not found in extents (sparse file hole)
        } else {
            // Internal node: find the right child
            let mut target_idx: Option<(usize, u64)> = None;
            
            for i in 0..entries {
                let offset = 12 + i * 12;
                if offset + 12 > data.len() { break; }
                
                let idx = unsafe {
                    core::ptr::read_unaligned(data[offset..].as_ptr() as *const Ext4ExtentIdx)
                };
                
                if logical_block >= idx.ei_block {
                    let child_block = (idx.ei_leaf_hi as u64) << 32
                        | idx.ei_leaf_lo as u64;
                    target_idx = Some((i, child_block));
                }
            }
            
            if let Some((_, child_block)) = target_idx {
                let mut child_data = vec![0u8; self.block_size as usize];
                self.read_block(child_block, &mut child_data)?;
                self.extent_search(&child_data, logical_block, depth_limit - 1)
            } else {
                Err(())
            }
        }
    }
    
    /// Read file data at a given file offset into buffer
    fn read_file_data(&self, inode: &Ext4Inode, file_offset: u64, buf: &mut [u8]) -> Result<usize, ()> {
        let file_size = self.inode_size(inode);
        if file_offset >= file_size {
            return Ok(0);
        }
        
        let read_len = ((file_size - file_offset) as usize).min(buf.len());
        if read_len == 0 { return Ok(0); }
        
        let block_size = self.block_size as u64;
        let mut remaining = read_len;
        let mut buf_offset = 0usize;
        let mut offset = file_offset;
        
        while remaining > 0 {
            let logical_block = (offset / block_size) as u32;
            let offset_in_block = (offset % block_size) as usize;
            let copy_len = (block_size as usize - offset_in_block).min(remaining);
            
            if (inode.i_flags & EXT4_EXTENTS_FL) != 0 {
                match self.extent_lookup(inode, logical_block) {
                    Ok(phys_block) => {
                        let mut block_buf = vec![0u8; block_size as usize];
                        self.read_block(phys_block, &mut block_buf)?;
                        buf[buf_offset..buf_offset + copy_len]
                            .copy_from_slice(&block_buf[offset_in_block..offset_in_block + copy_len]);
                    }
                    Err(()) => {
                        // Sparse hole: zero-fill
                        for b in &mut buf[buf_offset..buf_offset + copy_len] {
                            *b = 0;
                        }
                    }
                }
            } else {
                // Legacy block map (indirect blocks) — minimal support
                match self.legacy_block_lookup(inode, logical_block) {
                    Some(phys_block) if phys_block != 0 => {
                        let mut block_buf = vec![0u8; block_size as usize];
                        self.read_block(phys_block as u64, &mut block_buf)?;
                        buf[buf_offset..buf_offset + copy_len]
                            .copy_from_slice(&block_buf[offset_in_block..offset_in_block + copy_len]);
                    }
                    _ => {
                        // Sparse hole
                        for b in &mut buf[buf_offset..buf_offset + copy_len] {
                            *b = 0;
                        }
                    }
                }
            }
            
            buf_offset += copy_len;
            offset += copy_len as u64;
            remaining -= copy_len;
        }
        
        Ok(read_len)
    }
    
    /// Legacy block map (direct + single/double/triple indirect)
    fn legacy_block_lookup(&self, inode: &Ext4Inode, logical_block: u32) -> Option<u32> {
        let ptrs_per_block = self.block_size / 4; // u32 pointers per block
        
        if logical_block < 12 {
            // Direct blocks
            Some(inode.i_block[logical_block as usize])
        } else if logical_block < 12 + ptrs_per_block {
            // Single indirect
            let indirect_block = inode.i_block[12];
            if indirect_block == 0 { return Some(0); }
            self.read_indirect_entry(indirect_block as u64, (logical_block - 12) as usize)
        } else if logical_block < 12 + ptrs_per_block + ptrs_per_block * ptrs_per_block {
            // Double indirect
            let dind_block = inode.i_block[13];
            if dind_block == 0 { return Some(0); }
            let idx = logical_block - 12 - ptrs_per_block;
            let l1 = idx / ptrs_per_block;
            let l2 = idx % ptrs_per_block;
            let ind_block = self.read_indirect_entry(dind_block as u64, l1 as usize)?;
            if ind_block == 0 { return Some(0); }
            self.read_indirect_entry(ind_block as u64, l2 as usize)
        } else {
            // Triple indirect (very rare, skip for now)
            None
        }
    }
    
    /// Read a single u32 entry from an indirect block
    fn read_indirect_entry(&self, block: u64, index: usize) -> Option<u32> {
        let byte_offset = block * self.block_size as u64 + (index * 4) as u64;
        let mut buf = [0u8; 4];
        self.read_bytes(byte_offset, &mut buf).ok()?;
        Some(u32::from_le_bytes(buf))
    }
    
    /// Read directory entries from an inode
    fn read_dir_entries(&self, ino: u32) -> Result<Vec<(u32, String, u8)>, ()> {
        let inode = self.read_inode(ino)?;
        let file_size = self.inode_size(&inode);
        
        let mut entries = Vec::new();
        let mut offset = 0u64;
        
        while offset < file_size {
            let block_size = self.block_size as u64;
            let logical_block = (offset / block_size) as u32;
            let offset_in_block = (offset % block_size) as usize;
            
            // Read the block containing directory data
            let phys_block = if (inode.i_flags & EXT4_EXTENTS_FL) != 0 {
                self.extent_lookup(&inode, logical_block)?
            } else {
                self.legacy_block_lookup(&inode, logical_block)
                    .ok_or(())? as u64
            };
            
            let mut block_buf = vec![0u8; block_size as usize];
            self.read_block(phys_block, &mut block_buf)?;
            
            let mut pos = offset_in_block;
            while pos + 8 <= block_size as usize {
                let entry_inode = u32::from_le_bytes([
                    block_buf[pos], block_buf[pos+1], block_buf[pos+2], block_buf[pos+3]
                ]);
                let rec_len = u16::from_le_bytes([block_buf[pos+4], block_buf[pos+5]]) as usize;
                let name_len = block_buf[pos+6] as usize;
                let file_type = block_buf[pos+7];
                
                if rec_len == 0 { break; } // Prevent infinite loop
                
                if entry_inode != 0 && name_len > 0 && pos + 8 + name_len <= block_buf.len() {
                    let name = core::str::from_utf8(&block_buf[pos+8..pos+8+name_len])
                        .unwrap_or("")
                        .to_string();
                    if !name.is_empty() {
                        entries.push((entry_inode, name, file_type));
                    }
                }
                
                pos += rec_len;
                offset += rec_len as u64;
            }
            
            // If we haven't advanced to next block boundary, do so
            let next_block_offset = ((offset / block_size) + 1) * block_size;
            if offset < next_block_offset && pos >= block_size as usize {
                offset = next_block_offset;
            }
        }
        
        Ok(entries)
    }
    
    /// Lookup a name in a directory, return inode number
    fn dir_lookup(&self, dir_ino: u32, name: &str) -> Result<u32, ()> {
        let entries = self.read_dir_entries(dir_ino)?;
        for (ino, entry_name, _ft) in &entries {
            if entry_name == name {
                return Ok(*ino);
            }
        }
        Err(())
    }
    
    /// Resolve a path to an inode number starting from root
    fn resolve_path(&self, path: &str) -> Result<u32, ()> {
        let path = path.trim_start_matches('/');
        if path.is_empty() {
            return Ok(EXT4_ROOT_INO);
        }
        
        let mut current = EXT4_ROOT_INO;
        for component in path.split('/') {
            if component.is_empty() || component == "." { continue; }
            current = self.dir_lookup(current, component)?;
        }
        Ok(current)
    }
    
    /// Convert ext4 file type to VFS FileType
    fn inode_file_type(&self, inode: &Ext4Inode) -> FileType {
        match inode.i_mode & S_IFMT {
            S_IFREG => FileType::Regular,
            S_IFDIR => FileType::Directory,
            S_IFLNK => FileType::Symlink,
            S_IFCHR => FileType::CharDevice,
            S_IFBLK => FileType::BlockDevice,
            _ => FileType::Regular,
        }
    }
}

fn dir_entry_file_type(ft: u8) -> FileType {
    match ft {
        EXT4_FT_REG_FILE => FileType::Regular,
        EXT4_FT_DIR => FileType::Directory,
        EXT4_FT_SYMLINK => FileType::Symlink,
        EXT4_FT_CHRDEV => FileType::CharDevice,
        EXT4_FT_BLKDEV => FileType::BlockDevice,
        _ => FileType::Regular,
    }
}

// ============================================================================
// VFS Integration 
// ============================================================================

/// ext4 file handle (read-only)
struct Ext4File {
    fs: Arc<Ext4Fs>,
    ino: u32,
}

impl FileOps for Ext4File {
    fn read(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        let inner = self.fs.inner.lock();
        let inode = inner.read_inode(self.ino).map_err(|_| VfsError::IoError)?;
        inner.read_file_data(&inode, offset, buf).map_err(|_| VfsError::IoError)
    }
    
    fn write(&self, _offset: u64, _buf: &[u8]) -> VfsResult<usize> {
        Err(VfsError::ReadOnly) // Read-only filesystem
    }
    
    fn stat(&self) -> VfsResult<Stat> {
        let inner = self.fs.inner.lock();
        let inode = inner.read_inode(self.ino).map_err(|_| VfsError::IoError)?;
        let file_size = inner.inode_size(&inode);
        let ft = inner.inode_file_type(&inode);
        
        Ok(Stat {
            ino: self.ino as u64,
            file_type: ft,
            size: file_size,
            blocks: inode.i_blocks_lo as u64,
            block_size: inner.block_size,
            mode: inode.i_mode as u32,
            uid: inode.i_uid as u32,
            gid: inode.i_gid as u32,
            atime: inode.i_atime as u64,
            mtime: inode.i_mtime as u64,
            ctime: inode.i_ctime as u64,
        })
    }
}

/// ext4 directory handle (read-only)
struct Ext4Dir {
    fs: Arc<Ext4Fs>,
    ino: u32,
}

impl DirOps for Ext4Dir {
    fn lookup(&self, name: &str) -> VfsResult<Ino> {
        let inner = self.fs.inner.lock();
        inner.dir_lookup(self.ino, name)
            .map(|ino| ino as u64)
            .map_err(|_| VfsError::NotFound)
    }
    
    fn readdir(&self) -> VfsResult<Vec<DirEntry>> {
        let inner = self.fs.inner.lock();
        let entries = inner.read_dir_entries(self.ino)
            .map_err(|_| VfsError::IoError)?;
        
        Ok(entries.into_iter()
            .filter(|(_, name, _)| name != "." && name != "..")
            .map(|(ino, name, ft)| DirEntry {
                name,
                ino: ino as u64,
                file_type: dir_entry_file_type(ft),
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
        let inner = self.fs.inner.lock();
        let inode = inner.read_inode(self.ino).map_err(|_| VfsError::IoError)?;
        
        Ok(Stat {
            ino: self.ino as u64,
            file_type: FileType::Directory,
            size: inner.inode_size(&inode),
            blocks: inode.i_blocks_lo as u64,
            block_size: inner.block_size,
            mode: inode.i_mode as u32,
            uid: inode.i_uid as u32,
            gid: inode.i_gid as u32,
            atime: inode.i_atime as u64,
            mtime: inode.i_mtime as u64,
            ctime: inode.i_ctime as u64,
        })
    }
}

/// FileSystem implementation for ext4
impl FileSystem for Ext4Fs {
    fn name(&self) -> &str { "ext4" }
    
    fn root_inode(&self) -> Ino { EXT4_ROOT_INO as u64 }
    
    fn get_file(&self, ino: Ino) -> VfsResult<Arc<dyn FileOps>> {
        let inner = self.inner.lock();
        let inode = inner.read_inode(ino as u32).map_err(|_| VfsError::NotFound)?;
        let ft = inner.inode_file_type(&inode);
        if ft == FileType::Directory {
            return Err(VfsError::IsDirectory);
        }
        drop(inner);
        
        // We need an Arc<Ext4Fs> but we only have &self.
        // This is a known limitation — caller should hold an Arc and pass it.
        // For now we create a wrapper that stores the raw data needed.
        Ok(Arc::new(Ext4FileStandalone {
            ino: ino as u32,
            device: self.inner.lock().device.clone(),
            block_size: self.inner.lock().block_size,
            inode_size: self.inner.lock().inode_size,
            inodes_per_group: self.inner.lock().inodes_per_group,
            blocks_per_group: self.inner.lock().blocks_per_group,
            first_data_block: self.inner.lock().first_data_block,
            group_desc_size: self.inner.lock().group_desc_size,
        }))
    }
    
    fn get_dir(&self, ino: Ino) -> VfsResult<Arc<dyn DirOps>> {
        let inner = self.inner.lock();
        let inode = inner.read_inode(ino as u32).map_err(|_| VfsError::NotFound)?;
        let ft = inner.inode_file_type(&inode);
        if ft != FileType::Directory {
            return Err(VfsError::NotDirectory);
        }
        drop(inner);
        
        Ok(Arc::new(Ext4DirStandalone {
            ino: ino as u32,
            device: self.inner.lock().device.clone(),
            block_size: self.inner.lock().block_size,
            inode_size: self.inner.lock().inode_size,
            inodes_per_group: self.inner.lock().inodes_per_group,
            blocks_per_group: self.inner.lock().blocks_per_group,
            first_data_block: self.inner.lock().first_data_block,
            group_desc_size: self.inner.lock().group_desc_size,
        }))
    }
    
    fn stat(&self, ino: Ino) -> VfsResult<Stat> {
        let inner = self.inner.lock();
        let inode = inner.read_inode(ino as u32).map_err(|_| VfsError::NotFound)?;
        let file_size = inner.inode_size(&inode);
        let ft = inner.inode_file_type(&inode);
        
        Ok(Stat {
            ino,
            file_type: ft,
            size: file_size,
            blocks: inode.i_blocks_lo as u64,
            block_size: inner.block_size,
            mode: inode.i_mode as u32,
            uid: inode.i_uid as u32,
            gid: inode.i_gid as u32,
            atime: inode.i_atime as u64,
            mtime: inode.i_mtime as u64,
            ctime: inode.i_ctime as u64,
        })
    }
}

/// Standalone file handle that carries its own fs params (avoids Arc<Ext4Fs> issue)
struct Ext4FileStandalone {
    ino: u32,
    device: Arc<dyn BlockDevice>,
    block_size: u32,
    inode_size: u16,
    inodes_per_group: u32,
    blocks_per_group: u32,
    first_data_block: u32,
    group_desc_size: u32,
}

impl Ext4FileStandalone {
    fn make_inner(&self) -> Ext4FsInner {
        Ext4FsInner {
            block_size: self.block_size,
            inode_size: self.inode_size,
            inodes_per_group: self.inodes_per_group,
            blocks_per_group: self.blocks_per_group,
            first_data_block: self.first_data_block,
            group_desc_size: self.group_desc_size,
            total_groups: 0,
            device: self.device.clone(),
        }
    }
}

impl FileOps for Ext4FileStandalone {
    fn read(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        let inner = self.make_inner();
        let inode = inner.read_inode(self.ino).map_err(|_| VfsError::IoError)?;
        inner.read_file_data(&inode, offset, buf).map_err(|_| VfsError::IoError)
    }
    
    fn write(&self, _offset: u64, _buf: &[u8]) -> VfsResult<usize> {
        Err(VfsError::ReadOnly)
    }
    
    fn stat(&self) -> VfsResult<Stat> {
        let inner = self.make_inner();
        let inode = inner.read_inode(self.ino).map_err(|_| VfsError::IoError)?;
        let file_size = inner.inode_size(&inode);
        let ft = inner.inode_file_type(&inode);
        Ok(Stat {
            ino: self.ino as u64,
            file_type: ft,
            size: file_size,
            blocks: inode.i_blocks_lo as u64,
            block_size: inner.block_size,
            mode: inode.i_mode as u32,
            uid: inode.i_uid as u32,
            gid: inode.i_gid as u32,
            atime: inode.i_atime as u64,
            mtime: inode.i_mtime as u64,
            ctime: inode.i_ctime as u64,
        })
    }
}

/// Standalone directory handle
struct Ext4DirStandalone {
    ino: u32,
    device: Arc<dyn BlockDevice>,
    block_size: u32,
    inode_size: u16,
    inodes_per_group: u32,
    blocks_per_group: u32,
    first_data_block: u32,
    group_desc_size: u32,
}

impl Ext4DirStandalone {
    fn make_inner(&self) -> Ext4FsInner {
        Ext4FsInner {
            block_size: self.block_size,
            inode_size: self.inode_size,
            inodes_per_group: self.inodes_per_group,
            blocks_per_group: self.blocks_per_group,
            first_data_block: self.first_data_block,
            group_desc_size: self.group_desc_size,
            total_groups: 0,
            device: self.device.clone(),
        }
    }
}

impl DirOps for Ext4DirStandalone {
    fn lookup(&self, name: &str) -> VfsResult<Ino> {
        let inner = self.make_inner();
        inner.dir_lookup(self.ino, name)
            .map(|ino| ino as u64)
            .map_err(|_| VfsError::NotFound)
    }
    
    fn readdir(&self) -> VfsResult<Vec<DirEntry>> {
        let inner = self.make_inner();
        let entries = inner.read_dir_entries(self.ino)
            .map_err(|_| VfsError::IoError)?;
        
        Ok(entries.into_iter()
            .filter(|(_, name, _)| name != "." && name != "..")
            .map(|(ino, name, ft)| DirEntry {
                name,
                ino: ino as u64,
                file_type: dir_entry_file_type(ft),
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
        let inode = inner.read_inode(self.ino).map_err(|_| VfsError::IoError)?;
        Ok(Stat {
            ino: self.ino as u64,
            file_type: FileType::Directory,
            size: inner.inode_size(&inode),
            blocks: inode.i_blocks_lo as u64,
            block_size: inner.block_size,
            mode: inode.i_mode as u32,
            uid: inode.i_uid as u32,
            gid: inode.i_gid as u32,
            atime: inode.i_atime as u64,
            mtime: inode.i_mtime as u64,
            ctime: inode.i_ctime as u64,
        })
    }
}

// ============================================================================
// Mount / Probe
// ============================================================================

/// Try to mount an ext4 filesystem from a block device
pub fn mount(device: Arc<dyn BlockDevice>) -> Result<Arc<Ext4Fs>, &'static str> {
    // Read superblock (at byte offset 1024)
    let mut sb_buf = [0u8; 256]; // Only need first 256 bytes of superblock
    let sector_size = device.sector_size();
    
    // Read sectors covering offset 1024..1280
    let start_sector = SUPERBLOCK_OFFSET / sector_size as u64;
    let mut raw = vec![0u8; sector_size * 4]; // Read several sectors
    for i in 0..4u64 {
        let _ = device.read_sector(start_sector + i, &mut raw[i as usize * sector_size..(i as usize + 1) * sector_size]);
    }
    
    let sb_offset = (SUPERBLOCK_OFFSET - start_sector * sector_size as u64) as usize;
    if sb_offset + 256 > raw.len() {
        return Err("Superblock read overflow");
    }
    sb_buf.copy_from_slice(&raw[sb_offset..sb_offset + 256]);
    
    let sb = unsafe { core::ptr::read_unaligned(sb_buf.as_ptr() as *const Ext4SuperBlock) };
    
    // Verify magic
    if sb.s_magic != EXT4_SUPER_MAGIC {
        return Err("Not an ext4 filesystem (bad magic)");
    }
    
    let block_size = 1024u32 << sb.s_log_block_size;
    let inode_size = if sb.s_rev_level >= 1 { sb.s_inode_size } else { 128 };
    
    // Group descriptor size: 32 for non-64bit, 64 for 64-bit
    let is_64bit = (sb.s_feature_incompat & 0x80) != 0; // INCOMPAT_64BIT
    let group_desc_size = if is_64bit { 64u32 } else { 32 };
    
    let total_groups = (sb.s_blocks_count_lo + sb.s_blocks_per_group - 1) / sb.s_blocks_per_group;
    
    let volume_name = core::str::from_utf8(&sb.s_volume_name)
        .unwrap_or("")
        .trim_end_matches('\0');
    
    crate::serial_println!("[ext4] Mounted: \"{}\" block_size={} inode_size={} groups={} 64bit={}",
        volume_name, block_size, inode_size, total_groups, is_64bit);
    crate::serial_println!("[ext4] {} inodes, {} blocks per group",
        sb.s_inodes_per_group, sb.s_blocks_per_group);
    
    let fs = Arc::new(Ext4Fs {
        inner: Mutex::new(Ext4FsInner {
            block_size,
            inode_size,
            inodes_per_group: sb.s_inodes_per_group,
            blocks_per_group: sb.s_blocks_per_group,
            first_data_block: sb.s_first_data_block,
            group_desc_size,
            total_groups,
            device,
        }),
    });
    
    Ok(fs)
}

/// Probe a block device to see if it contains an ext4 filesystem
pub fn probe(device: &dyn BlockDevice) -> bool {
    let sector_size = device.sector_size();
    let start_sector = SUPERBLOCK_OFFSET / sector_size as u64;
    
    let mut raw = vec![0u8; sector_size * 4];
    for i in 0..4u64 {
        if device.read_sector(start_sector + i, &mut raw[i as usize * sector_size..(i as usize + 1) * sector_size]).is_err() {
            return false;
        }
    }
    
    let sb_offset = (SUPERBLOCK_OFFSET - start_sector * sector_size as u64) as usize;
    if sb_offset + 2 + 0x38 > raw.len() { return false; }
    
    let magic = u16::from_le_bytes([raw[sb_offset + 0x38], raw[sb_offset + 0x39]]);
    magic == EXT4_SUPER_MAGIC
}

/// Read a file from ext4 by path (convenience function)
pub fn read_file(fs: &Ext4Fs, path: &str) -> Result<Vec<u8>, &'static str> {
    let inner = fs.inner.lock();
    let ino = inner.resolve_path(path).map_err(|_| "File not found")?;
    let inode = inner.read_inode(ino).map_err(|_| "Failed to read inode")?;
    
    let ft = inner.inode_file_type(&inode);
    if ft == FileType::Directory {
        return Err("Is a directory");
    }
    
    let size = inner.inode_size(&inode);
    if size > 64 * 1024 * 1024 {
        return Err("File too large (>64MB)");
    }
    
    let mut buf = vec![0u8; size as usize];
    inner.read_file_data(&inode, 0, &mut buf).map_err(|_| "Read error")?;
    Ok(buf)
}

/// List directory contents from ext4 by path (convenience function)
pub fn list_dir(fs: &Ext4Fs, path: &str) -> Result<Vec<(String, FileType, u64)>, &'static str> {
    let inner = fs.inner.lock();
    let ino = inner.resolve_path(path).map_err(|_| "Directory not found")?;
    let inode = inner.read_inode(ino).map_err(|_| "Failed to read inode")?;
    
    let ft = inner.inode_file_type(&inode);
    if ft != FileType::Directory {
        return Err("Not a directory");
    }
    
    let entries = inner.read_dir_entries(ino).map_err(|_| "Read error")?;
    
    let mut result = Vec::new();
    for (entry_ino, name, ft_byte) in entries {
        if name == "." || name == ".." { continue; }
        
        let file_type = dir_entry_file_type(ft_byte);
        let size = if let Ok(entry_inode) = inner.read_inode(entry_ino) {
            inner.inode_size(&entry_inode)
        } else {
            0
        };
        
        result.push((name, file_type, size));
    }
    
    Ok(result)
}
