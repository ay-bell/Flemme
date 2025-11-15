# Script de configuration automatique pour Flemme sur une nouvelle machine
# Auteur: Script généré pour faciliter le transfert de développement
# Usage: PowerShell -ExecutionPolicy Bypass -File setup-new-machine.ps1

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "   Configuration de Flemme - Nouveau PC" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Fonction pour vérifier si une commande existe
function Test-Command {
    param($Command)
    try {
        if (Get-Command $Command -ErrorAction Stop) {
            return $true
        }
    }
    catch {
        return $false
    }
}

# Fonction pour afficher un check ou une croix
function Write-Status {
    param(
        [bool]$Success,
        [string]$Message
    )
    if ($Success) {
        Write-Host "[✓] $Message" -ForegroundColor Green
    } else {
        Write-Host "[✗] $Message" -ForegroundColor Red
    }
}

Write-Host "Étape 1/6: Vérification des prérequis" -ForegroundColor Yellow
Write-Host "--------------------------------------" -ForegroundColor Yellow

# Vérifier Rust
$rustInstalled = Test-Command "rustc"
Write-Status $rustInstalled "Rust (rustc)"
if (-not $rustInstalled) {
    Write-Host "  → Téléchargez depuis: https://rustup.rs/" -ForegroundColor Gray
}

# Vérifier Cargo
$cargoInstalled = Test-Command "cargo"
Write-Status $cargoInstalled "Cargo"

# Vérifier Node.js
$nodeInstalled = Test-Command "node"
Write-Status $nodeInstalled "Node.js"
if (-not $nodeInstalled) {
    Write-Host "  → Téléchargez depuis: https://nodejs.org/" -ForegroundColor Gray
} else {
    $nodeVersion = node --version
    Write-Host "  → Version installée: $nodeVersion" -ForegroundColor Gray
}

# Vérifier npm
$npmInstalled = Test-Command "npm"
Write-Status $npmInstalled "npm"
if ($npmInstalled) {
    $npmVersion = npm --version
    Write-Host "  → Version installée: $npmVersion" -ForegroundColor Gray
}

# Vérifier Git
$gitInstalled = Test-Command "git"
Write-Status $gitInstalled "Git"
if (-not $gitInstalled) {
    Write-Host "  → Téléchargez depuis: https://git-scm.com/" -ForegroundColor Gray
}

Write-Host ""

# Si des prérequis manquent, arrêter
$allPrereqsInstalled = $rustInstalled -and $cargoInstalled -and $nodeInstalled -and $npmInstalled -and $gitInstalled
if (-not $allPrereqsInstalled) {
    Write-Host "ERREUR: Certains prérequis ne sont pas installés." -ForegroundColor Red
    Write-Host "Veuillez installer les outils manquants et relancer ce script." -ForegroundColor Red
    Write-Host ""
    Write-Host "Prérequis supplémentaires requis:" -ForegroundColor Yellow
    Write-Host "  - Visual Studio Build Tools avec MSVC" -ForegroundColor Gray
    Write-Host "    https://visualstudio.microsoft.com/downloads/" -ForegroundColor Gray
    Write-Host "  - CMake (pour la compilation de whisper-rs)" -ForegroundColor Gray
    Write-Host "    https://cmake.org/download/" -ForegroundColor Gray
    exit 1
}

Write-Host "Étape 2/6: Vérification du répertoire de projet" -ForegroundColor Yellow
Write-Host "-----------------------------------------------" -ForegroundColor Yellow

$projectPath = Split-Path -Parent $MyInvocation.MyCommand.Path
$appPath = Join-Path $projectPath "flemme-app"

if (Test-Path $appPath) {
    Write-Status $true "Répertoire flemme-app trouvé"
    Set-Location $appPath
} else {
    Write-Status $false "Répertoire flemme-app non trouvé"
    Write-Host "ERREUR: Le répertoire flemme-app est introuvable." -ForegroundColor Red
    Write-Host "Assurez-vous d'être dans le bon répertoire du projet." -ForegroundColor Red
    exit 1
}

Write-Host ""

Write-Host "Étape 3/6: Installation des dépendances npm" -ForegroundColor Yellow
Write-Host "-------------------------------------------" -ForegroundColor Yellow

if (Test-Path "package.json") {
    Write-Host "Installation des packages npm..." -ForegroundColor Gray
    npm install
    if ($LASTEXITCODE -eq 0) {
        Write-Status $true "Dépendances npm installées"
    } else {
        Write-Status $false "Échec de l'installation npm"
        exit 1
    }
} else {
    Write-Status $false "package.json non trouvé"
    exit 1
}

Write-Host ""

Write-Host "Étape 4/6: Récupération des dépendances Rust" -ForegroundColor Yellow
Write-Host "--------------------------------------------" -ForegroundColor Yellow

Set-Location "src-tauri"
if (Test-Path "Cargo.toml") {
    Write-Host "Téléchargement des crates Rust..." -ForegroundColor Gray
    cargo fetch
    if ($LASTEXITCODE -eq 0) {
        Write-Status $true "Dépendances Rust récupérées"
    } else {
        Write-Status $false "Échec de la récupération des dépendances Rust"
        exit 1
    }
} else {
    Write-Status $false "Cargo.toml non trouvé"
    exit 1
}

Set-Location $projectPath
Write-Host ""

Write-Host "Étape 5/6: Téléchargement des modèles ML (CRITIQUE)" -ForegroundColor Yellow
Write-Host "---------------------------------------------------" -ForegroundColor Yellow

# Créer le répertoire de modèles s'il n'existe pas
$modelsPath = Join-Path $env:APPDATA "Flemme\models"
if (-not (Test-Path $modelsPath)) {
    New-Item -ItemType Directory -Path $modelsPath -Force | Out-Null
    Write-Status $true "Répertoire de modèles créé: $modelsPath"
} else {
    Write-Status $true "Répertoire de modèles existe: $modelsPath"
}

# Télécharger Whisper model
$whisperModelPath = Join-Path $modelsPath "ggml-small.bin"
if (Test-Path $whisperModelPath) {
    Write-Status $true "Modèle Whisper déjà téléchargé"
} else {
    Write-Host "Téléchargement du modèle Whisper Small (~466 MB)..." -ForegroundColor Gray
    Write-Host "Cela peut prendre plusieurs minutes..." -ForegroundColor Gray

    $whisperScriptPath = Join-Path $projectPath "download-whisper-model.ps1"
    if (Test-Path $whisperScriptPath) {
        & $whisperScriptPath
        if (Test-Path $whisperModelPath) {
            Write-Status $true "Modèle Whisper téléchargé avec succès"
        } else {
            Write-Status $false "Échec du téléchargement du modèle Whisper"
            Write-Host "Veuillez exécuter manuellement: .\download-whisper-model.ps1" -ForegroundColor Red
        }
    } else {
        Write-Status $false "Script download-whisper-model.ps1 non trouvé"
    }
}

# Télécharger Silero VAD model
$vadModelPath = Join-Path $modelsPath "silero_vad.onnx"
if (Test-Path $vadModelPath) {
    Write-Status $true "Modèle Silero VAD déjà téléchargé"
} else {
    Write-Host "Téléchargement du modèle Silero VAD (~20 MB)..." -ForegroundColor Gray

    $vadScriptPath = Join-Path $projectPath "download-silero-vad.ps1"
    if (Test-Path $vadScriptPath) {
        & $vadScriptPath
        # Le script download-silero-vad.ps1 peut nécessiter move-silero-vad.ps1
        $moveVadScriptPath = Join-Path $projectPath "move-silero-vad.ps1"
        if (Test-Path $moveVadScriptPath) {
            & $moveVadScriptPath
        }

        if (Test-Path $vadModelPath) {
            Write-Status $true "Modèle Silero VAD téléchargé avec succès"
        } else {
            Write-Status $false "Échec du téléchargement du modèle Silero VAD"
            Write-Host "Veuillez exécuter manuellement: .\download-silero-vad.ps1" -ForegroundColor Red
        }
    } else {
        Write-Status $false "Script download-silero-vad.ps1 non trouvé"
    }
}

Write-Host ""

Write-Host "Étape 6/6: Vérification de la configuration" -ForegroundColor Yellow
Write-Host "-------------------------------------------" -ForegroundColor Yellow

$whisperExists = Test-Path $whisperModelPath
$vadExists = Test-Path $vadModelPath
$nodeModulesExists = Test-Path (Join-Path $appPath "node_modules")

Write-Status $whisperExists "Modèle Whisper présent"
Write-Status $vadExists "Modèle Silero VAD présent"
Write-Status $nodeModulesExists "Dépendances npm installées"

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "   Configuration terminée!" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

if ($whisperExists -and $vadExists -and $nodeModulesExists) {
    Write-Host "✓ Tout est prêt pour le développement!" -ForegroundColor Green
    Write-Host ""
    Write-Host "Prochaines étapes:" -ForegroundColor Yellow
    Write-Host "  1. cd flemme-app" -ForegroundColor Gray
    Write-Host "  2. npm run dev          (dans un terminal)" -ForegroundColor Gray
    Write-Host "  3. cargo tauri dev      (dans un autre terminal)" -ForegroundColor Gray
    Write-Host ""
    Write-Host "Pour un build de production:" -ForegroundColor Yellow
    Write-Host "  1. cd flemme-app" -ForegroundColor Gray
    Write-Host "  2. npm run build" -ForegroundColor Gray
    Write-Host "  3. cargo tauri build" -ForegroundColor Gray
    Write-Host ""
    Write-Host "Consultez MIGRATION.md pour plus de détails." -ForegroundColor Gray
} else {
    Write-Host "⚠ Configuration incomplète" -ForegroundColor Yellow
    Write-Host "Veuillez vérifier les erreurs ci-dessus et réessayer." -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Pour de l'aide, consultez:" -ForegroundColor Gray
    Write-Host "  - MIGRATION.md" -ForegroundColor Gray
    Write-Host "  - SETUP_CHECKLIST.md" -ForegroundColor Gray
    Write-Host "  - WINDOWS_SETUP.md" -ForegroundColor Gray
}

Write-Host ""
