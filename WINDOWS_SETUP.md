# Configuration Windows - Flemme

Ce guide couvre toutes les configurations spécifiques à Windows nécessaires pour faire fonctionner Flemme correctement.

## Table des matières

1. [Permissions Windows](#permissions-windows)
2. [Configuration PowerShell](#configuration-powershell)
3. [Configuration Audio](#configuration-audio)
4. [Hotkeys et Raccourcis](#hotkeys-et-raccourcis)
5. [Pare-feu et Sécurité](#pare-feu-et-sécurité)
6. [Configuration GPU (Optionnel)](#configuration-gpu-optionnel)
7. [Dépannage Windows](#dépannage-windows)

---

## Permissions Windows

### 1. Permissions Microphone

Flemme nécessite l'accès au microphone pour enregistrer l'audio.

#### Activation Globale

1. **Ouvrir les Paramètres Windows**
   - Appuyez sur `Win + I`
   - Ou exécutez : `ms-settings:privacy-microphone`

2. **Naviguer vers Confidentialité et sécurité > Microphone**
   - Vérifiez que "Accès au microphone" est **Activé**
   - Vérifiez que "Autoriser les applications à accéder au microphone" est **Activé**

3. **Autoriser les applications de bureau**
   - Faites défiler vers le bas
   - Activez "Autoriser les applications de bureau à accéder au microphone"

#### Vérification

```powershell
# Tester si le microphone est détecté
# Dans l'application Flemme, aller dans Paramètres > Matériel
# La liste devrait afficher votre microphone
```

**Note** : Flemme doit être lancé au moins une fois pour apparaître dans la liste des applications autorisées.

---

### 2. Permissions Clipboard

Flemme lit et écrit dans le presse-papiers pour la fonctionnalité de collage automatique.

#### Vérification

Windows 10/11 autorise généralement l'accès au clipboard sans configuration supplémentaire.

**Test** :
1. Lancer Flemme
2. Activer "Collage automatique" dans les paramètres
3. Faire un enregistrement
4. Le texte devrait être automatiquement collé

Si le collage automatique ne fonctionne pas :
- Vérifiez que l'application cible a le focus
- Vérifiez les paramètres de confidentialité Windows

---

### 3. Permissions d'Administration (Hotkeys)

Les hotkeys globaux nécessitent parfois des privilèges élevés.

#### Si les hotkeys ne fonctionnent pas

**Option 1 : Lancer Flemme en tant qu'administrateur**
```powershell
# Clic droit sur flemme-app.exe > Exécuter en tant qu'administrateur
```

**Option 2 : Configurer l'exécution permanente en admin**
1. Clic droit sur `flemme-app.exe`
2. Propriétés > Compatibilité
3. Cocher "Exécuter ce programme en tant qu'administrateur"
4. Appliquer

**Note** : Cela n'est généralement PAS nécessaire. Essayez d'abord sans admin.

---

## Configuration PowerShell

### Politique d'Exécution de Scripts

Windows bloque par défaut l'exécution de scripts PowerShell non signés.

#### Solution Temporaire (Recommandée)

```powershell
# Ouvrir PowerShell en mode Administrateur
# Puis exécuter :
Set-ExecutionPolicy -ExecutionPolicy Bypass -Scope Process
```

Cette configuration est **temporaire** et ne s'applique qu'à la session PowerShell actuelle.

#### Solution Permanente (Moins Sécurisée)

```powershell
# Ouvrir PowerShell en mode Administrateur
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

**Avertissement** : Cela permet l'exécution de tous les scripts locaux. Utilisez avec prudence.

#### Vérifier la Politique Actuelle

```powershell
Get-ExecutionPolicy -List
```

---

## Configuration Audio

### 1. Sélection du Périphérique d'Enregistrement

#### Dans Windows

1. **Ouvrir les Paramètres de Son**
   - Clic droit sur l'icône de volume (barre des tâches)
   - "Paramètres de son"
   - Ou exécutez : `ms-settings:sound`

2. **Vérifier le périphérique d'entrée**
   - Section "Entrée"
   - Sélectionnez votre microphone
   - Testez en parlant (la barre devrait bouger)

3. **Définir comme périphérique par défaut**
   - "Paramètres avancés de son"
   - Onglet "Enregistrement"
   - Clic droit sur votre microphone > "Définir comme périphérique par défaut"

#### Dans Flemme

1. Ouvrir Flemme
2. Aller dans **Paramètres > Matériel**
3. Sélectionner le bon périphérique dans la liste déroulante
4. Cliquer sur le bouton **Rafraîchir** si le micro n'apparaît pas

---

### 2. Qualité Audio

Pour une meilleure transcription, configurez votre microphone :

1. **Ouvrir les Propriétés du Microphone**
   - Paramètres > Système > Son
   - Paramètres avancés de son > Enregistrement
   - Double-clic sur votre microphone

2. **Onglet "Avancé"**
   - Format par défaut : **16 bits, 48000 Hz (Qualité DVD)** ou supérieur
   - Appliquer

3. **Onglet "Niveaux"**
   - Niveau du microphone : **70-90%** (éviter 100% qui peut saturer)
   - Réduction du bruit : **Désactivé** (Flemme gère le bruit avec Silero VAD)

4. **Onglet "Améliorations"**
   - Décocher toutes les améliorations (pour un signal pur)
   - Ou désactiver complètement les améliorations

---

### 3. Permissions d'Accès Exclusif

Assurez-vous qu'aucune autre application ne monopolise le microphone.

1. **Propriétés du Microphone > Avancé**
2. **Décocher** "Autoriser les applications à prendre le contrôle exclusif de ce périphérique"
3. Appliquer

Cela permet à plusieurs applications (dont Flemme) d'accéder au micro simultanément.

---

## Hotkeys et Raccourcis

### 1. Enregistrement des Hotkeys Globaux

Flemme utilise `tauri-plugin-global-shortcut` pour enregistrer des raccourcis clavier globaux.

#### Raccourcis Disponibles

| Fonction | Raccourci Par Défaut | Configurable |
|----------|---------------------|--------------|
| Démarrer/Arrêter Enregistrement | `Ctrl+Alt+R` | ✅ Oui |
| Annuler Enregistrement | `Escape` | ✅ Oui |

#### Modifier les Raccourcis

1. Ouvrir Flemme
2. Aller dans **Paramètres**
3. Cliquer sur **Modifier** à côté du raccourci
4. Appuyer sur la nouvelle combinaison de touches
5. Enregistrer

---

### 2. Conflits de Raccourcis

Si un raccourci ne fonctionne pas, il peut être utilisé par une autre application.

#### Vérifier les Conflits

Applications courantes qui utilisent `Ctrl+Alt+R` :
- Logiciels d'enregistrement d'écran (OBS, Bandicam, etc.)
- Gestionnaires de presse-papiers
- Outils de productivité (AutoHotkey, etc.)

#### Résolution

1. **Fermer les applications conflictuelles** temporairement
2. **Changer le raccourci dans Flemme** pour une combinaison libre (ex: `Ctrl+Shift+R`)
3. **Changer le raccourci dans l'autre application**

---

### 3. Push-to-Talk vs Toggle

Flemme propose deux modes d'enregistrement :

#### Mode Toggle (Par Défaut)
- Appuyer une fois pour **démarrer** l'enregistrement
- Appuyer à nouveau pour **arrêter** et transcrire

#### Mode Push-to-Talk
- **Maintenir** le raccourci enfoncé pour enregistrer
- **Relâcher** pour arrêter et transcrire
- Utile pour des enregistrements courts et rapides

Configurez le mode dans **Paramètres > Push To Talk**.

---

## Pare-feu et Sécurité

### 1. Windows Defender

Flemme peut être bloqué par Windows Defender lors du premier lancement.

#### Si Flemme est Bloqué

1. **Windows Defender affiche une alerte**
   - "Windows a protégé votre ordinateur"
   - Cliquer sur **Informations complémentaires**
   - Cliquer sur **Exécuter quand même**

2. **Ajouter une Exception dans Defender**
   - Paramètres Windows > Confidentialité et sécurité > Sécurité Windows
   - Protection contre les virus et menaces > Gérer les paramètres
   - Exclusions > Ajouter ou supprimer des exclusions
   - Ajouter `C:\...\flemme-app.exe`

**Note** : Cela est courant pour les applications non signées. Flemme est open-source et sûr.

---

### 2. Pare-feu Windows

Flemme **ne nécessite PAS d'accès réseau** pour fonctionner.

Tout est local :
- Modèles ML stockés dans `%APPDATA%`
- Transcription en local via Whisper
- Aucun appel API externe (sauf si vous configurez des modèles LLM)

#### Si un Modèle LLM est Configuré

Si vous ajoutez un modèle LLM (ex: Gemini, ChatGPT) :
- Flemme aura besoin d'accès Internet
- Le pare-feu peut demander l'autorisation
- Autoriser l'accès pour "Réseaux privés" et "Réseaux publics"

---

## Configuration GPU (Optionnel)

Flemme est compilé avec le support CUDA pour accélérer Whisper sur GPU NVIDIA.

### Prérequis

- Carte graphique NVIDIA avec support CUDA (GTX 900 series ou plus récent)
- CUDA Toolkit installé (version 11.x ou 12.x)
- cuDNN installé

### Installation CUDA

1. **Télécharger CUDA Toolkit**
   - https://developer.nvidia.com/cuda-downloads
   - Choisir la version compatible avec votre carte

2. **Installer CUDA**
   - Suivre l'assistant d'installation
   - Redémarrer après installation

3. **Vérifier l'Installation**
   ```powershell
   nvcc --version
   # Devrait afficher la version CUDA
   ```

### Vérifier l'Utilisation GPU

Flemme utilisera automatiquement le GPU s'il est disponible.

**Vérifier pendant l'utilisation** :
1. Ouvrir le **Gestionnaire des tâches** (`Ctrl+Shift+Esc`)
2. Onglet **Performance**
3. Sélectionner **GPU**
4. Pendant une transcription, l'utilisation GPU devrait augmenter

**Note** : Si CUDA n'est pas disponible, Whisper fonctionnera sur CPU (plus lent mais fonctionnel).

---

## Dépannage Windows

### Problème : "L'application ne peut pas démarrer car VCRUNTIME140.dll est manquant"

**Cause** : Visual C++ Redistributable manquant

**Solution** :
1. Télécharger : https://aka.ms/vs/17/release/vc_redist.x64.exe
2. Installer le package
3. Redémarrer le PC
4. Relancer Flemme

---

### Problème : "Microphone non détecté"

**Solutions** :

1. **Vérifier les Permissions**
   ```powershell
   ms-settings:privacy-microphone
   ```
   - Activer toutes les options

2. **Tester le Microphone**
   - Ouvrir "Enregistreur vocal" Windows
   - Essayer d'enregistrer
   - Si ça ne fonctionne pas → problème hardware ou drivers

3. **Rafraîchir les Périphériques**
   - Dans Flemme : Paramètres > Matériel > Bouton Rafraîchir
   - Débrancher et rebrancher le microphone USB
   - Redémarrer Flemme

4. **Vérifier les Drivers**
   - Gestionnaire de périphériques (`devmgmt.msc`)
   - "Contrôleurs audio, vidéo et jeu"
   - Clic droit sur le microphone > Mettre à jour le pilote

---

### Problème : "Hotkey ne fonctionne pas"

**Solutions** :

1. **Tester avec un Raccourci Différent**
   - Paramètres Flemme > Modifier le raccourci
   - Essayer `Ctrl+Shift+F12` (peu utilisé)

2. **Fermer les Applications Conflictuelles**
   - Fermer OBS, ShareX, etc.
   - Réessayer

3. **Relancer Flemme**
   - Fermer complètement Flemme (vérifier la barre des tâches)
   - Relancer

4. **Lancer en Administrateur** (dernier recours)
   - Clic droit > Exécuter en tant qu'administrateur

---

### Problème : "Collage automatique ne fonctionne pas"

**Solutions** :

1. **Vérifier que l'Option est Activée**
   - Paramètres Flemme > Collage automatique : **ON**

2. **Vérifier le Focus de l'Application Cible**
   - Le collage ne fonctionne que si l'application cible a le focus
   - Cliquer dans le champ de texte cible avant d'enregistrer

3. **Tester Manuellement**
   - Désactiver "Collage automatique"
   - Faire un enregistrement
   - Appuyer sur `Ctrl+V` manuellement
   - Si ça fonctionne → problème de timing

4. **Augmenter le Délai** (si nécessaire - modification code)
   - Par défaut, Flemme attend 100ms avant de coller
   - Peut nécessiter d'augmenter pour certaines applications lentes

---

### Problème : "Transcription très lente"

**Causes possibles** :

1. **CPU faible**
   - Whisper est gourmand en CPU
   - Modèle "Small" nécessite un CPU moderne

2. **Pas d'accélération GPU**
   - Installer CUDA pour accélérer avec GPU NVIDIA
   - Voir [Configuration GPU](#configuration-gpu-optionnel)

3. **Modèle trop lourd**
   - Utiliser "ggml-base.bin" au lieu de "ggml-small.bin"
   - Plus rapide mais moins précis

**Télécharger un modèle plus léger** :
```powershell
# Télécharger Whisper Base (~140 MB)
Invoke-WebRequest -Uri "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin" -OutFile "$env:APPDATA\Flemme\models\ggml-base.bin"
```

Puis sélectionner "Whisper Base" dans Flemme > Paramètres > Modèles vocaux.

---

### Problème : "Flemme utilise trop de RAM"

**Solutions** :

1. **Fermer et Relancer**
   - Fermer complètement Flemme
   - Relancer (libère la mémoire)

2. **Utiliser un Modèle Plus Léger**
   - Base (140 MB) au lieu de Small (466 MB)
   - Consomme moins de RAM

3. **Vérifier les Fuites Mémoire**
   - Gestionnaire des tâches > Flemme
   - Si l'utilisation augmente continuellement sans enregistrement → bug
   - Redémarrer Flemme régulièrement

---

### Problème : "Fichiers de modèles corrompus"

**Solution** :

```powershell
# Supprimer les modèles existants
Remove-Item "$env:APPDATA\Flemme\models\*.bin"
Remove-Item "$env:APPDATA\Flemme\models\*.onnx"

# Retélécharger
.\download-whisper-model.ps1
.\download-silero-vad.ps1
```

---

## Chemins Windows Importants

| Élément | Chemin |
|---------|--------|
| Modèles ML | `%APPDATA%\Flemme\models\` |
| Configuration | `%APPDATA%\Flemme\settings.json` |
| Logs (si activés) | `%APPDATA%\Flemme\logs\` |
| Exécutable Dev | `flemme-app\src-tauri\target\debug\flemme-app.exe` |
| Exécutable Release | `flemme-app\src-tauri\target\release\flemme-app.exe` |
| Installer MSI | `flemme-app\src-tauri\target\release\bundle\msi\` |

---

## Variables d'Environnement Utiles

Aucune variable d'environnement n'est **requise**, mais vous pouvez en définir pour personnaliser :

```powershell
# Exemple : Changer le répertoire de modèles (non implémenté par défaut)
$env:FLEMME_MODELS_PATH = "D:\Flemme\models"

# Exemple : Activer les logs de debug (non implémenté par défaut)
$env:RUST_LOG = "debug"
```

**Note** : Ces variables ne sont pas utilisées par défaut. Elles sont mentionnées pour référence future.

---

## Résumé des Configurations Essentielles

| Configuration | Statut | Action |
|---------------|--------|--------|
| Permissions Microphone | ✅ Obligatoire | `ms-settings:privacy-microphone` |
| Permissions Clipboard | ✅ Automatique | Rien à faire |
| PowerShell Execution Policy | ⚠️ Requis pour scripts | `Set-ExecutionPolicy Bypass -Scope Process` |
| Périphérique Audio | ✅ Obligatoire | Sélectionner dans Paramètres Flemme |
| Hotkeys | ✅ Automatique | Configurable dans Paramètres |
| CUDA (GPU) | ❌ Optionnel | Installer si GPU NVIDIA disponible |
| Windows Defender Exception | ⚠️ Si bloqué | Ajouter exception |

---

**Configuration Windows terminée ! Flemme devrait maintenant fonctionner parfaitement. 🎉**

Pour toute question supplémentaire, consultez [MIGRATION.md](MIGRATION.md) ou [SETUP_CHECKLIST.md](SETUP_CHECKLIST.md).
