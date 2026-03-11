# TRustOs - Build & Debug Automation
$ErrorActionPreference = "Continue"
$VBoxManage = "C:\Program Files\Oracle\VirtualBox\VBoxManage.exe"

Write-Host "`n=== BUILD & DEBUG ===" -ForegroundColor Cyan

# Build
Write-Host "[1/4] Build..." -ForegroundColor Yellow
cargo build --release 2>&1 | Out-Null
if ($LASTEXITCODE -ne 0) { Write-Host "Build failed" -ForegroundColor Red; exit 1 }
Write-Host "OK" -ForegroundColor Green

# Bootimage
Write-Host "[2/4] Bootimage..." -ForegroundColor Yellow
cd kernel; cargo bootimage --release 2>&1 | Out-Null; cd ..
$imgSize = [math]::Round((Get-Item "target\x86_64-unknown-none\release\bootimage-trustos_kernel.bin").Length/1KB, 1)
Write-Host "OK: $imgSize KB" -ForegroundColor Green

# Cleanup
Write-Host "[3/4] VM setup..." -ForegroundColor Yellow
& $VBoxManage controlvm TRustOs poweroff 2>$null
Start-Sleep -Seconds 2
& $VBoxManage unregistervm TRustOs --delete 2>$null
Remove-Item "target\x86_64-unknown-none\release\trustos.vdi" -ErrorAction SilentlyContinue
Remove-Item "target\x86_64-unknown-none\release\bootimage-padded.bin" -ErrorAction SilentlyContinue

# Pad & convert
$srcImg = "target\x86_64-unknown-none\release\bootimage-trustos_kernel.bin"
$padImg = "target\x86_64-unknown-none\release\bootimage-padded.bin"
Copy-Item $srcImg $padImg -Force
$file = [System.IO.File]::Open($padImg, [System.IO.FileMode]::Append)
$file.SetLength(1048576)
$file.Close()

$vdiPath = "target\x86_64-unknown-none\release\trustos.vdi"
& $VBoxManage convertfromraw $padImg $vdiPath --format VDI 2>&1 | Out-Null

# Create VM
& $VBoxManage createvm --name TRustOs --ostype "Other_64" --register 2>&1 | Out-Null
& $VBoxManage modifyvm TRustOs --memory 512 --vram 16 --cpus 1 2>&1 | Out-Null
& $VBoxManage storagectl TRustOs --name "SATA" --add sata --controller IntelAhci 2>&1 | Out-Null
& $VBoxManage storageattach TRustOs --storagectl "SATA" --port 0 --device 0 --type hdd --medium $vdiPath 2>&1 | Out-Null
& $VBoxManage modifyvm TRustOs --uart1 0x3F8 4 --uartmode1 file serial.log 2>&1 | Out-Null
Write-Host "OK" -ForegroundColor Green

# Start & monitor
Write-Host "[4/4] Starting..." -ForegroundColor Yellow
& $VBoxManage startvm TRustOs 2>&1 | Out-Null
Start-Sleep -Seconds 2
Write-Host "OK" -ForegroundColor Green

Write-Host "`n=== MONITORING ===" -ForegroundColor Cyan
$panicDetected = $false
$successDetected = $false

for ($i = 0; $i -lt 15; $i++) {
    Start-Sleep -Seconds 1
    
    if (Test-Path serial.log) {
        $serial = Get-Content serial.log -Raw -ErrorAction SilentlyContinue
        if ($serial) {
            Write-Host "`nSerial:" -ForegroundColor Green
            Write-Host $serial -ForegroundColor White
            if ($serial -match "Kernel ready") { $successDetected = $true }
            if ($serial -match "panic") { $panicDetected = $true }
        }
    }
    
    $screenPath = "debug_screen_$i.png"
    & $VBoxManage controlvm TRustOs screenshotpng $screenPath 2>$null
    
    Write-Host "." -NoNewline -ForegroundColor Gray
}

Write-Host "`n`n=== STATUS ===" -ForegroundColor Cyan
$lastScreen = Get-ChildItem "debug_screen_*.png" | Sort-Object Name | Select-Object -Last 1
if ($lastScreen) { Invoke-Item $lastScreen.FullName }

if ($successDetected) {
    Write-Host "SUCCESS!" -ForegroundColor Green
} elseif ($panicDetected) {
    Write-Host "PANIC!" -ForegroundColor Red
} else {
    Write-Host "Check screenshot" -ForegroundColor Yellow
}
