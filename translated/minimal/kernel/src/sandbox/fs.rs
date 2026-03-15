






extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

use super::Ax;




#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SandboxFileType {
    
    Es,
    
    K,
}


#[derive(Debug, Clone)]
pub struct SandboxFile {
    pub j: String,
    pub kd: SandboxFileType,
    pub f: Vec<u8>,
    pub cju: u64,
    pub euw: u64,
    
    pub atf: String,
    
    pub awr: bool,
}

impl SandboxFile {
    fn gnm(j: &str, atf: &str) -> Self {
        let iu = crate::time::lc();
        Self {
            j: String::from(j),
            kd: SandboxFileType::Es,
            f: Vec::new(),
            cju: iu,
            euw: iu,
            atf: String::from(atf),
            awr: false,
        }
    }

    fn cll(j: &str) -> Self {
        let iu = crate::time::lc();
        Self {
            j: String::from(j),
            kd: SandboxFileType::K,
            f: Vec::new(),
            cju: iu,
            euw: iu,
            atf: String::new(),
            awr: false,
        }
    }
}


#[derive(Debug)]
pub enum FsError {
    N,
    Ri,
    Bpu,
    Bpt,
    Ciz,
    Bz,
    Pr,
    Tc,
    Lz,
}




pub struct SandboxFs {
    afh: Ax,
    
    sb: BTreeMap<String, SandboxFile>,
    
    hqz: usize,
    fnt: usize,
    
    xv: usize,
}

impl SandboxFs {
    pub fn new(afh: Ax, hqz: usize, fnt: usize) -> Self {
        let mut fs = Self {
            afh,
            sb: BTreeMap::new(),
            hqz,
            fnt,
            xv: 0,
        };
        
        fs.sb.insert(String::from("/"), SandboxFile::cll("/"));
        fs.sb.insert(String::from("/cache"), SandboxFile::cll("cache"));
        fs.sb.insert(String::from("/cookies"), SandboxFile::cll("cookies"));
        fs.sb.insert(String::from("/storage"), SandboxFile::cll("storage"));
        fs.sb.insert(String::from("/downloads"), SandboxFile::cll("downloads"));
        fs
    }

    
    fn bro(&self, path: &str) -> Result<String, FsError> {
        
        let dox = if path.cj('/') {
            String::from(path)
        } else {
            format!("/{}", path)
        };

        
        if dox.contains("..") || dox.contains("./") || dox.contains("//") {
            return Err(FsError::Ciz);
        }

        
        if dox.bf().any(|o| o == 0 || o < 0x20) {
            return Err(FsError::Pr);
        }

        
        if dox.len() > 256 {
            return Err(FsError::Pr);
        }

        Ok(dox)
    }

    
    pub fn write(&mut self, path: &str, f: &[u8], atf: &str) -> Result<(), FsError> {
        let path = self.bro(path)?;

        
        if let Some(xy) = self.sb.get(&path) {
            if xy.awr {
                return Err(FsError::Bz);
            }
            if xy.kd == SandboxFileType::K {
                return Err(FsError::Tc);
            }
            
            let jhs = xy.f.len();
            let oqe = self.xv - jhs + f.len();
            if oqe > self.fnt {
                return Err(FsError::Bpt);
            }
            self.xv = oqe;
        } else {
            
            if self.sb.len() >= self.hqz {
                return Err(FsError::Bpu);
            }
            if self.xv + f.len() > self.fnt {
                return Err(FsError::Bpt);
            }
            self.xv += f.len();
        }

        let mut file = SandboxFile::gnm(
            path.cmm('/').next().unwrap_or(&path),
            atf,
        );
        file.f = f.ip();
        self.sb.insert(path, file);
        Ok(())
    }

    
    pub fn read(&self, path: &str) -> Result<&[u8], FsError> {
        let path = self.bro(path)?;
        match self.sb.get(&path) {
            Some(bb) if bb.kd == SandboxFileType::Es => Ok(&bb.f),
            Some(_) => Err(FsError::Tc),
            None => Err(FsError::N),
        }
    }

    
    pub fn rvg(&mut self, path: &str) -> Result<(), FsError> {
        let path = self.bro(path)?;
        match self.sb.get(&path) {
            Some(bb) => {
                if bb.awr {
                    return Err(FsError::Bz);
                }
                if bb.kd == SandboxFileType::K {
                    
                    let adx = format!("{}/", path);
                    let tmf = self.sb.cai().any(|eh| eh.cj(&adx));
                    if tmf {
                        return Err(FsError::Lz); 
                    }
                }
                let aw = bb.f.len();
                self.sb.remove(&path);
                self.xv -= aw;
                Ok(())
            }
            None => Err(FsError::N),
        }
    }

    
    pub fn aoy(&self, hge: &str) -> Result<Vec<(&str, &SandboxFileType, usize)>, FsError> {
        let te = self.bro(hge)?;
        let adx = if te == "/" { String::from("/") } else { format!("{}/", te) };

        let mut ch = Vec::new();
        for (path, file) in &self.sb {
            if path == &te { continue; } 
            if path.cj(&adx) {
                
                let kr = &path[adx.len()..];
                if !kr.contains('/') {
                    ch.push((path.as_str(), &file.kd, file.f.len()));
                }
            }
        }
        Ok(ch)
    }

    
    pub fn aja(&self, path: &str) -> bool {
        if let Ok(ai) = self.bro(path) {
            self.sb.bgm(&ai)
        } else {
            false
        }
    }

    
    pub fn ut(&mut self, path: &str) -> Result<(), FsError> {
        let path = self.bro(path)?;
        if self.sb.bgm(&path) {
            return Err(FsError::Ri);
        }
        if self.sb.len() >= self.hqz {
            return Err(FsError::Bpu);
        }
        self.sb.insert(path.clone(), SandboxFile::cll(
            path.cmm('/').next().unwrap_or(&path)
        ));
        Ok(())
    }

    

    
    pub fn zmr(&mut self, vh: &str, j: &str, bn: &str) -> Result<(), FsError> {
        let path = format!("/cookies/{}_{}", vh.replace('.', "_"), j);
        self.write(&path, bn.as_bytes(), vh)
    }

    
    pub fn ysv(&self, vh: &str, j: &str) -> Option<String> {
        let path = format!("/cookies/{}_{}", vh.replace('.', "_"), j);
        self.read(&path).bq()
            .map(|f| String::azw(f).bkc())
    }

    
    pub fn zbp(&mut self, vh: &str, bs: &str, bn: &str) -> Result<(), FsError> {
        let te = format!("/storage/{}", vh.replace('.', "_"));
        if !self.aja(&te) {
            self.ut(&te)?;
        }
        let path = format!("{}/{}", te, bs);
        self.write(&path, bn.as_bytes(), vh)
    }

    
    pub fn zbo(&self, vh: &str, bs: &str) -> Option<String> {
        let path = format!("/storage/{}/{}", vh.replace('.', "_"), bs);
        self.read(&path).bq()
            .map(|f| String::azw(f).bkc())
    }

    
    pub fn yhd(&mut self, url: &str, f: &[u8]) -> Result<(), FsError> {
        
        let hash = url.bf().cqs(0u64, |btc, o| btc.hx(31).cn(o as u64));
        let path = format!("/cache/{:016x}", hash);
        self.write(&path, f, "cache")
    }

    
    pub fn ysr(&self, url: &str) -> Option<&[u8]> {
        let hash = url.bf().cqs(0u64, |btc, o| btc.hx(31).cn(o as u64));
        let path = format!("/cache/{:016x}", hash);
        self.read(&path).bq()
    }

    

    
    pub fn pxo(&self) -> (usize, usize, usize, usize) {
        let bec = self.sb.alv()
            .hi(|bb| bb.kd == SandboxFileType::Es)
            .az();
        (bec, self.hqz, self.xv, self.fnt)
    }

    
    pub fn clear(&mut self) {
        self.sb.clear();
        self.xv = 0;
        
        self.sb.insert(String::from("/"), SandboxFile::cll("/"));
        self.sb.insert(String::from("/cache"), SandboxFile::cll("cache"));
        self.sb.insert(String::from("/cookies"), SandboxFile::cll("cookies"));
        self.sb.insert(String::from("/storage"), SandboxFile::cll("storage"));
        self.sb.insert(String::from("/downloads"), SandboxFile::cll("downloads"));
    }

    
    pub fn iex(&self) -> String {
        let mut bd = String::from("Sandbox FS (");
        let (sb, ule, bf, ukt) = self.pxo();
        bd.t(&format!("{}/{} files, {}/{} bytes)\n", sb, ule, bf, ukt));

        let mut bcs: Vec<_> = self.sb.cai().collect();
        bcs.jqs();
        for path in bcs {
            let file = &self.sb[path];
            let eo = path.oh('/').az();
            let crn = "  ".afd(eo);
            match file.kd {
                SandboxFileType::K => {
                    bd.t(&format!("{}{}/\n", crn, file.j));
                }
                SandboxFileType::Es => {
                    bd.t(&format!("{}{} ({} bytes, origin: {})\n",
                        crn, file.j, file.f.len(), file.atf));
                }
            }
        }
        bd
    }
}
