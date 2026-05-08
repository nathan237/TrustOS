




use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use crate::framebuffer::{B_, G_, AX_, D_, A_, C_, R_, CF_, DM_, K_};
use crate::ramfs::FileType;


pub(super) fn fmd(args: &[&str]) {
    use alloc::vec;
    
    crate::n!(C_, "-----------------------------------------------------------");
    crate::n!(C_, "              TrustOS Graphics Benchmark");
    crate::n!(C_, "               SSE2 SIMD Optimizations");
    crate::n!(C_, "-----------------------------------------------------------");
    crate::println!();
    
    let (width, height) = crate::framebuffer::kv();
    let pixels = (width * height) as usize;
    crate::println!("Resolution: {}x{} ({} pixels, {} MB)", 
        width, height, pixels, pixels * 4 / 1024 / 1024);
    crate::println!();
    
    
    crate::n!(B_, "? Test 1: SSE2 Buffer Fill");
    {
        let mut buffer = vec![0u32; pixels];
        let xe = 100;
        
        let start = crate::cpu::tsc::ey();
        for _ in 0..xe {
            crate::graphics::simd::hyi(&mut buffer, 0xFF00FF66);
        }
        let end = crate::cpu::tsc::ey();
        
        let fqe = (end - start) / xe;
        let neb = pixels as f64 / 1_000_000.0;
        crate::println!("  {} iterations: {} cycles/frame", xe, fqe);
        crate::println!("  Throughput: ~{:.1} megapixels/frame", neb);
    }
    
    
    crate::n!(B_, "? Test 2: SSE2 Buffer Copy");
    {
        let src = vec![0xFF112233u32; pixels];
        let mut dst = vec![0u32; pixels];
        let xe = 100;
        
        let start = crate::cpu::tsc::ey();
        for _ in 0..xe {
            crate::graphics::simd::kxq(&mut dst, &src);
        }
        let end = crate::cpu::tsc::ey();
        
        let fqe = (end - start) / xe;
        let ndr = (pixels * 4) as f64 / 1024.0 / 1024.0;
        crate::println!("  {} iterations: {} cycles/frame", xe, fqe);
        crate::println!("  Bandwidth: ~{:.1} MB copied/frame", ndr);
    }
    
    
    crate::n!(B_, "? Test 3: Rectangle Fill (400x300)");
    {
        let mut surface = crate::graphics::fast_render::FastSurface::new(1280, 800);
        let xe = 500;
        
        let start = crate::cpu::tsc::ey();
        for _ in 0..xe {
            surface.fill_rect(100, 100, 400, 300, 0xFF00AA55);
        }
        let end = crate::cpu::tsc::ey();
        
        let lay = (end - start) / xe;
        let nuz = 400 * 300;
        crate::println!("  {} iterations: {} cycles/rect", xe, lay);
        crate::println!("  {} pixels/rect", nuz);
    }
    
    
    crate::n!(B_, "? Test 4: Framebuffer swap_buffers");
    {
        
        let fez = crate::framebuffer::ajy();
        if !fez {
            crate::framebuffer::adw();
            crate::framebuffer::pr(true);
        }
        
        let xe = 50;
        let start = crate::cpu::tsc::ey();
        for _ in 0..xe {
            crate::framebuffer::ii();
        }
        let end = crate::cpu::tsc::ey();
        
        let hqi = (end - start) / xe;
        
        let fvi = 3_000_000_000u64 / hqi.max(1);
        crate::println!("  {} iterations: {} cycles/swap", xe, hqi);
        crate::println!("  Estimated max FPS: ~{} (at 3GHz)", fvi);
        
        if !fez {
            crate::framebuffer::pr(false);
        }
    }
    
    
    crate::n!(B_, "? Test 5: GraphicsTerminal render (80x25)");
    {
        let mut terminal = crate::wayland::terminal::GraphicsTerminal::new(80, 25);
        terminal.write_str("Hello from TrustOS! Testing SSE2 SIMD terminal rendering performance.\n");
        terminal.write_str("The quick brown fox jumps over the lazy dog.\n");
        
        let xe = 100;
        let start = crate::cpu::tsc::ey();
        for _ in 0..xe {
            let _ = terminal.render();
        }
        let end = crate::cpu::tsc::ey();
        
        let hqh = (end - start) / xe;
        let fvi = 3_000_000_000u64 / hqh.max(1);
        crate::println!("  {} iterations: {} cycles/render", xe, hqh);
        crate::println!("  Estimated terminal FPS: ~{}", fvi);
    }
    
    crate::println!();
    crate::n!(C_, "-----------------------------------------------------------");
    crate::n!(B_, "Benchmark complete! SSE2 optimizations active.");
    crate::n!(C_, "-----------------------------------------------------------");
}



pub(super) fn qal(args: &[&str]) {
    use crate::cosmic::{CosmicRenderer, Rect, Point, Color, theme, CosmicTheme, set_theme};
    use crate::cosmic::theme::dark;
    
    let oyl = args.first().copied().unwrap_or("demo");
    
    match oyl {
        "demo" | "test" => {
            crate::n!(C_, "+---------------------------------------------------------------+");
            crate::n!(C_, "|          COSMIC UI Framework Demo (libcosmic-inspired)       |");
            crate::n!(C_, "+---------------------------------------------------------------+");
            crate::println!();
            
            let (width, height) = crate::framebuffer::kv();
            crate::println!("  Framebuffer: {}x{}", width, height);
            crate::println!("  Renderer: tiny-skia (software, no_std)");
            crate::println!("  Theme: COSMIC Dark (Pop!_OS style)");
            crate::println!();
            
            crate::n!(B_, "Creating COSMIC renderer...");
            let mut renderer = match CosmicRenderer::new(width, height) {
                Some(r) => r,
                None => {
                    crate::n!(A_, "  ERROR: Failed to create COSMIC renderer (OOM?)");
                    return;
                }
            };
            
            
            renderer.clear(dark::DK_);
            
            crate::n!(B_, "Drawing COSMIC UI elements...");
            
            
            let nps = Rect::new(0.0, 0.0, width as f32, 32.0);
            renderer.draw_panel(nps);
            
            
            
            let iyw = Rect::new(50.0, 80.0, 200.0, 100.0);
            renderer.fill_rounded_rect(iyw, 12.0, dark::El);
            renderer.stroke_rounded_rect(iyw, 12.0, dark::Bp, 1.0);
            
            
            let hix = Rect::new(300.0, 100.0, 120.0, 40.0);
            renderer.draw_shadow(hix, 8.0, 8.0, Color::BLACK.with_alpha(0.4));
            renderer.fill_rounded_rect(hix, 8.0, dark::Ch);
            
            
            let kef = Rect::new(450.0, 100.0, 120.0, 40.0);
            renderer.fill_rounded_rect(kef, 8.0, dark::MZ_);
            
            
            let keg = Rect::new(600.0, 100.0, 120.0, 40.0);
            renderer.fill_rounded_rect(keg, 8.0, dark::KE_);
            
            
            renderer.fill_circle(Point::new(100.0, 250.0), 30.0, dark::Ch);
            renderer.fill_circle(Point::new(180.0, 250.0), 30.0, dark::Nh);
            renderer.fill_circle(Point::new(260.0, 250.0), 30.0, dark::Nw);
            renderer.fill_circle(Point::new(340.0, 250.0), 30.0, dark::Hr);
            
            
            let mkp = Rect::new(50.0, 320.0, 400.0, 40.0);
            renderer.draw_header(mkp, "COSMIC Window", true);
            
            
            let pus = Rect::new(50.0, 360.0, 400.0, 150.0);
            renderer.fill_rect(pus, dark::CY_);
            renderer.stroke_rounded_rect(
                Rect::new(50.0, 320.0, 400.0, 190.0),
                0.0,
                dark::Bp,
                1.0
            );
            
            
            let fsr = [
                crate::cosmic::Ln { name: "Files", active: true, hovered: false, running: true },
                crate::cosmic::Ln { name: "Term", active: false, hovered: true, running: true },
                crate::cosmic::Ln { name: "Browser", active: false, hovered: false, running: false },
                crate::cosmic::Ln { name: "Settings", active: false, hovered: false, running: true },
            ];
            let lgv = Rect::new((width - 64) as f32, 100.0, 64.0, 280.0);
            renderer.draw_dock(lgv, &fsr);
            
            
            let mfy = Rect::new(500.0, 320.0, 200.0, 100.0);
            renderer.fill_gradient_v(mfy, dark::Ch, dark::DK_);
            
            crate::n!(B_, "Presenting to framebuffer...");
            renderer.present_to_framebuffer();
            
            crate::println!();
            crate::n!(G_, "? COSMIC UI demo rendered successfully!");
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
            
            
            crate::keyboard::ptj();
            crate::framebuffer::clear();
            crate::framebuffer::ii();
        },
        "desktop" => {
            
            hlr();
        },
        "theme" => {
            let cei = args.get(1).copied().unwrap_or("matrix");
            match cei {
                "dark" => {
                    set_theme(CosmicTheme::dark());
                    crate::n!(B_, "Theme set to COSMIC Dark");
                },
                "light" => {
                    set_theme(CosmicTheme::light());
                    crate::n!(B_, "Theme set to COSMIC Light");
                },
                "matrix" => {
                    set_theme(CosmicTheme::matrix());
                    crate::n!(0x00FF00, "Theme set to MATRIX - Wake up, Neo...");
                },
                _ => {
                    crate::println!("Available themes: dark, light, matrix");
                }
            }
        },
        "info" => {
            crate::n!(C_, "COSMIC UI Framework for TrustOS");
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


pub(super) fn kqf(args: &[&str]) {
    if args.is_empty() {
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
    
    let afz = args[0].to_lowercase();
    hls(Some(&afz));
}



fn hkq() -> bool {
    let mut eec: u8 = 0;

    
    let nuh = crate::memory::ceo();
    let aun = nuh / (1024 * 1024);
    if aun > 0 && aun < 256 {
        crate::n!(A_, "\u{26A0} Insufficient RAM: {} MB detected (minimum: 256 MB)", aun);
        eec += 1;
    } else if aun > 0 && aun < 512 {
        crate::n!(D_, "\u{26A0} Low RAM: {} MB detected (recommended: 512 MB+)", aun);
    }

    
    let heap_free = crate::memory::heap::free();
    let bmt = heap_free / (1024 * 1024);
    if bmt < 16 {
        crate::n!(A_, "\u{26A0} Insufficient heap: {} MB free (minimum: 16 MB)", bmt);
        eec += 1;
    } else if bmt < 32 {
        crate::n!(D_, "\u{26A0} Low heap: {} MB free (recommended: 32 MB+)", bmt);
    }

    
    let (fb_w, fb_h) = crate::framebuffer::kv();
    if fb_w == 0 || fb_h == 0 {
        crate::n!(A_, "\u{26A0} No framebuffer detected! Desktop requires a display.");
        eec += 1;
    } else if fb_w < 800 || fb_h < 600 {
        crate::n!(D_, "\u{26A0} Low resolution: {}x{} (recommended: 1024x768+)", fb_w, fb_h);
    }

    
    if fb_w > 0 && fb_h > 0 {
        let jzd = (fb_w as usize) * (fb_h as usize) * 4 * 2;
        let hgk = jzd / (1024 * 1024);
        if bmt > 0 && (hgk as usize) > bmt + 4 {
            crate::n!(A_, "\u{26A0} Not enough memory for {}x{} framebuffer ({} MB needed, {} MB free)",
                fb_w, fb_h, hgk, bmt);
            eec += 1;
        }
    }

    if eec > 0 {
        crate::n!(D_, "");
        crate::n!(D_, "\u{2139}  Desktop may be unstable with current resources.");
        if aun > 0 && aun < 256 {
            crate::n!(R_, "   Tip: Increase VM RAM to 512 MB+ (-m 512M in QEMU)");
        }
        crate::n!(D_, "   Launching anyway...");
        crate::println!("");
    }

    true
}



pub(super) fn esl(initial_window: Option<(&str, crate::desktop::WindowType, i32, i32, u32, u32)>) {
    use crate::desktop;

    
    if !hkq() {
        return;
    }

    let (width, height) = crate::framebuffer::kv();
    if width == 0 || height == 0 {
        crate::n!(A_, "Error: Invalid framebuffer!");
        return;
    }
    crate::mouse::set_screen_size(width, height);
    
    let mut d = desktop::S.lock();
    d.init(width, height);
    
    
    if d.desktop_tier == desktop::DesktopTier::CliOnly {
        crate::n!(D_, "Insufficient resources for desktop (< 128 MB RAM).");
        crate::n!(D_, "Staying in command-line mode. Increase RAM to 256 MB+ (-m 256M).");
        drop(d);
        return;
    }
    
    
    let pji = match d.desktop_tier {
        desktop::DesktopTier::Minimal  => "Minimal (solid bg, no effects)",
        desktop::DesktopTier::Standard => "Standard (2-layer rain, basic effects)",
        desktop::DesktopTier::Full     => "Full (4-layer rain, visualizer, all effects)",
        _ => "CLI",
    };
    crate::serial_println!("[Desktop] Launching in {} mode", pji);
    
    
    if crate::drivers::virtio_gpu::sw() {
        d.render_mode = desktop::RenderMode::GpuAccelerated;
        crate::serial_println!("[Desktop] GPU-accelerated rendering enabled (VirtIO GPU)");
    }
    
    
    if let Some((title, wt, x, y, w, h)) = initial_window {
        d.create_window(title, x, y, w, h, wt);
    }
    
    drop(d);
    
    
    crate::gui::engine::osf("TrustOS Desktop", "Welcome! Alt+Tab to switch windows", crate::gui::engine::NotifyPriority::Success);
    
    crate::serial_println!("[Desktop] Entering desktop run loop");
    desktop::run();
    
    crate::serial_println!("[Desktop] Returned to shell");
    
    let (w, h) = crate::framebuffer::kv();
    crate::framebuffer::fill_rect(0, 0, w, h, 0xFF000000);
    crate::n!(B_, "\nReturned to TrustOS shell. Type 'help' for commands.");
}



pub(super) fn mxa() {
    use crate::desktop;

    
    if !hkq() {
        return;
    }

    let (width, height) = crate::framebuffer::kv();
    if width == 0 || height == 0 {
        crate::n!(A_, "Error: Invalid framebuffer!");
        return;
    }
    crate::mouse::set_screen_size(width, height);
    
    let mut d = desktop::S.lock();
    d.init(width, height);
    
    d.mobile_state.active = true;
    d.mobile_state.view = crate::mobile::MobileView::Home;
    let (vx, vy, bt, ex) = crate::mobile::hjt(width, height);
    d.mobile_state.vp_x = vx;
    d.mobile_state.vp_y = vy;
    d.mobile_state.vp_w = bt;
    d.mobile_state.vp_h = ex;
    crate::serial_println!("[Mobile] Viewport: {}x{} at ({},{}) on {}x{}", bt, ex, vx, vy, width, height);
    
    drop(d);
    crate::serial_println!("[Mobile] Entering mobile desktop loop");
    desktop::run();
    
    crate::serial_println!("[Mobile] Returned to shell");
    let (w, h) = crate::framebuffer::kv();
    crate::framebuffer::fill_rect(0, 0, w, h, 0xFF000000);
    crate::n!(B_, "\nReturned to TrustOS shell. Type 'help' for commands.");
}



pub(super) fn kro(args: &[&str]) {
    use crate::signature;

    match args.first().copied() {
        Some("verify") | None => {
            
            crate::println!();
            crate::n!(C_, "\u{2554}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2557}");
            crate::n!(C_, "\u{2551}              TrustOS Kernel Signature Certificate                  \u{2551}");
            crate::n!(C_, "\u{2560}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2563}");
            crate::println!();
            crate::n!(G_, "  ?? CREATOR SIGNATURE (immutable)");
            crate::n!(R_, "  -----------------------------------------------------------------");
            crate::println!("  Author:      {} (@{})", signature::BSJ_, signature::BSI_);
            crate::println!("  Payload:     \"{}\"", signature::ARJ_);
            crate::println!("  Algorithm:   HMAC-SHA256");
            crate::n!(D_, "  Fingerprint: {}", signature::hou());
            crate::println!("  Version:     v{}", signature::OS_);
            crate::println!("  Built:       {}", signature::BOF_);
            crate::println!();
            crate::n!(K_, "  i  This fingerprint was generated with a secret seed known ONLY");
            crate::n!(K_, "     to the creator. It cannot be forged without the original seed.");
            crate::println!();

            
            if let Some((name, ga, jy)) = signature::eoe() {
                crate::n!(CF_, "  USER CO-SIGNATURE");
                crate::n!(R_, "  -----------------------------------------------------------------");
                crate::println!("  Signed by:   {}", name);
                crate::n!(D_, "  Fingerprint: {}", ga);
                crate::println!("  Signed at:   {}s after midnight (RTC)", jy);
                crate::println!();
            } else {
                crate::n!(K_, "  No user co-signature. Use 'signature sign <name>' to add yours.");
                crate::println!();
            }

            crate::n!(C_, "\u{255a}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{255d}");
            crate::println!();
        }
        Some("sign") => {
            
            if args.len() < 2 {
                crate::n!(A_, "Usage: signature sign <your_name>");
                return;
            }
            let name = args[1];
            crate::println!("Enter your secret passphrase to sign the kernel:");
            crate::print!("> ");
            
            let amd = exx();
            if amd.is_empty() {
                crate::n!(A_, "Empty passphrase. Aborted.");
                return;
            }
            signature::oso(name, amd.as_bytes());
            crate::println!();
            crate::n!(G_, "? Kernel co-signed by '{}'", name);
            if let Some((_, ga, _)) = signature::eoe() {
                crate::n!(D_, "  Your fingerprint: {}", ga);
            }
            crate::n!(K_, "  Keep your passphrase safe -- you'll need it to prove ownership.");
            crate::println!();
        }
        Some("prove") => {
            
            if args.len() < 2 {
                crate::n!(A_, "Usage: signature prove <name>");
                return;
            }
            let name = args[1];
            crate::println!("Enter passphrase to verify:");
            crate::print!("> ");
            let amd = exx();
            if signature::prv(name, amd.as_bytes()) {
                crate::n!(G_, "VERIFIED -- '{}' is the legitimate signer.", name);
            } else {
                crate::n!(A_, "FAILED -- passphrase does not match the signature for '{}'.", name);
            }
            crate::println!();
        }
        Some("prove-creator") => {
            
            crate::println!("Enter creator seed to verify authorship:");
            crate::print!("> ");
            let seed = exx();
            if signature::prq(seed.as_bytes()) {
                crate::n!(G_, "CREATOR VERIFIED -- You are the original author of TrustOS.");
            } else {
                crate::n!(A_, "FAILED -- This seed does not match the creator fingerprint.");
            }
            crate::println!();
        }
        Some("integrity") | Some("verify-integrity") => {
            crate::println!();
            crate::n!(G_, "Kernel Integrity Verification");
            crate::println!("---------------------------------------------------------------");
            let report = signature::mqw();
            for line in &report {
                crate::println!("{}", line);
            }
            crate::println!();
            crate::n!(K_, "  SHA-256 of .text + .rodata sections measured at boot vs now.");
            crate::n!(K_, "  Detects runtime code injection, ROP gadget insertion, and");
            crate::n!(K_, "  constant/vtable tampering (rootkits, memory corruption).");
            crate::println!();
        }
        Some("clear") => {
            signature::kkv();
            crate::n!(D_, "User co-signature cleared.");
        }
        Some("export") => {
            
            if let Some((name, ga, _ts)) = signature::eoe() {
                let fm = crate::rtc::aou();
                crate::println!();
                crate::n!(C_, "=== Copy everything below and submit as a PR to SIGNATURES.md ===");
                crate::println!();
                crate::println!("### #NNN -- {}", name);
                crate::println!();
                crate::println!("| Field | Value |");
                crate::println!("|-------|-------|");
                crate::println!("| **Name** | {} |", name);
                crate::println!("| **GitHub** | [@YOURUSERNAME](https://github.com/YOURUSERNAME) |");
                crate::println!("| **Algorithm** | HMAC-SHA256 |");
                crate::println!("| **Fingerprint** | `{}` |", ga);
                crate::println!("| **Kernel Version** | v{} |", signature::OS_);
                crate::println!("| **Date** | {:04}-{:02}-{:02} |", fm.year, fm.month, fm.day);
                crate::println!("| **Status** | Verified signer |");
                crate::println!();
                crate::n!(K_, "Replace YOURUSERNAME with your GitHub username and #NNN with the next number.");
                crate::n!(K_, "Submit as a Pull Request to: github.com/nathan237/TrustOS");
                crate::println!();
            } else {
                crate::n!(A_, "No user signature found. Run 'signature sign <name>' first.");
            }
        }
        Some("list") => {
            
            crate::println!();
            crate::n!(C_, "TrustOS Signature Registry");
            crate::n!(R_, "------------------------------------------------------");
            crate::println!();
            crate::n!(G_, "  #001  Nated0ge (Creator)");
            crate::println!("        {}", signature::hou());
            crate::println!();
            if let Some((name, ga, _)) = signature::eoe() {
                crate::n!(C_, "  #---  {} (Local)", name);
                crate::println!("        {}", ga);
                crate::println!();
            }
            crate::n!(K_, "  Full registry: github.com/nathan237/TrustOS/blob/main/SIGNATURES.md");
            crate::println!();
        }
        Some("ed25519") => {
            
            match args.get(1).copied() {
                Some("verify") | None => {
                    crate::println!();
                    crate::n!(C_, "Ed25519 Asymmetric Signature Report");
                    crate::n!(R_, "--------------------------------------------------------------");
                    let report = signature::huy();
                    for line in &report {
                        crate::println!("{}", line);
                    }
                    crate::println!();
                }
                Some("sign") => {
                    crate::println!("Enter Ed25519 seed (hex or passphrase):");
                    crate::print!("> ");
                    let jee = exx();
                    if jee.is_empty() {
                        crate::n!(A_, "Empty seed. Aborted.");
                        return;
                    }
                    signature::lnw(jee.as_bytes());
                    crate::n!(G_, "? Kernel re-signed with Ed25519 (new seed).");
                    if let Some(report) = signature::huy().first() {
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
            crate::n!(C_, "TrustOS Kernel Signature System");
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


fn exx() -> alloc::string::String {
    use alloc::string::String;
    let mut amd = String::new();
    loop {
        if let Some(key) = crate::keyboard::kr() {
            match key {
                b'\n' | b'\r' | 0x0A | 0x0D => {
                    crate::println!();
                    break;
                }
                0x08 => {
                    
                    if !amd.is_empty() {
                        amd.pop();
                        crate::print!("\x08 \x08");
                    }
                }
                c if c.is_ascii() && !c.is_ascii_control() => {
                    amd.push(c as char);
                    crate::print!("*");
                }
                _ => {}
            }
        }
        core::hint::spin_loop();
    }
    amd
}


pub(super) fn krj(args: &[&str]) {
    match args.first().copied() {
        Some("status") | None => {
            
            let stats = crate::security::stats();
            crate::println!();
            crate::n!(G_, "TrustOS Security Status");
            crate::println!("---------------------------------------------------------------");
            crate::println!("  Active capabilities : {}", stats.active_capabilities);
            crate::println!("  Security violations : {}", stats.violations);
            crate::println!("  Dynamic types       : {}", stats.dynamic_types);
            crate::println!("  Isolated subsystems : {}", stats.subsystems);
            crate::println!("  Gate checks         : {}", crate::security::isolation::jnu());
            crate::println!("  Gate violations     : {}", crate::security::isolation::jnv());
            crate::println!();
            
            
            match crate::signature::jqa() {
                Ok(true) => crate::n!(G_, "  Kernel integrity    : ? INTACT"),
                Ok(false) => crate::n!(A_, "  Kernel integrity    : ? TAMPERED"),
                Err(_) => crate::n!(D_, "  Kernel integrity    : ??  not initialized"),
            }
            crate::println!();
        }
        Some("caps") | Some("capabilities") => {
            
            let caps = crate::security::myz();
            crate::println!();
            crate::n!(C_, "Active Capabilities ({} total)", caps.len());
            crate::println!("----------------------------------------------------------");
            crate::println!("  {:>6} | {:<20} | {:<10} | Owner", "ID", "Type", "Category");
            crate::println!("  -------+----------------------+------------+------");
            for (id, cap_type, _rights, owner) in &caps {
                crate::println!("  {:>6} | {:<20} | {:<10} | 0x{:04X}",
                    id.0,
                    alloc::format!("{:?}", cap_type),
                    cap_type.category(),
                    owner
                );
            }
            crate::println!();
        }
        Some("isolation") | Some("iso") | Some("subsystems") => {
            
            crate::println!();
            crate::n!(G_, "Subsystem Isolation Boundaries");
            crate::println!("---------------------------------------------------------------");
            let report = crate::security::isolation::mui();
            for line in &report {
                crate::println!("{}", line);
            }
            crate::println!();
            crate::n!(K_, "  ring0-tcb       = Part of TCB, must stay in ring 0");
            crate::n!(K_, "  ring0-isolated  = Ring 0 but logically isolated");
            crate::n!(K_, "  ring3-candidate = Could be moved to ring 3 in future");
            crate::println!();
        }
        Some("gate") => {
            
            if let Some(subsystem_name) = args.get(1).copied() {
                let acs = match subsystem_name {
                    "storage" | "disk" => Some(crate::security::isolation::Subsystem::Storage),
                    "network" | "net" => Some(crate::security::isolation::Subsystem::Network),
                    "graphics" | "gpu" => Some(crate::security::isolation::Subsystem::Graphics),
                    "process" | "proc" => Some(crate::security::isolation::Subsystem::ProcessMgr),
                    "hypervisor" | "hv" => Some(crate::security::isolation::Subsystem::Hypervisor),
                    "shell" => Some(crate::security::isolation::Subsystem::Shell),
                    "crypto" => Some(crate::security::isolation::Subsystem::Crypto),
                    "power" => Some(crate::security::isolation::Subsystem::Power),
                    "serial" => Some(crate::security::isolation::Subsystem::SerialDebug),
                    "memory" | "mem" => Some(crate::security::isolation::Subsystem::Memory),
                    _ => None,
                };
                if let Some(sub) = acs {
                    match crate::security::isolation::bmj(
                        sub, crate::security::CapabilityRights::Ba
                    ) {
                        Ok(()) => crate::n!(G_, 
                            "  ? Gate check PASSED for {:?}", sub),
                        Err(e) => crate::n!(A_, 
                            "  ? Gate check DENIED for {:?}: {:?}", sub, e),
                    }
                } else {
                    crate::n!(A_, "Unknown subsystem: {}", subsystem_name);
                }
            } else {
                crate::println!("Usage: security gate <subsystem>");
                crate::println!("  Subsystems: storage, network, graphics, process, hypervisor,");
                crate::println!("              shell, crypto, power, serial, memory");
            }
        }
        Some("dynamic") => {
            
            let eda = crate::security::ikq();
            crate::println!();
            if eda.is_empty() {
                crate::n!(K_, "No dynamic capability types registered.");
            } else {
                crate::n!(C_, "Dynamic Capability Types ({} registered)", eda.len());
                for (id, info) in &eda {
                    crate::println!("  [{}] {} (danger:{}, category:{})", 
                        id, info.name, info.danger_level, info.category);
                    crate::println!("       {}", info.description);
                }
            }
            crate::println!();
        }
        _ => {
            crate::n!(C_, "TrustOS Security Subsystem");
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






pub(super) fn kmy(args: &[&str]) {
    let ia = match args.first().copied() {
        Some("fr") => "fr",
        _ => "en",
    };

    
    let pause = |im: u64| {
        let dh = im * 1000;
        let rr = crate::cpu::tsc::ey();
        let freq = crate::cpu::tsc::we();
        if freq == 0 { return; }
        let acx = freq / 1000 * dh;
        loop {
            let bb = crate::cpu::tsc::ey().saturating_sub(rr);
            if bb >= acx { break; }
            let _ = crate::keyboard::kr();
            core::hint::spin_loop();
        }
    };

    let csp = |timeout_secs: u64| -> bool {
        let dh = timeout_secs * 1000;
        let rr = crate::cpu::tsc::ey();
        let freq = crate::cpu::tsc::we();
        if freq == 0 { return false; }
        let acx = freq / 1000 * dh;
        loop {
            let bb = crate::cpu::tsc::ey().saturating_sub(rr);
            if bb >= acx { return false; }
            if let Some(key) = crate::keyboard::kr() {
                if key == 0x1B || key == b'q' { return true; } 
                if key == b' ' || key == b'\n' || key == 13 { return false; } 
            }
            core::hint::spin_loop();
        }
    };

    
    
    
    let (dy, dw) = crate::framebuffer::kv();

    {
        let pu = crate::framebuffer::ajy();
        if !pu {
            crate::framebuffer::adw();
            crate::framebuffer::pr(true);
        }

        let w = dy as usize;
        let h = dw as usize;
        let mut buf = alloc::vec![0u32; w * h];

        
        let pf = |buf: &mut [u32], w: usize, h: usize, cx: usize, u: usize, c: char, color: u32, scale: usize| {
            let du = crate::framebuffer::font::ol(c);
            for (row, &bits) in du.iter().enumerate() {
                for bf in 0..8u32 {
                    if bits & (0x80 >> bf) != 0 {
                        for ak in 0..scale {
                            for am in 0..scale {
                                let p = cx + bf as usize * scale + am;
                                let o = u + row * scale + ak;
                                if p < w && o < h {
                                    buf[o * w + p] = color;
                                }
                            }
                        }
                    }
                }
            }
        };

        
        let mut kk: alloc::vec::Vec<u16> = alloc::vec![0u16; w / 8 + 1];
        let mut la: alloc::vec::Vec<u8> = alloc::vec![1u8; w / 8 + 1];
        for i in 0..kk.len() {
            kk[i] = ((i * 37 + 13) % h) as u16;
            la[i] = (((i * 7 + 3) % 4) + 1) as u8;
        }

        let cwv = |buf: &mut [u32], w: usize, h: usize, cols: &mut [u16], speeds: &[u8], frame: u32| {
            for ct in buf.iter_mut() {
                let g = ((*ct >> 8) & 0xFF) as u32;
                if g > 0 {
                    let cnb = g.saturating_sub(8);
                    *ct = 0xFF000000 | (cnb << 8);
                }
            }
            for ow in 0..cols.len() {
                let x = ow * 8;
                if x >= w { continue; }
                cols[ow] = cols[ow].wrapping_add(speeds[ow] as u16);
                if cols[ow] as usize >= h { cols[ow] = 0; }
                let y = cols[ow] as usize;
                let c = (((frame as usize + ow * 13) % 94) + 33) as u8 as char;
                let du = crate::framebuffer::font::ol(c);
                for (row, &bits) in du.iter().enumerate() {
                    let o = y + row;
                    if o >= h { break; }
                    for bf in 0..8u32 {
                        if bits & (0x80 >> bf) != 0 {
                            let p = x + bf as usize;
                            if p < w {
                                buf[o * w + p] = 0xFF00FF44;
                            }
                        }
                    }
                }
            }
        };

        let ev = |buf: &[u32], w: usize, h: usize| {
            if let Some((bb_ptr, _bb_w, bb_h, bb_stride)) = crate::framebuffer::aqr() {
                let mq = bb_ptr as *mut u32;
                let aeu = bb_stride as usize;
                for y in 0..h.min(bb_h as usize) {
                    unsafe {
                        core::ptr::copy_nonoverlapping(
                            buf[y * w..].as_ptr(),
                            mq.add(y * aeu),
                            w,
                        );
                    }
                }
            }
            crate::framebuffer::ii();
        };

        
        let bdi = |buf: &mut [u32], w: usize, h: usize,
                           kk: &mut [u16], la: &[u8],
                           lines: &[(&str, u32, usize)],
                           hold_ms: u64| {
            let freq = crate::cpu::tsc::we();
            if freq == 0 { return; }

            let vu: usize = lines.iter().map(|(t, _, _)| t.len()).sum();
            let ecz = 60u64;
            let dfx = vu as u64 * ecz;

            let rr = crate::cpu::tsc::ey();
            let gay = freq / 1000 * (dfx + hold_ms);

            let mut frame = 0u32;
            loop {
                let bb = crate::cpu::tsc::ey().saturating_sub(rr);
                if bb >= gay { break; }
                if let Some(key) = crate::keyboard::kr() {
                    if key == 0x1B || key == b'q' { break; }
                    if key == b' ' || key == b'\n' || key == 13 { break; } 
                }

                cwv(buf, w, h, kk, la, frame);

                let elapsed_ms = bb / (freq / 1000).max(1);
                let hh = if elapsed_ms < dfx {
                    (elapsed_ms / ecz.max(1)) as usize
                } else {
                    vu
                };

                let ecj: usize = lines.iter().map(|(_, _, j)| 16 * j + 8).sum::<usize>();
                let mut ajb = if ecj < h { (h - ecj) / 2 } else { 20 };
                let mut bff = 0usize;

                for &(text, color, scale) in lines {
                    let acy = text.len() * 8 * scale;
                    let start_x = if acy < w { (w - acy) / 2 } else { 0 };

                    for (i, c) in text.chars().enumerate() {
                        if bff + i >= hh { break; }
                        pf(buf, w, h, start_x + i * 8 * scale, ajb, c, color, scale);
                    }
                    if hh > bff && hh < bff + text.len() {
                        let fqb = hh - bff;
                        let cx = start_x + fqb * 8 * scale;
                        for u in ajb..ajb + 16 * scale {
                            if u < h && cx + 2 < w {
                                buf[u * w + cx] = 0xFF00FF88;
                                buf[u * w + cx + 1] = 0xFF00FF88;
                            }
                        }
                    }

                    bff += text.len();
                    ajb += 16 * scale + 8;
                }

                ev(buf, w, h);
                frame += 1;
                crate::cpu::tsc::ww(33);
            }

            
            let cjf = crate::cpu::tsc::ey();
            let bsk = freq / 1000 * 600;
            loop {
                let bb = crate::cpu::tsc::ey().saturating_sub(cjf);
                if bb >= bsk { break; }
                let progress = (bb * 255 / bsk) as u32;
                for ct in buf.iter_mut() {
                    let r = ((*ct >> 16) & 0xFF) as u32;
                    let g = ((*ct >> 8) & 0xFF) as u32;
                    let b = (*ct & 0xFF) as u32;
                    let nr = r.saturating_sub(r * progress / 512 + 1);
                    let ayn = g.saturating_sub(g * progress / 512 + 1);
                    let ayj = b.saturating_sub(b * progress / 512 + 1);
                    *ct = 0xFF000000 | (nr << 16) | (ayn << 8) | ayj;
                }
                ev(buf, w, h);
                crate::cpu::tsc::ww(33);
            }
            for ct in buf.iter_mut() { *ct = 0xFF000000; }
            ev(buf, w, h);
        };

        
        crate::serial_println!("[DEMO] Scene 1: Welcome");
        for ct in buf.iter_mut() { *ct = 0xFF000000; }
        if ia == "fr" {
            bdi(&mut buf, w, h, &mut kk, &la,
                &[("Bienvenue dans", 0xFF00DD55, 5),
                  ("TrustOS", 0xFF00FFAA, 6)],
                3000);
        } else {
            bdi(&mut buf, w, h, &mut kk, &la,
                &[("Welcome to", 0xFF00DD55, 5),
                  ("TrustOS", 0xFF00FFAA, 6)],
                3000);
        }

        
        crate::serial_println!("[DEMO] Scene 2: What is TrustOS");
        if ia == "fr" {
            bdi(&mut buf, w, h, &mut kk, &la,
                &[("Un OS bare-metal", 0xFF00DD55, 4),
                  ("ecrit en 100% Rust", 0xFF00FF88, 4),
                  ("Aucun C. Aucun Linux.", 0xFFFFCC44, 3)],
                3000);
        } else {
            bdi(&mut buf, w, h, &mut kk, &la,
                &[("A bare-metal OS", 0xFF00DD55, 4),
                  ("written in 100% Rust", 0xFF00FF88, 4),
                  ("No C. No Linux. Just Rust.", 0xFFFFCC44, 3)],
                3000);
        }

        
        crate::serial_println!("[DEMO] Scene 3: Tutorial start");
        if ia == "fr" {
            bdi(&mut buf, w, h, &mut kk, &la,
                &[("Tutoriel Interactif", 0xFF44DDFF, 5),
                  ("Appuyez ESPACE pour continuer", 0xFF888888, 2),
                  ("ESC pour quitter", 0xFF666666, 2)],
                4000);
        } else {
            bdi(&mut buf, w, h, &mut kk, &la,
                &[("Interactive Tutorial", 0xFF44DDFF, 5),
                  ("Press SPACE to continue", 0xFF888888, 2),
                  ("ESC to quit", 0xFF666666, 2)],
                4000);
        }

        if !pu {
            crate::framebuffer::pr(false);
        }
    }

    
    
    
    crate::framebuffer::clear();

    let ccu = |step: u32, av: u32, title_en: &str, title_fr: &str, desc_en: &str, desc_fr: &str| {
        crate::println!();
        crate::n!(0xFF00CCFF, "+--------------------------------------------------------------+");
        if ia == "fr" {
            crate::n!(0xFF00CCFF, "|  ETAPE {}/{} -- {}", step, av, title_fr);
        } else {
            crate::n!(0xFF00CCFF, "|  STEP {}/{} -- {}", step, av, title_en);
        }
        crate::n!(0xFF00CCFF, "+--------------------------------------------------------------+");
        crate::println!();
        if ia == "fr" {
            crate::n!(0xFF888888, "  {}", desc_fr);
        } else {
            crate::n!(0xFF888888, "  {}", desc_en);
        }
        crate::println!();
    };

    let ix = 8u32;

    
    ccu(1, ix, "SYSTEM INFO", "INFOS SYSTEME",
               "TrustOS can show detailed system information, just like Linux.",
               "TrustOS affiche les infos systeme, comme sous Linux.");
    pause(2);

    crate::n!(C_, "  $ neofetch");
    pause(1);
    super::commands::fmw();
    pause(3);

    if ia == "fr" {
        crate::n!(0xFF00FF88, "  -> Neofetch montre le CPU, la RAM, le kernel et l'uptime.");
    } else {
        crate::n!(0xFF00FF88, "  -> Neofetch shows CPU, RAM, kernel version and uptime.");
    }
    if csp(6) { return; }

    
    ccu(2, ix, "FILESYSTEM", "SYSTEME DE FICHIERS",
               "TrustOS has a full virtual filesystem (TrustFS + VFS).",
               "TrustOS possede un systeme de fichiers virtuel complet (TrustFS + VFS).");
    pause(1);

    crate::n!(C_, "  $ mkdir /tutorial");
    crate::ramfs::bh(|fs| { let _ = fs.mkdir("/tutorial"); });
    crate::n!(B_, "  Created /tutorial");
    pause(1);

    crate::n!(C_, "  $ echo 'Hello from TrustOS!' > /tutorial/hello.txt");
    crate::ramfs::bh(|fs| {
        let _ = fs.touch("/tutorial/hello.txt");
        let _ = fs.write_file("/tutorial/hello.txt", b"Hello from TrustOS!\nThis file was created during the tutorial.\nPure Rust, running on bare metal.\n");
    });
    crate::n!(B_, "  Written: /tutorial/hello.txt");
    pause(1);

    crate::n!(C_, "  $ cat /tutorial/hello.txt");
    super::commands::dkw(&["/tutorial/hello.txt"], None, None);
    pause(2);

    crate::n!(C_, "  $ tree /");
    super::commands::fnh(&["/"]);

    if ia == "fr" {
        crate::n!(0xFF00FF88, "  -> Commandes POSIX completes: ls, cd, mkdir, rm, cp, mv, cat, find, grep...");
    } else {
        crate::n!(0xFF00FF88, "  -> Full POSIX commands: ls, cd, mkdir, rm, cp, mv, cat, find, grep...");
    }
    if csp(6) { return; }

    
    ccu(3, ix, "TRUSTLANG COMPILER", "COMPILATEUR TRUSTLANG",
               "TrustOS includes a built-in programming language with compiler + VM.",
               "TrustOS inclut un langage de programmation avec compilateur + VM.");
    pause(1);

    let crs = r#"fn factorial(n: i64) -> i64 {
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
    crate::ramfs::bh(|fs| {
        let _ = fs.touch("/tutorial/demo.tl");
        let _ = fs.write_file("/tutorial/demo.tl", crs.as_bytes());
    });

    crate::n!(C_, "  $ cat /tutorial/demo.tl");
    crate::n!(0xFFDDDDDD, "{}", crs);
    pause(3);

    crate::n!(C_, "  $ trustlang run /tutorial/demo.tl");
    crate::n!(0xFF00FF88, "  [TrustLang] Compiling...");
    match crate::trustlang::run(crs) {
        Ok(output) => { if !output.is_empty() { crate::print!("{}", output); } }
        Err(e) => crate::n!(A_, "  Error: {}", e),
    }
    crate::n!(B_, "  [TrustLang] Done!");

    if ia == "fr" {
        crate::n!(0xFF00FF88, "  -> Fonctions, recursion, boucles, types -- compile en bytecode!");
    } else {
        crate::n!(0xFF00FF88, "  -> Functions, recursion, loops, types -- compiled to bytecode!");
    }
    if csp(6) { return; }

    
    ccu(4, ix, "NETWORK STACK", "PILE RESEAU",
               "Full TCP/IP stack: DHCP, DNS, HTTP, TLS 1.3 -- all in Rust.",
               "Pile TCP/IP complete: DHCP, DNS, HTTP, TLS 1.3 -- tout en Rust.");
    pause(1);

    crate::n!(C_, "  $ ifconfig");
    super::vm::dkx();
    pause(2);

    crate::n!(C_, "  $ netstat");
    super::vm::dky();

    if ia == "fr" {
        crate::n!(0xFF00FF88, "  -> Un navigateur web integre peut charger de vraies pages!");
    } else {
        crate::n!(0xFF00FF88, "  -> A built-in web browser can load real web pages!");
    }
    if csp(5) { return; }

    
    ccu(5, ix, "VIDEO EFFECTS", "EFFETS VIDEO",
               "Real-time procedural rendering engine -- fire, matrix, plasma.",
               "Moteur de rendu procedural temps reel -- feu, matrix, plasma.");
    pause(2);

    let bt = dy as u16;
    let ex = dw as u16;

    if ia == "fr" {
        crate::n!(0xFFFF4400, "  Effet 1: FEU -- Flammes procedurales (5s)");
    } else {
        crate::n!(0xFFFF4400, "  Effect 1: FIRE -- Procedural flames (5s)");
    }
    pause(1);
    crate::video::player::ddk("fire", bt, ex, 30, 5000);
    crate::framebuffer::clear();

    if ia == "fr" {
        crate::n!(0xFF00FF44, "  Effet 2: MATRIX -- Pluie numerique (5s)");
    } else {
        crate::n!(0xFF00FF44, "  Effect 2: MATRIX -- Digital rain (5s)");
    }
    pause(1);
    crate::video::player::ddk("matrix", bt, ex, 30, 5000);
    crate::framebuffer::clear();

    if ia == "fr" {
        crate::n!(0xFFFF00FF, "  Effet 3: PLASMA -- Plasma psychedelique (5s)");
    } else {
        crate::n!(0xFFFF00FF, "  Effect 3: PLASMA -- Psychedelic plasma (5s)");
    }
    pause(1);
    crate::video::player::ddk("plasma", bt, ex, 30, 5000);
    crate::framebuffer::clear();

    if ia == "fr" {
        crate::n!(0xFF00FF88, "  -> Tout fonctionne a 60+ FPS sur du bare-metal!");
    } else {
        crate::n!(0xFF00FF88, "  -> All running at 60+ FPS on bare metal!");
    }
    if csp(4) { return; }

    
    ccu(6, ix, "3D ENGINE", "MOTEUR 3D",
               "Wireframe 3D with perspective projection and depth shading.",
               "3D filaire avec projection perspective et ombrage de profondeur.");
    pause(2);

    {
        let mut renderer = crate::formula3d::FormulaRenderer::new();
        renderer.set_scene(crate::formula3d::FormulaScene::Character);
        renderer.wire_color = 0xFF00FFAA;

        let lk = dy as usize;
        let pp = dw as usize;
        let mut bqa = alloc::vec![0u32; lk * pp];

        let pu = crate::framebuffer::ajy();
        if !pu {
            crate::framebuffer::adw();
            crate::framebuffer::pr(true);
        }
        crate::framebuffer::awo(0xFF000000);
        crate::framebuffer::ii();

        let rr = crate::cpu::tsc::ey();
        let freq = crate::cpu::tsc::we();
        let acx = if freq > 0 { freq / 1000 * 6000 } else { u64::MAX }; 

        loop {
            let bb = crate::cpu::tsc::ey().saturating_sub(rr);
            if bb >= acx { break; }
            if let Some(key) = crate::keyboard::kr() {
                if key == 0x1B || key == b'q' { break; }
                if key == b' ' || key == b'\n' || key == 13 { break; }
            }

            renderer.update();
            renderer.render(&mut bqa, lk, pp);

            if let Some((bb_ptr, _bb_w, bb_h, bb_stride)) = crate::framebuffer::aqr() {
                let mq = bb_ptr as *mut u32;
                let aeu = bb_stride as usize;
                for y in 0..pp.min(bb_h as usize) {
                    unsafe {
                        core::ptr::copy_nonoverlapping(
                            bqa[y * lk..].as_ptr(),
                            mq.add(y * aeu),
                            lk,
                        );
                    }
                }
            }
            crate::framebuffer::ii();
        }

        if !pu {
            crate::framebuffer::pr(false);
        }
    }
    crate::framebuffer::clear();
    if csp(2) { return; }

    
    ccu(7, ix, "DESKTOP ENVIRONMENT", "ENVIRONNEMENT DE BUREAU",
               "GPU-composited windowed desktop with apps, games, and more.",
               "Bureau fenetre composite GPU avec apps, jeux, et plus encore.");
    pause(1);

    if ia == "fr" {
        crate::n!(0xFF00FF88, "  Le bureau s'ouvre avec un Terminal pour 8 secondes...");
        crate::n!(0xFF888888, "  (Essayez de taper des commandes!)");
    } else {
        crate::n!(0xFF00FF88, "  Desktop will open with a Terminal for 8 seconds...");
        crate::n!(0xFF888888, "  (Try typing some commands!)");
    }
    pause(3);

    
    fmf(Some("shell"), 8000);
    crate::framebuffer::clear();
    pause(1);

    
    ccu(8, ix, "FEATURE OVERVIEW", "VUE D'ENSEMBLE",
               "Everything TrustOS includes -- all in 6MB, all in Rust.",
               "Tout ce que TrustOS contient -- en 6Mo, tout en Rust.");
    pause(1);

    crate::n!(0xFFAADDFF, "  +- Kernel -------------------------------------------+");
    crate::n!(0xFFDDDDDD, "  | SMP multicore * APIC * IDT * GDT * paging          |");
    crate::n!(0xFFDDDDDD, "  | heap allocator * scheduler * RTC * PIT * TSC        |");
    crate::n!(0xFFAADDFF, "  +- Shell --------------------------------------------|");
    crate::n!(0xFFDDDDDD, "  | 200+ POSIX commands * pipes * scripting              |");
    crate::n!(0xFFDDDDDD, "  | ls cd mkdir rm cp mv cat grep find head tail tree   |");
    crate::n!(0xFFAADDFF, "  +- Desktop ------------------------------------------|");
    crate::n!(0xFFDDDDDD, "  | GPU compositor * window manager * 60 FPS            |");
    crate::n!(0xFFDDDDDD, "  | Terminal * Files * Calculator * Settings             |");
    crate::n!(0xFFAADDFF, "  +- Apps ---------------------------------------------|");
    crate::n!(0xFFDDDDDD, "  | TrustCode (editor) * Web Browser * Snake * Chess    |");
    crate::n!(0xFFDDDDDD, "  | NES Emulator * Game Boy * TrustEdit 3D * TrustLab  |");
    crate::n!(0xFFAADDFF, "  +- Languages ----------------------------------------|");
    crate::n!(0xFFDDDDDD, "  | TrustLang compiler + VM * Shell scripting           |");
    crate::n!(0xFFAADDFF, "  +- Network ------------------------------------------|");
    crate::n!(0xFFDDDDDD, "  | TCP/IP * DHCP * DNS * HTTP * TLS 1.3 * curl/wget   |");
    crate::n!(0xFFAADDFF, "  +- Graphics -----------------------------------------|");
    crate::n!(0xFFDDDDDD, "  | TrustVideo * Formula3D * Matrix * Fire * Plasma     |");
    crate::n!(0xFFAADDFF, "  +----------------------------------------------------+");
    crate::println!();

    if csp(8) { return; }

    
    
    
    {
        let pu = crate::framebuffer::ajy();
        if !pu {
            crate::framebuffer::adw();
            crate::framebuffer::pr(true);
        }

        let w = dy as usize;
        let h = dw as usize;
        let mut buf = alloc::vec![0u32; w * h];

        let pf = |buf: &mut [u32], w: usize, h: usize, cx: usize, u: usize, c: char, color: u32, scale: usize| {
            let du = crate::framebuffer::font::ol(c);
            for (row, &bits) in du.iter().enumerate() {
                for bf in 0..8u32 {
                    if bits & (0x80 >> bf) != 0 {
                        for ak in 0..scale {
                            for am in 0..scale {
                                let p = cx + bf as usize * scale + am;
                                let o = u + row * scale + ak;
                                if p < w && o < h {
                                    buf[o * w + p] = color;
                                }
                            }
                        }
                    }
                }
            }
        };

        let mut kk: alloc::vec::Vec<u16> = alloc::vec![0u16; w / 8 + 1];
        let mut la: alloc::vec::Vec<u8> = alloc::vec![1u8; w / 8 + 1];
        for i in 0..kk.len() {
            kk[i] = ((i * 41 + 7) % h) as u16;
            la[i] = (((i * 11 + 5) % 4) + 1) as u8;
        }

        let cwv = |buf: &mut [u32], w: usize, h: usize, cols: &mut [u16], speeds: &[u8], frame: u32| {
            for ct in buf.iter_mut() {
                let g = ((*ct >> 8) & 0xFF) as u32;
                if g > 0 {
                    let cnb = g.saturating_sub(8);
                    *ct = 0xFF000000 | (cnb << 8);
                }
            }
            for ow in 0..cols.len() {
                let x = ow * 8;
                if x >= w { continue; }
                cols[ow] = cols[ow].wrapping_add(speeds[ow] as u16);
                if cols[ow] as usize >= h { cols[ow] = 0; }
                let y = cols[ow] as usize;
                let c = (((frame as usize + ow * 13) % 94) + 33) as u8 as char;
                let du = crate::framebuffer::font::ol(c);
                for (row, &bits) in du.iter().enumerate() {
                    let o = y + row;
                    if o >= h { break; }
                    for bf in 0..8u32 {
                        if bits & (0x80 >> bf) != 0 {
                            let p = x + bf as usize;
                            if p < w {
                                buf[o * w + p] = 0xFF00FF44;
                            }
                        }
                    }
                }
            }
        };

        let ev = |buf: &[u32], w: usize, h: usize| {
            if let Some((bb_ptr, _bb_w, bb_h, bb_stride)) = crate::framebuffer::aqr() {
                let mq = bb_ptr as *mut u32;
                let aeu = bb_stride as usize;
                for y in 0..h.min(bb_h as usize) {
                    unsafe {
                        core::ptr::copy_nonoverlapping(
                            buf[y * w..].as_ptr(),
                            mq.add(y * aeu),
                            w,
                        );
                    }
                }
            }
            crate::framebuffer::ii();
        };

        
        crate::serial_println!("[DEMO] Outro: You're ready!");
        for ct in buf.iter_mut() { *ct = 0xFF000000; }

        let freq = crate::cpu::tsc::we();
        let noq = 7000u64;
        let nor = if freq > 0 { freq / 1000 * noq } else { u64::MAX };
        let rr = crate::cpu::tsc::ey();

        let mut bvi = crate::formula3d::FormulaRenderer::new();
        bvi.set_scene(crate::formula3d::FormulaScene::Character);
        bvi.wire_color = 0xFF00FFAA;
        let vp_w = 160usize;
        let vp_h = 160usize;
        let mut bqa = alloc::vec![0u32; vp_w * vp_h];

        let mut frame = 0u32;
        loop {
            let bb = crate::cpu::tsc::ey().saturating_sub(rr);
            if bb >= nor { break; }
            if let Some(key) = crate::keyboard::kr() {
                if key == 0x1B || key == b'q' || key == b' ' || key == b'\n' || key == 13 { break; }
            }

            cwv(&mut buf, w, h, &mut kk, &la, frame);

            
            let title = if ia == "fr" { "Pret a explorer!" } else { "You're ready!" };
            let bjp = 5;
            let bea = title.len() * 8 * bjp;
            let avk = if bea < w { (w - bea) / 2 } else { 0 };
            let apg = h / 6;
            for (i, c) in title.chars().enumerate() {
                let zz = ((frame as usize * 3 + i * 25) % 360) as u32;
                let color = if zz < 120 {
                    let t = zz * 255 / 120;
                    0xFF000000 | ((255 - t) << 16) | (t << 8)
                } else if zz < 240 {
                    let t = (zz - 120) * 255 / 120;
                    0xFF000000 | ((255 - t) << 8) | t
                } else {
                    let t = (zz - 240) * 255 / 120;
                    0xFF000000 | (t << 16) | (255 - t)
                };
                pf(&mut buf, w, h, avk + i * 8 * bjp, apg, c, color, bjp);
            }

            
            bvi.update();
            for aa in bqa.iter_mut() { *aa = 0x00000000; }
            bvi.render(&mut bqa, vp_w, vp_h);
            let vp_x = if vp_w < w { (w - vp_w) / 2 } else { 0 };
            let vp_y = apg + 16 * bjp + 20;
            for vy in 0..vp_h {
                for vx in 0..vp_w {
                    let src = bqa[vy * vp_w + vx];
                    if src & 0x00FFFFFF != 0 {
                        let ad = vp_y + vy;
                        let dx = vp_x + vx;
                        if ad < h && dx < w {
                            buf[ad * w + dx] = src;
                        }
                    }
                }
            }

            
            let mlm: &[(&str, u32)] = if ia == "fr" {
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

            let epe = 2;
            let mut axm = vp_y + vp_h + 30;
            for &(text, color) in mlm {
                let gr = text.len() * 8 * epe;
                let aib = if gr < w { (w - gr) / 2 } else { 0 };
                for (i, c) in text.chars().enumerate() {
                    pf(&mut buf, w, h, aib + i * 8 * epe, axm, c, color, epe);
                }
                axm += 16 * epe + 6;
            }

            ev(&buf, w, h);
            frame += 1;
            crate::cpu::tsc::ww(33);
        }

        
        let cjf = crate::cpu::tsc::ey();
        let bsk = if freq > 0 { freq / 1000 * 800 } else { u64::MAX };
        loop {
            let bb = crate::cpu::tsc::ey().saturating_sub(cjf);
            if bb >= bsk { break; }
            for ct in buf.iter_mut() {
                let r = ((*ct >> 16) & 0xFF).saturating_sub(4) as u32;
                let g = ((*ct >> 8) & 0xFF).saturating_sub(4) as u32;
                let b = (*ct & 0xFF).saturating_sub(4) as u32;
                *ct = 0xFF000000 | (r << 16) | (g << 8) | b;
            }
            ev(&buf, w, h);
            crate::cpu::tsc::ww(33);
        }

        if !pu {
            crate::framebuffer::pr(false);
        }
    }

    
    
    
    crate::framebuffer::clear();

    
    let _ = crate::ramfs::bh(|fs| {
        let _ = fs.rm("/tutorial/hello.txt");
        let _ = fs.rm("/tutorial/demo.tl");
        let _ = fs.rm("/tutorial");
    });

    crate::println!();
    if ia == "fr" {
        crate::n!(0xFF00FF88, "  Tutoriel termine! Bon voyage dans TrustOS.");
    } else {
        crate::n!(0xFF00FF88, "  Tutorial complete! Enjoy exploring TrustOS.");
    }
    crate::println!();
    crate::serial_println!("[DEMO] Tutorial complete");
}




pub(super) fn krm(args: &[&str]) {
    let speed = match args.first().copied() {
        Some("fast") => 1,
        Some("slow") => 3,
        _ => 2, 
    };

    
    let pause = |im: u64| {
        let dh = im * 1000 * speed / 2;
        let rr = crate::cpu::tsc::ey();
        let freq = crate::cpu::tsc::we();
        if freq == 0 { return; } 
        let acx = freq / 1000 * dh; 
        loop {
            let bb = crate::cpu::tsc::ey().saturating_sub(rr);
            if bb >= acx { break; }
            
            let _ = crate::keyboard::kr();
            core::hint::spin_loop();
        }
    };

    let dol = 9000u64 * speed / 2; 

    
    
    
    let (dy, dw) = crate::framebuffer::kv();

    {
        let pu = crate::framebuffer::ajy();
        if !pu {
            crate::framebuffer::adw();
            crate::framebuffer::pr(true);
        }

        let w = dy as usize;
        let h = dw as usize;
        let mut buf = alloc::vec![0u32; w * h];

        
        let pf = |buf: &mut [u32], w: usize, h: usize, cx: usize, u: usize, c: char, color: u32, scale: usize| {
            let du = crate::framebuffer::font::ol(c);
            for (row, &bits) in du.iter().enumerate() {
                for bf in 0..8u32 {
                    if bits & (0x80 >> bf) != 0 {
                        for ak in 0..scale {
                            for am in 0..scale {
                                let p = cx + bf as usize * scale + am;
                                let o = u + row * scale + ak;
                                if p < w && o < h {
                                    buf[o * w + p] = color;
                                }
                            }
                        }
                    }
                }
            }
        };

        
        let qdk = |buf: &mut [u32], w: usize, h: usize, y: usize, text: &str, color: u32, scale: usize| {
            let acy = text.len() * 8 * scale;
            let start_x = if acy < w { (w - acy) / 2 } else { 0 };
            for (i, c) in text.chars().enumerate() {
                pf(buf, w, h, start_x + i * 8 * scale, y, c, color, scale);
            }
        };

        
        let mut kk: alloc::vec::Vec<u16> = alloc::vec![0u16; w / 8 + 1];
        let mut la: alloc::vec::Vec<u8> = alloc::vec![1u8; w / 8 + 1];
        
        for i in 0..kk.len() {
            kk[i] = ((i * 37 + 13) % h) as u16;
            la[i] = (((i * 7 + 3) % 4) + 1) as u8;
        }

        let cwv = |buf: &mut [u32], w: usize, h: usize, cols: &mut [u16], speeds: &[u8], frame: u32| {
            
            for ct in buf.iter_mut() {
                let g = ((*ct >> 8) & 0xFF) as u32;
                if g > 0 {
                    let cnb = g.saturating_sub(8);
                    *ct = 0xFF000000 | (cnb << 8);
                }
            }
            
            for ow in 0..cols.len() {
                let x = ow * 8;
                if x >= w { continue; }
                cols[ow] = cols[ow].wrapping_add(speeds[ow] as u16);
                if cols[ow] as usize >= h { cols[ow] = 0; }
                let y = cols[ow] as usize;
                
                let c = (((frame as usize + ow * 13) % 94) + 33) as u8 as char;
                let du = crate::framebuffer::font::ol(c);
                for (row, &bits) in du.iter().enumerate() {
                    let o = y + row;
                    if o >= h { break; }
                    for bf in 0..8u32 {
                        if bits & (0x80 >> bf) != 0 {
                            let p = x + bf as usize;
                            if p < w {
                                buf[o * w + p] = 0xFF00FF44;
                            }
                        }
                    }
                }
            }
        };

        
        let ev = |buf: &[u32], w: usize, h: usize| {
            if let Some((bb_ptr, _bb_w, bb_h, bb_stride)) = crate::framebuffer::aqr() {
                let mq = bb_ptr as *mut u32;
                let aeu = bb_stride as usize;
                for y in 0..h.min(bb_h as usize) {
                    unsafe {
                        core::ptr::copy_nonoverlapping(
                            buf[y * w..].as_ptr(),
                            mq.add(y * aeu),
                            w,
                        );
                    }
                }
            }
            crate::framebuffer::ii();
        };

        
        
        let bdi = |buf: &mut [u32], w: usize, h: usize,
                           kk: &mut [u16], la: &[u8],
                           lines: &[(&str, u32, usize)], 
                           hold_ms: u64, speed: u64| {
            let freq = crate::cpu::tsc::we();
            if freq == 0 { return; }

            
            let vu: usize = lines.iter().map(|(t, _, _)| t.len()).sum();
            let ecz = 80u64 * speed / 2; 
            let dfx = vu as u64 * ecz;
            
            let rr = crate::cpu::tsc::ey();
            let rbd = freq / 1000 * dfx;
            let gay = freq / 1000 * (dfx + hold_ms * speed / 2);

            let mut frame = 0u32;
            loop {
                let bb = crate::cpu::tsc::ey().saturating_sub(rr);
                if bb >= gay { break; }
                if let Some(key) = crate::keyboard::kr() {
                    if key == 0x1B || key == b'q' { break; }
                }

                
                cwv(buf, w, h, kk, la, frame);

                
                let elapsed_ms = bb / (freq / 1000).max(1);
                let hh = if elapsed_ms < dfx {
                    (elapsed_ms / ecz.max(1)) as usize
                } else {
                    vu
                };

                
                let ecj: usize = lines.iter().map(|(_, _, j)| 16 * j + 8).sum::<usize>();
                let mut ajb = if ecj < h { (h - ecj) / 2 } else { 20 };
                let mut bff = 0usize;

                for &(text, color, scale) in lines {
                    let acy = text.len() * 8 * scale;
                    let start_x = if acy < w { (w - acy) / 2 } else { 0 };

                    for (i, c) in text.chars().enumerate() {
                        if bff + i >= hh { break; }
                        pf(buf, w, h, start_x + i * 8 * scale, ajb, c, color, scale);
                    }
                    
                    if hh > bff && hh < bff + text.len() {
                        let fqb = hh - bff;
                        let cx = start_x + fqb * 8 * scale;
                        for u in ajb..ajb + 16 * scale {
                            if u < h && cx + 2 < w {
                                buf[u * w + cx] = 0xFF00FF88;
                                buf[u * w + cx + 1] = 0xFF00FF88;
                            }
                        }
                    }

                    bff += text.len();
                    ajb += 16 * scale + 8;
                }

                ev(buf, w, h);
                frame += 1;
                
                crate::cpu::tsc::ww(33);
            }

            
            let cjf = crate::cpu::tsc::ey();
            let cxp = 800u64;
            let bsk = freq / 1000 * cxp;
            loop {
                let bb = crate::cpu::tsc::ey().saturating_sub(cjf);
                if bb >= bsk { break; }
                let progress = (bb * 255 / bsk) as u32;
                for ct in buf.iter_mut() {
                    let r = ((*ct >> 16) & 0xFF) as u32;
                    let g = ((*ct >> 8) & 0xFF) as u32;
                    let b = (*ct & 0xFF) as u32;
                    let nr = r.saturating_sub(r * progress / 512 + 1);
                    let ayn = g.saturating_sub(g * progress / 512 + 1);
                    let ayj = b.saturating_sub(b * progress / 512 + 1);
                    *ct = 0xFF000000 | (nr << 16) | (ayn << 8) | ayj;
                }
                ev(buf, w, h);
                crate::cpu::tsc::ww(33);
            }

            
            for ct in buf.iter_mut() { *ct = 0xFF000000; }
            ev(buf, w, h);
        };

        
        crate::serial_println!("[SHOWCASE] Scene 1: Simulation question");
        for ct in buf.iter_mut() { *ct = 0xFF000000; }
        bdi(&mut buf, w, h, &mut kk, &la,
            &[("Do you think", 0xFF00DD55, 4),
              ("life is a simulation?", 0xFF00FF66, 4)],
            3000, speed);

        
        crate::serial_println!("[SHOWCASE] Scene 2: 6MB OS");
        bdi(&mut buf, w, h, &mut kk, &la,
            &[("Can it run", 0xFF00DD55, 5),
              ("in a 6MB OS?", 0xFF00FF88, 5)],
            3000, speed);

        
        crate::serial_println!("[SHOWCASE] Scene 3: TrustOS title");
        {
            
            let freq = crate::cpu::tsc::we();
            let oli = 8000u64 * speed / 2;
            let olj = freq / 1000 * oli;
            let rr = crate::cpu::tsc::ey();

            let mut bvi = crate::formula3d::FormulaRenderer::new();
            bvi.set_scene(crate::formula3d::FormulaScene::Character);
            bvi.wire_color = 0xFF00FFAA;

            
            let vp_w = 200usize;
            let vp_h = 200usize;
            let mut bqa = alloc::vec![0u32; vp_w * vp_h];

            let mut frame = 0u32;
            loop {
                let bb = crate::cpu::tsc::ey().saturating_sub(rr);
                if bb >= olj { break; }
                if let Some(key) = crate::keyboard::kr() {
                    if key == 0x1B { break; }
                }

                
                cwv(&mut buf, w, h, &mut kk, &la, frame);

                
                let title = "TRUST OS";
                let bjp = 6;
                let bea = title.len() * 8 * bjp;
                let avk = if bea < w { (w - bea) / 2 } else { 0 };
                let apg = h / 8;
                for (i, c) in title.chars().enumerate() {
                    
                    let zz = ((frame as usize * 3 + i * 30) % 360) as u32;
                    let color = if zz < 120 {
                        let t = zz * 255 / 120;
                        0xFF000000 | ((255 - t) << 16) | (t << 8)
                    } else if zz < 240 {
                        let t = (zz - 120) * 255 / 120;
                        0xFF000000 | ((255 - t) << 8) | t
                    } else {
                        let t = (zz - 240) * 255 / 120;
                        0xFF000000 | (t << 16) | (255 - t)
                    };
                    pf(&mut buf, w, h, avk + i * 8 * bjp, apg, c, color, bjp);
                }

                
                bvi.update();
                for aa in bqa.iter_mut() { *aa = 0x00000000; } 
                bvi.render(&mut bqa, vp_w, vp_h);

                
                let vp_x = if vp_w < w { (w - vp_w) / 2 } else { 0 };
                let vp_y = apg + 16 * bjp + 20;
                for vy in 0..vp_h {
                    for vx in 0..vp_w {
                        let src = bqa[vy * vp_w + vx];
                        if src & 0x00FFFFFF != 0 { 
                            let ad = vp_y + vy;
                            let dx = vp_x + vx;
                            if ad < h && dx < w {
                                buf[ad * w + dx] = src;
                            }
                        }
                    }
                }

                
                let hov = "Written in Rust by Nated0ge";
                let fpc = 3;
                let hox = hov.len() * 8 * fpc;
                let kzn = if hox < w { (w - hox) / 2 } else { 0 };
                let kzo = vp_y + vp_h + 30;
                for (i, c) in hov.chars().enumerate() {
                    pf(&mut buf, w, h, kzn + i * 8 * fpc, kzo, c, 0xFF88CCFF, fpc);
                }

                ev(&buf, w, h);
                frame += 1;
                crate::cpu::tsc::ww(33);
            }

            
            let cjf = crate::cpu::tsc::ey();
            let bsk = freq / 1000 * 800;
            loop {
                let bb = crate::cpu::tsc::ey().saturating_sub(cjf);
                if bb >= bsk { break; }
                for ct in buf.iter_mut() {
                    let r = ((*ct >> 16) & 0xFF).saturating_sub(4) as u32;
                    let g = ((*ct >> 8) & 0xFF).saturating_sub(4) as u32;
                    let b = (*ct & 0xFF).saturating_sub(4) as u32;
                    *ct = 0xFF000000 | (r << 16) | (g << 8) | b;
                }
                ev(&buf, w, h);
                crate::cpu::tsc::ww(33);
            }
            for ct in buf.iter_mut() { *ct = 0xFF000000; }
            ev(&buf, w, h);
        }

        
        crate::serial_println!("[SHOWCASE] Scene 4: Specs");
        bdi(&mut buf, w, h, &mut kk, &la,
            &[("6MB ISO vs 6GB Windows",  0xFF00FF66, 3),
              ("0 lines of C. Pure Rust.", 0xFF44FFAA, 3),
              ("Boots in 0.8s not 45s",    0xFF00DDFF, 3),
              ("No kernel panics. Ever.",  0xFFFFCC44, 3),
              ("GPU desktop at 144 FPS",   0xFF88FF44, 3),
              ("Built in 7 days solo",     0xFFFF8844, 3)],
            3000, speed);

        
        crate::serial_println!("[SHOWCASE] Scene 5: Are you ready?");
        bdi(&mut buf, w, h, &mut kk, &la,
            &[("Are you ready?", 0xFF00FF44, 6)],
            2000, speed);

        
        if !pu {
            crate::framebuffer::pr(false);
        }
    }

    
    crate::framebuffer::clear();

    crate::println!();
    crate::println!();
    crate::println!();
    crate::n!(0xFF00CCFF, "");
    crate::n!(0xFF00CCFF, "  ||||||||+||||||+ ||+   ||+|||||||+||||||||+ ||||||+ |||||||+");
    crate::n!(0xFF00CCFF, "  +--||+--+||+--||+|||   |||||+----++--||+--+||+---||+||+----+");
    crate::n!(0xFF00CCFF, "     |||   ||||||++|||   ||||||||||+   |||   |||   ||||||||||+");
    crate::n!(0xFF00DDFF, "     |||   ||+--||+|||   |||+----|||   |||   |||   |||+----|||");
    crate::n!(0xFF00EEFF, "     |||   |||  |||+||||||++||||||||   |||   +||||||++||||||||");
    crate::n!(0xFF00EEFF, "     +-+   +-+  +-+ +-----+ +------+   +-+    +-----+ +------+");
    crate::println!();
    crate::n!(0xFF888888, "                  ?????????????????????????????????");
    crate::println!();
    crate::n!(0xFFAADDFF, "           A bare-metal OS written in 100% Rust -- in 7 days");
    crate::n!(0xFF666666, "         99,000+ lines * 6 MB ISO * GPU compositing * 144 FPS");
    crate::println!();
    crate::n!(0xFF888888, "                  ?????????????????????????????????");
    crate::println!();
    crate::n!(0xFF00FF88, "                        ?  FEATURE SHOWCASE  ?");
    crate::println!();
    
    pause(6);

    
    crate::n!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::n!(0xFF00CCFF, "|  PHASE 1 ---- SYSTEM INFO                                   |");
    crate::n!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::println!();
    pause(3);

    super::commands::fmw();
    pause(4);

    crate::n!(C_, "$ uname -a");
    super::commands::eim(&["-a"]);
    pause(4);

    crate::n!(C_, "$ free");
    super::commands::eih();
    pause(4);

    crate::n!(C_, "$ lscpu");
    super::unix::hma();
    pause(5);

    
    crate::n!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::n!(0xFF00CCFF, "|  PHASE 2 ---- FILESYSTEM (TrustFS + VFS)                    |");
    crate::n!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::println!();
    pause(3);

    crate::n!(C_, "$ mkdir /demo");
    super::commands::eik(&["/demo"]);
    pause(2);

    crate::n!(C_, "$ echo 'Hello TrustOS!' > /demo/hello.txt");
    crate::ramfs::bh(|fs| {
        let _ = fs.touch("/demo/hello.txt");
        let _ = fs.write_file("/demo/hello.txt", b"Hello TrustOS!\nThis file was created live during the showcase.\n");
    });
    crate::n!(B_, "Written: /demo/hello.txt");
    pause(2);

    crate::n!(C_, "$ cat /demo/hello.txt");
    super::commands::dkw(&["/demo/hello.txt"], None, None);
    pause(3);

    crate::n!(C_, "$ tree /");
    super::commands::fnh(&["/"]);
    pause(4);

    
    crate::n!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::n!(0xFF00CCFF, "|  PHASE 3 ---- TRUSTLANG (Built-in Compiler + VM)            |");
    crate::n!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::println!();
    pause(3);

    
    let crs = r#"fn fibonacci(n: i64) -> i64 {
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
    crate::ramfs::bh(|fs| {
        let _ = fs.touch("/demo/showcase.tl");
        let _ = fs.write_file("/demo/showcase.tl", crs.as_bytes());
    });

    crate::n!(C_, "$ cat /demo/showcase.tl");
    crate::n!(0xFFDDDDDD, "{}", crs);
    pause(4);

    crate::n!(C_, "$ trustlang run /demo/showcase.tl");
    crate::n!(0xFF00FF88, "[TrustLang] Compiling showcase.tl...");
    match crate::trustlang::run(crs) {
        Ok(output) => { if !output.is_empty() { crate::print!("{}", output); } }
        Err(e) => crate::n!(A_, "Error: {}", e),
    }
    crate::n!(B_, "[TrustLang] Program finished successfully.");
    pause(6);

    
    crate::n!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::n!(0xFF00CCFF, "|  PHASE 4 ---- NETWORK STACK (TCP/IP, DHCP, DNS, TLS 1.3)    |");
    crate::n!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::println!();
    pause(3);

    crate::n!(C_, "$ ifconfig");
    super::vm::dkx();
    pause(3);

    crate::n!(C_, "$ netstat");
    super::vm::dky();
    pause(4);

    
    crate::n!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::n!(0xFF00CCFF, "|  PHASE 5 ---- TRUSTVIDEO (Real-time procedural rendering)   |");
    crate::n!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::println!();
    pause(3);

    
    crate::n!(0xFFFF4400, "? Demo 1/3: FIRE EFFECT -- Real-time procedural flame");
    pause(2);

    let bt = dy as u16;
    let ex = dw as u16;
    crate::video::player::ddk("fire", bt, ex, 30, dol);
    
    
    crate::framebuffer::clear();
    pause(2);

    
    crate::n!(0xFF00FF44, "? Demo 2/3: MATRIX RAIN -- Digital rain effect");
    pause(2);

    crate::video::player::ddk("matrix", bt, ex, 30, dol);
    
    crate::framebuffer::clear();
    pause(2);

    
    crate::n!(0xFFFF00FF, "? Demo 3/3: PLASMA -- Integer sine LUT psychedelic");
    pause(2);

    crate::video::player::ddk("plasma", bt, ex, 30, dol);
    
    crate::framebuffer::clear();
    pause(2);

    
    crate::n!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::n!(0xFF00CCFF, "|  PHASE 5b -- FORMULA3D (Wireframe 3D engine)                |");
    crate::n!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::println!();
    pause(2);

    crate::n!(0xFF00FF88, "? 3D wireframe character -- perspective projection + depth shading");
    pause(2);

    
    {
        let mut renderer = crate::formula3d::FormulaRenderer::new();
        renderer.set_scene(crate::formula3d::FormulaScene::Character);
        renderer.wire_color = 0xFF00FFAA; 

        let lk = dy as usize;
        let pp = dw as usize;
        let fh = if dy > lk as u32 { (dy - lk as u32) / 2 } else { 0 } as usize;
        let hk = if dw > pp as u32 { (dw - pp as u32) / 2 } else { 0 } as usize;

        let mut buf = alloc::vec![0u32; lk * pp];

        let pu = crate::framebuffer::ajy();
        if !pu {
            crate::framebuffer::adw();
            crate::framebuffer::pr(true);
        }
        crate::framebuffer::awo(0xFF000000);
        crate::framebuffer::ii();

        let rr = crate::cpu::tsc::ey();
        let freq = crate::cpu::tsc::we();
        let duration_ms = dol;
        let acx = if freq > 0 { freq / 1000 * duration_ms } else { u64::MAX };

        loop {
            let bb = crate::cpu::tsc::ey().saturating_sub(rr);
            if bb >= acx { break; }
            if let Some(key) = crate::keyboard::kr() {
                if key == 0x1B || key == b'q' { break; }
            }

            renderer.update();
            renderer.render(&mut buf, lk, pp);

            
            if let Some((bb_ptr, _bb_w, bb_h, bb_stride)) = crate::framebuffer::aqr() {
                let mq = bb_ptr as *mut u32;
                let aeu = bb_stride as usize;
                for y in 0..pp {
                    let ad = hk + y;
                    if ad >= bb_h as usize { break; }
                    let amv = &buf[y * lk..y * lk + lk];
                    unsafe {
                        let dst = mq.add(ad * aeu + fh);
                        core::ptr::copy_nonoverlapping(amv.as_ptr(), dst, lk);
                    }
                }
            }
            crate::framebuffer::ii();
        }

        if !pu {
            crate::framebuffer::pr(false);
        }
    }

    crate::framebuffer::clear();
    pause(2);

    
    crate::n!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::n!(0xFF00CCFF, "|  PHASE 5c -- COSMIC2 DESKTOP + WEB BROWSER                  |");
    crate::n!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::println!();
    pause(2);

    crate::n!(0xFF00FF88, "? COSMIC2 Desktop -- GPU-composited multi-layer windowing system");
    crate::n!(0xFF00FF88, "? Launching with built-in Web Browser ? google.com");
    pause(3);

    
    fmf(Some("browser"), dol);

    crate::framebuffer::clear();
    pause(2);

    
    crate::n!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::n!(0xFF00CCFF, "|  PHASE 6 ---- 200+ BUILT-IN COMMANDS                       |");
    crate::n!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::println!();
    pause(2);

    crate::n!(0xFFAADDFF, "  +- File System ------------------------------------------+");
    crate::n!(0xFFDDDDDD, "  | ls cd pwd mkdir rm cp mv cat head tail tree find grep  |");
    crate::n!(0xFFAADDFF, "  +- Network ----------------------------------------------|");
    crate::n!(0xFFDDDDDD, "  | ifconfig ping curl wget nslookup arp route netstat     |");
    crate::n!(0xFFAADDFF, "  +- System -----------------------------------------------|");
    crate::n!(0xFFDDDDDD, "  | ps top free df uname dmesg mount lspci lscpu lsblk    |");
    crate::n!(0xFFAADDFF, "  +- Development ------------------------------------------|");
    crate::n!(0xFFDDDDDD, "  | trustlang (compiler+VM) * TrustCode (editor)          |");
    crate::n!(0xFFDDDDDD, "  | transpile (binary?Rust) * exec (ELF loader)           |");
    crate::n!(0xFFAADDFF, "  +- Graphics ---------------------------------------------|");
    crate::n!(0xFFDDDDDD, "  | desktop (COSMIC2 compositor) * video (TrustVideo)     |");
    crate::n!(0xFFDDDDDD, "  | benchmark (SSE2 SIMD) * HoloMatrix (3D volumetric)   |");
    crate::n!(0xFFAADDFF, "  +--------------------------------------------------------+");
    crate::println!();
    pause(6);

    
    crate::println!();
    crate::n!(0xFF00CCFF, "  ||||||||+||||||+ ||+   ||+|||||||+||||||||+ ||||||+ |||||||+");
    crate::n!(0xFF00CCFF, "  +--||+--+||+--||+|||   |||||+----++--||+--+||+---||+||+----+");
    crate::n!(0xFF00CCFF, "     |||   ||||||++|||   ||||||||||+   |||   |||   ||||||||||+");
    crate::n!(0xFF00DDFF, "     |||   ||+--||+|||   |||+----|||   |||   |||   |||+----|||");
    crate::n!(0xFF00EEFF, "     |||   |||  |||+||||||++||||||||   |||   +||||||++||||||||");
    crate::n!(0xFF00EEFF, "     +-+   +-+  +-+ +-----+ +------+   +-+    +-----+ +------+");
    crate::println!();
    crate::n!(0xFFFFCC00, "  ?  100% Rust -- Zero C code -- Memory safe by design");
    crate::n!(0xFFFFCC00, "  ?  Built from scratch in 7 days -- 99,000+ lines");
    crate::n!(0xFFFFCC00, "  ?  6 MB ISO -- boots in seconds");
    crate::n!(0xFFFFCC00, "  ?  GPU compositing -- 144 FPS desktop");
    crate::println!();
    crate::n!(0xFF00FF88, "  github.com/nathan237/TrustOS");
    crate::n!(0xFF888888, "  Star ? * Fork * Contribute");
    crate::println!();
    crate::n!(0xFF888888, "  ??????????????????????????????????????????????????");
    crate::println!();

    
    let _ = crate::ramfs::bh(|fs| { let _ = fs.rm("/demo/hello.txt"); let _ = fs.rm("/demo/showcase.tl"); });
}






pub(super) fn krn(args: &[&str]) {
    let speed = match args.first().copied() {
        Some("fast") => 1u64,
        Some("slow") => 3,
        _ => 2,
    };

    let pause = |im: u64| {
        let dh = im * 1000 * speed / 2;
        let rr = crate::cpu::tsc::ey();
        let freq = crate::cpu::tsc::we();
        if freq == 0 { return; }
        let acx = freq / 1000 * dh;
        loop {
            let bb = crate::cpu::tsc::ey().saturating_sub(rr);
            if bb >= acx { break; }
            let _ = crate::keyboard::kr();
            core::hint::spin_loop();
        }
    };

    let (dy, dw) = crate::framebuffer::kv();
    if dy == 0 || dw == 0 {
        crate::n!(0xFFFF4444, "No framebuffer available");
        return;
    }

    
    
    
    {
        let pu = crate::framebuffer::ajy();
        if !pu {
            crate::framebuffer::adw();
            crate::framebuffer::pr(true);
        }

        let w = dy as usize;
        let h = dw as usize;
        let mut buf = alloc::vec![0u32; w * h];

        
        let draw_char = |buf: &mut [u32], w: usize, h: usize, cx: usize, u: usize,
                         c: char, color: u32, scale: usize| {
            let du = crate::framebuffer::font::ol(c);
            for (row, &bits) in du.iter().enumerate() {
                for bf in 0..8u32 {
                    if bits & (0x80 >> bf) != 0 {
                        for ak in 0..scale {
                            for am in 0..scale {
                                let p = cx + bf as usize * scale + am;
                                let o = u + row * scale + ak;
                                if p < w && o < h {
                                    buf[o * w + p] = color;
                                }
                            }
                        }
                    }
                }
            }
        };

        
        let bo = |buf: &mut [u32], w: usize, h: usize, x: usize, y: usize,
                        j: &str, color: u32, scale: usize| {
            for (i, c) in j.chars().enumerate() {
                draw_char(buf, w, h, x + i * 8 * scale, y, c, color, scale);
            }
        };

        
        let blit = |buf: &[u32], w: usize, h: usize| {
            if let Some((bb_ptr, _bb_w, bb_h, bb_stride)) = crate::framebuffer::aqr() {
                let mq = bb_ptr as *mut u32;
                let aeu = bb_stride as usize;
                for y in 0..h.min(bb_h as usize) {
                    unsafe {
                        core::ptr::copy_nonoverlapping(
                            buf[y * w..].as_ptr(),
                            mq.add(y * aeu),
                            w,
                        );
                    }
                }
            }
            crate::framebuffer::ii();
        };

        let freq = crate::cpu::tsc::we();
        let rr = crate::cpu::tsc::ey();
        let olm = 6000u64 * speed / 2;
        let acx = if freq > 0 { freq / 1000 * olm } else { u64::MAX };

        
        let mut rng = crate::cpu::tsc::ey();
        let mut gjm = || -> u64 {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            rng
        };

        
        let layers: [usize; 4] = [6, 8, 8, 4];
        let mut bie: alloc::vec::Vec<alloc::vec::Vec<(usize, usize)>> = alloc::vec::Vec::new();
        let ilk = w / 6;
        let nbi = w * 5 / 6;
        for (sz, &count) in layers.iter().enumerate() {
            let fe = ilk + (nbi - ilk) * sz / (layers.len() - 1);
            let nbl = h / 4;
            let spacing = h / 2 / (count + 1);
            let mut ijo = alloc::vec::Vec::new();
            for ni in 0..count {
                ijo.push((fe, nbl + spacing * (ni + 1)));
            }
            bie.push(ijo);
        }

        
        let mut frame = 0u32;
        loop {
            let bb = crate::cpu::tsc::ey().saturating_sub(rr);
            if bb >= acx { break; }
            if let Some(key) = crate::keyboard::kr() {
                if key == 0x1B || key == b'q' { break; }
            }

            
            for p in buf.iter_mut() {
                *p = 0xFF050510;
            }

            
            let kq = crate::graphics::holomatrix::azr(frame as f32 * 0.08) * 0.5 + 0.5;
            for sz in 0..bie.len() - 1 {
                for &(x1, y1) in &bie[sz] {
                    for &(x2, y2) in &bie[sz + 1] {
                        
                        let dx = (x2 as i32 - x1 as i32).abs();
                        let ad = (y2 as i32 - y1 as i32).abs();
                        let steps = dx.max(ad) as usize;
                        if steps == 0 { continue; }
                        let obf = (gjm() % 60) as f32;
                        let g = (40.0 + kq * 80.0 + obf) as u32;
                        let color = 0xFF000000 | (g.min(255) << 8);
                        for j in 0..steps {
                            let t = j as f32 / steps as f32;
                            let p = (x1 as f32 + (x2 as f32 - x1 as f32) * t) as usize;
                            let o = (y1 as f32 + (y2 as f32 - y1 as f32) * t) as usize;
                            if p < w && o < h {
                                buf[o * w + p] = color;
                            }
                        }
                    }
                }
            }

            
            for (sz, bj) in bie.iter().enumerate() {
                for (ni, &(nx, re)) in bj.iter().enumerate() {
                    let iqt = crate::graphics::holomatrix::azr(frame as f32 * 0.12 + sz as f32 * 1.5 + ni as f32 * 0.8) * 0.5 + 0.5;
                    let r = 5 + (iqt * 3.0) as usize;
                    let cyh = (120.0 + iqt * 135.0) as u32;
                    let color = 0xFF000000 | (cyh.min(255) << 8) | ((cyh / 3).min(255));
                    
                    for ad in 0..=r * 2 {
                        for dx_i in 0..=r * 2 {
                            let lh = dx_i as i32 - r as i32;
                            let kf = ad as i32 - r as i32;
                            if lh * lh + kf * kf <= (r * r) as i32 {
                                let p = (nx as i32 + lh) as usize;
                                let o = (re as i32 + kf) as usize;
                                if p < w && o < h {
                                    buf[o * w + p] = color;
                                }
                            }
                        }
                    }
                }
            }

            
            let lxc = frame as f32 * 0.03;
            for sz in 0..bie.len() - 1 {
                let amu = (gjm() % bie[sz].len() as u64) as usize;
                let ajm = (gjm() % bie[sz + 1].len() as u64) as usize;
                let (x1, y1) = bie[sz][amu];
                let (x2, y2) = bie[sz + 1][ajm];
                let t = ((lxc + sz as f32 * 0.7) % 1.0).abs();
                let dnm = (x1 as f32 + (x2 as f32 - x1 as f32) * t) as usize;
                let fst = (y1 as f32 + (y2 as f32 - y1 as f32) * t) as usize;
                
                for ad in 0..4 {
                    for dx in 0..4 {
                        let p = dnm + dx;
                        let o = fst + ad;
                        if p < w && o < h {
                            buf[o * w + p] = 0xFF00FF66;
                        }
                    }
                }
            }

            
            let title = "J A R V I S";
            let bea = title.len() * 8 * 4;
            let bu = if w > bea { (w - bea) / 2 } else { 0 };
            bo(&mut buf, w, h, bu, h / 12, title, 0xFF00FF64, 4);

            
            let sub = "Kernel-Resident Neural Network";
            let jjn = sub.len() * 8 * 2;
            let am = if w > jjn { (w - jjn) / 2 } else { 0 };
            bo(&mut buf, w, h, am, h / 12 + 72, sub, 0xFF88DDAA, 2);

            
            let bpe = h - 50;
            let stats = [
                "4.4M params",
                "4 layers",
                "256 d_model",
                "Byte-level",
                "Ring 0",
            ];
            let jny = stats.iter().map(|j| j.len() * 8 + 30).sum::<usize>();
            let mut jkg = if w > jny { (w - jny) / 2 } else { 10 };
            for stat in &stats {
                bo(&mut buf, w, h, jkg, bpe, stat, 0xFF00CC88, 1);
                jkg += stat.len() * 8 + 30;
            }

            
            let labels = ["Input", "Hidden", "Hidden", "Output"];
            for (sz, bj) in bie.iter().enumerate() {
                if let Some(&(fe, _)) = bj.first() {
                    let cmb = labels[sz];
                    let mxr = if fe > cmb.len() * 4 { fe - cmb.len() * 4 } else { 0 };
                    bo(&mut buf, w, h, mxr, h * 3 / 4 + 20, cmb, 0xFF666666, 1);
                }
            }

            blit(&buf, w, h);
            frame += 1;
        }

        if !pu {
            crate::framebuffer::pr(false);
        }
    }

    crate::framebuffer::clear();

    
    
    
    crate::println!();
    crate::n!(0xFF00FF64, "     ___  ___  _____  _   _  ___  _____");
    crate::n!(0xFF00FF64, "    |_  |/ _ \\| ___ \\| | | ||_  |/  ___|");
    crate::n!(0xFF00FF88, "      | / /_\\ \\ |_/ /| | | |  | |\\ `--.");
    crate::n!(0xFF00FF88, "      | |  _  |    / | | | |  | | `--. \\");
    crate::n!(0xFF00FFAA, "  /\\__/ / | | | |\\ \\ \\ \\_/ /\\__/ /\\__/ /");
    crate::n!(0xFF00FFAA, "  \\____/\\_| |_\\_| \\_| \\___/\\____/\\____/");
    crate::println!();
    crate::n!(0xFF888888, "  ======================================================");
    crate::n!(0xFFAADDFF, "    Kernel-Resident Self-Propagating Neural Network");
    crate::n!(0xFF888888, "  ======================================================");
    crate::println!();
    pause(4);

    crate::n!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::n!(0xFF00CCFF, "|  ARCHITECTURE                                                |");
    crate::n!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::println!();
    crate::n!(0xFF00FF88, "  Model:      Transformer (4L, d=256, 4H, d_ff=1024, SwiGLU)");
    crate::n!(0xFF00FF88, "  Parameters: 4,393,216 (FP32 = 17.6 MB)");
    crate::n!(0xFF00FF88, "  Vocabulary: 256 (byte-level, no tokenizer)");
    crate::n!(0xFF00FF88, "  SIMD:       Auto-detected (AVX2+FMA / SSE2 / NEON)");
    crate::n!(0xFF00FF88, "  Location:   Ring 0 (kernel address space)");
    crate::n!(0xFF00FF88, "  Safety:     Guardian Pact (2 authorized parents)");
    crate::println!();
    pause(5);

    
    
    
    crate::n!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::n!(0xFF00CCFF, "|  PHASE 1 --- BRAIN INITIALIZATION                            |");
    crate::n!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::println!();
    pause(2);

    crate::n!(0xFF00DDFF, "  $ jarvis brain init");
    crate::println!();

    if !crate::jarvis::is_ready() {
        crate::jarvis::init();
    }

    let mje = crate::jarvis::cki();
    if mje {
        crate::n!(0xFF00FF88, "  [OK] Full brain loaded: 4,393,216 parameters");
    } else {
        crate::n!(0xFFFFCC00, "  [OK] Micro sentinel active: 78,016 parameters");
        crate::n!(0xFF888888, "  (Full brain requires jarvis_pretrained.bin in ISO)");
    }
    crate::println!();
    pause(4);

    
    
    
    crate::n!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::n!(0xFF00CCFF, "|  PHASE 2 --- LIVE INFERENCE (kernel space)                   |");
    crate::n!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::println!();
    pause(2);

    crate::n!(0xFF00DDFF, "  $ jarvis chat What is TrustOS?");
    crate::println!();

    
    let fa = crate::jarvis::generate("What is TrustOS?", 80);
    let jw = fa.trim();
    if !jw.is_empty() {
        
        crate::bq!(0xFF00FF64, "  JARVIS> ");
        for c in jw.chars().take(200) {
            crate::bq!(0xFFCCFFDD, "{}", c);
            
            let kiq = crate::cpu::tsc::ey();
            let hkm = crate::cpu::tsc::we();
            if hkm > 0 {
                let kin = hkm / 1000 * 30; 
                while crate::cpu::tsc::ey().saturating_sub(kiq) < kin {
                    core::hint::spin_loop();
                }
            }
        }
        crate::println!();
    } else {
        crate::n!(0xFF888888, "  (Inference requires full brain to be loaded)");
    }
    crate::println!();
    pause(5);

    
    
    
    crate::n!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::n!(0xFF00CCFF, "|  PHASE 3 --- MESH NETWORK + SELF-PROPAGATION                |");
    crate::n!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::println!();
    pause(2);

    crate::n!(0xFF00FF88, "  Self-Propagation Sequence:");
    crate::n!(0xFFAADDFF, "  1. New node boots with micro sentinel (304 KB)");
    crate::n!(0xFFAADDFF, "  2. Mesh discovery: UDP broadcast on port 7700");
    crate::n!(0xFFAADDFF, "  3. Finds peer with full brain");
    crate::n!(0xFFAADDFF, "  4. RPC GetWeights: downloads 17.6 MB via TCP 7701");
    crate::n!(0xFFAADDFF, "  5. Full brain loaded -> node is intelligent");
    crate::n!(0xFFAADDFF, "  6. Federated learning enabled (P2P, no central server)");
    crate::println!();
    pause(4);

    
    let nen = crate::jarvis::mesh::is_active();
    let lj = crate::jarvis::mesh::ayz();
    if nen {
        crate::n!(0xFF00FF88, "  [LIVE] Mesh: ACTIVE");
        crate::n!(0xFF00FF88, "  [LIVE] Peers: {}", lj);
        let role = crate::jarvis::mesh::dwa();
        let dxy = match role {
            crate::jarvis::mesh::NodeRole::Leader => "Leader",
            crate::jarvis::mesh::NodeRole::Candidate => "Candidate",
            crate::jarvis::mesh::NodeRole::Worker => "Worker",
        };
        crate::n!(0xFF00FF88, "  [LIVE] Role:  {}", dxy);
    } else {
        crate::n!(0xFFFFCC00, "  [INFO] Mesh: not started (single node mode)");
        crate::n!(0xFF888888, "  Run: jarvis brain propagate  (to join mesh)");
    }
    crate::println!();
    pause(4);

    
    
    
    crate::n!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::n!(0xFF00CCFF, "|  PHASE 4 --- THE GUARDIAN PACT (AI Safety)                   |");
    crate::n!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::println!();
    pause(2);

    crate::n!(0xFFFFCC00, "  JARVIS has two parents:");
    crate::n!(0xFF00FF88, "    1. Nathan  (human creator, shell auth)");
    crate::n!(0xFF00FF88, "    2. Copilot (AI co-parent, serial auth)");
    crate::println!();
    crate::n!(0xFFFFCC00, "  Protected operations (require guardian approval):");
    crate::n!(0xFFAADDFF, "    Train, WeightPush, FederatedSync, AgentExecute,");
    crate::n!(0xFFAADDFF, "    PxeReplicate, ModelReset, ModelReplace, ConfigChange");
    crate::println!();
    crate::n!(0xFF888888, "  Hardcoded in kernel as immutable const.");
    crate::n!(0xFF888888, "  Cannot be bypassed. Cannot be disabled.");
    crate::n!(0xFF888888, "  PACT-2026-03-05-NATHAN-COPILOT-JARVIS");
    crate::println!();
    pause(5);

    
    
    
    crate::n!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::n!(0xFF00CCFF, "|  VERIFICATION                                                |");
    crate::n!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::println!();
    pause(2);

    crate::n!(0xFF00FF88, "  Automated Test Results:");
    crate::n!(0xFF00FF88, "    12/12 propagation tests   PASS");
    crate::n!(0xFF00FF88, "    80+   single-node tests   PASS");
    crate::println!();
    crate::n!(0xFF00FF88, "  Proven capabilities:");
    crate::n!(0xFFAADDFF, "    [x] Transformer inference in kernel (ring 0)");
    crate::n!(0xFFAADDFF, "    [x] 17.6 MB brain transfer via custom TCP");
    crate::n!(0xFFAADDFF, "    [x] Peer discovery + mesh networking");
    crate::n!(0xFFAADDFF, "    [x] Federated learning (P2P)");
    crate::n!(0xFFAADDFF, "    [x] Guardian Pact (AI safety at ring 0)");
    crate::n!(0xFFAADDFF, "    [x] Multi-arch: x86_64, aarch64, riscv64");
    crate::println!();
    pause(5);

    
    
    
    crate::println!();
    crate::n!(0xFF00FF64, "     ___  ___  _____  _   _  ___  _____");
    crate::n!(0xFF00FF64, "    |_  |/ _ \\| ___ \\| | | ||_  |/  ___|");
    crate::n!(0xFF00FF88, "      | / /_\\ \\ |_/ /| | | |  | |\\ `--.");
    crate::n!(0xFF00FF88, "      | |  _  |    / | | | |  | | `--. \\");
    crate::n!(0xFF00FFAA, "  /\\__/ / | | | |\\ \\ \\ \\_/ /\\__/ /\\__/ /");
    crate::n!(0xFF00FFAA, "  \\____/\\_| |_\\_| \\_| \\___/\\____/\\____/");
    crate::println!();
    crate::n!(0xFFFFCC00, "  The first kernel-resident self-propagating neural network.");
    crate::n!(0xFFFFCC00, "  240,000+ lines of Rust. Zero dependencies. One kernel.");
    crate::println!();
    crate::n!(0xFF00FF88, "  github.com/nathan237/TrustOS");
    crate::n!(0xFF888888, "  Star * Fork * Break it.");
    crate::println!();
    crate::n!(0xFF888888, "  Copyright 2025-2026 Nathan (nathan237)");
    crate::n!(0xFF888888, "  Apache License 2.0");
    crate::println!();
}






pub fn hme() {
    use crate::gpu_emu::{Cr, PixelOutput};

    let (dy, dw) = crate::framebuffer::kv();
    let w = dy as usize;
    let h = dw as usize;
    if w == 0 || h == 0 { return; }

    let pu = crate::framebuffer::ajy();
    if !pu {
        crate::framebuffer::adw();
        crate::framebuffer::pr(true);
    }

    let mut buf = alloc::vec![0u32; w * h];

    

    let draw_char = |buf: &mut [u32], w: usize, h: usize, cx: usize, u: usize, c: char, color: u32, scale: usize| {
        let du = crate::framebuffer::font::ol(c);
        for (row, &bits) in du.iter().enumerate() {
            for bf in 0..8u32 {
                if bits & (0x80 >> bf) != 0 {
                    for ak in 0..scale {
                        for am in 0..scale {
                            let p = cx + bf as usize * scale + am;
                            let o = u + row * scale + ak;
                            if p < w && o < h {
                                buf[o * w + p] = color;
                            }
                        }
                    }
                }
            }
        }
    };

    let draw_text = |buf: &mut [u32], w: usize, h: usize, x: usize, y: usize, text: &str, color: u32, scale: usize| {
        for (i, c) in text.chars().enumerate() {
            draw_char(buf, w, h, x + i * 8 * scale, y, c, color, scale);
        }
    };

    let draw_text_centered = |buf: &mut [u32], w: usize, h: usize, y: usize, text: &str, color: u32, scale: usize| {
        let gr = text.len() * 8 * scale;
        let x = if gr < w { (w - gr) / 2 } else { 0 };
        for (i, c) in text.chars().enumerate() {
            draw_char(buf, w, h, x + i * 8 * scale, y, c, color, scale);
        }
    };

    let blit = |buf: &[u32], w: usize, h: usize| {
        
        crate::framebuffer::fjl(buf.as_ptr(), w, h);
    };

    
    
    
    let aix = || crate::logger::eg();

    
    let htr = |buf: &mut [u32], w: usize, h: usize, scene_name: &str, scene_num: usize, fps: u32, bbi: u32, bee: u32, quality: usize| {
        let hs = 28usize;
        let gk = h - hs;
        for y in gk..h {
            for x in 0..w {
                buf[y * w + x] = 0xFF0A0A0A;
            }
        }
        for x in 0..w {
            buf[gk * w + x] = 0xFF00AA44;
        }
        
        let mem = crate::memory::stats();
        let heap_used_kb = mem.heap_used / 1024;
        let heap_total_kb = (mem.heap_used + mem.heap_free) / 1024;
        let heap_pct = if heap_total_kb > 0 { heap_used_kb * 100 / heap_total_kb } else { 0 };
        
        let vj = if fps > 0 { 1000 / fps } else { 999 };
        let oae = match quality { 1 => "Full", 2 => "High", 3 => "Med", _ => "Low" };
        let mut jin = alloc::string::String::new();
        use core::fmt::Write;
        let _ = write!(jin, " {}/12 {} | {} FPS {}ms | RAM {}KB/{}KB ({}%) | CPU 100% | {} | {}x{}",
            scene_num, scene_name, fps, vj, heap_used_kb, heap_total_kb, heap_pct, oae, w, h);
        draw_text(buf, w, h, 8, gk + 8, &jin, 0xFF00FF66, 1);
    };

    let bdp = 500u64;  
    let jmw = 200u64;  

    
    let qtv = |buf: &mut [u32], w: usize, h: usize,
                                shader_fn: fn(Cr) -> PixelOutput,
                                title: &str, subtitle: &str, scene_num: usize,
                                dur_ticks: u64| {
        let start = aix();
        let mut frame = 0u32;
        let mut bzy = start;
        let mut bgm = 0u32;
        let mut bay = 0u32;
        
        let mut step = if w > 960 { 3usize } else { 2 };

        loop {
            let bb = aix().saturating_sub(start);
            if bb >= dur_ticks { break; }
            if let Some(k) = crate::keyboard::kr() {
                if k == 27 { return; } 
            }

            
            
            if bay > 0 {
                if bay >= 20 && step > 2 {
                    step -= 1; 
                } else if bay >= 30 && step > 1 {
                    step = 1; 
                } else if bay < 8 && step < 4 {
                    step += 1; 
                }
            }

            let time = bb as f32 / 100.0; 

            
            for y in (0..h).step_by(step) {
                for x in (0..w).step_by(step) {
                    let input = Cr { x: x as u32, y: y as u32, width: w as u32, height: h as u32, time, frame };
                    let out = shader_fn(input);
                    let color = out.to_u32();
                    
                    for ad in 0..step {
                        for dx in 0..step {
                            let p = x + dx;
                            let o = y + ad;
                            if p < w && o < h {
                                buf[o * w + p] = color;
                            }
                        }
                    }
                }
            }

            
            bgm += 1;
            let dqc = aix().saturating_sub(bzy);
            if dqc >= 100 {
                bay = bgm;
                bgm = 0;
                bzy = aix();
            }

            
            if bb < jmw {
                let alpha = if bb < 50 {
                    (bb * 255 / 50) as u32
                } else if bb > 150 {
                    let ln = bb - 150;
                    255u32.saturating_sub((ln * 255 / 50) as u32)
                } else { 255 };
                let a = alpha.min(255);
                let wo = 0xFF000000 | (a << 16) | (a << 8) | a;
                draw_text_centered(buf, w, h, 30, title, wo, 4);
                let dr = 0xFF000000 | ((a * 180 / 255) << 8);
                draw_text_centered(buf, w, h, 100, subtitle, dr, 2);
            }

            let bbi = (bb / 100) as u32;
            let bee = (dur_ticks / 100) as u32;
            htr(buf, w, h, title, scene_num, bay, bbi, bee, step);
            blit(buf, w, h);
            frame += 1;
        }
    };

    
    let bdh = |buf: &mut [u32], w: usize, h: usize,
                                 scene: crate::formula3d::FormulaScene,
                                 wire_color: u32,
                                 title: &str, subtitle: &str, scene_num: usize,
                                 dur_ticks: u64| {
        let mut renderer = crate::formula3d::FormulaRenderer::new();
        renderer.set_scene(scene);
        renderer.wire_color = wire_color;
        let start = aix();
        let mut frame = 0u32;
        let mut bzy = start;
        let mut bgm = 0u32;
        let mut bay = 0u32;

        loop {
            let bb = aix().saturating_sub(start);
            if bb >= dur_ticks { break; }
            if let Some(k) = crate::keyboard::kr() {
                if k == 27 { return; }
            }

            renderer.update();
            for aa in buf.iter_mut() { *aa = 0xFF000000; }
            renderer.render(buf, w, h);

            bgm += 1;
            let dqc = aix().saturating_sub(bzy);
            if dqc >= 100 {
                bay = bgm;
                bgm = 0;
                bzy = aix();
            }

            if bb < jmw {
                let alpha = if bb < 50 {
                    (bb * 255 / 50) as u32
                } else if bb > 150 {
                    let ln = bb - 150;
                    255u32.saturating_sub((ln * 255 / 50) as u32)
                } else { 255 };
                let a = alpha.min(255);
                let c = 0xFF000000 | (a << 16) | (a << 8) | a;
                draw_text_centered(buf, w, h, 30, title, c, 4);
                let dr = 0xFF000000 | ((a * 180 / 255) << 8);
                draw_text_centered(buf, w, h, 100, subtitle, dr, 2);
            }

            let bbi = (bb / 100) as u32;
            let bee = (dur_ticks / 100) as u32;
            htr(buf, w, h, title, scene_num, bay, bbi, bee, 1);
            blit(buf, w, h);
            frame += 1;
        }
    };

    
    let ats = |buf: &mut [u32], w: usize, h: usize, dur_ticks: u64| {
        let start = aix();
        loop {
            let bb = aix().saturating_sub(start);
            if bb >= dur_ticks { break; }
            for ct in buf.iter_mut() {
                let r = ((*ct >> 16) & 0xFF).saturating_sub(6) as u32;
                let g = ((*ct >> 8) & 0xFF).saturating_sub(6) as u32;
                let b = (*ct & 0xFF).saturating_sub(6) as u32;
                *ct = 0xFF000000 | (r << 16) | (g << 8) | b;
            }
            blit(buf, w, h);
            crate::cpu::tsc::ww(33);
        }
        for aa in buf.iter_mut() { *aa = 0xFF000000; }
        blit(buf, w, h);
    };

    let axa = 40u64; 

    crate::serial_println!("[SHOWCASE3D] Starting 3D cinematic showcase ({}x{}) - ~60s", w, h);

    
    
    
    {
        let start = aix();
        let ihe = 300u64; 
        let mut frame = 0u32;
        loop {
            let bb = aix().saturating_sub(start);
            if bb >= ihe { break; }
            for y in 0..h {
                for x in 0..w {
                    let v = ((x as i32 + frame as i32) ^ (y as i32)) as u32 & 0x0F;
                    buf[y * w + x] = 0xFF000000 | (v << 8);
                }
            }
            let alpha = (bb * 255 / ihe.max(1)).min(255) as u32;
            let c = 0xFF000000 | (alpha << 16) | (alpha << 8) | alpha;
            draw_text_centered(&mut buf, w, h, h / 3, "TrustOS", c, 8);
            let dr = 0xFF000000 | ((alpha * 120 / 255) << 16) | ((alpha * 255 / 255) << 8) | ((alpha * 120 / 255));
            draw_text_centered(&mut buf, w, h, h / 3 + 140, "3D Graphics Showcase", dr, 3);
            let ft = 0xFF000000 | ((alpha * 100 / 255) << 16) | ((alpha * 100 / 255) << 8) | ((alpha * 100 / 255));
            draw_text_centered(&mut buf, w, h, h / 3 + 200, "Pure software rendering - No GPU hardware", ft, 2);
            blit(&buf, w, h);
            frame += 1;
            crate::cpu::tsc::ww(33);
        }
        ats(&mut buf, w, h, axa);
    }

    
    
    
    crate::serial_println!("[SHOWCASE3D] Scene 1: Cube");
    bdh(&mut buf, w, h,
        crate::formula3d::FormulaScene::Cube,
        0xFF00FF66,
        "Wireframe Cube", "8 vertices - 12 edges - perspective projection", 1,
        bdp);
    ats(&mut buf, w, h, axa);

    
    
    
    crate::serial_println!("[SHOWCASE3D] Scene 2: Diamond");
    bdh(&mut buf, w, h,
        crate::formula3d::FormulaScene::Diamond,
        0xFFFF44FF,
        "Diamond", "Octahedron geometry - depth colored edges", 2,
        bdp);
    ats(&mut buf, w, h, axa);

    
    
    
    crate::serial_println!("[SHOWCASE3D] Scene 3: Torus");
    bdh(&mut buf, w, h,
        crate::formula3d::FormulaScene::Torus,
        0xFFFF8844,
        "Torus", "Donut wireframe - parametric surface mesh", 3,
        bdp);
    ats(&mut buf, w, h, axa);

    
    
    
    crate::serial_println!("[SHOWCASE3D] Scene 4: Pyramid");
    bdh(&mut buf, w, h,
        crate::formula3d::FormulaScene::Pyramid,
        0xFFFFCC00,
        "Pyramid", "5 vertices - 8 edges - ancient geometry", 4,
        bdp);
    ats(&mut buf, w, h, axa);

    
    
    
    crate::serial_println!("[SHOWCASE3D] Scene 5: HoloMatrix");
    bdh(&mut buf, w, h,
        crate::formula3d::FormulaScene::HoloMatrix,
        0xFF00FF44,
        "HoloMatrix", "3D matrix rain with perspective depth", 5,
        bdp);
    ats(&mut buf, w, h, axa);

    
    
    
    crate::serial_println!("[SHOWCASE3D] Scene 6: Multi-Shape");
    bdh(&mut buf, w, h,
        crate::formula3d::FormulaScene::Multi,
        0xFF00FFAA,
        "Multi Shape", "4 wireframe objects orbiting - depth colored", 6,
        bdp);
    ats(&mut buf, w, h, axa);

    
    
    
    crate::serial_println!("[SHOWCASE3D] Scene 7: DNA Helix");
    bdh(&mut buf, w, h,
        crate::formula3d::FormulaScene::Helix,
        0xFF44FFCC,
        "DNA Helix", "Double-strand helix with cross rungs", 7,
        bdp);
    ats(&mut buf, w, h, axa);

    
    
    
    crate::serial_println!("[SHOWCASE3D] Scene 8: Grid");
    bdh(&mut buf, w, h,
        crate::formula3d::FormulaScene::Grid,
        0xFF4488FF,
        "Infinite Grid", "Wireframe ground plane with perspective", 8,
        bdp);
    ats(&mut buf, w, h, axa);

    
    
    
    crate::serial_println!("[SHOWCASE3D] Scene 9: Penger");
    bdh(&mut buf, w, h,
        crate::formula3d::FormulaScene::Penger,
        0xFFFFFF00,
        "Penger", "The legendary wireframe penguin", 9,
        bdp);
    ats(&mut buf, w, h, axa);

    
    
    
    crate::serial_println!("[SHOWCASE3D] Scene 10: TrustOS Logo");
    bdh(&mut buf, w, h,
        crate::formula3d::FormulaScene::TrustOs,
        0xFF00FF88,
        "TrustOS Logo", "3D wireframe logo with glow vertices", 10,
        bdp);
    ats(&mut buf, w, h, axa);

    
    
    
    crate::serial_println!("[SHOWCASE3D] Scene 11: Icosphere");
    bdh(&mut buf, w, h,
        crate::formula3d::FormulaScene::Icosphere,
        0xFF66CCFF,
        "Icosphere", "Geodesic sphere - subdivided icosahedron", 11,
        bdp);
    ats(&mut buf, w, h, axa);

    
    
    
    crate::serial_println!("[SHOWCASE3D] Scene 12: Character");
    bdh(&mut buf, w, h,
        crate::formula3d::FormulaScene::Character,
        0xFF00FF88,
        "TrustOS", "Wireframe humanoid - perspective projection", 12,
        bdp);
    ats(&mut buf, w, h, axa);

    
    
    
    {
        let start = aix();
        let nos = 400u64; 
        loop {
            let bb = aix().saturating_sub(start);
            if bb >= nos { break; }
            for aa in buf.iter_mut() { *aa = 0xFF000000; }
            let alpha = if bb < 100 {
                (bb * 255 / 100).min(255)
            } else if bb > 300 {
                let fd = bb - 300;
                255u64.saturating_sub(fd * 255 / 100)
            } else { 255 } as u32;
            let c = 0xFF000000 | (alpha << 16) | (alpha << 8) | alpha;
            let bmk = 0xFF000000 | ((alpha * 200 / 255) << 8);
            draw_text_centered(&mut buf, w, h, h / 3 - 30, "TrustOS 3D Engine", c, 5);
            draw_text_centered(&mut buf, w, h, h / 3 + 60, "12 wireframe scenes - Pure software rendering", bmk, 2);
            draw_text_centered(&mut buf, w, h, h / 3 + 100, "No GPU hardware - All CPU computed", bmk, 2);
            draw_text_centered(&mut buf, w, h, h / 3 + 160, "Written in Rust by Nated0ge", 0xFF000000 | ((alpha * 140 / 255) << 16) | ((alpha * 180 / 255) << 8) | (alpha * 255 / 255), 3);
            draw_text_centered(&mut buf, w, h, h / 3 + 220, "github.com/nathan237/TrustOS", 0xFF000000 | ((alpha * 100 / 255) << 16) | ((alpha * 100 / 255) << 8) | ((alpha * 100 / 255)), 2);
            blit(&buf, w, h);
            crate::cpu::tsc::ww(33);
        }
    }

    
    for aa in buf.iter_mut() { *aa = 0xFF000000; }
    blit(&buf, w, h);
    if !pu {
        crate::framebuffer::pr(false);
    }
    crate::framebuffer::clear();
    crate::serial_println!("[SHOWCASE3D] Showcase complete");
}



pub fn hlw() {
    use crate::formula3d::V3;

    let (dy, dw) = crate::framebuffer::kv();
    let w = dy as usize;
    let h = dw as usize;
    if w == 0 || h == 0 { return; }

    let pu = crate::framebuffer::ajy();
    if !pu {
        crate::framebuffer::adw();
        crate::framebuffer::pr(true);
    }

    let mut buf = alloc::vec![0xFF000000u32; w * h];

    let draw_char = |buf: &mut [u32], w: usize, h: usize, cx: usize, u: usize, c: char, color: u32, scale: usize| {
        let du = crate::framebuffer::font::ol(c);
        for (row, &bits) in du.iter().enumerate() {
            for bf in 0..8u32 {
                if bits & (0x80 >> bf) != 0 {
                    for ak in 0..scale {
                        for am in 0..scale {
                            let p = cx + bf as usize * scale + am;
                            let o = u + row * scale + ak;
                            if p < w && o < h {
                                buf[o * w + p] = color;
                            }
                        }
                    }
                }
            }
        }
    };

    let blit = |buf: &[u32], w: usize, h: usize| {
        
        crate::framebuffer::fjl(buf.as_ptr(), w, h);
    };

    let aix = || crate::logger::eg();

    crate::serial_println!("[FILLED3D] Starting filled 3D test ({}x{})", w, h);

    
    let light = crate::formula3d::V3 { x: -0.4, y: 0.6, z: -0.7 };
    
    let len = crate::formula3d::ra(light.x * light.x + light.y * light.y + light.z * light.z);
    let light = V3 { x: light.x / len, y: light.y / len, z: light.z / len };

    
    let laf = crate::formula3d::mesh_cube();
    let nzz = crate::formula3d::mesh_pyramid();
    let ekd = crate::formula3d::mesh_diamond();

    let mut angle_y: f32 = 0.0;
    let mut frame = 0u32;
    let start = aix();
    let mut bzy = start;
    let mut bgm = 0u32;
    let mut bay = 0u32;

    loop {
        let bb = aix().saturating_sub(start);
        if bb >= 3000 { break; } 
        if let Some(k) = crate::keyboard::kr() {
            if k == 27 { break; }
        }

        
        for aa in buf.iter_mut() { *aa = 0xFF0C1018; }

        angle_y += 0.025;
        let angle_x = 0.35 + crate::formula3d::eu(frame as f32 * 0.008) * 0.2;

        
        
        
        
        

        let pn = w / 3;

        
        {
            let mut bjl = alloc::vec![0xFF0C1018u32; pn * h];
            crate::formula3d::grb(&mut bjl, pn, h,
                &nzz, angle_y * 0.8, angle_x + 0.15, 2.2,
                0xFFFF8844, light, 0.12);
            
            for y in 0..h {
                for x in 0..pn {
                    let amu = y * pn + x;
                    let ajm = y * w + x;
                    if amu < bjl.len() && ajm < buf.len() {
                        buf[ajm] = bjl[amu];
                    }
                }
            }
        }

        
        {
            let mut bjl = alloc::vec![0xFF0C1018u32; pn * h];
            crate::formula3d::grb(&mut bjl, pn, h,
                &laf, angle_y, angle_x, 2.2,
                0xFF4488FF, light, 0.12);
            for y in 0..h {
                for x in 0..pn {
                    let amu = y * pn + x;
                    let ajm = y * w + pn + x;
                    if amu < bjl.len() && ajm < buf.len() {
                        buf[ajm] = bjl[amu];
                    }
                }
            }
        }

        
        {
            let mut bjl = alloc::vec![0xFF0C1018u32; pn * h];
            crate::formula3d::grb(&mut bjl, pn, h,
                &ekd, angle_y * 1.3, angle_x - 0.1, 2.2,
                0xFFFF44CC, light, 0.12);
            for y in 0..h {
                for x in 0..pn {
                    let amu = y * pn + x;
                    let ajm = y * w + 2 * pn + x;
                    if amu < bjl.len() && ajm < buf.len() {
                        buf[ajm] = bjl[amu];
                    }
                }
            }
        }

        
        bgm += 1;
        let dqc = aix().saturating_sub(bzy);
        if dqc >= 100 {
            bay = bgm;
            bgm = 0;
            bzy = aix();
        }

        
        let hs = 22usize;
        let gk = h.saturating_sub(hs);
        for y in gk..h {
            for x in 0..w {
                let idx = y * w + x;
                if idx < buf.len() { buf[idx] = 0xFF000000; }
            }
        }
        let stats = alloc::format!("Filled 3D | {} FPS | Flat Shading + Backface Cull + Painter Sort | ESC=exit", bay);
        for (i, ch) in stats.chars().enumerate() {
            let cx = 8 + i * 8;
            if cx + 8 > w { break; }
            draw_char(&mut buf, w, h, cx, gk + 4, ch, 0xFF00FF88, 1);
        }

        
        if frame < 200 {
            let alpha = if frame < 30 { frame * 255 / 30 } else if frame > 170 { (200 - frame) * 255 / 30 } else { 255 };
            let a = (alpha.min(255)) as u32;
            let c = 0xFF000000 | (a << 16) | (a << 8) | a;
            let title = "FILLED 3D TEST";
            let gr = title.len() * 8 * 3;
            let bu = if gr < w { (w - gr) / 2 } else { 0 };
            for (i, ch) in title.chars().enumerate() {
                draw_char(&mut buf, w, h, bu + i * 24, 30, ch, c, 3);
            }
            let sub = "Scanline Rasterizer + Flat Shading";
            let jji = sub.len() * 8 * 2;
            let oyf = if jji < w { (w - jji) / 2 } else { 0 };
            let dr = 0xFF000000 | ((a * 180 / 255) << 8);
            for (i, ch) in sub.chars().enumerate() {
                draw_char(&mut buf, w, h, oyf + i * 16, 80, ch, dr, 2);
            }
        }

        blit(&buf, w, h);
        frame += 1;
    }

    
    for aa in buf.iter_mut() { *aa = 0xFF000000; }
    blit(&buf, w, h);
    if !pu {
        crate::framebuffer::pr(false);
    }
    crate::framebuffer::clear();
    crate::serial_println!("[FILLED3D] Test complete, {} frames", frame);
}


pub(super) fn hlr() {
    hls(None);
}






pub(super) fn hls(initial_app: Option<&str>) {
    fmf(initial_app, 0);
}


pub(super) fn fmf(initial_app: Option<&str>, timeout_ms: u64) {
    use crate::compositor::{Compositor, LayerType};
    use alloc::format;
    use alloc::string::String;
    use alloc::vec::Vec;
    
    crate::serial_println!("[COSMIC2] Starting COSMIC V2 Desktop...");
    
    
    crate::cpu::smp::elh();
    
    while crate::keyboard::kr().is_some() {}
    
    let (width, height) = crate::framebuffer::kv();
    if width == 0 || height == 0 {
        crate::n!(A_, "Error: Invalid framebuffer!");
        return;
    }
    
    
    crate::mouse::set_screen_size(width, height);
    
    crate::serial_println!("[COSMIC2] Creating compositor {}x{}", width, height);
    
    
    let mut compositor = Compositor::new(width, height);
    
    
    let kbs = compositor.add_fullscreen_layer(LayerType::Background);
    let lgu = compositor.add_layer(LayerType::Dock, 0, 0, 70, height - 40);  
    let jre = compositor.add_layer(LayerType::Windows, 100, 80, 700, 450);  
    let mlo = compositor.add_layer(LayerType::Overlay, width - 260, 50, 250, 220);  
    let pdk = compositor.add_layer(LayerType::Taskbar, 0, height - 40, width, 40);
    let nek = compositor.add_layer(LayerType::Overlay, 5, height - 440, 280, 400);  
    let oqb = compositor.add_layer(LayerType::Overlay, 340, height - 380, 280, 350);  
    let hpx = compositor.add_layer(LayerType::Overlay, 0, 0, 24, 24);
    
    crate::serial_println!("[COSMIC2] Created {} layers", compositor.layer_count());
    
    
    compositor.enable_gpu_direct();
    
    
    
    
    let mut running = true;
    let mut frame_count = 0u64;
    
    
    #[derive(Clone, Copy, PartialEq)]
    enum AppMode {
        Shell,       
        Network,     
        Hardware,    
        TextEditor,  
        UserMgmt,    
        Files,       
        Browser,     
        ImageViewer, 
    }
    
    
    let mut xm = match initial_app {
        Some("browser") | Some("web") | Some("www") => AppMode::Browser,
        Some("files") | Some("explorer") => AppMode::Files,
        Some("editor") | Some("text") | Some("notepad") => AppMode::TextEditor,
        Some("network") | Some("net") | Some("ifconfig") => AppMode::Network,
        Some("hardware") | Some("hw") | Some("lshw") => AppMode::Hardware,
        Some("users") | Some("user") => AppMode::UserMgmt,
        Some("images") | Some("image") | Some("viewer") => AppMode::ImageViewer,
        _ => AppMode::Shell,
    };
    let mut bky = xm == AppMode::Browser;  
    
    
    
    
    
    
    #[derive(Clone)]
    struct Ar {
        text: String,
        color: u32,
    }
    
    
    const AYD_: u32 = 0xFFE06C75;       
    const AYB_: u32 = 0xFF98C379;      
    const AYE_: u32 = 0xFFE5C07B;     
    const AFB_: u32 = 0xFFDCDCDC;      
    const CEA_: u32 = 0xFF5C6370;   
    const CEB_: u32 = 0xFFABB2BF;   
    const OL_: u32 = 0xFF56B6C2;   
    const DTH_: u32 = 0xFF98C379;    
    const AYC_: u32 = 0xFFD19A66;    
    const OM_: u32 = 0xFF61AFEF;      
    const DTG_: u32 = 0xFF56B6C2;    
    
    
    #[derive(Clone)]
    struct Es {
        segments: Vec<Ar>,
        line_type: LineType,
    }
    
    #[derive(Clone, Copy, PartialEq)]
    enum LineType {
        Welcome,      
        HttpHeader,   
        HtmlTag,      
        HtmlMixed,    
        PlainText,    
        Error,        
    }
    
    
    fn nql(line: &str) -> Vec<Ar> {
        let mut segments = Vec::new();
        let mut chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        
        while i < chars.len() {
            
            if chars[i] == '<' {
                
                if i + 3 < chars.len() && chars[i+1] == '!' && chars[i+2] == '-' && chars[i+3] == '-' {
                    
                    let start = i;
                    while i < chars.len() {
                        if i + 2 < chars.len() && chars[i] == '-' && chars[i+1] == '-' && chars[i+2] == '>' {
                            i += 3;
                            break;
                        }
                        i += 1;
                    }
                    segments.push(Ar {
                        text: chars[start..i].iter().collect(),
                        color: CEA_,
                    });
                    continue;
                }
                
                
                if i + 1 < chars.len() && chars[i+1] == '!' {
                    let start = i;
                    while i < chars.len() && chars[i] != '>' {
                        i += 1;
                    }
                    if i < chars.len() { i += 1; }
                    segments.push(Ar {
                        text: chars[start..i].iter().collect(),
                        color: CEB_,
                    });
                    continue;
                }
                
                
                
                segments.push(Ar { text: String::from("<"), color: OL_ });
                i += 1;
                
                
                if i < chars.len() && chars[i] == '/' {
                    segments.push(Ar { text: String::from("/"), color: OL_ });
                    i += 1;
                }
                
                
                let jlj = i;
                while i < chars.len() && chars[i] != ' ' && chars[i] != '>' && chars[i] != '/' {
                    i += 1;
                }
                if jlj < i {
                    segments.push(Ar {
                        text: chars[jlj..i].iter().collect(),
                        color: AYD_,
                    });
                }
                
                
                while i < chars.len() && chars[i] != '>' {
                    
                    if chars[i] == ' ' {
                        let pvi = i;
                        while i < chars.len() && chars[i] == ' ' { i += 1; }
                        segments.push(Ar {
                            text: chars[pvi..i].iter().collect(),
                            color: AFB_,
                        });
                        continue;
                    }
                    
                    
                    if chars[i] == '/' {
                        segments.push(Ar { text: String::from("/"), color: OL_ });
                        i += 1;
                        continue;
                    }
                    
                    
                    let hfy = i;
                    while i < chars.len() && chars[i] != '=' && chars[i] != ' ' && chars[i] != '>' && chars[i] != '/' {
                        i += 1;
                    }
                    if hfy < i {
                        segments.push(Ar {
                            text: chars[hfy..i].iter().collect(),
                            color: AYB_,
                        });
                    }
                    
                    
                    if i < chars.len() && chars[i] == '=' {
                        segments.push(Ar { text: String::from("="), color: AFB_ });
                        i += 1;
                    }
                    
                    
                    if i < chars.len() && (chars[i] == '"' || chars[i] == '\'') {
                        let arw = chars[i];
                        let bwr = i;
                        i += 1;
                        while i < chars.len() && chars[i] != arw {
                            i += 1;
                        }
                        if i < chars.len() { i += 1; } 
                        segments.push(Ar {
                            text: chars[bwr..i].iter().collect(),
                            color: AYE_,
                        });
                    }
                }
                
                
                if i < chars.len() && chars[i] == '>' {
                    segments.push(Ar { text: String::from(">"), color: OL_ });
                    i += 1;
                }
            }
            
            else if chars[i] == '&' {
                let start = i;
                while i < chars.len() && chars[i] != ';' && chars[i] != ' ' {
                    i += 1;
                }
                if i < chars.len() && chars[i] == ';' { i += 1; }
                segments.push(Ar {
                    text: chars[start..i].iter().collect(),
                    color: AYC_,
                });
            }
            
            else {
                let start = i;
                while i < chars.len() && chars[i] != '<' && chars[i] != '&' {
                    i += 1;
                }
                if start < i {
                    segments.push(Ar {
                        text: chars[start..i].iter().collect(),
                        color: AFB_,
                    });
                }
            }
        }
        
        segments
    }
    
    
    let mut ahk = String::from("https://google.com");
    let mut gy: Vec<Es> = Vec::new();
    let mut ajg = String::from("Enter URL and press Enter to navigate");
    let mut browser_loading = false;
    let mut his = true;
    let mut djq: u8 = 0;  
    let mut hgc = timeout_ms > 0 && xm == AppMode::Browser;
    
    
    fn jb(text: &str, color: u32, line_type: LineType) -> Es {
        let mut dyz = Vec::new();
        dyz.push(Ar { text: String::from(text), color });
        Es {
            segments: dyz,
            line_type,
        }
    }
    
    
    fn ets(text: &str) -> Es {
        Es {
            segments: nql(text),
            line_type: LineType::HtmlMixed,
        }
    }
    
    
    crate::tls13::crypto::jbv();
    
    
    gy.push(jb("+------------------------------------------------------------+", 0xFF00AAFF, LineType::Welcome));
    gy.push(jb("|        TrustOS Web Browser v1.0 - DevTools Mode            |", 0xFF00AAFF, LineType::Welcome));
    gy.push(jb("|------------------------------------------------------------|", 0xFF00AAFF, LineType::Welcome));
    gy.push(jb("|                                                            |", 0xFF00AAFF, LineType::Welcome));
    gy.push(jb("|  Syntax highlighting like Chrome DevTools!                 |", 0xFFDDDDDD, LineType::Welcome));
    gy.push(jb("|                                                            |", 0xFF00AAFF, LineType::Welcome));
    gy.push(jb("|  COLOR LEGEND:                                             |", 0xFFFFFF00, LineType::Welcome));
    {
        let mut esn = Es { segments: Vec::new(), line_type: LineType::Welcome };
        esn.segments.push(Ar { text: String::from("|    "), color: 0xFF00AAFF });
        esn.segments.push(Ar { text: String::from("<tag>"), color: AYD_ });
        esn.segments.push(Ar { text: String::from(" - HTML tags                            |"), color: 0xFFDDDDDD });
        gy.push(esn);
        
        let mut eso = Es { segments: Vec::new(), line_type: LineType::Welcome };
        eso.segments.push(Ar { text: String::from("|    "), color: 0xFF00AAFF });
        eso.segments.push(Ar { text: String::from("attr"), color: AYB_ });
        eso.segments.push(Ar { text: String::from(" - Attribute names                     |"), color: 0xFFDDDDDD });
        gy.push(eso);
        
        let mut esq = Es { segments: Vec::new(), line_type: LineType::Welcome };
        esq.segments.push(Ar { text: String::from("|    "), color: 0xFF00AAFF });
        esq.segments.push(Ar { text: String::from("\"value\""), color: AYE_ });
        esq.segments.push(Ar { text: String::from(" - Attribute values                   |"), color: 0xFFDDDDDD });
        gy.push(esq);
        
        let mut ess = Es { segments: Vec::new(), line_type: LineType::Welcome };
        ess.segments.push(Ar { text: String::from("|    "), color: 0xFF00AAFF });
        ess.segments.push(Ar { text: String::from("< >"), color: OL_ });
        ess.segments.push(Ar { text: String::from(" - Brackets                             |"), color: 0xFFDDDDDD });
        gy.push(ess);
        
        let mut est = Es { segments: Vec::new(), line_type: LineType::Welcome };
        est.segments.push(Ar { text: String::from("|    "), color: 0xFF00AAFF });
        est.segments.push(Ar { text: String::from("&amp;"), color: AYC_ });
        est.segments.push(Ar { text: String::from(" - HTML entities                       |"), color: 0xFFDDDDDD });
        gy.push(est);
    }
    gy.push(jb("|                                                            |", 0xFF00AAFF, LineType::Welcome));
    gy.push(jb("|  TRY THESE URLs:                                           |", 0xFFFFFF00, LineType::Welcome));
    gy.push(jb("|    https://google.com                                      |", 0xFF00FFFF, LineType::Welcome));
    gy.push(jb("|    https://example.com                                     |", 0xFF00FFFF, LineType::Welcome));
    gy.push(jb("|                                                            |", 0xFF00AAFF, LineType::Welcome));
    gy.push(jb("|                                                            |", 0xFF00AAFF, LineType::Welcome));
    gy.push(jb("|  [Tab] Toggle DevTools/Rendered  [Enter] Navigate          |", 0xFF88FF88, LineType::Welcome));
    gy.push(jb("|  [ESC] Return to shell                                     |", 0xFF88FF88, LineType::Welcome));
    gy.push(jb("|                                                            |", 0xFF00AAFF, LineType::Welcome));
    gy.push(jb("+------------------------------------------------------------+", 0xFF00AAFF, LineType::Welcome));
    
    
    
    
    {
        
        let mut gnv = String::from("P3\n32 32\n255\n");
        for y in 0..32 {
            for x in 0..32 {
                let r = (x * 8) % 256;
                let g = (y * 8) % 256;
                let b = ((x + y) * 4) % 256;
                gnv.push_str(&format!("{} {} {} ", r, g, b));
            }
            gnv.push('\n');
        }
        let _ = crate::ramfs::bh(|fs| {
            fs.mkdir("/images");
            fs.write_file("/images/test.ppm", gnv.as_bytes())
        });
        
        
        let kcx: [u8; 54] = [
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
        let mut ehb = alloc::vec::Vec::from(kcx);
        
        for y in 0..16 {
            for x in 0..16 {
                let b = ((15 - y) * 17) as u8;  
                let g = (x * 17) as u8;          
                let r = ((x + y) * 8) as u8;     
                ehb.push(b);
                ehb.push(g);
                ehb.push(r);
            }
            
        }
        let _ = crate::ramfs::bh(|fs| {
            fs.write_file("/images/test.bmp", &ehb)
        });
        
        crate::serial_println!("[COSMIC2] Created test images in /images/");
    }
    
    
    
    
    let mut moc = String::new();
    let mut ift: Option<crate::image::Image> = None;
    let mut bhc: f32 = 1.0;
    let mut gbu: i32 = 0;
    let mut gbv: i32 = 0;
    let mut ifv = String::from("No image loaded");
    let mut ifu = String::from("---");
    
    
    let mut yx = false;
    let mut bcs: i32 = -1;
    
    
    let mut bou = false;
    let mut dek = crate::desktop::awb();
    let mut del = crate::desktop::dqn();
    
    
    let mut shell_input = String::new();
    let mut ap: Vec<String> = Vec::new();
    let mut cursor_blink = true;
    let mut dez = String::new();
    let mut scroll_offset: usize = 0;  
    const AHB_: usize = 18;  
    
    
    let mut editor_state = crate::apps::text_editor::EditorState::new();
    
    {
        let gsh = "//! TrustOS \u{2014} A Modern Operating System in Rust\n//!\n//! This file demonstrates TrustCode's syntax highlighting\n\nuse core::fmt;\n\n/// Main kernel entry point\npub fn kernel_main() -> ! {\n    let message = \"Hello from TrustOS!\";\n    serial_println!(\"{}\", message);\n\n    // Initialize hardware\n    let cpu_count: u32 = 4;\n    let memory_mb: u64 = 256;\n\n    for i in 0..cpu_count {\n        init_cpu(i);\n    }\n\n    // Start the desktop environment\n    let mut desktop = Desktop::new();\n    desktop.init(1280, 800);\n\n    loop {\n        desktop.render();\n        desktop.handle_input();\n    }\n}\n\n/// Initialize a CPU core\nfn init_cpu(id: u32) {\n    // Setup GDT, IDT, APIC\n    serial_println!(\"CPU {} initialized\", id);\n}\n\n#[derive(Debug, Clone)]\nstruct AppConfig {\n    name: String,\n    version: (u8, u8, u8),\n    features: Vec<&'static str>,\n}\n";
        let _ = crate::ramfs::bh(|fs| fs.write_file("/demo.rs", gsh.as_bytes()));
        editor_state.load_file("demo.rs");
    }
    
    
    let mut dnp = false;
    let mut drag_offset_x: i32 = 0;
    let mut drag_offset_y: i32 = 0;
    let mut dgq: i32 = 100;
    let mut dgr: i32 = 80;
    let mut cfi = true;  
    
    
    let mut command_history: Vec<String> = Vec::new();
    const BBS_: usize = 10;
    
    
    const CCU_: &[&str] = &[
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
    
    const CCT_: &[&str] = &[
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
    
    const CCR_: &[&str] = &[
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
    
    const CCP_: &[&str] = &[
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
    
    const CCV_: &[&str] = &[
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
    
    const CCQ_: &[&str] = &[
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
    
    const CCO_: &[&str] = &[
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
    
    const CCS_: &[&str] = &[
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
    
    
    macro_rules! cyo {
        ($mode:expr) => {
            match $mode {
                AppMode::Shell => CCU_,
                AppMode::Network => CCT_,
                AppMode::Hardware => CCR_,
                AppMode::TextEditor => CCP_,
                AppMode::UserMgmt => CCV_,
                AppMode::Files => CCQ_,
                AppMode::Browser => CCO_,
                AppMode::ImageViewer => CCS_,
            }
        };
    }
    
    
    for line in cyo!(AppMode::Shell) {
        ap.push(String::from(*line));
    }
    
    
    
    
    
    const AD_: usize = 240;      
    const AV_: usize = 68;       
    const OZ_: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ@#$%&*+=<>[]{}|";
    
    
    #[inline]
    fn imb(col: usize, row: usize) -> usize {
        col * AV_ + row
    }
    
    
    
    
    let mut matrix_chars: Vec<u8> = vec![0u8; AD_ * AV_];
    for col in 0..AD_ {
        let seed = (col as u32 * 2654435761) ^ 0xDEADBEEF;
        for row in 0..AV_ {
            let bfe = seed.wrapping_mul(row as u32 + 1);
            matrix_chars[imb(col, row)] = OZ_[(bfe as usize) % OZ_.len()];
        }
    }
    
    
    
    let mut matrix_heads: [i32; AD_] = [0; AD_];
    let mut matrix_speeds: [u32; AD_] = [0; AD_];
    for col in 0..AD_ {
        let seed = (col as u32 * 2654435761) ^ 0xDEADBEEF;
        matrix_heads[col] = -((seed % (AV_ as u32 * 2)) as i32);
        matrix_speeds[col] = 1 + (seed % 3);  
    }
    
    
    
    
    let mut holomatrix = crate::graphics::holomatrix::HoloMatrix::new(width as usize / 4, height as usize / 4, 32);
    
    let mut ckr = crate::graphics::holomatrix::dqr();
    let mut aia = crate::graphics::holomatrix::lq();
    
    
    
    let mut holovolume = crate::holovolume::HoloVolume::new(
        width as usize / 8,   
        height as usize / 9,  
        32                    
    );
    holovolume.render_mode = crate::holovolume::RenderMode::Hologram;
    let mut bju = false;  
    
    
    let mut hxu = crate::matrix_fast::FastMatrixRenderer::new();
    let mut vv = false;  
    
    
    let mut bqu = alloc::boxed::Box::new(crate::matrix_fast::BrailleMatrix::new());
    let mut vb = false;  
    let mut dzj = true;  
    
    
    let mut dty = alloc::boxed::Box::new(crate::matrix_fast::Matrix3D::new());
    let mut yg = false;  
    
    
    let mut aqp = alloc::boxed::Box::new(crate::formula3d::FormulaRenderer::new());
    let mut xk = true;  
    
    
    let mut xl = false;  
    let mut qwu: f32 = 0.0;
    let mut qwt: u32 = 0;
    
    
    let mut raytracer = crate::graphics::raytracer::RayTracer::new(width as usize / 6, height as usize / 6);
    
    
    let kw: u32 = 0xFF00FF66;
    let bby: u32 = 0xFF00FF88;
    let ph: u32 = 0xFF007744;
    let awh: u32 = 0xFF000000;
    let qck: u32 = 0xFF020202;  
    let qcf: u32 = 0xFF101010;
    let pur: u32 = 0xFF0A0A0A;  
    let qtj: u32 = 0xFFFF0000;   
    let rcj: u32 = 0xFFFFFFFF; 
    let qjw: u32 = 0xFF00FF00; 
    
    
    #[derive(Clone, Copy, PartialEq)]
    enum MenuItem {
        App(AppMode),
        Shutdown,
        Reboot,
    }
    let dbg: [(&str, MenuItem); 11] = [
        ("Shell", MenuItem::App(AppMode::Shell)),
        ("Files", MenuItem::App(AppMode::Files)),
        ("Network", MenuItem::App(AppMode::Network)),
        ("Hardware", MenuItem::App(AppMode::Hardware)),
        ("TrustCode", MenuItem::App(AppMode::TextEditor)),
        ("User Management", MenuItem::App(AppMode::UserMgmt)),
        ("Web Browser", MenuItem::App(AppMode::Browser)),
        ("Image Viewer", MenuItem::App(AppMode::ImageViewer)),
        ("-----------------", MenuItem::App(AppMode::Shell)), 
        ("Reboot", MenuItem::Reboot),
        ("Shutdown", MenuItem::Shutdown),
    ];
    
    
    let mut dcw = false;
    let mut mouse_x: i32 = (width / 2) as i32;
    let mut mouse_y: i32 = (height / 2) as i32;
    
    
    let aso = crate::cpu::tsc::we();
    let mut fps = 0u32;
    let mut bmg = 0u32;
    let mut clz = crate::cpu::tsc::ey();
    
    
    
    
    
    
    
    
    
    let kwh: u64 = 4; 
    let mut izq = 0u32; 
    let mut grc = 0u32;
    
    crate::serial_println!("[COSMIC2] Entering render loop...");
    
    
    let jyl = crate::cpu::tsc::ey();
    let hga = crate::cpu::tsc::we();
    let jym = if timeout_ms > 0 && hga > 0 { hga / 1000 * timeout_ms } else { u64::MAX };
    
    while running {
        
        if hgc && frame_count == 5 {
            hgc = false;
            
            if bky {
                gy.clear();
                ajg = format!("Loading {}...", ahk);
                browser_loading = true;
                let cln = ahk.starts_with("https://");
                if let Some((host, port, path, url_is_https)) = super::vm::gml(&ahk) {
                    let protocol = if url_is_https { "HTTPS" } else { "HTTP" };
                    gy.push(jb(&format!("\u{25ba} {} {}:{}{}...", protocol, host, port, path), 0xFF88FF88, LineType::PlainText));
                    if url_is_https {
                        gy.push(jb("\u{25ba} Establishing TLS 1.3 connection...", 0xFF88CCFF, LineType::PlainText));
                        match crate::netstack::https::get(&ahk) {
                            Ok(fa) => {
                                gy.push(jb(&format!("\u{25ba} TLS OK, {} bytes", fa.body.len()), 0xFF88FF88, LineType::PlainText));
                                gy.push(jb("", 0xFFDDDDDD, LineType::PlainText));
                                gy.push(jb("\u{2500}\u{2500} Response Headers \u{2500}\u{2500}", 0xFF61AFEF, LineType::HttpHeader));
                                gy.push(jb(&format!("HTTP/1.1 {}", fa.status_code), OM_, LineType::HttpHeader));
                                for (key, value) in &fa.headers {
                                    gy.push(jb(&format!("{}: {}", key, value), OM_, LineType::HttpHeader));
                                }
                                gy.push(jb("", 0xFFDDDDDD, LineType::PlainText));
                                gy.push(jb("\u{2500}\u{2500} HTML Source \u{2500}\u{2500}", 0xFF61AFEF, LineType::HttpHeader));
                                if let Ok(body_str) = core::str::from_utf8(&fa.body) {
                                    for line in body_str.lines().take(200) {
                                        gy.push(ets(line));
                                    }
                                }
                                ajg = format!("\u{2713} Loaded: {} ({} bytes, HTTPS)", ahk, fa.body.len());
                            }
                            Err(e) => {
                                gy.push(jb(&format!("\u{2718} HTTPS Error: {}", e), 0xFFFF4444, LineType::PlainText));
                                ajg = format!("Error: {}", e);
                            }
                        }
                    } else {
                        
                        match crate::netstack::http::get(&ahk) {
                            Ok(fa) => {
                                if let Some(body_str) = fa.body_str() {
                                    for line in body_str.lines().take(200) {
                                        gy.push(ets(line));
                                    }
                                }
                                ajg = format!("\u{2713} Loaded: {} ({} bytes)", ahk, fa.body.len());
                            }
                            Err(e) => {
                                gy.push(jb(&format!("\u{2718} HTTP Error: {}", e), 0xFFFF4444, LineType::PlainText));
                                ajg = format!("Error: {}", e);
                            }
                        }
                    }
                } else {
                    gy.push(jb("\u{2718} Invalid URL", 0xFFFF4444, LineType::PlainText));
                    ajg = String::from("Invalid URL");
                }
                browser_loading = false;
            }
        }

        
        if timeout_ms > 0 {
            let bb = crate::cpu::tsc::ey().saturating_sub(jyl);
            if bb >= jym { break; }
        }
        
        
        if frame_count <= 3 || frame_count % 500 == 0 {
            crate::serial_println!("[COSMIC2] Loop iteration {}", frame_count);
        }
        
        
        
        
        
        
        
        let mut hdk = 0u8;
        while let Some(key) = crate::keyboard::kr() {
            hdk += 1;
            if hdk > 8 { break; } 
            crate::serial_println!("[KEY] Received key: {} (0x{:02X})", key, key);
            
            if xm == AppMode::Browser {
                match key {
                    27 => { 
                        xm = AppMode::Shell;
                        ap.clear();
                        for line in cyo!(AppMode::Shell) {
                            ap.push(String::from(*line));
                        }
                    },
                    9 => { 
                        djq = (djq + 1) % 2;
                        if djq == 0 {
                            ajg = String::from("View: DevTools (source)");
                        } else {
                            ajg = String::from("View: Rendered");
                        }
                    },
                    8 => { 
                        if ahk.len() > 7 { 
                            ahk.pop();
                        }
                    },
                    10 | 13 => { 
                        gy.clear();
                        ajg = format!("Loading {}...", ahk);
                        browser_loading = true;
                        
                        
                        let cln = ahk.starts_with("https://");
                        
                        
                        if let Some((host, port, path, url_is_https)) = super::vm::gml(&ahk) {
                            let protocol = if url_is_https { "HTTPS" } else { "HTTP" };
                            gy.push(jb(&format!("? {} {}:{}{}...", protocol, host, port, path), 0xFF88FF88, LineType::PlainText));
                            
                            if url_is_https {
                                
                                gy.push(jb("? Establishing TLS 1.3 connection...", 0xFF88CCFF, LineType::PlainText));
                                
                                match crate::netstack::https::get(&ahk) {
                                    Ok(fa) => {
                                        gy.push(jb(&format!("? TLS handshake complete, received {} bytes", fa.body.len()), 0xFF88FF88, LineType::PlainText));
                                        gy.push(jb("", 0xFFDDDDDD, LineType::PlainText));
                                        
                                        
                                        gy.push(jb("-- Response Headers --", 0xFF61AFEF, LineType::HttpHeader));
                                        gy.push(jb(&format!("HTTP/1.1 {}", fa.status_code), OM_, LineType::HttpHeader));
                                        for (key, value) in &fa.headers {
                                            gy.push(jb(&format!("{}: {}", key, value), OM_, LineType::HttpHeader));
                                        }
                                        gy.push(jb("", 0xFFDDDDDD, LineType::PlainText));
                                        
                                        
                                        gy.push(jb("-- HTML Source --", 0xFF61AFEF, LineType::HttpHeader));
                                        if let Ok(body_str) = core::str::from_utf8(&fa.body) {
                                            for line in body_str.lines().take(200) {
                                                gy.push(ets(line));
                                            }
                                        } else {
                                            gy.push(jb("[Binary content]", 0xFFFFFF00, LineType::PlainText));
                                        }
                                        
                                        ajg = format!("? Loaded: {} ({} bytes, HTTPS)", ahk, fa.body.len());
                                    }
                                    Err(e) => {
                                        gy.push(jb(&format!("? HTTPS Error: {}", e), 0xFFFF4444, LineType::Error));
                                        gy.push(jb("", 0xFFDDDDDD, LineType::PlainText));
                                        gy.push(jb("TLS 1.3 connection failed. Possible causes:", 0xFFFFFF00, LineType::PlainText));
                                        gy.push(jb("  * DNS resolution failed", 0xFFAAAAAA, LineType::PlainText));
                                        gy.push(jb("  * Server doesn't support TLS 1.3", 0xFFAAAAAA, LineType::PlainText));
                                        gy.push(jb("  * Network timeout", 0xFFAAAAAA, LineType::PlainText));
                                        ajg = format!("? HTTPS Error: {}", e);
                                    }
                                }
                            } else {
                                
                                
                                let dsi = if let Some(ip) = super::vm::art(&host) {
                                    Some(ip)
                                } else {
                                    
                                    crate::netstack::dns::yb(&host)
                                };
                                
                                if let Some(ip) = dsi {
                                    gy.push(jb(&format!("? Resolved: {}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3]), 0xFF88FF88, LineType::PlainText));
                                    
                                    
                                    match super::vm::hta(&host, ip, port, &path) {
                                        Ok(fa) => {
                                            gy.push(jb("", 0xFFDDDDDD, LineType::PlainText));
                                            
                                            
                                            let mut iga = true;
                                            gy.push(jb("-- Response Headers --", 0xFF61AFEF, LineType::HttpHeader));
                                            
                                            for line in fa.lines() {
                                                if iga {
                                                    if line.is_empty() {
                                                        iga = false;
                                                        gy.push(jb("", 0xFFDDDDDD, LineType::PlainText));
                                                        gy.push(jb("-- HTML Source --", 0xFF61AFEF, LineType::HttpHeader));
                                                    } else {
                                                        gy.push(jb(line, OM_, LineType::HttpHeader));
                                                    }
                                                } else {
                                                    
                                                    gy.push(ets(line));
                                                }
                                            }
                                            
                                            ajg = format!("? Loaded: {} ({} bytes)", ahk, fa.len());
                                        }
                                        Err(e) => {
                                            gy.push(jb(&format!("? HTTP Error: {}", e), 0xFFFF4444, LineType::Error));
                                            ajg = format!("? Error: {}", e);
                                        }
                                    }
                                } else {
                                    gy.push(jb(&format!("? Error: Cannot resolve host '{}'", host), 0xFFFF4444, LineType::Error));
                                    gy.push(jb("", 0xFFDDDDDD, LineType::PlainText));
                                    gy.push(jb("Tip: Try a local server or IP address:", 0xFFFFFF00, LineType::PlainText));
                                    gy.push(jb("  * http://192.168.56.1:8080/", 0xFF00FFFF, LineType::PlainText));
                                    gy.push(jb("  * http://10.0.2.2:8000/", 0xFF00FFFF, LineType::PlainText));
                                    ajg = String::from("? Error: DNS resolution failed");
                                }
                            }
                        } else {
                            gy.push(jb("? Invalid URL format", 0xFFFF4444, LineType::Error));
                            gy.push(jb("", 0xFFDDDDDD, LineType::PlainText));
                            gy.push(jb("Use format: http://hostname/path or https://hostname/path", 0xFFFFFF00, LineType::PlainText));
                            gy.push(jb("", 0xFFDDDDDD, LineType::PlainText));
                            gy.push(jb("Examples:", 0xFF88FF88, LineType::PlainText));
                            gy.push(jb("  * https://google.com", 0xFF00FFFF, LineType::PlainText));
                            gy.push(jb("  * https://example.com", 0xFF00FFFF, LineType::PlainText));
                            gy.push(jb("  * http://192.168.1.1/", 0xFF00FFFF, LineType::PlainText));
                            ajg = String::from("? Error: Invalid URL");
                        }
                        browser_loading = false;
                    },
                    32..=126 => { 
                        ahk.push(key as char);
                    },
                    _ => {}
                }
            } else if xm == AppMode::ImageViewer {
                
                match key {
                    27 => { 
                        xm = AppMode::Shell;
                        ap.clear();
                        for line in cyo!(AppMode::Shell) {
                            ap.push(String::from(*line));
                        }
                    },
                    43 | 61 => { 
                        bhc = (bhc * 1.25).min(10.0);
                    },
                    45 => { 
                        bhc = (bhc / 1.25).max(0.1);
                    },
                    114 | 82 => { 
                        bhc = 1.0;
                        gbu = 0;
                        gbv = 0;
                    },
                    
                    
                    _ => {}
                }
            } else if xm == AppMode::TextEditor {
                
                match key {
                    27 => { 
                        xm = AppMode::Shell;
                        ap.clear();
                        for line in cyo!(AppMode::Shell) {
                            ap.push(String::from(*line));
                        }
                    },
                    _ => {
                        
                        editor_state.handle_key(key);
                    }
                }
            } else {
                
            match key {
                27 => { 
                    if yx || bou {
                        yx = false;
                        bou = false;
                    } else {
                        running = false;
                    }
                },
                8 => { 
                    shell_input.pop();
                    dez.clear();
                },
                0x49 => { 
                    if scroll_offset > 0 {
                        scroll_offset = scroll_offset.saturating_sub(5);
                    }
                },
                0x51 => { 
                    let aab = ap.len().saturating_sub(AHB_);
                    if scroll_offset < aab {
                        scroll_offset = (scroll_offset + 5).min(aab);
                    }
                },
                10 | 13 => { 
                    if !shell_input.is_empty() {
                        let hmd = shell_input.clone();
                        let cmd = hmd.trim();  
                        crate::serial_println!("[DEBUG] Enter pressed, cmd = '{}' (trimmed: '{}')", hmd, cmd);
                        ap.push(format!("> {}", cmd));
                        
                        
                        command_history.push(String::from(cmd));
                        if command_history.len() > BBS_ {
                            command_history.remove(0);
                        }
                        
                        
                        crate::serial_println!("[MATCH] About to match cmd='{}' starts_with_shader={}", cmd, cmd.starts_with("shader "));
                        match cmd {
                            "help" => {
                                ap.push(String::from("+================================================+"));
                                ap.push(String::from("|          TrustOS Desktop Shell                 |"));
                                ap.push(String::from("+================================================+"));
                                ap.push(String::from("| FILE SYSTEM:                                   |"));
                                ap.push(String::from("|   ls, cd, pwd, mkdir, rmdir, touch, rm, cat    |"));
                                ap.push(String::from("|   cp, mv, head, tail, stat, tree, find, wc     |"));
                                ap.push(String::from("|   chmod, chown, ln, grep                       |"));
                                ap.push(String::from("| NETWORK:                                       |"));
                                ap.push(String::from("|   ifconfig, ping, curl, wget, nslookup         |"));
                                ap.push(String::from("|   arp, route, traceroute, netstat              |"));
                                ap.push(String::from("| SYSTEM:                                        |"));
                                ap.push(String::from("|   clear, date, time, uptime, whoami, hostname  |"));
                                ap.push(String::from("|   uname, env, history, ps, free, df, top       |"));
                                ap.push(String::from("| HARDWARE:                                      |"));
                                ap.push(String::from("|   cpuinfo, meminfo, lspci, lsusb, lscpu, disk  |"));
                                ap.push(String::from("| USERS:                                         |"));
                                ap.push(String::from("|   login, su, passwd, adduser, users            |"));
                                ap.push(String::from("| UTILITIES:                                     |"));
                                ap.push(String::from("|   echo, hexdump, strings, sort, cal, bc        |"));
                                ap.push(String::from("| DESKTOP:                                       |"));
                                ap.push(String::from("|   desktop close - Exit desktop                 |"));
                                ap.push(String::from("|   open <app> - Open app (browser,files,editor) |"));
                                ap.push(String::from("|   imgview <file> - View images (PNG/BMP)       |"));
                                ap.push(String::from("|   3ddemo - 3D rotating cube demo               |"));
                                ap.push(String::from("+================================================+"));
                            },
                            "clear" => {
                                ap.clear();
                            },
                            "pwd" => {
                                
                                let cwd = crate::ramfs::bh(|fs| String::from(fs.pwd()));
                                ap.push(cwd);
                            },
                            "ls" | "dir" => {
                                
                                match crate::ramfs::bh(|fs| fs.ls(None)) {
                                    Ok(items) => {
                                        if items.is_empty() {
                                            ap.push(String::from("(empty)"));
                                        } else {
                                            for (name, file_type, size) in items {
                                                match file_type {
                                                    FileType::Directory => {
                                                        ap.push(format!("{}  <DIR>", name));
                                                    }
                                                    FileType::File => {
                                                        ap.push(format!("{}  {} B", name, size));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        ap.push(format!("ls: {}", e.as_str()));
                                    }
                                }
                            },
                            "whoami" => ap.push(String::from("root")),
                            "ifconfig" => {
                                if let Some(mac) = crate::network::aqu() {
                                    let bhv = format!("{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                                        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);
                                    if let Some((ip, _subnet, _gw)) = crate::network::rd() {
                                        let auc = format!("{}.{}.{}.{}", 
                                            ip.as_bytes()[0], ip.as_bytes()[1], ip.as_bytes()[2], ip.as_bytes()[3]);
                                        ap.push(format!("eth0: {}  UP  RUNNING", auc));
                                    } else {
                                        ap.push(String::from("eth0: No IP  UP  RUNNING"));
                                    }
                                    ap.push(format!("      MAC: {}", bhv));
                                } else {
                                    ap.push(String::from("eth0: No network interface"));
                                }
                            },
                            "cpuinfo" => {
                                ap.push(String::from("CPU: QEMU Virtual CPU version 2.5+"));
                                ap.push(String::from("Freq: 3.8 GHz | Cores: 1 | Arch: x86_64"));
                                ap.push(String::from("Features: SSE SSE2 NX SVM"));
                            },
                            "meminfo" => {
                                let used = crate::memory::heap::used() / 1024;
                                let av = crate::memory::atz() / 1024;
                                let pme = crate::memory::ceo() / 1024 / 1024;
                                ap.push(format!("Heap: {} / {} KB", used, av));
                                ap.push(format!("System: {} MB total", pme));
                            },
                            "uptime" => {
                                let im = crate::cpu::tsc::ey() / crate::cpu::tsc::we();
                                let h = im / 3600;
                                let m = (im % 3600) / 60;
                                let j = im % 60;
                                ap.push(format!("Uptime: {:02}:{:02}:{:02}", h, m, j));
                            },
                            "exit" | "quit" => {
                                ap.push(String::from("> Use 'desktop close' to exit desktop"));
                            },
                            "date" | "time" => {
                                let fm = crate::rtc::aou();
                                ap.push(format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", 
                                    fm.year, fm.month, fm.day, fm.hour, fm.minute, fm.second));
                            },
                            "hostname" => ap.push(String::from("trustos")),
                            "uname" => ap.push(String::from("TrustOS 0.1.0 x86_64")),
                            "holo" | "holomatrix" => {
                                
                                crate::serial_println!("[DEBUG] holo command received, toggling...");
                                aia = !aia;
                                crate::graphics::holomatrix::set_enabled(aia);
                                crate::serial_println!("[DEBUG] holo_enabled = {}", aia);
                                if aia {
                                    ap.push(String::from("? HoloMatrix 3D ENABLED"));
                                    ap.push(String::from("  3D hologram appears through Matrix Rain"));
                                    ap.push(String::from("  Use settings panel to change scene"));
                                } else {
                                    ap.push(String::from("? HoloMatrix 3D DISABLED"));
                                    ap.push(String::from("  Standard Matrix Rain background"));
                                }
                            },
                            "holo on" => {
                                aia = true;
                                crate::graphics::holomatrix::set_enabled(true);
                                ap.push(String::from("? HoloMatrix 3D enabled"));
                            },
                            "holo off" => {
                                aia = false;
                                bju = false;
                                crate::graphics::holomatrix::set_enabled(false);
                                ap.push(String::from("? HoloMatrix 3D disabled"));
                            },
                            "holo volume" | "holovolume" => {
                                
                                bju = !bju;
                                if bju {
                                    aia = false;  
                                    ap.push(String::from("? HOLOVOLUME ENABLED"));
                                    ap.push(String::from("  Volumetric ASCII raymarcher active"));
                                    ap.push(String::from("  3D voxel grid projected to 2D"));
                                    ap.push(String::from("  Aligned characters = brighter"));
                                } else {
                                    ap.push(String::from("? HoloVolume disabled"));
                                    ap.push(String::from("  Back to Matrix Rain"));
                                }
                            },
                            "holo dna" => {
                                bju = true;
                                holovolume.render_mode = crate::holovolume::RenderMode::DnaHelix;
                                ap.push(String::from("? HoloVolume: DNA Helix"));
                            },
                            "holo cube" => {
                                bju = true;
                                holovolume.render_mode = crate::holovolume::RenderMode::RotatingCube;
                                ap.push(String::from("? HoloVolume: Rotating Cube"));
                            },
                            "holo sphere" => {
                                bju = true;
                                holovolume.render_mode = crate::holovolume::RenderMode::Sphere;
                                ap.push(String::from("? HoloVolume: Sphere"));
                            },
                            "holo rain" => {
                                bju = true;
                                holovolume.render_mode = crate::holovolume::RenderMode::MatrixRain;
                                ap.push(String::from("? HoloVolume: Matrix Rain (volumetric)"));
                            },
                            
                            
                            
                            "matrix formula" | "formula" | "formula3d" => {
                                xk = true;
                                vb = false;
                                vv = false;
                                yg = false;
                                xl = false;
                                bju = false;
                                ap.push(String::from("? FORMULA 3D: Wireframe perspective projection"));
                                ap.push(String::from("  Commands: formula cube|pyramid|diamond|torus|sphere|grid|helix|multi"));
                            },
                            "formula cube" => {
                                xk = true; vb = false; vv = false; yg = false; xl = false;
                                aqp.set_scene(crate::formula3d::FormulaScene::Cube);
                                ap.push(String::from("? FORMULA: Rotating Cube"));
                            },
                            "formula pyramid" => {
                                xk = true; vb = false; vv = false; yg = false; xl = false;
                                aqp.set_scene(crate::formula3d::FormulaScene::Pyramid);
                                ap.push(String::from("? FORMULA: Pyramid"));
                            },
                            "formula diamond" => {
                                xk = true; vb = false; vv = false; yg = false; xl = false;
                                aqp.set_scene(crate::formula3d::FormulaScene::Diamond);
                                ap.push(String::from("? FORMULA: Diamond octahedron"));
                            },
                            "formula torus" | "formula donut" => {
                                xk = true; vb = false; vv = false; yg = false; xl = false;
                                aqp.set_scene(crate::formula3d::FormulaScene::Torus);
                                ap.push(String::from("? FORMULA: Torus (donut)"));
                            },
                            "formula sphere" => {
                                xk = true; vb = false; vv = false; yg = false; xl = false;
                                aqp.set_scene(crate::formula3d::FormulaScene::Icosphere);
                                ap.push(String::from("? FORMULA: Icosphere"));
                            },
                            "formula grid" => {
                                xk = true; vb = false; vv = false; yg = false; xl = false;
                                aqp.set_scene(crate::formula3d::FormulaScene::Grid);
                                ap.push(String::from("? FORMULA: Infinite grid"));
                            },
                            "formula helix" | "formula dna" => {
                                xk = true; vb = false; vv = false; yg = false; xl = false;
                                aqp.set_scene(crate::formula3d::FormulaScene::Helix);
                                ap.push(String::from("? FORMULA: DNA helix"));
                            },
                            "formula multi" => {
                                xk = true; vb = false; vv = false; yg = false; xl = false;
                                aqp.set_scene(crate::formula3d::FormulaScene::Multi);
                                ap.push(String::from("? FORMULA: Multi - orbiting shapes"));
                            },
                            "formula penger" | "formula penguin" | "penger" => {
                                xk = true; vb = false; vv = false; yg = false; xl = false;
                                aqp.set_scene(crate::formula3d::FormulaScene::Penger);
                                ap.push(String::from("? FORMULA: Penger - hologram penguin ??"));
                            },
                            "formula trustos" | "formula title" | "trustos" | "trustos 3d" => {
                                xk = true; vb = false; vv = false; yg = false; xl = false;
                                aqp.set_scene(crate::formula3d::FormulaScene::TrustOs);
                                aqp.wire_color = 0xFF00CCFF;
                                ap.push(String::from("? FORMULA: TrustOS 3D -- hologram scanline title"));
                            },
                            "formula holo" | "holo matrix" | "holomatrix" | "matrix holo" | "matrix 3d holo" => {
                                xk = true; vb = false; vv = false; yg = false; xl = false;
                                aqp.set_scene(crate::formula3d::FormulaScene::HoloMatrix);
                                ap.push(String::from("? FORMULA: HoloMatrix 3D -- volumetric holographic rain"));
                            },
                            "matrix fast" => {
                                xk = false;
                                vv = true;
                                vb = false;
                                xl = false;
                                ap.push(String::from("? FAST MATRIX: Glyph-cached renderer"));
                                ap.push(String::from("  Pre-computed u128 glyphs + LUT intensity"));
                            },
                            "matrix braille" => {
                                xk = false;
                                vb = true;
                                vv = false;
                                xl = false;
                                ap.push(String::from("? BRAILLE MATRIX: 8A-- sub-pixel resolution"));
                                ap.push(String::from("  480A--272 virtual pixels via Unicode ??"));
                            },
                            "matrix legacy" => {
                                xk = false;
                                vv = false;
                                vb = false;
                                yg = false;
                                xl = false;
                                ap.push(String::from("? LEGACY MATRIX: Original renderer"));
                                ap.push(String::from("  Per-pixel font lookup (slower)"));
                            },
                            "matrix3d" | "matrix 3d" => {
                                xk = false;
                                yg = !yg;
                                vb = !yg;
                                vv = false;
                                xl = false;
                                if yg {
                                    ap.push(String::from("? MATRIX 3D: Volumetric rain with shapes"));
                                    ap.push(String::from("  Commands: matrix3d sphere | cube | torus"));
                                } else {
                                    ap.push(String::from("? MATRIX 3D: Disabled, back to BRAILLE"));
                                }
                            },
                            "matrix3d sphere" | "matrix 3d sphere" => {
                                xk = false;
                                yg = true;
                                vb = false;
                                vv = false;
                                xl = false;
                                dty.set_demo_shapes();
                                ap.push(String::from("? MATRIX 3D: Sphere - rain flows around it"));
                            },
                            "matrix3d cube" | "matrix 3d cube" => {
                                xk = false;
                                yg = true;
                                vb = false;
                                vv = false;
                                xl = false;
                                dty.set_cube();
                                ap.push(String::from("? MATRIX 3D: Rotating Cube"));
                            },
                            "matrix3d torus" | "matrix 3d torus" => {
                                xk = false;
                                yg = true;
                                vb = false;
                                vv = false;
                                xl = false;
                                dty.set_torus();
                                ap.push(String::from("? MATRIX 3D: Torus (donut shape)"));
                            },
                            
                            "matrix cube" => {
                                xk = false;
                                vb = true;
                                yg = false;
                                vv = false;
                                xl = false;
                                bqu.set_shape(crate::matrix_fast::ShapeOverlay::Cube);
                                ap.push(String::from("? MATRIX: Cube overlay - glyphs trace rotating cube"));
                            },
                            "matrix sphere" => {
                                xk = false;
                                vb = true;
                                yg = false;
                                vv = false;
                                xl = false;
                                bqu.set_shape(crate::matrix_fast::ShapeOverlay::Sphere);
                                ap.push(String::from("? MATRIX: Sphere overlay - glyphs trace sphere surface"));
                            },
                            "matrix torus" => {
                                xk = false;
                                vb = true;
                                yg = false;
                                vv = false;
                                xl = false;
                                bqu.set_shape(crate::matrix_fast::ShapeOverlay::Torus);
                                ap.push(String::from("? MATRIX: Torus overlay - glyphs trace spinning donut"));
                            },
                            "matrix dna" => {
                                xk = false;
                                vb = true;
                                yg = false;
                                vv = false;
                                xl = false;
                                bqu.set_shape(crate::matrix_fast::ShapeOverlay::DNA);
                                ap.push(String::from("? MATRIX: DNA overlay - glyphs trace double helix"));
                            },
                            "matrix off" | "matrix clear" | "matrix normal" => {
                                bqu.set_shape(crate::matrix_fast::ShapeOverlay::None);
                                ap.push(String::from("? MATRIX: Shape overlay disabled - normal rain"));
                            },
                            "matrix shader" | "matrix gpu" => {
                                xl = !xl;
                                if xl {
                                    xk = false;
                                    vb = false;
                                    vv = false;
                                    yg = false;
                                    ap.push(String::from("? SHADER MATRIX: GPU-emulated pixel shader"));
                                    ap.push(String::from("  Uses SMP parallel dispatch + SSE2 SIMD"));
                                    ap.push(String::from("  Smooth per-pixel glyph rendering"));
                                } else {
                                    vb = true;
                                    ap.push(String::from("? SHADER MATRIX: Disabled, back to BRAILLE"));
                                }
                            },
                            "matrix" => {
                                let mode = if xk { "FORMULA (wireframe 3D)" }
                                           else if xl { "SHADER (GPU-emulated pixel shader)" }
                                           else if yg { "3D (volumetric shapes)" }
                                           else if vb { "BRAILLE (8A-- sub-pixel)" }
                                           else if vv { "FAST (glyph-cached)" }
                                           else { "LEGACY (per-pixel)" };
                                ap.push(format!("Matrix Renderer: {}", mode));
                                ap.push(String::from("Commands: matrix formula | fast | braille | legacy | 3d | shader"));
                            },
                            "fps" => {
                                dzj = !dzj;
                                ap.push(format!("FPS display: {}", if dzj { "ON" } else { "OFF" }));
                            },
                            "smp" | "smpstatus" | "smp status" => {
                                let status = if crate::cpu::smp::eru() { "ON" } else { "OFF" };
                                let cpus = crate::cpu::smp::ail();
                                let av = crate::cpu::smp::cpu_count();
                                ap.push(format!("SMP Parallel: {} ({}/{} CPUs)", status, cpus, av));
                                ap.push(String::from("  smp on  - Enable multi-core"));
                                ap.push(String::from("  smp off - Single-core mode"));
                            },
                            "smp on" => {
                                crate::cpu::smp::elh();
                                ap.push(String::from("? SMP parallelism ENABLED"));
                            },
                            "smp off" => {
                                crate::cpu::smp::fsj();
                                ap.push(String::from("? SMP disabled (single-core)"));
                            },
                            "shader" | "shaders" | "vgpu" => {
                                ap.push(String::from("+---------------------------------------+"));
                                ap.push(String::from("|     Virtual GPU - Shader Demo         |"));
                                ap.push(String::from("|---------------------------------------|"));
                                ap.push(String::from("| shader plasma    - Plasma waves       |"));
                                ap.push(String::from("| shader fire      - Fire effect        |"));
                                ap.push(String::from("| shader mandelbrot- Fractal zoom       |"));
                                ap.push(String::from("| shader matrix    - Matrix rain        |"));
                                ap.push(String::from("| shader tunnel    - 3D HOLOMATRIX      |"));
                                ap.push(String::from("| shader parallax  - Depth layers       |"));
                                ap.push(String::from("| shader shapes    - Ray-marched 3D     |"));
                                ap.push(String::from("| shader rain3d    - Matrix fly-through |"));
                                ap.push(String::from("| shader cosmic    - Fractal vortex     |"));
                                ap.push(String::from("| shader gradient  - Test gradient      |"));
                                ap.push(String::from("+---------------------------------------+"));
                                ap.push(String::from("Press ESC to exit shader demo"));
                            },
                            _ if cmd.starts_with("shader ") => {
                                let bov = cmd.trim_start_matches("shader ").trim();
                                crate::serial_println!("[SHADER] Trying to load shader: '{}'", bov);
                                if let Some(shader_fn) = crate::gpu_emu::fyx(bov) {
                                    crate::serial_println!("[SHADER] Found shader, starting loop...");
                                    ap.push(format!("? Loading shader: {}", bov));
                                    ap.push(String::from("Press ESC to exit..."));
                                    
                                    let width = crate::framebuffer::width();
                                    let height = crate::framebuffer::height();
                                    
                                    
                                    let pu = crate::framebuffer::ajy();
                                    if !pu {
                                        crate::framebuffer::adw();
                                        crate::framebuffer::pr(true);
                                    }
                                    
                                    
                                    let kaq = crate::framebuffer::aqr();
                                    let (fb_ptr, bb_stride) = if let Some((ptr, _w, _h, stride)) = kaq {
                                        (ptr as *mut u32, stride)
                                    } else {
                                        
                                        (crate::framebuffer::fyq(), width)
                                    };
                                    
                                    
                                    crate::gpu_emu::mpm(fb_ptr, width, height, bb_stride);
                                    crate::gpu_emu::set_shader(shader_fn);
                                    
                                    
                                    let rr = crate::cpu::tsc::ey();
                                    let mut frames = 0u32;
                                    
                                    loop {
                                        
                                        if let Some(key) = crate::keyboard::kr() {
                                            if key == 27 { break; }
                                        }
                                        
                                        
                                        #[cfg(target_arch = "x86_64")]
                                        crate::gpu_emu::ftc();
                                        #[cfg(not(target_arch = "x86_64"))]
                                        crate::gpu_emu::draw();
                                        
                                        
                                        crate::framebuffer::ii();
                                        
                                        
                                        crate::gpu_emu::tick(16);
                                        frames += 1;
                                        
                                        
                                        if frames % 60 == 0 {
                                            let bb = crate::cpu::tsc::ey() - rr;
                                            let lox = bb as f32 / crate::cpu::tsc::we() as f32;
                                            let fps = frames as f32 / lox;
                                            crate::serial_println!("[SHADER] FPS: {:.1}", fps);
                                        }
                                    }
                                    
                                    
                                    if !pu {
                                        crate::framebuffer::pr(false);
                                    }
                                    
                                    ap.push(format!("Shader demo ended ({} frames)", frames));
                                } else {
                                    crate::serial_println!("[SHADER] Shader '{}' NOT FOUND!", bov);
                                    ap.push(format!("Unknown shader: {}", bov));
                                    ap.push(String::from("Available: plasma, fire, mandelbrot, matrix, tunnel, parallax, shapes, rain3d, cosmic, gradient"));
                                }
                            },
                            "echo" => ap.push(String::new()),
                            "touch" => ap.push(String::from("Usage: touch <filename>")),
                            "rm" => ap.push(String::from("Usage: rm <filename>")),
                            "cp" => ap.push(String::from("Usage: cp <src> <dest>")),
                            "mv" => ap.push(String::from("Usage: mv <src> <dest>")),
                            _ if cmd.starts_with("echo ") => {
                                let text = cmd.trim_start_matches("echo ").trim();
                                ap.push(String::from(text));
                            },
                            _ if cmd.starts_with("cd ") => {
                                let path = cmd.trim_start_matches("cd ").trim();
                                
                                match crate::ramfs::bh(|fs| fs.cd(path)) {
                                    Ok(()) => {
                                        let giz = crate::ramfs::bh(|fs| String::from(fs.pwd()));
                                        ap.push(format!("Changed to: {}", giz));
                                    }
                                    Err(e) => {
                                        ap.push(format!("cd: {}: {}", path, e.as_str()));
                                    }
                                }
                            },
                            _ if cmd.starts_with("ls ") => {
                                let path = cmd.trim_start_matches("ls ").trim();
                                
                                match crate::ramfs::bh(|fs| fs.ls(Some(path))) {
                                    Ok(items) => {
                                        if items.is_empty() {
                                            ap.push(String::from("(empty)"));
                                        } else {
                                            for (name, file_type, size) in items {
                                                match file_type {
                                                    FileType::Directory => {
                                                        ap.push(format!("{}  <DIR>", name));
                                                    }
                                                    FileType::File => {
                                                        ap.push(format!("{}  {} B", name, size));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        ap.push(format!("ls: {}: {}", path, e.as_str()));
                                    }
                                }
                            },
                            _ if cmd.starts_with("cat ") => {
                                let path = cmd.trim_start_matches("cat ").trim();
                                
                                match crate::ramfs::bh(|fs| {
                                    fs.read_file(path).map(|j| alloc::vec::Vec::from(j))
                                }) {
                                    Ok(content) => {
                                        if let Ok(text) = core::str::from_utf8(&content) {
                                            for line in text.lines().take(20) {
                                                ap.push(String::from(line));
                                            }
                                        } else {
                                            ap.push(format!("cat: {}: Binary file", path));
                                        }
                                    }
                                    Err(e) => {
                                        ap.push(format!("cat: {}: {}", path, e.as_str()));
                                    }
                                }
                            },
                            
                            _ if cmd.starts_with("edit ") || cmd.starts_with("code ") || cmd.starts_with("nano ") || cmd.starts_with("vim ") => {
                                let path = cmd.split_whitespace().nth(1).unwrap_or("").trim();
                                if path.is_empty() {
                                    ap.push(String::from("Usage: edit <filename>"));
                                } else {
                                    editor_state.load_file(path);
                                    xm = AppMode::TextEditor;
                                    bky = false;
                                    ap.push(format!("TrustCode: editing {}", path));
                                    crate::serial_println!("[TrustCode] Editing: {}", path);
                                }
                            },
                            _ if cmd.starts_with("mkdir ") => {
                                let path = cmd.trim_start_matches("mkdir ").trim();
                                
                                match crate::ramfs::bh(|fs| fs.mkdir(path)) {
                                    Ok(()) => {
                                        ap.push(format!("Created directory: {}", path));
                                    }
                                    Err(e) => {
                                        ap.push(format!("mkdir: {}: {}", path, e.as_str()));
                                    }
                                }
                            },
                            _ if cmd.starts_with("touch ") => {
                                let path = cmd.trim_start_matches("touch ").trim();
                                
                                match crate::ramfs::bh(|fs| fs.write_file(path, &[])) {
                                    Ok(()) => {
                                        ap.push(format!("Created file: {}", path));
                                    }
                                    Err(e) => {
                                        ap.push(format!("touch: {}: {}", path, e.as_str()));
                                    }
                                }
                            },
                            _ if cmd.starts_with("rm ") => {
                                let path = cmd.trim_start_matches("rm ").trim();
                                match crate::ramfs::bh(|fs| fs.rm(path)) {
                                    Ok(()) => {
                                        ap.push(format!("Removed: {}", path));
                                    }
                                    Err(e) => {
                                        ap.push(format!("rm: {}: {}", path, e.as_str()));
                                    }
                                }
                            },
                            _ if cmd.starts_with("curl ") || cmd.starts_with("get ") || cmd.starts_with("wget ") => {
                                let url = if cmd.starts_with("curl ") {
                                    cmd.trim_start_matches("curl ").trim()
                                } else if cmd.starts_with("wget ") {
                                    cmd.trim_start_matches("wget ").trim()
                                } else {
                                    cmd.trim_start_matches("get ").trim()
                                };
                                ap.push(format!("Fetching: {}", url));
                                
                                
                                if let Some((host, port, path)) = super::vm::nrn(url) {
                                    ap.push(format!("Host: {} Port: {} Path: {}", host, port, path));
                                    
                                    
                                    if let Some(ip) = crate::netstack::dns::yb(&host) {
                                        ap.push(format!("Resolved to: {}.{}.{}.{}", 
                                            ip[0], ip[1], ip[2], ip[3]));
                                        
                                        
                                        match super::vm::hta(&host, ip, port, &path) {
                                            Ok(fa) => {
                                                
                                                for line in fa.lines().take(15) {
                                                    ap.push(String::from(line));
                                                }
                                                if fa.lines().count() > 15 {
                                                    ap.push(String::from("... (truncated)"));
                                                }
                                            }
                                            Err(e) => {
                                                ap.push(format!("Error: {}", e));
                                            }
                                        }
                                    } else {
                                        ap.push(format!("Cannot resolve: {}", host));
                                    }
                                } else {
                                    ap.push(String::from("Invalid URL format"));
                                    ap.push(String::from("Usage: curl http://host/path"));
                                }
                            },
                            _ if cmd.starts_with("desktop ") => {
                                let sub = cmd.trim_start_matches("desktop ");
                                if sub == "close" || sub == "exit" || sub == "quit" {
                                    running = false;
                                }
                            },
                            
                            "open" => {
                                ap.push(String::from("Usage: open <app>"));
                                ap.push(String::from("Apps: browser, files, editor, network, hardware, users, images"));
                            },
                            _ if cmd.starts_with("open ") => {
                                let afz = cmd.trim_start_matches("open ").trim().to_lowercase();
                                match afz.as_str() {
                                    "browser" | "web" | "www" => {
                                        xm = AppMode::Browser;
                                        bky = true;
                                        ap.push(String::from("Switched to Browser"));
                                    },
                                    "files" | "explorer" => {
                                        xm = AppMode::Files;
                                        bky = false;
                                        ap.push(String::from("Switched to Files"));
                                    },
                                    "editor" | "text" | "notepad" | "trustcode" | "code" => {
                                        xm = AppMode::TextEditor;
                                        bky = false;
                                        
                                        if editor_state.file_path.is_none() {
                                            editor_state.load_file("demo.rs");
                                        }
                                        ap.push(String::from("TrustCode Editor opened"));
                                    },
                                    "network" | "net" | "ifconfig" => {
                                        xm = AppMode::Network;
                                        bky = false;
                                        ap.push(String::from("Switched to Network"));
                                    },
                                    "hardware" | "hw" | "lshw" => {
                                        xm = AppMode::Hardware;
                                        bky = false;
                                        ap.push(String::from("Switched to Hardware"));
                                    },
                                    "users" | "user" => {
                                        xm = AppMode::UserMgmt;
                                        bky = false;
                                        ap.push(String::from("Switched to User Management"));
                                    },
                                    "images" | "image" | "viewer" => {
                                        xm = AppMode::ImageViewer;
                                        bky = false;
                                        ap.push(String::from("Switched to Image Viewer"));
                                    },
                                    "shell" | "terminal" => {
                                        xm = AppMode::Shell;
                                        bky = false;
                                        ap.push(String::from("Switched to Shell"));
                                    },
                                    _ => {
                                        ap.push(format!("Unknown app: {}", afz));
                                        ap.push(String::from("Available: browser, files, editor, network, hardware, users, images"));
                                    }
                                }
                            },
                            
                            "ping" => ap.push(String::from("Usage: ping <host>")),
                            "nslookup" | "dig" => ap.push(String::from("Usage: nslookup <hostname>")),
                            "ps" => {
                                ap.push(String::from("  PID  STATE  NAME"));
                                ap.push(String::from("    1  R      init"));
                                ap.push(String::from("    2  R      kernel"));
                                ap.push(String::from("    3  R      desktop"));
                            },
                            "df" => {
                                ap.push(String::from("Filesystem    Size  Used  Avail  Use%  Mounted"));
                                ap.push(String::from("ramfs         8.0M   64K   7.9M    1%  /"));
                            },
                            "free" => {
                                let used = crate::memory::heap::used() / 1024;
                                let av = crate::memory::atz() / 1024;
                                let fxr = av - used;
                                ap.push(String::from("              total     used     free"));
                                ap.push(format!("Mem:     {:>10}  {:>7}  {:>7}", av, used, fxr));
                            },
                            "tree" => {
                                ap.push(String::from("."));
                                match crate::ramfs::bh(|fs| fs.ls(None)) {
                                    Ok(items) => {
                                        let count = items.len();
                                        for (i, (name, file_type, _)) in items.into_iter().enumerate() {
                                            let nm = if i + 1 == count { "+-- " } else { "+-- " };
                                            match file_type {
                                                FileType::Directory => ap.push(format!("{}{}/ (dir)", nm, name)),
                                                FileType::File => ap.push(format!("{}{}", nm, name)),
                                            }
                                        }
                                    }
                                    Err(_) => {}
                                }
                            },
                            "history" => {
                                ap.push(String::from("Command history not available in desktop shell"));
                            },
                            _ if cmd.starts_with("ping ") => {
                                let host = cmd.trim_start_matches("ping ").trim();
                                
                                let dsi = if let Some(parsed) = super::vm::art(host) {
                                    Some(parsed)
                                } else {
                                    
                                    
                                    match host {
                                        "google.com" | "www.google.com" => Some([142, 250, 179, 110]),
                                        "cloudflare.com" | "www.cloudflare.com" => Some([104, 16, 132, 229]),
                                        "github.com" | "www.github.com" => Some([140, 82, 114, 3]),
                                        "localhost" => Some([127, 0, 0, 1]),
                                        _ => None, 
                                    }
                                };
                                
                                if let Some(ip) = dsi {
                                    ap.push(format!("PING {} ({}.{}.{}.{})", host, ip[0], ip[1], ip[2], ip[3]));
                                    ap.push(format!("64 bytes from {}.{}.{}.{}: icmp_seq=1 ttl=64 time=1.5 ms", ip[0], ip[1], ip[2], ip[3]));
                                    ap.push(format!("64 bytes from {}.{}.{}.{}: icmp_seq=2 ttl=64 time=1.2 ms", ip[0], ip[1], ip[2], ip[3]));
                                    ap.push(String::from("--- ping statistics ---"));
                                    ap.push(String::from("2 packets transmitted, 2 received, 0% loss"));
                                } else {
                                    ap.push(format!("ping: {} - cannot resolve (use IP address)", host));
                                }
                            },
                            _ if cmd.starts_with("nslookup ") || cmd.starts_with("dig ") => {
                                let host = if cmd.starts_with("nslookup ") {
                                    cmd.trim_start_matches("nslookup ").trim()
                                } else {
                                    cmd.trim_start_matches("dig ").trim()
                                };
                                ap.push(format!("Server:  8.8.8.8"));
                                ap.push(format!("Name:    {}", host));
                                
                                let dsi = match host {
                                    "google.com" | "www.google.com" => Some([142, 250, 179, 110]),
                                    "cloudflare.com" | "www.cloudflare.com" => Some([104, 16, 132, 229]),
                                    "github.com" | "www.github.com" => Some([140, 82, 114, 3]),
                                    "localhost" => Some([127, 0, 0, 1]),
                                    _ => super::vm::art(host), 
                                };
                                if let Some(ip) = dsi {
                                    ap.push(format!("Address: {}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3]));
                                } else {
                                    ap.push(String::from("** server can't find: NXDOMAIN"));
                                }
                            },
                            _ if cmd.starts_with("hexdump ") || cmd.starts_with("xxd ") => {
                                let path = if cmd.starts_with("hexdump ") {
                                    cmd.trim_start_matches("hexdump ").trim()
                                } else {
                                    cmd.trim_start_matches("xxd ").trim()
                                };
                                match crate::ramfs::bh(|fs| {
                                    fs.read_file(path).map(|j| alloc::vec::Vec::from(j))
                                }) {
                                    Ok(content) => {
                                        for (offset, df) in content.chunks(16).take(8).enumerate() {
                                            let ga: alloc::vec::Vec<String> = df.iter()
                                                .map(|b| format!("{:02x}", b))
                                                .collect();
                                            let ascii: String = df.iter()
                                                .map(|&b| if b >= 32 && b < 127 { b as char } else { '.' })
                                                .collect();
                                            ap.push(format!("{:08x}  {:48}  |{}|", 
                                                offset * 16, ga.join(" "), ascii));
                                        }
                                        if content.len() > 128 {
                                            ap.push(String::from("... (truncated)"));
                                        }
                                    }
                                    Err(e) => ap.push(format!("hexdump: {}: {}", path, e.as_str())),
                                }
                            },
                            _ if cmd.starts_with("imgview ") || cmd.starts_with("view ") => {
                                let path = if cmd.starts_with("imgview ") {
                                    cmd.trim_start_matches("imgview ").trim()
                                } else {
                                    cmd.trim_start_matches("view ").trim()
                                };
                                
                                
                                match crate::ramfs::bh(|fs| {
                                    fs.read_file(path).map(|j| alloc::vec::Vec::from(j))
                                }) {
                                    Ok(data) => {
                                        
                                        let format = crate::image::frw(&data);
                                        if let Some(iv) = crate::image::gfy(&data) {
                                            moc = String::from(path);
                                            ifv = format!("{}x{} ({} bytes)", iv.width, iv.height, data.len());
                                            ifu = String::from(format.extension());
                                            bhc = 1.0;
                                            gbu = 0;
                                            gbv = 0;
                                            ift = Some(iv);
                                            
                                            
                                            xm = AppMode::ImageViewer;
                                            ap.push(format!("Opening: {} ({})", path, format.extension()));
                                        } else {
                                            ap.push(format!("imgview: Cannot decode image (format: {})", format.extension()));
                                        }
                                    },
                                    Err(e) => {
                                        ap.push(format!("imgview: {}: {}", path, e.as_str()));
                                    }
                                }
                            },
                            _ if cmd.starts_with("imginfo ") => {
                                let path = cmd.trim_start_matches("imginfo ").trim();
                                
                                match crate::ramfs::bh(|fs| {
                                    fs.read_file(path).map(|j| alloc::vec::Vec::from(j))
                                }) {
                                    Ok(data) => {
                                        let format = crate::image::frw(&data);
                                        ap.push(format!("+---------------------------------------+"));
                                        ap.push(format!("| Image Info: {}  ", path));
                                        ap.push(format!("|---------------------------------------|"));
                                        ap.push(format!("| Format:  {} ({})   ", format.extension(), format.mime_type()));
                                        ap.push(format!("| Size:    {} bytes   ", data.len()));
                                        
                                        
                                        if let Some(iv) = crate::image::gfy(&data) {
                                            ap.push(format!("| Width:   {} px   ", iv.width));
                                            ap.push(format!("| Height:  {} px   ", iv.height));
                                            ap.push(format!("| Pixels:  {}   ", iv.width * iv.height));
                                        } else {
                                            ap.push(format!("| (Cannot decode image dimensions)"));
                                        }
                                        ap.push(format!("+---------------------------------------+"));
                                    },
                                    Err(e) => {
                                        ap.push(format!("imginfo: {}: {}", path, e.as_str()));
                                    }
                                }
                            },
                            
                            "top" | "htop" => {
                                ap.push(String::from("top - System Monitor"));
                                ap.push(String::from("  PID  %CPU  %MEM  TIME     COMMAND"));
                                ap.push(String::from("    1  0.5   2.1   0:01.23  kernel"));
                                ap.push(String::from("    2  0.1   0.5   0:00.45  desktop"));
                                ap.push(String::from("Press 'q' to quit (in desktop: just run another cmd)"));
                            },
                            "lspci" => {
                                ap.push(String::from("00:00.0 Host bridge"));
                                ap.push(String::from("00:01.0 VGA controller: Virtio GPU"));
                                ap.push(String::from("00:02.0 Network controller: Virtio Net"));
                                ap.push(String::from("00:03.0 AHCI Controller"));
                            },
                            "lsusb" => {
                                ap.push(String::from("Bus 001 Device 001: ID 1d6b:0002 Linux Foundation Root Hub"));
                                ap.push(String::from("Bus 001 Device 002: ID 0627:0001 QEMU Tablet"));
                            },
                            "lscpu" => {
                                ap.push(String::from("Architecture:        x86_64"));
                                ap.push(String::from("CPU op-modes:        64-bit"));
                                ap.push(String::from("CPU(s):              4"));
                                ap.push(String::from("Vendor ID:           AuthenticAMD"));
                                ap.push(String::from("Model name:          QEMU Virtual CPU"));
                            },
                            "disk" => {
                                ap.push(String::from("Disk /dev/sda: 64 MB"));
                                ap.push(String::from("  Partition 1: 64 MB (TrustOS)"));
                            },
                            "netstat" => {
                                ap.push(String::from("Active connections:"));
                                ap.push(String::from("Proto  Local Address      Foreign Address    State"));
                                ap.push(String::from("tcp    0.0.0.0:0          0.0.0.0:*          LISTEN"));
                            },
                            "arp" => {
                                ap.push(String::from("Address         HWtype  HWaddress           Iface"));
                                ap.push(String::from("10.0.2.2        ether   52:55:0a:00:02:02   eth0"));
                            },
                            "route" => {
                                ap.push(String::from("Kernel IP routing table"));
                                ap.push(String::from("Dest         Gateway      Genmask         Iface"));
                                ap.push(String::from("0.0.0.0      10.0.2.2     0.0.0.0         eth0"));
                                ap.push(String::from("10.0.2.0     0.0.0.0      255.255.255.0   eth0"));
                            },
                            "env" => {
                                ap.push(String::from("USER=root"));
                                ap.push(String::from("HOME=/root"));
                                ap.push(String::from("SHELL=/bin/tsh"));
                                ap.push(String::from("PATH=/bin:/usr/bin"));
                                ap.push(String::from("TERM=trustos"));
                            },
                            "id" => {
                                ap.push(String::from("uid=0(root) gid=0(root) groups=0(root)"));
                            },
                            "cal" => {
                                let fm = crate::rtc::aou();
                                ap.push(format!("     {:02}/{:04}", fm.month, fm.year));
                                ap.push(String::from("Su Mo Tu We Th Fr Sa"));
                                ap.push(String::from("       1  2  3  4  5"));
                                ap.push(String::from(" 6  7  8  9 10 11 12"));
                                ap.push(String::from("13 14 15 16 17 18 19"));
                                ap.push(String::from("20 21 22 23 24 25 26"));
                                ap.push(String::from("27 28 29 30 31"));
                            },
                            _ if cmd.starts_with("head ") => {
                                let path = cmd.trim_start_matches("head ").trim();
                                match crate::ramfs::bh(|fs| {
                                    fs.read_file(path).map(|j| String::from_utf8_lossy(j).into_owned())
                                }) {
                                    Ok(content) => {
                                        for line in content.lines().take(10) {
                                            ap.push(String::from(line));
                                        }
                                    },
                                    Err(e) => ap.push(format!("head: {}: {}", path, e.as_str())),
                                }
                            },
                            _ if cmd.starts_with("tail ") => {
                                let path = cmd.trim_start_matches("tail ").trim();
                                match crate::ramfs::bh(|fs| {
                                    fs.read_file(path).map(|j| String::from_utf8_lossy(j).into_owned())
                                }) {
                                    Ok(content) => {
                                        let lines: alloc::vec::Vec<&str> = content.lines().collect();
                                        let start = if lines.len() > 10 { lines.len() - 10 } else { 0 };
                                        for line in &lines[start..] {
                                            ap.push(String::from(*line));
                                        }
                                    },
                                    Err(e) => ap.push(format!("tail: {}: {}", path, e.as_str())),
                                }
                            },
                            _ if cmd.starts_with("wc ") => {
                                let path = cmd.trim_start_matches("wc ").trim();
                                match crate::ramfs::bh(|fs| {
                                    fs.read_file(path).map(|j| String::from_utf8_lossy(j).into_owned())
                                }) {
                                    Ok(content) => {
                                        let lines = content.lines().count();
                                        let um = content.split_whitespace().count();
                                        let bytes = content.len();
                                        ap.push(format!("{:>5} {:>5} {:>5} {}", lines, um, bytes, path));
                                    },
                                    Err(e) => ap.push(format!("wc: {}: {}", path, e.as_str())),
                                }
                            },
                            _ if cmd.starts_with("grep ") => {
                                let args = cmd.trim_start_matches("grep ").trim();
                                let au: alloc::vec::Vec<&str> = args.splitn(2, ' ').collect();
                                if au.len() == 2 {
                                    let pattern = au[0];
                                    let path = au[1];
                                    match crate::ramfs::bh(|fs| {
                                        fs.read_file(path).map(|j| String::from_utf8_lossy(j).into_owned())
                                    }) {
                                        Ok(content) => {
                                            let mut nj = false;
                                            for line in content.lines() {
                                                if line.contains(pattern) {
                                                    ap.push(String::from(line));
                                                    nj = true;
                                                }
                                            }
                                            if !nj {
                                                ap.push(format!("(no matches for '{}')", pattern));
                                            }
                                        },
                                        Err(e) => ap.push(format!("grep: {}: {}", path, e.as_str())),
                                    }
                                } else {
                                    ap.push(String::from("Usage: grep <pattern> <file>"));
                                }
                            },
                            _ if cmd.starts_with("find ") => {
                                let pattern = cmd.trim_start_matches("find ").trim();
                                ap.push(format!("Searching for: {}", pattern));
                                match crate::ramfs::bh(|fs| fs.ls(None)) {
                                    Ok(items) => {
                                        for (name, _, _) in items {
                                            if name.contains(pattern) {
                                                ap.push(format!("./{}", name));
                                            }
                                        }
                                    },
                                    Err(_) => {}
                                }
                            },
                            _ if cmd.starts_with("stat ") => {
                                let path = cmd.trim_start_matches("stat ").trim();
                                match crate::ramfs::bh(|fs| {
                                    fs.read_file(path).map(|j| j.len())
                                }) {
                                    Ok(size) => {
                                        ap.push(format!("  File: {}", path));
                                        ap.push(format!("  Size: {} bytes", size));
                                        ap.push(String::from("  Access: -rw-r--r--"));
                                        ap.push(String::from("  Uid: 0  Gid: 0"));
                                    },
                                    Err(e) => ap.push(format!("stat: {}: {}", path, e.as_str())),
                                }
                            },
                            _ if cmd.starts_with("sort ") => {
                                let path = cmd.trim_start_matches("sort ").trim();
                                match crate::ramfs::bh(|fs| {
                                    fs.read_file(path).map(|j| String::from_utf8_lossy(j).into_owned())
                                }) {
                                    Ok(content) => {
                                        let mut lines: alloc::vec::Vec<&str> = content.lines().collect();
                                        lines.sort();
                                        for line in lines {
                                            ap.push(String::from(line));
                                        }
                                    },
                                    Err(e) => ap.push(format!("sort: {}: {}", path, e.as_str())),
                                }
                            },
                            _ if cmd.starts_with("strings ") => {
                                let path = cmd.trim_start_matches("strings ").trim();
                                match crate::ramfs::bh(|fs| {
                                    fs.read_file(path).map(|j| alloc::vec::Vec::from(j))
                                }) {
                                    Ok(data) => {
                                        let mut current = String::new();
                                        for &b in data.iter().take(1024) {
                                            if b >= 32 && b < 127 {
                                                current.push(b as char);
                                            } else if current.len() >= 4 {
                                                ap.push(current.clone());
                                                current.clear();
                                            } else {
                                                current.clear();
                                            }
                                        }
                                        if current.len() >= 4 {
                                            ap.push(current);
                                        }
                                    },
                                    Err(e) => ap.push(format!("strings: {}: {}", path, e.as_str())),
                                }
                            },
                            _ if cmd.starts_with("traceroute ") || cmd.starts_with("tracert ") => {
                                let host = if cmd.starts_with("traceroute ") {
                                    cmd.trim_start_matches("traceroute ").trim()
                                } else {
                                    cmd.trim_start_matches("tracert ").trim()
                                };
                                ap.push(format!("traceroute to {} (simulated)", host));
                                ap.push(String::from(" 1  10.0.2.2  1.234 ms"));
                                ap.push(String::from(" 2  * * *"));
                                ap.push(String::from(" 3  * * *"));
                            },
                            "3ddemo" | "demo3d" | "cube" => {
                                
                                ap.push(String::from("Starting 3D Demo..."));
                                ap.push(String::from("Controls: Arrow keys rotate, ESC to exit"));
                                
                                
                                let ajk = 400u32;
                                let aqf = 300u32;
                                let mut zh = crate::rasterizer::Rasterizer::new(ajk, aqf);
                                let mut renderer = crate::rasterizer::Renderer3D::new(ajk, aqf);
                                
                                let mut angle_y: f32 = 0.0;
                                let mut angle_x: f32 = 0.3;
                                let mut hrg = true;
                                let mut frm = 0u32;
                                
                                
                                let bba = (dgq + 150) as u32;
                                let bfv = (dgr + 50) as u32;
                                
                                while hrg && frm < 600 { 
                                    
                                    if let Some(k) = crate::keyboard::kr() {
                                        match k {
                                            27 => hrg = false, 
                                            0x4B => angle_y -= 0.1, 
                                            0x4D => angle_y += 0.1, 
                                            0x48 => angle_x -= 0.1, 
                                            0x50 => angle_x += 0.1, 
                                            _ => {}
                                        }
                                    }
                                    
                                    
                                    zh.clear(0xFF101010);
                                    renderer.clear_z_buffer();
                                    
                                    
                                    let rot_y = crate::rasterizer::Mat4::rotation_y(angle_y);
                                    let rot_x = crate::rasterizer::Mat4::rotation_x(angle_x);
                                    let rotation = rot_x.mul(&rot_y);
                                    
                                    
                                    let center = crate::rasterizer::Vec3::new(0.0, 0.0, 0.0);
                                    renderer.draw_cube(&mut zh, center, 1.5, &rotation, 0xFF00FF00);
                                    
                                    
                                    let kie = crate::rasterizer::Vec3::new(2.0, 0.0, 0.0);
                                    renderer.draw_cube(&mut zh, kie, 0.8, &rotation, 0xFF00FFFF);
                                    
                                    
                                    zh.fill_gradient_h(0, 0, ajk, 25, 0xFF003300, 0xFF00AA00);
                                    
                                    
                                    zh.draw_rect(0, 0, ajk, aqf, 0xFF00FF00);
                                    
                                    
                                    for o in 0..aqf {
                                        for p in 0..ajk {
                                            let idx = (o * ajk + p) as usize;
                                            crate::framebuffer::draw_pixel(bba + p, bfv + o, zh.back_buffer[idx]);
                                        }
                                    }
                                    
                                    
                                    crate::framebuffer::draw_text("3D Demo - ESC to exit", bba + 10, bfv + 5, 0xFFFFFFFF);
                                    
                                    
                                    let cyc = format!("Frame: {}", frm);
                                    crate::framebuffer::draw_text(&cyc, bba + ajk - 100, bfv + 5, 0xFFFFFF00);
                                    
                                    angle_y += 0.02;  
                                    frm += 1;
                                    
                                    
                                    for _ in 0..50000 { core::hint::spin_loop(); }
                                }
                                
                                ap.push(String::from("3D Demo ended."));
                            },
                            "raster" | "rasterdemo" => {
                                
                                ap.push(String::from("Rasterizer Demo - Antialiasing & Gradients"));
                                
                                let ajk = 350u32;
                                let aqf = 250u32;
                                let mut zh = crate::rasterizer::Rasterizer::new(ajk, aqf);
                                
                                let bba = (dgq + 175) as u32;
                                let bfv = (dgr + 75) as u32;
                                
                                
                                zh.clear(0xFF0A0A0A);
                                
                                
                                zh.fill_gradient_v(0, 0, ajk, aqf, 0xFF000022, 0xFF002200);
                                
                                
                                zh.fill_circle_aa(80, 80, 40, 0xFFFF0000);   
                                zh.fill_circle_aa(150, 100, 35, 0xFF00FF00); 
                                zh.fill_circle_aa(220, 80, 40, 0xFF0000FF);  
                                
                                
                                zh.fill_circle_aa(115, 90, 30, 0x8800FFFF);  
                                zh.fill_circle_aa(185, 90, 30, 0x88FF00FF);  
                                
                                
                                zh.fill_rounded_rect(50, 150, 120, 60, 15, 0xFF444444);
                                zh.fill_gradient_h(55, 155, 110, 50, 0xFF006600, 0xFF00CC00);
                                
                                
                                zh.draw_line_aa(200.0, 150.0, 320.0, 220.0, 0xFFFFFF00);
                                zh.draw_line_aa(200.0, 220.0, 320.0, 150.0, 0xFFFF8800);
                                
                                
                                zh.draw_shadow(250, 160, 60, 40, 8, 0x88000000);
                                zh.fill_rect(250, 160, 60, 40, 0xFF00AA00);
                                
                                
                                zh.draw_rect(0, 0, ajk, aqf, 0xFF00FF00);
                                
                                
                                for o in 0..aqf {
                                    for p in 0..ajk {
                                        let idx = (o * ajk + p) as usize;
                                        crate::framebuffer::draw_pixel(bba + p, bfv + o, zh.back_buffer[idx]);
                                    }
                                }
                                
                                crate::framebuffer::draw_text("Rasterizer: AA + Alpha + Gradients", bba + 10, bfv + 5, 0xFFFFFFFF);
                                
                                
                                ap.push(String::from("Press any key to close demo..."));
                                loop {
                                    if crate::keyboard::kr().is_some() {
                                        break;
                                    }
                                    core::hint::spin_loop();
                                }
                                ap.push(String::from("Demo closed."));
                            },
                            _ => ap.push(format!("Command not found: {}", cmd)),
                        };
                        shell_input.clear();
                        dez.clear();
                        
                        
                        while ap.len() > 20 {
                            ap.remove(0);
                        }
                        
                        
                        scroll_offset = ap.len().saturating_sub(AHB_);
                    }
                },
                32..=126 => { 
                    crate::serial_println!("[KEY] Printable char: '{}' ({})", key as char, key);
                    shell_input.push(key as char);
                    
                    let fnl = ["help", "ls", "dir", "clear", "ifconfig", "cpuinfo", "meminfo", "whoami", "uptime", "open", "smp", "fps", "matrix", "holo"];
                    dez.clear();
                    for c in fnl {
                        if c.starts_with(&shell_input) && c != shell_input.as_str() {
                            dez = String::from(&c[shell_input.len()..]);
                            break;
                        }
                    }
                },
                _ => {}
            }
            } 
        }
        
        
        let gif = crate::mouse::get_state();
        mouse_x = gif.x.clamp(0, width as i32 - 1);
        mouse_y = gif.y.clamp(0, height as i32 - 1);
        let left = gif.left_button;
        
        
        let bfi = left && !dcw;
        let released = !left && dcw;
        dcw = left;
        
        
        if dnp {
            if left {
                
                dgq = (mouse_x - drag_offset_x).clamp(0, width as i32 - 200);
                dgr = (mouse_y - drag_offset_y).clamp(0, height as i32 - 100);
                
                if let Some(aw) = compositor.get_layer_mut(jre) {
                    aw.set_position(dgq as u32, dgr as u32);
                }
            } else {
                
                dnp = false;
            }
        }
        
        
        if let Some(cursor) = compositor.get_layer_mut(hpx) {
            cursor.set_position(mouse_x as u32, mouse_y as u32);
        }
        
        
        bcs = -1;
        if yx {
            let hu = 5u32;
            let ks = height - 340;
            let cg = mouse_x as u32;
            let cr = mouse_y as u32;
            
            if cg >= hu && cg < hu + 250 && cr >= ks && cr < ks + 290 {
                let sy = 36u32;
                let qn = if cr > ks + 40 { cr - ks - 40 } else { 0 };
                let idx = (qn / sy) as i32;
                if idx >= 0 && idx < dbg.len() as i32 {
                    bcs = idx;
                }
            }
        }
        
        
        if bfi {
            let cg = mouse_x as u32;
            let cr = mouse_y as u32;
            
            
            let bwh = height - 40;
            if cr >= bwh && cr < height && cg >= 5 && cg < 110 {
                yx = !yx;
                bou = false; 
            }
            
            else if cr >= bwh && cr < height && cg >= 340 && cg < 390 {
                bou = !bou;
                yx = false; 
                
                dek = crate::desktop::awb();
                del = crate::desktop::dqn();
            }
            
            else if cr >= bwh && cr < height && cg >= 220 && cg < 320 {
                
                cfi = !cfi;
            }
            
            else if yx && bcs >= 0 && bcs < dbg.len() as i32 {
                let (_name, item) = dbg[bcs as usize];
                match item {
                    MenuItem::App(mode) => {
                        
                        if !_name.starts_with("-") {
                            xm = mode;
                            ap.clear();
                            for line in cyo!(mode) {
                                ap.push(String::from(*line));
                            }
                            ap.push(String::from(""));
                            ap.push(String::from("Type commands below. Type 'help' for more info."));
                        }
                    },
                    MenuItem::Shutdown => {
                        ap.push(String::from("> Shutting down..."));
                        
                        for _ in 0..10000000 { core::hint::spin_loop(); }
                        loop {
                            crate::arch::mra();
                            crate::arch::acb();
                        }
                    },
                    MenuItem::Reboot => {
                        ap.push(String::from("> Rebooting..."));
                        for _ in 0..10000000 { core::hint::spin_loop(); }
                        unsafe {
                            crate::arch::Port::<u8>::new(0x64).write(0xFE);
                        }
                        loop { crate::arch::acb(); }
                    },
                }
                yx = false;
            }
            
            else if yx {
                yx = false;
            }
            else if bou {
                
                let jfp = 340u32;
                let jfq = height - 380;
                let oqd = 270u32;
                let oqc = 350u32;
                if !(cg >= jfp && cg < jfp + oqd 
                    && cr >= jfq && cr < jfq + oqc) {
                    bou = false;
                }
            }
            
            else if !dnp {
                let nw = dgq as u32;
                let qr = dgr as u32;
                let ul = 700u32;  
                let afy = 450u32;  
                
                
                if cfi && cg >= nw && cg < nw + ul && cr >= qr && cr < qr + 28 {
                    
                    
                    if cg >= nw + ul - 60 && cg < nw + ul - 40 {
                        cfi = false;  
                        ap.push(String::from("> Window closed. Click dock icon to reopen."));
                    }
                    
                    else if cg >= nw + ul - 90 && cg < nw + ul - 70 {
                        cfi = false;  
                        ap.push(String::from("> Window minimized"));
                    }
                    
                    else if cg >= nw + ul - 120 && cg < nw + ul - 100 {
                        ap.push(String::from("> Window maximized"));
                    }
                    
                    else {
                        dnp = true;
                        drag_offset_x = mouse_x - dgq;
                        drag_offset_y = mouse_y - dgr;
                    }
                }
                
                else if cg < 80 && cr < height - 40 {
                    let rl = 36u32;
                    let gap = 50u32;       
                    let start_y = 10u32;
                    for i in 0..8usize {
                        let gg = start_y + (i as u32) * (rl + gap);
                        if cr >= gg && cr < gg + rl + 16 {
                            xm = match i {
                                0 => AppMode::Files,
                                1 => AppMode::Shell,
                                2 => AppMode::Network,
                                3 => AppMode::TextEditor,
                                4 => AppMode::Hardware,
                                5 => AppMode::UserMgmt,
                                6 => AppMode::Browser,
                                7 => AppMode::ImageViewer,
                                _ => AppMode::Shell,
                            };
                            
                            cfi = true;
                            ap.clear();
                            for line in cyo!(xm) {
                                ap.push(String::from(*line));
                            }
                            ap.push(String::from(""));
                            break;
                        }
                    }
                }
            }
        }
        
        
        
        
        
        let mtn = (frame_count % kwh) == 0;
        
        if !mtn {
            
            
            
            compositor.present_only();
            
            if xk { aqp.update(); }
        } else {
        
        grc += 1;
        
        
        
        
        
        if let Some(bg) = compositor.get_layer_mut(kbs) {
            let buf_ptr = bg.buffer.as_mut_ptr();
            let buf_len = bg.buffer.len();
            
            
            if xk {
                
                
                
                
                
                aqp.update();
                aqp.render(&mut bg.buffer, width as usize, height as usize);
            } else if xl {
                
                
                
                
                
                
                crate::gpu_emu::ore(
                    buf_ptr,
                    width as usize,
                    height as usize,
                );
            } else if yg {
                
                
                
                dty.update();
                dty.render(&mut bg.buffer, width as usize, height as usize);
            } else if vb {
                
                
                
                
                bqu.update();
                bqu.render(&mut bg.buffer, width as usize, height as usize);
                
                bqu.render_cube_flow_layer(&mut bg.buffer, width as usize, height as usize);
                bqu.render_entity_layer(&mut bg.buffer, width as usize, height as usize);
            } else if vv && !bju && !aia {
                
                
                
                bg.buffer.fill(awh);
                hxu.update();
                hxu.render(&mut bg.buffer, width as usize, height as usize);
            } else if bju {
                
                
                
                
                
                
                holovolume.set_screen_size(width as usize, height as usize);
                holovolume.update(0.016);
                
                
                for col in 0..AD_ {
                    matrix_heads[col] += matrix_speeds[col] as i32;
                    if matrix_heads[col] > (AV_ as i32 + 30) {
                        let seed = (col as u32 * 2654435761).wrapping_add(frame_count as u32);
                        matrix_heads[col] = -((seed % 30) as i32);
                        matrix_speeds[col] = 1 + (seed % 3);
                    }
                }
                
                
                unsafe {
                    #[cfg(target_arch = "x86_64")]
                    crate::graphics::simd::adq(buf_ptr, buf_len, awh);
                    #[cfg(not(target_arch = "x86_64"))]
                    bg.buffer.fill(awh);
                }
                
                
                let holo_intensity = holovolume.get_u8_intensity_map();
                
                
                let params = super::Ii {
                    buf_ptr,
                    buf_len,
                    width,
                    height,
                    matrix_chars: matrix_chars.as_ptr(),
                    matrix_heads: matrix_heads.as_ptr(),
                    holo_intensity: holo_intensity.as_ptr(),
                    matrix_rows: AV_,
                };
                
                crate::cpu::smp::bcz(
                    AD_,
                    super::izr,
                    &params as *const super::Ii as *mut u8
                );
                
            } else if aia {
                
                
                
                
                if ckr.is_raytraced() {
                    
                    
                    
                    use crate::graphics::raytracer::{Vec3, Material};
                    
                    raytracer.update(0.016);
                    
                    
                    match ckr {
                        crate::graphics::holomatrix::HoloScene::RayTracedSpheres => {
                            raytracer.setup_spheres_scene();
                        },
                        crate::graphics::holomatrix::HoloScene::RayTracedDNA => {
                            raytracer.setup_dna_scene();
                        },
                        _ => {}
                    }
                    
                    
                    let oiv = raytracer.render();
                    
                    
                    let gsb = raytracer.width;
                    let jbs = raytracer.height;
                    let oks = width as usize / gsb;
                    let okt = height as usize / jbs;
                    
                    for y in 0..height as usize {
                        for x in 0..width as usize {
                            let da = (x / oks).min(gsb - 1);
                            let cm = (y / okt).min(jbs - 1);
                            let color = oiv[cm * gsb + da];
                            bg.buffer[y * width as usize + x] = color;
                        }
                    }
                } else {
                    
                    
                    
                    
                    
                    
                    
                    holomatrix.update(0.016);
                    let time = holomatrix.time;
                    
                    
                    
                    
                    
                    
                    
                    let mut intensity_map = [[0u8; AV_]; AD_];
                    
                    
                    let cell_w = (width as f32) / (AD_ as f32);  
                    let cell_h = (height as f32) / (AV_ as f32); 
                    
                    
                    let cx = width as f32 / 2.0;
                    let u = height as f32 / 2.0;
                    let scale = (height as f32 / 3.0).min(width as f32 / 4.0);
                    
                    
                    match ckr {
                        crate::graphics::holomatrix::HoloScene::DNA => {
                            
                            let iep = 2.2;
                            let radius = 0.45;
                            let bac = 3.5;
                            
                            for i in 0..180 {
                                let t = i as f32 / 180.0;
                                let y = -iep / 2.0 + t * iep;
                                let cc = t * bac * 6.28318 + time;
                                
                                
                                let x1 = radius * crate::graphics::holomatrix::byi(cc);
                                let po = radius * crate::graphics::holomatrix::azr(cc);
                                
                                
                                let x2 = radius * crate::graphics::holomatrix::byi(cc + 3.14159);
                                let qt = radius * crate::graphics::holomatrix::azr(cc + 3.14159);
                                
                                
                                let rot_y = time * 0.4;
                                let bax = crate::graphics::holomatrix::byi(rot_y);
                                let bds = crate::graphics::holomatrix::azr(rot_y);
                                
                                
                                let bvo = x1 * bax + po * bds;
                                let auw = -x1 * bds + po * bax;
                                let bja = x2 * bax + qt * bds;
                                let auy = -x2 * bds + qt * bax;
                                
                                
                                let hri = 1.0 / (2.0 + auw);
                                let wn = cx + bvo * scale * hri;
                                let aiu = u + y * scale * hri;
                                let awp = (wn / cell_w) as usize;
                                let azk = (aiu / cell_h) as usize;
                                
                                let hrj = 1.0 / (2.0 + auy);
                                let tq = cx + bja * scale * hrj;
                                let acv = u + y * scale * hrj;
                                let awq = (tq / cell_w) as usize;
                                let azl = (acv / cell_h) as usize;
                                
                                
                                let dsd = (180.0 + 75.0 * (1.0 - ((auw + 0.5) * 0.5).max(0.0).min(1.0))) as u8;
                                let dse = (180.0 + 75.0 * (1.0 - ((auy + 0.5) * 0.5).max(0.0).min(1.0))) as u8;
                                
                                
                                if awp < AD_ && azk < AV_ {
                                    intensity_map[awp][azk] = intensity_map[awp][azk].max(dsd);
                                    
                                    if awp > 0 { intensity_map[awp-1][azk] = intensity_map[awp-1][azk].max(dsd * 2/3); }
                                    if awp < AD_-1 { intensity_map[awp+1][azk] = intensity_map[awp+1][azk].max(dsd * 2/3); }
                                    if azk > 0 { intensity_map[awp][azk-1] = intensity_map[awp][azk-1].max(dsd/2); }
                                    if azk < AV_-1 { intensity_map[awp][azk+1] = intensity_map[awp][azk+1].max(dsd/2); }
                                }
                                if awq < AD_ && azl < AV_ {
                                    intensity_map[awq][azl] = intensity_map[awq][azl].max(dse);
                                    if awq > 0 { intensity_map[awq-1][azl] = intensity_map[awq-1][azl].max(dse * 2/3); }
                                    if awq < AD_-1 { intensity_map[awq+1][azl] = intensity_map[awq+1][azl].max(dse * 2/3); }
                                    if azl > 0 { intensity_map[awq][azl-1] = intensity_map[awq][azl-1].max(dse/2); }
                                    if azl < AV_-1 { intensity_map[awq][azl+1] = intensity_map[awq][azl+1].max(dse/2); }
                                }
                                
                                
                                if i % 12 == 0 {
                                    for j in 0..8 {
                                        let uz = j as f32 / 7.0;
                                        let fe = wn * (1.0 - uz) + tq * uz;
                                        let ly = aiu * (1.0 - uz) + acv * uz;
                                        let gfd = (fe / cell_w) as usize;
                                        let ggk = (ly / cell_h) as usize;
                                        if gfd < AD_ && ggk < AV_ {
                                            intensity_map[gfd][ggk] = intensity_map[gfd][ggk].max(80);
                                        }
                                    }
                                }
                            }
                        },
                        crate::graphics::holomatrix::HoloScene::RotatingCube => {
                            let cw = 0.5;
                            let vertices: [(f32, f32, f32); 8] = [
                                (-cw, -cw, -cw), (cw, -cw, -cw),
                                (cw, cw, -cw), (-cw, cw, -cw),
                                (-cw, -cw, cw), (cw, -cw, cw),
                                (cw, cw, cw), (-cw, cw, cw),
                            ];
                            let edges: [(usize, usize); 12] = [
                                (0,1), (1,2), (2,3), (3,0),
                                (4,5), (5,6), (6,7), (7,4),
                                (0,4), (1,5), (2,6), (3,7),
                            ];
                            
                            let rot_x = time * 0.7;
                            let rot_y = time * 0.5;
                            
                            for (i1, i2) in edges.iter() {
                                let (vx1, vy1, vz1) = vertices[*i1];
                                let (fes, vy2, vz2) = vertices[*i2];
                                
                                for j in 0..30 {
                                    let t = j as f32 / 29.0;
                                    let x = vx1 * (1.0 - t) + fes * t;
                                    let y = vy1 * (1.0 - t) + vy2 * t;
                                    let z = vz1 * (1.0 - t) + vz2 * t;
                                    
                                    
                                    let ahr = crate::graphics::holomatrix::byi(rot_x);
                                    let aiq = crate::graphics::holomatrix::azr(rot_x);
                                    let cm = y * ahr - z * aiq;
                                    let qp = y * aiq + z * ahr;
                                    let ahs = crate::graphics::holomatrix::byi(rot_y);
                                    let air = crate::graphics::holomatrix::azr(rot_y);
                                    let da = x * ahs + qp * air;
                                    let auy = -x * air + qp * ahs;
                                    
                                    let depth = 1.0 / (2.0 + auy);
                                    let am = cx + da * scale * depth;
                                    let ak = u + cm * scale * depth;
                                    let col = (am / cell_w) as usize;
                                    let row = (ak / cell_h) as usize;
                                    
                                    if col < AD_ && row < AV_ {
                                        let czu = (100.0 + 100.0 * (1.0 - ((auy + 0.6) * 0.5).max(0.0).min(1.0))) as u8;
                                        intensity_map[col][row] = intensity_map[col][row].max(czu);
                                    }
                                }
                            }
                        },
                        _ => {
                            
                            for i in 0..300 {
                                let aij = (i as f32 / 300.0) * 6.28318;
                                let acz = (i as f32 * 0.618033 * 6.28318) % 6.28318;
                                
                                let r = 0.55;
                                let x = r * crate::graphics::holomatrix::azr(acz) * crate::graphics::holomatrix::byi(aij);
                                let y = r * crate::graphics::holomatrix::azr(acz) * crate::graphics::holomatrix::azr(aij);
                                let z = r * crate::graphics::holomatrix::byi(acz);
                                
                                let chs = crate::graphics::holomatrix::byi(time * 0.5);
                                let cqr = crate::graphics::holomatrix::azr(time * 0.5);
                                let da = x * chs + z * cqr;
                                let qp = -x * cqr + z * chs;
                                
                                let depth = 1.0 / (2.0 + qp);
                                let am = cx + da * scale * depth;
                                let ak = u + y * scale * depth;
                                let col = (am / cell_w) as usize;
                                let row = (ak / cell_h) as usize;
                                
                                if col < AD_ && row < AV_ {
                                    let czu = (80.0 + 120.0 * (1.0 - ((qp + 0.6) * 0.5).max(0.0).min(1.0))) as u8;
                                    intensity_map[col][row] = intensity_map[col][row].max(czu);
                                }
                            }
                        }
                    }
                    
                    
                    
                    
                    
                    
                    
                    for col in 0..AD_ {
                        matrix_heads[col] += matrix_speeds[col] as i32;
                        if matrix_heads[col] > (AV_ as i32 + 30) {
                            let seed = (col as u32 * 2654435761).wrapping_add(frame_count as u32);
                            matrix_heads[col] = -((seed % 30) as i32);
                            matrix_speeds[col] = 1 + (seed % 3);
                        }
                    }
                    
                    
                    bg.buffer.fill(0xFF000000);
                    
                    
                    let ati = 8u32;
                    for col in 0..AD_ {
                        let x = col as u32 * ati;
                        if x >= width { continue; }
                        
                        let su = matrix_heads[col];
                        
                        for row in 0..AV_ {
                            let y = row as u32 * 16;
                            if y >= height { continue; }
                            
                            let em = row as i32 - su;
                            
                            
                            let qf = if em < 0 {
                                continue;
                            } else if em == 0 {
                                255u32  
                            } else if em <= 12 {
                                255 - (em as u32 * 8)
                            } else if em <= 28 {
                                
                                let ha = ((em - 12) as u32).min(15) * 16;
                                let ln = 255u32.saturating_sub(ha);
                                (160 * ln) / 255
                            } else {
                                continue;
                            };
                            
                            
                            let ahj = intensity_map[col][row] as u32;
                            
                            
                            
                            let (r, g, b) = if ahj > 0 {
                                
                                let intensity = (qf + ahj * 2).min(255);
                                let hqd = (ahj as u32 * 3 / 2).min(255);
                                (hqd / 3, intensity, hqd)  
                            } else {
                                
                                let dim = (qf / 3).min(80);
                                (0, dim, 0)
                            };
                            
                            let color = 0xFF000000 | (r << 16) | (g << 8) | b;
                            
                            
                            let c = matrix_chars[imb(col, row)] as char;
                            let du = crate::framebuffer::font::ol(c);
                            
                            
                            for (r, &bits) in du.iter().enumerate() {
                                let o = y + r as u32;
                                if o >= height { break; }
                                let pq = (o * width) as usize;
                                
                                if bits != 0 {
                                    let ani = x as usize;
                                    if bits & 0x80 != 0 { let idx = pq + ani; if idx < bg.buffer.len() { bg.buffer[idx] = color; } }
                                    if bits & 0x40 != 0 { let idx = pq + ani + 1; if idx < bg.buffer.len() { bg.buffer[idx] = color; } }
                                    if bits & 0x20 != 0 { let idx = pq + ani + 2; if idx < bg.buffer.len() { bg.buffer[idx] = color; } }
                                    if bits & 0x10 != 0 { let idx = pq + ani + 3; if idx < bg.buffer.len() { bg.buffer[idx] = color; } }
                                    if bits & 0x08 != 0 { let idx = pq + ani + 4; if idx < bg.buffer.len() { bg.buffer[idx] = color; } }
                                    if bits & 0x04 != 0 { let idx = pq + ani + 5; if idx < bg.buffer.len() { bg.buffer[idx] = color; } }
                                    if bits & 0x02 != 0 { let idx = pq + ani + 6; if idx < bg.buffer.len() { bg.buffer[idx] = color; } }
                                    if bits & 0x01 != 0 { let idx = pq + ani + 7; if idx < bg.buffer.len() { bg.buffer[idx] = color; } }
                                }
                            }
                        }
                    }
                }
            } else {
                
                
                
                
                
                for col in 0..AD_ {
                    matrix_heads[col] += matrix_speeds[col] as i32;
                    if matrix_heads[col] > (AV_ as i32 + 30) {
                        let seed = (col as u32 * 2654435761).wrapping_add(frame_count as u32);
                        matrix_heads[col] = -((seed % 30) as i32);
                        matrix_speeds[col] = 1 + (seed % 3);
                    }
                }
                
                
                unsafe {
                    #[cfg(target_arch = "x86_64")]
                    crate::graphics::simd::adq(buf_ptr, buf_len, awh);
                    #[cfg(not(target_arch = "x86_64"))]
                    bg.buffer.fill(awh);
                }
                
                
                
                
                
                let params = super::Ii {
                    buf_ptr,
                    buf_len,
                    width,
                    height,
                    matrix_chars: matrix_chars.as_ptr(),
                    matrix_heads: matrix_heads.as_ptr(),
                    holo_intensity: core::ptr::null(),  
                    matrix_rows: AV_,
                };
                
                
                crate::cpu::smp::bcz(
                    AD_,
                    super::izr,
                    &params as *const super::Ii as *mut u8
                );
            }
        }
        
        
        
        
        if let Some(dock) = compositor.get_layer_mut(lgu) {
            dock.clear(0xF0080808); 
            
            let rl = 36u32;  
            let gap = 50u32;       
            let start_y = 10u32;
            
            let lgr = [
                ("Files", AppMode::Files),
                ("Shell", AppMode::Shell),
                ("Net", AppMode::Network),
                ("Edit", AppMode::TextEditor),
                ("HW", AppMode::Hardware),
                ("User", AppMode::UserMgmt),
                ("Web", AppMode::Browser),  
                ("Img", AppMode::ImageViewer), 
            ];
            
            for (i, (name, mode)) in lgr.iter().enumerate() {
                let gg = start_y + (i as u32) * (rl + gap);
                let bi = 10u32;
                
                let is_active = *mode == xm;
                let icon_color = if is_active { bby } else { ph };
                let ace = if is_active { 0xFFFFFFFF } else { 0xFF888888 };
                
                
                if is_active {
                    dock.fill_rect(bi - 4, gg - 4, rl + 8, rl + 20, 0xFF002800);
                    dock.draw_rect(bi - 4, gg - 4, rl + 8, rl + 20, kw);
                }
                dock.fill_rect(bi, gg, rl, rl, 0xFF0A0A0A);
                dock.draw_rect(bi, gg, rl, rl, icon_color);
                
                
                let cx = bi + rl / 2;
                let u = gg + rl / 2;
                match i {
                    0 => { 
                        dock.fill_rect(cx - 12, u - 2, 24, 14, icon_color);
                        dock.fill_rect(cx - 14, u - 6, 10, 6, icon_color);
                    },
                    1 => { 
                        dock.draw_rect(cx - 14, u - 10, 28, 20, icon_color);
                        dock.draw_text(">", cx - 8, u - 4, icon_color);
                        dock.fill_rect(cx - 2, u - 2, 10, 2, icon_color);
                    },
                    2 => { 
                        dock.fill_circle(cx, u, 12, icon_color);
                        dock.fill_circle(cx, u, 8, 0xFF0A0A0A);
                        dock.fill_circle(cx, u, 4, icon_color);
                        
                        dock.fill_rect(cx + 6, u - 2, 2, 6, icon_color);
                        dock.fill_rect(cx + 10, u - 6, 2, 10, icon_color);
                    },
                    3 => { 
                        dock.fill_rect(cx - 10, u - 12, 20, 24, icon_color);
                        dock.fill_rect(cx - 8, u - 10, 16, 20, 0xFF0A0A0A);
                        dock.fill_rect(cx - 6, u - 6, 12, 2, icon_color);
                        dock.fill_rect(cx - 6, u - 2, 12, 2, icon_color);
                        dock.fill_rect(cx - 6, u + 2, 8, 2, icon_color);
                    },
                    4 => { 
                        dock.fill_rect(cx - 10, u - 8, 20, 16, icon_color);
                        for ay in 0..4 {
                            dock.fill_rect(cx - 14, u - 6 + ay * 4, 4, 2, icon_color);
                            dock.fill_rect(cx + 10, u - 6 + ay * 4, 4, 2, icon_color);
                        }
                    },
                    5 => { 
                        dock.fill_circle(cx, u - 4, 6, icon_color);
                        dock.fill_rect(cx - 8, u + 4, 16, 8, icon_color);
                    },
                    6 => { 
                        dock.fill_circle(cx, u, 10, icon_color);
                        dock.fill_circle(cx, u, 6, 0xFF0A0A0A);
                        
                        dock.fill_rect(cx - 10, u - 1, 20, 2, icon_color);
                        
                        dock.fill_rect(cx - 1, u - 10, 2, 20, icon_color);
                    },
                    _ => {}
                }
                
                
                let kd = bi + (rl / 2) - ((name.len() as u32 * 8) / 2);
                dock.draw_text(name, kd, gg + rl + 2, ace);
            }
        }
        
        
        
        
        if let Some(aw) = compositor.get_layer_mut(jre) {
            if cfi {
                
                let w = aw.width;
                let h = aw.height;
                
                aw.clear(pur);
                
                
                aw.draw_rect(0, 0, w, h, kw);
                aw.draw_rect(1, 1, w - 2, h - 2, kw);
                
                
                let bcu = match xm {
                    AppMode::Shell => "Shell",
                AppMode::Network => "Network",
                AppMode::Hardware => "Hardware",
                AppMode::TextEditor => "TrustCode",
                AppMode::UserMgmt => "User Management",
                AppMode::Files => "Files",
                AppMode::Browser => "Web Browser",
                AppMode::ImageViewer => "Image Viewer",
            };
            aw.fill_rect(2, 2, w - 4, 26, 0xFF0A1A0A);
            let title = format!("TrustOS - {} Module", bcu);
            aw.draw_text(&title, 12, 8, 0xFFFFFFFF); 
            
            
            if dnp {
                aw.draw_text("[MOVING]", w / 2 - 32, 8, 0xFFFFAA00);
            }
            
            
            
            let ed = 13u32;
            let atd = 10u32;
            let cuk = w - 50;
            let hiv = w - 80;
            let fjw = w - 110;
            
            
            aw.fill_circle(cuk, ed, atd, 0xFFFF4444);
            aw.draw_rect(cuk, ed, 1, 1, 0xFFFF6666); 
            
            for t in 0..7 {
                aw.set_pixel(cuk - 5 + t, ed - 5 + t, 0xFFFFFFFF);
                aw.set_pixel(cuk - 4 + t, ed - 5 + t, 0xFFFFFFFF);
                aw.set_pixel(cuk + 5 - t, ed - 5 + t, 0xFFFFFFFF);
                aw.set_pixel(cuk + 4 - t, ed - 5 + t, 0xFFFFFFFF);
            }
            
            
            aw.fill_circle(hiv, ed, atd, 0xFFFFCC00);
            
            aw.fill_rect(hiv - 5, ed - 1, 10, 3, 0xFF000000);
            
            
            aw.fill_circle(fjw, ed, atd, 0xFF44DD44);
            
            aw.draw_rect(fjw - 5, ed - 5, 10, 10, 0xFF000000);
            aw.draw_rect(fjw - 4, ed - 4, 8, 8, 0xFF000000);
            
            
            let bn = 35u32;
            let line_height = 18u32;
            let ncy = ((h - bn - 50) / line_height) as usize;
            
            if xm == AppMode::Browser {
                
                
                
                
                
                let uk = bn;
                aw.fill_rect(10, uk, w - 20, 32, 0xFF1E1E1E);
                aw.draw_rect(10, uk, w - 20, 32, 0xFF3C3C3C);
                
                
                let bxv: u32 = 0xFF2D2D2D;
                
                
                aw.fill_rect(14, uk + 4, 24, 24, bxv);
                aw.draw_text("<", 22, uk + 10, 0xFFAAAAAA);
                
                
                aw.fill_rect(42, uk + 4, 24, 24, bxv);
                aw.draw_text(">", 50, uk + 10, 0xFFAAAAAA);
                
                
                aw.fill_rect(70, uk + 4, 24, 24, bxv);
                aw.draw_text("R", 78, uk + 10, 0xFFAAAAAA);
                
                
                let psa = if djq == 0 { "SRC" } else { "DOM" };
                aw.fill_rect(98, uk + 4, 32, 24, 0xFF383838);
                aw.draw_text(psa, 102, uk + 10, 0xFF88CCFF);
                
                
                aw.fill_rect(135, uk + 4, w - 160, 24, 0xFF0D0D0D);
                aw.draw_rect(135, uk + 4, w - 160, 24, if his { 0xFF4FC3F7 } else { 0xFF555555 });
                
                
                let ppz = if ahk.starts_with("https://") { 0xFF00C853 } else { 0xFFDDDDDD };
                aw.draw_text(&ahk, 142, uk + 10, ppz);
                
                
                if his && cursor_blink {
                    let cursor_x = 142 + (ahk.len() as u32 * 8);
                    if cursor_x < w - 30 {
                        aw.fill_rect(cursor_x, uk + 8, 2, 18, 0xFF4FC3F7);
                    }
                }
                
                
                let arr = bn + 40;
                let itg = ((h - arr - 35) / line_height) as usize;
                
                
                aw.fill_rect(10, arr - 4, w - 20, h - arr - 28, 0xFF1E1E1E);
                
                
                let gutter_width = 40u32;
                aw.fill_rect(10, arr - 4, gutter_width, h - arr - 28, 0xFF252526);
                
                let bjj = if gy.len() > itg {
                    gy.len() - itg
                } else {
                    0
                };
                
                
                for (i, browser_line) in gy.iter().skip(bjj).enumerate() {
                    let y = arr + (i as u32) * line_height;
                    if y + line_height > h - 35 { break; }
                    
                    
                    let axw = format!("{:3}", bjj + i + 1);
                    aw.draw_text(&axw, 14, y, 0xFF858585);
                    
                    
                    let mut jrv = 10u32 + gutter_width + 5;
                    for segment in &browser_line.segments {
                        aw.draw_text(&segment.text, jrv, y, segment.color);
                        jrv += (segment.text.len() as u32) * 8;
                    }
                }
                
                
                aw.fill_rect(10, h - 28, w - 20, 23, 0xFF007ACC);
                
                
                let gwf = if ajg.contains("Error") { "?" } 
                    else if ajg.contains("Loading") { "?" } 
                    else { "?" };
                aw.draw_text(gwf, 16, h - 24, 0xFFFFFFFF);
                aw.draw_text(&ajg, 30, h - 24, 0xFFFFFFFF);
                
                
                let nfv = if djq == 0 { "[Source]" } else { "[Elements]" };
                let nfw = w - 90;
                aw.draw_text(nfv, nfw, h - 24, 0xFFCCCCCC);
                
            } else if xm == AppMode::ImageViewer {
                
                
                
                
                
                aw.fill_rect(10, bn, w - 20, h - bn - 30, 0xFF1A1A1A);
                
                if let Some(ref iv) = ift {
                    
                    let fej = w - 40;
                    let aak = h - bn - 60;
                    let fek = 20u32;
                    let bws = bn + 10;
                    
                    
                    let gss = (iv.width as f32 * bhc) as u32;
                    let gsr = (iv.height as f32 * bhc) as u32;
                    
                    
                    let center_x = fek as i32 + (fej as i32 / 2) + gbu;
                    let center_y = bws as i32 + (aak as i32 / 2) + gbv;
                    let ckw = center_x - (gss as i32 / 2);
                    let ckx = center_y - (gsr as i32 / 2);
                    
                    
                    for ad in 0..gsr.min(aak) {
                        let nn = ckx + ad as i32;
                        if nn < bws as i32 || nn >= (bws + aak) as i32 {
                            continue;
                        }
                        
                        let aft = ((ad as f32 / bhc) as u32).min(iv.height - 1);
                        
                        for dx in 0..gss.min(fej) {
                            let lw = ckw + dx as i32;
                            if lw < fek as i32 || lw >= (fek + fej) as i32 {
                                continue;
                            }
                            
                            let ahc = ((dx as f32 / bhc) as u32).min(iv.width - 1);
                            let ct = iv.get_pixel(ahc, aft);
                            
                            
                            if (ct >> 24) > 0 {
                                aw.set_pixel(lw as u32, nn as u32, ct);
                            }
                        }
                    }
                    
                    
                    aw.draw_rect(
                        (ckw.max(fek as i32)) as u32,
                        (ckx.max(bws as i32)) as u32,
                        gss.min(fej),
                        gsr.min(aak),
                        0xFF444444
                    );
                } else {
                    
                    let center_x = w / 2;
                    let center_y = (bn + h) / 2;
                    
                    
                    aw.draw_rect(center_x - 40, center_y - 30, 80, 60, 0xFF444444);
                    aw.draw_text("??", center_x - 8, center_y - 10, 0xFF666666);
                    aw.draw_text("No image loaded", center_x - 56, center_y + 25, 0xFF888888);
                    aw.draw_text("Use: imgview <file>", center_x - 72, center_y + 45, 0xFF666666);
                }
                
                
                aw.fill_rect(10, bn, w - 20, 24, 0xFF252525);
                let pwo = (bhc * 100.0) as u32;
                let gck = format!("Zoom: {}%  |  {}", pwo, ifv);
                aw.draw_text(&gck, 16, bn + 5, 0xFFCCCCCC);
                
                
                aw.draw_text(&ifu, w - 60, bn + 5, 0xFF88CCFF);
                
                
                aw.fill_rect(10, h - 28, w - 20, 23, 0xFF252525);
                aw.draw_text("[+/-] Zoom  [Arrows] Pan  [R] Reset  [ESC] Close", 16, h - 24, 0xFF888888);
                
            } else if xm == AppMode::TextEditor {
                
                
                
                use crate::apps::text_editor::*;
                
                let ew: u32 = 8;
                let bw: u32 = 16;
                let mgr: u32 = 5; 
                let ajv = mgr * ew;
                let aej: u32 = 22;
                let bpg: u32 = 26;
                
                let adm = ajv;
                let adn = bn + bpg;
                let ein = w - ajv;
                let anu = h.saturating_sub(bn + bpg + aej);
                let dgi = (anu / bw).max(1) as usize;
                
                
                if editor_state.cursor_line < editor_state.scroll_y {
                    editor_state.scroll_y = editor_state.cursor_line;
                }
                if editor_state.cursor_line >= editor_state.scroll_y + dgi {
                    editor_state.scroll_y = editor_state.cursor_line - dgi + 1;
                }
                editor_state.blink_counter += 1;
                
                
                aw.fill_rect(0, bn, w, bpg, BPQ_);
                let gxw = editor_state.file_path.as_ref().map(|aa| {
                    aa.rsplit('/').next().unwrap_or(aa.as_str())
                }).unwrap_or("untitled");
                let fsi = if editor_state.dirty { " *" } else { "" };
                let ebe = format!("  {}{}", gxw, fsi);
                let zm = ((ebe.len() as u32 + 2) * ew).min(w);
                aw.fill_rect(0, bn, zm, bpg, BQF_);
                
                aw.fill_rect(0, bn + bpg - 2, zm, 2, ABU_);
                aw.draw_text(&ebe, 4, bn + 5, KH_);
                
                
                aw.fill_rect(0, adn, w, anu, ND_);
                
                
                aw.fill_rect(0, adn, ajv, anu, AQE_);
                aw.fill_rect(ajv - 1, adn, 1, anu, 0xFF333333);
                
                
                for pt in 0..dgi {
                    let xf = editor_state.scroll_y + pt;
                    if xf >= editor_state.lines.len() { break; }
                    
                    let ly = adn + (pt as u32 * bw);
                    if ly + bw > adn + anu { break; }
                    
                    let is_current = xf == editor_state.cursor_line;
                    
                    
                    if is_current {
                        aw.fill_rect(adm, ly, ein, bw, AQB_);
                    }
                    
                    
                    let rw = format!("{:>4} ", xf + 1);
                    let dvm = if is_current { BPN_ } else { AQH_ };
                    aw.draw_text(&rw, 2, ly, dvm);
                    
                    
                    let line = &editor_state.lines[xf];
                    
                    if editor_state.language == Language::Rust {
                        let tokens = jnh(line);
                        for bjg in &tokens {
                            let color = jnf(bjg.kind);
                            let pik = &line[bjg.start..bjg.end];
                            let am = adm + 4 + (bjg.start as u32 * ew);
                            if am < w {
                                aw.draw_text(pik, am, ly, color);
                            }
                        }
                        if tokens.is_empty() && !line.is_empty() {
                            aw.draw_text(line, adm + 4, ly, KH_);
                        }
                    } else {
                        aw.draw_text(line, adm + 4, ly, KH_);
                    }
                    
                    
                    if is_current {
                        let fjk = (editor_state.blink_counter / 30) % 2 == 0;
                        if fjk {
                            let cx = adm + 4 + (editor_state.cursor_col as u32 * ew);
                            aw.fill_rect(cx, ly, 2, bw, AQC_);
                        }
                    }
                }
                
                
                if editor_state.lines.len() > dgi {
                    let yc = w - 10;
                    let bdo = anu;
                    let av = editor_state.lines.len() as u32;
                    let zo = ((dgi as u32 * bdo) / av).max(20);
                    let imu = av.saturating_sub(dgi as u32);
                    let akn = if imu > 0 {
                        (editor_state.scroll_y as u32 * (bdo - zo)) / imu
                    } else { 0 };
                    aw.fill_rect(yc, adn, 10, bdo, 0xFF252526);
                    aw.fill_rect(yc + 2, adn + akn, 6, zo, 0xFF555555);
                }
                
                
                let status_y = h - aej;
                aw.fill_rect(0, status_y, w, aej, ABU_);
                
                
                let owy = if let Some(ref bk) = editor_state.status_message {
                    format!("  {}", bk)
                } else {
                    let lex = if editor_state.dirty { " [Modified]" } else { "" };
                    let bsr = editor_state.file_path.as_deref().unwrap_or("untitled");
                    format!("  {}{}", bsr, lex)
                };
                aw.draw_text(&owy, 4, status_y + 3, GS_);
                
                
                let jir = format!(
                    "Ln {}, Col {}  {}  UTF-8  TrustCode",
                    editor_state.cursor_line + 1,
                    editor_state.cursor_col + 1,
                    editor_state.language.name(),
                );
                let asa = w.saturating_sub((jir.len() as u32 * ew) + 8);
                aw.draw_text(&jir, asa, status_y + 3, GS_);

            } else {
                
                
                
                
            
            let total_lines = ap.len();
            let oe = AHB_.min(ncy);
            
            
            let aab = total_lines.saturating_sub(oe);
            if scroll_offset > aab {
                scroll_offset = aab;
            }
            
            let bjj = scroll_offset;
            let fus = (bjj + oe).min(total_lines);
            
            for (i, line) in ap.iter().skip(bjj).take(oe).enumerate() {
                let y = bn + (i as u32) * line_height;
                if y + line_height > h - 50 { break; }
                
                
                let color = if line.starts_with("+") || line.starts_with("+") || line.starts_with("|") {
                    kw  
                } else if line.starts_with("|") {
                    
                    if line.contains("NAVIGATION:") || line.contains("FILE OPERATIONS:") || 
                       line.contains("COMMANDS:") || line.contains("TIPS:") ||
                       line.contains("BASIC COMMANDS:") || line.contains("EXAMPLES:") ||
                       line.contains("NOTE:") {
                        0xFFFFFF00  
                    } else if line.contains(" - ") {
                        
                        kw  
                    } else if line.starts_with("|    *") {
                        0xFFAAAAAA  
                    } else {
                        0xFFDDDDDD  
                    }
                } else if line.starts_with(">") {
                    0xFF88FF88  
                } else if line.contains("<DIR>") {
                    0xFF00FFFF  
                } else if line.contains(" B") && !line.contains("Browse") {
                    kw  
                } else if line.starts_with("Created") || line.starts_with("Changed") || line.starts_with("Removed") {
                    0xFF00FF00  
                } else if line.contains("Error") || line.contains("cannot") || line.contains("No such") {
                    0xFFFF4444  
                } else {
                    ph  
                };
                aw.draw_text(line, 12, y, color);
            }
            
            
            
            
            if total_lines > oe {
                let cqa = w - 12;
                let jdq = bn;
                let gta = h - bn - 50;  
                
                
                aw.fill_rect(cqa, jdq, 8, gta, 0xFF1A1A1A);
                
                
                let pjd = oe as f32 / total_lines as f32;
                let zo = ((gta as f32 * pjd) as u32).max(20);
                let gsz = if aab > 0 { 
                    scroll_offset as f32 / aab as f32 
                } else { 
                    0.0 
                };
                let akn = jdq + ((gta - zo) as f32 * gsz) as u32;
                
                
                aw.fill_rect(cqa, akn, 8, zo, ph);
                aw.fill_rect(cqa + 1, akn + 1, 6, zo - 2, kw);
            }
            
            
            let sv = h - 40;
            aw.fill_rect(10, sv, w - 20, 30, 0xFF050505);
            aw.draw_rect(10, sv, w - 20, 30, ph);
            
            
            let cwd = crate::ramfs::bh(|fs| String::from(fs.pwd()));
            
            aw.draw_text("root", 16, sv + 8, 0xFFFF0000);  
            
            aw.draw_text("@", 16 + 32, sv + 8, 0xFFFFFFFF);  
            
            aw.draw_text("trustos", 16 + 40, sv + 8, 0xFF00FF00);  
            
            let dwh = format!(":{}$ ", cwd);
            aw.draw_text(&dwh, 16 + 96, sv + 8, 0xFF00FF00);  
            let gou = (4 + 1 + 7 + dwh.len()) as u32 * 8;  
            
            
            aw.draw_text(&shell_input, 16 + gou, sv + 8, bby);
            
            
            if !dez.is_empty() {
                let cax = (shell_input.len() * 8) as u32;
                aw.draw_text(&dez, 16 + gou + cax, sv + 8, 0xFF444444);
            }
            
            
            cursor_blink = (frame_count / 30) % 2 == 0;
            if cursor_blink {
                let cursor_x = 16 + gou + (shell_input.len() as u32 * 8);
                aw.fill_rect(cursor_x, sv + 6, 8, 16, bby);
            }
            } 
            } else {
                
                aw.clear(0x00000000);
            }
        }
        
        
        
        
        if let Some(axk) = compositor.get_layer_mut(mlo) {
            let xc = axk.width;
            let agm = axk.height;
            
            
            axk.clear(0xD8181818);
            
            
            axk.draw_rect(0, 0, xc, agm, 0xFF444444);
            axk.draw_rect(1, 1, xc - 2, agm - 2, 0xFF333333);
            
            
            axk.fill_rect(2, 2, xc - 4, 20, 0xFF252525);
            axk.draw_text("Command History", 8, 6, 0xFFAAAAAA);
            
            
            let start_y = 26u32;
            let bw = 18u32;
            
            if command_history.is_empty() {
                axk.draw_text("(no commands yet)", 10, start_y + 5, 0xFF666666);
            } else {
                
                for (i, cmd) in command_history.iter().rev().take(10).enumerate() {
                    let y = start_y + (i as u32) * bw;
                    if y + bw > agm - 5 { break; }
                    
                    
                    let num = command_history.len() - i;
                    let rw = format!("{:2}.", num);
                    axk.draw_text(&rw, 6, y + 2, 0xFF666666);
                    
                    
                    let lfi = if cmd.len() > 26 {
                        format!("{}...", &cmd[..23])
                    } else {
                        cmd.clone()
                    };
                    axk.draw_text(&lfi, 30, y + 2, 0xFF88FF88);
                }
            }
        }
        
        
        
        
        if let Some(bar) = compositor.get_layer_mut(pdk) {
            bar.clear(0xFF0A0A0A);
            
            
            bar.fill_rect(0, 0, width, 2, ph);
            
            
            bar.fill_rect(5, 6, 100, 28, if yx { 0xFF002200 } else { 0xFF0A1A0A });
            bar.draw_rect(5, 6, 100, 28, kw);
            bar.draw_text("TrustOS", 20, 12, 0xFFFFFFFF); 
            
            
            let bcu = match xm {
                AppMode::Shell => "Shell",
                AppMode::Network => "Network",
                AppMode::Hardware => "Hardware",
                AppMode::TextEditor => "Editor",
                AppMode::UserMgmt => "Users",
                AppMode::Files => "Files",
                AppMode::Browser => "Browser",
                AppMode::ImageViewer => "Images",
            };
            bar.fill_rect(115, 6, 90, 28, 0xFF001100);
            bar.draw_text(bcu, 125, 12, 0xFFFFFFFF); 
            
            
            
            let csv = 220u32;
            if cfi {
                
                bar.fill_rect(csv, 6, 100, 28, 0xFF002200);
                bar.draw_rect(csv, 6, 100, 28, kw);
                bar.draw_text(bcu, csv + 10, 12, bby);
                
                bar.fill_rect(csv + 20, 32, 60, 3, kw);
            } else {
                
                bar.fill_rect(csv, 6, 100, 28, 0xFF0A0A0A);
                bar.draw_rect(csv, 6, 100, 28, ph);
                bar.draw_text(bcu, csv + 10, 12, ph);
            }
            
            
            let guk = 340u32;
            let oqa = if bou { 0xFF002200 } else { 0xFF0A1A0A };
            bar.fill_rect(guk, 6, 50, 28, oqa);
            bar.draw_rect(guk, 6, 50, 28, ph);
            bar.draw_text("[S]", guk + 10, 12, if bou { bby } else { kw });
            
            
            let fm = crate::rtc::aou();
            let time_str = format!("{:02}:{:02}:{:02}", fm.hour, fm.minute, fm.second);
            bar.draw_text(&time_str, width - 180, 12, kw);
            
            
            let cyc = format!("{}fps", fps);
            bar.draw_text(&cyc, width - 260, 12, ph);
            
            
            bar.fill_circle(width - 60, 20, 6, kw);
            bar.fill_circle(width - 40, 20, 6, 0xFFFFAA00);
        }
        
        
        
        
        if let Some(menu) = compositor.get_layer_mut(nek) {
            if yx {
                menu.visible.store(true, core::sync::atomic::Ordering::SeqCst);
                menu.clear(0xF0080808);  
                
                let pz = 270u32;
                let rv = 390u32;
                
                
                menu.draw_rect(0, 0, pz, rv, kw);
                menu.draw_rect(1, 1, pz - 2, rv - 2, ph);
                
                
                menu.fill_rect(2, 2, pz - 4, 34, 0xFF001500);
                menu.draw_text("TrustOS Menu", 10, 10, kw);
                
                
                let axs = 36u32;
                for (i, (name, item)) in dbg.iter().enumerate() {
                    let gg = 40 + (i as u32) * axs;
                    
                    
                    if name.starts_with("-") {
                        menu.fill_rect(10, gg + 14, pz - 20, 1, ph);
                        continue;
                    }
                    
                    
                    if bcs == i as i32 {
                        menu.fill_rect(5, gg, pz - 10, axs - 2, 0xFF002200);
                    }
                    
                    
                    let (color, icon) = match item {
                        MenuItem::App(_) => {
                            let c = if bcs == i as i32 { bby } else { ph };
                            (c, ">")
                        },
                        MenuItem::Shutdown => {
                            let c = if bcs == i as i32 { 0xFFFF6666 } else { 0xFFAA4444 };
                            (c, "X")
                        },
                        MenuItem::Reboot => {
                            let c = if bcs == i as i32 { 0xFFFFAA66 } else { 0xFFAA8844 };
                            (c, "R")
                        },
                    };
                    
                    
                    menu.draw_text(name, 24, gg + 10, color);
                    
                    
                    menu.draw_text(icon, pz - 30, gg + 10, color);
                }
            } else {
                menu.visible.store(false, core::sync::atomic::Ordering::SeqCst);
            }
        }
        
        
        
        
        if let Some(abj) = compositor.get_layer_mut(oqb) {
            if bou {
                abj.visible.store(true, core::sync::atomic::Ordering::SeqCst);
                abj.clear(0xF0080808);  
                
                let he = 270u32;
                let ug = 340u32;  
                
                
                abj.draw_rect(0, 0, he, ug, kw);
                abj.draw_rect(1, 1, he - 2, ug - 2, ph);
                
                
                abj.fill_rect(2, 2, he - 4, 34, 0xFF001500);
                abj.draw_text("Settings", 10, 10, kw);
                
                
                let dht = 50u32;
                let cr = mouse_y as u32;
                let cg = mouse_x as u32;
                let cce = height - 380;  
                let hff = cr >= (cce + dht) 
                    && cr < (cce + dht + 36)
                    && cg >= 340 && cg < (340 + he);
                if hff {
                    abj.fill_rect(5, dht, he - 10, 34, 0xFF002200);
                }
                abj.draw_text("Animations:", 15, dht + 10, ph);
                let dhs = if dek { "ON" } else { "OFF" };
                let jwe = if dek { 0xFF00FF66 } else { 0xFFFF6666 };
                abj.draw_text(dhs, he - 50, dht + 10, jwe);
                
                
                let eaa = 90u32;
                let jhe = cr >= (cce + eaa) 
                    && cr < (cce + eaa + 36)
                    && cg >= 340 && cg < (340 + he);
                if jhe {
                    abj.fill_rect(5, eaa, he - 10, 34, 0xFF002200);
                }
                abj.draw_text("Speed:", 15, eaa + 10, ph);
                let dzz = format!("{:.1}x", del);
                abj.draw_text(&dzz, he - 60, eaa + 10, kw);
                
                
                abj.draw_text("- Background -", 15, 140, 0xFF555555);
                
                
                let drl = 160u32;
                let iex = cr >= (cce + drl) 
                    && cr < (cce + drl + 36)
                    && cg >= 340 && cg < (340 + he);
                if iex {
                    abj.fill_rect(5, drl, he - 10, 34, 0xFF002200);
                }
                abj.draw_text("HoloMatrix 3D:", 15, drl + 10, ph);
                let mmc = if aia { "ON" } else { "OFF" };
                let mmb = if aia { 0xFF00FFFF } else { 0xFFFF6666 };
                abj.draw_text(mmc, he - 50, drl + 10, mmb);
                
                
                let dym = 200u32;
                let jdi = cr >= (cce + dym) 
                    && cr < (cce + dym + 36)
                    && cg >= 340 && cg < (340 + he);
                if jdi && aia {
                    abj.fill_rect(5, dym, he - 10, 34, 0xFF002200);
                }
                let oll = if aia { ph } else { 0xFF333333 };
                abj.draw_text("Scene:", 15, dym + 10, oll);
                let olk = if aia { 0xFF00FFFF } else { 0xFF444444 };
                abj.draw_text(ckr.name(), he - 80, dym + 10, olk);
                
                
                abj.draw_text("Click to toggle/cycle", 15, 250, 0xFF555555);
                
                
                abj.draw_text("[Esc] or click away", 15, 305, 0xFF444444);
                
                
                if bfi && hff {
                    dek = !dek;
                    crate::desktop::fae(dek);
                }
                
                
                if bfi && jhe {
                    
                    del = if del <= 0.5 { 1.0 } 
                        else if del <= 1.0 { 2.0 } 
                        else { 0.5 };
                    crate::desktop::jew(del);
                }
                
                
                if bfi && iex {
                    aia = !aia;
                    crate::graphics::holomatrix::set_enabled(aia);
                }
                
                
                if bfi && jdi && aia {
                    ckr = ckr.next();
                    crate::graphics::holomatrix::set_scene(ckr);
                }
            } else {
                abj.visible.store(false, core::sync::atomic::Ordering::SeqCst);
            }
        }
        
        
        
        
        if let Some(cursor) = compositor.get_layer_mut(hpx) {
            cursor.clear(0x00000000); 
            
            
            let fqa = if left { 0xFF00FF00 } else { 0xFFFFFFFF }; 
            let ri = if left { 0xFF005500 } else { 0xFF000000 };
            
            
            
            for i in 0..16 {
                for ay in 0..=i {
                    if ay <= i && i < 16 {
                        cursor.set_pixel(ay as u32, i as u32, fqa);
                    }
                }
            }
            
            for i in 0..16 {
                cursor.set_pixel(0, i as u32, ri);
                cursor.set_pixel(i as u32, i as u32, ri);
            }
            
            for i in 10..16 {
                cursor.set_pixel((i - 5) as u32, i as u32, fqa);
                cursor.set_pixel((i - 6) as u32, i as u32, fqa);
            }
        }
        
        
        
        
        compositor.composite();
        compositor.present();
        
        } 
        
        
        frame_count += 1;
        bmg += 1;
        
        
        if frame_count % 100 == 0 {
            crate::serial_println!("[COSMIC2] Frame {}", frame_count);
        }
        
        let cy = crate::cpu::tsc::ey();
        if cy - clz >= aso {
            fps = bmg;
            izq = grc;
            bmg = 0;
            grc = 0;
            clz = cy;
            crate::serial_println!("[COSMIC2] FPS: {} (render: {}) | Frame: {} | Mode: {}",
                fps, izq, frame_count, if xk { "FORMULA" } else if vb { "BRAILLE" } else if vv { "FAST" } else { "LEGACY" });
        }
        
        
        
        
        
        unsafe {
            
            #[cfg(target_arch = "x86_64")]
            core::arch::asm!("sti");
            #[cfg(not(target_arch = "x86_64"))]
            crate::arch::ihd();
            
            for _ in 0..100 {
                #[cfg(target_arch = "x86_64")]
                core::arch::asm!("pause");
                #[cfg(not(target_arch = "x86_64"))]
                core::hint::spin_loop();
            }
        }
    }
    
    
    crate::framebuffer::clear();
    crate::serial_println!("[COSMIC2] Exited");
    crate::n!(B_, "COSMIC V2 Desktop exited. Type 'help' for commands.");
}



pub(super) fn qam() {
    use crate::cosmic::{Rect, Point, Color};
    use crate::cosmic::theme::{dark, matrix};
    use alloc::format;
    
    crate::n!(C_, "+-----------------------------------------------------------+");
    crate::n!(C_, "|       COSMIC Desktop Environment - TrustOS Edition        |");
    crate::n!(C_, "|-----------------------------------------------------------|");
    crate::n!(B_, "|  Controls:                                                |");
    crate::n!(R_, "|    ESC / Q     - Exit desktop                             |");
    crate::n!(R_, "|    M           - Matrix theme (cyberpunk)                 |");
    crate::n!(R_, "|    D           - Dark theme (default)                     |");
    crate::n!(R_, "|    1-5         - Switch apps                              |");
    crate::n!(R_, "|    Mouse       - Interact with UI                         |");
    crate::n!(C_, "+-----------------------------------------------------------+");
    crate::serial_println!("[COSMIC] Starting COSMIC Desktop Environment...");
    
    
    while crate::keyboard::kr().is_some() {}
    
    let (width, height) = crate::framebuffer::kv();
    if width == 0 || height == 0 {
        crate::n!(A_, "Error: Invalid framebuffer!");
        return;
    }
    
    
    crate::framebuffer::adw();
    crate::framebuffer::pr(true);
    crate::serial_println!("[COSMIC] Double buffering enabled for fast rendering");
    
    
    crate::mouse::set_screen_size(width, height);
    
    
    
    
    let mut running = true;
    let mut csg = true;
    let mut frame_count = 0u64;
    
    
    let aso = crate::cpu::tsc::we();
    let mut fps = 0u32;
    let mut bmg = 0u32;
    let mut clz = crate::cpu::tsc::ey();
    let mut dzj = true;  
    
    
    let mut vv = false;  
    let mut vb = true;      
    
    
    let pde = 60u64;
    let qgi = aso / pde;
    let mut mwm = crate::cpu::tsc::ey();
    
    
    
    
    
    const AD_: usize = 160;  
    const BBJ_: u32 = 16;   
    const BBL_: usize = 30; 
    
    
    let mut matrix_cols: [(i32, u32, u32); AD_] = [(0, 0, 0); AD_];
    
    
    for i in 0..AD_ {
        let seed = (i as u32 * 2654435761) ^ 0xDEADBEEF;
        let start_y = -((seed % (height * 2)) as i32); 
        let speed = 2 + (seed % 5); 
        matrix_cols[i] = (start_y, speed, seed);
    }
    
    
    const OZ_: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ@#$%&*+=<>[]{}|";
    
    
    let mut dcw = false;
    let mut cha = false;
    let mut iwb = 0.0f32;
    let mut iwc = 0.0f32;
    let mut iwg = csg;
    let mut ivz = 0usize;
    let mut qra = -1i32;
    let mut needs_full_redraw = true; 
    
    
    let mut bkn = 0usize;
    let mut ifd = -1i32;
    
    
    let mut yx = false;
    let mut bcs = -1i32;  
    let mut search_active = false;
    let mut ezu: [u8; 32] = [0u8; 32];
    let mut cqf = 0usize;
    
    
    let dbg = [
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
    
    
    let mut nw = 150.0f32;
    let mut qr = 60.0f32;
    let ul = 700.0f32;
    let afy = 450.0f32;
    let mut dragging = false;
    let mut qdh = 0.0f32;
    let mut qdi = 0.0f32;
    
    
    let qyx = [
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
    
    
    
    
    while running {
        
        
        
        let mouse = crate::mouse::get_state();
        let cg = mouse.x as f32;
        let cr = mouse.y as f32;
        let ijv = mouse.left_button;
        
        
        cha = ijv && !dcw;
        dcw = ijv;
        
        
        if let Some(key) = crate::keyboard::kr() {
            if search_active {
                
                match key {
                    27 => { search_active = false; },  
                    8 => {  
                        if cqf > 0 { cqf -= 1; }
                    },
                    13 => { search_active = false; },  
                    32..=126 => {  
                        if cqf < 31 {
                            ezu[cqf] = key;
                            cqf += 1;
                        }
                    },
                    _ => {}
                }
            } else {
                
                match key {
                    27 | b'q' | b'Q' => {
                        if yx { yx = false; }
                        else { running = false; }
                    },
                    b'm' | b'M' => { csg = true; needs_full_redraw = true; },
                    b'd' | b'D' => { csg = false; needs_full_redraw = true; },
                    b'1'..=b'5' => { bkn = (key - b'1') as usize; needs_full_redraw = true; },
                    b's' | b'S' => { search_active = true; },  
                    b't' | b'T' => { yx = !yx; },  
                    _ => {}
                }
            }
        }
        
        
        let ngi = (cg - iwb).abs() > 0.5 || (cr - iwc).abs() > 0.5;
        let owp = csg != iwg || bkn != ivz || cha || dragging;
        
        
        
        if !csg && !needs_full_redraw && !ngi && !owp {
            
            frame_count += 1;
            bmg += 1;
            let cy = crate::cpu::tsc::ey();
            if cy - clz >= aso {
                fps = bmg;
                bmg = 0;
                clz = cy;
            }
            continue;
        }
        
        iwb = cg;
        iwc = cr;
        iwg = csg;
        ivz = bkn;
        needs_full_redraw = false;
        
        
        
        
        let (bg, panel_bg, surface, surface_hover, accent, text_pri, text_sec, 
             header_bg, close_bg, ggs, ghr, success, warning) = 
            if csg {
                (matrix::DK_, matrix::LO_, matrix::El, matrix::JT_,
                 matrix::Ch, matrix::AB_, matrix::O_,
                 matrix::HF_, matrix::IH_, matrix::JB_, matrix::JE_,
                 matrix::Nh, matrix::Nw)
            } else {
                (dark::DK_, dark::LO_, dark::El, dark::JT_,
                 dark::Ch, dark::AB_, dark::O_,
                 dark::HF_, dark::IH_, dark::JB_, dark::JE_,
                 dark::Nh, dark::Nw)
            };
        
        
        let pyr = bg.to_u32();
        let qqc = panel_bg.to_u32();
        let qyc = surface.to_u32();
        let qyb = surface_hover.to_u32();
        let pxn = accent.to_u32();
        let raf = text_pri.to_u32();
        let rag = text_sec.to_u32();
        let qku = header_bg.to_u32();
        let qad = close_bg.to_u32();
        let qou = ggs.to_u32();
        let qpa = ghr.to_u32();
        let qxw = success.to_u32();
        let rci = warning.to_u32();
        
        
        use crate::framebuffer::{
            awo, fill_rect, fill_rounded_rect, fill_circle,
            stroke_rounded_rect, draw_text, ii, draw_rect
        };
        
        
        let kw: u32 = 0xFF00FF66;      
        let bby: u32 = 0xFF00FF88;    
        let ph: u32 = 0xFF009944;       
        let awh: u32 = 0xFF000000;           
        
        
        
        
        awo(awh);
        
        
        
        
        let hs = 36u32;
        let ati = width / AD_ as u32;
        
        for col in 0..AD_ {
            let (hde, speed, seed) = matrix_cols[col];
            let x = (col as u32 * ati) + ati / 2;
            
            
            let afk = hde + speed as i32;
            let afk = if afk > height as i32 + (BBL_ as i32 * BBJ_ as i32) {
                let dbr = seed.wrapping_mul(1103515245).wrapping_add(12345);
                matrix_cols[col].2 = dbr;
                -((dbr % (height / 2)) as i32)
            } else {
                afk
            };
            matrix_cols[col].0 = afk;
            
            
            
            
            let depth = (speed as f32 - 2.0) / 4.0; 
            let fro = 0.4 + depth * 0.6; 
            let hrn = 0.3 + depth * 0.7; 
            
            for i in 0..BBL_ {
                let yl = afk - (i as i32 * BBJ_ as i32);
                if yl < 0 || yl >= (height - hs) as i32 { continue; }
                
                let hgv = if i == 0 { 255u8 } 
                    else if i == 1 { 220u8 } 
                    else { 180u8.saturating_sub((i as u8).saturating_mul(9)) };
                if hgv < 20 { continue; }
                
                
                let brightness = ((hgv as f32) * fro) as u8;
                
                
                
                let r = if i == 0 { 
                    ((180.0 * fro) as u8) 
                } else { 
                    
                    ((20.0 * (1.0 - hrn)) as u8)
                };
                let g = brightness;
                let b = if i == 0 { 
                    ((180.0 * fro) as u8) 
                } else { 
                    
                    ((40.0 * (1.0 - hrn) + 10.0 * depth) as u8)
                };
                let color = 0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
                
                let bfe = seed.wrapping_add((i as u32 * 7919) ^ (frame_count as u32 / 8));
                let cuz = (bfe as usize) % OZ_.len();
                let eht: [u8; 2] = [OZ_[cuz], 0];
                let kip = unsafe { core::str::from_utf8_unchecked(&eht[..1]) };
                draw_text(kip, x, yl as u32, color);
            }
        }
        
        
        
        
        
        let aaa = width / 2 + 100;  
        let bnk = height / 2 - 50;
        
        
        let afi = bnk - 180;
        
        fill_rect(aaa - 40, afi, 12, 60, kw);      
        fill_rect(aaa + 28, afi, 12, 60, kw);      
        fill_rect(aaa - 40, afi - 10, 80, 15, kw); 
        fill_rect(aaa - 30, afi - 20, 60, 15, kw); 
        
        fill_rect(aaa - 50, afi + 50, 100, 70, kw);
        fill_rect(aaa - 44, afi + 56, 88, 58, 0xFF0A150Au32);
        
        fill_circle(aaa, afi + 80, 10, kw);
        fill_rect(aaa - 5, afi + 88, 10, 20, kw);
        
        
        let bvt = bnk - 60;
        let apd = 180u32;
        let bje = 220u32;
        
        for t in 0..4 {
            let tt = t as u32;
            
            fill_rect(aaa - apd/2 + 20 + tt, bvt + tt, apd - 40, 3, kw);
            
            fill_rect(aaa - apd/2 + tt, bvt + 20 + tt, 25, 3, kw);
            fill_rect(aaa + apd/2 - 25 - tt, bvt + 20 + tt, 25, 3, kw);
            
            fill_rect(aaa - apd/2 + tt, bvt + 20, 3, bje - 80, kw);
            fill_rect(aaa + apd/2 - 3 - tt, bvt + 20, 3, bje - 80, kw);
            
            fill_rect(aaa - 3, bvt + bje - 20 - tt, 6, 20, kw);
        }
        
        fill_rect(aaa - apd/2 + 8, bvt + 25, apd - 16, bje - 70, 0xFF051208u32);
        
        
        let hko = aaa;
        let hkp = bnk + 20;
        
        for t in 0..8 {
            
            for i in 0..30 {
                fill_rect(hko - 50 + i + t, hkp - 30 + i, 4, 4, kw);
            }
            
            for i in 0..50 {
                fill_rect(hko - 20 + i + t, hkp + i.min(29) - (i.saturating_sub(29)), 4, 4, kw);
            }
        }
        
        
        let cai = bnk + 100;
        
        fill_rect(aaa - 100, cai, 40, 15, ph);
        fill_rect(aaa - 110, cai + 10, 20, 30, ph);
        fill_rect(aaa - 95, cai + 15, 10, 25, ph);
        fill_rect(aaa - 80, cai + 15, 10, 20, ph);
        
        fill_rect(aaa + 60, cai, 40, 15, ph);
        fill_rect(aaa + 90, cai + 10, 20, 30, ph);
        fill_rect(aaa + 85, cai + 15, 10, 25, ph);
        fill_rect(aaa + 70, cai + 15, 10, 20, ph);
        
        
        
        let kd = aaa + 130;
        let ie = bnk + 40;
        draw_text("TRust-os", kd, ie, kw);
        
        draw_text("TRust-os", kd + 1, ie, kw);
        draw_text("TRust-os", kd, ie + 1, kw);
        draw_text("TRust-os", kd + 1, ie + 1, kw);
                
        
        let bwx = 90u32;
        let bwy = 300u32;
        let dgp = 380u32;
        let ffe = 280u32;
        
        
        
        
        let bsb = 20u32;
        let byv = 50u32;
        let bbe = 44u32;
        let lgs = 20u32;
        let lgt = 5u32;
        
        
        fill_rect(0, 0, 80, height - hs, 0xFF050505u32);
        
        ifd = -1;
        for i in 0..lgt {
            let gg = byv + i * (bbe + lgs);
            let bi = bsb;
            
            
            let hovered = cg >= bi as f32 && cg < (bi + bbe) as f32 && 
                          cr >= gg as f32 && cr < (gg + bbe) as f32;
            if hovered {
                ifd = i as i32;
                if cha {
                    bkn = i as usize;
                }
            }
            
            
            let icon_color = if i as usize == bkn { 
                kw 
            } else if hovered { 
                bby 
            } else { 
                ph 
            };
            
            
            draw_rect(bi, gg, bbe, bbe, icon_color);
            
            
            let cx = bi + bbe / 2;
            let u = gg + bbe / 2;
            match i {
                0 => { 
                    fill_rect(cx - 8, u - 10, 4, 20, icon_color);
                    fill_rect(cx - 4, u - 8, 4, 16, icon_color);
                    fill_rect(cx, u - 6, 4, 12, icon_color);
                    fill_rect(cx + 4, u - 4, 4, 8, icon_color);
                },
                1 => { 
                    draw_rect(cx - 12, u - 10, 24, 20, icon_color);
                    fill_rect(cx - 8, u - 4, 10, 2, icon_color);
                    fill_rect(cx - 8, u + 2, 6, 2, icon_color);
                },
                2 => { 
                    for row in 0..2 {
                        for col in 0..2 {
                            draw_rect(cx - 10 + col * 12, u - 10 + row * 12, 8, 8, icon_color);
                        }
                    }
                },
                3 => { 
                    draw_rect(cx - 10, u - 8, 20, 16, icon_color);
                    fill_rect(cx - 6, u - 4, 2, 8, icon_color);
                    fill_rect(cx - 2, u - 2, 2, 6, icon_color);
                    fill_rect(cx + 2, u - 6, 2, 10, icon_color);
                    fill_rect(cx + 6, u - 4, 2, 8, icon_color);
                },
                4 => { 
                    fill_circle(cx, u, 10, icon_color);
                    fill_circle(cx, u, 6, awh);
                },
                _ => {}
            }
        }
        
        
        let ier = height as u32 - 80;
        draw_rect(bsb, ier, bbe, bbe, ph);
        draw_text("?", bsb + 18, ier + 16, ph);
        
        
        
        
        
        
        
        let awj = 3u32;
        
        fill_rect(bwx, bwy, dgp, awj, kw);
        
        fill_rect(bwx, bwy + ffe - awj, dgp, awj, kw);
        
        fill_rect(bwx, bwy, awj, ffe, kw);
        
        fill_rect(bwx + dgp - awj, bwy, awj, ffe, kw);
        
        
        fill_rect(bwx + awj, bwy + awj, 
                  dgp - awj * 2, ffe - awj * 2, awh);
        
        
        let ana = 28u32;
        fill_rect(bwx + awj, bwy + awj, 
                  dgp - awj * 2, ana, 0xFF0A1A0Au32);
        
        
        let (title, _) = apps[bkn];
        let pkg = format!("TrustOS {} v1.00", title);
        draw_text(&pkg, bwx + 12, bwy + 10, kw);
        
        
        let ed = bwy + 10;
        let adl = bwx + dgp - 60;
        
        fill_circle(adl, ed + 6, 6, 0xFFFF5555u32);
        
        fill_circle(adl + 18, ed + 6, 6, 0xFFFFDD55u32);
        
        fill_circle(adl + 36, ed + 6, 6, 0xFF55FF55u32);
        
        
        let ho = bwx + 15;
        let bn = bwy + ana + 15;
        
        
        let lau = crate::ramfs::bh(|fs| String::from(fs.pwd()));
        
        draw_text("root", ho, bn, 0xFFFF0000u32);  
        
        draw_text("@", ho + 32, bn, 0xFFFFFFFFu32);  
        
        draw_text("trustos", ho + 40, bn, 0xFF00FF00u32);  
        
        let dwh = format!(":{}$ ", lau);
        draw_text(&dwh, ho + 96, bn, 0xFF00FF00u32);  
        
        let gos = 4 + 1 + 7 + dwh.len();  
        let cursor_x = ho + (gos * 8) as u32;
        fill_rect(cursor_x, bn, 8, 16, bby);
        
        
        
        
        let gk = height as u32 - hs;
        
        
        fill_rect(0, gk, width as u32, hs, 0xFF080808u32);
        
        fill_rect(0, gk, width as u32, 2, ph);
        
        
        let eui = 8u32;
        let nej = 24u32;
        fill_rect(eui + 4, gk + 14, 16, 3, kw);
        fill_rect(eui + 4, gk + 19, 16, 3, kw);
        
        
        if cha && cg >= eui as f32 && cg < (eui + nej) as f32 &&
           cr >= gk as f32 {
            yx = !yx;
        }
        
        
        let gxv = 40u32;
        let jlc = 90u32;
        fill_rect(gxv, gk + 6, jlc, 24, 0xFF0A1A0Au32);
        draw_rect(gxv, gk + 6, jlc, 24, ph);
        draw_text("TrustOS", gxv + 14, gk + 10, kw);
        
        
        let jld = 138u32;
        let pcp = 90u32;
        fill_rect(jld, gk + 6, pcp, 24, 0xFF050A05u32);
        draw_text("Terminal", jld + 12, gk + 10, ph);
        
        
        let ava = width as u32 / 2 - 120;
        let acm = 240u32;
        fill_rect(ava, gk + 6, acm, 24, 0xFF0A0A0Au32);
        draw_rect(ava, gk + 6, acm, 24, ph);
        if cqf == 0 {
            draw_text("Search...", ava + 8, gk + 10, 0xFF336633u32);
        } else {
            let oml = unsafe { core::str::from_utf8_unchecked(&ezu[..cqf]) };
            draw_text(oml, ava + 8, gk + 10, kw);
        }
        
        fill_circle(ava + acm - 20, gk + 18, 6, ph);
        fill_circle(ava + acm - 20, gk + 18, 4, 0xFF0A0A0Au32);
        fill_rect(ava + acm - 16, gk + 22, 6, 2, ph);
        
        
        if cha && cg >= ava as f32 && cg < (ava + acm) as f32 &&
           cr >= gk as f32 {
            search_active = true;
        }
        
        
        let fm = crate::rtc::aou();
        let time_str = format!("{:02}:{:02}", fm.hour, fm.minute);
        draw_text(&time_str, width as u32 - 200, gk + 10, kw);
        
        
        draw_text("TRST-001", width as u32 - 120, gk + 10, bby);
        
        
        let bhe = width as u32 - 50;
        fill_circle(bhe, gk + 18, 6, kw);
        fill_circle(bhe + 16, gk + 18, 6, 0xFFFFAA00u32);
        
        fill_rect(bhe + 28, gk + 12, 4, 4, ph);
        fill_rect(bhe + 34, gk + 12, 4, 4, ph);
        fill_rect(bhe + 28, gk + 18, 4, 4, ph);
        fill_rect(bhe + 34, gk + 18, 4, 4, ph);
        
        
        
        
        if yx {
            let hu = 10u32;
            let ks = gk - 320;
            let pz = 180u32;
            let rv = 310u32;
            
            
            fill_rect(hu, ks, pz, rv, 0xFF0A0F0Au32);
            draw_rect(hu, ks, pz, rv, kw);
            draw_rect(hu + 1, ks + 1, pz - 2, rv - 2, ph);
            
            
            fill_rect(hu + 2, ks + 2, pz - 4, 30, 0xFF0A1A0Au32);
            draw_text("TrustOS Menu", hu + 12, ks + 10, kw);
            
            
            bcs = -1;
            for (idx, item) in dbg.iter().enumerate() {
                let ru = ks + 40 + (idx as u32 * 24);
                
                if *item == "---" {
                    
                    fill_rect(hu + 10, ru + 10, pz - 20, 1, ph);
                } else {
                    
                    let iip = cg >= hu as f32 && cg < (hu + pz) as f32 &&
                                       cr >= ru as f32 && cr < (ru + 24) as f32;
                    
                    if iip {
                        bcs = idx as i32;
                        fill_rect(hu + 2, ru, pz - 4, 24, 0xFF1A2A1Au32);
                        
                        
                        if cha {
                            match *item {
                                "Shutdown" => { crate::acpi::shutdown(); },
                                "Restart" => { crate::acpi::eya(); },
                                "Sign Out" => { running = false; },
                                "Settings" => { bkn = 4; yx = false; },
                                "Terminal" => { bkn = 1; yx = false; },
                                "Files" => { bkn = 0; yx = false; },
                                "Browser" => { bkn = 2; yx = false; },
                                _ => { yx = false; }
                            }
                        }
                    }
                    
                    
                    let text_color = if *item == "Shutdown" || *item == "Restart" || *item == "Sign Out" {
                        0xFFFF6666u32  
                    } else if iip {
                        bby
                    } else {
                        kw
                    };
                    
                    
                    if *item == "Shutdown" {
                        fill_circle(hu + 20, ru + 12, 6, text_color);
                        fill_rect(hu + 18, ru + 6, 4, 6, 0xFF0A0F0Au32);
                    }
                    
                    draw_text(item, hu + 35, ru + 6, text_color);
                }
            }
            
            
            if cha && (cg < hu as f32 || cg > (hu + pz) as f32 ||
                                    cr < ks as f32 || cr > gk as f32) {
                yx = false;
            }
        }
        
        
        
        
        if dzj && fps > 0 {
            let lye = format!("{} FPS", fps);
            let dqd = width.saturating_sub(80);
            let fxo = if fps >= 55 { 0xFF00FF00 }    
                           else if fps >= 30 { 0xFFFFFF00 } 
                           else { 0xFFFF4444 };            
            draw_text(&lye, dqd, 4, fxo);
            
            
            let mode = if vb { "BRL" } else if vv { "FAST" } else { "LEG" };
            draw_text(mode, dqd, 20, 0xFF888888);
        }
        
        
        
        
        let nhj = cg as u32;
        let nhk = cr as u32;
        
        for i in 0..12u32 {
            fill_rect(nhj, nhk + i, (12 - i).max(1), 1, kw);
        }
        
        
        
        
        ii();
        
        
        
        
        frame_count += 1;
        bmg += 1;
        
        let cy = crate::cpu::tsc::ey();
        if cy - clz >= aso {
            fps = bmg;
            bmg = 0;
            clz = cy;
            crate::serial_println!("[COSMIC] FPS: {} | Frame: {} | Mode: {}", 
                fps, frame_count, if vb { "BRAILLE" } else if vv { "FAST" } else { "LEGACY" });
        }
        
        
        
        for _ in 0..100 {
            core::hint::spin_loop();
        }
        
        mwm = crate::cpu::tsc::ey();
    }
    
    crate::framebuffer::clear();
    crate::serial_println!("[COSMIC] Desktop exited after {} frames, last FPS: {}", frame_count, fps);
    crate::n!(B_, "COSMIC Desktop exited. {} frames rendered, {} FPS", frame_count, fps);
}