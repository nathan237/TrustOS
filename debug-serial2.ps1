$QemuExe = "C:\Program Files\qemu\qemu-system-x86_64.exe"
$IsoPath = "C:\Users\nathan\Documents\Scripts\OSrust\trustos.iso"
$port = 5557

Get-Process -Name "qemu-system-x86_64" -ErrorAction SilentlyContinue | Stop-Process -Force 2>$null
Start-Sleep -Seconds 1

$q = Start-Process -FilePath $QemuExe -ArgumentList "-cdrom `"$IsoPath`" -m 256M -machine q35 -cpu max -smp 2 -accel tcg,thread=multi -display gtk -vga std -serial tcp:127.0.0.1:${port},server,nowait -no-reboot" -PassThru
Start-Sleep -Seconds 2

$c = New-Object System.Net.Sockets.TcpClient
for ($i = 0; $i -lt 60; $i++) {
    try { $c.Connect("127.0.0.1", $port); break }
    catch { Start-Sleep -Milliseconds 300 }
}

$s = $c.GetStream()
$s.ReadTimeout = 3000
$buf = New-Object byte[] 16384

$bootText = ""
$sw = [System.Diagnostics.Stopwatch]::StartNew()
while ($sw.Elapsed.TotalSeconds -lt 25) {
    if ($s.DataAvailable) {
        $r = $s.Read($buf, 0, $buf.Length)
        if ($r -gt 0) { $bootText += [System.Text.Encoding]::ASCII.GetString($buf, 0, $r) }
        if ($bootText -match "trustos") { break }
    } else { Start-Sleep -Milliseconds 150 }
}
Write-Host ("Booted in {0}s" -f [math]::Round($sw.Elapsed.TotalSeconds,1))
Start-Sleep -Seconds 1

function Send-Debug {
    param($stream, $cmd)
    # Drain
    while ($stream.DataAvailable) { $stream.Read($buf, 0, $buf.Length) | Out-Null }
    Start-Sleep -Milliseconds 200
    
    Write-Host "`n--- Sending: $cmd ---"
    $cmdBytes = [System.Text.Encoding]::ASCII.GetBytes("$cmd`r")
    $stream.Write($cmdBytes, 0, $cmdBytes.Length)
    $stream.Flush()
    Start-Sleep -Seconds 2
    
    $output = ""
    while ($stream.DataAvailable) {
        $r = $stream.Read($buf, 0, $buf.Length)
        if ($r -gt 0) { $output += [System.Text.Encoding]::ASCII.GetString($buf, 0, $r) }
    }
    Write-Host ("RAW len={0}:" -f $output.Length)
    $show = $output
    if ($show.Length -gt 500) { $show = $show.Substring(0, 500) }
    # Show with control chars visible
    for ($i = 0; $i -lt $show.Length; $i++) {
        $ch = $show[$i]
        $code = [int][char]$ch
        if ($code -lt 32 -and $code -ne 10 -and $code -ne 13) {
            Write-Host -NoNewline ("[0x{0}]" -f $code.ToString('X2'))
        } else {
            Write-Host -NoNewline $ch
        }
    }
    Write-Host ""
}

# Run a few "warm up" commands first to simulate being mid-test
Send-Debug $s "echo warmup1"
Send-Debug $s "echo warmup2"
Send-Debug $s "echo warmup3"

# Now test the commands that fail
Send-Debug $s "bc"
Send-Debug $s "base64"
Send-Debug $s "tty"
Send-Debug $s "blkid"
Send-Debug $s "disk"

# Cleanup
$c.Close()
Stop-Process -Id $q.Id -Force -ErrorAction SilentlyContinue
Write-Host "`nDone"
