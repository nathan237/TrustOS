




use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use alloc::collections::BTreeMap;



pub const BFS_: u32 = 0;
pub const CTD_: u32 = 1;
pub const BFU_: u32 = 2;
pub const CTE_: u32 = 3;
pub const AHW_: u32 = 4;
pub const CSZ_: u32 = 5;
pub const BFQ_: u32 = 6;
pub const CTC_: u32 = 7;
pub const CTB_: u32 = 8;
pub const BFT_: u32 = 9;
pub const BFR_: u32 = 11;
pub const CTA_: u32 = 14;
pub const CSV_: u32 = 15;
pub const CSW_: u32 = 0x6FFFFFF6;
pub const CSY_: u32 = 0x6FFFFFFF;
pub const CSX_: u32 = 0x6FFFFFFE;



pub const CSU_: u64 = 0x1;
pub const BFO_: u64 = 0x2;
pub const BFP_: u64 = 0x4;
pub const CSS_: u64 = 0x10;
pub const CST_: u64 = 0x20;



pub const CUH_: u8 = 0;
pub const CUG_: u8 = 1;
pub const CUI_: u8 = 2;

pub const CUP_: u8 = 0;
pub const BGN_: u8 = 1;
pub const BGM_: u8 = 2;
pub const CUQ_: u8 = 3;
pub const CUO_: u8 = 4;



pub const IU_: u32 = 1;
pub const AGT_: u32 = 2;
pub const WL_: u32 = 3;
pub const BDR_: u32 = 4;
pub const CMM_: u32 = 0x6474E550;
pub const BDQ_: u32 = 0x6474E551;
pub const BDP_: u32 = 0x6474E552;



pub const SU_: i64 = 0;
pub const ST_: i64 = 1;
pub const AQU_: i64 = 3;
pub const AQT_: i64 = 4;
pub const ABO_: i64 = 5;
pub const ABP_: i64 = 6;
pub const ABK_: i64 = 7;
pub const ABM_: i64 = 8;
pub const ABL_: i64 = 9;
pub const ABN_: i64 = 10;
pub const ABI_: i64 = 12;
pub const ABG_: i64 = 13;
pub const AQW_: i64 = 14;
pub const AQV_: i64 = 15;
pub const BSS_: i64 = 16;
pub const ABJ_: i64 = 23;
pub const BSJ_: i64 = 24;
pub const BSR_: i64 = 29;
pub const ABH_: i64 = 30;
pub const BSN_: i64 = 0x6FFFFEF5;
pub const BST_: i64 = 0x6FFFFFFE;
pub const BSU_: i64 = 0x6FFFFFFF;




#[derive(Debug, Clone)]
pub struct Ga {
    
    pub index: usize,
    
    pub j: String,
    
    pub dbx: u32,
    
    pub flags: u64,
    
    pub ag: u64,
    
    pub l: u64,
    
    pub aw: u64,
    
    pub arl: u32,
    
    pub co: u32,
    
    pub mub: u64,
    
    pub isx: u64,
}

impl Ga {
    pub fn ddc(&self) -> &'static str {
        match self.dbx {
            BFS_ => "NULL",
            CTD_ => "PROGBITS",
            BFU_ => "SYMTAB",
            CTE_ => "STRTAB",
            AHW_ => "RELA",
            CSZ_ => "HASH",
            BFQ_ => "DYNAMIC",
            CTC_ => "NOTE",
            CTB_ => "NOBITS",
            BFT_ => "REL",
            BFR_ => "DYNSYM",
            CTA_ => "INIT_ARRAY",
            CSV_ => "FINI_ARRAY",
            CSW_ => "GNU_HASH",
            CSY_ => "GNU_VERSYM",
            CSX_ => "GNU_VERNEED",
            _ => "UNKNOWN",
        }
    }

    pub fn kwm(&self) -> String {
        let mut e = String::new();
        if self.flags & CSU_ != 0 { e.push('W'); }
        if self.flags & BFO_ != 0 { e.push('A'); }
        if self.flags & BFP_ != 0 { e.push('X'); }
        if self.flags & CSS_ != 0 { e.push('M'); }
        if self.flags & CST_ != 0 { e.push('S'); }
        if e.is_empty() { e.push('-'); }
        e
    }

    
    pub fn clc(&self) -> bool {
        self.flags & BFP_ != 0
    }

    
    pub fn lfv(&self) -> bool {
        self.flags & BFO_ != 0
    }
}


#[derive(Debug, Clone)]
pub struct Go {
    
    pub j: String,
    
    pub bn: u64,
    
    pub aw: u64,
    
    pub gtt: u8,
    
    pub kdk: u8,
    
    pub pyi: u8,
    
    pub phd: u16,
}

impl Go {
    pub fn ddc(&self) -> &'static str {
        match self.gtt {
            CUP_ => "NOTYPE",
            BGN_ => "OBJECT",
            BGM_ => "FUNC",
            CUQ_ => "SECTION",
            CUO_ => "FILE",
            _ => "UNKNOWN",
        }
    }

    pub fn qpo(&self) -> &'static str {
        match self.kdk {
            CUH_ => "LOCAL",
            CUG_ => "GLOBAL",
            CUI_ => "WEAK",
            _ => "?",
        }
    }

    
    pub fn txm(&self) -> bool {
        self.gtt == BGM_
    }

    
    pub fn yzt(&self) -> bool {
        self.gtt == BGN_
    }

    
    pub fn ofu(&self) -> bool {
        self.phd != 0 && self.bn != 0
    }
}


#[derive(Debug, Clone)]
pub struct Qm {
    
    pub bku: u32,
    
    pub flags: u32,
    
    pub l: u64,
    
    pub uy: u64,
    
    pub otp: u64,
    
    pub hjh: u64,
    
    pub jfv: u64,
    
    pub align: u64,
}

impl Qm {
    pub fn ddc(&self) -> &'static str {
        match self.bku {
            IU_ => "LOAD",
            AGT_ => "DYNAMIC",
            WL_ => "INTERP",
            BDR_ => "NOTE",
            CMM_ => "GNU_EH_FRAME",
            BDQ_ => "GNU_STACK",
            BDP_ => "GNU_RELRO",
            6 => "PHDR",
            _ => "UNKNOWN",
        }
    }

    pub fn kwm(&self) -> String {
        let mut e = String::new();
        e.push(if self.flags & 4 != 0 { 'R' } else { '-' });
        e.push(if self.flags & 2 != 0 { 'W' } else { '-' });
        e.push(if self.flags & 1 != 0 { 'X' } else { '-' });
        e
    }
}


#[derive(Debug, Clone)]
pub struct Abg {
    pub ll: i64,
    pub bn: u64,
}

impl Abg {
    pub fn zqx(&self) -> &'static str {
        match self.ll {
            SU_ => "NULL",
            ST_ => "NEEDED",
            AQU_ => "PLTGOT",
            AQT_ => "HASH",
            ABO_ => "STRTAB",
            ABP_ => "SYMTAB",
            ABK_ => "RELA",
            ABM_ => "RELASZ",
            ABL_ => "RELAENT",
            ABN_ => "STRSZ",
            ABI_ => "INIT",
            ABG_ => "FINI",
            AQW_ => "SONAME",
            AQV_ => "RPATH",
            ABJ_ => "JMPREL",
            BSJ_ => "BIND_NOW",
            BSR_ => "RUNPATH",
            ABH_ => "FLAGS",
            BSN_ => "GNU_HASH",
            BST_ => "VERNEED",
            BSU_ => "VERNEEDNUM",
            _ => "?",
        }
    }
}


#[derive(Debug, Clone)]
pub struct Aef {
    
    pub l: u64,
    
    pub hyf: u32,
    
    pub jrx: u32,
    
    pub fcn: i64,
    
    pub ezj: String,
}

impl Aef {
    pub fn ddc(&self) -> &'static str {
        match self.hyf {
            0 => "R_X86_64_NONE",
            1 => "R_X86_64_64",
            2 => "R_X86_64_PC32",
            5 => "R_X86_64_COPY",
            6 => "R_X86_64_GLOB_DAT",
            7 => "R_X86_64_JUMP_SLOT",
            8 => "R_X86_64_RELATIVE",
            10 => "R_X86_64_32",
            11 => "R_X86_64_32S",
            _ => "R_X86_64_?",
        }
    }
}


#[derive(Debug, Clone)]
pub struct Abr {
    
    pub l: u64,
    
    pub uy: Option<u64>,
    
    pub ca: String,
    
    pub ava: String,
}




#[derive(Debug, Clone)]
pub struct Aro {
    
    pub class: &'static str,
    
    pub f: &'static str,
    
    pub uzl: &'static str,
    
    pub gfz: &'static str,
    
    pub czk: &'static str,
    
    pub bt: u64,
    
    pub ltu: u64,
    
    pub jpz: u64,
    
    pub ltt: u16,
    
    pub jpy: u16,
    
    pub jqe: u16,
    
    pub yy: usize,
}




#[derive(Debug)]
pub struct Abq {
    
    pub co: Aro,
    
    pub dku: Vec<Qm>,
    
    pub aeo: Vec<Ga>,
    
    pub bot: Vec<Go>,
    
    pub dqj: Vec<Go>,
    
    pub hhn: Vec<Abg>,
    
    pub bwp: Vec<Aef>,
    
    pub pd: Vec<Abr>,
    
    pub interpreter: Option<String>,
    
    pub djt: Vec<String>,
    
    pub blw: BTreeMap<u64, String>,
    
    ozm: usize,
}

impl Abq {
    
    pub fn zma(&self, j: &str) -> Option<&Ga> {
        self.aeo.iter().du(|e| e.j == j)
    }

    
    pub fn ioq(&self) -> Vec<&Ga> {
        self.aeo.iter().hi(|e| e.clc()).collect()
    }

    
    pub fn ajb(&self) -> Vec<&Go> {
        let mut hks: Vec<&Go> = self.bot.iter()
            .rh(self.dqj.iter())
            .hi(|e| e.txm() && e.ofu())
            .collect();
        hks.bxf(|e| e.bn);
        hks.ruy(|e| e.bn);
        hks
    }

    
    pub fn zql(&self, ag: u64) -> Option<&str> {
        self.blw.get(&ag).map(|e| e.as_str())
    }

    
    pub fn wfz(&self, ag: u64) -> Option<&Ga> {
        self.aeo.iter().du(|e| {
            e.lfv() && ag >= e.ag && ag < e.ag + e.aw
        })
    }

    
    pub fn yy(&self) -> usize {
        self.ozm
    }
}




pub fn vcd(f: &[u8]) -> Result<Abq, &'static str> {
    if f.len() < 64 {
        return Err("File too small for ELF header");
    }
    if &f[0..4] != b"\x7FELF" {
        return Err("Not an ELF file (bad magic)");
    }

    
    let class = f[4];
    if class != 2 {
        return Err("Not a 64-bit ELF");
    }
    let njl = f[5];
    if njl != 1 {
        return Err("Not little-endian (unsupported)");
    }

    let ceh = fap(f, 16);
    let cqb = fap(f, 18);
    let cxe = biq(f, 24);
    let epo = biq(f, 32);
    let ksl = biq(f, 40);
    let fhh = fap(f, 54);
    let dqk = fap(f, 56);
    let nov = fap(f, 58);
    let ksk = fap(f, 60);
    let ksm = fap(f, 62);

    let co = Aro {
        class: "ELF64",
        f: if njl == 1 { "Little Endian" } else { "Big Endian" },
        uzl: match f[7] {
            0 => "UNIX System V",
            3 => "Linux",
            _ => "Unknown",
        },
        gfz: match ceh {
            1 => "REL (Relocatable)",
            2 => "EXEC (Executable)",
            3 => "DYN (Shared/PIE)",
            4 => "CORE (Core dump)",
            _ => "Unknown",
        },
        czk: match cqb {
            62 => "x86-64",
            3 => "x86 (i386)",
            40 => "ARM",
            183 => "AArch64",
            243 => "RISC-V",
            _ => "Unknown",
        },
        bt: cxe,
        ltu: epo,
        jpz: ksl,
        ltt: dqk,
        jpy: ksk,
        jqe: ksm,
        yy: f.len(),
    };

    
    let mut dku = Vec::new();
    let mut interpreter = None;

    for a in 0..dqk as usize {
        let dz = epo as usize + a * fhh as usize;
        if dz + 56 > f.len() { break; }

        let bku = fxs(f, dz);
        let flags = fxs(f, dz + 4);
        let l = biq(f, dz + 8);
        let uy = biq(f, dz + 16);
        let otp = biq(f, dz + 24);
        let hjh = biq(f, dz + 32);
        let jfv = biq(f, dz + 40);
        let align = biq(f, dz + 48);

        
        if bku == WL_ {
            let ay = l as usize;
            let ci = (l + hjh) as usize;
            if ci <= f.len() {
                let e = &f[ay..ci];
                let len = e.iter().qf(|&o| o == 0).unwrap_or(e.len());
                interpreter = Some(String::from(core::str::jg(&e[..len]).unwrap_or("?")));
            }
        }

        dku.push(Qm {
            bku, flags, l, uy, otp, hjh, jfv, align,
        });
    }

    
    let aeo = vdn(f, ksl as usize, nov as usize, ksk as usize, ksm as usize);

    
    let bot = ouo(f, &aeo, BFU_);
    let dqj = ouo(f, &aeo, BFR_);

    
    let mut blw = BTreeMap::new();
    for aaw in bot.iter().rh(dqj.iter()) {
        if aaw.ofu() && !aaw.j.is_empty() {
            blw.insert(aaw.bn, aaw.j.clone());
        }
    }

    
    let (hhn, djt) = lsi(f, &aeo);

    
    let bwp = vdh(f, &aeo, &dqj);

    
    let pd = sqc(f, &aeo, &dku);

    Ok(Abq {
        co,
        dku,
        aeo,
        bot,
        dqj,
        hhn,
        bwp,
        pd,
        interpreter,
        djt,
        blw,
        ozm: f.len(),
    })
}



fn vdn(f: &[u8], jpz: usize, wmt: usize, jpy: usize, jqe: usize) -> Vec<Ga> {
    if jpz == 0 || jpy == 0 {
        return Vec::new();
    }

    
    let mut jle: Vec<(u32, u32, u64, u64, u64, u64, u32, u32, u64, u64)> = Vec::new();
    for a in 0..jpy {
        let dz = jpz + a * wmt;
        if dz + 64 > f.len() { break; }

        jle.push((
            fxs(f, dz),           
            fxs(f, dz + 4),       
            biq(f, dz + 8),       
            biq(f, dz + 16),      
            biq(f, dz + 24),      
            biq(f, dz + 32),      
            fxs(f, dz + 40),      
            fxs(f, dz + 44),      
            biq(f, dz + 48),      
            biq(f, dz + 56),      
        ));
    }

    
    let wns = if jqe < jle.len() {
        let (_, _, _, _, ppd, mhy, _, _, _, _) = jle[jqe];
        let ay = ppd as usize;
        let ci = (ppd + mhy) as usize;
        if ci <= f.len() { Some(&f[ay..ci]) } else { None }
    } else {
        None
    };

    
    let mut aeo = Vec::new();
    for (a, &(pjv, dbx, flags, ag, l, aw, arl, co, mub, isx)) in jle.iter().cf() {
        let j = if let Some(ezd) = wns {
            lxt(ezd, pjv as usize)
        } else {
            format!("section_{}", a)
        };

        aeo.push(Ga {
            index: a,
            j,
            dbx,
            flags,
            ag,
            l,
            aw,
            arl,
            co,
            mub,
            isx,
        });
    }

    aeo
}

fn ouo(f: &[u8], aeo: &[Ga], gud: u32) -> Vec<Go> {
    let mut bot = Vec::new();

    
    let gtu = match aeo.iter().du(|e| e.dbx == gud) {
        Some(e) => e,
        None => return bot,
    };

    
    let ppe = if (gtu.arl as usize) < aeo.len() {
        &aeo[gtu.arl as usize]
    } else {
        return bot;
    };

    let iby = ppe.l as usize;
    let ppc = iby + ppe.aw as usize;
    if ppc > f.len() { return bot; }
    let ezd = &f[iby..ppc];

    let www = gtu.l as usize;
    let mil = if gtu.isx > 0 {
        gtu.aw / gtu.isx
    } else {
        0
    };

    for a in 0..mil as usize {
        let dz = www + a * 24; 
        if dz + 24 > f.len() { break; }

        let pnm = fxs(f, dz);
        let mhf = f[dz + 4];
        let pno = f[dz + 5];
        let pnp = fap(f, dz + 6);
        let pnr = biq(f, dz + 8);
        let gsz = biq(f, dz + 16);

        let j = lxt(ezd, pnm as usize);
        let gtt = mhf & 0x0F;
        let kdk = mhf >> 4;
        let pyi = pno & 0x03;

        bot.push(Go {
            j,
            bn: pnr,
            aw: gsz,
            gtt,
            kdk,
            pyi,
            phd: pnp,
        });
    }

    bot
}

fn lsi(f: &[u8], aeo: &[Ga]) -> (Vec<Abg>, Vec<String>) {
    let mut ch = Vec::new();
    let mut djt = Vec::new();

    let ise = match aeo.iter().du(|e| e.dbx == BFQ_) {
        Some(e) => e,
        None => return (ch, djt),
    };

    
    let shz = if (ise.arl as usize) < aeo.len() {
        let e = &aeo[ise.arl as usize];
        let ay = e.l as usize;
        let ci = ay + e.aw as usize;
        if ci <= f.len() { Some(&f[ay..ci]) } else { None }
    } else {
        None
    };

    let ay = ise.l as usize;
    let az = ise.aw as usize / 16; 

    for a in 0..az {
        let dz = ay + a * 16;
        if dz + 16 > f.len() { break; }

        let ll = ocu(f, dz);
        let bn = biq(f, dz + 8);

        if ll == SU_ { break; }

        
        if ll == ST_ {
            if let Some(ezd) = shz {
                let j = lxt(ezd, bn as usize);
                djt.push(j);
            }
        }

        ch.push(Abg { ll, bn });
    }

    (ch, djt)
}

fn vdh(f: &[u8], aeo: &[Ga], nor: &[Go]) -> Vec<Aef> {
    let mut bwp = Vec::new();

    for ava in aeo.iter() {
        if ava.dbx != AHW_ && ava.dbx != BFT_ {
            continue;
        }

        let ay = ava.l as usize;
        let ogs = ava.dbx == AHW_;
        let acy = if ogs { 24 } else { 16 };
        let az = ava.aw as usize / acy;

        for a in 0..az {
            let dz = ay + a * acy;
            if dz + acy > f.len() { break; }

            let jla = biq(f, dz);
            let hwk = biq(f, dz + 8);
            let jky = if ogs { ocu(f, dz + 16) } else { 0 };

            let jrx = (hwk >> 32) as u32;
            let hyf = (hwk & 0xFFFFFFFF) as u32;

            let ezj = if (jrx as usize) < nor.len() {
                nor[jrx as usize].j.clone()
            } else {
                String::new()
            };

            bwp.push(Aef {
                l: jla,
                hyf,
                jrx,
                fcn: jky,
                ezj,
            });
        }
    }

    bwp
}

fn sqc(f: &[u8], aeo: &[Ga], dku: &[Qm]) -> Vec<Abr> {
    let mut pd = Vec::new();

    
    for ava in aeo.iter() {
        if ava.dbx == BFS_ || ava.aw == 0 {
            continue;
        }

        let ay = ava.l as usize;
        let ci = ay + ava.aw as usize;
        if ci > f.len() { continue; }

        let wfy = &f[ay..ci];
        let mut cv = String::new();
        let mut ibw = 0usize;

        for (a, &o) in wfy.iter().cf() {
            if o >= 0x20 && o < 0x7F {
                if cv.is_empty() {
                    ibw = a;
                }
                cv.push(o as char);
            } else {
                if cv.len() >= 4 {
                    let azv = ay + ibw;
                    
                    let uy = if ava.lfv() && ava.ag > 0 {
                        Some(ava.ag + ibw as u64)
                    } else {
                        None
                    };

                    pd.push(Abr {
                        l: azv as u64,
                        uy,
                        ca: cv.clone(),
                        ava: ava.j.clone(),
                    });
                }
                cv.clear();
            }
        }

        
        if cv.len() >= 4 {
            let azv = ay + ibw;
            let uy = if ava.lfv() && ava.ag > 0 {
                Some(ava.ag + ibw as u64)
            } else {
                None
            };
            pd.push(Abr {
                l: azv as u64,
                uy,
                ca: cv,
                ava: ava.j.clone(),
            });
        }
    }

    
    pd.bxf(|e| e.l);
    pd.ruy(|e| e.l);

    pd
}



fn fap(f: &[u8], dz: usize) -> u16 {
    if dz + 2 > f.len() { return 0; }
    u16::dj([f[dz], f[dz + 1]])
}

fn fxs(f: &[u8], dz: usize) -> u32 {
    if dz + 4 > f.len() { return 0; }
    u32::dj([f[dz], f[dz + 1], f[dz + 2], f[dz + 3]])
}

fn biq(f: &[u8], dz: usize) -> u64 {
    if dz + 8 > f.len() { return 0; }
    u64::dj([
        f[dz], f[dz + 1], f[dz + 2], f[dz + 3],
        f[dz + 4], f[dz + 5], f[dz + 6], f[dz + 7],
    ])
}

fn ocu(f: &[u8], dz: usize) -> i64 {
    biq(f, dz) as i64
}

fn lxt(ezd: &[u8], l: usize) -> String {
    if l >= ezd.len() {
        return String::new();
    }
    let bf = &ezd[l..];
    let len = bf.iter().qf(|&o| o == 0).unwrap_or(bf.len());
    String::from(core::str::jg(&bf[..len]).unwrap_or(""))
}
