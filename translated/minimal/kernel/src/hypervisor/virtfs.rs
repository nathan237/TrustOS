




use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use alloc::collections::BTreeMap;
use spin::Mutex;


#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VirtFsOp {
    Re = 0,
    Bcb = 1,
    Cqh = 2,
    Ck = 3,
    Read = 4,
    Write = 5,
    Mx = 6,
    Stat = 7,
    Bqr = 8,
    Cac = 9,
    Ckj = 10,
    Ada = 11,
}

impl TryFrom<u32> for VirtFsOp {
    type Q = ();
    
    fn try_from(bn: u32) -> Result<Self, Self::Q> {
        match bn {
            0 => Ok(VirtFsOp::Re),
            1 => Ok(VirtFsOp::Bcb),
            2 => Ok(VirtFsOp::Cqh),
            3 => Ok(VirtFsOp::Ck),
            4 => Ok(VirtFsOp::Read),
            5 => Ok(VirtFsOp::Write),
            6 => Ok(VirtFsOp::Mx),
            7 => Ok(VirtFsOp::Stat),
            8 => Ok(VirtFsOp::Bqr),
            9 => Ok(VirtFsOp::Cac),
            10 => Ok(VirtFsOp::Ckj),
            11 => Ok(VirtFsOp::Ada),
            _ => Err(()),
        }
    }
}


#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VirtFsError {
    Hf = 0,
    N = 1,
    Jt = 2,
    Av = 3,
    Xe = 4,
    Ddd = 5,
    Daf = 6,
    Bnj = 7,
    Cwh = 8,
    Tq = 9,
    Aul = 10,
}


#[derive(Debug, Clone)]
struct Bgv {
    path: String,
    ta: bool,
    l: u64,
}


#[derive(Debug, Clone)]
pub struct Bsq {
    
    pub cac: String,
    
    pub bqx: String,
    
    pub awr: bool,
}


pub struct VirtFs {
    fk: u64,
    
    ajf: Vec<Bsq>,
    
    aho: BTreeMap<u32, Bgv>,
    
    bca: u32,
}

impl VirtFs {
    pub fn new(fk: u64) -> Self {
        VirtFs {
            fk,
            ajf: Vec::new(),
            aho: BTreeMap::new(),
            bca: 3, 
        }
    }
    
    
    pub fn elx(&mut self, cac: &str, bqx: &str, awr: bool) {
        self.ajf.push(Bsq {
            cac: String::from(cac),
            bqx: String::from(bqx),
            awr,
        });
        crate::serial_println!("[VirtFS] VM {} mounted {} -> {} (ro={})", 
                              self.fk, cac, bqx, awr);
    }
    
    
    fn aqj(&self, bqx: &str) -> Option<(String, bool)> {
        for beu in &self.ajf {
            if bqx.cj(&beu.bqx) {
                let atj = &bqx[beu.bqx.len()..];
                let atj = atj.tl('/');
                let tpu = if beu.cac.pp('/') || atj.is_empty() {
                    format!("{}{}", beu.cac, atj)
                } else {
                    format!("{}/{}", beu.cac, atj)
                };
                return Some((tpu, beu.awr));
            }
        }
        None
    }
    
    
    pub fn lba(&mut self, op: VirtFsOp, n: &[u64]) -> (u32, Vec<u8>) {
        match op {
            VirtFsOp::Re => {
                
                let dk = b"VirtFS 1.0\0";
                (VirtFsError::Hf as u32, dk.ip())
            }
            
            VirtFsOp::Bcb => {
                
                (VirtFsError::Hf as u32, vec![])
            }
            
            VirtFsOp::Ck => {
                if n.is_empty() {
                    return (VirtFsError::Xe as u32, vec![]);
                }
                
                
                
                let da = self.bca;
                self.bca += 1;
                
                self.aho.insert(da, Bgv {
                    path: String::from("/"),
                    ta: false,
                    l: 0,
                });
                
                (VirtFsError::Hf as u32, da.ho().ip())
            }
            
            VirtFsOp::Read => {
                if n.len() < 3 {
                    return (VirtFsError::Xe as u32, vec![]);
                }
                
                let da = n[0] as u32;
                let dnv = n[1];
                let az = n[2] as usize;
                
                if let Some(kvi) = self.aho.get(&da) {
                    
                    
                    if let Some((qcf, ycc)) = self.aqj(&kvi.path) {
                        
                        let f: Vec<u8> = vec![];
                        let ajp = core::cmp::v(az, f.len());
                        (VirtFsError::Hf as u32, f[..ajp].ip())
                    } else {
                        (VirtFsError::N as u32, vec![])
                    }
                } else {
                    (VirtFsError::Aul as u32, vec![])
                }
            }
            
            VirtFsOp::Write => {
                if n.len() < 2 {
                    return (VirtFsError::Xe as u32, vec![]);
                }
                
                let da = n[0] as u32;
                
                if let Some(kvi) = self.aho.get(&da) {
                    if let Some((qcf, awr)) = self.aqj(&kvi.path) {
                        if awr {
                            return (VirtFsError::Jt as u32, vec![]);
                        }
                        
                        (VirtFsError::Hf as u32, vec![])
                    } else {
                        (VirtFsError::N as u32, vec![])
                    }
                } else {
                    (VirtFsError::Aul as u32, vec![])
                }
            }
            
            VirtFsOp::Mx => {
                if n.is_empty() {
                    return (VirtFsError::Xe as u32, vec![]);
                }
                
                let da = n[0] as u32;
                if self.aho.remove(&da).is_some() {
                    (VirtFsError::Hf as u32, vec![])
                } else {
                    (VirtFsError::Aul as u32, vec![])
                }
            }
            
            VirtFsOp::Stat => {
                
                let hm = [0u64; 4]; 
                let bf: Vec<u8> = hm.iter()
                    .iva(|&p| p.ho())
                    .collect();
                (VirtFsError::Hf as u32, bf)
            }
            
            VirtFsOp::Bqr => {
                
                (VirtFsError::Hf as u32, vec![])
            }
            
            _ => (VirtFsError::Xe as u32, vec![]),
        }
    }
}


pub struct VirtFsManager {
    jaj: BTreeMap<u64, VirtFs>,
}

impl VirtFsManager {
    pub const fn new() -> Self {
        VirtFsManager {
            jaj: BTreeMap::new(),
        }
    }
    
    pub fn avp(&mut self, fk: u64) -> &mut VirtFs {
        self.jaj.bt(fk).clq(|| VirtFs::new(fk))
    }
    
    pub fn get(&mut self, fk: u64) -> Option<&mut VirtFs> {
        self.jaj.ds(&fk)
    }
    
    pub fn remove(&mut self, fk: u64) {
        self.jaj.remove(&fk);
    }
}

static YH_: Mutex<VirtFsManager> = Mutex::new(VirtFsManager::new());


pub fn nhj(fk: u64) -> () {
    YH_.lock().avp(fk);
}


pub fn elx(fk: u64, cac: &str, bqx: &str, awr: bool) {
    let mut aas = YH_.lock();
    if let Some(vfs) = aas.get(fk) {
        vfs.elx(cac, bqx, awr);
    }
}


pub fn vuz(fk: u64) {
    YH_.lock().remove(fk);
}


pub fn lau(fk: u64, op: u32, n: &[u64]) -> (u32, Vec<u8>) {
    let mut aas = YH_.lock();
    
    if let Some(vfs) = aas.get(fk) {
        if let Ok(mpi) = VirtFsOp::try_from(op) {
            vfs.lba(mpi, n)
        } else {
            (VirtFsError::Xe as u32, vec![])
        }
    } else {
        (VirtFsError::N as u32, vec![])
    }
}
