$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path "$PSScriptRoot\..\..").Path
Set-Location $repoRoot

cargo build --release

$targetDir = Join-Path $env:LOCALAPPDATA 'R-touch\app'
if (-not (Test-Path $targetDir)) {
    New-Item -ItemType Directory -Force -Path $targetDir | Out-Null
}

$sourceExe = Join-Path $repoRoot 'target\release\rtouch.exe'
$destExe = Join-Path $targetDir 'rtouch.exe'
Copy-Item -Force -Path $sourceExe -Destination $destExe

$userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
$pathParts = if ($userPath) { $userPath -split ';' } else { @() }
if ($targetDir -notin $pathParts) {
    $newPath = if ($userPath) { "$userPath;$targetDir" } else { $targetDir }
    [Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
}

if ($targetDir -notin ($env:Path -split ';')) {
    $env:Path = "$env:Path;$targetDir"
}

& $destExe -V
