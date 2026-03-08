Set-Location "c:\Users\nathan\Documents\Scripts\OSrust"
$ErrorActionPreference = "Stop"

# 1. Kill old QEMU
Stop-Process -Name "qemu-system-aarch64" -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 1

# 2. Build
Write-Host "=== BUILDING ==="
$buildOutput = cargo build --release --target aarch64-unknown-none -p trustos_kernel 2>&1
$lastLine = ($buildOutput | Select-Object -Last 1)
Write-Host "Build result: $lastLine"
if ($lastLine -notmatch "Finished") {
    Write-Host "BUILD FAILED"
    $buildOutput | Where-Object { $_ -match "^error" } | Select-Object -First 10
    exit 1
}

# 3. Deploy
Copy-Item "target\aarch64-unknown-none\release\trustos_kernel" "iso_root_aarch64\boot\trustos_kernel" -Force
Write-Host "Deployed kernel: $((Get-Item 'iso_root_aarch64\boot\trustos_kernel').Length) bytes"

# 4. Fresh NVRAM
[System.IO.File]::WriteAllBytes("$PWD\OVMF_VARS_aarch64.fd", (New-Object byte[] (64*1024*1024)))
"" | Set-Content "serial_aarch64.log"

# 5. Launch QEMU
$qemu = "C:\Program Files\qemu\qemu-system-aarch64.exe"
$fw   = "C:\Program Files\qemu\share\edk2-aarch64-code.fd"
$proc = Start-Process $qemu -ArgumentList @(
    "-machine", "virt,gic-version=2",
    "-cpu", "cortex-a72",
    "-smp", "4",
    "-m", "512M",
    "-drive", "if=pflash,format=raw,readonly=on,file=$fw",
    "-drive", "if=pflash,format=raw,file=$PWD\OVMF_VARS_aarch64.fd",
    "-drive", "format=raw,file=fat:rw:$PWD\iso_root_aarch64",
    "-serial", "file:$PWD\serial_aarch64.log",
    "-display", "none",
    "-no-reboot"
) -WindowStyle Hidden -PassThru

Write-Host "QEMU PID: $($proc.Id)"

# 6. Monitor
$prevLines = 0
for ($i = 1; $i -le 18; $i++) {
    Start-Sleep -Seconds 10
    $elapsed = $i * 10
    if ($proc.HasExited) {
        Write-Host "`n=== QEMU EXITED at ${elapsed}s (code $($proc.ExitCode)) ==="
        break
    }
    $content = @(Get-Content "serial_aarch64.log" -ErrorAction SilentlyContinue)
    $lines = $content.Count
    if ($lines -ne $prevLines) {
        Write-Host "${elapsed}s: $lines lines (+$($lines - $prevLines))"
        $prevLines = $lines
    } else {
        Write-Host "${elapsed}s: $lines lines (no change)"
        if ($lines -gt 0 -and $i -ge 6) {
            Write-Host "  (appears stuck, dumping last 10 lines)"
            $content | Select-Object -Last 10
            break
        }
    }
}

Write-Host "`n=== SERIAL OUTPUT (last 40 lines) ==="
Get-Content "serial_aarch64.log" | Select-Object -Last 40
