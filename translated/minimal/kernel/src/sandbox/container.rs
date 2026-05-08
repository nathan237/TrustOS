





























extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use alloc::collections::BTreeMap;
use spin::Mutex;

use super::{Ag, SandboxState, SandboxError, BE_};
use super::policy::PolicyPreset;
use super::net_proxy::Cs;




#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsolationMode {
    
    
    SoftwareOnly,
    
    
    
    HardwareAssisted,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    
    Healthy,
    
    Degraded,
    
    Critical,
    
    Stopped,
}


#[derive(Debug, Clone)]
pub struct ContainerConfig {
    
    pub name: String,
    
    pub preset: PolicyPreset,
    
    pub isolation: IsolationMode,
    
    pub memory_limit: usize,
    
    pub request_limit: usize,
    
    pub max_response_size: usize,
    
    pub rate_limit: u32,
    
    pub allowed_domains: Vec<String>,
    
    pub blocked_domains: Vec<String>,
    
    pub auto_restart: bool,
    
    pub watchdog_timeout_secs: u64,
    
    pub allow_js: bool,
}

impl Default for ContainerConfig {
    fn default() -> Self {
        Self {
            name: String::from("web-container"),
            preset: PolicyPreset::Moderate,
            isolation: IsolationMode::SoftwareOnly,
            memory_limit: 4 * 1024 * 1024,     
            request_limit: 200,
            max_response_size: 1024 * 1024,     
            rate_limit: 60,
            allowed_domains: Vec::new(),
            blocked_domains: Vec::new(),
            auto_restart: true,
            watchdog_timeout_secs: 30,
            allow_js: false, 
        }
    }
}

impl ContainerConfig {
    
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

    
    pub fn s() -> Self {
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




pub struct WebContainer {
    
    pub id: u32,
    
    pub config: ContainerConfig,
    
    pub sandbox_id: Option<Ag>,
    
    pub health: HealthStatus,
    
    pub started_at: u64,
    
    pub last_activity: u64,
    
    pub watchdog_last_check: u64,
    
    pub restart_count: u32,
    
    pub history: Vec<String>,
    
    pub dns_log: Vec<(String, bool)>,
    
    pub total_bytes: usize,
    
    pub total_requests: usize,
    
    pub violations: usize,
}

impl WebContainer {
    fn new(id: u32, config: ContainerConfig) -> Self {
        let cy = crate::time::uptime_ms();
        Self {
            id,
            config,
            sandbox_id: None,
            health: HealthStatus::Stopped,
            started_at: cy,
            last_activity: cy,
            watchdog_last_check: cy,
            restart_count: 0,
            history: Vec::new(),
            dns_log: Vec::new(),
            total_bytes: 0,
            total_requests: 0,
            violations: 0,
        }
    }

    
    pub fn start(&mut self) -> Result<(), SandboxError> {
        if self.sandbox_id.is_some() {
            return Ok(()); 
        }

        
        let label = format!("container:{}", self.config.name);
        let sandbox_id = {
            let mut ng = BE_.lock();
            let sid = ng.create_sandbox(self.config.preset, Some(&label));

            
            if let Some(cv) = ng.get_mut(sid) {
                cv.limits.max_memory_bytes = self.config.memory_limit;
                cv.limits.max_total_requests = self.config.request_limit;
                cv.limits.max_response_size = self.config.max_response_size;

                
                for domain in &self.config.allowed_domains {
                    cv.policy.allow_domain(domain);
                }
                for domain in &self.config.blocked_domains {
                    cv.policy.deny_domain(domain);
                }

                
                cv.policy.block_javascript = !self.config.allow_js;
                cv.policy.rate_limit_per_min = self.config.rate_limit;
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

    
    pub fn stop(&mut self) {
        if let Some(sid) = self.sandbox_id.take() {
            let _ = BE_.lock().destroy(sid);
        }
        self.health = HealthStatus::Stopped;
        crate::serial_println!("[container:{}] Stopped", self.id);
    }

    
    pub fn restart(&mut self) -> Result<(), SandboxError> {
        self.stop();
        self.restart_count += 1;
        self.start()
    }

    
    pub fn navigate(&mut self, url: &str) -> Result<Cs, SandboxError> {
        let sid = self.sandbox_id.ok_or(SandboxError::Terminated)?;

        let fa = BE_.lock().navigate(sid, url)?;

        self.last_activity = crate::time::uptime_ms();
        self.total_bytes += fa.body.len();
        self.total_requests += 1;
        self.history.push(String::from(url));
        if self.history.len() > 50 {
            self.history.remove(0);
        }

        
        self.update_health();

        Ok(fa)
    }

    
    pub fn fetch(&mut self, url: &str) -> Result<Cs, SandboxError> {
        let sid = self.sandbox_id.ok_or(SandboxError::Terminated)?;

        let fa = BE_.lock().fetch_resource(sid, url)?;

        self.last_activity = crate::time::uptime_ms();
        self.total_bytes += fa.body.len();
        self.total_requests += 1;

        Ok(fa)
    }

    
    pub fn watchdog_check(&mut self) -> bool {
        if self.config.watchdog_timeout_secs == 0 {
            return true; 
        }
        if self.health == HealthStatus::Stopped {
            return true;
        }

        let cy = crate::time::uptime_ms();
        let timeout_ms = self.config.watchdog_timeout_secs * 1000;

        
        if let Some(sid) = self.sandbox_id {
            let ng = BE_.lock();
            if let Some(cv) = ng.get(sid) {
                if cv.state == SandboxState::Terminated {
                    drop(ng);
                    crate::serial_println!("[container:{}] watchdog: sandbox terminated, restarting", self.id);
                    if self.config.auto_restart {
                        self.sandbox_id = None;
                        let _ = self.restart();
                    } else {
                        self.health = HealthStatus::Critical;
                    }
                    return false;
                }
                
                if cy.saturating_sub(self.last_activity) > timeout_ms && self.total_requests > 0 {
                    self.health = HealthStatus::Degraded;
                }
            } else {
                drop(ng);
                
                if self.config.auto_restart {
                    self.sandbox_id = None;
                    let _ = self.restart();
                } else {
                    self.health = HealthStatus::Critical;
                }
                return false;
            }
        }

        self.watchdog_last_check = cy;
        true
    }

    
    fn update_health(&mut self) {
        if self.sandbox_id.is_none() {
            self.health = HealthStatus::Stopped;
            return;
        }

        
        let jae = if self.config.request_limit > 0 {
            (self.total_requests * 100) / self.config.request_limit
        } else {
            0
        };

        let inh = if self.config.memory_limit > 0 {
            (self.total_bytes * 100) / self.config.memory_limit
        } else {
            0
        };

        if jae > 90 || inh > 90 || self.violations > 10 {
            self.health = HealthStatus::Critical;
        } else if jae > 70 || inh > 70 || self.violations > 3 {
            self.health = HealthStatus::Degraded;
        } else {
            self.health = HealthStatus::Healthy;
        }
    }

    
    pub fn uptime_secs(&self) -> u64 {
        crate::time::uptime_ms().saturating_sub(self.started_at) / 1000
    }

    
    pub fn status_string(&self) -> String {
        let muj = match self.config.isolation {
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
            muj,
            self.config.preset,
            self.sandbox_id.map(|j| format!("#{}", j.0)).unwrap_or_else(|| String::from("none")),
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




pub struct ContainerDaemon {
    containers: BTreeMap<u32, WebContainer>,
    next_id: u32,
    
    pub booted: bool,
    
    pub boot_time: u64,
    
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

    
    pub fn boot(&mut self) {
        self.boot_time = crate::time::uptime_ms();
        self.booted = true;

        
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

    
    pub fn create_container(&mut self, config: ContainerConfig) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        let container = WebContainer::new(id, config);
        self.containers.insert(id, container);
        id
    }

    
    pub fn start_container(&mut self, id: u32) -> Result<(), SandboxError> {
        let container = self.containers.get_mut(&id)
            .ok_or(SandboxError::NotFound)?;
        container.start()
    }

    
    pub fn stop_container(&mut self, id: u32) -> Result<(), SandboxError> {
        let container = self.containers.get_mut(&id)
            .ok_or(SandboxError::NotFound)?;
        container.stop();
        Ok(())
    }

    
    pub fn restart_container(&mut self, id: u32) -> Result<(), SandboxError> {
        let container = self.containers.get_mut(&id)
            .ok_or(SandboxError::NotFound)?;
        container.restart()
    }

    
    pub fn destroy_container(&mut self, id: u32) -> Result<(), SandboxError> {
        if let Some(mut container) = self.containers.remove(&id) {
            container.stop();
            if self.default_container == Some(id) {
                self.default_container = None;
            }
        }
        Ok(())
    }

    
    pub fn navigate(&mut self, id: Option<u32>, url: &str) -> Result<Cs, SandboxError> {
        let foh = id
            .or(self.default_container)
            .ok_or(SandboxError::NotFound)?;
        let container = self.containers.get_mut(&foh)
            .ok_or(SandboxError::NotFound)?;
        container.navigate(url)
    }

    
    pub fn get(&self, id: u32) -> Option<&WebContainer> {
        self.containers.get(&id)
    }

    
    pub fn get_mut(&mut self, id: u32) -> Option<&mut WebContainer> {
        self.containers.get_mut(&id)
    }

    
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

    
    pub fn watchdog_tick(&mut self) {
        let ids: Vec<u32> = self.containers.keys().copied().collect();
        for id in ids {
            if let Some(container) = self.containers.get_mut(&id) {
                container.watchdog_check();
            }
        }
    }

    
    pub fn uptime_secs(&self) -> u64 {
        crate::time::uptime_ms().saturating_sub(self.boot_time) / 1000
    }

    
    pub fn status_string(&self) -> String {
        let av = self.containers.len();
        let running = self.containers.values()
            .filter(|c| c.health != HealthStatus::Stopped)
            .count();
        let mks = self.containers.values()
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
            av, running, mks,
            self.default_container.map(|id| format!("#{}", id)).unwrap_or_else(|| String::from("none")),
            "software",
            if Self::mix() { "available" } else { "not available" },
        )
    }

    
    fn mix() -> bool {
        
        
        crate::hypervisor::lq()
    }
}



lazy_static::lazy_static! {
    pub static ref CN_: Mutex<ContainerDaemon> = Mutex::new(ContainerDaemon::new());
}


pub fn kde() {
    let mut agj = CN_.lock();
    agj.boot();
}


pub fn navigate(url: &str) -> Result<Cs, SandboxError> {
    CN_.lock().navigate(None, url)
}


pub fn mzb() -> Vec<(u32, String, HealthStatus, bool)> {
    CN_.lock().list().into_iter()
        .map(|(id, ae, h, d)| (id, String::from(ae), h, d))
        .collect()
}


pub fn lbk() -> String {
    CN_.lock().status_string()
}


pub fn kxi(id: u32) -> Option<String> {
    CN_.lock().get(id).map(|c| c.status_string())
}


pub fn watchdog_tick() {
    CN_.lock().watchdog_tick();
}
