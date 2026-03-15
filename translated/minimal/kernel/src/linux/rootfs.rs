



use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;


pub fn rqt() -> Result<(), &'static str> {
    crate::ramfs::fh(|fs| {
        
        fs.ut("/linux").bq();
        
        
        let dgh = [
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
        
        for te in dgh {
            fs.ut(te).bq();
        }
        
        Ok(())
    })
}


pub fn wkr() -> Result<(), &'static str> {
    crate::ramfs::fh(|fs| {
        
        fs.touch("/linux/etc/hostname").bq();
        fs.ns("/linux/etc/hostname", b"alpine\n").bq();
        
        
        fs.touch("/linux/etc/hosts").bq();
        fs.ns("/linux/etc/hosts", 
            b"127.0.0.1\tlocalhost\n::1\t\tlocalhost\n127.0.0.1\talpine\n").bq();
        
        
        fs.touch("/linux/etc/passwd").bq();
        fs.ns("/linux/etc/passwd",
            b"root:x:0:0:root:/root:/bin/sh\nnobody:x:65534:65534:nobody:/:/sbin/nologin\n").bq();
        
        
        fs.touch("/linux/etc/group").bq();
        fs.ns("/linux/etc/group",
            b"root:x:0:\nnogroup:x:65534:\n").bq();
        
        
        fs.touch("/linux/etc/shadow").bq();
        fs.ns("/linux/etc/shadow",
            b"root::0:0:99999:7:::\n").bq();
        
        
        fs.touch("/linux/etc/os-release").bq();
        fs.ns("/linux/etc/os-release",
            b"NAME=\"Alpine Linux\"\nID=alpine\nVERSION_ID=3.19\nPRETTY_NAME=\"Alpine Linux v3.19 (TrustOS)\"\n").bq();
        
        
        fs.touch("/linux/etc/profile").bq();
        fs.ns("/linux/etc/profile",
            b"export PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin\nexport PS1='\\u@\\h:\\w\\$ '\n").bq();
        
        
        fs.touch("/linux/etc/resolv.conf").bq();
        fs.ns("/linux/etc/resolv.conf",
            b"nameserver 8.8.8.8\nnameserver 8.8.4.4\n").bq();
        
        
        fs.touch("/linux/etc/network/interfaces").bq();
        fs.ns("/linux/etc/network/interfaces",
            b"auto lo\niface lo inet loopback\n\nauto eth0\niface eth0 inet dhcp\n").bq();
        
        
        fs.touch("/linux/etc/motd").bq();
        fs.ns("/linux/etc/motd",
            b"\nWelcome to Alpine Linux on TrustOS!\n\nThis is a Linux-compatible environment running within TrustOS.\nType 'help' for available commands, 'exit' to return to TrustOS.\n\n").bq();
        
        
        fs.touch("/linux/root/.profile").bq();
        fs.ns("/linux/root/.profile",
            b"export HOME=/root\nexport PATH=/bin:/sbin:/usr/bin:/usr/sbin\n").bq();
        
        
        fs.touch("/linux/root/.ash_history").bq();
        
        
        fs.touch("/linux/proc/version").bq();
        fs.ns("/linux/proc/version", 
            b"Linux version 5.15.0-alpine (trustos@build) (gcc 12.2.0) TrustOS Subsystem\n").bq();
        
        fs.touch("/linux/proc/cpuinfo").bq();
        fs.ns("/linux/proc/cpuinfo",
            b"processor\t: 0\nvendor_id\t: TrustOS\nmodel name\t: TrustOS Virtual CPU\ncpu MHz\t\t: 3000.000\n").bq();
        
        fs.touch("/linux/proc/meminfo").bq();
        fs.ns("/linux/proc/meminfo",
            b"MemTotal:        4096000 kB\nMemFree:         3000000 kB\nMemAvailable:    3500000 kB\n").bq();
        
        fs.touch("/linux/proc/uptime").bq();
        fs.ns("/linux/proc/uptime", b"1000.00 900.00\n").bq();
        
        Ok(())
    })
}


pub fn ufk(dss: &str, jv: &str) -> String {
    let qen = if dss.cj('/') {
        bro(dss)
    } else if dss == "~" || dss.cj("~/") {
        if dss == "~" {
            String::from("/root")
        } else {
            format!("/root/{}", &dss[2..])
        }
    } else {
        
        if jv == "/" {
            bro(&format!("/{}", dss))
        } else {
            bro(&format!("{}/{}", jv, dss))
        }
    };
    
    format!("/linux{}", qen)
}


fn bro(path: &str) -> String {
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


pub fn aja(path: &str) -> bool {
    let dda = format!("/linux{}", path);
    crate::ramfs::fh(|fs| fs.aja(&dda))
}


pub fn cfr(path: &str) -> bool {
    let dda = format!("/linux{}", path);
    crate::ramfs::fh(|fs| {
        if let Ok(bt) = fs.hm(&dda) {
            bt.kd == crate::ramfs::FileType::K
        } else {
            false
        }
    })
}


pub fn mq(path: &str) -> Result<Vec<u8>, &'static str> {
    let dda = format!("/linux{}", path);
    crate::ramfs::fh(|fs| {
        fs.mq(&dda)
            .map(|f| f.ip())
            .jd(|_| "file not found")
    })
}


pub fn ns(path: &str, ca: &[u8]) -> Result<(), &'static str> {
    let dda = format!("/linux{}", path);
    crate::ramfs::fh(|fs| {
        
        let tu = bhs(&dda);
        nhf(fs, &tu);
        
        
        if !fs.aja(&dda) {
            fs.touch(&dda).jd(|_| "cannot create file")?;
        }
        
        fs.ns(&dda, ca).jd(|_| "write failed")
    })
}

fn bhs(path: &str) -> String {
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

fn nhf(fs: &mut crate::ramfs::RamFs, path: &str) {
    if path == "/" || path.is_empty() {
        return;
    }
    
    if !fs.aja(path) {
        
        let tu = bhs(path);
        nhf(fs, &tu);
        fs.ut(path).bq();
    }
}


pub fn ojo(path: &str) -> Result<Vec<(String, bool, usize)>, &'static str> {
    let dda = format!("/linux{}", path);
    crate::ramfs::fh(|fs| {
        let pj = fs.awb(Some(&dda)).jd(|_| "cannot list directory")?;
        Ok(pj.dse().map(|(j, are, aw)| {
            (j, are == crate::ramfs::FileType::K, aw)
        }).collect())
    })
}
