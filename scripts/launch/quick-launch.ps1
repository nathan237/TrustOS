$VBM = "C:\Program Files\Oracle\VirtualBox\VBoxManage.exe"
$VMName = "TRustOs"
$ISOPath = "C:\Users\nathan\Documents\Scripts\OSrust\trustos.iso"
$SerialLog = "C:\Users\nathan\Documents\Scripts\OSrust\serial.log"

# Cleanup
try { & $VBM controlvm $VMName poweroff 2>$null } catch {}
Start-Sleep -Seconds 2
try { & $VBM unregistervm $VMName --delete 2>$null } catch {}
Start-Sleep -Seconds 1

# Remove serial log
Remove-Item $SerialLog -ErrorAction SilentlyContinue

# Create VM
& $VBM createvm --name $VMName --ostype "Other_64" --register
if ($LASTEXITCODE -ne 0) { Write-Output "FAILED: createvm"; exit 1 }

# Configure
& $VBM modifyvm $VMName --memory 1024 --vram 128 --cpus 4
& $VBM modifyvm $VMName --firmware efi64
& $VBM modifyvm $VMName --graphicscontroller vboxsvga
& $VBM modifyvm $VMName --boot1 dvd --boot2 disk --boot3 none --boot4 none
& $VBM modifyvm $VMName --audio-driver default --audio-controller hda --audio-enabled on --audio-out on
& $VBM modifyvm $VMName --nic1 nat --nictype1 82540EM --cableconnected1 on
& $VBM storagectl $VMName --name "SATA" --add sata --controller IntelAhci --portcount 4
& $VBM storageattach $VMName --storagectl "SATA" --port 0 --device 0 --type dvddrive --medium $ISOPath
& $VBM modifyvm $VMName --uart1 0x3F8 4 --uartmode1 file $SerialLog
& $VBM setextradata $VMName "GUI/AutoresizeGuest" "false"

Write-Output "=== VM Created, starting... ==="
& $VBM startvm $VMName
Write-Output "=== VM Started ==="

# Wait for boot
Start-Sleep -Seconds 15
if (Test-Path $SerialLog) {
    Write-Output "=== Serial Log (last 10 lines) ==="
    Get-Content $SerialLog -Tail 10
} else {
    Write-Output "NO SERIAL LOG YET"
}
