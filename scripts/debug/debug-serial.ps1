$QemuExe = "C:\Program Files\qemu\qemu-system-x86_64.exe"
$IsoPath = "C:\Users\nathan\Documents\Scripts\OSrust\trustos.iso"
$port = 5556

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
$s.ReadTimeout = 3000
$buf = New-Object byte[] 16384

# Wait for boot
$bootText = ""
$sw = [System.Diagnostics.Stopwatch]::StartNew()
while ($sw.Elapsed.TotalSeconds -lt 25) {
    if ($s.DataAvailable) {
        $r = $s.Read($buf, 0, $buf.Length)
        if ($r -gt 0) { $bootText += [System.Text.Encoding]::ASCII.GetString($buf, 0, $r) }
        if ($bootText -match "trustos") { break }
    } else {
        Start-Sleep -Milliseconds 150
    }
}
Write-Host ("Booted in {0}s" -f [math]::Round($sw.Elapsed.TotalSeconds,1))
Start-Sleep -Seconds 1

# Drain
while ($s.DataAvailable) { $s.Read($buf, 0, $buf.Length) | Out-Null }

# Send "id\r"
Write-Host "Sending: id"
$cmdBytes = [System.Text.Encoding]::ASCII.GetBytes("id`r")
$s.Write($cmdBytes, 0, $cmdBytes.Length)
$s.Flush()
Start-Sleep -Seconds 3

# Read raw output
$output = ""
while ($s.DataAvailable) {
    $r = $s.Read($buf, 0, $buf.Length)
    if ($r -gt 0) { $output += [System.Text.Encoding]::ASCII.GetString($buf, 0, $r) }
}
Write-Host ("=== RAW OUTPUT (len={0}) ===" -f $output.Length)
for ($i = 0; $i -lt [Math]::Min(800, $output.Length); $i++) {
    $ch = $output[$i]
    $code = [int][char]$ch
    if ($code -lt 32 -and $code -ne 10 -and $code -ne 13) {
        Write-Host -NoNewline ("[0x{0}]" -f $code.ToString('X2'))
    } else {
        Write-Host -NoNewline $ch
    }
}
Write-Host ""
Write-Host "=== END ==="

# Now send "tty\r" 
Write-Host ""
while ($s.DataAvailable) { $s.Read($buf, 0, $buf.Length) | Out-Null }
Write-Host "Sending: tty"
$cmdBytes = [System.Text.Encoding]::ASCII.GetBytes("tty`r")
$s.Write($cmdBytes, 0, $cmdBytes.Length)
$s.Flush()
Start-Sleep -Seconds 3
$output = ""
while ($s.DataAvailable) {
    $r = $s.Read($buf, 0, $buf.Length)
    if ($r -gt 0) { $output += [System.Text.Encoding]::ASCII.GetString($buf, 0, $r) }
}
Write-Host ("=== RAW OUTPUT (len={0}) ===" -f $output.Length)
for ($i = 0; $i -lt [Math]::Min(800, $output.Length); $i++) {
    $ch = $output[$i]
    $code = [int][char]$ch
    if ($code -lt 32 -and $code -ne 10 -and $code -ne 13) {
        Write-Host -NoNewline ("[0x{0}]" -f $code.ToString('X2'))
    } else {
        Write-Host -NoNewline $ch
    }
}
Write-Host ""
Write-Host "=== END ==="

# Cleanup
$c.Close()
Stop-Process -Id $q.Id -Force -ErrorAction SilentlyContinue
Write-Host "Done"
