# TrustOS — Multi-Architecture Build System
# Builds TrustOS kernel for one or more CPU architectures.
#
# Usage:
#   .\build-multiarch.ps1                    # Build all architectures
#   .\build-multiarch.ps1 -Arch x86_64       # Build x86_64 only
#   .\build-multiarch.ps1 -Arch aarch64      # Build ARM64 only
#   .\build-multiarch.ps1 -Arch riscv64      # Build RISC-V 64 only
#   .\build-multiarch.ps1 -Iso               # Build all + create ISOs

param(
    [ValidateSet("all", "x86_64", "aarch64", "riscv64")]
    [string]$Arch = "all",
    [switch]$Iso,
    [switch]$NoRun
)

$ErrorActionPreference = "Stop"

# Architecture → Rust target mapping
$targets = @{
    "x86_64"  = @{
        target    = "x86_64-unknown-none"
        linker_ld = "kernel/linker.ld"
        efi_boot  = "BOOTX64.EFI"
        qemu_bin  = "qemu-system-x86_64"
        qemu_args = @("-machine", "q35", "-cpu", "qemu64,+rdrand,+rdseed,+aes,+avx2,+sse4.2")
        iso_fmt   = "bios+uefi"
    }
    "aarch64" = @{
        target    = "aarch64-unknown-none"
        linker_ld = "kernel/linker-aarch64.ld"
        efi_boot  = "BOOTAA64.EFI"
        qemu_bin  = "qemu-system-aarch64"
        qemu_args = @("-machine", "virt", "-cpu", "cortex-a72")
        iso_fmt   = "uefi-only"
    }
    "riscv64" = @{
        target    = "riscv64gc-unknown-none-elf"
        linker_ld = "kernel/linker-riscv64.ld"
        efi_boot  = "BOOTRISCV64.EFI"
        qemu_bin  = "qemu-system-riscv64"
        qemu_args = @("-machine", "virt", "-cpu", "rv64")
        iso_fmt   = "uefi-only"
    }
}

$archList = if ($Arch -eq "all") { @("x86_64", "aarch64", "riscv64") } else { @($Arch) }

Write-Host "`n============================================================" -ForegroundColor Cyan
Write-Host "  TrustOS Multi-Architecture Build" -ForegroundColor Cyan
Write-Host "  Targets: $($archList -join ', ')" -ForegroundColor Cyan
Write-Host "  $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')" -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan

$results = @{}

foreach ($archName in $archList) {
    $cfg = $targets[$archName]
    $rustTarget = $cfg.target
    
    Write-Host "`n--- Building for $archName ($rustTarget) ---" -ForegroundColor Yellow
    
    $ErrorActionPreference = "Continue"
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    
    $output = cargo build --release -p trustos_kernel --target $rustTarget 2>&1
    $buildResult = $LASTEXITCODE
    
    $sw.Stop()
    $ErrorActionPreference = "Stop"
    
    $kernelPath = "target/$rustTarget/release/trustos_kernel"
    
    if ($buildResult -eq 0 -and (Test-Path $kernelPath)) {
        $size = (Get-Item $kernelPath).Length
        $sizeMB = [math]::Round($size / 1MB, 2)
        Write-Host "  OK  $archName — ${sizeMB} MB ($($sw.Elapsed.TotalSeconds.ToString('0.0'))s)" -ForegroundColor Green
        $results[$archName] = @{ status = "OK"; size = $sizeMB; time = $sw.Elapsed.TotalSeconds }
    } else {
        Write-Host "  FAIL  $archName" -ForegroundColor Red
        # Show last error lines
        $output | Select-String "error" | Select-Object -Last 5 | ForEach-Object { Write-Host "    $_" -ForegroundColor Red }
        $results[$archName] = @{ status = "FAIL"; size = 0; time = $sw.Elapsed.TotalSeconds }
    }
}

# ============================================================================
# Create ISOs (if requested)
# ============================================================================
if ($Iso) {
    foreach ($archName in $archList) {
        if ($results[$archName].status -ne "OK") { continue }
        
        $cfg = $targets[$archName]
        $rustTarget = $cfg.target
        $kernelPath = "target/$rustTarget/release/trustos_kernel"
        $isoPath = "trustos-$archName.iso"
        
        Write-Host "`n--- Creating ISO for $archName ---" -ForegroundColor Yellow
        
        $isoDir = "iso_root_$archName"
        if (Test-Path $isoDir) { Remove-Item -Recurse -Force $isoDir }
        New-Item -ItemType Directory -Path $isoDir | Out-Null
        New-Item -ItemType Directory -Path "$isoDir/boot" | Out-Null
        New-Item -ItemType Directory -Path "$isoDir/boot/limine" | Out-Null
        New-Item -ItemType Directory -Path "$isoDir/EFI/BOOT" -Force | Out-Null
        
        Copy-Item $kernelPath "$isoDir/boot/trustos_kernel"
        Copy-Item "limine.cfg" "$isoDir/boot/limine/limine.cfg"
        Copy-Item "limine.cfg" "$isoDir/boot/limine/limine.conf"
        Copy-Item "limine.cfg" "$isoDir/limine.conf"
        
        # Copy arch-specific EFI bootloader
        $efiFile = $cfg.efi_boot
        if (Test-Path "limine/$efiFile") {
            Copy-Item "limine/$efiFile" "$isoDir/EFI/BOOT/$efiFile"
        }
        
        # x86_64 gets BIOS boot too
        if ($archName -eq "x86_64") {
            Copy-Item "limine/limine-bios.sys" "$isoDir/boot/limine/"
            Copy-Item "limine/limine-bios-cd.bin" "$isoDir/boot/limine/"
            Copy-Item "limine/limine-uefi-cd.bin" "$isoDir/boot/limine/"
            Copy-Item "limine/BOOTIA32.EFI" "$isoDir/EFI/BOOT/"
        } else {
            # UEFI-only: still need the UEFI CD binary
            if (Test-Path "limine/limine-uefi-cd.bin") {
                Copy-Item "limine/limine-uefi-cd.bin" "$isoDir/boot/limine/"
            }
        }
        
        Write-Host "  ISO dir created: $isoDir" -ForegroundColor Green
    }
}

# ============================================================================
# Summary
# ============================================================================
Write-Host "`n============================================================" -ForegroundColor Cyan
Write-Host "  BUILD RESULTS" -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan

$totalOk = 0
$totalFail = 0

foreach ($archName in $archList) {
    $r = $results[$archName]
    if ($r.status -eq "OK") {
        Write-Host "  $($archName.PadRight(10)) OK   $($r.size) MB  ($($r.time.ToString('0.0'))s)" -ForegroundColor Green
        $totalOk++
    } else {
        Write-Host "  $($archName.PadRight(10)) FAIL" -ForegroundColor Red
        $totalFail++
    }
}

Write-Host "`n  Total: $totalOk/$($archList.Count) architectures built" -ForegroundColor $(if ($totalFail -eq 0) { "Green" } else { "Yellow" })
Write-Host "============================================================`n" -ForegroundColor Cyan

if ($totalFail -gt 0) { exit 1 }
