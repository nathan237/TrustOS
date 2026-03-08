$ErrorActionPreference = "Continue"

# Kill existing
Get-Process -Name "qemu-system-x86_64" -ErrorAction SilentlyContinue | Stop-Process -Force 2>$null
Start-Sleep -Seconds 1

$QemuExe = "C:\Program Files\qemu\qemu-system-x86_64.exe"
$port = 5558

$proc = Start-Process -FilePath $QemuExe -ArgumentList @(
    "-cdrom", "$PSScriptRoot\trustos.iso",
    "-m", "512M",
    "-machine", "q35",
    "-cpu", "max",
    "-accel", "tcg",
    "-display", "none",
    "-serial", "tcp:127.0.0.1:${port},server,nowait",
    "-no-reboot"
) -PassThru -WindowStyle Hidden

Write-Output "QEMU PID: $($proc.Id)"

# Connect to serial with retries
$client = $null
for ($i = 0; $i -lt 40; $i++) {
    try {
        $client = New-Object System.Net.Sockets.TcpClient("127.0.0.1", $port)
        Write-Output "Serial connected (attempt $i)"
        break
    } catch {
        Start-Sleep -Milliseconds 500
    }
}

if (-not $client -or -not $client.Connected) {
    Write-Output "FATAL: Could not connect to serial"
    if (-not $proc.HasExited) { Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue }
    exit 1
}

$stream = $client.GetStream()
$writer = New-Object System.IO.StreamWriter($stream)
$writer.AutoFlush = $true
$buffer = New-Object byte[] 65536

function Read-Serial($seconds) {
    $output = ""
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    while ($sw.Elapsed.TotalSeconds -lt $seconds) {
        if ($stream.DataAvailable) {
            $read = $stream.Read($buffer, 0, $buffer.Length)
            if ($read -gt 0) { $output += [System.Text.Encoding]::ASCII.GetString($buffer, 0, $read) }
        } else {
            Start-Sleep -Milliseconds 100
        }
    }
    return $output
}

function Send-Cmd($cmd, $wait) {
    # Drain
    $drain = Read-Serial 0.3
    # Send
    $writer.Write("$cmd`r")
    $writer.Flush()
    # Wait for response
    return Read-Serial $wait
}

# Wait for boot (25s for TCG)
Write-Output "Waiting for boot..."
$bootOutput = Read-Serial 25
Write-Output "Boot output: $($bootOutput.Length) bytes"

# Test 1: jarvis brain init
Write-Output ""
Write-Output "=== TEST: jarvis brain init ==="
$result = Send-Cmd "jarvis brain init" 10
$result -split "`n" | ForEach-Object { Write-Output $_.Trim() }

# Test 2: jarvis brain info
Write-Output ""
Write-Output "=== TEST: jarvis brain info ==="
$result = Send-Cmd "jarvis brain info" 5
$result -split "`n" | ForEach-Object { Write-Output $_.Trim() }

# Test 3: jarvis brain status (compact stats)
Write-Output ""
Write-Output "=== TEST: jarvis brain status ==="
$result = Send-Cmd "jarvis brain status" 5
$result -split "`n" | ForEach-Object { Write-Output $_.Trim() }

# Test 4: jarvis chat (test generation routing)
Write-Output ""
Write-Output "=== TEST: jarvis chat hello ==="
$result = Send-Cmd "jarvis chat hello" 8
$result -split "`n" | ForEach-Object { Write-Output $_.Trim() }

# Cleanup
$client.Close()
if (-not $proc.HasExited) { Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue }
Write-Output ""
Write-Output "=== All tests complete ==="
