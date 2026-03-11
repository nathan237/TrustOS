# Full pretrain demo: init -> pretrain (92 sequences) -> eval
Set-Location $PSScriptRoot

$QemuExe = "C:\Program Files\qemu\qemu-system-x86_64.exe"
$SerialPort = 5556

Get-Process -Name "qemu-system-x86_64" -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 2

Write-Host "=== BUILDING ===" -ForegroundColor Cyan
& powershell -File build-limine.ps1 -NoRun 2>&1 | Out-Null
if (-not (Test-Path "trustos.iso")) { Write-Host "BUILD FAILED"; exit 1 }
Write-Host "ISO ready."

Write-Host "=== LAUNCHING QEMU ===" -ForegroundColor Cyan
$serialArg = "tcp:127.0.0.1:${SerialPort},server,nowait"
$qemuArgs = @(
    "-cdrom", "`"$PSScriptRoot\trustos.iso`"",
    "-m", "256M", "-machine", "q35", "-cpu", "max", "-smp", "2",
    "-accel", "tcg,thread=multi", "-display", "none", "-vga", "std",
    "-device", "virtio-net-pci,netdev=net0", "-netdev", "user,id=net0",
    "-serial", $serialArg, "-no-reboot"
)
$qemu = Start-Process -FilePath $QemuExe -ArgumentList $qemuArgs -PassThru
Write-Host "QEMU PID: $($qemu.Id)"

$client = New-Object System.Net.Sockets.TcpClient
for ($i = 0; $i -lt 60; $i++) {
    try { $client.Connect("127.0.0.1", $SerialPort); break } catch { Start-Sleep -Milliseconds 300 }
}
$stream = $client.GetStream()
$stream.ReadTimeout = 3000
$writer = New-Object System.IO.StreamWriter($stream)
$writer.AutoFlush = $true
Write-Host "Connected!"

# Wait for boot
$buffer = New-Object byte[] 16384
$sw = [System.Diagnostics.Stopwatch]::StartNew()
$bootText = ""
while ($sw.Elapsed.TotalSeconds -lt 30) {
    if ($stream.DataAvailable) {
        $read = $stream.Read($buffer, 0, $buffer.Length)
        if ($read -gt 0) {
            $bootText += [System.Text.Encoding]::ASCII.GetString($buffer, 0, $read)
            if ($bootText -match "trustos.*[\$#]") { break }
        }
    } else { Start-Sleep -Milliseconds 150 }
}
Write-Host "Booted."

function Send-Cmd {
    param([string]$cmd, [int]$timeout = 10)
    $buf = New-Object byte[] 16384
    $d = [System.Diagnostics.Stopwatch]::StartNew()
    $ld = $d.ElapsedMilliseconds
    while (($d.ElapsedMilliseconds - $ld) -lt 300 -and $d.ElapsedMilliseconds -lt 3000) {
        if ($stream.DataAvailable) { $stream.Read($buf, 0, $buf.Length) | Out-Null; $ld = $d.ElapsedMilliseconds }
        else { Start-Sleep -Milliseconds 50 }
    }
    $cmdBytes = [System.Text.Encoding]::ASCII.GetBytes("$cmd`r")
    $stream.Write($cmdBytes, 0, $cmdBytes.Length)
    $stream.Flush()
    $output = ""
    $s = [System.Diagnostics.Stopwatch]::StartNew()
    while ($s.Elapsed.TotalSeconds -lt $timeout) {
        if ($stream.DataAvailable) {
            $read = $stream.Read($buf, 0, $buf.Length)
            if ($read -gt 0) { $output += [System.Text.Encoding]::ASCII.GetString($buf, 0, $read) }
        } else { Start-Sleep -Milliseconds 50 }
        if ($s.ElapsedMilliseconds -ge 500 -and $output.Length -gt 5) {
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
    Write-Host "INIT: $initOut" -ForegroundColor Red
}

# === PRETRAIN (jarvis brain pretrain) ===
Write-Host "`n=== PRETRAIN (92 seq, please wait ~2min) ===" -ForegroundColor Yellow
$ptOut = Send-Cmd "jarvis brain pretrain" 300   # 5 min timeout
$ptOut | Out-File -FilePath "$PSScriptRoot\pretrain_output.txt" -Encoding UTF8

# Parse results
$lines = $ptOut -split "`n" | ForEach-Object { $_.Trim() } | Where-Object { $_.Length -gt 0 }
foreach ($line in $lines) {
    $clean = $line -replace '[\x00-\x1F\x7F]', '' -replace '\?{2,}', ''
    $clean = $clean.Trim()
    if ($clean.Length -gt 2) {
        # Show phase lines, loss lines, improvement lines
        if ($clean -match "Phase|loss|Loss|improved|step|Pre-train|eval|maturity") {
            Write-Host "  $clean"
        }
    }
}

# Extract before/after
if ($ptOut -match "Loss before:\s*([\d.]+)") { 
    Write-Host "`n  LOSS BEFORE: $($Matches[1])" -ForegroundColor Cyan 
}
if ($ptOut -match "Loss after:\s*([\d.]+)") {
    Write-Host "  LOSS AFTER:  $($Matches[1])" -ForegroundColor Cyan
}
if ($ptOut -match "improved by ([\d.]+)") {
    Write-Host "  IMPROVEMENT: $($Matches[1])" -ForegroundColor Green
} elseif ($ptOut -match "no improvement") {
    Write-Host "  NO IMPROVEMENT" -ForegroundColor Red
}

Write-Host "`nQEMU alive: $(-not $qemu.HasExited)" -ForegroundColor Cyan

$client.Close()
Stop-Process -Id $qemu.Id -Force -ErrorAction SilentlyContinue
Write-Host "Done."
