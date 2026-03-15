//! Distributed Task Engine — JARVIS Cluster Task Orchestration
//!
//! Enables JARVIS to distribute computational tasks across the mesh cluster,
//! collect results, and verify correctness. Each node receives a unique task
//! (e.g. a math problem), computes it locally, writes the result to a file,
//! and reports back via RPC.
//!
//! # Example Usage (from shell)
//! ```
//! jarvis brain task math        — distribute unique math problems to each node
//! jarvis brain task verify      — verify all nodes returned correct answers
//! ```

use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

/// Simple pseudo-random number generator (xorshift64)
/// Seeded from uptime to get different values each run
struct Rng(u64);

// Implementation block — defines methods for the type above.
impl Rng {
    fn new() -> Self {
        let seed = crate::time::uptime_mouse().wrapping_mul(6364136223846793005).wrapping_add(1);
        Self(if seed == 0 { 42 } else { seed })
    }

    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn range(&mut self, minimum: u64, maximum: u64) -> u64 {
        minimum + (self.next() % (maximum - minimum + 1))
    }
}

/// A math task sent to a node
#[derive(Debug, Clone)]
// Public structure — visible outside this module.
pub struct MathTask {
    /// Human-readable expression (e.g. "142 + 857")
    pub expression: String,
    /// The correct answer
    pub expected: i64,
    /// Target node IP
    pub node_ip: [u8; 4],
    /// Node hostname for display
    pub node_name: String,
}

/// Result from a node
#[derive(Debug, Clone)]
// Public structure — visible outside this module.
pub struct TaskResult {
    pub node_ip: [u8; 4],
    pub node_name: String,
    pub expression: String,
    pub expected: i64,
    pub got: i64,
    pub correct: bool,
    pub file_written: bool,
}

/// Operation types for math tasks
#[derive(Debug, Clone, Copy)]
enum MathOp { Add, Sub, Mul }

/// Generate a unique math task for a given node index
fn generate_math_task(rng: &mut Rng, node_index: usize, node_ip: [u8; 4]) -> MathTask {
    let op = // Pattern matching — Rust's exhaustive branching construct.
match rng.next() % 3 {
        0 => MathOp::Add,
        1 => MathOp::Sub,
        _ => MathOp::Mul,
    };

    let (a, b, expr, result) = // Pattern matching — Rust's exhaustive branching construct.
match op {
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

    MathTask {
        expression: expr,
        expected: result,
        node_ip,
        node_name: name,
    }
}

/// Parse a task payload: "TASK_MATH:<expression>\n"
/// Returns the expression string
fn parse_task_payload(payload: &[u8]) -> Option<&str> {
    let s = core::str::from_utf8(payload).ok()?;
    s.strip_prefix("TASK_MATH:")
}

/// Evaluate a simple math expression (a op b) where op is +, -, *
fn eval_expression(expr: &str) -> Option<i64> {
    // Try "a + b"
    if let Some((left, right)) = expr.split_once(" + ") {
        let a: i64 = left.trim().parse().ok()?;
        let b: i64 = right.trim().parse().ok()?;
        return Some(a + b);
    }
    // Try "a - b"
    if let Some((left, right)) = expr.split_once(" - ") {
        let a: i64 = left.trim().parse().ok()?;
        let b: i64 = right.trim().parse().ok()?;
        return Some(a - b);
    }
    // Try "a * b"
    if let Some((left, right)) = expr.split_once(" * ") {
        let a: i64 = left.trim().parse().ok()?;
        let b: i64 = right.trim().parse().ok()?;
        return Some(a * b);
    }
    None
}

/// Handle incoming TaskExecute RPC — compute the math and write result to file
pub fn handle_task_execute(payload: &[u8]) -> (super::rpc::Status, Vec<u8>) {
    let expr = // Pattern matching — Rust's exhaustive branching construct.
match parse_task_payload(payload) {
        Some(e) => e,
        None => return (super::rpc::Status::Error, b"Invalid task payload".to_vec()),
    };

    // Evaluate the expression
    let answer = // Pattern matching — Rust's exhaustive branching construct.
match eval_expression(expr) {
        Some(a) => a,
        None => return (super::rpc::Status::Error, b"Cannot evaluate expression".to_vec()),
    };

    // Write result to a file in ramfs
    let filename = format!("/task_result.txt");
    let content = format!("expression: {}\nanswer: {}\nnode: local\n", expr, answer);

    let file_ok = crate::ramfs::with_filesystem(|fs| {
        let _ = fs.touch(&filename);
        fs.write_file(&filename, content.as_bytes())
    }).is_ok();

    crate::serial_println!("[TASK] Computed: {} = {} (file={})", expr, answer, file_ok);

    // Return answer as response
    let response = format!("RESULT:{}:{}", answer, if file_ok { "FILE_OK" } else { "FILE_ERR" });
    (super::rpc::Status::Ok, response.into_bytes())
}

/// Parse a task result response: "RESULT:<answer>:<status>"
fn parse_task_response(data: &[u8]) -> Option<(i64, bool)> {
    let s = core::str::from_utf8(data).ok()?;
    let s = s.strip_prefix("RESULT:")?;
    let (answer_str, status) = s.split_once(':')?;
    let answer: i64 = answer_str.parse().ok()?;
    let file_ok = status == "FILE_OK";
    Some((answer, file_ok))
}

/// Run the distributed math verification task across the cluster
///
/// 1. Generate unique math problems for each peer + ourselves
/// 2. Send tasks via RPC to peers
/// 3. Compute our own task locally
/// 4. Collect and verify all results
pub fn run_distributed_math() -> Vec<TaskResult> {
    let mut results = Vec::new();
    let mut rng = Rng::new();

    // Get all alive mesh peers
    let peers = super::mesh::get_peers();
    let rpc_port = super::mesh::MESH_RPC_PORT;

    let total_nodes = peers.len() + 1; // peers + us
    crate::serial_println!("[TASK] Distributing math tasks to {} nodes ({} peers + self)",
        total_nodes, peers.len());

    // Generate tasks for all nodes
    let mut tasks: Vec<MathTask> = Vec::new();

    // Our own task (node 0)
    let our_ip = crate::network::get_ipv4_config()
        .map(|(ip, _, _)| *ip.as_bytes())
        .unwrap_or([127, 0, 0, 1]);
    tasks.push(generate_math_task(&mut rng, 0, our_ip));

    // Tasks for each peer
    for (i, peer) in peers.iter().enumerate() {
        tasks.push(generate_math_task(&mut rng, i + 1, peer.ip));
    }

    // Execute our own task locally (node 0)
    {
        let task = &tasks[0];
        let answer = eval_expression(&task.expression).unwrap_or(0);
        let filename = "/task_result.txt";
        let content = format!("expression: {}\nanswer: {}\nnode: self\n", task.expression, answer);
        let file_ok = crate::ramfs::with_filesystem(|fs| {
            let _ = fs.touch(filename);
            fs.write_file(filename, content.as_bytes())
        }).is_ok();

        results.push(TaskResult {
            node_ip: our_ip,
            node_name: format!("self ({}.{}.{}.{})", our_ip[0], our_ip[1], our_ip[2], our_ip[3]),
            expression: task.expression.clone(),
            expected: task.expected,
            got: answer,
            correct: answer == task.expected,
            file_written: file_ok,
        });
    }

    // Send tasks to peers via RPC
    for (i, peer) in peers.iter().enumerate() {
        let task = &tasks[i + 1];
        let payload = format!("TASK_MATH:{}", task.expression);

        crate::serial_println!("[TASK] Sending to {}.{}.{}.{}: {}",
            peer.ip[0], peer.ip[1], peer.ip[2], peer.ip[3], task.expression);

                // Pattern matching — Rust's exhaustive branching construct.
match super::rpc::call(peer.ip, rpc_port, super::rpc::Command::TaskExecute, payload.as_bytes()) {
            Ok((super::rpc::Status::Ok, response)) => {
                if let Some((answer, file_ok)) = parse_task_response(&response) {
                    results.push(TaskResult {
                        node_ip: peer.ip,
                        node_name: format!("{}.{}.{}.{} ({})",
                            peer.ip[0], peer.ip[1], peer.ip[2], peer.ip[3],
                            peer.arch.name()),
                        expression: task.expression.clone(),
                        expected: task.expected,
                        got: answer,
                        correct: answer == task.expected,
                        file_written: file_ok,
                    });
                } else {
                    results.push(TaskResult {
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
            Ok((status, response)) => {
                let error = core::str::from_utf8(&response).unwrap_or("?");
                crate::serial_println!("[TASK] Peer error: {:?} — {}", status, error);
                results.push(TaskResult {
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
                results.push(TaskResult {
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
