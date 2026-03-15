// sandbox/mod.rs — TrustOS Web Sandbox
// Kernel-level sandboxed execution environment for web content
// Zero-trust architecture: all network requests proxied through kernel,
// capability-gated, domain-filtered, rate-limited, content-inspected.

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
use self::net_proxy::{NetProxy, ProxiedResponse, ProxyError};
use self::fs::SandboxFs;

// ──── Types ────────────────────────────────────────────────────────────────

/// Unique sandbox instance identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SandboxId(pub u64);

/// Sandbox lifecycle states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxState {
    /// Created but not yet navigated
    Idle,
    /// Actively processing a page
    Active,
    /// Suspended (paused, no new requests)
    Suspended,
    /// Terminated (cleanup pending)
    Terminated,
}

/// Resource limits for a sandbox instance
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    /// Max memory in bytes (default 4 MB)
    pub max_memory_bytes: usize,
    /// Max concurrent network requests
    pub max_concurrent_requests: usize,
    /// Max total requests per session
    pub max_total_requests: usize,
    /// Max response body size in bytes (default 1 MB)
    pub max_response_size: usize,
    /// Max files in sandbox filesystem
    pub max_fs_files: usize,
    /// Max total filesystem storage in bytes (default 512 KB)
    pub max_fs_bytes: usize,
    /// JS execution time limit in ms (0 = unlimited)
    pub js_timeout_ms: u64,
    /// JS max stack depth
    pub js_max_stack: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: 4 * 1024 * 1024,       // 4 MB
            max_concurrent_requests: 4,
            max_total_requests: 100,
            max_response_size: 1024 * 1024,           // 1 MB
            max_fs_files: 64,
            max_fs_bytes: 512 * 1024,                 // 512 KB
            js_timeout_ms: 5000,
            js_max_stack: 64,
        }
    }
}

/// Audit log entry
#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub timestamp_ms: u64,
    pub sandbox_id: SandboxId,
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

/// Statistics for a sandbox instance
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

/// A single sandbox instance — isolated web execution environment
pub struct Sandbox {
    pub id: SandboxId,
    pub state: SandboxState,
    /// Capability token for this sandbox (derived, non-escalable)
    pub capability: CapabilityId,
    /// Network policy (domain allow/deny, rate limit)
    pub policy: SandboxPolicy,
    /// Kernel-controlled network proxy
    pub net_proxy: NetProxy,
    /// Jailed virtual filesystem
    pub filesystem: SandboxFs,
    /// Current navigated URL (if any)
    pub current_url: Option<String>,
    /// Resource limits
    pub limits: ResourceLimits,
    /// Runtime statistics
    pub stats: SandboxStats,
    /// Page content cache (URL → response body)
    pub page_cache: BTreeMap<String, Vec<u8>>,
    /// Session label
    pub label: String,
}

impl Sandbox {
    fn new(id: SandboxId, capability: CapabilityId, policy: SandboxPolicy, limits: ResourceLimits, label: String) -> Self {
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

// ──── Global Manager ───────────────────────────────────────────────────────

/// Global sandbox manager
pub struct SandboxManager {
    sandboxes: BTreeMap<u64, Sandbox>,
    next_id: u64,
    /// Dynamic capability type ID for WebSandbox
    cap_type_id: Option<u32>,
    /// Master capability (parent for all sandbox capabilities)
    master_cap: Option<CapabilityId>,
    /// Global audit log (ring buffer, max 256 entries)
    audit_log: Vec<AuditEntry>,
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

    /// Register the WebSandbox dynamic capability type
    pub fn register_capability_type(&mut self) {
        let type_id = security::register_dynamic_type(
            "WebSandbox",
            3, // danger level: elevated (network access)
            "Security",
            "Web sandbox isolation — controls network, JS, filesystem access for untrusted web content"
        );
        self.cap_type_id = Some(type_id);

        // Create master capability with full rights (kernel-only)
        let master = security::create_capability(
            CapabilityType::Dynamic(type_id),
            CapabilityRights::READ
                .union(CapabilityRights::WRITE)
                .union(CapabilityRights::CREATE)
                .union(CapabilityRights::CONTROL),
            0, // kernel owner
        );
        self.master_cap = Some(master);
    }

    /// Create a new sandbox with the given preset and optional custom label
    pub fn create_sandbox(&mut self, preset: PolicyPreset, label: Option<&str>) -> SandboxId {
        let id = SandboxId(self.next_id);
        self.next_id += 1;

        // Derive a restricted capability from master (read + write only, no control)
        let capability = if let Some(master) = self.master_cap {
            security::derive(
                master,
                CapabilityRights::READ.union(CapabilityRights::WRITE),
                id.0, // sandbox owns its own cap
            ).unwrap_or(master)
        } else {
            CapabilityId(0)
        };

        let policy = SandboxPolicy::from_preset(preset);
        let limits = ResourceLimits::default();
        let lbl = label.unwrap_or("sandbox").into();
        let sandbox = Sandbox::new(id, capability, policy, limits, lbl);
        self.sandboxes.insert(id.0, sandbox);

        self.audit(id, AuditAction::Created, format!("preset={:?}", preset));
        id
    }

    /// Navigate a sandbox to a URL — all requests go through kernel proxy
    pub fn navigate(&mut self, id: SandboxId, url: &str) -> Result<ProxiedResponse, SandboxError> {
        // Phase 1: Validate state and policy (immutable checks first)
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
                // Need mutable access for stats update
            }
        }

        // Phase 2: Check limits and policy with mutable access
        let sandbox = self.sandboxes.get_mut(&id.0)
            .ok_or(SandboxError::NotFound)?;

        if sandbox.stats.requests_made >= sandbox.limits.max_total_requests {
            sandbox.stats.policy_violations += 1;
            return Err(SandboxError::RequestLimitExceeded);
        }

        let verdict = sandbox.policy.evaluate_url(url);
        match verdict {
            PolicyVerdict::Allow => {},
            PolicyVerdict::Deny(reason) => {
                sandbox.stats.requests_blocked += 1;
                sandbox.stats.policy_violations += 1;
                return Err(SandboxError::PolicyDenied(reason));
            },
            PolicyVerdict::Log => {},
        }

        // Phase 3: Fetch through proxy
        sandbox.stats.requests_made += 1;
        let max_size = sandbox.limits.max_response_size;
        let response = sandbox.net_proxy.fetch(url, max_size)?;

        sandbox.stats.bytes_received += response.body.len();
        sandbox.current_url = Some(url.into());
        sandbox.state = SandboxState::Active;
        sandbox.page_cache.insert(url.into(), response.body.clone());

        // Phase 4: Audit (after releasing sandbox borrow)
        self.audit(id, AuditAction::Navigate, url.into());

        Ok(response)
    }

    /// Fetch a sub-resource (CSS, image, etc.) within a sandbox
    pub fn fetch_resource(&mut self, id: SandboxId, url: &str) -> Result<ProxiedResponse, SandboxError> {
        let sandbox = self.sandboxes.get_mut(&id.0)
            .ok_or(SandboxError::NotFound)?;

        if sandbox.state == SandboxState::Terminated {
            return Err(SandboxError::Terminated);
        }

        // Check cache first
        if let Some(cached) = sandbox.page_cache.get(url) {
            return Ok(ProxiedResponse {
                status_code: 200,
                content_type: String::from("text/html"),
                body: cached.clone(),
                headers: Vec::new(),
                was_cached: true,
            });
        }

        if sandbox.stats.requests_made >= sandbox.limits.max_total_requests {
            sandbox.stats.policy_violations += 1;
            return Err(SandboxError::RequestLimitExceeded);
        }

        let verdict = sandbox.policy.evaluate_url(url);
        match verdict {
            PolicyVerdict::Allow | PolicyVerdict::Log => {},
            PolicyVerdict::Deny(reason) => {
                sandbox.stats.requests_blocked += 1;
                return Err(SandboxError::PolicyDenied(reason));
            },
        }

        sandbox.stats.requests_made += 1;
        let max_size = sandbox.limits.max_response_size;
        let response = sandbox.net_proxy.fetch(url, max_size)?;
        sandbox.stats.bytes_received += response.body.len();
        sandbox.page_cache.insert(url.into(), response.body.clone());
        Ok(response)
    }

    /// Suspend a sandbox (pause all activity)
    pub fn suspend(&mut self, id: SandboxId) -> Result<(), SandboxError> {
        let sandbox = self.sandboxes.get_mut(&id.0)
            .ok_or(SandboxError::NotFound)?;
        sandbox.state = SandboxState::Suspended;
        Ok(())
    }

    /// Resume a suspended sandbox
    pub fn resume(&mut self, id: SandboxId) -> Result<(), SandboxError> {
        let sandbox = self.sandboxes.get_mut(&id.0)
            .ok_or(SandboxError::NotFound)?;
        if sandbox.state == SandboxState::Suspended {
            sandbox.state = SandboxState::Active;
        }
        Ok(())
    }

    /// Destroy a sandbox and revoke its capability
    pub fn destroy(&mut self, id: SandboxId) -> Result<(), SandboxError> {
        if let Some(sandbox) = self.sandboxes.get_mut(&id.0) {
            sandbox.state = SandboxState::Terminated;
            security::revoke(sandbox.capability);
            self.audit(id, AuditAction::Destroyed, String::new());
        }
        self.sandboxes.remove(&id.0);
        Ok(())
    }

    /// Get a reference to a sandbox
    pub fn get(&self, id: SandboxId) -> Option<&Sandbox> {
        self.sandboxes.get(&id.0)
    }

    /// Get a mutable reference to a sandbox
    pub fn get_mut(&mut self, id: SandboxId) -> Option<&mut Sandbox> {
        self.sandboxes.get_mut(&id.0)
    }

    /// List all sandbox IDs and labels
    pub fn list(&self) -> Vec<(SandboxId, &str, SandboxState)> {
        self.sandboxes.values()
            .map(|s| (s.id, s.label.as_str(), s.state))
            .collect()
    }

    /// Number of active sandboxes
    pub fn count(&self) -> usize {
        self.sandboxes.len()
    }

    /// Add audit log entry
    fn audit(&mut self, id: SandboxId, action: AuditAction, detail: String) {
        let ts = crate::time::uptime_ms();
        self.audit_log.push(AuditEntry {
            timestamp_ms: ts,
            sandbox_id: id,
            action,
            detail,
        });
        // Ring buffer: keep last 256 entries
        if self.audit_log.len() > 256 {
            self.audit_log.remove(0);
        }
    }

    /// Get audit log entries (most recent first)
    pub fn audit_log(&self) -> &[AuditEntry] {
        &self.audit_log
    }

    /// Get audit entries for a specific sandbox
    pub fn audit_for(&self, id: SandboxId) -> Vec<&AuditEntry> {
        self.audit_log.iter().filter(|e| e.sandbox_id == id).collect()
    }
}

// ──── Error Types ──────────────────────────────────────────────────────────

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
            ProxyError::NetworkError(s) => SandboxError::NetworkError(s),
            ProxyError::DnsError => SandboxError::NetworkError(String::from("DNS resolution failed")),
            ProxyError::TlsError => SandboxError::NetworkError(String::from("TLS handshake failed")),
            ProxyError::Timeout => SandboxError::NetworkError(String::from("request timeout")),
            ProxyError::InvalidUrl => SandboxError::PolicyDenied(String::from("invalid URL")),
        }
    }
}

// ──── Global Instance ──────────────────────────────────────────────────────

lazy_static::lazy_static! {
    pub static ref SANDBOX_MANAGER: Mutex<SandboxManager> = Mutex::new(SandboxManager::new());
}

/// Initialize the sandbox subsystem — call after security::init()
pub fn init() {
    let mut mgr = SANDBOX_MANAGER.lock();
    mgr.register_capability_type();
    crate::serial_println!("[sandbox] Web Sandbox subsystem initialized");
    crate::serial_println!("[sandbox] Capability type registered: WebSandbox (danger=3)");
}

/// Create a new sandbox with preset policy
pub fn create(preset: PolicyPreset, label: Option<&str>) -> SandboxId {
    SANDBOX_MANAGER.lock().create_sandbox(preset, label)
}

/// Navigate sandbox to URL
pub fn navigate(id: SandboxId, url: &str) -> Result<ProxiedResponse, SandboxError> {
    SANDBOX_MANAGER.lock().navigate(id, url)
}

/// Fetch sub-resource in sandbox
pub fn fetch_resource(id: SandboxId, url: &str) -> Result<ProxiedResponse, SandboxError> {
    SANDBOX_MANAGER.lock().fetch_resource(id, url)
}

/// Destroy sandbox
pub fn destroy(id: SandboxId) -> Result<(), SandboxError> {
    SANDBOX_MANAGER.lock().destroy(id)
}

/// List all sandboxes
pub fn list() -> Vec<(SandboxId, String, SandboxState)> {
    SANDBOX_MANAGER.lock().list().into_iter()
        .map(|(id, s, st)| (id, String::from(s), st))
        .collect()
}

/// Get sandbox status info string
pub fn status_string(id: SandboxId) -> Option<String> {
    let mgr = SANDBOX_MANAGER.lock();
    let sb = mgr.get(id)?;
    Some(format!(
        "Sandbox #{} '{}'\n  State: {:?}\n  URL: {}\n  Requests: {}/{} ({} blocked)\n  Data: {} bytes\n  JS execs: {}\n  Violations: {}\n  FS: {} files, {} bytes\n  Policy: {:?}",
        sb.id.0, sb.label, sb.state,
        sb.current_url.as_deref().unwrap_or("(none)"),
        sb.stats.requests_made, sb.limits.max_total_requests, sb.stats.requests_blocked,
        sb.stats.bytes_received,
        sb.stats.js_executions,
        sb.stats.policy_violations,
        sb.stats.fs_files_created, sb.stats.fs_bytes_used,
        sb.policy.preset,
    ))
}
