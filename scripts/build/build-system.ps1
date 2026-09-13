$ErrorActionPreference = 'Stop'

$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    $scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
    $hostExe = (Get-Process -Id $PID).Path
    Start-Process $hostExe -ArgumentList "-NoProfile -ExecutionPolicy Bypass -File `"$PSCommandPath`"" -WorkingDirectory $scriptDir -Verb RunAs
    exit
}

$repoRoot = (Resolve-Path "$PSScriptRoot\..\..").Path
Set-Location $repoRoot

cargo build --release

$targetDir = Join-Path $env:ProgramFiles 'R-touch\app'
if (-not (Test-Path $targetDir)) {
    New-Item -ItemType Directory -Force -Path $targetDir | Out-Null
}

$sourceExe = Join-Path $repoRoot 'target\release\rtouch.exe'
$destExe = Join-Path $targetDir 'rtouch.exe'
Copy-Item -Force -Path $sourceExe -Destination $destExe

$systemPath = [Environment]::GetEnvironmentVariable('Path', 'Machine')
$pathParts = if ($systemPath) { $systemPath -split ';' } else { @() }
if ($targetDir -notin $pathParts) {
    $newPath = if ($systemPath) { "$systemPath;$targetDir" } else { $targetDir }
    [Environment]::SetEnvironmentVariable('Path', $newPath, 'Machine')
}

if ($targetDir -notin ($env:Path -split ';')) {
    $env:Path = "$env:Path;$targetDir"
}

& $destExe -V
