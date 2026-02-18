<#
.SYNOPSIS
    TrustOS VM Debug - Run guest and capture debug output
#>

$QemuExe = "C:\Program Files\qemu\qemu-system-x86_64.exe"
$IsoPath = "$PSScriptRoot\trustos.iso"
$port = 5559
$OutputFile = "$PSScriptRoot\serial_debug_vm2.txt"

Write-Host "=== TrustOS VM Debug - Guest Run ===" -ForegroundColor Cyan

# Build ISO
$KernelPath = "$PSScriptRoot\target\x86_64-unknown-none\release\trustos_kernel"
$IsoRoot = "$PSScriptRoot\iso_root"
New-Item -ItemType Directory -Force -Path "$IsoRoot\boot\limine" | Out-Null
New-Item -ItemType Directory -Force -Path "$IsoRoot\EFI\BOOT" | Out-Null
Copy-Item $KernelPath "$IsoRoot\boot\trustos_kernel" -Force
Copy-Item "$PSScriptRoot\limine\BOOTX64.EFI" "$IsoRoot\EFI\BOOT\BOOTX64.EFI" -Force
Copy-Item "$PSScriptRoot\limine\limine-bios.sys" "$IsoRoot\boot\limine\limine-bios.sys" -Force -ErrorAction SilentlyContinue
Copy-Item "$PSScriptRoot\limine\limine-bios-cd.bin" "$IsoRoot\boot\limine\limine-bios-cd.bin" -Force -ErrorAction SilentlyContinue
Copy-Item "$PSScriptRoot\limine\limine-uefi-cd.bin" "$IsoRoot\boot\limine\limine-uefi-cd.bin" -Force -ErrorAction SilentlyContinue
Copy-Item "$PSScriptRoot\limine.conf" "$IsoRoot\limine.conf" -Force
Copy-Item "$PSScriptRoot\limine.conf" "$IsoRoot\boot\limine\limine.conf" -Force

$winRoot = (Resolve-Path $PSScriptRoot).Path
$driveLetter = $winRoot.Substring(0,1).ToLower()
$wslRoot = "/mnt/$driveLetter" + ($winRoot.Substring(2) -replace '\\', '/')
wsl -e xorriso -as mkisofs -b boot/limine/limine-bios-cd.bin `
    -no-emul-boot -boot-load-size 4 -boot-info-table `
    --efi-boot boot/limine/limine-uefi-cd.bin `
    -efi-boot-part --efi-boot-image --protective-msdos-label `
    -o "$wslRoot/trustos.iso" "$wslRoot/iso_root" 2>&1 | Out-Null
Write-Host "ISO ready" -ForegroundColor Green

# Kill leftover QEMU
Get-Process -Name "qemu-system-x86_64" -ErrorAction SilentlyContinue | Stop-Process -Force 2>$null
Start-Sleep -Seconds 1

# Launch QEMU
Write-Host "Launching QEMU..." -ForegroundColor Yellow
$q = Start-Process -FilePath $QemuExe -ArgumentList @(
    "-cdrom", "`"$IsoPath`"",
    "-m", "512M", "-machine", "q35", "-cpu", "max", "-smp", "2",
    "-accel", "tcg,thread=multi", "-display", "gtk", "-vga", "std",
    "-serial", "tcp:127.0.0.1:${port},server,nowait", "-no-reboot"
) -PassThru
Write-Host "QEMU PID: $($q.Id)" -ForegroundColor Gray
Start-Sleep -Seconds 2

# Connect
$c = New-Object System.Net.Sockets.TcpClient
for ($i = 0; $i -lt 60; $i++) {
    try { $c.Connect("127.0.0.1", $port); break }
    catch { Start-Sleep -Milliseconds 500 }
}
$s = $c.GetStream()
$s.ReadTimeout = 5000
$buf = New-Object byte[] 32768
$allOutput = "=== TrustOS VM Debug Session 2 - $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss') ===`r`n`r`n"

# Wait for boot
$bootText = ""
$sw = [System.Diagnostics.Stopwatch]::StartNew()
while ($sw.Elapsed.TotalSeconds -lt 35) {
    if ($s.DataAvailable) {
        $r = $s.Read($buf, 0, $buf.Length)
        if ($r -gt 0) { $bootText += [System.Text.Encoding]::ASCII.GetString($buf, 0, $r) }
        if ($bootText -match "trustos:/\$") { break }
    } else { Start-Sleep -Milliseconds 200 }
}
Write-Host "Booted in $([math]::Round($sw.Elapsed.TotalSeconds,1))s" -ForegroundColor Green
$allOutput += "=== BOOT ===`r`n$bootText`r`n`r`n"
Start-Sleep -Seconds 1

function Send-Cmd {
    param([string]$cmd, [int]$timeout = 8)
    while ($s.DataAvailable) { $s.Read($buf, 0, $buf.Length) | Out-Null }
    Start-Sleep -Milliseconds 300
    Write-Host "`n>>> $cmd" -ForegroundColor Cyan
    $cmdBytes = [System.Text.Encoding]::ASCII.GetBytes("$cmd`r")
    $s.Write($cmdBytes, 0, $cmdBytes.Length); $s.Flush()
    $output = ""
    $sw2 = [System.Diagnostics.Stopwatch]::StartNew()
    while ($sw2.Elapsed.TotalSeconds -lt $timeout) {
        if ($s.DataAvailable) {
            $r = $s.Read($buf, 0, $buf.Length)
            if ($r -gt 0) { $output += [System.Text.Encoding]::ASCII.GetString($buf, 0, $r) }
        } else { Start-Sleep -Milliseconds 100 }
        if ($sw2.ElapsedMilliseconds -ge 800 -and $output.Length -gt 3) {
            if ($output -match "trustos:[^\r\n]*\$\s*$") {
                Start-Sleep -Milliseconds 200
                while ($s.DataAvailable) {
                    $r = $s.Read($buf, 0, $buf.Length)
                    if ($r -gt 0) { $output += [System.Text.Encoding]::ASCII.GetString($buf, 0, $r) }
                }
                break
            }
        }
    }
    $clean = ($output -split "`r?`n" | ForEach-Object { $_.TrimEnd() }) -join "`n"
    Write-Host $clean -ForegroundColor White
    $script:allOutput += ">>> $cmd`r`n$clean`r`n`r`n"
    return $output
}

# ---- Test sequence ----
Write-Host "`n=== 1. INIT DEBUG MONITOR ===" -ForegroundColor Magenta
Send-Cmd "vm debug init"
Send-Cmd "vm debug status"

Write-Host "`n=== 2. CHECK HYPERVISOR ===" -ForegroundColor Magenta
Send-Cmd "hv status"
Send-Cmd "hv init"

Write-Host "`n=== 3. LIST AVAILABLE GUESTS ===" -ForegroundColor Magenta
Send-Cmd "vm guests"

Write-Host "`n=== 4. RUN A GUEST VM ===" -ForegroundColor Magenta
Send-Cmd "vm run hello" 15
Send-Cmd "vm run pm-test" 15

Write-Host "`n=== 5. DEBUG RESULTS ===" -ForegroundColor Magenta
Send-Cmd "vm debug status"
Send-Cmd "vm debug" 10
Send-Cmd "vm debug gaps" 10
Send-Cmd "vm debug io" 10
Send-Cmd "vm debug msr" 10
Send-Cmd "vm debug timeline 50" 10

Write-Host "`n=== 6. EXTRA DIAGNOSTICS ===" -ForegroundColor Magenta
Send-Cmd "vm list"
Send-Cmd "vm inspect" 10
Send-Cmd "test" 60

# Save
$allOutput | Out-File -FilePath $OutputFile -Encoding UTF8
Write-Host "`n=== DONE ===" -ForegroundColor Green
Write-Host "Output: $OutputFile" -ForegroundColor Cyan

$c.Close()
Stop-Process -Id $q.Id -Force -ErrorAction SilentlyContinue
