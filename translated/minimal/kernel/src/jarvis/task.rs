












use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};



struct Rng(u64);

impl Rng {
    fn new() -> Self {
        let dv = crate::time::lc().hx(6364136223846793005).cn(1);
        Self(if dv == 0 { 42 } else { dv })
    }

    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn cmb(&mut self, v: u64, am: u64) -> u64 {
        v + (self.next() % (am - v + 1))
    }
}


#[derive(Debug, Clone)]
pub struct Avi {
    
    pub xz: String,
    
    pub qy: i64,
    
    pub dah: [u8; 4],
    
    pub evj: String,
}


#[derive(Debug, Clone)]
pub struct Zc {
    pub dah: [u8; 4],
    pub evj: String,
    pub xz: String,
    pub qy: i64,
    pub ecf: i64,
    pub dzv: bool,
    pub ggz: bool,
}


#[derive(Debug, Clone, Copy)]
enum MathOp { Add, Sub, Mul }


fn nxm(rng: &mut Rng, zdr: usize, dah: [u8; 4]) -> Avi {
    let op = match rng.next() % 3 {
        0 => MathOp::Add,
        1 => MathOp::Sub,
        _ => MathOp::Mul,
    };

    let (q, o, expr, result) = match op {
        MathOp::Add => {
            let q = rng.cmb(100, 9999) as i64;
            let o = rng.cmb(100, 9999) as i64;
            (q, o, format!("{} + {}", q, o), q + o)
        }
        MathOp::Sub => {
            let q = rng.cmb(1000, 9999) as i64;
            let o = rng.cmb(100, q as u64) as i64;
            (q, o, format!("{} - {}", q, o), q - o)
        }
        MathOp::Mul => {
            let q = rng.cmb(10, 999) as i64;
            let o = rng.cmb(2, 99) as i64;
            (q, o, format!("{} * {}", q, o), q * o)
        }
    };

    let j = format!("node_{}.{}.{}.{}", dah[0], dah[1], dah[2], dah[3]);

    Avi {
        xz: expr,
        qy: result,
        dah,
        evj: j,
    }
}



fn vdz(ew: &[u8]) -> Option<&str> {
    let e = core::str::jg(ew).bq()?;
    e.blj("TASK_MATH:")
}


fn itb(expr: &str) -> Option<i64> {
    
    if let Some((fd, hw)) = expr.fve(" + ") {
        let q: i64 = fd.em().parse().bq()?;
        let o: i64 = hw.em().parse().bq()?;
        return Some(q + o);
    }
    
    if let Some((fd, hw)) = expr.fve(" - ") {
        let q: i64 = fd.em().parse().bq()?;
        let o: i64 = hw.em().parse().bq()?;
        return Some(q - o);
    }
    
    if let Some((fd, hw)) = expr.fve(" * ") {
        let q: i64 = fd.em().parse().bq()?;
        let o: i64 = hw.em().parse().bq()?;
        return Some(q * o);
    }
    None
}


pub fn tlg(ew: &[u8]) -> (super::rpc::Status, Vec<u8>) {
    let expr = match vdz(ew) {
        Some(aa) => aa,
        None => return (super::rpc::Status::Q, b"Invalid task payload".ip()),
    };

    
    let yt = match itb(expr) {
        Some(q) => q,
        None => return (super::rpc::Status::Q, b"Cannot evaluate expression".ip()),
    };

    
    let it = format!("/task_result.txt");
    let ca = format!("expression: {}\nanswer: {}\nnode: local\n", expr, yt);

    let eqf = crate::ramfs::fh(|fs| {
        let _ = fs.touch(&it);
        fs.ns(&it, ca.as_bytes())
    }).is_ok();

    crate::serial_println!("[TASK] Computed: {} = {} (file={})", expr, yt, eqf);

    
    let mk = format!("RESULT:{}:{}", yt, if eqf { "FILE_OK" } else { "FILE_ERR" });
    (super::rpc::Status::Ok, mk.cfq())
}


fn vea(f: &[u8]) -> Option<(i64, bool)> {
    let e = core::str::jg(f).bq()?;
    let e = e.blj("RESULT:")?;
    let (qiw, status) = e.fve(':')?;
    let yt: i64 = qiw.parse().bq()?;
    let eqf = status == "FILE_OK";
    Some((yt, eqf))
}







pub fn wbf() -> Vec<Zc> {
    let mut hd = Vec::new();
    let mut rng = Rng::new();

    
    let yp = super::mesh::dhn();
    let bsb = super::mesh::GV_;

    let xkl = yp.len() + 1; 
    crate::serial_println!("[TASK] Distributing math tasks to {} nodes ({} peers + self)",
        xkl, yp.len());

    
    let mut bcy: Vec<Avi> = Vec::new();

    
    let aro = crate::network::aou()
        .map(|(ip, _, _)| *ip.as_bytes())
        .unwrap_or([127, 0, 0, 1]);
    bcy.push(nxm(&mut rng, 0, aro));

    
    for (a, ko) in yp.iter().cf() {
        bcy.push(nxm(&mut rng, a + 1, ko.ip));
    }

    
    {
        let task = &bcy[0];
        let yt = itb(&task.xz).unwrap_or(0);
        let it = "/task_result.txt";
        let ca = format!("expression: {}\nanswer: {}\nnode: self\n", task.xz, yt);
        let eqf = crate::ramfs::fh(|fs| {
            let _ = fs.touch(it);
            fs.ns(it, ca.as_bytes())
        }).is_ok();

        hd.push(Zc {
            dah: aro,
            evj: format!("self ({}.{}.{}.{})", aro[0], aro[1], aro[2], aro[3]),
            xz: task.xz.clone(),
            qy: task.qy,
            ecf: yt,
            dzv: yt == task.qy,
            ggz: eqf,
        });
    }

    
    for (a, ko) in yp.iter().cf() {
        let task = &bcy[a + 1];
        let ew = format!("TASK_MATH:{}", task.xz);

        crate::serial_println!("[TASK] Sending to {}.{}.{}.{}: {}",
            ko.ip[0], ko.ip[1], ko.ip[2], ko.ip[3], task.xz);

        match super::rpc::bto(ko.ip, bsb, super::rpc::Command::Azs, ew.as_bytes()) {
            Ok((super::rpc::Status::Ok, lj)) => {
                if let Some((yt, eqf)) = vea(&lj) {
                    hd.push(Zc {
                        dah: ko.ip,
                        evj: format!("{}.{}.{}.{} ({})",
                            ko.ip[0], ko.ip[1], ko.ip[2], ko.ip[3],
                            ko.arch.j()),
                        xz: task.xz.clone(),
                        qy: task.qy,
                        ecf: yt,
                        dzv: yt == task.qy,
                        ggz: eqf,
                    });
                } else {
                    hd.push(Zc {
                        dah: ko.ip,
                        evj: format!("{}.{}.{}.{}", ko.ip[0], ko.ip[1], ko.ip[2], ko.ip[3]),
                        xz: task.xz.clone(),
                        qy: task.qy,
                        ecf: 0,
                        dzv: false,
                        ggz: false,
                    });
                }
            }
            Ok((status, lj)) => {
                let rq = core::str::jg(&lj).unwrap_or("?");
                crate::serial_println!("[TASK] Peer error: {:?} — {}", status, rq);
                hd.push(Zc {
                    dah: ko.ip,
                    evj: format!("{}.{}.{}.{}", ko.ip[0], ko.ip[1], ko.ip[2], ko.ip[3]),
                    xz: task.xz.clone(),
                    qy: task.qy,
                    ecf: 0,
                    dzv: false,
                    ggz: false,
                });
            }
            Err(aa) => {
                crate::serial_println!("[TASK] RPC failed to {}.{}.{}.{}: {}",
                    ko.ip[0], ko.ip[1], ko.ip[2], ko.ip[3], aa);
                hd.push(Zc {
                    dah: ko.ip,
                    evj: format!("{}.{}.{}.{}", ko.ip[0], ko.ip[1], ko.ip[2], ko.ip[3]),
                    xz: task.xz.clone(),
                    qy: task.qy,
                    ecf: 0,
                    dzv: false,
                    ggz: false,
                });
            }
        }
    }

    hd
}
