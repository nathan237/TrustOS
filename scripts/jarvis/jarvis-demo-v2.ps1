# Jarvis Training Demo v2 - longer timeouts for TCG emulation
$ErrorActionPreference = "Continue"
$IsoPath = "$PSScriptRoot\trustos.iso"
$Port    = 5558
$QemuExe = "C:\Program Files\qemu\qemu-system-x86_64.exe"
$OutFile = "$PSScriptRoot\jarvis_demo_v2.txt"

if (-not (Test-Path $IsoPath)) { Write-Host "No ISO"; exit 1 }

Stop-Process -Name qemu-system-x86_64 -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 1

$qemuArgs = @(
    "-cdrom", $IsoPath, "-m", "512", "-smp", "2",
    "-serial", "tcp:127.0.0.1:${Port},server,nowait",
    "-display", "none", "-no-reboot"
)
$proc = Start-Process -FilePath $QemuExe -ArgumentList $qemuArgs -PassThru -WindowStyle Hidden
Write-Host "QEMU PID=$($proc.Id)"
Start-Sleep -Seconds 3

function Send-Cmd {
    param($stream, $cmd, [int]$timeout = 10, [string]$waitFor = "", [int]$silenceTimeout = 5)
    $buffer = New-Object byte[] 32768
    # drain
    $sw0 = [System.Diagnostics.Stopwatch]::StartNew()
    while ($sw0.ElapsedMilliseconds -lt 500) {
        if ($stream.DataAvailable) { $stream.Read($buffer,0,$buffer.Length) | Out-Null } else { Start-Sleep -Milliseconds 50 }
    }
    $bytes = [System.Text.Encoding]::ASCII.GetBytes("$cmd`r")
    $stream.Write($bytes, 0, $bytes.Length); $stream.Flush()
    $out = ""
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $lastData = 0
    while ($sw.Elapsed.TotalSeconds -lt $timeout) {
        if ($stream.DataAvailable) {
            $read = $stream.Read($buffer, 0, $buffer.Length)
            if ($read -gt 0) { $out += [System.Text.Encoding]::ASCII.GetString($buffer, 0, $read); $lastData = $sw.ElapsedMilliseconds }
        } else {
            Start-Sleep -Milliseconds 50
            if ($out.Length -gt 20) {
                # Check for waitFor pattern
                if ($waitFor -and $out -match $waitFor) {
                    Start-Sleep -Milliseconds 1000
                    while ($stream.DataAvailable) {
                        $read = $stream.Read($buffer, 0, $buffer.Length)
                        if ($read -gt 0) { $out += [System.Text.Encoding]::ASCII.GetString($buffer, 0, $read) }
                    }
                    break
                }
                # Only use silence timeout if no waitFor pattern
                if (-not $waitFor -and $out -match "trustos:.*\$\s*$" -and ($sw.ElapsedMilliseconds - $lastData) -gt 1000) { break }
                if (-not $waitFor -and ($sw.ElapsedMilliseconds - $lastData) -gt ($silenceTimeout * 1000)) { break }
            }
        }
    }
    return $out
}

# Connect
$tcp = $null
for ($i = 0; $i -lt 10; $i++) {
    try { $tcp = New-Object System.Net.Sockets.TcpClient("127.0.0.1", $Port); break } catch { Start-Sleep -Seconds 2 }
}
if (-not $tcp) { Write-Host "FAIL"; exit 1 }
$stream = $tcp.GetStream()
Write-Host "Connected"

# Boot wait
$bootOut = ""
$buffer = New-Object byte[] 32768
$sw = [System.Diagnostics.Stopwatch]::StartNew()
while ($sw.Elapsed.TotalSeconds -lt 25) {
    if ($stream.DataAvailable) {
        $read = $stream.Read($buffer, 0, $buffer.Length)
        if ($read -gt 0) { $bootOut += [System.Text.Encoding]::ASCII.GetString($buffer, 0, $read) }
    } else { Start-Sleep -Milliseconds 100 }
    if ($bootOut -match "trustos:.*\$") { Start-Sleep -Seconds 2; break }
}
Write-Host "Booted ($($bootOut.Length) chars)"

$log = "=== JARVIS TRAINING DEMO v2 ===`nDate: $(Get-Date)`n`n"

# 1) Init brain
Write-Host "[1/3] jarvis brain init..."
$r = Send-Cmd $stream "jarvis brain init" 60 "Neural brain ready"
$log += "--- BRAIN INIT ---`n$r`n`n"
Write-Host "  -> Done"

# 2) Pretrain 1 epoch (includes before/after eval internally)
#    Estimated ~12 min in TCG: eval(3m) + train(6m) + eval(3m)
Write-Host "[2/3] jarvis brain pretrain 1 (includes before/after eval)..."
Write-Host "       Estimated ~12 min in TCG emulation..."
$r = Send-Cmd $stream "jarvis brain pretrain 1" 1200 "Training steps global"
$log += "--- PRETRAIN 1 EPOCH (with before/after eval) ---`n$r`n`n"
Write-Host "  -> Done"

# 3) Chat test
Write-Host "[3/3] jarvis chat Hello Jarvis..."
$r = Send-Cmd $stream "jarvis chat Hello Jarvis" 120
$log += "--- CHAT ---`n$r`n`n"

$log += "=== DONE ===`n"
$log | Out-File -FilePath $OutFile -Encoding UTF8
Write-Host "`nResults -> $OutFile"

$tcp.Close()
Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
