


































use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use spin::Mutex;

use super::{wr, sk, Sr};
use super::regs;
use crate::memory;







const DM_: usize = 4096;
const AHG_: usize = DM_ * 4;

const COR_: u32 = 12; 



const HE_: usize = 256 * 1024;


const CUF_: usize = 4096;


const ARW_: usize = 0x00;
const ARX_: usize = 0x10;

const CPD_: usize = 0x100;
const CPE_: usize = 0x110;


const BFI_: u64 = 10_000_000;


static AIX_: AtomicU64 = AtomicU64::new(0);
static AJA_: AtomicU64 = AtomicU64::new(0);













#[inline]
fn grq(op: u32, wvn: u32) -> u32 {
    ((wvn & 0x3) << 26) | ((op & 0x3FFFF) << 8)
}


#[inline]
fn zlx() -> u32 {
    grq(regs::CRQ_, 0)
}














fn wfi(cbz: u64, dgu: u64, aal: u32) -> [u32; 7] {
    [
        grq(regs::CRO_, regs::CRF_),
        aal,
        0, 
        (cbz & 0xFFFFFFFF) as u32,
        ((cbz >> 32) & 0xFFFFFFFF) as u32,
        (dgu & 0xFFFFFFFF) as u32,
        ((dgu >> 32) & 0xFFFFFFFF) as u32,
    ]
}












fn wfh(dgu: u64, eqg: u32, aal: u32) -> [u32; 5] {
    [
        grq(regs::CRN_, 0),
        (dgu & 0xFFFFFFFF) as u32,
        ((dgu >> 32) & 0xFFFFFFFF) as u32,
        eqg,
        aal,
    ]
}











fn pgx(ag: u64, bn: u32) -> [u32; 4] {
    [
        grq(regs::CRP_, 0),
        (ag & 0xFFFFFFFF) as u32,
        ((ag >> 32) & 0xFFFFFFFF) as u32,
        bn,
    ]
}










fn zly(ag: u64) -> [u32; 3] {
    [
        grq(regs::CRR_, 0),
        (ag & 0xFFFFFFFF) as u32,
        ((ag >> 32) & 0xFFFFFFFF) as u32,
    ]
}












fn zlz(dgu: u64, az: u32) -> [u32; 4] {
    [
        grq(regs::CRS_, regs::CRX_),
        (dgu & 0xFFFFFFFF) as u32,
        ((dgu >> 32) & 0xFFFFFFFF) as u32,
        az.ao(1),
    ]
}






struct Yr {
    
    index: usize,
    
    hv: u64,
    
    cbi: u32,
    
    dlh: u64,
    
    bhy: u64,
    
    ccn: u32,
    
    cxu: u32,
    
    faf: u64,
    
    bf: u64,
}


struct Bsi {
    jr: bool,
    hv: u64,
    
    ggf: [Option<Yr>; 2],
    
    dci: u64,
    bik: u64,
    
    dcf: u64,
    cie: u64,
}

static IX_: Mutex<Bsi> = Mutex::new(Bsi {
    jr: false,
    hv: 0,
    ggf: [None, None],
    dci: 0,
    bik: 0,
    dcf: 0,
    cie: 0,
});

static HD_: AtomicBool = AtomicBool::new(false);






fn cbk(engine: &mut Yr, f: &[u32]) {
    let mz = engine.dlh as *mut u32;
    for (a, &aix) in f.iter().cf() {
        let w = (engine.ccn as usize + a) % DM_;
        unsafe {
            core::ptr::write_volatile(mz.add(w), aix);
        }
    }
    engine.ccn = ((engine.ccn as usize + f.len()) % DM_) as u32;
}


fn jmn(engine: &Yr) {
    unsafe {
        
        let mqx = engine.ccn * 4;
        let pzu = engine.cbi + regs::BFH_;
        sk(engine.hv, pzu, mqx);
        sk(engine.hv, pzu + 4, 0); 
    }
}


fn zkf(engine: &Yr) -> u32 {
    unsafe {
        let was = engine.cbi + regs::BFG_;
        let war = wr(engine.hv, was);
        war / 4
    }
}






fn oej(
    hv: u64,
    cek: usize,
    dlh: u64,
    bhy: u64,
    bik: u64,
    wat: usize,
) -> Option<Yr> {
    let ar = if cek == 0 {
        regs::BFC_
    } else {
        regs::BFE_
    };

    crate::log!("[SDMA{}] Initializing engine (reg_base={:#X})", cek, ar);

    unsafe {
        
        let pon = if cek == 0 {
            regs::CQY_
        } else {
            regs::CRC_
        };
        let status = wr(hv, pon);
        crate::log!("[SDMA{}] STATUS={:#010X} (idle={})",
            cek, status, (status & regs::CRW_) != 0);

        
        let nsr = if cek == 0 {
            regs::BFD_
        } else {
            regs::BFF_
        };
        sk(hv, nsr, 1); 

        
        for _ in 0..1000 {
            core::hint::hc();
        }

        
        let ozt = ar + regs::CRI_;
        sk(hv, ozt, 0); 

        
        let hwt = bhy >> 8;
        let vqp = ar + regs::CRG_;
        let vqo = ar + regs::CRH_;
        sk(hv, vqp, (hwt & 0xFFFFFFFF) as u32);
        sk(hv, vqo, ((hwt >> 32) & 0xFFFFFFFF) as u32);

        
        sk(hv, ar + regs::BFG_, 0);
        sk(hv, ar + regs::CRL_, 0);
        sk(hv, ar + regs::BFH_, 0);
        sk(hv, ar + regs::CRM_, 0);

        
        let pei = bik + wat as u64;
        sk(hv, ar + regs::CRK_,
            (pei & 0xFFFFFFFF) as u32);
        sk(hv, ar + regs::CRJ_,
            ((pei >> 32) & 0xFFFFFFFF) as u32);

        
        
        
        
        
        let vqq = regs::CRT_
            | (COR_ << regs::CRU_)
            | regs::CRV_;
        sk(hv, ozt, vqq);

        
        sk(hv, nsr, 0); 

        
        for _ in 0..10000 {
            core::hint::hc();
        }
        let wtr = wr(hv, pon);
        crate::log!("[SDMA{}] Post-init STATUS={:#010X}", cek, wtr);
    }

    Some(Yr {
        index: cek,
        hv,
        cbi: ar,
        dlh,
        bhy,
        ccn: 0,
        cxu: 1,
        faf: 0,
        bf: 0,
    })
}







pub fn init(hv: u64) {
    crate::log!("[SDMA] ═══════════════════════════════════════════════════");
    crate::log!("[SDMA] SDMA Engine — Bare-metal DMA transfers (Navi 10)");
    crate::log!("[SDMA] ═══════════════════════════════════════════════════");

    if hv == 0 {
        crate::log!("[SDMA] No MMIO base — skipping");
        return;
    }

    
    let wfj = unsafe { wr(hv, regs::CRB_) };
    crate::log!("[SDMA] SDMA0 VERSION={:#010X}", wfj);

    
    let jmm = alloc::alloc::Layout::bjy(AHG_, 4096)
        .expect("sdma ring layout");

    let mae = unsafe { alloc::alloc::alloc_zeroed(jmm) } as u64;
    let mad = memory::abw(mae).unwrap_or(0);

    let mag = unsafe { alloc::alloc::alloc_zeroed(jmm) } as u64;
    let maf = memory::abw(mag).unwrap_or(0);

    if mad == 0 || maf == 0 {
        crate::log!("[SDMA] ERROR: Cannot get physical address for ring buffers");
        return;
    }

    crate::log!("[SDMA] Ring0: virt={:#X} phys={:#X} ({} dwords)",
        mae, mad, DM_);
    crate::log!("[SDMA] Ring1: virt={:#X} phys={:#X} ({} dwords)",
        mag, maf, DM_);

    
    let wtu = alloc::alloc::Layout::bjy(CUF_, 4096)
        .expect("sdma status layout");
    let dci = unsafe { alloc::alloc::alloc_zeroed(wtu) } as u64;
    let bik = memory::abw(dci).unwrap_or(0);

    if bik == 0 {
        crate::log!("[SDMA] ERROR: Cannot get phys for status buffer");
        return;
    }
    crate::log!("[SDMA] Status: virt={:#X} phys={:#X}", dci, bik);

    
    let wsf = alloc::alloc::Layout::bjy(HE_, 4096)
        .expect("sdma staging layout");
    let dcf = unsafe { alloc::alloc::alloc_zeroed(wsf) } as u64;
    let cie = memory::abw(dcf).unwrap_or(0);

    if cie == 0 {
        crate::log!("[SDMA] ERROR: Cannot get phys for staging buffer");
        return;
    }
    crate::log!("[SDMA] Staging: virt={:#X} phys={:#X} ({}KB)",
        dcf, cie, HE_ / 1024);

    
    let nqf = oej(
        hv, 0, mae, mad, bik, CPD_,
    );
    let nqg = oej(
        hv, 1, mag, maf, bik, CPE_,
    );

    let sia = nqf.is_some();
    let sib = nqg.is_some();

    
    let mut g = IX_.lock();
    g.jr = true;
    g.hv = hv;
    g.ggf[0] = nqf;
    g.ggf[1] = nqg;
    g.dci = dci;
    g.bik = bik;
    g.dcf = dcf;
    g.cie = cie;
    drop(g);

    HD_.store(true, Ordering::SeqCst);

    crate::log!("[SDMA] ───────────────────────────────────────────────────");
    crate::log!("[SDMA] Engine 0: {}", if sia { "READY" } else { "FAILED" });
    crate::log!("[SDMA] Engine 1: {}", if sib { "READY" } else { "FAILED" });
    crate::log!("[SDMA] Staging: {}KB for CPU→GPU transfers", HE_ / 1024);
    crate::log!("[SDMA] Commands: sdma copy|fill|test|bench|info");
    crate::log!("[SDMA] ───────────────────────────────────────────────────");
}











pub fn bdu(jrh: u64, fhc: u64, aal: u32, cek: usize) -> Result<u32, &'static str> {
    if !HD_.load(Ordering::Relaxed) {
        return Err("SDMA not initialized");
    }
    if aal == 0 {
        return Ok(0);
    }
    
    if aal > (1 << 26) {
        return Err("Transfer too large (max 64MB per SDMA packet)");
    }
    let cei = cek.v(1);

    let mut g = IX_.lock();
    let bik = g.bik;
    let dci = g.dci;
    let engine = g.ggf[cei].as_mut().ok_or("SDMA engine not ready")?;

    
    let dhd = engine.cxu;
    engine.cxu = engine.cxu.cn(1);
    if engine.cxu == 0 { engine.cxu = 1; }

    
    let hjc = if cei == 0 { ARW_ } else { ARX_ };
    let nte = dci + hjc as u64;
    let kvl = bik + hjc as u64;
    unsafe {
        core::ptr::write_volatile(nte as *mut u32, 0);
    }

    
    let rot = wfi(jrh, fhc, aal);
    let kvm = pgx(kvl, dhd);

    cbk(engine, &rot);
    cbk(engine, &kvm);

    
    jmn(engine);

    crate::serial_println!("[SDMA{}] COPY: {:#X} → {:#X} ({} bytes) fence={}",
        cei, jrh, fhc, aal, dhd);

    
    let mut ez = 0u64;
    loop {
        let cv = unsafe { core::ptr::read_volatile(nte as *const u32) };
        if cv == dhd {
            break;
        }
        ez += 1;
        if ez >= BFI_ {
            crate::serial_println!("[SDMA{}] TIMEOUT: fence expected {} got {}",
                cei, dhd, cv);
            return Err("SDMA copy timed out");
        }
        if ez % 100 == 0 {
            core::hint::hc();
        }
    }

    
    engine.faf += 1;
    engine.bf += aal as u64;
    drop(g);

    AIX_.fetch_add(aal as u64, Ordering::Relaxed);
    AJA_.fetch_add(1, Ordering::Relaxed);

    crate::serial_println!("[SDMA{}] Copy complete in {} iters", cei, ez);
    Ok(dhd)
}






pub fn vi(fhc: u64, eqg: u32, aal: u32, cek: usize) -> Result<u32, &'static str> {
    if !HD_.load(Ordering::Relaxed) {
        return Err("SDMA not initialized");
    }
    if aal == 0 {
        return Ok(0);
    }
    if aal & 3 != 0 {
        return Err("byte_count must be a multiple of 4");
    }
    if aal > (1 << 26) {
        return Err("Fill too large (max 64MB per SDMA packet)");
    }
    let cei = cek.v(1);

    let mut g = IX_.lock();
    let bik = g.bik;
    let dci = g.dci;
    let engine = g.ggf[cei].as_mut().ok_or("SDMA engine not ready")?;

    let dhd = engine.cxu;
    engine.cxu = engine.cxu.cn(1);
    if engine.cxu == 0 { engine.cxu = 1; }

    let hjc = if cei == 0 { ARW_ } else { ARX_ };
    let ntf = dci + hjc as u64;
    let kvl = bik + hjc as u64;
    unsafe {
        core::ptr::write_volatile(ntf as *mut u32, 0);
    }

    let ssk = wfh(fhc, eqg, aal);
    let kvm = pgx(kvl, dhd);

    cbk(engine, &ssk);
    cbk(engine, &kvm);
    jmn(engine);

    crate::serial_println!("[SDMA{}] FILL: {:#X} = {:#010X} x{} bytes, fence={}",
        cei, fhc, eqg, aal, dhd);

    let mut ez = 0u64;
    loop {
        let cv = unsafe { core::ptr::read_volatile(ntf as *const u32) };
        if cv == dhd {
            break;
        }
        ez += 1;
        if ez >= BFI_ {
            crate::serial_println!("[SDMA{}] TIMEOUT: fence expected {} got {}",
                cei, dhd, cv);
            return Err("SDMA fill timed out");
        }
        if ez % 100 == 0 {
            core::hint::hc();
        }
    }

    engine.faf += 1;
    engine.bf += aal as u64;
    drop(g);

    AIX_.fetch_add(aal as u64, Ordering::Relaxed);
    AJA_.fetch_add(1, Ordering::Relaxed);

    crate::serial_println!("[SDMA{}] Fill complete in {} iters", cei, ez);
    Ok(dhd)
}










pub fn mof(f: &[u8], fhc: u64, cek: usize) -> Result<usize, &'static str> {
    if !HD_.load(Ordering::Relaxed) {
        return Err("SDMA not initialized");
    }
    if f.is_empty() {
        return Ok(0);
    }

    
    let mut l = 0usize;
    while l < f.len() {
        let jj = (f.len() - l).v(HE_);
        
        let gye = (jj + 3) & !3;

        let g = IX_.lock();
        let dcf = g.dcf;
        let cie = g.cie;
        drop(g);

        
        unsafe {
            let cs = dcf as *mut u8;
            let cy = f.fq().add(l);
            core::ptr::copy_nonoverlapping(cy, cs, jj);
            
            if gye > jj {
                core::ptr::ahx(cs.add(jj), 0, gye - jj);
            }
        }

        
        bdu(
            cie,
            fhc + l as u64,
            gye as u32,
            cek,
        )?;

        l += jj;
    }

    Ok(f.len())
}







pub fn kqp(jrh: u64, k: &mut [u8], cek: usize) -> Result<usize, &'static str> {
    if !HD_.load(Ordering::Relaxed) {
        return Err("SDMA not initialized");
    }
    if k.is_empty() {
        return Ok(0);
    }

    let mut l = 0usize;
    while l < k.len() {
        let jj = (k.len() - l).v(HE_);
        let gye = (jj + 3) & !3;

        let g = IX_.lock();
        let dcf = g.dcf;
        let cie = g.cie;
        drop(g);

        
        bdu(
            jrh + l as u64,
            cie,
            gye as u32,
            cek,
        )?;

        
        unsafe {
            let cy = dcf as *const u8;
            let cs = k.mw().add(l);
            core::ptr::copy_nonoverlapping(cy, cs, jj);
        }

        l += jj;
    }

    Ok(k.len())
}













pub fn eyj() -> (u32, u32) {
    if !HD_.load(Ordering::Relaxed) {
        return (0, 0);
    }

    let mut afu = 0u32;
    let mut ace = 0u32;

    
    let layout = alloc::alloc::Layout::bjy(4096, 4096).expect("test layout");
    let hbp = unsafe { alloc::alloc::alloc_zeroed(layout) } as u64;
    let hbq = unsafe { alloc::alloc::alloc_zeroed(layout) } as u64;
    let hbo = memory::abw(hbp).unwrap_or(0);
    let kfg = memory::abw(hbq).unwrap_or(0);

    if hbo == 0 || kfg == 0 {
        crate::serial_println!("[SDMA-TEST] FAIL: cannot allocate test buffers");
        return (0, 1);
    }

    
    crate::serial_println!("[SDMA-TEST] Test 1: CONST_FILL (engine 0, 1024 bytes, pattern=0xFACEFEED)");
    match vi(hbo, 0xFACE_FEED, 1024, 0) {
        Ok(_) => {
            
            let ptr = hbp as *const u32;
            let mut bq = true;
            for a in 0..256 {
                let ap = unsafe { core::ptr::read_volatile(ptr.add(a)) };
                if ap != 0xFACE_FEED {
                    crate::serial_println!("[SDMA-TEST]   MISMATCH at [{}]: got {:#010X}", a, ap);
                    bq = false;
                    break;
                }
            }
            if bq { afu += 1; } else { ace += 1; }
        }
        Err(aa) => {
            crate::serial_println!("[SDMA-TEST]   Error: {}", aa);
            ace += 1;
        }
    }

    
    crate::serial_println!("[SDMA-TEST] Test 2: LINEAR COPY (engine 0, 1024 bytes)");
    match bdu(hbo, kfg, 1024, 0) {
        Ok(_) => {
            let lwb = hbq as *const u32;
            let mut bq = true;
            for a in 0..256 {
                let ap = unsafe { core::ptr::read_volatile(lwb.add(a)) };
                if ap != 0xFACE_FEED {
                    crate::serial_println!("[SDMA-TEST]   MISMATCH at [{}]: got {:#010X}", a, ap);
                    bq = false;
                    break;
                }
            }
            if bq { afu += 1; } else { ace += 1; }
        }
        Err(aa) => {
            crate::serial_println!("[SDMA-TEST]   Error: {}", aa);
            ace += 1;
        }
    }

    
    crate::serial_println!("[SDMA-TEST] Test 3: CONST_FILL (engine 1, 512 bytes, pattern=0xBAAD_C0DE)");
    
    unsafe {
        core::ptr::ahx(hbq as *mut u8, 0, 4096);
    }
    match vi(kfg, 0xBAAD_C0DE, 512, 1) {
        Ok(_) => {
            let lwb = hbq as *const u32;
            let mut bq = true;
            for a in 0..128 {
                let ap = unsafe { core::ptr::read_volatile(lwb.add(a)) };
                if ap != 0xBAAD_C0DE {
                    crate::serial_println!("[SDMA-TEST]   MISMATCH at [{}]: got {:#010X}", a, ap);
                    bq = false;
                    break;
                }
            }
            if bq { afu += 1; } else { ace += 1; }
        }
        Err(aa) => {
            crate::serial_println!("[SDMA-TEST]   Error: {}", aa);
            ace += 1;
        }
    }

    
    crate::serial_println!("[SDMA-TEST] Test 4: CPU Upload via staging (256 bytes)");
    
    unsafe { core::ptr::ahx(hbp as *mut u8, 0, 4096); }
    let ezo: [u8; 256] = {
        let mut bc = [0u8; 256];
        for a in 0..256 { bc[a] = a as u8; }
        bc
    };
    match mof(&ezo, hbo, 0) {
        Ok(_) => {
            let ptr = hbp as *const u8;
            let mut bq = true;
            for a in 0..256 {
                let ap = unsafe { core::ptr::read_volatile(ptr.add(a)) };
                if ap != a as u8 {
                    crate::serial_println!("[SDMA-TEST]   MISMATCH at [{}]: got {} expected {}", a, ap, a);
                    bq = false;
                    break;
                }
            }
            if bq { afu += 1; } else { ace += 1; }
        }
        Err(aa) => {
            crate::serial_println!("[SDMA-TEST]   Error: {}", aa);
            ace += 1;
        }
    }

    
    crate::serial_println!("[SDMA-TEST] Test 5: CPU Download via staging (256 bytes)");
    let mut bky = [0u8; 256];
    match kqp(hbo, &mut bky, 0) {
        Ok(_) => {
            let mut bq = true;
            for a in 0..256 {
                if bky[a] != a as u8 {
                    crate::serial_println!("[SDMA-TEST]   MISMATCH at [{}]: got {} expected {}", a, bky[a], a);
                    bq = false;
                    break;
                }
            }
            if bq { afu += 1; } else { ace += 1; }
        }
        Err(aa) => {
            crate::serial_println!("[SDMA-TEST]   Error: {}", aa);
            ace += 1;
        }
    }

    
    unsafe {
        alloc::alloc::dealloc(hbp as *mut u8, layout);
        alloc::alloc::dealloc(hbq as *mut u8, layout);
    }

    (afu, ace)
}








pub fn qoy(gs: u32) -> Result<(u64, u64), &'static str> {
    if !HD_.load(Ordering::Relaxed) {
        return Err("SDMA not initialized");
    }
    let afz = (gs as usize * 1024).v(HE_);
    let ciz = (afz + 3) & !3;

    
    let layout = alloc::alloc::Layout::bjy(ciz, 4096)
        .jd(|_| "allocation error")?;
    let fdt = unsafe { alloc::alloc::alloc_zeroed(layout) } as u64;
    let fdu = unsafe { alloc::alloc::alloc_zeroed(layout) } as u64;
    let fqr = memory::abw(fdt).ok_or("virt_to_phys failed")?;
    let hvb = memory::abw(fdu).ok_or("virt_to_phys failed")?;

    
    let _ = vi(fqr, 0, ciz as u32, 0);

    
    let bbu = 16u32;
    let xaa = crate::time::ave();
    for _ in 0..bbu {
        vi(fqr, 0xAAAA_BBBB, ciz as u32, 0)?;
    }
    let wzx = crate::time::ave();

    
    let wzz = crate::time::ave();
    for _ in 0..bbu {
        bdu(fqr, hvb, ciz as u32, 0)?;
    }
    let wzw = crate::time::ave();

    
    unsafe {
        alloc::alloc::dealloc(fdt as *mut u8, layout);
        alloc::alloc::dealloc(fdu as *mut u8, layout);
    }

    
    let ntt = wzx.ao(xaa).am(1);
    let nfy = wzw.ao(wzz).am(1);
    

    
    let xv = ciz as u64 * bbu as u64;

    
    
    
    
    
    let kvu = if ntt > 0 { (xv * 1000) / (ntt * 1024) } else { 0 };
    let kku = if nfy > 0 { (xv * 1000) / (nfy * 1024) } else { 0 };

    Ok((kvu, kku))
}






pub fn uc() -> bool {
    HD_.load(Ordering::Relaxed)
}


pub fn xv() -> u64 {
    AIX_.load(Ordering::Relaxed)
}


pub fn pvf() -> u64 {
    AJA_.load(Ordering::Relaxed)
}


pub fn awz() -> String {
    if uc() {
        let bf = xv();
        let faf = pvf();
        let cfv = bf / 1024;
        format!("SDMA: 2 engines, {} transfers, {} KB moved", faf, cfv)
    } else {
        String::from("SDMA: not initialized")
    }
}


pub fn zl() -> Vec<String> {
    let mut ak = Vec::new();

    if uc() {
        let g = IX_.lock();
        ak.push(String::from("╔══════════════════════════════════════════════════╗"));
        ak.push(String::from("║       SDMA Engine — Bare-metal DMA Transfers     ║"));
        ak.push(String::from("╠══════════════════════════════════════════════════╣"));
        ak.push(format!("║ Status Buffer: {:#X}                      ║", g.bik));
        ak.push(format!("║ Staging:       {:#X} ({}KB)              ║",
            g.cie, HE_ / 1024));
        ak.push(format!("║ Total Bytes:   {} KB                           ║", xv() / 1024));
        ak.push(format!("║ Total Xfers:   {}                              ║", pvf()));
        ak.push(String::from("╠══════════════════════════════════════════════════╣"));

        for a in 0..2 {
            if let Some(ref engine) = g.ggf[a] {
                ak.push(format!("║ SDMA{}: ring@{:#X} wptr={} seq={} xfers={} bytes={}",
                    a, engine.bhy, engine.ccn, engine.cxu,
                    engine.faf, engine.bf));
            } else {
                ak.push(format!("║ SDMA{}: not initialized                        ║", a));
            }
        }
        ak.push(String::from("╚══════════════════════════════════════════════════╝"));
    } else {
        ak.push(String::from("SDMA not initialized (requires AMD GPU)"));
    }

    ak
}


pub fn cie() -> Option<u64> {
    if !uc() { return None; }
    let g = IX_.lock();
    Some(g.cie)
}


pub fn zpl() -> usize {
    HE_
}
