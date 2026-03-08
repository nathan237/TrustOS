<#
.SYNOPSIS
    Test JARVIS distributed math task on 2-node cluster
#>

$ErrorActionPreference = "Continue"
$QemuExe = "C:\Program Files\qemu\qemu-system-x86_64.exe"
$iso = "$PSScriptRoot\trustos.iso"

function Send-Serial {
    param([System.Net.Sockets.NetworkStream]$Stream, [string]$Cmd, [int]$Timeout = 20, [string]$WaitFor = "")
    $buffer = New-Object byte[] 65536
    # Drain
    $sw0 = [System.Diagnostics.Stopwatch]::StartNew()
    while ($sw0.ElapsedMilliseconds -lt 500) {
        try { if ($Stream.DataAvailable) { $Stream.Read($buffer, 0, $buffer.Length) | Out-Null } else { Start-Sleep -Milliseconds 50 } } catch { break }
    }
    # Send
    $bytes = [System.Text.Encoding]::ASCII.GetBytes("$Cmd`r")
    $Stream.Write($bytes, 0, $bytes.Length); $Stream.Flush()
    # Read
    $out = ""
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    while ($sw.Elapsed.TotalSeconds -lt $Timeout) {
        if ($Stream.DataAvailable) {
            $read = $Stream.Read($buffer, 0, $buffer.Length)
            if ($read -gt 0) { $out += [System.Text.Encoding]::ASCII.GetString($buffer, 0, $read) }
        } else { Start-Sleep -Milliseconds 100 }
        if ($WaitFor -and $out -match $WaitFor) { Start-Sleep -Milliseconds 500; break }
        if (-not $WaitFor -and $out -match "trustos:.*\$\s*$" -and $out.Length -gt 20) {
            $sw2 = [System.Diagnostics.Stopwatch]::StartNew()
            while ($sw2.ElapsedMilliseconds -lt 1500) {
                if ($Stream.DataAvailable) { $read = $Stream.Read($buffer, 0, $buffer.Length); if ($read -gt 0) { $out += [System.Text.Encoding]::ASCII.GetString($buffer, 0, $read) } }
                else { Start-Sleep -Milliseconds 100 }
            }
            break
        }
    }
    return $out
}

Stop-Process -Name qemu-system-x86_64 -Force -ErrorAction SilentlyContinue
Start-Sleep 2

Write-Host "=== JARVIS TASK TEST ===" -ForegroundColor Cyan

# Boot node 0 (listen)
Write-Host "Booting Node 0..." -ForegroundColor Yellow
$p0 = Start-Process -FilePath $QemuExe -ArgumentList @(
    "-cdrom",$iso,"-m","256M","-smp","1","-cpu","Haswell","-machine","q35","-accel","whpx",
    "-display","none","-serial","tcp:127.0.0.1:5570,server,nowait",
    "-netdev","socket,id=mesh0,listen=:5580","-device","virtio-net-pci,netdev=mesh0,mac=52:54:00:00:01:01",
    "-no-reboot"
) -PassThru -WindowStyle Hidden
Write-Host "  PID: $($p0.Id)"
Start-Sleep 3

# Boot node 1 (connect)
Write-Host "Booting Node 1..." -ForegroundColor Yellow
$p1 = Start-Process -FilePath $QemuExe -ArgumentList @(
    "-cdrom",$iso,"-m","256M","-smp","1","-cpu","Haswell","-machine","q35","-accel","whpx",
    "-display","none","-serial","tcp:127.0.0.1:5571,server,nowait",
    "-netdev","socket,id=mesh0,connect=127.0.0.1:5580","-device","virtio-net-pci,netdev=mesh0,mac=52:54:00:00:01:02",
    "-no-reboot"
) -PassThru -WindowStyle Hidden
Write-Host "  PID: $($p1.Id)"
Start-Sleep 3

# Connect serial ports
Write-Host "`nConnecting serials..." -ForegroundColor Yellow
$streams = @()
foreach ($port in @(5570, 5571)) {
    $tcp = $null
    for ($r = 0; $r -lt 20; $r++) {
        try { $tcp = New-Object System.Net.Sockets.TcpClient("127.0.0.1", $port); break }
        catch { Start-Sleep -Milliseconds 500 }
    }
    if ($tcp) {
        $streams += $tcp.GetStream()
        Write-Host "  Port $port : OK" -ForegroundColor Green
    } else {
        $streams += $null
        Write-Host "  Port $port : FAIL" -ForegroundColor Red
    }
}

# Wait for boot
Write-Host "`nWaiting for boot (up to 40s)..." -ForegroundColor Yellow
foreach ($i in 0..1) {
    if (-not $streams[$i]) { continue }
    $buffer = New-Object byte[] 65536
    $out = ""
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    while ($sw.Elapsed.TotalSeconds -lt 40) {
        if ($streams[$i].DataAvailable) {
            $read = $streams[$i].Read($buffer, 0, $buffer.Length)
            if ($read -gt 0) { $out += [System.Text.Encoding]::ASCII.GetString($buffer, 0, $read) }
        } else { Start-Sleep -Milliseconds 100 }
        if ($out -match "trustos:.*\$") { break }
    }
    if ($out -match "trustos") {
        Write-Host "  Node $i : BOOTED" -ForegroundColor Green
    } else {
        Write-Host "  Node $i : TIMEOUT (got $($out.Length) bytes)" -ForegroundColor Red
        if ($out.Length -gt 0) { Write-Host "  Last 500: $($out.Substring([Math]::Max(0,$out.Length-500)))" }
    }
}

# Init brain on both nodes
Write-Host "`n=== Init brain on Node 0 ===" -ForegroundColor Cyan
Send-Serial -Stream $streams[0] -Cmd "jarvis brain init" -Timeout 15 | Write-Host

Write-Host "`n=== Init brain on Node 1 ===" -ForegroundColor Cyan
Send-Serial -Stream $streams[1] -Cmd "jarvis brain init" -Timeout 15 | Write-Host

# Start mesh on both nodes
Write-Host "`n=== Start mesh on Node 0 ===" -ForegroundColor Cyan
Send-Serial -Stream $streams[0] -Cmd "mesh start" -Timeout 10 | Write-Host

Write-Host "`n=== Start mesh on Node 1 ===" -ForegroundColor Cyan
Send-Serial -Stream $streams[1] -Cmd "mesh start" -Timeout 10 | Write-Host

# Give mesh time for mutual discovery via UDP broadcast
Write-Host "`nWaiting 20s for mesh discovery..." -ForegroundColor Yellow
Start-Sleep 20

# Check mesh status
Write-Host "`n=== Mesh status on Node 0 ===" -ForegroundColor Cyan
Send-Serial -Stream $streams[0] -Cmd "mesh status" -Timeout 10 | Write-Host

# Run distributed task on node 0
Write-Host "`n=== Running distributed task ===" -ForegroundColor Cyan
$taskOut = Send-Serial -Stream $streams[0] -Cmd "jarvis brain task" -Timeout 45 -WaitFor "PASS|FAIL|verified"
Write-Host $taskOut

# Save results
$taskOut | Out-File "$PSScriptRoot\task_test_results.txt" -Encoding UTF8
Write-Host "`nResults saved to task_test_results.txt" -ForegroundColor Green

# Cleanup
Write-Host "`nCleaning up..." -ForegroundColor Yellow
Stop-Process -Name qemu-system-x86_64 -Force -ErrorAction SilentlyContinue
Write-Host "Done!" -ForegroundColor Green
