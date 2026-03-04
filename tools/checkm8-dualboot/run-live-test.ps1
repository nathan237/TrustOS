# TrustOS Live Test Runner — PowerShell wrapper
# Usage: .\run-live-test.ps1 [command]
# Commands: probe, usb-scan, dump-rom, watch, read <addr> <size>

param(
    [Parameter(Position=0)]
    [string]$Command = "watch",
    
    [Parameter(Position=1)]
    [string]$Arg1 = "",
    
    [Parameter(Position=2)]
    [string]$Arg2 = ""
)

$ErrorActionPreference = "Continue"
$env:PYTHONIOENCODING = "utf-8"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$Python = "C:/Users/nathan/Documents/Scripts/OSrust/.venv/Scripts/python.exe"
$LiveTest = Join-Path $ScriptDir "live_test.py"
$ResultsDir = Join-Path $ScriptDir "results"

# Ensure results dir exists
if (-not (Test-Path $ResultsDir)) {
    New-Item -ItemType Directory -Path $ResultsDir -Force | Out-Null
}

Write-Host ""
Write-Host "=== TrustOS Live Test Runner ===" -ForegroundColor Cyan
Write-Host "Script: $LiveTest" -ForegroundColor DarkGray
Write-Host "Results: $ResultsDir" -ForegroundColor DarkGray
Write-Host ""

switch ($Command) {
    "probe" {
        & $Python $LiveTest --probe
    }
    "usb-scan" {
        & $Python $LiveTest --usb-scan
    }
    "dump-rom" {
        & $Python $LiveTest --dump-rom
    }
    "watch" {
        Write-Host "Mode WATCH: En attente de l'iPhone en DFU..." -ForegroundColor Yellow
        Write-Host "Ctrl+C pour arreter" -ForegroundColor DarkGray
        Write-Host ""
        & $Python $LiveTest --watch
    }
    "read" {
        if ($Arg1 -eq "" -or $Arg2 -eq "") {
            Write-Host "Usage: .\run-live-test.ps1 read 0x100000000 0x80000" -ForegroundColor Red
            return
        }
        & $Python $LiveTest --read $Arg1 $Arg2
    }
    "send-cmd" {
        # Send a command to the watch daemon via JSON file
        $CmdFile = Join-Path $ResultsDir "live_test_command.json"
        $cmdObj = @{
            command = $Arg1
            args = @{}
        } | ConvertTo-Json
        $cmdObj | Out-File -FilePath $CmdFile -Encoding utf8
        Write-Host "Command sent: $Arg1" -ForegroundColor Green
    }
    "result" {
        # Read latest result
        $ResultFile = Join-Path $ResultsDir "live_test_result.json"
        if (Test-Path $ResultFile) {
            Get-Content $ResultFile | ConvertFrom-Json | Format-List
        } else {
            Write-Host "No results yet" -ForegroundColor Yellow
        }
    }
    "log" {
        # Tail the log file
        $LogFile = Join-Path $ResultsDir "live_test_log.txt"
        if (Test-Path $LogFile) {
            Get-Content $LogFile -Tail 50
        } else {
            Write-Host "No log yet" -ForegroundColor Yellow
        }
    }
    default {
        Write-Host "Commands:" -ForegroundColor White
        Write-Host "  probe     - Detecter l'iPhone en DFU (pas d'exploit)" -ForegroundColor Gray
        Write-Host "  usb-scan  - Scanner les peripheriques Apple USB" -ForegroundColor Gray
        Write-Host "  dump-rom  - Dump complet du BootROM T8030" -ForegroundColor Gray
        Write-Host "  watch     - Mode daemon (attend DFU, auto-teste)" -ForegroundColor Gray
        Write-Host "  read A S  - Lire memoire physique (hex)" -ForegroundColor Gray
        Write-Host "  send-cmd  - Envoyer commande au daemon watch" -ForegroundColor Gray
        Write-Host "  result    - Afficher dernier resultat" -ForegroundColor Gray
        Write-Host "  log       - Afficher les logs" -ForegroundColor Gray
    }
}
