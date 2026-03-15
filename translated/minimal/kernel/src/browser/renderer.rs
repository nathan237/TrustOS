



use alloc::string::{String, Gd};
use alloc::vec::Vec;
use alloc::vec;

use super::{Su, HtmlNode, HtmlElement, Wz};
use super::css_parser::{self, CssValue, FontWeight, FontStyle as Ctp, Sa, Mj, Yt, SelectorPart};
use crate::framebuffer;


const MF_: u32 = 0xFFFFFFFF;           
const CI_: u32 = 0xFF1A1A1A;         
const AOI_: u32 = 0xFF0066CC;         
const DFK_: u32 = 0xFF551A8B; 
const DFI_: u32 = 0xFF000000;      
const BMZ_: u32 = 0xFFF5F5F5;      
const DFG_: u32 = 0xFFD63384;         
const AOF_: u32 = 0xFFCCCCCC;           
const BNL_: u32 = 0xFF0066CC; 
const DFQ_: u32 = 0xFFF0F7FF;     


pub struct RenderContext {
    pub b: i32,
    pub c: i32,
    pub dtb: u32,
    pub acg: i32,
    pub asv: FontSize,
    pub bpt: bool,
    pub gkl: bool,
    pub esk: Option<String>,
    pub czh: Vec<Wz>,
    pub eeq: i32,
    pub gjs: bool,
    
    pub agx: u32,
    
    pub vp: Option<u32>,
    
    pub adh: f32,
    
    pub dde: bool,
    pub dmb: bool,
    
    pub jdp: Vec<i32>,
    
    pub gjr: bool,
}

#[derive(Clone, Copy, PartialEq)]
pub enum FontSize {
    Ew,
    M,
    Ht,
    Aiv,
    Atj,
    Atk,
}

impl FontSize {
    fn ac(&self) -> i32 {
        match self {
            FontSize::Ew => 12,
            FontSize::M => 16,
            FontSize::Ht => 18,
            FontSize::Aiv => 32,
            FontSize::Atj => 26,
            FontSize::Atk => 20,
        }
    }
}

impl RenderContext {
    pub fn new(z: u32) -> Self {
        Self {
            b: 16,
            c: 16,
            dtb: z - 32,
            acg: 20,
            asv: FontSize::M,
            bpt: false,
            gkl: false,
            esk: None,
            czh: Vec::new(),
            eeq: 0,
            gjs: false,
            agx: CI_,
            vp: None,
            adh: 1.0,
            dde: false,
            dmb: false,
            jdp: Vec::new(),
            gjr: false,
        }
    }
    
    fn agr(&mut self) {
        self.b = 16 + (self.eeq * 24);
        self.c += self.acg;
    }
    
    fn atm(&mut self) {
        self.b += 6;
    }
}


pub fn vvy(
    doc: &Su,
    b: i32,
    c: i32,
    z: u32,
    ac: u32,
    ug: i32,
) -> Vec<Wz> {
    
    framebuffer::ah(b as u32, c as u32, z, ac, MF_);
    
    let mut be = RenderContext::new(z);
    be.c = c - ug;
    
    
    let nhp = nsn(&doc.xq);
    let eze = if !nhp.is_empty() {
        css_parser::lsx(&nhp)
    } else {
        Mj { bib: Vec::new() }
    };
    
    
    for anq in &doc.xq {
        pby(&mut be, anq, &eze, b, c, z, ac);
    }
    
    be.czh
}


fn pby(
    be: &mut RenderContext,
    anq: &HtmlNode,
    eze: &Mj,
    bgi: i32,
    yk: i32,
    axq: u32,
    aom: u32,
) {
    match anq {
        HtmlNode::Text(text) => {
            vwr(be, text, bgi, yk, axq, aom);
        }
        HtmlNode::Na(ij) => {
            vvq(be, ij, eze, bgi, yk, axq, aom);
        }
    }
}


fn vwr(
    be: &mut RenderContext,
    text: &str,
    bgi: i32,
    yk: i32,
    axq: u32,
    aom: u32,
) {
    let s = if be.esk.is_some() { AOI_ } else { be.agx };
    
    
    if be.gjs {
        for line in text.ak() {
            vwa(be, line, s, bgi, yk, axq, aom);
            be.agr();
        }
        return;
    }
    
    
    let aoh: Vec<&str> = text.ayt().collect();
    
    for od in aoh {
        let xve = od.len() as i32 * 8; 
        
        
        if be.b + xve > (bgi + axq as i32 - 16) && be.b > 16 + (be.eeq * 24) {
            be.agr();
        }
        
        vxc(be, od, s, bgi, yk, axq, aom);
        be.atm();
    }
}

fn vxc(
    be: &mut RenderContext,
    od: &str,
    s: u32,
    bgi: i32,
    yk: i32,
    axq: u32,
    aom: u32,
) {
    
    if be.c + be.acg < yk || be.c > yk + aom as i32 {
        be.b += od.len() as i32 * 8;
        return;
    }
    
    
    let eud = be.b;
    
    
    for r in od.bw() {
        if be.b >= bgi && be.b < bgi + axq as i32 - 8 {
            ahi(be.b as u32, be.c as u32, r, s);
        }
        be.b += 8;
    }
    
    
    if let Some(cae) = &be.esk {
        be.czh.push(Wz {
            cae: cae.clone(),
            b: eud,
            c: be.c,
            z: (be.b - eud) as u32,
            ac: be.acg as u32,
        });
        
        
        if be.c >= yk && be.c < yk + aom as i32 {
            framebuffer::ah(
                eud as u32,
                (be.c + be.acg - 2) as u32,
                (be.b - eud) as u32,
                1,
                AOI_,
            );
        }
    }
    
    
    if be.dde && be.esk.is_none() {
        let d = (be.b - eud) as u32;
        if d > 0 && be.c >= yk && be.c < yk + aom as i32 {
            framebuffer::ah(
                eud as u32,
                (be.c + be.acg - 2) as u32,
                d, 1, s,
            );
        }
    }
    
    
    if be.dmb {
        let d = (be.b - eud) as u32;
        if d > 0 && be.c >= yk && be.c < yk + aom as i32 {
            framebuffer::ah(
                eud as u32,
                (be.c + be.acg / 2) as u32,
                d, 1, be.agx,
            );
        }
    }
}

fn vwa(
    be: &mut RenderContext,
    line: &str,
    s: u32,
    bgi: i32,
    yk: i32,
    axq: u32,
    aom: u32,
) {
    if be.c + be.acg < yk || be.c > yk + aom as i32 {
        return;
    }
    
    for r in line.bw() {
        if be.b >= bgi && be.b < bgi + axq as i32 - 8 {
            ahi(be.b as u32, be.c as u32, r, s);
        }
        be.b += 8;
    }
}


fn vvq(
    be: &mut RenderContext,
    ij: &HtmlElement,
    eze: &Mj,
    bgi: i32,
    yk: i32,
    axq: u32,
    aom: u32,
) {
    let ll = ij.ll.as_str();
    
    
    if oh!(ll, "head" | "script" | "style" | "meta" | "link" | "title") {
        return;
    }
    
    
    let jnn = be.asv;
    let pfp = be.bpt;
    let pfq = be.esk.clone();
    let pfu = be.gjs;
    let pfx = be.agx;
    let pfo = be.vp;
    let pfs = be.adh;
    let pfy = be.dde;
    let pfw = be.dmb;
    let pft = be.gjr;
    
    
    if let Some(mhz) = ij.qn("style") {
        qjx(be, mhz);
    }
    
    
    qkb(be, ij, eze);
    
    
    if let Some(kjx) = ij.qn("color") {
        if let Some(r) = lsl(kjx) {
            be.agx = r;
        }
    }
    if let Some(qpm) = ij.qn("bgcolor") {
        if let Some(r) = lsl(qpm) {
            be.vp = Some(r);
        }
    }
    
    
    if be.c >= i32::O / 4 {
        be.asv = jnn;
        be.acg = jnn.ac() + 4;
        be.bpt = pfp;
        be.esk = pfq;
        be.gjs = pfu;
        be.agx = pfx;
        be.vp = pfo;
        be.adh = pfs;
        be.dde = pfy;
        be.dmb = pfw;
        be.gjr = pft;
        return;
    }
    
    
    match ll {
        
        "html" | "body" | "div" | "section" | "article" | "nav" | "header" | "footer" | "main" | 
        "noscript" | "span" | "form" | "label" | "fieldset" | "legend" | "details" | "summary" |
        "figure" | "figcaption" | "aside" | "dialog" | "abbr" | "address" | "cite" | "dfn" |
        "ruby" | "rt" | "rp" | "data" | "time" | "var" | "samp" | "kbd" | "wbr" | "bdi" | "bdo" => {
            
        }
        
        "p" => {
            be.agr();
            be.c += 8; 
        }
        
        "br" => {
            be.agr();
        }
        
        "hr" => {
            be.agr();
            be.c += 8;
            if be.c >= yk && be.c < yk + aom as i32 {
                framebuffer::ah(
                    (bgi + 16) as u32,
                    be.c as u32,
                    axq - 32,
                    1,
                    AOF_,
                );
            }
            be.c += 16;
            be.b = 16;
        }
        
        "h1" => {
            be.agr();
            be.c += 16;
            be.asv = FontSize::Aiv;
            be.acg = 40;
            be.bpt = true;
        }
        
        "h2" => {
            be.agr();
            be.c += 12;
            be.asv = FontSize::Atj;
            be.acg = 32;
            be.bpt = true;
        }
        
        "h3" | "h4" | "h5" | "h6" => {
            be.agr();
            be.c += 8;
            be.asv = FontSize::Atk;
            be.acg = 26;
            be.bpt = true;
        }
        
        "a" => {
            if let Some(cae) = ij.qn("href") {
                be.esk = Some(cae.to_string());
            }
        }
        
        "strong" | "b" => {
            be.bpt = true;
        }
        
        "em" | "i" => {
            be.gkl = true;
        }
        
        "code" => {
            
        }
        
        "pre" => {
            be.agr();
            be.c += 8;
            be.gjs = true;
            
            if be.c >= yk {
                framebuffer::ah(
                    (bgi + 8) as u32,
                    be.c as u32,
                    axq - 16,
                    100, 
                    BMZ_,
                );
            }
        }
        
        "blockquote" => {
            be.agr();
            be.c += 8;
            be.eeq += 1;
            
            if be.c >= yk {
                framebuffer::ah(
                    (bgi + 12) as u32,
                    be.c as u32,
                    4,
                    80,
                    BNL_,
                );
            }
        }
        
        "ul" | "ol" => {
            be.agr();
            be.eeq += 1;
            if ll == "ol" {
                be.gjr = true;
                be.jdp.push(0);
            }
        }
        
        "li" => {
            be.agr();
            
            if be.c >= yk && be.c < yk + aom as i32 {
                if be.gjr {
                    
                    if let Some(va) = be.jdp.dsq() {
                        *va += 1;
                        let ajh = alloc::format!("{}.", va);
                        let uwq = be.b - 20;
                        for (a, r) in ajh.bw().cf() {
                            ahi((uwq + a as i32 * 8) as u32, be.c as u32, r, CI_);
                        }
                    }
                } else {
                    
                    let qup = be.b - 12;
                    framebuffer::ah(
                        qup as u32,
                        (be.c + 6) as u32,
                        4,
                        4,
                        CI_,
                    );
                }
            }
        }
        
        "img" => {
            
            let bdj = ij.qn("alt").unwrap_or("");
            let cy = ij.qn("src").unwrap_or("");
            
            
            let gjp = ij.qn("width")
                .and_then(|d| d.bdd("px").parse::<u32>().bq())
                .unwrap_or(120)
                .v(axq.ao(32))
                .am(40);
            let ede = ij.qn("height")
                .and_then(|i| i.bdd("px").parse::<u32>().bq())
                .unwrap_or(60)
                .v(300)
                .am(20);
            
            if be.c >= yk && be.c < yk + aom as i32 {
                
                framebuffer::ah(be.b as u32, be.c as u32, gjp, ede, 0xFFF0F0F0);
                
                framebuffer::ah(be.b as u32, be.c as u32, gjp, 1, 0xFFDDDDDD);
                framebuffer::ah(be.b as u32, (be.c + ede as i32 - 1) as u32, gjp, 1, 0xFFDDDDDD);
                framebuffer::ah(be.b as u32, be.c as u32, 1, ede, 0xFFDDDDDD);
                framebuffer::ah((be.b + gjp as i32 - 1) as u32, be.c as u32, 1, ede, 0xFFDDDDDD);
                
                
                let trg = "[IMG]";
                let bel = be.b + (gjp as i32 / 2) - 20;
                let bem = be.c + (ede as i32 / 2) - 12;
                if ede > 24 {
                    for (a, r) in trg.bw().cf() {
                        ahi((bel + a as i32 * 8) as u32, bem as u32, r, 0xFF999999);
                    }
                }
                
                
                let dgj = if !bdj.is_empty() {
                    bdj
                } else if !cy.is_empty() {
                    cy.cmm('/').next().unwrap_or(cy)
                } else {
                    ""
                };
                if !dgj.is_empty() && ede > 40 {
                    let aem = (gjp / 8).ao(2) as usize;
                    let wg = be.b + 8;
                    let sl = be.c + (ede as i32 / 2) + 4;
                    for (a, r) in dgj.bw().take(aem).cf() {
                        ahi((wg + a as i32 * 8) as u32, sl as u32, r, 0xFF666666);
                    }
                }
            }
            be.c += ede as i32 + 4;
        }
        
        "table" => {
            be.agr();
            be.c += 8;
        }
        
        "tr" => {
            be.agr();
        }
        
        "td" | "th" => {
            be.b += 16;
        }
        
        "input" => {
            let tvc = ij.qn("type").unwrap_or("text");
            match tvc {
                "hidden" => {
                    return;
                }
                "submit" | "button" => {
                    
                    let bn = ij.qn("value").unwrap_or("Submit");
                    let hbn = (bn.len() as u32 * 8) + 24;
                    let dzf = 28u32;
                    if be.c >= yk && be.c < yk + aom as i32 {
                        framebuffer::ah(be.b as u32, be.c as u32, hbn, dzf, 0xFFE8E8E8);
                        
                        framebuffer::ah(be.b as u32, be.c as u32, hbn, 1, 0xFFBBBBBB);
                        framebuffer::ah(be.b as u32, (be.c + dzf as i32 - 1) as u32, hbn, 1, 0xFFBBBBBB);
                        framebuffer::ah(be.b as u32, be.c as u32, 1, dzf, 0xFFBBBBBB);
                        framebuffer::ah((be.b + hbn as i32 - 1) as u32, be.c as u32, 1, dzf, 0xFFBBBBBB);
                        for (a, r) in bn.bw().cf() {
                            ahi((be.b + 12 + a as i32 * 8) as u32, (be.c + 6) as u32, r, CI_);
                        }
                    }
                    be.b += hbn as i32 + 8;
                }
                "checkbox" => {
                    if be.c >= yk && be.c < yk + aom as i32 {
                        framebuffer::ah(be.b as u32, (be.c + 2) as u32, 14, 14, 0xFFFFFFFF);
                        framebuffer::ah(be.b as u32, (be.c + 2) as u32, 14, 1, 0xFF999999);
                        framebuffer::ah(be.b as u32, (be.c + 15) as u32, 14, 1, 0xFF999999);
                        framebuffer::ah(be.b as u32, (be.c + 2) as u32, 1, 14, 0xFF999999);
                        framebuffer::ah((be.b + 13) as u32, (be.c + 2) as u32, 1, 14, 0xFF999999);
                        if ij.qn("checked").is_some() {
                            framebuffer::ah((be.b + 3) as u32, (be.c + 8) as u32, 8, 2, CI_);
                            framebuffer::ah((be.b + 3) as u32, (be.c + 5) as u32, 2, 5, CI_);
                        }
                    }
                    be.b += 20;
                }
                "radio" => {
                    if be.c >= yk && be.c < yk + aom as i32 {
                        framebuffer::ah(be.b as u32, (be.c + 2) as u32, 14, 14, 0xFFFFFFFF);
                        framebuffer::ah(be.b as u32, (be.c + 2) as u32, 14, 1, 0xFF999999);
                        framebuffer::ah(be.b as u32, (be.c + 15) as u32, 14, 1, 0xFF999999);
                        framebuffer::ah(be.b as u32, (be.c + 2) as u32, 1, 14, 0xFF999999);
                        framebuffer::ah((be.b + 13) as u32, (be.c + 2) as u32, 1, 14, 0xFF999999);
                        if ij.qn("checked").is_some() {
                            framebuffer::ah((be.b + 4) as u32, (be.c + 6) as u32, 6, 6, CI_);
                        }
                    }
                    be.b += 20;
                }
                _ => {
                    
                    let fqy = ij.qn("placeholder").unwrap_or("");
                    let bn = ij.qn("value").unwrap_or("");
                    let dgj = if bn.is_empty() { fqy } else { bn };
                    let est = 200u32.v(axq.ao(40));
                    let jah = 26u32;
                    if be.c >= yk && be.c < yk + aom as i32 {
                        framebuffer::ah(be.b as u32, be.c as u32, est, jah, 0xFFFFFFFF);
                        framebuffer::ah(be.b as u32, be.c as u32, est, 1, 0xFF999999);
                        framebuffer::ah(be.b as u32, (be.c + jah as i32 - 1) as u32, est, 1, 0xFF999999);
                        framebuffer::ah(be.b as u32, be.c as u32, 1, jah, 0xFF999999);
                        framebuffer::ah((be.b + est as i32 - 1) as u32, be.c as u32, 1, jah, 0xFF999999);
                        let agx = if bn.is_empty() { 0xFF999999 } else { CI_ };
                        for (a, r) in dgj.bw().take((est / 8 - 2) as usize).cf() {
                            ahi((be.b + 4 + a as i32 * 8) as u32, (be.c + 5) as u32, r, agx);
                        }
                    }
                    be.b += est as i32 + 8;
                }
            }
        }
        
        "button" => {
            
            let dzf = 28u32;
            if be.c >= yk && be.c < yk + aom as i32 {
                framebuffer::ah(be.b as u32, be.c as u32, 120, dzf, 0xFFE8E8E8);
                framebuffer::ah(be.b as u32, be.c as u32, 120, 1, 0xFFBBBBBB);
                framebuffer::ah(be.b as u32, (be.c + dzf as i32 - 1) as u32, 120, 1, 0xFFBBBBBB);
                framebuffer::ah(be.b as u32, be.c as u32, 1, dzf, 0xFFBBBBBB);
                framebuffer::ah(119 + be.b as u32, be.c as u32, 1, dzf, 0xFFBBBBBB);
            }
            be.c += 6;
        }
        
        "textarea" => {
            be.agr();
            let jsj = core::cmp::v(axq.ao(32), 400);
            let icu = 80u32;
            if be.c >= yk && be.c < yk + aom as i32 {
                framebuffer::ah(be.b as u32, be.c as u32, jsj, icu, 0xFFFFFFFF);
                framebuffer::ah(be.b as u32, be.c as u32, jsj, 1, 0xFF999999);
                framebuffer::ah(be.b as u32, (be.c + icu as i32 - 1) as u32, jsj, 1, 0xFF999999);
                framebuffer::ah(be.b as u32, be.c as u32, 1, icu, 0xFF999999);
                framebuffer::ah((be.b + jsj as i32 - 1) as u32, be.c as u32, 1, icu, 0xFF999999);
            }
            be.c += icu as i32 + 8;
        }
        
        "select" => {
            let fub = 160u32.v(axq.ao(32));
            let jog = 26u32;
            if be.c >= yk && be.c < yk + aom as i32 {
                framebuffer::ah(be.b as u32, be.c as u32, fub, jog, 0xFFFFFFFF);
                framebuffer::ah(be.b as u32, be.c as u32, fub, 1, 0xFF999999);
                framebuffer::ah(be.b as u32, (be.c + jog as i32 - 1) as u32, fub, 1, 0xFF999999);
                framebuffer::ah(be.b as u32, be.c as u32, 1, jog, 0xFF999999);
                framebuffer::ah((be.b + fub as i32 - 1) as u32, be.c as u32, 1, jog, 0xFF999999);
                
                framebuffer::ah((be.b + fub as i32 - 16) as u32, (be.c + 10) as u32, 8, 2, 0xFF666666);
                framebuffer::ah((be.b + fub as i32 - 14) as u32, (be.c + 12) as u32, 4, 2, 0xFF666666);
            }
            be.b += fub as i32 + 8;
        }
        
        "option" | "optgroup" => {
            
            return;
        }
        
        "small" | "sub" | "sup" => {
            be.asv = FontSize::Ew;
            be.acg = FontSize::Ew.ac() + 4;
        }
        
        "mark" => {
            be.vp = Some(0xFFFFFF00); 
        }
        
        "del" | "s" | "strike" => {
            be.agx = 0xFF999999;
            be.dmb = true;
        }
        
        "u" | "ins" => {
            be.dde = true;
        }
        
        "progress" => {
            
            let aki: f32 = ij.qn("max").and_then(|p| p.parse().bq()).unwrap_or(1.0);
            let her: f32 = ij.qn("value").and_then(|p| p.parse().bq()).unwrap_or(0.0);
            let cgn = (her / aki).v(1.0).am(0.0);
            let lo = 200u32.v(axq.ao(40));
            let tn = 18u32;
            if be.c >= yk && be.c < yk + aom as i32 {
                
                framebuffer::ah(be.b as u32, be.c as u32, lo, tn, 0xFFE0E0E0);
                
                let akd = (lo as f32 * cgn) as u32;
                if akd > 0 {
                    framebuffer::ah(be.b as u32, be.c as u32, akd, tn, 0xFF4CAF50);
                }
                
                framebuffer::ah(be.b as u32, be.c as u32, lo, 1, 0xFFBBBBBB);
                framebuffer::ah(be.b as u32, (be.c + tn as i32 - 1) as u32, lo, 1, 0xFFBBBBBB);
                framebuffer::ah(be.b as u32, be.c as u32, 1, tn, 0xFFBBBBBB);
                framebuffer::ah((be.b + lo as i32 - 1) as u32, be.c as u32, 1, tn, 0xFFBBBBBB);
                
                let jiy = alloc::format!("{}%", (cgn * 100.0) as u32);
                let wg = be.b + (lo as i32 / 2) - (jiy.len() as i32 * 4);
                for (a, r) in jiy.bw().cf() {
                    ahi((wg + a as i32 * 8) as u32, (be.c + 1) as u32, r, CI_);
                }
            }
            be.b += lo as i32 + 8;
            return; 
        }
        
        "meter" => {
            
            let hro: f32 = ij.qn("min").and_then(|p| p.parse().bq()).unwrap_or(0.0);
            let aki: f32 = ij.qn("max").and_then(|p| p.parse().bq()).unwrap_or(1.0);
            let her: f32 = ij.qn("value").and_then(|p| p.parse().bq()).unwrap_or(0.0);
            let ail: f32 = ij.qn("low").and_then(|p| p.parse().bq()).unwrap_or(hro);
            let afq: f32 = ij.qn("high").and_then(|p| p.parse().bq()).unwrap_or(aki);
            let cmb = aki - hro;
            let cgn = if cmb > 0.0 { ((her - hro) / cmb).v(1.0).am(0.0) } else { 0.0 };
            let lo = 160u32.v(axq.ao(40));
            let tn = 16u32;
            
            let ebo = if her < ail {
                0xFFFF5722 
            } else if her > afq {
                0xFFFF5722 
            } else {
                0xFF4CAF50 
            };
            if be.c >= yk && be.c < yk + aom as i32 {
                framebuffer::ah(be.b as u32, be.c as u32, lo, tn, 0xFFE0E0E0);
                let akd = (lo as f32 * cgn) as u32;
                if akd > 0 {
                    framebuffer::ah(be.b as u32, be.c as u32, akd, tn, ebo);
                }
                framebuffer::ah(be.b as u32, be.c as u32, lo, 1, 0xFFBBBBBB);
                framebuffer::ah(be.b as u32, (be.c + tn as i32 - 1) as u32, lo, 1, 0xFFBBBBBB);
                framebuffer::ah(be.b as u32, be.c as u32, 1, tn, 0xFFBBBBBB);
                framebuffer::ah((be.b + lo as i32 - 1) as u32, be.c as u32, 1, tn, 0xFFBBBBBB);
            }
            be.b += lo as i32 + 8;
            return;
        }
        
        "dl" => {
            be.agr();
        }
        "dt" => {
            be.agr();
            be.bpt = true;
        }
        "dd" => {
            be.agr();
            be.b += 40; 
        }
        
        "details" => {
            be.agr();
            be.c += 4;
            
            if be.c >= yk && be.c < yk + aom as i32 {
                let guz = be.b;
                let aji = ij.qn("open").is_some();
                if aji {
                    
                    framebuffer::ah(guz as u32, be.c as u32, 8, 2, CI_);
                    framebuffer::ah((guz + 1) as u32, (be.c + 2) as u32, 6, 2, CI_);
                    framebuffer::ah((guz + 2) as u32, (be.c + 4) as u32, 4, 2, CI_);
                } else {
                    
                    framebuffer::ah(guz as u32, be.c as u32, 2, 8, CI_);
                    framebuffer::ah((guz + 2) as u32, (be.c + 1) as u32, 2, 6, CI_);
                    framebuffer::ah((guz + 4) as u32, (be.c + 2) as u32, 2, 4, CI_);
                }
            }
            be.b += 16; 
        }
        
        "summary" => {
            be.bpt = true;
        }
        
        "figure" => {
            be.agr();
            be.c += 8;
            be.eeq += 1;
        }
        
        "figcaption" => {
            be.agr();
            be.asv = FontSize::Ew;
            be.acg = FontSize::Ew.ac() + 4;
            be.agx = 0xFF666666;
        }
        
        "nav" => {
            be.vp = Some(0xFFF8F8F8);
        }
        
        "footer" => {
            be.agr();
            be.c += 16;
            if be.c >= yk && be.c < yk + aom as i32 {
                framebuffer::ah(
                    (bgi + 16) as u32, be.c as u32, axq - 32, 1, AOF_,
                );
            }
            be.c += 8;
            be.asv = FontSize::Ew;
            be.acg = FontSize::Ew.ac() + 4;
            be.agx = 0xFF666666;
        }
        
        "header" => {
            be.vp = Some(0xFFF0F0F0);
        }
        
        "main" | "article" | "section" | "aside" => {
            
        }
        
        "video" | "audio" | "canvas" | "svg" | "iframe" | "embed" | "object" => {
            
            let gpj = ij.qn("width")
                .and_then(|d| d.bdd("px").parse::<u32>().bq())
                .unwrap_or(320)
                .v(axq.ao(32));
            let fqz = ij.qn("height")
                .and_then(|i| i.bdd("px").parse::<u32>().bq())
                .unwrap_or(180)
                .v(400);
            if be.c >= yk && be.c < yk + aom as i32 {
                framebuffer::ah(be.b as u32, be.c as u32, gpj, fqz, 0xFF2C2C2C);
                
                let cu = match ll {
                    "video" => "[VIDEO]",
                    "audio" => "[AUDIO]",
                    "canvas" => "[CANVAS]",
                    "svg" => "[SVG]",
                    "iframe" => "[IFRAME]",
                    _ => "[EMBED]",
                };
                let mj = be.b + (gpj as i32 / 2) - (cu.len() as i32 * 4);
                let ct = be.c + (fqz as i32 / 2) - 8;
                for (a, r) in cu.bw().cf() {
                    ahi((mj + a as i32 * 8) as u32, ct as u32, r, 0xFFAAAAAA);
                }
                
                if ll == "video" || ll == "audio" {
                    let cx = be.b + gpj as i32 / 2;
                    let ae = be.c + fqz as i32 / 2 + 12;
                    for br in 0..16 {
                        let d = 16 - br;
                        framebuffer::ah((cx - 4) as u32, (ae + br) as u32, d as u32, 1, 0xFFFFFFFF);
                    }
                }
                
                framebuffer::ah(be.b as u32, be.c as u32, gpj, 1, 0xFF555555);
                framebuffer::ah(be.b as u32, (be.c + fqz as i32 - 1) as u32, gpj, 1, 0xFF555555);
                framebuffer::ah(be.b as u32, be.c as u32, 1, fqz, 0xFF555555);
                framebuffer::ah((be.b + gpj as i32 - 1) as u32, be.c as u32, 1, fqz, 0xFF555555);
            }
            be.c += fqz as i32 + 4;
            return; 
        }
        
        "center" => {
            be.agr();
            be.b = bgi + (axq as i32 / 4);
        }
        
        _ => {}
    }
    
    
    for aeh in &ij.zf {
        pby(be, aeh, eze, bgi, yk, axq, aom);
    }
    
    
    match ll {
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            be.agr();
            be.c += 8;
        }
        "p" => {
            be.agr();
            be.c += 8;
        }
        "ul" | "ol" => {
            be.eeq -= 1;
            be.jdp.pop();
            be.agr();
        }
        "blockquote" => {
            be.eeq -= 1;
            be.agr();
        }
        "pre" => {
            be.agr();
        }
        "table" => {
            be.agr();
        }
        "dl" => {
            be.agr();
            be.c += 4;
        }
        "dt" | "dd" => {
            be.agr();
        }
        "details" => {
            be.agr();
            be.c += 4;
        }
        "figure" => {
            be.eeq -= 1;
            be.agr();
            be.c += 8;
        }
        "footer" | "header" | "nav" => {
            be.agr();
        }
        _ => {}
    }
    
    be.asv = jnn;
    be.acg = jnn.ac() + 4;
    be.bpt = pfp;
    be.esk = pfq;
    be.gjs = pfu;
    be.agx = pfx;
    be.vp = pfo;
    be.adh = pfs;
    be.dde = pfy;
    be.dmb = pfw;
    be.gjr = pft;
}


fn nsn(xq: &[HtmlNode]) -> String {
    let mut eoe = String::new();
    for anq in xq {
        if let HtmlNode::Na(ij) = anq {
            if ij.ll == "style" {
                
                for aeh in &ij.zf {
                    if let HtmlNode::Text(text) = aeh {
                        eoe.t(text);
                        eoe.push('\n');
                    }
                }
            } else {
                
                let ncu = nsn(&ij.zf);
                if !ncu.is_empty() {
                    eoe.t(&ncu);
                }
            }
        }
    }
    eoe
}


fn ukk(bof: &Yt, ebd: &HtmlElement) -> bool {
    let ek = &bof.bgw;
    if ek.is_empty() {
        return false;
    }
    
    
    
    let mut mdg = 0;
    let mut lhy: Option<usize> = None;
    let mut oak = false;
    
    for (a, vu) in ek.iter().cf() {
        match vu {
            SelectorPart::Cau | SelectorPart::Bdm | SelectorPart::Bbm | SelectorPart::Bst => {
                oak = true;
                mdg = a + 1;
                lhy = None;
            }
            SelectorPart::Azr(_) => {
                if !oak {
                    if let Some(ybx) = lhy {
                        
                        mdg = a;
                    }
                }
                lhy = Some(a);
            }
            _ => {}
        }
    }
    
    let ie = &ek[mdg..];
    
    for vu in ie {
        match vu {
            SelectorPart::Azr(ll) => {
                if ebd.ll != *ll {
                    return false;
                }
            }
            SelectorPart::Bdo(class) => {
                let sjw = ebd.qn("class").unwrap_or("");
                if !sjw.ayt().any(|r| r == class.as_str()) {
                    return false;
                }
            }
            SelectorPart::Bjj(ad) => {
                let sjx = ebd.qn("id").unwrap_or("");
                if sjx != ad.as_str() {
                    return false;
                }
            }
            SelectorPart::Bvi => {}
            SelectorPart::Ms(qn, ap) => {
                match ebd.qn(qn) {
                    Some(sjy) => {
                        if let Some(qy) = ap {
                            if sjy != qy.as_str() {
                                return false;
                            }
                        }
                    }
                    None => return false,
                }
            }
            SelectorPart::Bpn(_) => {} 
            _ => {} 
        }
    }
    
    !ie.is_empty()
}


fn qkb(be: &mut RenderContext, ij: &HtmlElement, eze: &Mj) {
    for agu in &eze.bib {
        let ukj = agu.fud.iter().any(|fua| ukk(fua, ij));
        if ukj {
            mwa(be, &agu.dps);
        }
    }
}


fn mwa(be: &mut RenderContext, dps: &[Sa]) {
    for aqy in dps {
        match aqy.jki.as_str() {
            "color" => {
                if let Some(r) = nhq(&aqy.bn) {
                    be.agx = r;
                }
            }
            "background-color" | "background" => {
                if let Some(r) = nhq(&aqy.bn) {
                    be.vp = Some(r);
                }
            }
            "font-size" => {
                match &aqy.bn {
                    CssValue::Acp(y, _) => {
                        if *y <= 14.0 {
                            be.asv = FontSize::Ew;
                        } else if *y <= 18.0 {
                            be.asv = FontSize::M;
                        } else if *y <= 22.0 {
                            be.asv = FontSize::Ht;
                        } else if *y <= 28.0 {
                            be.asv = FontSize::Atk;
                        } else if *y <= 30.0 {
                            be.asv = FontSize::Atj;
                        } else {
                            be.asv = FontSize::Aiv;
                        }
                        be.acg = be.asv.ac() + 4;
                    }
                    CssValue::Bx(yo) => {
                        match yo.as_str() {
                            "small" | "x-small" | "xx-small" => be.asv = FontSize::Ew,
                            "medium" => be.asv = FontSize::M,
                            "large" => be.asv = FontSize::Ht,
                            "x-large" | "xx-large" => be.asv = FontSize::Aiv,
                            _ => {}
                        }
                        be.acg = be.asv.ac() + 4;
                    }
                    _ => {}
                }
            }
            "font-weight" => {
                match &aqy.bn {
                    CssValue::Bx(yo) if yo == "bold" => be.bpt = true,
                    CssValue::Bx(yo) if yo == "normal" => be.bpt = false,
                    CssValue::L(bo) if *bo >= 700.0 => be.bpt = true,
                    _ => {}
                }
            }
            "font-style" => {
                if let CssValue::Bx(yo) = &aqy.bn {
                    be.gkl = yo == "italic" || yo == "oblique";
                }
            }
            "display" => {
                if let CssValue::Bx(yo) = &aqy.bn {
                    if yo == "none" {
                        be.c = i32::O / 2;
                    }
                }
            }
            "text-align" => {
                if let CssValue::Bx(yo) = &aqy.bn {
                    match yo.as_str() {
                        "center" => { be.b = be.dtb as i32 / 4; }
                        _ => {}
                    }
                }
            }
            "margin-top" | "padding-top" => {
                if let CssValue::Acp(y, _) = &aqy.bn {
                    be.c += *y as i32;
                }
            }
            "margin-bottom" | "padding-bottom" => {
                
            }
            "visibility" => {
                if let CssValue::Bx(yo) = &aqy.bn {
                    if yo == "hidden" || yo == "collapse" {
                        be.c = i32::O / 2;
                    }
                }
            }
            "opacity" => {
                match &aqy.bn {
                    CssValue::L(bo) => {
                        be.adh = bo.am(0.0).v(1.0) as f32;
                    }
                    _ => {}
                }
            }
            "text-decoration" | "text-decoration-line" => {
                if let CssValue::Bx(yo) = &aqy.bn {
                    match yo.as_str() {
                        "underline" => be.dde = true,
                        "line-through" => be.dmb = true,
                        "none" => {
                            be.dde = false;
                            be.dmb = false;
                        }
                        _ => {}
                    }
                }
            }
            "text-transform" => {
                
                if let CssValue::Bx(yo) = &aqy.bn {
                    match yo.as_str() {
                        "uppercase" | "lowercase" | "capitalize" | "none" => {
                            
                        }
                        _ => {}
                    }
                }
            }
            "line-height" => {
                match &aqy.bn {
                    CssValue::Acp(y, _) => {
                        be.acg = *y as i32;
                    }
                    CssValue::L(bo) => {
                        be.acg = (*bo as i32) * be.asv.ac();
                    }
                    _ => {}
                }
            }
            "margin-left" | "padding-left" => {
                if let CssValue::Acp(y, _) = &aqy.bn {
                    be.b += *y as i32;
                }
            }
            "border" | "border-top" | "border-bottom" | "border-left" | "border-right" => {
                
            }
            _ => {} 
        }
    }
}


fn qjx(be: &mut RenderContext, mhz: &str) {
    let dps = css_parser::vcr(mhz);
    mwa(be, &dps);
}


fn nhq(bn: &CssValue) -> Option<u32> {
    match bn {
        CssValue::Color(r) => Some(*r),
        CssValue::Bx(j) => lsl(j),
        _ => None,
    }
}


fn lsl(e: &str) -> Option<u32> {
    let e = e.em();
    
    
    if e.cj('#') {
        let nu = &e[1..];
        if nu.len() == 6 {
            let m = u8::wa(&nu[0..2], 16).bq()?;
            let at = u8::wa(&nu[2..4], 16).bq()?;
            let o = u8::wa(&nu[4..6], 16).bq()?;
            return Some(0xFF000000 | (m as u32) << 16 | (at as u32) << 8 | o as u32);
        }
        if nu.len() == 3 {
            let m = u8::wa(&nu[0..1], 16).bq()? * 17;
            let at = u8::wa(&nu[1..2], 16).bq()? * 17;
            let o = u8::wa(&nu[2..3], 16).bq()? * 17;
            return Some(0xFF000000 | (m as u32) << 16 | (at as u32) << 8 | o as u32);
        }
        return None;
    }
    
    
    match e.aqn().as_str() {
        "black" => Some(0xFF000000),
        "white" => Some(0xFFFFFFFF),
        "red" => Some(0xFFFF0000),
        "green" => Some(0xFF008000),
        "blue" => Some(0xFF0000FF),
        "yellow" => Some(0xFFFFFF00),
        "cyan" | "aqua" => Some(0xFF00FFFF),
        "magenta" | "fuchsia" => Some(0xFFFF00FF),
        "gray" | "grey" => Some(0xFF808080),
        "silver" => Some(0xFFC0C0C0),
        "maroon" => Some(0xFF800000),
        "olive" => Some(0xFF808000),
        "navy" => Some(0xFF000080),
        "teal" => Some(0xFF008080),
        "purple" => Some(0xFF800080),
        "orange" => Some(0xFFFFA500),
        "pink" => Some(0xFFFFC0CB),
        "brown" => Some(0xFFA52A2A),
        "coral" => Some(0xFFFF7F50),
        "crimson" => Some(0xFFDC143C),
        "darkblue" => Some(0xFF00008B),
        "darkgreen" => Some(0xFF006400),
        "darkred" => Some(0xFF8B0000),
        "gold" => Some(0xFFFFD700),
        "indigo" => Some(0xFF4B0082),
        "lime" => Some(0xFF00FF00),
        "tomato" => Some(0xFFFF6347),
        "transparent" => Some(0x00000000),
        _ => None,
    }
}


fn ahi(b: u32, c: u32, r: char, s: u32) {
    let font = tcy(r);
    
    for (br, hf) in font.iter().cf() {
        for ga in 0..8 {
            if (hf >> (7 - ga)) & 1 != 0 {
                framebuffer::sf(b + ga, c + br as u32, s);
            }
        }
    }
}


fn tcy(r: char) -> [u8; 16] {
    
    match r {
        ' ' => [0x00; 16],
        '!' => [0x00,0x18,0x18,0x18,0x18,0x18,0x18,0x18,0x00,0x18,0x18,0x00,0x00,0x00,0x00,0x00],
        '"' => [0x00,0x66,0x66,0x66,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '#' => [0x00,0x36,0x36,0x7F,0x36,0x36,0x7F,0x36,0x36,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '.' => [0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x18,0x18,0x00,0x00,0x00,0x00,0x00],
        ',' => [0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x18,0x18,0x08,0x10,0x00,0x00,0x00],
        ':' => [0x00,0x00,0x00,0x18,0x18,0x00,0x00,0x00,0x18,0x18,0x00,0x00,0x00,0x00,0x00,0x00],
        ';' => [0x00,0x00,0x00,0x18,0x18,0x00,0x00,0x00,0x18,0x18,0x08,0x10,0x00,0x00,0x00,0x00],
        '?' => [0x00,0x3C,0x66,0x06,0x0C,0x18,0x18,0x00,0x18,0x18,0x00,0x00,0x00,0x00,0x00,0x00],
        '-' => [0x00,0x00,0x00,0x00,0x00,0x7E,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '_' => [0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x7F,0x00,0x00,0x00,0x00],
        '/' => [0x00,0x02,0x04,0x08,0x10,0x20,0x40,0x80,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '(' => [0x00,0x0C,0x18,0x30,0x30,0x30,0x30,0x30,0x18,0x0C,0x00,0x00,0x00,0x00,0x00,0x00],
        ')' => [0x00,0x30,0x18,0x0C,0x0C,0x0C,0x0C,0x0C,0x18,0x30,0x00,0x00,0x00,0x00,0x00,0x00],
        '[' => [0x00,0x3C,0x30,0x30,0x30,0x30,0x30,0x30,0x30,0x3C,0x00,0x00,0x00,0x00,0x00,0x00],
        ']' => [0x00,0x3C,0x0C,0x0C,0x0C,0x0C,0x0C,0x0C,0x0C,0x3C,0x00,0x00,0x00,0x00,0x00,0x00],
        '<' => [0x00,0x00,0x06,0x0C,0x18,0x30,0x18,0x0C,0x06,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '>' => [0x00,0x00,0x60,0x30,0x18,0x0C,0x18,0x30,0x60,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '=' => [0x00,0x00,0x00,0x7E,0x00,0x00,0x7E,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '+' => [0x00,0x00,0x18,0x18,0x18,0x7E,0x18,0x18,0x18,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '0' => [0x00,0x3C,0x66,0x66,0x6E,0x76,0x66,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '1' => [0x00,0x18,0x38,0x18,0x18,0x18,0x18,0x18,0x7E,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '2' => [0x00,0x3C,0x66,0x06,0x0C,0x18,0x30,0x60,0x7E,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '3' => [0x00,0x3C,0x66,0x06,0x1C,0x06,0x06,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '4' => [0x00,0x0C,0x1C,0x3C,0x6C,0x7E,0x0C,0x0C,0x0C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '5' => [0x00,0x7E,0x60,0x7C,0x06,0x06,0x06,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '6' => [0x00,0x1C,0x30,0x60,0x7C,0x66,0x66,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '7' => [0x00,0x7E,0x06,0x0C,0x18,0x30,0x30,0x30,0x30,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '8' => [0x00,0x3C,0x66,0x66,0x3C,0x66,0x66,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '9' => [0x00,0x3C,0x66,0x66,0x66,0x3E,0x06,0x0C,0x38,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'A' | 'a' => [0x00,0x18,0x3C,0x66,0x66,0x7E,0x66,0x66,0x66,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'B' | 'b' => [0x00,0x7C,0x66,0x66,0x7C,0x66,0x66,0x66,0x7C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'C' | 'c' => [0x00,0x3C,0x66,0x60,0x60,0x60,0x60,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'D' | 'd' => [0x00,0x78,0x6C,0x66,0x66,0x66,0x66,0x6C,0x78,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'E' | 'e' => [0x00,0x7E,0x60,0x60,0x7C,0x60,0x60,0x60,0x7E,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'F' | 'f' => [0x00,0x7E,0x60,0x60,0x7C,0x60,0x60,0x60,0x60,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'G' | 'g' => [0x00,0x3C,0x66,0x60,0x60,0x6E,0x66,0x66,0x3E,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'H' | 'h' => [0x00,0x66,0x66,0x66,0x7E,0x66,0x66,0x66,0x66,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'I' | 'i' => [0x00,0x7E,0x18,0x18,0x18,0x18,0x18,0x18,0x7E,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'J' | 'j' => [0x00,0x3E,0x0C,0x0C,0x0C,0x0C,0x0C,0x6C,0x38,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'K' | 'k' => [0x00,0x66,0x6C,0x78,0x70,0x78,0x6C,0x66,0x66,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'L' | 'l' => [0x00,0x60,0x60,0x60,0x60,0x60,0x60,0x60,0x7E,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'M' | 'm' => [0x00,0x63,0x77,0x7F,0x6B,0x63,0x63,0x63,0x63,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'N' | 'n' => [0x00,0x66,0x76,0x7E,0x7E,0x6E,0x66,0x66,0x66,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'O' | 'o' => [0x00,0x3C,0x66,0x66,0x66,0x66,0x66,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'P' | 'p' => [0x00,0x7C,0x66,0x66,0x7C,0x60,0x60,0x60,0x60,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'Q' | 'q' => [0x00,0x3C,0x66,0x66,0x66,0x66,0x6E,0x3C,0x0E,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'R' | 'r' => [0x00,0x7C,0x66,0x66,0x7C,0x78,0x6C,0x66,0x66,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'S' | 's' => [0x00,0x3C,0x66,0x60,0x3C,0x06,0x06,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'T' | 't' => [0x00,0x7E,0x18,0x18,0x18,0x18,0x18,0x18,0x18,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'U' | 'u' => [0x00,0x66,0x66,0x66,0x66,0x66,0x66,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'V' | 'v' => [0x00,0x66,0x66,0x66,0x66,0x66,0x3C,0x3C,0x18,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'W' | 'w' => [0x00,0x63,0x63,0x63,0x6B,0x7F,0x77,0x63,0x63,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'X' | 'x' => [0x00,0x66,0x66,0x3C,0x18,0x3C,0x66,0x66,0x66,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'Y' | 'y' => [0x00,0x66,0x66,0x66,0x3C,0x18,0x18,0x18,0x18,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'Z' | 'z' => [0x00,0x7E,0x06,0x0C,0x18,0x30,0x60,0x60,0x7E,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        _ => [0x00,0x3C,0x42,0x42,0x42,0x42,0x42,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00], 
    }
}
