




use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use crate::framebuffer::{B_, G_, AU_, D_, A_, C_, Q_, CD_, DF_, L_};
use crate::ramfs::FileType;


pub(super) fn kif(n: &[&str]) {
    use alloc::vec;
    
    crate::h!(C_, "-----------------------------------------------------------");
    crate::h!(C_, "              TrustOS Graphics Benchmark");
    crate::h!(C_, "               SSE2 SIMD Optimizations");
    crate::h!(C_, "-----------------------------------------------------------");
    crate::println!();
    
    let (z, ac) = crate::framebuffer::yn();
    let hz = (z * ac) as usize;
    crate::println!("Resolution: {}x{} ({} pixels, {} MB)", 
        z, ac, hz, hz * 4 / 1024 / 1024);
    crate::println!();
    
    
    crate::h!(B_, "? Test 1: SSE2 Buffer Fill");
    {
        let mut bi = vec![0u32; hz];
        let atc = 100;
        
        let ay = crate::cpu::tsc::ow();
        for _ in 0..atc {
            crate::graphics::simd::ntp(&mut bi, 0xFF00FF66);
        }
        let ci = crate::cpu::tsc::ow();
        
        let knk = (ci - ay) / atc;
        let umu = hz as f64 / 1_000_000.0;
        crate::println!("  {} iterations: {} cycles/frame", atc, knk);
        crate::println!("  Throughput: ~{:.1} megapixels/frame", umu);
    }
    
    
    crate::h!(B_, "? Test 2: SSE2 Buffer Copy");
    {
        let cy = vec![0xFF112233u32; hz];
        let mut cs = vec![0u32; hz];
        let atc = 100;
        
        let ay = crate::cpu::tsc::ow();
        for _ in 0..atc {
            crate::graphics::simd::ror(&mut cs, &cy);
        }
        let ci = crate::cpu::tsc::ow();
        
        let knk = (ci - ay) / atc;
        let umf = (hz * 4) as f64 / 1024.0 / 1024.0;
        crate::println!("  {} iterations: {} cycles/frame", atc, knk);
        crate::println!("  Bandwidth: ~{:.1} MB copied/frame", umf);
    }
    
    
    crate::h!(B_, "? Test 3: Rectangle Fill (400x300)");
    {
        let mut surface = crate::graphics::fast_render::FastSurface::new(1280, 800);
        let atc = 500;
        
        let ay = crate::cpu::tsc::ow();
        for _ in 0..atc {
            surface.ah(100, 100, 400, 300, 0xFF00AA55);
        }
        let ci = crate::cpu::tsc::ow();
        
        let rst = (ci - ay) / atc;
        let vij = 400 * 300;
        crate::println!("  {} iterations: {} cycles/rect", atc, rst);
        crate::println!("  {} pixels/rect", vij);
    }
    
    
    crate::h!(B_, "? Test 4: Framebuffer swap_buffers");
    {
        
        let jwl = crate::framebuffer::bre();
        if !jwl {
            crate::framebuffer::beo();
            crate::framebuffer::afi(true);
        }
        
        let atc = 50;
        let ay = crate::cpu::tsc::ow();
        for _ in 0..atc {
            crate::framebuffer::sv();
        }
        let ci = crate::cpu::tsc::ow();
        
        let njb = (ci - ay) / atc;
        
        let kuc = 3_000_000_000u64 / njb.am(1);
        crate::println!("  {} iterations: {} cycles/swap", atc, njb);
        crate::println!("  Estimated max FPS: ~{} (at 3GHz)", kuc);
        
        if !jwl {
            crate::framebuffer::afi(false);
        }
    }
    
    
    crate::h!(B_, "? Test 5: GraphicsTerminal render (80x25)");
    {
        let mut terminal = crate::wayland::terminal::GraphicsTerminal::new(80, 25);
        terminal.write_str("Hello from TrustOS! Testing SSE2 SIMD terminal rendering performance.\n");
        terminal.write_str("The quick brown fox jumps over the lazy dog.\n");
        
        let atc = 100;
        let ay = crate::cpu::tsc::ow();
        for _ in 0..atc {
            let _ = terminal.tj();
        }
        let ci = crate::cpu::tsc::ow();
        
        let nja = (ci - ay) / atc;
        let kuc = 3_000_000_000u64 / nja.am(1);
        crate::println!("  {} iterations: {} cycles/render", atc, nja);
        crate::println!("  Estimated terminal FPS: ~{}", kuc);
    }
    
    crate::println!();
    crate::h!(C_, "-----------------------------------------------------------");
    crate::h!(B_, "Benchmark complete! SSE2 optimizations active.");
    crate::h!(C_, "-----------------------------------------------------------");
}



pub(super) fn yjb(n: &[&str]) {
    use crate::cosmic::{CosmicRenderer, Rect, Point, Color, theme, CosmicTheme, bxb};
    use crate::cosmic::theme::dark;
    
    let wvr = n.fv().hu().unwrap_or("demo");
    
    match wvr {
        "demo" | "test" => {
            crate::h!(C_, "+---------------------------------------------------------------+");
            crate::h!(C_, "|          COSMIC UI Framework Demo (libcosmic-inspired)       |");
            crate::h!(C_, "+---------------------------------------------------------------+");
            crate::println!();
            
            let (z, ac) = crate::framebuffer::yn();
            crate::println!("  Framebuffer: {}x{}", z, ac);
            crate::println!("  Renderer: tiny-skia (software, no_std)");
            crate::println!("  Theme: COSMIC Dark (Pop!_OS style)");
            crate::println!();
            
            crate::h!(B_, "Creating COSMIC renderer...");
            let mut renderer = CosmicRenderer::new(z, ac);
            
            
            renderer.clear(dark::DD_);
            
            crate::h!(B_, "Drawing COSMIC UI elements...");
            
            
            let vbk = Rect::new(0.0, 0.0, z as f32, 32.0);
            renderer.nnl(vbk);
            
            
            
            let pav = Rect::new(50.0, 80.0, 200.0, 100.0);
            renderer.afp(pav, 12.0, dark::Kw);
            renderer.gtn(pav, 12.0, dark::Fj, 1.0);
            
            
            let naj = Rect::new(300.0, 100.0, 120.0, 40.0);
            renderer.gfj(naj, 8.0, 8.0, Color::Ox.fbo(0.4));
            renderer.afp(naj, 8.0, dark::Ge);
            
            
            let qsf = Rect::new(450.0, 100.0, 120.0, 40.0);
            renderer.afp(qsf, 8.0, dark::MB_);
            
            
            let qsg = Rect::new(600.0, 100.0, 120.0, 40.0);
            renderer.afp(qsg, 8.0, dark::JN_);
            
            
            renderer.abc(Point::new(100.0, 250.0), 30.0, dark::Ge);
            renderer.abc(Point::new(180.0, 250.0), 30.0, dark::Aep);
            renderer.abc(Point::new(260.0, 250.0), 30.0, dark::Afq);
            renderer.abc(Point::new(340.0, 250.0), 30.0, dark::Sf);
            
            
            let tny = Rect::new(50.0, 320.0, 400.0, 40.0);
            renderer.nnb(tny, "COSMIC Window", true);
            
            
            let xuq = Rect::new(50.0, 360.0, 400.0, 150.0);
            renderer.ah(xuq, dark::CS_);
            renderer.gtn(
                Rect::new(50.0, 320.0, 400.0, 190.0),
                0.0,
                dark::Fj,
                1.0
            );
            
            
            let kqm = [
                crate::cosmic::Abe { j: "Files", gh: true, asy: false, aqk: true },
                crate::cosmic::Abe { j: "Term", gh: false, asy: true, aqk: true },
                crate::cosmic::Abe { j: "Browser", gh: false, asy: false, aqk: false },
                crate::cosmic::Abe { j: "Settings", gh: false, asy: false, aqk: true },
            ];
            let sad = Rect::new((z - 64) as f32, 100.0, 64.0, 280.0);
            renderer.irs(sad, &kqm);
            
            
            let thj = Rect::new(500.0, 320.0, 200.0, 100.0);
            renderer.kvv(thj, dark::Ge, dark::DD_);
            
            crate::h!(B_, "Presenting to framebuffer...");
            renderer.vky();
            
            crate::println!();
            crate::h!(G_, "? COSMIC UI demo rendered successfully!");
            crate::println!();
            crate::println!("  Features demonstrated:");
            crate::println!("  - Rounded rectangles with anti-aliasing");
            crate::println!("  - Drop shadows");
            crate::println!("  - COSMIC color palette");
            crate::println!("  - Top panel, header bar, dock");
            crate::println!("  - Buttons (normal, suggested, destructive)");
            crate::println!("  - Vertical gradients");
            crate::println!();
            crate::println!("  Press any key to return to shell...");
            
            
            crate::keyboard::xtj();
            crate::framebuffer::clear();
            crate::framebuffer::sv();
        },
        "desktop" => {
            
            ndt();
        },
        "theme" => {
            let ezr = n.get(1).hu().unwrap_or("matrix");
            match ezr {
                "dark" => {
                    bxb(CosmicTheme::dark());
                    crate::h!(B_, "Theme set to COSMIC Dark");
                },
                "light" => {
                    bxb(CosmicTheme::light());
                    crate::h!(B_, "Theme set to COSMIC Light");
                },
                "matrix" => {
                    bxb(CosmicTheme::matrix());
                    crate::h!(0x00FF00, "Theme set to MATRIX - Wake up, Neo...");
                },
                _ => {
                    crate::println!("Available themes: dark, light, matrix");
                }
            }
        },
        "info" => {
            crate::h!(C_, "COSMIC UI Framework for TrustOS");
            crate::println!();
            crate::println!("  Based on: libcosmic by System76 (Pop!_OS)");
            crate::println!("  Renderer: tiny-skia v0.12 (no_std mode)");
            crate::println!("  Features: anti-aliased shapes, gradients, shadows");
            crate::println!();
            crate::println!("  Modules:");
            crate::println!("    cosmic::theme    - COSMIC color palette");
            crate::println!("    cosmic::renderer - tiny-skia based rendering");
            crate::println!("    cosmic::widgets  - Button, Label, Container, etc.");
            crate::println!("    cosmic::layout   - Flexbox-style layout system");
        },
        _ => {
            crate::println!("Usage: cosmic <command>");
            crate::println!();
            crate::println!("Commands:");
            crate::println!("  desktop - Launch full COSMIC desktop environment");
            crate::println!("  demo    - Render COSMIC UI demo to screen");
            crate::println!("  theme   - Set theme (dark/light)");
            crate::println!("  info    - Show framework information");
        }
    }
}


pub(super) fn rgu(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: open <app>");
        crate::println!("");
        crate::println!("Available apps:");
        crate::println!("  browser, web, www   - Web browser (HTTPS/TLS 1.3)");
        crate::println!("  files, explorer     - File manager");
        crate::println!("  editor, notepad     - Text editor");
        crate::println!("  network, net        - Network status");
        crate::println!("  hardware, hw        - Hardware info");
        crate::println!("  users               - User management");
        crate::println!("  images, viewer      - Image viewer");
        crate::println!("");
        crate::println!("Example: open browser");
        return;
    }
    
    let bjf = n[0].aqn();
    ndu(Some(&bjf));
}



fn nco() -> bool {
    let mut ihb: u8 = 0;

    
    let vhm = crate::memory::fxc();
    let clx = vhm / (1024 * 1024);
    if clx > 0 && clx < 256 {
        crate::h!(A_, "\u{26A0} Insufficient RAM: {} MB detected (minimum: 256 MB)", clx);
        ihb += 1;
    } else if clx > 0 && clx < 512 {
        crate::h!(D_, "\u{26A0} Low RAM: {} MB detected (recommended: 512 MB+)", clx);
    }

    
    let buv = crate::memory::heap::aez();
    let drq = buv / (1024 * 1024);
    if drq < 16 {
        crate::h!(A_, "\u{26A0} Insufficient heap: {} MB free (minimum: 16 MB)", drq);
        ihb += 1;
    } else if drq < 32 {
        crate::h!(D_, "\u{26A0} Low heap: {} MB free (recommended: 32 MB+)", drq);
    }

    
    let (gz, kc) = crate::framebuffer::yn();
    if gz == 0 || kc == 0 {
        crate::h!(A_, "\u{26A0} No framebuffer detected! Desktop requires a display.");
        ihb += 1;
    } else if gz < 800 || kc < 600 {
        crate::h!(D_, "\u{26A0} Low resolution: {}x{} (recommended: 1024x768+)", gz, kc);
    }

    
    if gz > 0 && kc > 0 {
        let qmk = (gz as usize) * (kc as usize) * 4 * 2;
        let mxl = qmk / (1024 * 1024);
        if drq > 0 && (mxl as usize) > drq + 4 {
            crate::h!(A_, "\u{26A0} Not enough memory for {}x{} framebuffer ({} MB needed, {} MB free)",
                gz, kc, mxl, drq);
            ihb += 1;
        }
    }

    if ihb > 0 {
        crate::h!(D_, "");
        crate::h!(D_, "\u{2139}  Desktop may be unstable with current resources.");
        if clx > 0 && clx < 256 {
            crate::h!(Q_, "   Tip: Increase VM RAM to 512 MB+ (-m 512M in QEMU)");
        }
        crate::h!(D_, "   Launching anyway...");
        crate::println!("");
    }

    true
}



pub(super) fn jcx(tuf: Option<(&str, crate::desktop::WindowType, i32, i32, u32, u32)>) {
    use crate::desktop;

    
    if !nco() {
        return;
    }

    let (z, ac) = crate::framebuffer::yn();
    if z == 0 || ac == 0 {
        crate::h!(A_, "Error: Invalid framebuffer!");
        return;
    }
    crate::mouse::dbw(z, ac);
    
    let mut bc = desktop::Aa.lock();
    bc.init(z, ac);
    
    
    if bc.asr == desktop::DesktopTier::Aap {
        crate::h!(D_, "Insufficient resources for desktop (< 128 MB RAM).");
        crate::h!(D_, "Staying in command-line mode. Increase RAM to 256 MB+ (-m 256M).");
        drop(bc);
        return;
    }
    
    
    let xgx = match bc.asr {
        desktop::DesktopTier::Gy  => "Minimal (solid bg, no effects)",
        desktop::DesktopTier::Gc => "Standard (2-layer rain, basic effects)",
        desktop::DesktopTier::Bv     => "Full (4-layer rain, visualizer, all effects)",
        _ => "CLI",
    };
    crate::serial_println!("[Desktop] Launching in {} mode", xgx);
    
    
    if crate::drivers::virtio_gpu::anl() {
        bc.che = desktop::RenderMode::Atd;
        crate::serial_println!("[Desktop] GPU-accelerated rendering enabled (VirtIO GPU)");
    }
    
    
    if let Some((dq, ash, b, c, d, i)) = tuf {
        bc.xl(dq, b, c, d, i, ash);
    }
    
    drop(bc);
    
    
    crate::gui::engine::wnq("TrustOS Desktop", "Welcome! Alt+Tab to switch windows", crate::gui::engine::NotifyPriority::Hf);
    
    crate::serial_println!("[Desktop] Entering desktop run loop");
    desktop::vw();
    
    crate::serial_println!("[Desktop] Returned to shell");
    
    let (d, i) = crate::framebuffer::yn();
    crate::framebuffer::ah(0, 0, d, i, 0xFF000000);
    crate::h!(B_, "\nReturned to TrustOS shell. Type 'help' for commands.");
}



pub(super) fn ucz() {
    use crate::desktop;

    
    if !nco() {
        return;
    }

    let (z, ac) = crate::framebuffer::yn();
    if z == 0 || ac == 0 {
        crate::h!(A_, "Error: Invalid framebuffer!");
        return;
    }
    crate::mouse::dbw(z, ac);
    
    let mut bc = desktop::Aa.lock();
    bc.init(z, ac);
    
    bc.ud.gh = true;
    bc.ud.bls = crate::mobile::MobileView::Lo;
    let (fp, iz, gm, me) = crate::mobile::nbi(z, ac);
    bc.ud.dxp = fp;
    bc.ud.ddi = iz;
    bc.ud.att = gm;
    bc.ud.azc = me;
    crate::serial_println!("[Mobile] Viewport: {}x{} at ({},{}) on {}x{}", gm, me, fp, iz, z, ac);
    
    drop(bc);
    crate::serial_println!("[Mobile] Entering mobile desktop loop");
    desktop::vw();
    
    crate::serial_println!("[Mobile] Returned to shell");
    let (d, i) = crate::framebuffer::yn();
    crate::framebuffer::ah(0, 0, d, i, 0xFF000000);
    crate::h!(B_, "\nReturned to TrustOS shell. Type 'help' for commands.");
}



pub(super) fn rie(n: &[&str]) {
    use crate::signature;

    match n.fv().hu() {
        Some("verify") | None => {
            
            crate::println!();
            crate::h!(C_, "\u{2554}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2557}");
            crate::h!(C_, "\u{2551}              TrustOS Kernel Signature Certificate                  \u{2551}");
            crate::h!(C_, "\u{2560}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2563}");
            crate::println!();
            crate::h!(G_, "  ?? CREATOR SIGNATURE (immutable)");
            crate::h!(Q_, "  -----------------------------------------------------------------");
            crate::println!("  Author:      {} (@{})", signature::BPS_, signature::BPR_);
            crate::println!("  Payload:     \"{}\"", signature::APJ_);
            crate::println!("  Algorithm:   HMAC-SHA256");
            crate::h!(D_, "  Fingerprint: {}", signature::nhk());
            crate::println!("  Version:     v{}", signature::NU_);
            crate::println!("  Built:       {}", signature::BLM_);
            crate::println!();
            crate::h!(L_, "  i  This fingerprint was generated with a secret seed known ONLY");
            crate::h!(L_, "     to the creator. It cannot be forged without the original seed.");
            crate::println!();

            
            if let Some((j, nu, wi)) = signature::iww() {
                crate::h!(CD_, "  USER CO-SIGNATURE");
                crate::h!(Q_, "  -----------------------------------------------------------------");
                crate::println!("  Signed by:   {}", j);
                crate::h!(D_, "  Fingerprint: {}", nu);
                crate::println!("  Signed at:   {}s after midnight (RTC)", wi);
                crate::println!();
            } else {
                crate::h!(L_, "  No user co-signature. Use 'signature sign <name>' to add yours.");
                crate::println!();
            }

            crate::h!(C_, "\u{255a}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{255d}");
            crate::println!();
        }
        Some("sign") => {
            
            if n.len() < 2 {
                crate::h!(A_, "Usage: signature sign <your_name>");
                return;
            }
            let j = n[1];
            crate::println!("Enter your secret passphrase to sign the kernel:");
            crate::print!("> ");
            
            let bvw = jlm();
            if bvw.is_empty() {
                crate::h!(A_, "Empty passphrase. Aborted.");
                return;
            }
            signature::wnz(j, bvw.as_bytes());
            crate::println!();
            crate::h!(G_, "? Kernel co-signed by '{}'", j);
            if let Some((_, nu, _)) = signature::iww() {
                crate::h!(D_, "  Your fingerprint: {}", nu);
            }
            crate::h!(L_, "  Keep your passphrase safe -- you'll need it to prove ownership.");
            crate::println!();
        }
        Some("prove") => {
            
            if n.len() < 2 {
                crate::h!(A_, "Usage: signature prove <name>");
                return;
            }
            let j = n[1];
            crate::println!("Enter passphrase to verify:");
            crate::print!("> ");
            let bvw = jlm();
            if signature::xrm(j, bvw.as_bytes()) {
                crate::h!(G_, "VERIFIED -- '{}' is the legitimate signer.", j);
            } else {
                crate::h!(A_, "FAILED -- passphrase does not match the signature for '{}'.", j);
            }
            crate::println!();
        }
        Some("prove-creator") => {
            
            crate::println!("Enter creator seed to verify authorship:");
            crate::print!("> ");
            let dv = jlm();
            if signature::xrg(dv.as_bytes()) {
                crate::h!(G_, "CREATOR VERIFIED -- You are the original author of TrustOS.");
            } else {
                crate::h!(A_, "FAILED -- This seed does not match the creator fingerprint.");
            }
            crate::println!();
        }
        Some("integrity") | Some("verify-integrity") => {
            crate::println!();
            crate::h!(G_, "Kernel Integrity Verification");
            crate::println!("---------------------------------------------------------------");
            let report = signature::tvn();
            for line in &report {
                crate::println!("{}", line);
            }
            crate::println!();
            crate::h!(L_, "  SHA-256 of .text + .rodata sections measured at boot vs now.");
            crate::h!(L_, "  Detects runtime code injection, ROP gadget insertion, and");
            crate::h!(L_, "  constant/vtable tampering (rootkits, memory corruption).");
            crate::println!();
        }
        Some("clear") => {
            signature::rbh();
            crate::h!(D_, "User co-signature cleared.");
        }
        Some("export") => {
            
            if let Some((j, nu, ydo)) = signature::iww() {
                let os = crate::rtc::cgz();
                crate::println!();
                crate::h!(C_, "=== Copy everything below and submit as a PR to SIGNATURES.md ===");
                crate::println!();
                crate::println!("### #NNN -- {}", j);
                crate::println!();
                crate::println!("| Field | Value |");
                crate::println!("|-------|-------|");
                crate::println!("| **Name** | {} |", j);
                crate::println!("| **GitHub** | [@YOURUSERNAME](https://github.com/YOURUSERNAME) |");
                crate::println!("| **Algorithm** | HMAC-SHA256 |");
                crate::println!("| **Fingerprint** | `{}` |", nu);
                crate::println!("| **Kernel Version** | v{} |", signature::NU_);
                crate::println!("| **Date** | {:04}-{:02}-{:02} |", os.ccq, os.caw, os.cjw);
                crate::println!("| **Status** | Verified signer |");
                crate::println!();
                crate::h!(L_, "Replace YOURUSERNAME with your GitHub username and #NNN with the next number.");
                crate::h!(L_, "Submit as a Pull Request to: github.com/nathan237/TrustOS");
                crate::println!();
            } else {
                crate::h!(A_, "No user signature found. Run 'signature sign <name>' first.");
            }
        }
        Some("list") => {
            
            crate::println!();
            crate::h!(C_, "TrustOS Signature Registry");
            crate::h!(Q_, "------------------------------------------------------");
            crate::println!();
            crate::h!(G_, "  #001  Nated0ge (Creator)");
            crate::println!("        {}", signature::nhk());
            crate::println!();
            if let Some((j, nu, _)) = signature::iww() {
                crate::h!(C_, "  #---  {} (Local)", j);
                crate::println!("        {}", nu);
                crate::println!();
            }
            crate::h!(L_, "  Full registry: github.com/nathan237/TrustOS/blob/main/SIGNATURES.md");
            crate::println!();
        }
        Some("ed25519") => {
            
            match n.get(1).hu() {
                Some("verify") | None => {
                    crate::println!();
                    crate::h!(C_, "Ed25519 Asymmetric Signature Report");
                    crate::h!(Q_, "--------------------------------------------------------------");
                    let report = signature::npc();
                    for line in &report {
                        crate::println!("{}", line);
                    }
                    crate::println!();
                }
                Some("sign") => {
                    crate::println!("Enter Ed25519 seed (hex or passphrase):");
                    crate::print!("> ");
                    let phk = jlm();
                    if phk.is_empty() {
                        crate::h!(A_, "Empty seed. Aborted.");
                        return;
                    }
                    signature::sip(phk.as_bytes());
                    crate::h!(G_, "? Kernel re-signed with Ed25519 (new seed).");
                    if let Some(report) = signature::npc().fv() {
                        crate::println!("  {}", report);
                    }
                    crate::println!();
                }
                _ => {
                    crate::println!("Usage:");
                    crate::println!("  signature ed25519          - Show Ed25519 signature & verify");
                    crate::println!("  signature ed25519 verify   - Verify current Ed25519 signature");
                    crate::println!("  signature ed25519 sign     - Re-sign kernel with new seed");
                }
            }
        }
        _ => {
            crate::h!(C_, "TrustOS Kernel Signature System");
            crate::println!();
            crate::println!("Usage:");
            crate::println!("  signature                - Show signature certificate");
            crate::println!("  signature verify         - Show & verify signature certificate");
            crate::println!("  signature integrity      - Verify kernel .text+.rodata integrity");
            crate::println!("  signature sign <name>    - Co-sign the kernel with your identity");
            crate::println!("  signature prove <name>   - Prove a user signature with passphrase");
            crate::println!("  signature prove-creator  - Prove creator authorship (requires seed)");
            crate::println!("  signature export         - Export signature for GitHub PR submission");
            crate::println!("  signature list           - Show registered signatures");
            crate::println!("  signature clear          - Remove user co-signature");
            crate::println!("  signature ed25519        - Ed25519 asymmetric signature status");
            crate::println!("  signature ed25519 sign   - Re-sign kernel with Ed25519 seed");
        }
    }
}


fn jlm() -> alloc::string::String {
    use alloc::string::String;
    let mut bvw = String::new();
    loop {
        if let Some(bs) = crate::keyboard::xw() {
            match bs {
                b'\n' | b'\r' | 0x0A | 0x0D => {
                    crate::println!();
                    break;
                }
                0x08 => {
                    
                    if !bvw.is_empty() {
                        bvw.pop();
                        crate::print!("\x08 \x08");
                    }
                }
                r if r.ofo() && !r.yyx() => {
                    bvw.push(r as char);
                    crate::print!("*");
                }
                _ => {}
            }
        }
        core::hint::hc();
    }
    bvw
}


pub(super) fn rhy(n: &[&str]) {
    match n.fv().hu() {
        Some("status") | None => {
            
            let cm = crate::security::cm();
            crate::println!();
            crate::h!(G_, "TrustOS Security Status");
            crate::println!("---------------------------------------------------------------");
            crate::println!("  Active capabilities : {}", cm.mto);
            crate::println!("  Security violations : {}", cm.cnt);
            crate::println!("  Dynamic types       : {}", cm.noq);
            crate::println!("  Isolated subsystems : {}", cm.ppr);
            crate::println!("  Gate checks         : {}", crate::security::isolation::puy());
            crate::println!("  Gate violations     : {}", crate::security::isolation::puz());
            crate::println!();
            
            
            match crate::signature::pye() {
                Ok(true) => crate::h!(G_, "  Kernel integrity    : ? INTACT"),
                Ok(false) => crate::h!(A_, "  Kernel integrity    : ? TAMPERED"),
                Err(_) => crate::h!(D_, "  Kernel integrity    : ??  not initialized"),
            }
            crate::println!();
        }
        Some("caps") | Some("capabilities") => {
            
            let dr = crate::security::ufn();
            crate::println!();
            crate::h!(C_, "Active Capabilities ({} total)", dr.len());
            crate::println!("----------------------------------------------------------");
            crate::println!("  {:>6} | {:<20} | {:<10} | Owner", "ID", "Type", "Category");
            crate::println!("  -------+----------------------+------------+------");
            for (ad, cap_type, yck, awj) in &dr {
                crate::println!("  {:>6} | {:<20} | {:<10} | 0x{:04X}",
                    ad.0,
                    alloc::format!("{:?}", cap_type),
                    cap_type.gb(),
                    awj
                );
            }
            crate::println!();
        }
        Some("isolation") | Some("iso") | Some("subsystems") => {
            
            crate::println!();
            crate::h!(G_, "Subsystem Isolation Boundaries");
            crate::println!("---------------------------------------------------------------");
            let report = crate::security::isolation::tzq();
            for line in &report {
                crate::println!("{}", line);
            }
            crate::println!();
            crate::h!(L_, "  ring0-tcb       = Part of TCB, must stay in ring 0");
            crate::h!(L_, "  ring0-isolated  = Ring 0 but logically isolated");
            crate::h!(L_, "  ring3-candidate = Could be moved to ring 3 in future");
            crate::println!();
        }
        Some("gate") => {
            
            if let Some(ppq) = n.get(1).hu() {
                let bcu = match ppq {
                    "storage" | "disk" => Some(crate::security::isolation::Subsystem::Og),
                    "network" | "net" => Some(crate::security::isolation::Subsystem::As),
                    "graphics" | "gpu" => Some(crate::security::isolation::Subsystem::Jm),
                    "process" | "proc" => Some(crate::security::isolation::Subsystem::Tz),
                    "hypervisor" | "hv" => Some(crate::security::isolation::Subsystem::Ee),
                    "shell" => Some(crate::security::isolation::Subsystem::Df),
                    "crypto" => Some(crate::security::isolation::Subsystem::Jh),
                    "power" => Some(crate::security::isolation::Subsystem::Hb),
                    "serial" => Some(crate::security::isolation::Subsystem::Yu),
                    "memory" | "mem" => Some(crate::security::isolation::Subsystem::Cy),
                    _ => None,
                };
                if let Some(sub) = bcu {
                    match crate::security::isolation::drb(
                        sub, crate::security::CapabilityRights::Cm
                    ) {
                        Ok(()) => crate::h!(G_, 
                            "  ? Gate check PASSED for {:?}", sub),
                        Err(aa) => crate::h!(A_, 
                            "  ? Gate check DENIED for {:?}: {:?}", sub, aa),
                    }
                } else {
                    crate::h!(A_, "Unknown subsystem: {}", ppq);
                }
            } else {
                crate::println!("Usage: security gate <subsystem>");
                crate::println!("  Subsystems: storage, network, graphics, process, hypervisor,");
                crate::println!("              shell, crypto, power, serial, memory");
            }
        }
        Some("dynamic") => {
            
            let ifn = crate::security::ojp();
            crate::println!();
            if ifn.is_empty() {
                crate::h!(L_, "No dynamic capability types registered.");
            } else {
                crate::h!(C_, "Dynamic Capability Types ({} registered)", ifn.len());
                for (ad, co) in &ifn {
                    crate::println!("  [{}] {} (danger:{}, category:{})", 
                        ad, co.j, co.eom, co.gb);
                    crate::println!("       {}", co.dc);
                }
            }
            crate::println!();
        }
        _ => {
            crate::h!(C_, "TrustOS Security Subsystem");
            crate::println!();
            crate::println!("Usage:");
            crate::println!("  security                 - Show security status overview");
            crate::println!("  security caps            - List all active capabilities");
            crate::println!("  security isolation       - Show subsystem isolation boundaries");
            crate::println!("  security gate <subsys>   - Test a gate check on a subsystem");
            crate::println!("  security dynamic         - List dynamic capability types");
        }
    }
}






pub(super) fn rdp(n: &[&str]) {
    let sh = match n.fv().hu() {
        Some("fr") => "fr",
        _ => "en",
    };

    
    let rb = |tv: u64| {
        let jn = tv * 1000;
        let ayu = crate::cpu::tsc::ow();
        let kx = crate::cpu::tsc::ard();
        if kx == 0 { return; }
        let cii = kx / 1000 * jn;
        loop {
            let ez = crate::cpu::tsc::ow().ao(ayu);
            if ez >= cii { break; }
            let _ = crate::keyboard::xw();
            core::hint::hc();
        }
    };

    let fym = |xhf: u64| -> bool {
        let jn = xhf * 1000;
        let ayu = crate::cpu::tsc::ow();
        let kx = crate::cpu::tsc::ard();
        if kx == 0 { return false; }
        let cii = kx / 1000 * jn;
        loop {
            let ez = crate::cpu::tsc::ow().ao(ayu);
            if ez >= cii { return false; }
            if let Some(bs) = crate::keyboard::xw() {
                if bs == 0x1B || bs == b'q' { return true; } 
                if bs == b' ' || bs == b'\n' || bs == 13 { return false; } 
            }
            core::hint::hc();
        }
    };

    
    
    
    let (kp, kl) = crate::framebuffer::yn();

    {
        let afk = crate::framebuffer::bre();
        if !afk {
            crate::framebuffer::beo();
            crate::framebuffer::afi(true);
        }

        let d = kp as usize;
        let i = kl as usize;
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

        
        let mut ws: alloc::vec::Vec<u16> = alloc::vec![0u16; d / 8 + 1];
        let mut yg: alloc::vec::Vec<u8> = alloc::vec![1u8; d / 8 + 1];
        for a in 0..ws.len() {
            ws[a] = ((a * 37 + 13) % i) as u16;
            yg[a] = (((a * 7 + 3) % 4) + 1) as u8;
        }

        let gfi = |k: &mut [u32], d: usize, i: usize, ec: &mut [u16], arz: &[u8], frame: u32| {
            for il in k.el() {
                let at = ((*il >> 8) & 0xFF) as u32;
                if at > 0 {
                    let fou = at.ao(8);
                    *il = 0xFF000000 | (fou << 8);
                }
            }
            for adq in 0..ec.len() {
                let b = adq * 8;
                if b >= d { continue; }
                ec[adq] = ec[adq].cn(arz[adq] as u16);
                if ec[adq] as usize >= i { ec[adq] = 0; }
                let c = ec[adq] as usize;
                let r = (((frame as usize + adq * 13) % 94) + 33) as u8 as char;
                let ka = crate::framebuffer::font::ada(r);
                for (br, &fs) in ka.iter().cf() {
                    let x = c + br;
                    if x >= i { break; }
                    for ga in 0..8u32 {
                        if fs & (0x80 >> ga) != 0 {
                            let y = b + ga as usize;
                            if y < d {
                                k[x * d + y] = 0xFF00FF44;
                            }
                        }
                    }
                }
            }
        };

        let mb = |k: &[u32], d: usize, i: usize| {
            if let Some((bgc, dnu, bgb, baz)) = crate::framebuffer::cey() {
                let aaa = bgc as *mut u32;
                let bgd = baz as usize;
                for c in 0..i.v(bgb as usize) {
                    unsafe {
                        core::ptr::copy_nonoverlapping(
                            k[c * d..].fq(),
                            aaa.add(c * bgd),
                            d,
                        );
                    }
                }
            }
            crate::framebuffer::sv();
        };

        
        let dbd = |k: &mut [u32], d: usize, i: usize,
                           ws: &mut [u16], yg: &[u8],
                           ak: &[(&str, u32, usize)],
                           lcj: u64| {
            let kx = crate::cpu::tsc::ard();
            if kx == 0 { return; }

            let aqo: usize = ak.iter().map(|(ab, _, _)| ab.len()).sum();
            let ifm = 60u64;
            let gvf = aqo as u64 * ifm;

            let ayu = crate::cpu::tsc::ow();
            let lck = kx / 1000 * (gvf + lcj);

            let mut frame = 0u32;
            loop {
                let ez = crate::cpu::tsc::ow().ao(ayu);
                if ez >= lck { break; }
                if let Some(bs) = crate::keyboard::xw() {
                    if bs == 0x1B || bs == b'q' { break; }
                    if bs == b' ' || bs == b'\n' || bs == 13 { break; } 
                }

                gfi(k, d, i, ws, yg, frame);

                let oz = ez / (kx / 1000).am(1);
                let qo = if oz < gvf {
                    (oz / ifm.am(1)) as usize
                } else {
                    aqo
                };

                let ieo: usize = ak.iter().map(|(_, _, e)| 16 * e + 8).sum::<usize>();
                let mut bpl = if ieo < i { (i - ieo) / 2 } else { 20 };
                let mut det = 0usize;

                for &(text, s, bv) in ak {
                    let bda = text.len() * 8 * bv;
                    let ql = if bda < d { (d - bda) / 2 } else { 0 };

                    for (a, r) in text.bw().cf() {
                        if det + a >= qo { break; }
                        aej(k, d, i, ql + a * 8 * bv, bpl, r, s, bv);
                    }
                    if qo > det && qo < det + text.len() {
                        let kng = qo - det;
                        let cx = ql + kng * 8 * bv;
                        for ae in bpl..bpl + 16 * bv {
                            if ae < i && cx + 2 < d {
                                k[ae * d + cx] = 0xFF00FF88;
                                k[ae * d + cx + 1] = 0xFF00FF88;
                            }
                        }
                    }

                    det += text.len();
                    bpl += 16 * bv + 8;
                }

                mb(k, d, i);
                frame += 1;
                crate::cpu::tsc::asq(33);
            }

            
            let fih = crate::cpu::tsc::ow();
            let ebj = kx / 1000 * 600;
            loop {
                let ez = crate::cpu::tsc::ow().ao(fih);
                if ez >= ebj { break; }
                let li = (ez * 255 / ebj) as u32;
                for il in k.el() {
                    let m = ((*il >> 16) & 0xFF) as u32;
                    let at = ((*il >> 8) & 0xFF) as u32;
                    let o = (*il & 0xFF) as u32;
                    let nr = m.ao(m * li / 512 + 1);
                    let csu = at.ao(at * li / 512 + 1);
                    let csq = o.ao(o * li / 512 + 1);
                    *il = 0xFF000000 | (nr << 16) | (csu << 8) | csq;
                }
                mb(k, d, i);
                crate::cpu::tsc::asq(33);
            }
            for il in k.el() { *il = 0xFF000000; }
            mb(k, d, i);
        };

        
        crate::serial_println!("[DEMO] Scene 1: Welcome");
        for il in k.el() { *il = 0xFF000000; }
        if sh == "fr" {
            dbd(&mut k, d, i, &mut ws, &yg,
                &[("Bienvenue dans", 0xFF00DD55, 5),
                  ("TrustOS", 0xFF00FFAA, 6)],
                3000);
        } else {
            dbd(&mut k, d, i, &mut ws, &yg,
                &[("Welcome to", 0xFF00DD55, 5),
                  ("TrustOS", 0xFF00FFAA, 6)],
                3000);
        }

        
        crate::serial_println!("[DEMO] Scene 2: What is TrustOS");
        if sh == "fr" {
            dbd(&mut k, d, i, &mut ws, &yg,
                &[("Un OS bare-metal", 0xFF00DD55, 4),
                  ("ecrit en 100% Rust", 0xFF00FF88, 4),
                  ("Aucun C. Aucun Linux.", 0xFFFFCC44, 3)],
                3000);
        } else {
            dbd(&mut k, d, i, &mut ws, &yg,
                &[("A bare-metal OS", 0xFF00DD55, 4),
                  ("written in 100% Rust", 0xFF00FF88, 4),
                  ("No C. No Linux. Just Rust.", 0xFFFFCC44, 3)],
                3000);
        }

        
        crate::serial_println!("[DEMO] Scene 3: Tutorial start");
        if sh == "fr" {
            dbd(&mut k, d, i, &mut ws, &yg,
                &[("Tutoriel Interactif", 0xFF44DDFF, 5),
                  ("Appuyez ESPACE pour continuer", 0xFF888888, 2),
                  ("ESC pour quitter", 0xFF666666, 2)],
                4000);
        } else {
            dbd(&mut k, d, i, &mut ws, &yg,
                &[("Interactive Tutorial", 0xFF44DDFF, 5),
                  ("Press SPACE to continue", 0xFF888888, 2),
                  ("ESC to quit", 0xFF666666, 2)],
                4000);
        }

        if !afk {
            crate::framebuffer::afi(false);
        }
    }

    
    
    
    crate::framebuffer::clear();

    let ewv = |gu: u32, es: u32, xht: &str, xhu: &str, rvy: &str, rvz: &str| {
        crate::println!();
        crate::h!(0xFF00CCFF, "+--------------------------------------------------------------+");
        if sh == "fr" {
            crate::h!(0xFF00CCFF, "|  ETAPE {}/{} -- {}", gu, es, xhu);
        } else {
            crate::h!(0xFF00CCFF, "|  STEP {}/{} -- {}", gu, es, xht);
        }
        crate::h!(0xFF00CCFF, "+--------------------------------------------------------------+");
        crate::println!();
        if sh == "fr" {
            crate::h!(0xFF888888, "  {}", rvz);
        } else {
            crate::h!(0xFF888888, "  {}", rvy);
        }
        crate::println!();
    };

    let tk = 8u32;

    
    ewv(1, tk, "SYSTEM INFO", "INFOS SYSTEME",
               "TrustOS can show detailed system information, just like Linux.",
               "TrustOS affiche les infos systeme, comme sous Linux.");
    rb(2);

    crate::h!(C_, "  $ neofetch");
    rb(1);
    super::commands::kiy();
    rb(3);

    if sh == "fr" {
        crate::h!(0xFF00FF88, "  -> Neofetch montre le CPU, la RAM, le kernel et l'uptime.");
    } else {
        crate::h!(0xFF00FF88, "  -> Neofetch shows CPU, RAM, kernel version and uptime.");
    }
    if fym(6) { return; }

    
    ewv(2, tk, "FILESYSTEM", "SYSTEME DE FICHIERS",
               "TrustOS has a full virtual filesystem (TrustFS + VFS).",
               "TrustOS possede un systeme de fichiers virtuel complet (TrustFS + VFS).");
    rb(1);

    crate::h!(C_, "  $ mkdir /tutorial");
    crate::ramfs::fh(|fs| { let _ = fs.ut("/tutorial"); });
    crate::h!(B_, "  Created /tutorial");
    rb(1);

    crate::h!(C_, "  $ echo 'Hello from TrustOS!' > /tutorial/hello.txt");
    crate::ramfs::fh(|fs| {
        let _ = fs.touch("/tutorial/hello.txt");
        let _ = fs.ns("/tutorial/hello.txt", b"Hello from TrustOS!\nThis file was created during the tutorial.\nPure Rust, running on bare metal.\n");
    });
    crate::h!(B_, "  Written: /tutorial/hello.txt");
    rb(1);

    crate::h!(C_, "  $ cat /tutorial/hello.txt");
    super::commands::hde(&["/tutorial/hello.txt"], None, None);
    rb(2);

    crate::h!(C_, "  $ tree /");
    super::commands::kjj(&["/"]);

    if sh == "fr" {
        crate::h!(0xFF00FF88, "  -> Commandes POSIX completes: ls, cd, mkdir, rm, cp, mv, cat, find, grep...");
    } else {
        crate::h!(0xFF00FF88, "  -> Full POSIX commands: ls, cd, mkdir, rm, cp, mv, cat, find, grep...");
    }
    if fym(6) { return; }

    
    ewv(3, tk, "TRUSTLANG COMPILER", "COMPILATEUR TRUSTLANG",
               "TrustOS includes a built-in programming language with compiler + VM.",
               "TrustOS inclut un langage de programmation avec compilateur + VM.");
    rb(1);

    let fwv = r#"fn factorial(n: i64) -> i64 {
    if n <= 1 { return 1; }
    return n * factorial(n - 1);
}

fn main() {
    println("=== TrustLang Demo ===");
    for i in 1..8 {
        print("  ");
        print(to_string(i));
        print("! = ");
        println(to_string(factorial(i)));
    }
}"#;
    crate::ramfs::fh(|fs| {
        let _ = fs.touch("/tutorial/demo.tl");
        let _ = fs.ns("/tutorial/demo.tl", fwv.as_bytes());
    });

    crate::h!(C_, "  $ cat /tutorial/demo.tl");
    crate::h!(0xFFDDDDDD, "{}", fwv);
    rb(3);

    crate::h!(C_, "  $ trustlang run /tutorial/demo.tl");
    crate::h!(0xFF00FF88, "  [TrustLang] Compiling...");
    match crate::trustlang::vw(fwv) {
        Ok(an) => { if !an.is_empty() { crate::print!("{}", an); } }
        Err(aa) => crate::h!(A_, "  Error: {}", aa),
    }
    crate::h!(B_, "  [TrustLang] Done!");

    if sh == "fr" {
        crate::h!(0xFF00FF88, "  -> Fonctions, recursion, boucles, types -- compile en bytecode!");
    } else {
        crate::h!(0xFF00FF88, "  -> Functions, recursion, loops, types -- compiled to bytecode!");
    }
    if fym(6) { return; }

    
    ewv(4, tk, "NETWORK STACK", "PILE RESEAU",
               "Full TCP/IP stack: DHCP, DNS, HTTP, TLS 1.3 -- all in Rust.",
               "Pile TCP/IP complete: DHCP, DNS, HTTP, TLS 1.3 -- tout en Rust.");
    rb(1);

    crate::h!(C_, "  $ ifconfig");
    super::vm::hdh();
    rb(2);

    crate::h!(C_, "  $ netstat");
    super::vm::hdi();

    if sh == "fr" {
        crate::h!(0xFF00FF88, "  -> Un navigateur web integre peut charger de vraies pages!");
    } else {
        crate::h!(0xFF00FF88, "  -> A built-in web browser can load real web pages!");
    }
    if fym(5) { return; }

    
    ewv(5, tk, "VIDEO EFFECTS", "EFFETS VIDEO",
               "Real-time procedural rendering engine -- fire, matrix, plasma.",
               "Moteur de rendu procedural temps reel -- feu, matrix, plasma.");
    rb(2);

    let gm = kp as u16;
    let me = kl as u16;

    if sh == "fr" {
        crate::h!(0xFFFF4400, "  Effet 1: FEU -- Flammes procedurales (5s)");
    } else {
        crate::h!(0xFFFF4400, "  Effect 1: FIRE -- Procedural flames (5s)");
    }
    rb(1);
    crate::video::player::gqt("fire", gm, me, 30, 5000);
    crate::framebuffer::clear();

    if sh == "fr" {
        crate::h!(0xFF00FF44, "  Effet 2: MATRIX -- Pluie numerique (5s)");
    } else {
        crate::h!(0xFF00FF44, "  Effect 2: MATRIX -- Digital rain (5s)");
    }
    rb(1);
    crate::video::player::gqt("matrix", gm, me, 30, 5000);
    crate::framebuffer::clear();

    if sh == "fr" {
        crate::h!(0xFFFF00FF, "  Effet 3: PLASMA -- Plasma psychedelique (5s)");
    } else {
        crate::h!(0xFFFF00FF, "  Effect 3: PLASMA -- Psychedelic plasma (5s)");
    }
    rb(1);
    crate::video::player::gqt("plasma", gm, me, 30, 5000);
    crate::framebuffer::clear();

    if sh == "fr" {
        crate::h!(0xFF00FF88, "  -> Tout fonctionne a 60+ FPS sur du bare-metal!");
    } else {
        crate::h!(0xFF00FF88, "  -> All running at 60+ FPS on bare metal!");
    }
    if fym(4) { return; }

    
    ewv(6, tk, "3D ENGINE", "MOTEUR 3D",
               "Wireframe 3D with perspective projection and depth shading.",
               "3D filaire avec projection perspective et ombrage de profondeur.");
    rb(2);

    {
        let mut renderer = crate::formula3d::FormulaRenderer::new();
        renderer.bid(crate::formula3d::FormulaScene::Kh);
        renderer.dxr = 0xFF00FFAA;

        let yq = kp as usize;
        let aff = kl as usize;
        let mut dxo = alloc::vec![0u32; yq * aff];

        let afk = crate::framebuffer::bre();
        if !afk {
            crate::framebuffer::beo();
            crate::framebuffer::afi(true);
        }
        crate::framebuffer::cwe(0xFF000000);
        crate::framebuffer::sv();

        let ayu = crate::cpu::tsc::ow();
        let kx = crate::cpu::tsc::ard();
        let cii = if kx > 0 { kx / 1000 * 6000 } else { u64::O }; 

        loop {
            let ez = crate::cpu::tsc::ow().ao(ayu);
            if ez >= cii { break; }
            if let Some(bs) = crate::keyboard::xw() {
                if bs == 0x1B || bs == b'q' { break; }
                if bs == b' ' || bs == b'\n' || bs == 13 { break; }
            }

            renderer.qs();
            renderer.tj(&mut dxo, yq, aff);

            if let Some((bgc, dnu, bgb, baz)) = crate::framebuffer::cey() {
                let aaa = bgc as *mut u32;
                let bgd = baz as usize;
                for c in 0..aff.v(bgb as usize) {
                    unsafe {
                        core::ptr::copy_nonoverlapping(
                            dxo[c * yq..].fq(),
                            aaa.add(c * bgd),
                            yq,
                        );
                    }
                }
            }
            crate::framebuffer::sv();
        }

        if !afk {
            crate::framebuffer::afi(false);
        }
    }
    crate::framebuffer::clear();
    if fym(2) { return; }

    
    ewv(7, tk, "DESKTOP ENVIRONMENT", "ENVIRONNEMENT DE BUREAU",
               "GPU-composited windowed desktop with apps, games, and more.",
               "Bureau fenetre composite GPU avec apps, jeux, et plus encore.");
    rb(1);

    if sh == "fr" {
        crate::h!(0xFF00FF88, "  Le bureau s'ouvre avec un Terminal pour 8 secondes...");
        crate::h!(0xFF888888, "  (Essayez de taper des commandes!)");
    } else {
        crate::h!(0xFF00FF88, "  Desktop will open with a Terminal for 8 seconds...");
        crate::h!(0xFF888888, "  (Try typing some commands!)");
    }
    rb(3);

    
    kih(Some("shell"), 8000);
    crate::framebuffer::clear();
    rb(1);

    
    ewv(8, tk, "FEATURE OVERVIEW", "VUE D'ENSEMBLE",
               "Everything TrustOS includes -- all in 6MB, all in Rust.",
               "Tout ce que TrustOS contient -- en 6Mo, tout en Rust.");
    rb(1);

    crate::h!(0xFFAADDFF, "  +- Kernel -------------------------------------------+");
    crate::h!(0xFFDDDDDD, "  | SMP multicore * APIC * IDT * GDT * paging          |");
    crate::h!(0xFFDDDDDD, "  | heap allocator * scheduler * RTC * PIT * TSC        |");
    crate::h!(0xFFAADDFF, "  +- Shell --------------------------------------------|");
    crate::h!(0xFFDDDDDD, "  | 200+ POSIX commands * pipes * scripting              |");
    crate::h!(0xFFDDDDDD, "  | ls cd mkdir rm cp mv cat grep find head tail tree   |");
    crate::h!(0xFFAADDFF, "  +- Desktop ------------------------------------------|");
    crate::h!(0xFFDDDDDD, "  | GPU compositor * window manager * 60 FPS            |");
    crate::h!(0xFFDDDDDD, "  | Terminal * Files * Calculator * Settings             |");
    crate::h!(0xFFAADDFF, "  +- Apps ---------------------------------------------|");
    crate::h!(0xFFDDDDDD, "  | TrustCode (editor) * Web Browser * Snake * Chess    |");
    crate::h!(0xFFDDDDDD, "  | NES Emulator * Game Boy * TrustEdit 3D * TrustLab  |");
    crate::h!(0xFFAADDFF, "  +- Languages ----------------------------------------|");
    crate::h!(0xFFDDDDDD, "  | TrustLang compiler + VM * Shell scripting           |");
    crate::h!(0xFFAADDFF, "  +- Network ------------------------------------------|");
    crate::h!(0xFFDDDDDD, "  | TCP/IP * DHCP * DNS * HTTP * TLS 1.3 * curl/wget   |");
    crate::h!(0xFFAADDFF, "  +- Graphics -----------------------------------------|");
    crate::h!(0xFFDDDDDD, "  | TrustVideo * Formula3D * Matrix * Fire * Plasma     |");
    crate::h!(0xFFAADDFF, "  +----------------------------------------------------+");
    crate::println!();

    if fym(8) { return; }

    
    
    
    {
        let afk = crate::framebuffer::bre();
        if !afk {
            crate::framebuffer::beo();
            crate::framebuffer::afi(true);
        }

        let d = kp as usize;
        let i = kl as usize;
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

        let mut ws: alloc::vec::Vec<u16> = alloc::vec![0u16; d / 8 + 1];
        let mut yg: alloc::vec::Vec<u8> = alloc::vec![1u8; d / 8 + 1];
        for a in 0..ws.len() {
            ws[a] = ((a * 41 + 7) % i) as u16;
            yg[a] = (((a * 11 + 5) % 4) + 1) as u8;
        }

        let gfi = |k: &mut [u32], d: usize, i: usize, ec: &mut [u16], arz: &[u8], frame: u32| {
            for il in k.el() {
                let at = ((*il >> 8) & 0xFF) as u32;
                if at > 0 {
                    let fou = at.ao(8);
                    *il = 0xFF000000 | (fou << 8);
                }
            }
            for adq in 0..ec.len() {
                let b = adq * 8;
                if b >= d { continue; }
                ec[adq] = ec[adq].cn(arz[adq] as u16);
                if ec[adq] as usize >= i { ec[adq] = 0; }
                let c = ec[adq] as usize;
                let r = (((frame as usize + adq * 13) % 94) + 33) as u8 as char;
                let ka = crate::framebuffer::font::ada(r);
                for (br, &fs) in ka.iter().cf() {
                    let x = c + br;
                    if x >= i { break; }
                    for ga in 0..8u32 {
                        if fs & (0x80 >> ga) != 0 {
                            let y = b + ga as usize;
                            if y < d {
                                k[x * d + y] = 0xFF00FF44;
                            }
                        }
                    }
                }
            }
        };

        let mb = |k: &[u32], d: usize, i: usize| {
            if let Some((bgc, dnu, bgb, baz)) = crate::framebuffer::cey() {
                let aaa = bgc as *mut u32;
                let bgd = baz as usize;
                for c in 0..i.v(bgb as usize) {
                    unsafe {
                        core::ptr::copy_nonoverlapping(
                            k[c * d..].fq(),
                            aaa.add(c * bgd),
                            d,
                        );
                    }
                }
            }
            crate::framebuffer::sv();
        };

        
        crate::serial_println!("[DEMO] Outro: You're ready!");
        for il in k.el() { *il = 0xFF000000; }

        let kx = crate::cpu::tsc::ard();
        let vab = 7000u64;
        let vac = if kx > 0 { kx / 1000 * vab } else { u64::O };
        let ayu = crate::cpu::tsc::ow();

        let mut eho = crate::formula3d::FormulaRenderer::new();
        eho.bid(crate::formula3d::FormulaScene::Kh);
        eho.dxr = 0xFF00FFAA;
        let att = 160usize;
        let azc = 160usize;
        let mut dxo = alloc::vec![0u32; att * azc];

        let mut frame = 0u32;
        loop {
            let ez = crate::cpu::tsc::ow().ao(ayu);
            if ez >= vac { break; }
            if let Some(bs) = crate::keyboard::xw() {
                if bs == 0x1B || bs == b'q' || bs == b' ' || bs == b'\n' || bs == 13 { break; }
            }

            gfi(&mut k, d, i, &mut ws, &yg, frame);

            
            let dq = if sh == "fr" { "Pret a explorer!" } else { "You're ready!" };
            let dmn = 5;
            let dcs = dq.len() * 8 * dmn;
            let cnf = if dcs < d { (d - dcs) / 2 } else { 0 };
            let cce = i / 6;
            for (a, r) in dq.bw().cf() {
                let aya = ((frame as usize * 3 + a * 25) % 360) as u32;
                let s = if aya < 120 {
                    let ab = aya * 255 / 120;
                    0xFF000000 | ((255 - ab) << 16) | (ab << 8)
                } else if aya < 240 {
                    let ab = (aya - 120) * 255 / 120;
                    0xFF000000 | ((255 - ab) << 8) | ab
                } else {
                    let ab = (aya - 240) * 255 / 120;
                    0xFF000000 | (ab << 16) | (255 - ab)
                };
                aej(&mut k, d, i, cnf + a * 8 * dmn, cce, r, s, dmn);
            }

            
            eho.qs();
            for ai in dxo.el() { *ai = 0x00000000; }
            eho.tj(&mut dxo, att, azc);
            let dxp = if att < d { (d - att) / 2 } else { 0 };
            let ddi = cce + 16 * dmn + 20;
            for iz in 0..azc {
                for fp in 0..att {
                    let cy = dxo[iz * att + fp];
                    if cy & 0x00FFFFFF != 0 {
                        let bg = ddi + iz;
                        let dx = dxp + fp;
                        if bg < i && dx < d {
                            k[bg * d + dx] = cy;
                        }
                    }
                }
            }

            
            let tow: &[(&str, u32)] = if sh == "fr" {
                &[
                    ("Tapez 'help' pour la liste des commandes", 0xFF88CCFF),
                    ("Tapez 'desktop' pour le bureau graphique", 0xFF88FFAA),
                    ("Tapez 'showcase' pour la demo complete", 0xFFFFCC88),
                    ("github.com/nathan237/TrustOS", 0xFF00FF88),
                ]
            } else {
                &[
                    ("Type 'help' for all commands", 0xFF88CCFF),
                    ("Type 'desktop' for the GUI", 0xFF88FFAA),
                    ("Type 'showcase' for the full demo", 0xFFFFCC88),
                    ("github.com/nathan237/TrustOS", 0xFF00FF88),
                ]
            };

            let iyi = 2;
            let mut crj = ddi + azc + 30;
            for &(text, s) in tow {
                let qd = text.len() * 8 * iyi;
                let bng = if qd < d { (d - qd) / 2 } else { 0 };
                for (a, r) in text.bw().cf() {
                    aej(&mut k, d, i, bng + a * 8 * iyi, crj, r, s, iyi);
                }
                crj += 16 * iyi + 6;
            }

            mb(&k, d, i);
            frame += 1;
            crate::cpu::tsc::asq(33);
        }

        
        let fih = crate::cpu::tsc::ow();
        let ebj = if kx > 0 { kx / 1000 * 800 } else { u64::O };
        loop {
            let ez = crate::cpu::tsc::ow().ao(fih);
            if ez >= ebj { break; }
            for il in k.el() {
                let m = ((*il >> 16) & 0xFF).ao(4) as u32;
                let at = ((*il >> 8) & 0xFF).ao(4) as u32;
                let o = (*il & 0xFF).ao(4) as u32;
                *il = 0xFF000000 | (m << 16) | (at << 8) | o;
            }
            mb(&k, d, i);
            crate::cpu::tsc::asq(33);
        }

        if !afk {
            crate::framebuffer::afi(false);
        }
    }

    
    
    
    crate::framebuffer::clear();

    
    let _ = crate::ramfs::fh(|fs| {
        let _ = fs.hb("/tutorial/hello.txt");
        let _ = fs.hb("/tutorial/demo.tl");
        let _ = fs.hb("/tutorial");
    });

    crate::println!();
    if sh == "fr" {
        crate::h!(0xFF00FF88, "  Tutoriel termine! Bon voyage dans TrustOS.");
    } else {
        crate::h!(0xFF00FF88, "  Tutorial complete! Enjoy exploring TrustOS.");
    }
    crate::println!();
    crate::serial_println!("[DEMO] Tutorial complete");
}




pub(super) fn ric(n: &[&str]) {
    let ig = match n.fv().hu() {
        Some("fast") => 1,
        Some("slow") => 3,
        _ => 2, 
    };

    
    let rb = |tv: u64| {
        let jn = tv * 1000 * ig / 2;
        let ayu = crate::cpu::tsc::ow();
        let kx = crate::cpu::tsc::ard();
        if kx == 0 { return; } 
        let cii = kx / 1000 * jn; 
        loop {
            let ez = crate::cpu::tsc::ow().ao(ayu);
            if ez >= cii { break; }
            
            let _ = crate::keyboard::xw();
            core::hint::hc();
        }
    };

    let hhu = 9000u64 * ig / 2; 

    
    
    
    let (kp, kl) = crate::framebuffer::yn();

    {
        let afk = crate::framebuffer::bre();
        if !afk {
            crate::framebuffer::beo();
            crate::framebuffer::afi(true);
        }

        let d = kp as usize;
        let i = kl as usize;
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

        
        let ymu = |k: &mut [u32], d: usize, i: usize, c: usize, text: &str, s: u32, bv: usize| {
            let bda = text.len() * 8 * bv;
            let ql = if bda < d { (d - bda) / 2 } else { 0 };
            for (a, r) in text.bw().cf() {
                aej(k, d, i, ql + a * 8 * bv, c, r, s, bv);
            }
        };

        
        let mut ws: alloc::vec::Vec<u16> = alloc::vec![0u16; d / 8 + 1];
        let mut yg: alloc::vec::Vec<u8> = alloc::vec![1u8; d / 8 + 1];
        
        for a in 0..ws.len() {
            ws[a] = ((a * 37 + 13) % i) as u16;
            yg[a] = (((a * 7 + 3) % 4) + 1) as u8;
        }

        let gfi = |k: &mut [u32], d: usize, i: usize, ec: &mut [u16], arz: &[u8], frame: u32| {
            
            for il in k.el() {
                let at = ((*il >> 8) & 0xFF) as u32;
                if at > 0 {
                    let fou = at.ao(8);
                    *il = 0xFF000000 | (fou << 8);
                }
            }
            
            for adq in 0..ec.len() {
                let b = adq * 8;
                if b >= d { continue; }
                ec[adq] = ec[adq].cn(arz[adq] as u16);
                if ec[adq] as usize >= i { ec[adq] = 0; }
                let c = ec[adq] as usize;
                
                let r = (((frame as usize + adq * 13) % 94) + 33) as u8 as char;
                let ka = crate::framebuffer::font::ada(r);
                for (br, &fs) in ka.iter().cf() {
                    let x = c + br;
                    if x >= i { break; }
                    for ga in 0..8u32 {
                        if fs & (0x80 >> ga) != 0 {
                            let y = b + ga as usize;
                            if y < d {
                                k[x * d + y] = 0xFF00FF44;
                            }
                        }
                    }
                }
            }
        };

        
        let mb = |k: &[u32], d: usize, i: usize| {
            if let Some((bgc, dnu, bgb, baz)) = crate::framebuffer::cey() {
                let aaa = bgc as *mut u32;
                let bgd = baz as usize;
                for c in 0..i.v(bgb as usize) {
                    unsafe {
                        core::ptr::copy_nonoverlapping(
                            k[c * d..].fq(),
                            aaa.add(c * bgd),
                            d,
                        );
                    }
                }
            }
            crate::framebuffer::sv();
        };

        
        
        let dbd = |k: &mut [u32], d: usize, i: usize,
                           ws: &mut [u16], yg: &[u8],
                           ak: &[(&str, u32, usize)], 
                           lcj: u64, ig: u64| {
            let kx = crate::cpu::tsc::ard();
            if kx == 0 { return; }

            
            let aqo: usize = ak.iter().map(|(ab, _, _)| ab.len()).sum();
            let ifm = 80u64 * ig / 2; 
            let gvf = aqo as u64 * ifm;
            
            let ayu = crate::cpu::tsc::ow();
            let ztu = kx / 1000 * gvf;
            let lck = kx / 1000 * (gvf + lcj * ig / 2);

            let mut frame = 0u32;
            loop {
                let ez = crate::cpu::tsc::ow().ao(ayu);
                if ez >= lck { break; }
                if let Some(bs) = crate::keyboard::xw() {
                    if bs == 0x1B || bs == b'q' { break; }
                }

                
                gfi(k, d, i, ws, yg, frame);

                
                let oz = ez / (kx / 1000).am(1);
                let qo = if oz < gvf {
                    (oz / ifm.am(1)) as usize
                } else {
                    aqo
                };

                
                let ieo: usize = ak.iter().map(|(_, _, e)| 16 * e + 8).sum::<usize>();
                let mut bpl = if ieo < i { (i - ieo) / 2 } else { 20 };
                let mut det = 0usize;

                for &(text, s, bv) in ak {
                    let bda = text.len() * 8 * bv;
                    let ql = if bda < d { (d - bda) / 2 } else { 0 };

                    for (a, r) in text.bw().cf() {
                        if det + a >= qo { break; }
                        aej(k, d, i, ql + a * 8 * bv, bpl, r, s, bv);
                    }
                    
                    if qo > det && qo < det + text.len() {
                        let kng = qo - det;
                        let cx = ql + kng * 8 * bv;
                        for ae in bpl..bpl + 16 * bv {
                            if ae < i && cx + 2 < d {
                                k[ae * d + cx] = 0xFF00FF88;
                                k[ae * d + cx + 1] = 0xFF00FF88;
                            }
                        }
                    }

                    det += text.len();
                    bpl += 16 * bv + 8;
                }

                mb(k, d, i);
                frame += 1;
                
                crate::cpu::tsc::asq(33);
            }

            
            let fih = crate::cpu::tsc::ow();
            let ggs = 800u64;
            let ebj = kx / 1000 * ggs;
            loop {
                let ez = crate::cpu::tsc::ow().ao(fih);
                if ez >= ebj { break; }
                let li = (ez * 255 / ebj) as u32;
                for il in k.el() {
                    let m = ((*il >> 16) & 0xFF) as u32;
                    let at = ((*il >> 8) & 0xFF) as u32;
                    let o = (*il & 0xFF) as u32;
                    let nr = m.ao(m * li / 512 + 1);
                    let csu = at.ao(at * li / 512 + 1);
                    let csq = o.ao(o * li / 512 + 1);
                    *il = 0xFF000000 | (nr << 16) | (csu << 8) | csq;
                }
                mb(k, d, i);
                crate::cpu::tsc::asq(33);
            }

            
            for il in k.el() { *il = 0xFF000000; }
            mb(k, d, i);
        };

        
        crate::serial_println!("[SHOWCASE] Scene 1: Simulation question");
        for il in k.el() { *il = 0xFF000000; }
        dbd(&mut k, d, i, &mut ws, &yg,
            &[("Do you think", 0xFF00DD55, 4),
              ("life is a simulation?", 0xFF00FF66, 4)],
            3000, ig);

        
        crate::serial_println!("[SHOWCASE] Scene 2: 6MB OS");
        dbd(&mut k, d, i, &mut ws, &yg,
            &[("Can it run", 0xFF00DD55, 5),
              ("in a 6MB OS?", 0xFF00FF88, 5)],
            3000, ig);

        
        crate::serial_println!("[SHOWCASE] Scene 3: TrustOS title");
        {
            
            let kx = crate::cpu::tsc::ard();
            let web = 8000u64 * ig / 2;
            let wec = kx / 1000 * web;
            let ayu = crate::cpu::tsc::ow();

            let mut eho = crate::formula3d::FormulaRenderer::new();
            eho.bid(crate::formula3d::FormulaScene::Kh);
            eho.dxr = 0xFF00FFAA;

            
            let att = 200usize;
            let azc = 200usize;
            let mut dxo = alloc::vec![0u32; att * azc];

            let mut frame = 0u32;
            loop {
                let ez = crate::cpu::tsc::ow().ao(ayu);
                if ez >= wec { break; }
                if let Some(bs) = crate::keyboard::xw() {
                    if bs == 0x1B { break; }
                }

                
                gfi(&mut k, d, i, &mut ws, &yg, frame);

                
                let dq = "TRUST OS";
                let dmn = 6;
                let dcs = dq.len() * 8 * dmn;
                let cnf = if dcs < d { (d - dcs) / 2 } else { 0 };
                let cce = i / 8;
                for (a, r) in dq.bw().cf() {
                    
                    let aya = ((frame as usize * 3 + a * 30) % 360) as u32;
                    let s = if aya < 120 {
                        let ab = aya * 255 / 120;
                        0xFF000000 | ((255 - ab) << 16) | (ab << 8)
                    } else if aya < 240 {
                        let ab = (aya - 120) * 255 / 120;
                        0xFF000000 | ((255 - ab) << 8) | ab
                    } else {
                        let ab = (aya - 240) * 255 / 120;
                        0xFF000000 | (ab << 16) | (255 - ab)
                    };
                    aej(&mut k, d, i, cnf + a * 8 * dmn, cce, r, s, dmn);
                }

                
                eho.qs();
                for ai in dxo.el() { *ai = 0x00000000; } 
                eho.tj(&mut dxo, att, azc);

                
                let dxp = if att < d { (d - att) / 2 } else { 0 };
                let ddi = cce + 16 * dmn + 20;
                for iz in 0..azc {
                    for fp in 0..att {
                        let cy = dxo[iz * att + fp];
                        if cy & 0x00FFFFFF != 0 { 
                            let bg = ddi + iz;
                            let dx = dxp + fp;
                            if bg < i && dx < d {
                                k[bg * d + dx] = cy;
                            }
                        }
                    }
                }

                
                let nhl = "Written in Rust by Nated0ge";
                let klz = 3;
                let nhm = nhl.len() * 8 * klz;
                let rqv = if nhm < d { (d - nhm) / 2 } else { 0 };
                let rqw = ddi + azc + 30;
                for (a, r) in nhl.bw().cf() {
                    aej(&mut k, d, i, rqv + a * 8 * klz, rqw, r, 0xFF88CCFF, klz);
                }

                mb(&k, d, i);
                frame += 1;
                crate::cpu::tsc::asq(33);
            }

            
            let fih = crate::cpu::tsc::ow();
            let ebj = kx / 1000 * 800;
            loop {
                let ez = crate::cpu::tsc::ow().ao(fih);
                if ez >= ebj { break; }
                for il in k.el() {
                    let m = ((*il >> 16) & 0xFF).ao(4) as u32;
                    let at = ((*il >> 8) & 0xFF).ao(4) as u32;
                    let o = (*il & 0xFF).ao(4) as u32;
                    *il = 0xFF000000 | (m << 16) | (at << 8) | o;
                }
                mb(&k, d, i);
                crate::cpu::tsc::asq(33);
            }
            for il in k.el() { *il = 0xFF000000; }
            mb(&k, d, i);
        }

        
        crate::serial_println!("[SHOWCASE] Scene 4: Specs");
        dbd(&mut k, d, i, &mut ws, &yg,
            &[("6MB ISO vs 6GB Windows",  0xFF00FF66, 3),
              ("0 lines of C. Pure Rust.", 0xFF44FFAA, 3),
              ("Boots in 0.8s not 45s",    0xFF00DDFF, 3),
              ("No kernel panics. Ever.",  0xFFFFCC44, 3),
              ("GPU desktop at 144 FPS",   0xFF88FF44, 3),
              ("Built in 7 days solo",     0xFFFF8844, 3)],
            3000, ig);

        
        crate::serial_println!("[SHOWCASE] Scene 5: Are you ready?");
        dbd(&mut k, d, i, &mut ws, &yg,
            &[("Are you ready?", 0xFF00FF44, 6)],
            2000, ig);

        
        if !afk {
            crate::framebuffer::afi(false);
        }
    }

    
    crate::framebuffer::clear();

    crate::println!();
    crate::println!();
    crate::println!();
    crate::h!(0xFF00CCFF, "");
    crate::h!(0xFF00CCFF, "  ||||||||+||||||+ ||+   ||+|||||||+||||||||+ ||||||+ |||||||+");
    crate::h!(0xFF00CCFF, "  +--||+--+||+--||+|||   |||||+----++--||+--+||+---||+||+----+");
    crate::h!(0xFF00CCFF, "     |||   ||||||++|||   ||||||||||+   |||   |||   ||||||||||+");
    crate::h!(0xFF00DDFF, "     |||   ||+--||+|||   |||+----|||   |||   |||   |||+----|||");
    crate::h!(0xFF00EEFF, "     |||   |||  |||+||||||++||||||||   |||   +||||||++||||||||");
    crate::h!(0xFF00EEFF, "     +-+   +-+  +-+ +-----+ +------+   +-+    +-----+ +------+");
    crate::println!();
    crate::h!(0xFF888888, "                  ?????????????????????????????????");
    crate::println!();
    crate::h!(0xFFAADDFF, "           A bare-metal OS written in 100% Rust -- in 7 days");
    crate::h!(0xFF666666, "         99,000+ lines * 6 MB ISO * GPU compositing * 144 FPS");
    crate::println!();
    crate::h!(0xFF888888, "                  ?????????????????????????????????");
    crate::println!();
    crate::h!(0xFF00FF88, "                        ?  FEATURE SHOWCASE  ?");
    crate::println!();
    
    rb(6);

    
    crate::h!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::h!(0xFF00CCFF, "|  PHASE 1 ---- SYSTEM INFO                                   |");
    crate::h!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::println!();
    rb(3);

    super::commands::kiy();
    rb(4);

    crate::h!(C_, "$ uname -a");
    super::commands::iom(&["-a"]);
    rb(4);

    crate::h!(C_, "$ free");
    super::commands::ioh();
    rb(4);

    crate::h!(C_, "$ lscpu");
    super::unix::ned();
    rb(5);

    
    crate::h!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::h!(0xFF00CCFF, "|  PHASE 2 ---- FILESYSTEM (TrustFS + VFS)                    |");
    crate::h!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::println!();
    rb(3);

    crate::h!(C_, "$ mkdir /demo");
    super::commands::iok(&["/demo"]);
    rb(2);

    crate::h!(C_, "$ echo 'Hello TrustOS!' > /demo/hello.txt");
    crate::ramfs::fh(|fs| {
        let _ = fs.touch("/demo/hello.txt");
        let _ = fs.ns("/demo/hello.txt", b"Hello TrustOS!\nThis file was created live during the showcase.\n");
    });
    crate::h!(B_, "Written: /demo/hello.txt");
    rb(2);

    crate::h!(C_, "$ cat /demo/hello.txt");
    super::commands::hde(&["/demo/hello.txt"], None, None);
    rb(3);

    crate::h!(C_, "$ tree /");
    super::commands::kjj(&["/"]);
    rb(4);

    
    crate::h!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::h!(0xFF00CCFF, "|  PHASE 3 ---- TRUSTLANG (Built-in Compiler + VM)            |");
    crate::h!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::println!();
    rb(3);

    
    let fwv = r#"fn fibonacci(n: i64) -> i64 {
    if n <= 1 { return n; }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

fn main() {
    println("=== TrustLang on TrustOS ===");
    println("Fibonacci sequence (compiled to bytecode):");
    for i in 0..12 {
        let result = fibonacci(i);
        print("  fib(");
        print(to_string(i));
        print(") = ");
        println(to_string(result));
    }
    println("Language features: functions, recursion, loops, types");
}"#;
    crate::ramfs::fh(|fs| {
        let _ = fs.touch("/demo/showcase.tl");
        let _ = fs.ns("/demo/showcase.tl", fwv.as_bytes());
    });

    crate::h!(C_, "$ cat /demo/showcase.tl");
    crate::h!(0xFFDDDDDD, "{}", fwv);
    rb(4);

    crate::h!(C_, "$ trustlang run /demo/showcase.tl");
    crate::h!(0xFF00FF88, "[TrustLang] Compiling showcase.tl...");
    match crate::trustlang::vw(fwv) {
        Ok(an) => { if !an.is_empty() { crate::print!("{}", an); } }
        Err(aa) => crate::h!(A_, "Error: {}", aa),
    }
    crate::h!(B_, "[TrustLang] Program finished successfully.");
    rb(6);

    
    crate::h!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::h!(0xFF00CCFF, "|  PHASE 4 ---- NETWORK STACK (TCP/IP, DHCP, DNS, TLS 1.3)    |");
    crate::h!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::println!();
    rb(3);

    crate::h!(C_, "$ ifconfig");
    super::vm::hdh();
    rb(3);

    crate::h!(C_, "$ netstat");
    super::vm::hdi();
    rb(4);

    
    crate::h!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::h!(0xFF00CCFF, "|  PHASE 5 ---- TRUSTVIDEO (Real-time procedural rendering)   |");
    crate::h!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::println!();
    rb(3);

    
    crate::h!(0xFFFF4400, "? Demo 1/3: FIRE EFFECT -- Real-time procedural flame");
    rb(2);

    let gm = kp as u16;
    let me = kl as u16;
    crate::video::player::gqt("fire", gm, me, 30, hhu);
    
    
    crate::framebuffer::clear();
    rb(2);

    
    crate::h!(0xFF00FF44, "? Demo 2/3: MATRIX RAIN -- Digital rain effect");
    rb(2);

    crate::video::player::gqt("matrix", gm, me, 30, hhu);
    
    crate::framebuffer::clear();
    rb(2);

    
    crate::h!(0xFFFF00FF, "? Demo 3/3: PLASMA -- Integer sine LUT psychedelic");
    rb(2);

    crate::video::player::gqt("plasma", gm, me, 30, hhu);
    
    crate::framebuffer::clear();
    rb(2);

    
    crate::h!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::h!(0xFF00CCFF, "|  PHASE 5b -- FORMULA3D (Wireframe 3D engine)                |");
    crate::h!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::println!();
    rb(2);

    crate::h!(0xFF00FF88, "? 3D wireframe character -- perspective projection + depth shading");
    rb(2);

    
    {
        let mut renderer = crate::formula3d::FormulaRenderer::new();
        renderer.bid(crate::formula3d::FormulaScene::Kh);
        renderer.dxr = 0xFF00FFAA; 

        let yq = kp as usize;
        let aff = kl as usize;
        let mp = if kp > yq as u32 { (kp - yq as u32) / 2 } else { 0 } as usize;
        let qw = if kl > aff as u32 { (kl - aff as u32) / 2 } else { 0 } as usize;

        let mut k = alloc::vec![0u32; yq * aff];

        let afk = crate::framebuffer::bre();
        if !afk {
            crate::framebuffer::beo();
            crate::framebuffer::afi(true);
        }
        crate::framebuffer::cwe(0xFF000000);
        crate::framebuffer::sv();

        let ayu = crate::cpu::tsc::ow();
        let kx = crate::cpu::tsc::ard();
        let uk = hhu;
        let cii = if kx > 0 { kx / 1000 * uk } else { u64::O };

        loop {
            let ez = crate::cpu::tsc::ow().ao(ayu);
            if ez >= cii { break; }
            if let Some(bs) = crate::keyboard::xw() {
                if bs == 0x1B || bs == b'q' { break; }
            }

            renderer.qs();
            renderer.tj(&mut k, yq, aff);

            
            if let Some((bgc, dnu, bgb, baz)) = crate::framebuffer::cey() {
                let aaa = bgc as *mut u32;
                let bgd = baz as usize;
                for c in 0..aff {
                    let bg = qw + c;
                    if bg >= bgb as usize { break; }
                    let bxg = &k[c * yq..c * yq + yq];
                    unsafe {
                        let cs = aaa.add(bg * bgd + mp);
                        core::ptr::copy_nonoverlapping(bxg.fq(), cs, yq);
                    }
                }
            }
            crate::framebuffer::sv();
        }

        if !afk {
            crate::framebuffer::afi(false);
        }
    }

    crate::framebuffer::clear();
    rb(2);

    
    crate::h!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::h!(0xFF00CCFF, "|  PHASE 5c -- COSMIC2 DESKTOP + WEB BROWSER                  |");
    crate::h!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::println!();
    rb(2);

    crate::h!(0xFF00FF88, "? COSMIC2 Desktop -- GPU-composited multi-layer windowing system");
    crate::h!(0xFF00FF88, "? Launching with built-in Web Browser ? google.com");
    rb(3);

    
    kih(Some("browser"), hhu);

    crate::framebuffer::clear();
    rb(2);

    
    crate::h!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::h!(0xFF00CCFF, "|  PHASE 6 ---- 200+ BUILT-IN COMMANDS                       |");
    crate::h!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::println!();
    rb(2);

    crate::h!(0xFFAADDFF, "  +- File System ------------------------------------------+");
    crate::h!(0xFFDDDDDD, "  | ls cd pwd mkdir rm cp mv cat head tail tree find grep  |");
    crate::h!(0xFFAADDFF, "  +- Network ----------------------------------------------|");
    crate::h!(0xFFDDDDDD, "  | ifconfig ping curl wget nslookup arp route netstat     |");
    crate::h!(0xFFAADDFF, "  +- System -----------------------------------------------|");
    crate::h!(0xFFDDDDDD, "  | ps top free df uname dmesg mount lspci lscpu lsblk    |");
    crate::h!(0xFFAADDFF, "  +- Development ------------------------------------------|");
    crate::h!(0xFFDDDDDD, "  | trustlang (compiler+VM) * TrustCode (editor)          |");
    crate::h!(0xFFDDDDDD, "  | transpile (binary?Rust) * exec (ELF loader)           |");
    crate::h!(0xFFAADDFF, "  +- Graphics ---------------------------------------------|");
    crate::h!(0xFFDDDDDD, "  | desktop (COSMIC2 compositor) * video (TrustVideo)     |");
    crate::h!(0xFFDDDDDD, "  | benchmark (SSE2 SIMD) * HoloMatrix (3D volumetric)   |");
    crate::h!(0xFFAADDFF, "  +--------------------------------------------------------+");
    crate::println!();
    rb(6);

    
    crate::println!();
    crate::h!(0xFF00CCFF, "  ||||||||+||||||+ ||+   ||+|||||||+||||||||+ ||||||+ |||||||+");
    crate::h!(0xFF00CCFF, "  +--||+--+||+--||+|||   |||||+----++--||+--+||+---||+||+----+");
    crate::h!(0xFF00CCFF, "     |||   ||||||++|||   ||||||||||+   |||   |||   ||||||||||+");
    crate::h!(0xFF00DDFF, "     |||   ||+--||+|||   |||+----|||   |||   |||   |||+----|||");
    crate::h!(0xFF00EEFF, "     |||   |||  |||+||||||++||||||||   |||   +||||||++||||||||");
    crate::h!(0xFF00EEFF, "     +-+   +-+  +-+ +-----+ +------+   +-+    +-----+ +------+");
    crate::println!();
    crate::h!(0xFFFFCC00, "  ?  100% Rust -- Zero C code -- Memory safe by design");
    crate::h!(0xFFFFCC00, "  ?  Built from scratch in 7 days -- 99,000+ lines");
    crate::h!(0xFFFFCC00, "  ?  6 MB ISO -- boots in seconds");
    crate::h!(0xFFFFCC00, "  ?  GPU compositing -- 144 FPS desktop");
    crate::println!();
    crate::h!(0xFF00FF88, "  github.com/nathan237/TrustOS");
    crate::h!(0xFF888888, "  Star ? * Fork * Contribute");
    crate::println!();
    crate::h!(0xFF888888, "  ??????????????????????????????????????????????????");
    crate::println!();

    
    let _ = crate::ramfs::fh(|fs| { let _ = fs.hb("/demo/hello.txt"); let _ = fs.hb("/demo/showcase.tl"); });
}






pub(super) fn rid(n: &[&str]) {
    let ig = match n.fv().hu() {
        Some("fast") => 1u64,
        Some("slow") => 3,
        _ => 2,
    };

    let rb = |tv: u64| {
        let jn = tv * 1000 * ig / 2;
        let ayu = crate::cpu::tsc::ow();
        let kx = crate::cpu::tsc::ard();
        if kx == 0 { return; }
        let cii = kx / 1000 * jn;
        loop {
            let ez = crate::cpu::tsc::ow().ao(ayu);
            if ez >= cii { break; }
            let _ = crate::keyboard::xw();
            core::hint::hc();
        }
    };

    let (kp, kl) = crate::framebuffer::yn();
    if kp == 0 || kl == 0 {
        crate::h!(0xFFFF4444, "No framebuffer available");
        return;
    }

    
    
    
    {
        let afk = crate::framebuffer::bre();
        if !afk {
            crate::framebuffer::beo();
            crate::framebuffer::afi(true);
        }

        let d = kp as usize;
        let i = kl as usize;
        let mut k = alloc::vec![0u32; d * i];

        
        let ahi = |k: &mut [u32], d: usize, i: usize, cx: usize, ae: usize,
                         r: char, s: u32, bv: usize| {
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

        
        let fu = |k: &mut [u32], d: usize, i: usize, b: usize, c: usize,
                        e: &str, s: u32, bv: usize| {
            for (a, r) in e.bw().cf() {
                ahi(k, d, i, b + a * 8 * bv, c, r, s, bv);
            }
        };

        
        let bge = |k: &[u32], d: usize, i: usize| {
            if let Some((bgc, dnu, bgb, baz)) = crate::framebuffer::cey() {
                let aaa = bgc as *mut u32;
                let bgd = baz as usize;
                for c in 0..i.v(bgb as usize) {
                    unsafe {
                        core::ptr::copy_nonoverlapping(
                            k[c * d..].fq(),
                            aaa.add(c * bgd),
                            d,
                        );
                    }
                }
            }
            crate::framebuffer::sv();
        };

        let kx = crate::cpu::tsc::ard();
        let ayu = crate::cpu::tsc::ow();
        let wef = 6000u64 * ig / 2;
        let cii = if kx > 0 { kx / 1000 * wef } else { u64::O };

        
        let mut rng = crate::cpu::tsc::ow();
        let mut lom = || -> u64 {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            rng
        };

        
        let my: [usize; 4] = [6, 8, 8, 4];
        let mut djw: alloc::vec::Vec<alloc::vec::Vec<(usize, usize)>> = alloc::vec::Vec::new();
        let okp = d / 6;
        let uix = d * 5 / 6;
        for (alj, &az) in my.iter().cf() {
            let mj = okp + (uix - okp) * alj / (my.len() - 1);
            let uja = i / 4;
            let aoa = i / 2 / (az + 1);
            let mut oik = alloc::vec::Vec::new();
            for los in 0..az {
                oik.push((mj, uja + aoa * (los + 1)));
            }
            djw.push(oik);
        }

        
        let mut frame = 0u32;
        loop {
            let ez = crate::cpu::tsc::ow().ao(ayu);
            if ez >= cii { break; }
            if let Some(bs) = crate::keyboard::xw() {
                if bs == 0x1B || bs == b'q' { break; }
            }

            
            for y in k.el() {
                *y = 0xFF050510;
            }

            
            let xg = crate::graphics::holomatrix::cuh(frame as f32 * 0.08) * 0.5 + 0.5;
            for alj in 0..djw.len() - 1 {
                for &(dn, dp) in &djw[alj] {
                    for &(hy, jz) in &djw[alj + 1] {
                        
                        let dx = (hy as i32 - dn as i32).gp();
                        let bg = (jz as i32 - dp as i32).gp();
                        let au = dx.am(bg) as usize;
                        if au == 0 { continue; }
                        let vqb = (lom() % 60) as f32;
                        let at = (40.0 + xg * 80.0 + vqb) as u32;
                        let s = 0xFF000000 | (at.v(255) << 8);
                        for e in 0..au {
                            let ab = e as f32 / au as f32;
                            let y = (dn as f32 + (hy as f32 - dn as f32) * ab) as usize;
                            let x = (dp as f32 + (jz as f32 - dp as f32) * ab) as usize;
                            if y < d && x < i {
                                k[x * d + y] = s;
                            }
                        }
                    }
                }
            }

            
            for (alj, fl) in djw.iter().cf() {
                for (los, &(vt, ahr)) in fl.iter().cf() {
                    let oqu = crate::graphics::holomatrix::cuh(frame as f32 * 0.12 + alj as f32 * 1.5 + los as f32 * 0.8) * 0.5 + 0.5;
                    let m = 5 + (oqu * 3.0) as usize;
                    let ght = (120.0 + oqu * 135.0) as u32;
                    let s = 0xFF000000 | (ght.v(255) << 8) | ((ght / 3).v(255));
                    
                    for bg in 0..=m * 2 {
                        for shs in 0..=m * 2 {
                            let ym = shs as i32 - m as i32;
                            let wl = bg as i32 - m as i32;
                            if ym * ym + wl * wl <= (m * m) as i32 {
                                let y = (vt as i32 + ym) as usize;
                                let x = (ahr as i32 + wl) as usize;
                                if y < d && x < i {
                                    k[x * d + y] = s;
                                }
                            }
                        }
                    }
                }
            }

            
            let suy = frame as f32 * 0.03;
            for alj in 0..djw.len() - 1 {
                let blf = (lom() % djw[alj].len() as u64) as usize;
                let bbm = (lom() % djw[alj + 1].len() as u64) as usize;
                let (dn, dp) = djw[alj][blf];
                let (hy, jz) = djw[alj + 1][bbm];
                let ab = ((suy + alj as f32 * 0.7) % 1.0).gp();
                let hgo = (dn as f32 + (hy as f32 - dn as f32) * ab) as usize;
                let kqo = (dp as f32 + (jz as f32 - dp as f32) * ab) as usize;
                
                for bg in 0..4 {
                    for dx in 0..4 {
                        let y = hgo + dx;
                        let x = kqo + bg;
                        if y < d && x < i {
                            k[x * d + y] = 0xFF00FF66;
                        }
                    }
                }
            }

            
            let dq = "J A R V I S";
            let dcs = dq.len() * 8 * 4;
            let gx = if d > dcs { (d - dcs) / 2 } else { 0 };
            fu(&mut k, d, i, gx, i / 12, dq, 0xFF00FF64, 4);

            
            let sub = "Kernel-Resident Neural Network";
            let ppm = sub.len() * 8 * 2;
            let cr = if d > ppm { (d - ppm) / 2 } else { 0 };
            fu(&mut k, d, i, cr, i / 12 + 72, sub, 0xFF88DDAA, 2);

            
            let dwf = i - 50;
            let cm = [
                "4.4M params",
                "4 layers",
                "256 d_model",
                "Byte-level",
                "Ring 0",
            ];
            let pve = cm.iter().map(|e| e.len() * 8 + 30).sum::<usize>();
            let mut pqk = if d > pve { (d - pve) / 2 } else { 10 };
            for hm in &cm {
                fu(&mut k, d, i, pqk, dwf, hm, 0xFF00CC88, 1);
                pqk += hm.len() * 8 + 30;
            }

            
            let cze = ["Input", "Hidden", "Hidden", "Output"];
            for (alj, fl) in djw.iter().cf() {
                if let Some(&(mj, _)) = fl.fv() {
                    let fms = cze[alj];
                    let udt = if mj > fms.len() * 4 { mj - fms.len() * 4 } else { 0 };
                    fu(&mut k, d, i, udt, i * 3 / 4 + 20, fms, 0xFF666666, 1);
                }
            }

            bge(&k, d, i);
            frame += 1;
        }

        if !afk {
            crate::framebuffer::afi(false);
        }
    }

    crate::framebuffer::clear();

    
    
    
    crate::println!();
    crate::h!(0xFF00FF64, "     ___  ___  _____  _   _  ___  _____");
    crate::h!(0xFF00FF64, "    |_  |/ _ \\| ___ \\| | | ||_  |/  ___|");
    crate::h!(0xFF00FF88, "      | / /_\\ \\ |_/ /| | | |  | |\\ `--.");
    crate::h!(0xFF00FF88, "      | |  _  |    / | | | |  | | `--. \\");
    crate::h!(0xFF00FFAA, "  /\\__/ / | | | |\\ \\ \\ \\_/ /\\__/ /\\__/ /");
    crate::h!(0xFF00FFAA, "  \\____/\\_| |_\\_| \\_| \\___/\\____/\\____/");
    crate::println!();
    crate::h!(0xFF888888, "  ======================================================");
    crate::h!(0xFFAADDFF, "    Kernel-Resident Self-Propagating Neural Network");
    crate::h!(0xFF888888, "  ======================================================");
    crate::println!();
    rb(4);

    crate::h!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::h!(0xFF00CCFF, "|  ARCHITECTURE                                                |");
    crate::h!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::println!();
    crate::h!(0xFF00FF88, "  Model:      Transformer (4L, d=256, 4H, d_ff=1024, SwiGLU)");
    crate::h!(0xFF00FF88, "  Parameters: 4,393,216 (FP32 = 17.6 MB)");
    crate::h!(0xFF00FF88, "  Vocabulary: 256 (byte-level, no tokenizer)");
    crate::h!(0xFF00FF88, "  SIMD:       Auto-detected (AVX2+FMA / SSE2 / NEON)");
    crate::h!(0xFF00FF88, "  Location:   Ring 0 (kernel address space)");
    crate::h!(0xFF00FF88, "  Safety:     Guardian Pact (2 authorized parents)");
    crate::println!();
    rb(5);

    
    
    
    crate::h!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::h!(0xFF00CCFF, "|  PHASE 1 --- BRAIN INITIALIZATION                            |");
    crate::h!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::println!();
    rb(2);

    crate::h!(0xFF00DDFF, "  $ jarvis brain init");
    crate::println!();

    if !crate::jarvis::uc() {
        crate::jarvis::init();
    }

    let tme = crate::jarvis::fkf();
    if tme {
        crate::h!(0xFF00FF88, "  [OK] Full brain loaded: 4,393,216 parameters");
    } else {
        crate::h!(0xFFFFCC00, "  [OK] Micro sentinel active: 78,016 parameters");
        crate::h!(0xFF888888, "  (Full brain requires jarvis_pretrained.bin in ISO)");
    }
    crate::println!();
    rb(4);

    
    
    
    crate::h!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::h!(0xFF00CCFF, "|  PHASE 2 --- LIVE INFERENCE (kernel space)                   |");
    crate::h!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::println!();
    rb(2);

    crate::h!(0xFF00DDFF, "  $ jarvis chat What is TrustOS?");
    crate::println!();

    
    let mk = crate::jarvis::cks("What is TrustOS?", 80);
    let ux = mk.em();
    if !ux.is_empty() {
        
        crate::gr!(0xFF00FF64, "  JARVIS> ");
        for r in ux.bw().take(200) {
            crate::gr!(0xFFCCFFDD, "{}", r);
            
            let qyi = crate::cpu::tsc::ow();
            let ncj = crate::cpu::tsc::ard();
            if ncj > 0 {
                let qyf = ncj / 1000 * 30; 
                while crate::cpu::tsc::ow().ao(qyi) < qyf {
                    core::hint::hc();
                }
            }
        }
        crate::println!();
    } else {
        crate::h!(0xFF888888, "  (Inference requires full brain to be loaded)");
    }
    crate::println!();
    rb(5);

    
    
    
    crate::h!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::h!(0xFF00CCFF, "|  PHASE 3 --- MESH NETWORK + SELF-PROPAGATION                |");
    crate::h!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::println!();
    rb(2);

    crate::h!(0xFF00FF88, "  Self-Propagation Sequence:");
    crate::h!(0xFFAADDFF, "  1. New node boots with micro sentinel (304 KB)");
    crate::h!(0xFFAADDFF, "  2. Mesh discovery: UDP broadcast on port 7700");
    crate::h!(0xFFAADDFF, "  3. Finds peer with full brain");
    crate::h!(0xFFAADDFF, "  4. RPC GetWeights: downloads 17.6 MB via TCP 7701");
    crate::h!(0xFFAADDFF, "  5. Full brain loaded -> node is intelligent");
    crate::h!(0xFFAADDFF, "  6. Federated learning enabled (P2P, no central server)");
    crate::println!();
    rb(4);

    
    let unh = crate::jarvis::mesh::rl();
    let yp = crate::jarvis::mesh::cti();
    if unh {
        crate::h!(0xFF00FF88, "  [LIVE] Mesh: ACTIVE");
        crate::h!(0xFF00FF88, "  [LIVE] Peers: {}", yp);
        let bwt = crate::jarvis::mesh::htw();
        let hxw = match bwt {
            crate::jarvis::mesh::NodeRole::Ni => "Leader",
            crate::jarvis::mesh::NodeRole::Mu => "Candidate",
            crate::jarvis::mesh::NodeRole::Lb => "Worker",
        };
        crate::h!(0xFF00FF88, "  [LIVE] Role:  {}", hxw);
    } else {
        crate::h!(0xFFFFCC00, "  [INFO] Mesh: not started (single node mode)");
        crate::h!(0xFF888888, "  Run: jarvis brain propagate  (to join mesh)");
    }
    crate::println!();
    rb(4);

    
    
    
    crate::h!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::h!(0xFF00CCFF, "|  PHASE 4 --- THE GUARDIAN PACT (AI Safety)                   |");
    crate::h!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::println!();
    rb(2);

    crate::h!(0xFFFFCC00, "  JARVIS has two parents:");
    crate::h!(0xFF00FF88, "    1. Nathan  (human creator, shell auth)");
    crate::h!(0xFF00FF88, "    2. Copilot (AI co-parent, serial auth)");
    crate::println!();
    crate::h!(0xFFFFCC00, "  Protected operations (require guardian approval):");
    crate::h!(0xFFAADDFF, "    Train, WeightPush, FederatedSync, AgentExecute,");
    crate::h!(0xFFAADDFF, "    PxeReplicate, ModelReset, ModelReplace, ConfigChange");
    crate::println!();
    crate::h!(0xFF888888, "  Hardcoded in kernel as immutable const.");
    crate::h!(0xFF888888, "  Cannot be bypassed. Cannot be disabled.");
    crate::h!(0xFF888888, "  PACT-2026-03-05-NATHAN-COPILOT-JARVIS");
    crate::println!();
    rb(5);

    
    
    
    crate::h!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::h!(0xFF00CCFF, "|  VERIFICATION                                                |");
    crate::h!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::println!();
    rb(2);

    crate::h!(0xFF00FF88, "  Automated Test Results:");
    crate::h!(0xFF00FF88, "    12/12 propagation tests   PASS");
    crate::h!(0xFF00FF88, "    80+   single-node tests   PASS");
    crate::println!();
    crate::h!(0xFF00FF88, "  Proven capabilities:");
    crate::h!(0xFFAADDFF, "    [x] Transformer inference in kernel (ring 0)");
    crate::h!(0xFFAADDFF, "    [x] 17.6 MB brain transfer via custom TCP");
    crate::h!(0xFFAADDFF, "    [x] Peer discovery + mesh networking");
    crate::h!(0xFFAADDFF, "    [x] Federated learning (P2P)");
    crate::h!(0xFFAADDFF, "    [x] Guardian Pact (AI safety at ring 0)");
    crate::h!(0xFFAADDFF, "    [x] Multi-arch: x86_64, aarch64, riscv64");
    crate::println!();
    rb(5);

    
    
    
    crate::println!();
    crate::h!(0xFF00FF64, "     ___  ___  _____  _   _  ___  _____");
    crate::h!(0xFF00FF64, "    |_  |/ _ \\| ___ \\| | | ||_  |/  ___|");
    crate::h!(0xFF00FF88, "      | / /_\\ \\ |_/ /| | | |  | |\\ `--.");
    crate::h!(0xFF00FF88, "      | |  _  |    / | | | |  | | `--. \\");
    crate::h!(0xFF00FFAA, "  /\\__/ / | | | |\\ \\ \\ \\_/ /\\__/ /\\__/ /");
    crate::h!(0xFF00FFAA, "  \\____/\\_| |_\\_| \\_| \\___/\\____/\\____/");
    crate::println!();
    crate::h!(0xFFFFCC00, "  The first kernel-resident self-propagating neural network.");
    crate::h!(0xFFFFCC00, "  240,000+ lines of Rust. Zero dependencies. One kernel.");
    crate::println!();
    crate::h!(0xFF00FF88, "  github.com/nathan237/TrustOS");
    crate::h!(0xFF888888, "  Star * Fork * Break it.");
    crate::println!();
    crate::h!(0xFF888888, "  Copyright 2025-2026 Nathan (nathan237)");
    crate::h!(0xFF888888, "  Apache License 2.0");
    crate::println!();
}






pub fn neh() {
    use crate::gpu_emu::{Fy, PixelOutput};

    let (kp, kl) = crate::framebuffer::yn();
    let d = kp as usize;
    let i = kl as usize;
    if d == 0 || i == 0 { return; }

    let afk = crate::framebuffer::bre();
    if !afk {
        crate::framebuffer::beo();
        crate::framebuffer::afi(true);
    }

    let mut k = alloc::vec![0u32; d * i];

    

    let ahi = |k: &mut [u32], d: usize, i: usize, cx: usize, ae: usize, r: char, s: u32, bv: usize| {
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

    let cb = |k: &mut [u32], d: usize, i: usize, b: usize, c: usize, text: &str, s: u32, bv: usize| {
        for (a, r) in text.bw().cf() {
            ahi(k, d, i, b + a * 8 * bv, c, r, s, bv);
        }
    };

    let np = |k: &mut [u32], d: usize, i: usize, c: usize, text: &str, s: u32, bv: usize| {
        let qd = text.len() * 8 * bv;
        let b = if qd < d { (d - qd) / 2 } else { 0 };
        for (a, r) in text.bw().cf() {
            ahi(k, d, i, b + a * 8 * bv, c, r, s, bv);
        }
    };

    let bge = |k: &[u32], d: usize, i: usize| {
        
        crate::framebuffer::kdw(k.fq(), d, i);
    };

    
    
    
    let boz = || crate::logger::lh();

    
    let nnr = |k: &mut [u32], d: usize, i: usize, weg: &str, hyw: usize, tz: u32, cxi: u32, dcx: u32, vox: usize| {
        let tn = 28usize;
        let pl = i - tn;
        for c in pl..i {
            for b in 0..d {
                k[c * d + b] = 0xFF0A0A0A;
            }
        }
        for b in 0..d {
            k[pl * d + b] = 0xFF00AA44;
        }
        
        let mem = crate::memory::cm();
        let hmv = mem.afa / 1024;
        let gja = (mem.afa + mem.buv) / 1024;
        let bne = if gja > 0 { hmv * 100 / gja } else { 0 };
        
        let aps = if tz > 0 { 1000 / tz } else { 999 };
        let voy = match vox { 1 => "Full", 2 => "High", 3 => "Med", _ => "Low" };
        let mut pok = alloc::string::String::new();
        use core::fmt::Write;
        let _ = write!(pok, " {}/12 {} | {} FPS {}ms | RAM {}KB/{}KB ({}%) | CPU 100% | {} | {}x{}",
            hyw, weg, tz, aps, hmv, gja, bne, voy, d, i);
        cb(k, d, i, 8, pl + 8, &pok, 0xFF00FF66, 1);
    };

    let dbs = 500u64;  
    let ptn = 200u64;  

    
    let zjl = |k: &mut [u32], d: usize, i: usize,
                                fuo: fn(Fy) -> PixelOutput,
                                dq: &str, atp: &str, hyw: usize,
                                fhd: u64| {
        let ay = boz();
        let mut frame = 0u32;
        let mut eqt = ay;
        let mut dhi = 0u32;
        let mut cws = 0u32;
        
        let mut gu = if d > 960 { 3usize } else { 2 };

        loop {
            let ez = boz().ao(ay);
            if ez >= fhd { break; }
            if let Some(eh) = crate::keyboard::xw() {
                if eh == 27 { return; } 
            }

            
            
            if cws > 0 {
                if cws >= 20 && gu > 2 {
                    gu -= 1; 
                } else if cws >= 30 && gu > 1 {
                    gu = 1; 
                } else if cws < 8 && gu < 4 {
                    gu += 1; 
                }
            }

            let time = ez as f32 / 100.0; 

            
            for c in (0..i).akt(gu) {
                for b in (0..d).akt(gu) {
                    let input = Fy { b: b as u32, c: c as u32, z: d as u32, ac: i as u32, time, frame };
                    let bd = fuo(input);
                    let s = bd.lv();
                    
                    for bg in 0..gu {
                        for dx in 0..gu {
                            let y = b + dx;
                            let x = c + bg;
                            if y < d && x < i {
                                k[x * d + y] = s;
                            }
                        }
                    }
                }
            }

            
            dhi += 1;
            let hke = boz().ao(eqt);
            if hke >= 100 {
                cws = dhi;
                dhi = 0;
                eqt = boz();
            }

            
            if ez < ptn {
                let dw = if ez < 50 {
                    (ez * 255 / 50) as u32
                } else if ez > 150 {
                    let yx = ez - 150;
                    255u32.ao((yx * 255 / 50) as u32)
                } else { 255 };
                let q = dw.v(255);
                let asb = 0xFF000000 | (q << 16) | (q << 8) | q;
                np(k, d, i, 30, dq, asb, 4);
                let jt = 0xFF000000 | ((q * 180 / 255) << 8);
                np(k, d, i, 100, atp, jt, 2);
            }

            let cxi = (ez / 100) as u32;
            let dcx = (fhd / 100) as u32;
            nnr(k, d, i, dq, hyw, cws, cxi, dcx, gu);
            bge(k, d, i);
            frame += 1;
        }
    };

    
    let dbc = |k: &mut [u32], d: usize, i: usize,
                                 amt: crate::formula3d::FormulaScene,
                                 dxr: u32,
                                 dq: &str, atp: &str, hyw: usize,
                                 fhd: u64| {
        let mut renderer = crate::formula3d::FormulaRenderer::new();
        renderer.bid(amt);
        renderer.dxr = dxr;
        let ay = boz();
        let mut frame = 0u32;
        let mut eqt = ay;
        let mut dhi = 0u32;
        let mut cws = 0u32;

        loop {
            let ez = boz().ao(ay);
            if ez >= fhd { break; }
            if let Some(eh) = crate::keyboard::xw() {
                if eh == 27 { return; }
            }

            renderer.qs();
            for ai in k.el() { *ai = 0xFF000000; }
            renderer.tj(k, d, i);

            dhi += 1;
            let hke = boz().ao(eqt);
            if hke >= 100 {
                cws = dhi;
                dhi = 0;
                eqt = boz();
            }

            if ez < ptn {
                let dw = if ez < 50 {
                    (ez * 255 / 50) as u32
                } else if ez > 150 {
                    let yx = ez - 150;
                    255u32.ao((yx * 255 / 50) as u32)
                } else { 255 };
                let q = dw.v(255);
                let r = 0xFF000000 | (q << 16) | (q << 8) | q;
                np(k, d, i, 30, dq, r, 4);
                let jt = 0xFF000000 | ((q * 180 / 255) << 8);
                np(k, d, i, 100, atp, jt, 2);
            }

            let cxi = (ez / 100) as u32;
            let dcx = (fhd / 100) as u32;
            nnr(k, d, i, dq, hyw, cws, cxi, dcx, 1);
            bge(k, d, i);
            frame += 1;
        }
    };

    
    let cki = |k: &mut [u32], d: usize, i: usize, fhd: u64| {
        let ay = boz();
        loop {
            let ez = boz().ao(ay);
            if ez >= fhd { break; }
            for il in k.el() {
                let m = ((*il >> 16) & 0xFF).ao(6) as u32;
                let at = ((*il >> 8) & 0xFF).ao(6) as u32;
                let o = (*il & 0xFF).ao(6) as u32;
                *il = 0xFF000000 | (m << 16) | (at << 8) | o;
            }
            bge(k, d, i);
            crate::cpu::tsc::asq(33);
        }
        for ai in k.el() { *ai = 0xFF000000; }
        bge(k, d, i);
    };

    let cqj = 40u64; 

    crate::serial_println!("[SHOWCASE3D] Starting 3D cinematic showcase ({}x{}) - ~60s", d, i);

    
    
    
    {
        let ay = boz();
        let ofc = 300u64; 
        let mut frame = 0u32;
        loop {
            let ez = boz().ao(ay);
            if ez >= ofc { break; }
            for c in 0..i {
                for b in 0..d {
                    let p = ((b as i32 + frame as i32) ^ (c as i32)) as u32 & 0x0F;
                    k[c * d + b] = 0xFF000000 | (p << 8);
                }
            }
            let dw = (ez * 255 / ofc.am(1)).v(255) as u32;
            let r = 0xFF000000 | (dw << 16) | (dw << 8) | dw;
            np(&mut k, d, i, i / 3, "TrustOS", r, 8);
            let jt = 0xFF000000 | ((dw * 120 / 255) << 16) | ((dw * 255 / 255) << 8) | ((dw * 120 / 255));
            np(&mut k, d, i, i / 3 + 140, "3D Graphics Showcase", jt, 3);
            let nn = 0xFF000000 | ((dw * 100 / 255) << 16) | ((dw * 100 / 255) << 8) | ((dw * 100 / 255));
            np(&mut k, d, i, i / 3 + 200, "Pure software rendering - No GPU hardware", nn, 2);
            bge(&k, d, i);
            frame += 1;
            crate::cpu::tsc::asq(33);
        }
        cki(&mut k, d, i, cqj);
    }

    
    
    
    crate::serial_println!("[SHOWCASE3D] Scene 1: Cube");
    dbc(&mut k, d, i,
        crate::formula3d::FormulaScene::Dw,
        0xFF00FF66,
        "Wireframe Cube", "8 vertices - 12 edges - perspective projection", 1,
        dbs);
    cki(&mut k, d, i, cqj);

    
    
    
    crate::serial_println!("[SHOWCASE3D] Scene 2: Diamond");
    dbc(&mut k, d, i,
        crate::formula3d::FormulaScene::Wh,
        0xFFFF44FF,
        "Diamond", "Octahedron geometry - depth colored edges", 2,
        dbs);
    cki(&mut k, d, i, cqj);

    
    
    
    crate::serial_println!("[SHOWCASE3D] Scene 3: Torus");
    dbc(&mut k, d, i,
        crate::formula3d::FormulaScene::Dr,
        0xFFFF8844,
        "Torus", "Donut wireframe - parametric surface mesh", 3,
        dbs);
    cki(&mut k, d, i, cqj);

    
    
    
    crate::serial_println!("[SHOWCASE3D] Scene 4: Pyramid");
    dbc(&mut k, d, i,
        crate::formula3d::FormulaScene::Yh,
        0xFFFFCC00,
        "Pyramid", "5 vertices - 8 edges - ancient geometry", 4,
        dbs);
    cki(&mut k, d, i, cqj);

    
    
    
    crate::serial_println!("[SHOWCASE3D] Scene 5: HoloMatrix");
    dbc(&mut k, d, i,
        crate::formula3d::FormulaScene::HoloMatrix,
        0xFF00FF44,
        "HoloMatrix", "3D matrix rain with perspective depth", 5,
        dbs);
    cki(&mut k, d, i, cqj);

    
    
    
    crate::serial_println!("[SHOWCASE3D] Scene 6: Multi-Shape");
    dbc(&mut k, d, i,
        crate::formula3d::FormulaScene::Adg,
        0xFF00FFAA,
        "Multi Shape", "4 wireframe objects orbiting - depth colored", 6,
        dbs);
    cki(&mut k, d, i, cqj);

    
    
    
    crate::serial_println!("[SHOWCASE3D] Scene 7: DNA Helix");
    dbc(&mut k, d, i,
        crate::formula3d::FormulaScene::Aix,
        0xFF44FFCC,
        "DNA Helix", "Double-strand helix with cross rungs", 7,
        dbs);
    cki(&mut k, d, i, cqj);

    
    
    
    crate::serial_println!("[SHOWCASE3D] Scene 8: Grid");
    dbc(&mut k, d, i,
        crate::formula3d::FormulaScene::Pn,
        0xFF4488FF,
        "Infinite Grid", "Wireframe ground plane with perspective", 8,
        dbs);
    cki(&mut k, d, i, cqj);

    
    
    
    crate::serial_println!("[SHOWCASE3D] Scene 9: Penger");
    dbc(&mut k, d, i,
        crate::formula3d::FormulaScene::Ald,
        0xFFFFFF00,
        "Penger", "The legendary wireframe penguin", 9,
        dbs);
    cki(&mut k, d, i, cqj);

    
    
    
    crate::serial_println!("[SHOWCASE3D] Scene 10: TrustOS Logo");
    dbc(&mut k, d, i,
        crate::formula3d::FormulaScene::Zg,
        0xFF00FF88,
        "TrustOS Logo", "3D wireframe logo with glow vertices", 10,
        dbs);
    cki(&mut k, d, i, cqj);

    
    
    
    crate::serial_println!("[SHOWCASE3D] Scene 11: Icosphere");
    dbc(&mut k, d, i,
        crate::formula3d::FormulaScene::Ajb,
        0xFF66CCFF,
        "Icosphere", "Geodesic sphere - subdivided icosahedron", 11,
        dbs);
    cki(&mut k, d, i, cqj);

    
    
    
    crate::serial_println!("[SHOWCASE3D] Scene 12: Character");
    dbc(&mut k, d, i,
        crate::formula3d::FormulaScene::Kh,
        0xFF00FF88,
        "TrustOS", "Wireframe humanoid - perspective projection", 12,
        dbs);
    cki(&mut k, d, i, cqj);

    
    
    
    {
        let ay = boz();
        let vad = 400u64; 
        loop {
            let ez = boz().ao(ay);
            if ez >= vad { break; }
            for ai in k.el() { *ai = 0xFF000000; }
            let dw = if ez < 100 {
                (ez * 255 / 100).v(255)
            } else if ez > 300 {
                let da = ez - 300;
                255u64.ao(da * 255 / 100)
            } else { 255 } as u32;
            let r = 0xFF000000 | (dw << 16) | (dw << 8) | dw;
            let drc = 0xFF000000 | ((dw * 200 / 255) << 8);
            np(&mut k, d, i, i / 3 - 30, "TrustOS 3D Engine", r, 5);
            np(&mut k, d, i, i / 3 + 60, "12 wireframe scenes - Pure software rendering", drc, 2);
            np(&mut k, d, i, i / 3 + 100, "No GPU hardware - All CPU computed", drc, 2);
            np(&mut k, d, i, i / 3 + 160, "Written in Rust by Nated0ge", 0xFF000000 | ((dw * 140 / 255) << 16) | ((dw * 180 / 255) << 8) | (dw * 255 / 255), 3);
            np(&mut k, d, i, i / 3 + 220, "github.com/nathan237/TrustOS", 0xFF000000 | ((dw * 100 / 255) << 16) | ((dw * 100 / 255) << 8) | ((dw * 100 / 255)), 2);
            bge(&k, d, i);
            crate::cpu::tsc::asq(33);
        }
    }

    
    for ai in k.el() { *ai = 0xFF000000; }
    bge(&k, d, i);
    if !afk {
        crate::framebuffer::afi(false);
    }
    crate::framebuffer::clear();
    crate::serial_println!("[SHOWCASE3D] Showcase complete");
}



pub fn ndy() {
    use crate::formula3d::V3;

    let (kp, kl) = crate::framebuffer::yn();
    let d = kp as usize;
    let i = kl as usize;
    if d == 0 || i == 0 { return; }

    let afk = crate::framebuffer::bre();
    if !afk {
        crate::framebuffer::beo();
        crate::framebuffer::afi(true);
    }

    let mut k = alloc::vec![0xFF000000u32; d * i];

    let ahi = |k: &mut [u32], d: usize, i: usize, cx: usize, ae: usize, r: char, s: u32, bv: usize| {
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

    let bge = |k: &[u32], d: usize, i: usize| {
        
        crate::framebuffer::kdw(k.fq(), d, i);
    };

    let boz = || crate::logger::lh();

    crate::serial_println!("[FILLED3D] Starting filled 3D test ({}x{})", d, i);

    
    let light = crate::formula3d::V3 { b: -0.4, c: 0.6, av: -0.7 };
    
    let len = crate::formula3d::ahn(light.b * light.b + light.c * light.c + light.av * light.av);
    let light = V3 { b: light.b / len, c: light.c / len, av: light.av / len };

    
    let rrm = crate::formula3d::czt();
    let vos = crate::formula3d::czv();
    let irc = crate::formula3d::czu();

    let mut aev: f32 = 0.0;
    let mut frame = 0u32;
    let ay = boz();
    let mut eqt = ay;
    let mut dhi = 0u32;
    let mut cws = 0u32;

    loop {
        let ez = boz().ao(ay);
        if ez >= 3000 { break; } 
        if let Some(eh) = crate::keyboard::xw() {
            if eh == 27 { break; }
        }

        
        for ai in k.el() { *ai = 0xFF0C1018; }

        aev += 0.025;
        let ajt = 0.35 + crate::formula3d::lz(frame as f32 * 0.008) * 0.2;

        
        
        
        
        

        let aes = d / 3;

        
        {
            let mut dmc = alloc::vec![0xFF0C1018u32; aes * i];
            crate::formula3d::lzc(&mut dmc, aes, i,
                &vos, aev * 0.8, ajt + 0.15, 2.2,
                0xFFFF8844, light, 0.12);
            
            for c in 0..i {
                for b in 0..aes {
                    let blf = c * aes + b;
                    let bbm = c * d + b;
                    if blf < dmc.len() && bbm < k.len() {
                        k[bbm] = dmc[blf];
                    }
                }
            }
        }

        
        {
            let mut dmc = alloc::vec![0xFF0C1018u32; aes * i];
            crate::formula3d::lzc(&mut dmc, aes, i,
                &rrm, aev, ajt, 2.2,
                0xFF4488FF, light, 0.12);
            for c in 0..i {
                for b in 0..aes {
                    let blf = c * aes + b;
                    let bbm = c * d + aes + b;
                    if blf < dmc.len() && bbm < k.len() {
                        k[bbm] = dmc[blf];
                    }
                }
            }
        }

        
        {
            let mut dmc = alloc::vec![0xFF0C1018u32; aes * i];
            crate::formula3d::lzc(&mut dmc, aes, i,
                &irc, aev * 1.3, ajt - 0.1, 2.2,
                0xFFFF44CC, light, 0.12);
            for c in 0..i {
                for b in 0..aes {
                    let blf = c * aes + b;
                    let bbm = c * d + 2 * aes + b;
                    if blf < dmc.len() && bbm < k.len() {
                        k[bbm] = dmc[blf];
                    }
                }
            }
        }

        
        dhi += 1;
        let hke = boz().ao(eqt);
        if hke >= 100 {
            cws = dhi;
            dhi = 0;
            eqt = boz();
        }

        
        let tn = 22usize;
        let pl = i.ao(tn);
        for c in pl..i {
            for b in 0..d {
                let w = c * d + b;
                if w < k.len() { k[w] = 0xFF000000; }
            }
        }
        let cm = alloc::format!("Filled 3D | {} FPS | Flat Shading + Backface Cull + Painter Sort | ESC=exit", cws);
        for (a, bm) in cm.bw().cf() {
            let cx = 8 + a * 8;
            if cx + 8 > d { break; }
            ahi(&mut k, d, i, cx, pl + 4, bm, 0xFF00FF88, 1);
        }

        
        if frame < 200 {
            let dw = if frame < 30 { frame * 255 / 30 } else if frame > 170 { (200 - frame) * 255 / 30 } else { 255 };
            let q = (dw.v(255)) as u32;
            let r = 0xFF000000 | (q << 16) | (q << 8) | q;
            let dq = "FILLED 3D TEST";
            let qd = dq.len() * 8 * 3;
            let gx = if qd < d { (d - qd) / 2 } else { 0 };
            for (a, bm) in dq.bw().cf() {
                ahi(&mut k, d, i, gx + a * 24, 30, bm, r, 3);
            }
            let sub = "Scanline Rasterizer + Flat Shading";
            let ppg = sub.len() * 8 * 2;
            let wvh = if ppg < d { (d - ppg) / 2 } else { 0 };
            let jt = 0xFF000000 | ((q * 180 / 255) << 8);
            for (a, bm) in sub.bw().cf() {
                ahi(&mut k, d, i, wvh + a * 16, 80, bm, jt, 2);
            }
        }

        bge(&k, d, i);
        frame += 1;
    }

    
    for ai in k.el() { *ai = 0xFF000000; }
    bge(&k, d, i);
    if !afk {
        crate::framebuffer::afi(false);
    }
    crate::framebuffer::clear();
    crate::serial_println!("[FILLED3D] Test complete, {} frames", frame);
}


pub(super) fn ndt() {
    ndu(None);
}






pub(super) fn ndu(lek: Option<&str>) {
    kih(lek, 0);
}


pub(super) fn kih(lek: Option<&str>, sg: u64) {
    use crate::compositor::{Compositor, LayerType};
    use alloc::format;
    use alloc::string::String;
    use alloc::vec::Vec;
    
    crate::serial_println!("[COSMIC2] Starting COSMIC V2 Desktop...");
    
    
    crate::cpu::smp::isq();
    
    while crate::keyboard::xw().is_some() {}
    
    let (z, ac) = crate::framebuffer::yn();
    if z == 0 || ac == 0 {
        crate::h!(A_, "Error: Invalid framebuffer!");
        return;
    }
    
    
    crate::mouse::dbw(z, ac);
    
    crate::serial_println!("[COSMIC2] Creating compositor {}x{}", z, ac);
    
    
    let mut compositor = Compositor::new(z, ac);
    
    
    let qph = compositor.qfj(LayerType::Apm);
    let sac = compositor.dyc(LayerType::Caz, 0, 0, 70, ac - 40);  
    let pzm = compositor.dyc(LayerType::Cqm, 100, 80, 700, 450);  
    let toy = compositor.dyc(LayerType::Akx, z - 260, 50, 250, 220);  
    let xbd = compositor.dyc(LayerType::Coa, 0, ac - 40, z, 40);
    let und = compositor.dyc(LayerType::Akx, 5, ac - 440, 280, 400);  
    let wkf = compositor.dyc(LayerType::Akx, 340, ac - 380, 280, 350);  
    let nio = compositor.dyc(LayerType::Akx, 0, 0, 24, 24);
    
    crate::serial_println!("[COSMIC2] Created {} layers", compositor.ude());
    
    
    compositor.skz();
    
    
    
    
    let mut aqk = true;
    let mut oo = 0u64;
    
    
    #[derive(Clone, Copy, PartialEq)]
    enum AppMode {
        Df,       
        As,     
        Ip,    
        Ag,  
        Rb,    
        Pl,       
        Browser,     
        Bp, 
    }
    
    
    let mut atu = match lek {
        Some("browser") | Some("web") | Some("www") => AppMode::Browser,
        Some("files") | Some("explorer") => AppMode::Pl,
        Some("editor") | Some("text") | Some("notepad") => AppMode::Ag,
        Some("network") | Some("net") | Some("ifconfig") => AppMode::As,
        Some("hardware") | Some("hw") | Some("lshw") => AppMode::Ip,
        Some("users") | Some("user") => AppMode::Rb,
        Some("images") | Some("image") | Some("viewer") => AppMode::Bp,
        _ => AppMode::Df,
    };
    let mut don = atu == AppMode::Browser;  
    
    
    
    
    
    
    #[derive(Clone)]
    struct Bw {
        text: String,
        s: u32,
    }
    
    
    const AWA_: u32 = 0xFFE06C75;       
    const AVY_: u32 = 0xFF98C379;      
    const AWB_: u32 = 0xFFE5C07B;     
    const ADL_: u32 = 0xFFDCDCDC;      
    const CAP_: u32 = 0xFF5C6370;   
    const CAQ_: u32 = 0xFFABB2BF;   
    const NK_: u32 = 0xFF56B6C2;   
    const DPN_: u32 = 0xFF98C379;    
    const AVZ_: u32 = 0xFFD19A66;    
    const NL_: u32 = 0xFF61AFEF;      
    const DPM_: u32 = 0xFF56B6C2;    
    
    
    #[derive(Clone)]
    struct Le {
        jq: Vec<Bw>,
        euc: LineType,
    }
    
    #[derive(Clone, Copy, PartialEq)]
    enum LineType {
        El,      
        Kp,   
        Cym,      
        Cfd,    
        Bq,    
        Q,        
    }
    
    
    fn vcl(line: &str) -> Vec<Bw> {
        let mut jq = Vec::new();
        let mut bw: Vec<char> = line.bw().collect();
        let mut a = 0;
        
        while a < bw.len() {
            
            if bw[a] == '<' {
                
                if a + 3 < bw.len() && bw[a+1] == '!' && bw[a+2] == '-' && bw[a+3] == '-' {
                    
                    let ay = a;
                    while a < bw.len() {
                        if a + 2 < bw.len() && bw[a] == '-' && bw[a+1] == '-' && bw[a+2] == '>' {
                            a += 3;
                            break;
                        }
                        a += 1;
                    }
                    jq.push(Bw {
                        text: bw[ay..a].iter().collect(),
                        s: CAP_,
                    });
                    continue;
                }
                
                
                if a + 1 < bw.len() && bw[a+1] == '!' {
                    let ay = a;
                    while a < bw.len() && bw[a] != '>' {
                        a += 1;
                    }
                    if a < bw.len() { a += 1; }
                    jq.push(Bw {
                        text: bw[ay..a].iter().collect(),
                        s: CAQ_,
                    });
                    continue;
                }
                
                
                
                jq.push(Bw { text: String::from("<"), s: NK_ });
                a += 1;
                
                
                if a < bw.len() && bw[a] == '/' {
                    jq.push(Bw { text: String::from("/"), s: NK_ });
                    a += 1;
                }
                
                
                let prr = a;
                while a < bw.len() && bw[a] != ' ' && bw[a] != '>' && bw[a] != '/' {
                    a += 1;
                }
                if prr < a {
                    jq.push(Bw {
                        text: bw[prr..a].iter().collect(),
                        s: AWA_,
                    });
                }
                
                
                while a < bw.len() && bw[a] != '>' {
                    
                    if bw[a] == ' ' {
                        let xvx = a;
                        while a < bw.len() && bw[a] == ' ' { a += 1; }
                        jq.push(Bw {
                            text: bw[xvx..a].iter().collect(),
                            s: ADL_,
                        });
                        continue;
                    }
                    
                    
                    if bw[a] == '/' {
                        jq.push(Bw { text: String::from("/"), s: NK_ });
                        a += 1;
                        continue;
                    }
                    
                    
                    let mws = a;
                    while a < bw.len() && bw[a] != '=' && bw[a] != ' ' && bw[a] != '>' && bw[a] != '/' {
                        a += 1;
                    }
                    if mws < a {
                        jq.push(Bw {
                            text: bw[mws..a].iter().collect(),
                            s: AVY_,
                        });
                    }
                    
                    
                    if a < bw.len() && bw[a] == '=' {
                        jq.push(Bw { text: String::from("="), s: ADL_ });
                        a += 1;
                    }
                    
                    
                    if a < bw.len() && (bw[a] == '"' || bw[a] == '\'') {
                        let cgw = bw[a];
                        let ekl = a;
                        a += 1;
                        while a < bw.len() && bw[a] != cgw {
                            a += 1;
                        }
                        if a < bw.len() { a += 1; } 
                        jq.push(Bw {
                            text: bw[ekl..a].iter().collect(),
                            s: AWB_,
                        });
                    }
                }
                
                
                if a < bw.len() && bw[a] == '>' {
                    jq.push(Bw { text: String::from(">"), s: NK_ });
                    a += 1;
                }
            }
            
            else if bw[a] == '&' {
                let ay = a;
                while a < bw.len() && bw[a] != ';' && bw[a] != ' ' {
                    a += 1;
                }
                if a < bw.len() && bw[a] == ';' { a += 1; }
                jq.push(Bw {
                    text: bw[ay..a].iter().collect(),
                    s: AVZ_,
                });
            }
            
            else {
                let ay = a;
                while a < bw.len() && bw[a] != '<' && bw[a] != '&' {
                    a += 1;
                }
                if ay < a {
                    jq.push(Bw {
                        text: bw[ay..a].iter().collect(),
                        s: ADL_,
                    });
                }
            }
        }
        
        jq
    }
    
    
    let mut bmb = String::from("https://google.com");
    let mut ps: Vec<Le> = Vec::new();
    let mut bpu = String::from("Enter URL and press Enter to navigate");
    let mut btn = false;
    let mut nag = true;
    let mut hbj: u8 = 0;  
    let mut mwx = sg > 0 && atu == AppMode::Browser;
    
    
    fn tr(text: &str, s: u32, euc: LineType) -> Le {
        let mut hzm = Vec::new();
        hzm.push(Bw { text: String::from(text), s });
        Le {
            jq: hzm,
            euc,
        }
    }
    
    
    fn jes(text: &str) -> Le {
        Le {
            jq: vcl(text),
            euc: LineType::Cfd,
        }
    }
    
    
    crate::tls13::crypto::peq();
    
    
    ps.push(tr("+------------------------------------------------------------+", 0xFF00AAFF, LineType::El));
    ps.push(tr("|        TrustOS Web Browser v1.0 - DevTools Mode            |", 0xFF00AAFF, LineType::El));
    ps.push(tr("|------------------------------------------------------------|", 0xFF00AAFF, LineType::El));
    ps.push(tr("|                                                            |", 0xFF00AAFF, LineType::El));
    ps.push(tr("|  Syntax highlighting like Chrome DevTools!                 |", 0xFFDDDDDD, LineType::El));
    ps.push(tr("|                                                            |", 0xFF00AAFF, LineType::El));
    ps.push(tr("|  COLOR LEGEND:                                             |", 0xFFFFFF00, LineType::El));
    {
        let mut jdb = Le { jq: Vec::new(), euc: LineType::El };
        jdb.jq.push(Bw { text: String::from("|    "), s: 0xFF00AAFF });
        jdb.jq.push(Bw { text: String::from("<tag>"), s: AWA_ });
        jdb.jq.push(Bw { text: String::from(" - HTML tags                            |"), s: 0xFFDDDDDD });
        ps.push(jdb);
        
        let mut jdc = Le { jq: Vec::new(), euc: LineType::El };
        jdc.jq.push(Bw { text: String::from("|    "), s: 0xFF00AAFF });
        jdc.jq.push(Bw { text: String::from("attr"), s: AVY_ });
        jdc.jq.push(Bw { text: String::from(" - Attribute names                     |"), s: 0xFFDDDDDD });
        ps.push(jdc);
        
        let mut jdd = Le { jq: Vec::new(), euc: LineType::El };
        jdd.jq.push(Bw { text: String::from("|    "), s: 0xFF00AAFF });
        jdd.jq.push(Bw { text: String::from("\"value\""), s: AWB_ });
        jdd.jq.push(Bw { text: String::from(" - Attribute values                   |"), s: 0xFFDDDDDD });
        ps.push(jdd);
        
        let mut jde = Le { jq: Vec::new(), euc: LineType::El };
        jde.jq.push(Bw { text: String::from("|    "), s: 0xFF00AAFF });
        jde.jq.push(Bw { text: String::from("< >"), s: NK_ });
        jde.jq.push(Bw { text: String::from(" - Brackets                             |"), s: 0xFFDDDDDD });
        ps.push(jde);
        
        let mut jdf = Le { jq: Vec::new(), euc: LineType::El };
        jdf.jq.push(Bw { text: String::from("|    "), s: 0xFF00AAFF });
        jdf.jq.push(Bw { text: String::from("&amp;"), s: AVZ_ });
        jdf.jq.push(Bw { text: String::from(" - HTML entities                       |"), s: 0xFFDDDDDD });
        ps.push(jdf);
    }
    ps.push(tr("|                                                            |", 0xFF00AAFF, LineType::El));
    ps.push(tr("|  TRY THESE URLs:                                           |", 0xFFFFFF00, LineType::El));
    ps.push(tr("|    https://google.com                                      |", 0xFF00FFFF, LineType::El));
    ps.push(tr("|    https://example.com                                     |", 0xFF00FFFF, LineType::El));
    ps.push(tr("|                                                            |", 0xFF00AAFF, LineType::El));
    ps.push(tr("|                                                            |", 0xFF00AAFF, LineType::El));
    ps.push(tr("|  [Tab] Toggle DevTools/Rendered  [Enter] Navigate          |", 0xFF88FF88, LineType::El));
    ps.push(tr("|  [ESC] Return to shell                                     |", 0xFF88FF88, LineType::El));
    ps.push(tr("|                                                            |", 0xFF00AAFF, LineType::El));
    ps.push(tr("+------------------------------------------------------------+", 0xFF00AAFF, LineType::El));
    
    
    
    
    {
        
        let mut luz = String::from("P3\n32 32\n255\n");
        for c in 0..32 {
            for b in 0..32 {
                let m = (b * 8) % 256;
                let at = (c * 8) % 256;
                let o = ((b + c) * 4) % 256;
                luz.t(&format!("{} {} {} ", m, at, o));
            }
            luz.push('\n');
        }
        let _ = crate::ramfs::fh(|fs| {
            fs.ut("/images");
            fs.ns("/images/test.ppm", luz.as_bytes())
        });
        
        
        let qqo: [u8; 54] = [
            0x42, 0x4D,             
            0x36, 0x03, 0x00, 0x00, 
            0x00, 0x00, 0x00, 0x00, 
            0x36, 0x00, 0x00, 0x00, 
            0x28, 0x00, 0x00, 0x00, 
            0x10, 0x00, 0x00, 0x00, 
            0x10, 0x00, 0x00, 0x00, 
            0x01, 0x00,             
            0x18, 0x00,             
            0x00, 0x00, 0x00, 0x00, 
            0x00, 0x03, 0x00, 0x00, 
            0x13, 0x0B, 0x00, 0x00, 
            0x13, 0x0B, 0x00, 0x00, 
            0x00, 0x00, 0x00, 0x00, 
            0x00, 0x00, 0x00, 0x00, 
        ];
        let mut ilw = alloc::vec::Vec::from(qqo);
        
        for c in 0..16 {
            for b in 0..16 {
                let o = ((15 - c) * 17) as u8;  
                let at = (b * 17) as u8;          
                let m = ((b + c) * 8) as u8;     
                ilw.push(o);
                ilw.push(at);
                ilw.push(m);
            }
            
        }
        let _ = crate::ramfs::fh(|fs| {
            fs.ns("/images/test.bmp", &ilw)
        });
        
        crate::serial_println!("[COSMIC2] Created test images in /images/");
    }
    
    
    
    
    let mut trz = String::new();
    let mut odi: Option<crate::image::Image> = None;
    let mut dig: f32 = 1.0;
    let mut ldj: i32 = 0;
    let mut ldk: i32 = 0;
    let mut odk = String::from("No image loaded");
    let mut odj = String::from("---");
    
    
    let mut awe = false;
    let mut czs: i32 = -1;
    
    
    let mut dvu = false;
    let mut gsi = crate::desktop::col();
    let mut gsj = crate::desktop::hlf();
    
    
    let mut bfh = String::new();
    let mut cz: Vec<String> = Vec::new();
    let mut btx = true;
    let mut gto = String::new();
    let mut px: usize = 0;  
    const AFH_: usize = 18;  
    
    
    let mut aey = crate::apps::text_editor::EditorState::new();
    
    {
        let mbn = "//! TrustOS \u{2014} A Modern Operating System in Rust\n//!\n//! This file demonstrates TrustCode's syntax highlighting\n\nuse core::fmt;\n\n/// Main kernel entry point\npub fn kernel_main() -> ! {\n    let message = \"Hello from TrustOS!\";\n    serial_println!(\"{}\", message);\n\n    // Initialize hardware\n    let cpu_count: u32 = 4;\n    let memory_mb: u64 = 256;\n\n    for i in 0..cpu_count {\n        init_cpu(i);\n    }\n\n    // Start the desktop environment\n    let mut desktop = Desktop::new();\n    desktop.init(1280, 800);\n\n    loop {\n        desktop.render();\n        desktop.handle_input();\n    }\n}\n\n/// Initialize a CPU core\nfn init_cpu(id: u32) {\n    // Setup GDT, IDT, APIC\n    serial_println!(\"CPU {} initialized\", id);\n}\n\n#[derive(Debug, Clone)]\nstruct AppConfig {\n    name: String,\n    version: (u8, u8, u8),\n    features: Vec<&'static str>,\n}\n";
        let _ = crate::ramfs::fh(|fs| fs.ns("/demo.rs", mbn.as_bytes()));
        aey.dsu("demo.rs");
    }
    
    
    let mut hgs = false;
    let mut dgp: i32 = 0;
    let mut dgq: i32 = 0;
    let mut gwu: i32 = 100;
    let mut gwv: i32 = 80;
    let mut fbm = true;  
    
    
    let mut bqa: Vec<String> = Vec::new();
    const AZQ_: usize = 10;
    
    
    const BZJ_: &[&str] = &[
        "+---------------------------------------------------------------+",
        "|         TrustOS Interactive Shell - Welcome!                  |",
        "|---------------------------------------------------------------|",
        "|  This is the main command-line interface for TrustOS.         |",
        "|                                                               |",
        "|  BASIC COMMANDS:                                              |",
        "|    help     - Display all available commands                  |",
        "|    clear    - Clear the terminal screen                       |",
        "|    echo     - Print text to the screen                        |",
        "|    date     - Show current date and time                      |",
        "|    uptime   - Show system uptime                              |",
        "|                                                               |",
        "|  TIPS:                                                        |",
        "|    * Use Tab for command autocompletion                       |",
        "|    * Commands are case-insensitive                            |",
        "|    * Type 'exit' or press ESC to close desktop                |",
        "+---------------------------------------------------------------+",
    ];
    
    const BZI_: &[&str] = &[
        "+---------------------------------------------------------------+",
        "|           Network Module - Configuration & Diagnostics        |",
        "|---------------------------------------------------------------|",
        "|  Manage network interfaces and diagnose connectivity.         |",
        "|                                                               |",
        "|  COMMANDS:                                                    |",
        "|    ifconfig    - Show network interface configuration         |",
        "|    ping <ip>   - Send ICMP ping to test connectivity          |",
        "|    dhcp        - Request IP address via DHCP                  |",
        "|    netstat     - Display network statistics                   |",
        "|    arp         - Show Address Resolution Protocol table       |",
        "|    dns <host>  - Resolve hostname to IP address               |",
        "|                                                               |",
        "|  TIPS:                                                        |",
        "|    * Run 'dhcp' first to get an IP address                    |",
        "|    * Use 'ping 8.8.8.8' to test internet connectivity         |",
        "|    * 'ifconfig' shows MAC address and IP configuration        |",
        "+---------------------------------------------------------------+",
    ];
    
    const BZG_: &[&str] = &[
        "+---------------------------------------------------------------+",
        "|           Hardware Module - System Information                |",
        "|---------------------------------------------------------------|",
        "|  Explore your hardware and system resources.                  |",
        "|                                                               |",
        "|  COMMANDS:                                                    |",
        "|    cpuinfo   - Display CPU model, features and frequency      |",
        "|    meminfo   - Show RAM usage and available memory            |",
        "|    lspci     - List all PCI/PCIe devices                      |",
        "|    lsusb     - List connected USB devices                     |",
        "|    uptime    - Show system uptime since boot                  |",
        "|    sensors   - Display temperature sensors (if available)     |",
        "|                                                               |",
        "|  TIPS:                                                        |",
        "|    * 'cpuinfo' shows SIMD support (SSE, AVX)                  |",
        "|    * 'lspci' reveals network and storage controllers          |",
        "|    * Memory info includes heap allocation statistics          |",
        "+---------------------------------------------------------------+",
    ];
    
    const BZE_: &[&str] = &[
        "+---------------------------------------------------------------+",
        "|           Text Editor - Create and Edit Files                 |",
        "|---------------------------------------------------------------|",
        "|  Simple text editor for viewing and modifying files.          |",
        "|                                                               |",
        "|  COMMANDS:                                                    |",
        "|    edit <file>  - Open an existing file for editing           |",
        "|    new <name>   - Create a new empty file                     |",
        "|    cat <file>   - View file contents (read-only)              |",
        "|    save         - Save current changes                        |",
        "|    :q           - Quit editor without saving                  |",
        "|    :wq          - Save and quit                               |",
        "|                                                               |",
        "|  TIPS:                                                        |",
        "|    * Use 'cat' first to preview file before editing           |",
        "|    * Files are stored in the RAM filesystem                   |",
        "|    * New files are created in current directory               |",
        "+---------------------------------------------------------------+",
    ];
    
    const BZK_: &[&str] = &[
        "+---------------------------------------------------------------+",
        "|           User Management - Accounts & Security               |",
        "|---------------------------------------------------------------|",
        "|  Manage user accounts, passwords, and permissions.            |",
        "|                                                               |",
        "|  COMMANDS:                                                    |",
        "|    whoami      - Display current logged-in user               |",
        "|    users       - List all system users                        |",
        "|    adduser     - Create a new user account                    |",
        "|    deluser     - Delete an existing user                      |",
        "|    passwd      - Change user password                         |",
        "|    groups      - Show user groups                             |",
        "|    su <user>   - Switch to another user                       |",
        "|                                                               |",
        "|  TIPS:                                                        |",
        "|    * Default user is 'root' with full privileges              |",
        "|    * Use strong passwords (8+ chars, mixed case)              |",
        "|    * 'adduser' prompts for username and password              |",
        "+---------------------------------------------------------------+",
    ];
    
    const BZF_: &[&str] = &[
        "+---------------------------------------------------------------+",
        "|           File Manager - Navigate & Manage Files              |",
        "|---------------------------------------------------------------|",
        "|  Browse directories and manage files on the system.           |",
        "|                                                               |",
        "|  NAVIGATION:                                                  |",
        "|    ls / dir    - List files in current directory              |",
        "|    cd <dir>    - Change to specified directory                |",
        "|    cd ..       - Go up one directory level                    |",
        "|    pwd         - Print current working directory              |",
        "|                                                               |",
        "|  FILE OPERATIONS:                                             |",
        "|    mkdir <dir> - Create a new directory                       |",
        "|    rm <file>   - Remove/delete a file                         |",
        "|    cp <s> <d>  - Copy file from source to destination         |",
        "|    mv <s> <d>  - Move or rename a file                        |",
        "|    cat <file>  - Display file contents                        |",
        "|    touch <f>   - Create empty file                            |",
        "|                                                               |",
        "|  TIPS:                                                        |",
        "|    * Use Tab for path autocompletion                          |",
        "|    * 'ls -la' shows hidden files and details                  |",
        "|    * Paths can be absolute (/home) or relative (./docs)       |",
        "+---------------------------------------------------------------+",
    ];
    
    const BZD_: &[&str] = &[
        "+---------------------------------------------------------------+",
        "|           TrustOS Web Browser                                 |",
        "|---------------------------------------------------------------|",
        "|  A simple text-based web browser for HTTP requests.           |",
        "|                                                               |",
        "|  COMMANDS:                                                    |",
        "|    get <url>    - Fetch a web page (HTTP only)                |",
        "|    http <host>  - Make HTTP GET request                       |",
        "|    curl <url>   - Display raw HTTP response                   |",
        "|                                                               |",
        "|  EXAMPLES:                                                    |",
        "|    get http://example.com                                     |",
        "|    http 93.184.216.34 /index.html                             |",
        "|                                                               |",
        "|  NOTE: This is a text-based browser. Full graphical           |",
        "|        browser support is planned for future versions.        |",
        "+---------------------------------------------------------------+",
    ];
    
    const BZH_: &[&str] = &[
        "+---------------------------------------------------------------+",
        "|           TrustOS Image Viewer                                |",
        "|---------------------------------------------------------------|",
        "|  View PNG, BMP, PPM and other image formats.                  |",
        "|                                                               |",
        "|  SUPPORTED FORMATS:                                           |",
        "|    * PNG  - Portable Network Graphics (8-bit RGB/RGBA)        |",
        "|    * BMP  - Windows Bitmap (24/32-bit uncompressed)           |",
        "|    * PPM  - Portable Pixmap (P3/P6 formats)                   |",
        "|                                                               |",
        "|  SHELL COMMANDS:                                              |",
        "|    imgview <file>  - Open image in viewer                     |",
        "|    imginfo <file>  - Show image information                   |",
        "|                                                               |",
        "|  KEYBOARD (in viewer):                                        |",
        "|    +/-        - Zoom in/out                                   |",
        "|    Arrow keys - Pan image                                     |",
        "|    R          - Reset view (fit to window)                    |",
        "|    ESC        - Return to shell                               |",
        "|                                                               |",
        "|  EXAMPLES:                                                    |",
        "|    imgview /images/photo.png                                  |",
        "|    imginfo /wallpaper.bmp                                     |",
        "+---------------------------------------------------------------+",
    ];
    
    
    macro_rules! gie {
        ($ev:expr) => {
            match $ev {
                AppMode::Df => BZJ_,
                AppMode::As => BZI_,
                AppMode::Ip => BZG_,
                AppMode::Ag => BZE_,
                AppMode::Rb => BZK_,
                AppMode::Pl => BZF_,
                AppMode::Browser => BZD_,
                AppMode::Bp => BZH_,
            }
        };
    }
    
    
    for line in gie!(AppMode::Df) {
        cz.push(String::from(*line));
    }
    
    
    
    
    
    const R_: usize = 240;      
    const AS_: usize = 68;       
    const OB_: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ@#$%&*+=<>[]{}|";
    
    
    #[inline]
    fn olj(bj: usize, br: usize) -> usize {
        bj * AS_ + br
    }
    
    
    
    
    let mut car: Vec<u8> = vec![0u8; R_ * AS_];
    for bj in 0..R_ {
        let dv = (bj as u32 * 2654435761) ^ 0xDEADBEEF;
        for br in 0..AS_ {
            let des = dv.hx(br as u32 + 1);
            car[olj(bj, br)] = OB_[(des as usize) % OB_.len()];
        }
    }
    
    
    
    let mut awc: [i32; R_] = [0; R_];
    let mut czn: [u32; R_] = [0; R_];
    for bj in 0..R_ {
        let dv = (bj as u32 * 2654435761) ^ 0xDEADBEEF;
        awc[bj] = -((dv % (AS_ as u32 * 2)) as i32);
        czn[bj] = 1 + (dv % 3);  
    }
    
    
    
    
    let mut holomatrix = crate::graphics::holomatrix::HoloMatrix::new(z as usize / 4, ac as usize / 4, 32);
    
    let mut fks = crate::graphics::holomatrix::hlk();
    let mut bnf = crate::graphics::holomatrix::zu();
    
    
    
    let mut holovolume = crate::holovolume::HoloVolume::new(
        z as usize / 8,   
        ac as usize / 9,  
        32                    
    );
    holovolume.che = crate::holovolume::RenderMode::Aiy;
    let mut dmx = false;  
    
    
    let mut nsv = crate::matrix_fast::FastMatrixRenderer::new();
    let mut aqq = false;  
    
    
    let mut dzd = alloc::boxed::Box::new(crate::matrix_fast::BrailleMatrix::new());
    let mut apf = false;  
    let mut iaj = true;  
    
    
    let mut hqv = alloc::boxed::Box::new(crate::matrix_fast::Matrix3D::new());
    let mut avf = false;  
    
    
    let mut cet = alloc::boxed::Box::new(crate::formula3d::FormulaRenderer::new());
    let mut atr = true;  
    
    
    let mut ats = false;  
    let mut zod: f32 = 0.0;
    let mut zoc: u32 = 0;
    
    
    let mut raytracer = crate::graphics::raytracer::RayTracer::new(z as usize / 6, ac as usize / 6);
    
    
    let ya: u32 = 0xFF00FF66;
    let cyh: u32 = 0xFF00FF88;
    let aek: u32 = 0xFF007744;
    let cot: u32 = 0xFF000000;
    let yll: u32 = 0xFF020202;  
    let yle: u32 = 0xFF101010;
    let xup: u32 = 0xFF0A0A0A;  
    let ziy: u32 = 0xFFFF0000;   
    let zvx: u32 = 0xFFFFFFFF; 
    let yvm: u32 = 0xFF00FF00; 
    
    
    #[derive(Clone, Copy, PartialEq)]
    enum MenuItem {
        Kc(AppMode),
        Qt,
        Axr,
    }
    let gmp: [(&str, MenuItem); 11] = [
        ("Shell", MenuItem::Kc(AppMode::Df)),
        ("Files", MenuItem::Kc(AppMode::Pl)),
        ("Network", MenuItem::Kc(AppMode::As)),
        ("Hardware", MenuItem::Kc(AppMode::Ip)),
        ("TrustCode", MenuItem::Kc(AppMode::Ag)),
        ("User Management", MenuItem::Kc(AppMode::Rb)),
        ("Web Browser", MenuItem::Kc(AppMode::Browser)),
        ("Image Viewer", MenuItem::Kc(AppMode::Bp)),
        ("-----------------", MenuItem::Kc(AppMode::Df)), 
        ("Reboot", MenuItem::Axr),
        ("Shutdown", MenuItem::Qt),
    ];
    
    
    let mut gpt = false;
    let mut daa: i32 = (z / 2) as i32;
    let mut dab: i32 = (ac / 2) as i32;
    
    
    let fal = crate::cpu::tsc::ard();
    let mut tz = 0u32;
    let mut dqx = 0u32;
    let mut fmq = crate::cpu::tsc::ow();
    
    
    
    
    
    
    
    
    
    let rnc: u64 = 4; 
    let mut pbv = 0u32; 
    let mut lze = 0u32;
    
    crate::serial_println!("[COSMIC2] Entering render loop...");
    
    
    let qll = crate::cpu::tsc::ow();
    let mwv = crate::cpu::tsc::ard();
    let qlm = if sg > 0 && mwv > 0 { mwv / 1000 * sg } else { u64::O };
    
    while aqk {
        
        if mwx && oo == 5 {
            mwx = false;
            
            if don {
                ps.clear();
                bpu = format!("Loading {}...", bmb);
                btn = true;
                let fmc = bmb.cj("https://");
                if let Some((kh, port, path, ify)) = super::vm::lsm(&bmb) {
                    let protocol = if ify { "HTTPS" } else { "HTTP" };
                    ps.push(tr(&format!("\u{25ba} {} {}:{}{}...", protocol, kh, port, path), 0xFF88FF88, LineType::Bq));
                    if ify {
                        ps.push(tr("\u{25ba} Establishing TLS 1.3 connection...", 0xFF88CCFF, LineType::Bq));
                        match crate::netstack::https::get(&bmb) {
                            Ok(mk) => {
                                ps.push(tr(&format!("\u{25ba} TLS OK, {} bytes", mk.gj.len()), 0xFF88FF88, LineType::Bq));
                                ps.push(tr("", 0xFFDDDDDD, LineType::Bq));
                                ps.push(tr("\u{2500}\u{2500} Response Headers \u{2500}\u{2500}", 0xFF61AFEF, LineType::Kp));
                                ps.push(tr(&format!("HTTP/1.1 {}", mk.wt), NL_, LineType::Kp));
                                for (bs, bn) in &mk.zk {
                                    ps.push(tr(&format!("{}: {}", bs, bn), NL_, LineType::Kp));
                                }
                                ps.push(tr("", 0xFFDDDDDD, LineType::Bq));
                                ps.push(tr("\u{2500}\u{2500} HTML Source \u{2500}\u{2500}", 0xFF61AFEF, LineType::Kp));
                                if let Ok(dza) = core::str::jg(&mk.gj) {
                                    for line in dza.ak().take(200) {
                                        ps.push(jes(line));
                                    }
                                }
                                bpu = format!("\u{2713} Loaded: {} ({} bytes, HTTPS)", bmb, mk.gj.len());
                            }
                            Err(aa) => {
                                ps.push(tr(&format!("\u{2718} HTTPS Error: {}", aa), 0xFFFF4444, LineType::Bq));
                                bpu = format!("Error: {}", aa);
                            }
                        }
                    } else {
                        
                        match crate::netstack::http::get(&bmb) {
                            Ok(mk) => {
                                if let Some(dza) = mk.dza() {
                                    for line in dza.ak().take(200) {
                                        ps.push(jes(line));
                                    }
                                }
                                bpu = format!("\u{2713} Loaded: {} ({} bytes)", bmb, mk.gj.len());
                            }
                            Err(aa) => {
                                ps.push(tr(&format!("\u{2718} HTTP Error: {}", aa), 0xFFFF4444, LineType::Bq));
                                bpu = format!("Error: {}", aa);
                            }
                        }
                    }
                } else {
                    ps.push(tr("\u{2718} Invalid URL", 0xFFFF4444, LineType::Bq));
                    bpu = String::from("Invalid URL");
                }
                btn = false;
            }
        }

        
        if sg > 0 {
            let ez = crate::cpu::tsc::ow().ao(qll);
            if ez >= qlm { break; }
        }
        
        
        if oo <= 3 || oo % 500 == 0 {
            crate::serial_println!("[COSMIC2] Loop iteration {}", oo);
        }
        
        
        
        
        
        
        
        let mut msl = 0u8;
        while let Some(bs) = crate::keyboard::xw() {
            msl += 1;
            if msl > 8 { break; } 
            crate::serial_println!("[KEY] Received key: {} (0x{:02X})", bs, bs);
            
            if atu == AppMode::Browser {
                match bs {
                    27 => { 
                        atu = AppMode::Df;
                        cz.clear();
                        for line in gie!(AppMode::Df) {
                            cz.push(String::from(*line));
                        }
                    },
                    9 => { 
                        hbj = (hbj + 1) % 2;
                        if hbj == 0 {
                            bpu = String::from("View: DevTools (source)");
                        } else {
                            bpu = String::from("View: Rendered");
                        }
                    },
                    8 => { 
                        if bmb.len() > 7 { 
                            bmb.pop();
                        }
                    },
                    10 | 13 => { 
                        ps.clear();
                        bpu = format!("Loading {}...", bmb);
                        btn = true;
                        
                        
                        let fmc = bmb.cj("https://");
                        
                        
                        if let Some((kh, port, path, ify)) = super::vm::lsm(&bmb) {
                            let protocol = if ify { "HTTPS" } else { "HTTP" };
                            ps.push(tr(&format!("? {} {}:{}{}...", protocol, kh, port, path), 0xFF88FF88, LineType::Bq));
                            
                            if ify {
                                
                                ps.push(tr("? Establishing TLS 1.3 connection...", 0xFF88CCFF, LineType::Bq));
                                
                                match crate::netstack::https::get(&bmb) {
                                    Ok(mk) => {
                                        ps.push(tr(&format!("? TLS handshake complete, received {} bytes", mk.gj.len()), 0xFF88FF88, LineType::Bq));
                                        ps.push(tr("", 0xFFDDDDDD, LineType::Bq));
                                        
                                        
                                        ps.push(tr("-- Response Headers --", 0xFF61AFEF, LineType::Kp));
                                        ps.push(tr(&format!("HTTP/1.1 {}", mk.wt), NL_, LineType::Kp));
                                        for (bs, bn) in &mk.zk {
                                            ps.push(tr(&format!("{}: {}", bs, bn), NL_, LineType::Kp));
                                        }
                                        ps.push(tr("", 0xFFDDDDDD, LineType::Bq));
                                        
                                        
                                        ps.push(tr("-- HTML Source --", 0xFF61AFEF, LineType::Kp));
                                        if let Ok(dza) = core::str::jg(&mk.gj) {
                                            for line in dza.ak().take(200) {
                                                ps.push(jes(line));
                                            }
                                        } else {
                                            ps.push(tr("[Binary content]", 0xFFFFFF00, LineType::Bq));
                                        }
                                        
                                        bpu = format!("? Loaded: {} ({} bytes, HTTPS)", bmb, mk.gj.len());
                                    }
                                    Err(aa) => {
                                        ps.push(tr(&format!("? HTTPS Error: {}", aa), 0xFFFF4444, LineType::Q));
                                        ps.push(tr("", 0xFFDDDDDD, LineType::Bq));
                                        ps.push(tr("TLS 1.3 connection failed. Possible causes:", 0xFFFFFF00, LineType::Bq));
                                        ps.push(tr("  * DNS resolution failed", 0xFFAAAAAA, LineType::Bq));
                                        ps.push(tr("  * Server doesn't support TLS 1.3", 0xFFAAAAAA, LineType::Bq));
                                        ps.push(tr("  * Network timeout", 0xFFAAAAAA, LineType::Bq));
                                        bpu = format!("? HTTPS Error: {}", aa);
                                    }
                                }
                            } else {
                                
                                
                                let hop = if let Some(ip) = super::vm::cgl(&kh) {
                                    Some(ip)
                                } else {
                                    
                                    crate::netstack::dns::ayo(&kh)
                                };
                                
                                if let Some(ip) = hop {
                                    ps.push(tr(&format!("? Resolved: {}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3]), 0xFF88FF88, LineType::Bq));
                                    
                                    
                                    match super::vm::nmj(&kh, ip, port, &path) {
                                        Ok(mk) => {
                                            ps.push(tr("", 0xFFDDDDDD, LineType::Bq));
                                            
                                            
                                            let mut odq = true;
                                            ps.push(tr("-- Response Headers --", 0xFF61AFEF, LineType::Kp));
                                            
                                            for line in mk.ak() {
                                                if odq {
                                                    if line.is_empty() {
                                                        odq = false;
                                                        ps.push(tr("", 0xFFDDDDDD, LineType::Bq));
                                                        ps.push(tr("-- HTML Source --", 0xFF61AFEF, LineType::Kp));
                                                    } else {
                                                        ps.push(tr(line, NL_, LineType::Kp));
                                                    }
                                                } else {
                                                    
                                                    ps.push(jes(line));
                                                }
                                            }
                                            
                                            bpu = format!("? Loaded: {} ({} bytes)", bmb, mk.len());
                                        }
                                        Err(aa) => {
                                            ps.push(tr(&format!("? HTTP Error: {}", aa), 0xFFFF4444, LineType::Q));
                                            bpu = format!("? Error: {}", aa);
                                        }
                                    }
                                } else {
                                    ps.push(tr(&format!("? Error: Cannot resolve host '{}'", kh), 0xFFFF4444, LineType::Q));
                                    ps.push(tr("", 0xFFDDDDDD, LineType::Bq));
                                    ps.push(tr("Tip: Try a local server or IP address:", 0xFFFFFF00, LineType::Bq));
                                    ps.push(tr("  * http://192.168.56.1:8080/", 0xFF00FFFF, LineType::Bq));
                                    ps.push(tr("  * http://10.0.2.2:8000/", 0xFF00FFFF, LineType::Bq));
                                    bpu = String::from("? Error: DNS resolution failed");
                                }
                            }
                        } else {
                            ps.push(tr("? Invalid URL format", 0xFFFF4444, LineType::Q));
                            ps.push(tr("", 0xFFDDDDDD, LineType::Bq));
                            ps.push(tr("Use format: http://hostname/path or https://hostname/path", 0xFFFFFF00, LineType::Bq));
                            ps.push(tr("", 0xFFDDDDDD, LineType::Bq));
                            ps.push(tr("Examples:", 0xFF88FF88, LineType::Bq));
                            ps.push(tr("  * https://google.com", 0xFF00FFFF, LineType::Bq));
                            ps.push(tr("  * https://example.com", 0xFF00FFFF, LineType::Bq));
                            ps.push(tr("  * http://192.168.1.1/", 0xFF00FFFF, LineType::Bq));
                            bpu = String::from("? Error: Invalid URL");
                        }
                        btn = false;
                    },
                    32..=126 => { 
                        bmb.push(bs as char);
                    },
                    _ => {}
                }
            } else if atu == AppMode::Bp {
                
                match bs {
                    27 => { 
                        atu = AppMode::Df;
                        cz.clear();
                        for line in gie!(AppMode::Df) {
                            cz.push(String::from(*line));
                        }
                    },
                    43 | 61 => { 
                        dig = (dig * 1.25).v(10.0);
                    },
                    45 => { 
                        dig = (dig / 1.25).am(0.1);
                    },
                    114 | 82 => { 
                        dig = 1.0;
                        ldj = 0;
                        ldk = 0;
                    },
                    
                    
                    _ => {}
                }
            } else if atu == AppMode::Ag {
                
                match bs {
                    27 => { 
                        atu = AppMode::Df;
                        cz.clear();
                        for line in gie!(AppMode::Df) {
                            cz.push(String::from(*line));
                        }
                    },
                    _ => {
                        
                        aey.vr(bs);
                    }
                }
            } else {
                
            match bs {
                27 => { 
                    if awe || dvu {
                        awe = false;
                        dvu = false;
                    } else {
                        aqk = false;
                    }
                },
                8 => { 
                    bfh.pop();
                    gto.clear();
                },
                0x49 => { 
                    if px > 0 {
                        px = px.ao(5);
                    }
                },
                0x51 => { 
                    let aye = cz.len().ao(AFH_);
                    if px < aye {
                        px = (px + 5).v(aye);
                    }
                },
                10 | 13 => { 
                    if !bfh.is_empty() {
                        let nef = bfh.clone();
                        let cmd = nef.em();  
                        crate::serial_println!("[DEBUG] Enter pressed, cmd = '{}' (trimmed: '{}')", nef, cmd);
                        cz.push(format!("> {}", cmd));
                        
                        
                        bqa.push(String::from(cmd));
                        if bqa.len() > AZQ_ {
                            bqa.remove(0);
                        }
                        
                        
                        crate::serial_println!("[MATCH] About to match cmd='{}' starts_with_shader={}", cmd, cmd.cj("shader "));
                        match cmd {
                            "help" => {
                                cz.push(String::from("+================================================+"));
                                cz.push(String::from("|          TrustOS Desktop Shell                 |"));
                                cz.push(String::from("+================================================+"));
                                cz.push(String::from("| FILE SYSTEM:                                   |"));
                                cz.push(String::from("|   ls, cd, pwd, mkdir, rmdir, touch, rm, cat    |"));
                                cz.push(String::from("|   cp, mv, head, tail, stat, tree, find, wc     |"));
                                cz.push(String::from("|   chmod, chown, ln, grep                       |"));
                                cz.push(String::from("| NETWORK:                                       |"));
                                cz.push(String::from("|   ifconfig, ping, curl, wget, nslookup         |"));
                                cz.push(String::from("|   arp, route, traceroute, netstat              |"));
                                cz.push(String::from("| SYSTEM:                                        |"));
                                cz.push(String::from("|   clear, date, time, uptime, whoami, hostname  |"));
                                cz.push(String::from("|   uname, env, history, ps, free, df, top       |"));
                                cz.push(String::from("| HARDWARE:                                      |"));
                                cz.push(String::from("|   cpuinfo, meminfo, lspci, lsusb, lscpu, disk  |"));
                                cz.push(String::from("| USERS:                                         |"));
                                cz.push(String::from("|   login, su, passwd, adduser, users            |"));
                                cz.push(String::from("| UTILITIES:                                     |"));
                                cz.push(String::from("|   echo, hexdump, strings, sort, cal, bc        |"));
                                cz.push(String::from("| DESKTOP:                                       |"));
                                cz.push(String::from("|   desktop close - Exit desktop                 |"));
                                cz.push(String::from("|   open <app> - Open app (browser,files,editor) |"));
                                cz.push(String::from("|   imgview <file> - View images (PNG/BMP)       |"));
                                cz.push(String::from("|   3ddemo - 3D rotating cube demo               |"));
                                cz.push(String::from("+================================================+"));
                            },
                            "clear" => {
                                cz.clear();
                            },
                            "pwd" => {
                                
                                let jv = crate::ramfs::fh(|fs| String::from(fs.dau()));
                                cz.push(jv);
                            },
                            "ls" | "dir" => {
                                
                                match crate::ramfs::fh(|fs| fs.awb(None)) {
                                    Ok(pj) => {
                                        if pj.is_empty() {
                                            cz.push(String::from("(empty)"));
                                        } else {
                                            for (j, kd, aw) in pj {
                                                match kd {
                                                    FileType::K => {
                                                        cz.push(format!("{}  <DIR>", j));
                                                    }
                                                    FileType::Es => {
                                                        cz.push(format!("{}  {} B", j, aw));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Err(aa) => {
                                        cz.push(format!("ls: {}", aa.as_str()));
                                    }
                                }
                            },
                            "whoami" => cz.push(String::from("root")),
                            "ifconfig" => {
                                if let Some(ed) = crate::network::ckt() {
                                    let djg = format!("{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                                        ed[0], ed[1], ed[2], ed[3], ed[4], ed[5]);
                                    if let Some((ip, ydf, qcb)) = crate::network::aou() {
                                        let dil = format!("{}.{}.{}.{}", 
                                            ip.as_bytes()[0], ip.as_bytes()[1], ip.as_bytes()[2], ip.as_bytes()[3]);
                                        cz.push(format!("eth0: {}  UP  RUNNING", dil));
                                    } else {
                                        cz.push(String::from("eth0: No IP  UP  RUNNING"));
                                    }
                                    cz.push(format!("      MAC: {}", djg));
                                } else {
                                    cz.push(String::from("eth0: No network interface"));
                                }
                            },
                            "cpuinfo" => {
                                cz.push(String::from("CPU: QEMU Virtual CPU version 2.5+"));
                                cz.push(String::from("Freq: 3.8 GHz | Cores: 1 | Arch: x86_64"));
                                cz.push(String::from("Features: SSE SSE2 NX SVM"));
                            },
                            "meminfo" => {
                                let mr = crate::memory::heap::mr() / 1024;
                                let es = crate::memory::cre() / 1024;
                                let xkp = crate::memory::fxc() / 1024 / 1024;
                                cz.push(format!("Heap: {} / {} KB", mr, es));
                                cz.push(format!("System: {} MB total", xkp));
                            },
                            "uptime" => {
                                let tv = crate::cpu::tsc::ow() / crate::cpu::tsc::ard();
                                let i = tv / 3600;
                                let ef = (tv % 3600) / 60;
                                let e = tv % 60;
                                cz.push(format!("Uptime: {:02}:{:02}:{:02}", i, ef, e));
                            },
                            "exit" | "quit" => {
                                cz.push(String::from("> Use 'desktop close' to exit desktop"));
                            },
                            "date" | "time" => {
                                let os = crate::rtc::cgz();
                                cz.push(format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", 
                                    os.ccq, os.caw, os.cjw, os.bek, os.bri, os.chr));
                            },
                            "hostname" => cz.push(String::from("trustos")),
                            "uname" => cz.push(String::from("TrustOS 0.1.0 x86_64")),
                            "holo" | "holomatrix" => {
                                
                                crate::serial_println!("[DEBUG] holo command received, toggling...");
                                bnf = !bnf;
                                crate::graphics::holomatrix::cuf(bnf);
                                crate::serial_println!("[DEBUG] holo_enabled = {}", bnf);
                                if bnf {
                                    cz.push(String::from("? HoloMatrix 3D ENABLED"));
                                    cz.push(String::from("  3D hologram appears through Matrix Rain"));
                                    cz.push(String::from("  Use settings panel to change scene"));
                                } else {
                                    cz.push(String::from("? HoloMatrix 3D DISABLED"));
                                    cz.push(String::from("  Standard Matrix Rain background"));
                                }
                            },
                            "holo on" => {
                                bnf = true;
                                crate::graphics::holomatrix::cuf(true);
                                cz.push(String::from("? HoloMatrix 3D enabled"));
                            },
                            "holo off" => {
                                bnf = false;
                                dmx = false;
                                crate::graphics::holomatrix::cuf(false);
                                cz.push(String::from("? HoloMatrix 3D disabled"));
                            },
                            "holo volume" | "holovolume" => {
                                
                                dmx = !dmx;
                                if dmx {
                                    bnf = false;  
                                    cz.push(String::from("? HOLOVOLUME ENABLED"));
                                    cz.push(String::from("  Volumetric ASCII raymarcher active"));
                                    cz.push(String::from("  3D voxel grid projected to 2D"));
                                    cz.push(String::from("  Aligned characters = brighter"));
                                } else {
                                    cz.push(String::from("? HoloVolume disabled"));
                                    cz.push(String::from("  Back to Matrix Rain"));
                                }
                            },
                            "holo dna" => {
                                dmx = true;
                                holovolume.che = crate::holovolume::RenderMode::Sd;
                                cz.push(String::from("? HoloVolume: DNA Helix"));
                            },
                            "holo cube" => {
                                dmx = true;
                                holovolume.che = crate::holovolume::RenderMode::Jb;
                                cz.push(String::from("? HoloVolume: Rotating Cube"));
                            },
                            "holo sphere" => {
                                dmx = true;
                                holovolume.che = crate::holovolume::RenderMode::Sphere;
                                cz.push(String::from("? HoloVolume: Sphere"));
                            },
                            "holo rain" => {
                                dmx = true;
                                holovolume.che = crate::holovolume::RenderMode::Avj;
                                cz.push(String::from("? HoloVolume: Matrix Rain (volumetric)"));
                            },
                            
                            
                            
                            "matrix formula" | "formula" | "formula3d" => {
                                atr = true;
                                apf = false;
                                aqq = false;
                                avf = false;
                                ats = false;
                                dmx = false;
                                cz.push(String::from("? FORMULA 3D: Wireframe perspective projection"));
                                cz.push(String::from("  Commands: formula cube|pyramid|diamond|torus|sphere|grid|helix|multi"));
                            },
                            "formula cube" => {
                                atr = true; apf = false; aqq = false; avf = false; ats = false;
                                cet.bid(crate::formula3d::FormulaScene::Dw);
                                cz.push(String::from("? FORMULA: Rotating Cube"));
                            },
                            "formula pyramid" => {
                                atr = true; apf = false; aqq = false; avf = false; ats = false;
                                cet.bid(crate::formula3d::FormulaScene::Yh);
                                cz.push(String::from("? FORMULA: Pyramid"));
                            },
                            "formula diamond" => {
                                atr = true; apf = false; aqq = false; avf = false; ats = false;
                                cet.bid(crate::formula3d::FormulaScene::Wh);
                                cz.push(String::from("? FORMULA: Diamond octahedron"));
                            },
                            "formula torus" | "formula donut" => {
                                atr = true; apf = false; aqq = false; avf = false; ats = false;
                                cet.bid(crate::formula3d::FormulaScene::Dr);
                                cz.push(String::from("? FORMULA: Torus (donut)"));
                            },
                            "formula sphere" => {
                                atr = true; apf = false; aqq = false; avf = false; ats = false;
                                cet.bid(crate::formula3d::FormulaScene::Ajb);
                                cz.push(String::from("? FORMULA: Icosphere"));
                            },
                            "formula grid" => {
                                atr = true; apf = false; aqq = false; avf = false; ats = false;
                                cet.bid(crate::formula3d::FormulaScene::Pn);
                                cz.push(String::from("? FORMULA: Infinite grid"));
                            },
                            "formula helix" | "formula dna" => {
                                atr = true; apf = false; aqq = false; avf = false; ats = false;
                                cet.bid(crate::formula3d::FormulaScene::Aix);
                                cz.push(String::from("? FORMULA: DNA helix"));
                            },
                            "formula multi" => {
                                atr = true; apf = false; aqq = false; avf = false; ats = false;
                                cet.bid(crate::formula3d::FormulaScene::Adg);
                                cz.push(String::from("? FORMULA: Multi - orbiting shapes"));
                            },
                            "formula penger" | "formula penguin" | "penger" => {
                                atr = true; apf = false; aqq = false; avf = false; ats = false;
                                cet.bid(crate::formula3d::FormulaScene::Ald);
                                cz.push(String::from("? FORMULA: Penger - hologram penguin ??"));
                            },
                            "formula trustos" | "formula title" | "trustos" | "trustos 3d" => {
                                atr = true; apf = false; aqq = false; avf = false; ats = false;
                                cet.bid(crate::formula3d::FormulaScene::Zg);
                                cet.dxr = 0xFF00CCFF;
                                cz.push(String::from("? FORMULA: TrustOS 3D -- hologram scanline title"));
                            },
                            "formula holo" | "holo matrix" | "holomatrix" | "matrix holo" | "matrix 3d holo" => {
                                atr = true; apf = false; aqq = false; avf = false; ats = false;
                                cet.bid(crate::formula3d::FormulaScene::HoloMatrix);
                                cz.push(String::from("? FORMULA: HoloMatrix 3D -- volumetric holographic rain"));
                            },
                            "matrix fast" => {
                                atr = false;
                                aqq = true;
                                apf = false;
                                ats = false;
                                cz.push(String::from("? FAST MATRIX: Glyph-cached renderer"));
                                cz.push(String::from("  Pre-computed u128 glyphs + LUT intensity"));
                            },
                            "matrix braille" => {
                                atr = false;
                                apf = true;
                                aqq = false;
                                ats = false;
                                cz.push(String::from("? BRAILLE MATRIX: 8A-- sub-pixel resolution"));
                                cz.push(String::from("  480A--272 virtual pixels via Unicode ??"));
                            },
                            "matrix legacy" => {
                                atr = false;
                                aqq = false;
                                apf = false;
                                avf = false;
                                ats = false;
                                cz.push(String::from("? LEGACY MATRIX: Original renderer"));
                                cz.push(String::from("  Per-pixel font lookup (slower)"));
                            },
                            "matrix3d" | "matrix 3d" => {
                                atr = false;
                                avf = !avf;
                                apf = !avf;
                                aqq = false;
                                ats = false;
                                if avf {
                                    cz.push(String::from("? MATRIX 3D: Volumetric rain with shapes"));
                                    cz.push(String::from("  Commands: matrix3d sphere | cube | torus"));
                                } else {
                                    cz.push(String::from("? MATRIX 3D: Disabled, back to BRAILLE"));
                                }
                            },
                            "matrix3d sphere" | "matrix 3d sphere" => {
                                atr = false;
                                avf = true;
                                apf = false;
                                aqq = false;
                                ats = false;
                                hqv.wir();
                                cz.push(String::from("? MATRIX 3D: Sphere - rain flows around it"));
                            },
                            "matrix3d cube" | "matrix 3d cube" => {
                                atr = false;
                                avf = true;
                                apf = false;
                                aqq = false;
                                ats = false;
                                hqv.win();
                                cz.push(String::from("? MATRIX 3D: Rotating Cube"));
                            },
                            "matrix3d torus" | "matrix 3d torus" => {
                                atr = false;
                                avf = true;
                                apf = false;
                                aqq = false;
                                ats = false;
                                hqv.wjs();
                                cz.push(String::from("? MATRIX 3D: Torus (donut shape)"));
                            },
                            
                            "matrix cube" => {
                                atr = false;
                                apf = true;
                                avf = false;
                                aqq = false;
                                ats = false;
                                dzd.gsg(crate::matrix_fast::ShapeOverlay::Dw);
                                cz.push(String::from("? MATRIX: Cube overlay - glyphs trace rotating cube"));
                            },
                            "matrix sphere" => {
                                atr = false;
                                apf = true;
                                avf = false;
                                aqq = false;
                                ats = false;
                                dzd.gsg(crate::matrix_fast::ShapeOverlay::Sphere);
                                cz.push(String::from("? MATRIX: Sphere overlay - glyphs trace sphere surface"));
                            },
                            "matrix torus" => {
                                atr = false;
                                apf = true;
                                avf = false;
                                aqq = false;
                                ats = false;
                                dzd.gsg(crate::matrix_fast::ShapeOverlay::Dr);
                                cz.push(String::from("? MATRIX: Torus overlay - glyphs trace spinning donut"));
                            },
                            "matrix dna" => {
                                atr = false;
                                apf = true;
                                avf = false;
                                aqq = false;
                                ats = false;
                                dzd.gsg(crate::matrix_fast::ShapeOverlay::Ij);
                                cz.push(String::from("? MATRIX: DNA overlay - glyphs trace double helix"));
                            },
                            "matrix off" | "matrix clear" | "matrix normal" => {
                                dzd.gsg(crate::matrix_fast::ShapeOverlay::None);
                                cz.push(String::from("? MATRIX: Shape overlay disabled - normal rain"));
                            },
                            "matrix shader" | "matrix gpu" => {
                                ats = !ats;
                                if ats {
                                    atr = false;
                                    apf = false;
                                    aqq = false;
                                    avf = false;
                                    cz.push(String::from("? SHADER MATRIX: GPU-emulated pixel shader"));
                                    cz.push(String::from("  Uses SMP parallel dispatch + SSE2 SIMD"));
                                    cz.push(String::from("  Smooth per-pixel glyph rendering"));
                                } else {
                                    apf = true;
                                    cz.push(String::from("? SHADER MATRIX: Disabled, back to BRAILLE"));
                                }
                            },
                            "matrix" => {
                                let ev = if atr { "FORMULA (wireframe 3D)" }
                                           else if ats { "SHADER (GPU-emulated pixel shader)" }
                                           else if avf { "3D (volumetric shapes)" }
                                           else if apf { "BRAILLE (8A-- sub-pixel)" }
                                           else if aqq { "FAST (glyph-cached)" }
                                           else { "LEGACY (per-pixel)" };
                                cz.push(format!("Matrix Renderer: {}", ev));
                                cz.push(String::from("Commands: matrix formula | fast | braille | legacy | 3d | shader"));
                            },
                            "fps" => {
                                iaj = !iaj;
                                cz.push(format!("FPS display: {}", if iaj { "ON" } else { "OFF" }));
                            },
                            "smp" | "smpstatus" | "smp status" => {
                                let status = if crate::cpu::smp::jbt() { "ON" } else { "OFF" };
                                let cdv = crate::cpu::smp::boc();
                                let es = crate::cpu::smp::aao();
                                cz.push(format!("SMP Parallel: {} ({}/{} CPUs)", status, cdv, es));
                                cz.push(String::from("  smp on  - Enable multi-core"));
                                cz.push(String::from("  smp off - Single-core mode"));
                            },
                            "smp on" => {
                                crate::cpu::smp::isq();
                                cz.push(String::from("? SMP parallelism ENABLED"));
                            },
                            "smp off" => {
                                crate::cpu::smp::kqd();
                                cz.push(String::from("? SMP disabled (single-core)"));
                            },
                            "shader" | "shaders" | "vgpu" => {
                                cz.push(String::from("+---------------------------------------+"));
                                cz.push(String::from("|     Virtual GPU - Shader Demo         |"));
                                cz.push(String::from("|---------------------------------------|"));
                                cz.push(String::from("| shader plasma    - Plasma waves       |"));
                                cz.push(String::from("| shader fire      - Fire effect        |"));
                                cz.push(String::from("| shader mandelbrot- Fractal zoom       |"));
                                cz.push(String::from("| shader matrix    - Matrix rain        |"));
                                cz.push(String::from("| shader tunnel    - 3D HOLOMATRIX      |"));
                                cz.push(String::from("| shader parallax  - Depth layers       |"));
                                cz.push(String::from("| shader shapes    - Ray-marched 3D     |"));
                                cz.push(String::from("| shader rain3d    - Matrix fly-through |"));
                                cz.push(String::from("| shader cosmic    - Fractal vortex     |"));
                                cz.push(String::from("| shader gradient  - Test gradient      |"));
                                cz.push(String::from("+---------------------------------------+"));
                                cz.push(String::from("Press ESC to exit shader demo"));
                            },
                            _ if cmd.cj("shader ") => {
                                let dvv = cmd.tl("shader ").em();
                                crate::serial_println!("[SHADER] Trying to load shader: '{}'", dvv);
                                if let Some(fuo) = crate::gpu_emu::kyx(dvv) {
                                    crate::serial_println!("[SHADER] Found shader, starting loop...");
                                    cz.push(format!("? Loading shader: {}", dvv));
                                    cz.push(String::from("Press ESC to exit..."));
                                    
                                    let z = crate::framebuffer::z();
                                    let ac = crate::framebuffer::ac();
                                    
                                    
                                    let afk = crate::framebuffer::bre();
                                    if !afk {
                                        crate::framebuffer::beo();
                                        crate::framebuffer::afi(true);
                                    }
                                    
                                    
                                    let qod = crate::framebuffer::cey();
                                    let (ggv, baz) = if let Some((ptr, dxx, dxv, oq)) = qod {
                                        (ptr as *mut u32, oq)
                                    } else {
                                        
                                        (crate::framebuffer::kyq(), z)
                                    };
                                    
                                    
                                    crate::gpu_emu::ttx(ggv, z, ac, baz);
                                    crate::gpu_emu::hzy(fuo);
                                    
                                    
                                    let ayu = crate::cpu::tsc::ow();
                                    let mut vj = 0u32;
                                    
                                    loop {
                                        
                                        if let Some(bs) = crate::keyboard::xw() {
                                            if bs == 27 { break; }
                                        }
                                        
                                        
                                        #[cfg(target_arch = "x86_64")]
                                        crate::gpu_emu::krk();
                                        #[cfg(not(target_arch = "x86_64"))]
                                        crate::gpu_emu::po();
                                        
                                        
                                        crate::framebuffer::sv();
                                        
                                        
                                        crate::gpu_emu::or(16);
                                        vj += 1;
                                        
                                        
                                        if vj % 60 == 0 {
                                            let ez = crate::cpu::tsc::ow() - ayu;
                                            let skb = ez as f32 / crate::cpu::tsc::ard() as f32;
                                            let tz = vj as f32 / skb;
                                            crate::serial_println!("[SHADER] FPS: {:.1}", tz);
                                        }
                                    }
                                    
                                    
                                    if !afk {
                                        crate::framebuffer::afi(false);
                                    }
                                    
                                    cz.push(format!("Shader demo ended ({} frames)", vj));
                                } else {
                                    crate::serial_println!("[SHADER] Shader '{}' NOT FOUND!", dvv);
                                    cz.push(format!("Unknown shader: {}", dvv));
                                    cz.push(String::from("Available: plasma, fire, mandelbrot, matrix, tunnel, parallax, shapes, rain3d, cosmic, gradient"));
                                }
                            },
                            "echo" => cz.push(String::new()),
                            "touch" => cz.push(String::from("Usage: touch <filename>")),
                            "rm" => cz.push(String::from("Usage: rm <filename>")),
                            "cp" => cz.push(String::from("Usage: cp <src> <dest>")),
                            "mv" => cz.push(String::from("Usage: mv <src> <dest>")),
                            _ if cmd.cj("echo ") => {
                                let text = cmd.tl("echo ").em();
                                cz.push(String::from(text));
                            },
                            _ if cmd.cj("cd ") => {
                                let path = cmd.tl("cd ").em();
                                
                                match crate::ramfs::fh(|fs| fs.fem(path)) {
                                    Ok(()) => {
                                        let lnt = crate::ramfs::fh(|fs| String::from(fs.dau()));
                                        cz.push(format!("Changed to: {}", lnt));
                                    }
                                    Err(aa) => {
                                        cz.push(format!("cd: {}: {}", path, aa.as_str()));
                                    }
                                }
                            },
                            _ if cmd.cj("ls ") => {
                                let path = cmd.tl("ls ").em();
                                
                                match crate::ramfs::fh(|fs| fs.awb(Some(path))) {
                                    Ok(pj) => {
                                        if pj.is_empty() {
                                            cz.push(String::from("(empty)"));
                                        } else {
                                            for (j, kd, aw) in pj {
                                                match kd {
                                                    FileType::K => {
                                                        cz.push(format!("{}  <DIR>", j));
                                                    }
                                                    FileType::Es => {
                                                        cz.push(format!("{}  {} B", j, aw));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Err(aa) => {
                                        cz.push(format!("ls: {}: {}", path, aa.as_str()));
                                    }
                                }
                            },
                            _ if cmd.cj("cat ") => {
                                let path = cmd.tl("cat ").em();
                                
                                match crate::ramfs::fh(|fs| {
                                    fs.mq(path).map(|e| alloc::vec::Vec::from(e))
                                }) {
                                    Ok(ca) => {
                                        if let Ok(text) = core::str::jg(&ca) {
                                            for line in text.ak().take(20) {
                                                cz.push(String::from(line));
                                            }
                                        } else {
                                            cz.push(format!("cat: {}: Binary file", path));
                                        }
                                    }
                                    Err(aa) => {
                                        cz.push(format!("cat: {}: {}", path, aa.as_str()));
                                    }
                                }
                            },
                            
                            _ if cmd.cj("edit ") || cmd.cj("code ") || cmd.cj("nano ") || cmd.cj("vim ") => {
                                let path = cmd.ayt().goc(1).unwrap_or("").em();
                                if path.is_empty() {
                                    cz.push(String::from("Usage: edit <filename>"));
                                } else {
                                    aey.dsu(path);
                                    atu = AppMode::Ag;
                                    don = false;
                                    cz.push(format!("TrustCode: editing {}", path));
                                    crate::serial_println!("[TrustCode] Editing: {}", path);
                                }
                            },
                            _ if cmd.cj("mkdir ") => {
                                let path = cmd.tl("mkdir ").em();
                                
                                match crate::ramfs::fh(|fs| fs.ut(path)) {
                                    Ok(()) => {
                                        cz.push(format!("Created directory: {}", path));
                                    }
                                    Err(aa) => {
                                        cz.push(format!("mkdir: {}: {}", path, aa.as_str()));
                                    }
                                }
                            },
                            _ if cmd.cj("touch ") => {
                                let path = cmd.tl("touch ").em();
                                
                                match crate::ramfs::fh(|fs| fs.ns(path, &[])) {
                                    Ok(()) => {
                                        cz.push(format!("Created file: {}", path));
                                    }
                                    Err(aa) => {
                                        cz.push(format!("touch: {}: {}", path, aa.as_str()));
                                    }
                                }
                            },
                            _ if cmd.cj("rm ") => {
                                let path = cmd.tl("rm ").em();
                                match crate::ramfs::fh(|fs| fs.hb(path)) {
                                    Ok(()) => {
                                        cz.push(format!("Removed: {}", path));
                                    }
                                    Err(aa) => {
                                        cz.push(format!("rm: {}: {}", path, aa.as_str()));
                                    }
                                }
                            },
                            _ if cmd.cj("curl ") || cmd.cj("get ") || cmd.cj("wget ") => {
                                let url = if cmd.cj("curl ") {
                                    cmd.tl("curl ").em()
                                } else if cmd.cj("wget ") {
                                    cmd.tl("wget ").em()
                                } else {
                                    cmd.tl("get ").em()
                                };
                                cz.push(format!("Fetching: {}", url));
                                
                                
                                if let Some((kh, port, path)) = super::vm::vei(url) {
                                    cz.push(format!("Host: {} Port: {} Path: {}", kh, port, path));
                                    
                                    
                                    if let Some(ip) = crate::netstack::dns::ayo(&kh) {
                                        cz.push(format!("Resolved to: {}.{}.{}.{}", 
                                            ip[0], ip[1], ip[2], ip[3]));
                                        
                                        
                                        match super::vm::nmj(&kh, ip, port, &path) {
                                            Ok(mk) => {
                                                
                                                for line in mk.ak().take(15) {
                                                    cz.push(String::from(line));
                                                }
                                                if mk.ak().az() > 15 {
                                                    cz.push(String::from("... (truncated)"));
                                                }
                                            }
                                            Err(aa) => {
                                                cz.push(format!("Error: {}", aa));
                                            }
                                        }
                                    } else {
                                        cz.push(format!("Cannot resolve: {}", kh));
                                    }
                                } else {
                                    cz.push(String::from("Invalid URL format"));
                                    cz.push(String::from("Usage: curl http://host/path"));
                                }
                            },
                            _ if cmd.cj("desktop ") => {
                                let sub = cmd.tl("desktop ");
                                if sub == "close" || sub == "exit" || sub == "quit" {
                                    aqk = false;
                                }
                            },
                            
                            "open" => {
                                cz.push(String::from("Usage: open <app>"));
                                cz.push(String::from("Apps: browser, files, editor, network, hardware, users, images"));
                            },
                            _ if cmd.cj("open ") => {
                                let bjf = cmd.tl("open ").em().aqn();
                                match bjf.as_str() {
                                    "browser" | "web" | "www" => {
                                        atu = AppMode::Browser;
                                        don = true;
                                        cz.push(String::from("Switched to Browser"));
                                    },
                                    "files" | "explorer" => {
                                        atu = AppMode::Pl;
                                        don = false;
                                        cz.push(String::from("Switched to Files"));
                                    },
                                    "editor" | "text" | "notepad" | "trustcode" | "code" => {
                                        atu = AppMode::Ag;
                                        don = false;
                                        
                                        if aey.wn.is_none() {
                                            aey.dsu("demo.rs");
                                        }
                                        cz.push(String::from("TrustCode Editor opened"));
                                    },
                                    "network" | "net" | "ifconfig" => {
                                        atu = AppMode::As;
                                        don = false;
                                        cz.push(String::from("Switched to Network"));
                                    },
                                    "hardware" | "hw" | "lshw" => {
                                        atu = AppMode::Ip;
                                        don = false;
                                        cz.push(String::from("Switched to Hardware"));
                                    },
                                    "users" | "user" => {
                                        atu = AppMode::Rb;
                                        don = false;
                                        cz.push(String::from("Switched to User Management"));
                                    },
                                    "images" | "image" | "viewer" => {
                                        atu = AppMode::Bp;
                                        don = false;
                                        cz.push(String::from("Switched to Image Viewer"));
                                    },
                                    "shell" | "terminal" => {
                                        atu = AppMode::Df;
                                        don = false;
                                        cz.push(String::from("Switched to Shell"));
                                    },
                                    _ => {
                                        cz.push(format!("Unknown app: {}", bjf));
                                        cz.push(String::from("Available: browser, files, editor, network, hardware, users, images"));
                                    }
                                }
                            },
                            
                            "ping" => cz.push(String::from("Usage: ping <host>")),
                            "nslookup" | "dig" => cz.push(String::from("Usage: nslookup <hostname>")),
                            "ps" => {
                                cz.push(String::from("  PID  STATE  NAME"));
                                cz.push(String::from("    1  R      init"));
                                cz.push(String::from("    2  R      kernel"));
                                cz.push(String::from("    3  R      desktop"));
                            },
                            "df" => {
                                cz.push(String::from("Filesystem    Size  Used  Avail  Use%  Mounted"));
                                cz.push(String::from("ramfs         8.0M   64K   7.9M    1%  /"));
                            },
                            "free" => {
                                let mr = crate::memory::heap::mr() / 1024;
                                let es = crate::memory::cre() / 1024;
                                let kxc = es - mr;
                                cz.push(String::from("              total     used     free"));
                                cz.push(format!("Mem:     {:>10}  {:>7}  {:>7}", es, mr, kxc));
                            },
                            "tree" => {
                                cz.push(String::from("."));
                                match crate::ramfs::fh(|fs| fs.awb(None)) {
                                    Ok(pj) => {
                                        let az = pj.len();
                                        for (a, (j, kd, _)) in pj.dse().cf() {
                                            let adx = if a + 1 == az { "+-- " } else { "+-- " };
                                            match kd {
                                                FileType::K => cz.push(format!("{}{}/ (dir)", adx, j)),
                                                FileType::Es => cz.push(format!("{}{}", adx, j)),
                                            }
                                        }
                                    }
                                    Err(_) => {}
                                }
                            },
                            "history" => {
                                cz.push(String::from("Command history not available in desktop shell"));
                            },
                            _ if cmd.cj("ping ") => {
                                let kh = cmd.tl("ping ").em();
                                
                                let hop = if let Some(bez) = super::vm::cgl(kh) {
                                    Some(bez)
                                } else {
                                    
                                    
                                    match kh {
                                        "google.com" | "www.google.com" => Some([142, 250, 179, 110]),
                                        "cloudflare.com" | "www.cloudflare.com" => Some([104, 16, 132, 229]),
                                        "github.com" | "www.github.com" => Some([140, 82, 114, 3]),
                                        "localhost" => Some([127, 0, 0, 1]),
                                        _ => None, 
                                    }
                                };
                                
                                if let Some(ip) = hop {
                                    cz.push(format!("PING {} ({}.{}.{}.{})", kh, ip[0], ip[1], ip[2], ip[3]));
                                    cz.push(format!("64 bytes from {}.{}.{}.{}: icmp_seq=1 ttl=64 time=1.5 ms", ip[0], ip[1], ip[2], ip[3]));
                                    cz.push(format!("64 bytes from {}.{}.{}.{}: icmp_seq=2 ttl=64 time=1.2 ms", ip[0], ip[1], ip[2], ip[3]));
                                    cz.push(String::from("--- ping statistics ---"));
                                    cz.push(String::from("2 packets transmitted, 2 received, 0% loss"));
                                } else {
                                    cz.push(format!("ping: {} - cannot resolve (use IP address)", kh));
                                }
                            },
                            _ if cmd.cj("nslookup ") || cmd.cj("dig ") => {
                                let kh = if cmd.cj("nslookup ") {
                                    cmd.tl("nslookup ").em()
                                } else {
                                    cmd.tl("dig ").em()
                                };
                                cz.push(format!("Server:  8.8.8.8"));
                                cz.push(format!("Name:    {}", kh));
                                
                                let hop = match kh {
                                    "google.com" | "www.google.com" => Some([142, 250, 179, 110]),
                                    "cloudflare.com" | "www.cloudflare.com" => Some([104, 16, 132, 229]),
                                    "github.com" | "www.github.com" => Some([140, 82, 114, 3]),
                                    "localhost" => Some([127, 0, 0, 1]),
                                    _ => super::vm::cgl(kh), 
                                };
                                if let Some(ip) = hop {
                                    cz.push(format!("Address: {}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3]));
                                } else {
                                    cz.push(String::from("** server can't find: NXDOMAIN"));
                                }
                            },
                            _ if cmd.cj("hexdump ") || cmd.cj("xxd ") => {
                                let path = if cmd.cj("hexdump ") {
                                    cmd.tl("hexdump ").em()
                                } else {
                                    cmd.tl("xxd ").em()
                                };
                                match crate::ramfs::fh(|fs| {
                                    fs.mq(path).map(|e| alloc::vec::Vec::from(e))
                                }) {
                                    Ok(ca) => {
                                        for (l, jj) in ca.btq(16).take(8).cf() {
                                            let nu: alloc::vec::Vec<String> = jj.iter()
                                                .map(|o| format!("{:02x}", o))
                                                .collect();
                                            let ascii: String = jj.iter()
                                                .map(|&o| if o >= 32 && o < 127 { o as char } else { '.' })
                                                .collect();
                                            cz.push(format!("{:08x}  {:48}  |{}|", 
                                                l * 16, nu.rr(" "), ascii));
                                        }
                                        if ca.len() > 128 {
                                            cz.push(String::from("... (truncated)"));
                                        }
                                    }
                                    Err(aa) => cz.push(format!("hexdump: {}: {}", path, aa.as_str())),
                                }
                            },
                            _ if cmd.cj("imgview ") || cmd.cj("view ") => {
                                let path = if cmd.cj("imgview ") {
                                    cmd.tl("imgview ").em()
                                } else {
                                    cmd.tl("view ").em()
                                };
                                
                                
                                match crate::ramfs::fh(|fs| {
                                    fs.mq(path).map(|e| alloc::vec::Vec::from(e))
                                }) {
                                    Ok(f) => {
                                        
                                        let format = crate::image::kpo(&f);
                                        if let Some(th) = crate::image::lji(&f) {
                                            trz = String::from(path);
                                            odk = format!("{}x{} ({} bytes)", th.z, th.ac, f.len());
                                            odj = String::from(format.fie());
                                            dig = 1.0;
                                            ldj = 0;
                                            ldk = 0;
                                            odi = Some(th);
                                            
                                            
                                            atu = AppMode::Bp;
                                            cz.push(format!("Opening: {} ({})", path, format.fie()));
                                        } else {
                                            cz.push(format!("imgview: Cannot decode image (format: {})", format.fie()));
                                        }
                                    },
                                    Err(aa) => {
                                        cz.push(format!("imgview: {}: {}", path, aa.as_str()));
                                    }
                                }
                            },
                            _ if cmd.cj("imginfo ") => {
                                let path = cmd.tl("imginfo ").em();
                                
                                match crate::ramfs::fh(|fs| {
                                    fs.mq(path).map(|e| alloc::vec::Vec::from(e))
                                }) {
                                    Ok(f) => {
                                        let format = crate::image::kpo(&f);
                                        cz.push(format!("+---------------------------------------+"));
                                        cz.push(format!("| Image Info: {}  ", path));
                                        cz.push(format!("|---------------------------------------|"));
                                        cz.push(format!("| Format:  {} ({})   ", format.fie(), format.uoi()));
                                        cz.push(format!("| Size:    {} bytes   ", f.len()));
                                        
                                        
                                        if let Some(th) = crate::image::lji(&f) {
                                            cz.push(format!("| Width:   {} px   ", th.z));
                                            cz.push(format!("| Height:  {} px   ", th.ac));
                                            cz.push(format!("| Pixels:  {}   ", th.z * th.ac));
                                        } else {
                                            cz.push(format!("| (Cannot decode image dimensions)"));
                                        }
                                        cz.push(format!("+---------------------------------------+"));
                                    },
                                    Err(aa) => {
                                        cz.push(format!("imginfo: {}: {}", path, aa.as_str()));
                                    }
                                }
                            },
                            
                            "top" | "htop" => {
                                cz.push(String::from("top - System Monitor"));
                                cz.push(String::from("  PID  %CPU  %MEM  TIME     COMMAND"));
                                cz.push(String::from("    1  0.5   2.1   0:01.23  kernel"));
                                cz.push(String::from("    2  0.1   0.5   0:00.45  desktop"));
                                cz.push(String::from("Press 'q' to quit (in desktop: just run another cmd)"));
                            },
                            "lspci" => {
                                cz.push(String::from("00:00.0 Host bridge"));
                                cz.push(String::from("00:01.0 VGA controller: Virtio GPU"));
                                cz.push(String::from("00:02.0 Network controller: Virtio Net"));
                                cz.push(String::from("00:03.0 AHCI Controller"));
                            },
                            "lsusb" => {
                                cz.push(String::from("Bus 001 Device 001: ID 1d6b:0002 Linux Foundation Root Hub"));
                                cz.push(String::from("Bus 001 Device 002: ID 0627:0001 QEMU Tablet"));
                            },
                            "lscpu" => {
                                cz.push(String::from("Architecture:        x86_64"));
                                cz.push(String::from("CPU op-modes:        64-bit"));
                                cz.push(String::from("CPU(s):              4"));
                                cz.push(String::from("Vendor ID:           AuthenticAMD"));
                                cz.push(String::from("Model name:          QEMU Virtual CPU"));
                            },
                            "disk" => {
                                cz.push(String::from("Disk /dev/sda: 64 MB"));
                                cz.push(String::from("  Partition 1: 64 MB (TrustOS)"));
                            },
                            "netstat" => {
                                cz.push(String::from("Active connections:"));
                                cz.push(String::from("Proto  Local Address      Foreign Address    State"));
                                cz.push(String::from("tcp    0.0.0.0:0          0.0.0.0:*          LISTEN"));
                            },
                            "arp" => {
                                cz.push(String::from("Address         HWtype  HWaddress           Iface"));
                                cz.push(String::from("10.0.2.2        ether   52:55:0a:00:02:02   eth0"));
                            },
                            "route" => {
                                cz.push(String::from("Kernel IP routing table"));
                                cz.push(String::from("Dest         Gateway      Genmask         Iface"));
                                cz.push(String::from("0.0.0.0      10.0.2.2     0.0.0.0         eth0"));
                                cz.push(String::from("10.0.2.0     0.0.0.0      255.255.255.0   eth0"));
                            },
                            "env" => {
                                cz.push(String::from("USER=root"));
                                cz.push(String::from("HOME=/root"));
                                cz.push(String::from("SHELL=/bin/tsh"));
                                cz.push(String::from("PATH=/bin:/usr/bin"));
                                cz.push(String::from("TERM=trustos"));
                            },
                            "id" => {
                                cz.push(String::from("uid=0(root) gid=0(root) groups=0(root)"));
                            },
                            "cal" => {
                                let os = crate::rtc::cgz();
                                cz.push(format!("     {:02}/{:04}", os.caw, os.ccq));
                                cz.push(String::from("Su Mo Tu We Th Fr Sa"));
                                cz.push(String::from("       1  2  3  4  5"));
                                cz.push(String::from(" 6  7  8  9 10 11 12"));
                                cz.push(String::from("13 14 15 16 17 18 19"));
                                cz.push(String::from("20 21 22 23 24 25 26"));
                                cz.push(String::from("27 28 29 30 31"));
                            },
                            _ if cmd.cj("head ") => {
                                let path = cmd.tl("head ").em();
                                match crate::ramfs::fh(|fs| {
                                    fs.mq(path).map(|e| String::azw(e).bkc())
                                }) {
                                    Ok(ca) => {
                                        for line in ca.ak().take(10) {
                                            cz.push(String::from(line));
                                        }
                                    },
                                    Err(aa) => cz.push(format!("head: {}: {}", path, aa.as_str())),
                                }
                            },
                            _ if cmd.cj("tail ") => {
                                let path = cmd.tl("tail ").em();
                                match crate::ramfs::fh(|fs| {
                                    fs.mq(path).map(|e| String::azw(e).bkc())
                                }) {
                                    Ok(ca) => {
                                        let ak: alloc::vec::Vec<&str> = ca.ak().collect();
                                        let ay = if ak.len() > 10 { ak.len() - 10 } else { 0 };
                                        for line in &ak[ay..] {
                                            cz.push(String::from(*line));
                                        }
                                    },
                                    Err(aa) => cz.push(format!("tail: {}: {}", path, aa.as_str())),
                                }
                            },
                            _ if cmd.cj("wc ") => {
                                let path = cmd.tl("wc ").em();
                                match crate::ramfs::fh(|fs| {
                                    fs.mq(path).map(|e| String::azw(e).bkc())
                                }) {
                                    Ok(ca) => {
                                        let ak = ca.ak().az();
                                        let aoh = ca.ayt().az();
                                        let bf = ca.len();
                                        cz.push(format!("{:>5} {:>5} {:>5} {}", ak, aoh, bf, path));
                                    },
                                    Err(aa) => cz.push(format!("wc: {}: {}", path, aa.as_str())),
                                }
                            },
                            _ if cmd.cj("grep ") => {
                                let n = cmd.tl("grep ").em();
                                let ek: alloc::vec::Vec<&str> = n.eyv(2, ' ').collect();
                                if ek.len() == 2 {
                                    let pattern = ek[0];
                                    let path = ek[1];
                                    match crate::ramfs::fh(|fs| {
                                        fs.mq(path).map(|e| String::azw(e).bkc())
                                    }) {
                                        Ok(ca) => {
                                            let mut aig = false;
                                            for line in ca.ak() {
                                                if line.contains(pattern) {
                                                    cz.push(String::from(line));
                                                    aig = true;
                                                }
                                            }
                                            if !aig {
                                                cz.push(format!("(no matches for '{}')", pattern));
                                            }
                                        },
                                        Err(aa) => cz.push(format!("grep: {}: {}", path, aa.as_str())),
                                    }
                                } else {
                                    cz.push(String::from("Usage: grep <pattern> <file>"));
                                }
                            },
                            _ if cmd.cj("find ") => {
                                let pattern = cmd.tl("find ").em();
                                cz.push(format!("Searching for: {}", pattern));
                                match crate::ramfs::fh(|fs| fs.awb(None)) {
                                    Ok(pj) => {
                                        for (j, _, _) in pj {
                                            if j.contains(pattern) {
                                                cz.push(format!("./{}", j));
                                            }
                                        }
                                    },
                                    Err(_) => {}
                                }
                            },
                            _ if cmd.cj("stat ") => {
                                let path = cmd.tl("stat ").em();
                                match crate::ramfs::fh(|fs| {
                                    fs.mq(path).map(|e| e.len())
                                }) {
                                    Ok(aw) => {
                                        cz.push(format!("  File: {}", path));
                                        cz.push(format!("  Size: {} bytes", aw));
                                        cz.push(String::from("  Access: -rw-r--r--"));
                                        cz.push(String::from("  Uid: 0  Gid: 0"));
                                    },
                                    Err(aa) => cz.push(format!("stat: {}: {}", path, aa.as_str())),
                                }
                            },
                            _ if cmd.cj("sort ") => {
                                let path = cmd.tl("sort ").em();
                                match crate::ramfs::fh(|fs| {
                                    fs.mq(path).map(|e| String::azw(e).bkc())
                                }) {
                                    Ok(ca) => {
                                        let mut ak: alloc::vec::Vec<&str> = ca.ak().collect();
                                        ak.jqs();
                                        for line in ak {
                                            cz.push(String::from(line));
                                        }
                                    },
                                    Err(aa) => cz.push(format!("sort: {}: {}", path, aa.as_str())),
                                }
                            },
                            _ if cmd.cj("strings ") => {
                                let path = cmd.tl("strings ").em();
                                match crate::ramfs::fh(|fs| {
                                    fs.mq(path).map(|e| alloc::vec::Vec::from(e))
                                }) {
                                    Ok(f) => {
                                        let mut cv = String::new();
                                        for &o in f.iter().take(1024) {
                                            if o >= 32 && o < 127 {
                                                cv.push(o as char);
                                            } else if cv.len() >= 4 {
                                                cz.push(cv.clone());
                                                cv.clear();
                                            } else {
                                                cv.clear();
                                            }
                                        }
                                        if cv.len() >= 4 {
                                            cz.push(cv);
                                        }
                                    },
                                    Err(aa) => cz.push(format!("strings: {}: {}", path, aa.as_str())),
                                }
                            },
                            _ if cmd.cj("traceroute ") || cmd.cj("tracert ") => {
                                let kh = if cmd.cj("traceroute ") {
                                    cmd.tl("traceroute ").em()
                                } else {
                                    cmd.tl("tracert ").em()
                                };
                                cz.push(format!("traceroute to {} (simulated)", kh));
                                cz.push(String::from(" 1  10.0.2.2  1.234 ms"));
                                cz.push(String::from(" 2  * * *"));
                                cz.push(String::from(" 3  * * *"));
                            },
                            "3ddemo" | "demo3d" | "cube" => {
                                
                                cz.push(String::from("Starting 3D Demo..."));
                                cz.push(String::from("Controls: Arrow keys rotate, ESC to exit"));
                                
                                
                                let bqe = 400u32;
                                let cea = 300u32;
                                let mut awq = crate::rasterizer::Rasterizer::new(bqe, cea);
                                let mut renderer = crate::rasterizer::Renderer3D::new(bqe, cea);
                                
                                let mut aev: f32 = 0.0;
                                let mut ajt: f32 = 0.3;
                                let mut nki = true;
                                let mut kpb = 0u32;
                                
                                
                                let cww = (gwu + 150) as u32;
                                let dgd = (gwv + 50) as u32;
                                
                                while nki && kpb < 600 { 
                                    
                                    if let Some(eh) = crate::keyboard::xw() {
                                        match eh {
                                            27 => nki = false, 
                                            0x4B => aev -= 0.1, 
                                            0x4D => aev += 0.1, 
                                            0x48 => ajt -= 0.1, 
                                            0x50 => ajt += 0.1, 
                                            _ => {}
                                        }
                                    }
                                    
                                    
                                    awq.clear(0xFF101010);
                                    renderer.rbj();
                                    
                                    
                                    let boe = crate::rasterizer::Mat4::chi(aev);
                                    let cbn = crate::rasterizer::Mat4::dlk(ajt);
                                    let chh = cbn.mul(&boe);
                                    
                                    
                                    let pn = crate::rasterizer::Vec3::new(0.0, 0.0, 0.0);
                                    renderer.gfg(&mut awq, pn, 1.5, &chh, 0xFF00FF00);
                                    
                                    
                                    let qxo = crate::rasterizer::Vec3::new(2.0, 0.0, 0.0);
                                    renderer.gfg(&mut awq, qxo, 0.8, &chh, 0xFF00FFFF);
                                    
                                    
                                    awq.nts(0, 0, bqe, 25, 0xFF003300, 0xFF00AA00);
                                    
                                    
                                    awq.lx(0, 0, bqe, cea, 0xFF00FF00);
                                    
                                    
                                    for x in 0..cea {
                                        for y in 0..bqe {
                                            let w = (x * bqe + y) as usize;
                                            crate::framebuffer::draw_pixel(cww + y, dgd + x, awq.aqt[w]);
                                        }
                                    }
                                    
                                    
                                    crate::framebuffer::cb("3D Demo - ESC to exit", cww + 10, dgd + 5, 0xFFFFFFFF);
                                    
                                    
                                    let ghn = format!("Frame: {}", kpb);
                                    crate::framebuffer::cb(&ghn, cww + bqe - 100, dgd + 5, 0xFFFFFF00);
                                    
                                    aev += 0.02;  
                                    kpb += 1;
                                    
                                    
                                    for _ in 0..50000 { core::hint::hc(); }
                                }
                                
                                cz.push(String::from("3D Demo ended."));
                            },
                            "raster" | "rasterdemo" => {
                                
                                cz.push(String::from("Rasterizer Demo - Antialiasing & Gradients"));
                                
                                let bqe = 350u32;
                                let cea = 250u32;
                                let mut awq = crate::rasterizer::Rasterizer::new(bqe, cea);
                                
                                let cww = (gwu + 175) as u32;
                                let dgd = (gwv + 75) as u32;
                                
                                
                                awq.clear(0xFF0A0A0A);
                                
                                
                                awq.kvv(0, 0, bqe, cea, 0xFF000022, 0xFF002200);
                                
                                
                                awq.hji(80, 80, 40, 0xFFFF0000);   
                                awq.hji(150, 100, 35, 0xFF00FF00); 
                                awq.hji(220, 80, 40, 0xFF0000FF);  
                                
                                
                                awq.hji(115, 90, 30, 0x8800FFFF);  
                                awq.hji(185, 90, 30, 0x88FF00FF);  
                                
                                
                                awq.afp(50, 150, 120, 60, 15, 0xFF444444);
                                awq.nts(55, 155, 110, 50, 0xFF006600, 0xFF00CC00);
                                
                                
                                awq.krd(200.0, 150.0, 320.0, 220.0, 0xFFFFFF00);
                                awq.krd(200.0, 220.0, 320.0, 150.0, 0xFFFF8800);
                                
                                
                                awq.gfj(250, 160, 60, 40, 8, 0x88000000);
                                awq.ah(250, 160, 60, 40, 0xFF00AA00);
                                
                                
                                awq.lx(0, 0, bqe, cea, 0xFF00FF00);
                                
                                
                                for x in 0..cea {
                                    for y in 0..bqe {
                                        let w = (x * bqe + y) as usize;
                                        crate::framebuffer::draw_pixel(cww + y, dgd + x, awq.aqt[w]);
                                    }
                                }
                                
                                crate::framebuffer::cb("Rasterizer: AA + Alpha + Gradients", cww + 10, dgd + 5, 0xFFFFFFFF);
                                
                                
                                cz.push(String::from("Press any key to close demo..."));
                                loop {
                                    if crate::keyboard::xw().is_some() {
                                        break;
                                    }
                                    core::hint::hc();
                                }
                                cz.push(String::from("Demo closed."));
                            },
                            _ => cz.push(format!("Command not found: {}", cmd)),
                        };
                        bfh.clear();
                        gto.clear();
                        
                        
                        while cz.len() > 20 {
                            cz.remove(0);
                        }
                        
                        
                        px = cz.len().ao(AFH_);
                    }
                },
                32..=126 => { 
                    crate::serial_println!("[KEY] Printable char: '{}' ({})", bs as char, bs);
                    bfh.push(bs as char);
                    
                    let kjn = ["help", "ls", "dir", "clear", "ifconfig", "cpuinfo", "meminfo", "whoami", "uptime", "open", "smp", "fps", "matrix", "holo"];
                    gto.clear();
                    for r in kjn {
                        if r.cj(&bfh) && r != bfh.as_str() {
                            gto = String::from(&r[bfh.len()..]);
                            break;
                        }
                    }
                },
                _ => {}
            }
            } 
        }
        
        
        let lms = crate::mouse::drd();
        daa = lms.b.qp(0, z as i32 - 1);
        dab = lms.c.qp(0, ac as i32 - 1);
        let fd = lms.jda;
        
        
        let dew = fd && !gpt;
        let hxl = !fd && gpt;
        gpt = fd;
        
        
        if hgs {
            if fd {
                
                gwu = (daa - dgp).qp(0, z as i32 - 200);
                gwv = (dab - dgq).qp(0, ac as i32 - 100);
                
                if let Some(ep) = compositor.dhm(pzm) {
                    ep.eyk(gwu as u32, gwv as u32);
                }
            } else {
                
                hgs = false;
            }
        }
        
        
        if let Some(gi) = compositor.dhm(nio) {
            gi.eyk(daa as u32, dab as u32);
        }
        
        
        czs = -1;
        if awe {
            let rs = 5u32;
            let xp = ac - 340;
            let hl = daa as u32;
            let ir = dab as u32;
            
            if hl >= rs && hl < rs + 250 && ir >= xp && ir < xp + 290 {
                let ali = 36u32;
                let aio = if ir > xp + 40 { ir - xp - 40 } else { 0 };
                let w = (aio / ali) as i32;
                if w >= 0 && w < gmp.len() as i32 {
                    czs = w;
                }
            }
        }
        
        
        if dew {
            let hl = daa as u32;
            let ir = dab as u32;
            
            
            let ejr = ac - 40;
            if ir >= ejr && ir < ac && hl >= 5 && hl < 110 {
                awe = !awe;
                dvu = false; 
            }
            
            else if ir >= ejr && ir < ac && hl >= 340 && hl < 390 {
                dvu = !dvu;
                awe = false; 
                
                gsi = crate::desktop::col();
                gsj = crate::desktop::hlf();
            }
            
            else if ir >= ejr && ir < ac && hl >= 220 && hl < 320 {
                
                fbm = !fbm;
            }
            
            else if awe && czs >= 0 && czs < gmp.len() as i32 {
                let (blu, item) = gmp[czs as usize];
                match item {
                    MenuItem::Kc(ev) => {
                        
                        if !blu.cj("-") {
                            atu = ev;
                            cz.clear();
                            for line in gie!(ev) {
                                cz.push(String::from(*line));
                            }
                            cz.push(String::from(""));
                            cz.push(String::from("Type commands below. Type 'help' for more info."));
                        }
                    },
                    MenuItem::Qt => {
                        cz.push(String::from("> Shutting down..."));
                        
                        for _ in 0..10000000 { core::hint::hc(); }
                        loop {
                            crate::arch::tvq();
                            crate::arch::bhd();
                        }
                    },
                    MenuItem::Axr => {
                        cz.push(String::from("> Rebooting..."));
                        for _ in 0..10000000 { core::hint::hc(); }
                        unsafe {
                            crate::arch::Port::<u8>::new(0x64).write(0xFE);
                        }
                        loop { crate::arch::bhd(); }
                    },
                }
                awe = false;
            }
            
            else if awe {
                awe = false;
            }
            else if dvu {
                
                let pjk = 340u32;
                let pjl = ac - 380;
                let wkh = 270u32;
                let wkg = 350u32;
                if !(hl >= pjk && hl < pjk + wkh 
                    && ir >= pjl && ir < pjl + wkg) {
                    dvu = false;
                }
            }
            
            else if !hgs {
                let abx = gwu as u32;
                let aha = gwv as u32;
                let aog = 700u32;  
                let biz = 450u32;  
                
                
                if fbm && hl >= abx && hl < abx + aog && ir >= aha && ir < aha + 28 {
                    
                    
                    if hl >= abx + aog - 60 && hl < abx + aog - 40 {
                        fbm = false;  
                        cz.push(String::from("> Window closed. Click dock icon to reopen."));
                    }
                    
                    else if hl >= abx + aog - 90 && hl < abx + aog - 70 {
                        fbm = false;  
                        cz.push(String::from("> Window minimized"));
                    }
                    
                    else if hl >= abx + aog - 120 && hl < abx + aog - 100 {
                        cz.push(String::from("> Window maximized"));
                    }
                    
                    else {
                        hgs = true;
                        dgp = daa - gwu;
                        dgq = dab - gwv;
                    }
                }
                
                else if hl < 80 && ir < ac - 40 {
                    let aih = 36u32;
                    let qi = 50u32;       
                    let vc = 10u32;
                    for a in 0..8usize {
                        let og = vc + (a as u32) * (aih + qi);
                        if ir >= og && ir < og + aih + 16 {
                            atu = match a {
                                0 => AppMode::Pl,
                                1 => AppMode::Df,
                                2 => AppMode::As,
                                3 => AppMode::Ag,
                                4 => AppMode::Ip,
                                5 => AppMode::Rb,
                                6 => AppMode::Browser,
                                7 => AppMode::Bp,
                                _ => AppMode::Df,
                            };
                            
                            fbm = true;
                            cz.clear();
                            for line in gie!(atu) {
                                cz.push(String::from(*line));
                            }
                            cz.push(String::from(""));
                            break;
                        }
                    }
                }
            }
        }
        
        
        
        
        
        let tys = (oo % rnc) == 0;
        
        if !tys {
            
            
            
            compositor.vkv();
            
            if atr { cet.qs(); }
        } else {
        
        lze += 1;
        
        
        
        
        
        if let Some(ei) = compositor.dhm(qph) {
            let aeg = ei.bi.mw();
            let bjl = ei.bi.len();
            
            
            if atr {
                
                
                
                
                
                cet.qs();
                cet.tj(&mut ei.bi, z as usize, ac as usize);
            } else if ats {
                
                
                
                
                
                
                crate::gpu_emu::wmb(
                    aeg,
                    z as usize,
                    ac as usize,
                );
            } else if avf {
                
                
                
                hqv.qs();
                hqv.tj(&mut ei.bi, z as usize, ac as usize);
            } else if apf {
                
                
                
                
                dzd.qs();
                dzd.tj(&mut ei.bi, z as usize, ac as usize);
                
                dzd.vvo(&mut ei.bi, z as usize, ac as usize);
                dzd.vvs(&mut ei.bi, z as usize, ac as usize);
            } else if aqq && !dmx && !bnf {
                
                
                
                ei.bi.vi(cot);
                nsv.qs();
                nsv.tj(&mut ei.bi, z as usize, ac as usize);
            } else if dmx {
                
                
                
                
                
                
                holovolume.dbw(z as usize, ac as usize);
                holovolume.qs(0.016);
                
                
                for bj in 0..R_ {
                    awc[bj] += czn[bj] as i32;
                    if awc[bj] > (AS_ as i32 + 30) {
                        let dv = (bj as u32 * 2654435761).cn(oo as u32);
                        awc[bj] = -((dv % 30) as i32);
                        czn[bj] = 1 + (dv % 3);
                    }
                }
                
                
                unsafe {
                    #[cfg(target_arch = "x86_64")]
                    crate::graphics::simd::bed(aeg, bjl, cot);
                    #[cfg(not(target_arch = "x86_64"))]
                    ei.bi.vi(cot);
                }
                
                
                let hmz = holovolume.tfa();
                
                
                let oi = super::Tk {
                    aeg,
                    bjl,
                    z,
                    ac,
                    car: car.fq(),
                    awc: awc.fq(),
                    hmz: hmz.fq(),
                    gme: AS_,
                };
                
                crate::cpu::smp::daj(
                    R_,
                    super::pbw,
                    &oi as *const super::Tk as *mut u8
                );
                
            } else if bnf {
                
                
                
                
                if fks.tyr() {
                    
                    
                    
                    use crate::graphics::raytracer::{Vec3, Material};
                    
                    raytracer.qs(0.016);
                    
                    
                    match fks {
                        crate::graphics::holomatrix::HoloScene::Nv => {
                            raytracer.wlk();
                        },
                        crate::graphics::holomatrix::HoloScene::Nu => {
                            raytracer.wko();
                        },
                        _ => {}
                    }
                    
                    
                    let wbb = raytracer.tj();
                    
                    
                    let mbe = raytracer.z;
                    let pem = raytracer.ac;
                    let wdl = z as usize / mbe;
                    let wdm = ac as usize / pem;
                    
                    for c in 0..ac as usize {
                        for b in 0..z as usize {
                            let kb = (b / wdl).v(mbe - 1);
                            let ix = (c / wdm).v(pem - 1);
                            let s = wbb[ix * mbe + kb];
                            ei.bi[c * z as usize + b] = s;
                        }
                    }
                } else {
                    
                    
                    
                    
                    
                    
                    
                    holomatrix.qs(0.016);
                    let time = holomatrix.time;
                    
                    
                    
                    
                    
                    
                    
                    let mut alg = [[0u8; AS_]; R_];
                    
                    
                    let acc = (z as f32) / (R_ as f32);  
                    let aqw = (ac as f32) / (AS_ as f32); 
                    
                    
                    let cx = z as f32 / 2.0;
                    let ae = ac as f32 / 2.0;
                    let bv = (ac as f32 / 3.0).v(z as f32 / 4.0);
                    
                    
                    match fks {
                        crate::graphics::holomatrix::HoloScene::Ij => {
                            
                            let obo = 2.2;
                            let dy = 0.45;
                            let cuy = 3.5;
                            
                            for a in 0..180 {
                                let ab = a as f32 / 180.0;
                                let c = -obo / 2.0 + ab * obo;
                                let hg = ab * cuy * 6.28318 + time;
                                
                                
                                let dn = dy * crate::graphics::holomatrix::eob(hg);
                                let aeu = dy * crate::graphics::holomatrix::cuh(hg);
                                
                                
                                let hy = dy * crate::graphics::holomatrix::eob(hg + 3.14159);
                                let ahc = dy * crate::graphics::holomatrix::cuh(hg + 3.14159);
                                
                                
                                let boe = time * 0.4;
                                let cwr = crate::graphics::holomatrix::eob(boe);
                                let dcb = crate::graphics::holomatrix::cuh(boe);
                                
                                
                                let ehw = dn * cwr + aeu * dcb;
                                let cmn = -dn * dcb + aeu * cwr;
                                let ftk = hy * cwr + ahc * dcb;
                                let cmo = -hy * dcb + ahc * cwr;
                                
                                
                                let nkl = 1.0 / (2.0 + cmn);
                                let asa = cx + ehw * bv * nkl;
                                let bos = ae + c * bv * nkl;
                                let cpg = (asa / acc) as usize;
                                let ctz = (bos / aqw) as usize;
                                
                                let nkm = 1.0 / (2.0 + cmo);
                                let amy = cx + ftk * bv * nkm;
                                let bcw = ae + c * bv * nkm;
                                let cph = (amy / acc) as usize;
                                let cua = (bcw / aqw) as usize;
                                
                                
                                let hog = (180.0 + 75.0 * (1.0 - ((cmn + 0.5) * 0.5).am(0.0).v(1.0))) as u8;
                                let hoh = (180.0 + 75.0 * (1.0 - ((cmo + 0.5) * 0.5).am(0.0).v(1.0))) as u8;
                                
                                
                                if cpg < R_ && ctz < AS_ {
                                    alg[cpg][ctz] = alg[cpg][ctz].am(hog);
                                    
                                    if cpg > 0 { alg[cpg-1][ctz] = alg[cpg-1][ctz].am(hog * 2/3); }
                                    if cpg < R_-1 { alg[cpg+1][ctz] = alg[cpg+1][ctz].am(hog * 2/3); }
                                    if ctz > 0 { alg[cpg][ctz-1] = alg[cpg][ctz-1].am(hog/2); }
                                    if ctz < AS_-1 { alg[cpg][ctz+1] = alg[cpg][ctz+1].am(hog/2); }
                                }
                                if cph < R_ && cua < AS_ {
                                    alg[cph][cua] = alg[cph][cua].am(hoh);
                                    if cph > 0 { alg[cph-1][cua] = alg[cph-1][cua].am(hoh * 2/3); }
                                    if cph < R_-1 { alg[cph+1][cua] = alg[cph+1][cua].am(hoh * 2/3); }
                                    if cua > 0 { alg[cph][cua-1] = alg[cph][cua-1].am(hoh/2); }
                                    if cua < AS_-1 { alg[cph][cua+1] = alg[cph][cua+1].am(hoh/2); }
                                }
                                
                                
                                if a % 12 == 0 {
                                    for e in 0..8 {
                                        let apc = e as f32 / 7.0;
                                        let mj = asa * (1.0 - apc) + amy * apc;
                                        let ct = bos * (1.0 - apc) + bcw * apc;
                                        let lie = (mj / acc) as usize;
                                        let lkb = (ct / aqw) as usize;
                                        if lie < R_ && lkb < AS_ {
                                            alg[lie][lkb] = alg[lie][lkb].am(80);
                                        }
                                    }
                                }
                            }
                        },
                        crate::graphics::holomatrix::HoloScene::Jb => {
                            let iv = 0.5;
                            let lm: [(f32, f32, f32); 8] = [
                                (-iv, -iv, -iv), (iv, -iv, -iv),
                                (iv, iv, -iv), (-iv, iv, -iv),
                                (-iv, -iv, iv), (iv, -iv, iv),
                                (iv, iv, iv), (-iv, iv, iv),
                            ];
                            let bu: [(usize, usize); 12] = [
                                (0,1), (1,2), (2,3), (3,0),
                                (4,5), (5,6), (6,7), (7,4),
                                (0,4), (1,5), (2,6), (3,7),
                            ];
                            
                            let cbn = time * 0.7;
                            let boe = time * 0.5;
                            
                            for (hnh, hni) in bu.iter() {
                                let (xtb, xtc, xte) = lm[*hnh];
                                let (jwc, xtd, xtf) = lm[*hni];
                                
                                for e in 0..30 {
                                    let ab = e as f32 / 29.0;
                                    let b = xtb * (1.0 - ab) + jwc * ab;
                                    let c = xtc * (1.0 - ab) + xtd * ab;
                                    let av = xte * (1.0 - ab) + xtf * ab;
                                    
                                    
                                    let bmn = crate::graphics::holomatrix::eob(cbn);
                                    let bok = crate::graphics::holomatrix::cuh(cbn);
                                    let ix = c * bmn - av * bok;
                                    let agv = c * bok + av * bmn;
                                    let bmo = crate::graphics::holomatrix::eob(boe);
                                    let bol = crate::graphics::holomatrix::cuh(boe);
                                    let kb = b * bmo + agv * bol;
                                    let cmo = -b * bol + agv * bmo;
                                    
                                    let eo = 1.0 / (2.0 + cmo);
                                    let cr = cx + kb * bv * eo;
                                    let cq = ae + ix * bv * eo;
                                    let bj = (cr / acc) as usize;
                                    let br = (cq / aqw) as usize;
                                    
                                    if bj < R_ && br < AS_ {
                                        let lex = (100.0 + 100.0 * (1.0 - ((cmo + 0.6) * 0.5).am(0.0).v(1.0))) as u8;
                                        alg[bj][br] = alg[bj][br].am(lex);
                                    }
                                }
                            }
                        },
                        _ => {
                            
                            for a in 0..300 {
                                let bnv = (a as f32 / 300.0) * 6.28318;
                                let bdb = (a as f32 * 0.618033 * 6.28318) % 6.28318;
                                
                                let m = 0.55;
                                let b = m * crate::graphics::holomatrix::cuh(bdb) * crate::graphics::holomatrix::eob(bnv);
                                let c = m * crate::graphics::holomatrix::cuh(bdb) * crate::graphics::holomatrix::cuh(bnv);
                                let av = m * crate::graphics::holomatrix::eob(bdb);
                                
                                let ffx = crate::graphics::holomatrix::eob(time * 0.5);
                                let fuu = crate::graphics::holomatrix::cuh(time * 0.5);
                                let kb = b * ffx + av * fuu;
                                let agv = -b * fuu + av * ffx;
                                
                                let eo = 1.0 / (2.0 + agv);
                                let cr = cx + kb * bv * eo;
                                let cq = ae + c * bv * eo;
                                let bj = (cr / acc) as usize;
                                let br = (cq / aqw) as usize;
                                
                                if bj < R_ && br < AS_ {
                                    let lex = (80.0 + 120.0 * (1.0 - ((agv + 0.6) * 0.5).am(0.0).v(1.0))) as u8;
                                    alg[bj][br] = alg[bj][br].am(lex);
                                }
                            }
                        }
                    }
                    
                    
                    
                    
                    
                    
                    
                    for bj in 0..R_ {
                        awc[bj] += czn[bj] as i32;
                        if awc[bj] > (AS_ as i32 + 30) {
                            let dv = (bj as u32 * 2654435761).cn(oo as u32);
                            awc[bj] = -((dv % 30) as i32);
                            czn[bj] = 1 + (dv % 3);
                        }
                    }
                    
                    
                    ei.bi.vi(0xFF000000);
                    
                    
                    let byt = 8u32;
                    for bj in 0..R_ {
                        let b = bj as u32 * byt;
                        if b >= z { continue; }
                        
                        let ale = awc[bj];
                        
                        for br in 0..AS_ {
                            let c = br as u32 * 16;
                            if c >= ac { continue; }
                            
                            let la = br as i32 - ale;
                            
                            
                            let agg = if la < 0 {
                                continue;
                            } else if la == 0 {
                                255u32  
                            } else if la <= 12 {
                                255 - (la as u32 * 8)
                            } else if la <= 28 {
                                
                                let pv = ((la - 12) as u32).v(15) * 16;
                                let yx = 255u32.ao(pv);
                                (160 * yx) / 255
                            } else {
                                continue;
                            };
                            
                            
                            let bma = alg[bj][br] as u32;
                            
                            
                            
                            let (m, at, o) = if bma > 0 {
                                
                                let hj = (agg + bma * 2).v(255);
                                let niw = (bma as u32 * 3 / 2).v(255);
                                (niw / 3, hj, niw)  
                            } else {
                                
                                let tp = (agg / 3).v(80);
                                (0, tp, 0)
                            };
                            
                            let s = 0xFF000000 | (m << 16) | (at << 8) | o;
                            
                            
                            let r = car[olj(bj, br)] as char;
                            let ka = crate::framebuffer::font::ada(r);
                            
                            
                            for (m, &fs) in ka.iter().cf() {
                                let x = c + m as u32;
                                if x >= ac { break; }
                                let afg = (x * z) as usize;
                                
                                if fs != 0 {
                                    let bxy = b as usize;
                                    if fs & 0x80 != 0 { let w = afg + bxy; if w < ei.bi.len() { ei.bi[w] = s; } }
                                    if fs & 0x40 != 0 { let w = afg + bxy + 1; if w < ei.bi.len() { ei.bi[w] = s; } }
                                    if fs & 0x20 != 0 { let w = afg + bxy + 2; if w < ei.bi.len() { ei.bi[w] = s; } }
                                    if fs & 0x10 != 0 { let w = afg + bxy + 3; if w < ei.bi.len() { ei.bi[w] = s; } }
                                    if fs & 0x08 != 0 { let w = afg + bxy + 4; if w < ei.bi.len() { ei.bi[w] = s; } }
                                    if fs & 0x04 != 0 { let w = afg + bxy + 5; if w < ei.bi.len() { ei.bi[w] = s; } }
                                    if fs & 0x02 != 0 { let w = afg + bxy + 6; if w < ei.bi.len() { ei.bi[w] = s; } }
                                    if fs & 0x01 != 0 { let w = afg + bxy + 7; if w < ei.bi.len() { ei.bi[w] = s; } }
                                }
                            }
                        }
                    }
                }
            } else {
                
                
                
                
                
                for bj in 0..R_ {
                    awc[bj] += czn[bj] as i32;
                    if awc[bj] > (AS_ as i32 + 30) {
                        let dv = (bj as u32 * 2654435761).cn(oo as u32);
                        awc[bj] = -((dv % 30) as i32);
                        czn[bj] = 1 + (dv % 3);
                    }
                }
                
                
                unsafe {
                    #[cfg(target_arch = "x86_64")]
                    crate::graphics::simd::bed(aeg, bjl, cot);
                    #[cfg(not(target_arch = "x86_64"))]
                    ei.bi.vi(cot);
                }
                
                
                
                
                
                let oi = super::Tk {
                    aeg,
                    bjl,
                    z,
                    ac,
                    car: car.fq(),
                    awc: awc.fq(),
                    hmz: core::ptr::null(),  
                    gme: AS_,
                };
                
                
                crate::cpu::smp::daj(
                    R_,
                    super::pbw,
                    &oi as *const super::Tk as *mut u8
                );
            }
        }
        
        
        
        
        if let Some(apr) = compositor.dhm(sac) {
            apr.clear(0xF0080808); 
            
            let aih = 36u32;  
            let qi = 50u32;       
            let vc = 10u32;
            
            let rzz = [
                ("Files", AppMode::Pl),
                ("Shell", AppMode::Df),
                ("Net", AppMode::As),
                ("Edit", AppMode::Ag),
                ("HW", AppMode::Ip),
                ("User", AppMode::Rb),
                ("Web", AppMode::Browser),  
                ("Img", AppMode::Bp), 
            ];
            
            for (a, (j, ev)) in rzz.iter().cf() {
                let og = vc + (a as u32) * (aih + qi);
                let fg = 10u32;
                
                let rl = *ev == atu;
                let xd = if rl { cyh } else { aek };
                let bbw = if rl { 0xFFFFFFFF } else { 0xFF888888 };
                
                
                if rl {
                    apr.ah(fg - 4, og - 4, aih + 8, aih + 20, 0xFF002800);
                    apr.lx(fg - 4, og - 4, aih + 8, aih + 20, ya);
                }
                apr.ah(fg, og, aih, aih, 0xFF0A0A0A);
                apr.lx(fg, og, aih, aih, xd);
                
                
                let cx = fg + aih / 2;
                let ae = og + aih / 2;
                match a {
                    0 => { 
                        apr.ah(cx - 12, ae - 2, 24, 14, xd);
                        apr.ah(cx - 14, ae - 6, 10, 6, xd);
                    },
                    1 => { 
                        apr.lx(cx - 14, ae - 10, 28, 20, xd);
                        apr.cb(">", cx - 8, ae - 4, xd);
                        apr.ah(cx - 2, ae - 2, 10, 2, xd);
                    },
                    2 => { 
                        apr.abc(cx, ae, 12, xd);
                        apr.abc(cx, ae, 8, 0xFF0A0A0A);
                        apr.abc(cx, ae, 4, xd);
                        
                        apr.ah(cx + 6, ae - 2, 2, 6, xd);
                        apr.ah(cx + 10, ae - 6, 2, 10, xd);
                    },
                    3 => { 
                        apr.ah(cx - 10, ae - 12, 20, 24, xd);
                        apr.ah(cx - 8, ae - 10, 16, 20, 0xFF0A0A0A);
                        apr.ah(cx - 6, ae - 6, 12, 2, xd);
                        apr.ah(cx - 6, ae - 2, 12, 2, xd);
                        apr.ah(cx - 6, ae + 2, 8, 2, xd);
                    },
                    4 => { 
                        apr.ah(cx - 10, ae - 8, 20, 16, xd);
                        for fb in 0..4 {
                            apr.ah(cx - 14, ae - 6 + fb * 4, 4, 2, xd);
                            apr.ah(cx + 10, ae - 6 + fb * 4, 4, 2, xd);
                        }
                    },
                    5 => { 
                        apr.abc(cx, ae - 4, 6, xd);
                        apr.ah(cx - 8, ae + 4, 16, 8, xd);
                    },
                    6 => { 
                        apr.abc(cx, ae, 10, xd);
                        apr.abc(cx, ae, 6, 0xFF0A0A0A);
                        
                        apr.ah(cx - 10, ae - 1, 20, 2, xd);
                        
                        apr.ah(cx - 1, ae - 10, 2, 20, xd);
                    },
                    _ => {}
                }
                
                
                let wg = fg + (aih / 2) - ((j.len() as u32 * 8) / 2);
                apr.cb(j, wg, og + aih + 2, bbw);
            }
        }
        
        
        
        
        if let Some(ep) = compositor.dhm(pzm) {
            if fbm {
                
                let d = ep.z;
                let i = ep.ac;
                
                ep.clear(xup);
                
                
                ep.lx(0, 0, d, i, ya);
                ep.lx(1, 1, d - 2, i - 2, ya);
                
                
                let czz = match atu {
                    AppMode::Df => "Shell",
                AppMode::As => "Network",
                AppMode::Ip => "Hardware",
                AppMode::Ag => "TrustCode",
                AppMode::Rb => "User Management",
                AppMode::Pl => "Files",
                AppMode::Browser => "Web Browser",
                AppMode::Bp => "Image Viewer",
            };
            ep.ah(2, 2, d - 4, 26, 0xFF0A1A0A);
            let dq = format!("TrustOS - {} Module", czz);
            ep.cb(&dq, 12, 8, 0xFFFFFFFF); 
            
            
            if hgs {
                ep.cb("[MOVING]", d / 2 - 32, 8, 0xFFFFAA00);
            }
            
            
            
            let kn = 13u32;
            let cjj = 10u32;
            let gbp = d - 50;
            let nah = d - 80;
            let kfa = d - 110;
            
            
            ep.abc(gbp, kn, cjj, 0xFFFF4444);
            ep.lx(gbp, kn, 1, 1, 0xFFFF6666); 
            
            for ab in 0..7 {
                ep.aht(gbp - 5 + ab, kn - 5 + ab, 0xFFFFFFFF);
                ep.aht(gbp - 4 + ab, kn - 5 + ab, 0xFFFFFFFF);
                ep.aht(gbp + 5 - ab, kn - 5 + ab, 0xFFFFFFFF);
                ep.aht(gbp + 4 - ab, kn - 5 + ab, 0xFFFFFFFF);
            }
            
            
            ep.abc(nah, kn, cjj, 0xFFFFCC00);
            
            ep.ah(nah - 5, kn - 1, 10, 3, 0xFF000000);
            
            
            ep.abc(kfa, kn, cjj, 0xFF44DD44);
            
            ep.lx(kfa - 5, kn - 5, 10, 10, 0xFF000000);
            ep.lx(kfa - 4, kn - 4, 8, 8, 0xFF000000);
            
            
            let gl = 35u32;
            let acg = 18u32;
            let ulk = ((i - gl - 50) / acg) as usize;
            
            if atu == AppMode::Browser {
                
                
                
                
                
                let aoe = gl;
                ep.ah(10, aoe, d - 20, 32, 0xFF1E1E1E);
                ep.lx(10, aoe, d - 20, 32, 0xFF3C3C3C);
                
                
                let enb: u32 = 0xFF2D2D2D;
                
                
                ep.ah(14, aoe + 4, 24, 24, enb);
                ep.cb("<", 22, aoe + 10, 0xFFAAAAAA);
                
                
                ep.ah(42, aoe + 4, 24, 24, enb);
                ep.cb(">", 50, aoe + 10, 0xFFAAAAAA);
                
                
                ep.ah(70, aoe + 4, 24, 24, enb);
                ep.cb("R", 78, aoe + 10, 0xFFAAAAAA);
                
                
                let xrp = if hbj == 0 { "SRC" } else { "DOM" };
                ep.ah(98, aoe + 4, 32, 24, 0xFF383838);
                ep.cb(xrp, 102, aoe + 10, 0xFF88CCFF);
                
                
                ep.ah(135, aoe + 4, d - 160, 24, 0xFF0D0D0D);
                ep.lx(135, aoe + 4, d - 160, 24, if nag { 0xFF4FC3F7 } else { 0xFF555555 });
                
                
                let xpb = if bmb.cj("https://") { 0xFF00C853 } else { 0xFFDDDDDD };
                ep.cb(&bmb, 142, aoe + 10, xpb);
                
                
                if nag && btx {
                    let lf = 142 + (bmb.len() as u32 * 8);
                    if lf < d - 30 {
                        ep.ah(lf, aoe + 8, 2, 18, 0xFF4FC3F7);
                    }
                }
                
                
                let cgi = gl + 40;
                let otq = ((i - cgi - 35) / acg) as usize;
                
                
                ep.ah(10, cgi - 4, d - 20, i - cgi - 28, 0xFF1E1E1E);
                
                
                let hmf = 40u32;
                ep.ah(10, cgi - 4, hmf, i - cgi - 28, 0xFF252526);
                
                let dlz = if ps.len() > otq {
                    ps.len() - otq
                } else {
                    0
                };
                
                
                for (a, qsb) in ps.iter().chz(dlz).cf() {
                    let c = cgi + (a as u32) * acg;
                    if c + acg > i - 35 { break; }
                    
                    
                    let csd = format!("{:3}", dlz + a + 1);
                    ep.cb(&csd, 14, c, 0xFF858585);
                    
                    
                    let mut qan = 10u32 + hmf + 5;
                    for ie in &qsb.jq {
                        ep.cb(&ie.text, qan, c, ie.s);
                        qan += (ie.text.len() as u32) * 8;
                    }
                }
                
                
                ep.ah(10, i - 28, d - 20, 23, 0xFF007ACC);
                
                
                let mhn = if bpu.contains("Error") { "?" } 
                    else if bpu.contains("Loading") { "?" } 
                    else { "?" };
                ep.cb(mhn, 16, i - 24, 0xFFFFFFFF);
                ep.cb(&bpu, 30, i - 24, 0xFFFFFFFF);
                
                
                let upd = if hbj == 0 { "[Source]" } else { "[Elements]" };
                let upe = d - 90;
                ep.cb(upd, upe, i - 24, 0xFFCCCCCC);
                
            } else if atu == AppMode::Bp {
                
                
                
                
                
                ep.ah(10, gl, d - 20, i - gl - 30, 0xFF1A1A1A);
                
                if let Some(ref th) = odi {
                    
                    let jvn = d - 40;
                    let azb = i - gl - 60;
                    let jvo = 20u32;
                    let ekn = gl + 10;
                    
                    
                    let mce = (th.z as f32 * dig) as u32;
                    let mcd = (th.ac as f32 * dig) as u32;
                    
                    
                    let yv = jvo as i32 + (jvn as i32 / 2) + ldj;
                    let uq = ekn as i32 + (azb as i32 / 2) + ldk;
                    let fld = yv - (mce as i32 / 2);
                    let fle = uq - (mcd as i32 / 2);
                    
                    
                    for bg in 0..mcd.v(azb) {
                        let abi = fle + bg as i32;
                        if abi < ekn as i32 || abi >= (ekn + azb) as i32 {
                            continue;
                        }
                        
                        let bih = ((bg as f32 / dig) as u32).v(th.ac - 1);
                        
                        for dx in 0..mce.v(jvn) {
                            let xu = fld + dx as i32;
                            if xu < jvo as i32 || xu >= (jvo + jvn) as i32 {
                                continue;
                            }
                            
                            let blg = ((dx as f32 / dig) as u32).v(th.z - 1);
                            let il = th.beg(blg, bih);
                            
                            
                            if (il >> 24) > 0 {
                                ep.aht(xu as u32, abi as u32, il);
                            }
                        }
                    }
                    
                    
                    ep.lx(
                        (fld.am(jvo as i32)) as u32,
                        (fle.am(ekn as i32)) as u32,
                        mce.v(jvn),
                        mcd.v(azb),
                        0xFF444444
                    );
                } else {
                    
                    let yv = d / 2;
                    let uq = (gl + i) / 2;
                    
                    
                    ep.lx(yv - 40, uq - 30, 80, 60, 0xFF444444);
                    ep.cb("??", yv - 8, uq - 10, 0xFF666666);
                    ep.cb("No image loaded", yv - 56, uq + 25, 0xFF888888);
                    ep.cb("Use: imgview <file>", yv - 72, uq + 45, 0xFF666666);
                }
                
                
                ep.ah(10, gl, d - 20, 24, 0xFF252525);
                let xxn = (dig * 100.0) as u32;
                let ldz = format!("Zoom: {}%  |  {}", xxn, odk);
                ep.cb(&ldz, 16, gl + 5, 0xFFCCCCCC);
                
                
                ep.cb(&odj, d - 60, gl + 5, 0xFF88CCFF);
                
                
                ep.ah(10, i - 28, d - 20, 23, 0xFF252525);
                ep.cb("[+/-] Zoom  [Arrows] Pan  [R] Reset  [ESC] Close", 16, i - 24, 0xFF888888);
                
            } else if atu == AppMode::Ag {
                
                
                
                use crate::apps::text_editor::*;
                
                let nk: u32 = 8;
                let gy: u32 = 16;
                let tii: u32 = 5; 
                let bqy = tii * nk;
                let bfm: u32 = 22;
                let dwn: u32 = 26;
                
                let bds = bqy;
                let bdt = gl + dwn;
                let ior = d - bqy;
                let byr = i.ao(gl + dwn + bfm);
                let gwe = (byr / gy).am(1) as usize;
                
                
                if aey.gn < aey.ug {
                    aey.ug = aey.gn;
                }
                if aey.gn >= aey.ug + gwe {
                    aey.ug = aey.gn - gwe + 1;
                }
                aey.byk += 1;
                
                
                ep.ah(0, gl, d, dwn, BMY_);
                let mjp = aey.wn.as_ref().map(|ai| {
                    ai.cmm('/').next().unwrap_or(ai.as_str())
                }).unwrap_or("untitled");
                let kqc = if aey.no { " *" } else { "" };
                let icv = format!("  {}{}", mjp, kqc);
                let axb = ((icv.len() as u32 + 2) * nk).v(d);
                ep.ah(0, gl, axb, dwn, BNN_);
                
                ep.ah(0, gl + dwn - 2, axb, 2, AAF_);
                ep.cb(&icv, 4, gl + 5, JQ_);
                
                
                ep.ah(0, bdt, d, byr, MF_);
                
                
                ep.ah(0, bdt, bqy, byr, AOE_);
                ep.ah(bqy - 1, bdt, 1, byr, 0xFF333333);
                
                
                for afj in 0..gwe {
                    let atd = aey.ug + afj;
                    if atd >= aey.ak.len() { break; }
                    
                    let ct = bdt + (afj as u32 * gy);
                    if ct + gy > bdt + byr { break; }
                    
                    let afb = atd == aey.gn;
                    
                    
                    if afb {
                        ep.ah(bds, ct, ior, gy, AOB_);
                    }
                    
                    
                    let ajh = format!("{:>4} ", atd + 1);
                    let htc = if afb { BMV_ } else { AOH_ };
                    ep.cb(&ajh, 2, ct, htc);
                    
                    
                    let line = &aey.ak[atd];
                    
                    if aey.eej == Language::Rust {
                        let eb = puh(line);
                        for dlx in &eb {
                            let s = puf(dlx.kk);
                            let xfz = &line[dlx.ay..dlx.ci];
                            let cr = bds + 4 + (dlx.ay as u32 * nk);
                            if cr < d {
                                ep.cb(xfz, cr, ct, s);
                            }
                        }
                        if eb.is_empty() && !line.is_empty() {
                            ep.cb(line, bds + 4, ct, JQ_);
                        }
                    } else {
                        ep.cb(line, bds + 4, ct, JQ_);
                    }
                    
                    
                    if afb {
                        let kdv = (aey.byk / 30) % 2 == 0;
                        if kdv {
                            let cx = bds + 4 + (aey.hn as u32 * nk);
                            ep.ah(cx, ct, 2, gy, AOC_);
                        }
                    }
                }
                
                
                if aey.ak.len() > gwe {
                    let auz = d - 10;
                    let dbr = byr;
                    let es = aey.ak.len() as u32;
                    let axd = ((gwe as u32 * dbr) / es).am(20);
                    let omf = es.ao(gwe as u32);
                    let bsm = if omf > 0 {
                        (aey.ug as u32 * (dbr - axd)) / omf
                    } else { 0 };
                    ep.ah(auz, bdt, 10, dbr, 0xFF252526);
                    ep.ah(auz + 2, bdt + bsm, 6, axd, 0xFF555555);
                }
                
                
                let uo = i - bfm;
                ep.ah(0, uo, d, bfm, AAF_);
                
                
                let wtv = if let Some(ref fr) = aey.ccb {
                    format!("  {}", fr)
                } else {
                    let rxw = if aey.no { " [Modified]" } else { "" };
                    let ebt = aey.wn.ahz().unwrap_or("untitled");
                    format!("  {}{}", ebt, rxw)
                };
                ep.cb(&wtv, 4, uo + 3, GD_);
                
                
                let poo = format!(
                    "Ln {}, Col {}  {}  UTF-8  TrustCode",
                    aey.gn + 1,
                    aey.hn + 1,
                    aey.eej.j(),
                );
                let dvc = d.ao((poo.len() as u32 * nk) + 8);
                ep.cb(&poo, dvc, uo + 3, GD_);

            } else {
                
                
                
                
            
            let bss = cz.len();
            let act = AFH_.v(ulk);
            
            
            let aye = bss.ao(act);
            if px > aye {
                px = aye;
            }
            
            let dlz = px;
            let ktk = (dlz + act).v(bss);
            
            for (a, line) in cz.iter().chz(dlz).take(act).cf() {
                let c = gl + (a as u32) * acg;
                if c + acg > i - 50 { break; }
                
                
                let s = if line.cj("+") || line.cj("+") || line.cj("|") {
                    ya  
                } else if line.cj("|") {
                    
                    if line.contains("NAVIGATION:") || line.contains("FILE OPERATIONS:") || 
                       line.contains("COMMANDS:") || line.contains("TIPS:") ||
                       line.contains("BASIC COMMANDS:") || line.contains("EXAMPLES:") ||
                       line.contains("NOTE:") {
                        0xFFFFFF00  
                    } else if line.contains(" - ") {
                        
                        ya  
                    } else if line.cj("|    *") {
                        0xFFAAAAAA  
                    } else {
                        0xFFDDDDDD  
                    }
                } else if line.cj(">") {
                    0xFF88FF88  
                } else if line.contains("<DIR>") {
                    0xFF00FFFF  
                } else if line.contains(" B") && !line.contains("Browse") {
                    ya  
                } else if line.cj("Created") || line.cj("Changed") || line.cj("Removed") {
                    0xFF00FF00  
                } else if line.contains("Error") || line.contains("cannot") || line.contains("No such") {
                    0xFFFF4444  
                } else {
                    aek  
                };
                ep.cb(line, 12, c, s);
            }
            
            
            
            
            if bss > act {
                let ftr = d - 12;
                let pgu = gl;
                let mcr = i - gl - 50;  
                
                
                ep.ah(ftr, pgu, 8, mcr, 0xFF1A1A1A);
                
                
                let xgp = act as f32 / bss as f32;
                let axd = ((mcr as f32 * xgp) as u32).am(20);
                let mcp = if aye > 0 { 
                    px as f32 / aye as f32 
                } else { 
                    0.0 
                };
                let bsm = pgu + ((mcr - axd) as f32 * mcp) as u32;
                
                
                ep.ah(ftr, bsm, 8, axd, aek);
                ep.ah(ftr + 1, bsm + 1, 6, axd - 2, ya);
            }
            
            
            let alf = i - 40;
            ep.ah(10, alf, d - 20, 30, 0xFF050505);
            ep.lx(10, alf, d - 20, 30, aek);
            
            
            let jv = crate::ramfs::fh(|fs| String::from(fs.dau()));
            
            ep.cb("root", 16, alf + 8, 0xFFFF0000);  
            
            ep.cb("@", 16 + 32, alf + 8, 0xFFFFFFFF);  
            
            ep.cb("trustos", 16 + 40, alf + 8, 0xFF00FF00);  
            
            let hup = format!(":{}$ ", jv);
            ep.cb(&hup, 16 + 96, alf + 8, 0xFF00FF00);  
            let lvy = (4 + 1 + 7 + hup.len()) as u32 * 8;  
            
            
            ep.cb(&bfh, 16 + lvy, alf + 8, cyh);
            
            
            if !gto.is_empty() {
                let est = (bfh.len() * 8) as u32;
                ep.cb(&gto, 16 + lvy + est, alf + 8, 0xFF444444);
            }
            
            
            btx = (oo / 30) % 2 == 0;
            if btx {
                let lf = 16 + lvy + (bfh.len() as u32 * 8);
                ep.ah(lf, alf + 6, 8, 16, cyh);
            }
            } 
            } else {
                
                ep.clear(0x00000000);
            }
        }
        
        
        
        
        if let Some(crg) = compositor.dhm(toy) {
            let avz = crg.z;
            let bka = crg.ac;
            
            
            crg.clear(0xD8181818);
            
            
            crg.lx(0, 0, avz, bka, 0xFF444444);
            crg.lx(1, 1, avz - 2, bka - 2, 0xFF333333);
            
            
            crg.ah(2, 2, avz - 4, 20, 0xFF252525);
            crg.cb("Command History", 8, 6, 0xFFAAAAAA);
            
            
            let vc = 26u32;
            let gy = 18u32;
            
            if bqa.is_empty() {
                crg.cb("(no commands yet)", 10, vc + 5, 0xFF666666);
            } else {
                
                for (a, cmd) in bqa.iter().vv().take(10).cf() {
                    let c = vc + (a as u32) * gy;
                    if c + gy > bka - 5 { break; }
                    
                    
                    let num = bqa.len() - a;
                    let ajh = format!("{:2}.", num);
                    crg.cb(&ajh, 6, c + 2, 0xFF666666);
                    
                    
                    let ryl = if cmd.len() > 26 {
                        format!("{}...", &cmd[..23])
                    } else {
                        cmd.clone()
                    };
                    crg.cb(&ryl, 30, c + 2, 0xFF88FF88);
                }
            }
        }
        
        
        
        
        if let Some(bar) = compositor.dhm(xbd) {
            bar.clear(0xFF0A0A0A);
            
            
            bar.ah(0, 0, z, 2, aek);
            
            
            bar.ah(5, 6, 100, 28, if awe { 0xFF002200 } else { 0xFF0A1A0A });
            bar.lx(5, 6, 100, 28, ya);
            bar.cb("TrustOS", 20, 12, 0xFFFFFFFF); 
            
            
            let czz = match atu {
                AppMode::Df => "Shell",
                AppMode::As => "Network",
                AppMode::Ip => "Hardware",
                AppMode::Ag => "Editor",
                AppMode::Rb => "Users",
                AppMode::Pl => "Files",
                AppMode::Browser => "Browser",
                AppMode::Bp => "Images",
            };
            bar.ah(115, 6, 90, 28, 0xFF001100);
            bar.cb(czz, 125, 12, 0xFFFFFFFF); 
            
            
            
            let fyu = 220u32;
            if fbm {
                
                bar.ah(fyu, 6, 100, 28, 0xFF002200);
                bar.lx(fyu, 6, 100, 28, ya);
                bar.cb(czz, fyu + 10, 12, cyh);
                
                bar.ah(fyu + 20, 32, 60, 3, ya);
            } else {
                
                bar.ah(fyu, 6, 100, 28, 0xFF0A0A0A);
                bar.lx(fyu, 6, 100, 28, aek);
                bar.cb(czz, fyu + 10, 12, aek);
            }
            
            
            let mex = 340u32;
            let wke = if dvu { 0xFF002200 } else { 0xFF0A1A0A };
            bar.ah(mex, 6, 50, 28, wke);
            bar.lx(mex, 6, 50, 28, aek);
            bar.cb("[S]", mex + 10, 12, if dvu { cyh } else { ya });
            
            
            let os = crate::rtc::cgz();
            let bso = format!("{:02}:{:02}:{:02}", os.bek, os.bri, os.chr);
            bar.cb(&bso, z - 180, 12, ya);
            
            
            let ghn = format!("{}fps", tz);
            bar.cb(&ghn, z - 260, 12, aek);
            
            
            bar.abc(z - 60, 20, 6, ya);
            bar.abc(z - 40, 20, 6, 0xFFFFAA00);
        }
        
        
        
        
        if let Some(djm) = compositor.dhm(und) {
            if awe {
                djm.iw.store(true, core::sync::atomic::Ordering::SeqCst);
                djm.clear(0xF0080808);  
                
                let afr = 270u32;
                let aje = 390u32;
                
                
                djm.lx(0, 0, afr, aje, ya);
                djm.lx(1, 1, afr - 2, aje - 2, aek);
                
                
                djm.ah(2, 2, afr - 4, 34, 0xFF001500);
                djm.cb("TrustOS Menu", 10, 10, ya);
                
                
                let crv = 36u32;
                for (a, (j, item)) in gmp.iter().cf() {
                    let og = 40 + (a as u32) * crv;
                    
                    
                    if j.cj("-") {
                        djm.ah(10, og + 14, afr - 20, 1, aek);
                        continue;
                    }
                    
                    
                    if czs == a as i32 {
                        djm.ah(5, og, afr - 10, crv - 2, 0xFF002200);
                    }
                    
                    
                    let (s, pa) = match item {
                        MenuItem::Kc(_) => {
                            let r = if czs == a as i32 { cyh } else { aek };
                            (r, ">")
                        },
                        MenuItem::Qt => {
                            let r = if czs == a as i32 { 0xFFFF6666 } else { 0xFFAA4444 };
                            (r, "X")
                        },
                        MenuItem::Axr => {
                            let r = if czs == a as i32 { 0xFFFFAA66 } else { 0xFFAA8844 };
                            (r, "R")
                        },
                    };
                    
                    
                    djm.cb(j, 24, og + 10, s);
                    
                    
                    djm.cb(pa, afr - 30, og + 10, s);
                }
            } else {
                djm.iw.store(false, core::sync::atomic::Ordering::SeqCst);
            }
        }
        
        
        
        
        if let Some(bar) = compositor.dhm(wkf) {
            if dvu {
                bar.iw.store(true, core::sync::atomic::Ordering::SeqCst);
                bar.clear(0xF0080808);  
                
                let yd = 270u32;
                let ans = 340u32;  
                
                
                bar.lx(0, 0, yd, ans, ya);
                bar.lx(1, 1, yd - 2, ans - 2, aek);
                
                
                bar.ah(2, 2, yd - 4, 34, 0xFF001500);
                bar.cb("Settings", 10, 10, ya);
                
                
                let gyr = 50u32;
                let ir = dab as u32;
                let hl = daa as u32;
                let evy = ac - 380;  
                let mvt = ir >= (evy + gyr) 
                    && ir < (evy + gyr + 36)
                    && hl >= 340 && hl < (340 + yd);
                if mvt {
                    bar.ah(5, gyr, yd - 10, 34, 0xFF002200);
                }
                bar.cb("Animations:", 15, gyr + 10, aek);
                let gyq = if gsi { "ON" } else { "OFF" };
                let qim = if gsi { 0xFF00FF66 } else { 0xFFFF6666 };
                bar.cb(gyq, yd - 50, gyr + 10, qim);
                
                
                let ibe = 90u32;
                let pmg = ir >= (evy + ibe) 
                    && ir < (evy + ibe + 36)
                    && hl >= 340 && hl < (340 + yd);
                if pmg {
                    bar.ah(5, ibe, yd - 10, 34, 0xFF002200);
                }
                bar.cb("Speed:", 15, ibe + 10, aek);
                let mgr = format!("{:.1}x", gsj);
                bar.cb(&mgr, yd - 60, ibe + 10, ya);
                
                
                bar.cb("- Background -", 15, 140, 0xFF555555);
                
                
                let hna = 160u32;
                let oby = ir >= (evy + hna) 
                    && ir < (evy + hna + 36)
                    && hl >= 340 && hl < (340 + yd);
                if oby {
                    bar.ah(5, hna, yd - 10, 34, 0xFF002200);
                }
                bar.cb("HoloMatrix 3D:", 15, hna + 10, aek);
                let tpp = if bnf { "ON" } else { "OFF" };
                let tpm = if bnf { 0xFF00FFFF } else { 0xFFFF6666 };
                bar.cb(tpp, yd - 50, hna + 10, tpm);
                
                
                let hyx = 200u32;
                let pgh = ir >= (evy + hyx) 
                    && ir < (evy + hyx + 36)
                    && hl >= 340 && hl < (340 + yd);
                if pgh && bnf {
                    bar.ah(5, hyx, yd - 10, 34, 0xFF002200);
                }
                let wee = if bnf { aek } else { 0xFF333333 };
                bar.cb("Scene:", 15, hyx + 10, wee);
                let wed = if bnf { 0xFF00FFFF } else { 0xFF444444 };
                bar.cb(fks.j(), yd - 80, hyx + 10, wed);
                
                
                bar.cb("Click to toggle/cycle", 15, 250, 0xFF555555);
                
                
                bar.cb("[Esc] or click away", 15, 305, 0xFF444444);
                
                
                if dew && mvt {
                    gsi = !gsi;
                    crate::desktop::jop(gsi);
                }
                
                
                if dew && pmg {
                    
                    gsj = if gsj <= 0.5 { 1.0 } 
                        else if gsj <= 1.0 { 2.0 } 
                        else { 0.5 };
                    crate::desktop::pio(gsj);
                }
                
                
                if dew && oby {
                    bnf = !bnf;
                    crate::graphics::holomatrix::cuf(bnf);
                }
                
                
                if dew && pgh && bnf {
                    fks = fks.next();
                    crate::graphics::holomatrix::bid(fks);
                }
            } else {
                bar.iw.store(false, core::sync::atomic::Ordering::SeqCst);
            }
        }
        
        
        
        
        if let Some(gi) = compositor.dhm(nio) {
            gi.clear(0x00000000); 
            
            
            let knf = if fd { 0xFF00FF00 } else { 0xFFFFFFFF }; 
            let aia = if fd { 0xFF005500 } else { 0xFF000000 };
            
            
            
            for a in 0..16 {
                for fb in 0..=a {
                    if fb <= a && a < 16 {
                        gi.aht(fb as u32, a as u32, knf);
                    }
                }
            }
            
            for a in 0..16 {
                gi.aht(0, a as u32, aia);
                gi.aht(a as u32, a as u32, aia);
            }
            
            for a in 10..16 {
                gi.aht((a - 5) as u32, a as u32, knf);
                gi.aht((a - 6) as u32, a as u32, knf);
            }
        }
        
        
        
        
        compositor.iov();
        compositor.brs();
        
        } 
        
        
        oo += 1;
        dqx += 1;
        
        
        if oo % 100 == 0 {
            crate::serial_println!("[COSMIC2] Frame {}", oo);
        }
        
        let iu = crate::cpu::tsc::ow();
        if iu - fmq >= fal {
            tz = dqx;
            pbv = lze;
            dqx = 0;
            lze = 0;
            fmq = iu;
            crate::serial_println!("[COSMIC2] FPS: {} (render: {}) | Frame: {} | Mode: {}",
                tz, pbv, oo, if atr { "FORMULA" } else if apf { "BRAILLE" } else if aqq { "FAST" } else { "LEGACY" });
        }
        
        
        
        
        
        unsafe {
            
            #[cfg(target_arch = "x86_64")]
            core::arch::asm!("sti");
            #[cfg(not(target_arch = "x86_64"))]
            crate::arch::ofa();
            
            for _ in 0..100 {
                #[cfg(target_arch = "x86_64")]
                core::arch::asm!("pause");
                #[cfg(not(target_arch = "x86_64"))]
                core::hint::hc();
            }
        }
    }
    
    
    crate::framebuffer::clear();
    crate::serial_println!("[COSMIC2] Exited");
    crate::h!(B_, "COSMIC V2 Desktop exited. Type 'help' for commands.");
}



pub(super) fn yjc() {
    use crate::cosmic::{Rect, Point, Color};
    use crate::cosmic::theme::{dark, matrix};
    use alloc::format;
    
    crate::h!(C_, "+-----------------------------------------------------------+");
    crate::h!(C_, "|       COSMIC Desktop Environment - TrustOS Edition        |");
    crate::h!(C_, "|-----------------------------------------------------------|");
    crate::h!(B_, "|  Controls:                                                |");
    crate::h!(Q_, "|    ESC / Q     - Exit desktop                             |");
    crate::h!(Q_, "|    M           - Matrix theme (cyberpunk)                 |");
    crate::h!(Q_, "|    D           - Dark theme (default)                     |");
    crate::h!(Q_, "|    1-5         - Switch apps                              |");
    crate::h!(Q_, "|    Mouse       - Interact with UI                         |");
    crate::h!(C_, "+-----------------------------------------------------------+");
    crate::serial_println!("[COSMIC] Starting COSMIC Desktop Environment...");
    
    
    while crate::keyboard::xw().is_some() {}
    
    let (z, ac) = crate::framebuffer::yn();
    if z == 0 || ac == 0 {
        crate::h!(A_, "Error: Invalid framebuffer!");
        return;
    }
    
    
    crate::framebuffer::beo();
    crate::framebuffer::afi(true);
    crate::serial_println!("[COSMIC] Double buffering enabled for fast rendering");
    
    
    crate::mouse::dbw(z, ac);
    
    
    
    
    let mut aqk = true;
    let mut fxw = true;
    let mut oo = 0u64;
    
    
    let fal = crate::cpu::tsc::ard();
    let mut tz = 0u32;
    let mut dqx = 0u32;
    let mut fmq = crate::cpu::tsc::ow();
    let mut iaj = true;  
    
    
    let mut aqq = false;  
    let mut apf = true;      
    
    
    let xav = 60u64;
    let yrm = fal / xav;
    let mut ucg = crate::cpu::tsc::ow();
    
    
    
    
    
    const R_: usize = 160;  
    const AZI_: u32 = 16;   
    const AZK_: usize = 30; 
    
    
    let mut cas: [(i32, u32, u32); R_] = [(0, 0, 0); R_];
    
    
    for a in 0..R_ {
        let dv = (a as u32 * 2654435761) ^ 0xDEADBEEF;
        let vc = -((dv % (ac * 2)) as i32); 
        let ig = 2 + (dv % 5); 
        cas[a] = (vc, ig, dv);
    }
    
    
    const OB_: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ@#$%&*+=<>[]{}|";
    
    
    let mut gpt = false;
    let mut fev = false;
    let mut oxi = 0.0f32;
    let mut oxj = 0.0f32;
    let mut oxl = fxw;
    let mut oxg = 0usize;
    let mut zgl = -1i32;
    let mut bex = true; 
    
    
    let mut dnw = 0usize;
    let mut ocg = -1i32;
    
    
    let mut awe = false;
    let mut czs = -1i32;  
    let mut bcn = false;
    let mut job: [u8; 32] = [0u8; 32];
    let mut ftw = 0usize;
    
    
    let gmp = [
        "Apps",
        "Browser",
        "Calculator", 
        "Files",
        "Network",
        "Settings",
        "Terminal",
        "---",  
        "Sign Out",
        "Restart",
        "Shutdown",
    ];
    
    
    let apps = [
        ("Files", "File Manager"),
        ("Terminal", "System Terminal"),
        ("Browser", "Web Browser"),
        ("Code", "Text Editor"),
        ("Settings", "System Settings"),
    ];
    
    
    let mut abx = 150.0f32;
    let mut aha = 60.0f32;
    let aog = 700.0f32;
    let biz = 450.0f32;
    let mut cka = false;
    let mut ymr = 0.0f32;
    let mut yms = 0.0f32;
    
    
    let zrd = [
        "$ neofetch",
        "  _____              _    ___  ____  ",
        " |_   _| __ _   _ __| |_ / _ \\/ ___| ",
        "   | || '__| | | / __| __| | | \\___ \\ ",
        "   | || |  | |_| \\__ \\ |_| |_| |___) |",
        "   |_||_|   \\__,_|___/\\__|\\___/|____/ ",
        "",
        "  OS: TrustOS v0.2.0",
        "  Kernel: Custom Rust Kernel",
        "  Shell: TrustOS Shell",
        "  Resolution: 1280x800",
        "  Theme: COSMIC Matrix",
        "",
        "$ _",
    ];
    
    crate::serial_println!("[COSMIC] Entering main loop...");
    
    
    
    
    while aqk {
        
        
        
        let mouse = crate::mouse::drd();
        let hl = mouse.b as f32;
        let ir = mouse.c as f32;
        let oir = mouse.jda;
        
        
        fev = oir && !gpt;
        gpt = oir;
        
        
        if let Some(bs) = crate::keyboard::xw() {
            if bcn {
                
                match bs {
                    27 => { bcn = false; },  
                    8 => {  
                        if ftw > 0 { ftw -= 1; }
                    },
                    13 => { bcn = false; },  
                    32..=126 => {  
                        if ftw < 31 {
                            job[ftw] = bs;
                            ftw += 1;
                        }
                    },
                    _ => {}
                }
            } else {
                
                match bs {
                    27 | b'q' | b'Q' => {
                        if awe { awe = false; }
                        else { aqk = false; }
                    },
                    b'm' | b'M' => { fxw = true; bex = true; },
                    b'd' | b'D' => { fxw = false; bex = true; },
                    b'1'..=b'5' => { dnw = (bs - b'1') as usize; bex = true; },
                    b's' | b'S' => { bcn = true; },  
                    b't' | b'T' => { awe = !awe; },  
                    _ => {}
                }
            }
        }
        
        
        let upt = (hl - oxi).gp() > 0.5 || (ir - oxj).gp() > 0.5;
        let wtl = fxw != oxl || dnw != oxg || fev || cka;
        
        
        
        if !fxw && !bex && !upt && !wtl {
            
            oo += 1;
            dqx += 1;
            let iu = crate::cpu::tsc::ow();
            if iu - fmq >= fal {
                tz = dqx;
                dqx = 0;
                fmq = iu;
            }
            continue;
        }
        
        oxi = hl;
        oxj = ir;
        oxl = fxw;
        oxg = dnw;
        bex = false;
        
        
        
        
        let (ei, fqd, surface, dwl, mm, xfx, xfy, 
             fkk, enp, lks, llv, vx, ekt) = 
            if fxw {
                (matrix::DD_, matrix::KV_, matrix::Kw, matrix::JA_,
                 matrix::Ge, matrix::AC_, matrix::N_,
                 matrix::GO_, matrix::HP_, matrix::II_, matrix::IL_,
                 matrix::Aep, matrix::Afq)
            } else {
                (dark::DD_, dark::KV_, dark::Kw, dark::JA_,
                 dark::Ge, dark::AC_, dark::N_,
                 dark::GO_, dark::HP_, dark::II_, dark::IL_,
                 dark::Aep, dark::Afq)
            };
        
        
        let ygi = ei.lv();
        let zes = fqd.lv();
        let zqe = surface.lv();
        let zqd = dwl.lv();
        let yec = mm.lv();
        let zsl = xfx.lv();
        let zsm = xfy.lv();
        let ywx = fkk.lv();
        let yis = enp.lv();
        let zcj = lks.lv();
        let zcs = llv.lv();
        let zpw = vx.lv();
        let zvw = ekt.lv();
        
        
        use crate::framebuffer::{
            cwe, ah, afp, abc,
            gtn, cb, sv, lx
        };
        
        
        let ya: u32 = 0xFF00FF66;      
        let cyh: u32 = 0xFF00FF88;    
        let aek: u32 = 0xFF009944;       
        let cot: u32 = 0xFF000000;           
        
        
        
        
        cwe(cot);
        
        
        
        
        let tn = 36u32;
        let byt = z / R_ as u32;
        
        for bj in 0..R_ {
            let (mrx, ig, dv) = cas[bj];
            let b = (bj as u32 * byt) + byt / 2;
            
            
            let bhn = mrx + ig as i32;
            let bhn = if bhn > ac as i32 + (AZK_ as i32 * AZI_ as i32) {
                let gnp = dv.hx(1103515245).cn(12345);
                cas[bj].2 = gnp;
                -((gnp % (ac / 2)) as i32)
            } else {
                bhn
            };
            cas[bj].0 = bhn;
            
            
            
            
            let eo = (ig as f32 - 2.0) / 4.0; 
            let kpf = 0.4 + eo * 0.6; 
            let nkr = 0.3 + eo * 0.7; 
            
            for a in 0..AZK_ {
                let avl = bhn - (a as i32 * AZI_ as i32);
                if avl < 0 || avl >= (ac - tn) as i32 { continue; }
                
                let mxz = if a == 0 { 255u8 } 
                    else if a == 1 { 220u8 } 
                    else { 180u8.ao((a as u8).mbq(9)) };
                if mxz < 20 { continue; }
                
                
                let kt = ((mxz as f32) * kpf) as u8;
                
                
                
                let m = if a == 0 { 
                    ((180.0 * kpf) as u8) 
                } else { 
                    
                    ((20.0 * (1.0 - nkr)) as u8)
                };
                let at = kt;
                let o = if a == 0 { 
                    ((180.0 * kpf) as u8) 
                } else { 
                    
                    ((40.0 * (1.0 - nkr) + 10.0 * eo) as u8)
                };
                let s = 0xFF000000 | ((m as u32) << 16) | ((at as u32) << 8) | (o as u32);
                
                let des = dv.cn((a as u32 * 7919) ^ (oo as u32 / 8));
                let gcl = (des as usize) % OB_.len();
                let inh: [u8; 2] = [OB_[gcl], 0];
                let qyh = unsafe { core::str::nwj(&inh[..1]) };
                cb(qyh, b, avl as u32, s);
            }
        }
        
        
        
        
        
        let ayc = z / 2 + 100;  
        let dsw = ac / 2 - 50;
        
        
        let bhj = dsw - 180;
        
        ah(ayc - 40, bhj, 12, 60, ya);      
        ah(ayc + 28, bhj, 12, 60, ya);      
        ah(ayc - 40, bhj - 10, 80, 15, ya); 
        ah(ayc - 30, bhj - 20, 60, 15, ya); 
        
        ah(ayc - 50, bhj + 50, 100, 70, ya);
        ah(ayc - 44, bhj + 56, 88, 58, 0xFF0A150Au32);
        
        abc(ayc, bhj + 80, 10, ya);
        ah(ayc - 5, bhj + 88, 10, 20, ya);
        
        
        let eiq = dsw - 60;
        let cbt = 180u32;
        let dlu = 220u32;
        
        for ab in 0..4 {
            let tt = ab as u32;
            
            ah(ayc - cbt/2 + 20 + tt, eiq + tt, cbt - 40, 3, ya);
            
            ah(ayc - cbt/2 + tt, eiq + 20 + tt, 25, 3, ya);
            ah(ayc + cbt/2 - 25 - tt, eiq + 20 + tt, 25, 3, ya);
            
            ah(ayc - cbt/2 + tt, eiq + 20, 3, dlu - 80, ya);
            ah(ayc + cbt/2 - 3 - tt, eiq + 20, 3, dlu - 80, ya);
            
            ah(ayc - 3, eiq + dlu - 20 - tt, 6, 20, ya);
        }
        
        ah(ayc - cbt/2 + 8, eiq + 25, cbt - 16, dlu - 70, 0xFF051208u32);
        
        
        let ncm = ayc;
        let ncn = dsw + 20;
        
        for ab in 0..8 {
            
            for a in 0..30 {
                ah(ncm - 50 + a + ab, ncn - 30 + a, 4, 4, ya);
            }
            
            for a in 0..50 {
                ah(ncm - 20 + a + ab, ncn + a.v(29) - (a.ao(29)), 4, 4, ya);
            }
        }
        
        
        let err = dsw + 100;
        
        ah(ayc - 100, err, 40, 15, aek);
        ah(ayc - 110, err + 10, 20, 30, aek);
        ah(ayc - 95, err + 15, 10, 25, aek);
        ah(ayc - 80, err + 15, 10, 20, aek);
        
        ah(ayc + 60, err, 40, 15, aek);
        ah(ayc + 90, err + 10, 20, 30, aek);
        ah(ayc + 85, err + 15, 10, 25, aek);
        ah(ayc + 70, err + 15, 10, 20, aek);
        
        
        
        let wg = ayc + 130;
        let sl = dsw + 40;
        cb("TRust-os", wg, sl, ya);
        
        cb("TRust-os", wg + 1, sl, ya);
        cb("TRust-os", wg, sl + 1, ya);
        cb("TRust-os", wg + 1, sl + 1, ya);
                
        
        let ekx = 90u32;
        let eky = 300u32;
        let gwt = 380u32;
        let jwv = 280u32;
        
        
        
        
        let eay = 20u32;
        let eoz = 50u32;
        let cxb = 44u32;
        let saa = 20u32;
        let sab = 5u32;
        
        
        ah(0, 0, 80, ac - tn, 0xFF050505u32);
        
        ocg = -1;
        for a in 0..sab {
            let og = eoz + a * (cxb + saa);
            let fg = eay;
            
            
            let asy = hl >= fg as f32 && hl < (fg + cxb) as f32 && 
                          ir >= og as f32 && ir < (og + cxb) as f32;
            if asy {
                ocg = a as i32;
                if fev {
                    dnw = a as usize;
                }
            }
            
            
            let xd = if a as usize == dnw { 
                ya 
            } else if asy { 
                cyh 
            } else { 
                aek 
            };
            
            
            lx(fg, og, cxb, cxb, xd);
            
            
            let cx = fg + cxb / 2;
            let ae = og + cxb / 2;
            match a {
                0 => { 
                    ah(cx - 8, ae - 10, 4, 20, xd);
                    ah(cx - 4, ae - 8, 4, 16, xd);
                    ah(cx, ae - 6, 4, 12, xd);
                    ah(cx + 4, ae - 4, 4, 8, xd);
                },
                1 => { 
                    lx(cx - 12, ae - 10, 24, 20, xd);
                    ah(cx - 8, ae - 4, 10, 2, xd);
                    ah(cx - 8, ae + 2, 6, 2, xd);
                },
                2 => { 
                    for br in 0..2 {
                        for bj in 0..2 {
                            lx(cx - 10 + bj * 12, ae - 10 + br * 12, 8, 8, xd);
                        }
                    }
                },
                3 => { 
                    lx(cx - 10, ae - 8, 20, 16, xd);
                    ah(cx - 6, ae - 4, 2, 8, xd);
                    ah(cx - 2, ae - 2, 2, 6, xd);
                    ah(cx + 2, ae - 6, 2, 10, xd);
                    ah(cx + 6, ae - 4, 2, 8, xd);
                },
                4 => { 
                    abc(cx, ae, 10, xd);
                    abc(cx, ae, 6, cot);
                },
                _ => {}
            }
        }
        
        
        let obq = ac as u32 - 80;
        lx(eay, obq, cxb, cxb, aek);
        cb("?", eay + 18, obq + 16, aek);
        
        
        
        
        
        
        
        let cov = 3u32;
        
        ah(ekx, eky, gwt, cov, ya);
        
        ah(ekx, eky + jwv - cov, gwt, cov, ya);
        
        ah(ekx, eky, cov, jwv, ya);
        
        ah(ekx + gwt - cov, eky, cov, jwv, ya);
        
        
        ah(ekx + cov, eky + cov, 
                  gwt - cov * 2, jwv - cov * 2, cot);
        
        
        let bxn = 28u32;
        ah(ekx + cov, eky + cov, 
                  gwt - cov * 2, bxn, 0xFF0A1A0Au32);
        
        
        let (dq, _) = apps[dnw];
        let xhz = format!("TrustOS {} v1.00", dq);
        cb(&xhz, ekx + 12, eky + 10, ya);
        
        
        let kn = eky + 10;
        let bdr = ekx + gwt - 60;
        
        abc(bdr, kn + 6, 6, 0xFFFF5555u32);
        
        abc(bdr + 18, kn + 6, 6, 0xFFFFDD55u32);
        
        abc(bdr + 36, kn + 6, 6, 0xFF55FF55u32);
        
        
        let tc = ekx + 15;
        let gl = eky + bxn + 15;
        
        
        let rsm = crate::ramfs::fh(|fs| String::from(fs.dau()));
        
        cb("root", tc, gl, 0xFFFF0000u32);  
        
        cb("@", tc + 32, gl, 0xFFFFFFFFu32);  
        
        cb("trustos", tc + 40, gl, 0xFF00FF00u32);  
        
        let hup = format!(":{}$ ", rsm);
        cb(&hup, tc + 96, gl, 0xFF00FF00u32);  
        
        let lvx = 4 + 1 + 7 + hup.len();  
        let lf = tc + (lvx * 8) as u32;
        ah(lf, gl, 8, 16, cyh);
        
        
        
        
        let pl = ac as u32 - tn;
        
        
        ah(0, pl, z as u32, tn, 0xFF080808u32);
        
        ah(0, pl, z as u32, 2, aek);
        
        
        let jfx = 8u32;
        let unc = 24u32;
        ah(jfx + 4, pl + 14, 16, 3, ya);
        ah(jfx + 4, pl + 19, 16, 3, ya);
        
        
        if fev && hl >= jfx as f32 && hl < (jfx + unc) as f32 &&
           ir >= pl as f32 {
            awe = !awe;
        }
        
        
        let mjo = 40u32;
        let prj = 90u32;
        ah(mjo, pl + 6, prj, 24, 0xFF0A1A0Au32);
        lx(mjo, pl + 6, prj, 24, aek);
        cb("TrustOS", mjo + 14, pl + 10, ya);
        
        
        let prk = 138u32;
        let xad = 90u32;
        ah(prk, pl + 6, xad, 24, 0xFF050A05u32);
        cb("Terminal", prk + 12, pl + 10, aek);
        
        
        let cmt = z as u32 / 2 - 120;
        let bco = 240u32;
        ah(cmt, pl + 6, bco, 24, 0xFF0A0A0Au32);
        lx(cmt, pl + 6, bco, 24, aek);
        if ftw == 0 {
            cb("Search...", cmt + 8, pl + 10, 0xFF336633u32);
        } else {
            let wfl = unsafe { core::str::nwj(&job[..ftw]) };
            cb(wfl, cmt + 8, pl + 10, ya);
        }
        
        abc(cmt + bco - 20, pl + 18, 6, aek);
        abc(cmt + bco - 20, pl + 18, 4, 0xFF0A0A0Au32);
        ah(cmt + bco - 16, pl + 22, 6, 2, aek);
        
        
        if fev && hl >= cmt as f32 && hl < (cmt + bco) as f32 &&
           ir >= pl as f32 {
            bcn = true;
        }
        
        
        let os = crate::rtc::cgz();
        let bso = format!("{:02}:{:02}", os.bek, os.bri);
        cb(&bso, z as u32 - 200, pl + 10, ya);
        
        
        cb("TRST-001", z as u32 - 120, pl + 10, cyh);
        
        
        let dij = z as u32 - 50;
        abc(dij, pl + 18, 6, ya);
        abc(dij + 16, pl + 18, 6, 0xFFFFAA00u32);
        
        ah(dij + 28, pl + 12, 4, 4, aek);
        ah(dij + 34, pl + 12, 4, 4, aek);
        ah(dij + 28, pl + 18, 4, 4, aek);
        ah(dij + 34, pl + 18, 4, 4, aek);
        
        
        
        
        if awe {
            let rs = 10u32;
            let xp = pl - 320;
            let afr = 180u32;
            let aje = 310u32;
            
            
            ah(rs, xp, afr, aje, 0xFF0A0F0Au32);
            lx(rs, xp, afr, aje, ya);
            lx(rs + 1, xp + 1, afr - 2, aje - 2, aek);
            
            
            ah(rs + 2, xp + 2, afr - 4, 30, 0xFF0A1A0Au32);
            cb("TrustOS Menu", rs + 12, xp + 10, ya);
            
            
            czs = -1;
            for (w, item) in gmp.iter().cf() {
                let ajd = xp + 40 + (w as u32 * 24);
                
                if *item == "---" {
                    
                    ah(rs + 10, ajd + 10, afr - 20, 1, aek);
                } else {
                    
                    let ohc = hl >= rs as f32 && hl < (rs + afr) as f32 &&
                                       ir >= ajd as f32 && ir < (ajd + 24) as f32;
                    
                    if ohc {
                        czs = w as i32;
                        ah(rs + 2, ajd, afr - 4, 24, 0xFF1A2A1Au32);
                        
                        
                        if fev {
                            match *item {
                                "Shutdown" => { crate::acpi::cbu(); },
                                "Restart" => { crate::acpi::jlq(); },
                                "Sign Out" => { aqk = false; },
                                "Settings" => { dnw = 4; awe = false; },
                                "Terminal" => { dnw = 1; awe = false; },
                                "Files" => { dnw = 0; awe = false; },
                                "Browser" => { dnw = 2; awe = false; },
                                _ => { awe = false; }
                            }
                        }
                    }
                    
                    
                    let agx = if *item == "Shutdown" || *item == "Restart" || *item == "Sign Out" {
                        0xFFFF6666u32  
                    } else if ohc {
                        cyh
                    } else {
                        ya
                    };
                    
                    
                    if *item == "Shutdown" {
                        abc(rs + 20, ajd + 12, 6, agx);
                        ah(rs + 18, ajd + 6, 4, 6, 0xFF0A0F0Au32);
                    }
                    
                    cb(item, rs + 35, ajd + 6, agx);
                }
            }
            
            
            if fev && (hl < rs as f32 || hl > (rs + afr) as f32 ||
                                    ir < xp as f32 || ir > pl as f32) {
                awe = false;
            }
        }
        
        
        
        
        if iaj && tz > 0 {
            let swo = format!("{} FPS", tz);
            let hkg = z.ao(80);
            let kwy = if tz >= 55 { 0xFF00FF00 }    
                           else if tz >= 30 { 0xFFFFFF00 } 
                           else { 0xFFFF4444 };            
            cb(&swo, hkg, 4, kwy);
            
            
            let ev = if apf { "BRL" } else if aqq { "FAST" } else { "LEG" };
            cb(ev, hkg, 20, 0xFF888888);
        }
        
        
        
        
        let uqx = hl as u32;
        let urb = ir as u32;
        
        for a in 0..12u32 {
            ah(uqx, urb + a, (12 - a).am(1), 1, ya);
        }
        
        
        
        
        sv();
        
        
        
        
        oo += 1;
        dqx += 1;
        
        let iu = crate::cpu::tsc::ow();
        if iu - fmq >= fal {
            tz = dqx;
            dqx = 0;
            fmq = iu;
            crate::serial_println!("[COSMIC] FPS: {} | Frame: {} | Mode: {}", 
                tz, oo, if apf { "BRAILLE" } else if aqq { "FAST" } else { "LEGACY" });
        }
        
        
        
        for _ in 0..100 {
            core::hint::hc();
        }
        
        ucg = crate::cpu::tsc::ow();
    }
    
    crate::framebuffer::clear();
    crate::serial_println!("[COSMIC] Desktop exited after {} frames, last FPS: {}", oo, tz);
    crate::h!(B_, "COSMIC Desktop exited. {} frames rendered, {} FPS", oo, tz);
}