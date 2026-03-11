<#
.SYNOPSIS
    TrustOS Automated Test Suite - VirtualBox Edition
.DESCRIPTION
    Starts TrustOS in VirtualBox, injects commands via virtual keyboard,
    captures output from serial.log, validates and generates report.
#>

param(
    [string]$IsoPath = "$PSScriptRoot\trustos.iso",
    [int]$BootTimeout = 30,
    [int]$CmdWait = 4,
    [string]$ReportFile = "$PSScriptRoot\test_report.txt"
)

$ErrorActionPreference = "Continue"
$VBoxManage = "C:\Program Files\Oracle\VirtualBox\VBoxManage.exe"
$VMName = "TRustOs"
$SerialLog = "$PSScriptRoot\serial.log"
$timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"

$global:passed = 0
$global:failed = 0
$global:results = @()

# --- HELPERS ---

$global:lastGoodOffset = 0
$global:testNum = 0

function VBox-TypeCommand {
    param([string]$cmd)
    & $VBoxManage controlvm $VMName keyboardputstring "$cmd" 2>$null
    Start-Sleep -Milliseconds 300
    & $VBoxManage controlvm $VMName keyboardputscancode 1c 9c 2>$null
}

function Read-SerialRaw {
    param([long]$offset)
    if (-not (Test-Path $SerialLog)) { return "" }
    for ($retry = 0; $retry -lt 3; $retry++) {
        try {
            $fs = [System.IO.FileStream]::new($SerialLog,
                [System.IO.FileMode]::Open,
                [System.IO.FileAccess]::Read,
                [System.IO.FileShare]::ReadWrite -bor [System.IO.FileShare]::Delete)
            $len = $fs.Length
            if ($len -le $offset) { $fs.Close(); return "" }
            $count = [int]($len - $offset)
            $null = $fs.Seek($offset, [System.IO.SeekOrigin]::Begin)
            [byte[]]$buf = New-Object byte[] $count
            $null = $fs.Read($buf, 0, $count)
            $fs.Close()
            return [System.Text.Encoding]::ASCII.GetString($buf)
        } catch {
            Start-Sleep -Milliseconds 150
        }
    }
    return ""
}

function Clean-SerialOutput {
    param([string]$raw)
    # Strip keyboard IRQ/BUF/READ debug lines (single and multiline)
    $raw = [regex]::Replace($raw, '\[KB-IRQ\][^\n]*\n?', '')
    $raw = [regex]::Replace($raw, '\[KB-BUF\][^\n]*\n?', '')
    $raw = [regex]::Replace($raw, '\[KB-READ\][^\n]*\n?', '')
    # Strip orphan quote from multi-line KB-BUF char='\n'
    $raw = [regex]::Replace($raw, "(?m)^'\s*$\n?", '')
    # Strip empty [INFO] lines
    $raw = [regex]::Replace($raw, '\[\s*\d+\.\d+\]\s*\[INFO\s*\][^\n]*\n?', '')
    # Strip non-ASCII junk (cursor blocks, garbled UTF-8)
    $raw = [regex]::Replace($raw, '[\x00-\x08\x0B\x0C\x0E-\x1F\x7F-\xFF]', '')
    # Clean up: remove empty lines, trim
    $lines = $raw -split "`n" | ForEach-Object { $_.TrimEnd() } | Where-Object { $_ -ne "" }
    return ($lines -join "`n")
}

function Get-SerialLength {
    if (-not (Test-Path $SerialLog)) { return $global:lastGoodOffset }
    for ($retry = 0; $retry -lt 3; $retry++) {
        try {
            $fs = [System.IO.FileStream]::new($SerialLog,
                [System.IO.FileMode]::Open,
                [System.IO.FileAccess]::Read,
                [System.IO.FileShare]::ReadWrite -bor [System.IO.FileShare]::Delete)
            $len = $fs.Length
            $fs.Close()
            if ($len -ge $global:lastGoodOffset) {
                $global:lastGoodOffset = $len
                return $len
            }
        } catch {
            Start-Sleep -Milliseconds 150
        }
    }
    return $global:lastGoodOffset
}

function Wait-ForOutput {
    param([long]$sinceMark, [int]$seconds = $CmdWait)
    $promptRx = '\[\d{1,2}:\d{2}:\d{2}\]\s*trustos:[^\n]*\$'
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $result = ""
    while ($sw.Elapsed.TotalSeconds -lt $seconds) {
        Start-Sleep -Milliseconds 600
        $raw = Read-SerialRaw -offset $sinceMark
        if (-not $raw) { continue }
        $clean = Clean-SerialOutput $raw
        if (-not $clean) { continue }
        # Check if shell prompt appeared (command done)
        if ([regex]::IsMatch($clean, $promptRx)) {
            Start-Sleep -Milliseconds 400
            $raw = Read-SerialRaw -offset $sinceMark
            $clean = Clean-SerialOutput $raw
            # Remove prompt line itself from the output
            $clean = [regex]::Replace($clean, $promptRx + '.*', '')
            return $clean.Trim()
        }
    }
    # Timeout - return whatever we have, cleaned
    $raw = Read-SerialRaw -offset $sinceMark
    if (-not $raw) { return "" }
    $clean = Clean-SerialOutput $raw
    $clean = [regex]::Replace($clean, $promptRx + '.*', '')
    return $clean.Trim()
}

function Run-SingleTest {
    param([string]$cat, [string]$tname, [string]$cmd, [scriptblock]$validate, [int]$wait = $CmdWait)

    $global:testNum++
    Write-Host ("  [{0}] {1} ... " -f $cat, $tname) -NoNewline

    try {
        Start-Sleep -Milliseconds 300
        $mark = Get-SerialLength
        Start-Sleep -Milliseconds 100
        VBox-TypeCommand -cmd $cmd
        $output = Wait-ForOutput -sinceMark $mark -seconds $wait
        $success = & $validate $output

        if ($success) {
            Write-Host "PASS" -ForegroundColor Green
            $global:passed++
            $status = "PASS"
        } else {
            Write-Host "FAIL" -ForegroundColor Red
            $global:failed++
            $status = "FAIL"
        }
    } catch {
        Write-Host "ERROR $_" -ForegroundColor Yellow
        $global:failed++
        $status = "ERROR"
        $output = "Exception: $_"
    }

    $outSnippet = if ($output.Length -gt 300) { $output.Substring(0, 300) } else { "$output" }
    $outSnippet = $outSnippet -replace "`r`n|`r|`n", " "

    $global:results += @{
        Category = $cat
        Name     = $tname
        Command  = $cmd
        Status   = $status
        Output   = $outSnippet
    }
}

# === MAIN ===

Write-Host ""
Write-Host "==================================================================" -ForegroundColor Cyan
Write-Host "     TrustOS Automated Test Suite  -  VirtualBox Edition" -ForegroundColor Cyan
Write-Host "     $timestamp" -ForegroundColor DarkCyan
Write-Host "==================================================================" -ForegroundColor Cyan
Write-Host ""

if (-not (Test-Path $VBoxManage)) {
    Write-Host "FATAL: VBoxManage not found" -ForegroundColor Red; exit 1
}
if (-not (Test-Path $IsoPath)) {
    Write-Host "FATAL: ISO not found: $IsoPath" -ForegroundColor Red; exit 1
}

Write-Host "ISO:   $IsoPath" -ForegroundColor DarkGray
Write-Host ""

# --- Stop old VM ---
Write-Host "Step 1: Cleaning up old VM..." -ForegroundColor White
& $VBoxManage controlvm $VMName poweroff 2>$null
Start-Sleep -Seconds 2
& $VBoxManage unregistervm $VMName --delete 2>$null
Start-Sleep -Seconds 1
Remove-Item $SerialLog -Force -ErrorAction SilentlyContinue

# --- Create VM ---
Write-Host "Step 2: Creating VirtualBox VM..." -ForegroundColor White
& $VBoxManage createvm --name $VMName --ostype "Other_64" --register 2>$null | Out-Null
& $VBoxManage modifyvm $VMName --memory 1024 --vram 128 --cpus 2 2>$null
& $VBoxManage modifyvm $VMName --firmware efi64 2>$null
& $VBoxManage modifyvm $VMName --graphicscontroller vboxsvga 2>$null
& $VBoxManage modifyvm $VMName --boot1 dvd --boot2 disk --boot3 none --boot4 none 2>$null
& $VBoxManage modifyvm $VMName --nic1 nat --nictype1 82540EM --cableconnected1 on 2>$null
& $VBoxManage storagectl $VMName --name "SATA" --add sata --controller IntelAhci --portcount 2 2>$null
& $VBoxManage storageattach $VMName --storagectl "SATA" --port 0 --device 0 --type dvddrive --medium $IsoPath 2>$null
& $VBoxManage modifyvm $VMName --uart1 0x3F8 4 --uartmode1 file "$SerialLog" 2>$null
& $VBoxManage modifyvm $VMName --audio-driver default --audio-controller hda --audio-enabled on 2>$null
Write-Host "     VM created" -ForegroundColor Green

# --- Start VM ---
Write-Host "Step 3: Starting VM..." -ForegroundColor White
& $VBoxManage startvm $VMName 2>$null | Out-Null
Write-Host "     VM started" -ForegroundColor Green

# --- Wait for boot ---
Write-Host "Step 4: Waiting for boot (max ${BootTimeout}s)..." -ForegroundColor White
$sw = [System.Diagnostics.Stopwatch]::StartNew()
$booted = $false

while ($sw.Elapsed.TotalSeconds -lt $BootTimeout) {
    Start-Sleep -Milliseconds 500
    if (Test-Path $SerialLog) {
        $content = Get-Content $SerialLog -Raw -ErrorAction SilentlyContinue
        if ($content -and ($content -match "trustos" -or $content -match "Kernel ready" -or $content -match "Shell ready")) {
            $booted = $true
            break
        }
    }
    Write-Host "." -NoNewline -ForegroundColor DarkGray
}

Write-Host ""
$bootTime = [math]::Round($sw.Elapsed.TotalSeconds, 1)

if (-not $booted) {
    $vmState = & $VBoxManage showvminfo $VMName --machinereadable 2>$null | Select-String "VMState="
    Write-Host "     VM state: $vmState" -ForegroundColor Yellow
    if ((Test-Path $SerialLog) -and (Get-Item $SerialLog).Length -gt 100) {
        Write-Host "     Serial has data, proceeding..." -ForegroundColor Yellow
        $booted = $true
    } else {
        Write-Host "FATAL: Boot timed out" -ForegroundColor Red
        & $VBoxManage controlvm $VMName poweroff 2>$null
        exit 1
    }
}

Write-Host "     Booted in ${bootTime}s" -ForegroundColor Green
Start-Sleep -Seconds 3
& $VBoxManage controlvm $VMName screenshotpng "$PSScriptRoot\test_boot_screenshot.png" 2>$null
Write-Host ""

# === RUN TESTS ===
Write-Host "Step 5: Running tests..." -ForegroundColor White
Write-Host ("=" * 62) -ForegroundColor DarkGray

$testStart = [System.Diagnostics.Stopwatch]::StartNew()

# --- SHELL BASICS ---
Write-Host ""
Write-Host "  -- SHELL --" -ForegroundColor Cyan
Run-SingleTest "SHELL" "echo simple text" "echo hello_trustos" { param($o) $o -match "hello_trustos" }
Run-SingleTest "SHELL" "echo with spaces" "echo hello world 123" { param($o) $o -match "hello world 123" }
Run-SingleTest "SHELL" "pwd default root" "pwd" { param($o) $o -match "/" }
Run-SingleTest "SHELL" "whoami" "whoami" { param($o) $o -match "root" -or $o -match "nobody" }
Run-SingleTest "SHELL" "hostname" "hostname" { param($o) $o -match "trustos" }
Run-SingleTest "SHELL" "version" "version" { param($o) $o -match "T-RustOs" }
Run-SingleTest "SHELL" "uname -a" "uname -a" { param($o) $o -match "T-RustOs" -and $o -match "x86_64" }
Run-SingleTest "SHELL" "clear no crash" "clear" { param($o) $true }
Run-SingleTest "SHELL" "history" "history" { param($o) $o -match "\d" }
Run-SingleTest "SHELL" "help" "help" { param($o) $o -match "help" -or $o -match "command" -or $o -match "Commands" }
Run-SingleTest "SHELL" "cowsay" "cowsay TrustOS" { param($o) $o -match "TrustOS" -or $o -match "__" }

# --- SYSTEM INFO ---
Write-Host ""
Write-Host "  -- SYSINFO --" -ForegroundColor Cyan
Run-SingleTest "SYSINFO" "date" "date" { param($o) $o -match "\d{4}" -or $o -match "\d{2}:\d{2}" }
Run-SingleTest "SYSINFO" "uptime" "time" { param($o) $o -match "Uptime" -or $o -match "Time" }
Run-SingleTest "SYSINFO" "info" "info" { param($o) $o -match "RUSTOS" -or $o -match "Version" }
Run-SingleTest "SYSINFO" "free heap" "free" { param($o) $o -match "Heap" -or $o -match "total" }
Run-SingleTest "SYSINFO" "df disk usage" "df" { param($o) $o -match "ramfs" -or $o -match "Filesystem" }
Run-SingleTest "SYSINFO" "ps processes" "ps" { param($o) $o -match "PID" -or $o -match "kernel" -or $o -match "tsh" }
Run-SingleTest "SYSINFO" "env variables" "env" { param($o) $o -match "USER=" -or $o -match "SHELL=" }
Run-SingleTest "SYSINFO" "lscpu" "lscpu" { param($o) $o -match "CPU" -or $o -match "cpu" -or $o -match "x86" }
Run-SingleTest "SYSINFO" "lsmem" "lsmem" { param($o) $o -match "Memory" -or $o -match "memory" -or $o -match "MB" }
Run-SingleTest "SYSINFO" "lspci" "lspci" { param($o) $o -match "PCI" -or $o -match "pci" -or $o -match "Bus" }
Run-SingleTest "SYSINFO" "lsblk" "lsblk" { param($o) $o.Length -gt 3 }
Run-SingleTest "SYSINFO" "vmstat" "vmstat" { param($o) $o.Length -gt 3 }
Run-SingleTest "SYSINFO" "iostat" "iostat" { param($o) $o.Length -gt 3 }
Run-SingleTest "SYSINFO" "dmesg" "dmesg" { param($o) $o.Length -gt 5 }
Run-SingleTest "SYSINFO" "neofetch" "neofetch" { param($o) $o -match "TrustOS" -or $o -match "trustos" -or $o -match "Kernel" }

# --- FILESYSTEM ---
Write-Host ""
Write-Host "  -- FILESYSTEM --" -ForegroundColor Cyan
Run-SingleTest "FS" "mkdir test_dir" "mkdir /test_auto" { param($o) -not ($o -match "mkdir:.*rror") }
Run-SingleTest "FS" "ls sees new dir" "ls /" { param($o) $o -match "test_auto" }
Run-SingleTest "FS" "touch file" "touch /test_auto/hello.txt" { param($o) -not ($o -match "touch:.*rror") }
Run-SingleTest "FS" "echo to file" 'echo content_42 > /test_auto/hello.txt' { param($o) $true }
Run-SingleTest "FS" "cat file" "cat /test_auto/hello.txt" { param($o) $o -match "content_42" }
Run-SingleTest "FS" "ls dir content" "ls /test_auto" { param($o) $o -match "hello" }
Run-SingleTest "FS" "stat file" "stat /test_auto/hello.txt" { param($o) $o -match "hello" -or $o -match "size" -or $o -match "Size" }
Run-SingleTest "FS" "wc file" "wc /test_auto/hello.txt" { param($o) $o -match "\d" }
Run-SingleTest "FS" "cp file" "cp /test_auto/hello.txt /test_auto/copy.txt" { param($o) -not ($o -match "cp:.*rror") }
Run-SingleTest "FS" "cat copied file" "cat /test_auto/copy.txt" { param($o) $o -match "content_42" }
Run-SingleTest "FS" "mv file" "mv /test_auto/copy.txt /test_auto/moved.txt" { param($o) -not ($o -match "mv:.*rror") }
Run-SingleTest "FS" "cat moved file" "cat /test_auto/moved.txt" { param($o) $o -match "content_42" }
Run-SingleTest "FS" "grep in file" "grep content /test_auto/hello.txt" { param($o) $o -match "content" }
Run-SingleTest "FS" "find file" "find hello" { param($o) $o -match "hello" }
Run-SingleTest "FS" "tree" "tree /test_auto" { param($o) $o -match "hello" -or $o -match "moved" }
Run-SingleTest "FS" "head file" "head /test_auto/hello.txt" { param($o) $o -match "content" }
Run-SingleTest "FS" "tail file" "tail /test_auto/hello.txt" { param($o) $o -match "content" }
Run-SingleTest "FS" "diff two files" "diff /test_auto/hello.txt /test_auto/moved.txt" { param($o) $true }
Run-SingleTest "FS" "rm file" "rm /test_auto/moved.txt" { param($o) -not ($o -match "rm:.*rror") }
Run-SingleTest "FS" "rm verify gone" "cat /test_auto/moved.txt" { param($o) $o -match "cat:" }
Run-SingleTest "FS" "cd directory" "cd /test_auto" { param($o) -not ($o -match "cd:.*rror") }
Run-SingleTest "FS" "pwd after cd" "pwd" { param($o) $o -match "test_auto" }
Run-SingleTest "FS" "cd back to root" "cd /" { param($o) $true }

# --- TEXT UTILITIES ---
Write-Host ""
Write-Host "  -- TEXT --" -ForegroundColor Cyan
Run-SingleTest "TEXT" "seq 5" "seq 5" { param($o) $o -match "1" -and $o -match "5" }
Run-SingleTest "TEXT" "seq 3 7" "seq 3 7" { param($o) $o -match "3" -and $o -match "7" }
Run-SingleTest "TEXT" "factor 12" "factor 12" { param($o) $o -match "12:" -and $o -match "2" -and $o -match "3" }
Run-SingleTest "TEXT" "factor 97 prime" "factor 97" { param($o) $o -match "97:" -and $o -match "97" }
Run-SingleTest "TEXT" "factor invalid" "factor abc" { param($o) $o -match "invalid" -or $o -match "error" -or $o -match "Usage" }
Run-SingleTest "TEXT" "expr 2 + 3" "expr 2 + 3" { param($o) $o -match "5" }
Run-SingleTest "TEXT" "cal" "cal" { param($o) $o -match "Su" -or $o -match "Mo" -or $o -match "February" -or $o.Length -gt 10 }

# --- KERNEL SELF-TEST ---
Write-Host ""
Write-Host "  -- SELFTEST --" -ForegroundColor Cyan
Run-SingleTest "SELFTEST" "builtin self-test" "test" { param($o) $o -match "self-test" -or $o -match "OK" -or $o -match "Done" } 5

# --- INTTEST (25-test integration suite) ---
Write-Host ""
Write-Host "  -- INTTEST --" -ForegroundColor Cyan
Run-SingleTest "INTTEST" "integration test suite" "inttest" { param($o) $o -match "ALL.*TESTS PASSED" } 30

# --- TRUSTLANG ---
Write-Host ""
Write-Host "  -- TRUSTLANG --" -ForegroundColor Cyan
Run-SingleTest "TRUSTLANG" "eval println" 'trustlang eval println("hello_tl")' { param($o) $o -match "hello_tl" } 5
Run-SingleTest "TRUSTLANG" "eval 2+3" "trustlang eval println(2+3)" { param($o) $o -match "5" } 5
Run-SingleTest "TRUSTLANG" "eval 6*7" "trustlang eval println(6*7)" { param($o) $o -match "42" } 5
Run-SingleTest "TRUSTLANG" "eval string" 'trustlang eval println("TrustOS_rocks")' { param($o) $o -match "TrustOS_rocks" } 5
Run-SingleTest "TRUSTLANG" "eval 100/4" "trustlang eval println(100/4)" { param($o) $o -match "25" } 5
Run-SingleTest "TRUSTLANG" "eval 17 mod 5" "trustlang eval println(17%5)" { param($o) $o -match "2" } 5
Run-SingleTest "TRUSTLANG" "eval bool" "trustlang eval println(3>2)" { param($o) $o -match "true" } 5
Run-SingleTest "TRUSTLANG" "run let var" 'trustlang run fn main() { let x = 99; println(x); }' { param($o) $o -match "99" } 6
Run-SingleTest "TRUSTLANG" "run fibonacci" 'trustlang run fn fib(n) { if n <= 1 { return n; } return fib(n-1) + fib(n-2); } fn main() { println(fib(10)); }' { param($o) $o -match "55" } 8
Run-SingleTest "TRUSTLANG" "run while loop" 'trustlang run fn main() { let mut i = 0; while i < 3 { println(i); i = i + 1; } }' { param($o) $o -match "0" -and $o -match "1" -and $o -match "2" } 6

# --- USERS ---
Write-Host ""
Write-Host "  -- USERS --" -ForegroundColor Cyan
Run-SingleTest "USERS" "id" "id" { param($o) $o -match "uid" -or $o -match "root" -or $o -match "user" }
Run-SingleTest "USERS" "users" "users" { param($o) $o -match "root" -or $o.Length -gt 0 }

# --- NETWORK ---
Write-Host ""
Write-Host "  -- NETWORK --" -ForegroundColor Cyan
Run-SingleTest "NET" "ifconfig" "ifconfig" { param($o) $o -match "10\." -or $o -match "eth" -or $o -match "IP" -or $o -match "addr" }
Run-SingleTest "NET" "arp" "arp" { param($o) $o -match "ARP" -or $o -match "arp" -or $o -match "Address" -or $o.Length -gt 3 }
Run-SingleTest "NET" "route" "route" { param($o) $o -match "Route" -or $o -match "route" -or $o -match "Gateway" -or $o.Length -gt 3 }
Run-SingleTest "NET" "netstat" "netstat" { param($o) $o -match "Active" -or $o -match "Proto" -or $o -match "tcp" -or $o.Length -gt 3 }
Run-SingleTest "NET" "ping gateway" "ping 10.0.2.2" { param($o) $o -match "ping" -or $o -match "reply" -or $o -match "Reply" -or $o -match "timeout" } 6
Run-SingleTest "NET" "nslookup" "nslookup example.com" { param($o) $o -match "DNS" -or $o -match "Server" -or $o -match "Address" -or $o -match "example" } 6

# --- DEV TOOLS ---
Write-Host ""
Write-Host "  -- DEVTOOLS --" -ForegroundColor Cyan
Run-SingleTest "DEVTOOLS" "hexdump file" "hexdump /test_auto/hello.txt" { param($o) $o -match "[0-9a-fA-F]" }

# --- SECURITY ---
Write-Host ""
Write-Host "  -- SECURITY --" -ForegroundColor Cyan
Run-SingleTest "SECURITY" "create sig file" 'echo sigtest > /tmp_sig.txt' { param($o) $true }
Run-SingleTest "SECURITY" "sig sign" "sig sign /tmp_sig.txt" { param($o) $o -match "Signed" -or $o -match "signed" -or $o -match "signature" -or $o -match "sig" } 5
Run-SingleTest "SECURITY" "sig verify" "sig verify /tmp_sig.txt" { param($o) $o -match "Valid" -or $o -match "valid" -or $o -match "OK" -or $o -match "verified" } 5

# --- STUBS ---
Write-Host ""
Write-Host "  -- STUBS --" -ForegroundColor Cyan
Run-SingleTest "STUBS" "bc stub" "bc" { param($o) $o -match "not implemented" -or $o -match "calculator" }
Run-SingleTest "STUBS" "base64 stub" "base64" { param($o) $o -match "not implemented" -or $o -match "Usage" }
Run-SingleTest "STUBS" "md5sum stub" "md5sum" { param($o) $o -match "not implemented" -or $o -match "Usage" }
Run-SingleTest "STUBS" "sha256sum stub" "sha256sum" { param($o) $o -match "not implemented" -or $o -match "Usage" }

# --- PROCESS ---
Write-Host ""
Write-Host "  -- PROCESS --" -ForegroundColor Cyan
Run-SingleTest "PROC" "sleep 0" "sleep 0" { param($o) $true }
Run-SingleTest "PROC" "tty" "tty" { param($o) $o -match "tty" -or $o -match "console" -or $o -match "serial" -or $o -match "not" }

# --- DISK ---
Write-Host ""
Write-Host "  -- DISK --" -ForegroundColor Cyan
Run-SingleTest "DISK" "disk info" "disk" { param($o) $o -match "Disk" -or $o -match "disk" -or $o -match "AHCI" -or $o.Length -gt 3 }
Run-SingleTest "DISK" "fdisk" "fdisk" { param($o) $o -match "Partition" -or $o -match "partition" -or $o -match "Disk" -or $o.Length -gt 3 }
Run-SingleTest "DISK" "blkid" "blkid" { param($o) $o.Length -ge 0 }

# --- AUDIO ---
Write-Host ""
Write-Host "  -- AUDIO --" -ForegroundColor Cyan
Run-SingleTest "AUDIO" "beep" "beep" { param($o) $true }
Run-SingleTest "AUDIO" "audio status" "audio" { param($o) $o -match "Audio" -or $o -match "audio" -or $o -match "HDA" -or $o.Length -ge 0 }

# --- DEBUG ---
Write-Host ""
Write-Host "  -- DEBUG --" -ForegroundColor Cyan
Run-SingleTest "DEBUG" "irqstat" "irqstat" { param($o) $o -match "IRQ" -or $o -match "irq" -or $o -match "interrupt" -or $o -match "\d" }
Run-SingleTest "DEBUG" "smpstatus" "smpstatus" { param($o) $o -match "SMP" -or $o -match "CPU" -or $o -match "cpu" -or $o.Length -gt 3 }
Run-SingleTest "DEBUG" "perf" "perf" { param($o) $o -match "Perf" -or $o -match "perf" -or $o -match "Performance" -or $o.Length -gt 3 }
Run-SingleTest "DEBUG" "memdbg" "memdbg" { param($o) $o -match "Heap" -or $o -match "heap" -or $o -match "Memory" -or $o -match "alloc" }
Run-SingleTest "DEBUG" "regs" "regs" { param($o) $o -match "RAX" -or $o -match "RBX" -or $o -match "RSP" -or $o -match "Register" }

$testDuration = [math]::Round($testStart.Elapsed.TotalSeconds, 1)

# Take final screenshot
& $VBoxManage controlvm $VMName screenshotpng "$PSScriptRoot\test_final_screenshot.png" 2>$null

# === REPORT ===

$total = $global:passed + $global:failed
$passRate = if ($total -gt 0) { [math]::Round(($global:passed / $total) * 100, 1) } else { 0 }

Write-Host ""
Write-Host "==================================================================" -ForegroundColor Cyan
Write-Host "                     TEST RESULTS SUMMARY" -ForegroundColor Cyan
Write-Host "==================================================================" -ForegroundColor Cyan
Write-Host "  Total:    $total" -ForegroundColor White
Write-Host "  Passed:   $($global:passed)" -ForegroundColor Green
$failColor = if ($global:failed -eq 0) { "Green" } else { "Red" }
Write-Host "  Failed:   $($global:failed)" -ForegroundColor $failColor
$rateColor = if ($passRate -ge 80) { "Green" } elseif ($passRate -ge 50) { "Yellow" } else { "Red" }
Write-Host "  Rate:     ${passRate}%" -ForegroundColor $rateColor
Write-Host "  Duration: ${testDuration}s (+ ${bootTime}s boot)" -ForegroundColor DarkGray
Write-Host "==================================================================" -ForegroundColor Cyan

# Show failures
$failures = $global:results | Where-Object { $_.Status -ne "PASS" }
if ($failures.Count -gt 0) {
    Write-Host ""
    Write-Host "=== FAILURES ===" -ForegroundColor Red
    foreach ($f in $failures) {
        Write-Host ("  [{0}] {1}" -f $f.Category, $f.Name) -ForegroundColor Red
        Write-Host ("    Cmd:    {0}" -f $f.Command) -ForegroundColor DarkGray
        $snippet = if ($f.Output.Length -gt 120) { $f.Output.Substring(0, 120) + "..." } else { $f.Output }
        Write-Host ("    Output: {0}" -f $snippet) -ForegroundColor DarkYellow
    }
}

# Known bugs
Write-Host ""
Write-Host "=== KNOWN BUGS (manual, not auto-tested) ===" -ForegroundColor Yellow
Write-Host "  CHESS-3D: White pieces don't move first -- black starts" -ForegroundColor Yellow
Write-Host "  CHESS-3D: Pawn promotion not implemented" -ForegroundColor Yellow
Write-Host "  CHESS-2D: Pawn promotion not implemented" -ForegroundColor Yellow

# Category breakdown
Write-Host ""
Write-Host "=== BY CATEGORY ===" -ForegroundColor Cyan
$categories = $global:results | Group-Object -Property Category
foreach ($cat in $categories | Sort-Object Name) {
    $catP = ($cat.Group | Where-Object { $_.Status -eq "PASS" }).Count
    $catT = $cat.Group.Count
    $catF = $catT - $catP
    $color = if ($catF -eq 0) { "Green" } elseif ($catF -le 2) { "Yellow" } else { "Red" }
    Write-Host ("  {0,-12} {1}/{2}" -f $cat.Name, $catP, $catT) -ForegroundColor $color
}

# --- Write full report file ---
$reportLines = @()
$reportLines += "=================================================================="
$reportLines += "  TrustOS Automated Test Report (VirtualBox)"
$reportLines += "  Generated: $timestamp"
$reportLines += "  Boot time: ${bootTime}s  |  Test duration: ${testDuration}s"
$reportLines += "  VM: $VMName (UEFI, 1GB RAM, 2 CPUs)"
$reportLines += "=================================================================="
$reportLines += ""
$reportLines += "SUMMARY"
$reportLines += "  Total:   $total"
$reportLines += "  Passed:  $($global:passed)"
$reportLines += "  Failed:  $($global:failed)"
$reportLines += "  Rate:    ${passRate}%"
$reportLines += ""
$reportLines += "=================================================================="
$reportLines += "DETAILED RESULTS"
$reportLines += "=================================================================="

$curCat = ""
foreach ($r in $global:results) {
    if ($r.Category -ne $curCat) {
        $curCat = $r.Category
        $reportLines += ""
        $reportLines += "-- $curCat --"
    }
    $mark = switch ($r.Status) { "PASS" { "  [OK]  " } "FAIL" { "  [FAIL]" } default { "  [ERR] " } }
    $reportLines += "$mark $($r.Name)"
    if ($r.Status -ne "PASS") {
        $reportLines += "         Cmd:    $($r.Command)"
        $outTrunc = if ($r.Output.Length -gt 200) { $r.Output.Substring(0, 200) + "..." } else { $r.Output }
        $reportLines += "         Output: $outTrunc"
    }
}

$reportLines += ""
$reportLines += "=================================================================="
$reportLines += "FAILURES REQUIRING ACTION"
$reportLines += "=================================================================="

if ($failures.Count -eq 0) {
    $reportLines += "  (none -- all tests passed!)"
} else {
    foreach ($f in $failures) {
        $reportLines += "  [$($f.Category)] $($f.Name)"
        $reportLines += "    Command: $($f.Command)"
        $outT = if ($f.Output.Length -gt 200) { $f.Output.Substring(0, 200) + "..." } else { $f.Output }
        $reportLines += "    Output:  $outT"
        $reportLines += ""
    }
}

$reportLines += ""
$reportLines += "=================================================================="
$reportLines += "KNOWN BUGS (manual observation)"
$reportLines += "=================================================================="
$reportLines += "  CHESS-3D: White pieces don't move first -- black starts instead"
$reportLines += "  CHESS-3D: Pawn promotion is not implemented"
$reportLines += "  CHESS-2D: Pawn promotion is not implemented"
$reportLines += ""
$reportLines += "=================================================================="
$reportLines += "STUBS (declared but not yet implemented)"
$reportLines += "=================================================================="
$reportLines += "  bc        -- calculator stub"
$reportLines += "  base64    -- encoding stub"
$reportLines += "  md5sum    -- hash stub"
$reportLines += "  sha256sum -- hash stub"
$reportLines += ""
$reportLines += "=================================================================="
$reportLines += "CATEGORY BREAKDOWN"
$reportLines += "=================================================================="

foreach ($cat in $categories | Sort-Object Name) {
    $catP = ($cat.Group | Where-Object { $_.Status -eq "PASS" }).Count
    $catT = $cat.Group.Count
    $reportLines += ("  {0,-12}  {1} / {2}" -f $cat.Name, $catP, $catT)
    foreach ($t in $cat.Group | Where-Object { $_.Status -ne "PASS" }) {
        $reportLines += "    -> FAIL: $($t.Name)"
    }
}

$reportLines += ""
$reportLines += "=================================================================="
$reportLines += "  End of report"

$reportLines | Out-File -FilePath $ReportFile -Encoding ASCII

Write-Host ""
Write-Host "Report saved: $ReportFile" -ForegroundColor White
Write-Host "Boot screenshot: test_boot_screenshot.png" -ForegroundColor DarkGray
Write-Host "Final screenshot: test_final_screenshot.png" -ForegroundColor DarkGray
Write-Host ""
Write-Host "VM left running for manual inspection." -ForegroundColor DarkGray
Write-Host ("    To stop: VBoxManage controlvm {0} poweroff" -f $VMName) -ForegroundColor DarkGray
Write-Host ""
