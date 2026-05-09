# TrustOS Disaster Recovery Runbook

This runbook is the operational recovery workflow after the 2026-05-09 crash.
The rule is simple: no critical TrustOS file should live in only one place for more than 4 hours.

## Current recovery baseline

- Repository: `C:\Users\nathan\Documents\Scripts\OSrust`
- Recovery branch: `recovery/post-crash-20260509`
- Known pushed recovery commit: `0500dbb recovery: restore 7 NULL-corrupted files from git history`
- Recovery workspace: `.recovery_workspace`
- Health logs: `.recovery_workspace\logs\health_<timestamp>.log`
- Resume logs: `.recovery_workspace\logs\resume_<timestamp>.log`

## Daily start checklist

Run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\recovery\system-health.ps1
```

Confirm:

- `cmd.exe`, `powershell.exe`, `git.exe`, and VS Code are found.
- Git branch has an upstream and `Ahead` is `0` after pushes.
- C: has enough free space. Target: at least 100 GB free.
- External backup disk is visible before running backup jobs.
- WSL status is readable if the workflow depends on WSL.

## Before reboot or risky system work

Run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\recovery\install-auto-resume.ps1
git status --short --branch
git push --all origin
```

Then reboot manually only after saving open work and attaching the external disk.
At the next logon the task `TrustOS-AutoResume` waits for the desktop, logs disks, WSL, Git, opens VS Code on this repo, and unregisters itself.

## After reboot

Check the newest resume log:

```powershell
Get-ChildItem .\.recovery_workspace\logs\resume_*.log |
  Sort-Object LastWriteTime -Descending |
  Select-Object -First 1 |
  Get-Content
```

If the log says no external disk was detected, attach it and run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\recovery\system-health.ps1
```

## Git protection workflow

Use this as the minimum loop while recovering:

```powershell
git status --short --branch
git add -A
git commit -m "recovery: checkpoint"
git push --all origin
```

Do not delete `.recovery_workspace`, `.git.corrupt.20260509`, or `*.corrupt.20260509` until the recovery branch has been reviewed and a full external backup exists.

## Backup workflow

Preferred layout:

- Copy 1: working tree on internal SSD.
- Copy 2: pushed GitHub branch.
- Copy 3: external disk backup.
- Copy 4: off-site/cloud backup when available.

Use Restic or another versioned backup tool for the external disk. A plain zip is not enough because it is hard to verify and easy to overwrite.

Minimum external-backup test:

```powershell
restic snapshots
restic check
restic restore latest --target C:\tmp\trustos_restore_test
```

Then build from the restore before trusting the backup.

## Windows recovery checks

Admin checks to run from an elevated PowerShell window:

```powershell
sfc /verifyonly
DISM /Online /Cleanup-Image /ScanHealth
chkdsk C: /scan
Get-MpComputerStatus
```

Non-admin checks:

```powershell
vssadmin list shadows
Get-ComputerRestorePoint
```

## Failure triage

If VS Code/Copilot fails:

- Check `.recovery_workspace\logs\health_<timestamp>.log`.
- Check VS Code logs under `%APPDATA%\Code\logs`.
- Confirm Copilot auth/licensing separately from local extension installation.

If Git fails:

- Confirm `git.exe` resolves from `C:\Program Files\Git\cmd\git.exe`.
- Check `git status --short --branch`.
- Do not run destructive Git commands until the current working tree is copied or committed.

If disk space is low:

- Do not run large recovery scans on C:.
- Attach external storage first.
- Move or clean generated build artifacts only after source files are pushed and backed up.
