//! RAM Filesystem
//! 
//! Simple in-memory filesystem for T-RustOs.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::format;
use spin::Mutex;

/// Maximum file size (32 MB for large downloads)
pub const MAX_FILE_SIZE: usize = 32 * 1024 * 1024;

/// File types
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FileType {
    File,
    Directory,
}

/// File/Directory entry
#[derive(Clone)]
pub struct FsEntry {
    pub name: String,
    pub file_type: FileType,
    pub content: Vec<u8>,
    pub children: Vec<String>, // For directories: child names
    pub created_at: u64,       // Ticks at creation
    pub modified_at: u64,      // Ticks at last modification
}

impl FsEntry {
    pub fn new_file(name: &str) -> Self {
        let ticks = crate::logger::get_ticks();
        Self {
            name: String::from(name),
            file_type: FileType::File,
            content: Vec::new(),
            children: Vec::new(),
            created_at: ticks,
            modified_at: ticks,
        }
    }
    
    pub fn new_dir(name: &str) -> Self {
        let ticks = crate::logger::get_ticks();
        Self {
            name: String::from(name),
            file_type: FileType::Directory,
            content: Vec::new(),
            children: Vec::new(),
            created_at: ticks,
            modified_at: ticks,
        }
    }
}

/// Global filesystem
pub struct RamFs {
    entries: BTreeMap<String, FsEntry>,
    current_dir: String,
}

impl RamFs {
    pub const fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
            current_dir: String::new(),
        }
    }
    
    pub fn init(&mut self) {
        // Create root directory
        self.current_dir.clear();
        self.entries.insert(String::from("/"), FsEntry::new_dir("/"));
        
        // Create default directories
        self.mkdir("/home").ok();
        self.mkdir("/tmp").ok();
        self.mkdir("/bin").ok();
        self.mkdir("/etc").ok();
        
        // Create a welcome file
        self.touch("/home/welcome.txt").ok();
        self.write_file("/home/welcome.txt", b"Welcome to T-RustOs!\n\nThis is a RAM filesystem.\nAll files are stored in memory.\n").ok();
        
        // Create /etc/hostname
        self.touch("/etc/hostname").ok();
        self.write_file("/etc/hostname", b"trustos").ok();
        
        // Create /etc/version
        self.touch("/etc/version").ok();
        self.write_file("/etc/version", b"T-RustOs v0.1.0\n").ok();
    }
    
    /// Get current directory
    pub fn pwd(&self) -> &str {
        if self.current_dir.is_empty() {
            "/"
        } else {
            &self.current_dir
        }
    }
    
    /// Change directory
    pub fn cd(&mut self, path: &str) -> Result<(), FsError> {
        let abs_path = self.resolve_path(path);
        
        if let Some(entry) = self.entries.get(&abs_path) {
            if entry.file_type == FileType::Directory {
                self.current_dir = abs_path;
                Ok(())
            } else {
                Err(FsError::NotADirectory)
            }
        } else {
            Err(FsError::NotFound)
        }
    }
    
    /// List directory contents
    pub fn ls(&self, path: Option<&str>) -> Result<Vec<(String, FileType, usize)>, FsError> {
        let dir_path = match path {
            Some(p) => self.resolve_path(p),
            None => {
                if self.current_dir.is_empty() {
                    String::from("/")
                } else {
                    self.current_dir.clone()
                }
            }
        };
        
        if let Some(entry) = self.entries.get(&dir_path) {
            if entry.file_type != FileType::Directory {
                return Err(FsError::NotADirectory);
            }
            
            let mut items: Vec<(String, FileType, usize)> = Vec::new();
            for child_name in &entry.children {
                let child_path = if dir_path == "/" {
                    format!("/{}", child_name)
                } else {
                    format!("{}/{}", dir_path, child_name)
                };
                
                if let Some(child) = self.entries.get(&child_path) {
                    let size = if child.file_type == FileType::File {
                        child.content.len()
                    } else {
                        child.children.len()
                    };
                    items.push((child_name.clone(), child.file_type, size));
                }
            }
            Ok(items)
        } else {
            Err(FsError::NotFound)
        }
    }
    
    /// Create a directory
    pub fn mkdir(&mut self, path: &str) -> Result<(), FsError> {
        let abs_path = self.resolve_path(path);
        
        if self.entries.contains_key(&abs_path) {
            return Err(FsError::AlreadyExists);
        }
        
        // Get parent directory
        let parent_path = self.parent_path(&abs_path);
        let name = self.basename(&abs_path);
        
        // Check parent exists and is a directory
        if let Some(parent) = self.entries.get_mut(&parent_path) {
            if parent.file_type != FileType::Directory {
                return Err(FsError::NotADirectory);
            }
            parent.children.push(name.clone());
        } else {
            return Err(FsError::NotFound);
        }
        
        // Create directory entry
        self.entries.insert(abs_path, FsEntry::new_dir(&name));
        Ok(())
    }
    
    /// Create an empty file (touch)
    pub fn touch(&mut self, path: &str) -> Result<(), FsError> {
        let abs_path = self.resolve_path(path);
        
        if self.entries.contains_key(&abs_path) {
            // Update modified time
            if let Some(entry) = self.entries.get_mut(&abs_path) {
                entry.modified_at = crate::logger::get_ticks();
            }
            return Ok(());
        }
        
        // Get parent directory
        let parent_path = self.parent_path(&abs_path);
        let name = self.basename(&abs_path);
        
        // Check parent exists and is a directory
        if let Some(parent) = self.entries.get_mut(&parent_path) {
            if parent.file_type != FileType::Directory {
                return Err(FsError::NotADirectory);
            }
            parent.children.push(name.clone());
        } else {
            return Err(FsError::NotFound);
        }
        
        // Create file entry
        self.entries.insert(abs_path, FsEntry::new_file(&name));
        Ok(())
    }
    
    /// Remove a file or empty directory
    pub fn rm(&mut self, path: &str) -> Result<(), FsError> {
        let abs_path = self.resolve_path(path);
        
        if abs_path == "/" {
            return Err(FsError::PermissionDenied);
        }
        
        // Check entry exists
        if let Some(entry) = self.entries.get(&abs_path) {
            // If directory, must be empty
            if entry.file_type == FileType::Directory && !entry.children.is_empty() {
                return Err(FsError::DirectoryNotEmpty);
            }
        } else {
            return Err(FsError::NotFound);
        }
        
        // Remove from parent's children
        let parent_path = self.parent_path(&abs_path);
        let name = self.basename(&abs_path);
        
        if let Some(parent) = self.entries.get_mut(&parent_path) {
            parent.children.retain(|c| c != &name);
        }
        
        // Remove entry
        self.entries.remove(&abs_path);
        Ok(())
    }
    
    /// Read file contents
    pub fn read_file(&self, path: &str) -> Result<&[u8], FsError> {
        let abs_path = self.resolve_path(path);
        
        if let Some(entry) = self.entries.get(&abs_path) {
            if entry.file_type == FileType::File {
                Ok(&entry.content)
            } else {
                Err(FsError::IsADirectory)
            }
        } else {
            Err(FsError::NotFound)
        }
    }
    
    /// Write to file
    pub fn write_file(&mut self, path: &str, content: &[u8]) -> Result<(), FsError> {
        let abs_path = self.resolve_path(path);
        
        if let Some(entry) = self.entries.get_mut(&abs_path) {
            if entry.file_type == FileType::File {
                if content.len() > MAX_FILE_SIZE {
                    return Err(FsError::FileTooLarge);
                }
                entry.content = content.to_vec();
                entry.modified_at = crate::logger::get_ticks();
                Ok(())
            } else {
                Err(FsError::IsADirectory)
            }
        } else {
            Err(FsError::NotFound)
        }
    }
    
    /// Append to file
    pub fn append_file(&mut self, path: &str, content: &[u8]) -> Result<(), FsError> {
        let abs_path = self.resolve_path(path);
        
        if let Some(entry) = self.entries.get_mut(&abs_path) {
            if entry.file_type == FileType::File {
                if entry.content.len() + content.len() > MAX_FILE_SIZE {
                    return Err(FsError::FileTooLarge);
                }
                entry.content.extend_from_slice(content);
                entry.modified_at = crate::logger::get_ticks();
                Ok(())
            } else {
                Err(FsError::IsADirectory)
            }
        } else {
            Err(FsError::NotFound)
        }
    }
    
    /// Copy file
    pub fn cp(&mut self, src: &str, dst: &str) -> Result<(), FsError> {
        let src_path = self.resolve_path(src);
        let dst_path = self.resolve_path(dst);
        
        // Read source
        let content = {
            if let Some(entry) = self.entries.get(&src_path) {
                if entry.file_type != FileType::File {
                    return Err(FsError::IsADirectory);
                }
                entry.content.clone()
            } else {
                return Err(FsError::NotFound);
            }
        };
        
        // Create destination if it doesn't exist
        if !self.entries.contains_key(&dst_path) {
            self.touch(dst)?;
        }
        
        // Write content
        self.write_file(dst, &content)
    }
    
    /// Move/rename file
    pub fn mv(&mut self, src: &str, dst: &str) -> Result<(), FsError> {
        self.cp(src, dst)?;
        self.rm(src)?;
        Ok(())
    }
    
    /// Check if path exists
    pub fn exists(&self, path: &str) -> bool {
        let abs_path = self.resolve_path(path);
        self.entries.contains_key(&abs_path)
    }
    
    /// Get file/dir info
    pub fn stat(&self, path: &str) -> Result<&FsEntry, FsError> {
        let abs_path = self.resolve_path(path);
        self.entries.get(&abs_path).ok_or(FsError::NotFound)
    }
    
    // Helper: resolve relative path to absolute
    fn resolve_path(&self, path: &str) -> String {
        if path.starts_with('/') {
            self.normalize_path(path)
        } else if path == "~" || path.starts_with("~/") {
            let rest = if path == "~" { "" } else { &path[2..] };
            if rest.is_empty() {
                String::from("/home")
            } else {
                format!("/home/{}", rest)
            }
        } else {
            let cwd = if self.current_dir.is_empty() { "/" } else { &self.current_dir };
            if cwd == "/" {
                self.normalize_path(&format!("/{}", path))
            } else {
                self.normalize_path(&format!("{}/{}", cwd, path))
            }
        }
    }
    
    // Helper: normalize path (handle . and ..)
    fn normalize_path(&self, path: &str) -> String {
        let mut parts: Vec<&str> = Vec::new();
        
        for part in path.split('/') {
            match part {
                "" | "." => continue,
                ".." => { parts.pop(); }
                _ => parts.push(part),
            }
        }
        
        if parts.is_empty() {
            String::from("/")
        } else {
            format!("/{}", parts.join("/"))
        }
    }
    
    // Helper: get parent path
    fn parent_path(&self, path: &str) -> String {
        if let Some(pos) = path.rfind('/') {
            if pos == 0 {
                String::from("/")
            } else {
                String::from(&path[..pos])
            }
        } else {
            String::from("/")
        }
    }
    
    // Helper: get basename
    fn basename(&self, path: &str) -> String {
        if let Some(pos) = path.rfind('/') {
            String::from(&path[pos + 1..])
        } else {
            String::from(path)
        }
    }
}

/// Filesystem errors
#[derive(Debug, Clone, Copy)]
pub enum FsError {
    NotFound,
    AlreadyExists,
    NotADirectory,
    IsADirectory,
    DirectoryNotEmpty,
    FileTooLarge,
    PermissionDenied,
}

impl FsError {
    pub fn as_str(&self) -> &'static str {
        match self {
            FsError::NotFound => "not found",
            FsError::AlreadyExists => "already exists",
            FsError::NotADirectory => "not a directory",
            FsError::IsADirectory => "is a directory",
            FsError::DirectoryNotEmpty => "directory not empty",
            FsError::FileTooLarge => "file too large",
            FsError::PermissionDenied => "permission denied",
        }
    }
}

/// Global filesystem instance
static FILESYSTEM: Mutex<Option<RamFs>> = Mutex::new(None);

/// Initialize filesystem
pub fn init() {
    let mut fs = RamFs::new();
    fs.init();
    *FILESYSTEM.lock() = Some(fs);
}

/// Check if filesystem is initialized
pub fn is_initialized() -> bool {
    FILESYSTEM.lock().is_some()
}

/// Get filesystem (for commands)
pub fn with_fs<F, R>(f: F) -> R
where
    F: FnOnce(&mut RamFs) -> R,
{
    let mut guard = FILESYSTEM.lock();
    f(guard.as_mut().expect("Filesystem not initialized"))
}
