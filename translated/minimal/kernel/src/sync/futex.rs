




use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU32, Ordering};
use spin::Mutex;


static KE_: Mutex<BTreeMap<u64, Vec<Bwn>>> = Mutex::new(BTreeMap::new());


#[derive(Debug, Clone)]
struct Bwn {
    
    ni: u64,
    
    qy: u32,
    
    wtf: u64,
    
    gun: u64,
}


pub mod op {
    pub const ACN_: u32 = 0;
    pub const ACO_: u32 = 1;
    pub const DMH_: u32 = 2;
    pub const BVT_: u32 = 3;
    pub const BVS_: u32 = 4;
    pub const DML_: u32 = 5;
    pub const DMI_: u32 = 6;
    pub const DMK_: u32 = 7;
    pub const DMJ_: u32 = 8;
    pub const BVU_: u32 = 9;
    pub const BVV_: u32 = 10;
    
    pub const ACM_: u32 = 128;
    pub const BVQ_: u32 = 256;
    
    pub const BVR_: u32 = !(ACM_ | BVQ_);
}










pub fn futex(
    aqp: u64,
    szm: u32,
    ap: u32,
    aah: u64,
    gvk: u64,
    jvf: u32,
) -> Result<i64, i32> {
    let cmd = szm & op::BVR_;
    
    match cmd {
        op::ACN_ => nwu(aqp, ap, aah),
        op::ACO_ => nwv(aqp, ap),
        op::BVT_ => nwt(aqp, ap, gvk, jvf),
        op::BVS_ => szl(aqp, ap, gvk, jvf, aah as u32),
        op::BVU_ => szn(aqp, ap, aah, jvf),
        op::BVV_ => szo(aqp, ap, jvf),
        _ => Err(-38), 
    }
}


fn nwu(aqp: u64, qy: u32, gun: u64) -> Result<i64, i32> {
    
    if !crate::memory::aov(aqp) && aqp != 0 {
        
        if aqp < 0xFFFF_8000_0000_0000 {
            return Err(-14); 
        }
    }
    
    
    let cv = unsafe { 
        let ptr = aqp as *const AtomicU32;
        (*ptr).load(Ordering::SeqCst)
    };
    
    
    if cv != qy {
        return Err(-11); 
    }
    
    
    let ni = crate::thread::bqd();
    let iu = crate::time::evk();
    
    
    {
        let mut bwj = KE_.lock();
        let queue = bwj.bt(aqp).clq(Vec::new);
        queue.push(Bwn {
            ni,
            qy,
            wtf: iu,
            gun,
        });
    }
    
    
    
    
    if gun > 0 {
        let ean = iu.akq(gun);
        crate::thread::eyp(ean);
    } else {
        
        crate::thread::mzq();
    }
    
    
    
    {
        let mut bwj = KE_.lock();
        if let Some(queue) = bwj.ds(&aqp) {
            if queue.iter().any(|aa| aa.ni == ni) {
                
                queue.ajm(|aa| aa.ni != ni);
                if queue.is_empty() {
                    bwj.remove(&aqp);
                }
                return Err(-110); 
            }
        }
    }
    
    
    Ok(0)
}


fn nwv(aqp: u64, az: u32) -> Result<i64, i32> {
    let mut bwj = KE_.lock();
    
    let fyv = if let Some(queue) = bwj.ds(&aqp) {
        let mlm = (az as usize).v(queue.len());
        
        
        let fyv: Vec<_> = queue.bbk(..mlm).collect();
        
        
        for bt in &fyv {
            crate::thread::wake(bt.ni);
        }
        
        if queue.is_empty() {
            bwj.remove(&aqp);
        }
        
        fyv.len() as i64
    } else {
        0
    };
    
    Ok(fyv)
}


fn nwt(aqp: u64, mqf: u32, gvk: u64, lzl: u32) -> Result<i64, i32> {
    let mut bwj = KE_.lock();
    
    let mut pvj = 0i64;
    
    if let Some(queue) = bwj.ds(&aqp) {
        
        let mlm = (mqf as usize).v(queue.len());
        let fyv: Vec<_> = queue.bbk(..mlm).collect();
        
        for bt in &fyv {
            crate::thread::wake(bt.ni);
        }
        pvj = fyv.len() as i64;
        
        
        let xin = (lzl as usize).v(queue.len());
        let vxn: Vec<_> = queue.bbk(..xin).collect();
        
        
        let vpi = bwj.bt(gvk).clq(Vec::new);
        vpi.lg(vxn);
    }
    
    
    if bwj.get(&aqp).map(|fm| fm.is_empty()).unwrap_or(false) {
        bwj.remove(&aqp);
    }
    
    Ok(pvj)
}


fn szl(aqp: u64, mqf: u32, gvk: u64, lzl: u32, qy: u32) -> Result<i64, i32> {
    
    let cv = unsafe { 
        let ptr = aqp as *const AtomicU32;
        (*ptr).load(Ordering::SeqCst)
    };
    if cv != qy {
        return Err(-11); 
    }
    
    nwt(aqp, mqf, gvk, lzl)
}


fn szn(aqp: u64, qy: u32, gun: u64, kdm: u32) -> Result<i64, i32> {
    if kdm == 0 {
        return Err(-22); 
    }
    
    
    
    nwu(aqp, qy, gun)
}


fn szo(aqp: u64, az: u32, kdm: u32) -> Result<i64, i32> {
    if kdm == 0 {
        return Err(-22); 
    }
    
    
    nwv(aqp, az)
}


pub fn yui(aqp: u64) -> usize {
    KE_.lock()
        .get(&aqp)
        .map(|fm| fm.len())
        .unwrap_or(0)
}


pub fn khu(ce: u64) {
    let mut bwj = KE_.lock();
    
    
    for queue in bwj.xqp() {
        queue.ajm(|aa| (aa.ni >> 32) != ce);
    }
    
    
    bwj.ajm(|_, fm| !fm.is_empty());
}
