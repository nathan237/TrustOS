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
pub mod ext4;
pub mod ntfs;

/// File descriptor type
pub // Alias de type — donne un nouveau nom à un type existant pour la clarté.
type Fd = i32;

/// Inode number
pub // Alias de type — donne un nouveau nom à un type existant pour la clarté.
type Ino = u64;

/// File open flags
#[derive(Clone, Copy, Debug)]
// Structure publique — visible à l'extérieur de ce module.
pub struct OpenFlags(pub u32);

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl OpenFlags {
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const O_RDONLY: u32 = 0;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const O_WRONLY: u32 = 1;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const O_RDWR: u32 = 2;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const O_CREAT: u32 = 0o100;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const O_TRUNC: u32 = 0o1000;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const O_APPEND: u32 = 0o2000;
    
        // Fonction publique — appelable depuis d'autres modules.
pub fn readable(&self) -> bool {
        (self.0 & 3) != Self::O_WRONLY
    }
    
        // Fonction publique — appelable depuis d'autres modules.
pub fn writable(&self) -> bool {
        (self.0 & 3) != Self::O_RDONLY
    }
    
        // Fonction publique — appelable depuis d'autres modules.
pub fn create(&self) -> bool {
        (self.0 & Self::O_CREAT) != 0
    }
    
        // Fonction publique — appelable depuis d'autres modules.
pub fn append(&self) -> bool {
        (self.0 & Self::O_APPEND) != 0
    }
}

/// File type
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
// Énumération — un type qui peut être l'une de plusieurs variantes.
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
// Structure publique — visible à l'extérieur de ce module.
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

// Implémentation de trait — remplit un contrat comportemental.
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
// Structure publique — visible à l'extérieur de ce module.
pub struct DirectoryEntry {
    pub name: String,
    pub ino: Ino,
    pub file_type: FileType,
}

/// VFS Error types
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
// Énumération — un type qui peut être l'une de plusieurs variantes.
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

pub // Alias de type — donne un nouveau nom à un type existant pour la clarté.
type VfsResult<T> = Result<T, VfsError>;

/// File operations trait - implemented by each filesystem
pub trait FileOperations: Send + Sync {
    fn read(&self, offset: u64, buffer: &mut [u8]) -> VfsResult<usize>;
    fn write(&self, offset: u64, buffer: &[u8]) -> VfsResult<usize>;
    fn status(&self) -> VfsResult<Stat>;
    fn truncate(&self, size: u64) -> VfsResult<()> { 
        let _ = size;
        Err(VfsError::NotSupported) 
    }
    fn sync(&self) -> VfsResult<()> { Ok(()) }
}

/// Directory operations trait
pub trait DirectoryOperations: Send + Sync {
    fn lookup(&self, name: &str) -> VfsResult<Ino>;
    fn readdir(&self) -> VfsResult<Vec<DirectoryEntry>>;
    fn create(&self, name: &str, file_type: FileType) -> VfsResult<Ino>;
    fn unlink(&self, name: &str) -> VfsResult<()>;
    fn status(&self) -> VfsResult<Stat>;
}

/// Filesystem trait - mount point handler
pub trait FileSystem: Send + Sync {
    fn name(&self) -> &str;
    fn root_inode(&self) -> Ino;
    fn get_file(&self, ino: Ino) -> VfsResult<Arc<dyn FileOperations>>;
    fn get_directory(&self, ino: Ino) -> VfsResult<Arc<dyn DirectoryOperations>>;
    fn status(&self, ino: Ino) -> VfsResult<Stat>;
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
    mount_index: usize,
    offset: u64,
    flags: OpenFlags,
}

/// Global VFS state
struct Vfs {
    mounts: Vec<MountPoint>,
    open_files: BTreeMap<Fd, OpenFile>,
    next_fd: AtomicU64,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl Vfs {
    const fn new() -> Self {
        Self {
            mounts: Vec::new(),
            open_files: BTreeMap::new(),
            next_fd: AtomicU64::new(3), // 0,1,2 reserved for stdin/out/err
        }
    }
    
    fn allocator_fd(&self) -> Fd {
        self.next_fd.fetch_add(1, Ordering::SeqCst) as Fd
    }
}

// État global partagé protégé par un Mutex (verrou d'exclusion mutuelle).
static VFS: RwLock<Vfs> = RwLock::new(Vfs::new());

/// Set up fd 0 (stdin), 1 (stdout), 2 (stderr) pointing to /dev/console.
/// Called before entering Ring 3 so user processes can do read(0)/write(1)/write(2).
pub fn setup_stdio() {
    // Resolve /dev/console
    let (mount_index, ino) = // Correspondance de motifs — branchement exhaustif de Rust.
match resolve_path("/dev/console") {
        Ok(r) => r,
        Err(_) => {
            crate::serial_println!("[VFS] Warning: /dev/console not found, stdio unavailable");
            return;
        }
    };

    let mut vfs = VFS.write();
    // fd 0 — stdin (read-only)
    vfs.open_files.insert(0, OpenFile {
        ino,
        mount_index,
        offset: 0,
        flags: OpenFlags(0), // O_RDONLY
    });
    // fd 1 — stdout (write-only)
    vfs.open_files.insert(1, OpenFile {
        ino,
        mount_index,
        offset: 0,
        flags: OpenFlags(1), // O_WRONLY
    });
    // fd 2 — stderr (write-only)
    vfs.open_files.insert(2, OpenFile {
        ino,
        mount_index,
        offset: 0,
        flags: OpenFlags(1), // O_WRONLY
    });
}

/// Clean up stdio descriptors after user process exits.
pub fn cleanup_stdio() {
    let mut vfs = VFS.write();
    vfs.open_files.remove(&0);
    vfs.open_files.remove(&1);
    vfs.open_files.remove(&2);
}

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
                // Correspondance de motifs — branchement exhaustif de Rust.
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
        // Skip disks that have TWAV magic (audio data disks)
        for device in &devices {
            if device.sector_count > 64 {
                // Check if this disk has TWAV magic — skip audio data disks
                let mut probe = alloc::vec![0u8; 512];
                if crate::drivers::ahci::read_sectors(device.port_number, 0, 1, &mut probe).is_ok() {
                    if probe.len() >= 4 && &probe[0..4] == b"TWAV" {
                        crate::log!("[VFS] Skipping AHCI port {} (TWAV audio data disk)", device.port_number);
                        continue;
                    }
                }
                let backend = Arc::new(fat32::AhciBlockReader::new(device.port_number as usize, 0));
                                // Correspondance de motifs — branchement exhaustif de Rust.
match trustfs::TrustFs::new(backend, device.sector_count) {
                    Ok(trustfs) => {
                        mount("/", Arc::new(trustfs)).ok();
                        crate::log!("[VFS] Mounted TrustFS at / (AHCI port {}, persistent)", device.port_number);
                        root_mounted = true;
                        break;
                    }
                    Err(e) => {
                        crate::log_debug!("[VFS] TrustFS on AHCI port {} failed: {:?}", device.port_number, e);
                    }
                }
            }
        }
    }
    
    if !root_mounted {
        crate::log_debug!("[VFS] No block device, root will be ramfs");
    }
    
    // Try to mount FAT32 partitions from AHCI (x86_64 only — AHCI uses PCI I/O ports)
    #[cfg(target_arch = "x86_64")]
    {
        crate::log_debug!("[VFS] Looking for FAT32 partitions...");
        if let Some(fat32_filesystem) = fat32::try_mount_fat32() {
            mount("/mnt/fat32", fat32_filesystem).ok();
            crate::log!("[VFS] Mounted FAT32 at /mnt/fat32");
        } else {
            crate::log_debug!("[VFS] No FAT32 partition found");
        }
    }
    
    // Try to mount NTFS partitions from AHCI (x86_64 only)
    #[cfg(target_arch = "x86_64")]
    {
        crate::log_debug!("[VFS] Looking for NTFS partitions...");
        if let Some(ntfs_filesystem) = ntfs::try_mount_ntfs() {
            mount("/mnt/ntfs", ntfs_filesystem).ok();
            crate::log!("[VFS] Mounted NTFS at /mnt/ntfs");
        } else {
            crate::log_debug!("[VFS] No NTFS partition found");
        }
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

/// Unmount a filesystem at a path
pub fn umount(path: &str) -> VfsResult<()> {
    let mut vfs = VFS.write();
    
    let index = vfs.mounts.iter().position(|mp| mp.path == path)
        .ok_or(VfsError::NotFound)?;
    
    // Don't allow unmounting root
    if vfs.mounts[index].path == "/" {
        return Err(VfsError::PermissionDenied);
    }
    
    vfs.mounts.remove(index);
    Ok(())
}

/// Find the mount point for a path
fn find_mount(path: &str) -> Option<(usize, String)> {
    let vfs = VFS.read();
    
    for (index, mp) in vfs.mounts.iter().enumerate() {
        if path == mp.path || path.starts_with(&format!("{}/", mp.path)) || mp.path == "/" {
            let relative = if mp.path == "/" {
                path.to_string()
            } else {
                path.strip_prefix(&mp.path).unwrap_or("/").to_string()
            };
            return Some((index, if relative.is_empty() { "/".to_string() } else { relative }));
        }
    }
    
    None
}

/// Resolve a path to an inode
fn resolve_path(path: &str) -> VfsResult<(usize, Ino)> {
    let (mount_index, relative) = find_mount(path).ok_or(VfsError::NotFound)?;
    
    let vfs = VFS.read();
    let fs = &vfs.mounts[mount_index].fs;
    
    if relative == "/" || relative.is_empty() {
        return Ok((mount_index, fs.root_inode()));
    }
    
    // Walk the path components
    let mut current_ino = fs.root_inode();
    let components: Vec<&str> = relative.split('/').filter(|s| !s.is_empty()).collect();
    
    for component in components {
        let directory = fs.get_directory(current_ino)?;
        current_ino = directory.lookup(component)?;
    }
    
    Ok((mount_index, current_ino))
}

/// Open a file
pub fn open(path: &str, flags: OpenFlags) -> VfsResult<Fd> {
    let (mount_index, ino) = // Correspondance de motifs — branchement exhaustif de Rust.
match resolve_path(path) {
        Ok(result) => result,
        Err(VfsError::NotFound) if flags.create() => {
            // Create the file
            let parent_path = parent_of(path);
            let filename = basename(path);
            
            let (mount_index, parent_ino) = resolve_path(&parent_path)?;
            let vfs = VFS.read();
            let fs = &vfs.mounts[mount_index].fs;
            let parent_directory = fs.get_directory(parent_ino)?;
            let ino = parent_directory.create(filename, FileType::Regular)?;
            drop(vfs);
            
            (mount_index, ino)
        }
        Err(e) => return Err(e),
    };
    
    // Permission check: verify caller has access
    {
        let vfs = VFS.read();
        if let Ok(st) = vfs.mounts[mount_index].fs.status(ino) {
            // Determine needed permission from flags
            let want_read  = flags.readable();
            let want_write = flags.writable();
            let need = (if want_read { 4 } else { 0 }) | (if want_write { 2 } else { 0 });
            if need > 0 && !check_permission(&st, need) {
                return Err(VfsError::PermissionDenied);
            }
        }
    }
    
    let fd = {
        let vfs = VFS.read();
        vfs.allocator_fd()
    };
    
    let mut vfs = VFS.write();
    vfs.open_files.insert(fd, OpenFile {
        ino,
        mount_index,
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
pub fn read(fd: Fd, buffer: &mut [u8]) -> VfsResult<usize> {
    let (mount_index, ino, offset) = {
        let vfs = VFS.read();
        let file = vfs.open_files.get(&fd).ok_or(VfsError::BadFd)?;
        if !file.flags.readable() {
            return Err(VfsError::PermissionDenied);
        }
        (file.mount_index, file.ino, file.offset)
    };
    
    let bytes_read = {
        let vfs = VFS.read();
        let fs = &vfs.mounts[mount_index].fs;
        let file_operations = fs.get_file(ino)?;
        file_operations.read(offset, buffer)?
    };
    
    // Update offset
    let mut vfs = VFS.write();
    if let Some(file) = vfs.open_files.get_mut(&fd) {
        file.offset += bytes_read as u64;
    }
    
    Ok(bytes_read)
}

/// Write to a file descriptor
pub fn write(fd: Fd, buffer: &[u8]) -> VfsResult<usize> {
    let (mount_index, ino, offset, append) = {
        let vfs = VFS.read();
        let file = vfs.open_files.get(&fd).ok_or(VfsError::BadFd)?;
        if !file.flags.writable() {
            return Err(VfsError::PermissionDenied);
        }
        (file.mount_index, file.ino, file.offset, file.flags.append())
    };
    
    let write_offset = if append {
        let vfs = VFS.read();
        let fs = &vfs.mounts[mount_index].fs;
        let status = fs.status(ino)?;
        status.size
    } else {
        offset
    };
    
    let bytes_written = {
        let vfs = VFS.read();
        let fs = &vfs.mounts[mount_index].fs;
        let file_operations = fs.get_file(ino)?;
        file_operations.write(write_offset, buffer)?
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
    
    let new_offset = // Correspondance de motifs — branchement exhaustif de Rust.
match whence {
        0 => offset as u64,                          // SEEK_SET
        1 => (file.offset as i64 + offset) as u64,   // SEEK_CUR
        2 => {
            // SEEK_END - need file size
            drop(vfs);
            let size = {
                let vfs = VFS.read();
                let open_file = vfs.open_files.get(&fd).ok_or(VfsError::BadFd)?;
                let fs = &vfs.mounts[open_file.mount_index].fs;
                fs.status(open_file.ino)?.size
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
pub fn status(path: &str) -> VfsResult<Stat> {
    let (mount_index, ino) = resolve_path(path)?;
    let vfs = VFS.read();
    let fs = &vfs.mounts[mount_index].fs;
    fs.status(ino)
}

/// Read directory entries
pub fn readdir(path: &str) -> VfsResult<Vec<DirectoryEntry>> {
    let (mount_index, ino) = resolve_path(path)?;
    let vfs = VFS.read();
    let fs = &vfs.mounts[mount_index].fs;
    let directory = fs.get_directory(ino)?;
    directory.readdir()
}

/// Create a directory
pub fn mkdir(path: &str) -> VfsResult<()> {
    let parent_path = parent_of(path);
    let dirname = basename(path);
    
    let (mount_index, parent_ino) = resolve_path(&parent_path)?;
    let vfs = VFS.read();
    let fs = &vfs.mounts[mount_index].fs;
    let parent_directory = fs.get_directory(parent_ino)?;
    parent_directory.create(dirname, FileType::Directory)?;
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
    
    let (mount_index, parent_ino) = resolve_path(&parent_path)?;
    let vfs = VFS.read();
    let fs = &vfs.mounts[mount_index].fs;
    let parent_directory = fs.get_directory(parent_ino)?;
    parent_directory.unlink(filename)
}

/// Get parent path
fn parent_of(path: &str) -> String {
    if let Some(position) = path.rfind('/') {
        if position == 0 {
            "/".to_string()
        } else {
            path[..position].to_string()
        }
    } else {
        "/".to_string()
    }
}

/// Get basename
fn basename(path: &str) -> &str {
    if let Some(position) = path.rfind('/') {
        &path[position + 1..]
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
    let status_result = status(path)?;
    if status_result.file_type != FileType::Directory {
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
pub fn initialize_cwd() {
    *CWD.write() = "/".to_string();
}

// ═══════════════════════════════════════════════════════════════════════════════
// UTILITY FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════════

/// Read entire file contents as bytes
pub fn read_file(path: &str) -> VfsResult<Vec<u8>> {
    let fd = open(path, OpenFlags(OpenFlags::O_RDONLY))?;
    
    // Get file size
    let status_information = status(path)?;
    let size = status_information.size as usize;
    
    // Read all content
    let mut buffer = alloc::vec![0u8; size.maximum(1024)];
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
    String::from_utf8(bytes).map_error(|_| VfsError::InvalidData)
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

/// Sync all mounted filesystems to disk
pub fn sync_all() -> VfsResult<()> {
    let vfs = VFS.read();
    for mount in vfs.mounts.iter() {
        let _ = mount.fs.sync();
    }
    // Also flush block cache
    let _ = block_cache::sync();
    Ok(())
}

/// Duplicate a file descriptor (lowest available new fd)
pub fn dup_fd(old_fd: Fd) -> VfsResult<Fd> {
    let vfs = VFS.read();
    let file = vfs.open_files.get(&old_fd).ok_or(VfsError::BadFd)?;
    let copy = OpenFile {
        ino: file.ino,
        mount_index: file.mount_index,
        offset: file.offset,
        flags: file.flags,
    };
    let new_fd = vfs.allocator_fd();
    drop(vfs);
    let mut vfs = VFS.write();
    vfs.open_files.insert(new_fd, copy);
    Ok(new_fd)
}

/// Duplicate fd to a specific target number
pub fn dup2_fd(old_fd: Fd, new_fd: Fd) -> VfsResult<Fd> {
    if old_fd == new_fd {
        if VFS.read().open_files.contains_key(&old_fd) { return Ok(new_fd); }
        return Err(VfsError::BadFd);
    }
    let vfs = VFS.read();
    let file = vfs.open_files.get(&old_fd).ok_or(VfsError::BadFd)?;
    let copy = OpenFile {
        ino: file.ino,
        mount_index: file.mount_index,
        offset: file.offset,
        flags: file.flags,
    };
    drop(vfs);
    let mut vfs = VFS.write();
    vfs.open_files.remove(&new_fd);
    vfs.open_files.insert(new_fd, copy);
    Ok(new_fd)
}

/// Get file stat by fd
pub fn fstat_fd(fd: Fd) -> VfsResult<Stat> {
    let vfs = VFS.read();
    let file = vfs.open_files.get(&fd).ok_or(VfsError::BadFd)?;
    let mount_index = file.mount_index;
    let ino = file.ino;
    let fs = &vfs.mounts[mount_index].fs;
    fs.status(ino)
}

// ============================================================================
// Poll support — query readiness of file descriptors
// ============================================================================

/// Readiness status for a file descriptor
pub struct PollStatus {
    /// Data is available for reading
    pub readable: bool,
    /// Writing will not block
    pub writable: bool,
    /// Error condition on fd
    pub error: bool,
    /// Hang-up (peer closed)
    pub hangup: bool,
}

/// Query the readiness of a file descriptor.
/// Returns `None` if the fd is invalid.
pub fn poll_fd(fd: Fd) -> Option<PollStatus> {
    // stdin — readable if keyboard has data
    if fd == 0 {
        return Some(PollStatus {
            readable: crate::keyboard::has_input(),
            writable: false,
            error: false,
            hangup: false,
        });
    }
    // stdout / stderr — always writable
    if fd == 1 || fd == 2 {
        return Some(PollStatus {
            readable: false,
            writable: true,
            error: false,
            hangup: false,
        });
    }
    // Pipes
    if crate::pipe::is_pipe_fd(fd) {
        let (has_data, has_space, peer_closed) = crate::pipe::poll(fd);
        return Some(PollStatus {
            readable: has_data,
            writable: has_space,
            error: false,
            hangup: peer_closed,
        });
    }
    // Sockets
    if crate::netstack::socket::is_socket(fd) {
        let has_data = crate::netstack::socket::has_readable_data(fd);
        return Some(PollStatus {
            readable: has_data,
            writable: true, // TCP send buffer assumed non-full
            error: false,
            hangup: false,
        });
    }
    // Regular VFS files — always ready for regular files
    let vfs = VFS.read();
    if vfs.open_files.contains_key(&fd) {
        return Some(PollStatus {
            readable: true,
            writable: true,
            error: false,
            hangup: false,
        });
    }
    // Unknown fd
    None
}

// ============================================================================
// Permission checking
// ============================================================================

/// Check if current process has the requested permission on a file.
/// `want` is a bitmask: 4=read, 2=write, 1=execute
pub fn check_permission(st: &Stat, want: u32) -> bool {
    let (uid, gid, euid, egid) = crate::process::current_credentials();
    // Root can do anything
    if euid == 0 { return true; }

    let mode = st.mode;
    let bits = if euid == st.uid {
        (mode >> 6) & 7 // owner bits
    } else if egid == st.gid {
        (mode >> 3) & 7 // group bits
    } else {
        mode & 7 // other bits
    };
    (bits & want) == want
}

/// Change file mode bits
pub fn chmod(path: &str, mode: u32) -> VfsResult<()> {
    let (_, _, euid, _) = crate::process::current_credentials();
    let st = status(path)?;
    // Only owner or root can chmod
    if euid != 0 && euid != st.uid {
        return Err(VfsError::PermissionDenied);
    }
    let (mount_index, ino) = resolve_path(path)?;
    let vfs = VFS.read();
    let _ = set_file_mode(&*vfs.mounts[mount_index].fs, ino, mode);
    Ok(())
}

/// Change file mode by fd
pub fn fchmod(fd: Fd, mode: u32) -> VfsResult<()> {
    let (_, _, euid, _) = crate::process::current_credentials();
    let vfs = VFS.read();
    let file = vfs.open_files.get(&fd).ok_or(VfsError::BadFd)?;
    let mount_index = file.mount_index;
    let ino = file.ino;
    let st = vfs.mounts[mount_index].fs.status(ino)?;
    if euid != 0 && euid != st.uid {
        return Err(VfsError::PermissionDenied);
    }
    let _ = set_file_mode(&*vfs.mounts[mount_index].fs, ino, mode);
    Ok(())
}

/// Change file owner/group
pub fn chown(path: &str, uid: u32, gid: u32) -> VfsResult<()> {
    let (mount_index, ino) = resolve_path(path)?;
    let vfs = VFS.read();
    let _ = set_file_owner(&*vfs.mounts[mount_index].fs, ino, uid, gid);
    Ok(())
}

/// Change file owner/group by fd
pub fn fchown(fd: Fd, uid: u32, gid: u32) -> VfsResult<()> {
    let vfs = VFS.read();
    let file = vfs.open_files.get(&fd).ok_or(VfsError::BadFd)?;
    let _ = set_file_owner(&*vfs.mounts[file.mount_index].fs, file.ino, uid, gid);
    Ok(())
}

/// Helper: set mode bits in metadata overlay (works for ramfs/trustfs)
fn set_file_mode(_filesystem: &dyn crate::vfs::FileSystem, ino: Ino, mode: u32) -> VfsResult<()> {
    // For ramfs/trustfs we can modify the inode's mode directly via the metadata overlay
    METADATA_OVERLAY.lock().insert(ino, MetadataOverride { mode: Some(mode), uid: None, gid: None });
    Ok(())
}

/// Helper: set owner in metadata overlay
fn set_file_owner(_filesystem: &dyn crate::vfs::FileSystem, ino: Ino, uid: u32, gid: u32) -> VfsResult<()> {
    let mut overlay = METADATA_OVERLAY.lock();
    let entry = overlay.entry(ino).or_insert(MetadataOverride { mode: None, uid: None, gid: None });
    if uid != 0xFFFFFFFF { entry.uid = Some(uid); }
    if gid != 0xFFFFFFFF { entry.gid = Some(gid); }
    Ok(())
}

/// Metadata override storage (for chmod/chown on any FS)
#[derive(Clone, Debug)]
struct MetadataOverride {
    mode: Option<u32>,
    uid: Option<u32>,
    gid: Option<u32>,
}

use spin::Mutex as SpinMutex;
use alloc::collections::BTreeMap as OverlayMap;
// État global partagé protégé par un Mutex (verrou d'exclusion mutuelle).
static METADATA_OVERLAY: SpinMutex<OverlayMap<Ino, MetadataOverride>> = SpinMutex::new(OverlayMap::new());
