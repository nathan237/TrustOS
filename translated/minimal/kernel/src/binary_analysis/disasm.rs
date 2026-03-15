






use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;




#[derive(Debug, Clone)]
pub struct Dc {
    
    pub re: u64,
    
    pub bf: Vec<u8>,
    
    pub bes: String,
    
    pub bvs: String,
    
    pub byv: Option<String>,
    
    pub ena: Option<u64>,
    
    pub etc: bool,
    
    pub edy: bool,
    
    pub etg: bool,
    
    pub etd: bool,
}



const Cjq: [&str; 16] = ["rax","rcx","rdx","rbx","rsp","rbp","rsi","rdi",
                            "r8","r9","r10","r11","r12","r13","r14","r15"];
const Cjp: [&str; 16] = ["eax","ecx","edx","ebx","esp","ebp","esi","edi",
                            "r8d","r9d","r10d","r11d","r12d","r13d","r14d","r15d"];
const Cjo: [&str; 16] = ["ax","cx","dx","bx","sp","bp","si","di",
                            "r8w","r9w","r10w","r11w","r12w","r13w","r14w","r15w"];
const Cjr:  [&str; 16] = ["al","cl","dl","bl","spl","bpl","sil","dil",
                            "r8b","r9b","r10b","r11b","r12b","r13b","r14b","r15b"];
const CNO_: [&str; 8] = ["al","cl","dl","bl","ah","ch","dh","bh"];

fn ati(w: u8, aw: u8, kf: bool) -> &'static str {
    let a = (w & 0x0F) as usize;
    match aw {
        8 => Cjq.get(a).hu().unwrap_or("?"),
        4 => Cjp.get(a).hu().unwrap_or("?"),
        2 => Cjo.get(a).hu().unwrap_or("?"),
        1 => {
            if kf {
                Cjr.get(a).hu().unwrap_or("?")
            } else {
                CNO_.get(a).hu().unwrap_or("?")
            }
        }
        _ => "?",
    }
}

fn cmw(aw: u8) -> &'static str {
    match aw {
        8 => "qword",
        4 => "dword",
        2 => "word",
        1 => "byte",
        _ => "",
    }
}



const RY_: [&str; 16] = [
    "o", "no", "b", "nb", "z", "nz", "be", "a",
    "s", "ns", "p", "np", "l", "nl", "le", "g",
];



pub struct Disassembler<'a> {
    aj: &'a [u8],
    sm: u64,
    u: usize,
}

impl<'a> Disassembler<'a> {
    pub fn new(aj: &'a [u8], sm: u64) -> Self {
        Self { aj, sm, u: 0 }
    }

    
    pub fn ryc(&mut self, ul: usize) -> Vec<Dc> {
        let mut bd = Vec::new();
        while self.u < self.aj.len() && bd.len() < ul {
            let fi = self.hfp();
            bd.push(fi);
        }
        bd
    }

    
    pub fn irf(&mut self) -> Vec<Dc> {
        self.ryc(8192)
    }

    fn hfp(&mut self) -> Dc {
        let ay = self.u;
        let ag = self.sm + ay as u64;

        
        let mut fkd = false;    
        let mut bus = false;    
        let mut ixq = false;    
        let mut gis = false;    
        let mut wgn = false;

        loop {
            if self.u >= self.aj.len() { break; }
            match self.aj[self.u] {
                0x66 => { fkd = true; self.u += 1; }
                0x67 => { bus = true; self.u += 1; }
                0xF2 => { ixq = true; self.u += 1; }
                0xF3 => { gis = true; self.u += 1; }
                0x26 | 0x2E | 0x36 | 0x3E | 0x64 | 0x65 => { wgn = true; self.u += 1; }
                _ => break,
            }
            
            if self.u - ay > 4 { break; }
        }

        if self.u >= self.aj.len() {
            return self.hqq(ay, ag);
        }

        
        let mut aip: u8 = 0;
        let kf;
        let o = self.aj[self.u];
        if o >= 0x40 && o <= 0x4F {
            aip = o;
            self.u += 1;
            kf = true;
        } else {
            kf = false;
        }

        let ako = (aip & 0x08) != 0;
        let nx = (aip & 0x04) != 0;
        let pg = (aip & 0x02) != 0;
        let ic = (aip & 0x01) != 0;

        
        let yc: u8 = if ako { 8 } else if fkd { 2 } else { 4 };

        if self.u >= self.aj.len() {
            return self.hqq(ay, ag);
        }

        let opcode = self.aj[self.u];
        self.u += 1;

        
        if opcode == 0x0F {
            return self.rud(ay, ag, aip, yc, kf, ixq, gis);
        }

        
        let result = match opcode {
            
            0x90 => Some(("nop", String::new(), None, false, false, false, false)),

            
            0xC3 => Some(("ret", String::new(), None, false, true, false, false)),

            
            0xC2 => {
                let gf = self.alp().unwrap_or(0);
                Some(("ret", format!("{:#x}", gf), None, false, true, false, false))
            }

            
            0xCC => Some(("int3", String::new(), None, false, false, false, false)),

            
            0xCD => {
                let gf = self.ady().unwrap_or(0);
                Some(("int", format!("{:#x}", gf), None, false, false, false, false))
            }

            
            0xF4 => Some(("hlt", String::new(), None, false, false, false, false)),

            
            0xF8 => Some(("clc", String::new(), None, false, false, false, false)),
            0xF9 => Some(("stc", String::new(), None, false, false, false, false)),
            0xFA => Some(("cli", String::new(), None, false, false, false, false)),
            0xFB => Some(("sti", String::new(), None, false, false, false, false)),
            0xFC => Some(("cld", String::new(), None, false, false, false, false)),
            0xFD => Some(("std", String::new(), None, false, false, false, false)),

            
            0xC9 => Some(("leave", String::new(), None, false, false, false, false)),

            
            0x99 => {
                let brk = if ako { "cqo" } else { "cdq" };
                Some((brk, String::new(), None, false, false, false, false))
            }

            
            0x98 => {
                let brk = if ako { "cdqe" } else if fkd { "cbw" } else { "cwde" };
                Some((brk, String::new(), None, false, false, false, false))
            }

            
            0x50..=0x57 => {
                let m = (opcode - 0x50) | if ic { 8 } else { 0 };
                Some(("push", String::from(ati(m, 8, kf)), None, false, false, false, false))
            }

            
            0x58..=0x5F => {
                let m = (opcode - 0x58) | if ic { 8 } else { 0 };
                Some(("pop", String::from(ati(m, 8, kf)), None, false, false, false, false))
            }

            
            0x6A => {
                let gf = self.cmd().unwrap_or(0) as i64;
                Some(("push", ces(gf), None, false, false, false, false))
            }

            
            0x68 => {
                let gf = self.amq().unwrap_or(0) as i64;
                Some(("push", ces(gf), None, false, false, false, false))
            }

            
            0xB0..=0xB7 => {
                
                let m = (opcode - 0xB0) | if ic { 8 } else { 0 };
                let gf = self.ady().unwrap_or(0);
                Some(("mov", format!("{}, {:#x}", ati(m, 1, kf), gf), None, false, false, false, false))
            }
            0xB8..=0xBF => {
                let m = (opcode - 0xB8) | if ic { 8 } else { 0 };
                let gf = if ako {
                    self.jll().unwrap_or(0)
                } else {
                    self.amq().unwrap_or(0) as i64
                };
                Some(("mov", format!("{}, {}", ati(m, yc, kf), ces(gf)), None, false, false, false, false))
            }

            
            0x91..=0x97 => {
                let m = (opcode - 0x90) | if ic { 8 } else { 0 };
                Some(("xchg", format!("{}, {}", ati(0, yc, kf), ati(m, yc, kf)), None, false, false, false, false))
            }

            
            0xE8 => {
                let adj = self.amq().unwrap_or(0) as i64;
                let cd = (ag as i64 + (self.u - ay) as i64 + adj) as u64;
                Some(("call", format!("{:#x}", cd), Some(cd), true, false, false, false))
            }

            
            0xE9 => {
                let adj = self.amq().unwrap_or(0) as i64;
                let cd = (ag as i64 + (self.u - ay) as i64 + adj) as u64;
                Some(("jmp", format!("{:#x}", cd), Some(cd), false, false, true, false))
            }

            
            0xEB => {
                let adj = self.cmd().unwrap_or(0) as i64;
                let cd = (ag as i64 + (self.u - ay) as i64 + adj) as u64;
                Some(("jmp", format!("{:#x}", cd), Some(cd), false, false, true, false))
            }

            
            0x70..=0x7F => {
                let nn = opcode - 0x70;
                let adj = self.cmd().unwrap_or(0) as i64;
                let cd = (ag as i64 + (self.u - ay) as i64 + adj) as u64;
                let brk = format!("j{}", RY_[nn as usize]);
                Some((&"jcc_placeholder", format!("{:#x}", cd), Some(cd), false, false, false, true))
                    .map(|(_, ops, xgf, bto, aux, uaj, ray)| (jcz(&brk), ops, xgf, bto, aux, uaj, ray))
            }

            
            0xE0 => {
                let adj = self.cmd().unwrap_or(0) as i64;
                let cd = (ag as i64 + (self.u - ay) as i64 + adj) as u64;
                Some(("loopne", format!("{:#x}", cd), Some(cd), false, false, false, true))
            }
            0xE1 => {
                let adj = self.cmd().unwrap_or(0) as i64;
                let cd = (ag as i64 + (self.u - ay) as i64 + adj) as u64;
                Some(("loope", format!("{:#x}", cd), Some(cd), false, false, false, true))
            }
            0xE2 => {
                let adj = self.cmd().unwrap_or(0) as i64;
                let cd = (ag as i64 + (self.u - ay) as i64 + adj) as u64;
                Some(("loop", format!("{:#x}", cd), Some(cd), false, false, false, true))
            }

            
            
            0x00 | 0x01 | 0x02 | 0x03 => self.eop(ay, ag, opcode, "add", yc, kf, nx, ic, pg),
            0x08 | 0x09 | 0x0A | 0x0B => self.eop(ay, ag, opcode, "or",  yc, kf, nx, ic, pg),
            0x10 | 0x11 | 0x12 | 0x13 => self.eop(ay, ag, opcode, "adc", yc, kf, nx, ic, pg),
            0x18 | 0x19 | 0x1A | 0x1B => self.eop(ay, ag, opcode, "sbb", yc, kf, nx, ic, pg),
            0x20 | 0x21 | 0x22 | 0x23 => self.eop(ay, ag, opcode, "and", yc, kf, nx, ic, pg),
            0x28 | 0x29 | 0x2A | 0x2B => self.eop(ay, ag, opcode, "sub", yc, kf, nx, ic, pg),
            0x30 | 0x31 | 0x32 | 0x33 => self.eop(ay, ag, opcode, "xor", yc, kf, nx, ic, pg),
            0x38 | 0x39 | 0x3A | 0x3B => self.eop(ay, ag, opcode, "cmp", yc, kf, nx, ic, pg),

            
            0x04 => { let gf = self.ady().unwrap_or(0); Some(("add", format!("al, {:#x}", gf), None, false, false, false, false)) }
            0x05 => { let gf = self.amq().unwrap_or(0); Some(("add", format!("{}, {}", ati(0, yc, kf), ces(gf as i64)), None, false, false, false, false)) }
            0x0C => { let gf = self.ady().unwrap_or(0); Some(("or",  format!("al, {:#x}", gf), None, false, false, false, false)) }
            0x0D => { let gf = self.amq().unwrap_or(0); Some(("or",  format!("{}, {}", ati(0, yc, kf), ces(gf as i64)), None, false, false, false, false)) }
            0x24 => { let gf = self.ady().unwrap_or(0); Some(("and", format!("al, {:#x}", gf), None, false, false, false, false)) }
            0x25 => { let gf = self.amq().unwrap_or(0); Some(("and", format!("{}, {}", ati(0, yc, kf), ces(gf as i64)), None, false, false, false, false)) }
            0x2C => { let gf = self.ady().unwrap_or(0); Some(("sub", format!("al, {:#x}", gf), None, false, false, false, false)) }
            0x2D => { let gf = self.amq().unwrap_or(0); Some(("sub", format!("{}, {}", ati(0, yc, kf), ces(gf as i64)), None, false, false, false, false)) }
            0x34 => { let gf = self.ady().unwrap_or(0); Some(("xor", format!("al, {:#x}", gf), None, false, false, false, false)) }
            0x35 => { let gf = self.amq().unwrap_or(0); Some(("xor", format!("{}, {}", ati(0, yc, kf), ces(gf as i64)), None, false, false, false, false)) }
            0x3C => { let gf = self.ady().unwrap_or(0); Some(("cmp", format!("al, {:#x}", gf), None, false, false, false, false)) }
            0x3D => { let gf = self.amq().unwrap_or(0); Some(("cmp", format!("{}, {}", ati(0, yc, kf), ces(gf as i64)), None, false, false, false, false)) }

            
            0x84 => {
                let (hb, reg) = self.avq(1, kf, nx, ic, pg);
                Some(("test", format!("{}, {}", hb, reg), None, false, false, false, false))
            }
            0x85 => {
                let (hb, reg) = self.avq(yc, kf, nx, ic, pg);
                Some(("test", format!("{}, {}", hb, reg), None, false, false, false, false))
            }

            
            0xA8 => { let gf = self.ady().unwrap_or(0); Some(("test", format!("al, {:#x}", gf), None, false, false, false, false)) }
            0xA9 => { let gf = self.amq().unwrap_or(0); Some(("test", format!("{}, {}", ati(0, yc, kf), ces(gf as i64)), None, false, false, false, false)) }

            
            0x88 => {
                let (hb, reg) = self.avq(1, kf, nx, ic, pg);
                Some(("mov", format!("{}, {}", hb, reg), None, false, false, false, false))
            }
            0x89 => {
                let (hb, reg) = self.avq(yc, kf, nx, ic, pg);
                Some(("mov", format!("{}, {}", hb, reg), None, false, false, false, false))
            }

            
            0x8A => {
                let (hb, reg) = self.avq(1, kf, nx, ic, pg);
                Some(("mov", format!("{}, {}", reg, hb), None, false, false, false, false))
            }
            0x8B => {
                let (hb, reg) = self.avq(yc, kf, nx, ic, pg);
                Some(("mov", format!("{}, {}", reg, hb), None, false, false, false, false))
            }

            
            0x8D => {
                let (hb, reg) = self.avq(yc, kf, nx, ic, pg);
                Some(("lea", format!("{}, {}", reg, hb), None, false, false, false, false))
            }

            
            0x86 => {
                let (hb, reg) = self.avq(1, kf, nx, ic, pg);
                Some(("xchg", format!("{}, {}", hb, reg), None, false, false, false, false))
            }
            0x87 => {
                let (hb, reg) = self.avq(yc, kf, nx, ic, pg);
                Some(("xchg", format!("{}, {}", hb, reg), None, false, false, false, false))
            }

            
            0xC6 => {
                let hb = self.bmu(1, kf, ic, pg);
                let gf = self.ady().unwrap_or(0);
                Some(("mov", format!("{}, {:#x}", hb, gf), None, false, false, false, false))
            }
            0xC7 => {
                let hb = self.bmu(yc, kf, ic, pg);
                let gf = self.amq().unwrap_or(0);
                Some(("mov", format!("{}, {}", hb, ces(gf as i64)), None, false, false, false, false))
            }

            
            0x80 => self.kon(1, kf, ic, pg, true),
            0x81 => self.kon(yc, kf, ic, pg, false),
            0x83 => self.kon(yc, kf, ic, pg, true),

            
            0xC0 => self.geg(1, kf, ic, pg, ShiftCount::Aub),
            0xC1 => self.geg(yc, kf, ic, pg, ShiftCount::Aub),
            0xD0 => self.geg(1, kf, ic, pg, ShiftCount::Awk),
            0xD1 => self.geg(yc, kf, ic, pg, ShiftCount::Awk),
            0xD2 => self.geg(1, kf, ic, pg, ShiftCount::Aag),
            0xD3 => self.geg(yc, kf, ic, pg, ShiftCount::Aag),

            
            0xFE => self.njz(1, kf, nx, ic, pg, ag, ay),
            0xFF => self.njz(yc, kf, nx, ic, pg, ag, ay),

            
            0xF6 => self.njy(1, kf, ic, pg),
            0xF7 => self.njy(yc, kf, ic, pg),

            
            
            0x6B => {
                let (hb, reg) = self.avq(yc, kf, nx, ic, pg);
                let gf = self.cmd().unwrap_or(0);
                Some(("imul", format!("{}, {}, {}", reg, hb, ces(gf as i64)), None, false, false, false, false))
            }
            
            0x69 => {
                let (hb, reg) = self.avq(yc, kf, nx, ic, pg);
                let gf = self.amq().unwrap_or(0);
                Some(("imul", format!("{}, {}, {}", reg, hb, ces(gf as i64)), None, false, false, false, false))
            }

            
            0xA4 => Some(( if gis { "rep movsb" } else { "movsb" }, String::new(), None, false, false, false, false)),
            0xA5 => Some(( if gis { "rep movsd" } else { "movsd" }, String::new(), None, false, false, false, false)),
            0xAA => Some(( if gis { "rep stosb" } else { "stosb" }, String::new(), None, false, false, false, false)),
            0xAB => Some(( if gis { "rep stosd" } else { "stosd" }, String::new(), None, false, false, false, false)),
            0xAC => Some(("lodsb", String::new(), None, false, false, false, false)),
            0xAD => Some(("lodsd", String::new(), None, false, false, false, false)),
            0xAE => Some(( if ixq { "repne scasb" } else { "scasb" }, String::new(), None, false, false, false, false)),
            0xAF => Some(( if ixq { "repne scasd" } else { "scasd" }, String::new(), None, false, false, false, false)),

            
            0xA0 => {
                let foo = self.jlo(ako);
                Some(("mov", format!("al, [{}]", ivh(foo)), None, false, false, false, false))
            }
            0xA1 => {
                let foo = self.jlo(ako);
                Some(("mov", format!("{}, [{}]", ati(0, yc, kf), ivh(foo)), None, false, false, false, false))
            }
            0xA2 => {
                let foo = self.jlo(ako);
                Some(("mov", format!("[{}], al", ivh(foo)), None, false, false, false, false))
            }
            0xA3 => {
                let foo = self.jlo(ako);
                Some(("mov", format!("[{}], {}", ivh(foo), ati(0, yc, kf)), None, false, false, false, false))
            }

            
            0xE4 => { let ai = self.ady().unwrap_or(0); Some(("in", format!("al, {:#x}", ai), None, false, false, false, false)) }
            0xE5 => { let ai = self.ady().unwrap_or(0); Some(("in", format!("eax, {:#x}", ai), None, false, false, false, false)) }
            0xE6 => { let ai = self.ady().unwrap_or(0); Some(("out", format!("{:#x}, al", ai), None, false, false, false, false)) }
            0xE7 => { let ai = self.ady().unwrap_or(0); Some(("out", format!("{:#x}, eax", ai), None, false, false, false, false)) }
            0xEC => Some(("in", String::from("al, dx"), None, false, false, false, false)),
            0xED => Some(("in", String::from("eax, dx"), None, false, false, false, false)),
            0xEE => Some(("out", String::from("dx, al"), None, false, false, false, false)),
            0xEF => Some(("out", String::from("dx, eax"), None, false, false, false, false)),

            
            _ => None,
        };

        match result {
            Some((brk, ops, cd, etc, edy, etg, etd)) => {
                let bf = self.aj[ay..self.u].ip();
                Dc {
                    re: ag,
                    bf,
                    bes: String::from(brk),
                    bvs: ops,
                    byv: None,
                    ena: cd,
                    etc,
                    edy,
                    etg,
                    etd,
                }
            }
            None => self.hqq(ay, ag),
        }
    }

    

    fn rud(&mut self, ay: usize, ag: u64, aip: u8, yc: u8, kf: bool, msi: bool, msj: bool) -> Dc {
        if self.u >= self.aj.len() {
            return self.hqq(ay, ag);
        }

        let nx = (aip & 0x04) != 0;
        let ic = (aip & 0x01) != 0;
        let pg = (aip & 0x02) != 0;

        let fpq = self.aj[self.u];
        self.u += 1;

        let result: Option<(&str, String, Option<u64>, bool, bool, bool, bool)> = match fpq {
            
            0x05 => Some(("syscall", String::new(), None, false, false, false, false)),

            
            0x07 => Some(("sysret", String::new(), None, false, false, false, false)),

            
            0xA2 => Some(("cpuid", String::new(), None, false, false, false, false)),

            
            0x31 => Some(("rdtsc", String::new(), None, false, false, false, false)),

            
            0x32 => Some(("rdmsr", String::new(), None, false, false, false, false)),
            0x30 => Some(("wrmsr", String::new(), None, false, false, false, false)),

            
            0x1F => {
                
                let _ = self.bmu(yc, kf, ic, pg);
                Some(("nop", String::new(), None, false, false, false, false))
            }

            
            0x80..=0x8F => {
                let nn = fpq - 0x80;
                let adj = self.amq().unwrap_or(0) as i64;
                let cd = (ag as i64 + (self.u - ay) as i64 + adj) as u64;
                let hrp = format!("j{}", RY_[nn as usize]);
                Some((jcz(&hrp), format!("{:#x}", cd), Some(cd), false, false, false, true))
            }

            
            0x90..=0x9F => {
                let nn = fpq - 0x90;
                let hb = self.bmu(1, kf, ic, pg);
                let hrp = format!("set{}", RY_[nn as usize]);
                Some((jcz(&hrp), hb, None, false, false, false, false))
            }

            
            0x40..=0x4F => {
                let nn = fpq - 0x40;
                let (hb, reg) = self.avq(yc, kf, nx, ic, pg);
                let hrp = format!("cmov{}", RY_[nn as usize]);
                Some((jcz(&hrp), format!("{}, {}", reg, hb), None, false, false, false, false))
            }

            
            0xB6 => {
                let (hb, reg) = self.avq(1, kf, nx, ic, pg);
                Some(("movzx", format!("{}, {}", ati(jge(nx, self.jjb()), yc, kf), hb), None, false, false, false, false))
            }

            
            0xB7 => {
                let (hb, reg) = self.avq(2, kf, nx, ic, pg);
                Some(("movzx", format!("{}, {}", ati(jge(nx, self.jjb()), yc, kf), hb), None, false, false, false, false))
            }

            
            0xBE => {
                let (hb, reg) = self.avq(1, kf, nx, ic, pg);
                Some(("movsx", format!("{}, {}", ati(jge(nx, self.jjb()), yc, kf), hb), None, false, false, false, false))
            }

            
            0xBF => {
                let (hb, reg) = self.avq(2, kf, nx, ic, pg);
                Some(("movsx", format!("{}, {}", ati(jge(nx, self.jjb()), yc, kf), hb), None, false, false, false, false))
            }

            
            0xAF => {
                let (hb, reg) = self.avq(yc, kf, nx, ic, pg);
                Some(("imul", format!("{}, {}", reg, hb), None, false, false, false, false))
            }

            
            0xBC => {
                let (hb, reg) = self.avq(yc, kf, nx, ic, pg);
                Some(("bsf", format!("{}, {}", reg, hb), None, false, false, false, false))
            }
            0xBD => {
                let (hb, reg) = self.avq(yc, kf, nx, ic, pg);
                Some(("bsr", format!("{}, {}", reg, hb), None, false, false, false, false))
            }

            
            0xA3 => {
                let (hb, reg) = self.avq(yc, kf, nx, ic, pg);
                Some(("bt", format!("{}, {}", hb, reg), None, false, false, false, false))
            }
            0xAB => {
                let (hb, reg) = self.avq(yc, kf, nx, ic, pg);
                Some(("bts", format!("{}, {}", hb, reg), None, false, false, false, false))
            }
            0xB3 => {
                let (hb, reg) = self.avq(yc, kf, nx, ic, pg);
                Some(("btr", format!("{}, {}", hb, reg), None, false, false, false, false))
            }
            0xBB => {
                let (hb, reg) = self.avq(yc, kf, nx, ic, pg);
                Some(("btc", format!("{}, {}", hb, reg), None, false, false, false, false))
            }

            
            0xC0 => {
                let (hb, reg) = self.avq(1, kf, nx, ic, pg);
                Some(("xadd", format!("{}, {}", hb, reg), None, false, false, false, false))
            }
            0xC1 => {
                let (hb, reg) = self.avq(yc, kf, nx, ic, pg);
                Some(("xadd", format!("{}, {}", hb, reg), None, false, false, false, false))
            }

            
            0xC8..=0xCF => {
                let m = (fpq - 0xC8) | if ic { 8 } else { 0 };
                Some(("bswap", String::from(ati(m, yc, kf)), None, false, false, false, false))
            }

            
            0x0B => Some(("ud2", String::new(), None, false, false, false, false)),

            _ => None,
        };

        match result {
            Some((brk, ops, cd, etc, edy, etg, etd)) => {
                let bf = self.aj[ay..self.u].ip();
                Dc {
                    re: ag,
                    bf,
                    bes: String::from(brk),
                    bvs: ops,
                    byv: None,
                    ena: cd,
                    etc,
                    edy,
                    etg,
                    etd,
                }
            }
            None => {
                
                self.hqq(ay, ag)
            }
        }
    }

    

    
    fn avq(&mut self, aw: u8, kf: bool, nx: bool, ic: bool, pg: bool) -> (String, String) {
        if self.u >= self.aj.len() {
            return (String::from("?"), String::from("?"));
        }

        let ms = self.aj[self.u];
        self.u += 1;

        let czy = (ms >> 6) & 3;
        let fsn = ((ms >> 3) & 7) | if nx { 8 } else { 0 };
        let ext = (ms & 7) | if ic { 8 } else { 0 };

        let vtz = String::from(ati(fsn, aw, kf));
        let vzn = self.nkb(czy, ext & 7, ic, pg, aw, kf);

        (vzn, vtz)
    }

    
    fn bmu(&mut self, aw: u8, kf: bool, ic: bool, pg: bool) -> String {
        if self.u >= self.aj.len() {
            return String::from("?");
        }

        let ms = self.aj[self.u];
        self.u += 1;

        let czy = (ms >> 6) & 3;
        let ext = (ms & 7) | if ic { 8 } else { 0 };

        self.nkb(czy, ext & 7, ic, pg, aw, kf)
    }

    
    fn nkb(&mut self, czy: u8, man: u8, ic: bool, pg: bool, aw: u8, kf: bool) -> String {
        let hb = man | if ic { 8 } else { 0 };

        
        if czy == 3 {
            return String::from(ati(hb, aw, kf));
        }

        
        let (dyq, uru) = if man == 4 {
            
            (String::new(), true)
        } else if man == 5 && czy == 0 {
            
            let aor = self.amq().unwrap_or(0);
            return format!("{} [rip{:+#x}]", cmw(aw), aor);
        } else {
            (String::from(ati(hb, 8, kf)), false)
        };

        if uru {
            return self.rur(czy, ic, pg, aw, kf);
        }

        
        match czy {
            0 => format!("{} [{}]", cmw(aw), dyq),
            1 => {
                let aor = self.cmd().unwrap_or(0) as i32;
                if aor == 0 {
                    format!("{} [{}]", cmw(aw), dyq)
                } else {
                    format!("{} [{}{:+#x}]", cmw(aw), dyq, aor)
                }
            }
            2 => {
                let aor = self.amq().unwrap_or(0);
                if aor == 0 {
                    format!("{} [{}]", cmw(aw), dyq)
                } else {
                    format!("{} [{}{:+#x}]", cmw(aw), dyq, aor)
                }
            }
            _ => String::from("?"),
        }
    }

    
    fn rur(&mut self, czy: u8, ic: bool, pg: bool, aw: u8, kf: bool) -> String {
        if self.u >= self.aj.len() {
            return String::from("?");
        }

        let iam = self.aj[self.u];
        self.u += 1;

        let bv = 1u8 << ((iam >> 6) & 3);
        let index = ((iam >> 3) & 7) | if pg { 8 } else { 0 };
        let ar = (iam & 7) | if ic { 8 } else { 0 };

        let oan = index != 4; 
        let dyq = if (ar & 7) == 5 && czy == 0 {
            
            let aor = self.amq().unwrap_or(0);
            if oan {
                if bv > 1 {
                    return format!("{} [{}*{}{:+#x}]", cmw(aw), ati(index, 8, kf), bv, aor);
                } else {
                    return format!("{} [{}{:+#x}]", cmw(aw), ati(index, 8, kf), aor);
                }
            } else {
                return format!("{} [{:#x}]", cmw(aw), aor);
            }
        } else {
            String::from(ati(ar, 8, kf))
        };

        
        let gxz = if oan {
            if bv > 1 {
                format!("{}+{}*{}", dyq, ati(index, 8, kf), bv)
            } else {
                format!("{}+{}", dyq, ati(index, 8, kf))
            }
        } else {
            dyq
        };

        match czy {
            0 => format!("{} [{}]", cmw(aw), gxz),
            1 => {
                let aor = self.cmd().unwrap_or(0) as i32;
                if aor == 0 {
                    format!("{} [{}]", cmw(aw), gxz)
                } else {
                    format!("{} [{}{:+#x}]", cmw(aw), gxz, aor)
                }
            }
            2 => {
                let aor = self.amq().unwrap_or(0);
                if aor == 0 {
                    format!("{} [{}]", cmw(aw), gxz)
                } else {
                    format!("{} [{}{:+#x}]", cmw(aw), gxz, aor)
                }
            }
            _ => String::from("?"),
        }
    }

    

    fn eop(&mut self, _start: usize, xxr: u64, opcode: u8, bes: &str, yc: u8, kf: bool, nx: bool, ic: bool, pg: bool) -> Option<(&'static str, String, Option<u64>, bool, bool, bool, bool)> {
        let twu = (opcode & 1) == 0;
        let te = (opcode & 2) != 0; 
        let nf = if twu { 1 } else { yc };
        let (hb, reg) = self.avq(nf, kf, nx, ic, pg);
        let ops = if te {
            format!("{}, {}", reg, hb)
        } else {
            format!("{}, {}", hb, reg)
        };
        let brk: &'static str = match bes {
            "add" => "add", "or" => "or", "adc" => "adc", "sbb" => "sbb",
            "and" => "and", "sub" => "sub", "xor" => "xor", "cmp" => "cmp",
            _ => "?alu",
        };
        Some((brk, ops, None, false, false, false, false))
    }

    fn kon(&mut self, aw: u8, kf: bool, ic: bool, pg: bool, tsb: bool) -> Option<(&'static str, String, Option<u64>, bool, bool, bool, bool)> {
        if self.u >= self.aj.len() { return None; }
        let ms = self.aj[self.u]; 
        let op = (ms >> 3) & 7;

        let hb = self.bmu(aw, kf, ic, pg);
        let gf = if tsb {
            self.cmd().unwrap_or(0) as i64
        } else {
            self.amq().unwrap_or(0) as i64
        };

        let brk: &'static str = match op {
            0 => "add", 1 => "or", 2 => "adc", 3 => "sbb",
            4 => "and", 5 => "sub", 6 => "xor", 7 => "cmp",
            _ => "?",
        };

        Some((brk, format!("{}, {}", hb, ces(gf)), None, false, false, false, false))
    }

    fn geg(&mut self, aw: u8, kf: bool, ic: bool, pg: bool, az: ShiftCount) -> Option<(&'static str, String, Option<u64>, bool, bool, bool, bool)> {
        if self.u >= self.aj.len() { return None; }
        let ms = self.aj[self.u];
        let op = (ms >> 3) & 7;

        let hb = self.bmu(aw, kf, ic, pg);
        let ffy = match az {
            ShiftCount::Awk => String::from("1"),
            ShiftCount::Aag => String::from("cl"),
            ShiftCount::Aub => {
                let gf = self.ady().unwrap_or(0);
                format!("{}", gf)
            }
        };

        let brk: &'static str = match op {
            0 => "rol", 1 => "ror", 2 => "rcl", 3 => "rcr",
            4 => "shl", 5 => "shr", 6 => "sal", 7 => "sar",
            _ => "?",
        };

        Some((brk, format!("{}, {}", hb, ffy), None, false, false, false, false))
    }

    fn njz(&mut self, aw: u8, kf: bool, ycj: bool, ic: bool, pg: bool, ag: u64, ay: usize) -> Option<(&'static str, String, Option<u64>, bool, bool, bool, bool)> {
        if self.u >= self.aj.len() { return None; }
        let ms = self.aj[self.u];
        let op = (ms >> 3) & 7;

        match op {
            0 => {
                let hb = self.bmu(aw, kf, ic, pg);
                Some(("inc", hb, None, false, false, false, false))
            }
            1 => {
                let hb = self.bmu(aw, kf, ic, pg);
                Some(("dec", hb, None, false, false, false, false))
            }
            2 if aw > 1 => {
                
                let hb = self.bmu(8, kf, ic, pg);
                Some(("call", hb, None, true, false, false, false))
            }
            4 if aw > 1 => {
                
                let hb = self.bmu(8, kf, ic, pg);
                Some(("jmp", hb, None, false, false, true, false))
            }
            6 if aw > 1 => {
                
                let hb = self.bmu(8, kf, ic, pg);
                Some(("push", hb, None, false, false, false, false))
            }
            _ => {
                let _ = self.bmu(aw, kf, ic, pg);
                None
            }
        }
    }

    fn njy(&mut self, aw: u8, kf: bool, ic: bool, pg: bool) -> Option<(&'static str, String, Option<u64>, bool, bool, bool, bool)> {
        if self.u >= self.aj.len() { return None; }
        let ms = self.aj[self.u];
        let op = (ms >> 3) & 7;

        match op {
            0 | 1 => {
                
                let hb = self.bmu(aw, kf, ic, pg);
                let gf = if aw == 1 {
                    self.ady().unwrap_or(0) as i64
                } else {
                    self.amq().unwrap_or(0) as i64
                };
                Some(("test", format!("{}, {}", hb, ces(gf)), None, false, false, false, false))
            }
            2 => {
                let hb = self.bmu(aw, kf, ic, pg);
                Some(("not", hb, None, false, false, false, false))
            }
            3 => {
                let hb = self.bmu(aw, kf, ic, pg);
                Some(("neg", hb, None, false, false, false, false))
            }
            4 => {
                let hb = self.bmu(aw, kf, ic, pg);
                Some(("mul", hb, None, false, false, false, false))
            }
            5 => {
                let hb = self.bmu(aw, kf, ic, pg);
                Some(("imul", hb, None, false, false, false, false))
            }
            6 => {
                let hb = self.bmu(aw, kf, ic, pg);
                Some(("div", hb, None, false, false, false, false))
            }
            7 => {
                let hb = self.bmu(aw, kf, ic, pg);
                Some(("idiv", hb, None, false, false, false, false))
            }
            _ => None,
        }
    }

    

    fn ady(&mut self) -> Option<u8> {
        if self.u >= self.aj.len() { return None; }
        let p = self.aj[self.u];
        self.u += 1;
        Some(p)
    }

    fn cmd(&mut self) -> Option<i8> {
        self.ady().map(|p| p as i8)
    }

    fn alp(&mut self) -> Option<u16> {
        if self.u + 2 > self.aj.len() { return None; }
        let p = u16::dj([self.aj[self.u], self.aj[self.u + 1]]);
        self.u += 2;
        Some(p)
    }

    fn amq(&mut self) -> Option<i32> {
        if self.u + 4 > self.aj.len() { return None; }
        let p = i32::dj([
            self.aj[self.u], self.aj[self.u + 1],
            self.aj[self.u + 2], self.aj[self.u + 3],
        ]);
        self.u += 4;
        Some(p)
    }

    fn jll(&mut self) -> Option<i64> {
        if self.u + 8 > self.aj.len() { return None; }
        let p = i64::dj([
            self.aj[self.u], self.aj[self.u + 1],
            self.aj[self.u + 2], self.aj[self.u + 3],
            self.aj[self.u + 4], self.aj[self.u + 5],
            self.aj[self.u + 6], self.aj[self.u + 7],
        ]);
        self.u += 8;
        Some(p)
    }

    fn jlo(&mut self, two: bool) -> u64 {
        if two {
            self.jll().unwrap_or(0) as u64
        } else {
            self.amq().unwrap_or(0) as u64
        }
    }

    
    fn jjb(&self) -> u8 {
        if self.u > 0 {
            
            
            self.aj.get(self.u.nj(1)).hu().unwrap_or(0)
        } else {
            0
        }
    }

    fn hqq(&mut self, ay: usize, ag: u64) -> Dc {
        
        if self.u <= ay {
            self.u = ay + 1;
        }
        let ci = self.u.v(self.aj.len());
        let bf = self.aj[ay..ci].ip();
        let nu: String = bf.iter().map(|o| format!("{:#04x}", o)).collect::<Vec<_>>().rr(", ");
        Dc {
            re: ag,
            bf,
            bes: String::from("db"),
            bvs: nu,
            byv: None,
            ena: None,
            etc: false,
            edy: false,
            etg: false,
            etd: false,
        }
    }
}



enum ShiftCount { Awk, Aag, Aub }

fn ces(p: i64) -> String {
    if p >= 0 && p <= 9 {
        format!("{}", p)
    } else if p >= 0 {
        format!("{:#x}", p)
    } else if p >= -128 {
        format!("-{:#x}", -p)
    } else {
        format!("-{:#x}", (-p) as u64)
    }
}

fn ivh(p: u64) -> String {
    format!("{:#x}", p)
}


fn jge(nx: bool, ms: u8) -> u8 {
    ((ms >> 3) & 7) | if nx { 8 } else { 0 }
}




fn jcz(e: &str) -> &'static str {
    use alloc::boxed::Box;
    let boxed = String::from(e).lfh();
    Box::fmu(boxed)
}




pub fn qiu(
    instructions: &mut [Dc],
    blw: &alloc::collections::BTreeMap<u64, String>,
) {
    
    let mut lhu: Option<i64> = None;

    for fi in instructions.el() {
        
        if let Some(cd) = fi.ena {
            if let Some(j) = blw.get(&cd) {
                fi.byv = Some(format!("<{}>", j));
            }
        }

        
        if fi.bes == "mov" && (fi.bvs.cj("eax,") || fi.bvs.cj("rax,")) {
            
            if let Some(rmq) = fi.bvs.du(',') {
                let tsc = fi.bvs[rmq + 1..].em();
                if let Some(ap) = vcq(tsc) {
                    lhu = Some(ap);
                }
            }
        } else if fi.bes == "xor" && fi.bvs.contains("eax") && fi.bvs.oh("eax").az() == 2 {
            lhu = Some(0);
        }

        
        if fi.bes == "syscall" {
            if let Some(num) = lhu {
                let j = crate::transpiler::gty(num as u64);
                fi.byv = Some(format!("sys_{} ({})", j, num));
            }
        }
    }
}

fn vcq(e: &str) -> Option<i64> {
    let e = e.em();
    if e.cj("0x") || e.cj("0X") {
        i64::wa(&e[2..], 16).bq()
    } else if e.cj("-0x") || e.cj("-0X") {
        i64::wa(&e[3..], 16).bq().map(|p| -p)
    } else {
        e.parse::<i64>().bq()
    }
}
