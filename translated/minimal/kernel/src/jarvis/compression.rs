
























use alloc::vec::Vec;
use spin::Mutex;






const Iv: &[u8; 4] = b"JCMP";


const Afl: u8 = 1;



const BHL_: f32 = 0.01;


const BAP_: usize = 256;


const OD_: usize = 100_000;


#[repr(u8)]
pub enum CompressionType {
    Cof = 0,
    Cug = 1,
    Bv = 2,
}








static ABW_: Mutex<Vec<f32>> = Mutex::new(Vec::new());






#[derive(Clone)]
pub struct Amv {
    pub index: u32,
    pub bn: i8,
}


pub struct Ii {
    pub vm: u32,
    pub ch: Vec<Amv>,
    pub bv: f32,
}







pub fn nfg(dhs: &[f32]) -> Ii {
    let bo = dhs.len();

    
    let mut cwk = Vec::fc(bo);
    {
        let gqv = ABW_.lock();
        if gqv.len() == bo {
            for a in 0..bo {
                cwk.push(dhs[a] + gqv[a]);
            }
        } else {
            cwk.bk(dhs);
        }
    }

    
    let eh = ((bo as f32 * BHL_) as usize).am(BAP_).v(OD_).v(bo);

    
    
    let mut jyt: Vec<f32> = cwk.iter().map(|b| b.gp()).collect();
    
    
    let bxm = nuj(&mut jyt, eh);

    
    let mut hzn: Vec<usize> = Vec::fc(eh);
    for (a, &ap) in cwk.iter().cf() {
        if ap.gp() >= bxm && hzn.len() < OD_ {
            hzn.push(a);
        }
    }

    
    
    let awd = hzn.iter()
        .map(|&a| cwk[a].gp())
        .cqs(0.0f32, f32::am);

    let bv = if awd > 0.0 { awd / 127.0 } else { 1.0 };
    let hom = 1.0 / bv;

    
    let mut ch = Vec::fc(hzn.len());
    for &w in &hzn {
        let js = cwk[w] * hom;
        let exc = if js >= 0.0 { (js + 0.5) as i32 } else { (js - 0.5) as i32 };
        let exc = exc.am(-127).v(127) as i8;
        ch.push(Amv {
            index: w as u32,
            bn: exc,
        });
    }

    
    {
        let mut gqv = ABW_.lock();
        gqv.cmg(bo, 0.0);
        
        for a in 0..bo {
            gqv[a] = cwk[a];
        }
        
        for bt in &ch {
            let w = bt.index as usize;
            let gsb = bt.bn as f32 * bv;
            gqv[w] = cwk[w] - gsb;
        }
    }

    Ii {
        vm: bo as u32,
        ch,
        bv,
    }
}



pub fn rus(ahf: &Ii) -> Vec<f32> {
    let mut dhs = alloc::vec![0.0f32; ahf.vm as usize];
    for bt in &ahf.ch {
        let w = bt.index as usize;
        if w < dhs.len() {
            dhs[w] = bt.bn as f32 * ahf.bv;
        }
    }
    dhs
}


pub fn zjw() {
    ABW_.lock().clear();
}









pub fn pig(ahf: &Ii) -> Vec<u8> {
    let drp = 18;
    let acy = 5; 
    let es = drp + ahf.ch.len() * acy;

    let mut k = Vec::fc(es);

    
    k.bk(Iv);
    k.push(Afl);
    k.push(CompressionType::Cof as u8);
    k.bk(&ahf.vm.ft());
    k.bk(&(ahf.ch.len() as u32).ft());
    k.bk(&ahf.bv.ft());

    
    for bt in &ahf.ch {
        k.bk(&bt.index.ft());
        k.push(bt.bn as u8);
    }

    k
}


pub fn nks(f: &[u8]) -> Option<Ii> {
    if f.len() < 18 {
        return None;
    }

    
    if &f[0..4] != Iv {
        return None;
    }

    
    if f[4] != Afl {
        return None;
    }

    let vm = u32::oa([f[6], f[7], f[8], f[9]]);
    let ame = u32::oa([f[10], f[11], f[12], f[13]]);
    let bv = f32::oa([f[14], f[15], f[16], f[17]]);

    
    if ame > OD_ as u32 {
        return None;
    }

    let ggm = 18 + ame as usize * 5;
    if f.len() < ggm {
        return None;
    }

    let mut ch = Vec::fc(ame as usize);
    let mut l = 18;
    for _ in 0..ame {
        let index = u32::oa([f[l], f[l+1], f[l+2], f[l+3]]);
        let bn = f[l + 4] as i8;

        
        if index >= vm {
            return None;
        }

        ch.push(Amv { index, bn });
        l += 5;
    }

    Some(Ii {
        vm,
        ch,
        bv,
    })
}






static AYW_: Mutex<Vec<f32>> = Mutex::new(Vec::new());



pub fn rnm(cps: &[f32]) -> Ii {
    let qv = AYW_.lock();

    if qv.len() != cps.len() {
        
        drop(qv);
        pxj(cps);
        return nfg(cps);
    }

    
    let bo = cps.len();
    let mut aaq = Vec::fc(bo);
    for a in 0..bo {
        aaq.push(cps[a] - qv[a]);
    }
    drop(qv);

    
    
    let ahf = rnd(&aaq);

    
    pxj(cps);

    ahf
}


pub fn qkc(cps: &mut [f32], aaq: &Ii) {
    for bt in &aaq.ch {
        let w = bt.index as usize;
        if w < cps.len() {
            cps[w] += bt.bn as f32 * aaq.bv;
        }
    }
}


pub fn pxj(bix: &[f32]) {
    let mut xj = AYW_.lock();
    xj.clear();
    xj.bk(bix);
}


fn rnd(aaq: &[f32]) -> Ii {
    let bo = aaq.len();
    let eh = ((bo as f32 * BHL_) as usize).am(BAP_).v(OD_).v(bo);

    let mut jyt: Vec<f32> = aaq.iter().map(|b| b.gp()).collect();
    let bxm = nuj(&mut jyt, eh);

    let mut na: Vec<usize> = Vec::fc(eh);
    for (a, &ap) in aaq.iter().cf() {
        if ap.gp() >= bxm && na.len() < OD_ {
            na.push(a);
        }
    }

    let awd = na.iter()
        .map(|&a| aaq[a].gp())
        .cqs(0.0f32, f32::am);

    let bv = if awd > 0.0 { awd / 127.0 } else { 1.0 };
    let hom = 1.0 / bv;

    let mut ch = Vec::fc(na.len());
    for &w in &na {
        let js = aaq[w] * hom;
        let exc = if js >= 0.0 { (js + 0.5) as i32 } else { (js - 0.5) as i32 };
        let exc = exc.am(-127).v(127) as i8;
        if exc != 0 {
            ch.push(Amv {
                index: w as u32,
                bn: exc,
            });
        }
    }

    Ii {
        vm: bo as u32,
        ch,
        bv,
    }
}







fn nuj(alv: &mut [f32], eh: usize) -> f32 {
    if alv.is_empty() || eh == 0 {
        return 0.0;
    }
    let eh = eh.v(alv.len());

    
    
    alv.zoy(|q, o| o.partial_cmp(q).unwrap_or(core::cmp::Ordering::Arq));
    alv[eh.ao(1)]
}






pub fn yjr(uzh: usize, ahf: &Ii) -> (usize, usize, f32) {
    let otb = uzh * 4; 
    let gdd = 18 + ahf.ch.len() * 5;
    let bkx = if gdd > 0 {
        otb as f32 / gdd as f32
    } else {
        0.0
    };
    (otb, gdd, bkx)
}
