





use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use crate::framebuffer::{B_, G_, AU_, D_, A_, C_, Q_, CD_, DF_, L_};






pub(super) fn res(n: &[&str]) {
    use crate::wayland::terminal;
    
    let air = n.get(0).hu().unwrap_or("launch");
    
    match air {
        "launch" | "start" | "run" => {
            crate::h!(C_, "+--------------------------------------------------------------+");
            crate::h!(C_, "|           TrustOS Graphical Terminal - Matrix Edition        |");
            crate::h!(C_, "+--------------------------------------------------------------+");
            crate::println!();
            
            
            let _ = crate::wayland::init();
            
            
            let (wf, aav) = crate::framebuffer::yn();
            
            
            let idf = (wf * 80 / 100) & !7; 
            let ide = (aav * 80 / 100) & !15; 
            
            crate::println!("Initializing terminal {}x{} pixels...", idf, ide);
            
            
            match terminal::init(idf, ide) {
                Ok(()) => {
                    crate::h!(B_, "[OK] Graphics terminal initialized");
                }
                Err(aa) => {
                    crate::h!(D_, "[WARN] {}", aa);
                }
            }
            
            
            crate::wayland::dne(|compositor| {
                let cmz = compositor.fgc();
                
                
                if let Some(bi) = terminal::tj() {
                    let (d, i) = terminal::gii().unwrap_or((idf, ide));
                    
                    
                    if let Some(surface) = compositor.axa.ds(&cmz) {
                        surface.dyl(bi, d, i);
                        surface.hzz("TrustOS Terminal");
                        let b = (wf - d) / 2;
                        let c = (aav - i) / 2;
                        surface.eyk(b as i32, c as i32);
                        surface.hqr();
                        surface.dfc();
                    }
                }
                
                crate::h!(B_, "[OK] Terminal surface created (ID: {})", cmz);
            });
            
            
            crate::wayland::ffn();
            
            crate::println!();
            crate::h!(B_, "Terminal launched!");
            crate::println!("Use 'gterm demo' for an interactive demo.");
            crate::println!("Use 'gterm fullscreen' for fullscreen mode.");
        },
        
        "demo" => {
            crate::h!(C_, "Starting interactive graphical terminal demo...");
            crate::println!();
            
            
            let _ = crate::wayland::init();
            
            let (wf, aav) = crate::framebuffer::yn();
            let idf = (wf * 85 / 100) & !7;
            let ide = (aav * 85 / 100) & !15;
            
            
            let _ = terminal::init(idf, ide);
            
            
            let cmz = crate::wayland::dne(|compositor| {
                let ad = compositor.fgc();
                
                if let Some(bi) = terminal::tj() {
                    if let Some((d, i)) = terminal::gii() {
                        if let Some(surface) = compositor.axa.ds(&ad) {
                            surface.dyl(bi, d, i);
                            surface.hzz("TrustOS Terminal Demo");
                            surface.eyk(
                                ((wf - d) / 2) as i32,
                                ((aav - i) / 2) as i32
                            );
                            surface.hqr();
                            surface.dfc();
                        }
                    }
                }
                ad
            }).unwrap_or(0);
            
            
            crate::wayland::ffn();
            
            
            terminal::write("\x1b[2J\x1b[H"); 
            terminal::write("\x1b[1;32m+----------------------------------------------------------+\r\n");
            terminal::write("|  \x1b[1;97mTrustOS\x1b[1;32m Graphical Terminal Demo                       |\r\n");
            terminal::write("|  Matrix Edition v1.0                                     |\r\n");
            terminal::write("+----------------------------------------------------------+\r\n");
            terminal::write("\x1b[0;32m\r\n");
            terminal::write("Type text and press Enter. Press ESC to exit.\r\n\r\n");
            terminal::write("\x1b[1;32m$ \x1b[0;32m");
            
            
            crate::wayland::dne(|compositor| {
                if let Some(bi) = terminal::tj() {
                    if let Some((d, i)) = terminal::gii() {
                        if let Some(surface) = compositor.axa.ds(&cmz) {
                            surface.dyl(bi, d, i);
                            surface.dfc();
                        }
                    }
                }
            });
            crate::wayland::ffn();
            
            
            let mut xn = alloc::string::String::new();
            loop {
                
                if crate::shell::etf() { break; }
                
                if let Some(bs) = crate::keyboard::auw() {
                    let r = bs as char;
                    match bs {
                        0x1b => {
                            
                            break;
                        }
                        0x0D | 0x0A => {
                            
                            terminal::write("\r\n");
                            
                            if !xn.is_empty() {
                                
                                let mk = alloc::format!("\x1b[0;36mYou typed: \x1b[1;97m{}\x1b[0;32m\r\n", xn);
                                terminal::write(&mk);
                                xn.clear();
                            }
                            
                            terminal::write("\x1b[1;32m$ \x1b[0;32m");
                        }
                        0x08 | 0x7F => {
                            
                            if !xn.is_empty() {
                                xn.pop();
                                terminal::write("\x08 \x08");
                            }
                        }
                        eh if eh >= 0x20 && eh < 0x7F => {
                            
                            xn.push(r);
                            let e = alloc::format!("{}", r);
                            terminal::write(&e);
                        }
                        _ => {}
                    }
                    
                    
                    crate::wayland::dne(|compositor| {
                        if let Some(bi) = terminal::tj() {
                            if let Some((d, i)) = terminal::gii() {
                                if let Some(surface) = compositor.axa.ds(&cmz) {
                                    surface.dyl(bi, d, i);
                                    surface.dfc();
                                }
                            }
                        }
                    });
                    crate::wayland::ffn();
                }
                
                
                for _ in 0..1000 { core::hint::hc(); }
            }
            
            
            crate::framebuffer::clear();
            crate::h!(B_, "Demo ended.");
        },
        
        "fullscreen" | "fs" => {
            crate::h!(C_, "Launching fullscreen terminal...");
            
            
            let (wf, aav) = crate::framebuffer::yn();
            
            let _ = crate::wayland::init();
            let _ = terminal::init(wf, aav);
            
            
            crate::wayland::dne(|compositor| {
                let ad = compositor.fgc();
                
                if let Some(bi) = terminal::tj() {
                    if let Some((d, i)) = terminal::gii() {
                        if let Some(surface) = compositor.axa.ds(&ad) {
                            surface.dyl(bi, d, i);
                            surface.hzz("TrustOS Terminal");
                            surface.eyk(0, 0);
                            surface.hqr();
                            surface.g.szf = true;
                            surface.dfc();
                        }
                    }
                }
            });
            
            crate::wayland::ffn();
            crate::h!(B_, "[OK] Fullscreen terminal active");
        },
        
        "test" => {
            
            crate::h!(C_, "Testing graphical terminal ANSI support...");
            
            let _ = crate::wayland::init();
            let (d, i) = crate::framebuffer::yn();
            let _ = terminal::init(d * 70 / 100, i * 70 / 100);
            
            
            terminal::write("\x1b[2J\x1b[H"); 
            terminal::write("\x1b[1;32m=== ANSI Escape Code Test ===\x1b[0m\r\n\r\n");
            
            
            terminal::write("\x1b[31mRed \x1b[32mGreen \x1b[33mYellow \x1b[34mBlue \x1b[35mMagenta \x1b[36mCyan\x1b[0m\r\n");
            terminal::write("\x1b[91mBright Red \x1b[92mBright Green \x1b[93mBright Yellow\x1b[0m\r\n\r\n");
            
            
            terminal::write("\x1b[1mBold\x1b[0m \x1b[2mDim\x1b[0m \x1b[4mUnderline\x1b[0m \x1b[7mReverse\x1b[0m\r\n\r\n");
            
            
            terminal::write("\x1b[32m");
            for a in 0..5 {
                for _ in 0..60 {
                    let r = ((a * 7 + 33) % 94 + 33) as u8 as char;
                    let e = alloc::format!("{}", r);
                    terminal::write(&e);
                }
                terminal::write("\r\n");
            }
            terminal::write("\x1b[0m\r\n");
            
            terminal::write("\x1b[1;97mTest complete!\x1b[0m\r\n");
            
            
            crate::wayland::dne(|compositor| {
                let ad = compositor.fgc();
                if let Some(bi) = terminal::tj() {
                    if let Some((qd, ejt)) = terminal::gii() {
                        if let Some(surface) = compositor.axa.ds(&ad) {
                            surface.dyl(bi, qd, ejt);
                            surface.hzz("ANSI Test");
                            surface.eyk(
                                ((d - qd) / 2) as i32,
                                ((i - ejt) / 2) as i32
                            );
                            surface.hqr();
                            surface.dfc();
                        }
                    }
                }
            });
            crate::wayland::ffn();
            
            crate::println!();
            crate::h!(B_, "Press any key to close...");
            loop {
                if crate::keyboard::auw().is_some() {
                    break;
                }
            }
            crate::framebuffer::clear();
        },
        
        _ => {
            crate::h!(C_, "+--------------------------------------------------------------+");
            crate::h!(C_, "|       TrustOS Graphical Terminal - Matrix Edition           |");
            crate::h!(C_, "+--------------------------------------------------------------+");
            crate::println!();
            crate::println!("A native graphical terminal emulator with VT100/ANSI support.");
            crate::println!("Inspired by Smithay, built from scratch for TrustOS.");
            crate::println!();
            crate::println!("Usage: gterm <command>");
            crate::println!();
            crate::println!("Commands:");
            crate::println!("  launch     - Open the graphical terminal window");
            crate::println!("  demo       - Interactive demo (type text, ESC to exit)");
            crate::println!("  fullscreen - Open fullscreen terminal");
            crate::println!("  test       - Test ANSI escape code rendering");
            crate::println!();
            crate::println!("Features:");
            crate::println!("  * Matrix-style green phosphor theme");
            crate::println!("  * VT100/ANSI escape code support");
            crate::println!("  * 256-color and 24-bit RGB colors");
            crate::println!("  * Scrollback buffer (1000 lines)");
            crate::println!("  * Phosphor glow effect");
            crate::println!("  * CRT scanline effect");
        }
    }
}


pub(super) fn rkp(n: &[&str]) {
    let air = n.get(0).hu().unwrap_or("help");
    
    match air {
        "init" | "start" => {
            crate::h!(C_, "+--------------------------------------------------------------+");
            crate::h!(C_, "|            TrustOS Wayland Compositor                        |");
            crate::h!(C_, "+--------------------------------------------------------------+");
            crate::println!();
            
            match crate::wayland::init() {
                Ok(()) => {
                    crate::h!(B_, "[OK] Wayland compositor initialized");
                    
                    
                    let (z, ac) = crate::framebuffer::yn();
                    crate::println!("     Display: {}x{}", z, ac);
                    crate::println!();
                    crate::println!("Available globals:");
                    for apu in crate::wayland::protocol::kys() {
                        crate::println!("  * {} v{}", apu.akf, apu.dk);
                    }
                }
                Err(aa) => {
                    crate::h!(A_, "[ERROR] {}", aa);
                }
            }
        },
        
        "demo" => {
            crate::h!(C_, "Starting Wayland demo...");
            
            
            let _ = crate::wayland::init();
            
            
            crate::wayland::dne(|compositor| {
                
                let cmz = compositor.fgc();
                
                
                let z = 400u32;
                let ac = 300u32;
                let mut bi = alloc::vec![0xFF0A0F0C_u32; (z * ac) as usize];
                
                
                for c in 0..ac {
                    for b in 0..z {
                        let m = (b * 255 / z) as u8;
                        let at = ((c * 255 / ac) as u8) / 2;
                        let o = 0x20_u8;
                        bi[(c * z + b) as usize] = 0xFF000000 | (m as u32) << 16 | (at as u32) << 8 | o as u32;
                    }
                }
                
                
                for b in 0..z {
                    bi[b as usize] = 0xFF00FF66;
                    bi[((ac - 1) * z + b) as usize] = 0xFF00FF66;
                }
                for c in 0..ac {
                    bi[(c * z) as usize] = 0xFF00FF66;
                    bi[(c * z + z - 1) as usize] = 0xFF00FF66;
                }
                
                
                if let Some(surface) = compositor.axa.ds(&cmz) {
                    surface.dyl(bi, z, ac);
                    surface.hzz("Wayland Demo");
                    surface.eyk(200, 150);
                    surface.hqr();
                    surface.dfc();
                }
                
                crate::h!(B_, "[OK] Created surface {}", cmz);
            });
            
            
            crate::wayland::ffn();
            crate::h!(B_, "[OK] Frame composed to framebuffer");
            crate::println!();
            crate::println!("Press any key to close demo...");
            
            
            loop {
                if let Some(_) = crate::keyboard::auw() {
                    break;
                }
            }
            
            
            crate::framebuffer::clear();
        },
        
        "status" => {
            crate::h!(C_, "Wayland Compositor Status");
            crate::h!(C_, "--------------------------");
            
            crate::wayland::dne(|compositor| {
                let (d, i) = (compositor.z, compositor.ac);
                crate::println!("Display: {}x{}", d, i);
                crate::println!("Surfaces: {}", compositor.axa.len());
                crate::println!("SHM Pools: {}", compositor.mfn.len());
                crate::println!("Frame: {}", compositor.kxa);
                crate::println!("Pointer: ({}, {})", compositor.hvo, compositor.hvp);
                
                if !compositor.axa.is_empty() {
                    crate::println!();
                    crate::println!("Surfaces:");
                    for (&ad, surface) in &compositor.axa {
                        let dq = if surface.dq.is_empty() { "<untitled>" } else { &surface.dq };
                        crate::println!("  #{}: {} @ ({},{}) {}x{}", 
                            ad, dq, surface.b, surface.c, surface.z, surface.ac);
                    }
                }
            }).unwrap_or_else(|| {
                crate::h!(D_, "Compositor not initialized");
                crate::println!("Run 'wayland init' first");
            });
        },
        
        _ => {
            crate::h!(C_, "TrustOS Wayland Compositor");
            crate::h!(C_, "--------------------------");
            crate::println!();
            crate::println!("A native Wayland display server for TrustOS.");
            crate::println!();
            crate::println!("Usage: wayland <command>");
            crate::println!();
            crate::println!("Commands:");
            crate::println!("  init    - Initialize the Wayland compositor");
            crate::println!("  demo    - Run a visual demo");
            crate::println!("  status  - Show compositor status");
            crate::println!();
            crate::println!("Protocol support:");
            crate::println!("  * wl_compositor v5 - Surface creation");
            crate::println!("  * wl_shm v1        - Shared memory buffers");
            crate::println!("  * wl_seat v8       - Input devices");
            crate::println!("  * xdg_wm_base v5   - Window management");
        }
    }
}


pub(super) fn rjp(n: &[&str]) {
    let air = n.fv().hu().unwrap_or("help");

    match air {
        "run" => {
            let it = match n.get(1) {
                Some(bb) => *bb,
                None => { crate::println!("Usage: trustlang run <file.tl>"); return; }
            };
            let path = if it.cj('/') {
                alloc::string::String::from(it)
            } else {
                alloc::format!("/{}", it)
            };
            let iy = match crate::ramfs::fh(|fs| fs.mq(&path).map(|bc| bc.ip())) {
                Ok(f) => match alloc::string::String::jg(f) {
                    Ok(e) => e,
                    Err(_) => { crate::println!("Error: file is not valid UTF-8"); return; }
                },
                Err(_) => { crate::println!("Error: file '{}' not found", it); return; }
            };
            crate::println!("\x1b[36m[TrustLang]\x1b[0m Compiling {}...", it);
            match crate::trustlang::vw(&iy) {
                Ok(an) => {
                    if !an.is_empty() { crate::print!("{}", an); }
                    crate::println!("\x1b[32m[TrustLang]\x1b[0m Program finished successfully.");
                }
                Err(aa) => crate::println!("\x1b[31m[TrustLang Error]\x1b[0m {}", aa),
            }
        }
        "check" => {
            let it = match n.get(1) {
                Some(bb) => *bb,
                None => { crate::println!("Usage: trustlang check <file.tl>"); return; }
            };
            let path = if it.cj('/') {
                alloc::string::String::from(it)
            } else {
                alloc::format!("/{}", it)
            };
            let iy = match crate::ramfs::fh(|fs| fs.mq(&path).map(|bc| bc.ip())) {
                Ok(f) => match alloc::string::String::jg(f) {
                    Ok(e) => e,
                    Err(_) => { crate::println!("Error: file is not valid UTF-8"); return; }
                },
                Err(_) => { crate::println!("Error: file '{}' not found", it); return; }
            };
            match crate::trustlang::feq(&iy) {
                Ok(()) => crate::println!("\x1b[32m?\x1b[0m {} -- no errors", it),
                Err(aa) => crate::println!("\x1b[31m?\x1b[0m {} -- {}", it, aa),
            }
        }
        "eval" => {
            
            let aj = n[1..].rr(" ");
            let fyx = alloc::format!("fn main() {{ {} }}", aj);
            match crate::trustlang::vw(&fyx) {
                Ok(an) => { if !an.is_empty() { crate::print!("{}", an); } }
                Err(aa) => crate::println!("\x1b[31mError:\x1b[0m {}", aa),
            }
        }
        "demo" => {
            
            let demo = r#"// TrustLang Demo -- Fibonacci
fn fibonacci(n: i64) -> i64 {
    if n <= 1 {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

fn main() {
    println("=== TrustLang on TrustOS ===");
    println("Computing Fibonacci sequence:");
    for i in 0..15 {
        let result = fibonacci(i);
        print("  fib(");
        print(to_string(i));
        print(") = ");
        println(to_string(result));
    }
    println("Done!");
}
"#;
            crate::ramfs::fh(|fs| {
                let _ = fs.ns("/demo.tl", demo.as_bytes());
            });
            crate::println!("Created /demo.tl -- run with: trustlang run demo.tl");
            
            match crate::trustlang::vw(demo) {
                Ok(an) => { if !an.is_empty() { crate::print!("{}", an); } }
                Err(aa) => crate::println!("\x1b[31mError:\x1b[0m {}", aa),
            }
        }
        "repl" => {
            crate::println!("\x1b[1;36mTrustLang REPL\x1b[0m v1.0 — type 'exit' or 'quit' to leave");
            crate::println!("  Expressions are auto-wrapped in fn main() {{ ... }}");
            crate::println!("  Available: print/println, math ops, if/else, for, while");
            crate::println!();
            loop {
                if crate::shell::etf() { break; }
                crate::print!("\x1b[36mtl>\x1b[0m ");
                let line = crate::shell::cts();
                let ux = line.em();
                if ux.is_empty() {
                    if crate::shell::etf() { break; }
                    continue;
                }
                if ux == "exit" || ux == "quit" { break; }
                if ux == "help" {
                    crate::println!("  println(\"hello\")       — print with newline");
                    crate::println!("  let x = 42;            — declare variable");
                    crate::println!("  for i in 0..5 {{ ... }}  — for loop");
                    crate::println!("  fn foo(n: i64) {{ ... }} — define function + fn main()");
                    continue;
                }
                match crate::trustlang::nrc(ux) {
                    Ok(an) => {
                        if !an.is_empty() { crate::print!("{}", an); }
                    }
                    Err(aa) => crate::println!("\x1b[31mError:\x1b[0m {}", aa),
                }
            }
        }
        "compile" | "native" => {
            
            let it = match n.get(1) {
                Some(bb) => *bb,
                None => { crate::println!("Usage: trustlang compile <file.tl>"); return; }
            };
            let path = if it.cj('/') {
                alloc::string::String::from(it)
            } else {
                alloc::format!("/{}", it)
            };
            let iy = match crate::ramfs::fh(|fs| fs.mq(&path).map(|bc| bc.ip())) {
                Ok(f) => match alloc::string::String::jg(f) {
                    Ok(e) => e,
                    Err(_) => { crate::println!("Error: file is not valid UTF-8"); return; }
                },
                Err(_) => { crate::println!("Error: file '{}' not found", it); return; }
            };
            crate::println!("\x1b[36m[TrustLang Native]\x1b[0m Compiling {} to x86_64...", it);
            fn urk(ad: u8, byg: usize, cjc: *const i64) -> i64 {
                
                match ad {
                    0 | 1 => { 
                        if byg > 0 {
                            let ap = unsafe { *cjc };
                            crate::print!("{}", ap);
                        }
                        if ad == 1 { crate::println!(); }
                        0
                    }
                    _ => 0,
                }
            }
            match crate::trustlang::rmy(&iy, urk) {
                Ok(result) => {
                    crate::println!("\x1b[32m[TrustLang Native]\x1b[0m Program returned: {}", result);
                }
                Err(aa) => crate::println!("\x1b[31m[TrustLang Native Error]\x1b[0m {}", aa),
            }
        }
        "test" => {
            
            crate::println!("\x1b[1;36m══════ TrustLang Native x86_64 Test Suite ══════\x1b[0m\n");
            let (cg, gv, yw) = crate::trustlang::wbi();
            crate::print!("{}", yw);
            crate::println!();
            if gv == 0 {
                crate::println!("\x1b[1;32m  ALL {} TESTS PASSED\x1b[0m", cg);
            } else {
                crate::println!("\x1b[1;31m  {}/{} tests failed\x1b[0m", gv, cg + gv);
            }
        }
        "bench" => {
            
            crate::println!("\x1b[1;36m── TrustLang Native Benchmark ──\x1b[0m");
            crate::println!("  Computing fib(25) natively...");
            let (result, yl) = crate::trustlang::tests::qov();
            crate::println!("  Result: fib(25) = {}", result);
            crate::println!("  Cycles: {} (~{} µs @ 3GHz)", yl, yl / 3000);

            
            crate::println!("  Computing fib(25) via bytecode VM...");
            let xsf = "fn fib(n: i64) -> i64 { if n <= 1 { return n; } return fib(n - 1) + fib(n - 2); } fn main() { let r = fib(25); println(to_string(r)); }";
            let ay = unsafe { core::arch::x86_64::dxw() };
            let _ = crate::trustlang::vw(xsf);
            let ci = unsafe { core::arch::x86_64::dxw() };
            let jvw = ci - ay;
            crate::println!("  VM Cycles: {} (~{} µs @ 3GHz)", jvw, jvw / 3000);
            if jvw > 0 {
                crate::println!("  \x1b[1;32mNative speedup: {:.1}x\x1b[0m", jvw as f64 / yl as f64);
            }
        }
        _ => {
            crate::println!("\x1b[1;36mTrustLang\x1b[0m -- Integrated Programming Language");
            crate::println!("  Rust-inspired syntax, bytecode VM + native x86_64 compiler\n");
            crate::println!("Commands:");
            crate::println!("  trustlang run <file.tl>        Compile & execute (bytecode VM)");
            crate::println!("  trustlang compile <file.tl>    Compile to native x86_64 & execute");
            crate::println!("  trustlang check <file.tl>      Syntax check only");
            crate::println!("  trustlang eval <code>          Evaluate inline code");
            crate::println!("  trustlang repl                 Interactive REPL");
            crate::println!("  trustlang demo                 Create & run demo program");
            crate::println!("  trustlang test                 Run native backend test suite");
            crate::println!("  trustlang bench                Benchmark native vs bytecode VM");
            crate::println!("\nExample:");
            crate::println!("  trustlang eval println(\"Hello TrustOS!\")");
            crate::println!("  trustlang eval \"let x = 42; println(to_string(x * 2))\"");
        }
    }
}



fn xmk(line: &str) -> alloc::vec::Vec<u32> {
    let bw: alloc::vec::Vec<char> = line.bw().collect();
    let len = bw.len();
    let mut colors = alloc::vec![0xFFD4D4D4u32; len]; 
    if len == 0 { return colors; }

    
    let nfb = {
        let mut nc = None;
        let bf = line.as_bytes();
        for a in 0..bf.len().ao(1) {
            if bf[a] == b'/' && bf[a + 1] == b'/' {
                nc = Some(line[..a].bw().az());
                break;
            }
        }
        nc
    };
    let hhw = nfb.unwrap_or(len);

    
    if let Some(bza) = nfb {
        for a in bza..len {
            colors[a] = 0xFF6A9955;
        }
    }

    
    let mut cyv = false;
    for a in 0..hhw {
        if bw[a] == '"' {
            colors[a] = 0xFFCE9178;
            cyv = !cyv;
        } else if cyv {
            colors[a] = 0xFFCE9178;
        }
    }

    
    cyv = false;
    let mut a = 0usize;
    while a < hhw {
        if bw[a] == '"' {
            cyv = !cyv;
            a += 1;
            continue;
        }
        if cyv { a += 1; continue; }

        
        if bw[a].atb() {
            colors[a] = 0xFFB5CEA8;
            a += 1;
            continue;
        }
        
        if oh!(bw[a], '(' | ')' | '{' | '}' | '[' | ']') {
            colors[a] = 0xFFFFD700;
            a += 1;
            continue;
        }
        
        if bw[a].jaz() || bw[a] == '_' {
            let ay = a;
            while a < hhw && (bw[a].etb() || bw[a] == '_') {
                a += 1;
            }
            let od: alloc::string::String = bw[ay..a].iter().collect();

            
            let mut amm = a;
            while amm < hhw && bw[amm] == ' ' { amm += 1; }
            let lgd = amm < hhw && bw[amm] == '(';

            
            let cvu: alloc::string::String = bw[..ay].iter().collect();
            let ux = cvu.eke();
            let tzk = ux.pp("let") || ux.pp("mut");

            if oh!(od.as_str(),
                "fn" | "let" | "mut" | "if" | "else" | "while" | "for" | "in" |
                "return" | "loop" | "break" | "continue" | "true" | "false" |
                "struct" | "enum" | "match" | "use" | "pub" | "const" | "static" |
                "impl" | "self" | "type")
            {
                for fb in ay..a { colors[fb] = 0xFFFF7B72; } 
            } else if lgd {
                for fb in ay..a { colors[fb] = 0xFF79C0FF; } 
            } else if tzk {
                for fb in ay..a { colors[fb] = 0xFF9CDCFE; } 
            }
            
            continue;
        }
        a += 1;
    }
    colors
}




pub(super) fn rjq() {
    let (kp, kl) = crate::framebuffer::yn();
    let d = kp as usize;
    let i = kl as usize;

    let afk = crate::framebuffer::bre();
    if !afk {
        crate::framebuffer::beo();
        crate::framebuffer::afi(true);
    }

    let mut k = alloc::vec![0u32; d * i];

    

    let aej = |k: &mut [u32], d: usize, i: usize, cx: usize, ae: usize, r: char, s: u32, bv: usize| {
        let ka = crate::framebuffer::font::ada(r);
        for (br, &fs) in ka.iter().cf() {
            for ga in 0..8u32 {
                if fs & (0x80 >> ga) != 0 {
                    for cq in 0..bv {
                        for cr in 0..bv {
                            let y = cx + ga as usize * bv + cr;
                            let x = ae + br * bv + cq;
                            if y < d && x < i {
                                k[x * d + y] = s;
                            }
                        }
                    }
                }
            }
        }
    };

    let ri = |k: &mut [u32], d: usize, i: usize, b: usize, c: usize, text: &str, s: u32, bv: usize| {
        for (a, r) in text.bw().cf() {
            aej(k, d, i, b + a * 8 * bv, c, r, s, bv);
        }
    };

    let np = |k: &mut [u32], d: usize, i: usize, c: usize, text: &str, s: u32, bv: usize| {
        let qd = text.len() * 8 * bv;
        let cr = if qd < d { (d - qd) / 2 } else { 0 };
        for (a, r) in text.bw().cf() {
            aej(k, d, i, cr + a * 8 * bv, c, r, s, bv);
        }
    };

    let mb = |k: &[u32], d: usize, i: usize| {
        if let Some((bgc, dnu, bgb, baz)) = crate::framebuffer::cey() {
            let aaa = bgc as *mut u32;
            let bgd = baz as usize;
            for c in 0..i.v(bgb as usize) {
                unsafe {
                    core::ptr::copy_nonoverlapping(k[c * d..].fq(), aaa.add(c * bgd), d);
                }
            }
        }
        crate::framebuffer::sv();
    };

    let aol = |k: &mut [u32]| {
        for ai in k.el() { *ai = 0xFF000000; }
    };

    
    let mut ws: alloc::vec::Vec<u16> = (0..d / 8 + 1).map(|a| ((a * 37 + 13) % i) as u16).collect();
    let yg: alloc::vec::Vec<u8> = (0..d / 8 + 1).map(|a| (((a * 7 + 3) % 4) + 1) as u8).collect();

    let eba = |k: &mut [u32], d: usize, i: usize, ec: &mut [u16], arz: &[u8], frame: u32| {
        for il in k.el() {
            let at = ((*il >> 8) & 0xFF) as u32;
            if at > 0 { *il = 0xFF000000 | (at.ao(6) << 8); }
        }
        for nc in 0..ec.len() {
            let b = nc * 8;
            if b >= d { continue; }
            ec[nc] = ec[nc].cn(arz[nc] as u16);
            if ec[nc] as usize >= i { ec[nc] = 0; }
            let c = ec[nc] as usize;
            let r = (((frame as usize + nc * 13) % 94) + 33) as u8 as char;
            let ka = crate::framebuffer::font::ada(r);
            for (br, &fs) in ka.iter().cf() {
                let x = c + br;
                if x >= i { break; }
                for ga in 0..8u32 {
                    if fs & (0x80 >> ga) != 0 {
                        let y = b + ga as usize;
                        if y < d { k[x * d + y] = 0xFF00FF44; }
                    }
                }
            }
        }
    };

    
    let aps: u64 = 30;

    
    let apq = |k: &mut [u32], d: usize, i: usize, bge: &dyn Fn(&[u32], usize, usize)| {
        for _ in 0..78 {
            for y in k.el() {
                let m = ((*y >> 16) & 0xFF).ao(4);
                let at = ((*y >> 8) & 0xFF).ao(4);
                let o = (*y & 0xFF).ao(4);
                *y = 0xFF000000 | (m << 16) | (at << 8) | o;
            }
            bge(k, d, i);
            crate::cpu::tsc::rd(aps);
        }
        for ai in k.el() { *ai = 0xFF000000; }
        bge(k, d, i);
        
        crate::cpu::tsc::rd(1400);
    };

    
    
    
    
    
    let jqb = |k: &mut [u32], d: usize, i: usize,
                            ws: &mut [u16], yg: &[u8],
                            ak: &[(&str, u32, usize)],
                            hsc: u64,
                            lci: u32| {
        let aqo: usize = ak.iter().map(|(ab, _, _)| ab.len()).sum();
        
        let hkm = (hsc / aps).am(1) as u32;
        let gvh = aqo as u32 * hkm;
        let agc = gvh + lci;
        let mut frame = 0u32;

        while frame < agc {
            
            if let Some(eh) = crate::keyboard::xw() {
                if eh == 0x1B { return; }
                if eh == b' ' || eh == b'\r' || eh == b'\n' { break; }
            }

            
            eba(k, d, i, ws, yg, frame);

            
            let qo = (frame / hkm) as usize;

            
            let aku: usize = ak.iter().map(|(_, _, e)| 16 * e + 12).sum();
            let mut c = if aku < i { (i - aku) / 2 } else { 20 };
            let mut bbh = 0usize;

            for &(text, s, bv) in ak {
                let qd = text.len() * 8 * bv;
                let cr = if qd < d { (d - qd) / 2 } else { 0 };
                for (a, r) in text.bw().cf() {
                    if bbh + a >= qo { break; }
                    aej(k, d, i, cr + a * 8 * bv, c, r, s, bv);
                }
                
                if qo > bbh && qo < bbh + text.len() {
                    let nc = qo - bbh;
                    let cx = cr + nc * 8 * bv;
                    if (frame / 8) % 2 == 0 {
                        for ae in c..c + 16 * bv {
                            if ae < i && cx + 2 < d {
                                k[ae * d + cx] = 0xFF00FF88;
                                k[ae * d + cx + 1] = 0xFF00FF88;
                            }
                        }
                    }
                }
                bbh += text.len();
                c += 16 * bv + 12;
            }

            mb(k, d, i);
            frame += 1;
            crate::cpu::tsc::rd(aps);
        }
    };

    
    
    
    
    
    
    let wne = |k: &mut [u32], d: usize, i: usize,
                             ws: &mut [u16], yg: &[u8],
                             dq: &str,
                             iy: &str,
                             oww: &str,
                             ybk: u64,
                             xzt: u32,
                             otj: u64| {
        
        
        
        

        let cao: alloc::vec::Vec<&str> = iy.ak().collect();
        let aqo: usize = iy.len();

        let czm = 40usize;
        let bbs = 50usize;
        let hdm = bbs + 30;
        let gy = 18usize;
        let gcv = 1usize;

        
        
        let mut inf: alloc::vec::Vec<(usize, usize, char)> = alloc::vec::Vec::new();
        for (alj, line) in cao.iter().cf() {
            for (nc, r) in line.bw().cf() {
                inf.push((alj, nc, r));
            }
            inf.push((alj, line.len(), '\n')); 
        }

        
        let mut ajn: u32 = 0xDEAD_BEEF;
        let mut exu = |g: &mut u32| -> u32 {
            *g ^= *g << 13;
            *g ^= *g >> 17;
            *g ^= *g << 5;
            *g
        };

        
        
        let xnu: alloc::vec::Vec<(usize, char)> = alloc::vec![
            (45, 'w'),     
            (180, 'p'),    
            (350, 'e'),    
            (520, '0'),    
            (700, ';'),    
        ];

        
        
        let mut qo: usize = 0;
        let mut cma: u32 = 0;

        
        let ehm = |k: &mut [u32], d: usize, i: usize,
                             ws: &mut [u16], yg: &[u8],
                             cma: u32,
                             qo: usize,
                             mns: Option<(usize, usize, char)>| {
            
            for ai in k.el() { *ai = 0xFF0A0A0A; }
            eba(k, d, i, ws, yg, cma);
            for ai in k.el() {
                let at = ((*ai >> 8) & 0xFF).v(25);
                *ai = 0xFF000000 | (at << 8);
            }

            
            for c in 0..bbs {
                for b in 0..d {
                    k[c * d + b] = 0xFF111111;
                }
            }
            ri(k, d, i, czm + 20, 15, dq, 0xFF00FF88, 2);

            if !oww.is_empty() {
                ri(k, d, i, czm + 20, bbs + 5, oww, 0xFF888888, 1);
            }

            
            let awm = czm;
            let yd = d - 2 * czm;
            let atg = hdm;
            let ans = i - hdm - 80;
            for x in atg..atg + ans {
                for y in awm..awm + yd {
                    if x < i && y < d {
                        k[x * d + y] = 0xFF0D1117;
                    }
                }
            }
            
            for y in awm..awm + yd {
                if atg < i { k[atg * d + y] = 0xFF00FF44; }
                let bjj = (atg + ans).v(i) - 1;
                k[bjj * d + y] = 0xFF00FF44;
            }
            for x in atg..(atg + ans).v(i) {
                k[x * d + awm] = 0xFF00FF44;
                let hw = (awm + yd - 1).v(d - 1);
                k[x * d + hw] = 0xFF00FF44;
            }

            
            let ayf = ans.ao(30) / gy;
            let gn = {
                let mut nc = 0usize;
                let mut glr = 0usize;
                for (alj, line) in cao.iter().cf() {
                    if nc + line.len() >= qo {
                        glr = alj;
                        break;
                    }
                    nc += line.len() + 1;
                    glr = alj + 1;
                }
                glr.v(cao.len().ao(1))
            };
            let px = if gn >= ayf.ao(2) {
                gn.ao(ayf.ao(3))
            } else {
                0
            };

            
            let bds = awm + 42;
            let mut gik = 0usize;
            
            for alj in 0..px.v(cao.len()) {
                gik += cao[alj].len() + 1; 
            }
            for afj in 0..(cao.len() - px.v(cao.len())) {
                let alj = afj + px;
                let ct = hdm + 15 + afj * gy;
                if ct + 16 > atg + ans { break; }
                let line = cao[alj];

                
                let ugh = alloc::format!("{:>3}", alj + 1);
                ri(k, d, i, awm + 8, ct, &ugh, 0xFF555555, gcv);
                let pia = awm + 35;
                for cq in ct..ct + 16 {
                    if cq < i && pia < d { k[cq * d + pia] = 0xFF333333; }
                }

                
                let uex = xmk(line);
                for (nc, r) in line.bw().cf() {
                    if gik >= qo { break; }
                    let s = uex.get(nc).hu().unwrap_or(0xFFD4D4D4);
                    aej(k, d, i, bds + nc * 8 * gcv, ct, r, s, gcv);
                    gik += 1;
                }

                
                if let Some((xic, xbk, asb)) = mns {
                    if alj == xic && gik == qo {
                        let uiy = hdm + 15 + afj * gy;
                        aej(k, d, i, bds + xbk * 8 * gcv, uiy, asb, 0xFFFF4444, gcv);
                    }
                }

                if gik < qo { gik += 1; } 
            }

            
            if cao.len() > ayf {
                let auz = awm + yd - 8;
                let hyt = atg + 2;
                let dbr = ans.ao(4);
                for x in hyt..hyt + dbr {
                    if x < i && auz < d { k[x * d + auz] = 0xFF1A1A1A; }
                }
                let axd = ((ayf * dbr) / cao.len()).am(10);
                let bsm = if cao.len() > 0 { hyt + (px * dbr) / cao.len() } else { hyt };
                for x in bsm..(bsm + axd).v(hyt + dbr) {
                    if x < i && auz < d {
                        k[x * d + auz] = 0xFF00FF44;
                        if auz + 1 < d { k[x * d + auz + 1] = 0xFF00FF44; }
                    }
                }
            }

            
            if qo <= aqo && gn >= px {
                let mut khp = 0usize;
                let mut mjy = 0usize;
                let mut mjx = 0usize;
                for (alj, line) in cao.iter().cf() {
                    if khp + line.len() >= qo {
                        mjy = alj;
                        mjx = qo - khp;
                        break;
                    }
                    khp += line.len() + 1;
                }
                
                let hn = if mns.is_some() && mns.unwrap().0 == mjy {
                    mjx + 1
                } else {
                    mjx
                };
                let xrz = mjy.ao(px);
                let ae = hdm + 15 + xrz * gy;
                let cx = awm + 42 + hn * 8 * gcv;
                if (cma / 5) % 2 == 0 {
                    for cq in ae..ae + 16 {
                        if cq < i && cx < d && cx + 2 < d {
                            k[cq * d + cx] = 0xFF00FF88;
                            k[cq * d + cx + 1] = 0xFF00FF88;
                        }
                    }
                }
            }

            
            let uo = i - 40;
            for x in uo..i {
                for y in 0..d { k[x * d + y] = 0xFF111111; }
            }
            {
                let rrp = {
                    let mut ncy = 0usize;
                    let mut glr = 1usize;
                    for (yam, line) in cao.iter().cf() {
                        if ncy + line.len() >= qo { break; }
                        ncy += line.len() + 1;
                        glr += 1;
                    }
                    glr
                };
                let status = alloc::format!("Ln {}  |  {} lines  |  TrustLang", rrp, cao.len());
                ri(k, d, i, czm, uo + 12, &status, 0xFF00CC66, 1);
            }

            mb(k, d, i);
        };

        
        while qo < aqo {
            if let Some(eh) = crate::keyboard::xw() {
                if eh == 0x1B { return; }
                if eh == b' ' || eh == b'\r' || eh == b'\n' { qo = aqo; break; }
            }

            
            let mut nlh = false;
            for &(pwv, qag) in xnu.iter() {
                if qo == pwv && pwv < aqo {
                    
                    let mut khq = 0usize;
                    let mut mko = 0usize;
                    let mut mkn = 0usize;
                    for (alj, line) in cao.iter().cf() {
                        if khq + line.len() > qo {
                            mko = alj;
                            mkn = qo - khq;
                            break;
                        }
                        khq += line.len() + 1;
                    }

                    
                    ehm(k, d, i, ws, yg, cma,
                        qo, Some((mko, mkn, qag)));
                    cma += 1;
                    crate::cpu::tsc::rd(120);

                    
                    ehm(k, d, i, ws, yg, cma,
                        qo, Some((mko, mkn, qag)));
                    cma += 1;
                    crate::cpu::tsc::rd(400);

                    
                    ehm(k, d, i, ws, yg, cma,
                        qo, None);
                    cma += 1;
                    crate::cpu::tsc::rd(150);

                    
                    nlh = true;
                    break;
                }
            }

            
            let r = inf.get(qo).map(|&(_, _, r)| r).unwrap_or(' ');
            let uuc = inf.get(qo + 1).map(|&(_, _, r)| r);

            
            let mut azn: u64 = 20;

            
            if r == '\n' {
                azn = 80 + (exu(&mut ajn) % 120) as u64; 
            }
            
            else if r == '{' || (uuc == Some('}')) {
                azn = 150 + (exu(&mut ajn) % 200) as u64;
            }
            
            else if r == '/' {
                azn = 40 + (exu(&mut ajn) % 60) as u64;
            }
            
            else if r == ' ' {
                azn = 15 + (exu(&mut ajn) % 40) as u64;
            }
            
            else if r == '(' || r == ')' || r == ';' || r == ',' {
                azn = 30 + (exu(&mut ajn) % 30) as u64;
            }
            
            else {
                azn = 18 + (exu(&mut ajn) % 25) as u64;
            }

            
            if exu(&mut ajn) % 100 < 5 {
                azn += 200 + (exu(&mut ajn) % 400) as u64;
            }

            
            if nlh {
                azn += 80;
            }

            
            ehm(k, d, i, ws, yg, cma, qo, None);
            cma += 1;

            
            qo += 1;
            crate::cpu::tsc::rd(azn);
        }

        
        ehm(k, d, i, ws, yg, cma, aqo, None);
        crate::cpu::tsc::rd(1200);

        
        
        
        {
            
            ehm(k, d, i, ws, yg, cma, aqo, None);

            
            let dkm = i - 120;
            let vbg = 80;
            let dkl = czm;
            let lrz = d - 2 * czm;

            
            for x in dkm..dkm + vbg {
                for y in dkl..dkl + lrz {
                    if x < i && y < d {
                        k[x * d + y] = 0xFF0A0E14;
                    }
                }
            }
            
            for y in dkl..dkl + lrz {
                if dkm < i { k[dkm * d + y] = 0xFF00FF44; }
            }
            
            ri(k, d, i, dkl + 8, dkm + 4, "OUTPUT", 0xFF888888, 1);

            
            ri(k, d, i, dkl + 8, dkm + 22, "$ trustlang compile youtube_dvd.tl", 0xFF00CC66, 1);
            mb(k, d, i);
            crate::cpu::tsc::rd(800);

            
            ri(k, d, i, dkl + 8, dkm + 38, "Compiling...", 0xFFAABBCC, 1);
            mb(k, d, i);
            crate::cpu::tsc::rd(1200);

            
            for x in dkm + 36..dkm + 56 {
                for y in dkl + 4..dkl + lrz - 4 {
                    if x < i && y < d {
                        k[x * d + y] = 0xFF0A0E14;
                    }
                }
            }
            ri(k, d, i, dkl + 8, dkm + 38, "Compiled successfully in 0.3s  (47 lines, 0 errors)", 0xFF00FF88, 1);
            
            ri(k, d, i, dkl + 8, dkm + 54, "Generated 284 bytecode instructions", 0xFF666666, 1);
            mb(k, d, i);
            crate::cpu::tsc::rd(2000);
        }

        
        

        
        crate::ramfs::fh(|fs| {
            let _ = fs.ns("/youtube_dvd.tl", iy.as_bytes());
        });

        
        {
            
            for ai in k.el() { *ai = 0xFF0A0A0A; }
            eba(k, d, i, ws, yg, cma);
            for ai in k.el() {
                let at = ((*ai >> 8) & 0xFF).v(15);
                *ai = 0xFF000000 | (at << 8);
            }

            
            let abx = 30usize;
            let aha = 20usize;
            let aog = d - 60;
            let biz = i - 40;
            
            for x in aha..aha + biz {
                for y in abx..abx + aog {
                    if x < i && y < d {
                        k[x * d + y] = 0xFF0D0D0D;
                    }
                }
            }
            
            for x in aha..aha + 28 {
                for y in abx..abx + aog {
                    if x < i && y < d {
                        k[x * d + y] = 0xFF1A1A1A;
                    }
                }
            }
            ri(k, d, i, abx + 12, aha + 6, "TrustOS Terminal", 0xFF00FF88, 1);
            
            for y in abx..abx + aog {
                if aha < i { k[aha * d + y] = 0xFF00FF44; }
                let bjj = (aha + biz - 1).v(i - 1);
                k[bjj * d + y] = 0xFF00FF44;
            }
            for x in aha..aha + biz {
                if x < i {
                    k[x * d + abx] = 0xFF00FF44;
                    let m = (abx + aog - 1).v(d - 1);
                    k[x * d + m] = 0xFF00FF44;
                }
            }

            let wg = abx + 16;
            let mut sl = aha + 40;

            
            ri(k, d, i, wg, sl, "TrustOS v2.0 - TrustLang Runtime", 0xFF00FF88, 1);
            sl += 20;
            ri(k, d, i, wg, sl, "Type 'help' for available commands.", 0xFF666666, 1);
            sl += 28;

            
            
            ri(k, d, i, wg, sl, "root", 0xFFFF0000, 1);
            ri(k, d, i, wg + 32, sl, "@", 0xFFFFFFFF, 1);
            ri(k, d, i, wg + 40, sl, "trustos", 0xFF00FF00, 1);
            ri(k, d, i, wg + 96, sl, ":/$ ", 0xFF00FF00, 1);
            ri(k, d, i, wg + 128, sl, "trustlang compile youtube_dvd.tl", 0xFFD4D4D4, 1);
            sl += 18;
            ri(k, d, i, wg, sl, "Compiled successfully in 0.3s", 0xFF00FF88, 1);
            sl += 18;
            ri(k, d, i, wg, sl, "Generated 284 bytecode instructions", 0xFF666666, 1);
            sl += 28;

            
            let gpx = sl;
            ri(k, d, i, wg, gpx, "root", 0xFFFF0000, 1);
            ri(k, d, i, wg + 32, gpx, "@", 0xFFFFFFFF, 1);
            ri(k, d, i, wg + 40, gpx, "trustos", 0xFF00FF00, 1);
            ri(k, d, i, wg + 96, gpx, ":/$ ", 0xFF00FF00, 1);
            mb(k, d, i);
            crate::cpu::tsc::rd(800);

            
            let wbe = "trustlang run youtube_dvd.tl";
            let ril = wg + 128;
            for (nc, r) in wbe.bw().cf() {
                aej(k, d, i, ril + nc * 8, gpx, r, 0xFFD4D4D4, 1);
                mb(k, d, i);
                let bc = 30 + (((nc as u32 * 7 + 13) ^ 0x5A) % 50) as u64;
                crate::cpu::tsc::rd(bc);
            }
            crate::cpu::tsc::rd(400);

            
            sl = gpx + 24;
            ri(k, d, i, wg, sl, "Running youtube_dvd.tl ...", 0xFFAABBCC, 1);
            mb(k, d, i);
            crate::cpu::tsc::rd(600);
        }

        
        match crate::trustlang::vw(iy) {
            Ok(an) => {
                if !an.is_empty() {
                    
                    
                    let uzu: alloc::vec::Vec<&str> = an.ak().collect();
                    aol(k);
                    np(k, d, i, 25, "OUTPUT", 0xFF00FF88, 3);
                    for (a, line) in uzu.iter().cf() {
                        let ct = 80 + a * 20;
                        if ct + 16 > i - 40 { break; }
                        let cr = if line.len() * 8 < d { (d - line.len() * 8) / 2 } else { 40 };
                        ri(k, d, i, cr, ct, line, 0xFFCCFFCC, 1);
                    }
                    mb(k, d, i);
                    crate::cpu::tsc::rd(otj);
                }
                if an.is_empty() {
                    
                    
                    if let Some((bgc, dnu, bgb, baz)) = crate::framebuffer::cey() {
                        let aaa = bgc as *mut u32;
                        let bgd = baz as usize;
                        for c in 0..i.v(bgb as usize) {
                            unsafe {
                                core::ptr::copy_nonoverlapping(
                                    aaa.add(c * bgd),
                                    k[c * d..].mw(),
                                    d,
                                );
                            }
                        }
                    }
                    
                    crate::cpu::tsc::rd(otj);
                }
            }
            Err(aa) => {
                aol(k);
                np(k, d, i, i / 2 - 20, "Runtime Error", 0xFFFF4444, 4);
                let sng = if aa.len() > 80 { &aa[..80] } else { &aa };
                np(k, d, i, i / 2 + 50, sng, 0xFFFF8888, 1);
                mb(k, d, i);
                crate::cpu::tsc::rd(3000);
            }
        }
    };

    
    
    
    

    crate::serial_println!("[TL_SHOWCASE] Starting TrustLang showcase -- YouTube DVD Screensaver");

    
    
    
    aol(&mut k);
    jqb(&mut k, d, i, &mut ws, &yg,
        &[("TrustLang", 0xFF00FF88, 6),
          ("", 0xFF000000, 1),
          ("Live Demo", 0xFF00CC66, 4),
          ("Programming Inside TrustOS", 0xFF008844, 2)],
        90, 200);   
    apq(&mut k, d, i, &mb);

    
    
    
    aol(&mut k);
    jqb(&mut k, d, i, &mut ws, &yg,
        &[("YouTube DVD Screensaver", 0xFFFF0000, 4),
          ("", 0xFF000000, 1),
          ("A bouncing 3D YouTube logo", 0xFFCCFFCC, 2),
          ("with 'Like & Subscribe' text.", 0xFFCCFFCC, 2),
          ("", 0xFF000000, 1),
          ("Written, compiled, and animated", 0xFF00FF88, 2),
          ("live inside the OS kernel.", 0xFF00FF88, 2),
          ("", 0xFF000000, 1),
          ("All in real-time. Zero dependencies.", 0xFF888888, 2)],
        70, 180);   
    apq(&mut k, d, i, &mb);

    
    
    
    aol(&mut k);
    jqb(&mut k, d, i, &mut ws, &yg,
        &[("How TrustLang Works", 0xFF00FF88, 4),
          ("", 0xFF000000, 1),
          ("tokenize()   Lexer -> Tokens", 0xFFFF7B72, 2),
          ("parse()      Tokens -> AST", 0xFFFFA657, 2),
          ("compile()    AST -> Bytecode", 0xFFA5D6FF, 2),
          ("execute()    Bytecode -> VM", 0xFF00FF88, 2),
          ("", 0xFF000000, 1),
          ("pixel() fill_rect() draw_text()", 0xFFFFD700, 2),
          ("flush() sleep() clear_screen()", 0xFFFFD700, 2)],
        60, 170);   
    apq(&mut k, d, i, &mb);

    
    
    
    aol(&mut k);
    wne(&mut k, d, i, &mut ws, &yg,
        "TrustCode  -  youtube_dvd.tl",
        r#"// YouTube DVD Screensaver in TrustLang!
fn main() {
    let screen_width = screen_w();
    let screen_height = screen_h();
    // Logo dimensions
    let logo_width = 200;
    let logo_height = 140;
    let total_height = logo_height + 70;
    // Starting position & velocity
    let mut pos_x = screen_width / 4;
    let mut pos_y = screen_height / 4;
    let mut speed_x = 4;
    let mut speed_y = 3;
    // Animate 300 frames (~10 seconds)
    let mut frame = 0;
    while frame < 300 {
        clear_screen(0, 0, 0);
        // 3D shadow offset
        fill_rect(pos_x + 8, pos_y + 8, logo_width, logo_height, 50, 0, 0);
        // Red YouTube rectangle
        fill_rect(pos_x, pos_y, logo_width, logo_height, 230, 0, 0);
        // 3D highlight on top + dark edge on bottom
        fill_rect(pos_x, pos_y, logo_width, 3, 255, 60, 60);
        fill_rect(pos_x, pos_y + logo_height - 3, logo_width, 3, 150, 0, 0);
        // Play button triangle (white)
        let center_x = pos_x + logo_width / 2;
        let center_y = pos_y + logo_height / 2;
        let mut row = 0;
        while row < 70 {
            let offset = row - 35;
            let mut dist = offset;
            if dist < 0 { dist = 0 - dist; }
            let bar_width = 40 * (35 - dist) / 35;
            if bar_width > 0 {
                fill_rect(center_x - 12, center_y - 35 + row, bar_width, 1, 255, 255, 255);
            }
            row = row + 1;
        }
        // Bouncing text below logo
        draw_text("LIKE AND", pos_x + 18, pos_y + logo_height + 12, 255, 255, 255, 2);
        draw_text("SUBSCRIBE!", pos_x + 5, pos_y + logo_height + 42, 255, 80, 80, 2);
        flush();
        sleep(33);
        // Move & bounce off screen edges
        pos_x = pos_x + speed_x;
        pos_y = pos_y + speed_y;
        if pos_x + logo_width > screen_width { speed_x = 0 - speed_x; }
        if pos_x < 0 { speed_x = 0 - speed_x; }
        if pos_y + total_height > screen_height { speed_y = 0 - speed_y; }
        if pos_y < 0 { speed_y = 0 - speed_y; }
        frame = frame + 1;
    }
}"#,
        "New file: /youtube_dvd.tl",
        30,     
        80,     
        3000);  
    apq(&mut k, d, i, &mb);

    
    
    
    aol(&mut k);
    jqb(&mut k, d, i, &mut ws, &yg,
        &[("TrustLang", 0xFF00FF88, 6),
          ("", 0xFF000000, 1),
          ("Lexer > Parser > Compiler > VM", 0xFFAADDAA, 2),
          ("Real-time graphics. Zero deps.", 0xFFAADDAA, 2),
          ("", 0xFF000000, 1),
          ("Built into TrustOS.", 0xFF00CC66, 3),
          ("", 0xFF000000, 1),
          ("github.com/nathan237/TrustOS", 0xFF00FF88, 2)],
        80, 250);   
    apq(&mut k, d, i, &mb);

    
    aol(&mut k);
    mb(&k, d, i);
    if !afk {
        crate::framebuffer::afi(false);
    }
    crate::framebuffer::clear();
    crate::serial_println!("[TL_SHOWCASE] Showcase complete");
}














pub(super) fn rjr() {
    let (kp, kl) = crate::framebuffer::yn();
    let d = kp as usize;
    let i = kl as usize;

    let afk = crate::framebuffer::bre();
    if !afk {
        crate::framebuffer::beo();
        crate::framebuffer::afi(true);
    }

    let mut k = alloc::vec![0u32; d * i];

    

    let aej = |k: &mut [u32], d: usize, i: usize,
                         cx: usize, ae: usize, r: char, s: u32, bv: usize| {
        let ka = crate::framebuffer::font::ada(r);
        for (br, &fs) in ka.iter().cf() {
            for ga in 0..8u32 {
                if fs & (0x80 >> ga) != 0 {
                    for cq in 0..bv {
                        for cr in 0..bv {
                            let y = cx + ga as usize * bv + cr;
                            let x = ae + br * bv + cq;
                            if y < d && x < i { k[x * d + y] = s; }
                        }
                    }
                }
            }
        }
    };

    let ri = |k: &mut [u32], d: usize, i: usize,
                        b: usize, c: usize, text: &str, s: u32, bv: usize| {
        for (a, r) in text.bw().cf() {
            aej(k, d, i, b + a * 8 * bv, c, r, s, bv);
        }
    };

    let np = |k: &mut [u32], d: usize, i: usize,
                              c: usize, text: &str, s: u32, bv: usize| {
        let qd = text.len() * 8 * bv;
        let cr = if qd < d { (d - qd) / 2 } else { 0 };
        for (a, r) in text.bw().cf() {
            aej(k, d, i, cr + a * 8 * bv, c, r, s, bv);
        }
    };

    let mb = |k: &[u32], d: usize, i: usize| {
        if let Some((bgc, dnu, bgb, baz)) = crate::framebuffer::cey() {
            let aaa = bgc as *mut u32;
            let bgd = baz as usize;
            for c in 0..i.v(bgb as usize) {
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        k[c * d..].fq(), aaa.add(c * bgd), d);
                }
            }
        }
        crate::framebuffer::sv();
    };

    let aol = |k: &mut [u32]| {
        for ai in k.el() { *ai = 0xFF000000; }
    };

    
    let lx = |k: &mut [u32], d: usize, i: usize,
                     b: usize, c: usize, yq: usize, aff: usize, s: u32| {
        for bg in 0..aff {
            for dx in 0..yq {
                let y = b + dx;
                let x = c + bg;
                if y < d && x < i { k[x * d + y] = s; }
            }
        }
    };

    let aps: u64 = 30;

    let apq = |k: &mut [u32], d: usize, i: usize,
                   bge: &dyn Fn(&[u32], usize, usize)| {
        for _ in 0..40 {
            for y in k.el() {
                let m = ((*y >> 16) & 0xFF).ao(8);
                let at = ((*y >> 8) & 0xFF).ao(8);
                let o = (*y & 0xFF).ao(8);
                *y = 0xFF000000 | (m << 16) | (at << 8) | o;
            }
            bge(k, d, i);
            crate::cpu::tsc::rd(aps);
        }
        for ai in k.el() { *ai = 0xFF000000; }
        bge(k, d, i);
        crate::cpu::tsc::rd(400);
    };

    
    

    
    let dyx = |k: &mut [u32], d: usize, i: usize, frame: u32| {
        
        
        let ib = (frame % 160) as u32;
        let xg = if ib < 80 { ib / 2 } else { (160 - ib) / 2 };
        let fqp = ((frame + 40) % 120) as u32;
        let lwd = if fqp < 60 { fqp / 2 } else { (120 - fqp) / 2 };
        for c in 0..i {
            let dnr = (c as u32 * 40) / i as u32;
            for b in 0..d {
                let iht = (b as u32 * 10) / d as u32;
                let m = (dnr / 4 + lwd / 3).v(40);
                let at = (iht / 3).v(15);
                let o = (dnr + xg + iht / 2).v(80);
                k[c * d + b] = 0xFF000000 | (m << 16) | (at << 8) | o;
            }
        }
    };

    
    let gaz = |k: &mut [u32], d: usize, i: usize, frame: u32| {
        let jc = (frame as usize * 2) % i;
        for c in 0..i {
            let cq = (c + jc) % i;
            let mhv = (cq / 4) % 2 == 0;
            for b in 0..d {
                let bdm = if mhv { 35u32 } else { 15 };
                let ceq = if (cq % 60) < 2 { 30u32 } else { 0 };
                let m = (bdm + ceq).v(65);
                let at = 2;
                let o = 5;
                k[c * d + b] = 0xFF000000 | (m << 16) | (at << 8) | o;
            }
        }
    };

    
    let myy = |k: &mut [u32], d: usize, i: usize, eln: u32| {
        for c in 0..i {
            for b in 0..d {
                let uyf = (b % 20 < 2) && (c % 20 < 2);
                let s = if uyf { 0xFF0A1A3A } else { 0xFF060E1E };
                k[c * d + b] = s;
            }
        }
    };

    
    let gbb = |k: &mut [u32], d: usize, i: usize, frame: u32| {
        
        for y in k.el() {
            let m = ((*y >> 16) & 0xFF).ao(8);
            let at = ((*y >> 8) & 0xFF).ao(12);
            let o = (*y & 0xFF).ao(8);
            *y = 0xFF000000 | (m << 16) | (at << 8) | o;
        }
        
        for a in 0..24u32 {
            let dv = (a.hx(2654435761).cn(frame.hx(37))) as usize;
            let y = (dv.hx(7919)) % d;
            let mal = (frame as usize + dv) % i;
            let x = i.ao(mal);
            let kt = (50 + (dv % 40)) as u32;
            if y < d && x < i {
                k[x * d + y] = 0xFF000000 | (kt / 4 << 16) | (kt << 8) | (kt / 3);
                if y + 1 < d { k[x * d + y + 1] = 0xFF000000 | (kt << 8); }
            }
        }
    };

    
    let mzc = |k: &mut [u32], d: usize, i: usize, frame: u32| {
        for ai in k.el() { *ai = 0xFF050510; }
        
        for a in 0..80u32 {
            let cr = ((a.hx(7919)) as usize) % d;
            let cq = ((a.hx(104729)) as usize) % i;
            let mnk = ((frame.cn(a * 17)) % 30) as u32;
            let aaj = if mnk < 15 { 40 + mnk * 3 } else { 40 + (30 - mnk) * 3 };
            let aaj = aaj.v(120);
            if cr < d && cq < i {
                k[cq * d + cr] = 0xFF000000 | (aaj << 16) | (aaj << 8) | aaj;
            }
        }
    };

    
    let gaw = |k: &mut [u32], d: usize, i: usize, frame: u32| {
        for ai in k.el() { *ai = 0xFF0A0A14; }
        
        let pvn = 0xFF0F2818u32;
        let mtp = 0xFF00AA44u32;
        for a in 0..20u32 {
            let ty = ((a.hx(7919) as usize) % i) & !3;
            let gx = ((a.hx(104729) as usize) % d) & !3;
            
            if ty < i {
                for b in 0..d {
                    k[ty * d + b] = pvn;
                }
            }
            
            if gx < d {
                for c in 0..i {
                    k[c * d + gx] = pvn;
                }
            }
        }
        
        let jko = ((frame as usize * 3) % i) & !3;
        if jko < i {
            let ars = (d / 4).v(120);
            let dav = (frame as usize * 5) % d;
            for dx in 0..ars {
                let y = (dav + dx) % d;
                k[jko * d + y] = mtp;
                if jko + 1 < i { k[(jko + 1) * d + y] = mtp; }
            }
        }
    };

    
    let hal = |k: &mut [u32], d: usize, i: usize, frame: u32| {
        let fmy = (frame as u32).v(60); 
        for c in 0..i {
            let dnr = c as u32 * 100 / i as u32; 
            let fyn = if dnr > 50 { (dnr - 50).v(50) + fmy } else { fmy / 2 };
            let m = (fyn * 2).v(90);
            let at = (fyn * 3 / 4).v(45);
            let o = (20u32.ao(fyn / 3)).v(30);
            for b in 0..d {
                k[c * d + b] = 0xFF000000 | (m << 16) | (at << 8) | o;
            }
        }
        
        let ppt = d / 2;
        let ppu = i - (fmy as usize * i / 200);
        let ezf = 80usize + fmy as usize;
        for bg in 0..ezf {
            for dx in 0..ezf {
                let ass = dx * dx + bg * bg;
                if ass < ezf * ezf {
                    let hj = (ezf * ezf - ass) * 60 / (ezf * ezf);
                    let hj = hj as u32;
                    for (cr, cq) in [(ppt + dx, ppu.nj(bg)),
                                     (ppt.nj(dx), ppu.nj(bg))] {
                        if cr < d && cq < i {
                            let xy = k[cq * d + cr];
                            let bqm = ((xy >> 16) & 0xFF) + hj;
                            let fhj = ((xy >> 8) & 0xFF) + hj * 2 / 3;
                            let ebc = (xy & 0xFF) + hj / 4;
                            k[cq * d + cr] = 0xFF000000
                                | (bqm.v(255) << 16)
                                | (fhj.v(255) << 8)
                                | ebc.v(255);
                        }
                    }
                }
            }
        }
    };

    
    let mut ws: alloc::vec::Vec<u16> =
        (0..d / 8 + 1).map(|a| ((a * 37 + 13) % i) as u16).collect();
    let yg: alloc::vec::Vec<u8> =
        (0..d / 8 + 1).map(|a| (((a * 7 + 3) % 4) + 1) as u8).collect();

    let eba = |k: &mut [u32], d: usize, i: usize,
                     ec: &mut [u16], arz: &[u8], frame: u32| {
        for il in k.el() {
            let at = ((*il >> 8) & 0xFF) as u32;
            if at > 0 { *il = 0xFF000000 | (at.ao(6) << 8); }
        }
        for nc in 0..ec.len() {
            let b = nc * 8;
            if b >= d { continue; }
            ec[nc] = ec[nc].cn(arz[nc] as u16);
            if ec[nc] as usize >= i { ec[nc] = 0; }
            let c = ec[nc] as usize;
            let r = (((frame as usize + nc * 13) % 94) + 33) as u8 as char;
            let ka = crate::framebuffer::font::ada(r);
            for (br, &fs) in ka.iter().cf() {
                let x = c + br;
                if x >= i { break; }
                for ga in 0..8u32 {
                    if fs & (0x80 >> ga) != 0 {
                        let y = b + ga as usize;
                        if y < d { k[x * d + y] = 0xFF00FF44; }
                    }
                }
            }
        }
    };

    
    
    
    
    let dvz = |k: &mut [u32], d: usize, i: usize,
                      ws: &mut [u16], yg: &[u8],
                      ak: &[(&str, u32, usize)],
                      hsc: u64, lci: u32, qpg: u8| {
        let aqo: usize = ak.iter().map(|(ab, _, _)| ab.len()).sum();
        let hkm = (hsc / aps).am(1) as u32;
        let gvh = aqo as u32 * hkm;
        let agc = gvh + lci;
        let mut frame = 0u32;
        while frame < agc {
            if let Some(eh) = crate::keyboard::xw() {
                if eh == 0x1B { return; }
                if eh == b' ' || eh == b'\r' || eh == b'\n' { break; }
            }
            
            match qpg {
                1 => dyx(k, d, i, frame),
                2 => gaz(k, d, i, frame),
                3 => myy(k, d, i, frame),
                4 => gbb(k, d, i, frame),
                5 => mzc(k, d, i, frame),
                6 => gaw(k, d, i, frame),
                7 => hal(k, d, i, frame),
                8 => eba(k, d, i, ws, yg, frame),
                _ => { for ai in k.el() { *ai = 0xFF000000; } }
            }
            let qo = (frame / hkm) as usize;
            let aku: usize = ak.iter().map(|(_, _, e)| 16 * e + 12).sum();
            let mut c = if aku < i { (i - aku) / 2 } else { 20 };
            let mut bbh = 0usize;
            for &(text, s, bv) in ak {
                let qd = text.len() * 8 * bv;
                let cr = if qd < d { (d - qd) / 2 } else { 0 };
                for (a, r) in text.bw().cf() {
                    if bbh + a >= qo { break; }
                    aej(k, d, i, cr + a * 8 * bv, c, r, s, bv);
                }
                
                if qo > bbh && qo < bbh + text.len() {
                    let nc = qo - bbh;
                    let cx = cr + nc * 8 * bv;
                    if (frame / 8) % 2 == 0 {
                        for ae in c..c + 16 * bv {
                            if ae < i && cx + 2 < d {
                                k[ae * d + cx] = 0xFFFFFFFF;
                                k[ae * d + cx + 1] = 0xFFFFFFFF;
                            }
                        }
                    }
                }
                bbh += text.len();
                c += 16 * bv + 12;
            }
            mb(k, d, i);
            frame += 1;
            crate::cpu::tsc::rd(aps);
        }
    };

    crate::serial_println!("[FILM] TrustOS Film started");

    
    
    
    aol(&mut k);
    dvz(&mut k, d, i, &mut ws, &yg,
        &[("ACT I", 0xFF88CCFF, 5)],
        50, 30, 1);
    apq(&mut k, d, i, &mb);

    
    {
        let avv = 2u32;
        let asd = "You use a computer";
        let bow = "every single day.";
        let aqo = asd.len() + bow.len();
        let agc = aqo as u32 * avv + 50;
        
        let mut xuu: [(i32,i32,usize,usize,u32,i32,i32); 6] = [
            (80, 40, 120, 80, 0xFF3355AA, 2, 1),
            (d as i32 - 220, 90, 100, 70, 0xFF55AA33, -1, 2),
            (180, i as i32 - 180, 130, 85, 0xFFAA5533, 1, -1),
            (d as i32 / 2, 60, 110, 75, 0xFF8844CC, -2, 1),
            (40, i as i32 / 2, 125, 80, 0xFF4488CC, 1, -2),
            (d as i32 - 160, i as i32 / 2 + 40, 100, 65, 0xFFCC8844, -1, -1),
        ];
        for frame in 0..agc {
            if let Some(eh) = crate::keyboard::xw() {
                if eh == 0x1B { break; }
                if eh == b' ' || eh == b'\r' || eh == b'\n' { break; }
            }
            dyx(&mut k, d, i, frame);
            
            for yi in 0..6 {
                let ep = &mut xuu[yi];
                
                if frame < (yi as u32) * 8 { continue; }
                ep.0 += ep.5;
                ep.1 += ep.6;
                if ep.0 < 0 || ep.0 + ep.2 as i32 > d as i32 { ep.5 = -ep.5; ep.0 += ep.5; }
                if ep.1 < 0 || ep.1 + ep.3 as i32 > i as i32 { ep.6 = -ep.6; ep.1 += ep.6; }
                let fx = ep.0.am(0) as usize;
                let lw = ep.1.am(0) as usize;
                let ekv = ep.4;
                let bfu = ((ekv >> 16) & 0xFF) / 3;
                let xug = ((ekv >> 8)  & 0xFF) / 3;
                let mqj = (ekv & 0xFF) / 3;
                let tp = 0xFF000000 | (bfu << 16) | (xug << 8) | mqj;
                lx(&mut k, d, i, fx, lw, ep.2, ep.3, tp);
                lx(&mut k, d, i, fx, lw, ep.2, 10, ekv);
                
                for alj in 0..3usize {
                    let ct = lw + 16 + alj * 12;
                    if ct + 5 < lw + ep.3 {
                        lx(&mut k, d, i, fx + 6, ct, ep.2.ao(12), 5, 0xFF222233);
                    }
                }
            }
            
            let qo = (frame / avv) as usize;
            let bv = 3usize;
            let gy = 16 * bv + 12;
            let dp = i / 2 - gy;
            let jz = i / 2 + 4;
            let cnj = asd.len() * 8 * bv;
            let asa = if cnj < d { (d - cnj) / 2 } else { 0 };
            for (a, r) in asd.bw().cf() {
                if a >= qo { break; }
                aej(&mut k, d, i, asa + a * 8 * bv, dp, r, 0xFFFFFFFF, bv);
            }
            if qo > asd.len() {
                let cch = bow.len() * 8 * bv;
                let amy = if cch < d { (d - cch) / 2 } else { 0 };
                let ang = qo - asd.len();
                for (a, r) in bow.bw().cf() {
                    if a >= ang { break; }
                    aej(&mut k, d, i, amy + a * 8 * bv, jz, r, 0xFFFFFFFF, bv);
                }
            }
            mb(&k, d, i);
            crate::cpu::tsc::rd(aps);
        }
    }
    apq(&mut k, d, i, &mb);

    
    {
        let avv = 2u32;
        let asd = "Do you really know";
        let bow = "what it does?";
        let aqo = asd.len() + bow.len();
        let agc = aqo as u32 * avv + 60;
        let orv = d / 10;
        let mut hwj: alloc::vec::Vec<i32> = (0..orv).map(|a| -((a * 37 % 200) as i32)).collect();
        for frame in 0..agc {
            if let Some(eh) = crate::keyboard::xw() {
                if eh == 0x1B { break; }
                if eh == b' ' || eh == b'\r' || eh == b'\n' { break; }
            }
            dyx(&mut k, d, i, frame);
            
            let ig = 1 + (frame / 30) as i32;
            for exb in 0..orv {
                hwj[exb] += ig + (exb as i32 % 3);
                if hwj[exb] > i as i32 { hwj[exb] = -(exb as i32 * 13 % 60); }
                if hwj[exb] >= 0 {
                    let y = exb * 10 + 2;
                    let x = hwj[exb] as usize;
                    let aaj = 0xFF000000 | (0x40 << 16) | (0x60 << 8) | 0xFF;
                    if y < d && x < i {
                        aej(&mut k, d, i, y, x, '?', aaj, 1);
                    }
                }
            }
            
            let qo = (frame / avv) as usize;
            let bv = 3usize;
            let dp = i / 2 - 40;
            let jz = i / 2 + 20;
            let cnj = asd.len() * 8 * bv;
            let asa = if cnj < d { (d - cnj) / 2 } else { 0 };
            for (a, r) in asd.bw().cf() {
                if a >= qo { break; }
                aej(&mut k, d, i, asa + a * 8 * bv, dp, r, 0xFFCCCCCC, bv);
            }
            if qo > asd.len() {
                let cch = bow.len() * 8 * bv;
                let amy = if cch < d { (d - cch) / 2 } else { 0 };
                let ang = qo - asd.len();
                for (a, r) in bow.bw().cf() {
                    if a >= ang { break; }
                    aej(&mut k, d, i, amy + a * 8 * bv, jz, r, 0xFFFF9944, 4);
                }
            }
            mb(&k, d, i);
            crate::cpu::tsc::rd(aps);
        }
    }
    apq(&mut k, d, i, &mb);

    
    {
        let avv = 2u32;
        let asd = "The honest answer...";
        let bow = "is no.";
        let dmw = (asd.len() + bow.len()) as u32 * avv;
        let wmq = 50u32;
        let agc = dmw + wmq;
        
        let rqf: [(i32, i32); 12] = [
            (3,0),(-3,0),(0,3),(0,-3),(2,2),(-2,2),(2,-2),(-2,-2),
            (3,1),(-3,1),(1,-3),(-1,3),
        ];
        for frame in 0..agc {
            if let Some(eh) = crate::keyboard::xw() {
                if eh == 0x1B { break; }
                if eh == b' ' || eh == b'\r' || eh == b'\n' { break; }
            }
            gaz(&mut k, d, i, frame);
            
            let qo = (frame / avv) as usize;
            let mca = 3usize;
            let dp = i / 2 - 60;
            let cnj = asd.len() * 8 * mca;
            let asa = if cnj < d { (d - cnj) / 2 } else { 0 };
            for (a, r) in asd.bw().cf() {
                if a >= qo { break; }
                aej(&mut k, d, i, asa + a * 8 * mca, dp, r, 0xFF888888, mca);
            }
            if qo > asd.len() {
                let mcb = 5usize;
                let cch = bow.len() * 8 * mcb;
                let amy = if cch < d { (d - cch) / 2 } else { 0 };
                let ang = qo - asd.len();
                for (a, r) in bow.bw().cf() {
                    if a >= ang { break; }
                    aej(&mut k, d, i, amy + a * 8 * mcb, i / 2, r, 0xFFFF4444, mcb);
                }
            }
            
            if frame > dmw {
                let li = frame - dmw;
                let cx = d / 2;
                let ae = i / 2;
                for &(qxc, qxd) in rqf.iter() {
                    for gu in 0..(li * 4) as i32 {
                        let y = (cx as i32 + qxc * gu).am(0) as usize;
                        let x = (ae as i32 + qxd * gu).am(0) as usize;
                        if y < d && x < i {
                            k[x * d + y] = 0xFFFFFFFF;
                            if y + 1 < d { k[x * d + y + 1] = 0xFFFFDDDD; }
                            if x + 1 < i { k[(x + 1) * d + y] = 0xFFFFDDDD; }
                        }
                    }
                }
            }
            mb(&k, d, i);
            crate::cpu::tsc::rd(aps);
        }
    }
    apq(&mut k, d, i, &mb);

    
    
    
    aol(&mut k);
    dvz(&mut k, d, i, &mut ws, &yg,
        &[("ACT II", 0xFFFF6644, 5),
          ("", 0xFF000000, 1),
          ("The Problem", 0xFFFF4444, 3)],
        50, 30, 2);
    apq(&mut k, d, i, &mb);

    
    {
        let avv = 2u32;
        let fna: [(&str, u32, usize); 5] = [
            ("Your computer runs on", 0xFFCCCCCC, 2),
            ("an operating system.", 0xFFCCCCCC, 2),
            ("", 0xFF000000, 1),
            ("It controls", 0xFFCCCCCC, 2),
            ("EVERYTHING.", 0xFFFF6644, 4),
        ];
        let aqo: usize = fna.iter().map(|(ab,_,_)| ab.len()).sum();
        let agc = aqo as u32 * avv + 70;
        for frame in 0..agc {
            if let Some(eh) = crate::keyboard::xw() {
                if eh == 0x1B { break; }
                if eh == b' ' || eh == b'\r' || eh == b'\n' { break; }
            }
            gaz(&mut k, d, i, frame);
            
            let suv = (frame as usize * 3).v(i);
            for sc in 0..suv {
                if sc >= i { break; }
                
                for szs in 0..d / 12 {
                    let jf = szs * 12;
                    let dv = (sc.hx(7919) + jf.hx(104729) + frame as usize * 37) % 100;
                    if dv < 15 {
                        let r = if dv < 8 { '0' } else { '1' };
                        let aaj = (20 + (dv * 2)) as u32;
                        let s = 0xFF000000 | (aaj << 16) | ((aaj / 2) << 8) | (aaj / 4);
                        aej(&mut k, d, i, jf, sc, r, s, 1);
                    }
                }
            }
            
            let qo = (frame / avv) as usize;
            let aku: usize = fna.iter().map(|(_,_,e)| 16 * e + 12).sum();
            let mut c = if aku < i { (i - aku) / 2 } else { 20 };
            let mut bbh = 0usize;
            for &(text, s, bv) in fna.iter() {
                let qd = text.len() * 8 * bv;
                let cr = if qd < d { (d - qd) / 2 } else { 0 };
                
                if !text.is_empty() {
                    lx(&mut k, d, i, cr.ao(4), c.ao(2),
                        qd + 8, 16 * bv + 4, 0xCC000000);
                }
                for (a, r) in text.bw().cf() {
                    if bbh + a >= qo { break; }
                    aej(&mut k, d, i, cr + a * 8 * bv, c, r, s, bv);
                }
                bbh += text.len();
                c += 16 * bv + 12;
            }
            mb(&k, d, i);
            crate::cpu::tsc::rd(aps);
        }
    }
    apq(&mut k, d, i, &mb);

    
    {
        let avv = 2u32;
        let fna: [(&str, u32, usize); 5] = [
            ("But nobody knows", 0xFFCCCCCC, 3),
            ("what's inside it.", 0xFFCCCCCC, 3),
            ("", 0xFF000000, 1),
            ("Not even the people", 0xFFFF4444, 2),
            ("who wrote it.", 0xFFFF4444, 2),
        ];
        let aqo: usize = fna.iter().map(|(ab,_,_)| ab.len()).sum();
        let dmw = aqo as u32 * avv;
        let vtl = 60u32;
        let agc = dmw + vtl;
        
        let hgm: [(&str, usize); 6] = [
            ("Source code: kernel/mm/init.c", 60),
            ("Author: CLASSIFIED", 140),
            ("Memory manager: UNKNOWN", 220),
            ("Security audit: NONE PERFORMED", 300),
            ("Bug count: UNTRACKED", 380),
            ("Last review: NEVER", 460),
        ];
        for frame in 0..agc {
            if let Some(eh) = crate::keyboard::xw() {
                if eh == 0x1B { break; }
                if eh == b' ' || eh == b'\r' || eh == b'\n' { break; }
            }
            gaz(&mut k, d, i, frame);
            
            let kql = 30usize;
            for &(line, bg) in hgm.iter() {
                if bg < i {
                    ri(&mut k, d, i, kql, bg, line, 0xFF445566, 1);
                }
            }
            
            if frame > dmw {
                let li = frame - dmw;
                for (di, &(yan, bg)) in hgm.iter().cf() {
                    let bmv = di as u32 * 6;
                    if li > bmv {
                        let lo = ((li - bmv) as usize * 12).v(280);
                        if bg < i {
                            lx(&mut k, d, i, kql, bg.ao(2),
                                lo, 14, 0xFF000000);
                            if lo > 80 {
                                ri(&mut k, d, i, kql + 4, bg,
                                    "REDACTED", 0xFFFF2222, 1);
                            }
                        }
                    }
                }
            }
            
            let qo = (frame / avv) as usize;
            let xgd = d / 2 + 20;
            let aku: usize = fna.iter().map(|(_,_,e)| 16 * e + 12).sum();
            let mut c = if aku < i { (i - aku) / 2 } else { 20 };
            let mut bbh = 0usize;
            for &(text, s, bv) in fna.iter() {
                let cr = xgd;
                for (a, r) in text.bw().cf() {
                    if bbh + a >= qo { break; }
                    aej(&mut k, d, i, cr + a * 8 * bv, c, r, s, bv);
                }
                bbh += text.len();
                c += 16 * bv + 12;
            }
            mb(&k, d, i);
            crate::cpu::tsc::rd(aps);
        }
    }
    apq(&mut k, d, i, &mb);

    
    {
        let mxu: [(&str, u32, u32); 4] = [
            ("Windows",  50_000_000, 0xFFFF4444),
            ("macOS",    30_000_000, 0xFFFFAA22),
            ("Linux",    28_000_000, 0xFFFF8800),
            ("TrustOS",     120_000, 0xFF00FF88),
        ];
        let aki = 50_000_000u32;
        let gar = d * 3 / 5;
        let kcb = 40usize;
        let mxy = 80usize;
        let vc = i / 2 - (mxu.len() * mxy) / 2;
        let dis = 40usize;

        let mut mfl: i32 = 0;
        let mut jps: i32 = 0;

        for frame in 0..160u32 {
            if let Some(eh) = crate::keyboard::xw() {
                if eh == 0x1B { break; }
                if eh == b' ' || eh == b'\r' || eh == b'\n' { break; }
            }
            myy(&mut k, d, i, frame);

            let li = if frame < 30 { 0u32 }
                else { ((frame - 30) * 100 / 70).v(100) };

            
            if frame > 100 && frame < 130 {
                let dv = frame.hx(7919) as i32;
                mfl = (dv % 7) - 3;
                jps = ((dv / 7) % 5) - 2;
            } else {
                mfl = 0;
                jps = 0;
            }

            np(&mut k, d, i,
                (30i32 + jps) as usize,
                "Lines of Code per OS", 0xFFFFFFFF, 3);

            for (a, &(j, ap, s)) in mxu.iter().cf() {
                let c = ((vc + a * mxy) as i32 + jps).am(0) as usize;
                let muc = (dis as i32 + mfl).am(0) as usize;
                ri(&mut k, d, i, muc, c + 10,
                    j, 0xFFFFFFFF, 2);
                let ajx = (muc + 170).v(d.ao(10));
                lx(&mut k, d, i, ajx, c,
                    gar, kcb, 0xFF111122);
                let szd = (ap as usize * gar) / aki as usize;
                let xaz = szd.am(12);
                let geb = xaz * li as usize / 100;
                lx(&mut k, d, i, ajx, c,
                    geb, kcb, s);

                
                if a == 3 && frame > 100 && frame < 110 {
                    let ceq = 0xFF88FFAA;
                    lx(&mut k, d, i, ajx, c,
                        geb + 4, kcb + 4, ceq);
                }

                if frame > 70 {
                    let cu = if ap >= 1_000_000 {
                        alloc::format!("{}M", ap / 1_000_000)
                    } else {
                        alloc::format!("{}K", ap / 1000)
                    };
                    ri(&mut k, d, i, ajx + geb + 10,
                        c + 10, &cu, 0xFFFFFFFF, 2);
                }
            }
            mb(&k, d, i);
            crate::cpu::tsc::rd(aps);
        }
    }
    apq(&mut k, d, i, &mb);

    
    aol(&mut k);
    dvz(&mut k, d, i, &mut ws, &yg,
        &[("50 million vs 120 thousand.", 0xFFFFFFFF, 3),
          ("", 0xFF000000, 1),
          ("Like comparing a city", 0xFFCCCCCC, 2),
          ("to a single house.", 0xFF00FF88, 2),
          ("", 0xFF000000, 1),
          ("Except the house", 0xFFCCCCCC, 2),
          ("does everything.", 0xFF00FF88, 3)],
        50, 80, 2);
    apq(&mut k, d, i, &mb);

    
    
    
    aol(&mut k);
    dvz(&mut k, d, i, &mut ws, &yg,
        &[("ACT III", 0xFF00FF88, 5),
          ("", 0xFF000000, 1),
          ("The Solution", 0xFF00CC66, 3)],
        50, 30, 4);
    apq(&mut k, d, i, &mb);

    
    {
        let avv = 2u32;
        let asd = "What if one person";
        let bow = "could understand ALL of it?";
        let aqo = asd.len() + bow.len();
        let agc = aqo as u32 * avv + 60;
        
        let vqn: [(i32, i32); 16] = [
            (4,0),(-4,0),(0,4),(0,-4),(3,3),(-3,3),(3,-3),(-3,-3),
            (4,1),(4,-1),(-4,1),(-4,-1),(1,4),(1,-4),(-1,4),(-1,-4),
        ];
        for frame in 0..agc {
            if let Some(eh) = crate::keyboard::xw() {
                if eh == 0x1B { break; }
                if eh == b' ' || eh == b'\r' || eh == b'\n' { break; }
            }
            gbb(&mut k, d, i, frame);
            
            let cx = d / 2;
            let ae = i / 2;
            
            let ib = (frame % 40) as u32;
            let xg = if ib < 20 { ib * 4 } else { (40 - ib) * 4 };
            
            let gqh = 40 + (frame / 2) as i32;
            for &(rdx, vre) in vqn.iter() {
                for gu in 0..gqh {
                    let y = (cx as i32 + rdx * gu).am(0) as usize;
                    let x = (ae as i32 + vre * gu).am(0) as usize;
                    if y < d && x < i {
                        let ckj = (gqh - gu) as u32 * 3;
                        let aaj = (xg + ckj).v(180);
                        let m = aaj;
                        let at = (aaj * 3 / 4).v(140);
                        let o = (aaj / 3).v(60);
                        let xy = k[x * d + y];
                        let bqm = ((xy >> 16) & 0xFF) + m;
                        let fhj = ((xy >> 8) & 0xFF) + at;
                        let ebc = (xy & 0xFF) + o;
                        k[x * d + y] = 0xFF000000
                            | (bqm.v(255) << 16)
                            | (fhj.v(255) << 8)
                            | ebc.v(255);
                    }
                }
            }
            
            let ece = 15 + (xg / 4) as usize;
            for bg in 0..ece {
                for dx in 0..ece {
                    if dx * dx + bg * bg < ece * ece {
                        for &(cr, cq) in &[(cx+dx, ae+bg),(cx+dx, ae.nj(bg)),
                                           (cx.nj(dx), ae+bg),
                                           (cx.nj(dx), ae.nj(bg))] {
                            if cr < d && cq < i {
                                k[cq * d + cr] = 0xFFFFFFCC;
                            }
                        }
                    }
                }
            }
            
            let qo = (frame / avv) as usize;
            let bv = 3usize;
            let dp = i / 4;
            let jz = dp + 16 * bv + 12;
            let cnj = asd.len() * 8 * bv;
            let asa = if cnj < d { (d - cnj) / 2 } else { 0 };
            for (a, r) in asd.bw().cf() {
                if a >= qo { break; }
                aej(&mut k, d, i, asa + a * 8 * bv, dp, r, 0xFFFFFFFF, bv);
            }
            if qo > asd.len() {
                let cch = bow.len() * 8 * bv;
                let amy = if cch < d { (d - cch) / 2 } else { 0 };
                let ang = qo - asd.len();
                for (a, r) in bow.bw().cf() {
                    if a >= ang { break; }
                    aej(&mut k, d, i, amy + a * 8 * bv, jz, r, 0xFF00FF88, bv);
                }
            }
            mb(&k, d, i);
            crate::cpu::tsc::rd(aps);
        }
    }
    apq(&mut k, d, i, &mb);

    
    {
        let cm: [(&str, &str, u32, u32); 4] = [
            ("", "lines of code", 120_000, 0xFF00FF88),
            ("", "author", 1, 0xFFFFFFFF),
            ("", "secrets", 0, 0xFFFFFFFF),
            ("100%", "Rust.  0% C.", 0, 0xFFFF7744),
        ];
        let agc = 140u32;
        for frame in 0..agc {
            if let Some(eh) = crate::keyboard::xw() {
                if eh == 0x1B { break; }
                if eh == b' ' || eh == b'\r' || eh == b'\n' { break; }
            }
            gbb(&mut k, d, i, frame);
            
            np(&mut k, d, i, 40, "TrustOS", 0xFF00FF88, 6);
            
            let li = if frame < 20 { 0u32 }
                else { ((frame - 20) * 100 / 80).v(100) };
            let ufb = i / 2 - 40;
            for (si, &(adx, cif, cd, s)) in cm.iter().cf() {
                let c = ufb + si * 48;
                let bv = 2usize;
                if cd > 0 {
                    let cv = (cd as u64 * li as u64 / 100) as u32;
                    let ajh = if cv >= 1000 {
                        alloc::format!("{},{:03}", cv / 1000, cv % 1000)
                    } else {
                        alloc::format!("{}", cv)
                    };
                    let auh = alloc::format!("{} {}", ajh, cif);
                    np(&mut k, d, i, c, &auh, s, bv);
                } else if !adx.is_empty() {
                    let auh = alloc::format!("{} {}", adx, cif);
                    if frame > 60 + si as u32 * 15 {
                        np(&mut k, d, i, c, &auh, s, bv);
                    }
                } else {
                    let auh = alloc::format!("0 {}", cif);
                    if frame > 60 + si as u32 * 15 {
                        np(&mut k, d, i, c, &auh, s, bv);
                    }
                }
            }
            mb(&k, d, i);
            crate::cpu::tsc::rd(aps);
        }
    }
    apq(&mut k, d, i, &mb);

    
    {
        let features: [(&str, &str, u32); 8] = [
            ("Network",   "TCP/IP, DNS, HTTP, DHCP",       0xFF4488FF),
            ("Security",  "TLS 1.3, Ed25519, SHA-256",     0xFFFF4444),
            ("Language",  "TrustLang: full compiler",      0xFF44FF88),
            ("GUI",       "Windows, themes, animations",   0xFFFFAA22),
            ("Storage",   "FAT32, persistence, AHCI",      0xFF8888FF),
            ("Browser",   "HTTP + HTML in the kernel",     0xFF44DDFF),
            ("Games",     "Snake, Tetris, Pong, Chess",    0xFFFF88FF),
            ("Video",     "Built-in video codec",          0xFFFFDD44),
        ];
        let ec = 2usize;
        let acc = (d.ao(120)) / ec;
        let aqw = 70usize;
        let gip = 100usize;
        let bqw = 40usize;

        for lzy in 0..features.len() {
            for frame in 0..30u32 {
                if let Some(eh) = crate::keyboard::xw() {
                    if eh == 0x1B { break; }
                }
                mzc(&mut k, d, i, frame + lzy as u32 * 30);
                np(&mut k, d, i, 30,
                    "All of this in 10 MB:", 0xFFFFFFFF, 3);
                for (cqk, &(j, desc, s)) in features.iter().cf() {
                    if cqk > lzy { break; }
                    let bj = cqk % ec;
                    let br = cqk / ec;
                    let jf = bqw + bj * (acc + 40);
                    let sc = gip + br * (aqw + 20);

                    
                    let tq = if cqk == lzy && frame < 15 {
                        (15 - frame) * 12
                    } else { 0 };
                    let tq = tq as u32;

                    
                    lx(&mut k, d, i, jf, sc, acc, aqw, 0xFF0E0E1E);

                    
                    if tq > 0 {
                        let drc = 0xFF000000 | (tq.v(255) << 16) | (tq.v(255) << 8) | tq.v(255);
                        lx(&mut k, d, i, jf.ao(2), sc.ao(2),
                            acc + 4, 3, drc);
                        lx(&mut k, d, i, jf.ao(2), sc + aqw,
                            acc + 4, 3, drc);
                        lx(&mut k, d, i, jf.ao(2), sc,
                            3, aqw, drc);
                        lx(&mut k, d, i, jf + acc, sc,
                            3, aqw, drc);
                    }

                    
                    lx(&mut k, d, i, jf, sc, acc, 3, s);
                    lx(&mut k, d, i, jf, sc + aqw - 1, acc, 1, 0xFF222244);
                    ri(&mut k, d, i, jf + 10, sc + 12,
                        j, s, 2);
                    ri(&mut k, d, i, jf + 10, sc + 42,
                        desc, 0xFFAAAAAA, 1);
                }
                mb(&k, d, i);
                crate::cpu::tsc::rd(aps);
            }
        }
        crate::cpu::tsc::rd(1500);
    }
    apq(&mut k, d, i, &mb);

    
    
    
    
    aol(&mut k);
    dvz(&mut k, d, i, &mut ws, &yg,
        &[("ACT IV", 0xFF00FF88, 5),
          ("", 0xFF000000, 1),
          ("The Proof", 0xFF00CC66, 3)],
        50, 30, 6);
    apq(&mut k, d, i, &mb);

    aol(&mut k);
    dvz(&mut k, d, i, &mut ws, &yg,
        &[("When you visit a website,", 0xFFCCCCCC, 2),
          ("this is what happens", 0xFFCCCCCC, 2),
          ("inside TrustOS:", 0xFF00FF88, 2)],
        50, 60, 6);
    apq(&mut k, d, i, &mb);

    
    {
        let blh: [(&str, u32); 5] = [
            ("App",     0xFF4488FF),
            ("TLS 1.3", 0xFFFF4444),
            ("TCP/IP",  0xFFFFAA22),
            ("Driver",  0xFF44FF88),
            ("Wire",    0xFF8888FF),
        ];
        let bo = blh.len();
        let bas = (d.ao(80)) / (bo + 1);
        let gta = 60usize;
        let fmo = i / 2 - gta / 2;

        for afu in 0..2u32 {
            let cu = if afu == 0 { "Sending packet..." }
                        else         { "Response received!" };
            let vil = if afu == 0 { 0xFF00FF88 } else { 0xFF44DDFF };

            for frame in 0..150u32 {
                if let Some(eh) = crate::keyboard::xw() {
                    if eh == 0x1B { break; }
                }
                gaw(&mut k, d, i, frame + afu * 150);

                np(&mut k, d, i, 30,
                    cu, 0xFFFFFFFF, 2);

                for (si, &(j, s)) in blh.iter().cf() {
                    let cr = 40 + si * bas;
                    let nm = bas.ao(15);
                    lx(&mut k, d, i, cr, fmo, nm, gta,
                              0xFF0E1020);
                    lx(&mut k, d, i, cr, fmo, nm, 3, s);
                    lx(&mut k, d, i, cr, fmo + gta - 1, nm, 1, 0xFF222244);
                    let gx = cr + nm / 2 - j.len() * 4;
                    ri(&mut k, d, i, gx, fmo + 22,
                        j, s, 1);
                    if si < bo - 1 {
                        let ax = cr + nm;
                        lx(&mut k, d, i, ax,
                            fmo + gta / 2 - 1, 15, 3, 0xFF334455);
                        
                        lx(&mut k, d, i, ax + 12,
                            fmo + gta / 2 - 3, 3, 7, 0xFF556677);
                    }
                }

                
                let li = (frame * 100 / 120).v(100) as usize;
                let iep = (bo - 1) * bas;
                let vim = if afu == 0 {
                    iep * li / 100
                } else {
                    iep - iep * li / 100
                };
                let jjj = 40 + vim + bas / 2 - 8;
                let lud = fmo + gta + 18;
                
                for ase in 1..6u32 {
                    let gx = if afu == 0 { jjj.ao(ase as usize * 6) }
                             else { jjj + ase as usize * 6 };
                    let dw = (60u32.ao(ase * 12)).v(255);
                    let asb = 0xFF000000 | (dw / 4 << 16) | (dw << 8) | (dw / 3);
                    lx(&mut k, d, i, gx, lud + 2, 8, 12, asb);
                }
                lx(&mut k, d, i, jjj, lud, 16, 16, vil);
                ri(&mut k, d, i,
                    jjj.ao(16), lud + 20,
                    "packet", 0xFFCCCCCC, 1);

                mb(&k, d, i);
                crate::cpu::tsc::rd(aps);
            }
        }
    }
    apq(&mut k, d, i, &mb);

    
    aol(&mut k);
    dvz(&mut k, d, i, &mut ws, &yg,
        &[("Every step is visible.", 0xFFFFFFFF, 3),
          ("Every byte is readable.", 0xFFFFFFFF, 3),
          ("", 0xFF000000, 1),
          ("Nothing is hidden.", 0xFF00FF88, 4)],
        50, 80, 4);
    apq(&mut k, d, i, &mb);

    
    
    
    aol(&mut k);
    dvz(&mut k, d, i, &mut ws, &yg,
        &[("ACT V", 0xFFFFDD88, 5),
          ("", 0xFF000000, 1),
          ("The Future", 0xFFFFAA44, 3)],
        50, 30, 7);
    apq(&mut k, d, i, &mb);

    aol(&mut k);
    dvz(&mut k, d, i, &mut ws, &yg,
        &[("You deserve to understand", 0xFFFFFFFF, 3),
          ("your own machine.", 0xFFFFFFFF, 3)],
        50, 60, 7);
    apq(&mut k, d, i, &mb);

    
    {
        let avv = 2u32;
        let asd = "Computing is not magic.";
        let bow = "It's math and logic.";
        let aqo = asd.len() + bow.len();
        let agc = aqo as u32 * avv + 80;
        for frame in 0..agc {
            if let Some(eh) = crate::keyboard::xw() {
                if eh == 0x1B { break; }
                if eh == b' ' || eh == b'\r' || eh == b'\n' { break; }
            }
            hal(&mut k, d, i, frame);
            
            let wqn = (frame * 2).v(200) as usize;
            for si in 0..wqn {
                let dv = si.hx(2654435761).cn(frame as usize * 131);
                let cr = dv % d;
                let cq = (dv / d) % i;
                
                let roc = frame > 60;
                let (jf, sc) = if roc {
                    
                    let cx = d / 2;
                    let ae = i / 2;
                    let wml = (si % 4);
                    match wml {
                        0 => {  
                            let mj = cx.ao(100) + (si * 3) % 200;
                            (mj, ae.ao(60))
                        }
                        1 => {  
                            let mj = cx.ao(100) + (si * 7) % 200;
                            (mj, ae + 60)
                        }
                        2 => {  
                            let ct = ae.ao(60) + (si * 5) % 120;
                            (cx.ao(100), ct)
                        }
                        _ => {  
                            let ct = ae.ao(60) + (si * 11) % 120;
                            (cx + 100, ct)
                        }
                    }
                } else {
                    (cr, cq)
                };
                if jf < d && sc < i {
                    let aaj = (100 + (dv % 155)) as u32;
                    k[sc * d + jf] = 0xFF000000 | (aaj << 16) | (aaj << 8) | aaj;
                    if jf + 1 < d { k[sc * d + jf + 1] = 0xFF000000 | (aaj << 16) | ((aaj / 2) << 8); }
                }
            }
            
            let qo = (frame / avv) as usize;
            let bv = 3usize;
            let dp = i / 4;
            let jz = dp + 16 * bv + 16;
            let cnj = asd.len() * 8 * bv;
            let asa = if cnj < d { (d - cnj) / 2 } else { 0 };
            for (a, r) in asd.bw().cf() {
                if a >= qo { break; }
                aej(&mut k, d, i, asa + a * 8 * bv, dp, r, 0xFFCCCCCC, bv);
            }
            if qo > asd.len() {
                let cch = bow.len() * 8 * bv;
                let amy = if cch < d { (d - cch) / 2 } else { 0 };
                let ang = qo - asd.len();
                for (a, r) in bow.bw().cf() {
                    if a >= ang { break; }
                    aej(&mut k, d, i, amy + a * 8 * bv, jz, r, 0xFFFFDD88, bv);
                }
            }
            mb(&k, d, i);
            crate::cpu::tsc::rd(aps);
        }
    }
    apq(&mut k, d, i, &mb);

    
    {
        let avv = 2u32;
        let text = "TrustOS proves it.";
        let dmw = text.len() as u32 * avv;
        let vza = 60u32;
        let agc = dmw + vza;
        for frame in 0..agc {
            if let Some(eh) = crate::keyboard::xw() {
                if eh == 0x1B { break; }
                if eh == b' ' || eh == b'\r' || eh == b'\n' { break; }
            }
            hal(&mut k, d, i, frame);
            let cx = d / 2;
            let ae = i / 2;
            
            if frame > dmw / 2 {
                let vzc = frame.ao(dmw / 2);
                let uwl = 5u32;
                for jl in 0..uwl {
                    let dy = (vzc as usize * 4).ao(jl as usize * 30);
                    if dy == 0 || dy > d { continue; }
                    let aaj = (200u32.ao(jl * 30)).v(255);
                    let oyx = 0xFF000000 | ((aaj / 3) << 16) | (aaj << 8) | ((aaj * 2 / 3).v(255));
                    
                    let oyw = dy * dy;
                    let dut = dy.ao(3);
                    let jkx = dut * dut;
                    let bpl = ae.ao(dy);
                    let dno = (ae + dy).v(i);
                    for x in bpl..dno {
                        let bg = if x >= ae { x - ae } else { ae - x };
                        let isd = bg * bg;
                        
                        if isd > oyw { continue; }
                        let sht = oyw - isd;
                        let shu = if jkx > isd { jkx - isd } else { 0 };
                        
                        let mut epm = 0usize;
                        while (epm + 1) * (epm + 1) <= sht { epm += 1; }
                        let mut hhi = 0usize;
                        while (hhi + 1) * (hhi + 1) <= shu { hhi += 1; }
                        
                        for dx in hhi..=epm {
                            let y = cx + dx;
                            if y < d { k[x * d + y] = oyx; }
                        }
                        
                        for dx in hhi..=epm {
                            let y = cx.nj(dx);
                            if y < d { k[x * d + y] = oyx; }
                        }
                    }
                }
            }
            
            let qo = (frame / avv) as usize;
            let bv = 4usize;
            let qd = text.len() * 8 * bv;
            let cr = if qd < d { (d - qd) / 2 } else { 0 };
            let ty = i / 2 - 8 * bv;
            for (a, r) in text.bw().cf() {
                if a >= qo { break; }
                aej(&mut k, d, i, cr + a * 8 * bv, ty, r, 0xFF00FF88, bv);
            }
            mb(&k, d, i);
            crate::cpu::tsc::rd(aps);
        }
    }
    apq(&mut k, d, i, &mb);

    
    
    
    aol(&mut k);
    dvz(&mut k, d, i, &mut ws, &yg,
        &[("Trust the code.", 0xFF00FF88, 5),
          ("", 0xFF000000, 1),
          ("Rust is the reason.", 0xFFFF7744, 3),
          ("", 0xFF000000, 1),
          ("github.com/nathan237/TrustOS", 0xFFCCCCCC, 2)],
        60, 150, 8);
    apq(&mut k, d, i, &mb);

    
    aol(&mut k);
    mb(&k, d, i);
    if !afk {
        crate::framebuffer::afi(false);
    }
    crate::framebuffer::clear();
    crate::serial_println!("[FILM] TrustOS Film complete");
}


pub(super) fn rju(n: &[&str]) {
    if n.is_empty() {
        crate::h!(C_, "TrustView -- Binary Analysis Viewer");
        crate::h!(B_, "Usage: trustview <file>");
        crate::h!(B_, "       tv <file>");
        crate::println!("");
        crate::println!("Opens an ELF binary in the desktop binary viewer.");
        crate::println!("Panels: Navigation | Hex | Disassembly | Info/Xrefs");
        crate::println!("");
        crate::println!("Quick analysis (terminal only):");
        crate::println!("  trustview info <file>  -- Print binary summary");
        return;
    }

    let air = n[0];

    if air == "info" {
        
        let path = n.get(1).hu().unwrap_or("");
        if path.is_empty() {
            crate::h!(A_, "Usage: trustview info <file>");
            return;
        }
        match crate::binary_analysis::mvr(path) {
            Ok(dyv) => {
                crate::h!(C_, "=== TrustView Analysis: {} ===", path);
                crate::println!("{}", dyv.awz());
                crate::println!("");
                
                crate::h!(C_, "Detected Functions ({}):", dyv.xrefs.ajb.len());
                for ke in dyv.xrefs.ajb.iter().take(20) {
                    let j = if ke.j.is_empty() {
                        alloc::format!("sub_{:X}", ke.bt)
                    } else {
                        ke.j.clone()
                    };
                    crate::println!("  0x{:08X} {} ({} insns, {} blocks)", 
                        ke.bt, j, ke.jak, ke.ikx);
                }
                if dyv.xrefs.ajb.len() > 20 {
                    crate::println!("  ... and {} more", dyv.xrefs.ajb.len() - 20);
                }
            },
            Err(aa) => crate::h!(A_, "Error: {}", aa),
        }
        return;
    }

    
    let path = air;
    use crate::desktop::Aa;
    let mut desktop = Aa.lock();
    match desktop.uyv(path) {
        Ok(ad) => {
            crate::h!(B_, "TrustView opened: {} (window #{})", path, ad);
        },
        Err(aa) => {
            crate::h!(A_, "Failed to open '{}': {}", path, aa);
        }
    }
}


pub(super) fn rhu(n: &[&str]) {
    let path = n.get(0).hu().unwrap_or("");

    if path.is_empty() || path == "help" || path == "-h" {
        crate::h!(C_, "+--------------------------------------------------------------+");
        crate::h!(C_, "|     TrustOS RISC-V Universal Translation Layer               |");
        crate::h!(C_, "|  Run x86_64/ARM64/MIPS binaries via RISC-V IR translation    |");
        crate::h!(C_, "+--------------------------------------------------------------+");
        crate::println!();
        crate::println!("Usage: rv-xlat <elf-binary>");
        crate::println!();
        crate::println!("Supports: x86_64, AArch64, RISC-V (passthrough), MIPS64");
        crate::println!("Pipeline: Source ISA → RISC-V IR → Interpreter → Syscall xlat");
        crate::println!();
        crate::println!("Examples:");
        crate::println!("  rv-xlat /alpine/bin/true    # Run x86_64 binary via RV translation");
        crate::println!("  rv-xlat /bin/hello_arm      # Run ARM64 binary via RV translation");
        crate::println!();
        crate::println!("See also: rv-disasm (show RISC-V IR without executing)");
        return;
    }

    
    let f = match crate::ramfs::fh(|fs| fs.mq(path).map(|bc| bc.ip())) {
        Ok(bc) => bc,
        Err(_) => {
            crate::h!(A_, "Cannot read file: {}", path);
            return;
        }
    };

    
    match crate::riscv_translator::kpn(&f) {
        Some(arch) => {
            crate::h!(B_, "[RV-XLAT] Detected: {} binary ({} bytes)", arch.j(), f.len());
        }
        None => {
            crate::h!(A_, "Not a valid ELF binary");
            return;
        }
    }

    
    match crate::riscv_translator::xlv(&f) {
        Ok(aj) => {
            crate::println!();
            if aj == 0 {
                crate::h!(B_, "[RV-XLAT] Process exited successfully (code 0)");
            } else {
                crate::h!(D_, "[RV-XLAT] Process exited with code {}", aj);
            }
        }
        Err(aa) => {
            crate::h!(A_, "[RV-XLAT] Error: {}", aa);
        }
    }
}


pub(super) fn rht(n: &[&str]) {
    let path = n.get(0).hu().unwrap_or("");

    if path.is_empty() || path == "help" {
        crate::println!("Usage: rv-disasm <elf-binary>");
        crate::println!("Shows the RISC-V IR translation of a binary");
        return;
    }

    let f = match crate::ramfs::fh(|fs| fs.mq(path).map(|bc| bc.ip())) {
        Ok(bc) => bc,
        Err(_) => {
            crate::h!(A_, "Cannot read file: {}", path);
            return;
        }
    };

    match crate::riscv_translator::pvz(&f) {
        Ok(an) => {
            crate::println!("{}", an);
        }
        Err(aa) => {
            crate::h!(A_, "Error: {}", aa);
        }
    }
}


pub(super) fn rjo(n: &[&str]) {
    let air = n.get(0).hu().unwrap_or("help");
    
    match air {
        "help" | "-h" | "--help" => {
            crate::h!(C_, "+--------------------------------------------------------------+");
            crate::h!(C_, "|           TrustOS Binary Transpiler                          |");
            crate::h!(C_, "|       Analyze Linux binaries ? Generate Rust code            |");
            crate::h!(C_, "+--------------------------------------------------------------+");
            crate::println!();
            crate::println!("Usage: transpile <subcommand> [file]");
            crate::println!();
            crate::println!("Subcommands:");
            crate::println!("  test             - Run demo with built-in test binary");
            crate::println!("  analyze <file>   - Analyze ELF binary and show disassembly");
            crate::println!("  disasm <file>    - Show disassembly only");
            crate::println!("  rust <file>      - Generate Rust code from binary");
            crate::println!("  strings <file>   - Extract strings from binary");
            crate::println!("  syscalls <file>  - List detected syscalls");
            crate::println!("  scan             - Scan /alpine/bin for binaries to transpile");
            crate::println!("  batch            - Transpile all simple binaries");
            crate::println!("  audit            - Full syscall audit of all Alpine binaries");
            crate::println!("  run <file>       - Execute transpiled binary natively");
            crate::println!("  execute <file>   - Same as run, with verbose output");
            crate::println!();
            crate::println!("Example:");
            crate::println!("  transpile test              # Test with demo binary");
            crate::println!("  transpile run /alpine/bin/true");
            crate::println!("  transpile analyze /alpine/bin/ls");
            crate::println!("  transpile rust /alpine/bin/pwd");
        }
        
        "test" | "demo" => {
            xly();
        }
        
        "analyze" | "a" => {
            let path = n.get(1).hu().unwrap_or("/alpine/bin/ls");
            jud(path, true, true, true);
        }
        
        "disasm" | "d" => {
            let path = n.get(1).hu().unwrap_or("/alpine/bin/ls");
            jud(path, true, false, false);
        }
        
        "rust" | "r" | "gen" => {
            let path = n.get(1).hu().unwrap_or("/alpine/bin/ls");
            jud(path, false, false, true);
        }
        
        "strings" | "s" => {
            let path = n.get(1).hu().unwrap_or("/alpine/bin/ls");
            xma(path);
        }
        
        "syscalls" | "sys" => {
            let path = n.get(1).hu().unwrap_or("/alpine/bin/ls");
            xmb(path);
        }
        
        "scan" => {
            xlz();
        }
        
        "batch" => {
            xlx();
        }
        
        "run" | "exec" => {
            let path = n.get(1).hu().unwrap_or("/alpine/bin/true");
            pwb(path, false);
        }
        
        "execute" | "x" => {
            let path = n.get(1).hu().unwrap_or("/alpine/bin/true");
            pwb(path, true);
        }

        "audit" | "stats" => {
            xlw();
        }

        _ => {
            
            if air.cj('/') || air.contains('.') {
                jud(air, true, true, true);
            } else {
                crate::h!(A_, "Unknown subcommand: {}", air);
                crate::println!("Use 'transpile help' for usage");
            }
        }
    }
}

fn jud(path: &str, wnh: bool, wno: bool, wnm: bool) {
    crate::h!(C_, "Analyzing binary: {}", path);
    crate::println!();
    
    
    let f = match crate::ramfs::fh(|fs| fs.mq(path).map(|bc| bc.ip())) {
        Ok(bc) => bc,
        Err(_) => {
            crate::h!(A_, "Cannot read file: {}", path);
            return;
        }
    };
    
    crate::println!("File size: {} bytes ({} KB)", f.len(), f.len() / 1024);
    
    
    match crate::transpiler::dob(&f) {
        Some(ln) => {
            crate::h!(B_, "ELF analysis successful!");
            crate::println!();
            crate::println!("Entry point: 0x{:x}", ln.mi);
            crate::println!("Functions: {}", ln.ajb.len());
            crate::println!("Syscalls: {:?}", ln.dck);
            crate::println!("Strings: {}", ln.pd.len());
            crate::println!();
            
            if wnh {
                if let Some(ke) = ln.ajb.fv() {
                    crate::h!(D_, "=== Disassembly ({} instructions) ===", ke.instructions.len());
                    let transpiler = crate::transpiler::Transpiler::new(ke.instructions.clone());
                    crate::println!("{}", transpiler.nxk());
                }
            }
            
            if wno && !ln.pd.is_empty() {
                crate::h!(D_, "=== Strings (first 20) ===");
                for (ag, e) in ln.pd.iter().take(20) {
                    crate::println!("0x{:06x}: \"{}\"", ag, e);
                }
                crate::println!();
            }
            
            if wnm {
                crate::h!(D_, "=== Generated Rust Code ===");
                crate::println!("{}", ln.hyh);
            }
        }
        None => {
            crate::h!(A_, "Not a valid ELF binary");
        }
    }
}

fn xma(path: &str) {
    let f = match crate::ramfs::fh(|fs| fs.mq(path).map(|bc| bc.ip())) {
        Ok(bc) => bc,
        Err(_) => {
            crate::h!(A_, "Cannot read file: {}", path);
            return;
        }
    };
    
    
    let mut pd = alloc::vec::Vec::new();
    let mut cv = alloc::string::String::new();
    let mut ay = 0usize;
    
    for (a, &o) in f.iter().cf() {
        if o >= 0x20 && o < 0x7F {
            if cv.is_empty() {
                ay = a;
            }
            cv.push(o as char);
        } else {
            if cv.len() >= 4 {
                pd.push((ay, cv.clone()));
            }
            cv.clear();
        }
    }
    
    crate::h!(C_, "Strings in {}: {} found", path, pd.len());
    crate::println!();
    
    for (ag, e) in pd.iter() {
        
        if e.bw().xx(|r| r.jbb() || r == ' ') {
            crate::println!("0x{:06x}: {}", ag, e);
        }
    }
}

fn xmb(path: &str) {
    let f = match crate::ramfs::fh(|fs| fs.mq(path).map(|bc| bc.ip())) {
        Ok(bc) => bc,
        Err(_) => {
            crate::h!(A_, "Cannot read file: {}", path);
            return;
        }
    };
    
    match crate::transpiler::dob(&f) {
        Some(ln) => {
            crate::h!(C_, "Syscalls in {}", path);
            crate::println!();
            
            for ke in &ln.ajb {
                if !ke.apd.is_empty() {
                    crate::println!("Function {} @ 0x{:x}:", ke.j, ke.re);
                    for jt in &ke.apd {
                        crate::println!("  0x{:x}: {} (#{})", jt.re, jt.j, jt.aqb);
                    }
                }
            }
            
            crate::println!();
            crate::println!("Summary: {:?}", ln.dck);
        }
        None => {
            crate::h!(A_, "Not a valid ELF binary");
        }
    }
}

fn xlz() {
    crate::h!(C_, "Scanning /alpine/bin for binaries...");
    crate::println!();
    
    let ch = match crate::ramfs::fh(|fs| fs.awb(Some("/alpine/bin"))) {
        Ok(aa) => aa,
        Err(_) => {
            crate::h!(A_, "Cannot access /alpine/bin - run 'alpine test' first");
            return;
        }
    };
    
    let mut mfx = alloc::vec::Vec::new();
    let mut hds = alloc::vec::Vec::new();
    
    for (j, _, dds) in ch {
        let path = alloc::format!("/alpine/bin/{}", j);
        
        if let Ok(f) = crate::ramfs::fh(|fs| fs.mq(&path).map(|bc| bc.ip())) {
            if let Some(ln) = crate::transpiler::dob(&f) {
                let gtx = ln.dck.len();
                let lew = ln.ajb.fv().map(|bb| bb.instructions.len()).unwrap_or(0);
                
                if gtx <= 3 && lew < 100 {
                    mfx.push((j.clone(), gtx, lew));
                } else {
                    hds.push((j.clone(), gtx, lew));
                }
            }
        }
    }
    
    crate::h!(B_, "Simple binaries ({} - easy to transpile):", mfx.len());
    for (j, jt, lev) in &mfx {
        crate::println!("  {} - {} syscalls, {} instructions", j, jt, lev);
    }
    
    crate::println!();
    crate::h!(D_, "Complex binaries ({} - need more work):", hds.len());
    for (j, jt, lev) in hds.iter().take(10) {
        crate::println!("  {} - {} syscalls, {} instructions", j, jt, lev);
    }
    if hds.len() > 10 {
        crate::println!("  ... and {} more", hds.len() - 10);
    }
}

fn xlx() {
    crate::h!(C_, "Batch transpiling simple binaries...");
    crate::println!();
    
    
    let won = ["true", "false", "pwd", "whoami", "hostname", "uname", "echo", "yes"];
    
    for j in &won {
        let path = alloc::format!("/alpine/bin/{}", j);
        
        if let Ok(f) = crate::ramfs::fh(|fs| fs.mq(&path).map(|bc| bc.ip())) {
            if let Some(ln) = crate::transpiler::dob(&f) {
                crate::h!(B_, "=== {} ===", j);
                crate::println!("Syscalls: {:?}", ln.dck);
                crate::println!();
                crate::println!("{}", ln.hyh);
                crate::println!();
            } else {
                crate::h!(D_, "{}: not found or not ELF", j);
            }
        } else {
            crate::h!(D_, "{}: not available", j);
        }
    }
}


fn xlw() {
    use alloc::collections::BTreeMap;
    
    crate::h!(C_, "+--------------------------------------------------------------+");
    crate::h!(C_, "|         TrustOS Transpiler - Alpine Syscall Audit            |");
    crate::h!(C_, "+--------------------------------------------------------------+");
    crate::println!();
    
    let ch = match crate::ramfs::fh(|fs| fs.awb(Some("/alpine/bin"))) {
        Ok(aa) => aa,
        Err(_) => {
            crate::h!(A_, "Cannot access /alpine/bin - run 'linux extract' first");
            return;
        }
    };
    
    
    let mut pra: BTreeMap<&'static str, usize> = BTreeMap::new();
    let mut jsf: BTreeMap<&'static str, u64> = BTreeMap::new();
    let mut hap = 0;
    let mut npm = 0;
    let mut ppv = 0;
    let mut pvb = 0usize;
    
    crate::println!("Scanning {} files...", ch.len());
    crate::println!();
    
    for (j, _, _) in &ch {
        let path = alloc::format!("/alpine/bin/{}", j);
        hap += 1;
        
        if let Ok(f) = crate::ramfs::fh(|fs| fs.mq(&path).map(|bc| bc.ip())) {
            if let Some(ln) = crate::transpiler::dob(&f) {
                npm += 1;
                
                
                for ke in &ln.ajb {
                    pvb += ke.instructions.len();
                    
                    for jt in &ke.apd {
                        *pra.bt(jt.j).gom(0) += 1;
                        jsf.insert(jt.j, jt.aqb);
                    }
                }
                
                
                let qgp = ln.dck.iter().xx(|jt| {
                    oh!(*jt, "exit" | "exit_group" | "write" | "read" | "open" | "close" |
                            "getcwd" | "uname" | "getpid" | "getuid" | "getgid" | "geteuid" | "getegid")
                });
                if qgp && !ln.dck.is_empty() {
                    ppv += 1;
                }
            }
        }
    }
    
    crate::h!(B_, "--- Statistics ---");
    crate::println!("Files scanned:      {}", hap);
    crate::println!("Valid ELF binaries: {}", npm);
    crate::println!("Fully supported:    {}", ppv);
    crate::println!("Total instructions: {}", pvb);
    crate::println!();
    
    
    let mut bcs: Vec<_> = pra.iter().collect();
    bcs.bxe(|q, o| o.1.cmp(q.1));
    
    crate::h!(C_, "--- Syscalls by Frequency ---");
    crate::println!("{:<20} {:>8} {:>8} {}", "Syscall", "Count", "Number", "Status");
    crate::println!("{}", "-".afd(50));
    
    for (j, az) in &bcs {
        let num = jsf.get(*j).hu().unwrap_or(0);
        let jy = crate::transpiler::pre(num);
        let status = match jy {
            "Full" => "Full",
            "Partial" => "Partial",
            "Stub" => "Stub",
            _ => "Missing",
        };
        crate::println!("{:<20} {:>8} {:>8} {}", j, az, num, status);
    }
    
    crate::println!();
    crate::h!(D_, "--- Missing Syscalls (need implementation) ---");
    let lmb: Vec<_> = bcs.iter()
        .hi(|(j, _)| {
            let num = jsf.get(*j).hu().unwrap_or(999);
            crate::transpiler::pre(num) == "None"
        })
        .collect();
    
    for (j, az) in &lmb {
        let num = jsf.get(*j).hu().unwrap_or(0);
        crate::println!("  {} (#{}) - used {} times", j, num, az);
    }
    
    if lmb.is_empty() {
        crate::h!(B_, "  All syscalls are at least partially implemented!");
    }
    
    crate::println!();
    crate::h!(C_, "--- Recommendation ---");
    crate::println!("To improve transpiler coverage, implement these syscalls in order:");
    let abv: Vec<_> = lmb.iter().take(5).collect();
    for (a, (j, az)) in abv.iter().cf() {
        crate::println!("  {}. {} (used {} times)", a + 1, j, az);
    }
}


pub(super) fn rqu() {
    
    let _ = crate::ramfs::fh(|fs| {
        let _ = fs.ut("/alpine");
        let _ = fs.ut("/alpine/bin");
        Ok::<(), ()>(())
    });
    
    fn lkf(aj: &[u8]) -> alloc::vec::Vec<u8> {
        let mut elf = alloc::vec![
            0x7fu8, 0x45, 0x4c, 0x46, 0x02, 0x01, 0x01, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x02, 0x00, 0x3e, 0x00, 0x01, 0x00, 0x00, 0x00,
            0x78, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x38, 0x00,
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        elf.bk(aj);
        while elf.len() < 256 { elf.push(0); }
        elf
    }
    
    let kdh: [(&str, &[u8]); 7] = [
        ("true", &[0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00, 0x48, 0x31, 0xff, 0x0f, 0x05]),
        ("false", &[0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00, 0x48, 0xc7, 0xc7, 0x01, 0x00, 0x00, 0x00, 0x0f, 0x05]),
        ("getpid", &[0x48, 0xc7, 0xc0, 0x27, 0x00, 0x00, 0x00, 0x0f, 0x05, 0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00, 0x48, 0x31, 0xff, 0x0f, 0x05]),
        ("getuid", &[0x48, 0xc7, 0xc0, 0x66, 0x00, 0x00, 0x00, 0x0f, 0x05, 0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00, 0x48, 0x31, 0xff, 0x0f, 0x05]),
        ("uname", &[0x48, 0xc7, 0xc0, 0x3f, 0x00, 0x00, 0x00, 0x48, 0x31, 0xff, 0x0f, 0x05, 0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00, 0x48, 0x31, 0xff, 0x0f, 0x05]),
        ("echo", &[0x48, 0xc7, 0xc0, 0x01, 0x00, 0x00, 0x00, 0x48, 0xc7, 0xc7, 0x01, 0x00, 0x00, 0x00, 0x48, 0x31, 0xf6, 0x48, 0xc7, 0xc2, 0x05, 0x00, 0x00, 0x00, 0x0f, 0x05, 0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00, 0x48, 0x31, 0xff, 0x0f, 0x05]),
        ("pwd", &[0x48, 0xc7, 0xc0, 0x4f, 0x00, 0x00, 0x00, 0x48, 0x31, 0xff, 0x48, 0x31, 0xf6, 0x0f, 0x05, 0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00, 0x48, 0x31, 0xff, 0x0f, 0x05]),
    ];
    
    let mut hei = 0;
    for (j, aj) in &kdh {
        let elf = lkf(aj);
        let path = alloc::format!("/alpine/bin/{}", j);
        if crate::ramfs::fh(|fs| { let _ = fs.touch(&path); fs.ns(&path, &elf) }).is_ok() {
            hei += 1;
        }
    }
    crate::h!(B_, "      Created {} binaries", hei);
}


pub(super) fn nhi() {
    crate::h!(C_, "+--------------------------------------------------------------+");
    crate::h!(C_, "|       Creating Test Binaries for Transpiler                  |");
    crate::h!(C_, "+--------------------------------------------------------------+");
    crate::println!();
    
    
    let _ = crate::ramfs::fh(|fs| {
        let _ = fs.ut("/alpine");
        let _ = fs.ut("/alpine/bin");
        Ok::<(), ()>(())
    });
    
    
    fn lkf(aj: &[u8]) -> alloc::vec::Vec<u8> {
        let mut elf = alloc::vec![
            
            0x7fu8, 0x45, 0x4c, 0x46,  
            0x02, 0x01, 0x01, 0x00,     
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  
            0x02, 0x00,                 
            0x3e, 0x00,                 
            0x01, 0x00, 0x00, 0x00,     
            0x78, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,  
            0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  
            0x00, 0x00, 0x00, 0x00,     
            0x40, 0x00,                 
            0x38, 0x00,                 
            0x01, 0x00,                 
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  
            
            
            0x01, 0x00, 0x00, 0x00,     
            0x05, 0x00, 0x00, 0x00,     
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  
            0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,  
            0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,  
            0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  
            0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  
            0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  
        ];
        elf.bk(aj);
        while elf.len() < 256 {
            elf.push(0);
        }
        elf
    }
    
    
    let kdh: [(&str, &[u8], &str); 7] = [
        ("true", &[
            0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00,  
            0x48, 0x31, 0xff,                          
            0x0f, 0x05,                                
        ], "exit(0)"),
        
        ("false", &[
            0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00,  
            0x48, 0xc7, 0xc7, 0x01, 0x00, 0x00, 0x00,  
            0x0f, 0x05,                                
        ], "exit(1)"),
        
        ("getpid", &[
            0x48, 0xc7, 0xc0, 0x27, 0x00, 0x00, 0x00,  
            0x0f, 0x05,
            0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00,  
            0x48, 0x31, 0xff,
            0x0f, 0x05,
        ], "getpid+exit"),
        
        ("getuid", &[
            0x48, 0xc7, 0xc0, 0x66, 0x00, 0x00, 0x00,  
            0x0f, 0x05,
            0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00,
            0x48, 0x31, 0xff,
            0x0f, 0x05,
        ], "getuid+exit"),
        
        ("uname", &[
            0x48, 0xc7, 0xc0, 0x3f, 0x00, 0x00, 0x00,  
            0x48, 0x31, 0xff,
            0x0f, 0x05,
            0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00,
            0x48, 0x31, 0xff,
            0x0f, 0x05,
        ], "uname+exit"),
        
        ("echo", &[
            0x48, 0xc7, 0xc0, 0x01, 0x00, 0x00, 0x00,  
            0x48, 0xc7, 0xc7, 0x01, 0x00, 0x00, 0x00,  
            0x48, 0x31, 0xf6,                          
            0x48, 0xc7, 0xc2, 0x05, 0x00, 0x00, 0x00,  
            0x0f, 0x05,
            0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00,
            0x48, 0x31, 0xff,
            0x0f, 0x05,
        ], "write+exit"),
        
        ("pwd", &[
            0x48, 0xc7, 0xc0, 0x4f, 0x00, 0x00, 0x00,  
            0x48, 0x31, 0xff,
            0x48, 0x31, 0xf6,
            0x0f, 0x05,
            0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00,
            0x48, 0x31, 0xff,
            0x0f, 0x05,
        ], "getcwd+exit"),
    ];
    
    let mut hei = 0;
    for (j, aj, desc) in &kdh {
        let elf = lkf(aj);
        let path = alloc::format!("/alpine/bin/{}", j);
        
        let result = crate::ramfs::fh(|fs| {
            let _ = fs.touch(&path);
            fs.ns(&path, &elf)
        });
        
        match result {
            Ok(_) => {
                crate::h!(B_, "? {} - {}", j, desc);
                hei += 1;
            }
            Err(_) => {
                crate::h!(A_, "? {} - failed", j);
            }
        }
    }
    
    crate::println!();
    crate::h!(B_, "Created {} test binaries in /alpine/bin", hei);
    crate::println!();
    crate::println!("Now run:");
    crate::println!("  transpile audit       - Analyze all syscalls");
    crate::println!("  transpile run /alpine/bin/true");
    crate::println!("  transpile analyze /alpine/bin/echo");
}


fn xly() {
    crate::h!(G_, "+--------------------------------------------------------------+");
    crate::h!(G_, "|         TrustOS Transpiler Demo - Built-in Test              |");
    crate::h!(G_, "+--------------------------------------------------------------+");
    crate::println!();
    
    
    
    
    
    
    
    crate::h!(C_, "Creating test binary: exit(0) program");
    crate::println!();
    
    
    
    #[rustfmt::chz]
    let kpa: &[u8] = &[
        
        0x7F, b'E', b'L', b'F',  
        0x02,                     
        0x01,                     
        0x01,                     
        0x00,                     
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  
        0x02, 0x00,               
        0x3E, 0x00,               
        0x01, 0x00, 0x00, 0x00,   
        0x78, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,  
        0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  
        0x00, 0x00, 0x00, 0x00,   
        0x40, 0x00,               
        0x38, 0x00,               
        0x01, 0x00,               
        0x00, 0x00,               
        0x00, 0x00,               
        0x00, 0x00,               
        
        
        0x01, 0x00, 0x00, 0x00,   
        0x05, 0x00, 0x00, 0x00,   
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  
        0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,  
        0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,  
        0x84, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  
        0x84, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  
        0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  
        
        
        
        
        
        
        0x48, 0xC7, 0xC0, 0x3C, 0x00, 0x00, 0x00,  
        0x48, 0x31, 0xFF,                          
        0x0F, 0x05,                                
    ];
    
    crate::println!("Demo binary: {} bytes", kpa.len());
    crate::println!("Code: mov rax, 60; xor rdi, rdi; syscall");
    crate::println!();
    
    
    match crate::transpiler::dob(&kpa) {
        Some(ln) => {
            crate::h!(B_, "? ELF Analysis Successful!");
            crate::println!();
            
            crate::h!(D_, "--- Binary Info ---");
            crate::println!("Entry point:  0x{:x}", ln.mi);
            crate::println!("Functions:    {}", ln.ajb.len());
            crate::println!("Syscalls:     {:?}", ln.dck);
            crate::println!();
            
            
            if let Some(ke) = ln.ajb.fv() {
                crate::h!(D_, "--- Disassembly ({} instructions) ---", ke.instructions.len());
                let transpiler = crate::transpiler::Transpiler::new(ke.instructions.clone());
                crate::println!("{}", transpiler.nxk());
            }
            
            
            crate::h!(D_, "--- Generated Rust Code ---");
            crate::println!("{}", ln.hyh);
            
            crate::h!(G_, "");
            crate::h!(G_, "? Transpiler test PASSED!");
            crate::println!();
            crate::println!("The transpiler successfully:");
            crate::println!("  1. Parsed ELF64 header");
            crate::println!("  2. Found executable segment");
            crate::println!("  3. Disassembled x86_64 code");
            crate::println!("  4. Detected syscall (sys_exit)");
            crate::println!("  5. Generated equivalent Rust code");
        }
        None => {
            crate::h!(A_, "? Failed to analyze demo binary");
        }
    }
    
    
    crate::println!();
    crate::h!(C_, "Saving demo binary to /tmp/demo_exit...");
    let hyn = crate::ramfs::fh(|fs| {
        let _ = fs.ut("/tmp");
        let _ = fs.touch("/tmp/demo_exit"); 
        fs.ns("/tmp/demo_exit", kpa)
    });
    match hyn {
        Ok(_) => {
            crate::h!(B_, "? Saved! You can now run:");
            crate::println!("  transpile analyze /tmp/demo_exit");
            crate::println!("  transpile rust /tmp/demo_exit");
        }
        Err(_) => {
            crate::h!(D_, "Could not save demo binary");
        }
    }
}


fn pwb(path: &str, igi: bool) {
    use crate::transpiler::{dob, BinaryType};
    
    crate::h!(C_, "+--------------------------------------------------------------+");
    crate::h!(C_, "|           TrustOS Transpiler - Execute Binary                |");
    crate::h!(C_, "+--------------------------------------------------------------+");
    crate::println!();
    crate::println!("Binary: {}", path);
    
    
    let f = match super::network::jli(path) {
        Some(bc) => bc,
        None => {
            crate::h!(A_, "Error: Could not read file");
            return;
        }
    };
    
    if igi {
        crate::println!("Size: {} bytes", f.len());
    }
    
    
    let ln = match dob(&f) {
        Some(q) => q,
        None => {
            crate::h!(A_, "Error: Not a valid ELF binary");
            return;
        }
    };
    
    if igi {
        crate::println!("Entry point: 0x{:x}", ln.mi);
        crate::println!("Syscalls: {:?}", ln.dck);
    }
    
    crate::println!();
    crate::h!(B_, "--- Executing transpiled binary ---");
    crate::println!();
    
    
    let nz = sou(&ln);
    
    crate::println!();
    crate::h!(C_, "---------------------------------------------------------------");
    crate::println!("Exit code: {}", nz);
}


fn sou(ln: &crate::transpiler::Rm) -> i32 {
    
    let apd = if let Some(ke) = ln.ajb.fv() {
        &ke.apd
    } else {
        crate::h!(A_, "No functions found in binary");
        return 1;
    };
    
    
    for syscall in apd {
        match syscall.j {
            "exit" | "exit_group" => {
                let aj = syscall.n.get(0).hu().unwrap_or(0) as i32;
                return aj;
            }
            "write" => {
                let da = syscall.n.get(0).hu().unwrap_or(1);
                if da == 1 || da == 2 {
                    
                    
                    
                    crate::print!("[write to fd {}]", da);
                }
            }
            "getcwd" => {
                crate::println!("/");
            }
            "uname" => {
                crate::println!("TrustOS trustos 1.0.0-transpiled #1 SMP x86_64");
            }
            "getpid" => {
                crate::println!("1");
            }
            "getuid" | "geteuid" => {
                crate::println!("0");
            }
            "getgid" | "getegid" => {
                crate::println!("0");
            }
            _ => {
                crate::println!("[syscall: {} not implemented]", syscall.j);
            }
        }
    }
    
    
    0
}


pub(super) fn rkg(n: &[&str]) {
    let air = n.fv().hu().unwrap_or("help");

    match air {
        "demo" => {
            let bzk = n.get(1).hu().unwrap_or("plasma");
            let tz = 30u64;
            let aps = 1000 / tz;

            crate::println!("=== TrustVideo Demo: {} ===", bzk);
            crate::println!("Rendering in real-time @ {}fps", tz);
            crate::println!("Press Q or ESC to stop");

            
            let kp = crate::framebuffer::z();
            let kl = crate::framebuffer::ac();
            let gm = kp.v(640) as u16;
            let me = kl.v(480) as u16;

            match bzk {
                "plasma" | "fire" | "matrix" | "shader" => {
                    crate::video::player::vwl(bzk, gm, me, tz as u16);
                }
                _ => {
                    crate::println!("Unknown effect: {}. Available: plasma, fire, matrix, shader", bzk);
                }
            }
        }

        "play" => {
            let it = match n.get(1) {
                Some(bb) => *bb,
                None => { crate::println!("Usage: video play <file.tv>"); return; }
            };

            let path = if it.cj('/') {
                alloc::string::String::from(it)
            } else {
                alloc::format!("/home/{}", it)
            };

            match crate::vfs::mq(&path) {
                Ok(f) => {
                    crate::println!("Playing {}...", it);
                    let mut player = crate::video::player::VideoPlayer::new();
                    match player.vit(f) {
                        Ok(fr) => crate::println!("{}", fr),
                        Err(aa) => crate::println!("Error: {}", aa),
                    }
                }
                Err(_) => crate::println!("File not found: {}", path),
            }
        }

        "info" => {
            let it = match n.get(1) {
                Some(bb) => *bb,
                None => { crate::println!("Usage: video info <file.tv>"); return; }
            };

            let path = if it.cj('/') {
                alloc::string::String::from(it)
            } else {
                alloc::format!("/home/{}", it)
            };

            match crate::vfs::mq(&path) {
                Ok(f) => {
                    if let Some(zj) = crate::video::codec::TvHeader::eca(&f) {
                        crate::println!("=== TrustVideo Info ===");
                        crate::println!("  Format:     TrustVideo v{}", zj.dk);
                        crate::println!("  Resolution: {}x{}", zj.z, zj.ac);
                        crate::println!("  FPS:        {}", zj.tz);
                        crate::println!("  Frames:     {}", zj.oo);
                        crate::println!("  Duration:   {:.1}s", zj.oo as f64 / zj.tz as f64);
                        crate::println!("  Keyframe:   every {} frames", zj.gkq);
                        crate::println!("  File size:  {} bytes ({} KB)", f.len(), f.len() / 1024);
                        let lxd = zj.z as usize * zj.ac as usize * 4 * zj.oo as usize;
                        if lxd > 0 {
                            let bkx = lxd as f64 / f.len() as f64;
                            crate::println!("  Compression: {:.1}x (raw would be {} KB)", bkx, lxd / 1024);
                        }
                    } else {
                        crate::println!("Not a valid TrustVideo file");
                    }
                }
                Err(_) => crate::println!("File not found: {}", path),
            }
        }

        _ => {
            crate::println!("TrustVideo -- Custom video codec for TrustOS");
            crate::println!("");
            crate::println!("Usage: video <command> [args]");
            crate::println!("");
            crate::println!("Commands:");
            crate::println!("  demo [effect]  Generate & play a demo animation");
            crate::println!("                 Effects: plasma, fire, matrix, shader");
            crate::println!("  play <file>    Play a .tv video file");
            crate::println!("  info <file>    Show video file info");
            crate::println!("");
            crate::println!("Controls during playback:");
            crate::println!("  Q / ESC        Stop playback");
            crate::println!("  Space          Pause / Resume");
        }
    }
}


pub(super) fn rft(n: &[&str]) {
    match n.fv().hu().unwrap_or("open") {
        "open" | "" => {
            crate::println!("\x01G[TrustLab]\x01W Opening OS Introspection Laboratory...");
            crate::println!("  6-panel real-time kernel dashboard");
            crate::println!("  Use Tab to cycle panels, arrow keys to navigate");
            crate::desktop::Aa.lock().osq();
        }
        "help" | "--help" | "-h" => {
            crate::println!("\x01B+------------------------------------------+");
            crate::println!("\x01B|  \x01WTrustLab -- OS Introspection Laboratory\x01B  |");
            crate::println!("\x01B+------------------------------------------+");
            crate::println!("");
            crate::println!("\x01YUsage:\x01W lab [command]");
            crate::println!("");
            crate::println!("\x01YCommands:\x01W");
            crate::println!("  open     Open the Lab window (default)");
            crate::println!("  help     Show this help");
            crate::println!("");
            crate::println!("\x01YPanels:\x01W");
            crate::println!("  1. \x01GHardware Status\x01W  -- CPU, memory, IRQ stats");
            crate::println!("  2. \x01YLive Kernel Trace\x01W -- Real-time event log");
            crate::println!("  3. \x01BCommand Guide\x01W    -- Searchable command reference");
            crate::println!("  4. \x01CFile System Tree\x01W -- VFS browser");
            crate::println!("  5. \x01MTrustLang Editor\x01W -- Inline code editor");
            crate::println!("  6. \x01YEvent Stream\x01W     -- Filtered event feed");
            crate::println!("");
            crate::println!("\x01YControls:\x01W");
            crate::println!("  Tab        Cycle focused panel");
            crate::println!("  Arrows     Navigate within panel");
            crate::println!("  PgUp/Down  Scroll faster");
            crate::println!("  F5         Run code (in editor panel)");
        }
        gq => {
            crate::println!("\x01RUnknown lab command: {}\x01W", gq);
            crate::println!("Type 'lab help' for usage.");
        }
    }
}


pub(super) fn rfd(n: &[&str]) {
    let an = crate::hwscan::oaf(n);
    crate::print!("{}", an);
}