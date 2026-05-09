# TrustOS â€” Work Journal

> Append-only log. One entry per non-trivial task. Newest at top.
> Visible by every AI agent (Claude Code, Copilot, Cursor, Codex, â€¦) and
> tracked in git history. Single source of truth for "what happened when".
>
> Each entry: date, scope, what was done, result, follow-up.
> Keep entries short (3â€“6 lines). No prose. No marketing.

## 2026-05-08 — Porte 1 HAL Phase G : riscv64 build VERT (premier essai)
- scope: rustup target add riscv64gc-unknown-none-elf ; cargo build --release -p trustos_kernel --target riscv64gc-unknown-none-elf
- did: zero migration nécessaire — tous les stubs/cfg gates des phases A+B+E couvraient déjà riscv64 via `#[cfg(not(target_arch = "x86_64"))]`
- result: 0 errs, ELF 12.4 MB, 1m23s. **3 arches verts simultanément** : x86_64 + aarch64 + riscv64. Porte 1 = 100% côté build.
- next: Phase F-bis (vrai boot E2E QEMU aarch64) requires Limine aarch64 ISO + OVMF.fd firmware. Reportée. Porte 2 (GPU stack stable Polaris) au retour board.

## 2026-05-08 — Porte 1 HAL Phase E : aarch64 build VERT (54 → 0)
- scope: stubs/{acpi,cpu}.rs réécrits parité totale, drivers/net/mod.rs (register_drivers gated), bulk migrations memory/{swap,paging,mod,cow}.rs + syscall/linux.rs + interrupts/handlers.rs + drivers/amdgpu/{mod,firmware}.rs + hwscan/timing.rs + exec.rs + shell/{vm,desktop}.rs + jarvis/micro_model.rs
- did: ACPI structs (FadtInfo+flags+méthodes, IntSourceOverride, LocalApic enabled/online_capable, HpetInfo period_fs/frequency, GenericAddress) ; CpuCapabilities.fma ; bulk asm! → HAL portable : invlpg→arch::flush_tlb, cr3 read/write→arch::{read,write}_page_table_root, mfence→fence(SeqCst), pause→spin_loop, pushfq IF→arch::are_interrupts_enabled
- result: cargo build aarch64-unknown-none = 0 errs (Finished release). cargo build x86_64 = 0 errs (Finished 1m46s). PORTE 1 ouverte côté build aarch64.
- next: Phase F = QEMU virt aarch64 boot test (qemu-system-aarch64 absent env parents, à faire env home). Phase G = riscv64.

## 2026-05-08 — Porte 1 HAL Phase B : 316 → 54 errs aarch64 (-83%)
- scope: stubs/apic.rs (watchdog), drivers/net/{e1000,rtl8139,rtl8169,iwl4965}, drivers/{nvidia,amdgpu/firmware}, drivers/amdgpu/gpu_train.rs, jarvis/{trace/mod,trace/pmu,micro_model}, trustlang/native.rs, signals/ptrace gate + new stubs/signals.rs
- did: watchdog stubs no-op; PCI legacy NICs cfg-gated x86; iwl4965 portable via crate::debug::post_code; gpu_train _rdtsc → crate::arch::timestamp; signals stub portable; POST=0 inline asm gated; pmu module gated; execute_native split x86/no-x86 stub
- result: aarch64 errs 316 → 54 (-83%). x86_64 build vert. Reste 54 errs = bug ACPI structs FadtInfo/IntSourceOverride/LocalApic/HpetInfo (champs manquants), indépendant portage
- next: Phase E = fix structs ACPI → aarch64 vert. Phase F = QEMU virt aarch64 boot.

## 2026-05-08 — Porte 1 HAL Phase A : baseline aarch64 + 1ères migrations (Axe 1)
- scope: kernel/src/{memory/heap.rs, shell/commands.rs, shell/apps.rs,
  trustlang/tests.rs, debug/mod.rs}.
- did: découvert que `kernel/src/arch/` HAL existe déjà avec aarch64+riscv64
  squelettes sérieux. Migré 10 sites `core::arch::x86_64::_rdtsc()` vers
  `crate::arch::timestamp()` dans modules portables. Ajouté `use alloc::vec;`
  manquant dans debug/mod.rs.
- result: build x86_64 reste vert (1m 36s). Build aarch64 baseline mesuré :
  316 erreurs en 5 buckets (78 watchdog, 44 asm x86, 30 process registres,
  15 arch x86 leak, 9 FADT). Pattern + plan documentés.
- next: Phase B = feature-gate modules x86-bound (electro, hypervisor, aesni,
  gpu_train CR3). Voir `/memories/repo/hal_migration_pattern.md`.

## 2026-05-08 — USB driver layer + mt7921u skeleton (Phase 1)
- scope: kernel/src/drivers/xhci.rs (extend descriptor parser to collect bulk
  endpoints for vendor class 0xFF, auto-configure them; new public wrappers
  `usb_control_in`, `usb_control_out_nodata`, `usb_configure_bulk_endpoints`,
  `find_by_vid_pid`), kernel/src/drivers/net/mt7921u.rs (new — VID/PID table
  for ASUS USB-AX56 + reference dongles, `probe()` that reads chip ID via
  vendor MULTI_READ control transfer), kernel/src/drivers/net/mod.rs,
  kernel/src/shell/commands.rs (`wifi probe-usb` subcommand).
- did: minimal API surface so mt76 family can drive xHCI without re-implementing
  USB plumbing. Skipped a separate façade module (YAGNI) — xhci public fns
  ARE the API.
- result: build clean (1m 43s). qemu-selftest PASS unchanged (parsers /
  handshake / ccmp / ccmp-smoke). `wifi probe-usb` returns "no MT7921U device
  found" on QEMU (expected — VM has no USB WiFi).
- next: real test on BTC-250PRO with USB-AX56 plugged in (chip ID readback).
  Then phases 3-7 (firmware upload, MCU bring-up, scan, assoc, data path).

## 2026-05-08 — AES-CCMP module + end-to-end CCMP roundtrip in vm-selftest
- scope: kernel/src/crypto/{mod.rs,ccmp.rs} (new — CCM-128 with L=2 M=8 per
  RFC 3610 + 802.11 nonce/header builders), kernel/src/netstack/wifi/mock_ap.rs
  (expose negotiated TK), kernel/src/main.rs (`vm_selftest` now runs handshake
  → derive TK on both sides → STA encrypts payload → AP decrypts → tamper-MIC
  rejection check).
- did: pure-Rust no_std AES-CCM-128 on top of existing `tls13::crypto::Aes128`,
  wired into the offline mock AP/STA harness.
- result: build clean, `qemu-selftest.ps1` PASS — `parsers ok / handshake ok /
  ccmp ok / ccmp-smoke ok / [VM-SELFTEST] PASS`.
- next: USB driver API on top of xhci → mt7921u skeleton (real WiFi target).

## 2026-05-08 — Headless QEMU self-test pipeline (`scripts/qemu-selftest.ps1`)
- scope: kernel/Cargo.toml (feature `vm-selftest`), kernel/src/main.rs
  (`vm_selftest()` runs after `coremark` block), kernel/src/netstack/wifi/fixtures.rs
  (fixed: removed 12 B fixed body, parser expects IEs only),
  scripts/qemu-selftest.ps1 (build + ISO + headless QEMU + serial grep).
- did: feature-gated boot-time self-test that runs frame parsers + WPA2
  4-way handshake against MockAccessPoint, prints `[VM-SELFTEST] PASS`
  on COM1. Host script captures serial via `-serial file:`, polls for
  marker, exits 0/1/2/3. xorriso path bug fixed by `cd $outDir` + relative
  paths (msys mangles Windows absolute paths).
- result: end-to-end PASS. Build 1m 49s, ISO 27.28 MB, boot+test ≈ 30 s.
  First run caught a real fixture bug (12-byte fixed body shouldn't be in
  IE blob) — exactly the kind of regression this harness exists for.
- next: AES-CCMP (extend MockAccessPoint with data-frame encrypt/decrypt),
  then ath9k PHY bring-up.

## 2026-05-08 — WiFi offline test harness (mock AP + frame fixtures + `wifi simtest`)
- scope: kernel/src/netstack/wifi/{mock_ap,fixtures,mod}.rs, kernel/src/shell/commands.rs
- did: ajouté `MockAccessPoint` qui simule un AP WPA2-PSK complet (PMK partagé,
  ANonce, vérif MIC msg2, msg3 signé, vérif MIC msg4) + helper
  `run_wpa2_handshake(ssid, pw, sta_mac, bssid)`. Ajouté `fixtures.rs` avec
  beacons canniques (WPA2 + Open) testant `parse_beacon_ies`. Nouvelle
  commande shell `wifi simtest` qui run frame parsers + 4-way handshake
  end-to-end. Marche en VM/QEMU/bare metal — aucune carte WiFi nécessaire.
- result: build clean (1m 54s). Garantit que la stack supplicant + crypto +
  parsers est correcte sur n'importe quelle machine sans dépendre d'un
  driver. Les bugs driver (PCI/MMIO/firmware) restent isolés et testables
  uniquement sur hardware.
- next: AES-CCMP (encryption data frames), puis ath9k PHY bring-up.

## 2026-05-08 — WiFi Phase B1: real WPA2 crypto + shell selftest + wifi devices
- scope: kernel/src/crypto/{mod,sha1,hmac,pbkdf2,wpa}.rs, kernel/src/netstack/wifi/supplicant.rs, kernel/src/shell/commands.rs, kernel/src/main.rs
- did: nouveau module `crate::crypto` (SHA-1 RFC 3174, HMAC-SHA1 RFC 2104,
  PBKDF2-HMAC-SHA1 RFC 2898, IEEE 802.11 PRF-512 + EAPOL MIC). Supplicant
  WPA2-PSK rebranché : PMK = PBKDF2(passphrase, ssid, 4096, 32), PTK = PRF
  réelle, MIC EAPOL = HMAC-SHA1(KCK, frame_with_zero_mic) tronqué 16B.
  Ajouté `wifi devices` (liste PCI WiFi + family + firmware needed) et
  `wifi crypto` (selftest SHA1/HMAC/PBKDF2 avec test vectors).
- result: build clean. Tests intégrés contre vecteurs : SHA1("abc")=a999...,
  HMAC-SHA1 RFC 2202 #1 = b617..., PBKDF2 retourne PMK non-zéro. WPA2-PSK
  end-to-end maintenant correct sur le wire (manque AES-CCMP pour data).
- next: AES-CCMP (data-frame encryption) — réutiliser `tls13::crypto::Aes128`
  + ajouter mode CCM. Puis ath9k PHY bring-up. Puis WPA3-SAE.

## 2026-05-08 — WiFi Phase A foundations + ath9k skeleton + iwl family classifier
- scope: kernel/src/drivers/firmware_loader.rs, kernel/src/netstack/wifi/{mod,frame,channel,supplicant}.rs, kernel/src/drivers/net/{iwl_family,ath9k,wifi,mod}.rs
- did: unified firmware loader (cache + ramfs `/lib/firmware/`); softMAC layer
  (802.11 frame builders + Probe/Auth + beacon IE parser); WPA2 supplicant
  state machine (Open/Wpa2Psk/Wpa3Sae) with PMK/PTK derivation **stubs**
  (TODO real PBKDF2-HMAC-SHA1 + HMAC-SHA1 PRF + AES-CCMP); iwl_family
  classifier covering Iwldvm→IwlBz with firmware-name hints; ath9k skeleton
  (probe + chip ID, PHY bring-up TODO); extended `wifi::probe_pci` to detect
  Atheros 0x168C and report family + firmware needed for unsupported Intel.
- result: `cargo build --release -p trustos_kernel` clean. PCI probe now
  recognises iwldvm (full driver), iwlmvm7/8/9, AX, BZ (logs firmware
  needed), and all ath9k devices (skeleton driver loads but `start()`
  returns "PHY bring-up not implemented").
- next: real crypto in `crate::crypto` (PBKDF2/HMAC-SHA1/CCMP), ath9k PHY
  bring-up + DMA rings, iwlmvm op-mode (huge), USB WiFi (rtl8188eu/mt7601u).
  See repo memory `wifi_stack_status.md`.

## 2026-05-03 — NTFS Sprint 1: dirty flag wired + reparse points detected
- scope: kernel/src/vfs/ntfs.rs, /memories/repo/ntfs_write_plan.md
- did: Wired `read_volume_dirty_flag` into `mount()` (`NtfsFs::is_dirty()` now
  reflects $Volume flags); added ATTR_REPARSE_POINT (0xC0) parser for tags
  `IO_REPARSE_TAG_SYMLINK` (0xA000_000C) + `IO_REPARSE_TAG_MOUNT_POINT`
  (0xA000_0003); `record_file_type` returns `FileType::Symlink` for both.
- result: `cargo build --release -p trustos_kernel` clean (2m 31s). Sprint 1
  now 5/6: only LZNT1 decompression remaining. Live test on Win11 C: pending.
- next: B1c LZNT1, then ISO + USB live test on Nathan's PC (no BitLocker).

## 2026-05-02 - Polaris SDMA VA/APE state cleaned and ruled down
- scope: `kernel/src/drivers/amdgpu/firmware.rs`, live RX 580 `gpu sdma {init-latef32,vadump,wrdiag}`
- did: added `gpu sdma vadump`; checked Linux `vi_srbm_select`/`sdma_v3_0_gfx_resume`; changed init to clear `SDMAx_GFX_VIRTUAL_ADDR/APE1` to zero for all VMIDs plus direct post-loop zero.
- result: `SRBM_GFX_CNTL` readback stays `0` and is not a useful validator. Direct zero fixed stale SDMA1 VA `0x10`; deployed `init-latef32` now shows SDMA0/1 VA/APE all zero. `wrdiag 1 fence` still fetches then faults `SDM0` MC0.
- next: stop treating CSA/VA as primary; next isolate RB_VMID/RB_PRIV/writeback aperture or missing VM/TLB flush before WRITE_LINEAR.

## 2026-05-02 - Polaris SDMA WRITE_LINEAR count/destination triage
- scope: `kernel/src/drivers/amdgpu/firmware.rs`, `kernel/src/shell/vm.rs`, live RX 580 `gpu sdma {wrdiag,test}`
- did: added `gpu sdma wrdiag <cnt> [fence|ring|vram]`; first version that halted/reprogrammed SDMA was rejected as self-contaminating, then changed to non-invasive submit-only mode.
- result: `COUNT=0` fetches to `RPTR_FETCH=0x14` but reintroduces `SDM0` MC0. Non-invasive `COUNT=1` to GART fence also fetches to `0x14` and reintroduces `SDM0` MC0. Full `gpu sdma test` can still stop replying until watchdog/recovery.
- next: do not chase `COUNT=0`; focus on SDMA context/CSA/VA readback before packet submit, because packet fetch works but execution context still dereferences MC0.

## 2026-05-02 - Polaris SDMA init MC0 fault fixed by late F32 unhalt
- scope: `kernel/src/drivers/amdgpu/firmware.rs`, `kernel/src/shell/vm.rs`, live RX 580 `gpu sdma {init,init-noctxsw,init-holdf32,init-latef32,test}`
- did: added delayed PF probes plus init variants for `AUTO_CTXSW` off, F32 held halted, and F32 unhalted late after `CONTEXT_CNTL`/`SDMA_CNTL`.
- result: early F32 unhalt faults at MC0 by 500 ms with `AUTO_CTXSW=1`, and by 2 s with `AUTO_CTXSW=0`; holding F32 halted stays clean. `init-latef32` keeps `PF_STATUS=0` through 2 s with `AUTO_CTXSW=1`.
- result: after `init-latef32`, `gpu sdma test` fetches the ring (`RPTR_FETCH=0x28`, `WPTR=0x28`) with no VM fault, but `WRITE_LINEAR` still does not retire (`RPTR=0`, fence unchanged, `mc_rreq_idle=0`).
- next: make late-F32 ordering the candidate default; debug remaining WRITE_LINEAR/retire path: packet encoding/count, destination translation/cacheability, and RPTR/WB publishing.

## 2026-05-01 - Polaris SDMA doorbell register fixes init-time MC0 fault
- scope: `kernel/src/drivers/amdgpu/firmware.rs`, live RX 580 `gpu sdma {init,test,regs,fault}`
- did: matched Linux `SDMAx_GFX_DOORBELL` register values (`0x100001E0/0x100001E1`); retested `IB_CNTL=0x101`, `PHASE0/1_QUANTUM=0x2000`, and delayed `TRAP_ENABLE`.
- result: doorbell register values stop the init-time `SDM0` MC0 fault. Test still does not retire: `RPTR_FETCH=0x14`, `RPTR=0`, fence unchanged. `IB=0x101`, `PHASE=0x2000`, and `TRAP=1` each reintroduce `SDM0` MC0 in this baseline.
- next: keep best baseline (`doorbell Linux`, `IB=0`, `PHASE=0`, `TRAP=0`, `WPTR_POLL=0x00401000`); investigate retire path/RLC scheduler state (`RLC SCHED=0`) and RPTR writeback visibility.

## 2026-05-01 - Polaris SDMA poll-off deployed; RB_CNTL Linux value rejected
- scope: live RX 580 SDMA debug, `kernel/src/drivers/amdgpu/firmware.rs`
- did: rebuilt/deployed poll-off image, rebooted board, verified pre-init `PF_STATUS=0`; ran `gpu smu start`, `gpu sdma init`, `gpu sdma regs/fault/ptediff`, then `gpu sdma test`.
- result: poll-off image removes the MC-zero init fault. Post-init: `WPTR_POLL_CNTL=0x00401000`, `VIRTUAL_ADDR=0`, `PF_STATUS=0`, PTE[0..7] match. Test still fails at retire: `RPTR_FETCH=0x14`, `RPTR=0`, fence unchanged. Isolated `RB_CNTL=0x00001017` caused `gpu sdma test` to stop shell replies until reboot, so code reverted to `0x00031015`.
- result: after the `0x1017` stall, `SDM0` MC0 fault stayed non-clearable even with F32 halted, RB disabled, CTX0 disabled, and SDMA soft reset pulsed.
- next: keep poll off and `RB_CNTL=0x31015`; start the next pass from a clean pre-init `PF_STATUS=0` baseline, then isolate (`IB_CNTL=0x101`, `PHASE*_QUANTUM=0x2000`, delayed `TRAP_ENABLE`) one at a time.

## 2026-05-01 - Polaris SDMA MC0 fault isolated to AUTO_CTXSW/poll context path
- scope: `kernel/src/drivers/amdgpu/firmware.rs`, live BTC-250PRO `gpu smu start`, `gpu sdma {init,fault,regs,ptediff,test}`
- did: deployed diagnostic-offset fixes; added SDMA halt/RB-disable around GRBM soft reset; disabled IB; tested `AUTO_CTXSW` on/off, `WPTR_POLL` on/off, and CSA-backed `SDMAx_GFX_VIRTUAL_ADDR`.
- result: root symptom is reproducible: `PF_STATUS=0x00077001`, `MCCLIENT='SDM0'`, `ADDR=0`, `VMID=0 CID=0x77 READ`. `IB_CNTL=0` is not enough. `AUTO_CTXSW=0` or `AUTO_CTXSW=1` with poll off avoids the init fault, but WRITE_LINEAR still does not retire.
- next: fix/prove SRBM-selected SDMA CSA/`VIRTUAL_ADDR` programming; only re-enable `WPTR_POLL` after VA/APE readback is sane.

## 2026-05-01 - Polaris SDMA diagnostic offsets corrected
- scope: `kernel/src/shell/vm.rs`, `memory/gpu_unified_memory.md`
- did: corrected stale read-only diagnostics: `gpumap probe` SDMA0 offsets, `gpu sdma fault` VMID0 MCCLIENT (`0x538`) and SDMA1 offset (`+0x200` dwords), plus `gpu sdma fclear` SDMA1 halt regs.
- result: `cargo build --release -p trustos_kernel` clean; no hardware reboot/test performed.
- next: continue live SDMA verification with trustworthy `gpu sdma fault/regs` output before changing retire logic.

## 2026-05-01 - Polaris SDMA retire variants tested; no full NOP retire yet
- scope: `kernel/src/drivers/amdgpu/firmware.rs`, live BTC-250PRO `gpu sdma test`
- did: tested `RB_CNTL.RPTR_WRITEBACK_TIMER=3`, bounded wait after `RPTR_FETCH`, `AUTO_CTXSW` off, `TRAP_ENABLE`, `CONTEXT_CNTL.RESUME_CTX`, and `IB_ENABLE` off.
- result: timer `RB_CNTL=0x31015` removes one no-fault path but does not publish `RB_RPTR`/WB after 20k loops; `TRAP_ENABLE`, `RESUME_CTX`, `PHASE=0x2000`, and IB-off variants do not fix retire and can reintroduce CID `0x77` fault at addr 0.
- next: keep safest state (`CNTL` preserved with AUTO_CTXSW, `PHASE=0`, `RB_CNTL=0x31015`, no WRITE_LINEAR); next likely root is context scheduler/RLC handshake or hidden SDMA VM fault source after internal fetch.

## 2026-04-30 - Polaris SDMA advanced past ring fetch; blocker is RPTR/WB retire
- scope: `kernel/src/drivers/amdgpu/firmware.rs`, `kernel/src/shell/vm.rs`, live BTC-250PRO tests `gpu smu start`, `gpu sdma {init,test,regs,ptediff}`
- did: matched Linux `sdma_v3_0_ctx_switch_enable` more closely by preserving `SDMA0_CNTL` preamble bits instead of clobbering to `0x00040003`; kept `PHASE*_QUANTUM=0`; cleared SDMA VMID VA/APE across VMID 0..15; fixed diagnostics to report real `CONTEXT_*`, VA/APE, and internal `RPTR_FETCH`.
- result: old failure `RPTR_FETCH=0` is fixed. Live test now shows `SDMA0_CNTL=0x08050402`, `RPTR_FETCH=0x4`, `WPTR=0x4`: F32 fetches/consumes the NOP, but `RB_RPTR=0` and RPTR writeback memory stays sentinel. Continuing to WRITE_LINEAR can hang the board.
- next: stop treating this as pre-fetch scheduler failure; focus on RPTR writeback/retire path, VMID1 write fault source, WB cacheability/visibility, RPTR_WRITEBACK_TIMER bits, and a short bounded WRITE_LINEAR diagnostic.

## 2026-04-30 - GPU memory unified around F32 scheduler blocker
- scope: `memory/gpu_unified_memory.md`, `memory/gpu_debug_master.md`, `memory/cp_sdma_debug_todo.md`, agent docs/memories
- did: created one canonical GPU memory from latest journal state; converted old GPU bible/TODO files into compatibility pointers; updated agent references to `gpu_unified_memory.md`.
- result: current source of truth now says SDMA blocker is F32 firmware/ring scheduler state, with VM/GART/sysRAM-above-4G ruled out by `PF_STATUS=0` and failing VRAM-only NOP.
- next: use unified plan to diff TrustOS SDMA F32 load/start sequence against Linux and add deeper ucode/status diagnostics.

## 2026-04-30 â€” Polaris SDMA root narrowed past VM/GART/sysRAM: F32 ring scheduler
- scope: `kernel/src/drivers/amdgpu/firmware.rs`, `kernel/src/shell/vm.rs`, live BTC-250PRO tests `gpu sdma {ptediff,test,gfx-init,vram-nop}`
- did: moved SDMA ring/shared/IH/CSA allocations to DMA32 frames below 4G; added `gpu sdma vram-nop` to test ring+WB in VRAM with `VM_CONTEXT0` disabled; tested `gfx-init -> sdma init -> test`, `AUTO_CTXSW` off, `UTC_L1` off, `RB_SIZE=11`, `RB_PRIV`, pollmem off/MMIO WPTR.
- result: DMA32 PTEs now map `phys=0x100000..0x107000` and match, but NOP still fails (`RPTR=0 WPTR=4 PF=0`). VRAM direct test also fails: `ring_mc=0xF400100000 wb_mc=0xF400102000`, `R=0 W=4 ST=0x4496EC56 WB=0xDEAD0000 PF=0`.
- conclusion: blocker is not SYS_APR, GART PTEs, sysRAM above 4G, VM faults, GFX init order, or WPTR poll. Remaining suspect is SDMA F32 firmware/ring scheduler state: exact ucode load mode/size, `F32_CNTL`/`RB_CNTL` semantics, or missing Linux reset/enable sequence before ring fetch.
- next: diff TrustOS SDMA firmware load against Linux `sdma_v3_0_init_microcode/load_microcode/start`; add diagnostics for ucode header/JT, `SDMA0_CHICKEN_BITS`, status2/program counters if available, and test force-full firmware load vs JT load.

## 2026-04-30 â€” Polaris SYS_APR deployed + SDMA blocker narrowed to F32 scheduler
- scope: PXE deploy/reboot BTC-250PRO, `gpu smu start`, `gpu sdma {init,vm-dump,test,reg,wptr,fault}`, `kernel/src/drivers/amdgpu/firmware.rs`
- did: deployed patched kernel to `pxe_tftp/trustos_kernel`, software rebooted board, started SMU, ran SDMA init/test. Added init-side shared-page/WPTR-poll zeroing + `wbinvd`, rebuilt/redeployed/rebooted, retested.
- result: `vm-dump` now shows correct SYS_APR at `0x2034/0x2038/0x203C = 0x0F400000/0x0F5FFFFF/0x0F400400`; `PF_STATUS=0` after init and after failed tests. Second patch fixed stale init `W0=4`; post-init now `R0=0 W0=0`.
- result: Level NOP still fails: `RPTR=0 WPTR=4`, `FAULT_STATUS=0`, `STATUS=0x46DC6452/0x46DC6C52`, `PKT_READY=1`, `mc_rreq_idle=0`. Live tests disabling `WPTR_POLL_CNTL` and setting `RB_PRIV` (`RB_CNTL=0x00801015`) did not advance RPTR.
- next: stop chasing VM/GART; next blocker is SDMA F32 scheduling/context state. Compare Linux `SDMA0_CNTL`, `GFX_CONTEXT_CNTL`, RLC/CSA handshake, and exact ring enable order; add diagnostics for context/status regs around F32 scheduler.

## 2026-04-30 â€” Polaris SYS_APR constants patched to live/Linux offsets
- scope: `kernel/src/drivers/amdgpu/firmware.rs`, `kernel/src/shell/vm.rs`, `kernel/src/drivers/amdgpu/pipeline_audit.rs`
- did: changed `MC_VM_SYS_APR_LOW/HIGH/DEFAULT` from ignored/stale offsets (`0x82A..0x82C` or `0x810..0x812`) to live-validated `0x80D/0x80E/0x80F` (`0x2034/0x2038/0x203C`). Fixed VM dumps, gpumap MC labels, audit expectations, and PT_BASE expected decode.
- result: `cargo build --release -p trustos_kernel` OK. Remaining old `0x82A..0x82C` reference is only explanatory comment documenting the rejected offsets.
- next: deploy via PXE, boot board, run `gpu smu start` if needed, `gpu sdma init`, then verify `gpu sdma vm-dump` shows SYS_APR `0x0F400000/0x0F5FFFFF/0x0F404000` and retest `gpu sdma test`.

## 2026-04-30 â€” Polaris SDMA live diag: system aperture offset mismatch
- scope: BTC-250PRO live shell UDP 7777, `gpu smu`, `gpu sdma {init,test,ptediff,vm-dump,fault}`, `kernel/src/drivers/amdgpu/{firmware,pipeline_audit}.rs`
- did: confirmed board up; SMU was `NOT RUNNING` after soft reset, then `gpu smu start` succeeded. `gpu sdma init` loaded SDMA FW and set GART ring at `0xFF00000000`; `ptediff` proved PTE[0] matches ring phys and flags `0x63`.
- result: SDMA NOP still fails: `RPTR=0 WPTR=4`, F32 running, `PF_STATUS=0x01078001` at `0xF400075000` client `CID=0x78`. Live writes to `0x20A8/0x20AC/0x20B0` are ignored; Linux reference offsets `0x2034/0x2038/0x203C` accept `0x0F400000/0x0F5FFFFF/0x0F404000`.
- result: current code/diagnostics disagree: `firmware.rs`/`vm-dump` use `0x82A..0x82C`, `pipeline_audit.rs` uses stale `0x810..0x812`, local Linux reference says `0x80D..0x80F`. `gpu sdma gart`/`audit` are unsafe on board and can watchdog/reset due VRAM BAR remap.
- next: patch one source of truth to `MC_VM_SYSTEM_APERTURE_* = 0x80D/0x80E/0x80F` if confirmed against headers; update `vm-dump`/audit expected values; avoid `gart`/`audit` until remap path is fixed; never physical-reboot board from agent workflow.

## 2026-04-30 â€” PXE/HTTP comm server sanity + BTC reachability
- scope: `scripts/pxe_server.py`, `scripts/http_server.py`, `pxe_tftp/limine.conf`, BTC-250PRO network reachability
- did: started TrustOS PXE/TFTP on UDP 67/69 and HTTP on TCP 8080; removed stale duplicate `start_pxe_server.ps1`/PXE instance.
- result: clean services active (`pxe_server.py` PID 21600, `http_server.py` PID 15592). BTC shell UDP 7777 returned no reply on 10.0.0.111; ARP has no `b8:97:5a:d9:54:66`; ping .110/.111 timed out.
- next: power-cycle/check BTC-250PRO physical power/link, then watch PXE DHCP/TFTP logs during boot.

## 2026-04-30 â€” BIOS workflow: SSD + `bios pull` (sneaker-net retired)
- scope: `pxe_setup/freedos/payload/{MENU,FLASH}.BAT`, `pxe_setup/freedos/README.md`, `tools/bios/README.md`, `kernel/src/shell/unix.rs` (cmd_bios help), `/memories/repo/bios_dump_workflow.md`
- did: aligned every doc + DOS script on the real plan: DUMP.BAT writes `C:\BIOS.ROM` on the BTC-250PRO **internal SSD** (first MBR FAT12/16/32 partition). After reboot to TrustOS: `diskscan`, `mount ahci:0:<lba> /mnt/ssd`, `bios pull /mnt/ssd/BIOS.ROM` streams the dump to dev box on UDP 7780 (hub menu `d` receiver, already implemented). Documented BIOS Setup prereqs (CSM/Legacy + IPv4 PXE + IPv6 off â€” board's factory defaults). Dropped lingering "USB stick" / "TFTP push" wording. Fixed `bios pull` help text (`menu '6'` â†’ `menu 'd'`).
- result: `cargo build --release -p trustos_kernel` clean (0.14s incremental). No code path change beyond the help string. SSD prerequisite called out: needs â‰¥64 MB FAT32 partition pre-created (FreeDOS ignores GPT/ext/NTFS).
- next: pre-stage a FAT32 partition on the BTC-250PRO SSD (one-time), then run the full DUMP â†’ reboot â†’ `bios pull` cycle for the first time.

## 2026-04-29 â€” FreeDOS BIOS-tools: sneaker-net pivot
- scope: `pxe_setup/freedos/payload/{AUTOEXEC,DUMP,FLASH}.BAT`, `tools/bios/make_freedos_floppy.py`, `tools/bios/build_freedos_pxe.ps1`, `scripts/pxe_server.py`
- did: pivoted away from mTCP TFTP after discovering (a) mTCP doesn't ship a TFTP client and (b) DOS packet drivers for the BTC-250PRO Realtek would be a deep rabbit hole. Switched to USB-stick sneaker-net: dump goes to `C:\BIOS.ROM` on a stick plugged in the board, operator carries it. Removed all DOS networking (no mTCP downloads, no MTCP.CFG, no NC/HTGET). Reverted the TFTP WRQ handler in pxe_server.py.
- result: floppy builds clean (1.44 MB). AFUDOS 2.36 (AFU236U.EXE from user Downloads) confirmed pure-DOS MZ binary. Status shows AFUDOS+fdboot+freedos.img all OK.
- next: plug FAT32 USB stick into BTC-250PRO, hub menu `b`, power-cycle, run DUMP.BAT, transport stick to dev box.

## 2026-04-29 â€” FreeDOS PXE BIOS-tools profile
- scope: `pxe_setup/freedos/` (new), `tools/bios/` (new), `scripts/trustos-hub.ps1`, `scripts/pxe_server.py`, `pxe_setup/start.{py,ps1}`, `pxe_setup/config.psd1`, `AGENTS.md`, `.gitignore`
- did: added 3rd PXE profile (parallel to trustos/alpine) that sanboots a FAT12 floppy with FreeDOS+AFUDOS+mTCP for in-place BIOS dump/flash on BTC-250PRO. `boot.ipxe` is now profile-managed (copied alongside `limine.conf` at switch time). `pxe_server.py` got a path-restricted TFTP WRQ handler (only `freedos/dumps/`, 32 MB cap) so DUMP.BAT can push dumps back automatically. Hub menu `b` builds floppy on demand.
- result: file edits only â€” not yet exercised on hardware. AFUDOS.EXE is operator-supplied (gitignored).
- next: drop AFUDOS.EXE in `tools/bios/payload/`, run `-Build`, validate first dump on BTC-250PRO, AMIBCP-unlock GPU options, flash, retest GPU init. See `/memories/repo/bios_dump_workflow.md`.

## 2026-04-29 â€” Knowledge consolidation + comment cleanup
- scope: `memory/gpu_debug_master.md` (rewritten), `memory/cp_sdma_debug_todo.md` (replaced), `kernel/src/drivers/amdgpu/{firmware,smu,mod,gpu_train}.rs`
- did: rewrote bible (sections HW / IP / memmap / pipeline status / current blocker / regs / rules / Linux ref / build / xrefs); replaced obsolete "boot Linux to dump regs" plan with concrete VMID0/GART diagnostic ladder; stripped non-factual comments (date refs, "BREAKTHROUGH", "previous version", "Let me fix" narrative) from amdgpu code while preserving all technical content (Linux refs, bit layouts, register decode).
- result: `cargo build --release -p trustos_kernel` clean (5.44s incremental). Bible now reflects current state (SMU OK, SDMA blocked on VM fault). Next-session attack plan ranked.
- next: implement `gpu sdma vm-dump` + `gpu sdma gart-dump` shell subcmds, then walk diagnostic ladder.

## 2026-04-29 â€” SDMA post-SMU: still FAIL, root cause = VMID/GART
- scope: `gpu sdma init` + `gpu sdma test` sur board (post-SMU running)
- did: confirmÃ© SMU running, lancÃ© init+test SDMA
- result: Level 0 PASS, Level NOP FAIL. RPTR=0 WPTR=4. STATUS=0x46FEED57 idle=0 halt=0. **FAULT_STATUS=0x01078001 FAULT_ADDR=0xF400075** (VMID=0 CID=0x78 RW=1 PROT=1). CTX0=0xFFFED9. SDMA cherche ring en VRAM au lieu de GART (ring MC=0xFF00000000).
- next: SMU n'Ã©tait PAS la cause root du bug SDMA. Le vrai bug = VM_CONTEXT0_PAGE_TABLE_BASE_ADDR ou bit SYSTEM/MTYPE mal config pour SDMA VMID0. Investiguer GMC CTX0 setup dans `gmc` subcmd.

## 2026-04-29 â€” SMU Polaris BRINGUP COMPLET (RUNNING)
- scope: kernel/src/drivers/amdgpu/smu.rs (dispatcher L700+), build _b30
- did: VBIOS-pre-init detection (boot_seq_done && RESP==1 && INPUT!=0) â†’ ne plus appeler `smu7_resume_protection_mode` (MSG_Test seul Ã©choue: SRAM secure mode = magic invalide). Appeler `smu7_start_smc_protection_mode` (full Linux flow: reset â†’ upload signÃ© â†’ enable_clk â†’ deassert â†’ wait BR â†’ MSG_Test â†’ DONE â†’ PASS â†’ 2nd reset â†’ wait FW_FLAGS).
- result: SMC started OK PC=0x00020498. SMU v7 (Polaris) RUNNING, FW=0x0003001E, RESP=0x01, SCLK=135MHz, MCLK=150MHz. SMU_DONE aprÃ¨s 48129 iters, FW validation PASSED, FW init aprÃ¨s 80 iters (FLAGS=0x1 INTERRUPTS_ENABLED).
- next: tester SDMA Level NOP (devrait fonctionner maintenant que mclk/BIF clocking propre), puis DPM/SCLK/MCLK boost.

## Format

```
## YYYY-MM-DD â€” <short title>
- scope: <files / area touched>
- did: <what changed, 1â€“2 lines>
- result: <build status / hardware result / RPTR-STATUS-WB diff>
- next: <follow-up or "done">
```

## Rules

- Append-only. Never rewrite past entries.
- Newest entry at the top, just under this section.
- Trivial Q&A (no code touched) â†’ no entry.
- If the task produced a recurring rule, also update the matching
  `memory/*.md` or `/memories/repo/*.md` note and link it from the entry.
- Keep entries 3â€“6 lines max. Diff RPTR / STATUS / WB if GPU work.

---
## 2026-04-29 â€” Polaris SDMA: RPTR_FETCH diag + RESUME_CTX off (b26/b27)
- scope: kernel/src/drivers/amdgpu/firmware.rs (Phase 3c, polaris_sdma_diag_dump)
- did: b26 added SDMA0_RB_RPTR_FETCH(0x340a) + IB_OFFSET_FETCH(0x340b) reads to diag â€” exposed F32 internal fetch ptr. b26 result: RPTR_FETCH=0 (F32 NOT consuming despite RB_EMPTY=1 oscillation). b27 set CONTEXT_CNTL=0 (was RESUME_CTX=0x10000 â†’ tells F32 to wait for nonexistent saved ctx). Linux sdma_v3_0_gfx_resume() value.
- result: b27 cleaned VM fault â€” FAULT_STATUS=0x00000000 (was 0x01078001 on 0xF400075000). Confirms RESUME_CTX caused the bogus fault. RPTR_FETCH still 0, STATUS=0x46FC6452 (PKT_READY=1, RB_EMPTY=0, busy). F32 sees ring non-empty but never advances fetch ptr.
- next: add SDMA0_STATUS1/STATUS2/FREEZE/PROGRAM diag; investigate why F32 parses but never fetches (possible: stale ucode boot state, missing SDMA0_VM_CTX_LO/HI init, or RB_VMID mismatch).
## 2026-04-29 â€” Polaris SDMA: AUTO_CTXSW unblocks F32 fetch (b25)
- scope: kernel/src/drivers/amdgpu/firmware.rs Phase 3c + Phase 4
- did: oss_3_0_sh_mask.h ground truth â€” CONTEXT_CNTL only has RESUME_CTX(b16) + SESSION_SEL(b24-27); the "CTX_ENABLE bit 18" assumption was wrong (that bit is AUTO_CTXSW_ENABLE in **SDMA0_CNTL** not CONTEXT_CNTL). Wrote CONTEXT_CNTL=0x10000 only, and SDMA0_CNTL=0x40003 (TRAP|ATC_L1|AUTO_CTXSW). Linux ctx_switch_enable(true) does this on bare metal.
- result: build 10371600B. STATUS 0x46FC7052â†’**0x46FEE056** (RB_EMPTY=1, DELTA_RPTR_FULL=1) â€” F32 now consumes ring. RPTR MMIO/WB still 0xDEAD; new VM fault FAULT_STATUS=0x01078001 VMID0 CID=0x78 addr=0xF400075000 (page-table read?). SDMA_CNTL=0x40003 confirmed.
- next: investigate (a) why RPTR writeback doesn't land despite RPTR_WRITEBACK_ENABLE=1 in RB_CNTL (0x1015), (b) the new VM fault on 0xF400075000 (likely PT walk in VRAM hitting unmapped page), (c) decode CID=0x78 against Polaris VM client list.

## 2026-04-29 â€” Polaris SDMA: 3 register offsets fixed via oss_3_0_d.h (b23/b24)
- scope: kernel/src/drivers/amdgpu/firmware.rs polaris_sdma_regs
- did: CONTEXT_CNTL 0x3491â†’0x3493 (was CONTEXT_STATUS RO!), VIRTUAL_ADDR 0x349Aâ†’0x34A7, APE1_CNTL 0x349Bâ†’0x34A8 â€” all confirmed against Linux torvalds/master oss_3_0_d.h. Also removed CSA fallback in Phase 2 (writes 0 like Linux when no CSA).
- result: writes to CONTEXT_CNTL now stick (post=0x10000). But ring still not consumed in b24 â€” bit 18 was rejected (only bit 16 in this register).
- next: see _b25 entry above.

## 2026-04-29 â€” Polaris SDMA: WPTR_POLL_CNTL F32_POLL_ENABLE bit set, NOP still stuck (b19/b20)
- scope: kernel/src/drivers/amdgpu/firmware.rs config_ring (WPTR_POLL_CNTL write)
- did: b19 wrote `WPTR_POLL_CNTL=1` (wrong: bit 0 is ENABLE not F32_POLL). b20 RMW set bit 2 (F32_POLL_ENABLE per oss_3_0_sh_mask.h: bit0=ENABLE, bit1=SWAP_ENABLE, bit2=F32_POLL_ENABLE).
- result: STATUS varies between hb iters (0x46DC6452 / 0x46DC6C52 / 0x46DC7052 / 0x46DC7852). PACKET_READY=1, RB_EMPTY=0 â†’ F32 IS reading the ring. EX_IDLE oscillates. RPTR stays 0, DELTA_RPTR_EMPTY=1 â†’ F32 fetched but never executed/committed packet. No fault.
- next: investigate (a) VM_CONTEXT0 page-table coverage of GART [0xFF00000000..0xFF0FFFF000] (likely missing â€” F32 reads ring OK because it's in some prefetch path but cannot translate WB writes); (b) SDMA0_CNTL.UTC_L1_ENABLE bit actually translating; (c) dump WPTR_POLL_CNTL readback to confirm bit 2 stuck; (d) verify NOP encoding matches Linux SDMA_PKT_NOP (header=0x0 OP=0).

## 2026-04-29 â€” Polaris GMC: VM fault FIXED â€” SYS_APR registers were wrong + wrong shift
- scope: kernel/src/drivers/amdgpu/firmware.rs (POL_MC_VM_SYS_APR_LOW/HIGH/DEFAULT constants + polaris_gmc_init shift)
- did: (1) fixed offsets 0x810/0x811/0x812 â†’ 0x82A/0x82B/0x82C (Linux gmc_8_1_d.h definitive). (2) fixed shift `vram_start >> 18` â†’ `>> 12` to match Linux gmc_v8_0_mc_program. Removed the bogus undo writes to 0x80D-0x80F.
- result: `gpu sdma init` clean PROBE thru all phases. `gpu sdma test` diagnostic now reports **FAULT_STATUS=0x0 FAULT_ADDR=0x0** (was 0x01078001/0xF400075000). HDP/DMIF/SDMA traffic to FB pages no longer range-faults.
- result: SDMA test STILL fails at Level NOP (RPTR=0 WPTR=4 ST=0x46DC6C52 F32=0) â€” but it's now a pure F32-fetch bug, not VM. The 0xF400075000 mystery is fully solved (was MCCLIENT="HDP" hitting unconfigured aperture).
- next: debug F32 ring fetch (RB_BASE/RB_CNTL/RB_VMID, UTC_L1 invalidate path, sysRAM coherency). Lesson: ALWAYS cross-check register dword offsets against `gmc_8_1_d.h` AND shift amounts against `gmc_v8_0_mc_program`. Comment "confirmed from Linux hw dump" was wrong.

---

## 2026-04-29 â€” Polaris SDMA: PF_STATUS=0x01078001 was STALE, not init-time
- scope: kernel/src/drivers/amdgpu/firmware.rs (_b17 added MCCLIENT decoder + BLOCK= 4-char ASCII tag in early_probe)
- did: built _b17 with PROTECTION_FAULT_MCCLIENT readout (CTX0=0x538*4, CTX1=0x539*4); deployed via PXE; fresh power-cycle reboot of BTC-250PRO board.
- result: ALL `[SDMA-PROBE]` lines through `polaris_sdma_full_init` now report `PF_STATUS=0 (clean)` â€” including `after-phase3c-ctxcntl`, `after-phase4-ctxsw`, `after-settle-10ms`. Previous "fault at every probe" was leftover state from prior boots/runs that the test path's "Cleared stale fault" message had been masking.
- result: `gpu sdma test` still FAILS at Level NOP â€” RPTR stays 0x0, WPTR_hw=0x4, ST cycles 0x46DC6452â†’0x46DC6C52 (F32 alive), F32_CNTL=0x0, but engine never consumes the NOP. Diagnostic dump shows fault REAPPEARS by test-time: FAULT_STATUS=0x01078001 FAULT_ADDR=0xF400075 (CID=0x78, RW=1, VMID=0, PROT=RANGE). Confirms the writer hits 0xF400075000 between init-end and test-start (~22s), not during init.
- result: CONTEXT_CNTL post-unhalt reads 0x5 e0/e1 even after we write 0 â†’ bit 2 (CTX_CACHE_ENABLE) is reasserted by F32 microcode. Our `SDMA0_GFX_VIRTUAL_ADDR` redirect (0x349A*4 â† csa_mc low32) does NOT prevent F32 from hitting hardcoded CSA addr 0xF400075000.
- next: find the actual per-engine CSA pointer register on Polaris (likely SDMA0_F32_REGS area or HQD-style CSA_ADDR_HI/LO), or investigate `SDMA0_RLC0_CONTEXT_STATUS` / preempt registers for a way to force the F32 to use a programmed CSA. Linux probably never trips this because RLC owns the CSA and the F32 reads it from RLC-supplied address.

---

## 2026-04-28 â€” Polaris SDMA: wbinvd + UTC_L1 BOTH ruled out, F32 fetch still stuck
- scope: kernel/src/drivers/amdgpu/firmware.rs (Phase 3 ring config + Phase 4 SDMA_CNTL)
- did: (1) added wbinvd in init after ring zero-fill and before unhalt to flush
  CPU cache to DDR â€” RPTR still 0; (2) reverted wbinvd; set SDMA_CNTL = 0x03
  (TRAP_ENABLE | UTC_L1_ENABLE bit 1) per sdma_v3_0_d.h to allow F32 to use the
  L1 TLB for VMID-based MC translation â€” RPTR still 0. Built _b14, _b15 OK.
- result: identical signature both runs: STATUS=0x46DC6C52 (mc_rreq_idle=0,
  rb_empty=0, halt=0), RPTR=0, WPTR=0x8 in HW, ring CPU-view all NOPs (0x0),
  FAULT_STATUS=0x01078001 (VALID, CID=0x78=DMIF, MORE=0). Display engine is
  re-faulting continuously between every clear, but MORE_FAULTS=0 means SDMA
  itself is NOT logging a fault â€” its MC reads simply never complete.
- next: stop fix-by-guess. Add targeted diagnostics in next session: (a) read
  RPTR_WB location directly to check if F32 ever wrote there; (b) dump GART
  PTE[0] from VRAM via BAR0 to verify it equals ring0_phys|flags; (c) clear
  FAULT_STATUS in a tight loop while polling RPTR to test the "DMIF stalls
  L2" hypothesis; (d) consider blanking DCE pipes before SDMA test. Kept the
  UTC_L1 bit (matches Linux, harmless); reverted both wbinvds (no effect).

---

## 2026-04-28 â€” Polaris SDMA: ucode-verify proves UCODE_DATA is WRITE-ONLY (open-bus on read)
- scope: kernel/src/drivers/amdgpu/firmware.rs (+`polaris_sdma_ucode_verify`), kernel/src/shell/vm.rs (+`gpu sdma ucode-verify`)
- did: Added a readback verifier that walks ADDR 0..63 and reads UCODE_DATA,
  comparing against embedded firmware bytes. Two iterations: first naive
  (assumed auto-inc on read), second with explicit `ADDR=i` write per read.
- result: SDMA0 reads return `0xCFBEFFDF` for ALL addresses; SDMA1 returns
  `0xFFFDEFFE`. ADDR write does work (`addr_after=0x3F` after writing 0x3F).
  Conclusion: **UCODE_DATA is write-only on Polaris SDMA F32**, reads return
  open-bus garbage. Cannot directly verify SRAM contents this way.
- INDIRECT proof ucode IS loaded: `STATUS.mc_rreq_idle=0` after unhalt means
  F32 IS executing code and issuing MC reads. A garbage SRAM would crash F32
  on illegal instruction before any MC activity. So ucode upload path works.
- next: Real blocker = F32 emits MC reads but RPTR stays 0. Suspects:
  1. CONTEXT_CNTL post-unhalt = 0x5 (bit2 = LOAD_VAR_FROM_VMID, F32 stuck
     loading CSA from VMID0 instead of polling ring).
  2. AUTO_CTXSW behavior (old journal L349 said it WAS the lock; we set
     CNTL=0x01 which has it off â€” verify).
  3. Test code corrupts ring[0..7] with 0x11111111..0x88888888 before NOP
     test â†’ first packet F32 fetches is malformed â†’ may hang silently.
  Next concrete steps: (a) clear ring fully before NOP test (gpu sdma test
  fix), (b) write CONTEXT_CNTL=0 explicitly post-unhalt and confirm F32
  enters ring-poll loop, (c) decode STATUS bits in `gpu sdma fault`.

---
- scope: kernel/src/drivers/amdgpu/firmware.rs (removed MEC halt block, fixed dump decoder)
- did: Build _b11 deployed. Removed dead `mmio_write32(CP_MEC_CNTL, 0x50000000)`
  block (CP_MEC_CNTL boot value already = 0x50000000 from VBIOS â€” verified by
  diagnostic read at function-entry). Also fixed the `gpu sdma test` /
  diagnostic-dump decoder in firmware.rs L2483 to gmc_v8 layout matching vm.rs.
- result: ALL 18 SDMA-PROBE points return PF_STATUS=0 (clean) end-to-end
  through `polaris_sdma_full_init`. Previous "fault stream during init" was an
  artifact of (a) wrong decoder + (b) reading stale post-test FAULT_STATUS.
  CP_MEC_CNTL boot = 0x50000000 confirmed (ME1+ME2 halted). `gpu sdma test`
  Level 0 PASS, Level NOP FAIL â€” STATUS=0x46DC6C52 (halt=0, RB_EN=1,
  mc_rreq_idle=0 â†’ F32 IS issuing reads but RPTR stays 0). Post-test
  FAULT_STATUS=0x01078001 @ MC=0xF400075000 = DCE/DMIF (CID=0x78), unrelated to
  SDMA fetch path.
- next: F32 fetches but RPTR doesn't advance. Investigate ATC bit in
  SDMA0_CNTL, WPTR_POLL_CNTL ENABLE, F32 ucode readback verification, or
  doorbell mode mismatch. Old journal line 349 said "AUTO_CTXSW was the lock"
  â€” re-check CONTEXT_CNTL post-unhalt (current logs show e0=0x5 e1=0x5,
  AUTO_CTXSW bit). Build counter â†’ _b12.

---

## 2026-04-28 â€” DECODER BUG: PF_STATUS bit layout was WRONG, fault is VMID=0 / CID=0x78
- scope: kernel/src/drivers/amdgpu/firmware.rs (early_probe + inline probe)
- did: After Nathan's parallel `gpu sdma fault` shell command (vm.rs decoder uses
  the correct gmc_v8 layout), confirmed that BOTH probe closures in firmware.rs
  used the wrong bit shifts:
    WRONG: CID=(st>>9)&0xFF  RW=(st>>17)&1  VMID=(st>>18)&0xF  PROT=(st>>24)&0xFF
    RIGHT: PROT=st&0xFF      CID=(st>>12)&0xFF  RW=(st>>24)&1  VMID=(st>>25)&0xF
  Re-decoding 0x01078001 with the correct layout:
    PROT=0x01 (RANGE), CID=0x78 (= MC client 120), RW=1 (write), VMID=0, MORE=0.
  Fixed both probes in firmware.rs to match vm.rs / Linux gmc_v8_0 layout.
- result: The previous "MEC2 in VMID=1" theory was based on garbled bits and is
  WRONG. Real picture: **MC client 0x78 = DMIF/DCE (Display Controller Engine)**
  â€” confirmed by earlier journal entry (line ~380, "CLIENT=0x78 (DMIF/DCE)").
  VBIOS keeps the display scanout running at boot, and DCE writes to a stale
  scanout target near FB+0x75000 in VMID=0 (system aperture) which TrustOS has
  not reprogrammed, so the write range-faults continuously. PROT=0x01 = RANGE.
  CTX0_CNTL=0x00FFFED9 (enabled). MC_VM_SYS_APR_{LOW,HIGH,DEFAULT} are
  programmed in polaris_gmc_init using shift-by-18, vram_start..vram_end â†’
  0xF400075000 IS in range mathematically, yet GMC still range-faults. Either
  (a) SYS_APR isn't taking effect because L1 TLB SYSTEM_ACCESS_MODE=3 forces
  all accesses through GART (and there's no GART entry for that page), or
  (b) DCE has its own private VM aperture path that bypasses the GMC system
  aperture and needs explicit display-block GMC programming.
- next:
  1) Halt the display engine at boot â€” disable DCE/DMIF scanout BEFORE
     polaris_gmc_init/sdma init so the fault stream stops. Linux dce_v11_0
     has dce_v11_0_set_vga_render_state() / disable_vga() / disable_dce_blocks().
  2) Or, if cleaner: program DCE's own GMC base/aperture so its scanout
     target lives in mapped FB.
  3) MEC halt was a no-op â€” CP_MEC_CNTL was already 0x50000000 at boot, so
     MEC1+MEC2 ARE halted by VBIOS. Drop or downgrade the MEC halt block in
     polaris_sdma_full_init to a read+log only.
- repo notes to update: rename the planned `polaris_mec_blocker.md` rev to
  `polaris_dce_vmid0_fault.md` (DCE/display, NOT MEC).

---

## 2026-04-28 â€” Polaris VM fault root cause: VBIOS-loaded MEC2 polling FB+0x75000 in VMID=1
- scope: kernel/src/drivers/amdgpu/firmware.rs (early_probe at function-entry)
- did: Hoisted PF_STATUS probe to the very FIRST line of `polaris_sdma_full_init`
  (before `gpu_alive`, `polaris_gmc_init`, `GRBM_SOFT_RESET`, golden, alloc, GART
  population). Built _b9.txt clean (10713552 B). Test sequence:
    1) Fresh reboot + `gpu sdma regs` (NO init):   PF_STATUS=0 (clean).
    2) Fresh reboot + `gpu sdma init` function-entry probe: PF_STATUS=0x01078001.
- result: The fault appears BETWEEN boot completion and the very first executable
  line of `polaris_sdma_full_init` â€” but no GPU MMIO write happens in that gap.
  Conclusion: a background client is generating WRITE faults continuously to
  MC=0xF400075000 (= FB base 0xF400000000 + offset 0x75000) in VMID=1, CID=0xC0.
  CID=0xC0 = MEC HQD/IQ. We know from prior probes that **MEC2_INSTR_PNTR=0x8**
  at boot â€” VBIOS-resident MEC2 firmware is alive and polling/writing a scratch
  area in FB through VMID=1's page table, which TrustOS never programmed.
  The fault rate is high enough to repopulate PF_STATUS within the latency of
  reading/clearing it. SDMA bringup is innocent.
- next: Three possible fixes:
  (A) Program VMID 1..15 page tables to identity-map FB (so MEC2 can write).
  (B) Force-halt MEC2 at boot (`MEC_CNTL bit ME2_HALT`) before SDMA init.
  (C) Skip `init_polaris` altogether and have a dedicated `gpu mec halt` that
      stops MEC2 first, then `gpu sdma init` runs in clean state.
  â†’ Try (B) first â€” cheapest, no GMC table churn. If MEC2 halt clears the
  fault stream, the SDMA bringup probes should all show PF_STATUS=0.
- repo notes to update: `polaris_mec_blocker.md` (new "MEC2 alive faulting" section).

---

## 2026-04-28 â€” Polaris SDMA: 12-probe init trace â†’ fault is PRE-EXISTING, NOT from SDMA
- scope: kernel/src/drivers/amdgpu/firmware.rs (init_polaris probe closure + 12 call sites)
- did: Inserted PF_STATUS+PF_ADDR snapshot probe between every init phase
  (init-entry, after-phase0/1-halt/1b-fw/2-csa/3-ring0/3-ring1/3b-unhalt-e0/e1
  /3c-ctxcntl/4-ctxsw/settle-10ms). Built _b7.txt clean (10363832 B), deployed,
  rebooted board, captured netconsole UDP 6666 (`tools/debug/netconsole.py`).
- result: ALL 12 probes report identical PF_STATUS=0x01078001 VMID=1 CID=**0xC0**
  RW=1 PROT=0x1 MC=0xF400075000 â€” INCLUDING `init-entry` (before any SDMA write).
  CID=0xC0 = GFX/MEC HQD/IQ, NOT SDMA. The SDMA bringup itself is innocent â€”
  the fault was already pending at init entry, induced by boot-time AMDGPU
  init (MEC1). PF_STATUS W1C clear in probe ineffective â†’ MEC re-faults
  continuously on each MMIO probe-read latency window.
- next: Pivot focus to MEC1 init at boot â€” see `polaris_mec_blocker.md`.
  SDMA can resume after MEC1 stops faulting. Consider: skip MEC1 boot init
  entirely (gate on `--no-mec`), then re-run `gpu sdma init` standalone to
  validate clean SDMA bringup.

---

## 2026-04-28 â€” Polaris SDMA: PF_STATUS decode added; pre-init = clean (fault is induced)
- scope: kernel/src/shell/vm.rs (gpu sdma regs: PF_STATUS field decoder)
- did: Added field-by-field decode (MORE/WALKER/PERM/MAPPING/CID/RW/VMID/PROT)
  with VMID!=0 highlight + Polaris MC-client-ID name lookup. Built (_b6.txt
  clean, 10364808 B), deployed pxe_tftp/. Rebooted board, ran `gpu sdma regs`
  BEFORE running any `gpu sdma init`.
- result: **PF_STATUS = 0 pre-init.** All VM ctx0 regs = 0, VIRTUAL_ADDR = 0,
  PT_BASE/START/END = 0, MC_VM_FB_LOCATION = 0xF5FFF400 (BIOS default).
  â†’ Fault at MC 0xF400075000 with VMID=1/CID=0xC0 from previous session is
  NOT pre-existing â€” it is **induced by our `gpu sdma init` flow**. After
  running init, kernel hangs / auto-recovers before regs reply (expected).
- next: Trace which write in firmware.rs Phase2/Phase3 triggers the fault.
  Suspects: (a) ring/IB doorbell ringing on uninit'd VMID 1; (b) MEC HQD
  default IQ pickup; (c) SMU-touched scratch range. Add a fault-clear at
  start of init then re-snapshot PF_STATUS after each major phase.

---

## 2026-04-28 â€” Polaris SDMA: wrong reg offsets fixed + VIRTUAL_ADDR write OK but fault unchanged
- scope: kernel/src/drivers/amdgpu/firmware.rs (offsets, CSA alloc, GART PTE6/7,
  Phase 2 VIRTUAL_ADDR setup)
- did: (1) Allocated 2x 4 KiB CSA pages in sysRAM, GART-mapped at PTE6/PTE7
  (MC 0xFF00006000/0xFF00007000). (2) Pre-unhalt programmed
  SDMA[01]_GFX_VIRTUAL_ADDR for all 16 VMIDs with CSA MC low 32 bits.
  (3) Discovered firmware.rs had **wrong sdma_v3_0 offsets** vs oss_3_0_d.h:
  VIRTUAL_ADDR `0x34A7â†’0x349A`, APE1_CNTL `0x34A8â†’0x349B`. Also `0x3491` is
  CONTEXT_STATUS (RO) not CONTEXT_CNTL â€” prior Phase 3c writes were no-ops.
- result: build OK (10.36 MB), VIRTUAL_ADDR now reads back **0x00006000** (write
  stuck after offset fix) âœ… but VM fault unchanged: `PF_STATUS=0x01078001`,
  `PF_ADDR=0xF400075000`. Conclusion: SDMA VIRTUAL_ADDR is per-VMID PTBR
  (Linux keeps it 0), not a CSA pointer. Fault originates elsewhere.
- next: decode `PF_STATUS=0x01078001` MEMORY_CLIENT_ID + VMID to identify the
  real originator IP (could be CP, SMU, MEC1, IH â€” not necessarily SDMA).
  Possibly fault is from a different IP block triggered by SDMA init flow.

---

## 2026-04-28 â€” Polaris SDMA: post-unhalt CONTEXT_CNTL=0 write FAILED (F32 reasserts)
- scope: kernel/src/drivers/amdgpu/firmware.rs (Phase 3c block after F32 unhalt)
- did: added Linux-style post-unhalt clear of `SDMA0_GFX_CONTEXT_CNTL` (0xD244)
  for both engines, mirroring `sdma_v3_0_ctx_switch_enable()` ordering. Pre-unhalt
  write retained as defense in depth.
- result: build OK (10.7 MB), board recovered from init fault (~30s auto-recovery
  via PCIe CTO+AER+0xCF9). `gpu sdma regs` post-init shows CONTEXT_CNTL=0x5
  unchanged + same fault `PF_STATUS=0x01078001 ADDR=0xF400075000`. F32 microcode
  actively rewrites bit 2 (CTX_CACHE_ENABLE), MMIO write does not stick.
- next: fallback step 2 â€” allocate 64 KiB CSA in GTT, GART-map, program
  `SDMA0_GFX_VIRTUAL_ADDR` (0xD268, currently 0) **before** F32 unhalt so default
  `FB+0x75000` CSA is never used. Bit 2 then becomes harmless.

---

## 2026-04-28 â€” Polaris SDMA fault root cause: CONTEXT_CNTL=0x5 bit 2 (CSA in VRAM)
- scope: kernel/src/shell/vm.rs (regs offsets fixed Polaris/sdma_v3_0; +SYS_APR/VM_CTX0 dump)
- did: corrected wrong Navi/sdma_v4 offsets in `gpu sdma regs` (was reading
  UCODE_DATA at 0x3401 thinking it was SDMA0_CNTL). Added system aperture +
  VM context0 PT base/range/default-addr dumps. Captured post-init state.
- result: SDMA0_CNTL=0x01 âœ… (TRAP only, our write sticks).
  CHICKEN_BITS=0x00810007 (set by init, conformity TBD).
  CONTEXT_CNTL=0x5 stays after F32 unhalt â†’ bit 2 = CSA cache enabled.
  RB/WB/POLL/IB pointers all in GART (0xFF00xxxxxx) âœ…
  SYS_APR: LOW=0 (should be 0x3D000000=vram>>18), HIGH=0x3D7FFF âœ…, DEFAULT=0xF400400000 âœ…
  VM_CTX0_PT_START..END=0xFF000000..0xFF0FFFF000 âœ… (GART OK)
  PF: status=0x01078001 addr=MC 0xF400075000 (FB low, OUTSIDE GART aperture)
- conclusion: F32 microcode writes to MC 0xF400075000 directly. Since VMID=0
  is configured for GART translation only ([0xFF00000000..0xFF0FFFF000]),
  any non-GART write via VMID=0 â†’ range fault. The write target = "F32
  Context Save Area" at default VRAM offset. CONTEXT_CNTL=0x5 bit 2 enables
  this CSA path. Linux either disables it (post-unhalt force write =0) or
  allocates a real CSA buffer in GTT and points F32 at it.
- next:
    1. Force CONTEXT_CNTL=0 in firmware.rs AFTER F32 unhalt (Linux ctx_switch
       order). Currently we write before unhalt and F32 reasserts to 0x5.
    2. If still faults, allocate 16 KiB CSA in sysRAM, GART-map, point F32
       at it via SDMAx_GFX_VIRTUAL_ADDR + SDMAx_RLC0_VIRTUAL_ADDR pair.
    3. Investigate SYS_APR_LOW=0 anomaly (separate issue, not blocking).

---

## 2026-04-28 â€” Polaris SDMA: `gpu sdma regs` read-only dump + post-init analysis
- scope: kernel/src/shell/vm.rs (NEW `regs` subcommand under "sdma"; ~110 lines, read-only)
- did: dump every SDMA0 register suspected of holding a stale VRAM pointer
  (engine-level + RB + WPTR_POLL + IB + CONTEXT_CNTL + VIRTUAL_ADDR + APE1 +
  HDP_NONSURFACE + VM_CTX0_PF). Added FB-aperture decode that flags any
  HI:LO pair pointing inside VRAM. Wraps with watchdog_arm(3s).
- result: build OK 10395640 B; HW validated post-reboot.
  Pre-init: VIRTUAL_ADDR=0x2 (ATC bit), CONTEXT_CNTL=0x5, WPTR_POLL_CNTL=0x401000.
  Post-init (post-fault, post-auto-recovery): RB_BASE=0xFF000000 (GART, OK),
  RPTR_ADDR=0xFF_0000_4000 (GART), WPTR_POLL_ADDR=0xFF_0000_4040 (GART),
  CONTEXT_CNTL=0x5 (should be 0!), VIRTUAL_ADDR=0x0 (cleared OK).
  âš ï¸ NO programmed SDMA pointer points to VRAM 0xF400075000 â†’ fault source
  is NOT a programmed RB/WB/POLL/IB. Likely culprits:
    1. VM_CONTEXT0_PROTECTION_FAULT_DEFAULT_ADDR pointing to VRAM low
    2. F32 microcode-internal CSA (CONTEXT_CNTL=5 bit 2 = CSA cache)
    3. SDMA RLC/PG context save areas not programmed
- next: extend `gpu sdma regs` to also dump VM_CTX0_PT_BASE/START/END/DEFAULT_ADDR;
  if DEFAULT_ADDR points to VRAM 0x75000 â†’ Linux fix = allocate dummy page in
  GTT, write its MC PFN to DEFAULT_ADDR. Also: post-F32-start, force write
  CONTEXT_CNTL=0 (Linux gfx_v8 ctx_switch_enable pattern).

---

## 2026-04-28 â€” TrustStrudel: feature gating + hub presets + propagation contract
- scope: kernel/Cargo.toml (+`strudel = []`, added to `trustos-audio`),
  kernel/src/audio/mod.rs (cfg-gated `pub mod strudel_dsl` + 3 dsl_* fns),
  kernel/src/shell/vm.rs (cfg-gated `live dsl` arm with rebuild fallback),
  scripts/trustos-hub.ps1 (new presets `strudel`, `audio-full`; updated
  `audio` Desc to mention TrustStrudel DSL; menu letters `t` and `f`),
  /memories/repo/feature_propagation_contract.md (NEW â€” checklist).
- did: P1 strudel_dsl was unconditionally compiled (oversight). Now properly
  gated behind `feature = "strudel"` (auto-included by `trustos-audio` and
  `trustos-audio-full`). Created propagation contract memory: any new feature
  MUST update code + Cargo + cfg + shell + hub + docs + memory + journal +
  multi-profile build verification.
- result: `cargo check --release` (default, has strudel via trustos-audio?
  no â€” defaults are jarvis/emulators/etc, NO strudel) â†’ 31.57s green.
  `cargo check --release --no-default-features --features trustos-audio`
  â†’ 21.32s green. DSL compiled-out cleanly when feature off.
- next: P3 synths (Pulse/Pink/Brown/Supersaw/FM), P4 fx routing, P5 combinators.
  Roadmap: docs/TRUSTSTRUDEL_FEATURE_PARITY.md.

---

## 2026-04-28 â€” TrustStrudel parity P1 + P2 + shell `live dsl` wired (build green)
- scope: kernel/src/audio/strudel_dsl/{lexer,parser,scales,eval,mod}.rs (NEW),
  kernel/src/audio/mod.rs (+`pub mod strudel_dsl;` + dsl_set_track/dsl_oneshot/dsl_inspect),
  kernel/src/audio/strudel.rs (mini-notation extensions),
  kernel/src/shell/vm.rs (`live dsl <play|parse|d1..d8> "<expr>"`).
- did: P1 â€” chained method DSL (`s("bd sd").gain(0.8).lpf(800).acidenv(0.7)`):
  lexer (Q16.16 fixed-point numbers), recursive-descent parser, 16-scale registry
  (`g:minor`, `c:dorian`, â€¦), evaluator with 40+ method dispatchers, source kinds
  (drums/notes/scale-degrees/freq), Controls struct (gain/pan/lpf/hpf/bpf + envs +
  room/delay/duck/orbit). All Q16.16, no f32 in kernel paths.
  P2 â€” mini-notation: `<a b c>` cycle alternation (first-element semantics for now,
  cycle-index threading deferred to P9 hot-reload), `?` deterministic degrade,
  `!N` replicate, `@N`/`/N` elongate, `(n,k[,o])` Euclidean rhythm (Bresenham
  distribution + rotation). All operators stack with existing `*N` / `[...]`.
  Shell â€” `live dsl play "<expr>"`, `live dsl parse "<expr>"`, `live dsl d1..d8 "<expr>"`.
- result: `cargo check --release -p trustos_kernel` finished clean (27s, 0 errors,
  0 unused warnings). Binary 10.3 MB. NOT yet PXE-tested on BTC-250PRO.
- next: P3 synth waveforms (Pulse/Pink/Brown/Supersaw/FM â€” touches 3 exhaustive
  matches), P4 fx routing (Schroeder reverb + sidechain duck via Controls), P5
  combinators (stack/jux/every), P6-P9 visual REPL. Roadmap: docs/TRUSTSTRUDEL_FEATURE_PARITY.md.

---

## 2026-04-28 â€” PCIe bus-lock auto-recovery (CTO + AERâ†’NMIâ†’0xCF9)
- scope: `kernel/src/pcie_recovery.rs` (new), `kernel/src/main.rs` (mod),
  `kernel/src/drivers/amdgpu/mod.rs` (init hook in `init_gpu`).
- did: new module arms three layers on GPU enumeration:
  (1) PCIe Completion Timeout (DEVCTL2 range B 16-55ms, CTO Disable=0)
      â†’ CPU MMIO read returns synthetic 0xFFFFFFFF instead of stalling;
  (2) AER on GPU + Root Port: clear status, unmask all uncorrectable,
      severity=fatal â†’ SERR# escalation;
  (3) Root Port: PCI CMD bit8 (SERR# Enable), RTCTL bits 2:0, AER Root
      Cmd bits 2:0 â†’ fatal AER triggers SERR# â†’ NMI â†’ existing NMI
      handler reboots via `acpi::reboot()` (port 0xCF9).
  Helper `mmio_read32_safe()` available for hot-path callers (detects
  CTO synthetic FFFFFFFF, double-reads to filter false positives, then
  reboots). Existing driver paths unchanged â€” opt-in.
- result: build OK 15:17:11 (10391552 B). Deployed PXE.
- next: wire `mmio_read32_safe` into `gpu sdma test` reads (RPTR/STATUS
  poll loop) so SDMA hangs trigger reboot in <60ms instead of physical
  power-cycle. Verify on hardware once board recovers.

---

## 2026-04-28 â€” Auto-recovery hardening (TCO 30sâ†’10s, host watchdog script)
- scope: `kernel/src/debug/remoteshell.rs`, `tools/debug/board_watchdog.py` (new),
  `/memories/repo/board_auto_recovery.md` (new).
- did: TCO timeout 30sâ†’10s (17 steps Ã— 0.6s), APIC software watchdog 15sâ†’8s.
  Created host-side `board_watchdog.py`: ping every 5s, UDP echo every 15s,
  declares dead after 60s of silence, optional `$env:TRUSTOS_PDU_URL` HTTP
  call (Shelly/Kasa). Documented why physical power-cycle is still required
  for PCIe bus locks (chipset reset blocked by hung bus).
- result: build OK (10392104 B, 15:06:06). Deployed to PXE.
- next: buy Shelly Plug S (~15â‚¬) â†’ set `$env:TRUSTOS_PDU_URL` â†’ fully
  hands-free recovery. Verify TCO actually fires by looking at next-boot
  `gpu hwdiag tco` for SECOND_TO_STS=1.

---

## 2026-04-28 â€” Board IP DHCP flips: .110 vs .111 (auto-discover via ARP)
- scope: `_send_cmd.py` (recreated), `memory/repo/network_topology.md`, `CLAUDE.md`, `AGENTS.md`.
- did: aprÃ¨s reboot, board reprise sur **10.0.0.110** (MAC b8:97:5a:d9:54:66),
  pas .111 comme hardcodÃ© partout. `_send_cmd.py` re-Ã©crit pour rÃ©soudre via
  `arp -a` + override `$env:TRUSTOS_IP`. Docs MAJ: ne plus hardcoder l'IP,
  utiliser MAC comme identifiant stable.
- result: `python _send_cmd.py "echo alive"` â†’ `target=10.0.0.110:7777` â†’
  `alive` âœ…. Board UDP shell rÃ©pond aprÃ¨s power-cycle implicite.
- next: appliquer le mÃªme auto-discover aux autres scripts (`shell_send.py`,
  `pxe_monitor.py`, `trustos-hub.ps1`, etc.) si re-touchÃ©s.

---

## 2026-04-28 â€” Polaris SDMA: 4 Linux-conformance fixes deployed (memset, CONTEXT_CNTL, WPTR_POLL_ADDR, wbinvd)
- scope: kernel/src/drivers/amdgpu/firmware.rs (config_ring closure + polaris_sdma_self_test).
- did: Audit vs amdgpulinuxpipeline.md â†’ 3 deltas. Implemented:
  (1) explicit memset ring 8KiB=0 before RB_BASE program (Linux step A);
  (2) write SDMA0_GFX_CONTEXT_CNTL=0 (clear stale context flags pre-config);
  (3) program SDMA0_GFX_RB_WPTR_POLL_ADDR_HI/LO = poll_mc (was missing â†’ F32
      polled garbage â†’ VM fault on stale addr); (4) wbinvd before/after
      poll_cpu writes in self-test (no PCIe ATS â†’ CPU cache stale to GPU MC).
  Closure signature now takes ring_cpu_addr param; both call sites updated.
- result: build OK (10349424 B @ 14:23). Deploy â†’ board silent (no UDP/ping
  >2 min). Hardware likely needs power-cycle. Test deferred.
- next: power-cycle board, run `gpu sdma init/test`, diff RPTR/STATUS/FAULT.

---

## 2026-04-28 â€” Polaris SDMA: RB_CNTL bit-decode fix (0x1215â†’0x1015), VM fault returns
- scope: kernel/src/drivers/amdgpu/firmware.rs (rb_cntl_final, GART_PTE_SYSRAM).
- did: Dropped RB_CNTL bit 9 (was misnamed RPTR_WB_EN, actually RB_SWAP_ENABLE
  â€” byte swap, corrupts ring on x86). Kept bit 12 = real RPTR_WRITEBACK_ENABLE.
  Also dropped GART PTE SN (snoop) bit â€” no PCIe ATS bare-metal.
- result: RB_CNTL=0x1015 confirmed on hw, priv=0, rb_empty=0, mc_rreq_idle=0,
  RPTR=0 still. FAULT returned 0x01078001 FAULT_ADDR=0xF400075 CLIENT=0x40 RW=1
  (SDMA write fault). Ring[0..8] CPU view all 0 â€” test didn't push payload OR
  cache coherency issue without snoop. WPTR=0x4 = 1 dword pushed.
- next: investigate fault source (likely WB sysRAM MC=0xFF000... w/o snoop)
  + verify why CPU ring writes don't appear in ring buffer.

## 2026-04-28 â€” Polaris SDMA: GART PTE proven correct, F32 stuck in MC read
- scope: kernel/src/drivers/amdgpu/firmware.rs (polaris_sdma_ptediff added),
  kernel/src/shell/vm.rs (`gpu sdma ptediff` cmd).
- did: New diag dumps GART table[0..7] decoded vs ring_phys + ring CPU view.
  Also added HDP flush + L2 invalidate before WPTR kick in self-test.
- result: PTE[0]=0x119324067 V=1 S=1 SN=1 R=1 W=1 phys=0x119324000 â€” MATCH
  ring_phys exactly. Ring CPU view = zeros (NOPs). After WPTR=4 kick:
  STATUS=0x46DC6C52 (rb_empty=0, mc_rreq_idle=0 â†’ fetching) but RPTR stays
  0 indefinitely â†’ F32 stalls in pending MC read, response never returns.
  GART config validated; eliminates PTE/aperture/L2 hypothesis.
- next: test (a) PTE without SNOOP bit (sysRAM via GART maybe needs IOMMU
  for snoop); fallback (b) F32 ucode/RLC handshake interlock.

## 2026-04-28 â€” Polaris SDMA F32 fetches! AUTO_CTXSW was the lock
- scope: kernel/src/drivers/amdgpu/firmware.rs (SDMA0_CNTL phase4+5).
- did: SDMA0_CNTL 0x08050403 â†’ 0x00000001 (TRAP only, AUTO_CTXSW off,
  ATC off). Linux ref: sdma_v3_0_ctx_switch_enable(false) when no RLC.
  Insight: AMD's atomic security model â€” F32 ucode refuses to fetch user
  rings while AUTO_CTXSW=1 without RLC handshake. Skip RLC â†’ silent spin.
- result: STATUS 0x46DEE856 â†’ **0x46DC6452** (rb_empty 1â†’0, mc_rreq_idle
  1â†’0). F32 now actively issues MC read for ring. RPTR still 0 â€” fetched
  data doesn't decode as valid NOP. SDMA_CNTL=0x01 confirmed in dump.
- next: HDP_MEM_COHERENCY_FLUSH_CNTL after CPU ring writes (stale L2);
  GMC L2 INVALIDATE wider mask; verify GART PTE MTYPE/snoop bits.

## 2026-04-28 â€” Polaris SDMA: 3-of-4 fixes deployed, F32 ring-fetch new blocker
- scope: kernel/src/drivers/amdgpu/firmware.rs (init+config_ring),
  kernel/src/shell/vm.rs (`gpu sdma ring` cmd).
- did: (1) removed auto-self-test from `polaris_sdma_full_init`,
  (2) removed WPTR pre-kick (8*4â†’0) in config_ring (Linux mainline order),
  (3) added `gpu sdma ring` shell cmd (ring/WB CPU dump). Step (4) DCE
  CRTC disable skipped (no offsets). Made POLARIS_BUF + fields pub(crate).
- result: build OK 2m05s. Post `gpu sdma init`: PF=0 âœ“ (SYSTEM_APERTURE
  still solid), WPTR=0 âœ“, ST=0x46DEE856 (idle=0/halt=0/RB_EMPTY=1).
  `gpu sdma test` Level 0 PASS (CPU ring rw OK), Level NOP FAIL â€”
  F32 doesn't fetch ring. F32_CNTL=0x0 (running), POLL_CNTL=0,
  DOORBELL=0. F32 alive but not advancing RPTR.
- next: try WPTR_POLL_CNTL ENABLE=1, verify F32 ucode load (readback),
  check SDMA0_CNTL ATC bit. `gpu sdma gart` STILL hangs â€” CPU virt
  access UB to fix. Detailed hypothesis list in /memories/session/polaris_pf_isolation.md.

## 2026-04-28 â€” Polaris VM PF_STATUS=0x01078001 â†’ SYSTEM_APERTURE fix VALIDATED
- scope: kernel/src/drivers/amdgpu/firmware.rs (polaris_gmc_init step 3),
  kernel/src/shell/vm.rs (gpu sdma vmctl norng/minim modes + fault decode).
- did: PF_ADDR reg corrected (0x537â†’0x53E). Decoded MC=0xF400075000
  CLIENT=0x78 (DMIF/DCE) WRITE RANGE_FAULT VMID=0. `vmctl norng` did NOT
  clear PF (re-asserts continuously). Real fix: program SYSTEM_APERTURE
  to FB range (Linux gmc_v8_0_mc_program). Patched step 3:
    APR_LOW=(vram_start>>18); APR_HI=(vram_end>>18);
    APR_DEF=((vram_start+0x400000)>>12).
- result: BOARD VALIDATED post power-cycle. After `gpu sdma init`+8s,
  `gpu sdma fault` reports PF_STATUS=0x00000000 (was 0x01078001) twice
  in a row with no activity â†’ fix confirmed. RPTR=0 WPTR=4 SDMA0
  ST=0x46DC7852 â€” but `gpu sdma test` still fails: F32 not fetching ring,
  separate blocker (CLIENT=0x40 mid-fault then DCE 0x78 re-fault). PF
  re-asserts only after activity â€” DCE periodic scanout, async to SDMA.
- next: investigate why SDMA F32 doesn't advance RPTR despite halt=0,
  RB_ENABLE=1, ring at MC=0xFF00000000, WB at 0xFF00004000. Suspect
  init-internal Step6 self_test triggers fault before F32 stabilizes,
  or doorbell/poll mode mismatch. Consider also disabling DCE/CRTCs
  pre-init to silence DMIF fault. See session memory.

## 2026-04-27 â€” GMC + IH + SDMA pipeline boot OK, F32 fault MC=0 to debug
- scope: hardware test seulement, code inchangÃ©.
- did: aprÃ¨s reboot frais, lancÃ© sÃ©quentiellement `gpu mc diag` (baseline propre)
  puis `gpu sdma init` (= polaris_sdma_full_init avec GMC+IH+SDMA V37/GART).
  Reply UDP vide (truncation >17KB) â€” pas un hang ! Re-check `gpu mc diag`
  montre toute la pile programmÃ©e.
- result: **MASSIVE WIN** â€” pipeline complet boot pour la 1Ã¨re fois :
  - GMC: L1=0x5B (en=1 SAM=3), L2=0xC0B8E03 (en=1), CTX0=0xFFFED9 (en=1)
  - IH ring: EN=1, RB_CNTL=0x1200015, BASE=0xFF000050 (GART PTE5)
  - SDMA0/1 unhalted: F32=0, RB_BASE=0xFF000000/0xFF000020 (GART PTE0/2)
  - VM FAULT 0x01078001 (FAULT_ADDR=0, CLIENT=0x40=SDMA0, RW=write)
  - SDMA0 STATUS=0x46DC7842, RPTR=0 / WPTR=8 â†’ F32 ne fetch pas
  - Self-test Level NOP fail : "F32 not fetching the ring"
- analysis: FAULT_ADDR=0 = F32 accÃ¨de MC=0 (hors plage GART [0xFF00000000,
  0xFF0FFFFF000]). CTX0 a RANGE_PROTECTION_FAULT_ENABLE_DEFAULT â†’ tout accÃ¨s
  hors GART fault. F32 firmware Polaris fait un accÃ¨s initial Ã  MC=0 (scratch
  state ? fence init ? VRAM bas ?). Fault sticky : write-back Ã  0x14D8 ou
  invalidate VMID ne clear pas (Polaris ring engine fault).
- next: option A â€” Ã©tudier ce que F32 ucode lit/Ã©crit avant d'utiliser le ring
  (ucode disasm ou logs Linux mmiotrace). Option B â€” Ã©largir CTX0 range pour
  inclure VRAM low + MC=0 zone, voir si fault disparaÃ®t. Option C â€” rÃ©tablir
  AGP/SYS_APR pour que les accÃ¨s hors GART pass-through au lieu de fault.
  Voir `/memories/repo/gpu_gart_lessons.md`.

---

## 2026-04-27 â€” gpu sdma init-vram â†’ board hard-hang (no watchdog recovery)
- scope: hardware test only, pas de code changÃ©.
- did: lancÃ© `gpu sdma init-vram` (= polaris_sdma_full_init = polaris_gmc_init
  + SDMA bringup) aprÃ¨s baseline `gpu mc diag` propre (FB_LOC=0xF5FFF400 OK,
  L1=0x503 SAM=0, L2 disabled, GART pas init, SDMA halt=1, MC type GDDR5,
  pas busy). Reply UDP = 0 bytes (truncated >17KB ou hang prÃ©coce).
- result: board ne rÃ©pond plus Ã  echo ping ni reboot UDP. Watchdog kernel
  60s n'a pas auto-rebootÃ©. **Need physical power-cycle de la BTC-250PRO.**
- next: aprÃ¨s reboot phys, NE PAS relancer init-vram tel quel. DÃ©couper en
  Ã©tapes (`gpu sdma alloc` puis `reset` puis `fw` puis `ring` puis `test`)
  pour isoler quelle Ã©tape wedge. Ajouter un mode `init-vram --step <n>`
  qui s'arrÃªte aprÃ¨s chaque sub-step + dump diag.

---

## 2026-04-27 â€” SMU port-1 hypothesis BUSTED + Phase 5b live OK
- scope: `kernel/src/drivers/amdgpu/regs.rs`, `smu.rs`, `shell/vm.rs`.
- did: ajoutÃ© SMC_IND_INDEX_1/DATA_1 (0x208/0x20C) + helpers `smu7_*_ind_p1`,
  nouveau `gpu smu probe` (compare bank 0 vs bank 1 sur 11 regs + write/read
  test SRAM @0x30000). Phase 5b live exec 19 = 2 MMIO writes + loop detector
  bail clean (SMU pas chargÃ© donc reg[0x95] never satisfies).
- result: bank 0 == bank 1 partout (PC bouge naturellement = la seule diff).
  La probe testait SRAM addresses dans bank 0 = MMIO regs random, PAS la SRAM.
  La VRAIE SRAM (bank 11) reste AAAA5555. HypothÃ¨se port-1 = **busted**.
  Bank 1 = juste un alias bank 0 pour MMIO indirect, pas une voie SRAM.
  Voir `/memories/repo/polaris_smu_bringup_status.md` (section 2026-04-27).
- next: revenir sur protection-mode start path (PPSMC_MSG_Test handshake)
  ou inspecter pour SMC_IND_INDEX_12 / autre bank dÃ©diÃ© SRAM.

## 2026-04-27 â€” AtomBIOS Phase 5a âœ… : premiÃ¨re exec LIVE sur HW (table 13)
- scope: `kernel/src/drivers/amdgpu/atom/exec.rs` (ps allocation fix).
- did: `ps` allouÃ© = 256 dwords (Linux: `kzalloc(256*4)`) au lieu de
  `ps_size.max(ps_in.len())`. Tables avec PS=0 lisaient ps[i] et
  abort "PS[0] OOB" en standalone.
- result: `gpu atom exec 13 --dry` (standalone) â†’ 32 insns/181B abort=false
  (vs 2 insns avant). `gpu atom exec 13` LIVE â†’ 32 insns/181B abort=false,
  recurse SetUniphyInstance (table 20, 12 insns, abort=false). **Board
  reste vivante aprÃ¨s** â€” premier touch MMIO rÃ©el via AtomBIOS sur la
  RX 580X. Display-block writes (0x1A**/0x1B**) seulement, pas de MC/SMU.
- next: tenter exec 19 (EnableASIC_StaticPwrMgt) live, puis asic-init live.
- pitfall (lesson): tables ATOM lisent souvent ps[] au-delÃ  du ps_size
  dÃ©clarÃ© dans le header â€” toujours allouer 256 dwords zÃ©roÃ©s.

---

## 2026-04-27 â€” AtomBIOS Phase 4 âœ… : IIO sub-program executor
- scope: `kernel/src/drivers/amdgpu/atom/exec.rs` (+~180 lines).
- did: ported Linux `atom_iio_execute()` and `atom_index_iio()`. New `iio:
  Vec<u32>` indexed by port (7-bit) populated at `AtomCtx::new` from data
  table 23 (IndirectIOAccess). MMIO read/write check `io_mode & 0x80` and
  dispatch through `atom_iio_execute(base, index, data)`. IIO READ is
  dry-skipped (returns placeholder) to avoid bus hangs on uninit MC/PLL.
- result: `gpu atom exec 71 --dry` â†’ 4783B, **71 insns** (vs 6 pre-IIO).
  `gpu atom asic-init` (dry) â†’ 17034B, traverses ASIC_Init â†’
  EnableDispPowerGating (table 13, 32 insns, abort=false âœ…) â†’ table 35
  hits insn-limit 8192 (expected: dry IIO READ = placeholder, JMP loop
  never satisfies condition).
- pitfall: `--trace` on asic-init produces output >> UDP buffer / serial
  cap, board appears hung but is fine. Run without trace; per-table
  inspection via `gpu atom exec <idx> --dry --trace`.
- next: Phase 5 â€” gated live mode with pre-bringup checks + asic-init
  ps[]/ps_size validation (table 0 PS=8 = 2 dwords; we pass [sclk, mclk]).

---

## 2026-04-27 â€” AtomBIOS Phase 3c âœ… : `gpu atom asic-init` shortcut
- scope: `kernel/src/drivers/amdgpu/atom/mod.rs` (cmd_asic_init + dispatch arm).
- did: read FirmwareInfo (data table 4) at MasterDataTable+4+8, extract
  ulDefaultEngineClock_10Khz @+8 / ulDefaultMemoryClock_10Khz @+12; build
  ps=[sclk_10k, mclk_10k]; exec table 0 (ASIC_Init). Default dry=true,
  `--live` required to override (live ASIC_Init = bus-hang risk).
- result: build OK 2m12s. Board test `gpu atom asic-init --trace` â†’
  FirmwareInfo @0x992A fw_rev=0x0F320201 sclk=300 MHz mclk=300 MHz,
  ps[0]=ps[1]=0x7530. ASIC_Init @0xAADA len=149 PS=8 â†’ CALLTABLE[71]
  (ASIC_RegistersInit) â†’ 6 insns then SETPORT io_mode=0x85 (ATOM_IO_IIO
  port 5) abort=true. Expected: IIO interpreter is Phase 4.
- next: Phase 4 â€” implement IIO sub-program executor (atom_iio_execute).

---

## 2026-04-27 â€” AtomBIOS Phase 3b âœ… + Frame.start fix + live hang risk
- scope: `kernel/src/drivers/amdgpu/atom/exec.rs` (NEW ~750L), `mod.rs` (cmd_exec + dispatch).
- did: portÃ© `atom_execute_table_locked` + handlers Move/ALU/Mul/Div/Shift/Compare/Test/Mask/Clear/Jump/CallTable/Delay/SetPort/SetRegBlock/SetFbBase/SetDataBlock/Switch/ProcessDs/PostCard/Debug. AtomCtx (mmio_base, dry_run, trace, reg_block, fb_base, data_block, io_mode, io_attr, divmul, shift, cs_*, scratch). Frame (ps/ws/ps_shift/start/code_end). MMIO via `mmio_read32/write32` mode ATOM_IO_MM (byte_off=reg_idx*4).
- bugs corrigÃ©s:
  1. `put_dst` rewindait `*ptr` â†’ Ã©crasait avancement de get_src â†’ opcodes suivants lus Ã  dst_after_read au lieu de end-of-src. Fix: `put_dst(dst_off: usize, ...)` ne mute plus le ptr. AppliquÃ© Ã  op_move/op_alu/op_clear/op_shift_legacy/op_mask.
  2. `Frame.start = code_start` (= entry+6) Ã©tait faux. Linux `ectx.start = base` (header). Fix: `frame.start = entry`. Sans Ã§a, JMP target dÃ©codait au mid-instruction (BE27 au lieu de BE21 pour le wait-loop de table 19).
- result dry: `gpu atom exec 19 --dry --trace` (EnableASIC_StaticPwrMgt) â†’ 7 insns / 27B / abort=false, matche dasm. SETPORTâ†’io_mode=ATOM_IO_MM, MOVEÃ—3 REG (BE17/BE1C/BE21), CMP eq=true (reg[0x95]=1), JMP NE skipped â†’ 0xBE21 (correct), MOVE PS, EOT.
- result live: **board hung** ~2 min aprÃ¨s `gpu atom exec 19` (sans --dry). Probablement Ã©criture MMIO[0x250]/MMIO[0x290] sur reg system avant SMU up â†’ bus lock. RÃ©cupÃ©rÃ© seul (auto-reboot kernel watchdog ou PXE).
- next: par dÃ©faut `--dry` = SAFE. Live exec demande prÃ©-conditions (SMU au moins partiellement up, ou guard "is_critical_reg"). Phase 3c `asic-init` Ã  coder en mode dry-only initialement.

---

## 2026-04-27 â€” AtomBIOS Phase 3a âœ… : disassembler live sur board
- scope: `kernel/src/drivers/amdgpu/atom/{types,interpreter,mod}.rs` (NEW types+interpreter), `_send_cmd.py`, `_send_cmd_to_file.py`.
- did: portÃ© les 124 entrÃ©es `opcode_table` de Linux `atom.c` vers Rust no_std. Disassembler skip-based avec dÃ©codage REG/PS/WS/FB/ID/IMM/PLL/MC, dst/src/attr, JMP/CALL/SWITCH/CMP/MASK/PROCESSDS. Ajout `gpu atom dasm <idx>` shell cmd. Patch EOT pour continuer le scan (tables ATOM peuvent avoir plusieurs branches via JMP).
- result: `dasm 0` (ASIC_Init) â†’ 42 insns / 143B. CallTable refs rÃ©solus: ASIC_RegistersInit (2), MemoryControllerInit (5), SetEngineClock (10), SetMemoryClock (11), EnableDispPowerGating (13). JMP LT loop @0xAAFDâ†’0x0017. `dasm 19` â†’ 7 insns / 27B. `dasm 13` (EDPG, EOT-continue fix) â†’ 69 insns / 385B (couverture complÃ¨te table 391B).
- next: Phase 3b â†’ AtomCtx + ExecCtx avec MMIO read/write callbacks, `gpu atom exec <idx> [--dry]`, puis `gpu atom asic-init` (lit FirmwareInfo data table â†’ fill ps[] â†’ exec table 0).

## 2026-04-27 â€” AtomBIOS Phase 2 âœ… : VBIOS exfil 64 KB + offline parse
- scope: `kernel/src/drivers/amdgpu/atom/{mod,bios}.rs` (chunked dump), `_atom_dump.py`, `_atom_parse_dump.py`, `vbios_full.rom`.
- did: ajoutÃ© `gpu atom dump <kb> [start_kb]` (max 32 KB par chunk car shell UDP saturation au-delÃ ). CapturÃ© 0..32 KB et 32..64 KB. Stitch â†’ `vbios_full.rom` (64 KB). Magic 55 AA OK, ATOM @ 0x228, image_size=58 KB.
- result: 22 cmd tables prÃ©sentes avec tailles + revs cohÃ©rentes. ASIC_Init @0xAADA (149B, rev 1.2), EnableDispPowerGating @0xB62E (391B, rev 2.1), EnableASIC_StaticPwrMgt @0xBE0E (33B, rev 2.1), MemoryControllerInit @0xAC80 (275B, rev 1.1). 10 MISSING attendus (DAC*/DVO/LVTMA/SMC_Init etc.) â€” VBIOS mining strippÃ©.
- next: Phase 3 â†’ porter atom.h structs (ATOM_COMMON_TABLE_HEADER, etc.) + interpreter dry-run. ASIC_Init est le candidat #1 pour exec.

## 2026-04-27 â€” AtomBIOS Phase 1 âœ… COMPLETE (parser fix validÃ© HW)
- scope: `kernel/src/drivers/amdgpu/atom/parser.rs` (offsets 0x1E/0x20/0x18/0x1A).
- did: build clean (2m25s), deploy PXE, reboot board (.110), re-test `gpu atom info|tables`.
- result: âœ… master_cmd_table=0x9756, master_data_table=0x97FC, subsys=1458:22F1 (Gigabyte). ASIC_Init [00] @0xAADA, EnableASIC_StaticPwrMgt [13] @0xBE0E, SetEngineClock @0xAE94, SetMemoryClock @0xB040, MemoryControllerInit @0xAC80, MemoryPLLInit @0xD210 â€” toutes les tables critiques pour POST sont prÃ©sentes. MISSING attendus: DAC*/DVO/LVTMA (mining VBIOS strippÃ© HDMI-only).
- next: Phase 2 â†’ `gpu atom dump 128` exfil VBIOS via netconsole pour analyse offline avec atomdis. Puis Phase 3: porter les structs atom.h en Rust no_std.

## 2026-04-27 â€” AtomBIOS Phase 1 test live: parser offsets bug dÃ©tectÃ©
- scope: deploy + test sur board (`gpu atom info|tables`), `kernel/src/drivers/amdgpu/atom/parser.rs`.
- did: dÃ©ployÃ© build atom skel sur 10.0.0.110 (board a glissÃ© .111â†’.110, IP drift, voir board_reachability_pitfalls.md). Cmd `gpu atom info` rÃ©pond, signature ATOM trouvÃ©e @0x224. Cmd `gpu atom tables` liste 32 entrÃ©es mais ASIC_Init [00] = 0x0000 (faux MISSING).
- result: bug trouvÃ© â€” mes offsets dans `AtomHeader::parse` sont dÃ©calÃ©s de +2. La struct `ATOM_ROM_HEADER` Linux a `usMasterCommandTableOffset` Ã  +0x1E (pas +0x20). J'ai traitÃ© le 1er champ comme U32 alors que c'est U16. Toutes les valeurs lues sont du champ suivant. Fix proposÃ© Ã  Nathan, attente "oui".
- next: aprÃ¨s fix â†’ re-build, re-deploy, re-test `gpu atom tables`. ASIC_Init devrait avoir un offset valide (~0x48 ou ~0x68). Puis `gpu atom dump` pour rÃ©cupÃ©rer le VBIOS complet.

---


- scope: `kernel/src/drivers/amdgpu/atom/{mod,bios,parser}.rs` (NEW), `drivers/amdgpu/mod.rs`, `shell/vm.rs` (gpu atom dispatch), `/memories/repo/atom_walker_plan.md` (NEW).
- did: dÃ©cision stratÃ©gique â€” voie D (HDMI) Ã©liminÃ©e (Ã©cran dÃ©jÃ  branchÃ©, SMU stuck quand mÃªme). LancÃ© voie A-light : port minimal de Linux `atom.c` pour exÃ©cuter ASIC_Init/DPM_Init/EnableASIC_StaticPwrMgt manquantes du VBIOS mining. Phase 1 = skeleton VBIOS read (PCI ROM BAR), parse AtomBIOS header (sig "ATOM" @offset 0x48+4, master_cmd_table, master_data_table), liste 32 command tables avec offsets. Cmds shell : `gpu atom info|tables|dump [kb]`.
- result: `cargo build --release -p trustos_kernel` clean (1m 30s). Skeleton n'Ã©crit aucun MMIO â€” phase 1 read-only safe Ã  tester sur board.
- next: tester sur BTC-250PRO (`gpu atom info` â†’ header valid, `gpu atom tables` â†’ ASIC_Init non-NULL). Capturer dump VBIOS via netconsole pour analyse offline avec `atomdis`. Phase 2 = booter Linux live + `amdgpu.atom_debug=1` pour rÃ©cupÃ©rer trace de rÃ©fÃ©rence.

---


- scope: `kernel/src/audio/{synth,live_engine,mod}.rs`, `kernel/src/shell/beat_mode.rs`
- did: replaced internal `LowPassFilter` with `MultiModeFilter` (Chamberlin SVF, integer Q15). Added `FilterMode` + `FilterSettings`, plumbed override through `SynthEngine` â†’ `Voice::note_on_with_filter`. Added `Track.filter` + setters (mode/cutoff/q/tone/clear). New BeatLab cmds `lpf|hpf|bpf|tone|reso|nofilter|filters` (top-level + `dN ...`). Cutoff/Q/tone presets in Tab autocomplete. Dashboard now shows `LP800Hz q120`. Snapshot tuple extended (8-field).
- result: `cargo build --release -p trustos_kernel` clean. Cutoff clamped to fs/6 (~8 kHz @48k) for SVF stability.
- next: Phase B â€” LFO + filter envelope. Test on board (BeatLab â†’ `d1 bd*4`, `d1 lpf 600`, `d1 reso 200`, `d1 tone 32`).

## 2026-04-27 â€” SMU Polaris: SRAM upload works, JUMP via bank 11 was destructive

- scope: `kernel/src/drivers/amdgpu/smu.rs` (`smu7_program_jump_on_start`,
  VBIOS-pre-init detection)
- did: (1) Abandoned MSG_Test in VBIOS-pre-init path â€” sending any MSG to the
  boot-ROM-idle SMU permanently flips SRAM into secure mode (all reads return
  `0xAAAA5555`, writes silently absorbed). Now we just log detection and let
  destructive flow run. (2) Reverted bank 0â†’11 "fix" in JUMP write â€” Linux
  `smu7_copy_bytes_to_smc` uses **bank 0** (`SMC_IND_INDEX_0`/`DATA_0`). Bank 11
  write to SRAM[0x0] was triggering secure mode immediately after JUMP. (3) Build
  _b17/_b18 successful (40 MB unstripped due to nightly rustc upgrade 1.97.0).
- result: _b17 destructive flow now uploads firmware correctly:
  `wrote DEADBEEF/CAFEBABE -> read DEADBEEF/CAFEBABE`,
  `SRAM readback @0x20000: A6E46FC4 6D82D211 5E5C9569 E79280BA` (matches
  expected[0]=A6E46FC4). But post-jump SRAM still goes to `0xAAAA5555` and PC
  stays in boot ROM idle (0x2A40-0x2C8C), FW_FLAGS never sets. _b18 with bank 0
  JUMP not yet runtime-tested.
- next: test _b18 â€” expect post-jump SRAM[0x0]=0xE0008040 (no sentinel), PC to
  jump out of boot ROM into firmware. If still stuck, investigate whether
  bit 30 RESET_CNTL hold is lifted by firmware itself, or if non-prot needs
  explicit MSG to start after upload.

---

## 2026-04-27 â€” BREAKTHROUGH: legacy CSM + MSG mailbox rÃ©agit (boot ROM SMU vivant)
- scope: BIOS BTC-250PRO toggle CSM/Legacy + tests `gpu smu diag` / `gpu smu send` live.
- did: passÃ© en legacy boot, rebootÃ©, observÃ© Ã©tat brut SMU sans/avec MSG=0x01. DÃ©couvert que VBIOS legacy a prÃ©-configurÃ© `SMU_INPUT_DATA=0x00020000` (descripteur firmware) et que le boot ROM rÃ©pond au mailbox.
- result: `SMC_RESP_0=0x01` (ready), `SMC_PC_C oscille 0x2950-0x2C8C` (boot ROM idle wait), `SRAM[0x20000]=0xA6E46FC4` (notre upload reste). MSG=0x01 fait basculer PC: 0x2B54â†’0x80 et SMU_FIRMWARE: 0x00030006â†’0x0003001E â†’ routine service-MSG s'exÃ©cute. **`SRAM[0x0]=0xAAAA5555` n'est PAS blocker** â€” c'est mirror boot ROM (zone < 0x100). Notre `smu7_start_smc` (reset/upload/JUMP/deassert) est DESTRUCTIVE et cassait l'Ã©tat VBIOS-pre-init.
- next: refactorer `smu7_start_smu` â€” dÃ©tecter `boot_seq_done && SMC_RESP_0==1 && SMU_INPUT_DATA!=0` â†’ skip reset/upload, envoyer directement MSG `Test` puis `PowerUpGfx` via mailbox. Augmenter `SMU7_TIMEOUT_US` Ã  1s. Path Linux pertinent: `polaris10_smumgr.c smu7_smu_init` + `reload_fw` flag.

---

## 2026-04-27 â€” Bug #4 fix (bank 0 â†’ bank 11 dans `smu7_program_jump_on_start`) sans effet
- scope: kernel/src/drivers/amdgpu/smu.rs `smu7_program_jump_on_start`, build _b14, test live.
- did: corrigÃ© l'Ã©criture du JUMP@SRAM[0]=0xE0008040 pour passer par bank 11 (Linux-faithful via `smu7_set_smc_sram_address` qui Ã©crit `mmSMC_IND_INDEX_11` et clear AUTO_INC_11). Diagnostic: probe post-jump bank0+bank11 readback.
- result: `post-jump SRAM[0x0]: bank0=0xAAAA5555 bank11=0xAAAA5555 (expect 0xE0008040)` â€” **identique au b13**. RESET_CNTL aprÃ¨s assert: 0x40000001 (bit 30 sticky 0x40000000 ne clear pas). PC tourne 0x2A40/0x2B4C/0x2C8C dans boot ROM. FW_FLAGS jamais set. Bank 11 SRAM works @ 0x20000 (DEADBEEF/CAFEBABE OK) mais @ SRAM[0] retourne 0xAAAA5555 sentinel â€” c'est le bus pattern quand le bloc SMU est tenu par bit 30 de RESET_CNTL.
- next: Bug #4 n'Ã©tait PAS le root cause. Vrai blocker: bit 30 (0x40000000) de SMC_SYSCON_RESET_CNTL reste set aprÃ¨s nos Ã©critures et empÃªche la boot ROM de release le SMU. Confirmer hypothÃ¨se atom POST manquant â€” Linux/vBIOS doit clear bit 30 via une sÃ©quence SMU non documentÃ©e OU on ne peut pas dÃ©marrer le SMU sans atom-POST prÃ©alable. DÃ©cision stratÃ©gique Nathan toujours en attente (A/B/C/D).

---

## 2026-04-27 â€” ChaÃ®ne complÃ¨te du blocker GPU confirmÃ©e (atom POST manquant)
- scope: tests live BTC-250PRO `gpu smu status` + `gpu smu start` post-_b9, /memories/repo/polaris_mec_blocker.md.
- did: re-vÃ©rifiÃ© SMU non-protected start: upload SMC ucode @0x20000 OK (readback F119C474...), reset de-asserted, mais boot ROM ne jump JAMAIS dans le ucode. PC oscille 0x2A40-0x2C8C. bank0 SRAM=0xAAAA5555 (bus error). PCI config reset retry â†’ mÃªme fail.
- result: chaÃ®ne complÃ¨te identifiÃ©e: (1) atom POST jamais exÃ©cutÃ© (mining sans display) â†’ (2) SMU boot ROM stuck â†’ (3) SMC pas RUNNING â†’ (4) PowerUpGfx impossible â†’ (5) bloc GFX non-clockÃ© â†’ (6) MEC SRAM writes silenced (0xF000EEF3) + MEC PC=0xDEADBEEF. Diagnostic probe + SMU re-test = mÃªme root cause.
- next: dÃ©cision Nathan stratÃ©gique: (A) atom POST partial walker [semaines], (B) kexec depuis Linux qui POST â†’ handover TrustOS, (C) GPU passthrough VFIO, (D) tester avec display HDMI attachÃ©. Aucune voie MMIO seule ne peut dÃ©bloquer.

---

## 2026-04-27 â€” Diagnostic probe: GFX block physiquement non-clockÃ© (root cause confirmÃ©e)
- scope: kernel/src/drivers/amdgpu/firmware.rs (load_mec ~+30 lignes RLC live + SRAM probe), /memories/repo/polaris_mec_blocker.md.
- did: ajout cold-read MEC1 SRAM addr=0 puis write CAFEBABE/DEADC0DE puis re-read; lecture live RLC_CNTL/STAT/GPM_STAT.
- result: `RLC live: CNTL=0x1 STAT=0x0 GPM_STAT=0x0` (F32 enabled mais aucune activitÃ©). `MEC1 SRAM: cold=0xF000EEF3/0xF000EEF3 â†’ after-write=0xF000E2C3(exp CAFEBABE) / 0xF000EEF3(exp DEADC0DE)`. Ã‰critures silenced (4 bits/32 sur DW0, DW1 ignorÃ©). Pattern 0xF000EEF3 = bus error read d'un bloc non alimentÃ©.
- next: vÃ©rifier si `smu7_powerup_gfx()` (Voie A) est rÃ©ellement appelÃ© dans le flow `gpu sdma cp-init`; si oui ajouter logs SMU MSG/RESP. Aucun fix MMIO seul possible â€” dÃ©pend de SMU PowerUpGfx fonctionnel.

---

## 2026-04-27 â€” Voie (a) GRBM_SOFT_RESET pulse: Ã‰LIMINÃ‰E
- scope: kernel/src/drivers/amdgpu/firmware.rs (+30 lignes prologue load_mec), /memories/repo/polaris_mec_blocker.md.
- did: ajoutÃ© pulse `GRBM_SOFT_RESET = 0x70000` (bits CP|GFX|RLC) post-halt, udelay(50), clear, udelay(50), re-halt MEC. Build 5.87 MB dÃ©ployÃ©+rebooted+testÃ©.
- result: pulse confirmÃ© (`pre=0x0 set=0x70000 clr=0x0`), donc reset register rÃ©pond, MAIS symptÃ´me INCHANGÃ‰: `MEC1 UCODE readback [0]=0xF000EEF3 [1]=0xF000EEF3 MISMATCH`, `post-unhalt: MEC1_PC=0xDEADBEEF MEC2_PC=0xDEADBEEF`. Soft reset ne dÃ©bloque PAS le SRAM MEC.
- next: voies (a)+PG+CGTT+halt+SRBM mux toutes Ã©liminÃ©es. Reste (b) sanity POST regs ou (c) atom POST walker. DÃ©cision Nathan requise pour la suite.

---

## 2026-04-27 â€” Polaris MEC ucode upload silently fails (`0xF000EEF3` sentinel)
- scope: kernel/src/drivers/amdgpu/firmware.rs (load_mec PG-disable prologue), /memories/repo/polaris_mec_blocker.md.
- did: ajoutÃ© `RLC_PG_CNTL = 0` (offset Polaris correct `0xEC43*4`, pas le `gc1()` Navi du regs.rs) avant halt MEC dans `load_mec`. Build 5.58 MB dÃ©ployÃ©+rebooted+testÃ© sur BTC-250PRO.
- result: `RLC_PG_CNTL: 0x0 -> 0x0` â†’ power gating Ã©tait DEJA off au boot. SymptÃ´me inchangÃ©: `MEC1 UCODE readback [0]=0xF000EEF3 [1]=0xF000EEF3 MISMATCH` (exp 0xC424000B/0x800001A1). `post-unhalt: MEC1_PC=0xDEADBEEF MEC2_PC=0xDEADBEEF`. CGTT_CP/RLC dÃ©jÃ  Ã  0. Mur atteint.
- next: dÃ©cision Nathan requise â€” (a) GRBM_SOFT_RESET full pulse prÃ©-load, (b) atom POST table walker (semaines), (c) abandonner GPU-on-bare-metal et passer Ã  un autre vecteur JARVIS-on-GPU (CPU-side infÃ©rence + GPU dump rÃ©servÃ© via Linux DRI).

---

## 2026-04-26 â€” Reboot warm OK + reproduction blocker MEC1
- scope: tests live BTC-250PRO sur kernel `trustos-gpudebug+mmio-trace+boot-phase-marker` (5.32 MB), shell/vm.rs (ack `gpu trace dump`), /memories/repo/polaris_mec_blocker.md.
- did: confirmÃ© que `reboot` UDP 7777 â†’ `acpi::reboot()` (8042+CF9+ACPI+triple) marche sur cette board (uptime PRE 31m â†’ POST 6s). PXE recharge `pxe_tftp/trustos_kernel`. `gpu trace clear` OK ; `gpu trace dump` flush via netconsole UDP 6666 + ack shell. Reproduit blocker MEC1 sur fresh boot via `gpu sdma init` + `gpu sdma cp-init` â†’ `MEC post-load: CNTL=0x0 PC1=0x0 PC2=0x8`.
- result: Voie A (SMU PowerUpGfx) formellement Ã©liminÃ©e â€” SMU bloque sur FW_FLAGS jamais set (PC F32 oscille 0x2A40-0x2C8C, bank0 SRAM=0xAAAA5555 bus error). Reste **Voie B = KIQ + MAP_QUEUES** (gfx_v8_0_kiq_kcq_enable, ~1-2 sem). Bug heuristique notÃ©: `mec_alive_check()` doit re-sÃ©lectionner SRBM_GFX_CNTL.MEID=0 avant lecture (sinon 2e read = 0xDEADBEEF).
- next: implÃ©menter Voie B (KIQ ring sur MEC1 pipe0 queue0). Fix mec_alive_check SRBM_GFX_CNTL avant.

## 2026-04-26 â€” MMIO ring-buffer + boot phase markers + `gpu trace` cmd
- scope: kernel/src/debug_trace.rs (mmio_ring module 1024 entries, push/dump/clear), main.rs (boot_phase! markers 1/2/3/5/10 sur serial/memory/interrupts/pci/amdgpu init), kernel/src/shell/vm.rs (gpu trace dump|clear), Cargo.toml (dÃ©jÃ  fait : mmio-trace + boot-phase-marker features).
- did: ring-buffer lock-free atomic seq + 16B/entry pour capture R/W MMIO sans dominer le timing UART. Drainage via `gpu trace dump` (netconsole). Boot markers `[BOOT-NN] label` pour bisection rapide des hangs.
- result: cargo check OK 3 configs (gpudebug, gpudebug+mmio+boot, default full). Build trustos-gpudebug+mmio-trace+boot-phase-marker = **5.32 MB** vs default **40.6 MB** (~8Ã— smaller). DÃ©ployÃ© pxe_tftp/trustos_kernel.
- next: reboot board pour valider trace ring sur hardware rÃ©el. Watchdog phase + `cp-diag --json` restent en queue.

## 2026-04-26 â€” Profil `trustos-gpudebug` + macros debug feature-gated
- scope: kernel/Cargo.toml (profil + 5 features), kernel/src/debug_trace.rs (NEW), kernel/src/main.rs (mod), kernel/src/drivers/amdgpu/mod.rs (mmio_read32/write32 instrumented).
- did: crÃ©Ã© profil slim `trustos-gpudebug = [amdgpu, netstack, tools]` + features `gpu-trace-verbose`, `mmio-trace`, `mem-trace`, `net-trace`, `boot-phase-marker`. Macros `gpu_trace!`/`mmio_trace!`/`mem_trace!`/`net_trace!`/`boot_phase!` zÃ©ro-coÃ»t quand feature off. Wired dans les 2 chokepoints MMIO (R/W).
- result: cargo check OK profil seul (24s clean) + avec mmio-trace ON (22s clean). Aucune erreur, aucun warning.
- next: utiliser `cargo build --release -p trustos_kernel --no-default-features --features "trustos-gpudebug mmio-trace"` pour debug GPU avec trace MMIO complÃ¨te sur netconsole UDP 6666. Voir `/memories/repo/gpu_build_workflow.md` + `debug_flags_philosophy.md`.

## 2026-04-26 â€” Renommage global .110â†’.111 + shell validÃ©
- scope: 45 fichiers patchÃ©s (AGENTS.md, CLAUDE.md, README.md, _*.py scratch, debugonly/, pxe_setup/, scripts/, tools/debug/, memory/). Skip: corpus_trustos.rs (Le Pacte), logs/, *.bak, gpu_linux_dump.txt.
- did: bulk replace `10.0.0.110` â†’ `10.0.0.111` via PowerShell ReadAllText/Replace/WriteAllText.
- result: `python tools\debug\shell_send.py "uptime"` â†’ `Uptime: 0h 10m 27s`. Pipeline complet end-to-end Ã  jour.
- next: surveillance heap stabilitÃ© (BUG-001 Ã  valider sur durÃ©e). Cleanup des `_*.py` scratch quand temps. Refresh `memory/gpu_debug_master.md` (V37 SYS_APR stale).

## 2026-04-26 â€” Reboot fresh + patches validÃ©s + IP board = .111 (pas .110)
- scope: BTC-250PRO reset physique, sniff UDP 6666/7779 90s
- did: captÃ© 230 paquets netconsole. Boot complet 15s. RÃ©seau actif Ã  t=0 (UnicastRX 0â†’59105). MAC b8:97:5a:d9:54:66 IP **10.0.0.111** (PAS .110 â€” toute la triangulation prÃ©cÃ©dente sniffait la mauvaise IP).
- result: patches MEM-01/03/04 build clean + boot clean. Heap free 2041 MB au boot. JARVIS first birth (Fetus). SMU toujours en Ã©chec (bank0=0xAAAA5555). BUG-001 = bug temporel confirmÃ© (pas init), validation t=3h reste Ã  faire.
- next: actualiser AGENTS.md/CLAUDE.md/copilot-instructions vers .111 ; tester shell UDP 7777 sur 10.0.0.111 ; laisser tourner 3h pour valider non-rÃ©gression heap.

## 2026-04-26 â€” MEM-01/03/04 patchÃ©s + dÃ©ployÃ©s sur board
- scope: `kernel/src/shell/mod.rs`, `kernel/src/netstack/icmp.rs`, `kernel/src/gui/engine.rs`
- did: cap CAPTURE_BUF Ã  1 MiB (drop oldest half), cap PING_RESPONSES + ICMP_ERRORS Ã  64 (FIFO), early-return get_notifications() si empty. MEM-02 retirÃ© (false positive â€” RX_QUEUE jamais push).
- result: build release OK (40641840 bytes), kernel copiÃ© dans pxe_tftp/. PXE/TFTP up (UDP 67/69 listening sur 10.0.0.1). Attente reset physique board pour test.
- next: aprÃ¨s reboot, tester shell UDP 7777 t=0 puis t=30min puis t=3h pour confirmer fix BUG-001.

## 2026-04-26 â€” Audit mÃ©moire complet â†’ 17 patches identifiÃ©s
- scope: audit `kernel/src/{shell,netstack,gui,desktop,compositor,browser}` ; nouveau `memory/memory_patches.md`
- did: subagent passe sur tous les modules suspects ; trouvÃ© 7 CRIT + 5 HIGH + 5 MED. Racine de BUG-001 = MEM-02 (RX_QUEUE unbounded), MEM-04 (NOTIFICATIONS clone/frame), MEM-01 (CAPTURE_BUF unbounded).
- result: backlog priorisÃ© prÃªt. 1 KB/frame Ã— 650k frames (3h@60fps) = 650 MB â†’ fragmentation explique le silence netstack.
- next: appliquer MEM-02 en premier (10 lignes, cap VecDeque) â€” attente "oui".

## 2026-04-26 â€” State tracking infra + BUG-001 ouvert
- scope: `memory/system_state.md` (new), `memory/known_bugs.md` (new), `/memories/repo/agent_autonomy_rules.md`
- did: crÃ©Ã© snapshot live de l'Ã©tat systÃ¨me + registre de bugs append-only avec format strict (status, severity, repro, hypothesis, evidence, fix, next). BUG-001 = "desktop OK mais netstack silent aprÃ¨s 3h uptime" â€” hypothÃ¨se leak compositor/scheduler starvation.
- result: tout AI (Copilot, Claude, etc.) a maintenant 3 rÃ©fÃ©rences croisÃ©es avant tout debug : journal (historique), system_state (snapshot live), known_bugs (registre). Pas de code touchÃ©.
- next: reboot fresh pour mesurer t_silence (init vs leak temporel), proposer patch heartbeat UDP 5s pour traÃ§abilitÃ© runtime.

## 2026-04-26 â€” Board BTC-250PRO silent on wire (triangulation complÃ¨te)
- scope: rÃ©seau dev box â†” board, pas de code touchÃ©
- did: 6 sondes en parallÃ¨le: pktmon (denied non-admin), UDP 7779/6666/7777 listen, broadcast probe sur .110/.111/.100/.255, ARP cache, DHCP/68 sniff, counters Eth, firewall, route. Bind explicite `('10.0.0.1',0)` testÃ©. Cf `/memories/repo/network_topology.md` + `/memories/repo/board_reachability_pitfalls.md` + nouveau `/memories/repo/agent_autonomy_rules.md`.
- result: **board envoie 0 paquet sur le cÃ¢ble**. ReceivedUnicastPackets=0, ARP empty, screencap/netconsole muets, aucun DHCP DISCOVER. Link UP 1 Gbps. Routing/bind/firewall Windows OK. HypothÃ¨ses restantes (prioritÃ©): (1) board bloquÃ©e prÃ©-init RTL8169 (BIOS/Limine/early panic), (2) cÃ¢ble RJ45 TX board cassÃ©, (3) TrustOS panic aprÃ¨s FB avant net init.
- next: signal visuel requis (Ã©cran board) ou bouger RJ45. Si board en TrustOS shell visuel mais silencieuse â†’ suspect netstack init. VBoxManage introuvable au PATH â†’ ajouter ou rÃ©installer VBox pour control experiment futur.

## 2026-04-26 â€” GPU autonomous recon (Phases 0+1) + reachability lesson
- scope: recon `kernel/src/drivers/amdgpu/{firmware.rs,sdma.rs,mod.rs,regs.rs,compute.rs}` + memory notes
- did: read smu_bringup_status / mec_blocker / gpu_gart_lessons / mec1_polaris_linux_ref / gpu_debug_master; preflight UDP 69 OK; `cargo build --release -p trustos_kernel` clean (1.40s)
- result: build green. Initial wrong call: declared board OFF based on ICMP timeout â€” **FAUX**, shell UDP 7777 is the real source of truth (board responds to it). Boot = TFTP IPv4 only, HTTP 8080 plus utilisÃ©. Updated `/memories/repo/pxe_setup_canonical.md` + new `/memories/repo/board_reachability_pitfalls.md`. Also: `polaris_gmc_init` writes SYS_APR=0 (Linux-matched) â€” contradicts stale `memory/gpu_debug_master.md` V37 doc (0xF400000/0xF5FFFFF).
- next: Phase 2 â€” `tools/debug/debug.ps1 -Cmd "gpu sdma cp-diag l2"` to baseline; then attack MEC1 Voie A (mec-check) or SMU SRAM@0x0 blocker. Refresh `gpu_debug_master.md`.


## 2026-04-26 â€” Comm server (PXE/TFTP) wired into debug pipeline + AGENTS.md
- scope: `AGENTS.md` Â§6, `tools/debug/debug.ps1` (UDP 69 health check), `tools/debug/README.md` (prerequisites)
- did: documented `scripts\trustos-hub.ps1` (menu 5 = start all servers) as mandatory first step; debug.ps1 now refuses to reboot if TFTP not listening (exit 2 + hint)
- result: any agent reading AGENTS.md Â§6 knows to start the hub before running the pipeline
- next: optional â€” auto-launch hub from debug.ps1 if no TFTP detected (currently just prints hint)

## 2026-04-26 â€” Debug pipeline automated (`tools/debug/`)
- scope: `tools/debug/{debug.ps1,netconsole.py,shell_send.py,parse_diag.py,journal_append.ps1,README.md,expected/}`, `AGENTS.md` Â§8
- did: built one-shot orchestrator (build â†’ SHA-skip deploy â†’ bg netconsole â†’ reboot+wait â†’ cmd â†’ parse â†’ diff/save baseline â†’ optional journal append); consolidates scratch `_*.py` at repo root
- result: `.\tools\debug\debug.ps1 -Cmd "<x>" [-Baseline â€¦] [-SaveBaseline â€¦] [-JournalTitle â€¦]` is now the canonical debug entry point
- next: cleanup obsolete `_*.py` at root once first real run validates the pipeline; pin first baselines in `tools/debug/expected/`

## 2026-04-26 â€” Journal moved to repo + AGENTS.md wired up
- scope: `memory/journal.md` (new), `AGENTS.md` Â§9, `/memories/repo/journal.md` (deprecated)
- did: migrated work journal from Copilot-only memory into the repo so all AI agents see it; updated AGENTS.md to point here
- result: journal now under git, visible by every agent
- next: append one entry per non-trivial task going forward

## 2026-04-26 â€” AGENTS.md created (universal agent contract)
- scope: `AGENTS.md` (root), wrappers `CLAUDE.md` + `.github/copilot-instructions.md`
- did: consolidated Claude + Copilot instructions into single source of truth; added Le Pacte, debug workflow, "done" criteria, journal rule
- result: any AI reading AGENTS.md gets full setup; wrappers still live but secondary
- next: optionally reduce CLAUDE.md / copilot-instructions.md to pure pointers
