





extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

use super::Ag;
use super::policy::{SandboxPolicy, cxk};




#[derive(Debug, Clone)]
pub struct Cs {
    pub status_code: u16,
    pub content_type: String,
    pub body: Vec<u8>,
    pub headers: Vec<(String, String)>,
    pub was_cached: bool,
}

impl Cs {
    
    pub fn body_string(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }

    
    pub fn is_html(&self) -> bool {
        self.content_type.contains("text/html") || self.content_type.contains("xhtml")
    }
}


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




struct RateLimiter {
    
    window: Vec<u64>,
    
    max_per_minute: u32,
}

impl RateLimiter {
    fn new(max_per_minute: u32) -> Self {
        Self {
            window: Vec::new(),
            max_per_minute,
        }
    }

    
    fn check_and_record(&mut self) -> bool {
        let cy = crate::time::uptime_ms();
        let nne = cy.saturating_sub(60_000);

        
        self.window.retain(|&jy| jy > nne);

        if self.window.len() as u32 >= self.max_per_minute {
            false
        } else {
            self.window.push(cy);
            true
        }
    }
}





pub struct NetProxy {
    sandbox_id: Ag,
    rate_limiter: RateLimiter,
    
    max_response_bytes: usize,
    allow_redirects: bool,
    max_redirect_depth: u8,
    block_private_ips: bool,
    
    total_requests: usize,
    
    total_bytes: usize,
}

impl NetProxy {
    pub fn new(sandbox_id: Ag, policy: &SandboxPolicy) -> Self {
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

    
    
    pub fn fetch(&mut self, url: &str, max_size: usize) -> Result<Cs, ProxyError> {
        
        if url.is_empty() {
            return Err(ProxyError::InvalidUrl);
        }

        
        let bnu = self.normalize_url(url);

        
        if !self.rate_limiter.check_and_record() {
            crate::serial_println!("[sandbox:{}] RATE LIMITED: {}", self.sandbox_id.0, bnu);
            return Err(ProxyError::RateLimited);
        }

        
        if self.block_private_ips {
            if let Some(domain) = cxk(&bnu) {
                if self.is_private_address(domain) {
                    crate::serial_println!("[sandbox:{}] SSRF BLOCKED: {}", self.sandbox_id.0, domain);
                    return Err(ProxyError::DomainBlocked(format!("private IP: {}", domain)));
                }
            }
        }

        
        let fue = core::cmp::min(max_size, self.max_response_bytes);
        let cln = bnu.starts_with("https://");

        crate::serial_println!("[sandbox:{}] FETCH: {} (max {} bytes)", self.sandbox_id.0, bnu, fue);

        let fa = if cln {
            self.fetch_https(&bnu, fue)?
        } else {
            self.fetch_http(&bnu, fue)?
        };

        self.total_requests += 1;
        self.total_bytes += fa.body.len();

        Ok(fa)
    }

    
    fn fetch_http(&self, url: &str, max_size: usize) -> Result<Cs, ProxyError> {
        
        
        match crate::netstack::http::get(url) {
            Ok(eo) => {
                let content_type = eo.headers.iter()
                    .find(|(k, _)| k.to_ascii_lowercase() == "content-type")
                    .map(|(_, v)| v.clone())
                    .unwrap_or_else(|| String::from("text/html"));

                let body = if eo.body.len() > max_size {
                    
                    crate::serial_println!("[sandbox:{}] Response truncated: {} -> {} bytes",
                        self.sandbox_id.0, eo.body.len(), max_size);
                    eo.body[..max_size].to_vec()
                } else {
                    eo.body
                };

                Ok(Cs {
                    status_code: eo.status_code,
                    content_type,
                    body,
                    headers: eo.headers,
                    was_cached: false,
                })
            }
            Err(e) => {
                crate::serial_println!("[sandbox:{}] HTTP error: {}", self.sandbox_id.0, e);
                Err(ProxyError::NetworkError(String::from(e)))
            }
        }
    }

    
    fn fetch_https(&self, url: &str, max_size: usize) -> Result<Cs, ProxyError> {
        match crate::netstack::https::get(url) {
            Ok(eo) => {
                let content_type = eo.headers.iter()
                    .find(|(k, _)| k.to_ascii_lowercase() == "content-type")
                    .map(|(_, v)| v.clone())
                    .unwrap_or_else(|| String::from("text/html"));

                let body = if eo.body.len() > max_size {
                    eo.body[..max_size].to_vec()
                } else {
                    eo.body
                };

                Ok(Cs {
                    status_code: eo.status_code,
                    content_type,
                    body,
                    headers: eo.headers,
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

    
    fn normalize_url(&self, url: &str) -> String {
        if url.starts_with("http://") || url.starts_with("https://") {
            String::from(url)
        } else if url.starts_with("//") {
            format!("http:{}", url)
        } else {
            format!("http://{}", url)
        }
    }

    
    fn is_private_address(&self, host: &str) -> bool {
        
        let nxj = [
            "localhost", "127.", "10.", "192.168.", "172.16.", "172.17.",
            "172.18.", "172.19.", "172.20.", "172.21.", "172.22.", "172.23.",
            "172.24.", "172.25.", "172.26.", "172.27.", "172.28.", "172.29.",
            "172.30.", "172.31.", "169.254.", "0.0.0.0", "[::]", "[::1]",
            "0.", "fc00:", "fd00:", "fe80:",
        ];
        let gj = host.to_ascii_lowercase();
        for pattern in &nxj {
            if gj.starts_with(pattern) || gj == *pattern {
                return true;
            }
        }
        
        
        if host.parse::<u32>().is_ok() {
            
            return true;
        }
        false
    }

    
    pub fn stats(&self) -> (usize, usize) {
        (self.total_requests, self.total_bytes)
    }
}
