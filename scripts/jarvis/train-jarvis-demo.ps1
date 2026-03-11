<#
.SYNOPSIS
    Train Jarvis live — demonstrates backprop+Adam speed
#>
$port = 5557
$buf = New-Object byte[] 65536

# Connect
$c = New-Object System.Net.Sockets.TcpClient
$connected = $false
for ($i = 0; $i -lt 60; $i++) {
    try { $c.Connect("127.0.0.1", $port); $connected = $true; break }
    catch { Start-Sleep -Milliseconds 500 }
}
if (-not $connected) { Write-Host "FAILED to connect"; exit 1 }
$s = $c.GetStream()
$s.ReadTimeout = 5000
$w = New-Object System.IO.StreamWriter($s)
$w.AutoFlush = $true
Write-Host "Connected to serial TCP $port" -ForegroundColor Green

# Wait for boot
$boot = ""
$sw = [System.Diagnostics.Stopwatch]::StartNew()
while ($sw.Elapsed.TotalSeconds -lt 40) {
    if ($s.DataAvailable) {
        $r = $s.Read($buf, 0, $buf.Length)
        if ($r -gt 0) { $boot += [System.Text.Encoding]::ASCII.GetString($buf, 0, $r) }
        if ($boot -match "trustos:/\$") { break }
    } else { Start-Sleep -Milliseconds 200 }
}
Write-Host "Booted in $([math]::Round($sw.Elapsed.TotalSeconds,1))s" -ForegroundColor Green

function Send-Cmd {
    param([string]$cmd, [int]$timeout = 30)
    while ($s.DataAvailable) { $s.Read($buf, 0, $buf.Length) | Out-Null }
    Start-Sleep -Milliseconds 300
    $cmdBytes = [System.Text.Encoding]::ASCII.GetBytes("$cmd`r")
    $s.Write($cmdBytes, 0, $cmdBytes.Length); $s.Flush()
    $output = ""
    $sw2 = [System.Diagnostics.Stopwatch]::StartNew()
    while ($sw2.Elapsed.TotalSeconds -lt $timeout) {
        if ($s.DataAvailable) {
            $r = $s.Read($buf, 0, $buf.Length)
            if ($r -gt 0) { $output += [System.Text.Encoding]::ASCII.GetString($buf, 0, $r) }
        } else { Start-Sleep -Milliseconds 100 }
        if ($sw2.ElapsedMilliseconds -ge 1000 -and $output.Length -gt 5) {
            if ($output -match "trustos:[^\r\n]*\$\s*$") {
                Start-Sleep -Milliseconds 300
                while ($s.DataAvailable) {
                    $r = $s.Read($buf, 0, $buf.Length)
                    if ($r -gt 0) { $output += [System.Text.Encoding]::ASCII.GetString($buf, 0, $r) }
                }
                break
            }
        }
    }
    return $output
}

# ---- TRAINING DEMO ----
$results = @()

Write-Host "`n=== JARVIS TRAINING DEMO (Backprop + Adam) ===" -ForegroundColor Cyan

# 1. Init
Write-Host "`n[1] Initializing brain..." -ForegroundColor Yellow
$r = Send-Cmd "jarvis brain init" 15
$results += "=== INIT ===`n$r"
# Extract key info
foreach ($line in $r.Split("`n")) {
    $l = $line.Trim()
    if ($l -match "Model:|Parameters:|Optimizer:|ready") { Write-Host "  $l" -ForegroundColor White }
}

# 2. Eval before training
Write-Host "`n[2] Evaluating BEFORE training..." -ForegroundColor Yellow
$r = Send-Cmd "jarvis brain eval" 30
$results += "`n=== EVAL BEFORE ===`n$r"
foreach ($line in $r.Split("`n")) {
    $l = $line.Trim()
    if ($l -match "loss|Phase|baseline") { Write-Host "  $l" -ForegroundColor White }
}

# 3. Pretrain 1 epoch
Write-Host "`n[3] Pre-training (1 epoch, backprop+Adam)..." -ForegroundColor Yellow
$sw3 = [System.Diagnostics.Stopwatch]::StartNew()
$r = Send-Cmd "jarvis brain pretrain 1" 120
$elapsed = $sw3.Elapsed.TotalSeconds
$results += "`n=== PRETRAIN 1 EPOCH ===`n$r"
foreach ($line in $r.Split("`n")) {
    $l = $line.Trim()
    if ($l -match "Phase|loss|step|done|epoch") { Write-Host "  $l" -ForegroundColor White }
}
Write-Host "  Wall time: $([math]::Round($elapsed,1))s" -ForegroundColor Green

# 4. Eval after training
Write-Host "`n[4] Evaluating AFTER training..." -ForegroundColor Yellow
$r = Send-Cmd "jarvis brain eval" 30
$results += "`n=== EVAL AFTER ===`n$r"
foreach ($line in $r.Split("`n")) {
    $l = $line.Trim()
    if ($l -match "loss|Phase|baseline") { Write-Host "  $l" -ForegroundColor White }
}

# 5. Chat
Write-Host "`n[5] Chatting with Jarvis..." -ForegroundColor Yellow
$prompts = @("Hello", "who are you", "help me", "TrustOS")
foreach ($prompt in $prompts) {
    $r = Send-Cmd "jarvis brain chat $prompt" 15
    $results += "`n=== CHAT: $prompt ===`n$r"
    # Find generated text
    foreach ($line in $r.Split("`n")) {
        $l = $line.Trim()
        if ($l -match "Generated|Output|Brain says") { Write-Host "  [$prompt] -> $l" -ForegroundColor Magenta }
    }
}

# 6. Save
Write-Host "`n[6] Saving weights..." -ForegroundColor Yellow
$r = Send-Cmd "jarvis brain save" 10
$results += "`n=== SAVE ===`n$r"
foreach ($line in $r.Split("`n")) {
    $l = $line.Trim()
    if ($l -match "Saved|KB|weights") { Write-Host "  $l" -ForegroundColor White }
}

Write-Host "`n=== TRAINING DEMO COMPLETE ===" -ForegroundColor Green

# Save full log
$results -join "`n" | Out-File -FilePath "C:\Users\nathan\Documents\Scripts\OSrust\jarvis_training_log.txt" -Encoding UTF8
Write-Host "Full log: jarvis_training_log.txt"

$c.Close()
