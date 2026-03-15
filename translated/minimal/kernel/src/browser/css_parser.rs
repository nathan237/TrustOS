




use alloc::string::{String, Gd};
use alloc::vec::Vec;
use alloc::collections::BTreeMap;


#[derive(Debug, Clone)]
pub struct Mj {
    pub bib: Vec<Aqj>,
}


#[derive(Debug, Clone)]
pub struct Aqj {
    pub fud: Vec<Yt>,
    pub dps: Vec<Sa>,
}


#[derive(Debug, Clone)]
pub struct Yt {
    pub bgw: Vec<SelectorPart>,
}


#[derive(Debug, Clone)]
pub enum SelectorPart {
    Azr(String),           
    Bdo(String),         
    Bjj(String),            
    Bvi,             
    Cau,            
    Bdm,                 
    Bbm,              
    Bst,               
    Bpn(String),        
    Ms(String, Option<String>), 
}


#[derive(Debug, Clone)]
pub struct Sa {
    pub jki: String,
    pub bn: CssValue,
    pub flg: bool,
}


#[derive(Debug, Clone)]
pub enum CssValue {
    Bx(String),              
    Color(u32),                   
    Acp(f32, LengthUnit),      
    L(f32),                  
    String(String),               
    Url(String),                  
    Cho(Vec<CssValue>),      
}


#[derive(Debug, Clone, Copy)]
pub enum LengthUnit {
    Cjc,     
    Cbp,     
    Rem,    
    Qk,
    Cpx,     
    Cpj,     
    Cjb,     
}


#[derive(Debug, Clone)]
pub struct Bzx {
    pub display: Display,
    pub s: u32,
    pub cdb: u32,
    pub asv: f32,
    pub svn: FontWeight,
    pub svm: FontStyle,
    pub xft: TextDecoration,
    pub xfp: TextAlign,
    pub adf: EdgeSizes,
    pub ob: EdgeSizes,
    pub dek: EdgeSizes,
    pub aia: u32,
    pub avh: f32,
    pub z: Option<f32>,
    pub ac: Option<f32>,
    pub dtb: Option<f32>,
    pub czx: Option<f32>,
    pub acg: f32,
    pub adh: f32,
    pub lrg: Overflow,
    pub qf: Position,
    pub xuj: WhiteSpace,
    pub xvb: WordBreak,
    pub gi: CursorStyle,
    pub qrp: Option<Bys>,
    pub xgc: TextTransform,
    pub uec: f32,
    pub ufw: ListStyleType,
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Overflow {
    Cpn,
    Cyi,
    Yq,
    Api,
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Position {
    Cnj,
    Dfv,
    Cri,
    Cxd,
    Diy,
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WhiteSpace {
    M,
    Dde,
    Des,
    Deu,
    Det,
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WordBreak {
    M,
    Csf,
    Das,
    Csg,
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CursorStyle {
    Default,
    Del,
    Text,
    Fw,
    Ddc,
    Bdw,
    Bwm,
}


#[derive(Debug, Clone, Copy)]
pub struct Bys {
    pub dtw: f32,
    pub dtx: f32,
    pub cou: f32,
    pub eyw: f32,
    pub s: u32,
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextTransform {
    None,
    Dkt,
    Dbk,
    Css,
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ListStyleType {
    Cax,
    Circle,
    Gb,
    Aay,
    None,
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Display {
    Dj,
    Aug,
    Czt,
    None,
    Cxe,
    Pn,
}


#[derive(Debug, Clone, Copy)]
pub enum FontWeight {
    M,
    Csd,
    Ddf(u16), 
}


#[derive(Debug, Clone, Copy)]
pub enum FontStyle {
    M,
    Dag,
    Ddl,
}


#[derive(Debug, Clone, Copy)]
pub enum TextDecoration {
    None,
    Dkq,
    Ddn,
    Dbh,
}


#[derive(Debug, Clone, Copy)]
pub enum TextAlign {
    Ap,
    Ca,
    Eo,
    Dam,
}


#[derive(Debug, Clone, Copy, Default)]
pub struct EdgeSizes {
    pub qc: f32,
    pub hw: f32,
    pub abm: f32,
    pub fd: f32,
}

impl Default for Bzx {
    fn default() -> Self {
        Self {
            display: Display::Dj,
            s: 0xFF000000,
            cdb: 0x00000000,
            asv: 16.0,
            svn: FontWeight::M,
            svm: FontStyle::M,
            xft: TextDecoration::None,
            xfp: TextAlign::Ap,
            adf: EdgeSizes::default(),
            ob: EdgeSizes::default(),
            dek: EdgeSizes::default(),
            aia: 0xFF000000,
            avh: 0.0,
            z: None,
            ac: None,
            dtb: None,
            czx: None,
            acg: 1.2,
            adh: 1.0,
            lrg: Overflow::Cpn,
            qf: Position::Cnj,
            xuj: WhiteSpace::M,
            xvb: WordBreak::M,
            gi: CursorStyle::Default,
            qrp: None,
            xgc: TextTransform::None,
            uec: 0.0,
            ufw: ListStyleType::Cax,
        }
    }
}


pub fn lsx(eoe: &str) -> Mj {
    let mut parser = CssParser::new(eoe);
    parser.lsx()
}


pub fn vcr(amx: &str) -> Vec<Sa> {
    let mut parser = CssParser::new(amx);
    parser.oue()
}


pub fn clt(bn: &str) -> Option<u32> {
    let bn = bn.em().aqn();
    
    
    match bn.as_str() {
        "aliceblue" => return Some(0xFFF0F8FF),
        "antiquewhite" => return Some(0xFFFAEBD7),
        "aqua" => return Some(0xFF00FFFF),
        "aquamarine" => return Some(0xFF7FFFD4),
        "azure" => return Some(0xFFF0FFFF),
        "beige" => return Some(0xFFF5F5DC),
        "bisque" => return Some(0xFFFFE4C4),
        "black" => return Some(0xFF000000),
        "blanchedalmond" => return Some(0xFFFFEBCD),
        "blue" => return Some(0xFF0000FF),
        "blueviolet" => return Some(0xFF8A2BE2),
        "brown" => return Some(0xFFA52A2A),
        "burlywood" => return Some(0xFFDEB887),
        "cadetblue" => return Some(0xFF5F9EA0),
        "chartreuse" => return Some(0xFF7FFF00),
        "chocolate" => return Some(0xFFD2691E),
        "coral" => return Some(0xFFFF7F50),
        "cornflowerblue" => return Some(0xFF6495ED),
        "cornsilk" => return Some(0xFFFFF8DC),
        "crimson" => return Some(0xFFDC143C),
        "cyan" => return Some(0xFF00FFFF),
        "darkblue" => return Some(0xFF00008B),
        "darkcyan" => return Some(0xFF008B8B),
        "darkgoldenrod" => return Some(0xFFB8860B),
        "darkgray" | "darkgrey" => return Some(0xFFA9A9A9),
        "darkgreen" => return Some(0xFF006400),
        "darkkhaki" => return Some(0xFFBDB76B),
        "darkmagenta" => return Some(0xFF8B008B),
        "darkolivegreen" => return Some(0xFF556B2F),
        "darkorange" => return Some(0xFFFF8C00),
        "darkorchid" => return Some(0xFF9932CC),
        "darkred" => return Some(0xFF8B0000),
        "darksalmon" => return Some(0xFFE9967A),
        "darkseagreen" => return Some(0xFF8FBC8F),
        "darkslateblue" => return Some(0xFF483D8B),
        "darkslategray" | "darkslategrey" => return Some(0xFF2F4F4F),
        "darkturquoise" => return Some(0xFF00CED1),
        "darkviolet" => return Some(0xFF9400D3),
        "deeppink" => return Some(0xFFFF1493),
        "deepskyblue" => return Some(0xFF00BFFF),
        "dimgray" | "dimgrey" => return Some(0xFF696969),
        "dodgerblue" => return Some(0xFF1E90FF),
        "firebrick" => return Some(0xFFB22222),
        "floralwhite" => return Some(0xFFFFFAF0),
        "forestgreen" => return Some(0xFF228B22),
        "fuchsia" => return Some(0xFFFF00FF),
        "gainsboro" => return Some(0xFFDCDCDC),
        "ghostwhite" => return Some(0xFFF8F8FF),
        "gold" => return Some(0xFFFFD700),
        "goldenrod" => return Some(0xFFDAA520),
        "gray" | "grey" => return Some(0xFF808080),
        "green" => return Some(0xFF008000),
        "greenyellow" => return Some(0xFFADFF2F),
        "honeydew" => return Some(0xFFF0FFF0),
        "hotpink" => return Some(0xFFFF69B4),
        "indianred" => return Some(0xFFCD5C5C),
        "indigo" => return Some(0xFF4B0082),
        "ivory" => return Some(0xFFFFFFF0),
        "khaki" => return Some(0xFFF0E68C),
        "lavender" => return Some(0xFFE6E6FA),
        "lavenderblush" => return Some(0xFFFFF0F5),
        "lawngreen" => return Some(0xFF7CFC00),
        "lemonchiffon" => return Some(0xFFFFFACD),
        "lightblue" => return Some(0xFFADD8E6),
        "lightcoral" => return Some(0xFFF08080),
        "lightcyan" => return Some(0xFFE0FFFF),
        "lightgoldenrodyellow" => return Some(0xFFFAFAD2),
        "lightgray" | "lightgrey" => return Some(0xFFD3D3D3),
        "lightgreen" => return Some(0xFF90EE90),
        "lightpink" => return Some(0xFFFFB6C1),
        "lightsalmon" => return Some(0xFFFFA07A),
        "lightseagreen" => return Some(0xFF20B2AA),
        "lightskyblue" => return Some(0xFF87CEFA),
        "lightslategray" | "lightslategrey" => return Some(0xFF778899),
        "lightsteelblue" => return Some(0xFFB0C4DE),
        "lightyellow" => return Some(0xFFFFFFE0),
        "lime" => return Some(0xFF00FF00),
        "limegreen" => return Some(0xFF32CD32),
        "linen" => return Some(0xFFFAF0E6),
        "magenta" => return Some(0xFFFF00FF),
        "maroon" => return Some(0xFF800000),
        "mediumaquamarine" => return Some(0xFF66CDAA),
        "mediumblue" => return Some(0xFF0000CD),
        "mediumorchid" => return Some(0xFFBA55D3),
        "mediumpurple" => return Some(0xFF9370DB),
        "mediumseagreen" => return Some(0xFF3CB371),
        "mediumslateblue" => return Some(0xFF7B68EE),
        "mediumspringgreen" => return Some(0xFF00FA9A),
        "mediumturquoise" => return Some(0xFF48D1CC),
        "mediumvioletred" => return Some(0xFFC71585),
        "midnightblue" => return Some(0xFF191970),
        "mintcream" => return Some(0xFFF5FFFA),
        "mistyrose" => return Some(0xFFFFE4E1),
        "moccasin" => return Some(0xFFFFE4B5),
        "navajowhite" => return Some(0xFFFFDEAD),
        "navy" => return Some(0xFF000080),
        "oldlace" => return Some(0xFFFDF5E6),
        "olive" => return Some(0xFF808000),
        "olivedrab" => return Some(0xFF6B8E23),
        "orange" => return Some(0xFFFFA500),
        "orangered" => return Some(0xFFFF4500),
        "orchid" => return Some(0xFFDA70D6),
        "palegoldenrod" => return Some(0xFFEEE8AA),
        "palegreen" => return Some(0xFF98FB98),
        "paleturquoise" => return Some(0xFFAFEEEE),
        "palevioletred" => return Some(0xFFDB7093),
        "papayawhip" => return Some(0xFFFFEFD5),
        "peachpuff" => return Some(0xFFFFDAB9),
        "peru" => return Some(0xFFCD853F),
        "pink" => return Some(0xFFFFC0CB),
        "plum" => return Some(0xFFDDA0DD),
        "powderblue" => return Some(0xFFB0E0E6),
        "purple" => return Some(0xFF800080),
        "rebeccapurple" => return Some(0xFF663399),
        "red" => return Some(0xFFFF0000),
        "rosybrown" => return Some(0xFFBC8F8F),
        "royalblue" => return Some(0xFF4169E1),
        "saddlebrown" => return Some(0xFF8B4513),
        "salmon" => return Some(0xFFFA8072),
        "sandybrown" => return Some(0xFFF4A460),
        "seagreen" => return Some(0xFF2E8B57),
        "seashell" => return Some(0xFFFFF5EE),
        "sienna" => return Some(0xFFA0522D),
        "silver" => return Some(0xFFC0C0C0),
        "skyblue" => return Some(0xFF87CEEB),
        "slateblue" => return Some(0xFF6A5ACD),
        "slategray" | "slategrey" => return Some(0xFF708090),
        "snow" => return Some(0xFFFFFAFA),
        "springgreen" => return Some(0xFF00FF7F),
        "steelblue" => return Some(0xFF4682B4),
        "tan" => return Some(0xFFD2B48C),
        "teal" => return Some(0xFF008080),
        "thistle" => return Some(0xFFD8BFD8),
        "tomato" => return Some(0xFFFF6347),
        "turquoise" => return Some(0xFF40E0D0),
        "violet" => return Some(0xFFEE82EE),
        "wheat" => return Some(0xFFF5DEB3),
        "white" => return Some(0xFFFFFFFF),
        "whitesmoke" => return Some(0xFFF5F5F5),
        "yellow" => return Some(0xFFFFFF00),
        "yellowgreen" => return Some(0xFF9ACD32),
        "transparent" => return Some(0x00000000),
        _ => {}
    }
    
    
    if bn.cj('#') {
        let nu = &bn[1..];
        return match nu.len() {
            3 => {
                let m = u8::wa(&nu[0..1], 16).bq()?;
                let at = u8::wa(&nu[1..2], 16).bq()?;
                let o = u8::wa(&nu[2..3], 16).bq()?;
                Some(0xFF000000 | ((m as u32 * 17) << 16) | ((at as u32 * 17) << 8) | (o as u32 * 17))
            }
            4 => {
                
                let m = u8::wa(&nu[0..1], 16).bq()?;
                let at = u8::wa(&nu[1..2], 16).bq()?;
                let o = u8::wa(&nu[2..3], 16).bq()?;
                let q = u8::wa(&nu[3..4], 16).bq()?;
                Some(((q as u32 * 17) << 24) | ((m as u32 * 17) << 16) | ((at as u32 * 17) << 8) | (o as u32 * 17))
            }
            6 => {
                let m = u8::wa(&nu[0..2], 16).bq()?;
                let at = u8::wa(&nu[2..4], 16).bq()?;
                let o = u8::wa(&nu[4..6], 16).bq()?;
                Some(0xFF000000 | ((m as u32) << 16) | ((at as u32) << 8) | (o as u32))
            }
            8 => {
                let m = u8::wa(&nu[0..2], 16).bq()?;
                let at = u8::wa(&nu[2..4], 16).bq()?;
                let o = u8::wa(&nu[4..6], 16).bq()?;
                let q = u8::wa(&nu[6..8], 16).bq()?;
                Some(((q as u32) << 24) | ((m as u32) << 16) | ((at as u32) << 8) | (o as u32))
            }
            _ => None,
        };
    }
    
    
    if bn.cj("rgb") {
        let ay = bn.du('(')?;
        let ci = bn.du(')')?;
        let ff = &bn[ay + 1..ci];
        let ek: Vec<&str> = ff.adk(',').collect();
        
        if ek.len() >= 3 {
            let m: u8 = ek[0].em().parse().bq()?;
            let at: u8 = ek[1].em().parse().bq()?;
            let o: u8 = ek[2].em().parse().bq()?;
            let q: u8 = if ek.len() >= 4 {
                let dw: f32 = ek[3].em().parse().bq()?;
                (dw * 255.0) as u8
            } else {
                255
            };
            return Some(((q as u32) << 24) | ((m as u32) << 16) | ((at as u32) << 8) | (o as u32));
        }
    }
    
    
    if bn.cj("hsl") {
        let ay = bn.du('(')?;
        let ci = bn.du(')')?;
        let ff = &bn[ay + 1..ci];
        let ek: Vec<&str> = ff.adk(',').collect();
        if ek.len() >= 3 {
            let i: f32 = ek[0].em().bdd("deg").parse::<f32>().bq()?;
            let e: f32 = ek[1].em().bdd('%').parse::<f32>().bq()? / 100.0;
            let dm: f32 = ek[2].em().bdd('%').parse::<f32>().bq()? / 100.0;
            let q: f32 = if ek.len() >= 4 {
                ek[3].em().parse::<f32>().bq()?
            } else {
                1.0
            };
            let (m, at, o) = tqm(i, e, dm);
            let dw = (q * 255.0) as u8;
            return Some(((dw as u32) << 24) | ((m as u32) << 16) | ((at as u32) << 8) | (o as u32));
        }
    }
    
    None
}


fn tqm(i: f32, e: f32, dm: f32) -> (u8, u8, u8) {
    if e == 0.0 {
        let p = (dm * 255.0) as u8;
        return (p, p, p);
    }
    let aya = ((i % 360.0) + 360.0) % 360.0 / 360.0;
    let fm = if dm < 0.5 { dm * (1.0 + e) } else { dm + e - dm * e };
    let ai = 2.0 * dm - fm;
    let m = lcv(ai, fm, aya + 1.0 / 3.0);
    let at = lcv(ai, fm, aya);
    let o = lcv(ai, fm, aya - 1.0 / 3.0);
    ((m * 255.0) as u8, (at * 255.0) as u8, (o * 255.0) as u8)
}

fn lcv(ai: f32, fm: f32, mut ab: f32) -> f32 {
    if ab < 0.0 { ab += 1.0; }
    if ab > 1.0 { ab -= 1.0; }
    if ab < 1.0 / 6.0 { return ai + (fm - ai) * 6.0 * ab; }
    if ab < 1.0 / 2.0 { return fm; }
    if ab < 2.0 / 3.0 { return ai + (fm - ai) * (2.0 / 3.0 - ab) * 6.0; }
    ai
}


struct CssParser<'a> {
    input: &'a str,
    u: usize,
}

impl<'a> CssParser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, u: 0 }
    }
    
    fn lsx(&mut self) -> Mj {
        let mut bib = Vec::new();
        
        loop {
            self.wpj();
            if self.u >= self.input.len() {
                break;
            }
            
            
            if self.asp() == Some('@') {
                self.wpf();
                continue;
            }
            
            if let Some(agu) = self.vdl() {
                bib.push(agu);
            }
        }
        
        Mj { bib }
    }
    
    fn vdl(&mut self) -> Option<Aqj> {
        let fud = self.vdp();
        if fud.is_empty() {
            return None;
        }
        
        self.ayr();
        if !self.cwp('{') {
            return None;
        }
        
        let dps = self.oue();
        
        self.ayr();
        self.cwp('}');
        
        Some(Aqj { fud, dps })
    }
    
    fn vdp(&mut self) -> Vec<Yt> {
        let mut fud = Vec::new();
        
        loop {
            self.ayr();
            if let Some(bof) = self.vdo() {
                fud.push(bof);
            }
            
            self.ayr();
            if !self.cwp(',') {
                break;
            }
        }
        
        fud
    }
    
    fn vdo(&mut self) -> Option<Yt> {
        let mut bgw = Vec::new();
        
        loop {
            self.ayr();
            
            match self.asp()? {
                '{' | ',' => break,
                '*' => {
                    self.u += 1;
                    bgw.push(SelectorPart::Bvi);
                }
                '.' => {
                    self.u += 1;
                    let class = self.ege();
                    bgw.push(SelectorPart::Bdo(class));
                }
                '#' => {
                    self.u += 1;
                    let ad = self.ege();
                    bgw.push(SelectorPart::Bjj(ad));
                }
                ':' => {
                    self.u += 1;
                    if self.asp() == Some(':') {
                        self.u += 1; 
                    }
                    let dkw = self.ege();
                    bgw.push(SelectorPart::Bpn(dkw));
                }
                '[' => {
                    self.u += 1;
                    let qn = self.vbu();
                    bgw.push(qn);
                }
                '>' => {
                    self.u += 1;
                    bgw.push(SelectorPart::Bdm);
                }
                '+' => {
                    self.u += 1;
                    bgw.push(SelectorPart::Bbm);
                }
                '~' => {
                    self.u += 1;
                    bgw.push(SelectorPart::Bst);
                }
                r if r.jaz() || r == '-' || r == '_' => {
                    let ll = self.ege();
                    bgw.push(SelectorPart::Azr(ll));
                }
                _ => break,
            }
        }
        
        if bgw.is_empty() {
            None
        } else {
            Some(Yt { bgw })
        }
    }
    
    fn vbu(&mut self) -> SelectorPart {
        let mwr = self.ege();
        self.ayr();
        
        if self.cwp(']') {
            return SelectorPart::Ms(mwr, None);
        }
        
        
        while let Some(r) = self.asp() {
            if r == '"' || r == '\'' || r.etb() {
                break;
            }
            self.u += 1;
        }
        
        let bn = self.vdv();
        self.ayr();
        self.cwp(']');
        
        SelectorPart::Ms(mwr, Some(bn))
    }
    
    fn oue(&mut self) -> Vec<Sa> {
        let mut dps = Vec::new();
        
        loop {
            self.ayr();
            
            if self.asp() == Some('}') || self.u >= self.input.len() {
                break;
            }
            
            if let Some(aqy) = self.vcc() {
                dps.push(aqy);
            }
            
            self.ayr();
            self.cwp(';');
        }
        
        dps
    }
    
    fn vcc(&mut self) -> Option<Sa> {
        self.ayr();
        
        let jki = self.ege();
        if jki.is_empty() {
            return None;
        }
        
        self.ayr();
        if !self.cwp(':') {
            return None;
        }
        
        self.ayr();
        let (bn, flg) = self.vek();
        
        Some(Sa { jki, bn, flg })
    }
    
    fn vek(&mut self) -> (CssValue, bool) {
        self.ayr();
        
        let mut alv = Vec::new();
        let mut flg = false;
        
        loop {
            self.ayr();
            
            match self.asp() {
                None | Some(';') | Some('}') => break,
                Some('!') => {
                    self.u += 1;
                    let od = self.ege();
                    if od == "important" {
                        flg = true;
                    }
                    break;
                }
                Some(_) => {
                    if let Some(p) = self.vds() {
                        alv.push(p);
                    } else {
                        break;
                    }
                }
            }
        }
        
        let bn = if alv.len() == 1 {
            alv.dse().next().unwrap()
        } else {
            CssValue::Cho(alv)
        };
        
        (bn, flg)
    }
    
    fn vds(&mut self) -> Option<CssValue> {
        self.ayr();
        
        match self.asp()? {
            '#' => {
                
                let ay = self.u;
                self.u += 1;
                while let Some(r) = self.asp() {
                    if r.ofp() {
                        self.u += 1;
                    } else {
                        break;
                    }
                }
                let kjx = &self.input[ay..self.u];
                if let Some(s) = clt(kjx) {
                    Some(CssValue::Color(s))
                } else {
                    None
                }
            }
            '"' | '\'' => {
                
                let e = self.lsv();
                Some(CssValue::String(e))
            }
            r if r.atb() || r == '-' || r == '.' => {
                
                let (num, ifv) = self.vcy();
                if let Some(ifv) = ifv {
                    Some(CssValue::Acp(num, ifv))
                } else {
                    Some(CssValue::L(num))
                }
            }
            r if r.jaz() => {
                let ay = self.u;
                let od = self.ege();
                
                
                if od == "url" && self.cwp('(') {
                    self.ayr();
                    let url = if self.asp() == Some('"') || self.asp() == Some('\'') {
                        self.lsv()
                    } else {
                        let ay = self.u;
                        while let Some(r) = self.asp() {
                            if r == ')' { break; }
                            self.u += 1;
                        }
                        self.input[ay..self.u].to_string()
                    };
                    self.ayr();
                    self.cwp(')');
                    return Some(CssValue::Url(url));
                }
                
                
                if let Some(s) = clt(&od) {
                    return Some(CssValue::Color(s));
                }
                
                
                if (od == "rgb" || od == "rgba") && self.cwp('(') {
                    let szi = ay;
                    while let Some(r) = self.asp() {
                        if r == ')' { break; }
                        self.u += 1;
                    }
                    self.cwp(')');
                    let szj = &self.input[szi..self.u];
                    if let Some(s) = clt(szj) {
                        return Some(CssValue::Color(s));
                    }
                }
                
                Some(CssValue::Bx(od))
            }
            _ => None,
        }
    }
    
    fn vcy(&mut self) -> (f32, Option<LengthUnit>) {
        let ay = self.u;
        
        
        if self.asp() == Some('-') {
            self.u += 1;
        }
        
        
        while let Some(r) = self.asp() {
            if r.atb() || r == '.' {
                self.u += 1;
            } else {
                break;
            }
        }
        
        let ajh = &self.input[ay..self.u];
        let num: f32 = ajh.parse().unwrap_or(0.0);
        
        
        let xoe = self.u;
        while let Some(r) = self.asp() {
            if r.jaz() || r == '%' {
                self.u += 1;
            } else {
                break;
            }
        }
        
        let xof = &self.input[xoe..self.u];
        let ifv = match xof {
            "px" => Some(LengthUnit::Cjc),
            "em" => Some(LengthUnit::Cbp),
            "rem" => Some(LengthUnit::Rem),
            "%" => Some(LengthUnit::Qk),
            "vw" => Some(LengthUnit::Cpx),
            "vh" => Some(LengthUnit::Cpj),
            "pt" => Some(LengthUnit::Cjb),
            "" => None,
            _ => None,
        };
        
        (num, ifv)
    }
    
    fn ege(&mut self) -> String {
        let ay = self.u;
        while let Some(r) = self.asp() {
            if r.etb() || r == '-' || r == '_' {
                self.u += 1;
            } else {
                break;
            }
        }
        self.input[ay..self.u].to_string()
    }
    
    fn lsv(&mut self) -> String {
        let cgw = self.asp().unwrap_or('"');
        self.u += 1;
        
        let ay = self.u;
        while let Some(r) = self.asp() {
            if r == cgw {
                break;
            }
            if r == '\\' {
                self.u += 2; 
            } else {
                self.u += 1;
            }
        }
        let e = self.input[ay..self.u].to_string();
        self.cwp(cgw);
        e
    }
    
    fn vdv(&mut self) -> String {
        match self.asp() {
            Some('"') | Some('\'') => self.lsv(),
            _ => self.ege(),
        }
    }
    
    fn ayr(&mut self) {
        while let Some(r) = self.asp() {
            if r.fme() {
                self.u += 1;
            } else {
                break;
            }
        }
    }
    
    fn wpj(&mut self) {
        loop {
            self.ayr();
            
            if self.cj("/*") {
                
                self.u += 2;
                while self.u < self.input.len() {
                    if self.cj("*/") {
                        self.u += 2;
                        break;
                    }
                    self.u += 1;
                }
            } else {
                break;
            }
        }
    }
    
    fn wpf(&mut self) {
        
        while let Some(r) = self.asp() {
            if r == ';' {
                self.u += 1;
                break;
            }
            if r == '{' {
                
                let mut eo = 1;
                self.u += 1;
                while self.u < self.input.len() && eo > 0 {
                    match self.input.bw().goc(self.u) {
                        Some('{') => eo += 1,
                        Some('}') => eo -= 1,
                        _ => {}
                    }
                    self.u += 1;
                }
                break;
            }
            self.u += 1;
        }
    }
    
    fn asp(&self) -> Option<char> {
        self.input.bw().goc(self.u)
    }
    
    fn cj(&self, e: &str) -> bool {
        self.input[self.u..].cj(e)
    }
    
    fn cwp(&mut self, qy: char) -> bool {
        if self.asp() == Some(qy) {
            self.u += 1;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn xea() {
        assert_eq!(clt("#fff"), Some(0xFFFFFFFF));
        assert_eq!(clt("#000"), Some(0xFF000000));
        assert_eq!(clt("#ff0000"), Some(0xFFFF0000));
        assert_eq!(clt("red"), Some(0xFFFF0000));
        assert_eq!(clt("transparent"), Some(0x00000000));
    }
}
