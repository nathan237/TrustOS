# TRustOs - VirtualBox Setup Script (Limine UEFI)

$VBoxManage = "C:\Program Files\Oracle\VirtualBox\VBoxManage.exe"
$VMName = "TRustOs"
$ISOPath = "$PSScriptRoot\trustos.iso"
$KernelPath = "$PSScriptRoot\target\x86_64-unknown-none\release\trustos_kernel"

Write-Host "=== TRustOs - VirtualBox Setup (UEFI) ===" -ForegroundColor Cyan

# Check kernel exists
if (!(Test-Path $KernelPath)) {
    Write-Host "ERROR: Kernel not found at $KernelPath" -ForegroundColor Red
    Write-Host "Run: cargo build --release" -ForegroundColor Yellow
    exit 1
}

Write-Host "Kernel found: $KernelPath ($([math]::Round((Get-Item $KernelPath).Length/1KB, 1)) KB)" -ForegroundColor Green

# Build ISO with Limine
Write-Host "`nBuilding bootable ISO..." -ForegroundColor Yellow

$IsoRoot = "$PSScriptRoot\iso_root"

# Ensure iso_root structure
New-Item -ItemType Directory -Force -Path "$IsoRoot\boot\limine" | Out-Null
New-Item -ItemType Directory -Force -Path "$IsoRoot\EFI\BOOT" | Out-Null

# Copy kernel
Copy-Item $KernelPath "$IsoRoot\boot\trustos_kernel" -Force

# Copy Limine files
Copy-Item "$PSScriptRoot\limine\BOOTX64.EFI" "$IsoRoot\EFI\BOOT\BOOTX64.EFI" -Force
Copy-Item "$PSScriptRoot\limine\limine-bios.sys" "$IsoRoot\boot\limine\limine-bios.sys" -Force -ErrorAction SilentlyContinue
Copy-Item "$PSScriptRoot\limine\limine-bios-cd.bin" "$IsoRoot\boot\limine\limine-bios-cd.bin" -Force -ErrorAction SilentlyContinue
Copy-Item "$PSScriptRoot\limine\limine-uefi-cd.bin" "$IsoRoot\boot\limine\limine-uefi-cd.bin" -Force -ErrorAction SilentlyContinue

# Copy limine.conf
Copy-Item "$PSScriptRoot\limine.conf" "$IsoRoot\limine.conf" -Force
Copy-Item "$PSScriptRoot\limine.conf" "$IsoRoot\boot\limine\limine.conf" -Force

Write-Host "ISO root prepared at: $IsoRoot" -ForegroundColor Green

# Create ISO using WSL xorriso
Write-Host "Creating ISO with WSL xorriso..." -ForegroundColor Yellow
$wslIsoRoot = "/mnt/c/Users/nathan/Documents/Scripts/OSrust/iso_root"
$wslIsoPath = "/mnt/c/Users/nathan/Documents/Scripts/OSrust/trustos.iso"
wsl -e xorriso -as mkisofs -b boot/limine/limine-bios-cd.bin `
    -no-emul-boot -boot-load-size 4 -boot-info-table `
    --efi-boot boot/limine/limine-uefi-cd.bin `
    -efi-boot-part --efi-boot-image --protective-msdos-label `
    -o $wslIsoPath $wslIsoRoot 2>&1 | Out-Null
Write-Host "ISO created: $ISOPath" -ForegroundColor Green

# Remove existing VM if exists
Write-Host "`nCleaning old VM..." -ForegroundColor Yellow
& $VBoxManage unregistervm $VMName --delete 2>$null

# Create VM
Write-Host "`nCreating VM '$VMName'..." -ForegroundColor Yellow
& $VBoxManage createvm --name $VMName --ostype "Other_64" --register

# Configure VM with UEFI enabled
Write-Host "Configuring VM (UEFI mode)..." -ForegroundColor Yellow
& $VBoxManage modifyvm $VMName --memory 512 --vram 128 --cpus 2
& $VBoxManage modifyvm $VMName --firmware efi64
& $VBoxManage modifyvm $VMName --graphicscontroller vmsvga
& $VBoxManage modifyvm $VMName --boot1 dvd --boot2 disk --boot3 none --boot4 none

# Configure network with Intel e1000 (Host-Only mode for direct host access)
& $VBoxManage modifyvm $VMName --nic1 hostonly --nictype1 82540EM --cableconnected1 on
& $VBoxManage modifyvm $VMName --hostonlyadapter1 "VirtualBox Host-Only Ethernet Adapter"

# Add storage controllers
& $VBoxManage storagectl $VMName --name "SATA" --add sata --controller IntelAhci --portcount 2

# Attach ISO if it exists
if (Test-Path $ISOPath) {
    & $VBoxManage storageattach $VMName --storagectl "SATA" --port 0 --device 0 --type dvddrive --medium $ISOPath
    Write-Host "ISO attached: $ISOPath" -ForegroundColor Green
} else {
    # Use iso_root folder directly - requires manual setup
    Write-Host "ISO not created. Manual setup required:" -ForegroundColor Yellow
    Write-Host "  1. Install xorriso (via MSYS2: pacman -S xorriso)" -ForegroundColor Gray
    Write-Host "  2. Or use WSL: wsl xorriso ..." -ForegroundColor Gray
    Write-Host "  3. Or run build-limine.ps1" -ForegroundColor Gray
}

# Enable serial port for output
& $VBoxManage modifyvm $VMName --uart1 0x3F8 4 --uartmode1 file "$PSScriptRoot\serial.log"

Write-Host "`n=== VM Ready ===" -ForegroundColor Green
Write-Host "Name: $VMName" -ForegroundColor Cyan
Write-Host "Firmware: UEFI" -ForegroundColor Cyan
Write-Host "RAM: 512 MB" -ForegroundColor Cyan
Write-Host "Graphics: VMSVGA 128MB" -ForegroundColor Cyan
Write-Host "Serial: serial.log" -ForegroundColor Cyan

Write-Host "`nStarting VM..." -ForegroundColor Yellow
& $VBoxManage startvm $VMName

Write-Host "`nVM launched! Check VirtualBox window." -ForegroundColor Green
Write-Host "Serial output will be in: serial.log" -ForegroundColor Cyan
