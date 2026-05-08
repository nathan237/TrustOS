
























use alloc::vec::Vec;
use spin::Mutex;






const Dp: &[u8; 4] = b"JCMP";


const Nt: u8 = 1;



const BJP_: f32 = 0.01;


const BCR_: usize = 256;


const PB_: usize = 100_000;


#[repr(u8)]
pub enum CompressionType {
    TopKQuant = 0,
    Delta = 1,
    Full = 2,
}








static ADM_: Mutex<Vec<f32>> = Mutex::new(Vec::new());






#[derive(Clone)]
pub struct Qj {
    pub index: u32,
    pub value: i8,
}


pub struct Dm {
    pub param_count: u32,
    pub entries: Vec<Qj>,
    pub scale: f32,
}







pub fn hne(bgs: &[f32]) -> Dm {
    let ae = bgs.len();

    
    let mut bav = Vec::with_capacity(ae);
    {
        let ddl = ADM_.lock();
        if ddl.len() == ae {
            for i in 0..ae {
                bav.push(bgs[i] + ddl[i]);
            }
        } else {
            bav.extend_from_slice(bgs);
        }
    }

    
    let k = ((ae as f32 * BJP_) as usize).max(BCR_).min(PB_).min(ae);

    
    
    let mut ffx: Vec<f32> = bav.iter().map(|x| x.abs()).collect();
    
    
    let amz = hyx(&mut ffx, k);

    
    let mut dza: Vec<usize> = Vec::with_capacity(k);
    for (i, &val) in bav.iter().enumerate() {
        if val.abs() >= amz && dza.len() < PB_ {
            dza.push(i);
        }
    }

    
    
    let yw = dza.iter()
        .map(|&i| bav[i].abs())
        .fold(0.0f32, f32::max);

    let scale = if yw > 0.0 { yw / 127.0 } else { 1.0 };
    let dsh = 1.0 / scale;

    
    let mut entries = Vec::with_capacity(dza.len());
    for &idx in &dza {
        let dm = bav[idx] * dsh;
        let ccy = if dm >= 0.0 { (dm + 0.5) as i32 } else { (dm - 0.5) as i32 };
        let ccy = ccy.max(-127).min(127) as i8;
        entries.push(Qj {
            index: idx as u32,
            value: ccy,
        });
    }

    
    {
        let mut ddl = ADM_.lock();
        ddl.resize(ae, 0.0);
        
        for i in 0..ae {
            ddl[i] = bav[i];
        }
        
        for entry in &entries {
            let idx = entry.index as usize;
            let deg = entry.value as f32 * scale;
            ddl[idx] = bav[idx] - deg;
        }
    }

    Dm {
        param_count: ae as u32,
        entries,
        scale,
    }
}



pub fn lct(qv: &Dm) -> Vec<f32> {
    let mut bgs = alloc::vec![0.0f32; qv.param_count as usize];
    for entry in &qv.entries {
        let idx = entry.index as usize;
        if idx < bgs.len() {
            bgs[idx] = entry.value as f32 * qv.scale;
        }
    }
    bgs
}


pub fn que() {
    ADM_.lock().clear();
}









pub fn jet(qv: &Dm) -> Vec<u8> {
    let bms = 18;
    let oi = 5; 
    let av = bms + qv.entries.len() * oi;

    let mut buf = Vec::with_capacity(av);

    
    buf.extend_from_slice(Dp);
    buf.push(Nt);
    buf.push(CompressionType::TopKQuant as u8);
    buf.extend_from_slice(&qv.param_count.to_be_bytes());
    buf.extend_from_slice(&(qv.entries.len() as u32).to_be_bytes());
    buf.extend_from_slice(&qv.scale.to_be_bytes());

    
    for entry in &qv.entries {
        buf.extend_from_slice(&entry.index.to_be_bytes());
        buf.push(entry.value as u8);
    }

    buf
}


pub fn hro(data: &[u8]) -> Option<Dm> {
    if data.len() < 18 {
        return None;
    }

    
    if &data[0..4] != Dp {
        return None;
    }

    
    if data[4] != Nt {
        return None;
    }

    let param_count = u32::from_be_bytes([data[6], data[7], data[8], data[9]]);
    let entry_count = u32::from_be_bytes([data[10], data[11], data[12], data[13]]);
    let scale = f32::from_be_bytes([data[14], data[15], data[16], data[17]]);

    
    if entry_count > PB_ as u32 {
        return None;
    }

    let cxj = 18 + entry_count as usize * 5;
    if data.len() < cxj {
        return None;
    }

    let mut entries = Vec::with_capacity(entry_count as usize);
    let mut offset = 18;
    for _ in 0..entry_count {
        let index = u32::from_be_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]]);
        let value = data[offset + 4] as i8;

        
        if index >= param_count {
            return None;
        }

        entries.push(Qj { index, value });
        offset += 5;
    }

    Some(Dm {
        param_count,
        entries,
        scale,
    })
}






static BAX_: Mutex<Vec<f32>> = Mutex::new(Vec::new());



pub fn kwq(awt: &[f32]) -> Dm {
    let last = BAX_.lock();

    if last.len() != awt.len() {
        
        drop(last);
        jpi(awt);
        return hne(awt);
    }

    
    let ae = awt.len();
    let mut mk = Vec::with_capacity(ae);
    for i in 0..ae {
        mk.push(awt[i] - last[i]);
    }
    drop(last);

    
    
    let qv = kwi(&mk);

    
    jpi(awt);

    qv
}


pub fn jxg(awt: &mut [f32], mk: &Dm) {
    for entry in &mk.entries {
        let idx = entry.index as usize;
        if idx < awt.len() {
            awt[idx] += entry.value as f32 * mk.scale;
        }
    }
}


pub fn jpi(afx: &[f32]) {
    let mut jp = BAX_.lock();
    jp.clear();
    jp.extend_from_slice(afx);
}


fn kwi(mk: &[f32]) -> Dm {
    let ae = mk.len();
    let k = ((ae as f32 * BJP_) as usize).max(BCR_).min(PB_).min(ae);

    let mut ffx: Vec<f32> = mk.iter().map(|x| x.abs()).collect();
    let amz = hyx(&mut ffx, k);

    let mut selected: Vec<usize> = Vec::with_capacity(k);
    for (i, &val) in mk.iter().enumerate() {
        if val.abs() >= amz && selected.len() < PB_ {
            selected.push(i);
        }
    }

    let yw = selected.iter()
        .map(|&i| mk[i].abs())
        .fold(0.0f32, f32::max);

    let scale = if yw > 0.0 { yw / 127.0 } else { 1.0 };
    let dsh = 1.0 / scale;

    let mut entries = Vec::with_capacity(selected.len());
    for &idx in &selected {
        let dm = mk[idx] * dsh;
        let ccy = if dm >= 0.0 { (dm + 0.5) as i32 } else { (dm - 0.5) as i32 };
        let ccy = ccy.max(-127).min(127) as i8;
        if ccy != 0 {
            entries.push(Qj {
                index: idx as u32,
                value: ccy,
            });
        }
    }

    Dm {
        param_count: ae as u32,
        entries,
        scale,
    }
}







fn hyx(values: &mut [f32], k: usize) -> f32 {
    if values.is_empty() || k == 0 {
        return 0.0;
    }
    let k = k.min(values.len());

    
    
    values.sort_unstable_by(|a, b| b.partial_cmp(a).unwrap_or(core::cmp::Ordering::Equal));
    values[k.saturating_sub(1)]
}






pub fn qaz(original_params: usize, qv: &Dm) -> (usize, usize, f32) {
    let isu = original_params * 4; 
    let cvl = 18 + qv.entries.len() * 5;
    let zi = if cvl > 0 {
        isu as f32 / cvl as f32
    } else {
        0.0
    };
    (isu, cvl, zi)
}
