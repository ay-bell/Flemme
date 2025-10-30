# Flemme - Sprint 0 : POC (Proof of Concept)

> **Objectif** : Valider la stack Rust + Tauri + whisper.cpp et comparer les performances avec VoiceToText (Python)

**Durée estimée** : 2-3 jours

**Scope** : MVP minimal
- ✅ Hotkey global → Enregistrement → Transcription → Auto-paste
- ❌ Pas d'UI settings (config hardcodée pour le POC)
- ❌ Pas de download manager (modèle déjà installé)
- ❌ Pas de system tray (juste une fenêtre minimale)

---

## Prérequis

### Installation Rust

```bash
# Windows
# Télécharger depuis https://rustup.rs/
rustup-init.exe

# Vérifier installation
rustc --version
cargo --version
```

### Installation whisper.cpp

**Option A : Utiliser whisper-rs crate** (recommandé)
```toml
# Cargo.toml
[dependencies]
whisper-rs = "0.10"
```

**Option B : Build whisper.cpp manually**
```bash
git clone https://github.com/ggerganov/whisper.cpp
cd whisper.cpp
mkdir build && cd build
cmake ..
cmake --build . --config Release
```

### Télécharger un modèle Whisper

```bash
# Depuis whisper.cpp/models/
bash download-ggml-model.sh base

# Ou télécharger directement
# https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin
```

Placer le modèle dans : `C:\Users\[USER]\AppData\Roaming\Flemme\models\ggml-base.bin`

---

## Tasks Sprint 0

### ✅ Task 1 : Setup projet Tauri + Svelte

**Durée** : 30 min

**Steps :**

1. **Créer le projet**
```bash
# Installer Tauri CLI
cargo install tauri-cli --version "^2.0.0"

# Créer projet
cargo create-tauri-app flemme

# Sélectionner :
# - Framework : Svelte
# - TypeScript : Yes
# - Template : shadcn-svelte (ou base, on ajoutera shadcn après)
```

2. **Vérifier que ça fonctionne**
```bash
cd flemme
cargo tauri dev
```

Une fenêtre devrait s'ouvrir avec l'interface Svelte par défaut.

3. **Installer shadcn-svelte**
```bash
npx shadcn-svelte@latest init

# Sélectionner :
# - Style : Default
# - Base color : Slate
# - CSS variables : Yes
```

4. **Structure du projet créée**
```
flemme/
├── src-tauri/        # Backend Rust
├── src/              # Frontend Svelte
├── package.json
├── Cargo.toml
└── tauri.conf.json
```

**Validation :**
- [ ] `cargo tauri dev` lance l'app
- [ ] Interface Svelte s'affiche
- [ ] Pas d'erreurs dans la console

---

### ✅ Task 2 : Module Audio Recorder (Rust)

**Durée** : 1h

**Fichier** : `src-tauri/src/audio/recorder.rs`

**Implémentation :**

```rust
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig};
use std::sync::{Arc, Mutex};

pub struct AudioRecorder {
    device: Device,
    config: StreamConfig,
    stream: Option<Stream>,
    buffer: Arc<Mutex<Vec<f32>>>,
    sample_rate: u32,
}

impl AudioRecorder {
    /// Créer un nouveau recorder avec le microphone par défaut
    pub fn new() -> Result<Self, String> {
        let host = cpal::default_host();
        
        let device = host
            .default_input_device()
            .ok_or("Aucun microphone trouvé")?;
        
        let config = device
            .default_input_config()
            .map_err(|e| format!("Erreur config: {}", e))?;
        
        Ok(Self {
            device,
            config: config.into(),
            stream: None,
            buffer: Arc::new(Mutex::new(Vec::new())),
            sample_rate: 16000, // Whisper requiert 16kHz
        })
    }
    
    /// Démarrer l'enregistrement
    pub fn start_recording(&mut self) -> Result<(), String> {
        let buffer = Arc::clone(&self.buffer);
        
        // Réinitialiser le buffer
        buffer.lock().unwrap().clear();
        
        let stream = self.device
            .build_input_stream(
                &self.config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    // Callback appelé pour chaque chunk audio
                    let mut buf = buffer.lock().unwrap();
                    buf.extend_from_slice(data);
                },
                |err| eprintln!("Erreur stream: {}", err),
                None,
            )
            .map_err(|e| format!("Erreur création stream: {}", e))?;
        
        stream.play().map_err(|e| format!("Erreur play: {}", e))?;
        
        self.stream = Some(stream);
        Ok(())
    }
    
    /// Arrêter l'enregistrement et retourner les samples
    pub fn stop_recording(&mut self) -> Result<Vec<f32>, String> {
        if let Some(stream) = self.stream.take() {
            drop(stream); // Arrête le stream
        }
        
        let buffer = self.buffer.lock().unwrap();
        let audio = buffer.clone();
        
        Ok(audio)
    }
    
    /// Vérifier si en cours d'enregistrement
    pub fn is_recording(&self) -> bool {
        self.stream.is_some()
    }
}
```

**Dépendances à ajouter :**
```toml
# Cargo.toml
[dependencies]
cpal = "0.15"
```

**Tests manuels :**
```rust
// Dans main.rs ou un test
let mut recorder = AudioRecorder::new().unwrap();
recorder.start_recording().unwrap();

std::thread::sleep(std::time::Duration::from_secs(3));

let audio = recorder.stop_recording().unwrap();
println!("Enregistré {} samples", audio.len());
```

**Validation :**
- [ ] Enregistrement fonctionne (pas d'erreur)
- [ ] Buffer contient des données
- [ ] Durée cohérente (~3s → ~48000 samples à 16kHz)

---

### ✅ Task 3 : Intégration whisper.cpp (Rust)

**Durée** : 2h

**Fichier** : `src-tauri/src/transcription/engine.rs`

**Implémentation :**

```rust
use whisper_rs::{WhisperContext, FullParams, SamplingStrategy};
use std::path::Path;

pub struct TranscriptionEngine {
    ctx: WhisperContext,
    language: String,
}

impl TranscriptionEngine {
    /// Charger le modèle Whisper
    pub fn new(model_path: &Path, language: &str) -> Result<Self, String> {
        let ctx = WhisperContext::new(model_path.to_str().unwrap())
            .map_err(|e| format!("Erreur chargement modèle: {}", e))?;
        
        Ok(Self {
            ctx,
            language: language.to_string(),
        })
    }
    
    /// Transcrire de l'audio
    pub fn transcribe(&self, audio: &[f32]) -> Result<TranscriptionResult, String> {
        // Configuration des paramètres Whisper
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        
        // Langue
        params.set_language(Some(&self.language));
        
        // Pas de traduction
        params.set_translate(false);
        
        // Paramètres de qualité
        params.set_temperature(0.0);
        params.set_no_context(true);
        
        // Thread unique pour stabilité
        params.set_n_threads(1);
        
        // Créer une session de transcription
        let mut state = self.ctx.create_state()
            .map_err(|e| format!("Erreur création state: {}", e))?;
        
        // Transcrire
        state.full(params, audio)
            .map_err(|e| format!("Erreur transcription: {}", e))?;
        
        // Récupérer le nombre de segments
        let num_segments = state.full_n_segments()
            .map_err(|e| format!("Erreur récupération segments: {}", e))?;
        
        // Concaténer tous les segments
        let mut text = String::new();
        for i in 0..num_segments {
            let segment = state.full_get_segment_text(i)
                .map_err(|e| format!("Erreur segment {}: {}", i, e))?;
            text.push_str(&segment);
        }
        
        Ok(TranscriptionResult {
            text: text.trim().to_string(),
            language: self.language.clone(),
        })
    }
}

#[derive(Debug)]
pub struct TranscriptionResult {
    pub text: String,
    pub language: String,
}
```

**Dépendances :**
```toml
# Cargo.toml
[dependencies]
whisper-rs = "0.10"
```

**Configuration du modèle :**
```rust
// Dans main.rs
let model_path = Path::new("C:\\Users\\[USER]\\AppData\\Roaming\\Flemme\\models\\ggml-base.bin");
let engine = TranscriptionEngine::new(&model_path, "fr").unwrap();
```

**Tests manuels :**
```rust
// Enregistrer 3s d'audio
let audio = recorder.stop_recording().unwrap();

// Transcrire
let result = engine.transcribe(&audio).unwrap();
println!("Transcription : {}", result.text);
```

**Validation :**
- [ ] Modèle se charge sans erreur
- [ ] Transcription retourne du texte
- [ ] Texte cohérent avec l'audio
- [ ] Performance < 500ms pour 3s d'audio

---

### ✅ Task 4 : Hotkey Global (Tauri Plugin)

**Durée** : 1h

**Installation du plugin :**
```toml
# Cargo.toml
[dependencies]
tauri-plugin-global-shortcut = "2.0.0"
```

**Configuration Tauri :**
```json
// tauri.conf.json
{
  "plugins": {
    "globalShortcut": {
      "all": true
    }
  }
}
```

**Implémentation :**

```rust
// src-tauri/src/main.rs
use tauri::Manager;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct AppState {
    is_recording: Arc<Mutex<bool>>,
    recorder: Arc<Mutex<Option<AudioRecorder>>>,
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            // État partagé
            let state = Arc::new(AppState::default());
            app.manage(state.clone());
            
            // Initialiser le recorder
            let recorder = AudioRecorder::new().unwrap();
            *state.recorder.lock().unwrap() = Some(recorder);
            
            // Enregistrer le hotkey
            let shortcut = Shortcut::new(Some(Modifier::CONTROL), Code::Space);
            
            app.global_shortcut().on_shortcut(shortcut, {
                let state = state.clone();
                move |app, event| {
                    match event.state {
                        ShortcutState::Pressed => {
                            println!("🎤 Hotkey pressé : début enregistrement");
                            let mut is_rec = state.is_recording.lock().unwrap();
                            if !*is_rec {
                                let mut rec = state.recorder.lock().unwrap();
                                if let Some(recorder) = rec.as_mut() {
                                    recorder.start_recording().unwrap();
                                    *is_rec = true;
                                }
                            }
                        }
                        ShortcutState::Released => {
                            println!("⏹️ Hotkey relâché : fin enregistrement");
                            let mut is_rec = state.is_recording.lock().unwrap();
                            if *is_rec {
                                let mut rec = state.recorder.lock().unwrap();
                                if let Some(recorder) = rec.as_mut() {
                                    let audio = recorder.stop_recording().unwrap();
                                    *is_rec = false;
                                    
                                    // TODO: Transcrire l'audio
                                    println!("Audio capturé : {} samples", audio.len());
                                }
                            }
                        }
                    }
                }
            })?;
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Erreur lancement Tauri");
}
```

**Validation :**
- [ ] Hotkey Ctrl+Space détecté (Press)
- [ ] Hotkey Ctrl+Space détecté (Release)
- [ ] Enregistrement démarre au press
- [ ] Enregistrement s'arrête au release
- [ ] Logs corrects dans la console

---

### ✅ Task 5 : Clipboard + Auto-paste

**Durée** : 45 min

**Fichier** : `src-tauri/src/clipboard/manager.rs`

**Dépendances :**
```toml
# Cargo.toml
[dependencies]
arboard = "3.3"
enigo = "0.2"
```

**Implémentation :**

```rust
use arboard::Clipboard;
use enigo::{Enigo, Key, KeyboardControllable};
use std::thread;
use std::time::Duration;

pub struct ClipboardManager {
    clipboard: Clipboard,
    auto_paste: bool,
}

impl ClipboardManager {
    pub fn new(auto_paste: bool) -> Result<Self, String> {
        let clipboard = Clipboard::new()
            .map_err(|e| format!("Erreur init clipboard: {}", e))?;
        
        Ok(Self {
            clipboard,
            auto_paste,
        })
    }
    
    /// Copier du texte dans le clipboard
    pub fn copy(&mut self, text: &str) -> Result<(), String> {
        self.clipboard
            .set_text(text)
            .map_err(|e| format!("Erreur copie: {}", e))
    }
    
    /// Copier et coller automatiquement
    pub fn copy_and_paste(&mut self, text: &str) -> Result<(), String> {
        // Copier
        self.copy(text)?;
        
        // Auto-paste si activé
        if self.auto_paste {
            // Petit délai pour laisser le temps à l'app de relâcher le hotkey
            thread::sleep(Duration::from_millis(100));
            
            // Simuler Ctrl+V
            let mut enigo = Enigo::new();
            enigo.key_down(Key::Control);
            enigo.key_click(Key::Layout('v'));
            enigo.key_up(Key::Control);
        }
        
        Ok(())
    }
}
```

**Intégration dans le hotkey :**

```rust
// Dans ShortcutState::Released
let audio = recorder.stop_recording().unwrap();

// Transcrire
let result = engine.transcribe(&audio).unwrap();
println!("📝 Transcription : {}", result.text);

// Copier et coller
let mut clipboard = ClipboardManager::new(true).unwrap();
clipboard.copy_and_paste(&result.text).unwrap();
println!("✅ Texte collé automatiquement");
```

**Validation :**
- [ ] Texte copié dans clipboard
- [ ] Ctrl+V simulé fonctionne
- [ ] Texte apparaît dans l'app active (Notepad, etc.)

---

### ✅ Task 6 : Intégration complète + Tests

**Durée** : 1h

**Objectif :** Faire fonctionner le flow complet end-to-end

**Flow à valider :**
```
1. User presse Ctrl+Space
   ↓
2. Enregistrement audio démarre
   ↓
3. User maintient 3-5 secondes en parlant
   ↓
4. User relâche Ctrl+Space
   ↓
5. Enregistrement s'arrête
   ↓
6. Transcription whisper.cpp (~200-500ms)
   ↓
7. Texte copié dans clipboard
   ↓
8. Ctrl+V simulé automatiquement
   ↓
9. Texte apparaît dans l'app active
   ✅ Succès !
```

**Scénarios de test :**

1. **Test basique**
   - Ouvrir Notepad
   - Presser Ctrl+Space
   - Dire "Bonjour ceci est un test"
   - Relâcher Ctrl+Space
   - Vérifier que le texte apparaît dans Notepad

2. **Test longue durée**
   - Enregistrer 30 secondes
   - Vérifier stabilité

3. **Test multi-langues**
   - Enregistrer en anglais
   - Vérifier transcription

4. **Test sans parole**
   - Enregistrer 5s de silence
   - Vérifier comportement (texte vide ou erreur)

**Métriques à noter :**
- Temps transcription pour 3s audio : ___ ms
- Temps transcription pour 10s audio : ___ ms
- Utilisation RAM : ___ MB
- Taille exécutable : ___ MB

**Validation finale :**
- [ ] Flow complet fonctionne sans erreur
- [ ] Performances meilleures que Python
- [ ] Pas de crash ou memory leak
- [ ] Qualité transcription correcte

---

### ✅ Task 7 : Benchmark vs Python (VoiceToText)

**Durée** : 30 min

**Objectif :** Comparer les performances Rust vs Python

**Métriques à mesurer :**

| Métrique | Python (VoiceToText) | Rust (Flemme) | Écart |
|----------|---------------------|---------------|-------|
| **Temps démarrage** | ___ s | ___ s | ___ |
| **RAM au repos** | ___ MB | ___ MB | ___ |
| **RAM pendant transcription** | ___ MB | ___ MB | ___ |
| **Transcription 3s audio** | ___ ms | ___ ms | ___ |
| **Transcription 10s audio** | ___ ms | ___ ms | ___ |
| **Taille installeur** | ~800 MB | ___ MB | ___ |
| **Qualité transcription** | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | Identique ? |

**Tests à faire :**

1. **Démarrage app**
   ```
   Python : Mesurer temps entre lancement et "app prête"
   Rust : Mesurer temps entre lancement et "app prête"
   ```

2. **Transcription 3s audio**
   ```
   Enregistrer exactement 3 secondes de parole
   Python : Noter le temps de transcription dans les logs
   Rust : Noter le temps de transcription dans les logs
   ```

3. **Utilisation RAM**
   ```
   Python : Ouvrir Gestionnaire des tâches, noter RAM
   Rust : Ouvrir Gestionnaire des tâches, noter RAM
   ```

**Résultats attendus :**
- Rust devrait être **5-10x plus rapide** en transcription
- Rust devrait utiliser **3-5x moins de RAM**
- Rust devrait démarrer **5-10x plus vite**
- Taille installeur : Rust ~15 MB vs Python ~800 MB

**Validation :**
- [ ] Benchmark complété
- [ ] Résultats documentés
- [ ] Gains significatifs confirmés (si oui → GO pour la suite !)

---

## Livrables Sprint 0

1. **Code source**
   - Projet Tauri + Svelte fonctionnel
   - Modules Rust (audio, transcription, clipboard, hotkey)
   - Compilable et exécutable

2. **Documentation**
   - README.md avec instructions build
   - Benchmark Python vs Rust
   - Notes sur les difficultés rencontrées

3. **Démo**
   - Vidéo ou GIF du flow complet
   - Comparaison côte à côte Python vs Rust

---

## Problèmes potentiels & Solutions

### ❌ Problème : whisper-rs ne compile pas

**Symptômes :**
```
error: linking with `link.exe` failed
```

**Solutions :**
1. Installer Visual Studio Build Tools (MSVC)
2. Ou utiliser bindings FFI directs vers whisper.cpp
3. Ou précompiler whisper.cpp en DLL et linker dynamiquement

---

### ❌ Problème : Audio capturé est vide

**Symptômes :**
```
Enregistré 0 samples
```

**Solutions :**
1. Vérifier que le micro est bien détecté : `cpal::default_input_device()`
2. Vérifier permissions Windows (paramètres confidentialité micro)
3. Tester avec un autre micro

---

### ❌ Problème : Hotkey ne se déclenche pas

**Symptômes :**
```
Ctrl+Space pressé mais aucun log
```

**Solutions :**
1. Vérifier que `tauri-plugin-global-shortcut` est bien installé
2. Tester avec un autre hotkey (ex: `Ctrl+Alt+Space`)
3. Lancer en mode Administrateur (certains hotkeys nécessitent privilèges)

---

### ❌ Problème : Transcription très lente (>5s)

**Symptômes :**
```
Transcription 3s audio : 8000ms
```

**Solutions :**
1. Vérifier que le modèle est bien en CPU mode (pas de CUDA sans GPU)
2. Utiliser un modèle plus petit (tiny au lieu de base)
3. Réduire `n_threads` dans les params Whisper
4. Compiler whisper.cpp avec optimisations (`-O3`, AVX2)

---

## Prochaines étapes (si POC validé ✅)

**Sprint 1 : Distribution**
- Download manager modèles
- Installeur Windows (NSIS)
- First Launch wizard

**Sprint 2 : UI Settings**
- Page settings complète (shadcn-svelte)
- Sauvegarde config dynamique
- Validation formulaire

**Sprint 3 : Features avancées**
- System tray icon
- Barre flottante enregistrement
- Notifications
- VAD (Voice Activity Detection)

---

## Commandes utiles

### Build du projet
```bash
# Development (hot reload)
cargo tauri dev

# Production (optimisé)
cargo tauri build
```

### Tests
```bash
# Tests unitaires Rust
cargo test

# Tests backend seul
cd src-tauri && cargo test
```

### Logs
```bash
# Activer logs détaillés
$env:RUST_LOG="debug"
cargo tauri dev
```

### Clean
```bash
# Nettoyer build artifacts
cargo clean
rm -rf target
rm -rf node_modules
```

---

**Bon courage pour le Sprint 0 ! 🚀**

*N'hésite pas à ajuster les estimations de durée selon ton rythme. L'important est de valider le POC end-to-end.*
