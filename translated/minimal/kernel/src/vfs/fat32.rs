










use alloc::string::{String, ToString};
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

const H_: usize = 512;


#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Sf {
    jmp_boot: [u8; 3],
    oem_name: [u8; 8],
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sectors: u16,
    num_fats: u8,
    root_entry_count: u16,  
    total_sectors_16: u16,  
    media_type: u8,
    fat_size_16: u16,       
    sectors_per_track: u16,
    num_heads: u16,
    hidden_sectors: u32,
    total_sectors_32: u32,
    
    fat_size_32: u32,
    ext_flags: u16,
    fs_version: u16,
    root_cluster: u32,
    fs_info: u16,
    backup_boot: u16,
    reserved: [u8; 12],
    drive_number: u8,
    reserved1: u8,
    boot_sig: u8,
    volume_id: u32,
    volume_label: [u8; 11],
    fs_type: [u8; 8],
}

impl Sf {
    fn is_valid(&self) -> bool {
        
        let bytes_per_sector = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(self.bytes_per_sector)) };
        let sectors_per_cluster = self.sectors_per_cluster;
        let reserved = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(self.reserved_sectors)) };
        let num_fats = self.num_fats;
        
        bytes_per_sector >= 512 && 
        bytes_per_sector <= 4096 &&
        sectors_per_cluster >= 1 &&
        sectors_per_cluster <= 128 &&
        reserved >= 1 &&
        num_fats >= 1
    }
    
    fn cluster_size(&self) -> usize {
        let djm = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(self.bytes_per_sector)) } as usize;
        let dzv = self.sectors_per_cluster as usize;
        djm * dzv
    }
    
    fn first_data_sector(&self) -> u64 {
        let reserved = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(self.reserved_sectors)) } as u64;
        let num_fats = self.num_fats as u64;
        let dpi = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(self.fat_size_32)) } as u64;
        reserved + (num_fats * dpi)
    }
    
    fn first_fat_sector(&self) -> u64 {
        (unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(self.reserved_sectors)) }) as u64
    }
    
    fn cluster_to_sector(&self, cluster: u32) -> u64 {
        let dzv = self.sectors_per_cluster as u64;
        self.first_data_sector() + ((cluster - 2) as u64 * dzv)
    }
}


#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Fat32DirEntry {
    name: [u8; 8],
    ext: [u8; 3],
    attr: u8,
    nt_reserved: u8,
    create_time_tenth: u8,
    create_time: u16,
    create_date: u16,
    access_date: u16,
    cluster_hi: u16,
    modify_time: u16,
    modify_date: u16,
    cluster_lo: u16,
    file_size: u32,
}

impl Fat32DirEntry {
    const DGO_: u8 = 0x01;
    const DGN_: u8 = 0x02;
    const DGP_: u8 = 0x04;
    const BNF_: u8 = 0x08;
    const MR_: u8 = 0x10;
    const BNA_: u8 = 0x20;
    const AMT_: u8 = 0x0F;
    
    fn is_free(&self) -> bool {
        self.name[0] == 0x00 || self.name[0] == 0xE5
    }
    
    fn is_end(&self) -> bool {
        self.name[0] == 0x00
    }
    
    fn is_long_name(&self) -> bool {
        (self.attr & Self::AMT_) == Self::AMT_
    }
    
    fn is_directory(&self) -> bool {
        (self.attr & Self::MR_) != 0
    }
    
    fn is_volume_label(&self) -> bool {
        (self.attr & Self::BNF_) != 0 && !self.is_long_name()
    }
    
    fn cluster(&self) -> u32 {
        let hi = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(self.cluster_hi)) } as u32;
        let lo = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(self.cluster_lo)) } as u32;
        (hi << 16) | lo
    }
    
    fn size(&self) -> u32 {
        unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(self.file_size)) }
    }
    
    fn get_short_name(&self) -> String {
        let agt = self.name;
        let lte = self.ext;
        
        
        let name: String = agt.iter()
            .take_while(|&&c| c != b' ' && c != 0)
            .map(|&c| {
                if c == 0x05 { 0xE5 as char } 
                else { c as char }
            })
            .collect();
        
        let ext: String = lte.iter()
            .take_while(|&&c| c != b' ' && c != 0)
            .map(|&c| c as char)
            .collect();
        
        if ext.is_empty() {
            name
        } else {
            alloc::format!("{}.{}", name, ext)
        }
    }
}


#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Yt {
    order: u8,
    name1: [u16; 5],  
    attr: u8,         
    lfn_type: u8,     
    checksum: u8,
    name2: [u16; 6],  
    cluster: u16,     
    name3: [u16; 2],  
}

impl Yt {
    fn get_chars(&self) -> Vec<char> {
        let mut chars = Vec::with_capacity(13);
        
        
        let name1 = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(self.name1)) };
        let name2 = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(self.name2)) };
        let name3 = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(self.name3)) };
        
        for &c in &name1 {
            if c == 0 || c == 0xFFFF { return chars; }
            chars.push(char::from_u32(c as u32).unwrap_or('?'));
        }
        for &c in &name2 {
            if c == 0 || c == 0xFFFF { return chars; }
            chars.push(char::from_u32(c as u32).unwrap_or('?'));
        }
        for &c in &name3 {
            if c == 0 || c == 0xFFFF { return chars; }
            chars.push(char::from_u32(c as u32).unwrap_or('?'));
        }
        
        chars
    }
    
    fn order(&self) -> u8 {
        self.order & 0x3F
    }
    
    fn clo(&self) -> bool {
        (self.order & 0x40) != 0
    }
}


const ATS_: u32 = 0x00000000;
const BXX_: u32 = 0x0FFFFFF8;  
const BXW_: u32 = 0x0FFFFFF7;


struct Hx {
    cluster: u32,
    size: u64,
    is_dir: bool,
    name: String,
    
    dir_cluster: u32,
    
    dir_entry_offset: usize,
}


pub trait Ak: Send + Sync {
    fn read_sector(&self, dj: u64, buffer: &mut [u8]) -> Result<(), ()>;
    fn write_sector(&self, dj: u64, buffer: &[u8]) -> Result<(), ()>;
    fn sector_size(&self) -> usize { H_ }
}


pub trait Ahl: Ak {}
impl<T: Ak> Ahl for T {}


pub struct Agd;

impl Ak for Agd {
    fn read_sector(&self, dj: u64, buffer: &mut [u8]) -> Result<(), ()> {
        
        crate::virtio_blk::read_sectors(dj, 1, buffer)
            .map_err(|_| ())
    }
    
    fn write_sector(&self, dj: u64, buffer: &[u8]) -> Result<(), ()> {
        crate::virtio_blk::write_sectors(dj, 1, buffer)
            .map_err(|_| ())
    }
}


pub struct AhciBlockReader {
    port: usize,
    partition_start: u64,  
}

impl AhciBlockReader {
    pub fn new(port: usize, partition_start: u64) -> Self {
        Self { port, partition_start }
    }
}

impl Ak for AhciBlockReader {
    fn read_sector(&self, dj: u64, buffer: &mut [u8]) -> Result<(), ()> {
        let ffy = self.partition_start + dj;
        crate::drivers::ahci::read_sectors(self.port as u8, ffy, 1, buffer)
            .map(|_| ())
            .map_err(|_| ())
    }
    
    fn write_sector(&self, dj: u64, buffer: &[u8]) -> Result<(), ()> {
        let ffy = self.partition_start + dj;
        crate::drivers::ahci::write_sectors(self.port as u8, ffy, 1, buffer)
            .map(|_| ())
            .map_err(|_| ())
    }
}


pub struct Fat32Fs {
    reader: Arc<dyn Ak>,
    bpb: Sf,
    inodes: RwLock<BTreeMap<K, Hx>>,
    next_ino: AtomicU64,
    root_ino: K,
}

impl Fat32Fs {
    
    pub fn abd(reader: Arc<dyn Ak>) -> E<Self> {
        
        let mut bap = [0u8; H_];
        reader.read_sector(0, &mut bap)
            .map_err(|_| VfsError::IoError)?;
        
        
        if bap[510] != 0x55 || bap[511] != 0xAA {
            crate::log_warn!("[FAT32] Invalid boot signature");
            return Err(VfsError::InvalidPath);
        }
        
        
        let bpb = unsafe { 
            core::ptr::read_unaligned(bap.as_ptr() as *const Sf)
        };
        
        if !bpb.is_valid() {
            crate::log_warn!("[FAT32] Invalid BPB");
            return Err(VfsError::InvalidPath);
        }
        
        let root_cluster = { bpb.root_cluster };
        let bytes_per_sector = { bpb.bytes_per_sector };
        let sectors_per_cluster = { bpb.sectors_per_cluster };
        
        crate::log!("[FAT32] Mounted: {} bytes/sector, {} sectors/cluster, root cluster {}",
            bytes_per_sector, sectors_per_cluster, root_cluster);
        
        let fs = Self {
            reader,
            bpb,
            inodes: RwLock::new(BTreeMap::new()),
            next_ino: AtomicU64::new(2),
            root_ino: 1,
        };
        
        
        {
            let mut inodes = fs.inodes.write();
            inodes.insert(1, Hx {
                cluster: root_cluster,
                size: 0,
                is_dir: true,
                name: String::from("/"),
                dir_cluster: 0,
                dir_entry_offset: 0,
            });
        }
        
        Ok(fs)
    }
    
    
    fn read_cluster(&self, cluster: u32) -> E<Vec<u8>> {
        let dj = self.bpb.cluster_to_sector(cluster);
        let sectors_per_cluster = { self.bpb.sectors_per_cluster } as u64;
        let cluster_size = self.bpb.cluster_size();
        
        let mut data = vec![0u8; cluster_size];
        
        for i in 0..sectors_per_cluster {
            let offset = (i as usize) * H_;
            self.reader.read_sector(dj + i, &mut data[offset..offset + H_])
                .map_err(|_| VfsError::IoError)?;
        }
        
        Ok(data)
    }
    
    
    fn read_fat_entry(&self, cluster: u32) -> E<u32> {
        let dpg = cluster * 4;
        let bytes_per_sector = { self.bpb.bytes_per_sector } as u32;
        let dph = self.bpb.first_fat_sector() + (dpg / bytes_per_sector) as u64;
        let afl = (dpg % bytes_per_sector) as usize;
        
        let mut mx = [0u8; H_];
        self.reader.read_sector(dph, &mut mx)
            .map_err(|_| VfsError::IoError)?;
        
        let entry = u32::from_le_bytes([
            mx[afl],
            mx[afl + 1],
            mx[afl + 2],
            mx[afl + 3],
        ]) & 0x0FFFFFFF;  
        
        Ok(entry)
    }
    
    
    fn get_cluster_chain(&self, bji: u32) -> E<Vec<u32>> {
        let mut chain = Vec::new();
        let mut current = bji;
        
        while current >= 2 && current < BXW_ {
            chain.push(current);
            current = self.read_fat_entry(current)?;
            
            
            if chain.len() > 1_000_000 {
                return Err(VfsError::IoError);
            }
        }
        
        Ok(chain)
    }
    
    
    fn read_chain(&self, bji: u32, size: Option<u64>) -> E<Vec<u8>> {
        let chain = self.get_cluster_chain(bji)?;
        let cluster_size = self.bpb.cluster_size();
        let total_size = size.unwrap_or((chain.len() * cluster_size) as u64) as usize;
        
        let mut data = Vec::with_capacity(total_size);
        
        for cluster in chain {
            let chd = self.read_cluster(cluster)?;
            data.extend_from_slice(&chd);
            if data.len() >= total_size {
                break;
            }
        }
        
        data.truncate(total_size);
        Ok(data)
    }
    
    
    fn write_cluster(&self, cluster: u32, data: &[u8]) -> E<()> {
        let dj = self.bpb.cluster_to_sector(cluster);
        let sectors_per_cluster = { self.bpb.sectors_per_cluster } as u64;
        let cluster_size = self.bpb.cluster_size();
        
        if data.len() < cluster_size {
            return Err(VfsError::IoError);
        }
        
        for i in 0..sectors_per_cluster {
            let offset = (i as usize) * H_;
            self.reader.write_sector(dj + i, &data[offset..offset + H_])
                .map_err(|_| VfsError::IoError)?;
        }
        
        Ok(())
    }
    
    
    fn write_fat_entry(&self, cluster: u32, value: u32) -> E<()> {
        let dpg = cluster * 4;
        let bytes_per_sector = { self.bpb.bytes_per_sector } as u32;
        let dph = self.bpb.first_fat_sector() + (dpg / bytes_per_sector) as u64;
        let afl = (dpg % bytes_per_sector) as usize;
        
        
        let mut mx = [0u8; H_];
        self.reader.read_sector(dph, &mut mx)
            .map_err(|_| VfsError::IoError)?;
        
        
        let gjj = (value & 0x0FFFFFFF) | 
                        (u32::from_le_bytes([mx[afl], 
                                            mx[afl + 1],
                                            mx[afl + 2], 
                                            mx[afl + 3]]) & 0xF0000000);
        
        mx[afl..afl + 4]
            .copy_from_slice(&gjj.to_le_bytes());
        
        
        self.reader.write_sector(dph, &mx)
            .map_err(|_| VfsError::IoError)?;
        
        
        let num_fats = { self.bpb.num_fats } as u64;
        if num_fats > 1 {
            let dpi = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(self.bpb.fat_size_32)) } as u64;
            for fat_idx in 1..num_fats {
                let jze = dph + fat_idx * dpi;
                let _ = self.reader.write_sector(jze, &mx);
            }
        }
        
        Ok(())
    }
    
    
    fn allocate_cluster(&self) -> E<u32> {
        let bytes_per_sector = { self.bpb.bytes_per_sector } as u32;
        let luj = self.bpb.first_fat_sector();
        let dpi = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(self.bpb.fat_size_32)) };
        let total_sectors_32 = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(self.bpb.total_sectors_32)) };
        let lbs = total_sectors_32 as u64 - self.bpb.first_data_sector();
        let dzv = self.bpb.sectors_per_cluster as u64;
        let plq = (lbs / dzv) as u32 + 2;
        
        
        let cxf = bytes_per_sector / 4;
        let mut mx = [0u8; H_];
        
        for avb in 0..dpi {
            self.reader.read_sector(luj + avb as u64, &mut mx)
                .map_err(|_| VfsError::IoError)?;
            
            for ado in 0..cxf {
                let cluster = avb * cxf + ado;
                if cluster < 2 || cluster >= plq {
                    continue;
                }
                
                let offset = (ado * 4) as usize;
                let value = u32::from_le_bytes([
                    mx[offset],
                    mx[offset + 1],
                    mx[offset + 2],
                    mx[offset + 3],
                ]) & 0x0FFFFFFF;
                
                if value == ATS_ {
                    
                    self.write_fat_entry(cluster, BXX_)?;
                    return Ok(cluster);
                }
            }
        }
        
        Err(VfsError::NoSpace)
    }
    
    
    fn extend_chain(&self, last_cluster: u32) -> E<u32> {
        let afj = self.allocate_cluster()?;
        self.write_fat_entry(last_cluster, afj)?;
        Ok(afj)
    }
    
    
    fn write_file_data(&self, bji: u32, offset: u64, data: &[u8], fpx: u64) -> E<(u32, u64)> {
        let cluster_size = self.bpb.cluster_size();
        let mut chain = self.get_cluster_chain(bji)?;
        
        
        let bzl = offset + data.len() as u64;
        let fmc = ((bzl + cluster_size as u64 - 1) / cluster_size as u64) as usize;
        
        
        while chain.len() < fmc {
            let last = *chain.last().unwrap_or(&bji);
            let afj = if chain.is_empty() {
                self.allocate_cluster()?
            } else {
                self.extend_chain(last)?
            };
            chain.push(afj);
        }
        
        
        let mut ck = data;
        let mut cfn = offset as usize;
        
        for &cluster in &chain {
            let eie = (chain.iter().position(|&c| c == cluster).unwrap()) * cluster_size;
            let klh = eie + cluster_size;
            
            if cfn >= klh || ck.is_empty() {
                continue;
            }
            
            if cfn < eie {
                cfn = eie;
            }
            
            
            let mut chd = self.read_cluster(cluster)?;
            
            
            let bik = cfn - eie;
            let ouk = cluster_size - bik;
            let bpo = core::cmp::min(ouk, ck.len());
            
            
            chd[bik..bik + bpo]
                .copy_from_slice(&ck[..bpo]);
            
            
            self.write_cluster(cluster, &chd)?;
            
            ck = &ck[bpo..];
            cfn += bpo;
        }
        
        let akf = core::cmp::max(fpx, bzl);
        let gji = *chain.first().unwrap_or(&bji);
        
        Ok((gji, akf))
    }
    
    
    fn mci(long_name: &str) -> [u8; 11] {
        let mut dzh = [b' '; 11];
        let fea = long_name.to_uppercase();
        
        
        let (name_part, ext_part) = if let Some(dot_pos) = fea.rfind('.') {
            (&fea[..dot_pos], &fea[dot_pos + 1..])
        } else {
            (fea.as_str(), "")
        };
        
        
        for (i, ch) in name_part.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '_').take(8).enumerate() {
            dzh[i] = ch as u8;
        }
        
        
        for (i, ch) in ext_part.chars().filter(|c| c.is_ascii_alphanumeric()).take(3).enumerate() {
            dzh[8 + i] = ch as u8;
        }
        
        dzh
    }
    
    
    fn find_free_dir_entry(&self, dir_cluster: u32) -> E<(u32, usize)> {
        let cluster_size = self.bpb.cluster_size();
        let oi = core::mem::size_of::<Fat32DirEntry>();
        let ell = cluster_size / oi;
        
        let chain = self.get_cluster_chain(dir_cluster)?;
        
        for &cluster in &chain {
            let data = self.read_cluster(cluster)?;
            
            for i in 0..ell {
                let offset = i * oi;
                let ems = data[offset];
                
                if ems == 0x00 || ems == 0xE5 {
                    return Ok((cluster, offset));
                }
            }
        }
        
        
        let last = *chain.last().unwrap_or(&dir_cluster);
        let afj = self.extend_chain(last)?;
        
        
        let pwl = vec![0u8; cluster_size];
        self.write_cluster(afj, &pwl)?;
        
        Ok((afj, 0))
    }
    
    
    fn create_entry(&self, dir_cluster: u32, name: &str, is_dir: bool) -> E<(u32, u64)> {
        let (entry_cluster, entry_offset) = self.find_free_dir_entry(dir_cluster)?;
        
        
        let afj = if is_dir {
            let cluster = self.allocate_cluster()?;
            
            let cluster_size = self.bpb.cluster_size();
            let mut fsg = vec![0u8; cluster_size];
            
            
            let lha = Fat32DirEntry {
                name: [b'.', b' ', b' ', b' ', b' ', b' ', b' ', b' '],
                ext: [b' ', b' ', b' '],
                attr: Fat32DirEntry::MR_,
                nt_reserved: 0,
                create_time_tenth: 0,
                create_time: 0,
                create_date: 0,
                access_date: 0,
                cluster_hi: ((cluster >> 16) & 0xFFFF) as u16,
                modify_time: 0,
                modify_date: 0,
                cluster_lo: (cluster & 0xFFFF) as u16,
                file_size: 0,
            };
            
            
            let lhd = Fat32DirEntry {
                name: [b'.', b'.', b' ', b' ', b' ', b' ', b' ', b' '],
                ext: [b' ', b' ', b' '],
                attr: Fat32DirEntry::MR_,
                nt_reserved: 0,
                create_time_tenth: 0,
                create_time: 0,
                create_date: 0,
                access_date: 0,
                cluster_hi: ((dir_cluster >> 16) & 0xFFFF) as u16,
                modify_time: 0,
                modify_date: 0,
                cluster_lo: (dir_cluster & 0xFFFF) as u16,
                file_size: 0,
            };
            
            let oi = core::mem::size_of::<Fat32DirEntry>();
            unsafe {
                core::ptr::write(fsg.as_mut_ptr() as *mut Fat32DirEntry, lha);
                core::ptr::write((fsg.as_mut_ptr().add(oi)) as *mut Fat32DirEntry, lhd);
            }
            
            self.write_cluster(cluster, &fsg)?;
            cluster
        } else {
            0 
        };
        
        
        let short_name = Self::mci(name);
        let new_entry = Fat32DirEntry {
            name: short_name[..8].try_into().unwrap_or([b' '; 8]),
            ext: short_name[8..11].try_into().unwrap_or([b' '; 3]),
            attr: if is_dir { Fat32DirEntry::MR_ } else { Fat32DirEntry::BNA_ },
            nt_reserved: 0,
            create_time_tenth: 0,
            create_time: 0,
            create_date: 0,
            access_date: 0,
            cluster_hi: ((afj >> 16) & 0xFFFF) as u16,
            modify_time: 0,
            modify_date: 0,
            cluster_lo: (afj & 0xFFFF) as u16,
            file_size: 0,
        };
        
        
        let mut chd = self.read_cluster(entry_cluster)?;
        unsafe {
            core::ptr::write(
                chd.as_mut_ptr().add(entry_offset) as *mut Fat32DirEntry,
                new_entry
            );
        }
        self.write_cluster(entry_cluster, &chd)?;
        
        Ok((afj, 0))
    }
    
    
    fn update_dir_entry(&self, ino: K, afj: u32, akf: u64) -> E<()> {
        let inodes = self.inodes.read();
        let inode = inodes.get(&ino).ok_or(VfsError::NotFound)?;
        let dir_cluster = inode.dir_cluster;
        let hsj = inode.dir_entry_offset;
        drop(inodes);

        if dir_cluster == 0 {
            
            return Ok(());
        }

        let oi = core::mem::size_of::<Fat32DirEntry>();
        let cluster_size = self.bpb.cluster_size();
        let ell = cluster_size / oi;

        
        let chain = self.get_cluster_chain(dir_cluster)?;
        let hlq = hsj / cluster_size;
        let bik = hsj % cluster_size;

        if hlq >= chain.len() {
            return Err(VfsError::IoError);
        }

        let jlo = chain[hlq];
        let mut data = self.read_cluster(jlo)?;

        
        let entry = unsafe {
            &mut *(data.as_mut_ptr().add(bik) as *mut Fat32DirEntry)
        };

        
        entry.cluster_lo = (afj & 0xFFFF) as u16;
        entry.cluster_hi = ((afj >> 16) & 0xFFFF) as u16;

        
        if entry.attr & Fat32DirEntry::MR_ == 0 {
            entry.file_size = akf as u32;
        }

        
        self.write_cluster(jlo, &data)?;

        Ok(())
    }
    
    
    fn delete_entry(&self, dir_cluster: u32, name: &str) -> E<()> {
        let cluster_size = self.bpb.cluster_size();
        let oi = core::mem::size_of::<Fat32DirEntry>();
        let ell = cluster_size / oi;
        
        let chain = self.get_cluster_chain(dir_cluster)?;
        
        for &cluster in &chain {
            let mut data = self.read_cluster(cluster)?;
            
            for i in 0..ell {
                let offset = i * oi;
                let entry = unsafe {
                    core::ptr::read_unaligned(data[offset..].as_ptr() as *const Fat32DirEntry)
                };
                
                if entry.is_end() {
                    return Err(VfsError::NotFound);
                }
                
                if !entry.is_free() && !entry.is_long_name() && !entry.is_volume_label() {
                    let bbl = entry.get_short_name();
                    if bbl.eq_ignore_ascii_case(name) {
                        
                        data[offset] = 0xE5;
                        self.write_cluster(cluster, &data)?;
                        
                        
                        let hyd = entry.cluster();
                        if hyd >= 2 {
                            self.free_cluster_chain(hyd)?;
                        }
                        
                        return Ok(());
                    }
                }
            }
        }
        
        Err(VfsError::NotFound)
    }
    
    
    fn free_cluster_chain(&self, bji: u32) -> E<()> {
        let chain = self.get_cluster_chain(bji)?;
        
        for cluster in chain {
            self.write_fat_entry(cluster, ATS_)?;
        }
        
        Ok(())
    }
    
    
    fn parse_directory(&self, cluster: u32) -> E<Vec<(String, Fat32DirEntry, usize)>> {
        let data = self.read_chain(cluster, None)?;
        let oi = core::mem::size_of::<Fat32DirEntry>();
        let mut entries = Vec::new();
        let mut dan: Vec<(u8, Vec<char>)> = Vec::new();
        
        let mut i = 0;
        while i + oi <= data.len() {
            let entry = unsafe {
                core::ptr::read_unaligned(data[i..].as_ptr() as *const Fat32DirEntry)
            };
            
            if entry.is_end() {
                break;
            }
            
            if entry.is_free() {
                dan.clear();
                i += oi;
                continue;
            }
            
            if entry.is_long_name() {
                
                let ika = unsafe {
                    core::ptr::read_unaligned(data[i..].as_ptr() as *const Yt)
                };
                dan.push((ika.order(), ika.get_chars()));
            } else if !entry.is_volume_label() {
                
                let name = if !dan.is_empty() {
                    
                    dan.sort_by_key(|(order, _)| *order);
                    let cyf: String = dan.iter()
                        .flat_map(|(_, chars)| chars.iter())
                        .collect();
                    dan.clear();
                    cyf
                } else {
                    entry.get_short_name()
                };
                
                
                if name != "." && name != ".." {
                    entries.push((name, entry, i));
                }
            }
            
            i += oi;
        }
        
        Ok(entries)
    }
    
    fn get_inode(&self, ino: K) -> E<Hx> {
        let inodes = self.inodes.read();
        inodes.get(&ino).cloned().ok_or(VfsError::NotFound)
    }
    
    fn alloc_ino(&self) -> K {
        self.next_ino.fetch_add(1, Ordering::SeqCst)
    }
}


impl Clone for Hx {
    fn clone(&self) -> Self {
        Self {
            cluster: self.cluster,
            size: self.size,
            is_dir: self.is_dir,
            dir_cluster: self.dir_cluster,
            dir_entry_offset: self.dir_entry_offset,
            name: self.name.clone(),
        }
    }
}

impl Au for Fat32Fs {
    fn name(&self) -> &str {
        "fat32"
    }
    
    fn root_inode(&self) -> K {
        self.root_ino
    }
    
    fn get_file(&self, ino: K) -> E<Arc<dyn Bx>> {
        let inode = self.get_inode(ino)?;
        if inode.is_dir {
            return Err(VfsError::IsDirectory);
        }
        
        Ok(Arc::new(Oy {
            fs: self as *const Fat32Fs,
            ino,
            cluster: RwLock::new(inode.cluster),
            size: RwLock::new(inode.size),
        }))
    }
    
    fn get_dir(&self, ino: K) -> E<Arc<dyn Bv>> {
        let inode = self.get_inode(ino)?;
        if !inode.is_dir {
            return Err(VfsError::NotDirectory);
        }
        
        Ok(Arc::new(Ox {
            fs: self as *const Fat32Fs,
            ino,
            cluster: inode.cluster,
        }))
    }
    
    fn stat(&self, ino: K) -> E<Stat> {
        let inode = self.get_inode(ino)?;
        
        Ok(Stat {
            ino,
            file_type: if inode.is_dir { FileType::Directory } else { FileType::Regular },
            size: inode.size,
            blocks: (inode.size + 511) / 512,
            block_size: self.bpb.cluster_size() as u32,
            mode: if inode.is_dir { 0o755 } else { 0o644 },
            uid: 0,
            gid: 0,
            atime: 0,
            mtime: 0,
            ctime: 0,
        })
    }
}


struct Oy {
    fs: *const Fat32Fs,
    ino: K,
    cluster: RwLock<u32>,
    size: RwLock<u64>,
}

unsafe impl Send for Oy {}
unsafe impl Sync for Oy {}

impl Bx for Oy {
    fn read(&self, offset: u64, buf: &mut [u8]) -> E<usize> {
        let size = *self.size.read();
        let cluster = *self.cluster.read();
        
        if offset >= size {
            return Ok(0);
        }
        
        let fs = unsafe { &*self.fs };
        let rz = core::cmp::min(buf.len() as u64, size - offset) as usize;
        
        if cluster < 2 {
            
            return Ok(0);
        }
        
        
        let data = fs.read_chain(cluster, Some(size))?;
        
        let start = offset as usize;
        let end = start + rz;
        
        if end <= data.len() {
            buf[..rz].copy_from_slice(&data[start..end]);
            Ok(rz)
        } else {
            Err(VfsError::IoError)
        }
    }
    
    fn write(&self, offset: u64, buf: &[u8]) -> E<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        
        let fs = unsafe { &*self.fs };
        let hpp = *self.cluster.read();
        let fpx = *self.size.read();
        
        
        let bji = if hpp < 2 {
            let afj = fs.allocate_cluster()?;
            *self.cluster.write() = afj;
            afj
        } else {
            hpp
        };
        
        
        let (afj, akf) = fs.write_file_data(bji, offset, buf, fpx)?;
        
        
        *self.cluster.write() = afj;
        *self.size.write() = akf;
        
        
        if let Some(mut inodes) = fs.inodes.try_write() {
            if let Some(inode) = inodes.get_mut(&self.ino) {
                inode.cluster = afj;
                inode.size = akf;
            }
        }
        
        
        let _ = fs.update_dir_entry(self.ino, afj, akf);
        
        Ok(buf.len())
    }
    
    fn stat(&self) -> E<Stat> {
        let size = *self.size.read();
        Ok(Stat {
            ino: self.ino,
            file_type: FileType::Regular,
            size,
            blocks: (size + 511) / 512,
            block_size: 512,
            mode: 0o644,
            uid: 0,
            gid: 0,
            atime: 0,
            mtime: 0,
            ctime: 0,
        })
    }
}


struct Ox {
    fs: *const Fat32Fs,
    ino: K,
    cluster: u32,
}

unsafe impl Send for Ox {}
unsafe impl Sync for Ox {}

impl Bv for Ox {
    fn lookup(&self, name: &str) -> E<K> {
        let fs = unsafe { &*self.fs };
        
        
        {
            let inodes = fs.inodes.read();
            for (ino, inode) in inodes.iter() {
                if inode.name.eq_ignore_ascii_case(name) {
                    return Ok(*ino);
                }
            }
        }
        
        
        let entries = fs.parse_directory(self.cluster)?;
        
        for (bbl, entry, uo) in entries {
            if bbl.eq_ignore_ascii_case(name) {
                let ino = fs.alloc_ino();
                let inode = Hx {
                    cluster: entry.cluster(),
                    size: entry.size() as u64,
                    is_dir: entry.is_directory(),
                    name: bbl,
                    dir_cluster: self.cluster,
                    dir_entry_offset: uo,
                };
                fs.inodes.write().insert(ino, inode);
                return Ok(ino);
            }
        }
        
        Err(VfsError::NotFound)
    }
    
    fn readdir(&self) -> E<Vec<Ap>> {
        let fs = unsafe { &*self.fs };
        let entries = fs.parse_directory(self.cluster)?;
        
        let mut result = Vec::new();
        
        for (name, entry, uo) in entries {
            let ino = fs.alloc_ino();
            let is_dir = entry.is_directory();
            
            
            fs.inodes.write().insert(ino, Hx {
                cluster: entry.cluster(),
                size: entry.size() as u64,
                is_dir,
                name: name.clone(),
                dir_cluster: self.cluster,
                dir_entry_offset: uo,
            });
            
            result.push(Ap {
                name,
                ino,
                file_type: if is_dir { FileType::Directory } else { FileType::Regular },
            });
        }
        
        Ok(result)
    }
    
    fn create(&self, name: &str, file_type: FileType) -> E<K> {
        let fs = unsafe { &*self.fs };
        
        
        if self.lookup(name).is_ok() {
            return Err(VfsError::AlreadyExists);
        }
        
        let is_dir = matches!(file_type, FileType::Directory);
        
        
        let (cluster, bek) = fs.create_entry(self.cluster, name, is_dir)?;
        
        
        let dir_entry_offset = if let Ok(entries) = fs.parse_directory(self.cluster) {
            entries.iter()
                .find(|(ae, _, _)| ae.eq_ignore_ascii_case(name))
                .map(|(_, _, off)| *off)
                .unwrap_or(0)
        } else {
            0
        };
        
        
        let ino = fs.alloc_ino();
        fs.inodes.write().insert(ino, Hx {
            cluster,
            size: 0,
            is_dir,
            name: name.to_string(),
            dir_cluster: self.cluster,
            dir_entry_offset,
        });
        
        Ok(ino)
    }
    
    fn unlink(&self, name: &str) -> E<()> {
        let fs = unsafe { &*self.fs };
        
        
        if name == "." || name == ".." {
            return Err(VfsError::InvalidPath);
        }
        
        
        fs.delete_entry(self.cluster, name)?;
        
        
        let mut inodes = fs.inodes.write();
        let aph: Vec<K> = inodes.iter()
            .filter(|(_, inode)| inode.name.eq_ignore_ascii_case(name))
            .map(|(ino, _)| *ino)
            .collect();
        
        for ino in aph {
            inodes.remove(&ino);
        }
        
        Ok(())
    }
    
    fn stat(&self) -> E<Stat> {
        let fs = unsafe { &*self.fs };
        fs.stat(self.ino)
    }
}


pub fn pnx() -> Option<Arc<Fat32Fs>> {
    use crate::drivers::partition::{dwf, PartitionType};
    use crate::drivers::ahci;
    
    
    let devices = ahci::adz();
    crate::log_debug!("[FAT32] Checking {} AHCI devices", devices.len());
    
    for device in devices {
        let port = device.port_num;
        let zp = device.sector_count;
        crate::log_debug!("[FAT32] Port {}: {} sectors", port, zp);
        
        
        let read_fn = |dj: u64, buf: &mut [u8]| -> Result<(), &'static str> {
            ahci::read_sectors(port, dj, 1, buf)
                .map(|_| ())
        };
        
        
        let odk = Arc::new(AhciBlockReader::new(port as usize, 0));
        if let Ok(fs) = Fat32Fs::abd(odk) {
            crate::log!("[FAT32] Mounted superfloppy FAT32 from port {}", port);
            return Some(Arc::new(fs));
        }
        
        
        if let Ok(bs) = dwf(read_fn, zp) {
            for partition in &bs.partitions {
                
                match partition.partition_type {
                    PartitionType::Fat32 | 
                    PartitionType::Fat32Lba |
                    PartitionType::MicrosoftBasicData => {
                        crate::log!("[FAT32] Found partition at LBA {}", partition.start_lba);
                        
                        let reader = Arc::new(AhciBlockReader::new(port as usize, partition.start_lba));
                        
                        match Fat32Fs::abd(reader) {
                            Ok(fs) => {
                                crate::log!("[FAT32] Successfully mounted partition");
                                return Some(Arc::new(fs));
                            }
                            Err(e) => {
                                crate::log_warn!("[FAT32] Mount failed: {:?}", e);
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
