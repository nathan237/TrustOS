




use alloc::string::{String, ToString};
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


pub type Cm = i32;


pub type K = u64;


#[derive(Clone, Copy, Debug)]
pub struct OpenFlags(pub u32);

impl OpenFlags {
    pub const PM_: u32 = 0;
    pub const PN_: u32 = 1;
    pub const ECB_: u32 = 2;
    pub const PL_: u32 = 0o100;
    pub const BEJ_: u32 = 0o1000;
    pub const BEI_: u32 = 0o2000;
    
    pub fn readable(&self) -> bool {
        (self.0 & 3) != Self::PN_
    }
    
    pub fn writable(&self) -> bool {
        (self.0 & 3) != Self::PM_
    }
    
    pub fn create(&self) -> bool {
        (self.0 & Self::PL_) != 0
    }
    
    pub fn append(&self) -> bool {
        (self.0 & Self::BEI_) != 0
    }
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileType {
    Regular,
    Directory,
    CharDevice,
    Ak,
    Symlink,
    Pipe,
    Socket,
}


#[derive(Clone, Debug)]
pub struct Stat {
    pub ino: K,
    pub file_type: FileType,
    pub size: u64,
    pub blocks: u64,
    pub block_size: u32,
    pub mode: u32,      
    pub uid: u32,
    pub gid: u32,
    pub atime: u64,     
    pub mtime: u64,     
    pub ctime: u64,     
}

impl Default for Stat {
    fn default() -> Self {
        Self {
            ino: 0,
            file_type: FileType::Regular,
            size: 0,
            blocks: 0,
            block_size: 512,
            mode: 0o644,
            uid: 0,
            gid: 0,
            atime: 0,
            mtime: 0,
            ctime: 0,
        }
    }
}


#[derive(Clone, Debug)]
pub struct Ap {
    pub name: String,
    pub ino: K,
    pub file_type: FileType,
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VfsError {
    NotFound,
    PermissionDenied,
    AlreadyExists,
    NotDirectory,
    IsDirectory,
    NotEmpty,
    InvalidPath,
    InvalidData,
    NoSpace,
    IoError,
    NotSupported,
    BadFd,
    TooManyOpenFiles,
    ReadOnly,
    Busy,
}

pub type E<T> = Result<T, VfsError>;


pub trait Bx: Send + Sync {
    fn read(&self, offset: u64, buf: &mut [u8]) -> E<usize>;
    fn write(&self, offset: u64, buf: &[u8]) -> E<usize>;
    fn stat(&self) -> E<Stat>;
    fn truncate(&self, size: u64) -> E<()> { 
        let _ = size;
        Err(VfsError::NotSupported) 
    }
    fn sync(&self) -> E<()> { Ok(()) }
}


pub trait Bv: Send + Sync {
    fn lookup(&self, name: &str) -> E<K>;
    fn readdir(&self) -> E<Vec<Ap>>;
    fn create(&self, name: &str, file_type: FileType) -> E<K>;
    fn unlink(&self, name: &str) -> E<()>;
    fn stat(&self) -> E<Stat>;
}


pub trait Au: Send + Sync {
    fn name(&self) -> &str;
    fn root_inode(&self) -> K;
    fn get_file(&self, ino: K) -> E<Arc<dyn Bx>>;
    fn get_dir(&self, ino: K) -> E<Arc<dyn Bv>>;
    fn stat(&self, ino: K) -> E<Stat>;
    fn sync(&self) -> E<()> { Ok(()) }
}


struct Abm {
    path: String,
    fs: Arc<dyn Au>,
}


struct Il {
    ino: K,
    mount_idx: usize,
    offset: u64,
    flags: OpenFlags,
}


struct Vfs {
    mounts: Vec<Abm>,
    open_files: BTreeMap<Cm, Il>,
    next_fd: AtomicU64,
}

impl Vfs {
    const fn new() -> Self {
        Self {
            mounts: Vec::new(),
            open_files: BTreeMap::new(),
            next_fd: AtomicU64::new(3), 
        }
    }
    
    fn alloc_fd(&self) -> Cm {
        self.next_fd.fetch_add(1, Ordering::SeqCst) as Cm
    }
}

static Aj: RwLock<Vfs> = RwLock::new(Vfs::new());



pub fn gul() {
    
    let (mount_idx, ino) = match resolve_path("/dev/console") {
        Ok(r) => r,
        Err(_) => {
            crate::serial_println!("[VFS] Warning: /dev/console not found, stdio unavailable");
            return;
        }
    };

    let mut vfs = Aj.write();
    
    vfs.open_files.insert(0, Il {
        ino,
        mount_idx,
        offset: 0,
        flags: OpenFlags(0), 
    });
    
    vfs.open_files.insert(1, Il {
        ino,
        mount_idx,
        offset: 0,
        flags: OpenFlags(1), 
    });
    
    vfs.open_files.insert(2, Il {
        ino,
        mount_idx,
        offset: 0,
        flags: OpenFlags(1), 
    });
}


pub fn flv() {
    let mut vfs = Aj.write();
    vfs.open_files.remove(&0);
    vfs.open_files.remove(&1);
    vfs.open_files.remove(&2);
}


pub fn init() {
    crate::log!("[VFS] Initializing Virtual File System...");
    
    
    if let Ok(devfs) = devfs::DevFs::new() {
        abd("/dev", Arc::new(devfs)).ok();
        crate::log_debug!("[VFS] Mounted devfs at /dev");
    }
    
    
    if let Ok(procfs) = procfs::ProcFs::new() {
        abd("/proc", Arc::new(procfs)).ok();
        crate::log_debug!("[VFS] Mounted procfs at /proc");
    }
    
    
    let mut eyx = false;
    
    if crate::virtio_blk::is_initialized() {
        let backend = Arc::new(fat32::Agd);
        let capacity = crate::virtio_blk::capacity();
        match trustfs::TrustFs::new(backend, capacity) {
            Ok(trustfs) => {
                abd("/", Arc::new(trustfs)).ok();
                crate::log!("[VFS] Mounted TrustFS at / (virtio-blk, persistent)");
                eyx = true;
            }
            Err(e) => {
                crate::log!("[VFS] TrustFS mount on virtio-blk failed: {:?}", e);
            }
        }
    }
    
    
    if !eyx && crate::drivers::ahci::is_initialized() {
        let devices = crate::drivers::ahci::adz();
        
        
        for s in &devices {
            if s.sector_count > 64 {
                
                let mut probe = alloc::vec![0u8; 512];
                if crate::drivers::ahci::read_sectors(s.port_num, 0, 1, &mut probe).is_ok() {
                    if probe.len() >= 4 && &probe[0..4] == b"TWAV" {
                        crate::log!("[VFS] Skipping AHCI port {} (TWAV audio data disk)", s.port_num);
                        continue;
                    }
                }
                let backend = Arc::new(fat32::AhciBlockReader::new(s.port_num as usize, 0));
                match trustfs::TrustFs::new(backend, s.sector_count) {
                    Ok(trustfs) => {
                        abd("/", Arc::new(trustfs)).ok();
                        crate::log!("[VFS] Mounted TrustFS at / (AHCI port {}, persistent)", s.port_num);
                        eyx = true;
                        break;
                    }
                    Err(e) => {
                        crate::log_debug!("[VFS] TrustFS on AHCI port {} failed: {:?}", s.port_num, e);
                    }
                }
            }
        }
    }
    
    if !eyx {
        crate::log_debug!("[VFS] No block device, root will be ramfs");
    }
    
    
    #[cfg(target_arch = "x86_64")]
    {
        crate::log_debug!("[VFS] Looking for FAT32 partitions...");
        if let Some(fat32_fs) = fat32::pnx() {
            abd("/mnt/fat32", fat32_fs).ok();
            crate::log!("[VFS] Mounted FAT32 at /mnt/fat32");
        } else {
            crate::log_debug!("[VFS] No FAT32 partition found");
        }
    }
    
    
    #[cfg(target_arch = "x86_64")]
    {
        crate::log_debug!("[VFS] Looking for NTFS partitions...");
        if let Some(ntfs_fs) = ntfs::pny() {
            abd("/mnt/ntfs", ntfs_fs).ok();
            crate::log!("[VFS] Mounted NTFS at /mnt/ntfs");
        } else {
            crate::log_debug!("[VFS] No NTFS partition found");
        }
    }
    
    crate::log!("[OK] VFS initialized");
}


pub fn abd(path: &str, fs: Arc<dyn Au>) -> E<()> {
    let mut vfs = Aj.write();
    
    
    for ic in &vfs.mounts {
        if ic.path == path {
            return Err(VfsError::Busy);
        }
    }
    
    vfs.mounts.push(Abm {
        path: String::from(path),
        fs,
    });
    
    
    vfs.mounts.sort_by(|a, b| b.path.len().cmp(&a.path.len()));
    
    Ok(())
}


pub fn ppk(path: &str) -> E<()> {
    let mut vfs = Aj.write();
    
    let idx = vfs.mounts.iter().position(|ic| ic.path == path)
        .ok_or(VfsError::NotFound)?;
    
    
    if vfs.mounts[idx].path == "/" {
        return Err(VfsError::PermissionDenied);
    }
    
    vfs.mounts.remove(idx);
    Ok(())
}


fn lvz(path: &str) -> Option<(usize, String)> {
    let vfs = Aj.read();
    
    for (idx, ic) in vfs.mounts.iter().enumerate() {
        if path == ic.path
            || (path.starts_with(&ic.path)
                && path.as_bytes().get(ic.path.len()) == Some(&b'/'))
            || ic.path == "/"
        {
            let xj = if ic.path == "/" {
                path.to_string()
            } else {
                path.strip_prefix(&ic.path).unwrap_or("/").to_string()
            };
            return Some((idx, if xj.is_empty() { "/".to_string() } else { xj }));
        }
    }
    
    None
}


fn resolve_path(path: &str) -> E<(usize, K)> {
    let (mount_idx, xj) = lvz(path).ok_or(VfsError::NotFound)?;
    
    let vfs = Aj.read();
    let fs = &vfs.mounts[mount_idx].fs;
    
    if xj == "/" || xj.is_empty() {
        return Ok((mount_idx, fs.root_inode()));
    }
    
    
    let mut fpq = fs.root_inode();
    
    for chn in xj.split('/').filter(|j| !j.is_empty()) {
        let it = fs.get_dir(fpq)?;
        fpq = it.lookup(chn)?;
    }
    
    Ok((mount_idx, fpq))
}


pub fn open(path: &str, flags: OpenFlags) -> E<Cm> {
    let (mount_idx, ino) = match resolve_path(path) {
        Ok(result) => result,
        Err(VfsError::NotFound) if flags.create() => {
            
            let parent_path = ewe(path);
            let filename = basename(path);
            
            let (mount_idx, parent_ino) = resolve_path(&parent_path)?;
            let vfs = Aj.read();
            let fs = &vfs.mounts[mount_idx].fs;
            let dwd = fs.get_dir(parent_ino)?;
            let ino = dwd.create(filename, FileType::Regular)?;
            drop(vfs);
            
            (mount_idx, ino)
        }
        Err(e) => return Err(e),
    };
    
    
    {
        let vfs = Aj.read();
        if let Ok(uz) = vfs.mounts[mount_idx].fs.stat(ino) {
            
            let ptt  = flags.readable();
            let ptu = flags.writable();
            let duy = (if ptt { 4 } else { 0 }) | (if ptu { 2 } else { 0 });
            if duy > 0 && !kjl(&uz, duy) {
                return Err(VfsError::PermissionDenied);
            }
        }
    }
    
    let fd = {
        let vfs = Aj.read();
        vfs.alloc_fd()
    };
    
    let mut vfs = Aj.write();
    vfs.open_files.insert(fd, Il {
        ino,
        mount_idx,
        offset: 0,
        flags,
    });
    
    
    crate::lab_mode::trace_bus::emit(
        crate::lab_mode::trace_bus::EventCategory::Au,
        alloc::format!("open(\"{}\")", path),
        fd as u64,
    );
    
    Ok(fd)
}


pub fn read(fd: Cm, buf: &mut [u8]) -> E<usize> {
    let (mount_idx, ino, offset) = {
        let vfs = Aj.read();
        let file = vfs.open_files.get(&fd).ok_or(VfsError::BadFd)?;
        if !file.flags.readable() {
            return Err(VfsError::PermissionDenied);
        }
        (file.mount_idx, file.ino, file.offset)
    };
    
    let atf = {
        let vfs = Aj.read();
        let fs = &vfs.mounts[mount_idx].fs;
        let fwp = fs.get_file(ino)?;
        fwp.read(offset, buf)?
    };
    
    
    let mut vfs = Aj.write();
    if let Some(file) = vfs.open_files.get_mut(&fd) {
        file.offset += atf as u64;
    }
    
    Ok(atf)
}


pub fn write(fd: Cm, buf: &[u8]) -> E<usize> {
    let (mount_idx, ino, offset, append) = {
        let vfs = Aj.read();
        let file = vfs.open_files.get(&fd).ok_or(VfsError::BadFd)?;
        if !file.flags.writable() {
            return Err(VfsError::PermissionDenied);
        }
        (file.mount_idx, file.ino, file.offset, file.flags.append())
    };
    
    let cfn = if append {
        let vfs = Aj.read();
        let fs = &vfs.mounts[mount_idx].fs;
        let stat = fs.stat(ino)?;
        stat.size
    } else {
        offset
    };
    
    let atg = {
        let vfs = Aj.read();
        let fs = &vfs.mounts[mount_idx].fs;
        let fwp = fs.get_file(ino)?;
        fwp.write(cfn, buf)?
    };
    
    
    let mut vfs = Aj.write();
    if let Some(file) = vfs.open_files.get_mut(&fd) {
        file.offset = cfn + atg as u64;
    }
    
    
    crate::lab_mode::trace_bus::emit(
        crate::lab_mode::trace_bus::EventCategory::Au,
        alloc::format!("write fd={} {} bytes", fd, atg),
        atg as u64,
    );
    
    Ok(atg)
}


pub fn close(fd: Cm) -> E<()> {
    
    crate::lab_mode::trace_bus::emit(
        crate::lab_mode::trace_bus::EventCategory::Au,
        alloc::format!("close fd={}", fd),
        fd as u64,
    );
    
    let mut vfs = Aj.write();
    vfs.open_files.remove(&fd).ok_or(VfsError::BadFd)?;
    Ok(())
}


pub fn onc(fd: Cm, offset: i64, whence: i32) -> E<u64> {
    let mut vfs = Aj.write();
    let file = vfs.open_files.get_mut(&fd).ok_or(VfsError::BadFd)?;
    
    let iqd = match whence {
        0 => offset as u64,                          
        1 => (file.offset as i64 + offset) as u64,   
        2 => {
            
            drop(vfs);
            let size = {
                let vfs = Aj.read();
                let open_file = vfs.open_files.get(&fd).ok_or(VfsError::BadFd)?;
                let fs = &vfs.mounts[open_file.mount_idx].fs;
                fs.stat(open_file.ino)?.size
            };
            let mut vfs = Aj.write();
            let file = vfs.open_files.get_mut(&fd).ok_or(VfsError::BadFd)?;
            file.offset = (size as i64 + offset) as u64;
            return Ok(file.offset);
        }
        _ => return Err(VfsError::InvalidPath),
    };
    
    file.offset = iqd;
    Ok(iqd)
}


pub fn stat(path: &str) -> E<Stat> {
    let (mount_idx, ino) = resolve_path(path)?;
    let vfs = Aj.read();
    let fs = &vfs.mounts[mount_idx].fs;
    fs.stat(ino)
}


pub fn readdir(path: &str) -> E<Vec<Ap>> {
    let (mount_idx, ino) = resolve_path(path)?;
    let vfs = Aj.read();
    let fs = &vfs.mounts[mount_idx].fs;
    let it = fs.get_dir(ino)?;
    it.readdir()
}


pub fn mkdir(path: &str) -> E<()> {
    let parent_path = ewe(path);
    let cil = basename(path);
    
    let (mount_idx, parent_ino) = resolve_path(&parent_path)?;
    let vfs = Aj.read();
    let fs = &vfs.mounts[mount_idx].fs;
    let dwd = fs.get_dir(parent_ino)?;
    dwd.create(cil, FileType::Directory)?;
    Ok(())
}


pub fn nfn(path: &str) -> E<()> {
    let path = path.trim_end_matches('/');
    if path.is_empty() || path == "/" {
        return Ok(()); 
    }
    
    
    if mkdir(path).is_ok() {
        return Ok(());
    }
    
    
    let parent = ewe(path);
    if !parent.is_empty() && parent != "/" {
        nfn(&parent)?;
    }
    
    
    mkdir(path)
}


pub fn unlink(path: &str) -> E<()> {
    let parent_path = ewe(path);
    let filename = basename(path);
    
    let (mount_idx, parent_ino) = resolve_path(&parent_path)?;
    let vfs = Aj.read();
    let fs = &vfs.mounts[mount_idx].fs;
    let dwd = fs.get_dir(parent_ino)?;
    dwd.unlink(filename)
}


fn ewe(path: &str) -> String {
    if let Some(pos) = path.rfind('/') {
        if pos == 0 {
            "/".to_string()
        } else {
            path[..pos].to_string()
        }
    } else {
        "/".to_string()
    }
}


fn basename(path: &str) -> &str {
    if let Some(pos) = path.rfind('/') {
        &path[pos + 1..]
    } else {
        path
    }
}


pub fn dtl() -> Vec<(String, String)> {
    let vfs = Aj.read();
    vfs.mounts.iter().map(|ic| (ic.path.clone(), ic.fs.name().to_string())).collect()
}






pub fn nbd(fd: Cm, offset: i64, whence: u32) -> E<u64> {
    onc(fd, offset, whence as i32)
}


static Ri: RwLock<String> = RwLock::new(String::new());


pub fn eof() -> String {
    let cwd = Ri.read();
    if cwd.is_empty() {
        "/".to_string()
    } else {
        cwd.clone()
    }
}


pub fn kir(path: &str) -> E<()> {
    
    let owo = stat(path)?;
    if owo.file_type != FileType::Directory {
        return Err(VfsError::NotDirectory);
    }
    
    
    let giz = if path.starts_with('/') {
        path.to_string()
    } else {
        let current = eof();
        if current == "/" {
            format!("/{}", path)
        } else {
            format!("{}/{}", current, path)
        }
    };
    
    *Ri.write() = giz;
    Ok(())
}


pub fn qkz() {
    *Ri.write() = "/".to_string();
}






pub fn read_file(path: &str) -> E<Vec<u8>> {
    let fd = open(path, OpenFlags(OpenFlags::PM_))?;
    
    
    let own = stat(path)?;
    let size = own.size as usize;
    
    
    let mut buffer = alloc::vec![0u8; size.max(1024)];
    let mut offset = 0;
    while offset < buffer.len() {
        let ae = read(fd, &mut buffer[offset..])?;
        if ae == 0 { break; }
        offset += ae;
    }
    buffer.truncate(offset);
    
    close(fd)?;
    Ok(buffer)
}


pub fn gqh(path: &str) -> E<String> {
    let bytes = read_file(path)?;
    String::from_utf8(bytes).map_err(|_| VfsError::InvalidData)
}


pub fn write_file(path: &str, data: &[u8]) -> E<()> {
    
    let _ = mkdir(path); 
    
    let fd = open(path, OpenFlags(OpenFlags::PN_ | OpenFlags::PL_ | OpenFlags::BEJ_))?;
    let mut offset = 0;
    while offset < data.len() {
        let ae = write(fd, &data[offset..])?;
        if ae == 0 { break; }
        offset += ae;
    }
    close(fd)?;
    Ok(())
}


pub fn jkk() -> E<()> {
    let vfs = Aj.read();
    for abd in vfs.mounts.iter() {
        let _ = abd.fs.sync();
    }
    
    let _ = block_cache::sync();
    Ok(())
}


pub fn ftn(old_fd: Cm) -> E<Cm> {
    let vfs = Aj.read();
    let file = vfs.open_files.get(&old_fd).ok_or(VfsError::BadFd)?;
    let copy = Il {
        ino: file.ino,
        mount_idx: file.mount_idx,
        offset: file.offset,
        flags: file.flags,
    };
    let ue = vfs.alloc_fd();
    drop(vfs);
    let mut vfs = Aj.write();
    vfs.open_files.insert(ue, copy);
    Ok(ue)
}


pub fn hui(old_fd: Cm, ue: Cm) -> E<Cm> {
    if old_fd == ue {
        if Aj.read().open_files.contains_key(&old_fd) { return Ok(ue); }
        return Err(VfsError::BadFd);
    }
    let vfs = Aj.read();
    let file = vfs.open_files.get(&old_fd).ok_or(VfsError::BadFd)?;
    let copy = Il {
        ino: file.ino,
        mount_idx: file.mount_idx,
        offset: file.offset,
        flags: file.flags,
    };
    drop(vfs);
    let mut vfs = Aj.write();
    vfs.open_files.remove(&ue);
    vfs.open_files.insert(ue, copy);
    Ok(ue)
}


pub fn lzw(fd: Cm) -> E<Stat> {
    let vfs = Aj.read();
    let file = vfs.open_files.get(&fd).ok_or(VfsError::BadFd)?;
    let mount_idx = file.mount_idx;
    let ino = file.ino;
    let fs = &vfs.mounts[mount_idx].fs;
    fs.stat(ino)
}






pub struct Kk {
    
    pub readable: bool,
    
    pub writable: bool,
    
    pub error: bool,
    
    pub hangup: bool,
}



pub fn ivm(fd: Cm) -> Option<Kk> {
    
    if fd == 0 {
        return Some(Kk {
            readable: crate::keyboard::has_input(),
            writable: false,
            error: false,
            hangup: false,
        });
    }
    
    if fd == 1 || fd == 2 {
        return Some(Kk {
            readable: false,
            writable: true,
            error: false,
            hangup: false,
        });
    }
    
    if crate::pipe::dab(fd) {
        let (has_data, gab, ewo) = crate::pipe::poll(fd);
        return Some(Kk {
            readable: has_data,
            writable: gab,
            error: false,
            hangup: ewo,
        });
    }
    
    if crate::netstack::socket::mts(fd) {
        let has_data = crate::netstack::socket::mjy(fd);
        return Some(Kk {
            readable: has_data,
            writable: true, 
            error: false,
            hangup: false,
        });
    }
    
    let vfs = Aj.read();
    if vfs.open_files.contains_key(&fd) {
        return Some(Kk {
            readable: true,
            writable: true,
            error: false,
            hangup: false,
        });
    }
    
    None
}







pub fn kjl(uz: &Stat, want: u32) -> bool {
    let (uid, gid, euid, egid) = crate::process::bfs();
    
    if euid == 0 { return true; }

    let mode = uz.mode;
    let bits = if euid == uz.uid {
        (mode >> 6) & 7 
    } else if egid == uz.gid {
        (mode >> 3) & 7 
    } else {
        mode & 7 
    };
    (bits & want) == want
}


pub fn kkf(path: &str, mode: u32) -> E<()> {
    let (_, _, euid, _) = crate::process::bfs();
    let uz = stat(path)?;
    
    if euid != 0 && euid != uz.uid {
        return Err(VfsError::PermissionDenied);
    }
    let (mount_idx, ino) = resolve_path(path)?;
    let vfs = Aj.read();
    let _ = jfd(&*vfs.mounts[mount_idx].fs, ino, mode);
    Ok(())
}


pub fn lum(fd: Cm, mode: u32) -> E<()> {
    let (_, _, euid, _) = crate::process::bfs();
    let vfs = Aj.read();
    let file = vfs.open_files.get(&fd).ok_or(VfsError::BadFd)?;
    let mount_idx = file.mount_idx;
    let ino = file.ino;
    let uz = vfs.mounts[mount_idx].fs.stat(ino)?;
    if euid != 0 && euid != uz.uid {
        return Err(VfsError::PermissionDenied);
    }
    let _ = jfd(&*vfs.mounts[mount_idx].fs, ino, mode);
    Ok(())
}


pub fn kkh(path: &str, uid: u32, gid: u32) -> E<()> {
    let (mount_idx, ino) = resolve_path(path)?;
    let vfs = Aj.read();
    let _ = jfe(&*vfs.mounts[mount_idx].fs, ino, uid, gid);
    Ok(())
}


pub fn luo(fd: Cm, uid: u32, gid: u32) -> E<()> {
    let vfs = Aj.read();
    let file = vfs.open_files.get(&fd).ok_or(VfsError::BadFd)?;
    let _ = jfe(&*vfs.mounts[file.mount_idx].fs, file.ino, uid, gid);
    Ok(())
}


fn jfd(_fs: &dyn crate::vfs::Au, ino: K, mode: u32) -> E<()> {
    
    BCN_.lock().insert(ino, Ts { mode: Some(mode), uid: None, gid: None });
    Ok(())
}


fn jfe(_fs: &dyn crate::vfs::Au, ino: K, uid: u32, gid: u32) -> E<()> {
    let mut ayx = BCN_.lock();
    let entry = ayx.entry(ino).or_insert(Ts { mode: None, uid: None, gid: None });
    if uid != 0xFFFFFFFF { entry.uid = Some(uid); }
    if gid != 0xFFFFFFFF { entry.gid = Some(gid); }
    Ok(())
}


#[derive(Clone, Debug)]
struct Ts {
    mode: Option<u32>,
    uid: Option<u32>,
    gid: Option<u32>,
}

use spin::Mutex as SpinMutex;
use alloc::collections::BTreeMap as OverlayMap;
static BCN_: SpinMutex<OverlayMap<K, Ts>> = SpinMutex::new(OverlayMap::new());
