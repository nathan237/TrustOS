# Auto-monitor VM output

$VBoxManage = "C:\Program Files\Oracle\VirtualBox\VBoxManage.exe"
$ScreenshotPath = "vm_screen.png"

Write-Host "=== Monitoring TRustOs VM ===" -ForegroundColor Cyan

for ($i = 0; $i -lt 10; $i++) {
    Start-Sleep -Seconds 2
    
    # Capture screen
    & $VBoxManage controlvm TRustOs screenshotpng $ScreenshotPath 2>$null
    
    if (Test-Path $ScreenshotPath) {
        $size = (Get-Item $ScreenshotPath).Length
        Write-Host "Screenshot $($i+1): $size bytes - $(Get-Date -Format 'HH:mm:ss')" -ForegroundColor Green
    }
    
    # Check serial log
    if (Test-Path serial.log) {
        $content = Get-Content serial.log -Raw
        if ($content) {
            Write-Host "`n=== SERIAL OUTPUT DETECTED ===" -ForegroundColor Green
            Write-Host $content -ForegroundColor Yellow
            break
        }
    }
    
    # Check VM state
    $state = & $VBoxManage showvminfo TRustOs --machinereadable | Select-String "^VMState="
    if ($state -match 'VMState="(?!running)') {
        Write-Host "`nVM stopped or crashed" -ForegroundColor Red
        break
    }
}

Write-Host "`nMonitoring terminé. Ouvre vm_screen.png pour voir l'écran" -ForegroundColor Cyan
