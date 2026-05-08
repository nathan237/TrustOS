
















use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;


const ATT_: u32 = 0x00000001;
const ATU_: u32   = 0x00000002;
const ATW_: u32        = 0x00000003;
const ATV_: u32         = 0x00000004;
const ADQ_: u32         = 0x00000009;


const KT_: u32 = 0xD00DFEED;


#[repr(C)]
pub struct FdtHeader {
    pub magic: u32,
    pub totalsize: u32,
    pub off_dt_struct: u32,
    pub off_dt_strings: u32,
    pub off_mem_rsvmap: u32,
    pub version: u32,
    pub last_comp_version: u32,
    pub boot_cpuid_phys: u32,
    pub size_dt_strings: u32,
    pub size_dt_struct: u32,
}


#[derive(Debug, Clone)]
pub struct Xz {
    
    pub path: String,
    
    pub compatible: String,
    
    pub reg_base: u64,
    
    pub reg_size: u64,
    
    pub interrupts: Vec<u32>,
    
    pub status: String,
}


#[derive(Debug, Clone)]
pub struct Adn {
    pub name: String,
    pub base: u64,
    pub size: u64,
    pub no_map: bool,
}


#[derive(Debug, Clone)]
pub struct Aew {
    pub base: u64,
    pub size: u64,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub format: String,
}


#[derive(Debug, Clone)]
pub struct ParsedDtb {
    
    pub model: String,
    
    pub compatible: Vec<String>,
    
    pub memory: Vec<(u64, u64)>,
    
    pub stdout_path: String,
    
    pub uart_base: u64,
    
    pub devices: Vec<Xz>,
    
    pub reserved: Vec<Adn>,
    
    pub simplefb: Option<Aew>,
    
    pub dtb_size: u32,
    
    pub node_count: u32,
    
    pub bootargs: String,
}

impl ParsedDtb {
    pub fn new() -> Self {
        Self {
            model: String::new(),
            compatible: Vec::new(),
            memory: Vec::new(),
            stdout_path: String::new(),
            uart_base: 0,
            devices: Vec::new(),
            reserved: Vec::new(),
            simplefb: None,
            dtb_size: 0,
            node_count: 0,
            bootargs: String::new(),
        }
    }
}


unsafe fn aky(ptr: *const u8) -> u32 {
    let b = core::slice::from_raw_parts(ptr, 4);
    u32::from_be_bytes([b[0], b[1], b[2], b[3]])
}


unsafe fn hha(ptr: *const u8) -> u64 {
    let hi = aky(ptr) as u64;
    let lo = aky(ptr.add(4)) as u64;
    (hi << 32) | lo
}


unsafe fn cdb(ptr: *const u8, max: usize) -> String {
    let mut len = 0;
    while len < max && *ptr.add(len) != 0 {
        len += 1;
    }
    let bytes = core::slice::from_raw_parts(ptr, len);
    String::from_utf8_lossy(bytes).into_owned()
}


fn hek(offset: u32) -> u32 {
    (offset + 3) & !3
}


fn nrl(name: &str) -> Option<u64> {
    if let Some(at_pos) = name.find('@') {
        let mlb = &name[at_pos + 1..];
        u64::from_str_radix(mlb, 16).ok()
    } else {
        None
    }
}





pub unsafe fn ewg(dtb_ptr: *const u8) -> Option<ParsedDtb> {
    if dtb_ptr.is_null() {
        return None;
    }

    
    let magic = aky(dtb_ptr);
    if magic != KT_ {
        return None;
    }

    let totalsize = aky(dtb_ptr.add(4));
    let gkl = aky(dtb_ptr.add(8));
    let irz = aky(dtb_ptr.add(12));

    
    if totalsize > 16 * 1024 * 1024 || gkl >= totalsize || irz >= totalsize {
        return None;
    }

    let ean = dtb_ptr.add(gkl as usize);
    let oye = dtb_ptr.add(irz as usize);

    let mut result = ParsedDtb::new();
    result.dtb_size = totalsize;

    
    let mut offset: u32 = 0;
    let ggy = totalsize - gkl;
    let mut dci: Vec<String> = Vec::new();
    let mut ht = String::new();

    
    let mut ejh = String::new();
    let mut bfq: u64 = 0;
    let mut blk: u64 = 0;
    let mut fpl = String::from("okay");
    let mut fpj: Vec<u32> = Vec::new();
    let mut gcg = false;
    let mut ige = false;
    let mut eqh = false;
    let mut cas = false;

    
    let mut fak: u64 = 0;
    let mut fal: u64 = 0;
    let mut gup: u32 = 0;
    let mut guo: u32 = 0;
    let mut jfu: u32 = 0;
    let mut jft = String::new();

    
    let mut fgg: u32 = 2;
    let mut jgl: u32 = 1;

    while offset + 4 <= ggy {
        let abm = aky(ean.add(offset as usize));
        offset += 4;

        match abm {
            ATT_ => {
                let name = cdb(ean.add(offset as usize), 256);
                let name_len = name.len() as u32 + 1; 
                offset = hek(offset + name_len);

                
                if name.is_empty() {
                    ht = String::from("/");
                } else {
                    if ht == "/" {
                        ht = format!("/{}", name);
                    } else {
                        ht = format!("{}/{}", ht, name);
                    }
                }
                dci.push(ht.clone());
                result.node_count += 1;

                
                ejh = String::new();
                bfq = 0;
                blk = 0;
                fpl = String::from("okay");
                fpj = Vec::new();

                
                gcg = name.starts_with("memory");
                eqh = name == "chosen";
                ige = name == "reserved-memory" || ht.contains("/reserved-memory/");
                cas = name.contains("framebuffer") || name.contains("simple-framebuffer");
            }

            ATU_ => {
                
                let msi = bfq != 0 || blk != 0;

                if gcg && (bfq != 0 || blk != 0) {
                    result.memory.push((bfq, blk));
                } else if ige && !ht.ends_with("reserved-memory") && bfq != 0 {
                    result.reserved.push(Adn {
                        name: dci.last().cloned().unwrap_or_default(),
                        base: bfq,
                        size: blk,
                        no_map: false, 
                    });
                } else if cas {
                    if fak == 0 { fak = bfq; }
                    if fal == 0 { fal = blk; }
                    if gup > 0 && guo > 0 {
                        result.simplefb = Some(Aew {
                            base: fak,
                            size: fal,
                            width: gup,
                            height: guo,
                            stride: jfu,
                            format: jft.clone(),
                        });
                    }
                    cas = false;
                } else if msi && !ejh.is_empty() {
                    result.devices.push(Xz {
                        path: ht.clone(),
                        compatible: ejh.clone(),
                        reg_base: bfq,
                        reg_size: blk,
                        interrupts: fpj.clone(),
                        status: fpl.clone(),
                    });
                }

                
                dci.pop();
                ht = dci.last().cloned().unwrap_or(String::from("/"));
                gcg = false;
                eqh = ht.contains("chosen");
            }

            ATW_ => {
                if offset + 8 > ggy { break; }
                let aki = aky(ean.add(offset as usize));
                let gio = aky(ean.add(offset as usize + 4));
                offset += 8;

                let nyw = cdb(oye.add(gio as usize), 128);
                let ami = ean.add(offset as usize);
                offset = hek(offset + aki);

                
                match nyw.as_str() {
                    "model" if dci.len() <= 1 => {
                        result.model = cdb(ami, aki as usize);
                    }
                    "compatible" => {
                        let kwc = cdb(ami, aki as usize);
                        if dci.len() <= 1 {
                            
                            let bytes = core::slice::from_raw_parts(ami, aki as usize);
                            for df in bytes.split(|&b| b == 0) {
                                if !df.is_empty() {
                                    result.compatible.push(String::from_utf8_lossy(df).into_owned());
                                }
                            }
                        }
                        ejh = kwc;
                    }
                    "reg" => {
                        
                        if aki >= 4 {
                            if fgg == 2 && aki >= 8 {
                                bfq = hha(ami);
                            } else {
                                bfq = aky(ami) as u64;
                            }

                            let avc = (fgg * 4) as usize;
                            if (avc + 4) <= aki as usize {
                                if jgl == 2 && (avc + 8) <= aki as usize {
                                    blk = hha(ami.add(avc));
                                } else {
                                    blk = aky(ami.add(avc)) as u64;
                                }
                            }
                        }

                        
                        if cas {
                            fak = bfq;
                            fal = blk;
                        }
                    }
                    "#address-cells" => {
                        if aki >= 4 {
                            fgg = aky(ami);
                        }
                    }
                    "#size-cells" => {
                        if aki >= 4 {
                            jgl = aky(ami);
                        }
                    }
                    "status" => {
                        fpl = cdb(ami, aki as usize);
                    }
                    "interrupts" | "interrupts-extended" => {
                        let count = aki / 4;
                        for i in 0..count {
                            fpj.push(aky(ami.add(i as usize * 4)));
                        }
                    }
                    "stdout-path" if eqh => {
                        result.stdout_path = cdb(ami, aki as usize);
                        
                        
                        if let Some(addr) = nrl(&result.stdout_path) {
                            result.uart_base = addr;
                        }
                    }
                    "bootargs" if eqh => {
                        result.bootargs = cdb(ami, aki as usize);
                    }
                    
                    "width" if cas => {
                        if aki >= 4 { gup = aky(ami); }
                    }
                    "height" if cas => {
                        if aki >= 4 { guo = aky(ami); }
                    }
                    "stride" if cas => {
                        if aki >= 4 { jfu = aky(ami); }
                    }
                    "format" if cas => {
                        jft = cdb(ami, aki as usize);
                    }
                    _ => {}
                }
            }

            ATV_ => {  }
            ADQ_ => break,
            _ => {
                
                break;
            }
        }

        
        if offset > ggy {
            break;
        }
    }

    Some(result)
}


pub fn hzo(dtb: &ParsedDtb) -> String {
    let mut out = String::new();

    out.push_str("\x01C== Device Tree Blob (DTB) Report ==\x01W\n\n");
    out.push_str(&format!("Model: {}\n", if dtb.model.is_empty() { "(unknown)" } else { &dtb.model }));

    if !dtb.compatible.is_empty() {
        out.push_str("Compatible: ");
        for (i, c) in dtb.compatible.iter().enumerate() {
            if i > 0 { out.push_str(", "); }
            out.push_str(c);
        }
        out.push('\n');
    }

    out.push_str(&format!("DTB size: {} bytes  |  {} nodes parsed\n", dtb.dtb_size, dtb.node_count));

    if !dtb.bootargs.is_empty() {
        out.push_str(&format!("Bootargs: {}\n", dtb.bootargs));
    }
    if !dtb.stdout_path.is_empty() {
        out.push_str(&format!("Console: {} (UART @ 0x{:X})\n", dtb.stdout_path, dtb.uart_base));
    }

    
    if !dtb.memory.is_empty() {
        out.push_str("\n\x01Y--- Physical Memory ---\x01W\n");
        for (base, size) in &dtb.memory {
            let aop = size / (1024 * 1024);
            out.push_str(&format!("  0x{:010X} - 0x{:010X}  ({} MB)\n", base, base + size, aop));
        }
    }

    
    if !dtb.reserved.is_empty() {
        out.push_str("\n\x01R--- Reserved Memory (Firmware / TrustZone) ---\x01W\n");
        for r in &dtb.reserved {
            let arh = r.size / 1024;
            out.push_str(&format!("  0x{:010X} - 0x{:010X}  ({:>6} KB) {}\n",
                r.base, r.base + r.size, arh, r.name));
        }
        out.push_str(&format!("  Total reserved: {} regions\n", dtb.reserved.len()));
    }

    
    if let Some(ref sfb) = dtb.simplefb {
        out.push_str("\n\x01G--- SimpleFB Framebuffer ---\x01W\n");
        out.push_str(&format!("  Base: 0x{:010X}  Size: {} bytes\n", sfb.base, sfb.size));
        out.push_str(&format!("  Resolution: {}x{}  Stride: {}  Format: {}\n",
            sfb.width, sfb.height, sfb.stride, sfb.format));
    }

    
    if !dtb.devices.is_empty() {
        out.push_str("\n\x01Y--- Discovered Peripherals ---\x01W\n");
        out.push_str(&format!("{:<40} {:<14} {:<10} {}\n",
            "PATH", "REG BASE", "SIZE", "COMPATIBLE"));
        out.push_str(&format!("{}\n", "-".repeat(90)));

        for s in &dtb.devices {
            let gwf = match s.status.as_str() {
                "okay" | "ok" => "\x01G",
                "disabled" => "\x01R",
                _ => "\x01Y",
            };
            out.push_str(&format!("{}{:<40}\x01W 0x{:010X}  {:<10} {}\n",
                gwf,
                if s.path.len() > 39 { &s.path[s.path.len()-39..] } else { &s.path },
                s.reg_base,
                format!("0x{:X}", s.reg_size),
                s.compatible));
        }
        out.push_str(&format!("\nTotal: {} devices ({} enabled)\n",
            dtb.devices.len(),
            dtb.devices.iter().filter(|d| d.status == "okay" || d.status == "ok").count()));
    }

    out
}
