








use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use spin::Mutex;


static Dr: AtomicBool = AtomicBool::new(false);


static ADS_: AtomicU64 = AtomicU64::new(0);


static ZO_: Mutex<BTreeMap<&'static str, &'static [u8]>> = Mutex::new(BTreeMap::new());


static It: Mutex<BTreeMap<u16, Qp>> = Mutex::new(BTreeMap::new());


static XC_: Mutex<u16> = Mutex::new(50000);


mod opcode {
    pub const Aoj: u16 = 1;   
    pub const Ary: u16 = 2;   
    pub const Xk: u16 = 3;  
    pub const Dk: u16 = 4;   
    pub const Hr: u16 = 5; 
}


mod error_code {
    pub const BDS_: u16 = 1;
    pub const BLM_: u16 = 2;
    pub const EOQ_: u16 = 3;
    pub const AYZ_: u16 = 4;
}


const AT_: usize = 512;


struct Qp {
    
    client_ip: [u8; 4],
    
    client_port: u16,
    
    local_port: u16,
    
    file_data: &'static [u8],
    
    current_block: u16,
    
    total_blocks: u16,
    
    complete: bool,
    
    last_send: u64,
    
    retries: u8,
}


pub fn is_running() -> bool {
    Dr.load(Ordering::Relaxed)
}


pub fn fwq() -> u64 {
    ADS_.load(Ordering::Relaxed)
}


pub fn cdg(name: &'static str, data: &'static [u8]) {
    let mut files = ZO_.lock();
    files.insert(name, data);
    crate::serial_println!("[TFTPD] Registered file '{}' ({} bytes)", name, data.len());
}


pub fn start() {
    if Dr.load(Ordering::Relaxed) {
        crate::serial_println!("[TFTPD] Already running");
        return;
    }

    Dr.store(true, Ordering::Relaxed);
    ADS_.store(0, Ordering::Relaxed);

    let files = ZO_.lock();
    crate::serial_println!("[TFTPD] TFTP server started on port 69 ({} files registered)", files.len());
    for (name, data) in files.iter() {
        crate::serial_println!("[TFTPD]   {} ({} bytes, {} blocks)",
            name, data.len(), (data.len() + AT_ - 1) / AT_);
    }
}


pub fn stop() {
    Dr.store(false, Ordering::Relaxed);
    let mut asf = It.lock();
    asf.clear();
    crate::serial_println!("[TFTPD] Server stopped");
}



pub fn mif(data: &[u8], src_ip: [u8; 4], src_port: u16) {
    if !Dr.load(Ordering::Relaxed) || data.len() < 4 {
        return;
    }

    let opcode = u16::from_be_bytes([data[0], data[1]]);

    match opcode {
        opcode::Aoj => mie(data, src_ip, src_port),
        opcode::Ary => {
            
            dzc(src_ip, src_port, 69, error_code::BLM_, "Write not supported");
        }
        _ => {
            dzc(src_ip, src_port, 69, error_code::AYZ_, "Invalid opcode");
        }
    }
}


pub fn mit(data: &[u8], src_ip: [u8; 4], src_port: u16, local_port: u16) {
    if !Dr.load(Ordering::Relaxed) || data.len() < 4 {
        return;
    }

    let opcode = u16::from_be_bytes([data[0], data[1]]);

    if opcode == opcode::Dk {
        let block = u16::from_be_bytes([data[2], data[3]]);
        mhf(src_ip, src_port, local_port, block);
    }
}


fn mie(data: &[u8], client_ip: [u8; 4], client_port: u16) {
    
    let payload = &data[2..];

    
    let lve = match payload.iter().position(|&b| b == 0) {
        Some(pos) => pos,
        None => {
            dzc(client_ip, client_port, 69, error_code::AYZ_, "Bad request");
            return;
        }
    };

    let filename = match core::str::from_utf8(&payload[..lve]) {
        Ok(j) => j,
        Err(_) => {
            dzc(client_ip, client_port, 69, error_code::BDS_, "Invalid filename");
            return;
        }
    };

    
    let bld = filename.trim_start_matches('/');

    crate::serial_println!("[TFTPD] RRQ for '{}' from {}.{}.{}.{}:{}",
        bld,
        client_ip[0], client_ip[1], client_ip[2], client_ip[3],
        client_port);

    
    let files = ZO_.lock();
    let file_data = match files.get(bld) {
        Some(data) => *data,
        None => {
            
            let jvh = if bld.starts_with("boot/") {
                &bld[5..]
            } else {
                bld
            };
            match files.get(jvh) {
                Some(data) => *data,
                None => {
                    drop(files);
                    crate::serial_println!("[TFTPD] File not found: '{}'", bld);
                    dzc(client_ip, client_port, 69, error_code::BDS_, "File not found");
                    return;
                }
            }
        }
    };
    drop(files);

    
    let local_port = {
        let mut tid = XC_.lock();
        let port = *tid;
        *tid = tid.wrapping_add(1);
        if *tid < 50000 { *tid = 50000; }
        port
    };

    let total_blocks = ((file_data.len() + AT_ - 1) / AT_).max(1) as u16;

    crate::serial_println!("[TFTPD] Starting transfer: '{}' ({} bytes, {} blocks) TID={}",
        bld, file_data.len(), total_blocks, local_port);

    
    let by = Qp {
        client_ip,
        client_port,
        local_port,
        file_data,
        current_block: 1,
        total_blocks,
        complete: false,
        last_send: crate::time::uptime_ms(),
        retries: 0,
    };

    
    gtu(&by);

    
    let mut asf = It.lock();
    asf.insert(local_port, by);
}


fn mhf(client_ip: [u8; 4], client_port: u16, local_port: u16, block: u16) {
    let mut asf = It.lock();

    let by = match asf.get_mut(&local_port) {
        Some(j) => j,
        None => return,
    };

    
    if by.client_ip != client_ip || by.client_port != client_port {
        return;
    }

    if block == by.current_block {
        
        by.current_block += 1;
        by.retries = 0;

        if by.current_block > by.total_blocks {
            
            let mwj = by.file_data.len() % AT_;
            if mwj == 0 && !by.file_data.is_empty() {
                
                onp(by);
            }
            crate::serial_println!("[TFTPD] Transfer complete for TID={}", local_port);
            by.complete = true;
            ADS_.fetch_add(1, Ordering::Relaxed);
            let port = local_port;
            drop(asf);
            kkr(port);
            return;
        }

        
        gtu(by);
        by.last_send = crate::time::uptime_ms();
    }
    
}


fn gtu(by: &Qp) {
    let block = by.current_block;
    let offset = ((block - 1) as usize) * AT_;
    let end = (offset + AT_).min(by.file_data.len());

    let hqr = if offset < by.file_data.len() {
        &by.file_data[offset..end]
    } else {
        &[]
    };

    
    let mut be = Vec::with_capacity(4 + hqr.len());
    be.extend_from_slice(&opcode::Xk.to_be_bytes());
    be.extend_from_slice(&block.to_be_bytes());
    be.extend_from_slice(hqr);

    let _ = crate::netstack::udp::azq(
        by.client_ip,
        by.client_port,
        by.local_port,
        &be,
    );
}


fn onp(by: &Qp) {
    let block = by.current_block;
    let mut be = Vec::with_capacity(4);
    be.extend_from_slice(&opcode::Xk.to_be_bytes());
    be.extend_from_slice(&block.to_be_bytes());

    let _ = crate::netstack::udp::azq(
        by.client_ip,
        by.client_port,
        by.local_port,
        &be,
    );
}


fn dzc(dest_ip: [u8; 4], dest_port: u16, src_port: u16, code: u16, bk: &str) {
    let ior = bk.as_bytes();
    let mut be = Vec::with_capacity(5 + ior.len());
    be.extend_from_slice(&opcode::Hr.to_be_bytes());
    be.extend_from_slice(&code.to_be_bytes());
    be.extend_from_slice(ior);
    be.push(0); 

    let _ = crate::netstack::udp::azq(dest_ip, dest_port, src_port, &be);
}


fn kkr(port: u16) {
    let mut asf = It.lock();
    asf.remove(&port);
}


pub fn poll() {
    if !Dr.load(Ordering::Relaxed) {
        return;
    }

    let cy = crate::time::uptime_ms();
    let mut asf = It.lock();
    let mut aph = Vec::new();

    for (port, by) in asf.iter_mut() {
        if by.complete {
            aph.push(*port);
            continue;
        }

        
        if cy.saturating_sub(by.last_send) > 3000 {
            if by.retries >= 5 {
                crate::serial_println!("[TFTPD] Transfer timeout for TID={}", port);
                aph.push(*port);
            } else {
                by.retries += 1;
                by.last_send = cy;
                gtu(by);
                crate::serial_println!("[TFTPD] Retransmit block {} for TID={} (retry {})",
                    by.current_block, port, by.retries);
            }
        }
    }

    for port in aph {
        asf.remove(&port);
    }
}


pub fn etb() -> Vec<(&'static str, usize)> {
    let files = ZO_.lock();
    files.iter().map(|(name, data)| (*name, data.len())).collect()
}


pub fn fgb() -> usize {
    let asf = It.lock();
    asf.len()
}


pub fn mtx(port: u16) -> bool {
    let asf = It.lock();
    asf.contains_key(&port)
}
