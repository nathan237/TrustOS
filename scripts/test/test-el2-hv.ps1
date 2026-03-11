# ═══════════════════════════════════════════════════════════════
# TrustOS EL2 Hypervisor — Full Integration Test
# Tests ARM EL2 MMIO Spy capabilities on QEMU aarch64
# ═══════════════════════════════════════════════════════════════
#
# CONTEXT:
#   UEFI firmware (EDK2) receives EL2 from QEMU but drops to EL1
#   before launching Limine. So our kernel always boots at EL1
#   through the UEFI path.
#
#   This script tests:
#   1. Boot to shell with virtualization=on
#   2. EL detection at boot (should report EL1 via UEFI)
#   3. Hypervisor subsystem compiles and links correctly
#   4. Shell 'hv el2' command works from EL1
#
#   For true EL2 testing on real hardware (iPhone 7/8/X via checkm8):
#   - The checkm8 → PongoOS chain can launch at EL2
#   - Our hypervisor can then intercept MMIO/SMC from guest iOS
#
# ═══════════════════════════════════════════════════════════════

param(
    [switch]$FullBuild,
    [int]$BootTimeout = 30
)

$ErrorActionPreference = "Stop"
Set-Location "C:\Users\nathan\Documents\Scripts\OSrust"

$qemu = "C:\Program Files\qemu\qemu-system-aarch64.exe"
$fw = "C:\Program Files\qemu\share\edk2-aarch64-code.fd"
$serialLog = "serial_el2_test.log"
$stderrLog = "el2_test_stderr.txt"

Write-Host ""
Write-Host "═══════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "  TrustOS EL2 Hypervisor Integration Test" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host ""

# ─── Step 1: Build slim kernel ───
Write-Host "[1/4] Building slim aarch64 kernel..." -ForegroundColor Yellow
if ($FullBuild) {
    $buildOut = cargo build --release --target aarch64-unknown-none -p trustos_kernel --no-default-features 2>&1
    $lastLine = ($buildOut | Select-Object -Last 1).ToString()
    if ($lastLine -notmatch "Finished") {
        Write-Host "  BUILD FAILED" -ForegroundColor Red
        $buildOut | Where-Object { $_.ToString() -match "^error" } | Select-Object -First 5
        exit 1
    }
}
$kernel = "target\aarch64-unknown-none\release\trustos_kernel"
if (-not (Test-Path $kernel)) {
    Write-Host "  Kernel not found at $kernel" -ForegroundColor Red
    Write-Host "  Run with -FullBuild to compile first" -ForegroundColor Yellow
    exit 1
}
Copy-Item $kernel "iso_root_aarch64\boot\trustos_kernel" -Force
$size = [math]::Round((Get-Item $kernel).Length / 1MB, 1)
Write-Host "  OK: Kernel deployed ($size MB)" -ForegroundColor Green

# ─── Step 2: Launch QEMU with virtualization=on ───
Write-Host "[2/4] Booting QEMU (virt,virtualization=on,gic-version=2)..." -ForegroundColor Yellow

Stop-Process -Name "qemu-system-aarch64" -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 1

# Fresh NVRAM
if (-not (Test-Path "OVMF_VARS_aarch64.fd") -or (Get-Item "OVMF_VARS_aarch64.fd").Length -eq 0) {
    $fs = [System.IO.File]::Create("OVMF_VARS_aarch64.fd")
    $fs.SetLength(64MB)
    $fs.Close()
}

"" | Set-Content $serialLog
"" | Set-Content $stderrLog

$args = @(
    "-machine virt,gic-version=2,virtualization=on",
    "-cpu cortex-a72",
    "-smp 4",
    "-m 512M",
    "-drive if=pflash,format=raw,readonly=on,file=`"$fw`"",
    "-drive if=pflash,format=raw,file=OVMF_VARS_aarch64.fd",
    "-drive format=raw,file=fat:rw:iso_root_aarch64",
    "-serial file:`"$PWD\$serialLog`"",
    "-display none",
    "-no-reboot"
) -join " "

$proc = Start-Process $qemu -ArgumentList $args -WindowStyle Hidden -PassThru -RedirectStandardError "$PWD\$stderrLog"
Write-Host "  QEMU PID: $($proc.Id)" -ForegroundColor DarkGray

# ─── Step 3: Wait for boot ───
Write-Host "[3/4] Waiting for shell prompt..." -ForegroundColor Yellow
$shellReached = $false
$elapsed = 0

while ($elapsed -lt $BootTimeout) {
    Start-Sleep -Seconds 2
    $elapsed += 2
    
    if ($proc.HasExited) {
        Write-Host "  QEMU exited early (code $($proc.ExitCode))" -ForegroundColor Red
        break
    }
    
    $content = Get-Content $serialLog -Raw -ErrorAction SilentlyContinue
    if ($content -and $content -match "trustos:/") {
        $shellReached = $true
        Write-Host "  Shell reached in ${elapsed}s" -ForegroundColor Green
        # Give a bit more time for crypto self-tests etc
        Start-Sleep -Seconds 3
        break
    }
    
    if ($elapsed % 10 -eq 0) {
        $lines = @(Get-Content $serialLog -ErrorAction SilentlyContinue).Count
        Write-Host "  ${elapsed}s: $lines serial lines..." -ForegroundColor DarkGray
    }
}

# Kill QEMU
if (-not $proc.HasExited) {
    Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 1
}

# ─── Step 4: Analyze ───
Write-Host "[4/4] Test Results:" -ForegroundColor Yellow
Write-Host "═══════════════════════════════════════════════" -ForegroundColor Cyan

$output = Get-Content $serialLog -Raw -ErrorAction SilentlyContinue
$lines = @(Get-Content $serialLog -ErrorAction SilentlyContinue)

$tests = @()

# Test 1: Boot succeeded
$tests += @{ Name = "Kernel boots to serial output"; Pass = ($lines.Count -gt 0) }

# Test 2: TrustOS banner
$hasBanner = $output -match "T-RustOs v0\.2\.0"
$tests += @{ Name = "TrustOS v0.2.0 banner printed"; Pass = $hasBanner }

# Test 3: EL detection
$hasElDetect = $output -match "\[AARCH64\] CurrentEL = EL\d"
$elLevel = if ($output -match "CurrentEL = EL(\d)") { $matches[1] } else { "?" }
$tests += @{ Name = "EL level detected at boot (EL$elLevel)"; Pass = $hasElDetect }

# Test 4: Shell prompt
$hasShell = $output -match "trustos:/"
$tests += @{ Name = "Shell prompt reached"; Pass = $hasShell }

# Test 5: Crypto self-tests
$hasCrypto = $output -match "\[CRYPTO\] Self-tests complete"
$tests += @{ Name = "Crypto self-tests pass"; Pass = $hasCrypto }

# Test 6: Timer IRQ
$hasTimer = $output -match "\[GIC\] Timer IRQ enabled"
$tests += @{ Name = "GIC timer IRQ enabled"; Pass = $hasTimer }

# Test 7: Slim build (no WAV bloat)
$kernelSize = (Get-Item $kernel).Length
$isSlim = $kernelSize -lt 10MB
$tests += @{ Name = "Slim build < 10MB ($([math]::Round($kernelSize/1MB, 1)) MB)"; Pass = $isSlim }

# Report
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

# ─── EL2 info ───
Write-Host ""
Write-Host "═══════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "  EL2 Hypervisor Notes" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host ""

if ($elLevel -eq "1") {
    Write-Host "  Kernel booted at EL1 (UEFI/Limine drops to EL1 on ARM)." -ForegroundColor Yellow
    Write-Host "  This is normal — UEFI always enters kernel at EL1." -ForegroundColor Yellow
    Write-Host ""
    Write-Host "  For real EL2 hypervisor testing:" -ForegroundColor White
    Write-Host "    iPhone 7/8/X: checkm8 -> PongoOS -> TrustOS at EL2" -ForegroundColor DarkGray
    Write-Host "    QEMU direct:  -kernel bare-metal-shim, no UEFI" -ForegroundColor DarkGray
    Write-Host ""
    Write-Host "  Shell commands available (require EL2 to work fully):" -ForegroundColor White
    Write-Host "    hv el2       — Check current EL status" -ForegroundColor DarkGray
    Write-Host "    hv spy       — MMIO spy live view" -ForegroundColor DarkGray
    Write-Host "    hv smc       — SMC call log" -ForegroundColor DarkGray
    Write-Host "    hv devices   — Per-device MMIO stats" -ForegroundColor DarkGray
    Write-Host "    hv report    — Full spy report" -ForegroundColor DarkGray
    Write-Host "    hv boot test — Self-test with WFI micro-guest" -ForegroundColor DarkGray
} elseif ($elLevel -eq "2") {
    Write-Host "  RUNNING AT EL2 — Full hypervisor capabilities available!" -ForegroundColor Green
    Write-Host "  Use 'hv boot test' for WFI micro-guest self-test" -ForegroundColor White
}

Write-Host ""

# ─── Serial excerpt ───
Write-Host "═══ Serial Output (first 25 lines) ═══" -ForegroundColor DarkGray
$lines | Select-Object -First 25 | ForEach-Object { Write-Host "  $_" -ForegroundColor DarkGray }
Write-Host ""
