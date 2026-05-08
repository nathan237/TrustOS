




use alloc::vec::Vec;
use alloc::string::String;


mod asn1_tags {
    pub const Ahh: u8 = 0x01;
    pub const Axe: u8 = 0x02;
    pub const BNV_: u8 = 0x03;
    pub const EBA_: u8 = 0x04;
    pub const Azs: u8 = 0x05;
    pub const Azv: u8 = 0x06;
    pub const ENG_: u8 = 0x0C;
    pub const EET_: u8 = 0x13;
    pub const DTU_: u8 = 0x16;
    pub const BKK_: u8 = 0x17;
    pub const AUW_: u8 = 0x18;
    pub const Ha: u8 = 0x30;
    pub const Aox: u8 = 0x31;
    pub const BRK_: u8 = 0xA0;
    pub const DKJ_: u8 = 0xA1;
    pub const DKK_: u8 = 0xA2;
    pub const BRL_: u8 = 0xA3;
}


fn itw(data: &[u8], pos: &mut usize) -> Option<usize> {
    if *pos >= data.len() {
        return None;
    }
    
    let first = data[*pos];
    *pos += 1;
    
    if first < 0x80 {
        Some(first as usize)
    } else if first == 0x81 {
        if *pos >= data.len() {
            return None;
        }
        let len = data[*pos] as usize;
        *pos += 1;
        Some(len)
    } else if first == 0x82 {
        if *pos + 1 >= data.len() {
            return None;
        }
        let len = ((data[*pos] as usize) << 8) | (data[*pos + 1] as usize);
        *pos += 2;
        Some(len)
    } else if first == 0x83 {
        if *pos + 2 >= data.len() {
            return None;
        }
        let len = ((data[*pos] as usize) << 16) 
            | ((data[*pos + 1] as usize) << 8) 
            | (data[*pos + 2] as usize);
        *pos += 3;
        Some(len)
    } else {
        None
    }
}


fn parse_element<'a>(data: &'a [u8], pos: &mut usize) -> Option<(u8, &'a [u8])> {
    if *pos >= data.len() {
        return None;
    }
    
    let tag = data[*pos];
    *pos += 1;
    
    let len = itw(data, pos)?;
    
    if *pos + len > data.len() {
        return None;
    }
    
    let value = &data[*pos..*pos + len];
    *pos += len;
    
    Some((tag, value))
}


#[derive(Debug)]
pub struct Certificate {
    
    pub dm: Vec<u8>,
    
    pub subject_cn: Option<String>,
    
    pub issuer_cn: Option<String>,
    
    pub evf: Option<String>,
    
    pub eve: Option<String>,
    
    pub san: Vec<String>,
    
    pub pubkey_algo: Vec<u8>,
    
    pub pubkey: Vec<u8>,
}

impl Certificate {
    
    pub fn parse(data: &[u8]) -> Option<Self> {
        let mut pos = 0;
        
        
        let (tag, cert_data) = parse_element(data, &mut pos)?;
        if tag != asn1_tags::Ha {
            return None;
        }
        
        let mut kif = 0;
        
        
        let (tag, tbs_data) = parse_element(cert_data, &mut kif)?;
        if tag != asn1_tags::Ha {
            return None;
        }
        
        
        let mut bjn = 0;
        
        
        if bjn < tbs_data.len() && tbs_data[bjn] == asn1_tags::BRK_ {
            let _ = parse_element(tbs_data, &mut bjn)?;
        }
        
        
        let _ = parse_element(tbs_data, &mut bjn)?;
        
        
        let _ = parse_element(tbs_data, &mut bjn)?;
        
        
        let (tag, issuer_data) = parse_element(tbs_data, &mut bjn)?;
        let issuer_cn = if tag == asn1_tags::Ha {
            hxl(issuer_data)
        } else {
            None
        };
        
        
        let (tag, validity_data) = parse_element(tbs_data, &mut bjn)?;
        let (evf, eve) = if tag == asn1_tags::Ha {
            nro(validity_data)
        } else {
            (None, None)
        };
        
        
        let (tag, subject_data) = parse_element(tbs_data, &mut bjn)?;
        let subject_cn = if tag == asn1_tags::Ha {
            hxl(subject_data)
        } else {
            None
        };
        
        
        let (tag, spki_data) = parse_element(tbs_data, &mut bjn)?;
        let (pubkey_algo, pubkey) = if tag == asn1_tags::Ha {
            nrf(spki_data)
        } else {
            (Vec::new(), Vec::new())
        };
        
        
        let mut san = Vec::new();
        while bjn < tbs_data.len() {
            if let Some((tag, ext_wrapper)) = parse_element(tbs_data, &mut bjn) {
                if tag == asn1_tags::BRL_ {
                    
                    let mut cjd = 0;
                    if let Some((_, extensions)) = parse_element(ext_wrapper, &mut cjd) {
                        san = nrb(extensions);
                    }
                }
            }
        }
        
        Some(Certificate {
            dm: data.to_vec(),
            subject_cn,
            issuer_cn,
            evf,
            eve,
            san,
            pubkey_algo,
            pubkey,
        })
    }
    
    
    pub fn valid_for_hostname(&self, hostname: &str) -> bool {
        
        for name in &self.san {
            if ilz(name, hostname) {
                return true;
            }
        }
        
        
        if let Some(cn) = &self.subject_cn {
            if ilz(cn, hostname) {
                return true;
            }
        }
        
        false
    }
}


fn hxl(data: &[u8]) -> Option<String> {
    
    let kum = [0x55, 0x04, 0x03];
    
    let mut pos = 0;
    while pos < data.len() {
        if let Some((tag, rdn_set)) = parse_element(data, &mut pos) {
            if tag == asn1_tags::Aox {
                let mut opi = 0;
                if let Some((_, rdn_seq)) = parse_element(rdn_set, &mut opi) {
                    let mut jes = 0;
                    if let Some((_, oid)) = parse_element(rdn_seq, &mut jes) {
                        if oid == kum {
                            if let Some((_, value)) = parse_element(rdn_seq, &mut jes) {
                                return Some(String::from_utf8_lossy(value).into_owned());
                            }
                        }
                    }
                }
            }
        } else {
            break;
        }
    }
    
    None
}


fn nro(data: &[u8]) -> (Option<String>, Option<String>) {
    let mut pos = 0;
    
    let evf = if let Some((tag, value)) = parse_element(data, &mut pos) {
        if tag == asn1_tags::BKK_ || tag == asn1_tags::AUW_ {
            Some(String::from_utf8_lossy(value).into_owned())
        } else {
            None
        }
    } else {
        None
    };
    
    let eve = if let Some((tag, value)) = parse_element(data, &mut pos) {
        if tag == asn1_tags::BKK_ || tag == asn1_tags::AUW_ {
            Some(String::from_utf8_lossy(value).into_owned())
        } else {
            None
        }
    } else {
        None
    };
    
    (evf, eve)
}


fn nrf(data: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let mut pos = 0;
    
    
    let juh = if let Some((tag, algo_seq)) = parse_element(data, &mut pos) {
        if tag == asn1_tags::Ha {
            let mut jui = 0;
            if let Some((_, oid)) = parse_element(algo_seq, &mut jui) {
                oid.to_vec()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };
    
    
    let pubkey = if let Some((tag, value)) = parse_element(data, &mut pos) {
        if tag == asn1_tags::BNV_ && !value.is_empty() {
            
            value[1..].to_vec()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };
    
    (juh, pubkey)
}


fn nrb(extensions: &[u8]) -> Vec<String> {
    
    let oka = [0x55, 0x1D, 0x11];
    let mut result = Vec::new();
    
    let mut pos = 0;
    while pos < extensions.len() {
        if let Some((tag, ext_seq)) = parse_element(extensions, &mut pos) {
            if tag == asn1_tags::Ha {
                let mut cjd = 0;
                if let Some((_, oid)) = parse_element(ext_seq, &mut cjd) {
                    if oid == oka {
                        
                        if cjd < ext_seq.len() && ext_seq[cjd] == asn1_tags::Ahh {
                            let _ = parse_element(ext_seq, &mut cjd);
                        }
                        
                        
                        if let Some((_, octet)) = parse_element(ext_seq, &mut cjd) {
                            let mut okb = 0;
                            if let Some((_, san_seq)) = parse_element(octet, &mut okb) {
                                result = nqj(san_seq);
                            }
                        }
                    }
                }
            }
        } else {
            break;
        }
    }
    
    result
}


fn nqj(data: &[u8]) -> Vec<String> {
    let mut result = Vec::new();
    let mut pos = 0;
    
    while pos < data.len() {
        let tag = data[pos];
        pos += 1;
        
        if let Some(len) = itw(data, &mut pos) {
            if pos + len <= data.len() {
                
                if tag == 0x82 {
                    let name = String::from_utf8_lossy(&data[pos..pos + len]).into_owned();
                    result.push(name);
                }
                pos += len;
            } else {
                break;
            }
        } else {
            break;
        }
    }
    
    result
}


fn ilz(pattern: &str, hostname: &str) -> bool {
    let pattern = pattern.to_lowercase();
    let hostname = hostname.to_lowercase();
    
    if pattern.starts_with("*.") {
        
        let asi = &pattern[2..];
        if let Some(dot_pos) = hostname.find('.') {
            return &hostname[dot_pos + 1..] == asi;
        }
        false
    } else {
        pattern == hostname
    }
}


pub fn nqa(data: &[u8]) -> Vec<Certificate> {
    let mut result = Vec::new();
    
    if data.len() < 7 {
        return result;
    }
    
    
    let mut pos = 4;
    
    
    if pos >= data.len() {
        return result;
    }
    let lae = data[pos] as usize;
    pos += 1 + lae;
    
    
    if pos + 3 > data.len() {
        return result;
    }
    let gfq = ((data[pos] as usize) << 16)
        | ((data[pos + 1] as usize) << 8)
        | (data[pos + 2] as usize);
    pos += 3;
    
    let dti = pos + gfq;
    
    while pos + 3 <= dti {
        
        let fld = ((data[pos] as usize) << 16)
            | ((data[pos + 1] as usize) << 8)
            | (data[pos + 2] as usize);
        pos += 3;
        
        if pos + fld > data.len() {
            break;
        }
        
        
        if let Some(cert) = Certificate::parse(&data[pos..pos + fld]) {
            result.push(cert);
        }
        pos += fld;
        
        
        if pos + 2 <= dti {
            let dpc = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
            pos += 2 + dpc;
        }
    }
    
    result
}
