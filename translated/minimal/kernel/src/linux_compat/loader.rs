



use alloc::vec::Vec;
use alloc::string::String;


pub struct Ajv {
    
    pub bt: u64,
    
    pub ar: u64,
    
    pub ci: u64,
    
    pub qsa: u64,
    
    pub jq: Vec<Bsk>,
    
    pub ahp: Option<String>,
}


pub struct Bsk {
    pub ag: u64,
    pub aw: u64,
    pub f: Vec<u8>,
    pub flags: u32,
}


pub fn zbd(path: &str) -> Result<Ajv, &'static str> {
    crate::serial_println!("[LOADER] Loading: {}", path);
    
    
    let f = crate::linux::rootfs::mq(path)
        .jd(|_| "file not found")?;
    
    
    if f.len() < 64 {
        return Err("file too small");
    }
    
    
    if &f[0..4] != b"\x7fELF" {
        return Err("not an ELF file");
    }
    
    
    if f[4] != 2 {
        return Err("not a 64-bit ELF");
    }
    
    
    if f[5] != 1 {
        return Err("not little-endian");
    }
    
    
    let ceh = u16::dj([f[16], f[17]]);
    let cqb = u16::dj([f[18], f[19]]);
    let cxe = u64::dj(f[24..32].try_into().unwrap());
    let epo = u64::dj(f[32..40].try_into().unwrap());
    let fhh = u16::dj([f[54], f[55]]);
    let dqk = u16::dj([f[56], f[57]]);
    
    crate::serial_println!("[LOADER] ELF type={}, machine={}, entry={:#x}, phnum={}", 
        ceh, cqb, cxe, dqk);
    
    
    if ceh != 2 && ceh != 3 {
        return Err("not an executable");
    }
    
    
    if cqb != 62 {
        return Err("not x86_64");
    }
    
    let mut jq = Vec::new();
    let mut llu = u64::O;
    let mut fns = 0u64;
    let mut ahp: Option<String> = None;
    
    
    for a in 0..dqk as usize {
        let bnu = epo as usize + a * fhh as usize;
        if bnu + 56 > f.len() {
            continue;
        }
        
        let bku = u32::dj(f[bnu..bnu+4].try_into().unwrap());
        let bvv = u32::dj(f[bnu+4..bnu+8].try_into().unwrap());
        let caz = u64::dj(f[bnu+8..bnu+16].try_into().unwrap());
        let ctg = u64::dj(f[bnu+16..bnu+24].try_into().unwrap());
        let cgh = u64::dj(f[bnu+32..bnu+40].try_into().unwrap());
        let ctf = u64::dj(f[bnu+40..bnu+48].try_into().unwrap());
        
        match bku {
            1 => {
                
                crate::serial_println!("[LOADER] LOAD segment: vaddr={:#x}, filesz={}, memsz={}", 
                    ctg, cgh, ctf);
                
                
                if ctg < llu {
                    llu = ctg;
                }
                if ctg + ctf > fns {
                    fns = ctg + ctf;
                }
                
                
                let mut hzl = alloc::vec![0u8; ctf as usize];
                let ssd = caz as usize;
                let ntm = (caz + cgh) as usize;
                
                if ntm <= f.len() {
                    hzl[..cgh as usize].dg(&f[ssd..ntm]);
                }
                
                jq.push(Bsk {
                    ag: ctg,
                    aw: ctf,
                    f: hzl,
                    flags: bvv,
                });
            }
            3 => {
                
                let ay = caz as usize;
                let ci = (caz + cgh) as usize;
                if ci <= f.len() {
                    let path = &f[ay..ci];
                    let ouu = core::str::jg(path)
                        .unwrap_or("")
                        .bdd('\0');
                    ahp = Some(String::from(ouu));
                    crate::serial_println!("[LOADER] Interpreter: {}", ouu);
                }
            }
            _ => {}
        }
    }
    
    if jq.is_empty() {
        return Err("no loadable segments");
    }
    
    
    
    let bt = if ahp.is_some() {
        crate::serial_println!("[LOADER] Warning: dynamic binary, interpreter not supported yet");
        cxe
    } else {
        cxe
    };
    
    Ok(Ajv {
        bt,
        ar: llu,
        ci: fns,
        qsa: (fns + 4095) & !4095, 
        jq,
        ahp,
    })
}
