# TrustOS â€” Known Bugs Registry

> Append-only catalog of observed bugs. One entry per bug, status updated in-place.
> Cross-linked from `memory/journal.md` and `memory/system_state.md`.
> Visible par tous les agents AI.

## Format

```
## BUG-NNN â€” <short title>
- discovered: YYYY-MM-DD
- status: open | investigating | fixed-untested | fixed-confirmed | wontfix
- severity: critical | high | medium | low
- area: <kernel module>
- symptom: <observable behavior>
- repro: <steps to reproduce>
- hypothesis: <best guess>
- evidence: <data points>
- fix: <commit/patch ref or "TBD">
- next: <action>
```

Rules:
- Status flips â†’ update in-place (not append). Add `- updated: YYYY-MM-DD` line.
- New bugs go on top.
- Mark `fixed-confirmed` only after PXE reboot + reproduction step shows it gone.

---

## BUG-002 - Post-crash untracked files still NULL-filled
- discovered: 2026-05-09
- status: investigating
- severity: high
- area: recovery / repository hygiene / untracked source assets
- symptom: after post-crash restore, active untracked docs/scripts/source files contain only `0x00` bytes.
- repro: scan `kernel/src`, `scripts`, `tools`, `docs`, `memory`, `.github` for text-like files with zero non-null bytes.
- evidence: recovery audit found branch/Git objects healthy, then active-source scan found 86 all-zero files; 10 complete copies restored from `.recovery_workspace\found`; 76 all-zero files remain. `D:\TrustOS_SafeMirror` contains no non-null copy for those 76 paths.
- fix: partial. Restored 10 complete non-null recovery candidates on 2026-05-09. Do not overwrite from `.partial` candidates unless manually reviewed.
- next: locate older backups/GitHub/offsite copies for the 76 remaining files, or mark them lost/generated; only then commit or archive.## BUG-001 â€” Desktop responsive but network stack silent after long uptime
- discovered: 2026-04-26
- status: open
- severity: high
- area: kernel/src/desktop.rs OR compositor.rs OR memory/* OR scheduler/* OR netstack
- symptom: aprÃ¨s ~3h d'uptime, desktop visible mais extrÃªmement lent. Shell UDP 7777, netconsole 6666, screencap 7779 tous muets. Eth counter PC = 0 unicast reÃ§u de la board, ARP empty.
- repro: boot TrustOS sur BTC-250PRO, laisser tourner ~3h sur desktop, tenter shell UDP 7777 depuis PC (10.0.0.1).
- hypothesis: probable memory leak ou heap fragmentation dans compositor/desktop qui starve le scheduler â†’ tÃ¢ches rÃ©seau plus jamais ordonnancÃ©es. Alternatif: panic silencieuse netstack laissant la GUI tourner.
- evidence:
  - **2026-04-26 reboot fresh** : board IP rÃ©elle = `10.0.0.111` (PAS `.110` â€” toute la triangulation prÃ©cÃ©dente sniffait sur mauvaise IP). MAC `b8:97:5a:d9:54:66`.
  - AprÃ¨s patches MEM-01/03/04 + reboot : netconsole 230 paquets captÃ©s en 15s, ReceivedUnicastPackets 0â†’159105, prompt reÃ§u, heap free 2041 MB. RÃ©seau marche Ã  t=0 â†’ BUG = temporel.
  - (ancien) Counter Eth Windows: `ReceivedUnicastPackets=0` aprÃ¨s 3h â€” Ã  reproduire sur 10.0.0.111.
- fix: voir [memory/memory_patches.md](memory_patches.md) â€” 17 patches identifiÃ©s (MEM-01..17). Racine = MEM-02 (RX_QUEUE non capÃ©) + MEM-04 (NOTIFICATIONS clone par frame) + MEM-01 (CAPTURE_BUF unbounded).
- next:
  1. Appliquer MEM-02 (cap RX_QUEUE 256, drop oldest) â€” fix probable du silence rÃ©seau.
  2. Appliquer MEM-04 + MEM-01 â€” fragmentation heap.
  3. Reboot + test shell t=0 puis t=30min puis t=3h â†’ confirmer fix.
  4. Tous nÃ©cessitent "oui" (Ã©dits code).

