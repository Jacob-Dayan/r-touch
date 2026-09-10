$ErrorActionPreference = "Stop"

$ProjectRoot = $PSScriptRoot
if (-not $ProjectRoot) {
    $ProjectRoot = (Get-Location).Path
}

$UsageLibDir = Join-Path $ProjectRoot "usage/lib"
$UsageCliDir = Join-Path $ProjectRoot "usage/cli"

if (-not (Test-Path $UsageLibDir)) {
    Write-Error "Error: Directory $UsageLibDir does not exist."
    exit 1
}

$BaseTmpDir = Join-Path ([System.IO.Path]::GetTempPath()) ("rtouch_test_all_" + [System.Guid]::NewGuid().ToString("N"))
Write-Host "DEBUG: setting up temp directory at $BaseTmpDir" -ForegroundColor Cyan

if (Test-Path $BaseTmpDir) {
    Remove-Item -Recurse -Force $BaseTmpDir
}
New-Item -ItemType Directory -Path $BaseTmpDir | Out-Null

function Cleanup {
    if (Test-Path $BaseTmpDir) {
        Remove-Item -Recurse -Force $BaseTmpDir -ErrorAction SilentlyContinue
    }
}

try {
    $TestProjectDir = Join-Path $BaseTmpDir "test-runner"
    cargo new --bin $TestProjectDir --quiet
    Write-Host "DEBUG: finished setup" -ForegroundColor Cyan

    Push-Location $TestProjectDir
    try {
        Write-Host "DEBUG: adding R-touch to dependencies" -ForegroundColor Cyan
        cargo add rtouch --path "$ProjectRoot" --quiet
        if ($LASTEXITCODE -ne 0) {
            cargo add rtouch --quiet
        }
    } finally {
        Pop-Location
    }

    Write-Host "DONE.`nStarting library tests...`n" -ForegroundColor Green

    $rsFiles = Get-ChildItem -Path $UsageLibDir -Filter "*.rs" -Recurse
    foreach ($file in $rsFiles) {
        Write-Host "=============================================================================================="
        Write-Host "Testing: $($file.FullName)" -ForegroundColor Yellow
        Write-Host "=============================================================================================="

        Copy-Item -Path $file.FullName -Destination (Join-Path $TestProjectDir "src/main.rs") -Force

        Push-Location $TestProjectDir
        try {
            Write-Host "Running cargo test..."
            $env:CARGO_TARGET_DIR = Join-Path ([System.IO.Path]::GetTempPath()) "cargo-target-runner"
            cargo test
            if ($LASTEXITCODE -ne 0) {
                throw "Test for $($file.Name) failed with exit code $LASTEXITCODE"
            }
        } finally {
            Pop-Location
        }

        Write-Host "Test for $($file.Name) passed successfully!`n" -ForegroundColor Green
    }

    Write-Host "Starting CLI tests...`n" -ForegroundColor Green

    # Run shell scripts if bash or wsl is available
    $shFiles = Get-ChildItem -Path $UsageCliDir -Filter "*.sh" -Recurse
    foreach ($file in $shFiles) {
        Write-Host "==============================================================================================="
        Write-Host "Testing CLI: $($file.FullName)" -ForegroundColor Yellow
        Write-Host "==============================================================================================="

        if ($file.FullName -match '^\\\\wsl(?:\.localhost)?\\([^\\]+)(.*)$') {
            $wslDistro = $Matches[1]
            $wslPath = $Matches[2].Replace('\', '/')
            & wsl -d $wslDistro bash -c "cd /home/shahar/codes/rust/r-touch && source ~/.cargo/env && bash '$wslPath'"
        } elseif (Get-Command "wsl" -ErrorAction SilentlyContinue) {
            $drive = $file.FullName.Substring(0, 1).ToLower()
            $rest = $file.FullName.Substring(3).Replace('\', '/')
            $wslPath = "/mnt/$drive/$rest"
            & wsl bash "$wslPath"
        } elseif (Get-Command "bash" -ErrorAction SilentlyContinue) {
            $bashPath = $file.FullName.Replace('\', '/')
            & bash "$bashPath"
        } else {
            Write-Host "Bash or WSL not found; skipping .sh execution" -ForegroundColor Yellow
            break
        }

        if ($LASTEXITCODE -ne 0) {
            throw "CLI test $($file.Name) failed with exit code $LASTEXITCODE"
        }

        Write-Host "Test for $($file.Name) passed successfully!`n" -ForegroundColor Green
    }

    Write-Host "All tests passed successfully!" -ForegroundColor Green
} finally {
    Cleanup
}
