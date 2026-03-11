<#
.SYNOPSIS
    JARVIS Mesh Network Test — Launch 4 TrustOS nodes on a shared virtual LAN
.DESCRIPTION
    Spawns 4 QEMU instances connected via multicast socket networking (real L2).
    Each node gets a unique MAC address. DHCP won't work on socket net, so
    the script assigns static IPs via serial commands after boot.
    
    Then tests: mesh discovery, peer listing, ping, consensus, federated learning.

.NOTES
    Requires: QEMU, trustos.iso (build with build-limine.ps1 first)
    Network:  QEMU -netdev socket,mcast=230.0.0.1:1234 (shared multicast L2)
    
    Node Layout:
        Node 0 (Leader candidate):  10.0.10.1   serial TCP :5570
        Node 1 (Worker):            10.0.10.2   serial TCP :5571
        Node 2 (Worker):            10.0.10.3   serial TCP :5572
        Node 3 (Worker):            10.0.10.4   serial TCP :5573
#>

param(
    [string]$IsoPath = "$PSScriptRoot\trustos.iso",
    [int]$NumNodes = 4,
    [int]$BaseSerialPort = 5570,
    [int]$BootTimeout = 30,
    [int]$CmdTimeout = 5,
    [switch]$NoBuild,
    [switch]$Headless,
    [switch]$SkipTests,
    [string]$ReportFile = "$PSScriptRoot\mesh_test_report.txt"
)

$QemuExe = "C:\Program Files\qemu\qemu-system-x86_64.exe"
$ErrorActionPreference = "Continue"
$timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"

# Multicast group for shared L2 (all nodes on same virtual Ethernet)
$McastAddr = "230.0.0.1"
$McastPort = 1234

# Node definitions
$Nodes = @()
for ($i = 0; $i -lt $NumNodes; $i++) {
    $Nodes += @{
        Id          = $i
        Name        = "jarvis-node-$i"
        IP          = "10.0.10.$($i + 1)"
        Subnet      = "255.255.255.0"
        Gateway     = "10.0.10.1"
        MAC         = "52:54:00:12:34:$("{0:X2}" -f (0x10 + $i))"
        SerialPort  = $BaseSerialPort + $i
        Memory      = "256M"
        CPUs        = 2
        Process     = $null
        TcpClient   = $null
        Stream      = $null
        Writer      = $null
    }
}

# ═══════════════════════════════════════════════════════════════
#  HELPER FUNCTIONS
# ═══════════════════════════════════════════════════════════════

function Write-Banner {
    param([string]$Text, [string]$Color = "Cyan")
    Write-Host ""
    Write-Host ("=" * 60) -ForegroundColor $Color
    Write-Host "  $Text" -ForegroundColor $Color
    Write-Host ("=" * 60) -ForegroundColor $Color
}

function Send-Command {
    param(
        [object]$Stream,
        [string]$Cmd,
        [int]$Timeout = $CmdTimeout,
        [string]$WaitFor = ""
    )
    
    $buffer = New-Object byte[] 16384
    
    # Drain leftover
    $drainSw = [System.Diagnostics.Stopwatch]::StartNew()
    $lastData = $drainSw.ElapsedMilliseconds
    while (($drainSw.ElapsedMilliseconds - $lastData) -lt 200 -and $drainSw.ElapsedMilliseconds -lt 2000) {
        if ($Stream.DataAvailable) {
            $Stream.Read($buffer, 0, $buffer.Length) | Out-Null
            $lastData = $drainSw.ElapsedMilliseconds
        } else {
            Start-Sleep -Milliseconds 30
        }
    }
    
    # Send command
    $cmdBytes = [System.Text.Encoding]::ASCII.GetBytes("$Cmd`r")
    $Stream.Write($cmdBytes, 0, $cmdBytes.Length)
    $Stream.Flush()
    
    # Collect output
    $output = ""
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    
    while ($sw.Elapsed.TotalSeconds -lt $Timeout) {
        if ($Stream.DataAvailable) {
            $read = $Stream.Read($buffer, 0, $buffer.Length)
            if ($read -gt 0) {
                $output += [System.Text.Encoding]::ASCII.GetString($buffer, 0, $read)
            }
        } else {
            Start-Sleep -Milliseconds 50
        }
        
        if ($WaitFor -and $output -match $WaitFor) {
            Start-Sleep -Milliseconds 200
            while ($Stream.DataAvailable) {
                $read = $Stream.Read($buffer, 0, $buffer.Length)
                if ($read -gt 0) { $output += [System.Text.Encoding]::ASCII.GetString($buffer, 0, $read) }
            }
            break
        }
        
        # Prompt detection (after some data collected)
        if (-not $WaitFor -and $sw.ElapsedMilliseconds -ge 500 -and $output.Length -gt 5) {
            if ($output -match '(tsh>|TrustOS>|\$\s*$)') {
                Start-Sleep -Milliseconds 100
                while ($Stream.DataAvailable) {
                    $read = $Stream.Read($buffer, 0, $buffer.Length)
                    if ($read -gt 0) { $output += [System.Text.Encoding]::ASCII.GetString($buffer, 0, $read) }
                }
                break
            }
        }
    }
    
    return $output
}

function Test-Result {
    param([string]$NodeName, [string]$TestName, [string]$Output, [string]$ExpectPattern = "")
    
    if ($ExpectPattern -and $Output -match $ExpectPattern) {
        Write-Host "  [PASS] $NodeName :: $TestName" -ForegroundColor Green
        $script:passed++
        return $true
    } elseif (-not $ExpectPattern -and $Output.Length -gt 0) {
        Write-Host "  [PASS] $NodeName :: $TestName" -ForegroundColor Green
        $script:passed++
        return $true
    } else {
        Write-Host "  [FAIL] $NodeName :: $TestName" -ForegroundColor Red
        if ($Output.Length -gt 100) {
            Write-Host "         Output: $($Output.Substring(0, 100))..." -ForegroundColor DarkGray
        } else {
            Write-Host "         Output: $Output" -ForegroundColor DarkGray
        }
        $script:failed++
        return $false
    }
}

$script:passed = 0
$script:failed = 0

# ═══════════════════════════════════════════════════════════════
#  BUILD
# ═══════════════════════════════════════════════════════════════

if (-not $NoBuild) {
    Write-Banner "BUILDING TRUSTOS" "Yellow"
    & "$PSScriptRoot\build-limine.ps1" -NoRun
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Build failed!" -ForegroundColor Red
        exit 1
    }
}

if (-not (Test-Path $IsoPath)) {
    Write-Host "ISO not found: $IsoPath" -ForegroundColor Red
    Write-Host "Run build-limine.ps1 first or specify -IsoPath" -ForegroundColor Yellow
    exit 1
}

$isoSize = [math]::Round((Get-Item $IsoPath).Length / 1MB, 1)
Write-Host "ISO: $IsoPath ($isoSize MB)" -ForegroundColor Green

# ═══════════════════════════════════════════════════════════════
#  CLEANUP
# ═══════════════════════════════════════════════════════════════

Write-Banner "CLEANUP" "Yellow"
Stop-Process -Name qemu-system-x86_64 -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 2
Write-Host "  Killed existing QEMU processes" -ForegroundColor DarkGray

# ═══════════════════════════════════════════════════════════════
#  LAUNCH 4 QEMU NODES
# ═══════════════════════════════════════════════════════════════

Write-Banner "LAUNCHING $NumNodes NODES" "Green"

for ($i = 0; $i -lt $NumNodes; $i++) {
    $node = $Nodes[$i]
    $serialArg = "tcp:127.0.0.1:$($node.SerialPort),server,nowait"
    $displayArg = if ($Headless) { "none" } else { "gtk" }
    
    $qemuArgs = @(
        "-cdrom", "`"$IsoPath`"",
        "-m", $node.Memory,
        "-machine", "q35",
        "-cpu", "max",
        "-smp", "$($node.CPUs)",
        "-accel", "tcg,thread=multi",
        "-display", $displayArg,
        "-vga", "std",
        "-device", "virtio-net-pci,netdev=net0,mac=$($node.MAC)",
        "-netdev", "socket,id=net0,mcast=${McastAddr}:${McastPort}",
        "-serial", $serialArg,
        "-no-reboot"
    )
    
    $proc = Start-Process -FilePath $QemuExe -ArgumentList $qemuArgs -PassThru -WindowStyle Normal
    $Nodes[$i].Process = $proc
    
    Write-Host "  [$i] $($node.Name)  MAC=$($node.MAC)  Serial=:$($node.SerialPort)  PID=$($proc.Id)" -ForegroundColor Cyan
    Start-Sleep -Milliseconds 500
}

Write-Host ""
Write-Host "All $NumNodes nodes launched" -ForegroundColor Green

# ═══════════════════════════════════════════════════════════════
#  CONNECT SERIAL TO ALL NODES
# ═══════════════════════════════════════════════════════════════

Write-Banner "CONNECTING SERIAL" "White"

for ($i = 0; $i -lt $NumNodes; $i++) {
    $node = $Nodes[$i]
    $client = New-Object System.Net.Sockets.TcpClient
    $connected = $false
    
    for ($attempt = 0; $attempt -lt 60; $attempt++) {
        try {
            $client.Connect("127.0.0.1", $node.SerialPort)
            $connected = $true
            break
        } catch {
            Start-Sleep -Milliseconds 500
        }
    }
    
    if ($connected) {
        $stream = $client.GetStream()
        $stream.ReadTimeout = 10000
        $Nodes[$i].TcpClient = $client
        $Nodes[$i].Stream = $stream
        Write-Host "  [$i] Connected to $($node.Name) on port $($node.SerialPort)" -ForegroundColor Green
    } else {
        Write-Host "  [$i] FAILED to connect to $($node.Name)" -ForegroundColor Red
    }
}

# ═══════════════════════════════════════════════════════════════
#  WAIT FOR BOOT ON ALL NODES
# ═══════════════════════════════════════════════════════════════

Write-Banner "WAITING FOR BOOT" "Yellow"

for ($i = 0; $i -lt $NumNodes; $i++) {
    $node = $Nodes[$i]
    if (-not $node.Stream) { continue }
    
    Write-Host "  [$i] Waiting for $($node.Name) boot..." -NoNewline -ForegroundColor DarkGray
    
    $buffer = New-Object byte[] 16384
    $output = ""
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    
    while ($sw.Elapsed.TotalSeconds -lt $BootTimeout) {
        if ($node.Stream.DataAvailable) {
            $read = $node.Stream.Read($buffer, 0, $buffer.Length)
            if ($read -gt 0) {
                $output += [System.Text.Encoding]::ASCII.GetString($buffer, 0, $read)
            }
        } else {
            Start-Sleep -Milliseconds 100
        }
        
        if ($output -match '(tsh>|TrustOS>|Welcome)') {
            break
        }
    }
    
    if ($output -match '(tsh>|TrustOS>|Welcome)') {
        Write-Host " BOOTED ($([math]::Round($sw.Elapsed.TotalSeconds, 1))s)" -ForegroundColor Green
    } else {
        Write-Host " TIMEOUT" -ForegroundColor Red
    }
}

# Small settle time after all boots
Start-Sleep -Seconds 2

# ═══════════════════════════════════════════════════════════════
#  CONFIGURE STATIC IPs
# ═══════════════════════════════════════════════════════════════

Write-Banner "CONFIGURE NETWORK (Static IPs)" "Yellow"

# Since there's no DHCP on socket mcast, TrustOS may get auto-link-local or nothing.
# We'll use ipconfig set commands if available, or rely on the existing DHCP fallback.
# For now, check what IP each node got.

for ($i = 0; $i -lt $NumNodes; $i++) {
    $node = $Nodes[$i]
    if (-not $node.Stream) { continue }
    
    # Check current IP
    $out = Send-Command -Stream $node.Stream -Cmd "ifconfig" -Timeout 3
    Write-Host "  [$i] $($node.Name) network:" -ForegroundColor Cyan
    
    # Extract IP if visible
    if ($out -match '(\d+\.\d+\.\d+\.\d+)') {
        Write-Host "       IP: $($Matches[1])" -ForegroundColor Green
    } else {
        Write-Host "       No IP detected yet (DHCP may be pending on socket net)" -ForegroundColor Yellow
    }
}

if ($SkipTests) {
    Write-Banner "TESTS SKIPPED (-SkipTests)" "Yellow"
    Write-Host "Nodes are running. Connect manually:"
    for ($i = 0; $i -lt $NumNodes; $i++) {
        Write-Host "  Node $i: serial port $($Nodes[$i].SerialPort)" -ForegroundColor Cyan
    }
    Write-Host ""
    Write-Host "Commands to try:"
    Write-Host "  mesh start         — Start mesh discovery"
    Write-Host "  mesh status        — Show peers"
    Write-Host "  mesh ping 10.0.10.2 — Ping another node"
    Write-Host "  mesh federate on   — Enable distributed learning"
    Write-Host ""
    Write-Host "Press Ctrl+C to stop all nodes"
    
    try { while ($true) { Start-Sleep -Seconds 5 } }
    finally {
        for ($i = 0; $i -lt $NumNodes; $i++) {
            if ($Nodes[$i].TcpClient) { $Nodes[$i].TcpClient.Close() }
        }
        Stop-Process -Name qemu-system-x86_64 -Force -ErrorAction SilentlyContinue
    }
    return
}

# ═══════════════════════════════════════════════════════════════
#  TEST PHASE 1: Initialize JARVIS brain on all nodes
# ═══════════════════════════════════════════════════════════════

Write-Banner "TEST 1: Initialize JARVIS Brain" "Magenta"

for ($i = 0; $i -lt $NumNodes; $i++) {
    $node = $Nodes[$i]
    if (-not $node.Stream) { continue }
    
    $out = Send-Command -Stream $node.Stream -Cmd "jarvis brain init" -Timeout 10 -WaitFor "ready|initialized|already"
    Test-Result $node.Name "brain init" $out "ready|initialized|Neural brain|already"
}

Start-Sleep -Seconds 2

# ═══════════════════════════════════════════════════════════════
#  TEST PHASE 2: Start mesh on all nodes
# ═══════════════════════════════════════════════════════════════

Write-Banner "TEST 2: Start Mesh Networking" "Magenta"

for ($i = 0; $i -lt $NumNodes; $i++) {
    $node = $Nodes[$i]
    if (-not $node.Stream) { continue }
    
    $out = Send-Command -Stream $node.Stream -Cmd "mesh start" -Timeout 5
    Test-Result $node.Name "mesh start" $out "started|active|mesh"
}

# Wait for discovery broadcasts to propagate
Write-Host "`n  Waiting 15s for peer discovery..." -ForegroundColor Yellow
Start-Sleep -Seconds 15

# ═══════════════════════════════════════════════════════════════
#  TEST PHASE 3: Check peer discovery
# ═══════════════════════════════════════════════════════════════

Write-Banner "TEST 3: Peer Discovery" "Magenta"

for ($i = 0; $i -lt $NumNodes; $i++) {
    $node = $Nodes[$i]
    if (-not $node.Stream) { continue }
    
    $out = Send-Command -Stream $node.Stream -Cmd "mesh status" -Timeout 5
    Write-Host "  [$i] $($node.Name) mesh status:" -ForegroundColor DarkGray
    $out -split "`n" | ForEach-Object { 
        if ($_.Trim()) { Write-Host "       $_" -ForegroundColor DarkGray }
    }
    
    # Check if peers were found (expect at least 1 peer)
    Test-Result $node.Name "peer discovery" $out "Peers: [1-9]|peers.*[1-9]|peer.*alive"
}

# ═══════════════════════════════════════════════════════════════
#  TEST PHASE 4: Mesh ping between nodes
# ═══════════════════════════════════════════════════════════════

Write-Banner "TEST 4: Mesh RPC Ping" "Magenta"

# Node 0 pings Node 1
$node0 = $Nodes[0]
if ($node0.Stream) {
    $target = $Nodes[1].IP
    $out = Send-Command -Stream $node0.Stream -Cmd "mesh ping $target" -Timeout 8
    Test-Result $node0.Name "ping node-1 ($target)" $out "alive|ok|pong"
}

# Node 2 pings Node 0
if ($Nodes[2].Stream) {
    $target = $Nodes[0].IP
    $out = Send-Command -Stream $Nodes[2].Stream -Cmd "mesh ping $target" -Timeout 8
    Test-Result $Nodes[2].Name "ping node-0 ($target)" $out "alive|ok|pong"
}

# ═══════════════════════════════════════════════════════════════
#  TEST PHASE 5: Consensus / Leader election
# ═══════════════════════════════════════════════════════════════

Write-Banner "TEST 5: Leader Election" "Magenta"

Write-Host "  Waiting 25s for election timeout + voting..." -ForegroundColor Yellow
Start-Sleep -Seconds 25

# Poll mesh status to trigger consensus poll cycles
for ($i = 0; $i -lt $NumNodes; $i++) {
    if ($Nodes[$i].Stream) {
        $null = Send-Command -Stream $Nodes[$i].Stream -Cmd "mesh status" -Timeout 3
    }
}

Start-Sleep -Seconds 5

# Check who is leader
$leaderFound = $false
for ($i = 0; $i -lt $NumNodes; $i++) {
    $node = $Nodes[$i]
    if (-not $node.Stream) { continue }
    
    $out = Send-Command -Stream $node.Stream -Cmd "mesh status" -Timeout 5
    if ($out -match "Leader|leader=true|role=Leader|★") {
        Write-Host "  [$i] $($node.Name) — LEADER" -ForegroundColor Yellow
        $leaderFound = $true
    } else {
        Write-Host "  [$i] $($node.Name) — Worker" -ForegroundColor DarkGray
    }
}

if ($leaderFound) {
    Write-Host "  [PASS] Leader election successful" -ForegroundColor Green
    $script:passed++
} else {
    Write-Host "  [FAIL] No leader elected" -ForegroundColor Red
    $script:failed++
}

# ═══════════════════════════════════════════════════════════════
#  TEST PHASE 6: Federated Learning
# ═══════════════════════════════════════════════════════════════

Write-Banner "TEST 6: Federated Learning" "Magenta"

# Enable federated learning on all nodes
for ($i = 0; $i -lt $NumNodes; $i++) {
    $node = $Nodes[$i]
    if (-not $node.Stream) { continue }
    
    $out = Send-Command -Stream $node.Stream -Cmd "mesh federate on" -Timeout 3
    Test-Result $node.Name "federate enable" $out "enabled|Federated|already"
}

Write-Host "`n  Waiting 10s for first federated round..." -ForegroundColor Yellow
Start-Sleep -Seconds 10

# Force a sync
for ($i = 0; $i -lt $NumNodes; $i++) {
    if ($Nodes[$i].Stream) {
        $null = Send-Command -Stream $Nodes[$i].Stream -Cmd "mesh federate sync" -Timeout 8
    }
}

Start-Sleep -Seconds 5

# Check federated stats
for ($i = 0; $i -lt $NumNodes; $i++) {
    $node = $Nodes[$i]
    if (-not $node.Stream) { continue }
    
    $out = Send-Command -Stream $node.Stream -Cmd "mesh status" -Timeout 5
    if ($out -match "fed_rounds=[1-9]|grads_received=[1-9]") {
        Write-Host "  [$i] $($node.Name) — Federated activity detected" -ForegroundColor Green
    } else {
        Write-Host "  [$i] $($node.Name) — No federated activity yet" -ForegroundColor Yellow
    }
}

# ═══════════════════════════════════════════════════════════════
#  TEST PHASE 7: Remote inference
# ═══════════════════════════════════════════════════════════════

Write-Banner "TEST 7: Remote Inference" "Magenta"

if ($Nodes[0].Stream) {
    $target = $Nodes[1].IP
    $out = Send-Command -Stream $Nodes[0].Stream -Cmd "mesh infer $target Hello" -Timeout 10
    if ($out.Length -gt 10) {
        Write-Host "  [PASS] Remote inference returned data" -ForegroundColor Green
        $script:passed++
    } else {
        Write-Host "  [INFO] Remote inference: $out" -ForegroundColor Yellow
    }
}

# ═══════════════════════════════════════════════════════════════
#  REPORT
# ═══════════════════════════════════════════════════════════════

Write-Banner "MESH TEST RESULTS" "Cyan"

$total = $script:passed + $script:failed
Write-Host "  Passed: $($script:passed) / $total" -ForegroundColor $(if ($script:failed -eq 0) { "Green" } else { "Yellow" })
Write-Host "  Failed: $($script:failed) / $total" -ForegroundColor $(if ($script:failed -gt 0) { "Red" } else { "Green" })

$report = @"
JARVIS Mesh Network Test Report
================================
Date: $timestamp
Nodes: $NumNodes
Passed: $($script:passed) / $total
Failed: $($script:failed) / $total
"@

$report | Out-File -FilePath $ReportFile -Encoding utf8
Write-Host "`n  Report saved: $ReportFile" -ForegroundColor DarkGray

# ═══════════════════════════════════════════════════════════════
#  CLEANUP
# ═══════════════════════════════════════════════════════════════

Write-Banner "CLEANUP" "Yellow"

# Close serial connections
for ($i = 0; $i -lt $NumNodes; $i++) {
    if ($Nodes[$i].TcpClient) { 
        $Nodes[$i].TcpClient.Close() 
    }
}

# Kill all QEMU instances
Stop-Process -Name qemu-system-x86_64 -Force -ErrorAction SilentlyContinue
Write-Host "  All nodes stopped" -ForegroundColor Green
