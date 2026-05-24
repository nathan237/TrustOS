#Requires -Version 5.1
# T-Wookie Management Console
# Usage: . .\twookie_console.ps1  puis  tw help

# ── Config ─────────────────────────────────────────────────────────────────────
$script:REPO      = "D:\TrustOS_SafeMirror\Documents_Scripts\OSrust"
$script:PORT      = 8895
$script:NGROK_API = "http://127.0.0.1:4040"
$script:DB_PATH   = "$script:REPO\reports\tbacon\latest\tbacon.sqlite3"
$script:SESSION   = "$script:REPO\reports\tbacon\latest\twookie_ngrok_session.json"
$script:LOG_OUT   = "$script:REPO\reports\tbacon\latest\server.out.log"
$script:LOG_ERR   = "$script:REPO\reports\tbacon\latest\server.err.log"
$script:SELF      = $MyInvocation.MyCommand.Path
if (-not $script:SELF) { $script:SELF = "$script:REPO\tools\twookie_console.ps1" }

$script:PYTHON = if (Test-Path "$script:REPO\.venv\Scripts\python.exe") {
    "$script:REPO\.venv\Scripts\python.exe"
} else { "python" }

# ── ANSI couleurs (PS 5.1 compat) ─────────────────────────────────────────────
$script:ESC = [char]27

function _c($code, $text) {
    if ($Host.UI.SupportsVirtualTerminal) { return "$($script:ESC)[${code}m${text}$($script:ESC)[0m" }
    return $text
}
function _green($t)  { _c "32" $t }
function _red($t)    { _c "31" $t }
function _yellow($t) { _c "33" $t }
function _cyan($t)   { _c "36" $t }
function _bold($t)   { _c "1"  $t }
function _dim($t)    { _c "2"  $t }

# ── Detection des processus ────────────────────────────────────────────────────

function _is_server_running {
    $p = Get-CimInstance Win32_Process -ErrorAction SilentlyContinue |
         Where-Object { $_.Name -eq "python.exe" -and $_.CommandLine -like "*tbacon_server.py*" }
    return [bool]$p
}

function _is_ngrok_running {
    $p = Get-CimInstance Win32_Process -ErrorAction SilentlyContinue |
         Where-Object { $_.Name -eq "ngrok.exe" }
    return [bool]$p
}

function _is_discord_running {
    $p = Get-CimInstance Win32_Process -ErrorAction SilentlyContinue |
         Where-Object { $_.Name -eq "tbacon-discord.exe" -or
                       ($_.Name -eq "cargo.exe" -and $_.CommandLine -like "*tbacon-discord*") }
    return [bool]$p
}

function _find_ngrok {
    $cmd = Get-Command ngrok -ErrorAction SilentlyContinue
    if ($cmd) { return $cmd.Source }
    $wg = Join-Path $env:LOCALAPPDATA "Microsoft\WinGet\Packages\Ngrok.Ngrok_Microsoft.Winget.Source_8wekyb3d8bbwe\ngrok.exe"
    if (Test-Path $wg) { return $wg }
    $found = Get-ChildItem -Path $env:LOCALAPPDATA -Recurse -Filter ngrok.exe -ErrorAction SilentlyContinue |
             Select-Object -First 1 -ExpandProperty FullName
    return $found
}

function _get_ngrok_url {
    try {
        $t = (Invoke-RestMethod "$script:NGROK_API/api/tunnels" -UseBasicParsing -ErrorAction SilentlyContinue).tunnels
        $h = $t | Where-Object { $_.proto -eq "https" } | Select-Object -First 1
        return $h.public_url
    } catch { return $null }
}

# ── Serveur ────────────────────────────────────────────────────────────────────

function _start_server {
    # Toujours tuer les anciens process tbacon avant de demarrer
    $old = Get-CimInstance Win32_Process -ErrorAction SilentlyContinue |
           Where-Object { $_.Name -eq "python.exe" -and $_.CommandLine -like "*tbacon*" }
    if ($old) {
        $old | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
        Start-Sleep -Milliseconds 400
        Write-Host ("  server   " + (_yellow "ancien process tue"))
    }

    if (-not $env:TBACON_BRIDGE_TOKEN) {
        $bytes = New-Object byte[] 24
        $rng = [Security.Cryptography.RandomNumberGenerator]::Create()
        $rng.GetBytes($bytes); $rng.Dispose()
        $env:TBACON_BRIDGE_TOKEN = ([BitConverter]::ToString($bytes) -replace "-","").ToLower()
    }

    New-Item -ItemType Directory -Force -Path (Split-Path $script:LOG_OUT) | Out-Null

    $argList = @(
        "$script:REPO\tools\tbacon\tbacon_server.py",
        "--port", "$script:PORT",
        "--db",   $script:DB_PATH
    )
    Start-Process -FilePath $script:PYTHON `
        -ArgumentList $argList `
        -WorkingDirectory $script:REPO `
        -WindowStyle Hidden `
        -RedirectStandardOutput $script:LOG_OUT `
        -RedirectStandardError  $script:LOG_ERR

    for ($i = 0; $i -lt 12; $i++) {
        Start-Sleep -Milliseconds 500
        try {
            $r = Invoke-WebRequest -UseBasicParsing "http://127.0.0.1:$script:PORT/" -ErrorAction Stop
            if ($r.StatusCode -eq 200) {
                Write-Host ("  server   " + (_green "demarre") + "  -> http://127.0.0.1:$script:PORT/")
                return
            }
        } catch {}
    }
    Write-Host ("  server   " + (_red "echec") + "  -> tw logs server")
}

function _stop_server {
    $procs = Get-CimInstance Win32_Process -ErrorAction SilentlyContinue |
             Where-Object { $_.Name -eq "python.exe" -and $_.CommandLine -like "*tbacon_server.py*" }
    if (-not $procs) { Write-Host ("  server   " + (_dim "non actif")); return }
    $procs | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
    Write-Host ("  server   " + (_yellow "arrete"))
}

# ── Ngrok ──────────────────────────────────────────────────────────────────────

function _start_ngrok {
    # Tuer ngrok existant pour forcer une session propre
    $old = Get-CimInstance Win32_Process -ErrorAction SilentlyContinue |
           Where-Object { $_.Name -eq "ngrok.exe" }
    if ($old) {
        $old | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
        Start-Sleep -Milliseconds 600
        Write-Host ("  ngrok    " + (_yellow "ancien process tue"))
    }
    $ngrokExe = _find_ngrok
    if (-not $ngrokExe) {
        Write-Host ("  ngrok    " + (_red "introuvable") + "  -> winget install --id Ngrok.Ngrok -e")
        return
    }
    Start-Process -FilePath $ngrokExe -WindowStyle Hidden `
        -ArgumentList "http", "http://127.0.0.1:$script:PORT"

    $url = $null
    for ($i = 0; $i -lt 24; $i++) {
        Start-Sleep -Milliseconds 500
        $url = _get_ngrok_url
        if ($url) { break }
    }

    if ($url) {
        $session = [ordered]@{
            created_at      = (Get-Date).ToString("o")
            local_url       = "http://127.0.0.1:$script:PORT/"
            public_url      = "$url/"
            ai_context_url  = "$url/ai-context"
            ngrok_inspector = $script:NGROK_API
        }
        New-Item -ItemType Directory -Force -Path (Split-Path $script:SESSION) | Out-Null
        $session | ConvertTo-Json -Depth 5 | Set-Content $script:SESSION -Encoding UTF8
        Write-Host ("  ngrok    " + (_green "demarre") + "  -> $url")
    } else {
        Write-Host ("  ngrok    " + (_red "demarre mais URL non detectee"))
    }
}

function _stop_ngrok {
    $procs = Get-CimInstance Win32_Process -ErrorAction SilentlyContinue |
             Where-Object { $_.Name -eq "ngrok.exe" }
    if (-not $procs) { Write-Host ("  ngrok    " + (_dim "non actif")); return }
    $procs | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
    Write-Host ("  ngrok    " + (_yellow "arrete"))
}

# ── Discord bot ────────────────────────────────────────────────────────────────

function _start_discord {
    if (_is_discord_running) { Write-Host ("  discord  " + (_yellow "deja actif")); return }
    if (-not $env:DISCORD_TOKEN) {
        Write-Host ("  discord  " + (_red "DISCORD_TOKEN non defini"))
        Write-Host (_dim "    -> `$env:DISCORD_TOKEN = 'ton-token'  puis  tw start discord")
        return
    }
    $runScript = "$script:REPO\tools\tbacon-discord\run.ps1"
    Start-Process -FilePath "powershell.exe" -WindowStyle Minimized `
        -ArgumentList "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", $runScript `
        -WorkingDirectory $script:REPO
    Start-Sleep -Seconds 2
    Write-Host ("  discord  " + (_green "demarre") + "  (cargo build en cours...)")
}

function _stop_discord {
    $procs = Get-CimInstance Win32_Process -ErrorAction SilentlyContinue |
             Where-Object { $_.Name -eq "tbacon-discord.exe" -or
                           ($_.Name -eq "cargo.exe" -and $_.CommandLine -like "*tbacon-discord*") }
    if (-not $procs) { Write-Host ("  discord  " + (_dim "non actif")); return }
    $procs | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
    Write-Host ("  discord  " + (_yellow "arrete"))
}

# ── Status ─────────────────────────────────────────────────────────────────────

function _show_status {
    Write-Host ""
    Write-Host (_bold "  Services T-Wookie")
    Write-Host (_dim  "  ----------------------------------------")

    if (_is_server_running) {
        Write-Host ("  server   " + (_green "actif") + "   http://127.0.0.1:$script:PORT/")
    } else {
        Write-Host ("  server   " + (_red "arrete"))
    }

    if (_is_ngrok_running) {
        $url = _get_ngrok_url
        if ($url) {
            Write-Host ("  ngrok    " + (_green "actif") + "   $url")
        } else {
            Write-Host ("  ngrok    " + (_yellow "demarrage..."))
        }
    } else {
        Write-Host ("  ngrok    " + (_red "arrete"))
    }

    if (_is_discord_running) {
        Write-Host ("  discord  " + (_green "actif"))
    } else {
        Write-Host ("  discord  " + (_red "arrete"))
    }

    Write-Host ""
}

# ── Rapport d'utilisation ─────────────────────────────────────────────────────

function _show_report {
    if (-not (Test-Path $script:DB_PATH)) {
        Write-Host (_red "`n  DB introuvable: $script:DB_PATH")
        Write-Host (_dim "  -> Lance d'abord: tw start`n")
        return
    }

    $py  = $script:PYTHON
    $db  = $script:DB_PATH
    $tmp = [System.IO.Path]::GetTempFileName() -replace '\.tmp$', '.py'

    @'
import sqlite3, json, time, sys
db = sqlite3.connect(sys.argv[1])
db.row_factory = sqlite3.Row
now = int(time.time())
h24 = now - 86400; h1 = now - 3600; d7 = now - 604800

def q(sql, *a):
    try: return db.execute(sql, a).fetchone()[0]
    except: return 0

def has(t):
    return bool(db.execute("SELECT name FROM sqlite_master WHERE type='table' AND name=?", (t,)).fetchone())

r = {
    "accounts_total":   q("SELECT COUNT(*) FROM twookie_accounts"),
    "accounts_24h":     q("SELECT COUNT(*) FROM twookie_accounts WHERE created_at>=?", h24),
    "accounts_7d":      q("SELECT COUNT(*) FROM twookie_accounts WHERE created_at>=?", d7),
    "messages_total":   q("SELECT COUNT(*) FROM twookie_bridge_messages"),
    "messages_24h":     q("SELECT COUNT(*) FROM twookie_bridge_messages WHERE created_at>=?", h24),
    "messages_7d":      q("SELECT COUNT(*) FROM twookie_bridge_messages WHERE created_at>=?", d7),
    "invites_total":    q("SELECT COUNT(*) FROM twookie_invites"),
    "invites_accepted": q("SELECT COUNT(*) FROM twookie_invites WHERE accepted_by IS NOT NULL"),
    "invites_pending":  q("SELECT COUNT(*) FROM twookie_invites WHERE accepted_by IS NULL AND revoked=0"),
    "contacts_total":   q("SELECT COUNT(*) FROM twookie_contacts"),
    "reqs_total": q("SELECT COUNT(*) FROM request_log") if has("request_log") else -1,
    "reqs_1h":    q("SELECT COUNT(*) FROM request_log WHERE ts>=?", h1)  if has("request_log") else -1,
    "reqs_24h":   q("SELECT COUNT(*) FROM request_log WHERE ts>=?", h24) if has("request_log") else -1,
    "recon_24h":  q("SELECT COUNT(*) FROM request_log WHERE ts>=? AND flag='recon'", h24) if has("request_log") else -1,
    "bans":       q("SELECT COUNT(*) FROM ip_bans") if has("ip_bans") else -1,
}
top = []
try:
    rows = db.execute("SELECT display_name, created_at FROM twookie_accounts ORDER BY created_at DESC LIMIT 5").fetchall()
    top = [{"name": r["display_name"], "ts": r["created_at"]} for r in rows]
except: pass
r["top_accounts"] = top
print(json.dumps(r))
db.close()
'@ | Set-Content -Path $tmp -Encoding UTF8

    $data = & $py $tmp $db 2>$null
    Remove-Item $tmp -ErrorAction SilentlyContinue

    if (-not $data) {
        Write-Host (_red "`n  Erreur lecture DB`n"); return
    }

    try { $d = $data | ConvertFrom-Json }
    catch { Write-Host (_red "`n  Erreur parsing donnees`n"); return }

    $dbSizeKb = [math]::Round((Get-Item $script:DB_PATH).Length / 1KB, 1)
    $uptime = ""
    $proc = Get-CimInstance Win32_Process -ErrorAction SilentlyContinue |
            Where-Object { $_.Name -eq "python.exe" -and $_.CommandLine -like "*tbacon_server.py*" } |
            Select-Object -First 1
    if ($proc) {
        $diff = (Get-Date) - $proc.CreationDate
        if ($diff.TotalHours -ge 1) { $uptime = "$([math]::Floor($diff.TotalHours))h$($diff.Minutes)m" }
        else { $uptime = "$($diff.Minutes)m$($diff.Seconds)s" }
    }

    $sep  = (_dim "  ----------------------------------------")
    $pad1 = 22
    $pad2 = 10

    Write-Host ""
    Write-Host (_bold "  Rapport d'utilisation T-Wookie")
    Write-Host $sep

    # Serveur
    Write-Host (_bold "  SERVEUR")
    $srvStatus = if (_is_server_running) { (_green "actif") + $(if ($uptime) { (_dim "  uptime $uptime") } else { "" }) } else { _red "arrete" }
    Write-Host ("  " + "Statut".PadRight($pad1) + $srvStatus)
    Write-Host ("  " + "DB".PadRight($pad1) + (_cyan "$dbSizeKb KB"))
    Write-Host ""

    # Comptes
    Write-Host (_bold "  COMPTES")
    Write-Host ("  " + "Total".PadRight($pad1)        + (_cyan $d.accounts_total))
    Write-Host ("  " + "Nouveaux 24h".PadRight($pad1) + (_cyan $d.accounts_24h))
    Write-Host ("  " + "Nouveaux 7j".PadRight($pad1)  + (_cyan $d.accounts_7d))
    Write-Host ("  " + "Contacts (paires)".PadRight($pad1) + (_cyan $d.contacts_total))
    if ($d.top_accounts) {
        Write-Host ("  " + (_dim "Recents:"))
        foreach ($a in $d.top_accounts) {
            $ts = [System.DateTimeOffset]::FromUnixTimeSeconds($a.ts).LocalDateTime.ToString("MM-dd HH:mm")
            Write-Host ("    " + (_dim $ts) + "  " + $a.name)
        }
    }
    Write-Host ""

    # Messages
    Write-Host (_bold "  MESSAGES (bridge E2E)")
    Write-Host ("  " + "Total".PadRight($pad1)     + (_cyan $d.messages_total))
    Write-Host ("  " + "Derniere 24h".PadRight($pad1) + (_cyan $d.messages_24h))
    Write-Host ("  " + "Derniers 7j".PadRight($pad1)  + (_cyan $d.messages_7d))
    Write-Host ""

    # Invites
    Write-Host (_bold "  INVITES")
    Write-Host ("  " + "Total crees".PadRight($pad1)   + (_cyan $d.invites_total))
    Write-Host ("  " + "Acceptees".PadRight($pad1)     + (_green $d.invites_accepted))
    Write-Host ("  " + "En attente".PadRight($pad1)    + (_yellow $d.invites_pending))
    Write-Host ""

    # Trafic
    if ($d.reqs_total -ge 0) {
        Write-Host (_bold "  TRAFIC HTTP")
        Write-Host ("  " + "Total requetes".PadRight($pad1)  + (_cyan $d.reqs_total))
        Write-Host ("  " + "Derniere heure".PadRight($pad1)  + (_cyan $d.reqs_1h))
        Write-Host ("  " + "Derniere 24h".PadRight($pad1)    + (_cyan $d.reqs_24h))
        $reconColor = if ($d.recon_24h -gt 0) { _red $d.recon_24h } else { _green $d.recon_24h }
        Write-Host ("  " + "Probes recon 24h".PadRight($pad1) + $reconColor)
        $banColor = if ($d.bans -gt 0) { _red $d.bans } else { _green "0" }
        Write-Host ("  " + "IPs bannies".PadRight($pad1)     + $banColor)
        Write-Host ""
    }

    Write-Host $sep
    $secUrl = if (_is_ngrok_running) {
        $u = _get_ngrok_url; if ($u) { "$u/admin/security" } else { "http://127.0.0.1:$script:PORT/admin/security" }
    } else { "http://127.0.0.1:$script:PORT/admin/security" }
    Write-Host ("  " + (_dim "Rapport securite complet: ") + (_cyan $secUrl))
    Write-Host ""
}

# ── Raccourci Bureau ───────────────────────────────────────────────────────────

function _create_shortcut {
    $desktopPath  = [Environment]::GetFolderPath("Desktop")
    $shortcutPath = Join-Path $desktopPath "T-Wookie Console.lnk"
    $wsh = New-Object -ComObject WScript.Shell
    $s   = $wsh.CreateShortcut($shortcutPath)
    $s.TargetPath       = "powershell.exe"
    $s.Arguments        = "-NoExit -ExecutionPolicy Bypass -NoProfile -Command `". '$script:SELF'`""
    $s.WorkingDirectory = $script:REPO
    $s.IconLocation     = "powershell.exe,0"
    $s.Description      = "T-Wookie Management Console"
    $s.Save()
    Write-Host (_green "`n  Raccourci cree: $shortcutPath`n")
}

# ── Banner ─────────────────────────────────────────────────────────────────────

function _show_banner {
    Clear-Host
    Write-Host (_cyan "
  ========================================
    T - W O O K I E   C O N S O L E
    port $script:PORT   |   tbacon server + ngrok
  ========================================
")
    Write-Host (_bold "  Commandes disponibles")
    Write-Host (_dim  "  ----------------------------------------")

    $pad = 36
    @(
        @{ c = "tw start  [all|server|ngrok|discord]"; d = "Demarrer les services" }
        @{ c = "tw stop   [all|server|ngrok|discord]"; d = "Arreter les services" }
        @{ c = "tw restart[all|server|ngrok|discord]"; d = "Redemarrer" }
        @{ c = "tw status";                            d = "Etat de tous les services" }
        @{ c = "tw url";                               d = "URL ngrok publique + routes" }
        @{ c = "tw open";                              d = "Ouvrir l'UI dans le navigateur" }
        @{ c = "tw logs   [server|ngrok]";             d = "Afficher les logs" }
        @{ c = "tw token";                             d = "Afficher / regen TBACON_BRIDGE_TOKEN" }
        @{ c = "tw report";                            d = "Rapport d utilisation (comptes, messages, trafic)" }
        @{ c = "tw shortcut";                          d = "Creer un raccourci sur le Bureau" }
        @{ c = "tw help";                              d = "Cette aide" }
    ) | ForEach-Object {
        Write-Host ("  " + (_cyan $_.c.PadRight($pad)) + " " + (_dim $_.d))
    }
    Write-Host ""
    Write-Host (_dim "  TAB = completion + descriptions   CTRL+C = quitter")
    Write-Host ""
}

# ── Commande principale ────────────────────────────────────────────────────────

function tw {
    <#
    .SYNOPSIS
        T-Wookie Management Console - gere l'environnement serveur.
    .DESCRIPTION
        Commandes: start | stop | restart | status | url | open | logs | token | shortcut | help
        Services : all | server | ngrok | discord
    .EXAMPLE
        tw start all        # Demarre server Python + tunnel ngrok
        tw status           # Etat de tous les services
        tw url              # URL publique ngrok avec toutes les routes
        tw open             # Ouvre l'UI T-Wookie dans le navigateur
        tw logs server      # Derniers logs du serveur Python
        tw shortcut         # Cree le raccourci sur le Bureau
    #>
    [CmdletBinding()]
    param(
        [Parameter(Position = 0, HelpMessage = "Commande a executer")]
        [ValidateSet('start','stop','restart','status','url','open','logs','token','report','shortcut','help')]
        [string]$Command = 'help',

        [Parameter(Position = 1, HelpMessage = "Service cible")]
        [ValidateSet('all','server','ngrok','discord')]
        [string]$Service = 'all'
    )

    switch ($Command) {

        'start' {
            Write-Host (_bold "`n  Demarrage - $Service")
            Write-Host (_dim  "  ----------------------------------------")
            switch ($Service) {
                'all'     { _start_server; _start_ngrok }
                'server'  { _start_server }
                'ngrok'   { _start_ngrok }
                'discord' { _start_discord }
            }
            Write-Host ""
        }

        'stop' {
            Write-Host (_bold "`n  Arret - $Service")
            Write-Host (_dim  "  ----------------------------------------")
            switch ($Service) {
                'all'     { _stop_server; _stop_ngrok; _stop_discord }
                'server'  { _stop_server }
                'ngrok'   { _stop_ngrok }
                'discord' { _stop_discord }
            }
            Write-Host ""
        }

        'restart' {
            Write-Host (_bold "`n  Redemarrage - $Service")
            Write-Host (_dim  "  ----------------------------------------")
            switch ($Service) {
                'all' {
                    _stop_server; _stop_ngrok; _stop_discord
                    Start-Sleep -Seconds 1
                    _start_server; _start_ngrok
                }
                'server'  { _stop_server;  Start-Sleep -Seconds 1; _start_server }
                'ngrok'   { _stop_ngrok;   Start-Sleep -Seconds 1; _start_ngrok }
                'discord' { _stop_discord; Start-Sleep -Seconds 2; _start_discord }
            }
            Write-Host ""
        }

        'status' { _show_status }

        'report' { _show_report }

        'url' {
            $url = _get_ngrok_url
            if ($url) {
                Write-Host ""
                Write-Host ("  " + (_bold "Public URL  ") + (_green $url))
                Write-Host ("  " + (_dim  "T-Wookie UI ") + "$url/twookie")
                Write-Host ("  " + (_dim  "AI Context  ") + "$url/ai-context")
                Write-Host ("  " + (_dim  "API Context ") + "$url/api/twookie/context")
                Write-Host ("  " + (_dim  "Inspector   ") + $script:NGROK_API)
                Write-Host ("  " + (_dim  "Local UI    ") + "http://127.0.0.1:$script:PORT/twookie")
                Write-Host ""
            } else {
                Write-Host (_red "`n  ngrok non actif - lance: tw start ngrok`n")
            }
        }

        'open' {
            $url = _get_ngrok_url
            $target = if ($url) { "$url/twookie" } else { "http://127.0.0.1:$script:PORT/twookie" }
            Start-Process $target
            Write-Host (_green "`n  Ouverture: $target`n")
        }

        'logs' {
            switch ($Service) {
                'server' {
                    if (Test-Path $script:LOG_OUT) {
                        Write-Host (_bold "`n  stdout - $script:LOG_OUT")
                        Write-Host (_dim  "  ----------------------------------------")
                        Get-Content $script:LOG_OUT -Tail 50 | ForEach-Object { "  $_" }
                    }
                    if (Test-Path $script:LOG_ERR) {
                        $errLines = Get-Content $script:LOG_ERR -Tail 20
                        if ($errLines) {
                            Write-Host (_bold "`n  stderr - $script:LOG_ERR")
                            Write-Host (_dim  "  ----------------------------------------")
                            $errLines | ForEach-Object { "  " + (_red $_) }
                        }
                    }
                    Write-Host ""
                }
                'ngrok' {
                    Start-Process "http://127.0.0.1:4040"
                    Write-Host (_green "`n  Ngrok inspector ouvert: $script:NGROK_API`n")
                }
                default {
                    Write-Host "`n  Usage: tw logs server  |  tw logs ngrok`n"
                }
            }
        }

        'token' {
            if ($env:TBACON_BRIDGE_TOKEN) {
                Write-Host "`n  TBACON_BRIDGE_TOKEN: " (_cyan $env:TBACON_BRIDGE_TOKEN)
            } else {
                $bytes = New-Object byte[] 24
                $rng   = [Security.Cryptography.RandomNumberGenerator]::Create()
                $rng.GetBytes($bytes); $rng.Dispose()
                $env:TBACON_BRIDGE_TOKEN = ([BitConverter]::ToString($bytes) -replace "-","").ToLower()
                Write-Host "`n  Nouveau token: " (_green $env:TBACON_BRIDGE_TOKEN)
            }
            Write-Host ""
        }

        'shortcut' { _create_shortcut }

        'help' { _show_banner; _show_status }
    }
}

# ── Tab completion avec descriptions ──────────────────────────────────────────

Register-ArgumentCompleter -CommandName tw -ParameterName Command -ScriptBlock {
    param($cmd, $param, $word, $ast, $bound)
    @(
        [System.Management.Automation.CompletionResult]::new('start',    'start',    'ParameterValue', 'Demarrer un ou tous les services')
        [System.Management.Automation.CompletionResult]::new('stop',     'stop',     'ParameterValue', 'Arreter un ou tous les services')
        [System.Management.Automation.CompletionResult]::new('restart',  'restart',  'ParameterValue', 'Redemarrer un ou tous les services')
        [System.Management.Automation.CompletionResult]::new('status',   'status',   'ParameterValue', 'Voir l etat de tous les services')
        [System.Management.Automation.CompletionResult]::new('url',      'url',      'ParameterValue', 'URL ngrok publique + routes T-Wookie')
        [System.Management.Automation.CompletionResult]::new('open',     'open',     'ParameterValue', 'Ouvrir l UI dans le navigateur')
        [System.Management.Automation.CompletionResult]::new('logs',     'logs',     'ParameterValue', 'Afficher les logs server ou ngrok inspector')
        [System.Management.Automation.CompletionResult]::new('token',    'token',    'ParameterValue', 'Afficher ou regenerer TBACON_BRIDGE_TOKEN')
        [System.Management.Automation.CompletionResult]::new('report',   'report',   'ParameterValue', 'Rapport utilisation: comptes, messages, trafic, securite')
        [System.Management.Automation.CompletionResult]::new('shortcut', 'shortcut', 'ParameterValue', 'Creer un raccourci sur le Bureau')
        [System.Management.Automation.CompletionResult]::new('help',     'help',     'ParameterValue', 'Afficher l aide et l etat des services')
    ) | Where-Object { $_.CompletionText -like "$word*" }
}

Register-ArgumentCompleter -CommandName tw -ParameterName Service -ScriptBlock {
    param($cmd, $param, $word, $ast, $bound)
    @(
        [System.Management.Automation.CompletionResult]::new('all',     'all',     'ParameterValue', 'Tous les services - server + ngrok')
        [System.Management.Automation.CompletionResult]::new('server',  'server',  'ParameterValue', "Serveur Python tbacon_server.py port $script:PORT")
        [System.Management.Automation.CompletionResult]::new('ngrok',   'ngrok',   'ParameterValue', 'Tunnel HTTPS ngrok vers le serveur local')
        [System.Management.Automation.CompletionResult]::new('discord', 'discord', 'ParameterValue', 'Bot Discord Rust tbacon-discord')
    ) | Where-Object { $_.CompletionText -like "$word*" }
}

# ── Prompt enrichi ─────────────────────────────────────────────────────────────

function prompt {
    $si = if (_is_server_running) { _green "s" } else { _red "s" }
    $ni = if (_is_ngrok_running)  { _green "n" } else { _red "n" }
    return "[${si}${ni}] " + (_cyan "tw") + " > "
}

# ── Boot ───────────────────────────────────────────────────────────────────────

_show_banner
_show_status
Write-Host (_dim "  -> tw start all    pour tout demarrer")
Write-Host (_dim "  -> tw shortcut     pour creer le raccourci Bureau")
Write-Host ""
