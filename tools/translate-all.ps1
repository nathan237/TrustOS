param(
    [switch]$Build,
    [switch]$Clean,
    [string]$Only = "",
    [switch]$Verbose
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$translator = Join-Path $PSScriptRoot "source_translator.py"
$translatedRoot = Join-Path $root "translated"
$kernelSrc = Join-Path $root "kernel\src"

function New-StandaloneCargoToml {
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
    $origContent = Get-Content (Join-Path $root "Cargo.toml") -Raw
    if ($origContent -match "mbedtls-platform-support") {
        $cratesSrc = Join-Path $root "crates"
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

$versions = @(
    @{ Name = "original";       Preset = "original";     Lang = "en"; Desc = "Exact copy of original source" }
    @{ Name = "minimal";        Preset = "minimal";      Lang = "en"; Desc = "Shortest identifiers, no comments" }
    @{ Name = "educational-en"; Preset = "educational";  Lang = "en"; Desc = "Expanded names + English annotations" }
    @{ Name = "educational-fr"; Preset = "educational";  Lang = "fr"; Desc = "Expanded names + French annotations" }
)

if ($Only) {
    $versions = $versions | Where-Object { $_.Name -eq $Only }
    if (-not $versions) {
        Write-Host "Unknown version: $Only" -ForegroundColor Red
        Write-Host "Available: original, minimal, educational-en, educational-fr"
        exit 1
    }
}

Write-Host ""
Write-Host "=== TrustOS Source Translator ===" -ForegroundColor Cyan
Write-Host "    Kernel source : $kernelSrc" -ForegroundColor DarkGray
Write-Host "    Output root   : $translatedRoot" -ForegroundColor DarkGray
Write-Host ""

if ($Clean -and (Test-Path $translatedRoot)) {
    Write-Host "[Clean] Removing $translatedRoot ..." -ForegroundColor Yellow
    Remove-Item -Recurse -Force $translatedRoot
}

$generated = 0
foreach ($ver in $versions) {
    $name   = $ver.Name
    $preset = $ver.Preset
    $lang   = $ver.Lang
    $desc   = $ver.Desc
    $outDir = Join-Path (Join-Path $translatedRoot $name) "kernel\src"
    $mapDir = Join-Path $translatedRoot $name

    Write-Host "[$($generated+1)/$($versions.Count)] Generating: $name" -ForegroundColor Yellow
    Write-Host "    $desc" -ForegroundColor DarkGray

    New-Item -ItemType Directory -Path $mapDir -Force | Out-Null

    $pyArgs = @(
        $translator,
        "--preset", $preset,
        "--lang", $lang,
        "-i", $kernelSrc,
        "-o", $outDir,
        "--save-mapping", (Join-Path $mapDir "mapping.json")
    )
    if ($Verbose) { $pyArgs += "--verbose" }

    $pyOutput = & python @pyArgs 2>&1
    $exitCode = $LASTEXITCODE

    foreach ($line in $pyOutput) {
        if ($line -match "error|Error|ERROR") {
            Write-Host "    $line" -ForegroundColor Red
        } elseif ($line -match "warning|Warning") {
            Write-Host "    $line" -ForegroundColor Yellow
        } else {
            Write-Host "    $line" -ForegroundColor DarkGray
        }
    }

    if ($exitCode -ne 0) {
        Write-Host "    FAILED (exit code $exitCode)" -ForegroundColor Red
        continue
    }

    $kernelRoot = Join-Path $root "kernel"
    $destKernel = Join-Path (Join-Path $translatedRoot $name) "kernel"

    Copy-Item (Join-Path $kernelRoot "Cargo.toml") (Join-Path $destKernel "Cargo.toml") -Force
    Copy-Item (Join-Path $kernelRoot "build.rs") (Join-Path $destKernel "build.rs") -Force
    foreach ($ld in (Get-ChildItem -Path $kernelRoot -Filter "linker*.ld")) {
        Copy-Item $ld.FullName (Join-Path $destKernel $ld.Name) -Force
    }

    $versionRoot = Join-Path $translatedRoot $name
    New-StandaloneCargoToml $versionRoot
    Copy-Item (Join-Path $root "rust-toolchain.toml") (Join-Path $versionRoot "rust-toolchain.toml") -Force

    $mbedInc = Join-Path $kernelRoot "mbedtls-include"
    $destMbedInc = Join-Path $destKernel "mbedtls-include"
    if ((Test-Path $mbedInc) -and -not (Test-Path $destMbedInc)) {
        Copy-Item -Recurse $mbedInc $destMbedInc
    }

    $rsCount = (Get-ChildItem -Recurse -Filter "*.rs" -Path $outDir).Count
    Write-Host "    OK - $rsCount .rs files" -ForegroundColor Green
    $generated++
}

Write-Host ""
Write-Host "=== Generation complete: $generated/$($versions.Count) versions ===" -ForegroundColor Cyan

if ($Build) {
    Write-Host ""
    Write-Host "=== Building translated versions ===" -ForegroundColor Cyan
    $built = 0

    foreach ($ver in $versions) {
        $name = $ver.Name
        $versionRoot = Join-Path $translatedRoot $name

        if (-not (Test-Path (Join-Path $versionRoot "kernel\src"))) {
            Write-Host "  [$name] Skipped - no sources" -ForegroundColor Yellow
            continue
        }

        Write-Host "  [$name] Building..." -ForegroundColor Yellow
        Push-Location $versionRoot

        try {
            $cmakeBin = "C:\Program Files\CMake\bin"
            $llvmBin = "C:\Program Files\LLVM\bin"
            if (Test-Path $cmakeBin) { $env:Path = "$cmakeBin;" + $env:Path }
            if (Test-Path $llvmBin) { $env:Path = "$llvmBin;" + $env:Path }
            $env:CC = "clang"; $env:CXX = "clang++"; $env:AR = "llvm-ar"

            $mbedtlsInclude = Join-Path $versionRoot "kernel\mbedtls-include"
            if (Test-Path $mbedtlsInclude) {
                $env:CFLAGS = "-I`"$mbedtlsInclude`" -mcmodel=kernel -mno-red-zone -ffreestanding"
                $env:BINDGEN_EXTRA_CLANG_ARGS = "-I`"$mbedtlsInclude`""
                $env:C_INCLUDE_PATH = $mbedtlsInclude
                $env:CPLUS_INCLUDE_PATH = $mbedtlsInclude
            }

            $buildOutput = cargo build --release -p trustos_kernel 2>&1
            $buildExit = $LASTEXITCODE
            $buildOutput | Out-File -FilePath (Join-Path $versionRoot "build.log") -Encoding utf8

            if ($buildExit -eq 0) {
                $kernelBin = Join-Path $versionRoot "target\x86_64-unknown-none\release\trustos_kernel"
                if (Test-Path $kernelBin) {
                    $sz = [math]::Round((Get-Item $kernelBin).Length / 1MB, 2)
                    Write-Host "  [$name] OK - kernel: $sz MB" -ForegroundColor Green
                } else {
                    Write-Host "  [$name] OK (binary not at expected path)" -ForegroundColor Green
                }
                $built++
            } else {
                $errors = $buildOutput | Select-String "error\[E" | Select-Object -First 5
                foreach ($e in $errors) {
                    Write-Host "    $($e.Line)" -ForegroundColor Red
                }
                Write-Host "  [$name] BUILD FAILED - see $versionRoot\build.log" -ForegroundColor Red
            }
        }
        catch {
            Write-Host "  [$name] ERROR: $_" -ForegroundColor Red
        }
        finally {
            Pop-Location
        }
    }

    Write-Host ""
    Write-Host "=== Build results: $built/$($versions.Count) succeeded ===" -ForegroundColor Cyan
}

Write-Host ""

