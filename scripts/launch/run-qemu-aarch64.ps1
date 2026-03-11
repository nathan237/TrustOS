Set-Location "c:\Users\nathan\Documents\Scripts\OSrust"
Stop-Process -Name "qemu-system-aarch64" -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 1

$qemu = "C:\Program Files\qemu\qemu-system-aarch64.exe"
$fw   = "C:\Program Files\qemu\share\edk2-aarch64-code.fd"
$vars = "OVMF_VARS_aarch64.fd"
$log  = "serial_aarch64.log"

# Fresh NVRAM
[System.IO.File]::WriteAllBytes("$PWD\$vars", (New-Object byte[] (64*1024*1024)))

# Clear log
"" | Set-Content $log

$args = @(
    "-machine", "virt,gic-version=2",
    "-cpu", "cortex-a72",
    "-smp", "4",
    "-m", "512M",
    "-drive", "if=pflash,format=raw,readonly=on,file=$fw",
    "-drive", "if=pflash,format=raw,file=$PWD\$vars",
    "-drive", "format=raw,file=fat:rw:$PWD\iso_root_aarch64",
    "-serial", "file:$PWD\$log",
    "-display", "none",
    "-no-reboot"
)

$proc = Start-Process $qemu -ArgumentList $args -WindowStyle Hidden -PassThru
Write-Host "QEMU PID: $($proc.Id)"

# Wait and poll
for ($i = 1; $i -le 12; $i++) {
    Start-Sleep -Seconds 10
    $elapsed = $i * 10
    if ($proc.HasExited) {
        Write-Host "QEMU EXITED at ${elapsed}s with code $($proc.ExitCode)"
        break
    }
    $lines = @(Get-Content $log).Count
    Write-Host "${elapsed}s: $lines lines"
    if ($lines -gt 0) {
        Get-Content $log | Select-Object -Last 5
    }
}

# Final dump
Write-Host "`n=== FINAL SERIAL OUTPUT (last 40 lines) ==="
Get-Content $log | Select-Object -Last 40
