<#
.SYNOPSIS
    Quick JARVIS generation quality test
.DESCRIPTION
    Boots TrustOS in QEMU, inits JARVIS brain, runs chat tests, captures output.
#>

$QemuExe = "C:\Program Files\qemu\qemu-system-x86_64.exe"
$IsoPath = "$PSScriptRoot\trustos.iso"
$SerialPort = 5555

function Send-Command {
    param($writer, $stream, $cmd, [int]$timeout = 8)
    $buffer = New-Object byte[] 16384
    # Drain
    $drainSw = [System.Diagnostics.Stopwatch]::StartNew()
    $lastData = $drainSw.ElapsedMilliseconds
    while (($drainSw.ElapsedMilliseconds - $lastData) -lt 300 -and $drainSw.ElapsedMilliseconds -lt 2000) {
        if ($stream.DataAvailable) {
            $stream.Read($buffer, 0, $buffer.Length) | Out-Null
            $lastData = $drainSw.ElapsedMilliseconds
        } else { Start-Sleep -Milliseconds 50 }
    }
    # Send
    $cmdBytes = [System.Text.Encoding]::ASCII.GetBytes("$cmd`r")
    $stream.Write($cmdBytes, 0, $cmdBytes.Length)
    $stream.Flush()
    # Receive
    $response = ""
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $lastData2 = $sw.ElapsedMilliseconds
    while ($sw.ElapsedMilliseconds -lt ($timeout * 1000)) {
        if ($stream.DataAvailable) {
            $n = $stream.Read($buffer, 0, $buffer.Length)
            $response += [System.Text.Encoding]::ASCII.GetString($buffer, 0, $n)
            $lastData2 = $sw.ElapsedMilliseconds
        } elseif (($sw.ElapsedMilliseconds - $lastData2) -gt 2000 -and $response.Length -gt 0) {
            break
        } else { Start-Sleep -Milliseconds 100 }
    }
    return $response
}

Write-Host "`n=== JARVIS Generation Test ===" -ForegroundColor Cyan

# Kill existing QEMU
Get-Process -Name "qemu-system-x86_64" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 1

# Launch QEMU
Write-Host "[1] Starting QEMU..." -ForegroundColor White
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
$proc = Start-Process -FilePath $QemuExe -ArgumentList $qemuArgs -PassThru
Write-Host "  PID: $($proc.Id)"

# Connect serial
Write-Host "[2] Connecting serial..." -ForegroundColor White
$client = New-Object System.Net.Sockets.TcpClient
for ($i = 0; $i -lt 60; $i++) {
    try { $client.Connect("127.0.0.1", $SerialPort); break } catch { Start-Sleep -Milliseconds 300 }
}
$stream = $client.GetStream()
$stream.ReadTimeout = 3000
$writer = New-Object System.IO.StreamWriter($stream)
$writer.AutoFlush = $true

# Wait for boot
Write-Host "[3] Waiting for boot..." -ForegroundColor White
Start-Sleep -Seconds 20

# Drain boot output
$bootOutput = Send-Command $writer $stream "" 5
Write-Host "  Boot complete." -ForegroundColor Green

# Init brain
Write-Host "[4] Initializing JARVIS brain..." -ForegroundColor White
$r = Send-Command $writer $stream "jarvis brain init" 15
Write-Host $r -ForegroundColor DarkGray

# Eval
Write-Host "`n[5] Running eval..." -ForegroundColor White
$r = Send-Command $writer $stream "jarvis brain eval" 15
Write-Host $r -ForegroundColor Yellow

# Chat tests
$prompts = @(
    "jarvis brain chat Hello",
    "jarvis brain chat who are you?",
    "jarvis brain chat What is TrustOS?",
    "jarvis brain chat help",
    "jarvis brain chat bonjour",
    "jarvis brain chat qui es-tu?",
    "jarvis brain chat tell me about yourself"
)

Write-Host "`n[6] Chat tests:" -ForegroundColor White
$allResults = ""
foreach ($p in $prompts) {
    Write-Host "`n  > $p" -ForegroundColor Cyan
    $r = Send-Command $writer $stream $p 20
    Write-Host "  $r" -ForegroundColor Green
    $allResults += "`n> $p`n$r`n"
}

# Save results
$allResults | Out-File "$PSScriptRoot\jarvis_gen_test.txt" -Encoding UTF8

Write-Host "`n=== Test Complete ===" -ForegroundColor Cyan
Write-Host "Results saved to jarvis_gen_test.txt"

# Cleanup
$client.Close()
Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
