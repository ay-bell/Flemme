# Script de téléchargement du modèle Silero VAD
# Ce script télécharge le modèle ONNX de détection de voix depuis Hugging Face
# Usage: .\download-silero-vad.ps1 [-Force]

param(
    [switch]$Force
)

Write-Host "================================================" -ForegroundColor Cyan
Write-Host "  Téléchargement du modèle Silero VAD" -ForegroundColor Cyan
Write-Host "================================================" -ForegroundColor Cyan
Write-Host ""

# Définir les chemins
$appDataPath = [Environment]::GetFolderPath('ApplicationData')
$flemmeDir = Join-Path $appDataPath "Flemme"
$modelsDir = Join-Path $flemmeDir "models"
$modelPath = Join-Path $modelsDir "silero_vad.onnx"

Write-Host "Modèle: Silero VAD (Voice Activity Detection)" -ForegroundColor Cyan
Write-Host "Taille estimée: ~20 MB" -ForegroundColor Cyan
Write-Host "Dossier de destination: $modelsDir" -ForegroundColor Yellow
Write-Host ""

# Créer le dossier s'il n'existe pas
if (-not (Test-Path $modelsDir)) {
    Write-Host "[1/3] Création du dossier models..." -ForegroundColor Green
    New-Item -ItemType Directory -Path $modelsDir -Force | Out-Null
    Write-Host "      ✓ Dossier créé: $modelsDir" -ForegroundColor Green
} else {
    Write-Host "[1/3] Le dossier models existe déjà" -ForegroundColor Green
}

Write-Host ""

# Vérifier si le modèle existe déjà
if (Test-Path $modelPath) {
    $fileSize = (Get-Item $modelPath).Length / 1MB
    Write-Host "⚠ Le modèle Silero VAD existe déjà!" -ForegroundColor Yellow
    Write-Host "  Taille: $([math]::Round($fileSize, 2)) MB" -ForegroundColor Yellow
    Write-Host ""

    if (-not $Force) {
        $response = Read-Host "Voulez-vous le re-télécharger? (o/N)"
        if ($response -ne 'o' -and $response -ne 'O') {
            Write-Host ""
            Write-Host "✓ Téléchargement annulé. Le modèle existant sera utilisé." -ForegroundColor Green
            Write-Host ""
            Write-Host "Chemin du modèle: $modelPath" -ForegroundColor Cyan
            exit 0
        }
    } else {
        Write-Host "Mode -Force activé, re-téléchargement forcé..." -ForegroundColor Yellow
    }
    Write-Host ""
}

# URL du modèle sur Hugging Face
$modelUrl = "https://huggingface.co/onnx-community/silero-vad/resolve/main/onnx/model.onnx"

Write-Host "[2/3] Téléchargement du modèle (~20 MB)..." -ForegroundColor Green
Write-Host "      URL: $modelUrl" -ForegroundColor Gray
Write-Host ""

try {
    # Télécharger avec barre de progression
    $ProgressPreference = 'Continue'
    Invoke-WebRequest -Uri $modelUrl -OutFile $modelPath -UseBasicParsing

    Write-Host ""
    Write-Host "      ✓ Téléchargement terminé!" -ForegroundColor Green
} catch {
    Write-Host ""
    Write-Host "✗ Erreur lors du téléchargement:" -ForegroundColor Red
    Write-Host "  $($_.Exception.Message)" -ForegroundColor Red
    Write-Host ""
    Write-Host "Vous pouvez télécharger manuellement depuis:" -ForegroundColor Yellow
    Write-Host "  $modelUrl" -ForegroundColor Yellow
    Write-Host "Et placer le fichier dans:" -ForegroundColor Yellow
    Write-Host "  $modelPath" -ForegroundColor Yellow
    exit 1
}

Write-Host ""
Write-Host "[3/3] Vérification du fichier..." -ForegroundColor Green

if (Test-Path $modelPath) {
    $fileSize = (Get-Item $modelPath).Length / 1MB
    Write-Host "      ✓ Fichier vérifié" -ForegroundColor Green
    Write-Host "      Taille: $([math]::Round($fileSize, 2)) MB" -ForegroundColor Gray
    Write-Host "      Chemin: $modelPath" -ForegroundColor Gray
} else {
    Write-Host "      ✗ Le fichier n'a pas été créé correctement" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "================================================" -ForegroundColor Cyan
Write-Host "  ✓ Installation terminée avec succès!" -ForegroundColor Green
Write-Host "================================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Le modèle Silero VAD est prêt à être utilisé." -ForegroundColor Green
Write-Host "Ce modèle permet de filtrer les silences dans vos enregistrements." -ForegroundColor Green
Write-Host ""