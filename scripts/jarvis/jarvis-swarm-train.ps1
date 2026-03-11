<#
.SYNOPSIS
    JARVIS Distributed Swarm Training via Shared Network
.DESCRIPTION
    Boots N TrustOS nodes from ISO on a shared QEMU multicast L2 network.
    Each node auto-discovers peers via mesh (UDP 7700), elects a leader,
    and performs federated training, exchanging gradients every 30 seconds.
    Commands are sent to each VM via bidirectional TCP serial.
.NOTES
    Requires: QEMU, trustos.iso (run build-limine.ps1 first)
    Network:  QEMU socket listen/connect (shared L2, node 0 listens)
    Serial:   TCP serial per-node for command injection
              Base port = 5570 (node0=5570, node1=5571, ...)
#>

param(
    [string]$IsoPath   = "$PSScriptRoot\trustos.iso",
    [int]$Nodes        = 2,
    [int]$Epochs       = 2,
    [int]$BasePort     = 5570,
    [int]$BootTimeout  = 40,
    [switch]$NoBuild,
    [switch]$Headless
)

$ErrorActionPreference = "Continue"
$QemuExe = "C:\Program Files\qemu\qemu-system-x86_64.exe"
$LogDir  = "$PSScriptRoot\swarm_logs"
$Report  = "$PSScriptRoot\jarvis_swarm_results.txt"
$Display = if ($Headless) { "none" } else { "gtk" }
$MeshListenPort = 5580

# =============================================================================
# SERIAL COMMUNICATION
# =============================================================================

function Send-Cmd {
    param(
        [System.Net.Sockets.NetworkStream]$Stream,
        [string]$Cmd,
        [int]$Timeout = 10,
        [string]$WaitFor = ""
    )
    $buffer = New-Object byte[] 65536

    try { $null = $Stream.DataAvailable } catch { return "[CONNECTION LOST]" }

    # Drain pending data
    $sw0 = [System.Diagnostics.Stopwatch]::StartNew()
    while ($sw0.ElapsedMilliseconds -lt 500) {
        try {
            if ($Stream.DataAvailable) { $Stream.Read($buffer, 0, $buffer.Length) | Out-Null }
            else { Start-Sleep -Milliseconds 50 }
        } catch { break }
    }

    # Send command
    $bytes = [System.Text.Encoding]::ASCII.GetBytes("$Cmd`r")
    try {
        $Stream.Write($bytes, 0, $bytes.Length)
        $Stream.Flush()
    } catch {
        return "[CONNECTION LOST]"
    }

    # Read response
    $out = ""
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $lastData = 0

    while ($sw.Elapsed.TotalSeconds -lt $Timeout) {
        if ($Stream.DataAvailable) {
            $read = $Stream.Read($buffer, 0, $buffer.Length)
            if ($read -gt 0) {
                $out += [System.Text.Encoding]::ASCII.GetString($buffer, 0, $read)
                $lastData = $sw.ElapsedMilliseconds
            }
        } else {
            Start-Sleep -Milliseconds 50
            if ($out.Length -gt 20) {
                if ($WaitFor -and ($out -match $WaitFor)) {
                    Start-Sleep -Milliseconds 500
                    while ($Stream.DataAvailable) {
                        $read = $Stream.Read($buffer, 0, $buffer.Length)
                        if ($read -gt 0) { $out += [System.Text.Encoding]::ASCII.GetString($buffer, 0, $read) }
                    }
                    break
                }
                if (-not $WaitFor -and ($out -match "trustos:.*\$\s*$") -and (($sw.ElapsedMilliseconds - $lastData) -gt 1000)) {
                    break
                }
                if (-not $WaitFor -and $lastData -gt 0 -and (($sw.ElapsedMilliseconds - $lastData) -gt 5000)) {
                    break
                }
            }
        }
    }
    return $out
}

function Wait-Boot {
    param([System.Net.Sockets.NetworkStream]$Stream, [int]$Timeout = 40)
    $buffer = New-Object byte[] 65536
    $out = ""
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    while ($sw.Elapsed.TotalSeconds -lt $Timeout) {
        if ($Stream.DataAvailable) {
            $read = $Stream.Read($buffer, 0, $buffer.Length)
            if ($read -gt 0) { $out += [System.Text.Encoding]::ASCII.GetString($buffer, 0, $read) }
        } else { Start-Sleep -Milliseconds 100 }
        if ($out -match "trustos:.*\$") {
            Start-Sleep -Seconds 1
            break
        }
    }
    return $out
}

# =============================================================================
# MAIN
# =============================================================================

$startTime = Get-Date

Write-Host ""
Write-Host "======================================================" -ForegroundColor Magenta
Write-Host "  JARVIS DISTRIBUTED SWARM TRAINING" -ForegroundColor Magenta
Write-Host ("  {0} nodes x {1} epochs, shared L2 mesh network" -f $Nodes, $Epochs) -ForegroundColor DarkMagenta
Write-Host "======================================================" -ForegroundColor Magenta

# Verify prereqs
if (-not (Test-Path $IsoPath)) {
    Write-Host ("ERROR: No ISO at {0} - run build-limine.ps1 first" -f $IsoPath) -ForegroundColor Red
    exit 1
}
if (-not (Test-Path $QemuExe)) {
    Write-Host "ERROR: QEMU not found" -ForegroundColor Red
    exit 1
}

# Create log directory
if (Test-Path $LogDir) { Remove-Item -Recurse -Force $LogDir }
New-Item -ItemType Directory -Path $LogDir -Force | Out-Null

# Optional rebuild
if (-not $NoBuild) {
    Write-Host "`n[BUILD] Compiling kernel + ISO..." -ForegroundColor Yellow
    Push-Location $PSScriptRoot
    & .\build-limine.ps1
    Pop-Location
    if (-not (Test-Path $IsoPath)) {
        Write-Host "ERROR: Build failed" -ForegroundColor Red
        exit 1
    }
    Write-Host "[BUILD] Done" -ForegroundColor Green
}

# Kill existing QEMU
Stop-Process -Name qemu-system-x86_64 -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 2

# =============================================================================
# PHASE 1: Boot all nodes
# =============================================================================

Write-Host ("`n=== Phase 1: Booting {0} nodes on shared L2 ===" -f $Nodes) -ForegroundColor Cyan

$procs = @()
$tcpClients = @()
$streams = @()

for ($i = 0; $i -lt $Nodes; $i++) {
    $port = $BasePort + $i
    $macHex = "{0:X2}" -f ($i + 1)
    $mac = "52:54:00:00:01:$macHex"
    $serialArg = "tcp:127.0.0.1:${port},server,nowait"

    # Each node: ISO boot + shared L2 (socket) + unique MAC + TCP serial
    # Node 0 listens, all other nodes connect to node 0
    if ($i -eq 0) {
        $netArg = "socket,id=mesh0,listen=:$MeshListenPort"
    } else {
        $netArg = "socket,id=mesh0,connect=127.0.0.1:$MeshListenPort"
    }
    $qemuArgs = @(
        "-cdrom", $IsoPath,
        "-m", "256M",
        "-smp", "1",
        "-cpu", "Haswell",
        "-machine", "q35",
        "-accel", "whpx",
        "-display", $Display,
        "-serial", $serialArg,
        "-netdev", $netArg,
        "-device", "virtio-net-pci,netdev=mesh0,mac=$mac",
        "-no-reboot"
    )

    Write-Host ("  Node {0}: port={1} mac={2}" -f $i, $port, $mac) -NoNewline
    $proc = Start-Process -FilePath $QemuExe -ArgumentList $qemuArgs -PassThru -WindowStyle Hidden
    if ($proc) {
        $procs += $proc
        Write-Host (" PID={0}" -f $proc.Id) -ForegroundColor DarkGray
    } else {
        Write-Host " FAILED" -ForegroundColor Red
        $procs += $null
    }

    # Node 0 must be listening before others connect
    if ($i -eq 0) {
        Start-Sleep -Seconds 3
    } else {
        Start-Sleep -Seconds 1
    }
}

# Connect serial to each node
Write-Host "`n  Connecting serial ports..." -ForegroundColor DarkGray
Start-Sleep -Seconds 3

for ($i = 0; $i -lt $Nodes; $i++) {
    $port = $BasePort + $i
    $tcp = $null
    for ($retry = 0; $retry -lt 20; $retry++) {
        try { $tcp = New-Object System.Net.Sockets.TcpClient("127.0.0.1", $port); break }
        catch { Start-Sleep -Milliseconds 500 }
    }
    if ($tcp) {
        $tcpClients += $tcp
        $streams += $tcp.GetStream()
        Write-Host ("  Node {0} serial: OK" -f $i) -ForegroundColor Green
    } else {
        $tcpClients += $null
        $streams += $null
        Write-Host ("  Node {0} serial: FAILED" -f $i) -ForegroundColor Red
    }
}

# Wait for all nodes to boot
Write-Host "`n  Waiting for boot..." -ForegroundColor DarkGray
$bootResults = @()
for ($i = 0; $i -lt $Nodes; $i++) {
    if ($streams[$i]) {
        $bootOut = Wait-Boot -Stream $streams[$i] -Timeout $BootTimeout
        if ($bootOut -match "trustos") {
            Write-Host ("  Node {0}: BOOTED" -f $i) -ForegroundColor Green
            $bootResults += $true
        } else {
            Write-Host ("  Node {0}: TIMEOUT" -f $i) -ForegroundColor Red
            $bootResults += $false
        }
    } else {
        $bootResults += $false
    }
}

$liveNodes = @($bootResults | Where-Object { $_ }).Count
if ($liveNodes -eq 0) {
    Write-Host "`nERROR: No nodes booted successfully" -ForegroundColor Red
    foreach ($p in $procs) { if ($p) { Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue } }
    exit 1
}
Write-Host ("`n  {0}/{1} nodes ready" -f $liveNodes, $Nodes) -ForegroundColor Cyan

# =============================================================================
# PHASE 2: Start swarm training on all nodes
# =============================================================================

Write-Host "`n=== Phase 2: Starting swarm training ===" -ForegroundColor Cyan

# 45 min per epoch max (4.4M model is slow)
$pretrainTimeout = $Epochs * 2700

for ($i = 0; $i -lt $Nodes; $i++) {
    if (-not $bootResults[$i]) { continue }

    # Hardware scan for richer training data
    Write-Host ("  Node {0}: HW scan..." -f $i) -ForegroundColor DarkCyan
    $hwOut = Send-Cmd -Stream $streams[$i] -Cmd "jarvis boot" -Timeout 120 -WaitFor "Boot Scan Complete"
    if ($hwOut -match "Boot Scan Complete") {
        Write-Host "    HW scan: OK" -ForegroundColor Green
    } else {
        Write-Host "    HW scan: partial" -ForegroundColor Yellow
    }
    $hwOut | Out-File -FilePath ("$LogDir\node{0}_hwscan.log" -f $i) -Encoding UTF8
}

# Step-by-step setup (proven reliable): init -> mesh -> wait discovery -> federate -> train
Write-Host "`n  Initializing brains..." -ForegroundColor DarkCyan
for ($i = 0; $i -lt $Nodes; $i++) {
    if (-not $bootResults[$i]) { continue }
    $initOut = Send-Cmd -Stream $streams[$i] -Cmd "jarvis brain init" -Timeout 15 -WaitFor "brain ready"
    if ($initOut -match "brain ready") {
        Write-Host ("    Node {0}: brain ready" -f $i) -ForegroundColor Green
    } else {
        Write-Host ("    Node {0}: brain init issue" -f $i) -ForegroundColor Yellow
    }
}

Write-Host "`n  Starting mesh network..." -ForegroundColor DarkCyan
for ($i = 0; $i -lt $Nodes; $i++) {
    if (-not $bootResults[$i]) { continue }
    $meshOut = Send-Cmd -Stream $streams[$i] -Cmd "mesh start" -Timeout 10
    if ($meshOut -match "mesh.*started|Mesh.*active") {
        Write-Host ("    Node {0}: mesh active" -f $i) -ForegroundColor Green
    } else {
        Write-Host ("    Node {0}: mesh issue" -f $i) -ForegroundColor Yellow
    }
}

Write-Host "`n  Waiting 20s for auto-discovery..." -ForegroundColor DarkCyan
Start-Sleep -Seconds 20

for ($i = 0; $i -lt $Nodes; $i++) {
    if (-not $bootResults[$i]) { continue }
    $stOut = Send-Cmd -Stream $streams[$i] -Cmd "mesh status" -Timeout 5
    if ($stOut -match "Peers: (\d+) alive") {
        Write-Host ("    Node {0}: {1} peer(s)" -f $i, $Matches[1]) -ForegroundColor Green
    }
}

Write-Host "`n  Waiting 25s for leader election..." -ForegroundColor DarkCyan
Start-Sleep -Seconds 25

for ($i = 0; $i -lt $Nodes; $i++) {
    if (-not $bootResults[$i]) { continue }
    $leOut = Send-Cmd -Stream $streams[$i] -Cmd "mesh status" -Timeout 5
    if ($leOut -match "leader=true") {
        Write-Host ("    Node {0}: LEADER (elected)" -f $i) -ForegroundColor Cyan
    } elseif ($leOut -match "leader=false") {
        Write-Host ("    Node {0}: Worker" -f $i) -ForegroundColor DarkGray
    }
}

Write-Host "`n  Enabling federated learning..." -ForegroundColor DarkCyan
for ($i = 0; $i -lt $Nodes; $i++) {
    if (-not $bootResults[$i]) { continue }
    Send-Cmd -Stream $streams[$i] -Cmd "mesh federate on" -Timeout 5 | Out-Null
}
Write-Host "    Federated enabled on all nodes" -ForegroundColor Green

# Send pretrain to ALL nodes (quick send, they run in parallel)
Write-Host ("`n  Sending pretrain ({0} epochs) to all nodes..." -f $Epochs) -ForegroundColor DarkCyan
for ($i = 0; $i -lt $Nodes; $i++) {
    if (-not $bootResults[$i]) { continue }

    $bytes = [System.Text.Encoding]::ASCII.GetBytes("jarvis brain pretrain $Epochs`r")
    try {
        # Drain first
        $buffer = New-Object byte[] 65536
        $sw0 = [System.Diagnostics.Stopwatch]::StartNew()
        while ($sw0.ElapsedMilliseconds -lt 300) {
            if ($streams[$i].DataAvailable) { $streams[$i].Read($buffer, 0, $buffer.Length) | Out-Null }
            else { Start-Sleep -Milliseconds 50 }
        }
        $streams[$i].Write($bytes, 0, $bytes.Length)
        $streams[$i].Flush()
        Write-Host ("  Node {0}: swarm command sent" -f $i) -ForegroundColor Green
    } catch {
        Write-Host ("  Node {0}: send failed" -f $i) -ForegroundColor Red
    }
}

# =============================================================================
# PHASE 3: Monitor training progress
# =============================================================================

Write-Host "`n=== Phase 3: Monitoring distributed training ===" -ForegroundColor Cyan
Write-Host ("  All {0} nodes training in parallel with gradient exchange" -f $liveNodes) -ForegroundColor DarkGray

$buffer = New-Object byte[] 65536
$outputs = @{}
$finished = @{}
for ($i = 0; $i -lt $Nodes; $i++) { $outputs[$i] = ""; $finished[$i] = $false }

$trainSw = [System.Diagnostics.Stopwatch]::StartNew()
$lastStatus = 0

while ($trainSw.Elapsed.TotalSeconds -lt $pretrainTimeout) {
    $allDone = $true

    for ($i = 0; $i -lt $Nodes; $i++) {
        if (-not $bootResults[$i]) { continue }
        if ($finished[$i]) { continue }
        $allDone = $false

        if ($streams[$i].DataAvailable) {
            $read = $streams[$i].Read($buffer, 0, $buffer.Length)
            if ($read -gt 0) {
                $chunk = [System.Text.Encoding]::ASCII.GetString($buffer, 0, $read)
                $outputs[$i] += $chunk

                # Check for completion
                if ($outputs[$i] -match "Pre-training done|Swarm training complete") {
                    $finished[$i] = $true
                    Write-Host ("  Node {0}: DONE" -f $i) -ForegroundColor Green

                    if ($outputs[$i] -match 'Loss after:\s+([0-9.]+)') {
                        Write-Host ("    Final loss: {0}" -f $Matches[1]) -ForegroundColor Cyan
                    }
                    if ($outputs[$i] -match 'Mesh peers:\s+(\d+)') {
                        Write-Host ("    Mesh peers: {0}" -f $Matches[1]) -ForegroundColor Cyan
                    }
                }
            }
        }
    }

    # Periodic status every 30s
    $elapsed = [math]::Floor($trainSw.Elapsed.TotalSeconds)
    if (($elapsed - $lastStatus) -ge 30) {
        $lastStatus = $elapsed
        $doneCount = @($finished.Values | Where-Object { $_ }).Count
        Write-Host ("  [{0}s] {1}/{2} nodes complete" -f $elapsed, $doneCount, $liveNodes) -ForegroundColor DarkGray

        for ($i = 0; $i -lt $Nodes; $i++) {
            if ($bootResults[$i] -and -not $finished[$i]) {
                if ($outputs[$i] -match '(?s).*avg loss=([0-9.]+)') {
                    Write-Host ("    Node {0}: training... last avg_loss={1}" -f $i, $Matches[1]) -ForegroundColor DarkGray
                }
            }
        }
    }

    if ($allDone -or (@($finished.Values | Where-Object { $_ }).Count -eq $liveNodes)) { break }
    Start-Sleep -Milliseconds 100
}

# =============================================================================
# PHASE 4: Results
# =============================================================================

Write-Host "`n=== Phase 4: Results ===" -ForegroundColor Cyan

$totalTime = [math]::Round((Get-Date).Subtract($startTime).TotalSeconds)
$reportLines = @()
$reportLines += "JARVIS SWARM TRAINING REPORT"
$reportLines += "============================"
$reportLines += ("Date:    {0}" -f (Get-Date))
$reportLines += ("Nodes:   {0} ({1} active)" -f $Nodes, $liveNodes)
$reportLines += ("Epochs:  {0}" -f $Epochs)
$reportLines += ("Network: QEMU socket listen/connect port {0}" -f $MeshListenPort)
$reportLines += ("Total:   {0}s" -f $totalTime)
$reportLines += ""

Write-Host ""
Write-Host "  Node  | Status     | Final Loss       | Peers" -ForegroundColor White
Write-Host "  ------+------------+------------------+------" -ForegroundColor White

for ($i = 0; $i -lt $Nodes; $i++) {
    $status = "FAIL"
    if ($finished[$i]) { $status = "DONE" }
    elseif ($bootResults[$i]) { $status = "TIMEOUT" }
    $loss = "N/A"
    $peers = "N/A"

    if ($outputs[$i] -match 'Loss after:\s+([0-9.]+)') { $loss = $Matches[1] }
    if ($outputs[$i] -match 'Mesh peers:\s+(\d+)') { $peers = $Matches[1] }

    $statusColor = "Red"
    if ($status -eq "DONE") { $statusColor = "Green" }
    elseif ($status -eq "TIMEOUT") { $statusColor = "Yellow" }

    $line = "  {0,-5} | {1,-10} | {2,-16} | {3}" -f $i, $status, $loss, $peers
    Write-Host $line -ForegroundColor $statusColor

    $reportLines += ("Node {0}: {1}  loss={2}  peers={3}" -f $i, $status, $loss, $peers)

    $outputs[$i] | Out-File -FilePath ("$LogDir\node{0}_output.log" -f $i) -Encoding UTF8
}

Write-Host ""

$reportLines += ""
$reportLines += ("Duration: {0}s" -f $totalTime)
$reportLines | Out-File -FilePath $Report -Encoding UTF8

Write-Host ("  Total time: {0}s" -f $totalTime) -ForegroundColor Magenta
Write-Host "  Logs: $LogDir\" -ForegroundColor DarkGray
Write-Host "  Report: $Report" -ForegroundColor DarkGray

# Cleanup
Write-Host "`n  Shutting down VMs..." -ForegroundColor DarkGray
for ($i = 0; $i -lt $Nodes; $i++) {
    if ($streams[$i]) { try { $streams[$i].Close() } catch {} }
    if ($tcpClients[$i]) { try { $tcpClients[$i].Close() } catch {} }
}
foreach ($p in $procs) {
    if ($p -and -not $p.HasExited) {
        Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
    }
}

Write-Host ""
Write-Host "======================================================" -ForegroundColor Magenta
Write-Host "  SWARM TRAINING COMPLETE" -ForegroundColor Magenta
Write-Host "======================================================" -ForegroundColor Magenta
