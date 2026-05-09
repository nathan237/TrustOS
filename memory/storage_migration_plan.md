# TrustOS Storage Migration Plan

Updated: 2026-05-09

## Current facts

- Windows OS / scripts host: `C:` on Samsung SSD 970 EVO Plus 500GB, healthy, ~15 GB free.
- Storage disk: `D:` label `storage` on ADATA SU655 SATA 120GB, healthy, ~77 GB free after cleanup.
- Current TrustOS repo: `C:\Users\nathan\Documents\Scripts\OSrust` (~26 GB).
- Current D: mirror source tree was removed; retained only `D:\TrustOS_SafeMirror\_logs`, `_disk_health`, and `D:\TrustOS_Scripts\OSrust` scripts/memory snapshot (~1.5 MB).
- WSL `Debian-TrustOS` is now default and lives on ADATA at `D:\WSL\Debian-TrustOS\ext4.vhdx`; old `Debian` remains on C: as fallback.

## Recovery state

- Active branch: `recovery/post-crash-20260509`.
- Last recovery commit: `d116271 recovery: add local guardrails and audit scripts`.
- Git object check: active repo passes `git fsck --full`; only dangling objects were reported.
- Recovery audit report: `.recovery_workspace\reports\recovery_audit_20260509_171351.txt`.
- 10 complete non-null recovery candidates were restored from `.recovery_workspace\found`.
- 76 active untracked text/source/doc files remain all-zero; `D:` mirror has no better copies for them.
- `trustos_minimal.zip` failed ZIP central-directory validation and must not be treated as a trusted backup.

## Target layout

- Keep Windows launch/recovery scripts on `C:`:
  - hub launchers, scheduled tasks, emergency repair scripts, tiny wrappers.
- Move project source/data workspace off cramped `C:` after recovery is clean:
  - preferred source workspace: Linux/WSL native filesystem for Codex autonomy.
  - ADATA role: scripts/logs + WSL work distro. Do not use the old mirror as authoritative source. Rebuild source from GitHub plus manually verified local files.
- Keep backups versioned, not only robocopy/zip:
  - GitHub branch for source.
  - D: mirror for fast local copy.
  - WSL/Linux workspace for autonomous builds/tools.
  - external/offsite backup before deleting any C: copy.

## Next sequence

1. Finish recovery classification for the 76 all-zero files.
2. Commit or archive the known-good recovery state.
3. Export/import Debian WSL to storage before putting TrustOS source/builds inside it; otherwise the VHDX grows on the cramped OS disk.
4. Build a clean source workspace in WSL/Linux via `git clone` from the pushed recovery branch, not from the corrupted C: working tree.
5. Keep small C: wrappers that call into WSL or D: paths.
6. Create a versioned backup job and verify restore by building from the restored copy. Only after verified restore, move large source/assets/build data away from `C:`.

## Rules

- Do not treat `D:\TrustOS_SafeMirror` as authoritative until it is rebuilt from a clean source tree.
- Do not delete `.git.corrupt.20260509`, `.recovery_workspace`, or `*.corrupt.20260509` until GitHub + D: + WSL + external/offsite copies are verified.
- Do not overwrite files from `.partial` recovery candidates without manual review.



## Admin Runbook

- Reusable command/script record: D:\TrustOS_Admin_Runbook.
- Compact scripts snapshot: D:\TrustOS_Scripts\OSrust.


## WSL / Codex launcher state

- Native WSL Codex is installed and verified: `/usr/local/bin/codex`, `codex-cli 0.130.0`.
- Windows launcher: `D:\TrustOS_Admin_Runbook\codex-wsl.bat`.
- PowerShell launcher: `D:\TrustOS_Admin_Runbook\scripts\launch-codex-wsl.ps1`.
- Linux wrapper: `/home/natedoge/bin/trustos-codex`.
- Use `D:\TrustOS_Admin_Runbook\scripts\launch-codex-wsl.ps1 -NoPull -Check` to verify without opening an interactive Codex session.
- Workflow/security/backup plan: `memory/workflow_backup_security_plan.md`.
