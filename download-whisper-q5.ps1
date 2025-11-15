# Script de telechargement du modele Whisper Small Q5_1 (quantifie)
# Ce modele est plus leger et plus rapide que le modele standard

Write-Host "================================================" -ForegroundColor Cyan
Write-Host "  Telechargement du modele Whisper Small Q5_1" -ForegroundColor Cyan
Write-Host "================================================" -ForegroundColor Cyan
Write-Host ""

$appDataPath = [Environment]::GetFolderPath('ApplicationData')
$modelsDir = Join-Path $appDataPath "Flemme\models"
$modelPath = Join-Path $modelsDir "ggml-small-q5_1.bin"

Write-Host "Modele: Whisper Small Q5_1 (quantifie)" -ForegroundColor Cyan
Write-Host "Taille estimee: ~80 MB (plus leger que le modele standard)" -ForegroundColor Cyan
Write-Host "Dossier de destination: $modelsDir" -ForegroundColor Yellow
Write-Host ""

# Creer le dossier s'il n'existe pas
if (-not (Test-Path $modelsDir)) {
    New-Item -ItemType Directory -Path $modelsDir -Force | Out-Null
}

# Verifier si le modele existe deja
if (Test-Path $modelPath) {
    $fileSize = (Get-Item $modelPath).Length / 1MB
    Write-Host "! Le modele existe deja!" -ForegroundColor Yellow
    Write-Host "  Taille: $([math]::Round($fileSize, 2)) MB" -ForegroundColor Yellow
    Write-Host "  Chemin: $modelPath" -ForegroundColor Cyan
    exit 0
}

# URL du modele quantifie sur Hugging Face
$modelUrl = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.en-q5_1.bin"

Write-Host "Telechargement du modele..." -ForegroundColor Green
Write-Host "URL: $modelUrl" -ForegroundColor Gray
Write-Host ""

try {
    $ProgressPreference = 'Continue'
    Invoke-WebRequest -Uri $modelUrl -OutFile $modelPath -UseBasicParsing

    Write-Host ""
    Write-Host "- Telechargement termine!" -ForegroundColor Green

    if (Test-Path $modelPath) {
        $fileSize = (Get-Item $modelPath).Length / 1MB
        Write-Host "- Fichier verifie" -ForegroundColor Green
        Write-Host "  Taille: $([math]::Round($fileSize, 2)) MB" -ForegroundColor Gray
        Write-Host "  Chemin: $modelPath" -ForegroundColor Gray
    }
} catch {
    Write-Host ""
    Write-Host "x Erreur lors du telechargement:" -ForegroundColor Red
    Write-Host "  $($_.Exception.Message)" -ForegroundColor Red
    Write-Host ""
    Write-Host "Essayez le modele multilingue si l'anglais ne fonctionne pas:" -ForegroundColor Yellow
    Write-Host "  https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small-q5_1.bin" -ForegroundColor Yellow
    exit 1
}

Write-Host ""
Write-Host "================================================" -ForegroundColor Cyan
Write-Host "  + Installation terminee avec succes!" -ForegroundColor Green
Write-Host "================================================" -ForegroundColor Cyan
Write-Host ""
