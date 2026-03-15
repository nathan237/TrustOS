<# 
.SYNOPSIS
    TrustOS -- Unified Build Shell
.DESCRIPTION
    Single entry point for all build, test, translate, and release operations.
    Ensures the original source (kernel/src/) is ALWAYS the one being built/modified.
    Translated versions are auto-generated and NEVER edited manually.

.EXAMPLE
    .\trustos.ps1 build                  # Build kernel + ISO, launch VBox for testing
    .\trustos.ps1 build -NoRun           # Build only, don't launch VM
    .\trustos.ps1 build -Edition jarvispack  # Build JarvisPack edition
    .\trustos.ps1 release                # Full release: build all ISOs, translate, commit, push
    .\trustos.ps1 release -Tag v0.3.0    # Release with specific tag
    .\trustos.ps1 translate              # Translate source to all versions
    .\trustos.ps1 translate -Only minimal # Translate only one version
    .\trustos.ps1 clean                  # Clean all build artifacts
    .\trustos.ps1 status                 # Show project status
#>
param(
    [Parameter(Position = 0)]
    [ValidateSet("build", "release", "translate", "clean", "status", "help")]
    [string]$Command = "help",

    # build options
    [switch]$NoRun,
    [switch]$Clean,
    [ValidateSet("base", "jarvispack")]
    [string]$Edition = "base",

    # release options
    [string]$Tag = "",
    [string]$Message = "",
    [switch]$NoPush,
    [switch]$NoTranslate,

    # translate options
    [string]$Only = "",
    [switch]$DetailedOutput
)

$ErrorActionPreference = "Stop"
$Root = $PSScriptRoot
$KernelSrc = Join-Path $Root "kernel\src"
$TranslatedRoot = Join-Path $Root "translated"
$BuildsDir = Join-Path $Root "builds"
$LogsDir = Join-Path $Root "logs"
$ToolsDir = Join-Path $Root "tools"
$Translator = Join-Path $ToolsDir "source_translator.py"
$TranslateScript = Join-Path $ToolsDir "translate-all.ps1"
$LimineDir = Join-Path $Root "limine"
$FirmwareDir = Join-Path $Root "firmware"

# ================================================================
#  ENVIRONMENT SETUP
# ================================================================

function Initialize-BuildEnv {
    $cmakeBin = "C:\Program Files\CMake\bin"
    $llvmBin = "C:\Program Files\LLVM\bin"
    if (Test-Path $cmakeBin) { $env:Path = "$cmakeBin;" + $env:Path }
    if (Test-Path $llvmBin) { $env:Path = "$llvmBin;" + $env:Path }
    $env:CC = "clang"; $env:CXX = "clang++"; $env:AR = "llvm-ar"

    $mbedtlsInclude = Join-Path $Root "kernel\mbedtls-include"
    if (Test-Path $mbedtlsInclude) {
        $env:CFLAGS = "-I""$mbedtlsInclude"" -mcmodel=kernel -mno-red-zone -ffreestanding"
        $env:BINDGEN_EXTRA_CLANG_ARGS = "-I""$mbedtlsInclude"""
        $env:C_INCLUDE_PATH = $mbedtlsInclude
        $env:CPLUS_INCLUDE_PATH = $mbedtlsInclude
    }
}

# ================================================================
#  SAFETY: Protect original source
# ================================================================

function Assert-OriginalSource {
    <# Verify we are building from the ORIGINAL kernel source, not a translated copy #>
    $cwd = (Get-Location).Path
    if ($cwd -like "*\translated\*") {
        Write-Host "SAFETY ERROR: You are inside the translated/ directory!" -ForegroundColor Red
        Write-Host "  The original source is at: kernel\src\" -ForegroundColor Yellow
        Write-Host "  Translated versions are AUTO-GENERATED. Never edit them." -ForegroundColor Yellow
        exit 1
    }
}

# ================================================================
#  CORE: Build Kernel
# ================================================================

function Build-Kernel {
    param([switch]$CleanFirst)

    Assert-OriginalSource
    Initialize-BuildEnv
    Push-Location $Root

    try {
        if ($CleanFirst) {
            Write-Host "  [clean] Removing previous build artifacts..." -ForegroundColor Yellow
            cargo clean 2>$null
        }

        Write-Host "  [build] Compiling kernel (release)..." -ForegroundColor Yellow
        $ErrorActionPreference = "Continue"
        cargo build --release -p trustos_kernel 2>&1 | ForEach-Object { Write-Host "    $_" }
        $ErrorActionPreference = "Stop"

        if ($LASTEXITCODE -ne 0) {
            Write-Host "  BUILD FAILED!" -ForegroundColor Red
            return $false
        }

        $kernelPath = "target\x86_64-unknown-none\release\trustos_kernel"
        if (-not (Test-Path $kernelPath)) {
            Write-Host "  Kernel binary not found at $kernelPath" -ForegroundColor Red
            return $false
        }

        $kernelSize = [math]::Round((Get-Item $kernelPath).Length / 1MB, 2)
        Write-Host "  [build] Kernel OK: $kernelSize MB" -ForegroundColor Green
        return $true
    }
    finally { Pop-Location }
}

# ================================================================
#  CORE: Create ISO
# ================================================================

function New-TrustOSIso {
    param(
        [string]$EditionName = "base",
        [string]$OutputPath = "",
        [string]$BrainFile = ""
    )

    Push-Location $Root
    try {
        $kernelPath = "target\x86_64-unknown-none\release\trustos_kernel"
        if (-not (Test-Path $kernelPath)) {
            Write-Host "  Kernel not found -- build first" -ForegroundColor Red
            return $false
        }

        $isoDir = "iso_root"
        if (Test-Path $isoDir) { Remove-Item -Recurse -Force $isoDir }
        New-Item -ItemType Directory -Path (Join-Path $isoDir "boot\limine") -Force | Out-Null
        New-Item -ItemType Directory -Path (Join-Path $isoDir "EFI\BOOT") -Force | Out-Null

        Copy-Item $kernelPath (Join-Path $isoDir "boot\trustos_kernel")

        # Limine config: strip module lines for base edition
        if ($EditionName -eq "base") {
            $limineLines = (Get-Content "limine.conf") | Where-Object { $_ -notmatch "module_path|module_cmdline" }
            $limineLines | Set-Content (Join-Path $isoDir "boot\limine\limine.conf")
            $limineLines | Set-Content (Join-Path $isoDir "limine.conf")
            $limineLines | Set-Content (Join-Path $isoDir "boot\limine\limine.cfg")
        }
        else {
            Copy-Item "limine.conf" (Join-Path $isoDir "boot\limine\limine.conf")
            Copy-Item "limine.conf" (Join-Path $isoDir "limine.conf")
            Copy-Item "limine.conf" (Join-Path $isoDir "boot\limine\limine.cfg")
        }

        # Copy bootloader files
        foreach ($f in @("limine-bios.sys", "limine-bios-cd.bin", "limine-uefi-cd.bin")) {
            $src = Join-Path $LimineDir $f
            if (Test-Path $src) { Copy-Item $src (Join-Path $isoDir "boot\limine") }
        }
        foreach ($f in @("BOOTX64.EFI", "BOOTIA32.EFI")) {
            $src = Join-Path $LimineDir $f
            if (Test-Path $src) { Copy-Item $src (Join-Path $isoDir "EFI\BOOT") }
        }

        # JarvisPack: include brain weights
        if ($BrainFile -and (Test-Path $BrainFile)) {
            Copy-Item $BrainFile (Join-Path $isoDir "jarvis_pretrained.bin")
            $brainSz = [math]::Round((Get-Item $BrainFile).Length / 1MB, 2)
            Write-Host "  [iso] JARVIS brain included: $brainSz MB" -ForegroundColor Magenta
        }

        # Determine output path
        if (-not $OutputPath) {
            if ($EditionName -eq "jarvispack") { $edDir = "trustos-jarvispack"; $isoName = "trustos-jarvispack.iso" }
            else { $edDir = "trustos"; $isoName = "trustos.iso" }
            $outDir = Join-Path $BuildsDir $edDir
            New-Item -ItemType Directory -Path $outDir -Force | Out-Null
            $OutputPath = Join-Path $outDir $isoName
        }

        # Create ISO via xorriso
        Write-Host "  [iso] Creating bootable ISO..." -ForegroundColor Yellow
        $xorriso = Get-Command xorriso -ErrorAction SilentlyContinue
        $oldErr = $ErrorActionPreference
        $ErrorActionPreference = "Continue"
        if (-not $xorriso) {
            # Use WSL xorriso
            $full = [System.IO.Path]::GetFullPath($isoDir)
            $drive = $full.Substring(0, 1).ToLower()
            $rest = $full.Substring(2) -replace "\\", "/"
            $wslIsoDir = "/mnt/$drive$rest"

            $full2 = [System.IO.Path]::GetFullPath($OutputPath)
            $drive2 = $full2.Substring(0, 1).ToLower()
            $rest2 = $full2.Substring(2) -replace "\\", "/"
            $wslIsoPath = "/mnt/$drive2$rest2"

            wsl -e xorriso -as mkisofs -R -r -J `
                -b boot/limine/limine-bios-cd.bin -no-emul-boot -boot-load-size 4 -boot-info-table `
                --efi-boot boot/limine/limine-uefi-cd.bin -efi-boot-part --efi-boot-image `
                --protective-msdos-label --mbr-force-bootable `
                $wslIsoDir -o $wslIsoPath 2>&1 | Out-Null
        }
        else {
            xorriso -as mkisofs -R -r -J `
                -b boot/limine/limine-bios-cd.bin -no-emul-boot -boot-load-size 4 -boot-info-table `
                --efi-boot boot/limine/limine-uefi-cd.bin -efi-boot-part --efi-boot-image `
                --protective-msdos-label --mbr-force-bootable `
                $isoDir -o $OutputPath 2>&1 | Out-Null
        }
        $ErrorActionPreference = $oldErr

        if ($LASTEXITCODE -ne 0) {
            Write-Host "  ISO creation failed!" -ForegroundColor Red
            return $false
        }

        # Install Limine BIOS boot sectors
        $limineExe = Join-Path $LimineDir "limine.exe"
        if (Test-Path $limineExe) {
            $ErrorActionPreference = "Continue"
            & $limineExe bios-install $OutputPath 2>&1 | Out-Null
            $ErrorActionPreference = "Stop"
        }

        # Patch MBR bootable flag
        $isoBytes = [System.IO.File]::ReadAllBytes($OutputPath)
        if ($isoBytes.Length -gt 446 -and $isoBytes[446] -ne 0x80) {
            $isoBytes[446] = 0x80
            [System.IO.File]::WriteAllBytes($OutputPath, $isoBytes)
        }

        # Copy to root for VBox convenience
        Copy-Item $OutputPath (Join-Path $Root "trustos.iso") -Force

        $isoSize = [math]::Round((Get-Item $OutputPath).Length / 1MB, 2)
        Write-Host "  [iso] $EditionName ISO: $isoSize MB -> $OutputPath" -ForegroundColor Green
        return $true
    }
    finally { Pop-Location }
}

# ================================================================
#  CORE: Launch VirtualBox for Testing
# ================================================================

function Start-VBoxTest {
    $VBM = "C:\Program Files\Oracle\VirtualBox\VBoxManage.exe"
    $VMName = "TRustOs"
    $ISOPath = Join-Path $Root "trustos.iso"
    $SerialLog = Join-Path $LogsDir "serial_vbox_test.log"
    $LogFile = Join-Path $LogsDir "vbox_test_launch.log"

    if (-not (Test-Path $ISOPath)) {
        Write-Host "  No ISO found at $ISOPath" -ForegroundColor Red
        return
    }
    if (-not (Test-Path $VBM)) {
        Write-Host "  VBoxManage not found -- is VirtualBox installed?" -ForegroundColor Red
        return
    }

    Write-Host "  [vbox] Setting up test VM..." -ForegroundColor Yellow

    $ErrorActionPreference = "Continue"

    # Cleanup previous VM
    & $VBM controlvm $VMName poweroff 2>&1 | Out-Null
    Start-Sleep -Seconds 2
    & $VBM unregistervm $VMName --delete 2>&1 | Out-Null
    Start-Sleep -Seconds 1

    # Clean inaccessible VMs
    $vms = & $VBM list vms 2>&1
    foreach ($vmLine in $vms) {
        if ($vmLine -like '*<inaccessible>*') {
            $uuid = $vmLine -replace '.*\{' -replace '\}.*'
            if ($uuid) { & $VBM unregistervm $uuid --delete 2>&1 | Out-Null }
        }
    }

    # Create and configure VM
    & $VBM createvm --name $VMName --ostype "Other_64" --register 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) {
        Write-Host "  Failed to create VM" -ForegroundColor Red
        $ErrorActionPreference = "Stop"
        return
    }

    & $VBM modifyvm $VMName --memory 1024 --vram 128 --cpus 4 2>&1 | Out-Null
    & $VBM modifyvm $VMName --firmware efi64 2>&1 | Out-Null
    & $VBM modifyvm $VMName --graphicscontroller vboxsvga 2>&1 | Out-Null
    & $VBM modifyvm $VMName --boot1 dvd --boot2 disk --boot3 none --boot4 none 2>&1 | Out-Null
    & $VBM modifyvm $VMName --audio-driver default --audio-controller hda --audio-enabled on --audio-out on 2>&1 | Out-Null
    & $VBM modifyvm $VMName --nic1 nat --nictype1 82540EM --cableconnected1 on 2>&1 | Out-Null

    # Storage
    & $VBM storagectl $VMName --name "SATA" --add sata --controller IntelAhci --portcount 4 2>&1 | Out-Null
    & $VBM storageattach $VMName --storagectl "SATA" --port 0 --device 0 --type dvddrive --medium $ISOPath 2>&1 | Out-Null

    # Data disk (if exists)
    $DataDisk = Join-Path $BuildsDir "trustos_data.img"
    if (Test-Path $DataDisk) {
        $DataVDI = Join-Path $BuildsDir "trustos_data.vdi"
        Remove-Item $DataVDI -ErrorAction SilentlyContinue
        & $VBM convertfromraw $DataDisk $DataVDI --format VDI 2>&1 | Out-Null
        & $VBM storageattach $VMName --storagectl "SATA" --port 3 --device 0 --type hdd --medium $DataVDI 2>&1 | Out-Null
    }

    # Serial
    Remove-Item $SerialLog -ErrorAction SilentlyContinue
    & $VBM modifyvm $VMName --uart1 0x3F8 4 --uartmode1 file $SerialLog 2>&1 | Out-Null

    # Launch
    Write-Host "  [vbox] Starting VM..." -ForegroundColor Green
    & $VBM startvm $VMName 2>&1 | Out-Null

    $ErrorActionPreference = "Stop"

    Write-Host "  [vbox] VM launched. Serial log: logs\serial_vbox_test.log" -ForegroundColor Cyan
    Write-Host "  [vbox] Waiting 15s for boot..." -ForegroundColor DarkGray
    Start-Sleep -Seconds 15

    if (Test-Path $SerialLog) {
        Write-Host ""
        Write-Host "  === Last 10 lines of serial output ===" -ForegroundColor Cyan
        Get-Content $SerialLog -Tail 10 | ForEach-Object { Write-Host "    $_" -ForegroundColor DarkGray }
    }
}

# ================================================================
#  CORE: Translate Source
# ================================================================

function Invoke-Translate {
    param(
        [string]$OnlyVersion = "",
        [switch]$VerboseOutput
    )

    if (-not (Test-Path $Translator)) {
        Write-Host "  Translator not found at $Translator" -ForegroundColor Red
        return $false
    }

    Write-Host "  [translate] Generating translated source versions..." -ForegroundColor Yellow

    $versions = @(
        @{ Name = "original";       Preset = "original";    Lang = "en"; Desc = "Exact copy of original source" }
        @{ Name = "minimal";        Preset = "minimal";     Lang = "en"; Desc = "Shortest identifiers, no comments" }
        @{ Name = "educational-en"; Preset = "educational"; Lang = "en"; Desc = "Expanded names + English annotations" }
        @{ Name = "educational-fr"; Preset = "educational"; Lang = "fr"; Desc = "Expanded names + French annotations" }
    )

    if ($OnlyVersion) {
        $versions = $versions | Where-Object { $_.Name -eq $OnlyVersion }
        if (-not $versions) {
            Write-Host "  Unknown version: $OnlyVersion" -ForegroundColor Red
            Write-Host "  Available: original, minimal, educational-en, educational-fr" -ForegroundColor Yellow
            return $false
        }
    }

    $generated = 0
    foreach ($ver in $versions) {
        $name   = $ver.Name
        $outDir = Join-Path (Join-Path $TranslatedRoot $name) "kernel\src"
        $mapDir = Join-Path $TranslatedRoot $name

        Write-Host "    [$($generated+1)/$($versions.Count)] $name -- $($ver.Desc)" -ForegroundColor Yellow

        New-Item -ItemType Directory -Path $mapDir -Force | Out-Null

        $pyArgs = @(
            $Translator,
            "--preset", $ver.Preset,
            "--lang", $ver.Lang,
            "-i", $KernelSrc,
            "-o", $outDir,
            "--save-mapping", (Join-Path $mapDir "mapping.json")
        )
        if ($VerboseOutput) { $pyArgs += "--verbose" }

        $pyOutput = & python @pyArgs 2>&1
        $exitCode = $LASTEXITCODE

        if ($exitCode -ne 0) {
            foreach ($line in $pyOutput) {
                if ($line -match "error|Error|ERROR") { Write-Host "      $line" -ForegroundColor Red }
            }
            Write-Host "      FAILED" -ForegroundColor Red
            continue
        }

        # Copy build support files
        $kernelRoot = Join-Path $Root "kernel"
        $destKernel = Join-Path (Join-Path $TranslatedRoot $name) "kernel"

        Copy-Item (Join-Path $kernelRoot "Cargo.toml") (Join-Path $destKernel "Cargo.toml") -Force
        Copy-Item (Join-Path $kernelRoot "build.rs") (Join-Path $destKernel "build.rs") -Force
        foreach ($ld in (Get-ChildItem -Path $kernelRoot -Filter "linker*.ld")) {
            Copy-Item $ld.FullName (Join-Path $destKernel $ld.Name) -Force
        }

        # Create standalone workspace Cargo.toml
        New-TranslatedCargoToml (Join-Path $TranslatedRoot $name)
        Copy-Item (Join-Path $Root "rust-toolchain.toml") (Join-Path (Join-Path $TranslatedRoot $name) "rust-toolchain.toml") -Force

        # Copy mbedtls includes if needed
        $mbedInc = Join-Path $kernelRoot "mbedtls-include"
        $destMbedInc = Join-Path $destKernel "mbedtls-include"
        if ((Test-Path $mbedInc) -and -not (Test-Path $destMbedInc)) {
            Copy-Item -Recurse $mbedInc $destMbedInc
        }

        # Place DO_NOT_EDIT marker
        $markerPath = Join-Path (Join-Path $TranslatedRoot $name) "DO_NOT_EDIT.md"
        if (-not (Test-Path $markerPath)) {
            $markerDate = Get-Date -Format "yyyy-MM-dd HH:mm"
            $markerContent = @(
                "# DO NOT EDIT - AUTO-GENERATED",
                "",
                "This directory is a **translated version** of the TrustOS source code.",
                "It is generated automatically by ``tools/source_translator.py``.",
                "",
                "**To modify TrustOS source code, edit the ORIGINAL at ``kernel/src/``**",
                "",
                "Any manual changes here will be OVERWRITTEN on the next translation run.",
                "Generated: $markerDate"
            ) -join "`n"
            $markerContent | Set-Content $markerPath -Encoding utf8
        }

        $rsCount = (Get-ChildItem -Recurse -Filter "*.rs" -Path $outDir -ErrorAction SilentlyContinue).Count
        Write-Host "      OK -- $rsCount .rs files" -ForegroundColor Green
        $generated++
    }

    Write-Host "  [translate] $generated/$($versions.Count) versions generated" -ForegroundColor Cyan
    return ($generated -eq $versions.Count)
}

function New-TranslatedCargoToml {
    param([string]$VersionRoot)
    $lines = @(
        '[workspace]'
        'resolver = "2"'
        'members = ["kernel"]'
        ''
        '[workspace.package]'
        'version = "0.2.0"'
        'edition = "2021"'
        'authors = ["Nated0ge <nathan@trustos.dev>"]'
        'license = "MIT"'
        ''
        '[profile.dev]'
        'panic = "abort"'
        ''
        '[profile.release]'
        'panic = "abort"'
        'lto = true'
        'opt-level = "s"'
    )

    $origContent = Get-Content (Join-Path $Root "Cargo.toml") -Raw
    if ($origContent -match "mbedtls-platform-support") {
        $cratesSrc = Join-Path $Root "crates"
        $cratesDst = Join-Path $VersionRoot "crates"
        if (Test-Path (Join-Path $cratesSrc "mbedtls-platform-support")) {
            if (-not (Test-Path $cratesDst)) {
                New-Item -ItemType Directory -Path $cratesDst -Force | Out-Null
                Copy-Item -Recurse (Join-Path $cratesSrc "mbedtls-platform-support") (Join-Path $cratesDst "mbedtls-platform-support")
            }
            $lines += ''
            $lines += '[patch.crates-io]'
            $lines += 'mbedtls-platform-support = { path = "crates/mbedtls-platform-support" }'
        }
    }

    $lines -join "`n" | Set-Content (Join-Path $VersionRoot "Cargo.toml") -Encoding utf8 -NoNewline
}

# ================================================================
#  CORE: Build ISO for translated versions
# ================================================================

function Build-TranslatedISO {
    param([string]$VersionName)

    $versionRoot = Join-Path $TranslatedRoot $VersionName
    if (-not (Test-Path (Join-Path $versionRoot "kernel\src"))) {
        Write-Host "    [$VersionName] No sources found -- skip" -ForegroundColor Yellow
        return $false
    }

    Write-Host "    [$VersionName] Building..." -ForegroundColor Yellow
    Initialize-BuildEnv
    Push-Location $versionRoot

    try {
        # Ensure limine/ is accessible for include_bytes! macros
        $limineDst = Join-Path $versionRoot "limine"
        if (-not (Test-Path $limineDst)) {
            # Create junction to root limine directory
            & cmd /c mklink /J "$limineDst" "$LimineDir" 2>&1 | Out-Null
        }

        $mbedtlsInclude = Join-Path $versionRoot "kernel\mbedtls-include"
        if (Test-Path $mbedtlsInclude) {
            $env:CFLAGS = "-I""$mbedtlsInclude"" -mcmodel=kernel -mno-red-zone -ffreestanding"
            $env:BINDGEN_EXTRA_CLANG_ARGS = "-I""$mbedtlsInclude"""
            $env:C_INCLUDE_PATH = $mbedtlsInclude
            $env:CPLUS_INCLUDE_PATH = $mbedtlsInclude
        }

        $ErrorActionPreference = "Continue"
        $buildOutput = cargo build --release -p trustos_kernel 2>&1
        $buildExit = $LASTEXITCODE
        $ErrorActionPreference = "Stop"

        $buildOutput | Out-File -FilePath (Join-Path $versionRoot "build.log") -Encoding utf8

        if ($buildExit -ne 0) {
            $errors = $buildOutput | Select-String "error\[E" | Select-Object -First 3
            foreach ($e in $errors) { Write-Host "      $($e.Line)" -ForegroundColor Red }
            Write-Host "    [$VersionName] FAILED -- see translated\$VersionName\build.log" -ForegroundColor Red
            return $false
        }

        # Create ISO for this version
        $kernelBin = Join-Path $versionRoot "target\x86_64-unknown-none\release\trustos_kernel"
        if (-not (Test-Path $kernelBin)) {
            Write-Host "    [$VersionName] Kernel binary not found" -ForegroundColor Red
            return $false
        }

        $sz = [math]::Round((Get-Item $kernelBin).Length / 1MB, 2)

        # Build ISO for this translated version
        $isoDir = Join-Path $versionRoot "iso_root"
        if (Test-Path $isoDir) { Remove-Item -Recurse -Force $isoDir }
        New-Item -ItemType Directory -Path (Join-Path $isoDir "boot\limine") -Force | Out-Null
        New-Item -ItemType Directory -Path (Join-Path $isoDir "EFI\BOOT") -Force | Out-Null

        Copy-Item $kernelBin (Join-Path $isoDir "boot\trustos_kernel")

        # Limine config (base, no jarvis module)
        $limineLines = (Get-Content (Join-Path $Root "limine.conf")) | Where-Object { $_ -notmatch "module_path|module_cmdline" }
        $limineLines | Set-Content (Join-Path $isoDir "boot\limine\limine.conf")
        $limineLines | Set-Content (Join-Path $isoDir "limine.conf")
        $limineLines | Set-Content (Join-Path $isoDir "boot\limine\limine.cfg")

        foreach ($f in @("limine-bios.sys", "limine-bios-cd.bin", "limine-uefi-cd.bin")) {
            $src = Join-Path $LimineDir $f
            if (Test-Path $src) { Copy-Item $src (Join-Path $isoDir "boot\limine") }
        }
        foreach ($f in @("BOOTX64.EFI", "BOOTIA32.EFI")) {
            $src = Join-Path $LimineDir $f
            if (Test-Path $src) { Copy-Item $src (Join-Path $isoDir "EFI\BOOT") }
        }

        $isoPath = Join-Path $BuildsDir "trustos-$VersionName.iso"

        $ErrorActionPreference = "Continue"
        $xorriso = Get-Command xorriso -ErrorAction SilentlyContinue
        if (-not $xorriso) {
            $full = [System.IO.Path]::GetFullPath($isoDir)
            $drive = $full.Substring(0, 1).ToLower()
            $rest = $full.Substring(2) -replace "\\", "/"
            $wslIsoDir = "/mnt/$drive$rest"

            $full2 = [System.IO.Path]::GetFullPath($isoPath)
            $drive2 = $full2.Substring(0, 1).ToLower()
            $rest2 = $full2.Substring(2) -replace "\\", "/"
            $wslIsoPath = "/mnt/$drive2$rest2"

            wsl -e xorriso -as mkisofs -R -r -J `
                -b boot/limine/limine-bios-cd.bin -no-emul-boot -boot-load-size 4 -boot-info-table `
                --efi-boot boot/limine/limine-uefi-cd.bin -efi-boot-part --efi-boot-image `
                --protective-msdos-label --mbr-force-bootable `
                $wslIsoDir -o $wslIsoPath 2>&1 | Out-Null
        }
        else {
            xorriso -as mkisofs -R -r -J `
                -b boot/limine/limine-bios-cd.bin -no-emul-boot -boot-load-size 4 -boot-info-table `
                --efi-boot boot/limine/limine-uefi-cd.bin -efi-boot-part --efi-boot-image `
                --protective-msdos-label --mbr-force-bootable `
                $isoDir -o $isoPath 2>&1 | Out-Null
        }
        $ErrorActionPreference = "Stop"

        # Limine BIOS install
        $limineExe = Join-Path $LimineDir "limine.exe"
        if (Test-Path $limineExe) {
            $ErrorActionPreference = "Continue"
            & $limineExe bios-install $isoPath 2>&1 | Out-Null
            $ErrorActionPreference = "Stop"
        }

        # MBR patch
        if (Test-Path $isoPath) {
            $isoBytes = [System.IO.File]::ReadAllBytes($isoPath)
            if ($isoBytes.Length -gt 446 -and $isoBytes[446] -ne 0x80) {
                $isoBytes[446] = 0x80
                [System.IO.File]::WriteAllBytes($isoPath, $isoBytes)
            }
        }

        # Cleanup temp iso_root inside translated dir
        if (Test-Path $isoDir) { Remove-Item -Recurse -Force $isoDir }

        if (Test-Path $isoPath) {
            $isoSz = [math]::Round((Get-Item $isoPath).Length / 1MB, 2)
            Write-Host "    [$VersionName] OK -- kernel: $sz MB, ISO: $isoSz MB" -ForegroundColor Green
            return $true
        }
        else {
            Write-Host "    [$VersionName] ISO creation failed" -ForegroundColor Red
            return $false
        }
    }
    finally { Pop-Location }
}

# ================================================================
#  COMMAND: build
# ================================================================

function Invoke-Build {
    Write-Host ""
    Write-Host "=== TrustOS Build ===" -ForegroundColor Cyan
    Write-Host "  Edition: $Edition" -ForegroundColor DarkGray
    Write-Host "  Source:  kernel\src\ (ORIGINAL)" -ForegroundColor DarkGray
    Write-Host ""

    # Step 1: Build kernel
    $ok = Build-Kernel -CleanFirst:$Clean
    if (-not $ok) { exit 1 }

    # Step 2: Create ISO
    $brainFile = ""
    if ($Edition -eq "jarvispack") {
        $brainLocations = @(
            (Join-Path $BuildsDir "trustos-jarvispack\jarvis_pretrained.bin"),
            "jarvis-bench\jarvis_pretrained.bin"
        )
        foreach ($bp in $brainLocations) {
            if (Test-Path $bp) { $brainFile = $bp; break }
        }
    }

    $ok = New-TrustOSIso -EditionName $Edition -BrainFile $brainFile
    if (-not $ok) { exit 1 }

    Write-Host ""
    Write-Host "  Build complete!" -ForegroundColor Green

    # Step 3: Launch VBox (unless -NoRun)
    if (-not $NoRun) {
        Write-Host ""
        Start-VBoxTest
    }
}

# ================================================================
#  COMMAND: release
# ================================================================

function Invoke-Release {
    Write-Host ""
    Write-Host "=== TrustOS Release Pipeline ===" -ForegroundColor Magenta
    Write-Host ""

    # Determine version tag
    if (-not $Tag) {
        $existingTags = git tag --sort=-v:refname 2>$null | Select-Object -First 1
        if ($existingTags) {
            Write-Host "  Latest tag: $existingTags" -ForegroundColor DarkGray
        }
        $Tag = Read-Host "  Enter release tag (e.g. v0.3.0)"
        if (-not $Tag) { Write-Host "  No tag specified, aborting." -ForegroundColor Red; return }
    }

    $releaseMsg = if ($Message) { $Message } else { "TrustOS $Tag release" }

    Write-Host ""
    Write-Host "  Tag:     $Tag" -ForegroundColor Cyan
    Write-Host "  Message: $releaseMsg" -ForegroundColor Cyan
    Write-Host ""

    # ── Step 1: Build original kernel + ISO ──
    Write-Host "[1/6] Building original kernel..." -ForegroundColor Yellow
    $ok = Build-Kernel -CleanFirst:$Clean
    if (-not $ok) { Write-Host "  Build failed -- aborting release" -ForegroundColor Red; exit 1 }

    Write-Host "[2/6] Creating base ISO..." -ForegroundColor Yellow
    $ok = New-TrustOSIso -EditionName "base"
    if (-not $ok) { Write-Host "  ISO failed -- aborting release" -ForegroundColor Red; exit 1 }

    # JarvisPack ISO
    $brainFile = ""
    $brainLocations = @(
        (Join-Path $BuildsDir "trustos-jarvispack\jarvis_pretrained.bin"),
        "jarvis-bench\jarvis_pretrained.bin"
    )
    foreach ($bp in $brainLocations) {
        if (Test-Path $bp) { $brainFile = $bp; break }
    }
    if ($brainFile) {
        Write-Host "  Creating JarvisPack ISO..." -ForegroundColor Yellow
        New-TrustOSIso -EditionName "jarvispack" -BrainFile $brainFile | Out-Null
    }

    # ── Step 2: Translate all versions ──
    if (-not $NoTranslate) {
        Write-Host "[3/6] Translating source to all versions..." -ForegroundColor Yellow
        $ok = Invoke-Translate -OnlyVersion $Only -VerboseOutput:$DetailedOutput
        if (-not $ok) {
            Write-Host "  Translation had errors -- continuing anyway" -ForegroundColor Yellow
        }

        # ── Step 3: Build ISO for each translated version ──
        Write-Host "[4/6] Building ISOs for translated versions..." -ForegroundColor Yellow
        $translatedVersions = @("original", "minimal", "educational-en", "educational-fr")
        if ($Only) { $translatedVersions = @($Only) }

        foreach ($ver in $translatedVersions) {
            Build-TranslatedISO -VersionName $ver | Out-Null
        }
    }
    else {
        Write-Host "[3/6] Skipped (--NoTranslate)" -ForegroundColor DarkGray
        Write-Host "[4/6] Skipped (--NoTranslate)" -ForegroundColor DarkGray
    }

    # ── Step 4: Git commit + tag ──
    Write-Host "[5/6] Git commit and tag..." -ForegroundColor Yellow
    Push-Location $Root
    $oldEAP = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    try {
        git add -A 2>&1 | Out-Null
        $status = git status --porcelain 2>&1
        if ($status) {
            git commit -m $releaseMsg 2>&1 | ForEach-Object { Write-Host "    $_" -ForegroundColor DarkGray }
        }
        else {
            Write-Host "    No changes to commit" -ForegroundColor DarkGray
        }

        # Create tag
        git tag -a $Tag -m $releaseMsg 2>&1 | ForEach-Object { Write-Host "    $_" -ForegroundColor DarkGray }
    }
    finally {
        $ErrorActionPreference = $oldEAP
        Pop-Location
    }

    # ── Step 5: Push ──
    if (-not $NoPush) {
        Write-Host "[6/6] Pushing to remote..." -ForegroundColor Yellow
        Push-Location $Root
        $oldEAP = $ErrorActionPreference
        $ErrorActionPreference = "Continue"
        try {
            git push 2>&1 | ForEach-Object { Write-Host "    $_" -ForegroundColor DarkGray }
            git push --tags 2>&1 | ForEach-Object { Write-Host "    $_" -ForegroundColor DarkGray }
        }
        finally {
            $ErrorActionPreference = $oldEAP
            Pop-Location
        }
        Write-Host "  Pushed! GitHub Actions will build the release ISO." -ForegroundColor Green
    }
    else {
        Write-Host "[6/6] Skipped (--NoPush)" -ForegroundColor DarkGray
    }

    # ── Summary ──
    Write-Host ""
    Write-Host "=== Release Summary ===" -ForegroundColor Magenta
    Write-Host "  Tag:        $Tag" -ForegroundColor Cyan
    Write-Host "  Commit:     $(git rev-parse --short HEAD 2>$null)" -ForegroundColor Cyan

    $allIsos = Get-ChildItem -Path $BuildsDir -Filter "*.iso" -Recurse -ErrorAction SilentlyContinue
    foreach ($iso in $allIsos) {
        $relPath = $iso.FullName.Replace($Root, "").TrimStart("\")
        $sz = [math]::Round($iso.Length / 1MB, 2)
        Write-Host "  ISO:        $relPath ($sz MB)" -ForegroundColor Cyan
    }
    # Also check root-level ISOs
    $rootIso = Join-Path $Root "trustos.iso"
    if (Test-Path $rootIso) {
        $sz = [math]::Round((Get-Item $rootIso).Length / 1MB, 2)
        Write-Host "  ISO (root): trustos.iso ($sz MB)" -ForegroundColor Cyan
    }

    Write-Host ""
    Write-Host "  Release pipeline complete!" -ForegroundColor Green
}

# ================================================================
#  COMMAND: clean
# ================================================================

function Invoke-Clean {
    Write-Host ""
    Write-Host "=== TrustOS Clean ===" -ForegroundColor Yellow

    Push-Location $Root
    try {
        Write-Host "  Cleaning cargo build..." -ForegroundColor Yellow
        cargo clean 2>$null

        if (Test-Path "iso_root") {
            Remove-Item -Recurse -Force "iso_root"
            Write-Host "  Removed iso_root/" -ForegroundColor DarkGray
        }

        # Clean translated build artifacts (not the sources themselves)
        foreach ($ver in @("original", "minimal", "educational-en", "educational-fr")) {
            $targetDir = Join-Path $TranslatedRoot "$ver\target"
            $isoRoot = Join-Path $TranslatedRoot "$ver\iso_root"
            if (Test-Path $targetDir) {
                Remove-Item -Recurse -Force $targetDir
                Write-Host "  Removed translated\$ver\target\" -ForegroundColor DarkGray
            }
            if (Test-Path $isoRoot) {
                Remove-Item -Recurse -Force $isoRoot
                Write-Host "  Removed translated\$ver\iso_root\" -ForegroundColor DarkGray
            }
        }

        Write-Host "  Clean complete." -ForegroundColor Green
    }
    finally { Pop-Location }
}

# ================================================================
#  COMMAND: status
# ================================================================

function Invoke-Status {
    Write-Host ""
    Write-Host "=== TrustOS Project Status ===" -ForegroundColor Cyan
    Write-Host ""

    # Source
    $rsFiles = (Get-ChildItem -Recurse -Filter "*.rs" -Path $KernelSrc -ErrorAction SilentlyContinue).Count
    Write-Host "  Source (ORIGINAL):  kernel\src\ -- $rsFiles .rs files" -ForegroundColor Green

    # Translated versions
    foreach ($ver in @("original", "minimal", "educational-en", "educational-fr")) {
        $verSrc = Join-Path $TranslatedRoot "$ver\kernel\src"
        if (Test-Path $verSrc) {
            $count = (Get-ChildItem -Recurse -Filter "*.rs" -Path $verSrc -ErrorAction SilentlyContinue).Count
            $mapping = Join-Path $TranslatedRoot "$ver\mapping.json"
            $mapAge = if (Test-Path $mapping) { "updated $(((Get-Item $mapping).LastWriteTime).ToString('yyyy-MM-dd HH:mm'))" } else { "no mapping" }
            Write-Host "  Translated ($ver): $count .rs files ($mapAge)" -ForegroundColor DarkGray
        }
        else {
            Write-Host "  Translated ($ver): NOT GENERATED" -ForegroundColor Yellow
        }
    }

    # ISOs
    Write-Host ""
    $allIsos = Get-ChildItem -Path $BuildsDir -Filter "*.iso" -Recurse -ErrorAction SilentlyContinue
    $rootIso = Join-Path $Root "trustos.iso"
    if (Test-Path $rootIso) {
        $sz = [math]::Round((Get-Item $rootIso).Length / 1MB, 2)
        $age = ((Get-Item $rootIso).LastWriteTime).ToString('yyyy-MM-dd HH:mm')
        Write-Host "  ISO (root):  trustos.iso -- $sz MB ($age)" -ForegroundColor Cyan
    }
    foreach ($iso in $allIsos) {
        $relPath = $iso.FullName.Replace($Root, "").TrimStart("\")
        $sz = [math]::Round($iso.Length / 1MB, 2)
        $age = $iso.LastWriteTime.ToString('yyyy-MM-dd HH:mm')
        Write-Host "  ISO: $relPath -- $sz MB ($age)" -ForegroundColor Cyan
    }

    # Git
    Write-Host ""
    Push-Location $Root
    $branch = git branch --show-current 2>$null
    $commit = git rev-parse --short HEAD 2>$null
    $lastTag = git describe --tags --abbrev=0 2>$null
    $dirty = git status --porcelain 2>$null
    $dirtyCount = ($dirty | Measure-Object).Count
    Pop-Location
    Write-Host "  Git branch: $branch ($commit)" -ForegroundColor DarkGray
    if ($lastTag) { Write-Host "  Last tag:   $lastTag" -ForegroundColor DarkGray }
    if ($dirtyCount -gt 0) {
        Write-Host "  Uncommitted: $dirtyCount files" -ForegroundColor Yellow
    }
    else {
        Write-Host "  Working tree: clean" -ForegroundColor Green
    }

    Write-Host ""
}

# ================================================================
#  COMMAND: help
# ================================================================

function Show-Help {
    Write-Host ""
    Write-Host "===================================================" -ForegroundColor Cyan
    Write-Host "  TrustOS -- Unified Build Shell" -ForegroundColor Cyan
    Write-Host "===================================================" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "  IMPORTANT: Always edit kernel\src\ (the ORIGINAL source)" -ForegroundColor Yellow
    Write-Host "  Never edit translated\ -- it is AUTO-GENERATED" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "  Commands:" -ForegroundColor White
    Write-Host "    build      Build kernel + ISO, launch VBox for testing" -ForegroundColor Green
    Write-Host "    release    Full release: build all ISOs, translate, git push" -ForegroundColor Magenta
    Write-Host "    translate  Generate translated source versions" -ForegroundColor Yellow
    Write-Host "    clean      Remove all build artifacts" -ForegroundColor Red
    Write-Host "    status     Show project status" -ForegroundColor Cyan
    Write-Host "    help       Show this help" -ForegroundColor DarkGray
    Write-Host ""
    Write-Host "  Build options:" -ForegroundColor White
    Write-Host "    .\trustos.ps1 build                      # Build + test in VBox" -ForegroundColor DarkGray
    Write-Host "    .\trustos.ps1 build -NoRun                # Build only" -ForegroundColor DarkGray
    Write-Host "    .\trustos.ps1 build -Clean                # Clean build" -ForegroundColor DarkGray
    Write-Host "    .\trustos.ps1 build -Edition jarvispack   # JarvisPack edition" -ForegroundColor DarkGray
    Write-Host ""
    Write-Host "  Release options:" -ForegroundColor White
    Write-Host "    .\trustos.ps1 release                     # Interactive (asks for tag)" -ForegroundColor DarkGray
    Write-Host "    .\trustos.ps1 release -Tag v0.3.0         # Specific tag" -ForegroundColor DarkGray
    Write-Host "    .\trustos.ps1 release -NoPush             # Don't push to remote" -ForegroundColor DarkGray
    Write-Host "    .\trustos.ps1 release -NoTranslate        # Skip translation step" -ForegroundColor DarkGray
    Write-Host ""
    Write-Host "  Translate options:" -ForegroundColor White
    Write-Host "    .\trustos.ps1 translate                   # All versions" -ForegroundColor DarkGray
    Write-Host "    .\trustos.ps1 translate -Only minimal     # One version only" -ForegroundColor DarkGray
    Write-Host "    .\trustos.ps1 translate -DetailedOutput   # Detailed output" -ForegroundColor DarkGray
    Write-Host ""
    Write-Host "  Folder Structure:" -ForegroundColor White
    Write-Host "    kernel/src/      THE source code (edit HERE)" -ForegroundColor Green
    Write-Host "    translated/      Auto-generated versions (DO NOT EDIT)" -ForegroundColor Red
    Write-Host "    builds/          ISOs and build outputs" -ForegroundColor Cyan
    Write-Host "    logs/            All logs (serial, build, crash, qemu)" -ForegroundColor DarkGray
    Write-Host "    scripts/         Build/test/launch/debug scripts" -ForegroundColor DarkGray
    Write-Host "    tools/           Source translator, analysis tools" -ForegroundColor DarkGray
    Write-Host "    docs/            Documentation" -ForegroundColor DarkGray
    Write-Host "    media/           Logo, screenshots" -ForegroundColor DarkGray
    Write-Host "    firmware/        OVMF, UEFI vars" -ForegroundColor DarkGray
    Write-Host ""
}

# ================================================================
#  DISPATCH
# ================================================================

switch ($Command) {
    "build"     { Invoke-Build }
    "release"   { Invoke-Release }
    "translate" {
        Write-Host ""
        Write-Host "=== TrustOS Translate ===" -ForegroundColor Yellow
        Invoke-Translate -OnlyVersion $Only -VerboseOutput:$DetailedOutput
    }
    "clean"     { Invoke-Clean }
    "status"    { Invoke-Status }
    "help"      { Show-Help }
}
