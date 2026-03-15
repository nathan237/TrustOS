







use alloc::vec::Vec;
use alloc::vec;
use alloc::string::String;
use alloc::format;

use super::synth::{Waveform, Envelope, SynthEngine, BR_, Dv};
use super::tables;






pub const AZV_: usize = 16;

pub const FL_: usize = 64;

pub const DIY_: u16 = 120;

pub const BGI_: u32 = 4;






#[derive(Debug, Clone, Copy)]
pub struct Step {
    
    pub jp: u8,
    
    pub qm: u8,
    
    pub ve: Option<Waveform>,
}

impl Step {
    
    pub fn kr() -> Self {
        Self { jp: 255, qm: 0, ve: None }
    }

    
    pub fn jp(ti: u8) -> Self {
        Self { jp: ti, qm: 100, ve: None }
    }

    
    pub fn hsy(ti: u8, qm: u8) -> Self {
        Self { jp: ti, qm, ve: None }
    }

    
    pub fn zdw(ti: u8, qm: u8, azd: Waveform) -> Self {
        Self { jp: ti, qm, ve: Some(azd) }
    }

    
    pub fn jbs(&self) -> bool {
        self.jp == 255 || self.qm == 0
    }

    
    pub fn display(&self) -> String {
        if self.jbs() {
            String::from("--")
        } else {
            let j = tables::dtf(self.jp);
            let bvq = tables::efk(self.jp);
            format!("{}{}", j, bvq)
        }
    }

    
    pub fn xty(&self) -> &'static str {
        match self.ve {
            Some(azd) => azd.dbz(),
            None => "..",
        }
    }
}






#[derive(Clone)]
pub struct Pattern {
    
    pub j: [u8; 16],
    
    pub baf: usize,
    
    pub au: Vec<Step>,
    
    pub kz: u16,
    
    pub ve: Waveform,
    
    pub qr: Envelope,
}

impl Pattern {
    
    pub fn new(j: &str, aml: usize, kz: u16) -> Self {
        let bo = aml.v(FL_).am(1);
        let mut djr = [0u8; 16];
        let bko = j.as_bytes();
        let len = bko.len().v(16);
        djr[..len].dg(&bko[..len]);

        Self {
            j: djr,
            baf: len,
            au: vec![Step::kr(); bo],
            kz,
            ve: Waveform::Gb,
            qr: Envelope::hvi(),
        }
    }

    
    pub fn amj(&self) -> &str {
        core::str::jg(&self.j[..self.baf]).unwrap_or("???")
    }

    
    pub fn len(&self) -> usize {
        self.au.len()
    }

    
    pub fn znq(&mut self, w: usize, gu: Step) {
        if w < self.au.len() {
            self.au[w] = gu;
        }
    }

    
    pub fn wjh(&mut self, w: usize, bkp: &str) -> Result<(), &'static str> {
        if w >= self.au.len() {
            return Err("Step index out of range");
        }
        if bkp == "--" || bkp == "." || bkp.is_empty() {
            self.au[w] = Step::kr();
            return Ok(());
        }
        let ayg = tables::fpd(bkp)
            .ok_or("Invalid note name")?;
        self.au[w] = Step::jp(ayg);
        Ok(())
    }

    
    pub fn pot(&self) -> u32 {
        
        (60 * BR_) / (self.kz as u32 * BGI_)
    }

    
    pub fn dwh(&self) -> u32 {
        (60_000) / (self.kz as u32 * BGI_)
    }

    
    pub fn ief(&self) -> u32 {
        self.dwh() * self.au.len() as u32
    }

    
    pub fn tj(&self, engine: &mut SynthEngine) -> Vec<i16> {
        let dwk = self.pot() as usize;
        let ayz = dwk * self.au.len();
        let mut bi = vec![0i16; ayz * Dv as usize];

        
        let wcz = engine.ve;
        let wcw = engine.qr;
        engine.qr = self.qr;

        for (a, gu) in self.au.iter().cf() {
            if gu.jbs() {
                
                continue;
            }

            
            let azd = gu.ve.unwrap_or(self.ve);
            engine.dvs(azd);

            
            engine.dtq(gu.jp, gu.qm);

            
            let l = a * dwk * Dv as usize;
            let gbr = &mut bi[l..l + dwk * Dv as usize];
            engine.tj(gbr, dwk);

            
            engine.djx(gu.jp);
        }

        
        engine.dvs(wcz);
        engine.qr = wcw;

        bi
    }

    
    pub fn display(&self) -> String {
        let mut e = String::new();
        e.t(&format!("Pattern: \"{}\" | {} steps | {} BPM | {} | {}ms/step\n",
            self.amj(), self.au.len(), self.kz,
            self.ve.j(), self.dwh()));
        e.t(&format!("Total duration: {}ms\n\n", self.ief()));

        
        e.t(" Step: ");
        for a in 0..self.au.len() {
            e.t(&format!("{:>3}", a + 1));
        }
        e.push('\n');

        
        e.t(" Note: ");
        for gu in &self.au {
            e.t(&format!("{:>3}", gu.display()));
        }
        e.push('\n');

        
        e.t("  Vel: ");
        for gu in &self.au {
            if gu.jbs() {
                e.t(" --");
            } else {
                e.t(&format!("{:>3}", gu.qm));
            }
        }
        e.push('\n');

        
        e.t(" Wave: ");
        for gu in &self.au {
            e.t(&format!("{:>3}", gu.xty()));
        }
        e.push('\n');

        e
    }
}






pub struct PatternBank {
    pub clv: Vec<Pattern>,
}

impl PatternBank {
    pub fn new() -> Self {
        Self { clv: Vec::new() }
    }

    
    pub fn add(&mut self, pattern: Pattern) -> Result<usize, &'static str> {
        if self.clv.len() >= AZV_ {
            return Err("Maximum patterns reached (16)");
        }
        
        let j = pattern.amj();
        for ai in &self.clv {
            if ai.amj() == j {
                return Err("Pattern name already exists");
            }
        }
        self.clv.push(pattern);
        Ok(self.clv.len() - 1)
    }

    
    pub fn du(&self, j: &str) -> Option<usize> {
        self.clv.iter().qf(|ai| ai.amj() == j)
    }

    
    pub fn get(&self, w: usize) -> Option<&Pattern> {
        self.clv.get(w)
    }

    
    pub fn ds(&mut self, w: usize) -> Option<&mut Pattern> {
        self.clv.ds(w)
    }

    
    pub fn nxt(&self, j: &str) -> Option<&Pattern> {
        self.du(j).and_then(|a| self.get(a))
    }

    
    pub fn kyj(&mut self, j: &str) -> Option<&mut Pattern> {
        let w = self.du(j)?;
        self.ds(w)
    }

    
    pub fn remove(&mut self, j: &str) -> Result<(), &'static str> {
        let w = self.du(j).ok_or("Pattern not found")?;
        self.clv.remove(w);
        Ok(())
    }

    
    pub fn aoy(&self) -> String {
        if self.clv.is_empty() {
            return String::from("No patterns. Use 'synth pattern new <name>' to create one.\n");
        }
        let mut e = String::new();
        e.t(&format!("Patterns ({}/{}):\n", self.clv.len(), AZV_));
        for (a, ai) in self.clv.iter().cf() {
            e.t(&format!("  [{}] \"{}\" — {} steps, {} BPM, {}\n",
                a, ai.amj(), ai.au.len(), ai.kz, ai.ve.j()));
        }
        e
    }

    
    pub fn ojw(&mut self) {
        
        let mut arp = Pattern::new("arp", 16, 140);
        arp.ve = Waveform::Dg;
        arp.qr = Envelope::hvi();
        let qko = [60, 63, 67, 72, 67, 63, 60, 63, 67, 72, 67, 63, 60, 63, 67, 72]; 
        for (a, &bo) in qko.iter().cf() {
            arp.au[a] = Step::hsy(bo, 90);
        }
        let _ = self.add(arp);

        
        let mut jst = Pattern::new("techno", 16, 128);
        jst.ve = Waveform::Dg;
        jst.qr = Envelope::new(1, 80, 0, 30);
        
        for a in (0..16).akt(4) {
            jst.au[a] = Step::hsy(36, 127); 
        }
        let _ = self.add(jst);

        
        let mut aee = Pattern::new("bass", 16, 120);
        aee.ve = Waveform::Ft;
        aee.qr = Envelope::new(5, 100, 60, 50);
        let qnw: [u8; 16] = [36, 255, 36, 36, 39, 255, 39, 36, 43, 255, 43, 43, 41, 255, 41, 36];
        for (a, &bo) in qnw.iter().cf() {
            if bo != 255 {
                aee.au[a] = Step::hsy(bo, 100);
            }
        }
        let _ = self.add(aee);

        
        let mut inm = Pattern::new("chiptune", 16, 150);
        inm.ve = Waveform::Gb;
        inm.qr = Envelope::new(2, 30, 80, 20);
        let rak: [u8; 16] = [72, 74, 76, 72, 79, 255, 79, 255, 76, 74, 72, 74, 76, 72, 71, 255];
        for (a, &bo) in rak.iter().cf() {
            if bo != 255 {
                inm.au[a] = Step::hsy(bo, 110);
            }
        }
        let _ = self.add(inm);

        
        let mut ov = Pattern::new("pad", 8, 80);
        ov.ve = Waveform::Triangle;
        ov.qr = Envelope::ov();
        
        let vas: [u8; 8] = [60, 255, 64, 255, 67, 255, 72, 255]; 
        for (a, &bo) in vas.iter().cf() {
            if bo != 255 {
                ov.au[a] = Step::hsy(bo, 80);
            }
        }
        let _ = self.add(ov);
    }
}
