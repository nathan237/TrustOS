# A12 T8020 SecureROM — EMFI Attack Plan & DIY Device Design
## Complete Boot Chain Analysis + Homemade EMFI Hardware

---

## TABLE OF CONTENTS
1. [Recovery Mode Bypass — IMPOSSIBLE](#1-recovery-mode-bypass--impossible)
2. [Boot Chain Analysis — EMFI Target Identified](#2-boot-chain-analysis--emfi-target-identified)
3. [DIY EMFI Device — 3 Approaches](#3-diy-emfi-device--3-approaches)
4. [EMFI Attack Plan — Step by Step](#4-emfi-attack-plan--step-by-step)
5. [Timing Analysis](#5-timing-analysis)
6. [Bill of Materials](#6-bill-of-materials)

---

## 1. RECOVERY MODE BYPASS — IMPOSSIBLE

**Question**: Can we permanently boot into a modified recovery mode WITHOUT hacking SecureROM?

**Answer**: **NO. Absolutely impossible.**

### Proof from ROM Analysis

1. **SecureROM only knows DFU mode** — The entire ROM was searched for boot mode strings:
   - Found: `"DFU Mode"` at `0x10001C26F`
   - Found: `"iBoot-3865.0.0.4.7"` at `0x100000280`
   - **NOT found**: "recovery", "NOR", "NAND", "restore" — NONE of these exist in ROM

2. **ALL images pass through signature verification** — The boot chain is:
   ```
   SecureROM → img4_verify(A704) → cbz w8 → trampoline(882C) → iBoot
   ```
   There is NO alternate path. Every image (iBSS, LLB, iBoot) must pass `img4_verify`.

3. **Recovery mode is an iBoot feature, not SecureROM** — Recovery mode is implemented
   by iBoot itself. The chain is:
   ```
   SecureROM → signed iBSS → signed iBoot(recovery) → signed ramdisk
   ```
   Every step requires valid Apple signatures.

4. **Even jailbroken devices can't modify SecureROM flow** — Jailbreaks that modify
   recovery mode (e.g., custom restore images) work by exploiting iBoot AFTER
   SecureROM has already verified it. They still need a signed iBoot.

### Why Each Alternative Fails

| Approach | Why it fails |
|----------|-------------|
| Modified recovery IPSW | SecureROM verifies iBSS/LLB signature before executing |
| Custom NOR image | A12 boot ROM has no NOR boot path (only DFU) |
| Persistent boot args | Boot args stored in NVRAM, read by iBoot (not SecureROM) |
| SEP bypass | SEP has its own ROM, doesn't control boot chain |
| Downgrade to vulnerable iBoot | SecureROM enforces anti-rollback via ECID nonce |

**Conclusion**: There is NO path to unsigned code execution without either:
- (a) **SecureROM exploit** (our EMFI path), or
- (b) **iBoot exploit** (patchable by Apple, temporary)

---

## 2. BOOT CHAIN ANALYSIS — EMFI TARGET IDENTIFIED

### Complete Boot Chain (from DFU image reception to execution)

```
                    USB HOST (computer)
                         │
                         ▼
              ┌─────────────────────┐
              │  DFU_DNLOAD handler │  E4AC: receives image chunks
              │  Writes to SRAM     │  → 0x19C030000 (fixed buffer)
              │  E5F0: stores ptr   │  → USB_STATE+0x28
              └─────────┬───────────┘
                        │
                        ▼
              ┌─────────────────────┐
              │  DFU_MANIFEST       │  Transfer complete
              │  1AB0: dfu_run()    │  x0=0x19C030000, w1=imageSize
              └─────────┬───────────┘
                        │
                        ▼
              ┌─────────────────────┐
              │  A86C: create desc  │  malloc(0x18), tag='Memz'
              │  Stores image ptr   │  desc+0x10 = 0x19C030000
              └─────────┬───────────┘
                        │
                        ▼
    ┌───────────────────────────────────────┐
    │  1BC0: bl 0x10000A704                 │
    │  ═══════════════════════════════════  │
    │  img4_verify(desc, &tag, 1, 0,       │
    │             0x19C030000, maxsize, 0)  │
    │  Checks IMG4 manifest, ECID, nonce,  │
    │  hash chain, RSA-2048 signature       │
    │  Returns: w0 = 0 (OK) or -1 (FAIL)   │
    └───────────────────┬───────────────────┘
                        │
                        ▼
    ╔═══════════════════════════════════════╗
    ║  0x100001BC8: cbz w8, 0x1C5C         ║ ◄── THE EMFI TARGET
    ║  ═══════════════════════════════════  ║
    ║  If w8 == 0: VERIFIED   → goto 1C5C  ║
    ║  If w8 != 0: FAILED     → error path ║
    ╚═══════════════════╤═══════════════════╝
                        │
              ┌─────────┴─────────┐
              │                   │
         w8 == 0            w8 != 0
         (SUCCESS)          (FAILURE)
              │                   │
              ▼                   ▼
    ┌──────────────┐     ┌──────────────┐
    │ 1C5C: clean  │     │ 1BCC: error  │
    │ call A8C0    │     │ DFU retry    │
    │ call 7008    │     │ or reboot    │
    └──────┬───────┘     └──────────────┘
           │
           ▼
    ┌──────────────────────────┐
    │  882C: TRAMPOLINE        │
    │  SVC #0                  │
    │  Set flags:              │
    │    0x19C014034 = 1       │
    │    0x19C011418 = 1       │
    │    0x19C014031/32 = 1    │
    │  Platform cleanup:       │
    │    90A4, 71D4, 6C48      │
    │    6B5C, 11C70, FD0C     │
    │  Disable MMU: 430, 6798  │
    │  Get handler: 739C       │
    └──────────┬───────────────┘
               │
               ▼
    ┌──────────────────────────┐
    │  8914: blr x20           │  Jump to loaded image!
    │  x0 = boot_args          │  → 0x19C030000 (our code!)
    │  8918: b 8918 (inf loop) │
    └──────────────────────────┘
```

### THE TARGET: `cbz w8, 0x1C5C` at `0x100001BC8`

**ROM bytes at offset 0x1BC8**: This is a 4-byte ARM64 instruction.

**What EMFI needs to do** (any ONE of these):
1. **Corrupt w8 to 0** → cbz takes the branch → success path
2. **Skip the instruction** → NOP-equivalent → fall-through to next useful instruction  
3. **Corrupt the branch target** → but this is riskier
4. **Corrupt img4_verify return value** → w0/w8 becomes 0 before reaching cbz

**Option 1 is the PRIMARY goal**: Induce a transient fault during/before the `cbz` execution
that makes w8 read as 0, causing the branch to be taken.

### Alternative EMFI Targets (backup)

| Address | Instruction | Effect if corrupted |
|---------|-------------|-------------------|
| `0x100001BC8` | `cbz w8, 0x1C5C` | **PRIMARY** — skip signature check |
| `0x10000A814` | `cbnz w0, 0xA838` | Inside img4_verify — skip hash comparison fail |
| `0x10000A810` | `bl 0x100005480`  | The actual crypto call — corrupt return value |
| `0x100005480+` | RSA verify internals | Corrupt the RSA comparison itself |

---

## 3. DIY EMFI DEVICE — 3 APPROACHES

### Approach A: PicoEMP (RECOMMENDED — Best Balance)

The **PicoEMP** is an open-source EMFI tool by NewAE Technology. It's the best starting
point because it's proven, documented, and costs ~$30-50 to build.

**Architecture**:
```
┌─────────────┐    PWM      ┌──────────────┐    250V     ┌────────────┐
│  Raspberry   │───────────►│  HV Boost     │───────────►│  HV Cap    │
│  Pi Pico     │            │  Transformer  │            │  0.47µF    │
│  (RP2040)    │            │  ATB322524    │            │  630V      │
│              │   Pulse    │  (×2 needed)  │            │            │
│  GPIO pin    │───────────►│  IGBT Q2      │◄───────────┘            │
│              │            │  RGT16BM65DTL │                         │
│              │   Status   │               │    SMA      ┌──────────┐
│              │◄───────────│  Opto Q1      │───────────►│ EM Coil  │
│              │            │  LDA111STR    │            │ (ferrite  │
└─────────────┘            └──────────────┘            │ + wire)   │
                                                        └──────────┘
```

**How it works**:
1. **Charge phase**: Pi Pico generates PWM → drives boost transformer (×2 ATB322524 in series) → charges C3 (0.47µF 630V) to ~250V
2. **Armed**: Optocoupler feedback tells Pico when HV capacitor is charged
3. **Pulse**: Pico fires GPIO → gate drive transformer activates IGBT (Q2) → 250V dumped through SMA → EM coil generates magnetic pulse
4. **Isolation**: High-voltage side is galvanically isolated from low-voltage side (gate drive transformer)

**Key advantage for our use**: We can program the Pico to:
- Monitor USB traffic (trigger on specific packets)
- Generate pulse with programmable delay (nanosecond precision via PIO)
- Automated scanning loop

**Build cost**: ~$30-50 (PCB + components + Pico)

**Build time**: 2-4 hours (SMD soldering required, 0603 components)

**Files**: https://github.com/newaetech/chipshouter-picoemp

---

### Approach B: Ultra-Minimal DIY (Battery + MOSFET + Coil)

For those who want to understand the physics and build from absolute scratch.

**Circuit Schematic**:
```
                                    ┌─────────────────┐
                                    │    EMFI COIL     │
                                    │  (5-10 turns     │
                                    │   enameled wire  │
                                    │   on ferrite)    │
                                    └──┬──────────┬────┘
                                       │          │
                 ┌─────────────────────┘          │
                 │                                │
    ┌────────────┴──────────────┐                 │
    │        HV CAPACITOR       │                 │
    │     C1: 0.22-1µF 400V+    │                 │
    │    (ceramic or film cap)   │                 │
    │         CHARGED TO         │                 │
    │        200-300V DC         │                 │
    └────────────┬──────────────┘                 │
                 │                                │
                 │    DRAIN                        │
                 ├──────────┤                      │
                 │          │ IGBT/MOSFET          │
                 │          │ (500V+ rated)        │
                 │          │ e.g. IRF840          │
                 │          │ or IGBT              │
                 │    GATE  │ FGH40N60SMD          │
                 │    ┌─────┤                      │
                 │    │     │                      │
                 │    │     │ SOURCE               │
                 │    │     ├──────────────────────┘
                 │    │     │
                 │    │    GND ─────────────────┐
                 │    │                         │
                 │    │   ┌──────────────────┐  │
                 │    │   │   MCU TRIGGER    │  │
                 │    └───┤  Arduino/Pico    ├──┘
                 │        │  GPIO → Gate     │
                 │        │  (with 10Ω gate  │
                 │        │   resistor)      │
                 │        └──────┬───────────┘
                 │               │
    ┌────────────┴───────┐       │
    │   HV POWER SUPPLY  │       │
    │                    │       │
    │  Option 1: Wall    │       │
    │  adapter + boost   │       │
    │  converter module  │       │
    │  (MT3608 → 200V)   │       │
    │                    │   ┌───┴────────┐
    │  Option 2: Camera  │   │ USB to PC  │
    │  flash circuit     │   │ (trigger   │
    │  (salvaged)        │   │  source)   │
    │                    │   └────────────┘
    │  Option 3: Battery │
    │  + ZVS driver      │
    │  + flyback xformer │
    └────────────────────┘
```

#### Component Details

**1. HV Source (choose one)**:

| Option | Parts | Output | Cost | Difficulty |
|--------|-------|--------|------|------------|
| Camera flash salvage | Old disposable camera circuit | 300V, ~120µF built-in | $0-5 | Easy |
| ZVS driver + flyback | ZVS kit ($5) + old CRT flyback | 200-400V adjustable | $10-15 | Medium |
| MT3608 boost cascade | 2-3× MT3608 modules in series | 200V (with care) | $5-10 | Medium |
| Cockcroft-Walton | 1N4007 diodes + caps + AC source | 200-600V | $5 | Hard (tuning) |

**Best for battery operation**: ZVS driver (runs off 12V Li-Po batteries)
```
Battery 12V → ZVS Induction Heater Module → Flyback Transformer → 200-300V DC
                                              (from old CRT TV)
```

**2. HV Capacitor**:
- **Type**: Ceramic (best: MLCC 630V) or Film (polypropylene)
- **Value**: 0.22µF to 1µF at 400V+ rating
- **Energy**: E = ½CV² = ½ × 0.47µF × 250² = **14.7 mJ** (sufficient for EMFI)
- **Specific part**: KRM55TR72J474MH01K (0.47µF 630V, same as PicoEMP)
- **Alternative**: Salvaged camera flash cap (120-330µF 300V — WAY more energy, be careful)

**3. Switching Element**:
- **IGBT** (preferred): FGH40N60SMD (600V, 40A, fast switching)
  - Better for inductive loads (the coil is inductive)
  - Gate voltage: 10-15V
- **MOSFET** (alternative): IRF840 (500V, 8A, N-channel)
  - Faster switching but less rugged with inductive loads
  - MUST add flyback diode across coil (1N4007 or BYV26C)
- **Gate drive**: 10Ω resistor in series with gate, driven by MCU GPIO
  - If MCU is 3.3V: add gate driver IC (TC4420 or similar for fast 
    turn-on into the >1nF gate capacitance)

**4. EM Injection Coil** (THE BUSINESS END):
```
              ┌──────────────────────┐
              │    FERRITE CORE      │
              │                      │
              │   ┌──────────────┐   │
              │   │  5-10 turns  │   │
              │   │  of 0.3-0.5mm│   │
              │   │  enameled    │   │
              │   │  copper wire │   │
              │   └──────────────┘   │
              │                      │
              │   Tip: 1-3mm dia     │ ← This end points at chip
              │   (sharpened/ground  │
              │    for localization) │
              └──────────────────────┘
```

**Coil construction options**:

| Type | Core | Turns | Tip size | Best for |
|------|------|-------|----------|----------|
| Needle probe | Ferrite rod (3mm dia, 20mm long) | 5-8 | ~2mm | Precise targeting |
| Flat probe | Half ferrite toroid | 3-5 | ~5mm | Wide area scanning |
| SMA probe | SMA connector + ferrite bead | 10 | ~1mm | Ultra-precise |

**How to build the needle probe**:
1. Get a ferrite rod (from old AM radio antenna, or buy Ø3×20mm rod)
2. Wind 5-8 turns of 0.3mm enameled copper wire tightly around one end
3. Solder wire ends to SMA connector (or directly to cap/MOSFET)
4. Optional: sharpen the ferrite tip on sandpaper for better localization
5. Optional: add ferrite shielding tube around sides to focus field downward

**5. Trigger Circuit** (CRITICAL for A12 attack):
```
┌─────────────┐     USB       ┌──────────────┐
│  iPhone XR   │◄────D+/D-───►│  USB Pass-    │
│  (DFU mode)  │              │  through      │
└──────────────┘              │  board        │
                              └──────┬───────┘
                                     │ D+ tap
                                     │ (high-Z)
                              ┌──────┴───────┐
                              │  Trigger MCU  │
                              │  (Pico/CW)   │
                              │              │
                              │  Detects:    │
                              │  - SOF pkts  │
                              │  - SETUP pkt │
                              │  - STATUS    │
                              │    stage     │
                              │  completion  │
                              │              │
                              │  Outputs:    │
                              │  GPIO pulse  │──────► MOSFET GATE
                              │  with delay  │        (triggers EMFI)
                              │  (adjustable)│
                              └──────────────┘
```

**Trigger options**:
- **Best**: ChipWhisperer-Husky (~$550) — has built-in USB triggering, sub-ns jitter
- **Good**: Raspberry Pi Pico PIO — can detect USB packets, ~10ns jitter
- **OK**: Arduino + USB host shield — detect transfers, ~100ns jitter
- **Simple**: Power trace trigger — monitor iPhone VBUS current spike during boot

---

### Approach C: BBQ Lighter "Proof of Concept" ($5)

For initial testing that EMFI has ANY effect on the A12:

```
┌───────────────────┐
│   BBQ LIGHTER     │     ┌────────────┐
│  (piezoelectric   │     │ Half       │
│   igniter)        │────►│ ferrite    │
│                   │     │ toroid     │
│  Press button     │     │ + 3 turns  │──────► Point at A12 chip
│  → HV spark       │     │ wire       │
│  → EM pulse       │     └────────────┘
└───────────────────┘
```

**How to build** (5 minutes):
1. Remove piezo igniter from BBQ lighter
2. Take half of a small ferrite toroid (break one in half)
3. Wind 3 turns of thin wire around the ferrite half
4. Solder wire ends to the two igniter terminals
5. Press the button → generates a single EM pulse

**Limitations**:
- ❌ No timing control (manual button press)
- ❌ Low energy (single piezo spark)
- ❌ Not repeatable
- ✅ Proves concept — can you disturb the A12 at all?
- ✅ Costs $5 or less

**Use case**: Only for initial proof that the A12 reacts to EM pulses (crashes, resets, etc.)

---

## 4. EMFI ATTACK PLAN — STEP BY STEP

### Phase 0: Equipment Assembly (1-2 days)

**Required equipment**:

| Item | Purpose | Cost | Source |
|------|---------|------|--------|
| PicoEMP (or DIY) | EMFI pulse generator | $30-50 | Build from schematic |
| Raspberry Pi Pico | PicoEMP controller + trigger | $4 | Any electronics store |
| EM injection tip | Focused magnetic pulse | $5 | Build (ferrite + wire) |
| USB isolator | Protect computer | $15-30 | Adafruit 2107 |
| XYZ positioning | Precise probe placement | $50-200 | 3D printer / manual stage |
| Hot air station | PoP DRAM removal | $50-100 | 858D or similar |
| iPhone XR | Target device | — | Already have |
| USB-A/Lightning cable | DFU connection | $5 | Any |
| Python + libusb | Automated DFU control | Free | Already have |
| Oscilloscope (optional) | Timing measurement | $100-400 | Rigol DS1054Z |

**Total minimum budget**: ~$100-200 (building PicoEMP from scratch)

### Phase 1: Board Preparation (1 day)

**Step 1.1**: Disassemble iPhone XR
- Remove screws, disconnect battery, lift display
- Disconnect all flex cables
- Extract logic board

**Step 1.2**: Remove PoP DRAM from A12 
- **WHY**: DRAM is stacked directly on top of A12, blocking EM access to the die
- **HOW**: Hot air rework station at 250-280°C, BGA underfill may need chemical softener
- **RISK**: Medium — the A12 itself must survive. Practice on dead boards first
- **ALTERNATIVE**: Try attacks WITHOUT removing DRAM first (through the DRAM is possible 
  but much less likely to succeed)

**Step 1.3**: Prepare trigger connection
- Solder thin wire to USB D+ pad (or use USB breakout board in-line)
- Alternative: monitor VBUS current with shunt resistor (100mΩ in series)

**Step 1.4**: Build mounting jig
- 3D print a holder for the logic board
- Mount EMFI probe on XYZ stage (manual micrometer stage or 3D printer gantry)
- Ensure probe tip can reach A12 die surface with <1mm precision

### Phase 2: Initial Characterization (2-5 days)

**Step 2.1**: Verify DFU boot works
```python
import usb.core
# Put iPhone in DFU mode (hold buttons)
dev = usb.core.find(idVendor=0x05ac, idProduct=0x1227)
# Send a small test image (will fail signature check — that's expected)
test_image = b'\x00' * 0x800
# DFU_DNLOAD
dev.ctrl_transfer(0x21, 1, 0, 0, test_image)
# Verify device is still in DFU (didn't crash)
```

**Step 2.2**: BBQ lighter smoke test (if using Approach C first)
- Put iPhone in DFU mode
- Hold BBQ lighter probe over A12
- Click igniter while monitoring USB connection
- **Expected**: Device crashes/resets/disconnects = EM sensitivity confirmed
- **If nothing happens**: Need to remove DRAM to expose die

**Step 2.3**: Systematic XY scan with PicoEMP
```
FOR each X position (0 to A12_width, step 0.5mm):
  FOR each Y position (0 to A12_height, step 0.5mm):
    FOR each timing delay (0 to 500µs, step 5µs):
      1. Enter DFU mode
      2. Send unsigned iBSS image
      3. Wait for DFU_MANIFEST state
      4. Trigger EMFI pulse at (X, Y, delay)
      5. Monitor USB response:
         - Normal error response → no effect
         - Device disconnects → crash (promising!)
         - Different error code → instruction corruption!
         - Device boots → SUCCESS!
      6. Log (X, Y, delay, response)
```

**Step 2.4**: Build heat map
- Plot response types on A12 die grid
- Identify "hot spots" where crashes occur
- These spots are where the CPU cores are located
- Focus further efforts on these regions

### Phase 3: Precision Attack (1-3 weeks)

**Step 3.1**: Refine timing around signature verification

The timing chain from our trigger point:
```
USB STATUS stage complete (trigger)
    │
    ├──→ ~10-50µs ──→ DFU state machine processes MANIFEST
    │
    ├──→ ~20-80µs ──→ Main thread wakes, calls dfu_run (1AB0)
    │
    ├──→ ~30-100µs ──→ A86C creates image descriptor
    │
    ├──→ ~40-150µs ──→ A704 img4_verify starts
    │
    ├──→ ~50-200µs ──→ A704 processes IMG4 manifest
    │                   (this takes variable time depending on image size)
    │
    ├──→ ~100-500µs ──→ 5480: actual RSA signature verification
    │                    (RSA-2048 takes ~ms on A12 at 2.49GHz)
    │
    ├──→ ~1-5ms ──→ A704 returns
    │
    └──→ ~1-5ms + 2 cycles ──→ 1BC8: cbz w8, 0x1C5C ← TARGET
```

**Step 3.2**: Narrow the timing window

After finding crashes in Phase 2:
1. Identify the timing range where crashes cluster
2. Sweep that range with 1µs steps, then 100ns steps
3. Look for the "sweet spot" where response changes from crash → different-error
4. That's near the signature verification code path

**Step 3.3**: Vary pulse power
- Start at low power (PicoEMP default ~250V)
- If too many crashes → reduce duty cycle of HV charge (lower stored energy)
- If no effect → need higher power (ChipSHOUTER-grade, or larger capacitor)
- Sweet spot: just enough to flip a few bits, not enough to crash the whole SoC

**Step 3.4**: Attack the target instruction
- When timing + position are narrowed:
- Craft a valid-looking IMG4 container with unsigned payload inside
- The payload should be a minimal iBSS that immediately signals success via USB
- Send via DFU, trigger EMFI at calibrated (X, Y, timing)
- **Success indicator**: Device starts executing our code instead of showing DFU error

### Phase 4: Payload Development

**Step 4.1**: Craft unsigned iBSS payload
```
Minimal bootloader that:
1. Initializes UART (for debug output)
2. Sends "PWNED" string over USB control transfers
3. Patches iBoot signature check in memory
4. Loads and boots custom iBoot
```

**Step 4.2**: Verify execution
- If EMFI succeeds, device will execute our code at 0x19C030000
- Our code runs in EL1/EL3 with FULL hardware access
- Can then: dump SecureROM, patch boot chain, load unsigned iBoot

**Step 4.3**: Build persistence
- Once we can boot unsigned iBSS via EMFI:
- Modify NVRAM/boot chain to load custom iBoot permanently
- Or: patch SecureROM at runtime (write to MMIO to remap ROM area)
- Or: install persistent boot hook in SPI NOR

---

## 5. TIMING ANALYSIS

### A12 Clock Speed: 2.49 GHz (DFU mode may run slower)

| Event | Estimated time from trigger | Precision needed |
|-------|---------------------------|-----------------|
| USB STATUS complete | T₀ (trigger) | — |
| IRQ handler returns | T₀ + 5-20µs | 10µs |
| DFU state machine | T₀ + 10-50µs | 10µs |
| dfu_run called | T₀ + 20-100µs | 10µs |
| A86C descriptor creation | T₀ + 30-150µs | 10µs |
| A704 starts (img4_verify) | T₀ + 40-200µs | 10µs |
| 5480 RSA verify starts | T₀ + 100-500µs | 50µs |
| A704 returns | T₀ + 1-10ms | 1ms |
| **1BC8 cbz w8** | **T₀ + 1-10ms + ~4ns** | **<100ns** |

### Timing Strategy

**Wide sweep first** (Phase 2):
- Delay range: 0 to 20ms
- Step size: 50µs
- 400 timing points × scanning takes ~1 hour per XY position

**Narrow sweep** (Phase 3):
- Once crash timing range identified: sweep with 1µs steps
- Then 100ns steps around the most promising timing
- Goal: find the ~10-100ns window where cbz executes

### USB-Based Triggering

```
SOF ─── SOF ─── SOF ─── SOF ─── SETUP(DFU_DNLOAD) ─── DATA ─── STATUS
                                                                    │
                                                              TRIGGER POINT
                                                                    │
                                                              delay (T)
                                                                    │
                                                              EMFI PULSE
```

We trigger on the STATUS stage ACK of the last DFU_DNLOAD transfer. This is the moment
the device transitions to DFU_MANIFEST state and begins processing the image.

---

## 6. BILL OF MATERIALS

### Option 1: PicoEMP Build (Recommended)

| # | Part | Qty | PN / Description | Est. Cost |
|---|------|-----|-----------------|-----------|
| 1 | Raspberry Pi Pico | 1 | SC0915 | $4 |
| 2 | HV Transformer | 2 | ATB322524-0110-T000 | $3 ea |
| 3 | IGBT 500V | 1 | RGT16BM65DTL (TO-252) | $2 |
| 4 | Logic MOSFETs | 2 | AO3422 (SOT-23) | $1 |
| 5 | HV Capacitor 0.47µF 630V | 1 | KRM55TR72J474MH01K | $3 |
| 6 | Optocoupler | 1 | LDA111STR | $2 |
| 7 | Zener 18V | 1 | MM3Z18VB | $0.50 |
| 8 | HV Rectifier | 1 | MURA160T3G | $1 |
| 9 | Misc resistors/caps/LEDs | lot | See BOM above | $5 |
| 10 | PCB (JLCPCB) | 5 | From gerbers | $5-10 |
| 11 | Safety shield | 1 | Hammond 1551BTRD | $3 |
| 12 | SMA connector (edge) | 1 | 0732511150 | $2 |
| 13 | **Ferrite rod** | 1 | Ø3mm × 20mm | $2 |
| 14 | **Enameled wire** | 1m | 0.3mm / 30AWG | $1 |
| 15 | **SMA male** | 1 | For coil connector | $2 |
| | | | **TOTAL PicoEMP** | **~$40-50** |

### Option 2: Absolute Minimum DIY

| # | Part | Qty | Description | Est. Cost |
|---|------|-----|-------------|-----------|
| 1 | Arduino Nano / Pi Pico | 1 | Trigger + control | $4-8 |
| 2 | IGBT or N-MOSFET 500V+ | 1 | IRF840 / FGH40N60 | $2-5 |
| 3 | Camera flash circuit | 1 | Salvaged from disposable camera | $0-5 |
| 4 | Film capacitor 400V | 1 | 0.22-1µF (if not using camera cap) | $3 |
| 5 | Ferrite rod | 1 | From old AM radio or bought | $2 |
| 6 | Enameled wire | 1m | 0.3-0.5mm | $1 |
| 7 | Gate resistor 10Ω | 1 | Any | $0.10 |
| 8 | Flyback diode | 1 | 1N4007 or UF4007 | $0.10 |
| 9 | Misc wires/connectors | lot | | $2 |
| | | | **TOTAL Minimum** | **~$15-30** |

### Additional Tools Needed

| Tool | Purpose | Own or Buy |
|------|---------|-----------|
| Hot air rework (858D) | Remove PoP DRAM | ~$50-80 |
| Soldering iron (fine tip) | Board mods, coil building | ~$30-50 |
| USB isolator | Protect computer from HV | ~$15-25 |
| Micrometer XYZ stage OR 3D printer | Probe positioning | $50-200 |
| Multimeter | HV measurement, debugging | ~$20 |
| Oscilloscope (optional but helpful) | Timing measurement | ~$100-400 |

---

## 7. SAFETY WARNINGS

⚠️ **HIGH VOLTAGE DANGER**: The EMFI device operates at 200-300V DC. This can cause
painful shocks and potentially cardiac arrest if current path crosses the heart.

**Safety rules**:
1. **ALWAYS** use the safety shield / insulating enclosure
2. **NEVER** touch the SMA output or HV capacitor when armed
3. **ALWAYS** discharge capacitors before handling (press discharge button or short with insulated screwdriver)
4. **USE** a USB isolator between computer and EMFI device
5. **WORK** on an ESD mat with shoes on (not barefoot)
6. **DO NOT** use near water or in humid environments
7. **HAVE** someone else present when working with HV

---

## 8. SUCCESS CRITERIA

The attack is successful when:

1. ✅ iPhone enters DFU mode normally
2. ✅ We send an unsigned iBSS image via DFU_DNLOAD
3. ✅ EMFI pulse fires at calibrated timing
4. ✅ The `cbz w8, 0x1C5C` at `0x100001BC8` is corrupted → branch taken
5. ✅ Trampoline at `0x10000882C` executes
6. ✅ Our unsigned code at `0x19C030000` begins executing
7. ✅ We achieve arbitrary code execution in SecureROM context (EL3/EL1)

**From there**: We can dump the ROM, patch the boot chain, achieve untethered dualboot.

---

## 9. ESTIMATED TIMELINE

| Phase | Duration | Description |
|-------|----------|-------------|
| Equipment build | 1-2 days | Build PicoEMP + coil + jig |
| Board prep | 1 day | Disassemble iPhone, remove DRAM |
| Initial characterization | 3-7 days | XY scanning + timing sweep |
| Precision attack | 1-3 weeks | Narrow parameters, find window |
| Payload development | 2-3 days | Build unsigned iBSS + test |
| Optimization | 1 week | Improve success rate |
| **Total** | **3-6 weeks** | From build to first success |

**Key risk**: If the A12 has internal EM shielding or the die geometry prevents
coil coupling, we may need higher power (ChipSHOUTER-grade) or a different
probe geometry. This can add 1-2 weeks of iteration.

---

*Document created: Session 16*
*Based on: Complete SecureROM boot chain reverse engineering + PicoEMP/BADFET research*
*Target: `cbz w8, 0x1C5C` at `0x100001BC8` in A12 T8020 SecureROM*
