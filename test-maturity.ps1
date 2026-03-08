param([int]$Port = 5557)

$OutputFile = "C:\Users\nathan\Documents\Scripts\OSrust\jarvis_maturity_results.txt"
$buf = New-Object byte[] 65536
$log = ""

function Log($msg) {
    $script:log += "$msg`n"
    Write-Host $msg
}

$c = New-Object System.Net.Sockets.TcpClient
$connected = $false
Write-Host "Connecting to serial port $Port..." -ForegroundColor Yellow
for ($i = 0; $i -lt 60; $i++) {
    try { $c.Connect("127.0.0.1", $Port); $connected = $true; break }
    catch { Start-Sleep -Milliseconds 500 }
}
if (-not $connected) { "FAILED: Could not connect" | Out-File $OutputFile; exit 1 }
$s = $c.GetStream(); $s.ReadTimeout = 5000
Log "CONNECTED to serial port $Port"

$text = ""; $sw = [System.Diagnostics.Stopwatch]::StartNew()
while ($sw.Elapsed.TotalSeconds -lt 90) {
    if ($s.DataAvailable) {
        $r = $s.Read($buf,0,$buf.Length)
        if($r -gt 0){ $text += [System.Text.Encoding]::ASCII.GetString($buf,0,$r) }
    } else { Start-Sleep -Milliseconds 200 }
    if ($text -match 'trustos:[^\r\n]*\$') { break }
}
Log "BOOT: $([math]::Round($sw.Elapsed.TotalSeconds,1))s"

function SendCmd($cmd, $timeout) {
    Start-Sleep -Milliseconds 300
    while ($s.DataAvailable) { $s.Read($buf,0,$buf.Length) | Out-Null }
    Start-Sleep -Milliseconds 100
    $cb = [System.Text.Encoding]::ASCII.GetBytes("$cmd`r")
    $s.Write($cb,0,$cb.Length); $s.Flush()
    Write-Host ">>> $cmd" -ForegroundColor Cyan
    $o = ""; $sw2 = [System.Diagnostics.Stopwatch]::StartNew()
    while ($sw2.Elapsed.TotalSeconds -lt $timeout) {
        if ($s.DataAvailable) {
            $r = $s.Read($buf,0,$buf.Length)
            if ($r -gt 0) { $o += [System.Text.Encoding]::ASCII.GetString($buf,0,$r) }
        } else { Start-Sleep -Milliseconds 100 }
        if ($sw2.ElapsedMilliseconds -ge 2000 -and $o.Length -gt 5 -and $o -match 'trustos:[^\r\n]*\$\s*$') {
            Start-Sleep -Milliseconds 500
            while ($s.DataAvailable) { $r = $s.Read($buf,0,$buf.Length); if($r -gt 0){$o += [System.Text.Encoding]::ASCII.GetString($buf,0,$r)} }
            break
        }
    }
    return $o
}

$timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
Log "=========================================="
Log "  JARVIS MATURITY TEST -- $timestamp"
Log "=========================================="

Log "===== STEP 1: INIT BRAIN ====="
$r = SendCmd "jarvis brain init" 30
Log $r

Log "===== STEP 2: EVAL BEFORE TRAINING ====="
$r = SendCmd "jarvis brain eval" 180
Log $r

Log "===== STEP 3: PRETRAIN 1 EPOCH ====="
$sw3 = [System.Diagnostics.Stopwatch]::StartNew()
$r = SendCmd "jarvis brain pretrain 1" 600
$elapsed = $sw3.Elapsed.TotalSeconds
Log $r
Log "Pretrain wall-clock: $([math]::Round($elapsed,1))s"

Log "===== STEP 4: EVAL AFTER TRAINING ====="
$r = SendCmd "jarvis brain eval" 180
Log $r

Log "===== STEP 5: SAVE WEIGHTS ====="
$r = SendCmd "jarvis brain save" 30
Log $r

foreach ($prompt in @("Hello", "who are you")) {
    Log "===== CHAT: $prompt ====="
    $r = SendCmd "jarvis brain chat $prompt" 20
    Log $r
}

Log "=========================================="
Log "  TEST COMPLETE"
Log "=========================================="

$lossPattern = 'loss\s*[=:]\s*(\d+\.\d+)'
$allMatches = [regex]::Matches($log, $lossPattern)
if ($allMatches.Count -gt 0) {
    $lastLoss = $allMatches[$allMatches.Count - 1].Groups[1].Value
    Log "FINAL LOSS: $lastLoss"
    $lossVal = [double]$lastLoss
    if ($lossVal -lt 2.0) {
        Log "MATURITY: ADULT (Level 3) - READY FOR FULL PROPAGATION"
    } elseif ($lossVal -lt 3.5) {
        Log "MATURITY: TEEN (Level 2) - READY FOR MESH + FEDERATED"
    } elseif ($lossVal -lt 5.0) {
        Log "MATURITY: CHILD (Level 1) - NEEDS MORE TRAINING"
    } else {
        Log "MATURITY: INFANT (Level 0) - UNTRAINED"
    }
}

$log | Out-File $OutputFile -Encoding UTF8
Write-Host "Results saved to: $OutputFile" -ForegroundColor Green
$c.Close()
