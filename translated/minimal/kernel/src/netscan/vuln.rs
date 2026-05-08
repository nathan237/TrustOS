








use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;


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


#[derive(Debug, Clone)]
pub struct Ad {
    pub port: u16,
    pub service: &'static str,
    pub severity: Severity,
    pub title: String,
    pub description: String,
    pub recommendation: String,
}


pub fn scan(target: [u8; 4], bil: &[u16]) -> Vec<Ad> {
    let mut fw = Vec::new();

    for &port in bil {
        match port {
            21 => kiy(target, &mut fw),
            22 => kjv(target, &mut fw),
            23 => kjw(target, &mut fw),
            25 | 587 => kju(target, port, &mut fw),
            53 => kiw(target, &mut fw),
            80 | 8080 | 8000 | 8008 => kjb(target, port, &mut fw),
            110 | 995 => fw.push(Ad {
                port, service: "pop3", severity: Severity::Info,
                title: String::from("POP3 service detected"),
                description: String::from("POP3 mail service is listening"),
                recommendation: String::from("Ensure POP3S (SSL) is used"),
            }),
            139 | 445 => kjt(target, port, &mut fw),
            443 | 8443 => kjc(target, port, &mut fw),
            1433 => kjh(target, &mut fw),
            3306 => kji(target, &mut fw),
            3389 => kjn(target, &mut fw),
            5432 => kjm(target, &mut fw),
            5900 => kjx(target, &mut fw),
            6379 => kjo(target, &mut fw),
            27017 => kjg(target, &mut fw),
            _ => {
                
                fw.push(Ad {
                    port,
                    service: super::cqk(port),
                    severity: Severity::Info,
                    title: format!("Open port {}", port),
                    description: format!("Service {} is listening on port {}", super::cqk(port), port),
                    recommendation: String::from("Verify this service is intentionally exposed"),
                });
            }
        }
    }

    fw
}

fn dfv(target: [u8; 4], port: u16) -> Option<String> {
    super::banner::grab_banner(target, port, 2000).map(|b| b.banner)
}

fn kiy(target: [u8; 4], fw: &mut Vec<Ad>) {
    if let Some(banner) = dfv(target, 21) {
        fw.push(Ad {
            port: 21, service: "ftp", severity: Severity::Medium,
            title: String::from("FTP service exposed"),
            description: format!("FTP banner: {}", banner),
            recommendation: String::from("Use SFTP instead of FTP. Disable anonymous login."),
        });

        if banner.to_ascii_lowercase().contains("anonymous") {
            fw.push(Ad {
                port: 21, service: "ftp", severity: Severity::High,
                title: String::from("FTP anonymous access allowed"),
                description: String::from("Server may allow anonymous FTP login"),
                recommendation: String::from("Disable anonymous FTP access"),
            });
        }
    } else {
        fw.push(Ad {
            port: 21, service: "ftp", severity: Severity::Medium,
            title: String::from("FTP service detected"),
            description: String::from("FTP service is listening (no banner)"),
            recommendation: String::from("Consider using SFTP instead"),
        });
    }
}

fn kjv(target: [u8; 4], fw: &mut Vec<Ad>) {
    if let Some(banner) = dfv(target, 22) {
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

        let dxn = if banner.contains("SSH-1") {
            String::from("Upgrade to SSHv2 immediately — SSHv1 has known vulnerabilities")
        } else {
            String::from("Ensure key-based auth is used, disable password auth")
        };

        fw.push(Ad {
            port: 22, service: "ssh", severity,
            title,
            description: format!("SSH banner: {}", banner),
            recommendation: dxn,
        });
    }
}

fn kjw(target: [u8; 4], fw: &mut Vec<Ad>) {
    fw.push(Ad {
        port: 23, service: "telnet", severity: Severity::Critical,
        title: String::from("Telnet service exposed (INSECURE)"),
        description: String::from("Telnet transmits credentials in cleartext"),
        recommendation: String::from("Replace with SSH. Disable Telnet immediately."),
    });
}

fn kju(target: [u8; 4], port: u16, fw: &mut Vec<Ad>) {
    if let Some(banner) = dfv(target, port) {
        fw.push(Ad {
            port, service: "smtp", severity: Severity::Medium,
            title: String::from("SMTP service exposed"),
            description: format!("SMTP banner: {}", banner),
            recommendation: String::from("Ensure STARTTLS is required. Check for open relay."),
        });
    }
}

fn kiw(target: [u8; 4], fw: &mut Vec<Ad>) {
    fw.push(Ad {
        port: 53, service: "dns", severity: Severity::Low,
        title: String::from("DNS service exposed"),
        description: String::from("DNS server is publicly accessible"),
        recommendation: String::from("Restrict DNS access. Disable zone transfers to unauthorized hosts."),
    });
}

fn kjb(target: [u8; 4], port: u16, fw: &mut Vec<Ad>) {
    if let Some(banner) = dfv(target, port) {
        let severity = if banner.to_ascii_lowercase().contains("apache/2.2")
            || banner.to_ascii_lowercase().contains("apache/2.0") {
            Severity::High
        } else {
            Severity::Info
        };

        fw.push(Ad {
            port, service: "http", severity,
            title: String::from("HTTP service detected (unencrypted)"),
            description: format!("Server: {}", banner),
            recommendation: String::from("Use HTTPS. Check for sensitive info exposure."),
        });

        
        if banner.to_ascii_lowercase().contains("server:") {
            fw.push(Ad {
                port, service: "http", severity: Severity::Low,
                title: String::from("Server version disclosure"),
                description: format!("Server header reveals version: {}", banner),
                recommendation: String::from("Configure server to suppress version headers"),
            });
        }
    }
}

fn kjc(target: [u8; 4], port: u16, fw: &mut Vec<Ad>) {
    fw.push(Ad {
        port, service: "https", severity: Severity::Info,
        title: String::from("HTTPS service detected"),
        description: String::from("TLS-encrypted web service"),
        recommendation: String::from("Verify TLS 1.2+ is enforced. Check certificate validity."),
    });
}

fn kjt(target: [u8; 4], port: u16, fw: &mut Vec<Ad>) {
    fw.push(Ad {
        port, service: "smb", severity: Severity::High,
        title: String::from("SMB service exposed"),
        description: format!("SMB (port {}) is accessible — potential target for EternalBlue", port),
        recommendation: String::from("Restrict SMB access to trusted networks. Ensure SMBv1 is disabled. Apply MS17-010 patch."),
    });
}

fn kjh(target: [u8; 4], fw: &mut Vec<Ad>) {
    fw.push(Ad {
        port: 1433, service: "ms-sql", severity: Severity::High,
        title: String::from("MS-SQL database exposed"),
        description: String::from("Microsoft SQL Server is directly accessible"),
        recommendation: String::from("Restrict access to application servers only. Use Windows Auth."),
    });
}

fn kji(target: [u8; 4], fw: &mut Vec<Ad>) {
    if let Some(banner) = dfv(target, 3306) {
        fw.push(Ad {
            port: 3306, service: "mysql", severity: Severity::High,
            title: String::from("MySQL database exposed"),
            description: format!("MySQL version: {}", banner),
            recommendation: String::from("Restrict MySQL to localhost/internal network. Disable remote root login."),
        });
    } else {
        fw.push(Ad {
            port: 3306, service: "mysql", severity: Severity::High,
            title: String::from("MySQL database exposed"),
            description: String::from("MySQL service is directly accessible"),
            recommendation: String::from("Restrict MySQL to localhost or trusted hosts"),
        });
    }
}

fn kjn(target: [u8; 4], fw: &mut Vec<Ad>) {
    fw.push(Ad {
        port: 3389, service: "rdp", severity: Severity::High,
        title: String::from("RDP service exposed"),
        description: String::from("Remote Desktop Protocol is accessible — common attack vector"),
        recommendation: String::from("Use VPN for RDP access. Enable NLA. Apply BlueKeep patches."),
    });
}

fn kjm(target: [u8; 4], fw: &mut Vec<Ad>) {
    fw.push(Ad {
        port: 5432, service: "postgresql", severity: Severity::High,
        title: String::from("PostgreSQL database exposed"),
        description: String::from("PostgreSQL is directly accessible"),
        recommendation: String::from("Restrict to trusted hosts via pg_hba.conf. Use SSL."),
    });
}

fn kjx(target: [u8; 4], fw: &mut Vec<Ad>) {
    fw.push(Ad {
        port: 5900, service: "vnc", severity: Severity::Critical,
        title: String::from("VNC service exposed"),
        description: String::from("VNC remote desktop is accessible — often weakly protected"),
        recommendation: String::from("Use SSH tunnel for VNC. Enable strong authentication."),
    });
}

fn kjo(target: [u8; 4], fw: &mut Vec<Ad>) {
    if let Some(banner) = dfv(target, 6379) {
        let severity = if !banner.to_ascii_lowercase().contains("noauth") {
            Severity::Critical
        } else {
            Severity::High
        };

        fw.push(Ad {
            port: 6379, service: "redis", severity,
            title: String::from("Redis exposed (likely unauthenticated)"),
            description: format!("Redis response: {}", banner),
            recommendation: String::from("Enable AUTH password. Bind to localhost. Use firewall rules."),
        });
    } else {
        fw.push(Ad {
            port: 6379, service: "redis", severity: Severity::Critical,
            title: String::from("Redis service exposed"),
            description: String::from("Redis is directly accessible — often without authentication"),
            recommendation: String::from("Enable authentication. Restrict network access."),
        });
    }
}

fn kjg(target: [u8; 4], fw: &mut Vec<Ad>) {
    fw.push(Ad {
        port: 27017, service: "mongodb", severity: Severity::Critical,
        title: String::from("MongoDB exposed"),
        description: String::from("MongoDB is directly accessible — common ransomware target"),
        recommendation: String::from("Enable authentication. Bind to localhost. Use firewall."),
    });
}


pub fn format_report(target: [u8; 4], fw: &[Ad]) -> String {
    let mut report = String::new();

    report.push_str(&format!("Security Scan Report for {}\n", super::uw(target)));
    report.push_str(&format!("{}\n\n", "=".repeat(50)));

    let aqb = fw.iter().filter(|f| f.severity == Severity::Critical).count();
    let high = fw.iter().filter(|f| f.severity == Severity::High).count();
    let dbd = fw.iter().filter(|f| f.severity == Severity::Medium).count();
    let low = fw.iter().filter(|f| f.severity == Severity::Low).count();
    let info = fw.iter().filter(|f| f.severity == Severity::Info).count();

    report.push_str(&format!("Summary: {} findings\n", fw.len()));
    report.push_str(&format!("  CRITICAL: {}  HIGH: {}  MEDIUM: {}  LOW: {}  INFO: {}\n\n",
        aqb, high, dbd, low, info));

    
    let mut acq: Vec<&Ad> = fw.iter().collect();
    acq.sort_by_key(|f| match f.severity {
        Severity::Critical => 0,
        Severity::High => 1,
        Severity::Medium => 2,
        Severity::Low => 3,
        Severity::Info => 4,
    });

    for finding in acq {
        report.push_str(&format!("[{}] Port {}/{} — {}\n",
            finding.severity.as_str(), finding.port, finding.service, finding.title));
        report.push_str(&format!("  {}\n", finding.description));
        report.push_str(&format!("  Fix: {}\n\n", finding.recommendation));
    }

    report
}
