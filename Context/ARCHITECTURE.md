# Flemme - Architecture Technique

> **Migration de VoiceToText (Python) vers Flemme (Rust + Tauri)**
> 
> Objectif : Améliorer drastiquement les performances en utilisant whisper.cpp au lieu de faster-whisper/PyTorch

---

## Vue d'ensemble

**Flemme** est une application de transcription vocale instantanée pour Windows, construite avec :
- **Backend** : Rust + Tauri 2.0
- **Frontend** : Svelte 5 + Tailwind + shadcn-svelte
- **Transcription** : whisper.cpp (bindings Rust)
- **Distribution** : Installeur Windows (.exe) avec download manager de modèles

---

## Stack technique détaillée

### Backend (Rust)

| Composant | Crate / Technologie | Justification |
|-----------|---------------------|---------------|
| **Framework** | Tauri 2.0 | Framework app desktop moderne, léger, sécurisé |
| **Audio recording** | `cpal` | Bibliothèque audio cross-platform, bas niveau |
| **Transcription** | `whisper-rs` ou bindings directs whisper.cpp | Performance native C++, jusqu'à 10x plus rapide que Python |
| **Hotkeys globaux** | `tauri-plugin-global-shortcut` | Plugin officiel Tauri, gère bien les modificateurs |
| **Clipboard** | `arboard` | Clipboard cross-platform simple et efficace |
| **Config** | `serde` + `toml` ou `json` | Sérialisation config utilisateur |
| **Logging** | `tracing` + `tracing-subscriber` | Logging moderne et performant |
| **HTTP downloads** | `reqwest` + `tokio` | Pour download des modèles Whisper |

### Frontend (Svelte)

| Composant | Technologie | Justification |
|-----------|-------------|---------------|
| **Framework** | Svelte 5 | Lightweight, réactif, bundle petit |
| **Styling** | Tailwind CSS | Utility-first, rapide à développer |
| **Components** | shadcn-svelte | Composants modernes accessibles |
| **Build** | Vite | Build ultra-rapide |
| **State** | Svelte stores | Réactivité native, pas besoin de Redux |

---

## Architecture modulaire

```
Flemme/
├── src-tauri/                      # Backend Rust
│   ├── src/
│   │   ├── main.rs                 # Point d'entrée
│   │   │
│   │   ├── audio/
│   │   │   ├── mod.rs              # Module audio
│   │   │   ├── recorder.rs         # Enregistrement audio
│   │   │   └── vad.rs              # Voice Activity Detection
│   │   │
│   │   ├── transcription/
│   │   │   ├── mod.rs              # Module transcription
│   │   │   ├── engine.rs           # Wrapper whisper.cpp
│   │   │   ├── models.rs           # Gestion modèles (cache, download)
│   │   │   └── downloader.rs       # Download manager modèles
│   │   │
│   │   ├── hotkey/
│   │   │   ├── mod.rs              # Module hotkeys
│   │   │   └── listener.rs         # Listener global shortcuts
│   │   │
│   │   ├── clipboard/
│   │   │   ├── mod.rs              # Module clipboard
│   │   │   └── manager.rs          # Copy + auto-paste
│   │   │
│   │   ├── config/
│   │   │   ├── mod.rs              # Module config
│   │   │   └── settings.rs         # Chargement/sauvegarde config
│   │   │
│   │   ├── commands.rs             # Tauri commands (API frontend)
│   │   └── utils.rs                # Utilitaires divers
│   │
│   ├── Cargo.toml                  # Dépendances Rust
│   ├── tauri.conf.json             # Config Tauri (window, installeur, etc.)
│   ├── build.rs                    # Build script (linking whisper.cpp)
│   └── icons/                      # Icônes app
│
├── src/                            # Frontend Svelte
│   ├── lib/
│   │   ├── components/
│   │   │   ├── ui/                 # shadcn-svelte components
│   │   │   ├── Settings.svelte     # Page settings
│   │   │   ├── RecordingBar.svelte # Barre flottante enregistrement
│   │   │   └── FirstLaunch.svelte  # Wizard premier lancement
│   │   │
│   │   ├── stores/
│   │   │   ├── config.ts           # Store config app
│   │   │   └── recording.ts        # Store état enregistrement
│   │   │
│   │   └── api/
│   │       └── tauri.ts            # Wrapper API Tauri commands
│   │
│   ├── App.svelte                  # Composant racine
│   ├── main.ts                     # Point d'entrée frontend
│   └── app.css                     # Styles globaux (Tailwind)
│
├── public/                         # Assets statiques
├── package.json                    # Dépendances npm
├── vite.config.ts                  # Config Vite
├── tailwind.config.js              # Config Tailwind
├── tsconfig.json                   # Config TypeScript
└── README.md                       # Documentation utilisateur
```

---

## Composants clés - Backend Rust

### 1. Audio Recorder (`audio/recorder.rs`)

**Responsabilités :**
- Capture audio depuis le microphone par défaut
- Format : mono 16kHz (requis par Whisper)
- Buffer circulaire pour streaming
- Start/stop non-bloquant

**API :**
```rust
pub struct AudioRecorder {
    stream: Option<Stream>,
    buffer: Arc<Mutex<Vec<f32>>>,
    sample_rate: u32,
}

impl AudioRecorder {
    pub fn new(sample_rate: u32) -> Result<Self>;
    pub fn start_recording(&mut self) -> Result<()>;
    pub fn stop_recording(&mut self) -> Result<Vec<f32>>;
    pub fn is_recording(&self) -> bool;
}
```

**Crates :**
- `cpal` : Audio capture cross-platform
- `hound` : Écriture fichiers WAV (optionnel, pour debug)

---

### 2. VAD Filter (`audio/vad.rs`)

**Responsabilités :**
- Détection Voice Activity (parole vs silence)
- Découpe segments de parole
- Évite les hallucinations Whisper sur longs silences

**API :**
```rust
pub struct VADFilter {
    threshold: f32,
    min_speech_duration_ms: u32,
    min_silence_duration_ms: u32,
}

impl VADFilter {
    pub fn new(threshold: f32, ...) -> Self;
    pub fn extract_speech_segments(&self, audio: &[f32]) -> Vec<AudioSegment>;
    pub fn has_speech(&self, audio: &[f32]) -> bool;
}
```

**Implémentation :**
- Utiliser un modèle VAD léger (ex: Silero VAD via ONNX)
- Ou algorithme simple basé sur énergie RMS + zero-crossing rate

---

### 3. Transcription Engine (`transcription/engine.rs`)

**Responsabilités :**
- Wrapper autour de whisper.cpp
- Détection automatique device (CPU/GPU)
- Configuration des hyperparamètres

**API :**
```rust
pub struct TranscriptionEngine {
    model: WhisperModel,
    language: String,
    device: Device,
}

impl TranscriptionEngine {
    pub fn new(model_path: &Path, language: &str, device: Device) -> Result<Self>;
    pub fn transcribe(&self, audio: &[f32]) -> Result<TranscriptionResult>;
    pub fn get_device_info(&self) -> DeviceInfo;
}

pub struct TranscriptionResult {
    pub text: String,
    pub language: String,
    pub language_probability: f32,
    pub duration_ms: u64,
}
```

**Bindings whisper.cpp :**
- Utiliser `whisper-rs` crate (bindings Rust communautaires)
- Ou créer des bindings FFI directs si besoin de contrôle fin

---

### 4. Model Manager (`transcription/models.rs`)

**Responsabilités :**
- Gestion cache des modèles Whisper
- Stockage dans `%APPDATA%/Flemme/models/`
- Validation intégrité (checksums)

**API :**
```rust
pub struct ModelManager {
    models_dir: PathBuf,
}

impl ModelManager {
    pub fn new() -> Result<Self>;
    pub fn list_installed_models() -> Vec<ModelInfo>;
    pub fn is_model_installed(&self, model_name: &str) -> bool;
    pub fn get_model_path(&self, model_name: &str) -> Option<PathBuf>;
}

pub struct ModelInfo {
    pub name: String,      // "tiny", "base", "small", etc.
    pub size_mb: u64,
    pub path: PathBuf,
}
```

---

### 5. Model Downloader (`transcription/downloader.rs`)

**Responsabilités :**
- Téléchargement des modèles depuis Hugging Face ou autre
- Progress bar pour l'UI
- Retry en cas d'échec
- Validation après download

**API :**
```rust
pub struct ModelDownloader {
    client: reqwest::Client,
}

impl ModelDownloader {
    pub fn new() -> Self;
    pub async fn download_model(
        &self, 
        model_name: &str,
        on_progress: impl Fn(u64, u64) // (downloaded, total)
    ) -> Result<PathBuf>;
}
```

**URLs des modèles :**
- Tiny : ~75 MB
- Base : ~145 MB
- Small : ~466 MB
- Medium : ~1.5 GB
- Large : ~3 GB

---

### 6. Hotkey Listener (`hotkey/listener.rs`)

**Responsabilités :**
- Enregistrement de hotkeys globaux
- Callbacks press/release pour mode press-to-record
- Support modificateurs (Ctrl, Alt, Shift)

**API :**
```rust
pub struct HotkeyListener {
    hotkey: String,
}

impl HotkeyListener {
    pub fn new(hotkey: &str) -> Result<Self>;
    pub fn register<F>(&self, on_press: F, on_release: F) -> Result<()>
        where F: Fn() + Send + 'static;
    pub fn unregister(&self) -> Result<()>;
}
```

**Tauri integration :**
```rust
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

app.global_shortcut()
    .on_shortcut("Ctrl+Space", |app, event| {
        match event.state {
            ShortcutState::Pressed => on_press(),
            ShortcutState::Released => on_release(),
        }
    })
    .build()?;
```

---

### 7. Clipboard Manager (`clipboard/manager.rs`)

**Responsabilités :**
- Copie texte dans clipboard
- Auto-paste optionnel (simulation Ctrl+V)
- Délai configurable avant paste

**API :**
```rust
pub struct ClipboardManager {
    auto_paste: bool,
    paste_delay_ms: u64,
}

impl ClipboardManager {
    pub fn new(auto_paste: bool, paste_delay_ms: u64) -> Self;
    pub fn copy(&self, text: &str) -> Result<()>;
    pub fn copy_and_paste(&self, text: &str) -> Result<()>;
}
```

**Crates :**
- `arboard` : Clipboard cross-platform
- `enigo` : Simulation clavier pour auto-paste

---

### 8. Config Manager (`config/settings.rs`)

**Responsabilités :**
- Chargement/sauvegarde config utilisateur
- Fichier `%APPDATA%/Flemme/config.toml`
- Validation et valeurs par défaut

**Structure config :**
```toml
# config.toml

[general]
hotkey = "ctrl+space"
language = "fr"
auto_paste = true
show_notifications = true

[audio]
sample_rate = 16000
max_duration_sec = 300

[transcription]
model = "base"
device = "auto"  # "auto", "cpu", "cuda"
temperature = 0.0
no_repeat_ngram_size = 0
repetition_penalty = 1.2

[vad]
enabled = true
threshold = 0.7
min_speech_duration_ms = 250
min_silence_duration_ms = 500
padding_ms = 100

[vocabulary]
custom_words = [
    "Flemme",
    "Python",
    "Rust",
    "Tauri"
]
```

**API :**
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub general: GeneralConfig,
    pub audio: AudioConfig,
    pub transcription: TranscriptionConfig,
    pub vad: VADConfig,
    pub vocabulary: VocabularyConfig,
}

impl AppConfig {
    pub fn load() -> Result<Self>;
    pub fn save(&self) -> Result<()>;
    pub fn default() -> Self;
}
```

---

### 9. Tauri Commands (`commands.rs`)

**Responsabilités :**
- API entre frontend (Svelte) et backend (Rust)
- Commandes invocables depuis JS via `invoke()`

**Exemples de commandes :**
```rust
// Récupérer la config
#[tauri::command]
async fn get_config() -> Result<AppConfig, String> {
    AppConfig::load().map_err(|e| e.to_string())
}

// Sauvegarder la config
#[tauri::command]
async fn save_config(config: AppConfig) -> Result<(), String> {
    config.save().map_err(|e| e.to_string())
}

// Lister les modèles installés
#[tauri::command]
async fn list_models() -> Result<Vec<ModelInfo>, String> {
    Ok(ModelManager::new()?.list_installed_models())
}

// Télécharger un modèle
#[tauri::command]
async fn download_model(
    model_name: String,
    window: tauri::Window
) -> Result<(), String> {
    let downloader = ModelDownloader::new();
    downloader.download_model(&model_name, |downloaded, total| {
        // Émettre événement vers frontend pour progress bar
        window.emit("download-progress", (downloaded, total)).ok();
    }).await.map_err(|e| e.to_string())
}

// Démarrer l'enregistrement (test manuel depuis UI)
#[tauri::command]
async fn start_recording(state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.recorder.lock().unwrap().start_recording()
        .map_err(|e| e.to_string())
}

// Arrêter l'enregistrement et transcrire
#[tauri::command]
async fn stop_recording_and_transcribe(
    state: tauri::State<'_, AppState>
) -> Result<String, String> {
    // ... logique de transcription ...
    Ok(text)
}
```

---

## Composants clés - Frontend Svelte

### 1. Settings Page (`Settings.svelte`)

**Responsabilités :**
- Interface de configuration
- Formulaire avec validation
- Sauvegarde automatique ou manuelle

**Composants shadcn-svelte utilisés :**
- `Input` : Hotkey, vocabulaire
- `Select` : Modèle, langue, device
- `Switch` : Auto-paste, notifications, VAD
- `Slider` : Seuils VAD, températures
- `Button` : Sauvegarder, annuler
- `Dialog` : Confirmation download modèle

**Exemple structure :**
```svelte
<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Select } from '$lib/components/ui/select';
  import { Switch } from '$lib/components/ui/switch';
  import { invoke } from '@tauri-apps/api/core';
  
  let config = $state<AppConfig>();
  
  onMount(async () => {
    config = await invoke('get_config');
  });
  
  async function saveConfig() {
    await invoke('save_config', { config });
  }
</script>

<div class="p-6 max-w-2xl mx-auto">
  <h1 class="text-2xl font-bold mb-6">Paramètres</h1>
  
  <!-- Sections : Général, Audio, Transcription, VAD, Vocabulaire -->
</div>
```

---

### 2. Recording Bar (`RecordingBar.svelte`)

**Responsabilités :**
- Barre flottante pendant enregistrement
- Affichage durée en temps réel
- Animation visuelle

**Props :**
- `visible: boolean` : Afficher/masquer
- `duration: number` : Durée en secondes
- `modelName: string` : Modèle utilisé
- `hotkey: string` : Hotkey pour arrêter

**Exemple :**
```svelte
<script lang="ts">
  import { fade } from 'svelte/transition';
  
  let { visible, duration, modelName, hotkey } = $props<{
    visible: boolean;
    duration: number;
    modelName: string;
    hotkey: string;
  }>();
</script>

{#if visible}
  <div 
    class="fixed top-4 right-4 bg-red-500 text-white px-4 py-2 rounded-lg shadow-lg"
    transition:fade
  >
    <div class="flex items-center gap-2">
      <span class="animate-pulse">🔴</span>
      <span>{duration}s</span>
      <span class="text-sm opacity-75">({modelName})</span>
    </div>
    <div class="text-xs opacity-75 mt-1">
      Relâchez {hotkey} pour arrêter
    </div>
  </div>
{/if}
```

---

### 3. First Launch Wizard (`FirstLaunch.svelte`)

**Responsabilités :**
- Interface première utilisation
- Sélection du modèle à télécharger
- Progress bar download

**Steps :**
1. Bienvenue + explication
2. Choix du modèle (tiny/base/small)
3. Téléchargement avec progress
4. Configuration initiale (hotkey, langue)
5. Prêt à l'emploi !

**Événements Tauri écoutés :**
```typescript
import { listen } from '@tauri-apps/api/event';

listen('download-progress', (event) => {
  const { downloaded, total } = event.payload;
  const percent = (downloaded / total) * 100;
  // Mettre à jour la progress bar
});
```

---

### 4. Stores Svelte (`stores/`)

**config.ts :**
```typescript
import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export const config = writable<AppConfig | null>(null);

export async function loadConfig() {
  const cfg = await invoke<AppConfig>('get_config');
  config.set(cfg);
}

export async function saveConfig(cfg: AppConfig) {
  await invoke('save_config', { config: cfg });
  config.set(cfg);
}
```

**recording.ts :**
```typescript
import { writable } from 'svelte/store';

export const isRecording = writable(false);
export const recordingDuration = writable(0);
```

---

## Flow d'exécution - POC

### Startup

```
1. Démarrage app
   ├─> Charger config depuis %APPDATA%/Flemme/config.toml
   ├─> Initialiser logger
   ├─> Vérifier modèle installé
   │   ├─> Si non : Afficher FirstLaunch wizard
   │   └─> Si oui : Continuer
   ├─> Initialiser AudioRecorder
   ├─> Initialiser TranscriptionEngine (charger modèle)
   ├─> Enregistrer hotkey global (tauri-plugin-global-shortcut)
   └─> Afficher système tray icon (minimized)
```

### Recording Flow

```
1. User presse Ctrl+Space (hotkey)
   ├─> Callback on_press
   ├─> AudioRecorder.start_recording()
   ├─> Émettre événement "recording-started" vers frontend
   └─> Frontend affiche RecordingBar

2. User maintient le hotkey (enregistrement en cours)
   ├─> Audio capturé en continu dans buffer
   └─> Timer durée mis à jour (frontend)

3. User relâche Ctrl+Space
   ├─> Callback on_release
   ├─> AudioRecorder.stop_recording() → Vec<f32>
   ├─> Émettre événement "recording-stopped" vers frontend
   ├─> Frontend masque RecordingBar
   └─> Passer à la transcription
```

### Transcription Flow

```
1. Audio brut récupéré (Vec<f32>)
   ├─> Si VAD activé :
   │   ├─> VADFilter.extract_speech_segments()
   │   ├─> Vérifier has_speech
   │   │   ├─> Si non : Annuler, notifier "Aucune parole détectée"
   │   │   └─> Si oui : Continuer avec segments découpés
   │   └─> Log ratio parole/silence
   └─> Si VAD désactivé : Utiliser audio complet

2. Transcription
   ├─> TranscriptionEngine.transcribe(audio)
   ├─> Whisper génère le texte
   └─> Récupérer TranscriptionResult { text, language, ... }

3. Post-transcription
   ├─> Log texte, langue, durée
   ├─> ClipboardManager.copy_and_paste(text)
   │   ├─> Copier dans clipboard
   │   └─> Si auto_paste : Simuler Ctrl+V
   ├─> Émettre événement "transcription-done" vers frontend
   └─> Frontend affiche notification (optionnel)
```

---

## Distribution & Installation

### Installeur Windows (NSIS ou WiX)

**Contenu :**
- Binaire principal : `Flemme.exe` (~10-15 MB)
- Ressources : Icônes, assets
- Pas de modèle inclus (trop lourd)

**Installation :**
```
C:\Program Files\Flemme\
├── Flemme.exe
├── resources\
└── uninstall.exe
```

**Données utilisateur :**
```
%APPDATA%\Flemme\
├── config.toml
├── models\
│   ├── ggml-base.bin
│   └── ggml-small.bin
└── logs\
    └── flemme.log
```

### First Launch Wizard

**Au premier lancement :**
1. Afficher dialog "Bienvenue dans Flemme !"
2. Expliquer fonctionnement
3. Proposer choix du modèle :
   - Tiny (75 MB) - Ultra rapide, qualité correcte
   - **Base (145 MB) - Recommandé** ⭐
   - Small (466 MB) - Très bonne qualité
4. Download avec progress bar
5. Configuration initiale :
   - Hotkey : `ctrl+space` (par défaut)
   - Langue : Détection auto ou sélection
6. Prêt !

---

## Performances attendues

### Comparaison Python (VoiceToText) vs Rust (Flemme)

| Métrique | Python | Rust (whisper.cpp) | Gain |
|----------|--------|-------------------|------|
| **Taille installeur** | ~800 MB | ~15 MB | **98% plus léger** |
| **Mémoire RAM** | ~1.5 GB | ~300 MB | **80% moins** |
| **Temps transcription (4s audio, base)** | ~1s | ~100-200ms | **5-10x plus rapide** |
| **Démarrage app** | ~3-5s | ~500ms | **6-10x plus rapide** |
| **Taille exécutable** | ~5 MB (+ PyInstaller) | ~5-8 MB | Comparable |

### Optimisations whisper.cpp

- **CPU** : AVX2, FMA, quantization int8
- **GPU** : CUDA ou Metal (optionnel)
- **Modèles quantifiés** : Q5_0, Q8_0 pour réduire encore la taille

---

## Sécurité

### Tauri Security Features

- **Content Security Policy (CSP)** : Bloque XSS
- **Allowlist API** : Seules les commandes explicitement exposées sont appelables
- **No eval()** : Pas d'exécution de code arbitraire
- **Process isolation** : Frontend et backend séparés

### Best Practices

- Valider toutes les entrées utilisateur
- Sanitiser paths avant accès filesystem
- Limiter durée max enregistrement (300s par défaut)
- Pas d'exécution de code depuis config.toml

---

## Tests

### Tests unitaires Rust

```bash
cargo test
```

**Modules à tester :**
- `audio/recorder.rs` : Mock audio input
- `transcription/engine.rs` : Mock whisper.cpp
- `config/settings.rs` : Load/save config
- `clipboard/manager.rs` : Mock clipboard

### Tests d'intégration

**Scénarios :**
1. Enregistrement → Transcription → Clipboard
2. Download modèle → Vérification intégrité
3. Changement config → Reload app
4. Hotkey press/release

### Tests manuels

- [ ] Enregistrement audio de différentes durées (1s, 10s, 60s)
- [ ] Transcription multilingue (fr, en, es)
- [ ] VAD sur audio avec silences
- [ ] Auto-paste dans différentes apps (Notepad, Word, Chrome)
- [ ] Changement de modèle à chaud
- [ ] Installation sur machine propre

---

## Logging & Debug

### Niveaux de log

```rust
use tracing::{info, warn, error, debug};

info!("🚀 Flemme démarré");
debug!("🎤 Enregistrement : {} samples", buffer.len());
warn!("⚠️ VAD: Aucune parole détectée");
error!("❌ Erreur transcription: {}", err);
```

### Fichier log

```
%APPDATA%\Flemme\logs\flemme.log
```

**Rotation :** 1 fichier par jour, max 7 jours conservés

---

## Roadmap technique

### Sprint 0 - POC (Priority 1)
- [x] Architecture documentée
- [ ] Setup projet Tauri + Svelte
- [ ] Intégration whisper.cpp (bindings)
- [ ] Enregistrement audio (cpal)
- [ ] Hotkey global (tauri-plugin-global-shortcut)
- [ ] Transcription basique (modèle base)
- [ ] Auto-paste (arboard + enigo)
- [ ] Validation perfs vs Python

### Phase 2 - Distribution (Priority 2)
- [ ] Download manager modèles
- [ ] First Launch wizard
- [ ] Installeur Windows (NSIS)
- [ ] Configuration système tray
- [ ] Tests installation sur machine propre

### Phase 3 - Features (Priority 3)
- [ ] Settings GUI complète
- [ ] Barre flottante enregistrement
- [ ] Notifications système
- [ ] VAD (Voice Activity Detection)
- [ ] Support vocabulaire personnalisé
- [ ] Paramètres transcription avancés

### Phase 4 - Polish (Priority 4)
- [ ] Localisation EN/FR
- [ ] Thèmes (dark/light)
- [ ] Raccourcis clavier multiples
- [ ] Profils de configuration
- [ ] Historique des transcriptions

---

## Ressources & Liens

**Tauri :**
- [Tauri 2.0 Docs](https://v2.tauri.app/)
- [Tauri Plugin Global Shortcut](https://v2.tauri.app/plugin/global-shortcut/)

**whisper.cpp :**
- [GitHub whisper.cpp](https://github.com/ggerganov/whisper.cpp)
- [Bindings Rust whisper-rs](https://github.com/tazz4843/whisper-rs)

**Svelte :**
- [Svelte 5 Docs](https://svelte-5-preview.vercel.app/)
- [shadcn-svelte](https://www.shadcn-svelte.com/)

**Audio :**
- [cpal](https://docs.rs/cpal/)
- [Silero VAD](https://github.com/snakers4/silero-vad)

---

*Dernière mise à jour : 30 octobre 2025*
