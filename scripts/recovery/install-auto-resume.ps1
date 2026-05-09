param(
    [string]$Repo = "C:\Users\nathan\Documents\Scripts\OSrust",
    [string]$TaskName = "TrustOS-AutoResume"
)

$ErrorActionPreference = "Stop"
$script = Join-Path $Repo "scripts\recovery\auto-resume.ps1"
if (-not (Test-Path -LiteralPath $script)) {
    throw "Missing auto-resume script: $script"
}

$action = New-ScheduledTaskAction -Execute "powershell.exe" -Argument "-NoProfile -ExecutionPolicy Bypass -File `"$script`" -Repo `"$Repo`" -TaskName `"$TaskName`""
$trigger = New-ScheduledTaskTrigger -AtLogOn -User $env:USERNAME
$settings = New-ScheduledTaskSettingsSet -Compatibility Win8 -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -StartWhenAvailable
$principal = New-ScheduledTaskPrincipal -UserId "$env:USERDOMAIN\$env:USERNAME" -LogonType Interactive -RunLevel Limited

Register-ScheduledTask -TaskName $TaskName -Action $action -Trigger $trigger -Settings $settings -Principal $principal -Force | Out-Null

"Installed scheduled task '$TaskName'. It will run once at next logon, write .recovery_workspace\logs\resume_<timestamp>.log, open VS Code, then remove itself."
