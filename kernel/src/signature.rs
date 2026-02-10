// =============================================================================
// TrustOS Kernel Signature System
// =============================================================================
//
// Cryptographic proof-of-authorship embedded in the kernel binary.
//
// HOW IT WORKS:
// - The CREATOR signature is a HMAC-SHA256 computed with a SECRET SEED
//   known only to Nated0ge. The seed never appears in the binary — only
//   the resulting hash is stored here. Nobody can forge it.
// - Users can ADD their own signature on top (co-signing), but the
//   creator's original fingerprint is always visible and immutable.
// - `signature verify` shows both the creator proof and any user signature.
//
// SECURITY MODEL:
// - Creator seed is NOT in source code — only the output hash is compiled in.
// - HMAC-SHA256(seed, payload) = fingerprint. Without the seed, the hash
//   cannot be reproduced, proving only the original creator could have
//   generated it.
// =============================================================================

use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;

// =============================================================================
// CREATOR SIGNATURE — IMMUTABLE, HARDCODED
// =============================================================================
// This is the HMAC-SHA256 fingerprint produced by the creator's secret seed.
// The seed itself is NEVER stored in the binary. Only the creator can reproduce
// this hash by knowing the original seed.
//
// Payload signed: "TrustOS Kernel — Created by Nated0ge — All rights reserved"
// Algorithm: HMAC-SHA256(secret_seed, payload)
//
// Anyone can VERIFY this hash exists. Nobody can FORGE a new one without the seed.

/// The signed payload — this is the message that was signed
pub const CREATOR_SIGNED_PAYLOAD: &str = 
    "TrustOS Kernel — Created by Nated0ge (nathan237) — Sole author and originator — All rights reserved 2025-2026";

/// The creator's identity
pub const CREATOR_NAME: &str = "Nated0ge";
pub const CREATOR_GITHUB: &str = "nathan237";

/// HMAC-SHA256 fingerprint — produced with the creator's private seed.
/// This is the ONLY artifact of the seed that exists in the binary.
/// To verify authenticity, the creator can re-derive this hash with their seed.
///
/// GENERATION (done offline by the creator, seed kept private):
///   let seed = b"<CREATOR_SECRET_SEED — NEVER COMMITTED>";
///   let hash = hmac_sha256(seed, CREATOR_SIGNED_PAYLOAD.as_bytes());
///   // → CREATOR_FINGERPRINT below
pub const CREATOR_FINGERPRINT: [u8; 32] = [
    0x0c, 0x1a, 0x99, 0xfb, 0x1e, 0x87, 0x77, 0xce,
    0x12, 0x0c, 0xca, 0x83, 0x4e, 0x75, 0x60, 0x8e,
    0x95, 0xa4, 0xb6, 0xc5, 0xd3, 0x04, 0x7a, 0x92,
    0xa1, 0xfe, 0x10, 0xb3, 0x10, 0xb8, 0x7c, 0xbd,
];

/// Build timestamp embedded at compile time
pub const BUILD_TIMESTAMP: &str = env!("TRUSTOS_BUILD_TIME", "unknown");

/// Kernel version
pub const KERNEL_VERSION: &str = "0.1.2";

// =============================================================================
// USER SIGNATURE (runtime, optional)
// =============================================================================

/// A user-generated co-signature (added at runtime via `signature sign`)
pub struct UserSignature {
    pub name: String,
    pub fingerprint: [u8; 32],
    pub timestamp: u64, // RTC ticks at signing time
}

/// Global user signature slot — only one active user signature at a time
static USER_SIGNATURE: Mutex<Option<UserSignature>> = Mutex::new(None);

// =============================================================================
// PUBLIC API
// =============================================================================

/// Format a 32-byte hash as a hex string
pub fn hash_to_hex(hash: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for &b in hash.iter() {
        let hi = b >> 4;
        let lo = b & 0x0F;
        s.push(if hi < 10 { (b'0' + hi) as char } else { (b'a' + hi - 10) as char });
        s.push(if lo < 10 { (b'0' + lo) as char } else { (b'a' + lo - 10) as char });
    }
    s
}

/// Get the creator's fingerprint as a hex string
pub fn creator_fingerprint_hex() -> String {
    hash_to_hex(&CREATOR_FINGERPRINT)
}

/// Verify the creator signature by re-computing HMAC with a provided seed.
/// Returns true if the seed produces the same fingerprint.
/// Only the real creator knows the seed that returns true.
pub fn verify_creator_seed(seed: &[u8]) -> bool {
    let computed = crate::tls13::crypto::hmac_sha256(seed, CREATOR_SIGNED_PAYLOAD.as_bytes());
    // Constant-time comparison to prevent timing attacks
    let mut diff = 0u8;
    for i in 0..32 {
        diff |= computed[i] ^ CREATOR_FINGERPRINT[i];
    }
    diff == 0
}

/// Sign the kernel with a user identity.
/// The user provides their name and a personal passphrase (seed).
/// The HMAC-SHA256 of (seed, payload) becomes their co-signature.
pub fn sign_as_user(name: &str, passphrase: &[u8]) {
    // Build the user's signing payload (includes their name for binding)
    let mut payload = Vec::new();
    payload.extend_from_slice(b"TrustOS User Signature: ");
    payload.extend_from_slice(name.as_bytes());
    payload.extend_from_slice(b" -- co-signed kernel v");
    payload.extend_from_slice(KERNEL_VERSION.as_bytes());

    let fingerprint = crate::tls13::crypto::hmac_sha256(passphrase, &payload);

    let ts = crate::rtc::get_time_seconds() as u64;

    let sig = UserSignature {
        name: String::from(name),
        fingerprint,
        timestamp: ts,
    };

    let mut slot = USER_SIGNATURE.lock();
    *slot = Some(sig);
}

/// Get the current user signature (if any)
pub fn get_user_signature() -> Option<(String, String, u64)> {
    let slot = USER_SIGNATURE.lock();
    slot.as_ref().map(|s| (s.name.clone(), hash_to_hex(&s.fingerprint), s.timestamp))
}

/// Verify a user's signature by re-computing with their passphrase
pub fn verify_user_seed(name: &str, passphrase: &[u8]) -> bool {
    let mut payload = Vec::new();
    payload.extend_from_slice(b"TrustOS User Signature: ");
    payload.extend_from_slice(name.as_bytes());
    payload.extend_from_slice(b" -- co-signed kernel v");
    payload.extend_from_slice(KERNEL_VERSION.as_bytes());

    let computed = crate::tls13::crypto::hmac_sha256(passphrase, &payload);

    let slot = USER_SIGNATURE.lock();
    if let Some(ref sig) = *slot {
        if sig.name != name {
            return false;
        }
        let mut diff = 0u8;
        for i in 0..32 {
            diff |= computed[i] ^ sig.fingerprint[i];
        }
        diff == 0
    } else {
        false
    }
}

/// Clear the user signature
pub fn clear_user_signature() {
    let mut slot = USER_SIGNATURE.lock();
    *slot = None;
}

// =============================================================================
// KERNEL INTEGRITY VERIFICATION
// =============================================================================
// Computes SHA-256 of the kernel .text section at boot and stores it.
// The `verify-integrity` command re-computes the hash and compares it,
// detecting runtime code modification (rootkits, memory corruption, etc.).

extern "C" {
    static __text_start: u8;
    static __text_end: u8;
}

/// The SHA-256 hash of the .text section computed at boot
static BOOT_TEXT_HASH: Mutex<Option<[u8; 32]>> = Mutex::new(None);

/// Compute SHA-256 of the kernel .text section
fn hash_text_section() -> [u8; 32] {
    let start = unsafe { &__text_start as *const u8 as usize };
    let end = unsafe { &__text_end as *const u8 as usize };
    let size = end.saturating_sub(start);
    
    // Read the .text section as a byte slice
    let text_bytes = unsafe {
        core::slice::from_raw_parts(start as *const u8, size)
    };
    
    crate::tls13::crypto::sha256(text_bytes)
}

/// Called once at boot to record the reference hash of the .text section.
/// Must be called after heap init but before any self-modifying code.
pub fn init_integrity() {
    let hash = hash_text_section();
    let hex = hash_to_hex(&hash);
    crate::serial_println!("[INTEGRITY] .text section: {} bytes, SHA-256: {}...{}", 
        text_section_size(), &hex[..16], &hex[56..]);
    *BOOT_TEXT_HASH.lock() = Some(hash);
}

/// Get the size of the .text section
pub fn text_section_size() -> usize {
    let start = unsafe { &__text_start as *const u8 as usize };
    let end = unsafe { &__text_end as *const u8 as usize };
    end.saturating_sub(start)
}

/// Get the boot-time hash as hex (for display)
pub fn boot_text_hash_hex() -> Option<String> {
    BOOT_TEXT_HASH.lock().map(|h| hash_to_hex(&h))
}

/// Verify kernel integrity: re-hash .text and compare to boot-time hash.
/// Returns Ok(true) if matching, Ok(false) if tampered, Err if not initialized.
pub fn verify_integrity() -> Result<bool, &'static str> {
    let boot_hash = BOOT_TEXT_HASH.lock().ok_or("Integrity not initialized (call init_integrity at boot)")?;
    let current_hash = hash_text_section();
    
    // Constant-time comparison
    let mut diff = 0u8;
    for i in 0..32 {
        diff |= boot_hash[i] ^ current_hash[i];
    }
    
    Ok(diff == 0)
}

/// Get full integrity report
pub fn integrity_report() -> Vec<String> {
    let mut lines = Vec::new();
    let size = text_section_size();
    
    lines.push(String::from("  Kernel Integrity Verification"));
    lines.push(String::from("  ─────────────────────────────────────────────"));
    lines.push(alloc::format!("  .text section : {} bytes ({} KB)", size, size / 1024));
    
    if let Some(hex) = boot_text_hash_hex() {
        lines.push(alloc::format!("  Boot hash     : {}", hex));
    } else {
        lines.push(String::from("  Boot hash     : NOT INITIALIZED"));
        return lines;
    }
    
    let current_hash = hash_text_section();
    let current_hex = hash_to_hex(&current_hash);
    lines.push(alloc::format!("  Current hash  : {}", current_hex));
    
    match verify_integrity() {
        Ok(true) => {
            lines.push(String::from("  Status        : ✅ INTEGRITY OK — .text unmodified"));
        }
        Ok(false) => {
            lines.push(String::from("  Status        : ❌ INTEGRITY VIOLATION — .text was modified!"));
            lines.push(String::from("  WARNING: Kernel code has been tampered with since boot."));
        }
        Err(e) => {
            lines.push(alloc::format!("  Status        : ⚠️  {}", e));
        }
    }
    
    lines
}
