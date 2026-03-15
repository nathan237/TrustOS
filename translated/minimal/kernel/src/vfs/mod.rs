




use alloc::string::{String, Gd};
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::format;
use spin::RwLock;
use core::sync::atomic::{AtomicU64, Ordering};

pub mod devfs;
pub mod procfs;
pub mod trustfs;
pub mod fat32;
pub mod block_cache;
pub mod wal;
pub mod ext4;
pub mod ntfs;


pub type Fo = i32;


pub type I = u64;


#[derive(Clone, Copy, Debug)]
pub struct OpenFlags(pub u32);

impl OpenFlags {
    pub const OO_: u32 = 0;
    pub const OP_: u32 = 1;
    pub const DYK_: u32 = 2;
    pub const ON_: u32 = 0o100;
    pub const BCG_: u32 = 0o1000;
    pub const BCF_: u32 = 0o2000;
    
    pub fn bob(&self) -> bool {
        (self.0 & 3) != Self::OP_
    }
    
    pub fn bjb(&self) -> bool {
        (self.0 & 3) != Self::OO_
    }
    
    pub fn avp(&self) -> bool {
        (self.0 & Self::ON_) != 0
    }
    
    pub fn bte(&self) -> bool {
        (self.0 & Self::BCF_) != 0
    }
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileType {
    Ea,
    K,
    Mv,
    Bj,
    Anh,
    Yc,
    Socket,
}


#[derive(Clone, Debug)]
pub struct Stat {
    pub dd: I,
    pub kd: FileType,
    pub aw: u64,
    pub xk: u64,
    pub py: u32,
    pub ev: u32,      
    pub pi: u32,
    pub pw: u32,
    pub byi: u64,     
    pub bnp: u64,     
    pub cpq: u64,     
}

impl Default for Stat {
    fn default() -> Self {
        Self {
            dd: 0,
            kd: FileType::Ea,
            aw: 0,
            xk: 0,
            py: 512,
            ev: 0o644,
            pi: 0,
            pw: 0,
            byi: 0,
            bnp: 0,
            cpq: 0,
        }
    }
}


#[derive(Clone, Debug)]
pub struct Br {
    pub j: String,
    pub dd: I,
    pub kd: FileType,
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VfsError {
    N,
    Jt,
    Ri,
    Lz,
    Tc,
    Bnj,
    Pr,
    Bjs,
    Tq,
    Av,
    Cib,
    Jg,
    Dju,
    Bz,
    Rq,
}

pub type B<T> = Result<T, VfsError>;


pub trait Et: Send + Sync {
    fn read(&self, l: u64, k: &mut [u8]) -> B<usize>;
    fn write(&self, l: u64, k: &[u8]) -> B<usize>;
    fn hm(&self) -> B<Stat>;
    fn dmu(&self, aw: u64) -> B<()> { 
        let _ = aw;
        Err(VfsError::Cib) 
    }
    fn sync(&self) -> B<()> { Ok(()) }
}


pub trait Ep: Send + Sync {
    fn cga(&self, j: &str) -> B<I>;
    fn brx(&self) -> B<Vec<Br>>;
    fn avp(&self, j: &str, kd: FileType) -> B<I>;
    fn cnm(&self, j: &str) -> B<()>;
    fn hm(&self) -> B<Stat>;
}


pub trait Cc: Send + Sync {
    fn j(&self) -> &str;
    fn cbm(&self) -> I;
    fn era(&self, dd: I) -> B<Arc<dyn Et>>;
    fn dhl(&self, dd: I) -> B<Arc<dyn Ep>>;
    fn hm(&self, dd: I) -> B<Stat>;
    fn sync(&self) -> B<()> { Ok(()) }
}


struct Bmp {
    path: String,
    fs: Arc<dyn Cc>,
}


struct Tt {
    dd: I,
    abs: usize,
    l: u64,
    flags: OpenFlags,
}


struct Vfs {
    ajf: Vec<Bmp>,
    awi: BTreeMap<Fo, Tt>,
    bca: AtomicU64,
}

impl Vfs {
    const fn new() -> Self {
        Self {
            ajf: Vec::new(),
            awi: BTreeMap::new(),
            bca: AtomicU64::new(3), 
        }
    }
    
    fn jzz(&self) -> Fo {
        self.bca.fetch_add(1, Ordering::SeqCst) as Fo
    }
}

static Bi: RwLock<Vfs> = RwLock::new(Vfs::new());



pub fn mfc() {
    
    let (abs, dd) = match aqj("/dev/console") {
        Ok(m) => m,
        Err(_) => {
            crate::serial_println!("[VFS] Warning: /dev/console not found, stdio unavailable");
            return;
        }
    };

    let mut vfs = Bi.write();
    
    vfs.awi.insert(0, Tt {
        dd,
        abs,
        l: 0,
        flags: OpenFlags(0), 
    });
    
    vfs.awi.insert(1, Tt {
        dd,
        abs,
        l: 0,
        flags: OpenFlags(1), 
    });
    
    vfs.awi.insert(2, Tt {
        dd,
        abs,
        l: 0,
        flags: OpenFlags(1), 
    });
}


pub fn khv() {
    let mut vfs = Bi.write();
    vfs.awi.remove(&0);
    vfs.awi.remove(&1);
    vfs.awi.remove(&2);
}


pub fn init() {
    crate::log!("[VFS] Initializing Virtual File System...");
    
    
    if let Ok(devfs) = devfs::DevFs::new() {
        beu("/dev", Arc::new(devfs)).bq();
        crate::log_debug!("[VFS] Mounted devfs at /dev");
    }
    
    
    if let Ok(procfs) = procfs::ProcFs::new() {
        beu("/proc", Arc::new(procfs)).bq();
        crate::log_debug!("[VFS] Mounted procfs at /proc");
    }
    
    
    let mut jmt = false;
    
    if crate::virtio_blk::ky() {
        let backend = Arc::new(fat32::Bvw);
        let aty = crate::virtio_blk::aty();
        match trustfs::TrustFs::new(backend, aty) {
            Ok(trustfs) => {
                beu("/", Arc::new(trustfs)).bq();
                crate::log!("[VFS] Mounted TrustFS at / (virtio-blk, persistent)");
                jmt = true;
            }
            Err(aa) => {
                crate::log!("[VFS] TrustFS mount on virtio-blk failed: {:?}", aa);
            }
        }
    }
    
    
    if !jmt && crate::drivers::ahci::ky() {
        let ik = crate::drivers::ahci::bhh();
        
        
        for ba in &ik {
            if ba.agw > 64 {
                
                let mut probe = alloc::vec![0u8; 512];
                if crate::drivers::ahci::ain(ba.kg, 0, 1, &mut probe).is_ok() {
                    if probe.len() >= 4 && &probe[0..4] == b"TWAV" {
                        crate::log!("[VFS] Skipping AHCI port {} (TWAV audio data disk)", ba.kg);
                        continue;
                    }
                }
                let backend = Arc::new(fat32::AhciBlockReader::new(ba.kg as usize, 0));
                match trustfs::TrustFs::new(backend, ba.agw) {
                    Ok(trustfs) => {
                        beu("/", Arc::new(trustfs)).bq();
                        crate::log!("[VFS] Mounted TrustFS at / (AHCI port {}, persistent)", ba.kg);
                        jmt = true;
                        break;
                    }
                    Err(aa) => {
                        crate::log_debug!("[VFS] TrustFS on AHCI port {} failed: {:?}", ba.kg, aa);
                    }
                }
            }
        }
    }
    
    if !jmt {
        crate::log_debug!("[VFS] No block device, root will be ramfs");
    }
    
    
    #[cfg(target_arch = "x86_64")]
    {
        crate::log_debug!("[VFS] Looking for FAT32 partitions...");
        if let Some(src) = fat32::xmq() {
            beu("/mnt/fat32", src).bq();
            crate::log!("[VFS] Mounted FAT32 at /mnt/fat32");
        } else {
            crate::log_debug!("[VFS] No FAT32 partition found");
        }
    }
    
    
    #[cfg(target_arch = "x86_64")]
    {
        crate::log_debug!("[VFS] Looking for NTFS partitions...");
        if let Some(uwb) = ntfs::xmr() {
            beu("/mnt/ntfs", uwb).bq();
            crate::log!("[VFS] Mounted NTFS at /mnt/ntfs");
        } else {
            crate::log_debug!("[VFS] No NTFS partition found");
        }
    }
    
    crate::log!("[OK] VFS initialized");
}


pub fn beu(path: &str, fs: Arc<dyn Cc>) -> B<()> {
    let mut vfs = Bi.write();
    
    
    for sn in &vfs.ajf {
        if sn.path == path {
            return Err(VfsError::Rq);
        }
    }
    
    vfs.ajf.push(Bmp {
        path: String::from(path),
        fs,
    });
    
    
    vfs.ajf.bxe(|q, o| o.path.len().cmp(&q.path.len()));
    
    Ok(())
}


pub fn xob(path: &str) -> B<()> {
    let mut vfs = Bi.write();
    
    let w = vfs.ajf.iter().qf(|sn| sn.path == path)
        .ok_or(VfsError::N)?;
    
    
    if vfs.ajf[w].path == "/" {
        return Err(VfsError::Jt);
    }
    
    vfs.ajf.remove(w);
    Ok(())
}


fn stk(path: &str) -> Option<(usize, String)> {
    let vfs = Bi.read();
    
    for (w, sn) in vfs.ajf.iter().cf() {
        if path == sn.path || path.cj(&format!("{}/", sn.path)) || sn.path == "/" {
            let atj = if sn.path == "/" {
                path.to_string()
            } else {
                path.blj(&sn.path).unwrap_or("/").to_string()
            };
            return Some((w, if atj.is_empty() { "/".to_string() } else { atj }));
        }
    }
    
    None
}


fn aqj(path: &str) -> B<(usize, I)> {
    let (abs, atj) = stk(path).ok_or(VfsError::N)?;
    
    let vfs = Bi.read();
    let fs = &vfs.ajf[abs].fs;
    
    if atj == "/" || atj.is_empty() {
        return Ok((abs, fs.cbm()));
    }
    
    
    let mut kmw = fs.cbm();
    let rnb: Vec<&str> = atj.adk('/').hi(|e| !e.is_empty()).collect();
    
    for ffm in rnb {
        let te = fs.dhl(kmw)?;
        kmw = te.cga(ffm)?;
    }
    
    Ok((abs, kmw))
}


pub fn aji(path: &str, flags: OpenFlags) -> B<Fo> {
    let (abs, dd) = match aqj(path) {
        Ok(result) => result,
        Err(VfsError::N) if flags.avp() => {
            
            let bhs = jiq(path);
            let it = fdf(path);
            
            let (abs, dak) = aqj(&bhs)?;
            let vfs = Bi.read();
            let fs = &vfs.ajf[abs].fs;
            let hug = fs.dhl(dak)?;
            let dd = hug.avp(it, FileType::Ea)?;
            drop(vfs);
            
            (abs, dd)
        }
        Err(aa) => return Err(aa),
    };
    
    
    {
        let vfs = Bi.read();
        if let Ok(apc) = vfs.ajf[abs].fs.hm(dd) {
            
            let xtp  = flags.bob();
            let xtq = flags.bjb();
            let hsm = (if xtp { 4 } else { 0 }) | (if xtq { 2 } else { 0 });
            if hsm > 0 && !qzk(&apc, hsm) {
                return Err(VfsError::Jt);
            }
        }
    }
    
    let da = {
        let vfs = Bi.read();
        vfs.jzz()
    };
    
    let mut vfs = Bi.write();
    vfs.awi.insert(da, Tt {
        dd,
        abs,
        l: 0,
        flags,
    });
    
    
    crate::lab_mode::trace_bus::fj(
        crate::lab_mode::trace_bus::EventCategory::Cc,
        alloc::format!("open(\"{}\")", path),
        da as u64,
    );
    
    Ok(da)
}


pub fn read(da: Fo, k: &mut [u8]) -> B<usize> {
    let (abs, dd, l) = {
        let vfs = Bi.read();
        let file = vfs.awi.get(&da).ok_or(VfsError::Jg)?;
        if !file.flags.bob() {
            return Err(VfsError::Jt);
        }
        (file.abs, file.dd, file.l)
    };
    
    let cjl = {
        let vfs = Bi.read();
        let fs = &vfs.ajf[abs].fs;
        let kvr = fs.era(dd)?;
        kvr.read(l, k)?
    };
    
    
    let mut vfs = Bi.write();
    if let Some(file) = vfs.awi.ds(&da) {
        file.l += cjl as u64;
    }
    
    Ok(cjl)
}


pub fn write(da: Fo, k: &[u8]) -> B<usize> {
    let (abs, dd, l, bte) = {
        let vfs = Bi.read();
        let file = vfs.awi.get(&da).ok_or(VfsError::Jg)?;
        if !file.flags.bjb() {
            return Err(VfsError::Jt);
        }
        (file.abs, file.dd, file.l, file.flags.bte())
    };
    
    let fbt = if bte {
        let vfs = Bi.read();
        let fs = &vfs.ajf[abs].fs;
        let hm = fs.hm(dd)?;
        hm.aw
    } else {
        l
    };
    
    let cjm = {
        let vfs = Bi.read();
        let fs = &vfs.ajf[abs].fs;
        let kvr = fs.era(dd)?;
        kvr.write(fbt, k)?
    };
    
    
    let mut vfs = Bi.write();
    if let Some(file) = vfs.awi.ds(&da) {
        file.l = fbt + cjm as u64;
    }
    
    
    crate::lab_mode::trace_bus::fj(
        crate::lab_mode::trace_bus::EventCategory::Cc,
        alloc::format!("write fd={} {} bytes", da, cjm),
        cjm as u64,
    );
    
    Ok(cjm)
}


pub fn agj(da: Fo) -> B<()> {
    
    crate::lab_mode::trace_bus::fj(
        crate::lab_mode::trace_bus::EventCategory::Cc,
        alloc::format!("close fd={}", da),
        da as u64,
    );
    
    let mut vfs = Bi.write();
    vfs.awi.remove(&da).ok_or(VfsError::Jg)?;
    Ok(())
}


pub fn wgl(da: Fo, l: i64, gwp: i32) -> B<u64> {
    let mut vfs = Bi.write();
    let file = vfs.awi.ds(&da).ok_or(VfsError::Jg)?;
    
    let opz = match gwp {
        0 => l as u64,                          
        1 => (file.l as i64 + l) as u64,   
        2 => {
            
            drop(vfs);
            let aw = {
                let vfs = Bi.read();
                let gol = vfs.awi.get(&da).ok_or(VfsError::Jg)?;
                let fs = &vfs.ajf[gol.abs].fs;
                fs.hm(gol.dd)?.aw
            };
            let mut vfs = Bi.write();
            let file = vfs.awi.ds(&da).ok_or(VfsError::Jg)?;
            file.l = (aw as i64 + l) as u64;
            return Ok(file.l);
        }
        _ => return Err(VfsError::Pr),
    };
    
    file.l = opz;
    Ok(opz)
}


pub fn hm(path: &str) -> B<Stat> {
    let (abs, dd) = aqj(path)?;
    let vfs = Bi.read();
    let fs = &vfs.ajf[abs].fs;
    fs.hm(dd)
}


pub fn brx(path: &str) -> B<Vec<Br>> {
    let (abs, dd) = aqj(path)?;
    let vfs = Bi.read();
    let fs = &vfs.ajf[abs].fs;
    let te = fs.dhl(dd)?;
    te.brx()
}


pub fn ut(path: &str) -> B<()> {
    let bhs = jiq(path);
    let fgu = fdf(path);
    
    let (abs, dak) = aqj(&bhs)?;
    let vfs = Bi.read();
    let fs = &vfs.ajf[abs].fs;
    let hug = fs.dhl(dak)?;
    hug.avp(fgu, FileType::K)?;
    Ok(())
}


pub fn uot(path: &str) -> B<()> {
    let path = path.bdd('/');
    if path.is_empty() || path == "/" {
        return Ok(()); 
    }
    
    
    if ut(path).is_ok() {
        return Ok(());
    }
    
    
    let tu = jiq(path);
    if !tu.is_empty() && tu != "/" {
        uot(&tu)?;
    }
    
    
    ut(path)
}


pub fn cnm(path: &str) -> B<()> {
    let bhs = jiq(path);
    let it = fdf(path);
    
    let (abs, dak) = aqj(&bhs)?;
    let vfs = Bi.read();
    let fs = &vfs.ajf[abs].fs;
    let hug = fs.dhl(dak)?;
    hug.cnm(it)
}


fn jiq(path: &str) -> String {
    if let Some(u) = path.bhx('/') {
        if u == 0 {
            "/".to_string()
        } else {
            path[..u].to_string()
        }
    } else {
        "/".to_string()
    }
}


fn fdf(path: &str) -> &str {
    if let Some(u) = path.bhx('/') {
        &path[u + 1..]
    } else {
        path
    }
}


pub fn hqa() -> Vec<(String, String)> {
    let vfs = Bi.read();
    vfs.ajf.iter().map(|sn| (sn.path.clone(), sn.fs.j().to_string())).collect()
}






pub fn uis(da: Fo, l: i64, gwp: u32) -> B<u64> {
    wgl(da, l, gwp as i32)
}


static Aps: RwLock<String> = RwLock::new(String::new());


pub fn iwx() -> String {
    let jv = Aps.read();
    if jv.is_empty() {
        "/".to_string()
    } else {
        jv.clone()
    }
}


pub fn qyj(path: &str) -> B<()> {
    
    let wtk = hm(path)?;
    if wtk.kd != FileType::K {
        return Err(VfsError::Lz);
    }
    
    
    let lnt = if path.cj('/') {
        path.to_string()
    } else {
        let cv = iwx();
        if cv == "/" {
            format!("/{}", path)
        } else {
            format!("{}/{}", cv, path)
        }
    };
    
    *Aps.write() = lnt;
    Ok(())
}


pub fn yxo() {
    *Aps.write() = "/".to_string();
}






pub fn mq(path: &str) -> B<Vec<u8>> {
    let da = aji(path, OpenFlags(OpenFlags::OO_))?;
    
    
    let wtj = hm(path)?;
    let aw = wtj.aw as usize;
    
    
    let mut bi = alloc::vec![0u8; aw.am(1024)];
    let mut l = 0;
    while l < bi.len() {
        let bo = read(da, &mut bi[l..])?;
        if bo == 0 { break; }
        l += bo;
    }
    bi.dmu(l);
    
    agj(da)?;
    Ok(bi)
}


pub fn lxu(path: &str) -> B<String> {
    let bf = mq(path)?;
    String::jg(bf).jd(|_| VfsError::Bjs)
}


pub fn ns(path: &str, f: &[u8]) -> B<()> {
    
    let _ = ut(path); 
    
    let da = aji(path, OpenFlags(OpenFlags::OP_ | OpenFlags::ON_ | OpenFlags::BCG_))?;
    let mut l = 0;
    while l < f.len() {
        let bo = write(da, &f[l..])?;
        if bo == 0 { break; }
        l += bo;
    }
    agj(da)?;
    Ok(())
}


pub fn wxb() -> B<()> {
    let vfs = Bi.read();
    for beu in vfs.ajf.iter() {
        let _ = beu.fs.sync();
    }
    
    let _ = block_cache::sync();
    Ok(())
}


pub fn ksb(bns: Fo) -> B<Fo> {
    let vfs = Bi.read();
    let file = vfs.awi.get(&bns).ok_or(VfsError::Jg)?;
    let bdu = Tt {
        dd: file.dd,
        abs: file.abs,
        l: file.l,
        flags: file.flags,
    };
    let anp = vfs.jzz();
    drop(vfs);
    let mut vfs = Bi.write();
    vfs.awi.insert(anp, bdu);
    Ok(anp)
}


pub fn noj(bns: Fo, anp: Fo) -> B<Fo> {
    if bns == anp {
        if Bi.read().awi.bgm(&bns) { return Ok(anp); }
        return Err(VfsError::Jg);
    }
    let vfs = Bi.read();
    let file = vfs.awi.get(&bns).ok_or(VfsError::Jg)?;
    let bdu = Tt {
        dd: file.dd,
        abs: file.abs,
        l: file.l,
        flags: file.flags,
    };
    drop(vfs);
    let mut vfs = Bi.write();
    vfs.awi.remove(&anp);
    vfs.awi.insert(anp, bdu);
    Ok(anp)
}


pub fn syo(da: Fo) -> B<Stat> {
    let vfs = Bi.read();
    let file = vfs.awi.get(&da).ok_or(VfsError::Jg)?;
    let abs = file.abs;
    let dd = file.dd;
    let fs = &vfs.ajf[abs].fs;
    fs.hm(dd)
}






pub struct Ye {
    
    pub bob: bool,
    
    pub bjb: bool,
    
    pub zt: bool,
    
    pub fkc: bool,
}



pub fn owj(da: Fo) -> Option<Ye> {
    
    if da == 0 {
        return Some(Ye {
            bob: crate::keyboard::hmo(),
            bjb: false,
            zt: false,
            fkc: false,
        });
    }
    
    if da == 1 || da == 2 {
        return Some(Ye {
            bob: false,
            bjb: true,
            zt: false,
            fkc: false,
        });
    }
    
    if crate::pipe::gkh(da) {
        let (cyk, lbi, jjd) = crate::pipe::poll(da);
        return Some(Ye {
            bob: cyk,
            bjb: lbi,
            zt: false,
            fkc: jjd,
        });
    }
    
    if crate::netstack::socket::tyx(da) {
        let cyk = crate::netstack::socket::tna(da);
        return Some(Ye {
            bob: cyk,
            bjb: true, 
            zt: false,
            fkc: false,
        });
    }
    
    let vfs = Bi.read();
    if vfs.awi.bgm(&da) {
        return Some(Ye {
            bob: true,
            bjb: true,
            zt: false,
            fkc: false,
        });
    }
    
    None
}







pub fn qzk(apc: &Stat, pzb: u32) -> bool {
    let (pi, pw, ahl, bqj) = crate::process::dfk();
    
    if ahl == 0 { return true; }

    let ev = apc.ev;
    let fs = if ahl == apc.pi {
        (ev >> 6) & 7 
    } else if bqj == apc.pw {
        (ev >> 3) & 7 
    } else {
        ev & 7 
    };
    (fs & pzb) == pzb
}


pub fn ral(path: &str, ev: u32) -> B<()> {
    let (_, _, ahl, _) = crate::process::dfk();
    let apc = hm(path)?;
    
    if ahl != 0 && ahl != apc.pi {
        return Err(VfsError::Jt);
    }
    let (abs, dd) = aqj(path)?;
    let vfs = Bi.read();
    let _ = piy(&*vfs.ajf[abs].fs, dd, ev);
    Ok(())
}


pub fn srh(da: Fo, ev: u32) -> B<()> {
    let (_, _, ahl, _) = crate::process::dfk();
    let vfs = Bi.read();
    let file = vfs.awi.get(&da).ok_or(VfsError::Jg)?;
    let abs = file.abs;
    let dd = file.dd;
    let apc = vfs.ajf[abs].fs.hm(dd)?;
    if ahl != 0 && ahl != apc.pi {
        return Err(VfsError::Jt);
    }
    let _ = piy(&*vfs.ajf[abs].fs, dd, ev);
    Ok(())
}


pub fn ran(path: &str, pi: u32, pw: u32) -> B<()> {
    let (abs, dd) = aqj(path)?;
    let vfs = Bi.read();
    let _ = piz(&*vfs.ajf[abs].fs, dd, pi, pw);
    Ok(())
}


pub fn sri(da: Fo, pi: u32, pw: u32) -> B<()> {
    let vfs = Bi.read();
    let file = vfs.awi.get(&da).ok_or(VfsError::Jg)?;
    let _ = piz(&*vfs.ajf[file.abs].fs, file.dd, pi, pw);
    Ok(())
}


fn piy(qby: &dyn crate::vfs::Cc, dd: I, ev: u32) -> B<()> {
    
    BAL_.lock().insert(dd, Avr { ev: Some(ev), pi: None, pw: None });
    Ok(())
}


fn piz(qby: &dyn crate::vfs::Cc, dd: I, pi: u32, pw: u32) -> B<()> {
    let mut cte = BAL_.lock();
    let bt = cte.bt(dd).gom(Avr { ev: None, pi: None, pw: None });
    if pi != 0xFFFFFFFF { bt.pi = Some(pi); }
    if pw != 0xFFFFFFFF { bt.pw = Some(pw); }
    Ok(())
}


#[derive(Clone, Debug)]
struct Avr {
    ev: Option<u32>,
    pi: Option<u32>,
    pw: Option<u32>,
}

use spin::Mutex as SpinMutex;
use alloc::collections::BTreeMap as OverlayMap;
static BAL_: SpinMutex<OverlayMap<I, Avr>> = SpinMutex::new(OverlayMap::new());
