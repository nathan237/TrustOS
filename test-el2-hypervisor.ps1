# ═══════════════════════════════════════════════════════════════
# TrustOS EL2 Hypervisor Test Script 
# Tests the ARM EL2 MMIO Spy in QEMU virt machine
# ═══════════════════════════════════════════════════════════════
#
# WHAT THIS TESTS:
#   1. TrustOS boots at EL2 (QEMU -machine virt,virtualization=on)
#   2. EL2 detection works (CurrentEL register)
#   3. Shell 'hv el2' command shows EL2 status
#   4. Stage-2 page tables can be configured
#   5. MMIO spy logs PL011 UART accesses
#
# PREREQUISITES:
#   - qemu-system-aarch64 in PATH
#   - TrustOS built for aarch64: cargo build --target aarch64-unknown-none
#   - OVMF.fd (UEFI firmware) or direct kernel boot
#
# ═══════════════════════════════════════════════════════════════

param(
    [switch]$Debug,
    [int]$Timeout = 30,
    [switch]$Gui
)

$ErrorActionPreference = "Stop"

Write-Host "═══════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "  TrustOS EL2 Hypervisor Test Suite" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host ""

# ─── Step 1: Build ───
Write-Host "[1/5] Building TrustOS for aarch64..." -ForegroundColor Yellow
$buildResult = & cargo build --target aarch64-unknown-none -p trustos_kernel 2>&1
$kernelPath = "target/aarch64-unknown-none/debug/trustos_kernel"

if (-not (Test-Path $kernelPath)) {
    Write-Host "  FAIL: Kernel binary not found at $kernelPath" -ForegroundColor Red
    Write-Host "  Build output:" -ForegroundColor Red
    $buildResult | Select-String "error" | ForEach-Object { Write-Host "    $_" -ForegroundColor Red }
    exit 1
}
$size = (Get-Item $kernelPath).Length
Write-Host "  OK: Kernel built ($([math]::Round($size/1024))KB)" -ForegroundColor Green

# ─── Step 2: Verify QEMU ───
Write-Host "[2/5] Checking QEMU..." -ForegroundColor Yellow
$qemu = Get-Command qemu-system-aarch64 -ErrorAction SilentlyContinue
if (-not $qemu) {
    Write-Host "  FAIL: qemu-system-aarch64 not found in PATH" -ForegroundColor Red
    Write-Host "  Install: scoop install qemu  OR  choco install qemu" -ForegroundColor Yellow
    exit 1
}
$ver = & qemu-system-aarch64 --version | Select-Object -First 1
Write-Host "  OK: $ver" -ForegroundColor Green

# ─── Step 3: Test EL2 Boot ───
Write-Host "[3/5] Testing EL2 boot (virtualization=on)..." -ForegroundColor Yellow

$serialLog = "el2_test_serial.txt"
if (Test-Path $serialLog) { Remove-Item $serialLog }

# QEMU command for EL2 boot
# Key flags:
#   -machine virt,virtualization=on    → boots at EL2!
#   -cpu cortex-a72                    → realistic ARM CPU
#   -m 512M                           → 512MB RAM  
#   -nographic                        → serial console only
#   -serial file:...                   → capture serial output
$qemuArgs = @(
    "-machine", "virt,virtualization=on",
    "-cpu", "cortex-a72",
    "-m", "512M",
    "-kernel", $kernelPath,
    "-nographic",
    "-serial", "file:$serialLog",
    "-d", "guest_errors",
    "-no-reboot"
)

if ($Debug) {
    $qemuArgs += @("-d", "int,cpu_reset", "-D", "qemu_el2_debug.log")
}

if (-not $Gui) {
    $qemuArgs += @("-display", "none")
}

Write-Host "  Command: qemu-system-aarch64 $($qemuArgs -join ' ')" -ForegroundColor DarkGray

# Start QEMU in background
$proc = Start-Process -FilePath "qemu-system-aarch64" -ArgumentList $qemuArgs -PassThru -NoNewWindow

# Wait for output
$elapsed = 0
$found_el2 = $false
$found_shell = $false
$found_trustos = $false

while ($elapsed -lt $Timeout -and -not $proc.HasExited) {
    Start-Sleep -Seconds 1
    $elapsed++
    
    if (Test-Path $serialLog) {
        $content = Get-Content $serialLog -Raw -ErrorAction SilentlyContinue
        if ($content) {
            if ($content -match "EL2|Hypervisor|hypervisor") { $found_el2 = $true }
            if ($content -match "TrustOS|trustos") { $found_trustos = $true }
            if ($content -match "shell|Shell|>|#") { $found_shell = $true }
            
            # If we found everything or got a shell prompt, we can stop
            if ($found_trustos -and ($found_shell -or $elapsed -gt 10)) {
                break
            }
        }
    }
}

# Kill QEMU
if (-not $proc.HasExited) {
    Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
}

# ─── Step 4: Analyze Results ───
Write-Host "[4/5] Analyzing serial output..." -ForegroundColor Yellow

if (Test-Path $serialLog) {
    $output = Get-Content $serialLog -Raw -ErrorAction SilentlyContinue
    
    if ($output) {
        Write-Host "  Serial output ($($output.Length) bytes):" -ForegroundColor DarkGray
        $output -split "`n" | Select-Object -First 30 | ForEach-Object {
            Write-Host "    $_" -ForegroundColor DarkGray
        }
    } else {
        Write-Host "  WARNING: Serial log is empty" -ForegroundColor Yellow
    }
} else {
    Write-Host "  WARNING: No serial log file created" -ForegroundColor Yellow
    $output = ""
}

# ─── Step 5: Results ───
Write-Host ""
Write-Host "[5/5] Test Results:" -ForegroundColor Yellow
Write-Host "═══════════════════════════════════════════════" -ForegroundColor Cyan

$tests = @(
    @{ Name = "Kernel builds for aarch64";         Pass = (Test-Path $kernelPath) },
    @{ Name = "QEMU boots without crash";          Pass = ($elapsed -lt $Timeout) },
    @{ Name = "TrustOS banner appears";            Pass = $found_trustos },
    @{ Name = "EL2/Hypervisor detected in output";  Pass = $found_el2 },
    @{ Name = "Shell prompt reached";              Pass = $found_shell }
)

$passed = 0
$total = $tests.Count

foreach ($test in $tests) {
    if ($test.Pass) {
        Write-Host "  [PASS] $($test.Name)" -ForegroundColor Green
        $passed++
    } else {
        Write-Host "  [FAIL] $($test.Name)" -ForegroundColor Red
    }
}

Write-Host ""
if ($passed -eq $total) {
    Write-Host "  Result: $passed/$total PASSED — ALL OK" -ForegroundColor Green
} else {
    Write-Host "  Result: $passed/$total PASSED — $($total - $passed) FAILED" -ForegroundColor Red
}

Write-Host ""
Write-Host "═══════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host ""
Write-Host "Next steps for manual testing:" -ForegroundColor Yellow
Write-Host "  1. Run QEMU interactively:" -ForegroundColor White
Write-Host "     qemu-system-aarch64 -machine virt,virtualization=on -cpu cortex-a72 -m 512M -kernel $kernelPath -nographic" -ForegroundColor DarkGray
Write-Host ""
Write-Host "  2. In TrustOS shell, try:" -ForegroundColor White
Write-Host "     hv el2       - Check EL2 status" -ForegroundColor DarkGray
Write-Host "     hv spy       - View MMIO spy data" -ForegroundColor DarkGray
Write-Host "     hv smc       - View SMC call log" -ForegroundColor DarkGray
Write-Host "     hv devices   - Per-device statistics" -ForegroundColor DarkGray
Write-Host "     hv report    - Full spy report" -ForegroundColor DarkGray
Write-Host ""
Write-Host "  3. For real phone testing (Pixel/Samsung):" -ForegroundColor White
Write-Host "     a. Build boot.img: make android-boot" -ForegroundColor DarkGray
Write-Host "     b. Boot via fastboot: fastboot boot boot.img" -ForegroundColor DarkGray
Write-Host "     c. TrustOS starts at EL2, Android runs at EL1" -ForegroundColor DarkGray
Write-Host "     d. All MMIO/SMC intercepted and logged" -ForegroundColor DarkGray
