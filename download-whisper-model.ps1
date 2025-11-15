# Script de telechargement du modele Whisper Small
# Ce script telecharge le modele ggml-small.bin depuis Hugging Face
# Usage: .\download-whisper-model.ps1 [-Force] [-Model <base|small|medium|large-v2|large-v3-turbo>]

param(
    [switch]$Force,
    [ValidateSet("base", "small", "medium", "large-v2", "large-v3-turbo")]
    [string]$Model = "small"
)

Write-Host "================================================" -ForegroundColor Cyan
Write-Host "  Telechargement du modele Whisper" -ForegroundColor Cyan
Write-Host "================================================" -ForegroundColor Cyan
Write-Host ""

# Definir les chemins
$appDataPath = [Environment]::GetFolderPath('ApplicationData')
$flemmeDir = Join-Path $appDataPath "Flemme"
$modelsDir = Join-Path $flemmeDir "models"

# Mapper le nom du modele au nom de fichier et a l'URL
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

Write-Host "Modele selectionne: $($selectedModel.DisplayName)" -ForegroundColor Cyan
Write-Host "Taille estimee: $($selectedModel.Size)" -ForegroundColor Cyan
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
    Write-Host "! Le modele $($selectedModel.DisplayName) existe deja!" -ForegroundColor Yellow
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
$modelUrl = $selectedModel.Url

Write-Host "[2/3] Telechargement du modele ($($selectedModel.Size))..." -ForegroundColor Green
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
Write-Host "Le modele $($selectedModel.DisplayName) est pret a etre utilise." -ForegroundColor Green
Write-Host "Vous pouvez maintenant lancer l'application Flemme." -ForegroundColor Green
Write-Host ""
Write-Host "Astuce: Telechargez d'autres modeles avec:" -ForegroundColor Cyan
Write-Host "   .\download-whisper-model.ps1 -Model base      (Plus rapide, moins precis)" -ForegroundColor Gray
Write-Host "   .\download-whisper-model.ps1 -Model medium    (Plus lent, plus precis)" -ForegroundColor Gray
Write-Host "   .\download-whisper-model.ps1 -Model large-v2  (Tres lent, tres precis)" -ForegroundColor Gray
Write-Host ""
