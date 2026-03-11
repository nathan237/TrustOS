# Quick boot test - verify ISO boots and brain module loads
$ErrorActionPreference = "Continue"

Stop-Process -Name qemu-system-x86_64 -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 5

$p = Start-Process -FilePath "C:\Program Files\qemu\qemu-system-x86_64.exe" -ArgumentList @(
    "-cdrom","trustos.iso","-m","256M","-smp","1","-cpu","Haswell",
    "-machine","q35","-accel","whpx","-display","none",
    "-serial","tcp:127.0.0.1:5590,server,nowait",
    "-no-reboot"
) -PassThru -WindowStyle Hidden

Write-Host "QEMU PID=$($p.Id)"
Start-Sleep -Seconds 10

if ($p.HasExited) {
    Write-Host "QEMU CRASHED: exit=$($p.ExitCode)" -ForegroundColor Red
    exit 1
}

Write-Host "QEMU running, connecting serial..."

$tcp = New-Object System.Net.Sockets.TcpClient("127.0.0.1", 5590)
$s = $tcp.GetStream()
$buf = New-Object byte[] 65536
$out = ""
$sw = [System.Diagnostics.Stopwatch]::StartNew()
while ($sw.Elapsed.TotalSeconds -lt 45) {
    if ($s.DataAvailable) {
        $n = $s.Read($buf, 0, $buf.Length)
        $out += [System.Text.Encoding]::ASCII.GetString($buf, 0, $n)
    } else { Start-Sleep -Milliseconds 100 }
    if ($out -match 'trustos:') { break }
}
$tcp.Close()

Write-Host ""
Write-Host "=== BOOT OUTPUT ($($out.Length) chars) ==="

if ($out -match 'trustos:') {
    Write-Host "BOOT: OK" -ForegroundColor Green
} else {
    Write-Host "BOOT: FAILED (no prompt)" -ForegroundColor Red
}

# Show key lines
foreach ($line in ($out -split "`n")) {
    $t = $line.Trim()
    if ($t -match 'JARVIS|Module|TSL|module|brain|RamFS|weights|PHASE|FAT32|VFS|OK') {
        Write-Host "  $t"
    }
}

Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
Write-Host "Done."
