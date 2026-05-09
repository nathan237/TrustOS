param(
    [string]$Source = "C:\Users\nathan\Documents\Scripts",
    [string]$Destination = "D:\TrustOS_SafeMirror\Documents_Scripts",
    [switch]$Quiet
)

$ErrorActionPreference = "Continue"

if (-not (Test-Path -LiteralPath $Source)) {
    throw "Source not found: $Source"
}

$destRoot = Split-Path -Parent $Destination
if (-not (Test-Path -LiteralPath $destRoot)) {
    throw "Destination root not found: $destRoot"
}

$logDir = Join-Path $destRoot "_logs"
New-Item -ItemType Directory -Force -Path $logDir | Out-Null
$stamp = Get-Date -Format "yyyyMMdd_HHmmss"
$log = Join-Path $logDir "sync_safe_$stamp.log"

$args = @(
    $Source,
    $Destination,
    "/E",
    "/COPY:DAT",
    "/DCOPY:DAT",
    "/R:1",
    "/W:1",
    "/XJ",
    "/FFT",
    "/NP",
    "/LOG:$log"
)

if (-not $Quiet) {
    $args += "/TEE"
}

& robocopy @args
$code = $LASTEXITCODE

$summary = [PSCustomObject]@{
    Time = (Get-Date).ToString("o")
    Source = $Source
    Destination = $Destination
    Log = $log
    RobocopyExit = $code
    Ok = ($code -le 7)
}

$summaryPath = Join-Path $logDir "last_sync_safe.json"
$summary | ConvertTo-Json | Set-Content -LiteralPath $summaryPath -Encoding ASCII

if ($code -le 7) {
    exit 0
}
exit $code
