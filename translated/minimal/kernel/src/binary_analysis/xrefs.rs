




use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::format;

use super::disasm::Dc;




#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XrefType {
    
    En,
    
    Nh,
    
    Ahd,
    
    Aaw,
}


#[derive(Debug, Clone)]
pub struct Lc {
    
    pub from: u64,
    
    pub wh: u64,
    
    pub dnl: XrefType,
}




#[derive(Debug, Clone)]
pub struct Sb {
    
    pub bt: u64,
    
    pub ci: u64,
    
    pub j: String,
    
    pub jak: usize,
    
    pub imr: Vec<u64>,
    
    pub imq: Vec<u64>,
    
    pub ikx: usize,
}




#[derive(Debug)]
pub struct XrefDatabase {
    
    pub xrefs: Vec<Lc>,
    
    pub hxh: BTreeMap<u64, Vec<Lc>>,
    
    pub ehj: BTreeMap<u64, Vec<Lc>>,
    
    pub ajb: Vec<Sb>,
    
    pub hkt: BTreeMap<u64, usize>,
}

impl XrefDatabase {
    
    pub fn qsy(
        instructions: &[Dc],
        blw: &BTreeMap<u64, String>,
    ) -> Self {
        let mut xrefs = Vec::new();
        let mut hxh: BTreeMap<u64, Vec<Lc>> = BTreeMap::new();
        let mut ehj: BTreeMap<u64, Vec<Lc>> = BTreeMap::new();

        
        for fi in instructions {
            if let Some(cd) = fi.ena {
                let xwt = if fi.etc {
                    XrefType::En
                } else if fi.etd {
                    XrefType::Ahd
                } else if fi.etg {
                    XrefType::Nh
                } else {
                    XrefType::Aaw
                };

                let bta = Lc {
                    from: fi.re,
                    wh: cd,
                    dnl: xwt,
                };

                xrefs.push(bta.clone());
                hxh.bt(fi.re).clq(Vec::new).push(bta.clone());
                ehj.bt(cd).clq(Vec::new).push(bta);
            }

            
            if fi.bes == "lea" && fi.bvs.contains("rip") {
                
                if let Some(ag) = sqg(&fi.bvs, fi.re, fi.bf.len() as u64) {
                    let bta = Lc {
                        from: fi.re,
                        wh: ag,
                        dnl: XrefType::Aaw,
                    };
                    xrefs.push(bta.clone());
                    hxh.bt(fi.re).clq(Vec::new).push(bta.clone());
                    ehj.bt(ag).clq(Vec::new).push(bta);
                }
            }
        }

        
        let ajb = rwq(instructions, &ehj, blw);

        
        let mut hkt = BTreeMap::new();
        for (a, bb) in ajb.iter().cf() {
            hkt.insert(bb.bt, a);
        }

        Self {
            xrefs,
            hxh,
            ehj,
            ajb,
            hkt,
        }
    }

    
    pub fn ihw(&self, ag: u64) -> &[Lc] {
        self.ehj.get(&ag).map(|p| p.gai()).unwrap_or(&[])
    }

    
    pub fn gxa(&self, ag: u64) -> &[Lc] {
        self.hxh.get(&ag).map(|p| p.gai()).unwrap_or(&[])
    }

    
    pub fn szk(&self, ag: u64) -> Option<&Sb> {
        
        for ke in self.ajb.iter().vv() {
            if ag >= ke.bt && ag < ke.ci {
                return Some(ke);
            }
        }
        None
    }

    
    pub fn txn(&self, ag: u64) -> bool {
        self.hkt.bgm(&ag)
    }

    
    pub fn ytg(&self, bt: u64) -> Option<&Sb> {
        self.hkt.get(&bt)
            .and_then(|&w| self.ajb.get(w))
    }

    
    pub fn ztk(&self) -> usize {
        self.xrefs.len()
    }

    
    pub fn yhg(&self) -> usize {
        self.xrefs.iter().hi(|b| b.dnl == XrefType::En).az()
    }

    
    pub fn awz(&self) -> String {
        let kgf = self.xrefs.iter().hi(|b| b.dnl == XrefType::En).az();
        let uav = self.xrefs.iter().hi(|b| b.dnl == XrefType::Nh || b.dnl == XrefType::Ahd).az();
        let f = self.xrefs.iter().hi(|b| b.dnl == XrefType::Aaw).az();

        format!(
            "Xrefs: {} total ({} calls, {} jumps, {} data) | {} functions detected",
            self.xrefs.len(), kgf, uav, f, self.ajb.len()
        )
    }
}



fn rwq(
    instructions: &[Dc],
    ehj: &BTreeMap<u64, Vec<Lc>>,
    blw: &BTreeMap<u64, String>,
) -> Vec<Sb> {
    if instructions.is_empty() {
        return Vec::new();
    }

    
    let mut ch: BTreeMap<u64, String> = BTreeMap::new();

    
    for (ag, xrefs) in ehj.iter() {
        for bta in xrefs {
            if bta.dnl == XrefType::En {
                let j = blw.get(ag)
                    .abn()
                    .unwrap_or_else(|| format!("sub_{:x}", ag));
                ch.insert(*ag, j);
                break;
            }
        }
    }

    
    for (ag, j) in blw {
        ch.bt(*ag).clq(|| j.clone());
    }

    
    let nur = instructions[0].re;
    ch.bt(nur).clq(|| {
        blw.get(&nur)
            .abn()
            .unwrap_or_else(|| String::from("_start"))
    });

    
    let mgl: Vec<(u64, String)> = ch.dse().collect();

    
    let mua: BTreeMap<u64, usize> = instructions.iter()
        .cf()
        .map(|(a, fi)| (fi.re, a))
        .collect();

    
    let mut ajb = Vec::new();
    for (a, (bt, j)) in mgl.iter().cf() {
        
        let dlz = match mua.get(bt) {
            Some(&w) => w,
            None => continue, 
        };

        
        let ktk = if a + 1 < mgl.len() {
            let uud = mgl[a + 1].0;
            mua.get(&uud).hu().unwrap_or(instructions.len())
        } else {
            instructions.len()
        };

        if dlz >= ktk { continue; }

        let iwa = &instructions[dlz..ktk];
        let sll = iwa.qv()
            .map(|fi| fi.re + fi.bf.len() as u64)
            .unwrap_or(*bt);

        
        let imr: Vec<u64> = iwa.iter()
            .hi(|fi| fi.etc)
            .kwb(|fi| fi.ena)
            .collect();

        
        let imq: Vec<u64> = ehj.get(bt)
            .map(|xrefs| xrefs.iter()
                .hi(|b| b.dnl == XrefType::En)
                .map(|b| b.from)
                .collect())
            .age();

        
        let ikx = 1 + iwa.iter()
            .hi(|fi| fi.etg || fi.etd || fi.edy)
            .az();

        ajb.push(Sb {
            bt: *bt,
            ci: sll,
            j: j.clone(),
            jak: iwa.len(),
            imr,
            imq,
            ikx,
        });
    }

    ajb.bxf(|bb| bb.bt);
    ajb
}




fn sqg(bvr: &str, dik: u64, tvg: u64) -> Option<u64> {
    
    let mai = bvr.du("rip")?;
    let muk = &bvr[mai + 3..];

    let hsu = dik + tvg; 

    if let Some(kr) = muk.blj('+') {
        
        let ci = kr.du(']').unwrap_or(kr.len());
        let nu = kr[..ci].em();
        let l = if nu.cj("0x") {
            u64::wa(&nu[2..], 16).bq()?
        } else {
            nu.parse::<u64>().bq()?
        };
        Some(hsu + l)
    } else if let Some(kr) = muk.blj('-') {
        
        let ci = kr.du(']').unwrap_or(kr.len());
        let nu = kr[..ci].em();
        let l = if nu.cj("0x") {
            u64::wa(&nu[2..], 16).bq()?
        } else {
            nu.parse::<u64>().bq()?
        };
        Some(hsu.nj(l))
    } else {
        
        Some(hsu)
    }
}
