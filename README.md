# Flemme 🦥

**Application de bureau pour la transcription vocale en temps réel avec traitement LLM optionnel**

Flemme est une application desktop moderne permettant de transcrire la parole en texte via un raccourci clavier global, avec support optionnel de traitement par modèles de langage (LLM). Migration haute performance d'une application Python vers un stack Rust/Tauri pour des gains significatifs en rapidité, mémoire et taille de distribution.

[![Tauri](https://img.shields.io/badge/Tauri-2.0-24C8DB?logo=tauri)](https://tauri.app)
[![Rust](https://img.shields.io/badge/Rust-1.70+-orange?logo=rust)](https://www.rust-lang.org)
[![Svelte](https://img.shields.io/badge/Svelte-5-FF3E00?logo=svelte)](https://svelte.dev)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.6-3178C6?logo=typescript)](https://www.typescriptlang.org)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

---

## 📋 Table des matières

- [Fonctionnalités](#-fonctionnalités)
- [Architecture technique](#-architecture-technique)
- [Installation](#-installation)
- [Utilisation](#-utilisation)
- [Configuration](#-configuration)
- [Développement](#-développement)
- [Build de production](#-build-de-production)
- [Performances](#-performances)
- [Roadmap](#-roadmap)
- [Contribuer](#-contribuer)
- [Licence](#-licence)

---

## ✨ Fonctionnalités

### Transcription vocale
- **Enregistrement par raccourci clavier** - Mode Push-to-talk (maintenir) ou Toggle (appuyer une fois)
- **Transcription temps réel** - Utilise Whisper.cpp avec modèles quantizés Q5 (Tiny/Base/Small/Medium/Large)
- **Détection d'activité vocale (VAD)** - Filtrage automatique des silences via Silero VAD avec padding intelligent (150ms)
- **Multi-langues** - Support de FR, EN, ES, DE avec détection automatique
- **Vocabulaire personnalisé** - Contextual biasing pour noms propres et termes techniques
- **Accélération GPU** - Support CUDA optionnel (~3x plus rapide sur NVIDIA)
- **Faible latence** - 150-220ms total (GPU) ou 250-320ms (CPU) pour 4 secondes d'audio

### Intégration système
- **Raccourcis globaux** - Fonctionne dans toutes les applications (Ctrl+Alt+R par défaut)
- **Touche d'annulation** - Stop sans transcription (Escape en mode toggle)
- **Copie automatique** - Collage automatique du texte transcrit avec simulation Ctrl+V
- **System tray** - Icône paresseux dans la zone de notification avec menu contextuel
- **Sélection de périphérique** - Choix du microphone ou utilisation du périphérique par défaut
- **Minimisation en tray** - La fenêtre se cache au lieu de se fermer
- **Auto-updater** - Vérification automatique des mises à jour sur GitHub avec signatures cryptographiques

### Traitement LLM avancé
- **5 Providers supportés**
  - **OpenRouter** - Accès à +100 modèles via une seule API
  - **Gemini** - Modèles Google (Gemini Pro, Flash, etc.)
  - **OpenAI** - GPT-3.5, GPT-4, GPT-4o
  - **LM Studio** - Inference locale sans clé API (auto-détection des modèles)
  - **Ollama** - Inference locale sans clé API (auto-détection des modèles)
- **Modes d'exécution** - Configurations multiples avec prompts système personnalisés
  - Mode Standard (transcription seule)
  - Modes personnalisés avec traitement LLM (correction, traduction, résumé, etc.)
  - Sélection rapide depuis le system tray
- **Stockage sécurisé** - Clés API dans le trousseau système (Windows Credential Manager)
- **Gestion des timeouts** - 30s pour APIs cloud, 5min pour LM Studio/Ollama
- **Détection automatique** - Découverte des modèles disponibles sur LM Studio et Ollama

### Interface utilisateur
- **Panneau de configuration complet** - Interface à onglets pour tous les paramètres
  1. **Paramètres** - Hotkey, langue, mode push/toggle, auto-paste, périphérique
  2. **Vocabulaire** - Gestion des mots personnalisés
  3. **Modèles Vocaux** - Téléchargement et gestion des modèles Whisper
  4. **IA et Modèles** - Configuration LLM (cloud et local)
  5. **Modes d'Exécution** - Création et édition de modes personnalisés
  6. **À propos** - Vérification des mises à jour
- **Indicateur d'enregistrement flottant** - Fenêtre avec :
  - Visualisation spectrale en temps réel (gradient vert personnalisé)
  - Affichage du mode actif et modèle Whisper utilisé
  - Animation de chargement pendant la transcription
  - Positionnement automatique en bas-centre de l'écran
- **Design moderne** - Thème sombre, animations fluides, icône paresseux
- **Configuration persistante** - Sauvegarde automatique dans `%APPDATA%/Flemme/settings.json`

### Optimisations
- **Architecture multi-threads** - Workers dédiés pour audio et transcription (pas de blocage UI)
- **Chargement paresseux** - Modèles Whisper chargés uniquement à la première utilisation
- **Threading optimisé** - Allocation automatique basée sur num_cpus
- **Resampling haute qualité** - Rubato FFT-based pour conversion vers 16kHz
- **VAD intelligent** - Stratégie adaptative : garde tout si >30% parole, sinon extrait segments
- **Faible empreinte mémoire** - <300 MB idle, <500 MB en enregistrement

---

## 🏗️ Architecture technique

### Stack technologique

#### Backend (Rust)
| Composant | Technologie | Version | Rôle |
|-----------|-------------|---------|------|
| Framework | Tauri | 2.0 | Framework desktop avec UI web |
| Audio | cpal | 0.15 | Capture audio cross-platform |
| Transcription | whisper-rs | 0.15 | Liaison Rust pour whisper.cpp |
| VAD | ONNX Runtime | 2.0.0-rc.10 | Détection d'activité vocale (Silero) |
| Resampling | rubato | 0.15 | Rééchantillonnage audio haute qualité FFT |
| Presse-papiers | arboard | 3 | Accès clipboard cross-platform |
| Clavier | enigo | 0.2 | Simulation clavier (auto-paste) |
| HTTP | reqwest + tokio | 0.11 + 1 | Client HTTP async pour APIs LLM |
| Raccourcis | tauri-plugin-global-shortcut | 2 | Enregistrement de hotkeys globales |
| Credentials | keyring | 2 | Stockage sécurisé clés API (OS keyring) |
| Auto-updater | tauri-plugin-updater | 2 | Système de mise à jour automatique |

#### Frontend (Svelte/TypeScript)
| Composant | Technologie | Version | Rôle |
|-----------|-------------|---------|------|
| Framework | Svelte | 5 (runes) | Framework UI réactif avec signaux |
| Build Tool | Vite | 6 | Bundler ultra-rapide |
| Langage | TypeScript | 5.6.2 | JavaScript type-safe |
| UI Components | Bits UI | 2.14.2 | Composants headless accessibles |
| Styling | Tailwind CSS | 4 | Framework CSS utility-first |
| Icons | Lucide Svelte | 0.544 | Bibliothèque d'icônes |
| Audio Viz | AudioMotion-Analyzer | 4.5.1 | Visualisation spectrale temps réel |

### Architecture logicielle

```
┌──────────────────────────────────────────────────────────────┐
│                      Frontend (Svelte)                       │
│  ┌────────────────┐  ┌──────────────┐  ┌─────────────────┐ │
│  │   Settings.    │  │  Recording   │  │  UpdateChecker  │ │
│  │    svelte      │  │  Indicator   │  │                 │ │
│  └────────┬───────┘  └──────┬───────┘  └─────────────────┘ │
└───────────┼──────────────────┼──────────────────────────────┘
            │                  │
            │   Tauri Commands │
            ▼                  ▼
┌──────────────────────────────────────────────────────────────┐
│                      Backend (Rust)                          │
│  ┌─────────────────────────────────────────────────────────┐│
│  │         lib.rs (AppState + Commands + Workers)          ││
│  └──┬───────┬──────────┬──────────┬──────────┬────────────┘│
│     │       │          │          │          │              │
│  ┌──▼──┐ ┌─▼─────┐ ┌──▼──────┐ ┌─▼──────┐ ┌▼────────────┐ │
│  │Audio│ │Transc-│ │Hotkey   │ │Clip-   │ │Config/LLM   │ │
│  │Worker│Ription││ │Listener │ │board   │ │             │ │
│  │Thread Worker  │ │         │ │Manager │ │             │ │
│  └─────┘ └───────┘ └─────────┘ └────────┘ └─────────────┘ │
│                                                              │
│  ┌──────────────┐  ┌─────────────┐  ┌──────────────────┐  │
│  │ Silero VAD   │  │ Whisper.cpp │  │  OS Keyring      │  │
│  │ (ONNX)       │  │ (CPU/CUDA)  │  │  (API Keys)      │  │
│  └──────────────┘  └─────────────┘  └──────────────────┘  │
└──────────────────────────────────────────────────────────────┘
            │                  │                 │
            ▼                  ▼                 ▼
    ┌──────────────┐   ┌──────────────┐  ┌─────────────┐
    │  Microphone  │   │  LLM APIs    │  │System Tray  │
    │   (cpal)     │   │ 5 Providers  │  │Menu + Icon  │
    └──────────────┘   └──────────────┘  └─────────────┘
```

### Flux d'exécution

#### 1. Enregistrement
```
Utilisateur appuie sur Ctrl+Alt+R
    ↓
HotkeyListener déclenche commande start_recording
    ↓
AudioWorker commence la capture (buffer circulaire cpal)
    ↓
Fenêtre indicateur apparaît avec visualisation spectrale temps réel
    ↓
Utilisateur relâche (push-to-talk) ou appuie à nouveau (toggle)
    ↓
AudioWorker arrête et retourne le buffer audio (Vec<f32>)
```

#### 2. Transcription + VAD
```
Buffer audio reçu (resamplé à 16kHz si nécessaire)
    ↓
Silero VAD analyse l'audio (chunking 512/1024/1536 samples)
    ↓
Si >30% parole détectée → Garde tout l'audio
Sinon → Extrait segments de parole avec contraintes durée
    ↓
Padding 150ms avant chaque segment (préserve début de parole)
    ↓
WhisperEngine charge le modèle (lazy loading, une seule fois)
    ↓
Préparation custom words pour contextual biasing
    ↓
Inférence Whisper (greedy sampling, 8 threads CPU ou GPU CUDA)
    ↓
Détection langue auto si language = "auto"
    ↓
Texte transcrit retourné
```

#### 3. Traitement LLM (si mode personnalisé actif)
```
Texte transcrit obtenu
    ↓
Vérification du mode actif (standard ou personnalisé)
    ↓
Si mode personnalisé avec LLM assigné:
    ├─ Récupération config LLM (service_type, api_url, model_name)
    ├─ Détection type service (OpenRouter/Gemini/OpenAI/LMStudio/Ollama)
    ├─ Obtention clé API depuis Windows Credential Manager (ou skip si local)
    ├─ Construction payload JSON selon provider
    ├─ Appel HTTP POST avec timeout (30s cloud, 5min local)
    ├─ Parsing réponse selon format provider
    └─ Retour texte traité (au lieu de transcription brute)
```

#### 4. Auto-paste
```
Texte final prêt (transcription ou résultat LLM)
    ↓
Copie dans le presse-papiers (arboard)
    ↓
Attente 50ms pour stabilisation clipboard
    ↓
Simulation Ctrl+V via enigo (key down + up)
    ↓
Texte inséré dans l'application active
    ↓
Fenêtre indicateur se cache avec fade-out
```

---

## 🚀 Installation

### Téléchargement

**Releases officielles** : [GitHub Releases](https://github.com/ay-bell/Flemme/releases)

Deux versions disponibles :
- **flemme-app-cuda-vX.X.X-setup.exe** - Version avec accélération GPU NVIDIA (recommandée si GPU compatible)
- **flemme-app-cpu-vX.X.X-setup.exe** - Version CPU universelle (compatible tous PC)

### Prérequis

#### Utilisateur
- **Système** : Windows 10/11 (64-bit)
- **GPU** (optionnel) : NVIDIA avec CUDA 11.x/12.x pour version CUDA
- **Microphone** : Périphérique d'entrée audio fonctionnel
- **Espace disque** : ~500 MB (application + modèles)

#### Développement
- **Rust** : 1.70+ ([rustup.rs](https://rustup.rs))
- **Node.js** : 18+ ([nodejs.org](https://nodejs.org))
- **Visual Studio Build Tools** : Pour la compilation sur Windows
- **CUDA Toolkit** (optionnel) : 11.x ou 12.x pour build GPU

### Installation depuis les sources

```bash
# 1. Cloner le dépôt
git clone https://github.com/ay-bell/Flemme.git
cd Flemme/flemme-app

# 2. Installer les dépendances frontend
npm install

# 3. Lancer en mode développement
npm run tauri dev

# 4. Build de production (voir section dédiée)
```

### Premier lancement

1. **Installation** - Lancer le setup.exe, l'application s'installe dans `%LOCALAPPDATA%\flemme-app`
2. **Téléchargement modèle** - Au premier lancement, aller dans Paramètres > Modèles Vocaux
   - Recommandé : **ggml-base-q5_1.bin** (60 MB, bon équilibre qualité/vitesse)
   - Le modèle VAD (silero_vad.onnx) est inclus dans l'installation
3. **Configuration** - Tester le raccourci clavier dans l'onglet Paramètres
4. **Premier test** - Appuyer sur Ctrl+Alt+R, parler, relâcher → le texte devrait apparaître !

---

## 📖 Utilisation

### Démarrage rapide

1. **Lancer l'application** - Icône paresseux dans le menu démarrer ou system tray
2. **Vérifier les paramètres**
   - Langue : Français (par défaut)
   - Mode : Push-to-talk ou Toggle
   - Auto-paste : Activé
   - Microphone : Par défaut ou sélection manuelle
3. **Enregistrer**
   - **Mode Push-to-talk** : Maintenir Ctrl+Alt+R enfoncé, parler, relâcher
   - **Mode Toggle** : Appuyer sur Ctrl+Alt+R, parler, appuyer à nouveau (ou Escape pour annuler)
4. **Le texte est automatiquement collé** dans l'application active

### System Tray

L'icône paresseux dans la barre des tâches permet :
- **Clic droit > Paramètres** : Ouvrir la fenêtre de configuration
- **Clic droit > Modes** : Changer rapidement de mode d'exécution (Standard, Correction, etc.)
- **Clic droit > Quitter** : Fermer l'application
- **Double-clic** : Afficher la fenêtre principale si elle est cachée

### Vocabulaire personnalisé

Améliore la reconnaissance des noms propres et termes techniques :

```
Paramètres > Vocabulaire
1. Ajouter des mots spécifiques à votre domaine
2. Exemples : "Aymeric Bellavoine", "PPAT", "Harmonie Mutuelle", "SvelteKit"
3. Ces mots seront prioritaires lors de la transcription (contextual biasing)
4. Maximum recommandé : 50 mots pour ne pas surcharger
```

### Configuration LLM

#### Services Cloud (OpenRouter, Gemini, OpenAI)

```
Paramètres > IA et Modèles > LLM Cloud
1. Cliquer "Ajouter un modèle LLM"
2. Sélectionner le provider (OpenRouter/Gemini/OpenAI)
3. Entrer :
   - Nom du modèle (ex: "GPT-4o")
   - URL API (pré-remplie)
   - Nom du modèle dans l'API (ex: "gpt-4o")
   - Clé API (stockée de façon sécurisée)
4. Tester la connexion
5. Sauvegarder
```

#### Services Locaux (LM Studio, Ollama)

```
Paramètres > IA et Modèles > LLM Local

Pour LM Studio:
1. Lancer LM Studio en mode serveur (port 1234 par défaut)
2. Charger un modèle
3. Cliquer "Détecter modèles LM Studio"
4. Les modèles disponibles apparaissent automatiquement

Pour Ollama:
1. Installer Ollama (ollama.com)
2. Lancer : ollama serve
3. Télécharger un modèle : ollama pull llama2
4. Cliquer "Détecter modèles Ollama"
5. Les modèles disponibles apparaissent automatiquement

Aucune clé API requise pour les services locaux !
```

### Modes d'exécution

Créez des workflows personnalisés :

```
Paramètres > Modes d'Exécution

Mode Standard (par défaut) :
- Transcription pure sans traitement LLM
- Rapide et sans latence réseau

Modes personnalisés (exemples) :
1. "Correction orthographe"
   - LLM : Gemini Pro
   - Prompt : "Corrige uniquement l'orthographe et la grammaire. Ne modifie pas le sens."

2. "Traduction EN → FR"
   - LLM : GPT-4o
   - Prompt : "Traduis ce texte en français professionnel."

3. "Résumé court"
   - LLM : LM Studio (llama-3.1-8b)
   - Prompt : "Résume en 3 points clés maximum."

4. "Email formel"
   - LLM : Gemini Flash
   - Prompt : "Reformule en email professionnel formel avec formule de politesse."

Changement rapide :
- System tray > Modes > Sélectionner le mode
- Ou Paramètres > Modes d'Exécution > Activer
```

---

## ⚙️ Configuration

### Fichier de configuration
**Emplacement** : `%APPDATA%/Flemme/settings.json`

```json
{
  "hotkey": "Ctrl+Alt+R",
  "cancel_key": "Escape",
  "language": "fr",
  "auto_paste": true,
  "model_name": "ggml-base-q5_1.bin",
  "push_to_talk": false,
  "device_name": null,
  "custom_words": [
    "Aymeric Bellavoine",
    "PPAT",
    "Harmonie Mutuelle"
  ],
  "llm_models": [
    {
      "id": "uuid-generated",
      "name": "Gemini Pro",
      "api_url": "https://generativelanguage.googleapis.com/v1beta/models/gemini-pro:generateContent",
      "model_name": "gemini-pro",
      "service_type": "gemini"
    },
    {
      "id": "lm-studio-local",
      "name": "Llama 3.1 8B (Local)",
      "api_url": "http://localhost:1234/v1/chat/completions",
      "model_name": "llama-3.1-8b-instruct",
      "service_type": "lm_studio"
    }
  ],
  "execution_modes": [
    {
      "id": "standard",
      "name": "Standard",
      "llm_model_id": null,
      "system_prompt": ""
    },
    {
      "id": "mode-correction",
      "name": "Correction orthographe",
      "llm_model_id": "uuid-gemini",
      "system_prompt": "Corrige l'orthographe et la grammaire..."
    }
  ],
  "active_mode": "standard"
}
```

### Paramètres disponibles

| Paramètre | Type | Par défaut | Description |
|-----------|------|------------|-------------|
| `hotkey` | string | "Ctrl+Alt+R" | Raccourci d'enregistrement global |
| `cancel_key` | string | "Escape" | Touche d'annulation (mode toggle) |
| `language` | string | "fr" | Langue transcription (fr/en/es/de/auto) |
| `auto_paste` | boolean | true | Collage automatique du résultat |
| `model_name` | string | "ggml-base-q5_1.bin" | Modèle Whisper utilisé |
| `push_to_talk` | boolean | false | true = maintenir, false = toggle |
| `device_name` | string? | null | Microphone spécifique ou défaut |
| `custom_words` | string[] | [] | Vocabulaire personnalisé (contextual biasing) |

### Modèles Whisper disponibles

Tous les modèles sont quantizés Q5 pour un bon équilibre qualité/taille :

| Modèle | Taille | Précision | Vitesse CPU | Vitesse GPU | Recommandation |
|--------|--------|-----------|-------------|-------------|----------------|
| **ggml-tiny-q5_1.bin** | 32 MB | ★★☆☆☆ | ★★★★★ | ★★★★★ | Tests rapides |
| **ggml-base-q5_1.bin** | 60 MB | ★★★☆☆ | ★★★★☆ | ★★★★★ | **Usage général** ✅ |
| **ggml-small-q5_1.bin** | 192 MB | ★★★★☆ | ★★★☆☆ | ★★★★☆ | Haute qualité |
| **ggml-medium-q5_0.bin** | 940 MB | ★★★★★ | ★★☆☆☆ | ★★★☆☆ | Qualité max |
| **ggml-large-v3-turbo-q5_0.bin** | 950 MB | ★★★★★ | ★★☆☆☆ | ★★★★☆ | Large rapide |

Téléchargement via l'interface (Paramètres > Modèles Vocaux) ou depuis [Hugging Face](https://huggingface.co/ggerganov/whisper.cpp).

---

## 👨‍💻 Développement

### Structure du projet

```
Flemme/
├── flemme-app/                  # Application principale
│   ├── src-tauri/               # Backend Rust
│   │   ├── src/
│   │   │   ├── main.rs          # Point d'entrée binaire
│   │   │   ├── lib.rs           # État app, commandes, workers (1553 lignes)
│   │   │   ├── audio/
│   │   │   │   ├── recorder.rs  # Capture audio (cpal)
│   │   │   │   └── vad.rs       # Silero VAD (ONNX)
│   │   │   ├── transcription/
│   │   │   │   ├── whisper.rs   # Moteur Whisper principal
│   │   │   │   ├── engine.rs    # Wrapper WhisperEngine
│   │   │   │   └── models.rs    # Métadonnées modèles
│   │   │   ├── hotkey/
│   │   │   │   └── listener.rs  # Gestion raccourcis globaux
│   │   │   ├── clipboard/
│   │   │   │   └── manager.rs   # Presse-papiers & auto-paste
│   │   │   ├── config/
│   │   │   │   └── settings.rs  # Configuration persistante
│   │   │   └── llm/
│   │   │       ├── mod.rs       # Clients API LLM (547 lignes)
│   │   │       └── keyring_manager.rs # Stockage sécurisé
│   │   ├── Cargo.toml           # Dépendances Rust + features
│   │   └── tauri.conf.json      # Config Tauri (NSIS, updater)
│   │
│   ├── src/                     # Frontend Svelte
│   │   ├── lib/
│   │   │   ├── components/
│   │   │   │   ├── Settings.svelte       # UI paramètres (~1900 lignes)
│   │   │   │   ├── RecordingIndicator.svelte # Indicateur enregistrement
│   │   │   │   ├── UpdateChecker.svelte  # Vérification MAJ
│   │   │   │   └── ui/          # Composants Bits UI
│   │   │   └── utils.ts
│   │   ├── routes/
│   │   │   ├── +page.svelte     # Page principale (settings)
│   │   │   ├── +layout.svelte   # Layout avec auto-update check
│   │   │   └── indicator/
│   │   │       └── +page.svelte # Fenêtre indicateur flottante
│   │   └── app.css
│   │
│   ├── package.json
│   ├── vite.config.js
│   ├── svelte.config.js
│   └── tailwind.config.js
│
├── build-cuda.ps1               # Script build version CUDA
├── build-cpu.ps1                # Script build version CPU
├── generate-latest-json.ps1     # Génère manifest auto-updater
├── logo_picto.png               # Logo paresseux source
└── README.md
```

### Commandes de développement

```bash
# Développement avec hot reload
cd flemme-app
npm run tauri dev

# Build frontend seul
npm run build

# Vérification TypeScript/Svelte
npm run check

# Tests Rust
cd src-tauri && cargo test

# Formatage code Rust
cd src-tauri && cargo fmt

# Linting Rust
cd src-tauri && cargo clippy -- -D warnings
```

### Architecture des workers

**AudioWorker Thread**
- État isolé pour la capture audio
- Communication via canaux mpsc : `StartRecording`, `StopRecording`, `IsRecording`, `Shutdown`
- Buffer circulaire pour éviter les allocations dynamiques pendant l'enregistrement
- Passage de messages non-bloquant

**TranscriptionWorker Thread**
- Chargement paresseux du modèle Whisper (première utilisation seulement)
- Commandes : `Transcribe`, `ReloadModel`, `Shutdown`
- Charge les custom words depuis settings pour chaque transcription
- Maintient le modèle en mémoire pour performance
- Support CUDA via feature flag

**Main Thread (Tauri Event Loop)**
- Exécute la boucle événements Tauri
- Gère les fenêtres (main + indicator)
- Expose les commandes Tauri au frontend
- Gère les raccourcis globaux et system tray

Cette conception empêche la capture audio ou la transcription de bloquer l'UI, garantissant une expérience fluide.

### Commandes Tauri exposées

Plus de 30 commandes disponibles pour le frontend :

**Enregistrement**
- `start_recording()` → `Result<(), String>`
- `stop_recording_and_transcribe(language, apply_llm_mode)` → `Result<String, String>`
- `is_recording()` → `Result<bool, String>`
- `cancel_recording()` → `Result<(), String>`

**Paramètres**
- `get_settings()` → `Result<Settings, String>`
- `save_settings(settings)` → `Result<(), String>`
- `update_hotkey(new_hotkey)` → `Result<(), String>`
- `update_cancel_key(new_key)` → `Result<(), String>`
- `get_audio_devices()` → `Result<Vec<String>, String>`

**Vocabulaire**
- `get_custom_words()` → `Result<Vec<String>, String>`
- `add_custom_word(word)` → `Result<(), String>`
- `remove_custom_word(word)` → `Result<(), String>`
- `clear_custom_words()` → `Result<(), String>`

**Modèles**
- `list_available_models()` → `Result<Vec<ModelInfo>, String>`
- `download_model(model_name, url)` → `Result<(), String>`
- `delete_model(model_name)` → `Result<(), String>`
- `reload_model(model_name)` → `Result<(), String>`

**LLM**
- `get_llm_models()` → `Result<Vec<LLMModel>, String>`
- `add_llm_model(config)` → `Result<String, String>`
- `update_llm_model(id, config)` → `Result<(), String>`
- `delete_llm_model(id)` → `Result<(), String>`
- `detect_lm_studio_models(port)` → `Result<Vec<...>, String>`
- `detect_ollama_models(port)` → `Result<Vec<...>, String>`
- `check_local_service_status(service_type, port)` → `Result<bool, String>`

**Modes d'exécution**
- `get_execution_modes()` → `Result<Vec<ExecutionMode>, String>`
- `get_active_mode()` → `Result<String, String>`
- `set_active_mode(id)` → `Result<(), String>`
- `add_execution_mode(...)` → `Result<String, String>`
- `update_execution_mode(...)` → `Result<(), String>`
- `delete_execution_mode(id)` → `Result<(), String>`
- `get_indicator_info()` → `Result<IndicatorInfo, String>`

---

## 🔧 Build de production

### Deux versions disponibles

1. **Version CUDA** - Accélération GPU NVIDIA (~3x plus rapide)
2. **Version CPU** - Compatible tous PC Windows

### Scripts de build

```powershell
# Build version CUDA (nécessite CUDA Toolkit 11.x/12.x)
.\build-cuda.ps1
# Produit: release-builds\v0.1.4\flemme-app-cuda-v0.1.4-setup.exe

# Build version CPU (universelle)
.\build-cpu.ps1
# Produit: release-builds\v0.1.4\flemme-app-cpu-v0.1.4-setup.exe

# Générer latest.json pour auto-updater
.\generate-latest-json.ps1
# Produit: release-builds\v0.1.4\latest.json
```

### Configuration build

**Cargo.toml** :
- Feature `cuda` pour whisper-rs avec support CUDA
- Profile release optimisé : `opt-level = 2`, `lto = false`, `codegen-units = 16`
- Console Windows désactivée en production (`debug-assertions = false`)

**tauri.conf.json** :
- Bundle target : NSIS (Nullsoft Installer)
- Installeur en français, installation par utilisateur (`perUser`)
- Auto-updater avec vérification signatures cryptographiques
- Updater endpoint : `https://github.com/ay-bell/Flemme/releases/latest/download/latest.json`

### Workflow de release

1. **Bump version** dans `package.json`, `Cargo.toml`, `tauri.conf.json`
2. **Commit et push** sur main
3. **Build localement** avec scripts PowerShell
4. **Créer release GitHub** (v0.1.X)
5. **Upload assets** :
   - `flemme-app-cuda-vX.X.X-setup.exe`
   - `flemme-app-cpu-vX.X.X-setup.exe`
   - `latest.json`
6. **Tag Git** : `git tag v0.1.X && git push origin v0.1.X`

L'auto-updater vérifiera automatiquement les nouvelles versions au démarrage (throttling 24h).

---

## 📊 Performances

### Benchmarks mesurés

**Configuration test** : Intel i7-10700K @ 3.8GHz, 32GB RAM, RTX 3070

| Opération | CPU (8 threads) | GPU (CUDA) | Notes |
|-----------|-----------------|------------|-------|
| Capture audio 4s | 4000ms | 4000ms | Temps réel obligatoire |
| VAD processing | 15-25ms | 15-25ms | Silero ONNX (CPU) |
| Whisper inference | 180-220ms | 80-120ms | Modèle base Q5 |
| LLM call (Gemini) | 800-1200ms | 800-1200ms | Latence réseau |
| LM Studio local | 300-500ms | 300-500ms | Llama 3.1 8B |
| Copie + paste | 55-70ms | 55-70ms | arboard + enigo |
| **Total (standard)** | **~250-320ms** | **~150-220ms** | Sans temps de parole |
| **Total (avec LLM cloud)** | **~1050-1520ms** | **~950-1420ms** | + latence réseau |

### Comparaison vs Python VoiceToText

| Métrique | Python | Rust/Tauri | Amélioration |
|----------|--------|------------|--------------|
| **Taille installeur** | ~800 MB | 18 MB (NSIS) | **97.8% ↓** |
| **Mémoire (idle)** | ~800 MB | <300 MB | **62.5% ↓** |
| **Mémoire (recording)** | ~1.5 GB | <500 MB | **66.7% ↓** |
| **Transcription 4s** | ~1000ms | 180-220ms (CPU) | **78-82% ↓** |
| **Démarrage app** | 3-5s | <500ms | **83-90% ↓** |
| **Chargement modèle** | ~2s (toujours) | 0ms (lazy) | **100% ↓** |

### Optimisations clés

1. **Lazy loading** - Modèle Whisper chargé seulement au premier usage
2. **Worker threads** - Audio et transcription isolés (pas de blocage UI)
3. **VAD intelligent** - Stratégie adaptative selon % de parole détectée
4. **Padding VAD** - 150ms avant segments pour préserver début de parole
5. **Resampling FFT** - Rubato pour conversion haute qualité vers 16kHz
6. **CUDA acceleration** - ~3x speedup sur GPU NVIDIA
7. **OS Keyring** - Pas de surcharge I/O pour clés API
8. **NSIS installer** - 20x plus petit que MSI grâce à compression

---

## 🗺️ Roadmap

### ✅ v0.1.0 - MVP Complet
- [x] Enregistrement audio avec sélection périphérique
- [x] VAD temps réel avec Silero ONNX
- [x] Intégration Whisper.cpp (CPU + CUDA)
- [x] Raccourcis globaux (push-to-talk + toggle)
- [x] Auto-paste avec simulation Ctrl+V
- [x] Vocabulaire personnalisé (contextual biasing)
- [x] Interface paramètres complète (5 onglets)
- [x] Fenêtre indicateur avec visualisation spectrale
- [x] Persistance configuration JSON

### ✅ v0.1.1-0.1.3 - Intégrations LLM & System Tray
- [x] Support 5 providers LLM (OpenRouter/Gemini/OpenAI/LMStudio/Ollama)
- [x] Modes d'exécution personnalisables
- [x] Stockage sécurisé clés API (OS keyring)
- [x] Auto-détection modèles locaux (LM Studio, Ollama)
- [x] System tray avec menu contextuel
- [x] Changement rapide de mode depuis tray
- [x] Hide to tray au lieu de quit

### ✅ v0.1.4 - Release Production
- [x] Logo paresseux personnalisé (icône + tray)
- [x] Fix bug réouverture fenêtre depuis tray
- [x] Build NSIS avec installeur français
- [x] Auto-updater avec signatures
- [x] Vérification MAJ au démarrage (throttling 24h)
- [x] Console Windows désactivée en production
- [x] Scripts build automatisés (CUDA + CPU)

### 🔄 v0.2.0 - Améliorations UX (En cours)
- [ ] Gestionnaire téléchargement modèles avec barre de progression
- [ ] Assistant premier lancement (wizard)
- [ ] Historique des transcriptions (persistant)
- [ ] Export transcriptions (TXT, MD, JSON)
- [ ] Statistiques d'utilisation (temps enregistré, nombre transcriptions, etc.)

### 📅 v0.3.0 - Internationalisation
- [ ] Localisation UI (EN, ES, DE)
- [ ] Sélection langue UI depuis paramètres
- [ ] Installeur multilingue

### 📅 v0.4.0 - Personnalisation
- [ ] Toggle thème dark/light
- [ ] Customisation couleurs indicateur
- [ ] Sélection position indicateur (coins écran)
- [ ] Templates de modes d'exécution prédéfinis

### 📅 v0.5.0 - Avancé
- [ ] Mode streaming pour longs audios (>60s)
- [ ] Support modèles Whisper custom fine-tunés
- [ ] Détection automatique meilleur modèle selon langue
- [ ] Compression audio avant envoi LLM (économie tokens)

### 📅 v1.0.0 - Cross-platform
- [ ] Support macOS (M1/M2 + Intel)
- [ ] Support Linux (Debian, Fedora, Arch)
- [ ] CI/CD multi-platform
- [ ] Installeurs natifs (.dmg, .deb, .rpm, .AppImage)

### 📅 Futur
- [ ] Intégration services cloud (Dropbox, Google Drive, OneDrive)
- [ ] Synchronisation settings entre machines
- [ ] Plugin system pour extensions tierces
- [ ] API REST locale pour intégration apps tierces
- [ ] Support langues additionnelles (IT, PT, RU, ZH, JA)

---

## 🤝 Contribuer

Les contributions sont les bienvenues ! Voici comment participer :

### Processus
1. **Fork** le projet sur GitHub
2. **Créer une branche** feature : `git checkout -b feature/AmazingFeature`
3. **Commit** vos changements : `git commit -m 'feat: Add AmazingFeature'`
4. **Push** vers la branche : `git push origin feature/AmazingFeature`
5. **Ouvrir une Pull Request** avec description détaillée

### Guidelines
- **Code Rust** : Suivre rustfmt + clippy sans warnings
- **Code TypeScript** : Respecter config ESLint + Prettier
- **Commits** : Format conventionnel (`feat:`, `fix:`, `docs:`, `refactor:`, etc.)
- **Tests** : Ajouter tests unitaires pour nouvelles fonctionnalités
- **Documentation** : Mettre à jour README + commentaires code
- **Performance** : Benchmarker les changements critiques

### Bugs et suggestions
Ouvrir une **issue** sur GitHub en décrivant :
- **Comportement attendu** vs observé
- **Steps to reproduce** (étapes pour reproduire)
- **Logs d'erreur** si applicable (chercher dans `%APPDATA%/Flemme/logs/`)
- **Version** de l'application et système d'exploitation
- **Screenshots** si problème UI

### Développeurs recherchés
Domaines où nous avons besoin d'aide :
- **macOS/Linux support** - Portage cross-platform
- **UI/UX design** - Amélioration interface et expérience utilisateur
- **Tests** - Suite de tests automatisés (unit + intégration)
- **Documentation** - Tutoriels, vidéos, traductions
- **Performance** - Optimisations audio/transcription

---

## 📄 Licence

Ce projet est sous licence **MIT** - voir le fichier [LICENSE](LICENSE) pour plus de détails.

**TL;DR** : Vous pouvez utiliser, modifier, distribuer ce code librement, même commercialement, tant que vous conservez le copyright notice.

---

## 🙏 Remerciements

### Technologies et bibliothèques
- [Tauri](https://tauri.app) - Framework desktop cross-platform Rust+Web
- [whisper.cpp](https://github.com/ggerganov/whisper.cpp) - Implémentation C/C++ de Whisper par Georgi Gerganov
- [whisper-rs](https://github.com/tazz4843/whisper-rs) - Bindings Rust pour whisper.cpp
- [Silero VAD](https://github.com/snakers4/silero-vad) - Détection d'activité vocale légère et rapide
- [Svelte](https://svelte.dev) - Framework UI réactif compilé
- [Tailwind CSS](https://tailwindcss.com) - Framework CSS utility-first
- [cpal](https://github.com/RustAudio/cpal) - Audio I/O cross-platform Rust
- [rubato](https://github.com/HEnquist/rubato) - Resampler audio haute qualité
- [arboard](https://github.com/1Password/arboard) - Clipboard management Rust
- [enigo](https://github.com/enigo-rs/enigo) - Keyboard simulation cross-platform

### Inspiration
Migration et amélioration majeure du projet **VoiceToText** original (Python) vers un stack haute performance Rust/Tauri, avec gains significatifs en vitesse, mémoire et taille de distribution.

### Logo
Icône paresseux 🦥 généré pour représenter l'esprit du projet : *"Parce que taper au clavier, c'est la flemme !"*

---

## 📞 Support

Pour toute question ou problème :
- **Issues** : [GitHub Issues](https://github.com/ay-bell/Flemme/issues)
- **Discussions** : [GitHub Discussions](https://github.com/ay-bell/Flemme/discussions)
- **Email** : Voir profil GitHub pour contact direct

---

**Développé avec ❤️ par Aymeric Bellavoine**

*Flemme - Parce que taper au clavier, c'est la flemme* 🦥
