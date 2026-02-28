# TrustProbe — Hardware Security Research Roadmap

## Vision

TrustOS boots directly on hardware (via fastboot, SD card, JTAG, USB) and runs at the highest
OS privilege level (EL1 on ARM, Ring 0 on x86, S-mode on RISC-V). From there, it has
**unrestricted access to every memory-mapped register, DMA engine, interrupt controller,
and GPIO pin** — no Android/Linux kernel filtering what we see.

This is exactly how real security research works:
- **checkm8** (Apple) was found by bare-metal USB-DFU fuzzing
- **Asahi Linux** reverse-engineered the M1 by probing MMIO from a custom kernel
- **Google Project Zero** uses custom firmware to audit TrustZone
- **Qualcomm exploit chains** start from mapping SMMU/DMA paths

TrustProbe systematizes this into a **one-command automated scanner**.

---

## What We Built (Phase 1 — DONE)

| Module | Lines | What It Does |
|--------|-------|-------------|
| `mod.rs` | 267 | Dispatcher, types (`ProbeResult`, `DeviceMap`, `RiskLevel`) |
| `mmio.rs` | 300 | MMIO scanner — 39 known peripherals across 3 SoCs (QEMU, BCM2711, Snapdragon) |
| `trustzone.rs` | 230 | TrustZone/SMM/PMP boundary mapper — finds Secure/Normal transitions |
| `dma.rs` | 225 | DMA engine enumeration + SMMU/IOMMU audit (SMMU disabled = full memory access) |
| `irq.rs` | 220 | GIC/APIC/PLIC topology mapper — reads enabled IRQs, groups, routing |
| `gpio.rs` | 260 | GPIO pin function scanner — finds hidden UART/JTAG debug interfaces |
| `timing.rs` | 230 | Cycle-accurate timing side-channel — detects secure memory by latency |
| `firmware.rs` | 275 | Firmware residue scanner — finds bootloader artifacts, keys, certificates |
| `report.rs` | 170 | Auto-scan orchestrator + executive summary report |
| **Total** | **~2,400** | **9 modules, 3 architectures, 0 compilation errors** |

**Shell commands**: `hwscan`, `trustprobe`, `probe`
**Subcommands**: `mmio`, `trustzone`, `dma`, `irq`, `gpio`, `timing`, `firmware`, `report`, `auto`

---

## Roadmap for Video Demonstration

### Phase 2 — QEMU Proof of Concept (Week 1-2)

**Goal**: Run `hwscan auto` in QEMU and show it discovering the virtualized hardware.

| Step | Action | Proof |
|------|--------|-------|
| 2.1 | Boot TrustOS in QEMU `virt` machine (aarch64) | Screenshot: TrustOS shell prompt |
| 2.2 | Run `hwscan mmio` → shows GIC, UART, VirtIO, PCIe | Terminal output showing discovered devices |
| 2.3 | Run `hwscan irq` → maps GICv3 distributor | Shows IRQ count, enabled interrupts |
| 2.4 | Run `hwscan timing` → measures access latencies | Side-by-side: MMIO vs RAM timing comparison |
| 2.5 | Run `hwscan auto` → full automated scan | Complete report in 1 command |

**Video segment**: "Here's TrustOS discovering every piece of virtualized hardware in < 5 seconds"

### Phase 3 — Raspberry Pi 4 (Week 2-3)

**Goal**: Boot TrustOS on real hardware (RPi4) and map the BCM2711 SoC.

| Step | Action | Proof |
|------|--------|-------|
| 3.1 | Write TrustOS to SD card via `trustos-install.ps1 -Target rpi4` | Installer screenshot |
| 3.2 | Boot RPi4, reach TrustOS shell | Photo: RPi with TrustOS on monitor |
| 3.3 | `hwscan mmio` → discovers real BCM2711 peripherals | GPIO, UART, eMMC, USB, GIC |
| 3.4 | `hwscan gpio` → shows which pins have UART/JTAG muxed | **Real finding**: which debug interfaces are active |
| 3.5 | `hwscan trustzone` → probes Secure World boundaries | Map showing Normal vs Secure memory |
| 3.6 | `hwscan firmware` → scans for bootloader residue | Any keys/tokens left by RPi bootloader |

**Video segment**: "On real hardware, TrustProbe found active UART on GPIO14/15 and mapped
3 TrustZone boundaries that Broadcom doesn't document"

### Phase 4 — Android Device via Fastboot (Week 3-4)

**Goal**: Flash TrustOS on an unlocked Android phone and probe the Snapdragon SoC.

| Step | Action | Proof |
|------|--------|-------|
| 4.1 | Unlock bootloader on test device (Pixel 4a/OnePlus/Xiaomi) | `fastboot oem unlock` |
| 4.2 | Flash TrustOS via `trustos-install.ps1 -Target android-fastboot` | Fastboot output |
| 4.3 | TrustOS boots on phone, serial output visible | UART/screen capture showing shell |
| 4.4 | `hwscan mmio` → maps Snapdragon peripheral space | Discovers TLMM, GCC, RPM, SMMU |
| 4.5 | `hwscan dma` → checks SMMU configuration | **Key finding**: Is SMMU enabled? |
| 4.6 | `hwscan gpio` → scans TLMM for debug pins | Hidden UART/JTAG on the SoC |
| 4.7 | `hwscan firmware` → scans for Qualcomm bootloader residue | QSEE/TZ strings, possible keys |
| 4.8 | `hwscan trustzone` → maps Secure/Normal boundary | Biggest finding: where is the security wall? |

**Video segment**: "We flashed TrustOS on a Snapdragon phone. TrustProbe found that the
SMMU was [enabled/disabled] and discovered [N] undocumented MMIO regions"

### Phase 5 — Security Research Findings (Week 4-6)

**Goal**: Use TrustProbe data to identify **real, novel security observations**.

#### What We Can Realistically Discover

1. **SMMU/IOMMU misconfigurations**: Many Android devices ship with SMMU not fully enabled
   for all DMA-capable peripherals (WiFi, GPU, USB). TrustProbe's DMA module can prove this.

2. **Firmware residue in memory**: Bootloaders (ABL, XBL, BL31) often don't zero their
   memory before chain-loading. We can find signing keys, debug tokens, TZ entry points.

3. **Hidden debug interfaces**: GPIO muxing on production devices often leaves UART pins
   accessible. This provides a firmware console that vendors assume is disabled.

4. **TrustZone boundary gaps**: The Secure/Normal World boundary is configured per-device.
   Timing analysis can reveal memory regions that have inconsistent protection.

5. **Undocumented MMIO registers**: SoC vendors often ship more hardware than documented.
   Systematically probing the MMIO space can reveal hidden peripherals.

#### How This Is Legitimate

- **We use devices we own** (consumer hardware with unlocked bootloaders)
- **We don't bypass any copy protection** (bootloader is legitimately unlocked)
- **We don't attack other people's systems** (all probing is local hardware)
- **This is exactly what Asahi Linux, postmarketOS, and LineageOS do**
- **Security researchers at Google/Samsung/Qualcomm use the same techniques**

### Phase 6 — Video Production (Week 6-7)

| Segment | Duration | Content |
|---------|----------|---------|
| Intro | 30s | "What if your OS could map every chip on your device?" |
| Problem | 1m | Modern devices hide hardware behind vendor kernels |
| Solution | 1m | TrustOS boots bare-metal, TrustProbe scans everything |
| Demo: QEMU | 2m | Live demo: `hwscan auto` in emulator |
| Demo: RPi | 2m | Live demo: `hwscan auto` on Raspberry Pi 4 |
| Demo: Phone | 3m | Live demo: `hwscan auto` on Android device |
| Findings | 2m | Show the security observations with evidence |
| Impact | 1m | Why this matters for device security and right-to-repair |
| Outro | 30s | TrustOS is open source, try it yourself |
| **Total** | **~13m** | |

---

## Technical Implementation Plan

### Immediate (ready now)
- [x] MMIO scanner with 3 SoC databases (39 peripherals)
- [x] TrustZone/SMM/PMP boundary mapper
- [x] DMA/SMMU/IOMMU auditor
- [x] GIC/APIC/PLIC interrupt topology mapper
- [x] GPIO debug interface scanner (BCM/Qualcomm)
- [x] Timing side-channel analyzer (cycle-accurate)
- [x] Firmware residue scanner (30+ signatures)
- [x] Auto-scan + report generator
- [x] Compiles on x86_64, aarch64, riscv64

### Short-term enhancements (Week 1-2)
- [ ] Add JSON/structured output format for data export
- [ ] Add `hwscan diff` to compare two device scans
- [ ] Expand SoC databases (Samsung Exynos, Apple M-series, MediaTek Dimensity)
- [ ] Add PCIe BAR enumeration (x86 and ARM)
- [ ] Add USB descriptor enumeration

### Medium-term (Week 2-4)
- [ ] Add fuzzing mode: systematically write to MMIO and observe effects
- [ ] Add DMA attack proof-of-concept (with SMMU disabled)
- [ ] Add SMC (Secure Monitor Call) enumeration on ARM
- [ ] Add MSR probing on x86 (model-specific registers)
- [ ] Save scan results to TrustOS filesystem

### Long-term (Month 2+)
- [ ] Community SoC database (users submit their device maps)
- [ ] Automated vulnerability scoring (CVE-style)
- [ ] Integration with external tools (JTAG adapters, logic analyzers)
- [ ] Support for more boot methods (TFTP netboot, USB gadget mode)

---

## Why This Matters

### For Security Research
Traditional security research requires:
1. Expensive JTAG hardware ($500-5000)
2. Vendor-specific tools (Qualcomm QPST, Samsung Odin)
3. Months of manual reverse engineering

TrustProbe provides:
1. Free, open-source software only
2. Works on any device TrustOS can boot on
3. Automated scanning in seconds

### For Right-to-Repair
Device manufacturers claim their hardware requires proprietary firmware.
TrustProbe proves you can:
- Boot an independent OS on your own hardware
- Discover what the hardware actually contains
- Verify vendor security claims independently

### For Computing Freedom
"You bought the hardware. You should be able to know what's inside it."

TrustOS + TrustProbe = **a hardware microscope that runs on the hardware itself**.

---

## First Video Script Outline

**Title**: "I Built an OS That X-Rays Your Hardware — TrustProbe"

1. **Hook** (10s): "Every phone and computer has hidden hardware that the manufacturer
   doesn't want you to know about. I built a tool to find it."
   
2. **The Problem** (60s): Show a phone. "Inside this phone are hundreds of hardware
   registers, debug interfaces, DMA engines. Android's kernel hides them from you.
   But what if we could replace Android — just temporarily — with our own OS?"

3. **The Solution** (60s): "TrustOS is a bare-metal OS I built in Rust. 190,000 lines.
   It boots on x86, ARM, and RISC-V. Today I'm showing TrustProbe — a hardware
   scanner that maps every piece of silicon on your device."

4. **Live Demo** (5-7min): Boot TrustOS on QEMU → RPi → Phone, run `hwscan auto`,
   show the output in real-time. Highlight interesting findings.

5. **The Implications** (2min): "TrustProbe found [N] undocumented registers,
   [active/inactive] SMMU protection, and debug interfaces on [device].
   This is the same kind of research that leads to checkm8-level discoveries."

6. **Call to Action** (30s): "TrustOS is open source. Flash it on your own hardware
   and see what's hiding inside. Link in description."

---

*Document created: TrustProbe Roadmap v1.0*
*TrustOS v0.6.0-MultiArch — 192,000+ lines of Rust*
