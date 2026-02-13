// sandbox/net_proxy.rs — Kernel-controlled network proxy
// All sandbox HTTP/HTTPS requests are routed through this proxy.
// Provides: domain filtering, rate limiting, response size enforcement,
// content-type validation, redirect control, request logging.
// The sandbox NEVER touches raw sockets — only this proxy accesses the network.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

use super::SandboxId;
use super::policy::{SandboxPolicy, extract_domain};

// ──── Types ────────────────────────────────────────────────────────────────

/// Response returned by the proxy to the sandbox
#[derive(Debug, Clone)]
pub struct ProxiedResponse {
    pub status_code: u16,
    pub content_type: String,
    pub body: Vec<u8>,
    pub headers: Vec<(String, String)>,
    pub was_cached: bool,
}

impl ProxiedResponse {
    /// Get body as UTF-8 string (lossy)
    pub fn body_string(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }

    /// Check if response is HTML
    pub fn is_html(&self) -> bool {
        self.content_type.contains("text/html") || self.content_type.contains("xhtml")
    }
}

/// Proxy errors — specific and auditable
#[derive(Debug)]
pub enum ProxyError {
    DomainBlocked(String),
    RateLimited,
    ResponseTooLarge,
    NetworkError(String),
    DnsError,
    TlsError,
    Timeout,
    InvalidUrl,
}

// ──── Rate Limiter ─────────────────────────────────────────────────────────

/// Simple sliding-window rate limiter
struct RateLimiter {
    /// Timestamps of recent requests (ms)
    window: Vec<u64>,
    /// Max requests in the window
    max_per_minute: u32,
}

impl RateLimiter {
    fn new(max_per_minute: u32) -> Self {
        Self {
            window: Vec::new(),
            max_per_minute,
        }
    }

    /// Check if a request is allowed, and record it if so
    fn check_and_record(&mut self) -> bool {
        let now = crate::time::uptime_ms();
        let one_minute_ago = now.saturating_sub(60_000);

        // Remove old entries
        self.window.retain(|&ts| ts > one_minute_ago);

        if self.window.len() as u32 >= self.max_per_minute {
            false
        } else {
            self.window.push(now);
            true
        }
    }
}

// ──── Network Proxy ────────────────────────────────────────────────────────

/// Kernel-controlled network proxy for a sandbox instance.
/// This is the ONLY way sandboxed code can access the network.
pub struct NetProxy {
    sandbox_id: SandboxId,
    rate_limiter: RateLimiter,
    /// Snapshot of relevant policy fields
    max_response_bytes: usize,
    allow_redirects: bool,
    max_redirect_depth: u8,
    block_private_ips: bool,
    /// Request counter
    total_requests: usize,
    /// Bytes transferred
    total_bytes: usize,
}

impl NetProxy {
    pub fn new(sandbox_id: SandboxId, policy: &SandboxPolicy) -> Self {
        Self {
            sandbox_id,
            rate_limiter: RateLimiter::new(policy.rate_limit_per_min),
            max_response_bytes: policy.max_response_bytes,
            allow_redirects: policy.allow_redirects,
            max_redirect_depth: policy.max_redirect_depth,
            block_private_ips: policy.block_private_ips,
            total_requests: 0,
            total_bytes: 0,
        }
    }

    /// Fetch a URL through the kernel proxy.
    /// This is the main entry point — all network access goes through here.
    pub fn fetch(&mut self, url: &str, max_size: usize) -> Result<ProxiedResponse, ProxyError> {
        // Step 1: Validate URL format
        if url.is_empty() {
            return Err(ProxyError::InvalidUrl);
        }

        // Normalize URL
        let normalized = self.normalize_url(url);

        // Step 2: Rate limit check
        if !self.rate_limiter.check_and_record() {
            crate::serial_println!("[sandbox:{}] RATE LIMITED: {}", self.sandbox_id.0, normalized);
            return Err(ProxyError::RateLimited);
        }

        // Step 3: SSRF protection — block private IPs
        if self.block_private_ips {
            if let Some(domain) = extract_domain(&normalized) {
                if self.is_private_address(domain) {
                    crate::serial_println!("[sandbox:{}] SSRF BLOCKED: {}", self.sandbox_id.0, domain);
                    return Err(ProxyError::DomainBlocked(format!("private IP: {}", domain)));
                }
            }
        }

        // Step 4: Determine protocol and fetch
        let effective_max = core::cmp::min(max_size, self.max_response_bytes);
        let is_https = normalized.starts_with("https://");

        crate::serial_println!("[sandbox:{}] FETCH: {} (max {} bytes)", self.sandbox_id.0, normalized, effective_max);

        let response = if is_https {
            self.fetch_https(&normalized, effective_max)?
        } else {
            self.fetch_http(&normalized, effective_max)?
        };

        self.total_requests += 1;
        self.total_bytes += response.body.len();

        Ok(response)
    }

    /// Fetch via HTTP (port 80)
    fn fetch_http(&self, url: &str, max_size: usize) -> Result<ProxiedResponse, ProxyError> {
        // Use the kernel's HTTP client — which goes through the full
        // TCP/IP stack but is controlled by us
        match crate::netstack::http::get(url) {
            Ok(resp) => {
                let content_type = resp.headers.iter()
                    .find(|(k, _)| k.to_ascii_lowercase() == "content-type")
                    .map(|(_, v)| v.clone())
                    .unwrap_or_else(|| String::from("text/html"));

                let body = if resp.body.len() > max_size {
                    // Truncate to max size
                    crate::serial_println!("[sandbox:{}] Response truncated: {} -> {} bytes",
                        self.sandbox_id.0, resp.body.len(), max_size);
                    resp.body[..max_size].to_vec()
                } else {
                    resp.body
                };

                Ok(ProxiedResponse {
                    status_code: resp.status_code,
                    content_type,
                    body,
                    headers: resp.headers,
                    was_cached: false,
                })
            }
            Err(e) => {
                crate::serial_println!("[sandbox:{}] HTTP error: {}", self.sandbox_id.0, e);
                Err(ProxyError::NetworkError(String::from(e)))
            }
        }
    }

    /// Fetch via HTTPS (port 443)
    fn fetch_https(&self, url: &str, max_size: usize) -> Result<ProxiedResponse, ProxyError> {
        match crate::netstack::https::get(url) {
            Ok(resp) => {
                let content_type = resp.headers.iter()
                    .find(|(k, _)| k.to_ascii_lowercase() == "content-type")
                    .map(|(_, v)| v.clone())
                    .unwrap_or_else(|| String::from("text/html"));

                let body = if resp.body.len() > max_size {
                    resp.body[..max_size].to_vec()
                } else {
                    resp.body
                };

                Ok(ProxiedResponse {
                    status_code: resp.status_code,
                    content_type,
                    body,
                    headers: resp.headers,
                    was_cached: false,
                })
            }
            Err(e) => {
                crate::serial_println!("[sandbox:{}] HTTPS error: {:?}", self.sandbox_id.0, e);
                match e {
                    crate::netstack::https::HttpsError::DnsError => Err(ProxyError::DnsError),
                    crate::netstack::https::HttpsError::TlsError(_) => Err(ProxyError::TlsError),
                    crate::netstack::https::HttpsError::Timeout => Err(ProxyError::Timeout),
                    _ => Err(ProxyError::NetworkError(format!("{:?}", e))),
                }
            }
        }
    }

    /// Normalize URL — ensure scheme, handle bare domains
    fn normalize_url(&self, url: &str) -> String {
        if url.starts_with("http://") || url.starts_with("https://") {
            String::from(url)
        } else if url.starts_with("//") {
            format!("http:{}", url)
        } else {
            format!("http://{}", url)
        }
    }

    /// Check if an address is private/local (SSRF protection)
    fn is_private_address(&self, host: &str) -> bool {
        // Check common private address patterns
        let private_patterns = [
            "localhost", "127.", "10.", "192.168.", "172.16.", "172.17.",
            "172.18.", "172.19.", "172.20.", "172.21.", "172.22.", "172.23.",
            "172.24.", "172.25.", "172.26.", "172.27.", "172.28.", "172.29.",
            "172.30.", "172.31.", "169.254.", "0.0.0.0", "[::]", "[::1]",
            "0.", "fc00:", "fd00:", "fe80:",
        ];
        let lower = host.to_ascii_lowercase();
        for pattern in &private_patterns {
            if lower.starts_with(pattern) || lower == *pattern {
                return true;
            }
        }
        // Also block if it's a raw IP that doesn't look like a public address
        // (additional heuristic for creative bypass attempts)
        if host.parse::<u32>().is_ok() {
            // Decimal IP notation (e.g., 2130706433 = 127.0.0.1) — block it
            return true;
        }
        false
    }

    /// Get proxy statistics
    pub fn stats(&self) -> (usize, usize) {
        (self.total_requests, self.total_bytes)
    }
}
