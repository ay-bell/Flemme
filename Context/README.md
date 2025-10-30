# Flemme - Documentation de Setup

> **Migration de VoiceToText (Python) vers Flemme (Rust + Tauri + whisper.cpp)**

---

## 📚 Fichiers de Spécification

Ce package contient tous les fichiers nécessaires pour démarrer le développement de Flemme avec Claude Code :

### 1. **ARCHITECTURE.md**
📖 Architecture technique complète du projet
- Stack détaillée (Rust, Tauri, Svelte, whisper.cpp)
- Modules backend et composants frontend
- APIs et structures de données
- Performances attendues vs VoiceToText Python

**Utilisation :** Lire en premier pour comprendre la vision globale

---

### 2. **SPRINT_0_POC.md**
🎯 Plan détaillé du Sprint 0 (Proof of Concept)
- Tasks étape par étape avec code d'exemple
- Objectif : Hotkey → Enregistrement → Transcription → Auto-paste
- Durée estimée : 2-3 jours
- Checklist de validation

**Utilisation :** Guide de développement pour le POC

---

### 3. **SETUP_GUIDE.md**
🚀 Guide de setup spécifique pour Claude Code
- Instructions structurées pour chaque phase
- Prompts optimisés pour Claude Code
- Troubleshooting des problèmes courants

**Utilisation :** Donner en contexte à Claude Code dans VS Code

---

### 4. **project_structure.txt**
📁 Arborescence complète du projet
- Structure des dossiers backend (Rust)
- Structure des dossiers frontend (Svelte)
- Détails sur chaque module et sa responsabilité

**Utilisation :** Référence pour organiser le code

---

### 5. **cargo_config_template.toml**
⚙️ Template Cargo.toml annoté
- Dépendances Rust commentées
- Optimisations build
- Features (CPU, CUDA, VAD)
- Notes d'implémentation

**Utilisation :** Copier dans `src-tauri/Cargo.toml`

---

### 6. **tauri_config_template.json**
🔧 Template tauri.conf.json annoté
- Configuration Tauri complète
- Permissions et sécurité
- Build settings
- Notes détaillées en commentaires JSON

**Utilisation :** Copier dans `src-tauri/tauri.conf.json`

---

## 🚀 Comment Démarrer

### Option A : Avec Claude Desktop (Architecture & Planning)

**Ce que tu fais maintenant** : Discussions stratégiques, décisions d'architecture

1. Lis **ARCHITECTURE.md** pour comprendre la vision
2. Lis **SPRINT_0_POC.md** pour le plan d'action
3. Pose des questions à Claude Desktop si besoin de clarifications

**Ensuite** : Passe à Claude Code pour l'implémentation

---

### Option B : Avec Claude Code dans VS Code (Implémentation)

**Prérequis :**
- [ ] Rust installé (`rustc --version`)
- [ ] Node.js 18+ installé
- [ ] VS Code + Extension Claude Code
- [ ] Git

**Steps :**

1. **Créer le repo localement**
   ```bash
   mkdir Flemme
   cd Flemme
   git init
   ```

2. **Copier les fichiers de spec dans le repo**
   ```
   Flemme/
   ├── ARCHITECTURE.md
   ├── SPRINT_0_POC.md
   ├── SETUP_GUIDE.md
   ├── project_structure.txt
   ├── cargo_config_template.toml
   └── tauri_config_template.json
   ```

3. **Ouvrir dans VS Code**
   ```bash
   code .
   ```

4. **Lancer Claude Code**
   - Ouvrir le panneau Claude Code (Cmd/Ctrl+Shift+P → "Claude Code")
   - Lui donner le contexte :
     ```
     Je veux setup le projet Flemme selon les specs.
     Commence par lire SETUP_GUIDE.md et suis les instructions.
     Référence ARCHITECTURE.md et SPRINT_0_POC.md au besoin.
     ```

5. **Claude Code va :**
   - Lire les fichiers de spec
   - Créer le projet Tauri + Svelte
   - Installer shadcn-svelte
   - Créer la structure de dossiers backend
   - Implémenter les modules un par un
   - Tester le flow complet

---

## 📝 Workflow Recommandé

### Phase 1 : Setup Initial (avec Claude Code)
- Création du projet Tauri + Svelte
- Installation des dépendances
- Vérification que `cargo tauri dev` fonctionne

### Phase 2 : Implémentation Modules (avec Claude Code)
- Module AudioRecorder
- Module TranscriptionEngine
- Module HotkeyListener
- Module ClipboardManager
- Intégration complète

### Phase 3 : Tests & Validation (Manuel + Claude Code)
- Tests end-to-end
- Benchmark vs VoiceToText Python
- Documentation des résultats

### Phase 4 : Retour à Claude Desktop (Planning suite)
- Analyse des performances
- Décisions pour Sprint 1 (Distribution)
- Planification features avancées

---

## 🎯 Objectifs du POC

**Succès si :**
- [ ] Flow complet fonctionne (Hotkey → Enregistrement → Transcription → Paste)
- [ ] Performances **5-10x meilleures** que Python
- [ ] RAM **3-5x moins** utilisée
- [ ] Pas de bug majeur ou crash
- [ ] Qualité transcription identique à VoiceToText

**Si échec :**
- Analyser les blocages
- Retour à Claude Desktop pour ajuster la stratégie
- Peut-être rester sur Python mais optimiser différemment

---

## 🔍 Points d'Attention

### whisper-rs Compilation

**Problème potentiel :** Long à compiler, peut échouer sur certains systèmes

**Solutions :**
1. S'assurer que Visual Studio Build Tools (MSVC) est installé
2. Vérifier que cmake est dans le PATH
3. Si échec : Utiliser bindings FFI directs vers whisper.cpp

### Permissions Audio

**Problème potentiel :** Microphone non détecté

**Solutions :**
1. Vérifier paramètres Windows : Confidentialité > Microphone
2. Donner accès à l'app
3. Tester avec un autre micro

### Hotkeys Globaux

**Problème potentiel :** Hotkey ne se déclenche pas

**Solutions :**
1. Lancer en mode Administrateur
2. Changer le hotkey (ex: Ctrl+Alt+Space)
3. Vérifier qu'aucune autre app n'utilise le même hotkey

---

## 📊 Comparaison Python vs Rust (Cible)

| Métrique | Python (VoiceToText) | Rust (Flemme) | Objectif |
|----------|---------------------|---------------|----------|
| **Démarrage** | ~3-5s | < 500ms | ✅ 6-10x plus rapide |
| **RAM repos** | ~800 MB | < 300 MB | ✅ 60% moins |
| **RAM transcription** | ~1.5 GB | < 500 MB | ✅ 66% moins |
| **Transcription 3s** | ~1s | < 200ms | ✅ 5x plus rapide |
| **Transcription 10s** | ~3s | < 500ms | ✅ 6x plus rapide |
| **Taille installeur** | ~800 MB | < 20 MB | ✅ 97% plus léger |

---

## 📚 Ressources

**Documentation officielle :**
- [Tauri 2.0](https://v2.tauri.app/)
- [whisper.cpp](https://github.com/ggerganov/whisper.cpp)
- [Svelte 5](https://svelte-5-preview.vercel.app/)
- [shadcn-svelte](https://www.shadcn-svelte.com/)

**Crates Rust utiles :**
- [cpal](https://docs.rs/cpal/)
- [whisper-rs](https://docs.rs/whisper-rs/)
- [arboard](https://docs.rs/arboard/)
- [enigo](https://docs.rs/enigo/)

---

## 🆘 Support

**Si tu es bloqué :**

1. **Problèmes techniques** : Lis SPRINT_0_POC.md section "Problèmes potentiels & Solutions"
2. **Questions d'architecture** : Référence ARCHITECTURE.md
3. **Setup Claude Code** : Lis SETUP_GUIDE.md
4. **Besoin de clarifications** : Retour à Claude Desktop avec le contexte complet

---

## 📅 Timeline Prévue

**Sprint 0 - POC** : 2-3 jours
- Jour 1 : Setup + Audio + Transcription
- Jour 2 : Hotkeys + Clipboard + Intégration
- Jour 3 : Tests + Benchmark + Documentation

**Sprint 1 - Distribution** : 1-2 jours (si POC validé)
- Download manager modèles
- Installeur Windows
- First Launch wizard

**Sprint 2 - UI** : 2-3 jours
- Settings page complète
- System tray
- Barre flottante

**Sprint 3+** : Features avancées (VAD, notifications, etc.)

---

## ✅ Checklist Avant de Commencer

- [ ] J'ai lu ARCHITECTURE.md
- [ ] J'ai lu SPRINT_0_POC.md
- [ ] J'ai lu SETUP_GUIDE.md
- [ ] J'ai Rust installé et fonctionnel
- [ ] J'ai Node.js installé
- [ ] J'ai VS Code avec Claude Code
- [ ] J'ai un modèle Whisper téléchargé (base recommandé)
- [ ] Je sais où je vais mettre le repo (ex: C:\Dev\Flemme)

---

## 🎉 Let's Go !

**Tu es prêt à démarrer le développement de Flemme !**

1. Crée ton repo
2. Copie ces fichiers dedans
3. Ouvre dans VS Code
4. Lance Claude Code avec SETUP_GUIDE.md
5. Let's build something awesome ! 🚀

---

*Dernière mise à jour : 30 octobre 2025*

**Questions ? → Retour à Claude Desktop avec ce README et ton contexte**
