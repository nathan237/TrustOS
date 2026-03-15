















use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;

use crate::framebuffer::{
    Q_, MG_, C_, D_, A_,
    B_, L_, G_, CD_, DF_,
};


const CFU_: usize = 64;

struct EditorState {
    ak: Vec<String>,
    qu: usize,    
    hn: usize,    
    dbt: usize,    
    eic: usize,    
    it: String,
    bvl: bool,
    aoc: String,
    mho: u64,
    fex: Vec<String>,    
    bsu: Vec<UndoEntry>,
    bla: String,
    aqk: bool,
    jsv: usize,
    idd: usize,
}

#[derive(Clone)]
enum UndoEntry {
    Auh { br: usize, bj: usize },
    Aqr { br: usize, bj: usize, bm: char },
    Bjo { br: usize },
    Bek { br: usize, ca: String },
    Bth { br: usize, bj: usize },
    Auq { br: usize, bj: usize },
}

impl EditorState {
    fn new(it: &str) -> Self {
        let ec = crate::framebuffer::z() as usize / 8;
        let lk = crate::framebuffer::ac() as usize / 16;
        
        Self {
            ak: vec![String::new()],
            qu: 0,
            hn: 0,
            dbt: 0,
            eic: 0,
            it: String::from(it),
            bvl: false,
            aoc: String::from("Ctrl+S = Save | Ctrl+X = Exit | Ctrl+F = Find | Ctrl+G = Goto"),
            mho: crate::time::lc(),
            fex: Vec::new(),
            bsu: Vec::new(),
            bla: String::new(),
            aqk: true,
            jsv: lk,
            idd: ec,
        }
    }

    
    fn cxg(&self) -> usize {
        if self.jsv > 3 { self.jsv - 3 } else { 1 }
    }

    
    fn hmf(&self) -> usize {
        let jfi = self.ak.len();
        let ird = if jfi < 10 { 1 }
            else if jfi < 100 { 2 }
            else if jfi < 1000 { 3 }
            else if jfi < 10000 { 4 }
            else { 5 };
        ird + 2 
    }

    fn bru(&mut self, bt: UndoEntry) {
        if self.bsu.len() >= CFU_ {
            self.bsu.remove(0);
        }
        self.bsu.push(bt);
    }

    fn cbs(&mut self, fr: &str) {
        self.aoc = String::from(fr);
        self.mho = crate::time::lc();
    }
}


pub(super) fn rgm(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: nano <filename>");
        crate::println!("       edit <filename>");
        return;
    }

    let it = n[0];
    let mut g = EditorState::new(it);

    
    dsu(&mut g);

    
    crate::framebuffer::clear();
    nnq(&g);

    
    while g.aqk {
        if let Some(bs) = crate::keyboard::auw() {
            vr(&mut g, bs);
            wes(&mut g);
            nnq(&g);
        } else {
            core::hint::hc();
        }
    }

    
    crate::framebuffer::clear();
    crate::framebuffer::bld(0, 0);
}

fn dsu(g: &mut EditorState) {
    let path = &g.it;
    
    
    let ca: Option<String> = if path.cj("/mnt/") || path.cj("/dev/") || path.cj("/proc/") {
        crate::vfs::lxu(path).bq()
    } else {
        
        crate::ramfs::fh(|fs| {
            fs.mq(path)
                .map(|bf| String::from(core::str::jg(bf).unwrap_or("")))
                .bq()
        })
    };

    match ca {
        Some(ref text) => {
            g.ak = text.ak().map(|dm| String::from(dm)).collect();
            if g.ak.is_empty() {
                g.ak.push(String::new());
            }
            g.cbs(&format!("Opened \"{}\" — {} lines", g.it, g.ak.len()));
        }
        None => {
            g.ak = vec![String::new()];
            g.cbs(&format!("New file: \"{}\"", g.it));
        }
    }
}

fn ftm(g: &mut EditorState) {
    let ca = g.ak.rr("\n");
    let path = &g.it;
    
    let result = if path.cj("/mnt/") || path.cj("/dev/") {
        crate::vfs::ns(path, ca.as_bytes()).jd(|_| "VFS write error")
    } else {
        crate::ramfs::fh(|fs| {
            if !fs.aja(path) {
                let _ = fs.touch(path);
            }
            fs.ns(path, ca.as_bytes())
                .jd(|_| "RamFS write error")
        })
    };

    match result {
        Ok(()) => {
            g.bvl = false;
            g.cbs(&format!("Saved \"{}\" — {} lines, {} bytes", 
                g.it, g.ak.len(), ca.len()));
        }
        Err(aa) => {
            g.cbs(&format!("ERROR: Could not save: {}", aa));
        }
    }
}

fn wes(g: &mut EditorState) {
    let cxg = g.cxg();
    
    
    if g.qu < g.dbt {
        g.dbt = g.qu;
    }
    if g.qu >= g.dbt + cxg {
        g.dbt = g.qu - cxg + 1;
    }
    
    
    let dhu = g.hmf();
    let jvt = if g.idd > dhu { g.idd - dhu } else { 1 };
    if g.hn < g.eic {
        g.eic = g.hn;
    }
    if g.hn >= g.eic + jvt {
        g.eic = g.hn - jvt + 1;
    }
}

fn nnq(g: &EditorState) {
    let ec = g.idd;
    let lk = g.jsv;
    let cxg = g.cxg();
    let dhu = g.hmf();

    
    let dq = if g.bvl {
        format!(" TrustEdit — {}  [modified]", g.it)
    } else {
        format!(" TrustEdit — {}", g.it)
    };
    let dh = vau(&dq, ec);
    
    crate::framebuffer::ah(0, 0, ec as u32 * 8, 16, 0xFF1A1A2E);
    crate::framebuffer::cb(&dh, 0, 0, C_);

    
    for dvl in 0..cxg {
        let irk = g.dbt + dvl;
        let c = ((dvl + 1) * 16) as u32;

        
        crate::framebuffer::ah(0, c, ec as u32 * 8, 16, 0xFF0D0D1A);

        if irk < g.ak.len() {
            
            let csd = format!("{:>width$} ", irk + 1, z = dhu - 2);
            
            crate::framebuffer::ah(0, c, dhu as u32 * 8, 16, 0xFF15152A);
            
            if irk == g.qu {
                crate::framebuffer::ah(dhu as u32 * 8, c, (ec - dhu) as u32 * 8, 16, 0xFF1A1A30);
            }
            crate::framebuffer::cb(&csd, 0, c, L_);

            
            let line = &g.ak[irk];
            let jvt = ec - dhu;
            let ign: String = if g.eic < line.len() {
                let ci = core::cmp::v(line.len(), g.eic + jvt);
                String::from(&line[g.eic..ci])
            } else {
                String::new()
            };
            
            
            sfx(&ign, (dhu * 8) as u32, c, &g.it);
        } else {
            
            crate::framebuffer::ah(0, c, dhu as u32 * 8, 16, 0xFF15152A);
            crate::framebuffer::cb("~", 0, c, CD_);
        }
    }

    
    let uo = ((cxg + 1) * 16) as u32;
    crate::framebuffer::ah(0, uo, ec as u32 * 8, 16, 0xFF2A2A4A);
    let fd = format!(" Ln {}, Col {} | {} lines | {}", 
        g.qu + 1, 
        g.hn + 1,
        g.ak.len(),
        if g.bvl { "MODIFIED" } else { "saved" }
    );
    let hw = format!("{} ", rwp(&g.it));
    let ob = if ec > fd.len() + hw.len() { ec - fd.len() - hw.len() } else { 0 };
    let bli = format!("{}{:>pad$}{}", fd, "", hw, ov = ob);
    crate::framebuffer::cb(&bli, 0, uo, B_);

    
    let dti = ((cxg + 2) * 16) as u32;
    crate::framebuffer::ah(0, dti, ec as u32 * 8, 16, 0xFF0D0D1A);
    let qfy = crate::time::lc().ao(g.mho);
    if qfy < 8000 {
        crate::framebuffer::cb(&g.aoc, 0, dti, D_);
    } else {
        crate::framebuffer::cb(
            " Ctrl+S=Save  Ctrl+X=Exit  Ctrl+F=Find  Ctrl+G=Goto  Ctrl+K=Cut  Ctrl+U=Paste",
            0, dti, L_,
        );
    }

    
    let rsj = g.qu - g.dbt;
    let rsh = g.hn - g.eic + dhu;
    let cx = (rsh * 8) as u32;
    let ae = ((rsj + 1) * 16) as u32;
    
    crate::framebuffer::ah(cx, ae, 8, 16, Q_);
    
    if g.qu < g.ak.len() {
        let line = &g.ak[g.qu];
        if g.hn < line.len() {
            let bm = &line[g.hn..g.hn + 1];
            crate::framebuffer::cb(bm, cx, ae, 0xFF0D0D1A);
        }
    }
}


fn sfx(text: &str, b: u32, c: u32, it: &str) {
    let wm = it.cmm('.').next().unwrap_or("");
    let twy = oh!(wm, "rs" | "c" | "h" | "py" | "js" | "ts" | "sh" | "toml" | "json" | "cfg" | "conf");
    
    if !twy || text.is_empty() {
        crate::framebuffer::cb(text, b, c, Q_);
        return;
    }

    
    let fmj = [
        "fn ", "let ", "mut ", "pub ", "use ", "mod ", "struct ", "enum ", "impl ",
        "trait ", "type ", "const ", "static ", "match ", "if ", "else ", "for ",
        "while ", "loop ", "return ", "break ", "continue ", "async ", "await ",
        "unsafe ", "extern ", "crate ", "self ", "super ", "where ",
        "def ", "class ", "import ", "from ", "try ", "except ", "with ",
        "function ", "var ", "const ", "export ", "import ",
    ];

    let ux = text.ifa();
    
    
    if ux.cj("//") || ux.cj('#') {
        crate::framebuffer::cb(text, b, c, L_);
        return;
    }

    
    for yo in &fmj {
        if ux.cj(yo) || ux.cj(yo.eke()) {
            
            crate::framebuffer::cb(text, b, c, C_);
            return;
        }
    }

    
    if ux.contains('"') {
        crate::framebuffer::cb(text, b, c, D_);
        return;
    }

    
    if ux.contains("!(") || ux.contains("!{") {
        crate::framebuffer::cb(text, b, c, DF_);
        return;
    }

    
    crate::framebuffer::cb(text, b, c, Q_);
}

fn rwp(it: &str) -> &str {
    match it.cmm('.').next().unwrap_or("") {
        "rs" => "Rust",
        "py" => "Python",
        "js" => "JavaScript",
        "ts" => "TypeScript",
        "c" | "h" => "C",
        "cpp" | "hpp" => "C++",
        "sh" => "Shell",
        "toml" => "TOML",
        "json" => "JSON",
        "md" => "Markdown",
        "txt" => "Text",
        "cfg" | "conf" | "ini" => "Config",
        "html" => "HTML",
        "css" => "CSS",
        _ => "Plain Text",
    }
}

fn vau(e: &str, z: usize) -> String {
    if e.len() >= z {
        String::from(&e[..z])
    } else {
        let mut bd = String::from(e);
        for _ in 0..(z - e.len()) {
            bd.push(' ');
        }
        bd
    }
}

fn vr(g: &mut EditorState, bs: u8) {
    match bs {
        
        0x13 => ftm(g),
        
        
        0x18 => {
            if g.bvl {
                g.cbs("File has unsaved changes! Ctrl+S to save, Ctrl+Q to quit without saving");
            } else {
                g.aqk = false;
            }
        }

        
        0x11 => {
            g.aqk = false;
        }
        
        
        0x07 => {
            if let Some(lis) = oyf(g, "Go to line: ") {
                if let Ok(line) = lis.em().parse::<usize>() {
                    if line > 0 && line <= g.ak.len() {
                        g.qu = line - 1;
                        g.hn = 0;
                        g.cbs(&format!("Jumped to line {}", line));
                    } else {
                        g.cbs("Invalid line number");
                    }
                }
            }
        }
        
        
        0x06 => {
            let default = g.bla.clone();
            let vnf = if default.is_empty() {
                String::from("Search: ")
            } else {
                format!("Search [{}]: ", default)
            };
            if let Some(query) = oyf(g, &vnf) {
                let fm = if query.is_empty() { default } else { query };
                if !fm.is_empty() {
                    g.bla = fm.clone();
                    kwf(g, &fm);
                }
            }
        }
        
        
        0x0B => {
            if g.qu < g.ak.len() {
                let gqs = g.ak.remove(g.qu);
                g.bru(UndoEntry::Bek {
                    br: g.qu,
                    ca: gqs.clone(),
                });
                g.fex = vec![gqs];
                if g.ak.is_empty() {
                    g.ak.push(String::new());
                }
                if g.qu >= g.ak.len() {
                    g.qu = g.ak.len() - 1;
                }
                g.hn = 0;
                g.bvl = true;
                g.cbs("Line cut");
            }
        }
        
        
        0x15 => {
            if !g.fex.is_empty() {
                for (a, line) in g.fex.clone().iter().cf() {
                    g.ak.insert(g.qu + a, line.clone());
                    g.bru(UndoEntry::Bjo { br: g.qu + a });
                }
                g.bvl = true;
                g.cbs("Pasted");
            }
        }
        
        
        0x03 => {
            if g.qu < g.ak.len() {
                g.fex = vec![g.ak[g.qu].clone()];
                g.cbs("Line copied");
            }
        }
        
        
        0x1A => {
            ifu(g);
        }
        
        
        0x0C => {
            crate::framebuffer::clear();
            g.cbs("Screen refreshed");
        }
        
        
        27 => {}
        
        
        crate::keyboard::V_ => {
            if g.qu > 0 {
                g.qu -= 1;
                dow(g);
            }
        }
        crate::keyboard::U_ => {
            if g.qu + 1 < g.ak.len() {
                g.qu += 1;
                dow(g);
            }
        }
        crate::keyboard::AH_ => {
            if g.hn > 0 {
                g.hn -= 1;
            } else if g.qu > 0 {
                g.qu -= 1;
                g.hn = g.ak[g.qu].len();
            }
        }
        crate::keyboard::AI_ => {
            let ark = g.ak.get(g.qu).map(|dm| dm.len()).unwrap_or(0);
            if g.hn < ark {
                g.hn += 1;
            } else if g.qu + 1 < g.ak.len() {
                g.qu += 1;
                g.hn = 0;
            }
        }
        
        
        crate::keyboard::CQ_ => {
            g.hn = 0;
        }
        
        crate::keyboard::CP_ => {
            g.hn = g.ak.get(g.qu).map(|dm| dm.len()).unwrap_or(0);
        }
        
        
        crate::keyboard::AM_ => {
            let eeb = g.cxg();
            g.qu = g.qu.ao(eeb);
            dow(g);
        }
        
        crate::keyboard::AQ_ => {
            let eeb = g.cxg();
            g.qu = core::cmp::v(g.qu + eeb, g.ak.len().ao(1));
            dow(g);
        }
        
        
        crate::keyboard::CX_ => {
            let ark = g.ak.get(g.qu).map(|dm| dm.len()).unwrap_or(0);
            if g.hn < ark {
                let bm = g.ak[g.qu].remove(g.hn);
                g.bru(UndoEntry::Aqr { br: g.qu, bj: g.hn, bm });
                g.bvl = true;
            } else if g.qu + 1 < g.ak.len() {
                
                let next = g.ak.remove(g.qu + 1);
                g.bru(UndoEntry::Auq { br: g.qu, bj: g.hn });
                g.ak[g.qu].t(&next);
                g.bvl = true;
            }
        }
        
        
        0x08 => {
            if g.hn > 0 {
                g.hn -= 1;
                let bm = g.ak[g.qu].remove(g.hn);
                g.bru(UndoEntry::Aqr { br: g.qu, bj: g.hn, bm });
                g.bvl = true;
            } else if g.qu > 0 {
                
                let cv = g.ak.remove(g.qu);
                g.qu -= 1;
                g.hn = g.ak[g.qu].len();
                g.bru(UndoEntry::Auq { br: g.qu, bj: g.hn });
                g.ak[g.qu].t(&cv);
                g.bvl = true;
            }
        }
        
        
        0x0A => {
            let bgp = &g.ak[g.qu];
            let dlf = String::from(&bgp[g.hn..]);
            g.ak[g.qu] = String::from(&bgp[..g.hn]);
            g.bru(UndoEntry::Bth { br: g.qu, bj: g.hn });
            g.qu += 1;
            g.ak.insert(g.qu, dlf);
            g.hn = 0;
            g.bvl = true;
        }
        
        
        0x09 => {
            for _ in 0..4 {
                g.ak[g.qu].insert(g.hn, ' ');
                g.bru(UndoEntry::Auh { br: g.qu, bj: g.hn });
                g.hn += 1;
            }
            g.bvl = true;
        }
        
        
        bm if bm >= 32 && bm < 127 => {
            g.ak[g.qu].insert(g.hn, bm as char);
            g.bru(UndoEntry::Auh { br: g.qu, bj: g.hn });
            g.hn += 1;
            g.bvl = true;
        }
        
        _ => {}
    }
}

fn dow(g: &mut EditorState) {
    let ark = g.ak.get(g.qu).map(|dm| dm.len()).unwrap_or(0);
    if g.hn > ark {
        g.hn = ark;
    }
}

fn ifu(g: &mut EditorState) {
    if let Some(bt) = g.bsu.pop() {
        match bt {
            UndoEntry::Auh { br, bj } => {
                if br < g.ak.len() && bj < g.ak[br].len() {
                    g.ak[br].remove(bj);
                    g.qu = br;
                    g.hn = bj;
                }
            }
            UndoEntry::Aqr { br, bj, bm } => {
                if br < g.ak.len() {
                    g.ak[br].insert(bj, bm);
                    g.qu = br;
                    g.hn = bj + 1;
                }
            }
            UndoEntry::Bjo { br } => {
                if br < g.ak.len() {
                    g.ak.remove(br);
                    if g.ak.is_empty() {
                        g.ak.push(String::new());
                    }
                    g.qu = br.v(g.ak.len() - 1);
                    g.hn = 0;
                }
            }
            UndoEntry::Bek { br, ca } => {
                g.ak.insert(br, ca);
                g.qu = br;
                g.hn = 0;
            }
            UndoEntry::Bth { br, bj } => {
                if br + 1 < g.ak.len() {
                    let next = g.ak.remove(br + 1);
                    g.ak[br].t(&next);
                    g.qu = br;
                    g.hn = bj;
                }
            }
            UndoEntry::Auq { br, bj } => {
                if br < g.ak.len() {
                    let dlf = String::from(&g.ak[br][bj..]);
                    g.ak[br] = String::from(&g.ak[br][..bj]);
                    g.ak.insert(br + 1, dlf);
                    g.qu = br + 1;
                    g.hn = 0;
                }
            }
        }
        g.bvl = true;
        g.cbs("Undo");
    } else {
        g.cbs("Nothing to undo");
    }
}

fn kwf(g: &mut EditorState, query: &str) {
    
    let dwe = g.qu;
    let bii = g.hn + 1;
    
    for br in dwe..g.ak.len() {
        let kjw = if br == dwe { bii } else { 0 };
        if kjw < g.ak[br].len() {
            if let Some(u) = g.ak[br][kjw..].du(query) {
                g.qu = br;
                g.hn = kjw + u;
                g.cbs(&format!("Found \"{}\" at line {}", query, br + 1));
                return;
            }
        }
    }
    
    
    for br in 0..=dwe {
        let rlu = if br == dwe { bii } else { g.ak[br].len() };
        if let Some(u) = g.ak[br][..rlu.v(g.ak[br].len())].du(query) {
            g.qu = br;
            g.hn = u;
            g.cbs(&format!("Found \"{}\" at line {} (wrapped)", query, br + 1));
            return;
        }
    }
    
    g.cbs(&format!("\"{}\" not found", query));
}


fn oyf(g: &mut EditorState, aau: &str) -> Option<String> {
    let ec = g.idd;
    let dti = ((g.cxg() + 2) * 16) as u32;
    
    let mut input = String::new();
    
    loop {
        
        crate::framebuffer::ah(0, dti, ec as u32 * 8, 16, 0xFF1A1A2E);
        let display = format!("{}{}", aau, input);
        crate::framebuffer::cb(&display, 0, dti, D_);
        
        let cx = ((aau.len() + input.len()) * 8) as u32;
        crate::framebuffer::ah(cx, dti, 8, 16, Q_);
        
        if let Some(bs) = crate::keyboard::auw() {
            match bs {
                0x0A => return Some(input), 
                27 => return None,           
                0x08 => { input.pop(); }     
                bm if bm >= 32 && bm < 127 => input.push(bm as char),
                _ => {}
            }
        } else {
            core::hint::hc();
        }
    }
}
