# Script de telechargement du modele Silero VAD
# Ce script telecharge le modele ONNX de detection de voix depuis Hugging Face
# Usage: .\download-silero-vad.ps1 [-Force]

param(
    [switch]$Force
)

Write-Host "================================================" -ForegroundColor Cyan
Write-Host "  Telechargement du modele Silero VAD" -ForegroundColor Cyan
Write-Host "================================================" -ForegroundColor Cyan
Write-Host ""

# Definir les chemins
$appDataPath = [Environment]::GetFolderPath('ApplicationData')
$flemmeDir = Join-Path $appDataPath "Flemme"
$modelsDir = Join-Path $flemmeDir "models"
$modelPath = Join-Path $modelsDir "silero_vad.onnx"

Write-Host "Modele: Silero VAD (Voice Activity Detection)" -ForegroundColor Cyan
Write-Host "Taille estimee: ~20 MB" -ForegroundColor Cyan
Write-Host "Dossier de destination: $modelsDir" -ForegroundColor Yellow
Write-Host ""

# Creer le dossier s'il n'existe pas
if (-not (Test-Path $modelsDir)) {
    Write-Host "[1/3] Creation du dossier models..." -ForegroundColor Green
    New-Item -ItemType Directory -Path $modelsDir -Force | Out-Null
    Write-Host "      - Dossier cree: $modelsDir" -ForegroundColor Green
} else {
    Write-Host "[1/3] Le dossier models existe deja" -ForegroundColor Green
}

Write-Host ""

# Verifier si le modele existe deja
if (Test-Path $modelPath) {
    $fileSize = (Get-Item $modelPath).Length / 1MB
    Write-Host "! Le modele Silero VAD existe deja!" -ForegroundColor Yellow
    Write-Host "  Taille: $([math]::Round($fileSize, 2)) MB" -ForegroundColor Yellow
    Write-Host ""

    if (-not $Force) {
        $response = Read-Host "Voulez-vous le re-telecharger? (o/N)"
        if ($response -ne 'o' -and $response -ne 'O') {
            Write-Host ""
            Write-Host "- Telechargement annule. Le modele existant sera utilise." -ForegroundColor Green
            Write-Host ""
            Write-Host "Chemin du modele: $modelPath" -ForegroundColor Cyan
            exit 0
        }
    } else {
        Write-Host "Mode -Force active, re-telechargement force..." -ForegroundColor Yellow
    }
    Write-Host ""
}

# URL du modele sur Hugging Face
$modelUrl = "https://huggingface.co/onnx-community/silero-vad/resolve/main/onnx/model.onnx"

Write-Host "[2/3] Telechargement du modele (~20 MB)..." -ForegroundColor Green
Write-Host "      URL: $modelUrl" -ForegroundColor Gray
Write-Host ""

try {
    # Telecharger avec barre de progression
    $ProgressPreference = 'Continue'
    Invoke-WebRequest -Uri $modelUrl -OutFile $modelPath -UseBasicParsing

    Write-Host ""
    Write-Host "      - Telechargement termine!" -ForegroundColor Green
} catch {
    Write-Host ""
    Write-Host "x Erreur lors du telechargement:" -ForegroundColor Red
    Write-Host "  $($_.Exception.Message)" -ForegroundColor Red
    Write-Host ""
    Write-Host "Vous pouvez telecharger manuellement depuis:" -ForegroundColor Yellow
    Write-Host "  $modelUrl" -ForegroundColor Yellow
    Write-Host "Et placer le fichier dans:" -ForegroundColor Yellow
    Write-Host "  $modelPath" -ForegroundColor Yellow
    exit 1
}

Write-Host ""
Write-Host "[3/3] Verification du fichier..." -ForegroundColor Green

if (Test-Path $modelPath) {
    $fileSize = (Get-Item $modelPath).Length / 1MB
    Write-Host "      - Fichier verifie" -ForegroundColor Green
    Write-Host "      Taille: $([math]::Round($fileSize, 2)) MB" -ForegroundColor Gray
    Write-Host "      Chemin: $modelPath" -ForegroundColor Gray
} else {
    Write-Host "      x Le fichier n'a pas ete cree correctement" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "================================================" -ForegroundColor Cyan
Write-Host "  - Installation terminee avec succes!" -ForegroundColor Green
Write-Host "================================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Le modele Silero VAD est pret a etre utilise." -ForegroundColor Green
Write-Host "Ce modele permet de filtrer les silences dans vos enregistrements." -ForegroundColor Green
Write-Host ""
