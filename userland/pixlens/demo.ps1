# Pixlens Demo Script
# Génère des fichiers de test et analyse avec différents mappings
# Author: Nathan (nated0ge)

$pixlens = "C:\Users\nathan\Documents\Scripts\OSrust\userland\pixlens\target\x86_64-pc-windows-msvc\release\pixlens.exe"
$outDir = "C:\temp\pixlens_demo"

Write-Host "=== PIXLENS DEMO ===" -ForegroundColor Cyan
Write-Host "Creating test files and analyzing..." -ForegroundColor Yellow

# Create output directory
New-Item -ItemType Directory -Path $outDir -Force | Out-Null

# ════════════════════════════════════════════════════════════════════════════
# 1. Generate test files
# ════════════════════════════════════════════════════════════════════════════

Write-Host "`n[1/6] Generating random data (good encryption simulation)..." -ForegroundColor Green
$random = New-Object byte[] 65536
(New-Object Random).NextBytes($random)
[IO.File]::WriteAllBytes("$outDir\random.bin", $random)

Write-Host "[2/6] Generating XOR-encrypted data (weak crypto)..." -ForegroundColor Green
# Text XORed with 0x42 - should reveal pattern with xor mapping
$text = "This is a secret message that was encrypted with a simple XOR cipher. " * 100
$textBytes = [System.Text.Encoding]::ASCII.GetBytes($text)
$xored = $textBytes | ForEach-Object { $_ -bxor 0x42 }
[IO.File]::WriteAllBytes("$outDir\xor_weak.bin", [byte[]]$xored)

Write-Host "[3/6] Generating ECB-like pattern (block repetition)..." -ForegroundColor Green
# Same 16-byte block repeated - simulates ECB mode weakness
$block = [byte[]](0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 
                   0x49, 0x4A, 0x4B, 0x4C, 0x4D, 0x4E, 0x4F, 0x50)
$ecb = New-Object byte[] 65536
for ($i = 0; $i -lt 65536; $i++) { $ecb[$i] = $block[$i % 16] }
[IO.File]::WriteAllBytes("$outDir\ecb_pattern.bin", $ecb)

Write-Host "[4/6] Generating mixed content (headers + random)..." -ForegroundColor Green
# Simulates real encrypted file: plaintext header + encrypted body
$header = [System.Text.Encoding]::ASCII.GetBytes("ENCRYPTED_FILE_V1.0`n" + ("X" * 100))
$body = New-Object byte[] 60000
(New-Object Random).NextBytes($body)
$mixed = $header + $body
[IO.File]::WriteAllBytes("$outDir\mixed.bin", [byte[]]$mixed)

Write-Host "[5/6] Generating gradient (low entropy)..." -ForegroundColor Green
$gradient = New-Object byte[] 65536
for ($i = 0; $i -lt 65536; $i++) { $gradient[$i] = $i % 256 }
[IO.File]::WriteAllBytes("$outDir\gradient.bin", $gradient)

Write-Host "[6/6] Generating null-padded data..." -ForegroundColor Green
$padded = New-Object byte[] 65536
$payload = [System.Text.Encoding]::ASCII.GetBytes("SECRET_DATA_HERE")
for ($i = 0; $i -lt $payload.Length; $i++) { $padded[1000 + $i] = $payload[$i] }
for ($i = 0; $i -lt $payload.Length; $i++) { $padded[30000 + $i] = $payload[$i] }
[IO.File]::WriteAllBytes("$outDir\null_padded.bin", $padded)

# ════════════════════════════════════════════════════════════════════════════
# 2. Run analysis on each file
# ════════════════════════════════════════════════════════════════════════════

$testFiles = @(
    @{name="random"; desc="Random (good crypto simulation)"},
    @{name="xor_weak"; desc="XOR weak encryption"},
    @{name="ecb_pattern"; desc="ECB mode (block repetition)"},
    @{name="mixed"; desc="Mixed header+encrypted"},
    @{name="gradient"; desc="Low entropy gradient"},
    @{name="null_padded"; desc="Null-padded data"}
)

Write-Host "`n=== ANALYZING FILES ===" -ForegroundColor Cyan

foreach ($file in $testFiles) {
    Write-Host "`nAnalyzing $($file.desc)..." -ForegroundColor Yellow
    
    # Key mappings for crypto analysis
    & $pixlens -i "$outDir\$($file.name).bin" -o "$outDir\$($file.name)_digraph.ppm" -m digraph
    & $pixlens -i "$outDir\$($file.name).bin" -o "$outDir\$($file.name)_entropy.ppm" -m entropy -p 32
    & $pixlens -i "$outDir\$($file.name).bin" -o "$outDir\$($file.name)_linear.ppm" -m linear -w 256
    & $pixlens -i "$outDir\$($file.name).bin" -o "$outDir\$($file.name)_modulo16.ppm" -m modulo -p 16
}

# ════════════════════════════════════════════════════════════════════════════
# 3. Also analyze the TrustOS kernel for comparison
# ════════════════════════════════════════════════════════════════════════════

$kernel = "C:\Users\nathan\Documents\Scripts\OSrust\target\x86_64-unknown-none\release\trustos_kernel"
if (Test-Path $kernel) {
    Write-Host "`nAnalyzing TrustOS kernel..." -ForegroundColor Yellow
    & $pixlens -i $kernel -o "$outDir\kernel_digraph.ppm" -m digraph
    & $pixlens -i $kernel -o "$outDir\kernel_entropy.ppm" -m entropy -p 64
    & $pixlens -i $kernel -o "$outDir\kernel_highent.ppm" -m highent -p 32
}

# ════════════════════════════════════════════════════════════════════════════
# 4. Summary
# ════════════════════════════════════════════════════════════════════════════

Write-Host "`n=== ANALYSIS COMPLETE ===" -ForegroundColor Cyan
Write-Host "Results saved to: $outDir" -ForegroundColor Green
Write-Host "`nFiles generated:" -ForegroundColor Yellow
Get-ChildItem $outDir\*.ppm | ForEach-Object { 
    Write-Host "  $($_.Name)" -ForegroundColor White 
}

Write-Host "`n=== HOW TO INTERPRET ===" -ForegroundColor Cyan
Write-Host @"

DIGRAPH (256x256):
  - Random/good crypto = uniform noise, no patterns
  - Weak crypto = visible lines, clusters, or patterns
  - XOR = diagonal lines
  - Text = clusters in printable ASCII range

ENTROPY (colored):
  - RED = high entropy (encrypted/compressed)
  - ORANGE = medium-high (compressed?)
  - YELLOW = medium (code)
  - GREEN = low (text)
  - BLUE = very low (padding/nulls)

MODULO-16:
  - Reveals 16-byte block patterns (AES block size)
  - Vertical lines = repeating blocks (ECB weakness!)

LINEAR:
  - Raw visual of the file
  - Headers, sections visible

"@ -ForegroundColor Gray

Write-Host "Opening folder..." -ForegroundColor Yellow
Start-Process explorer.exe $outDir
