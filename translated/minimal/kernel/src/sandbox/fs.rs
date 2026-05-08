






extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

use super::Ag;




#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SandboxFileType {
    
    File,
    
    Directory,
}


#[derive(Debug, Clone)]
pub struct SandboxFile {
    pub name: String,
    pub file_type: SandboxFileType,
    pub data: Vec<u8>,
    pub created_at: u64,
    pub modified_at: u64,
    
    pub origin: String,
    
    pub readonly: bool,
}

impl SandboxFile {
    fn dbp(name: &str, origin: &str) -> Self {
        let cy = crate::time::uptime_ms();
        Self {
            name: String::from(name),
            file_type: SandboxFileType::File,
            data: Vec::new(),
            created_at: cy,
            modified_at: cy,
            origin: String::from(origin),
            readonly: false,
        }
    }

    fn auj(name: &str) -> Self {
        let cy = crate::time::uptime_ms();
        Self {
            name: String::from(name),
            file_type: SandboxFileType::Directory,
            data: Vec::new(),
            created_at: cy,
            modified_at: cy,
            origin: String::new(),
            readonly: false,
        }
    }
}


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




pub struct SandboxFs {
    sandbox_id: Ag,
    
    files: BTreeMap<String, SandboxFile>,
    
    max_files: usize,
    max_bytes: usize,
    
    total_bytes: usize,
}

impl SandboxFs {
    pub fn new(sandbox_id: Ag, max_files: usize, max_bytes: usize) -> Self {
        let mut fs = Self {
            sandbox_id,
            files: BTreeMap::new(),
            max_files,
            max_bytes,
            total_bytes: 0,
        };
        
        fs.files.insert(String::from("/"), SandboxFile::auj("/"));
        fs.files.insert(String::from("/cache"), SandboxFile::auj("cache"));
        fs.files.insert(String::from("/cookies"), SandboxFile::auj("cookies"));
        fs.files.insert(String::from("/storage"), SandboxFile::auj("storage"));
        fs.files.insert(String::from("/downloads"), SandboxFile::auj("downloads"));
        fs
    }

    
    fn normalize_path(&self, path: &str) -> Result<String, FsError> {
        
        let blc = if path.starts_with('/') {
            String::from(path)
        } else {
            format!("/{}", path)
        };

        
        if blc.contains("..") || blc.contains("./") || blc.contains("//") {
            return Err(FsError::PathViolation);
        }

        
        if blc.bytes().any(|b| b == 0 || b < 0x20) {
            return Err(FsError::InvalidPath);
        }

        
        if blc.len() > 256 {
            return Err(FsError::InvalidPath);
        }

        Ok(blc)
    }

    
    pub fn write(&mut self, path: &str, data: &[u8], origin: &str) -> Result<(), FsError> {
        let path = self.normalize_path(path)?;

        
        if let Some(ku) = self.files.get(&path) {
            if ku.readonly {
                return Err(FsError::ReadOnly);
            }
            if ku.file_type == SandboxFileType::Directory {
                return Err(FsError::IsDirectory);
            }
            
            let evr = ku.data.len();
            let iqh = self.total_bytes - evr + data.len();
            if iqh > self.max_bytes {
                return Err(FsError::QuotaBytes);
            }
            self.total_bytes = iqh;
        } else {
            
            if self.files.len() >= self.max_files {
                return Err(FsError::QuotaFiles);
            }
            if self.total_bytes + data.len() > self.max_bytes {
                return Err(FsError::QuotaBytes);
            }
            self.total_bytes += data.len();
        }

        let mut file = SandboxFile::dbp(
            path.rsplit('/').next().unwrap_or(&path),
            origin,
        );
        file.data = data.to_vec();
        self.files.insert(path, file);
        Ok(())
    }

    
    pub fn read(&self, path: &str) -> Result<&[u8], FsError> {
        let path = self.normalize_path(path)?;
        match self.files.get(&path) {
            Some(f) if f.file_type == SandboxFileType::File => Ok(&f.data),
            Some(_) => Err(FsError::IsDirectory),
            None => Err(FsError::NotFound),
        }
    }

    
    pub fn delete(&mut self, path: &str) -> Result<(), FsError> {
        let path = self.normalize_path(path)?;
        match self.files.get(&path) {
            Some(f) => {
                if f.readonly {
                    return Err(FsError::ReadOnly);
                }
                if f.file_type == SandboxFileType::Directory {
                    
                    let nm = format!("{}/", path);
                    let mjf = self.files.keys().any(|k| k.starts_with(&nm));
                    if mjf {
                        return Err(FsError::NotDirectory); 
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

    
    pub fn list(&self, dnd: &str) -> Result<Vec<(&str, &SandboxFileType, usize)>, FsError> {
        let it = self.normalize_path(dnd)?;
        let nm = if it == "/" { String::from("/") } else { format!("{}/", it) };

        let mut entries = Vec::new();
        for (path, file) in &self.files {
            if path == &it { continue; } 
            if path.starts_with(&nm) {
                
                let ef = &path[nm.len()..];
                if !ef.contains('/') {
                    entries.push((path.as_str(), &file.file_type, file.data.len()));
                }
            }
        }
        Ok(entries)
    }

    
    pub fn exists(&self, path: &str) -> bool {
        if let Ok(aa) = self.normalize_path(path) {
            self.files.contains_key(&aa)
        } else {
            false
        }
    }

    
    pub fn mkdir(&mut self, path: &str) -> Result<(), FsError> {
        let path = self.normalize_path(path)?;
        if self.files.contains_key(&path) {
            return Err(FsError::AlreadyExists);
        }
        if self.files.len() >= self.max_files {
            return Err(FsError::QuotaFiles);
        }
        self.files.insert(path.clone(), SandboxFile::auj(
            path.rsplit('/').next().unwrap_or(&path)
        ));
        Ok(())
    }

    

    
    pub fn qvo(&mut self, domain: &str, name: &str, value: &str) -> Result<(), FsError> {
        let path = format!("/cookies/{}_{}", domain.replace('.', "_"), name);
        self.write(&path, value.as_bytes(), domain)
    }

    
    pub fn qhk(&self, domain: &str, name: &str) -> Option<String> {
        let path = format!("/cookies/{}_{}", domain.replace('.', "_"), name);
        self.read(&path).ok()
            .map(|data| String::from_utf8_lossy(data).into_owned())
    }

    
    pub fn qod(&mut self, domain: &str, key: &str, value: &str) -> Result<(), FsError> {
        let it = format!("/storage/{}", domain.replace('.', "_"));
        if !self.exists(&it) {
            self.mkdir(&it)?;
        }
        let path = format!("{}/{}", it, key);
        self.write(&path, value.as_bytes(), domain)
    }

    
    pub fn qoc(&self, domain: &str, key: &str) -> Option<String> {
        let path = format!("/storage/{}/{}", domain.replace('.', "_"), key);
        self.read(&path).ok()
            .map(|data| String::from_utf8_lossy(data).into_owned())
    }

    
    pub fn pza(&mut self, url: &str, data: &[u8]) -> Result<(), FsError> {
        
        let hash = url.bytes().fold(0u64, |aku, b| aku.wrapping_mul(31).wrapping_add(b as u64));
        let path = format!("/cache/{:016x}", hash);
        self.write(&path, data, "cache")
    }

    
    pub fn qhg(&self, url: &str) -> Option<&[u8]> {
        let hash = url.bytes().fold(0u64, |aku, b| aku.wrapping_mul(31).wrapping_add(b as u64));
        let path = format!("/cache/{:016x}", hash);
        self.read(&path).ok()
    }

    

    
    pub fn usage(&self) -> (usize, usize, usize, usize) {
        let adp = self.files.values()
            .filter(|f| f.file_type == SandboxFileType::File)
            .count();
        (adp, self.max_files, self.total_bytes, self.max_bytes)
    }

    
    pub fn clear(&mut self) {
        self.files.clear();
        self.total_bytes = 0;
        
        self.files.insert(String::from("/"), SandboxFile::auj("/"));
        self.files.insert(String::from("/cache"), SandboxFile::auj("cache"));
        self.files.insert(String::from("/cookies"), SandboxFile::auj("cookies"));
        self.files.insert(String::from("/storage"), SandboxFile::auj("storage"));
        self.files.insert(String::from("/downloads"), SandboxFile::auj("downloads"));
    }

    
    pub fn tree(&self) -> String {
        let mut out = String::from("Sandbox FS (");
        let (files, max_f, bytes, max_b) = self.usage();
        out.push_str(&format!("{}/{} files, {}/{} bytes)\n", files, max_f, bytes, max_b));

        let mut acq: Vec<_> = self.files.keys().collect();
        acq.sort();
        for path in acq {
            let file = &self.files[path];
            let depth = path.matches('/').count();
            let axq = "  ".repeat(depth);
            match file.file_type {
                SandboxFileType::Directory => {
                    out.push_str(&format!("{}{}/\n", axq, file.name));
                }
                SandboxFileType::File => {
                    out.push_str(&format!("{}{} ({} bytes, origin: {})\n",
                        axq, file.name, file.data.len(), file.origin));
                }
            }
        }
        out
    }
}
