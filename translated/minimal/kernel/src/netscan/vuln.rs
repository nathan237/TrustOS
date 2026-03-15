








use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    V,
    Eg,
    Bc,
    Ao,
    Aj,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::V => "INFO",
            Severity::Eg => "LOW",
            Severity::Bc => "MEDIUM",
            Severity::Ao => "HIGH",
            Severity::Aj => "CRITICAL",
        }
    }
}


#[derive(Debug, Clone)]
pub struct Au {
    pub port: u16,
    pub xi: &'static str,
    pub qj: Severity,
    pub dq: String,
    pub dc: String,
    pub aws: String,
}


pub fn arx(cd: [u8; 4], dkf: &[u16]) -> Vec<Au> {
    let mut nq = Vec::new();

    for &port in dkf {
        match port {
            21 => qyu(cd, &mut nq),
            22 => qzv(cd, &mut nq),
            23 => qzx(cd, &mut nq),
            25 | 587 => qzu(cd, port, &mut nq),
            53 => qyr(cd, &mut nq),
            80 | 8080 | 8000 | 8008 => qyy(cd, port, &mut nq),
            110 | 995 => nq.push(Au {
                port, xi: "pop3", qj: Severity::V,
                dq: String::from("POP3 service detected"),
                dc: String::from("POP3 mail service is listening"),
                aws: String::from("Ensure POP3S (SSL) is used"),
            }),
            139 | 445 => qzt(cd, port, &mut nq),
            443 | 8443 => qyz(cd, port, &mut nq),
            1433 => qzf(cd, &mut nq),
            3306 => qzg(cd, &mut nq),
            3389 => qzn(cd, &mut nq),
            5432 => qzm(cd, &mut nq),
            5900 => qzz(cd, &mut nq),
            6379 => qzo(cd, &mut nq),
            27017 => qze(cd, &mut nq),
            _ => {
                
                nq.push(Au {
                    port,
                    xi: super::fui(port),
                    qj: Severity::V,
                    dq: format!("Open port {}", port),
                    dc: format!("Service {} is listening on port {}", super::fui(port), port),
                    aws: String::from("Verify this service is intentionally exposed"),
                });
            }
        }
    }

    nq
}

fn gvc(cd: [u8; 4], port: u16) -> Option<String> {
    super::banner::ern(cd, port, 2000).map(|o| o.banner)
}

fn qyu(cd: [u8; 4], nq: &mut Vec<Au>) {
    if let Some(banner) = gvc(cd, 21) {
        nq.push(Au {
            port: 21, xi: "ftp", qj: Severity::Bc,
            dq: String::from("FTP service exposed"),
            dc: format!("FTP banner: {}", banner),
            aws: String::from("Use SFTP instead of FTP. Disable anonymous login."),
        });

        if banner.avd().contains("anonymous") {
            nq.push(Au {
                port: 21, xi: "ftp", qj: Severity::Ao,
                dq: String::from("FTP anonymous access allowed"),
                dc: String::from("Server may allow anonymous FTP login"),
                aws: String::from("Disable anonymous FTP access"),
            });
        }
    } else {
        nq.push(Au {
            port: 21, xi: "ftp", qj: Severity::Bc,
            dq: String::from("FTP service detected"),
            dc: String::from("FTP service is listening (no banner)"),
            aws: String::from("Consider using SFTP instead"),
        });
    }
}

fn qzv(cd: [u8; 4], nq: &mut Vec<Au>) {
    if let Some(banner) = gvc(cd, 22) {
        let qj = if banner.contains("SSH-1") {
            Severity::Aj
        } else {
            Severity::V
        };

        let dq = if banner.contains("SSH-1") {
            String::from("SSHv1 protocol detected (INSECURE)")
        } else {
            String::from("SSH service detected")
        };

        let hxf = if banner.contains("SSH-1") {
            String::from("Upgrade to SSHv2 immediately — SSHv1 has known vulnerabilities")
        } else {
            String::from("Ensure key-based auth is used, disable password auth")
        };

        nq.push(Au {
            port: 22, xi: "ssh", qj,
            dq,
            dc: format!("SSH banner: {}", banner),
            aws: hxf,
        });
    }
}

fn qzx(cd: [u8; 4], nq: &mut Vec<Au>) {
    nq.push(Au {
        port: 23, xi: "telnet", qj: Severity::Aj,
        dq: String::from("Telnet service exposed (INSECURE)"),
        dc: String::from("Telnet transmits credentials in cleartext"),
        aws: String::from("Replace with SSH. Disable Telnet immediately."),
    });
}

fn qzu(cd: [u8; 4], port: u16, nq: &mut Vec<Au>) {
    if let Some(banner) = gvc(cd, port) {
        nq.push(Au {
            port, xi: "smtp", qj: Severity::Bc,
            dq: String::from("SMTP service exposed"),
            dc: format!("SMTP banner: {}", banner),
            aws: String::from("Ensure STARTTLS is required. Check for open relay."),
        });
    }
}

fn qyr(cd: [u8; 4], nq: &mut Vec<Au>) {
    nq.push(Au {
        port: 53, xi: "dns", qj: Severity::Eg,
        dq: String::from("DNS service exposed"),
        dc: String::from("DNS server is publicly accessible"),
        aws: String::from("Restrict DNS access. Disable zone transfers to unauthorized hosts."),
    });
}

fn qyy(cd: [u8; 4], port: u16, nq: &mut Vec<Au>) {
    if let Some(banner) = gvc(cd, port) {
        let qj = if banner.avd().contains("apache/2.2")
            || banner.avd().contains("apache/2.0") {
            Severity::Ao
        } else {
            Severity::V
        };

        nq.push(Au {
            port, xi: "http", qj,
            dq: String::from("HTTP service detected (unencrypted)"),
            dc: format!("Server: {}", banner),
            aws: String::from("Use HTTPS. Check for sensitive info exposure."),
        });

        
        if banner.avd().contains("server:") {
            nq.push(Au {
                port, xi: "http", qj: Severity::Eg,
                dq: String::from("Server version disclosure"),
                dc: format!("Server header reveals version: {}", banner),
                aws: String::from("Configure server to suppress version headers"),
            });
        }
    }
}

fn qyz(cd: [u8; 4], port: u16, nq: &mut Vec<Au>) {
    nq.push(Au {
        port, xi: "https", qj: Severity::V,
        dq: String::from("HTTPS service detected"),
        dc: String::from("TLS-encrypted web service"),
        aws: String::from("Verify TLS 1.2+ is enforced. Check certificate validity."),
    });
}

fn qzt(cd: [u8; 4], port: u16, nq: &mut Vec<Au>) {
    nq.push(Au {
        port, xi: "smb", qj: Severity::Ao,
        dq: String::from("SMB service exposed"),
        dc: format!("SMB (port {}) is accessible — potential target for EternalBlue", port),
        aws: String::from("Restrict SMB access to trusted networks. Ensure SMBv1 is disabled. Apply MS17-010 patch."),
    });
}

fn qzf(cd: [u8; 4], nq: &mut Vec<Au>) {
    nq.push(Au {
        port: 1433, xi: "ms-sql", qj: Severity::Ao,
        dq: String::from("MS-SQL database exposed"),
        dc: String::from("Microsoft SQL Server is directly accessible"),
        aws: String::from("Restrict access to application servers only. Use Windows Auth."),
    });
}

fn qzg(cd: [u8; 4], nq: &mut Vec<Au>) {
    if let Some(banner) = gvc(cd, 3306) {
        nq.push(Au {
            port: 3306, xi: "mysql", qj: Severity::Ao,
            dq: String::from("MySQL database exposed"),
            dc: format!("MySQL version: {}", banner),
            aws: String::from("Restrict MySQL to localhost/internal network. Disable remote root login."),
        });
    } else {
        nq.push(Au {
            port: 3306, xi: "mysql", qj: Severity::Ao,
            dq: String::from("MySQL database exposed"),
            dc: String::from("MySQL service is directly accessible"),
            aws: String::from("Restrict MySQL to localhost or trusted hosts"),
        });
    }
}

fn qzn(cd: [u8; 4], nq: &mut Vec<Au>) {
    nq.push(Au {
        port: 3389, xi: "rdp", qj: Severity::Ao,
        dq: String::from("RDP service exposed"),
        dc: String::from("Remote Desktop Protocol is accessible — common attack vector"),
        aws: String::from("Use VPN for RDP access. Enable NLA. Apply BlueKeep patches."),
    });
}

fn qzm(cd: [u8; 4], nq: &mut Vec<Au>) {
    nq.push(Au {
        port: 5432, xi: "postgresql", qj: Severity::Ao,
        dq: String::from("PostgreSQL database exposed"),
        dc: String::from("PostgreSQL is directly accessible"),
        aws: String::from("Restrict to trusted hosts via pg_hba.conf. Use SSL."),
    });
}

fn qzz(cd: [u8; 4], nq: &mut Vec<Au>) {
    nq.push(Au {
        port: 5900, xi: "vnc", qj: Severity::Aj,
        dq: String::from("VNC service exposed"),
        dc: String::from("VNC remote desktop is accessible — often weakly protected"),
        aws: String::from("Use SSH tunnel for VNC. Enable strong authentication."),
    });
}

fn qzo(cd: [u8; 4], nq: &mut Vec<Au>) {
    if let Some(banner) = gvc(cd, 6379) {
        let qj = if !banner.avd().contains("noauth") {
            Severity::Aj
        } else {
            Severity::Ao
        };

        nq.push(Au {
            port: 6379, xi: "redis", qj,
            dq: String::from("Redis exposed (likely unauthenticated)"),
            dc: format!("Redis response: {}", banner),
            aws: String::from("Enable AUTH password. Bind to localhost. Use firewall rules."),
        });
    } else {
        nq.push(Au {
            port: 6379, xi: "redis", qj: Severity::Aj,
            dq: String::from("Redis service exposed"),
            dc: String::from("Redis is directly accessible — often without authentication"),
            aws: String::from("Enable authentication. Restrict network access."),
        });
    }
}

fn qze(cd: [u8; 4], nq: &mut Vec<Au>) {
    nq.push(Au {
        port: 27017, xi: "mongodb", qj: Severity::Aj,
        dq: String::from("MongoDB exposed"),
        dc: String::from("MongoDB is directly accessible — common ransomware target"),
        aws: String::from("Enable authentication. Bind to localhost. Use firewall."),
    });
}


pub fn fix(cd: [u8; 4], nq: &[Au]) -> String {
    let mut report = String::new();

    report.t(&format!("Security Scan Report for {}\n", super::aot(cd)));
    report.t(&format!("{}\n\n", "=".afd(50)));

    let cpp = nq.iter().hi(|bb| bb.qj == Severity::Aj).az();
    let afq = nq.iter().hi(|bb| bb.qj == Severity::Ao).az();
    let gmm = nq.iter().hi(|bb| bb.qj == Severity::Bc).az();
    let ail = nq.iter().hi(|bb| bb.qj == Severity::Eg).az();
    let co = nq.iter().hi(|bb| bb.qj == Severity::V).az();

    report.t(&format!("Summary: {} findings\n", nq.len()));
    report.t(&format!("  CRITICAL: {}  HIGH: {}  MEDIUM: {}  LOW: {}  INFO: {}\n\n",
        cpp, afq, gmm, ail, co));

    
    let mut bcs: Vec<&Au> = nq.iter().collect();
    bcs.bxf(|bb| match bb.qj {
        Severity::Aj => 0,
        Severity::Ao => 1,
        Severity::Bc => 2,
        Severity::Eg => 3,
        Severity::V => 4,
    });

    for ghc in bcs {
        report.t(&format!("[{}] Port {}/{} — {}\n",
            ghc.qj.as_str(), ghc.port, ghc.xi, ghc.dq));
        report.t(&format!("  {}\n", ghc.dc));
        report.t(&format!("  Fix: {}\n\n", ghc.aws));
    }

    report
}
