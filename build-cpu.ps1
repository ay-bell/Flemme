$ErrorActionPreference = "Stop"
$version = "0.1.4"

Write-Host "Building CPU version (no CUDA)..."

$outputDir = ".\release-builds\v$version"
if (-not (Test-Path $outputDir)) {
    New-Item -ItemType Directory -Path $outputDir | Out-Null
}

Set-Location flemme-app

# Clean previous build
Remove-Item -Path "src-tauri\target\release" -Recurse -Force -ErrorAction SilentlyContinue

# Temporarily modify Cargo.toml to disable CUDA by default
$cargoToml = Get-Content "src-tauri\Cargo.toml" -Raw
$originalCargoToml = $cargoToml
$cargoToml = $cargoToml -replace 'default = \["cuda"\]', 'default = []'
Set-Content "src-tauri\Cargo.toml" -Value $cargoToml

try {
    # Build frontend
    npm run build

    # Build with Tauri CLI (no CUDA since we changed default)
    npm run tauri build

    # Find and copy the MSI installer
    $msiInstaller = Get-ChildItem -Path "src-tauri\target\release\bundle\msi\" -Filter "*.msi" | Select-Object -First 1
    if ($msiInstaller) {
        $newName = "flemme-app-cpu-v$version-setup.msi"
        Copy-Item $msiInstaller.FullName "..\$outputDir\$newName"
        Write-Host "Done: $newName"

        $fileInfo = Get-Item "..\$outputDir\$newName"
        Write-Host "Size: $([math]::Round($fileInfo.Length / 1MB, 2)) MB"
    } else {
        Write-Error "MSI installer not found"
    }
} finally {
    # Restore original Cargo.toml
    Set-Content "src-tauri\Cargo.toml" -Value $originalCargoToml
    Write-Host "Cargo.toml restored"
}

Set-Location ..
