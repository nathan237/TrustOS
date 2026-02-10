# =============================================================================
# TrustOS — Optimized VirtualBox Launch Script
# =============================================================================
# VirtualBox uses HARDWARE VIRTUALIZATION (VT-x/AMD-V) = much faster than QEMU TCG
# Optimized for maximum 3D/desktop performance
# =============================================================================

param(
    [string]$Resolution = "1920x1080",    # Display resolution
    [int]$RAM = 1024,                      # RAM in MB (1 GB default)
    [int]$VRAM = 256,                      # Video RAM in MB (max)
    [int]$CPUs = 4,                        # CPU cores
    [switch]$Rebuild,                      # Force rebuild
    [switch]$NoStart,                      # Just configure, don't start
    [switch]$Reset                         # Delete and recreate VM
)

$VBoxManage = "C:\Program Files\Oracle\VirtualBox\VBoxManage.exe"
$VMName = "TrustOS"
$ISOPath = "$PSScriptRoot\trustos.iso"
$KernelPath = "$PSScriptRoot\target\x86_64-unknown-none\release\trustos_kernel"
$SerialLog = "$PSScriptRoot\vbox_serial.txt"

Write-Host ""
Write-Host "  ╔═══════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "  ║        TrustOS — VirtualBox (Hardware Accelerated)       ║" -ForegroundColor Cyan
Write-Host "  ╠═══════════════════════════════════════════════════════════╣" -ForegroundColor Cyan
Write-Host "  ║  Resolution: $($Resolution.PadRight(13))  RAM: $("${RAM}MB".PadRight(8))  CPUs: $CPUs      ║" -ForegroundColor White
Write-Host "  ║  VRAM: ${VRAM}MB           Accel: VT-x/AMD-V (native)    ║" -ForegroundColor White
Write-Host "  ╚═══════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

# ─── Step 1: Build kernel if needed ───────────────────────────────────────────

if ($Rebuild -or !(Test-Path $KernelPath)) {
    Write-Host "[BUILD] Compiling kernel..." -ForegroundColor Yellow
    cargo build --release -p trustos_kernel 2>&1 | Select-Object -Last 3
    if (!(Test-Path $KernelPath)) {
        Write-Host "[ERROR] Build failed!" -ForegroundColor Red
        exit 1
    }
}

$KernelSize = [math]::Round((Get-Item $KernelPath).Length / 1KB, 1)
Write-Host "[OK] Kernel: $KernelSize KB" -ForegroundColor Green

# ─── Step 2: Update limine.conf with requested resolution ─────────────────────

$LimineConf = "TIMEOUT=3`n`n:TrustOS`nPROTOCOL=limine`nKERNEL_PATH=boot:///boot/trustos_kernel`nRESOLUTION=$Resolution"

[System.IO.File]::WriteAllText("$PSScriptRoot\limine.conf", $LimineConf)
[System.IO.File]::WriteAllText("$PSScriptRoot\iso_root\limine.conf", $LimineConf)
[System.IO.File]::WriteAllText("$PSScriptRoot\iso_root\boot\limine\limine.conf", $LimineConf)
Write-Host "[OK] limine.conf updated: RESOLUTION=$Resolution" -ForegroundColor Green

# ─── Step 3: Build ISO ────────────────────────────────────────────────────────

Write-Host "[ISO] Building bootable ISO..." -ForegroundColor Yellow

# Prepare iso_root
$IsoRoot = "$PSScriptRoot\iso_root"
New-Item -ItemType Directory -Force -Path "$IsoRoot\boot\limine" | Out-Null
New-Item -ItemType Directory -Force -Path "$IsoRoot\EFI\BOOT" | Out-Null

Copy-Item $KernelPath "$IsoRoot\boot\trustos_kernel" -Force
Copy-Item "$PSScriptRoot\limine\BOOTX64.EFI" "$IsoRoot\EFI\BOOT\BOOTX64.EFI" -Force
@("limine-bios.sys", "limine-bios-cd.bin", "limine-uefi-cd.bin") | ForEach-Object {
    Copy-Item "$PSScriptRoot\limine\$_" "$IsoRoot\boot\limine\$_" -Force -ErrorAction SilentlyContinue
}

# Create ISO
wsl bash -c "cd /mnt/c/Users/nathan/Documents/Scripts/OSrust && xorriso -as mkisofs -b boot/limine/limine-bios-cd.bin -no-emul-boot -boot-load-size 4 -boot-info-table --efi-boot boot/limine/limine-uefi-cd.bin -efi-boot-part --efi-boot-image --protective-msdos-label iso_root -o trustos.iso 2>&1 | tail -1"
& "$PSScriptRoot\limine\limine.exe" bios-install trustos.iso 2>&1 | Out-Null

$ISOSize = [math]::Round((Get-Item $ISOPath).Length / 1MB, 2)
Write-Host "[OK] ISO: $ISOSize MB" -ForegroundColor Green

# ─── Step 4: Configure VirtualBox VM ──────────────────────────────────────────

# Delete existing VM if reset requested or first time
$vmExists = & $VBoxManage list vms 2>$null | Select-String $VMName
if ($Reset) {
    Write-Host "[VM] Removing old VM..." -ForegroundColor Yellow
    & $VBoxManage controlvm $VMName poweroff 2>$null
    Start-Sleep -Seconds 2
    if ($vmExists) {
        & $VBoxManage unregistervm $VMName --delete 2>$null
    }
    # Clean up leftover files if VM dir still exists
    $VmDir = "$env:USERPROFILE\VirtualBox VMs\$VMName"
    if (Test-Path $VmDir) {
        Remove-Item $VmDir -Recurse -Force -ErrorAction SilentlyContinue
    }
    $vmExists = $null
}

if (!$vmExists) {
    Write-Host "[VM] Creating VM '$VMName'..." -ForegroundColor Yellow
    & $VBoxManage createvm --name $VMName --ostype "Other_64" --register 2>$null

    # Storage controller
    & $VBoxManage storagectl $VMName --name "SATA" --add sata --controller IntelAhci --portcount 2 2>$null
} else {
    # Stop running VM
    & $VBoxManage controlvm $VMName poweroff 2>$null
    Start-Sleep -Seconds 2
}

# ─── VM Configuration (optimized for performance) ─────────────────────────────

Write-Host "[VM] Configuring for max performance..." -ForegroundColor Yellow

# Core settings — hardware acceleration (VBox 7.2+ option names)
& $VBoxManage modifyvm $VMName `
    --memory $RAM `
    --vram $VRAM `
    --cpus $CPUs `
    --firmware efi64 `
    --graphicscontroller vboxsvga `
    --accelerate-3d off `
    --nested-hw-virt off `
    --x86-pae on `
    --x86-long-mode on `
    --large-pages on `
    --x86-vtx-vpid on `
    --nested-paging on

# Boot order
& $VBoxManage modifyvm $VMName --boot1 dvd --boot2 disk --boot3 none --boot4 none

# Performance: disable unnecessary features
& $VBoxManage modifyvm $VMName `
    --audio-enabled off `
    --clipboard-mode disabled `
    --drag-and-drop disabled `
    --usb-ehci off `
    --usb-xhci off

# Network: NAT (for internet access)
& $VBoxManage modifyvm $VMName `
    --nic1 nat `
    --nic-type1 82540EM `
    --cable-connected1 on

# Serial port (for debug output)
& $VBoxManage modifyvm $VMName `
    --uart1 0x3F8 4 `
    --uart-mode1 file "$SerialLog"

# Enable GUI scaling so the VM display scales when the window is resized
& $VBoxManage setextradata $VMName "GUI/ScaleFactor" "1.0"
& $VBoxManage setextradata $VMName "GUI/AutoresizeGuest" "false"
& $VBoxManage setextradata $VMName "CustomVideoMode1" "1920x1080x32"

# Attach ISO
& $VBoxManage storageattach $VMName --storagectl "SATA" --port 0 --device 0 --type dvddrive --medium $ISOPath 2>$null
# If medium already attached, modify it
& $VBoxManage storageattach $VMName --storagectl "SATA" --port 0 --device 0 --type dvddrive --medium $ISOPath --forceunmount 2>$null

# ─── Display Info ─────────────────────────────────────────────────────────────

Write-Host ""
Write-Host "  ┌─────────────────────────────────────────┐" -ForegroundColor Green
Write-Host "  │  VM Configuration                       │" -ForegroundColor Green
Write-Host "  ├─────────────────────────────────────────┤" -ForegroundColor Green
Write-Host "  │  RAM:        $("${RAM} MB".PadRight(27))│" -ForegroundColor White
Write-Host "  │  VRAM:       $("${VRAM} MB".PadRight(27))│" -ForegroundColor White
Write-Host "  │  CPUs:       $("$CPUs cores".PadRight(27))│" -ForegroundColor White
Write-Host "  │  Accel:      VT-x + Nested Paging       │" -ForegroundColor White
Write-Host "  │  Display:    $("$Resolution (VMSVGA)".PadRight(27))│" -ForegroundColor White
Write-Host "  │  Network:    NAT (Intel e1000)           │" -ForegroundColor White
Write-Host "  │  Serial:     vbox_serial.txt             │" -ForegroundColor White
Write-Host "  └─────────────────────────────────────────┘" -ForegroundColor Green
Write-Host ""

# ─── Step 5: Launch ───────────────────────────────────────────────────────────

if (!$NoStart) {
    Write-Host "[LAUNCH] Starting TrustOS in VirtualBox..." -ForegroundColor Cyan
    & $VBoxManage startvm $VMName --type gui

    Write-Host ""
    Write-Host "[OK] TrustOS running in VirtualBox!" -ForegroundColor Green
    Write-Host "     Serial: $SerialLog" -ForegroundColor Gray
    Write-Host ""
    Write-Host "     QEMU TCG (software emu) vs VirtualBox (VT-x hardware):" -ForegroundColor Yellow
    Write-Host "       - CPU:   ~5-10x faster (native execution)" -ForegroundColor White
    Write-Host "       - RAM:   $RAM MB (vs 512 MB QEMU)" -ForegroundColor White
    Write-Host "       - VRAM:  $VRAM MB (vs ~16 MB QEMU)" -ForegroundColor White
    Write-Host "       - Cores: $CPUs (all hardware-accelerated)" -ForegroundColor White
    Write-Host ""
} else {
    Write-Host "[OK] VM configured. Run: VBoxManage startvm $VMName" -ForegroundColor Green
}
