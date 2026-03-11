$QemuExe = "C:\Program Files\qemu\qemu-system-x86_64.exe"
$IsoPath = "C:\Users\nathan\Documents\Scripts\OSrust\trustos.iso"
$port = 5557

Get-Process -Name "qemu-system-x86_64" -ErrorAction SilentlyContinue | Stop-Process -Force 2>$null
Start-Sleep -Seconds 1

$q = Start-Process -FilePath $QemuExe -ArgumentList "-cdrom `"$IsoPath`" -m 256M -machine q35 -cpu max -smp 2 -accel tcg,thread=multi -display gtk -vga std -serial tcp:127.0.0.1:${port},server,nowait -no-reboot" -PassThru
Write-Host "QEMU PID: $($q.Id)"
Start-Sleep -Seconds 2

$c = New-Object System.Net.Sockets.TcpClient
for ($i = 0; $i -lt 60; $i++) {
    try { $c.Connect("127.0.0.1", $port); break }
    catch { Start-Sleep -Milliseconds 300 }
}

$s = $c.GetStream()
$s.ReadTimeout = 5000
$buf = New-Object byte[] 16384

# Wait for boot - look for prompt
$bootText = ""
$sw = [System.Diagnostics.Stopwatch]::StartNew()
while ($sw.Elapsed.TotalSeconds -lt 30) {
    if ($s.DataAvailable) {
        $r = $s.Read($buf, 0, $buf.Length)
        if ($r -gt 0) { $bootText += [System.Text.Encoding]::ASCII.GetString($buf, 0, $r) }
        if ($bootText -match "trustos:/\$") { break }
    } else {
        Start-Sleep -Milliseconds 200
    }
}
Write-Host ("Booted in {0}s" -f [math]::Round($sw.Elapsed.TotalSeconds,1))
Start-Sleep -Seconds 2

function Send-Debug($cmd) {
    # Drain anything pending
    while ($s.DataAvailable) { $s.Read($buf, 0, $buf.Length) | Out-Null }
    Start-Sleep -Milliseconds 200
    
    Write-Host ""
    Write-Host "========================================" 
    Write-Host "SENDING: $cmd"
    Write-Host "========================================" 
    
    $cmdBytes = [System.Text.Encoding]::ASCII.GetBytes("$cmd`r")
    $s.Write($cmdBytes, 0, $cmdBytes.Length)
    $s.Flush()
    
    # Wait and collect output for 4 seconds
    $output = ""
    $sw2 = [System.Diagnostics.Stopwatch]::StartNew()
    while ($sw2.Elapsed.TotalSeconds -lt 4) {
        if ($s.DataAvailable) {
            $r = $s.Read($buf, 0, $buf.Length)
            if ($r -gt 0) { $output += [System.Text.Encoding]::ASCII.GetString($buf, 0, $r) }
        } else {
            Start-Sleep -Milliseconds 100
        }
    }
    
    Write-Host "RAW (len=$($output.Length)):"
    # Show with hex for control chars
    $display = ""
    for ($i = 0; $i -lt [Math]::Min(1200, $output.Length); $i++) {
        $ch = $output[$i]
        $code = [int][char]$ch
        if ($code -lt 32 -and $code -ne 10 -and $code -ne 13) {
            $display += "[0x$($code.ToString('X2'))]"
        } else {
            $display += $ch
        }
    }
    Write-Host $display
    Write-Host "--- LINES ---"
    $lines = $output -split "`r?`n"
    for ($li = 0; $li -lt $lines.Count; $li++) {
        Write-Host ("  L{0}: [{1}]" -f $li, $lines[$li].TrimEnd())
    }
    Write-Host "--- END ---"
}

# Test the 8 problematic commands
Send-Debug "bc 2+3"
Send-Debug "base64 hello"
Send-Debug "md5sum hello"
Send-Debug "sha256sum hello"
Send-Debug "tty"
Send-Debug "disk"
Send-Debug "fdisk"
Send-Debug "blkid"

# Cleanup
$c.Close()
Stop-Process -Id $q.Id -Force -ErrorAction SilentlyContinue
Write-Host "`nDone"
