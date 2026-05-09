param(
    [string]$Repo = "C:\Users\nathan\Documents\Scripts\OSrust"
)

$ErrorActionPreference = "Continue"

function Section([string]$Name) {
    "`n=== $Name ==="
}

function Run([string]$Label, [scriptblock]$Block) {
    Section $Label
    try {
        & $Block
    } catch {
        "ERROR: $($_.Exception.Message)"
    }
}

$stamp = Get-Date -Format "yyyyMMdd_HHmmss"
$logDir = Join-Path $Repo ".recovery_workspace\logs"
New-Item -ItemType Directory -Force -Path $logDir | Out-Null
$log = Join-Path $logDir "health_$stamp.log"

Start-Transcript -Path $log -Force | Out-Null

"TrustOS recovery health check"
"Time: $(Get-Date -Format o)"
"Host: $env:COMPUTERNAME"
"User: $env:USERNAME"
"Repo: $Repo"

Run "Windows shells" {
    "ComSpec process: $env:ComSpec"
    Get-ItemProperty -Path "HKCU:\Environment","HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Environment" |
        Select-Object PSPath,ComSpec | Format-List
    Get-Command cmd.exe,powershell.exe,git.exe,code.cmd -ErrorAction SilentlyContinue |
        Select-Object Name,Source,Version | Format-Table -AutoSize
    cmd.exe /d /c echo CMD_OK
    powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "`$PSVersionTable.PSVersion.ToString()"
}

Run "Disks and volumes" {
    Get-Disk | Select-Object Number,FriendlyName,BusType,OperationalStatus,HealthStatus,
        @{Name="SizeGB";Expression={[math]::Round($_.Size/1GB,1)}} | Format-Table -AutoSize
    Get-Volume | Select-Object DriveLetter,FileSystemLabel,FileSystem,DriveType,HealthStatus,
        @{Name="FreeGB";Expression={[math]::Round($_.SizeRemaining/1GB,1)}},
        @{Name="SizeGB";Expression={[math]::Round($_.Size/1GB,1)}} | Format-Table -AutoSize
}

Run "Git state" {
    Set-Location $Repo
    git status --short --branch
    git log -1 --oneline --decorate
    git remote -v
    git rev-parse --abbrev-ref --symbolic-full-name "@{u}" 2>$null
    $ahead = git rev-list --count "@{u}..HEAD" 2>$null
    $behind = git rev-list --count "HEAD..@{u}" 2>$null
    "Ahead: $ahead"
    "Behind: $behind"
}

Run "WSL" {
    wsl --status 2>&1
    wsl --list --verbose 2>&1
}

Run "VS Code and Copilot" {
    if (Test-Path "$env:LOCALAPPDATA\Programs\Microsoft VS Code\Code.exe") {
        & "$env:LOCALAPPDATA\Programs\Microsoft VS Code\Code.exe" --version
    }
    $codeCmd = "$env:LOCALAPPDATA\Programs\Microsoft VS Code\bin\code.cmd"
    if (Test-Path $codeCmd) {
        & $codeCmd --list-extensions --show-versions | Select-String -Pattern "copilot|powershell|github" -CaseSensitive:$false
    }
    $copilotBuiltin = "$env:LOCALAPPDATA\Programs\Microsoft VS Code\8b640eef5a\resources\app\extensions\copilot\package.json"
    if (Test-Path $copilotBuiltin) {
        $pkg = Get-Content -LiteralPath $copilotBuiltin -Raw | ConvertFrom-Json
        "Built-in Copilot: $($pkg.publisher).$($pkg.name) $($pkg.version)"
    }
}

Run "Scheduled tasks" {
    schtasks /Query /TN TrustOS-AutoResume /V /FO LIST 2>&1
    schtasks /Query /TN TrustOS-HealthCheck /V /FO LIST 2>&1
}

Run "Restore points and shadows" {
    Get-ComputerRestorePoint | Sort-Object CreationTime -Descending |
        Select-Object -First 5 CreationTime,Description,RestorePointType,SequenceNumber | Format-Table -AutoSize
    vssadmin list shadows 2>&1
}

"`nHealth log: $log"
Stop-Transcript | Out-Null
