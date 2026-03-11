# Full pretrain + eval + chat test — STREAMING output
# Usage: powershell -File pretrain-full.ps1 [epochs]
param([int]$Epochs = 5)

Set-Location $PSScriptRoot

$QemuExe = "C:\Program Files\qemu\qemu-system-x86_64.exe"
$SerialPort = 5556
$OutputFile = "$PSScriptRoot\pretrain_full_output.txt"

# Kill any existing QEMU
Get-Process -Name "qemu-system-x86_64" -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 2

Write-Host "=== BUILDING ===" -ForegroundColor Cyan
& powershell -File build-limine.ps1 -NoRun 2>&1 | Out-Null
if (-not (Test-Path "trustos.iso")) { Write-Host "BUILD FAILED" -ForegroundColor Red; exit 1 }
Write-Host "ISO ready."

Write-Host "=== LAUNCHING QEMU ===" -ForegroundColor Cyan
$serialArg = "tcp:127.0.0.1:${SerialPort},server,nowait"
$qemuArgs = @(
    "-cdrom", "`"$PSScriptRoot\trustos.iso`"",
    "-m", "512M", "-machine", "q35", "-cpu", "max", "-smp", "2",
    "-accel", "tcg,thread=multi", "-display", "none", "-vga", "std",
    "-device", "virtio-net-pci,netdev=net0", "-netdev", "user,id=net0",
    "-serial", $serialArg, "-no-reboot"
)
$qemu = Start-Process -FilePath $QemuExe -ArgumentList $qemuArgs -PassThru
Write-Host "QEMU PID: $($qemu.Id)"

# Connect to serial
$client = New-Object System.Net.Sockets.TcpClient
for ($i = 0; $i -lt 60; $i++) {
    try { $client.Connect("127.0.0.1", $SerialPort); break } catch { Start-Sleep -Milliseconds 300 }
}
if (-not $client.Connected) { Write-Host "FAILED to connect" -ForegroundColor Red; exit 1 }
$stream = $client.GetStream()
$stream.ReadTimeout = 3000
$writer = New-Object System.IO.StreamWriter($stream)
$writer.AutoFlush = $true
Write-Host "Connected to serial!"

$allOutput = ""

# Wait for boot
$buffer = New-Object byte[] 65536
$sw = [System.Diagnostics.Stopwatch]::StartNew()
$bootText = ""
while ($sw.Elapsed.TotalSeconds -lt 45) {
    if ($stream.DataAvailable) {
        $read = $stream.Read($buffer, 0, $buffer.Length)
        if ($read -gt 0) {
            $chunk = [System.Text.Encoding]::ASCII.GetString($buffer, 0, $read)
            $bootText += $chunk
            $allOutput += $chunk
            if ($bootText -match "trustos.*[\$#]") { break }
        }
    } else { Start-Sleep -Milliseconds 150 }
}
Write-Host "Booted." -ForegroundColor Green

# Send a command and stream output while waiting for prompt
# Shows real-time progress for long-running commands
function Send-Streaming {
    param([string]$cmd, [int]$timeout = 60, [switch]$ShowSerial)
    $buf = New-Object byte[] 65536
    # Drain pending
    $d = [System.Diagnostics.Stopwatch]::StartNew()
    while ($d.ElapsedMilliseconds -lt 1000) {
        if ($stream.DataAvailable) { $stream.Read($buf, 0, $buf.Length) | Out-Null }
        else { Start-Sleep -Milliseconds 50 }
    }
    # Send
    $cmdBytes = [System.Text.Encoding]::ASCII.GetBytes("$cmd`r")
    $stream.Write($cmdBytes, 0, $cmdBytes.Length)
    $stream.Flush()
    # Stream output
    $output = ""
    $s = [System.Diagnostics.Stopwatch]::StartNew()
    $lastPrint = 0
    while ($s.Elapsed.TotalSeconds -lt $timeout) {
        if ($stream.DataAvailable) {
            $read = $stream.Read($buf, 0, $buf.Length)
            if ($read -gt 0) {
                $chunk = [System.Text.Encoding]::ASCII.GetString($buf, 0, $read)
                $output += $chunk
                # Show serial debug output in real-time (lines with [JARVIS])
                if ($ShowSerial) {
                    $newLines = $chunk -split "`n"
                    foreach ($nl in $newLines) {
                        $cl = $nl -replace '[\x00-\x1F\x7F]', '' 
                        $cl = $cl.Trim()
                        if ($cl.Length -gt 3) {
                            if ($cl -match "\[JARVIS\]|Phase|loss|Loss|improved|complete|before|after|Pre-train|step|Epoch") {
                                Write-Host "  >> $cl" -ForegroundColor DarkGray
                            }
                        }
                    }
                }
            }
        } else { Start-Sleep -Milliseconds 100 }
        # Show elapsed every 30s
        if ($s.ElapsedMilliseconds - $lastPrint -gt 30000) {
            $lastPrint = $s.ElapsedMilliseconds
            Write-Host "  ... $([math]::Round($s.Elapsed.TotalSeconds))s elapsed" -ForegroundColor DarkGray
        }
        # Detect shell prompt
        if ($s.ElapsedMilliseconds -ge 500 -and $output.Length -gt 5) {
            if ($output -match "\d{2}:\d{2}:\d{2}\]\s*trustos:[^\r\n]*\$\s*$") {
                Start-Sleep -Milliseconds 300
                while ($stream.DataAvailable) {
                    $read = $stream.Read($buf, 0, $buf.Length)
                    if ($read -gt 0) { $output += [System.Text.Encoding]::ASCII.GetString($buf, 0, $read) }
                }
                break
            }
        }
    }
    if ($s.Elapsed.TotalSeconds -ge $timeout) {
        Write-Host "  [TIMEOUT after ${timeout}s]" -ForegroundColor Red
    }
    $script:allOutput += $output
    return $output
}

# =============================
# STEP 1: INIT
# =============================
Write-Host "`n=== STEP 1: INIT ===" -ForegroundColor Yellow
$initOut = Send-Streaming "jarvis brain init" 30
if ($initOut -match "Neural brain ready|already initialized") {
    Write-Host "Brain initialized OK" -ForegroundColor Green
} else {
    Write-Host "INIT output:" -ForegroundColor Red
    Write-Host $initOut
}

# =============================
# STEP 2: PRETRAIN
# =============================
Write-Host "`n=== STEP 2: PRETRAIN ($Epochs epochs, d_model=128, grad_accum=4) ===" -ForegroundColor Yellow
Write-Host "Streaming serial output..." -ForegroundColor Gray

$pretrainTimer = [System.Diagnostics.Stopwatch]::StartNew()
# 30 min timeout for 5 epochs at d_model=128
$ptOut = Send-Streaming "jarvis brain pretrain $Epochs" 1800 -ShowSerial
$pretrainTimer.Stop()

Write-Host "`nPretrain wall-clock: $([math]::Round($pretrainTimer.Elapsed.TotalSeconds, 1))s" -ForegroundColor Green

# Extract metrics
$lossBefore = "N/A"
$lossAfter = "N/A"
if ($ptOut -match "Loss before:\s*([\d.]+)") { $lossBefore = $Matches[1] }
if ($ptOut -match "Loss after:\s*([\d.]+)") { $lossAfter = $Matches[1] }
Write-Host "  LOSS BEFORE: $lossBefore" -ForegroundColor Cyan
Write-Host "  LOSS AFTER:  $lossAfter" -ForegroundColor Cyan

if ($ptOut -match "improved by ([\d.]+)") {
    Write-Host "  IMPROVEMENT: $($Matches[1])" -ForegroundColor Green
} elseif ($ptOut -match "no improvement") {
    Write-Host "  NO IMPROVEMENT" -ForegroundColor Red
}

# =============================
# STEP 3: EVAL
# =============================
Write-Host "`n=== STEP 3: EVAL ===" -ForegroundColor Yellow
$evalOut = Send-Streaming "jarvis brain eval" 300 -ShowSerial
$evalLines = $evalOut -split "`n" | ForEach-Object { $_.Trim() } | Where-Object { $_.Length -gt 2 }
foreach ($line in $evalLines) {
    $clean = $line -replace '[\x00-\x1F\x7F]', '' -replace '\?{2,}', ''
    $clean = $clean.Trim()
    if ($clean.Length -gt 2 -and ($clean -match "Phase|loss|Loss|Average|baseline|Good|learning|stage")) {
        Write-Host "  $clean"
    }
}

# =============================
# STEP 4: CHAT TESTS
# =============================
Write-Host "`n=== STEP 4: CHAT TESTS ===" -ForegroundColor Yellow

$chatPrompts = @(
    "Who are you?",
    "What is TrustOS?",
    "Hello",
    "Help me",
    "What can you do?"
)

foreach ($prompt in $chatPrompts) {
    Write-Host "`n  Prompt: '$prompt'" -ForegroundColor White
    $chatOut = Send-Streaming "jarvis brain chat $prompt" 60
    $chatLines = $chatOut -split "`n" | ForEach-Object { $_.Trim() } | Where-Object { $_.Length -gt 2 }
    foreach ($line in $chatLines) {
        $clean = $line -replace '[\x00-\x1F\x7F]', '' -replace '\?{2,}', ''
        $clean = $clean.Trim()
        if ($clean.Length -gt 2 -and ($clean -match "Jarvis:|You:|ms,|\d+ chars")) {
            Write-Host "    $clean" -ForegroundColor Gray
        }
    }
}

# =============================
# SAVE OUTPUT
# =============================
$allOutput | Out-File -FilePath $OutputFile -Encoding UTF8
Write-Host "`n=== FULL OUTPUT SAVED ===" -ForegroundColor Cyan
Write-Host "  $OutputFile" -ForegroundColor Gray

# Summary
Write-Host "`n=== SUMMARY ===" -ForegroundColor Cyan
Write-Host "  Epochs:     $Epochs" 
Write-Host "  Loss Before: $lossBefore"
Write-Host "  Loss After:  $lossAfter"
Write-Host "  Wall-clock:  $([math]::Round($pretrainTimer.Elapsed.TotalMinutes, 1)) min"

# Cleanup
Write-Host "`nCleaning up..." -ForegroundColor Gray
$client.Close()
if (-not $qemu.HasExited) {
    Stop-Process -Id $qemu.Id -Force -ErrorAction SilentlyContinue
}
Write-Host "Done!" -ForegroundColor Green
