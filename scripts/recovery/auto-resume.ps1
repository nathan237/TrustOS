param(
    [string]$Repo = "C:\Users\nathan\Documents\Scripts\OSrust",
    [string]$TaskName = "TrustOS-AutoResume"
)

$ErrorActionPreference = "Continue"
$logDir = Join-Path $Repo ".recovery_workspace\logs"
New-Item -ItemType Directory -Force -Path $logDir | Out-Null
$log = Join-Path $logDir "resume_$(Get-Date -Format 'yyyyMMdd_HHmmss').log"

function Log([string]$Message) {
    $line = "[$(Get-Date -Format 'HH:mm:ss')] $Message"
    Add-Content -Path $log -Value $line
    Write-Host $line
}

Log "=== TrustOS auto-resume started ==="
Log "Host=$env:COMPUTERNAME User=$env:USERNAME Repo=$Repo"

Start-Sleep -Seconds 20
Start-Sleep -Seconds 15

Log "--- Disk snapshot ---"
$baselineFriendly = "Samsung SSD 970 EVO"
$disks = Get-Disk
$disks | ForEach-Object {
    Log ("Disk {0}: {1} size={2}GB bus={3} status={4}" -f $_.Number,$_.FriendlyName,[math]::Round($_.Size/1GB,1),$_.BusType,$_.OperationalStatus)
}

$newDisks = $disks | Where-Object {
    $_.OperationalStatus -eq "Online" -and $_.FriendlyName -notmatch [regex]::Escape($baselineFriendly)
}

if ($newDisks) {
    Log "[OK] Non-baseline disk(s) detected:"
    foreach ($disk in $newDisks) {
        Log ("  Disk {0}: {1}" -f $disk.Number,$disk.FriendlyName)
        Get-Partition -DiskNumber $disk.Number -ErrorAction SilentlyContinue |
            Get-Volume -ErrorAction SilentlyContinue |
            Where-Object DriveLetter |
            ForEach-Object {
                Log ("    {0}: label='{1}' free={2}GB size={3}GB" -f $_.DriveLetter,$_.FileSystemLabel,[math]::Round($_.SizeRemaining/1GB,1),[math]::Round($_.Size/1GB,1))
            }
    }
} else {
    Log "[WARN] No non-baseline disk detected. External backup target may not be attached."
}

Log "--- Volumes ---"
Get-Volume | Where-Object DriveLetter | ForEach-Object {
    Log ("{0}: {1} {2} free={3}GB size={4}GB health={5}" -f $_.DriveLetter,$_.FileSystemLabel,$_.FileSystem,[math]::Round($_.SizeRemaining/1GB,1),[math]::Round($_.Size/1GB,1),$_.HealthStatus)
}

Log "--- WSL ---"
Log ((wsl --status 2>&1 | Out-String).Trim())
Log ((wsl --list --verbose 2>&1 | Out-String).Trim())

Log "--- Git ---"
Set-Location $Repo
Log ((git status --short --branch 2>&1 | Out-String).Trim())
Log ("Last commit: " + ((git log -1 --oneline --decorate 2>&1 | Out-String).Trim()))
$ahead = git rev-list --count "@{u}..HEAD" 2>$null
$behind = git rev-list --count "HEAD..@{u}" 2>$null
Log "Ahead=$ahead Behind=$behind"

Log "--- VS Code ---"
$codeExe = "$env:LOCALAPPDATA\Programs\Microsoft VS Code\Code.exe"
$codeCmd = "$env:LOCALAPPDATA\Programs\Microsoft VS Code\bin\code.cmd"
if (Test-Path $codeExe) {
    Start-Process $codeExe -ArgumentList "`"$Repo`""
    Log "[OK] VS Code launched via Code.exe"
} elseif (Test-Path $codeCmd) {
    Start-Process $codeCmd -ArgumentList "`"$Repo`""
    Log "[OK] VS Code launched via code.cmd"
} else {
    Log "[WARN] VS Code not found"
}

Log "--- One-shot cleanup ---"
try {
    Unregister-ScheduledTask -TaskName $TaskName -Confirm:$false -ErrorAction Stop
    Log "[OK] Scheduled task '$TaskName' removed"
} catch {
    Log "[INFO] Task cleanup skipped: $($_.Exception.Message)"
}

Log "=== TrustOS auto-resume complete ==="
Log "Log file: $log"
