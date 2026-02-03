# TrustOS SDK Build Script
# Builds all examples and copies them to the kernel's ramfs

param(
    [switch]$Clean,
    [switch]$Release
)

$ErrorActionPreference = "Stop"
$SDK_ROOT = $PSScriptRoot
$EXAMPLES_DIR = Join-Path $SDK_ROOT "examples"
$OUTPUT_DIR = Join-Path $SDK_ROOT "bin"

# Create output directory
if (-not (Test-Path $OUTPUT_DIR)) {
    New-Item -ItemType Directory -Path $OUTPUT_DIR | Out-Null
}

Write-Host "=== TrustOS SDK Build ===" -ForegroundColor Cyan
Write-Host ""

# Check for Rust target
$targets = rustup target list --installed
if ($targets -notcontains "x86_64-unknown-none") {
    Write-Host "Installing x86_64-unknown-none target..." -ForegroundColor Yellow
    rustup target add x86_64-unknown-none
}

# Build mode
$buildMode = if ($Release) { "--release" } else { "" }
$targetDir = if ($Release) { "release" } else { "debug" }

# Get all example directories
$examples = Get-ChildItem -Path $EXAMPLES_DIR -Directory

foreach ($example in $examples) {
    Write-Host "Building: $($example.Name)" -ForegroundColor Green
    
    Push-Location $example.FullName
    
    try {
        if ($Clean) {
            cargo clean 2>&1 | Out-Null
        }
        
        $output = cargo build $buildMode 2>&1
        
        if ($LASTEXITCODE -ne 0) {
            Write-Host "  FAILED:" -ForegroundColor Red
            Write-Host $output
        } else {
            # Find the built binary
            $targetPath = Join-Path $example.FullName "target\x86_64-unknown-none\$targetDir\$($example.Name)"
            
            if (Test-Path $targetPath) {
                $size = (Get-Item $targetPath).Length
                Copy-Item $targetPath $OUTPUT_DIR -Force
                Write-Host "  OK ($size bytes)" -ForegroundColor Green
            } else {
                Write-Host "  Binary not found at $targetPath" -ForegroundColor Yellow
            }
        }
    }
    finally {
        Pop-Location
    }
}

Write-Host ""
Write-Host "=== Build Complete ===" -ForegroundColor Cyan
Write-Host "Binaries in: $OUTPUT_DIR" -ForegroundColor White
Write-Host ""

# List built binaries
Get-ChildItem $OUTPUT_DIR | ForEach-Object {
    Write-Host "  $($_.Name) ($($_.Length) bytes)"
}
