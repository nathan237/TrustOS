







use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;


#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum Op {
    
    Adz,    
    Bpq,    
    Bpp,   
    Bpr,    
    Bpg,        
    Abf,        

    
    Ti,  
    Qv, 

    
    Cgr,  
    Cnl, 

    
    Zq, Azl, Avv, Ara, Bmj, Bnd,
    
    Bxr, Cnn, Chm, Cay, Chx,
    
    Bga, Bnc, Auy, Bik, Bkz, Bhw,
    Cbs, Chw, Cgy, Ceq, Cgo, Cej,
    
    Ex, Fx, Np,
    
    Vm, Vn, Vo, Ob, Oc,

    
    Biy, Bgk,

    
    Nh,       
    Ajk, 
    En,       
    Bdi, 
    Hd,     

    
    Avz,   
    Bby,   
    Bbz,   
    Bxy,   
    Bxz,  

    
    Cnm,  

    
    Bio,       
}

impl Op {
    
    
    #[inline(always)]
    fn ckp(p: u8) -> Option<Op> {
        if p <= Op::Bio as u8 {
            
            Some(unsafe { core::mem::transmute(p) })
        } else {
            None
        }
    }
}


#[derive(Debug, Clone)]
pub enum Value {
    Ab(i64),
    R(f64),
    Em(bool),
    He(String),
    U(Vec<Value>),
    Cn,
}

impl Value {
    fn abb(&self) -> Result<i64, String> {
        match self { Value::Ab(p) => Ok(*p), _ => Err(format!("expected i64, got {:?}", self)) }
    }
    fn dyj(&self) -> Result<f64, String> {
        match self { Value::R(p) => Ok(*p), _ => Err(format!("expected f64, got {:?}", self)) }
    }
    fn gah(&self) -> Result<bool, String> {
        match self { Value::Em(p) => Ok(*p), _ => Err(format!("expected bool, got {:?}", self)) }
    }
    fn guq(&self) -> String {
        match self {
            Value::Ab(p) => format!("{}", p),
            Value::R(p) => format!("{:.6}", p),
            Value::Em(p) => format!("{}", p),
            Value::He(e) => e.clone(),
            Value::U(q) => {
                let pj: Vec<String> = q.iter().map(|p| p.guq()).collect();
                format!("[{}]", pj.rr(", "))
            }
            Value::Cn => String::from("()"),
        }
    }
}


#[derive(Debug, Clone)]
pub struct Bs {
    pub j: String,
    pub qkm: u8,         
    pub bbx: u8,        
    pub aj: Vec<u8>,     
}


#[derive(Debug, Clone)]
pub struct Aaf {
    pub ajb: Vec<Bs>,
    pub pd: Vec<String>,     
    pub bt: usize,             
}


struct CallFrame {
    hkq: usize,
    ip: usize,
    ar: usize, 
    bbx: [Value; 256],
}

impl CallFrame {
    fn new(hkq: usize, ar: usize) -> Self {
        const Cph: Value = Value::Cn;
        Self {
            hkq,
            ip: 0,
            ar,
            bbx: [Cph; 256],
        }
    }
}


const AMV_: u8 = 0;
const AMW_: u8 = 1;
const AMQ_: u8 = 2;
const AMX_: u8 = 3;
const ANH_: u8 = 4;
const ANG_: u8 = 5;
const AND_: u8 = 6;
const AMD_: u8 = 7;
const AMU_: u8 = 8;
const AMG_: u8 = 9;
const AMM_: u8 = 10;
const AMJ_: u8 = 11;
const AMI_: u8 = 12;
const ANA_: u8 = 13;
const AMZ_: u8 = 14;
const AMN_: u8 = 15;
const AMK_: u8 = 16;
const ANC_: u8 = 17;
const ANF_: u8 = 18;
const AMY_: u8 = 19;

const AMF_: u8 = 20;
const AME_: u8 = 21;
const ANE_: u8 = 22;
const AMR_: u8 = 23;
const AMP_: u8 = 24;
const ANI_: u8 = 25;
const AML_: u8 = 26;
const AMO_: u8 = 27;
const ANB_: u8 = 28;
const AMH_: u8 = 29;
const AMS_: u8 = 30;
const AMT_: u8 = 31;


pub fn imj(j: &str) -> Option<u8> {
    match j {
        "print" => Some(AMV_),
        "println" => Some(AMW_),
        "len" => Some(AMQ_),
        "push" => Some(AMX_),
        "to_string" => Some(ANH_),
        "to_int" => Some(ANG_),
        "to_float" => Some(ANF_),
        "sqrt" => Some(AND_),
        "abs" => Some(AMD_),
        "pixel" => Some(AMU_),
        "clear_screen" => Some(AMG_),
        "fill_rect" => Some(AMM_),
        "draw_line" => Some(AMJ_),
        "draw_circle" => Some(AMI_),
        "screen_w" => Some(ANA_),
        "screen_h" => Some(AMZ_),
        "flush" => Some(AMN_),
        "draw_text" => Some(AMK_),
        "sleep" => Some(ANC_),
        "read_line" => Some(AMY_),
        
        "beat" => Some(AMF_),
        "bass" => Some(AME_),
        "sub_bass" => Some(ANE_),
        "mid" => Some(AMR_),
        "high_mid" => Some(AMP_),
        "treble" => Some(ANI_),
        "energy" => Some(AML_),
        "frame_num" => Some(AMO_),
        "sin_f" => Some(ANB_),
        "cos_f" => Some(AMH_),
        "mouse_x" => Some(AMS_),
        "mouse_y" => Some(AMT_),
        _ => None,
    }
}


pub fn bna(hby: &Aaf) -> Result<String, String> {
    let mut an = String::new();
    let mut jo: Vec<Value> = Vec::fc(1024);
    let mut vj: Vec<CallFrame> = Vec::fc(64);

    vj.push(CallFrame::new(hby.bt, 0));

    let csk = 500_000_000; 
    let mut au = 0;

    loop {
        au += 1;
        if au > csk {
            return Err(String::from("execution limit exceeded (10M steps)"));
        }

        let frame = vj.dsq().ok_or("no call frame")?;
        let ke = &hby.ajb[frame.hkq];

        if frame.ip >= ke.aj.len() {
            
            if vj.len() <= 1 { break; }
            vj.pop();
            jo.push(Value::Cn);
            continue;
        }

        let oso = ke.aj[frame.ip];
        frame.ip += 1;

        let op = match Op::ckp(oso) {
            Some(dkb) => dkb,
            None => return Err(format!("unknown opcode: {}", oso)),
        };

        match op {
            Op::Adz => {
                let bf = day(&ke.aj, &mut frame.ip, 8);
                let p = i64::dj(bf.try_into().unwrap());
                jo.push(Value::Ab(p));
            }
            Op::Bpq => {
                let bf = day(&ke.aj, &mut frame.ip, 8);
                let p = f64::dj(bf.try_into().unwrap());
                jo.push(Value::R(p));
            }
            Op::Bpp => {
                let p = ke.aj[frame.ip] != 0;
                frame.ip += 1;
                jo.push(Value::Em(p));
            }
            Op::Bpr => {
                let w = alp(&ke.aj, &mut frame.ip) as usize;
                let e = hby.pd.get(w).abn().age();
                jo.push(Value::He(e));
            }
            Op::Bpg => { jo.pop(); }
            Op::Abf => {
                let p = jo.qv().abn().unwrap_or(Value::Cn);
                jo.push(p);
            }
            Op::Ti => {
                let gk = ke.aj[frame.ip] as usize;
                frame.ip += 1;
                jo.push(frame.bbx[gk].clone());
            }
            Op::Qv => {
                let gk = ke.aj[frame.ip] as usize;
                frame.ip += 1;
                let ap = jo.pop().unwrap_or(Value::Cn);
                frame.bbx[gk] = ap;
            }
            Op::Cgr | Op::Cnl => {
                
                frame.ip += 2;
            }
            
            Op::Zq => { emu(&mut jo, |q, o| q.cn(o), |q, o| q + o)?; }
            Op::Azl => { emu(&mut jo, |q, o| q.nj(o), |q, o| q - o)?; }
            Op::Avv => { emu(&mut jo, |q, o| q.hx(o), |q, o| q * o)?; }
            Op::Ara => {
                let coo = jo.pop().unwrap_or(Value::Ab(0));
                let ddt = jo.pop().unwrap_or(Value::Ab(0));
                match (&ddt, &coo) {
                    (Value::R(q), Value::R(o)) => jo.push(Value::R(q / o)),
                    (Value::Ab(q), Value::R(o)) => jo.push(Value::R(*q as f64 / o)),
                    (Value::R(q), Value::Ab(o)) => jo.push(Value::R(q / *o as f64)),
                    _ => {
                        let o = coo.abb()?;
                        let q = ddt.abb()?;
                        if o == 0 { return Err(String::from("division by zero")); }
                        jo.push(Value::Ab(q / o));
                    }
                }
            }
            Op::Bmj => {
                let coo = jo.pop().unwrap_or(Value::Ab(0));
                let ddt = jo.pop().unwrap_or(Value::Ab(0));
                match (&ddt, &coo) {
                    (Value::R(q), Value::R(o)) => jo.push(Value::R(q % o)),
                    (Value::Ab(q), Value::R(o)) => jo.push(Value::R(*q as f64 % o)),
                    (Value::R(q), Value::Ab(o)) => jo.push(Value::R(q % *o as f64)),
                    _ => {
                        let o = coo.abb()?;
                        let q = ddt.abb()?;
                        if o == 0 { return Err(String::from("modulo by zero")); }
                        jo.push(Value::Ab(q % o));
                    }
                }
            }
            Op::Bnd => {
                let p = jo.pop().unwrap_or(Value::Ab(0));
                match p {
                    Value::R(bb) => jo.push(Value::R(-bb)),
                    _ => jo.push(Value::Ab(-p.abb()?)),
                }
            }
            
            Op::Bxr => { ilp(&mut jo, |q, o| q + o)?; }
            Op::Cnn => { ilp(&mut jo, |q, o| q - o)?; }
            Op::Chm => { ilp(&mut jo, |q, o| q * o)?; }
            Op::Cay => { ilp(&mut jo, |q, o| q / o)?; }
            Op::Chx => {
                let p = jo.pop().unwrap_or(Value::R(0.0)).dyj()?;
                jo.push(Value::R(-p));
            }
            
            Op::Bga => { gct(&mut jo, |q, o| q == o, |q, o| q == o)?; }
            Op::Bnc => { gct(&mut jo, |q, o| q != o, |q, o| q != o)?; }
            Op::Auy => { gct(&mut jo, |q, o| q < o, |q, o| q < o)?; }
            Op::Bik => { gct(&mut jo, |q, o| q > o, |q, o| q > o)?; }
            Op::Bkz => { gct(&mut jo, |q, o| q <= o, |q, o| q <= o)?; }
            Op::Bhw => { gct(&mut jo, |q, o| q >= o, |q, o| q >= o)?; }
            
            Op::Cbs => { gcs(&mut jo, |q, o| q == o)?; }
            Op::Chw => { gcs(&mut jo, |q, o| q != o)?; }
            Op::Cgy => { gcs(&mut jo, |q, o| q < o)?; }
            Op::Ceq => { gcs(&mut jo, |q, o| q > o)?; }
            Op::Cgo => { gcs(&mut jo, |q, o| q <= o)?; }
            Op::Cej => { gcs(&mut jo, |q, o| q >= o)?; }
            
            Op::Ex => {
                let o = jo.pop().unwrap_or(Value::Em(false)).gah()?;
                let q = jo.pop().unwrap_or(Value::Em(false)).gah()?;
                jo.push(Value::Em(q && o));
            }
            Op::Fx => {
                let o = jo.pop().unwrap_or(Value::Em(false)).gah()?;
                let q = jo.pop().unwrap_or(Value::Em(false)).gah()?;
                jo.push(Value::Em(q || o));
            }
            Op::Np => {
                let p = jo.pop().unwrap_or(Value::Em(false)).gah()?;
                jo.push(Value::Em(!p));
            }
            
            Op::Vm => { emu(&mut jo, |q, o| q & o, |q, o| (q as i64 & o as i64) as f64)?; }
            Op::Vn => { emu(&mut jo, |q, o| q | o, |q, o| (q as i64 | o as i64) as f64)?; }
            Op::Vo => { emu(&mut jo, |q, o| q ^ o, |q, o| (q as i64 ^ o as i64) as f64)?; }
            Op::Ob => { emu(&mut jo, |q, o| q << (o & 63), |q, o| ((q as i64) << (o as i64 & 63)) as f64)?; }
            Op::Oc => { emu(&mut jo, |q, o| q >> (o & 63), |q, o| ((q as i64) >> (o as i64 & 63)) as f64)?; }
            
            Op::Biy => {
                let p = jo.pop().unwrap_or(Value::Ab(0)).abb()?;
                jo.push(Value::R(p as f64));
            }
            Op::Bgk => {
                let p = jo.pop().unwrap_or(Value::R(0.0)).dyj()?;
                jo.push(Value::Ab(p as i64));
            }
            
            Op::Nh => {
                let dz = alp(&ke.aj, &mut frame.ip) as usize;
                frame.ip = dz;
            }
            Op::Ajk => {
                let dz = alp(&ke.aj, &mut frame.ip) as usize;
                let mo = jo.pop().unwrap_or(Value::Em(false)).gah()?;
                if !mo { frame.ip = dz; }
            }
            Op::En => {
                let hkq = alp(&ke.aj, &mut frame.ip) as usize;
                let byg = ke.aj[frame.ip] as usize;
                frame.ip += 1;
                
                let mut n = Vec::fc(byg);
                for _ in 0..byg {
                    n.push(jo.pop().unwrap_or(Value::Cn));
                }
                n.dbh();
                
                let mut opu = CallFrame::new(hkq, jo.len());
                for (a, ji) in n.dse().cf() {
                    opu.bbx[a] = ji;
                }
                vj.push(opu);
            }
            Op::Bdi => {
                let que = ke.aj[frame.ip];
                frame.ip += 1;
                let byg = ke.aj[frame.ip] as usize;
                frame.ip += 1;
                let mut n = Vec::fc(byg);
                for _ in 0..byg {
                    n.push(jo.pop().unwrap_or(Value::Cn));
                }
                n.dbh();
                let result = sob(que, &n, &mut an)?;
                jo.push(result);
            }
            Op::Hd => {
                let aux = jo.pop().unwrap_or(Value::Cn);
                if vj.len() <= 1 {
                    jo.push(aux);
                    break;
                }
                vj.pop();
                jo.push(aux);
            }
            
            Op::Avz => {
                let az = alp(&ke.aj, &mut frame.ip) as usize;
                let mut sy = Vec::fc(az);
                for _ in 0..az {
                    sy.push(jo.pop().unwrap_or(Value::Cn));
                }
                sy.dbh();
                jo.push(Value::U(sy));
            }
            Op::Bby => {
                let w = jo.pop().unwrap_or(Value::Ab(0)).abb()? as usize;
                let sy = jo.pop().unwrap_or(Value::U(Vec::new()));
                match sy {
                    Value::U(q) => {
                        let p = q.get(w).abn().unwrap_or(Value::Cn);
                        jo.push(p);
                    }
                    Value::He(e) => {
                        let r = e.as_bytes().get(w).hu().unwrap_or(0);
                        jo.push(Value::Ab(r as i64));
                    }
                    _ => return Err(String::from("index on non-array")),
                }
            }
            Op::Bbz => {
                let ap = jo.pop().unwrap_or(Value::Cn);
                let w = jo.pop().unwrap_or(Value::Ab(0)).abb()? as usize;
                let sy = jo.pop().unwrap_or(Value::U(Vec::new()));
                match sy {
                    Value::U(mut q) => {
                        if w < q.len() { q[w] = ap; }
                        jo.push(Value::U(q));
                    }
                    _ => return Err(String::from("index-set on non-array")),
                }
            }
            Op::Bxy => {
                let p = jo.pop().unwrap_or(Value::Cn);
                let len = match &p {
                    Value::U(q) => q.len() as i64,
                    Value::He(e) => e.len() as i64,
                    _ => 0,
                };
                jo.push(Value::Ab(len));
            }
            Op::Bxz => {
                let ap = jo.pop().unwrap_or(Value::Cn);
                let sy = jo.pop().unwrap_or(Value::U(Vec::new()));
                match sy {
                    Value::U(mut q) => {
                        q.push(ap);
                        jo.push(Value::U(q));
                    }
                    _ => return Err(String::from("push on non-array")),
                }
            }
            Op::Cnm => {
                let o = jo.pop().unwrap_or(Value::He(String::new())).guq();
                let q = jo.pop().unwrap_or(Value::He(String::new())).guq();
                jo.push(Value::He(format!("{}{}", q, o)));
            }
            Op::Bio => break,
        }
    }

    Ok(an)
}



fn day(aj: &[u8], ip: &mut usize, bo: usize) -> Vec<u8> {
    let bf = aj[*ip..*ip + bo].ip();
    *ip += bo;
    bf
}

fn alp(aj: &[u8], ip: &mut usize) -> u16 {
    let p = u16::dj([aj[*ip], aj[*ip + 1]]);
    *ip += 2;
    p
}

fn emu(jo: &mut Vec<Value>, cqk: fn(i64, i64) -> i64, fik: fn(f64, f64) -> f64) -> Result<(), String> {
    let coo = jo.pop().unwrap_or(Value::Ab(0));
    let ddt = jo.pop().unwrap_or(Value::Ab(0));
    
    match (&ddt, &coo) {
        (Value::R(q), Value::R(o)) => jo.push(Value::R(fik(*q, *o))),
        (Value::Ab(q), Value::R(o)) => jo.push(Value::R(fik(*q as f64, *o))),
        (Value::R(q), Value::Ab(o)) => jo.push(Value::R(fik(*q, *o as f64))),
        _ => {
            let q = ddt.abb()?;
            let o = coo.abb()?;
            jo.push(Value::Ab(cqk(q, o)));
        }
    }
    Ok(())
}

fn ilp(jo: &mut Vec<Value>, bb: fn(f64, f64) -> f64) -> Result<(), String> {
    let o = jo.pop().unwrap_or(Value::R(0.0)).dyj()?;
    let q = jo.pop().unwrap_or(Value::R(0.0)).dyj()?;
    jo.push(Value::R(bb(q, o)));
    Ok(())
}

fn gct(jo: &mut Vec<Value>, cqk: fn(i64, i64) -> bool, fik: fn(f64, f64) -> bool) -> Result<(), String> {
    let coo = jo.pop().unwrap_or(Value::Ab(0));
    let ddt = jo.pop().unwrap_or(Value::Ab(0));
    
    match (&ddt, &coo) {
        (Value::R(q), Value::R(o)) => jo.push(Value::Em(fik(*q, *o))),
        (Value::Ab(q), Value::R(o)) => jo.push(Value::Em(fik(*q as f64, *o))),
        (Value::R(q), Value::Ab(o)) => jo.push(Value::Em(fik(*q, *o as f64))),
        _ => {
            let q = ddt.abb()?;
            let o = coo.abb()?;
            jo.push(Value::Em(cqk(q, o)));
        }
    }
    Ok(())
}

fn gcs(jo: &mut Vec<Value>, bb: fn(f64, f64) -> bool) -> Result<(), String> {
    let o = jo.pop().unwrap_or(Value::R(0.0)).dyj()?;
    let q = jo.pop().unwrap_or(Value::R(0.0)).dyj()?;
    jo.push(Value::Em(bb(q, o)));
    Ok(())
}


fn sob(ad: u8, n: &[Value], an: &mut String) -> Result<Value, String> {
    match ad {
        AMV_ => {
            for ji in n { an.t(&ji.guq()); }
            Ok(Value::Cn)
        }
        AMW_ => {
            for ji in n { an.t(&ji.guq()); }
            an.push('\n');
            Ok(Value::Cn)
        }
        AMQ_ => {
            let p = n.fv().unwrap_or(&Value::Cn);
            match p {
                Value::U(q) => Ok(Value::Ab(q.len() as i64)),
                Value::He(e) => Ok(Value::Ab(e.len() as i64)),
                _ => Ok(Value::Ab(0)),
            }
        }
        AMX_ => {
            if n.len() >= 2 {
                if let Value::U(mut q) = n[0].clone() {
                    q.push(n[1].clone());
                    return Ok(Value::U(q));
                }
            }
            Err(String::from("push expects (array, value)"))
        }
        ANH_ => {
            let p = n.fv().unwrap_or(&Value::Cn);
            Ok(Value::He(p.guq()))
        }
        ANG_ => {
            let p = n.fv().unwrap_or(&Value::Cn);
            match p {
                Value::Ab(bo) => Ok(Value::Ab(*bo)),
                Value::R(bb) => Ok(Value::Ab(*bb as i64)),
                Value::Em(o) => Ok(Value::Ab(if *o { 1 } else { 0 })),
                Value::He(e) => {
                    let bo: i64 = vcp(e.em());
                    Ok(Value::Ab(bo))
                }
                _ => Ok(Value::Ab(0)),
            }
        }
        AND_ => {
            let p = n.fv().unwrap_or(&Value::R(0.0)).dyj().unwrap_or(0.0);
            Ok(Value::R(libm::ibi(p)))
        }
        AMD_ => {
            match n.fv().unwrap_or(&Value::Ab(0)) {
                Value::Ab(bo) => Ok(Value::Ab(bo.gp())),
                Value::R(bb) => Ok(Value::R(libm::sqq(*bb))),
                _ => Ok(Value::Ab(0)),
            }
        }
        
        
        
        AMU_ => {
            
            let b = n.get(0).and_then(|p| p.abb().bq()).unwrap_or(0) as u32;
            let c = n.get(1).and_then(|p| p.abb().bq()).unwrap_or(0) as u32;
            let m = n.get(2).and_then(|p| p.abb().bq()).unwrap_or(255) as u32 & 0xFF;
            let at = n.get(3).and_then(|p| p.abb().bq()).unwrap_or(255) as u32 & 0xFF;
            let o = n.get(4).and_then(|p| p.abb().bq()).unwrap_or(255) as u32 & 0xFF;
            let s = 0xFF000000 | (m << 16) | (at << 8) | o;
            crate::framebuffer::sf(b, c, s);
            Ok(Value::Cn)
        }
        AMG_ => {
            
            let m = n.get(0).and_then(|p| p.abb().bq()).unwrap_or(0) as u32 & 0xFF;
            let at = n.get(1).and_then(|p| p.abb().bq()).unwrap_or(0) as u32 & 0xFF;
            let o = n.get(2).and_then(|p| p.abb().bq()).unwrap_or(0) as u32 & 0xFF;
            let s = 0xFF000000 | (m << 16) | (at << 8) | o;
            let (kp, kl) = crate::framebuffer::yn();
            crate::framebuffer::ah(0, 0, kp, kl, s);
            Ok(Value::Cn)
        }
        AMM_ => {
            
            let b = n.get(0).and_then(|p| p.abb().bq()).unwrap_or(0) as u32;
            let c = n.get(1).and_then(|p| p.abb().bq()).unwrap_or(0) as u32;
            let d = n.get(2).and_then(|p| p.abb().bq()).unwrap_or(0) as u32;
            let i = n.get(3).and_then(|p| p.abb().bq()).unwrap_or(0) as u32;
            let m = n.get(4).and_then(|p| p.abb().bq()).unwrap_or(255) as u32 & 0xFF;
            let at = n.get(5).and_then(|p| p.abb().bq()).unwrap_or(255) as u32 & 0xFF;
            let o = n.get(6).and_then(|p| p.abb().bq()).unwrap_or(255) as u32 & 0xFF;
            let s = 0xFF000000 | (m << 16) | (at << 8) | o;
            crate::framebuffer::ah(b, c, d, i, s);
            Ok(Value::Cn)
        }
        AMJ_ => {
            
            let fy = n.get(0).and_then(|p| p.abb().bq()).unwrap_or(0);
            let fo = n.get(1).and_then(|p| p.abb().bq()).unwrap_or(0);
            let dn = n.get(2).and_then(|p| p.abb().bq()).unwrap_or(0);
            let dp = n.get(3).and_then(|p| p.abb().bq()).unwrap_or(0);
            let m = n.get(4).and_then(|p| p.abb().bq()).unwrap_or(255) as u32 & 0xFF;
            let at = n.get(5).and_then(|p| p.abb().bq()).unwrap_or(255) as u32 & 0xFF;
            let o = n.get(6).and_then(|p| p.abb().bq()).unwrap_or(255) as u32 & 0xFF;
            let s = 0xFF000000 | (m << 16) | (at << 8) | o;
            
            let mut cx = fy;
            let mut ae = fo;
            let dx = (dn - fy).gp();
            let bg = -(dp - fo).gp();
            let cr: i64 = if fy < dn { 1 } else { -1 };
            let cq: i64 = if fo < dp { 1 } else { -1 };
            let mut rq = dx + bg;
            loop {
                if cx >= 0 && ae >= 0 {
                    crate::framebuffer::sf(cx as u32, ae as u32, s);
                }
                if cx == dn && ae == dp { break; }
                let agl = 2 * rq;
                if agl >= bg { rq += bg; cx += cr; }
                if agl <= dx { rq += dx; ae += cq; }
            }
            Ok(Value::Cn)
        }
        AMI_ => {
            
            let cx = n.get(0).and_then(|p| p.abb().bq()).unwrap_or(0);
            let ae = n.get(1).and_then(|p| p.abb().bq()).unwrap_or(0);
            let dy = n.get(2).and_then(|p| p.abb().bq()).unwrap_or(0);
            let m = n.get(3).and_then(|p| p.abb().bq()).unwrap_or(255) as u32 & 0xFF;
            let at = n.get(4).and_then(|p| p.abb().bq()).unwrap_or(255) as u32 & 0xFF;
            let o = n.get(5).and_then(|p| p.abb().bq()).unwrap_or(255) as u32 & 0xFF;
            let s = 0xFF000000 | (m << 16) | (at << 8) | o;
            
            let mut b = dy;
            let mut c: i64 = 0;
            let mut bc = 1 - dy;
            while b >= c {
                let frp = [
                    (cx + b, ae + c), (cx - b, ae + c),
                    (cx + b, ae - c), (cx - b, ae - c),
                    (cx + c, ae + b), (cx - c, ae + b),
                    (cx + c, ae - b), (cx - c, ae - b),
                ];
                for (y, x) in frp {
                    if y >= 0 && x >= 0 {
                        crate::framebuffer::sf(y as u32, x as u32, s);
                    }
                }
                c += 1;
                if bc <= 0 {
                    bc += 2 * c + 1;
                } else {
                    b -= 1;
                    bc += 2 * (c - b) + 1;
                }
            }
            Ok(Value::Cn)
        }
        ANA_ => {
            let (d, _) = crate::framebuffer::yn();
            Ok(Value::Ab(d as i64))
        }
        AMZ_ => {
            let (_, i) = crate::framebuffer::yn();
            Ok(Value::Ab(i as i64))
        }
        AMN_ => {
            
            crate::framebuffer::sv();
            Ok(Value::Cn)
        }
        AMK_ => {
            
            if let Some(Value::He(text)) = n.get(0) {
                let b = n.get(1).and_then(|p| p.abb().bq()).unwrap_or(0) as u32;
                let c = n.get(2).and_then(|p| p.abb().bq()).unwrap_or(0) as u32;
                let m = n.get(3).and_then(|p| p.abb().bq()).unwrap_or(255) as u32 & 0xFF;
                let at = n.get(4).and_then(|p| p.abb().bq()).unwrap_or(255) as u32 & 0xFF;
                let o = n.get(5).and_then(|p| p.abb().bq()).unwrap_or(255) as u32 & 0xFF;
                let bv = n.get(6).and_then(|p| p.abb().bq()).unwrap_or(1) as u32;
                let s = 0xFF000000 | (m << 16) | (at << 8) | o;
                
                let mut cx = b;
                for r in text.bw() {
                    let ka = crate::framebuffer::font::ada(r);
                    for (br, &fs) in ka.iter().cf() {
                        for ga in 0..8u32 {
                            if fs & (0x80 >> ga) != 0 {
                                for cq in 0..bv {
                                    for cr in 0..bv {
                                        crate::framebuffer::sf(
                                            cx + ga * bv + cr,
                                            c + br as u32 * bv + cq,
                                            s,
                                        );
                                    }
                                }
                            }
                        }
                    }
                    cx += 8 * bv;
                }
            }
            Ok(Value::Cn)
        }
        ANC_ => {
            
            let jn = n.get(0).and_then(|p| p.abb().bq()).unwrap_or(0) as u64;
            crate::cpu::tsc::rd(jn);
            Ok(Value::Cn)
        }
        ANF_ => {
            let p = n.fv().unwrap_or(&Value::Cn);
            match p {
                Value::R(bb) => Ok(Value::R(*bb)),
                Value::Ab(bo) => Ok(Value::R(*bo as f64)),
                Value::Em(o) => Ok(Value::R(if *o { 1.0 } else { 0.0 })),
                Value::He(e) => {
                    let bb = vcf(e.em());
                    Ok(Value::R(bb))
                }
                _ => Ok(Value::R(0.0)),
            }
        }
        AMY_ => {
            
            let line = crate::shell::cts();
            Ok(Value::He(line))
        }
        
        
        
        AMF_ => Ok(Value::R(crate::trustdaw::live_viz::tcw() as f64)),
        AME_ => Ok(Value::R(crate::trustdaw::live_viz::tcv() as f64)),
        ANE_ => Ok(Value::R(crate::trustdaw::live_viz::tet() as f64)),
        AMR_ => Ok(Value::R(crate::trustdaw::live_viz::teb() as f64)),
        AMP_ => Ok(Value::R(crate::trustdaw::live_viz::tdr() as f64)),
        ANI_ => Ok(Value::R(crate::trustdaw::live_viz::tez() as f64)),
        AML_ => Ok(Value::R(crate::trustdaw::live_viz::tdo() as f64)),
        AMO_ => Ok(Value::Ab(crate::trustdaw::live_viz::tdq() as i64)),
        ANB_ => {
            let b = n.fv().unwrap_or(&Value::R(0.0)).dyj().unwrap_or(0.0);
            Ok(Value::R(libm::ayq(b)))
        }
        AMH_ => {
            let b = n.fv().unwrap_or(&Value::R(0.0)).dyj().unwrap_or(0.0);
            Ok(Value::R(libm::cjt(b)))
        }
        AMS_ => {
            let bc = crate::desktop::Aa.lock();
            let hl = bc.lf;
            drop(bc);
            Ok(Value::Ab(hl as i64))
        }
        AMT_ => {
            let bc = crate::desktop::Aa.lock();
            let ir = bc.ot;
            drop(bc);
            Ok(Value::Ab(ir as i64))
        }
        _ => Err(format!("unknown builtin id: {}", ad)),
    }
}


fn vcp(e: &str) -> i64 {
    let mut ap: i64 = 0;
    let mut neg = false;
    for (a, bm) in e.bw().cf() {
        if a == 0 && bm == '-' { neg = true; continue; }
        if !bm.atb() { break; }
        ap = ap.hx(10).cn((bm as i64) - 48);
    }
    if neg { -ap } else { ap }
}


fn vcf(e: &str) -> f64 {
    let mut ap: f64 = 0.0;
    let mut neg = false;
    let mut avw = false;
    let mut hkh: f64 = 1.0;
    for (a, bm) in e.bw().cf() {
        if a == 0 && bm == '-' { neg = true; continue; }
        if bm == '.' && !avw { avw = true; continue; }
        if !bm.atb() { break; }
        let bc = (bm as u8 - b'0') as f64;
        if avw {
            hkh *= 10.0;
            ap += bc / hkh;
        } else {
            ap = ap * 10.0 + bc;
        }
    }
    if neg { -ap } else { ap }
}
