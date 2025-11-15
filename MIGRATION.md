# Guide de Migration - Flemme

Ce guide vous aidera à transférer le développement du projet Flemme vers un nouveau PC Windows.

## Table des matières

1. [Prérequis](#prérequis)
2. [Installation automatique](#installation-automatique)
3. [Installation manuelle](#installation-manuelle)
4. [Vérification](#vérification)
5. [Dépannage](#dépannage)

---

## Prérequis

Avant de commencer, assurez-vous d'avoir accès à :

### Matériel requis
- PC Windows 10 ou 11
- Minimum 8 GB RAM (16 GB recommandé)
- Minimum 5 GB d'espace disque libre
- Connexion Internet (pour télécharger les modèles ML)
- Microphone (pour tester l'enregistrement audio)

### Logiciels à installer

#### 1. **Rust** (obligatoire)
- Télécharger depuis : https://rustup.rs/
- Installer la version stable (recommandé : latest stable)
- Vérifier l'installation : `rustc --version` et `cargo --version`

#### 2. **Node.js** (obligatoire)
- Télécharger depuis : https://nodejs.org/
- Version requise : **18.0.0 ou supérieur**
- Vérifier l'installation : `node --version` et `npm --version`

#### 3. **Visual Studio Build Tools** (obligatoire pour Windows)
- Télécharger depuis : https://visualstudio.microsoft.com/downloads/
- Installer **"Desktop development with C++"** workload
- Inclure : MSVC v143, Windows SDK
- Alternative : Visual Studio Community avec C++ workload

#### 4. **Git** (obligatoire)
- Télécharger depuis : https://git-scm.com/
- Installer avec les options par défaut
- Vérifier l'installation : `git --version`

#### 5. **CMake** (recommandé)
- Télécharger depuis : https://cmake.org/download/
- Nécessaire pour compiler whisper-rs
- Vérifier l'installation : `cmake --version`

---

## Installation automatique

### Méthode recommandée pour gagner du temps

1. **Cloner ou copier le projet**
   ```powershell
   # Si vous clonez depuis GitHub
   git clone <url-du-repo> Flemme
   cd Flemme

   # Si vous copiez depuis une clé USB ou autre
   # Copiez simplement le dossier complet
   ```

2. **Ouvrir PowerShell en mode Administrateur**
   - Clic droit sur le menu Démarrer → "Terminal (Admin)" ou "PowerShell (Admin)"

3. **Autoriser l'exécution de scripts**
   ```powershell
   Set-ExecutionPolicy -ExecutionPolicy Bypass -Scope Process
   ```

4. **Naviguer vers le dossier du projet**
   ```powershell
   cd C:\chemin\vers\Flemme
   ```

5. **Exécuter le script d'installation**
   ```powershell
   .\setup-new-machine.ps1
   ```

6. **Suivre les instructions à l'écran**
   - Le script vérifiera tous les prérequis
   - Installera automatiquement les dépendances npm et Rust
   - Téléchargera les modèles ML (Whisper et Silero VAD)

⏱️ **Temps estimé** : 15-30 minutes (selon votre connexion Internet)

---

## Installation manuelle

Si vous préférez installer manuellement ou si le script automatique échoue :

### Étape 1 : Vérifier les prérequis

```powershell
# Vérifier Rust
rustc --version
cargo --version

# Vérifier Node.js
node --version
npm --version

# Vérifier Git
git --version
```

Si une commande échoue, installez le logiciel manquant (voir section [Prérequis](#prérequis)).

### Étape 2 : Installer les dépendances npm

```powershell
cd flemme-app
npm install
```

**Attendez que l'installation se termine** (peut prendre 5-10 minutes).

### Étape 3 : Récupérer les dépendances Rust

```powershell
cd src-tauri
cargo fetch
cd ..\..
```

### Étape 4 : Télécharger les modèles ML (CRITIQUE)

Les modèles sont **essentiels** pour que l'application fonctionne.

#### Modèle Whisper Small (~466 MB)

```powershell
.\download-whisper-model.ps1
```

Ce modèle sera téléchargé dans : `%APPDATA%\Flemme\models\ggml-small.bin`

#### Modèle Silero VAD (~20 MB)

```powershell
.\download-silero-vad.ps1
.\move-silero-vad.ps1
```

Ce modèle sera téléchargé dans : `%APPDATA%\Flemme\models\silero_vad.onnx`

### Étape 5 : Vérifier les modèles

```powershell
# Ouvrir l'explorateur de fichiers au bon endroit
explorer "$env:APPDATA\Flemme\models"
```

Vous devriez voir :
- `ggml-small.bin` (~466 MB)
- `silero_vad.onnx` (~20 MB)

---

## Vérification

### Test du build de développement

1. **Ouvrir deux terminaux** dans le dossier `flemme-app`

2. **Terminal 1 - Frontend**
   ```powershell
   cd flemme-app
   npm run dev
   ```

   Attendez de voir :
   ```
   VITE v6.x.x  ready in XXX ms
   ➜  Local:   http://localhost:1420/
   ```

3. **Terminal 2 - Backend**
   ```powershell
   cd flemme-app
   cargo tauri dev
   ```

   La compilation Rust prendra **5-15 minutes la première fois**. Les builds suivants seront plus rapides (30 secondes à 2 minutes).

4. **Vérifier que l'application s'ouvre**
   - Une fenêtre devrait s'ouvrir automatiquement
   - Vous devriez voir l'interface utilisateur de Flemme
   - Testez un enregistrement avec le raccourci clavier configuré

### Test du build de production

```powershell
cd flemme-app
npm run build
cargo tauri build
```

L'exécutable sera créé dans : `flemme-app\src-tauri\target\release\flemme-app.exe`

**Note** : Le premier build de production peut prendre **10-20 minutes**.

---

## Dépannage

### Problème : "npm install" échoue

**Solution** :
```powershell
# Nettoyer le cache npm
npm cache clean --force

# Supprimer node_modules et package-lock.json
Remove-Item -Recurse -Force node_modules
Remove-Item package-lock.json

# Réessayer
npm install
```

### Problème : "cargo fetch" ou "cargo build" échoue

**Causes possibles** :
1. Visual Studio Build Tools non installé correctement
2. Rust toolchain incomplet

**Solutions** :
```powershell
# Mettre à jour Rust
rustup update

# Ajouter la toolchain Windows
rustup target add x86_64-pc-windows-msvc

# Vérifier que MSVC est installé
# Réinstaller Visual Studio Build Tools si nécessaire
```

### Problème : "whisper-rs" ne compile pas

**Cause** : CMake manquant ou MSVC mal configuré

**Solution** :
1. Installer CMake : https://cmake.org/download/
2. Ajouter CMake au PATH Windows
3. Redémarrer le terminal
4. Réessayer `cargo build`

### Problème : Les modèles ne se téléchargent pas

**Solution 1** : Téléchargement manuel

```powershell
# Créer le dossier manuellement
New-Item -ItemType Directory -Path "$env:APPDATA\Flemme\models" -Force

# Télécharger Whisper manuellement
Invoke-WebRequest -Uri "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin" -OutFile "$env:APPDATA\Flemme\models\ggml-small.bin"

# Télécharger Silero VAD manuellement
Invoke-WebRequest -Uri "https://huggingface.co/onnx-community/silero-vad/resolve/main/onnx/model.onnx" -OutFile "$env:APPDATA\Flemme\models\silero_vad.onnx"
```

**Solution 2** : Télécharger avec un navigateur

1. Whisper Small : https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin
2. Silero VAD : https://huggingface.co/onnx-community/silero-vad/resolve/main/onnx/model.onnx
3. Copier les fichiers dans `%APPDATA%\Flemme\models\`

### Problème : "Permission denied" lors de l'exécution de scripts

**Solution** :
```powershell
# Ouvrir PowerShell en mode Administrateur
# Puis exécuter :
Set-ExecutionPolicy -ExecutionPolicy Bypass -Scope Process
```

### Problème : L'application compile mais ne démarre pas

**Vérifications** :
1. Les modèles ML sont-ils présents dans `%APPDATA%\Flemme\models\` ?
2. Le microphone est-il branché et activé dans Windows ?
3. Les permissions microphone sont-elles accordées ?

**Consulter les logs** :
```powershell
# En mode développement, les erreurs s'affichent dans le terminal
# En mode release, utiliser :
.\flemme-app\run-release-with-logs.bat
# Puis consulter flemme-app-release-logs.txt
```

### Problème : Erreurs de permissions Windows

Consultez [WINDOWS_SETUP.md](WINDOWS_SETUP.md) pour configurer :
- Permissions microphone
- Permissions clipboard
- Enregistrement de hotkeys globaux

---

## Fichiers importants à ne pas oublier

Lors du transfert, assurez-vous de **copier tous ces fichiers** :

### Essentiels
- `flemme-app/` - Dossier principal de l'application
- `flemme-app/package.json` - Dépendances npm
- `flemme-app/src-tauri/Cargo.toml` - Dépendances Rust
- `flemme-app/src-tauri/tauri.conf.json` - Configuration Tauri

### Scripts utiles
- `download-whisper-model.ps1`
- `download-silero-vad.ps1`
- `move-silero-vad.ps1`
- `setup-new-machine.ps1`

### Documentation
- `MIGRATION.md` (ce fichier)
- `SETUP_CHECKLIST.md`
- `WINDOWS_SETUP.md`
- `Context/` - Documentation d'architecture

### À NE PAS copier (seront régénérés)
- `flemme-app/node_modules/` - Sera recréé par `npm install`
- `flemme-app/src-tauri/target/` - Sera recréé par `cargo build`
- `flemme-app/.svelte-kit/` - Cache Svelte Kit
- `flemme-app/build/` - Build frontend

---

## Récapitulatif rapide

Pour un transfert réussi :

1. ✅ Installer les prérequis (Rust, Node.js, MSVC, Git, CMake)
2. ✅ Copier le projet complet
3. ✅ Exécuter `.\setup-new-machine.ps1` **OU** suivre l'installation manuelle
4. ✅ Télécharger les modèles ML (Whisper + Silero VAD)
5. ✅ Tester `npm run dev` et `cargo tauri dev`
6. ✅ Vérifier qu'un enregistrement audio fonctionne

**Temps total estimé** : 30-60 minutes (première installation complète)

---

## Besoin d'aide ?

- Consultez [SETUP_CHECKLIST.md](SETUP_CHECKLIST.md) pour une liste de vérification étape par étape
- Consultez [WINDOWS_SETUP.md](WINDOWS_SETUP.md) pour les configurations Windows spécifiques
- Vérifiez les logs dans `flemme-app-release-logs.txt` en cas d'erreur au runtime

---

**Note** : Ce guide suppose un transfert vers un PC Windows. Flemme n'est actuellement pas compatible avec macOS ou Linux sans modifications importantes.
