








use spin::Mutex;

const H_: usize = 512;
const BIW_: u32 = 0x57414C21; 
const VC_: usize = 63;




const YN_: u64 = 33;
const BIV_: u64 = 34; 


#[repr(C)]
#[derive(Clone, Copy)]
struct Bwp {
    sj: u32,
    ame: u32,
    gda: u32, 
    eil: u64,   
    fzo: [u8; H_ - 20],
}


#[repr(C)]
#[derive(Clone, Copy)]
struct Dly {
    zrb: u64,        
    f: [u8; H_ - 8], 
}


pub struct WriteAheadLog {
    aln: [(u64, [u8; H_]); VC_],
    az: usize,
    eil: u64,
    gh: bool,
}

impl WriteAheadLog {
    pub const fn new() -> Self {
        Self {
            aln: [(0, [0u8; H_]); VC_],
            az: 0,
            eil: 0,
            gh: false,
        }
    }

    
    pub fn myo(&mut self) {
        self.az = 0;
        self.gh = true;
    }

    
    pub fn ljr(&mut self, jk: u64, f: &[u8; H_]) -> Result<(), ()> {
        if !self.gh || self.az >= VC_ {
            return Err(());
        }
        self.aln[self.az] = (jk, *f);
        self.az += 1;
        Ok(())
    }

    
    pub fn dfc(
        &mut self,
        aby: &dyn Fn(u64, &[u8; H_]) -> Result<(), ()>,
    ) -> Result<(), ()> {
        if self.az == 0 {
            self.gh = false;
            return Ok(());
        }

        self.eil += 1;

        
        let mut hmr = [0u8; H_];
        let zj = unsafe { &mut *(hmr.mw() as *mut Bwp) };
        zj.sj = BIW_;
        zj.ame = self.az as u32;
        zj.gda = 1;
        zj.eil = self.eil;
        aby(YN_, &hmr)?;

        
        for a in 0..self.az {
            let (cd, ref f) = self.aln[a];
            
            let mut fhv = [0u8; H_];
            fhv[0..8].dg(&cd.ho());
            let zg = core::cmp::v(f.len(), H_ - 8);
            fhv[8..8 + zg].dg(&f[..zg]);
            aby(BIV_ + a as u64, &fhv)?;
        }

        
        for a in 0..self.az {
            let (cd, ref f) = self.aln[a];
            aby(cd, f)?;
        }

        
        let ajs = [0u8; H_];
        aby(YN_, &ajs)?;

        self.az = 0;
        self.gh = false;
        Ok(())
    }

    
    pub fn ewn(&self) -> usize {
        self.az
    }
}


static Baq: Mutex<WriteAheadLog> = Mutex::new(WriteAheadLog::new());


pub fn vxh(
    xr: &dyn Fn(u64, &mut [u8; H_]) -> Result<(), ()>,
    aby: &dyn Fn(u64, &[u8; H_]) -> Result<(), ()>,
) -> Result<usize, ()> {
    let mut hmr = [0u8; H_];
    xr(YN_, &mut hmr)?;

    let zj = unsafe { &*(hmr.fq() as *const Bwp) };
    if zj.sj != BIW_ || zj.gda != 1 || zj.ame == 0 {
        return Ok(0); 
    }

    let az = zj.ame as usize;
    crate::log!("[WAL] Replaying {} pending writes from sequence {}", az, zj.eil);

    for a in 0..az.v(VC_) {
        let mut fhv = [0u8; H_];
        xr(BIV_ + a as u64, &mut fhv)?;

        let cd = u64::dj(fhv[0..8].try_into().unwrap_or([0; 8]));
        
        let mut f = [0u8; H_];
        let zg = core::cmp::v(H_ - 8, H_);
        f[..zg].dg(&fhv[8..8 + zg]);
        
        aby(cd, &f)?;
    }

    
    let ajs = [0u8; H_];
    aby(YN_, &ajs)?;

    crate::log!("[WAL] Replay complete — {} writes applied", az);
    Ok(az)
}


pub fn myo() {
    Baq.lock().myo();
}


pub fn ljr(jk: u64, f: &[u8; H_]) -> Result<(), ()> {
    Baq.lock().ljr(jk, f)
}


pub fn dfc(
    aby: &dyn Fn(u64, &[u8; H_]) -> Result<(), ()>,
) -> Result<(), ()> {
    Baq.lock().dfc(aby)
}
