# 🔐 TrustOS Kernel Signature Registry

> **Public ledger of every signed TrustOS kernel.**  
> Each entry is an HMAC-SHA256 fingerprint — cryptographic proof that the signer built and endorsed their copy of TrustOS.

---

## How It Works

Every TrustOS kernel contains a **signature system** (`signature` command). Users can:

1. **Sign** their kernel with a personal passphrase → generates a unique HMAC-SHA256 fingerprint
2. **Export** their signature → `signature export` outputs a formatted entry
3. **Submit a PR** to this file to be publicly registered
4. **Prove** ownership anytime with `signature prove <name>` (requires original passphrase)

The fingerprint is derived from `HMAC-SHA256(passphrase, payload)`. Without the passphrase, nobody can forge the signature. With it, anyone can verify.

---

## ⚠️ Rules

- **One entry per person** — update your entry if you re-sign with a new key
- **Creator signature (#001) is immutable** — it cannot be modified or removed
- **Do not modify other people's entries**
- **Include your GitHub username** so we can verify PR authorship
- Submit via **Pull Request** only

---

## 📋 Registry

### #001 — CREATOR (immutable)

| Field | Value |
|-------|-------|
| **Name** | Nated0ge |
| **GitHub** | [@nathan237](https://github.com/nathan237) |
| **Role** | Creator & sole author |
| **Algorithm** | HMAC-SHA256 |
| **Signed Payload** | `TrustOS Kernel — Created by Nated0ge (nathan237) — Sole author and originator — All rights reserved 2025-2026` |
| **Fingerprint** | `0c1a99fb1e8777ce120cca834e75608e95a4b6c5d3047a92a1fe10b310b87cbd` |
| **Kernel Version** | v0.1.1 |
| **Date** | 2025-2026 |
| **Status** | ✅ Original creator — hardcoded in kernel binary |

> 🔒 This signature is compiled into every TrustOS kernel. The secret seed is known **only** to the creator.  
> Verify with: `signature prove-creator` inside TrustOS.

---

### Community Signatures

<!-- 
  TO ADD YOUR SIGNATURE:
  1. Boot TrustOS and run: signature sign <your_name>
  2. Then run: signature export
  3. Copy the output below
  4. Submit a Pull Request adding your entry

  FORMAT:
  ### #NNN — Your Name
  | Field | Value |
  |-------|-------|
  | **Name** | YourName |
  | **GitHub** | [@yourusername](https://github.com/yourusername) |
  | **Algorithm** | HMAC-SHA256 |
  | **Fingerprint** | `your_64_char_hex_fingerprint_here` |
  | **Kernel Version** | v0.1.1 |
  | **Date** | YYYY-MM-DD |
  | **Status** | ✅ Verified signer |
-->

*No community signatures yet. Be the first to sign your TrustOS kernel and submit a PR!*

---

## 📊 Stats

| Metric | Count |
|--------|-------|
| Total signatures | 1 |
| Creator | Nated0ge |
| Latest version signed | v0.1.1 |

---

<div align="center">

*Each signature is a cryptographic commitment — proof that someone trusted TrustOS enough to endorse it with their identity.*

**Sign yours today:** `signature sign <your_name>` → `signature export` → Submit PR

</div>
