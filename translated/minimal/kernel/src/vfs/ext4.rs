












use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::vec;
use alloc::sync::Arc;
use alloc::format;
use spin::Mutex;

use super::{Bx, Bv, Au, FileType, Stat, Ap, E, VfsError, K};
use super::fat32::Ak;






const ATP_: u16 = 0xEF53;


const YV_: u64 = 1024;


const ATO_: u32 = 0x00080000;


const ADO_: u32 = 2;


const DPA_: u8 = 0;
const BXR_: u8 = 1;
const BXQ_: u8 = 2;
const BXP_: u8 = 3;
const BXO_: u8 = 4;
const DOY_: u8 = 5;
const DOZ_: u8 = 6;
const BXS_: u8 = 7;


const BJA_: u16 = 0xF000;
const QS_: u16 = 0x8000;
const AKJ_: u16 = 0x4000;
const AKK_: u16 = 0xA000;
const YX_: u16 = 0x2000;
const DAW_: u16 = 0x6000;


#[repr(C)]
#[derive(Clone, Copy)]
struct Ajk {
    s_inodes_count: u32,         
    s_blocks_count_lo: u32,      
    s_r_blocks_count_lo: u32,    
    s_free_blocks_count_lo: u32, 
    s_free_inodes_count: u32,    
    s_first_data_block: u32,     
    s_log_block_size: u32,       
    s_log_cluster_size: u32,     
    s_blocks_per_group: u32,     
    s_clusters_per_group: u32,   
    s_inodes_per_group: u32,     
    s_mtime: u32,                
    s_wtime: u32,                
    s_mnt_count: u16,            
    s_max_mnt_count: u16,        
    s_magic: u16,                
    s_state: u16,                
    s_errors: u16,               
    s_minor_rev_level: u16,      
    s_lastcheck: u32,            
    s_checkinterval: u32,        
    s_creator_os: u32,           
    s_rev_level: u32,            
    s_def_resuid: u16,           
    s_def_resgid: u16,           
    
    s_first_ino: u32,            
    s_inode_size: u16,           
    s_block_group_nr: u16,       
    s_feature_compat: u32,       
    s_feature_incompat: u32,     
    s_feature_ro_compat: u32,    
    s_uuid: [u8; 16],            
    s_volume_name: [u8; 16],     
    _pad1: [u8; 168],            
    
    
}


#[repr(C)]
#[derive(Clone, Copy)]
struct Avk {
    bg_block_bitmap_lo: u32,     
    bg_inode_bitmap_lo: u32,     
    bg_inode_table_lo: u32,      
    bg_free_blocks_count_lo: u16,
    bg_free_inodes_count_lo: u16,
    bg_used_dirs_count_lo: u16,  
    bg_flags: u16,               
    bg_exclude_bitmap_lo: u32,   
    bg_block_bitmap_csum_lo: u16,
    bg_inode_bitmap_csum_lo: u16,
    bg_itable_unused_lo: u16,    
    bg_checksum: u16,            
    
    bg_block_bitmap_hi: u32,     
    bg_inode_bitmap_hi: u32,     
    bg_inode_table_hi: u32,      
}


#[repr(C)]
#[derive(Clone, Copy)]
struct Hw {
    i_mode: u16,                 
    i_uid: u16,                  
    i_size_lo: u32,              
    i_atime: u32,                
    i_ctime: u32,                
    i_mtime: u32,                
    i_dtime: u32,                
    i_gid: u16,                  
    i_links_count: u16,          
    i_blocks_lo: u32,            
    i_flags: u32,                
    i_osd1: u32,                 
    i_block: [u32; 15],          
    i_generation: u32,           
    i_file_acl_lo: u32,          
    i_size_high: u32,            
    _pad: [u8; 16],              
}


#[repr(C)]
#[derive(Clone, Copy)]
struct Ajh {
    eh_magic: u16,      
    eh_entries: u16,     
    eh_max: u16,         
    eh_depth: u16,       
    eh_generation: u32,  
}


#[repr(C)]
#[derive(Clone, Copy)]
struct Ajg {
    ee_block: u32,       
    ee_len: u16,         
    ee_start_hi: u16,    
    ee_start_lo: u32,    
}


#[repr(C)]
#[derive(Clone, Copy)]
struct Aji {
    ei_block: u32,       
    ei_leaf_lo: u32,     
    ei_leaf_hi: u16,     
    _pad: u16,
}


#[repr(C)]
struct Avj {
    inode: u32,          
    eyb: u16,        
    name_len: u8,        
    file_type: u8,       
    
}






pub struct Hu {
    inner: Mutex<Hv>,
}

struct Hv {
    block_size: u32,
    inode_size: u16,
    inodes_per_group: u32,
    blocks_per_group: u32,
    first_data_block: u32,
    group_desc_size: u32,
    ece: u32,
    device: Arc<dyn Ak>,
}

impl Hv {
    
    fn read_bytes(&self, uo: u64, buf: &mut [u8]) -> Result<(), ()> {
        let sector_size = self.device.sector_size() as u64;
        let start_sector = uo / sector_size;
        let afl = (uo % sector_size) as usize;
        
        
        let total_bytes = afl + buf.len();
        let gkd = (total_bytes + sector_size as usize - 1) / sector_size as usize;
        
        let mut mx = vec![0u8; sector_size as usize];
        let mut ck = buf.len();
        let mut yj = 0usize;
        
        for i in 0..gkd {
            self.device.read_sector(start_sector + i as u64, &mut mx)?;
            
            let zl = if i == 0 { afl } else { 0 };
            let mb = (sector_size as usize - zl).min(ck);
            
            buf[yj..yj + mb]
                .copy_from_slice(&mx[zl..zl + mb]);
            
            yj += mb;
            ck -= mb;
        }
        
        Ok(())
    }
    
    
    fn read_block(&self, block_num: u64, buf: &mut [u8]) -> Result<(), ()> {
        let uo = block_num * self.block_size as u64;
        self.read_bytes(uo, buf)
    }
    
    
    fn get_inode_table_block(&self, bbz: u32) -> Result<u64, ()> {
        
        let mbm = if self.block_size == 1024 { 2 } else { 1 };
        let mbn = mbm as u64 * self.block_size as u64
            + bbz as u64 * self.group_desc_size as u64;
        
        let mut brw = [0u8; 64];
        let arx = (self.group_desc_size as usize).min(64);
        self.read_bytes(mbn, &mut brw[..arx])?;
        
        let lo = u32::from_le_bytes([brw[8], brw[9], brw[10], brw[11]]);
        let hi = if self.group_desc_size >= 64 {
            u32::from_le_bytes([brw[0x28], brw[0x29], brw[0x2A], brw[0x2B]])
        } else {
            0
        };
        
        Ok(lo as u64 | ((hi as u64) << 32))
    }
    
    
    fn read_inode(&self, ino: u32) -> Result<Hw, ()> {
        if ino == 0 { return Err(()); }
        
        let bbz = (ino - 1) / self.inodes_per_group;
        let index = (ino - 1) % self.inodes_per_group;
        
        let mqi = self.get_inode_table_block(bbz)?;
        let mqg = mqi * self.block_size as u64
            + index as u64 * self.inode_size as u64;
        
        let mut buf = [0u8; 128];
        self.read_bytes(mqg, &mut buf)?;
        
        Ok(unsafe { core::ptr::read_unaligned(buf.as_ptr() as *const Hw) })
    }
    
    
    fn inode_size(&self, inode: &Hw) -> u64 {
        let lo = inode.i_size_lo as u64;
        let hi = if inode.i_mode & QS_ == QS_ {
            (inode.i_size_high as u64) << 32
        } else {
            0
        };
        lo | hi
    }
    
    
    fn extent_lookup(&self, inode: &Hw, agq: u32) -> Result<u64, ()> {
        
        let dm = unsafe {
            core::slice::from_raw_parts(
                inode.i_block.as_ptr() as *const u8,
                60, 
            )
        };
        
        self.extent_search(dm, agq, 4) 
    }
    
    
    fn extent_search(&self, data: &[u8], agq: u32, depth_limit: u32) -> Result<u64, ()> {
        if data.len() < 12 || depth_limit == 0 { return Err(()); }
        
        let header = unsafe { core::ptr::read_unaligned(data.as_ptr() as *const Ajh) };
        
        if header.eh_magic != 0xF30A {
            return Err(()); 
        }
        
        let entries = header.eh_entries as usize;
        
        if header.eh_depth == 0 {
            
            for i in 0..entries {
                let offset = 12 + i * 12;
                if offset + 12 > data.len() { break; }
                
                let elw = unsafe {
                    core::ptr::read_unaligned(data[offset..].as_ptr() as *const Ajg)
                };
                
                let start = elw.ee_block;
                let len = elw.ee_len as u32;
                
                if agq >= start && agq < start + len {
                    let gmy = (elw.ee_start_hi as u64) << 32
                        | elw.ee_start_lo as u64;
                    let kct = (agq - start) as u64;
                    return Ok(gmy + kct);
                }
            }
            Err(()) 
        } else {
            
            let mut jlp: Option<(usize, u64)> = None;
            
            for i in 0..entries {
                let offset = 12 + i * 12;
                if offset + 12 > data.len() { break; }
                
                let idx = unsafe {
                    core::ptr::read_unaligned(data[offset..].as_ptr() as *const Aji)
                };
                
                if agq >= idx.ei_block {
                    let fln = (idx.ei_leaf_hi as u64) << 32
                        | idx.ei_leaf_lo as u64;
                    jlp = Some((i, fln));
                }
            }
            
            if let Some((_, fln)) = jlp {
                let mut hkv = vec![0u8; self.block_size as usize];
                self.read_block(fln, &mut hkv)?;
                self.extent_search(&hkv, agq, depth_limit - 1)
            } else {
                Err(())
            }
        }
    }
    
    
    fn read_file_data(&self, inode: &Hw, aaw: u64, buf: &mut [u8]) -> Result<usize, ()> {
        let file_size = self.inode_size(inode);
        if aaw >= file_size {
            return Ok(0);
        }
        
        let arx = ((file_size - aaw) as usize).min(buf.len());
        if arx == 0 { return Ok(0); }
        
        let block_size = self.block_size as u64;
        let mut ck = arx;
        let mut yj = 0usize;
        let mut offset = aaw;
        
        while ck > 0 {
            let agq = (offset / block_size) as u32;
            let cni = (offset % block_size) as usize;
            let mb = (block_size as usize - cni).min(ck);
            
            if (inode.i_flags & ATO_) != 0 {
                match self.extent_lookup(inode, agq) {
                    Ok(aum) => {
                        let mut se = vec![0u8; block_size as usize];
                        self.read_block(aum, &mut se)?;
                        buf[yj..yj + mb]
                            .copy_from_slice(&se[cni..cni + mb]);
                    }
                    Err(()) => {
                        
                        for b in &mut buf[yj..yj + mb] {
                            *b = 0;
                        }
                    }
                }
            } else {
                
                match self.legacy_block_lookup(inode, agq) {
                    Some(aum) if aum != 0 => {
                        let mut se = vec![0u8; block_size as usize];
                        self.read_block(aum as u64, &mut se)?;
                        buf[yj..yj + mb]
                            .copy_from_slice(&se[cni..cni + mb]);
                    }
                    _ => {
                        
                        for b in &mut buf[yj..yj + mb] {
                            *b = 0;
                        }
                    }
                }
            }
            
            yj += mb;
            offset += mb as u64;
            ck -= mb;
        }
        
        Ok(arx)
    }
    
    
    fn legacy_block_lookup(&self, inode: &Hw, agq: u32) -> Option<u32> {
        let com = self.block_size / 4; 
        
        if agq < 12 {
            
            Some(inode.i_block[agq as usize])
        } else if agq < 12 + com {
            
            let igl = inode.i_block[12];
            if igl == 0 { return Some(0); }
            self.read_indirect_entry(igl as u64, (agq - 12) as usize)
        } else if agq < 12 + com + com * com {
            
            let hsh = inode.i_block[13];
            if hsh == 0 { return Some(0); }
            let idx = agq - 12 - com;
            let clt = idx / com;
            let alv = idx % com;
            let igh = self.read_indirect_entry(hsh as u64, clt as usize)?;
            if igh == 0 { return Some(0); }
            self.read_indirect_entry(igh as u64, alv as usize)
        } else {
            
            None
        }
    }
    
    
    fn read_indirect_entry(&self, block: u64, index: usize) -> Option<u32> {
        let uo = block * self.block_size as u64 + (index * 4) as u64;
        let mut buf = [0u8; 4];
        self.read_bytes(uo, &mut buf).ok()?;
        Some(u32::from_le_bytes(buf))
    }
    
    
    fn read_dir_entries(&self, ino: u32) -> Result<Vec<(u32, String, u8)>, ()> {
        let inode = self.read_inode(ino)?;
        let file_size = self.inode_size(&inode);
        
        let mut entries = Vec::new();
        let mut offset = 0u64;
        
        while offset < file_size {
            let block_size = self.block_size as u64;
            let agq = (offset / block_size) as u32;
            let cni = (offset % block_size) as usize;
            
            
            let aum = if (inode.i_flags & ATO_) != 0 {
                self.extent_lookup(&inode, agq)?
            } else {
                self.legacy_block_lookup(&inode, agq)
                    .ok_or(())? as u64
            };
            
            let mut se = vec![0u8; block_size as usize];
            self.read_block(aum, &mut se)?;
            
            let mut pos = cni;
            while pos + 8 <= block_size as usize {
                let elm = u32::from_le_bytes([
                    se[pos], se[pos+1], se[pos+2], se[pos+3]
                ]);
                let eyb = u16::from_le_bytes([se[pos+4], se[pos+5]]) as usize;
                let name_len = se[pos+6] as usize;
                let file_type = se[pos+7];
                
                if eyb == 0 { break; } 
                
                if elm != 0 && name_len > 0 && pos + 8 + name_len <= se.len() {
                    let name = core::str::from_utf8(&se[pos+8..pos+8+name_len])
                        .unwrap_or("")
                        .to_string();
                    if !name.is_empty() {
                        entries.push((elm, name, file_type));
                    }
                }
                
                pos += eyb;
                offset += eyb as u64;
            }
            
            
            let iqm = ((offset / block_size) + 1) * block_size;
            if offset < iqm && pos >= block_size as usize {
                offset = iqm;
            }
        }
        
        Ok(entries)
    }
    
    
    fn dir_lookup(&self, dir_ino: u32, name: &str) -> Result<u32, ()> {
        let entries = self.read_dir_entries(dir_ino)?;
        for (ino, bbl, _ft) in &entries {
            if bbl == name {
                return Ok(*ino);
            }
        }
        Err(())
    }
    
    
    fn resolve_path(&self, path: &str) -> Result<u32, ()> {
        let path = path.trim_start_matches('/');
        if path.is_empty() {
            return Ok(ADO_);
        }
        
        let mut current = ADO_;
        for chn in path.split('/') {
            if chn.is_empty() || chn == "." { continue; }
            current = self.dir_lookup(current, chn)?;
        }
        Ok(current)
    }
    
    
    fn inode_file_type(&self, inode: &Hw) -> FileType {
        match inode.i_mode & BJA_ {
            QS_ => FileType::Regular,
            AKJ_ => FileType::Directory,
            AKK_ => FileType::Symlink,
            YX_ => FileType::CharDevice,
            DAW_ => FileType::Ak,
            _ => FileType::Regular,
        }
    }
}

fn fsh(qk: u8) -> FileType {
    match qk {
        BXR_ => FileType::Regular,
        BXQ_ => FileType::Directory,
        BXS_ => FileType::Symlink,
        BXP_ => FileType::CharDevice,
        BXO_ => FileType::Ak,
        _ => FileType::Regular,
    }
}






struct Ajj {
    fs: Arc<Hu>,
    ino: u32,
}

impl Bx for Ajj {
    fn read(&self, offset: u64, buf: &mut [u8]) -> E<usize> {
        let inner = self.fs.inner.lock();
        let inode = inner.read_inode(self.ino).map_err(|_| VfsError::IoError)?;
        inner.read_file_data(&inode, offset, buf).map_err(|_| VfsError::IoError)
    }
    
    fn write(&self, bkm: u64, _buf: &[u8]) -> E<usize> {
        Err(VfsError::ReadOnly) 
    }
    
    fn stat(&self) -> E<Stat> {
        let inner = self.fs.inner.lock();
        let inode = inner.read_inode(self.ino).map_err(|_| VfsError::IoError)?;
        let file_size = inner.inode_size(&inode);
        let qk = inner.inode_file_type(&inode);
        
        Ok(Stat {
            ino: self.ino as u64,
            file_type: qk,
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


struct Ajf {
    fs: Arc<Hu>,
    ino: u32,
}

impl Bv for Ajf {
    fn lookup(&self, name: &str) -> E<K> {
        let inner = self.fs.inner.lock();
        inner.dir_lookup(self.ino, name)
            .map(|ino| ino as u64)
            .map_err(|_| VfsError::NotFound)
    }
    
    fn readdir(&self) -> E<Vec<Ap>> {
        let inner = self.fs.inner.lock();
        let entries = inner.read_dir_entries(self.ino)
            .map_err(|_| VfsError::IoError)?;
        
        Ok(entries.into_iter()
            .filter(|(_, name, _)| name != "." && name != "..")
            .map(|(ino, name, qk)| Ap {
                name,
                ino: ino as u64,
                file_type: fsh(qk),
            })
            .collect())
    }
    
    fn create(&self, _name: &str, _file_type: FileType) -> E<K> {
        Err(VfsError::ReadOnly)
    }
    
    fn unlink(&self, _name: &str) -> E<()> {
        Err(VfsError::ReadOnly)
    }
    
    fn stat(&self) -> E<Stat> {
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


impl Au for Hu {
    fn name(&self) -> &str { "ext4" }
    
    fn root_inode(&self) -> K { ADO_ as u64 }
    
    fn get_file(&self, ino: K) -> E<Arc<dyn Bx>> {
        let inner = self.inner.lock();
        let inode = inner.read_inode(ino as u32).map_err(|_| VfsError::NotFound)?;
        let qk = inner.inode_file_type(&inode);
        if qk == FileType::Directory {
            return Err(VfsError::IsDirectory);
        }
        drop(inner);
        
        
        
        
        Ok(Arc::new(Rz {
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
    
    fn get_dir(&self, ino: K) -> E<Arc<dyn Bv>> {
        let inner = self.inner.lock();
        let inode = inner.read_inode(ino as u32).map_err(|_| VfsError::NotFound)?;
        let qk = inner.inode_file_type(&inode);
        if qk != FileType::Directory {
            return Err(VfsError::NotDirectory);
        }
        drop(inner);
        
        Ok(Arc::new(Ry {
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
    
    fn stat(&self, ino: K) -> E<Stat> {
        let inner = self.inner.lock();
        let inode = inner.read_inode(ino as u32).map_err(|_| VfsError::NotFound)?;
        let file_size = inner.inode_size(&inode);
        let qk = inner.inode_file_type(&inode);
        
        Ok(Stat {
            ino,
            file_type: qk,
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


struct Rz {
    ino: u32,
    device: Arc<dyn Ak>,
    block_size: u32,
    inode_size: u16,
    inodes_per_group: u32,
    blocks_per_group: u32,
    first_data_block: u32,
    group_desc_size: u32,
}

impl Rz {
    fn make_inner(&self) -> Hv {
        Hv {
            block_size: self.block_size,
            inode_size: self.inode_size,
            inodes_per_group: self.inodes_per_group,
            blocks_per_group: self.blocks_per_group,
            first_data_block: self.first_data_block,
            group_desc_size: self.group_desc_size,
            ece: 0,
            device: self.device.clone(),
        }
    }
}

impl Bx for Rz {
    fn read(&self, offset: u64, buf: &mut [u8]) -> E<usize> {
        let inner = self.make_inner();
        let inode = inner.read_inode(self.ino).map_err(|_| VfsError::IoError)?;
        inner.read_file_data(&inode, offset, buf).map_err(|_| VfsError::IoError)
    }
    
    fn write(&self, bkm: u64, _buf: &[u8]) -> E<usize> {
        Err(VfsError::ReadOnly)
    }
    
    fn stat(&self) -> E<Stat> {
        let inner = self.make_inner();
        let inode = inner.read_inode(self.ino).map_err(|_| VfsError::IoError)?;
        let file_size = inner.inode_size(&inode);
        let qk = inner.inode_file_type(&inode);
        Ok(Stat {
            ino: self.ino as u64,
            file_type: qk,
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


struct Ry {
    ino: u32,
    device: Arc<dyn Ak>,
    block_size: u32,
    inode_size: u16,
    inodes_per_group: u32,
    blocks_per_group: u32,
    first_data_block: u32,
    group_desc_size: u32,
}

impl Ry {
    fn make_inner(&self) -> Hv {
        Hv {
            block_size: self.block_size,
            inode_size: self.inode_size,
            inodes_per_group: self.inodes_per_group,
            blocks_per_group: self.blocks_per_group,
            first_data_block: self.first_data_block,
            group_desc_size: self.group_desc_size,
            ece: 0,
            device: self.device.clone(),
        }
    }
}

impl Bv for Ry {
    fn lookup(&self, name: &str) -> E<K> {
        let inner = self.make_inner();
        inner.dir_lookup(self.ino, name)
            .map(|ino| ino as u64)
            .map_err(|_| VfsError::NotFound)
    }
    
    fn readdir(&self) -> E<Vec<Ap>> {
        let inner = self.make_inner();
        let entries = inner.read_dir_entries(self.ino)
            .map_err(|_| VfsError::IoError)?;
        
        Ok(entries.into_iter()
            .filter(|(_, name, _)| name != "." && name != "..")
            .map(|(ino, name, qk)| Ap {
                name,
                ino: ino as u64,
                file_type: fsh(qk),
            })
            .collect())
    }
    
    fn create(&self, _name: &str, _file_type: FileType) -> E<K> {
        Err(VfsError::ReadOnly)
    }
    
    fn unlink(&self, _name: &str) -> E<()> {
        Err(VfsError::ReadOnly)
    }
    
    fn stat(&self) -> E<Stat> {
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






pub fn abd(device: Arc<dyn Ak>) -> Result<Arc<Hu>, &'static str> {
    
    let mut jcy = [0u8; 256]; 
    let sector_size = device.sector_size();
    
    
    let start_sector = YV_ / sector_size as u64;
    let mut dm = vec![0u8; sector_size * 4]; 
    for i in 0..4u64 {
        let _ = device.read_sector(start_sector + i, &mut dm[i as usize * sector_size..(i as usize + 1) * sector_size]);
    }
    
    let cpy = (YV_ - start_sector * sector_size as u64) as usize;
    if cpy + 256 > dm.len() {
        return Err("Superblock read overflow");
    }
    jcy.copy_from_slice(&dm[cpy..cpy + 256]);
    
    let cv = unsafe { core::ptr::read_unaligned(jcy.as_ptr() as *const Ajk) };
    
    
    if cv.s_magic != ATP_ {
        return Err("Not an ext4 filesystem (bad magic)");
    }
    
    let block_size = 1024u32 << cv.s_log_block_size;
    let inode_size = if cv.s_rev_level >= 1 { cv.s_inode_size } else { 128 };
    
    
    let arf = (cv.s_feature_incompat & 0x80) != 0; 
    let group_desc_size = if arf { 64u32 } else { 32 };
    
    let ece = (cv.s_blocks_count_lo + cv.s_blocks_per_group - 1) / cv.s_blocks_per_group;
    
    let ptc = core::str::from_utf8(&cv.s_volume_name)
        .unwrap_or("")
        .trim_end_matches('\0');
    
    crate::serial_println!("[ext4] Mounted: \"{}\" block_size={} inode_size={} groups={} 64bit={}",
        ptc, block_size, inode_size, ece, arf);
    crate::serial_println!("[ext4] {} inodes, {} blocks per group",
        cv.s_inodes_per_group, cv.s_blocks_per_group);
    
    let fs = Arc::new(Hu {
        inner: Mutex::new(Hv {
            block_size,
            inode_size,
            inodes_per_group: cv.s_inodes_per_group,
            blocks_per_group: cv.s_blocks_per_group,
            first_data_block: cv.s_first_data_block,
            group_desc_size,
            ece,
            device,
        }),
    });
    
    Ok(fs)
}


pub fn probe(device: &dyn Ak) -> bool {
    let sector_size = device.sector_size();
    let start_sector = YV_ / sector_size as u64;
    
    let mut dm = vec![0u8; sector_size * 4];
    for i in 0..4u64 {
        if device.read_sector(start_sector + i, &mut dm[i as usize * sector_size..(i as usize + 1) * sector_size]).is_err() {
            return false;
        }
    }
    
    let cpy = (YV_ - start_sector * sector_size as u64) as usize;
    if cpy + 2 + 0x38 > dm.len() { return false; }
    
    let magic = u16::from_le_bytes([dm[cpy + 0x38], dm[cpy + 0x39]]);
    magic == ATP_
}


pub fn read_file(fs: &Hu, path: &str) -> Result<Vec<u8>, &'static str> {
    let inner = fs.inner.lock();
    let ino = inner.resolve_path(path).map_err(|_| "File not found")?;
    let inode = inner.read_inode(ino).map_err(|_| "Failed to read inode")?;
    
    let qk = inner.inode_file_type(&inode);
    if qk == FileType::Directory {
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


pub fn ikp(fs: &Hu, path: &str) -> Result<Vec<(String, FileType, u64)>, &'static str> {
    let inner = fs.inner.lock();
    let ino = inner.resolve_path(path).map_err(|_| "Directory not found")?;
    let inode = inner.read_inode(ino).map_err(|_| "Failed to read inode")?;
    
    let qk = inner.inode_file_type(&inode);
    if qk != FileType::Directory {
        return Err("Not a directory");
    }
    
    let entries = inner.read_dir_entries(ino).map_err(|_| "Read error")?;
    
    let mut result = Vec::new();
    for (entry_ino, name, ft_byte) in entries {
        if name == "." || name == ".." { continue; }
        
        let file_type = fsh(ft_byte);
        let size = if let Ok(elm) = inner.read_inode(entry_ino) {
            inner.inode_size(&elm)
        } else {
            0
        };
        
        result.push((name, file_type, size));
    }
    
    Ok(result)
}
