<#
.SYNOPSIS
    TrustOS CyberSec Reconnaissance Scenario Test
.DESCRIPTION
    Simulates a full penetration testing reconnaissance workflow using TrustScan tools.
    Chains: Network Recon → Host Discovery → Port Scan → Banner Grab → Vuln Assessment → Sniffer Analysis
    
    Target: QEMU gateway (10.0.2.2) — acts as a simulated target host.
    
    Phases:
      Phase 1: Network Enumeration (ifconfig, route, arp)
      Phase 2: Target Discovery (ping sweep, ARP)
      Phase 3: Port Scanning (SYN stealth scan)
      Phase 4: Service Enumeration (banner grab)
      Phase 5: Aggressive Scan (nmap -A)
      Phase 6: Vulnerability Assessment (vulnscan)
      Phase 7: Packet Capture & Analysis (sniff)
      Phase 8: Traceroute
.NOTES
    Requires: QEMU + trustos.iso (built with build-limine.ps1 -NoRun)
#>

param(
    [string]$IsoPath = "$PSScriptRoot\trustos.iso",
    [int]$SerialPort = 5557,
    [int]$BootTimeout = 25,
    [string]$Target = "10.0.2.2"
)

$QemuExe = "C:\Program Files\qemu\qemu-system-x86_64.exe"
$timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"

# ---------------------------------------------------------------
#  HELPERS
# ---------------------------------------------------------------

function Send-Command {
    param($stream, $cmd, [int]$timeout = 8)
    
    $buffer = New-Object byte[] 32768

    # Drain
    $drainSw = [System.Diagnostics.Stopwatch]::StartNew()
    $lastData = $drainSw.ElapsedMilliseconds
    while (($drainSw.ElapsedMilliseconds - $lastData) -lt 300 -and $drainSw.ElapsedMilliseconds -lt 2000) {
        if ($stream.DataAvailable) {
            $stream.Read($buffer, 0, $buffer.Length) | Out-Null
            $lastData = $drainSw.ElapsedMilliseconds
        } else {
            Start-Sleep -Milliseconds 50
        }
    }

    # Send
    $cmdBytes = [System.Text.Encoding]::ASCII.GetBytes("$cmd`r")
    $stream.Write($cmdBytes, 0, $cmdBytes.Length)
    $stream.Flush()

    # Collect
    $output = ""
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    while ($sw.Elapsed.TotalSeconds -lt $timeout) {
        if ($stream.DataAvailable) {
            $read = $stream.Read($buffer, 0, $buffer.Length)
            if ($read -gt 0) {
                $output += [System.Text.Encoding]::ASCII.GetString($buffer, 0, $read)
            }
        } else {
            Start-Sleep -Milliseconds 30
        }

        # Detect shell prompt → command done
        if ($sw.ElapsedMilliseconds -ge 500 -and $output.Length -gt 5) {
            if ($output -match "\d{2}:\d{2}:\d{2}\]\s*trustos:[^\r\n]*\$\s*$") {
                Start-Sleep -Milliseconds 100
                while ($stream.DataAvailable) {
                    $read = $stream.Read($buffer, 0, $buffer.Length)
                    if ($read -gt 0) { $output += [System.Text.Encoding]::ASCII.GetString($buffer, 0, $read) }
                }
                break
            }
        }
    }

    # Clean ANSI/control chars for display
    $clean = $output -replace '\x1B\[[0-9;]*[a-zA-Z]', '' -replace '[\x00-\x09\x0B\x0C\x0E-\x1F\x7F]', ''
    return $clean
}

function Print-Phase {
    param([int]$num, [string]$title, [string]$desc)
    Write-Host ""
    Write-Host ("  +============================================================+") -ForegroundColor Cyan
    Write-Host ("  |  PHASE {0}: {1,-48}|" -f $num, $title) -ForegroundColor Cyan
    Write-Host ("  +============================================================+") -ForegroundColor Cyan
    Write-Host "  $desc" -ForegroundColor DarkGray
    Write-Host ""
}

function Print-CmdOutput {
    param([string]$cmd, [string]$output)
    Write-Host "  trustos$ " -ForegroundColor Green -NoNewline
    Write-Host $cmd -ForegroundColor White
    $lines = $output -split "`n"
    foreach ($line in $lines) {
        $trimmed = $line.Trim()
        if ($trimmed.Length -eq 0) { continue }
        # Skip echoed command and prompt lines
        if ($trimmed -match "^\d{2}:\d{2}:\d{2}\]") { continue }
        if ($trimmed -eq $cmd) { continue }
        
        # Color code by content
        if ($trimmed -match "CRITICAL|Critical|HIGH|High") {
            Write-Host "    $trimmed" -ForegroundColor Red
        } elseif ($trimmed -match "MEDIUM|Medium|WARNING|filtered") {
            Write-Host "    $trimmed" -ForegroundColor Yellow
        } elseif ($trimmed -match "open|Open|OPEN|reply|Reply") {
            Write-Host "    $trimmed" -ForegroundColor Green
        } elseif ($trimmed -match "closed|Closed|CLOSED") {
            Write-Host "    $trimmed" -ForegroundColor DarkGray
        } elseif ($trimmed -match "INFO|Info|LOW|Low") {
            Write-Host "    $trimmed" -ForegroundColor Cyan
        } else {
            Write-Host "    $trimmed" -ForegroundColor Gray
        }
    }
    Write-Host ""
}

# ---------------------------------------------------------------
#  MAIN
# ---------------------------------------------------------------

Write-Host ""
Write-Host "  ===================================================================" -ForegroundColor Red
Write-Host "   _____               _   ____                                   " -ForegroundColor Red
Write-Host "  |_   _| __ _   _ ___| |_/ ___|  ___ __ _ _ __                   " -ForegroundColor Yellow
Write-Host "    | || '__| | | / __| __\___ \ / __/ _' | '_ \                  " -ForegroundColor Yellow
Write-Host "    | || |  | |_| \__ \ |_ ___) | (_| (_| | | | |                 " -ForegroundColor Red
Write-Host "    |_||_|   \__,_|___/\__|____/ \___\__,_|_| |_|                 " -ForegroundColor Red
Write-Host "  ===================================================================" -ForegroundColor Red
Write-Host ""
Write-Host "  TrustScan -- Cybersecurity Reconnaissance Scenario" -ForegroundColor White
Write-Host "  ---------------------------------------------------" -ForegroundColor DarkGray
Write-Host ("  Target:    {0}" -f $Target) -ForegroundColor Yellow
Write-Host ("  Date:      {0}" -f $timestamp) -ForegroundColor DarkGray
Write-Host ("  Scenario:  Full Recon -> Scan -> Enumerate -> Vuln Assess") -ForegroundColor DarkGray
Write-Host ""

# Pre-flight
if (-not (Test-Path $QemuExe)) { Write-Host "FATAL: QEMU not found" -ForegroundColor Red; exit 1 }
if (-not (Test-Path $IsoPath)) { Write-Host "FATAL: ISO not found - run build-limine.ps1 -NoRun first" -ForegroundColor Red; exit 1 }

# Kill existing QEMU
Get-Process -Name "qemu-system-x86_64" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 1

# Launch QEMU
Write-Host "[*] Booting TrustOS in QEMU..." -ForegroundColor White
$serialArg = "tcp:127.0.0.1:${SerialPort},server,nowait"
$qemuArgs = @(
    "-cdrom", "`"$IsoPath`"",
    "-m", "512M",
    "-machine", "q35",
    "-cpu", "max",
    "-smp", "2",
    "-accel", "tcg,thread=multi",
    "-display", "none",
    "-vga", "std",
    "-device", "virtio-net-pci,netdev=net0",
    "-netdev", "user,id=net0",
    "-device", "intel-hda",
    "-device", "hda-duplex",
    "-serial", $serialArg,
    "-no-reboot"
)
$qemuProcess = Start-Process -FilePath $QemuExe -ArgumentList $qemuArgs -PassThru -WindowStyle Hidden
Write-Host ("  QEMU PID: {0}" -f $qemuProcess.Id) -ForegroundColor DarkGray

# Connect serial
Write-Host "[*] Connecting to serial console..." -ForegroundColor White
$client = New-Object System.Net.Sockets.TcpClient
$connected = $false
for ($i = 0; $i -lt 60; $i++) {
    try {
        $client.Connect("127.0.0.1", $SerialPort)
        $connected = $true
        break
    } catch { Start-Sleep -Milliseconds 300 }
}
if (-not $connected) {
    Write-Host "FATAL: Serial connect failed" -ForegroundColor Red
    Stop-Process -Id $qemuProcess.Id -Force -ErrorAction SilentlyContinue
    exit 1
}
$stream = $client.GetStream()
$stream.ReadTimeout = 3000
Write-Host "  Connected!" -ForegroundColor Green

# Wait for boot
Write-Host "[*] Waiting for TrustOS shell..." -ForegroundColor White
$buffer = New-Object byte[] 16384
$sw = [System.Diagnostics.Stopwatch]::StartNew()
$bootText = ""
while ($sw.Elapsed.TotalSeconds -lt $BootTimeout) {
    if ($stream.DataAvailable) {
        $read = $stream.Read($buffer, 0, $buffer.Length)
        if ($read -gt 0) {
            $bootText += [System.Text.Encoding]::ASCII.GetString($buffer, 0, $read)
            if ($bootText -match "trustos.*[\$#]") { break }
        }
    } else { Start-Sleep -Milliseconds 150 }
}

if (-not ($bootText -match "trustos.*[\$#]")) {
    Write-Host "FATAL: TrustOS did not boot" -ForegroundColor Red
    Stop-Process -Id $qemuProcess.Id -Force -ErrorAction SilentlyContinue
    exit 1
}

$bootTime = [math]::Round($sw.Elapsed.TotalSeconds, 1)
Write-Host "  TrustOS booted in ${bootTime}s" -ForegroundColor Green
Write-Host ""
Write-Host "  =============================================================" -ForegroundColor Yellow
    Write-Host "  |   RECONNAISSANCE SCENARIO START                        |" -ForegroundColor Yellow
    Write-Host "  =============================================================" -ForegroundColor Yellow

$scenarioSw = [System.Diagnostics.Stopwatch]::StartNew()

# ═══════════════════════════════════════════════════════════
# PHASE 1: Network Enumeration
# ═══════════════════════════════════════════════════════════
Print-Phase 1 "NETWORK ENUMERATION" "Gathering local network configuration — IP, routes, ARP cache"

$out = Send-Command -stream $stream -cmd "ifconfig" -timeout 5
Print-CmdOutput "ifconfig" $out

$out = Send-Command -stream $stream -cmd "route" -timeout 5
Print-CmdOutput "route" $out

$out = Send-Command -stream $stream -cmd "arp" -timeout 5
Print-CmdOutput "arp" $out

# ═══════════════════════════════════════════════════════════
# PHASE 2: Target Discovery
# ═══════════════════════════════════════════════════════════
Print-Phase 2 "TARGET DISCOVERY" "ICMP ping to verify target is alive, measure RTT"

$out = Send-Command -stream $stream -cmd "ping $Target" -timeout 10
Print-CmdOutput "ping $Target" $out

$out = Send-Command -stream $stream -cmd "discover arp" -timeout 10
Print-CmdOutput "discover arp" $out

# ═══════════════════════════════════════════════════════════
# PHASE 3: Port Scanning (Stealth SYN)
# ═══════════════════════════════════════════════════════════
Print-Phase 3 "STEALTH PORT SCAN" "SYN scan (half-open) — stealthy, does not complete TCP handshake"

$out = Send-Command -stream $stream -cmd "nmap $Target -sS" -timeout 20
Print-CmdOutput "nmap $Target -sS" $out

# ═══════════════════════════════════════════════════════════
# PHASE 4: Service Enumeration (Banner Grab)
# ═══════════════════════════════════════════════════════════
Print-Phase 4 "SERVICE ENUMERATION" "Connect to open ports, grab service banners and version info"

$out = Send-Command -stream $stream -cmd "banner $Target" -timeout 15
Print-CmdOutput "banner $Target" $out

# ═══════════════════════════════════════════════════════════
# PHASE 5: Aggressive Scan (nmap -A)
# ═══════════════════════════════════════════════════════════
Print-Phase 5 "AGGRESSIVE SCAN" "Full nmap -A: port scan + banner grab + vulnerability assessment"

$out = Send-Command -stream $stream -cmd "nmap $Target -A" -timeout 30
Print-CmdOutput "nmap $Target -A" $out

# ═══════════════════════════════════════════════════════════
# PHASE 6: Vulnerability Assessment
# ═══════════════════════════════════════════════════════════
Print-Phase 6 "VULNERABILITY ASSESSMENT" "Checking open services against known CVEs and misconfigurations"

$out = Send-Command -stream $stream -cmd "vulnscan $Target" -timeout 15
Print-CmdOutput "vulnscan $Target" $out

# ═══════════════════════════════════════════════════════════
# PHASE 7: Packet Capture
# ═══════════════════════════════════════════════════════════
Print-Phase 7 "PACKET CAPTURE & ANALYSIS" "Start sniffer, generate traffic, analyze captured packets"

$out = Send-Command -stream $stream -cmd "sniff start" -timeout 5
Print-CmdOutput "sniff start" $out

# Generate some traffic to capture
$out = Send-Command -stream $stream -cmd "ping $Target" -timeout 8
Print-CmdOutput "ping $Target (generating traffic)" $out

$out = Send-Command -stream $stream -cmd "sniff stop" -timeout 5
Print-CmdOutput "sniff stop" $out

$out = Send-Command -stream $stream -cmd "sniff show 10" -timeout 5
Print-CmdOutput "sniff show 10" $out

$out = Send-Command -stream $stream -cmd "sniff stats" -timeout 5
Print-CmdOutput "sniff stats" $out

# ═══════════════════════════════════════════════════════════
# PHASE 8: Traceroute
# ═══════════════════════════════════════════════════════════
Print-Phase 8 "TRACEROUTE" "Map network path to target using TTL-based ICMP probes"

$out = Send-Command -stream $stream -cmd "traceroute $Target" -timeout 15
Print-CmdOutput "traceroute $Target" $out

# ═══════════════════════════════════════════════════════════
# SUMMARY
# ═══════════════════════════════════════════════════════════
$elapsed = [math]::Round($scenarioSw.Elapsed.TotalSeconds, 1)

Write-Host ""
Write-Host "  =============================================================" -ForegroundColor Yellow
Write-Host "  |   RECONNAISSANCE COMPLETE                              |" -ForegroundColor Yellow
Write-Host "  =============================================================" -ForegroundColor Yellow
Write-Host ""
Write-Host "  Scenario Duration: ${elapsed}s" -ForegroundColor White
Write-Host "  Target:            $Target" -ForegroundColor White
Write-Host "  Tools Used:        ifconfig, route, arp, ping, discover," -ForegroundColor DarkGray
Write-Host "                     nmap (SYN + aggressive), banner, vulnscan," -ForegroundColor DarkGray
Write-Host "                     sniff (capture + analysis), traceroute" -ForegroundColor DarkGray
Write-Host ""
Write-Host "  All phases executed from bare-metal TrustOS kernel." -ForegroundColor Green
Write-Host "  No userspace, no libc, no Linux - direct hardware access." -ForegroundColor Green
Write-Host ""

# Cleanup
Write-Host "[*] Shutting down QEMU..." -ForegroundColor White
$client.Close()
Stop-Process -Id $qemuProcess.Id -Force -ErrorAction SilentlyContinue
Write-Host "  Done." -ForegroundColor Green
