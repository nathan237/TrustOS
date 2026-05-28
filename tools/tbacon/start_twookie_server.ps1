# start_twookie_server.ps1
# Wrapper serveurlapointetech — chemins adaptes au serveur.
# Appele au demarrage via Task Scheduler (compte Serveur, RunLevel Highest).

param(
  [int]$Port = 8895,
  [string]$PublicUrl = "integrative-janay-unaxed.ngrok-free.dev",
  [switch]$NoStartServer
)

$ErrorActionPreference = "Stop"

$NativeRoot = "D:\TrustOS_Dev\twookie-native"
$LogDir     = "D:\TrustOS_Dev\twookie-native\logs"
$DataDir    = "D:\TrustOS_Dev\twookie-native\data"
$ReportsDir = "D:\TrustOS_Dev\TrustOS\reports\tbacon\latest"

New-Item -ItemType Directory -Force -Path $LogDir    | Out-Null
New-Item -ItemType Directory -Force -Path $DataDir   | Out-Null
New-Item -ItemType Directory -Force -Path $ReportsDir | Out-Null

$Ngrok = "C:\ProgramData\ngrok\ngrok.exe"
if (-not (Test-Path $Ngrok)) { throw "ngrok.exe introuvable : $Ngrok" }
$Python = Join-Path $NativeRoot ".venv\Scripts\python.exe"
if (-not (Test-Path $Python)) { throw "Venv absent : $Python" }

# --- Demarrer le backend ---
if (-not $NoStartServer) {
  $running = Get-CimInstance Win32_Process | Where-Object {
    $_.Name -eq "python.exe" -and $_.CommandLine -like "*uvicorn*main:app*"
  }
  foreach ($p in $running) { Stop-Process -Id $p.ProcessId -Force -ErrorAction SilentlyContinue }

  $env:TWOOKIE_DB = "$DataDir\twookie.sqlite3"
  $env:PORT       = "$Port"

  $ApnsEnv = "D:\TrustOS_Dev\twookie-native\backend\.env"
  if (Test-Path $ApnsEnv) {
    foreach ($line in Get-Content -LiteralPath $ApnsEnv) {
      if ($line -match '^\s*([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(.*)\s*$') {
        [Environment]::SetEnvironmentVariable($Matches[1], $Matches[2].Trim().Trim('"').Trim("'"), "Process")
      }
    }
  }
  # Charger le contenu du .p8 dans APNS_KEY_PEM (le backend lit le PEM, pas le chemin)
  $keyPath = $env:APNS_KEY_PATH
  if ($keyPath -and (Test-Path $keyPath)) {
    $env:APNS_KEY_PEM = Get-Content -Raw -LiteralPath $keyPath
  }

  $OutLog = "$LogDir\twookie_stdout.log"
  $ErrLog = "$LogDir\twookie_stderr.log"
  $Stamp  = Get-Date -Format "yyyyMMdd_HHmmss"
  foreach ($log in @($OutLog, $ErrLog)) {
    if ((Test-Path $log) -and (Get-Item $log).Length -gt 0) {
      $arch = $log -replace '\.log$', "_$Stamp.log"
      Move-Item -LiteralPath $log -Destination $arch
    }
  }

  Start-Process -FilePath $Python -WindowStyle Hidden `
    -WorkingDirectory "$NativeRoot\backend" `
    -RedirectStandardOutput $OutLog -RedirectStandardError $ErrLog `
    -ArgumentList @("-m","uvicorn","main:app","--host","127.0.0.1","--port","$Port","--workers","1","--no-access-log","--log-level","warning")

  Start-Sleep -Seconds 3
  $status = (Invoke-WebRequest -UseBasicParsing "http://127.0.0.1:$Port/health").StatusCode
  if ($status -ne 200) { throw "Backend ne repond pas sur le port $Port" }
}

# --- Demarrer ngrok ---
$ngrokRunning = Get-CimInstance Win32_Process |
  Where-Object { $_.Name -eq "ngrok.exe" -and $_.CommandLine -like "* http *$Port*" }
foreach ($p in $ngrokRunning) { Stop-Process -Id $p.ProcessId -Force -ErrorAction SilentlyContinue }

$ngrokArgs = @("http", "--url=$PublicUrl", "http://127.0.0.1:$Port", "--inspect=false", "--config=C:\ProgramData\ngrok\ngrok.yml")
Start-Process -FilePath $Ngrok -WindowStyle Hidden -ArgumentList $ngrokArgs

$publicUrl = ""
for ($i = 0; $i -lt 30; $i++) {
  Start-Sleep -Milliseconds 500
  try {
    $tunnels = Invoke-RestMethod -UseBasicParsing "http://127.0.0.1:4040/api/tunnels"
    $https = $tunnels.tunnels | Where-Object { $_.proto -eq "https" } | Select-Object -First 1
    if ($https.public_url) { $publicUrl = $https.public_url; break }
  } catch {}
}

if (-not $publicUrl) { throw "ngrok demarre mais aucune URL publique detectee." }

$session = [ordered]@{
  created_at              = (Get-Date).ToString("o")
  local_url               = "http://127.0.0.1:$Port/"
  public_url              = "$publicUrl/"
  health_url              = "$publicUrl/health"
  runtime                 = "twookie-native FastAPI"
  ngrok_inspector         = "http://127.0.0.1:4040"
  ngrok_inspection_enabled = $false
}
$session | ConvertTo-Json -Depth 5 | Set-Content -Path "$ReportsDir\twookie_ngrok_session.json" -Encoding UTF8

"T-Wookie public: $($session.public_url)"
"Health:          $($session.health_url)"
"Session saved:   $ReportsDir\twookie_ngrok_session.json"
