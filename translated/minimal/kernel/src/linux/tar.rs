



use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;


#[repr(C)]
struct Djs {
    j: [u8; 100],
    ev: [u8; 8],
    pi: [u8; 8],
    pw: [u8; 8],
    aw: [u8; 12],
    bnp: [u8; 12],
    yid: [u8; 8],
    pwu: u8,
    zau: [u8; 100],
    sj: [u8; 6],
    dk: [u8; 2],
    cin: [u8; 32],
    yvd: [u8; 32],
    yly: [u8; 8],
    ylz: [u8; 8],
    adx: [u8; 155],
    fzo: [u8; 12],
}

const HF_: usize = 512;


const Cjs: u8 = b'0';
const Bxj: u8 = 0;
const Cah: u8 = b'5';
const Cmn: u8 = b'2';


pub fn sqk(xau: &str, aac: &str) -> Result<usize, &'static str> {
    
    let ahf = crate::ramfs::fh(|fs| {
        fs.mq(xau)
            .map(|f| f.ip())
            .jd(|_| "tarball not found")
    })?;
    
    
    let dpu = rut(&ahf)?;
    
    
    sqj(&dpu, aac)
}


fn rut(f: &[u8]) -> Result<Vec<u8>, &'static str> {
    
    if f.len() < 10 || f[0] != 0x1f || f[1] != 0x8b {
        return Err("not a gzip file");
    }
    
    
    if f[2] != 8 {
        return Err("unsupported compression method");
    }
    
    let flags = f[3];
    let mut u = 10;
    
    
    if flags & 0x04 != 0 {
        if u + 2 > f.len() {
            return Err("truncated gzip");
        }
        let mrq = (f[u] as usize) | ((f[u + 1] as usize) << 8);
        u += 2 + mrq;
    }
    
    
    if flags & 0x08 != 0 {
        while u < f.len() && f[u] != 0 {
            u += 1;
        }
        u += 1;
    }
    
    
    if flags & 0x10 != 0 {
        while u < f.len() && f[u] != 0 {
            u += 1;
        }
        u += 1;
    }
    
    
    if flags & 0x02 != 0 {
        u += 2;
    }
    
    if u >= f.len() {
        return Err("truncated gzip header");
    }
    
    
    let len = f.len();
    if len < 8 {
        return Err("truncated gzip");
    }
    let ota = (f[len - 4] as usize)
        | ((f[len - 3] as usize) << 8)
        | ((f[len - 2] as usize) << 16)
        | ((f[len - 1] as usize) << 24);
    
    
    let gde = &f[u..len - 8];
    
    
    let mut an = Vec::fc(ota);
    
    
    match tsv(gde, &mut an, ota) {
        Ok(()) => Ok(an),
        Err(aa) => Err(aa),
    }
}


fn tsv(input: &[u8], an: &mut Vec<u8>, ggm: usize) -> Result<(), &'static str> {
    
    let mut azm = DeflateDecoder::new(input);
    
    loop {
        match azm.rug(an)? {
            BlockResult::Cg => continue,
            BlockResult::Bev => break,
        }
        
        
        if an.len() > ggm + 1024 * 1024 {
            return Err("decompression overflow");
        }
    }
    
    Ok(())
}

enum BlockResult {
    Cg,
    Bev,
}

struct DeflateDecoder<'a> {
    input: &'a [u8],
    u: usize,
    har: u32,
    gbc: u8,
}

impl<'a> DeflateDecoder<'a> {
    fn new(input: &'a [u8]) -> Self {
        Self {
            input,
            u: 0,
            har: 0,
            gbc: 0,
        }
    }
    
    fn dax(&mut self, bo: u8) -> Result<u32, &'static str> {
        while self.gbc < bo {
            if self.u >= self.input.len() {
                return Err("unexpected end of input");
            }
            self.har |= (self.input[self.u] as u32) << self.gbc;
            self.u += 1;
            self.gbc += 8;
        }
        
        let result = self.har & ((1 << bo) - 1);
        self.har >>= bo;
        self.gbc -= bo;
        Ok(result)
    }
    
    fn rug(&mut self, an: &mut Vec<u8>) -> Result<BlockResult, &'static str> {
        let qpa = self.dax(1)?;
        let qsn = self.dax(2)?;
        
        match qsn {
            0 => {
                
                self.har = 0;
                self.gbc = 0;
                
                if self.u + 4 > self.input.len() {
                    return Err("truncated stored block");
                }
                
                let len = (self.input[self.u] as u16) | ((self.input[self.u + 1] as u16) << 8);
                self.u += 4;
                
                let len = len as usize;
                if self.u + len > self.input.len() {
                    return Err("truncated stored block data");
                }
                
                an.bk(&self.input[self.u..self.u + len]);
                self.u += len;
            }
            1 => {
                
                self.nka(an, true)?;
            }
            2 => {
                
                self.nka(an, false)?;
            }
            _ => return Err("invalid block type"),
        }
        
        if qpa != 0 {
            Ok(BlockResult::Bev)
        } else {
            Ok(BlockResult::Cg)
        }
    }
    
    fn nka(&mut self, an: &mut Vec<u8>, sui: bool) -> Result<(), &'static str> {
        
        let (eer, hgk) = if sui {
            Self::suj()
        } else {
            self.vrp()?
        };
        
        loop {
            let aaw = self.koo(&eer)?;
            
            if aaw < 256 {
                an.push(aaw as u8);
            } else if aaw == 256 {
                break;
            } else {
                
                let go = self.rup(aaw)?;
                let rzd = self.koo(&hgk)?;
                let eoy = self.rui(rzd)?;
                
                if eoy > an.len() {
                    return Err("invalid back reference");
                }
                
                let ay = an.len() - eoy;
                for a in 0..go {
                    let hf = an[ay + (a % eoy)];
                    an.push(hf);
                }
            }
        }
        
        Ok(())
    }
    
    fn suj() -> (Vec<u8>, Vec<u8>) {
        let mut eer = vec![0u8; 288];
        for a in 0..144 { eer[a] = 8; }
        for a in 144..256 { eer[a] = 9; }
        for a in 256..280 { eer[a] = 7; }
        for a in 280..288 { eer[a] = 8; }
        
        let hgk = vec![5u8; 32];
        
        (eer, hgk)
    }
    
    fn vrp(&mut self) -> Result<(Vec<u8>, Vec<u8>), &'static str> {
        let iyl = self.dax(5)? as usize + 257;
        let obe = self.dax(5)? as usize + 1;
        let tnr = self.dax(4)? as usize + 4;
        
        const BMU_: [usize; 19] = [16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15];
        
        let mut gcu = [0u8; 19];
        for a in 0..tnr {
            gcu[BMU_[a]] = self.dax(3)? as u8;
        }
        
        let rle = gcu.ip();
        
        
        let mut emc = Vec::fc(iyl + obe);
        while emc.len() < iyl + obe {
            let aaw = self.koo(&rle)?;
            match aaw {
                0..=15 => emc.push(aaw as u8),
                16 => {
                    let az = self.dax(2)? as usize + 3;
                    let qv = *emc.qv().ok_or("invalid code")?;
                    for _ in 0..az { emc.push(qv); }
                }
                17 => {
                    let az = self.dax(3)? as usize + 3;
                    for _ in 0..az { emc.push(0); }
                }
                18 => {
                    let az = self.dax(7)? as usize + 11;
                    for _ in 0..az { emc.push(0); }
                }
                _ => return Err("invalid code length symbol"),
            }
        }
        
        let eer = emc[..iyl].ip();
        let hgk = emc[iyl..].ip();
        
        Ok((eer, hgk))
    }
    
    fn koo(&mut self, gcu: &[u8]) -> Result<u16, &'static str> {
        
        let mut aj = 0u32;
        let mut fv = [0u32; 16];
        let mut index = [0u16; 16];
        
        
        let mut az = [0u16; 16];
        for &len in gcu {
            if len > 0 && (len as usize) < 16 {
                az[len as usize] += 1;
            }
        }
        
        
        let mut w = 0u16;
        for a in 1..16 {
            fv[a] = aj;
            index[a] = w;
            aj = (aj + az[a] as u32) << 1;
            w += az[a];
        }
        
        
        let mut bot = vec![0u16; gcu.len()];
        for (aaw, &len) in gcu.iter().cf() {
            if len > 0 {
                let len = len as usize;
                let dwm = index[len] as usize;
                if dwm < bot.len() {
                    bot[dwm] = aaw as u16;
                    index[len] += 1;
                }
            }
        }
        
        
        let mut aj = 0u32;
        let mut iuv = [0u16; 16];
        let mut kwk = [0u32; 16];
        
        w = 0;
        let mut r = 0u32;
        for a in 1..16 {
            kwk[a] = r;
            iuv[a] = w;
            r = (r + az[a] as u32) << 1;
            w += az[a];
        }
        
        for len in 1..16 {
            aj = (aj << 1) | self.dax(1)?;
            let ffg = az[len];
            if aj < kwk[len] + ffg as u32 {
                let dwm = iuv[len] + (aj - kwk[len]) as u16;
                return Ok(bot[dwm as usize]);
            }
        }
        
        Err("invalid huffman code")
    }
    
    fn rup(&mut self, aaw: u16) -> Result<usize, &'static str> {
        const AYY_: [u16; 29] = [3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131, 163, 195, 227, 258];
        const CEL_: [u8; 29] = [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0];
        
        let w = (aaw - 257) as usize;
        if w >= AYY_.len() {
            return Err("invalid length code");
        }
        
        let ar = AYY_[w] as usize;
        let ang = CEL_[w];
        let kur = if ang > 0 { self.dax(ang)? as usize } else { 0 };
        
        Ok(ar + kur)
    }
    
    fn rui(&mut self, aaw: u16) -> Result<usize, &'static str> {
        const AQI_: [u16; 30] = [1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537, 2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577];
        const BRV_: [u8; 30] = [0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13];
        
        let w = aaw as usize;
        if w >= AQI_.len() {
            return Err("invalid distance code");
        }
        
        let ar = AQI_[w] as usize;
        let ang = BRV_[w];
        let kur = if ang > 0 { self.dax(ang)? as usize } else { 0 };
        
        Ok(ar + kur)
    }
}


fn sqj(f: &[u8], aac: &str) -> Result<usize, &'static str> {
    let mut u = 0;
    let mut bec = 0;
    
    while u + HF_ <= f.len() {
        let dh = &f[u..u + HF_];
        
        
        if dh.iter().xx(|&o| o == 0) {
            break;
        }
        
        
        let j = vdy(dh)?;
        let aw = vda(&dh[124..136])?;
        let pwu = dh[156];
        
        
        let adx = jit(&dh[345..500]);
        let ghr = if adx.is_empty() {
            j
        } else {
            format!("{}/{}", adx, j)
        };
        
        
        if ghr.is_empty() || ghr == "." || ghr == "./" {
            u += HF_;
            u += jzv(aw);
            continue;
        }
        
        
        let doy = ghr.tl("./").tl('/');
        if doy.is_empty() {
            u += HF_;
            u += jzv(aw);
            continue;
        }
        
        let dge = format!("{}/{}", aac, doy);
        
        u += HF_;
        
        match pwu {
            Cah | b'/' => {
                
                crate::ramfs::fh(|fs| {
                    let _ = fs.ut(&dge);
                });
            }
            Cjs | Bxj | b'\0' => {
                
                if aw > 0 && u + aw <= f.len() {
                    let ca = &f[u..u + aw];
                    
                    
                    let tu = bhs(&dge);
                    nqh(&tu);
                    
                    crate::ramfs::fh(|fs| {
                        let _ = fs.touch(&dge);
                        let _ = fs.ns(&dge, ca);
                    });
                    
                    bec += 1;
                }
            }
            Cmn => {
                
                let uff = jit(&dh[157..257]);
                crate::ramfs::fh(|fs| {
                    let _ = fs.touch(&dge);
                    let _ = fs.ns(&dge, uff.as_bytes());
                });
            }
            _ => {
                
            }
        }
        
        u += jzv(aw);
    }
    
    Ok(bec)
}

fn vdy(dh: &[u8]) -> Result<String, &'static str> {
    Ok(jit(&dh[0..100]))
}

fn jit(bf: &[u8]) -> String {
    let ci = bf.iter().qf(|&o| o == 0).unwrap_or(bf.len());
    let cow = String::azw(&bf[..ci]);
    String::from(cow.em())
}

fn vda(bf: &[u8]) -> Result<usize, &'static str> {
    let e = jit(bf);
    if e.is_empty() {
        return Ok(0);
    }
    usize::wa(&e, 8).jd(|_| "invalid octal")
}

fn jzv(aw: usize) -> usize {
    if aw == 0 {
        0
    } else {
        ((aw + HF_ - 1) / HF_) * HF_
    }
}

fn bhs(path: &str) -> String {
    if let Some(u) = path.bhx('/') {
        if u == 0 {
            String::from("/")
        } else {
            String::from(&path[..u])
        }
    } else {
        String::from("/")
    }
}

fn nqh(path: &str) {
    if path == "/" || path.is_empty() {
        return;
    }
    
    crate::ramfs::fh(|fs| {
        if !fs.aja(path) {
            let tu = bhs(path);
            nqh(&tu);
            let _ = fs.ut(path);
        }
    });
}
