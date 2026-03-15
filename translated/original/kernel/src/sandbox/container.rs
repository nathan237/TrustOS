// sandbox/container.rs — WebContainer: persistent isolated web service runtime
// Auto-started at boot, provides a long-running sandboxed execution environment
// for web services with kernel-enforced resource limits, watchdog, and audit.
//
// Architecture:
//   ┌──────────────────────────────────────────────────────┐
//   │                 WebContainer                         │
//   │  ┌────────────┐ ┌────────────┐ ┌──────────────────┐ │
//   │  │  NetProxy   │ │ SandboxFs  │ │ JsSandbox        │ │
//   │  │  (filtered) │ │ (jailed)   │ │ (threat-scanned) │ │
//   │  └──────┬─────┘ └─────┬──────┘ └────────┬─────────┘ │
//   │         │             │                  │           │
//   │  ┌──────┴─────────────┴──────────────────┴─────────┐ │
//   │  │      Container Runtime                          │ │
//   │  │  - Capability token (non-escalable)             │ │
//   │  │  - Memory budget enforcement                    │ │
//   │  │  - Request budget + rate limit                  │ │
//   │  │  - DNS allow/deny (kernel-level)                │ │
//   │  │  - Watchdog timer (stall detection)             │ │
//   │  │  - Health checks                                │ │
//   │  │  - Full audit trail                             │ │
//   │  └─────────────────────────────────────────────────┘ │
//   │         │                                            │
//   │  kernel TCP/IP ──→ NIC driver ──→ network            │
//   └──────────────────────────────────────────────────────┘
//
// Future: when EPT/VT-x isolation is fixed, the container can be
// promoted to a hardware-isolated micro-VM with no code changes
// (the same sandbox API wraps both modes).

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use alloc::collections::BTreeMap;
use spin::Mutex;

use super::{SandboxId, SandboxState, SandboxError, SANDBOX_MANAGER};
use super::policy::PolicyPreset;
use super::net_proxy::ProxiedResponse;

// ──── Container Configuration ──────────────────────────────────────────────

/// Container isolation mode — determines the enforcement backend
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsolationMode {
    /// Software-only: capability gates, policy engine, kernel proxy
    /// Works everywhere (VBox, QEMU, bare metal)
    SoftwareOnly,
    /// Hardware-assisted: EPT/NPT memory isolation (requires VT-x/AMD-V)
    /// Only works on bare metal or QEMU with nested VMX
    /// (Not yet implemented - EPT map_page() is a stub)
    HardwareAssisted,
}

/// Container health status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    /// Container is healthy and operating normally
    Healthy,
    /// Container is degraded (e.g., rate limited, nearing quotas)
    Degraded,
    /// Container has critical issues (watchdog triggered, policy violations)
    Critical,
    /// Container is stopped
    Stopped,
}

/// Configuration for a web container
#[derive(Debug, Clone)]
pub struct ContainerConfig {
    /// Human-readable name
    pub name: String,
    /// Security preset (determines policy)
    pub preset: PolicyPreset,
    /// Isolation mode
    pub isolation: IsolationMode,
    /// Max memory in bytes (per container)
    pub memory_limit: usize,
    /// Max total HTTP requests per session
    pub request_limit: usize,
    /// Max response body size
    pub max_response_size: usize,
    /// Request rate limit (per minute)
    pub rate_limit: u32,
    /// Allowed domains (empty = use preset default)
    pub allowed_domains: Vec<String>,
    /// Blocked domains (prepended to preset)
    pub blocked_domains: Vec<String>,
    /// Auto-restart on failure
    pub auto_restart: bool,
    /// Watchdog timeout in seconds (0 = disabled)
    pub watchdog_timeout_secs: u64,
    /// Allow JavaScript execution
    pub allow_js: bool,
}

impl Default for ContainerConfig {
    fn default() -> Self {
        Self {
            name: String::from("web-container"),
            preset: PolicyPreset::Moderate,
            isolation: IsolationMode::SoftwareOnly,
            memory_limit: 4 * 1024 * 1024,     // 4 MB
            request_limit: 200,
            max_response_size: 1024 * 1024,     // 1 MB
            rate_limit: 60,
            allowed_domains: Vec::new(),
            blocked_domains: Vec::new(),
            auto_restart: true,
            watchdog_timeout_secs: 30,
            allow_js: false, // secure by default
        }
    }
}

impl ContainerConfig {
    /// Secure preset: strict policy, no JS, low limits
    pub fn secure() -> Self {
        Self {
            name: String::from("web-secure"),
            preset: PolicyPreset::Strict,
            isolation: IsolationMode::SoftwareOnly,
            memory_limit: 2 * 1024 * 1024,
            request_limit: 50,
            max_response_size: 512 * 1024,
            rate_limit: 20,
            allowed_domains: Vec::new(),
            blocked_domains: Vec::new(),
            auto_restart: true,
            watchdog_timeout_secs: 15,
            allow_js: false,
        }
    }

    /// Dev preset: permissive, JS allowed, higher limits
    pub fn dev() -> Self {
        Self {
            name: String::from("web-dev"),
            preset: PolicyPreset::Permissive,
            isolation: IsolationMode::SoftwareOnly,
            memory_limit: 8 * 1024 * 1024,
            request_limit: 500,
            max_response_size: 4 * 1024 * 1024,
            rate_limit: 120,
            allowed_domains: Vec::new(),
            blocked_domains: Vec::new(),
            auto_restart: false,
            watchdog_timeout_secs: 0,
            allow_js: true,
        }
    }
}

// ──── Web Container ────────────────────────────────────────────────────────

/// A persistent web container — long-lived sandbox with lifecycle management
pub struct WebContainer {
    /// Unique container ID
    pub id: u32,
    /// Configuration
    pub config: ContainerConfig,
    /// Underlying sandbox ID (links to SandboxManager)
    pub sandbox_id: Option<SandboxId>,
    /// Health status
    pub health: HealthStatus,
    /// Boot timestamp (ms since boot)
    pub started_at: u64,
    /// Last activity timestamp
    pub last_activity: u64,
    /// Watchdog last check
    pub watchdog_last_check: u64,
    /// Number of restarts
    pub restart_count: u32,
    /// Navigation history
    pub history: Vec<String>,
    /// DNS cache (domain → resolved, for audit)
    pub dns_log: Vec<(String, bool)>,
    /// Total bytes transferred
    pub total_bytes: usize,
    /// Total requests made
    pub total_requests: usize,
    /// Total policy violations
    pub violations: usize,
}

impl WebContainer {
    fn new(id: u32, config: ContainerConfig) -> Self {
        let now = crate::time::uptime_ms();
        Self {
            id,
            config,
            sandbox_id: None,
            health: HealthStatus::Stopped,
            started_at: now,
            last_activity: now,
            watchdog_last_check: now,
            restart_count: 0,
            history: Vec::new(),
            dns_log: Vec::new(),
            total_bytes: 0,
            total_requests: 0,
            violations: 0,
        }
    }

    /// Start the container — creates underlying sandbox
    pub fn start(&mut self) -> Result<(), SandboxError> {
        if self.sandbox_id.is_some() {
            return Ok(()); // already running
        }

        // Create sandbox through the global manager
        let label = format!("container:{}", self.config.name);
        let sandbox_id = {
            let mut mgr = SANDBOX_MANAGER.lock();
            let sid = mgr.create_sandbox(self.config.preset, Some(&label));

            // Apply container-specific configuration to the sandbox
            if let Some(sb) = mgr.get_mut(sid) {
                sb.limits.max_memory_bytes = self.config.memory_limit;
                sb.limits.max_total_requests = self.config.request_limit;
                sb.limits.max_response_size = self.config.max_response_size;

                // Apply domain rules
                for domain in &self.config.allowed_domains {
                    sb.policy.allow_domain(domain);
                }
                for domain in &self.config.blocked_domains {
                    sb.policy.deny_domain(domain);
                }

                // JS policy
                sb.policy.block_javascript = !self.config.allow_js;
                sb.policy.rate_limit_per_min = self.config.rate_limit;
            }

            sid
        };

        self.sandbox_id = Some(sandbox_id);
        self.health = HealthStatus::Healthy;
        self.started_at = crate::time::uptime_ms();
        self.last_activity = self.started_at;

        crate::serial_println!("[container:{}] Started (sandbox #{}, {:?}, {:?})",
            self.id, sandbox_id.0, self.config.preset, self.config.isolation);

        Ok(())
    }

    /// Stop the container — destroys underlying sandbox
    pub fn stop(&mut self) {
        if let Some(sid) = self.sandbox_id.take() {
            let _ = SANDBOX_MANAGER.lock().destroy(sid);
        }
        self.health = HealthStatus::Stopped;
        crate::serial_println!("[container:{}] Stopped", self.id);
    }

    /// Restart the container (stop + start)
    pub fn restart(&mut self) -> Result<(), SandboxError> {
        self.stop();
        self.restart_count += 1;
        self.start()
    }

    /// Navigate to a URL through the container's sandbox
    pub fn navigate(&mut self, url: &str) -> Result<ProxiedResponse, SandboxError> {
        let sid = self.sandbox_id.ok_or(SandboxError::Terminated)?;

        let response = SANDBOX_MANAGER.lock().navigate(sid, url)?;

        self.last_activity = crate::time::uptime_ms();
        self.total_bytes += response.body.len();
        self.total_requests += 1;
        self.history.push(String::from(url));
        if self.history.len() > 50 {
            self.history.remove(0);
        }

        // Update health based on stats
        self.update_health();

        Ok(response)
    }

    /// Fetch a sub-resource
    pub fn fetch(&mut self, url: &str) -> Result<ProxiedResponse, SandboxError> {
        let sid = self.sandbox_id.ok_or(SandboxError::Terminated)?;

        let response = SANDBOX_MANAGER.lock().fetch_resource(sid, url)?;

        self.last_activity = crate::time::uptime_ms();
        self.total_bytes += response.body.len();
        self.total_requests += 1;

        Ok(response)
    }

    /// Check watchdog — detect stalled containers
    pub fn watchdog_check(&mut self) -> bool {
        if self.config.watchdog_timeout_secs == 0 {
            return true; // watchdog disabled
        }
        if self.health == HealthStatus::Stopped {
            return true;
        }

        let now = crate::time::uptime_ms();
        let timeout_ms = self.config.watchdog_timeout_secs * 1000;

        // Check if sandbox is still responsive
        if let Some(sid) = self.sandbox_id {
            let mgr = SANDBOX_MANAGER.lock();
            if let Some(sb) = mgr.get(sid) {
                if sb.state == SandboxState::Terminated {
                    drop(mgr);
                    crate::serial_println!("[container:{}] watchdog: sandbox terminated, restarting", self.id);
                    if self.config.auto_restart {
                        self.sandbox_id = None;
                        let _ = self.restart();
                    } else {
                        self.health = HealthStatus::Critical;
                    }
                    return false;
                }
                // Check for stall (no activity for timeout period)
                if now.saturating_sub(self.last_activity) > timeout_ms && self.total_requests > 0 {
                    self.health = HealthStatus::Degraded;
                }
            } else {
                drop(mgr);
                // Sandbox disappeared — restart if auto-restart
                if self.config.auto_restart {
                    self.sandbox_id = None;
                    let _ = self.restart();
                } else {
                    self.health = HealthStatus::Critical;
                }
                return false;
            }
        }

        self.watchdog_last_check = now;
        true
    }

    /// Update health status based on resource usage
    fn update_health(&mut self) {
        if self.sandbox_id.is_none() {
            self.health = HealthStatus::Stopped;
            return;
        }

        // Check if nearing limits
        let req_pct = if self.config.request_limit > 0 {
            (self.total_requests * 100) / self.config.request_limit
        } else {
            0
        };

        let mem_pct = if self.config.memory_limit > 0 {
            (self.total_bytes * 100) / self.config.memory_limit
        } else {
            0
        };

        if req_pct > 90 || mem_pct > 90 || self.violations > 10 {
            self.health = HealthStatus::Critical;
        } else if req_pct > 70 || mem_pct > 70 || self.violations > 3 {
            self.health = HealthStatus::Degraded;
        } else {
            self.health = HealthStatus::Healthy;
        }
    }

    /// Get uptime in seconds
    pub fn uptime_secs(&self) -> u64 {
        crate::time::uptime_ms().saturating_sub(self.started_at) / 1000
    }

    /// Get container status as string
    pub fn status_string(&self) -> String {
        let isolation_str = match self.config.isolation {
            IsolationMode::SoftwareOnly => "software (capabilities)",
            IsolationMode::HardwareAssisted => "hardware (EPT/VT-x)",
        };

        format!(
            "Container #{} '{}'\n\
             ├─ Health:     {:?}\n\
             ├─ Isolation:  {}\n\
             ├─ Policy:     {:?}\n\
             ├─ Sandbox:    {}\n\
             ├─ Uptime:     {}s\n\
             ├─ Requests:   {}/{}\n\
             ├─ Data:       {} bytes (limit {} MB)\n\
             ├─ Violations: {}\n\
             ├─ Restarts:   {}\n\
             ├─ JS:         {}\n\
             ├─ Rate limit: {}/min\n\
             └─ Watchdog:   {}",
            self.id, self.config.name,
            self.health,
            isolation_str,
            self.config.preset,
            self.sandbox_id.map(|s| format!("#{}", s.0)).unwrap_or_else(|| String::from("none")),
            self.uptime_secs(),
            self.total_requests, self.config.request_limit,
            self.total_bytes, self.config.memory_limit / (1024 * 1024),
            self.violations,
            self.restart_count,
            if self.config.allow_js { "allowed" } else { "BLOCKED" },
            self.config.rate_limit,
            if self.config.watchdog_timeout_secs > 0 {
                format!("{}s", self.config.watchdog_timeout_secs)
            } else {
                String::from("disabled")
            },
        )
    }
}

// ──── Container Daemon ─────────────────────────────────────────────────────

/// The container daemon — manages all web containers, auto-started at boot
pub struct ContainerDaemon {
    containers: BTreeMap<u32, WebContainer>,
    next_id: u32,
    /// Whether the daemon was initialized at boot
    pub booted: bool,
    /// Boot timestamp
    pub boot_time: u64,
    /// Default container ID (the auto-started one)
    pub default_container: Option<u32>,
}

impl ContainerDaemon {
    pub fn new() -> Self {
        Self {
            containers: BTreeMap::new(),
            next_id: 1,
            booted: false,
            boot_time: 0,
            default_container: None,
        }
    }

    /// Boot the daemon — creates and starts the default web container
    pub fn boot(&mut self) {
        self.boot_time = crate::time::uptime_ms();
        self.booted = true;

        // Create default secure web container
        let id = self.create_container(ContainerConfig::default());
        if let Some(container) = self.containers.get_mut(&id) {
            match container.start() {
                Ok(()) => {
                    self.default_container = Some(id);
                    crate::serial_println!("[daemon] Default web container #{} started", id);
                }
                Err(e) => {
                    crate::serial_println!("[daemon] Failed to start default container: {:?}", e);
                }
            }
        }

        crate::serial_println!("[daemon] Container daemon booted at {}ms", self.boot_time);
    }

    /// Create a new container (does not start it)
    pub fn create_container(&mut self, config: ContainerConfig) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        let container = WebContainer::new(id, config);
        self.containers.insert(id, container);
        id
    }

    /// Start a container
    pub fn start_container(&mut self, id: u32) -> Result<(), SandboxError> {
        let container = self.containers.get_mut(&id)
            .ok_or(SandboxError::NotFound)?;
        container.start()
    }

    /// Stop a container
    pub fn stop_container(&mut self, id: u32) -> Result<(), SandboxError> {
        let container = self.containers.get_mut(&id)
            .ok_or(SandboxError::NotFound)?;
        container.stop();
        Ok(())
    }

    /// Restart a container
    pub fn restart_container(&mut self, id: u32) -> Result<(), SandboxError> {
        let container = self.containers.get_mut(&id)
            .ok_or(SandboxError::NotFound)?;
        container.restart()
    }

    /// Destroy a container
    pub fn destroy_container(&mut self, id: u32) -> Result<(), SandboxError> {
        if let Some(mut container) = self.containers.remove(&id) {
            container.stop();
            if self.default_container == Some(id) {
                self.default_container = None;
            }
        }
        Ok(())
    }

    /// Navigate the default container (or specified container) to a URL
    pub fn navigate(&mut self, id: Option<u32>, url: &str) -> Result<ProxiedResponse, SandboxError> {
        let container_id = id
            .or(self.default_container)
            .ok_or(SandboxError::NotFound)?;
        let container = self.containers.get_mut(&container_id)
            .ok_or(SandboxError::NotFound)?;
        container.navigate(url)
    }

    /// Get a container reference
    pub fn get(&self, id: u32) -> Option<&WebContainer> {
        self.containers.get(&id)
    }

    /// Get a mutable container reference
    pub fn get_mut(&mut self, id: u32) -> Option<&mut WebContainer> {
        self.containers.get_mut(&id)
    }

    /// List all containers
    pub fn list(&self) -> Vec<(u32, &str, HealthStatus, bool)> {
        self.containers.values()
            .map(|c| (
                c.id,
                c.config.name.as_str(),
                c.health,
                self.default_container == Some(c.id),
            ))
            .collect()
    }

    /// Run watchdog checks on all containers
    pub fn watchdog_tick(&mut self) {
        let ids: Vec<u32> = self.containers.keys().copied().collect();
        for id in ids {
            if let Some(container) = self.containers.get_mut(&id) {
                container.watchdog_check();
            }
        }
    }

    /// Get daemon uptime in seconds
    pub fn uptime_secs(&self) -> u64 {
        crate::time::uptime_ms().saturating_sub(self.boot_time) / 1000
    }

    /// Get daemon status summary
    pub fn status_string(&self) -> String {
        let total = self.containers.len();
        let running = self.containers.values()
            .filter(|c| c.health != HealthStatus::Stopped)
            .count();
        let healthy = self.containers.values()
            .filter(|c| c.health == HealthStatus::Healthy)
            .count();

        format!(
            "Container Daemon\n\
             ├─ Status:     {}\n\
             ├─ Uptime:     {}s\n\
             ├─ Containers: {} total, {} running, {} healthy\n\
             ├─ Default:    {}\n\
             └─ Isolation:  {} (EPT/VT-x: {})",
            if self.booted { "running" } else { "stopped" },
            self.uptime_secs(),
            total, running, healthy,
            self.default_container.map(|id| format!("#{}", id)).unwrap_or_else(|| String::from("none")),
            "software",
            if Self::hardware_isolation_available() { "available" } else { "not available" },
        )
    }

    /// Check if hardware isolation (VT-x/EPT) is available
    fn hardware_isolation_available() -> bool {
        // Check if the hypervisor detected VT-x or AMD-V support
        // In VirtualBox with nested-hw-virt off, this will be false
        crate::hypervisor::is_enabled()
    }
}

// ──── Global Instance ──────────────────────────────────────────────────────

lazy_static::lazy_static! {
    pub static ref CONTAINER_DAEMON: Mutex<ContainerDaemon> = Mutex::new(ContainerDaemon::new());
}

/// Boot the container daemon — call from main.rs after sandbox::init()
pub fn boot_daemon() {
    let mut daemon = CONTAINER_DAEMON.lock();
    daemon.boot();
}

/// Navigate through the default container
pub fn navigate(url: &str) -> Result<ProxiedResponse, SandboxError> {
    CONTAINER_DAEMON.lock().navigate(None, url)
}

/// List containers
pub fn list_containers() -> Vec<(u32, String, HealthStatus, bool)> {
    CONTAINER_DAEMON.lock().list().into_iter()
        .map(|(id, n, h, d)| (id, String::from(n), h, d))
        .collect()
}

/// Get daemon status
pub fn daemon_status() -> String {
    CONTAINER_DAEMON.lock().status_string()
}

/// Get container status
pub fn container_status(id: u32) -> Option<String> {
    CONTAINER_DAEMON.lock().get(id).map(|c| c.status_string())
}

/// Run watchdog tick
pub fn watchdog_tick() {
    CONTAINER_DAEMON.lock().watchdog_tick();
}
