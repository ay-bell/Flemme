# Script de build pour Flemme v0.1.4
# Compile deux versions : CUDA et CPU-only

$ErrorActionPreference = "Stop"
$version = "0.1.4"

Write-Host "=====================================" -ForegroundColor Cyan
Write-Host "Building Flemme v$version" -ForegroundColor Cyan
Write-Host "=====================================" -ForegroundColor Cyan

# Créer le dossier de sortie
$outputDir = ".\release-builds\v$version"
if (Test-Path $outputDir) {
    Remove-Item $outputDir -Recurse -Force
}
New-Item -ItemType Directory -Path $outputDir | Out-Null

# Nettoyer les builds précédents
Write-Host "`nCleaning previous builds..." -ForegroundColor Yellow
Set-Location flemme-app
Remove-Item -Path "src-tauri\target\release" -Recurse -Force -ErrorAction SilentlyContinue

# Build frontend une seule fois
Write-Host "`nBuilding frontend..." -ForegroundColor Yellow
npm run build

# ====================
# Build 1: CUDA version
# ====================
Write-Host "`n=====================================" -ForegroundColor Green
Write-Host "Building CUDA version..." -ForegroundColor Green
Write-Host "=====================================" -ForegroundColor Green

cargo build --manifest-path="src-tauri\Cargo.toml" --release --features cuda

# Copier l'exécutable CUDA
$cudaExe = "src-tauri\target\release\flemme-app.exe"
if (Test-Path $cudaExe) {
    Copy-Item $cudaExe "..\$outputDir\flemme-app-cuda-v$version.exe"
    Write-Host "✓ CUDA version compiled successfully" -ForegroundColor Green
} else {
    Write-Error "Failed to build CUDA version"
}

# ====================
# Build 2: CPU-only version
# ====================
Write-Host "`n=====================================" -ForegroundColor Green
Write-Host "Building CPU-only version..." -ForegroundColor Green
Write-Host "=====================================" -ForegroundColor Green

# Nettoyer le build CUDA
Remove-Item -Path "src-tauri\target\release" -Recurse -Force -ErrorAction SilentlyContinue

cargo build --manifest-path="src-tauri\Cargo.toml" --release --no-default-features

# Copier l'exécutable CPU
$cpuExe = "src-tauri\target\release\flemme-app.exe"
if (Test-Path $cpuExe) {
    Copy-Item $cpuExe "..\$outputDir\flemme-app-cpu-v$version.exe"
    Write-Host "✓ CPU-only version compiled successfully" -ForegroundColor Green
} else {
    Write-Error "Failed to build CPU version"
}

Set-Location ..

# ====================
# Générer latest.json
# ====================
Write-Host "`n=====================================" -ForegroundColor Cyan
Write-Host "Generating latest.json..." -ForegroundColor Cyan
Write-Host "=====================================" -ForegroundColor Cyan

$pubDate = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")

# Créer le latest.json (format pour l'auto-updater Tauri)
# Note: Pour des .exe portables, on utilisera la version CUDA par défaut
$json = "{`n"
$json += "  `"version`": `"$version`",`n"
$json += "  `"notes`": `"Nouvelle version avec logo paresseux et correction du bug de la fenêtre system tray`",`n"
$json += "  `"pub_date`": `"$pubDate`",`n"
$json += "  `"platforms`": {`n"
$json += "    `"windows-x86_64`": {`n"
$json += "      `"signature`": `"`",`n"
$json += "      `"url`": `"https://github.com/ay-bell/Flemme/releases/download/v$version/flemme-app-cuda-v$version.exe`"`n"
$json += "    }`n"
$json += "  }`n"
$json += "}`n"

# Écrire le fichier sans BOM
[System.IO.File]::WriteAllText("$outputDir\latest.json", $json, [System.Text.UTF8Encoding]::new($false))

Write-Host "`n=====================================" -ForegroundColor Green
Write-Host "Build complete!" -ForegroundColor Green
Write-Host "=====================================" -ForegroundColor Green
Write-Host "`nFiles created in: $outputDir" -ForegroundColor Cyan
Get-ChildItem $outputDir | Format-Table Name, Length -AutoSize

Write-Host "`nNext steps:" -ForegroundColor Yellow
Write-Host "1. Test both executables" -ForegroundColor White
Write-Host "2. Create GitHub release v$version" -ForegroundColor White
Write-Host "3. Upload all files from $outputDir" -ForegroundColor White
Write-Host "4. Tag the release: git tag v$version && git push origin v$version" -ForegroundColor White
