

use alloc::collections::{BTreeMap, VecDeque};
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU16, Ordering};
use spin::Mutex;


pub mod flags {
    pub const Yn: u8 = 0x01;
    pub const Qd: u8 = 0x02;
    pub const Adg: u8 = 0x04;
    pub const Ug: u8 = 0x08;
    pub const Dk: u8 = 0x10;
    pub const Bek: u8 = 0x20;
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TcpState {
    Closed,
    Listen,
    SynSent,
    SynReceived,
    Established,
    FinWait1,
    FinWait2,
    CloseWait,
    LastAck,
    TimeWait,
    Closing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Db {
    src_ip: u32,
    dst_ip: u32,
    src_port: u16,
    dst_port: u16,
}

#[derive(Debug, Clone)]
struct Vj {
    state: TcpState,
    seq: u32, 
    ack: u32, 
    fin_received: bool,
    fin_sent: bool,
    
    pending_acks: u8,
    last_ack_time: u64,
    
    last_sent_seq: u32,
    last_sent_time: u64,
    retransmit_count: u8,
    
    snd_una: u32,
}


#[derive(Clone)]
struct Afw {
    conn_id: Db,
    seq: u32,
    data: Vec<u8>,       
    dest_ip: [u8; 4],
    dest_port: u16,
    src_port: u16,
    sent_time: u64,
    retries: u8,
}



static QC_: Mutex<VecDeque<Afw>> = Mutex::new(VecDeque::new());

const CIY_: usize = 32;

static Ci: Mutex<BTreeMap<Db, Vj>> = Mutex::new(BTreeMap::new());
static LT_: Mutex<BTreeMap<Db, VecDeque<Vec<u8>>>> = Mutex::new(BTreeMap::new());
static XB_: AtomicU16 = AtomicU16::new(49152);

static AZK_: Mutex<[u8; 16]> = Mutex::new([0u8; 16]);



fn iaz(src_ip: [u8; 4], dst_ip: [u8; 4], src_port: u16, dst_port: u16) -> u32 {
    let bvr = AZK_.lock();
    let mut data = [0u8; 28]; 
    data[0..4].copy_from_slice(&src_ip);
    data[4..8].copy_from_slice(&dst_ip);
    data[8..10].copy_from_slice(&src_port.to_be_bytes());
    data[10..12].copy_from_slice(&dst_port.to_be_bytes());
    data[12..28].copy_from_slice(&*bvr);
    drop(bvr);
    let hash = crate::tls13::crypto::asg(&data);
    let h = u32::from_be_bytes([hash[0], hash[1], hash[2], hash[3]]);
    
    let gx = crate::logger::eg() as u32;
    h.wrapping_add(gx)
}


pub fn mpj() {
    let mut j = AZK_.lock();
    crate::rng::jeb(&mut *j);
    crate::serial_println!("[TCP] ISN secret initialized (RFC 6528)");
}


struct Aav {
    backlog: u32,
    accept_queue: VecDeque<Db>,
}


static Mk: Mutex<BTreeMap<u16, Aav>> = Mutex::new(BTreeMap::new());


pub fn etd(port: u16, backlog: u32) {
    Mk.lock().insert(port, Aav { backlog, accept_queue: VecDeque::new() });
    crate::serial_println!("[TCP] Listening on port {}", port);
}


pub fn gwj(port: u16) {
    Mk.lock().remove(&port);
}



pub fn eew(cmi: u16) -> Option<(u16, [u8; 4], u16)> {
    let mut dar = Mk.lock();
    let daq = dar.get_mut(&cmi)?;
    let conn_id = daq.accept_queue.pop_front()?;
    let tn = [
        ((conn_id.dst_ip >> 24) & 0xFF) as u8,
        ((conn_id.dst_ip >> 16) & 0xFF) as u8,
        ((conn_id.dst_ip >> 8) & 0xFF) as u8,
        (conn_id.dst_ip & 0xFF) as u8,
    ];
    Some((conn_id.src_port, tn, conn_id.dst_port))
}


const BUF_: u8 = 4;
const BUE_: u64 = 20;


const JW_: u16 = 65535;


const BGN_: u64 = 1000;

const AGY_: u8 = 3;


const AGR_: usize = 512;

const BJJ_: u64 = 60_000;

fn abb(ip: [u8; 4]) -> u32 {
    ((ip[0] as u32) << 24) | ((ip[1] as u32) << 16) | ((ip[2] as u32) << 8) | (ip[3] as u32)
}


fn dkq(sum: &mut u32, data: &[u8]) {
    let mut i = 0;
    while i + 1 < data.len() {
        *sum += ((data[i] as u32) << 8) | (data[i + 1] as u32);
        i += 2;
    }
    if i < data.len() {
        *sum += (data[i] as u32) << 8;
    }
}

fn hkt(mut sum: u32) -> u16 {
    while (sum >> 16) != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    !(sum as u16)
}

fn checksum(data: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    dkq(&mut sum, data);
    hkt(sum)
}


fn ced(src_ip: [u8; 4], dst_ip: [u8; 4], segment: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    
    dkq(&mut sum, &src_ip);
    dkq(&mut sum, &dst_ip);
    let nyz: [u8; 4] = [0, 6, (segment.len() >> 8) as u8, segment.len() as u8];
    dkq(&mut sum, &nyz);
    dkq(&mut sum, segment);
    hkt(sum)
}


pub fn jlr(src_ip: [u8; 4], dst_ip: [u8; 4], segment: &[u8]) -> u16 {
    ced(src_ip, dst_ip, segment)
}



pub fn qgy() {
    let cy = crate::logger::eg();
    let mut nc = Ci.lock();
    let mut da = LT_.lock();
    nc.retain(|id, et| {
        let bts = match et.state {
            TcpState::TimeWait => cy.wrapping_sub(et.last_ack_time) < BJJ_,
            TcpState::Closed => false,
            _ => true,
        };
        if !bts {
            da.remove(id);
        }
        bts
    });
}


pub fn kxd() -> usize {
    Ci.lock().len()
}


pub fn mza() -> Vec<String> {
    let nc = Ci.lock();
    let mut out = Vec::with_capacity(nc.len());
    for (id, et) in nc.iter() {
        let dst = [
            ((id.dst_ip >> 24) & 0xFF) as u8,
            ((id.dst_ip >> 16) & 0xFF) as u8,
            ((id.dst_ip >> 8) & 0xFF) as u8,
            (id.dst_ip & 0xFF) as u8,
        ];
        out.push(alloc::format!(
            "{:<6} {}.{}.{}.{}:{:<5}  {:?}",
            id.src_port, dst[0], dst[1], dst[2], dst[3], id.dst_port, et.state
        ));
    }
    out
}

fn axg() -> [u8; 4] {
    crate::network::rd()
        .map(|(ip, _, _)| *ip.as_bytes())
        .unwrap_or([10, 0, 2, 15])
}


pub fn alq(data: &[u8], src_ip: [u8; 4], dst_ip: [u8; 4]) {
    if data.len() < 20 {
        return;
    }

    let src_port = u16::from_be_bytes([data[0], data[1]]);
    let dst_port = u16::from_be_bytes([data[2], data[3]]);
    let seq = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
    let dhc = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);
    let data_offset = data[12] >> 4;
    let bdy = data[13];
    let bte = (data_offset as usize) * 4;

    if data.len() < bte {
        return;
    }

    let conn_id = Db {
        src_ip: abb(dst_ip),
        dst_ip: abb(src_ip),
        src_port: dst_port,
        dst_port: src_port,
    };

    let payload = &data[bte..];
    let emp  = (bdy & flags::Yn) != 0;
    let fbx  = (bdy & flags::Qd) != 0;
    let ack  = (bdy & flags::Dk) != 0;
    let dds  = (bdy & flags::Adg) != 0;
    let nzh  = (bdy & flags::Ug) != 0;
    let payload_len = payload.len() as u32;

    let mut nc = Ci.lock();

    if let Some(et) = nc.get_mut(&conn_id) {
        
        if dds {
            et.state = TcpState::Closed;
            et.fin_received = true;
            et.fin_sent = true;
            return;
        }

        if et.fin_received && et.fin_sent && et.state == TcpState::Closed {
            return;
        }

        match et.state {
            
            TcpState::SynSent => {
                if fbx && ack {
                    et.state = TcpState::Established;
                    et.ack = seq.wrapping_add(1);
                    et.pending_acks = 0;
                    et.last_ack_time = crate::logger::eg();
                    drop(nc);
                    let _ = cqi(src_ip, src_port, dst_port);
                }
            }

            
            TcpState::SynReceived => {
                if ack {
                    et.state = TcpState::Established;
                    let cmi = conn_id.src_port;
                    drop(nc);
                    
                    let mut dar = Mk.lock();
                    if let Some(daq) = dar.get_mut(&cmi) {
                        daq.accept_queue.push_back(conn_id);
                    }
                }
            }

            
            TcpState::Established => {
                
                if ack && dhc > et.snd_una {
                    et.snd_una = dhc;
                    et.retransmit_count = 0;
                    
                    let mut cdm = QC_.lock();
                    cdm.retain(|gq| {
                        gq.conn_id != conn_id || gq.seq.wrapping_add(gq.data.len() as u32) > dhc
                    });
                }

                
                if !payload.is_empty() {
                    let ipn = seq.wrapping_add(payload_len);
                    if ipn > et.ack {
                        et.ack = ipn;
                        let mut da = LT_.lock();
                        da.entry(conn_id).or_insert_with(VecDeque::new).push_back(payload.to_vec());
                    }
                }

                if emp {
                    
                    et.ack = seq.wrapping_add(payload_len).wrapping_add(1);
                    et.fin_received = true;
                    et.state = TcpState::CloseWait;
                    drop(nc);
                    let _ = cqi(src_ip, src_port, dst_port);
                    return;
                }

                
                if !payload.is_empty() {
                    et.pending_acks = et.pending_acks.saturating_add(1);
                    let cy = crate::logger::eg();
                    let guw = nzh
                        || et.pending_acks >= BUF_
                        || cy.saturating_sub(et.last_ack_time) >= BUE_;
                    if guw {
                        et.pending_acks = 0;
                        et.last_ack_time = cy;
                        drop(nc);
                        let _ = cqi(src_ip, src_port, dst_port);
                    }
                }
            }

            
            TcpState::FinWait1 => {
                if emp && ack {
                    
                    et.ack = seq.wrapping_add(1);
                    et.fin_received = true;
                    et.state = TcpState::TimeWait;
                    et.last_ack_time = crate::logger::eg();
                    drop(nc);
                    let _ = cqi(src_ip, src_port, dst_port);
                } else if emp {
                    et.ack = seq.wrapping_add(1);
                    et.fin_received = true;
                    et.state = TcpState::Closing;
                    drop(nc);
                    let _ = cqi(src_ip, src_port, dst_port);
                } else if ack {
                    et.state = TcpState::FinWait2;
                }
            }

            TcpState::FinWait2 => {
                if emp {
                    et.ack = seq.wrapping_add(1);
                    et.fin_received = true;
                    et.state = TcpState::TimeWait;
                    et.last_ack_time = crate::logger::eg();
                    drop(nc);
                    let _ = cqi(src_ip, src_port, dst_port);
                }
            }

            TcpState::Closing => {
                if ack {
                    et.state = TcpState::TimeWait;
                    et.last_ack_time = crate::logger::eg(); 
                }
            }

            TcpState::LastAck => {
                if ack {
                    et.state = TcpState::Closed;
                }
            }

            TcpState::CloseWait => {
                
                if !payload.is_empty() {
                    et.ack = seq.wrapping_add(payload_len);
                    let mut da = LT_.lock();
                    da.entry(conn_id).or_insert_with(VecDeque::new).push_back(payload.to_vec());
                }
            }

            _ => {}
        }
    } else {
        
        if fbx && !ack {
            
            if nc.len() >= AGR_ {
                let cy = crate::logger::eg();
                let mut da = LT_.lock();
                nc.retain(|id, c| {
                    let bts = match c.state {
                        TcpState::TimeWait => cy.wrapping_sub(c.last_ack_time) < BJJ_,
                        TcpState::Closed => false,
                        _ => true,
                    };
                    if !bts { da.remove(id); }
                    bts
                });
            }
            if nc.len() >= AGR_ {
                crate::serial_println!("[TCP] Connection limit reached ({}), dropping SYN", AGR_);
                return;
            }
            let mut dar = Mk.lock();
            if let Some(daq) = dar.get_mut(&dst_port) {
                if daq.accept_queue.len() < daq.backlog as usize + 16 {
                    let ovj = [
                        ((conn_id.src_ip >> 24) & 0xFF) as u8,
                        ((conn_id.src_ip >> 16) & 0xFF) as u8,
                        ((conn_id.src_ip >> 8) & 0xFF) as u8,
                        (conn_id.src_ip & 0xFF) as u8,
                    ];
                    let llp = [
                        ((conn_id.dst_ip >> 24) & 0xFF) as u8,
                        ((conn_id.dst_ip >> 16) & 0xFF) as u8,
                        ((conn_id.dst_ip >> 8) & 0xFF) as u8,
                        (conn_id.dst_ip & 0xFF) as u8,
                    ];
                    let eqm = iaz(ovj, llp, conn_id.src_port, conn_id.dst_port);
                    let nit = Vj {
                        state: TcpState::SynReceived,
                        seq: eqm.wrapping_add(1),
                        ack: seq.wrapping_add(1),
                        fin_received: false,
                        fin_sent: false,
                        pending_acks: 0,
                        last_ack_time: crate::logger::eg(),
                        last_sent_seq: eqm,
                        last_sent_time: crate::logger::eg(),
                        retransmit_count: 0,
                        snd_una: eqm.wrapping_add(1),
                    };
                    drop(dar);
                    nc.insert(conn_id, nit);
                    drop(nc);
                    
                    let _ = ooc(src_ip, src_port, dst_port,
                                         seq.wrapping_add(1), eqm);
                }
            }
        }
    }
}


pub fn azp(dest_ip: [u8; 4], dest_port: u16) -> Result<u16, &'static str> {
    let src_ip = axg();
    
    let src_port = XB_.fetch_add(1, Ordering::Relaxed);
    let seq = iaz(src_ip, dest_ip, src_port, dest_port);

    let mut segment = Vec::with_capacity(20);
    segment.extend_from_slice(&src_port.to_be_bytes());
    segment.extend_from_slice(&dest_port.to_be_bytes());
    segment.extend_from_slice(&seq.to_be_bytes());
    segment.extend_from_slice(&0u32.to_be_bytes()); 
    segment.push(0x50); 
    segment.push(flags::Qd);
    segment.extend_from_slice(&JW_.to_be_bytes()); 
    segment.extend_from_slice(&0u16.to_be_bytes()); 
    segment.extend_from_slice(&0u16.to_be_bytes()); 

    let ig = ced(src_ip, dest_ip, &segment);
    segment[16] = (ig >> 8) as u8;
    segment[17] = (ig & 0xFF) as u8;
    
    

    let conn_id = Db {
        src_ip: abb(src_ip),
        dst_ip: abb(dest_ip),
        src_port,
        dst_port: dest_port,
    };
    Ci.lock().insert(conn_id, Vj {
        state: TcpState::SynSent,
        seq: seq.wrapping_add(1),
        ack: 0,
        fin_received: false,
        fin_sent: false,
        pending_acks: 0,
        last_ack_time: crate::logger::eg(),
        last_sent_seq: seq,
        last_sent_time: crate::logger::eg(),
        retransmit_count: 0,
        snd_una: seq.wrapping_add(1),
    });

    

    crate::netstack::ip::aha(dest_ip, 6, &segment)?;
    Ok(src_port)
}


fn ooc(dest_ip: [u8; 4], dest_port: u16, src_port: u16, dhc: u32, seq: u32) -> Result<(), &'static str> {
    let src_ip = axg();

    let mut segment = Vec::with_capacity(20);
    segment.extend_from_slice(&src_port.to_be_bytes());
    segment.extend_from_slice(&dest_port.to_be_bytes());
    segment.extend_from_slice(&seq.to_be_bytes());
    segment.extend_from_slice(&dhc.to_be_bytes());
    segment.push(0x50);
    segment.push(flags::Qd | flags::Dk);
    segment.extend_from_slice(&JW_.to_be_bytes());
    segment.extend_from_slice(&0u16.to_be_bytes());
    segment.extend_from_slice(&0u16.to_be_bytes());

    let ig = ced(src_ip, dest_ip, &segment);
    segment[16] = (ig >> 8) as u8;
    segment[17] = (ig & 0xFF) as u8;

    crate::netstack::ip::aha(dest_ip, 6, &segment)
}


pub fn cqi(dest_ip: [u8; 4], dest_port: u16, src_port: u16) -> Result<(), &'static str> {
    let src_ip = axg();
    let conn_id = Db {
        src_ip: abb(src_ip),
        dst_ip: abb(dest_ip),
        src_port,
        dst_port: dest_port,
    };

    let (seq, ack) = {
        let nc = Ci.lock();
        let et = nc.get(&conn_id).ok_or("Connection not found")?;
        (et.seq, et.ack)
    };

    let mut segment = Vec::with_capacity(20);
    segment.extend_from_slice(&src_port.to_be_bytes());
    segment.extend_from_slice(&dest_port.to_be_bytes());
    segment.extend_from_slice(&seq.to_be_bytes());
    segment.extend_from_slice(&ack.to_be_bytes());
    segment.push(0x50);
    segment.push(flags::Dk);
    segment.extend_from_slice(&JW_.to_be_bytes());
    segment.extend_from_slice(&0u16.to_be_bytes());
    segment.extend_from_slice(&0u16.to_be_bytes());

    let ig = ced(src_ip, dest_ip, &segment);
    segment[16] = (ig >> 8) as u8;
    segment[17] = (ig & 0xFF) as u8;

    crate::netstack::ip::aha(dest_ip, 6, &segment)
}


pub fn bjc(dest_ip: [u8; 4], dest_port: u16, src_port: u16, payload: &[u8]) -> Result<(), &'static str> {
    let src_ip = axg();
    let conn_id = Db {
        src_ip: abb(src_ip),
        dst_ip: abb(dest_ip),
        src_port,
        dst_port: dest_port,
    };

    let (seq, ack) = {
        let nc = Ci.lock();
        let et = nc.get(&conn_id).ok_or("Connection not found")?;
        (et.seq, et.ack)
    };

    let mut segment = Vec::with_capacity(20 + payload.len());
    segment.extend_from_slice(&src_port.to_be_bytes());
    segment.extend_from_slice(&dest_port.to_be_bytes());
    segment.extend_from_slice(&seq.to_be_bytes());
    segment.extend_from_slice(&ack.to_be_bytes());
    segment.push(0x50);
    segment.push(flags::Ug | flags::Dk);
    segment.extend_from_slice(&JW_.to_be_bytes());
    segment.extend_from_slice(&0u16.to_be_bytes());
    segment.extend_from_slice(&0u16.to_be_bytes());
    segment.extend_from_slice(payload);

    let ig = ced(src_ip, dest_ip, &segment);
    segment[16] = (ig >> 8) as u8;
    segment[17] = (ig & 0xFF) as u8;

    crate::netstack::ip::aha(dest_ip, 6, &segment)?;

    let cy = crate::time::uptime_ms();
    let mut nc = Ci.lock();
    if let Some(et) = nc.get_mut(&conn_id) {
        et.last_sent_seq = seq;
        et.last_sent_time = cy;
        et.seq = et.seq.wrapping_add(payload.len() as u32);
    }
    drop(nc);

    
    let mut cdm = QC_.lock();
    if cdm.len() >= CIY_ {
        cdm.pop_front(); 
    }
    cdm.push_back(Afw {
        conn_id,
        seq,
        data: payload.to_vec(),
        dest_ip,
        dest_port,
        src_port,
        sent_time: cy,
        retries: 0,
    });

    Ok(())
}


pub fn ams(dest_ip: [u8; 4], dest_port: u16, src_port: u16) -> Result<(), &'static str> {
    let src_ip = axg();
    let conn_id = Db {
        src_ip: abb(src_ip),
        dst_ip: abb(dest_ip),
        src_port,
        dst_port: dest_port,
    };

    let (seq, ack, already_sent) = {
        let nc = Ci.lock();
        let et = nc.get(&conn_id).ok_or("Connection not found")?;
        (et.seq, et.ack, et.fin_sent)
    };

    if already_sent {
        return Ok(());
    }

    let mut segment = Vec::with_capacity(20);
    segment.extend_from_slice(&src_port.to_be_bytes());
    segment.extend_from_slice(&dest_port.to_be_bytes());
    segment.extend_from_slice(&seq.to_be_bytes());
    segment.extend_from_slice(&ack.to_be_bytes());
    segment.push(0x50);
    segment.push(flags::Yn | flags::Dk);
    segment.extend_from_slice(&JW_.to_be_bytes());
    segment.extend_from_slice(&0u16.to_be_bytes());
    segment.extend_from_slice(&0u16.to_be_bytes());

    let ig = ced(src_ip, dest_ip, &segment);
    segment[16] = (ig >> 8) as u8;
    segment[17] = (ig & 0xFF) as u8;

    crate::netstack::ip::aha(dest_ip, 6, &segment)?;

    
    QC_.lock().retain(|gq| gq.conn_id != conn_id);

    let mut nc = Ci.lock();
    if let Some(et) = nc.get_mut(&conn_id) {
        et.seq = et.seq.wrapping_add(1);
        et.fin_sent = true;
        
        match et.state {
            TcpState::Established => et.state = TcpState::FinWait1,
            TcpState::CloseWait  => et.state = TcpState::LastAck,
            _ => {}
        }
    }
    Ok(())
}


pub fn cjr(dest_ip: [u8; 4], dest_port: u16, src_port: u16) {
    let src_ip = axg();
    let conn_id = Db {
        src_ip: abb(src_ip),
        dst_ip: abb(dest_ip),
        src_port,
        dst_port: dest_port,
    };

    let guw = {
        let mut nc = Ci.lock();
        if let Some(et) = nc.get_mut(&conn_id) {
            if et.pending_acks > 0 {
                et.pending_acks = 0;
                et.last_ack_time = crate::logger::eg();
                true
            } else {
                false
            }
        } else {
            false
        }
    };

    if guw {
        let _ = cqi(dest_ip, dest_port, src_port);
    }
}


pub fn aus(dest_ip: [u8; 4], dest_port: u16, src_port: u16) -> Option<Vec<u8>> {
    let src_ip = axg();
    let conn_id = Db {
        src_ip: abb(src_ip),
        dst_ip: abb(dest_ip),
        src_port,
        dst_port: dest_port,
    };

    {
        let mut da = LT_.lock();
        if let Some(queue) = da.get_mut(&conn_id) {
            if !queue.is_empty() {
                return queue.pop_front();
            }
        }
    }

    let mut nc = Ci.lock();
    if let Some(et) = nc.get(&conn_id) {
        if et.fin_received && et.fin_sent {
            nc.remove(&conn_id);
            LT_.lock().remove(&conn_id);
        }
    }
    None
}


pub fn fin_received(dest_ip: [u8; 4], dest_port: u16, src_port: u16) -> bool {
    let src_ip = axg();
    let conn_id = Db {
        src_ip: abb(src_ip),
        dst_ip: abb(dest_ip),
        src_port,
        dst_port: dest_port,
    };

    Ci
        .lock()
        .get(&conn_id)
        .map(|c| c.fin_received)
        .unwrap_or(true)
}


pub fn bjy(dest_ip: [u8; 4], dest_port: u16, src_port: u16, timeout_ms: u32) -> bool {
    let src_ip = axg();
    let conn_id = Db {
        src_ip: abb(src_ip),
        dst_ip: abb(dest_ip),
        src_port,
        dst_port: dest_port,
    };

    let start = crate::logger::eg();
    let mut ijl = start;
    let mut jkj: u8 = 0;
    let mut my: u32 = 0;
    
    loop {
        crate::netstack::poll();

        if let Some(et) = Ci.lock().get(&conn_id) {
            if et.state == TcpState::Established {
                return true;
            }
            
            if et.state == TcpState::Closed {
                return false;
            }
        }

        let cy = crate::logger::eg();
        
        
        if cy.saturating_sub(ijl) > BGN_ && jkj < AGY_ {
            jkj += 1;
            ijl = cy;
            
            
            
            let seq = {
                let nc = Ci.lock();
                nc.get(&conn_id).map(|c| c.last_sent_seq).unwrap_or(0)
            };
            
            let mut segment = Vec::with_capacity(20);
            segment.extend_from_slice(&src_port.to_be_bytes());
            segment.extend_from_slice(&dest_port.to_be_bytes());
            segment.extend_from_slice(&seq.to_be_bytes());
            segment.extend_from_slice(&0u32.to_be_bytes());
            segment.push(0x50);
            segment.push(flags::Qd);
            segment.extend_from_slice(&JW_.to_be_bytes());
            segment.extend_from_slice(&0u16.to_be_bytes());
            segment.extend_from_slice(&0u16.to_be_bytes());
            
            let ig = ced(src_ip, dest_ip, &segment);
            segment[16] = (ig >> 8) as u8;
            segment[17] = (ig & 0xFF) as u8;
            
            let _ = crate::netstack::ip::aha(dest_ip, 6, &segment);
        }

        if cy.saturating_sub(start) > timeout_ms as u64 {
            return false;
        }
        my = my.wrapping_add(1);
        if my > 2_000_000 {
            return false;
        }
        
        crate::thread::ajc();
    }
}






pub fn czx(dest_ip: [u8; 4], dest_port: u16) -> bool {
    let src_ip = axg();
    let nc = Ci.lock();
    
    
    
    for (id, et) in nc.iter() {
        if id.dst_ip == abb(dest_ip) && id.dst_port == dest_port
            && id.src_ip == abb(src_ip)
            && et.state == TcpState::Established
        {
            return true;
        }
    }
    false
}


pub fn cqj(dest_ip: [u8; 4], dest_port: u16, src_port: u16, data: &[u8]) -> Result<(), &'static str> {
    if data.is_empty() {
        return Ok(());
    }
    
    
    const To: usize = 1400;
    
    const Ahe: usize = 4;
    
    let ras = (data.len() + To - 1) / To;
    
    for (i, df) in data.chunks(To).enumerate() {
        
        let mut retries = 0u32;
        loop {
            match bjc(dest_ip, dest_port, src_port, df) {
                Ok(()) => break,
                Err(e) if retries < 200 => {
                    
                    crate::netstack::poll();
                    crate::thread::ajc();
                    retries += 1;
                }
                Err(e) => return Err(e),
            }
        }
        
        
        if (i + 1) % Ahe == 0 {
            crate::netstack::poll();
            
            crate::thread::ajc();
        }
        
    }
    
    Ok(())
}


pub fn iyp(dest_ip: [u8; 4], dest_port: u16, src_port: u16) -> Option<alloc::vec::Vec<u8>> {
    aus(dest_ip, dest_port, src_port)
}


pub fn ibk(dest_ip: [u8; 4], dest_port: u16, src_port: u16) -> Option<TcpState> {
    let src_ip = axg();
    let conn_id = Db {
        src_ip: abb(src_ip),
        dst_ip: abb(dest_ip),
        src_port,
        dst_port: dest_port,
    };
    
    Ci.lock().get(&conn_id).map(|c| c.state)
}



pub fn kjp() {
    let cy = crate::time::uptime_ms();
    let mut cdm = QC_.lock();

    for gq in cdm.iter_mut() {
        if cy.wrapping_sub(gq.sent_time) < BGN_ {
            continue;
        }
        if gq.retries >= AGY_ {
            continue; 
        }

        
        let conn_id = gq.conn_id;
        let nc = Ci.lock();
        let oxj = nc.get(&conn_id)
            .map(|c| c.state == TcpState::Established && c.snd_una <= gq.seq)
            .unwrap_or(false);
        drop(nc);

        if !oxj {
            continue;
        }

        
        let src_ip = axg();
        let dest_ip = gq.dest_ip;
        let jtk = {
            let nc = Ci.lock();
            nc.get(&conn_id).map(|c| c.ack).unwrap_or(0)
        };

        let mut avi = Vec::with_capacity(20 + gq.data.len());
        avi.extend_from_slice(&gq.src_port.to_be_bytes());
        avi.extend_from_slice(&gq.dest_port.to_be_bytes());
        avi.extend_from_slice(&gq.seq.to_be_bytes());
        avi.extend_from_slice(&jtk.to_be_bytes());
        avi.push(0x50);
        avi.push(flags::Ug | flags::Dk);
        avi.extend_from_slice(&JW_.to_be_bytes());
        avi.extend_from_slice(&0u16.to_be_bytes());
        avi.extend_from_slice(&0u16.to_be_bytes());
        avi.extend_from_slice(&gq.data);

        let ig = ced(src_ip, dest_ip, &avi);
        avi[16] = (ig >> 8) as u8;
        avi[17] = (ig & 0xFF) as u8;

        let _ = crate::netstack::ip::aha(dest_ip, 6, &avi);
        gq.retries += 1;
        gq.sent_time = cy; 
    }

    
    cdm.retain(|gq| gq.retries < AGY_);
}


pub fn qaa(dest_ip: [u8; 4], dest_port: u16, src_port: u16) {
    let src_ip = axg();
    let conn_id = Db {
        src_ip: abb(src_ip),
        dst_ip: abb(dest_ip),
        src_port,
        dst_port: dest_port,
    };
    QC_.lock().retain(|gq| gq.conn_id != conn_id);
}
