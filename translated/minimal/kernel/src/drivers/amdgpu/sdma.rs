


































use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use spin::Mutex;

use super::{kj, ib, Hz};
use super::regs;
use crate::memory;







const DU_: usize = 4096;
const AJC_: usize = DU_ * 4;

const CSG_: u32 = 12; 



const HW_: usize = 256 * 1024;


const CXX_: usize = 4096;


const ATY_: usize = 0x00;
const ATZ_: usize = 0x10;

const CSS_: usize = 0x100;
const CST_: usize = 0x110;


const BHM_: u64 = 10_000_000;


static AKT_: AtomicU64 = AtomicU64::new(0);
static AKW_: AtomicU64 = AtomicU64::new(0);













#[inline]
fn ddz(op: u32, sub_op: u32) -> u32 {
    ((sub_op & 0x3) << 26) | ((op & 0x3FFFF) << 8)
}


#[inline]
fn qux() -> u32 {
    ddz(regs::CVH_, 0)
}














fn omj(src_addr: u64, dst_addr: u64, nb: u32) -> [u32; 7] {
    [
        ddz(regs::CVF_, regs::CUW_),
        nb,
        0, 
        (src_addr & 0xFFFFFFFF) as u32,
        ((src_addr >> 32) & 0xFFFFFFFF) as u32,
        (dst_addr & 0xFFFFFFFF) as u32,
        ((dst_addr >> 32) & 0xFFFFFFFF) as u32,
    ]
}












fn omi(dst_addr: u64, fill_value: u32, nb: u32) -> [u32; 5] {
    [
        ddz(regs::CVE_, 0),
        (dst_addr & 0xFFFFFFFF) as u32,
        ((dst_addr >> 32) & 0xFFFFFFFF) as u32,
        fill_value,
        nb,
    ]
}











fn jdt(addr: u64, value: u32) -> [u32; 4] {
    [
        ddz(regs::CVG_, 0),
        (addr & 0xFFFFFFFF) as u32,
        ((addr >> 32) & 0xFFFFFFFF) as u32,
        value,
    ]
}










fn quy(addr: u64) -> [u32; 3] {
    [
        ddz(regs::CVI_, 0),
        (addr & 0xFFFFFFFF) as u32,
        ((addr >> 32) & 0xFFFFFFFF) as u32,
    ]
}












fn quz(dst_addr: u64, count: u32) -> [u32; 4] {
    [
        ddz(regs::CVJ_, regs::CVO_),
        (dst_addr & 0xFFFFFFFF) as u32,
        ((dst_addr >> 32) & 0xFFFFFFFF) as u32,
        count.saturating_sub(1),
    ]
}






struct Kp {
    
    index: usize,
    
    mmio_base: u64,
    
    reg_base: u32,
    
    ring_virt: u64,
    
    ring_phys: u64,
    
    wptr: u32,
    
    fence_seq: u32,
    
    transfers: u64,
    
    bytes: u64,
}


struct Aep {
    initialized: bool,
    mmio_base: u64,
    
    engines: [Option<Kp>; 2],
    
    status_virt: u64,
    status_phys: u64,
    
    staging_virt: u64,
    staging_phys: u64,
}

static JQ_: Mutex<Aep> = Mutex::new(Aep {
    initialized: false,
    mmio_base: 0,
    engines: [None, None],
    status_virt: 0,
    status_phys: 0,
    staging_virt: 0,
    staging_phys: 0,
});

static HV_: AtomicBool = AtomicBool::new(false);






fn aow(engine: &mut Kp, data: &[u32]) {
    let dq = engine.ring_virt as *mut u32;
    for (i, &qx) in data.iter().enumerate() {
        let idx = (engine.wptr as usize + i) % DU_;
        unsafe {
            core::ptr::write_volatile(dq.add(idx), qx);
        }
    }
    engine.wptr = ((engine.wptr as usize + data.len()) % DU_) as u32;
}


fn eyu(engine: &Kp) {
    unsafe {
        
        let hco = engine.wptr * 4;
        let jrk = engine.reg_base + regs::BHL_;
        ib(engine.mmio_base, jrk, hco);
        ib(engine.mmio_base, jrk + 4, 0); 
    }
}


fn quk(engine: &Kp) -> u32 {
    unsafe {
        let oio = engine.reg_base + regs::BHK_;
        let oin = kj(engine.mmio_base, oio);
        oin / 4
    }
}






fn igq(
    mmio_base: u64,
    engine_idx: usize,
    ring_virt: u64,
    ring_phys: u64,
    status_phys: u64,
    rptr_wb_offset: usize,
) -> Option<Kp> {
    let base = if engine_idx == 0 {
        regs::BHG_
    } else {
        regs::BHI_
    };

    crate::log!("[SDMA{}] Initializing engine (reg_base={:#X})", engine_idx, base);

    unsafe {
        
        let jiq = if engine_idx == 0 {
            regs::CUP_
        } else {
            regs::CUT_
        };
        let status = kj(mmio_base, jiq);
        crate::log!("[SDMA{}] STATUS={:#010X} (idle={})",
            engine_idx, status, (status & regs::CVN_) != 0);

        
        let hxq = if engine_idx == 0 {
            regs::BHH_
        } else {
            regs::BHJ_
        };
        ib(mmio_base, hxq, 1); 

        
        for _ in 0..1000 {
            core::hint::spin_loop();
        }

        
        let iye = base + regs::CUZ_;
        ib(mmio_base, iye, 0); 

        
        let dxi = ring_phys >> 8;
        let obu = base + regs::CUX_;
        let obt = base + regs::CUY_;
        ib(mmio_base, obu, (dxi & 0xFFFFFFFF) as u32);
        ib(mmio_base, obt, ((dxi >> 32) & 0xFFFFFFFF) as u32);

        
        ib(mmio_base, base + regs::BHK_, 0);
        ib(mmio_base, base + regs::CVC_, 0);
        ib(mmio_base, base + regs::BHL_, 0);
        ib(mmio_base, base + regs::CVD_, 0);

        
        let jbp = status_phys + rptr_wb_offset as u64;
        ib(mmio_base, base + regs::CVB_,
            (jbp & 0xFFFFFFFF) as u32);
        ib(mmio_base, base + regs::CVA_,
            ((jbp >> 32) & 0xFFFFFFFF) as u32);

        
        
        
        
        
        let obv = regs::CVK_
            | (CSG_ << regs::CVL_)
            | regs::CVM_;
        ib(mmio_base, iye, obv);

        
        ib(mmio_base, hxq, 0); 

        
        for _ in 0..10000 {
            core::hint::spin_loop();
        }
        let owv = kj(mmio_base, jiq);
        crate::log!("[SDMA{}] Post-init STATUS={:#010X}", engine_idx, owv);
    }

    Some(Kp {
        index: engine_idx,
        mmio_base,
        reg_base: base,
        ring_virt,
        ring_phys,
        wptr: 0,
        fence_seq: 1,
        transfers: 0,
        bytes: 0,
    })
}







pub fn init(mmio_base: u64) {
    crate::log!("[SDMA] ═══════════════════════════════════════════════════");
    crate::log!("[SDMA] SDMA Engine — Bare-metal DMA transfers (Navi 10)");
    crate::log!("[SDMA] ═══════════════════════════════════════════════════");

    if mmio_base == 0 {
        crate::log!("[SDMA] No MMIO base — skipping");
        return;
    }

    
    let omk = unsafe { kj(mmio_base, regs::CUS_) };
    crate::log!("[SDMA] SDMA0 VERSION={:#010X}", omk);

    
    let eyt = match alloc::alloc::Layout::from_size_align(AJC_, 4096) {
        Ok(l) => l,
        Err(_) => { crate::log!("[SDMA] ERROR: invalid ring layout"); return; }
    };

    let grn = unsafe { alloc::alloc::alloc_zeroed(eyt) } as u64;
    let grm = memory::lc(grn).unwrap_or(0);

    let grp = unsafe { alloc::alloc::alloc_zeroed(eyt) } as u64;
    let gro = memory::lc(grp).unwrap_or(0);

    if grm == 0 || gro == 0 {
        crate::log!("[SDMA] ERROR: Cannot get physical address for ring buffers");
        return;
    }

    crate::log!("[SDMA] Ring0: virt={:#X} phys={:#X} ({} dwords)",
        grn, grm, DU_);
    crate::log!("[SDMA] Ring1: virt={:#X} phys={:#X} ({} dwords)",
        grp, gro, DU_);

    
    let owx = match alloc::alloc::Layout::from_size_align(CXX_, 4096) {
        Ok(l) => l,
        Err(_) => { crate::log!("[SDMA] ERROR: invalid status layout"); return; }
    };
    let status_virt = unsafe { alloc::alloc::alloc_zeroed(owx) } as u64;
    let status_phys = memory::lc(status_virt).unwrap_or(0);

    if status_phys == 0 {
        crate::log!("[SDMA] ERROR: Cannot get phys for status buffer");
        return;
    }
    crate::log!("[SDMA] Status: virt={:#X} phys={:#X}", status_virt, status_phys);

    
    let ovw = match alloc::alloc::Layout::from_size_align(HW_, 4096) {
        Ok(l) => l,
        Err(_) => { crate::log!("[SDMA] ERROR: invalid staging layout"); return; }
    };
    let staging_virt = unsafe { alloc::alloc::alloc_zeroed(ovw) } as u64;
    let staging_phys = memory::lc(staging_virt).unwrap_or(0);

    if staging_phys == 0 {
        crate::log!("[SDMA] ERROR: Cannot get phys for staging buffer");
        return;
    }
    crate::log!("[SDMA] Staging: virt={:#X} phys={:#X} ({}KB)",
        staging_virt, staging_phys, HW_ / 1024);

    
    let hvx = igq(
        mmio_base, 0, grn, grm, status_phys, CSS_,
    );
    let hvy = igq(
        mmio_base, 1, grp, gro, status_phys, CST_,
    );

    let lnb = hvx.is_some();
    let lnc = hvy.is_some();

    
    let mut state = JQ_.lock();
    state.initialized = true;
    state.mmio_base = mmio_base;
    state.engines[0] = hvx;
    state.engines[1] = hvy;
    state.status_virt = status_virt;
    state.status_phys = status_phys;
    state.staging_virt = staging_virt;
    state.staging_phys = staging_phys;
    drop(state);

    HV_.store(true, Ordering::SeqCst);

    crate::log!("[SDMA] ───────────────────────────────────────────────────");
    crate::log!("[SDMA] Engine 0: {}", if lnb { "READY" } else { "FAILED" });
    crate::log!("[SDMA] Engine 1: {}", if lnc { "READY" } else { "FAILED" });
    crate::log!("[SDMA] Staging: {}KB for CPU→GPU transfers", HW_ / 1024);
    crate::log!("[SDMA] Commands: sdma copy|fill|test|bench|info");
    crate::log!("[SDMA] ───────────────────────────────────────────────────");
}











pub fn copy(src_phys: u64, dst_phys: u64, nb: u32, engine_idx: usize) -> Result<u32, &'static str> {
    if !HV_.load(Ordering::Relaxed) {
        return Err("SDMA not initialized");
    }
    if nb == 0 {
        return Ok(0);
    }
    
    if nb > (1 << 26) {
        return Err("Transfer too large (max 64MB per SDMA packet)");
    }
    let aqk = engine_idx.min(1);

    let mut state = JQ_.lock();
    let status_phys = state.status_phys;
    let status_virt = state.status_virt;
    let engine = state.engines[aqk].as_mut().ok_or("SDMA engine not ready")?;

    
    let bgk = engine.fence_seq;
    engine.fence_seq = engine.fence_seq.wrapping_add(1);
    if engine.fence_seq == 0 { engine.fence_seq = 1; }

    
    let dpl = if aqk == 0 { ATY_ } else { ATZ_ };
    let hya = status_virt + dpl as u64;
    let fwk = status_phys + dpl as u64;
    unsafe {
        core::ptr::write_volatile(hya as *mut u32, 0);
    }

    
    let kxs = omj(src_phys, dst_phys, nb);
    let fwl = jdt(fwk, bgk);

    aow(engine, &kxs);
    aow(engine, &fwl);

    
    eyu(engine);

    crate::serial_println!("[SDMA{}] COPY: {:#X} → {:#X} ({} bytes) fence={}",
        aqk, src_phys, dst_phys, nb, bgk);

    
    let mut bb = 0u64;
    loop {
        let current = unsafe { core::ptr::read_volatile(hya as *const u32) };
        if current == bgk {
            break;
        }
        bb += 1;
        if bb >= BHM_ {
            crate::serial_println!("[SDMA{}] TIMEOUT: fence expected {} got {}",
                aqk, bgk, current);
            return Err("SDMA copy timed out");
        }
        if bb % 100 == 0 {
            core::hint::spin_loop();
        }
    }

    
    engine.transfers += 1;
    engine.bytes += nb as u64;
    drop(state);

    AKT_.fetch_add(nb as u64, Ordering::Relaxed);
    AKW_.fetch_add(1, Ordering::Relaxed);

    crate::serial_println!("[SDMA{}] Copy complete in {} iters", aqk, bb);
    Ok(bgk)
}






pub fn fill(dst_phys: u64, fill_value: u32, nb: u32, engine_idx: usize) -> Result<u32, &'static str> {
    if !HV_.load(Ordering::Relaxed) {
        return Err("SDMA not initialized");
    }
    if nb == 0 {
        return Ok(0);
    }
    if nb & 3 != 0 {
        return Err("byte_count must be a multiple of 4");
    }
    if nb > (1 << 26) {
        return Err("Fill too large (max 64MB per SDMA packet)");
    }
    let aqk = engine_idx.min(1);

    let mut state = JQ_.lock();
    let status_phys = state.status_phys;
    let status_virt = state.status_virt;
    let engine = state.engines[aqk].as_mut().ok_or("SDMA engine not ready")?;

    let bgk = engine.fence_seq;
    engine.fence_seq = engine.fence_seq.wrapping_add(1);
    if engine.fence_seq == 0 { engine.fence_seq = 1; }

    let dpl = if aqk == 0 { ATY_ } else { ATZ_ };
    let hyb = status_virt + dpl as u64;
    let fwk = status_phys + dpl as u64;
    unsafe {
        core::ptr::write_volatile(hyb as *mut u32, 0);
    }

    let lvg = omi(dst_phys, fill_value, nb);
    let fwl = jdt(fwk, bgk);

    aow(engine, &lvg);
    aow(engine, &fwl);
    eyu(engine);

    crate::serial_println!("[SDMA{}] FILL: {:#X} = {:#010X} x{} bytes, fence={}",
        aqk, dst_phys, fill_value, nb, bgk);

    let mut bb = 0u64;
    loop {
        let current = unsafe { core::ptr::read_volatile(hyb as *const u32) };
        if current == bgk {
            break;
        }
        bb += 1;
        if bb >= BHM_ {
            crate::serial_println!("[SDMA{}] TIMEOUT: fence expected {} got {}",
                aqk, bgk, current);
            return Err("SDMA fill timed out");
        }
        if bb % 100 == 0 {
            core::hint::spin_loop();
        }
    }

    engine.transfers += 1;
    engine.bytes += nb as u64;
    drop(state);

    AKT_.fetch_add(nb as u64, Ordering::Relaxed);
    AKW_.fetch_add(1, Ordering::Relaxed);

    crate::serial_println!("[SDMA{}] Fill complete in {} iters", aqk, bb);
    Ok(bgk)
}










pub fn upload(data: &[u8], dst_phys: u64, engine_idx: usize) -> Result<usize, &'static str> {
    if !HV_.load(Ordering::Relaxed) {
        return Err("SDMA not initialized");
    }
    if data.is_empty() {
        return Ok(0);
    }

    
    let mut offset = 0usize;
    while offset < data.len() {
        let df = (data.len() - offset).min(HW_);
        
        let dhi = (df + 3) & !3;

        let state = JQ_.lock();
        let staging_virt = state.staging_virt;
        let staging_phys = state.staging_phys;
        drop(state);

        
        unsafe {
            let dst = staging_virt as *mut u8;
            let src = data.as_ptr().add(offset);
            core::ptr::copy_nonoverlapping(src, dst, df);
            
            if dhi > df {
                core::ptr::write_bytes(dst.add(df), 0, dhi - df);
            }
        }

        
        copy(
            staging_phys,
            dst_phys + offset as u64,
            dhi as u32,
            engine_idx,
        )?;

        offset += df;
    }

    Ok(data.len())
}







pub fn fsu(src_phys: u64, buf: &mut [u8], engine_idx: usize) -> Result<usize, &'static str> {
    if !HV_.load(Ordering::Relaxed) {
        return Err("SDMA not initialized");
    }
    if buf.is_empty() {
        return Ok(0);
    }

    let mut offset = 0usize;
    while offset < buf.len() {
        let df = (buf.len() - offset).min(HW_);
        let dhi = (df + 3) & !3;

        let state = JQ_.lock();
        let staging_virt = state.staging_virt;
        let staging_phys = state.staging_phys;
        drop(state);

        
        copy(
            src_phys + offset as u64,
            staging_phys,
            dhi as u32,
            engine_idx,
        )?;

        
        unsafe {
            let src = staging_virt as *const u8;
            let dst = buf.as_mut_ptr().add(offset);
            core::ptr::copy_nonoverlapping(src, dst, df);
        }

        offset += df;
    }

    Ok(buf.len())
}













pub fn cdp() -> (u32, u32) {
    if !HV_.load(Ordering::Relaxed) {
        return (0, 0);
    }

    let mut gd = 0u32;
    let mut gv = 0u32;

    
    let layout = match alloc::alloc::Layout::from_size_align(4096, 4096) {
        Ok(l) => l,
        Err(_) => { crate::serial_println!("[SDMA-TEST] FAIL: invalid test layout"); return (0, 1); }
    };
    let djw = unsafe { alloc::alloc::alloc_zeroed(layout) } as u64;
    let djx = unsafe { alloc::alloc::alloc_zeroed(layout) } as u64;
    let djv = memory::lc(djw).unwrap_or(0);
    let fjz = memory::lc(djx).unwrap_or(0);

    if djv == 0 || fjz == 0 {
        crate::serial_println!("[SDMA-TEST] FAIL: cannot allocate test buffers");
        return (0, 1);
    }

    
    crate::serial_println!("[SDMA-TEST] Test 1: CONST_FILL (engine 0, 1024 bytes, pattern=0xFACEFEED)");
    match fill(djv, 0xFACE_FEED, 1024, 0) {
        Ok(_) => {
            
            let ptr = djw as *const u32;
            let mut ok = true;
            for i in 0..256 {
                let val = unsafe { core::ptr::read_volatile(ptr.add(i)) };
                if val != 0xFACE_FEED {
                    crate::serial_println!("[SDMA-TEST]   MISMATCH at [{}]: got {:#010X}", i, val);
                    ok = false;
                    break;
                }
            }
            if ok { gd += 1; } else { gv += 1; }
        }
        Err(e) => {
            crate::serial_println!("[SDMA-TEST]   Error: {}", e);
            gv += 1;
        }
    }

    
    crate::serial_println!("[SDMA-TEST] Test 2: LINEAR COPY (engine 0, 1024 bytes)");
    match copy(djv, fjz, 1024, 0) {
        Ok(_) => {
            let gow = djx as *const u32;
            let mut ok = true;
            for i in 0..256 {
                let val = unsafe { core::ptr::read_volatile(gow.add(i)) };
                if val != 0xFACE_FEED {
                    crate::serial_println!("[SDMA-TEST]   MISMATCH at [{}]: got {:#010X}", i, val);
                    ok = false;
                    break;
                }
            }
            if ok { gd += 1; } else { gv += 1; }
        }
        Err(e) => {
            crate::serial_println!("[SDMA-TEST]   Error: {}", e);
            gv += 1;
        }
    }

    
    crate::serial_println!("[SDMA-TEST] Test 3: CONST_FILL (engine 1, 512 bytes, pattern=0xBAAD_C0DE)");
    
    unsafe {
        core::ptr::write_bytes(djx as *mut u8, 0, 4096);
    }
    match fill(fjz, 0xBAAD_C0DE, 512, 1) {
        Ok(_) => {
            let gow = djx as *const u32;
            let mut ok = true;
            for i in 0..128 {
                let val = unsafe { core::ptr::read_volatile(gow.add(i)) };
                if val != 0xBAAD_C0DE {
                    crate::serial_println!("[SDMA-TEST]   MISMATCH at [{}]: got {:#010X}", i, val);
                    ok = false;
                    break;
                }
            }
            if ok { gd += 1; } else { gv += 1; }
        }
        Err(e) => {
            crate::serial_println!("[SDMA-TEST]   Error: {}", e);
            gv += 1;
        }
    }

    
    crate::serial_println!("[SDMA-TEST] Test 4: CPU Upload via staging (256 bytes)");
    
    unsafe { core::ptr::write_bytes(djw as *mut u8, 0, 4096); }
    let cef: [u8; 256] = {
        let mut d = [0u8; 256];
        for i in 0..256 { d[i] = i as u8; }
        d
    };
    match upload(&cef, djv, 0) {
        Ok(_) => {
            let ptr = djw as *const u8;
            let mut ok = true;
            for i in 0..256 {
                let val = unsafe { core::ptr::read_volatile(ptr.add(i)) };
                if val != i as u8 {
                    crate::serial_println!("[SDMA-TEST]   MISMATCH at [{}]: got {} expected {}", i, val, i);
                    ok = false;
                    break;
                }
            }
            if ok { gd += 1; } else { gv += 1; }
        }
        Err(e) => {
            crate::serial_println!("[SDMA-TEST]   Error: {}", e);
            gv += 1;
        }
    }

    
    crate::serial_println!("[SDMA-TEST] Test 5: CPU Download via staging (256 bytes)");
    let mut agx = [0u8; 256];
    match fsu(djv, &mut agx, 0) {
        Ok(_) => {
            let mut ok = true;
            for i in 0..256 {
                if agx[i] != i as u8 {
                    crate::serial_println!("[SDMA-TEST]   MISMATCH at [{}]: got {} expected {}", i, agx[i], i);
                    ok = false;
                    break;
                }
            }
            if ok { gd += 1; } else { gv += 1; }
        }
        Err(e) => {
            crate::serial_println!("[SDMA-TEST]   Error: {}", e);
            gv += 1;
        }
    }

    
    unsafe {
        alloc::alloc::dealloc(djw as *mut u8, layout);
        alloc::alloc::dealloc(djx as *mut u8, layout);
    }

    (gd, gv)
}








pub fn kbl(size_kb: u32) -> Result<(u64, u64), &'static str> {
    if !HV_.load(Ordering::Relaxed) {
        return Err("SDMA not initialized");
    }
    let size_bytes = (size_kb as usize * 1024).min(HW_);
    let asw = (size_bytes + 3) & !3;

    
    let layout = alloc::alloc::Layout::from_size_align(asw, 4096)
        .map_err(|_| "allocation error")?;
    let bey = unsafe { alloc::alloc::alloc_zeroed(layout) } as u64;
    let bez = unsafe { alloc::alloc::alloc_zeroed(layout) } as u64;
    let buv = memory::lc(bey).ok_or("virt_to_phys failed")?;
    let dwm = memory::lc(bez).ok_or("virt_to_phys failed")?;

    
    let _ = fill(buv, 0, asw as u32, 0);

    
    let acd = 16u32;
    let pcl = crate::time::yf();
    for _ in 0..acd {
        fill(buv, 0xAAAA_BBBB, asw as u32, 0)?;
    }
    let pch = crate::time::yf();

    
    let pck = crate::time::yf();
    for _ in 0..acd {
        copy(buv, dwm, asw as u32, 0)?;
    }
    let pcg = crate::time::yf();

    
    unsafe {
        alloc::alloc::dealloc(bey as *mut u8, layout);
        alloc::alloc::dealloc(bez as *mut u8, layout);
    }

    
    let hyl = pch.saturating_sub(pcl).max(1);
    let hnr = pcg.saturating_sub(pck).max(1);
    

    
    let total_bytes = asw as u64 * acd as u64;

    
    
    
    
    
    let fwr = if hyl > 0 { (total_bytes * 1000) / (hyl * 1024) } else { 0 };
    let fol = if hnr > 0 { (total_bytes * 1000) / (hnr * 1024) } else { 0 };

    Ok((fwr, fol))
}






pub fn is_ready() -> bool {
    HV_.load(Ordering::Relaxed)
}


pub fn total_bytes() -> u64 {
    AKT_.load(Ordering::Relaxed)
}


pub fn jnz() -> u64 {
    AKW_.load(Ordering::Relaxed)
}


pub fn summary() -> String {
    if is_ready() {
        let bytes = total_bytes();
        let transfers = jnz();
        let arh = bytes / 1024;
        format!("SDMA: 2 engines, {} transfers, {} KB moved", transfers, arh)
    } else {
        String::from("SDMA: not initialized")
    }
}


pub fn info_lines() -> Vec<String> {
    let mut lines = Vec::new();

    if is_ready() {
        let state = JQ_.lock();
        lines.push(String::from("╔══════════════════════════════════════════════════╗"));
        lines.push(String::from("║       SDMA Engine — Bare-metal DMA Transfers     ║"));
        lines.push(String::from("╠══════════════════════════════════════════════════╣"));
        lines.push(format!("║ Status Buffer: {:#X}                      ║", state.status_phys));
        lines.push(format!("║ Staging:       {:#X} ({}KB)              ║",
            state.staging_phys, HW_ / 1024));
        lines.push(format!("║ Total Bytes:   {} KB                           ║", total_bytes() / 1024));
        lines.push(format!("║ Total Xfers:   {}                              ║", jnz()));
        lines.push(String::from("╠══════════════════════════════════════════════════╣"));

        for i in 0..2 {
            if let Some(ref engine) = state.engines[i] {
                lines.push(format!("║ SDMA{}: ring@{:#X} wptr={} seq={} xfers={} bytes={}",
                    i, engine.ring_phys, engine.wptr, engine.fence_seq,
                    engine.transfers, engine.bytes));
            } else {
                lines.push(format!("║ SDMA{}: not initialized                        ║", i));
            }
        }
        lines.push(String::from("╚══════════════════════════════════════════════════╝"));
    } else {
        lines.push(String::from("SDMA not initialized (requires AMD GPU)"));
    }

    lines
}


pub fn staging_phys() -> Option<u64> {
    if !is_ready() { return None; }
    let state = JQ_.lock();
    Some(state.staging_phys)
}


pub fn qxo() -> usize {
    HW_
}
