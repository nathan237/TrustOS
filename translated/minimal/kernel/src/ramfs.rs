



use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::format;
use spin::Mutex;


pub const BBR_: usize = 32 * 1024 * 1024;


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FileType {
    File,
    Directory,
}


#[derive(Clone)]
pub struct FsEntry {
    pub name: String,
    pub file_type: FileType,
    pub content: Vec<u8>,
    pub children: Vec<String>, 
    pub created_at: u64,       
    pub modified_at: u64,      
}

impl FsEntry {
    pub fn dbp(name: &str) -> Self {
        let gx = crate::logger::eg();
        Self {
            name: String::from(name),
            file_type: FileType::File,
            content: Vec::new(),
            children: Vec::new(),
            created_at: gx,
            modified_at: gx,
        }
    }
    
    pub fn auj(name: &str) -> Self {
        let gx = crate::logger::eg();
        Self {
            name: String::from(name),
            file_type: FileType::Directory,
            content: Vec::new(),
            children: Vec::new(),
            created_at: gx,
            modified_at: gx,
        }
    }
}


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
        
        self.current_dir.clear();
        self.entries.insert(String::from("/"), FsEntry::auj("/"));
        
        
        self.mkdir("/home").ok();
        self.mkdir("/tmp").ok();
        self.mkdir("/bin").ok();
        self.mkdir("/etc").ok();
        self.mkdir("/documents").ok();
        self.mkdir("/downloads").ok();
        self.mkdir("/music").ok();
        self.mkdir("/pictures").ok();
        
        
        self.touch("/home/welcome.txt").ok();
        self.write_file("/home/welcome.txt", b"Welcome to TrustOS!\n\nThis is a RAM filesystem.\nAll files are stored in memory.\n").ok();
        
        
        self.touch("/etc/hostname").ok();
        self.write_file("/etc/hostname", b"trustos").ok();
        
        
        self.touch("/etc/version").ok();
        self.write_file("/etc/version", b"TrustOS v0.9.4\n").ok();
        
        
        self.touch("/documents/readme.md").ok();
        self.write_file("/documents/readme.md", b"# My Documents\n\nWelcome to TrustOS! Place your files here.\n").ok();
        self.touch("/documents/notes.txt").ok();
        self.write_file("/documents/notes.txt", b"My notes\n--------\n").ok();
        
        self.touch("/downloads/example.txt").ok();
        self.write_file("/downloads/example.txt", b"Downloaded files will appear here.\n").ok();
        
        self.touch("/music/playlist.toml").ok();
        self.write_file("/music/playlist.toml", b"[playlist]\nname = \"My Music\"\ntracks = []\n").ok();
        
        self.touch("/pictures/info.txt").ok();
        self.write_file("/pictures/info.txt", b"Screenshots and images will be saved here.\n").ok();
    }
    
    
    pub fn pwd(&self) -> &str {
        if self.current_dir.is_empty() {
            "/"
        } else {
            &self.current_dir
        }
    }
    
    
    pub fn cd(&mut self, path: &str) -> Result<(), FsError> {
        let vw = self.resolve_path(path);
        
        if let Some(entry) = self.entries.get(&vw) {
            if entry.file_type == FileType::Directory {
                self.current_dir = vw;
                Ok(())
            } else {
                Err(FsError::NotADirectory)
            }
        } else {
            Err(FsError::NotFound)
        }
    }
    
    
    pub fn ls(&self, path: Option<&str>) -> Result<Vec<(String, FileType, usize)>, FsError> {
        let dnd = match path {
            Some(aa) => self.resolve_path(aa),
            None => {
                if self.current_dir.is_empty() {
                    String::from("/")
                } else {
                    self.current_dir.clone()
                }
            }
        };
        
        if let Some(entry) = self.entries.get(&dnd) {
            if entry.file_type != FileType::Directory {
                return Err(FsError::NotADirectory);
            }
            
            let mut items: Vec<(String, FileType, usize)> = Vec::new();
            for child_name in &entry.children {
                let bxx = if dnd == "/" {
                    format!("/{}", child_name)
                } else {
                    format!("{}/{}", dnd, child_name)
                };
                
                if let Some(pd) = self.entries.get(&bxx) {
                    let size = if pd.file_type == FileType::File {
                        pd.content.len()
                    } else {
                        pd.children.len()
                    };
                    items.push((child_name.clone(), pd.file_type, size));
                }
            }
            Ok(items)
        } else {
            Err(FsError::NotFound)
        }
    }
    
    
    pub fn mkdir(&mut self, path: &str) -> Result<(), FsError> {
        let vw = self.resolve_path(path);
        
        if self.entries.contains_key(&vw) {
            return Err(FsError::AlreadyExists);
        }
        
        
        let parent_path = self.parent_path(&vw);
        let name = self.basename(&vw);
        
        
        if let Some(parent) = self.entries.get_mut(&parent_path) {
            if parent.file_type != FileType::Directory {
                return Err(FsError::NotADirectory);
            }
            parent.children.push(name.clone());
        } else {
            return Err(FsError::NotFound);
        }
        
        
        self.entries.insert(vw, FsEntry::auj(&name));
        Ok(())
    }
    
    
    pub fn touch(&mut self, path: &str) -> Result<(), FsError> {
        let vw = self.resolve_path(path);
        
        if self.entries.contains_key(&vw) {
            
            if let Some(entry) = self.entries.get_mut(&vw) {
                entry.modified_at = crate::logger::eg();
            }
            return Ok(());
        }
        
        
        let parent_path = self.parent_path(&vw);
        let name = self.basename(&vw);
        
        
        if let Some(parent) = self.entries.get_mut(&parent_path) {
            if parent.file_type != FileType::Directory {
                return Err(FsError::NotADirectory);
            }
            parent.children.push(name.clone());
        } else {
            return Err(FsError::NotFound);
        }
        
        
        self.entries.insert(vw, FsEntry::dbp(&name));
        Ok(())
    }
    
    
    pub fn rm(&mut self, path: &str) -> Result<(), FsError> {
        let vw = self.resolve_path(path);
        
        if vw == "/" {
            return Err(FsError::PermissionDenied);
        }
        
        
        if let Some(entry) = self.entries.get(&vw) {
            
            if entry.file_type == FileType::Directory && !entry.children.is_empty() {
                return Err(FsError::DirectoryNotEmpty);
            }
        } else {
            return Err(FsError::NotFound);
        }
        
        
        let parent_path = self.parent_path(&vw);
        let name = self.basename(&vw);
        
        if let Some(parent) = self.entries.get_mut(&parent_path) {
            parent.children.retain(|c| c != &name);
        }
        
        
        self.entries.remove(&vw);
        Ok(())
    }
    
    
    pub fn read_file(&self, path: &str) -> Result<&[u8], FsError> {
        let vw = self.resolve_path(path);
        
        if let Some(entry) = self.entries.get(&vw) {
            if entry.file_type == FileType::File {
                Ok(&entry.content)
            } else {
                Err(FsError::IsADirectory)
            }
        } else {
            Err(FsError::NotFound)
        }
    }
    
    
    pub fn write_file(&mut self, path: &str, content: &[u8]) -> Result<(), FsError> {
        let vw = self.resolve_path(path);
        
        if let Some(entry) = self.entries.get_mut(&vw) {
            if entry.file_type == FileType::File {
                if content.len() > BBR_ {
                    return Err(FsError::FileTooLarge);
                }
                entry.content = content.to_vec();
                entry.modified_at = crate::logger::eg();
                Ok(())
            } else {
                Err(FsError::IsADirectory)
            }
        } else {
            Err(FsError::NotFound)
        }
    }
    
    
    pub fn append_file(&mut self, path: &str, content: &[u8]) -> Result<(), FsError> {
        let vw = self.resolve_path(path);
        
        if let Some(entry) = self.entries.get_mut(&vw) {
            if entry.file_type == FileType::File {
                if entry.content.len() + content.len() > BBR_ {
                    return Err(FsError::FileTooLarge);
                }
                entry.content.extend_from_slice(content);
                entry.modified_at = crate::logger::eg();
                Ok(())
            } else {
                Err(FsError::IsADirectory)
            }
        } else {
            Err(FsError::NotFound)
        }
    }
    
    
    pub fn cp(&mut self, src: &str, dst: &str) -> Result<(), FsError> {
        let eag = self.resolve_path(src);
        let dnv = self.resolve_path(dst);
        
        
        let content = {
            if let Some(entry) = self.entries.get(&eag) {
                if entry.file_type != FileType::File {
                    return Err(FsError::IsADirectory);
                }
                entry.content.clone()
            } else {
                return Err(FsError::NotFound);
            }
        };
        
        
        if !self.entries.contains_key(&dnv) {
            self.touch(dst)?;
        }
        
        
        self.write_file(dst, &content)
    }
    
    
    pub fn mv(&mut self, src: &str, dst: &str) -> Result<(), FsError> {
        self.cp(src, dst)?;
        self.rm(src)?;
        Ok(())
    }
    
    
    pub fn exists(&self, path: &str) -> bool {
        let vw = self.resolve_path(path);
        self.entries.contains_key(&vw)
    }
    
    
    pub fn stat(&self, path: &str) -> Result<&FsEntry, FsError> {
        let vw = self.resolve_path(path);
        self.entries.get(&vw).ok_or(FsError::NotFound)
    }
    
    
    fn resolve_path(&self, path: &str) -> String {
        if path.starts_with('/') {
            self.normalize_path(path)
        } else if path == "~" || path.starts_with("~/") {
            let ef = if path == "~" { "" } else { &path[2..] };
            if ef.is_empty() {
                String::from("/home")
            } else {
                format!("/home/{}", ef)
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
    
    
    fn normalize_path(&self, path: &str) -> String {
        let mut au: Vec<&str> = Vec::new();
        
        for jn in path.split('/') {
            match jn {
                "" | "." => continue,
                ".." => { au.pop(); }
                _ => au.push(jn),
            }
        }
        
        if au.is_empty() {
            String::from("/")
        } else {
            format!("/{}", au.join("/"))
        }
    }
    
    
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
    
    
    fn basename(&self, path: &str) -> String {
        if let Some(pos) = path.rfind('/') {
            String::from(&path[pos + 1..])
        } else {
            String::from(path)
        }
    }
}


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


static Sc: Mutex<Option<RamFs>> = Mutex::new(None);


pub fn init() {
    let mut fs = RamFs::new();
    fs.init();
    *Sc.lock() = Some(fs);
}


pub fn is_initialized() -> bool {
    Sc.lock().is_some()
}


pub fn bh<F, U>(f: F) -> U
where
    F: FnOnce(&mut RamFs) -> U,
{
    let mut jg = Sc.lock();
    f(jg.as_mut().expect("Filesystem not initialized"))
}
