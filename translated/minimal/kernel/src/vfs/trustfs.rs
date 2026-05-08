










use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::sync::Arc;
use alloc::collections::BTreeMap;
use spin::RwLock;
use core::sync::atomic::{AtomicU64, Ordering};

use super::{
    Au, Bx, Bv, Stat, Ap, FileType,
    K, E, VfsError
};
use super::fat32::Ak;

const H_: usize = 512;
const Dp: u32 = 0x54525553; 
const Nt: u32 = 1;

const AKH_: u64 = 0;
const VN_: u64 = 1;
const CFD_: u64 = 16;
const MT_: u64 = 17;
const AAX_: u64 = 16;

const BD_: u64 = 97;

const AGU_: usize = 256;
const VM_: usize = H_ / core::mem::size_of::<DiskInode>();

const LI_: usize = 28;
const BZ_: usize = 12;
const BH_: usize = H_ / 4; 

const BBQ_: usize = BZ_ + BH_ + BH_ * BH_;


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
    fn new(zp: u64) -> Self {
        Self {
            magic: Dp,
            version: Nt,
            total_blocks: (zp - BD_) as u32,
            free_blocks: (zp - BD_ - 1) as u32, 
            total_inodes: AGU_ as u32,
            free_inodes: (AGU_ - 1) as u32, 
            block_size: H_ as u32,
            root_inode: 1,
            reserved: [0; 8],
        }
    }
    
    fn is_valid(&self) -> bool {
        self.magic == Dp && self.version == Nt
    }
}


#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct DiskInode {
    mode: u16,          
    nlink: u16,         
    size: u32,          
    blocks: u32,        
    atime: u32,         
    mtime: u32,         
    direct: [u32; BZ_], 
    indirect: u32,      
    double_indirect: u32, 
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
            direct: [0; BZ_],
            indirect: 0,
            double_indirect: 0,
        }
    }
}

impl DiskInode {
    fn file_type(&self) -> FileType {
        match (self.mode >> 12) & 0xF {
            0x4 => FileType::Directory,
            0x8 => FileType::Regular,
            0x2 => FileType::CharDevice,
            0x6 => FileType::Ak,
            _ => FileType::Regular,
        }
    }
    
    fn is_dir(&self) -> bool {
        self.file_type() == FileType::Directory
    }
    
    fn set_type(&mut self, qk: FileType) {
        let poz = match qk {
            FileType::Directory => 0x4,
            FileType::Regular => 0x8,
            FileType::CharDevice => 0x2,
            FileType::Ak => 0x6,
            _ => 0x8,
        };
        self.mode = (self.mode & 0x0FFF) | (poz << 12);
    }
}


#[repr(C)]
#[derive(Clone, Copy)]
struct Fl {
    inode: u32,
    name: [u8; LI_],
}

impl Fl {
    fn name_str(&self) -> &str {
        let end = self.name.iter().position(|&b| b == 0).unwrap_or(LI_);
        core::str::from_utf8(&self.name[..end]).unwrap_or("")
    }
}


struct InodeCache {
    inodes: BTreeMap<K, DiskInode>,
}

impl InodeCache {
    fn new() -> Self {
        Self {
            inodes: BTreeMap::new(),
        }
    }
}


struct Afm {
    fs: Arc<Nn>,
    ino: K,
}

impl Bx for Afm {
    fn read(&self, offset: u64, buf: &mut [u8]) -> E<usize> {
        self.fs.read_file(self.ino, offset, buf)
    }
    
    fn write(&self, offset: u64, buf: &[u8]) -> E<usize> {
        self.fs.write_file(self.ino, offset, buf)
    }
    
    fn stat(&self) -> E<Stat> {
        self.fs.stat(self.ino)
    }
    
    fn truncate(&self, size: u64) -> E<()> {
        self.fs.truncate(self.ino, size)
    }
    
    fn sync(&self) -> E<()> {
        self.fs.sync()
    }
}


struct Afl {
    fs: Arc<Nn>,
    ino: K,
}

impl Bv for Afl {
    fn lookup(&self, name: &str) -> E<K> {
        self.fs.lookup(self.ino, name)
    }
    
    fn readdir(&self) -> E<Vec<Ap>> {
        self.fs.readdir(self.ino)
    }
    
    fn create(&self, name: &str, file_type: FileType) -> E<K> {
        self.fs.create(self.ino, name, file_type)
    }
    
    fn unlink(&self, name: &str) -> E<()> {
        self.fs.unlink(self.ino, name)
    }
    
    fn stat(&self) -> E<Stat> {
        self.fs.stat(self.ino)
    }
}


struct Nn {
    superblock: RwLock<Superblock>,
    inode_cache: RwLock<InodeCache>,
    dirty: RwLock<bool>,
    backend: Arc<dyn Ak>,
}

impl Nn {
    
    fn read_sector(&self, dj: u64, buf: &mut [u8; H_]) -> E<()> {
        if super::block_cache::kgv(dj, buf).is_ok() {
            return Ok(());
        }
        self.backend.read_sector(dj, buf)
            .map_err(|_| VfsError::IoError)
    }
    
    
    fn write_sector(&self, dj: u64, buf: &[u8; H_]) -> E<()> {
        if super::block_cache::kgw(dj, buf).is_ok() {
            return Ok(());
        }
        self.backend.write_sector(dj, buf)
            .map_err(|_| VfsError::IoError)
    }

    
    fn rde(&self, dj: u64, buf: &[u8; H_]) -> E<()> {
        
        let _ = super::wal::log_write(dj, buf);
        self.write_sector(dj, buf)
    }

    
    fn flush_wal(&self) -> E<()> {
        let backend = &self.backend;
        let write_fn = |dj: u64, data: &[u8; H_]| -> Result<(), ()> {
            backend.write_sector(dj, data)
        };
        super::wal::commit(&write_fn).map_err(|_| VfsError::IoError)
    }
    
    
    fn read_inode(&self, ino: K) -> E<DiskInode> {
        
        {
            let adk = self.inode_cache.read();
            if let Some(inode) = adk.inodes.get(&ino) {
                return Ok(*inode);
            }
        }
        
        
        let dj = VN_ + (ino as u64 / VM_ as u64);
        let offset = (ino as usize % VM_) * core::mem::size_of::<DiskInode>();
        
        let mut buf = [0u8; H_];
        self.read_sector(dj, &mut buf)?;
        
        let clc = buf[offset..].as_ptr() as *const DiskInode;
        let inode = unsafe { *clc };
        
        
        {
            let mut adk = self.inode_cache.write();
            adk.inodes.insert(ino, inode);
        }
        
        Ok(inode)
    }
    
    
    fn write_inode(&self, ino: K, inode: &DiskInode) -> E<()> {
        let dj = VN_ + (ino as u64 / VM_ as u64);
        let offset = (ino as usize % VM_) * core::mem::size_of::<DiskInode>();
        
        
        let mut buf = [0u8; H_];
        self.read_sector(dj, &mut buf)?;
        
        let clc = buf[offset..].as_mut_ptr() as *mut DiskInode;
        unsafe { *clc = *inode; }
        
        self.write_sector(dj, &buf)?;
        
        
        {
            let mut adk = self.inode_cache.write();
            adk.inodes.insert(ino, *inode);
        }
        
        *self.dirty.write() = true;
        Ok(())
    }
    
    
    fn alloc_inode(&self) -> E<K> {
        let mut cv = self.superblock.write();
        if cv.free_inodes == 0 {
            return Err(VfsError::NoSpace);
        }
        
        
        for ino in 1..AGU_ as u64 {
            let inode = self.read_inode(ino)?;
            if inode.nlink == 0 && inode.mode == 0 {
                cv.free_inodes -= 1;
                return Ok(ino);
            }
        }
        
        Err(VfsError::NoSpace)
    }
    
    
    fn alloc_block(&self) -> E<u32> {
        let mut cv = self.superblock.write();
        if cv.free_blocks == 0 {
            return Err(VfsError::NoSpace);
        }
        
        
        for cgh in 0..AAX_ {
            let mut buf = [0u8; H_];
            self.read_sector(MT_ + cgh, &mut buf)?;
            
            for yk in 0..H_ {
                if buf[yk] != 0xFF {
                    
                    for bf in 0..8 {
                        if (buf[yk] & (1 << bf)) == 0 {
                            
                            buf[yk] |= 1 << bf;
                            self.write_sector(MT_ + cgh, &buf)?;
                            
                            cv.free_blocks -= 1;
                            let block = (cgh as u32 * H_ as u32 * 8)
                                + (yk as u32 * 8)
                                + bf as u32;
                            return Ok(block);
                        }
                    }
                }
            }
        }
        
        Err(VfsError::NoSpace)
    }
    
    
    fn free_block(&self, block: u32) -> E<()> {
        let mut cv = self.superblock.write();
        
        let cgh = block as u64 / (H_ as u64 * 8);
        let yk = (block as usize / 8) % H_;
        let bf = block as usize % 8;
        
        if cgh >= AAX_ {
            return Err(VfsError::InvalidData);
        }
        
        let mut buf = [0u8; H_];
        self.read_sector(MT_ + cgh, &mut buf)?;
        
        
        buf[yk] &= !(1 << bf);
        self.write_sector(MT_ + cgh, &buf)?;
        
        cv.free_blocks += 1;
        Ok(())
    }
    
    
    fn free_inode_blocks(&self, inode: &DiskInode) -> E<()> {
        
        for i in 0..BZ_ {
            if inode.direct[i] != 0 {
                self.free_block(inode.direct[i])?;
            }
        }
        
        
        if inode.indirect != 0 {
            let mut axp = [0u8; H_];
            self.read_sector(BD_ + inode.indirect as u64, &mut axp)?;
            let bvb = unsafe { &*(axp.as_ptr() as *const [u32; BH_]) };
            
            for &ptr in bvb.iter() {
                if ptr != 0 {
                    self.free_block(ptr)?;
                }
            }
            
            
            self.free_block(inode.indirect)?;
        }
        
        
        if inode.double_indirect != 0 {
            let mut btt = [0u8; H_];
            self.read_sector(BD_ + inode.double_indirect as u64, &mut btt)?;
            let cbg = unsafe { &*(btt.as_ptr() as *const [u32; BH_]) };
            
            for &btu in cbg.iter() {
                if btu != 0 {
                    let mut btv = [0u8; H_];
                    self.read_sector(BD_ + btu as u64, &mut btv)?;
                    let dtc = unsafe { &*(btv.as_ptr() as *const [u32; BH_]) };
                    
                    for &data_block in dtc.iter() {
                        if data_block != 0 {
                            self.free_block(data_block)?;
                        }
                    }
                    
                    
                    self.free_block(btu)?;
                }
            }
            
            
            self.free_block(inode.double_indirect)?;
        }
        
        Ok(())
    }
    
    
    fn resolve_block(&self, inode: &DiskInode, abt: usize) -> E<u32> {
        if abt < BZ_ {
            Ok(inode.direct[abt])
        } else if abt < BZ_ + BH_ {
            if inode.indirect == 0 { return Ok(0); }
            let mut axp = [0u8; H_];
            self.read_sector(BD_ + inode.indirect as u64, &mut axp)?;
            let bvb = unsafe { &*(axp.as_ptr() as *const [u32; BH_]) };
            Ok(bvb[abt - BZ_])
        } else if abt < BBQ_ {
            
            if inode.double_indirect == 0 { return Ok(0); }
            let dnb = abt - BZ_ - BH_;
            let axu = dnb / BH_;
            let bnf = dnb % BH_;
            
            let mut btt = [0u8; H_];
            self.read_sector(BD_ + inode.double_indirect as u64, &mut btt)?;
            let cbg = unsafe { &*(btt.as_ptr() as *const [u32; BH_]) };
            let btu = cbg[axu];
            if btu == 0 { return Ok(0); }
            
            let mut btv = [0u8; H_];
            self.read_sector(BD_ + btu as u64, &mut btv)?;
            let dtc = unsafe { &*(btv.as_ptr() as *const [u32; BH_]) };
            Ok(dtc[bnf])
        } else {
            Err(VfsError::NoSpace) 
        }
    }

    
    fn write_indirect_ptr(&self, inode: &mut DiskInode, idx: usize, block_num: u32) -> E<()> {
        if inode.indirect == 0 {
            inode.indirect = self.alloc_block()?;
            let zero = [0u8; H_];
            self.write_sector(BD_ + inode.indirect as u64, &zero)?;
        }
        let mut axp = [0u8; H_];
        self.read_sector(BD_ + inode.indirect as u64, &mut axp)?;
        let bvb = unsafe { &mut *(axp.as_mut_ptr() as *mut [u32; BH_]) };
        bvb[idx] = block_num;
        self.write_sector(BD_ + inode.indirect as u64, &axp)
    }

    
    fn write_double_indirect_ptr(&self, inode: &mut DiskInode, dnb: usize, block_num: u32) -> E<()> {
        let zero = [0u8; H_];
        
        if inode.double_indirect == 0 {
            inode.double_indirect = self.alloc_block()?;
            self.write_sector(BD_ + inode.double_indirect as u64, &zero)?;
        }
        let axu = dnb / BH_;
        let bnf = dnb % BH_;
        
        let mut btt = [0u8; H_];
        self.read_sector(BD_ + inode.double_indirect as u64, &mut btt)?;
        let cbg = unsafe { &mut *(btt.as_mut_ptr() as *mut [u32; BH_]) };
        
        if cbg[axu] == 0 {
            cbg[axu] = self.alloc_block()?;
            self.write_sector(BD_ + inode.double_indirect as u64, &btt)?;
            self.write_sector(BD_ + cbg[axu] as u64, &zero)?;
        }
        let btu = cbg[axu];
        
        let mut btv = [0u8; H_];
        self.read_sector(BD_ + btu as u64, &mut btv)?;
        let dtc = unsafe { &mut *(btv.as_mut_ptr() as *mut [u32; BH_]) };
        dtc[bnf] = block_num;
        self.write_sector(BD_ + btu as u64, &btv)
    }

    
    fn read_file(&self, ino: K, offset: u64, buf: &mut [u8]) -> E<usize> {
        let inode = self.read_inode(ino)?;
        
        if offset >= inode.size as u64 {
            return Ok(0); 
        }
        
        let rz = core::cmp::min(buf.len(), (inode.size as u64 - offset) as usize);
        let mut atf = 0;
        let mut aaw = offset as usize;
        
        while atf < rz {
            let abt = aaw / H_;
            let bxu = aaw % H_;
            
            let aum = self.resolve_block(&inode, abt)?;
            if aum == 0 { break; }
            
            let mut mx = [0u8; H_];
            self.read_sector(BD_ + aum as u64, &mut mx)?;
            
            let rs = core::cmp::min(H_ - bxu, rz - atf);
            buf[atf..atf + rs]
                .copy_from_slice(&mx[bxu..bxu + rs]);
            
            atf += rs;
            aaw += rs;
        }
        
        Ok(atf)
    }
    
    
    fn write_file(&self, ino: K, offset: u64, buf: &[u8]) -> E<usize> {
        let mut inode = self.read_inode(ino)?;
        
        let mut atg = 0;
        let mut aaw = offset as usize;
        let ncr = BBQ_;
        
        while atg < buf.len() {
            let abt = aaw / H_;
            let bxu = aaw % H_;
            
            if abt >= ncr { break; }
            
            
            let aum = self.resolve_block(&inode, abt)?;
            let aum = if aum == 0 {
                let euv = self.alloc_block()?;
                inode.blocks += 1;
                if abt < BZ_ {
                    inode.direct[abt] = euv;
                } else if abt < BZ_ + BH_ {
                    self.write_indirect_ptr(&mut inode, abt - BZ_, euv)?;
                } else {
                    self.write_double_indirect_ptr(&mut inode, abt - BZ_ - BH_, euv)?;
                }
                euv
            } else {
                aum
            };
            
            let dj = BD_ + aum as u64;
            let rs = core::cmp::min(H_ - bxu, buf.len() - atg);
            
            
            let mut mx = [0u8; H_];
            if bxu > 0 || rs < H_ {
                self.read_sector(dj, &mut mx)?;
            }
            
            mx[bxu..bxu + rs]
                .copy_from_slice(&buf[atg..atg + rs]);
            
            self.write_sector(dj, &mx)?;
            
            atg += rs;
            aaw += rs;
        }
        
        
        let akf = core::cmp::max(inode.size, (offset + atg as u64) as u32);
        if akf != inode.size {
            inode.size = akf;
            inode.mtime = (crate::logger::eg() / 100) as u32;
            self.write_inode(ino, &inode)?;
        }
        
        Ok(atg)
    }
    
    
    fn lookup(&self, dir_ino: K, name: &str) -> E<K> {
        let entries = self.readdir(dir_ino)?;
        for entry in entries {
            if entry.name == name {
                return Ok(entry.ino);
            }
        }
        Err(VfsError::NotFound)
    }
    
    
    fn readdir(&self, dir_ino: K) -> E<Vec<Ap>> {
        let inode = self.read_inode(dir_ino)?;
        if !inode.is_dir() {
            return Err(VfsError::NotDirectory);
        }
        
        let mut entries = Vec::new();
        let oi = core::mem::size_of::<Fl>();
        let dvn = inode.size as usize / oi;
        
        for i in 0..dvn {
            let offset = (i * oi) as u64;
            let mut buf = [0u8; 32]; 
            self.read_file(dir_ino, offset, &mut buf)?;
            
            let entry_ptr = buf.as_ptr() as *const Fl;
            let ekg = unsafe { &*entry_ptr };
            
            if ekg.inode != 0 {
                let kkd = self.read_inode(ekg.inode as K)?;
                entries.push(Ap {
                    name: String::from(ekg.name_str()),
                    ino: ekg.inode as K,
                    file_type: kkd.file_type(),
                });
            }
        }
        
        Ok(entries)
    }
    
    
    fn create(&self, parent_ino: K, name: &str, file_type: FileType) -> E<K> {
        if name.len() > LI_ {
            return Err(VfsError::InvalidPath);
        }
        
        
        if self.lookup(parent_ino, name).is_ok() {
            return Err(VfsError::AlreadyExists);
        }
        
        
        let gja = self.alloc_inode()?;
        let mut dvc = DiskInode::default();
        dvc.set_type(file_type);
        dvc.nlink = 1;
        dvc.mode |= 0o644; 
        
        if file_type == FileType::Directory {
            dvc.mode |= 0o111; 
        }
        
        self.write_inode(gja, &dvc)?;
        
        
        let mut entry = Fl {
            inode: gja as u32,
            name: [0; LI_],
        };
        let agt = name.as_bytes();
        let mb = core::cmp::min(agt.len(), LI_);
        entry.name[..mb].copy_from_slice(&agt[..mb]);
        
        let npx = self.read_inode(parent_ino)?;
        let offset = npx.size as u64;
        
        let dos = unsafe {
            core::slice::from_raw_parts(
                &entry as *const Fl as *const u8,
                core::mem::size_of::<Fl>()
            )
        };
        
        self.write_file(parent_ino, offset, dos)?;
        
        Ok(gja)
    }
    
    
    fn unlink(&self, parent_ino: K, name: &str) -> E<()> {
        let entries = self.readdir(parent_ino)?;
        let oi = core::mem::size_of::<Fl>();
        
        for (i, entry) in entries.iter().enumerate() {
            if entry.name == name {
                
                let mut inode = self.read_inode(entry.ino)?;
                
                
                if inode.is_dir() && inode.size > 0 {
                    let children = self.readdir(entry.ino)?;
                    if !children.is_empty() {
                        return Err(VfsError::NotEmpty);
                    }
                }
                
                
                inode.nlink = inode.nlink.saturating_sub(1);
                
                if inode.nlink == 0 {
                    
                    if let Err(e) = self.free_inode_blocks(&inode) {
                        crate::log_warn!("[TRUSTFS] Warning: failed to free blocks for inode {}: {:?}", entry.ino, e);
                    }
                    
                    inode.mode = 0;
                    inode.size = 0;
                    inode.blocks = 0;
                    inode.direct = [0; BZ_];
                    inode.indirect = 0;
                    
                    
                    {
                        let mut cv = self.superblock.write();
                        cv.free_inodes += 1;
                    }
                }
                
                self.write_inode(entry.ino, &inode)?;
                
                
                let mut pwm = Fl {
                    inode: 0,
                    name: [0; LI_],
                };
                let offset = (i * oi) as u64;
                let dos = unsafe {
                    core::slice::from_raw_parts(
                        &pwm as *const Fl as *const u8,
                        oi
                    )
                };
                self.write_file(parent_ino, offset, dos)?;
                
                return Ok(());
            }
        }
        
        Err(VfsError::NotFound)
    }
    
    
    fn truncate(&self, ino: K, size: u64) -> E<()> {
        let mut inode = self.read_inode(ino)?;
        let evr = inode.size as u64;
        let akf = size;
        
        if akf < evr {
            
            let nmn = ((evr + H_ as u64 - 1) / H_ as u64) as usize;
            let ipp = ((akf + H_ as u64 - 1) / H_ as u64) as usize;
            
            for abt in ipp..nmn {
                if abt < BZ_ {
                    if inode.direct[abt] != 0 {
                        let _ = self.free_block(inode.direct[abt]);
                        inode.direct[abt] = 0;
                        inode.blocks = inode.blocks.saturating_sub(1);
                    }
                } else if abt < BZ_ + BH_ {
                    if inode.indirect != 0 {
                        let mut axp = [0u8; H_];
                        self.read_sector(BD_ + inode.indirect as u64, &mut axp)?;
                        let bvb = unsafe { &mut *(axp.as_mut_ptr() as *mut [u32; BH_]) };
                        let idx = abt - BZ_;
                        if bvb[idx] != 0 {
                            let _ = self.free_block(bvb[idx]);
                            bvb[idx] = 0;
                            inode.blocks = inode.blocks.saturating_sub(1);
                            self.write_sector(BD_ + inode.indirect as u64, &axp)?;
                        }
                    }
                }
            }
            
            
            if ipp <= BZ_ && inode.indirect != 0 {
                let _ = self.free_block(inode.indirect);
                inode.indirect = 0;
            }
        }
        
        inode.size = akf as u32;
        inode.mtime = (crate::logger::eg() / 100) as u32;
        self.write_inode(ino, &inode)
    }
    
    
    fn stat(&self, ino: K) -> E<Stat> {
        let inode = self.read_inode(ino)?;
        Ok(Stat {
            ino,
            file_type: inode.file_type(),
            size: inode.size as u64,
            blocks: inode.blocks as u64,
            block_size: H_ as u32,
            mode: inode.mode as u32,
            uid: 0,
            gid: 0,
            atime: inode.atime as u64,
            mtime: inode.mtime as u64,
            ctime: 0,
        })
    }
    
    
    fn sync(&self) -> E<()> {
        
        self.flush_wal()?;
        
        let _ = super::block_cache::sync();
        
        let cv = self.superblock.read();
        let mut buf = [0u8; H_];
        let dyi = buf.as_mut_ptr() as *mut Superblock;
        unsafe { *dyi = *cv; }
        self.write_sector(AKH_, &buf)?;
        
        *self.dirty.write() = false;
        crate::log_debug!("[TrustFS] sync complete");
        Ok(())
    }
}


pub struct TrustFs {
    inner: Arc<Nn>,
}

impl TrustFs {
    
    pub fn new(backend: Arc<dyn Ak>, capacity: u64) -> E<Self> {
        
        let mut buf = [0u8; H_];
        backend.read_sector(AKH_, &mut buf)
            .map_err(|_| VfsError::IoError)?;
        
        let dyi = buf.as_ptr() as *const Superblock;
        let hxa = unsafe { *dyi };
        
        let superblock = if hxa.is_valid() {
            crate::log_debug!("[TrustFS] Found existing filesystem");
            hxa
        } else {
            crate::log!("[TrustFS] Formatting new filesystem...");
            Self::lxw(&*backend, capacity)?
        };
        
        
        let hgl = &*backend;
        let ofy = |dj: u64, buf: &mut [u8; H_]| -> Result<(), ()> {
            hgl.read_sector(dj, buf).map_err(|_| ())
        };
        let ofz = |dj: u64, data: &[u8; H_]| -> Result<(), ()> {
            hgl.write_sector(dj, data).map_err(|_| ())
        };
        match super::wal::ofx(&ofy, &ofz) {
            Ok(0) => {},
            Ok(ae) => crate::log!("[TrustFS] WAL replay: {} writes recovered", ae),
            Err(_) => crate::log_warn!("[TrustFS] WAL replay failed"),
        }

        let inner = Arc::new(Nn {
            superblock: RwLock::new(superblock),
            inode_cache: RwLock::new(InodeCache::new()),
            dirty: RwLock::new(false),
            backend,
        });
        
        Ok(Self { inner })
    }
    
    
    fn lxw(backend: &dyn Ak, capacity: u64) -> E<Superblock> {
        let cv = Superblock::new(capacity);
        
        
        let mut buf = [0u8; H_];
        let dyi = buf.as_mut_ptr() as *mut Superblock;
        unsafe { *dyi = cv; }
        backend.write_sector(AKH_, &buf)
            .map_err(|_| VfsError::IoError)?;
        
        
        let jsi = [0u8; H_];
        for i in 0..CFD_ {
            backend.write_sector(VN_ + i, &jsi)
                .map_err(|_| VfsError::IoError)?;
        }
        
        
        for i in 0..AAX_ {
            backend.write_sector(MT_ + i, &jsi)
                .map_err(|_| VfsError::IoError)?;
        }
        
        
        let mut root_inode = DiskInode::default();
        root_inode.set_type(FileType::Directory);
        root_inode.nlink = 1;
        root_inode.mode |= 0o755;
        
        let mqh = VN_;
        let mut gcw = [0u8; H_];
        let clc = gcw[core::mem::size_of::<DiskInode>()..].as_mut_ptr() as *mut DiskInode;
        unsafe { *clc = root_inode; }
        
        let ohy = core::mem::size_of::<DiskInode>(); 
        let clc = gcw[ohy..].as_mut_ptr() as *mut DiskInode;
        unsafe { *clc = root_inode; }
        backend.write_sector(mqh, &gcw)
            .map_err(|_| VfsError::IoError)?;
        
        crate::log!("[TrustFS] Formatted: {} blocks, {} inodes", cv.total_blocks, cv.total_inodes);
        
        Ok(cv)
    }
}

impl Au for TrustFs {
    fn name(&self) -> &str {
        "trustfs"
    }
    
    fn root_inode(&self) -> K {
        1
    }
    
    fn get_file(&self, ino: K) -> E<Arc<dyn Bx>> {
        let inode = self.inner.read_inode(ino)?;
        if inode.is_dir() {
            return Err(VfsError::IsDirectory);
        }
        Ok(Arc::new(Afm {
            fs: Arc::clone(&self.inner),
            ino,
        }))
    }
    
    fn get_dir(&self, ino: K) -> E<Arc<dyn Bv>> {
        let inode = self.inner.read_inode(ino)?;
        if !inode.is_dir() {
            return Err(VfsError::NotDirectory);
        }
        Ok(Arc::new(Afl {
            fs: Arc::clone(&self.inner),
            ino,
        }))
    }
    
    fn stat(&self, ino: K) -> E<Stat> {
        self.inner.stat(ino)
    }
    
    fn sync(&self) -> E<()> {
        self.inner.sync()
    }
}
