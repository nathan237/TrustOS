# Jarvis single-train test with full serial capture
$ErrorActionPreference = "Continue"
$QemuExe = "C:\Program Files\qemu\qemu-system-x86_64.exe"
$iso = "$PSScriptRoot\trustos.iso"
$Port = 5560
$OutFile = "$PSScriptRoot\train_test_output.txt"

Stop-Process -Name qemu-system-x86_64 -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 2

$proc = Start-Process -FilePath $QemuExe -ArgumentList @(
    "-cdrom", $iso, "-m", "512", "-smp", "2",
    "-serial", "tcp:127.0.0.1:${Port},server,nowait",
    "-display", "none", "-no-reboot"
) -PassThru -WindowStyle Hidden
Write-Host "QEMU PID=$($proc.Id)"
Start-Sleep -Seconds 5

# Connect
$tcp = $null
for ($i = 0; $i -lt 10; $i++) {
    try { $tcp = New-Object System.Net.Sockets.TcpClient("127.0.0.1", $Port); break }
    catch { Start-Sleep -Seconds 2 }
}
if (-not $tcp) { Write-Host "FAIL connect"; exit 1 }
$stream = $tcp.GetStream()
Write-Host "Connected"

# Collect ALL serial output into a buffer
$allOutput = ""
$buffer = New-Object byte[] 65536

# Wait for boot (up to 30s)
$sw = [System.Diagnostics.Stopwatch]::StartNew()
while ($sw.Elapsed.TotalSeconds -lt 30) {
    if ($stream.DataAvailable) {
        $read = $stream.Read($buffer, 0, $buffer.Length)
        if ($read -gt 0) { $allOutput += [System.Text.Encoding]::ASCII.GetString($buffer, 0, $read) }
    } else { Start-Sleep -Milliseconds 200 }
    if ($allOutput -match "trustos:.*\$" -and $sw.ElapsedMilliseconds -gt 5000) { break }
}
Write-Host "Booted ($($allOutput.Length) chars)"

# Helper to send and wait  
function SendWait {
    param([string]$cmd, [int]$waitSec)
    # small drain
    Start-Sleep -Milliseconds 500
    while ($stream.DataAvailable) { $stream.Read($buffer,0,$buffer.Length) | Out-Null }
    
    $bytes = [System.Text.Encoding]::ASCII.GetBytes("$cmd`r")
    $stream.Write($bytes, 0, $bytes.Length); $stream.Flush()
    Write-Host ">> $cmd (waiting ${waitSec}s)"
    
    $result = ""
    $sw2 = [System.Diagnostics.Stopwatch]::StartNew()
    $lastData = 0
    while ($sw2.Elapsed.TotalSeconds -lt $waitSec) {
        if ($stream.DataAvailable) {
            $read = $stream.Read($buffer, 0, $buffer.Length)
            if ($read -gt 0) { 
                $chunk = [System.Text.Encoding]::ASCII.GetString($buffer, 0, $read)
                $result += $chunk
                $lastData = $sw2.ElapsedMilliseconds
            }
        } else {
            Start-Sleep -Milliseconds 200
            # After data and 3s silence, check for prompt
            if ($result.Length -gt 10 -and ($sw2.ElapsedMilliseconds - $lastData) -gt 3000) {
                if ($result -match "trustos:.*\$\s*$") { break }
            }
        }
    }
    return $result
}

# 1. Init
$r = SendWait "jarvis brain init" 60
$allOutput += "`n--- INIT ---`n$r`n"
Write-Host "Init done ($($r.Length) chars)"

# 2. Single train on small text
$r = SendWait "jarvis brain train Hello" 180
$allOutput += "`n--- TRAIN ---`n$r`n"
Write-Host "Train done ($($r.Length) chars)"

# Check if QEMU is alive
$alive = $false
try { Get-Process -Id $proc.Id -ErrorAction Stop | Out-Null; $alive = $true } catch {}
$allOutput += "`n--- QEMU ALIVE: $alive ---`n"
Write-Host "QEMU alive: $alive"

if ($alive) {
    # 3. Try eval
    $r = SendWait "jarvis brain eval" 600
    $allOutput += "`n--- EVAL ---`n$r`n"
    Write-Host "Eval done ($($r.Length) chars)"
}

$allOutput | Out-File -FilePath $OutFile -Encoding UTF8
Write-Host "`nFull output ($($allOutput.Length) chars) -> $OutFile"

$tcp.Close()
Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
