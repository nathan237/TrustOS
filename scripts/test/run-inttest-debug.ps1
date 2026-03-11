# TrustOS - Run inttest with serial debug capture
param(
    [int]$SerialPort = 5555,
    [int]$BootTimeout = 45,
    [int]$CmdTimeout = 20
)

$ErrorActionPreference = "Continue"
$QemuExe = "C:\Program Files\qemu\qemu-system-x86_64.exe"
$IsoPath = "trustos.iso"

Write-Host "`n============================================" -ForegroundColor Cyan
Write-Host "  TrustOS Integration Test Debug Runner" -ForegroundColor Cyan
Write-Host "============================================`n" -ForegroundColor Cyan

# Pre-flight
if (-not (Test-Path $IsoPath)) {
    Write-Host "FATAL: ISO not found: $IsoPath" -ForegroundColor Red
    exit 1
}

# Kill existing QEMU
Get-Process -Name "qemu-system-x86_64" -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 1

# Launch QEMU
Write-Host "[1/5] Starting QEMU..." -ForegroundColor Yellow
$serialArg = "tcp:127.0.0.1:${SerialPort},server,nowait"
$qemuArgs = @(
    "-cdrom", "`"$IsoPath`"",
    "-m", "256M",
    "-machine", "q35",
    "-cpu", "max",
    "-smp", "2",
    "-accel", "tcg,thread=multi",
    "-display", "gtk",
    "-vga", "std",
    "-serial", $serialArg,
    "-no-reboot"
)
$qemuProcess = Start-Process -FilePath $QemuExe -ArgumentList $qemuArgs -PassThru
Write-Host "  PID: $($qemuProcess.Id)" -ForegroundColor DarkGray
Start-Sleep -Seconds 2

# Connect serial TCP
Write-Host "[2/5] Connecting serial TCP on port $SerialPort..." -ForegroundColor Yellow
$client = New-Object System.Net.Sockets.TcpClient
$connected = $false
for ($i = 0; $i -lt 40; $i++) {
    try {
        $client.Connect("127.0.0.1", $SerialPort)
        $connected = $true
        break
    } catch {
        Start-Sleep -Milliseconds 500
    }
}
if (-not $connected) {
    Write-Host "FATAL: Could not connect to serial TCP" -ForegroundColor Red
    Stop-Process -Id $qemuProcess.Id -Force -ErrorAction SilentlyContinue
    exit 1
}
$stream = $client.GetStream()
$stream.ReadTimeout = 3000
$writer = New-Object System.IO.StreamWriter($stream)
$writer.AutoFlush = $true
Write-Host "  Connected!" -ForegroundColor Green

# Wait for boot (shell prompt)
Write-Host "[3/5] Waiting for TrustOS shell..." -ForegroundColor Yellow
$buffer = New-Object byte[] 65536
$bootText = ""
$booted = $false
$sw = [System.Diagnostics.Stopwatch]::StartNew()

while ($sw.Elapsed.TotalSeconds -lt $BootTimeout) {
    if ($stream.DataAvailable) {
        $read = $stream.Read($buffer, 0, $buffer.Length)
        if ($read -gt 0) {
            $text = [System.Text.Encoding]::ASCII.GetString($buffer, 0, $read)
            $bootText += $text
            # Look for shell prompt
            if ($bootText -match "trustos.*[\$#>]" -or $bootText -match ">\s*$") {
                $booted = $true
                break
            }
        }
    } else {
        Start-Sleep -Milliseconds 200
    }
}

if (-not $booted) {
    Write-Host "FATAL: Boot timed out after ${BootTimeout}s" -ForegroundColor Red
    Write-Host "`n--- Boot output (last 1000 chars) ---" -ForegroundColor DarkGray
    $showLen = [Math]::Min(1000, $bootText.Length)
    if ($showLen -gt 0) {
        Write-Host $bootText.Substring([Math]::Max(0, $bootText.Length - $showLen)) -ForegroundColor DarkGray
    } else {
        Write-Host "(no output received)" -ForegroundColor DarkGray
    }

    # Save full boot log
    $bootText | Out-File -FilePath "serial_inttest_debug.txt" -Encoding UTF8
    Write-Host "Full boot log saved to serial_inttest_debug.txt" -ForegroundColor DarkGray

    try { $client.Close() } catch {}
    Stop-Process -Id $qemuProcess.Id -Force -ErrorAction SilentlyContinue
    exit 1
}

$bootTime = [math]::Round($sw.Elapsed.TotalSeconds, 1)
Write-Host "  Booted in ${bootTime}s" -ForegroundColor Green

# Save boot log
Write-Host "`n--- Boot log (last 500 chars) ---" -ForegroundColor DarkGray
$showLen = [Math]::Min(500, $bootText.Length)
if ($showLen -gt 0) {
    Write-Host $bootText.Substring([Math]::Max(0, $bootText.Length - $showLen)) -ForegroundColor DarkGray
}

# Stabilize: send empty line, drain
Start-Sleep -Milliseconds 500
$writer.Write("`r")
$writer.Flush()
Start-Sleep -Milliseconds 500
while ($stream.DataAvailable) {
    $stream.Read($buffer, 0, $buffer.Length) | Out-Null
}

# Send inttest command
Write-Host "`n[4/5] Sending 'inttest' command..." -ForegroundColor Yellow
$writer.Write("inttest`r")
$writer.Flush()

# Capture output until we see the result or timeout
$cmdOutput = ""
$sw2 = [System.Diagnostics.Stopwatch]::StartNew()
$done = $false

while ($sw2.Elapsed.TotalSeconds -lt $CmdTimeout) {
    if ($stream.DataAvailable) {
        $read = $stream.Read($buffer, 0, $buffer.Length)
        if ($read -gt 0) {
            $text = [System.Text.Encoding]::ASCII.GetString($buffer, 0, $read)
            $cmdOutput += $text
            # Check for completion markers
            if ($cmdOutput -match "TESTS PASSED" -or $cmdOutput -match "FAILED ===" -or $cmdOutput -match "trustos.*[\$#>].*$") {
                # Give a moment for any trailing output
                Start-Sleep -Milliseconds 500
                while ($stream.DataAvailable) {
                    $read = $stream.Read($buffer, 0, $buffer.Length)
                    $text = [System.Text.Encoding]::ASCII.GetString($buffer, 0, $read)
                    $cmdOutput += $text
                }
                $done = $true
                break
            }
        }
    } else {
        Start-Sleep -Milliseconds 200
    }
}

$cmdTime = [math]::Round($sw2.Elapsed.TotalSeconds, 1)

# Display results
Write-Host "`n[5/5] Results (${cmdTime}s):" -ForegroundColor Yellow
Write-Host "============================================" -ForegroundColor Cyan

# Clean ANSI codes
$cleanOutput = $cmdOutput -replace '\x1b\[[0-9;]*[mGKHJ]', '' -replace '\x1b\[\?[0-9]*[hl]', ''
Write-Host $cleanOutput

Write-Host "============================================" -ForegroundColor Cyan

# Save full output
$fullLog = "=== BOOT LOG ===`r`n$bootText`r`n`r`n=== INTTEST OUTPUT ===`r`n$cmdOutput`r`n`r`n=== CLEAN OUTPUT ===`r`n$cleanOutput"
$fullLog | Out-File -FilePath "serial_inttest_debug.txt" -Encoding UTF8
Write-Host "`nFull log saved to: serial_inttest_debug.txt" -ForegroundColor Green

# Summary
if ($cmdOutput -match "ALL.*TESTS PASSED") {
    Write-Host "`n  >>> ALL TESTS PASSED <<<" -ForegroundColor Green
} elseif ($cmdOutput -match "(\d+)/(\d+) passed.*?(\d+) FAILED") {
    Write-Host "`n  >>> SOME TESTS FAILED <<<" -ForegroundColor Red
} elseif (-not $done) {
    Write-Host "`n  >>> TIMEOUT - command did not complete in ${CmdTimeout}s <<<" -ForegroundColor Red
} else {
    Write-Host "`n  >>> UNKNOWN RESULT - check output above <<<" -ForegroundColor Yellow
}

# Cleanup
try { $client.Close() } catch {}
Start-Sleep -Seconds 1
Stop-Process -Id $qemuProcess.Id -Force -ErrorAction SilentlyContinue
Write-Host "`nQEMU terminated." -ForegroundColor DarkGray
