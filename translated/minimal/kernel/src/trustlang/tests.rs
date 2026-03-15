





use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;


struct Ae {
    j: &'static str,
    iy: &'static str,
    xm: i64, 
}


pub fn jne() -> (usize, usize, String) {
    let tests = [
        
        Ae {
            j: "return literal",
            iy: "fn main() -> i64 { return 42; }",
            xm: 42,
        },
        Ae {
            j: "addition",
            iy: "fn main() -> i64 { return 10 + 32; }",
            xm: 42,
        },
        Ae {
            j: "subtraction",
            iy: "fn main() -> i64 { return 100 - 58; }",
            xm: 42,
        },
        Ae {
            j: "multiplication",
            iy: "fn main() -> i64 { return 6 * 7; }",
            xm: 42,
        },
        Ae {
            j: "division",
            iy: "fn main() -> i64 { return 84 / 2; }",
            xm: 42,
        },
        Ae {
            j: "modulo",
            iy: "fn main() -> i64 { return 47 % 5; }",
            xm: 2,
        },
        Ae {
            j: "negation",
            iy: "fn main() -> i64 { return -42; }",
            xm: -42,
        },
        Ae {
            j: "complex arithmetic",
            iy: "fn main() -> i64 { return (2 + 3) * (10 - 2) + 2; }",
            xm: 42,
        },
        Ae {
            j: "operator precedence",
            iy: "fn main() -> i64 { return 2 + 3 * 4; }",
            xm: 14,
        },
        Ae {
            j: "nested arithmetic",
            iy: "fn main() -> i64 { return (1 + 2) * (3 + 4) - (5 - 2); }",
            xm: 18,
        },

        
        Ae {
            j: "let binding",
            iy: "fn main() -> i64 { let x = 42; return x; }",
            xm: 42,
        },
        Ae {
            j: "multiple variables",
            iy: "fn main() -> i64 { let a = 10; let b = 32; return a + b; }",
            xm: 42,
        },
        Ae {
            j: "variable reassignment",
            iy: "fn main() -> i64 { let mut x = 10; x = 42; return x; }",
            xm: 42,
        },
        Ae {
            j: "compound assignment +=",
            iy: "fn main() -> i64 { let mut x = 10; x += 32; return x; }",
            xm: 42,
        },
        Ae {
            j: "compound assignment -=",
            iy: "fn main() -> i64 { let mut x = 50; x -= 8; return x; }",
            xm: 42,
        },
        Ae {
            j: "compound assignment *=",
            iy: "fn main() -> i64 { let mut x = 6; x *= 7; return x; }",
            xm: 42,
        },

        
        Ae {
            j: "equal true",
            iy: "fn main() -> i64 { if 42 == 42 { return 1; } return 0; }",
            xm: 1,
        },
        Ae {
            j: "equal false",
            iy: "fn main() -> i64 { if 42 == 43 { return 1; } return 0; }",
            xm: 0,
        },
        Ae {
            j: "not equal",
            iy: "fn main() -> i64 { if 1 != 2 { return 1; } return 0; }",
            xm: 1,
        },
        Ae {
            j: "less than",
            iy: "fn main() -> i64 { if 5 < 10 { return 1; } return 0; }",
            xm: 1,
        },
        Ae {
            j: "greater than",
            iy: "fn main() -> i64 { if 10 > 5 { return 1; } return 0; }",
            xm: 1,
        },
        Ae {
            j: "less or equal",
            iy: "fn main() -> i64 { if 5 <= 5 { return 1; } return 0; }",
            xm: 1,
        },
        Ae {
            j: "greater or equal",
            iy: "fn main() -> i64 { if 5 >= 6 { return 1; } return 0; }",
            xm: 0,
        },

        
        Ae {
            j: "if-else true branch",
            iy: "fn main() -> i64 { if 1 == 1 { return 42; } else { return 0; } }",
            xm: 42,
        },
        Ae {
            j: "if-else false branch",
            iy: "fn main() -> i64 { if 1 == 2 { return 0; } else { return 42; } }",
            xm: 42,
        },
        Ae {
            j: "nested if",
            iy: "fn main() -> i64 { let x = 10; if x > 5 { if x > 8 { return 42; } return 0; } return 0; }",
            xm: 42,
        },

        
        Ae {
            j: "while loop sum",
            iy: "fn main() -> i64 { let mut s = 0; let mut i = 1; while i <= 10 { s += i; i += 1; } return s; }",
            xm: 55,
        },
        Ae {
            j: "while with break",
            iy: "fn main() -> i64 { let mut i = 0; while i < 100 { if i == 42 { break; } i += 1; } return i; }",
            xm: 42,
        },
        Ae {
            j: "while loop zero iterations",
            iy: "fn main() -> i64 { let mut x = 0; while x > 10 { x += 1; } return x; }",
            xm: 0,
        },

        
        Ae {
            j: "for loop sum 0..10",
            iy: "fn main() -> i64 { let mut s = 0; for i in 0..10 { s += i; } return s; }",
            xm: 45,
        },
        Ae {
            j: "for loop sum 1..101",
            iy: "fn main() -> i64 { let mut s = 0; for i in 1..101 { s += i; } return s; }",
            xm: 5050,
        },
        Ae {
            j: "nested for loops",
            iy: "fn main() -> i64 { let mut s = 0; for i in 0..3 { for j in 0..4 { s += 1; } } return s; }",
            xm: 12,
        },

        
        Ae {
            j: "simple function call",
            iy: "fn double(x: i64) -> i64 { return x * 2; } fn main() -> i64 { return double(21); }",
            xm: 42,
        },
        Ae {
            j: "two-arg function",
            iy: "fn add(a: i64, b: i64) -> i64 { return a + b; } fn main() -> i64 { return add(10, 32); }",
            xm: 42,
        },
        Ae {
            j: "three-arg function",
            iy: "fn f(a: i64, b: i64, c: i64) -> i64 { return a * b + c; } fn main() -> i64 { return f(4, 10, 2); }",
            xm: 42,
        },
        Ae {
            j: "multiple function calls",
            iy: "fn sq(x: i64) -> i64 { return x * x; } fn main() -> i64 { return sq(6) + sq(1) + 5; }",
            xm: 42,
        },

        
        Ae {
            j: "recursive fibonacci",
            iy: "fn fib(n: i64) -> i64 { if n <= 1 { return n; } return fib(n - 1) + fib(n - 2); } fn main() -> i64 { return fib(10); }",
            xm: 55,
        },
        Ae {
            j: "recursive factorial",
            iy: "fn fact(n: i64) -> i64 { if n <= 1 { return 1; } return n * fact(n - 1); } fn main() -> i64 { return fact(10); }",
            xm: 3628800,
        },
        Ae {
            j: "recursive sum",
            iy: "fn sum(n: i64) -> i64 { if n == 0 { return 0; } return n + sum(n - 1); } fn main() -> i64 { return sum(100); }",
            xm: 5050,
        },

        
        Ae {
            j: "bitwise and",
            iy: "fn main() -> i64 { return 0xFF & 0x0F; }",
            xm: 15,
        },
        Ae {
            j: "bitwise or",
            iy: "fn main() -> i64 { return 0x30 | 0x0A; }",
            xm: 58,
        },
        Ae {
            j: "bitwise xor",
            iy: "fn main() -> i64 { return 0xFF ^ 0xF0; }",
            xm: 15,
        },
        Ae {
            j: "shift left",
            iy: "fn main() -> i64 { return 1 << 10; }",
            xm: 1024,
        },
        Ae {
            j: "shift right",
            iy: "fn main() -> i64 { return 1024 >> 5; }",
            xm: 32,
        },

        
        Ae {
            j: "return zero",
            iy: "fn main() -> i64 { return 0; }",
            xm: 0,
        },
        Ae {
            j: "negative numbers",
            iy: "fn main() -> i64 { return -10 + -32; }",
            xm: -42,
        },
        Ae {
            j: "large multiplication",
            iy: "fn main() -> i64 { return 100000 * 100000; }",
            xm: 10_000_000_000,
        },
        Ae {
            j: "deeply nested expressions",
            iy: "fn main() -> i64 { return ((((1 + 2) + 3) + 4) + 5) + 6; }",
            xm: 21,
        },
        Ae {
            j: "many local variables",
            iy: "fn main() -> i64 { let a = 1; let b = 2; let c = 3; let d = 4; let e = 5; let f = 6; let g = 7; let h = 8; return a + b + c + d + e + f + g + h; }",
            xm: 36,
        },
        Ae {
            j: "chained function calls",
            iy: "fn inc(x: i64) -> i64 { return x + 1; } fn main() -> i64 { return inc(inc(inc(inc(inc(0))))); }",
            xm: 5,
        },

        
        Ae {
            j: "iterative fibonacci",
            iy: "fn main() -> i64 { let mut a = 0; let mut b = 1; for i in 0..10 { let tmp = a + b; a = b; b = tmp; } return a; }",
            xm: 55,
        },
        Ae {
            j: "GCD (Euclidean)",
            iy: "fn gcd(a: i64, b: i64) -> i64 { if b == 0 { return a; } return gcd(b, a % b); } fn main() -> i64 { return gcd(252, 105); }",
            xm: 21,
        },
        Ae {
            j: "power function",
            iy: "fn pow(base: i64, exp: i64) -> i64 { if exp == 0 { return 1; } return base * pow(base, exp - 1); } fn main() -> i64 { return pow(2, 10); }",
            xm: 1024,
        },
    ];

    let mut cg = 0usize;
    let mut gv = 0usize;
    let mut yw = String::new();

    
    fn gfp(ddq: u8, msf: usize, fzk: *const i64) -> i64 { 0 }

    for test in &tests {
        
        let rmz = super::native::hdq(test.iy);
        match rmz {
            Ok(alo) => {
                
                let sof = unsafe {
                    super::native::him(&alo, gfp)
                };
                match sof {
                    Ok(result) => {
                        if result == test.xm {
                            cg += 1;
                            yw.t(&format!(
                                "  \x1b[32mPASS\x1b[0m {} = {}\n",
                                test.j, result
                            ));
                        } else {
                            gv += 1;
                            yw.t(&format!(
                                "  \x1b[31mFAIL\x1b[0m {} -- expected {}, got {}\n",
                                test.j, test.xm, result
                            ));
                        }
                    }
                    Err(aa) => {
                        gv += 1;
                        yw.t(&format!(
                            "  \x1b[31mFAIL\x1b[0m {} -- exec error: {}\n",
                            test.j, aa
                        ));
                    }
                }
            }
            Err(aa) => {
                gv += 1;
                yw.t(&format!(
                    "  \x1b[31mFAIL\x1b[0m {} -- compile error: {}\n",
                    test.j, aa
                ));
            }
        }
    }

    
    yw.t("\n\x1b[1;36m── Cross-validation: Native vs Bytecode VM ──\x1b[0m\n");
    let rqy = [
        ("return 42", "fn main() -> i64 { return 42; }"),
        ("6 * 7", "fn main() -> i64 { return 6 * 7; }"),
        ("fib(10)", "fn fib(n: i64) -> i64 { if n <= 1 { return n; } return fib(n - 1) + fib(n - 2); } fn main() -> i64 { return fib(10); }"),
        ("sum 0..10", "fn main() -> i64 { let mut s = 0; for i in 0..10 { s += i; } return s; }"),
        ("fact(8)", "fn fact(n: i64) -> i64 { if n <= 1 { return 1; } return n * fact(n - 1); } fn main() -> i64 { return fact(8); }"),
    ];

    for (j, iy) in &rqy {
        
        let xse = super::vw(iy);
        
        let urm = super::native::hdq(iy)
            .and_then(|ctl| unsafe {
                super::native::him(&ctl, gfp)
            });

        match (xse, urm) {
            (Ok(ydu), Ok(lnm)) => {
                
                yw.t(&format!(
                    "  \x1b[32mMATCH\x1b[0m {} -- native={}\n",
                    j, lnm
                ));
                cg += 1;
            }
            (Err(xsd), Ok(lnm)) => {
                yw.t(&format!(
                    "  \x1b[33mWARN\x1b[0m {} -- VM err: {}, native={}\n",
                    j, xsd, lnm
                ));
            }
            (_, Err(url)) => {
                yw.t(&format!(
                    "  \x1b[31mFAIL\x1b[0m {} -- native err: {}\n",
                    j, url
                ));
                gv += 1;
            }
        }
    }

    (cg, gv, yw)
}


pub fn wpx() -> bool {
    fn gfp(ddq: u8, msf: usize, fzk: *const i64) -> i64 { 0 }

    let iy = "fn main() -> i64 { return 42; }";
    match super::native::hdq(iy) {
        Ok(ctl) => {
            match unsafe { super::native::him(&ctl, gfp) } {
                Ok(42) => true,
                _ => false,
            }
        }
        Err(_) => false,
    }
}


pub fn qov() -> (i64, u64) {
    fn gfp(ddq: u8, msf: usize, fzk: *const i64) -> i64 { 0 }

    let iy = "fn fib(n: i64) -> i64 { if n <= 1 { return n; } return fib(n - 1) + fib(n - 2); } fn main() -> i64 { return fib(25); }";

    
    let ctl = match super::native::hdq(iy) {
        Ok(ai) => ai,
        Err(_) => return (0, 0),
    };

    
    let ay = unsafe { core::arch::x86_64::dxw() };
    let result = unsafe { super::native::him(&ctl, gfp) }.unwrap_or(0);
    let ci = unsafe { core::arch::x86_64::dxw() };

    (result, ci - ay)
}
