<#
.SYNOPSIS
    Boot TrustOS in QEMU and run a full network scan on google.com via serial.
.DESCRIPTION
    Boots TrustOS, waits for shell prompt, then runs all network commands
    targeting google.com, capturing serial output for each command.
#>

param(
    [string]$IsoPath = "$PSScriptRoot\trustos.iso",
    [int]$SerialPort = 5556,
    [int]$BootTimeout = 30
)

$QemuExe = "C:\Program Files\qemu\qemu-system-x86_64.exe"

if (-not (Test-Path $QemuExe)) { Write-Error "QEMU not found"; exit 1 }
if (-not (Test-Path $IsoPath)) { Write-Error "ISO not found: $IsoPath"; exit 1 }

# ── Helpers ──────────────────────────────────────────────────────────
function Send-Command {
    param($stream, $writer, [string]$cmd, [int]$timeout = 8)

    $buffer = New-Object byte[] 32768

    # Drain leftover
    $drainSw = [System.Diagnostics.Stopwatch]::StartNew()
    $lastData = $drainSw.ElapsedMilliseconds
    while (($drainSw.ElapsedMilliseconds - $lastData) -lt 300 -and $drainSw.ElapsedMilliseconds -lt 3000) {
        if ($stream.DataAvailable) {
            $stream.Read($buffer, 0, $buffer.Length) | Out-Null
            $lastData = $drainSw.ElapsedMilliseconds
        } else {
            Start-Sleep -Milliseconds 50
        }
    }

    # Send command
    $cmdBytes = [System.Text.Encoding]::ASCII.GetBytes("$cmd`r")
    $stream.Write($cmdBytes, 0, $cmdBytes.Length)
    $stream.Flush()

    # Collect output
    $output = ""
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    while ($sw.Elapsed.TotalSeconds -lt $timeout) {
        if ($stream.DataAvailable) {
            $read = $stream.Read($buffer, 0, $buffer.Length)
            if ($read -gt 0) {
                $output += [System.Text.Encoding]::ASCII.GetString($buffer, 0, $read)
            }
        } else {
            Start-Sleep -Milliseconds 100
        }
        # Check for prompt return (shell ready)
        if ($sw.ElapsedMilliseconds -ge 800 -and $output.Length -gt 5) {
            if ($output -match "\d{2}:\d{2}:\d{2}\]\s*trustos:[^\r\n]*\$\s*$") {
                Start-Sleep -Milliseconds 200
                while ($stream.DataAvailable) {
                    $read = $stream.Read($buffer, 0, $buffer.Length)
                    if ($read -gt 0) { $output += [System.Text.Encoding]::ASCII.GetString($buffer, 0, $read) }
                }
                break
            }
        }
    }

    return $output
}

# ── Kill existing QEMU ───────────────────────────────────────────────
$existingQemu = Get-Process -Name "qemu-system-x86_64" -ErrorAction SilentlyContinue
if ($existingQemu) {
    Write-Host "Killing existing QEMU..." -ForegroundColor Yellow
    $existingQemu | Stop-Process -Force
    Start-Sleep -Seconds 2
}

# ── Launch QEMU ──────────────────────────────────────────────────────
Write-Host ""
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "  TrustOS Network Scan - google.com (Serial Mode)" -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host ""

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
    "-device", "virtio-net-pci,netdev=net0",
    "-netdev", "user,id=net0",
    "-device", "intel-hda",
    "-device", "hda-duplex",
    "-device", "qemu-xhci,id=xhci",
    "-device", "usb-kbd,bus=xhci.0",
    "-device", "usb-mouse,bus=xhci.0",
    "-serial", $serialArg,
    "-no-reboot"
)

Write-Host "[1/3] Starting QEMU..." -ForegroundColor White
$qemuProcess = Start-Process -FilePath $QemuExe -ArgumentList $qemuArgs -PassThru
Write-Host ("  PID: {0}" -f $qemuProcess.Id) -ForegroundColor DarkGray

# ── Connect serial TCP ───────────────────────────────────────────────
Write-Host "[2/3] Connecting serial TCP port $SerialPort..." -ForegroundColor White
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
    Write-Host "FATAL: Could not connect to serial port $SerialPort" -ForegroundColor Red
    Stop-Process -Id $qemuProcess.Id -Force -ErrorAction SilentlyContinue
    exit 1
}

$stream = $client.GetStream()
$stream.ReadTimeout = 3000
$writer = New-Object System.IO.StreamWriter($stream)
$writer.AutoFlush = $true
Write-Host "  Connected!" -ForegroundColor Green

# ── Wait for boot ────────────────────────────────────────────────────
Write-Host "[3/3] Waiting for TrustOS boot..." -ForegroundColor White
$buffer = New-Object byte[] 16384
$sw = [System.Diagnostics.Stopwatch]::StartNew()
$bootText = ""
$booted = $false

while ($sw.Elapsed.TotalSeconds -lt $BootTimeout) {
    if ($stream.DataAvailable) {
        $read = $stream.Read($buffer, 0, $buffer.Length)
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
    $client.Close()
    Stop-Process -Id $qemuProcess.Id -Force -ErrorAction SilentlyContinue
    exit 1
}

$bootTime = [math]::Round($sw.Elapsed.TotalSeconds, 1)
Write-Host ("  Booted in {0}s" -f $bootTime) -ForegroundColor Green
Write-Host ""

# Stabilize
Start-Sleep -Milliseconds 500
$writer.Write("`r")
$writer.Flush()
Start-Sleep -Milliseconds 500
while ($stream.DataAvailable) { $stream.Read($buffer, 0, $buffer.Length) | Out-Null }

# ── Network commands to execute ──────────────────────────────────────
$commands = @(
    @{ Name = "ifconfig";               Cmd = "ifconfig";                   Timeout = 8  }
    @{ Name = "arp";                     Cmd = "arp";                        Timeout = 5  }
    @{ Name = "route";                   Cmd = "route";                      Timeout = 5  }
    @{ Name = "netstat";                 Cmd = "netstat";                    Timeout = 5  }
    @{ Name = "ping gateway";            Cmd = "ping 10.0.2.2";             Timeout = 10 }
    @{ Name = "nslookup google.com";     Cmd = "nslookup google.com";       Timeout = 10 }
    @{ Name = "ping google.com";         Cmd = "ping google.com";           Timeout = 12 }
    @{ Name = "nmap google.com";         Cmd = "nmap google.com";           Timeout = 20 }
    @{ Name = "nmap google.com -A";      Cmd = "nmap google.com -A";        Timeout = 30 }
    @{ Name = "banner google.com 80";    Cmd = "banner google.com 80";      Timeout = 15 }
    @{ Name = "curl google.com";         Cmd = "curl http://google.com";    Timeout = 15 }
    @{ Name = "discover";                Cmd = "discover";                   Timeout = 15 }
    @{ Name = "scantest google.com";     Cmd = "scantest google.com";       Timeout = 30 }
)

$fullLog = ""
$totalStart = [System.Diagnostics.Stopwatch]::StartNew()

Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "  Running Network Commands" -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host ""

foreach ($entry in $commands) {
    $name = $entry.Name
    $cmd = $entry.Cmd
    $cmdTimeout = $entry.Timeout

    Write-Host ("  [{0}] {1}" -f $name, $cmd) -ForegroundColor Cyan -NoNewline
    Write-Host " ... " -NoNewline

    $output = Send-Command -stream $stream -writer $writer -cmd $cmd -timeout $cmdTimeout

    # Clean output for display: remove ANSI codes, keep readable text
    $cleanOutput = $output -replace '\x1B\[[0-9;]*[mGHJK]', ''
    $cleanOutput = $cleanOutput -replace '[\x00-\x08\x0B\x0C\x0E-\x1F]', ''

    if ($cleanOutput.Length -gt 10) {
        Write-Host "[OK]" -ForegroundColor Green
    } else {
        Write-Host "[EMPTY]" -ForegroundColor Yellow
    }

    # Print output
    Write-Host "  ──────────────────────────────────────────────────" -ForegroundColor DarkGray
    $lines = $cleanOutput -split "`n"
    foreach ($line in $lines) {
        $trimmed = $line.Trim()
        if ($trimmed.Length -gt 0) {
            Write-Host "    $trimmed" -ForegroundColor White
        }
    }
    Write-Host "  ──────────────────────────────────────────────────" -ForegroundColor DarkGray
    Write-Host ""

    $fullLog += "`n=== $name ($cmd) ===`n$cleanOutput`n"
}

$totalDuration = [math]::Round($totalStart.Elapsed.TotalSeconds, 1)

# ── Cleanup ──────────────────────────────────────────────────────────
Write-Host "Shutting down QEMU..." -ForegroundColor DarkGray
try { $client.Close() } catch {}
Start-Sleep -Seconds 1
Stop-Process -Id $qemuProcess.Id -Force -ErrorAction SilentlyContinue

# ── Save log ─────────────────────────────────────────────────────────
$reportFile = "$PSScriptRoot\google_scan_report.txt"
$header = @"
==============================================================
  TrustOS Network Scan Report — google.com
  Date: $(Get-Date -Format "yyyy-MM-dd HH:mm:ss")
  Boot time: ${bootTime}s | Scan duration: ${totalDuration}s
  QEMU: q35, 256M, virtio-net, user-mode NAT
==============================================================
"@

($header + $fullLog) | Out-File -FilePath $reportFile -Encoding UTF8

Write-Host ""
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "  Scan Complete" -ForegroundColor Cyan
Write-Host ("  Duration: {0}s (+ {1}s boot)" -f $totalDuration, $bootTime) -ForegroundColor White
Write-Host ("  Report: {0}" -f $reportFile) -ForegroundColor White
Write-Host "============================================================" -ForegroundColor Cyan
