# Jarvis Training Demo - inline TCP serial
$ErrorActionPreference = "Continue"
$IsoPath = "$PSScriptRoot\trustos.iso"
$Port    = 5558
$QemuExe = "C:\Program Files\qemu\qemu-system-x86_64.exe"
$OutFile = "$PSScriptRoot\jarvis_demo_out.txt"

if (-not (Test-Path $IsoPath)) { Write-Host "No ISO"; exit 1 }

# Kill old qemu
Stop-Process -Name qemu-system-x86_64 -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 1

# Launch QEMU
$qemuArgs = @(
    "-cdrom", $IsoPath,
    "-m", "512",
    "-smp", "2",
    "-serial", "tcp:127.0.0.1:${Port},server,nowait",
    "-display", "none",
    "-no-reboot"
)
$proc = Start-Process -FilePath $QemuExe -ArgumentList $qemuArgs -PassThru -WindowStyle Hidden
Write-Host "QEMU PID=$($proc.Id) on port $Port"
Start-Sleep -Seconds 3

# Helper: send command, collect output
function Send-Cmd {
    param($stream, $cmd, [int]$timeout = 10)
    $buffer = New-Object byte[] 32768
    # drain
    $sw0 = [System.Diagnostics.Stopwatch]::StartNew()
    while ($sw0.ElapsedMilliseconds -lt 500) {
        if ($stream.DataAvailable) { $stream.Read($buffer,0,$buffer.Length) | Out-Null } else { Start-Sleep -Milliseconds 50 }
    }
    # send
    $bytes = [System.Text.Encoding]::ASCII.GetBytes("$cmd`r")
    $stream.Write($bytes, 0, $bytes.Length); $stream.Flush()
    # collect
    $out = ""
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $lastData = 0
    while ($sw.Elapsed.TotalSeconds -lt $timeout) {
        if ($stream.DataAvailable) {
            $read = $stream.Read($buffer, 0, $buffer.Length)
            if ($read -gt 0) { $out += [System.Text.Encoding]::ASCII.GetString($buffer, 0, $read); $lastData = $sw.ElapsedMilliseconds }
        } else {
            Start-Sleep -Milliseconds 20
            # if we have data and 2s of silence, break early
            if ($out.Length -gt 20 -and ($sw.ElapsedMilliseconds - $lastData) -gt 2000) { break }
        }
    }
    return $out
}

# Connect
$attempts = 0
$tcp = $null
while ($attempts -lt 10) {
    try {
        $tcp = New-Object System.Net.Sockets.TcpClient("127.0.0.1", $Port)
        Write-Host "Connected!"
        break
    } catch {
        $attempts++
        Write-Host "Retry $attempts..."
        Start-Sleep -Seconds 2
    }
}
if (-not $tcp) { Write-Host "FAIL: cannot connect"; exit 1 }
$stream = $tcp.GetStream()

# Wait for boot (collect boot messages for 15s)
Write-Host "Waiting for boot..."
$bootOut = ""
$buffer = New-Object byte[] 32768
$sw = [System.Diagnostics.Stopwatch]::StartNew()
while ($sw.Elapsed.TotalSeconds -lt 20) {
    if ($stream.DataAvailable) {
        $read = $stream.Read($buffer, 0, $buffer.Length)
        if ($read -gt 0) { $bootOut += [System.Text.Encoding]::ASCII.GetString($buffer, 0, $read) }
    } else {
        Start-Sleep -Milliseconds 100
    }
    if ($bootOut -match "trustos:.*\$") { Start-Sleep -Seconds 1; break }
}
Write-Host "Boot done ($($bootOut.Length) chars)"

$log = "=== JARVIS TRAINING DEMO ===`n"
$log += "Date: $(Get-Date)`n`n"
$log += "--- BOOT ---`n$bootOut`n`n"

# Step 1: Init brain
Write-Host ">> jarvis brain init"
$r = Send-Cmd $stream "jarvis brain init" 30
$log += "--- BRAIN INIT ---`n$r`n`n"
Write-Host ($r -split "`n" | Select-Object -Last 5 | Out-String)

# Step 2: Eval BEFORE training
Write-Host ">> jarvis brain eval"
$r = Send-Cmd $stream "jarvis brain eval" 60
$log += "--- EVAL BEFORE TRAINING ---`n$r`n`n"
Write-Host ($r -split "`n" | Select-Object -Last 5 | Out-String)

# Step 3: Pretrain 1 epoch
Write-Host ">> jarvis brain pretrain 1"
$r = Send-Cmd $stream "jarvis brain pretrain 1" 300
$log += "--- PRETRAIN 1 EPOCH ---`n$r`n`n"
Write-Host ($r -split "`n" | Select-Object -Last 5 | Out-String)

# Step 4: Eval AFTER training
Write-Host ">> jarvis brain eval (after)"
$r = Send-Cmd $stream "jarvis brain eval" 60
$log += "--- EVAL AFTER TRAINING ---`n$r`n`n"
Write-Host ($r -split "`n" | Select-Object -Last 5 | Out-String)

# Step 5: Chat test
Write-Host ">> jarvis chat Hello Jarvis"
$r = Send-Cmd $stream "jarvis chat Hello Jarvis" 30
$log += "--- CHAT ---`n$r`n`n"
Write-Host ($r -split "`n" | Select-Object -Last 5 | Out-String)

# Step 6: Save weights
Write-Host ">> jarvis brain save"
$r = Send-Cmd $stream "jarvis brain save" 10
$log += "--- SAVE ---`n$r`n`n"

$log += "=== DONE ===`n"
$log | Out-File -FilePath $OutFile -Encoding UTF8
Write-Host "`nResults written to $OutFile"

# Cleanup
$tcp.Close()
Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
