# Flemme

Application de transcription vocale en temps réel avec détection de voix et intégration LLM.

## 🚀 Démarrage Rapide

### Pour Commencer le Développement

1. **Installer les prérequis** (Rust, Node.js, Visual Studio Build Tools, Git)
2. **Exécuter le script d'installation automatique** :
   ```powershell
   .\setup-new-machine.ps1
   ```
3. **Lancer le mode développement** :
   ```powershell
   cd flemme-app
   npm run dev          # Terminal 1
   cargo tauri dev      # Terminal 2
   ```

### Pour Transférer sur un Nouveau PC

Consultez **[MIGRATION.md](MIGRATION.md)** pour le guide complet de transfert.

---

## 📚 Documentation

| Document | Description |
|----------|-------------|
| **[MIGRATION.md](MIGRATION.md)** | Guide complet pour transférer le projet sur un nouveau PC |
| **[SETUP_CHECKLIST.md](SETUP_CHECKLIST.md)** | Checklist étape par étape pour vérifier l'installation |
| **[WINDOWS_SETUP.md](WINDOWS_SETUP.md)** | Configurations Windows spécifiques (permissions, audio, etc.) |
| **[Context/](Context/)** | Documentation d'architecture et spécifications techniques |

---

## 🎯 Fonctionnalités

- **🎤 Transcription vocale** : Enregistrement et transcription en temps réel avec Whisper
- **🔇 Détection de voix** : Filtrage automatique des silences avec Silero VAD
- **⌨️ Hotkeys globaux** : Raccourcis clavier personnalisables
- **📋 Collage automatique** : Colle automatiquement le texte transcrit
- **🤖 Modes d'exécution** : Intégration LLM pour post-traitement du texte
- **📝 Vocabulaire personnalisé** : Amélioration de la transcription avec vos termes
- **🎨 Interface moderne** : UI sombre avec Svelte 5 et Tailwind CSS

---

## 🛠️ Stack Technique

### Frontend
- **Svelte 5** - Framework réactif avec runes
- **SvelteKit** - Build system et routing
- **Tailwind CSS** - Styling utilitaire
- **shadcn-svelte** - Composants UI

### Backend
- **Tauri 2** - Framework desktop natif
- **Rust** - Backend haute performance
- **whisper-rs** - Moteur de transcription (OpenAI Whisper)
- **Silero VAD** - Détection d'activité vocale (ONNX)
- **cpal** - Capture audio cross-platform

---

## 📦 Installation

### Prérequis Système

- **Windows 10/11** (64-bit)
- **8 GB RAM** minimum (16 GB recommandé)
- **5 GB d'espace disque** libre
- **Microphone** fonctionnel

### Prérequis Logiciels

1. **Rust** - https://rustup.rs/
2. **Node.js 18+** - https://nodejs.org/
3. **Visual Studio Build Tools** - https://visualstudio.microsoft.com/downloads/
4. **Git** - https://git-scm.com/

### Installation Automatique

```powershell
# Cloner le projet
git clone <url-du-repo> Flemme
cd Flemme

# Exécuter le script d'installation
.\setup-new-machine.ps1
```

Le script va :
- ✅ Vérifier tous les prérequis
- ✅ Installer les dépendances npm et Rust
- ✅ Télécharger les modèles ML nécessaires
- ✅ Configurer l'environnement de développement

### Installation Manuelle

Consultez **[MIGRATION.md](MIGRATION.md)** pour les instructions détaillées.

---

## 🎮 Utilisation

### Développement

```powershell
cd flemme-app

# Terminal 1 - Frontend (Vite dev server)
npm run dev

# Terminal 2 - Backend (Tauri avec hot reload)
cargo tauri dev
```

### Build de Production

```powershell
cd flemme-app

# Build frontend
npm run build

# Build application complète
cargo tauri build
```

L'exécutable sera créé dans : `flemme-app/src-tauri/target/release/flemme-app.exe`

---

## 🔧 Scripts Utiles

### Téléchargement de Modèles

```powershell
# Télécharger Whisper Small (défaut, ~466 MB)
.\download-whisper-model.ps1

# Télécharger un modèle différent
.\download-whisper-model.ps1 -Model base          # Plus rapide (~140 MB)
.\download-whisper-model.ps1 -Model medium        # Plus précis (~1.5 GB)
.\download-whisper-model.ps1 -Model large-v2      # Très précis (~3 GB)
.\download-whisper-model.ps1 -Model large-v3-turbo # Équilibré (~1.6 GB)

# Re-télécharger en mode force
.\download-whisper-model.ps1 -Force

# Télécharger Silero VAD (~20 MB)
.\download-silero-vad.ps1
```

### Configuration

```powershell
# Vérifier la version des outils
rustc --version
cargo --version
node --version
npm --version

# Nettoyer et reconstruire
cd flemme-app
Remove-Item -Recurse -Force node_modules
npm install
cargo clean
cargo build
```

---

## 📁 Structure du Projet

```
Flemme/
├── flemme-app/                 # Application principale
│   ├── src/                    # Frontend Svelte
│   ├── src-tauri/              # Backend Rust
│   ├── static/                 # Assets statiques
│   └── package.json            # Dépendances npm
├── Context/                    # Documentation technique
├── setup-new-machine.ps1       # Script d'installation automatique
├── download-whisper-model.ps1  # Téléchargement modèle Whisper
├── download-silero-vad.ps1     # Téléchargement modèle Silero VAD
├── MIGRATION.md                # Guide de migration
├── SETUP_CHECKLIST.md          # Checklist d'installation
├── WINDOWS_SETUP.md            # Configuration Windows
└── README.md                   # Ce fichier
```

---

## 🔐 Permissions Windows Requises

- **Microphone** : Paramètres > Confidentialité > Microphone
- **Clipboard** : Accès automatique (pas de configuration)
- **Global Hotkeys** : Peut nécessiter droits admin

Consultez **[WINDOWS_SETUP.md](WINDOWS_SETUP.md)** pour les détails.

---

## 🎨 Configuration de l'Application

### Paramètres Disponibles

- **Raccourcis clavier** : Personnalisables (défaut : Ctrl+Alt+R)
- **Push-to-talk** : Maintenir ou toggle
- **Collage automatique** : Activer/désactiver
- **Langue** : Français, Anglais, etc.
- **Modèle vocal** : Choisir entre Base, Small, Medium, Large
- **Modes d'exécution** : Intégration LLM personnalisée
- **Vocabulaire** : Mots personnalisés pour améliorer la transcription

---

## 🐛 Dépannage

### L'application ne compile pas

```powershell
# Vérifier les prérequis
rustc --version
node --version

# Nettoyer et reconstruire
cargo clean
npm install
cargo build
```

### Les modèles ne se téléchargent pas

```powershell
# Vérifier le dossier de modèles
explorer "$env:APPDATA\Flemme\models"

# Re-télécharger manuellement
.\download-whisper-model.ps1 -Force
.\download-silero-vad.ps1 -Force
```

### Le microphone n'est pas détecté

1. Vérifier les permissions Windows : `ms-settings:privacy-microphone`
2. Tester avec l'Enregistreur vocal Windows
3. Rafraîchir les périphériques dans Flemme

### Plus de solutions

Consultez **[WINDOWS_SETUP.md](WINDOWS_SETUP.md)** section Dépannage.

---

## 🚀 Performance

### Optimisations CPU

- Utiliser le modèle **Base** pour plus de rapidité
- Fermer les applications gourmandes pendant l'enregistrement
- Utiliser le mode **Push-to-talk** pour des enregistrements courts

### Optimisations GPU (NVIDIA)

Flemme supporte l'accélération CUDA pour Whisper :

1. Installer CUDA Toolkit : https://developer.nvidia.com/cuda-downloads
2. Installer cuDNN
3. Reconstruire l'application : `cargo build --release`

Le GPU sera automatiquement utilisé si disponible.

---

## 📊 Modèles Disponibles

| Modèle | Taille | Précision | Rapidité | Recommandé pour |
|--------|--------|-----------|----------|-----------------|
| **Base** | ~140 MB | ⭐⭐ | ⭐⭐⭐⭐⭐ | Tests rapides, PC faibles |
| **Small** | ~466 MB | ⭐⭐⭐ | ⭐⭐⭐⭐ | Usage quotidien (défaut) |
| **Medium** | ~1.5 GB | ⭐⭐⭐⭐ | ⭐⭐⭐ | Meilleure précision |
| **Large V2** | ~3 GB | ⭐⭐⭐⭐⭐ | ⭐⭐ | Transcription critique |
| **Large V3 Turbo** | ~1.6 GB | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | Meilleur compromis |

---

## 🤝 Contribution

### Workflow de Développement

1. Créer une branche : `git checkout -b feature/ma-fonctionnalite`
2. Développer et tester
3. Commiter : `git commit -m "feat: ma fonctionnalité"`
4. Pousser : `git push origin feature/ma-fonctionnalite`
5. Créer une Pull Request

### Standards de Code

- **Frontend** : ESLint + Prettier
- **Backend** : rustfmt + clippy
- **Commits** : Convention Conventional Commits

---

## 📝 Licence

[À définir]

---

## 🙏 Remerciements

- **OpenAI Whisper** - Modèle de transcription
- **Silero VAD** - Détection de voix
- **Tauri** - Framework desktop
- **Svelte** - Framework frontend

---

## 📞 Support

Pour toute question ou problème :

1. Consultez la documentation dans le dossier **Context/**
2. Vérifiez **[MIGRATION.md](MIGRATION.md)** et **[WINDOWS_SETUP.md](WINDOWS_SETUP.md)**
3. Consultez la checklist dans **[SETUP_CHECKLIST.md](SETUP_CHECKLIST.md)**

---

**Développé avec ❤️ pour faciliter la transcription vocale**
