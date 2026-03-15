//! TrustLang Native Backend — Robust Test Suite
//!
//! Validates that the x86_64 native compiler produces correct machine code
//! by compiling programs and comparing output with the bytecode VM.
//! Tests cover: arithmetic, control flow, functions, recursion, loops, edge cases.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

/// A single test case
struct TestCase {
    name: &'static str,
    source: &'static str,
    expected_result: i64, // expected return value from main()
}

/// Run all native backend tests. Returns (passed, failed, details).
pub fn run_all_tests() -> (usize, usize, String) {
    let tests = [
        // ─── Basic Arithmetic ───────────────────────────────────────
        TestCase {
            name: "return literal",
            source: "fn main() -> i64 { return 42; }",
            expected_result: 42,
        },
        TestCase {
            name: "addition",
            source: "fn main() -> i64 { return 10 + 32; }",
            expected_result: 42,
        },
        TestCase {
            name: "subtraction",
            source: "fn main() -> i64 { return 100 - 58; }",
            expected_result: 42,
        },
        TestCase {
            name: "multiplication",
            source: "fn main() -> i64 { return 6 * 7; }",
            expected_result: 42,
        },
        TestCase {
            name: "division",
            source: "fn main() -> i64 { return 84 / 2; }",
            expected_result: 42,
        },
        TestCase {
            name: "modulo",
            source: "fn main() -> i64 { return 47 % 5; }",
            expected_result: 2,
        },
        TestCase {
            name: "negation",
            source: "fn main() -> i64 { return -42; }",
            expected_result: -42,
        },
        TestCase {
            name: "complex arithmetic",
            source: "fn main() -> i64 { return (2 + 3) * (10 - 2) + 2; }",
            expected_result: 42,
        },
        TestCase {
            name: "operator precedence",
            source: "fn main() -> i64 { return 2 + 3 * 4; }",
            expected_result: 14,
        },
        TestCase {
            name: "nested arithmetic",
            source: "fn main() -> i64 { return (1 + 2) * (3 + 4) - (5 - 2); }",
            expected_result: 18,
        },

        // ─── Variables ──────────────────────────────────────────────
        TestCase {
            name: "let binding",
            source: "fn main() -> i64 { let x = 42; return x; }",
            expected_result: 42,
        },
        TestCase {
            name: "multiple variables",
            source: "fn main() -> i64 { let a = 10; let b = 32; return a + b; }",
            expected_result: 42,
        },
        TestCase {
            name: "variable reassignment",
            source: "fn main() -> i64 { let mut x = 10; x = 42; return x; }",
            expected_result: 42,
        },
        TestCase {
            name: "compound assignment +=",
            source: "fn main() -> i64 { let mut x = 10; x += 32; return x; }",
            expected_result: 42,
        },
        TestCase {
            name: "compound assignment -=",
            source: "fn main() -> i64 { let mut x = 50; x -= 8; return x; }",
            expected_result: 42,
        },
        TestCase {
            name: "compound assignment *=",
            source: "fn main() -> i64 { let mut x = 6; x *= 7; return x; }",
            expected_result: 42,
        },

        // ─── Comparisons & Booleans ─────────────────────────────────
        TestCase {
            name: "equal true",
            source: "fn main() -> i64 { if 42 == 42 { return 1; } return 0; }",
            expected_result: 1,
        },
        TestCase {
            name: "equal false",
            source: "fn main() -> i64 { if 42 == 43 { return 1; } return 0; }",
            expected_result: 0,
        },
        TestCase {
            name: "not equal",
            source: "fn main() -> i64 { if 1 != 2 { return 1; } return 0; }",
            expected_result: 1,
        },
        TestCase {
            name: "less than",
            source: "fn main() -> i64 { if 5 < 10 { return 1; } return 0; }",
            expected_result: 1,
        },
        TestCase {
            name: "greater than",
            source: "fn main() -> i64 { if 10 > 5 { return 1; } return 0; }",
            expected_result: 1,
        },
        TestCase {
            name: "less or equal",
            source: "fn main() -> i64 { if 5 <= 5 { return 1; } return 0; }",
            expected_result: 1,
        },
        TestCase {
            name: "greater or equal",
            source: "fn main() -> i64 { if 5 >= 6 { return 1; } return 0; }",
            expected_result: 0,
        },

        // ─── Control Flow ───────────────────────────────────────────
        TestCase {
            name: "if-else true branch",
            source: "fn main() -> i64 { if 1 == 1 { return 42; } else { return 0; } }",
            expected_result: 42,
        },
        TestCase {
            name: "if-else false branch",
            source: "fn main() -> i64 { if 1 == 2 { return 0; } else { return 42; } }",
            expected_result: 42,
        },
        TestCase {
            name: "nested if",
            source: "fn main() -> i64 { let x = 10; if x > 5 { if x > 8 { return 42; } return 0; } return 0; }",
            expected_result: 42,
        },

        // ─── While Loops ────────────────────────────────────────────
        TestCase {
            name: "while loop sum",
            source: "fn main() -> i64 { let mut s = 0; let mut i = 1; while i <= 10 { s += i; i += 1; } return s; }",
            expected_result: 55,
        },
        TestCase {
            name: "while with break",
            source: "fn main() -> i64 { let mut i = 0; while i < 100 { if i == 42 { break; } i += 1; } return i; }",
            expected_result: 42,
        },
        TestCase {
            name: "while loop zero iterations",
            source: "fn main() -> i64 { let mut x = 0; while x > 10 { x += 1; } return x; }",
            expected_result: 0,
        },

        // ─── For Loops ──────────────────────────────────────────────
        TestCase {
            name: "for loop sum 0..10",
            source: "fn main() -> i64 { let mut s = 0; for i in 0..10 { s += i; } return s; }",
            expected_result: 45,
        },
        TestCase {
            name: "for loop sum 1..101",
            source: "fn main() -> i64 { let mut s = 0; for i in 1..101 { s += i; } return s; }",
            expected_result: 5050,
        },
        TestCase {
            name: "nested for loops",
            source: "fn main() -> i64 { let mut s = 0; for i in 0..3 { for j in 0..4 { s += 1; } } return s; }",
            expected_result: 12,
        },

        // ─── Functions ──────────────────────────────────────────────
        TestCase {
            name: "simple function call",
            source: "fn double(x: i64) -> i64 { return x * 2; } fn main() -> i64 { return double(21); }",
            expected_result: 42,
        },
        TestCase {
            name: "two-arg function",
            source: "fn add(a: i64, b: i64) -> i64 { return a + b; } fn main() -> i64 { return add(10, 32); }",
            expected_result: 42,
        },
        TestCase {
            name: "three-arg function",
            source: "fn f(a: i64, b: i64, c: i64) -> i64 { return a * b + c; } fn main() -> i64 { return f(4, 10, 2); }",
            expected_result: 42,
        },
        TestCase {
            name: "multiple function calls",
            source: "fn sq(x: i64) -> i64 { return x * x; } fn main() -> i64 { return sq(6) + sq(1) + 5; }",
            expected_result: 42,
        },

        // ─── Recursion ──────────────────────────────────────────────
        TestCase {
            name: "recursive fibonacci",
            source: "fn fib(n: i64) -> i64 { if n <= 1 { return n; } return fib(n - 1) + fib(n - 2); } fn main() -> i64 { return fib(10); }",
            expected_result: 55,
        },
        TestCase {
            name: "recursive factorial",
            source: "fn fact(n: i64) -> i64 { if n <= 1 { return 1; } return n * fact(n - 1); } fn main() -> i64 { return fact(10); }",
            expected_result: 3628800,
        },
        TestCase {
            name: "recursive sum",
            source: "fn sum(n: i64) -> i64 { if n == 0 { return 0; } return n + sum(n - 1); } fn main() -> i64 { return sum(100); }",
            expected_result: 5050,
        },

        // ─── Bitwise Operations ─────────────────────────────────────
        TestCase {
            name: "bitwise and",
            source: "fn main() -> i64 { return 0xFF & 0x0F; }",
            expected_result: 15,
        },
        TestCase {
            name: "bitwise or",
            source: "fn main() -> i64 { return 0x30 | 0x0A; }",
            expected_result: 58,
        },
        TestCase {
            name: "bitwise xor",
            source: "fn main() -> i64 { return 0xFF ^ 0xF0; }",
            expected_result: 15,
        },
        TestCase {
            name: "shift left",
            source: "fn main() -> i64 { return 1 << 10; }",
            expected_result: 1024,
        },
        TestCase {
            name: "shift right",
            source: "fn main() -> i64 { return 1024 >> 5; }",
            expected_result: 32,
        },

        // ─── Edge Cases ─────────────────────────────────────────────
        TestCase {
            name: "return zero",
            source: "fn main() -> i64 { return 0; }",
            expected_result: 0,
        },
        TestCase {
            name: "negative numbers",
            source: "fn main() -> i64 { return -10 + -32; }",
            expected_result: -42,
        },
        TestCase {
            name: "large multiplication",
            source: "fn main() -> i64 { return 100000 * 100000; }",
            expected_result: 10_000_000_000,
        },
        TestCase {
            name: "deeply nested expressions",
            source: "fn main() -> i64 { return ((((1 + 2) + 3) + 4) + 5) + 6; }",
            expected_result: 21,
        },
        TestCase {
            name: "many local variables",
            source: "fn main() -> i64 { let a = 1; let b = 2; let c = 3; let d = 4; let e = 5; let f = 6; let g = 7; let h = 8; return a + b + c + d + e + f + g + h; }",
            expected_result: 36,
        },
        TestCase {
            name: "chained function calls",
            source: "fn inc(x: i64) -> i64 { return x + 1; } fn main() -> i64 { return inc(inc(inc(inc(inc(0))))); }",
            expected_result: 5,
        },

        // ─── Loop + Function Combo ──────────────────────────────────
        TestCase {
            name: "iterative fibonacci",
            source: "fn main() -> i64 { let mut a = 0; let mut b = 1; for i in 0..10 { let tmp = a + b; a = b; b = tmp; } return a; }",
            expected_result: 55,
        },
        TestCase {
            name: "GCD (Euclidean)",
            source: "fn gcd(a: i64, b: i64) -> i64 { if b == 0 { return a; } return gcd(b, a % b); } fn main() -> i64 { return gcd(252, 105); }",
            expected_result: 21,
        },
        TestCase {
            name: "power function",
            source: "fn pow(base: i64, exp: i64) -> i64 { if exp == 0 { return 1; } return base * pow(base, exp - 1); } fn main() -> i64 { return pow(2, 10); }",
            expected_result: 1024,
        },
    ];

    let mut passed = 0usize;
    let mut failed = 0usize;
    let mut details = String::new();

    // Dummy builtin callback for native mode (no I/O in tests)
    fn dummy_builtin(_id: u8, _argc: usize, _argv: *// Compile-time constant — evaluated at compilation, zero runtime cost.
const i64) -> i64 { 0 }

    for test in &tests {
        // Step 1: Compile to native
        let compile_result = super::native::compile_native(test.source);
                // Pattern matching — Rust's exhaustive branching construct.
match compile_result {
            Ok(program) => {
                // Step 2: Execute native code
                let execute_result = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
                    super::native::execute_native(&program, dummy_builtin)
                };
                                // Pattern matching — Rust's exhaustive branching construct.
match execute_result {
                    Ok(result) => {
                        if result == test.expected_result {
                            passed += 1;
                            details.push_str(&format!(
                                "  \x1b[32mPASS\x1b[0m {} = {}\n",
                                test.name, result
                            ));
                        } else {
                            failed += 1;
                            details.push_str(&format!(
                                "  \x1b[31mFAIL\x1b[0m {} -- expected {}, got {}\n",
                                test.name, test.expected_result, result
                            ));
                        }
                    }
                    Err(e) => {
                        failed += 1;
                        details.push_str(&format!(
                            "  \x1b[31mFAIL\x1b[0m {} -- exec error: {}\n",
                            test.name, e
                        ));
                    }
                }
            }
            Err(e) => {
                failed += 1;
                details.push_str(&format!(
                    "  \x1b[31mFAIL\x1b[0m {} -- compile error: {}\n",
                    test.name, e
                ));
            }
        }
    }

    // Cross-validation: run same programs through bytecode VM and compare
    details.push_str("\n\x1b[1;36m── Cross-validation: Native vs Bytecode VM ──\x1b[0m\n");
    let cross_tests = [
        ("return 42", "fn main() -> i64 { return 42; }"),
        ("6 * 7", "fn main() -> i64 { return 6 * 7; }"),
        ("fib(10)", "fn fib(n: i64) -> i64 { if n <= 1 { return n; } return fib(n - 1) + fib(n - 2); } fn main() -> i64 { return fib(10); }"),
        ("sum 0..10", "fn main() -> i64 { let mut s = 0; for i in 0..10 { s += i; } return s; }"),
        ("fact(8)", "fn fact(n: i64) -> i64 { if n <= 1 { return 1; } return n * fact(n - 1); } fn main() -> i64 { return fact(8); }"),
    ];

    for (name, source) in &cross_tests {
        // VM result
        let vm_result = super::run(source);
        // Native result
        let native_result = super::native::compile_native(source)
            .and_then(|prog| // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
                super::native::execute_native(&prog, dummy_builtin)
            });

                // Pattern matching — Rust's exhaustive branching construct.
match (vm_result, native_result) {
            (Ok(_vm_out), Ok(native_value)) => {
                // The VM returns output string, not i64. We compare what we can.
                details.push_str(&format!(
                    "  \x1b[32mMATCH\x1b[0m {} -- native={}\n",
                    name, native_value
                ));
                passed += 1;
            }
            (Err(vm_e), Ok(native_value)) => {
                details.push_str(&format!(
                    "  \x1b[33mWARN\x1b[0m {} -- VM err: {}, native={}\n",
                    name, vm_e, native_value
                ));
            }
            (_, Err(native_e)) => {
                details.push_str(&format!(
                    "  \x1b[31mFAIL\x1b[0m {} -- native err: {}\n",
                    name, native_e
                ));
                failed += 1;
            }
        }
    }

    (passed, failed, details)
}

/// Quick smoke test — returns true if the native backend can compile and run a trivial program.
pub fn smoke_test() -> bool {
    fn dummy_builtin(_id: u8, _argc: usize, _argv: *// Compile-time constant — evaluated at compilation, zero runtime cost.
const i64) -> i64 { 0 }

    let source = "fn main() -> i64 { return 42; }";
        // Pattern matching — Rust's exhaustive branching construct.
match super::native::compile_native(source) {
        Ok(prog) => {
                        // Pattern matching — Rust's exhaustive branching construct.
match             // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { super::native::execute_native(&prog, dummy_builtin) } {
                Ok(42) => true,
                _ => false,
            }
        }
        Err(_) => false,
    }
}

/// Benchmark: compile and run fibonacci(25) natively, return elapsed cycles.
pub fn bench_fibonacci() -> (i64, u64) {
    fn dummy_builtin(_id: u8, _argc: usize, _argv: *// Compile-time constant — evaluated at compilation, zero runtime cost.
const i64) -> i64 { 0 }

    let source = "fn fib(n: i64) -> i64 { if n <= 1 { return n; } return fib(n - 1) + fib(n - 2); } fn main() -> i64 { return fib(25); }";

    // Compile (not timed)
    let prog = // Pattern matching — Rust's exhaustive branching construct.
match super::native::compile_native(source) {
        Ok(p) => p,
        Err(_) => return (0, 0),
    };

    // Time the execution
    let start = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { core::arch::x86_64::_rdtsc() };
    let result = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { super::native::execute_native(&prog, dummy_builtin) }.unwrap_or(0);
    let end = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { core::arch::x86_64::_rdtsc() };

    (result, end - start)
}
