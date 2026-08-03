cargo build --release

$targetDir = "$env:LOCALAPPDATA\R-touch\bin"
New-Item -ItemType Directory -Force -Path $targetDir

Move-Item -Force -Path ".\target\release\rtouch.exe" "$targetDir\rtouch.exe"

[Environment]::SetEnvironmentVariable("Path", $env:Path + ";$targetDir", "User")
$env:Path += ";$targetDir"

# Test
reset
rtouch -V
