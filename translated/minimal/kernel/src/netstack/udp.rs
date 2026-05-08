

use alloc::collections::BTreeMap;
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU16, Ordering};
use spin::Mutex;


struct Afv {
    src_ip: [u8; 4],
    src_port: u16,
    data: Vec<u8>,
}

static AJH_: Mutex<BTreeMap<u16, VecDeque<Afv>>> = Mutex::new(BTreeMap::new());
static CLE_: AtomicU16 = AtomicU16::new(49152);

pub fn alq(data: &[u8], src_ip: [u8; 4]) {
    if data.len() < 8 {
        return;
    }

    let src_port = u16::from_be_bytes([data[0], data[1]]);
    let dst_port = u16::from_be_bytes([data[2], data[3]]);
    let payload = &data[8..];

    
    if dst_port == 68 {
        crate::netstack::dhcp::alq(payload);
        return;
    }

    
    if dst_port == 67 {
        crate::netstack::dhcpd::alq(payload);
        return;
    }

    
    if dst_port == 69 {
        crate::netstack::tftpd::mif(payload, src_ip, src_port);
        return;
    }

    
    if crate::netstack::tftpd::mtx(dst_port) {
        crate::netstack::tftpd::mit(payload, src_ip, src_port, dst_port);
        return;
    }

    let mut da = AJH_.lock();
    let queue = da.entry(dst_port).or_insert_with(VecDeque::new);
    if queue.len() < 32 {
        queue.push_back(Afv {
            src_ip,
            src_port,
            data: payload.to_vec(),
        });
    }
}

pub fn heu() -> u16 {
    CLE_.fetch_add(1, Ordering::Relaxed)
}

pub fn azq(dest_ip: [u8; 4], dest_port: u16, src_port: u16, payload: &[u8]) -> Result<(), &'static str> {
    let length = 8 + payload.len();
    let mut segment = Vec::with_capacity(length);
    segment.extend_from_slice(&src_port.to_be_bytes());
    segment.extend_from_slice(&dest_port.to_be_bytes());
    segment.extend_from_slice(&(length as u16).to_be_bytes());
    segment.extend_from_slice(&0u16.to_be_bytes()); 
    segment.extend_from_slice(payload);

    crate::netstack::ip::aha(dest_ip, 17, &segment)
}

pub fn eyc(port: u16) -> Option<Vec<u8>> {
    let mut da = AJH_.lock();
    let queue = da.get_mut(&port)?;
    let dmz = queue.pop_front()?;
    Some(dmz.data)
}


pub fn odt(port: u16) -> Option<(Vec<u8>, [u8; 4], u16)> {
    let mut da = AJH_.lock();
    let queue = da.get_mut(&port)?;
    let dmz = queue.pop_front()?;
    Some((dmz.data, dmz.src_ip, dmz.src_port))
}
