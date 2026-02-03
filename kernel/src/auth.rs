//! Authentication System for TrustOS
//!
//! Implements user authentication with passwd/shadow files,
//! login/logout, and user management commands.

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::format;
use alloc::collections::BTreeMap;
use spin::Mutex;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

/// User ID type
pub type Uid = u32;
/// Group ID type  
pub type Gid = u32;

/// Root user ID
pub const ROOT_UID: Uid = 0;
/// Root group ID
pub const ROOT_GID: Gid = 0;
/// Default user group ID
pub const USERS_GID: Gid = 100;

/// Maximum username length
pub const MAX_USERNAME_LEN: usize = 32;
/// Maximum password length
pub const MAX_PASSWORD_LEN: usize = 128;

/// Simple hash for passwords (in a real OS, use bcrypt/argon2)
/// This is a basic FNV-1a hash - NOT cryptographically secure!
fn hash_password(password: &str, salt: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325; // FNV offset basis
    
    // Hash salt first
    for byte in salt.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3); // FNV prime
    }
    
    // Then password
    for byte in password.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    
    hash
}

/// Generate a simple salt from username and a counter
fn generate_salt(username: &str) -> String {
    static SALT_COUNTER: AtomicU32 = AtomicU32::new(0);
    let counter = SALT_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{}${}", username, counter)
}

/// User entry (like /etc/passwd)
#[derive(Clone, Debug)]
pub struct UserEntry {
    pub username: String,
    pub uid: Uid,
    pub gid: Gid,
    pub gecos: String,       // Full name / description
    pub home_dir: String,
    pub shell: String,
}

impl UserEntry {
    /// Create a new user entry
    pub fn new(username: &str, uid: Uid, gid: Gid) -> Self {
        Self {
            username: String::from(username),
            uid,
            gid,
            gecos: String::new(),
            home_dir: format!("/home/{}", username),
            shell: String::from("/bin/tsh"),
        }
    }
    
    /// Create root user entry
    pub fn root() -> Self {
        Self {
            username: String::from("root"),
            uid: ROOT_UID,
            gid: ROOT_GID,
            gecos: String::from("System Administrator"),
            home_dir: String::from("/root"),
            shell: String::from("/bin/tsh"),
        }
    }
    
    /// Format as passwd line: username:x:uid:gid:gecos:home:shell
    pub fn to_passwd_line(&self) -> String {
        format!("{}:x:{}:{}:{}:{}:{}",
            self.username, self.uid, self.gid,
            self.gecos, self.home_dir, self.shell)
    }
    
    /// Parse from passwd line
    pub fn from_passwd_line(line: &str) -> Option<Self> {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() < 7 {
            return None;
        }
        
        Some(Self {
            username: String::from(parts[0]),
            uid: parts[2].parse().ok()?,
            gid: parts[3].parse().ok()?,
            gecos: String::from(parts[4]),
            home_dir: String::from(parts[5]),
            shell: String::from(parts[6]),
        })
    }
}

/// Shadow entry (like /etc/shadow)
#[derive(Clone, Debug)]
pub struct ShadowEntry {
    pub username: String,
    pub password_hash: u64,
    pub salt: String,
    pub last_changed: u64,    // Days since epoch
    pub min_days: u32,        // Min days before change
    pub max_days: u32,        // Max days before must change
    pub warn_days: u32,       // Days before expiry to warn
    pub inactive_days: i32,   // Days after expiry until account disabled (-1 = never)
    pub expire_date: i64,     // Date account expires (-1 = never)
}

impl ShadowEntry {
    /// Create a new shadow entry with password
    pub fn new(username: &str, password: &str) -> Self {
        let salt = generate_salt(username);
        let hash = hash_password(password, &salt);
        
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
    
    /// Create entry with no password (locked)
    pub fn locked(username: &str) -> Self {
        Self {
            username: String::from(username),
            password_hash: 0,
            salt: String::from("!"),
            last_changed: 0,
            min_days: 0,
            max_days: 99999,
            warn_days: 7,
            inactive_days: -1,
            expire_date: -1,
        }
    }
    
    /// Check if password matches
    pub fn verify_password(&self, password: &str) -> bool {
        if self.salt == "!" {
            return false; // Account locked
        }
        let hash = hash_password(password, &self.salt);
        hash == self.password_hash
    }
    
    /// Update password
    pub fn set_password(&mut self, password: &str) {
        self.salt = generate_salt(&self.username);
        self.password_hash = hash_password(password, &self.salt);
    }
    
    /// Format as shadow line
    pub fn to_shadow_line(&self) -> String {
        format!("{}:{}${}:{}:{}:{}:{}:{}:{}:",
            self.username,
            self.password_hash,
            self.salt,
            self.last_changed,
            self.min_days,
            self.max_days,
            self.warn_days,
            self.inactive_days,
            self.expire_date)
    }
    
    /// Parse from shadow line
    pub fn from_shadow_line(line: &str) -> Option<Self> {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() < 9 {
            return None;
        }
        
        // Parse hash$salt
        let hash_salt: Vec<&str> = parts[1].split('$').collect();
        let (hash, salt) = if hash_salt.len() >= 2 {
            (hash_salt[0].parse().unwrap_or(0), String::from(hash_salt[1]))
        } else {
            (0, String::from("!"))
        };
        
        Some(Self {
            username: String::from(parts[0]),
            password_hash: hash,
            salt,
            last_changed: parts[2].parse().unwrap_or(0),
            min_days: parts[3].parse().unwrap_or(0),
            max_days: parts[4].parse().unwrap_or(99999),
            warn_days: parts[5].parse().unwrap_or(7),
            inactive_days: parts[6].parse().unwrap_or(-1),
            expire_date: parts[7].parse().unwrap_or(-1),
        })
    }
}

/// Group entry (like /etc/group)
#[derive(Clone, Debug)]
pub struct GroupEntry {
    pub name: String,
    pub gid: Gid,
    pub members: Vec<String>,
}

impl GroupEntry {
    pub fn new(name: &str, gid: Gid) -> Self {
        Self {
            name: String::from(name),
            gid,
            members: Vec::new(),
        }
    }
    
    /// Format as group line: name:x:gid:member1,member2,...
    pub fn to_group_line(&self) -> String {
        format!("{}:x:{}:{}", self.name, self.gid, self.members.join(","))
    }
}

/// Current session state
pub struct Session {
    pub logged_in: bool,
    pub uid: Uid,
    pub gid: Gid,
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
        self.uid == ROOT_UID
    }
}

/// User database
pub struct UserDatabase {
    users: BTreeMap<String, UserEntry>,
    shadows: BTreeMap<String, ShadowEntry>,
    groups: BTreeMap<String, GroupEntry>,
    next_uid: Uid,
    next_gid: Gid,
}

impl UserDatabase {
    pub fn new() -> Self {
        let mut db = Self {
            users: BTreeMap::new(),
            shadows: BTreeMap::new(),
            groups: BTreeMap::new(),
            next_uid: 1000, // Regular users start at 1000
            next_gid: 1000,
        };
        
        // Create default groups
        db.groups.insert(String::from("root"), GroupEntry::new("root", ROOT_GID));
        db.groups.insert(String::from("users"), GroupEntry::new("users", USERS_GID));
        db.groups.insert(String::from("wheel"), GroupEntry::new("wheel", 10)); // sudo group
        
        // Create root user with default password "toor"
        let root = UserEntry::root();
        let root_shadow = ShadowEntry::new("root", "toor");
        db.users.insert(String::from("root"), root);
        db.shadows.insert(String::from("root"), root_shadow);
        
        // Create guest user with password "guest"
        let guest = UserEntry {
            username: String::from("guest"),
            uid: 1000,
            gid: USERS_GID,
            gecos: String::from("Guest User"),
            home_dir: String::from("/home/guest"),
            shell: String::from("/bin/tsh"),
        };
        let guest_shadow = ShadowEntry::new("guest", "guest");
        db.users.insert(String::from("guest"), guest);
        db.shadows.insert(String::from("guest"), guest_shadow);
        db.next_uid = 1001;
        
        db
    }
    
    /// Get user by username
    pub fn get_user(&self, username: &str) -> Option<&UserEntry> {
        self.users.get(username)
    }
    
    /// Get user by UID
    pub fn get_user_by_uid(&self, uid: Uid) -> Option<&UserEntry> {
        self.users.values().find(|u| u.uid == uid)
    }
    
    /// Authenticate user
    pub fn authenticate(&self, username: &str, password: &str) -> bool {
        if let Some(shadow) = self.shadows.get(username) {
            shadow.verify_password(password)
        } else {
            false
        }
    }
    
    /// Add a new user
    pub fn add_user(&mut self, username: &str, password: &str, is_admin: bool) -> Result<Uid, &'static str> {
        // Validate username
        if username.is_empty() || username.len() > MAX_USERNAME_LEN {
            return Err("Invalid username length");
        }
        
        if self.users.contains_key(username) {
            return Err("User already exists");
        }
        
        // Check for invalid characters
        if !username.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err("Invalid characters in username");
        }
        
        let uid = self.next_uid;
        self.next_uid += 1;
        
        let gid = if is_admin { 10 } else { USERS_GID }; // wheel or users
        
        let user = UserEntry::new(username, uid, gid);
        let shadow = ShadowEntry::new(username, password);
        
        self.users.insert(String::from(username), user);
        self.shadows.insert(String::from(username), shadow);
        
        Ok(uid)
    }
    
    /// Delete a user
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
    
    /// Change user password
    pub fn change_password(&mut self, username: &str, new_password: &str) -> Result<(), &'static str> {
        if let Some(shadow) = self.shadows.get_mut(username) {
            shadow.set_password(new_password);
            Ok(())
        } else {
            Err("User not found")
        }
    }
    
    /// List all users
    pub fn list_users(&self) -> Vec<&UserEntry> {
        self.users.values().collect()
    }
    
    /// Generate /etc/passwd content
    pub fn generate_passwd(&self) -> String {
        self.users.values()
            .map(|u| u.to_passwd_line())
            .collect::<Vec<_>>()
            .join("\n")
    }
    
    /// Generate /etc/group content
    pub fn generate_group(&self) -> String {
        self.groups.values()
            .map(|g| g.to_group_line())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

// Global state
static USER_DB: Mutex<Option<UserDatabase>> = Mutex::new(None);
static CURRENT_SESSION: Mutex<Option<Session>> = Mutex::new(None);
static LOGIN_REQUIRED: AtomicBool = AtomicBool::new(false);
static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Initialize the authentication system
pub fn init() {
    let mut db = USER_DB.lock();
    *db = Some(UserDatabase::new());
    
    let mut session = CURRENT_SESSION.lock();
    *session = Some(Session::new());
    
    INITIALIZED.store(true, Ordering::SeqCst);
    
    crate::log_debug!("[AUTH] Authentication system initialized");
}

/// Check if auth is initialized
pub fn is_initialized() -> bool {
    INITIALIZED.load(Ordering::SeqCst)
}

/// Enable login requirement
pub fn set_login_required(required: bool) {
    LOGIN_REQUIRED.store(required, Ordering::SeqCst);
}

/// Check if login is required
pub fn is_login_required() -> bool {
    LOGIN_REQUIRED.load(Ordering::SeqCst)
}

/// Get current username
pub fn current_user() -> String {
    let session = CURRENT_SESSION.lock();
    if let Some(ref s) = *session {
        if s.logged_in {
            return s.username.clone();
        }
    }
    String::from("nobody")
}

/// Get current UID
pub fn current_uid() -> Uid {
    let session = CURRENT_SESSION.lock();
    if let Some(ref s) = *session {
        s.uid
    } else {
        65534 // nobody
    }
}

/// Get current GID
pub fn current_gid() -> Gid {
    let session = CURRENT_SESSION.lock();
    if let Some(ref s) = *session {
        s.gid
    } else {
        65534 // nogroup
    }
}

/// Check if current user is root
pub fn is_root() -> bool {
    let session = CURRENT_SESSION.lock();
    if let Some(ref s) = *session {
        s.is_root()
    } else {
        false
    }
}

/// Check if logged in
pub fn is_logged_in() -> bool {
    let session = CURRENT_SESSION.lock();
    if let Some(ref s) = *session {
        s.logged_in
    } else {
        false
    }
}

/// Attempt login
pub fn login(username: &str, password: &str) -> Result<(), &'static str> {
    let db = USER_DB.lock();
    let db = db.as_ref().ok_or("Auth not initialized")?;
    
    if !db.authenticate(username, password) {
        return Err("Invalid username or password");
    }
    
    let user = db.get_user(username).ok_or("User not found")?;
    
    drop(db); // Release lock before acquiring session lock
    
    let mut session = CURRENT_SESSION.lock();
    if let Some(ref mut s) = *session {
        s.logged_in = true;
        s.uid = user.uid;
        s.gid = user.gid;
        s.username = user.username.clone();
        s.home_dir = user.home_dir.clone();
        s.login_time = crate::time::uptime_ms();
    }
    
    Ok(())
}

/// Logout current user
pub fn logout() {
    let mut session = CURRENT_SESSION.lock();
    if let Some(ref mut s) = *session {
        s.logged_in = false;
        s.uid = 0;
        s.gid = 0;
        s.username.clear();
        s.home_dir = String::from("/");
        s.login_time = 0;
    }
}

/// Add a new user (requires root)
pub fn add_user(username: &str, password: &str, is_admin: bool) -> Result<Uid, &'static str> {
    if !is_root() && is_logged_in() {
        return Err("Permission denied: must be root");
    }
    
    let mut db = USER_DB.lock();
    let db = db.as_mut().ok_or("Auth not initialized")?;
    db.add_user(username, password, is_admin)
}

/// Delete a user (requires root)
pub fn delete_user(username: &str) -> Result<(), &'static str> {
    if !is_root() {
        return Err("Permission denied: must be root");
    }
    
    let mut db = USER_DB.lock();
    let db = db.as_mut().ok_or("Auth not initialized")?;
    db.delete_user(username)
}

/// Change password
pub fn change_password(username: &str, old_password: &str, new_password: &str) -> Result<(), &'static str> {
    let db = USER_DB.lock();
    let db_ref = db.as_ref().ok_or("Auth not initialized")?;
    
    // Users can only change their own password (unless root)
    let current = current_user();
    if current != username && !is_root() {
        return Err("Permission denied");
    }
    
    // Verify old password (unless root)
    if !is_root() && !db_ref.authenticate(username, old_password) {
        return Err("Current password incorrect");
    }
    
    drop(db);
    
    let mut db = USER_DB.lock();
    let db_mut = db.as_mut().ok_or("Auth not initialized")?;
    db_mut.change_password(username, new_password)
}

/// List all users
pub fn list_users() -> Vec<(String, Uid, Gid, String)> {
    let db = USER_DB.lock();
    if let Some(ref db) = *db {
        db.list_users()
            .iter()
            .map(|u| (u.username.clone(), u.uid, u.gid, u.gecos.clone()))
            .collect()
    } else {
        Vec::new()
    }
}

/// Get home directory for user
pub fn get_home_dir(username: &str) -> Option<String> {
    let db = USER_DB.lock();
    db.as_ref()?.get_user(username).map(|u| u.home_dir.clone())
}

/// Display login prompt and handle authentication
pub fn login_prompt() -> bool {
    use crate::framebuffer::{COLOR_CYAN, COLOR_GREEN, COLOR_RED, COLOR_WHITE, COLOR_YELLOW};
    
    crate::println!();
    crate::println_color!(COLOR_CYAN, "╔════════════════════════════════════════╗");
    crate::println_color!(COLOR_CYAN, "║         T-RustOS Login                 ║");
    crate::println_color!(COLOR_CYAN, "╚════════════════════════════════════════╝");
    crate::println!();
    
    let mut attempts = 0;
    const MAX_ATTEMPTS: u32 = 3;
    
    while attempts < MAX_ATTEMPTS {
        // Get username
        crate::print_color!(COLOR_GREEN, "login: ");
        let mut username_buf = [0u8; 64];
        let username_len = crate::keyboard::read_line(&mut username_buf);
        let username = core::str::from_utf8(&username_buf[..username_len])
            .unwrap_or("")
            .trim();
        
        if username.is_empty() {
            continue;
        }
        
        // Get password (hidden input)
        crate::print_color!(COLOR_GREEN, "password: ");
        let mut password_buf = [0u8; 128];
        let password_len = crate::keyboard::read_line_hidden(&mut password_buf);
        let password = core::str::from_utf8(&password_buf[..password_len])
            .unwrap_or("")
            .trim();
        crate::println!(); // New line after hidden input
        
        // Attempt login
        match login(username, password) {
            Ok(()) => {
                crate::println!();
                crate::println_color!(COLOR_GREEN, "Welcome, {}!", username);
                crate::println!();
                return true;
            }
            Err(_) => {
                attempts += 1;
                if attempts < MAX_ATTEMPTS {
                    crate::println_color!(COLOR_RED, "Login incorrect. {} attempts remaining.", 
                        MAX_ATTEMPTS - attempts);
                } else {
                    crate::println_color!(COLOR_RED, "Too many failed attempts.");
                }
            }
        }
    }
    
    false
}

/// Auto-login as root (for development/recovery)
pub fn auto_login_root() {
    let _ = login("root", "toor");
}

/// Create /etc files in the filesystem
pub fn create_etc_files() {
    if !crate::ramfs::is_initialized() {
        return;
    }
    
    // Generate and write passwd
    let db = USER_DB.lock();
    if let Some(ref db) = *db {
        let passwd_content = db.generate_passwd();
        let group_content = db.generate_group();
        
        drop(db);
        
        crate::ramfs::with_fs(|fs| {
            // Create passwd
            let _ = fs.touch("/etc/passwd");
            let _ = fs.write_file("/etc/passwd", passwd_content.as_bytes());
            
            // Create group  
            let _ = fs.touch("/etc/group");
            let _ = fs.write_file("/etc/group", group_content.as_bytes());
            
            // Create shadow (permissions should be root only in real OS)
            let _ = fs.touch("/etc/shadow");
            let _ = fs.write_file("/etc/shadow", b"# Shadow file - passwords hidden\n");
        });
    }
}
