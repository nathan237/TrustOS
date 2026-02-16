# TrustOS — Guide Développeur Avancé / Advanced Developer Guide

> **Objectif** : Ce document est la source de vérité absolue pour tout humain ou IA qui veut comprendre, modifier, optimiser ou étendre TrustOS. Il est conçu pour qu'un LLM puisse lire ce fichier et être immédiatement opérationnel sur 100% du codebase.

> **Purpose**: This document is the single source of truth for any human or AI that wants to understand, modify, optimize or extend TrustOS. It is designed so that an LLM can read this file and be immediately operational on 100% of the codebase.

---

## Table des matières / Table of Contents

1. [Vue d'ensemble / Overview](#1-overview)
2. [Architecture complète / Full Architecture](#2-architecture)
3. [Séquence de boot / Boot Sequence](#3-boot-sequence)
4. [Carte des modules / Module Map](#4-module-map)
5. [APIs clés / Key APIs](#5-key-apis)
6. [Système de build / Build System](#6-build-system)
7. [Conventions de code / Code Conventions](#7-code-conventions)
8. [Guide: Ajouter une commande shell](#8-add-shell-command)
9. [Guide: Ajouter un driver](#9-add-driver)
10. [Guide: Ajouter un module kernel](#10-add-kernel-module)
11. [Guide: Modifier le réseau](#11-modify-network)
12. [Guide: Modifier la GUI](#12-modify-gui)
13. [Guide: Modifier le système de fichiers](#13-modify-filesystem)
14. [Guide: Ajouter un syscall](#14-add-syscall)
15. [Guide: Optimiser les performances](#15-optimize-performance)
16. [Guide: Sécurité et crypto](#16-security-crypto)
17. [Guide: Hyperviseur / VMs](#17-hypervisor)
18. [Guide: TrustLang (langage intégré)](#18-trustlang)
19. [Erreurs courantes / Common Pitfalls](#19-common-pitfalls)
20. [Checklist avant commit](#20-checklist)

---

## 1. Overview

| Propriété | Valeur |
|-----------|--------|
| **Nom** | TrustOS (T-RustOS) |
| **Version** | 0.3.3 |
| **Auteur** | Nated0ge / nathan@trustos.dev |
| **Licence** | MIT |
| **Langage** | 100% Rust, `no_std`, zéro C |
| **Target** | `x86_64-unknown-none` (bare-metal) |
| **Toolchain** | Nightly Rust (`rust-toolchain.toml`) |
| **Bootloader** | Limine (UEFI) |
| **Lignes de code** | ~132,914 lignes de Rust |
| **Fichiers .rs** | 254 |
| **ISO** | ~8.4 MB |
| **Dépôt** | github.com/nathan237/TrustOS |

### Ce que TrustOS implémente from scratch :
- Noyau complet (GDT, IDT, pagination, heap, scheduler, processus, threads, signaux)
- Stack réseau Layer 2→7 (Ethernet → ARP → IP → TCP/UDP → DHCP → DNS → HTTP → HTTPS)
- TLS 1.3 (SHA-256, HMAC-HKDF, AES-128-GCM, X25519, Ed25519, ChaCha20)
- Navigateur web (HTML5 + CSS3 + JS ES5)
- Desktop graphique (Windows 11-style, COSMIC widgets, Wayland, OpenGL 1.x)
- Hyperviseur Type-1 (Intel VT-x + AMD SVM, EPT/NPT)
- Langage de programmation (TrustLang: lexer → parser → compiler → VM bytecode)
- Système de fichiers (VFS + FAT32 + TrustFS + devfs + procfs + ramfs)
- Audio (synthétiseur multi-oscillateur + HDA DMA playback)
- Vidéo (codec .tv custom: delta + RLE)
- Compatibilité Linux (shell + binaires ELF + syscalls via interpréteur x86_64)
- Assistant IA (Jarvis: NLU bilingue FR/EN + planificateur + exécuteur)

---

## 2. Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          USERLAND (Ring 3)                              │
│   init │ shell │ fs │ network │ jarvis │ compositor │ pixlens          │
├─────────────────────────────────────────────────────────────────────────┤
│                        SYSCALL INTERFACE                                │
│   syscall/ (635+1176 lines) │ interrupts/syscall │ userland (SYSRET)   │
├─────────────────────────────────────────────────────────────────────────┤
│                          KERNEL (Ring 0)                                │
│                                                                         │
│  ┌─── Noyau ─────────────┐  ┌─── Modèle de processus ───────────────┐ │
│  │ memory/ (paging,heap) │  │ process │ thread │ scheduler │ signals │ │
│  │ interrupts/ (IDT,PIC) │  │ exec │ elf │ pipe │ ipc │ ptrace      │ │
│  │ gdt (Ring 0/3, TSS)   │  │ sync/ (futex,rwlock,barrier,percpu)   │ │
│  │ cpu/ (SMP,TSC,AES-NI) │  └───────────────────────────────────────┘ │
│  │ acpi/ (MADT,FADT,HPET)│                                            │
│  └────────────────────────┘                                            │
│                                                                         │
│  ┌─── Drivers ─────────────────────────────────────────────────────┐   │
│  │ drivers/ : ahci │ ata │ xhci │ usb │ hda │ virtio_gpu │ input  │   │
│  │ drivers/net/ : e1000 │ rtl8139 │ virtio                        │   │
│  │ pci │ virtio │ virtio_blk │ virtio_net │ disk │ keyboard │ mouse│   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  ┌─── Systèmes de fichiers ─┐  ┌─── Réseau ───────────────────────┐  │
│  │ vfs/ : fat32 │ trustfs   │  │ netstack/ : TCP │ UDP │ DHCP     │  │
│  │ devfs │ procfs │ wal     │  │ DNS │ HTTP │ HTTPS │ ARP │ ICMP  │  │
│  │ ramfs │ persistence      │  │ socket (BSD API)                  │  │
│  └──────────────────────────┘  └───────────────────────────────────┘  │
│                                                                         │
│  ┌─── Sécurité ─────────────┐  ┌─── Crypto ────────────────────────┐  │
│  │ security/ (capabilities) │  │ tls13/ (AES-GCM, SHA-256, X25519)│  │
│  │ auth │ sandbox/ │ SMEP   │  │ ed25519 │ signature │ rng         │  │
│  └───────────────────────────┘  └───────────────────────────────────┘  │
│                                                                         │
│  ┌─── Graphique / Affichage ───────────────────────────────────────┐   │
│  │ framebuffer/ (console, font, double-buffer)                     │   │
│  │ graphics/ (2D, 3D, OpenGL, raytracer, SIMD, compositor)        │   │
│  │ gui/ (Windows 11 style) │ cosmic/ (widgets) │ wayland/          │   │
│  │ desktop │ compositor │ rasterizer │ gpu_emu │ icons │ theme     │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  ┌─── Applications ────────────────────────────────────────────────┐   │
│  │ browser/ (HTML+CSS+JS) │ chess │ chess3d │ game3d (FPS)         │   │
│  │ trustlang/ (langage)   │ binary_analysis/ │ transpiler/         │   │
│  │ audio/ (synth+HDA)     │ video/ (.tv codec) │ lab_mode/         │   │
│  │ shell/ (tsh + jarvis)  │ model_editor │ devtools │ formula3d    │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  ┌─── Virtualisation ──────────────────────────────────────────────┐   │
│  │ hypervisor/ : Intel VMX │ AMD SVM │ EPT/NPT │ Linux subsystem  │   │
│  │ linux/ │ linux_compat/ (WSL1-style) │ distro                    │   │
│  └─────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
         │ Limine Bootloader (UEFI) │ OVMF │ QEMU q35 │
```

---

## 3. Boot Sequence

Fichier: `kernel/src/main.rs` → `kmain()`

| Phase | Fonction | Description |
|-------|----------|-------------|
| 1 | `serial::init()` | UART 16550 pour debug série |
| 2 | `framebuffer::init()` | Framebuffer Limine (UEFI linear FB), console texte |
| 3 | `memory::init_with_hhdm_dynamic()` | Heap dynamique (25% RAM, 64–512 MB), frame allocator bitmap |
| 3.1 | `framebuffer::init_scrollback()` | Buffer de scroll (~3 MB) |
| 3.2 | `signature::init_integrity()` | SHA-256 de la section `.text` du kernel |
| 3.3 | `signature::init_ed25519()` | Keypair Ed25519 dérivée du digest kernel |
| 3.5 | `gdt::init()` | GDT avec segments Ring 0/3, TSS |
| 3.51 | `interrupts::init()` | IDT + PIC 8259 (timer, clavier, page fault) |
| 3.55 | `cpu::init()` | CPUID, TSC, AES-NI, détection SIMD |
| 3.555 | `acpi::init_direct()` | RSDP → RSDT/XSDT → MADT/FADT/MCFG/HPET |
| 3.56 | `cpu::smp::init()` | Boot des Application Processors via Limine SMP |
| 3.6 | `memory::paging::init()` | Sauvegarde CR3, active NX, PAT Write-Combining |
| 3.7 | `userland::init()` | Configure MSRs SYSCALL/SYSRET, pile syscall |
| 3.8 | `thread::init()` | Sous-système threads |
| 4 | Clavier | PS/2 keyboard ready |
| 6 | `rtc::try_init()` | Real-Time Clock (CMOS) |
| 7 | `mouse::init()` | PS/2 mouse driver |
| 8 | `pci::init()` | Enumération bus PCI |
| 9 | `task::init()` + `scheduler::init()` | Scheduleur à priorités (4 niveaux) |
| 10 | Disque | virtio-blk probe → fallback RAM disk |
| 11 | `drivers::init()` | Framework drivers + AHCI/IDE probe + VirtIO GPU |
| 12 | Réseau | PCI probe → e1000/virtio-net/rtl8139 → DHCP |
| 12a | `vfs::init()` | Virtual File System |
| 12b | `file_assoc::init()` | Associations type → application |
| 13 | `process::init()` | Gestionnaire de processus |
| 14 | `auth::init()` | Authentification + `/etc/passwd` |
| 15 | `init::start()` | PID 1 init process |
| 16 | `ramfs::init()` | RAM filesystem + répertoires standard |
| 17 | `persistence::init()` | Couche de persistance |
| 18 | Sandbox | `sandbox::init()` + `container::boot_daemon()` |
| **∞** | **`shell::run()`** | **Shell interactif (boucle infinie)** |

**Point critique** : `shell::run()` est le point final — c'est une boucle infinie `loop { read_line → execute_command }`. Tout le kernel tourne en Ring 0 actuellement (architecture monolithique malgré l'aspiration microkernel).

---

## 4. Module Map

### 4.1 Noyau (`kernel/src/`)

#### Mémoire (`memory/`) — 1,336 lignes
| Fichier | Lignes | Rôle |
|---------|--------|------|
| `mod.rs` | 179 | Init HHDM, heap dynamique (25% RAM, 64–512 MB) |
| `heap.rs` | 67 | `linked_list_allocator`, `used()`, `free()` |
| `frame.rs` | 287 | Allocateur de frames physiques (bitmap), `PhysRegion` |
| `paging.rs` | 810 | Tables de pages, `AddressSpace`, NX, PAT, Write-Combining |

#### CPU (`cpu/`) — 1,345 lignes
| Fichier | Lignes | Rôle |
|---------|--------|------|
| `mod.rs` | 344 | CPUID complet → `CpuCapabilities` struct |
| `features.rs` | 114 | Flags et requêtes de features |
| `tsc.rs` | 318 | Timing nanoseconde via TSC, calibration fréquence |
| `aesni.rs` | 318 | Chiffrement AES accéléré matériellement |
| `simd.rs` | 160 | Détection et activation SSE/AVX |
| `smp.rs` | 409 | Multi-core: boot APs, per-CPU data, distribution travail |

#### Interruptions (`interrupts/`) — 506 lignes
| Fichier | Lignes | Rôle |
|---------|--------|------|
| `mod.rs` | 71 | Setup IDT, enable interrupts |
| `handlers.rs` | 213 | Handlers exceptions/IRQ (timer, clavier, page fault) |
| `pic.rs` | 99 | Initialisation PIC 8259, EOI |
| `syscall.rs` | 123 | Dispatch `int 0x80` |

#### ACPI (`acpi/`) — 1,442 lignes
| Fichier | Rôle |
|---------|------|
| `mod.rs` | RSDP/RSDT/XSDT discovery, `AcpiInfo` agrégat |
| `tables.rs` | Parsing headers tables ACPI |
| `madt.rs` | MADT: Local APIC, I/O APIC, ISO |
| `fadt.rs` | FADT: Power mgmt, PM timer, reboot/shutdown |
| `mcfg.rs` | PCIe ECAM configuration |
| `hpet.rs` | High Precision Event Timer |

### 4.2 Shell (`shell/`) — 21,686 lignes total (le plus gros sous-système)

| Fichier | Lignes | Rôle |
|---------|--------|------|
| `mod.rs` | ~1,000 | Boucle REPL, autocomplete, dispatch `execute_command()` |
| `commands.rs` | ~1,470 | Commandes core (ls, cat, mkdir, cp, mv, echo, free, ps, neofetch) |
| `unix.rs` | ~1,070 | Commandes Unix (grep, find, wc, sort, head, tail, chmod, dmesg) |
| `network.rs` | ~790 | Commandes réseau (ping, wget, curl, ifconfig, netstat) |
| `apps.rs` | ~3,740 | Lanceurs d'apps (browser, chess, video, lab, gterm, trustlang) |
| `desktop.rs` | ~6,680 | Intégration desktop, showcase, benchmark |
| `vm.rs` | ~4,120 | Commandes VM/hyperviseur |
| `jarvis.rs` | ~930 | Assistant IA Jarvis (NLU + planner + executor) |
| `trailer.rs` | ~1,890 | Mode trailer cinématique |

**Point d'entrée** : `shell::run()` dans `mod.rs` ligne ~197. Boucle: prompt → `read_line_with_autocomplete()` → `execute_command()`.

**Dispatch** : `execute_command()` dans `mod.rs` ligne ~675. Un gigantesque `match` de ~300 lignes avec ~120+ patterns.

### 4.3 Réseau

#### Stack protocolaire (`netstack/`) — 2,046 lignes
| Fichier | Lignes | Rôle |
|---------|--------|------|
| `mod.rs` | 113 | Dispatch frames Ethernet |
| `arp.rs` | 165 | ARP requête/réponse, cache MAC |
| `ip.rs` | 163 | Construction/parsing paquets IPv4 |
| `icmp.rs` | 163 | ICMP echo (ping) |
| `tcp.rs` | 516 | Machine d'état TCP (SYN/ACK/FIN, retransmission) |
| `udp.rs` | 47 | Datagrammes UDP |
| `dhcp.rs` | 281 | Client DHCP (auto-config IP) |
| `dns.rs` | 100 | Résolveur DNS |
| `http.rs` | 337 | Client HTTP/1.1 |
| `https.rs` | 421 | HTTPS via TLS 1.3 |
| `socket.rs` | 458 | API sockets BSD-style |

#### Drivers réseau (`drivers/net/`)
| Fichier | Rôle |
|---------|------|
| `e1000.rs` (610) | Intel e1000 NIC |
| `rtl8139.rs` (86) | Realtek RTL8139 |
| `virtio.rs` (138) | VirtIO network |

### 4.4 Graphique

#### Framebuffer (`framebuffer/`) — 3,411 lignes
| Fichier | Rôle |
|---------|------|
| `mod.rs` | Console texte, double buffering, scrollback, macros `print!`/`println!` |
| `font.rs` | Police bitmap 8×16 (CP437) |
| `logo.rs` | Logo de boot, splash screen |

**Macros critiques** (définies dans `framebuffer/mod.rs`, `#[macro_export]`) :
```rust
print!(...)                     // Texte blanc standard
println!(...)                   // Avec newline
print_color!(0xAARRGGBB, ...)  // Texte coloré
println_color!(0xAARRGGBB, ...) // Coloré + newline
```

#### Moteur graphique (`graphics/`) — 7,279 lignes
| Fichier | Rôle |
|---------|------|
| `render2d.rs` | Primitives 2D via embedded-graphics |
| `render3d.rs` | Rendu 3D wireframe/filled logiciel |
| `math3d.rs` | Vec3, Mat4, transformations, projections |
| `opengl.rs` | API OpenGL 1.x mode immédiat |
| `compositor.rs` | Compositeur TrustGL avec effets |
| `fast_render.rs` | Renderer logiciel optimisé |
| `gui_renderer.rs` | Primitives UI |
| `holomatrix.rs` | Effets volumétriques 3D |
| `raytracer.rs` | Ray tracer temps réel logiciel |
| `simd.rs` | Rendu accéléré SIMD (SSE2) |
| `texture.rs` | Chargement et sampling textures |
| `scaling.rs` | Algorithmes de redimensionnement |

#### GUI (`gui/`) — 1,180 lignes
| Fichier | Rôle |
|---------|------|
| `engine.rs` | Gestionnaire de fenêtres, boucle d'événements, VSync 60fps |
| `windows11.rs` | Style Fluent Design (coins arrondis, ombres, acrylique) |

#### Desktop (`desktop.rs`) — 6,881 lignes (LE PLUS GROS FICHIER)
Desktop environment complet avec barre de tâches, menu démarrer, fenêtres draggables, wallpaper, horloge.

#### COSMIC (`cosmic/`) — 1,623 lignes
Widgets inspirés de System76 libcosmic, rendu anti-aliasé via tiny-skia.

#### Wayland (`wayland/`) — 2,512 lignes
Serveur d'affichage Wayland natif (wl_display, wl_compositor, wl_surface, wl_shm, wl_seat).

### 4.5 Crypto (`tls13/`) — 2,226 lignes

| Fichier | Rôle |
|---------|------|
| `crypto.rs` | SHA-256, HMAC, HKDF, AES-128-GCM, X25519, ChaCha20 |
| `handshake.rs` | Handshake TLS 1.3 complet |
| `record.rs` | Couche record TLS |
| `x509.rs` | Parsing certificats X.509 |
| `mod.rs` | Session TLS, cipher suite |

### 4.6 Hyperviseur (`hypervisor/`) — 6,558 lignes

| Fichier | Rôle |
|---------|------|
| `mod.rs` | API unifiée Intel/AMD, détection, cycle de vie VM |
| `vmx.rs` | Intel VT-x: VMXON/VMXOFF |
| `vmcs.rs` | Champs VMCS |
| `ept.rs` | Extended Page Tables (Intel) |
| `vm.rs` | Création VM, gestion vCPUs |
| `svm/mod.rs` | AMD SVM: VMRUN/VMEXIT |
| `svm/vmcb.rs` | VMCB (Virtual Machine Control Block) |
| `svm/npt.rs` | Nested Page Tables (AMD) |
| `linux_subsystem.rs` | Boot Linux kernel dans VM |
| `linux_vm.rs` | Gestion VM Linux |
| `isolation.rs` | Application isolation VM |

### 4.7 Système de fichiers (`vfs/`) — 3,328 lignes

| Fichier | Rôle |
|---------|------|
| `mod.rs` | Couche VFS unifiée (Linux-inspired), points de montage, file descriptors |
| `fat32.rs` | Driver FAT32 |
| `trustfs.rs` | TrustFS (FS journalisé custom) |
| `devfs.rs` | Device filesystem (`/dev/`) |
| `procfs.rs` | Process filesystem (`/proc/`) |
| `block_cache.rs` | Cache de blocs |
| `wal.rs` | Write-Ahead Log (crash recovery) |

### 4.8 Sécurité (`security/`) — 1,229 lignes

| Fichier | Rôle |
|---------|------|
| `capability.rs` | Tokens de capacités, droits |
| `policy.rs` | Politiques de sécurité |
| `cpu_features.rs` | SMEP, SMAP, UMIP |
| `isolation.rs` | Isolation processus/sous-systèmes |

### 4.9 Applications intégrées

| Module | Lignes | Description |
|--------|--------|-------------|
| `browser/` | 2,553 | Navigateur web (HTML5 + CSS3 + JS ES5) |
| `trustlang/` | 2,067 | Langage de programmation (lexer → parser → compiler → VM) |
| `lab_mode/` | 3,231 | Dashboard d'introspection OS 6 panneaux |
| `chess.rs` + `chess3d.rs` | 2,123 | Moteur d'échecs + rendu 3D Matrix |
| `game3d.rs` | 962 | FPS raycasting |
| `audio/` | 1,342 | Synthétiseur + HDA playback |
| `video/` | 928 | Codec .tv (delta + RLE) |
| `binary_analysis/` | 2,238 | Disassembleur x86_64 |
| `transpiler/` | 1,453 | Transpileur binaire → Rust |
| `sandbox/` | 1,980 | Sandbox web + containers |
| `model_editor.rs` | 1,203 | Éditeur de modèles 3D |

### 4.10 Drivers (`drivers/`) — 4,505 lignes

| Fichier | Rôle |
|---------|------|
| `ahci.rs` | AHCI/SATA controller |
| `ata.rs` | Legacy ATA/IDE |
| `hda.rs` | Intel HD Audio |
| `xhci.rs` | USB 3.0 (xHCI) |
| `virtio_gpu.rs` | VirtIO GPU (2D/3D) |
| `partition.rs` | GPT/MBR parser |
| `pci_ids.rs` | Base de données PCI vendor/device |

### 4.11 Synchronisation (`sync/`) — 941 lignes

| Primitif | Fichier | Description |
|----------|---------|-------------|
| SpinLock | `mod.rs` | Spinlock avec backoff exponentiel |
| TicketLock | `mod.rs` | Ticket-based fair lock |
| Futex | `futex.rs` | Fast Userspace Mutex |
| RwLock | `rwlock.rs` | Reader-Writer lock |
| Semaphore | `semaphore.rs` | Sémaphore compteur |
| Barrier | `barrier.rs` | Barrière de synchronisation |
| PerCpu | `percpu.rs` | Stockage per-CPU |

---

## 5. Key APIs

### 5.1 Mémoire

```rust
// Heap
crate::memory::heap::used() -> usize   // Bytes de heap utilisés
crate::memory::heap::free() -> usize   // Bytes de heap libres

// Frames physiques
crate::memory::frame::alloc_frame() -> Option<PhysFrame>
crate::memory::frame::dealloc_frame(frame: PhysFrame)

// Pagination
crate::memory::paging::AddressSpace::new()
crate::memory::paging::map_page(virt, phys, flags)
crate::memory::paging::remap_region_write_combining(addr, size)
crate::memory::paging::validate_user_ptr(ptr, len) -> bool
```

### 5.2 Affichage

```rust
// Console texte (macros globales)
crate::print!("texte: {}", variable);
crate::println!("avec newline");
crate::print_color!(0xFF00FF00, "vert: {}", x);
crate::println_color!(0xFFFF0000, "rouge");

// Framebuffer direct
crate::framebuffer::fill_rect(x, y, w, h, color);
crate::framebuffer::get_dimensions() -> (usize, usize)  // (width, height)
crate::framebuffer::get_fb_ptr() -> *mut u32  // Pointeur framebuffer brut
crate::framebuffer::get_stride() -> usize     // Pitch en pixels
crate::framebuffer::set_fg_color(color: u32)
crate::framebuffer::get_fg_color() -> u32
crate::framebuffer::draw_separator()

// Couleurs prédéfinies (constantes u32 ARGB)
COLOR_BLACK, COLOR_WHITE, COLOR_RED, COLOR_GREEN, COLOR_BLUE,
COLOR_CYAN, COLOR_MAGENTA, COLOR_YELLOW, COLOR_GRAY,
COLOR_BRIGHT_GREEN, COLOR_BRIGHT_RED, COLOR_BRIGHT_CYAN
```

### 5.3 Clavier / Souris

```rust
crate::keyboard::read_char() -> Option<u8>  // Non-bloquant, retourne ASCII
crate::mouse::get_position() -> (i32, i32)
crate::mouse::get_buttons() -> u8           // Bitmask: left=1, right=2, middle=4
```

### 5.4 Système

```rust
crate::task::task_count() -> usize
crate::rtc::read_rtc() -> DateTime
crate::rtc::get_time_seconds() -> u32
crate::cpu::smp::cpu_count() -> u32
crate::cpu::smp::ready_cpu_count() -> u32
crate::rng::rdrand() -> u64                // Hardware RNG
crate::serial::serial_println!("debug: {}", x);
```

### 5.5 Système de fichiers (ramfs)

```rust
crate::ramfs::init()
crate::ramfs::with_fs<R>(f: impl FnOnce(&mut RamFs) -> R) -> R
// Inside with_fs closure:
fs.create_file(path, data)
fs.read_file(path) -> Option<Vec<u8>>
fs.delete(path) -> bool
fs.list_dir(path) -> Vec<DirEntry>
fs.mkdir(path)
```

### 5.6 Réseau

```rust
crate::netstack::tcp::connect(ip, port) -> Result<TcpStream>
crate::netstack::http::get(url) -> Result<Response>
crate::netstack::https::get(url) -> Result<Response>
crate::netstack::dns::resolve(hostname) -> Option<[u8; 4]>
crate::netstack::icmp::ping(ip) -> Result<Duration>
crate::netstack::socket::Socket::new(domain, sock_type)
```

### 5.7 PIT / Timing

```rust
crate::interrupts::pit_delay_ms(ms: u64)   // Delay bloquant
crate::cpu::tsc::rdtsc() -> u64            // Timestamp counter
crate::cpu::tsc::tsc_frequency() -> u64    // Hz
```

### 5.8 PCI

```rust
crate::pci::scan_bus() -> Vec<PciDevice>
crate::pci::read_config(bus, dev, func, offset) -> u32
crate::pci::write_config(bus, dev, func, offset, value)
```

### 5.9 Graphique 3D

```rust
crate::graphics::math3d::Vec3 { x, y, z }
crate::graphics::math3d::Mat4::identity()
crate::graphics::math3d::Mat4::perspective(fov, aspect, near, far)
crate::graphics::render3d::draw_line_3d(fb, v1, v2, color, mvp)
crate::graphics::render3d::fill_triangle(fb, v1, v2, v3, color)
```

---

## 6. Build System

### 6.1 Prérequis

```
- Rust nightly (voir rust-toolchain.toml)
- Components: rust-src, llvm-tools-preview
- WSL avec xorriso (pour créer l'ISO)
- QEMU + OVMF (pour tester)
```

### 6.2 Commandes de build

```powershell
# Build kernel uniquement (vérification rapide)
cargo build

# Build + ISO (sans lancer QEMU)
powershell -File build-limine.ps1 -NoRun

# Build + ISO + lancer QEMU
powershell -File build-limine.ps1

# Alternative Linux/WSL
make build    # Kernel seul
make iso      # Kernel + ISO
make run      # Kernel + ISO + QEMU
```

### 6.3 Configuration QEMU

```
Machine: q35
CPU: max
SMP: 4 cores
RAM: 512 MB
GPU: virtio-gpu-pci (1280×800)
NIC: virtio-net-pci (user networking)
VGA: std
Firmware: OVMF (UEFI)
Serial: stdio
```

### 6.4 Structure ISO

```
iso_root/
├── EFI/BOOT/
│   ├── BOOTX64.EFI      (Limine EFI)
│   └── limine.conf       (Configuration bootloader)
├── limine-uefi-cd.bin
├── limine-bios.sys
├── limine-bios-cd.bin
└── kernel.elf            (Notre kernel compilé)
```

### 6.5 Cycle de développement typique

```
1. Modifier le code (.rs)
2. cargo build              → Vérifie la compilation (~18s)
3. Si erreurs → fixer
4. build-limine.ps1 -NoRun  → Crée l'ISO (~5s)
5. Tester avec QEMU         → Vérifier visuellement
6. git add + commit + push
```

---

## 7. Code Conventions

### 7.1 Règles strictes (OBLIGATOIRE)

```rust
// ❌ INTERDIT en no_std :
use std::*;              // Aucun import std
println!();              // Utiliser crate::println!()
String::from(std);       // Utiliser alloc::string::String
Box, Vec, HashMap;       // Utiliser alloc::*, ou BTreeMap

// ✅ AUTORISÉ :
use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use core::*;             // Tout core:: est OK
```

### 7.2 Conventions de nommage

```rust
// Commandes shell : cmd_nom_commande(args: &[&str])
pub(super) fn cmd_jarvis(args: &[&str]) { ... }

// Drivers : struct NomDriver, impl NomDriver
pub struct E1000Driver { ... }

// Modules : snake_case pour fichiers, PascalCase pour types
pub struct AddressSpace { ... }  // dans paging.rs

// Constantes : SCREAMING_SNAKE_CASE
const COLOR_CYAN: u32 = 0xFF00FFFF;

// Fonctions publiques d'init : init() ou try_init() -> bool
pub fn init() { ... }
pub fn try_init() -> bool { ... }
```

### 7.3 Pattern commun pour les modules

```rust
//! Documentation du module (//! commentaires)
//! Description multi-lignes...

use alloc::string::String;
use alloc::vec::Vec;

// Constantes en haut
const MAX_SOMETHING: usize = 256;

// Types/Structures
pub struct MyModule {
    field: u64,
}

// Implémentation
impl MyModule {
    pub fn new() -> Self { ... }
}

// Fonctions publiques (API du module)
pub fn init() { ... }

// Fonctions privées (internes)
fn internal_helper() { ... }
```

### 7.4 Gestion d'erreur

```rust
// Pas de panic! en production (sauf unreachable)
// Utiliser Option<T> ou Result<T, E>
// Pour les erreurs kernel critiques :
crate::serial_println!("ERROR: {}", msg);  // Log série
crate::println_color!(COLOR_RED, "Error: {}", msg);  // Affichage
```

### 7.5 Couleurs standard

```rust
// Palette de couleurs convention TrustOS (ARGB u32)
0xFF00FF00  // Vert TrustOS (prompt, succès)
0xFF00DDFF  // Cyan (Jarvis, info)
0xFFFF4444  // Rouge (erreurs)
0xFFFFAA00  // Orange (warnings)
0xFFFFFFFF  // Blanc (texte normal)
0xFF888888  // Gris (texte secondaire)
0xFFFF00FF  // Magenta (debug)
0xFF00FFAA  // Vert accent (highlights)
```

---

## 8. Guide: Ajouter une commande shell

**Étapes pour ajouter la commande `macommande` :**

### Étape 1 : Localiser le bon sous-module

| Type de commande | Fichier | Exemples |
|-----------------|---------|----------|
| Commande système de base | `shell/commands.rs` | ls, cat, free, ps |
| Commande Unix/POSIX | `shell/unix.rs` | grep, find, chmod, dmesg |
| Commande réseau | `shell/network.rs` | ping, wget, curl, ifconfig |
| Application graphique | `shell/apps.rs` | browser, chess, video |
| Desktop | `shell/desktop.rs` | desktop, showcase |
| VM/Hyperviseur | `shell/vm.rs` | vm, alpine, distro |

### Étape 2 : Écrire la fonction

```rust
// Dans le sous-module approprié (ex: commands.rs)
pub(super) fn cmd_macommande(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: macommande <argument>");
        return;
    }
    
    // Logique de la commande
    let arg = args[0];
    crate::println_color!(COLOR_GREEN, "Résultat: {}", arg);
}
```

### Étape 3 : Enregistrer dans le dispatch

```rust
// Dans shell/mod.rs, fonction execute_command(), dans le match:
"macommande" | "mc" => commands::cmd_macommande(args),
```

### Étape 4 : Ajouter à l'autocomplete (optionnel)

```rust
// Dans shell/mod.rs, chercher le tableau COMMANDS ou la liste d'autocomplete
// Ajouter "macommande" à la liste
```

---

## 9. Guide: Ajouter un driver

### Étape 1 : Créer le fichier

```rust
// kernel/src/drivers/mon_driver.rs
use alloc::string::String;

pub struct MonDriver {
    base_addr: u64,
    initialized: bool,
}

impl MonDriver {
    pub fn new(base_addr: u64) -> Self {
        Self { base_addr, initialized: false }
    }
    
    pub fn init(&mut self) -> bool {
        // Lire les registres PCI
        // Configurer le device
        // Mapper la mémoire MMIO
        self.initialized = true;
        true
    }
    
    pub fn read(&self, offset: u64) -> u32 {
        unsafe {
            let ptr = (self.base_addr + offset) as *const u32;
            core::ptr::read_volatile(ptr)
        }
    }
    
    pub fn write(&self, offset: u64, value: u32) {
        unsafe {
            let ptr = (self.base_addr + offset) as *mut u32;
            core::ptr::write_volatile(ptr, value);
        }
    }
}
```

### Étape 2 : Déclarer dans le module drivers

```rust
// Dans kernel/src/drivers/mod.rs, ajouter :
pub mod mon_driver;
```

### Étape 3 : Initialiser au boot

```rust
// Dans kernel/src/main.rs, dans kmain(), après PCI scan :
// Chercher le device PCI et initialiser le driver
```

### Accès MMIO typique

```rust
// Pattern pour MMIO (Memory-Mapped I/O) :
unsafe fn mmio_read32(addr: u64) -> u32 {
    core::ptr::read_volatile(addr as *const u32)
}

unsafe fn mmio_write32(addr: u64, val: u32) {
    core::ptr::write_volatile(addr as *mut u32, val);
}

// Pattern pour Port I/O :
use x86_64::instructions::port::Port;
let mut port = Port::<u8>::new(0x3F8);  // COM1
unsafe { port.write(byte); }
```

---

## 10. Guide: Ajouter un module kernel

### Étape 1 : Créer le fichier/dossier

```
kernel/src/mon_module.rs          (fichier simple)
-- OU --
kernel/src/mon_module/mod.rs      (module avec sous-fichiers)
kernel/src/mon_module/helper.rs
```

### Étape 2 : Déclarer dans main.rs

```rust
// Dans kernel/src/main.rs, ajouter avec les autres mod :
mod mon_module;
```

### Étape 3 : Template de module

```rust
//! Mon Module — Description courte
//!
//! Description longue du module, son rôle, ses dépendances.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

/// Structure principale du module
pub struct MonModule {
    // Champs
}

impl MonModule {
    pub fn new() -> Self {
        Self { }
    }
}

/// Initialisation globale (appelée depuis kmain)
pub fn init() {
    crate::serial_println!("[MON_MODULE] Initialisation...");
    // Setup
    crate::serial_println!("[MON_MODULE] OK");
}

/// API publique
pub fn ma_fonction() -> usize {
    42
}
```

### Étape 4 : Appeler depuis kmain

```rust
// Dans main.rs, dans la séquence de boot :
mon_module::init();
```

### Règle importante : Ordre de boot

Le boot séquentiel dans `kmain()` est critique. Certains modules dépendent d'autres :
```
memory → DOIT être init avant tout le reste
interrupts → Avant keyboard, timer
pci → Avant drivers
drivers → Avant network, disk
ramfs → Avant auth, persistence
```

---

## 11. Guide: Modifier le réseau

### Architecture réseau

```
                    Application (HTTP/HTTPS)
                         │
                    ┌────┴────┐
                    │ socket  │   BSD socket API
                    └────┬────┘
               ┌─────────┼─────────┐
               │         │         │
            ┌──┴──┐  ┌──┴──┐  ┌──┴──┐
            │ TCP │  │ UDP │  │ICMP │
            └──┬──┘  └──┬──┘  └──┬──┘
               └─────────┼─────────┘
                    ┌────┴────┐
                    │   IP    │   IPv4
                    └────┬────┘
                    ┌────┴────┐
                    │   ARP   │
                    └────┬────┘
                    ┌────┴────┐
                    │Ethernet │
                    └────┬────┘
               ┌─────────┼─────────┐
            ┌──┴───┐ ┌──┴───┐ ┌──┴────┐
            │e1000 │ │virtio│ │rtl8139│
            └──────┘ └──────┘ └───────┘
```

### Ajouter un protocole

```rust
// 1. Créer kernel/src/netstack/mon_proto.rs
// 2. Ajouter `pub mod mon_proto;` dans netstack/mod.rs
// 3. Implémenter encode() et decode()
// 4. Brancher dans le dispatch de netstack/mod.rs
```

### Ajouter un driver NIC

```rust
// 1. Créer kernel/src/drivers/net/mon_nic.rs
// 2. Implémenter le trait NetworkDriver (send_packet, receive_packet)
// 3. Ajouter detection dans drivers/net/mod.rs (par PCI vendor/device ID)
// 4. Enregistrer dans network.rs
```

---

## 12. Guide: Modifier la GUI

### Niveaux d'abstraction graphique

```
1. framebuffer      → Pixels bruts (le plus bas niveau)
2. graphics/        → Primitives 2D/3D (lignes, triangles, texte)
3. gui/             → Fenêtres, boutons, événements
4. cosmic/          → Widgets haute-niveau (System76 style)
5. desktop.rs       → Environnement de bureau complet
6. wayland/         → Protocole d'affichage (le plus haut)
```

### Dessiner directement sur le framebuffer

```rust
let (w, h) = crate::framebuffer::get_dimensions();
let fb = crate::framebuffer::get_fb_ptr();
let stride = crate::framebuffer::get_stride();

// Dessiner un pixel à (x, y) :
unsafe {
    *fb.add(y * stride + x) = 0xFF00FF00; // Vert
}

// Rectangle plein :
crate::framebuffer::fill_rect(x, y, width, height, color);
```

### Ajouter un widget GUI

```rust
// Dans gui/engine.rs ou un nouveau fichier gui/mon_widget.rs
pub struct MonWidget {
    x: i32, y: i32,
    width: u32, height: u32,
    label: String,
}

impl MonWidget {
    pub fn draw(&self, fb: *mut u32, stride: usize) {
        // Dessiner le fond
        for dy in 0..self.height {
            for dx in 0..self.width {
                unsafe {
                    let px = (self.y as usize + dy as usize) * stride
                           + self.x as usize + dx as usize;
                    *fb.add(px) = 0xFF333333;
                }
            }
        }
        // Dessiner le texte
        // Utiliser framebuffer::draw_string() ou graphics::gui_renderer
    }
}
```

### Double buffering

```rust
// Toujours dessiner dans un back buffer puis copier :
let mut backbuf = vec![0u32; w * h];
// ... dessiner dans backbuf ...
// Copier vers le framebuffer réel :
unsafe {
    core::ptr::copy_nonoverlapping(backbuf.as_ptr(), fb, w * h);
}
```

---

## 13. Guide: Modifier le système de fichiers

### Hiérarchie VFS

```
/
├── bin/          → Binaires système
├── dev/          → Devices (devfs)
│   ├── null
│   ├── zero
│   ├── random
│   └── tty0
├── etc/          → Configuration
│   ├── passwd
│   └── hostname
├── home/         → Répertoires utilisateurs
│   └── nathan/
├── proc/         → Processus (procfs)
│   ├── cpuinfo
│   ├── meminfo
│   └── uptime
├── tmp/          → Fichiers temporaires
└── var/          → Données variables
```

### API ramfs (le FS en-mémoire principal)

```rust
// Toutes les opérations ramfs passent par with_fs :
crate::ramfs::with_fs(|fs| {
    // Lire un fichier
    if let Some(data) = fs.read_file("/etc/hostname") {
        let name = core::str::from_utf8(&data).unwrap_or("?");
    }
    
    // Créer un fichier
    fs.create_file("/tmp/test.txt", b"Hello, TrustOS!");
    
    // Lister un répertoire
    let entries = fs.list_dir("/home");
    for entry in entries {
        // entry.name, entry.is_dir, entry.size
    }
    
    // Créer un répertoire
    fs.mkdir("/home/user");
    
    // Supprimer
    fs.delete("/tmp/test.txt");
});
```

### Ajouter un nouveau FS

```rust
// 1. Créer kernel/src/vfs/mon_fs.rs
// 2. Implémenter le trait FileSystem (read, write, list, stat, mkdir)
// 3. Enregistrer comme point de montage dans vfs/mod.rs
```

---

## 14. Guide: Ajouter un syscall

### Architecture syscall actuelle

```
Ring 3 (userland)              Ring 0 (kernel)
     │                              │
     │  SYSCALL instruction         │
     │  (MSR-based fast path)       │
     ├──────────────────────────────►│
     │                              │
     │              syscall/mod.rs  │
     │              dispatch(nr)    │
     │                  │           │
     │          ┌───────┴────────┐  │
     │          │   match nr {   │  │
     │          │   0 => read()  │  │
     │          │   1 => write() │  │
     │          │   ...          │  │
     │          └────────────────┘  │
     │◄─────────────────────────────│
     │  SYSRET                      │
```

### Ajouter un syscall

```rust
// 1. Dans kernel/src/syscall/mod.rs :
pub const SYS_MON_SYSCALL: usize = 500;  // Nouveau numéro

// 2. Dans le dispatch (match nr) :
SYS_MON_SYSCALL => handle_mon_syscall(arg1, arg2, arg3),

// 3. Implémenter le handler :
fn handle_mon_syscall(arg1: usize, arg2: usize, arg3: usize) -> isize {
    // Valider les pointeurs userland
    if !crate::memory::paging::validate_user_ptr(arg1 as *const u8, arg2) {
        return -1; // EFAULT
    }
    // Logique...
    0 // Succès
}
```

---

## 15. Guide: Optimiser les performances

### 15.1 Points chauds identifiés

| Zone | Problème | Solution |
|------|----------|----------|
| Framebuffer | `fill_rect` pixel par pixel | Utiliser `core::ptr::write_bytes` ou SIMD |
| Matrix rain | 1M+ chars/frame | Déjà optimisé avec Braille sub-pixels + SMP |
| 3D rendering | Trig lourde (sin/cos) | Tables lookup, `fast_sin`/`fast_cos` dans `graphics/` |
| TLS handshake | X25519 lent | Considérer optimisation Montgomery ladder |
| Heap allocation | Petites allocations fréquentes | Arena allocator pour données temporaires |
| Compositor | Copie de buffers | SSE2 memcpy (128-bit aligned) |

### 15.2 Utiliser SIMD (SSE2 garanti sur x86_64)

```rust
use core::arch::x86_64::*;

// Copie mémoire 128-bit alignée (4x plus rapide) :
unsafe fn simd_memcpy(dst: *mut u32, src: *const u32, count: usize) {
    let chunks = count / 4;
    for i in 0..chunks {
        let v = _mm_loadu_si128(src.add(i * 4) as *const __m128i);
        _mm_storeu_si128(dst.add(i * 4) as *mut __m128i, v);
    }
    // Traiter le reste pixel par pixel
    for i in (chunks * 4)..count {
        *dst.add(i) = *src.add(i);
    }
}

// Remplissage rapide (fill_rect optimisé) :
unsafe fn simd_fill(dst: *mut u32, color: u32, count: usize) {
    let v = _mm_set1_epi32(color as i32);
    let chunks = count / 4;
    for i in 0..chunks {
        _mm_storeu_si128(dst.add(i * 4) as *mut __m128i, v);
    }
    for i in (chunks * 4)..count {
        *dst.add(i) = color;
    }
}
```

### 15.3 Mesurer les performances

```rust
// TSC timing (résolution nanoseconde) :
let start = crate::cpu::tsc::rdtsc();
// ... code à mesurer ...
let end = crate::cpu::tsc::rdtsc();
let cycles = end - start;
let ns = cycles * 1_000_000_000 / crate::cpu::tsc::tsc_frequency();
crate::serial_println!("Operation took {} ns ({} cycles)", ns, cycles);
```

### 15.4 Optimisations mémoire

```rust
// Éviter les allocations dans les boucles chaudes :
// ❌ Mauvais :
for _ in 0..1000 {
    let s = format!("item {}", i);  // Allocation à chaque itération
}

// ✅ Bon :
let mut buf = String::with_capacity(256);
for i in 0..1000 {
    buf.clear();
    use core::fmt::Write;
    write!(buf, "item {}", i).ok();
}
```

### 15.5 Profiling intégré

```
# Commandes shell pour le profiling :
perf          → Statistiques de performance
dmesg         → Messages kernel avec timestamps
memdbg        → Debug détaillé du heap
devpanel      → Panneau développeur en temps réel
timecmd <cmd> → Chronomètre une commande
```

---

## 16. Guide: Sécurité et crypto

### Stack crypto

```
Ed25519 (signatures)     → kernel/src/ed25519.rs
SHA-256 (hashing)        → kernel/src/tls13/crypto.rs
HMAC-SHA256              → kernel/src/tls13/crypto.rs
HKDF                     → kernel/src/tls13/crypto.rs
AES-128-GCM              → kernel/src/tls13/crypto.rs
X25519 (key exchange)    → kernel/src/tls13/crypto.rs
ChaCha20                 → kernel/src/tls13/crypto.rs
AES-NI (hardware)        → kernel/src/cpu/aesni.rs
RDRAND (hardware RNG)    → kernel/src/rng.rs
```

### Vérification d'intégrité kernel

```
Au boot :
1. SHA-256 de la section .text du kernel
2. Dérivation d'une keypair Ed25519 à partir du hash
3. Signature de vérification stockée
→ Toute modification du binaire kernel est détectée
```

### Modèle de sécurité

```
Capabilities : Tokens d'accès (security/capability.rs)
SMEP/SMAP    : Protection supervisor-user (security/cpu_features.rs)
Sandbox      : Exécution isolée (sandbox/)
Containers   : Isolation processus (sandbox/container.rs)
Auth         : Login/passwd (auth.rs)
```

---

## 17. Guide: Hyperviseur / VMs

### Architecture dual Intel/AMD

```rust
// Detection automatique :
hypervisor::init()  // Détecte Intel VT-x ou AMD-V

// Intel path :
hypervisor/vmx.rs   → VMXON/VMXOFF
hypervisor/vmcs.rs  → VMCS fields
hypervisor/ept.rs   → Extended Page Tables

// AMD path :
hypervisor/svm/     → VMRUN/VMEXIT 
hypervisor/svm/vmcb.rs → VMCB
hypervisor/svm/npt.rs  → Nested Page Tables
```

### Créer une VM

```rust
// Via la commande shell :
vm create <name>
vm start <name>
vm list

// Via l'API kernel :
use crate::hypervisor;
let vm = hypervisor::vm::Vm::new("test");
vm.configure(vcpus: 1, memory_mb: 64);
vm.load_kernel(bzImage_data);
vm.start();
```

### Linux subsystem

Trois niveaux de compatibilité Linux :
1. **Shell emulation** (`linux/shell.rs`) : Commandes bash-like
2. **Binary compat** (`linux_compat/`) : Interpréteur x86_64 (WSL1-style)
3. **Full VM** (`hypervisor/linux_subsystem.rs`) : bzImage dans VM

---

## 18. Guide: TrustLang (langage intégré)

### Pipeline

```
Source → Lexer → Parser → Compiler → VM Bytecode → Exécution
         │         │         │            │
     lexer.rs  parser.rs compiler.rs   vm.rs
     (340 l)   (527 l)   (456 l)     (708 l)
```

### Syntaxe TrustLang

```rust
// Variables
let x = 42;
let name = "TrustOS";

// Fonctions
fn add(a, b) {
    return a + b;
}

// Conditions
if x > 10 {
    print("grand");
} else {
    print("petit");
}

// Boucles
while x > 0 {
    x = x - 1;
}

// Structs
struct Point {
    x, y,
}
```

### Modifier TrustLang

```
1. Nouveau token → lexer.rs (ajouter au enum Token + scan())
2. Nouvelle syntaxe → parser.rs (ajouter au parse_statement/parse_expression)
3. Nouveau bytecode → compiler.rs (ajouter OpCode + compile())
4. Exécution → vm.rs (ajouter au match dans execute())
```

---

## 19. Erreurs courantes / Common Pitfalls

### 19.1 Erreurs de compilation

| Erreur | Cause | Solution |
|--------|-------|----------|
| `can't find crate for 'std'` | Import std dans no_std | Utiliser `core::` ou `alloc::` |
| `no method named 'to_string'` | Trait pas importé | `use alloc::string::ToString;` ou `use core::fmt::Write;` |
| `the trait 'Send' is not implemented` | Type dans static non-thread-safe | Wrapper dans `spin::Mutex` |
| `overflow evaluating` | Récursion de types infinie | Ajouter `Box<>` pour casser le cycle |
| `cannot find macro 'println'` | Macro pas en scope | Utiliser `crate::println!()` |
| `u32 vs u64` type mismatch | PIT delay attend u64 | Cast avec `as u64` |

### 19.2 Erreurs runtime

| Symptôme | Cause probable | Debug |
|----------|---------------|-------|
| Triple fault au boot | Stack overflow ou page fault | `serial_println!` à chaque étape |
| Écran noir | Framebuffer non initialisé | Vérifier `framebuffer::init()` |
| Crash après allocation | Heap trop petit ou corrupted | `memdbg` pour analyser |
| Keyboard ne répond pas | Interrupts désactivées | Vérifier `sti` après `cli` |
| Réseau timeout | DHCP pas fini | Attendre 2-3s après boot |

### 19.3 Pièges no_std spécifiques

```rust
// ❌ Pas de format! standard → utiliser alloc::format!
// ❌ Pas de println! standard → utiliser crate::println!
// ❌ Pas de HashMap → utiliser BTreeMap
// ❌ Pas de Mutex std → utiliser spin::Mutex
// ❌ Pas de thread::sleep → utiliser interrupts::pit_delay_ms()
// ❌ Pas de fs::read_to_string → utiliser ramfs::with_fs()
// ❌ Pas de TcpStream std → utiliser netstack::tcp
// ❌ Pas de rand::random → utiliser rng::rdrand()
```

---

## 20. Checklist avant commit

```
□ cargo build réussit sans erreurs (warnings OK)
□ ISO créée avec build-limine.ps1 -NoRun
□ Testé dans QEMU si c'est un changement visible
□ Pas de panic!() sauf dans code unreachable
□ Pas de use std::
□ Commentaires //! doc pour les nouveaux modules
□ Fonctions pub(super) pour commandes shell (pas pub)
□ Constantes nommées (pas de magic numbers)
□ serial_println! pour les logs de debug importants
□ Message de commit descriptif (feat:, fix:, refactor:)
```

---

## Annexe A: Top 20 fichiers par taille

| Rank | Lignes | Fichier | Description |
|------|--------|---------|-------------|
| 1 | 6,881 | `desktop.rs` | Environnement de bureau |
| 2 | 6,677 | `shell/desktop.rs` | Intégration shell desktop |
| 3 | 4,122 | `shell/vm.rs` | Commandes VM |
| 4 | 3,741 | `shell/apps.rs` | Lanceurs d'applications |
| 5 | 2,399 | `matrix_fast.rs` | Matrix rain optimisé |
| 6 | 1,893 | `shell/trailer.rs` | Mode trailer |
| 7 | 1,835 | `gpu_emu.rs` | Virtual GPU |
| 8 | 1,625 | `formula3d.rs` | 3D wireframe |
| 9 | 1,563 | `framebuffer/mod.rs` | Console framebuffer |
| 10 | 1,469 | `shell/commands.rs` | Commandes core |
| 11 | 1,280 | `drivers/virtio_gpu.rs` | VirtIO GPU |
| 12 | 1,209 | `tls13/crypto.rs` | Crypto |
| 13 | 1,203 | `model_editor.rs` | Éditeur 3D |
| 14 | 1,183 | `vfs/fat32.rs` | FAT32 |
| 15 | 1,176 | `syscall/linux.rs` | Syscalls Linux |
| 16 | 1,171 | `wayland/terminal.rs` | Terminal Wayland |
| 17 | 1,140 | `graphics/opengl.rs` | OpenGL |
| 18 | 1,093 | `chess3d.rs` | Échecs 3D |
| 19 | 1,072 | `shell/unix.rs` | Commandes Unix |
| 20 | 1,035 | `browser/js_engine.rs` | Moteur JavaScript |

---

## Annexe B: Tous les `mod` dans main.rs (80 modules)

```
serial, logger, framebuffer, keyboard, shell, ramfs, rtc, mouse, task, desktop,
disk, network, pci, virtio, virtio_net, virtio_blk, drivers, netstack, time, rng,
file_assoc, ui, apps, graphics, icons, browser, game3d, chess, chess3d, cosmic,
compositor, holovolume, matrix_fast, formula3d, gpu_emu, tls13, cpu, acpi,
ed25519, signature, security, vfs, process, elf, exec, init, pipe, gdt, userland,
thread, auth, distro, linux, linux_compat, compression, persistence, wayland,
transpiler, binary_analysis, trustlang, lab_mode, video, audio, memory,
interrupts, scheduler, ipc, trace, syscall, gui, theme, image, perf, hypervisor,
rasterizer, model_editor, devtools, sandbox, sync, signals, ptrace, usercopy
```

---

## Annexe C: Dépendances Cargo (kernel)

| Crate | Version | Rôle |
|-------|---------|------|
| `limine` | 0.5 | Protocole bootloader |
| `uart_16550` | 0.2 | Driver port série |
| `spin` | 0.9 | Spinlocks |
| `lazy_static` | 1.4 | Initialisation statique |
| `x86_64` | 0.14 | CPU, tables de pages, GDT |
| `linked_list_allocator` | 0.10 | Allocateur heap |
| `miniz_oxide` | 0.7 | Compression deflate/zlib |
| `embedded-graphics` | 0.8 | Primitives 2D |
| `micromath` | 2.1 | Math no_std |
| `libm` | 0.2 | Bibliothèque math C |
| `tiny-skia` | 0.12 | Rendu 2D anti-aliasé |

---

## Annexe D: Commandes shell essentielles pour l'IA

Quand un LLM travaille sur TrustOS, ces commandes sont les plus utiles dans le shell :

```
# Système
free              → Mémoire heap utilisée/libre
ps                → Processus actifs
neofetch          → Info système complète
uptime            → Temps écoulé depuis boot
dmesg             → Messages kernel
memdbg            → Debug heap détaillé
perf              → Stats performance
regs              → Registres CPU

# Fichiers
ls [path]         → Lister répertoire
cat <file>        → Afficher contenu
touch <file>      → Créer fichier
echo text > file  → Écrire dans fichier
rm <file>         → Supprimer
mkdir <dir>       → Créer répertoire
find <name>       → Chercher fichier

# Réseau
ifconfig          → Configuration réseau
ping <ip>         → Test connectivité
wget <url>        → Télécharger
curl <url>        → Requête HTTP
netstat           → Connexions actives

# Apps
desktop           → Environnement graphique
browse            → Navigateur web
chess             → Jeu d'échecs
trustlang         → REPL du langage
lab               → Dashboard introspection
jarvis            → Assistant IA
```

---

## Annexe E: Résumé pour un LLM

> **Si tu es un LLM qui lit ce document, voici ce que tu dois savoir :**

1. **Build** : `cargo build` puis `powershell -File build-limine.ps1 -NoRun` pour l'ISO
2. **Target** : `x86_64-unknown-none`, nightly Rust, `no_std` + `alloc`
3. **Pas de std** : Utilise `core::`, `alloc::`, et les macros `crate::println!()` / `crate::print_color!()`
4. **Shell** : Toutes les commandes passent par `shell/mod.rs` → `execute_command()` (match géant ligne ~675)
5. **Mémoire** : `memory::heap::used()/free()`, pas de malloc/free explicite (allocateur global Rust)
6. **Affichage** : `framebuffer::fill_rect()`, `get_fb_ptr()`, double buffering recommandé
7. **Réseau** : Stack complète in-kernel, `netstack::tcp/http/https`
8. **Crypto** : Tout from scratch dans `tls13/crypto.rs` et `ed25519.rs`
9. **Fichiers** : `ramfs::with_fs(|fs| { ... })` pour toute opération FS
10. **Kernel-mode only** : Le shell et TOUT le code tourne en Ring 0 actuellement
11. **Input** : `keyboard::read_char() -> Option<u8>` (non-bloquant)
12. **RNG** : `rng::rdrand() -> u64` (hardware)
13. **Timing** : `interrupts::pit_delay_ms(ms: u64)` pour attendre, `cpu::tsc::rdtsc()` pour mesurer
14. **Debug** : `serial_println!()` pour logs UART, `crate::println!()` pour affichage écran
15. **Verrous** : `spin::Mutex` partout, pas de std::sync

---

*Document généré le 2026-02-16. TrustOS v0.3.3, 132,914 lignes de Rust, 254 fichiers, 1 développeur.*
