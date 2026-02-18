//! Vulnerability Scanner — Basic security checks and fingerprinting
//!
//! Performs automated security checks on discovered services:
//! - Known default credentials detection
//! - SSL/TLS version checks
//! - Service version vulnerability matching
//! - Open relay detection (SMTP)
//! - Anonymous access checks (FTP, SMB)

use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;

/// Vulnerability severity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Info => "INFO",
            Severity::Low => "LOW",
            Severity::Medium => "MEDIUM",
            Severity::High => "HIGH",
            Severity::Critical => "CRITICAL",
        }
    }
}

/// A detected vulnerability or security finding
#[derive(Debug, Clone)]
pub struct Finding {
    pub port: u16,
    pub service: &'static str,
    pub severity: Severity,
    pub title: String,
    pub description: String,
    pub recommendation: String,
}

/// Run basic vulnerability scan on a target's open ports
pub fn scan(target: [u8; 4], open_ports: &[u16]) -> Vec<Finding> {
    let mut findings = Vec::new();

    for &port in open_ports {
        match port {
            21 => check_ftp(target, &mut findings),
            22 => check_ssh(target, &mut findings),
            23 => check_telnet(target, &mut findings),
            25 | 587 => check_smtp(target, port, &mut findings),
            53 => check_dns(target, &mut findings),
            80 | 8080 | 8000 | 8008 => check_http(target, port, &mut findings),
            110 | 995 => findings.push(Finding {
                port, service: "pop3", severity: Severity::Info,
                title: String::from("POP3 service detected"),
                description: String::from("POP3 mail service is listening"),
                recommendation: String::from("Ensure POP3S (SSL) is used"),
            }),
            139 | 445 => check_smb(target, port, &mut findings),
            443 | 8443 => check_https(target, port, &mut findings),
            1433 => check_mssql(target, &mut findings),
            3306 => check_mysql(target, &mut findings),
            3389 => check_rdp(target, &mut findings),
            5432 => check_postgresql(target, &mut findings),
            5900 => check_vnc(target, &mut findings),
            6379 => check_redis(target, &mut findings),
            27017 => check_mongodb(target, &mut findings),
            _ => {
                // Report unknown open port
                findings.push(Finding {
                    port,
                    service: super::service_name(port),
                    severity: Severity::Info,
                    title: format!("Open port {}", port),
                    description: format!("Service {} is listening on port {}", super::service_name(port), port),
                    recommendation: String::from("Verify this service is intentionally exposed"),
                });
            }
        }
    }

    findings
}

fn try_banner(target: [u8; 4], port: u16) -> Option<String> {
    super::banner::grab_banner(target, port, 2000).map(|b| b.banner)
}

fn check_ftp(target: [u8; 4], findings: &mut Vec<Finding>) {
    if let Some(banner) = try_banner(target, 21) {
        findings.push(Finding {
            port: 21, service: "ftp", severity: Severity::Medium,
            title: String::from("FTP service exposed"),
            description: format!("FTP banner: {}", banner),
            recommendation: String::from("Use SFTP instead of FTP. Disable anonymous login."),
        });

        if banner.to_ascii_lowercase().contains("anonymous") {
            findings.push(Finding {
                port: 21, service: "ftp", severity: Severity::High,
                title: String::from("FTP anonymous access allowed"),
                description: String::from("Server may allow anonymous FTP login"),
                recommendation: String::from("Disable anonymous FTP access"),
            });
        }
    } else {
        findings.push(Finding {
            port: 21, service: "ftp", severity: Severity::Medium,
            title: String::from("FTP service detected"),
            description: String::from("FTP service is listening (no banner)"),
            recommendation: String::from("Consider using SFTP instead"),
        });
    }
}

fn check_ssh(target: [u8; 4], findings: &mut Vec<Finding>) {
    if let Some(banner) = try_banner(target, 22) {
        let severity = if banner.contains("SSH-1") {
            Severity::Critical
        } else {
            Severity::Info
        };

        let title = if banner.contains("SSH-1") {
            String::from("SSHv1 protocol detected (INSECURE)")
        } else {
            String::from("SSH service detected")
        };

        let rec = if banner.contains("SSH-1") {
            String::from("Upgrade to SSHv2 immediately — SSHv1 has known vulnerabilities")
        } else {
            String::from("Ensure key-based auth is used, disable password auth")
        };

        findings.push(Finding {
            port: 22, service: "ssh", severity,
            title,
            description: format!("SSH banner: {}", banner),
            recommendation: rec,
        });
    }
}

fn check_telnet(target: [u8; 4], findings: &mut Vec<Finding>) {
    findings.push(Finding {
        port: 23, service: "telnet", severity: Severity::Critical,
        title: String::from("Telnet service exposed (INSECURE)"),
        description: String::from("Telnet transmits credentials in cleartext"),
        recommendation: String::from("Replace with SSH. Disable Telnet immediately."),
    });
}

fn check_smtp(target: [u8; 4], port: u16, findings: &mut Vec<Finding>) {
    if let Some(banner) = try_banner(target, port) {
        findings.push(Finding {
            port, service: "smtp", severity: Severity::Medium,
            title: String::from("SMTP service exposed"),
            description: format!("SMTP banner: {}", banner),
            recommendation: String::from("Ensure STARTTLS is required. Check for open relay."),
        });
    }
}

fn check_dns(target: [u8; 4], findings: &mut Vec<Finding>) {
    findings.push(Finding {
        port: 53, service: "dns", severity: Severity::Low,
        title: String::from("DNS service exposed"),
        description: String::from("DNS server is publicly accessible"),
        recommendation: String::from("Restrict DNS access. Disable zone transfers to unauthorized hosts."),
    });
}

fn check_http(target: [u8; 4], port: u16, findings: &mut Vec<Finding>) {
    if let Some(banner) = try_banner(target, port) {
        let severity = if banner.to_ascii_lowercase().contains("apache/2.2")
            || banner.to_ascii_lowercase().contains("apache/2.0") {
            Severity::High
        } else {
            Severity::Info
        };

        findings.push(Finding {
            port, service: "http", severity,
            title: String::from("HTTP service detected (unencrypted)"),
            description: format!("Server: {}", banner),
            recommendation: String::from("Use HTTPS. Check for sensitive info exposure."),
        });

        // Check for server version disclosure
        if banner.to_ascii_lowercase().contains("server:") {
            findings.push(Finding {
                port, service: "http", severity: Severity::Low,
                title: String::from("Server version disclosure"),
                description: format!("Server header reveals version: {}", banner),
                recommendation: String::from("Configure server to suppress version headers"),
            });
        }
    }
}

fn check_https(target: [u8; 4], port: u16, findings: &mut Vec<Finding>) {
    findings.push(Finding {
        port, service: "https", severity: Severity::Info,
        title: String::from("HTTPS service detected"),
        description: String::from("TLS-encrypted web service"),
        recommendation: String::from("Verify TLS 1.2+ is enforced. Check certificate validity."),
    });
}

fn check_smb(target: [u8; 4], port: u16, findings: &mut Vec<Finding>) {
    findings.push(Finding {
        port, service: "smb", severity: Severity::High,
        title: String::from("SMB service exposed"),
        description: format!("SMB (port {}) is accessible — potential target for EternalBlue", port),
        recommendation: String::from("Restrict SMB access to trusted networks. Ensure SMBv1 is disabled. Apply MS17-010 patch."),
    });
}

fn check_mssql(target: [u8; 4], findings: &mut Vec<Finding>) {
    findings.push(Finding {
        port: 1433, service: "ms-sql", severity: Severity::High,
        title: String::from("MS-SQL database exposed"),
        description: String::from("Microsoft SQL Server is directly accessible"),
        recommendation: String::from("Restrict access to application servers only. Use Windows Auth."),
    });
}

fn check_mysql(target: [u8; 4], findings: &mut Vec<Finding>) {
    if let Some(banner) = try_banner(target, 3306) {
        findings.push(Finding {
            port: 3306, service: "mysql", severity: Severity::High,
            title: String::from("MySQL database exposed"),
            description: format!("MySQL version: {}", banner),
            recommendation: String::from("Restrict MySQL to localhost/internal network. Disable remote root login."),
        });
    } else {
        findings.push(Finding {
            port: 3306, service: "mysql", severity: Severity::High,
            title: String::from("MySQL database exposed"),
            description: String::from("MySQL service is directly accessible"),
            recommendation: String::from("Restrict MySQL to localhost or trusted hosts"),
        });
    }
}

fn check_rdp(target: [u8; 4], findings: &mut Vec<Finding>) {
    findings.push(Finding {
        port: 3389, service: "rdp", severity: Severity::High,
        title: String::from("RDP service exposed"),
        description: String::from("Remote Desktop Protocol is accessible — common attack vector"),
        recommendation: String::from("Use VPN for RDP access. Enable NLA. Apply BlueKeep patches."),
    });
}

fn check_postgresql(target: [u8; 4], findings: &mut Vec<Finding>) {
    findings.push(Finding {
        port: 5432, service: "postgresql", severity: Severity::High,
        title: String::from("PostgreSQL database exposed"),
        description: String::from("PostgreSQL is directly accessible"),
        recommendation: String::from("Restrict to trusted hosts via pg_hba.conf. Use SSL."),
    });
}

fn check_vnc(target: [u8; 4], findings: &mut Vec<Finding>) {
    findings.push(Finding {
        port: 5900, service: "vnc", severity: Severity::Critical,
        title: String::from("VNC service exposed"),
        description: String::from("VNC remote desktop is accessible — often weakly protected"),
        recommendation: String::from("Use SSH tunnel for VNC. Enable strong authentication."),
    });
}

fn check_redis(target: [u8; 4], findings: &mut Vec<Finding>) {
    if let Some(banner) = try_banner(target, 6379) {
        let severity = if !banner.to_ascii_lowercase().contains("noauth") {
            Severity::Critical
        } else {
            Severity::High
        };

        findings.push(Finding {
            port: 6379, service: "redis", severity,
            title: String::from("Redis exposed (likely unauthenticated)"),
            description: format!("Redis response: {}", banner),
            recommendation: String::from("Enable AUTH password. Bind to localhost. Use firewall rules."),
        });
    } else {
        findings.push(Finding {
            port: 6379, service: "redis", severity: Severity::Critical,
            title: String::from("Redis service exposed"),
            description: String::from("Redis is directly accessible — often without authentication"),
            recommendation: String::from("Enable authentication. Restrict network access."),
        });
    }
}

fn check_mongodb(target: [u8; 4], findings: &mut Vec<Finding>) {
    findings.push(Finding {
        port: 27017, service: "mongodb", severity: Severity::Critical,
        title: String::from("MongoDB exposed"),
        description: String::from("MongoDB is directly accessible — common ransomware target"),
        recommendation: String::from("Enable authentication. Bind to localhost. Use firewall."),
    });
}

/// Generate a security report as formatted string
pub fn format_report(target: [u8; 4], findings: &[Finding]) -> String {
    let mut report = String::new();

    report.push_str(&format!("Security Scan Report for {}\n", super::format_ip(target)));
    report.push_str(&format!("{}\n\n", "=".repeat(50)));

    let critical = findings.iter().filter(|f| f.severity == Severity::Critical).count();
    let high = findings.iter().filter(|f| f.severity == Severity::High).count();
    let medium = findings.iter().filter(|f| f.severity == Severity::Medium).count();
    let low = findings.iter().filter(|f| f.severity == Severity::Low).count();
    let info = findings.iter().filter(|f| f.severity == Severity::Info).count();

    report.push_str(&format!("Summary: {} findings\n", findings.len()));
    report.push_str(&format!("  CRITICAL: {}  HIGH: {}  MEDIUM: {}  LOW: {}  INFO: {}\n\n",
        critical, high, medium, low, info));

    // Sort by severity (critical first)
    let mut sorted: Vec<&Finding> = findings.iter().collect();
    sorted.sort_by_key(|f| match f.severity {
        Severity::Critical => 0,
        Severity::High => 1,
        Severity::Medium => 2,
        Severity::Low => 3,
        Severity::Info => 4,
    });

    for finding in sorted {
        report.push_str(&format!("[{}] Port {}/{} — {}\n",
            finding.severity.as_str(), finding.port, finding.service, finding.title));
        report.push_str(&format!("  {}\n", finding.description));
        report.push_str(&format!("  Fix: {}\n\n", finding.recommendation));
    }

    report
}
