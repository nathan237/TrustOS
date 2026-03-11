<#
.SYNOPSIS
    JARVIS eval-only test with long timeout
#>

$QemuExe = "C:\Program Files\qemu\qemu-system-x86_64.exe"
$IsoPath = "$PSScriptRoot\trustos.iso"
$SerialPort = 5556

function Read-Serial($stream, [int]$timeoutMs = 60000) {
    $buffer = New-Object byte[] 16384
    $response = ""
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $lastData = 0
    while ($sw.ElapsedMilliseconds -lt $timeoutMs) {
        if ($stream.DataAvailable) {
            $n = $stream.Read($buffer, 0, $buffer.Length)
            $chunk = [System.Text.Encoding]::ASCII.GetString($buffer, 0, $n)
            $response += $chunk
            Write-Host -NoNewline $chunk
            $lastData = $sw.ElapsedMilliseconds
        } elseif ($response.Length -gt 50 -and ($sw.ElapsedMilliseconds - $lastData) -gt 5000) {
            break
        } else {
            Start-Sleep -Milliseconds 100
        }
    }
    return $response
}

function Send-And-Wait($writer, $stream, $cmd, [int]$timeoutMs = 120000) {
    # Drain
    $buffer = New-Object byte[] 16384
    while ($stream.DataAvailable) { $stream.Read($buffer, 0, $buffer.Length) | Out-Null }
    Start-Sleep -Milliseconds 500
    while ($stream.DataAvailable) { $stream.Read($buffer, 0, $buffer.Length) | Out-Null }
    
    # Send
    Write-Host "`n>>> SENDING: $cmd" -ForegroundColor Cyan
    $cmdBytes = [System.Text.Encoding]::ASCII.GetBytes("$cmd`r")
    $stream.Write($cmdBytes, 0, $cmdBytes.Length)
    $stream.Flush()
    
    # Read with long timeout
    return Read-Serial $stream $timeoutMs
}

Write-Host "=== JARVIS Eval Test (Long Timeout) ===" -ForegroundColor Green

# Kill existing
Get-Process -Name "qemu-system-x86_64" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep 1

# Launch QEMU with WHPX (falls back to TCG)
$serialArg = "tcp:127.0.0.1:${SerialPort},server,nowait"
$qemuArgs = @(
    "-cdrom", "`"$IsoPath`"",
    "-m", "512M",
    "-machine", "q35",
    "-cpu", "max",
    "-smp", "2",
    "-accel", "tcg,thread=multi",
    "-display", "gtk",
    "-vga", "std",
    "-serial", $serialArg,
    "-no-reboot"
)
Write-Host "Starting QEMU..." -ForegroundColor White
$proc = Start-Process -FilePath $QemuExe -ArgumentList $qemuArgs -PassThru
Write-Host "PID: $($proc.Id)"

# Connect
$client = New-Object System.Net.Sockets.TcpClient
for ($i = 0; $i -lt 80; $i++) {
    try { $client.Connect("127.0.0.1", $SerialPort); break } catch { Start-Sleep -Milliseconds 300 }
}
$stream = $client.GetStream()
$stream.ReadTimeout = 5000
$writer = New-Object System.IO.StreamWriter($stream)
$writer.AutoFlush = $true

# Wait for boot
Write-Host "Waiting 25s for boot..." -ForegroundColor White
Start-Sleep 25
$bootOut = Read-Serial $stream 5000
Write-Host "`n--- Boot done ---" -ForegroundColor Green

# Init brain (takes time to load 17MB)
$r = Send-And-Wait $writer $stream "jarvis brain init" 60000
Write-Host "`n--- Init done ---" -ForegroundColor Green

# Eval (should show loss per phase - fast since it's just forward passes)
$r = Send-And-Wait $writer $stream "jarvis brain eval" 120000
$r | Out-File "$PSScriptRoot\jarvis_eval_result.txt" -Encoding UTF8
Write-Host "`n--- Eval done ---" -ForegroundColor Green

# Try ONE short chat with very long timeout (5 min)
$r = Send-And-Wait $writer $stream "jarvis brain chat Hello" 300000
$r | Out-File "$PSScriptRoot\jarvis_chat_result.txt" -Encoding UTF8
Write-Host "`n--- Chat done ---" -ForegroundColor Green

# Cleanup
$client.Close()
Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
Write-Host "`nResults saved to jarvis_eval_result.txt and jarvis_chat_result.txt" -ForegroundColor Green
