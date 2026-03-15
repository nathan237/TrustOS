




use alloc::vec::Vec;
use alloc::string::String;


mod asn1_tags {
    pub const Byi: u8 = 0x01;
    pub const Cyt: u8 = 0x02;
    pub const BLD_: u8 = 0x03;
    pub const DXJ_: u8 = 0x04;
    pub const Dcv: u8 = 0x05;
    pub const Ddi: u8 = 0x06;
    pub const EJS_: u8 = 0x0C;
    pub const EBC_: u8 = 0x13;
    pub const DQA_: u8 = 0x16;
    pub const BID_: u8 = 0x17;
    pub const ASS_: u8 = 0x18;
    pub const Qp: u8 = 0x30;
    pub const Cli: u8 = 0x31;
    pub const BOT_: u8 = 0xA0;
    pub const DGQ_: u8 = 0xA1;
    pub const DGR_: u8 = 0xA2;
    pub const BOU_: u8 = 0xA3;
}


fn ouj(f: &[u8], u: &mut usize) -> Option<usize> {
    if *u >= f.len() {
        return None;
    }
    
    let fv = f[*u];
    *u += 1;
    
    if fv < 0x80 {
        Some(fv as usize)
    } else if fv == 0x81 {
        if *u >= f.len() {
            return None;
        }
        let len = f[*u] as usize;
        *u += 1;
        Some(len)
    } else if fv == 0x82 {
        if *u + 1 >= f.len() {
            return None;
        }
        let len = ((f[*u] as usize) << 8) | (f[*u + 1] as usize);
        *u += 2;
        Some(len)
    } else if fv == 0x83 {
        if *u + 2 >= f.len() {
            return None;
        }
        let len = ((f[*u] as usize) << 16) 
            | ((f[*u + 1] as usize) << 8) 
            | (f[*u + 2] as usize);
        *u += 3;
        Some(len)
    } else {
        None
    }
}


fn aut<'a>(f: &'a [u8], u: &mut usize) -> Option<(u8, &'a [u8])> {
    if *u >= f.len() {
        return None;
    }
    
    let ll = f[*u];
    *u += 1;
    
    let len = ouj(f, u)?;
    
    if *u + len > f.len() {
        return None;
    }
    
    let bn = &f[*u..*u + len];
    *u += len;
    
    Some((ll, bn))
}


#[derive(Debug)]
pub struct Certificate {
    
    pub js: Vec<u8>,
    
    pub icc: Option<String>,
    
    pub lgr: Option<String>,
    
    pub jhg: Option<String>,
    
    pub jhf: Option<String>,
    
    pub grg: Vec<String>,
    
    pub lwc: Vec<u8>,
    
    pub cbd: Vec<u8>,
}

impl Certificate {
    
    pub fn parse(f: &[u8]) -> Option<Self> {
        let mut u = 0;
        
        
        let (ll, qxu) = aut(f, &mut u)?;
        if ll != asn1_tags::Qp {
            return None;
        }
        
        let mut qxv = 0;
        
        
        let (ll, dmk) = aut(qxu, &mut qxv)?;
        if ll != asn1_tags::Qp {
            return None;
        }
        
        
        let mut dml = 0;
        
        
        if dml < dmk.len() && dmk[dml] == asn1_tags::BOT_ {
            let _ = aut(dmk, &mut dml)?;
        }
        
        
        let _ = aut(dmk, &mut dml)?;
        
        
        let _ = aut(dmk, &mut dml)?;
        
        
        let (ll, tzt) = aut(dmk, &mut dml)?;
        let lgr = if ll == asn1_tags::Qp {
            nsj(tzt)
        } else {
            None
        };
        
        
        let (ll, xqn) = aut(dmk, &mut dml)?;
        let (jhg, jhf) = if ll == asn1_tags::Qp {
            vej(xqn)
        } else {
            (None, None)
        };
        
        
        let (ll, wvs) = aut(dmk, &mut dml)?;
        let icc = if ll == asn1_tags::Qp {
            nsj(wvs)
        } else {
            None
        };
        
        
        let (ll, wrf) = aut(dmk, &mut dml)?;
        let (lwc, cbd) = if ll == asn1_tags::Qp {
            vdt(wrf)
        } else {
            (Vec::new(), Vec::new())
        };
        
        
        let mut grg = Vec::new();
        while dml < dmk.len() {
            if let Some((ll, spw)) = aut(dmk, &mut dml) {
                if ll == asn1_tags::BOU_ {
                    
                    let mut fid = 0;
                    if let Some((_, itk)) = aut(spw, &mut fid) {
                        grg = vdm(itk);
                    }
                }
            }
        }
        
        Some(Certificate {
            js: f.ip(),
            icc,
            lgr,
            jhg,
            jhf,
            grg,
            lwc,
            cbd,
        })
    }
    
    
    pub fn xqk(&self, ajc: &str) -> bool {
        
        for j in &self.grg {
            if olg(j, ajc) {
                return true;
            }
        }
        
        
        if let Some(rla) = &self.icc {
            if olg(rla, ajc) {
                return true;
            }
        }
        
        false
    }
}


fn nsj(f: &[u8]) -> Option<String> {
    
    let rlb = [0x55, 0x04, 0x03];
    
    let mut u = 0;
    while u < f.len() {
        if let Some((ll, vqz)) = aut(f, &mut u) {
            if ll == asn1_tags::Cli {
                let mut wjl = 0;
                if let Some((_, ozu)) = aut(vqz, &mut wjl) {
                    let mut pid = 0;
                    if let Some((_, htm)) = aut(ozu, &mut pid) {
                        if htm == rlb {
                            if let Some((_, bn)) = aut(ozu, &mut pid) {
                                return Some(String::azw(bn).bkc());
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


fn vej(f: &[u8]) -> (Option<String>, Option<String>) {
    let mut u = 0;
    
    let jhg = if let Some((ll, bn)) = aut(f, &mut u) {
        if ll == asn1_tags::BID_ || ll == asn1_tags::ASS_ {
            Some(String::azw(bn).bkc())
        } else {
            None
        }
    } else {
        None
    };
    
    let jhf = if let Some((ll, bn)) = aut(f, &mut u) {
        if ll == asn1_tags::BID_ || ll == asn1_tags::ASS_ {
            Some(String::azw(bn).bkc())
        } else {
            None
        }
    } else {
        None
    };
    
    (jhg, jhf)
}


fn vdt(f: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let mut u = 0;
    
    
    let qgc = if let Some((ll, qge)) = aut(f, &mut u) {
        if ll == asn1_tags::Qp {
            let mut qgd = 0;
            if let Some((_, htm)) = aut(qge, &mut qgd) {
                htm.ip()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };
    
    
    let cbd = if let Some((ll, bn)) = aut(f, &mut u) {
        if ll == asn1_tags::BLD_ && !bn.is_empty() {
            
            bn[1..].ip()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };
    
    (qgc, cbd)
}


fn vdm(itk: &[u8]) -> Vec<String> {
    
    let wcn = [0x55, 0x1D, 0x11];
    let mut result = Vec::new();
    
    let mut u = 0;
    while u < itk.len() {
        if let Some((ll, hiq)) = aut(itk, &mut u) {
            if ll == asn1_tags::Qp {
                let mut fid = 0;
                if let Some((_, htm)) = aut(hiq, &mut fid) {
                    if htm == wcn {
                        
                        if fid < hiq.len() && hiq[fid] == asn1_tags::Byi {
                            let _ = aut(hiq, &mut fid);
                        }
                        
                        
                        if let Some((_, uwz)) = aut(hiq, &mut fid) {
                            let mut wco = 0;
                            if let Some((_, wcp)) = aut(uwz, &mut wco) {
                                result = vck(wcp);
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


fn vck(f: &[u8]) -> Vec<String> {
    let mut result = Vec::new();
    let mut u = 0;
    
    while u < f.len() {
        let ll = f[u];
        u += 1;
        
        if let Some(len) = ouj(f, &mut u) {
            if u + len <= f.len() {
                
                if ll == 0x82 {
                    let j = String::azw(&f[u..u + len]).bkc();
                    result.push(j);
                }
                u += len;
            } else {
                break;
            }
        } else {
            break;
        }
    }
    
    result
}


fn olg(pattern: &str, ajc: &str) -> bool {
    let pattern = pattern.aqn();
    let ajc = ajc.aqn();
    
    if pattern.cj("*.") {
        
        let cif = &pattern[2..];
        if let Some(fgw) = ajc.du('.') {
            return &ajc[fgw + 1..] == cif;
        }
        false
    } else {
        pattern == ajc
    }
}


pub fn vbz(f: &[u8]) -> Vec<Certificate> {
    let mut result = Vec::new();
    
    if f.len() < 7 {
        return result;
    }
    
    
    let mut u = 4;
    
    
    if u >= f.len() {
        return result;
    }
    let rrl = f[u] as usize;
    u += 1 + rrl;
    
    
    if u + 3 > f.len() {
        return result;
    }
    let liw = ((f[u] as usize) << 16)
        | ((f[u + 1] as usize) << 8)
        | (f[u + 2] as usize);
    u += 3;
    
    let hpx = u + liw;
    
    while u + 3 <= hpx {
        
        let kha = ((f[u] as usize) << 16)
            | ((f[u + 1] as usize) << 8)
            | (f[u + 2] as usize);
        u += 3;
        
        if u + kha > f.len() {
            break;
        }
        
        
        if let Some(qxt) = Certificate::parse(&f[u..u + kha]) {
            result.push(qxt);
        }
        u += kha;
        
        
        if u + 2 <= hpx {
            let hip = u16::oa([f[u], f[u + 1]]) as usize;
            u += 2 + hip;
        }
    }
    
    result
}
