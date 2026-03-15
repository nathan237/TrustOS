




use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::collections::BTreeMap;
use spin::Mutex;


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FileCategory {
    Text,
    Image,
    Rj,
    Zj,
    Vi,
    Ahv,
    Are,
    F,
}


#[derive(Clone, Debug, PartialEq)]
pub enum Program {
    Ag,
    Bp,
    Rk,
    VideoPlayer,
    Ay,
    Ak,
    Is,
    None,
}

impl Program {
    pub fn j(&self) -> &'static str {
        match self {
            Program::Ag => "Text Editor",
            Program::Bp => "Image Viewer",
            Program::Rk => "Audio Player",
            Program::VideoPlayer => "Video Player",
            Program::Ay => "Terminal",
            Program::Ak => "File Manager",
            Program::Is => "Hex Viewer",
            Program::None => "(None)",
        }
    }
    
    pub fn pa(&self) -> &'static str {
        match self {
            Program::Ag => "Aa",
            Program::Bp => "[]",
            Program::Rk => "d)",
            Program::VideoPlayer => ">|",
            Program::Ay => ">_",
            Program::Ak => "//",
            Program::Is => "0x",
            Program::None => "??",
        }
    }
}


#[derive(Clone, Debug)]
pub struct Abs {
    pub fie: String,
    pub gb: FileCategory,
    pub alo: Program,
    pub dc: String,
}


pub struct FileRegistry {
    gak: BTreeMap<String, Abs>,
}

impl FileRegistry {
    pub const fn new() -> Self {
        FileRegistry {
            gak: BTreeMap::new(),
        }
    }
    
    
    pub fn tth(&mut self) {
        
        self.nw("txt", FileCategory::Text, Program::Ag, "Text Document");
        self.nw("log", FileCategory::Text, Program::Ag, "Log File");
        self.nw("md", FileCategory::Text, Program::Ag, "Markdown");
        self.nw("rs", FileCategory::Text, Program::Ag, "Rust Source");
        self.nw("c", FileCategory::Text, Program::Ag, "C Source");
        self.nw("h", FileCategory::Text, Program::Ag, "C Header");
        self.nw("cpp", FileCategory::Text, Program::Ag, "C++ Source");
        self.nw("py", FileCategory::Text, Program::Ag, "Python Script");
        self.nw("js", FileCategory::Text, Program::Ag, "JavaScript");
        self.nw("json", FileCategory::Text, Program::Ag, "JSON Data");
        self.nw("xml", FileCategory::Text, Program::Ag, "XML Document");
        self.nw("html", FileCategory::Text, Program::Ag, "HTML Page");
        self.nw("css", FileCategory::Text, Program::Ag, "CSS Stylesheet");
        self.nw("toml", FileCategory::Text, Program::Ag, "TOML Config");
        self.nw("cfg", FileCategory::Text, Program::Ag, "Config File");
        self.nw("ini", FileCategory::Text, Program::Ag, "INI Config");
        
        
        self.nw("png", FileCategory::Image, Program::Bp, "PNG Image");
        self.nw("jpg", FileCategory::Image, Program::Bp, "JPEG Image");
        self.nw("jpeg", FileCategory::Image, Program::Bp, "JPEG Image");
        self.nw("gif", FileCategory::Image, Program::Bp, "GIF Image");
        self.nw("bmp", FileCategory::Image, Program::Bp, "Bitmap Image");
        self.nw("ico", FileCategory::Image, Program::Bp, "Icon File");
        self.nw("svg", FileCategory::Image, Program::Bp, "SVG Vector");
        
        
        self.nw("mp3", FileCategory::Rj, Program::Rk, "MP3 Audio");
        self.nw("wav", FileCategory::Rj, Program::Rk, "WAV Audio");
        self.nw("ogg", FileCategory::Rj, Program::Rk, "OGG Audio");
        self.nw("flac", FileCategory::Rj, Program::Rk, "FLAC Audio");
        
        
        self.nw("mp4", FileCategory::Zj, Program::VideoPlayer, "MP4 Video");
        self.nw("avi", FileCategory::Zj, Program::VideoPlayer, "AVI Video");
        self.nw("mkv", FileCategory::Zj, Program::VideoPlayer, "MKV Video");
        self.nw("webm", FileCategory::Zj, Program::VideoPlayer, "WebM Video");
        
        
        self.nw("zip", FileCategory::Vi, Program::Ak, "ZIP Archive");
        self.nw("tar", FileCategory::Vi, Program::Ak, "TAR Archive");
        self.nw("gz", FileCategory::Vi, Program::Ak, "GZip Archive");
        self.nw("7z", FileCategory::Vi, Program::Ak, "7-Zip Archive");
        self.nw("rar", FileCategory::Vi, Program::Ak, "RAR Archive");
        
        
        self.nw("elf", FileCategory::Ahv, Program::Ay, "ELF Executable");
        self.nw("bin", FileCategory::Ahv, Program::Is, "Binary File");
        self.nw("exe", FileCategory::Ahv, Program::Ay, "Executable");
        
        
        self.nw("pdf", FileCategory::Are, Program::Ag, "PDF Document");
        self.nw("doc", FileCategory::Are, Program::Ag, "Word Document");
    }
    
    
    pub fn nw(&mut self, wm: &str, gb: FileCategory, alo: Program, desc: &str) {
        let nsc = wm.aqn();
        self.gak.insert(nsc.clone(), Abs {
            fie: nsc,
            gb,
            alo,
            dc: String::from(desc),
        });
    }
    
    
    pub fn get(&self, wm: &str) -> Option<&Abs> {
        self.gak.get(&wm.aqn())
    }
    
    
    pub fn tem(&self, wm: &str) -> Program {
        self.get(wm)
            .map(|q| q.alo.clone())
            .unwrap_or(Program::None)
    }
    
    
    pub fn nxu(&self, wm: &str) -> FileCategory {
        self.get(wm)
            .map(|q| q.gb)
            .unwrap_or(FileCategory::F)
    }
    
    
    pub fn jpe(&mut self, wm: &str, alo: Program) {
        if let Some(qkt) = self.gak.ds(&wm.aqn()) {
            qkt.alo = alo;
        }
    }
    
    
    pub fn ufm(&self) -> Vec<&Abs> {
        self.gak.alv().collect()
    }
    
    
    pub fn zav(&self, gb: FileCategory) -> Vec<&Abs> {
        self.gak.alv()
            .hi(|q| q.gb == gb)
            .collect()
    }
    
    
    pub fn kyn(it: &str) -> Option<&str> {
        it.cmm('.').next()
    }
    
    
    pub fn iwq(&self, it: &str) -> &'static str {
        if let Some(wm) = Self::kyn(it) {
            match self.nxu(wm) {
                FileCategory::Text => "Aa",
                FileCategory::Image => "<>",
                FileCategory::Rj => "d)",
                FileCategory::Zj => ">|",
                FileCategory::Vi => "{}",
                FileCategory::Ahv => ">>",
                FileCategory::Are => "[]",
                FileCategory::F => "??",
            }
        } else {
            "??"
        }
    }
}


static KA_: Mutex<FileRegistry> = Mutex::new(FileRegistry::new());


pub fn init() {
    KA_.lock().tth();
    crate::serial_println!("[FILE_ASSOC] File associations initialized");
}


pub fn gih(it: &str) -> Program {
    let chc = KA_.lock();
    if let Some(wm) = FileRegistry::kyn(it) {
        chc.tem(wm)
    } else {
        Program::None
    }
}


pub fn yte(it: &str) -> FileCategory {
    let chc = KA_.lock();
    if let Some(wm) = FileRegistry::kyn(it) {
        chc.nxu(wm)
    } else {
        FileCategory::F
    }
}


pub fn iwq(it: &str) -> &'static str {
    let chc = KA_.lock();
    chc.iwq(it)
}


pub fn jpe(wm: &str, alo: Program) {
    KA_.lock().jpe(wm, alo);
}


pub fn ojn() -> Vec<(String, String, String)> {
    let chc = KA_.lock();
    chc.ufm()
        .iter()
        .map(|q| (q.fie.clone(), q.alo.j().into(), q.dc.clone()))
        .collect()
}


pub fn zaz() -> Vec<(Program, &'static str)> {
    vec![
        (Program::Ag, "Text Editor"),
        (Program::Bp, "Image Viewer"),
        (Program::Rk, "Audio Player"),
        (Program::VideoPlayer, "Video Player"),
        (Program::Ay, "Terminal"),
        (Program::Ak, "File Manager"),
        (Program::Is, "Hex Viewer"),
        (Program::None, "(None)"),
    ]
}
