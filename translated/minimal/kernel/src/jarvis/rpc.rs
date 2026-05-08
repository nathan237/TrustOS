























use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use spin::Mutex;





const Dp: &[u8; 4] = b"JRPC";
const DR_: usize = 13;
const BBY_: usize = 32 * 1024 * 1024; 
const ACD_: u32 = 30000;






#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Command {
    
    Ping = 0,

    
    
    GetWeights = 1,

    
    
    PushWeights = 2,

    
    
    PushGradients = 3,

    
    
    Inference = 4,

    
    
    GetStatus = 5,

    
    
    VoteRequest = 6,

    
    
    LeaderHeartbeat = 7,

    
    
    TrainData = 8,

    
    
    GetTrainingSteps = 9,

    
    
    TaskExecute = 10,

    
    
    
    GetWeightsChunk = 11,
}

impl Command {
    fn fxt(b: u8) -> Option<Self> {
        match b {
            0 => Some(Command::Ping),
            1 => Some(Command::GetWeights),
            2 => Some(Command::PushWeights),
            3 => Some(Command::PushGradients),
            4 => Some(Command::Inference),
            5 => Some(Command::GetStatus),
            6 => Some(Command::VoteRequest),
            7 => Some(Command::LeaderHeartbeat),
            8 => Some(Command::TrainData),
            9 => Some(Command::GetTrainingSteps),
            10 => Some(Command::TaskExecute),
            11 => Some(Command::GetWeightsChunk),
            _ => None,
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Status {
    Ok = 0,
    Error = 1,
    Busy = 2,
}






static DG_: AtomicBool = AtomicBool::new(false);


static CLD_: AtomicU32 = AtomicU32::new(1);


static APV_: AtomicU64 = AtomicU64::new(0);


static APU_: AtomicU64 = AtomicU64::new(0);






fn kfs(azh: u32, cmd: Command, payload: &[u8]) -> Vec<u8> {
    let mut fj = Vec::with_capacity(DR_ + payload.len());
    fj.extend_from_slice(Dp);
    fj.extend_from_slice(&azh.to_be_bytes());
    fj.push(cmd as u8);
    fj.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    fj.extend_from_slice(payload);
    fj
}


fn hjc(azh: u32, status: Status, payload: &[u8]) -> Vec<u8> {
    let mut fj = Vec::with_capacity(DR_ + payload.len());
    fj.extend_from_slice(Dp);
    fj.extend_from_slice(&azh.to_be_bytes());
    fj.push(status as u8);
    fj.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    fj.extend_from_slice(payload);
    fj
}


fn kft(azh: u32, status: Status, payload_len: u32) -> Vec<u8> {
    let mut fj = Vec::with_capacity(DR_);
    fj.extend_from_slice(Dp);
    fj.extend_from_slice(&azh.to_be_bytes());
    fj.push(status as u8);
    fj.extend_from_slice(&payload_len.to_be_bytes());
    fj
}



fn cnu(data: &[u8]) -> Option<(u32, u8, u32)> {
    if data.len() < DR_ {
        return None;
    }
    if &data[0..4] != Dp {
        return None;
    }
    let azh = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
    let cmd = data[8];
    let payload_len = u32::from_be_bytes([data[9], data[10], data[11], data[12]]);

    if payload_len as usize > BBY_ {
        return None;
    }

    Some((azh, cmd, payload_len))
}







pub fn alb(dest_ip: [u8; 4], dest_port: u16, cmd: Command, payload: &[u8]) -> Result<(Status, Vec<u8>), &'static str> {
    let azh = CLD_.fetch_add(1, Ordering::SeqCst);

    
    let src_port = crate::netstack::tcp::azp(dest_ip, dest_port)?;

    if !crate::netstack::tcp::bjy(dest_ip, dest_port, src_port, ACD_) {
        return Err("RPC connect timeout");
    }
    
    let request = kfs(azh, cmd, payload);
    crate::netstack::tcp::cqj(dest_ip, dest_port, src_port, &request)?;

    
    let mut ddn = Vec::new();
    let start = crate::time::uptime_ms();
    let timeout_ms = ACD_ as u64 * 4; 
    loop {
        crate::netstack::poll();

        
        while let Some(data) = crate::netstack::tcp::aus(dest_ip, dest_port, src_port) {
            ddn.extend_from_slice(&data);
        }

        
        if ddn.len() >= DR_ {
            if let Some((_, _, plen)) = cnu(&ddn) {
                let av = DR_ + plen as usize;
                if ddn.len() >= av {
                    
                    let status = match ddn[8] {
                        0 => Status::Ok,
                        2 => Status::Busy,
                        _ => Status::Error,
                    };
                    let ogk = ddn[DR_..av].to_vec();

                    
                    let _ = crate::netstack::tcp::ams(dest_ip, dest_port, src_port);
                    APU_.fetch_add(1, Ordering::SeqCst);

                    return Ok((status, ogk));
                }
            }
        }

        
        if crate::time::uptime_ms().wrapping_sub(start) > timeout_ms {
            let _ = crate::netstack::tcp::ams(dest_ip, dest_port, src_port);
            return Err("RPC response timeout");
        }

        
        for _ in 0..200 { core::hint::spin_loop(); }
    }
}






pub fn iux(dest_ip: [u8; 4], dest_port: u16) -> Result<bool, &'static str> {
    let (status, _) = alb(dest_ip, dest_port, Command::Ping, &[])?;
    Ok(status == Status::Ok)
}



pub fn fyy(dest_ip: [u8; 4], dest_port: u16) -> Result<Vec<u8>, &'static str> {
    let (status, payload) = alb(dest_ip, dest_port, Command::GetWeights, &[])?;
    if status == Status::Ok {
        Ok(payload)
    } else {
        Err("Peer returned error for GetWeights")
    }
}


pub fn ixc(dest_ip: [u8; 4], dest_port: u16, hcg: &[u8]) -> Result<(), &'static str> {
    let (status, _) = alb(dest_ip, dest_port, Command::PushWeights, hcg)?;
    if status == Status::Ok {
        Ok(())
    } else {
        Err("Peer rejected PushWeights")
    }
}



pub fn mec(dest_ip: [u8; 4], dest_port: u16, offset: u32, rs: u32) -> Result<Vec<u8>, &'static str> {
    let mut payload = Vec::with_capacity(8);
    payload.extend_from_slice(&offset.to_be_bytes());
    payload.extend_from_slice(&rs.to_be_bytes());
    let (status, data) = alb(dest_ip, dest_port, Command::GetWeightsChunk, &payload)?;
    if status == Status::Ok {
        Ok(data)
    } else {
        Err("Peer returned error for GetWeightsChunk")
    }
}



pub fn qiz(dest_ip: [u8; 4], dest_port: u16, total_size: u32, rs: u32) -> Result<Vec<u8>, &'static str> {
    let mut result = Vec::with_capacity(total_size as usize);
    let mut offset: u32 = 0;
    while offset < total_size {
        let ck = total_size - offset;
        let piv = rs.min(ck);
        let df = mec(dest_ip, dest_port, offset, piv)?;
        result.extend_from_slice(&df);
        offset += df.len() as u32;
        if df.is_empty() {
            break; 
        }
    }
    Ok(result)
}


pub fn nzr(dest_ip: [u8; 4], dest_port: u16, grad_bytes: &[u8]) -> Result<(), &'static str> {
    let (status, _) = alb(dest_ip, dest_port, Command::PushGradients, grad_bytes)?;
    if status == Status::Ok {
        Ok(())
    } else {
        Err("Peer rejected PushGradients")
    }
}


pub fn oev(dest_ip: [u8; 4], dest_port: u16, nh: &str) -> Result<String, &'static str> {
    let (status, payload) = alb(dest_ip, dest_port, Command::Inference, nh.as_bytes())?;
    if status == Status::Ok {
        Ok(String::from_utf8_lossy(&payload).into_owned())
    } else {
        Err("Remote inference failed")
    }
}


pub fn qtp(dest_ip: [u8; 4], dest_port: u16, text: &str) -> Result<f32, &'static str> {
    let (status, payload) = alb(dest_ip, dest_port, Command::TrainData, text.as_bytes())?;
    if status == Status::Ok && payload.len() == 4 {
        Ok(f32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]))
    } else {
        Err("Remote train failed")
    }
}






pub fn owj() {
    if DG_.load(Ordering::SeqCst) {
        return;
    }

    let port = super::mesh::HM_;
    crate::netstack::tcp::etd(port, 8);
    DG_.store(true, Ordering::SeqCst);
    crate::serial_println!("[RPC] Server started on port {}", port);
}


pub fn oxo() {
    if !DG_.load(Ordering::SeqCst) {
        return;
    }

    crate::netstack::tcp::gwj(super::mesh::HM_);
    DG_.store(false, Ordering::SeqCst);
    crate::serial_println!("[RPC] Server stopped");
}


pub fn nwb() {
    if !DG_.load(Ordering::SeqCst) {
        return;
    }

    let port = super::mesh::HM_;

    
    if let Some((src_port, tn, remote_port)) = crate::netstack::tcp::eew(port) {
        mhl(src_port, tn, remote_port);
    }
}


fn mhl(src_port: u16, tn: [u8; 4], remote_port: u16) {
    let mut buf = Vec::new();
    let start = crate::time::uptime_ms();

    
    while let Some(data) = crate::netstack::tcp::aus(tn, remote_port, src_port) {
        buf.extend_from_slice(&data);
    }

    loop {
        crate::netstack::poll();

        while let Some(data) = crate::netstack::tcp::aus(tn, remote_port, src_port) {
            buf.extend_from_slice(&data);
        }

        
        if buf.len() >= DR_ {
            if let Some((_, _, plen)) = cnu(&buf) {
                if buf.len() >= DR_ + plen as usize {
                    break;
                }
            }
        }

        if crate::time::uptime_ms().wrapping_sub(start) > ACD_ as u64 {
            let _ = crate::netstack::tcp::ams(tn, remote_port, src_port);
            return;
        }

        for _ in 0..200 { core::hint::spin_loop(); }
    }

    
    let (azh, cmd_byte, payload_len) = match cnu(&buf) {
        Some(v) => v,
        None => {
            let _ = crate::netstack::tcp::ams(tn, remote_port, src_port);
            return;
        }
    };

    let cmd = match Command::fxt(cmd_byte) {
        Some(c) => c,
        None => {
            let eo = hjc(azh, Status::Error, b"Unknown command");
            let _ = crate::netstack::tcp::cqj(tn, remote_port, src_port, &eo);
            let _ = crate::netstack::tcp::ams(tn, remote_port, src_port);
            return;
        }
    };

    let payload = &buf[DR_..DR_ + payload_len as usize];

    
    let (status, response_payload) = lfe(cmd, payload);

    
    
    let pmf = DR_ + response_payload.len();

    if response_payload.len() <= 65536 {
        
        let eo = hjc(azh, status, &response_payload);
        let _ = crate::netstack::tcp::cqj(tn, remote_port, src_port, &eo);
    } else {
        
        let ogj = kft(azh, status, response_payload.len() as u32);
        let _ = crate::netstack::tcp::cqj(tn, remote_port, src_port, &ogj);
        crate::netstack::poll();
        let _ = crate::netstack::tcp::cqj(tn, remote_port, src_port, &response_payload);
    }

    
    
    let lxd = if pmf > 1_000_000 { 100 } else { 20 };
    for _ in 0..lxd {
        crate::netstack::poll();
        for _ in 0..10_000 { core::hint::spin_loop(); }
    }

    let _ = crate::netstack::tcp::ams(tn, remote_port, src_port);
    APV_.fetch_add(1, Ordering::SeqCst);
}


fn lfe(cmd: Command, payload: &[u8]) -> (Status, Vec<u8>) {
    match cmd {
        Command::Ping => {
            (Status::Ok, Vec::new())
        }

        Command::GetWeights => {
            if !super::is_ready() {
                return (Status::Error, b"Brain not ready".to_vec());
            }
            let model = super::Ay.lock();
            match model.as_ref() {
                Some(m) => {
                    let bytes = m.serialize_to_bytes();
                    drop(model);
                    (Status::Ok, bytes)
                }
                None => (Status::Error, b"No model".to_vec()),
            }
        }

        Command::PushWeights => {
            if let Err(_) = super::guardian::bxo(super::guardian::ProtectedOp::ModelReplace) {
                return (Status::Error, b"Guardian denied: PushWeights".to_vec());
            }
            if !super::is_ready() {
                return (Status::Error, b"Brain not ready".to_vec());
            }
            let xn = fkj(payload);
            match super::model::TransformerWeights::byt(&xn) {
                Some(new_weights) => {
                    *super::Ay.lock() = Some(new_weights);
                    crate::serial_println!("[RPC] Model weights replaced from remote");
                    (Status::Ok, Vec::new())
                }
                None => (Status::Error, b"Invalid weights data".to_vec()),
            }
        }

        Command::PushGradients => {
            if let Err(_) = super::guardian::bxo(super::guardian::ProtectedOp::FederatedSync) {
                return (Status::Error, b"Guardian denied: PushGradients".to_vec());
            }
            
            if !super::is_ready() {
                return (Status::Error, b"Brain not ready".to_vec());
            }
            super::federated::iyq(payload);
            (Status::Ok, Vec::new())
        }

        Command::Inference => {
            if !super::is_ready() {
                return (Status::Error, b"Brain not ready".to_vec());
            }
            let nh = core::str::from_utf8(payload).unwrap_or("");
            let result = super::generate(nh, 64);
            (Status::Ok, result.into_bytes())
        }

        Command::GetStatus => {
            let status = format!(
                "ready={} steps={} role={:?} peers={}",
                super::is_ready(),
                super::BY_.load(Ordering::SeqCst),
                super::mesh::dwa(),
                super::mesh::ayz()
            );
            (Status::Ok, status.into_bytes())
        }

        Command::VoteRequest => {
            let result = super::consensus::miu(payload);
            (Status::Ok, result)
        }

        Command::LeaderHeartbeat => {
            let result = super::consensus::mhz(payload);
            (Status::Ok, result)
        }

        Command::TrainData => {
            if let Err(_) = super::guardian::bxo(super::guardian::ProtectedOp::Train) {
                return (Status::Error, b"Guardian denied: TrainData".to_vec());
            }
            if !super::is_ready() {
                return (Status::Error, b"Brain not ready".to_vec());
            }
            let text = core::str::from_utf8(payload).unwrap_or("");
            if text.is_empty() {
                return (Status::Error, b"Empty training data".to_vec());
            }
            let ka = super::bwo(text, 0.001);
            (Status::Ok, ka.to_be_bytes().to_vec())
        }

        Command::GetTrainingSteps => {
            let steps = super::BY_.load(Ordering::SeqCst);
            (Status::Ok, steps.to_be_bytes().to_vec())
        }

        Command::TaskExecute => {
            super::task::mir(payload)
        }

        Command::GetWeightsChunk => {
            if !super::is_ready() {
                return (Status::Error, b"Brain not ready".to_vec());
            }
            
            if payload.len() < 8 {
                return (Status::Error, b"Invalid chunk request".to_vec());
            }
            let offset = u32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]) as usize;
            let length = u32::from_be_bytes([payload[4], payload[5], payload[6], payload[7]]) as usize;

            let model = super::Ay.lock();
            match model.as_ref() {
                Some(m) => {
                    let total_bytes = m.param_count() * 4;
                    if offset >= total_bytes {
                        return (Status::Error, b"Offset out of range".to_vec());
                    }
                    let end = (offset + length).min(total_bytes);
                    let xo = m.serialize_to_bytes();
                    drop(model);
                    let df = xo[offset..end].to_vec();
                    (Status::Ok, df)
                }
                None => (Status::Error, b"No model".to_vec()),
            }
        }
    }
}






pub fn hzf(xn: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(xn.len() * 4);
    for f in xn {
        bytes.extend_from_slice(&f.to_be_bytes());
    }
    bytes
}


pub fn fkj(bytes: &[u8]) -> Vec<f32> {
    let count = bytes.len() / 4;
    let mut xn = Vec::with_capacity(count);
    for i in 0..count {
        let b = [bytes[i*4], bytes[i*4+1], bytes[i*4+2], bytes[i*4+3]];
        xn.push(f32::from_be_bytes(b));
    }
    xn
}


pub fn get_stats() -> (u64, u64, bool) {
    (
        APV_.load(Ordering::SeqCst),
        APU_.load(Ordering::SeqCst),
        DG_.load(Ordering::SeqCst),
    )
}
