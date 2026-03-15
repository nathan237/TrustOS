
















use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;


const ARR_: u32 = 0x00000001;
const ARS_: u32   = 0x00000002;
const ARU_: u32        = 0x00000003;
const ART_: u32         = 0x00000004;
const ACA_: u32         = 0x00000009;


const JZ_: u32 = 0xD00DFEED;


#[repr(C)]
pub struct FdtHeader {
    pub sj: u32,
    pub fac: u32,
    pub uxd: u32,
    pub uxc: u32,
    pub uxe: u32,
    pub dk: u32,
    pub uce: u32,
    pub qqv: u32,
    pub wpb: u32,
    pub wpc: u32,
}


#[derive(Debug, Clone)]
pub struct Bfb {
    
    pub path: String,
    
    pub bjp: String,
    
    pub cbi: u64,
    
    pub pbi: u64,
    
    pub interrupts: Vec<u32>,
    
    pub status: String,
}


#[derive(Debug, Clone)]
pub struct Bqz {
    pub j: String,
    pub ar: u64,
    pub aw: u64,
    pub uuu: bool,
}


#[derive(Debug, Clone)]
pub struct Bsu {
    pub ar: u64,
    pub aw: u64,
    pub z: u32,
    pub ac: u32,
    pub oq: u32,
    pub format: String,
}


#[derive(Debug, Clone)]
pub struct ParsedDtb {
    
    pub model: String,
    
    pub bjp: Vec<String>,
    
    pub memory: Vec<(u64, u64)>,
    
    pub ibu: String,
    
    pub cnl: u64,
    
    pub ik: Vec<Bfb>,
    
    pub awt: Vec<Bqz>,
    
    pub jql: Option<Bsu>,
    
    pub dgv: u32,
    
    pub fpb: u32,
    
    pub haz: String,
}

impl ParsedDtb {
    pub fn new() -> Self {
        Self {
            model: String::new(),
            bjp: Vec::new(),
            memory: Vec::new(),
            ibu: String::new(),
            cnl: 0,
            ik: Vec::new(),
            awt: Vec::new(),
            jql: None,
            dgv: 0,
            fpb: 0,
            haz: String::new(),
        }
    }
}


unsafe fn btj(ptr: *const u8) -> u32 {
    let o = core::slice::anh(ptr, 4);
    u32::oa([o[0], o[1], o[2], o[3]])
}


unsafe fn myi(ptr: *const u8) -> u64 {
    let gd = btj(ptr) as u64;
    let hh = btj(ptr.add(4)) as u64;
    (gd << 32) | hh
}


unsafe fn exf(ptr: *const u8, am: usize) -> String {
    let mut len = 0;
    while len < am && *ptr.add(len) != 0 {
        len += 1;
    }
    let bf = core::slice::anh(ptr, len);
    String::azw(bf).bkc()
}


fn mun(l: u32) -> u32 {
    (l + 3) & !3
}


fn veg(j: &str) -> Option<u64> {
    if let Some(ikc) = j.du('@') {
        let tok = &j[ikc + 1..];
        u64::wa(tok, 16).bq()
    } else {
        None
    }
}





pub unsafe fn jis(ceg: *const u8) -> Option<ParsedDtb> {
    if ceg.abq() {
        return None;
    }

    
    let sj = btj(ceg);
    if sj != JZ_ {
        return None;
    }

    let fac = btj(ceg.add(4));
    let lpv = btj(ceg.add(8));
    let osd = btj(ceg.add(12));

    
    if fac > 16 * 1024 * 1024 || lpv >= fac || osd >= fac {
        return None;
    }

    let ibz = ceg.add(lpv as usize);
    let wvc = ceg.add(osd as usize);

    let mut result = ParsedDtb::new();
    result.dgv = fac;

    
    let mut l: u32 = 0;
    let llb = fac - lpv;
    let mut goy: Vec<String> = Vec::new();
    let mut rp = String::new();

    
    let mut ipv = String::new();
    let mut dfi: u64 = 0;
    let mut dpi: u64 = 0;
    let mut kmp = String::from("okay");
    let mut kmn: Vec<u32> = Vec::new();
    let mut ldv = false;
    let mut odv = false;
    let mut izq = false;
    let mut esl = false;

    
    let mut jpm: u64 = 0;
    let mut jpn: u64 = 0;
    let mut mfg: u32 = 0;
    let mut mff: u32 = 0;
    let mut pju: u32 = 0;
    let mut pjt = String::new();

    
    let mut jzl: u32 = 2;
    let mut pld: u32 = 1;

    while l + 4 <= llb {
        let bat = btj(ibz.add(l as usize));
        l += 4;

        match bat {
            ARR_ => {
                let j = exf(ibz.add(l as usize), 256);
                let baf = j.len() as u32 + 1; 
                l = mun(l + baf);

                
                if j.is_empty() {
                    rp = String::from("/");
                } else {
                    if rp == "/" {
                        rp = format!("/{}", j);
                    } else {
                        rp = format!("{}/{}", rp, j);
                    }
                }
                goy.push(rp.clone());
                result.fpb += 1;

                
                ipv = String::new();
                dfi = 0;
                dpi = 0;
                kmp = String::from("okay");
                kmn = Vec::new();

                
                ldv = j.cj("memory");
                izq = j == "chosen";
                odv = j == "reserved-memory" || rp.contains("/reserved-memory/");
                esl = j.contains("framebuffer") || j.contains("simple-framebuffer");
            }

            ARS_ => {
                
                let txf = dfi != 0 || dpi != 0;

                if ldv && (dfi != 0 || dpi != 0) {
                    result.memory.push((dfi, dpi));
                } else if odv && !rp.pp("reserved-memory") && dfi != 0 {
                    result.awt.push(Bqz {
                        j: goy.qv().abn().age(),
                        ar: dfi,
                        aw: dpi,
                        uuu: false, 
                    });
                } else if esl {
                    if jpm == 0 { jpm = dfi; }
                    if jpn == 0 { jpn = dpi; }
                    if mfg > 0 && mff > 0 {
                        result.jql = Some(Bsu {
                            ar: jpm,
                            aw: jpn,
                            z: mfg,
                            ac: mff,
                            oq: pju,
                            format: pjt.clone(),
                        });
                    }
                    esl = false;
                } else if txf && !ipv.is_empty() {
                    result.ik.push(Bfb {
                        path: rp.clone(),
                        bjp: ipv.clone(),
                        cbi: dfi,
                        pbi: dpi,
                        interrupts: kmn.clone(),
                        status: kmp.clone(),
                    });
                }

                
                goy.pop();
                rp = goy.qv().abn().unwrap_or(String::from("/"));
                ldv = false;
                izq = rp.contains("chosen");
            }

            ARU_ => {
                if l + 8 > llb { break; }
                let brt = btj(ibz.add(l as usize));
                let lnj = btj(ibz.add(l as usize + 4));
                l += 8;

                let vnh = exf(wvc.add(lnj as usize), 128);
                let bwd = ibz.add(l as usize);
                l = mun(l + brt);

                
                match vnh.as_str() {
                    "model" if goy.len() <= 1 => {
                        result.model = exf(bwd, brt as usize);
                    }
                    "compatible" => {
                        let rmw = exf(bwd, brt as usize);
                        if goy.len() <= 1 {
                            
                            let bf = core::slice::anh(bwd, brt as usize);
                            for jj in bf.adk(|&o| o == 0) {
                                if !jj.is_empty() {
                                    result.bjp.push(String::azw(jj).bkc());
                                }
                            }
                        }
                        ipv = rmw;
                    }
                    "reg" => {
                        
                        if brt >= 4 {
                            if jzl == 2 && brt >= 8 {
                                dfi = myi(bwd);
                            } else {
                                dfi = btj(bwd) as u64;
                            }

                            let cmv = (jzl * 4) as usize;
                            if (cmv + 4) <= brt as usize {
                                if pld == 2 && (cmv + 8) <= brt as usize {
                                    dpi = myi(bwd.add(cmv));
                                } else {
                                    dpi = btj(bwd.add(cmv)) as u64;
                                }
                            }
                        }

                        
                        if esl {
                            jpm = dfi;
                            jpn = dpi;
                        }
                    }
                    "#address-cells" => {
                        if brt >= 4 {
                            jzl = btj(bwd);
                        }
                    }
                    "#size-cells" => {
                        if brt >= 4 {
                            pld = btj(bwd);
                        }
                    }
                    "status" => {
                        kmp = exf(bwd, brt as usize);
                    }
                    "interrupts" | "interrupts-extended" => {
                        let az = brt / 4;
                        for a in 0..az {
                            kmn.push(btj(bwd.add(a as usize * 4)));
                        }
                    }
                    "stdout-path" if izq => {
                        result.ibu = exf(bwd, brt as usize);
                        
                        
                        if let Some(ag) = veg(&result.ibu) {
                            result.cnl = ag;
                        }
                    }
                    "bootargs" if izq => {
                        result.haz = exf(bwd, brt as usize);
                    }
                    
                    "width" if esl => {
                        if brt >= 4 { mfg = btj(bwd); }
                    }
                    "height" if esl => {
                        if brt >= 4 { mff = btj(bwd); }
                    }
                    "stride" if esl => {
                        if brt >= 4 { pju = btj(bwd); }
                    }
                    "format" if esl => {
                        pjt = exf(bwd, brt as usize);
                    }
                    _ => {}
                }
            }

            ART_ => {  }
            ACA_ => break,
            _ => {
                
                break;
            }
        }

        
        if l > llb {
            break;
        }
    }

    Some(result)
}


pub fn nvp(azq: &ParsedDtb) -> String {
    let mut bd = String::new();

    bd.t("\x01C== Device Tree Blob (DTB) Report ==\x01W\n\n");
    bd.t(&format!("Model: {}\n", if azq.model.is_empty() { "(unknown)" } else { &azq.model }));

    if !azq.bjp.is_empty() {
        bd.t("Compatible: ");
        for (a, r) in azq.bjp.iter().cf() {
            if a > 0 { bd.t(", "); }
            bd.t(r);
        }
        bd.push('\n');
    }

    bd.t(&format!("DTB size: {} bytes  |  {} nodes parsed\n", azq.dgv, azq.fpb));

    if !azq.haz.is_empty() {
        bd.t(&format!("Bootargs: {}\n", azq.haz));
    }
    if !azq.ibu.is_empty() {
        bd.t(&format!("Console: {} (UART @ 0x{:X})\n", azq.ibu, azq.cnl));
    }

    
    if !azq.memory.is_empty() {
        bd.t("\n\x01Y--- Physical Memory ---\x01W\n");
        for (ar, aw) in &azq.memory {
            let csm = aw / (1024 * 1024);
            bd.t(&format!("  0x{:010X} - 0x{:010X}  ({} MB)\n", ar, ar + aw, csm));
        }
    }

    
    if !azq.awt.is_empty() {
        bd.t("\n\x01R--- Reserved Memory (Firmware / TrustZone) ---\x01W\n");
        for m in &azq.awt {
            let cfv = m.aw / 1024;
            bd.t(&format!("  0x{:010X} - 0x{:010X}  ({:>6} KB) {}\n",
                m.ar, m.ar + m.aw, cfv, m.j));
        }
        bd.t(&format!("  Total reserved: {} regions\n", azq.awt.len()));
    }

    
    if let Some(ref bxc) = azq.jql {
        bd.t("\n\x01G--- SimpleFB Framebuffer ---\x01W\n");
        bd.t(&format!("  Base: 0x{:010X}  Size: {} bytes\n", bxc.ar, bxc.aw));
        bd.t(&format!("  Resolution: {}x{}  Stride: {}  Format: {}\n",
            bxc.z, bxc.ac, bxc.oq, bxc.format));
    }

    
    if !azq.ik.is_empty() {
        bd.t("\n\x01Y--- Discovered Peripherals ---\x01W\n");
        bd.t(&format!("{:<40} {:<14} {:<10} {}\n",
            "PATH", "REG BASE", "SIZE", "COMPATIBLE"));
        bd.t(&format!("{}\n", "-".afd(90)));

        for ba in &azq.ik {
            let mhn = match ba.status.as_str() {
                "okay" | "ok" => "\x01G",
                "disabled" => "\x01R",
                _ => "\x01Y",
            };
            bd.t(&format!("{}{:<40}\x01W 0x{:010X}  {:<10} {}\n",
                mhn,
                if ba.path.len() > 39 { &ba.path[ba.path.len()-39..] } else { &ba.path },
                ba.cbi,
                format!("0x{:X}", ba.pbi),
                ba.bjp));
        }
        bd.t(&format!("\nTotal: {} devices ({} enabled)\n",
            azq.ik.len(),
            azq.ik.iter().hi(|bc| bc.status == "okay" || bc.status == "ok").az()));
    }

    bd
}
