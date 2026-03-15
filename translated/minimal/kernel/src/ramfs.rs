



use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::format;
use spin::Mutex;


pub const AZP_: usize = 32 * 1024 * 1024;


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FileType {
    Es,
    K,
}


#[derive(Clone)]
pub struct FsEntry {
    pub j: String,
    pub kd: FileType,
    pub ca: Vec<u8>,
    pub zf: Vec<String>, 
    pub cju: u64,       
    pub euw: u64,      
}

impl FsEntry {
    pub fn gnm(j: &str) -> Self {
        let qb = crate::logger::lh();
        Self {
            j: String::from(j),
            kd: FileType::Es,
            ca: Vec::new(),
            zf: Vec::new(),
            cju: qb,
            euw: qb,
        }
    }
    
    pub fn cll(j: &str) -> Self {
        let qb = crate::logger::lh();
        Self {
            j: String::from(j),
            kd: FileType::K,
            ca: Vec::new(),
            zf: Vec::new(),
            cju: qb,
            euw: qb,
        }
    }
}


pub struct RamFs {
    ch: BTreeMap<String, FsEntry>,
    eae: String,
}

impl RamFs {
    pub const fn new() -> Self {
        Self {
            ch: BTreeMap::new(),
            eae: String::new(),
        }
    }
    
    pub fn init(&mut self) {
        
        self.eae.clear();
        self.ch.insert(String::from("/"), FsEntry::cll("/"));
        
        
        self.ut("/home").bq();
        self.ut("/tmp").bq();
        self.ut("/bin").bq();
        self.ut("/etc").bq();
        self.ut("/documents").bq();
        self.ut("/downloads").bq();
        self.ut("/music").bq();
        self.ut("/pictures").bq();
        
        
        self.touch("/home/welcome.txt").bq();
        self.ns("/home/welcome.txt", b"Welcome to TrustOS!\n\nThis is a RAM filesystem.\nAll files are stored in memory.\n").bq();
        
        
        self.touch("/etc/hostname").bq();
        self.ns("/etc/hostname", b"trustos").bq();
        
        
        self.touch("/etc/version").bq();
        self.ns("/etc/version", b"TrustOS v0.9.4\n").bq();
        
        
        self.touch("/documents/readme.md").bq();
        self.ns("/documents/readme.md", b"# My Documents\n\nWelcome to TrustOS! Place your files here.\n").bq();
        self.touch("/documents/notes.txt").bq();
        self.ns("/documents/notes.txt", b"My notes\n--------\n").bq();
        
        self.touch("/downloads/example.txt").bq();
        self.ns("/downloads/example.txt", b"Downloaded files will appear here.\n").bq();
        
        self.touch("/music/playlist.toml").bq();
        self.ns("/music/playlist.toml", b"[playlist]\nname = \"My Music\"\ntracks = []\n").bq();
        
        self.touch("/pictures/info.txt").bq();
        self.ns("/pictures/info.txt", b"Screenshots and images will be saved here.\n").bq();
    }
    
    
    pub fn dau(&self) -> &str {
        if self.eae.is_empty() {
            "/"
        } else {
            &self.eae
        }
    }
    
    
    pub fn fem(&mut self, path: &str) -> Result<(), FsError> {
        let aqs = self.aqj(path);
        
        if let Some(bt) = self.ch.get(&aqs) {
            if bt.kd == FileType::K {
                self.eae = aqs;
                Ok(())
            } else {
                Err(FsError::Adm)
            }
        } else {
            Err(FsError::N)
        }
    }
    
    
    pub fn awb(&self, path: Option<&str>) -> Result<Vec<(String, FileType, usize)>, FsError> {
        let hge = match path {
            Some(ai) => self.aqj(ai),
            None => {
                if self.eae.is_empty() {
                    String::from("/")
                } else {
                    self.eae.clone()
                }
            }
        };
        
        if let Some(bt) = self.ch.get(&hge) {
            if bt.kd != FileType::K {
                return Err(FsError::Adm);
            }
            
            let mut pj: Vec<(String, FileType, usize)> = Vec::new();
            for khm in &bt.zf {
                let enk = if hge == "/" {
                    format!("/{}", khm)
                } else {
                    format!("{}/{}", hge, khm)
                };
                
                if let Some(aeh) = self.ch.get(&enk) {
                    let aw = if aeh.kd == FileType::Es {
                        aeh.ca.len()
                    } else {
                        aeh.zf.len()
                    };
                    pj.push((khm.clone(), aeh.kd, aw));
                }
            }
            Ok(pj)
        } else {
            Err(FsError::N)
        }
    }
    
    
    pub fn ut(&mut self, path: &str) -> Result<(), FsError> {
        let aqs = self.aqj(path);
        
        if self.ch.bgm(&aqs) {
            return Err(FsError::Ri);
        }
        
        
        let bhs = self.bhs(&aqs);
        let j = self.fdf(&aqs);
        
        
        if let Some(tu) = self.ch.ds(&bhs) {
            if tu.kd != FileType::K {
                return Err(FsError::Adm);
            }
            tu.zf.push(j.clone());
        } else {
            return Err(FsError::N);
        }
        
        
        self.ch.insert(aqs, FsEntry::cll(&j));
        Ok(())
    }
    
    
    pub fn touch(&mut self, path: &str) -> Result<(), FsError> {
        let aqs = self.aqj(path);
        
        if self.ch.bgm(&aqs) {
            
            if let Some(bt) = self.ch.ds(&aqs) {
                bt.euw = crate::logger::lh();
            }
            return Ok(());
        }
        
        
        let bhs = self.bhs(&aqs);
        let j = self.fdf(&aqs);
        
        
        if let Some(tu) = self.ch.ds(&bhs) {
            if tu.kd != FileType::K {
                return Err(FsError::Adm);
            }
            tu.zf.push(j.clone());
        } else {
            return Err(FsError::N);
        }
        
        
        self.ch.insert(aqs, FsEntry::gnm(&j));
        Ok(())
    }
    
    
    pub fn hb(&mut self, path: &str) -> Result<(), FsError> {
        let aqs = self.aqj(path);
        
        if aqs == "/" {
            return Err(FsError::Jt);
        }
        
        
        if let Some(bt) = self.ch.get(&aqs) {
            
            if bt.kd == FileType::K && !bt.zf.is_empty() {
                return Err(FsError::Bep);
            }
        } else {
            return Err(FsError::N);
        }
        
        
        let bhs = self.bhs(&aqs);
        let j = self.fdf(&aqs);
        
        if let Some(tu) = self.ch.ds(&bhs) {
            tu.zf.ajm(|r| r != &j);
        }
        
        
        self.ch.remove(&aqs);
        Ok(())
    }
    
    
    pub fn mq(&self, path: &str) -> Result<&[u8], FsError> {
        let aqs = self.aqj(path);
        
        if let Some(bt) = self.ch.get(&aqs) {
            if bt.kd == FileType::Es {
                Ok(&bt.ca)
            } else {
                Err(FsError::Aci)
            }
        } else {
            Err(FsError::N)
        }
    }
    
    
    pub fn ns(&mut self, path: &str, ca: &[u8]) -> Result<(), FsError> {
        let aqs = self.aqj(path);
        
        if let Some(bt) = self.ch.ds(&aqs) {
            if bt.kd == FileType::Es {
                if ca.len() > AZP_ {
                    return Err(FsError::Asi);
                }
                bt.ca = ca.ip();
                bt.euw = crate::logger::lh();
                Ok(())
            } else {
                Err(FsError::Aci)
            }
        } else {
            Err(FsError::N)
        }
    }
    
    
    pub fn ijw(&mut self, path: &str, ca: &[u8]) -> Result<(), FsError> {
        let aqs = self.aqj(path);
        
        if let Some(bt) = self.ch.ds(&aqs) {
            if bt.kd == FileType::Es {
                if bt.ca.len() + ca.len() > AZP_ {
                    return Err(FsError::Asi);
                }
                bt.ca.bk(ca);
                bt.euw = crate::logger::lh();
                Ok(())
            } else {
                Err(FsError::Aci)
            }
        } else {
            Err(FsError::N)
        }
    }
    
    
    pub fn bza(&mut self, cy: &str, cs: &str) -> Result<(), FsError> {
        let ibl = self.aqj(cy);
        let hhc = self.aqj(cs);
        
        
        let ca = {
            if let Some(bt) = self.ch.get(&ibl) {
                if bt.kd != FileType::Es {
                    return Err(FsError::Aci);
                }
                bt.ca.clone()
            } else {
                return Err(FsError::N);
            }
        };
        
        
        if !self.ch.bgm(&hhc) {
            self.touch(cs)?;
        }
        
        
        self.ns(cs, &ca)
    }
    
    
    pub fn euz(&mut self, cy: &str, cs: &str) -> Result<(), FsError> {
        self.bza(cy, cs)?;
        self.hb(cy)?;
        Ok(())
    }
    
    
    pub fn aja(&self, path: &str) -> bool {
        let aqs = self.aqj(path);
        self.ch.bgm(&aqs)
    }
    
    
    pub fn hm(&self, path: &str) -> Result<&FsEntry, FsError> {
        let aqs = self.aqj(path);
        self.ch.get(&aqs).ok_or(FsError::N)
    }
    
    
    fn aqj(&self, path: &str) -> String {
        if path.cj('/') {
            self.bro(path)
        } else if path == "~" || path.cj("~/") {
            let kr = if path == "~" { "" } else { &path[2..] };
            if kr.is_empty() {
                String::from("/home")
            } else {
                format!("/home/{}", kr)
            }
        } else {
            let jv = if self.eae.is_empty() { "/" } else { &self.eae };
            if jv == "/" {
                self.bro(&format!("/{}", path))
            } else {
                self.bro(&format!("{}/{}", jv, path))
            }
        }
    }
    
    
    fn bro(&self, path: &str) -> String {
        let mut ek: Vec<&str> = Vec::new();
        
        for vu in path.adk('/') {
            match vu {
                "" | "." => continue,
                ".." => { ek.pop(); }
                _ => ek.push(vu),
            }
        }
        
        if ek.is_empty() {
            String::from("/")
        } else {
            format!("/{}", ek.rr("/"))
        }
    }
    
    
    fn bhs(&self, path: &str) -> String {
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
    
    
    fn fdf(&self, path: &str) -> String {
        if let Some(u) = path.bhx('/') {
            String::from(&path[u + 1..])
        } else {
            String::from(path)
        }
    }
}


#[derive(Debug, Clone, Copy)]
pub enum FsError {
    N,
    Ri,
    Adm,
    Aci,
    Bep,
    Asi,
    Jt,
}

impl FsError {
    pub fn as_str(&self) -> &'static str {
        match self {
            FsError::N => "not found",
            FsError::Ri => "already exists",
            FsError::Adm => "not a directory",
            FsError::Aci => "is a directory",
            FsError::Bep => "directory not empty",
            FsError::Asi => "file too large",
            FsError::Jt => "permission denied",
        }
    }
}


static Arx: Mutex<Option<RamFs>> = Mutex::new(None);


pub fn init() {
    let mut fs = RamFs::new();
    fs.init();
    *Arx.lock() = Some(fs);
}


pub fn ky() -> bool {
    Arx.lock().is_some()
}


pub fn fh<G, Ac>(bb: G) -> Ac
where
    G: FnOnce(&mut RamFs) -> Ac,
{
    let mut adb = Arx.lock();
    bb(adb.as_mut().expect("Filesystem not initialized"))
}
