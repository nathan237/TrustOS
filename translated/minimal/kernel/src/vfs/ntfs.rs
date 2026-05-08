



















use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::vec;
use alloc::sync::Arc;
use alloc::format;
use spin::Mutex;

use super::{Bx, Bv, Au, FileType, Stat, Ap, E, VfsError, K};
use super::fat32::Ak;






const BCP_: u32 = 0x454C4946; 


const BDU_: &[u8; 8] = b"NTFS    ";


const DXQ_: u64 = 0;          
const BCQ_: u64 = 5;         


const BNE_: u32 = 0x10;
const BNB_: u32 = 0x30;
const AMR_: u32 = 0x80;
const BND_: u32 = 0x90;
const BNC_: u32 = 0xA0;
const DGM_: u32 = 0xB0;
const AMS_: u32 = 0xFFFFFFFF;


const AUD_: u8 = 0;
const AUE_: u8 = 1;
const ADT_: u8 = 2;
const AUF_: u8 = 3;


const DXP_: u16 = 0x0001;
const CJS_: u16 = 0x0002;


const DUS_: u32 = 0x01;
const CEY_: u32 = 0x02;


const H_: usize = 512;






#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Abu {
    jmp_boot: [u8; 3],          
    oem_id: [u8; 8],            
    bytes_per_sector: u16,      
    sectors_per_cluster: u8,    
    _reserved1: [u8; 7],        
    media_descriptor: u8,       
    _reserved2: [u8; 2],        
    sectors_per_track: u16,     
    num_heads: u16,             
    hidden_sectors: u32,        
    _reserved3: u32,            
    _reserved4: u32,            
    zp: u64,         
    mft_lcn: u64,               
    mft_mirror_lcn: u64,        
    mft_record_size: i8,        
    _reserved5: [u8; 3],        
    index_block_size: i8,       
    _reserved6: [u8; 3],        
    volume_serial: u64,         
}

impl Abu {
    fn is_valid(&self) -> bool {
        self.oem_id == *BDU_
    }

    fn cluster_size(&self) -> u32 {
        let djm = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(self.bytes_per_sector)) };
        djm as u32 * self.sectors_per_cluster as u32
    }

    fn mft_record_bytes(&self) -> u32 {
        if self.mft_record_size > 0 {
            self.mft_record_size as u32 * self.cluster_size()
        } else {
            
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


#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
struct Amu {
    magic: u32,                 
    dgb: u16,     
    edj: u16,       
    log_seq_number: u64,        
    sequence_number: u16,       
    hard_link_count: u16,       
    first_attr_offset: u16,     
    flags: u16,                 
    used_size: u32,             
    allocated_size: u32,        
    base_record: u64,           
    next_attr_id: u16,          
}


#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
struct Asv {
    dhz: u32,             
    length: u32,                
    dbu: u8,           
    name_length: u8,            
    name_offset: u16,           
    flags: u16,                 
    attr_id: u16,               
}


#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
struct Bbk {
    value_length: u32,          
    value_offset: u16,          
    indexed_flag: u16,          
}


#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
struct Ty {
    lowest_vcn: u64,            
    highest_vcn: u64,           
    data_runs_offset: u16,      
    compression_unit: u16,      
    _padding: u32,              
    allocated_size: u64,        
    real_size: u64,             
    initialized_size: u64,      
}


#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Yw {
    parent_ref: u64,            
    creation_time: u64,         
    modification_time: u64,     
    mft_modification_time: u64, 
    access_time: u64,           
    allocated_size: u64,        
    real_size: u64,             
    flags: u32,                 
    reparse_or_ea: u32,        
    name_length: u8,            
    namespace: u8,              
    
}


#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Aqf {
    creation_time: u64,
    modification_time: u64,
    mft_modification_time: u64,
    access_time: u64,
    file_attributes: u32,       
    _padding: [u8; 4],
}


#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Axx {
    dhz: u32,             
    collation_rule: u32,        
    index_block_size: u32,      
    clusters_per_index: u8,     
    _padding: [u8; 3],
}


#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Axw {
    hwb: u32,        
    total_size: u32,            
    allocated_size: u32,        
    flags: u32,                 
}


#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Alr {
    mft_reference: u64,         
    entry_length: u16,          
    content_length: u16,        
    flags: u32,                 
}


const CEZ_: u32 = 0x58444E49; 

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Axv {
    magic: u32,                 
    dgb: u16,
    edj: u16,
    log_seq_number: u64,
    avq: u64,                   
    
}






#[derive(Clone, Debug)]
struct Gj {
    
    vcn_start: u64,
    
    length: u64,
    
    lcn: u64,
}


fn frb(data: &[u8]) -> Vec<Gj> {
    let mut ezh = Vec::new();
    let mut offset = 0usize;
    let mut chz: u64 = 0;
    let mut hps: i64 = 0;

    while offset < data.len() {
        let header = data[offset];
        if header == 0 {
            break; 
        }

        let esv = (header & 0x0F) as usize;
        let cnj = ((header >> 4) & 0x0F) as usize;
        offset += 1;

        if esv == 0 || offset + esv + cnj > data.len() {
            break;
        }

        
        let mut gsc: u64 = 0;
        for i in 0..esv {
            gsc |= (data[offset + i] as u64) << (i * 8);
        }
        offset += esv;

        
        let mut ezg: i64 = 0;
        if cnj > 0 {
            for i in 0..cnj {
                ezg |= (data[offset + i] as i64) << (i * 8);
            }
            
            let osp = 1i64 << (cnj * 8 - 1);
            if ezg & osp != 0 {
                ezg |= !((1i64 << (cnj * 8)) - 1);
            }
            offset += cnj;

            hps += ezg;
        }

        let lcn = if cnj == 0 {
            0 
        } else {
            hps as u64
        };

        ezh.push(Gj {
            vcn_start: chz,
            length: gsc,
            lcn,
        });

        chz += gsc;
    }

    ezh
}






#[derive(Clone)]
struct Ke {
    
    record_number: u64,
    
    flags: u16,
    
    fwo: String,
    
    parent_ref: u64,
    
    file_size: u64,
    
    is_directory: bool,
    
    creation_time: u64,
    modification_time: u64,
    access_time: u64,
    
    file_attributes: u32,
    
    data_runs: Vec<Gj>,
    
    data_resident: bool,
    
    resident_data: Vec<u8>,
    
    index_root_data: Vec<u8>,
    
    index_alloc_runs: Vec<Gj>,
}






pub struct Pq {
    inner: Mutex<Ik>,
}

struct Ik {
    device: Arc<dyn Ak>,
    cluster_size: u32,
    mft_record_size: u32,
    index_block_size: u32,
    mft_start_byte: u64,
    sectors_per_cluster: u8,
    bytes_per_sector: u16,
    
    mft_data_runs: Vec<Gj>,
}

impl Ik {
    
    fn read_bytes(&self, uo: u64, buf: &mut [u8]) -> Result<(), ()> {
        let sector_size = self.device.sector_size() as u64;
        let start_sector = uo / sector_size;
        let afl = (uo % sector_size) as usize;

        let total_bytes = afl + buf.len();
        let gkd = (total_bytes + sector_size as usize - 1) / sector_size as usize;

        let mut ck = buf.len();
        let mut yj = 0usize;
        let mut mx = vec![0u8; sector_size as usize];

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

    
    fn qrw(&self, lcn: u64, count: u64, buf: &mut [u8]) -> Result<(), ()> {
        let uo = lcn * self.cluster_size as u64;
        let hjj = count as usize * self.cluster_size as usize;
        if buf.len() < hjj {
            return Err(());
        }
        self.read_bytes(uo, &mut buf[..hjj])
    }

    
    fn apply_fixups(&self, buf: &mut [u8], ddh: usize) -> Result<(), ()> {
        if buf.len() < 6 {
            return Err(());
        }
        let dgb = u16::from_le_bytes([buf[4], buf[5]]) as usize;
        let edj = u16::from_le_bytes([buf[6], buf[7]]) as usize;

        if edj < 2 || dgb + edj * 2 > buf.len() {
            return Err(());
        }

        
        let signature = u16::from_le_bytes([
            buf[dgb],
            buf[dgb + 1],
        ]);

        
        let sector_size = self.bytes_per_sector as usize;
        for i in 1..edj {
            let gtj = i * sector_size;
            if gtj > ddh || gtj < 2 {
                break;
            }
            let emu = gtj - 2;

            
            let gwl = u16::from_le_bytes([buf[emu], buf[emu + 1]]);
            if gwl != signature {
                return Err(()); 
            }

            
            let gsm = dgb + i * 2;
            if gsm + 1 < buf.len() {
                buf[emu] = buf[gsm];
                buf[emu + 1] = buf[gsm + 1];
            }
        }

        Ok(())
    }

    
    fn read_mft_record_raw(&self, record_num: u64) -> Result<Vec<u8>, ()> {
        let ddh = self.mft_record_size as usize;
        let mut buf = vec![0u8; ddh];

        
        let hjk = record_num * ddh as u64;
        let avq = hjk / self.cluster_size as u64;
        let bik = (hjk % self.cluster_size as u64) as usize;

        
        let mut fki = ddh;
        let mut fka = 0;
        let mut chz = avq;
        let mut fpu = bik;

        while fki > 0 {
            
            let run = self.mft_data_runs.iter().find(|r| {
                chz >= r.vcn_start && chz < r.vcn_start + r.length
            });

            match run {
                Some(run) => {
                    let prh = chz - run.vcn_start;
                    let lcn = run.lcn + prh;
                    let uo = lcn * self.cluster_size as u64 + fpu as u64;

                    let jys = self.cluster_size as usize - fpu;
                    let rz = fki.min(jys);

                    self.read_bytes(uo, &mut buf[fka..fka + rz])?;

                    fka += rz;
                    fki -= rz;
                    fpu = 0;
                    chz += 1;
                }
                None => return Err(()),
            }
        }

        
        self.apply_fixups(&mut buf, ddh)?;

        
        let magic = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
        if magic != BCP_ {
            return Err(());
        }

        Ok(buf)
    }

    
    fn parse_mft_record(&self, record_num: u64, buf: &[u8]) -> Result<Ke, ()> {
        let header = unsafe {
            core::ptr::read_unaligned(buf.as_ptr() as *const Amu)
        };

        let flags = header.flags;
        let is_directory = (flags & CJS_) != 0;
        let fxb = header.first_attr_offset as usize;
        let used_size = header.used_size as usize;

        let mut fwo = String::new();
        let mut hhj: Option<u8> = None;
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

        let mut offset = fxb;
        let jm = used_size.min(buf.len());

        while offset + 4 <= jm {
            let dhz = u32::from_le_bytes([
                buf[offset], buf[offset + 1], buf[offset + 2], buf[offset + 3],
            ]);

            if dhz == AMS_ || dhz == 0 {
                break;
            }

            if offset + 8 > jm {
                break;
            }

            let cfy = u32::from_le_bytes([
                buf[offset + 4], buf[offset + 5], buf[offset + 6], buf[offset + 7],
            ]) as usize;

            if cfy < 16 || cfy > jm - offset {
                break;
            }

            let dbu = buf[offset + 8];
            let name_length = buf[offset + 9] as usize;

            
            let mtz = name_length == 0;

            match dhz {
                BNE_ if dbu == 0 => {
                    
                    if offset + 24 <= jm {
                        let bpt = u32::from_le_bytes([
                            buf[offset + 16], buf[offset + 17],
                            buf[offset + 18], buf[offset + 19],
                        ]) as usize;
                        let csh = u16::from_le_bytes([
                            buf[offset + 20], buf[offset + 21],
                        ]) as usize;
                        let data_start = offset + csh;
                        if bpt >= 48 && data_start + 48 <= buf.len() {
                            let si = unsafe {
                                core::ptr::read_unaligned(
                                    buf[data_start..].as_ptr() as *const Aqf
                                )
                            };
                            creation_time = si.creation_time;
                            modification_time = si.modification_time;
                            access_time = si.access_time;
                            file_attributes = si.file_attributes;
                        }
                    }
                }

                BNB_ if dbu == 0 => {
                    
                    if offset + 24 <= jm {
                        let bpt = u32::from_le_bytes([
                            buf[offset + 16], buf[offset + 17],
                            buf[offset + 18], buf[offset + 19],
                        ]) as usize;
                        let csh = u16::from_le_bytes([
                            buf[offset + 20], buf[offset + 21],
                        ]) as usize;
                        let data_start = offset + csh;
                        if bpt >= 66 && data_start + 66 <= buf.len() {
                            let bzv = unsafe {
                                core::ptr::read_unaligned(
                                    buf[data_start..].as_ptr() as *const Yw
                                )
                            };
                            let ayq = bzv.namespace;
                            let duu = bzv.name_length as usize;
                            let sj = data_start + 66;

                            
                            
                            let priority = match ayq {
                                AUE_ => 4,
                                AUF_ => 3,
                                AUD_ => 2,
                                ADT_ => 1,
                                _ => 0,
                            };
                            let fpv = hhj.map(|ae| match ae {
                                AUE_ => 4,
                                AUF_ => 3,
                                AUD_ => 2,
                                ADT_ => 1,
                                _ => 0,
                            }).unwrap_or(0);

                            if priority > fpv {
                                if sj + duu * 2 <= buf.len() {
                                    fwo = hrb(
                                        &buf[sj..sj + duu * 2]
                                    );
                                    hhj = Some(ayq);
                                    parent_ref = bzv.parent_ref & 0x0000FFFFFFFFFFFF;

                                    
                                    if file_size == 0 {
                                        file_size = unsafe {
                                            core::ptr::read_unaligned(
                                                core::ptr::addr_of!(bzv.real_size)
                                            )
                                        };
                                    }
                                }
                            }
                        }
                    }
                }

                AMR_ if mtz => {
                    if dbu == 0 {
                        
                        data_resident = true;
                        if offset + 24 <= jm {
                            let bpt = u32::from_le_bytes([
                                buf[offset + 16], buf[offset + 17],
                                buf[offset + 18], buf[offset + 19],
                            ]) as usize;
                            let csh = u16::from_le_bytes([
                                buf[offset + 20], buf[offset + 21],
                            ]) as usize;
                            let data_start = offset + csh;
                            if data_start + bpt <= buf.len() {
                                resident_data = buf[data_start..data_start + bpt].to_vec();
                                file_size = bpt as u64;
                            }
                        }
                    } else {
                        
                        data_resident = false;
                        if offset + 64 <= jm {
                            let evj = unsafe {
                                core::ptr::read_unaligned(
                                    buf[offset + 16..].as_ptr() as *const Ty
                                )
                            };
                            file_size = evj.real_size;
                            let gse = unsafe {
                                core::ptr::read_unaligned(
                                    core::ptr::addr_of!(evj.data_runs_offset)
                                )
                            } as usize;
                            let cdn = offset + gse;
                            if cdn < offset + cfy {
                                data_runs = frb(
                                    &buf[cdn..offset + cfy]
                                );
                            }
                        }
                    }
                }

                BND_ if dbu == 0 => {
                    
                    if offset + 24 <= jm {
                        let bpt = u32::from_le_bytes([
                            buf[offset + 16], buf[offset + 17],
                            buf[offset + 18], buf[offset + 19],
                        ]) as usize;
                        let csh = u16::from_le_bytes([
                            buf[offset + 20], buf[offset + 21],
                        ]) as usize;
                        let data_start = offset + csh;
                        if data_start + bpt <= buf.len() {
                            index_root_data = buf[data_start..data_start + bpt].to_vec();
                        }
                    }
                }

                BNC_ if dbu != 0 => {
                    
                    if offset + 64 <= jm {
                        let evj = unsafe {
                            core::ptr::read_unaligned(
                                buf[offset + 16..].as_ptr() as *const Ty
                            )
                        };
                        let gse = unsafe {
                            core::ptr::read_unaligned(
                                core::ptr::addr_of!(evj.data_runs_offset)
                            )
                        } as usize;
                        let cdn = offset + gse;
                        if cdn < offset + cfy {
                            index_alloc_runs = frb(
                                &buf[cdn..offset + cfy]
                            );
                        }
                    }
                }

                _ => {}
            }

            offset += cfy;
        }

        Ok(Ke {
            record_number: record_num,
            flags,
            fwo,
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
        })
    }

    
    fn read_mft_record(&self, record_num: u64) -> Result<Ke, ()> {
        let dm = self.read_mft_record_raw(record_num)?;
        self.parse_mft_record(record_num, &dm)
    }

    
    fn read_file_data(
        &self,
        record: &Ke,
        aaw: u64,
        buf: &mut [u8],
    ) -> Result<usize, ()> {
        if aaw >= record.file_size {
            return Ok(0);
        }

        let arx = ((record.file_size - aaw) as usize).min(buf.len());
        if arx == 0 {
            return Ok(0);
        }

        if record.data_resident {
            
            let start = aaw as usize;
            let end = start + arx;
            if end <= record.resident_data.len() {
                buf[..arx].copy_from_slice(&record.resident_data[start..end]);
            } else {
                let avail = record.resident_data.len().saturating_sub(start);
                buf[..avail].copy_from_slice(&record.resident_data[start..start + avail]);
            }
            return Ok(arx);
        }

        
        let cluster_size = self.cluster_size as u64;
        let mut ck = arx;
        let mut yj = 0usize;
        let mut offset = aaw;

        while ck > 0 {
            let avq = offset / cluster_size;
            let bik = (offset % cluster_size) as usize;

            
            let run = record.data_runs.iter().find(|r| {
                avq >= r.vcn_start && avq < r.vcn_start + r.length
            });

            match run {
                Some(run) if run.lcn > 0 => {
                    let hbg = avq - run.vcn_start;
                    let lcn = run.lcn + hbg;
                    let uo = lcn * cluster_size + bik as u64;

                    let available = cluster_size as usize - bik;
                    let rz = ck.min(available);

                    self.read_bytes(uo, &mut buf[yj..yj + rz])?;

                    yj += rz;
                    offset += rz as u64;
                    ck -= rz;
                }
                Some(_) => {
                    
                    let available = cluster_size as usize - bik;
                    let fcz = ck.min(available);
                    for b in &mut buf[yj..yj + fcz] {
                        *b = 0;
                    }
                    yj += fcz;
                    offset += fcz as u64;
                    ck -= fcz;
                }
                None => {
                    
                    for b in &mut buf[yj..yj + ck] {
                        *b = 0;
                    }
                    ck = 0;
                }
            }
        }

        Ok(arx)
    }

    
    fn read_from_runs(&self, ezh: &[Gj], avq: u64, buf: &mut [u8]) -> Result<(), ()> {
        let run = ezh.iter().find(|r| {
            avq >= r.vcn_start && avq < r.vcn_start + r.length
        });

        match run {
            Some(run) if run.lcn > 0 => {
                let hbg = avq - run.vcn_start;
                let lcn = run.lcn + hbg;
                let fmc = (buf.len() + self.cluster_size as usize - 1)
                    / self.cluster_size as usize;
                
                for i in 0..fmc {
                    let kgm = (lcn + i as u64) * self.cluster_size as u64;
                    let hiz = i * self.cluster_size as usize;
                    let keo = (hiz + self.cluster_size as usize).min(buf.len());
                    self.read_bytes(kgm, &mut buf[hiz..keo])?;
                }
                Ok(())
            }
            _ => Err(()),
        }
    }

    
    fn read_dir_entries(&self, record: &Ke) -> Result<Vec<(u64, String, bool)>, ()> {
        let mut entries = Vec::new();

        
        if record.index_root_data.len() >= 32 {
            let bhh = &record.index_root_data;

            
            let bic = 16; 
            if bic + 16 <= bhh.len() {
                let hwb = u32::from_le_bytes([
                    bhh[bic], bhh[bic + 1],
                    bhh[bic + 2], bhh[bic + 3],
                ]) as usize;
                let total_size = u32::from_le_bytes([
                    bhh[bic + 4], bhh[bic + 5],
                    bhh[bic + 6], bhh[bic + 7],
                ]) as usize;

                let start = bic + hwb;
                let end = (bic + total_size).min(bhh.len());

                self.parse_index_entries(&bhh[start..end], &mut entries);
            }
        }

        
        if !record.index_alloc_runs.is_empty() {
            let index_block_size = self.index_block_size as usize;
            let kli = (index_block_size + self.cluster_size as usize - 1)
                / self.cluster_size as usize;

            
            let pmj: u64 = record.index_alloc_runs.iter()
                .map(|r| r.length)
                .sum();

            let mut avq: u64 = 0;
            while avq < pmj {
                let mut se = vec![0u8; index_block_size];
                if self.read_from_runs(&record.index_alloc_runs, avq, &mut se).is_ok() {
                    
                    let _ = self.apply_fixups(&mut se, index_block_size);

                    let magic = u32::from_le_bytes([
                        se[0], se[1], se[2], se[3],
                    ]);
                    if magic == CEZ_ {
                        
                        let bib = 0x18;
                        if bib + 16 <= se.len() {
                            let lqy = u32::from_le_bytes([
                                se[bib], se[bib + 1],
                                se[bib + 2], se[bib + 3],
                            ]) as usize;
                            let jy = u32::from_le_bytes([
                                se[bib + 4], se[bib + 5],
                                se[bib + 6], se[bib + 7],
                            ]) as usize;

                            let start = bib + lqy;
                            let end = (bib + jy).min(se.len());
                            if start < end {
                                self.parse_index_entries(&se[start..end], &mut entries);
                            }
                        }
                    }
                }
                avq += kli as u64;
            }
        }

        Ok(entries)
    }

    
    fn parse_index_entries(
        &self,
        data: &[u8],
        entries: &mut Vec<(u64, String, bool)>,
    ) {
        let mut pos = 0;
        while pos + 16 <= data.len() {
            let aob = unsafe {
                core::ptr::read_unaligned(data[pos..].as_ptr() as *const Alr)
            };

            let alm = aob.entry_length as usize;
            let anw = aob.content_length as usize;
            let flags = aob.flags;

            if alm < 16 || alm > data.len() - pos {
                break;
            }

            if (flags & CEY_) != 0 {
                break; 
            }

            if anw >= 66 {
                
                let brc = pos + 16; 
                if brc + anw <= data.len() {
                    let ena = &data[brc..brc + anw];
                    if ena.len() >= 66 {
                        let bzv = unsafe {
                            core::ptr::read_unaligned(
                                ena.as_ptr() as *const Yw
                            )
                        };

                        let ayq = bzv.namespace;
                        
                        if ayq != ADT_ {
                            let duu = bzv.name_length as usize;
                            let sj = 66;
                            if sj + duu * 2 <= ena.len() {
                                let name = hrb(
                                    &ena[sj..sj + duu * 2]
                                );

                                let duh = aob.mft_reference & 0x0000FFFFFFFFFFFF;
                                let lwr = unsafe {
                                    core::ptr::read_unaligned(
                                        core::ptr::addr_of!(bzv.flags)
                                    )
                                };
                                let is_dir = (lwr & 0x10000000) != 0;

                                
                                if !name.starts_with('$') && !name.is_empty() {
                                    entries.push((duh, name, is_dir));
                                }
                            }
                        }
                    }
                }
            }

            pos += alm;
        }
    }

    
    fn dir_lookup(&self, dir_record_num: u64, name: &str) -> Result<u64, ()> {
        let record = self.read_mft_record(dir_record_num)?;
        let entries = self.read_dir_entries(&record)?;
        for (duh, bbl, _is_dir) in &entries {
            if bbl.eq_ignore_ascii_case(name) {
                return Ok(*duh);
            }
        }
        Err(())
    }

    
    fn record_file_type(&self, record: &Ke) -> FileType {
        if record.is_directory {
            FileType::Directory
        } else {
            FileType::Regular
        }
    }
}






fn hrb(data: &[u8]) -> String {
    let mut chars = Vec::with_capacity(data.len() / 2);
    for df in data.chunks_exact(2) {
        let kuw = u16::from_le_bytes([df[0], df[1]]);
        chars.push(kuw);
    }

    
    let mut result = String::with_capacity(chars.len());
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c >= 0xD800 && c <= 0xDBFF && i + 1 < chars.len() {
            
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






fn bun(ntfs_time: u64) -> u64 {
    if ntfs_time == 0 {
        return 0;
    }
    
    
    const CLI_: u64 = 11644473600;
    let abi = ntfs_time / 10_000_000; 
    abi.saturating_sub(CLI_)
}






struct Ub {
    record_num: u64,
    device: Arc<dyn Ak>,
    cluster_size: u32,
    mft_record_size: u32,
    index_block_size: u32,
    mft_start_byte: u64,
    sectors_per_cluster: u8,
    bytes_per_sector: u16,
    mft_data_runs: Vec<Gj>,
}

impl Ub {
    fn make_inner(&self) -> Ik {
        Ik {
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

impl Bx for Ub {
    fn read(&self, offset: u64, buf: &mut [u8]) -> E<usize> {
        let inner = self.make_inner();
        let record = inner.read_mft_record(self.record_num)
            .map_err(|_| VfsError::IoError)?;
        inner.read_file_data(&record, offset, buf)
            .map_err(|_| VfsError::IoError)
    }

    fn write(&self, bkm: u64, _buf: &[u8]) -> E<usize> {
        Err(VfsError::ReadOnly)
    }

    fn stat(&self) -> E<Stat> {
        let inner = self.make_inner();
        let record = inner.read_mft_record(self.record_num)
            .map_err(|_| VfsError::IoError)?;
        Ok(Stat {
            ino: self.record_num,
            file_type: inner.record_file_type(&record),
            size: record.file_size,
            blocks: (record.file_size + 511) / 512,
            block_size: inner.cluster_size,
            mode: 0o444, 
            uid: 0,
            gid: 0,
            atime: bun(record.access_time),
            mtime: bun(record.modification_time),
            ctime: bun(record.creation_time),
        })
    }
}


struct Ua {
    record_num: u64,
    device: Arc<dyn Ak>,
    cluster_size: u32,
    mft_record_size: u32,
    index_block_size: u32,
    mft_start_byte: u64,
    sectors_per_cluster: u8,
    bytes_per_sector: u16,
    mft_data_runs: Vec<Gj>,
}

impl Ua {
    fn make_inner(&self) -> Ik {
        Ik {
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

impl Bv for Ua {
    fn lookup(&self, name: &str) -> E<K> {
        let inner = self.make_inner();
        inner.dir_lookup(self.record_num, name)
            .map_err(|_| VfsError::NotFound)
    }

    fn readdir(&self) -> E<Vec<Ap>> {
        let inner = self.make_inner();
        let record = inner.read_mft_record(self.record_num)
            .map_err(|_| VfsError::IoError)?;
        let entries = inner.read_dir_entries(&record)
            .map_err(|_| VfsError::IoError)?;

        Ok(entries.into_iter()
            .map(|(duh, name, is_dir)| Ap {
                name,
                ino: duh,
                file_type: if is_dir { FileType::Directory } else { FileType::Regular },
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
        let record = inner.read_mft_record(self.record_num)
            .map_err(|_| VfsError::IoError)?;
        Ok(Stat {
            ino: self.record_num,
            file_type: FileType::Directory,
            size: record.file_size,
            blocks: 0,
            block_size: inner.cluster_size,
            mode: 0o555, 
            uid: 0,
            gid: 0,
            atime: bun(record.access_time),
            mtime: bun(record.modification_time),
            ctime: bun(record.creation_time),
        })
    }
}


impl Au for Pq {
    fn name(&self) -> &str { "ntfs" }

    fn root_inode(&self) -> K { BCQ_ }

    fn get_file(&self, ino: K) -> E<Arc<dyn Bx>> {
        let inner = self.inner.lock();
        let record = inner.read_mft_record(ino).map_err(|_| VfsError::NotFound)?;
        if record.is_directory {
            return Err(VfsError::IsDirectory);
        }
        Ok(Arc::new(Ub {
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

    fn get_dir(&self, ino: K) -> E<Arc<dyn Bv>> {
        let inner = self.inner.lock();
        let record = inner.read_mft_record(ino).map_err(|_| VfsError::NotFound)?;
        if !record.is_directory {
            return Err(VfsError::NotDirectory);
        }
        Ok(Arc::new(Ua {
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

    fn stat(&self, ino: K) -> E<Stat> {
        let inner = self.inner.lock();
        let record = inner.read_mft_record(ino).map_err(|_| VfsError::NotFound)?;
        let qk = inner.record_file_type(&record);
        Ok(Stat {
            ino,
            file_type: qk,
            size: record.file_size,
            blocks: (record.file_size + 511) / 512,
            block_size: inner.cluster_size,
            mode: if record.is_directory { 0o555 } else { 0o444 },
            uid: 0,
            gid: 0,
            atime: bun(record.access_time),
            mtime: bun(record.modification_time),
            ctime: bun(record.creation_time),
        })
    }
}






pub fn abd(device: Arc<dyn Ak>) -> Result<Arc<Pq>, &'static str> {
    
    let mut bap = [0u8; H_];
    device.read_sector(0, &mut bap).map_err(|_| "Failed to read NTFS boot sector")?;

    
    let bpb = unsafe { core::ptr::read_unaligned(bap.as_ptr() as *const Abu) };
    if !bpb.is_valid() {
        return Err("Not an NTFS filesystem (bad OEM ID)");
    }

    
    if bap[510] != 0x55 || bap[511] != 0xAA {
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

    
    
    let mut qa = vec![0u8; mft_record_size as usize];
    let sector_size = device.sector_size() as u64;
    let nfe = mft_start_byte / sector_size;
    let bdq = (mft_record_size as u64 + sector_size - 1) / sector_size;

    let mut ixw = vec![0u8; (bdq * sector_size) as usize];
    for i in 0..bdq {
        device.read_sector(nfe + i, 
            &mut ixw[(i as usize * sector_size as usize)..((i + 1) as usize * sector_size as usize)])
            .map_err(|_| "Failed to read MFT record 0")?;
    }
    qa.copy_from_slice(&ixw[..mft_record_size as usize]);

    
    {
        if qa.len() < 8 {
            return Err("MFT record too small");
        }
        let feb = u16::from_le_bytes([qa[4], qa[5]]) as usize;
        let hav = u16::from_le_bytes([qa[6], qa[7]]) as usize;
        if hav >= 2 && feb + hav * 2 <= qa.len() {
            let sig = u16::from_le_bytes([qa[feb], qa[feb + 1]]);
            let omv = bytes_per_sector as usize;
            for i in 1..hav {
                let gti = i * omv;
                if gti <= qa.len() && gti >= 2 {
                    let pos = gti - 2;
                    let gwl = u16::from_le_bytes([qa[pos], qa[pos + 1]]);
                    if gwl == sig {
                        let gsl = feb + i * 2;
                        if gsl + 1 < qa.len() {
                            qa[pos] = qa[gsl];
                            qa[pos + 1] = qa[gsl + 1];
                        }
                    }
                }
            }
        }
    }

    
    let magic = u32::from_le_bytes([qa[0], qa[1], qa[2], qa[3]]);
    if magic != BCP_ {
        return Err("MFT record 0 has bad magic");
    }

    
    let fxb = u16::from_le_bytes([qa[20], qa[21]]) as usize;
    let used_size = u32::from_le_bytes([qa[24], qa[25], qa[26], qa[27]]) as usize;
    let mut mft_data_runs = Vec::new();

    let mut off = fxb;
    let jm = used_size.min(qa.len());
    while off + 8 <= jm {
        let dia = u32::from_le_bytes([
            qa[off], qa[off + 1], qa[off + 2], qa[off + 3],
        ]);
        let dhh = u32::from_le_bytes([
            qa[off + 4], qa[off + 5], qa[off + 6], qa[off + 7],
        ]) as usize;

        if dia == AMS_ || dia == 0 || dhh < 16 || dhh > jm - off {
            break;
        }

        if dia == AMR_ && off + 9 < jm && qa[off + 8] == 1 {
            
            let name_len = qa[off + 9] as usize;
            if name_len == 0 && off + 64 <= jm {
                let nr = unsafe {
                    core::ptr::read_unaligned(
                        qa[off + 16..].as_ptr() as *const Ty
                    )
                };
                let oji = unsafe {
                    core::ptr::read_unaligned(core::ptr::addr_of!(nr.data_runs_offset))
                } as usize;
                let cdn = off + oji;
                if cdn < off + dhh {
                    mft_data_runs = frb(&qa[cdn..off + dhh]);
                }
            }
        }

        off += dhh;
    }

    if mft_data_runs.is_empty() {
        return Err("Failed to parse $MFT data runs");
    }

    let pmb: u64 = mft_data_runs.iter().map(|r| r.length).sum();
    crate::serial_println!("[NTFS] $MFT: {} data runs, {} clusters total",
        mft_data_runs.len(), pmb);

    let fs = Arc::new(Pq {
        inner: Mutex::new(Ik {
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

    
    {
        let inner = fs.inner.lock();
        match inner.read_mft_record(BCQ_) {
            Ok(cdl) => {
                if !cdl.is_directory {
                    return Err("MFT record 5 is not a directory");
                }
                crate::serial_println!("[NTFS] Root directory OK, reading entries...");
                match inner.read_dir_entries(&cdl) {
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


pub fn probe(device: &dyn Ak) -> bool {
    let mut bap = [0u8; H_];
    if device.read_sector(0, &mut bap).is_err() {
        return false;
    }
    
    &bap[3..11] == BDU_
}


pub fn pny() -> Option<Arc<Pq>> {
    use crate::drivers::partition::{dwf, PartitionType};
    use crate::drivers::ahci;
    use super::fat32::AhciBlockReader;

    let devices = ahci::adz();
    crate::serial_println!("[NTFS] Scanning {} AHCI devices for NTFS partitions", devices.len());

    for device in devices {
        let port = device.port_num;
        let zp = device.sector_count;

        let read_fn = |dj: u64, buf: &mut [u8]| -> Result<(), &'static str> {
            ahci::read_sectors(port, dj, 1, buf).map(|_| ())
        };

        if let Ok(bs) = dwf(read_fn, zp) {
            for partition in &bs.partitions {
                match partition.partition_type {
                    PartitionType::Ntfs | PartitionType::MicrosoftBasicData => {
                        crate::serial_println!("[NTFS] Found candidate partition at LBA {} ({})",
                            partition.start_lba, partition.size_human());

                        let reader = Arc::new(AhciBlockReader::new(
                            port as usize,
                            partition.start_lba,
                        ));

                        
                        if probe(&*reader) {
                            match abd(reader) {
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
