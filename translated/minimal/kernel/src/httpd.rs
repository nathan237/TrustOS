












use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};


static CZ_: AtomicBool = AtomicBool::new(false);

static PK_: AtomicU32 = AtomicU32::new(8080);

static PE_: AtomicU64 = AtomicU64::new(0);


pub fn dsi() -> bool {
    CZ_.load(Ordering::SeqCst)
}


pub fn asx() -> (u16, u64, bool) {
    (
        PK_.load(Ordering::SeqCst) as u16,
        PE_.load(Ordering::SeqCst),
        dsi(),
    )
}



pub fn ay(port: u16, ome: u32) {
    if CZ_.load(Ordering::SeqCst) {
        crate::h!(crate::framebuffer::D_, "HTTP server already running");
        return;
    }

    PK_.store(port as u32, Ordering::SeqCst);
    CZ_.store(true, Ordering::SeqCst);
    PE_.store(0, Ordering::SeqCst);

    
    let aro = crate::network::aou()
        .map(|(ip, _, _)| { let o = ip.as_bytes(); format!("{}.{}.{}.{}", o[0], o[1], o[2], o[3]) })
        .unwrap_or_else(|| String::from("0.0.0.0"));

    crate::h!(crate::framebuffer::G_,
        "TrustOS HTTP Server v1.0");
    crate::println!("Listening on http://{}:{}", aro, port);
    crate::println!("Press Ctrl+C or run 'httpd stop' to stop");
    crate::println!();

    
    crate::netstack::tcp::jdt(port, 8);

    let mut mdz: u32 = 0;
    let ul = if ome == 0 { u32::O } else { ome };

    
    while CZ_.load(Ordering::SeqCst) && mdz < ul {
        crate::netstack::poll();

        
        if let Some((ey, ams, bci)) =
            crate::netstack::tcp::iir(port)
        {
            let bwq = format!("{}.{}.{}.{}", ams[0], ams[1], ams[2], ams[3]);
            crate::serial_println!("[HTTPD] Connection from {}:{}", bwq, bci);

            
            let request = vsj(ams, port, ey, 3000);

            if !request.is_empty() {
                
                let (clk, path) = vdi(&request);
                crate::println!("{} {} — {}:{}", clk, path, bwq, bci);

                
                let mk = wai(&clk, &path);

                
                let _ = crate::netstack::tcp::fuf(ams, bci, ey, mk.as_bytes());

                
                for _ in 0..50_000 { core::hint::hc(); }

                mdz += 1;
                PE_.fetch_add(1, Ordering::SeqCst);
            }

            
            let _ = crate::netstack::tcp::bwx(ams, bci, ey);
        }

        
        if crate::keyboard::hmo() {
            let bs = crate::keyboard::xw();
            if bs == Some(3) { 
                break;
            }
        }

        
        crate::arch::bhd();
    }

    
    crate::netstack::tcp::mhr(port);
    CZ_.store(false, Ordering::SeqCst);
    crate::println!();
    crate::h!(crate::framebuffer::C_,
        "Server stopped. {} requests served.", mdz);
}


pub fn qg() {
    CZ_.store(false, Ordering::SeqCst);
}


fn vsj(ams: [u8; 4], fnd: u16, ey: u16, sg: u32) -> String {
    let mut f = Vec::new();
    let ay = crate::logger::lh();

    loop {
        crate::netstack::poll();

        if let Some(jj) = crate::netstack::tcp::cme(ams, fnd, ey) {
            f.bk(&jj);
            
            if f.ee(4).any(|d| d == b"\r\n\r\n") {
                break;
            }
        }

        if crate::logger::lh().ao(ay) > sg as u64 {
            break;
        }

        core::hint::hc();
    }

    String::azw(&f).bkc()
}


fn vdi(request: &str) -> (String, String) {
    let suc = request.ak().next().unwrap_or("");
    let ek: Vec<&str> = suc.ayt().collect();
    let clk = String::from(*ek.fv().unwrap_or(&"GET"));
    let path = String::from(*ek.get(1).unwrap_or(&"/"));
    (clk, path)
}


fn wai(clk: &str, path: &str) -> String {
    match path {
        "/" => vaz(),
        "/status" => vba(),
        "/api/info" => qjg(),
        "/api/stats" => qji(),
        "/api/processes" => qjh(),
        "/favicon.ico" => pcu(),
        _ if path.cj("/files") => vax(path),
        _ => pcu(),
    }
}


fn gjh(ahg: &str, gj: &str) -> String {
    format!(
        "HTTP/1.0 200 OK\r\n\
         Content-Type: {}\r\n\
         Content-Length: {}\r\n\
         Server: TrustOS/0.4.0\r\n\
         Connection: close\r\n\
         \r\n\
         {}",
        ahg, gj.len(), gj
    )
}


fn pcu() -> String {
    let gj = "<html><head><title>404</title></head><body>\
                <h1>404 Not Found</h1><p>The requested resource was not found on TrustOS.</p>\
                <hr><em>TrustOS/0.4.0</em></body></html>";
    format!(
        "HTTP/1.0 404 Not Found\r\n\
         Content-Type: text/html\r\n\
         Content-Length: {}\r\n\
         Server: TrustOS/0.4.0\r\n\
         Connection: close\r\n\
         \r\n\
         {}",
        gj.len(), gj
    )
}





fn vaz() -> String {
    let ekj = crate::time::lc() / 1000;
    let cad = ekj / 3600;
    let bbz = (ekj % 3600) / 60;
    let tv = ekj % 60;

    let (es, mr) = crate::memory::frame::cm();
    let dtd = (es * 4096) / (1024 * 1024);
    let mol = (mr * 4096) / (1024 * 1024);

    let ffw = crate::cpu::smp::boc();

    let gj = format!(r#"<!DOCTYPE html>
<html>
<head>
    <title>TrustOS</title>
    <style>
        body {{ font-family: 'Segoe UI', sans-serif; background: #0a0a0a; color: #e0e0e0; margin: 40px; }}
        h1 {{ color: #00ff88; font-size: 2.5em; }}
        .card {{ background: #1a1a2e; border: 1px solid #333; border-radius: 8px; padding: 20px; margin: 15px 0; }}
        .stat {{ display: inline-block; margin: 10px 20px; }}
        .stat .value {{ font-size: 2em; color: #00ff88; font-weight: bold; }}
        .stat .label {{ color: #888; font-size: 0.9em; }}
        a {{ color: #4fc3f7; text-decoration: none; }}
        a:hover {{ text-decoration: underline; }}
        .nav {{ margin: 20px 0; }}
        .nav a {{ margin-right: 20px; padding: 8px 16px; background: #16213e; border-radius: 4px; }}
        code {{ background: #1a1a2e; padding: 2px 6px; border-radius: 3px; color: #00ff88; }}
        footer {{ margin-top: 40px; color: #555; border-top: 1px solid #333; padding-top: 10px; }}
    </style>
</head>
<body>
    <h1>TrustOS</h1>
    <p><em>Trust the code. Rust is the reason.</em></p>
    
    <div class="nav">
        <a href="/">Home</a>
        <a href="/status">System Status</a>
        <a href="/files/">Browse Files</a>
        <a href="/api/info">API</a>
    </div>

    <div class="card">
        <h2>System Overview</h2>
        <div class="stat"><div class="value">{}</div><div class="label">CPU Cores</div></div>
        <div class="stat"><div class="value">{} MB</div><div class="label">Total RAM</div></div>
        <div class="stat"><div class="value">{} MB</div><div class="label">Used RAM</div></div>
        <div class="stat"><div class="value">{:02}:{:02}:{:02}</div><div class="label">Uptime</div></div>
    </div>

    <div class="card">
        <h2>About</h2>
        <p>TrustOS is a bare-metal operating system written entirely in Rust.</p>
        <ul>
            <li>165,000+ lines of pure Rust — no C, no assembly</li>
            <li>Full TCP/IP networking with TLS 1.3</li>
            <li>Type-1 hypervisor (Intel VT-x / AMD SVM)</li>
            <li>COSMIC desktop environment</li>
            <li>TrustScan security toolkit</li>
            <li>NES + Game Boy emulators</li>
            <li>TrustLang programming language</li>
        </ul>
    </div>

    <div class="card">
        <h2>API Endpoints</h2>
        <ul>
            <li><a href="/api/info">/api/info</a> — System information (JSON)</li>
            <li><a href="/api/stats">/api/stats</a> — Server statistics (JSON)</li>
            <li><a href="/api/processes">/api/processes</a> — Process list (JSON)</li>
        </ul>
    </div>

    <footer>Served by TrustOS HTTP Server v1.0 | Powered by Rust</footer>
</body>
</html>"#, ffw, dtd, mol, cad, bbz, tv);

    gjh("text/html; charset=utf-8", &gj)
}

fn vba() -> String {
    let ekj = crate::time::lc() / 1000;
    let (es, mr) = crate::memory::frame::cm();
    let aez = es - mr;
    let ffw = crate::cpu::smp::boc();
    let xka = crate::cpu::smp::aao();
    let jgp = crate::network::asx();
    let hxn = PE_.load(Ordering::Relaxed);

    let tmu = crate::drivers::net::bzy();
    let usc = if tmu { "active" } else { "none" };

    let gj = format!(r#"<!DOCTYPE html>
<html><head><title>TrustOS Status</title>
<style>
    body {{ font-family: monospace; background: #0a0a0a; color: #e0e0e0; margin: 40px; }}
    h1 {{ color: #00ff88; }} table {{ border-collapse: collapse; width: 100%; }}
    th, td {{ border: 1px solid #333; padding: 8px 12px; text-align: left; }}
    th {{ background: #16213e; color: #4fc3f7; }} tr:hover {{ background: #1a1a2e; }}
    .green {{ color: #00ff88; }} .yellow {{ color: #ffd700; }} .red {{ color: #ff4444; }}
    a {{ color: #4fc3f7; }}
</style></head><body>
<h1>System Status</h1>
<p><a href="/">← Home</a></p>
<table>
<tr><th>Metric</th><th>Value</th></tr>
<tr><td>Uptime</td><td>{} seconds</td></tr>
<tr><td>CPU Cores</td><td class="green">{}/{} online</td></tr>
<tr><td>Memory Total</td><td>{} frames ({} MB)</td></tr>
<tr><td>Memory Used</td><td>{} frames ({} MB)</td></tr>
<tr><td>Memory Free</td><td class="green">{} frames ({} MB)</td></tr>
<tr><td>Network Driver</td><td>{}</td></tr>
<tr><td>Packets RX</td><td>{}</td></tr>
<tr><td>Packets TX</td><td>{}</td></tr>
<tr><td>Bytes RX</td><td>{}</td></tr>
<tr><td>Bytes TX</td><td>{}</td></tr>
<tr><td>HTTP Requests Served</td><td class="green">{}</td></tr>
</table>
</body></html>"#,
        ekj,
        ffw, xka,
        es, (es * 4096) / (1024 * 1024),
        mr, (mr * 4096) / (1024 * 1024),
        aez, (aez * 4096) / (1024 * 1024),
        usc,
        jgp.dub, jgp.egc,
        jgp.cdm, jgp.feb,
        hxn,
    );

    gjh("text/html; charset=utf-8", &gj)
}

fn vax(path: &str) -> String {
    let hko = if path == "/files" || path == "/files/" {
        "/"
    } else {
        &path[6..] 
    };

    let ch = crate::ramfs::fh(|fs| {
        fs.awb(Some(hko)).age()
    });

    let mut lk = String::new();
    for (j, are, aw) in &ch {
        let pa = if *are == crate::ramfs::FileType::K { "d" } else { "-" };
        let arl = format!("/files{}{}{}", hko,
            if hko.pp('/') { "" } else { "/" }, j);
        let cae = if *are == crate::ramfs::FileType::K {
            format!("<a href=\"{}\">{} {}/</a>", arl, pa, j)
        } else {
            format!("{} {} ({} bytes)", pa, j, aw)
        };
        lk.t(&format!("<tr><td>{}</td></tr>\n", cae));
    }

    let gj = format!(r#"<!DOCTYPE html>
<html><head><title>Files: {}</title>
<style>
    body {{ font-family: monospace; background: #0a0a0a; color: #e0e0e0; margin: 40px; }}
    h1 {{ color: #00ff88; }} a {{ color: #4fc3f7; }}
    table {{ border-collapse: collapse; }} td {{ padding: 4px 12px; border-bottom: 1px solid #222; }}
</style></head><body>
<h1>Files: {}</h1>
<p><a href="/">← Home</a> | <a href="/files/">Root</a></p>
<table>{}</table>
</body></html>"#, hko, hko, lk);

    gjh("text/html; charset=utf-8", &gj)
}

fn qjg() -> String {
    let bxp = crate::time::lc();
    let (es, mr) = crate::memory::frame::cm();
    let ffw = crate::cpu::smp::boc();

    let gj = format!(r#"{{"os":"TrustOS","version":"0.4.0","uptime_ms":{},"cores":{},"memory_total_kb":{},"memory_used_kb":{},"rust":"nightly","arch":"x86_64"}}"#,
        bxp, ffw, es * 4, mr * 4);

    gjh("application/json", &gj)
}

fn qji() -> String {
    let net = crate::network::asx();
    let hxn = PE_.load(Ordering::Relaxed);
    let port = PK_.load(Ordering::Relaxed);

    let gj = format!(r#"{{"server_port":{},"requests_served":{},"packets_rx":{},"packets_tx":{},"bytes_rx":{},"bytes_tx":{}}}"#,
        port, hxn, net.dub, net.egc,
        net.cdm, net.feb);

    gjh("application/json", &gj)
}

fn qjh() -> String {
    let gj = format!(r#"{{"pid":0,"name":"kernel","state":"running","threads":{}}}"#,
        crate::cpu::smp::boc());
    gjh("application/json", &gj)
}
