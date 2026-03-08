<#
.SYNOPSIS
    Federated learning micro-test: boot 2 nodes, auto-discovery, leader election, federated sync
#>
$ErrorActionPreference = "Continue"
$QemuExe = "C:\Program Files\qemu\qemu-system-x86_64.exe"
$iso = "$PSScriptRoot\trustos.iso"

function Send-Serial($port, $cmd, $timeoutSec) {
    $tcp = New-Object System.Net.Sockets.TcpClient("127.0.0.1", $port)
    $s = $tcp.GetStream()
    $buf = New-Object byte[] 65536

    # Drain buffered data
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
Write-Host "  FEDERATED LEARNING MICRO-TEST" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

$results = @()

# ── Step 1: Boot both nodes ──
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

Write-Host "[1b] Booting Node 1 (connect)..." -NoNewline
$p1 = Start-Process -FilePath $QemuExe -ArgumentList @(
    "-cdrom",$iso,"-m","256M","-smp","1","-cpu","Haswell",
    "-machine","q35","-accel","whpx","-display","none",
    "-serial","tcp:127.0.0.1:5571,server,nowait",
    "-netdev","socket,id=mesh0,connect=127.0.0.1:5580",
    "-device","virtio-net-pci,netdev=mesh0,mac=52:54:00:00:01:02",
    "-no-reboot"
) -PassThru -WindowStyle Hidden
Write-Host " PID=$($p1.Id)" -ForegroundColor Green

# ── Step 2: Wait for boot ──
Write-Host "[2] Waiting for boot (35s)..." -ForegroundColor Yellow
$boot0 = Wait-Serial 5570 "trustos:.*\$" 35
$boot1 = Wait-Serial 5571 "trustos:.*\$" 35

$b0ok = $boot0 -match "trustos"
$b1ok = $boot1 -match "trustos"
if ($b0ok) { Write-Host "  Node 0: BOOTED" -ForegroundColor Green }
else { Write-Host "  Node 0: TIMEOUT" -ForegroundColor Red }
if ($b1ok) { Write-Host "  Node 1: BOOTED" -ForegroundColor Green }
else { Write-Host "  Node 1: TIMEOUT" -ForegroundColor Red }
$results += "Boot:       $(if ($b0ok -and $b1ok) {'PASS'} else {'FAIL'})"

if (-not ($b0ok -and $b1ok)) {
    Write-Host "Boot failed, aborting." -ForegroundColor Red
    Stop-Process -Id $p0.Id -Force -ErrorAction SilentlyContinue
    Stop-Process -Id $p1.Id -Force -ErrorAction SilentlyContinue
    exit 1
}

# ── Step 3: Brain init ──
Write-Host "`n[3] Brain init..." -ForegroundColor Yellow
$init0 = Send-Serial 5570 "jarvis brain init" 15
$init1 = Send-Serial 5571 "jarvis brain init" 15

$i0ok = $init0 -match "brain ready"
$i1ok = $init1 -match "brain ready"
foreach ($line in ($init0 -split "`n")) {
    if ($line -match "SIMD|Parameters|brain ready") { Write-Host "  N0: $($line.Trim())" }
}
foreach ($line in ($init1 -split "`n")) {
    if ($line -match "SIMD|Parameters|brain ready") { Write-Host "  N1: $($line.Trim())" }
}
$results += "Brain init: $(if ($i0ok -and $i1ok) {'PASS'} else {'FAIL'})"

# ── Step 4: Mesh start ──
Write-Host "`n[4] Mesh start..." -ForegroundColor Yellow
$mesh0 = Send-Serial 5570 "mesh start" 10
$mesh1 = Send-Serial 5571 "mesh start" 10

$m0ok = $mesh0 -match "mesh.*started|Mesh.*active"
$m1ok = $mesh1 -match "mesh.*started|Mesh.*active"
foreach ($line in ($mesh0 -split "`n")) {
    if ($line -match "MESH|Fallback|RPC|7700|7701|Announce") { Write-Host "  N0: $($line.Trim())" }
}
foreach ($line in ($mesh1 -split "`n")) {
    if ($line -match "MESH|Fallback|RPC|7700|7701|Announce") { Write-Host "  N1: $($line.Trim())" }
}
$results += "Mesh start: $(if ($m0ok -and $m1ok) {'PASS'} else {'FAIL'})"

# ── Step 5: Wait for auto-discovery via broadcast ──
Write-Host "`n[5] Waiting 15s for auto-discovery..." -ForegroundColor Yellow
Start-Sleep -Seconds 15

$st0 = Send-Serial 5570 "mesh status" 5
$st1 = Send-Serial 5571 "mesh status" 5

Write-Host "  --- Node 0 ---"
foreach ($line in ($st0 -split "`n")) {
    if ($line -match "Mesh:|Consensus:|peer|Peer|Discovered|leader|10\.0") { Write-Host "  $($line.Trim())" }
}
Write-Host "  --- Node 1 ---"
foreach ($line in ($st1 -split "`n")) {
    if ($line -match "Mesh:|Consensus:|peer|Peer|Discovered|leader|10\.0") { Write-Host "  $($line.Trim())" }
}

$disc0 = $st0 -match "Discovered: [1-9]|Peers: [1-9]"
$disc1 = $st1 -match "Discovered: [1-9]|Peers: [1-9]"
$results += "Discovery:  $(if ($disc0 -and $disc1) {'PASS'} else {'FAIL'})"

# ── Step 6: Peer ping (RPC connectivity) ──
Write-Host "`n[6] RPC peer ping..." -ForegroundColor Yellow
$ping0 = Send-Serial 5570 "mesh ping 10.0.100.2" 15
$ping1 = Send-Serial 5571 "mesh ping 10.0.100.1" 15

$p0ok = $ping0 -match "alive"
$p1ok = $ping1 -match "alive"
foreach ($line in ($ping0 -split "`n")) {
    if ($line -match "alive|failed|Peer|timeout|RPC") { Write-Host "  N0->N1: $($line.Trim())" }
}
foreach ($line in ($ping1 -split "`n")) {
    if ($line -match "alive|failed|Peer|timeout|RPC") { Write-Host "  N1->N0: $($line.Trim())" }
}
$results += "RPC ping:   $(if ($p0ok -and $p1ok) {'PASS'} else {'FAIL'})"

# ── Step 7: Wait for leader election (20s election timeout + some margin) ──
Write-Host "`n[7] Waiting 30s for leader election..." -ForegroundColor Yellow
Start-Sleep -Seconds 30

$le0 = Send-Serial 5570 "mesh status" 5
$le1 = Send-Serial 5571 "mesh status" 5

$hasLeader = ($le0 -match "leader=true") -or ($le1 -match "leader=true")
Write-Host "  --- Node 0 ---"
foreach ($line in ($le0 -split "`n")) {
    if ($line -match "Consensus:|leader|Leader|term") { Write-Host "  $($line.Trim())" }
}
Write-Host "  --- Node 1 ---"
foreach ($line in ($le1 -split "`n")) {
    if ($line -match "Consensus:|leader|Leader|term") { Write-Host "  $($line.Trim())" }
}
$results += "Election:   $(if ($hasLeader) {'PASS'} else {'FAIL'})"

# ── Step 8: Enable federated + force sync ──
Write-Host "`n[8] Enable federated + force sync..." -ForegroundColor Yellow
$fed0 = Send-Serial 5570 "mesh federate on" 5
$fed1 = Send-Serial 5571 "mesh federate on" 5
Write-Host "  Federated enabled on both nodes"

Start-Sleep -Seconds 2

$sync0 = Send-Serial 5570 "mesh federate sync" 20
$sync1 = Send-Serial 5571 "mesh federate sync" 20

Write-Host "  --- Node 0 sync ---"
foreach ($line in ($sync0 -split "`n")) {
    if ($line -match "FED|fed|sync|Sync|gradient|Gradient|leader|worker|aggr|round|push|Pull|weights") {
        Write-Host "  $($line.Trim())"
    }
}
Write-Host "  --- Node 1 sync ---"
foreach ($line in ($sync1 -split "`n")) {
    if ($line -match "FED|fed|sync|Sync|gradient|Gradient|leader|worker|aggr|round|push|Pull|weights") {
        Write-Host "  $($line.Trim())"
    }
}
$syncOk = ($sync0 -match "sync|gradient|aggregate|round|push|weight") -or ($sync1 -match "sync|gradient|aggregate|round|push|weight")
$results += "Fed sync:   $(if ($syncOk) {'PASS'} else {'FAIL'})"

# ── Step 9: Mesh status final ──
Write-Host "`n[9] Final mesh status..." -ForegroundColor Yellow
$final0 = Send-Serial 5570 "mesh status" 5
$final1 = Send-Serial 5571 "mesh status" 5

Write-Host "  --- Node 0 ---"
foreach ($line in ($final0 -split "`n")) {
    if ($line -match "Mesh:|Consensus:|Federated:|peer|round|grad|10\.0") { Write-Host "  $($line.Trim())" }
}
Write-Host "  --- Node 1 ---"
foreach ($line in ($final1 -split "`n")) {
    if ($line -match "Mesh:|Consensus:|Federated:|peer|round|grad|10\.0") { Write-Host "  $($line.Trim())" }
}

# ── Cleanup ──
Write-Host "`n[10] Cleanup..." -ForegroundColor Yellow
Stop-Process -Id $p0.Id -Force -ErrorAction SilentlyContinue
Stop-Process -Id $p1.Id -Force -ErrorAction SilentlyContinue
Write-Host "  VMs stopped" -ForegroundColor Green

# ── Results ──
Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "  RESULTS" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
foreach ($r in $results) {
    $color = if ($r -match "PASS") { "Green" } else { "Red" }
    Write-Host "  $r" -ForegroundColor $color
}
Write-Host "========================================" -ForegroundColor Cyan
