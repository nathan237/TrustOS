// sandbox/policy.rs — URL / domain / content security policy engine
// Implements allow/deny lists, rate limiting, content type filtering,
// and preset security levels for web sandbox isolation.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

// ──── Policy Verdicts ──────────────────────────────────────────────────────

/// Result of policy evaluation
#[derive(Debug, Clone)]
pub enum PolicyVerdict {
    /// Request is allowed
    Allow,
    /// Request is denied with reason
    Deny(String),
    /// Request is allowed but logged for audit
    Log,
}

// ──── Security Presets ─────────────────────────────────────────────────────

/// Predefined security levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyPreset {
    /// Only explicitly allowlisted domains — maximum security
    Strict,
    /// Block known-dangerous patterns, allow most sites
    Moderate,
    /// Allow everything, log suspicious activity
    Permissive,
}

// ──── Domain Rules ─────────────────────────────────────────────────────────

/// A single domain rule
#[derive(Debug, Clone)]
pub struct DomainRule {
    /// Domain pattern (exact match or wildcard prefix *.example.com)
    pub pattern: String,
    /// Allow or deny
    pub allow: bool,
}

impl DomainRule {
    pub fn allow(pattern: &str) -> Self {
        Self { pattern: String::from(pattern), allow: true }
    }

    pub fn deny(pattern: &str) -> Self {
        Self { pattern: String::from(pattern), allow: false }
    }

    /// Check if this rule matches a domain
    pub fn matches(&self, domain: &str) -> bool {
        if self.pattern.starts_with("*.") {
            // Wildcard: *.example.com matches sub.example.com, example.com
            let suffix = &self.pattern[1..]; // .example.com
            domain.ends_with(suffix) || domain == &self.pattern[2..]
        } else {
            domain == self.pattern
        }
    }
}

// ──── Content Type Policy ──────────────────────────────────────────────────

/// Allowed content types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentFilter {
    /// Allow HTML pages
    AllowHtml,
    /// Allow CSS stylesheets
    AllowCss,
    /// Allow images (png, jpg, gif, svg, webp, ico)
    AllowImages,
    /// Allow JavaScript (⚠ requires JS sandbox)
    AllowJavaScript,
    /// Allow JSON/API responses
    AllowJson,
    /// Allow plain text
    AllowText,
    /// Allow fonts (woff, woff2, ttf)
    AllowFonts,
}

// ──── Port Policy ──────────────────────────────────────────────────────────

/// Blocked ports (security-sensitive)
const BLOCKED_PORTS: &[u16] = &[
    21,   // FTP
    22,   // SSH
    23,   // Telnet
    25,   // SMTP
    110,  // POP3
    143,  // IMAP
    445,  // SMB
    3306, // MySQL
    3389, // RDP
    5432, // PostgreSQL
    5900, // VNC
    6379, // Redis
    27017, // MongoDB
];

// ──── Dangerous URL Patterns ───────────────────────────────────────────────

/// URL patterns that should always be blocked
const DANGEROUS_PATTERNS: &[&str] = &[
    "file://",
    "javascript:",
    "data:text/html",
    "blob:",
    "about:",
    "localhost",
    "127.0.0.1",
    "0.0.0.0",
    "[::]",
    "[::1]",
    "169.254.",     // link-local
    "10.",          // private class A
    "192.168.",     // private class C
    "172.16.",      // private class B (simplified)
];

// ──── URL Sanitization ─────────────────────────────────────────────────────

/// Check URL for dangerous patterns
fn is_dangerous_url(url: &str) -> Option<&'static str> {
    let lower = url.to_ascii_lowercase();
    for &pat in DANGEROUS_PATTERNS {
        if lower.contains(pat) {
            return Some(pat);
        }
    }
    // Check for IP-based URLs (potential SSRF)
    // Simple heuristic: after "://", if there's a digit before any dot
    if let Some(host_start) = lower.find("://") {
        let after = &lower[host_start + 3..];
        if let Some(first) = after.chars().next() {
            if first.is_ascii_digit() {
                // Could be an IP address — only block if it's private
                // Public IPs are ok
                if after.starts_with("10.") || after.starts_with("192.168.") 
                    || after.starts_with("172.16.") || after.starts_with("127.")
                    || after.starts_with("0.") {
                    return Some("private IP address");
                }
            }
        }
    }
    None
}

/// Extract domain from URL
pub fn extract_domain(url: &str) -> Option<&str> {
    let without_scheme = if let Some(pos) = url.find("://") {
        &url[pos + 3..]
    } else {
        url
    };
    // Remove path, query, fragment
    let host = without_scheme.split('/').next()?;
    // Remove port
    let domain = if host.contains(':') {
        host.split(':').next()?
    } else {
        host
    };
    // Remove userinfo (user:pass@)
    let domain = if domain.contains('@') {
        domain.split('@').last()?
    } else {
        domain
    };
    if domain.is_empty() { None } else { Some(domain) }
}

/// Extract port from URL (default 80/443)
fn extract_port(url: &str) -> u16 {
    let lower = url.to_ascii_lowercase();
    let is_https = lower.starts_with("https://");
    let default_port = if is_https { 443 } else { 80 };

    let without_scheme = if let Some(pos) = url.find("://") {
        &url[pos + 3..]
    } else {
        url
    };
    let host = without_scheme.split('/').next().unwrap_or("");
    if let Some(colon) = host.rfind(':') {
        host[colon + 1..].parse().unwrap_or(default_port)
    } else {
        default_port
    }
}

// ──── Main Policy Engine ───────────────────────────────────────────────────

/// Complete sandbox policy configuration
#[derive(Debug, Clone)]
pub struct SandboxPolicy {
    pub preset: PolicyPreset,
    /// Domain rules (evaluated in order, first match wins)
    pub domain_rules: Vec<DomainRule>,
    /// Allowed content types
    pub allowed_content: Vec<ContentFilter>,
    /// Max requests per minute (0 = unlimited)
    pub rate_limit_per_min: u32,
    /// Max response body size in bytes
    pub max_response_bytes: usize,
    /// Block all JavaScript execution
    pub block_javascript: bool,
    /// Block inline scripts in HTML
    pub block_inline_scripts: bool,
    /// Block external script loading
    pub block_external_scripts: bool,
    /// Allow redirects
    pub allow_redirects: bool,
    /// Max redirect depth
    pub max_redirect_depth: u8,
    /// Block private/internal IPs (SSRF protection)
    pub block_private_ips: bool,
    /// Log all requests (even allowed)
    pub log_all: bool,
    /// Custom blocked URL substrings
    pub blocked_substrings: Vec<String>,
}

impl SandboxPolicy {
    /// Create policy from a preset
    pub fn from_preset(preset: PolicyPreset) -> Self {
        match preset {
            PolicyPreset::Strict => Self::strict(),
            PolicyPreset::Moderate => Self::moderate(),
            PolicyPreset::Permissive => Self::permissive(),
        }
    }

    /// Strict: only allowlisted domains, no JS, no redirects off-domain
    fn strict() -> Self {
        Self {
            preset: PolicyPreset::Strict,
            domain_rules: Vec::new(), // empty = deny all until user adds allows
            allowed_content: alloc::vec![
                ContentFilter::AllowHtml,
                ContentFilter::AllowCss,
                ContentFilter::AllowImages,
                ContentFilter::AllowText,
            ],
            rate_limit_per_min: 30,
            max_response_bytes: 512 * 1024, // 512 KB
            block_javascript: true,
            block_inline_scripts: true,
            block_external_scripts: true,
            allow_redirects: true,
            max_redirect_depth: 2,
            block_private_ips: true,
            log_all: true,
            blocked_substrings: Vec::new(),
        }
    }

    /// Moderate: block known-bad, allow most sites, JS sandboxed
    fn moderate() -> Self {
        Self {
            preset: PolicyPreset::Moderate,
            domain_rules: alloc::vec![
                // Block known trackers/ad networks by default
                DomainRule::deny("*.doubleclick.net"),
                DomainRule::deny("*.googlesyndication.com"),
                DomainRule::deny("*.analytics.google.com"),
                DomainRule::deny("*.facebook.com"),
                DomainRule::deny("*.fbcdn.net"),
            ],
            allowed_content: alloc::vec![
                ContentFilter::AllowHtml,
                ContentFilter::AllowCss,
                ContentFilter::AllowImages,
                ContentFilter::AllowJson,
                ContentFilter::AllowText,
                ContentFilter::AllowJavaScript,
                ContentFilter::AllowFonts,
            ],
            rate_limit_per_min: 60,
            max_response_bytes: 1024 * 1024, // 1 MB
            block_javascript: false,
            block_inline_scripts: false,
            block_external_scripts: false,
            allow_redirects: true,
            max_redirect_depth: 5,
            block_private_ips: true,
            log_all: false,
            blocked_substrings: Vec::new(),
        }
    }

    /// Permissive: allow everything, log suspicious activity
    fn permissive() -> Self {
        Self {
            preset: PolicyPreset::Permissive,
            domain_rules: Vec::new(), // no rules = allow all
            allowed_content: alloc::vec![
                ContentFilter::AllowHtml,
                ContentFilter::AllowCss,
                ContentFilter::AllowImages,
                ContentFilter::AllowJson,
                ContentFilter::AllowText,
                ContentFilter::AllowJavaScript,
                ContentFilter::AllowFonts,
            ],
            rate_limit_per_min: 120,
            max_response_bytes: 4 * 1024 * 1024, // 4 MB
            block_javascript: false,
            block_inline_scripts: false,
            block_external_scripts: false,
            allow_redirects: true,
            max_redirect_depth: 10,
            block_private_ips: true, // always block SSRF
            log_all: false,
            blocked_substrings: Vec::new(),
        }
    }

    /// Add a domain allow rule
    pub fn allow_domain(&mut self, pattern: &str) {
        self.domain_rules.insert(0, DomainRule::allow(pattern));
    }

    /// Add a domain deny rule
    pub fn deny_domain(&mut self, pattern: &str) {
        self.domain_rules.insert(0, DomainRule::deny(pattern));
    }

    /// Evaluate a URL against this policy
    pub fn evaluate_url(&self, url: &str) -> PolicyVerdict {
        // Step 1: Always block dangerous URL schemes
        if let Some(pattern) = is_dangerous_url(url) {
            return PolicyVerdict::Deny(format!("blocked dangerous pattern: {}", pattern));
        }

        // Step 2: Check port
        let port = extract_port(url);
        if BLOCKED_PORTS.contains(&port) {
            return PolicyVerdict::Deny(format!("blocked port: {}", port));
        }

        // Step 3: Check custom blocked substrings
        let lower = url.to_ascii_lowercase();
        for blocked in &self.blocked_substrings {
            if lower.contains(&blocked.to_ascii_lowercase()) {
                return PolicyVerdict::Deny(format!("blocked pattern: {}", blocked));
            }
        }

        // Step 4: Extract domain and check rules
        let domain = match extract_domain(url) {
            Some(d) => d,
            None => return PolicyVerdict::Deny(String::from("cannot extract domain")),
        };

        // Step 5: Check domain rules (first match wins)
        for rule in &self.domain_rules {
            if rule.matches(domain) {
                if rule.allow {
                    return if self.log_all { PolicyVerdict::Log } else { PolicyVerdict::Allow };
                } else {
                    return PolicyVerdict::Deny(format!("domain blocked: {}", domain));
                }
            }
        }

        // Step 6: Default behavior depends on preset
        match self.preset {
            PolicyPreset::Strict => {
                // Strict: deny unless explicitly allowed
                PolicyVerdict::Deny(format!("domain not in allowlist: {}", domain))
            },
            PolicyPreset::Moderate => {
                // Moderate: allow unless explicitly denied
                if self.log_all { PolicyVerdict::Log } else { PolicyVerdict::Allow }
            },
            PolicyPreset::Permissive => {
                PolicyVerdict::Allow
            },
        }
    }

    /// Check if JavaScript is allowed
    pub fn js_allowed(&self) -> bool {
        !self.block_javascript
    }

    /// Check if a content type is allowed
    pub fn content_type_allowed(&self, content_type: &str) -> bool {
        let ct = content_type.to_ascii_lowercase();
        for filter in &self.allowed_content {
            match filter {
                ContentFilter::AllowHtml => if ct.contains("text/html") || ct.contains("application/xhtml") { return true; },
                ContentFilter::AllowCss => if ct.contains("text/css") { return true; },
                ContentFilter::AllowImages => if ct.contains("image/") { return true; },
                ContentFilter::AllowJavaScript => if ct.contains("javascript") || ct.contains("ecmascript") { return true; },
                ContentFilter::AllowJson => if ct.contains("json") || ct.contains("application/xml") { return true; },
                ContentFilter::AllowText => if ct.contains("text/plain") || ct.contains("text/") { return true; },
                ContentFilter::AllowFonts => if ct.contains("font/") || ct.contains("woff") { return true; },
            }
        }
        false
    }

    /// Display policy summary
    pub fn summary(&self) -> String {
        let allows: usize = self.domain_rules.iter().filter(|r| r.allow).count();
        let denies: usize = self.domain_rules.iter().filter(|r| !r.allow).count();
        format!(
            "Policy: {:?}\n  Domain rules: {} allow, {} deny\n  Rate limit: {}/min\n  Max response: {} KB\n  JS: {}\n  Redirects: {} (max depth {})\n  SSRF protection: {}\n  Log all: {}",
            self.preset,
            allows, denies,
            self.rate_limit_per_min,
            self.max_response_bytes / 1024,
            if self.block_javascript { "BLOCKED" } else { "sandboxed" },
            if self.allow_redirects { "allowed" } else { "blocked" },
            self.max_redirect_depth,
            if self.block_private_ips { "ON" } else { "OFF" },
            if self.log_all { "yes" } else { "no" },
        )
    }
}
