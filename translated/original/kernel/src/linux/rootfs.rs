//! Linux rootfs management
//!
//! Handles creation and management of the Linux root filesystem structure.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

/// Create the Linux filesystem structure
pub fn create_structure() -> Result<(), &'static str> {
    crate::ramfs::with_fs(|fs| {
        // Create /linux root
        fs.mkdir("/linux").ok();
        
        // Create standard Linux directories
        let dirs = [
            "/linux/bin",
            "/linux/sbin",
            "/linux/etc",
            "/linux/etc/init.d",
            "/linux/etc/network",
            "/linux/dev",
            "/linux/proc",
            "/linux/sys",
            "/linux/tmp",
            "/linux/var",
            "/linux/var/log",
            "/linux/var/run",
            "/linux/home",
            "/linux/root",
            "/linux/usr",
            "/linux/usr/bin",
            "/linux/usr/sbin",
            "/linux/usr/lib",
            "/linux/usr/share",
            "/linux/lib",
            "/linux/lib64",
            "/linux/opt",
            "/linux/mnt",
            "/linux/run",
        ];
        
        for dir in dirs {
            fs.mkdir(dir).ok();
        }
        
        Ok(())
    })
}

/// Setup essential system files
pub fn setup_essential_files() -> Result<(), &'static str> {
    crate::ramfs::with_fs(|fs| {
        // /etc/hostname
        fs.touch("/linux/etc/hostname").ok();
        fs.write_file("/linux/etc/hostname", b"alpine\n").ok();
        
        // /etc/hosts
        fs.touch("/linux/etc/hosts").ok();
        fs.write_file("/linux/etc/hosts", 
            b"127.0.0.1\tlocalhost\n::1\t\tlocalhost\n127.0.0.1\talpine\n").ok();
        
        // /etc/passwd
        fs.touch("/linux/etc/passwd").ok();
        fs.write_file("/linux/etc/passwd",
            b"root:x:0:0:root:/root:/bin/sh\nnobody:x:65534:65534:nobody:/:/sbin/nologin\n").ok();
        
        // /etc/group
        fs.touch("/linux/etc/group").ok();
        fs.write_file("/linux/etc/group",
            b"root:x:0:\nnogroup:x:65534:\n").ok();
        
        // /etc/shadow (empty passwords)
        fs.touch("/linux/etc/shadow").ok();
        fs.write_file("/linux/etc/shadow",
            b"root::0:0:99999:7:::\n").ok();
        
        // /etc/os-release
        fs.touch("/linux/etc/os-release").ok();
        fs.write_file("/linux/etc/os-release",
            b"NAME=\"Alpine Linux\"\nID=alpine\nVERSION_ID=3.19\nPRETTY_NAME=\"Alpine Linux v3.19 (TrustOS)\"\n").ok();
        
        // /etc/profile
        fs.touch("/linux/etc/profile").ok();
        fs.write_file("/linux/etc/profile",
            b"export PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin\nexport PS1='\\u@\\h:\\w\\$ '\n").ok();
        
        // /etc/resolv.conf
        fs.touch("/linux/etc/resolv.conf").ok();
        fs.write_file("/linux/etc/resolv.conf",
            b"nameserver 8.8.8.8\nnameserver 8.8.4.4\n").ok();
        
        // /etc/network/interfaces
        fs.touch("/linux/etc/network/interfaces").ok();
        fs.write_file("/linux/etc/network/interfaces",
            b"auto lo\niface lo inet loopback\n\nauto eth0\niface eth0 inet dhcp\n").ok();
        
        // /etc/motd (message of the day)
        fs.touch("/linux/etc/motd").ok();
        fs.write_file("/linux/etc/motd",
            b"\nWelcome to Alpine Linux on TrustOS!\n\nThis is a Linux-compatible environment running within TrustOS.\nType 'help' for available commands, 'exit' to return to TrustOS.\n\n").ok();
        
        // /root/.profile
        fs.touch("/linux/root/.profile").ok();
        fs.write_file("/linux/root/.profile",
            b"export HOME=/root\nexport PATH=/bin:/sbin:/usr/bin:/usr/sbin\n").ok();
        
        // /root/.ash_history (empty)
        fs.touch("/linux/root/.ash_history").ok();
        
        // Create /proc entries (virtual)
        fs.touch("/linux/proc/version").ok();
        fs.write_file("/linux/proc/version", 
            b"Linux version 5.15.0-alpine (trustos@build) (gcc 12.2.0) TrustOS Subsystem\n").ok();
        
        fs.touch("/linux/proc/cpuinfo").ok();
        fs.write_file("/linux/proc/cpuinfo",
            b"processor\t: 0\nvendor_id\t: TrustOS\nmodel name\t: TrustOS Virtual CPU\ncpu MHz\t\t: 3000.000\n").ok();
        
        fs.touch("/linux/proc/meminfo").ok();
        fs.write_file("/linux/proc/meminfo",
            b"MemTotal:        4096000 kB\nMemFree:         3000000 kB\nMemAvailable:    3500000 kB\n").ok();
        
        fs.touch("/linux/proc/uptime").ok();
        fs.write_file("/linux/proc/uptime", b"1000.00 900.00\n").ok();
        
        Ok(())
    })
}

/// Get the absolute TrustOS path from a Linux path
pub fn linux_to_trustos_path(linux_path: &str, cwd: &str) -> String {
    let abs_linux = if linux_path.starts_with('/') {
        normalize_path(linux_path)
    } else if linux_path == "~" || linux_path.starts_with("~/") {
        if linux_path == "~" {
            String::from("/root")
        } else {
            format!("/root/{}", &linux_path[2..])
        }
    } else {
        // Relative path
        if cwd == "/" {
            normalize_path(&format!("/{}", linux_path))
        } else {
            normalize_path(&format!("{}/{}", cwd, linux_path))
        }
    };
    
    format!("/linux{}", abs_linux)
}

/// Normalize a path (handle . and ..)
fn normalize_path(path: &str) -> String {
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

/// Check if a path exists in the Linux rootfs
pub fn exists(path: &str) -> bool {
    let trustos_path = format!("/linux{}", path);
    crate::ramfs::with_fs(|fs| fs.exists(&trustos_path))
}

/// Check if path is a directory
pub fn is_directory(path: &str) -> bool {
    let trustos_path = format!("/linux{}", path);
    crate::ramfs::with_fs(|fs| {
        if let Ok(entry) = fs.stat(&trustos_path) {
            entry.file_type == crate::ramfs::FileType::Directory
        } else {
            false
        }
    })
}

/// Read a file from Linux rootfs
pub fn read_file(path: &str) -> Result<Vec<u8>, &'static str> {
    let trustos_path = format!("/linux{}", path);
    crate::ramfs::with_fs(|fs| {
        fs.read_file(&trustos_path)
            .map(|data| data.to_vec())
            .map_err(|_| "file not found")
    })
}

/// Write a file to Linux rootfs
pub fn write_file(path: &str, content: &[u8]) -> Result<(), &'static str> {
    let trustos_path = format!("/linux{}", path);
    crate::ramfs::with_fs(|fs| {
        // Create parent directories if needed
        let parent = parent_path(&trustos_path);
        create_parent_dirs(fs, &parent);
        
        // Create file if it doesn't exist
        if !fs.exists(&trustos_path) {
            fs.touch(&trustos_path).map_err(|_| "cannot create file")?;
        }
        
        fs.write_file(&trustos_path, content).map_err(|_| "write failed")
    })
}

fn parent_path(path: &str) -> String {
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

fn create_parent_dirs(fs: &mut crate::ramfs::RamFs, path: &str) {
    if path == "/" || path.is_empty() {
        return;
    }
    
    if !fs.exists(path) {
        // Create parent first
        let parent = parent_path(path);
        create_parent_dirs(fs, &parent);
        fs.mkdir(path).ok();
    }
}

/// List directory contents
pub fn list_dir(path: &str) -> Result<Vec<(String, bool, usize)>, &'static str> {
    let trustos_path = format!("/linux{}", path);
    crate::ramfs::with_fs(|fs| {
        let items = fs.ls(Some(&trustos_path)).map_err(|_| "cannot list directory")?;
        Ok(items.into_iter().map(|(name, ftype, size)| {
            (name, ftype == crate::ramfs::FileType::Directory, size)
        }).collect())
    })
}
