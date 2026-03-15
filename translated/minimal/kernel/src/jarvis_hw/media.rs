













use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;





#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryFormat {
    Wl,
    Sg,
    Adr,       
    Alc,       
    Avg,    
    Cdp, 
    Asd,   
    Bgi,  
    Bnn,  
    Wu,        
    Nm,        
    F,
}

impl BinaryFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            BinaryFormat::Wl => "ELF32",
            BinaryFormat::Sg => "ELF64",
            BinaryFormat::Adr => "PE32",
            BinaryFormat::Alc => "PE64",
            BinaryFormat::Avg => "Mach-O 64",
            BinaryFormat::Cdp => "Flat Binary",
            BinaryFormat::Asd => "FAT Filesystem",
            BinaryFormat::Bgi => "ext4 Filesystem",
            BinaryFormat::Bnn => "NTFS Filesystem",
            BinaryFormat::Wu => "GPT Partition Table",
            BinaryFormat::Nm => "MBR Partition Table",
            BinaryFormat::F => "Unknown",
        }
    }
}


pub fn hfz(f: &[u8]) -> BinaryFormat {
    if f.len() < 16 { return BinaryFormat::F; }

    
    if f[0] == 0x7F && f[1] == b'E' && f[2] == b'L' && f[3] == b'F' {
        return if f[4] == 2 { BinaryFormat::Sg } else { BinaryFormat::Wl };
    }

    
    if f[0] == b'M' && f[1] == b'Z' && f.len() >= 64 {
        
        let jja = u32::dj([f[0x3C], f[0x3D], f[0x3E], f[0x3F]]) as usize;
        if jja + 6 < f.len() && f[jja] == b'P' && f[jja + 1] == b'E' {
            
            let lqu = jja + 24;
            if lqu + 2 <= f.len() {
                let uyw = u16::dj([f[lqu], f[lqu + 1]]);
                return if uyw == 0x020B { BinaryFormat::Alc } else { BinaryFormat::Adr };
            }
            return BinaryFormat::Adr;
        }
    }

    
    if f.len() >= 4 {
        let sj = u32::dj([f[0], f[1], f[2], f[3]]);
        if sj == 0xFEEDFACF || sj == 0xCFFAEDFE {
            return BinaryFormat::Avg;
        }
    }

    
    if f.len() >= 520 && &f[512..520] == b"EFI PART" {
        return BinaryFormat::Wu;
    }

    
    if f.len() >= 512 && f[510] == 0x55 && f[511] == 0xAA {
        
        if f.len() >= 62 && (f[54..62] == *b"FAT12   " || f[54..62] == *b"FAT16   ") {
            return BinaryFormat::Asd;
        }
        if f.len() >= 90 && f[82..90] == *b"FAT32   " {
            return BinaryFormat::Asd;
        }
        return BinaryFormat::Nm;
    }

    
    if f.len() >= 11 && &f[3..11] == b"NTFS    " {
        return BinaryFormat::Bnn;
    }

    
    if f.len() >= 1082 {
        let sps = u16::dj([f[1080], f[1081]]);
        if sps == 0xEF53 {
            return BinaryFormat::Bgi;
        }
    }

    BinaryFormat::F
}





#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryArch {
    Bbc,
    BT_,
    Bbx,
    Fg,
    Brb,
    Jy,
    Bmh,
    Acz,
    Aod,
    F,
}

impl BinaryArch {
    pub fn as_str(&self) -> &'static str {
        match self {
            BinaryArch::Bbc => "x86",
            BinaryArch::BT_ => "x86_64",
            BinaryArch::Bbx => "ARM32",
            BinaryArch::Fg => "AArch64",
            BinaryArch::Brb => "RISC-V 32",
            BinaryArch::Jy => "RISC-V 64",
            BinaryArch::Bmh => "MIPS32",
            BinaryArch::Acz => "MIPS64",
            BinaryArch::Aod => "WebAssembly",
            BinaryArch::F => "Unknown",
        }
    }
}


pub fn rwm(f: &[u8]) -> BinaryArch {
    let format = hfz(f);

    match format {
        BinaryFormat::Wl | BinaryFormat::Sg => {
            if f.len() < 20 { return BinaryArch::F; }
            let cqb = u16::dj([f[18], f[19]]);
            match cqb {
                0x03 => BinaryArch::Bbc,
                0x3E => BinaryArch::BT_,
                0x28 => BinaryArch::Bbx,
                0xB7 => BinaryArch::Fg,
                0xF3 => if f[4] == 2 { BinaryArch::Jy } else { BinaryArch::Brb },
                0x08 => BinaryArch::Bmh,
                _ => BinaryArch::F,
            }
        }
        BinaryFormat::Adr => BinaryArch::Bbc,
        BinaryFormat::Alc => BinaryArch::BT_,
        BinaryFormat::Avg => {
            if f.len() >= 8 {
                let rpy = u32::dj([f[4], f[5], f[6], f[7]]);
                match rpy {
                    0x01000007 | 7 => BinaryArch::BT_,
                    0x0100000C | 12 => BinaryArch::Fg,
                    _ => BinaryArch::F,
                }
            } else {
                BinaryArch::F
            }
        }
        _ => BinaryArch::F,
    }
}






#[derive(Clone)]
pub struct Rm {
    pub format: BinaryFormat,
    pub arch: BinaryArch,
    pub afz: usize,
    
    pub aeo: Vec<Ayl>,
    
    pub jsg: Vec<String>,
    
    pub jam: Vec<String>,
    
    pub mmv: bool,
    
    pub mbi: String,
    
    pub mdc: Vec<String>,
}

#[derive(Clone)]
pub struct Ayl {
    pub j: String,
    pub l: usize,
    pub aw: usize,
    pub flags: String,
}


pub fn qhw(f: &[u8]) -> Rm {
    let format = hfz(f);
    let arch = rwm(f);
    let aeo = sqh(f, format);
    let pd = kut(f);
    let security = qks(f, format, arch);

    
    let (mmv, wbp, apd) = match format {
        BinaryFormat::Sg | BinaryFormat::Wl => {
            xmv(f)
        }
        _ => (false, String::new(), Vec::new()),
    };

    Rm {
        format,
        arch,
        afz: f.len(),
        aeo,
        jsg: apd,
        jam: pd,
        mmv,
        mbi: wbp,
        mdc: security,
    }
}


fn sqh(f: &[u8], format: BinaryFormat) -> Vec<Ayl> {
    let mut aeo = Vec::new();

    match format {
        BinaryFormat::Sg => {
            if f.len() < 64 { return aeo; }

            
            let pjx = u64::dj([
                f[40], f[41], f[42], f[43],
                f[44], f[45], f[46], f[47],
            ]) as usize;

            
            let mfh = u16::dj([f[58], f[59]]) as usize;
            
            let pjw = u16::dj([f[60], f[61]]) as usize;

            if pjx == 0 || mfh < 64 || pjw > 100 { return aeo; }

            for a in 0..pjw.v(50) {
                let ar = pjx + a * mfh;
                if ar + 64 > f.len() { break; }

                let dbx = u32::dj([
                    f[ar + 4], f[ar + 5], f[ar + 6], f[ar + 7]]);
                let jpp = u64::dj([
                    f[ar + 8], f[ar + 9], f[ar + 10], f[ar + 11],
                    f[ar + 12], f[ar + 13], f[ar + 14], f[ar + 15]]);
                let pjy = u64::dj([
                    f[ar + 24], f[ar + 25], f[ar + 26], f[ar + 27],
                    f[ar + 28], f[ar + 29], f[ar + 30], f[ar + 31]]) as usize;
                let pjz = u64::dj([
                    f[ar + 32], f[ar + 33], f[ar + 34], f[ar + 35],
                    f[ar + 36], f[ar + 37], f[ar + 38], f[ar + 39]]) as usize;

                let bde = match dbx {
                    0 => continue, 
                    1 => "PROGBITS",
                    2 => "SYMTAB",
                    3 => "STRTAB",
                    4 => "RELA",
                    5 => "HASH",
                    6 => "DYNAMIC",
                    7 => "NOTE",
                    8 => "NOBITS",
                    _ => "OTHER",
                };

                let mut ghe = String::new();
                if jpp & 1 != 0 { ghe.push('W'); }
                if jpp & 2 != 0 { ghe.push('A'); }
                if jpp & 4 != 0 { ghe.push('X'); }

                aeo.push(Ayl {
                    j: String::from(bde),
                    l: pjy,
                    aw: pjz,
                    flags: ghe,
                });
            }
        }
        _ => {}
    }

    aeo
}


fn kut(f: &[u8]) -> Vec<String> {
    let mut pd = Vec::new();
    let mut cv = String::new();

    for &hf in f.iter().take(64 * 1024) { 
        if hf >= 0x20 && hf < 0x7F {
            cv.push(hf as char);
        } else {
            if cv.len() >= 6 {
                
                let pb = cv.avd();
                let tvp = pb.contains("http")
                    || pb.contains("password")
                    || pb.contains("key")
                    || pb.contains("token")
                    || pb.contains("secret")
                    || pb.contains("root")
                    || pb.contains("admin")
                    || pb.contains("linux")
                    || pb.contains("android")
                    || pb.contains("error")
                    || pb.contains("/dev/")
                    || pb.contains("/proc/")
                    || pb.contains("/sys/")
                    || pb.contains(".so")
                    || pb.contains(".dll")
                    || (cv.len() >= 20); 

                if tvp && pd.len() < 50 {
                    pd.push(cv.clone());
                }
            }
            cv.clear();
        }
    }

    pd
}


fn qks(f: &[u8], format: BinaryFormat, arch: BinaryArch) -> Vec<String> {
    let mut ts = Vec::new();

    match format {
        BinaryFormat::Sg | BinaryFormat::Wl => {
            
            if tmk(f) {
                ts.push(String::from("WARN: Executable stack detected (NX disabled)"));
            }

            
            if !tnc(f) {
                ts.push(String::from("NOTE: No RELRO — GOT/PLT writable"));
            }

            
            let tni = f.ee(4).any(|d| {
                d == [0x02, 0x00, 0x00, 0x00] 
            });
            if !tni {
                ts.push(String::from("INFO: Likely stripped (no symbol table)"));
            }
        }
        BinaryFormat::Adr | BinaryFormat::Alc => {
            ts.push(format!("PE binary ({})", arch.as_str()));
            
        }
        _ => {}
    }

    ts
}

fn tmk(f: &[u8]) -> bool {
    
    if f.len() < 64 { return false; }
    let abt = u64::dj([
        f[32], f[33], f[34], f[35],
        f[36], f[37], f[38], f[39],
    ]) as usize;
    let ovf = u16::dj([f[54], f[55]]) as usize;
    let vhd = u16::dj([f[56], f[57]]) as usize;

    if abt == 0 || ovf < 56 { return false; }

    for a in 0..vhd.v(20) {
        let ar = abt + a * ovf;
        if ar + 8 > f.len() { break; }
        let bku = u32::dj([f[ar], f[ar+1], f[ar+2], f[ar+3]]);
        if bku == 0x6474E551 { 
            let bvv = u32::dj([f[ar+4], f[ar+5], f[ar+6], f[ar+7]]);
            return bvv & 1 != 0; 
        }
    }
    false
}

fn tnc(iia: &[u8]) -> bool {
    
    
    true 
}






fn xmv(f: &[u8]) -> (bool, String, Vec<String>) {
    
    match crate::riscv_translator::pvz(f) {
        Ok(disasm) => {
            let mut apd = Vec::new();

            
            for line in disasm.ak() {
                if line.contains("ECALL") || line.contains("SYSCALL") || line.contains("SVC") {
                    apd.push(String::from(line.em()));
                }
            }

            
            let hvz: String = disasm.ak()
                .take(40)
                .collect::<Vec<&str>>()
                .rr("\n");

            (true, hvz, apd)
        }
        Err(_) => (false, String::new(), Vec::new()),
    }
}






#[derive(Clone)]
pub struct Ya {
    pub index: u8,
    pub kk: String,
    pub aag: u64,
    pub fuv: u64,
    pub aga: u64,
    pub cji: bool,
}


pub fn vde(f: &[u8]) -> Vec<Ya> {
    let mut ek = Vec::new();

    if f.len() < 512 { return ek; }

    
    if f.len() >= 1024 && &f[512..520] == b"EFI PART" {
        lsk(f, &mut ek);
    }
    
    else if f[510] == 0x55 && f[511] == 0xAA {
        lsp(f, &mut ek);
    }

    ek
}

fn lsp(f: &[u8], ek: &mut Vec<Ya>) {
    
    for a in 0..4u8 {
        let ar = 446 + a as usize * 16;
        if ar + 16 > f.len() { break; }

        let status = f[ar];
        let frq = f[ar + 4];
        let aag = u32::dj([
            f[ar + 8], f[ar + 9], f[ar + 10], f[ar + 11]
        ]) as u64;
        let fuv = u32::dj([
            f[ar + 12], f[ar + 13], f[ar + 14], f[ar + 15]
        ]) as u64;

        if frq == 0 || fuv == 0 { continue; }

        let kk = match frq {
            0x01 => "FAT12",
            0x04 | 0x06 | 0x0E => "FAT16",
            0x0B | 0x0C => "FAT32",
            0x07 => "NTFS/exFAT",
            0x82 => "Linux Swap",
            0x83 => "Linux",
            0xEE => "GPT Protective",
            0xEF => "EFI System",
            _ => "Unknown",
        };

        ek.push(Ya {
            index: a,
            kk: String::from(kk),
            aag,
            fuv,
            aga: fuv * 512 / (1024 * 1024),
            cji: status == 0x80,
        });
    }
}

fn lsk(f: &[u8], ek: &mut Vec<Ya>) {
    if f.len() < 1024 { return; }

    
    let htd = u32::dj([f[592], f[593], f[594], f[595]]) as usize;
    let acy = u32::dj([f[596], f[597], f[598], f[599]]) as usize;
    let ktr = u64::dj([
        f[584], f[585], f[586], f[587],
        f[588], f[589], f[590], f[591],
    ]);

    let slx = (ktr * 512) as usize;
    if acy < 128 { return; }

    for a in 0..htd.v(32) {
        let ar = slx + a * acy;
        if ar + 128 > f.len() { break; }

        
        let fxq = &f[ar..ar + 16];
        if fxq.iter().xx(|&o| o == 0) { continue; }

        let aag = u64::dj([
            f[ar + 32], f[ar + 33], f[ar + 34], f[ar + 35],
            f[ar + 36], f[ar + 37], f[ar + 38], f[ar + 39],
        ]);
        let fhr = u64::dj([
            f[ar + 40], f[ar + 41], f[ar + 42], f[ar + 43],
            f[ar + 44], f[ar + 45], f[ar + 46], f[ar + 47],
        ]);

        let fuv = fhr.ao(aag) + 1;

        
        let kk = trl(fxq);

        ek.push(Ya {
            index: a as u8,
            kk,
            aag,
            fuv,
            aga: fuv * 512 / (1024 * 1024),
            cji: false,
        });
    }
}

fn trl(aar: &[u8]) -> String {
    
    
    if aar[0] == 0x28 && aar[1] == 0x73 && aar[2] == 0x2A && aar[3] == 0xC1 {
        return String::from("EFI System");
    }
    
    if aar[0] == 0xAF && aar[1] == 0x3D && aar[2] == 0xC6 && aar[3] == 0x0F {
        return String::from("Linux");
    }
    
    if aar[0] == 0x6D && aar[1] == 0xFD && aar[2] == 0x57 && aar[3] == 0x06 {
        return String::from("Linux Swap");
    }
    
    if aar[0] == 0xA2 && aar[1] == 0xA0 && aar[2] == 0xD0 && aar[3] == 0xEB {
        return String::from("Microsoft Basic Data");
    }
    String::from("Unknown")
}





impl Rm {
    pub fn fix(&self) -> String {
        let mut e = String::new();

        e.t("\x01C╔══════════════════════════════════════════════════════════╗\n");
        e.t("║         JARVIS Binary Intelligence Report                ║\n");
        e.t("╚══════════════════════════════════════════════════════════╝\x01W\n\n");

        e.t(&format!("\x01Y[Format]\x01W {} ({})\n", self.format.as_str(), self.arch.as_str()));
        e.t(&format!("\x01Y[Size]\x01W {} bytes ({} KB)\n\n", self.afz, self.afz / 1024));

        
        if !self.aeo.is_empty() {
            e.t("\x01Y[Sections]\x01W\n");
            for zw in &self.aeo {
                e.t(&format!("  {:12} off=0x{:08X} size=0x{:06X} [{}]\n",
                    zw.j, zw.l, zw.aw, zw.flags));
            }
            e.push('\n');
        }

        
        if self.mmv {
            e.t("\x01G[RISC-V Translation]\x01W OK — binary decoded into universal IR\n");
            if !self.jsg.is_empty() {
                e.t(&format!("  Syscalls detected: {}\n", self.jsg.len()));
                for jt in self.jsg.iter().take(10) {
                    e.t(&format!("    {}\n", jt));
                }
            }
            if !self.mbi.is_empty() {
                e.t("\n\x01C  --- Disassembly Preview ---\x01W\n");
                for line in self.mbi.ak().take(20) {
                    e.t(&format!("  {}\n", line));
                }
                e.push('\n');
            }
        } else if oh!(self.format, BinaryFormat::Wl | BinaryFormat::Sg) {
            e.t("\x01R[RISC-V Translation]\x01W Failed — unsupported arch or corrupted\n\n");
        }

        
        if !self.jam.is_empty() {
            e.t(&format!("\x01Y[Interesting Strings]\x01W ({} found)\n", self.jam.len()));
            for apc in self.jam.iter().take(15) {
                e.t(&format!("  \"{}\"\n", apc));
            }
            e.push('\n');
        }

        
        if !self.mdc.is_empty() {
            e.t("\x01Y[Security Assessment]\x01W\n");
            for jp in &self.mdc {
                e.t(&format!("  {}\n", jp));
            }
        }

        e
    }
}


pub fn svx(ek: &[Ya]) -> String {
    let mut e = String::new();
    e.t("\x01C═══ Partition Table ═══\x01W\n");
    for ai in ek {
        e.t(&format!("  #{}: {} — start=LBA {} size={} MB {}\n",
            ai.index, ai.kk, ai.aag, ai.aga,
            if ai.cji { "[BOOT]" } else { "" }));
    }
    e
}
