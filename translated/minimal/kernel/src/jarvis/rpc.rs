























use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use spin::Mutex;





const Iv: &[u8; 4] = b"JRPC";
const DJ_: usize = 13;
const AZW_: usize = 32 * 1024 * 1024; 
const AAQ_: u32 = 30000;






#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Command {
    
    Aww = 0,

    
    
    Asy = 1,

    
    
    Axf = 2,

    
    
    Axe = 3,

    
    
    Aue = 4,

    
    
    Bhx = 5,

    
    
    Bao = 6,

    
    
    Auu = 7,

    
    
    Azz = 8,

    
    
    Bhy = 9,

    
    
    Azs = 10,

    
    
    
    Asz = 11,
}

impl Command {
    fn kxf(o: u8) -> Option<Self> {
        match o {
            0 => Some(Command::Aww),
            1 => Some(Command::Asy),
            2 => Some(Command::Axf),
            3 => Some(Command::Axe),
            4 => Some(Command::Aue),
            5 => Some(Command::Bhx),
            6 => Some(Command::Bao),
            7 => Some(Command::Auu),
            8 => Some(Command::Azz),
            9 => Some(Command::Bhy),
            10 => Some(Command::Azs),
            11 => Some(Command::Asz),
            _ => None,
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Status {
    Ok = 0,
    Q = 1,
    Rq = 2,
}






static CZ_: AtomicBool = AtomicBool::new(false);


static CHU_: AtomicU32 = AtomicU32::new(1);


static ANR_: AtomicU64 = AtomicU64::new(0);


static ANQ_: AtomicU64 = AtomicU64::new(0);






fn qtz(ctv: u32, cmd: Command, ew: &[u8]) -> Vec<u8> {
    let mut mt = Vec::fc(DJ_ + ew.len());
    mt.bk(Iv);
    mt.bk(&ctv.ft());
    mt.push(cmd as u8);
    mt.bk(&(ew.len() as u32).ft());
    mt.bk(ew);
    mt
}


fn naq(ctv: u32, status: Status, ew: &[u8]) -> Vec<u8> {
    let mut mt = Vec::fc(DJ_ + ew.len());
    mt.bk(Iv);
    mt.bk(&ctv.ft());
    mt.push(status as u8);
    mt.bk(&(ew.len() as u32).ft());
    mt.bk(ew);
    mt
}


fn qua(ctv: u32, status: Status, bvx: u32) -> Vec<u8> {
    let mut mt = Vec::fc(DJ_);
    mt.bk(Iv);
    mt.bk(&ctv.ft());
    mt.push(status as u8);
    mt.bk(&bvx.ft());
    mt
}



fn fqg(f: &[u8]) -> Option<(u32, u8, u32)> {
    if f.len() < DJ_ {
        return None;
    }
    if &f[0..4] != Iv {
        return None;
    }
    let ctv = u32::oa([f[4], f[5], f[6], f[7]]);
    let cmd = f[8];
    let bvx = u32::oa([f[9], f[10], f[11], f[12]]);

    if bvx as usize > AZW_ {
        return None;
    }

    Some((ctv, cmd, bvx))
}







pub fn bto(kv: [u8; 4], rz: u16, cmd: Command, ew: &[u8]) -> Result<(Status, Vec<u8>), &'static str> {
    let ctv = CHU_.fetch_add(1, Ordering::SeqCst);

    
    let ey = crate::netstack::tcp::cue(kv, rz)?;

    if !crate::netstack::tcp::dnd(kv, rz, ey, AAQ_) {
        return Err("RPC connect timeout");
    }
    
    let request = qtz(ctv, cmd, ew);
    crate::netstack::tcp::fuf(kv, rz, ey, &request)?;

    
    let mut gqx = Vec::new();
    let ay = crate::time::lc();
    let sg = AAQ_ as u64 * 4; 
    loop {
        crate::netstack::poll();

        
        while let Some(f) = crate::netstack::tcp::cme(kv, rz, ey) {
            gqx.bk(&f);
        }

        
        if gqx.len() >= DJ_ {
            if let Some((_, _, hvh)) = fqg(&gqx) {
                let es = DJ_ + hvh as usize;
                if gqx.len() >= es {
                    
                    let status = match gqx[8] {
                        0 => Status::Ok,
                        2 => Status::Rq,
                        _ => Status::Q,
                    };
                    let vya = gqx[DJ_..es].ip();

                    
                    let _ = crate::netstack::tcp::bwx(kv, rz, ey);
                    ANQ_.fetch_add(1, Ordering::SeqCst);

                    return Ok((status, vya));
                }
            }
        }

        
        if crate::time::lc().nj(ay) > sg {
            let _ = crate::netstack::tcp::bwx(kv, rz, ey);
            return Err("RPC response timeout");
        }

        
        for _ in 0..200 { core::hint::hc(); }
    }
}






pub fn ovs(kv: [u8; 4], rz: u16) -> Result<bool, &'static str> {
    let (status, _) = bto(kv, rz, Command::Aww, &[])?;
    Ok(status == Status::Ok)
}



pub fn kyz(kv: [u8; 4], rz: u16) -> Result<Vec<u8>, &'static str> {
    let (status, ew) = bto(kv, rz, Command::Asy, &[])?;
    if status == Status::Ok {
        Ok(ew)
    } else {
        Err("Peer returned error for GetWeights")
    }
}


pub fn oym(kv: [u8; 4], rz: u16, mqk: &[u8]) -> Result<(), &'static str> {
    let (status, _) = bto(kv, rz, Command::Axf, mqk)?;
    if status == Status::Ok {
        Ok(())
    } else {
        Err("Peer rejected PushWeights")
    }
}



pub fn tfd(kv: [u8; 4], rz: u16, l: u32, aiw: u32) -> Result<Vec<u8>, &'static str> {
    let mut ew = Vec::fc(8);
    ew.bk(&l.ft());
    ew.bk(&aiw.ft());
    let (status, f) = bto(kv, rz, Command::Asz, &ew)?;
    if status == Status::Ok {
        Ok(f)
    } else {
        Err("Peer returned error for GetWeightsChunk")
    }
}



pub fn yuj(kv: [u8; 4], rz: u16, aay: u32, aiw: u32) -> Result<Vec<u8>, &'static str> {
    let mut result = Vec::fc(aay as usize);
    let mut l: u32 = 0;
    while l < aay {
        let ia = aay - l;
        let xgj = aiw.v(ia);
        let jj = tfd(kv, rz, l, xgj)?;
        result.bk(&jj);
        l += jj.len() as u32;
        if jj.is_empty() {
            break; 
        }
    }
    Ok(result)
}


pub fn voi(kv: [u8; 4], rz: u16, ixe: &[u8]) -> Result<(), &'static str> {
    let (status, _) = bto(kv, rz, Command::Axe, ixe)?;
    if status == Status::Ok {
        Ok(())
    } else {
        Err("Peer rejected PushGradients")
    }
}


pub fn vut(kv: [u8; 4], rz: u16, aau: &str) -> Result<String, &'static str> {
    let (status, ew) = bto(kv, rz, Command::Aue, aau.as_bytes())?;
    if status == Status::Ok {
        Ok(String::azw(&ew).bkc())
    } else {
        Err("Remote inference failed")
    }
}


pub fn zjf(kv: [u8; 4], rz: u16, text: &str) -> Result<f32, &'static str> {
    let (status, ew) = bto(kv, rz, Command::Azz, text.as_bytes())?;
    if status == Status::Ok && ew.len() == 4 {
        Ok(f32::oa([ew[0], ew[1], ew[2], ew[3]]))
    } else {
        Err("Remote train failed")
    }
}






pub fn wtc() {
    if CZ_.load(Ordering::SeqCst) {
        return;
    }

    let port = super::mesh::GV_;
    crate::netstack::tcp::jdt(port, 8);
    CZ_.store(true, Ordering::SeqCst);
    crate::serial_println!("[RPC] Server started on port {}", port);
}


pub fn wup() {
    if !CZ_.load(Ordering::SeqCst) {
        return;
    }

    crate::netstack::tcp::mhr(super::mesh::GV_);
    CZ_.store(false, Ordering::SeqCst);
    crate::serial_println!("[RPC] Server stopped");
}


pub fn vju() {
    if !CZ_.load(Ordering::SeqCst) {
        return;
    }

    let port = super::mesh::GV_;

    
    if let Some((ey, ams, bci)) = crate::netstack::tcp::iir(port) {
        tjd(ey, ams, bci);
    }
}


fn tjd(ey: u16, ams: [u8; 4], bci: u16) {
    let mut k = Vec::new();
    let ay = crate::time::lc();

    
    while let Some(f) = crate::netstack::tcp::cme(ams, bci, ey) {
        k.bk(&f);
    }

    loop {
        crate::netstack::poll();

        while let Some(f) = crate::netstack::tcp::cme(ams, bci, ey) {
            k.bk(&f);
        }

        
        if k.len() >= DJ_ {
            if let Some((_, _, hvh)) = fqg(&k) {
                if k.len() >= DJ_ + hvh as usize {
                    break;
                }
            }
        }

        if crate::time::lc().nj(ay) > AAQ_ as u64 {
            let _ = crate::netstack::tcp::bwx(ams, bci, ey);
            return;
        }

        for _ in 0..200 { core::hint::hc(); }
    }

    
    let (ctv, rct, bvx) = match fqg(&k) {
        Some(p) => p,
        None => {
            let _ = crate::netstack::tcp::bwx(ams, bci, ey);
            return;
        }
    };

    let cmd = match Command::kxf(rct) {
        Some(r) => r,
        None => {
            let lj = naq(ctv, Status::Q, b"Unknown command");
            let _ = crate::netstack::tcp::fuf(ams, bci, ey, &lj);
            let _ = crate::netstack::tcp::bwx(ams, bci, ey);
            return;
        }
    };

    let ew = &k[DJ_..DJ_ + bvx as usize];

    
    let (status, hxq) = ryg(cmd, ew);

    
    
    let xkq = DJ_ + hxq.len();

    if hxq.len() <= 65536 {
        
        let lj = naq(ctv, status, &hxq);
        let _ = crate::netstack::tcp::fuf(ams, bci, ey, &lj);
    } else {
        
        let vxy = qua(ctv, status, hxq.len() as u32);
        let _ = crate::netstack::tcp::fuf(ams, bci, ey, &vxy);
        crate::netstack::poll();
        let _ = crate::netstack::tcp::fuf(ams, bci, ey, &hxq);
    }

    
    
    let sva = if xkq > 1_000_000 { 100 } else { 20 };
    for _ in 0..sva {
        crate::netstack::poll();
        for _ in 0..10_000 { core::hint::hc(); }
    }

    let _ = crate::netstack::tcp::bwx(ams, bci, ey);
    ANR_.fetch_add(1, Ordering::SeqCst);
}


fn ryg(cmd: Command, ew: &[u8]) -> (Status, Vec<u8>) {
    match cmd {
        Command::Aww => {
            (Status::Ok, Vec::new())
        }

        Command::Asy => {
            if !super::uc() {
                return (Status::Q, b"Brain not ready".ip());
            }
            let model = super::Ci.lock();
            match model.as_ref() {
                Some(ef) => {
                    let bf = ef.pih();
                    drop(model);
                    (Status::Ok, bf)
                }
                None => (Status::Q, b"No model".ip()),
            }
        }

        Command::Axf => {
            if let Err(_) = super::guardian::emj(super::guardian::ProtectedOp::Bml) {
                return (Status::Q, b"Guardian denied: PushWeights".ip());
            }
            if !super::uc() {
                return (Status::Q, b"Brain not ready".ip());
            }
            let aue = kfu(ew);
            match super::model::TransformerWeights::eos(&aue) {
                Some(hst) => {
                    *super::Ci.lock() = Some(hst);
                    crate::serial_println!("[RPC] Model weights replaced from remote");
                    (Status::Ok, Vec::new())
                }
                None => (Status::Q, b"Invalid weights data".ip()),
            }
        }

        Command::Axe => {
            if let Err(_) = super::guardian::emj(super::guardian::ProtectedOp::Asf) {
                return (Status::Q, b"Guardian denied: PushGradients".ip());
            }
            
            if !super::uc() {
                return (Status::Q, b"Brain not ready".ip());
            }
            super::federated::pan(ew);
            (Status::Ok, Vec::new())
        }

        Command::Aue => {
            if !super::uc() {
                return (Status::Q, b"Brain not ready".ip());
            }
            let aau = core::str::jg(ew).unwrap_or("");
            let result = super::cks(aau, 64);
            (Status::Ok, result.cfq())
        }

        Command::Bhx => {
            let status = format!(
                "ready={} steps={} role={:?} peers={}",
                super::uc(),
                super::BW_.load(Ordering::SeqCst),
                super::mesh::htw(),
                super::mesh::cti()
            );
            (Status::Ok, status.cfq())
        }

        Command::Bao => {
            let result = super::consensus::tlq(ew);
            (Status::Ok, result)
        }

        Command::Auu => {
            let result = super::consensus::tkg(ew);
            (Status::Ok, result)
        }

        Command::Azz => {
            if let Err(_) = super::guardian::emj(super::guardian::ProtectedOp::Zf) {
                return (Status::Q, b"Guardian denied: TrainData".ip());
            }
            if !super::uc() {
                return (Status::Q, b"Brain not ready".ip());
            }
            let text = core::str::jg(ew).unwrap_or("");
            if text.is_empty() {
                return (Status::Q, b"Empty training data".ip());
            }
            let vl = super::ekd(text, 0.001);
            (Status::Ok, vl.ft().ip())
        }

        Command::Bhy => {
            let au = super::BW_.load(Ordering::SeqCst);
            (Status::Ok, au.ft().ip())
        }

        Command::Azs => {
            super::task::tlg(ew)
        }

        Command::Asz => {
            if !super::uc() {
                return (Status::Q, b"Brain not ready".ip());
            }
            
            if ew.len() < 8 {
                return (Status::Q, b"Invalid chunk request".ip());
            }
            let l = u32::oa([ew[0], ew[1], ew[2], ew[3]]) as usize;
            let go = u32::oa([ew[4], ew[5], ew[6], ew[7]]) as usize;

            let model = super::Ci.lock();
            match model.as_ref() {
                Some(ef) => {
                    let xv = ef.vm() * 4;
                    if l >= xv {
                        return (Status::Q, b"Offset out of range".ip());
                    }
                    let ci = (l + go).v(xv);
                    let auh = ef.pih();
                    drop(model);
                    let jj = auh[l..ci].ip();
                    (Status::Ok, jj)
                }
                None => (Status::Q, b"No model".ip()),
            }
        }
    }
}






pub fn nvc(aue: &[f32]) -> Vec<u8> {
    let mut bf = Vec::fc(aue.len() * 4);
    for bb in aue {
        bf.bk(&bb.ft());
    }
    bf
}


pub fn kfu(bf: &[u8]) -> Vec<f32> {
    let az = bf.len() / 4;
    let mut aue = Vec::fc(az);
    for a in 0..az {
        let o = [bf[a*4], bf[a*4+1], bf[a*4+2], bf[a*4+3]];
        aue.push(f32::oa(o));
    }
    aue
}


pub fn asx() -> (u64, u64, bool) {
    (
        ANR_.load(Ordering::SeqCst),
        ANQ_.load(Ordering::SeqCst),
        CZ_.load(Ordering::SeqCst),
    )
}
