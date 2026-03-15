




use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use crate::framebuffer::{B_, G_, AU_, D_, A_, C_, Q_, CD_, DF_, L_};

pub(super) fn rcs(n: &[&str]) {
    if n.is_empty() {
        crate::println!("TrustOS Web Browser");
        crate::println!("Usage: browse <url>");
        crate::println!("  Example: browse http://example.com");
        crate::println!("  Example: browse http://info.cern.ch");
        crate::println!("");
        crate::println!("Note: Only HTTP is supported (no HTTPS yet)");
        return;
    }
    
    let url = n[0];
    let url = if !url.cj("http://") && !url.cj("https://") {
        alloc::format!("http://{}", url)
    } else {
        String::from(url)
    };
    
    crate::println!("[Browser] Loading {}...", url);
    
    
    let mut browser = crate::browser::Browser::new(800, 600);
    
    match browser.bvn(&url) {
        Ok(()) => {
            crate::h!(B_, "Page loaded successfully!");
            
            
            if let Some(ref doc) = browser.ama {
                if !doc.dq.is_empty() {
                    crate::println!("Title: {}", doc.dq);
                }
                
                crate::println!("");
                
                
                lzb(doc, 0);
            }
        }
        Err(aa) => {
            crate::h!(A_, "Failed to load page: {}", aa);
        }
    }
}





pub(super) fn rda(n: &[&str]) {
    let air = n.fv().hu().unwrap_or("help");
    match air {
        "status" | "info" => {
            if n.len() >= 2 {
                
                let ad: u32 = n[1].parse().unwrap_or(0);
                match crate::sandbox::container::roe(ad) {
                    Some(e) => crate::println!("{}", e),
                    None => crate::h!(A_, "Container #{} not found", ad),
                }
            } else {
                
                crate::println!("{}", crate::sandbox::container::rth());
            }
        }

        "list" | "ls" => {
            let bmm = crate::sandbox::container::ufp();
            if bmm.is_empty() {
                crate::println!("No containers.");
            } else {
                crate::h!(C_, "Web Containers:");
                crate::println!("  {:>4}  {:>10}  {:>3}  {}", "ID", "Health", "Def", "Name");
                crate::println!("  {}", "-".afd(45));
                for (ad, j, arh, txe) in &bmm {
                    let ruz = if *txe { " * " } else { "   " };
                    crate::println!("  {:>4}  {:>10?}  {}  {}", ad, arh, ruz, j);
                }
            }
        }

        "create" => {
            
            let lvc = n.get(1).hu().unwrap_or("default");
            let config = match lvc {
                "secure" => crate::sandbox::container::ContainerConfig::hzi(),
                "dev" => crate::sandbox::container::ContainerConfig::ba(),
                _ => {
                    let mut cfg = crate::sandbox::container::ContainerConfig::default();
                    
                    if lvc != "default" {
                        cfg.j = String::from(lvc);
                    }
                    cfg
                }
            };
            let mut bjs = crate::sandbox::container::CJ_.lock();
            let ad = bjs.nha(config);
            match bjs.pnz(ad) {
                Ok(()) => crate::h!(B_, "Container #{} created and started", ad),
                Err(aa) => crate::h!(A_, "Created #{} but failed to start: {:?}", ad, aa),
            }
        }

        "go" | "navigate" | "open" => {
            
            if n.len() < 2 {
                crate::println!("Usage: container go <url> [container_id]");
                return;
            }
            let url = n[1];
            let kkq: Option<u32> = n.get(2).and_then(|e| e.parse().bq());

            crate::h!(C_, "[Container] Fetching {}...", url);
            let mut bjs = crate::sandbox::container::CJ_.lock();
            match bjs.bvn(kkq, url) {
                Ok(lj) => {
                    crate::h!(B_, "  Status: {} | {} | {} bytes",
                        lj.wt, lj.ahg, lj.gj.len());

                    if lj.oga() {
                        let iyx = lj.hax();
                        let doc = crate::browser::html_parser::due(&iyx);
                        if !doc.dq.is_empty() {
                            crate::println!("  Title: {}", doc.dq);
                        }
                        crate::println!();
                        lzb(&doc, 0);
                    } else {
                        let text = lj.hax();
                        let hvz: String = text.bw().take(2000).collect();
                        crate::println!("{}", hvz);
                        if text.len() > 2000 {
                            crate::println!("  ... ({} bytes total)", text.len());
                        }
                    }
                }
                Err(aa) => {
                    crate::h!(A_, "  Error: {:?}", aa);
                }
            }
        }

        "stop" => {
            if n.len() < 2 {
                crate::println!("Usage: container stop <id>");
                return;
            }
            let ad: u32 = n[1].parse().unwrap_or(0);
            let mut bjs = crate::sandbox::container::CJ_.lock();
            match bjs.wun(ad) {
                Ok(()) => crate::h!(B_, "Container #{} stopped", ad),
                Err(aa) => crate::h!(A_, "Error: {:?}", aa),
            }
        }

        "start" => {
            if n.len() < 2 {
                crate::println!("Usage: container start <id>");
                return;
            }
            let ad: u32 = n[1].parse().unwrap_or(0);
            let mut bjs = crate::sandbox::container::CJ_.lock();
            match bjs.pnz(ad) {
                Ok(()) => crate::h!(B_, "Container #{} started", ad),
                Err(aa) => crate::h!(A_, "Error: {:?}", aa),
            }
        }

        "restart" => {
            if n.len() < 2 {
                crate::println!("Usage: container restart <id>");
                return;
            }
            let ad: u32 = n[1].parse().unwrap_or(0);
            let mut bjs = crate::sandbox::container::CJ_.lock();
            match bjs.vyd(ad) {
                Ok(()) => crate::h!(B_, "Container #{} restarted", ad),
                Err(aa) => crate::h!(A_, "Error: {:?}", aa),
            }
        }

        "destroy" | "rm" => {
            if n.len() < 2 {
                crate::println!("Usage: container destroy <id>");
                return;
            }
            let ad: u32 = n[1].parse().unwrap_or(0);
            let mut bjs = crate::sandbox::container::CJ_.lock();
            match bjs.rwj(ad) {
                Ok(()) => crate::h!(B_, "Container #{} destroyed", ad),
                Err(aa) => crate::h!(A_, "Error: {:?}", aa),
            }
        }

        "allow" => {
            if n.len() < 3 {
                crate::println!("Usage: container allow <id> <domain>");
                return;
            }
            let ad: u32 = n[1].parse().unwrap_or(0);
            let vh = n[2];
            
            let afh = {
                let mut bjs = crate::sandbox::container::CJ_.lock();
                if let Some(container) = bjs.ds(ad) {
                    container.config.gyk.push(String::from(vh));
                    container.afh
                } else {
                    crate::h!(A_, "Container #{} not found", ad);
                    return;
                }
            };
            
            if let Some(ary) = afh {
                let mut aas = crate::sandbox::BD_.lock();
                if let Some(is) = aas.ds(ary) {
                    is.policy.kaf(vh);
                }
            }
            crate::h!(B_, "Domain '{}' allowed in container #{}", vh, ad);
        }

        "deny" | "block" => {
            if n.len() < 3 {
                crate::println!("Usage: container deny <id> <domain>");
                return;
            }
            let ad: u32 = n[1].parse().unwrap_or(0);
            let vh = n[2];
            let afh = {
                let mut bjs = crate::sandbox::container::CJ_.lock();
                if let Some(container) = bjs.ds(ad) {
                    container.config.hav.push(String::from(vh));
                    container.afh
                } else {
                    crate::h!(A_, "Container #{} not found", ad);
                    return;
                }
            };
            if let Some(ary) = afh {
                let mut aas = crate::sandbox::BD_.lock();
                if let Some(is) = aas.ds(ary) {
                    is.policy.kpc(vh);
                }
            }
            crate::h!(B_, "Domain '{}' blocked in container #{}", vh, ad);
        }

        "watchdog" => {
            crate::sandbox::container::jwm();
            crate::h!(B_, "Watchdog tick executed.");
        }

        "history" => {
            if n.len() < 2 {
                crate::println!("Usage: container history <id>");
                return;
            }
            let ad: u32 = n[1].parse().unwrap_or(0);
            let bjs = crate::sandbox::container::CJ_.lock();
            if let Some(container) = bjs.get(ad) {
                if container.adv.is_empty() {
                    crate::println!("No navigation history.");
                } else {
                    crate::h!(C_, "Navigation history for container #{}:", ad);
                    for (a, url) in container.adv.iter().cf() {
                        crate::println!("  {}. {}", a + 1, url);
                    }
                }
            } else {
                crate::h!(A_, "Container #{} not found", ad);
            }
        }

        _ => {
            crate::h!(C_, "TrustOS Web Container -- persistent isolated web service daemon");
            crate::println!();
            crate::println!("Usage: container <command> [args...]");
            crate::println!();
            crate::h!(Q_, "  Daemon:");
            crate::println!("    status [id]           Show daemon or container status");
            crate::println!("    list                  List all containers");
            crate::println!("    watchdog              Run watchdog health check");
            crate::println!();
            crate::h!(Q_, "  Lifecycle:");
            crate::println!("    create [secure|dev]   Create a new container");
            crate::println!("    start <id>            Start a container");
            crate::println!("    stop <id>             Stop a container");
            crate::println!("    restart <id>          Restart a container");
            crate::println!("    destroy <id>          Remove a container");
            crate::println!();
            crate::h!(Q_, "  Navigation:");
            crate::println!("    go <url> [id]         Navigate through container");
            crate::println!("    history <id>          Show navigation history");
            crate::println!();
            crate::h!(Q_, "  Security:");
            crate::println!("    allow <id> <domain>   Allow domain access");
            crate::println!("    deny <id> <domain>    Block domain access");
            crate::println!();
            crate::h!(D_, "  The default container is auto-started at boot.");
            crate::h!(D_, "  All web traffic is proxied through the kernel.");
            crate::h!(D_, "  Isolation: software (capabilities) | hardware (EPT - future)");
        }
    }
}





pub(super) fn rhv(n: &[&str]) {
    let air = n.fv().hu().unwrap_or("help");
    match air {
        "open" | "navigate" | "go" => {
            
            if n.len() < 2 {
                crate::println!("Usage: sandbox open <url> [strict|moderate|permissive]");
                return;
            }
            let url = n[1];
            let akl = match n.get(2).hu() {
                Some("strict") => crate::sandbox::policy::PolicyPreset::Aet,
                Some("permissive") => crate::sandbox::policy::PolicyPreset::Ads,
                _ => crate::sandbox::policy::PolicyPreset::Ade,
            };

            crate::h!(C_, "[Sandbox] Creating sandbox ({:?})...", akl);
            let ad = crate::sandbox::avp(akl, Some(url));
            crate::println!("  Sandbox #{} created", ad.0);

            crate::h!(C_, "[Sandbox] Navigating to {}...", url);
            match crate::sandbox::bvn(ad, url) {
                Ok(lj) => {
                    crate::h!(B_, "  Status: {} | Content-Type: {} | {} bytes",
                        lj.wt, lj.ahg, lj.gj.len());

                    if lj.oga() {
                        
                        let iyx = lj.hax();
                        let doc = crate::browser::html_parser::due(&iyx);
                        if !doc.dq.is_empty() {
                            crate::println!("  Title: {}", doc.dq);
                        }
                        crate::println!();
                        lzb(&doc, 0);

                        
                        let aas = crate::sandbox::BD_.lock();
                        let policy = aas.get(ad).map(|e| e.policy.ohg()).unwrap_or(false);
                        drop(aas);
                        if policy {
                            let mut lgz = crate::sandbox::js_sandbox::JsSandbox::new(
                                ad, crate::sandbox::js_sandbox::JsSandboxConfig::default()
                            );
                            let hd = lgz.sop(&iyx);
                            if !hd.is_empty() {
                                crate::h!(D_, "\n  [JS] {} script(s) processed", hd.len());
                                for (a, m) in hd.iter().cf() {
                                    if m.cpn {
                                        crate::h!(B_, "    Script {}: OK ({}ms)", a+1, m.oz);
                                    } else {
                                        crate::h!(A_, "    Script {}: {}", a+1,
                                            m.zt.ahz().unwrap_or("failed"));
                                    }
                                    for line in &m.an {
                                        crate::println!("      > {}", line);
                                    }
                                }
                            }
                        } else {
                            crate::h!(D_, "  [JS blocked by policy]");
                        }
                    } else {
                        
                        let text = lj.hax();
                        let hvz: String = text.bw().take(2000).collect();
                        crate::println!("{}", hvz);
                        if text.len() > 2000 {
                            crate::println!("  ... ({} bytes total)", text.len());
                        }
                    }
                }
                Err(aa) => {
                    crate::h!(A_, "  Error: {:?}", aa);
                }
            }
        }

        "list" | "ls" => {
            let bse = crate::sandbox::aoy();
            if bse.is_empty() {
                crate::println!("No active sandboxes.");
            } else {
                crate::h!(C_, "Active sandboxes:");
                crate::println!("  {:>4}  {:>10}  {}", "ID", "State", "Label");
                crate::println!("  {}", "-".afd(40));
                for (ad, cu, g) in &bse {
                    crate::println!("  {:>4}  {:>10?}  {}", ad.0, g, cu);
                }
            }
        }

        "status" | "info" => {
            if n.len() < 2 {
                crate::println!("Usage: sandbox status <id>");
                return;
            }
            let asz: u64 = n[1].parse().unwrap_or(0);
            let ad = crate::sandbox::Ax(asz);
            match crate::sandbox::ibt(ad) {
                Some(e) => crate::println!("{}", e),
                None => crate::h!(A_, "Sandbox #{} not found", asz),
            }
        }

        "kill" | "destroy" | "close" => {
            if n.len() < 2 {
                crate::println!("Usage: sandbox kill <id|all>");
                return;
            }
            if n[1] == "all" {
                let bse = crate::sandbox::aoy();
                for (ad, _, _) in &bse {
                    let _ = crate::sandbox::hfy(*ad);
                }
                crate::h!(B_, "All sandboxes destroyed.");
            } else {
                let asz: u64 = n[1].parse().unwrap_or(0);
                let ad = crate::sandbox::Ax(asz);
                match crate::sandbox::hfy(ad) {
                    Ok(()) => crate::h!(B_, "Sandbox #{} destroyed.", asz),
                    Err(aa) => crate::h!(A_, "Error: {:?}", aa),
                }
            }
        }

        "allow" => {
            
            if n.len() < 3 {
                crate::println!("Usage: sandbox allow <id> <domain>");
                crate::println!("  Example: sandbox allow 1 example.com");
                crate::println!("  Example: sandbox allow 1 *.github.com");
                return;
            }
            let asz: u64 = n[1].parse().unwrap_or(0);
            let vh = n[2];
            let mut aas = crate::sandbox::BD_.lock();
            if let Some(is) = aas.ds(crate::sandbox::Ax(asz)) {
                is.policy.kaf(vh);
                crate::h!(B_, "Domain '{}' allowed in sandbox #{}", vh, asz);
            } else {
                crate::h!(A_, "Sandbox #{} not found", asz);
            }
        }

        "deny" | "block" => {
            
            if n.len() < 3 {
                crate::println!("Usage: sandbox deny <id> <domain>");
                return;
            }
            let asz: u64 = n[1].parse().unwrap_or(0);
            let vh = n[2];
            let mut aas = crate::sandbox::BD_.lock();
            if let Some(is) = aas.ds(crate::sandbox::Ax(asz)) {
                is.policy.kpc(vh);
                crate::h!(B_, "Domain '{}' blocked in sandbox #{}", vh, asz);
            } else {
                crate::h!(A_, "Sandbox #{} not found", asz);
            }
        }

        "policy" => {
            if n.len() < 2 {
                crate::println!("Usage: sandbox policy <id>");
                return;
            }
            let asz: u64 = n[1].parse().unwrap_or(0);
            let aas = crate::sandbox::BD_.lock();
            if let Some(is) = aas.get(crate::sandbox::Ax(asz)) {
                crate::println!("{}", is.policy.awz());
            } else {
                crate::h!(A_, "Sandbox #{} not found", asz);
            }
        }

        "fs" => {
            
            if n.len() < 3 {
                crate::println!("Usage: sandbox fs <id> <ls|tree|read|write|del> [path] [data]");
                return;
            }
            let asz: u64 = n[1].parse().unwrap_or(0);
            let nwn = n[2];
            let mut aas = crate::sandbox::BD_.lock();
            if let Some(is) = aas.ds(crate::sandbox::Ax(asz)) {
                match nwn {
                    "tree" => {
                        crate::println!("{}", is.fio.iex());
                    }
                    "ls" => {
                        let te = n.get(3).hu().unwrap_or("/");
                        match is.fio.aoy(te) {
                            Ok(ch) => {
                                crate::h!(C_, "{}:", te);
                                for (path, are, aw) in &ch {
                                    let pa = if *are == &crate::sandbox::fs::SandboxFileType::K { "[D]" } else { "[F]" };
                                    crate::println!("  {} {} ({} bytes)", pa, path, aw);
                                }
                            }
                            Err(aa) => crate::h!(A_, "Error: {:?}", aa),
                        }
                    }
                    "read" => {
                        let path = n.get(3).hu().unwrap_or("/");
                        match is.fio.read(path) {
                            Ok(f) => {
                                let text = core::str::jg(f).unwrap_or("<binary>");
                                crate::println!("{}", text);
                            }
                            Err(aa) => crate::h!(A_, "Error: {:?}", aa),
                        }
                    }
                    "write" => {
                        if n.len() < 5 {
                            crate::println!("Usage: sandbox fs <id> write <path> <data>");
                            return;
                        }
                        let path = n[3];
                        let f = n[4..].rr(" ");
                        match is.fio.write(path, f.as_bytes(), "shell") {
                            Ok(()) => crate::h!(B_, "Written {} bytes to {}", f.len(), path),
                            Err(aa) => crate::h!(A_, "Error: {:?}", aa),
                        }
                    }
                    "del" | "rm" => {
                        let path = n.get(3).hu().unwrap_or("");
                        match is.fio.rvg(path) {
                            Ok(()) => crate::h!(B_, "Deleted {}", path),
                            Err(aa) => crate::h!(A_, "Error: {:?}", aa),
                        }
                    }
                    _ => crate::println!("Unknown fs command: {}", nwn),
                }
            } else {
                crate::h!(A_, "Sandbox #{} not found", asz);
            }
        }

        "js" | "eval" => {
            
            if n.len() < 3 {
                crate::println!("Usage: sandbox js <id> <code>");
                crate::println!("  Example: sandbox js 1 console.log('hello')");
                return;
            }
            let asz: u64 = n[1].parse().unwrap_or(0);
            let aj = n[2..].rr(" ");
            let aas = crate::sandbox::BD_.lock();
            let aja = aas.get(crate::sandbox::Ax(asz)).is_some();
            let uar = aas.get(crate::sandbox::Ax(asz))
                .map(|e| e.policy.ohg()).unwrap_or(false);
            drop(aas);

            if !aja {
                crate::h!(A_, "Sandbox #{} not found", asz);
                return;
            }
            if !uar {
                crate::h!(A_, "JavaScript is blocked by sandbox policy (use 'moderate' or 'permissive' preset)");
                return;
            }

            let mut lgz = crate::sandbox::js_sandbox::JsSandbox::new(
                crate::sandbox::Ax(asz),
                crate::sandbox::js_sandbox::JsSandboxConfig::default(),
            );
            let result = lgz.bna(&aj);
            if result.cpn {
                crate::h!(B_, "= {}", result.jmj);
            } else {
                crate::h!(A_, "Error: {}", result.zt.ahz().unwrap_or("unknown"));
            }
            for line in &result.an {
                crate::println!("  > {}", line);
            }
            crate::h!(Q_, "  ({}ms)", result.oz);
        }

        "audit" | "log" => {
            if n.len() < 2 {
                
                let aas = crate::sandbox::BD_.lock();
                let log = aas.emi();
                if log.is_empty() {
                    crate::println!("No audit entries.");
                } else {
                    crate::h!(C_, "Audit log ({} entries):", log.len());
                    for bt in log.iter().vv().take(20) {
                        crate::println!("  [{}ms] #{} {:?}: {}",
                            bt.aet, bt.afh.0,
                            bt.hr, bt.eu);
                    }
                }
            } else {
                let asz: u64 = n[1].parse().unwrap_or(0);
                let aas = crate::sandbox::BD_.lock();
                let ch = aas.qlf(crate::sandbox::Ax(asz));
                if ch.is_empty() {
                    crate::println!("No audit entries for sandbox #{}", asz);
                } else {
                    crate::h!(C_, "Audit for sandbox #{} ({} entries):", asz, ch.len());
                    for bt in ch.iter().vv().take(20) {
                        crate::println!("  [{}ms] {:?}: {}",
                            bt.aet, bt.hr, bt.eu);
                    }
                }
            }
        }

        _ => {
            crate::h!(C_, "TrustOS Web Sandbox -- Secure isolated web execution");
            crate::println!();
            crate::println!("Usage: sandbox <command> [args...]");
            crate::println!();
            crate::h!(Q_, "  Navigation:");
            crate::println!("    open <url> [preset]     Open URL in new sandbox");
            crate::println!("                             Presets: strict, moderate (default), permissive");
            crate::println!();
            crate::h!(Q_, "  Sandbox Management:");
            crate::println!("    list                    List active sandboxes");
            crate::println!("    status <id>             Show sandbox details & stats");
            crate::println!("    kill <id|all>           Destroy sandbox(es)");
            crate::println!("    audit [id]              View audit log");
            crate::println!();
            crate::h!(Q_, "  Security Policy:");
            crate::println!("    allow <id> <domain>     Add domain to allowlist");
            crate::println!("    deny <id> <domain>      Add domain to denylist");
            crate::println!("    policy <id>             Show policy config");
            crate::println!();
            crate::h!(Q_, "  Sandboxed Filesystem:");
            crate::println!("    fs <id> tree             Show filesystem tree");
            crate::println!("    fs <id> ls [dir]         List directory");
            crate::println!("    fs <id> read <path>      Read file");
            crate::println!("    fs <id> write <path> <d> Write data to file");
            crate::println!("    fs <id> del <path>       Delete file");
            crate::println!();
            crate::h!(Q_, "  JavaScript (sandboxed):");
            crate::println!("    js <id> <code>           Execute JS in sandbox");
            crate::println!();
            crate::h!(D_, "  Security features:");
            crate::println!("    - Capability-gated network (kernel proxy)");
            crate::println!("    - Domain allow/deny lists + SSRF protection");
            crate::println!("    - Rate limiting + response size limits");
            crate::println!("    - JS static analysis (blocks eval, prototype pollution)");
            crate::println!("    - Jailed filesystem with quotas");
            crate::println!("    - Full audit trail");
        }
    }
}


fn lzb(doc: &crate::browser::Su, xyr: usize) {
    for anq in &doc.xq {
        pbz(anq, 0);
    }
}


fn pbz(anq: &crate::browser::HtmlNode, eo: usize) {
    match anq {
        crate::browser::HtmlNode::Text(text) => {
            let text = text.em();
            if !text.is_empty() {
                crate::println!("{}", text);
            }
        }
        crate::browser::HtmlNode::Na(ij) => {
            let ll = ij.ll.as_str();
            
            
            if oh!(ll, "head" | "script" | "style" | "meta" | "link" | "title" | "noscript") {
                return;
            }
            
            
            match ll {
                "h1" => {
                    crate::println!("");
                    crate::h!(C_, "=== {} ===", hlg(ij));
                    return;
                }
                "h2" => {
                    crate::println!("");
                    crate::h!(C_, "== {} ==", hlg(ij));
                    return;
                }
                "h3" | "h4" | "h5" | "h6" => {
                    crate::println!("");
                    crate::h!(C_, "= {} =", hlg(ij));
                    return;
                }
                "p" => {
                    crate::println!("");
                }
                "br" => {
                    crate::println!("");
                }
                "hr" => {
                    crate::println!("----------------------------------------");
                }
                "a" => {
                    if let Some(cae) = ij.qn("href") {
                        let text = hlg(ij);
                        if !text.is_empty() {
                            crate::h!(CD_, "[{}] ({})", text, cae);
                        }
                    }
                    return;
                }
                "li" => {
                    let crn = "  ".afd(eo);
                    crate::print!("{}* ", crn);
                }
                "pre" | "code" => {
                    crate::h!(DF_, "{}", hlg(ij));
                    return;
                }
                "img" => {
                    if let Some(bdj) = ij.qn("alt") {
                        crate::println!("[Image: {}]", bdj);
                    } else {
                        crate::println!("[Image]");
                    }
                    return;
                }
                _ => {}
            }
            
            
            for aeh in &ij.zf {
                pbz(aeh, eo + 1);
            }
            
            
            if oh!(ll, "p" | "div" | "section" | "article" | "ul" | "ol" | "table" | "tr") {
                crate::println!("");
            }
        }
    }
}


fn hlg(ij: &crate::browser::HtmlElement) -> String {
    use alloc::string::Gd;
    let mut result = String::new();
    nev(&ij.zf, &mut result);
    result.em().to_string()
}


fn nev(xq: &[crate::browser::HtmlNode], result: &mut String) {
    use alloc::string::Gd;
    
    for anq in xq {
        match anq {
            crate::browser::HtmlNode::Text(ab) => {
                result.t(ab);
                result.push(' ');
            }
            crate::browser::HtmlNode::Na(ij) => {
                nev(&ij.zf, result);
            }
        }
    }
}


pub(super) fn fse(path: &str) -> Option<String> {
    
    if let Ok(f) = crate::ramfs::fh(|fs| {
        fs.mq(path).map(|slice| {
            String::from(core::str::jg(slice).unwrap_or(""))
        })
    }) {
        return Some(f);
    }

    
    match crate::vfs::aji(path, crate::vfs::OpenFlags(0)) {
        Ok(da) => {
            let mut k = [0u8; 4096];
            let bo = crate::vfs::read(da, &mut k).unwrap_or(0);
            crate::vfs::agj(da).bq();
            Some(String::from(core::str::jg(&k[..bo]).unwrap_or("")))
        }
        Err(_) => None,
    }
}


pub(super) fn jli(path: &str) -> Option<Vec<u8>> {
    
    if let Ok(f) = crate::ramfs::fh(|fs| {
        fs.mq(path).map(|slice| slice.ip())
    }) {
        return Some(f);
    }
    
    
    match crate::vfs::aji(path, crate::vfs::OpenFlags(0)) {
        Ok(da) => {
            let mut k = Vec::new();
            let mut jj = [0u8; 4096];
            loop {
                match crate::vfs::read(da, &mut jj) {
                    Ok(0) => break,
                    Ok(bo) => k.bk(&jj[..bo]),
                    Err(_) => break,
                }
            }
            crate::vfs::agj(da).bq();
            Some(k)
        }
        Err(_) => None,
    }
}