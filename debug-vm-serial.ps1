<#
.SYNOPSIS
    TrustOS VM Debug Monitor - Serial Debug Session
.DESCRIPTION
    Boots TrustOS in QEMU, connects via serial TCP, initializes the VM debug
    monitor, runs diagnostic commands, and captures all output for analysis.
#>

$QemuExe = "C:\Program Files\qemu\qemu-system-x86_64.exe"
$IsoPath = "$PSScriptRoot\trustos.iso"
$port = 5558
$OutputFile = "$PSScriptRoot\serial_debug_vm.txt"

Write-Host "=== TrustOS VM Debug - Serial Session ===" -ForegroundColor Cyan

# Build ISO first
Write-Host "Building ISO..." -ForegroundColor Yellow
$KernelPath = "$PSScriptRoot\target\x86_64-unknown-none\release\trustos_kernel"
if (!(Test-Path $KernelPath)) {
    Write-Host "ERROR: Kernel not found. Run: cargo build --release" -ForegroundColor Red
    exit 1
}

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

# Launch QEMU with serial TCP
Write-Host "Launching QEMU (serial on TCP port $port)..." -ForegroundColor Yellow
$q = Start-Process -FilePath $QemuExe -ArgumentList @(
    "-cdrom", "`"$IsoPath`"",
    "-m", "512M",
    "-machine", "q35",
    "-cpu", "max",
    "-smp", "2",
    "-accel", "tcg,thread=multi",
    "-display", "gtk",
    "-vga", "std",
    "-serial", "tcp:127.0.0.1:${port},server,nowait",
    "-no-reboot"
) -PassThru
Write-Host "QEMU PID: $($q.Id)" -ForegroundColor Gray
Start-Sleep -Seconds 2

# Connect to serial
Write-Host "Connecting to serial..." -ForegroundColor Yellow
$c = New-Object System.Net.Sockets.TcpClient
for ($i = 0; $i -lt 60; $i++) {
    try { $c.Connect("127.0.0.1", $port); break }
    catch { Start-Sleep -Milliseconds 500 }
}
if (-not $c.Connected) {
    Write-Host "ERROR: Cannot connect to serial" -ForegroundColor Red
    Stop-Process -Id $q.Id -Force -ErrorAction SilentlyContinue
    exit 1
}
$s = $c.GetStream()
$s.ReadTimeout = 5000
$buf = New-Object byte[] 32768
Write-Host "Serial connected!" -ForegroundColor Green

# Capture everything to file
$allOutput = "=== TrustOS VM Debug Session - $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss') ===`r`n`r`n"

# Wait for TrustOS boot
Write-Host "Waiting for TrustOS to boot..." -ForegroundColor Yellow
$bootText = ""
$sw = [System.Diagnostics.Stopwatch]::StartNew()
while ($sw.Elapsed.TotalSeconds -lt 35) {
    if ($s.DataAvailable) {
        $r = $s.Read($buf, 0, $buf.Length)
        if ($r -gt 0) { 
            $chunk = [System.Text.Encoding]::ASCII.GetString($buf, 0, $r)
            $bootText += $chunk
        }
        if ($bootText -match "trustos:/\$") { break }
    } else {
        Start-Sleep -Milliseconds 200
    }
}
$bootTime = [math]::Round($sw.Elapsed.TotalSeconds, 1)
Write-Host "Booted in ${bootTime}s" -ForegroundColor Green
$allOutput += "=== BOOT OUTPUT ===`r`n$bootText`r`n`r`n"
Start-Sleep -Seconds 1

# --- Send command helper ---
function Send-Cmd {
    param([string]$cmd, [int]$timeout = 5)
    
    # Drain
    while ($s.DataAvailable) { $s.Read($buf, 0, $buf.Length) | Out-Null }
    Start-Sleep -Milliseconds 300
    
    Write-Host "`n>>> $cmd" -ForegroundColor Cyan
    
    $cmdBytes = [System.Text.Encoding]::ASCII.GetBytes("$cmd`r")
    $s.Write($cmdBytes, 0, $cmdBytes.Length)
    $s.Flush()
    
    $output = ""
    $sw2 = [System.Diagnostics.Stopwatch]::StartNew()
    while ($sw2.Elapsed.TotalSeconds -lt $timeout) {
        if ($s.DataAvailable) {
            $r = $s.Read($buf, 0, $buf.Length)
            if ($r -gt 0) { $output += [System.Text.Encoding]::ASCII.GetString($buf, 0, $r) }
        } else {
            Start-Sleep -Milliseconds 100
        }
        # Check for prompt
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
    
    # Clean display
    $clean = ($output -split "`r?`n" | ForEach-Object { $_.TrimEnd() }) -join "`n"
    Write-Host $clean -ForegroundColor White
    
    $script:allOutput += ">>> $cmd`r`n$clean`r`n`r`n"
    return $output
}

# ---------------------------------------------------------------
#   DEBUG COMMANDS SEQUENCE
# ---------------------------------------------------------------

Write-Host "`n=== SYSTEM INFO ===" -ForegroundColor Magenta
Send-Cmd "uname -a"
Send-Cmd "cpuinfo"
Send-Cmd "mem"

Write-Host "`n=== VM HYPERVISOR CHECK ===" -ForegroundColor Magenta
Send-Cmd "vm status"
Send-Cmd "vm info"

Write-Host "`n=== VM DEBUG MONITOR ===" -ForegroundColor Magenta
Send-Cmd "vm debug init"
Send-Cmd "vm debug status"

Write-Host "`n=== VM OPERATIONS ===" -ForegroundColor Magenta
Send-Cmd "vm list"
Send-Cmd "vm inspect" 6

Write-Host "`n=== DEBUG DASHBOARD ===" -ForegroundColor Magenta
Send-Cmd "vm debug" 6
Send-Cmd "vm debug gaps" 6
Send-Cmd "vm debug io" 6
Send-Cmd "vm debug msr" 6
Send-Cmd "vm debug timeline" 6

Write-Host "`n=== ADDITIONAL DIAGNOSTICS ===" -ForegroundColor Magenta
Send-Cmd "pci"
Send-Cmd "disk"
Send-Cmd "net status"
Send-Cmd "ps"
Send-Cmd "selftest" 30

# ---------------------------------------------------------------
#   SAVE REPORT
# ---------------------------------------------------------------

$allOutput | Out-File -FilePath $OutputFile -Encoding UTF8
Write-Host "`n=== SESSION COMPLETE ===" -ForegroundColor Green
Write-Host "Full output saved to: $OutputFile" -ForegroundColor Cyan
Write-Host "Boot time: ${bootTime}s" -ForegroundColor Gray

# Cleanup
$c.Close()
Stop-Process -Id $q.Id -Force -ErrorAction SilentlyContinue
Write-Host "QEMU terminated. Done." -ForegroundColor Gray
