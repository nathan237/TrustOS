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
# Convert Windows path to WSL path dynamically
$winRoot = (Get-Location).Path
$driveLetter = $winRoot.Substring(0,1).ToLower()
$wslRoot = "/mnt/$driveLetter" + ($winRoot.Substring(2) -replace '\\', '/')
$wslIsoRoot = "$wslRoot/iso_root"
$wslIsoPath = "$wslRoot/trustos.iso"
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
& $VBoxManage modifyvm $VMName --memory 1024 --vram 128 --cpus 4
& $VBoxManage modifyvm $VMName --firmware efi64
& $VBoxManage modifyvm $VMName --graphicscontroller vboxsvga
& $VBoxManage modifyvm $VMName --boot1 dvd --boot2 disk --boot3 none --boot4 none

# Configure Intel HDA audio (required for TrustOS HDA driver)
& $VBoxManage modifyvm $VMName --audio-driver default --audio-controller hda --audio-enabled on --audio-out on

# Configure network with Intel e1000 (NAT mode for DHCP + internet)
& $VBoxManage modifyvm $VMName --nic1 nat --nictype1 82540EM --cableconnected1 on

# Add storage controllers
& $VBoxManage storagectl $VMName --name "SATA" --add sata --controller IntelAhci --portcount 4

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

# Create and attach a persistent data disk (AHCI port 1)
$DataVdi = "$PSScriptRoot\trustos_data.vdi"
if (-not (Test-Path $DataVdi)) {
    Write-Host "Creating 64 MB data disk: $DataVdi" -ForegroundColor Yellow
    & $VBoxManage createmedium disk --filename $DataVdi --size 64 --format VDI
    Write-Host "Data disk created" -ForegroundColor Green
}
& $VBoxManage storageattach $VMName --storagectl "SATA" --port 1 --device 0 --type hdd --medium $DataVdi
Write-Host "Data disk attached on SATA port 1" -ForegroundColor Green

# Enable serial port for output
& $VBoxManage modifyvm $VMName --uart1 0x3F8 4 --uartmode1 file "$PSScriptRoot\serial.log"

# Enable GUI scaling so the VM display scales when the window is resized
# Without Guest Additions, the guest can't change resolution dynamically,
# but VBox will scale the framebuffer output to fit the window.
& $VBoxManage setextradata $VMName "GUI/ScaleFactor" "1.0"
& $VBoxManage setextradata $VMName "GUI/AutoresizeGuest" "false"
& $VBoxManage setextradata $VMName "CustomVideoMode1" "1920x1080x32"

Write-Host "`n=== VM Ready ===" -ForegroundColor Green
Write-Host "Name: $VMName" -ForegroundColor Cyan
Write-Host "Firmware: UEFI" -ForegroundColor Cyan
Write-Host "RAM: 1024 MB" -ForegroundColor Cyan
Write-Host "Graphics: VMSVGA 128MB" -ForegroundColor Cyan
Write-Host "Audio: Intel HDA (output enabled)" -ForegroundColor Cyan
Write-Host "Serial: serial.log" -ForegroundColor Cyan

Write-Host "`nStarting VM..." -ForegroundColor Yellow
& $VBoxManage startvm $VMName

Write-Host "`nVM launched! Check VirtualBox window." -ForegroundColor Green
Write-Host "Serial output will be in: serial.log" -ForegroundColor Cyan
