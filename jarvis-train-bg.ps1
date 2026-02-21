<#
.SYNOPSIS
    Jarvis Training Demo — runs all training commands and saves output to file
#>
param([int]$Port = 5557)

$OutputFile = "C:\Users\nathan\Documents\Scripts\OSrust\jarvis_training_results.txt"
$buf = New-Object byte[] 65536
$log = ""

function Log($msg) { $script:log += "$msg`n" }

# Connect
$c = New-Object System.Net.Sockets.TcpClient
$connected = $false
for ($i = 0; $i -lt 30; $i++) {
    try { $c.Connect("127.0.0.1", $Port); $connected = $true; break }
    catch { Start-Sleep -Milliseconds 500 }
}
if (-not $connected) { "FAILED" | Out-File $OutputFile; exit 1 }
$s = $c.GetStream(); $s.ReadTimeout = 5000
Log "CONNECTED to port $Port"

# Wait for shell prompt
$text = ""; $sw = [System.Diagnostics.Stopwatch]::StartNew()
while ($sw.Elapsed.TotalSeconds -lt 60) {
    if ($s.DataAvailable) { $r = $s.Read($buf,0,$buf.Length); if($r -gt 0){ $text += [System.Text.Encoding]::ASCII.GetString($buf,0,$r) } }
    else { Start-Sleep -Milliseconds 200 }
    if ($text -match "trustos:[^\r\n]*\$") { break }
}
Log "BOOT: $([math]::Round($sw.Elapsed.TotalSeconds,1))s"

function SendCmd($cmd, $timeout) {
    # Drain
    Start-Sleep -Milliseconds 200
    while ($s.DataAvailable) { $s.Read($buf,0,$buf.Length) | Out-Null }
    Start-Sleep -Milliseconds 100
    
    # Send
    $cb = [System.Text.Encoding]::ASCII.GetBytes("$cmd`r")
    $s.Write($cb,0,$cb.Length); $s.Flush()
    
    # Collect output with prompt detection
    $o = ""; $sw2 = [System.Diagnostics.Stopwatch]::StartNew()
    while ($sw2.Elapsed.TotalSeconds -lt $timeout) {
        if ($s.DataAvailable) {
            $r = $s.Read($buf,0,$buf.Length)
            if ($r -gt 0) { $o += [System.Text.Encoding]::ASCII.GetString($buf,0,$r) }
        } else { Start-Sleep -Milliseconds 100 }
        # Prompt detection after 2s
        if ($sw2.ElapsedMilliseconds -ge 2000 -and $o.Length -gt 5 -and $o -match "trustos:[^\r\n]*\$\s*$") {
            Start-Sleep -Milliseconds 300
            while ($s.DataAvailable) { $r = $s.Read($buf,0,$buf.Length); if($r -gt 0){$o += [System.Text.Encoding]::ASCII.GetString($buf,0,$r)} }
            break
        }
    }
    return $o
}

# 1. INIT
Log "`n===== JARVIS BRAIN INIT ====="
$r = SendCmd "jarvis brain init" 20
Log $r

# 2. EVAL BEFORE
Log "`n===== EVAL BEFORE TRAINING ====="
$r = SendCmd "jarvis brain eval" 60
Log $r

# 3. PRETRAIN 1 epoch (may take a while in TCG)
Log "`n===== PRETRAIN 1 EPOCH (backprop+Adam) ====="
$sw3 = [System.Diagnostics.Stopwatch]::StartNew()
$r = SendCmd "jarvis brain pretrain 1" 600
$elapsed = $sw3.Elapsed.TotalSeconds
Log $r
Log "Wall-clock: $([math]::Round($elapsed,1))s"

# 4. EVAL AFTER
Log "`n===== EVAL AFTER TRAINING ====="
$r = SendCmd "jarvis brain eval" 60
Log $r

# 5. CHAT TESTS
foreach ($prompt in @("Hello", "who are you", "help", "TrustOS")) {
    Log "`n===== CHAT: $prompt ====="
    $r = SendCmd "jarvis brain chat $prompt" 20
    Log $r
}

# 6. SAVE
Log "`n===== SAVE WEIGHTS ====="
$r = SendCmd "jarvis brain save" 15
Log $r

Log "`n===== DONE ====="
$log | Out-File $OutputFile -Encoding UTF8
$c.Close()
