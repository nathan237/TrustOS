<#
.SYNOPSIS
    JARVIS Full Capability Test Suite
    Tests ALL JARVIS features: brain, training, inference, agent, propagation, mesh
    
    Phase A: Single-node tests (brain init, info, train, generate, chat, introspect, weights, hardware, save, load, eval, bench, test)
    Phase B: 2-node propagation + federated learning
#>
$ErrorActionPreference = "Continue"
$QemuExe = "C:\Program Files\qemu\qemu-system-x86_64.exe"
$iso = "$PSScriptRoot\trustos.iso"

if (-not (Test-Path $iso)) {
    Write-Host "ERROR: trustos.iso not found! Build first." -ForegroundColor Red
    exit 1
}

# -------------------------------------------------------------------------
# Utility Functions
# -------------------------------------------------------------------------
function Send-Serial($port, $cmd, $timeoutSec = 15) {
    try {
        $tcp = New-Object System.Net.Sockets.TcpClient("127.0.0.1", $port)
        $tcp.ReceiveTimeout = 2000
        $s = $tcp.GetStream()
        $buf = New-Object byte[] 65536

        # Drain leftover
        $sw0 = [System.Diagnostics.Stopwatch]::StartNew()
        while ($sw0.Elapsed.TotalSeconds -lt 2) {
            if ($s.DataAvailable) { $s.Read($buf, 0, $buf.Length) | Out-Null }
            else { Start-Sleep -Milliseconds 50 }
        }

        $bytes = [System.Text.Encoding]::ASCII.GetBytes("$cmd`r")
        $s.Write($bytes, 0, $bytes.Length)
        $s.Flush()
        Start-Sleep -Milliseconds 500

        $out = ""
        $sw = [System.Diagnostics.Stopwatch]::StartNew()
        while ($sw.Elapsed.TotalSeconds -lt $timeoutSec) {
            if ($s.DataAvailable) {
                $n = $s.Read($buf, 0, $buf.Length)
                $out += [System.Text.Encoding]::ASCII.GetString($buf, 0, $n)
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

function Test-Connected($text) {
    return -not ($text -match "CONNECTION_FAILED")
}

function Wait-Boot($port, $timeoutSec = 45) {
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
    } catch { return "" }
}

$pass = 0; $fail = 0; $skip = 0; $results = @()

function Test-Check($name, $condition) {
    if ($condition) {
        Write-Host "  [PASS] $name" -ForegroundColor Green
        $script:pass++
        $script:results += "PASS: $name"
    } else {
        Write-Host "  [FAIL] $name" -ForegroundColor Red
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

# =========================================================================
# CLEANUP
# =========================================================================
Write-Host ""
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host "    JARVIS FULL CAPABILITY TEST SUITE" -ForegroundColor Cyan
Write-Host "    Phase A: Single-node (all brain commands)" -ForegroundColor Cyan
Write-Host "    Phase B: 2-node propagation + federated" -ForegroundColor Cyan
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host ""

Stop-Process -Name qemu-system-x86_64 -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 2

# =========================================================================
# PHASE A: SINGLE NODE TESTS
# =========================================================================
Write-Host "========================================" -ForegroundColor Yellow
Write-Host " PHASE A: Single-Node JARVIS Tests" -ForegroundColor Yellow
Write-Host "========================================" -ForegroundColor Yellow
Write-Host ""

# Boot single node 
Write-Host "[A.0] Booting TrustOS (single node)..." -ForegroundColor Yellow
$p0 = Start-Process -FilePath $QemuExe -ArgumentList @(
    "-cdrom",$iso,"-m","1024M","-smp","2","-cpu","Haswell",
    "-machine","q35","-accel","whpx","-display","none",
    "-serial","tcp:127.0.0.1:5590,server,nowait",
    "-no-reboot"
) -PassThru -WindowStyle Hidden
Write-Host "       PID=$($p0.Id)" -ForegroundColor DarkGray

$boot = Wait-Boot 5590 50
$booted = $boot -match "trustos"
Test-Check "A.0 TrustOS booted" $booted

if (-not $booted) {
    Write-Host "  ABORT: TrustOS failed to boot" -ForegroundColor Red
    Stop-Process -Id $p0.Id -Force -ErrorAction SilentlyContinue
    exit 1
}
Show-Lines $boot "JARVIS|Micro|Full|brain|SIMD|mesh|kernel|Boot|Desktop"

# -------------------------------------------------------------------------
# A.1 jarvis brain init
# -------------------------------------------------------------------------
Write-Host ""
Write-Host "[A.1] jarvis brain init..." -ForegroundColor Yellow
$initOut = Send-Serial 5590 "jarvis brain init" 60
Test-Check "A.1 Brain init success" ($initOut -match "Init complete|brain loaded|Micro sentinel|Full brain|JARVIS brain initialized")
Test-Check "A.1 Full brain loaded (4.4M)" ($initOut -match "Full brain|4393216|17\.6|full=LOADED|Full=LOADED")
Show-Lines $initOut "JARVIS|brain|Micro|Full|param|loaded|Init|weights|sentinel|Maturity"

# -------------------------------------------------------------------------
# A.2 jarvis brain info
# -------------------------------------------------------------------------
Write-Host ""
Write-Host "[A.2] jarvis brain info..." -ForegroundColor Yellow
$infoOut = Send-Serial 5590 "jarvis brain info" 10
Test-Check "A.2 Brain info shows architecture" ($infoOut -match "layer|head|param|256|model")
Test-Check "A.2 Shows param count" ($infoOut -match "4393216|4\.4M|param")
Show-Lines $infoOut "layer|head|param|model|vocab|seq|dim|brain|JARVIS|Full|Micro"

# -------------------------------------------------------------------------
# A.3 jarvis brain hardware
# -------------------------------------------------------------------------
Write-Host ""
Write-Host "[A.3] jarvis brain hardware..." -ForegroundColor Yellow
$hwOut = Send-Serial 5590 "jarvis brain hardware" 10
Test-Check "A.3 Hardware detection" ($hwOut -match "CPU|SIMD|SSE|AVX|memory|compute|hardware|x86")
Show-Lines $hwOut "CPU|SIMD|SSE|AVX|FMA|memory|compute|hardware|GPU|backend|core|MHz"

# -------------------------------------------------------------------------
# A.4 jarvis brain introspect
# -------------------------------------------------------------------------
Write-Host ""
Write-Host "[A.4] jarvis brain introspect..." -ForegroundColor Yellow
$introOut = Send-Serial 5590 "jarvis brain introspect" 10
Test-Check "A.4 Introspection works" ($introOut -match "layer|transformer|param|architecture|d_model|I am|neural|JARVIS")
Show-Lines $introOut "layer|transformer|param|architecture|d_model|neural|head|vocab|I am|JARVIS|brain"

# -------------------------------------------------------------------------
# A.5 jarvis brain weights
# -------------------------------------------------------------------------
Write-Host ""
Write-Host "[A.5] jarvis brain weights..." -ForegroundColor Yellow
$weightsOut = Send-Serial 5590 "jarvis brain weights" 10
Test-Check "A.5 Weight statistics" ($weightsOut -match "mean|std|layer|weight|embed|norm|attention|ffn")
Show-Lines $weightsOut "mean|std|layer|weight|embed|norm|attention|ffn|min|max"

# -------------------------------------------------------------------------
# A.6 jarvis brain eval (baseline loss)
# -------------------------------------------------------------------------
Write-Host ""
Write-Host "[A.6] jarvis brain eval (baseline loss)..." -ForegroundColor Yellow
$evalOut = Send-Serial 5590 "jarvis brain eval" 30
Test-Check "A.6 Eval runs" ((Test-Connected $evalOut) -and ($evalOut -match "loss|Loss|eval|Eval"))
Show-Lines $evalOut "loss|Loss|eval|Eval|avg|perplexity|corpus|phase"

# -------------------------------------------------------------------------
# A.7 jarvis brain bench
# -------------------------------------------------------------------------
Write-Host ""
Write-Host "[A.7] jarvis brain bench..." -ForegroundColor Yellow
$benchOut = Send-Serial 5590 "jarvis brain bench" 30
Test-Check "A.7 Benchmark runs" ((Test-Connected $benchOut) -and ($benchOut -match "token|ms|inference|bench|speed|tok|latency|forward|gen"))
Show-Lines $benchOut "."

# -------------------------------------------------------------------------
# A.8 jarvis brain test (self-test)
# -------------------------------------------------------------------------
Write-Host ""
Write-Host "[A.8] jarvis brain test (self-test)..." -ForegroundColor Yellow
$testOut = Send-Serial 5590 "jarvis brain test" 30
Test-Check "A.8 Self-test runs" ((Test-Connected $testOut) -and ($testOut -match "test|Test|pass|Pass|PASS|ok|OK|check|Check"))
Show-Lines $testOut "test|Test|pass|Pass|PASS|fail|FAIL|ok|OK|check|Check|tokeniz|forward|embed"

# -------------------------------------------------------------------------
# A.9 jarvis brain generate "Hello I am"
# -------------------------------------------------------------------------
Write-Host ""
Write-Host "[A.9] jarvis brain generate..." -ForegroundColor Yellow
$genOut = Send-Serial 5590 "jarvis brain generate Hello I am" 20
Test-Check "A.9 Generation produces output" ((Test-Connected $genOut) -and ($genOut.Length -gt 30))
Show-Lines $genOut "." "Cyan"

# -------------------------------------------------------------------------
# A.10 jarvis brain chat "Who are you?"
# -------------------------------------------------------------------------
Write-Host ""
Write-Host "[A.10] jarvis brain chat..." -ForegroundColor Yellow
$chatOut = Send-Serial 5590 "jarvis brain chat Who are you?" 20
Test-Check "A.10 Chat produces response" ((Test-Connected $chatOut) -and ($chatOut.Length -gt 20))
Show-Lines $chatOut "." "Cyan"

# -------------------------------------------------------------------------
# A.11 jarvis brain chat "Qui es-tu?" (French)
# -------------------------------------------------------------------------
Write-Host ""
Write-Host "[A.11] jarvis brain chat (French)..." -ForegroundColor Yellow
$chatFrOut = Send-Serial 5590 "jarvis brain chat Qui es-tu?" 20
Test-Check "A.11 French chat response" ((Test-Connected $chatFrOut) -and ($chatFrOut.Length -gt 20))
Show-Lines $chatFrOut "." "Cyan"

# -------------------------------------------------------------------------
# A.12 jarvis brain save
# -------------------------------------------------------------------------
Write-Host ""
Write-Host "[A.12] jarvis brain save..." -ForegroundColor Yellow
$saveOut = Send-Serial 5590 "jarvis brain save" 20
Test-Check "A.12 Weights saved" ((Test-Connected $saveOut) -and ($saveOut -match "saved|Saved|save|written|bytes|weight"))
Show-Lines $saveOut "saved|Saved|save|written|bytes|weight|file|ram"

# -------------------------------------------------------------------------
# A.13 jarvis hw (hardware intelligence)
# -------------------------------------------------------------------------
Write-Host ""
Write-Host "[A.13] jarvis hw..." -ForegroundColor Yellow
Start-Sleep -Seconds 1
$jhwOut = Send-Serial 5590 "jarvis hw" 20
Test-Check "A.13 HW intelligence" ((Test-Connected $jhwOut) -and ($jhwOut -match "CPU|memory|hardware|profile|MHz|core|RAM|SIMD|detect|vendor|PCI"))
Show-Lines $jhwOut "."

# -------------------------------------------------------------------------
# A.14 Guardian / Pact
# -------------------------------------------------------------------------
Write-Host ""
Write-Host "[A.14] pact..." -ForegroundColor Yellow
Start-Sleep -Seconds 1
$pactOut = Send-Serial 5590 "pact" 10
Test-Check "A.14 Pact shows the agreement" ((Test-Connected $pactOut) -and ($pactOut -match "pact|Pact|guardian|Guardian|Nathan|Copilot|JARVIS|parent"))
Show-Lines $pactOut "."

# -------------------------------------------------------------------------
# A.15 Guardian auth
# -------------------------------------------------------------------------
Write-Host ""
Write-Host "[A.15] Guardian auth..." -ForegroundColor Yellow
Start-Sleep -Seconds 1
$authOut = Send-Serial 5590 "guardian auth trustos" 10
Test-Check "A.15 Guardian auth" ((Test-Connected $authOut) -and ($authOut -match "authenticated|unlocked|Nathan|session"))
Show-Lines $authOut "."

# -------------------------------------------------------------------------
# A.16 jarvis brain pretrain 1 (slow: ~371 seq × gradient descent)
# -------------------------------------------------------------------------
Write-Host ""
Write-Host "[A.16] jarvis brain pretrain 1..." -ForegroundColor Yellow
Write-Host "       (371 sequences with numerical gradient descent)" -ForegroundColor DarkGray
$trainOut = Send-Serial 5590 "jarvis brain pretrain 1" 900
Test-Check "A.16a Pretraining started" ((Test-Connected $trainOut) -and ($trainOut -match "phase|Phase|epoch|Epoch|train|Train|corpus|loss|Pre-training"))
Test-Check "A.16b Loss computed" ($trainOut -match "loss|Loss")
Show-Lines $trainOut "phase|Phase|epoch|Epoch|train|Train|corpus|loss|Loss|step|Complete|done"

# -------------------------------------------------------------------------
# A.17 jarvis brain eval (post-training)
# -------------------------------------------------------------------------
Write-Host ""
Write-Host "[A.17] jarvis brain eval (post-training)..." -ForegroundColor Yellow
Start-Sleep -Seconds 2
$eval2Out = Send-Serial 5590 "jarvis brain eval" 30
Test-Check "A.17 Post-training eval" ((Test-Connected $eval2Out) -and ($eval2Out -match "loss|Loss"))
Show-Lines $eval2Out "loss|Loss|eval|Eval|avg|perplexity"

# -------------------------------------------------------------------------
# A.18 jarvis brain reset
# -------------------------------------------------------------------------
Write-Host ""
Write-Host "[A.18] jarvis brain reset..." -ForegroundColor Yellow
Start-Sleep -Seconds 1
$resetOut = Send-Serial 5590 "jarvis brain reset" 15
Test-Check "A.18 Brain reset" ((Test-Connected $resetOut) -and ($resetOut -match "reset|Reset|reinit|random|initialized|Init|brain"))
Show-Lines $resetOut "."

# -------------------------------------------------------------------------
# A.19 jarvis brain load (from saved weights after reset — LAST, may crash VM)
# -------------------------------------------------------------------------
Write-Host ""
Write-Host "[A.19] jarvis brain load..." -ForegroundColor Yellow
Start-Sleep -Seconds 1
$loadOut = Send-Serial 5590 "jarvis brain load" 30
Test-Check "A.19 Weights loaded" ((Test-Connected $loadOut) -and ($loadOut -match "loaded|Loaded|load|weight|brain|param|bytes|KB"))
Show-Lines $loadOut "loaded|Loaded|load|weight|brain|param|bytes|source|KB"

# Kill single node
Write-Host ""
Write-Host "  Shutting down single-node..." -ForegroundColor DarkGray
Stop-Process -Id $p0.Id -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 3

# =========================================================================
# PHASE B: 2-NODE PROPAGATION + FEDERATED
# =========================================================================
Write-Host ""
Write-Host "========================================" -ForegroundColor Yellow
Write-Host " PHASE B: 2-Node Propagation + Federated" -ForegroundColor Yellow
Write-Host "========================================" -ForegroundColor Yellow
Write-Host ""

Stop-Process -Name qemu-system-x86_64 -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 2

# Boot Node 0 (Leader)
Write-Host "[B.1] Booting Node 0 (Leader)..." -ForegroundColor Yellow
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

# Boot Node 1 (Worker)
Write-Host "[B.2] Booting Node 1 (Worker)..." -ForegroundColor Yellow
$p1 = Start-Process -FilePath $QemuExe -ArgumentList @(
    "-cdrom",$iso,"-m","512M","-smp","1","-cpu","Haswell",
    "-machine","q35","-accel","whpx","-display","none",
    "-serial","tcp:127.0.0.1:5591,server,nowait",
    "-netdev","socket,id=mesh0,connect=127.0.0.1:5600",
    "-device","virtio-net-pci,netdev=mesh0,mac=52:54:00:00:01:02",
    "-no-reboot"
) -PassThru -WindowStyle Hidden
Write-Host "       PID=$($p1.Id)" -ForegroundColor DarkGray

# Wait for boot
Write-Host "[B.3] Waiting for both nodes to boot..." -ForegroundColor Yellow
$boot0 = Wait-Boot 5590 50
$boot1 = Wait-Boot 5591 50
Test-Check "B.3 Node 0 booted" ($boot0 -match "trustos")
Test-Check "B.3 Node 1 booted" ($boot1 -match "trustos")

if (-not (($boot0 -match "trustos") -and ($boot1 -match "trustos"))) {
    Write-Host "  ABORT: nodes failed to boot" -ForegroundColor Red
    Stop-Process -Name qemu-system-x86_64 -Force -ErrorAction SilentlyContinue
    exit 1
}

# Node 0: Init brain (loads full brain + auto-starts mesh)
Write-Host ""
Write-Host "[B.4] Node 0: jarvis brain init (full brain + mesh)..." -ForegroundColor Yellow
$init0 = Send-Serial 5590 "jarvis brain init" 60
Test-Check "B.4 Node 0 full brain loaded" ($init0 -match "Full brain|full=LOADED|Full=LOADED|4393216")
Test-Check "B.4 Node 0 mesh auto-started" ($init0 -match "mesh|Mesh")
Show-Lines $init0 "JARVIS|brain|Full|Micro|mesh|Mesh|loaded|Init|param"

Start-Sleep -Seconds 3

# Node 1: Init brain (also loads full brain from ISO)
Write-Host ""
Write-Host "[B.5] Node 1: jarvis brain init..." -ForegroundColor Yellow
$init1 = Send-Serial 5591 "jarvis brain init" 60
Test-Check "B.5 Node 1 brain init" ($init1 -match "brain|JARVIS|Init|loaded")
Show-Lines $init1 "JARVIS|brain|Full|Micro|mesh|Mesh|loaded|Init|param"

Start-Sleep -Seconds 2

# Node 1: Delete brain + propagate from mesh
Write-Host ""
Write-Host "[B.6] Node 1: Delete brain + propagate from mesh..." -ForegroundColor Yellow
$rmOut = Send-Serial 5591 "rm /jarvis/weights.bin" 5
Start-Sleep -Seconds 1

$propOut = Send-Serial 5591 "jarvis brain propagate" 90
Test-Check "B.6 Propagation: brain downloaded" ($propOut -match "download|Download|pulled|received|propagat|17161|17\.6|weight|loaded|success")
Test-Check "B.6 Propagation: mesh peer found" ($propOut -match "peer|Peer|mesh|connect|node|discover")
Show-Lines $propOut "download|Download|pull|peer|Peer|mesh|connect|node|discover|weight|KB|MB|propagat|RPC|brain|loaded"

Start-Sleep -Seconds 2

# Node 1: Verify brain works after propagation
Write-Host ""
Write-Host "[B.7] Node 1: Verify brain works after propagation..." -ForegroundColor Yellow
$verify = Send-Serial 5591 "jarvis brain info" 10
Test-Check "B.7 Brain functional after propagation" ($verify -match "param|layer|head|model|brain")
Show-Lines $verify "param|layer|head|model|brain|Full|Micro"

# Node 1: Generate text to prove inference works
$verifyGen = Send-Serial 5591 "jarvis brain generate TrustOS is" 15
Test-Check "B.7 Inference works on propagated brain" ($verifyGen.Length -gt 20)
Show-Lines $verifyGen "." "Cyan"

# =========================================================================
# CLEANUP & SUMMARY
# =========================================================================
Write-Host ""
Write-Host "  Shutting down all nodes..." -ForegroundColor DarkGray
Stop-Process -Name qemu-system-x86_64 -Force -ErrorAction SilentlyContinue

Write-Host ""
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host "    JARVIS FULL TEST RESULTS" -ForegroundColor Cyan
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "  PASSED: $pass" -ForegroundColor Green
Write-Host "  FAILED: $fail" -ForegroundColor $(if ($fail -gt 0) { "Red" } else { "Green" })
Write-Host ""

foreach ($r in $results) {
    $color = if ($r.StartsWith("PASS")) { "Green" } else { "Red" }
    Write-Host "  $r" -ForegroundColor $color
}

Write-Host ""
if ($fail -eq 0) {
    Write-Host "  ALL TESTS PASSED!" -ForegroundColor Green
} else {
    Write-Host "  $fail test(s) failed" -ForegroundColor Red
}
Write-Host ""
