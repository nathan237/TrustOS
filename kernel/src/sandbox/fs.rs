// sandbox/fs.rs — Sandboxed virtual filesystem
// Each sandbox gets an isolated in-memory filesystem with:
// - Path jailing (no access outside sandbox root)
// - Quota enforcement (max files, max total bytes)
// - No access to the real kernel VFS
// - Cookie/cache/localStorage emulation

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

use super::SandboxId;

// ──── Types ────────────────────────────────────────────────────────────────

/// File type in sandbox filesystem
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SandboxFileType {
    /// Regular file with content
    File,
    /// Directory
    Directory,
}

/// A file entry in the sandbox filesystem
#[derive(Debug, Clone)]
pub struct SandboxFile {
    pub name: String,
    pub file_type: SandboxFileType,
    pub data: Vec<u8>,
    pub created_at: u64,
    pub modified_at: u64,
    /// Domain that created this file (for cookie isolation)
    pub origin: String,
    /// Read-only flag
    pub readonly: bool,
}

impl SandboxFile {
    fn new_file(name: &str, origin: &str) -> Self {
        let now = crate::time::uptime_ms();
        Self {
            name: String::from(name),
            file_type: SandboxFileType::File,
            data: Vec::new(),
            created_at: now,
            modified_at: now,
            origin: String::from(origin),
            readonly: false,
        }
    }

    fn new_dir(name: &str) -> Self {
        let now = crate::time::uptime_ms();
        Self {
            name: String::from(name),
            file_type: SandboxFileType::Directory,
            data: Vec::new(),
            created_at: now,
            modified_at: now,
            origin: String::new(),
            readonly: false,
        }
    }
}

/// Filesystem error
#[derive(Debug)]
pub enum FsError {
    NotFound,
    AlreadyExists,
    QuotaFiles,
    QuotaBytes,
    PathViolation,
    ReadOnly,
    InvalidPath,
    IsDirectory,
    NotDirectory,
}

// ──── Sandbox Filesystem ───────────────────────────────────────────────────

/// Per-sandbox isolated virtual filesystem
pub struct SandboxFs {
    sandbox_id: SandboxId,
    /// Flat map: normalized path → file entry
    files: BTreeMap<String, SandboxFile>,
    /// Limits
    max_files: usize,
    max_bytes: usize,
    /// Current usage
    total_bytes: usize,
}

impl SandboxFs {
    pub fn new(sandbox_id: SandboxId, max_files: usize, max_bytes: usize) -> Self {
        let mut fs = Self {
            sandbox_id,
            files: BTreeMap::new(),
            max_files,
            max_bytes,
            total_bytes: 0,
        };
        // Create root directories
        fs.files.insert(String::from("/"), SandboxFile::new_dir("/"));
        fs.files.insert(String::from("/cache"), SandboxFile::new_dir("cache"));
        fs.files.insert(String::from("/cookies"), SandboxFile::new_dir("cookies"));
        fs.files.insert(String::from("/storage"), SandboxFile::new_dir("storage"));
        fs.files.insert(String::from("/downloads"), SandboxFile::new_dir("downloads"));
        fs
    }

    /// Normalize and validate a path — jail to sandbox root
    fn normalize_path(&self, path: &str) -> Result<String, FsError> {
        // Must start with /
        let clean = if path.starts_with('/') {
            String::from(path)
        } else {
            format!("/{}", path)
        };

        // Block path traversal attacks
        if clean.contains("..") || clean.contains("./") || clean.contains("//") {
            return Err(FsError::PathViolation);
        }

        // Block null bytes and control characters
        if clean.bytes().any(|b| b == 0 || b < 0x20) {
            return Err(FsError::InvalidPath);
        }

        // Max path length
        if clean.len() > 256 {
            return Err(FsError::InvalidPath);
        }

        Ok(clean)
    }

    /// Write a file (create or overwrite)
    pub fn write(&mut self, path: &str, data: &[u8], origin: &str) -> Result<(), FsError> {
        let path = self.normalize_path(path)?;

        // Check if file exists and is read-only
        if let Some(existing) = self.files.get(&path) {
            if existing.readonly {
                return Err(FsError::ReadOnly);
            }
            if existing.file_type == SandboxFileType::Directory {
                return Err(FsError::IsDirectory);
            }
            // Update: account for size change
            let old_size = existing.data.len();
            let new_total = self.total_bytes - old_size + data.len();
            if new_total > self.max_bytes {
                return Err(FsError::QuotaBytes);
            }
            self.total_bytes = new_total;
        } else {
            // New file: check quotas
            if self.files.len() >= self.max_files {
                return Err(FsError::QuotaFiles);
            }
            if self.total_bytes + data.len() > self.max_bytes {
                return Err(FsError::QuotaBytes);
            }
            self.total_bytes += data.len();
        }

        let mut file = SandboxFile::new_file(
            path.rsplit('/').next().unwrap_or(&path),
            origin,
        );
        file.data = data.to_vec();
        self.files.insert(path, file);
        Ok(())
    }

    /// Read a file
    pub fn read(&self, path: &str) -> Result<&[u8], FsError> {
        let path = self.normalize_path(path)?;
        match self.files.get(&path) {
            Some(f) if f.file_type == SandboxFileType::File => Ok(&f.data),
            Some(_) => Err(FsError::IsDirectory),
            None => Err(FsError::NotFound),
        }
    }

    /// Delete a file
    pub fn delete(&mut self, path: &str) -> Result<(), FsError> {
        let path = self.normalize_path(path)?;
        match self.files.get(&path) {
            Some(f) => {
                if f.readonly {
                    return Err(FsError::ReadOnly);
                }
                if f.file_type == SandboxFileType::Directory {
                    // Only delete empty directories
                    let prefix = format!("{}/", path);
                    let has_children = self.files.keys().any(|k| k.starts_with(&prefix));
                    if has_children {
                        return Err(FsError::NotDirectory); // directory not empty
                    }
                }
                let size = f.data.len();
                self.files.remove(&path);
                self.total_bytes -= size;
                Ok(())
            }
            None => Err(FsError::NotFound),
        }
    }

    /// List files in a directory
    pub fn list(&self, dir_path: &str) -> Result<Vec<(&str, &SandboxFileType, usize)>, FsError> {
        let dir = self.normalize_path(dir_path)?;
        let prefix = if dir == "/" { String::from("/") } else { format!("{}/", dir) };

        let mut entries = Vec::new();
        for (path, file) in &self.files {
            if path == &dir { continue; } // skip the directory itself
            if path.starts_with(&prefix) {
                // Only direct children (no nested)
                let rest = &path[prefix.len()..];
                if !rest.contains('/') {
                    entries.push((path.as_str(), &file.file_type, file.data.len()));
                }
            }
        }
        Ok(entries)
    }

    /// Check if a path exists
    pub fn exists(&self, path: &str) -> bool {
        if let Ok(p) = self.normalize_path(path) {
            self.files.contains_key(&p)
        } else {
            false
        }
    }

    /// Create a directory
    pub fn mkdir(&mut self, path: &str) -> Result<(), FsError> {
        let path = self.normalize_path(path)?;
        if self.files.contains_key(&path) {
            return Err(FsError::AlreadyExists);
        }
        if self.files.len() >= self.max_files {
            return Err(FsError::QuotaFiles);
        }
        self.files.insert(path.clone(), SandboxFile::new_dir(
            path.rsplit('/').next().unwrap_or(&path)
        ));
        Ok(())
    }

    // ── Cookie/LocalStorage Helpers ────────────────────────────────────────

    /// Store a cookie for a domain
    pub fn set_cookie(&mut self, domain: &str, name: &str, value: &str) -> Result<(), FsError> {
        let path = format!("/cookies/{}_{}", domain.replace('.', "_"), name);
        self.write(&path, value.as_bytes(), domain)
    }

    /// Get a cookie for a domain
    pub fn get_cookie(&self, domain: &str, name: &str) -> Option<String> {
        let path = format!("/cookies/{}_{}", domain.replace('.', "_"), name);
        self.read(&path).ok()
            .map(|data| String::from_utf8_lossy(data).into_owned())
    }

    /// Store localStorage data for a domain
    pub fn local_storage_set(&mut self, domain: &str, key: &str, value: &str) -> Result<(), FsError> {
        let dir = format!("/storage/{}", domain.replace('.', "_"));
        if !self.exists(&dir) {
            self.mkdir(&dir)?;
        }
        let path = format!("{}/{}", dir, key);
        self.write(&path, value.as_bytes(), domain)
    }

    /// Get localStorage data for a domain
    pub fn local_storage_get(&self, domain: &str, key: &str) -> Option<String> {
        let path = format!("/storage/{}/{}", domain.replace('.', "_"), key);
        self.read(&path).ok()
            .map(|data| String::from_utf8_lossy(data).into_owned())
    }

    /// Cache a response body
    pub fn cache_response(&mut self, url: &str, data: &[u8]) -> Result<(), FsError> {
        // Simple hash of URL for filename
        let hash = url.bytes().fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
        let path = format!("/cache/{:016x}", hash);
        self.write(&path, data, "cache")
    }

    /// Get cached response
    pub fn get_cached(&self, url: &str) -> Option<&[u8]> {
        let hash = url.bytes().fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
        let path = format!("/cache/{:016x}", hash);
        self.read(&path).ok()
    }

    // ── Stats ──────────────────────────────────────────────────────────────

    /// Current filesystem usage
    pub fn usage(&self) -> (usize, usize, usize, usize) {
        let file_count = self.files.values()
            .filter(|f| f.file_type == SandboxFileType::File)
            .count();
        (file_count, self.max_files, self.total_bytes, self.max_bytes)
    }

    /// Clear all files (reset filesystem)
    pub fn clear(&mut self) {
        self.files.clear();
        self.total_bytes = 0;
        // Re-create root directories
        self.files.insert(String::from("/"), SandboxFile::new_dir("/"));
        self.files.insert(String::from("/cache"), SandboxFile::new_dir("cache"));
        self.files.insert(String::from("/cookies"), SandboxFile::new_dir("cookies"));
        self.files.insert(String::from("/storage"), SandboxFile::new_dir("storage"));
        self.files.insert(String::from("/downloads"), SandboxFile::new_dir("downloads"));
    }

    /// Display filesystem tree
    pub fn tree(&self) -> String {
        let mut out = String::from("Sandbox FS (");
        let (files, max_f, bytes, max_b) = self.usage();
        out.push_str(&format!("{}/{} files, {}/{} bytes)\n", files, max_f, bytes, max_b));

        let mut sorted: Vec<_> = self.files.keys().collect();
        sorted.sort();
        for path in sorted {
            let file = &self.files[path];
            let depth = path.matches('/').count();
            let indent = "  ".repeat(depth);
            match file.file_type {
                SandboxFileType::Directory => {
                    out.push_str(&format!("{}{}/\n", indent, file.name));
                }
                SandboxFileType::File => {
                    out.push_str(&format!("{}{} ({} bytes, origin: {})\n",
                        indent, file.name, file.data.len(), file.origin));
                }
            }
        }
        out
    }
}
