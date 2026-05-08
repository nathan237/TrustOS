



use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;


pub fn kzl() -> Result<(), &'static str> {
    crate::ramfs::bh(|fs| {
        
        fs.mkdir("/linux").ok();
        
        
        let bfy = [
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
        
        for it in bfy {
            fs.mkdir(it).ok();
        }
        
        Ok(())
    })
}


pub fn oqg() -> Result<(), &'static str> {
    crate::ramfs::bh(|fs| {
        
        fs.touch("/linux/etc/hostname").ok();
        fs.write_file("/linux/etc/hostname", b"alpine\n").ok();
        
        
        fs.touch("/linux/etc/hosts").ok();
        fs.write_file("/linux/etc/hosts", 
            b"127.0.0.1\tlocalhost\n::1\t\tlocalhost\n127.0.0.1\talpine\n").ok();
        
        
        fs.touch("/linux/etc/passwd").ok();
        fs.write_file("/linux/etc/passwd",
            b"root:x:0:0:root:/root:/bin/sh\nnobody:x:65534:65534:nobody:/:/sbin/nologin\n").ok();
        
        
        fs.touch("/linux/etc/group").ok();
        fs.write_file("/linux/etc/group",
            b"root:x:0:\nnogroup:x:65534:\n").ok();
        
        
        fs.touch("/linux/etc/shadow").ok();
        fs.write_file("/linux/etc/shadow",
            b"root::0:0:99999:7:::\n").ok();
        
        
        fs.touch("/linux/etc/os-release").ok();
        fs.write_file("/linux/etc/os-release",
            b"NAME=\"Alpine Linux\"\nID=alpine\nVERSION_ID=3.19\nPRETTY_NAME=\"Alpine Linux v3.19 (TrustOS)\"\n").ok();
        
        
        fs.touch("/linux/etc/profile").ok();
        fs.write_file("/linux/etc/profile",
            b"export PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin\nexport PS1='\\u@\\h:\\w\\$ '\n").ok();
        
        
        fs.touch("/linux/etc/resolv.conf").ok();
        fs.write_file("/linux/etc/resolv.conf",
            b"nameserver 8.8.8.8\nnameserver 8.8.4.4\n").ok();
        
        
        fs.touch("/linux/etc/network/interfaces").ok();
        fs.write_file("/linux/etc/network/interfaces",
            b"auto lo\niface lo inet loopback\n\nauto eth0\niface eth0 inet dhcp\n").ok();
        
        
        fs.touch("/linux/etc/motd").ok();
        fs.write_file("/linux/etc/motd",
            b"\nWelcome to Alpine Linux on TrustOS!\n\nThis is a Linux-compatible environment running within TrustOS.\nType 'help' for available commands, 'exit' to return to TrustOS.\n\n").ok();
        
        
        fs.touch("/linux/root/.profile").ok();
        fs.write_file("/linux/root/.profile",
            b"export HOME=/root\nexport PATH=/bin:/sbin:/usr/bin:/usr/sbin\n").ok();
        
        
        fs.touch("/linux/root/.ash_history").ok();
        
        
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


pub fn myx(bni: &str, cwd: &str) -> String {
    let jtd = if bni.starts_with('/') {
        normalize_path(bni)
    } else if bni == "~" || bni.starts_with("~/") {
        if bni == "~" {
            String::from("/root")
        } else {
            format!("/root/{}", &bni[2..])
        }
    } else {
        
        if cwd == "/" {
            normalize_path(&format!("/{}", bni))
        } else {
            normalize_path(&format!("{}/{}", cwd, bni))
        }
    };
    
    format!("/linux{}", jtd)
}


fn normalize_path(path: &str) -> String {
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


pub fn exists(path: &str) -> bool {
    let beg = format!("/linux{}", path);
    crate::ramfs::bh(|fs| fs.exists(&beg))
}


pub fn is_directory(path: &str) -> bool {
    let beg = format!("/linux{}", path);
    crate::ramfs::bh(|fs| {
        if let Ok(entry) = fs.stat(&beg) {
            entry.file_type == crate::ramfs::FileType::Directory
        } else {
            false
        }
    })
}


pub fn read_file(path: &str) -> Result<Vec<u8>, &'static str> {
    let beg = format!("/linux{}", path);
    crate::ramfs::bh(|fs| {
        fs.read_file(&beg)
            .map(|data| data.to_vec())
            .map_err(|_| "file not found")
    })
}


pub fn write_file(path: &str, content: &[u8]) -> Result<(), &'static str> {
    let beg = format!("/linux{}", path);
    crate::ramfs::bh(|fs| {
        
        let parent = parent_path(&beg);
        hor(fs, &parent);
        
        
        if !fs.exists(&beg) {
            fs.touch(&beg).map_err(|_| "cannot create file")?;
        }
        
        fs.write_file(&beg, content).map_err(|_| "write failed")
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

fn hor(fs: &mut crate::ramfs::RamFs, path: &str) {
    if path == "/" || path.is_empty() {
        return;
    }
    
    if !fs.exists(path) {
        
        let parent = parent_path(path);
        hor(fs, &parent);
        fs.mkdir(path).ok();
    }
}


pub fn ikp(path: &str) -> Result<Vec<(String, bool, usize)>, &'static str> {
    let beg = format!("/linux{}", path);
    crate::ramfs::bh(|fs| {
        let items = fs.ls(Some(&beg)).map_err(|_| "cannot list directory")?;
        Ok(items.into_iter().map(|(name, wf, size)| {
            (name, wf == crate::ramfs::FileType::Directory, size)
        }).collect())
    })
}
