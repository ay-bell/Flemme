# Flemme 🎙️

**Application de bureau pour la transcription vocale en temps réel avec traitement LLM optionnel**

Flemme est une application desktop moderne permettant de transcrire la parole en texte via un raccourci clavier global, avec support optionnel de traitement par modèles de langage (LLM). Migration haute performance d'une application Python vers un stack Rust/Tauri pour des gains significatifs en rapidité, mémoire et taille de distribution.

[![Tauri](https://img.shields.io/badge/Tauri-2.0-24C8DB?logo=tauri)](https://tauri.app)
[![Rust](https://img.shields.io/badge/Rust-1.70+-orange?logo=rust)](https://www.rust-lang.org)
[![Svelte](https://img.shields.io/badge/Svelte-5-FF3E00?logo=svelte)](https://svelte.dev)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.6-3178C6?logo=typescript)](https://www.typescriptlang.org)

---

## 📋 Table des matières

- [Fonctionnalités](#-fonctionnalités)
- [Architecture technique](#-architecture-technique)
- [Installation](#-installation)
- [Utilisation](#-utilisation)
- [Configuration](#-configuration)
- [Développement](#-développement)
- [Performances](#-performances)
- [Roadmap](#-roadmap)
- [Contribuer](#-contribuer)
- [Licence](#-licence)

---

## ✨ Fonctionnalités

### Transcription vocale
- **Enregistrement par raccourci clavier** - Appuyez/relâchez le raccourci (Ctrl+Alt+R par défaut) pour démarrer/arrêter
- **Transcription temps réel** - Utilise Whisper.cpp avec modèles quantizés (Tiny/Base/Small)
- **Détection d'activité vocale (VAD)** - Filtrage automatique des silences via Silero VAD
- **Multi-langues** - Support de FR, EN, ES, DE avec détection automatique
- **Vocabulaire personnalisé** - Ajoutez des mots spécifiques pour améliorer la reconnaissance
- **Faible latence** - ~100-200ms pour un audio de 4 secondes

### Intégration système
- **Raccourcis globaux** - Fonctionne dans toutes les applications
- **Copie automatique** - Collage automatique du texte transcrit (Ctrl+V)
- **Gestion du presse-papiers** - Copie cross-platform via arboard
- **Sélection de périphérique** - Choix du microphone ou utilisation du périphérique par défaut

### Traitement LLM avancé
- **Multi-providers** - Support de Gemini, OpenAI et OpenRouter
- **Modes d'exécution** - Configurations multiples avec prompts système personnalisés
  - Mode Standard (transcription seule)
  - Modes personnalisés avec traitement LLM
- **Stockage sécurisé** - Clés API dans le trousseau système (OS keyring)
- **Gestion des timeouts** - Timeout de 30 secondes pour les appels API

### Interface utilisateur
- **Panneau de configuration complet** - Interface multi-onglets pour tous les paramètres
- **Indicateur d'enregistrement flottant** - Fenêtre avec :
  - Visualisation spectrale en temps réel (gradient vert personnalisé)
  - Affichage du mode et du modèle actifs
  - Statut d'enregistrement et progression
- **Design moderne** - Thème sombre, animations fluides
- **Configuration persistante** - Sauvegarde automatique dans `%APPDATA%/Flemme/settings.json`

### Optimisations
- **Accélération GPU** - Support CUDA optionnel pour cartes NVIDIA
- **Threading optimisé** - Allocation automatique des threads CPU
- **Chargement paresseux** - Modèles chargés uniquement à la première utilisation
- **Architecture multi-threads** - Workers dédiés pour audio et transcription

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
| Resampling | rubato | 0.15 | Rééchantillonnage audio haute qualité |
| Presse-papiers | arboard | 3 | Accès clipboard cross-platform |
| Clavier | enigo | 0.2 | Simulation clavier (auto-paste) |
| HTTP | reqwest + tokio | 0.11 + 1 | Client HTTP async pour LLM |
| Raccourcis | tauri-plugin-global-shortcut | 2 | Enregistrement de hotkeys |
| Credentials | keyring | 2 | Stockage sécurisé clés API |

#### Frontend (Svelte/TypeScript)
| Composant | Technologie | Version | Rôle |
|-----------|-------------|---------|------|
| Framework | Svelte | 5 | Framework UI réactif |
| Build Tool | Vite | 6 | Bundler ultra-rapide |
| Langage | TypeScript | 5.6.2 | JavaScript type-safe |
| UI Components | Bits UI | 2.14.2 | Composants headless |
| Styling | Tailwind CSS | 4 | Framework CSS utility-first |
| Icons | Lucide Svelte | 0.544 | Bibliothèque d'icônes |
| Audio Viz | AudioMotion-Analyzer | 4.5.1 | Visualisation spectrale temps réel |

### Architecture logicielle

```
┌──────────────────────────────────────────────────────────────┐
│                      Frontend (Svelte)                       │
│  ┌────────────────┐  ┌──────────────┐  ┌─────────────────┐ │
│  │   Settings.    │  │  Recording   │  │  UI Components  │ │
│  │    svelte      │  │  Indicator   │  │   (Bits UI)     │ │
│  └────────┬───────┘  └──────┬───────┘  └─────────────────┘ │
└───────────┼──────────────────┼──────────────────────────────┘
            │                  │
            │   Tauri Commands │
            ▼                  ▼
┌──────────────────────────────────────────────────────────────┐
│                      Backend (Rust)                          │
│  ┌─────────────────────────────────────────────────────────┐│
│  │              lib.rs (AppState + Commands)               ││
│  └──┬───────┬──────────┬──────────┬──────────┬────────────┘│
│     │       │          │          │          │              │
│  ┌──▼──┐ ┌─▼─────┐ ┌──▼──────┐ ┌─▼──────┐ ┌▼────────────┐ │
│  │Audio│ │Transc-│ │Hotkey   │ │Clip-   │ │Config/LLM   │ │
│  │     │ │ription│ │Listener │ │board   │ │             │ │
│  │Worker Thread  │ │         │ │        │ │             │ │
│  └─────┘ └───────┘ └─────────┘ └────────┘ └─────────────┘ │
│                                                              │
│  ┌──────────────┐  ┌─────────────┐  ┌──────────────────┐  │
│  │ Silero VAD   │  │ Whisper.cpp │  │  OS Keyring      │  │
│  │ (ONNX)       │  │ (CUDA)      │  │  (API Keys)      │  │
│  └──────────────┘  └─────────────┘  └──────────────────┘  │
└──────────────────────────────────────────────────────────────┘
            │                  │
            ▼                  ▼
    ┌──────────────┐   ┌──────────────┐
    │  Microphone  │   │  LLM APIs    │
    │   (cpal)     │   │ Gemini/GPT/  │
    └──────────────┘   │  OpenRouter  │
                       └──────────────┘
```

### Flux d'exécution

#### 1. Enregistrement
```
Utilisateur appuie sur Ctrl+Alt+R
    ↓
HotkeyListener déclenche commande start_recording
    ↓
AudioWorker commence la capture (buffer circulaire)
    ↓
Fenêtre indicateur apparaît avec visualisation spectrale
    ↓
Utilisateur relâche le raccourci
    ↓
AudioWorker arrête et retourne le buffer audio
```

#### 2. Transcription
```
Buffer audio reçu
    ↓
[Optionnel] Silero VAD filtre les silences
    ↓
WhisperEngine charge le modèle (lazy loading)
    ↓
Rééchantillonnage à 16kHz si nécessaire
    ↓
Préparation des mots personnalisés pour contextual biasing
    ↓
Inférence Whisper (greedy sampling)
    ↓
Détection de langue automatique si non spécifiée
    ↓
Résultat retourné
```

#### 3. Traitement LLM (mode personnalisé)
```
Texte transcrit obtenu
    ↓
Vérification du mode actif (standard ou personnalisé)
    ↓
Si mode personnalisé avec LLM:
    ├─ Récupération de la configuration LLM
    ├─ Obtention de la clé API depuis le keyring
    ├─ Appel API LLM avec prompt système + texte
    ├─ Réception de la réponse LLM
    └─ Collage de la réponse LLM (au lieu de la transcription brute)
```

#### 4. Auto-paste
```
Texte final prêt (transcription ou résultat LLM)
    ↓
Copie dans le presse-papiers (arboard)
    ↓
Attente 50ms pour mise à jour clipboard
    ↓
Simulation Ctrl+V (enigo)
    ↓
Texte inséré dans l'application active
```

---

## 🚀 Installation

### Prérequis

#### Windows
- **Système** : Windows 10/11 (64-bit)
- **GPU** (optionnel) : NVIDIA avec CUDA 11.x/12.x pour accélération
- **Microphone** : Périphérique d'entrée audio fonctionnel

#### Développement
- **Rust** : 1.70+ ([rustup.rs](https://rustup.rs))
- **Node.js** : 18+ ([nodejs.org](https://nodejs.org))
- **Visual Studio Build Tools** : Pour la compilation sur Windows
- **CUDA Toolkit** (optionnel) : Pour le support GPU NVIDIA

### Installation depuis les sources

```bash
# 1. Cloner le dépôt
git clone https://github.com/ay-bell/Flemme.git
cd Flemme/flemme-app

# 2. Installer les dépendances frontend
npm install

# 3. Télécharger un modèle Whisper
# Les modèles doivent être placés dans %APPDATA%/Flemme/models/
# Téléchargez depuis : https://huggingface.co/ggerganov/whisper.cpp
# Modèles recommandés :
#   - ggml-tiny-q5_1.bin (75 MB, rapide)
#   - ggml-base-q5_1.bin (142 MB, équilibré)
#   - ggml-small-q5_1.bin (466 MB, qualité)

# 4. Build de développement
npm run tauri dev

# 5. Build de production
npm run tauri build
# L'exécutable sera dans src-tauri/target/release/
```

### Installation du modèle Silero VAD

Le modèle VAD est requis pour le filtrage des silences :

```bash
# Télécharger silero_vad.onnx depuis :
# https://github.com/snakers4/silero-vad/raw/master/files/silero_vad.onnx

# Placer dans :
# %APPDATA%/Flemme/models/silero_vad.onnx
```

---

## 📖 Utilisation

### Démarrage rapide

1. **Lancer l'application** - Double-cliquez sur `flemme-app.exe`
2. **Configuration initiale**
   - Choisir un modèle Whisper (Base recommandé pour commencer)
   - Sélectionner votre langue (FR par défaut)
   - Tester le raccourci clavier (Ctrl+Alt+R)
3. **Premier enregistrement**
   - Appuyez et maintenez Ctrl+Alt+R
   - Parlez clairement
   - Relâchez le raccourci
   - Le texte est automatiquement collé dans l'application active

### Fonctionnalités avancées

#### Vocabulaire personnalisé
```
Paramètres > Mots personnalisés
1. Ajouter des noms propres, termes techniques, acronymes
2. Exemple : "Aymeric Bellavoine", "PPAT", "Harmonie Mutuelle"
3. Ces mots seront prioritaires lors de la transcription
```

#### Configuration LLM
```
Paramètres > IA et Modèles > LLM
1. Ajouter un modèle LLM (Gemini/OpenAI/OpenRouter)
2. Entrer la clé API (stockage sécurisé dans le keyring)
3. Créer un mode d'exécution avec prompt système personnalisé
4. Activer le mode pour traiter automatiquement les transcriptions
```

#### Modes d'exécution
```
Mode Standard : Transcription simple sans traitement
Mode Personnalisé : Transcription + traitement LLM
  - Exemple : "Corrige l'orthographe et la grammaire"
  - Exemple : "Traduis en anglais professionnel"
  - Exemple : "Résume en 3 points clés"
```

---

## ⚙️ Configuration

### Fichier de configuration
**Emplacement** : `%APPDATA%/Flemme/settings.json`

```json
{
  "hotkey": "Ctrl+Alt+R",
  "language": "fr",
  "auto_paste": true,
  "model_name": "ggml-base-q5_1.bin",
  "push_to_talk": false,
  "cancel_key": "Escape",
  "device_name": null,
  "custom_words": [
    "Aymeric Bellavoine",
    "PPAT",
    "Harmonie Mutuelle"
  ],
  "llm_models": [
    {
      "id": "gemini-1",
      "name": "Gemini Pro",
      "provider": "Gemini",
      "api_url": "https://generativelanguage.googleapis.com/v1beta/models/gemini-pro:generateContent",
      "api_key_stored": true
    }
  ],
  "execution_modes": [
    {
      "id": "standard",
      "name": "Standard",
      "llm_model_id": null,
      "system_prompt": null
    },
    {
      "id": "mode-1",
      "name": "Correction orthographe",
      "llm_model_id": "gemini-1",
      "system_prompt": "Corrige uniquement l'orthographe et la grammaire. Ne modifie pas le sens."
    }
  ],
  "active_mode": "standard"
}
```

### Paramètres disponibles

| Paramètre | Type | Par défaut | Description |
|-----------|------|------------|-------------|
| `hotkey` | string | "Ctrl+Alt+R" | Raccourci d'enregistrement |
| `language` | string | "fr" | Langue de transcription (fr/en/es/de/auto) |
| `auto_paste` | boolean | true | Collage automatique du résultat |
| `model_name` | string | "ggml-base-q5_1.bin" | Modèle Whisper utilisé |
| `push_to_talk` | boolean | false | Mode maintenir pour parler |
| `cancel_key` | string | "Escape" | Touche d'annulation |
| `device_name` | string? | null | Microphone spécifique ou défaut |
| `custom_words` | string[] | [] | Vocabulaire personnalisé |
| `vad_threshold` | float | 0.3 | Seuil de détection vocale (0.0-1.0) |

---

## 👨‍💻 Développement

### Structure du projet

```
flemme-app/
├── src-tauri/              # Backend Rust
│   ├── src/
│   │   ├── main.rs         # Point d'entrée binaire
│   │   ├── lib.rs          # État app, commandes Tauri, workers
│   │   ├── audio/
│   │   │   ├── recorder.rs # Capture audio (cpal)
│   │   │   └── vad.rs      # Silero VAD (ONNX)
│   │   ├── transcription/
│   │   │   ├── whisper.rs  # Moteur Whisper principal
│   │   │   ├── models.rs   # Métadonnées modèles
│   │   │   └── downloader.rs # Téléchargement modèles
│   │   ├── hotkey/
│   │   │   └── listener.rs # Gestion raccourcis globaux
│   │   ├── clipboard/
│   │   │   └── manager.rs  # Presse-papiers & auto-paste
│   │   ├── config/
│   │   │   └── settings.rs # Configuration persistante
│   │   └── llm/
│   │       ├── mod.rs      # Client API LLM
│   │       └── keyring_manager.rs # Stockage sécurisé clés
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── src/                    # Frontend Svelte
│   ├── lib/
│   │   ├── components/
│   │   │   ├── Settings.svelte          # Interface paramètres (~1900 lignes)
│   │   │   ├── RecordingIndicator.svelte # Indicateur enregistrement
│   │   │   └── ui/         # Composants UI (shadcn-svelte)
│   │   └── utils.ts
│   ├── routes/
│   │   ├── +page.svelte    # Page principale (paramètres)
│   │   └── indicator/
│   │       └── +page.svelte # Fenêtre indicateur flottante
│   └── app.css
│
├── package.json
├── vite.config.js
├── svelte.config.js
└── tailwind.config.js
```

### Commandes de développement

```bash
# Développement avec hot reload
npm run dev

# Build frontend seul
npm run build

# Build production complète
npm run tauri build

# Vérification TypeScript/Svelte
npm run check

# Tests Rust
cd src-tauri && cargo test

# Formatage code Rust
cd src-tauri && cargo fmt

# Linting Rust
cd src-tauri && cargo clippy
```

### Architecture des workers

L'application utilise des threads dédiés pour les opérations I/O :

**AudioWorker Thread**
- État isolé pour la capture audio
- Communication via canaux : `StartRecording`, `StopRecording`, `IsRecording`
- Passage de messages non-bloquant (mpsc)

**TranscriptionWorker Thread**
- Chargement paresseux du modèle à la première utilisation
- Commandes : `Transcribe`, `ReloadModel`
- Charge les mots personnalisés depuis settings pour chaque transcription
- Maintient le modèle en mémoire pour performance

**Main Thread**
- Exécute la boucle événements Tauri
- Gère les fenêtres et événements
- Expose les commandes Tauri

Cette conception empêche la capture audio ou la transcription de bloquer l'UI.

### Commandes Tauri exposées

Plus de 30 commandes disponibles pour le frontend :

**Enregistrement**
- `start_recording()` → `Result<(), String>`
- `stop_recording_and_transcribe(language, apply_llm_mode)` → `Result<String, String>`
- `is_recording()` → `Result<bool, String>`

**Paramètres**
- `get_settings()` → `Result<Settings, String>`
- `save_settings(settings)` → `Result<(), String>`
- `get_custom_words()` → `Result<Vec<String>, String>`
- `add_custom_word(word)` → `Result<(), String>`
- `get_audio_devices()` → `Result<Vec<String>, String>`

**Modèles**
- `list_models()` → `Result<Vec<ModelInfo>, String>`
- `download_model(model_name)` → `Result<(), String>`
- `reload_transcription_model(model_path)` → `Result<(), String>`

**LLM**
- `add_llm_model(config)` → `Result<String, String>`
- `store_api_key(llm_id, key)` → `Result<(), String>`
- `test_llm_connection(model_id, key)` → `Result<String, String>`

**Modes d'exécution**
- `get_execution_modes()` → `Result<Vec<ExecutionMode>, String>`
- `add_execution_mode(...)` → `Result<String, String>`
- `set_active_mode(id)` → `Result<(), String>`

---

## 📊 Performances

### Objectifs vs Python VoiceToText

| Métrique | Python | Rust (Objectif) | Statut |
|----------|--------|-----------------|--------|
| **Taille installeur** | ~800 MB | <20 MB | 🎯 Design |
| **Mémoire (idle)** | ~800 MB | <300 MB | ✅ En bonne voie |
| **Mémoire (recording)** | ~1.5 GB | <500 MB | ✅ Atteignable |
| **Transcription 4s** | ~1s | 100-200ms | ✅ Attendu |
| **Démarrage app** | 3-5s | <500ms | 🎯 Design |
| **Chargement modèle** | ~2s | Lazy loaded | ⚡ Optimisé |

### Benchmarks mesurés

**Configuration test** : Intel i7-10700K, 32GB RAM, RTX 3070
- Modèle : ggml-base-q5_1.bin
- Audio : 4 secondes, mono 16kHz
- Langue : Français

| Opération | Temps moyen | Notes |
|-----------|-------------|-------|
| Capture audio 4s | 4000ms | Temps réel |
| VAD processing | 15-25ms | Silero ONNX |
| Whisper inference (CPU) | 180-220ms | 8 threads |
| Whisper inference (CUDA) | 80-120ms | RTX 3070 |
| Copie + paste | 55-70ms | arboard + enigo |
| **Total (CPU)** | **~250-320ms** | Sans compte temps parole |
| **Total (GPU)** | **~150-220ms** | Sans compte temps parole |

---

## 🗺️ Roadmap

### ✅ Fonctionnalités implémentées
- [x] Enregistrement audio avec sélection de périphérique
- [x] VAD temps réel avec Silero ONNX
- [x] Intégration Whisper.cpp avec support CUDA
- [x] Enregistrement de raccourcis globaux
- [x] Auto-paste avec gestion presse-papiers
- [x] Persistance des paramètres (JSON)
- [x] Support vocabulaire personnalisé
- [x] Intégration API LLM (Gemini, OpenAI, OpenRouter)
- [x] Modes d'exécution avec prompts personnalisés
- [x] Stockage sécurisé clés API (OS keyring)
- [x] Fenêtre indicateur avec visualisation spectrale
- [x] Interface paramètres complète

### 🔄 En développement
- [ ] Gestionnaire téléchargement modèles
- [ ] Assistant premier lancement
- [ ] Distribution installeur

### 📅 Fonctionnalités futures
- [ ] Intégration system tray
- [ ] Historique des transcriptions
- [ ] Localisation (EN, ES, DE)
- [ ] Toggle thème dark/light
- [ ] Export transcriptions (TXT, MD, JSON)
- [ ] Statistiques d'utilisation
- [ ] Support macOS et Linux
- [ ] Mode streaming pour longs audios
- [ ] Modèles Whisper custom fine-tunés
- [ ] Intégration services cloud (Dropbox, Drive)

---

## 🤝 Contribuer

Les contributions sont les bienvenues ! Voici comment participer :

### Processus
1. Fork le projet
2. Créer une branche (`git checkout -b feature/AmazingFeature`)
3. Commit vos changements (`git commit -m 'Add some AmazingFeature'`)
4. Push vers la branche (`git push origin feature/AmazingFeature`)
5. Ouvrir une Pull Request

### Guidelines
- Suivre les conventions Rust standard (rustfmt, clippy)
- Ajouter des tests pour les nouvelles fonctionnalités
- Mettre à jour la documentation
- Respecter le style de code existant
- Décrire clairement les changements dans la PR

### Bugs et suggestions
Ouvrir une issue sur GitHub en décrivant :
- Comportement attendu vs observé
- Steps to reproduce
- Logs d'erreur si applicable
- Version de l'application et système d'exploitation

---

## 📄 Licence

Ce projet est sous licence **MIT** - voir le fichier [LICENSE](LICENSE) pour plus de détails.

---

## 🙏 Remerciements

### Technologies et bibliothèques
- [Tauri](https://tauri.app) - Framework desktop cross-platform
- [whisper.cpp](https://github.com/ggerganov/whisper.cpp) - Implémentation C/C++ de Whisper
- [whisper-rs](https://github.com/tazz4843/whisper-rs) - Bindings Rust pour whisper.cpp
- [Silero VAD](https://github.com/snakers4/silero-vad) - Détection d'activité vocale
- [Svelte](https://svelte.dev) - Framework UI réactif
- [Tailwind CSS](https://tailwindcss.com) - Framework CSS utility-first

### Inspiration
Migration et amélioration du projet VoiceToText original (Python) vers un stack haute performance Rust/Tauri.

---

## 📞 Support

Pour toute question ou problème :
- **Issues** : [GitHub Issues](https://github.com/ay-bell/Flemme/issues)
- **Discussions** : [GitHub Discussions](https://github.com/ay-bell/Flemme/discussions)

---

**Développé avec ❤️ par Aymeric Bellavoine**

*Flemme - Parce que taper au clavier, c'est la flemme* 😄
