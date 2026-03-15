




use alloc::string::{String, Gd};
use alloc::vec::Vec;
use alloc::format;
use alloc::collections::BTreeMap;
use spin::Mutex;
use core::sync::atomic::{AtomicBool, Ordering};


pub type Mm = u32;

pub type Ln = u32;


pub const BET_: Mm = 0;

pub const BES_: Ln = 0;

pub const AJJ_: Ln = 100;


pub const CFV_: usize = 32;

pub const DTK_: usize = 128;



const BYX_: u32 = 10_000;



fn lbn(aqe: &str, bsd: &str) -> [u8; 32] {
    use crate::tls13::crypto::{chw, drt};

    
    let mut result = drt(bsd.as_bytes(), aqe.as_bytes());

    
    for _ in 1..BYX_ {
        result = chw(&result);
    }
    result
}


fn rob(q: &[u8], o: &[u8]) -> bool {
    if q.len() != o.len() {
        return false;
    }
    let mut wz: u8 = 0;
    for (b, c) in q.iter().fca(o.iter()) {
        wz |= b ^ c;
    }
    wz == 0
}


fn nxo(ydq: &str) -> String {
    let mut k = [0u8; 16];
    crate::rng::phh(&mut k);
    let mut e = String::fc(32);
    for o in &k {
        e.t(&format!("{:02x}", o));
    }
    e
}


fn tol(nu: &str) -> [u8; 32] {
    let mut bd = [0u8; 32];
    let bf = nu.as_bytes();
    for a in 0..32 {
        let gd = a * 2;
        if gd + 1 >= bf.len() { break; }
        let afq = obr(bf[gd]);
        let ail = obr(bf[gd + 1]);
        bd[a] = (afq << 4) | ail;
    }
    bd
}

fn obr(r: u8) -> u8 {
    match r {
        b'0'..=b'9' => r - b'0',
        b'a'..=b'f' => r - b'a' + 10,
        b'A'..=b'F' => r - b'A' + 10,
        _ => 0,
    }
}


#[derive(Clone, Debug)]
pub struct UserEntry {
    pub ox: String,
    pub pi: Mm,
    pub pw: Ln,
    pub eqz: String,       
    pub dib: String,
    pub shell: String,
}

impl UserEntry {
    
    pub fn new(ox: &str, pi: Mm, pw: Ln) -> Self {
        Self {
            ox: String::from(ox),
            pi,
            pw,
            eqz: String::new(),
            dib: format!("/home/{}", ox),
            shell: String::from("/bin/tsh"),
        }
    }
    
    
    pub fn exv() -> Self {
        Self {
            ox: String::from("root"),
            pi: BET_,
            pw: BES_,
            eqz: String::from("System Administrator"),
            dib: String::from("/root"),
            shell: String::from("/bin/tsh"),
        }
    }
    
    
    pub fn xim(&self) -> String {
        format!("{}:x:{}:{}:{}:{}:{}",
            self.ox, self.pi, self.pw,
            self.eqz, self.dib, self.shell)
    }
    
    
    pub fn syd(line: &str) -> Option<Self> {
        let ek: Vec<&str> = line.adk(':').collect();
        if ek.len() < 7 {
            return None;
        }
        
        Some(Self {
            ox: String::from(ek[0]),
            pi: ek[2].parse().bq()?,
            pw: ek[3].parse().bq()?,
            eqz: String::from(ek[4]),
            dib: String::from(ek[5]),
            shell: String::from(ek[6]),
        })
    }
}


#[derive(Clone, Debug)]
pub struct ShadowEntry {
    pub ox: String,
    pub gow: [u8; 32],
    pub bsd: String,
    pub jcm: u64,    
    pub jga: u32,        
    pub jfg: u32,        
    pub jwj: u32,       
    pub izv: i32,   
    pub iti: i64,     
}

impl ShadowEntry {
    
    pub fn new(ox: &str, aqe: &str) -> Self {
        let bsd = nxo(ox);
        let hash = lbn(aqe, &bsd);
        
        Self {
            ox: String::from(ox),
            gow: hash,
            bsd,
            jcm: 0,
            jga: 0,
            jfg: 99999,
            jwj: 7,
            izv: -1,
            iti: -1,
        }
    }
    
    
    pub fn caq(ox: &str) -> Self {
        Self {
            ox: String::from(ox),
            gow: [0u8; 32],
            bsd: String::from("!"),
            jcm: 0,
            jga: 0,
            jfg: 99999,
            jwj: 7,
            izv: -1,
            iti: -1,
        }
    }
    
    
    pub fn xrj(&self, aqe: &str) -> bool {
        if self.bsd == "!" {
            return false; 
        }
        let hash = lbn(aqe, &self.bsd);
        rob(&hash, &self.gow)
    }
    
    
    pub fn wjj(&mut self, aqe: &str) {
        self.bsd = nxo(&self.ox);
        self.gow = lbn(aqe, &self.bsd);
    }
    
    
    pub fn zsx(&self) -> String {
        
        let mut obc = String::fc(64);
        for o in &self.gow {
            obc.t(&format!("{:02x}", o));
        }
        format!("{}:{}${}:{}:{}:{}:{}:{}:{}:",
            self.ox,
            obc,
            self.bsd,
            self.jcm,
            self.jga,
            self.jfg,
            self.jwj,
            self.izv,
            self.iti)
    }
    
    
    pub fn yrs(line: &str) -> Option<Self> {
        let ek: Vec<&str> = line.adk(':').collect();
        if ek.len() < 9 {
            return None;
        }
        
        
        let lbo: Vec<&str> = ek[1].adk('$').collect();
        let (hash, bsd) = if lbo.len() >= 2 {
            (tol(lbo[0]), String::from(lbo[1]))
        } else {
            ([0u8; 32], String::from("!"))
        };
        
        Some(Self {
            ox: String::from(ek[0]),
            gow: hash,
            bsd,
            jcm: ek[2].parse().unwrap_or(0),
            jga: ek[3].parse().unwrap_or(0),
            jfg: ek[4].parse().unwrap_or(99999),
            jwj: ek[5].parse().unwrap_or(7),
            izv: ek[6].parse().unwrap_or(-1),
            iti: ek[7].parse().unwrap_or(-1),
        })
    }
}


#[derive(Clone, Debug)]
pub struct GroupEntry {
    pub j: String,
    pub pw: Ln,
    pub jft: Vec<String>,
}

impl GroupEntry {
    pub fn new(j: &str, pw: Ln) -> Self {
        Self {
            j: String::from(j),
            pw,
            jft: Vec::new(),
        }
    }
    
    
    pub fn xij(&self) -> String {
        format!("{}:x:{}:{}", self.j, self.pw, self.jft.rr(","))
    }
    
    
    pub fn yrp(line: &str) -> Option<Self> {
        let ek: Vec<&str> = line.adk(':').collect();
        if ek.len() < 3 {
            return None;
        }
        let j = String::from(ek[0]);
        let pw: u32 = ek[2].parse().bq()?;
        let jft = if ek.len() > 3 && !ek[3].is_empty() {
            ek[3].adk(',').map(|e| String::from(e.em())).collect()
        } else {
            Vec::new()
        };
        Some(Self { j, pw, jft })
    }
}


pub struct Session {
    pub hqi: bool,
    pub pi: Mm,
    pub pw: Ln,
    pub ox: String,
    pub dib: String,
    pub ljt: u64,
}

impl Session {
    pub fn new() -> Self {
        Self {
            hqi: false,
            pi: 0,
            pw: 0,
            ox: String::new(),
            dib: String::from("/"),
            ljt: 0,
        }
    }
    
    pub fn crt(&self) -> bool {
        self.pi == BET_
    }
}


pub struct UserDatabase {
    ddg: BTreeMap<String, UserEntry>,
    fup: BTreeMap<String, ShadowEntry>,
    hlz: BTreeMap<String, GroupEntry>,
    jha: Mm,
    uuf: Ln,
}

impl UserDatabase {
    pub fn new() -> Self {
        let mut ng = Self {
            ddg: BTreeMap::new(),
            fup: BTreeMap::new(),
            hlz: BTreeMap::new(),
            jha: 1000, 
            uuf: 1000,
        };
        
        
        ng.hlz.insert(String::from("root"), GroupEntry::new("root", BES_));
        ng.hlz.insert(String::from("users"), GroupEntry::new("users", AJJ_));
        ng.hlz.insert(String::from("wheel"), GroupEntry::new("wheel", 10)); 
        
        
        let exv = UserEntry::exv();
        let vzz = ShadowEntry::new("root", "toor");
        ng.ddg.insert(String::from("root"), exv);
        ng.fup.insert(String::from("root"), vzz);
        
        
        let cra = UserEntry {
            ox: String::from("guest"),
            pi: 1000,
            pw: AJJ_,
            eqz: String::from("Guest User"),
            dib: String::from("/home/guest"),
            shell: String::from("/bin/tsh"),
        };
        let tid = ShadowEntry::new("guest", "guest");
        ng.ddg.insert(String::from("guest"), cra);
        ng.fup.insert(String::from("guest"), tid);
        ng.jha = 1001;
        
        ng
    }
    
    
    pub fn nyt(&self, ox: &str) -> Option<&UserEntry> {
        self.ddg.get(ox)
    }
    
    
    pub fn yud(&self, pi: Mm) -> Option<&UserEntry> {
        self.ddg.alv().du(|tm| tm.pi == pi)
    }
    
    
    pub fn mwu(&self, ox: &str, aqe: &str) -> bool {
        if let Some(zc) = self.fup.get(ox) {
            zc.xrj(aqe)
        } else {
            false
        }
    }
    
    
    pub fn jzj(&mut self, ox: &str, aqe: &str, hos: bool) -> Result<Mm, &'static str> {
        
        if ox.is_empty() || ox.len() > CFV_ {
            return Err("Invalid username length");
        }
        
        if self.ddg.bgm(ox) {
            return Err("User already exists");
        }
        
        
        if !ox.bw().xx(|r| r.etb() || r == '_' || r == '-') {
            return Err("Invalid characters in username");
        }
        
        let pi = self.jha;
        self.jha += 1;
        
        let pw = if hos { 10 } else { AJJ_ }; 
        
        let cnp = UserEntry::new(ox, pi, pw);
        let zc = ShadowEntry::new(ox, aqe);
        
        self.ddg.insert(String::from(ox), cnp);
        self.fup.insert(String::from(ox), zc);
        
        Ok(pi)
    }
    
    
    pub fn kou(&mut self, ox: &str) -> Result<(), &'static str> {
        if ox == "root" {
            return Err("Cannot delete root user");
        }
        
        if self.ddg.remove(ox).is_none() {
            return Err("User not found");
        }
        
        self.fup.remove(ox);
        Ok(())
    }
    
    
    pub fn khc(&mut self, ox: &str, fov: &str) -> Result<(), &'static str> {
        if let Some(zc) = self.fup.ds(ox) {
            zc.wjj(fov);
            Ok(())
        } else {
            Err("User not found")
        }
    }
    
    
    pub fn liz(&self) -> Vec<&UserEntry> {
        self.ddg.alv().collect()
    }
    
    
    pub fn tci(&self) -> String {
        self.ddg.alv()
            .map(|tm| tm.xim())
            .collect::<Vec<_>>()
            .rr("\n")
    }
    
    
    pub fn tce(&self) -> String {
        self.hlz.alv()
            .map(|at| at.xij())
            .collect::<Vec<_>>()
            .rr("\n")
    }
}


static EX_: Mutex<Option<UserDatabase>> = Mutex::new(None);
static GE_: Mutex<Option<Session>> = Mutex::new(None);
static AZA_: AtomicBool = AtomicBool::new(false);
static Be: AtomicBool = AtomicBool::new(false);


pub fn init() {
    let mut ng = EX_.lock();
    *ng = Some(UserDatabase::new());
    
    let mut he = GE_.lock();
    *he = Some(Session::new());
    
    Be.store(true, Ordering::SeqCst);
    
    crate::log_debug!("[AUTH] Authentication system initialized");
}


pub fn ky() -> bool {
    Be.load(Ordering::SeqCst)
}


pub fn znj(cbj: bool) {
    AZA_.store(cbj, Ordering::SeqCst);
}


pub fn yzo() -> bool {
    AZA_.load(Ordering::SeqCst)
}


pub fn hey() -> String {
    let he = GE_.lock();
    if let Some(ref e) = *he {
        if e.hqi {
            return e.ox.clone();
        }
    }
    String::from("nobody")
}


pub fn kne() -> Mm {
    let he = GE_.lock();
    if let Some(ref e) = *he {
        e.pi
    } else {
        65534 
    }
}


pub fn kmu() -> Ln {
    let he = GE_.lock();
    if let Some(ref e) = *he {
        e.pw
    } else {
        65534 
    }
}


pub fn crt() -> bool {
    let he = GE_.lock();
    if let Some(ref e) = *he {
        e.crt()
    } else {
        false
    }
}


pub fn tyd() -> bool {
    let he = GE_.lock();
    if let Some(ref e) = *he {
        e.hqi
    } else {
        false
    }
}


pub fn ljs(ox: &str, aqe: &str) -> Result<(), &'static str> {
    let ng = EX_.lock();
    let ng = ng.as_ref().ok_or("Auth not initialized")?;
    
    if !ng.mwu(ox, aqe) {
        return Err("Invalid username or password");
    }
    
    let cnp = ng.nyt(ox).ok_or("User not found")?;
    
    drop(ng); 
    
    let mut he = GE_.lock();
    if let Some(ref mut e) = *he {
        e.hqi = true;
        e.pi = cnp.pi;
        e.pw = cnp.pw;
        e.ox = cnp.ox.clone();
        e.dib = cnp.dib.clone();
        e.ljt = crate::time::lc();
    }
    
    Ok(())
}


pub fn oki() {
    let mut he = GE_.lock();
    if let Some(ref mut e) = *he {
        e.hqi = false;
        e.pi = 0;
        e.pw = 0;
        e.ox.clear();
        e.dib = String::from("/");
        e.ljt = 0;
    }
}


pub fn jzj(ox: &str, aqe: &str, hos: bool) -> Result<Mm, &'static str> {
    if !crt() && tyd() {
        return Err("Permission denied: must be root");
    }
    
    let mut ng = EX_.lock();
    let ng = ng.as_mut().ok_or("Auth not initialized")?;
    ng.jzj(ox, aqe, hos)
}


pub fn kou(ox: &str) -> Result<(), &'static str> {
    if !crt() {
        return Err("Permission denied: must be root");
    }
    
    let mut ng = EX_.lock();
    let ng = ng.as_mut().ok_or("Auth not initialized")?;
    ng.kou(ox)
}


pub fn khc(ox: &str, lqa: &str, fov: &str) -> Result<(), &'static str> {
    let ng = EX_.lock();
    let rtu = ng.as_ref().ok_or("Auth not initialized")?;
    
    
    let cv = hey();
    if cv != ox && !crt() {
        return Err("Permission denied");
    }
    
    
    if !crt() && !rtu.mwu(ox, lqa) {
        return Err("Current password incorrect");
    }
    
    drop(ng);
    
    let mut ng = EX_.lock();
    let rtt = ng.as_mut().ok_or("Auth not initialized")?;
    rtt.khc(ox, fov)
}


pub fn liz() -> Vec<(String, Mm, Ln, String)> {
    let ng = EX_.lock();
    if let Some(ref ng) = *ng {
        ng.liz()
            .iter()
            .map(|tm| (tm.ox.clone(), tm.pi, tm.pw, tm.eqz.clone()))
            .collect()
    } else {
        Vec::new()
    }
}


pub fn yth(ox: &str) -> Option<String> {
    let ng = EX_.lock();
    ng.as_ref()?.nyt(ox).map(|tm| tm.dib.clone())
}


pub fn okd() -> bool {
    use crate::framebuffer::{C_, B_, A_, Q_, D_};
    
    crate::println!();
    crate::h!(C_, "╔════════════════════════════════════════╗");
    crate::h!(C_, "║         T-RustOS Login                 ║");
    crate::h!(C_, "╚════════════════════════════════════════╝");
    crate::println!();
    
    let mut ikd = 0;
    const AEU_: u32 = 3;
    
    while ikd < AEU_ {
        
        crate::gr!(B_, "login: ");
        let mut pxt = [0u8; 64];
        let xpx = crate::keyboard::cts(&mut pxt);
        let ox = core::str::jg(&pxt[..xpx])
            .unwrap_or("")
            .em();
        
        if ox.is_empty() {
            continue;
        }
        
        
        crate::gr!(B_, "password: ");
        let mut ewe = [0u8; 128];
        let hun = crate::keyboard::fsf(&mut ewe);
        let aqe = core::str::jg(&ewe[..hun])
            .unwrap_or("")
            .em();
        crate::println!(); 
        
        
        match ljs(ox, aqe) {
            Ok(()) => {
                crate::println!();
                crate::h!(B_, "Welcome, {}!", ox);
                crate::println!();
                return true;
            }
            Err(_) => {
                ikd += 1;
                if ikd < AEU_ {
                    crate::h!(A_, "Login incorrect. {} attempts remaining.", 
                        AEU_ - ikd);
                } else {
                    crate::h!(A_, "Too many failed attempts.");
                }
            }
        }
    }
    
    false
}


pub fn mww() {
    let _ = ljs("root", "toor");
}


pub fn nhc() {
    if !crate::ramfs::ky() {
        return;
    }
    
    
    let ng = EX_.lock();
    if let Some(ref ng) = *ng {
        let vet = ng.tci();
        let thp = ng.tce();
        
        drop(ng);
        
        crate::ramfs::fh(|fs| {
            
            let _ = fs.touch("/etc/passwd");
            let _ = fs.ns("/etc/passwd", vet.as_bytes());
            
            
            let _ = fs.touch("/etc/group");
            let _ = fs.ns("/etc/group", thp.as_bytes());
            
            
            let _ = fs.touch("/etc/shadow");
            let _ = fs.ns("/etc/shadow", b"# Shadow file - passwords hidden\n");
        });
    }
}



pub fn ugw() {
    if !crate::ramfs::ky() {
        return;
    }
    
    let mut our: Option<alloc::vec::Vec<u8>> = None;
    let mut nzu: Option<alloc::vec::Vec<u8>> = None;
    
    crate::ramfs::fh(|fs| {
        if let Ok(f) = fs.mq("/etc/passwd") {
            our = Some(f.ip());
        }
        if let Ok(f) = fs.mq("/etc/group") {
            nzu = Some(f.ip());
        }
    });
    
    let mut ng = EX_.lock();
    if let Some(ref mut ng) = *ng {
        
        if let Some(ref f) = our {
            if let Ok(ca) = core::str::jg(f) {
                let mut diz = 0u32;
                for line in ca.ak() {
                    let line = line.em();
                    if line.is_empty() || line.cj('#') { continue; }
                    if let Some(bt) = UserEntry::syd(line) {
                        if !ng.ddg.bgm(&bt.ox) {
                            let j = bt.ox.clone();
                            ng.ddg.insert(j, bt);
                            diz += 1;
                        }
                    }
                }
                if diz > 0 {
                    crate::log!("[AUTH] Loaded {} users from /etc/passwd", diz);
                }
            }
        }
        
        
        
        
        let _ = nzu; 
    }
}


pub fn zqo() {
    nhc();
    crate::log_debug!("[AUTH] Synced user database to /etc files");
}
