//! Byte-level tokenizer for Jarvis
//!
//! The simplest possible tokenizer: each byte (0..255) is a token.
//! No vocabulary, no BPE merges, no training needed.
//! Works on any text in any language (UTF-8 bytes are valid tokens).
//!
//! Advantages:
//! - Zero-cost encoding/decoding (identity function)
//! - Handles any language, emoji, binary data
//! - No OOV (out-of-vocabulary) tokens ever
//!
//! Disadvantages:
//! - Each character = 1-4 tokens (UTF-8)
//! - Model must learn byte-level patterns
//!
//! This is exactly what Meta's LLaMA 3 does (byte-level BPE).
//! We start without BPE merges for simplicity; add them later.

use alloc::string::String;
use alloc::vec::Vec;

/// Special tokens
pub const PAD_TOKEN: u8 = 0x00;
pub const BOS_TOKEN: u8 = 0x01;  // Beginning of sequence
pub const EOS_TOKEN: u8 = 0x02;  // End of sequence
pub const SEP_TOKEN: u8 = 0x03;  // Separator (user/assistant boundary)

/// Encode a string into tokens (bytes)
pub fn encode(text: &str) -> Vec<u8> {
    let mut tokens = Vec::with_capacity(text.len() + 2);
    tokens.push(BOS_TOKEN);
    for byte in text.bytes() {
        tokens.push(byte);
    }
    tokens.push(EOS_TOKEN);
    tokens
}

/// Encode raw bytes without BOS/EOS (for training sequences)
pub fn encode_raw(text: &str) -> Vec<u8> {
    text.bytes().collect()
}

/// Decode tokens back to a string (lossy — invalid UTF-8 becomes '?')
pub fn decode(tokens: &[u8]) -> String {
    let mut s = String::with_capacity(tokens.len());
    for &t in tokens {
        match t {
            PAD_TOKEN => {}      // Skip padding
            BOS_TOKEN => {}      // Skip BOS
            EOS_TOKEN => break,  // Stop at EOS
            SEP_TOKEN => s.push('\n'),
            0x20..=0x7E => s.push(t as char),  // Printable ASCII
            0x0A => s.push('\n'),               // Newline
            0x09 => s.push('\t'),               // Tab
            0x0D => {}                           // Skip CR
            _ => {
                // Try to reconstruct UTF-8 or show as-is
                s.push(t as char);
            }
        }
    }
    s
}

/// Get token count for a string (without BOS/EOS)
pub fn token_count(text: &str) -> usize {
    text.len() // Byte-level: 1 byte = 1 token
}

/// Check if a token is printable
pub fn is_printable(token: u8) -> bool {
    matches!(token, 0x20..=0x7E | 0x0A | 0x09)
}

/// Format a token for display
pub fn token_name(token: u8) -> &'static str {
    match token {
        0x00 => "<PAD>",
        0x01 => "<BOS>",
        0x02 => "<EOS>",
        0x03 => "<SEP>",
        0x0A => "<NL>",
        0x0D => "<CR>",
        0x09 => "<TAB>",
        0x20 => "<SP>",
        _ => "",  // Caller should use the char itself
    }
}
