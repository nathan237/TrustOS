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
// Computes SHA-256 of the kernel .text AND .rodata sections at boot.
// The `verify-integrity` command re-computes the hashes and compares them,
// detecting runtime code or data modification (rootkits, ROP gadgets, etc.).
//
// SECURITY MODEL:
// - .text hash detects code injection / instruction patching
// - .rodata hash detects constant/vtable/string tampering
// - Combined kernel digest = SHA-256(.text_hash || .rodata_hash) ties both
//   together into a single unforgeable measurement, similar to TPM PCR extend.
// - The creator signature payload includes the kernel version, binding the
//   HMAC fingerprint to a specific release. Rebuilding with different code
//   changes the integrity hashes, making substitution detectable.
//
// THREAT MODEL (see GitHub issue #2):
// - Current scheme uses HMAC-SHA256 (symmetric). A future version should
//   migrate to an asymmetric scheme (Ed25519 or SLH-DSA) so that signature
//   verification does not require knowledge of the signing key.
// - The integrity hashes are computed at boot and compared at runtime,
//   which detects post-boot tampering. Pre-boot tampering (replacing the
//   entire kernel binary) requires Secure Boot or a hardware root-of-trust.

extern "C" {
    static __text_start: u8;
    static __text_end: u8;
    static __rodata_start: u8;
    static __rodata_end: u8;
}

/// Boot-time SHA-256 of the .text section
static BOOT_TEXT_HASH: Mutex<Option<[u8; 32]>> = Mutex::new(None);

/// Boot-time SHA-256 of the .rodata section
static BOOT_RODATA_HASH: Mutex<Option<[u8; 32]>> = Mutex::new(None);

/// Combined kernel digest: SHA-256(text_hash || rodata_hash)
static BOOT_KERNEL_DIGEST: Mutex<Option<[u8; 32]>> = Mutex::new(None);

/// Compute SHA-256 of the kernel .text section
fn hash_text_section() -> [u8; 32] {
    let start = unsafe { &__text_start as *const u8 as usize };
    let end = unsafe { &__text_end as *const u8 as usize };
    let size = end.saturating_sub(start);
    
    let text_bytes = unsafe {
        core::slice::from_raw_parts(start as *const u8, size)
    };
    
    crate::tls13::crypto::sha256(text_bytes)
}

/// Compute SHA-256 of the kernel .rodata section
fn hash_rodata_section() -> [u8; 32] {
    let start = unsafe { &__rodata_start as *const u8 as usize };
    let end = unsafe { &__rodata_end as *const u8 as usize };
    let size = end.saturating_sub(start);
    
    let rodata_bytes = unsafe {
        core::slice::from_raw_parts(start as *const u8, size)
    };
    
    crate::tls13::crypto::sha256(rodata_bytes)
}

/// Compute combined kernel digest: SHA-256(text_hash || rodata_hash)
/// This is analogous to a TPM PCR extend — ordering matters.
fn compute_kernel_digest(text_hash: &[u8; 32], rodata_hash: &[u8; 32]) -> [u8; 32] {
    let mut combined = [0u8; 64];
    combined[..32].copy_from_slice(text_hash);
    combined[32..].copy_from_slice(rodata_hash);
    crate::tls13::crypto::sha256(&combined)
}

/// Called once at boot to record reference hashes of .text and .rodata sections.
/// Must be called after heap init but before any self-modifying code.
pub fn init_integrity() {
    let text_hash = hash_text_section();
    let rodata_hash = hash_rodata_section();
    let digest = compute_kernel_digest(&text_hash, &rodata_hash);
    
    let text_hex = hash_to_hex(&text_hash);
    let rodata_hex = hash_to_hex(&rodata_hash);
    let digest_hex = hash_to_hex(&digest);
    
    crate::serial_println!("[INTEGRITY] .text   : {} bytes, SHA-256: {}...{}", 
        text_section_size(), &text_hex[..16], &text_hex[56..]);
    crate::serial_println!("[INTEGRITY] .rodata : {} bytes, SHA-256: {}...{}", 
        rodata_section_size(), &rodata_hex[..16], &rodata_hex[56..]);
    crate::serial_println!("[INTEGRITY] kernel digest: {}...{}", 
        &digest_hex[..16], &digest_hex[56..]);
    
    *BOOT_TEXT_HASH.lock() = Some(text_hash);
    *BOOT_RODATA_HASH.lock() = Some(rodata_hash);
    *BOOT_KERNEL_DIGEST.lock() = Some(digest);
}

/// Get the size of the .text section
pub fn text_section_size() -> usize {
    let start = unsafe { &__text_start as *const u8 as usize };
    let end = unsafe { &__text_end as *const u8 as usize };
    end.saturating_sub(start)
}

/// Get the size of the .rodata section
pub fn rodata_section_size() -> usize {
    let start = unsafe { &__rodata_start as *const u8 as usize };
    let end = unsafe { &__rodata_end as *const u8 as usize };
    end.saturating_sub(start)
}

/// Get the boot-time .text hash as hex
pub fn boot_text_hash_hex() -> Option<String> {
    BOOT_TEXT_HASH.lock().map(|h| hash_to_hex(&h))
}

/// Get the boot-time .rodata hash as hex
pub fn boot_rodata_hash_hex() -> Option<String> {
    BOOT_RODATA_HASH.lock().map(|h| hash_to_hex(&h))
}

/// Get the boot-time kernel digest as hex
pub fn boot_kernel_digest_hex() -> Option<String> {
    BOOT_KERNEL_DIGEST.lock().map(|h| hash_to_hex(&h))
}

/// Verify kernel integrity: re-hash .text and .rodata and compare to boot-time hashes.
/// Returns Ok(true) if matching, Ok(false) if tampered, Err if not initialized.
pub fn verify_integrity() -> Result<bool, &'static str> {
    let boot_text = BOOT_TEXT_HASH.lock().ok_or("Integrity not initialized")?;
    let boot_rodata = BOOT_RODATA_HASH.lock().ok_or("Integrity not initialized")?;
    
    let current_text = hash_text_section();
    let current_rodata = hash_rodata_section();
    
    // Constant-time comparison for both sections
    let mut diff = 0u8;
    for i in 0..32 {
        diff |= boot_text[i] ^ current_text[i];
        diff |= boot_rodata[i] ^ current_rodata[i];
    }
    
    Ok(diff == 0)
}

/// Verify only .text section integrity
pub fn verify_text_integrity() -> Result<bool, &'static str> {
    let boot_hash = BOOT_TEXT_HASH.lock().ok_or("Integrity not initialized")?;
    let current = hash_text_section();
    let mut diff = 0u8;
    for i in 0..32 { diff |= boot_hash[i] ^ current[i]; }
    Ok(diff == 0)
}

/// Verify only .rodata section integrity
pub fn verify_rodata_integrity() -> Result<bool, &'static str> {
    let boot_hash = BOOT_RODATA_HASH.lock().ok_or("Integrity not initialized")?;
    let current = hash_rodata_section();
    let mut diff = 0u8;
    for i in 0..32 { diff |= boot_hash[i] ^ current[i]; }
    Ok(diff == 0)
}

/// Get full integrity report
pub fn integrity_report() -> Vec<String> {
    let mut lines = Vec::new();
    let text_size = text_section_size();
    let rodata_size = rodata_section_size();
    
    lines.push(String::from("  Kernel Integrity Verification"));
    lines.push(String::from("  ─────────────────────────────────────────────────"));
    lines.push(alloc::format!("  .text section   : {} bytes ({} KB)", text_size, text_size / 1024));
    lines.push(alloc::format!("  .rodata section : {} bytes ({} KB)", rodata_size, rodata_size / 1024));
    lines.push(String::from("  ─────────────────────────────────────────────────"));
    
    // .text hash
    if let Some(hex) = boot_text_hash_hex() {
        lines.push(alloc::format!("  .text boot hash   : {}", hex));
    } else {
        lines.push(String::from("  .text boot hash   : NOT INITIALIZED"));
        return lines;
    }
    let current_text = hash_text_section();
    lines.push(alloc::format!("  .text current     : {}", hash_to_hex(&current_text)));
    
    match verify_text_integrity() {
        Ok(true) => lines.push(String::from("  .text status      : ✅ INTACT")),
        Ok(false) => lines.push(String::from("  .text status      : ❌ MODIFIED")),
        Err(e) => lines.push(alloc::format!("  .text status      : ⚠️  {}", e)),
    }
    
    lines.push(String::from("  ─────────────────────────────────────────────────"));
    
    // .rodata hash
    if let Some(hex) = boot_rodata_hash_hex() {
        lines.push(alloc::format!("  .rodata boot hash : {}", hex));
    } else {
        lines.push(String::from("  .rodata boot hash : NOT INITIALIZED"));
    }
    let current_rodata = hash_rodata_section();
    lines.push(alloc::format!("  .rodata current   : {}", hash_to_hex(&current_rodata)));
    
    match verify_rodata_integrity() {
        Ok(true) => lines.push(String::from("  .rodata status    : ✅ INTACT")),
        Ok(false) => lines.push(String::from("  .rodata status    : ❌ MODIFIED")),
        Err(e) => lines.push(alloc::format!("  .rodata status    : ⚠️  {}", e)),
    }
    
    lines.push(String::from("  ─────────────────────────────────────────────────"));
    
    // Combined digest
    if let Some(hex) = boot_kernel_digest_hex() {
        lines.push(alloc::format!("  Kernel digest     : {}", hex));
    }
    let digest = compute_kernel_digest(&current_text, &current_rodata);
    lines.push(alloc::format!("  Current digest    : {}", hash_to_hex(&digest)));
    
    match verify_integrity() {
        Ok(true) => {
            lines.push(String::from("  Overall status    : ✅ INTEGRITY OK — kernel unmodified"));
        }
        Ok(false) => {
            lines.push(String::from("  Overall status    : ❌ INTEGRITY VIOLATION — kernel was tampered!"));
            lines.push(String::from("  WARNING: Code or read-only data modified since boot."));
        }
        Err(e) => {
            lines.push(alloc::format!("  Overall status    : ⚠️  {}", e));
        }
    }
    
    lines.push(String::from("  ─────────────────────────────────────────────────"));
    lines.push(String::from("  Algorithm: SHA-256 per-section + combined digest"));
    lines.push(String::from("  Threat model: detects post-boot code/data tampering"));
    
    lines
}
