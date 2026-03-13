# TrustOS Browser Test via Serial
# Assumes QEMU is already running with serial on port 5555 and network

param(
    [int]$SerialPort = 5555
)

function Read-Serial {
    param ($stream, [int]$waitMs = 3000)
    $buf = New-Object byte[] 65536
    $out = ""
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    while ($sw.ElapsedMilliseconds -lt $waitMs) {
        if ($stream.DataAvailable) {
            $rd = $stream.Read($buf, 0, $buf.Length)
            if ($rd -gt 0) { $out += [System.Text.Encoding]::UTF8.GetString($buf, 0, $rd) }
        }
        Start-Sleep -Milliseconds 30
    }
    return $out
}

function Send-Cmd {
    param ($stream, [string]$cmd, [int]$waitMs = 5000)
    $cmdBytes = [System.Text.Encoding]::ASCII.GetBytes("$cmd`r")
    $stream.Write($cmdBytes, 0, $cmdBytes.Length)
    $stream.Flush()
    Start-Sleep -Milliseconds 300
    return (Read-Serial $stream $waitMs)
}

try {
    $client = New-Object System.Net.Sockets.TcpClient
    $client.Connect("127.0.0.1", $SerialPort)
    Write-Host "Connected to serial port $SerialPort" -ForegroundColor Green
    $stream = $client.GetStream()
    $stream.ReadTimeout = 2000

    # Drain boot messages
    Write-Host "Draining boot messages..." -ForegroundColor Yellow
    $boot = Read-Serial $stream 3000
    Write-Host "Boot drain: $($boot.Length) bytes" -ForegroundColor DarkGray

    # Send Enter to get prompt
    $prompt = Send-Cmd $stream "" 1000
    Write-Host "Prompt: $prompt" -ForegroundColor DarkGray

    # Test 1: curl to test HTTP connectivity
    Write-Host "`n=== TEST 1: curl http://10.0.2.2:8080/ ===" -ForegroundColor Cyan
    $curlOut = Send-Cmd $stream "curl http://10.0.2.2:8080/" 8000
    Write-Host "curl output ($($curlOut.Length) bytes):" -ForegroundColor Green
    # Show first 2000 chars
    if ($curlOut.Length -gt 2000) {
        Write-Host ($curlOut.Substring(0, 2000))
        Write-Host "... (truncated)"
    } else {
        Write-Host $curlOut
    }

    # Test 2: browse to test browser parser
    Write-Host "`n=== TEST 2: browse http://10.0.2.2:8080/ ===" -ForegroundColor Cyan
    $browseOut = Send-Cmd $stream "browse http://10.0.2.2:8080/" 10000
    Write-Host "browse output ($($browseOut.Length) bytes):" -ForegroundColor Green
    if ($browseOut.Length -gt 3000) {
        Write-Host ($browseOut.Substring(0, 3000))
        Write-Host "... (truncated)"
    } else {
        Write-Host $browseOut
    }

    # Test 3: httpget for raw response
    Write-Host "`n=== TEST 3: httpget http://10.0.2.2:8080/ ===" -ForegroundColor Cyan
    $httpgetOut = Send-Cmd $stream "httpget http://10.0.2.2:8080/" 8000
    Write-Host "httpget output ($($httpgetOut.Length) bytes):" -ForegroundColor Green
    if ($httpgetOut.Length -gt 2000) {
        Write-Host ($httpgetOut.Substring(0, 2000))
        Write-Host "... (truncated)"
    } else {
        Write-Host $httpgetOut
    }

    # Save all results
    $allResults = @"
=== TrustOS Browser Test Results ===
Date: $(Get-Date)

=== CURL OUTPUT ===
$curlOut

=== BROWSE OUTPUT ===
$browseOut

=== HTTPGET OUTPUT ===
$httpgetOut
"@
    $allResults | Out-File -FilePath "browser_test_results.txt" -Encoding utf8
    Write-Host "`nResults saved to browser_test_results.txt" -ForegroundColor Green

    $client.Close()
} catch {
    Write-Host "Error: $_" -ForegroundColor Red
}
