





use alloc::vec::Vec;
use alloc::vec;

use crate::audio::synth::{SynthEngine, Envelope, BR_};
use super::track::{Project, Track, Note, jtg};






#[derive(Debug, Clone, Copy)]
pub struct MixerChannel {
    
    pub hq: u8,
    
    pub arp: i8,
    
    pub so: bool,
    
    pub cic: bool,
}

impl MixerChannel {
    pub fn new() -> Self {
        Self {
            hq: 200,
            arp: 0,
            so: false,
            cic: false,
        }
    }
}






pub struct Mixer {
    
    pub lq: Vec<MixerChannel>,
    
    pub euo: u8,
}

impl Mixer {
    
    pub fn new(uwf: usize) -> Self {
        Self {
            lq: vec![MixerChannel::new(); uwf],
            euo: 220,
        }
    }

    
    pub fn chv(&mut self, bm: usize, hq: u8) -> Result<(), &'static str> {
        let channel = self.lq.ds(bm).ok_or("Invalid channel")?;
        channel.hq = hq;
        Ok(())
    }

    
    pub fn meq(&mut self, bm: usize, arp: i8) -> Result<(), &'static str> {
        let channel = self.lq.ds(bm).ok_or("Invalid channel")?;
        channel.arp = arp.qp(-100, 100);
        Ok(())
    }

    
    pub fn mlo(&mut self, bm: usize) -> Result<bool, &'static str> {
        let channel = self.lq.ds(bm).ok_or("Invalid channel")?;
        channel.so = !channel.so;
        Ok(channel.so)
    }

    
    pub fn mlr(&mut self, bm: usize) -> Result<bool, &'static str> {
        let channel = self.lq.ds(bm).ok_or("Invalid channel")?;
        channel.cic = !channel.cic;
        Ok(channel.cic)
    }

    
    pub fn tng(&self) -> bool {
        self.lq.iter().any(|r| r.cic)
    }

    
    pub fn tws(&self, bm: usize) -> bool {
        if let Some(channel) = self.lq.get(bm) {
            if channel.so { return false; }
            if self.tng() { return channel.cic; }
            true
        } else {
            false
        }
    }

    
    
    pub fn qjo(&self, bm: usize, fd: i32, hw: i32) -> (i32, i32) {
        if let Some(channel) = self.lq.get(bm) {
            let api = channel.hq as i32;
            
            
            
            
            let arp = channel.arp as i32;
            let udv = (100 - arp).qp(0, 200);
            let vyv = (100 + arp).qp(0, 200);

            let dm = fd * api / 255 * udv / 100;
            let m = hw * api / 255 * vyv / 100;
            (dm, m)
        } else {
            (fd, hw)
        }
    }
}






fn vwu(track: &Track, kz: u32, vb: u32, ayz: usize) -> Vec<i32> {
    let mut bi = vec![0i32; ayz];

    if track.ts.is_empty() {
        return bi;
    }

    
    let mut engine = SynthEngine::new();
    engine.dvs(track.ve);
    engine.qr = track.qr;

    
    
    struct Awe {
        dvj: usize,
        jb: u8,
        qm: u8,
        lgh: bool,
    }

    let mut events: Vec<Awe> = Vec::new();

    for jp in &track.ts {
        if jp.ckg() <= vb {
            continue; 
        }

        let orb = if jp.vb >= vb {
            jtg(jp.vb - vb, kz) as usize
        } else {
            0 
        };

        let oqy = jtg(
            jp.ckg().ao(vb), kz
        ) as usize;

        if orb < ayz {
            events.push(Awe {
                dvj: orb,
                jb: jp.jb,
                qm: jp.qm,
                lgh: true,
            });
        }

        if oqy < ayz {
            events.push(Awe {
                dvj: oqy,
                jb: jp.jb,
                qm: jp.qm,
                lgh: false,
            });
        }
    }

    
    events.bxf(|aa| aa.dvj);

    
    let mut fia = 0;
    let mut mkd = vec![0i16; 2]; 

    for yr in 0..ayz {
        
        while fia < events.len() && events[fia].dvj <= yr {
            let aiz = &events[fia];
            if aiz.lgh {
                engine.dtq(aiz.jb, aiz.qm);
            } else {
                engine.djx(aiz.jb);
            }
            fia += 1;
        }

        
        engine.tj(&mut mkd, 1);
        
        bi[yr] = (mkd[0] as i32 + mkd[1] as i32) / 2;
    }

    bi
}



pub fn pcb(nv: &Project, mixer: &Mixer, kz: u32, vb: u32) -> Vec<i16> {
    if nv.af.is_empty() {
        return Vec::new();
    }

    
    let cng = nv.oiu();
    if cng <= vb {
        return Vec::new();
    }

    let vwt = cng - vb;
    let ayz = jtg(vwt, kz) as usize;

    if ayz == 0 {
        return Vec::new();
    }

    
    let xlf: Vec<Vec<i32>> = nv.af.iter()
        .map(|track| vwu(track, kz, vb, ayz))
        .collect();

    
    let mut an = vec![0i16; ayz * 2];

    for yr in 0..ayz {
        let mut glh: i32 = 0;
        let mut gqz: i32 = 0;

        for (gck, xle) in xlf.iter().cf() {
            if !mixer.tws(gck) {
                continue;
            }

            let ony = xle[yr];
            let (dm, m) = mixer.qjo(gck, ony, ony);
            glh += dm;
            gqz += m;
        }

        
        glh = glh * mixer.euo as i32 / 255;
        gqz = gqz * mixer.euo as i32 / 255;

        
        glh = plx(glh);
        gqz = plx(gqz);

        an[yr * 2] = glh.qp(-32767, 32767) as i16;
        an[yr * 2 + 1] = gqz.qp(-32767, 32767) as i16;
    }

    an
}


fn plx(yr: i32) -> i32 {
    const Yz: i32 = 24000;
    if yr > Yz {
        let hik = yr - Yz;
        let ahf = hik * 8000 / (hik + 8000); 
        Yz + ahf
    } else if yr < -Yz {
        let hik = -yr - Yz;
        let ahf = hik * 8000 / (hik + 8000);
        -(Yz + ahf)
    } else {
        yr
    }
}
