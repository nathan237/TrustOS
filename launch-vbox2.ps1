$VBM = "C:\Program Files\Oracle\VirtualBox\VBoxManage.exe"
$VMName = "TRustOs"
$Base = "C:\Users\nathan\Documents\Scripts\OSrust"
$ISOPath = "$Base\trustos.iso"
$SerialLog = "$Base\serial.log"

Remove-Item $SerialLog -ErrorAction SilentlyContinue

# Unregister old VM (ignore errors)
$null = & $VBM unregistervm $VMName --delete 2>&1

# Create VM
Write-Host "Creating VM..."
$null = & $VBM createvm --name $VMName --ostype "Other_64" --register 2>&1

# Configure
Write-Host "Configuring..."
$null = & $VBM modifyvm $VMName --memory 1024 --vram 128 --cpus 4 2>&1
$null = & $VBM modifyvm $VMName --firmware efi64 2>&1
$null = & $VBM modifyvm $VMName --graphicscontroller vboxsvga 2>&1
$null = & $VBM modifyvm $VMName --boot1 dvd --boot2 disk --boot3 none --boot4 none 2>&1
$null = & $VBM modifyvm $VMName --audio-driver default --audio-controller hda --audio-enabled on --audio-out on 2>&1
$null = & $VBM modifyvm $VMName --nic1 nat --nictype1 82540EM --cableconnected1 on 2>&1
$null = & $VBM storagectl $VMName --name "SATA" --add sata --controller IntelAhci --portcount 4 2>&1
$null = & $VBM storageattach $VMName --storagectl "SATA" --port 0 --device 0 --type dvddrive --medium $ISOPath 2>&1
$null = & $VBM modifyvm $VMName --uart1 0x3F8 4 --uartmode1 file $SerialLog 2>&1
$null = & $VBM setextradata $VMName "GUI/AutoresizeGuest" "false" 2>&1

# Verify VM exists
$check = & $VBM showvminfo $VMName --machinereadable 2>&1 | Select-String "name="
Write-Host "VM check: $check"

# Start VM
Write-Host "Starting VM..."
& $VBM startvm $VMName 2>&1
Write-Host "Exit code: $LASTEXITCODE"
