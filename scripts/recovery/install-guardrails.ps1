param(
    [string]$Repo = "C:\Users\nathan\Documents\Scripts\OSrust",
    [string]$TaskName = "TrustOS-SafeMirror-OnLogon"
)

$ErrorActionPreference = "Stop"

$syncScript = Join-Path $Repo "scripts\recovery\sync-safe-mirror.ps1"
if (-not (Test-Path -LiteralPath $syncScript)) {
    throw "Missing sync script: $syncScript"
}

$hookDir = Join-Path $Repo ".git\hooks"
New-Item -ItemType Directory -Force -Path $hookDir | Out-Null

$postCommit = Join-Path $hookDir "post-commit"
$hook = @'
#!/bin/sh
# Non-blocking safety mirror after every commit.
repo="$(git rev-parse --show-toplevel 2>/dev/null)"
if [ -n "$repo" ]; then
  powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$repo/scripts/recovery/sync-safe-mirror.ps1" -Quiet >/dev/null 2>&1 &
  git push --all origin >/dev/null 2>&1 &
fi
exit 0
'@
$hook | Set-Content -LiteralPath $postCommit -Encoding ASCII

$action = New-ScheduledTaskAction -Execute "powershell.exe" -Argument "-NoProfile -ExecutionPolicy Bypass -File `"$syncScript`" -Quiet"
$trigger = New-ScheduledTaskTrigger -AtLogOn -User $env:USERNAME
$settings = New-ScheduledTaskSettingsSet -Compatibility Win8 -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -StartWhenAvailable
$principal = New-ScheduledTaskPrincipal -UserId "$env:USERDOMAIN\$env:USERNAME" -LogonType Interactive -RunLevel Limited

Register-ScheduledTask -TaskName $TaskName -Action $action -Trigger $trigger -Settings $settings -Principal $principal -Force | Out-Null

"Installed:"
"  Git hook: $postCommit"
"  Scheduled task: $TaskName"
"  Sync script: $syncScript"
