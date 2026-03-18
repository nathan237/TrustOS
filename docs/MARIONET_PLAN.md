# MARIONET — Plan Architectural Complet

**Plateforme de Debugging Bare-Metal Temps-Réel pour TrustOS**

*Version: Plan v1.0 — 16 mars 2026*
*Auteur: Nathan + Copilot*

---

## 1. VISION

Marionet est un outil intégré à TrustOS qui **cartographie, surveille et visualise en temps réel** l'intégralité du matériel d'un PC bare-metal. L'objectif : donner au développeur/debugger une **radiographie complète** de la machine, comme si chaque fil (chaque "ficelle" — d'où "Marionet") reliant le CPU au hardware était visible et traçable.

**Cas d'usage principaux :**
- Debugger un PC inconnu (on le boot en TrustOS, Marionet scanne tout)
- Diagnostiquer un crash bare-metal (quel contrôleur a déclenché quel interrupt ?)
- Reverse-engineer du hardware (mapper les registres MMIO, les IRQ, les DMA)
- Surveiller en temps réel la santé thermique/électrique
- Tracer les exceptions et comprendre la chaîne de causalité

---

## 2. ARCHITECTURE GLOBALE

```
┌─────────────────────────────────────────────────────────┐
│                   MARIONET UI LAYER                     │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌───────────┐  │
│  │ Topology │ │ Live     │ │ Timeline │ │ Exception │  │
│  │ View     │ │ Monitors │ │ Tracer   │ │ Inspector │  │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └─────┬─────┘  │
├───────┴────────────┴────────────┴──────────────┴────────┤
│                 MARIONET CORE ENGINE                    │
│  ┌─────────────┐ ┌──────────────┐ ┌──────────────────┐  │
│  │ Probe       │ │ Exception    │ │ Event Ring       │  │
│  │ Scheduler   │ │ Safety Layer │ │ Buffer           │  │
│  └─────────────┘ └──────────────┘ └──────────────────┘  │
├─────────────────────────────────────────────────────────┤
│               HARDWARE PROBE MODULES                    │
│  ┌─────┐┌─────┐┌─────┐┌─────┐┌─────┐┌──────┐┌──────┐  │
│  │ CPU ││ MEM ││ PCI ││ ACPI││ IRQ ││ STOR ││ NET  │  │
│  └─────┘└─────┘└─────┘└─────┘└─────┘└──────┘└──────┘  │
│  ┌─────┐┌─────┐┌─────┐┌─────┐┌─────┐┌──────┐┌──────┐  │
│  │ USB ││ GPU ││AUDIO││INPUT││THERM││ SEC  ││CHIPST│  │
│  └─────┘└─────┘└─────┘└─────┘└─────┘└──────┘└──────┘  │
├─────────────────────────────────────────────────────────┤
│              EXCEPTION SAFETY FRAMEWORK                 │
│  Page Fault Guard │ GPF Guard │ Timeout │ NMI Handler   │
└─────────────────────────────────────────────────────────┘
```

---

## 3. CATALOGUE EXHAUSTIF DES CONTRÔLEURS À MAPPER

### 3.1 CPU & Registres Internes

| Sous-système | Ce qu'on lit | Comment |
|---|---|---|
| **CPUID** (toutes les feuilles) | Vendor, famille, modèle, stepping, features (SSE, AVX, AES-NI, etc.), cache topology, XSAVE state size | `cpuid` instruction, leaf 0x00→0x1F + extended 0x80000000→0x80000008 |
| **MSR** (Model Specific Registers) | Fréquence, température, voltage, microcode rev, IA32_APIC_BASE, EFER, PAT, MTRR, IA32_PERF_CTL, IA32_THERM_STATUS | `rdmsr`/`wrmsr` — **attention GPF si MSR invalide** |
| **Control Registers** | CR0 (mode protégé, paging, FPU), CR2 (page fault addr), CR3 (page table base), CR4 (PAE, SSE, SMEP, SMAP, PCID) | Lecture directe — privilège ring 0 |
| **Debug Registers** | DR0-DR3 (breakpoint addresses), DR6 (status), DR7 (control) | Lecture directe |
| **Performance Counters** | Instructions retired, cache misses, branch mispredicts, TLB misses | PMC via MSR, RDPMC instruction |
| **TSC** | Fréquence CPU, calibration | RDTSC + calibration via PIT/HPET/ACPI PM timer |
| **Microcode** | Version chargée | MSR 0x8B |
| **CPU Topology** | Cores, threads, packages, NUMA | CPUID leaf 0x0B (x2APIC) + ACPI SRAT |
| **FPU/SSE/AVX State** | Taille XSAVE, état sauvegardé | CPUID leaf 0x0D, XGETBV |
| **Machine Check Architecture** | Erreurs matérielles (ECC, bus, cache) | MSR MCi_STATUS/ADDR/MISC, MCG_CAP |

### 3.2 Mémoire

| Sous-système | Ce qu'on lit | Comment |
|---|---|---|
| **Memory Map physique** | Régions RAM, réservées, ACPI, MMIO holes | Limine memmap / E820 |
| **Page Tables** | Mappages PML4→PDP→PD→PT, flags (P, RW, US, NX, G, PAT) | Walk CR3 tree |
| **MTRR** (Memory Type Range Registers) | Write-back, write-through, uncacheable par région | MSR 0x200-0x20F |
| **PAT** (Page Attribute Table) | Attributs cache par page | MSR 0x277 |
| **Cache Topology** | L1i/L1d/L2/L3 taille, associativité, line size | CPUID leaf 0x04 |
| **TLB** | Entrées, niveaux | CPUID leaf 0x02 / 0x18 |
| **NUMA** | Nœuds, affinity, latences | ACPI SRAT/SLIT tables |
| **Heap TrustOS** | Utilisation, fragmentation | Interne allocateur |
| **DMA Zones** | ISA DMA (<16MB), regular, high | Allocation physique |

### 3.3 Interrupts & Exceptions

| Sous-système | Ce qu'on lit | Comment |
|---|---|---|
| **IDT** (256 vecteurs) | Handler address, DPL, type (interrupt/trap), IST stack | Lecture table IDT (SIDT) |
| **PIC 8259** (legacy) | IMR, ISR, IRR pour master+slave | Ports 0x20-0x21, 0xA0-0xA1 |
| **Local APIC** | ID, version, TPR, ISR, IRR, TMR, LVT entries, timer config | MMIO @ IA32_APIC_BASE |
| **I/O APIC** | ID, version, 24 redirection entries (vector, delivery, mask, dest) | MMIO @ ACPI MADT address |
| **MSI/MSI-X** | Message address/data par device PCI | PCI capability structures |
| **x2APIC** | Si supporté, passage aux MSR au lieu de MMIO | CPUID + MSR |
| **NMI** | Sources, routing | ACPI MADT NMI entries |
| **IRQ Routing** | Table complète IRQ→pin→APIC→vector | Combinaison ACPI + I/O APIC |

### 3.4 PCI / PCIe

| Sous-système | Ce qu'on lit | Comment |
|---|---|---|
| **Enumération complète** | Bus 0-255, Device 0-31, Function 0-7 | Config space @ 0xCF8/0xCFC ou ECAM (MCFG) |
| **Config Space** (256B legacy / 4KB PCIe) | Vendor/Device ID, Class, Subclass, Revision, BARs, Interrupt line/pin | Type 0/1 headers |
| **BARs** | Adresses MMIO et I/O, taille (via probe), prefetchable, 64-bit | Écriture 0xFFFFFFFF + lecture back |
| **Capabilities** | PM, MSI, MSI-X, PCIe, VPD, HT, vendor-specific | Linked list @ offset 0x34 |
| **PCIe Extended** | AER (Advanced Error Reporting), Link status/speed/width, LTR, L1 substates | Extended config space 0x100+ |
| **PCIe Link** | Gen (1/2/3/4/5), width (x1/x4/x8/x16), errors | PCIe capability @ offset 0x12 |
| **SR-IOV** | Virtual Functions | PCIe extended capability |
| **IOMMU** (Intel VT-d / AMD-Vi) | DMA remapping, Interrupt remapping tables | ACPI DMAR/IVRS tables |

### 3.5 ACPI & Firmware

| Sous-système | Ce qu'on lit | Comment |
|---|---|---|
| **RSDP/XSDP** | OEM, revision, RSDT/XSDT address | Scan mémoire ou Limine |
| **Toutes les tables** | FADT, MADT, MCFG, HPET, BGRT, SSDT, DSDT, BERT, EINJ, SRAT, SLIT, DMAR, IVRS, WAET, FPDT | RSDT/XSDT pointers |
| **DSDT/SSDT (AML)** | Device tree, méthodes _STA, _CRS, _PRS, _INI | **Parser AML minimal** (complexe mais essentiel) |
| **FADT** | PM1a/b, GPE blocks, reset register, century, boot flags, ACPI enable/disable ports | Table fixe |
| **EC** (Embedded Controller) | Température, fans, batterie, LEDs, charge | ACPI EC space (port 0x62/0x66 ou MMIO) |
| **SMBIOS/DMI** | Motherboard model, BIOS version, serial #, RAM slots (type, speed, taille), CPU socket | Table @ 0xF0000-0xFFFFF ou EFI |
| **BGRT** | Boot logo (pour debug : "c'est quel BIOS ?") | Table ACPI |

### 3.6 Stockage

| Contrôleur | Ce qu'on lit | Comment |
|---|---|---|
| **AHCI** (SATA) | Ports, device signatures, FIS, command slots, NCQ depth | BAR 5 MMIO (ABAR) |
| **NVMe** | Namespaces, queues, controller identify, SMART/health | BAR 0 MMIO |
| **ATA/IDE** (legacy) | IDENTIFY response (modèle, série, firmware, LBA count, DMA modes) | Ports 0x1F0-0x1F7 / 0x170-0x177 |
| **ATAPI** | CD/DVD identify | Même ports, PACKET command |
| **RAID** | Metadata, array config | Vendor-specific (Intel RST, etc.) |
| **SMART** | Température, heures on, reallocated sectors, pending sectors | ATA SMART commands |
| **USB Mass Storage** | Descriptors, endpoints | USB stack |

### 3.7 Réseau

| Contrôleur | Ce qu'on lit | Comment |
|---|---|---|
| **Ethernet MACs** | MAC address, link status, speed/duplex, autoneg | Per-driver (e1000, RTL, etc.) |
| **PHY** | MDIO registers (status, abilities, link partner) | MII management via driver |
| **TX/RX Rings** | Descriptors, head/tail pointers, pending packets | Per-driver MMIO |
| **WiFi** | SSID scan, signal strength, channel, firmware version | Per-driver (iwl4965) |
| **Wake-on-LAN** | Magic packet config | PCI PM + NIC registers |
| **Offload Engines** | Checksum, TSO, RSS, VLAN | PCI capabilities + NIC config |

### 3.8 USB

| Sous-système | Ce qu'on lit | Comment |
|---|---|---|
| **xHCI** (USB 3.x) | Ports, devices, speed, slots, en/endpoints | BAR MMIO |
| **EHCI** (USB 2.0) | Ports, companion controllers | BAR MMIO |
| **OHCI/UHCI** (USB 1.x) | Legacy controllers | BAR MMIO / I/O ports |
| **Device Tree** | Hub topology, descriptors (device, config, interface, endpoint, string) | USB transactions |
| **Power Delivery** | Port power, max current | xHCI extended capabilities |

### 3.9 GPU / Display

| Sous-système | Ce qu'on lit | Comment |
|---|---|---|
| **Framebuffer Info** | Résolution, pitch, BPP, adresse physique | Limine framebuffer protocol |
| **VGA Registers** | Séquencer, CRT controller, Graphics, Attribute, DAC | Ports 0x3C0-0x3DF (si VGA compatible) |
| **EDID** | Moniteur : modèle, résolutions supportées, timing | I2C/DDC (port GPIO GPU) |
| **GPU PCI** | Vendor (Intel/AMD/NVIDIA), VRAM BAR size | PCI config space |
| **VirtIO GPU** | Scanouts, resources | VirtIO queues |
| **GOP Modes** | EFI Graphics Output Protocol modes disponibles | UEFI runtime (si disponible) |

### 3.10 Audio

| Sous-système | Ce qu'on lit | Comment |
|---|---|---|
| **HDA** (Intel High Definition Audio) | Codec enumeration, widget tree (DAC, ADC, Mixer, Pin), connections, amp capabilities | BAR MMIO + CORB/RIRB |
| **AC97** (legacy) | Codec registers, mixer levels | BAR I/O ports |
| **HDA Pin Config** | Default device (line-out, HP, mic, speaker), location, connector type, color | Widget verb 0xF1C |
| **HDA Stream** | BDL (Buffer Descriptor List), format, running state | MMIO registers |

### 3.11 Input

| Sous-système | Ce qu'on lit | Comment |
|---|---|---|
| **i8042** (PS/2 Controller) | Status, control byte, port 1/2 enabled, self-test | Ports 0x60/0x64 |
| **Keyboard** | Scancode set (1/2/3), LED state, typematic rate | PS/2 commands |
| **Mouse** | Protocol (standard, IntelliMouse, Explorer), sample rate, resolution | PS/2 commands |
| **Touchpad** | Synaptics/ALPS identification, mode, capabilities | PS/2 vendor extensions |
| **USB HID** | Report descriptors, usage pages | USB HID class |

### 3.12 Thermique / Power

| Sous-système | Ce qu'on lit | Comment |
|---|---|---|
| **CPU Température** | Package temp, per-core temp, Tjmax | MSR 0x19C (IA32_THERM_STATUS), 0x1A2 (TEMPERATURE_TARGET) |
| **Fans** | Speed (RPM), duty cycle | ACPI EC, SuperI/O registers |
| **Batterie** | Charge, voltage, design capacity, cycle count, status | ACPI _BST/_BIF/_BIX |
| **AC Adapter** | Present, type | ACPI _PSR |
| **Throttling** | CPU throttle state, raison | MSR IA32_THERM_STATUS bit 0 (PROCHOT) |
| **P-States / C-States** | Fréquences disponibles, état idle | MSR IA32_PERF_STATUS, ACPI _PSS/_CST |
| **S-States** | Sleep states supportés (S0-S5) | ACPI FADT |

### 3.13 Sécurité

| Sous-système | Ce qu'on lit | Comment |
|---|---|---|
| **TPM** | Version (1.2/2.0), manufacturer, PCR values | MMIO @ ACPI TPM2 table / LPC I/O |
| **Secure Boot** | State, PK, KEK, db/dbx | UEFI variables (si runtime dispo) |
| **IOMMU** | Intel VT-d / AMD-Vi tables, remapping status | ACPI DMAR/IVRS |
| **Intel TXT** | TXT capable, ACM | MSR + MMIO |
| **AMD SEV/SME** | Memory encryption status | MSR 0xC0010131 |
| **SMEP/SMAP** | Activé dans CR4 | CR4 bits 20/21 |

### 3.14 Chipset & Low-Level

| Sous-système | Ce qu'on lit | Comment |
|---|---|---|
| **SMBus / I2C** | Devices on bus (EEPROM, sensors, SPD), addresses | PCI BAR + SMBus protocol |
| **SPD** (RAM info) | Type (DDR3/4/5), speed, timings, manufacturer, serial | SMBus @ 0x50-0x57 |
| **Super I/O** | Chip ID (ITE, Winbond, Nuvoton), fan control, HW monitor, GPIO | I/O ports 0x2E/0x2F ou 0x4E/0x4F |
| **LPC / eSPI Bridge** | Chipset ID, decode ranges, generic I/O | PCI device 00:1F.0 |
| **SPI Flash** | BIOS chip ID, protection, regions | Chipset SPI interface (MMIO) |
| **GPIO** | Pad ownership, configuration, state | Chipset MMIO (PCH GPIO communities) |
| **CMOS** | RTC time, century, boot flags, diagnostic status | Ports 0x70/0x71 (128 bytes) |
| **Watchdog** | TCO watchdog status, timeout, reboot reason | Chipset SMBus region (PCI 00:1F.4) |
| **POST Codes** | Dernier code POST (debug startup) | Port 0x80 (write/read) |
| **DMA Controllers** | 8237 legacy DMA channels, page registers | Ports 0x00-0x0F (DMA1), 0xC0-0xDF (DMA2) |
| **PIT** (8254 Timer) | Mode, reload value, current count | Ports 0x40-0x43 |
| **HPET** | Capabilities, period, comparators, routing | ACPI HPET table → MMIO |
| **RTC** (Real Time Clock) | Time, alarm, periodic interrupt, status registers | Port 0x70/0x71, registres 0x00-0x0D |

---

## 4. FRAMEWORK DE GESTION DES EXCEPTIONS

**Principe fondamental : Marionet ne doit JAMAIS crasher le système, même en sondant du hardware invalide.**

### 4.1 Types d'exceptions à gérer

| Exception | Vecteur | Cause dans Marionet | Stratégie |
|---|---|---|---|
| **#DE** Divide Error | 0 | Bug calcul interne | Report + skip probe |
| **#DB** Debug | 1 | Hardware breakpoint hit | Log + continue |
| **#NMI** Non-Maskable Interrupt | 2 | Hardware error (RAM, bus) | Log critique + alert dashboard |
| **#BP** Breakpoint | 3 | Debug intentionnel | Pass to debugger |
| **#OF** Overflow | 4 | Bug calcul interne | Report + skip |
| **#BR** Bound Range | 5 | Bug interne | Report + skip |
| **#UD** Invalid Opcode | 6 | CPU ne supporte pas l'instruction (ex: RDRAND sur vieux CPU) | **Catch → feature not supported** |
| **#NM** Device Not Available | 7 | FPU/SSE pas activé | Skip FPU probe |
| **#DF** Double Fault | 8 | Exception pendant exception handler | **CRITICAL — dump state + stop probe** |
| **#TS** Invalid TSS | 10 | Bug TSS (ne devrait pas arriver) | Report + abort |
| **#NP** Segment Not Present | 11 | Accès mémoire invalide | Report + skip |
| **#SS** Stack Segment Fault | 12 | Stack corruption | **CRITICAL — emergency dump** |
| **#GP** General Protection Fault | 13 | **LE PLUS FRÉQUENT**: accès I/O port invalide, MSR invalide, MMIO non-mappé | **Probe-safe recovery (voir 4.2)** |
| **#PF** Page Fault | 14 | Accès MMIO non-mappé, mapping raté | **Probe-safe recovery (voir 4.2)** |
| **#MF** x87 FPU Error | 16 | Bug FPU | Skip + report |
| **#AC** Alignment Check | 17 | Accès non-aligné | Skip + report |
| **#MC** Machine Check | 18 | **Hardware failure réel** (RAM, bus, cache) | **CRITICAL — full MCA bank dump + alert** |
| **#XM** SIMD Exception | 19 | Bug SSE/AVX | Skip + report |

### 4.2 Mécanisme de Probe-Safe Recovery

```
┌─────────────────────────────────────────┐
│         PROBE-SAFE EXECUTION            │
│                                         │
│  1. Set PROBE_ACTIVE flag (per-CPU)     │
│  2. Save recovery RIP in PROBE_RECOVER  │
│  3. Execute probe instruction           │
│     ├─ Success → clear flag, return OK  │
│     └─ Exception (#GP/#PF) →            │
│        Exception handler checks flag:   │
│        ├─ Flag SET → jump to RECOVER    │
│        │   return Err(ProbeException)   │
│        └─ Flag CLEAR → normal fault     │
│           (panic/kill process)          │
└─────────────────────────────────────────┘
```

**Implémentation conceptuelle :**
- Variable per-CPU `PROBE_CONTEXT` contenant : flag actif, adresse de recovery, type de probe, timeout
- Les handlers #GP et #PF vérifient `PROBE_CONTEXT.active` AVANT de paniquer
- Si actif : **modifier RIP** dans le stack frame d'exception pour sauter à l'adresse recovery, mettre le résultat en erreur
- C'est le pattern utilisé par Linux (`probe_kernel_read`, `get_user`/`put_user` avec `_ASM_EXTABLE`)

### 4.3 Timeout et Watchdog

| Situation | Mécanisme |
|---|---|
| **Hardware qui ne répond pas** (bus hang) | APIC timer watchdog : si un probe prend > N µs, NMI fired → abort probe |
| **Boucle infinie dans polling** | Counter-based timeout (max N itérations) |
| **DMA qui ne complète pas** | Timeout sur completion queue polling |
| **USB device qui hang** | Transaction timeout + port reset |
| **SMBus stuck** | Bus recovery sequence (9 clock pulses) |

### 4.4 Niveaux de Sévérité

```
SAFE     — Lecture CPU (CPUID, registres) : aucun risque
CAUTION  — Lecture I/O ports connus : rare #GP
RISKY    — Scan I/O ports inconnus : possible #GP, possible side-effects
DANGER   — Écriture MMIO/MSR : peut altérer l'état hardware
CRITICAL — Reset hardware, power management : peut éteindre/rebooter
```

Chaque probe module déclare son niveau. L'UI affiche le niveau. Le mode par défaut ne fait que SAFE+CAUTION.

---

## 5. PLATEFORME UI — LE DASHBOARD MARIONET

### 5.1 Layout Principal (framebuffer)

```
┌──────────────────────────────────────────────────────────────────────┐
│ MARIONET v1.0 ─ TrustOS Hardware Debug Platform                    │
│ CPU: Intel Core i7-XXXX @ 2.8GHz │ RAM: 8192 MB │ Uptime: 00:05:32│
├──────────────────────┬───────────────────────────────────────────────┤
│                      │  ┌─ LIVE MONITOR ──────────────────────────┐ │
│  SYSTEM TOPOLOGY     │  │ CPU Temp: ████████░░ 62°C / 100°C      │ │
│                      │  │ CPU Load: ██████░░░░ 58%                │ │
│  [CPU] ─┬─ [L1]     │  │ RAM Used: ████░░░░░░ 2048/8192 MB      │ │
│         ├─ [L2]     │  │ IRQ Rate: ████████░░ 1247/s             │ │
│         └─ [L3]     │  │ PF Rate:  █░░░░░░░░░ 3/s               │ │
│    │                 │  │ NMI:     0  GPF: 0  DF: 0              │ │
│  [PCH/Chipset]      │  └─────────────────────────────────────────┘ │
│   ├─ [PCI Bus 0]    │                                              │
│   │  ├─ 00:02.0 GPU │  ┌─ EVENT TIMELINE ────────────────────────┐ │
│   │  ├─ 00:1F.0 LPC │  │ 00:05:31.003 IRQ 1  → Keyboard         │ │
│   │  ├─ 00:1F.2 AHCI│  │ 00:05:31.047 IRQ 14 → Timer tick       │ │
│   │  └─ 00:1F.3 HDA │  │ 00:05:31.102 PF     → 0xFFFF8000_1234  │ │
│   ├─ [PS/2]         │  │ 00:05:31.150 IRQ 11 → AHCI completion   │ │
│   ├─ [COM1]         │  │ 00:05:31.203 IRQ 1  → Keyboard          │ │
│   └─ [USB xHCI]     │  │ ▼ scroll                                │ │
│      ├─ Hub Port 1  │  └─────────────────────────────────────────┘ │
│      └─ Hub Port 2  │                                              │
│                      │  ┌─ EXCEPTION LOG ─────────────────────────┐ │
│  MEMORY MAP          │  │ (empty — no exceptions caught)          │ │
│  ┌────────────────┐  │  │                                         │ │
│  │ 0x0000-0x9FFFF │  │  │                                         │ │
│  │ Conventional   │  │  └─────────────────────────────────────────┘ │
│  │ 0x100000-...   │  │                                              │
│  │ Kernel         │  │  [F1]Help [F2]Scan [F3]Export [F5]Refresh   │
│  │ 0xFEE00000     │  │  [F6]Freeze [F7]Filter [F10]Quit           │
│  │ APIC           │  │                                              │
│  └────────────────┘  │                                              │
├──────────────────────┴───────────────────────────────────────────────┤
│ > marionet probe pci                                 [SAFE MODE ON] │
└──────────────────────────────────────────────────────────────────────┘
```

### 5.2 Vues Disponibles

| Vue | Touche | Contenu |
|---|---|---|
| **Topology** | F2 → 1 | Arbre CPU → Chipset → Devices (comme ci-dessus) |
| **PCI Tree** | F2 → 2 | Arbre PCI complet avec détails par device (clic = expand) |
| **Memory Map** | F2 → 3 | Carte mémoire physique colorée (RAM/MMIO/reserved/ACPI) |
| **IRQ Map** | F2 → 4 | Tableau IRQ source → pin → APIC → vector → handler |
| **CPU State** | F2 → 5 | Registres CPU dump temps réel (CR, DR, MSR sélectionnés) |
| **ACPI Tables** | F2 → 6 | Liste toutes tables ACPI, hex dump sélectif |
| **Storage** | F2 → 7 | Disques détectés, SMART, partitions |
| **Network** | F2 → 8 | NICs, link status, PHY, MAC, stats |
| **USB Tree** | F2 → 9 | Arbre USB : hubs, devices, descriptors |
| **Audio** | F2 → A | HDA codecs, widget tree, pin config |
| **Thermal** | F2 → B | CPU temp, fans, batterie (live graphs) |
| **Timeline** | F2 → C | Chronologie scrollable de TOUS les events hardware |
| **Exception Inspector** | F2 → D | Détail des exceptions capturées (registres, stack, cause) |
| **SMBIOS** | F2 → E | Infos motherboard, BIOS, RAM slots |
| **Raw Probe** | F2 → F | Accès direct : lire port I/O, MSR, MMIO (mode DANGER) |

### 5.3 Fonctionnalités UI

- **Live refresh** : Les moniteurs se rafraîchissent en continu (configurable : 100ms → 5s)
- **Freeze** (F6) : Snapshot l'état, arrête le refresh pour analyse
- **Export** (F3) : Dump l'intégralité de la cartographie en texte (serial ou fichier)
- **Filter** (F7) : Filtre les events par type (IRQ seulement, exceptions seulement, etc.)
- **Navigation clavier** : Flèches pour se déplacer, Enter pour expand, Esc pour revenir
- **Serial mirror** : Tout le dashboard peut être redirigé sur le port série (pour debug sans écran)
- **Color coding** : Vert = OK, Jaune = warning, Rouge = erreur/exception, Bleu = info

---

## 6. MODULES DE PROBE — DÉTAIL

### 6.1 Module CPU Probe

```
Fonctions :
├── cpuid_full_dump()         → Toutes les feuilles CPUID
├── msr_safe_read(addr)       → Lecture MSR avec #GP recovery
├── msr_batch_read(list)      → Lecture batch de MSR connus
├── cr_dump()                 → CR0/CR2/CR3/CR4
├── dr_dump()                 → DR0-DR7
├── detect_features()         → Feature flags humainement lisibles
├── topology_map()            → Cores/threads/packages
├── perf_counter_setup()      → Configure PMC pour monitoring
├── thermal_read()            → Température via MSR
├── microcode_version()       → Version microcode
└── mca_bank_dump()           → Machine Check banks (erreurs HW)
```

### 6.2 Module PCI Probe

```
Fonctions :
├── enumerate_all()           → Scan bus 0-255 (ECAM si MCFG dispo)
├── read_config(bdf, offset)  → Lecture config space
├── dump_bars(bdf)            → Taille et type de chaque BAR
├── list_capabilities(bdf)    → Walk capability chain
├── pcie_link_status(bdf)     → Speed, width, errors
├── find_by_class(class)      → Chercher devices par classe
├── find_by_vendor(vid, did)  → Chercher par vendor/device
├── msi_info(bdf)             → MSI/MSI-X config
└── aer_status(bdf)           → Advanced Error Reporting
```

### 6.3 Module ACPI Probe

```
Fonctions :
├── list_tables()             → Toutes les tables trouvées
├── dump_table(signature)     → Hex dump d'une table
├── parse_madt()              → CPUs, APICs, overrides
├── parse_fadt()              → Power management info
├── parse_mcfg()              → PCIe ECAM base address
├── parse_hpet()              → High Precision Event Timer
├── parse_srat()              → NUMA affinity
├── parse_dmar()              → IOMMU tables
├── parse_smbios()            → DMI system info
├── ec_read(offset)           → Embedded Controller register
└── aml_evaluate(path)        → (Future) Évaluer un objet AML
```

### 6.4 Module Interrupt Probe

```
Fonctions :
├── dump_idt()                → Les 256 entrées IDT
├── dump_pic_state()          → IMR/ISR/IRR des deux PICs
├── dump_lapic()              → Tous les registres Local APIC
├── dump_ioapic()             → Toutes les redirection entries
├── irq_route_table()         → Mapping complet source → handler
├── msi_table(bdf)            → MSI-X table entries
├── count_irqs()              → Compteurs par vecteur (live)
└── trace_irq(vector)         → Log chaque occurrence d'un IRQ
```

### 6.5 Module Memory Probe

```
Fonctions :
├── physical_map()            → Carte mémoire physique complète
├── walk_page_tables(vaddr)   → Walk PML4→PT pour une adresse
├── dump_mtrr()               → Memory Type Range Registers
├── dump_pat()                → Page Attribute Table
├── cache_topology()          → L1/L2/L3 info
├── heap_stats()              → Utilisation heap TrustOS
├── find_mmio_regions()       → Régions MMIO depuis PCI BARs
└── test_address(addr)        → Test si accessible (probe-safe)
```

### 6.6 Module Storage Probe

```
Fonctions :
├── detect_ahci()             → Scan ports AHCI
├── detect_nvme()             → Scan controllers NVMe
├── detect_ide()              → Scan legacy ATA/IDE
├── identify_disk(port)       → ATA IDENTIFY / NVMe Identify
├── smart_data(port)          → SMART attributes
├── partition_table(disk)     → MBR/GPT parsing
└── io_benchmark(disk)        → Benchmark lecture séquentielle/random
```

### 6.7 Module Network Probe

```
Fonctions :
├── detect_nics()             → Scan PCI pour NICs
├── link_status(nic)          → Up/down, speed, duplex
├── mac_address(nic)          → Adresse MAC
├── phy_registers(nic)        → Dump MDIO/PHY
├── ring_status(nic)          → TX/RX ring descriptors
├── stats(nic)                → Packets TX/RX, errors, drops
└── wifi_scan(nic)            → (Si WiFi) Scan SSIDs
```

### 6.8 Module Chipset Probe

```
Fonctions :
├── identify_chipset()        → Vendor/model du PCH/southbridge
├── smbus_scan()              → Scan I2C bus (0x03-0x77)
├── spd_read(slot)            → RAM SPD data
├── superio_detect()          → Identifier le Super I/O chip
├── superio_hwmon()           → Voltages, fans, températures
├── gpio_dump()               → GPIO pad configuration
├── spi_flash_id()            → BIOS chip identification
├── lpc_decode()              → LPC decode ranges
├── watchdog_status()         → TCO watchdog state
└── cmos_dump()               → 128 bytes CMOS
```

---

## 7. COMMANDES SHELL

```
marionet                      → Lance le dashboard interactif
marionet scan                 → Full scan (SAFE+CAUTION), affiche résumé
marionet scan --deep          → Scan complet (inclut RISKY)
marionet probe cpu            → Probe CPU uniquement
marionet probe pci            → Probe PCI uniquement
marionet probe acpi           → Probe ACPI uniquement
marionet probe irq            → Probe interrupts
marionet probe mem            → Probe mémoire
marionet probe storage        → Probe stockage
marionet probe net            → Probe réseau
marionet probe usb            → Probe USB
marionet probe audio          → Probe audio
marionet probe chipset        → Probe chipset
marionet probe thermal        → Probe thermique
marionet probe security       → Probe sécurité (TPM, SecureBoot, IOMMU)
marionet export               → Export complet vers serial
marionet export file report   → Export vers fichier
marionet watch irq            → Mode watch : IRQ en temps réel
marionet watch temp           → Mode watch : température
marionet watch pf             → Mode watch : page faults
marionet raw port <addr>      → Lire un port I/O (DANGER)
marionet raw msr <addr>       → Lire un MSR (DANGER)
marionet raw mmio <addr>      → Lire un MMIO (DANGER)
marionet diff                 → Compare deux snapshots (avant/après)
marionet help                 → Aide complète
```

---

## 8. INTÉGRATION AVEC L'EXISTANT TRUSTOS

### 8.1 Ce qui existe déjà (à réutiliser)

| Existant | Fichier | Réutilisation dans Marionet |
|---|---|---|
| PCI enumeration | `kernel/src/pci.rs` | Base du PCI probe module |
| ACPI tables | `kernel/src/acpi/` | Base de l'ACPI probe |
| APIC/I/O APIC | `kernel/src/apic.rs` | IRQ probe |
| Interrupts/IDT | `kernel/src/interrupts/` | Exception safety + IRQ trace |
| NVMe driver | `kernel/src/nvme.rs` | Storage probe (NVMe) |
| ATA driver | `kernel/src/drivers/ata.rs` | Storage probe (ATA) |
| Network drivers | `kernel/src/drivers/net/` | Network probe |
| hwscan module | `kernel/src/hwscan/` | **À intégrer/absorber dans Marionet** |
| Serial output | `kernel/src/serial.rs` | Export serial |
| Framebuffer | `kernel/src/framebuffer/` | Dashboard UI |
| Shell | `kernel/src/shell/` | Commandes `marionet` |
| CPU ops | `kernel/src/arch/x86_64/cpu.rs` | CPU probe |
| Keyboard | `kernel/src/keyboard.rs` | Dashboard navigation |

### 8.2 Ce qui est nouveau (à créer)

| Nouveau | But |
|---|---|
| **Exception Safety Framework** | Probe-safe #GP/#PF recovery |
| **Event Ring Buffer** | Anneau circulaire pour events (IRQ, exceptions, probes) |
| **Dashboard Renderer** | UI framebuffer avec panneaux, scrolling, couleurs |
| **MSR Database** | Table des MSR connus (Intel + AMD) avec noms, descriptions |
| **SMBIOS Parser** | Parsing tables DMI |
| **HDA Codec Enumerator** | Walk HDA widget tree |
| **SMBus Scanner** | I2C/SMBus scan et SPD reader |
| **Super I/O Detector** | Détection Winbond/ITE/Nuvoton |
| **Thermal Monitor** | Lecture continue température (MSR + EC + SuperIO) |
| **Timeline Engine** | Stockage chronologique + rendu scrollable |
| **Snapshot/Diff Engine** | Capturer l'état complet, comparer deux snapshots |
| **AML Mini-Interpreter** | (Phase future) Interpréter le bytecode ACPI |

---

## 9. PLAN DE PHASES

### Phase 1 — Fondations
- [ ] Exception Safety Framework (probe-safe #GP/#PF recovery)
- [ ] Event Ring Buffer
- [ ] Module structure (`kernel/src/marionet/`)
- [ ] Commande shell `marionet` basique
- [ ] CPU Probe (CPUID, MSR safe read, CR dump)
- [ ] Intégration du PCI existant dans le format Marionet

### Phase 2 — Core Probes
- [ ] ACPI probe étendu (toutes tables)
- [ ] Memory probe (physical map, page table walker, MTRR)
- [ ] Interrupt probe (IDT dump, APIC state, IRQ counting)
- [ ] Storage probe (AHCI, NVMe, ATA, SMART)
- [ ] SMBIOS parser
- [ ] Export serial/fichier

### Phase 3 — Dashboard
- [ ] Dashboard renderer (panneaux, couleurs, scrolling)
- [ ] Topology view
- [ ] Live monitors (temp, IRQ rate, CPU load)
- [ ] Timeline tracer
- [ ] Exception inspector
- [ ] Navigation clavier

### Phase 4 — Advanced Probes
- [ ] Network probe (PHY, link, stats)
- [ ] USB probe (xHCI device tree)
- [ ] Audio probe (HDA codec walk)
- [ ] Chipset probe (SMBus, SuperIO, GPIO, SPI)
- [ ] Thermal monitoring (MSR + EC + SuperIO)
- [ ] Security probe (TPM, IOMMU, SecureBoot)

### Phase 5 — Intelligence
- [ ] Snapshot/Diff engine
- [ ] Raw probe mode (I/O port, MSR, MMIO avec warnings)
- [ ] Auto-diagnostic (détection d'anomalies : IRQ storm, thermal throttle, etc.)
- [ ] AML mini-interpreter (pour EC access, battery, etc.)
- [ ] Integration JARVIS (JARVIS peut analyser les rapports Marionet)

---

## 10. STRUCTURE DE FICHIERS PROPOSÉE

```
kernel/src/marionet/
├── mod.rs                 # Module principal, commande shell
├── safety.rs              # Exception Safety Framework (probe-safe)
├── event.rs               # Event Ring Buffer
├── probe/
│   ├── mod.rs             # Trait ProbeModule, types communs
│   ├── cpu.rs             # CPU probe (CPUID, MSR, CR, topology)
│   ├── pci.rs             # PCI/PCIe probe
│   ├── acpi.rs            # ACPI tables probe
│   ├── memory.rs          # Memory map, page tables, MTRR
│   ├── interrupt.rs       # IDT, PIC, APIC, IRQ routing
│   ├── storage.rs         # AHCI, NVMe, ATA, SMART
│   ├── network.rs         # Ethernet, WiFi, PHY
│   ├── usb.rs             # xHCI, device tree
│   ├── audio.rs           # HDA, AC97
│   ├── input.rs           # PS/2, touchpad
│   ├── chipset.rs         # SMBus, SuperIO, GPIO, SPI, LPC
│   ├── thermal.rs         # CPU temp, fans, battery
│   ├── security.rs        # TPM, IOMMU, SecureBoot
│   └── smbios.rs          # DMI/SMBIOS tables
├── dashboard/
│   ├── mod.rs             # Dashboard engine principal
│   ├── renderer.rs        # Rendu framebuffer (panneaux, couleurs)
│   ├── topology.rs        # Vue topologie
│   ├── monitors.rs        # Live monitors (temp, IRQ, CPU)
│   ├── timeline.rs        # Timeline event tracer
│   ├── memory_map.rs      # Vue carte mémoire
│   ├── inspector.rs       # Exception inspector
│   └── raw.rs             # Raw probe interface
├── export.rs              # Export serial/fichier
├── snapshot.rs            # Snapshot + diff engine
└── db/
    ├── msr_intel.rs       # Base de données MSR Intel
    ├── msr_amd.rs         # Base de données MSR AMD
    ├── pci_ids.rs         # Vendor/device ID → nom
    └── class_codes.rs     # PCI class/subclass → description
```

---

## 11. CONSIDÉRATIONS TECHNIQUES CRITIQUES

### 11.1 Performance
- Les probes NE DOIVENT PAS ralentir le système — polling configuré, pas de busy-wait
- Le dashboard tourne dans un "mode" séparé (pas pendant des opérations critiques)
- L'event ring buffer est lock-free (single producer pour les interrupts)

### 11.2 Portabilité Multi-Architecture
- Les probes CPU/MSR sont `#[cfg(target_arch = "x86_64")]`
- ARM : pas de MSR, pas de ports I/O — utiliser MMIO et Device Tree
- RISC-V : pas de MSR — utiliser CSR
- Le dashboard et l'event engine sont arch-indépendants

### 11.3 Sécurité
- Les commandes `raw` (port I/O, MSR, MMIO) requièrent confirmation explicite
- L'écriture sur hardware est INTERDITE par défaut (lecture seule sauf raw mode)
- Les probes DANGER nécessitent `marionet --unsafe` ou confirmation interactive
- Integration avec le Guardian (Le Pacte) : les raw writes passent par l'autorisation

### 11.4 Mémoire
- Budget mémoire estimé : ~256 KB pour les structures de données
- Event ring : 4096 entrées × 64 bytes = 256 KB
- MSR/PCI databases : compilées en `const` (pas de heap)
- Dashboard framebuffer : réutilise le framebuffer existant

---

*Ce document est le plan directeur. Aucun code n'a été écrit. Prochaine étape : review par Nathan, puis Phase 1.*
