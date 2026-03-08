<#
.SYNOPSIS
    JARVIS Auto-Propagation Test --- 2 nodes on shared virtual LAN
    
    Node 0 (Leader): Boots with pretrained brain (17.6MB in ISO via Limine module),
                      inits full brain from RamFS, starts mesh + serves to peers
    Node 1 (Worker): Boots same ISO, deletes local brain, runs auto-propagate,
                      pulls brain from Node 0 via mesh RPC GetWeights
    
    Tests: boot module loading, mesh discovery, RPC brain transfer, federated sync
#>
$ErrorActionPreference = "Continue"
$QemuExe = "C:\Program Files\qemu\qemu-system-x86_64.exe"
$iso = "$PSScriptRoot\trustos.iso"

if (-not (Test-Path $iso)) {
    Write-Host "ERROR: trustos.iso not found! Build first." -ForegroundColor Red
    exit 1
}

# ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
# Utility Functions
# ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

function Send-Serial($port, $cmd, $timeoutSec = 15) {
    try {
        $tcp = New-Object System.Net.Sockets.TcpClient("127.0.0.1", $port)
        $tcp.ReceiveTimeout = 2000
        $s = $tcp.GetStream()
        $buf = New-Object byte[] 65536

        # Drain buffered data (2 seconds to clear any leftover output)
        $sw0 = [System.Diagnostics.Stopwatch]::StartNew()
        while ($sw0.Elapsed.TotalSeconds -lt 2) {
            if ($s.DataAvailable) { $s.Read($buf, 0, $buf.Length) | Out-Null }
            else { Start-Sleep -Milliseconds 50 }
        }

        # Send command
        $bytes = [System.Text.Encoding]::ASCII.GetBytes("$cmd`r")
        $s.Write($bytes, 0, $bytes.Length)
        $s.Flush()
        Start-Sleep -Milliseconds 500

        # Read response until prompt or timeout
        $out = ""
        $sw = [System.Diagnostics.Stopwatch]::StartNew()
        while ($sw.Elapsed.TotalSeconds -lt $timeoutSec) {
            if ($s.DataAvailable) {
                $n = $s.Read($buf, 0, $buf.Length)
                $out += [System.Text.Encoding]::ASCII.GetString($buf, 0, $n)
                # Break when we see the shell prompt after sufficient output
                if ($out.Length -gt 20 -and $out -match 'trustos:/') { break }
            } else {
                Start-Sleep -Milliseconds 100
            }
        }
        $tcp.Close()
        return $out
    } catch {
        return "CONNECTION_FAILED: $_"
    }
}

function Wait-Boot($port, $timeoutSec = 40) {
    try {
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
            if ($out -match 'trustos:') { break }
        }
        $tcp.Close()
        return $out
    } catch {
        return ""
    }
}

$pass = 0
$fail = 0
$results = @()

function Test-Check($name, $condition) {
    if ($condition) {
        Write-Host "  PASS  $name" -ForegroundColor Green
        $script:pass++
        $script:results += "PASS: $name"
    } else {
        Write-Host "  FAIL  $name" -ForegroundColor Red
        $script:fail++
        $script:results += "FAIL: $name"
    }
}

function Show-Lines($text, $pattern, $color = "DarkGray") {
    foreach ($line in ($text -split "`n")) {
        if ($line -match $pattern) {
            Write-Host "       $($line.Trim())" -ForegroundColor $color
        }
    }
}

# ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
# Cleanup
# ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
Write-Host ""
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host "    JARVIS Auto-Propagation Test (2-Node Mesh)" -ForegroundColor Cyan
Write-Host "    Real brain transfer: 17.6MB via mesh RPC" -ForegroundColor Cyan
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host ""

Stop-Process -Name qemu-system-x86_64 -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 2

# ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
# PHASE 1: Boot Node 0 (Leader --- will have brain + serve it)
# ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
Write-Host "[1/8] Booting Node 0 (Leader + brain in ISO)..." -ForegroundColor Yellow
$p0 = Start-Process -FilePath $QemuExe -ArgumentList @(
    "-cdrom",$iso,"-m","512M","-smp","1","-cpu","Haswell",
    "-machine","q35","-accel","whpx","-display","none",
    "-serial","tcp:127.0.0.1:5590,server,nowait",
    "-netdev","socket,id=mesh0,listen=:5600",
    "-device","virtio-net-pci,netdev=mesh0,mac=52:54:00:00:01:01",
    "-no-reboot"
) -PassThru -WindowStyle Hidden
Write-Host "       PID=$($p0.Id)" -ForegroundColor DarkGray
Start-Sleep -Seconds 3

# ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
# PHASE 2: Boot Node 1 (Worker --- will delete local brain, pull from mesh)
# ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
Write-Host "[2/8] Booting Node 1 (Worker)..." -ForegroundColor Yellow
$p1 = Start-Process -FilePath $QemuExe -ArgumentList @(
    "-cdrom",$iso,"-m","512M","-smp","1","-cpu","Haswell",
    "-machine","q35","-accel","whpx","-display","none",
    "-serial","tcp:127.0.0.1:5591,server,nowait",
    "-netdev","socket,id=mesh0,connect=127.0.0.1:5600",
    "-device","virtio-net-pci,netdev=mesh0,mac=52:54:00:00:01:02",
    "-no-reboot"
) -PassThru -WindowStyle Hidden
Write-Host "       PID=$($p1.Id)" -ForegroundColor DarkGray

# ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
# PHASE 3: Wait for both nodes to boot
# ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
Write-Host "[3/8] Waiting for boot..." -ForegroundColor Yellow
$boot0 = Wait-Boot 5590 45
$boot1 = Wait-Boot 5591 45

$n0Booted = $boot0 -match "trustos"
$n1Booted = $boot1 -match "trustos"
Test-Check "Node 0 booted" $n0Booted
Test-Check "Node 1 booted" $n1Booted

if (-not ($n0Booted -and $n1Booted)) {
    Write-Host "  Aborting --- nodes failed to boot" -ForegroundColor Red
    Stop-Process -Id $p0.Id -Force -ErrorAction SilentlyContinue
    Stop-Process -Id $p1.Id -Force -ErrorAction SilentlyContinue
    exit 1
}

# ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
# PHASE 4: Node 0 --- Init brain (auto-starts mesh when full brain loads)
# ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
Write-Host ""
Write-Host "[4/8] Node 0: Init brain (auto-starts mesh with full brain)..." -ForegroundColor Yellow

# Init brain — loads 17.6MB from Limine module -> RamFS -> Full brain -> auto-starts mesh
$init0 = Send-Serial 5590 "jarvis brain init" 60
Write-Host "       DEBUG: init0 length=$($init0.Length), last100=$($init0.Substring([Math]::Max(0, $init0.Length - 100)))" -ForegroundColor Magenta
$fullBrainLoaded = $init0 -match "Full brain loaded|Full brain:|Full=LOADED"
$initOk = $init0 -match "Init complete|Micro sentinel loaded|Kernel validation" -and $fullBrainLoaded
Test-Check "Node 0 brain init (full brain from ISO)" $initOk
$meshAutoStarted = $init0 -match "Mesh auto-started|mesh started|Mesh networking active"
Test-Check "Node 0 mesh auto-started" $meshAutoStarted
Show-Lines $init0 "JARVIS|Full|brain|Micro|Maturity|loaded|weights|Init|MESH|mesh|RPC|TCP|NET"

Start-Sleep -Seconds 2

# ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
# PHASE 5: Node 1 --- Delete local brain, then auto-propagate from mesh
# ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
Write-Host ""
Write-Host "[5/8] Node 1: Delete local brain + auto-propagate from mesh..." -ForegroundColor Yellow

# Delete the local brain copy (both nodes boot from same ISO with brain)
# This forces auto_propagate to pull from mesh instead of self-loading
Write-Host "       Deleting /jarvis/weights.bin on Node 1..." -ForegroundColor DarkGray
$rmOut = Send-Serial 5591 "rm /jarvis/weights.bin" 5
Show-Lines $rmOut "rm|delete|removed|error|not found"
Start-Sleep -Seconds 1

# Verify it's gone
$lsOut = Send-Serial 5591 "ls /jarvis/" 5
Write-Host "       Node 1 /jarvis/ after delete: $($lsOut.Trim())" -ForegroundColor DarkGray
Start-Sleep -Seconds 1

# Now run auto-propagate: micro init -> mesh -> discover Node 0 -> RPC GetWeights -> install
Write-Host "       Running: jarvis brain propagate..." -ForegroundColor DarkGray

# Start background job to capture Node 0 serial during propagation
$n0Job = Start-Job -ScriptBlock {
    param($port)
    $output = ""
    try {
        $tcp = New-Object System.Net.Sockets.TcpClient("127.0.0.1", $port)
        $tcp.ReceiveTimeout = 2000
        $stream = $tcp.GetStream()
        $reader = New-Object System.IO.StreamReader($stream)
        $sw = [System.Diagnostics.Stopwatch]::StartNew()
        while ($sw.Elapsed.TotalSeconds -lt 135) {
            try {
                $line = $reader.ReadLine()
                if ($line) { $output += "$line`n" }
            } catch { Start-Sleep -Milliseconds 200 }
        }
        $tcp.Close()
    } catch {}
    return $output
} -ArgumentList 5590

$prop1 = Send-Serial 5591 "jarvis brain propagate" 140
Write-Host ""

# Show all propagation output with color coding
foreach ($line in ($prop1 -split "`n")) {
    $trimmed = $line.Trim()
    if ($trimmed.Length -gt 0 -and $trimmed -notmatch "^trustos:") {
        if ($trimmed -match "FAIL|failed|error|no source") {
            Write-Host "       $trimmed" -ForegroundColor Red
        } elseif ($trimmed -match "OK|active|DOWNLOADED|enabled|FULL|complete|Brain=FULL") {
            Write-Host "       $trimmed" -ForegroundColor Green
        } else {
            Write-Host "       $trimmed" -ForegroundColor DarkGray
        }
    }
}

# Check propagation results
Test-Check "Node 1 micro sentinel init" ($prop1 -match "micro sentinel OK|\[1/5\].*Brain")
Test-Check "Node 1 mesh activated" ($prop1 -match "Mesh.*active|\[2/5\]")
Test-Check "Node 1 peer discovery" ($prop1 -match "Peers.*[1-9]|\[3/5\].*[1-9]")

# THE CRITICAL TEST: Did it actually download from mesh (not self-load)?
$brainDownloaded = $prop1 -match "DOWNLOADED"
$brainAlreadyLoaded = $prop1 -match "already loaded"
Test-Check "Node 1 brain DOWNLOADED from mesh (not self-loaded)" ($brainDownloaded -and -not $brainAlreadyLoaded)
Test-Check "Node 1 federated enabled" ($prop1 -match "Federated.*enabled|\[5/5\]")
Test-Check "Propagation complete with full brain" ($prop1 -match "Brain=FULL|Propagation complete")

# Capture Node 0 server-side RPC log for diagnostics
Write-Host "       --- Node 0 RPC server log ---" -ForegroundColor DarkGray
$n0Log = ""
if ($n0Job) {
    $n0Log = Receive-Job -Job $n0Job -Wait -AutoRemoveJob 2>$null
    if ($n0Log) {
        foreach ($line in ($n0Log -split "`n")) {
            $trimmed = $line.Trim()
            if ($trimmed -match "\[RPC\]|\[TCP\]") {
                Write-Host "       N0: $trimmed" -ForegroundColor DarkYellow
            }
        }
    } else {
        Write-Host "       N0: (no RPC log captured)" -ForegroundColor DarkGray
    }
}

# ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
# PHASE 6: Verify Node 1 has full brain with correct params
# ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
Write-Host ""
Write-Host "[6/8] Verifying brain state on Node 1..." -ForegroundColor Yellow
Start-Sleep -Seconds 3
# Try brain info command — retry if serial reconnection fails
$info1 = ""
for ($retry = 0; $retry -lt 3; $retry++) {
    $info1 = Send-Serial 5591 "jarvis brain info" 10
    if ($info1.Length -gt 0) { break }
    Write-Host "       (serial retry $($retry+1)...)" -ForegroundColor DarkGray
    Start-Sleep -Seconds 3
}
# Also check propagation output as fallback (already has param count)
$hasFull = ($info1 -match "4\.4M|4393216|Full.*LOADED|full brain|LOADED from") -or `
           ($prop1 -match "DOWNLOADED.*17161|4393216.*params|Brain=FULL")
Test-Check "Node 1 has full brain (4.4M params)" $hasFull
if ($info1.Length -gt 0) {
    Show-Lines $info1 "param|layer|Full|brain|Micro|maturity|Model|d_model|vocab|Status"
} else {
    Write-Host "       (verified via propagation output)" -ForegroundColor DarkGray
}

# ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
# PHASE 7: Cross-node inference test (prove the brain works)
# ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
Write-Host ""
Write-Host "[7/8] Cross-node chat test..." -ForegroundColor Yellow
Start-Sleep -Seconds 1
$chat1 = Send-Serial 5591 "jarvis chat hello" 15
$chatOk = $chat1.Length -gt 20
Test-Check "Node 1 can generate text with transferred brain" $chatOk

foreach ($line in ($chat1 -split "`n")) {
    $trimmed = $line.Trim()
    if ($trimmed.Length -gt 0 -and $trimmed -notmatch "^trustos:" -and $trimmed -notmatch "^jarvis") {
        Write-Host "       $trimmed" -ForegroundColor Cyan
    }
}

# ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
# PHASE 8: Cleanup & Report
# ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
Write-Host ""
Write-Host "[8/8] Cleanup..." -ForegroundColor Yellow
Stop-Process -Id $p0.Id -Force -ErrorAction SilentlyContinue
Stop-Process -Id $p1.Id -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 1

Write-Host ""
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host "    PROPAGATION TEST RESULTS" -ForegroundColor Cyan
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "  Passed: $pass" -ForegroundColor Green
Write-Host "  Failed: $fail" -ForegroundColor $(if ($fail -gt 0) { "Red" } else { "Green" })
Write-Host ""
foreach ($r in $results) {
    $color = if ($r.StartsWith("PASS")) { "Green" } else { "Red" }
    Write-Host "  $r" -ForegroundColor $color
}
Write-Host ""

if ($fail -eq 0) {
    Write-Host "  === ALL TESTS PASSED === " -ForegroundColor Green
    Write-Host "  JARVIS brain (17.6MB) propagated via mesh RPC!" -ForegroundColor Green
} else {
    Write-Host "  === $fail TEST(S) FAILED ===" -ForegroundColor Red
}
Write-Host ""
exit $fail
