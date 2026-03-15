






use alloc::string::{String, Gd};
use alloc::vec::Vec;
use alloc::vec;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::format;
use libm::{hjw, qxf, jmv, ibi, pwe};


#[derive(Debug, Clone)]
pub enum JsValue {
    Ba,
    Gm,
    Cb(bool),
    L(f64),
    String(String),
    Cw(BTreeMap<String, JsValue>),
    U(Vec<JsValue>),
    Bs(String, Vec<String>, String), 
    H(String),
}

impl JsValue {
    
    pub fn ezy(&self) -> bool {
        match self {
            JsValue::Ba | JsValue::Gm => false,
            JsValue::Cb(o) => *o,
            JsValue::L(bo) => *bo != 0.0 && !bo.ogj(),
            JsValue::String(e) => !e.is_empty(),
            JsValue::Cw(_) | JsValue::U(_) => true,
            JsValue::Bs(..) | JsValue::H(_) => true,
        }
    }
    
    
    pub fn zo(&self) -> f64 {
        match self {
            JsValue::Ba => f64::Lx,
            JsValue::Gm => 0.0,
            JsValue::Cb(true) => 1.0,
            JsValue::Cb(false) => 0.0,
            JsValue::L(bo) => *bo,
            JsValue::String(e) => e.parse().unwrap_or(f64::Lx),
            _ => f64::Lx,
        }
    }
    
    
    pub fn to_string(&self) -> String {
        match self {
            JsValue::Ba => "undefined".to_string(),
            JsValue::Gm => "null".to_string(),
            JsValue::Cb(true) => "true".to_string(),
            JsValue::Cb(false) => "false".to_string(),
            JsValue::L(bo) => format!("{}", bo),
            JsValue::String(e) => e.clone(),
            JsValue::Cw(_) => "[object Object]".to_string(),
            JsValue::U(sy) => {
                let ek: Vec<String> = sy.iter().map(|p| p.to_string()).collect();
                ek.rr(",")
            }
            JsValue::Bs(j, ..) => format!("function {}() {{ [native code] }}", j),
            JsValue::H(j) => format!("function {}() {{ [native code] }}", j),
        }
    }
}


pub struct JsContext {
    pub apu: BTreeMap<String, JsValue>,
    pub ffp: Vec<String>,
}

impl JsContext {
    pub fn new() -> Self {
        let mut be = Self {
            apu: BTreeMap::new(),
            ffp: Vec::new(),
        };
        
        
        be.wkj();
        be
    }
    
    fn wkj(&mut self) {
        
        let mut console = BTreeMap::new();
        console.insert("log".to_string(), JsValue::H("console.log".to_string()));
        console.insert("warn".to_string(), JsValue::H("console.warn".to_string()));
        console.insert("error".to_string(), JsValue::H("console.error".to_string()));
        console.insert("info".to_string(), JsValue::H("console.log".to_string()));
        console.insert("debug".to_string(), JsValue::H("console.log".to_string()));
        self.apu.insert("console".to_string(), JsValue::Cw(console));
        
        
        let mut math = BTreeMap::new();
        math.insert("PI".to_string(), JsValue::L(core::f64::consts::Eu));
        math.insert("E".to_string(), JsValue::L(core::f64::consts::Se));
        math.insert("LN2".to_string(), JsValue::L(core::f64::consts::IG_));
        math.insert("LN10".to_string(), JsValue::L(core::f64::consts::DSP_));
        math.insert("SQRT2".to_string(), JsValue::L(core::f64::consts::EGR_));
        math.insert("random".to_string(), JsValue::H("Math.random".to_string()));
        math.insert("floor".to_string(), JsValue::H("Math.floor".to_string()));
        math.insert("ceil".to_string(), JsValue::H("Math.ceil".to_string()));
        math.insert("round".to_string(), JsValue::H("Math.round".to_string()));
        math.insert("abs".to_string(), JsValue::H("Math.abs".to_string()));
        math.insert("sqrt".to_string(), JsValue::H("Math.sqrt".to_string()));
        math.insert("min".to_string(), JsValue::H("Math.min".to_string()));
        math.insert("max".to_string(), JsValue::H("Math.max".to_string()));
        math.insert("pow".to_string(), JsValue::H("Math.pow".to_string()));
        math.insert("sin".to_string(), JsValue::H("Math.sin".to_string()));
        math.insert("cos".to_string(), JsValue::H("Math.cos".to_string()));
        math.insert("tan".to_string(), JsValue::H("Math.tan".to_string()));
        math.insert("log".to_string(), JsValue::H("Math.log".to_string()));
        math.insert("sign".to_string(), JsValue::H("Math.sign".to_string()));
        math.insert("trunc".to_string(), JsValue::H("Math.trunc".to_string()));
        self.apu.insert("Math".to_string(), JsValue::Cw(math));
        
        
        let mut lgy = BTreeMap::new();
        lgy.insert("parse".to_string(), JsValue::H("JSON.parse".to_string()));
        lgy.insert("stringify".to_string(), JsValue::H("JSON.stringify".to_string()));
        self.apu.insert("JSON".to_string(), JsValue::Cw(lgy));
        
        
        let mut ama = BTreeMap::new();
        ama.insert("getElementById".to_string(), JsValue::H("document.getElementById".to_string()));
        ama.insert("querySelector".to_string(), JsValue::H("document.querySelector".to_string()));
        ama.insert("querySelectorAll".to_string(), JsValue::H("document.querySelectorAll".to_string()));
        ama.insert("createElement".to_string(), JsValue::H("document.createElement".to_string()));
        ama.insert("createTextNode".to_string(), JsValue::H("document.createTextNode".to_string()));
        ama.insert("write".to_string(), JsValue::H("document.write".to_string()));
        ama.insert("title".to_string(), JsValue::String("TrustOS Browser".to_string()));
        ama.insert("readyState".to_string(), JsValue::String("complete".to_string()));
        
        
        let mut gj = BTreeMap::new();
        gj.insert("innerHTML".to_string(), JsValue::String(String::new()));
        gj.insert("textContent".to_string(), JsValue::String(String::new()));
        gj.insert("className".to_string(), JsValue::String(String::new()));
        gj.insert("style".to_string(), JsValue::Cw(BTreeMap::new()));
        gj.insert("appendChild".to_string(), JsValue::H("element.appendChild".to_string()));
        gj.insert("children".to_string(), JsValue::U(Vec::new()));
        gj.insert("tagName".to_string(), JsValue::String("BODY".to_string()));
        ama.insert("body".to_string(), JsValue::Cw(gj));
        self.apu.insert("document".to_string(), JsValue::Cw(ama));
        
        
        let mut bh = BTreeMap::new();
        bh.insert("innerWidth".to_string(), JsValue::L(1024.0));
        bh.insert("innerHeight".to_string(), JsValue::L(768.0));
        bh.insert("location".to_string(), JsValue::Cw(BTreeMap::new()));
        bh.insert("navigator".to_string(), JsValue::Cw({
            let mut jgl = BTreeMap::new();
            jgl.insert("userAgent".to_string(), JsValue::String("TrustOS/1.0".to_string()));
            jgl.insert("platform".to_string(), JsValue::String("TrustOS".to_string()));
            jgl.insert("language".to_string(), JsValue::String("en-US".to_string()));
            jgl
        }));
        self.apu.insert("window".to_string(), JsValue::Cw(bh));
        
        
        self.apu.insert("parseInt".to_string(), JsValue::H("parseInt".to_string()));
        self.apu.insert("parseFloat".to_string(), JsValue::H("parseFloat".to_string()));
        self.apu.insert("isNaN".to_string(), JsValue::H("isNaN".to_string()));
        self.apu.insert("isFinite".to_string(), JsValue::H("isFinite".to_string()));
        self.apu.insert("alert".to_string(), JsValue::H("alert".to_string()));
        self.apu.insert("setTimeout".to_string(), JsValue::H("setTimeout".to_string()));
        self.apu.insert("setInterval".to_string(), JsValue::H("setInterval".to_string()));
        self.apu.insert("clearTimeout".to_string(), JsValue::H("clearTimeout".to_string()));
        self.apu.insert("clearInterval".to_string(), JsValue::H("clearInterval".to_string()));
        self.apu.insert("encodeURIComponent".to_string(), JsValue::H("encodeURIComponent".to_string()));
        self.apu.insert("decodeURIComponent".to_string(), JsValue::H("decodeURIComponent".to_string()));
        self.apu.insert("String".to_string(), JsValue::H("String".to_string()));
        self.apu.insert("Number".to_string(), JsValue::H("Number".to_string()));
        self.apu.insert("Boolean".to_string(), JsValue::H("Boolean".to_string()));
        self.apu.insert("Array".to_string(), JsValue::H("Array".to_string()));
        self.apu.insert("Object".to_string(), JsValue::H("Object".to_string()));
    }
    
    
    pub fn bna(&mut self, aj: &str) -> Result<JsValue, String> {
        let eb = fwz(aj)?;
        let gzb = parse(&eb)?;
        self.ggk(&gzb)
    }
    
    
    fn ggk(&mut self, boq: &[Statement]) -> Result<JsValue, String> {
        let mut result = JsValue::Ba;
        for stmt in boq {
            result = self.nre(stmt)?;
        }
        Ok(result)
    }
    
    
    fn nre(&mut self, stmt: &Statement) -> Result<JsValue, String> {
        match stmt {
            Statement::Bvr(j, expr) => {
                let bn = if let Some(aa) = expr {
                    self.bbo(aa)?
                } else {
                    JsValue::Ba
                };
                self.apu.insert(j.clone(), bn);
                Ok(JsValue::Ba)
            }
            Statement::Expr(expr) => self.bbo(expr),
            Statement::Gx(mo, cne, ckc) => {
                let rnn = self.bbo(mo)?;
                if rnn.ezy() {
                    self.ggk(cne)
                } else if let Some(skf) = ckc {
                    self.ggk(skf)
                } else {
                    Ok(JsValue::Ba)
                }
            }
            Statement::La(mo, gj) => {
                while self.bbo(mo)?.ezy() {
                    self.ggk(gj)?;
                }
                Ok(JsValue::Ba)
            }
            Statement::Ll(init, mo, qs, gj) => {
                if let Some(ttw) = init {
                    self.nre(ttw)?;
                }
                while mo.as_ref().map(|r| self.bbo(r).map(|p| p.ezy()).unwrap_or(false)).unwrap_or(true) {
                    self.ggk(gj)?;
                    if let Some(xop) = qs {
                        self.bbo(xop)?;
                    }
                }
                Ok(JsValue::Ba)
            }
            Statement::Hd(expr) => {
                if let Some(aa) = expr {
                    self.bbo(aa)
                } else {
                    Ok(JsValue::Ba)
                }
            }
            Statement::Bs(j, oi, gj) => {
                self.apu.insert(
                    j.clone(),
                    JsValue::Bs(j.clone(), oi.clone(), gj.clone()),
                );
                Ok(JsValue::Ba)
            }
            Statement::Dj(boq) => self.ggk(boq),
        }
    }
    
    
    fn bbo(&mut self, expr: &Expr) -> Result<JsValue, String> {
        match expr {
            Expr::Th(ugd) => Ok(ugd.clone()),
            Expr::Lp(j) => {
                self.apu.get(j).abn().ok_or_else(|| format!("ReferenceError: {} is not defined", j))
            }
            Expr::Rl(op, fd, hw) => {
                let uiv = self.bbo(fd)?;
                let wbq = self.bbo(hw)?;
                self.snn(op, uiv, wbq)
            }
            Expr::Baf(op, htq) => {
                let ap = self.bbo(htq)?;
                self.snp(op, ap)
            }
            Expr::En(kgd, n) => {
                
                let (ke, afw) = if let Expr::Avl(uwv, yby) = kgd.as_ref() {
                    let ehf = self.bbo(uwv)?;
                    let bb = self.bbo(kgd)?;
                    (bb, Some(ehf))
                } else {
                    (self.bbo(kgd)?, None)
                };
                let mut mwf = Vec::new();
                for ji in n {
                    mwf.push(self.bbo(ji)?);
                }
                self.nbk(ke, mwf, afw)
            }
            Expr::Avl(lpq, frl) => {
                let lpr = self.bbo(lpq)?;
                match &lpr {
                    JsValue::Cw(map) => {
                        Ok(map.get(frl).abn().unwrap_or(JsValue::Ba))
                    }
                    JsValue::U(sy) => {
                        match frl.as_str() {
                            "length" => Ok(JsValue::L(sy.len() as f64)),
                            "push" => Ok(JsValue::H("Array.push".to_string())),
                            "pop" => Ok(JsValue::H("Array.pop".to_string())),
                            "shift" => Ok(JsValue::H("Array.shift".to_string())),
                            "unshift" => Ok(JsValue::H("Array.unshift".to_string())),
                            "join" => Ok(JsValue::H("Array.join".to_string())),
                            "reverse" => Ok(JsValue::H("Array.reverse".to_string())),
                            "indexOf" => Ok(JsValue::H("Array.indexOf".to_string())),
                            "includes" => Ok(JsValue::H("Array.includes".to_string())),
                            "slice" => Ok(JsValue::H("Array.slice".to_string())),
                            "concat" => Ok(JsValue::H("Array.concat".to_string())),
                            "map" => Ok(JsValue::H("Array.map".to_string())),
                            "filter" => Ok(JsValue::H("Array.filter".to_string())),
                            "forEach" => Ok(JsValue::H("Array.forEach".to_string())),
                            "find" => Ok(JsValue::H("Array.find".to_string())),
                            "some" => Ok(JsValue::H("Array.some".to_string())),
                            "every" => Ok(JsValue::H("Array.every".to_string())),
                            "sort" => Ok(JsValue::H("Array.sort".to_string())),
                            "fill" => Ok(JsValue::H("Array.fill".to_string())),
                            "flat" => Ok(JsValue::H("Array.flat".to_string())),
                            "reduce" => Ok(JsValue::H("Array.reduce".to_string())),
                            _ => {
                                if let Ok(w) = frl.parse::<usize>() {
                                    Ok(sy.get(w).abn().unwrap_or(JsValue::Ba))
                                } else {
                                    Ok(JsValue::Ba)
                                }
                            }
                        }
                    }
                    JsValue::String(e) => {
                        match frl.as_str() {
                            "length" => Ok(JsValue::L(e.len() as f64)),
                            "toUpperCase" => Ok(JsValue::H("String.toUpperCase".to_string())),
                            "toLowerCase" => Ok(JsValue::H("String.toLowerCase".to_string())),
                            "trim" => Ok(JsValue::H("String.trim".to_string())),
                            "trimStart" | "trimLeft" => Ok(JsValue::H("String.trimStart".to_string())),
                            "trimEnd" | "trimRight" => Ok(JsValue::H("String.trimEnd".to_string())),
                            "includes" => Ok(JsValue::H("String.includes".to_string())),
                            "indexOf" => Ok(JsValue::H("String.indexOf".to_string())),
                            "lastIndexOf" => Ok(JsValue::H("String.lastIndexOf".to_string())),
                            "startsWith" => Ok(JsValue::H("String.startsWith".to_string())),
                            "endsWith" => Ok(JsValue::H("String.endsWith".to_string())),
                            "slice" => Ok(JsValue::H("String.slice".to_string())),
                            "substring" => Ok(JsValue::H("String.substring".to_string())),
                            "replace" => Ok(JsValue::H("String.replace".to_string())),
                            "split" => Ok(JsValue::H("String.split".to_string())),
                            "charAt" => Ok(JsValue::H("String.charAt".to_string())),
                            "charCodeAt" => Ok(JsValue::H("String.charCodeAt".to_string())),
                            "repeat" => Ok(JsValue::H("String.repeat".to_string())),
                            "padStart" => Ok(JsValue::H("String.padStart".to_string())),
                            "padEnd" => Ok(JsValue::H("String.padEnd".to_string())),
                            "concat" => Ok(JsValue::H("String.concat".to_string())),
                            "match" => Ok(JsValue::H("String.match".to_string())),
                            "search" => Ok(JsValue::H("String.search".to_string())),
                            _ => Ok(JsValue::Ba),
                        }
                    }
                    JsValue::L(ybl) => {
                        match frl.as_str() {
                            "toFixed" => Ok(JsValue::H("Number.toFixed".to_string())),
                            "toString" => Ok(JsValue::H("Number.toString".to_string())),
                            _ => Ok(JsValue::Ba),
                        }
                    }
                    _ => Ok(JsValue::Ba),
                }
            }
            Expr::Index(lpq, w) => {
                let lpr = self.bbo(lpq)?;
                let odf = self.bbo(w)?;
                match lpr {
                    JsValue::U(sy) => {
                        let a = odf.zo() as usize;
                        Ok(sy.get(a).abn().unwrap_or(JsValue::Ba))
                    }
                    JsValue::Cw(map) => {
                        let bs = odf.to_string();
                        Ok(map.get(&bs).abn().unwrap_or(JsValue::Ba))
                    }
                    _ => Ok(JsValue::Ba),
                }
            }
            Expr::U(bgw) => {
                let mut sy = Vec::new();
                for ij in bgw {
                    sy.push(self.bbo(ij)?);
                }
                Ok(JsValue::U(sy))
            }
            Expr::Cw(gpy) => {
                let mut map = BTreeMap::new();
                for (bs, ap) in gpy {
                    map.insert(bs.clone(), self.bbo(ap)?);
                }
                Ok(JsValue::Cw(map))
            }
            Expr::Vk(j, bn) => {
                let ap = self.bbo(bn)?;
                self.apu.insert(j.clone(), ap.clone());
                Ok(ap)
            }
            Expr::Bud(mo, mkp, ksz) => {
                if self.bbo(mo)?.ezy() {
                    self.bbo(mkp)
                } else {
                    self.bbo(ksz)
                }
            }
        }
    }
    
    fn snn(&self, op: &str, fd: JsValue, hw: JsValue) -> Result<JsValue, String> {
        match op {
            "+" => {
                
                match (&fd, &hw) {
                    (JsValue::String(q), _) => Ok(JsValue::String(format!("{}{}", q, hw.to_string()))),
                    (_, JsValue::String(o)) => Ok(JsValue::String(format!("{}{}", fd.to_string(), o))),
                    _ => Ok(JsValue::L(fd.zo() + hw.zo())),
                }
            }
            "-" => Ok(JsValue::L(fd.zo() - hw.zo())),
            "*" => Ok(JsValue::L(fd.zo() * hw.zo())),
            "/" => Ok(JsValue::L(fd.zo() / hw.zo())),
            "%" => Ok(JsValue::L(fd.zo() % hw.zo())),
            "<" => Ok(JsValue::Cb(fd.zo() < hw.zo())),
            ">" => Ok(JsValue::Cb(fd.zo() > hw.zo())),
            "<=" => Ok(JsValue::Cb(fd.zo() <= hw.zo())),
            ">=" => Ok(JsValue::Cb(fd.zo() >= hw.zo())),
            "==" | "===" => {
                
                Ok(JsValue::Cb(fd.to_string() == hw.to_string()))
            }
            "!=" | "!==" => {
                Ok(JsValue::Cb(fd.to_string() != hw.to_string()))
            }
            "&&" => {
                if !fd.ezy() {
                    Ok(fd)
                } else {
                    Ok(hw)
                }
            }
            "||" => {
                if fd.ezy() {
                    Ok(fd)
                } else {
                    Ok(hw)
                }
            }
            _ => Err(format!("Unknown operator: {}", op)),
        }
    }
    
    fn snp(&self, op: &str, ap: JsValue) -> Result<JsValue, String> {
        match op {
            "!" => Ok(JsValue::Cb(!ap.ezy())),
            "-" => Ok(JsValue::L(-ap.zo())),
            "+" => Ok(JsValue::L(ap.zo())),
            "typeof" => {
                let ab = match ap {
                    JsValue::Ba => "undefined",
                    JsValue::Gm => "object", 
                    JsValue::Cb(_) => "boolean",
                    JsValue::L(_) => "number",
                    JsValue::String(_) => "string",
                    JsValue::Cw(_) | JsValue::U(_) => "object",
                    JsValue::Bs(..) | JsValue::H(_) => "function",
                };
                Ok(JsValue::String(ab.to_string()))
            }
            _ => Err(format!("Unknown unary operator: {}", op)),
        }
    }
    
    fn nbk(&mut self, ke: JsValue, n: Vec<JsValue>, afw: Option<JsValue>) -> Result<JsValue, String> {
        match ke {
            JsValue::H(j) => self.qvn(&j, n, afw),
            JsValue::Bs(blu, oi, gj) => {
                for (a, evz) in oi.iter().cf() {
                    let ap = n.get(a).abn().unwrap_or(JsValue::Ba);
                    self.apu.insert(evz.clone(), ap);
                }
                self.bna(&gj)
            }
            _ => Err("TypeError: not a function".to_string()),
        }
    }

    fn yhh(&mut self, ke: JsValue, n: Vec<JsValue>) -> Result<JsValue, String> {
        self.nbk(ke, n, None)
    }
    
    fn qvn(&mut self, j: &str, n: Vec<JsValue>, afw: Option<JsValue>) -> Result<JsValue, String> {
        match j {
            
            "console.log" | "console.warn" | "console.error" => {
                let an: Vec<String> = n.iter().map(|p| p.to_string()).collect();
                let line = an.rr(" ");
                self.ffp.push(line);
                Ok(JsValue::Ba)
            }
            "alert" => {
                if let Some(fr) = n.fv() {
                    self.ffp.push(format!("[ALERT] {}", fr.to_string()));
                }
                Ok(JsValue::Ba)
            }

            
            "setTimeout" | "setInterval" => Ok(JsValue::L(0.0)),
            "clearTimeout" | "clearInterval" => Ok(JsValue::Ba),

            
            "Math.random" => {
                let dv = crate::cpu::tsc::ow();
                let vqa = ((dv >> 16) as f64) / 65536.0;
                Ok(JsValue::L(vqa % 1.0))
            }
            "Math.floor" => {
                let bo = n.fv().map(|p| p.zo()).unwrap_or(0.0);
                Ok(JsValue::L(hjw(bo)))
            }
            "Math.ceil" => {
                let bo = n.fv().map(|p| p.zo()).unwrap_or(0.0);
                Ok(JsValue::L(qxf(bo)))
            }
            "Math.round" => {
                let bo = n.fv().map(|p| p.zo()).unwrap_or(0.0);
                Ok(JsValue::L(jmv(bo)))
            }
            "Math.abs" => {
                let bo = n.fv().map(|p| p.zo()).unwrap_or(0.0);
                Ok(JsValue::L(libm::sqq(bo)))
            }
            "Math.sqrt" => {
                let bo = n.fv().map(|p| p.zo()).unwrap_or(0.0);
                Ok(JsValue::L(ibi(bo)))
            }
            "Math.min" => {
                if n.is_empty() { return Ok(JsValue::L(f64::Att)); }
                let mut ef = n[0].zo();
                for q in &n[1..] { let p = q.zo(); if p < ef { ef = p; } }
                Ok(JsValue::L(ef))
            }
            "Math.max" => {
                if n.is_empty() { return Ok(JsValue::L(f64::IP_)); }
                let mut ef = n[0].zo();
                for q in &n[1..] { let p = q.zo(); if p > ef { ef = p; } }
                Ok(JsValue::L(ef))
            }
            "Math.pow" => {
                let ar = n.fv().map(|p| p.zo()).unwrap_or(0.0);
                let bgz = n.get(1).map(|p| p.zo()).unwrap_or(0.0);
                Ok(JsValue::L(libm::vke(ar, bgz)))
            }
            "Math.sin" => {
                let bo = n.fv().map(|p| p.zo()).unwrap_or(0.0);
                Ok(JsValue::L(libm::ayq(bo)))
            }
            "Math.cos" => {
                let bo = n.fv().map(|p| p.zo()).unwrap_or(0.0);
                Ok(JsValue::L(libm::cjt(bo)))
            }
            "Math.tan" => {
                let bo = n.fv().map(|p| p.zo()).unwrap_or(0.0);
                Ok(JsValue::L(libm::mjs(bo)))
            }
            "Math.log" => {
                let bo = n.fv().map(|p| p.zo()).unwrap_or(0.0);
                Ok(JsValue::L(libm::log(bo)))
            }
            "Math.sign" => {
                let bo = n.fv().map(|p| p.zo()).unwrap_or(0.0);
                Ok(JsValue::L(if bo > 0.0 { 1.0 } else if bo < 0.0 { -1.0 } else { 0.0 }))
            }
            "Math.trunc" => {
                let bo = n.fv().map(|p| p.zo()).unwrap_or(0.0);
                Ok(JsValue::L(pwe(bo)))
            }

            
            "parseInt" => {
                let e = n.fv().map(|p| p.to_string()).age();
                let bo: f64 = e.em().parse().unwrap_or(f64::Lx);
                Ok(JsValue::L(pwe(bo)))
            }
            "parseFloat" => {
                let e = n.fv().map(|p| p.to_string()).age();
                let bo: f64 = e.em().parse().unwrap_or(f64::Lx);
                Ok(JsValue::L(bo))
            }
            "isNaN" => {
                let bo = n.fv().map(|p| p.zo()).unwrap_or(f64::Lx);
                Ok(JsValue::Cb(bo.ogj()))
            }
            "isFinite" => {
                let bo = n.fv().map(|p| p.zo()).unwrap_or(f64::Lx);
                Ok(JsValue::Cb(bo.dsg()))
            }

            
            "String" => Ok(JsValue::String(n.fv().map(|p| p.to_string()).age())),
            "Number" => Ok(JsValue::L(n.fv().map(|p| p.zo()).unwrap_or(0.0))),
            "Boolean" => Ok(JsValue::Cb(n.fv().map(|p| p.ezy()).unwrap_or(false))),
            "Array" => Ok(JsValue::U(n)),
            "Object" => Ok(JsValue::Cw(BTreeMap::new())),

            
            "encodeURIComponent" => {
                let e = n.fv().map(|p| p.to_string()).age();
                let mut ckd = String::new();
                for o in e.bf() {
                    if o.bvb() || b"-_.!~*'()".contains(&o) {
                        ckd.push(o as char);
                    } else {
                        ckd.t(&format!("%{:02X}", o));
                    }
                }
                Ok(JsValue::String(ckd))
            }
            "decodeURIComponent" => {
                let e = n.fv().map(|p| p.to_string()).age();
                let mut aoq = Vec::new();
                let bf = e.as_bytes();
                let mut a = 0;
                while a < bf.len() {
                    if bf[a] == b'%' && a + 2 < bf.len() {
                        if let Ok(o) = u8::wa(core::str::jg(&bf[a+1..a+3]).unwrap_or("00"), 16) {
                            aoq.push(o);
                            a += 3;
                            continue;
                        }
                    }
                    aoq.push(bf[a]);
                    a += 1;
                }
                Ok(JsValue::String(String::jg(aoq).age()))
            }

            
            "JSON.parse" => {
                let e = n.fv().map(|p| p.to_string()).age();
                Ok(self.lso(&e))
            }
            "JSON.stringify" => {
                let ap = n.fv().abn().unwrap_or(JsValue::Ba);
                Ok(JsValue::String(self.mhu(&ap)))
            }

            
            "document.getElementById" | "document.querySelector" | "document.querySelectorAll" => {
                
                let yct = n.fv().map(|p| p.to_string()).age();
                let mut ij = BTreeMap::new();
                ij.insert("innerHTML".to_string(), JsValue::String(String::new()));
                ij.insert("textContent".to_string(), JsValue::String(String::new()));
                ij.insert("className".to_string(), JsValue::String(String::new()));
                ij.insert("id".to_string(), JsValue::String(String::new()));
                ij.insert("tagName".to_string(), JsValue::String("DIV".to_string()));
                ij.insert("style".to_string(), JsValue::Cw(BTreeMap::new()));
                ij.insert("children".to_string(), JsValue::U(Vec::new()));
                ij.insert("parentNode".to_string(), JsValue::Gm);
                ij.insert("setAttribute".to_string(), JsValue::H("element.setAttribute".to_string()));
                ij.insert("getAttribute".to_string(), JsValue::H("element.getAttribute".to_string()));
                ij.insert("appendChild".to_string(), JsValue::H("element.appendChild".to_string()));
                ij.insert("removeChild".to_string(), JsValue::H("element.removeChild".to_string()));
                ij.insert("addEventListener".to_string(), JsValue::H("element.addEventListener".to_string()));
                ij.insert("classList".to_string(), JsValue::Cw({
                    let mut cl = BTreeMap::new();
                    cl.insert("add".to_string(), JsValue::H("classList.add".to_string()));
                    cl.insert("remove".to_string(), JsValue::H("classList.remove".to_string()));
                    cl.insert("toggle".to_string(), JsValue::H("classList.toggle".to_string()));
                    cl.insert("contains".to_string(), JsValue::H("classList.contains".to_string()));
                    cl
                }));
                if j == "document.querySelectorAll" {
                    Ok(JsValue::U(vec![JsValue::Cw(ij)]))
                } else {
                    Ok(JsValue::Cw(ij))
                }
            }
            "document.createElement" => {
                let ll = n.fv().map(|p| p.to_string()).unwrap_or("div".to_string());
                let mut ij = BTreeMap::new();
                ij.insert("tagName".to_string(), JsValue::String(ll.idx()));
                ij.insert("innerHTML".to_string(), JsValue::String(String::new()));
                ij.insert("textContent".to_string(), JsValue::String(String::new()));
                ij.insert("className".to_string(), JsValue::String(String::new()));
                ij.insert("style".to_string(), JsValue::Cw(BTreeMap::new()));
                ij.insert("children".to_string(), JsValue::U(Vec::new()));
                ij.insert("appendChild".to_string(), JsValue::H("element.appendChild".to_string()));
                ij.insert("addEventListener".to_string(), JsValue::H("element.addEventListener".to_string()));
                Ok(JsValue::Cw(ij))
            }
            "document.createTextNode" => {
                let text = n.fv().map(|p| p.to_string()).age();
                Ok(JsValue::String(text))
            }
            "document.write" => {
                let text = n.fv().map(|p| p.to_string()).age();
                self.ffp.push(format!("[document.write] {}", text));
                Ok(JsValue::Ba)
            }

            
            "element.appendChild" | "element.removeChild" | "element.setAttribute" |
            "element.getAttribute" | "element.addEventListener" |
            "classList.add" | "classList.remove" | "classList.toggle" => Ok(JsValue::Ba),
            "classList.contains" => Ok(JsValue::Cb(false)),

            
            "String.toUpperCase" => {
                if let Some(JsValue::String(e)) = afw.as_ref().efx(n.fv()) {
                    Ok(JsValue::String(e.idx()))
                } else { Ok(JsValue::Ba) }
            }
            "String.toLowerCase" => {
                if let Some(JsValue::String(e)) = afw.as_ref().efx(n.fv()) {
                    Ok(JsValue::String(e.aqn()))
                } else { Ok(JsValue::Ba) }
            }
            "String.trim" => {
                if let Some(JsValue::String(e)) = afw.as_ref().efx(n.fv()) {
                    Ok(JsValue::String(e.em().to_string()))
                } else { Ok(JsValue::Ba) }
            }
            "String.trimStart" => {
                if let Some(JsValue::String(e)) = afw.as_ref().efx(n.fv()) {
                    Ok(JsValue::String(e.ifa().to_string()))
                } else { Ok(JsValue::Ba) }
            }
            "String.trimEnd" => {
                if let Some(JsValue::String(e)) = afw.as_ref().efx(n.fv()) {
                    Ok(JsValue::String(e.eke().to_string()))
                } else { Ok(JsValue::Ba) }
            }
            "String.includes" => {
                if let Some(JsValue::String(e)) = afw.as_ref() {
                    let anw = n.fv().map(|p| p.to_string()).age();
                    Ok(JsValue::Cb(e.contains(&anw)))
                } else { Ok(JsValue::Cb(false)) }
            }
            "String.indexOf" => {
                if let Some(JsValue::String(e)) = afw.as_ref() {
                    let anw = n.fv().map(|p| p.to_string()).age();
                    Ok(JsValue::L(e.du(&anw).map(|a| a as f64).unwrap_or(-1.0)))
                } else { Ok(JsValue::L(-1.0)) }
            }
            "String.lastIndexOf" => {
                if let Some(JsValue::String(e)) = afw.as_ref() {
                    let anw = n.fv().map(|p| p.to_string()).age();
                    Ok(JsValue::L(e.bhx(&anw).map(|a| a as f64).unwrap_or(-1.0)))
                } else { Ok(JsValue::L(-1.0)) }
            }
            "String.startsWith" => {
                if let Some(JsValue::String(e)) = afw.as_ref() {
                    let adx = n.fv().map(|p| p.to_string()).age();
                    Ok(JsValue::Cb(e.cj(&adx)))
                } else { Ok(JsValue::Cb(false)) }
            }
            "String.endsWith" => {
                if let Some(JsValue::String(e)) = afw.as_ref() {
                    let cif = n.fv().map(|p| p.to_string()).age();
                    Ok(JsValue::Cb(e.pp(&cif)))
                } else { Ok(JsValue::Cb(false)) }
            }
            "String.slice" | "String.substring" => {
                if let Some(JsValue::String(e)) = afw.as_ref() {
                    let ay = n.fv().map(|p| p.zo() as i64).unwrap_or(0);
                    let ci = n.get(1).map(|p| p.zo() as i64);
                    let len = e.len() as i64;
                    let ay = if ay < 0 { (len + ay).am(0) as usize } else { (ay as usize).v(e.len()) };
                    let ci = match ci {
                        Some(aa) if aa < 0 => (len + aa).am(0) as usize,
                        Some(aa) => (aa as usize).v(e.len()),
                        None => e.len(),
                    };
                    if ay <= ci {
                        Ok(JsValue::String(e[ay..ci].to_string()))
                    } else {
                        Ok(JsValue::String(String::new()))
                    }
                } else { Ok(JsValue::String(String::new())) }
            }
            "String.replace" => {
                if let Some(JsValue::String(e)) = afw.as_ref() {
                    let from = n.fv().map(|p| p.to_string()).age();
                    let wh = n.get(1).map(|p| p.to_string()).age();
                    Ok(JsValue::String(e.zjr(&from, &wh, 1)))
                } else { Ok(JsValue::String(String::new())) }
            }
            "String.split" => {
                if let Some(JsValue::String(e)) = afw.as_ref() {
                    let jol = n.fv().map(|p| p.to_string()).age();
                    let ek: Vec<JsValue> = if jol.is_empty() {
                        e.bw().map(|r| JsValue::String(r.to_string())).collect()
                    } else {
                        e.adk(&jol).map(|ai| JsValue::String(ai.to_string())).collect()
                    };
                    Ok(JsValue::U(ek))
                } else { Ok(JsValue::U(Vec::new())) }
            }
            "String.charAt" => {
                if let Some(JsValue::String(e)) = afw.as_ref() {
                    let w = n.fv().map(|p| p.zo() as usize).unwrap_or(0);
                    Ok(JsValue::String(e.bw().goc(w).map(|r| r.to_string()).age()))
                } else { Ok(JsValue::String(String::new())) }
            }
            "String.charCodeAt" => {
                if let Some(JsValue::String(e)) = afw.as_ref() {
                    let w = n.fv().map(|p| p.zo() as usize).unwrap_or(0);
                    Ok(JsValue::L(e.bw().goc(w).map(|r| r as u32 as f64).unwrap_or(f64::Lx)))
                } else { Ok(JsValue::L(f64::Lx)) }
            }
            "String.repeat" => {
                if let Some(JsValue::String(e)) = afw.as_ref() {
                    let az = n.fv().map(|p| p.zo() as usize).unwrap_or(0).v(10000);
                    Ok(JsValue::String(e.afd(az)))
                } else { Ok(JsValue::String(String::new())) }
            }
            "String.padStart" => {
                if let Some(JsValue::String(e)) = afw.as_ref() {
                    let dwp = n.fv().map(|p| p.zo() as usize).unwrap_or(0);
                    let ov = n.get(1).map(|p| p.to_string()).unwrap_or(" ".to_string());
                    let mut result = e.clone();
                    while result.len() < dwp { result = format!("{}{}", ov, result); }
                    Ok(JsValue::String(result[result.len().ao(dwp)..].to_string()))
                } else { Ok(JsValue::String(String::new())) }
            }
            "String.padEnd" => {
                if let Some(JsValue::String(e)) = afw.as_ref() {
                    let dwp = n.fv().map(|p| p.zo() as usize).unwrap_or(0);
                    let ov = n.get(1).map(|p| p.to_string()).unwrap_or(" ".to_string());
                    let mut result = e.clone();
                    while result.len() < dwp { result.t(&ov); }
                    result.dmu(dwp);
                    Ok(JsValue::String(result))
                } else { Ok(JsValue::String(String::new())) }
            }
            "String.concat" => {
                if let Some(JsValue::String(e)) = afw.as_ref() {
                    let mut result = e.clone();
                    for q in &n { result.t(&q.to_string()); }
                    Ok(JsValue::String(result))
                } else { Ok(JsValue::String(String::new())) }
            }
            "String.match" | "String.search" => Ok(JsValue::Gm), 

            
            "Number.toFixed" => {
                if let Some(JsValue::L(bo)) = afw.as_ref() {
                    let ird = n.fv().map(|p| p.zo() as usize).unwrap_or(0).v(20);
                    
                    let pv = libm::vke(10.0, ird as f64);
                    let wae = jmv(*bo * pv) / pv;
                    Ok(JsValue::String(format!("{:.prec$}", wae, zgd = ird)))
                } else { Ok(JsValue::String("NaN".to_string())) }
            }
            "Number.toString" => {
                if let Some(JsValue::L(bo)) = afw.as_ref() {
                    Ok(JsValue::String(format!("{}", bo)))
                } else { Ok(JsValue::String(String::new())) }
            }

            
            "Array.push" => {
                if let Some(JsValue::U(sy)) = afw.as_ref() {
                    let mut fos = sy.clone();
                    for q in n { fos.push(q); }
                    let len = fos.len() as f64;
                    Ok(JsValue::L(len))
                } else { Ok(JsValue::L(0.0)) }
            }
            "Array.pop" => {
                if let Some(JsValue::U(sy)) = afw.as_ref() {
                    let mut fos = sy.clone();
                    Ok(fos.pop().unwrap_or(JsValue::Ba))
                } else { Ok(JsValue::Ba) }
            }
            "Array.shift" => {
                if let Some(JsValue::U(sy)) = afw.as_ref() {
                    if sy.is_empty() { return Ok(JsValue::Ba); }
                    Ok(sy[0].clone())
                } else { Ok(JsValue::Ba) }
            }
            "Array.unshift" => {
                if let Some(JsValue::U(sy)) = afw.as_ref() {
                    Ok(JsValue::L((sy.len() + n.len()) as f64))
                } else { Ok(JsValue::L(0.0)) }
            }
            "Array.join" => {
                if let Some(JsValue::U(sy)) = afw.as_ref() {
                    let jol = n.fv().map(|p| p.to_string()).unwrap_or(",".to_string());
                    let ek: Vec<String> = sy.iter().map(|p| p.to_string()).collect();
                    Ok(JsValue::String(ek.rr(&jol)))
                } else { Ok(JsValue::String(String::new())) }
            }
            "Array.reverse" => {
                if let Some(JsValue::U(sy)) = afw.as_ref() {
                    let mut fos = sy.clone();
                    fos.dbh();
                    Ok(JsValue::U(fos))
                } else { Ok(JsValue::U(Vec::new())) }
            }
            "Array.indexOf" => {
                if let Some(JsValue::U(sy)) = afw.as_ref() {
                    let anw = n.fv().abn().unwrap_or(JsValue::Ba);
                    let mcv = anw.to_string();
                    for (a, item) in sy.iter().cf() {
                        if item.to_string() == mcv { return Ok(JsValue::L(a as f64)); }
                    }
                    Ok(JsValue::L(-1.0))
                } else { Ok(JsValue::L(-1.0)) }
            }
            "Array.includes" => {
                if let Some(JsValue::U(sy)) = afw.as_ref() {
                    let anw = n.fv().abn().unwrap_or(JsValue::Ba);
                    let mcv = anw.to_string();
                    Ok(JsValue::Cb(sy.iter().any(|p| p.to_string() == mcv)))
                } else { Ok(JsValue::Cb(false)) }
            }
            "Array.slice" => {
                if let Some(JsValue::U(sy)) = afw.as_ref() {
                    let ay = n.fv().map(|p| p.zo() as i64).unwrap_or(0);
                    let ci = n.get(1).map(|p| p.zo() as i64);
                    let len = sy.len() as i64;
                    let ay = if ay < 0 { (len + ay).am(0) as usize } else { (ay as usize).v(sy.len()) };
                    let ci = match ci {
                        Some(aa) if aa < 0 => (len + aa).am(0) as usize,
                        Some(aa) => (aa as usize).v(sy.len()),
                        None => sy.len(),
                    };
                    Ok(JsValue::U(sy[ay..ci].ip()))
                } else { Ok(JsValue::U(Vec::new())) }
            }
            "Array.concat" => {
                if let Some(JsValue::U(sy)) = afw.as_ref() {
                    let mut result = sy.clone();
                    for q in n {
                        if let JsValue::U(gq) = q { result.lg(gq); }
                        else { result.push(q); }
                    }
                    Ok(JsValue::U(result))
                } else { Ok(JsValue::U(Vec::new())) }
            }
            "Array.map" | "Array.filter" | "Array.forEach" | "Array.find" |
            "Array.some" | "Array.every" | "Array.sort" | "Array.fill" |
            "Array.flat" | "Array.reduce" => {
                
                if let Some(JsValue::U(sy)) = afw.as_ref() {
                    Ok(JsValue::U(sy.clone()))
                } else { Ok(JsValue::U(Vec::new())) }
            }

            _ => Err(format!("Unknown native function: {}", j)),
        }
    }

    
    fn lso(&self, e: &str) -> JsValue {
        let e = e.em();
        if e.is_empty() { return JsValue::Gm; }
        match e.as_bytes()[0] {
            b'"' => {
                
                if e.len() >= 2 && e.pp('"') {
                    JsValue::String(e[1..e.len()-1].to_string())
                } else { JsValue::Gm }
            }
            b'{' => {
                
                let mut map = BTreeMap::new();
                let ff = &e[1..e.len().ao(1)];
                let mut eo = 0i32;
                let mut ay = 0;
                let mut ch = Vec::new();
                for (a, r) in ff.bw().cf() {
                    match r {
                        '{' | '[' => eo += 1,
                        '}' | ']' => eo -= 1,
                        ',' if eo == 0 => { ch.push(&ff[ay..a]); ay = a + 1; }
                        _ => {}
                    }
                }
                if ay < ff.len() { ch.push(&ff[ay..]); }
                for bt in ch {
                    let bt = bt.em();
                    if let Some(cpj) = bt.du(':') {
                        let bs = bt[..cpj].em().dcz('"');
                        let ap = bt[cpj+1..].em();
                        map.insert(bs.to_string(), self.lso(ap));
                    }
                }
                JsValue::Cw(map)
            }
            b'[' => {
                
                let ff = &e[1..e.len().ao(1)];
                let mut eo = 0i32;
                let mut ay = 0;
                let mut pj = Vec::new();
                for (a, r) in ff.bw().cf() {
                    match r {
                        '{' | '[' => eo += 1,
                        '}' | ']' => eo -= 1,
                        ',' if eo == 0 => { pj.push(&ff[ay..a]); ay = a + 1; }
                        _ => {}
                    }
                }
                if !ff.em().is_empty() { pj.push(&ff[ay..]); }
                JsValue::U(pj.iter().map(|a| self.lso(a.em())).collect())
            }
            b't' => JsValue::Cb(true),
            b'f' => JsValue::Cb(false),
            b'n' => JsValue::Gm,
            _ => {
                
                if let Ok(bo) = e.parse::<f64>() {
                    JsValue::L(bo)
                } else { JsValue::Gm }
            }
        }
    }

    
    fn mhu(&self, ap: &JsValue) -> String {
        match ap {
            JsValue::Ba | JsValue::Gm => "null".to_string(),
            JsValue::Cb(o) => if *o { "true" } else { "false" }.to_string(),
            JsValue::L(bo) => format!("{}", bo),
            JsValue::String(e) => format!("\"{}\"", e.replace('\\', "\\\\").replace('"', "\\\"")),
            JsValue::U(sy) => {
                let pj: Vec<String> = sy.iter().map(|p| self.mhu(p)).collect();
                format!("[{}]", pj.rr(","))
            }
            JsValue::Cw(map) => {
                let ch: Vec<String> = map.iter()
                    .map(|(eh, p)| format!("\"{}\":{}", eh, self.mhu(p)))
                    .collect();
                format!("{{{}}}", ch.rr(","))
            }
            JsValue::Bs(..) | JsValue::H(_) => "null".to_string(),
        }
    }
}





#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    L(f64),
    String(String),
    Lp(String),
    Bx(String),
    Fb(String),
    Da(char),
    Im,
}

fn fwz(aj: &str) -> Result<Vec<Token>, String> {
    let mut eb = Vec::new();
    let bw: Vec<char> = aj.bw().collect();
    let mut a = 0;
    
    while a < bw.len() {
        let r = bw[a];
        
        
        if r.fme() {
            a += 1;
            continue;
        }
        
        
        if r == '/' && a + 1 < bw.len() {
            if bw[a + 1] == '/' {
                
                while a < bw.len() && bw[a] != '\n' {
                    a += 1;
                }
                continue;
            }
            if bw[a + 1] == '*' {
                
                a += 2;
                while a + 1 < bw.len() && !(bw[a] == '*' && bw[a + 1] == '/') {
                    a += 1;
                }
                a += 2;
                continue;
            }
        }
        
        
        if r.atb() || (r == '.' && a + 1 < bw.len() && bw[a + 1].atb()) {
            let ay = a;
            while a < bw.len() && (bw[a].atb() || bw[a] == '.') {
                a += 1;
            }
            let ajh: String = bw[ay..a].iter().collect();
            let num: f64 = ajh.parse().unwrap_or(0.0);
            eb.push(Token::L(num));
            continue;
        }
        
        
        if r == '"' || r == '\'' {
            let cgw = r;
            a += 1;
            let ay = a;
            while a < bw.len() && bw[a] != cgw {
                if bw[a] == '\\' && a + 1 < bw.len() {
                    a += 2;
                } else {
                    a += 1;
                }
            }
            let e: String = bw[ay..a].iter().collect();
            eb.push(Token::String(e));
            a += 1; 
            continue;
        }
        
        
        if r.jaz() || r == '_' || r == '$' {
            let ay = a;
            while a < bw.len() && (bw[a].etb() || bw[a] == '_' || bw[a] == '$') {
                a += 1;
            }
            let od: String = bw[ay..a].iter().collect();
            let fmj = ["var", "let", "const", "function", "if", "else", "for", "while", 
                          "return", "true", "false", "null", "undefined", "typeof", "new"];
            if fmj.contains(&od.as_str()) {
                eb.push(Token::Bx(od));
            } else {
                eb.push(Token::Lp(od));
            }
            continue;
        }
        
        
        let pwn: String = bw[a..].iter().take(2).collect();
        let psy: String = bw[a..].iter().take(3).collect();
        
        if ["===", "!=="].contains(&psy.as_str()) {
            eb.push(Token::Fb(psy));
            a += 3;
            continue;
        }
        
        if ["==", "!=", "<=", ">=", "&&", "||", "++", "--", "+=", "-=", "*=", "/=", "=>"]
            .contains(&pwn.as_str()) {
            eb.push(Token::Fb(pwn));
            a += 2;
            continue;
        }
        
        
        if "+-*/%<>=!&|?:".contains(r) {
            eb.push(Token::Fb(r.to_string()));
            a += 1;
            continue;
        }
        
        
        if "{}[]();,.".contains(r) {
            eb.push(Token::Da(r));
            a += 1;
            continue;
        }
        
        
        a += 1;
    }
    
    eb.push(Token::Im);
    Ok(eb)
}





#[derive(Debug, Clone)]
pub enum Statement {
    Bvr(String, Option<Expr>),
    Expr(Expr),
    Gx(Expr, Vec<Statement>, Option<Vec<Statement>>),
    La(Expr, Vec<Statement>),
    Ll(Option<Box<Statement>>, Option<Expr>, Option<Expr>, Vec<Statement>),
    Hd(Option<Expr>),
    Bs(String, Vec<String>, String),
    Dj(Vec<Statement>),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Th(JsValue),
    Lp(String),
    Rl(String, Box<Expr>, Box<Expr>),
    Baf(String, Box<Expr>),
    En(Box<Expr>, Vec<Expr>),
    Avl(Box<Expr>, String),
    Index(Box<Expr>, Box<Expr>),
    U(Vec<Expr>),
    Cw(Vec<(String, Expr)>),
    Vk(String, Box<Expr>),
    Bud(Box<Expr>, Box<Expr>, Box<Expr>),
}

fn parse(eb: &[Token]) -> Result<Vec<Statement>, String> {
    let mut parser = Parser { eb, u: 0 };
    parser.lsw()
}

struct Parser<'a> {
    eb: &'a [Token],
    u: usize,
}

impl<'a> Parser<'a> {
    fn cv(&self) -> &Token {
        self.eb.get(self.u).unwrap_or(&Token::Im)
    }
    
    fn nb(&mut self) {
        if self.u < self.eb.len() {
            self.u += 1;
        }
    }
    
    fn cem(&mut self, r: char) -> Result<(), String> {
        if let Token::Da(ai) = self.cv() {
            if *ai == r {
                self.nb();
                return Ok(());
            }
        }
        Err(format!("Expected '{}'", r))
    }
    
    fn lsw(&mut self) -> Result<Vec<Statement>, String> {
        let mut boq = Vec::new();
        
        while !oh!(self.cv(), Token::Im | Token::Da('}')) {
            boq.push(self.oun()?);
        }
        
        Ok(boq)
    }
    
    fn oun(&mut self) -> Result<Statement, String> {
        match self.cv() {
            Token::Bx(yo) if yo == "var" || yo == "let" || yo == "const" => {
                self.nb();
                if let Token::Lp(j) = self.cv().clone() {
                    self.nb();
                    let bn = if let Token::Fb(op) = self.cv() {
                        if op == "=" {
                            self.nb();
                            Some(self.cgk()?)
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    if let Token::Da(';') = self.cv() {
                        self.nb();
                    }
                    Ok(Statement::Bvr(j, bn))
                } else {
                    Err("Expected identifier after var".to_string())
                }
            }
            Token::Bx(yo) if yo == "if" => {
                self.nb();
                self.cem('(')?;
                let mo = self.cgk()?;
                self.cem(')')?;
                let cne = self.lsf()?;
                let ckc = if let Token::Bx(yo) = self.cv() {
                    if yo == "else" {
                        self.nb();
                        Some(self.lsf()?)
                    } else {
                        None
                    }
                } else {
                    None
                };
                Ok(Statement::Gx(mo, cne, ckc))
            }
            Token::Bx(yo) if yo == "while" => {
                self.nb();
                self.cem('(')?;
                let mo = self.cgk()?;
                self.cem(')')?;
                let gj = self.lsf()?;
                Ok(Statement::La(mo, gj))
            }
            Token::Bx(yo) if yo == "return" => {
                self.nb();
                let bn = if let Token::Da(';') | Token::Da('}') = self.cv() {
                    None
                } else {
                    Some(self.cgk()?)
                };
                if let Token::Da(';') = self.cv() {
                    self.nb();
                }
                Ok(Statement::Hd(bn))
            }
            Token::Bx(yo) if yo == "function" => {
                self.nb();
                if let Token::Lp(j) = self.cv().clone() {
                    self.nb();
                    self.cem('(')?;
                    let oi = self.lss()?;
                    self.cem(')')?;
                    self.cem('{')?;
                    
                    let gj = self.vbx()?;
                    Ok(Statement::Bs(j, oi, gj))
                } else {
                    Err("Expected function name".to_string())
                }
            }
            Token::Da('{') => {
                self.nb();
                let boq = self.lsw()?;
                self.cem('}')?;
                Ok(Statement::Dj(boq))
            }
            _ => {
                let expr = self.cgk()?;
                if let Token::Da(';') = self.cv() {
                    self.nb();
                }
                Ok(Statement::Expr(expr))
            }
        }
    }
    
    fn lsf(&mut self) -> Result<Vec<Statement>, String> {
        if let Token::Da('{') = self.cv() {
            self.nb();
            let boq = self.lsw()?;
            self.cem('}')?;
            Ok(boq)
        } else {
            Ok(vec![self.oun()?])
        }
    }
    
    fn vbx(&mut self) -> Result<String, String> {
        
        let mut eo = 1;
        let ay = self.u;
        while eo > 0 && !oh!(self.cv(), Token::Im) {
            match self.cv() {
                Token::Da('{') => eo += 1,
                Token::Da('}') => eo -= 1,
                _ => {}
            }
            if eo > 0 {
                self.nb();
            }
        }
        self.nb(); 
        Ok(String::new()) 
    }
    
    fn lss(&mut self) -> Result<Vec<String>, String> {
        let mut oi = Vec::new();
        while let Token::Lp(j) = self.cv().clone() {
            oi.push(j);
            self.nb();
            if let Token::Da(',') = self.cv() {
                self.nb();
            } else {
                break;
            }
        }
        Ok(oi)
    }
    
    fn cgk(&mut self) -> Result<Expr, String> {
        self.veb()
    }
    
    fn veb(&mut self) -> Result<Expr, String> {
        let mo = self.lsr()?;
        if let Token::Fb(op) = self.cv() {
            if op == "?" {
                self.nb();
                let mkp = self.cgk()?;
                if let Token::Fb(op) = self.cv() {
                    if op == ":" {
                        self.nb();
                        let ksz = self.cgk()?;
                        return Ok(Expr::Bud(Box::new(mo), Box::new(mkp), Box::new(ksz)));
                    }
                }
            }
        }
        Ok(mo)
    }
    
    fn lsr(&mut self) -> Result<Expr, String> {
        let mut fd = self.hui()?;
        while let Token::Fb(op) = self.cv() {
            if op == "||" {
                let op = op.clone();
                self.nb();
                let hw = self.hui()?;
                fd = Expr::Rl(op, Box::new(fd), Box::new(hw));
            } else {
                break;
            }
        }
        Ok(fd)
    }
    
    fn hui(&mut self) -> Result<Expr, String> {
        let mut fd = self.oug()?;
        while let Token::Fb(op) = self.cv() {
            if op == "&&" {
                let op = op.clone();
                self.nb();
                let hw = self.oug()?;
                fd = Expr::Rl(op, Box::new(fd), Box::new(hw));
            } else {
                break;
            }
        }
        Ok(fd)
    }
    
    fn oug(&mut self) -> Result<Expr, String> {
        let mut fd = self.huj()?;
        while let Token::Fb(op) = self.cv() {
            if op == "==" || op == "!=" || op == "===" || op == "!==" {
                let op = op.clone();
                self.nb();
                let hw = self.huj()?;
                fd = Expr::Rl(op, Box::new(fd), Box::new(hw));
            } else {
                break;
            }
        }
        Ok(fd)
    }
    
    fn huj(&mut self) -> Result<Expr, String> {
        let mut fd = self.got()?;
        while let Token::Fb(op) = self.cv() {
            if op == "<" || op == ">" || op == "<=" || op == ">=" {
                let op = op.clone();
                self.nb();
                let hw = self.got()?;
                fd = Expr::Rl(op, Box::new(fd), Box::new(hw));
            } else {
                break;
            }
        }
        Ok(fd)
    }
    
    fn got(&mut self) -> Result<Expr, String> {
        let mut fd = self.huk()?;
        while let Token::Fb(op) = self.cv() {
            if op == "+" || op == "-" {
                let op = op.clone();
                self.nb();
                let hw = self.huk()?;
                fd = Expr::Rl(op, Box::new(fd), Box::new(hw));
            } else {
                break;
            }
        }
        Ok(fd)
    }
    
    fn huk(&mut self) -> Result<Expr, String> {
        let mut fd = self.fqk()?;
        while let Token::Fb(op) = self.cv() {
            if op == "*" || op == "/" || op == "%" {
                let op = op.clone();
                self.nb();
                let hw = self.fqk()?;
                fd = Expr::Rl(op, Box::new(fd), Box::new(hw));
            } else {
                break;
            }
        }
        Ok(fd)
    }
    
    fn fqk(&mut self) -> Result<Expr, String> {
        if let Token::Fb(op) = self.cv() {
            if op == "!" || op == "-" || op == "+" {
                let op = op.clone();
                self.nb();
                let htq = self.fqk()?;
                return Ok(Expr::Baf(op, Box::new(htq)));
            }
        }
        if let Token::Bx(yo) = self.cv() {
            if yo == "typeof" {
                self.nb();
                let htq = self.fqk()?;
                return Ok(Expr::Baf("typeof".to_string(), Box::new(htq)));
            }
        }
        self.hum()
    }
    
    fn hum(&mut self) -> Result<Expr, String> {
        let mut expr = self.lsu()?;
        
        loop {
            match self.cv() {
                Token::Da('.') => {
                    self.nb();
                    if let Token::Lp(frl) = self.cv().clone() {
                        self.nb();
                        expr = Expr::Avl(Box::new(expr), frl);
                    } else {
                        break;
                    }
                }
                Token::Da('[') => {
                    self.nb();
                    let index = self.cgk()?;
                    self.cem(']')?;
                    expr = Expr::Index(Box::new(expr), Box::new(index));
                }
                Token::Da('(') => {
                    self.nb();
                    let n = self.vbr()?;
                    self.cem(')')?;
                    expr = Expr::En(Box::new(expr), n);
                }
                _ => break,
            }
        }
        
        Ok(expr)
    }
    
    fn lsu(&mut self) -> Result<Expr, String> {
        match self.cv().clone() {
            Token::L(bo) => {
                self.nb();
                Ok(Expr::Th(JsValue::L(bo)))
            }
            Token::String(e) => {
                self.nb();
                Ok(Expr::Th(JsValue::String(e)))
            }
            Token::Bx(yo) if yo == "true" => {
                self.nb();
                Ok(Expr::Th(JsValue::Cb(true)))
            }
            Token::Bx(yo) if yo == "false" => {
                self.nb();
                Ok(Expr::Th(JsValue::Cb(false)))
            }
            Token::Bx(yo) if yo == "null" => {
                self.nb();
                Ok(Expr::Th(JsValue::Gm))
            }
            Token::Bx(yo) if yo == "undefined" => {
                self.nb();
                Ok(Expr::Th(JsValue::Ba))
            }
            Token::Lp(j) => {
                self.nb();
                
                if let Token::Fb(op) = self.cv() {
                    if op == "=" {
                        self.nb();
                        let bn = self.cgk()?;
                        return Ok(Expr::Vk(j, Box::new(bn)));
                    }
                }
                Ok(Expr::Lp(j))
            }
            Token::Da('(') => {
                self.nb();
                let expr = self.cgk()?;
                self.cem(')')?;
                Ok(expr)
            }
            Token::Da('[') => {
                self.nb();
                let bgw = self.vbs()?;
                self.cem(']')?;
                Ok(Expr::U(bgw))
            }
            Token::Da('{') => {
                self.nb();
                let gpy = self.vcz()?;
                self.cem('}')?;
                Ok(Expr::Cw(gpy))
            }
            _ => Err(format!("Unexpected token: {:?}", self.cv())),
        }
    }
    
    fn vbr(&mut self) -> Result<Vec<Expr>, String> {
        let mut n = Vec::new();
        if !oh!(self.cv(), Token::Da(')')) {
            n.push(self.cgk()?);
            while let Token::Da(',') = self.cv() {
                self.nb();
                n.push(self.cgk()?);
            }
        }
        Ok(n)
    }
    
    fn vbs(&mut self) -> Result<Vec<Expr>, String> {
        let mut bgw = Vec::new();
        if !oh!(self.cv(), Token::Da(']')) {
            bgw.push(self.cgk()?);
            while let Token::Da(',') = self.cv() {
                self.nb();
                if oh!(self.cv(), Token::Da(']')) {
                    break;
                }
                bgw.push(self.cgk()?);
            }
        }
        Ok(bgw)
    }
    
    fn vcz(&mut self) -> Result<Vec<(String, Expr)>, String> {
        let mut gpy = Vec::new();
        while !oh!(self.cv(), Token::Da('}')) {
            let bs = match self.cv().clone() {
                Token::Lp(e) | Token::String(e) => {
                    self.nb();
                    e
                }
                _ => return Err("Expected property name".to_string()),
            };
            self.cem(':')?;
            let bn = self.cgk()?;
            gpy.push((bs, bn));
            if let Token::Da(',') = self.cv() {
                self.nb();
            } else {
                break;
            }
        }
        Ok(gpy)
    }
}
