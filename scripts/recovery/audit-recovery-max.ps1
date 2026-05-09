param(
    [string]$Repo = "C:\Users\nathan\Documents\Scripts\OSrust"
)

$ErrorActionPreference = "Continue"

function Get-Sha256OrNull([string]$Path) {
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        return $null
    }
    try {
        return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash
    } catch {
        return $null
    }
}

$reportDir = Join-Path $Repo ".recovery_workspace\reports"
New-Item -ItemType Directory -Force -Path $reportDir | Out-Null
$stamp = Get-Date -Format "yyyyMMdd_HHmmss"

$summaryPath = Join-Path $reportDir "recovery_audit_$stamp.txt"
$candidateCsv = Join-Path $reportDir "recovery_candidates_$stamp.csv"
$gitStatusPath = Join-Path $reportDir "git_status_$stamp.txt"

Set-Location $Repo

$status = git status --short --branch 2>&1 | Out-String
$status | Set-Content -LiteralPath $gitStatusPath -Encoding UTF8

$candidateRoots = @(
    ".recovery_workspace\found",
    ".recovery_workspace\found_from_git_history"
)

$rows = New-Object System.Collections.Generic.List[object]
foreach ($rootRel in $candidateRoots) {
    $root = Join-Path $Repo $rootRel
    if (-not (Test-Path -LiteralPath $root)) {
        continue
    }

    Get-ChildItem -LiteralPath $root -Recurse -File -Force -ErrorAction SilentlyContinue | ForEach-Object {
        $rel = $_.FullName.Substring($root.Length).TrimStart("\")
        $targetRel = $rel
        $isPartial = $false
        if ($targetRel.EndsWith(".partial")) {
            $targetRel = $targetRel.Substring(0, $targetRel.Length - ".partial".Length)
            $isPartial = $true
        }

        $target = Join-Path $Repo $targetRel
        $targetExists = Test-Path -LiteralPath $target -PathType Leaf
        $sourceHash = Get-Sha256OrNull $_.FullName
        $targetHash = Get-Sha256OrNull $target

        $rows.Add([PSCustomObject]@{
            SourceRoot = $rootRel
            Candidate = $rel
            Target = $targetRel
            Partial = $isPartial
            CandidateBytes = $_.Length
            TargetExists = $targetExists
            TargetBytes = if ($targetExists) { (Get-Item -LiteralPath $target).Length } else { $null }
            HashMatch = if ($targetHash) { $sourceHash -eq $targetHash } else { $false }
            CandidateSha256 = $sourceHash
            TargetSha256 = $targetHash
        })
    }
}

$rows | Export-Csv -LiteralPath $candidateCsv -NoTypeInformation -Encoding UTF8

$corruptFiles = Get-ChildItem -LiteralPath $Repo -Recurse -File -Force -ErrorAction SilentlyContinue |
    Where-Object { $_.Name -match "\.corrupt\.20260509$" -or $_.FullName -match "\\\.git\.corrupt\.20260509\\" }

$trackedModified = (git diff --name-only 2>$null | Measure-Object).Count
$untracked = (git ls-files --others --exclude-standard 2>$null | Measure-Object).Count
$candidateCount = $rows.Count
$candidateMissing = ($rows | Where-Object { -not $_.TargetExists }).Count
$candidateDifferent = ($rows | Where-Object { $_.TargetExists -and -not $_.HashMatch }).Count
$candidateSame = ($rows | Where-Object { $_.HashMatch }).Count

$lines = @()
$lines += "TrustOS maximum recovery audit"
$lines += "Time: $(Get-Date -Format o)"
$lines += "Repo: $Repo"
$lines += ""
$lines += "Git:"
$lines += "  Branch: $(git branch --show-current 2>$null)"
$lines += "  Last commit: $((git log -1 --oneline --decorate 2>$null | Out-String).Trim())"
$lines += "  Tracked modified/deleted: $trackedModified"
$lines += "  Untracked: $untracked"
$lines += ""
$lines += "Recovery candidates:"
$lines += "  Total candidates: $candidateCount"
$lines += "  Already identical in working tree: $candidateSame"
$lines += "  Target missing: $candidateMissing"
$lines += "  Target exists but differs: $candidateDifferent"
$lines += "  CSV: $candidateCsv"
$lines += ""
$lines += "Corrupt markers:"
$lines += "  Count: $($corruptFiles.Count)"
$lines += ""
$lines += "Next safe actions:"
$lines += "  1. Review recovery_candidates CSV for missing/different files."
$lines += "  2. Copy only source files that are missing or clearly better than current targets."
$lines += "  3. Keep .git.corrupt.20260509 and *.corrupt.20260509 until GitHub + D: + WSL copies exist."
$lines += "  4. Commit recovery scripts separately from project code changes."
$lines += ""
$lines += "Git status file: $gitStatusPath"

$lines | Set-Content -LiteralPath $summaryPath -Encoding UTF8

Get-Content -LiteralPath $summaryPath
