




use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::format;
use alloc::collections::BTreeMap;
use spin::Mutex;
use core::sync::atomic::{AtomicBool, Ordering};


pub type Ff = u32;

pub type Ew = u32;


pub const BGV_: Ff = 0;

pub const BGU_: Ew = 0;

pub const ALE_: Ew = 100;


pub const CJF_: usize = 32;

pub const DXC_: usize = 128;



const CCD_: u32 = 10_000;



fn gac(uy: &str, salt: &str) -> [u8; 32] {
    use crate::tls13::crypto::{asg, bmu};

    
    let mut result = bmu(salt.as_bytes(), uy.as_bytes());

    
    for _ in 1..CCD_ {
        result = asg(&result);
    }
    result
}


fn kxf(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut jr: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        jr |= x ^ y;
    }
    jr == 0
}


fn ibd(_username: &str) -> String {
    let mut buf = [0u8; 16];
    crate::rng::jeb(&mut buf);
    let mut j = String::with_capacity(32);
    for b in &buf {
        j.push_str(&format!("{:02x}", b));
    }
    j
}


fn mlc(ga: &str) -> [u8; 32] {
    let mut out = [0u8; 32];
    let bytes = ga.as_bytes();
    for i in 0..32 {
        let hi = i * 2;
        if hi + 1 >= bytes.len() { break; }
        let high = ies(bytes[hi]);
        let low = ies(bytes[hi + 1]);
        out[i] = (high << 4) | low;
    }
    out
}

fn ies(c: u8) -> u8 {
    match c {
        b'0'..=b'9' => c - b'0',
        b'a'..=b'f' => c - b'a' + 10,
        b'A'..=b'F' => c - b'A' + 10,
        _ => 0,
    }
}


#[derive(Clone, Debug)]
pub struct UserEntry {
    pub username: String,
    pub uid: Ff,
    pub gid: Ew,
    pub gecos: String,       
    pub home_dir: String,
    pub shell: String,
}

impl UserEntry {
    
    pub fn new(username: &str, uid: Ff, gid: Ew) -> Self {
        Self {
            username: String::from(username),
            uid,
            gid,
            gecos: String::new(),
            home_dir: format!("/home/{}", username),
            shell: String::from("/bin/tsh"),
        }
    }
    
    
    pub fn cdl() -> Self {
        Self {
            username: String::from("root"),
            uid: BGV_,
            gid: BGU_,
            gecos: String::from("System Administrator"),
            home_dir: String::from("/root"),
            shell: String::from("/bin/tsh"),
        }
    }
    
    
    pub fn to_passwd_line(&self) -> String {
        format!("{}:x:{}:{}:{}:{}:{}",
            self.username, self.uid, self.gid,
            self.gecos, self.home_dir, self.shell)
    }
    
    
    pub fn lzn(line: &str) -> Option<Self> {
        let au: Vec<&str> = line.split(':').collect();
        if au.len() < 7 {
            return None;
        }
        
        Some(Self {
            username: String::from(au[0]),
            uid: au[2].parse().ok()?,
            gid: au[3].parse().ok()?,
            gecos: String::from(au[4]),
            home_dir: String::from(au[5]),
            shell: String::from(au[6]),
        })
    }
}


#[derive(Clone, Debug)]
pub struct ShadowEntry {
    pub username: String,
    pub password_hash: [u8; 32],
    pub salt: String,
    pub last_changed: u64,    
    pub min_days: u32,        
    pub max_days: u32,        
    pub warn_days: u32,       
    pub inactive_days: i32,   
    pub expire_date: i64,     
}

impl ShadowEntry {
    
    pub fn new(username: &str, uy: &str) -> Self {
        let salt = ibd(username);
        let hash = gac(uy, &salt);
        
        Self {
            username: String::from(username),
            password_hash: hash,
            salt,
            last_changed: 0,
            min_days: 0,
            max_days: 99999,
            warn_days: 7,
            inactive_days: -1,
            expire_date: -1,
        }
    }
    
    
    pub fn locked(username: &str) -> Self {
        Self {
            username: String::from(username),
            password_hash: [0u8; 32],
            salt: String::from("!"),
            last_changed: 0,
            min_days: 0,
            max_days: 99999,
            warn_days: 7,
            inactive_days: -1,
            expire_date: -1,
        }
    }
    
    
    pub fn verify_password(&self, uy: &str) -> bool {
        if self.salt == "!" {
            return false; 
        }
        let hash = gac(uy, &self.salt);
        kxf(&hash, &self.password_hash)
    }
    
    
    pub fn set_password(&mut self, uy: &str) {
        self.salt = ibd(&self.username);
        self.password_hash = gac(uy, &self.salt);
    }
    
    
    pub fn ral(&self) -> String {
        
        let mut ieg = String::with_capacity(64);
        for b in &self.password_hash {
            ieg.push_str(&format!("{:02x}", b));
        }
        format!("{}:{}${}:{}:{}:{}:{}:{}:{}:",
            self.username,
            ieg,
            self.salt,
            self.last_changed,
            self.min_days,
            self.max_days,
            self.warn_days,
            self.inactive_days,
            self.expire_date)
    }
    
    
    pub fn qgm(line: &str) -> Option<Self> {
        let au: Vec<&str> = line.split(':').collect();
        if au.len() < 9 {
            return None;
        }
        
        
        let gad: Vec<&str> = au[1].split('$').collect();
        let (hash, salt) = if gad.len() >= 2 {
            (mlc(gad[0]), String::from(gad[1]))
        } else {
            ([0u8; 32], String::from("!"))
        };
        
        Some(Self {
            username: String::from(au[0]),
            password_hash: hash,
            salt,
            last_changed: au[2].parse().unwrap_or(0),
            min_days: au[3].parse().unwrap_or(0),
            max_days: au[4].parse().unwrap_or(99999),
            warn_days: au[5].parse().unwrap_or(7),
            inactive_days: au[6].parse().unwrap_or(-1),
            expire_date: au[7].parse().unwrap_or(-1),
        })
    }
}


#[derive(Clone, Debug)]
pub struct GroupEntry {
    pub name: String,
    pub gid: Ew,
    pub members: Vec<String>,
}

impl GroupEntry {
    pub fn new(name: &str, gid: Ew) -> Self {
        Self {
            name: String::from(name),
            gid,
            members: Vec::new(),
        }
    }
    
    
    pub fn to_group_line(&self) -> String {
        format!("{}:x:{}:{}", self.name, self.gid, self.members.join(","))
    }
    
    
    pub fn qgk(line: &str) -> Option<Self> {
        let au: Vec<&str> = line.split(':').collect();
        if au.len() < 3 {
            return None;
        }
        let name = String::from(au[0]);
        let gid: u32 = au[2].parse().ok()?;
        let members = if au.len() > 3 && !au[3].is_empty() {
            au[3].split(',').map(|j| String::from(j.trim())).collect()
        } else {
            Vec::new()
        };
        Some(Self { name, gid, members })
    }
}


pub struct Session {
    pub logged_in: bool,
    pub uid: Ff,
    pub gid: Ew,
    pub username: String,
    pub home_dir: String,
    pub login_time: u64,
}

impl Session {
    pub fn new() -> Self {
        Self {
            logged_in: false,
            uid: 0,
            gid: 0,
            username: String::new(),
            home_dir: String::from("/"),
            login_time: 0,
        }
    }
    
    pub fn is_root(&self) -> bool {
        self.uid == BGV_
    }
}


pub struct UserDatabase {
    users: BTreeMap<String, UserEntry>,
    shadows: BTreeMap<String, ShadowEntry>,
    groups: BTreeMap<String, GroupEntry>,
    next_uid: Ff,
    next_gid: Ew,
}

impl UserDatabase {
    pub fn new() -> Self {
        let mut fu = Self {
            users: BTreeMap::new(),
            shadows: BTreeMap::new(),
            groups: BTreeMap::new(),
            next_uid: 1000, 
            next_gid: 1000,
        };
        
        
        fu.groups.insert(String::from("root"), GroupEntry::new("root", BGU_));
        fu.groups.insert(String::from("users"), GroupEntry::new("users", ALE_));
        fu.groups.insert(String::from("wheel"), GroupEntry::new("wheel", 10)); 
        
        
        let cdl = UserEntry::cdl();
        let ohz = ShadowEntry::new("root", "toor");
        fu.users.insert(String::from("root"), cdl);
        fu.shadows.insert(String::from("root"), ohz);
        
        
        let axj = UserEntry {
            username: String::from("guest"),
            uid: 1000,
            gid: ALE_,
            gecos: String::from("Guest User"),
            home_dir: String::from("/home/guest"),
            shell: String::from("/bin/tsh"),
        };
        let mgm = ShadowEntry::new("guest", "guest");
        fu.users.insert(String::from("guest"), axj);
        fu.shadows.insert(String::from("guest"), mgm);
        fu.next_uid = 1001;
        
        fu
    }
    
    
    pub fn get_user(&self, username: &str) -> Option<&UserEntry> {
        self.users.get(username)
    }
    
    
    pub fn qit(&self, uid: Ff) -> Option<&UserEntry> {
        self.users.values().find(|iy| iy.uid == uid)
    }
    
    
    pub fn authenticate(&self, username: &str, uy: &str) -> bool {
        if let Some(shadow) = self.shadows.get(username) {
            shadow.verify_password(uy)
        } else {
            false
        }
    }
    
    
    pub fn add_user(&mut self, username: &str, uy: &str, dsj: bool) -> Result<Ff, &'static str> {
        
        if username.is_empty() || username.len() > CJF_ {
            return Err("Invalid username length");
        }
        
        if self.users.contains_key(username) {
            return Err("User already exists");
        }
        
        
        if !username.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err("Invalid characters in username");
        }
        
        let uid = self.next_uid;
        self.next_uid += 1;
        
        let gid = if dsj { 10 } else { ALE_ }; 
        
        let avp = UserEntry::new(username, uid, gid);
        let shadow = ShadowEntry::new(username, uy);
        
        self.users.insert(String::from(username), avp);
        self.shadows.insert(String::from(username), shadow);
        
        Ok(uid)
    }
    
    
    pub fn delete_user(&mut self, username: &str) -> Result<(), &'static str> {
        if username == "root" {
            return Err("Cannot delete root user");
        }
        
        if self.users.remove(username).is_none() {
            return Err("User not found");
        }
        
        self.shadows.remove(username);
        Ok(())
    }
    
    
    pub fn change_password(&mut self, username: &str, cnc: &str) -> Result<(), &'static str> {
        if let Some(shadow) = self.shadows.get_mut(username) {
            shadow.set_password(cnc);
            Ok(())
        } else {
            Err("User not found")
        }
    }
    
    
    pub fn list_users(&self) -> Vec<&UserEntry> {
        self.users.values().collect()
    }
    
    
    pub fn generate_passwd(&self) -> String {
        self.users.values()
            .map(|iy| iy.to_passwd_line())
            .collect::<Vec<_>>()
            .join("\n")
    }
    
    
    pub fn generate_group(&self) -> String {
        self.groups.values()
            .map(|g| g.to_group_line())
            .collect::<Vec<_>>()
            .join("\n")
    }
}


static FN_: Mutex<Option<UserDatabase>> = Mutex::new(None);
static GV_: Mutex<Option<Session>> = Mutex::new(None);
static BBB_: AtomicBool = AtomicBool::new(false);
static Ah: AtomicBool = AtomicBool::new(false);


pub fn init() {
    let mut fu = FN_.lock();
    *fu = Some(UserDatabase::new());
    
    let mut by = GV_.lock();
    *by = Some(Session::new());
    
    Ah.store(true, Ordering::SeqCst);
    
    crate::log_debug!("[AUTH] Authentication system initialized");
}


pub fn is_initialized() -> bool {
    Ah.load(Ordering::SeqCst)
}


pub fn qwf(aov: bool) {
    BBB_.store(aov, Ordering::SeqCst);
}


pub fn qmn() -> bool {
    BBB_.load(Ordering::SeqCst)
}


pub fn dmb() -> String {
    let by = GV_.lock();
    if let Some(ref j) = *by {
        if j.logged_in {
            return j.username.clone();
        }
    }
    String::from("nobody")
}


pub fn fpz() -> Ff {
    let by = GV_.lock();
    if let Some(ref j) = *by {
        j.uid
    } else {
        65534 
    }
}


pub fn fpp() -> Ew {
    let by = GV_.lock();
    if let Some(ref j) = *by {
        j.gid
    } else {
        65534 
    }
}


pub fn is_root() -> bool {
    let by = GV_.lock();
    if let Some(ref j) = *by {
        j.is_root()
    } else {
        false
    }
}


pub fn mta() -> bool {
    let by = GV_.lock();
    if let Some(ref j) = *by {
        j.logged_in
    } else {
        false
    }
}


pub fn ggf(username: &str, uy: &str) -> Result<(), &'static str> {
    let fu = FN_.lock();
    let fu = fu.as_ref().ok_or("Auth not initialized")?;
    
    if !fu.authenticate(username, uy) {
        return Err("Invalid username or password");
    }
    
    let avp = fu.get_user(username).ok_or("User not found")?;
    
    drop(fu); 
    
    let mut by = GV_.lock();
    if let Some(ref mut j) = *by {
        j.logged_in = true;
        j.uid = avp.uid;
        j.gid = avp.gid;
        j.username = avp.username.clone();
        j.home_dir = avp.home_dir.clone();
        j.login_time = crate::time::uptime_ms();
    }
    
    Ok(())
}


pub fn ilf() {
    let mut by = GV_.lock();
    if let Some(ref mut j) = *by {
        j.logged_in = false;
        j.uid = 0;
        j.gid = 0;
        j.username.clear();
        j.home_dir = String::from("/");
        j.login_time = 0;
    }
}


pub fn add_user(username: &str, uy: &str, dsj: bool) -> Result<Ff, &'static str> {
    if !is_root() && mta() {
        return Err("Permission denied: must be root");
    }
    
    let mut fu = FN_.lock();
    let fu = fu.as_mut().ok_or("Auth not initialized")?;
    fu.add_user(username, uy, dsj)
}


pub fn delete_user(username: &str) -> Result<(), &'static str> {
    if !is_root() {
        return Err("Permission denied: must be root");
    }
    
    let mut fu = FN_.lock();
    let fu = fu.as_mut().ok_or("Auth not initialized")?;
    fu.delete_user(username)
}


pub fn change_password(username: &str, gkq: &str, cnc: &str) -> Result<(), &'static str> {
    let fu = FN_.lock();
    let lbw = fu.as_ref().ok_or("Auth not initialized")?;
    
    
    let current = dmb();
    if current != username && !is_root() {
        return Err("Permission denied");
    }
    
    
    if !is_root() && !lbw.authenticate(username, gkq) {
        return Err("Current password incorrect");
    }
    
    drop(fu);
    
    let mut fu = FN_.lock();
    let lbv = fu.as_mut().ok_or("Auth not initialized")?;
    lbv.change_password(username, cnc)
}


pub fn list_users() -> Vec<(String, Ff, Ew, String)> {
    let fu = FN_.lock();
    if let Some(ref fu) = *fu {
        fu.list_users()
            .iter()
            .map(|iy| (iy.username.clone(), iy.uid, iy.gid, iy.gecos.clone()))
            .collect()
    } else {
        Vec::new()
    }
}


pub fn qhx(username: &str) -> Option<String> {
    let fu = FN_.lock();
    fu.as_ref()?.get_user(username).map(|iy| iy.home_dir.clone())
}


pub fn ila() -> bool {
    use crate::framebuffer::{C_, B_, A_, R_, D_};
    
    crate::println!();
    crate::n!(C_, "╔════════════════════════════════════════╗");
    crate::n!(C_, "║         T-RustOS Login                 ║");
    crate::n!(C_, "╚════════════════════════════════════════╝");
    crate::println!();
    
    let mut efr = 0;
    const AGO_: u32 = 3;
    
    while efr < AGO_ {
        
        crate::bq!(B_, "login: ");
        let mut jpq = [0u8; 64];
        let pqq = crate::keyboard::read_line(&mut jpq);
        let username = core::str::from_utf8(&jpq[..pqq])
            .unwrap_or("")
            .trim();
        
        if username.is_empty() {
            continue;
        }
        
        
        crate::bq!(B_, "password: ");
        let mut cci = [0u8; 128];
        let dwg = crate::keyboard::cpb(&mut cci);
        let uy = core::str::from_utf8(&cci[..dwg])
            .unwrap_or("")
            .trim();
        crate::println!(); 
        
        
        match ggf(username, uy) {
            Ok(()) => {
                crate::println!();
                crate::n!(B_, "Welcome, {}!", username);
                crate::println!();
                return true;
            }
            Err(_) => {
                efr += 1;
                if efr < AGO_ {
                    crate::n!(A_, "Login incorrect. {} attempts remaining.", 
                        AGO_ - efr);
                } else {
                    crate::n!(A_, "Too many failed attempts.");
                }
            }
        }
    }
    
    false
}


pub fn hgb() {
    let _ = ggf("root", "toor");
}


pub fn hoo() {
    if !crate::ramfs::is_initialized() {
        return;
    }
    
    
    let fu = FN_.lock();
    if let Some(ref fu) = *fu {
        let nru = fu.generate_passwd();
        let mgd = fu.generate_group();
        
        drop(fu);
        
        crate::ramfs::bh(|fs| {
            
            let _ = fs.touch("/etc/passwd");
            let _ = fs.write_file("/etc/passwd", nru.as_bytes());
            
            
            let _ = fs.touch("/etc/group");
            let _ = fs.write_file("/etc/group", mgd.as_bytes());
            
            
            let _ = fs.touch("/etc/shadow");
            let _ = fs.write_file("/etc/shadow", b"# Shadow file - passwords hidden\n");
        });
    }
}



pub fn nab() {
    if !crate::ramfs::is_initialized() {
        return;
    }
    
    let mut iua: Option<alloc::vec::Vec<u8>> = None;
    let mut icy: Option<alloc::vec::Vec<u8>> = None;
    
    crate::ramfs::bh(|fs| {
        if let Ok(data) = fs.read_file("/etc/passwd") {
            iua = Some(data.to_vec());
        }
        if let Ok(data) = fs.read_file("/etc/group") {
            icy = Some(data.to_vec());
        }
    });
    
    let mut fu = FN_.lock();
    if let Some(ref mut fu) = *fu {
        
        if let Some(ref data) = iua {
            if let Ok(content) = core::str::from_utf8(data) {
                let mut bhq = 0u32;
                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') { continue; }
                    if let Some(entry) = UserEntry::lzn(line) {
                        if !fu.users.contains_key(&entry.username) {
                            let name = entry.username.clone();
                            fu.users.insert(name, entry);
                            bhq += 1;
                        }
                    }
                }
                if bhq > 0 {
                    crate::log!("[AUTH] Loaded {} users from /etc/passwd", bhq);
                }
            }
        }
        
        
        
        
        let _ = icy; 
    }
}


pub fn qyk() {
    hoo();
    crate::log_debug!("[AUTH] Synced user database to /etc files");
}
