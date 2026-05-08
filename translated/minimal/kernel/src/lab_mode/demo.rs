









extern crate alloc;

use alloc::string::String;
use alloc::format;




#[inline]
fn bih() -> u64 {
    crate::time::uptime_ms()
}



#[derive(Clone, Copy)]
enum Pos {
    Center,
    Bs(usize),   
    BlackScreen,    
}



struct Bn {
    lines: &'static [&'static str],
    highlights: &'static [&'static str],
    pos: Pos,
    focus: Option<usize>,
    
    dur_ms: u64,
    
    big: bool,
}




const FI_: u32 = 28;
const CWI_: u32 = 28;

const AEJ_: u64 = 350; 

const Ds: &[Bn] = &[
    
    Bn { lines: &[""],
        highlights: &[], pos: Pos::BlackScreen, focus: None,
        dur_ms: 800, big: true },
    Bn { lines: &["Are you ready",  "to see the Matrix, Neo?"],
        highlights: &["Matrix", "Neo"], pos: Pos::BlackScreen, focus: None,
        dur_ms: 2200, big: true },

    
    Bn { lines: &["You don't understand", "how your computer works."],
        highlights: &["don't", "computer"], pos: Pos::Center, focus: None,
        dur_ms: 2000, big: true },
    Bn { lines: &["This is TrustLab."],
        highlights: &["TrustLab"], pos: Pos::Center, focus: None,
        dur_ms: 1500, big: true },

    
    Bn { lines: &["HARDWARE STATUS"],
        highlights: &["HARDWARE"], pos: Pos::Bs(0), focus: Some(0),
        dur_ms: 500, big: true },
    Bn { lines: &["Real CPU. Real memory.", "Raw silicon."],
        highlights: &["CPU", "memory", "Raw"], pos: Pos::Bs(0), focus: Some(0),
        dur_ms: 1500, big: true },
    Bn { lines: &["What Task Manager", "will never show you."],
        highlights: &["never"], pos: Pos::Bs(0), focus: Some(0),
        dur_ms: 1200, big: true },

    
    Bn { lines: &["LIVE KERNEL TRACE"],
        highlights: &["KERNEL", "TRACE"], pos: Pos::Bs(1), focus: Some(1),
        dur_ms: 500, big: true },
    Bn { lines: &["Every interrupt.", "Every syscall."],
        highlights: &["interrupt", "syscall"], pos: Pos::Bs(1), focus: Some(1),
        dur_ms: 1500, big: true },
    Bn { lines: &["Raw kernel truth."],
        highlights: &["Raw", "truth"], pos: Pos::Bs(1), focus: Some(1),
        dur_ms: 1200, big: true },

    
    Bn { lines: &["EXECUTION PIPELINE"],
        highlights: &["PIPELINE"], pos: Pos::Bs(5), focus: Some(5),
        dur_ms: 500, big: true },
    Bn { lines: &["Watch data flow through", "the kernel in real time."],
        highlights: &["flow", "real time"], pos: Pos::Bs(5), focus: Some(5),
        dur_ms: 1500, big: true },

    
    Bn { lines: &["HEX EDITOR"],
        highlights: &["HEX"], pos: Pos::Bs(6), focus: Some(6),
        dur_ms: 500, big: true },
    Bn { lines: &["Raw bytes. Color-coded."],
        highlights: &["Raw", "bytes"], pos: Pos::Bs(6), focus: Some(6),
        dur_ms: 1300, big: true },

    
    Bn { lines: &["FILE SYSTEM"],
        highlights: &["FILE"], pos: Pos::Bs(3), focus: Some(3),
        dur_ms: 500, big: true },
    Bn { lines: &["Live filesystem. In memory."],
        highlights: &["Live", "memory"], pos: Pos::Bs(3), focus: Some(3),
        dur_ms: 1200, big: true },

    
    Bn { lines: &["TRUSTLANG EDITOR"],
        highlights: &["TRUSTLANG"], pos: Pos::Bs(4), focus: Some(4),
        dur_ms: 500, big: true },
    Bn { lines: &["Write code inside the kernel.", "Execute it."],
        highlights: &["code", "kernel", "Execute"], pos: Pos::Bs(4), focus: Some(4),
        dur_ms: 1500, big: true },

    
    Bn { lines: &["52 COMMANDS"],
        highlights: &["52"], pos: Pos::Bs(2), focus: Some(2),
        dur_ms: 500, big: true },
    Bn { lines: &["Full shell. All built-in."],
        highlights: &["shell", "built-in"], pos: Pos::Bs(2), focus: Some(2),
        dur_ms: 1200, big: true },

    
    Bn { lines: &["TrustLab is not a tool."],
        highlights: &["not"], pos: Pos::Center, focus: None,
        dur_ms: 1500, big: true },
    Bn { lines: &["Bare metal. Rust. Open source."],
        highlights: &["Rust", "Open source"], pos: Pos::Center, focus: None,
        dur_ms: 1500, big: true },
    Bn { lines: &["Boot it. Break it.", "Understand it."],
        highlights: &["Boot", "Break", "Understand"], pos: Pos::Center, focus: None,
        dur_ms: 2000, big: true },
];



pub struct DemoState {
    pub active: bool,
    pub current_slide: usize,
    
    slide_start_ms: u64,
    
    demo_start_ms: u64,
    
    seed: u32,
    
    last_slide: usize,
    
    pub tick_in_slide: u64,
    pub total_ticks: u64,
}

impl DemoState {
    pub fn new() -> Self {
        Self {
            active: false,
            current_slide: 0,
            slide_start_ms: 0,
            demo_start_ms: 0,
            seed: 12345,
            last_slide: usize::MAX,
            tick_in_slide: 0,
            total_ticks: 0,
        }
    }

    pub fn start(&mut self) {
        self.active = true;
        self.current_slide = 0;
        self.demo_start_ms = bih();
        self.slide_start_ms = self.demo_start_ms;
        self.seed = (self.demo_start_ms & 0xFFFF) as u32 ^ 0xA5A5;
        self.last_slide = usize::MAX;
        self.tick_in_slide = 0;
        self.total_ticks = 0;
        let total_ms: u64 = Ds.iter().map(|j| j.dur_ms).sum();
        crate::serial_println!("[DEMO] Started! now_ms={} total_script={}ms slides={}",
            self.demo_start_ms, total_ms, Ds.len());
    }

    pub fn stop(&mut self) {
        self.active = false;
    }

    
    pub fn tick(&mut self) -> Option<usize> {
        if !self.active { return None; }
        self.total_ticks += 1;

        let t = bih();
        let total_elapsed_ms = t - self.demo_start_ms;

        if self.current_slide >= Ds.len() {
            self.stop();
            return None;
        }

        
        let mut ejg: u64 = 0;
        for i in 0..=self.current_slide {
            if i < Ds.len() {
                ejg += Ds[i].dur_ms;
            }
        }

        
        if self.total_ticks % 200 == 0 {
            crate::serial_println!("[DEMO] slide={} total={}ms deadline={}ms ticks={}",
                self.current_slide, total_elapsed_ms, ejg, self.total_ticks);
        }

        if total_elapsed_ms >= ejg {
            
            crate::serial_println!("[DEMO] -> next slide {} at total={}ms (deadline={}ms)",
                self.current_slide + 1, total_elapsed_ms, ejg);
            self.current_slide += 1;
            self.slide_start_ms = t;
            self.tick_in_slide = 0;
            if self.current_slide >= Ds.len() {
                self.stop();
                return None;
            }
            return Ds[self.current_slide].focus;
        }

        self.tick_in_slide += 1;

        
        if self.last_slide != self.current_slide {
            self.last_slide = self.current_slide;
            return Ds[self.current_slide].focus;
        }
        None
    }

    
    pub fn handle_key(&mut self, key: u8) -> bool {
        if !self.active { return false; }
        match key {
            0x1B => { self.stop(); true }
            0x20 => {
                self.current_slide += 1;
                self.slide_start_ms = bih();
                self.tick_in_slide = 0;
                if self.current_slide >= Ds.len() { self.stop(); }
                true
            }
            _ => true
        }
    }

    
    fn slide_elapsed_ms(&self) -> u64 {
        bih() - self.slide_start_ms
    }

    
    fn total_elapsed_ms(&self) -> u64 {
        bih() - self.demo_start_ms
    }

    
    fn pseudo_rand(&self, ua: u32) -> u32 {
        let mut v = self.seed.wrapping_add(self.total_ticks as u32).wrapping_add(ua);
        v ^= v << 13;
        v ^= v >> 17;
        v ^= v << 5;
        v
    }
}



struct Gx { x: i32, y: i32, w: u32, h: u32 }

fn npu(wx: i32, wy: i32, ca: u32, er: u32) -> [Gx; 7] {
    let cx = wx + 2;
    let u = wy + FI_ as i32 + 2;
    let aq = ca.saturating_sub(4);
    let ch = er.saturating_sub(FI_ + 4);
    let gap = 4u32;
    let en = ch.saturating_sub(CWI_ + gap);
    let col_w = aq.saturating_sub(gap * 2) / 3;
    let ep = en.saturating_sub(gap) / 2;
    let bm = cx;
    let x1 = cx + col_w as i32 + gap as i32;
    let x2 = cx + (col_w as i32 + gap as i32) * 2;
    let az = u;
    let y1 = u + ep as i32 + gap as i32;
    let bye = (aq as i32 - (x2 - cx)).max(40) as u32;
    let crz = ep.saturating_sub(gap) / 2;
    let gnc = ep.saturating_sub(crz + gap);
    let gnd = az + crz as i32 + gap as i32;

    [
        Gx { x: bm, y: az, w: col_w, h: ep },
        Gx { x: x1, y: az, w: col_w, h: crz },
        Gx { x: x2, y: az, w: bye, h: ep },
        Gx { x: bm, y: y1, w: col_w, h: ep },
        Gx { x: x1, y: y1, w: col_w, h: ep },
        Gx { x: x1, y: gnd, w: col_w, h: gnc },
        Gx { x: x2, y: y1, w: bye, h: ep },
    ]
}




fn total_duration_ms() -> u64 {
    Ds.iter().map(|j| j.dur_ms).sum()
}


fn qxe(ae: usize) -> u64 {
    Ds[..ae].iter().map(|j| j.dur_ms).sum()
}


pub fn ljz(state: &DemoState, wx: i32, wy: i32, ca: u32, er: u32) {
    if !state.active { return; }
    if state.current_slide >= Ds.len() { return; }

    let boz = &Ds[state.current_slide];
    let elapsed_ms = state.slide_elapsed_ms();
    let scale: u32 = 3; 

    let ehs = 8i32 * scale as i32;
    let gfl = 16i32 * scale as i32 + 8;

    
    let bhi = matches!(boz.pos, Pos::BlackScreen);
    if bhi {
        crate::framebuffer::fill_rect(wx.max(0) as u32, wy.max(0) as u32, ca, er, 0xFF000000);
        ljs(state, wx, wy, ca, er);
    }

    
    if !bhi && elapsed_ms < AEJ_ {
        liz(state, wx, wy, ca, er, elapsed_ms);
    }

    
    let cxp = 200u64;
    let alpha = if elapsed_ms < cxp {
        (elapsed_ms * 255 / cxp).min(255) as u32
    } else if elapsed_ms > boz.dur_ms.saturating_sub(cxp) {
        let rem = boz.dur_ms.saturating_sub(elapsed_ms);
        (rem * 255 / cxp).min(255) as u32
    } else {
        255u32
    };
    if alpha < 8 { return; }

    
    let mkg = boz.lines.iter().any(|l| !l.is_empty());
    if !mkg { return; }

    
    let aoo = boz.lines.iter().map(|l| l.len()).max().unwrap_or(1);
    let fjn = aoo as i32 * ehs;
    let fjm = boz.lines.len() as i32 * gfl;

    
    let aoq = npu(wx, wy, ca, er);

    let (bu, ty) = match boz.pos {
        Pos::Center | Pos::BlackScreen => {
            (wx + (ca as i32 - fjn) / 2,
             wy + (er as i32 - fjm) / 2)
        }
        Pos::Bs(idx) => {
            let aa = &aoq[idx.min(6)];
            let bx = aa.x + (aa.w as i32 - fjn) / 2;
            let dc = aa.y + (aa.h as i32 - fjm) / 2;
            (bx.max(wx + 4).min(wx + ca as i32 - fjn - 4),
             dc.max(wy + 32).min(wy + er as i32 - fjm - 4))
        }
    };

    
    let mut ly = ty;
    for line in boz.lines {
        if line.is_empty() { ly += gfl; continue; }
        
        eko(bu + 2, ly + 2, line, boz.highlights, scale, alpha * 2 / 3, true);
        
        eko(bu, ly, line, boz.highlights, scale, alpha, false);
        ly += gfl;
    }

    
    let total_ms = total_duration_ms();
    let hvh = state.total_elapsed_ms().min(total_ms);
    let nyq = (hvh as u32 * ca / total_ms.max(1) as u32).min(ca);
    let azc = (wy + er as i32 - 3).max(0) as u32;
    crate::framebuffer::fill_rect(wx.max(0) as u32, azc, ca, 3, 0xFF1C2128);
    crate::framebuffer::fill_rect(wx.max(0) as u32, azc, nyq, 3, 0xFFFF2020);

    
    let im = hvh / 1000;
    let fdj = total_ms / 1000;
    let timer = format!("{}s/{}s", im, fdj);
    let fcm = super::ew();
    let pjw = wx + ca as i32 - (timer.len() as i32 * fcm) - 8;
    super::eh(pjw, azc as i32 - 16, &timer, dim_color(0xFF8B949E, alpha));

    
    super::eh(wx + 8, azc as i32 - 16, "[Esc] stop  [Space] next",
        dim_color(0xFF484F58, alpha));
}




fn liz(state: &DemoState, wx: i32, wy: i32, ca: u32, er: u32, elapsed_ms: u64) {
    let intensity = ((AEJ_ - elapsed_ms) * 255 / AEJ_).min(255) as u32;
    if intensity < 10 { return; }

    
    let dlc = 12u32;
    let num_cols = ca / dlc;
    let chars = b"01?#@!$%&*<>{}[]|/\\~";

    for c in 0..num_cols {
        let seed = state.pseudo_rand(c * 7919 + 31);
        let chm = wx + (c * dlc) as i32;
        let speed = (seed % 5 + 2) as i32;
        let offset = (state.total_ticks as i32 * speed + seed as i32) % er as i32;

        
        let count = (seed % 4 + 3) as i32;
        for ay in 0..count {
            let u = wy + (offset + ay * 14) % er as i32;
            let cuy = ((seed >> (ay as u32 * 3)) as usize + state.total_ticks as usize) % chars.len();
            let ch = chars[cuy] as char;

            
            let brightness = if ay == 0 { intensity } else { intensity * (count - ay) as u32 / count as u32 / 2 };
            let g = (brightness * 255 / 255).min(255);
            let color = 0xFF000000 | ((g / 4) << 16) | (g << 8) | (g / 4);

            let mut buf = [0u8; 1];
            buf[0] = ch as u8;
            if let Ok(j) = core::str::from_utf8(&buf) {
                crate::graphics::scaling::aat(chm, u, j, color, 1);
            }
        }
    }

    
    let egb = (intensity / 40).min(6);
    for b in 0..egb {
        let egg = state.pseudo_rand(b * 1337 + 42);
        let gk = wy + (egg % er) as i32;
        let ek = (egg % (ca / 2)) + 20;
        let pv = wx + (egg % (ca / 3)) as i32;
        let g = (egg % 100 + 50).min(200);
        let color = 0xFF000000 | ((g / 6) << 16) | (g << 8) | ((g / 4) & 0xFF);
        crate::framebuffer::fill_rect(
            pv.max(0) as u32, gk.max(0) as u32,
            ek.min(ca), 2, color,
        );
    }
}




fn ljs(state: &DemoState, wx: i32, wy: i32, ca: u32, er: u32) {
    let dlc = 10u32;
    let num_cols = ca / dlc;
    let chars = b"01?#@$%&*<>{}[]|/\\~:;_=+-.";

    for c in 0..num_cols {
        let seed = state.pseudo_rand(c * 6271 + 17);
        let chm = wx + (c * dlc) as i32;
        let speed = (seed % 3 + 1) as i32;
        let offset = (state.total_ticks as i32 * speed + (seed as i32 * 37)) % (er as i32 * 2);

        
        let count = (seed % 8 + 8) as i32;
        for ay in 0..count {
            let u = wy + (offset + ay * 12) % er as i32;
            let cuy = ((seed >> (ay as u32 * 2 + 1)) as usize + state.total_ticks as usize) % chars.len();
            let ch = chars[cuy];

            
            let brightness = if ay == 0 { 255u32 }
                else { (200u32).saturating_sub(ay as u32 * 18) };
            let g = brightness.min(255);
            let r = if ay == 0 { g / 2 } else { 0 };
            let b = if ay == 0 { g / 3 } else { 0 };
            let color = 0xFF000000 | (r << 16) | (g << 8) | b;

            let mut buf = [0u8; 1];
            buf[0] = ch;
            if let Ok(j) = core::str::from_utf8(&buf) {
                crate::graphics::scaling::aat(chm, u, j, color, 1);
            }
        }
    }
}





fn eko(x: i32, y: i32, line: &str, highlights: &[&str],
                         scale: u32, alpha: u32, shadow: bool)
{
    let ehs = 8i32 * scale as i32;
    let gus = dim_color(0xFF000000, alpha);
    let normal = dim_color(0xFFFF3030, alpha);   
    let accent = dim_color(0xFFFF6060, alpha);   
    let cyz  = dim_color(0xFF00FF41, alpha);   

    let mut cx = x;
    let mut bxa = String::new();

    let mut chars = line.chars().peekable();
    while let Some(&ch) = chars.peek() {
        if ch.is_alphanumeric() || ch == '\'' {
            bxa.push(ch);
            chars.next();
        } else {
            if !bxa.is_empty() {
                let col = if shadow { gus }
                          else { iur(&bxa, highlights, normal, accent, cyz) };
                crate::graphics::scaling::aat(cx, y, &bxa, col, scale);
                cx += bxa.len() as i32 * ehs;
                bxa.clear();
            }
            let mut buf = [0u8; 4];
            let j = ch.encode_utf8(&mut buf);
            let col = if shadow { gus } else { normal };
            crate::graphics::scaling::aat(cx, y, j, col, scale);
            cx += ehs;
            chars.next();
        }
    }
    if !bxa.is_empty() {
        let col = if shadow { gus }
                  else { iur(&bxa, highlights, normal, accent, cyz) };
        crate::graphics::scaling::aat(cx, y, &bxa, col, scale);
    }
}


fn iur(fx: &str, highlights: &[&str], normal: u32, accent: u32, cyz: u32) -> u32 {
    for hl in highlights {
        if fx.eq_ignore_ascii_case(hl) { return accent; }
    }
    match fx {
        "kernel" | "Kernel" | "KERNEL" => cyz,
        "TrustLab" | "TRUSTLAB" | "TrustOS" | "TRUSTOS" => accent,
        "Matrix" | "MATRIX" => cyz,
        "Neo" | "NEO" => cyz,
        "Rust" | "RUST" => 0xFFD18616,
        _ => normal,
    }
}


fn dim_color(color: u32, alpha: u32) -> u32 {
    let a = alpha.min(255);
    let r = ((color >> 16) & 0xFF) * a / 255;
    let g = ((color >> 8) & 0xFF) * a / 255;
    let b = (color & 0xFF) * a / 255;
    0xFF000000 | (r << 16) | (g << 8) | b
}
