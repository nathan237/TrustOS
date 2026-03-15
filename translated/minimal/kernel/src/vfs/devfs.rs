



use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::sync::Arc;
use core::sync::atomic::{AtomicU64, Ordering};

use super::{
    Cc, Et, Ep, Stat, Br, FileType, 
    I, B, VfsError
};


#[derive(Clone, Copy, Debug)]
enum DeviceType {
    Gm,       
    Bbg,       
    Alu,     
    Rw,    
    Zi,        
}


struct Ben {
    cwx: DeviceType,
    dd: I,
}

impl Et for Ben {
    fn read(&self, l: u64, k: &mut [u8]) -> B<usize> {
        match self.cwx {
            DeviceType::Gm => Ok(0), 
            DeviceType::Bbg => {
                for o in k.el() {
                    *o = 0;
                }
                Ok(k.len())
            }
            DeviceType::Alu => {
                
                for o in k.el() {
                    *o = crate::rng::hsw() as u8;
                }
                Ok(k.len())
            }
            DeviceType::Rw => {
                
                
                let mut es = 0usize;
                let ulx = 500_000_000u64; 
                let mut aaf = 0u64;
                
                while es == 0 && aaf < ulx {
                    while es < k.len() {
                        if let Some(bm) = crate::keyboard::auw() {
                            k[es] = bm;
                            es += 1;
                            
                            if bm == b'\n' || bm == b'\r' {
                                if bm == b'\r' {
                                    k[es - 1] = b'\n';
                                }
                                return Ok(es);
                            }
                        } else {
                            break; 
                        }
                    }
                    if es == 0 {
                        
                        core::hint::hc();
                        aaf += 1;
                    }
                }
                Ok(es)
            }
            DeviceType::Zi => {
                
                if !crate::virtio_blk::ky() {
                    return Err(VfsError::Av);
                }
                let zn = 512u64;
                let awy = l / zn;
                let bho = (l % zn) as usize;
                let mut gut = 0usize;
                let mut gru = awy;
                let mut u = bho;
                let mut aae = [0u8; 512];
                while gut < k.len() {
                    if crate::virtio_blk::xr(gru, &mut aae).is_err() {
                        break;
                    }
                    let apk = 512 - u;
                    let acq = core::cmp::v(apk, k.len() - gut);
                    k[gut..gut + acq].dg(&aae[u..u + acq]);
                    gut += acq;
                    gru += 1;
                    u = 0;
                }
                Ok(gut)
            }
        }
    }
    
    fn write(&self, l: u64, k: &[u8]) -> B<usize> {
        match self.cwx {
            DeviceType::Gm => Ok(k.len()), 
            DeviceType::Bbg => Err(VfsError::Bz),
            DeviceType::Alu => Err(VfsError::Bz),
            DeviceType::Rw => {
                
                for &o in k {
                    crate::serial_print!("{}", o as char);
                }
                Ok(k.len())
            }
            DeviceType::Zi => {
                if !crate::virtio_blk::ky() {
                    return Err(VfsError::Av);
                }
                let zn = 512u64;
                let awy = l / zn;
                let bho = (l % zn) as usize;
                let mut fxe = 0usize;
                let mut gru = awy;
                let mut u = bho;
                let mut aae = [0u8; 512];
                while fxe < k.len() {
                    
                    if u != 0 || (k.len() - fxe) < 512 {
                        let _ = crate::virtio_blk::xr(gru, &mut aae);
                    }
                    let apk = 512 - u;
                    let acq = core::cmp::v(apk, k.len() - fxe);
                    aae[u..u + acq].dg(&k[fxe..fxe + acq]);
                    if crate::virtio_blk::aby(gru, &aae).is_err() {
                        break;
                    }
                    fxe += acq;
                    gru += 1;
                    u = 0;
                }
                Ok(fxe)
            }
        }
    }
    
    fn hm(&self) -> B<Stat> {
        let kd = match self.cwx {
            DeviceType::Zi => FileType::Bj,
            _ => FileType::Mv,
        };
        
        Ok(Stat {
            dd: self.dd,
            kd,
            aw: 0,
            xk: 0,
            py: 512,
            ev: 0o666,
            ..Default::default()
        })
    }
}


struct Aba {
    j: String,
    cwx: DeviceType,
    dd: I,
}


struct Bem {
    ik: Vec<Aba>,
}

impl Ep for Bem {
    fn cga(&self, j: &str) -> B<I> {
        for ba in &self.ik {
            if ba.j == j {
                return Ok(ba.dd);
            }
        }
        Err(VfsError::N)
    }
    
    fn brx(&self) -> B<Vec<Br>> {
        let mut ch = vec![
            Br { j: String::from("."), dd: 1, kd: FileType::K },
            Br { j: String::from(".."), dd: 1, kd: FileType::K },
        ];
        
        for ba in &self.ik {
            ch.push(Br {
                j: ba.j.clone(),
                dd: ba.dd,
                kd: match ba.cwx {
                    DeviceType::Zi => FileType::Bj,
                    _ => FileType::Mv,
                },
            });
        }
        
        Ok(ch)
    }
    
    fn avp(&self, blu: &str, gxf: FileType) -> B<I> {
        Err(VfsError::Bz) 
    }
    
    fn cnm(&self, blu: &str) -> B<()> {
        Err(VfsError::Bz)
    }
    
    fn hm(&self) -> B<Stat> {
        Ok(Stat {
            dd: 1,
            kd: FileType::K,
            aw: 0,
            xk: 0,
            py: 512,
            ev: 0o755,
            ..Default::default()
        })
    }
}


pub struct DevFs {
    ik: Vec<Aba>,
    hsv: AtomicU64,
}

impl DevFs {
    pub fn new() -> B<Self> {
        let mut fs = Self {
            ik: Vec::new(),
            hsv: AtomicU64::new(2), 
        };
        
        
        fs.fcm("null", DeviceType::Gm);
        fs.fcm("zero", DeviceType::Bbg);
        fs.fcm("random", DeviceType::Alu);
        fs.fcm("urandom", DeviceType::Alu);
        fs.fcm("console", DeviceType::Rw);
        fs.fcm("tty", DeviceType::Rw);
        
        
        if crate::virtio_blk::ky() {
            fs.fcm("vda", DeviceType::Zi);
        }
        
        Ok(fs)
    }
    
    fn fcm(&mut self, j: &str, cwx: DeviceType) {
        let dd = self.hsv.fetch_add(1, Ordering::SeqCst);
        self.ik.push(Aba {
            j: String::from(j),
            cwx,
            dd,
        });
    }
    
    fn nuf(&self, dd: I) -> Option<&Aba> {
        self.ik.iter().du(|bc| bc.dd == dd)
    }
}

impl Cc for DevFs {
    fn j(&self) -> &str {
        "devfs"
    }
    
    fn cbm(&self) -> I {
        1
    }
    
    fn era(&self, dd: I) -> B<Arc<dyn Et>> {
        let ba = self.nuf(dd).ok_or(VfsError::N)?;
        Ok(Arc::new(Ben {
            cwx: ba.cwx,
            dd: ba.dd,
        }))
    }
    
    fn dhl(&self, dd: I) -> B<Arc<dyn Ep>> {
        if dd == 1 {
            Ok(Arc::new(Bem {
                ik: self.ik.clone(),
            }))
        } else {
            Err(VfsError::Lz)
        }
    }
    
    fn hm(&self, dd: I) -> B<Stat> {
        if dd == 1 {
            Ok(Stat {
                dd: 1,
                kd: FileType::K,
                ev: 0o755,
                ..Default::default()
            })
        } else if let Some(ba) = self.nuf(dd) {
            Ok(Stat {
                dd: ba.dd,
                kd: match ba.cwx {
                    DeviceType::Zi => FileType::Bj,
                    _ => FileType::Mv,
                },
                ev: 0o666,
                ..Default::default()
            })
        } else {
            Err(VfsError::N)
        }
    }
}

impl Clone for Aba {
    fn clone(&self) -> Self {
        Self {
            j: self.j.clone(),
            cwx: self.cwx,
            dd: self.dd,
        }
    }
}
