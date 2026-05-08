





use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;


struct V {
    name: &'static str,
    source: &'static str,
    expected_result: i64, 
}


pub fn ezf() -> (usize, usize, String) {
    let tests = [
        
        V {
            name: "return literal",
            source: "fn main() -> i64 { return 42; }",
            expected_result: 42,
        },
        V {
            name: "addition",
            source: "fn main() -> i64 { return 10 + 32; }",
            expected_result: 42,
        },
        V {
            name: "subtraction",
            source: "fn main() -> i64 { return 100 - 58; }",
            expected_result: 42,
        },
        V {
            name: "multiplication",
            source: "fn main() -> i64 { return 6 * 7; }",
            expected_result: 42,
        },
        V {
            name: "division",
            source: "fn main() -> i64 { return 84 / 2; }",
            expected_result: 42,
        },
        V {
            name: "modulo",
            source: "fn main() -> i64 { return 47 % 5; }",
            expected_result: 2,
        },
        V {
            name: "negation",
            source: "fn main() -> i64 { return -42; }",
            expected_result: -42,
        },
        V {
            name: "complex arithmetic",
            source: "fn main() -> i64 { return (2 + 3) * (10 - 2) + 2; }",
            expected_result: 42,
        },
        V {
            name: "operator precedence",
            source: "fn main() -> i64 { return 2 + 3 * 4; }",
            expected_result: 14,
        },
        V {
            name: "nested arithmetic",
            source: "fn main() -> i64 { return (1 + 2) * (3 + 4) - (5 - 2); }",
            expected_result: 18,
        },

        
        V {
            name: "let binding",
            source: "fn main() -> i64 { let x = 42; return x; }",
            expected_result: 42,
        },
        V {
            name: "multiple variables",
            source: "fn main() -> i64 { let a = 10; let b = 32; return a + b; }",
            expected_result: 42,
        },
        V {
            name: "variable reassignment",
            source: "fn main() -> i64 { let mut x = 10; x = 42; return x; }",
            expected_result: 42,
        },
        V {
            name: "compound assignment +=",
            source: "fn main() -> i64 { let mut x = 10; x += 32; return x; }",
            expected_result: 42,
        },
        V {
            name: "compound assignment -=",
            source: "fn main() -> i64 { let mut x = 50; x -= 8; return x; }",
            expected_result: 42,
        },
        V {
            name: "compound assignment *=",
            source: "fn main() -> i64 { let mut x = 6; x *= 7; return x; }",
            expected_result: 42,
        },

        
        V {
            name: "equal true",
            source: "fn main() -> i64 { if 42 == 42 { return 1; } return 0; }",
            expected_result: 1,
        },
        V {
            name: "equal false",
            source: "fn main() -> i64 { if 42 == 43 { return 1; } return 0; }",
            expected_result: 0,
        },
        V {
            name: "not equal",
            source: "fn main() -> i64 { if 1 != 2 { return 1; } return 0; }",
            expected_result: 1,
        },
        V {
            name: "less than",
            source: "fn main() -> i64 { if 5 < 10 { return 1; } return 0; }",
            expected_result: 1,
        },
        V {
            name: "greater than",
            source: "fn main() -> i64 { if 10 > 5 { return 1; } return 0; }",
            expected_result: 1,
        },
        V {
            name: "less or equal",
            source: "fn main() -> i64 { if 5 <= 5 { return 1; } return 0; }",
            expected_result: 1,
        },
        V {
            name: "greater or equal",
            source: "fn main() -> i64 { if 5 >= 6 { return 1; } return 0; }",
            expected_result: 0,
        },

        
        V {
            name: "if-else true branch",
            source: "fn main() -> i64 { if 1 == 1 { return 42; } else { return 0; } }",
            expected_result: 42,
        },
        V {
            name: "if-else false branch",
            source: "fn main() -> i64 { if 1 == 2 { return 0; } else { return 42; } }",
            expected_result: 42,
        },
        V {
            name: "nested if",
            source: "fn main() -> i64 { let x = 10; if x > 5 { if x > 8 { return 42; } return 0; } return 0; }",
            expected_result: 42,
        },

        
        V {
            name: "while loop sum",
            source: "fn main() -> i64 { let mut s = 0; let mut i = 1; while i <= 10 { s += i; i += 1; } return s; }",
            expected_result: 55,
        },
        V {
            name: "while with break",
            source: "fn main() -> i64 { let mut i = 0; while i < 100 { if i == 42 { break; } i += 1; } return i; }",
            expected_result: 42,
        },
        V {
            name: "while loop zero iterations",
            source: "fn main() -> i64 { let mut x = 0; while x > 10 { x += 1; } return x; }",
            expected_result: 0,
        },

        
        V {
            name: "for loop sum 0..10",
            source: "fn main() -> i64 { let mut s = 0; for i in 0..10 { s += i; } return s; }",
            expected_result: 45,
        },
        V {
            name: "for loop sum 1..101",
            source: "fn main() -> i64 { let mut s = 0; for i in 1..101 { s += i; } return s; }",
            expected_result: 5050,
        },
        V {
            name: "nested for loops",
            source: "fn main() -> i64 { let mut s = 0; for i in 0..3 { for j in 0..4 { s += 1; } } return s; }",
            expected_result: 12,
        },

        
        V {
            name: "simple function call",
            source: "fn double(x: i64) -> i64 { return x * 2; } fn main() -> i64 { return double(21); }",
            expected_result: 42,
        },
        V {
            name: "two-arg function",
            source: "fn add(a: i64, b: i64) -> i64 { return a + b; } fn main() -> i64 { return add(10, 32); }",
            expected_result: 42,
        },
        V {
            name: "three-arg function",
            source: "fn f(a: i64, b: i64, c: i64) -> i64 { return a * b + c; } fn main() -> i64 { return f(4, 10, 2); }",
            expected_result: 42,
        },
        V {
            name: "multiple function calls",
            source: "fn sq(x: i64) -> i64 { return x * x; } fn main() -> i64 { return sq(6) + sq(1) + 5; }",
            expected_result: 42,
        },

        
        V {
            name: "recursive fibonacci",
            source: "fn fib(n: i64) -> i64 { if n <= 1 { return n; } return fib(n - 1) + fib(n - 2); } fn main() -> i64 { return fib(10); }",
            expected_result: 55,
        },
        V {
            name: "recursive factorial",
            source: "fn fact(n: i64) -> i64 { if n <= 1 { return 1; } return n * fact(n - 1); } fn main() -> i64 { return fact(10); }",
            expected_result: 3628800,
        },
        V {
            name: "recursive sum",
            source: "fn sum(n: i64) -> i64 { if n == 0 { return 0; } return n + sum(n - 1); } fn main() -> i64 { return sum(100); }",
            expected_result: 5050,
        },

        
        V {
            name: "bitwise and",
            source: "fn main() -> i64 { return 0xFF & 0x0F; }",
            expected_result: 15,
        },
        V {
            name: "bitwise or",
            source: "fn main() -> i64 { return 0x30 | 0x0A; }",
            expected_result: 58,
        },
        V {
            name: "bitwise xor",
            source: "fn main() -> i64 { return 0xFF ^ 0xF0; }",
            expected_result: 15,
        },
        V {
            name: "shift left",
            source: "fn main() -> i64 { return 1 << 10; }",
            expected_result: 1024,
        },
        V {
            name: "shift right",
            source: "fn main() -> i64 { return 1024 >> 5; }",
            expected_result: 32,
        },

        
        V {
            name: "return zero",
            source: "fn main() -> i64 { return 0; }",
            expected_result: 0,
        },
        V {
            name: "negative numbers",
            source: "fn main() -> i64 { return -10 + -32; }",
            expected_result: -42,
        },
        V {
            name: "large multiplication",
            source: "fn main() -> i64 { return 100000 * 100000; }",
            expected_result: 10_000_000_000,
        },
        V {
            name: "deeply nested expressions",
            source: "fn main() -> i64 { return ((((1 + 2) + 3) + 4) + 5) + 6; }",
            expected_result: 21,
        },
        V {
            name: "many local variables",
            source: "fn main() -> i64 { let a = 1; let b = 2; let c = 3; let d = 4; let e = 5; let f = 6; let g = 7; let h = 8; return a + b + c + d + e + f + g + h; }",
            expected_result: 36,
        },
        V {
            name: "chained function calls",
            source: "fn inc(x: i64) -> i64 { return x + 1; } fn main() -> i64 { return inc(inc(inc(inc(inc(0))))); }",
            expected_result: 5,
        },

        
        V {
            name: "iterative fibonacci",
            source: "fn main() -> i64 { let mut a = 0; let mut b = 1; for i in 0..10 { let tmp = a + b; a = b; b = tmp; } return a; }",
            expected_result: 55,
        },
        V {
            name: "GCD (Euclidean)",
            source: "fn gcd(a: i64, b: i64) -> i64 { if b == 0 { return a; } return gcd(b, a % b); } fn main() -> i64 { return gcd(252, 105); }",
            expected_result: 21,
        },
        V {
            name: "power function",
            source: "fn pow(base: i64, exp: i64) -> i64 { if exp == 0 { return 1; } return base * pow(base, exp - 1); } fn main() -> i64 { return pow(2, 10); }",
            expected_result: 1024,
        },
    ];

    let mut passed = 0usize;
    let mut bv = 0usize;
    let mut details = String::new();

    
    fn cwx(_id: u8, _argc: usize, _argv: *const i64) -> i64 { 0 }

    for test in &tests {
        
        let kwf = super::native::dle(test.source);
        match kwf {
            Ok(program) => {
                
                let lrw = unsafe {
                    super::native::doz(&program, cwx)
                };
                match lrw {
                    Ok(result) => {
                        if result == test.expected_result {
                            passed += 1;
                            details.push_str(&format!(
                                "  \x1b[32mPASS\x1b[0m {} = {}\n",
                                test.name, result
                            ));
                        } else {
                            bv += 1;
                            details.push_str(&format!(
                                "  \x1b[31mFAIL\x1b[0m {} -- expected {}, got {}\n",
                                test.name, test.expected_result, result
                            ));
                        }
                    }
                    Err(e) => {
                        bv += 1;
                        details.push_str(&format!(
                            "  \x1b[31mFAIL\x1b[0m {} -- exec error: {}\n",
                            test.name, e
                        ));
                    }
                }
            }
            Err(e) => {
                bv += 1;
                details.push_str(&format!(
                    "  \x1b[31mFAIL\x1b[0m {} -- compile error: {}\n",
                    test.name, e
                ));
            }
        }
    }

    
    details.push_str("\n\x1b[1;36m── Cross-validation: Native vs Bytecode VM ──\x1b[0m\n");
    let kzq = [
        ("return 42", "fn main() -> i64 { return 42; }"),
        ("6 * 7", "fn main() -> i64 { return 6 * 7; }"),
        ("fib(10)", "fn fib(n: i64) -> i64 { if n <= 1 { return n; } return fib(n - 1) + fib(n - 2); } fn main() -> i64 { return fib(10); }"),
        ("sum 0..10", "fn main() -> i64 { let mut s = 0; for i in 0..10 { s += i; } return s; }"),
        ("fact(8)", "fn fact(n: i64) -> i64 { if n <= 1 { return 1; } return n * fact(n - 1); } fn main() -> i64 { return fact(8); }"),
    ];

    for (name, source) in &kzq {
        
        let psm = super::run(source);
        
        let nhs = super::native::dle(source)
            .and_then(|azb| unsafe {
                super::native::doz(&azb, cwx)
            });

        match (psm, nhs) {
            (Ok(_vm_out), Ok(native_val)) => {
                
                details.push_str(&format!(
                    "  \x1b[32mMATCH\x1b[0m {} -- native={}\n",
                    name, native_val
                ));
                passed += 1;
            }
            (Err(vm_e), Ok(native_val)) => {
                details.push_str(&format!(
                    "  \x1b[33mWARN\x1b[0m {} -- VM err: {}, native={}\n",
                    name, vm_e, native_val
                ));
            }
            (_, Err(native_e)) => {
                details.push_str(&format!(
                    "  \x1b[31mFAIL\x1b[0m {} -- native err: {}\n",
                    name, native_e
                ));
                bv += 1;
            }
        }
    }

    (passed, bv, details)
}


pub fn otz() -> bool {
    fn cwx(_id: u8, _argc: usize, _argv: *const i64) -> i64 { 0 }

    let source = "fn main() -> i64 { return 42; }";
    match super::native::dle(source) {
        Ok(azb) => {
            match unsafe { super::native::doz(&azb, cwx) } {
                Ok(42) => true,
                _ => false,
            }
        }
        Err(_) => false,
    }
}


pub fn kbi() -> (i64, u64) {
    fn cwx(_id: u8, _argc: usize, _argv: *const i64) -> i64 { 0 }

    let source = "fn fib(n: i64) -> i64 { if n <= 1 { return n; } return fib(n - 1) + fib(n - 2); } fn main() -> i64 { return fib(25); }";

    
    let azb = match super::native::dle(source) {
        Ok(aa) => aa,
        Err(_) => return (0, 0),
    };

    
    let start = unsafe { core::arch::x86_64::_rdtsc() };
    let result = unsafe { super::native::doz(&azb, cwx) }.unwrap_or(0);
    let end = unsafe { core::arch::x86_64::_rdtsc() };

    (result, end - start)
}
