# Loss decrease verification test
# Trains on the same text multiple times and checks if loss decreases
Set-Location $PSScriptRoot

$QemuExe = "C:\Program Files\qemu\qemu-system-x86_64.exe"
$SerialPort = 5556

# Kill old QEMU
Get-Process -Name "qemu-system-x86_64" -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 2

# Build ISO
Write-Host "=== BUILDING ===" -ForegroundColor Cyan
$buildOut = & powershell -File build-limine.ps1 -NoRun 2>&1
if (-not (Test-Path "trustos.iso")) { Write-Host "BUILD FAILED"; exit 1 }
Write-Host "ISO ready."

# Launch QEMU
Write-Host "=== LAUNCHING QEMU ===" -ForegroundColor Cyan
$serialArg = "tcp:127.0.0.1:${SerialPort},server,nowait"
$qemuArgs = @(
    "-cdrom", "`"$PSScriptRoot\trustos.iso`"",
    "-m", "256M",
    "-machine", "q35",
    "-cpu", "max",
    "-smp", "2",
    "-accel", "tcg,thread=multi",
    "-display", "none",
    "-vga", "std",
    "-device", "virtio-net-pci,netdev=net0",
    "-netdev", "user,id=net0",
    "-serial", $serialArg,
    "-no-reboot"
)
$qemu = Start-Process -FilePath $QemuExe -ArgumentList $qemuArgs -PassThru
Write-Host "QEMU PID: $($qemu.Id)"

# Connect serial TCP (with retry)
Write-Host "Connecting serial..." -ForegroundColor Cyan
$client = New-Object System.Net.Sockets.TcpClient
$connected = $false
for ($i = 0; $i -lt 60; $i++) {
    try {
        $client.Connect("127.0.0.1", $SerialPort)
        $connected = $true
        break
    } catch {
        Start-Sleep -Milliseconds 300
    }
}
if (-not $connected) {
    Write-Host "FATAL: Could not connect to serial" -ForegroundColor Red
    Stop-Process -Id $qemu.Id -Force -ErrorAction SilentlyContinue
    exit 1
}
$stream = $client.GetStream()
$stream.ReadTimeout = 3000
$writer = New-Object System.IO.StreamWriter($stream)
$writer.AutoFlush = $true
Write-Host "Connected!" -ForegroundColor Green

# Wait for boot
Write-Host "Waiting for boot..." -ForegroundColor Cyan
$buffer = New-Object byte[] 16384
$sw = [System.Diagnostics.Stopwatch]::StartNew()
$bootText = ""
$booted = $false
while ($sw.Elapsed.TotalSeconds -lt 30) {
    if ($stream.DataAvailable) {
        $read = $stream.Read($buffer, 0, $buffer.Length)
        if ($read -gt 0) {
            $bootText += [System.Text.Encoding]::ASCII.GetString($buffer, 0, $read)
            if ($bootText -match "trustos.*[\$#]") { $booted = $true; break }
        }
    } else { Start-Sleep -Milliseconds 150 }
}
if (-not $booted) { Write-Host "Boot timeout!"; Stop-Process -Id $qemu.Id -Force; exit 1 }
Write-Host "Booted OK ($($bootText.Length) chars)"

# Send command helper (from auto-test pattern)
function Send-Cmd {
    param([string]$cmd, [int]$timeout = 10)
    
    $buf = New-Object byte[] 16384
    # Drain
    $drainSw = [System.Diagnostics.Stopwatch]::StartNew()
    $lastData = $drainSw.ElapsedMilliseconds
    while (($drainSw.ElapsedMilliseconds - $lastData) -lt 300 -and $drainSw.ElapsedMilliseconds -lt 3000) {
        if ($stream.DataAvailable) {
            $stream.Read($buf, 0, $buf.Length) | Out-Null
            $lastData = $drainSw.ElapsedMilliseconds
        } else { Start-Sleep -Milliseconds 50 }
    }
    
    # Send
    $cmdBytes = [System.Text.Encoding]::ASCII.GetBytes("$cmd`r")
    $stream.Write($cmdBytes, 0, $cmdBytes.Length)
    $stream.Flush()
    
    # Collect
    $output = ""
    $csw = [System.Diagnostics.Stopwatch]::StartNew()
    while ($csw.Elapsed.TotalSeconds -lt $timeout) {
        if ($stream.DataAvailable) {
            $read = $stream.Read($buf, 0, $buf.Length)
            if ($read -gt 0) { $output += [System.Text.Encoding]::ASCII.GetString($buf, 0, $read) }
        } else { Start-Sleep -Milliseconds 50 }
        # Check for prompt (done)
        if ($csw.ElapsedMilliseconds -ge 500 -and $output.Length -gt 5) {
            if ($output -match "\d{2}:\d{2}:\d{2}\]\s*trustos:[^\r\n]*\$\s*$") {
                Start-Sleep -Milliseconds 100
                while ($stream.DataAvailable) {
                    $read = $stream.Read($buf, 0, $buf.Length)
                    if ($read -gt 0) { $output += [System.Text.Encoding]::ASCII.GetString($buf, 0, $read) }
                }
                break
            }
        }
    }
    return $output
}

# === INIT ===
Write-Host "`n=== INIT ===" -ForegroundColor Yellow
$initOut = Send-Cmd "jarvis brain init" 10
if ($initOut -match "Neural brain ready") {
    Write-Host "Brain initialized OK" -ForegroundColor Green
} else {
    Write-Host "Init output: $($initOut.Substring(0, [Math]::Min(300, $initOut.Length)))" -ForegroundColor Red
}

# === TRAIN 5x on "Hello World" ===
Write-Host "`n=== TRAINING 5x 'Hello World' ===" -ForegroundColor Yellow
$losses = @()
for ($i = 1; $i -le 5; $i++) {
    $trainOut = Send-Cmd "jarvis brain train Hello World" 15
    if ($trainOut -match "Loss:\s*([\d.]+)") {
        $loss = [double]$Matches[1]
        $losses += $loss
        Write-Host ("  Iteration {0}: Loss = {1:F4}" -f $i, $loss)
    } else {
        Write-Host "  Iteration $i : NO LOSS FOUND" -ForegroundColor Red
        $preview = $trainOut.Substring(0, [Math]::Min(200, $trainOut.Length))
        Write-Host "  Raw: $preview" -ForegroundColor DarkGray
    }
}

# === ANALYSIS ===
Write-Host "`n=== ANALYSIS ===" -ForegroundColor Cyan
if ($losses.Count -ge 2) {
    $first = $losses[0]
    $last = $losses[$losses.Count - 1]
    $delta = $last - $first
    Write-Host "  First loss:  $($first.ToString('F4'))"
    Write-Host "  Last loss:   $($last.ToString('F4'))"
    Write-Host "  Delta:       $($delta.ToString('F4'))"
    Write-Host "  All losses:  $($losses -join ', ')"
    if ($delta -lt -0.001) {
        Write-Host "  >>> LOSS DECREASED! Backprop WORKS! <<<" -ForegroundColor Green
    } elseif ([Math]::Abs($delta) -lt 0.001) {
        Write-Host "  >>> Loss unchanged" -ForegroundColor Yellow
    } else {
        Write-Host "  >>> Loss INCREASED - possible issue" -ForegroundColor Red
    }
} else {
    Write-Host "  Not enough loss values ($($losses.Count))" -ForegroundColor Red
}

# === TRAIN 5x on longer text ===
Write-Host "`n=== TRAINING 5x 'TrustOS is a secure operating system' ===" -ForegroundColor Yellow
$losses2 = @()
for ($i = 1; $i -le 5; $i++) {
    $trainOut = Send-Cmd "jarvis brain train TrustOS is a secure operating system" 15
    if ($trainOut -match "Loss:\s*([\d.]+)") {
        $loss = [double]$Matches[1]
        $losses2 += $loss
        Write-Host ("  Iteration {0}: Loss = {1:F4}" -f $i, $loss)
    } else {
        Write-Host "  Iteration $i : NO LOSS FOUND" -ForegroundColor Red
    }
}

if ($losses2.Count -ge 2) {
    $first2 = $losses2[0]
    $last2 = $losses2[$losses2.Count - 1]
    $delta2 = $last2 - $first2
    Write-Host "  First loss:  $($first2.ToString('F4'))"
    Write-Host "  Last loss:   $($last2.ToString('F4'))"
    Write-Host "  Delta:       $($delta2.ToString('F4'))"
    if ($delta2 -lt -0.001) {
        Write-Host "  >>> LOSS DECREASED! <<<" -ForegroundColor Green
    } else {
        Write-Host "  >>> Loss did NOT decrease" -ForegroundColor Red
    }
}

# Check alive
Write-Host "`nQEMU alive: $(-not $qemu.HasExited)" -ForegroundColor Cyan

# Cleanup
$client.Close()
Stop-Process -Id $qemu.Id -Force -ErrorAction SilentlyContinue
Write-Host "Done."
