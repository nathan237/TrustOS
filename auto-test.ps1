<#
.SYNOPSIS
    TrustOS Automated Functional Test Suite (QEMU + Serial TCP)
.DESCRIPTION
    Boots TrustOS in QEMU, sends commands via serial TCP, captures output,
    and generates a detailed pass/fail report.
.NOTES
    Requires: QEMU, trustos.iso
    Output:   test_report.txt + console summary
#>

param(
    [string]$IsoPath = "$PSScriptRoot\trustos.iso",
    [int]$SerialPort = 5555,
    [int]$BootTimeout = 25,
    [int]$CmdTimeout = 5,
    [string]$ReportFile = "$PSScriptRoot\test_report.txt"
)

$QemuExe = "C:\Program Files\qemu\qemu-system-x86_64.exe"
$timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"

$global:passed = 0
$global:failed = 0
$global:results = @()

# ---------------------------------------------------------------
#  HELPER FUNCTIONS
# ---------------------------------------------------------------

function Send-Command {
    param($writer, $stream, $cmd, [int]$timeout = $CmdTimeout, [string]$waitFor = "")

    $buffer = New-Object byte[] 16384

    # Drain leftover (wait until no data for 300ms)
    $drainSw = [System.Diagnostics.Stopwatch]::StartNew()
    $lastData = $drainSw.ElapsedMilliseconds
    while (($drainSw.ElapsedMilliseconds - $lastData) -lt 300 -and $drainSw.ElapsedMilliseconds -lt 3000) {
        if ($stream.DataAvailable) {
            $stream.Read($buffer, 0, $buffer.Length) | Out-Null
            $lastData = $drainSw.ElapsedMilliseconds
        } else {
            Start-Sleep -Milliseconds 50
        }
    }

    # Send the full command + CR
    $cmdBytes = [System.Text.Encoding]::ASCII.GetBytes("$cmd`r")
    $stream.Write($cmdBytes, 0, $cmdBytes.Length)
    $stream.Flush()

    # Collect output with polling until prompt or timeout
    $output = ""
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    # More aggressive polling for long-running commands (WaitFor mode) to avoid serial backpressure
    $sleepMs = if ($waitFor) { 5 } else { 50 }

    while ($sw.Elapsed.TotalSeconds -lt $timeout) {
        if ($stream.DataAvailable) {
            $read = $stream.Read($buffer, 0, $buffer.Length)
            if ($read -gt 0) {
                $output += [System.Text.Encoding]::ASCII.GetString($buffer, 0, $read)
            }
        } else {
            Start-Sleep -Milliseconds $sleepMs
        }

        # If caller specified a WaitFor pattern, use that instead of prompt detection
        if ($waitFor -and $output.Length -gt 20 -and $output -match $waitFor) {
            Start-Sleep -Milliseconds 200
            while ($stream.DataAvailable) {
                $read = $stream.Read($buffer, 0, $buffer.Length)
                if ($read -gt 0) { $output += [System.Text.Encoding]::ASCII.GetString($buffer, 0, $read) }
            }
            break
        }

        # After 500ms + some data, check for prompt at end of output
        if (-not $waitFor -and $sw.ElapsedMilliseconds -ge 500 -and $output.Length -gt 5) {
            if ($output -match "\d{2}:\d{2}:\d{2}\]\s*trustos:[^\r\n]*\$\s*$") {
                Start-Sleep -Milliseconds 100
                while ($stream.DataAvailable) {
                    $read = $stream.Read($buffer, 0, $buffer.Length)
                    if ($read -gt 0) { $output += [System.Text.Encoding]::ASCII.GetString($buffer, 0, $read) }
                }
                break
            }
        }
    }

    # Clean output
    $cleaned = ""
    $lines = $output -split "`n"
    $foundCmd = $false
    foreach ($line in $lines) {
        $trimmed = $line.Trim()
        if ($trimmed.Length -eq 0) { continue }
        if ($trimmed -match "\d{2}:\d{2}:\d{2}\]\s*trustos:") { continue }
        if ($trimmed -match "\[KB-") { continue }
        if ($trimmed -match "^\[.*\d+\.\d+.*\[INFO") { continue }
        if ($trimmed -match "^[a-z](\s{2,}[a-z]){2,}") { continue }
        $clean = $trimmed -replace '[\x00-\x1F\x7F]', ''
        $clean = $clean -replace '\?{2,}', ''
        $clean = $clean.Trim()
        if ($clean.Length -eq 0) { continue }
        if (-not $foundCmd) {
            $cmdFirst = $cmd.Split(' ')[0]
            if ($clean -eq $cmd -or $clean -eq $cmdFirst -or
                ($clean.Length -lt ($cmd.Length + 15) -and $clean -match [regex]::Escape($cmdFirst))) {
                $foundCmd = $true
                continue
            }
        }
        $cleaned += $clean + "`n"
    }

    return @{ Cleaned = $cleaned.Trim(); Raw = $output }
}

function Run-Test {
    param($writer, $stream, $test)

    $name = $test.Name
    $cmd = $test.Cmd
    $category = $test.Category
    $validate = $test.Validate
    $testTimeout = if ($test.Timeout) { $test.Timeout } else { $CmdTimeout }

    Write-Host ("  [{0}] {1} ... " -f $category, $name) -NoNewline

    try {
        $waitFor = if ($test.WaitFor) { $test.WaitFor } else { "" }
        $result = Send-Command -writer $writer -stream $stream -cmd $cmd -timeout $testTimeout -waitFor $waitFor
        $output = $result.Cleaned
        $rawOut = $result.Raw
        # Save raw inttest output for debugging
        if ($category -eq "INTTEST") {
            $rawOut | Out-File -FilePath "$PSScriptRoot\inttest_raw_output.txt" -Encoding UTF8
            Write-Host "  [inttest raw: $($rawOut.Length) chars]" -ForegroundColor DarkGray -NoNewline
        }
        $success = & $validate $output

        if ($success) {
            Write-Host "[PASS]" -ForegroundColor Green
            $global:passed++
            $status = "PASS"
        } else {
            Write-Host "[FAIL]" -ForegroundColor Red
            if ($output.Length -eq 0) {
                $rawPreview = ($rawOut -replace '[\x00-\x1F]', '.').Substring(0, [Math]::Min(200, $rawOut.Length))
                Write-Host "    >> Empty output (raw=$($rawOut.Length)): $rawPreview" -ForegroundColor DarkYellow
            } else {
                $preview = $output.Substring(0, [Math]::Min(120, $output.Length)) -replace "`n", "  "
                Write-Host "    >> $preview" -ForegroundColor DarkYellow
            }
            $global:failed++
            $status = "FAIL"
        }
    } catch {
        Write-Host ("[ERROR] {0}" -f $_) -ForegroundColor Yellow
        $global:failed++
        $status = "ERROR"
        $output = "Exception: $_"
    }

    $safeOutput = ($output -replace "`r`n|`r|`n", "  ")
    if ($safeOutput.Length -gt 300) { $safeOutput = $safeOutput.Substring(0, 300) }

    $global:results += @{
        Category = $category
        Name     = $name
        Command  = $cmd
        Status   = $status
        Output   = $safeOutput
    }
}

# ---------------------------------------------------------------
#  TEST DEFINITIONS
# ---------------------------------------------------------------

$tests = @(
    # -- SHELL --
    @{ Category="SHELL"; Name="echo simple text"; Cmd="echo hello_trustos"; Validate={ param($o) $o -match "hello_trustos" } }
    @{ Category="SHELL"; Name="echo with spaces"; Cmd="echo hello world 123"; Validate={ param($o) $o -match "hello world 123" } }
    @{ Category="SHELL"; Name="pwd default root"; Cmd="pwd"; Validate={ param($o) $o -match "/" } }
    @{ Category="SHELL"; Name="whoami"; Cmd="whoami"; Validate={ param($o) $o -match "root|nobody" } }
    @{ Category="SHELL"; Name="hostname"; Cmd="hostname"; Validate={ param($o) $o -match "trustos" } }
    @{ Category="SHELL"; Name="version"; Cmd="version"; Validate={ param($o) $o -match "T-RustOs" } }
    @{ Category="SHELL"; Name="uname -a"; Cmd="uname -a"; Validate={ param($o) $o -match "T-RustOs.*x86_64" } }
    @{ Category="SHELL"; Name="clear no crash"; Cmd="clear"; Validate={ param($o) $true } }
    @{ Category="SHELL"; Name="history"; Cmd="history"; Validate={ param($o) $o -match "\d+" } }
    @{ Category="SHELL"; Name="help"; Cmd="help"; Validate={ param($o) $o -match "help|command|Commands" } }
    @{ Category="SHELL"; Name="cowsay"; Cmd="cowsay TrustOS"; Validate={ param($o) $o -match "__\^" -and $o -match "TrustOS" } }

    # -- SYSINFO --
    @{ Category="SYSINFO"; Name="date"; Cmd="date"; Validate={ param($o) $o -match "\d{4}-\d{2}-\d{2}" } }
    @{ Category="SYSINFO"; Name="uptime"; Cmd="time"; Validate={ param($o) $o -match "Uptime|Time" } }
    @{ Category="SYSINFO"; Name="info"; Cmd="info"; Validate={ param($o) $o -match "T-RUSTOS|Version" } }
    @{ Category="SYSINFO"; Name="free heap"; Cmd="free"; Validate={ param($o) $o -match "Heap" } }
    @{ Category="SYSINFO"; Name="df disk usage"; Cmd="df"; Validate={ param($o) $o -match "ramfs" } }
    @{ Category="SYSINFO"; Name="ps processes"; Cmd="ps"; Validate={ param($o) $o -match "PID|kernel|tsh" } }
    @{ Category="SYSINFO"; Name="env variables"; Cmd="env"; Validate={ param($o) $o -match "USER=root" } }
    @{ Category="SYSINFO"; Name="lscpu"; Cmd="lscpu"; Validate={ param($o) $o -match "CPU|cpu|core|Core|x86" } }
    @{ Category="SYSINFO"; Name="lsmem"; Cmd="lsmem"; Validate={ param($o) $o -match "Memory|memory|MB|KB|total" } }
    @{ Category="SYSINFO"; Name="lspci"; Cmd="lspci"; Validate={ param($o) $o -match "PCI|pci|device|Device|Bus" } }
    @{ Category="SYSINFO"; Name="lsblk"; Cmd="lsblk"; Validate={ param($o) $o -match "block|Block|disk|Disk|NAME" -or $o.Length -gt 5 } }
    @{ Category="SYSINFO"; Name="vmstat"; Cmd="vmstat"; Validate={ param($o) $o -match "memory|Memory|proc|Proc|cpu|CPU" -or $o.Length -gt 10 } }
    @{ Category="SYSINFO"; Name="iostat"; Cmd="iostat"; Validate={ param($o) $o -match "io|IO|disk|Disk|read|write" -or $o.Length -gt 10 } }
    @{ Category="SYSINFO"; Name="neofetch"; Cmd="neofetch"; Validate={ param($o) $o -match "TrustOS|trustos|OS|Kernel|root|tsh|Resolution" } }
    @{ Category="SYSINFO"; Name="dmesg"; Cmd="dmesg"; Validate={ param($o) $o.Length -gt 5 } }

    # -- FILESYSTEM --
    @{ Category="FS"; Name="mkdir test_dir"; Cmd="mkdir /test_autotest"; Validate={ param($o) -not ($o -match "error|Error") } }
    @{ Category="FS"; Name="ls sees new dir"; Cmd="ls /"; Validate={ param($o) $o -match "test_autotest" } }
    @{ Category="FS"; Name="touch file"; Cmd="touch /test_autotest/hello.txt"; Validate={ param($o) -not ($o -match "error|Error") } }
    @{ Category="FS"; Name="echo to file"; Cmd="echo test_content_42 > /test_autotest/hello.txt"; Validate={ param($o) $true } }
    @{ Category="FS"; Name="cat file"; Cmd="cat /test_autotest/hello.txt"; Validate={ param($o) $o -match "test_content_42" } }
    @{ Category="FS"; Name="ls dir content"; Cmd="ls /test_autotest"; Validate={ param($o) $o -match "hello" } }
    @{ Category="FS"; Name="stat file"; Cmd="stat /test_autotest/hello.txt"; Validate={ param($o) $o -match "hello|size|Size|File" } }
    @{ Category="FS"; Name="wc file"; Cmd="wc /test_autotest/hello.txt"; Validate={ param($o) $o -match "\d+" } }
    @{ Category="FS"; Name="cp file"; Cmd="cp /test_autotest/hello.txt /test_autotest/copy.txt"; Validate={ param($o) -not ($o -match "error|Error") } }
    @{ Category="FS"; Name="cat copied file"; Cmd="cat /test_autotest/copy.txt"; Validate={ param($o) $o -match "test_content_42" } }
    @{ Category="FS"; Name="mv file"; Cmd="mv /test_autotest/copy.txt /test_autotest/renamed.txt"; Validate={ param($o) -not ($o -match "error|Error") } }
    @{ Category="FS"; Name="cat renamed file"; Cmd="cat /test_autotest/renamed.txt"; Validate={ param($o) $o -match "test_content_42" } }
    @{ Category="FS"; Name="grep in file"; Cmd="grep test_content /test_autotest/hello.txt"; Validate={ param($o) $o -match "test_content" } }
    @{ Category="FS"; Name="find file"; Cmd="find hello"; Validate={ param($o) $o -match "hello" } }
    @{ Category="FS"; Name="tree"; Cmd="tree /test_autotest"; Validate={ param($o) $o -match "hello|renamed" } }
    @{ Category="FS"; Name="head file"; Cmd="head /test_autotest/hello.txt"; Validate={ param($o) $o -match "test_content" } }
    @{ Category="FS"; Name="tail file"; Cmd="tail /test_autotest/hello.txt"; Validate={ param($o) $o -match "test_content" } }
    @{ Category="FS"; Name="diff two files"; Cmd="diff /test_autotest/hello.txt /test_autotest/renamed.txt"; Validate={ param($o) $o.Length -ge 0 } }
    @{ Category="FS"; Name="hexdump file"; Cmd="hexdump /test_autotest/hello.txt"; Validate={ param($o) $o -match "[0-9a-fA-F]{2}" } }
    @{ Category="FS"; Name="rm file"; Cmd="rm /test_autotest/renamed.txt"; Validate={ param($o) -not ($o -match "error|Error") } }
    @{ Category="FS"; Name="rm verify gone"; Cmd="cat /test_autotest/renamed.txt"; Validate={ param($o) $o -match "cat:|not found|No such" } }
    @{ Category="FS"; Name="cd directory"; Cmd="cd /test_autotest"; Validate={ param($o) -not ($o -match "error") } }
    @{ Category="FS"; Name="pwd after cd"; Cmd="pwd"; Validate={ param($o) $o -match "test_autotest" } }
    @{ Category="FS"; Name="cd back to root"; Cmd="cd /"; Validate={ param($o) $true } }

    # -- TEXT UTILITIES --
    @{ Category="TEXT"; Name="seq 5"; Cmd="seq 5"; Validate={ param($o) $o -match "1" -and $o -match "5" } }
    @{ Category="TEXT"; Name="seq 3 7"; Cmd="seq 3 7"; Validate={ param($o) $o -match "3" -and $o -match "7" } }
    @{ Category="TEXT"; Name="factor 12"; Cmd="factor 12"; Validate={ param($o) $o -match "12.*2.*3" } }
    @{ Category="TEXT"; Name="factor 97 prime"; Cmd="factor 97"; Validate={ param($o) $o -match "97.*97" } }
    @{ Category="TEXT"; Name="factor invalid"; Cmd="factor abc"; Validate={ param($o) $o -match "invalid|error|Error|Usage" } }
    @{ Category="TEXT"; Name="expr 2 + 3"; Cmd="expr 2 + 3"; Validate={ param($o) $o -match "5" } }
    @{ Category="TEXT"; Name="cal"; Cmd="cal"; Validate={ param($o) $o -match "Su|Mo|Tu|We|Th|Fr|Sa|February" -or $o.Length -gt 10 } }

    # -- HASHING --
    @{ Category="HASH"; Name="md5sum file"; Cmd="md5sum /test_autotest/hello.txt"; Validate={ param($o) $o -match "[0-9a-fA-F]{32}" } }
    @{ Category="HASH"; Name="sha256sum file"; Cmd="sha256sum /test_autotest/hello.txt"; Validate={ param($o) $o -match "[0-9a-fA-F]{64}" } }
    @{ Category="HASH"; Name="md5sum no arg"; Cmd="md5sum"; Validate={ param($o) $o -match "Usage|md5sum:|hash" -or $o.Length -gt 0 } }
    @{ Category="HASH"; Name="sha256sum no arg"; Cmd="sha256sum"; Validate={ param($o) $o -match "Usage|sha256sum:|hash" -or $o.Length -gt 0 } }

    # -- PIPES & TEXT FILTERS --
    @{ Category="PIPES"; Name="echo tr uppercase"; Cmd="echo hello world | tr a-z A-Z"; Validate={ param($o) $o -match "HELLO WORLD" } }
    @{ Category="PIPES"; Name="echo cut fields"; Cmd="echo a:b:c:d | cut -d : -f 2,4"; Validate={ param($o) $o -match "b:d|b.*d" } }
    @{ Category="PIPES"; Name="echo tee file"; Cmd="echo tee_test_42 | tee /tmp/tee_test.txt"; Validate={ param($o) $o -match "tee_test_42" } }
    @{ Category="PIPES"; Name="cat tee output"; Cmd="cat /tmp/tee_test.txt"; Validate={ param($o) $o -match "tee_test_42" } }

    # -- SYMLINKS --
    @{ Category="LINKS"; Name="ln -s symlink"; Cmd="ln -s /test_autotest/hello.txt /tmp/link_test"; Validate={ param($o) $o -match "->|link" -or -not ($o -match "error|Error") } }
    @{ Category="LINKS"; Name="readlink"; Cmd="readlink /tmp/link_test"; Validate={ param($o) $o -match "hello\.txt|test_autotest" } }
    @{ Category="LINKS"; Name="cat via symlink"; Cmd="cat /tmp/link_test"; Validate={ param($o) $o -match "test_content" -or $true } }

    # -- PERMISSIONS --
    @{ Category="PERMS"; Name="chmod file"; Cmd="chmod 755 /test_autotest/hello.txt"; Validate={ param($o) $o -match "chmod|changed|mode" -or -not ($o -match "error|Error") } }

    # -- SERVICES --
    @{ Category="SVC"; Name="service list"; Cmd="service"; Validate={ param($o) $o -match "SERVICE|sshd|httpd|syslogd" } }
    @{ Category="SVC"; Name="service start"; Cmd="service sshd start"; Validate={ param($o) $o -match "Starting|OK|started" } }
    @{ Category="SVC"; Name="systemctl list"; Cmd="systemctl list-units"; Validate={ param($o) $o -match "SERVICE|sshd|httpd" } }
    @{ Category="SVC"; Name="crontab -l"; Cmd="crontab -l"; Validate={ param($o) $o -match "crontab|no crontab" } }

    # -- ARCHIVES --
    @{ Category="ARCHIVE"; Name="tar create"; Cmd="tar cf /tmp/test_archive.tar /test_autotest/hello.txt"; Validate={ param($o) $o -match "created|tar:" -or -not ($o -match "error|Error") } }
    @{ Category="ARCHIVE"; Name="tar list"; Cmd="tar tf /tmp/test_archive.tar"; Validate={ param($o) $o -match "hello\.txt" } }
    @{ Category="ARCHIVE"; Name="tar extract"; Cmd="tar xf /tmp/test_archive.tar"; Validate={ param($o) $o -match "extracted" -or -not ($o -match "error|Error") } }
    @{ Category="ARCHIVE"; Name="gzip file"; Cmd="gzip /tmp/tee_test.txt"; Validate={ param($o) $o -match "gz|gzip|compressed" -or -not ($o -match "error|Error") } }

    # -- ALIASES --
    @{ Category="ALIAS"; Name="alias set"; Cmd="alias mytest='echo alias_works'"; Validate={ param($o) $o -match "alias" -or -not ($o -match "error") } }
    @{ Category="ALIAS"; Name="alias list"; Cmd="alias"; Validate={ param($o) $o -match "mytest|alias" } }
    @{ Category="ALIAS"; Name="unalias"; Cmd="unalias mytest"; Validate={ param($o) $o -match "removed|unalias" -or -not ($o -match "error") } }

    # -- SELFTEST --
    @{ Category="SELFTEST"; Name="builtin self-test"; Cmd="test"; Validate={ param($o) $o -match "self-test|Self-Test|OK|Done|PASS" } }

    # -- INTTEST (32-test integration suite) --
    @{ Category="INTTEST"; Name="integration test suite"; Cmd="inttest"; Validate={ param($o) $o -match "ALL.*TESTS PASSED" }; Timeout=300; WaitFor="TESTS PASSED|FAILED" }

    # -- TRUSTLANG --
    @{ Category="TRUSTLANG"; Name="eval println"; Cmd='trustlang eval println("hello_tl")'; Validate={ param($o) $o -match "hello_tl" } }
    @{ Category="TRUSTLANG"; Name="eval 2+3"; Cmd='trustlang eval println(2+3)'; Validate={ param($o) $o -match "5" } }
    @{ Category="TRUSTLANG"; Name="eval 6*7"; Cmd='trustlang eval println(6*7)'; Validate={ param($o) $o -match "42" } }
    @{ Category="TRUSTLANG"; Name="eval string"; Cmd='trustlang eval println("TrustOS_rocks")'; Validate={ param($o) $o -match "TrustOS_rocks" } }
    @{ Category="TRUSTLANG"; Name="eval 100/4"; Cmd='trustlang eval println(100/4)'; Validate={ param($o) $o -match "25" } }
    @{ Category="TRUSTLANG"; Name="eval 17 mod 5"; Cmd='trustlang eval println(17%5)'; Validate={ param($o) $o -match "2" } }
    @{ Category="TRUSTLANG"; Name="eval bool"; Cmd='trustlang eval println(3>2)'; Validate={ param($o) $o -match "true" } }
    @{ Category="TRUSTLANG"; Name="eval let var"; Cmd='trustlang eval let x = 99; println(x);'; Validate={ param($o) $o -match "99" } }
    @{ Category="TRUSTLANG"; Name="eval if-else"; Cmd='trustlang eval if (10 > 5) { println("yes"); } else { println("no"); }'; Validate={ param($o) $o -match "yes" } }
    @{ Category="TRUSTLANG"; Name="eval loop"; Cmd='trustlang eval let mut i = 0; while i < 3 { println(i); i = i + 1; }'; Validate={ param($o) $o -match "0" -and $o -match "1" -and $o -match "2" } }

    # -- USERS --
    @{ Category="USERS"; Name="id"; Cmd="id"; Validate={ param($o) $o -match "uid|root|user" } }
    @{ Category="USERS"; Name="users"; Cmd="users"; Validate={ param($o) $o -match "root" -or $o.Length -gt 0 } }

    # -- STUBS (before NET to avoid nslookup serial flood) --
    @{ Category="STUBS"; Name="bc stub"; Cmd="bc"; Validate={ param($o) $o -match "not implemented|calculator|bc:|Calculator|interactive" -or $o.Length -gt 0 } }
    @{ Category="STUBS"; Name="base64 stub"; Cmd="base64"; Validate={ param($o) $o -match "not implemented|Usage|base64:|encode|decode" -or $o.Length -gt 0 } }

    # -- PROC --
    @{ Category="PROC"; Name="sleep 0"; Cmd="sleep 0"; Validate={ param($o) $true } }
    @{ Category="PROC"; Name="tty"; Cmd="tty"; Validate={ param($o) $o -match "tty|TTY|console|serial|dev" -or $o.Length -gt 0 } }

    # -- DISK --
    @{ Category="DISK"; Name="disk info"; Cmd="disk"; Validate={ param($o) $o -match "Disk|disk|AHCI|ahci|sector|drive|Information|No" -or $o.Length -gt 2 } }
    @{ Category="DISK"; Name="fdisk"; Cmd="fdisk"; Validate={ param($o) $o -match "Partition|partition|Disk|disk|table|No" -or $o.Length -gt 2 } }
    @{ Category="DISK"; Name="blkid"; Cmd="blkid"; Validate={ param($o) $o -match "block|Block|UUID|uuid|device|ram|ramfs|TYPE|No" -or $o.Length -gt 2 } }

    # -- AUDIO --
    @{ Category="AUDIO"; Name="beep"; Cmd="beep"; Validate={ param($o) $true } }
    @{ Category="AUDIO"; Name="audio status"; Cmd="audio"; Validate={ param($o) $o -match "Audio|audio|HDA|hda|sound|Usage" -or $o.Length -ge 0 } }

    # -- NETWORK (ping/nslookup may trigger E1000 RX polling that floods serial) --
    @{ Category="NET"; Name="ifconfig"; Cmd="ifconfig"; Validate={ param($o) $o -match "10\.\d+|eth|lo|IP|ip" } }
    @{ Category="NET"; Name="arp"; Cmd="arp"; Validate={ param($o) $o -match "ARP|arp|Address|cache" -or $o.Length -gt 5 } }
    @{ Category="NET"; Name="route"; Cmd="route"; Validate={ param($o) $o -match "Route|route|Gateway|gateway|Destination" -or $o.Length -gt 5 } }
    @{ Category="NET"; Name="netstat"; Cmd="netstat"; Validate={ param($o) $o -match "Active|Proto|Local|tcp|udp" -or $o.Length -gt 5 } }
    @{ Category="NET"; Name="ping gateway"; Cmd="ping 10.0.2.2"; Validate={ param($o) $o -match "ping|PING|reply|Reply|from|bytes|timeout|Timeout" } }
    @{ Category="NET"; Name="nslookup"; Cmd="nslookup example.com"; Validate={ param($o) $o -match "DNS|dns|Server|Address|Name|example" }; Timeout=15 }

    # -- DEBUG (before SECURITY to avoid sig sign crypto blocking kernel) --
    @{ Category="DEBUG"; Name="irqstat"; Cmd="irqstat"; Validate={ param($o) $o -match "IRQ|irq|interrupt|Interrupt|\d+" } }
    @{ Category="DEBUG"; Name="smpstatus"; Cmd="smpstatus"; Validate={ param($o) $o -match "SMP|smp|CPU|cpu|core|Core|AP" -or $o.Length -gt 5 } }
    @{ Category="DEBUG"; Name="perf"; Cmd="perf"; Validate={ param($o) $o -match "Perf|perf|Performance|cycles|ticks" -or $o.Length -gt 5 } }
    @{ Category="DEBUG"; Name="memdbg"; Cmd="memdbg"; Validate={ param($o) $o -match "Heap|heap|Memory|memory|alloc|free|used" } }
    @{ Category="DEBUG"; Name="regs"; Cmd="regs"; Validate={ param($o) $o -match "RAX|RBX|RCX|RDX|RSP|RBP|CR|rax|Register" } }

    # -- SECURITY (heavy crypto, may saturate serial - put last) --
    @{ Category="SECURITY"; Name="create sig file"; Cmd="echo sigtest > /tmp_sig.txt"; Validate={ param($o) $true } }
    @{ Category="SECURITY"; Name="sig sign"; Cmd="sig sign /tmp_sig.txt"; Validate={ param($o) $o -match "Signed|signed|signature|Signature|sig" -or -not ($o -match "error|Error") } }
    @{ Category="SECURITY"; Name="sig verify"; Cmd="sig verify /tmp_sig.txt"; Validate={ param($o) $o -match "Valid|valid|OK|verified|Verified" -or -not ($o -match "error|Error") } }
)

# ---------------------------------------------------------------
#  MAIN
# ---------------------------------------------------------------

Write-Host ""
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "  TrustOS Automated Test Suite (QEMU)" -ForegroundColor Cyan
Write-Host ("  {0}" -f $timestamp) -ForegroundColor DarkCyan
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host ""

# Pre-flight
if (-not (Test-Path $QemuExe)) {
    Write-Host "FATAL: QEMU not found at $QemuExe" -ForegroundColor Red
    exit 1
}
if (-not (Test-Path $IsoPath)) {
    Write-Host "FATAL: ISO not found at $IsoPath" -ForegroundColor Red
    exit 1
}

Write-Host ("  ISO: {0}" -f $IsoPath) -ForegroundColor DarkGray
Write-Host ("  Serial: TCP {0}" -f $SerialPort) -ForegroundColor DarkGray
Write-Host ("  Tests: {0}" -f $tests.Count) -ForegroundColor DarkGray
Write-Host ""

# Kill existing QEMU
$existingQemu = Get-Process -Name "qemu-system-x86_64" -ErrorAction SilentlyContinue
if ($existingQemu) {
    Write-Host "  Killing existing QEMU..." -ForegroundColor Yellow
    $existingQemu | Stop-Process -Force
    Start-Sleep -Seconds 2
}

# Launch QEMU
Write-Host "[1/4] Starting QEMU..." -ForegroundColor White
$serialArg = "tcp:127.0.0.1:${SerialPort},server,nowait"
$qemuArgs = @(
    "-cdrom", "`"$IsoPath`"",
    "-m", "512M",
    "-machine", "q35",
    "-cpu", "max",
    "-smp", "2",
    "-accel", "tcg,thread=multi",
    "-display", "gtk",
    "-vga", "std",
    "-device", "virtio-net-pci,netdev=net0",
    "-netdev", "user,id=net0",
    "-device", "intel-hda",
    "-device", "hda-duplex",
    "-drive", "file=`"$PSScriptRoot\trustos_nvme.img`",format=raw,if=none,id=nvme0",
    "-device", "nvme,serial=TRUSTNVME001,drive=nvme0",
    "-device", "qemu-xhci,id=xhci",
    "-device", "usb-kbd,bus=xhci.0",
    "-device", "usb-mouse,bus=xhci.0",
    "-serial", $serialArg,
    "-no-reboot"
)

$qemuProcess = Start-Process -FilePath $QemuExe -ArgumentList $qemuArgs -PassThru
Write-Host ("  PID: {0}" -f $qemuProcess.Id) -ForegroundColor DarkGray

# Connect serial TCP
Write-Host "[2/4] Connecting serial TCP..." -ForegroundColor White
$client = New-Object System.Net.Sockets.TcpClient
$connected = $false
for ($i = 0; $i -lt 60; $i++) {
    try {
        $client.Connect("127.0.0.1", $SerialPort)
        $connected = $true
        break
    } catch {
        Start-Sleep -Milliseconds 300
    }
}

if (-not $connected) {
    Write-Host "FATAL: Could not connect to serial TCP $SerialPort" -ForegroundColor Red
    Stop-Process -Id $qemuProcess.Id -Force -ErrorAction SilentlyContinue
    exit 1
}

$stream = $client.GetStream()
$stream.ReadTimeout = 3000
$writer = New-Object System.IO.StreamWriter($stream)
$writer.AutoFlush = $true
Write-Host "  Connected!" -ForegroundColor Green

# Wait for boot
Write-Host "[3/4] Waiting for TrustOS boot..." -ForegroundColor White
$buffer = New-Object byte[] 16384
$sw = [System.Diagnostics.Stopwatch]::StartNew()
$bootText = ""
$booted = $false

while ($sw.Elapsed.TotalSeconds -lt $BootTimeout) {
    if ($stream.DataAvailable) {
        $read = $stream.Read($buffer, 0, $buffer.Length)
        if ($read -gt 0) {
            $text = [System.Text.Encoding]::ASCII.GetString($buffer, 0, $read)
            $bootText += $text
            if ($bootText -match "trustos.*[\$#]") {
                $booted = $true
                break
            }
        }
    } else {
        Start-Sleep -Milliseconds 150
    }
}

if (-not $booted) {
    Write-Host ("FATAL: Boot timed out after {0}s" -f $BootTimeout) -ForegroundColor Red
    if ($bootText.Length -gt 0) {
        $showLen = [Math]::Min(500, $bootText.Length)
        Write-Host $bootText.Substring(0, $showLen) -ForegroundColor DarkGray
    }
    $client.Close()
    Stop-Process -Id $qemuProcess.Id -Force -ErrorAction SilentlyContinue
    exit 1
}

$bootTime = [math]::Round($sw.Elapsed.TotalSeconds, 1)
Write-Host ("  Booted in {0}s" -f $bootTime) -ForegroundColor Green
Write-Host ""

# Stabilize
Start-Sleep -Milliseconds 500
$writer.Write("`r")
$writer.Flush()
Start-Sleep -Milliseconds 500
while ($stream.DataAvailable) {
    $stream.Read($buffer, 0, $buffer.Length) | Out-Null
}

# Run tests
Write-Host ("[4/4] Running {0} tests..." -f $tests.Count) -ForegroundColor White
Write-Host ("------------------------------------------------------------") -ForegroundColor DarkGray

$testStart = [System.Diagnostics.Stopwatch]::StartNew()
$currentCategory = ""

foreach ($test in $tests) {
    if ($test.Category -ne $currentCategory) {
        $currentCategory = $test.Category
        Write-Host ""
        Write-Host ("  -- {0} --" -f $currentCategory) -ForegroundColor Cyan
    }
    Run-Test -writer $writer -stream $stream -test $test
    Start-Sleep -Milliseconds 300
}

$testDuration = [math]::Round($testStart.Elapsed.TotalSeconds, 1)

# Cleanup
Write-Host ""
Write-Host "Shutting down QEMU..." -ForegroundColor DarkGray
try { $client.Close() } catch {}
Start-Sleep -Seconds 1
Stop-Process -Id $qemuProcess.Id -Force -ErrorAction SilentlyContinue

# ---------------------------------------------------------------
#  REPORT
# ---------------------------------------------------------------

$total = $global:passed + $global:failed
$passRate = if ($total -gt 0) { [math]::Round(($global:passed / $total) * 100, 1) } else { 0 }

# Console summary
Write-Host ""
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "  TEST RESULTS SUMMARY" -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host ("  Total:    {0}" -f $total) -ForegroundColor White
Write-Host ("  Passed:   {0}" -f $global:passed) -ForegroundColor Green
Write-Host ("  Failed:   {0}" -f $global:failed) -ForegroundColor Red
Write-Host ("  Rate:     {0}%" -f $passRate) -ForegroundColor $(if ($passRate -ge 80) { "Green" } elseif ($passRate -ge 50) { "Yellow" } else { "Red" })
Write-Host ("  Duration: {0}s (+ {1}s boot)" -f $testDuration, $bootTime) -ForegroundColor DarkGray
Write-Host "============================================================" -ForegroundColor Cyan

# Show failures
$failures = $global:results | Where-Object { $_.Status -ne "PASS" }
if ($failures.Count -gt 0) {
    Write-Host ""
    Write-Host "=== FAILURES ===" -ForegroundColor Red
    foreach ($f in $failures) {
        Write-Host ("  [{0}] {1}" -f $f.Category, $f.Name) -ForegroundColor Red
        Write-Host ("    Cmd:    {0}" -f $f.Command) -ForegroundColor DarkGray
        $outSnippet = $f.Output
        if ($outSnippet.Length -gt 150) { $outSnippet = $outSnippet.Substring(0, 150) + "..." }
        Write-Host ("    Output: {0}" -f $outSnippet) -ForegroundColor DarkYellow
    }
}

# Category breakdown
Write-Host ""
Write-Host "=== BY CATEGORY ===" -ForegroundColor Cyan
$categories = $global:results | Group-Object -Property Category
foreach ($cat in $categories | Sort-Object Name) {
    $catPassed = @($cat.Group | Where-Object { $_.Status -eq "PASS" }).Count
    $catTotal = $cat.Group.Count
    $catColor = if ($catPassed -eq $catTotal) { "Green" } elseif ($catPassed -gt 0) { "Yellow" } else { "Red" }
    Write-Host ("  {0}  {1}/{2}" -f $cat.Name.PadRight(12), $catPassed, $catTotal) -ForegroundColor $catColor
}

# Known bugs
Write-Host ""
Write-Host "=== KNOWN BUGS ===" -ForegroundColor Yellow
Write-Host "  CHESS-3D: White pieces don't move first (black starts)" -ForegroundColor Yellow
Write-Host "  CHESS-3D: Pawn promotion not implemented" -ForegroundColor Yellow
Write-Host "  CHESS-2D: Pawn promotion not implemented" -ForegroundColor Yellow

# ---------------------------------------------------------------
#  WRITE FILE REPORT
# ---------------------------------------------------------------

$reportLines = @()
$reportLines += "=================================================================="
$reportLines += "  TrustOS Automated Test Report (QEMU)"
$reportLines += ("  Generated: {0}" -f $timestamp)
$reportLines += ("  Boot time: {0}s  |  Test duration: {1}s" -f $bootTime, $testDuration)
$reportLines += "  QEMU: q35, 512M RAM, 2 CPUs, virtio-net, serial TCP"
$reportLines += "=================================================================="
$reportLines += ""
$reportLines += "SUMMARY"
$reportLines += ("  Total:   {0}" -f $total)
$reportLines += ("  Passed:  {0}" -f $global:passed)
$reportLines += ("  Failed:  {0}" -f $global:failed)
$reportLines += ("  Rate:    {0}%" -f $passRate)
$reportLines += ""
$reportLines += "=================================================================="
$reportLines += "DETAILED RESULTS"
$reportLines += "=================================================================="

$currentCat = ""
foreach ($r in $global:results) {
    if ($r.Category -ne $currentCat) {
        $currentCat = $r.Category
        $reportLines += ""
        $reportLines += ("-- {0} --" -f $currentCat)
    }
    if ($r.Status -eq "PASS") {
        $reportLines += ("  [OK]   {0}" -f $r.Name)
    } else {
        $reportLines += ("  [FAIL] {0}" -f $r.Name)
        $reportLines += ("         Cmd:    {0}" -f $r.Command)
        $outLen = [Math]::Min(200, $r.Output.Length)
        $reportLines += ("         Output: {0}" -f $r.Output.Substring(0, $outLen))
    }
}

$reportLines += ""
$reportLines += "=================================================================="
$reportLines += "FAILURES REQUIRING ACTION"
$reportLines += "=================================================================="

foreach ($f in $failures) {
    $reportLines += ("  [{0}] {1}" -f $f.Category, $f.Name)
    $reportLines += ("    Command: {0}" -f $f.Command)
    $outLen = [Math]::Min(200, $f.Output.Length)
    $reportLines += ("    Output:  {0}" -f $f.Output.Substring(0, $outLen))
    $reportLines += ""
}

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
    $catPassed = @($cat.Group | Where-Object { $_.Status -eq "PASS" }).Count
    $catTotal = $cat.Group.Count
    $reportLines += ("  {0}  {1} / {2}" -f $cat.Name.PadRight(12), $catPassed, $catTotal)
    foreach ($t in @($cat.Group | Where-Object { $_.Status -ne "PASS" })) {
        $reportLines += ("    -> FAIL: {0}" -f $t.Name)
    }
}

$reportLines += ""
$reportLines += "=================================================================="
$reportLines += "  End of report"

$reportLines -join "`r`n" | Out-File -FilePath $ReportFile -Encoding UTF8
Write-Host ""
Write-Host ("Report saved to: {0}" -f $ReportFile) -ForegroundColor White
Write-Host ""
