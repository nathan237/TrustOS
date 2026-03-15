




use alloc::collections::BTreeMap;
use spin::Mutex;

const BLT_: usize = 256; 
const H_: usize = 512;

struct Bdh {
    f: [u8; H_],
    no: bool,
    vz: u64,     
}

pub struct BlockCache {
    ch: BTreeMap<u64, Bdh>,
    va: u64,
    dld: fn(u64, &mut [u8; H_]) -> Result<(), ()>,
    eld: fn(u64, &[u8; H_]) -> Result<(), ()>,
}

impl BlockCache {
    pub fn new(
        dld: fn(u64, &mut [u8; H_]) -> Result<(), ()>,
        eld: fn(u64, &[u8; H_]) -> Result<(), ()>,
    ) -> Self {
        Self {
            ch: BTreeMap::new(),
            va: 0,
            dld,
            eld,
        }
    }

    
    pub fn read(&mut self, jk: u64, k: &mut [u8; H_]) -> Result<(), ()> {
        self.va += 1;
        if let Some(bt) = self.ch.ds(&jk) {
            bt.vz = self.va;
            k.dg(&bt.f);
            return Ok(());
        }
        
        (self.dld)(jk, k)?;
        self.insert(jk, *k, false);
        Ok(())
    }

    
    pub fn write(&mut self, jk: u64, k: &[u8; H_]) -> Result<(), ()> {
        self.va += 1;
        if let Some(bt) = self.ch.ds(&jk) {
            bt.f.dg(k);
            bt.no = true;
            bt.vz = self.va;
            return Ok(());
        }
        self.insert(jk, *k, true);
        Ok(())
    }

    
    fn insert(&mut self, jk: u64, f: [u8; H_], no: bool) {
        if self.ch.len() >= BLT_ {
            self.snv();
        }
        self.ch.insert(jk, Bdh {
            f,
            no,
            vz: self.va,
        });
    }

    
    fn snv(&mut self) {
        let uiq = self.ch.iter()
            .zct(|(_, aa)| aa.vz)
            .map(|(&e, _)| e);
        if let Some(jk) = uiq {
            if let Some(bt) = self.ch.remove(&jk) {
                if bt.no {
                    let _ = (self.eld)(jk, &bt.f);
                }
            }
        }
    }

    
    pub fn sync(&mut self) -> Result<(), ()> {
        for (&jk, bt) in self.ch.el() {
            if bt.no {
                (self.eld)(jk, &bt.f)?;
                bt.no = false;
            }
        }
        Ok(())
    }

    
    pub fn yyo(&mut self, jk: u64) {
        if let Some(bt) = self.ch.remove(&jk) {
            if bt.no {
                let _ = (self.eld)(jk, &bt.f);
            }
        }
    }

    
    pub fn qvh(&self) -> usize {
        self.ch.len()
    }

    
    pub fn rxv(&self) -> usize {
        self.ch.alv().hi(|aa| aa.no).az()
    }
}


static LX_: Mutex<Option<BlockCache>> = Mutex::new(None);


pub fn init(
    dld: fn(u64, &mut [u8; H_]) -> Result<(), ()>,
    eld: fn(u64, &[u8; H_]) -> Result<(), ()>,
) {
    *LX_.lock() = Some(BlockCache::new(dld, eld));
}


pub fn qvi(jk: u64, k: &mut [u8; H_]) -> Result<(), ()> {
    if let Some(bdq) = LX_.lock().as_mut() {
        bdq.read(jk, k)
    } else {
        Err(()) 
    }
}


pub fn qvj(jk: u64, k: &[u8; H_]) -> Result<(), ()> {
    if let Some(bdq) = LX_.lock().as_mut() {
        bdq.write(jk, k)
    } else {
        Err(())
    }
}


pub fn sync() -> Result<(), ()> {
    if let Some(bdq) = LX_.lock().as_mut() {
        bdq.sync()
    } else {
        Ok(())
    }
}


pub fn cm() -> (usize, usize) {
    if let Some(bdq) = LX_.lock().as_ref() {
        (bdq.qvh(), bdq.rxv())
    } else {
        (0, 0)
    }
}
