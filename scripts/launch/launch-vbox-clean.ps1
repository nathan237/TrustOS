$ErrorActionPreference = "Continue"
$VBM = "C:\Program Files\Oracle\VirtualBox\VBoxManage.exe"
$VMName = "TRustOs"
$ISOPath = "C:\Users\nathan\Documents\Scripts\OSrust\trustos.iso"
$SerialLog = "C:\Users\nathan\Documents\Scripts\OSrust\serial.log"
$LogFile = "C:\Users\nathan\Documents\Scripts\OSrust\vbox_launch_log.txt"

# Log everything
Start-Transcript -Path $LogFile -Force

Write-Output "=== Starting VBox Launch $(Get-Date) ==="

# Step 1: Kill any running TRustOs
Write-Output "Step 1: Cleanup..."
& $VBM controlvm $VMName poweroff 2>&1
Start-Sleep -Seconds 3
# Detach install test disk before unregister to prevent deletion
& $VBM storageattach $VMName --storagectl "SATA" --port 1 --device 0 --type hdd --medium none 2>&1
& $VBM unregistervm $VMName --delete 2>&1
Start-Sleep -Seconds 2

# Also clean up inaccessible VMs that might block
Write-Output "Cleaning inaccessible VMs..."
$vms = & $VBM list vms 2>&1
foreach ($line in $vms) {
    if ($line -match '"<inaccessible>" \{([^}]+)\}') {
        $uuid = $matches[1]
        Write-Output "Removing inaccessible VM: $uuid"
        & $VBM unregistervm $uuid --delete 2>&1
    }
}
Start-Sleep -Seconds 1

# Step 2: Verify ISO
Write-Output "Step 2: Check ISO..."
if (-not (Test-Path $ISOPath)) {
    Write-Output "ERROR: ISO not found at $ISOPath"
    Stop-Transcript
    exit 1
}
Write-Output "ISO found: $((Get-Item $ISOPath).Length / 1MB) MB"

# Step 3: Create VM
Write-Output "Step 3: Create VM..."
$result = & $VBM createvm --name $VMName --ostype "Other_64" --register 2>&1
Write-Output "createvm result: $result"
if ($LASTEXITCODE -ne 0) {
    Write-Output "FAILED createvm (exit code $LASTEXITCODE)"
    Stop-Transcript
    exit 1
}

# Step 4: Configure VM
Write-Output "Step 4: Configure VM..."
& $VBM modifyvm $VMName --memory 2048 --vram 128 --cpus 4 2>&1
& $VBM modifyvm $VMName --firmware efi64 2>&1
& $VBM modifyvm $VMName --graphicscontroller vboxsvga 2>&1
& $VBM modifyvm $VMName --boot1 dvd --boot2 disk --boot3 none --boot4 none 2>&1
& $VBM modifyvm $VMName --audio-driver default --audio-controller hda --audio-enabled on --audio-out on 2>&1
& $VBM modifyvm $VMName --nic1 nat --nictype1 82540EM --cableconnected1 on 2>&1

# Storage
& $VBM storagectl $VMName --name "SATA" --add sata --controller IntelAhci --portcount 4 2>&1
& $VBM storageattach $VMName --storagectl "SATA" --port 0 --device 0 --type dvddrive --medium $ISOPath 2>&1

# Audio data disk on AHCI port 3 (raw sectors with WAV tracks)
# Port 2 is used by TrustFS for the root filesystem
$DataDisk = "C:\Users\nathan\Documents\Scripts\OSrust\builds\trustos_data.img"
if (Test-Path $DataDisk) {
    # Convert raw .img to VDI for VirtualBox
    $DataVDI = "C:\Users\nathan\Documents\Scripts\OSrust\builds\trustos_data.vdi"
    Remove-Item $DataVDI -ErrorAction SilentlyContinue
    & $VBM convertfromraw $DataDisk $DataVDI --format VDI 2>&1
    & $VBM storageattach $VMName --storagectl "SATA" --port 3 --device 0 --type hdd --medium $DataVDI 2>&1
    Write-Output "Audio data disk attached on SATA port 3"
} else {
    Write-Output "WARNING: No audio data disk found at $DataDisk"
}

# Install test disk on AHCI port 1 (1GB, for installer testing)
$InstallTestDisk = "C:\Users\nathan\Documents\Scripts\OSrust\builds\trustos_install_test.vdi"
if (-not (Test-Path $InstallTestDisk)) {
    & $VBM createmedium disk --filename $InstallTestDisk --size 1024 --format VDI 2>&1
    Write-Output "Created 1GB install test disk"
}
if (Test-Path $InstallTestDisk) {
    & $VBM storageattach $VMName --storagectl "SATA" --port 1 --device 0 --type hdd --medium $InstallTestDisk 2>&1
    Write-Output "Install test disk attached on SATA port 1"
}

# Serial
Remove-Item $SerialLog -ErrorAction SilentlyContinue
& $VBM modifyvm $VMName --uart1 0x3F8 4 --uartmode1 file $SerialLog 2>&1
& $VBM setextradata $VMName "GUI/AutoresizeGuest" "false" 2>&1

Write-Output "VM configured successfully"

# Step 5: Start VM
Write-Output "Step 5: Starting VM..."
$startResult = & $VBM startvm $VMName 2>&1
Write-Output "startvm result: $startResult"
Write-Output "startvm exit code: $LASTEXITCODE"

# Step 6: Wait for boot
Write-Output "Step 6: Waiting 20s for boot..."
Start-Sleep -Seconds 20

# Check serial log
if (Test-Path $SerialLog) {
    Write-Output "=== Serial Log (last 15 lines) ==="
    Get-Content $SerialLog -Tail 15
} else {
    Write-Output "NO SERIAL LOG FOUND"
}

# Check if VM is running
$running = & $VBM list runningvms 2>&1
Write-Output "=== Running VMs ==="
Write-Output $running

Write-Output "=== Done ==="
Stop-Transcript
