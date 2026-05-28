# Instructions pour Claude sur serveurlapointetech

Salut Claude. Tu es sur `serveurlapointetech`. Suis ces etapes dans l'ordre.
A chaque etape marquee COMMIT, commit et push le fichier dans ce repo.
Le Claude sur `nate` lit tes resultats en temps reel et te donne la suite.

---

## ETAPE 1 — Audit du serveur

Lance ce script en PowerShell Admin, puis committe le fichier produit.

```powershell
$out = "audit_serveur.txt"
"===== AUDIT serveurlapointetech $(Get-Date) =====" | Out-File $out -Encoding UTF8

"`n--- OS ---" | Add-Content $out
(Get-ComputerInfo | Select-Object WindowsProductName,WindowsVersion,OsArchitecture) | Format-List | Add-Content $out

"`n--- CPU/RAM ---" | Add-Content $out
(Get-CimInstance Win32_ComputerSystem | Select-Object TotalPhysicalMemory,NumberOfLogicalProcessors) | Add-Content $out

"`n--- Disques ---" | Add-Content $out
Get-PSDrive -PSProvider FileSystem | Select-Object Name,@{N='FreeGB';E={[math]::Round($_.Free/1GB,1)}},Root | Add-Content $out

"`n--- Logiciels ---" | Add-Content $out
Get-ItemProperty "HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*" |
    Where-Object DisplayName | Select-Object DisplayName,DisplayVersion |
    Sort-Object DisplayName | Add-Content $out

"`n--- Winget ---" | Add-Content $out
winget list 2>&1 | Add-Content $out

"`n--- Services actifs ---" | Add-Content $out
Get-Service | Where-Object Status -eq Running | Select-Object Name,DisplayName | Add-Content $out

"`n--- Ports ---" | Add-Content $out
netstat -ano | findstr LISTENING | Add-Content $out

"`n--- Users ---" | Add-Content $out
Get-LocalUser | Select-Object Name,Enabled | Add-Content $out

"`n--- Variables systeme ---" | Add-Content $out
[System.Environment]::GetEnvironmentVariables("Machine") | Add-Content $out

"`n--- Outils dev ---" | Add-Content $out
"cargo: $((cargo --version 2>&1))" | Add-Content $out
"node: $((node --version 2>&1))" | Add-Content $out
"python: $((python --version 2>&1))" | Add-Content $out
"git: $((git --version 2>&1))" | Add-Content $out

"`n--- Tailscale ---" | Add-Content $out
& "C:\Program Files\Tailscale\tailscale.exe" status 2>&1 | Add-Content $out

"`n--- Taches planifiees ---" | Add-Content $out
Get-ScheduledTask | Where-Object State -ne Disabled | Select-Object TaskName,State | Add-Content $out

Write-Host "Done. Committe audit_serveur.txt"
```

Committe ensuite :
```bash
git add audit_serveur.txt
git commit -m "audit: serveurlapointetech scan"
git push origin recovery/post-crash-20260509
```

---

## ETAPE 2 — Installer l'environnement de dev (PowerShell Admin)

### 2.1 — Rust + Cargo

```powershell
winget install Rustlang.Rustup -e --accept-source-agreements --accept-package-agreements
# Redemarrer le terminal apres, puis:
rustup default stable
rustup target add x86_64-unknown-none aarch64-unknown-none riscv64gc-unknown-none-elf
rustup component add rust-src llvm-tools-preview
cargo --version
```

### 2.2 — Python

```powershell
winget install Python.Python.3.13 -e --accept-source-agreements --accept-package-agreements
python --version
pip --version
```

### 2.3 — GitHub CLI

```powershell
winget install GitHub.cli -e --accept-source-agreements --accept-package-agreements
gh --version
# Authentifier:
gh auth login
```

### 2.4 — VS Code

```powershell
winget install Microsoft.VisualStudioCode -e --accept-source-agreements --accept-package-agreements
```

### 2.5 — Claude Code

```powershell
npm install -g @anthropic-ai/claude-code
claude --version
```

### 2.6 — Cloner TrustOS complet

```powershell
cd D:\
mkdir TrustOS_Dev -ErrorAction SilentlyContinue
cd TrustOS_Dev
git clone https://github.com/nathan237/TrustOS.git --branch recovery/post-crash-20260509
cd TrustOS
```

### 2.7 — Verifier que tout compile

```powershell
cd D:\TrustOS_Dev\TrustOS
cargo check 2>&1 | tail -5
```

### 2.9 — Fix SSH pour acces depuis nate

```powershell
# 1. Verifier l'etat du service
Get-Service sshd | Select-Object Name, Status, StartType

# 2. Demarrer et mettre en auto si pas fait
Start-Service sshd
Set-Service sshd -StartupType Automatic

# 3. Ajouter regle firewall Tailscale uniquement (subnet 100.64.0.0/10)
New-NetFirewallRule `
    -Name "SSH-Tailscale-In" `
    -DisplayName "SSH via Tailscale" `
    -Enabled True `
    -Direction Inbound `
    -Protocol TCP `
    -Action Allow `
    -LocalPort 22 `
    -RemoteAddress "100.64.0.0/10" `
    -ErrorAction SilentlyContinue

# 4. Verifier que sshd ecoute
netstat -ano | findstr ":22"

# 5. Verifier le user SSH autorise
Get-Content "C:\ProgramData\ssh\sshd_config" | Select-String "Allow|Deny|PasswordAuth|PubkeyAuth"
```

Committe le resultat :
```powershell
"sshd status:" | Out-File ssh_fix.txt
Get-Service sshd | Add-Content ssh_fix.txt
"netstat port 22:" | Add-Content ssh_fix.txt
netstat -ano | findstr ":22" | Add-Content ssh_fix.txt
"firewall rules SSH:" | Add-Content ssh_fix.txt
Get-NetFirewallRule | Where-Object { $_.LocalPort -eq 22 -or $_.DisplayName -match "SSH" } | Select-Object DisplayName,Enabled,Direction | Add-Content ssh_fix.txt
```

```bash
git add ssh_fix.txt
git commit -m "migration: ssh fix"
git push origin recovery/post-crash-20260509
```

### 2.8 — Committer le resultat

```powershell
"Rust: $(cargo --version)" | Out-File setup_result.txt
"Python: $(python --version 2>&1)" | Add-Content setup_result.txt
"Node: $(node --version)" | Add-Content setup_result.txt
"Claude: $(claude --version 2>&1)" | Add-Content setup_result.txt
"gh: $(gh --version 2>&1 | head -1)" | Add-Content setup_result.txt
```

```bash
git add setup_result.txt
git commit -m "migration: dev environment setup result"
git push origin recovery/post-crash-20260509
```
