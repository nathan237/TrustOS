




pub mod policy;
pub mod net_proxy;
pub mod fs;
pub mod js_sandbox;
pub mod container;

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use spin::Mutex;

use crate::security;
use crate::security::{CapabilityId, CapabilityType, CapabilityRights};

use self::policy::{SandboxPolicy, PolicyPreset, PolicyVerdict};
use self::net_proxy::{NetProxy, Cs, ProxyError};
use self::fs::SandboxFs;




#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ag(pub u64);


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxState {
    
    Idle,
    
    Active,
    
    Suspended,
    
    Terminated,
}


#[derive(Debug, Clone)]
pub struct ResourceLimits {
    
    pub max_memory_bytes: usize,
    
    pub max_concurrent_requests: usize,
    
    pub max_total_requests: usize,
    
    pub max_response_size: usize,
    
    pub max_fs_files: usize,
    
    pub max_fs_bytes: usize,
    
    pub js_timeout_ms: u64,
    
    pub js_max_stack: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: 4 * 1024 * 1024,       
            max_concurrent_requests: 4,
            max_total_requests: 100,
            max_response_size: 1024 * 1024,           
            max_fs_files: 64,
            max_fs_bytes: 512 * 1024,                 
            js_timeout_ms: 5000,
            js_max_stack: 64,
        }
    }
}


#[derive(Debug, Clone)]
pub struct Eb {
    pub timestamp_ms: u64,
    pub sandbox_id: Ag,
    pub action: AuditAction,
    pub detail: String,
}

#[derive(Debug, Clone)]
pub enum AuditAction {
    Created,
    Navigate,
    NetworkRequest,
    NetworkBlocked,
    JsExecute,
    JsBlocked,
    FsAccess,
    PolicyViolation,
    Destroyed,
}


#[derive(Debug, Clone)]
pub struct SandboxStats {
    pub requests_made: usize,
    pub requests_blocked: usize,
    pub bytes_received: usize,
    pub js_executions: usize,
    pub policy_violations: usize,
    pub fs_files_created: usize,
    pub fs_bytes_used: usize,
}

impl SandboxStats {
    fn new() -> Self {
        Self {
            requests_made: 0,
            requests_blocked: 0,
            bytes_received: 0,
            js_executions: 0,
            policy_violations: 0,
            fs_files_created: 0,
            fs_bytes_used: 0,
        }
    }
}


pub struct Sandbox {
    pub id: Ag,
    pub state: SandboxState,
    
    pub capability: CapabilityId,
    
    pub policy: SandboxPolicy,
    
    pub net_proxy: NetProxy,
    
    pub filesystem: SandboxFs,
    
    pub current_url: Option<String>,
    
    pub limits: ResourceLimits,
    
    pub stats: SandboxStats,
    
    pub page_cache: BTreeMap<String, Vec<u8>>,
    
    pub label: String,
}

impl Sandbox {
    fn new(id: Ag, capability: CapabilityId, policy: SandboxPolicy, limits: ResourceLimits, label: String) -> Self {
        let net_proxy = NetProxy::new(id, &policy);
        let filesystem = SandboxFs::new(id, limits.max_fs_files, limits.max_fs_bytes);
        Self {
            id,
            state: SandboxState::Idle,
            capability,
            policy,
            net_proxy,
            filesystem,
            current_url: None,
            limits,
            stats: SandboxStats::new(),
            page_cache: BTreeMap::new(),
            label,
        }
    }
}




pub struct SandboxManager {
    sandboxes: BTreeMap<u64, Sandbox>,
    next_id: u64,
    
    cap_type_id: Option<u32>,
    
    master_cap: Option<CapabilityId>,
    
    audit_log: Vec<Eb>,
}

impl SandboxManager {
    pub fn new() -> Self {
        Self {
            sandboxes: BTreeMap::new(),
            next_id: 1,
            cap_type_id: None,
            master_cap: None,
            audit_log: Vec::new(),
        }
    }

    
    pub fn register_capability_type(&mut self) {
        let joy = security::izk(
            "WebSandbox",
            3, 
            "Security",
            "Web sandbox isolation — controls network, JS, filesystem access for untrusted web content"
        );
        self.cap_type_id = Some(joy);

        
        let etw = security::fpa(
            CapabilityType::Dynamic(joy),
            CapabilityRights::Ba
                .union(CapabilityRights::Bh)
                .union(CapabilityRights::Jq)
                .union(CapabilityRights::Fj),
            0, 
        );
        self.master_cap = Some(etw);
    }

    
    pub fn create_sandbox(&mut self, preset: PolicyPreset, label: Option<&str>) -> Ag {
        let id = Ag(self.next_id);
        self.next_id += 1;

        
        let capability = if let Some(etw) = self.master_cap {
            security::derive(
                etw,
                CapabilityRights::Ba.union(CapabilityRights::Bh),
                id.0, 
            ).unwrap_or(etw)
        } else {
            CapabilityId(0)
        };

        let policy = SandboxPolicy::lzo(preset);
        let limits = ResourceLimits::default();
        let cmb = label.unwrap_or("sandbox").into();
        let sandbox = Sandbox::new(id, capability, policy, limits, cmb);
        self.sandboxes.insert(id.0, sandbox);

        self.audit(id, AuditAction::Created, format!("preset={:?}", preset));
        id
    }

    
    pub fn navigate(&mut self, id: Ag, url: &str) -> Result<Cs, SandboxError> {
        
        {
            let sandbox = self.sandboxes.get(&id.0)
                .ok_or(SandboxError::NotFound)?;
            if sandbox.state == SandboxState::Terminated {
                return Err(SandboxError::Terminated);
            }
            if sandbox.state == SandboxState::Suspended {
                return Err(SandboxError::Suspended);
            }
            if sandbox.stats.requests_made >= sandbox.limits.max_total_requests {
                
            }
        }

        
        let sandbox = self.sandboxes.get_mut(&id.0)
            .ok_or(SandboxError::NotFound)?;

        if sandbox.stats.requests_made >= sandbox.limits.max_total_requests {
            sandbox.stats.policy_violations += 1;
            return Err(SandboxError::RequestLimitExceeded);
        }

        let edq = sandbox.policy.evaluate_url(url);
        match edq {
            PolicyVerdict::Allow => {},
            PolicyVerdict::Deny(azg) => {
                sandbox.stats.requests_blocked += 1;
                sandbox.stats.policy_violations += 1;
                return Err(SandboxError::PolicyDenied(azg));
            },
            PolicyVerdict::Log => {},
        }

        
        sandbox.stats.requests_made += 1;
        let max_size = sandbox.limits.max_response_size;
        let fa = sandbox.net_proxy.fetch(url, max_size)?;

        sandbox.stats.bytes_received += fa.body.len();
        sandbox.current_url = Some(url.into());
        sandbox.state = SandboxState::Active;
        sandbox.page_cache.insert(url.into(), fa.body.clone());

        
        self.audit(id, AuditAction::Navigate, url.into());

        Ok(fa)
    }

    
    pub fn fetch_resource(&mut self, id: Ag, url: &str) -> Result<Cs, SandboxError> {
        let sandbox = self.sandboxes.get_mut(&id.0)
            .ok_or(SandboxError::NotFound)?;

        if sandbox.state == SandboxState::Terminated {
            return Err(SandboxError::Terminated);
        }

        
        if let Some(bfd) = sandbox.page_cache.get(url) {
            return Ok(Cs {
                status_code: 200,
                content_type: String::from("text/html"),
                body: bfd.clone(),
                headers: Vec::new(),
                was_cached: true,
            });
        }

        if sandbox.stats.requests_made >= sandbox.limits.max_total_requests {
            sandbox.stats.policy_violations += 1;
            return Err(SandboxError::RequestLimitExceeded);
        }

        let edq = sandbox.policy.evaluate_url(url);
        match edq {
            PolicyVerdict::Allow | PolicyVerdict::Log => {},
            PolicyVerdict::Deny(azg) => {
                sandbox.stats.requests_blocked += 1;
                return Err(SandboxError::PolicyDenied(azg));
            },
        }

        sandbox.stats.requests_made += 1;
        let max_size = sandbox.limits.max_response_size;
        let fa = sandbox.net_proxy.fetch(url, max_size)?;
        sandbox.stats.bytes_received += fa.body.len();
        sandbox.page_cache.insert(url.into(), fa.body.clone());
        Ok(fa)
    }

    
    pub fn crf(&mut self, id: Ag) -> Result<(), SandboxError> {
        let sandbox = self.sandboxes.get_mut(&id.0)
            .ok_or(SandboxError::NotFound)?;
        sandbox.state = SandboxState::Suspended;
        Ok(())
    }

    
    pub fn resume(&mut self, id: Ag) -> Result<(), SandboxError> {
        let sandbox = self.sandboxes.get_mut(&id.0)
            .ok_or(SandboxError::NotFound)?;
        if sandbox.state == SandboxState::Suspended {
            sandbox.state = SandboxState::Active;
        }
        Ok(())
    }

    
    pub fn destroy(&mut self, id: Ag) -> Result<(), SandboxError> {
        if let Some(sandbox) = self.sandboxes.get_mut(&id.0) {
            sandbox.state = SandboxState::Terminated;
            security::ogu(sandbox.capability);
            self.audit(id, AuditAction::Destroyed, String::new());
        }
        self.sandboxes.remove(&id.0);
        Ok(())
    }

    
    pub fn get(&self, id: Ag) -> Option<&Sandbox> {
        self.sandboxes.get(&id.0)
    }

    
    pub fn get_mut(&mut self, id: Ag) -> Option<&mut Sandbox> {
        self.sandboxes.get_mut(&id.0)
    }

    
    pub fn list(&self) -> Vec<(Ag, &str, SandboxState)> {
        self.sandboxes.values()
            .map(|j| (j.id, j.label.as_str(), j.state))
            .collect()
    }

    
    pub fn count(&self) -> usize {
        self.sandboxes.len()
    }

    
    fn audit(&mut self, id: Ag, action: AuditAction, detail: String) {
        let jy = crate::time::uptime_ms();
        self.audit_log.push(Eb {
            timestamp_ms: jy,
            sandbox_id: id,
            action,
            detail,
        });
        
        if self.audit_log.len() > 256 {
            self.audit_log.remove(0);
        }
    }

    
    pub fn audit_log(&self) -> &[Eb] {
        &self.audit_log
    }

    
    pub fn audit_for(&self, id: Ag) -> Vec<&Eb> {
        self.audit_log.iter().filter(|e| e.sandbox_id == id).collect()
    }
}



#[derive(Debug)]
pub enum SandboxError {
    NotFound,
    Terminated,
    Suspended,
    PolicyDenied(String),
    RequestLimitExceeded,
    NetworkError(String),
    FsQuotaExceeded,
    FsNotFound,
    JsTimeout,
    JsStackOverflow,
    CapabilityDenied,
}

impl From<ProxyError> for SandboxError {
    fn from(e: ProxyError) -> Self {
        match e {
            ProxyError::DomainBlocked(d) => SandboxError::PolicyDenied(d),
            ProxyError::RateLimited => SandboxError::RequestLimitExceeded,
            ProxyError::ResponseTooLarge => SandboxError::PolicyDenied(String::from("response too large")),
            ProxyError::NetworkError(j) => SandboxError::NetworkError(j),
            ProxyError::DnsError => SandboxError::NetworkError(String::from("DNS resolution failed")),
            ProxyError::TlsError => SandboxError::NetworkError(String::from("TLS handshake failed")),
            ProxyError::Timeout => SandboxError::NetworkError(String::from("request timeout")),
            ProxyError::InvalidUrl => SandboxError::PolicyDenied(String::from("invalid URL")),
        }
    }
}



lazy_static::lazy_static! {
    pub static ref BE_: Mutex<SandboxManager> = Mutex::new(SandboxManager::new());
}


pub fn init() {
    let mut ng = BE_.lock();
    ng.register_capability_type();
    crate::serial_println!("[sandbox] Web Sandbox subsystem initialized");
    crate::serial_println!("[sandbox] Capability type registered: WebSandbox (danger=3)");
}


pub fn create(preset: PolicyPreset, label: Option<&str>) -> Ag {
    BE_.lock().create_sandbox(preset, label)
}


pub fn navigate(id: Ag, url: &str) -> Result<Cs, SandboxError> {
    BE_.lock().navigate(id, url)
}


pub fn fetch_resource(id: Ag, url: &str) -> Result<Cs, SandboxError> {
    BE_.lock().fetch_resource(id, url)
}


pub fn destroy(id: Ag) -> Result<(), SandboxError> {
    BE_.lock().destroy(id)
}


pub fn list() -> Vec<(Ag, String, SandboxState)> {
    BE_.lock().list().into_iter()
        .map(|(id, j, uz)| (id, String::from(j), uz))
        .collect()
}


pub fn status_string(id: Ag) -> Option<String> {
    let ng = BE_.lock();
    let cv = ng.get(id)?;
    Some(format!(
        "Sandbox #{} '{}'\n  State: {:?}\n  URL: {}\n  Requests: {}/{} ({} blocked)\n  Data: {} bytes\n  JS execs: {}\n  Violations: {}\n  FS: {} files, {} bytes\n  Policy: {:?}",
        cv.id.0, cv.label, cv.state,
        cv.current_url.as_deref().unwrap_or("(none)"),
        cv.stats.requests_made, cv.limits.max_total_requests, cv.stats.requests_blocked,
        cv.stats.bytes_received,
        cv.stats.js_executions,
        cv.stats.policy_violations,
        cv.stats.fs_files_created, cv.stats.fs_bytes_used,
        cv.policy.preset,
    ))
}
