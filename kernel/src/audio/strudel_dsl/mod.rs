//! TrustStrudel chained-DSL parser & evaluator.
//!
//! Produces a `Pattern` (with global `Controls`) from a Strudel-like
//! method-chaining source string such as:
//!
//! ```text
//! n("0 4 7 12").scale("g:minor").s("sawtooth").lpf(800).lpenv(4).gain(0.7)
//! ```
//!
//! See `docs/TRUSTSTRUDEL_FEATURE_PARITY.md` for the full feature-parity
//! roadmap.

pub mod lexer;
pub mod parser;
pub mod scales;
pub mod eval;

pub use eval::{Controls, Bundle, parse_eval, evaluate};
pub use parser::{parse, Expr};
