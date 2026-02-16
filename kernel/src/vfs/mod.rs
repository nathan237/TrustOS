//! Virtual File System (VFS)
//!
//! Provides a unified interface for all filesystems in TrustOS.
//! Inspired by Linux VFS architecture.

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

/// File descriptor type
pub type Fd = i32;

/// Inode number
pub type Ino = u64;

/// File open flags
#[derive(Clone, Copy, Debug)]
pub struct OpenFlags(pub u32);

impl OpenFlags {
    pub const O_RDONLY: u32 = 0;
    pub const O_WRONLY: u32 = 1;
    pub const O_RDWR: u32 = 2;
    pub const O_CREAT: u32 = 0o100;
    pub const O_TRUNC: u32 = 0o1000;
    pub const O_APPEND: u32 = 0o2000;
    
    pub fn readable(&self) -> bool {
        (self.0 & 3) != Self::O_WRONLY
    }
    
    pub fn writable(&self) -> bool {
        (self.0 & 3) != Self::O_RDONLY
    }
    
    pub fn create(&self) -> bool {
        (self.0 & Self::O_CREAT) != 0
    }
    
    pub fn append(&self) -> bool {
        (self.0 & Self::O_APPEND) != 0
    }
}

/// File type
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileType {
    Regular,
    Directory,
    CharDevice,
    BlockDevice,
    Symlink,
    Pipe,
    Socket,
}

/// File metadata (stat)
#[derive(Clone, Debug)]
pub struct Stat {
    pub ino: Ino,
    pub file_type: FileType,
    pub size: u64,
    pub blocks: u64,
    pub block_size: u32,
    pub mode: u32,      // permissions
    pub uid: u32,
    pub gid: u32,
    pub atime: u64,     // access time
    pub mtime: u64,     // modification time
    pub ctime: u64,     // creation time
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

/// Directory entry
#[derive(Clone, Debug)]
pub struct DirEntry {
    pub name: String,
    pub ino: Ino,
    pub file_type: FileType,
}

/// VFS Error types
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

pub type VfsResult<T> = Result<T, VfsError>;

/// File operations trait - implemented by each filesystem
pub trait FileOps: Send + Sync {
    fn read(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize>;
    fn write(&self, offset: u64, buf: &[u8]) -> VfsResult<usize>;
    fn stat(&self) -> VfsResult<Stat>;
    fn truncate(&self, size: u64) -> VfsResult<()> { 
        let _ = size;
        Err(VfsError::NotSupported) 
    }
    fn sync(&self) -> VfsResult<()> { Ok(()) }
}

/// Directory operations trait
pub trait DirOps: Send + Sync {
    fn lookup(&self, name: &str) -> VfsResult<Ino>;
    fn readdir(&self) -> VfsResult<Vec<DirEntry>>;
    fn create(&self, name: &str, file_type: FileType) -> VfsResult<Ino>;
    fn unlink(&self, name: &str) -> VfsResult<()>;
    fn stat(&self) -> VfsResult<Stat>;
}

/// Filesystem trait - mount point handler
pub trait FileSystem: Send + Sync {
    fn name(&self) -> &str;
    fn root_inode(&self) -> Ino;
    fn get_file(&self, ino: Ino) -> VfsResult<Arc<dyn FileOps>>;
    fn get_dir(&self, ino: Ino) -> VfsResult<Arc<dyn DirOps>>;
    fn stat(&self, ino: Ino) -> VfsResult<Stat>;
    fn sync(&self) -> VfsResult<()> { Ok(()) }
}

/// Mount point entry
struct MountPoint {
    path: String,
    fs: Arc<dyn FileSystem>,
}

/// Open file handle
struct OpenFile {
    ino: Ino,
    mount_idx: usize,
    offset: u64,
    flags: OpenFlags,
}

/// Global VFS state
struct Vfs {
    mounts: Vec<MountPoint>,
    open_files: BTreeMap<Fd, OpenFile>,
    next_fd: AtomicU64,
}

impl Vfs {
    const fn new() -> Self {
        Self {
            mounts: Vec::new(),
            open_files: BTreeMap::new(),
            next_fd: AtomicU64::new(3), // 0,1,2 reserved for stdin/out/err
        }
    }
    
    fn alloc_fd(&self) -> Fd {
        self.next_fd.fetch_add(1, Ordering::SeqCst) as Fd
    }
}

static VFS: RwLock<Vfs> = RwLock::new(Vfs::new());

/// Initialize VFS with default mounts
pub fn init() {
    crate::log!("[VFS] Initializing Virtual File System...");
    
    // Mount devfs at /dev
    if let Ok(devfs) = devfs::DevFs::new() {
        mount("/dev", Arc::new(devfs)).ok();
        crate::log_debug!("[VFS] Mounted devfs at /dev");
    }
    
    // Mount procfs at /proc
    if let Ok(procfs) = procfs::ProcFs::new() {
        mount("/proc", Arc::new(procfs)).ok();
        crate::log_debug!("[VFS] Mounted procfs at /proc");
    }
    
    // Mount TrustFS as root — try virtio-blk first, then AHCI
    let mut root_mounted = false;
    
    if crate::virtio_blk::is_initialized() {
        let backend = Arc::new(fat32::VirtioBlockDevice);
        let capacity = crate::virtio_blk::capacity();
        match trustfs::TrustFs::new(backend, capacity) {
            Ok(trustfs) => {
                mount("/", Arc::new(trustfs)).ok();
                crate::log!("[VFS] Mounted TrustFS at / (virtio-blk, persistent)");
                root_mounted = true;
            }
            Err(e) => {
                crate::log!("[VFS] TrustFS mount on virtio-blk failed: {:?}", e);
            }
        }
    }
    
    // Fallback: try TrustFS on AHCI data disk (e.g. VirtualBox)
    if !root_mounted && crate::drivers::ahci::is_initialized() {
        let devices = crate::drivers::ahci::list_devices();
        // Use the first non-optical AHCI port with sectors > 0
        for dev in &devices {
            if dev.sector_count > 64 {
                let backend = Arc::new(fat32::AhciBlockReader::new(dev.port_num as usize, 0));
                match trustfs::TrustFs::new(backend, dev.sector_count) {
                    Ok(trustfs) => {
                        mount("/", Arc::new(trustfs)).ok();
                        crate::log!("[VFS] Mounted TrustFS at / (AHCI port {}, persistent)", dev.port_num);
                        root_mounted = true;
                        break;
                    }
                    Err(e) => {
                        crate::log_debug!("[VFS] TrustFS on AHCI port {} failed: {:?}", dev.port_num, e);
                    }
                }
            }
        }
    }
    
    if !root_mounted {
        crate::log_debug!("[VFS] No block device, root will be ramfs");
    }
    
    // Try to mount FAT32 partitions from AHCI
    crate::log_debug!("[VFS] Looking for FAT32 partitions...");
    if let Some(fat32_fs) = fat32::try_mount_fat32() {
        mount("/mnt/fat32", fat32_fs).ok();
        crate::log!("[VFS] Mounted FAT32 at /mnt/fat32");
    } else {
        crate::log_debug!("[VFS] No FAT32 partition found");
    }
    
    crate::log!("[OK] VFS initialized");
}

/// Mount a filesystem at a path
pub fn mount(path: &str, fs: Arc<dyn FileSystem>) -> VfsResult<()> {
    let mut vfs = VFS.write();
    
    // Check if already mounted
    for mp in &vfs.mounts {
        if mp.path == path {
            return Err(VfsError::Busy);
        }
    }
    
    vfs.mounts.push(MountPoint {
        path: String::from(path),
        fs,
    });
    
    // Sort mounts by path length (longest first) for proper lookup
    vfs.mounts.sort_by(|a, b| b.path.len().cmp(&a.path.len()));
    
    Ok(())
}

/// Find the mount point for a path
fn find_mount(path: &str) -> Option<(usize, String)> {
    let vfs = VFS.read();
    
    for (idx, mp) in vfs.mounts.iter().enumerate() {
        if path == mp.path || path.starts_with(&format!("{}/", mp.path)) || mp.path == "/" {
            let relative = if mp.path == "/" {
                path.to_string()
            } else {
                path.strip_prefix(&mp.path).unwrap_or("/").to_string()
            };
            return Some((idx, if relative.is_empty() { "/".to_string() } else { relative }));
        }
    }
    
    None
}

/// Resolve a path to an inode
fn resolve_path(path: &str) -> VfsResult<(usize, Ino)> {
    let (mount_idx, relative) = find_mount(path).ok_or(VfsError::NotFound)?;
    
    let vfs = VFS.read();
    let fs = &vfs.mounts[mount_idx].fs;
    
    if relative == "/" || relative.is_empty() {
        return Ok((mount_idx, fs.root_inode()));
    }
    
    // Walk the path components
    let mut current_ino = fs.root_inode();
    let components: Vec<&str> = relative.split('/').filter(|s| !s.is_empty()).collect();
    
    for component in components {
        let dir = fs.get_dir(current_ino)?;
        current_ino = dir.lookup(component)?;
    }
    
    Ok((mount_idx, current_ino))
}

/// Open a file
pub fn open(path: &str, flags: OpenFlags) -> VfsResult<Fd> {
    let (mount_idx, ino) = match resolve_path(path) {
        Ok(result) => result,
        Err(VfsError::NotFound) if flags.create() => {
            // Create the file
            let parent_path = parent_of(path);
            let filename = basename(path);
            
            let (mount_idx, parent_ino) = resolve_path(&parent_path)?;
            let vfs = VFS.read();
            let fs = &vfs.mounts[mount_idx].fs;
            let parent_dir = fs.get_dir(parent_ino)?;
            let ino = parent_dir.create(filename, FileType::Regular)?;
            drop(vfs);
            
            (mount_idx, ino)
        }
        Err(e) => return Err(e),
    };
    
    let fd = {
        let vfs = VFS.read();
        vfs.alloc_fd()
    };
    
    let mut vfs = VFS.write();
    vfs.open_files.insert(fd, OpenFile {
        ino,
        mount_idx,
        offset: 0,
        flags,
    });
    
    // TrustLab trace
    crate::lab_mode::trace_bus::emit(
        crate::lab_mode::trace_bus::EventCategory::FileSystem,
        alloc::format!("open(\"{}\")", path),
        fd as u64,
    );
    
    Ok(fd)
}

/// Read from a file descriptor
pub fn read(fd: Fd, buf: &mut [u8]) -> VfsResult<usize> {
    let (mount_idx, ino, offset) = {
        let vfs = VFS.read();
        let file = vfs.open_files.get(&fd).ok_or(VfsError::BadFd)?;
        if !file.flags.readable() {
            return Err(VfsError::PermissionDenied);
        }
        (file.mount_idx, file.ino, file.offset)
    };
    
    let bytes_read = {
        let vfs = VFS.read();
        let fs = &vfs.mounts[mount_idx].fs;
        let file_ops = fs.get_file(ino)?;
        file_ops.read(offset, buf)?
    };
    
    // Update offset
    let mut vfs = VFS.write();
    if let Some(file) = vfs.open_files.get_mut(&fd) {
        file.offset += bytes_read as u64;
    }
    
    Ok(bytes_read)
}

/// Write to a file descriptor
pub fn write(fd: Fd, buf: &[u8]) -> VfsResult<usize> {
    let (mount_idx, ino, offset, append) = {
        let vfs = VFS.read();
        let file = vfs.open_files.get(&fd).ok_or(VfsError::BadFd)?;
        if !file.flags.writable() {
            return Err(VfsError::PermissionDenied);
        }
        (file.mount_idx, file.ino, file.offset, file.flags.append())
    };
    
    let write_offset = if append {
        let vfs = VFS.read();
        let fs = &vfs.mounts[mount_idx].fs;
        let stat = fs.stat(ino)?;
        stat.size
    } else {
        offset
    };
    
    let bytes_written = {
        let vfs = VFS.read();
        let fs = &vfs.mounts[mount_idx].fs;
        let file_ops = fs.get_file(ino)?;
        file_ops.write(write_offset, buf)?
    };
    
    // Update offset
    let mut vfs = VFS.write();
    if let Some(file) = vfs.open_files.get_mut(&fd) {
        file.offset = write_offset + bytes_written as u64;
    }
    
    // TrustLab trace
    crate::lab_mode::trace_bus::emit(
        crate::lab_mode::trace_bus::EventCategory::FileSystem,
        alloc::format!("write fd={} {} bytes", fd, bytes_written),
        bytes_written as u64,
    );
    
    Ok(bytes_written)
}

/// Close a file descriptor
pub fn close(fd: Fd) -> VfsResult<()> {
    // TrustLab trace
    crate::lab_mode::trace_bus::emit(
        crate::lab_mode::trace_bus::EventCategory::FileSystem,
        alloc::format!("close fd={}", fd),
        fd as u64,
    );
    
    let mut vfs = VFS.write();
    vfs.open_files.remove(&fd).ok_or(VfsError::BadFd)?;
    Ok(())
}

/// Seek in a file
pub fn seek(fd: Fd, offset: i64, whence: i32) -> VfsResult<u64> {
    let mut vfs = VFS.write();
    let file = vfs.open_files.get_mut(&fd).ok_or(VfsError::BadFd)?;
    
    let new_offset = match whence {
        0 => offset as u64,                          // SEEK_SET
        1 => (file.offset as i64 + offset) as u64,   // SEEK_CUR
        2 => {
            // SEEK_END - need file size
            drop(vfs);
            let size = {
                let vfs = VFS.read();
                let open_file = vfs.open_files.get(&fd).ok_or(VfsError::BadFd)?;
                let fs = &vfs.mounts[open_file.mount_idx].fs;
                fs.stat(open_file.ino)?.size
            };
            let mut vfs = VFS.write();
            let file = vfs.open_files.get_mut(&fd).ok_or(VfsError::BadFd)?;
            file.offset = (size as i64 + offset) as u64;
            return Ok(file.offset);
        }
        _ => return Err(VfsError::InvalidPath),
    };
    
    file.offset = new_offset;
    Ok(new_offset)
}

/// Get file statistics
pub fn stat(path: &str) -> VfsResult<Stat> {
    let (mount_idx, ino) = resolve_path(path)?;
    let vfs = VFS.read();
    let fs = &vfs.mounts[mount_idx].fs;
    fs.stat(ino)
}

/// Read directory entries
pub fn readdir(path: &str) -> VfsResult<Vec<DirEntry>> {
    let (mount_idx, ino) = resolve_path(path)?;
    let vfs = VFS.read();
    let fs = &vfs.mounts[mount_idx].fs;
    let dir = fs.get_dir(ino)?;
    dir.readdir()
}

/// Create a directory
pub fn mkdir(path: &str) -> VfsResult<()> {
    let parent_path = parent_of(path);
    let dirname = basename(path);
    
    let (mount_idx, parent_ino) = resolve_path(&parent_path)?;
    let vfs = VFS.read();
    let fs = &vfs.mounts[mount_idx].fs;
    let parent_dir = fs.get_dir(parent_ino)?;
    parent_dir.create(dirname, FileType::Directory)?;
    Ok(())
}

/// Create a directory and all parent directories (like mkdir -p)
pub fn mkdir_p(path: &str) -> VfsResult<()> {
    let path = path.trim_end_matches('/');
    if path.is_empty() || path == "/" {
        return Ok(()); // Root exists
    }
    
    // Try creating the directory first
    if mkdir(path).is_ok() {
        return Ok(());
    }
    
    // If failed, try creating parent first
    let parent = parent_of(path);
    if !parent.is_empty() && parent != "/" {
        mkdir_p(&parent)?;
    }
    
    // Now try creating this directory again
    mkdir(path)
}

/// Remove a file or directory
pub fn unlink(path: &str) -> VfsResult<()> {
    let parent_path = parent_of(path);
    let filename = basename(path);
    
    let (mount_idx, parent_ino) = resolve_path(&parent_path)?;
    let vfs = VFS.read();
    let fs = &vfs.mounts[mount_idx].fs;
    let parent_dir = fs.get_dir(parent_ino)?;
    parent_dir.unlink(filename)
}

/// Get parent path
fn parent_of(path: &str) -> String {
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

/// Get basename
fn basename(path: &str) -> &str {
    if let Some(pos) = path.rfind('/') {
        &path[pos + 1..]
    } else {
        path
    }
}

/// List mounted filesystems
pub fn list_mounts() -> Vec<(String, String)> {
    let vfs = VFS.read();
    vfs.mounts.iter().map(|mp| (mp.path.clone(), mp.fs.name().to_string())).collect()
}

// ============================================================================
// Additional functions for Linux syscall compatibility
// ============================================================================

/// lseek - seek in file (Linux syscall compatible wrapper)
pub fn lseek(fd: Fd, offset: i64, whence: u32) -> VfsResult<u64> {
    seek(fd, offset, whence as i32)
}

/// Current working directory per-process
static CWD: RwLock<String> = RwLock::new(String::new());

/// Get current working directory
pub fn getcwd() -> String {
    let cwd = CWD.read();
    if cwd.is_empty() {
        "/".to_string()
    } else {
        cwd.clone()
    }
}

/// Change current working directory
pub fn chdir(path: &str) -> VfsResult<()> {
    // Verify the path exists and is a directory
    let stat_result = stat(path)?;
    if stat_result.file_type != FileType::Directory {
        return Err(VfsError::NotDirectory);
    }
    
    // Normalize path
    let new_cwd = if path.starts_with('/') {
        path.to_string()
    } else {
        let current = getcwd();
        if current == "/" {
            format!("/{}", path)
        } else {
            format!("{}/{}", current, path)
        }
    };
    
    *CWD.write() = new_cwd;
    Ok(())
}

/// Initialize CWD
pub fn init_cwd() {
    *CWD.write() = "/".to_string();
}

// ═══════════════════════════════════════════════════════════════════════════════
// UTILITY FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════════

/// Read entire file contents as bytes
pub fn read_file(path: &str) -> VfsResult<Vec<u8>> {
    let fd = open(path, OpenFlags(OpenFlags::O_RDONLY))?;
    
    // Get file size
    let stat_info = stat(path)?;
    let size = stat_info.size as usize;
    
    // Read all content
    let mut buffer = alloc::vec![0u8; size.max(1024)];
    let mut offset = 0;
    while offset < buffer.len() {
        let n = read(fd, &mut buffer[offset..])?;
        if n == 0 { break; }
        offset += n;
    }
    buffer.truncate(offset);
    
    close(fd)?;
    Ok(buffer)
}

/// Read entire file contents as String
pub fn read_to_string(path: &str) -> VfsResult<String> {
    let bytes = read_file(path)?;
    String::from_utf8(bytes).map_err(|_| VfsError::InvalidData)
}

/// Write bytes to a file (creates or truncates)
pub fn write_file(path: &str, data: &[u8]) -> VfsResult<()> {
    // Try to create if doesn't exist
    let _ = mkdir(path); // Ensure parent dir exists - will fail silently if file
    
    let fd = open(path, OpenFlags(OpenFlags::O_WRONLY | OpenFlags::O_CREAT | OpenFlags::O_TRUNC))?;
    let mut offset = 0;
    while offset < data.len() {
        let n = write(fd, &data[offset..])?;
        if n == 0 { break; }
        offset += n;
    }
    close(fd)?;
    Ok(())
}
