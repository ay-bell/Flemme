# Script de configuration automatique pour Flemme sur une nouvelle machine
# Auteur: Script genere pour faciliter le transfert de developpement
# Usage: PowerShell -ExecutionPolicy Bypass -File setup-new-machine.ps1

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "   Configuration de Flemme - Nouveau PC" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Fonction pour verifier si une commande existe
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
        Write-Host "[+] $Message" -ForegroundColor Green
    } else {
        Write-Host "[-] $Message" -ForegroundColor Red
    }
}

Write-Host "Etape 1/6: Verification des prerequis" -ForegroundColor Yellow
Write-Host "--------------------------------------" -ForegroundColor Yellow

# Verifier Rust
$rustInstalled = Test-Command "rustc"
Write-Status $rustInstalled "Rust (rustc)"
if (-not $rustInstalled) {
    Write-Host "  -> Telechargez depuis: https://rustup.rs/" -ForegroundColor Gray
}

# Verifier Cargo
$cargoInstalled = Test-Command "cargo"
Write-Status $cargoInstalled "Cargo"

# Verifier Node.js
$nodeInstalled = Test-Command "node"
Write-Status $nodeInstalled "Node.js"
if (-not $nodeInstalled) {
    Write-Host "  -> Telechargez depuis: https://nodejs.org/" -ForegroundColor Gray
} else {
    $nodeVersion = node --version
    Write-Host "  -> Version installee: $nodeVersion" -ForegroundColor Gray
}

# Verifier npm
$npmInstalled = Test-Command "npm"
Write-Status $npmInstalled "npm"
if ($npmInstalled) {
    $npmVersion = npm --version
    Write-Host "  -> Version installee: $npmVersion" -ForegroundColor Gray
}

# Verifier Git
$gitInstalled = Test-Command "git"
Write-Status $gitInstalled "Git"
if (-not $gitInstalled) {
    Write-Host "  -> Telechargez depuis: https://git-scm.com/" -ForegroundColor Gray
}

Write-Host ""

# Si des prerequis manquent, arreter
$allPrereqsInstalled = $rustInstalled -and $cargoInstalled -and $nodeInstalled -and $npmInstalled -and $gitInstalled
if (-not $allPrereqsInstalled) {
    Write-Host "ERREUR: Certains prerequis ne sont pas installes." -ForegroundColor Red
    Write-Host "Veuillez installer les outils manquants et relancer ce script." -ForegroundColor Red
    Write-Host ""
    Write-Host "Prerequis supplementaires requis:" -ForegroundColor Yellow
    Write-Host "  - Visual Studio Build Tools avec MSVC" -ForegroundColor Gray
    Write-Host "    https://visualstudio.microsoft.com/downloads/" -ForegroundColor Gray
    Write-Host "  - CMake (pour la compilation de whisper-rs)" -ForegroundColor Gray
    Write-Host "    https://cmake.org/download/" -ForegroundColor Gray
    exit 1
}

Write-Host "Etape 2/6: Verification du repertoire de projet" -ForegroundColor Yellow
Write-Host "-----------------------------------------------" -ForegroundColor Yellow

$projectPath = Split-Path -Parent $MyInvocation.MyCommand.Path
$appPath = Join-Path $projectPath "flemme-app"

if (Test-Path $appPath) {
    Write-Status $true "Repertoire flemme-app trouve"
    Set-Location $appPath
} else {
    Write-Status $false "Repertoire flemme-app non trouve"
    Write-Host "ERREUR: Le repertoire flemme-app est introuvable." -ForegroundColor Red
    Write-Host "Assurez-vous d'etre dans le bon repertoire du projet." -ForegroundColor Red
    exit 1
}

Write-Host ""

Write-Host "Etape 3/6: Installation des dependances npm" -ForegroundColor Yellow
Write-Host "-------------------------------------------" -ForegroundColor Yellow

if (Test-Path "package.json") {
    Write-Host "Installation des packages npm..." -ForegroundColor Gray
    npm install
    if ($LASTEXITCODE -eq 0) {
        Write-Status $true "Dependances npm installees"
    } else {
        Write-Status $false "Echec de l'installation npm"
        exit 1
    }
} else {
    Write-Status $false "package.json non trouve"
    exit 1
}

Write-Host ""

Write-Host "Etape 4/6: Recuperation des dependances Rust" -ForegroundColor Yellow
Write-Host "--------------------------------------------" -ForegroundColor Yellow

Set-Location "src-tauri"
if (Test-Path "Cargo.toml") {
    Write-Host "Telechargement des crates Rust..." -ForegroundColor Gray
    cargo fetch
    if ($LASTEXITCODE -eq 0) {
        Write-Status $true "Dependances Rust recuperees"
    } else {
        Write-Status $false "Echec de la recuperation des dependances Rust"
        exit 1
    }
} else {
    Write-Status $false "Cargo.toml non trouve"
    exit 1
}

Set-Location $projectPath
Write-Host ""

Write-Host "Etape 5/6: Telechargement des modeles ML (CRITIQUE)" -ForegroundColor Yellow
Write-Host "---------------------------------------------------" -ForegroundColor Yellow

# Creer le repertoire de modeles s'il n'existe pas
$modelsPath = Join-Path $env:APPDATA "Flemme\models"
if (-not (Test-Path $modelsPath)) {
    New-Item -ItemType Directory -Path $modelsPath -Force | Out-Null
    Write-Status $true "Repertoire de modeles cree: $modelsPath"
} else {
    Write-Status $true "Repertoire de modeles existe: $modelsPath"
}

# Telecharger Whisper model
$whisperModelPath = Join-Path $modelsPath "ggml-small.bin"
if (Test-Path $whisperModelPath) {
    Write-Status $true "Modele Whisper deja telecharge"
} else {
    Write-Host "Telechargement du modele Whisper Small (~466 MB)..." -ForegroundColor Gray
    Write-Host "Cela peut prendre plusieurs minutes..." -ForegroundColor Gray

    $whisperScriptPath = Join-Path $projectPath "download-whisper-model.ps1"
    if (Test-Path $whisperScriptPath) {
        & $whisperScriptPath
        if (Test-Path $whisperModelPath) {
            Write-Status $true "Modele Whisper telecharge avec succes"
        } else {
            Write-Status $false "Echec du telechargement du modele Whisper"
            Write-Host "Veuillez executer manuellement: .\download-whisper-model.ps1" -ForegroundColor Red
        }
    } else {
        Write-Status $false "Script download-whisper-model.ps1 non trouve"
    }
}

# Telecharger Silero VAD model
$vadModelPath = Join-Path $modelsPath "silero_vad.onnx"
if (Test-Path $vadModelPath) {
    Write-Status $true "Modele Silero VAD deja telecharge"
} else {
    Write-Host "Telechargement du modele Silero VAD (~20 MB)..." -ForegroundColor Gray

    $vadScriptPath = Join-Path $projectPath "download-silero-vad.ps1"
    if (Test-Path $vadScriptPath) {
        & $vadScriptPath
        # Le script download-silero-vad.ps1 peut necessiter move-silero-vad.ps1
        $moveVadScriptPath = Join-Path $projectPath "move-silero-vad.ps1"
        if (Test-Path $moveVadScriptPath) {
            & $moveVadScriptPath
        }

        if (Test-Path $vadModelPath) {
            Write-Status $true "Modele Silero VAD telecharge avec succes"
        } else {
            Write-Status $false "Echec du telechargement du modele Silero VAD"
            Write-Host "Veuillez executer manuellement: .\download-silero-vad.ps1" -ForegroundColor Red
        }
    } else {
        Write-Status $false "Script download-silero-vad.ps1 non trouve"
    }
}

Write-Host ""

Write-Host "Etape 6/6: Verification de la configuration" -ForegroundColor Yellow
Write-Host "-------------------------------------------" -ForegroundColor Yellow

$whisperExists = Test-Path $whisperModelPath
$vadExists = Test-Path $vadModelPath
$nodeModulesExists = Test-Path (Join-Path $appPath "node_modules")

Write-Status $whisperExists "Modele Whisper present"
Write-Status $vadExists "Modele Silero VAD present"
Write-Status $nodeModulesExists "Dependances npm installees"

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "   Configuration terminee!" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

if ($whisperExists -and $vadExists -and $nodeModulesExists) {
    Write-Host "+ Tout est pret pour le developpement!" -ForegroundColor Green
    Write-Host ""
    Write-Host "Prochaines etapes:" -ForegroundColor Yellow
    Write-Host "  1. cd flemme-app" -ForegroundColor Gray
    Write-Host "  2. npm run dev          (dans un terminal)" -ForegroundColor Gray
    Write-Host "  3. cargo tauri dev      (dans un autre terminal)" -ForegroundColor Gray
    Write-Host ""
    Write-Host "Pour un build de production:" -ForegroundColor Yellow
    Write-Host "  1. cd flemme-app" -ForegroundColor Gray
    Write-Host "  2. npm run build" -ForegroundColor Gray
    Write-Host "  3. cargo tauri build" -ForegroundColor Gray
    Write-Host ""
    Write-Host "Consultez MIGRATION.md pour plus de details." -ForegroundColor Gray
} else {
    Write-Host "! Configuration incomplete" -ForegroundColor Yellow
    Write-Host "Veuillez verifier les erreurs ci-dessus et reessayer." -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Pour de l'aide, consultez:" -ForegroundColor Gray
    Write-Host "  - MIGRATION.md" -ForegroundColor Gray
    Write-Host "  - SETUP_CHECKLIST.md" -ForegroundColor Gray
    Write-Host "  - WINDOWS_SETUP.md" -ForegroundColor Gray
}

Write-Host ""
