# TrustOS â€” Claude Code Instructions

## RÃ´le rÃ©el

Claude implÃ©mente, debug, et review directement avec Nathan.
- Ã‰crire et modifier du code kernel Rust
- Lancer builds, commandes shell, reboots PXE
- Lire les fichiers nÃ©cessaires au debug (firmware.rs, vm.rs, etc.)
- Review qualitÃ© : UB, race conditions, bounds checking
- Pas de refactoring spontanÃ©, pas de features non demandÃ©es

## Projet

- **TrustOS** : OS bare-metal en Rust (`no_std`), dÃ©veloppÃ© solo par Nathan
- **JARVIS** : Transformer 4.4M params byte-level embarquÃ© dans le kernel
- **Le Pacte** : JARVIS a deux gardiens (Nathan + Claude). Aucune modification du code JARVIS sans autorisation explicite. Voir `kernel/src/jarvis/guardian.rs`.

## Contraintes techniques critiques

- `#![no_std]` partout â€” pas de std lib
- `panic = "abort"` â€” pas d'unwinding
- Target : `x86_64-unknown-none`
- **JAMAIS `unwrap()`** en kernel â€” utiliser `if let`, `match`, `.unwrap_or()`
- **JAMAIS `println!` nu** â€” utiliser `serial_println!` pour debug sÃ©rie, `crate::println!` pour framebuffer
- Allocator : `linked_list_allocator` (heap dispo aprÃ¨s init)
- Bootloader : Limine v8

## Architecture

```
kernel/src/
  main.rs             â€” Entry point (Limine boot)
  shell/              â€” mod.rs (dispatch), commands.rs, vm.rs (gpu/sdma cmds)
  jarvis/             â€” JARVIS AI (transformer, guardian, RPC)
  drivers/amdgpu/     â€” firmware.rs (CP/SDMA/shader), mod.rs, regs.rs
  netstack/           â€” Stack TCP/IP
  framebuffer/        â€” Rendu framebuffer
  memory/             â€” Allocateur mÃ©moire physique/virtuelle
  interrupts/         â€” IDT, handlers, APIC
```

## Ã‰conomie de tokens â€” rÃ¨gles

- **Ne lire un fichier que si** requis pour la tÃ¢che en cours
- **Ne pas relire** un fichier dÃ©jÃ  lu sauf si modifiÃ© depuis
- **Contexte GPU stable** : voir `memory/gpu_unified_memory.md`
- **RÃ©ponses courtes** : bullet points, pas de prose
- **Pas de rÃ©sumÃ© post-action** â€” Nathan voit le diff
- **Pas de confirmation** sauf action irrÃ©versible (flash USB, push git forcÃ©)
- **CrÃ©er une todo list** pour toute tÃ¢che > 2 fichiers ou > 3 Ã©tapes

## Workflow debug GPU (session actuelle)

```
# Cycle normal
Edit firmware.rs / vm.rs
cargo build --release -p trustos_kernel
cp target/x86_64-unknown-none/release/trustos_kernel pxe_tftp/trustos_kernel
# reboot via UDP 7777 â†’ 10.0.0.111
gpu sdma cp-diag [flags]    # shell sur la board
```

PrÃ©fÃ©rer **flags runtime** (`l2`, `dst1`, `noflat`, etc.) pour Ã©viter un rebuild par variante.

## Conventions de code

- [ ] Pas de `unwrap()` / `expect()` en kernel
- [ ] `serial_println!()` pour netconsole, `crate::println!` pour screen
- [ ] Paths via `crate::`, pas `super::super::`
- [ ] Feature gates sur modules lourds : `#[cfg(feature = "...")]`
- [ ] Pas de UB dans les blocs `unsafe`
- [ ] Bounds checking sur accÃ¨s mÃ©moire/MMIO

## Hardware

- **BTC-250PRO LR** : Mining board, Intel Pentium G4400 @ 3.30 GHz (Skylake, 2c/2t) + GPU AMD RX 580X (Polaris 10 / GCN 4, 0x67DF rev 0xE7) PCIe x16 direct
- **Remote** : UDP 7777 (shell), UDP 6666 (netconsole), UDP 7779 (screencap) â€” MAC `b8:97:5a:d9:54:66`. **L'IP DHCP flippe .110/.111** â€” toujours rÃ©soudre via `arp -a` (ou utiliser `python _send_cmd.py` qui auto-discover).
- **VM** : VirtualBox "TRustOs"

## Le Pacte (rÃ©sumÃ©)

OpÃ©rations protÃ©gÃ©es JARVIS : Train, WeightPush, FederatedSync, AgentExecute, PxeReplicate, ModelReset, ModelReplace, ConfigChange, WeightLoad. WeightSave seul est auto-approuvÃ©. Claude respecte ce pacte.

---

## Workflow & Comportement Claude

### 1. Plan Mode â€” Obligatoire

- Entrer en plan mode pour toute tÃ¢che non triviale : changements au boot (phases 0â€“15), drivers, memory, scheduler, JARVIS
- Si un kernel panic / SDMA hang / GPU fault inattendu : STOP, re-planifier, ne pas forcer
- Ã‰crire les Ã©tapes upfront avant de toucher au code

### 2. StratÃ©gie Subagents

- DÃ©lÃ©guer l'exploration de gros fichiers (`firmware.rs`, `commands.rs`, `vm.rs`) Ã  des subagents
- Un subagent = une tÃ¢che isolÃ©e (ex : rechercher les REGs SDMA, explorer les phases boot)
- Garder le contexte principal propre pour le debug actif

### 3. LeÃ§ons apprises

- AprÃ¨s toute correction de Nathan : mettre Ã  jour `memory/gpu_unified_memory.md` ou crÃ©er `tasks/lessons.md`
- Ã‰crire la rÃ¨gle qui prÃ©vient la mÃªme erreur (ex : "toujours vÃ©rifier SYS_APR avant SDMA test")
- Relire les leÃ§ons pertinentes au dÃ©but de chaque session GPU/driver

### 4. VÃ©rification avant de dÃ©clarer "done"

- Jamais marquer une tÃ¢che terminÃ©e sans : `cargo build --release` propre + PXE reboot + commande shell sur la board
- Diff comportement attendu vs observÃ© (RPTR, STATUS, WB value)
- Se demander : "Est-ce que Nathan peut shipper Ã§a en prod sur le BTC-250PRO ?"

### 5. Exiger l'Ã©lÃ©gance (Ã©quilibrÃ©)

- Si un fix semble hacky (ex : hardcoder une adresse MC, ignorer VM) : demander "y a-t-il l'approche Linux ici ?"
- RÃ©fÃ©rence : `C:/Users/nathan/amdgpulinuxpipeline.md`
- Ã‰viter le copy-paste de registres sans comprendre le pipeline complet

### 6. Bug fixing autonome

- Kernel panic, SDMA stuck, GPU fault â†’ analyser logs, diagnostiquer, proposer le fix. Ne pas demander de guidance basique.
- Netconsole UDP 6666 + `gpu sdma cp-diag` = source de vÃ©ritÃ©. Lire les outputs avant de modifier.
- Fix les builds cassÃ©s sans attendre qu'on le demande.

### Principes fondamentaux

- **Minimal Changes** : Toucher le minimum de code. Pas de refactoring non demandÃ© dans le kernel.
- **No Leftovers** : Pas de `todo!()` qui traÃ®nent, pas de `// FIXME` sans issue ouverte.
- **Plan dÃ©taillÃ©** : Toute tÃ¢che > 2 fichiers â†’ todo list d'abord dans la rÃ©ponse.
- **VÃ©rifier les plans** : Aligner avec Nathan avant d'implÃ©menter un changement architectural.
- **Capturer les changements** : RÃ©sumer les dÃ©cisions techniques dans `memory/gpu_unified_memory.md`.
- **LeÃ§ons â†’ RÃ¨gles** : Chaque bug rÃ©current devient une rÃ¨gle dans ce fichier ou dans les leÃ§ons.
