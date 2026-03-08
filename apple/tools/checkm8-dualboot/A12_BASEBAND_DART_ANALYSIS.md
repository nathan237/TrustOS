# A12 iPhone XR — Baseband & DART Attack Surface Analysis

## Date: Session 17 (continued)
## Objective: Evaluate Intel PMB9955 baseband as alternative attack vector

---

## 1. CORRECTION: Chip Identification

**WRONG**: "Intel XMM 7660"  
**CORRECT**: **Intel PMB9955 (X-Gold 756)**, codename **ICE18** (iOS/iPadOS)

Source: TheiPhoneWiki + TheAppleWiki, confirmed for:
- iPhone XR, iPhone XS, iPhone XS Max
- iPad Pro 11" (1st gen), iPad Pro 12.9" (3rd gen)
- iPad (7th-9th gen), iPad Air 3rd gen, iPad mini 5th gen
- Apple Watch Series 4 through Ultra (as IBIS18)

---

## 2. Existing Security Research on PMB9955

### 2.1 Known Exploits: **ZERO**

From TheiPhoneWiki/TheAppleWiki Baseband Device page:
- PMB9955 X-Gold 756: **"None"**
- ALL Intel basebands after X-Gold 618 (2011): **"None"**

Only exploited Intel basebands:
| Chip | Era | Exploits |
|------|-----|----------|
| PMB8876 (S-Gold 2) | iPhone 1 | Fakeblank, IPSF, Minus 0x400 |
| PMB8878 (X-Gold 608) | iPhone 3G/3GS | JerrySIM, AT+XLOG, AT+XEMN heap overflow, AT+XAPP, AT+FNS |
| XMM 6180 (X-Gold 618) | iPhone 4 | AT+XAPP |

**No public exploit exists for any Intel baseband since 2011.**

### 2.2 Public RE Work: **MINIMAL**

- No dedicated RE tool exists for Intel iPhone basebands (unlike Samsung Shannon → ShannonRE)
- No Comsecuris/NCC Group/Google P0 public research found targeting PMB9955
- Firmware is **encrypted** — extracting from IPSW requires decryption keys
- Baseband firmware file in IPSW: typically named `ICE18-x.xx.xx.Release.bbfw`
- The `.bbfw` file is an img4-wrapped, encrypted firmware blob

### 2.3 Attack Surface Assessment

The Intel baseband has:
- Its own **boot ROM** (separate from AP A12 SecureROM)
- Its own **bootloader** chain
- Its own **firmware** in NOR flash
- Its own **RAM** (separate from AP DRAM)
- Communication with AP via **shared memory / mailbox** interface
- **AT commands** interface (but Apple restricts access severely since iOS 6+)

Theoretical attack vectors listed on TheiPhoneWiki:
1. **NCK Brute Force** — for carrier unlock, not code execution
2. **Baseband JTAG** — requires physical access to test pads on PCB

---

## 3. Apple DART (Device Address Resolution Table)

### 3.1 Official Apple Documentation

From Apple Platform Security Guide (January 2026):

> "Apple systems on chip contain an Input/Output Memory Management Unit (IOMMU)
> for each DMA agent in the system. Because each IOMMU has its own set of address
> translation tables to translate DMA requests, peripherals connected can access
> only memory that has been explicitly mapped for their use. Peripherals can't
> access memory belonging to other parts of the system—such as the kernel or
> firmware—memory assigned to other peripherals. If an IOMMU detects an attempt
> by a peripheral to access memory that isn't mapped for that peripheral's use,
> it institutes a kernel panic."

Key properties:
- **Per-device IOMMU**: Each DMA agent (including baseband) has its own DART
- **Default-deny policy**: Hardware blocks ALL DMA until explicitly allowed
- **Separate translation tables**: Baseband's DART cannot see AP kernel memory
- **Kernel panic on violation**: Unauthorized DMA access crashes the device

### 3.2 DART in pmap-io-ranges (from Kaspersky Triangulation research)

From Operation Triangulation analysis by Boris Larin (Kaspersky, 37C3 2023):
- DART mappings are stored in **pmap-io-ranges** in the DeviceTree
- Each entry has a **tag name**: PCIe, DART, DAPF, etc.
- Apple added unknown GPU MMIO ranges (0x206000000–0x206400000) to pmap-io-ranges
  as mitigation for CVE-2023-38606

### 3.3 DART Isolation: Baseband → AP

Architecture on A12 iPhone XR:
```
┌─────────────────────┐     ┌──────────────────────┐
│   AP (A12 Bionic)   │     │  Baseband (PMB9955)  │
│                     │     │  X-Gold 756 / ICE18  │
│  ┌───────────────┐  │     │                      │
│  │ SecureROM     │  │     │  ┌──────────────┐    │
│  │ iBoot         │  │     │  │ BB BootROM   │    │
│  │ XNU Kernel    │  │     │  │ BB Bootldr   │    │
│  │ iOS Userland  │  │     │  │ BB Firmware   │    │
│  └───────────────┘  │     │  └──────────────┘    │
│         ↕           │     │         ↕             │
│  ┌───────────────┐  │     │  ┌──────────────┐    │
│  │   AP DART     │  │     │  │  BB DART     │    │
│  │  (kernel-     │  │     │  │ (restricts   │    │
│  │   managed)    │  │     │  │  BB DMA)     │    │
│  └───────┬───────┘  │     │  └──────┬───────┘    │
│          │          │     │         │             │
└──────────┼──────────┘     └─────────┼─────────────┘
           │                          │
           └──────────┬───────────────┘
                      │
              ┌───────▼───────┐
              │  Shared DRAM  │
              │  (mapped via  │
              │   both DARTs) │
              └───────────────┘
```

**Critical isolation points:**
1. BB DART restricts the baseband to only its allocated DRAM region
2. AP DART restricts the baseband's view from the AP side
3. Even if baseband is compromised, it can only access shared memory buffers
4. iOS kernel controls what's mapped into the shared region

---

## 4. Operation Triangulation — Precedent for Hardware Bypass

### 4.1 CVE-2023-38606: Undocumented MMIO Bypass of PPL

The most sophisticated iPhone exploit chain ever documented (Kaspersky, 2023):
- Targeted **A12–A16 Bionic** SoCs (including our iPhone XR's A12!)
- Used **undocumented MMIO registers** at 0x206040000, 0x206140000, 0x206150000
- These registers belonged to the **GPU coprocessor** but were NOT in DeviceTree
- Provided **direct cache DMA** bypassing Page Protection Layer (PPL)
- Required a custom hash (actually Hamming ECC code) for each write

### 4.2 Relevance to Baseband Attack

Key insight from Triangulation: **DART can be bypassed** — but NOT through the
baseband. The Triangulation attackers didn't use the baseband at all. They:

1. Started from **iMessage** (remote code execution on AP)
2. Used **kernel vuln** (CVE-2023-32434) for read/write physical memory
3. Used **undocumented GPU MMIO** to bypass PPL
4. Never touched the baseband

This tells us:
- **DART is effective** — even state-level attackers with 4 zero-days chose
  to bypass PPL via GPU MMIO rather than through the baseband
- The baseband isolation is strong enough that it wasn't considered a viable
  pivot path even by the most advanced threat actors

---

## 5. Feasibility Assessment: Baseband → AP Attack Path

### 5.1 Attack Requirements

To exploit the baseband for AP code execution, you would need:

| Step | Requirement | Difficulty |
|------|-------------|------------|
| 1 | Access to baseband firmware (decrypted) | **VERY HARD** — encrypted in IPSW |
| 2 | Reverse engineer baseband firmware | **EXTREME** — no public tools, unknown architecture details |
| 3 | Find baseband vulnerability | **HARD** — no AT command access on modern iOS |
| 4 | Trigger baseband vuln | **HARD** — need rogue cell tower or SIM-based attack |
| 5 | Achieve code execution on baseband | **HARD** — unknown mitigations |
| 6 | Escape DART isolation | **EXTREME** — default-deny, per-device, crash-on-violation |
| 7 | Gain AP kernel access | **EXTREME** — shared memory is controlled by AP kernel |

### 5.2 Compared to EMFI

| Factor | Baseband Path | EMFI Path |
|--------|--------------|-----------|
| Entry point | Encrypted firmware, no tools | Known target: cbz at 0x1BC8 |
| Tooling cost | Rogue cell tower ($10K+) | PicoEMP ($40-50) |
| Existing research | Zero for PMB9955 | Documented EMFI on A-series |
| Isolation to bypass | DART (hardware IOMMU) | None (pre-boot) |
| Time estimate | 6-12+ months of RE | Days to weeks |
| Success probability | < 5% | ~30-60% per attempt |

### 5.3 Verdict

**The baseband attack vector is NOT viable for our goal** (dualboot via SecureROM).

Reasons:
1. **No public research exists** on PMB9955 security
2. **Firmware is encrypted** — we can't even start RE without decryption keys
3. **DART isolation is hardware-level** — even Operation Triangulation didn't use baseband
4. **Even if we owned the baseband**, we'd still need to escape DART AND then
   exploit the AP kernel AND then still need to bypass SecureROM signature check
5. **The baseband can't touch SecureROM** — it's a completely separate processor
   that runs before the kernel/DART even exist

### 5.4 One Exception: TF-A Vulnerability Patterns

From our earlier `tfa_sip_handler_vulnerability_analysis.md`:
- Intel/Altera SoCFPGA had 7+ HIGH severity vulns (missing address validation in SMC handlers)
- Pattern: V3 API added new SMC calls without `is_address_in_ddr_range()` checks
- Apple doesn't use public TF-A code, but the PATTERN is universal

**IF** someone were to RE the PMB9955 firmware, these patterns would be the
first thing to audit:
- Missing bounds checks on shared memory buffer addresses
- Integer overflows in mailbox message handlers
- Use-after-free in connection state management
- Stack buffer overflows in AT command parsers (historical precedent on PMB8878)

But this requires the encrypted firmware to be decrypted first — a chicken-and-egg problem.

---

## 6. Confirmed Primary Path: EMFI

The analysis reinforces our conclusion from preceding sessions:

### EMFI remains the ONLY viable path to bypass A12 SecureROM signature verification.

**4 ranked targets** (from A12_EMFI_ATTACK_PLAN.md):
1. `cbz w8, 0x1C5C` at `0x100001BC8` — THE conditional branch after img4_verify
2. Fuse read at `0x100007394` → `0x10000073BC`
3. Config read at `0x100007474` → `0x10000074C` 
4. Crypto result at `0x10000A814`

### DART is irrelevant for EMFI because:
- EMFI targets the **SecureROM** which runs **before** the kernel
- There is no DART/IOMMU active during DFU mode
- The attack is physical (electromagnetic) not software

---

## 7. Summary

| Research Question | Answer |
|---|---|
| What chip is in iPhone XR? | **PMB9955 X-Gold 756** (ICE18), NOT "XMM 7660" |
| Any public exploits? | **ZERO** for PMB9955 (since 2011 for any Intel BB) |
| Any public RE work? | **MINIMAL** — firmware encrypted, no tools |
| What is DART? | **Per-device IOMMU**, default-deny, crash-on-violation |
| Can baseband bypass DART? | **NO** — even Operation Triangulation (state-level) used GPU instead |
| Is baseband → AP viable? | **NO** for our goal (6+ hard steps, all blocking) |
| Best path? | **EMFI** — confirmed, documented, affordable |

---

## References

1. TheAppleWiki — Baseband Device: https://theapplewiki.com/wiki/Baseband_Device
2. TheiPhoneWiki — PMB9955: https://www.theiphonewiki.com/wiki/PMB9955
3. Apple Platform Security Guide — DMA Protections: https://support.apple.com/guide/security/direct-memory-access-protections-seca4960c2b5/web
4. Kaspersky — Operation Triangulation: The Last (Hardware) Mystery: https://securelist.com/operation-triangulation-the-last-hardware-mystery/111669/
5. CVE-2023-38606 — Undocumented MMIO bypass of PPL on A12-A16
6. TF-A SiP Handler Vulnerability Analysis — `tfa_sip_handler_vulnerability_analysis.md` (our workspace)
7. Quarkslab — Titan M Exploitation (CVE-2022-20233): analog for secure chip RE methodology
