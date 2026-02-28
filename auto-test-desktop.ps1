<#
.SYNOPSIS
    TrustOS Desktop App Input Stress Test (QEMU + Serial TCP + QMP Monitor)
.DESCRIPTION
    Boots TrustOS in QEMU, enters desktop mode, opens each app one by one,
    and systematically sends ALL possible keyboard inputs to each app to detect
    panics, crashes, and hangs. Uses QEMU monitor sendkey for modifier keys
    and serial TCP for ASCII input.
.NOTES
    Requires: QEMU, trustos.iso
    Output:   desktop_test_report.txt + console summary
#>

param(
    [string]$IsoPath = "$PSScriptRoot\trustos.iso",
    [int]$SerialPort = 5556,
    [int]$MonitorPort = 4445,
    [int]$BootTimeout = 30,
    [int]$CmdTimeout = 5,
    [string]$ReportFile = "$PSScriptRoot\desktop_test_report.txt"
)

$QemuExe = "C:\Program Files\qemu\qemu-system-x86_64.exe"
$timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"

$global:passed = 0
$global:failed = 0
$global:crashed = 0
$global:results = @()
$global:panics = @()

# ---------------------------------------------------------------
#  HELPER FUNCTIONS
# ---------------------------------------------------------------

function Send-Serial {
    param($stream, [string]$text)
    $bytes = [System.Text.Encoding]::ASCII.GetBytes($text)
    $stream.Write($bytes, 0, $bytes.Length)
    $stream.Flush()
}

function Send-SerialByte {
    param($stream, [byte]$b)
    $stream.WriteByte($b)
    $stream.Flush()
}

function Read-Serial {
    param($stream, [int]$timeoutMs = 500)
    $buffer = New-Object byte[] 16384
    $output = ""
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    while ($sw.ElapsedMilliseconds -lt $timeoutMs) {
        if ($stream.DataAvailable) {
            $read = $stream.Read($buffer, 0, $buffer.Length)
            if ($read -gt 0) {
                $output += [System.Text.Encoding]::ASCII.GetString($buffer, 0, $read)
            }
        } else {
            Start-Sleep -Milliseconds 30
        }
    }
    return $output
}

function Drain-Serial {
    param($stream)
    $buffer = New-Object byte[] 16384
    while ($stream.DataAvailable) {
        $stream.Read($buffer, 0, $buffer.Length) | Out-Null
    }
}

function Send-Monitor {
    param($monStream, $monWriter, [string]$cmd)
    $monWriter.Write("$cmd`r`n")
    $monWriter.Flush()
    Start-Sleep -Milliseconds 100
    # Read response
    $buffer = New-Object byte[] 4096
    $response = ""
    if ($monStream.DataAvailable) {
        $read = $monStream.Read($buffer, 0, $buffer.Length)
        if ($read -gt 0) {
            $response = [System.Text.Encoding]::ASCII.GetString($buffer, 0, $read)
        }
    }
    return $response
}

function Send-Key {
    param($monStream, $monWriter, [string]$keyName, [int]$holdMs = 100)
    Send-Monitor -monStream $monStream -monWriter $monWriter -cmd "sendkey $keyName $holdMs"
}

function Send-KeyCombo {
    param($monStream, $monWriter, [string]$combo, [int]$holdMs = 200)
    # combo format: "meta_l-e" for Win+E, "alt-f4" for Alt+F4
    Send-Monitor -monStream $monStream -monWriter $monWriter -cmd "sendkey $combo $holdMs"
}

function Check-ForPanic {
    param($stream, [string]$context)
    $output = Read-Serial -stream $stream -timeoutMs 300
    if ($output -match "PANIC|panic|EXCEPTION|page fault|double fault|stack overflow|kernel panic|assertion failed") {
        $global:crashed++
        $global:panics += @{
            Context = $context
            Output = $output.Substring(0, [Math]::Min(300, $output.Length))
        }
        Write-Host "    *** CRASH DETECTED: $context ***" -ForegroundColor Red
        return $true
    }
    return $false
}

function Open-StartMenu {
    param($monStream, $monWriter, $serialStream)
    # Send Win key press+release via QEMU monitor to toggle start menu
    Send-Key -monStream $monStream -monWriter $monWriter -keyName "meta_l" -holdMs 150
    Start-Sleep -Milliseconds 400
}

function Open-App-Via-StartMenu {
    param($monStream, $monWriter, $serialStream, [string]$appName)
    
    # Open start menu
    Open-StartMenu -monStream $monStream -monWriter $monWriter -serialStream $serialStream
    
    # Type app name in search
    foreach ($ch in $appName.ToCharArray()) {
        Send-SerialByte -stream $serialStream -b ([byte][char]$ch)
        Start-Sleep -Milliseconds 50
    }
    Start-Sleep -Milliseconds 200
    
    # Press Enter to launch
    Send-SerialByte -stream $serialStream -b 0x0D
    Start-Sleep -Milliseconds 500
}

function Close-App {
    param($serialStream)
    # ESC to close focused window
    Send-SerialByte -stream $serialStream -b 0x1B
    Start-Sleep -Milliseconds 300
}

function Record-Result {
    param([string]$category, [string]$name, [string]$status, [string]$detail = "")
    
    if ($status -eq "PASS") { $global:passed++ }
    elseif ($status -eq "CRASH") { $global:crashed++ }
    else { $global:failed++ }
    
    $global:results += @{
        Category = $category
        Name     = $name
        Status   = $status
        Detail   = $detail
    }
}

# ---------------------------------------------------------------
#  APP TEST FUNCTIONS
# ---------------------------------------------------------------

function Test-Calculator {
    param($monStream, $monWriter, $serialStream)
    
    Write-Host "  [CALC] Testing Calculator..." -ForegroundColor Cyan
    Open-App-Via-StartMenu -monStream $monStream -monWriter $monWriter -serialStream $serialStream -appName "Calculator"
    Start-Sleep -Milliseconds 500
    
    $crashed = $false
    
    # Test 1: All digit keys
    Write-Host "    Digits 0-9..." -NoNewline
    foreach ($d in 0..9) {
        Send-SerialByte -stream $serialStream -b ([byte]([char]"$d"[0]))
        Start-Sleep -Milliseconds 30
    }
    Start-Sleep -Milliseconds 100
    $crashed = Check-ForPanic -stream $serialStream -context "Calculator: digits 0-9"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Clear
    Send-SerialByte -stream $serialStream -b ([byte][char]'C')
    Start-Sleep -Milliseconds 100
    
    # Test 2: All operators
    Write-Host "    Operators +-*/%%..." -NoNewline
    foreach ($op in @('+', '-', '*', '/', '%')) {
        Send-SerialByte -stream $serialStream -b ([byte][char]'5')
        Send-SerialByte -stream $serialStream -b ([byte][char]$op)
        Send-SerialByte -stream $serialStream -b ([byte][char]'3')
        Send-SerialByte -stream $serialStream -b 0x0D  # Enter / equals
        Start-Sleep -Milliseconds 50
        Send-SerialByte -stream $serialStream -b ([byte][char]'C')
        Start-Sleep -Milliseconds 50
    }
    $crashed = Check-ForPanic -stream $serialStream -context "Calculator: operators"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 3: Division by zero
    Write-Host "    Division by zero..." -NoNewline
    Send-SerialByte -stream $serialStream -b ([byte][char]'5')
    Send-SerialByte -stream $serialStream -b ([byte][char]'/')
    Send-SerialByte -stream $serialStream -b ([byte][char]'0')
    Send-SerialByte -stream $serialStream -b 0x0D
    Start-Sleep -Milliseconds 200
    $crashed = Check-ForPanic -stream $serialStream -context "Calculator: div by zero"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    Send-SerialByte -stream $serialStream -b ([byte][char]'C')
    
    # Test 4: Parentheses mismatch
    Write-Host "    Unbalanced parens..." -NoNewline
    foreach ($seq in @("((((", "))))", "(((1+", ")1+2(")) {
        foreach ($ch in $seq.ToCharArray()) {
            Send-SerialByte -stream $serialStream -b ([byte][char]$ch)
            Start-Sleep -Milliseconds 20
        }
        Send-SerialByte -stream $serialStream -b 0x0D
        Start-Sleep -Milliseconds 50
        Send-SerialByte -stream $serialStream -b ([byte][char]'C')
    }
    $crashed = Check-ForPanic -stream $serialStream -context "Calculator: unbalanced parens"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 5: Backspace on empty
    Write-Host "    Backspace on empty..." -NoNewline
    Send-SerialByte -stream $serialStream -b ([byte][char]'C')
    for ($i = 0; $i -lt 20; $i++) {
        Send-SerialByte -stream $serialStream -b 0x08  # Backspace
        Start-Sleep -Milliseconds 20
    }
    $crashed = Check-ForPanic -stream $serialStream -context "Calculator: backspace empty"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 6: Repeated equals with nothing
    Write-Host "    Repeated Enter..." -NoNewline
    Send-SerialByte -stream $serialStream -b ([byte][char]'C')
    for ($i = 0; $i -lt 20; $i++) {
        Send-SerialByte -stream $serialStream -b 0x0D
        Start-Sleep -Milliseconds 20
    }
    $crashed = Check-ForPanic -stream $serialStream -context "Calculator: repeated enter"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 7: Scientific functions
    Write-Host "    Scientific funcs..." -NoNewline
    Send-SerialByte -stream $serialStream -b ([byte][char]'C')
    Send-SerialByte -stream $serialStream -b ([byte][char]'0')
    Send-SerialByte -stream $serialStream -b ([byte][char]'s')  # sqrt(0)
    Start-Sleep -Milliseconds 50
    Send-SerialByte -stream $serialStream -b ([byte][char]'C')
    $crashed = Check-ForPanic -stream $serialStream -context "Calculator: sqrt(0)"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 8: Overflow - huge number
    Write-Host "    Overflow..." -NoNewline
    Send-SerialByte -stream $serialStream -b ([byte][char]'C')
    for ($i = 0; $i -lt 60; $i++) {
        Send-SerialByte -stream $serialStream -b ([byte][char]'9')
        Start-Sleep -Milliseconds 10
    }
    Send-SerialByte -stream $serialStream -b ([byte][char]'*')
    for ($i = 0; $i -lt 60; $i++) {
        Send-SerialByte -stream $serialStream -b ([byte][char]'9')
        Start-Sleep -Milliseconds 10
    }
    Send-SerialByte -stream $serialStream -b 0x0D
    Start-Sleep -Milliseconds 200
    $crashed = Check-ForPanic -stream $serialStream -context "Calculator: overflow"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 9: All printable ASCII
    Write-Host "    All printable ASCII..." -NoNewline
    Send-SerialByte -stream $serialStream -b ([byte][char]'C')
    for ($b = 0x20; $b -le 0x7E; $b++) {
        Send-SerialByte -stream $serialStream -b ([byte]$b)
        Start-Sleep -Milliseconds 10
    }
    Start-Sleep -Milliseconds 200
    $crashed = Check-ForPanic -stream $serialStream -context "Calculator: all ASCII"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    Record-Result -category "CALCULATOR" -name "Full input fuzz" -status $(if ($crashed) { "CRASH" } else { "PASS" })
    
    Close-App -serialStream $serialStream
    Start-Sleep -Milliseconds 300
}

function Test-Terminal {
    param($monStream, $monWriter, $serialStream)
    
    Write-Host "  [TERM] Testing Terminal..." -ForegroundColor Cyan
    Open-App-Via-StartMenu -monStream $monStream -monWriter $monWriter -serialStream $serialStream -appName "Terminal"
    Start-Sleep -Milliseconds 500
    
    $crashed = $false
    
    # Test 1: Empty Enter
    Write-Host "    Empty Enter x20..." -NoNewline
    for ($i = 0; $i -lt 20; $i++) {
        Send-SerialByte -stream $serialStream -b 0x0D
        Start-Sleep -Milliseconds 50
    }
    $crashed = Check-ForPanic -stream $serialStream -context "Terminal: empty enter"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 2: Backspace on empty input
    Write-Host "    Backspace on empty..." -NoNewline
    for ($i = 0; $i -lt 30; $i++) {
        Send-SerialByte -stream $serialStream -b 0x08
        Start-Sleep -Milliseconds 20
    }
    $crashed = Check-ForPanic -stream $serialStream -context "Terminal: backspace empty"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 3: Very long input line
    Write-Host "    Long input (500 chars)..." -NoNewline
    for ($i = 0; $i -lt 500; $i++) {
        Send-SerialByte -stream $serialStream -b ([byte][char]'A')
        Start-Sleep -Milliseconds 5
    }
    Send-SerialByte -stream $serialStream -b 0x0D
    Start-Sleep -Milliseconds 300
    $crashed = Check-ForPanic -stream $serialStream -context "Terminal: long input"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 4: Arrow keys (history navigation with empty history)
    Write-Host "    Arrow keys..." -NoNewline
    for ($i = 0; $i -lt 10; $i++) {
        Send-SerialByte -stream $serialStream -b 0xF0  # KEY_UP
        Start-Sleep -Milliseconds 30
        Send-SerialByte -stream $serialStream -b 0xF1  # KEY_DOWN
        Start-Sleep -Milliseconds 30
    }
    $crashed = Check-ForPanic -stream $serialStream -context "Terminal: arrow keys"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 5: All control characters
    Write-Host "    Control characters..." -NoNewline
    for ($b = 0x01; $b -le 0x1F; $b++) {
        Send-SerialByte -stream $serialStream -b ([byte]$b)
        Start-Sleep -Milliseconds 30
    }
    Send-SerialByte -stream $serialStream -b 0x7F  # DEL
    Start-Sleep -Milliseconds 200
    $crashed = Check-ForPanic -stream $serialStream -context "Terminal: ctrl chars"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 6: Special navigation keys
    Write-Host "    Navigation keys..." -NoNewline
    $navKeys = @(0xF0, 0xF1, 0xF2, 0xF3, 0xF4, 0xF5, 0xF6, 0xF7, 0xF8)  # UP DOWN LEFT RIGHT HOME END DEL PGUP PGDN
    foreach ($k in $navKeys) {
        for ($i = 0; $i -lt 5; $i++) {
            Send-SerialByte -stream $serialStream -b ([byte]$k)
            Start-Sleep -Milliseconds 20
        }
    }
    $crashed = Check-ForPanic -stream $serialStream -context "Terminal: nav keys"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 7: Scroll test (fill terminal with output then scroll)
    Write-Host "    Scroll stress..." -NoNewline
    Send-Serial -stream $serialStream -text "seq 200`r"
    Start-Sleep -Milliseconds 1000
    # Scroll up many times
    for ($i = 0; $i -lt 30; $i++) {
        Send-SerialByte -stream $serialStream -b 0xF7  # PGUP
        Start-Sleep -Milliseconds 30
    }
    for ($i = 0; $i -lt 30; $i++) {
        Send-SerialByte -stream $serialStream -b 0xF8  # PGDN
        Start-Sleep -Milliseconds 30
    }
    $crashed = Check-ForPanic -stream $serialStream -context "Terminal: scroll stress"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 8: All printable ASCII as single command
    Write-Host "    All printable ASCII..." -NoNewline
    for ($b = 0x20; $b -le 0x7E; $b++) {
        Send-SerialByte -stream $serialStream -b ([byte]$b)
        Start-Sleep -Milliseconds 5
    }
    Send-SerialByte -stream $serialStream -b 0x0D
    Start-Sleep -Milliseconds 300
    $crashed = Check-ForPanic -stream $serialStream -context "Terminal: all ASCII"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    Record-Result -category "TERMINAL" -name "Full input fuzz" -status $(if ($crashed) { "CRASH" } else { "PASS" })
    
    Close-App -serialStream $serialStream
    Start-Sleep -Milliseconds 300
}

function Test-TextEditor {
    param($monStream, $monWriter, $serialStream)
    
    Write-Host "  [EDIT] Testing TrustCode Editor..." -ForegroundColor Cyan
    Open-App-Via-StartMenu -monStream $monStream -monWriter $monWriter -serialStream $serialStream -appName "Text Editor"
    Start-Sleep -Milliseconds 500
    
    $crashed = $false
    
    # Test 1: Type text and Enter
    Write-Host "    Type + Enter..." -NoNewline
    Send-Serial -stream $serialStream -text "Hello World"
    Send-SerialByte -stream $serialStream -b 0x0D
    Send-Serial -stream $serialStream -text "Second line"
    Send-SerialByte -stream $serialStream -b 0x0D
    Start-Sleep -Milliseconds 200
    $crashed = Check-ForPanic -stream $serialStream -context "TrustCode: type+enter"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 2: Backspace through all text
    Write-Host "    Backspace stress..." -NoNewline
    for ($i = 0; $i -lt 50; $i++) {
        Send-SerialByte -stream $serialStream -b 0x08
        Start-Sleep -Milliseconds 10
    }
    $crashed = Check-ForPanic -stream $serialStream -context "TrustCode: backspace stress"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 3: Backspace on empty document
    Write-Host "    Backspace on empty..." -NoNewline
    for ($i = 0; $i -lt 20; $i++) {
        Send-SerialByte -stream $serialStream -b 0x08
        Start-Sleep -Milliseconds 10
    }
    $crashed = Check-ForPanic -stream $serialStream -context "TrustCode: backspace empty"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 4: Enter on empty lines (rapid)
    Write-Host "    Rapid Enter x50..." -NoNewline
    for ($i = 0; $i -lt 50; $i++) {
        Send-SerialByte -stream $serialStream -b 0x0D
        Start-Sleep -Milliseconds 10
    }
    $crashed = Check-ForPanic -stream $serialStream -context "TrustCode: rapid enter"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 5: Arrow keys at document boundaries
    Write-Host "    Arrow keys at boundary..." -NoNewline
    # Up many times (past beginning)
    for ($i = 0; $i -lt 100; $i++) {
        Send-SerialByte -stream $serialStream -b 0xF0  # UP
        Start-Sleep -Milliseconds 5
    }
    # Left many times
    for ($i = 0; $i -lt 100; $i++) {
        Send-SerialByte -stream $serialStream -b 0xF2  # LEFT
        Start-Sleep -Milliseconds 5
    }
    # Down many times (past end)
    for ($i = 0; $i -lt 200; $i++) {
        Send-SerialByte -stream $serialStream -b 0xF1  # DOWN
        Start-Sleep -Milliseconds 5
    }
    # Right many times
    for ($i = 0; $i -lt 200; $i++) {
        Send-SerialByte -stream $serialStream -b 0xF3  # RIGHT
        Start-Sleep -Milliseconds 5
    }
    $crashed = Check-ForPanic -stream $serialStream -context "TrustCode: arrow boundary"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 6: Home/End/PgUp/PgDn
    Write-Host "    Nav keys..." -NoNewline
    $navKeys = @(0xF4, 0xF5, 0xF7, 0xF8)  # HOME END PGUP PGDN
    foreach ($k in $navKeys) {
        for ($i = 0; $i -lt 10; $i++) {
            Send-SerialByte -stream $serialStream -b ([byte]$k)
            Start-Sleep -Milliseconds 20
        }
    }
    $crashed = Check-ForPanic -stream $serialStream -context "TrustCode: nav keys"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 7: Delete key
    Write-Host "    Delete key stress..." -NoNewline
    for ($i = 0; $i -lt 30; $i++) {
        Send-SerialByte -stream $serialStream -b 0xF6  # DELETE
        Start-Sleep -Milliseconds 10
    }
    $crashed = Check-ForPanic -stream $serialStream -context "TrustCode: delete key"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 8: Tab key
    Write-Host "    Tab key..." -NoNewline
    for ($i = 0; $i -lt 20; $i++) {
        Send-SerialByte -stream $serialStream -b 0x09  # TAB
        Start-Sleep -Milliseconds 10
    }
    $crashed = Check-ForPanic -stream $serialStream -context "TrustCode: tab key"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 9: Full printable ASCII
    Write-Host "    All printable ASCII..." -NoNewline
    for ($b = 0x20; $b -le 0x7E; $b++) {
        Send-SerialByte -stream $serialStream -b ([byte]$b)
        Start-Sleep -Milliseconds 5
    }
    Send-SerialByte -stream $serialStream -b 0x0D
    Start-Sleep -Milliseconds 200
    $crashed = Check-ForPanic -stream $serialStream -context "TrustCode: all ASCII"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 10: Ctrl+Z (undo) on empty
    Write-Host "    Ctrl+Z undo stress..." -NoNewline
    for ($i = 0; $i -lt 30; $i++) {
        Send-SerialByte -stream $serialStream -b 0x1A  # Ctrl+Z
        Start-Sleep -Milliseconds 20
    }
    $crashed = Check-ForPanic -stream $serialStream -context "TrustCode: undo stress"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 11: Ctrl+Y (redo) on empty
    Write-Host "    Ctrl+Y redo stress..." -NoNewline
    for ($i = 0; $i -lt 30; $i++) {
        Send-SerialByte -stream $serialStream -b 0x19  # Ctrl+Y
        Start-Sleep -Milliseconds 20
    }
    $crashed = Check-ForPanic -stream $serialStream -context "TrustCode: redo stress"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    Record-Result -category "TRUSTCODE" -name "Full input fuzz" -status $(if ($crashed) { "CRASH" } else { "PASS" })
    
    Close-App -serialStream $serialStream
    Start-Sleep -Milliseconds 300
}

function Test-Snake {
    param($monStream, $monWriter, $serialStream)
    
    Write-Host "  [SNAKE] Testing Snake Game..." -ForegroundColor Cyan
    Open-App-Via-StartMenu -monStream $monStream -monWriter $monWriter -serialStream $serialStream -appName "Snake"
    Start-Sleep -Milliseconds 500
    
    $crashed = $false
    
    # Test 1: Start game (space) and direction keys
    Write-Host "    WASD directions..." -NoNewline
    Send-SerialByte -stream $serialStream -b ([byte][char]' ')  # Start game
    Start-Sleep -Milliseconds 200
    foreach ($dir in @('w', 'a', 's', 'd', 'w', 'd', 's', 'a')) {
        Send-SerialByte -stream $serialStream -b ([byte][char]$dir)
        Start-Sleep -Milliseconds 100
    }
    $crashed = Check-ForPanic -stream $serialStream -context "Snake: WASD"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 2: Rapid direction changes (180-degree reversal)
    Write-Host "    Rapid reversal..." -NoNewline
    for ($i = 0; $i -lt 20; $i++) {
        Send-SerialByte -stream $serialStream -b ([byte][char]'w')
        Send-SerialByte -stream $serialStream -b ([byte][char]'s')
        Start-Sleep -Milliseconds 50
    }
    $crashed = Check-ForPanic -stream $serialStream -context "Snake: rapid reversal"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 3: Pause/unpause (p key)
    Write-Host "    Pause toggle..." -NoNewline
    for ($i = 0; $i -lt 10; $i++) {
        Send-SerialByte -stream $serialStream -b ([byte][char]'p')
        Start-Sleep -Milliseconds 100
    }
    $crashed = Check-ForPanic -stream $serialStream -context "Snake: pause toggle"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 4: All printable ASCII while playing
    Write-Host "    All ASCII during game..." -NoNewline
    Send-SerialByte -stream $serialStream -b ([byte][char]'p')  # Unpause if paused
    Start-Sleep -Milliseconds 100
    for ($b = 0x20; $b -le 0x7E; $b++) {
        Send-SerialByte -stream $serialStream -b ([byte]$b)
        Start-Sleep -Milliseconds 10
    }
    $crashed = Check-ForPanic -stream $serialStream -context "Snake: all ASCII"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 5: Space spam (restart game)
    Write-Host "    Space spam..." -NoNewline
    for ($i = 0; $i -lt 20; $i++) {
        Send-SerialByte -stream $serialStream -b ([byte][char]' ')
        Start-Sleep -Milliseconds 50
    }
    $crashed = Check-ForPanic -stream $serialStream -context "Snake: space spam"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 6: Difficulty keys (1-3)
    Write-Host "    Difficulty keys..." -NoNewline
    foreach ($d in @('1', '2', '3')) {
        Send-SerialByte -stream $serialStream -b ([byte][char]$d)
        Start-Sleep -Milliseconds 100
    }
    $crashed = Check-ForPanic -stream $serialStream -context "Snake: difficulty"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    Record-Result -category "SNAKE" -name "Full input fuzz" -status $(if ($crashed) { "CRASH" } else { "PASS" })
    
    Close-App -serialStream $serialStream
    Start-Sleep -Milliseconds 300
}

function Test-FileManager {
    param($monStream, $monWriter, $serialStream)
    
    Write-Host "  [FILES] Testing File Manager..." -ForegroundColor Cyan
    Open-App-Via-StartMenu -monStream $monStream -monWriter $monWriter -serialStream $serialStream -appName "Files"
    Start-Sleep -Milliseconds 500
    
    $crashed = $false
    
    # Test 1: Navigate with arrows (empty file list)
    Write-Host "    Arrow navigation..." -NoNewline
    for ($i = 0; $i -lt 20; $i++) {
        Send-SerialByte -stream $serialStream -b 0xF0  # UP
        Start-Sleep -Milliseconds 20
        Send-SerialByte -stream $serialStream -b 0xF1  # DOWN
        Start-Sleep -Milliseconds 20
    }
    $crashed = Check-ForPanic -stream $serialStream -context "FileManager: arrow nav"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 2: Enter on selections
    Write-Host "    Enter on items..." -NoNewline
    for ($i = 0; $i -lt 5; $i++) {
        Send-SerialByte -stream $serialStream -b 0x0D  # Enter
        Start-Sleep -Milliseconds 100
        Send-SerialByte -stream $serialStream -b 0x08  # Backspace (go back)
        Start-Sleep -Milliseconds 100
    }
    $crashed = Check-ForPanic -stream $serialStream -context "FileManager: enter"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 3: Delete key (should prompt or do nothing)
    Write-Host "    Delete key..." -NoNewline
    Send-SerialByte -stream $serialStream -b 0xF6  # DELETE
    Start-Sleep -Milliseconds 200
    Send-SerialByte -stream $serialStream -b 0x1B  # ESC to cancel any dialog
    Start-Sleep -Milliseconds 100
    $crashed = Check-ForPanic -stream $serialStream -context "FileManager: delete"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 4: Navigate way past list boundaries
    Write-Host "    Boundary scroll..." -NoNewline
    for ($i = 0; $i -lt 100; $i++) {
        Send-SerialByte -stream $serialStream -b 0xF0  # UP many times
        Start-Sleep -Milliseconds 10
    }
    for ($i = 0; $i -lt 100; $i++) {
        Send-SerialByte -stream $serialStream -b 0xF1  # DOWN many times
        Start-Sleep -Milliseconds 10
    }
    $crashed = Check-ForPanic -stream $serialStream -context "FileManager: boundary scroll"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 5: All printable ASCII
    Write-Host "    All printable ASCII..." -NoNewline
    for ($b = 0x20; $b -le 0x7E; $b++) {
        Send-SerialByte -stream $serialStream -b ([byte]$b)
        Start-Sleep -Milliseconds 10
    }
    $crashed = Check-ForPanic -stream $serialStream -context "FileManager: all ASCII"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 6: Tab key (might switch panels)
    Write-Host "    Tab key..." -NoNewline
    for ($i = 0; $i -lt 10; $i++) {
        Send-SerialByte -stream $serialStream -b 0x09
        Start-Sleep -Milliseconds 50
    }
    $crashed = Check-ForPanic -stream $serialStream -context "FileManager: tab"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    Record-Result -category "FILEMANAGER" -name "Full input fuzz" -status $(if ($crashed) { "CRASH" } else { "PASS" })
    
    Close-App -serialStream $serialStream
    Start-Sleep -Milliseconds 300
}

function Test-Chess {
    param($monStream, $monWriter, $serialStream)
    
    Write-Host "  [CHESS] Testing Chess..." -ForegroundColor Cyan
    Open-App-Via-StartMenu -monStream $monStream -monWriter $monWriter -serialStream $serialStream -appName "Chess"
    Start-Sleep -Milliseconds 500
    
    $crashed = $false
    
    # Test 1: Arrow keys to navigate board
    Write-Host "    Board navigation..." -NoNewline
    for ($i = 0; $i -lt 30; $i++) {
        Send-SerialByte -stream $serialStream -b 0xF0  # UP
        Start-Sleep -Milliseconds 20
        Send-SerialByte -stream $serialStream -b 0xF3  # RIGHT
        Start-Sleep -Milliseconds 20
        Send-SerialByte -stream $serialStream -b 0xF1  # DOWN
        Start-Sleep -Milliseconds 20
        Send-SerialByte -stream $serialStream -b 0xF2  # LEFT
        Start-Sleep -Milliseconds 20
    }
    $crashed = Check-ForPanic -stream $serialStream -context "Chess: board nav"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 2: Select/deselect (Enter/Space)
    Write-Host "    Select/deselect..." -NoNewline
    for ($i = 0; $i -lt 20; $i++) {
        Send-SerialByte -stream $serialStream -b 0x0D  # Enter (select)
        Start-Sleep -Milliseconds 50
        Send-SerialByte -stream $serialStream -b 0xF3  # Move right
        Start-Sleep -Milliseconds 50
        Send-SerialByte -stream $serialStream -b 0x0D  # Enter (try move)
        Start-Sleep -Milliseconds 50
    }
    $crashed = Check-ForPanic -stream $serialStream -context "Chess: select/deselect"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 3: Try to move to same square
    Write-Host "    Self-move..." -NoNewline
    Send-SerialByte -stream $serialStream -b 0x0D  # Select
    Start-Sleep -Milliseconds 50
    Send-SerialByte -stream $serialStream -b 0x0D  # Move to same
    Start-Sleep -Milliseconds 200
    $crashed = Check-ForPanic -stream $serialStream -context "Chess: self-move"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 4: Difficulty change
    Write-Host "    Difficulty keys..." -NoNewline
    foreach ($d in @('1', '2', '3')) {
        Send-SerialByte -stream $serialStream -b ([byte][char]$d)
        Start-Sleep -Milliseconds 100
    }
    $crashed = Check-ForPanic -stream $serialStream -context "Chess: difficulty"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 5: New game (n key)
    Write-Host "    New game..." -NoNewline
    Send-SerialByte -stream $serialStream -b ([byte][char]'n')
    Start-Sleep -Milliseconds 200
    $crashed = Check-ForPanic -stream $serialStream -context "Chess: new game"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 6: Make valid pawn move then AI responds
    Write-Host "    Valid pawn move + AI..." -NoNewline
    # Navigate to e2 pawn (row 6, col 4 from top-left)
    for ($i = 0; $i -lt 8; $i++) { Send-SerialByte -stream $serialStream -b 0xF0 } # go to top
    for ($i = 0; $i -lt 8; $i++) { Send-SerialByte -stream $serialStream -b 0xF2 } # go to left
    # Go to e2: right 4, down 6
    for ($i = 0; $i -lt 4; $i++) { Send-SerialByte -stream $serialStream -b 0xF3; Start-Sleep -Milliseconds 10 }
    for ($i = 0; $i -lt 6; $i++) { Send-SerialByte -stream $serialStream -b 0xF1; Start-Sleep -Milliseconds 10 }
    Send-SerialByte -stream $serialStream -b 0x0D  # Select pawn
    Start-Sleep -Milliseconds 100
    # Move up 2 (e4)
    Send-SerialByte -stream $serialStream -b 0xF0
    Send-SerialByte -stream $serialStream -b 0xF0
    Send-SerialByte -stream $serialStream -b 0x0D  # Confirm move
    Start-Sleep -Milliseconds 2000  # Wait for AI
    $crashed = Check-ForPanic -stream $serialStream -context "Chess: pawn move + AI"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 7: All printable ASCII
    Write-Host "    All printable ASCII..." -NoNewline
    for ($b = 0x20; $b -le 0x7E; $b++) {
        Send-SerialByte -stream $serialStream -b ([byte]$b)
        Start-Sleep -Milliseconds 10
    }
    $crashed = Check-ForPanic -stream $serialStream -context "Chess: all ASCII"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    Record-Result -category "CHESS" -name "Full input fuzz" -status $(if ($crashed) { "CRASH" } else { "PASS" })
    
    Close-App -serialStream $serialStream
    Start-Sleep -Milliseconds 300
}

function Test-Browser {
    param($monStream, $monWriter, $serialStream)
    
    Write-Host "  [WEB] Testing Browser..." -ForegroundColor Cyan
    Open-App-Via-StartMenu -monStream $monStream -monWriter $monWriter -serialStream $serialStream -appName "Browser"
    Start-Sleep -Milliseconds 500
    
    $crashed = $false
    
    # Test 1: Type URL
    Write-Host "    Type URL..." -NoNewline
    Send-Serial -stream $serialStream -text "example.com"
    Send-SerialByte -stream $serialStream -b 0x0D
    Start-Sleep -Milliseconds 1000
    $crashed = Check-ForPanic -stream $serialStream -context "Browser: URL"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 2: Backspace entire URL
    Write-Host "    Backspace URL..." -NoNewline
    for ($i = 0; $i -lt 30; $i++) {
        Send-SerialByte -stream $serialStream -b 0x08
        Start-Sleep -Milliseconds 10
    }
    $crashed = Check-ForPanic -stream $serialStream -context "Browser: backspace URL"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 3: Empty URL Enter
    Write-Host "    Empty URL Enter..." -NoNewline
    Send-SerialByte -stream $serialStream -b 0x0D
    Start-Sleep -Milliseconds 300
    $crashed = Check-ForPanic -stream $serialStream -context "Browser: empty URL"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 4: Scroll
    Write-Host "    Scroll..." -NoNewline
    for ($i = 0; $i -lt 20; $i++) {
        Send-SerialByte -stream $serialStream -b 0xF7  # PGUP
        Start-Sleep -Milliseconds 30
        Send-SerialByte -stream $serialStream -b 0xF8  # PGDN
        Start-Sleep -Milliseconds 30
    }
    $crashed = Check-ForPanic -stream $serialStream -context "Browser: scroll"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 5: All printable ASCII in URL bar
    Write-Host "    All ASCII in URL..." -NoNewline
    for ($b = 0x20; $b -le 0x7E; $b++) {
        Send-SerialByte -stream $serialStream -b ([byte]$b)
        Start-Sleep -Milliseconds 5
    }
    Send-SerialByte -stream $serialStream -b 0x0D
    Start-Sleep -Milliseconds 500
    $crashed = Check-ForPanic -stream $serialStream -context "Browser: all ASCII URL"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    Record-Result -category "BROWSER" -name "Full input fuzz" -status $(if ($crashed) { "CRASH" } else { "PASS" })
    
    Close-App -serialStream $serialStream
    Start-Sleep -Milliseconds 300
}

function Test-Game3D {
    param($monStream, $monWriter, $serialStream)
    
    Write-Host "  [3D] Testing TrustDoom 3D..." -ForegroundColor Cyan
    # Game3D might not be searchable as "TrustDoom", try index-based
    Open-StartMenu -monStream $monStream -monWriter $monWriter -serialStream $serialStream
    # Navigate to Game3D (it's not in the default list easily searchable)
    # Use arrow down to find it
    for ($i = 0; $i -lt 20; $i++) {
        Send-SerialByte -stream $serialStream -b 0xF1  # DOWN
        Start-Sleep -Milliseconds 50
    }
    Send-SerialByte -stream $serialStream -b 0x1B  # Cancel
    Start-Sleep -Milliseconds 200
    
    # Try direct: search "Snake" first then close, because Game3D doesn't have easy search match
    # Actually let's just test Snake's code paths since Game3D is harder to open via search
    # For now, skip or try with specific index
    Write-Host "    (testing via keyboard only - manual open needed)" -ForegroundColor DarkGray
    
    # We can still test behavior via a focused window if one was already open
    # For automated purposes, let's mark as skipped if we can't open it
    Record-Result -category "GAME3D" -name "Input fuzz" -status "SKIP" -detail "Cannot reliably open via serial search"
}

function Test-Settings {
    param($monStream, $monWriter, $serialStream)
    
    Write-Host "  [SET] Testing Settings..." -ForegroundColor Cyan
    Open-App-Via-StartMenu -monStream $monStream -monWriter $monWriter -serialStream $serialStream -appName "Settings"
    Start-Sleep -Milliseconds 500
    
    $crashed = $false
    
    # Test 1: Number keys 1-9 (settings options)
    Write-Host "    Option keys 1-9..." -NoNewline
    foreach ($d in 1..9) {
        Send-SerialByte -stream $serialStream -b ([byte]([char]"$d"[0]))
        Start-Sleep -Milliseconds 100
    }
    $crashed = Check-ForPanic -stream $serialStream -context "Settings: options 1-9"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 2: All printable ASCII
    Write-Host "    All printable ASCII..." -NoNewline
    for ($b = 0x20; $b -le 0x7E; $b++) {
        Send-SerialByte -stream $serialStream -b ([byte]$b)
        Start-Sleep -Milliseconds 10
    }
    $crashed = Check-ForPanic -stream $serialStream -context "Settings: all ASCII"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    Record-Result -category "SETTINGS" -name "Full input fuzz" -status $(if ($crashed) { "CRASH" } else { "PASS" })
    
    Close-App -serialStream $serialStream
    Start-Sleep -Milliseconds 300
}

function Test-HexViewer {
    param($monStream, $monWriter, $serialStream)
    
    Write-Host "  [HEX] Testing Hex Viewer (via File Manager)..." -ForegroundColor Cyan
    # Hex Viewer is opened by opening a file from File Manager
    # We'll test the BinaryViewer window type if we can open it
    # For now, test navigation keys that BinaryViewer handles
    
    Record-Result -category "HEXVIEWER" -name "Input fuzz" -status "SKIP" -detail "Requires file open from FileManager"
}

function Test-StartMenu {
    param($monStream, $monWriter, $serialStream)
    
    Write-Host "  [MENU] Testing Start Menu..." -ForegroundColor Cyan
    
    $crashed = $false
    
    # Test 1: Open/close toggle
    Write-Host "    Toggle open/close..." -NoNewline
    for ($i = 0; $i -lt 5; $i++) {
        Open-StartMenu -monStream $monStream -monWriter $monWriter -serialStream $serialStream
        Start-Sleep -Milliseconds 200
        Send-SerialByte -stream $serialStream -b 0x1B  # ESC to close
        Start-Sleep -Milliseconds 200
    }
    $crashed = Check-ForPanic -stream $serialStream -context "StartMenu: toggle"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 2: Search with all printable chars
    Write-Host "    Search fuzz..." -NoNewline
    Open-StartMenu -monStream $monStream -monWriter $monWriter -serialStream $serialStream
    for ($b = 0x20; $b -le 0x7E; $b++) {
        Send-SerialByte -stream $serialStream -b ([byte]$b)
        Start-Sleep -Milliseconds 10
    }
    Start-Sleep -Milliseconds 200
    $crashed = Check-ForPanic -stream $serialStream -context "StartMenu: search fuzz"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    Send-SerialByte -stream $serialStream -b 0x1B  # Close
    
    # Test 3: Backspace spam in search
    Write-Host "    Backspace spam..." -NoNewline
    Open-StartMenu -monStream $monStream -monWriter $monWriter -serialStream $serialStream
    Send-Serial -stream $serialStream -text "test"
    for ($i = 0; $i -lt 20; $i++) {
        Send-SerialByte -stream $serialStream -b 0x08
        Start-Sleep -Milliseconds 20
    }
    $crashed = Check-ForPanic -stream $serialStream -context "StartMenu: backspace spam"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    Send-SerialByte -stream $serialStream -b 0x1B  # Close
    
    # Test 4: Arrow navigation wrap-around
    Write-Host "    Arrow wrap-around..." -NoNewline
    Open-StartMenu -monStream $monStream -monWriter $monWriter -serialStream $serialStream
    for ($i = 0; $i -lt 30; $i++) {
        Send-SerialByte -stream $serialStream -b 0xF1  # DOWN
        Start-Sleep -Milliseconds 20
    }
    for ($i = 0; $i -lt 30; $i++) {
        Send-SerialByte -stream $serialStream -b 0xF0  # UP
        Start-Sleep -Milliseconds 20
    }
    $crashed = Check-ForPanic -stream $serialStream -context "StartMenu: arrow wrap"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    Send-SerialByte -stream $serialStream -b 0x1B  # Close
    
    # Test 5: Enter with no selection and no search
    Write-Host "    Enter empty..." -NoNewline
    Open-StartMenu -monStream $monStream -monWriter $monWriter -serialStream $serialStream
    Send-SerialByte -stream $serialStream -b 0x0D
    Start-Sleep -Milliseconds 200
    $crashed = Check-ForPanic -stream $serialStream -context "StartMenu: enter empty"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Close any window that might have opened
    Send-SerialByte -stream $serialStream -b 0x1B
    Start-Sleep -Milliseconds 200
    
    Record-Result -category "STARTMENU" -name "Full input fuzz" -status $(if ($crashed) { "CRASH" } else { "PASS" })
}

function Test-GlobalHotkeys {
    param($monStream, $monWriter, $serialStream)
    
    Write-Host "  [HOTKEY] Testing Global Hotkeys..." -ForegroundColor Cyan
    
    $crashed = $false
    
    # Test 1: Win+E (File Manager)
    Write-Host "    Win+E..." -NoNewline
    Send-KeyCombo -monStream $monStream -monWriter $monWriter -combo "meta_l-e"
    Start-Sleep -Milliseconds 500
    $crashed = Check-ForPanic -stream $serialStream -context "Hotkey: Win+E"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    Close-App -serialStream $serialStream
    Start-Sleep -Milliseconds 200
    
    # Test 2: Win+D (show desktop)
    Write-Host "    Win+D..." -NoNewline
    Send-KeyCombo -monStream $monStream -monWriter $monWriter -combo "meta_l-d"
    Start-Sleep -Milliseconds 300
    $crashed = Check-ForPanic -stream $serialStream -context "Hotkey: Win+D"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 3: Win+I (settings)
    Write-Host "    Win+I..." -NoNewline
    Send-KeyCombo -monStream $monStream -monWriter $monWriter -combo "meta_l-i"
    Start-Sleep -Milliseconds 500
    $crashed = Check-ForPanic -stream $serialStream -context "Hotkey: Win+I"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    Close-App -serialStream $serialStream
    Start-Sleep -Milliseconds 200
    
    # Test 4: Win+H (high contrast toggle)
    Write-Host "    Win+H (high contrast)..." -NoNewline
    Send-KeyCombo -monStream $monStream -monWriter $monWriter -combo "meta_l-h"
    Start-Sleep -Milliseconds 300
    Send-KeyCombo -monStream $monStream -monWriter $monWriter -combo "meta_l-h"  # Toggle back
    Start-Sleep -Milliseconds 300
    $crashed = Check-ForPanic -stream $serialStream -context "Hotkey: Win+H"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 5: Win+Arrows (snap) with no window
    Write-Host "    Win+Arrows (no window)..." -NoNewline
    Send-KeyCombo -monStream $monStream -monWriter $monWriter -combo "meta_l-up"
    Start-Sleep -Milliseconds 100
    Send-KeyCombo -monStream $monStream -monWriter $monWriter -combo "meta_l-down"
    Start-Sleep -Milliseconds 100
    Send-KeyCombo -monStream $monStream -monWriter $monWriter -combo "meta_l-left"
    Start-Sleep -Milliseconds 100
    Send-KeyCombo -monStream $monStream -monWriter $monWriter -combo "meta_l-right"
    Start-Sleep -Milliseconds 100
    $crashed = Check-ForPanic -stream $serialStream -context "Hotkey: Win+Arrows"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 6: Alt+Tab with no windows
    Write-Host "    Alt+Tab (no windows)..." -NoNewline
    Send-KeyCombo -monStream $monStream -monWriter $monWriter -combo "alt-tab"
    Start-Sleep -Milliseconds 300
    $crashed = Check-ForPanic -stream $serialStream -context "Hotkey: Alt+Tab"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Test 7: Alt+F4 with no windows
    Write-Host "    Alt+F4 (no windows)..." -NoNewline
    Send-KeyCombo -monStream $monStream -monWriter $monWriter -combo "alt-f4"
    Start-Sleep -Milliseconds 300
    # This might exit desktop, check serial
    $output = Read-Serial -stream $serialStream -timeoutMs 500
    if ($output -match "exiting desktop|exit.*shell") {
        Write-Host " (exited desktop - re-entering)" -ForegroundColor Yellow
        # Re-enter desktop
        Start-Sleep -Milliseconds 500
        Send-Serial -stream $serialStream -text "desktop`r"
        Start-Sleep -Milliseconds 3000
        Drain-Serial -stream $serialStream
    } else {
        $crashed = Check-ForPanic -stream $serialStream -context "Hotkey: Alt+F4"
        if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    }
    
    Record-Result -category "HOTKEYS" -name "Full hotkey fuzz" -status $(if ($crashed) { "CRASH" } else { "PASS" })
}

function Test-RawByteFuzz {
    param($monStream, $monWriter, $serialStream)
    
    Write-Host "  [FUZZ] Testing Raw Byte Fuzz (all 0x00-0xFF)..." -ForegroundColor Cyan
    
    # Open a terminal to absorb the bytes
    Open-App-Via-StartMenu -monStream $monStream -monWriter $monWriter -serialStream $serialStream -appName "Terminal"
    Start-Sleep -Milliseconds 500
    
    $crashed = $false
    
    # Send every possible byte value
    Write-Host "    Bytes 0x00-0xFF..." -NoNewline
    for ($b = 0; $b -le 255; $b++) {
        Send-SerialByte -stream $serialStream -b ([byte]$b)
        Start-Sleep -Milliseconds 10
    }
    Start-Sleep -Milliseconds 500
    $crashed = Check-ForPanic -stream $serialStream -context "RawFuzz: all bytes 0x00-0xFF"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    # Send high bytes (special key range 0xF0-0xF8) rapidly
    Write-Host "    Rapid nav key fuzz..." -NoNewline
    for ($i = 0; $i -lt 50; $i++) {
        $b = Get-Random -Minimum 0xF0 -Maximum 0xF9
        Send-SerialByte -stream $serialStream -b ([byte]$b)
        Start-Sleep -Milliseconds 5
    }
    Start-Sleep -Milliseconds 300
    $crashed = Check-ForPanic -stream $serialStream -context "RawFuzz: rapid nav keys"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    
    Record-Result -category "RAWFUZZ" -name "All byte values" -status $(if ($crashed) { "CRASH" } else { "PASS" })
    
    Close-App -serialStream $serialStream
    Start-Sleep -Milliseconds 300
}

# ---------------------------------------------------------------
#  NEW FEATURE TESTS (v0.5.0 Desktop Enhancements)
# ---------------------------------------------------------------

function Test-LockScreen {
    param($monStream, $monWriter, $serialStream)
    
    Write-Host "  [LOCK] Testing Lock Screen..." -ForegroundColor Cyan
    
    $crashed = $false
    
    # Test 1: Win+L activates lock screen
    Write-Host "    Win+L activate..." -NoNewline
    Send-KeyCombo -monStream $monStream -monWriter $monWriter -combo "meta_l-l" -holdMs 200
    Start-Sleep -Milliseconds 800
    $output = Read-Serial -stream $serialStream -timeoutMs 500
    if ($output -match "\[GUI\] Win\+L.*lock screen") {
        Write-Host " OK" -ForegroundColor Green
        Record-Result -category "LOCKSCREEN" -name "Win+L activate" -status "PASS"
    } else {
        $crashed = Check-ForPanic -stream $serialStream -context "LockScreen: Win+L"
        if (-not $crashed) {
            Write-Host " OK (no serial confirm)" -ForegroundColor Yellow
            Record-Result -category "LOCKSCREEN" -name "Win+L activate" -status "PASS" -detail "no serial confirm but no crash"
        }
    }
    
    # Test 2: Wrong PIN triggers shake
    Write-Host "    Wrong PIN..." -NoNewline
    Send-Serial -stream $serialStream -text "9999"
    Start-Sleep -Milliseconds 100
    Send-SerialByte -stream $serialStream -b 0x0D  # Enter
    Start-Sleep -Milliseconds 500
    $output = Read-Serial -stream $serialStream -timeoutMs 500
    $wrongPinOk = $output -match "\[LOCK\] Wrong PIN"
    $crashed = Check-ForPanic -stream $serialStream -context "LockScreen: wrong PIN"
    if (-not $crashed) {
        Write-Host " OK" -ForegroundColor Green
        Record-Result -category "LOCKSCREEN" -name "Wrong PIN shake" -status "PASS"
    }
    
    # Test 3: Backspace clears PIN input
    Write-Host "    Backspace in PIN..." -NoNewline
    Send-Serial -stream $serialStream -text "123"
    for ($i = 0; $i -lt 10; $i++) {
        Send-SerialByte -stream $serialStream -b 0x08
        Start-Sleep -Milliseconds 20
    }
    $crashed = Check-ForPanic -stream $serialStream -context "LockScreen: backspace PIN"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    Record-Result -category "LOCKSCREEN" -name "Backspace PIN" -status $(if ($crashed) { "CRASH" } else { "PASS" })
    
    # Test 4: All printable ASCII while locked (should not crash)
    Write-Host "    All ASCII while locked..." -NoNewline
    for ($b = 0x20; $b -le 0x7E; $b++) {
        Send-SerialByte -stream $serialStream -b ([byte]$b)
        Start-Sleep -Milliseconds 5
    }
    Start-Sleep -Milliseconds 200
    $crashed = Check-ForPanic -stream $serialStream -context "LockScreen: all ASCII"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    Record-Result -category "LOCKSCREEN" -name "All ASCII while locked" -status $(if ($crashed) { "CRASH" } else { "PASS" })
    
    # Test 5: Rapid Enter spam while locked
    Write-Host "    Rapid Enter spam..." -NoNewline
    for ($i = 0; $i -lt 20; $i++) {
        Send-SerialByte -stream $serialStream -b 0x0D
        Start-Sleep -Milliseconds 30
    }
    $crashed = Check-ForPanic -stream $serialStream -context "LockScreen: enter spam"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    Record-Result -category "LOCKSCREEN" -name "Rapid Enter spam" -status $(if ($crashed) { "CRASH" } else { "PASS" })
    
    # Test 6: Long PIN overflow (16 char limit)
    Write-Host "    Long PIN overflow..." -NoNewline
    for ($i = 0; $i -lt 30; $i++) {
        Send-SerialByte -stream $serialStream -b ([byte][char]'9')
        Start-Sleep -Milliseconds 10
    }
    Send-SerialByte -stream $serialStream -b 0x0D
    Start-Sleep -Milliseconds 300
    $crashed = Check-ForPanic -stream $serialStream -context "LockScreen: long PIN"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    Record-Result -category "LOCKSCREEN" -name "Long PIN overflow" -status $(if ($crashed) { "CRASH" } else { "PASS" })
    
    # Test 7: Correct PIN unlocks (1234)
    Write-Host "    Correct PIN unlock..." -NoNewline
    Send-Serial -stream $serialStream -text "1234"
    Start-Sleep -Milliseconds 100
    Send-SerialByte -stream $serialStream -b 0x0D
    Start-Sleep -Milliseconds 500
    $output = Read-Serial -stream $serialStream -timeoutMs 500
    if ($output -match "\[LOCK\] Screen unlocked") {
        Write-Host " OK" -ForegroundColor Green
        Record-Result -category "LOCKSCREEN" -name "Correct PIN unlock" -status "PASS"
    } else {
        $crashed = Check-ForPanic -stream $serialStream -context "LockScreen: correct PIN"
        if (-not $crashed) {
            Write-Host " OK (no serial confirm)" -ForegroundColor Yellow
            Record-Result -category "LOCKSCREEN" -name "Correct PIN unlock" -status "PASS" -detail "no serial confirm"
        }
    }
    
    # Test 8: Empty PIN unlocks (enter with nothing)
    Write-Host "    Empty PIN unlock..." -NoNewline
    Send-KeyCombo -monStream $monStream -monWriter $monWriter -combo "meta_l-l" -holdMs 200
    Start-Sleep -Milliseconds 500
    Drain-Serial -stream $serialStream
    Send-SerialByte -stream $serialStream -b 0x0D  # Enter with empty input
    Start-Sleep -Milliseconds 500
    $output = Read-Serial -stream $serialStream -timeoutMs 500
    $crashed = Check-ForPanic -stream $serialStream -context "LockScreen: empty PIN"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    Record-Result -category "LOCKSCREEN" -name "Empty PIN unlock" -status $(if ($crashed) { "CRASH" } else { "PASS" })
    
    # Test 9: Lock+unlock cycle rapidely
    Write-Host "    Rapid lock/unlock cycle..." -NoNewline
    for ($i = 0; $i -lt 5; $i++) {
        Send-KeyCombo -monStream $monStream -monWriter $monWriter -combo "meta_l-l" -holdMs 150
        Start-Sleep -Milliseconds 300
        Send-SerialByte -stream $serialStream -b 0x0D  # Empty PIN unlock 
        Start-Sleep -Milliseconds 300
    }
    $crashed = Check-ForPanic -stream $serialStream -context "LockScreen: rapid cycle"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    Record-Result -category "LOCKSCREEN" -name "Rapid lock/unlock cycle" -status $(if ($crashed) { "CRASH" } else { "PASS" })
    
    Drain-Serial -stream $serialStream
}

function Test-FileManagerViewToggle {
    param($monStream, $monWriter, $serialStream)
    
    Write-Host "  [VIEW] Testing File Manager View Toggle..." -ForegroundColor Cyan
    Open-App-Via-StartMenu -monStream $monStream -monWriter $monWriter -serialStream $serialStream -appName "Files"
    Start-Sleep -Milliseconds 500
    Drain-Serial -stream $serialStream
    
    $crashed = $false
    
    # Test 1: V key toggles to icon grid
    Write-Host "    V key toggle to grid..." -NoNewline
    Send-SerialByte -stream $serialStream -b ([byte][char]'V')
    Start-Sleep -Milliseconds 300
    $output = Read-Serial -stream $serialStream -timeoutMs 500
    if ($output -match "\[FM\] Toggled view mode") {
        Write-Host " OK" -ForegroundColor Green
        Record-Result -category "VIEWTOGGLE" -name "V key to grid" -status "PASS"
    } else {
        $crashed = Check-ForPanic -stream $serialStream -context "ViewToggle: V to grid"
        if (-not $crashed) {
            Write-Host " OK (no serial confirm)" -ForegroundColor Yellow
            Record-Result -category "VIEWTOGGLE" -name "V key to grid" -status "PASS" -detail "no serial msg"
        }
    }
    
    # Test 2: V key toggles back to list
    Write-Host "    V key toggle to list..." -NoNewline
    Send-SerialByte -stream $serialStream -b ([byte][char]'V')
    Start-Sleep -Milliseconds 300
    $output = Read-Serial -stream $serialStream -timeoutMs 500
    $crashed = Check-ForPanic -stream $serialStream -context "ViewToggle: V to list"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    Record-Result -category "VIEWTOGGLE" -name "V key to list" -status $(if ($crashed) { "CRASH" } else { "PASS" })
    
    # Test 3: Rapid V toggle stress
    Write-Host "    Rapid V toggle x20..." -NoNewline
    for ($i = 0; $i -lt 20; $i++) {
        Send-SerialByte -stream $serialStream -b ([byte][char]'V')
        Start-Sleep -Milliseconds 30
    }
    Start-Sleep -Milliseconds 300
    $crashed = Check-ForPanic -stream $serialStream -context "ViewToggle: rapid toggle"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    Record-Result -category "VIEWTOGGLE" -name "Rapid V toggle x20" -status $(if ($crashed) { "CRASH" } else { "PASS" })
    
    # Test 4: Navigate in grid mode (arrows should still work)
    Write-Host "    Arrow nav in grid..." -NoNewline
    Send-SerialByte -stream $serialStream -b ([byte][char]'V')  # Switch to grid
    Start-Sleep -Milliseconds 100
    for ($i = 0; $i -lt 10; $i++) {
        Send-SerialByte -stream $serialStream -b 0xF0  # UP
        Start-Sleep -Milliseconds 20
        Send-SerialByte -stream $serialStream -b 0xF1  # DOWN
        Start-Sleep -Milliseconds 20
        Send-SerialByte -stream $serialStream -b 0xF2  # LEFT
        Start-Sleep -Milliseconds 20
        Send-SerialByte -stream $serialStream -b 0xF3  # RIGHT
        Start-Sleep -Milliseconds 20
    }
    $crashed = Check-ForPanic -stream $serialStream -context "ViewToggle: arrow nav grid"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    Record-Result -category "VIEWTOGGLE" -name "Arrow nav in grid" -status $(if ($crashed) { "CRASH" } else { "PASS" })
    
    # Test 5: Enter on item in grid mode
    Write-Host "    Enter in grid mode..." -NoNewline
    Send-SerialByte -stream $serialStream -b 0xF1  # DOWN to select first item
    Start-Sleep -Milliseconds 50
    Send-SerialByte -stream $serialStream -b 0x0D  # Enter
    Start-Sleep -Milliseconds 200
    Send-SerialByte -stream $serialStream -b 0x08  # Backspace to go back
    Start-Sleep -Milliseconds 200
    $crashed = Check-ForPanic -stream $serialStream -context "ViewToggle: enter in grid"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    Record-Result -category "VIEWTOGGLE" -name "Enter in grid mode" -status $(if ($crashed) { "CRASH" } else { "PASS" })
    
    Close-App -serialStream $serialStream
    Start-Sleep -Milliseconds 300
}

function Test-FileClipboard {
    param($monStream, $monWriter, $serialStream)
    
    Write-Host "  [CLIP] Testing File Manager Clipboard..." -ForegroundColor Cyan
    Open-App-Via-StartMenu -monStream $monStream -monWriter $monWriter -serialStream $serialStream -appName "Files"
    Start-Sleep -Milliseconds 500
    Drain-Serial -stream $serialStream
    
    $crashed = $false
    
    # Navigate to a file first (down arrow to select)
    Send-SerialByte -stream $serialStream -b 0xF1  # DOWN to select item
    Start-Sleep -Milliseconds 100
    
    # Test 1: Ctrl+C (copy) — serial byte 0x03
    Write-Host "    Ctrl+C copy..." -NoNewline
    Send-SerialByte -stream $serialStream -b 0x03  # Ctrl+C
    Start-Sleep -Milliseconds 300
    $output = Read-Serial -stream $serialStream -timeoutMs 500
    if ($output -match "\[FM\].*Copied|copy") {
        Write-Host " OK (serial confirmed)" -ForegroundColor Green
    } else {
        $crashed = Check-ForPanic -stream $serialStream -context "Clipboard: Ctrl+C"
        if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    }
    Record-Result -category "CLIPBOARD" -name "Ctrl+C copy" -status $(if ($crashed) { "CRASH" } else { "PASS" })
    
    # Test 2: Ctrl+V (paste) — serial byte 0x16
    Write-Host "    Ctrl+V paste..." -NoNewline
    Send-SerialByte -stream $serialStream -b 0x16  # Ctrl+V
    Start-Sleep -Milliseconds 500
    $output = Read-Serial -stream $serialStream -timeoutMs 500
    $crashed = Check-ForPanic -stream $serialStream -context "Clipboard: Ctrl+V"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    Record-Result -category "CLIPBOARD" -name "Ctrl+V paste" -status $(if ($crashed) { "CRASH" } else { "PASS" })
    
    # Test 3: Ctrl+X (cut) — serial byte 0x18
    Write-Host "    Ctrl+X cut..." -NoNewline
    Send-SerialByte -stream $serialStream -b 0xF1  # DOWN to select item
    Start-Sleep -Milliseconds 50
    Send-SerialByte -stream $serialStream -b 0x18  # Ctrl+X
    Start-Sleep -Milliseconds 300
    $output = Read-Serial -stream $serialStream -timeoutMs 500
    $crashed = Check-ForPanic -stream $serialStream -context "Clipboard: Ctrl+X"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    Record-Result -category "CLIPBOARD" -name "Ctrl+X cut" -status $(if ($crashed) { "CRASH" } else { "PASS" })
    
    # Test 4: Ctrl+V paste after cut
    Write-Host "    Ctrl+V paste after cut..." -NoNewline
    Send-SerialByte -stream $serialStream -b 0x16  # Ctrl+V
    Start-Sleep -Milliseconds 500
    $output = Read-Serial -stream $serialStream -timeoutMs 500
    $crashed = Check-ForPanic -stream $serialStream -context "Clipboard: paste after cut"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    Record-Result -category "CLIPBOARD" -name "Ctrl+V after cut" -status $(if ($crashed) { "CRASH" } else { "PASS" })
    
    # Test 5: Ctrl+V with nothing copied (empty clipboard)
    Write-Host "    Ctrl+V empty clipboard..." -NoNewline
    # Close and reopen file manager (fresh clipboard)
    Close-App -serialStream $serialStream
    Start-Sleep -Milliseconds 300
    Open-App-Via-StartMenu -monStream $monStream -monWriter $monWriter -serialStream $serialStream -appName "Files"
    Start-Sleep -Milliseconds 500
    Send-SerialByte -stream $serialStream -b 0x16  # Ctrl+V
    Start-Sleep -Milliseconds 300
    $crashed = Check-ForPanic -stream $serialStream -context "Clipboard: paste empty"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    Record-Result -category "CLIPBOARD" -name "Ctrl+V empty clipboard" -status $(if ($crashed) { "CRASH" } else { "PASS" })
    
    # Test 6: Rapid Ctrl+C/X/V cycling
    Write-Host "    Rapid C/X/V cycle..." -NoNewline
    for ($i = 0; $i -lt 10; $i++) {
        Send-SerialByte -stream $serialStream -b 0x03  # Ctrl+C
        Start-Sleep -Milliseconds 20
        Send-SerialByte -stream $serialStream -b 0x18  # Ctrl+X
        Start-Sleep -Milliseconds 20
        Send-SerialByte -stream $serialStream -b 0x16  # Ctrl+V
        Start-Sleep -Milliseconds 20
    }
    Start-Sleep -Milliseconds 300
    $crashed = Check-ForPanic -stream $serialStream -context "Clipboard: rapid cycle"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    Record-Result -category "CLIPBOARD" -name "Rapid C/X/V cycle" -status $(if ($crashed) { "CRASH" } else { "PASS" })
    
    Close-App -serialStream $serialStream
    Start-Sleep -Milliseconds 300
}

function Test-ImageViewer {
    param($monStream, $monWriter, $serialStream)
    
    Write-Host "  [IMG] Testing Image Viewer..." -ForegroundColor Cyan
    
    $crashed = $false
    
    # First, create a minimal BMP file via the shell (we need to exit desktop briefly)
    # Actually, we can test by trying to open a .bmp file from file manager
    # The image viewer opens when you select a .bmp file and press Enter
    
    # Open file manager and navigate
    Open-App-Via-StartMenu -monStream $monStream -monWriter $monWriter -serialStream $serialStream -appName "Files"
    Start-Sleep -Milliseconds 500
    
    # Navigate to find any file and open it (the viewer type depends on extension)
    # Since we may not have .bmp files, test that the image viewer doesn't crash on
    # zoom/pan keys even when sent to the current focused window
    
    # Test 1: Open file manager, navigate, try opening files
    Write-Host "    Navigate and Enter..." -NoNewline
    for ($i = 0; $i -lt 5; $i++) {
        Send-SerialByte -stream $serialStream -b 0xF1  # DOWN
        Start-Sleep -Milliseconds 50
    }
    Send-SerialByte -stream $serialStream -b 0x0D  # Enter (open selected)
    Start-Sleep -Milliseconds 500
    $crashed = Check-ForPanic -stream $serialStream -context "ImageViewer: open file"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    Record-Result -category "IMAGEVIEWER" -name "Open file from FM" -status $(if ($crashed) { "CRASH" } else { "PASS" })
    
    # Whether or not an image viewer opened, send image viewer keys
    # These should be safe even if the focused window is not an image viewer
    
    # Test 2: Zoom in (+)
    Write-Host "    Zoom in (+)..." -NoNewline
    for ($i = 0; $i -lt 10; $i++) {
        Send-SerialByte -stream $serialStream -b ([byte][char]'+')
        Start-Sleep -Milliseconds 30
    }
    $crashed = Check-ForPanic -stream $serialStream -context "ImageViewer: zoom in"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    Record-Result -category "IMAGEVIEWER" -name "Zoom in (+)" -status $(if ($crashed) { "CRASH" } else { "PASS" })
    
    # Test 3: Zoom out (-)
    Write-Host "    Zoom out (-)..." -NoNewline
    for ($i = 0; $i -lt 20; $i++) {
        Send-SerialByte -stream $serialStream -b ([byte][char]'-')
        Start-Sleep -Milliseconds 30
    }
    $crashed = Check-ForPanic -stream $serialStream -context "ImageViewer: zoom out"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    Record-Result -category "IMAGEVIEWER" -name "Zoom out (-)" -status $(if ($crashed) { "CRASH" } else { "PASS" })
    
    # Test 4: Pan with arrow keys
    Write-Host "    Pan arrows..." -NoNewline
    for ($i = 0; $i -lt 10; $i++) {
        Send-SerialByte -stream $serialStream -b 0xF0  # UP
        Start-Sleep -Milliseconds 20
        Send-SerialByte -stream $serialStream -b 0xF1  # DOWN
        Start-Sleep -Milliseconds 20
        Send-SerialByte -stream $serialStream -b 0xF2  # LEFT
        Start-Sleep -Milliseconds 20
        Send-SerialByte -stream $serialStream -b 0xF3  # RIGHT
        Start-Sleep -Milliseconds 20
    }
    $crashed = Check-ForPanic -stream $serialStream -context "ImageViewer: pan arrows"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    Record-Result -category "IMAGEVIEWER" -name "Pan with arrows" -status $(if ($crashed) { "CRASH" } else { "PASS" })
    
    # Test 5: Reset view (0 key)
    Write-Host "    Reset view (0)..." -NoNewline
    Send-SerialByte -stream $serialStream -b ([byte][char]'0')
    Start-Sleep -Milliseconds 200
    $crashed = Check-ForPanic -stream $serialStream -context "ImageViewer: reset"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    Record-Result -category "IMAGEVIEWER" -name "Reset view (0)" -status $(if ($crashed) { "CRASH" } else { "PASS" })
    
    # Test 6: Extreme zoom in (way past max)
    Write-Host "    Extreme zoom in..." -NoNewline
    for ($i = 0; $i -lt 50; $i++) {
        Send-SerialByte -stream $serialStream -b ([byte][char]'+')
        Start-Sleep -Milliseconds 10
    }
    $crashed = Check-ForPanic -stream $serialStream -context "ImageViewer: extreme zoom in"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    Record-Result -category "IMAGEVIEWER" -name "Extreme zoom in" -status $(if ($crashed) { "CRASH" } else { "PASS" })
    
    # Test 7: Extreme zoom out (way past min)
    Write-Host "    Extreme zoom out..." -NoNewline
    for ($i = 0; $i -lt 100; $i++) {
        Send-SerialByte -stream $serialStream -b ([byte][char]'-')
        Start-Sleep -Milliseconds 10
    }
    $crashed = Check-ForPanic -stream $serialStream -context "ImageViewer: extreme zoom out"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    Record-Result -category "IMAGEVIEWER" -name "Extreme zoom out" -status $(if ($crashed) { "CRASH" } else { "PASS" })
    
    # Test 8: Rapid zoom toggle
    Write-Host "    Rapid zoom toggle..." -NoNewline
    for ($i = 0; $i -lt 30; $i++) {
        Send-SerialByte -stream $serialStream -b ([byte][char]'+')
        Send-SerialByte -stream $serialStream -b ([byte][char]'-')
        Start-Sleep -Milliseconds 5
    }
    $crashed = Check-ForPanic -stream $serialStream -context "ImageViewer: rapid zoom"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    Record-Result -category "IMAGEVIEWER" -name "Rapid zoom toggle" -status $(if ($crashed) { "CRASH" } else { "PASS" })
    
    # Test 9: All printable ASCII
    Write-Host "    All printable ASCII..." -NoNewline
    for ($b = 0x20; $b -le 0x7E; $b++) {
        Send-SerialByte -stream $serialStream -b ([byte]$b)
        Start-Sleep -Milliseconds 5
    }
    $crashed = Check-ForPanic -stream $serialStream -context "ImageViewer: all ASCII"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    Record-Result -category "IMAGEVIEWER" -name "All printable ASCII" -status $(if ($crashed) { "CRASH" } else { "PASS" })
    
    Close-App -serialStream $serialStream
    Start-Sleep -Milliseconds 300
}

function Test-SysTray {
    param($monStream, $monWriter, $serialStream)
    
    Write-Host "  [TRAY] Testing System Tray Indicators..." -ForegroundColor Cyan
    
    $crashed = $false
    
    # The system tray is drawn every frame as part of the taskbar. 
    # We can't directly interact with it via serial, but we verify
    # that the desktop doesn't crash after our features were added.
    # Test: open/close apps quickly to force redraws with the tray visible.
    
    # Test 1: Rapid window open/close cycle (forces tray redraw)
    Write-Host "    Rapid open/close cycle..." -NoNewline
    for ($i = 0; $i -lt 3; $i++) {
        Send-KeyCombo -monStream $monStream -monWriter $monWriter -combo "meta_l-e" -holdMs 150
        Start-Sleep -Milliseconds 400
        Send-SerialByte -stream $serialStream -b 0x1B  # ESC close
        Start-Sleep -Milliseconds 300
    }
    $crashed = Check-ForPanic -stream $serialStream -context "SysTray: rapid open/close"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    Record-Result -category "SYSTRAY" -name "Rapid open/close cycle" -status $(if ($crashed) { "CRASH" } else { "PASS" })
    
    # Test 2: Multiple windows open simultaneously (tray must render with each)
    Write-Host "    Multi-window tray..." -NoNewline
    Send-KeyCombo -monStream $monStream -monWriter $monWriter -combo "meta_l-e" -holdMs 150
    Start-Sleep -Milliseconds 300
    Send-KeyCombo -monStream $monStream -monWriter $monWriter -combo "meta_l-i" -holdMs 150
    Start-Sleep -Milliseconds 300
    # Open calculator via start menu
    Open-App-Via-StartMenu -monStream $monStream -monWriter $monWriter -serialStream $serialStream -appName "Calculator"
    Start-Sleep -Milliseconds 300
    $crashed = Check-ForPanic -stream $serialStream -context "SysTray: multi-window"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    Record-Result -category "SYSTRAY" -name "Multi-window tray" -status $(if ($crashed) { "CRASH" } else { "PASS" })
    
    # Close all windows
    for ($i = 0; $i -lt 5; $i++) {
        Send-SerialByte -stream $serialStream -b 0x1B
        Start-Sleep -Milliseconds 200
    }
    
    # Test 3: Alt+Tab with tray visible
    Write-Host "    Alt+Tab with tray..." -NoNewline
    Send-KeyCombo -monStream $monStream -monWriter $monWriter -combo "meta_l-e" -holdMs 150
    Start-Sleep -Milliseconds 300
    Send-KeyCombo -monStream $monStream -monWriter $monWriter -combo "meta_l-i" -holdMs 150
    Start-Sleep -Milliseconds 300
    for ($i = 0; $i -lt 5; $i++) {
        Send-KeyCombo -monStream $monStream -monWriter $monWriter -combo "alt-tab" -holdMs 150
        Start-Sleep -Milliseconds 200
    }
    $crashed = Check-ForPanic -stream $serialStream -context "SysTray: alt-tab"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    Record-Result -category "SYSTRAY" -name "Alt+Tab with tray" -status $(if ($crashed) { "CRASH" } else { "PASS" })
    
    # Close everything
    for ($i = 0; $i -lt 5; $i++) {
        Send-SerialByte -stream $serialStream -b 0x1B
        Start-Sleep -Milliseconds 200
    }
    
    Drain-Serial -stream $serialStream
}

function Test-DragDrop {
    param($monStream, $monWriter, $serialStream)
    
    Write-Host "  [DRAG] Testing Drag-and-Drop (keyboard-driven)..." -ForegroundColor Cyan
    
    $crashed = $false
    
    # Drag-and-drop is mouse-driven, so we can't fully test it via serial.
    # However, we verify that the drag state code paths don't crash when:
    # 1) Multiple file managers are open
    # 2) Keys are pressed during potential drag states
    
    # Test 1: Open two file managers side by side
    Write-Host "    Two file managers..." -NoNewline
    Send-KeyCombo -monStream $monStream -monWriter $monWriter -combo "meta_l-e" -holdMs 150
    Start-Sleep -Milliseconds 400
    Send-KeyCombo -monStream $monStream -monWriter $monWriter -combo "meta_l-e" -holdMs 150
    Start-Sleep -Milliseconds 400
    $crashed = Check-ForPanic -stream $serialStream -context "DragDrop: two FMs"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    Record-Result -category "DRAGDROP" -name "Two file managers open" -status $(if ($crashed) { "CRASH" } else { "PASS" })
    
    # Test 2: Rapid Alt+Tab between two FMs
    Write-Host "    Alt+Tab between FMs..." -NoNewline
    for ($i = 0; $i -lt 10; $i++) {
        Send-KeyCombo -monStream $monStream -monWriter $monWriter -combo "alt-tab" -holdMs 100
        Start-Sleep -Milliseconds 200
    }
    $crashed = Check-ForPanic -stream $serialStream -context "DragDrop: alt-tab FMs"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    Record-Result -category "DRAGDROP" -name "Alt+Tab between FMs" -status $(if ($crashed) { "CRASH" } else { "PASS" })
    
    # Test 3: Navigate and clipboard ops across two FMs
    Write-Host "    Cross-FM clipboard..." -NoNewline
    Send-SerialByte -stream $serialStream -b 0xF1  # DOWN to select
    Start-Sleep -Milliseconds 50
    Send-SerialByte -stream $serialStream -b 0x03  # Ctrl+C
    Start-Sleep -Milliseconds 100
    Send-KeyCombo -monStream $monStream -monWriter $monWriter -combo "alt-tab" -holdMs 100
    Start-Sleep -Milliseconds 200
    Send-SerialByte -stream $serialStream -b 0x16  # Ctrl+V
    Start-Sleep -Milliseconds 300
    $crashed = Check-ForPanic -stream $serialStream -context "DragDrop: cross-FM clipboard"
    if (-not $crashed) { Write-Host " OK" -ForegroundColor Green }
    Record-Result -category "DRAGDROP" -name "Cross-FM clipboard" -status $(if ($crashed) { "CRASH" } else { "PASS" })
    
    # Close all
    for ($i = 0; $i -lt 5; $i++) {
        Send-SerialByte -stream $serialStream -b 0x1B
        Start-Sleep -Milliseconds 200
    }
    
    Drain-Serial -stream $serialStream
}

# ---------------------------------------------------------------
#  MAIN
# ---------------------------------------------------------------

Write-Host ""
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "  TrustOS Desktop Input Stress Test" -ForegroundColor Cyan
Write-Host ("  {0}" -f $timestamp) -ForegroundColor DarkCyan
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host ""

# Pre-flight
if (-not (Test-Path $QemuExe)) {
    Write-Host "FATAL: QEMU not found at $QemuExe" -ForegroundColor Red
    exit 1
}
if (-not (Test-Path $IsoPath)) {
    Write-Host "FATAL: ISO not found at $IsoPath" -ForegroundColor Red
    exit 1
}

Write-Host ("  ISO:     {0}" -f $IsoPath) -ForegroundColor DarkGray
Write-Host ("  Serial:  TCP {0}" -f $SerialPort) -ForegroundColor DarkGray
Write-Host ("  Monitor: TCP {0}" -f $MonitorPort) -ForegroundColor DarkGray
Write-Host ""

# Kill existing QEMU
$existingQemu = Get-Process -Name "qemu-system-x86_64" -ErrorAction SilentlyContinue
if ($existingQemu) {
    Write-Host "  Killing existing QEMU..." -ForegroundColor Yellow
    $existingQemu | Stop-Process -Force
    Start-Sleep -Seconds 2
}

# Create data disk if needed
$dataImg = "$PSScriptRoot\trustos_data.img"
if (-not (Test-Path $dataImg)) {
    Write-Host "  Creating data disk..." -ForegroundColor Yellow
    $qemuImg = Join-Path (Split-Path $QemuExe) "qemu-img.exe"
    if (Test-Path $qemuImg) {
        & $qemuImg create -f raw $dataImg 64M 2>&1 | Out-Null
    } else {
        $fs = [System.IO.File]::Create($dataImg)
        $fs.SetLength(64 * 1024 * 1024)
        $fs.Close()
    }
}

# Launch QEMU
Write-Host "[1/5] Starting QEMU..." -ForegroundColor White
$serialArg = "tcp:127.0.0.1:${SerialPort},server,nowait"
$monitorArg = "tcp:127.0.0.1:${MonitorPort},server,nowait"
$qemuArgs = @(
    "-cdrom", "`"$IsoPath`"",
    "-m", "256M",
    "-machine", "q35",
    "-cpu", "max",
    "-smp", "2",
    "-accel", "tcg,thread=multi",
    "-display", "gtk",
    "-vga", "std",
    "-device", "virtio-net-pci,netdev=net0",
    "-netdev", "user,id=net0",
    "-device", "intel-hda",
    "-device", "hda-duplex",
    "-device", "qemu-xhci,id=xhci",
    "-device", "usb-kbd,bus=xhci.0",
    "-device", "usb-mouse,bus=xhci.0",
    "-serial", $serialArg,
    "-monitor", $monitorArg,
    "-no-reboot"
)

$qemuProcess = Start-Process -FilePath $QemuExe -ArgumentList $qemuArgs -PassThru
Write-Host ("  PID: {0}" -f $qemuProcess.Id) -ForegroundColor DarkGray

# Connect serial TCP
Write-Host "[2/5] Connecting serial TCP..." -ForegroundColor White
$serialClient = New-Object System.Net.Sockets.TcpClient
$connected = $false
for ($i = 0; $i -lt 60; $i++) {
    try {
        $serialClient.Connect("127.0.0.1", $SerialPort)
        $connected = $true
        break
    } catch {
        Start-Sleep -Milliseconds 300
    }
}
if (-not $connected) {
    Write-Host "FATAL: Could not connect to serial TCP $SerialPort" -ForegroundColor Red
    Stop-Process -Id $qemuProcess.Id -Force -ErrorAction SilentlyContinue
    exit 1
}
$serialStream = $serialClient.GetStream()
$serialStream.ReadTimeout = 3000
Write-Host "  Serial connected!" -ForegroundColor Green

# Connect QEMU monitor TCP
Write-Host "[3/5] Connecting QEMU monitor..." -ForegroundColor White
$monClient = New-Object System.Net.Sockets.TcpClient
$monConnected = $false
for ($i = 0; $i -lt 30; $i++) {
    try {
        $monClient.Connect("127.0.0.1", $MonitorPort)
        $monConnected = $true
        break
    } catch {
        Start-Sleep -Milliseconds 300
    }
}
if (-not $monConnected) {
    Write-Host "WARNING: Could not connect to QEMU monitor (sendkey unavailable)" -ForegroundColor Yellow
    $monStream = $null
    $monWriter = $null
} else {
    $monStream = $monClient.GetStream()
    $monStream.ReadTimeout = 2000
    $monWriter = New-Object System.IO.StreamWriter($monStream)
    $monWriter.AutoFlush = $true
    # Read initial banner
    Start-Sleep -Milliseconds 500
    $buffer = New-Object byte[] 4096
    if ($monStream.DataAvailable) {
        $monStream.Read($buffer, 0, $buffer.Length) | Out-Null
    }
    Write-Host "  Monitor connected!" -ForegroundColor Green
}

# Wait for boot
Write-Host "[4/5] Waiting for TrustOS boot..." -ForegroundColor White
$buffer = New-Object byte[] 16384
$sw = [System.Diagnostics.Stopwatch]::StartNew()
$bootText = ""
$booted = $false

while ($sw.Elapsed.TotalSeconds -lt $BootTimeout) {
    if ($serialStream.DataAvailable) {
        $read = $serialStream.Read($buffer, 0, $buffer.Length)
        if ($read -gt 0) {
            $text = [System.Text.Encoding]::ASCII.GetString($buffer, 0, $read)
            $bootText += $text
            if ($bootText -match "trustos.*[\$#]") {
                $booted = $true
                break
            }
        }
    } else {
        Start-Sleep -Milliseconds 150
    }
}

if (-not $booted) {
    Write-Host ("FATAL: Boot timed out after {0}s" -f $BootTimeout) -ForegroundColor Red
    try { $serialClient.Close() } catch {}
    try { $monClient.Close() } catch {}
    Stop-Process -Id $qemuProcess.Id -Force -ErrorAction SilentlyContinue
    exit 1
}
$bootTime = [math]::Round($sw.Elapsed.TotalSeconds, 1)
Write-Host ("  Booted in {0}s" -f $bootTime) -ForegroundColor Green

# Stabilize
Start-Sleep -Milliseconds 500
Drain-Serial -stream $serialStream

# Enter desktop mode
Write-Host "[5/5] Entering desktop mode..." -ForegroundColor White
Send-Serial -stream $serialStream -text "desktop`r"
Start-Sleep -Milliseconds 3000

# Wait for desktop init
$desktopText = Read-Serial -stream $serialStream -timeoutMs 5000
if ($desktopText -match "\[GUI\] Starting desktop") {
    Write-Host "  Desktop started!" -ForegroundColor Green
} else {
    Write-Host "  Desktop may have started (no confirm msg detected)" -ForegroundColor Yellow
}
Drain-Serial -stream $serialStream

# ---------------------------------------------------------------
#  RUN DESKTOP TESTS
# ---------------------------------------------------------------

Write-Host ""
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "  Running Desktop Input Tests" -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host ""

$testStart = [System.Diagnostics.Stopwatch]::StartNew()

# Test global hotkeys first (no app needed)
Test-GlobalHotkeys -monStream $monStream -monWriter $monWriter -serialStream $serialStream

# Test start menu
Test-StartMenu -monStream $monStream -monWriter $monWriter -serialStream $serialStream

# Test each app
Test-Calculator -monStream $monStream -monWriter $monWriter -serialStream $serialStream
Test-Terminal -monStream $monStream -monWriter $monWriter -serialStream $serialStream
Test-TextEditor -monStream $monStream -monWriter $monWriter -serialStream $serialStream
Test-Snake -monStream $monStream -monWriter $monWriter -serialStream $serialStream
Test-FileManager -monStream $monStream -monWriter $monWriter -serialStream $serialStream
Test-Chess -monStream $monStream -monWriter $monWriter -serialStream $serialStream
Test-Browser -monStream $monStream -monWriter $monWriter -serialStream $serialStream
Test-Settings -monStream $monStream -monWriter $monWriter -serialStream $serialStream

# Test v0.5.0 desktop enhancements
Test-LockScreen -monStream $monStream -monWriter $monWriter -serialStream $serialStream
Test-FileManagerViewToggle -monStream $monStream -monWriter $monWriter -serialStream $serialStream
Test-FileClipboard -monStream $monStream -monWriter $monWriter -serialStream $serialStream
Test-ImageViewer -monStream $monStream -monWriter $monWriter -serialStream $serialStream
Test-SysTray -monStream $monStream -monWriter $monWriter -serialStream $serialStream
Test-DragDrop -monStream $monStream -monWriter $monWriter -serialStream $serialStream

# Raw byte fuzz (last - most aggressive)
Test-RawByteFuzz -monStream $monStream -monWriter $monWriter -serialStream $serialStream

$testDuration = [math]::Round($testStart.Elapsed.TotalSeconds, 1)

# ---------------------------------------------------------------
#  CLEANUP
# ---------------------------------------------------------------

Write-Host ""
Write-Host "Exiting desktop and shutting down..." -ForegroundColor DarkGray

# Exit desktop (ESC with no windows focused)
Send-SerialByte -stream $serialStream -b 0x1B
Start-Sleep -Milliseconds 500
Send-SerialByte -stream $serialStream -b 0x1B
Start-Sleep -Milliseconds 500

try { $serialClient.Close() } catch {}
try { $monClient.Close() } catch {}
Start-Sleep -Seconds 1
Stop-Process -Id $qemuProcess.Id -Force -ErrorAction SilentlyContinue

# ---------------------------------------------------------------
#  REPORT
# ---------------------------------------------------------------

$total = $global:passed + $global:failed + $global:crashed
$passRate = if ($total -gt 0) { [math]::Round(($global:passed / $total) * 100, 1) } else { 0 }

Write-Host ""
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "  DESKTOP TEST RESULTS" -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host ("  Total:    {0}" -f $total) -ForegroundColor White
Write-Host ("  Passed:   {0}" -f $global:passed) -ForegroundColor Green
Write-Host ("  Failed:   {0}" -f $global:failed) -ForegroundColor $(if ($global:failed -gt 0) { "Red" } else { "Green" })
Write-Host ("  Crashed:  {0}" -f $global:crashed) -ForegroundColor $(if ($global:crashed -gt 0) { "Red" } else { "Green" })
Write-Host ("  Rate:     {0}%%" -f $passRate) -ForegroundColor $(if ($passRate -ge 80) { "Green" } elseif ($passRate -ge 50) { "Yellow" } else { "Red" })
Write-Host ("  Duration: {0}s (+ {1}s boot)" -f $testDuration, $bootTime) -ForegroundColor DarkGray
Write-Host "============================================================" -ForegroundColor Cyan

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
$failures = $global:results | Where-Object { $_.Status -ne "PASS" -and $_.Status -ne "SKIP" }
if ($failures.Count -gt 0) {
    Write-Host ""
    Write-Host "=== FAILURES ===" -ForegroundColor Red
    foreach ($f in $failures) {
        Write-Host ("  [{0}] {1}: {2}" -f $f.Category, $f.Name, $f.Detail) -ForegroundColor Red
    }
}

# Show skipped
$skipped = $global:results | Where-Object { $_.Status -eq "SKIP" }
if ($skipped.Count -gt 0) {
    Write-Host ""
    Write-Host "=== SKIPPED ===" -ForegroundColor Yellow
    foreach ($s in $skipped) {
        Write-Host ("  [{0}] {1}: {2}" -f $s.Category, $s.Name, $s.Detail) -ForegroundColor Yellow
    }
}

# Category breakdown
Write-Host ""
Write-Host "=== BY CATEGORY ===" -ForegroundColor Cyan
$categories = $global:results | Group-Object -Property Category
foreach ($cat in $categories | Sort-Object Name) {
    $catPassed = @($cat.Group | Where-Object { $_.Status -eq "PASS" }).Count
    $catTotal = $cat.Group.Count
    $catColor = if ($catPassed -eq $catTotal) { "Green" } elseif ($catPassed -gt 0) { "Yellow" } else { "Red" }
    Write-Host ("  {0}  {1}/{2}" -f $cat.Name.PadRight(14), $catPassed, $catTotal) -ForegroundColor $catColor
}

# ---------------------------------------------------------------
#  WRITE REPORT FILE
# ---------------------------------------------------------------

$reportLines = @()
$reportLines += "=================================================================="
$reportLines += "  TrustOS Desktop Input Stress Test Report"
$reportLines += ("  Generated: {0}" -f $timestamp)
$reportLines += ("  Boot: {0}s  |  Test: {1}s" -f $bootTime, $testDuration)
$reportLines += "=================================================================="
$reportLines += ""
$reportLines += "SUMMARY"
$reportLines += ("  Passed:  {0}" -f $global:passed)
$reportLines += ("  Failed:  {0}" -f $global:failed)
$reportLines += ("  Crashed: {0}" -f $global:crashed)
$reportLines += ("  Rate:    {0}%%" -f $passRate)
$reportLines += ""

if ($global:panics.Count -gt 0) {
    $reportLines += "CRASHES"
    foreach ($p in $global:panics) {
        $reportLines += ("  [{0}]" -f $p.Context)
        $reportLines += ("  {0}" -f $p.Output)
        $reportLines += ""
    }
}

$reportLines += "DETAILED RESULTS"
foreach ($r in $global:results) {
    $reportLines += ("  [{0}] {1}: {2} {3}" -f $r.Category, $r.Name, $r.Status, $r.Detail)
}

$reportLines += ""
$reportLines += "=================================================================="
$reportLines += ""
$reportLines += "TEST COVERAGE"
$reportLines += "  Calculator:   digits, operators, div/0, parens, overflow, backspace, scientific, all ASCII"
$reportLines += "  Terminal:     empty enter, backspace, long input, arrows, ctrl chars, nav keys, scroll, all ASCII"
$reportLines += "  TrustCode:    type, backspace, enter, arrows, nav keys, delete, tab, all ASCII, undo, redo"
$reportLines += "  Snake:        WASD, reversals, pause, space, difficulty, all ASCII"
$reportLines += "  FileManager:  arrows, enter, delete, boundary scroll, all ASCII, tab"
$reportLines += "  Chess:        board nav, select/deselect, self-move, difficulty, new game, pawn+AI, all ASCII"
$reportLines += "  Browser:      URL, backspace, empty enter, scroll, all ASCII"
$reportLines += "  Settings:     option keys 1-9, all ASCII"
$reportLines += "  StartMenu:    toggle, search fuzz, backspace, arrows, enter empty"
$reportLines += "  Hotkeys:      Win+E/D/I/H, Win+Arrows, Alt+Tab, Alt+F4"
$reportLines += "  RawFuzz:      all 256 byte values, rapid nav keys"
$reportLines += "  LockScreen:   Win+L activate, wrong PIN, backspace, all ASCII, enter spam, long PIN, correct PIN, empty PIN, rapid cycle"
$reportLines += "  ViewToggle:   V key grid/list, rapid toggle, arrow nav in grid, enter in grid"
$reportLines += "  Clipboard:    Ctrl+C copy, Ctrl+V paste, Ctrl+X cut, paste-after-cut, empty clipboard, rapid C/X/V"
$reportLines += "  ImageViewer:  open file, zoom +/-, pan arrows, reset, extreme zoom, rapid zoom, all ASCII"
$reportLines += "  SysTray:      rapid open/close, multi-window, Alt+Tab with tray"
$reportLines += "  DragDrop:     two FMs, Alt+Tab between, cross-FM clipboard"
$reportLines += ""
$reportLines += "APPS NOT TESTED (require mouse interaction to open)"
$reportLines += "  Game3D (TrustDoom), HexViewer, NES Emulator, GameBoy, TrustEdit 3D, GameLab"
$reportLines += ""
$reportLines += "=================================================================="

$reportLines -join "`r`n" | Out-File -FilePath $ReportFile -Encoding UTF8
Write-Host ""
Write-Host ("Report saved to: {0}" -f $ReportFile) -ForegroundColor White
Write-Host ""
