<#
.SYNOPSIS
    Jarvis Multi-Config Auto-Training Pipeline
.DESCRIPTION
    Boots TrustOS in multiple QEMU hardware configurations in sequence.
    Each config triggers: hwprobe → brain init → pretrain → save.
    The hardware corpus generator creates unique training data per config,
    teaching Jarvis to understand diverse hardware environments.
.NOTES
    Requires: QEMU, trustos.iso (run build-limine.ps1 first)
    Output:   jarvis_training_results.txt (full log)
              Serial output per config in training_logs/
#>

param(
    [string]$IsoPath   = "$PSScriptRoot\trustos.iso",
    [int]$Port         = 5560,
    [int]$Epochs       = 2,
    [int]$BootTimeout  = 30,
    [switch]$NoBuild,
    [switch]$Headless
)

$ErrorActionPreference = "Continue"
$QemuExe = "C:\Program Files\qemu\qemu-system-x86_64.exe"
$LogDir  = "$PSScriptRoot\training_logs"
$Report  = "$PSScriptRoot\jarvis_training_results.txt"
$Display = if ($Headless) { "none" } else { "gtk" }

# ═══════════════════════════════════════════════════════════════════════════════
# HARDWARE CONFIGURATIONS (diverse profiles for broad training)
# ═══════════════════════════════════════════════════════════════════════════════

$Configs = @(
    @{
        Name    = "minimal"
        Desc    = "Low-end: 128MB RAM, 1 CPU, no extras"
        Args    = @(
            "-m", "128M", "-smp", "1", "-cpu", "qemu64",
            "-machine", "q35", "-vga", "none"
        )
    },
    @{
        Name    = "standard"
        Desc    = "Mid-range: 256MB RAM, 2 CPUs, net+audio"
        Args    = @(
            "-m", "256M", "-smp", "2", "-cpu", "max",
            "-machine", "q35", "-vga", "std",
            "-device", "virtio-net-pci,netdev=net0",
            "-netdev", "user,id=net0",
            "-device", "intel-hda", "-device", "hda-duplex"
        )
    },
    @{
        Name    = "server"
        Desc    = "Server: 1GB RAM, 4 CPUs, NVMe+net, no display"
        Args    = @(
            "-m", "1G", "-smp", "4", "-cpu", "max",
            "-machine", "q35", "-vga", "none",
            "-device", "virtio-net-pci,netdev=net0",
            "-netdev", "user,id=net0",
            "-drive", "file=$PSScriptRoot\trustos_nvme.img,format=raw,if=none,id=nvme0",
            "-device", "nvme,serial=TRUSTNVME001,drive=nvme0"
        )
    },
    @{
        Name    = "desktop"
        Desc    = "Desktop: 512MB RAM, 2 CPUs, USB+audio+display+NVMe"
        Args    = @(
            "-m", "512M", "-smp", "2", "-cpu", "max",
            "-machine", "q35", "-vga", "std",
            "-device", "intel-hda", "-device", "hda-duplex",
            "-device", "qemu-xhci,id=xhci",
            "-device", "usb-mouse,bus=xhci.0",
            "-device", "virtio-net-pci,netdev=net0",
            "-netdev", "user,id=net0",
            "-drive", "file=$PSScriptRoot\trustos_nvme.img,format=raw,if=none,id=nvme0",
            "-device", "nvme,serial=TRUSTNVME001,drive=nvme0"
        )
    },
    @{
        Name    = "embedded"
        Desc    = "Embedded: 128MB RAM, 1 CPU, minimal SSE, no peripherals"
        Args    = @(
            "-m", "128M", "-smp", "1", "-cpu", "qemu64,+sse,+sse2,-sse3,-ssse3,-sse4.1,-sse4.2,-avx,-avx2",
            "-machine", "q35", "-vga", "none"
        )
    },
    @{
        Name    = "workstation"
        Desc    = "Workstation: 2GB RAM, 4 CPUs, full AVX2, all devices"
        Args    = @(
            "-m", "2G", "-smp", "4", "-cpu", "max",
            "-machine", "q35", "-vga", "std",
            "-device", "virtio-net-pci,netdev=net0",
            "-netdev", "user,id=net0",
            "-device", "intel-hda", "-device", "hda-duplex",
            "-device", "qemu-xhci,id=xhci",
            "-device", "usb-mouse,bus=xhci.0",
            "-device", "usb-tablet,bus=xhci.0",
            "-drive", "file=$PSScriptRoot\trustos_nvme.img,format=raw,if=none,id=nvme0",
            "-device", "nvme,serial=TRUSTNVME001,drive=nvme0"
        )
    }
)

# ═══════════════════════════════════════════════════════════════════════════════
# SERIAL COMMUNICATION
# ═══════════════════════════════════════════════════════════════════════════════

function Send-Cmd {
    param(
        [System.Net.Sockets.NetworkStream]$Stream,
        [string]$Cmd,
        [int]$Timeout = 10,
        [string]$WaitFor = ""
    )
    $buffer = New-Object byte[] 65536

    # Check if stream is alive
    try { $null = $Stream.DataAvailable } catch { return "[CONNECTION LOST]" }

    # Drain
    $sw0 = [System.Diagnostics.Stopwatch]::StartNew()
    while ($sw0.ElapsedMilliseconds -lt 500) {
        try {
            if ($Stream.DataAvailable) { $Stream.Read($buffer, 0, $buffer.Length) | Out-Null }
            else { Start-Sleep -Milliseconds 50 }
        } catch { break }
    }

    # Send
    $bytes = [System.Text.Encoding]::ASCII.GetBytes("$Cmd`r")
    try {
        $Stream.Write($bytes, 0, $bytes.Length)
        $Stream.Flush()
    } catch {
        return "[CONNECTION LOST]"
    }

    # Receive
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
                # WaitFor pattern match
                if ($WaitFor -and ($out -match $WaitFor)) {
                    Start-Sleep -Milliseconds 500
                    while ($Stream.DataAvailable) {
                        $read = $Stream.Read($buffer, 0, $buffer.Length)
                        if ($read -gt 0) { $out += [System.Text.Encoding]::ASCII.GetString($buffer, 0, $read) }
                    }
                    break
                }
                # Shell prompt = done
                if (-not $WaitFor -and ($out -match "trustos:.*\$\s*$") -and (($sw.ElapsedMilliseconds - $lastData) -gt 1000)) {
                    break
                }
                # Silence timeout
                if (-not $WaitFor -and $lastData -gt 0 -and (($sw.ElapsedMilliseconds - $lastData) -gt 5000)) {
                    break
                }
            }
        }
    }
    return $out
}

function Wait-Boot {
    param([System.Net.Sockets.NetworkStream]$Stream, [int]$Timeout = 30)
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

# ═══════════════════════════════════════════════════════════════════════════════
# TRAINING PIPELINE FOR A SINGLE CONFIG
# ═══════════════════════════════════════════════════════════════════════════════

function Train-Config {
    param($Config, [int]$Index, [int]$Total)

    $name = $Config.Name
    $desc = $Config.Desc
    $logFile = "$LogDir\${name}.log"

    Write-Host "`n╔══════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
    Write-Host "║  CONFIG $($Index+1)/$Total : $name" -ForegroundColor Cyan
    Write-Host "║  $desc" -ForegroundColor DarkCyan
    Write-Host "╚══════════════════════════════════════════════════════════════╝" -ForegroundColor Cyan

    $configLog = "=== CONFIG: $name ===`nDesc: $desc`nDate: $(Get-Date)`n`n"
    $configResult = @{ Name = $name; Success = $false; Loss = "N/A"; HwSeqs = 0; Steps = 0; Duration = 0 }
    $configSw = [System.Diagnostics.Stopwatch]::StartNew()

    # Kill any existing QEMU
    Stop-Process -Name qemu-system-x86_64 -Force -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 2

    # Assemble QEMU args
    $serialArg = "tcp:127.0.0.1:${Port},server,nowait"
    $qemuArgs = @("-cdrom", $IsoPath) + $Config.Args + @(
        "-accel", "whpx",
        "-display", $Display,
        "-serial", $serialArg,
        "-no-reboot"
    )

    # Launch QEMU
    Write-Host "  [1/6] Starting QEMU..." -NoNewline
    $proc = Start-Process -FilePath $QemuExe -ArgumentList $qemuArgs -PassThru -WindowStyle Hidden
    if (-not $proc) {
        Write-Host " FAILED" -ForegroundColor Red
        $configLog += "ERROR: Failed to start QEMU`n"
        $configLog | Out-File -FilePath $logFile -Encoding UTF8
        return $configResult
    }
    Write-Host " PID=$($proc.Id)" -ForegroundColor DarkGray
    Start-Sleep -Seconds 2

    # Connect serial
    Write-Host "  [2/6] Connecting serial..." -NoNewline
    $tcp = $null
    for ($i = 0; $i -lt 20; $i++) {
        try { $tcp = New-Object System.Net.Sockets.TcpClient("127.0.0.1", $Port); break }
        catch { Start-Sleep -Milliseconds 500 }
    }
    if (-not $tcp) {
        Write-Host " FAILED" -ForegroundColor Red
        Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
        $configLog += "ERROR: TCP connect failed`n"
        $configLog | Out-File -FilePath $logFile -Encoding UTF8
        return $configResult
    }
    $stream = $tcp.GetStream()
    Write-Host " OK" -ForegroundColor Green

    # Wait for boot
    Write-Host "  [3/6] Waiting for boot..." -NoNewline
    $bootOut = Wait-Boot -Stream $stream -Timeout $BootTimeout
    if ($bootOut -notmatch "trustos") {
        Write-Host " TIMEOUT" -ForegroundColor Red
        $configLog += "ERROR: Boot timeout`n$bootOut`n"
        $stream.Close(); $tcp.Close()
        Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
        $configLog | Out-File -FilePath $logFile -Encoding UTF8
        return $configResult
    }
    Write-Host " OK" -ForegroundColor Green
    $configLog += "--- BOOT ---`n$bootOut`n`n"

    # Step 4: Hardware scan (triggers probe.rs → cached for hw_corpus)
    Write-Host "  [4/6] Hardware scan (jarvis boot)..." -NoNewline
    $hwOut = Send-Cmd -Stream $stream -Cmd "jarvis boot" -Timeout 120 -WaitFor "Boot Scan Complete"
    $configLog += "--- HW SCAN ---`n$hwOut`n`n"
    if ($hwOut -match "Boot Scan Complete") {
        Write-Host " OK" -ForegroundColor Green
    } else {
        Write-Host " partial" -ForegroundColor Yellow
    }

    # Step 5: Brain init + pretrain
    Write-Host "  [5/6] Brain init..." -NoNewline
    $initOut = Send-Cmd -Stream $stream -Cmd "jarvis brain init" -Timeout 120 -WaitFor "Neural brain ready"
    $configLog += "--- BRAIN INIT ---`n$initOut`n`n"

    # Extract HW corpus count
    if ($initOut -match "Hardware corpus: (\d+) sequences") {
        $configResult.HwSeqs = [int]$Matches[1]
    }

    if ($initOut -match "Neural brain ready") {
        Write-Host " OK (HW seqs: $($configResult.HwSeqs))" -ForegroundColor Green
    } else {
        Write-Host " partial" -ForegroundColor Yellow
    }

    # Pretrain with configured epochs
    # Estimate: ~6-8 min per epoch in TCG for ~400 sequences
    $pretrainTimeout = $Epochs * 900  # 15 min per epoch max
    Write-Host "  [6/6] Pretrain $Epochs epochs..." -NoNewline
    $ptOut = Send-Cmd -Stream $stream -Cmd "jarvis brain pretrain $Epochs" -Timeout $pretrainTimeout -WaitFor "Pre-training done"
    $configLog += "--- PRETRAIN ---`n$ptOut`n`n"

    # Parse results
    if ($ptOut -match 'avg loss=([0-9.]+)') {
        $configResult.Loss = $Matches[1]
    }
    if ($ptOut -match "(\d+) steps") {
        $configResult.Steps = [int]$Matches[1]
    }

    if ($ptOut -match "Pre-training done") {
        Write-Host " loss=$($configResult.Loss), steps=$($configResult.Steps)" -ForegroundColor Green
        $configResult.Success = $true
    } else {
        Write-Host " incomplete" -ForegroundColor Yellow
    }

    # Save weights (in RamFS — mainly for verification, not persistent across boots)
    $saveOut = Send-Cmd -Stream $stream -Cmd "jarvis brain save" -Timeout 15
    $configLog += "--- SAVE ---`n$saveOut`n`n"

    # Quick chat test to see if the model responds
    $chatOut = Send-Cmd -Stream $stream -Cmd "jarvis chat What hardware am I running on?" -Timeout 60 -WaitFor "trustos:"
    $configLog += "--- CHAT TEST ---`n$chatOut`n`n"

    # Cleanup
    $configSw.Stop()
    $configResult.Duration = [math]::Round($configSw.Elapsed.TotalSeconds)
    $stream.Close(); $tcp.Close()
    Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 2

    $configLog += "Duration: $($configResult.Duration)s`n"
    $configLog | Out-File -FilePath $logFile -Encoding UTF8

    Write-Host "  Done in $($configResult.Duration)s" -ForegroundColor DarkGray
    return $configResult
}

# ═══════════════════════════════════════════════════════════════════════════════
# MAIN
# ═══════════════════════════════════════════════════════════════════════════════

$startTime = Get-Date

Write-Host ""
Write-Host "╔══════════════════════════════════════════════════════════════╗" -ForegroundColor White
Write-Host "║      JARVIS MULTI-CONFIG AUTO-TRAINING PIPELINE            ║" -ForegroundColor White
Write-Host "║      $($Configs.Count) hardware configs x $Epochs epochs each                  ║" -ForegroundColor DarkGray
Write-Host "╚══════════════════════════════════════════════════════════════╝" -ForegroundColor White

# Verify prereqs
if (-not (Test-Path $IsoPath)) {
    Write-Host "ERROR: No ISO at $IsoPath — run build-limine.ps1 first" -ForegroundColor Red
    exit 1
}
if (-not (Test-Path $QemuExe)) {
    Write-Host "ERROR: QEMU not found at $QemuExe" -ForegroundColor Red
    exit 1
}

# Create NVMe disk image if needed (some configs reference it)
$nvmePath = "$PSScriptRoot\trustos_nvme.img"
if (-not (Test-Path $nvmePath)) {
    Write-Host "Creating NVMe disk image (64MB)..." -ForegroundColor DarkGray
    & "$($QemuExe -replace 'qemu-system-x86_64','qemu-img')" create -f raw $nvmePath 64M 2>$null
    if (-not (Test-Path $nvmePath)) {
        # Fallback: create a raw file
        $fs = [System.IO.File]::Create($nvmePath)
        $fs.SetLength(64MB)
        $fs.Close()
    }
}

# Create log directory
if (-not (Test-Path $LogDir)) { New-Item -ItemType Directory -Path $LogDir -Force | Out-Null }

# Optional rebuild
if (-not $NoBuild) {
    Write-Host "`n[BUILD] Compiling kernel + ISO..." -ForegroundColor Yellow
    Push-Location $PSScriptRoot
    & .\build-limine.ps1
    Pop-Location
    if (-not (Test-Path $IsoPath)) {
        Write-Host "ERROR: Build failed — no ISO produced" -ForegroundColor Red
        exit 1
    }
    Write-Host "[BUILD] Done" -ForegroundColor Green
}

# Run each config
$results = @()
for ($i = 0; $i -lt $Configs.Count; $i++) {
    $result = Train-Config -Config $Configs[$i] -Index $i -Total $Configs.Count
    $results += $result
}

# ═══════════════════════════════════════════════════════════════════════════════
# SUMMARY REPORT
# ═══════════════════════════════════════════════════════════════════════════════

$endTime = Get-Date
$totalDuration = [math]::Round(($endTime - $startTime).TotalMinutes, 1)

Write-Host "`n"
Write-Host "╔══════════════════════════════════════════════════════════════╗" -ForegroundColor White
Write-Host "║                    TRAINING SUMMARY                        ║" -ForegroundColor White
Write-Host "╠══════════════════════════════════════════════════════════════╣" -ForegroundColor White

$successCount = ($results | Where-Object { $_.Success }).Count
$totalSteps   = ($results | Measure-Object -Property Steps -Sum).Sum
$totalHwSeqs  = ($results | Measure-Object -Property HwSeqs -Sum).Sum

foreach ($r in $results) {
    $status = if ($r.Success) { "[OK]" } else { "[!!]" }
    $color  = if ($r.Success) { "Green" } else { "Red" }
    $line = "  {0} {1,-15} loss={2,-8} steps={3,-6} hw_seqs={4,-4} {5}s" -f $status, $r.Name, $r.Loss, $r.Steps, $r.HwSeqs, $r.Duration
    Write-Host $line -ForegroundColor $color
}

Write-Host "╠══════════════════════════════════════════════════════════════╣" -ForegroundColor White
Write-Host ("  Configs: {0}/{1} OK  |  Total steps: {2}  |  HW sequences: {3}" -f $successCount, $Configs.Count, $totalSteps, $totalHwSeqs) -ForegroundColor Cyan
Write-Host ("  Total duration: {0} min" -f $totalDuration) -ForegroundColor DarkGray
Write-Host "╚══════════════════════════════════════════════════════════════╝" -ForegroundColor White

# Write report file
$report = @"
═══════════════════════════════════════════════════════════════
  JARVIS MULTI-CONFIG TRAINING REPORT
  Date: $startTime
  Epochs per config: $Epochs
  Total duration: $totalDuration min
═══════════════════════════════════════════════════════════════

RESULTS:
"@

foreach ($r in $results) {
    $status = if ($r.Success) { "OK" } else { "FAIL" }
    $report += "`n  [$status] $($r.Name) — loss=$($r.Loss), steps=$($r.Steps), hw_seqs=$($r.HwSeqs), $($r.Duration)s"
}

$report += @"

TOTALS:
  Configs passed: $successCount / $($Configs.Count)
  Total training steps: $totalSteps
  Total HW corpus sequences: $totalHwSeqs
  Total duration: $totalDuration min

Per-config logs: $LogDir\
"@

$report | Out-File -FilePath $Report -Encoding UTF8
Write-Host "`nReport: $Report" -ForegroundColor DarkGray
Write-Host "Logs:   $LogDir\" -ForegroundColor DarkGray
