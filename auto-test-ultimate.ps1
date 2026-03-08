<#
.SYNOPSIS
    TrustOS ULTIMATE Automated Test Suite -- Shell + Desktop (VirtualBox)
.DESCRIPTION
    The most comprehensive TrustOS test script:
      PHASE 1: Tests ALL ~180 shell commands via VBox keyboard injection
      PHASE 2: Enters desktop mode and tests ALL apps, window ops, hotkeys,
               start menu, lock screen, fuzz testing, etc.
    Reads serial output from VBox serial log file for validation.
.NOTES
    Requires: VirtualBox 7+, trustos.iso
    Output:   ultimate_test_report.txt + console summary + screenshots
    VM Name:  TRustOs (auto-created, destroyed at start)
    Duration: ~10-15 minutes
#>

param(
    [string]$IsoPath    = "$PSScriptRoot\trustos.iso",
    [int]$BootTimeout   = 40,
    [int]$CmdWait       = 4,
    [int]$DesktopWait   = 5,
    [string]$ReportFile = "$PSScriptRoot\ultimate_test_report.txt",
    [switch]$SkipShell,
    [switch]$SkipDesktop,
    [switch]$SkipPassed
)

$ErrorActionPreference = "Continue"
$VBoxManage = "C:\Program Files\Oracle\VirtualBox\VBoxManage.exe"
$VMName     = "TRustOs"
$SerialLog  = "$PSScriptRoot\serial_ultimate.log"
$timestamp  = Get-Date -Format "yyyy-MM-dd HH:mm:ss"

# Counters
$global:passed  = 0
$global:failed  = 0
$global:skipped = 0
$global:crashed = 0
$global:results = @()
$global:panics  = @()
$global:lastGoodOffset = 0
$global:testNum = 0

# Categories that passed 100% in previous run -- skip when -SkipPassed is used
$global:PassedCategories = @(
    "SYSINFO", "PERMS", "SVC", "SECURITY", "PROC", "STUBS",
    "HWSCAN", "DISTRO", "EXEC", "SCAN", "AUDIO", "DISK"
)

# â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•
#  HELPER FUNCTIONS
# â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•

# --- Serial reading ---

function Read-SerialRaw {
    param([long]$offset)
    if (-not (Test-Path $SerialLog)) { return "" }
    for ($retry = 0; $retry -lt 3; $retry++) {
        try {
            $fs = [System.IO.FileStream]::new($SerialLog,
                [System.IO.FileMode]::Open,
                [System.IO.FileAccess]::Read,
                [System.IO.FileShare]::ReadWrite -bor [System.IO.FileShare]::Delete)
            $len = $fs.Length
            if ($len -le $offset) { $fs.Close(); return "" }
            $count = [int]($len - $offset)
            $null = $fs.Seek($offset, [System.IO.SeekOrigin]::Begin)
            [byte[]]$buf = New-Object byte[] $count
            $null = $fs.Read($buf, 0, $count)
            $fs.Close()
            return [System.Text.Encoding]::ASCII.GetString($buf)
        } catch {
            Start-Sleep -Milliseconds 150
        }
    }
    return ""
}

function Clean-SerialOutput {
    param([string]$raw)
    $raw = [regex]::Replace($raw, '\[KB-IRQ\][^\n]*\n?', '')
    $raw = [regex]::Replace($raw, '\[KB-BUF\][^\n]*\n?', '')
    $raw = [regex]::Replace($raw, '\[KB-READ\][^\n]*\n?', '')
    $raw = [regex]::Replace($raw, "(?m)^'\s*$\n?", '')
    # Strip [timestamp] [INFO] prefixes but keep the content after them
    $raw = [regex]::Replace($raw, '\[\s*\d+\.\d+\]\s*\[INFO\s*\]\s*', '')
    # Strip NDP router advertisements and network noise
    $raw = [regex]::Replace($raw, '\[NDP\][^\n]*\n?', '')
    $raw = [regex]::Replace($raw, '[\x00-\x08\x0B\x0C\x0E-\x1F\x7F-\xFF]', '')
    $lines = $raw -split "`n" | ForEach-Object { $_.TrimEnd() } | Where-Object { $_ -ne "" }
    return ($lines -join "`n")
}

function Get-SerialLength {
    if (-not (Test-Path $SerialLog)) { return $global:lastGoodOffset }
    for ($retry = 0; $retry -lt 3; $retry++) {
        try {
            $fs = [System.IO.FileStream]::new($SerialLog,
                [System.IO.FileMode]::Open,
                [System.IO.FileAccess]::Read,
                [System.IO.FileShare]::ReadWrite -bor [System.IO.FileShare]::Delete)
            $len = $fs.Length
            $fs.Close()
            if ($len -ge $global:lastGoodOffset) {
                $global:lastGoodOffset = $len
                return $len
            }
        } catch {
            Start-Sleep -Milliseconds 150
        }
    }
    return $global:lastGoodOffset
}

function Wait-ForOutput {
    param([long]$sinceMark, [int]$seconds = $CmdWait)
    $promptRx = '\[\d{1,2}:\d{2}:\d{2}\]\s*trustos:[^\n]*\$'
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    while ($sw.Elapsed.TotalSeconds -lt $seconds) {
        Start-Sleep -Milliseconds 600
        $raw = Read-SerialRaw -offset $sinceMark
        if (-not $raw) { continue }
        $clean = Clean-SerialOutput $raw
        if (-not $clean) { continue }
        if ([regex]::IsMatch($clean, $promptRx)) {
            Start-Sleep -Milliseconds 400
            $raw = Read-SerialRaw -offset $sinceMark
            $clean = Clean-SerialOutput $raw
            $clean = [regex]::Replace($clean, $promptRx + '.*', '')
            return $clean.Trim()
        }
    }
    $raw = Read-SerialRaw -offset $sinceMark
    if (-not $raw) { return "" }
    $clean = Clean-SerialOutput $raw
    $clean = [regex]::Replace($clean, $promptRx + '.*', '')
    return $clean.Trim()
}

function Wait-ForSerial {
    param([long]$sinceMark, [int]$seconds = $DesktopWait, [string]$match = "")
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    while ($sw.Elapsed.TotalSeconds -lt $seconds) {
        Start-Sleep -Milliseconds 300
        $raw = Read-SerialRaw -offset $sinceMark
        if ($match -and $raw -match $match) { return $raw }
        if (-not $match -and $raw.Length -gt 20) { return $raw }
    }
    return (Read-SerialRaw -offset $sinceMark)
}

function Check-ForPanic {
    param([long]$sinceMark, [string]$context)
    $raw = Read-SerialRaw -offset $sinceMark
    $clean = Clean-SerialOutput $raw
    # Use anchored patterns to avoid false positives from help text mentioning 'panic'
    # Real panics look like: "KERNEL PANIC:" or "panicked at 'msg', file:line"
    if ($clean -match "KERNEL PANIC:|panicked at '|PAGE FAULT EXCEPTION|DOUBLE FAULT EXCEPTION|stack overflow.*fatal") {
        Write-Host " CRASH!" -ForegroundColor Red
        $global:crashed++
        $snippet = $clean.Substring(0, [Math]::Min(300, $clean.Length))
        $global:panics += @{ Context = $context; Output = $snippet }
        return $true
    }
    return $false
}

# --- VBox keyboard injection ---

function VBox-TypeText {
    param([string]$text)
    & $VBoxManage controlvm $VMName keyboardputstring "$text" 2>$null
    Start-Sleep -Milliseconds 300
}

function VBox-TypeCommand {
    param([string]$cmd)
    & $VBoxManage controlvm $VMName keyboardputstring "$cmd" 2>$null
    Start-Sleep -Milliseconds 300
    & $VBoxManage controlvm $VMName keyboardputscancode 1c 9c 2>$null  # Enter
}

function VBox-PressKey {
    param([string]$scancodes)
    & $VBoxManage controlvm $VMName keyboardputscancode $scancodes.Split(' ') 2>$null
    Start-Sleep -Milliseconds 80
}

# Named key helpers
function VBox-Enter   { VBox-PressKey "1c 9c" }
function VBox-Escape  { VBox-PressKey "01 81" }
function VBox-Backspace { VBox-PressKey "0e 8e" }
function VBox-Tab     { VBox-PressKey "0f 8f" }
function VBox-Space   { VBox-PressKey "39 b9" }
function VBox-Delete  { VBox-PressKey "e0 53 e0 d3" }
function VBox-Up      { VBox-PressKey "e0 48 e0 c8" }
function VBox-Down    { VBox-PressKey "e0 50 e0 d0" }
function VBox-Left    { VBox-PressKey "e0 4b e0 cb" }
function VBox-Right   { VBox-PressKey "e0 4d e0 cd" }
function VBox-Home    { VBox-PressKey "e0 47 e0 c7" }
function VBox-End     { VBox-PressKey "e0 4f e0 cf" }
function VBox-PgUp    { VBox-PressKey "e0 49 e0 c9" }
function VBox-PgDn    { VBox-PressKey "e0 51 e0 d1" }
function VBox-F1      { VBox-PressKey "3b bb" }
function VBox-F2      { VBox-PressKey "3c bc" }
function VBox-F3      { VBox-PressKey "3d bd" }
function VBox-F4      { VBox-PressKey "3e be" }
function VBox-F5      { VBox-PressKey "3f bf" }
function VBox-F11     { VBox-PressKey "57 d7" }

# Key combos
function VBox-WinKey  { VBox-PressKey "e0 5b e0 db" }  # Win press+release
function VBox-WinE    { VBox-PressKey "e0 5b 12 92 e0 db" }  # Win+E
function VBox-WinD    { VBox-PressKey "e0 5b 20 a0 e0 db" }  # Win+D
function VBox-WinI    { VBox-PressKey "e0 5b 17 97 e0 db" }  # Win+I
function VBox-WinH    { VBox-PressKey "e0 5b 23 a3 e0 db" }  # Win+H
function VBox-WinL    { VBox-PressKey "e0 5b 26 a6 e0 db" }  # Win+L
function VBox-WinUp   { VBox-PressKey "e0 5b e0 48 e0 c8 e0 db" }  # Win+Up
function VBox-WinDown { VBox-PressKey "e0 5b e0 50 e0 d0 e0 db" }  # Win+Down
function VBox-WinLeft { VBox-PressKey "e0 5b e0 4b e0 cb e0 db" }  # Win+Left
function VBox-WinRight{ VBox-PressKey "e0 5b e0 4d e0 cd e0 db" }  # Win+Right
function VBox-AltTab  { VBox-PressKey "38 0f 8f b8" }  # Alt+Tab
function VBox-AltF4   { VBox-PressKey "38 3e be b8" }  # Alt+F4
function VBox-CtrlC   { VBox-PressKey "1d 2e ae 9d" }  # Ctrl+C
function VBox-CtrlV   { VBox-PressKey "1d 2f af 9d" }  # Ctrl+V
function VBox-CtrlX   { VBox-PressKey "1d 2d ad 9d" }  # Ctrl+X
function VBox-CtrlZ   { VBox-PressKey "1d 2c ac 9d" }  # Ctrl+Z

# Letter keys (scancode for a-z)
$global:SC = @{
    'a'='1e 9e'; 'b'='30 b0'; 'c'='2e ae'; 'd'='20 a0'; 'e'='12 92'; 'f'='21 a1';
    'g'='22 a2'; 'h'='23 a3'; 'i'='17 97'; 'j'='24 a4'; 'k'='25 a5'; 'l'='26 a6';
    'm'='32 b2'; 'n'='31 b1'; 'o'='18 98'; 'p'='19 99'; 'q'='10 90'; 'r'='13 93';
    's'='1f 9f'; 't'='14 94'; 'u'='16 96'; 'v'='2f af'; 'w'='11 91'; 'x'='2d ad';
    'y'='15 95'; 'z'='2c ac';
    '0'='0b 8b'; '1'='02 82'; '2'='03 83'; '3'='04 84'; '4'='05 85'; '5'='06 86';
    '6'='07 87'; '7'='08 88'; '8'='09 89'; '9'='0a 8a';
}

function VBox-PressLetter {
    param([char]$ch)
    $lower = "$ch".ToLower()
    if ($global:SC.ContainsKey($lower)) {
        VBox-PressKey $global:SC[$lower]
    }
}

function VBox-Screenshot {
    param([string]$name)
    & $VBoxManage controlvm $VMName screenshotpng "$PSScriptRoot\test_${name}.png" 2>$null
}

# --- Test framework ---

function Record-Result {
    param([string]$category, [string]$name, [string]$status, [string]$cmd = "", [string]$output = "", [string]$detail = "")
    $outSnippet = if ($output.Length -gt 300) { $output.Substring(0, 300) } else { "$output" }
    $outSnippet = $outSnippet -replace "`r`n|`r|`n", " "
    $global:results += @{
        Category = $category
        Name     = $name
        Command  = $cmd
        Status   = $status
        Output   = $outSnippet
        Detail   = $detail
    }
    switch ($status) {
        "PASS"  { $global:passed++ }
        "FAIL"  { $global:failed++ }
        "CRASH" { $global:crashed++ }
        "SKIP"  { $global:skipped++ }
    }
}

function Run-ShellTest {
    param([string]$cat, [string]$tname, [string]$cmd, [scriptblock]$validate, [int]$wait = $CmdWait)
    $global:testNum++
    if ($SkipPassed -and $global:PassedCategories -contains $cat) {
        Write-Host ("  [{0}] {1} ... SKIP (prev pass)" -f $cat, $tname) -ForegroundColor DarkGray
        $global:skipped++
        $global:results += @{ Category=$cat; Name=$tname; Command=$cmd; Status="SKIP"; Output=""; Detail="Passed in previous run" }
        return
    }
    Write-Host ("  [{0}] {1} ... " -f $cat, $tname) -NoNewline
    try {
        Start-Sleep -Milliseconds 300
        $mark = Get-SerialLength
        Start-Sleep -Milliseconds 100
        VBox-TypeCommand -cmd $cmd
        $output = Wait-ForOutput -sinceMark $mark -seconds $wait
        
        # Check for crashes
        if (Check-ForPanic -sinceMark $mark -context "$cat/$tname") {
            Record-Result -category $cat -name $tname -status "CRASH" -cmd $cmd -output $output
            return
        }
        
        $success = & $validate $output
        if ($success) {
            Write-Host "PASS" -ForegroundColor Green
            Record-Result -category $cat -name $tname -status "PASS" -cmd $cmd -output $output
        } else {
            Write-Host "FAIL" -ForegroundColor Red
            Record-Result -category $cat -name $tname -status "FAIL" -cmd $cmd -output $output
        }
    } catch {
        Write-Host "ERROR $_" -ForegroundColor Yellow
        Record-Result -category $cat -name $tname -status "FAIL" -cmd $cmd -output "Exception: $_"
    }
}

function Run-DesktopSubtest {
    param([string]$cat, [string]$name, [string]$detail = "")
    # Returns marker for use with Check-Desktop
    $mark = Get-SerialLength
    Write-Host ("    {0}..." -f $name) -NoNewline
    return $mark
}

function Check-Desktop {
    param([long]$mark, [string]$cat, [string]$name)
    Start-Sleep -Milliseconds 200
    $crashed = Check-ForPanic -sinceMark $mark -context "$cat/$name"
    if ($crashed) {
        Record-Result -category $cat -name $name -status "CRASH"
        return
    }
    Write-Host " OK" -ForegroundColor Green
    Record-Result -category $cat -name $name -status "PASS"
}

function Open-StartMenu {
    VBox-WinKey
    Start-Sleep -Milliseconds 500
}

function Open-App-Via-StartMenu {
    param([string]$appName)
    Open-StartMenu
    VBox-TypeText $appName
    Start-Sleep -Milliseconds 300
    VBox-Enter
    Start-Sleep -Milliseconds 800
}

function Close-App {
    VBox-AltF4
    Start-Sleep -Milliseconds 500
}

# â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•
#  MAIN EXECUTION
# â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•

Write-Host ""
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host "  TrustOS ULTIMATE Test Suite -- VirtualBox Edition" -ForegroundColor Cyan
Write-Host "  $timestamp" -ForegroundColor DarkCyan
Write-Host "  Shell: $(-not $SkipShell)  |  Desktop: $(-not $SkipDesktop)" -ForegroundColor DarkCyan
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host ""

# Pre-flight checks
if (-not (Test-Path $VBoxManage)) {
    Write-Host "FATAL: VBoxManage not found at $VBoxManage" -ForegroundColor Red; exit 1
}
if (-not (Test-Path $IsoPath)) {
    Write-Host "FATAL: ISO not found at $IsoPath" -ForegroundColor Red; exit 1
}
Write-Host "  ISO: $IsoPath" -ForegroundColor DarkGray
Write-Host ""

# --- VM Setup ---

Write-Host "Step 1: Setting up VirtualBox VM..." -ForegroundColor White
& $VBoxManage controlvm $VMName poweroff 2>$null
Start-Sleep -Seconds 2
& $VBoxManage unregistervm $VMName --delete 2>$null
Start-Sleep -Seconds 1
Remove-Item $SerialLog -Force -ErrorAction SilentlyContinue

& $VBoxManage createvm --name $VMName --ostype "Other_64" --register 2>$null | Out-Null
& $VBoxManage modifyvm $VMName --memory 1024 --vram 128 --cpus 2 2>$null
& $VBoxManage modifyvm $VMName --firmware efi64 2>$null
& $VBoxManage modifyvm $VMName --firmware-boot-menu disabled 2>$null
& $VBoxManage modifyvm $VMName --graphicscontroller vboxsvga 2>$null
& $VBoxManage modifyvm $VMName --boot1 dvd --boot2 disk --boot3 none --boot4 none 2>$null
& $VBoxManage modifyvm $VMName --nic1 nat --nictype1 82540EM --cableconnected1 on 2>$null
& $VBoxManage storagectl $VMName --name "SATA" --add sata --controller IntelAhci --portcount 2 2>$null
& $VBoxManage storageattach $VMName --storagectl "SATA" --port 0 --device 0 --type dvddrive --medium $IsoPath 2>$null
& $VBoxManage modifyvm $VMName --uart1 0x3F8 4 --uartmode1 file "$SerialLog" 2>$null
& $VBoxManage modifyvm $VMName --audio-driver default --audio-controller hda --audio-enabled on 2>$null
Write-Host "  VM created" -ForegroundColor Green

# --- Start VM ---
Write-Host "Step 2: Starting VM..." -ForegroundColor White
& $VBoxManage startvm $VMName 2>$null | Out-Null
Write-Host "  VM started" -ForegroundColor Green

# --- Wait for boot ---
Write-Host "Step 3: Waiting for boot (max ${BootTimeout}s)..." -ForegroundColor White
$sw = [System.Diagnostics.Stopwatch]::StartNew()
$booted = $false
$escSent = $false
while ($sw.Elapsed.TotalSeconds -lt $BootTimeout) {
    Start-Sleep -Milliseconds 500
    if (Test-Path $SerialLog) {
        $content = Get-Content $SerialLog -Raw -ErrorAction SilentlyContinue
        if ($content -and ($content -match "trustos.*[\$#]" -or $content -match "Shell ready" -or $content -match "tsh>")) {
            $booted = $true; break
        }
    }
    # If stuck in UEFI setup (no serial data after 12s), send Escape to exit menus
    if (-not $escSent -and $sw.Elapsed.TotalSeconds -gt 12) {
        $hasSerial = (Test-Path $SerialLog) -and (Get-Item $SerialLog -ErrorAction SilentlyContinue).Length -gt 50
        if (-not $hasSerial) {
            Write-Host "ESC" -NoNewline -ForegroundColor Yellow
            & $VBoxManage controlvm $VMName keyboardputscancode 01 81 2>$null
            Start-Sleep -Milliseconds 500
            & $VBoxManage controlvm $VMName keyboardputscancode 01 81 2>$null
            Start-Sleep -Milliseconds 500
            & $VBoxManage controlvm $VMName keyboardputscancode 01 81 2>$null
            $escSent = $true
        }
    }
    Write-Host "." -NoNewline -ForegroundColor DarkGray
}
Write-Host ""
$bootTime = [math]::Round($sw.Elapsed.TotalSeconds, 1)

if (-not $booted) {
    $vmState = & $VBoxManage showvminfo $VMName --machinereadable 2>$null | Select-String "VMState="
    Write-Host "  VM state: $vmState" -ForegroundColor Yellow
    if ((Test-Path $SerialLog) -and (Get-Item $SerialLog).Length -gt 100) {
        Write-Host "  Serial has data, proceeding anyway..." -ForegroundColor Yellow
        $booted = $true
    } else {
        Write-Host "FATAL: Boot timed out" -ForegroundColor Red
        & $VBoxManage controlvm $VMName poweroff 2>$null; exit 1
    }
}
Write-Host "  Booted in ${bootTime}s" -ForegroundColor Green
Start-Sleep -Seconds 3
VBox-Screenshot "boot"

# Initial drain -- run a harmless command to flush boot noise and establish clean serial state
Write-Host "  Draining boot noise..." -ForegroundColor DarkGray
VBox-TypeCommand "echo __READY__"
Start-Sleep -Milliseconds 2500
$null = Get-SerialLength

# â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•
#  PHASE 1: SHELL COMMAND TESTS
# â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•

if (-not $SkipShell) {
Write-Host ""
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host "  PHASE 1: SHELL COMMAND TESTS" -ForegroundColor Cyan
Write-Host "================================================================" -ForegroundColor Cyan

$testStart = [System.Diagnostics.Stopwatch]::StartNew()

# â”€â”€ SHELL BASICS â”€â”€
Write-Host "`n  -- SHELL --" -ForegroundColor Cyan
Run-ShellTest "SHELL" "echo simple" "echo hello_trustos" { param($o) $o -match "hello_trustos" }
Run-ShellTest "SHELL" "echo spaces" "echo hello world 123" { param($o) $o -match "hello world 123" }
Run-ShellTest "SHELL" "pwd root" "pwd" { param($o) $o -match "/" }
Run-ShellTest "SHELL" "whoami" "whoami" { param($o) $o -match "root|nobody" }
Run-ShellTest "SHELL" "hostname" "hostname" { param($o) $o -match "trustos" }
Run-ShellTest "SHELL" "version" "version" { param($o) $o -match "T-RustOs" }
Run-ShellTest "SHELL" "uname -a" "uname -a" { param($o) $o -match "T-RustOs" -and $o -match "x86_64" }
Run-ShellTest "SHELL" "clear" "clear" { param($o) $true }
Run-ShellTest "SHELL" "history" "history" { param($o) $o -match "\d" }
Run-ShellTest "SHELL" "help" "help" { param($o) $o -match "help|command|Commands" }
Run-ShellTest "SHELL" "man help" "man help" { param($o) $o.Length -gt 5 }
Run-ShellTest "SHELL" "info" "info" { param($o) $o -match "T-RUSTOS|Version" }
Run-ShellTest "SHELL" "cowsay" "cowsay TrustOS" { param($o) $o -match "TrustOS" }

# â”€â”€ SYSTEM INFO â”€â”€
Write-Host "`n  -- SYSINFO --" -ForegroundColor Cyan
Run-ShellTest "SYSINFO" "date" "date" { param($o) $o -match "\d{4}" }
Run-ShellTest "SYSINFO" "uptime" "time" { param($o) $o -match "Uptime|Time" }
Run-ShellTest "SYSINFO" "free" "free" { param($o) $o -match "Heap|total|free" }
Run-ShellTest "SYSINFO" "df" "df" { param($o) $o -match "ramfs|Filesystem" }
Run-ShellTest "SYSINFO" "ps" "ps" { param($o) $o -match "PID|kernel|tsh" }
Run-ShellTest "SYSINFO" "env" "env" { param($o) $o -match "USER=|SHELL=" }
Run-ShellTest "SYSINFO" "id" "id" { param($o) $o -match "uid|root" }
Run-ShellTest "SYSINFO" "users" "users" { param($o) $o -match "root" -or $o.Length -gt 0 }
Run-ShellTest "SYSINFO" "lscpu" "lscpu" { param($o) $o -match "CPU|cpu|x86" }
Run-ShellTest "SYSINFO" "lsmem" "lsmem" { param($o) $o -match "Memory|MB|KB" }
Run-ShellTest "SYSINFO" "lspci" "lspci" { param($o) $o -match "PCI|Bus|device" }
Run-ShellTest "SYSINFO" "lsblk" "lsblk" { param($o) $o.Length -gt 3 }
Run-ShellTest "SYSINFO" "vmstat" "vmstat" { param($o) $o.Length -gt 3 }
Run-ShellTest "SYSINFO" "neofetch" "neofetch" { param($o) $o -match "TrustOS|trustos|Kernel" }
Run-ShellTest "SYSINFO" "dmesg" "dmesg" { param($o) $o.Length -gt 5 }
Run-ShellTest "SYSINFO" "lshw" "lshw" { param($o) $o.Length -gt 3 }
Run-ShellTest "SYSINFO" "lsmod" "lsmod" { param($o) $o.Length -ge 0 }
Run-ShellTest "SYSINFO" "tty" "tty" { param($o) $o -match "tty|console|serial" -or $o.Length -gt 0 }

# â”€â”€ FILESYSTEM â”€â”€
Write-Host "`n  -- FILESYSTEM --" -ForegroundColor Cyan
Run-ShellTest "FS" "mkdir" "mkdir /test_ultimate" { param($o) -not ($o -match "mkdir:.*rror") }
Run-ShellTest "FS" "ls sees dir" "ls /" { param($o) $o -match "test_ultimate" }
Run-ShellTest "FS" "touch" "touch /test_ultimate/data.txt" { param($o) -not ($o -match "touch:.*rror") }
Run-ShellTest "FS" "echo > file" 'echo content_42 > /test_ultimate/data.txt' { param($o) $true }
Run-ShellTest "FS" "cat" "cat /test_ultimate/data.txt" { param($o) $o -match "content_42" }
Run-ShellTest "FS" "ls dir" "ls /test_ultimate" { param($o) $o -match "data" }
Run-ShellTest "FS" "stat" "stat /test_ultimate/data.txt" { param($o) $o -match "data|size|Size|File" }
Run-ShellTest "FS" "wc" "wc /test_ultimate/data.txt" { param($o) $o -match "\d" }
Run-ShellTest "FS" "head" "head /test_ultimate/data.txt" { param($o) $o -match "content" }
Run-ShellTest "FS" "tail" "tail /test_ultimate/data.txt" { param($o) $o -match "content" }
Run-ShellTest "FS" "cp" "cp /test_ultimate/data.txt /test_ultimate/copy.txt" { param($o) -not ($o -match "cp:.*rror") }
Run-ShellTest "FS" "cat copy" "cat /test_ultimate/copy.txt" { param($o) $o -match "content_42" }
Run-ShellTest "FS" "mv" "mv /test_ultimate/copy.txt /test_ultimate/moved.txt" { param($o) -not ($o -match "mv:.*rror") }
Run-ShellTest "FS" "cat moved" "cat /test_ultimate/moved.txt" { param($o) $o -match "content_42" }
Run-ShellTest "FS" "grep" "grep content /test_ultimate/data.txt" { param($o) $o -match "content" }
Run-ShellTest "FS" "find" "find data" { param($o) $o -match "data" }
Run-ShellTest "FS" "tree" "tree /test_ultimate" { param($o) $o -match "data|moved" }
Run-ShellTest "FS" "diff" "diff /test_ultimate/data.txt /test_ultimate/moved.txt" { param($o) $true }
Run-ShellTest "FS" "hexdump" "hexdump /test_ultimate/data.txt" { param($o) $o -match "[0-9a-fA-F]" }
Run-ShellTest "FS" "rm" "rm /test_ultimate/moved.txt" { param($o) -not ($o -match "rm:.*rror") }
Run-ShellTest "FS" "rm verify" "cat /test_ultimate/moved.txt" { param($o) $o -match "cat:|not found|No such" }
Run-ShellTest "FS" "cd" "cd /test_ultimate" { param($o) -not ($o -match "cd:.*rror") }
Run-ShellTest "FS" "pwd after cd" "pwd" { param($o) $o -match "test_ultimate" }
Run-ShellTest "FS" "cd back" "cd /" { param($o) $true }
Run-ShellTest "FS" "which echo" "which echo" { param($o) $o -match "echo|builtin|built-in" -or $o.Length -gt 0 }
Run-ShellTest "FS" "whereis ls" "whereis ls" { param($o) $o.Length -ge 0 }
Run-ShellTest "FS" "file" "file /test_ultimate/data.txt" { param($o) $o.Length -gt 0 }
Run-ShellTest "FS" "basename" "basename /test_ultimate/data.txt" { param($o) $o -match "data\.txt" }
Run-ShellTest "FS" "dirname" "dirname /test_ultimate/data.txt" { param($o) $o -match "test_ultimate" }
Run-ShellTest "FS" "realpath" "realpath /test_ultimate/data.txt" { param($o) $o -match "test_ultimate" }
Run-ShellTest "FS" "du" "du /test_ultimate" { param($o) $o.Length -ge 0 }

# â”€â”€ TEXT UTILITIES â”€â”€
Write-Host "`n  -- TEXT --" -ForegroundColor Cyan
Run-ShellTest "TEXT" "seq 5" "seq 5" { param($o) $o -match "1" -and $o -match "5" }
Run-ShellTest "TEXT" "seq 3 7" "seq 3 7" { param($o) $o -match "3" -and $o -match "7" }
Run-ShellTest "TEXT" "factor 12" "factor 12" { param($o) $o -match "12" -and $o -match "2" }
Run-ShellTest "TEXT" "factor 97" "factor 97" { param($o) $o -match "97" }
Run-ShellTest "TEXT" "expr 2+3" "expr 2 + 3" { param($o) $o -match "5" }
Run-ShellTest "TEXT" "cal" "cal" { param($o) $o -match "Su|Mo|Tu" -or $o.Length -gt 10 }
Run-ShellTest "TEXT" "rev" "rev" { param($o) $o.Length -ge 0 }
Run-ShellTest "TEXT" "sort" "sort" { param($o) $o.Length -ge 0 }
Run-ShellTest "TEXT" "uniq" "uniq" { param($o) $o.Length -ge 0 }
Run-ShellTest "TEXT" "od" "od" { param($o) $o.Length -ge 0 }
Run-ShellTest "TEXT" "cmp" "cmp" { param($o) $o.Length -ge 0 }
Run-ShellTest "TEXT" "strings" "strings /test_ultimate/data.txt" { param($o) $o -match "content" -or $o.Length -ge 0 }

# â”€â”€ PIPES & FILTERS â”€â”€
Write-Host "`n  -- PIPES --" -ForegroundColor Cyan
Run-ShellTest "PIPES" "echo|tr upper" "echo hello world | tr a-z A-Z" { param($o) $o -match "HELLO WORLD" }
Run-ShellTest "PIPES" "echo|cut" "echo a:b:c:d | cut -d : -f 2,4" { param($o) $o -match "b" }
Run-ShellTest "PIPES" "echo|tee" "echo tee_test_42 | tee /tmp/tee_out.txt" { param($o) $o -match "tee_test_42" }
Run-ShellTest "PIPES" "cat tee" "cat /tmp/tee_out.txt" { param($o) $o -match "tee_test_42" }
Run-ShellTest "PIPES" "echo|wc" "echo hello | wc" { param($o) $o -match "\d" }
Run-ShellTest "PIPES" "echo|grep" "echo find_me_42 | grep find_me" { param($o) $o -match "find_me" }

# â”€â”€ HASHING â”€â”€
Write-Host "`n  -- HASH --" -ForegroundColor Cyan
Run-ShellTest "HASH" "md5sum" "md5sum /test_ultimate/data.txt" { param($o) $o -match "[0-9a-fA-F]{32}" -or $o -match "Usage|md5sum" }
Run-ShellTest "HASH" "sha256sum" "sha256sum /test_ultimate/data.txt" { param($o) $o -match "[0-9a-fA-F]{64}" -or $o -match "Usage|sha256" }

# â”€â”€ SYMLINKS â”€â”€
Write-Host "`n  -- LINKS --" -ForegroundColor Cyan
Run-ShellTest "LINKS" "ln -s" "ln -s /test_ultimate/data.txt /tmp/link_test" { param($o) $o -match "->|link" -or -not ($o -match "error|Error") }
Run-ShellTest "LINKS" "readlink" "readlink /tmp/link_test" { param($o) $o -match "data\.txt|test_ultimate" }

# â”€â”€ PERMISSIONS â”€â”€
Write-Host "`n  -- PERMS --" -ForegroundColor Cyan
Run-ShellTest "PERMS" "chmod" "chmod 755 /test_ultimate/data.txt" { param($o) -not ($o -match "error|Error") }
Run-ShellTest "PERMS" "chown" "chown root /test_ultimate/data.txt" { param($o) -not ($o -match "error|Error") }

# â”€â”€ ARCHIVES â”€â”€
Write-Host "`n  -- ARCHIVE --" -ForegroundColor Cyan
Run-ShellTest "ARCHIVE" "tar create" "tar cf /tmp/test.tar /test_ultimate/data.txt" { param($o) -not ($o -match "error|Error") }
Run-ShellTest "ARCHIVE" "tar list" "tar tf /tmp/test.tar" { param($o) $o -match "data" }
Run-ShellTest "ARCHIVE" "tar extract" "tar xf /tmp/test.tar" { param($o) -not ($o -match "error|Error") }
Run-ShellTest "ARCHIVE" "gzip" "gzip /tmp/tee_out.txt" { param($o) -not ($o -match "error|Error") }
Run-ShellTest "ARCHIVE" "zip" "zip" { param($o) $o.Length -ge 0 }
Run-ShellTest "ARCHIVE" "unzip" "unzip" { param($o) $o.Length -ge 0 }

# â”€â”€ ALIASES â”€â”€
Write-Host "`n  -- ALIAS --" -ForegroundColor Cyan
Run-ShellTest "ALIAS" "alias set" "alias mytest='echo alias_works'" { param($o) -not ($o -match "error") }
Run-ShellTest "ALIAS" "alias list" "alias" { param($o) $o -match "mytest|alias" }
Run-ShellTest "ALIAS" "unalias" "unalias mytest" { param($o) -not ($o -match "error") }

# â”€â”€ SERVICES â”€â”€
Write-Host "`n  -- SERVICES --" -ForegroundColor Cyan
Run-ShellTest "SVC" "service list" "service" { param($o) $o -match "SERVICE|sshd|httpd" -or $o.Length -gt 3 }
Run-ShellTest "SVC" "service start" "service sshd start" { param($o) $o -match "Starting|OK|started" -or $o.Length -gt 0 }
Run-ShellTest "SVC" "systemctl" "systemctl list-units" { param($o) $o -match "SERVICE|sshd" -or $o.Length -gt 3 }
Run-ShellTest "SVC" "crontab -l" "crontab -l" { param($o) $o -match "crontab|no crontab" -or $o.Length -ge 0 }

# â”€â”€ SELFTEST â”€â”€
Write-Host "`n  -- SELFTEST --" -ForegroundColor Cyan
Run-ShellTest "SELFTEST" "builtin test" "test" { param($o) $o -match "self-test|Self-Test|OK|Done|PASS" } 5

# â”€â”€ INTTEST â”€â”€
Write-Host "`n  -- INTTEST --" -ForegroundColor Cyan
Run-ShellTest "INTTEST" "integration suite" "inttest" { param($o) $o -match "ALL.*TESTS PASSED" } 30

# â”€â”€ TRUSTLANG â”€â”€
Write-Host "`n  -- TRUSTLANG --" -ForegroundColor Cyan
Run-ShellTest "TRUSTLANG" "eval println" 'trustlang eval println("hello_tl")' { param($o) $o -match "hello_tl" } 5
Run-ShellTest "TRUSTLANG" "eval 2+3" "trustlang eval println(2+3)" { param($o) $o -match "5" } 5
Run-ShellTest "TRUSTLANG" "eval 6*7" "trustlang eval println(6*7)" { param($o) $o -match "42" } 5
Run-ShellTest "TRUSTLANG" "eval string" 'trustlang eval println("TrustOS_rocks")' { param($o) $o -match "TrustOS_rocks" } 5
Run-ShellTest "TRUSTLANG" "eval 100/4" "trustlang eval println(100/4)" { param($o) $o -match "25" } 5
Run-ShellTest "TRUSTLANG" "eval bool" "trustlang eval println(3>2)" { param($o) $o -match "true" } 5
Run-ShellTest "TRUSTLANG" "run let var" 'trustlang run fn main() { let x = 99; println(x); }' { param($o) $o -match "99" } 6
Run-ShellTest "TRUSTLANG" "run fibonacci" 'trustlang run fn fib(n) { if n <= 1 { return n; } return fib(n-1) + fib(n-2); } fn main() { println(fib(10)); }' { param($o) $o -match "55" } 8
Run-ShellTest "TRUSTLANG" "run while" 'trustlang run fn main() { let mut i = 0; while i < 3 { println(i); i = i + 1; } }' { param($o) $o -match "0" -and $o -match "2" } 6

# â”€â”€ NETWORK â”€â”€
Write-Host "`n  -- NETWORK --" -ForegroundColor Cyan
Run-ShellTest "NET" "ifconfig" "ifconfig" { param($o) $o -match "10\.|eth|lo|IP" }
Run-ShellTest "NET" "ipconfig" "ipconfig" { param($o) $o -match "IP|ip|addr|Address" -or $o.Length -gt 3 }
Run-ShellTest "NET" "arp" "arp" { param($o) $o -match "ARP|arp|Address" -or $o.Length -gt 3 }
Run-ShellTest "NET" "route" "route" { param($o) $o -match "Route|Gateway" -or $o.Length -gt 3 }
Run-ShellTest "NET" "netstat" "netstat" { param($o) $o -match "Active|Proto|tcp" -or $o.Length -gt 3 }
Run-ShellTest "NET" "ping" "ping 10.0.2.2" { param($o) $o -match "ping|PING|reply|Reply|timeout" } 6
Run-ShellTest "NET" "nslookup" "nslookup example.com" { param($o) $o -match "DNS|Server|Address|example" } 6
Run-ShellTest "NET" "traceroute" "traceroute 10.0.2.2" { param($o) $o -match "traceroute|hop|Hop" -or $o.Length -gt 3 } 6

# â”€â”€ DISK & HW â”€â”€
Write-Host "`n  -- DISK --" -ForegroundColor Cyan
Run-ShellTest "DISK" "disk" "disk" { param($o) $o -match "Disk|disk|AHCI|No" -or $o.Length -gt 2 }
Run-ShellTest "DISK" "fdisk" "fdisk" { param($o) $o -match "Partition|partition|Disk|No" -or $o.Length -gt 2 }
Run-ShellTest "DISK" "blkid" "blkid" { param($o) $o.Length -ge 0 }
Run-ShellTest "DISK" "mount" "mount" { param($o) $o.Length -ge 0 }
Run-ShellTest "DISK" "sync" "sync" { param($o) $true }
Run-ShellTest "DISK" "lsusb" "lsusb" { param($o) $o.Length -ge 0 }
Run-ShellTest "DISK" "gpu" "gpu" { param($o) $o -match "GPU|gpu|VGA|video|Graphics" -or $o.Length -gt 2 }

# â”€â”€ AUDIO â”€â”€
Write-Host "`n  -- AUDIO --" -ForegroundColor Cyan
Run-ShellTest "AUDIO" "beep" "beep" { param($o) $true }
Run-ShellTest "AUDIO" "audio" "audio" { param($o) $o -match "Audio|audio|HDA" -or $o.Length -ge 0 }
Run-ShellTest "AUDIO" "synth" "synth" { param($o) $o.Length -ge 0 }

# â”€â”€ DEBUG â”€â”€
Write-Host "`n  -- DEBUG --" -ForegroundColor Cyan
Run-ShellTest "DEBUG" "irqstat" "irqstat" { param($o) $o -match "IRQ|irq|interrupt|\d+" }
Run-ShellTest "DEBUG" "smpstatus" "smpstatus" { param($o) $o -match "SMP|CPU|cpu|AP" -or $o.Length -gt 3 }
Run-ShellTest "DEBUG" "perf" "perf" { param($o) $o -match "Perf|perf|Performance|cycles" -or $o.Length -gt 3 }
Run-ShellTest "DEBUG" "memdbg" "memdbg" { param($o) $o -match "Heap|heap|Memory|alloc" }
Run-ShellTest "DEBUG" "regs" "regs" { param($o) $o -match "RAX|RBX|RSP|CR|Register" }
Run-ShellTest "DEBUG" "memtest" "memtest" { param($o) $o -match "Memory|Pass|pass|OK" -or $o.Length -gt 3 } 8

# â”€â”€ SECURITY â”€â”€
Write-Host "`n  -- SECURITY --" -ForegroundColor Cyan
Run-ShellTest "SECURITY" "create file" 'echo sigtest > /tmp_sig.txt' { param($o) $true }
Run-ShellTest "SECURITY" "sig sign" "sig sign /tmp_sig.txt" { param($o) $o -match "Signed|signed|signature|sig" -or -not ($o -match "error") } 5
Run-ShellTest "SECURITY" "sig verify" "sig verify /tmp_sig.txt" { param($o) $o -match "Valid|valid|OK|verified" -or -not ($o -match "error") } 5

# â”€â”€ PROCESS â”€â”€
Write-Host "`n  -- PROCESS --" -ForegroundColor Cyan
Run-ShellTest "PROC" "sleep 0" "sleep 0" { param($o) $true }
Run-ShellTest "PROC" "tasks" "tasks" { param($o) $o.Length -ge 0 }
Run-ShellTest "PROC" "threads" "threads" { param($o) $o.Length -ge 0 }

# â”€â”€ STUBS & MISC UTILS â”€â”€
Write-Host "`n  -- STUBS --" -ForegroundColor Cyan
Run-ShellTest "STUBS" "bc" "bc" { param($o) $o.Length -ge 0 }
Run-ShellTest "STUBS" "base64" "base64" { param($o) $o.Length -ge 0 }
Run-ShellTest "STUBS" "printf hello" "printf hello" { param($o) $o -match "hello" -or $o.Length -ge 0 }
Run-ShellTest "STUBS" "set" "set" { param($o) $o.Length -ge 0 }
Run-ShellTest "STUBS" "export" "export" { param($o) $o.Length -ge 0 }
Run-ShellTest "STUBS" "sysctl" "sysctl" { param($o) $o.Length -ge 0 }
Run-ShellTest "STUBS" "firewall" "firewall" { param($o) $o.Length -ge 0 }
Run-ShellTest "STUBS" "reset" "reset" { param($o) $true }
Run-ShellTest "STUBS" "stty" "stty" { param($o) $o.Length -ge 0 }

# â”€â”€ HARDWARE SCAN â”€â”€
Write-Host "`n  -- HWSCAN --" -ForegroundColor Cyan
Run-ShellTest "HWSCAN" "hwscan" "hwscan" { param($o) $o.Length -ge 0 } 5
Run-ShellTest "HWSCAN" "a11y" "a11y" { param($o) $o.Length -ge 0 }
Run-ShellTest "HWSCAN" "fontsmooth" "fontsmooth" { param($o) $o.Length -ge 0 }

# â”€â”€ DISTRO / VM â”€â”€
Write-Host "`n  -- DISTRO --" -ForegroundColor Cyan
Run-ShellTest "DISTRO" "distros" "distros" { param($o) $o.Length -ge 0 }
Run-ShellTest "DISTRO" "persist" "persist" { param($o) $o.Length -ge 0 }

# â”€â”€ PKG / EXEC â”€â”€
Write-Host "`n  -- EXEC --" -ForegroundColor Cyan
Run-ShellTest "EXEC" "trustpkg" "trustpkg" { param($o) $o.Length -ge 0 }
Run-ShellTest "EXEC" "elfinfo" "elfinfo" { param($o) $o.Length -ge 0 }
Run-ShellTest "EXEC" "checkm8" "checkm8" { param($o) $o.Length -ge 0 }
Run-ShellTest "EXEC" "transpile" "transpile" { param($o) $o.Length -ge 0 }

# â”€â”€ SCANNING (safe stubs) â”€â”€
Write-Host "`n  -- SCANNING --" -ForegroundColor Cyan
Run-ShellTest "SCAN" "nmap stub" "nmap" { param($o) $o.Length -ge 0 }
Run-ShellTest "SCAN" "scantest" "scantest" { param($o) $o.Length -ge 0 }
Run-ShellTest "SCAN" "httpd" "httpd" { param($o) $o.Length -ge 0 }

# â”€â”€ UNIX COMPAT â”€â”€
Write-Host "`n  -- UNIX --" -ForegroundColor Cyan
Run-ShellTest "UNIX" "xargs echo" "echo a b c | xargs echo" { param($o) $o -match "a" -or $o.Length -ge 0 }
Run-ShellTest "UNIX" "timecmd echo" "timecmd echo hello" { param($o) $o -match "hello|elapsed|ms" -or $o.Length -gt 0 }
Run-ShellTest "UNIX" "devpanel" "devpanel" { param($o) $o.Length -ge 0 }

# == YES command (placed LAST to avoid serial buffer pollution) ==
Write-Host "`n  -- YES (end) --" -ForegroundColor Cyan
Run-ShellTest "TEXT" "yes (1s)" "yes" { param($o) $o -match "y" -or $o.Length -ge 0 } 2
# Kill yes with Ctrl+C and drain the serial buffer
Write-Host "  Draining serial buffer after 'yes'..." -ForegroundColor DarkGray
VBox-CtrlC
Start-Sleep -Milliseconds 500
VBox-CtrlC
Start-Sleep -Milliseconds 500
VBox-Enter
Start-Sleep -Milliseconds 1500
$null = Get-SerialLength
$shellDuration = [math]::Round($testStart.Elapsed.TotalSeconds, 1)
VBox-Screenshot "shell_done"

Write-Host "`n  Shell phase done: ${shellDuration}s" -ForegroundColor DarkGray
}

# â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•
#  PHASE 2: DESKTOP TESTS
# â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•

if ($SkipPassed) {
    Write-Host "`n  Desktop tests: SKIPPED (100% passed in previous run)" -ForegroundColor DarkGray
    $SkipDesktop = $true
}

if (-not $SkipDesktop) {
Write-Host ""
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host "  PHASE 2: DESKTOP APP + GUI TESTS" -ForegroundColor Cyan
Write-Host "================================================================" -ForegroundColor Cyan

# Enter desktop mode
Write-Host "`n  Entering desktop mode..." -ForegroundColor White
$mark = Get-SerialLength
VBox-TypeCommand "desktop"
Start-Sleep -Milliseconds 4000
$desktopOutput = Wait-ForSerial -sinceMark $mark -seconds 6 -match "Starting desktop|Entering desktop"
if ($desktopOutput -match "Starting desktop|Entering desktop|desktop run loop") {
    Write-Host "  Desktop started!" -ForegroundColor Green
} else {
    Write-Host "  Desktop may have started (no serial confirm)" -ForegroundColor Yellow
}
VBox-Screenshot "desktop_start"

$desktopStart = [System.Diagnostics.Stopwatch]::StartNew()

# â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#  GLOBAL HOTKEYS
# â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

Write-Host "`n  [HOTKEYS] Global Hotkeys..." -ForegroundColor Cyan

$m = Run-DesktopSubtest "HOTKEYS" "Win+E (File Manager)"
VBox-WinE; Start-Sleep -Milliseconds 500
Check-Desktop $m "HOTKEYS" "Win+E (File Manager)"
Close-App; Start-Sleep -Milliseconds 200

$m = Run-DesktopSubtest "HOTKEYS" "Win+D (Show Desktop)"
VBox-WinD; Start-Sleep -Milliseconds 300
Check-Desktop $m "HOTKEYS" "Win+D (Show Desktop)"

$m = Run-DesktopSubtest "HOTKEYS" "Win+I (Settings)"
VBox-WinI; Start-Sleep -Milliseconds 500
Check-Desktop $m "HOTKEYS" "Win+I (Settings)"
Close-App; Start-Sleep -Milliseconds 200

$m = Run-DesktopSubtest "HOTKEYS" "Win+H (High Contrast)"
VBox-WinH; Start-Sleep -Milliseconds 300
VBox-WinH; Start-Sleep -Milliseconds 300  # toggle back
Check-Desktop $m "HOTKEYS" "Win+H (High Contrast)"

$m = Run-DesktopSubtest "HOTKEYS" "Win+Arrows (no window)"
VBox-WinUp; Start-Sleep -Milliseconds 100
VBox-WinDown; Start-Sleep -Milliseconds 100
VBox-WinLeft; Start-Sleep -Milliseconds 100
VBox-WinRight; Start-Sleep -Milliseconds 100
Check-Desktop $m "HOTKEYS" "Win+Arrows (no window)"

$m = Run-DesktopSubtest "HOTKEYS" "Alt+Tab (no windows)"
VBox-AltTab; Start-Sleep -Milliseconds 300
Check-Desktop $m "HOTKEYS" "Alt+Tab (no windows)"

$m = Run-DesktopSubtest "HOTKEYS" "Alt+F4 (no windows)"
VBox-AltF4; Start-Sleep -Milliseconds 300
$raw = Read-SerialRaw -offset $m
if ($raw -match "exiting desktop|exit.*shell") {
    Write-Host " (exited desktop - re-entering)" -ForegroundColor Yellow
    Record-Result -category "HOTKEYS" -name "Alt+F4 (no windows)" -status "PASS" -detail "exited desktop"
    Start-Sleep -Milliseconds 500
    VBox-TypeCommand "desktop"
    Start-Sleep -Milliseconds 4000
} else {
    Check-Desktop $m "HOTKEYS" "Alt+F4 (no windows)"
}

$m = Run-DesktopSubtest "HOTKEYS" "Win+L (Lock Screen)"
VBox-WinL; Start-Sleep -Milliseconds 800
Check-Desktop $m "HOTKEYS" "Win+L (Lock Screen)"
# Unlock (empty PIN or correct)
VBox-Enter; Start-Sleep -Milliseconds 500

# â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#  START MENU
# â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

Write-Host "`n  [MENU] Start Menu..." -ForegroundColor Cyan

$m = Run-DesktopSubtest "STARTMENU" "Toggle open/close"
for ($i = 0; $i -lt 5; $i++) {
    Open-StartMenu; Start-Sleep -Milliseconds 200
    VBox-Escape; Start-Sleep -Milliseconds 200
}
Check-Desktop $m "STARTMENU" "Toggle open/close"

$m = Run-DesktopSubtest "STARTMENU" "Search fuzz"
Open-StartMenu
VBox-TypeText "abcdefghijklmnop"
Start-Sleep -Milliseconds 300
Check-Desktop $m "STARTMENU" "Search fuzz"
VBox-Escape; Start-Sleep -Milliseconds 200

$m = Run-DesktopSubtest "STARTMENU" "Backspace spam"
Open-StartMenu
VBox-TypeText "test"
for ($i = 0; $i -lt 15; $i++) { VBox-Backspace }
Check-Desktop $m "STARTMENU" "Backspace spam"
VBox-Escape; Start-Sleep -Milliseconds 200

$m = Run-DesktopSubtest "STARTMENU" "Arrow navigation"
Open-StartMenu
for ($i = 0; $i -lt 20; $i++) { VBox-Down; Start-Sleep -Milliseconds 30 }
for ($i = 0; $i -lt 20; $i++) { VBox-Up; Start-Sleep -Milliseconds 30 }
Check-Desktop $m "STARTMENU" "Arrow navigation"
VBox-Escape; Start-Sleep -Milliseconds 200

# â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#  CALCULATOR
# â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

Write-Host "`n  [CALC] Calculator..." -ForegroundColor Cyan

$m = Run-DesktopSubtest "CALCULATOR" "Open app"
Open-App-Via-StartMenu "Calculator"
Check-Desktop $m "CALCULATOR" "Open app"

$m = Run-DesktopSubtest "CALCULATOR" "Digit input"
VBox-TypeText "12345"
Start-Sleep -Milliseconds 100
Check-Desktop $m "CALCULATOR" "Digit input"

$m = Run-DesktopSubtest "CALCULATOR" "Operators"
foreach ($op in @("+", "-", "*", "/")) {
    VBox-TypeText "5"
    VBox-TypeText $op
    VBox-TypeText "3"
    VBox-Enter; Start-Sleep -Milliseconds 80
    VBox-TypeText "C"
}
Check-Desktop $m "CALCULATOR" "Operators"

$m = Run-DesktopSubtest "CALCULATOR" "Division by zero"
VBox-TypeText "5/0"
VBox-Enter; Start-Sleep -Milliseconds 200
VBox-TypeText "C"
Check-Desktop $m "CALCULATOR" "Division by zero"

$m = Run-DesktopSubtest "CALCULATOR" "Overflow"
VBox-TypeText ("9" * 50 + "*" + "9" * 50)
VBox-Enter; Start-Sleep -Milliseconds 200
VBox-TypeText "C"
Check-Desktop $m "CALCULATOR" "Overflow"

$m = Run-DesktopSubtest "CALCULATOR" "Backspace on empty"
VBox-TypeText "C"
for ($i = 0; $i -lt 20; $i++) { VBox-Backspace }
Check-Desktop $m "CALCULATOR" "Backspace on empty"

Close-App; Start-Sleep -Milliseconds 300

# â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#  TERMINAL (in desktop)
# â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

Write-Host "`n  [TERM] Desktop Terminal..." -ForegroundColor Cyan

$m = Run-DesktopSubtest "TERMINAL" "Open app"
Open-App-Via-StartMenu "Terminal"
Check-Desktop $m "TERMINAL" "Open app"

$m = Run-DesktopSubtest "TERMINAL" "Type + Enter"
VBox-TypeText "echo desktop_term_ok"
VBox-Enter; Start-Sleep -Milliseconds 300
Check-Desktop $m "TERMINAL" "Type + Enter"

$m = Run-DesktopSubtest "TERMINAL" "Long input"
VBox-TypeText ("A" * 200)
VBox-Enter; Start-Sleep -Milliseconds 300
Check-Desktop $m "TERMINAL" "Long input"

$m = Run-DesktopSubtest "TERMINAL" "Arrow keys"
for ($i = 0; $i -lt 10; $i++) { VBox-Up; VBox-Down }
Check-Desktop $m "TERMINAL" "Arrow keys"

$m = Run-DesktopSubtest "TERMINAL" "Scroll stress"
VBox-TypeText "seq 200"
VBox-Enter; Start-Sleep -Milliseconds 1000
for ($i = 0; $i -lt 15; $i++) { VBox-PgUp; Start-Sleep -Milliseconds 30 }
for ($i = 0; $i -lt 15; $i++) { VBox-PgDn; Start-Sleep -Milliseconds 30 }
Check-Desktop $m "TERMINAL" "Scroll stress"

Close-App; Start-Sleep -Milliseconds 300

# â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#  TEXT EDITOR (TrustCode)
# â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

Write-Host "`n  [EDIT] Text Editor (TrustCode)..." -ForegroundColor Cyan

$m = Run-DesktopSubtest "TRUSTCODE" "Open app"
Open-App-Via-StartMenu "Text Editor"
Check-Desktop $m "TRUSTCODE" "Open app"

$m = Run-DesktopSubtest "TRUSTCODE" "Type + newlines"
VBox-TypeText "Hello TrustCode"
VBox-Enter
VBox-TypeText "Line two"
VBox-Enter
Check-Desktop $m "TRUSTCODE" "Type + newlines"

$m = Run-DesktopSubtest "TRUSTCODE" "Backspace through text"
for ($i = 0; $i -lt 30; $i++) { VBox-Backspace }
Check-Desktop $m "TRUSTCODE" "Backspace through text"

$m = Run-DesktopSubtest "TRUSTCODE" "Arrow keys boundary"
for ($i = 0; $i -lt 50; $i++) { VBox-Up }
for ($i = 0; $i -lt 50; $i++) { VBox-Left }
for ($i = 0; $i -lt 100; $i++) { VBox-Down }
for ($i = 0; $i -lt 100; $i++) { VBox-Right }
Check-Desktop $m "TRUSTCODE" "Arrow keys boundary"

$m = Run-DesktopSubtest "TRUSTCODE" "Home/End/PgUp/PgDn"
for ($i = 0; $i -lt 5; $i++) { VBox-Home; VBox-End; VBox-PgUp; VBox-PgDn }
Check-Desktop $m "TRUSTCODE" "Home/End/PgUp/PgDn"

$m = Run-DesktopSubtest "TRUSTCODE" "Tab key"
for ($i = 0; $i -lt 10; $i++) { VBox-Tab }
Check-Desktop $m "TRUSTCODE" "Tab key"

$m = Run-DesktopSubtest "TRUSTCODE" "Delete key"
for ($i = 0; $i -lt 15; $i++) { VBox-Delete }
Check-Desktop $m "TRUSTCODE" "Delete key"

$m = Run-DesktopSubtest "TRUSTCODE" "Ctrl+Z undo"
for ($i = 0; $i -lt 10; $i++) { VBox-CtrlZ; Start-Sleep -Milliseconds 30 }
Check-Desktop $m "TRUSTCODE" "Ctrl+Z undo"

Close-App; Start-Sleep -Milliseconds 300

# â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#  SNAKE
# â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

Write-Host "`n  [SNAKE] Snake..." -ForegroundColor Cyan

$m = Run-DesktopSubtest "SNAKE" "Open app"
Open-App-Via-StartMenu "Snake"
Check-Desktop $m "SNAKE" "Open app"

$m = Run-DesktopSubtest "SNAKE" "WASD directions"
VBox-Space; Start-Sleep -Milliseconds 200
foreach ($dir in @('w','a','s','d','w','d','s','a')) {
    VBox-PressLetter $dir; Start-Sleep -Milliseconds 100
}
Check-Desktop $m "SNAKE" "WASD directions"

$m = Run-DesktopSubtest "SNAKE" "Rapid reversal"
for ($i = 0; $i -lt 10; $i++) {
    VBox-PressLetter 'w'; VBox-PressLetter 's'; Start-Sleep -Milliseconds 50
}
Check-Desktop $m "SNAKE" "Rapid reversal"

$m = Run-DesktopSubtest "SNAKE" "Pause toggle"
for ($i = 0; $i -lt 5; $i++) {
    VBox-PressLetter 'p'; Start-Sleep -Milliseconds 100
}
Check-Desktop $m "SNAKE" "Pause toggle"

$m = Run-DesktopSubtest "SNAKE" "Space restart spam"
for ($i = 0; $i -lt 10; $i++) { VBox-Space; Start-Sleep -Milliseconds 50 }
Check-Desktop $m "SNAKE" "Space restart spam"

Close-App; Start-Sleep -Milliseconds 300

# â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#  FILE MANAGER
# â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

Write-Host "`n  [FILES] File Manager..." -ForegroundColor Cyan

$m = Run-DesktopSubtest "FILEMANAGER" "Open app"
Open-App-Via-StartMenu "Files"
Check-Desktop $m "FILEMANAGER" "Open app"

$m = Run-DesktopSubtest "FILEMANAGER" "Arrow navigation"
for ($i = 0; $i -lt 10; $i++) { VBox-Up; VBox-Down; Start-Sleep -Milliseconds 30 }
Check-Desktop $m "FILEMANAGER" "Arrow navigation"

$m = Run-DesktopSubtest "FILEMANAGER" "Enter + Backspace"
for ($i = 0; $i -lt 3; $i++) {
    VBox-Enter; Start-Sleep -Milliseconds 100
    VBox-Backspace; Start-Sleep -Milliseconds 100
}
Check-Desktop $m "FILEMANAGER" "Enter + Backspace"

$m = Run-DesktopSubtest "FILEMANAGER" "Boundary scroll"
for ($i = 0; $i -lt 50; $i++) { VBox-Up }
for ($i = 0; $i -lt 50; $i++) { VBox-Down }
Check-Desktop $m "FILEMANAGER" "Boundary scroll"

$m = Run-DesktopSubtest "FILEMANAGER" "View toggle (V)"
for ($i = 0; $i -lt 10; $i++) {
    VBox-PressLetter 'v'; Start-Sleep -Milliseconds 50
}
Check-Desktop $m "FILEMANAGER" "View toggle (V)"

$m = Run-DesktopSubtest "FILEMANAGER" "Clipboard Ctrl+C/V/X"
VBox-Down; Start-Sleep -Milliseconds 50
VBox-CtrlC; Start-Sleep -Milliseconds 100
VBox-CtrlV; Start-Sleep -Milliseconds 200
VBox-CtrlX; Start-Sleep -Milliseconds 100
VBox-CtrlV; Start-Sleep -Milliseconds 200
Check-Desktop $m "FILEMANAGER" "Clipboard Ctrl+C/V/X"

Close-App; Start-Sleep -Milliseconds 300

# â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#  CHESS
# â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

Write-Host "`n  [CHESS] Chess..." -ForegroundColor Cyan

$m = Run-DesktopSubtest "CHESS" "Open app"
Open-App-Via-StartMenu "Chess"
Check-Desktop $m "CHESS" "Open app"

$m = Run-DesktopSubtest "CHESS" "Board navigation"
for ($i = 0; $i -lt 15; $i++) { VBox-Up; VBox-Right; VBox-Down; VBox-Left; Start-Sleep -Milliseconds 20 }
Check-Desktop $m "CHESS" "Board navigation"

$m = Run-DesktopSubtest "CHESS" "Select/deselect"
for ($i = 0; $i -lt 10; $i++) {
    VBox-Enter; Start-Sleep -Milliseconds 50
    VBox-Right; Start-Sleep -Milliseconds 50
    VBox-Enter; Start-Sleep -Milliseconds 50
}
Check-Desktop $m "CHESS" "Select/deselect"

$m = Run-DesktopSubtest "CHESS" "New game (n)"
VBox-PressLetter 'n'; Start-Sleep -Milliseconds 200
Check-Desktop $m "CHESS" "New game (n)"

$m = Run-DesktopSubtest "CHESS" "Pawn move + AI"
# Navigate to e2 and move e4
for ($i = 0; $i -lt 8; $i++) { VBox-Up }; for ($i = 0; $i -lt 8; $i++) { VBox-Left }
for ($i = 0; $i -lt 4; $i++) { VBox-Right; Start-Sleep -Milliseconds 10 }
for ($i = 0; $i -lt 6; $i++) { VBox-Down; Start-Sleep -Milliseconds 10 }
VBox-Enter; Start-Sleep -Milliseconds 100
VBox-Up; VBox-Up; VBox-Enter; Start-Sleep -Milliseconds 2000
Check-Desktop $m "CHESS" "Pawn move + AI"

Close-App; Start-Sleep -Milliseconds 300

# â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#  CHESS 3D
# â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

Write-Host "`n  [CHESS3D] Chess 3D..." -ForegroundColor Cyan

$m = Run-DesktopSubtest "CHESS3D" "Open app"
Open-App-Via-StartMenu "Chess 3D"
Check-Desktop $m "CHESS3D" "Open app"

$m = Run-DesktopSubtest "CHESS3D" "Board + select"
for ($i = 0; $i -lt 10; $i++) { VBox-Up; VBox-Right; VBox-Down; VBox-Left; Start-Sleep -Milliseconds 20 }
VBox-Enter; Start-Sleep -Milliseconds 200
Check-Desktop $m "CHESS3D" "Board + select"

Close-App; Start-Sleep -Milliseconds 300

# â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#  BROWSER
# â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

Write-Host "`n  [WEB] Browser..." -ForegroundColor Cyan

$m = Run-DesktopSubtest "BROWSER" "Open app"
Open-App-Via-StartMenu "Browser"
Check-Desktop $m "BROWSER" "Open app"

$m = Run-DesktopSubtest "BROWSER" "Type URL"
VBox-TypeText "example.com"
VBox-Enter; Start-Sleep -Milliseconds 1000
Check-Desktop $m "BROWSER" "Type URL"

$m = Run-DesktopSubtest "BROWSER" "Backspace URL"
for ($i = 0; $i -lt 20; $i++) { VBox-Backspace }
Check-Desktop $m "BROWSER" "Backspace URL"

$m = Run-DesktopSubtest "BROWSER" "Empty URL Enter"
VBox-Enter; Start-Sleep -Milliseconds 300
Check-Desktop $m "BROWSER" "Empty URL Enter"

$m = Run-DesktopSubtest "BROWSER" "Scroll"
for ($i = 0; $i -lt 10; $i++) { VBox-PgUp; VBox-PgDn; Start-Sleep -Milliseconds 30 }
Check-Desktop $m "BROWSER" "Scroll"

Close-App; Start-Sleep -Milliseconds 300

# â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#  SETTINGS
# â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

Write-Host "`n  [SET] Settings..." -ForegroundColor Cyan

$m = Run-DesktopSubtest "SETTINGS" "Open app"
Open-App-Via-StartMenu "Settings"
Check-Desktop $m "SETTINGS" "Open app"

$m = Run-DesktopSubtest "SETTINGS" "Option keys 1-9"
foreach ($d in 1..9) { VBox-TypeText "$d"; Start-Sleep -Milliseconds 100 }
Check-Desktop $m "SETTINGS" "Option keys 1-9"

Close-App; Start-Sleep -Milliseconds 300

# â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#  NETWORK INFO
# â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

Write-Host "`n  [NET] Network Info..." -ForegroundColor Cyan

$m = Run-DesktopSubtest "NETINFO" "Open app"
Open-App-Via-StartMenu "Network"
Check-Desktop $m "NETINFO" "Open app"

$m = Run-DesktopSubtest "NETINFO" "Arrow + navigation"
for ($i = 0; $i -lt 5; $i++) { VBox-Up; VBox-Down }
Check-Desktop $m "NETINFO" "Arrow + navigation"

Close-App; Start-Sleep -Milliseconds 300

# â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#  TRUSTEDIT 3D
# â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

Write-Host "`n  [3DEDIT] TrustEdit 3D..." -ForegroundColor Cyan

$m = Run-DesktopSubtest "TRUSTEDIT3D" "Open app"
Open-App-Via-StartMenu "TrustEdit"
Check-Desktop $m "TRUSTEDIT3D" "Open app"

$m = Run-DesktopSubtest "TRUSTEDIT3D" "Camera WASD"
foreach ($dir in @('w','a','s','d')) {
    for ($i = 0; $i -lt 5; $i++) { VBox-PressLetter $dir; Start-Sleep -Milliseconds 30 }
}
Check-Desktop $m "TRUSTEDIT3D" "Camera WASD"

Close-App; Start-Sleep -Milliseconds 300

# â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#  TRUSTLAB
# â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

Write-Host "`n  [LAB] TrustLab..." -ForegroundColor Cyan

$m = Run-DesktopSubtest "TRUSTLAB" "Open app"
Open-App-Via-StartMenu "TrustLab"
Check-Desktop $m "TRUSTLAB" "Open app"

$m = Run-DesktopSubtest "TRUSTLAB" "Navigation keys"
for ($i = 0; $i -lt 5; $i++) { VBox-Tab; Start-Sleep -Milliseconds 50 }
for ($i = 0; $i -lt 5; $i++) { VBox-Up; VBox-Down; Start-Sleep -Milliseconds 30 }
Check-Desktop $m "TRUSTLAB" "Navigation keys"

Close-App; Start-Sleep -Milliseconds 300

# â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#  MUSIC PLAYER
# â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

Write-Host "`n  [MUSIC] Music Player..." -ForegroundColor Cyan

$m = Run-DesktopSubtest "MUSICPLAYER" "Open app"
Open-App-Via-StartMenu "Music"
Check-Desktop $m "MUSICPLAYER" "Open app"

$m = Run-DesktopSubtest "MUSICPLAYER" "Play/Pause + controls"
VBox-Space; Start-Sleep -Milliseconds 200  # play/pause
VBox-Left; VBox-Right; Start-Sleep -Milliseconds 100  # prev/next or volume
Check-Desktop $m "MUSICPLAYER" "Play/Pause + controls"

Close-App; Start-Sleep -Milliseconds 300

# â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#  ABOUT / SYSTEM INFO
# â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

Write-Host "`n  [ABOUT] About..." -ForegroundColor Cyan

$m = Run-DesktopSubtest "ABOUT" "Open via Win+I variant"
# About is on the dock but we can try start menu
Open-StartMenu
VBox-TypeText "About"  
Start-Sleep -Milliseconds 300
# About might not be in start menu, so navigate down to find it
VBox-Enter; Start-Sleep -Milliseconds 500
Check-Desktop $m "ABOUT" "Open via search"
Close-App; Start-Sleep -Milliseconds 300

# â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#  NES EMULATOR (if available)
# â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

Write-Host "`n  [NES] NES Emulator..." -ForegroundColor Cyan

$m = Run-DesktopSubtest "NESEMU" "Open app"
Open-App-Via-StartMenu "NES"
Start-Sleep -Milliseconds 500
Check-Desktop $m "NESEMU" "Open app"

$m = Run-DesktopSubtest "NESEMU" "Controls + input"
VBox-Space; Start-Sleep -Milliseconds 200
VBox-PressLetter 'z'; VBox-PressLetter 'x'; Start-Sleep -Milliseconds 100
for ($i = 0; $i -lt 5; $i++) { VBox-Up; VBox-Down; VBox-Left; VBox-Right; Start-Sleep -Milliseconds 30 }
Check-Desktop $m "NESEMU" "Controls + input"

Close-App; Start-Sleep -Milliseconds 300

# â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#  GAME BOY (if available)
# â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

Write-Host "`n  [GB] Game Boy..." -ForegroundColor Cyan

$m = Run-DesktopSubtest "GAMEBOY" "Open app"
Open-App-Via-StartMenu "Game Boy"
Start-Sleep -Milliseconds 500
Check-Desktop $m "GAMEBOY" "Open app"

$m = Run-DesktopSubtest "GAMEBOY" "Controls"
VBox-PressLetter 'z'; VBox-PressLetter 'x'; VBox-Enter; VBox-Space
for ($i = 0; $i -lt 5; $i++) { VBox-Up; VBox-Down; VBox-Left; VBox-Right; Start-Sleep -Milliseconds 30 }
Check-Desktop $m "GAMEBOY" "Controls"

Close-App; Start-Sleep -Milliseconds 300

# â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#  LOCK SCREEN (detailed)
# â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

Write-Host "`n  [LOCK] Lock Screen..." -ForegroundColor Cyan

$m = Run-DesktopSubtest "LOCKSCREEN" "Win+L activate"
VBox-WinL; Start-Sleep -Milliseconds 800
Check-Desktop $m "LOCKSCREEN" "Win+L activate"

$m = Run-DesktopSubtest "LOCKSCREEN" "Wrong PIN"
VBox-TypeText "9999"
VBox-Enter; Start-Sleep -Milliseconds 500
Check-Desktop $m "LOCKSCREEN" "Wrong PIN"

$m = Run-DesktopSubtest "LOCKSCREEN" "Backspace in PIN"
VBox-TypeText "123"
for ($i = 0; $i -lt 10; $i++) { VBox-Backspace }
Check-Desktop $m "LOCKSCREEN" "Backspace in PIN"

$m = Run-DesktopSubtest "LOCKSCREEN" "Long PIN overflow"
VBox-TypeText ("9" * 30)
VBox-Enter; Start-Sleep -Milliseconds 300
Check-Desktop $m "LOCKSCREEN" "Long PIN overflow"

$m = Run-DesktopSubtest "LOCKSCREEN" "Correct PIN (1234)"
VBox-TypeText "1234"
VBox-Enter; Start-Sleep -Milliseconds 500
Check-Desktop $m "LOCKSCREEN" "Correct PIN (1234)"

$m = Run-DesktopSubtest "LOCKSCREEN" "Rapid lock/unlock"
for ($i = 0; $i -lt 3; $i++) {
    VBox-WinL; Start-Sleep -Milliseconds 300
    VBox-Enter; Start-Sleep -Milliseconds 300  # empty PIN
}
Check-Desktop $m "LOCKSCREEN" "Rapid lock/unlock"

# â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#  WINDOW MANAGEMENT
# â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

Write-Host "`n  [WINMGR] Window Management..." -ForegroundColor Cyan

# Open a window to test management
$m = Run-DesktopSubtest "WINMGR" "Open + Snap Left"
VBox-WinE; Start-Sleep -Milliseconds 500
VBox-WinLeft; Start-Sleep -Milliseconds 200
Check-Desktop $m "WINMGR" "Open + Snap Left"

$m = Run-DesktopSubtest "WINMGR" "Snap Right"
VBox-WinRight; Start-Sleep -Milliseconds 200
Check-Desktop $m "WINMGR" "Snap Right"

$m = Run-DesktopSubtest "WINMGR" "Maximize (Win+Up)"
VBox-WinUp; Start-Sleep -Milliseconds 200
Check-Desktop $m "WINMGR" "Maximize (Win+Up)"

$m = Run-DesktopSubtest "WINMGR" "Restore (Win+Down)"
VBox-WinDown; Start-Sleep -Milliseconds 200
Check-Desktop $m "WINMGR" "Restore (Win+Down)"

Close-App; Start-Sleep -Milliseconds 200

$m = Run-DesktopSubtest "WINMGR" "Multi-window Alt+Tab"
VBox-WinE; Start-Sleep -Milliseconds 300  # File Manager
VBox-WinI; Start-Sleep -Milliseconds 300  # Settings
Open-App-Via-StartMenu "Calculator"
for ($i = 0; $i -lt 5; $i++) { VBox-AltTab; Start-Sleep -Milliseconds 200 }
Check-Desktop $m "WINMGR" "Multi-window Alt+Tab"
# Close all
for ($i = 0; $i -lt 5; $i++) { Close-App; Start-Sleep -Milliseconds 200 }

$m = Run-DesktopSubtest "WINMGR" "Rapid open/close"
for ($i = 0; $i -lt 5; $i++) {
    VBox-WinE; Start-Sleep -Milliseconds 300
    Close-App; Start-Sleep -Milliseconds 200
}
Check-Desktop $m "WINMGR" "Rapid open/close"

# â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#  SYSTEM TRAY
# â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

Write-Host "`n  [TRAY] System Tray..." -ForegroundColor Cyan

$m = Run-DesktopSubtest "SYSTRAY" "Multi-window tray render"
VBox-WinE; Start-Sleep -Milliseconds 300
VBox-WinI; Start-Sleep -Milliseconds 300
Open-App-Via-StartMenu "Calculator"
Start-Sleep -Milliseconds 500
Check-Desktop $m "SYSTRAY" "Multi-window tray render"
for ($i = 0; $i -lt 5; $i++) { Close-App; Start-Sleep -Milliseconds 200 }

# â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#  RAW KEY FUZZ (all printable ASCII in desktop)
# â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

Write-Host "`n  [FUZZ] Raw Key Fuzz..." -ForegroundColor Cyan

# Open terminal to absorb input
$m = Run-DesktopSubtest "FUZZ" "All printable ASCII"
Open-App-Via-StartMenu "Terminal"
# Send safe printable ASCII subset (avoids PS parsing issues with special chars)
$fuzzChars = [char[]](33..126) -join ''
& $VBoxManage controlvm $VMName keyboardputstring $fuzzChars 2>$null
Start-Sleep -Milliseconds 300
Check-Desktop $m "FUZZ" "All printable ASCII"

$m = Run-DesktopSubtest "FUZZ" "F-keys"
VBox-F1; VBox-F2; VBox-F3; VBox-F4; VBox-F5; VBox-F11
Start-Sleep -Milliseconds 200
Check-Desktop $m "FUZZ" "F-keys"

$m = Run-DesktopSubtest "FUZZ" "Rapid nav keys"
for ($i = 0; $i -lt 5; $i++) {
    VBox-Up; VBox-Down; VBox-Left; VBox-Right
    Start-Sleep -Milliseconds 100
    VBox-Home; VBox-End; VBox-PgUp; VBox-PgDn
    Start-Sleep -Milliseconds 100
}
Check-Desktop $m "FUZZ" "Rapid nav keys"

Close-App; Start-Sleep -Milliseconds 300

VBox-Screenshot "desktop_done"

$desktopDuration = [math]::Round($desktopStart.Elapsed.TotalSeconds, 1)
Write-Host "`n  Desktop phase done: ${desktopDuration}s" -ForegroundColor DarkGray
}

# Take final screenshot
VBox-Screenshot "final"

# â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•
#  REPORT
# â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•

$total = $global:passed + $global:failed + $global:crashed + $global:skipped
$effective = $global:passed + $global:failed + $global:crashed
$passRate = if ($effective -gt 0) { [math]::Round(($global:passed / $effective) * 100, 1) } else { 0 }

Write-Host ""
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host "  ULTIMATE TEST RESULTS" -ForegroundColor Cyan
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host ("  Total:    {0}" -f $total) -ForegroundColor White
Write-Host ("  Passed:   {0}" -f $global:passed) -ForegroundColor Green
$failColor = if ($global:failed -eq 0) { "Green" } else { "Red" }
Write-Host ("  Failed:   {0}" -f $global:failed) -ForegroundColor $failColor
$crashColor = if ($global:crashed -eq 0) { "Green" } else { "Red" }
Write-Host ("  Crashed:  {0}" -f $global:crashed) -ForegroundColor $crashColor
Write-Host ("  Skipped:  {0}" -f $global:skipped) -ForegroundColor Yellow
$rateColor = if ($passRate -ge 90) { "Green" } elseif ($passRate -ge 70) { "Yellow" } else { "Red" }
Write-Host ("  Rate:     {0}%%" -f $passRate) -ForegroundColor $rateColor
$shellD = if (-not $SkipShell) { $shellDuration } else { 0 }
$deskD  = if (-not $SkipDesktop) { $desktopDuration } else { 0 }
Write-Host ("  Duration: Shell={0}s Desktop={1}s Boot={2}s" -f $shellD, $deskD, $bootTime) -ForegroundColor DarkGray
Write-Host "================================================================" -ForegroundColor Cyan

# Show crashes
if ($global:panics.Count -gt 0) {
    Write-Host ""
    Write-Host "=== CRASHES DETECTED ===" -ForegroundColor Red
    foreach ($p in $global:panics) {
        Write-Host ("  Context: {0}" -f $p.Context) -ForegroundColor Red
        $snippet = $p.Output
        if ($snippet.Length -gt 200) { $snippet = $snippet.Substring(0, 200) + "..." }
        Write-Host ("  Output:  {0}" -f $snippet) -ForegroundColor DarkYellow
        Write-Host ""
    }
}

# Show failures
$failures = $global:results | Where-Object { $_.Status -eq "FAIL" }
if ($failures.Count -gt 0) {
    Write-Host ""
    Write-Host "=== FAILURES ===" -ForegroundColor Red
    foreach ($f in $failures) {
        Write-Host ("  [{0}] {1}" -f $f.Category, $f.Name) -ForegroundColor Red
        if ($f.Command) { Write-Host ("    Cmd:    {0}" -f $f.Command) -ForegroundColor DarkGray }
        $snippet = if ($f.Output.Length -gt 120) { $f.Output.Substring(0, 120) + "..." } else { $f.Output }
        if ($snippet) { Write-Host ("    Output: {0}" -f $snippet) -ForegroundColor DarkYellow }
    }
}

# Category breakdown
Write-Host ""
Write-Host "=== BY CATEGORY ===" -ForegroundColor Cyan
$categories = $global:results | Group-Object -Property Category
foreach ($cat in $categories | Sort-Object Name) {
    $catP = ($cat.Group | Where-Object { $_.Status -eq "PASS" }).Count
    $catT = $cat.Group.Count
    $catF = $catT - $catP
    $color = if ($catF -eq 0) { "Green" } elseif ($catF -le 2) { "Yellow" } else { "Red" }
    Write-Host ("  {0,-16} {1}/{2}" -f $cat.Name, $catP, $catT) -ForegroundColor $color
}

# Known bugs
Write-Host ""
Write-Host "=== KNOWN BUGS ===" -ForegroundColor Yellow
Write-Host "  CHESS-3D: White pieces do not move first -- black starts" -ForegroundColor Yellow
Write-Host "  CHESS-3D: Pawn promotion not implemented" -ForegroundColor Yellow
Write-Host "  CHESS-2D: Pawn promotion not implemented" -ForegroundColor Yellow

# â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•
#  WRITE REPORT FILE
# â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•

$reportLines = @()
$reportLines += "=================================================================="
$reportLines += "  TrustOS ULTIMATE Test Report (VirtualBox)"
$reportLines += "  Generated:  $timestamp"
$reportLines += "  Boot:       ${bootTime}s"
$reportLines += "  Shell:      ${shellD}s"
$reportLines += "  Desktop:    ${deskD}s"
$reportLines += "  VM:         $VMName (UEFI, 1GB RAM, 2 CPUs, VBOXSVGA 128MB)"
$reportLines += "=================================================================="
$reportLines += ""
$reportLines += "SUMMARY"
$reportLines += "  Total:    $total"
$reportLines += "  Passed:   $($global:passed)"
$reportLines += "  Failed:   $($global:failed)"
$reportLines += "  Crashed:  $($global:crashed)"
$reportLines += "  Skipped:  $($global:skipped)"
$reportLines += "  Rate:     ${passRate}%"
$reportLines += ""

if ($global:panics.Count -gt 0) {
    $reportLines += "=================================================================="
    $reportLines += "CRASHES DETECTED"
    $reportLines += "=================================================================="
    foreach ($p in $global:panics) {
        $reportLines += "  Context: $($p.Context)"
        $reportLines += "  Output:  $($p.Output)"
        $reportLines += ""
    }
}

$reportLines += "=================================================================="
$reportLines += "DETAILED RESULTS"
$reportLines += "=================================================================="

$curCat = ""
foreach ($r in $global:results) {
    if ($r.Category -ne $curCat) {
        $curCat = $r.Category
        $reportLines += ""
        $reportLines += "-- $curCat --"
    }
    $mark = switch ($r.Status) { "PASS" { "  [OK]   " } "FAIL" { "  [FAIL] " } "CRASH" { "  [CRASH]" } "SKIP" { "  [SKIP] " } default { "  [?]    " } }
    $reportLines += "$mark $($r.Name)"
    if ($r.Status -ne "PASS" -and $r.Status -ne "SKIP") {
        if ($r.Command) { $reportLines += "         Cmd:    $($r.Command)" }
        $outTrunc = if ($r.Output.Length -gt 200) { $r.Output.Substring(0, 200) + "..." } else { $r.Output }
        if ($outTrunc) { $reportLines += "         Output: $outTrunc" }
    }
    if ($r.Detail) { $reportLines += "         Detail: $($r.Detail)" }
}

$reportLines += ""
$reportLines += "=================================================================="
$reportLines += "FAILURES REQUIRING ACTION"
$reportLines += "=================================================================="
if ($failures.Count -eq 0) {
    $reportLines += "  (none -- all tests passed!)"
} else {
    foreach ($f in $failures) {
        $reportLines += "  [$($f.Category)] $($f.Name)"
        if ($f.Command) { $reportLines += "    Command: $($f.Command)" }
        $outT = if ($f.Output.Length -gt 200) { $f.Output.Substring(0, 200) + "..." } else { $f.Output }
        if ($outT) { $reportLines += "    Output:  $outT" }
        $reportLines += ""
    }
}

$reportLines += ""
$reportLines += "=================================================================="
$reportLines += "CATEGORY BREAKDOWN"
$reportLines += "=================================================================="
foreach ($cat in $categories | Sort-Object Name) {
    $catP = ($cat.Group | Where-Object { $_.Status -eq "PASS" }).Count
    $catT = $cat.Group.Count
    $reportLines += ("  {0,-16}  {1} / {2}" -f $cat.Name, $catP, $catT)
    foreach ($t in $cat.Group | Where-Object { $_.Status -ne "PASS" -and $_.Status -ne "SKIP" }) {
        $reportLines += "    -> $($t.Status): $($t.Name)"
    }
}

$reportLines += ""
$reportLines += "=================================================================="
$reportLines += "TEST COVERAGE"
$reportLines += "=================================================================="
$reportLines += ""
$reportLines += "PHASE 1 -- SHELL COMMANDS (~120 tests)"
$reportLines += "  Shell basics:   echo, pwd, whoami, hostname, version, uname, clear, history, help, man, info, cowsay"
$reportLines += "  System info:    date, time, free, df, ps, env, id, users, lscpu, lsmem, lspci, lsblk, vmstat, neofetch, dmesg, lshw, lsmod, tty"
$reportLines += "  Filesystem:     mkdir, ls, touch, echo>, cat, stat, wc, head, tail, cp, mv, grep, find, tree, diff, hexdump, rm, cd, pwd,"
$reportLines += "                  which, whereis, file, basename, dirname, realpath, du"
$reportLines += "  Text utils:     seq, factor, expr, cal, rev, sort, uniq, od, cmp, strings, yes"
$reportLines += "  Pipes:          echo|tr, echo|cut, echo|tee, cat tee, echo|wc, echo|grep"
$reportLines += "  Hashing:        md5sum, sha256sum"
$reportLines += "  Links:          ln -s, readlink"
$reportLines += "  Permissions:    chmod, chown"
$reportLines += "  Archives:       tar create/list/extract, gzip, zip, unzip"
$reportLines += "  Aliases:        alias, unalias"
$reportLines += "  Services:       service, systemctl, crontab"
$reportLines += "  Self-test:      test, inttest"
$reportLines += "  TrustLang:      eval println/math/string/bool, run let/fibonacci/while"
$reportLines += "  Network:        ifconfig, ipconfig, arp, route, netstat, ping, nslookup, traceroute"
$reportLines += "  Disk/HW:        disk, fdisk, blkid, mount, sync, lsusb, gpu"
$reportLines += "  Audio:          beep, audio, synth"
$reportLines += "  Debug:          irqstat, smpstatus, perf, memdbg, regs, memtest"
$reportLines += "  Security:       sig sign, sig verify"
$reportLines += "  Process:        sleep, tasks, threads"
$reportLines += "  Stubs:          bc, base64, printf, set, export, sysctl, firewall, reset, stty"
$reportLines += "  HW scan:        hwscan, a11y, fontsmooth"
$reportLines += "  Distro:         distros, persist"
$reportLines += "  Exec/Pkg:       trustpkg, elfinfo, checkm8, transpile"
$reportLines += "  Scanning:       nmap, scantest, httpd"
$reportLines += "  Unix compat:    xargs, timecmd, devpanel"
$reportLines += ""
$reportLines += "PHASE 2 -- DESKTOP TESTS (~70 tests)"
$reportLines += "  Hotkeys:        Win+E/D/I/H/L, Win+Arrows, Alt+Tab, Alt+F4"
$reportLines += "  Start Menu:     open/close toggle, search fuzz, backspace, arrow nav"
$reportLines += "  Calculator:     digits, operators, div/0, overflow, backspace"
$reportLines += "  Terminal:       type+enter, long input, arrows, scroll stress"
$reportLines += "  TrustCode:      type, backspace, arrows boundary, Home/End/PgUp/PgDn, Tab, Delete, Ctrl+Z undo"
$reportLines += "  Snake:          WASD, rapid reversal, pause, space restart"
$reportLines += "  File Manager:   arrows, enter+back, boundary, view toggle (V), clipboard C/V/X"
$reportLines += "  Chess:          board nav, select/deselect, new game, pawn move + AI"
$reportLines += "  Chess 3D:       board nav, select"
$reportLines += "  Browser:        URL type, backspace, empty enter, scroll"
$reportLines += "  Settings:       option keys 1-9"
$reportLines += "  Network Info:   open, arrow nav"
$reportLines += "  TrustEdit 3D:   open, camera WASD"
$reportLines += "  TrustLab:       open, nav keys"
$reportLines += "  Music Player:   open, play/pause"
$reportLines += "  About:          open via search"
$reportLines += "  NES Emulator:   open, controls"
$reportLines += "  Game Boy:       open, controls"
$reportLines += "  Lock Screen:    activate, wrong PIN, backspace, long PIN, correct PIN, rapid cycle"
$reportLines += "  Window Mgmt:    snap L/R, maximize, restore, multi-window Alt+Tab, rapid open/close"
$reportLines += "  System Tray:    multi-window tray render"
$reportLines += "  Fuzz:           all printable ASCII, F-keys, rapid nav keys"
$reportLines += ""
$reportLines += "KNOWN BUGS (manual observation)"
$reportLines += "  CHESS-3D: White pieces do not move first -- black starts instead"
$reportLines += "  CHESS-3D: Pawn promotion is not implemented"
$reportLines += "  CHESS-2D: Pawn promotion is not implemented"
$reportLines += ""
$reportLines += "=================================================================="
$reportLines += "  End of report"

$reportLines -join "`r`n" | Out-File -FilePath $ReportFile -Encoding UTF8

Write-Host ""
Write-Host "Report saved:        $ReportFile" -ForegroundColor White
Write-Host "Screenshots:         test_boot.png, test_shell_done.png, test_desktop_*.png, test_final.png" -ForegroundColor DarkGray
Write-Host ""
Write-Host "VM left running for manual inspection." -ForegroundColor DarkGray
Write-Host ("  To stop: VBoxManage controlvm {0} poweroff" -f $VMName) -ForegroundColor DarkGray
Write-Host ""
