# Script de téléchargement du modèle Whisper Small
# Ce script télécharge le modèle ggml-small.bin depuis Hugging Face
# Usage: .\download-whisper-model.ps1 [-Force] [-Model <base|small|medium|large-v2|large-v3-turbo>]

param(
    [switch]$Force,
    [ValidateSet("base", "small", "medium", "large-v2", "large-v3-turbo")]
    [string]$Model = "small"
)

Write-Host "================================================" -ForegroundColor Cyan
Write-Host "  Téléchargement du modèle Whisper" -ForegroundColor Cyan
Write-Host "================================================" -ForegroundColor Cyan
Write-Host ""

# Définir les chemins
$appDataPath = [Environment]::GetFolderPath('ApplicationData')
$flemmeDir = Join-Path $appDataPath "Flemme"
$modelsDir = Join-Path $flemmeDir "models"

# Mapper le nom du modèle au nom de fichier et à l'URL
$modelInfo = @{
    "base" = @{
        FileName = "ggml-base.bin"
        Url = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin"
        Size = "~140 MB"
        DisplayName = "Whisper Base"
    }
    "small" = @{
        FileName = "ggml-small.bin"
        Url = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin"
        Size = "~466 MB"
        DisplayName = "Whisper Small"
    }
    "medium" = @{
        FileName = "ggml-medium.bin"
        Url = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin"
        Size = "~1.5 GB"
        DisplayName = "Whisper Medium"
    }
    "large-v2" = @{
        FileName = "ggml-large-v2.bin"
        Url = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v2.bin"
        Size = "~3 GB"
        DisplayName = "Whisper Large V2"
    }
    "large-v3-turbo" = @{
        FileName = "ggml-large-v3-turbo.bin"
        Url = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo.bin"
        Size = "~1.6 GB"
        DisplayName = "Whisper Large V3 Turbo"
    }
}

$selectedModel = $modelInfo[$Model]
$modelPath = Join-Path $modelsDir $selectedModel.FileName

Write-Host "Modèle sélectionné: $($selectedModel.DisplayName)" -ForegroundColor Cyan
Write-Host "Taille estimée: $($selectedModel.Size)" -ForegroundColor Cyan
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
    Write-Host "⚠ Le modèle $($selectedModel.DisplayName) existe déjà!" -ForegroundColor Yellow
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
$modelUrl = $selectedModel.Url

Write-Host "[2/3] Téléchargement du modèle ($($selectedModel.Size))..." -ForegroundColor Green
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
Write-Host "Le modèle $($selectedModel.DisplayName) est prêt à être utilisé." -ForegroundColor Green
Write-Host "Vous pouvez maintenant lancer l'application Flemme." -ForegroundColor Green
Write-Host ""
Write-Host "💡 Astuce: Téléchargez d'autres modèles avec:" -ForegroundColor Cyan
Write-Host "   .\download-whisper-model.ps1 -Model base      (Plus rapide, moins précis)" -ForegroundColor Gray
Write-Host "   .\download-whisper-model.ps1 -Model medium    (Plus lent, plus précis)" -ForegroundColor Gray
Write-Host "   .\download-whisper-model.ps1 -Model large-v2  (Très lent, très précis)" -ForegroundColor Gray
Write-Host ""
