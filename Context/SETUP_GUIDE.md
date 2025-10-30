# Flemme - Guide de Setup avec Claude Code

> **Ce document est conçu pour être utilisé avec Claude Code dans VS Code**

---

## 🎯 Objectif

Ce guide vous aidera (ainsi que Claude Code) à setup le projet Flemme de A à Z.

## 📋 Prérequis

Avant de commencer, assurez-vous d'avoir :

- [x] **Rust** installé (`rustc --version` doit fonctionner)
- [x] **Node.js** 18+ installé (`node --version`)
- [x] **Visual Studio Build Tools** (Windows, pour compiler Rust)
- [x] **Git** installé
- [x] **VS Code** avec l'extension **Claude Code**

---

## 🚀 Instructions pour Claude Code

### Phase 1 : Création du projet

**Prompt pour Claude Code :**

```
Setup initial du projet Flemme :

1. Créer un nouveau projet Tauri + Svelte :
   - Utiliser `cargo create-tauri-app flemme`
   - Framework : Svelte
   - TypeScript : Yes
   - Template : Base (on ajoutera shadcn après)

2. Installer shadcn-svelte :
   - Lancer `npx shadcn-svelte@latest init`
   - Style : Default
   - Base color : Slate
   - CSS variables : Yes

3. Vérifier que le projet compile :
   - Lancer `cargo tauri dev`
   - Confirmer que la fenêtre s'ouvre

Référence l'architecture dans ARCHITECTURE.md pour comprendre la structure cible.
```

**Résultat attendu :**
- Projet Tauri + Svelte créé
- shadcn-svelte installé
- `cargo tauri dev` fonctionne

---

### Phase 2 : Structure des modules Backend

**Prompt pour Claude Code :**

```
Créer la structure modulaire backend Rust selon ARCHITECTURE.md :

1. Dans src-tauri/src/, créer les dossiers :
   - audio/
   - transcription/
   - hotkey/
   - clipboard/
   - config/

2. Pour chaque dossier, créer un mod.rs :
   - audio/mod.rs
   - transcription/mod.rs
   - etc.

3. Créer les fichiers vides suivants :
   - audio/recorder.rs
   - audio/vad.rs
   - transcription/engine.rs
   - transcription/models.rs
   - transcription/downloader.rs
   - hotkey/listener.rs
   - clipboard/manager.rs
   - config/settings.rs

4. Ajouter les exports dans chaque mod.rs

Ne pas implémenter le code pour l'instant, juste la structure.
```

**Résultat attendu :**
- Structure de dossiers créée
- Fichiers vides créés
- `cargo build` compile sans erreur

---

### Phase 3 : Implémentation Audio Recorder

**Prompt pour Claude Code :**

```
Implémenter le module AudioRecorder dans src-tauri/src/audio/recorder.rs

Référence : SPRINT_0_POC.md, section "Task 2 : Module Audio Recorder"

Fonctionnalités :
- Capture audio depuis le micro par défaut
- Format mono 16kHz
- Start/stop non-bloquant
- Buffer partagé thread-safe (Arc<Mutex<Vec<f32>>>)

Dépendances à ajouter dans Cargo.toml :
- cpal = "0.15"

Implémenter la struct AudioRecorder avec les méthodes :
- new() -> Result<Self, String>
- start_recording(&mut self) -> Result<(), String>
- stop_recording(&mut self) -> Result<Vec<f32>, String>
- is_recording(&self) -> bool

Ajouter des tests unitaires si possible.
```

**Résultat attendu :**
- AudioRecorder implémenté
- Tests passent (ou commentés pour l'instant)
- Code compile

---

### Phase 4 : Intégration whisper.cpp

**Prompt pour Claude Code :**

```
Implémenter le module TranscriptionEngine dans src-tauri/src/transcription/engine.rs

Référence : SPRINT_0_POC.md, section "Task 3 : Intégration whisper.cpp"

Utiliser le crate whisper-rs :
- Ajouter dans Cargo.toml : whisper-rs = "0.10"

Implémenter la struct TranscriptionEngine avec :
- new(model_path: &Path, language: &str) -> Result<Self, String>
- transcribe(&self, audio: &[f32]) -> Result<TranscriptionResult, String>

struct TranscriptionResult {
    pub text: String,
    pub language: String,
}

Configuration Whisper :
- SamplingStrategy::Greedy
- temperature = 0.0
- no_context = true
- n_threads = 1

IMPORTANT : Pour l'instant, hardcoder le chemin du modèle pour le POC :
C:\Users\[USER]\AppData\Roaming\Flemme\models\ggml-base.bin

(On fera le download manager plus tard)
```

**Résultat attendu :**
- TranscriptionEngine implémenté
- Code compile
- Modèle chargeable (si déjà téléchargé)

---

### Phase 5 : Hotkey Global avec Tauri Plugin

**Prompt pour Claude Code :**

```
Implémenter les hotkeys globaux avec tauri-plugin-global-shortcut

Référence : SPRINT_0_POC.md, section "Task 4 : Hotkey Global"

1. Ajouter dépendance dans Cargo.toml :
   tauri-plugin-global-shortcut = "2.0.0"

2. Ajouter dans tauri.conf.json :
   {
     "plugins": {
       "globalShortcut": { "all": true }
     }
   }

3. Dans src-tauri/src/main.rs :
   - Créer une struct AppState avec :
     - is_recording: Arc<Mutex<bool>>
     - recorder: Arc<Mutex<Option<AudioRecorder>>>
     - engine: Arc<Mutex<Option<TranscriptionEngine>>>
   
   - Dans setup(), enregistrer le hotkey Ctrl+Space :
     - Press → start_recording()
     - Release → stop_recording() + transcribe()
   
   - Pour l'instant, juste logger le texte transcrit (console)

Référence le code d'exemple dans SPRINT_0_POC.md pour l'implémentation complète.
```

**Résultat attendu :**
- Hotkey Ctrl+Space enregistré
- Press/Release détectés
- Enregistrement + transcription fonctionnels
- Texte loggé dans la console

---

### Phase 6 : Clipboard + Auto-paste

**Prompt pour Claude Code :**

```
Implémenter le ClipboardManager dans src-tauri/src/clipboard/manager.rs

Référence : SPRINT_0_POC.md, section "Task 5 : Clipboard + Auto-paste"

Dépendances à ajouter :
- arboard = "3.3"
- enigo = "0.2"

Implémenter :
- struct ClipboardManager { clipboard, auto_paste }
- new(auto_paste: bool) -> Result<Self, String>
- copy(&mut self, text: &str) -> Result<(), String>
- copy_and_paste(&mut self, text: &str) -> Result<(), String>

Dans copy_and_paste() :
1. Copier dans clipboard
2. Si auto_paste :
   - Sleep 100ms
   - Simuler Ctrl+V avec enigo

Intégrer dans le hotkey (main.rs) :
- Après transcription, appeler clipboard.copy_and_paste(text)
```

**Résultat attendu :**
- ClipboardManager implémenté
- Texte copié dans clipboard
- Auto-paste fonctionne

---

### Phase 7 : Tests end-to-end

**Prompt pour Claude Code :**

```
Valider le flow complet end-to-end :

1. Compiler en mode release : `cargo tauri build --debug`

2. Tester le flow :
   - Lancer l'app
   - Ouvrir Notepad
   - Presser Ctrl+Space
   - Parler 3-5 secondes
   - Relâcher Ctrl+Space
   - Vérifier que le texte apparaît dans Notepad

3. Logger les métriques de performance :
   - Temps transcription (ajouter un timer dans main.rs)
   - Utilisation RAM (observer dans Gestionnaire des tâches)

4. Si ça fonctionne → documenter les résultats dans un fichier BENCHMARK.md

Référence : SPRINT_0_POC.md, section "Task 6 : Intégration complète + Tests"
```

**Résultat attendu :**
- Flow complet fonctionne
- Texte transcrit automatiquement collé
- Performances mesurées

---

## 📝 Checklist finale

Une fois le setup terminé, vérifier que :

- [ ] `cargo tauri dev` lance l'app sans erreur
- [ ] Ctrl+Space déclenche l'enregistrement
- [ ] Relâcher Ctrl+Space transcrit et colle le texte
- [ ] Les performances sont bonnes (< 500ms pour 3s audio)
- [ ] Pas de crash ou memory leak
- [ ] Code est propre et commenté

---

## 🐛 Debugging

### Logs détaillés

Pour activer les logs Rust détaillés :

```bash
# Windows PowerShell
$env:RUST_LOG="debug"
cargo tauri dev
```

### Problèmes courants

**Problème : whisper-rs ne compile pas**
```
Solution : Installer Visual Studio Build Tools avec C++ workload
```

**Problème : Micro non détecté**
```
Solution : Vérifier permissions Windows (Paramètres > Confidentialité > Microphone)
```

**Problème : Hotkey ne fonctionne pas**
```
Solution : Lancer VS Code / app en mode Administrateur
```

---

## 🎓 Ressources

- [ARCHITECTURE.md](./ARCHITECTURE.md) - Architecture complète
- [SPRINT_0_POC.md](./SPRINT_0_POC.md) - Plan détaillé du POC
- [Tauri Docs](https://v2.tauri.app/)
- [whisper.cpp](https://github.com/ggerganov/whisper.cpp)
- [cpal docs](https://docs.rs/cpal/)

---

## 📞 Support

Si vous êtes bloqué, référencez les fichiers d'architecture et le plan de sprint. Si besoin, demandez à Claude Code de lire ces fichiers pour avoir le contexte complet.

**Bon développement ! 🚀**
