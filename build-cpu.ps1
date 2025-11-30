$ErrorActionPreference = "Stop"
$version = "0.1.4"

Write-Host "Building CPU version with NSIS installer..."

$outputDir = ".\release-builds\v$version"
if (-not (Test-Path $outputDir)) {
    New-Item -ItemType Directory -Path $outputDir | Out-Null
}

Set-Location flemme-app

# Clean previous build
Remove-Item -Path "src-tauri\target\release" -Recurse -Force -ErrorAction SilentlyContinue

# Build with Tauri CLI (creates NSIS installer)
npm run tauri build -- --no-default-features

# Find and copy the NSIS installer
$nsisInstaller = Get-ChildItem -Path "src-tauri\target\release\bundle\nsis\" -Filter "*.exe" | Select-Object -First 1
if ($nsisInstaller) {
    $newName = "flemme-app-cpu-v$version-setup.exe"
    Copy-Item $nsisInstaller.FullName "..\$outputDir\$newName"
    Write-Host "Done: $newName"

    $fileInfo = Get-Item "..\$outputDir\$newName"
    Write-Host "Size: $([math]::Round($fileInfo.Length / 1MB, 2)) MB"
} else {
    Write-Error "NSIS installer not found"
}

Set-Location ..
