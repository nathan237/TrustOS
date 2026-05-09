# TrustOS Workflow / Backup / Security Plan

Updated: 2026-05-09

## Current operating model

- Windows `C:` remains the OS/control plane for launchers, emergency repair scripts, scheduled tasks, and small admin wrappers.
- ADATA `D:` is the local automation/data plane:
  - `D:\TrustOS_Admin_Runbook` stores reusable commands and admin scripts.
  - `D:\TrustOS_Scripts\OSrust` stores the compact scripts/memory snapshot.
  - `D:\WSL\Debian-TrustOS\ext4.vhdx` hosts the Linux development environment.
- WSL distro `Debian-TrustOS` is the preferred autonomous work environment for Codex.
- Native Codex CLI is installed in WSL at `/usr/local/bin/codex`, version `codex-cli 0.130.0`.
- Windows launcher: `D:\TrustOS_Admin_Runbook\codex-wsl.bat`.
- Linux wrapper: `/home/natedoge/bin/trustos-codex`.
- Clean Linux repo: `/home/natedoge/work/TrustOS`, branch `recovery/post-crash-20260509`.

## Daily workflow

1. Launch Codex with `D:\TrustOS_Admin_Runbook\codex-wsl.bat`.
2. Work from WSL path `/home/natedoge/work/TrustOS` when possible.
3. Keep tiny launch/recovery/admin scripts on Windows/ADATA; keep source builds and Linux tooling inside WSL.
4. Record reusable commands in `D:\TrustOS_Admin_Runbook\commands_20260509_174137.md` or a new dated command log.
5. After changing scripts/memory, run `D:\TrustOS_Admin_Runbook\scripts\sync-trustos-scripts-to-adata.ps1`.

## Safety rules

- Do not bulk-copy the old corrupted Windows working tree into WSL.
- Do not trust `trustos_minimal.zip`; it failed ZIP central-directory validation.
- Do not delete `.recovery_workspace`, `.git.corrupt.20260509`, or `*.corrupt.20260509` until GitHub + WSL + ADATA + external/offsite restore are verified.
- Do not use robocopy-only mirrors as the sole backup; require at least one versioned backup.
- Before deleting from `C:`, prove restore by building from the restored copy.

## Proposed improvements

### Security

- Create a least-privilege scheduled task for backups instead of running ad hoc admin shells.
- Keep Codex auth copied only as minimal `~/.codex` config; never log token contents.
- Add a script integrity manifest: SHA256 for every file in `D:\TrustOS_Admin_Runbook\scripts` and `D:\TrustOS_Scripts\OSrust`.
- Add a preflight script that checks disk health, free space, WSL distro path, git branch, and uncommitted changes before destructive cleanup.

### Backup / restore

- Create a daily versioned archive of `memory`, `scripts`, `tools/debug`, and `D:\TrustOS_Admin_Runbook`.
- Export WSL weekly to `D:\WSL\exports`, then copy the newest export to external/offsite storage.
- Add a restore drill script: import WSL export under a temporary name, clone repo, run `cargo check`, then delete the temp distro only after success.
- Keep GitHub as source history; keep ADATA as fast local recovery; add one external/offsite copy before freeing more `C:` data.

### Protection

- Add a guard script that refuses to delete/move paths unless resolved paths are inside explicitly allowed roots.
- Add a zero-byte / all-NULL scanner to pre-commit checks for source, docs, and scripts.
- Add a `git fsck` + `git status` report to the daily runbook.
- Add SMART/volume checks for C: and D: into the runbook log.

### Fluidity / speed

- Prefer WSL native filesystem over `/mnt/c` for builds and searches.
- Keep build artifacts inside WSL or a dedicated data path, not the cramped Windows profile.
- Avoid concurrent WSL starts; if `E_ACCESSDENIED` appears, run `wsl --terminate Debian-TrustOS`, wait 3 seconds, retry once.
- Add a one-command `trustos doctor` wrapper that checks WSL, Codex, Rust targets, repo status, and disk space.
