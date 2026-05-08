












use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};


static DG_: AtomicBool = AtomicBool::new(false);

static QH_: AtomicU32 = AtomicU32::new(8080);

static QB_: AtomicU64 = AtomicU64::new(0);


pub fn is_running() -> bool {
    DG_.load(Ordering::SeqCst)
}


pub fn get_stats() -> (u16, u64, bool) {
    (
        QH_.load(Ordering::SeqCst) as u16,
        QB_.load(Ordering::SeqCst),
        is_running(),
    )
}



pub fn start(port: u16, max_requests: u32) {
    if DG_.load(Ordering::SeqCst) {
        crate::n!(crate::framebuffer::D_, "HTTP server already running");
        return;
    }

    QH_.store(port as u32, Ordering::SeqCst);
    DG_.store(true, Ordering::SeqCst);
    QB_.store(0, Ordering::SeqCst);

    
    let wj = crate::network::rd()
        .map(|(ip, _, _)| { let b = ip.as_bytes(); format!("{}.{}.{}.{}", b[0], b[1], b[2], b[3]) })
        .unwrap_or_else(|| String::from("0.0.0.0"));

    crate::n!(crate::framebuffer::G_,
        "TrustOS HTTP Server v1.0");
    crate::println!("Listening on http://{}:{}", wj, port);
    crate::println!("Press Ctrl+C or run 'httpd stop' to stop");
    crate::println!();

    
    crate::netstack::tcp::etd(port, 8);

    let mut guc: u32 = 0;
    let jm = if max_requests == 0 { u32::MAX } else { max_requests };

    
    while DG_.load(Ordering::SeqCst) && guc < jm {
        crate::netstack::poll();

        
        if let Some((src_port, tn, remote_port)) =
            crate::netstack::tcp::eew(port)
        {
            let remote = format!("{}.{}.{}.{}", tn[0], tn[1], tn[2], tn[3]);
            crate::serial_println!("[HTTPD] Connection from {}:{}", remote, remote_port);

            
            let request = oda(tn, port, src_port, 3000);

            if !request.is_empty() {
                
                let (aui, path) = nqz(&request);
                crate::println!("{} {} — {}:{}", aui, path, remote, remote_port);

                
                let fa = oih(&aui, &path);

                
                let _ = crate::netstack::tcp::cqj(tn, remote_port, src_port, fa.as_bytes());

                
                for _ in 0..50_000 { core::hint::spin_loop(); }

                guc += 1;
                QB_.fetch_add(1, Ordering::SeqCst);
            }

            
            let _ = crate::netstack::tcp::ams(tn, remote_port, src_port);
        }

        
        if crate::keyboard::has_input() {
            let key = crate::keyboard::kr();
            if key == Some(3) { 
                break;
            }
        }

        
        crate::arch::acb();
    }

    
    crate::netstack::tcp::gwj(port);
    DG_.store(false, Ordering::SeqCst);
    crate::println!();
    crate::n!(crate::framebuffer::C_,
        "Server stopped. {} requests served.", guc);
}


pub fn stop() {
    DG_.store(false, Ordering::SeqCst);
}


fn oda(tn: [u8; 4], cmi: u16, src_port: u16, timeout_ms: u32) -> String {
    let mut data = Vec::new();
    let start = crate::logger::eg();

    loop {
        crate::netstack::poll();

        if let Some(df) = crate::netstack::tcp::aus(tn, cmi, src_port) {
            data.extend_from_slice(&df);
            
            if data.windows(4).any(|w| w == b"\r\n\r\n") {
                break;
            }
        }

        if crate::logger::eg().saturating_sub(start) > timeout_ms as u64 {
            break;
        }

        core::hint::spin_loop();
    }

    String::from_utf8_lossy(&data).into_owned()
}


fn nqz(request: &str) -> (String, String) {
    let lwj = request.lines().next().unwrap_or("");
    let au: Vec<&str> = lwj.split_whitespace().collect();
    let aui = String::from(*au.first().unwrap_or(&"GET"));
    let path = String::from(*au.get(1).unwrap_or(&"/"));
    (aui, path)
}


fn oih(aui: &str, path: &str) -> String {
    match path {
        "/" => npm(),
        "/status" => npn(),
        "/api/info" => jwo(),
        "/api/stats" => jwq(),
        "/api/processes" => jwp(),
        "/favicon.ico" => jak(),
        _ if path.starts_with("/files") => npk(path),
        _ => jak(),
    }
}


fn czi(content_type: &str, body: &str) -> String {
    format!(
        "HTTP/1.0 200 OK\r\n\
         Content-Type: {}\r\n\
         Content-Length: {}\r\n\
         Server: TrustOS/0.4.0\r\n\
         Connection: close\r\n\
         \r\n\
         {}",
        content_type, body.len(), body
    )
}


fn jak() -> String {
    let body = "<html><head><title>404</title></head><body>\
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
        body.len(), body
    )
}





fn npm() -> String {
    let bwq = crate::time::uptime_ms() / 1000;
    let aoi = bwq / 3600;
    let acf = (bwq % 3600) / 60;
    let im = bwq % 60;

    let (av, used) = crate::memory::frame::stats();
    let bnn = (av * 4096) / (1024 * 1024);
    let haw = (used * 4096) / (1024 * 1024);

    let cores = crate::cpu::smp::ail();

    let body = format!(r#"<!DOCTYPE html>
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
</html>"#, cores, bnn, haw, aoi, acf, im);

    czi("text/html; charset=utf-8", &body)
}

fn npn() -> String {
    let bwq = crate::time::uptime_ms() / 1000;
    let (av, used) = crate::memory::frame::stats();
    let free = av - used;
    let cores = crate::cpu::smp::ail();
    let plt = crate::cpu::smp::cpu_count();
    let euu = crate::network::get_stats();
    let dxr = QB_.load(Ordering::Relaxed);

    let mjs = crate::drivers::net::aoh();
    let nii = if mjs { "active" } else { "none" };

    let body = format!(r#"<!DOCTYPE html>
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
        bwq,
        cores, plt,
        av, (av * 4096) / (1024 * 1024),
        used, (used * 4096) / (1024 * 1024),
        free, (free * 4096) / (1024 * 1024),
        nii,
        euu.packets_received, euu.packets_sent,
        euu.bytes_received, euu.bytes_sent,
        dxr,
    );

    czi("text/html; charset=utf-8", &body)
}

fn npk(path: &str) -> String {
    let dqh = if path == "/files" || path == "/files/" {
        "/"
    } else {
        &path[6..] 
    };

    let entries = crate::ramfs::bh(|fs| {
        fs.ls(Some(dqh)).unwrap_or_default()
    });

    let mut rows = String::new();
    for (name, wf, size) in &entries {
        let icon = if *wf == crate::ramfs::FileType::Directory { "d" } else { "-" };
        let link = format!("/files{}{}{}", dqh,
            if dqh.ends_with('/') { "" } else { "/" }, name);
        let href = if *wf == crate::ramfs::FileType::Directory {
            format!("<a href=\"{}\">{} {}/</a>", link, icon, name)
        } else {
            format!("{} {} ({} bytes)", icon, name, size)
        };
        rows.push_str(&format!("<tr><td>{}</td></tr>\n", href));
    }

    let body = format!(r#"<!DOCTYPE html>
<html><head><title>Files: {}</title>
<style>
    body {{ font-family: monospace; background: #0a0a0a; color: #e0e0e0; margin: 40px; }}
    h1 {{ color: #00ff88; }} a {{ color: #4fc3f7; }}
    table {{ border-collapse: collapse; }} td {{ padding: 4px 12px; border-bottom: 1px solid #222; }}
</style></head><body>
<h1>Files: {}</h1>
<p><a href="/">← Home</a> | <a href="/files/">Root</a></p>
<table>{}</table>
</body></html>"#, dqh, dqh, rows);

    czi("text/html; charset=utf-8", &body)
}

fn jwo() -> String {
    let aiz = crate::time::uptime_ms();
    let (av, used) = crate::memory::frame::stats();
    let cores = crate::cpu::smp::ail();

    let body = format!(r#"{{"os":"TrustOS","version":"0.4.0","uptime_ms":{},"cores":{},"memory_total_kb":{},"memory_used_kb":{},"rust":"nightly","arch":"x86_64"}}"#,
        aiz, cores, av * 4, used * 4);

    czi("application/json", &body)
}

fn jwq() -> String {
    let net = crate::network::get_stats();
    let dxr = QB_.load(Ordering::Relaxed);
    let port = QH_.load(Ordering::Relaxed);

    let body = format!(r#"{{"server_port":{},"requests_served":{},"packets_rx":{},"packets_tx":{},"bytes_rx":{},"bytes_tx":{}}}"#,
        port, dxr, net.packets_received, net.packets_sent,
        net.bytes_received, net.bytes_sent);

    czi("application/json", &body)
}

fn jwp() -> String {
    let body = format!(r#"{{"pid":0,"name":"kernel","state":"running","threads":{}}}"#,
        crate::cpu::smp::ail());
    czi("application/json", &body)
}
