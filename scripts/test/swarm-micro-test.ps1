<#
.SYNOPSIS
    Quick swarm micro-test: boot 2 nodes, mesh start, peer ping
#>
$ErrorActionPreference = "Continue"
$QemuExe = "C:\Program Files\qemu\qemu-system-x86_64.exe"
$iso = "$PSScriptRoot\trustos.iso"

function Send-Serial($port, $cmd, $timeoutSec) {
    $tcp = New-Object System.Net.Sockets.TcpClient("127.0.0.1", $port)
    $s = $tcp.GetStream()
    $buf = New-Object byte[] 65536

    # Drain any buffered data
    $sw0 = [System.Diagnostics.Stopwatch]::StartNew()
    while ($sw0.Elapsed.TotalSeconds -lt 1) {
        if ($s.DataAvailable) { $s.Read($buf, 0, $buf.Length) | Out-Null }
        else { Start-Sleep -Milliseconds 50 }
    }

    # Send command
    $bytes = [System.Text.Encoding]::ASCII.GetBytes("$cmd`r")
    $s.Write($bytes, 0, $bytes.Length)
    $s.Flush()
    Start-Sleep -Milliseconds 300

    # Read response
    $out = ""
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    while ($sw.Elapsed.TotalSeconds -lt $timeoutSec) {
        if ($s.DataAvailable) {
            $n = $s.Read($buf, 0, $buf.Length)
            $out += [System.Text.Encoding]::ASCII.GetString($buf, 0, $n)
        } else {
            Start-Sleep -Milliseconds 100
            if ($out.Length -gt 20 -and $out -match "trustos:.*\$") { break }
        }
    }
    $tcp.Close()
    return $out
}

function Wait-Serial($port, $pattern, $timeoutSec) {
    $tcp = New-Object System.Net.Sockets.TcpClient("127.0.0.1", $port)
    $s = $tcp.GetStream()
    $buf = New-Object byte[] 65536
    $out = ""
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    while ($sw.Elapsed.TotalSeconds -lt $timeoutSec) {
        if ($s.DataAvailable) {
            $n = $s.Read($buf, 0, $buf.Length)
            $out += [System.Text.Encoding]::ASCII.GetString($buf, 0, $n)
        } else { Start-Sleep -Milliseconds 100 }
        if ($out -match $pattern) { break }
    }
    $tcp.Close()
    return $out
}

# Kill existing
Stop-Process -Name qemu-system-x86_64 -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 2

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  SWARM MICRO-TEST (2 nodes)" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# Boot Node 0 (listen)
Write-Host "`n[1] Booting Node 0 (listen)..." -NoNewline
$p0 = Start-Process -FilePath $QemuExe -ArgumentList @(
    "-cdrom",$iso,"-m","256M","-smp","1","-cpu","Haswell",
    "-machine","q35","-accel","whpx","-display","none",
    "-serial","tcp:127.0.0.1:5570,server,nowait",
    "-netdev","socket,id=mesh0,listen=:5580",
    "-device","virtio-net-pci,netdev=mesh0,mac=52:54:00:00:01:01",
    "-no-reboot"
) -PassThru -WindowStyle Hidden
Write-Host " PID=$($p0.Id)" -ForegroundColor Green
Start-Sleep -Seconds 3

# Boot Node 1 (connect)
Write-Host "[2] Booting Node 1 (connect)..." -NoNewline
$p1 = Start-Process -FilePath $QemuExe -ArgumentList @(
    "-cdrom",$iso,"-m","256M","-smp","1","-cpu","Haswell",
    "-machine","q35","-accel","whpx","-display","none",
    "-serial","tcp:127.0.0.1:5571,server,nowait",
    "-netdev","socket,id=mesh0,connect=127.0.0.1:5580",
    "-device","virtio-net-pci,netdev=mesh0,mac=52:54:00:00:01:02",
    "-no-reboot"
) -PassThru -WindowStyle Hidden
Write-Host " PID=$($p1.Id)" -ForegroundColor Green

# Wait for boot
Write-Host "[3] Waiting for boot (30s)..." -ForegroundColor Yellow
$boot0 = Wait-Serial 5570 "trustos:.*\$" 35
$boot1 = Wait-Serial 5571 "trustos:.*\$" 35

if ($boot0 -match "trustos") { Write-Host "  Node 0: BOOTED" -ForegroundColor Green }
else { Write-Host "  Node 0: TIMEOUT" -ForegroundColor Red }
if ($boot1 -match "trustos") { Write-Host "  Node 1: BOOTED" -ForegroundColor Green }
else { Write-Host "  Node 1: TIMEOUT" -ForegroundColor Red }

# Brain init
Write-Host "`n[4] Brain init..." -ForegroundColor Yellow
$init0 = Send-Serial 5570 "jarvis brain init" 15
$init1 = Send-Serial 5571 "jarvis brain init" 15

foreach ($line in ($init0 -split "`n")) {
    if ($line -match "SIMD|Parameters|brain ready") { Write-Host "  N0: $($line.Trim())" }
}
foreach ($line in ($init1 -split "`n")) {
    if ($line -match "SIMD|Parameters|brain ready") { Write-Host "  N1: $($line.Trim())" }
}

# Mesh start
Write-Host "`n[5] Mesh start..." -ForegroundColor Yellow
$mesh0 = Send-Serial 5570 "mesh start" 10
$mesh1 = Send-Serial 5571 "mesh start" 10

foreach ($line in ($mesh0 -split "`n")) {
    if ($line -match "MESH|Fallback|RPC|7700|7701") { Write-Host "  N0: $($line.Trim())" }
}
foreach ($line in ($mesh1 -split "`n")) {
    if ($line -match "MESH|Fallback|RPC|7700|7701") { Write-Host "  N1: $($line.Trim())" }
}

# Peer ping
Write-Host "`n[6] Peer discovery..." -ForegroundColor Yellow
$ping0 = Send-Serial 5570 "mesh ping 10.0.100.2" 15
foreach ($line in ($ping0 -split "`n")) {
    if ($line -match "alive|failed|Peer|ARP|RPC|timeout") { Write-Host "  N0->N1: $($line.Trim())" }
}

$ping1 = Send-Serial 5571 "mesh ping 10.0.100.1" 15
foreach ($line in ($ping1 -split "`n")) {
    if ($line -match "alive|failed|Peer|ARP|RPC|timeout") { Write-Host "  N1->N0: $($line.Trim())" }
}

# Mesh status
Write-Host "`n[7] Mesh status..." -ForegroundColor Yellow
$st = Send-Serial 5570 "mesh status" 5
foreach ($line in ($st -split "`n")) {
    if ($line -match "peer|Peer|node|active|IP|10\.0") { Write-Host "  N0: $($line.Trim())" }
}

# Cleanup
Write-Host "`n[8] Cleanup..." -ForegroundColor Yellow
Stop-Process -Id $p0.Id -Force -ErrorAction SilentlyContinue
Stop-Process -Id $p1.Id -Force -ErrorAction SilentlyContinue
Write-Host "  VMs stopped" -ForegroundColor Green

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "  MICRO-TEST COMPLETE" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
