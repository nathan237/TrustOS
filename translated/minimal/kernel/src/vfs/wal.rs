








use spin::Mutex;

const H_: usize = 512;
const BLC_: u32 = 0x57414C21; 
const WL_: usize = 63;




const ZQ_: u64 = 33;
const BLB_: u64 = 34; 


#[repr(C)]
#[derive(Clone, Copy)]
struct Agp {
    magic: u32,
    entry_count: u32,
    committed: u32, 
    sequence: u64,   
    _pad: [u8; H_ - 20],
}


#[repr(C)]
#[derive(Clone, Copy)]
struct Bfm {
    target_sector: u64,        
    data: [u8; H_ - 8], 
}


pub struct WriteAheadLog {
    pending: [(u64, [u8; H_]); WL_],
    count: usize,
    sequence: u64,
    active: bool,
}

impl WriteAheadLog {
    pub const fn new() -> Self {
        Self {
            pending: [(0, [0u8; H_]); WL_],
            count: 0,
            sequence: 0,
            active: false,
        }
    }

    
    pub fn begin(&mut self) {
        self.count = 0;
        self.active = true;
    }

    
    pub fn log_write(&mut self, dj: u64, data: &[u8; H_]) -> Result<(), ()> {
        if !self.active || self.count >= WL_ {
            return Err(());
        }
        self.pending[self.count] = (dj, *data);
        self.count += 1;
        Ok(())
    }

    
    pub fn commit(
        &mut self,
        write_sector: &dyn Fn(u64, &[u8; H_]) -> Result<(), ()>,
    ) -> Result<(), ()> {
        if self.count == 0 {
            self.active = false;
            return Ok(());
        }

        self.sequence += 1;

        
        let mut drg = [0u8; H_];
        let kp = unsafe { &mut *(drg.as_mut_ptr() as *mut Agp) };
        kp.magic = BLC_;
        kp.entry_count = self.count as u32;
        kp.committed = 1;
        kp.sequence = self.sequence;
        write_sector(ZQ_, &drg)?;

        
        for i in 0..self.count {
            let (target, ref data) = self.pending[i];
            
            let mut ciz = [0u8; H_];
            ciz[0..8].copy_from_slice(&target.to_le_bytes());
            let mb = core::cmp::min(data.len(), H_ - 8);
            ciz[8..8 + mb].copy_from_slice(&data[..mb]);
            write_sector(BLB_ + i as u64, &ciz)?;
        }

        
        for i in 0..self.count {
            let (target, ref data) = self.pending[i];
            write_sector(target, data)?;
        }

        
        let zero = [0u8; H_];
        write_sector(ZQ_, &zero)?;

        self.count = 0;
        self.active = false;
        Ok(())
    }

    
    pub fn pending_count(&self) -> usize {
        self.count
    }
}


static Vy: Mutex<WriteAheadLog> = Mutex::new(WriteAheadLog::new());


pub fn ofx(
    read_sector: &dyn Fn(u64, &mut [u8; H_]) -> Result<(), ()>,
    write_sector: &dyn Fn(u64, &[u8; H_]) -> Result<(), ()>,
) -> Result<usize, ()> {
    let mut drg = [0u8; H_];
    read_sector(ZQ_, &mut drg)?;

    let kp = unsafe { &*(drg.as_ptr() as *const Agp) };
    if kp.magic != BLC_ || kp.committed != 1 || kp.entry_count == 0 {
        return Ok(0); 
    }

    let count = kp.entry_count as usize;
    crate::log!("[WAL] Replaying {} pending writes from sequence {}", count, kp.sequence);

    for i in 0..count.min(WL_) {
        let mut ciz = [0u8; H_];
        read_sector(BLB_ + i as u64, &mut ciz)?;

        let target = u64::from_le_bytes(ciz[0..8].try_into().unwrap_or([0; 8]));
        
        let mut data = [0u8; H_];
        let mb = core::cmp::min(H_ - 8, H_);
        data[..mb].copy_from_slice(&ciz[8..8 + mb]);
        
        write_sector(target, &data)?;
    }

    
    let zero = [0u8; H_];
    write_sector(ZQ_, &zero)?;

    crate::log!("[WAL] Replay complete — {} writes applied", count);
    Ok(count)
}


pub fn begin() {
    Vy.lock().begin();
}


pub fn log_write(dj: u64, data: &[u8; H_]) -> Result<(), ()> {
    Vy.lock().log_write(dj, data)
}


pub fn commit(
    write_sector: &dyn Fn(u64, &[u8; H_]) -> Result<(), ()>,
) -> Result<(), ()> {
    Vy.lock().commit(write_sector)
}
