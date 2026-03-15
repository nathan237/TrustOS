
















use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use super::{HypervisorError, Result};






const DO_: u64 = 0x7000;


const HQ_: u64 = 0x20000;


const BMO_: usize = 2048;


const OS_: u64 = 0x70000;


const IB_: u64 = 0x60000;


const FJ_: u64 = 0x100000;


const UD_: u64 = 0x1000000; 


const BYT_: u64 = 0x80000; 







#[derive(Debug, Clone)]
pub struct Bsp {
    
    pub boi: u8,
    
    pub prf: u32,
    
    pub tnx: u32,
    
    pub dk: u16,
    
    pub pwt: u8,
    
    pub eet: u8,
    
    pub ffh: u32,
    
    pub ozh: u32,
    
    pub hwp: u32,
    
    pub nec: u32,
    
    pub gjy: u32,
    
    pub hpj: u32,
    
    pub pbo: u8,
    
    pub ong: u8,
    
    pub mrr: u16,
    
    pub gjx: u32,
    
    pub gpq: u64,
}


pub const UT_: u8 = 0x01;      
pub const ZU_: u8 = 0x80;     


pub const BJF_: u16 = 0x01;   
pub const EKU_: u16 = 0x02;
pub const EKV_: u16 = 0x04;
pub const EKW_: u16 = 0x08;


#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum E820Type {
    Jw = 1,
    Nw = 2,
    Bxp = 3,
    Ddg = 4,
    Dks = 5,
}


#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Arj {
    pub ag: u64,
    pub aw: u64,
    pub avt: u32,
}






pub struct Aju {
    
    pub dh: Bsp,
    
    pub abr: Vec<u8>,
    
    pub dlq: Vec<u8>,
    
    pub gtp: bool,
    
    pub hie: u64,
}


pub fn oud(f: &[u8]) -> Result<Aju> {
    if f.len() < 0x300 {
        crate::serial_println!("[Linux] bzImage too small: {} bytes", f.len());
        return Err(HypervisorError::Xd);
    }

    
    let sj = za(f, 0x202);
    if sj != 0x53726448 { 
        crate::serial_println!("[Linux] Invalid bzImage magic: 0x{:08X} (expected 0x53726448)", sj);
        return Err(HypervisorError::Xd);
    }

    let dk = alp(f, 0x206);
    crate::serial_println!("[Linux] Boot protocol version: {}.{}", 
                          dk >> 8, dk & 0xFF);

    if dk < 0x020F {
        crate::serial_println!("[Linux] Boot protocol version too old (need >= 2.15)");
        return Err(HypervisorError::Xd);
    }

    let boi = f[0x1F1];
    let boi = if boi == 0 { 4 } else { boi }; 

    let dh = Bsp {
        boi,
        prf: za(f, 0x1F4),
        tnx: sj,
        dk,
        pwt: f[0x210],
        eet: f[0x211],
        ffh: za(f, 0x214),
        ozh: za(f, 0x218),
        hwp: za(f, 0x21C),
        nec: za(f, 0x228),
        gjy: za(f, 0x22C),
        hpj: za(f, 0x230),
        pbo: f[0x234],
        ong: f[0x235],
        mrr: alp(f, 0x236),
        gjx: za(f, 0x260),
        gpq: aqi(f, 0x258),
    };

    let gtp = (dh.mrr & BJF_) != 0;

    
    let jpl = (1 + boi as usize) * 512;
    let dlq = if jpl <= f.len() {
        f[..jpl].ip()
    } else {
        crate::serial_println!("[Linux] Warning: setup size {} > image size {}", jpl, f.len());
        f.ip()
    };

    
    let ohr = jpl;
    let abr = if ohr < f.len() {
        f[ohr..].ip()
    } else {
        crate::serial_println!("[Linux] No protected-mode kernel data!");
        return Err(HypervisorError::Xd);
    };

    
    let hie = if gtp {
        FJ_ + 0x200
    } else {
        FJ_
    };

    crate::serial_println!("[Linux] Parsed bzImage:");
    crate::serial_println!("  Setup sectors: {}", boi);
    crate::serial_println!("  Kernel size: {} bytes ({} KB)", 
                          abr.len(), abr.len() / 1024);
    crate::serial_println!("  64-bit: {}", gtp);
    crate::serial_println!("  Entry point: 0x{:X}", hie);
    crate::serial_println!("  Preferred load: 0x{:X}", dh.gpq);
    crate::serial_println!("  Init size: {} KB", dh.gjx / 1024);

    Ok(Aju {
        dh,
        abr,
        dlq,
        gtp,
        hie,
    })
}






pub struct Acq {
    
    pub wx: String,
    
    pub apy: u64,
    
    pub apw: Option<Vec<u8>>,
}

impl Default for Acq {
    fn default() -> Self {
        Self {
            wx: String::from("console=ttyS0 earlyprintk=serial,ttyS0 nokaslr"),
            apy: 256 * 1024 * 1024, 
            apw: None,
        }
    }
}


pub struct Ajt {
    
    pub mi: u64,
    
    pub ahu: u64,
    
    pub avg: u64,
    
    pub jm: u64,
    
    pub bun: u64,
}










pub fn ojt(
    fe: &mut [u8],
    acf: &Aju,
    config: &Acq,
) -> Result<Ajt> {
    let czr = fe.len() as u64;
    
    crate::serial_println!("[Linux] Loading kernel into {} MB guest memory",
                          czr / (1024 * 1024));

    
    let onk = FJ_ + acf.abr.len() as u64 + 0x100000;
    if czr < onk {
        crate::serial_println!("[Linux] Insufficient guest memory: need {} MB, have {} MB",
                              onk / (1024 * 1024), czr / (1024 * 1024));
        return Err(HypervisorError::Ns);
    }

    
    let dip = FJ_ as usize + acf.abr.len();
    if dip > fe.len() {
        return Err(HypervisorError::Ns);
    }
    fe[FJ_ as usize..dip]
        .dg(&acf.abr);
    crate::serial_println!("[Linux] Kernel loaded at 0x{:X}-0x{:X}", 
                          FJ_, dip);

    
    let dzn = config.wx.as_bytes();
    let ffd = dzn.len().v(BMO_ - 1);
    let enq = HQ_ as usize;
    fe[enq..enq + ffd]
        .dg(&dzn[..ffd]);
    fe[enq + ffd] = 0; 
    crate::serial_println!("[Linux] Command line at 0x{:X}: \"{}\"", 
                          HQ_, config.wx);

    
    mez(fe, acf, config)?;

    
    wkz(fe, czr)?;

    
    wkw(fe)?;

    
    let dll = super::acpi::esu(fe);
    
    tw(fe, DO_ as usize + 0x070, dll);
    
    
    if let Some(ref cyw) = config.apw {
        let gjz = UD_ as usize + cyw.len();
        if gjz > fe.len() {
            crate::serial_println!("[Linux] Initrd too large for guest memory");
            return Err(HypervisorError::Ns);
        }
        fe[UD_ as usize..gjz]
            .dg(cyw);
        crate::serial_println!("[Linux] Initrd loaded at 0x{:X}-0x{:X} ({} KB)",
                              UD_, gjz, cyw.len() / 1024);
    }

    Ok(Ajt {
        mi: acf.hie,
        ahu: BYT_,
        avg: DO_,
        jm: OS_,
        bun: IB_,
    })
}









fn mez(
    fe: &mut [u8],
    acf: &Aju,
    config: &Acq,
) -> Result<()> {
    let bp = DO_ as usize;
    
    
    for a in 0..4096 {
        if bp + a < fe.len() {
            fe[bp + a] = 0;
        }
    }

    
    
    let obh = 0x1F1;
    let obg = 0x290.v(acf.dlq.len());
    if obg > obh {
        let nkt = bp + 0x1F1;
        let cy = &acf.dlq[obh..obg];
        let aac = &mut fe[nkt..nkt + cy.len()];
        aac.dg(cy);
    }

    

    
    fe[bp + 0x210] = 0xFF;

    
    fe[bp + 0x211] = UT_ | ZU_;

    
    aqr(fe, bp + 0x224, 0xFE00);

    
    sx(fe, bp + 0x228, HQ_ as u32);

    
    if config.apw.is_some() {
        sx(fe, bp + 0x218, UD_ as u32);
        sx(fe, bp + 0x21C, 
                  config.apw.as_ref().unwrap().len() as u32);
    }

    
    
    fe[bp + 0x06] = 80;  
    fe[bp + 0x07] = 25;  
    fe[bp + 0x0F] = 0x22; 

    
    
    let czr = config.apy;
    let ksf = mfa(fe, bp, czr);
    fe[bp + 0x1E8] = ksf;

    crate::serial_println!("[Linux] boot_params at 0x{:X}, {} e820 entries, cmdline at 0x{:X}",
                          DO_, ksf, HQ_);

    Ok(())
}



fn mfa(fe: &mut [u8], bp: usize, czr: u64) -> u8 {
    
    
    let hho = bp + 0x2D0;
    let mut az = 0u8;

    
    dxs(fe, hho, az, 
                     0, 0x9FC00, E820Type::Jw);
    az += 1;

    
    dxs(fe, hho, az,
                     0x9FC00, 0xA0000 - 0x9FC00, E820Type::Nw);
    az += 1;

    
    dxs(fe, hho, az,
                     0x50000, 0x1000, E820Type::Bxp);
    az += 1;

    
    dxs(fe, hho, az,
                     0xA0000, 0x60000, E820Type::Nw);
    az += 1;

    
    let jeo = czr - 0x100000;
    dxs(fe, hho, az,
                     0x100000, jeo, E820Type::Jw);
    az += 1;

    az
}

fn dxs(
    mem: &mut [u8], ar: usize, index: u8,
    ag: u64, aw: u64, avt: E820Type,
) {
    let l = ar + (index as usize) * 20;
    tw(mem, l, ag);
    tw(mem, l + 8, aw);
    sx(mem, l + 16, avt as u32);
}















fn wkz(fe: &mut [u8], yay: u64) -> Result<()> {
    let frn = OS_ as usize;

    
    for a in 0..(6 * 4096) {
        if frn + a < fe.len() {
            fe[frn + a] = 0;
        }
    }

    
    let vgh = OS_ + 0x1000;
    tw(fe, frn, vgh | 0x3); 

    
    for a in 0..4u64 {
        let vgb = OS_ + 0x2000 + a * 0x1000;
        tw(fe, frn + 0x1000 + (a as usize) * 8,
                  vgb | 0x3); 
    }

    
    for rn in 0..4u64 {
        let vgd = frn + 0x2000 + (rn as usize) * 0x1000;
        for bt in 0..512u64 {
            let ki = (rn * 512 + bt) * 0x200000; 
            
            let gpd = ki | 0x83; 
            tw(fe, vgd + (bt as usize) * 8, gpd);
        }
    }

    crate::serial_println!("[Linux] Guest page tables at 0x{:X} (identity map 0-4GB, 2MB pages)",
                          OS_);
    Ok(())
}















fn wkw(fe: &mut [u8]) -> Result<()> {
    let bun = IB_ as usize;

    
    for a in 0..512 {
        if bun + a < fe.len() {
            fe[bun + a] = 0;
        }
    }

    

    
    
    
    tw(fe, bun + 0x08, 
              iwj(0, 0xFFFFF, 0x9A, 0xA)); 

    
    
    tw(fe, bun + 0x10,
              iwj(0, 0xFFFFF, 0x92, 0xC)); 

    
    tw(fe, bun + 0x18,
              iwj(0, 0xFFFFF, 0x9A, 0xC)); 

    
    tw(fe, bun + 0x20,
              iwj(0, 0xFFFFF, 0x92, 0xC)); 

    
    let kxw = 5 * 8 - 1; 
    aqr(fe, bun + 0x100, kxw as u16);
    tw(fe, bun + 0x102, IB_);

    crate::serial_println!("[Linux] Guest GDT at 0x{:X}: null, code64(0x08), data64(0x10), code32(0x18), data32(0x20)",
                          IB_);
    Ok(())
}







fn iwj(ar: u32, ul: u32, vz: u8, flags: u8) -> u64 {
    let mut bt = 0u64;
    
    
    bt |= (ul & 0xFFFF) as u64;
    
    bt |= ((ar & 0xFFFF) as u64) << 16;
    
    bt |= (((ar >> 16) & 0xFF) as u64) << 32;
    
    bt |= (vz as u64) << 40;
    
    bt |= (((ul >> 16) & 0xF) as u64) << 48;
    
    bt |= ((flags & 0xF) as u64) << 52;
    
    bt |= (((ar >> 24) & 0xFF) as u64) << 56;
    
    bt
}












pub fn rnu(
    vmcs: &super::vmcs::Vmcs,
    aeq: &Ajt,
) -> Result<()> {
    use super::vmcs::fields;

    
    
    let akb = 0x8005_0033u64; 
    vmcs.write(fields::ATY_, akb)?;
    
    
    vmcs.write(fields::ATZ_, aeq.jm)?;
    
    
    let cr4 = 0x000006A0u64; 
    vmcs.write(fields::AUA_, cr4)?;

    
    
    vmcs.write(fields::AUE_, 0x08)?;
    vmcs.write(fields::AUC_, 0)?;
    vmcs.write(fields::AUD_, 0xFFFFFFFF)?;
    vmcs.write(fields::AUB_, 0xA09B)?; 

    
    vmcs.write(fields::AVJ_, 0x10)?;
    vmcs.write(fields::AVH_, 0)?;
    vmcs.write(fields::AVI_, 0xFFFFFFFF)?;
    vmcs.write(fields::AVG_, 0xC093)?; 

    
    for (wgq, qni, uet, qet) in [
        (fields::AUI_, fields::AUG_, fields::AUH_, fields::AUF_),
        (fields::AUM_, fields::AUK_, fields::AUL_, fields::AUJ_),
        (fields::AUQ_, fields::AUO_, fields::AUP_, fields::AUN_),
        (fields::AUW_, fields::AUU_, fields::AUV_, fields::AUT_),
    ] {
        vmcs.write(wgq, 0x10)?;
        vmcs.write(qni, 0)?;
        vmcs.write(uet, 0xFFFFFFFF)?;
        vmcs.write(qet, 0xC093)?;
    }

    
    vmcs.write(fields::AVN_, 0)?;
    vmcs.write(fields::AVL_, 0)?;
    vmcs.write(fields::AVM_, 0xFFFF)?;
    vmcs.write(fields::AVK_, 0x8B)?; 

    
    vmcs.write(fields::AVD_, 0)?;
    vmcs.write(fields::AVB_, 0)?;
    vmcs.write(fields::AVC_, 0xFFFF)?;
    vmcs.write(fields::AVA_, 0x10082)?; 

    
    vmcs.write(fields::AUR_, aeq.bun)?;
    vmcs.write(fields::AUS_, 5 * 8 - 1)?;

    
    vmcs.write(fields::AUY_, 0)?;
    vmcs.write(fields::AUZ_, 0xFFF)?;

    
    vmcs.write(fields::FG_, aeq.mi)?;
    vmcs.write(fields::AVF_, aeq.ahu)?;
    vmcs.write(fields::AVE_, 0x2)?; 

    
    
    vmcs.write(fields::AUX_, 0x501)?; 

    crate::serial_println!("[Linux] VMCS configured: RIP=0x{:X} RSP=0x{:X} CR3=0x{:X} RSI=0x{:X}",
                          aeq.mi, aeq.ahu, aeq.jm, aeq.avg);

    Ok(())
}








pub fn vks(
    fe: &mut [u8],
    cwc: &[u8],
    wx: &str,
    apw: Option<&[u8]>,
) -> Result<Ajt> {
    
    let acf = oud(cwc)?;

    if !acf.gtp {
        crate::serial_println!("[Linux] Warning: kernel does not advertise 64-bit support");
        
    }

    
    let config = Acq {
        wx: String::from(wx),
        apy: fe.len() as u64,
        apw: apw.map(|bc| bc.ip()),
    };

    
    ojt(fe, &acf, &config)
}






pub fn klw() -> Vec<u8> {
    
    
    

    let mut image = vec![0u8; 4096]; 

    
    image[0x1F1] = 1;

    
    image[0x202] = b'H';
    image[0x203] = b'd';
    image[0x204] = b'r';
    image[0x205] = b'S';

    
    image[0x206] = 0x0F;
    image[0x207] = 0x02;

    
    image[0x211] = UT_;

    
    sx(&mut image, 0x214, 0x100000);

    
    aqr(&mut image, 0x236, BJF_);

    
    tw(&mut image, 0x258, 0x100000);

    
    sx(&mut image, 0x260, 0x100000);

    
    
    
    
    
    while image.len() < 1024 {
        image.push(0);
    }

    
    let jcb = image.len();
    
    
    for _ in 0..0x200 {
        image.push(0x90); 
    }

    
    
    
    
    
    
    let message = b"[TrustVM-Linux] Boot OK - 64-bit entry reached!\r\n";
    for &hf in message {
        
        image.bk(&[0x66, 0xBA, 0xF8, 0x03]);
        
        image.bk(&[0xB0, hf]);
        
        image.push(0xEE);
    }

    
    let rub = b"[TrustVM-Linux] 64-bit kernel entry OK\n";
    for &hf in rub {
        image.bk(&[0xB0, hf]);
        image.bk(&[0xE6, 0xE9]);
    }

    
    
    image.bk(&[0x48, 0xC7, 0xC0, 0x00, 0x00, 0x00, 0x00]); 
    image.bk(&[0x0F, 0x01, 0xC1]); 

    
    image.bk(&[0x48, 0xC7, 0xC0, 0x01, 0x00, 0x00, 0x00]); 
    image.bk(&[0x0F, 0x01, 0xC1]); 

    
    image.push(0xF4);

    
    let bvc = image.len() - jcb;
    sx(&mut image, 0x1F4, (bvc / 16) as u32);

    crate::serial_println!("[Linux] Created test kernel: {} bytes ({} setup + {} kernel)",
                          image.len(), jcb, bvc);

    image
}





fn alp(f: &[u8], l: usize) -> u16 {
    u16::dj([f[l], f[l + 1]])
}

fn za(f: &[u8], l: usize) -> u32 {
    u32::dj([
        f[l], f[l + 1],
        f[l + 2], f[l + 3],
    ])
}

fn aqi(f: &[u8], l: usize) -> u64 {
    u64::dj([
        f[l], f[l + 1], f[l + 2], f[l + 3],
        f[l + 4], f[l + 5], f[l + 6], f[l + 7],
    ])
}

fn aqr(f: &mut [u8], l: usize, bn: u16) {
    let bf = bn.ho();
    f[l] = bf[0];
    f[l + 1] = bf[1];
}

fn sx(f: &mut [u8], l: usize, bn: u32) {
    let bf = bn.ho();
    f[l] = bf[0];
    f[l + 1] = bf[1];
    f[l + 2] = bf[2];
    f[l + 3] = bf[3];
}

fn tw(f: &mut [u8], l: usize, bn: u64) {
    let bf = bn.ho();
    for a in 0..8 {
        f[l + a] = bf[a];
    }
}
