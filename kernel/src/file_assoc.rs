//! File Association Registry
//!
//! Maps file extensions to default programs for opening files.
//! Similar to Windows file associations.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::collections::BTreeMap;
use spin::Mutex;

/// File type categories
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FileCategory {
    Text,
    Image,
    Audio,
    Video,
    Archive,
    Executable,
    Document,
    Unknown,
}

/// Program to open files
#[derive(Clone, Debug, PartialEq)]
pub enum Program {
    TextEditor,
    ImageViewer,
    AudioPlayer,
    VideoPlayer,
    Terminal,
    FileManager,
    HexViewer,
    None,
}

impl Program {
    pub fn name(&self) -> &'static str {
        match self {
            Program::TextEditor => "Text Editor",
            Program::ImageViewer => "Image Viewer",
            Program::AudioPlayer => "Audio Player",
            Program::VideoPlayer => "Video Player",
            Program::Terminal => "Terminal",
            Program::FileManager => "File Manager",
            Program::HexViewer => "Hex Viewer",
            Program::None => "(None)",
        }
    }
    
    pub fn icon(&self) -> &'static str {
        match self {
            Program::TextEditor => "Aa",
            Program::ImageViewer => "[]",
            Program::AudioPlayer => "d)",
            Program::VideoPlayer => ">|",
            Program::Terminal => ">_",
            Program::FileManager => "//",
            Program::HexViewer => "0x",
            Program::None => "??",
        }
    }
}

/// File association entry
#[derive(Clone, Debug)]
pub struct FileAssociation {
    pub extension: String,
    pub category: FileCategory,
    pub program: Program,
    pub description: String,
}

/// Global file association registry
pub struct FileRegistry {
    associations: BTreeMap<String, FileAssociation>,
}

impl FileRegistry {
    pub const fn new() -> Self {
        FileRegistry {
            associations: BTreeMap::new(),
        }
    }
    
    /// Initialize with default associations
    pub fn init_defaults(&mut self) {
        // Text files
        self.register("txt", FileCategory::Text, Program::TextEditor, "Text Document");
        self.register("log", FileCategory::Text, Program::TextEditor, "Log File");
        self.register("md", FileCategory::Text, Program::TextEditor, "Markdown");
        self.register("rs", FileCategory::Text, Program::TextEditor, "Rust Source");
        self.register("c", FileCategory::Text, Program::TextEditor, "C Source");
        self.register("h", FileCategory::Text, Program::TextEditor, "C Header");
        self.register("cpp", FileCategory::Text, Program::TextEditor, "C++ Source");
        self.register("py", FileCategory::Text, Program::TextEditor, "Python Script");
        self.register("js", FileCategory::Text, Program::TextEditor, "JavaScript");
        self.register("json", FileCategory::Text, Program::TextEditor, "JSON Data");
        self.register("xml", FileCategory::Text, Program::TextEditor, "XML Document");
        self.register("html", FileCategory::Text, Program::TextEditor, "HTML Page");
        self.register("css", FileCategory::Text, Program::TextEditor, "CSS Stylesheet");
        self.register("toml", FileCategory::Text, Program::TextEditor, "TOML Config");
        self.register("cfg", FileCategory::Text, Program::TextEditor, "Config File");
        self.register("ini", FileCategory::Text, Program::TextEditor, "INI Config");
        
        // Image files
        self.register("png", FileCategory::Image, Program::ImageViewer, "PNG Image");
        self.register("jpg", FileCategory::Image, Program::ImageViewer, "JPEG Image");
        self.register("jpeg", FileCategory::Image, Program::ImageViewer, "JPEG Image");
        self.register("gif", FileCategory::Image, Program::ImageViewer, "GIF Image");
        self.register("bmp", FileCategory::Image, Program::ImageViewer, "Bitmap Image");
        self.register("ico", FileCategory::Image, Program::ImageViewer, "Icon File");
        self.register("svg", FileCategory::Image, Program::ImageViewer, "SVG Vector");
        
        // Audio files
        self.register("mp3", FileCategory::Audio, Program::AudioPlayer, "MP3 Audio");
        self.register("wav", FileCategory::Audio, Program::AudioPlayer, "WAV Audio");
        self.register("ogg", FileCategory::Audio, Program::AudioPlayer, "OGG Audio");
        self.register("flac", FileCategory::Audio, Program::AudioPlayer, "FLAC Audio");
        
        // Video files
        self.register("mp4", FileCategory::Video, Program::VideoPlayer, "MP4 Video");
        self.register("avi", FileCategory::Video, Program::VideoPlayer, "AVI Video");
        self.register("mkv", FileCategory::Video, Program::VideoPlayer, "MKV Video");
        self.register("webm", FileCategory::Video, Program::VideoPlayer, "WebM Video");
        
        // Archive files
        self.register("zip", FileCategory::Archive, Program::FileManager, "ZIP Archive");
        self.register("tar", FileCategory::Archive, Program::FileManager, "TAR Archive");
        self.register("gz", FileCategory::Archive, Program::FileManager, "GZip Archive");
        self.register("7z", FileCategory::Archive, Program::FileManager, "7-Zip Archive");
        self.register("rar", FileCategory::Archive, Program::FileManager, "RAR Archive");
        
        // Executable files
        self.register("elf", FileCategory::Executable, Program::Terminal, "ELF Executable");
        self.register("bin", FileCategory::Executable, Program::HexViewer, "Binary File");
        self.register("exe", FileCategory::Executable, Program::Terminal, "Executable");
        
        // Documents
        self.register("pdf", FileCategory::Document, Program::TextEditor, "PDF Document");
        self.register("doc", FileCategory::Document, Program::TextEditor, "Word Document");
    }
    
    /// Register a file association
    pub fn register(&mut self, ext: &str, category: FileCategory, program: Program, desc: &str) {
        let ext_lower = ext.to_lowercase();
        self.associations.insert(ext_lower.clone(), FileAssociation {
            extension: ext_lower,
            category,
            program,
            description: String::from(desc),
        });
    }
    
    /// Get association for an extension
    pub fn get(&self, ext: &str) -> Option<&FileAssociation> {
        self.associations.get(&ext.to_lowercase())
    }
    
    /// Get program for a file extension
    pub fn get_program(&self, ext: &str) -> Program {
        self.get(ext)
            .map(|a| a.program.clone())
            .unwrap_or(Program::None)
    }
    
    /// Get category for a file extension
    pub fn get_category(&self, ext: &str) -> FileCategory {
        self.get(ext)
            .map(|a| a.category)
            .unwrap_or(FileCategory::Unknown)
    }
    
    /// Set program for an extension
    pub fn set_program(&mut self, ext: &str, program: Program) {
        if let Some(assoc) = self.associations.get_mut(&ext.to_lowercase()) {
            assoc.program = program;
        }
    }
    
    /// Get all associations
    pub fn list_all(&self) -> Vec<&FileAssociation> {
        self.associations.values().collect()
    }
    
    /// Get associations by category
    pub fn list_by_category(&self, category: FileCategory) -> Vec<&FileAssociation> {
        self.associations.values()
            .filter(|a| a.category == category)
            .collect()
    }
    
    /// Get file extension from filename
    pub fn get_extension(filename: &str) -> Option<&str> {
        filename.rsplit('.').next()
    }
    
    /// Get icon for file based on extension
    pub fn get_file_icon(&self, filename: &str) -> &'static str {
        if let Some(ext) = Self::get_extension(filename) {
            match self.get_category(ext) {
                FileCategory::Text => "Aa",
                FileCategory::Image => "<>",
                FileCategory::Audio => "d)",
                FileCategory::Video => ">|",
                FileCategory::Archive => "{}",
                FileCategory::Executable => ">>",
                FileCategory::Document => "[]",
                FileCategory::Unknown => "??",
            }
        } else {
            "??"
        }
    }
}

/// Global registry instance
static FILE_REGISTRY: Mutex<FileRegistry> = Mutex::new(FileRegistry::new());

/// Initialize file associations
pub fn init() {
    FILE_REGISTRY.lock().init_defaults();
    crate::serial_println!("[FILE_ASSOC] File associations initialized");
}

/// Get program for file
pub fn get_program_for_file(filename: &str) -> Program {
    let registry = FILE_REGISTRY.lock();
    if let Some(ext) = FileRegistry::get_extension(filename) {
        registry.get_program(ext)
    } else {
        Program::None
    }
}

/// Get file category
pub fn get_file_category(filename: &str) -> FileCategory {
    let registry = FILE_REGISTRY.lock();
    if let Some(ext) = FileRegistry::get_extension(filename) {
        registry.get_category(ext)
    } else {
        FileCategory::Unknown
    }
}

/// Get file icon
pub fn get_file_icon(filename: &str) -> &'static str {
    let registry = FILE_REGISTRY.lock();
    registry.get_file_icon(filename)
}

/// Set program for extension
pub fn set_program(ext: &str, program: Program) {
    FILE_REGISTRY.lock().set_program(ext, program);
}

/// List all associations
pub fn list_associations() -> Vec<(String, String, String)> {
    let registry = FILE_REGISTRY.lock();
    registry.list_all()
        .iter()
        .map(|a| (a.extension.clone(), a.program.name().into(), a.description.clone()))
        .collect()
}

/// List all available programs
pub fn list_programs() -> Vec<(Program, &'static str)> {
    vec![
        (Program::TextEditor, "Text Editor"),
        (Program::ImageViewer, "Image Viewer"),
        (Program::AudioPlayer, "Audio Player"),
        (Program::VideoPlayer, "Video Player"),
        (Program::Terminal, "Terminal"),
        (Program::FileManager, "File Manager"),
        (Program::HexViewer, "Hex Viewer"),
        (Program::None, "(None)"),
    ]
}
