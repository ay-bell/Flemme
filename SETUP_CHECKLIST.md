# Checklist de Configuration - Flemme

Utilisez cette checklist pour vous assurer que tout est correctement configuré sur votre nouveau PC.

## 📋 Checklist Complète

### Phase 1 : Prérequis Système

- [ ] **Windows 10 ou 11** installé
- [ ] **8 GB RAM minimum** (16 GB recommandé)
- [ ] **5 GB d'espace disque** disponible
- [ ] **Connexion Internet** active
- [ ] **Microphone** branché et fonctionnel

---

### Phase 2 : Installation des Outils

#### Rust
- [ ] Téléchargé depuis https://rustup.rs/
- [ ] Installé avec succès
- [ ] Vérification : `rustc --version` fonctionne
  ```powershell
  rustc --version
  # Devrait afficher : rustc 1.x.x (...)
  ```
- [ ] Vérification : `cargo --version` fonctionne
  ```powershell
  cargo --version
  # Devrait afficher : cargo 1.x.x (...)
  ```

#### Node.js
- [ ] Téléchargé depuis https://nodejs.org/
- [ ] Version **18.0.0 ou supérieur** installée
- [ ] Vérification : `node --version` fonctionne
  ```powershell
  node --version
  # Devrait afficher : v18.x.x ou supérieur
  ```
- [ ] Vérification : `npm --version` fonctionne
  ```powershell
  npm --version
  # Devrait afficher : 9.x.x ou supérieur
  ```

#### Visual Studio Build Tools
- [ ] Téléchargé depuis https://visualstudio.microsoft.com/downloads/
- [ ] Installé avec "Desktop development with C++"
- [ ] MSVC v143 inclus
- [ ] Windows SDK inclus
- [ ] Redémarrage effectué après installation (si requis)

#### Git
- [ ] Téléchargé depuis https://git-scm.com/
- [ ] Installé avec les options par défaut
- [ ] Vérification : `git --version` fonctionne
  ```powershell
  git --version
  # Devrait afficher : git version 2.x.x
  ```

#### CMake (optionnel mais recommandé)
- [ ] Téléchargé depuis https://cmake.org/download/
- [ ] Installé et ajouté au PATH
- [ ] Vérification : `cmake --version` fonctionne
  ```powershell
  cmake --version
  # Devrait afficher : cmake version 3.x.x
  ```

---

### Phase 3 : Configuration du Projet

#### Récupération du Code
- [ ] Projet cloné depuis Git **OU** copié depuis une source
  ```powershell
  git clone <url-du-repo> Flemme
  cd Flemme
  ```
- [ ] Tous les fichiers présents (flemme-app/, scripts .ps1, etc.)
- [ ] Dossier `flemme-app` existe
- [ ] Dossier `Context` existe (documentation)

#### Permissions PowerShell
- [ ] PowerShell ouvert en mode Administrateur
- [ ] Politique d'exécution configurée
  ```powershell
  Set-ExecutionPolicy -ExecutionPolicy Bypass -Scope Process
  # Devrait afficher : Exécution Policy modifiée
  ```

---

### Phase 4 : Installation Automatique (Recommandé)

- [ ] Script `setup-new-machine.ps1` trouvé dans le dossier racine
- [ ] Script exécuté avec succès
  ```powershell
  .\setup-new-machine.ps1
  ```
- [ ] Toutes les vérifications passées (✓ en vert)
- [ ] Aucune erreur affichée en rouge

**Si le script automatique échoue, passer à la Phase 5 (Installation Manuelle)**

---

### Phase 5 : Installation Manuelle (Si Automatique a Échoué)

#### Dépendances npm
- [ ] Navigué vers `flemme-app`
  ```powershell
  cd flemme-app
  ```
- [ ] `npm install` exécuté avec succès
  ```powershell
  npm install
  # Attendre 5-10 minutes
  ```
- [ ] Dossier `node_modules` créé
- [ ] Aucune erreur affichée

#### Dépendances Rust
- [ ] Navigué vers `src-tauri`
  ```powershell
  cd src-tauri
  ```
- [ ] `cargo fetch` exécuté avec succès
  ```powershell
  cargo fetch
  # Attendre quelques minutes
  ```
- [ ] Aucune erreur affichée
- [ ] Retour au dossier racine
  ```powershell
  cd ..\..
  ```

---

### Phase 6 : Téléchargement des Modèles ML (CRITIQUE)

#### Création du Répertoire de Modèles
- [ ] Répertoire créé automatiquement **OU** créé manuellement
  ```powershell
  New-Item -ItemType Directory -Path "$env:APPDATA\Flemme\models" -Force
  ```
- [ ] Vérification de l'existence
  ```powershell
  Test-Path "$env:APPDATA\Flemme\models"
  # Devrait afficher : True
  ```

#### Modèle Whisper Small
- [ ] Script `download-whisper-model.ps1` exécuté
  ```powershell
  .\download-whisper-model.ps1
  ```
- [ ] Téléchargement complété (~466 MB)
- [ ] Fichier vérifié
  ```powershell
  Test-Path "$env:APPDATA\Flemme\models\ggml-small.bin"
  # Devrait afficher : True
  ```
- [ ] Taille du fichier correcte (~466 MB)
  ```powershell
  (Get-Item "$env:APPDATA\Flemme\models\ggml-small.bin").Length / 1MB
  # Devrait afficher : ~466
  ```

#### Modèle Silero VAD
- [ ] Script `download-silero-vad.ps1` exécuté
  ```powershell
  .\download-silero-vad.ps1
  ```
- [ ] Script `move-silero-vad.ps1` exécuté (si nécessaire)
  ```powershell
  .\move-silero-vad.ps1
  ```
- [ ] Téléchargement complété (~20 MB)
- [ ] Fichier vérifié
  ```powershell
  Test-Path "$env:APPDATA\Flemme\models\silero_vad.onnx"
  # Devrait afficher : True
  ```

#### Vérification Visuelle des Modèles
- [ ] Explorateur de fichiers ouvert
  ```powershell
  explorer "$env:APPDATA\Flemme\models"
  ```
- [ ] Fichier `ggml-small.bin` présent (~466 MB)
- [ ] Fichier `silero_vad.onnx` présent (~20 MB)

---

### Phase 7 : Test du Build de Développement

#### Terminal 1 - Frontend
- [ ] Nouveau terminal PowerShell ouvert
- [ ] Navigué vers `flemme-app`
  ```powershell
  cd flemme-app
  ```
- [ ] Serveur de développement démarré
  ```powershell
  npm run dev
  ```
- [ ] Message "VITE ready" affiché
- [ ] URL http://localhost:1420/ affichée
- [ ] **Laisser ce terminal ouvert**

#### Terminal 2 - Backend
- [ ] Nouveau terminal PowerShell ouvert (séparé)
- [ ] Navigué vers `flemme-app`
  ```powershell
  cd flemme-app
  ```
- [ ] Build Tauri démarré
  ```powershell
  cargo tauri dev
  ```
- [ ] Compilation Rust réussie (⚠️ Peut prendre 5-15 minutes la première fois)
- [ ] Fenêtre de l'application ouverte automatiquement
- [ ] Interface utilisateur visible

#### Test de l'Application
- [ ] Interface de paramètres accessible
- [ ] Raccourci clavier configurable
- [ ] Microphone détecté dans la liste des périphériques
- [ ] Test d'enregistrement fonctionnel
  - [ ] Appuyer sur le raccourci clavier (ex: Ctrl+Alt+R)
  - [ ] Parler quelques mots
  - [ ] Transcription affichée
  - [ ] Texte collé automatiquement (si activé)
- [ ] Aucune erreur dans les terminaux

---

### Phase 8 : Test du Build de Production (Optionnel)

- [ ] Serveurs de développement arrêtés (Ctrl+C dans les deux terminaux)
- [ ] Navigué vers `flemme-app`
  ```powershell
  cd flemme-app
  ```
- [ ] Build frontend exécuté
  ```powershell
  npm run build
  ```
- [ ] Build Tauri exécuté
  ```powershell
  cargo tauri build
  ```
- [ ] Compilation réussie (⚠️ Peut prendre 10-20 minutes la première fois)
- [ ] Exécutable créé dans `src-tauri\target\release\flemme-app.exe`
- [ ] Exécutable testé manuellement
  ```powershell
  .\src-tauri\target\release\flemme-app.exe
  ```
- [ ] Application fonctionne correctement

---

### Phase 9 : Configuration Windows (Important)

#### Permissions Microphone
- [ ] Paramètres Windows ouverts : `ms-settings:privacy-microphone`
- [ ] "Autoriser les applications à accéder au microphone" **activé**
- [ ] "flemme-app" autorisé dans la liste (après premier lancement)

#### Permissions Clipboard
- [ ] Test de collage automatique fonctionnel
- [ ] Ctrl+V colle bien le texte transcrit (si désactivé dans l'app)

#### Hotkeys Globaux
- [ ] Raccourci clavier enregistré sans erreur
- [ ] Raccourci fonctionne même quand l'app est en arrière-plan
- [ ] Aucun conflit avec d'autres applications

---

### Phase 10 : Vérification Finale

#### Fichiers et Dossiers
- [ ] `flemme-app/node_modules/` existe et contient des fichiers
- [ ] `flemme-app/src-tauri/target/debug/` existe (après cargo tauri dev)
- [ ] `%APPDATA%\Flemme\models\` contient les 2 modèles
- [ ] `%APPDATA%\Flemme\settings.json` créé (après première utilisation)

#### Fonctionnalités
- [ ] Enregistrement audio fonctionne
- [ ] Transcription fonctionne (Whisper)
- [ ] Filtrage de silence fonctionne (Silero VAD)
- [ ] Collage automatique fonctionne (si activé)
- [ ] Push-to-talk fonctionne (si activé)
- [ ] Annulation d'enregistrement fonctionne (Escape)
- [ ] Indicateur flottant s'affiche pendant l'enregistrement
- [ ] Changement de périphérique audio fonctionne
- [ ] Modes d'exécution fonctionnent (si configurés)
- [ ] Modèles LLM fonctionnent (si configurés)
- [ ] Vocabulaire personnalisé fonctionne

#### Performance
- [ ] Temps de démarrage acceptable (< 5 secondes)
- [ ] Réactivité de l'interface correcte
- [ ] Transcription rapide (quelques secondes pour 10-20 secondes d'audio)
- [ ] Pas de lag lors de l'enregistrement
- [ ] Utilisation mémoire raisonnable (< 500 MB)

---

## 🎉 Configuration Terminée !

Si toutes les cases sont cochées, votre environnement de développement Flemme est prêt !

### Prochaines Étapes

#### Développement Quotidien
```powershell
# Terminal 1
cd flemme-app
npm run dev

# Terminal 2
cd flemme-app
cargo tauri dev
```

#### Build de Production
```powershell
cd flemme-app
npm run build
cargo tauri build
```

#### Mise à Jour des Dépendances
```powershell
# npm
npm update

# Rust
cargo update
```

---

## ❌ En Cas de Problème

Si des cases ne sont pas cochées :

1. **Consultez [MIGRATION.md](MIGRATION.md)** section Dépannage
2. **Consultez [WINDOWS_SETUP.md](WINDOWS_SETUP.md)** pour les configurations Windows
3. **Vérifiez les logs** dans les terminaux ou `flemme-app-release-logs.txt`
4. **Réessayez** les étapes qui ont échoué
5. **Nettoyez et recommencez** si nécessaire :
   ```powershell
   # Nettoyer npm
   Remove-Item -Recurse -Force flemme-app/node_modules
   Remove-Item flemme-app/package-lock.json
   npm install

   # Nettoyer Rust
   cargo clean
   cargo build
   ```

---

**Bon développement avec Flemme ! 🎤✨**
