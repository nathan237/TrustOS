# Migration serveurlapointetech → PC Nathan (nate)
## Plan complet + scripts d'audit + validation

**Serveur source** : `serveurlapointetech` — `100.119.124.16`  
**Destination** : `nate` — `100.118.70.112` (ce PC)  
**Date préparation** : 2026-05-27

---

## ÉTAPE 0 — Setup accès distant (Claude peut lancer des commandes)

### 0.1 — Installer WSL2 sur ce PC

WSL n'est pas installé (`Class not registered`). Lancer en PowerShell admin :

```powershell
# Activer les features
dism.exe /online /enable-feature /featurename:Microsoft-Windows-Subsystem-Linux /all /norestart
dism.exe /online /enable-feature /featurename:VirtualMachinePlatform /all /norestart
# Reboot requis ensuite, puis :
wsl --install -d Ubuntu
wsl --set-default-version 2
```

### 0.2 — Générer clé SSH dans WSL

```bash
# Dans WSL Ubuntu
ssh-keygen -t ed25519 -C "nate-to-serveur" -f ~/.ssh/serveur_lapointetech
cat ~/.ssh/serveur_lapointetech.pub
# Copier cette clé publique
```

### 0.3 — Autoriser la clé sur le serveur

Sur `serveurlapointetech` (via RDP) :
```powershell
# Si serveur Windows
$authorizedKeysPath = "$env:USERPROFILE\.ssh\authorized_keys"
New-Item -Force -ItemType Directory "$env:USERPROFILE\.ssh"
Add-Content $authorizedKeysPath "COLLER_LA_CLE_PUBLIQUE_ICI"
# Activer OpenSSH Server si pas fait
Add-WindowsCapability -Online -Name OpenSSH.Server~~~~0.0.1.0
Start-Service sshd
Set-Service -Name sshd -StartupType Automatic
```

### 0.4 — Tester la connexion SSH depuis WSL

```bash
ssh -i ~/.ssh/serveur_lapointetech ton_user@100.119.124.16
# Si ça marche → Claude peut lancer des commandes via ce tunnel
```

---

## ÉTAPE 1 — Audit du serveur (lancer sur serveurlapointetech)

Sauvegarder en `audit_serveur.txt` et me donner le fichier.

```powershell
# ===== AUDIT COMPLET serveurlapointetech =====
# Lancer en PowerShell Admin sur le SERVEUR
# Output : C:\audit_serveur.txt

$out = "C:\audit_serveur.txt"
"===== AUDIT serveurlapointetech — $(Get-Date) =====" | Out-File $out

# OS
"`n--- OS ---" | Add-Content $out
Get-ComputerInfo | Select-Object WindowsProductName, WindowsVersion, OsArchitecture, CsName, CsProcessors | Format-List | Add-Content $out

# RAM / CPU
"`n--- Hardware ---" | Add-Content $out
Get-CimInstance Win32_ComputerSystem | Select-Object TotalPhysicalMemory, NumberOfLogicalProcessors | Add-Content $out

# Disques
"`n--- Disques ---" | Add-Content $out
Get-PSDrive -PSProvider FileSystem | Select-Object Name, Used, Free, Root | Add-Content $out

# Logiciels installés
"`n--- Logiciels installés ---" | Add-Content $out
Get-ItemProperty HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\* |
    Select-Object DisplayName, DisplayVersion, Publisher |
    Where-Object DisplayName |
    Sort-Object DisplayName | Add-Content $out
Get-ItemProperty HKLM:\Software\Wow6432Node\Microsoft\Windows\CurrentVersion\Uninstall\* |
    Select-Object DisplayName, DisplayVersion, Publisher |
    Where-Object DisplayName |
    Sort-Object DisplayName | Add-Content $out

# Services en cours
"`n--- Services actifs ---" | Add-Content $out
Get-Service | Where-Object Status -eq Running | Select-Object Name, DisplayName, StartType | Add-Content $out

# Services au démarrage
"`n--- Services Auto ---" | Add-Content $out
Get-Service | Where-Object StartType -eq Automatic | Select-Object Name, DisplayName, Status | Add-Content $out

# Ports ouverts
"`n--- Ports en écoute ---" | Add-Content $out
netstat -ano | findstr LISTENING | Add-Content $out

# Firewall rules
"`n--- Règles Firewall (entrantes actives) ---" | Add-Content $out
Get-NetFirewallRule | Where-Object { $_.Enabled -eq $true -and $_.Direction -eq "Inbound" } |
    Select-Object DisplayName, Action, Protocol, LocalPort | Add-Content $out

# Users locaux
"`n--- Utilisateurs locaux ---" | Add-Content $out
Get-LocalUser | Select-Object Name, Enabled, LastLogon | Add-Content $out
Get-LocalGroupMember -Group Administrators | Add-Content $out

# Tâches planifiées
"`n--- Tâches planifiées ---" | Add-Content $out
Get-ScheduledTask | Where-Object State -ne Disabled | Select-Object TaskName, TaskPath, State | Add-Content $out

# Partages réseau
"`n--- Partages réseau ---" | Add-Content $out
Get-SmbShare | Select-Object Name, Path, Description | Add-Content $out

# Variables d'env
"`n--- Variables environnement système ---" | Add-Content $out
[System.Environment]::GetEnvironmentVariables("Machine") | Add-Content $out

# IPs
"`n--- Interfaces réseau ---" | Add-Content $out
Get-NetIPAddress | Select-Object InterfaceAlias, IPAddress, PrefixLength | Add-Content $out

# Hostname / domaine
"`n--- Identité machine ---" | Add-Content $out
"Hostname: $env:COMPUTERNAME" | Add-Content $out
"Domaine: $env:USERDOMAIN" | Add-Content $out
"Tailscale IP: 100.119.124.16" | Add-Content $out

# Programmes au démarrage
"`n--- Startup programs ---" | Add-Content $out
Get-CimInstance Win32_StartupCommand | Select-Object Name, Command, Location | Add-Content $out

# Tailscale
"`n--- Tailscale ---" | Add-Content $out
& "C:\Program Files\Tailscale\tailscale.exe" status 2>&1 | Add-Content $out

# Chocolatey / winget packages
"`n--- Winget packages ---" | Add-Content $out
winget list 2>&1 | Add-Content $out

Write-Host "Audit terminé : $out"
```

---

## ÉTAPE 2 — Audit de ce PC (nate)

```powershell
# Même script, lancer ici → C:\audit_nate.txt
# Remplacer $out = "C:\audit_nate.txt"
# Comparer les deux fichiers ensuite
```

---

## ÉTAPE 3 — Migration (après analyse des audits)

Les étapes exactes dépendent du contenu des audits, mais voici la structure générale :

### 3.1 — Logiciels
```powershell
# Sur nate — installer tout ce qui est sur le serveur mais pas ici
# winget install [package] pour chaque logiciel manquant
```

### 3.2 — Services
```powershell
# Reproduire les services auto qui tournent sur le serveur
# Pour chaque service manquant :
Set-Service -Name "NomService" -StartupType Automatic
Start-Service "NomService"
```

### 3.3 — Firewall
```powershell
# Reproduire les règles firewall du serveur
New-NetFirewallRule -DisplayName "..." -Direction Inbound -Protocol TCP -LocalPort XXXX -Action Allow
```

### 3.4 — Users/Permissions
```powershell
# Créer les mêmes comptes locaux
New-LocalUser -Name "..." -Password (ConvertTo-SecureString "..." -AsPlainText -Force)
Add-LocalGroupMember -Group "Administrators" -Member "..."
```

### 3.5 — Variables d'environnement
```powershell
[System.Environment]::SetEnvironmentVariable("VAR", "valeur", "Machine")
```

### 3.6 — Tâches planifiées
```powershell
# Exporter depuis le serveur
schtasks /query /xml > taches_serveur.xml
# Importer sur nate
schtasks /create /xml taches_serveur.xml /tn "NomTache"
```

### 3.7 — Données / fichiers
```powershell
# Via Tailscale (robocopy depuis le serveur)
robocopy \\100.119.124.16\partage C:\destination /MIR /Z /MT:8
```

---

## ÉTAPE 4 — Tests de validation

Lancer sur `nate` après migration. Tous les checks doivent passer ✅.

```powershell
# ===== VALIDATION MIGRATION =====
# Comparer avec audit_serveur.txt

$pass = 0
$fail = 0

function Check($label, $condition) {
    if ($condition) {
        Write-Host "[PASS] $label" -ForegroundColor Green
        $global:pass++
    } else {
        Write-Host "[FAIL] $label" -ForegroundColor Red
        $global:fail++
    }
}

# --- Services critiques ---
# (remplacer avec les services trouvés dans audit_serveur.txt)
# Check "Service IIS en cours" ((Get-Service W3SVC).Status -eq "Running")
# Check "Service SQL Server" ((Get-Service MSSQLSERVER).Status -eq "Running")

# --- Ports ---
# Check "Port 80 en écoute" ((Get-NetTCPConnection -LocalPort 80 -ErrorAction SilentlyContinue) -ne $null)
# Check "Port 443 en écoute" ((Get-NetTCPConnection -LocalPort 443 -ErrorAction SilentlyContinue) -ne $null)

# --- Connectivity ---
Check "Ping serveur original" (Test-Connection 100.119.124.16 -Count 1 -Quiet)
Check "Ping Tailscale DNS" (Test-Connection 100.100.100.100 -Count 1 -Quiet)

# --- Users ---
Check "User admin existe" ((Get-LocalUser | Where-Object Name -eq "Administrator").Enabled -eq $true)

# --- Tailscale ---
Check "Tailscale running" ((Get-Service Tailscale).Status -eq "Running")

# --- Disque ---
$disk = Get-PSDrive C
$freeGB = [math]::Round($disk.Free / 1GB, 1)
Check "Espace disque > 10GB" ($freeGB -gt 10)

# --- RDP ---
Check "RDP activé" ((Get-ItemProperty "HKLM:\System\CurrentControlSet\Control\Terminal Server").fDenyTSConnections -eq 0)

# --- Résumé ---
Write-Host "`n===== RÉSULTAT =====" -ForegroundColor Cyan
Write-Host "PASS: $pass" -ForegroundColor Green
Write-Host "FAIL: $fail" -ForegroundColor Red
```

---

## ÉTAPE 5 — Workflow remote (une fois WSL installé)

Une fois WSL + SSH configurés, Claude peut lancer des commandes sur le serveur directement :

```bash
# Dans WSL — commande unique sur le serveur
ssh -i ~/.ssh/serveur_lapointetech ton_user@100.119.124.16 "powershell -Command 'Get-Service | Where Status -eq Running'"

# Récupérer le fichier audit
scp -i ~/.ssh/serveur_lapointetech ton_user@100.119.124.16:C:/audit_serveur.txt ~/audit_serveur.txt

# Tunnel pour accéder à un service distant localement
ssh -i ~/.ssh/serveur_lapointetech -L 8080:localhost:80 ton_user@100.119.124.16
```

---

## Checklist résumée

```
PHASE 0 — ACCÈS
[ ] Installer WSL2 sur nate (reboot requis)
[ ] Générer clé SSH ed25519 dans WSL
[ ] Activer OpenSSH Server sur serveurlapointetech (via RDP)
[ ] Copier clé publique → authorized_keys sur serveur
[ ] Tester : ssh ton_user@100.119.124.16

PHASE 1 — AUDIT
[ ] Lancer audit_serveur.ps1 sur serveurlapointetech → audit_serveur.txt
[ ] Lancer audit_nate.ps1 sur ce PC → audit_nate.txt
[ ] Me donner les deux fichiers pour analyse diff

PHASE 2 — MIGRATION (après analyse)
[ ] Installer logiciels manquants
[ ] Reproduire services
[ ] Reproduire règles firewall
[ ] Créer users/permissions
[ ] Copier variables d'environnement
[ ] Migrer tâches planifiées
[ ] Copier données (robocopy)

PHASE 3 — VALIDATION
[ ] Lancer validation.ps1 → 0 FAIL
[ ] Test RDP vers nate depuis un autre device
[ ] Test des services critiques manuellement
[ ] Comparer outputs avant/après
```

---

**Prochaine étape immédiate** : lancer le script audit sur `serveurlapointetech` via RDP et me coller le contenu de `C:\audit_serveur.txt`.
