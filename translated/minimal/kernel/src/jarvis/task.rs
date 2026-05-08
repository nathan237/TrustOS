












use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};



struct Rng(u64);

impl Rng {
    fn new() -> Self {
        let seed = crate::time::uptime_ms().wrapping_mul(6364136223846793005).wrapping_add(1);
        Self(if seed == 0 { 42 } else { seed })
    }

    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn range(&mut self, min: u64, max: u64) -> u64 {
        min + (self.next() % (max - min + 1))
    }
}


#[derive(Debug, Clone)]
pub struct Tq {
    
    pub expression: String,
    
    pub expected: i64,
    
    pub node_ip: [u8; 4],
    
    pub node_name: String,
}


#[derive(Debug, Clone)]
pub struct Ku {
    pub node_ip: [u8; 4],
    pub node_name: String,
    pub expression: String,
    pub expected: i64,
    pub got: i64,
    pub correct: bool,
    pub file_written: bool,
}


#[derive(Debug, Clone, Copy)]
enum MathOp { Add, Sub, Mul }


fn ibb(rng: &mut Rng, node_idx: usize, node_ip: [u8; 4]) -> Tq {
    let op = match rng.next() % 3 {
        0 => MathOp::Add,
        1 => MathOp::Sub,
        _ => MathOp::Mul,
    };

    let (a, b, expr, result) = match op {
        MathOp::Add => {
            let a = rng.range(100, 9999) as i64;
            let b = rng.range(100, 9999) as i64;
            (a, b, format!("{} + {}", a, b), a + b)
        }
        MathOp::Sub => {
            let a = rng.range(1000, 9999) as i64;
            let b = rng.range(100, a as u64) as i64;
            (a, b, format!("{} - {}", a, b), a - b)
        }
        MathOp::Mul => {
            let a = rng.range(10, 999) as i64;
            let b = rng.range(2, 99) as i64;
            (a, b, format!("{} * {}", a, b), a * b)
        }
    };

    let name = format!("node_{}.{}.{}.{}", node_ip[0], node_ip[1], node_ip[2], node_ip[3]);

    Tq {
        expression: expr,
        expected: result,
        node_ip,
        node_name: name,
    }
}



fn nrh(payload: &[u8]) -> Option<&str> {
    let j = core::str::from_utf8(payload).ok()?;
    j.strip_prefix("TASK_MATH:")
}


fn elp(expr: &str) -> Option<i64> {
    
    if let Some((left, right)) = expr.split_once(" + ") {
        let a: i64 = left.trim().parse().ok()?;
        let b: i64 = right.trim().parse().ok()?;
        return Some(a + b);
    }
    
    if let Some((left, right)) = expr.split_once(" - ") {
        let a: i64 = left.trim().parse().ok()?;
        let b: i64 = right.trim().parse().ok()?;
        return Some(a - b);
    }
    
    if let Some((left, right)) = expr.split_once(" * ") {
        let a: i64 = left.trim().parse().ok()?;
        let b: i64 = right.trim().parse().ok()?;
        return Some(a * b);
    }
    None
}


pub fn mir(payload: &[u8]) -> (super::rpc::Status, Vec<u8>) {
    let expr = match nrh(payload) {
        Some(e) => e,
        None => return (super::rpc::Status::Error, b"Invalid task payload".to_vec()),
    };

    
    let answer = match elp(expr) {
        Some(a) => a,
        None => return (super::rpc::Status::Error, b"Cannot evaluate expression".to_vec()),
    };

    
    let filename = format!("/task_result.txt");
    let content = format!("expression: {}\nanswer: {}\nnode: local\n", expr, answer);

    let bzq = crate::ramfs::bh(|fs| {
        let _ = fs.touch(&filename);
        fs.write_file(&filename, content.as_bytes())
    }).is_ok();

    crate::serial_println!("[TASK] Computed: {} = {} (file={})", expr, answer, bzq);

    
    let fa = format!("RESULT:{}:{}", answer, if bzq { "FILE_OK" } else { "FILE_ERR" });
    (super::rpc::Status::Ok, fa.into_bytes())
}


fn nri(data: &[u8]) -> Option<(i64, bool)> {
    let j = core::str::from_utf8(data).ok()?;
    let j = j.strip_prefix("RESULT:")?;
    let (answer_str, status) = j.split_once(':')?;
    let answer: i64 = answer_str.parse().ok()?;
    let bzq = status == "FILE_OK";
    Some((answer, bzq))
}







pub fn oja() -> Vec<Ku> {
    let mut results = Vec::new();
    let mut rng = Rng::new();

    
    let lj = super::mesh::bgo();
    let rpc_port = super::mesh::HM_;

    let pmc = lj.len() + 1; 
    crate::serial_println!("[TASK] Distributing math tasks to {} nodes ({} peers + self)",
        pmc, lj.len());

    
    let mut tasks: Vec<Tq> = Vec::new();

    
    let wj = crate::network::rd()
        .map(|(ip, _, _)| *ip.as_bytes())
        .unwrap_or([127, 0, 0, 1]);
    tasks.push(ibb(&mut rng, 0, wj));

    
    for (i, peer) in lj.iter().enumerate() {
        tasks.push(ibb(&mut rng, i + 1, peer.ip));
    }

    
    {
        let task = &tasks[0];
        let answer = elp(&task.expression).unwrap_or(0);
        let filename = "/task_result.txt";
        let content = format!("expression: {}\nanswer: {}\nnode: self\n", task.expression, answer);
        let bzq = crate::ramfs::bh(|fs| {
            let _ = fs.touch(filename);
            fs.write_file(filename, content.as_bytes())
        }).is_ok();

        results.push(Ku {
            node_ip: wj,
            node_name: format!("self ({}.{}.{}.{})", wj[0], wj[1], wj[2], wj[3]),
            expression: task.expression.clone(),
            expected: task.expected,
            got: answer,
            correct: answer == task.expected,
            file_written: bzq,
        });
    }

    
    for (i, peer) in lj.iter().enumerate() {
        let task = &tasks[i + 1];
        let payload = format!("TASK_MATH:{}", task.expression);

        crate::serial_println!("[TASK] Sending to {}.{}.{}.{}: {}",
            peer.ip[0], peer.ip[1], peer.ip[2], peer.ip[3], task.expression);

        match super::rpc::alb(peer.ip, rpc_port, super::rpc::Command::TaskExecute, payload.as_bytes()) {
            Ok((super::rpc::Status::Ok, eo)) => {
                if let Some((answer, bzq)) = nri(&eo) {
                    results.push(Ku {
                        node_ip: peer.ip,
                        node_name: format!("{}.{}.{}.{} ({})",
                            peer.ip[0], peer.ip[1], peer.ip[2], peer.ip[3],
                            peer.arch.name()),
                        expression: task.expression.clone(),
                        expected: task.expected,
                        got: answer,
                        correct: answer == task.expected,
                        file_written: bzq,
                    });
                } else {
                    results.push(Ku {
                        node_ip: peer.ip,
                        node_name: format!("{}.{}.{}.{}", peer.ip[0], peer.ip[1], peer.ip[2], peer.ip[3]),
                        expression: task.expression.clone(),
                        expected: task.expected,
                        got: 0,
                        correct: false,
                        file_written: false,
                    });
                }
            }
            Ok((status, eo)) => {
                let err = core::str::from_utf8(&eo).unwrap_or("?");
                crate::serial_println!("[TASK] Peer error: {:?} — {}", status, err);
                results.push(Ku {
                    node_ip: peer.ip,
                    node_name: format!("{}.{}.{}.{}", peer.ip[0], peer.ip[1], peer.ip[2], peer.ip[3]),
                    expression: task.expression.clone(),
                    expected: task.expected,
                    got: 0,
                    correct: false,
                    file_written: false,
                });
            }
            Err(e) => {
                crate::serial_println!("[TASK] RPC failed to {}.{}.{}.{}: {}",
                    peer.ip[0], peer.ip[1], peer.ip[2], peer.ip[3], e);
                results.push(Ku {
                    node_ip: peer.ip,
                    node_name: format!("{}.{}.{}.{}", peer.ip[0], peer.ip[1], peer.ip[2], peer.ip[3]),
                    expression: task.expression.clone(),
                    expected: task.expected,
                    got: 0,
                    correct: false,
                    file_written: false,
                });
            }
        }
    }

    results
}
