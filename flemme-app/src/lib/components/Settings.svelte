<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { Switch } from "$lib/components/ui/switch";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";

  // Type definitions
  interface AppSettings {
    hotkey: string;
    language: string;
    auto_paste: boolean;
    model_name: string;
    push_to_talk: boolean;
    cancel_key: string;
    device_name: string | null;
  }

  interface AudioDevice {
    name: string;
    is_default: boolean;
  }

  // Settings state
  let activeTab = $state("parametres");
  let hotkey = $state("Ctrl+Alt+R");
  let cancelKey = $state("Escape");
  let language = $state("fr");
  let autoPaste = $state(true);
  let pushToTalk = $state(false);
  let selectedModel = $state("ggml-small.bin");
  let selectedDevice = $state<string | null>(null);
  let audioDevices = $state<AudioDevice[]>([]);
  let loading = $state(true);
  let saveStatus = $state("");
  let customWords = $state<string[]>(["Aymeric Bellavoine", "PPAT", "Harmonie Mutuelle"]);
  let newWord = $state("");
  let isEditingHotkey = $state(false);
  let isEditingCancelKey = $state(false);
  let capturedKeys = $state<string[]>([]);
  let capturedCancelKeys = $state<string[]>([]);

  const languages = [
    { value: "fr", label: "Français" },
    { value: "en", label: "English" },
    { value: "es", label: "Español" },
    { value: "de", label: "Deutsch" }
  ];

  const models = [
    {
      value: "ggml-tiny.bin",
      label: "Tiny",
      size: "75 MB",
      precision: 2,
      speed: 5
    },
    {
      value: "ggml-base.bin",
      label: "Base",
      size: "142 MB",
      precision: 3,
      speed: 4
    },
    {
      value: "ggml-small.bin",
      label: "Small",
      size: "466 MB",
      precision: 4,
      speed: 3
    },
    {
      value: "ggml-medium.bin",
      label: "Medium",
      size: "1.5 GB",
      precision: 5,
      speed: 2
    }
  ];

  // Load settings on mount
  onMount(async () => {
    try {
      const settings = await invoke<AppSettings>("get_settings");
      hotkey = settings.hotkey;
      cancelKey = settings.cancel_key;
      language = settings.language;
      autoPaste = settings.auto_paste;
      selectedModel = settings.model_name;
      pushToTalk = settings.push_to_talk;
      selectedDevice = settings.device_name;
      console.log("Settings loaded:", settings);

      // Load audio devices
      try {
        const devices = await invoke<[string, boolean][]>("get_audio_devices");
        audioDevices = devices.map(([name, is_default]) => ({ name, is_default }));
        console.log("Audio devices loaded:", audioDevices);
      } catch (error) {
        console.error("Failed to load audio devices:", error);
      }
    } catch (error) {
      console.error("Failed to load settings:", error);
    } finally {
      loading = false;
    }
  });

  // Auto-save when language, autoPaste, pushToTalk, or selectedDevice changes
  $effect(() => {
    if (loading) return; // Don't save during initial load

    // Auto-save settings changes
    (async () => {
      try {
        await invoke("save_settings", {
          settings: {
            hotkey,
            cancel_key: cancelKey,
            language,
            auto_paste: autoPaste,
            model_name: selectedModel,
            push_to_talk: pushToTalk,
            device_name: selectedDevice
          }
        });
        console.log("Settings auto-saved");
      } catch (error) {
        console.error("Failed to auto-save settings:", error);
      }
    })();
  });

  async function handleSave() {
    try {
      // Get current settings to check if model changed
      const currentSettings = await invoke<AppSettings>("get_settings");
      const modelChanged = currentSettings.model_name !== selectedModel;

      // Save settings
      await invoke("save_settings", {
        settings: {
          hotkey,
          language,
          auto_paste: autoPaste,
          model_name: selectedModel
        }
      });

      // Reload model if it changed
      if (modelChanged) {
        try {
          await invoke("reload_model", { modelName: selectedModel });
          saveStatus = "Paramètres enregistrés et modèle rechargé!";
        } catch (error) {
          console.error("Failed to reload model:", error);
          saveStatus = "Paramètres sauvegardés mais échec du rechargement du modèle: " + error;
        }
      } else {
        saveStatus = "Paramètres enregistrés avec succès!";
      }

      setTimeout(() => saveStatus = "", 3000);
    } catch (error) {
      console.error("Failed to save settings:", error);
      saveStatus = "Erreur lors de l'enregistrement";
      setTimeout(() => saveStatus = "", 3000);
    }
  }

  function addCustomWord() {
    if (newWord.trim() && !customWords.includes(newWord.trim())) {
      customWords = [...customWords, newWord.trim()];
      newWord = "";
    }
  }

  function removeCustomWord(word: string) {
    customWords = customWords.filter(w => w !== word);
  }

  function clearAllWords() {
    customWords = [];
  }

  function startEditingHotkey() {
    isEditingHotkey = true;
    capturedKeys = [];
  }

  function cancelEditingHotkey() {
    isEditingHotkey = false;
    capturedKeys = [];
  }

  function handleHotkeyCapture(event: KeyboardEvent) {
    if (!isEditingHotkey) return;

    event.preventDefault();
    event.stopPropagation();

    const keys: string[] = [];

    if (event.ctrlKey) keys.push("Ctrl");
    if (event.altKey) keys.push("Alt");
    if (event.shiftKey) keys.push("Shift");
    if (event.metaKey) keys.push("Meta");

    // Mapper les touches spéciales vers le format attendu par Tauri
    const keyMappings: Record<string, string> = {
      " ": "Space",
      "Enter": "Enter",
      "Tab": "Tab",
      "Escape": "Escape",
      "Backspace": "Backspace",
      "Delete": "Delete",
      "Insert": "Insert",
      "Home": "Home",
      "End": "End",
      "PageUp": "PageUp",
      "PageDown": "PageDown",
      "ArrowUp": "Up",
      "ArrowDown": "Down",
      "ArrowLeft": "Left",
      "ArrowRight": "Right",
      "F1": "F1", "F2": "F2", "F3": "F3", "F4": "F4",
      "F5": "F5", "F6": "F6", "F7": "F7", "F8": "F8",
      "F9": "F9", "F10": "F10", "F11": "F11", "F12": "F12"
    };

    // Ajouter la touche principale (pas les modificateurs)
    const mainKey = event.key;
    if (!["Control", "Alt", "Shift", "Meta"].includes(mainKey)) {
      const mappedKey = keyMappings[mainKey] || mainKey.toUpperCase();
      keys.push(mappedKey);
    }

    if (keys.length > 0) {
      capturedKeys = keys;
    }
  }

  async function saveHotkey() {
    if (capturedKeys.length === 0) {
      cancelEditingHotkey();
      return;
    }

    const newHotkey = capturedKeys.join("+");

    try {
      await invoke("update_hotkey", { newHotkey });
      hotkey = newHotkey;
      isEditingHotkey = false;
      capturedKeys = [];
      saveStatus = "Raccourci clavier mis à jour!";
      setTimeout(() => saveStatus = "", 3000);
    } catch (error) {
      console.error("Failed to update hotkey:", error);
      saveStatus = "Erreur: " + error;
      setTimeout(() => saveStatus = "", 3000);
    }
  }

  function startEditingCancelKey() {
    isEditingCancelKey = true;
    capturedCancelKeys = [];
  }

  function cancelEditingCancelKey() {
    isEditingCancelKey = false;
    capturedCancelKeys = [];
  }

  function handleCancelKeyCapture(event: KeyboardEvent) {
    if (!isEditingCancelKey) return;

    event.preventDefault();
    event.stopPropagation();

    const keys: string[] = [];

    if (event.ctrlKey) keys.push("Ctrl");
    if (event.altKey) keys.push("Alt");
    if (event.shiftKey) keys.push("Shift");
    if (event.metaKey) keys.push("Meta");

    // Mapper les touches spéciales vers le format attendu par Tauri
    const keyMappings: Record<string, string> = {
      " ": "Space",
      "Enter": "Enter",
      "Tab": "Tab",
      "Escape": "Escape",
      "Backspace": "Backspace",
      "Delete": "Delete",
      "Insert": "Insert",
      "Home": "Home",
      "End": "End",
      "PageUp": "PageUp",
      "PageDown": "PageDown",
      "ArrowUp": "Up",
      "ArrowDown": "Down",
      "ArrowLeft": "Left",
      "ArrowRight": "Right",
      "F1": "F1", "F2": "F2", "F3": "F3", "F4": "F4",
      "F5": "F5", "F6": "F6", "F7": "F7", "F8": "F8",
      "F9": "F9", "F10": "F10", "F11": "F11", "F12": "F12"
    };

    // Ajouter la touche principale (pas les modificateurs)
    const mainKey = event.key;
    if (!["Control", "Alt", "Shift", "Meta"].includes(mainKey)) {
      const mappedKey = keyMappings[mainKey] || mainKey.toUpperCase();
      keys.push(mappedKey);
    }

    if (keys.length > 0) {
      capturedCancelKeys = keys;
    }
  }

  async function saveCancelKey() {
    if (capturedCancelKeys.length === 0) {
      cancelEditingCancelKey();
      return;
    }

    const newCancelKey = capturedCancelKeys.join("+");

    try {
      await invoke("update_cancel_key", { newCancelKey });
      cancelKey = newCancelKey;
      isEditingCancelKey = false;
      capturedCancelKeys = [];
      saveStatus = "Touche d'annulation mise à jour!";
      setTimeout(() => saveStatus = "", 3000);
    } catch (error) {
      console.error("Failed to update cancel key:", error);
      saveStatus = "Erreur: " + error;
      setTimeout(() => saveStatus = "", 3000);
    }
  }

  async function refreshAudioDevices() {
    try {
      const devices = await invoke<[string, boolean][]>("get_audio_devices");
      audioDevices = devices.map(([name, is_default]) => ({ name, is_default }));
      console.log("Audio devices refreshed:", audioDevices);
      saveStatus = "Périphériques audio rafraîchis!";
      setTimeout(() => saveStatus = "", 2000);
    } catch (error) {
      console.error("Failed to refresh audio devices:", error);
      saveStatus = "Erreur lors du rafraîchissement";
      setTimeout(() => saveStatus = "", 2000);
    }
  }
</script>

<div class="app-container">
  <!-- Left Sidebar -->
  <aside class="sidebar">
    <div class="logo">
      <h1>Flemme</h1>
    </div>

    <nav class="nav-menu">
      <button
        class="nav-item {activeTab === 'parametres' ? 'active' : ''}"
        onclick={() => activeTab = 'parametres'}
      >
        <img src="/icons/Parametres.svg" alt="" class="nav-icon" />
        <span>Paramètres</span>
      </button>

      <button
        class="nav-item {activeTab === 'modeles' ? 'active' : ''}"
        onclick={() => activeTab = 'modeles'}
      >
        <img src="/icons/ModeleIA.svg" alt="" class="nav-icon" />
        <span>Modèles IA</span>
      </button>

      <button
        class="nav-item {activeTab === 'vocabulaire' ? 'active' : ''}"
        onclick={() => activeTab = 'vocabulaire'}
      >
        <img src="/icons/Vocabulaire.svg" alt="" class="nav-icon" />
        <span>Vocabulaire</span>
      </button>
    </nav>
  </aside>

  <!-- Main Content -->
  <main class="main-content">
    {#if activeTab === 'parametres'}
      <div class="content-section">
        <h2 class="section-title">Configuration</h2>

        <div class="setting-group">
          <label class="setting-label">Démarrer un enregistrement</label>
          {#if isEditingHotkey}
            <div class="hotkey-editor">
              <input
                type="text"
                class="hotkey-input"
                placeholder="Appuyez sur les touches..."
                readonly
                value={capturedKeys.length > 0 ? capturedKeys.join(" + ") : ""}
                onkeydown={handleHotkeyCapture}
                autofocus
              />
              <div class="hotkey-editor-buttons">
                <Button onclick={saveHotkey} size="sm">Valider</Button>
                <Button onclick={cancelEditingHotkey} variant="outline" size="sm">Annuler</Button>
              </div>
            </div>
          {:else}
            <div class="hotkey-display-row">
              <div class="hotkey-display">
                {#each hotkey.split("+") as key, i}
                  {#if i > 0}<span class="hotkey-plus">+</span>{/if}
                  <Badge variant="secondary">{key.trim().toLowerCase()}</Badge>
                {/each}
              </div>
              <Button onclick={startEditingHotkey} variant="outline" size="sm">Modifier</Button>
            </div>
          {/if}
        </div>

        <div class="setting-group {pushToTalk ? 'disabled' : ''}">
          <label class="setting-label">Annuler un enregistrement</label>
          {#if isEditingCancelKey}
            <div class="hotkey-editor">
              <input
                type="text"
                class="hotkey-input"
                placeholder="Appuyez sur les touches..."
                readonly
                value={capturedCancelKeys.length > 0 ? capturedCancelKeys.join(" + ") : ""}
                onkeydown={handleCancelKeyCapture}
                autofocus
              />
              <div class="hotkey-editor-buttons">
                <Button onclick={saveCancelKey} size="sm">Valider</Button>
                <Button onclick={cancelEditingCancelKey} variant="outline" size="sm">Annuler</Button>
              </div>
            </div>
          {:else}
            <div class="hotkey-display-row">
              <div class="hotkey-display">
                {#each cancelKey.split("+") as key, i}
                  {#if i > 0}<span class="hotkey-plus">+</span>{/if}
                  <Badge variant="secondary">{key.trim().toLowerCase()}</Badge>
                {/each}
              </div>
              <Button onclick={startEditingCancelKey} variant="outline" size="sm" disabled={pushToTalk}>Modifier</Button>
            </div>
          {/if}
          {#if pushToTalk}
            <p class="disabled-hint">Non disponible en mode Push-to-Talk</p>
          {/if}
        </div>

        <div class="setting-group">
          <div class="setting-row">
            <label class="setting-label">Collage automatique</label>
            <Switch bind:checked={autoPaste} />
          </div>
        </div>

        <div class="setting-group">
          <div class="setting-row">
            <label class="setting-label">Push To Talk</label>
            <Switch bind:checked={pushToTalk} />
          </div>
        </div>

        <div class="setting-group">
          <label class="setting-label">Langue</label>
          <select class="select-input" bind:value={language}>
            {#each languages as lang}
              <option value={lang.value}>{lang.label}</option>
            {/each}
          </select>
        </div>

        <div class="setting-group">
          <label class="setting-label">Matériel</label>
          <div class="device-selector-row">
            <select class="select-input" bind:value={selectedDevice}>
              <option value={null}>Microphone par défaut</option>
              {#each audioDevices as device}
                <option value={device.name}>
                  {device.name}{device.is_default ? " (par défaut)" : ""}
                </option>
              {/each}
            </select>
            <Button onclick={refreshAudioDevices} variant="outline" size="sm">
              <svg width="16" height="16" viewBox="0 0 16 16" fill="none" style="margin-right: 4px;">
                <path d="M13.65 2.35C12.2 0.9 10.21 0 8 0v2c1.66 0 3.14.69 4.22 1.78l-1.51 1.51L15 7V2l-1.35 1.35zM2 7l4.29-1.71-1.51-1.51C5.86 2.69 7.34 2 9 2V0C6.79 0 4.8.9 3.35 2.35L2 1v6zm6 9c-1.66 0-3.14-.69-4.22-1.78l1.51-1.51L1 11v5l1.35-1.35C3.8 15.1 5.79 16 8 16v-2zm5.65-2.35C12.86 14.31 11.34 15 9.66 15v2c2.21 0 4.2-.9 5.65-2.35L16 16v-6l-4.29 1.71 1.51 1.51z" fill="currentColor"/>
              </svg>
              Rafraîchir
            </Button>
          </div>
        </div>

        <div class="button-group">
          {#if saveStatus}
            <p class="save-status {saveStatus.includes('succès') ? 'success' : 'error'}">
              {saveStatus}
            </p>
          {/if}
          <Button onclick={handleSave} disabled={loading} class="save-button">
            Enregistrer
          </Button>
        </div>
      </div>
    {:else if activeTab === 'modeles'}
      <div class="content-section">
        <h2 class="section-title">Modèles disponibles</h2>

        <div class="models-list">
          {#each models as model}
            <div class="model-item">
              <label class="model-radio">
                <input
                  type="radio"
                  name="model"
                  value={model.value}
                  checked={selectedModel === model.value}
                  onchange={() => selectedModel = model.value}
                />
                <div class="model-info">
                  <div class="model-header">
                    <span class="model-name">{model.label}</span>
                    <span class="model-size">{model.size}</span>
                  </div>

                  <div class="model-metrics">
                    <div class="metric">
                      <span class="metric-label">Précision</span>
                      <div class="metric-dots">
                        {#each Array(5) as _, i}
                          <span class="dot {i < model.precision ? 'filled' : ''}"></span>
                        {/each}
                      </div>
                    </div>

                    <div class="metric">
                      <span class="metric-label">Rapidité</span>
                      <div class="metric-dots">
                        {#each Array(5) as _, i}
                          <span class="dot {i < model.speed ? 'filled' : ''}"></span>
                        {/each}
                      </div>
                    </div>
                  </div>
                </div>
              </label>

              <div class="model-actions">
                <button class="icon-button">
                  <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
                    <path d="M8 2V14M2 8H14" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
                  </svg>
                </button>
                <button class="icon-button delete">
                  <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
                    <path d="M2 4H14M6 4V2H10V4M3 4V14H13V4" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
                  </svg>
                </button>
              </div>
            </div>
          {/each}
        </div>
      </div>
    {:else if activeTab === 'vocabulaire'}
      <div class="content-section">
        <h2 class="section-title">Bibliothèque de mots</h2>

        <div class="vocabulary-input-group">
          <input
            type="text"
            bind:value={newWord}
            placeholder="mot personnalisé"
            class="vocab-input"
            onkeydown={(e) => e.key === 'Enter' && addCustomWord()}
          />
          <Button onclick={addCustomWord} class="add-button">Ajouter</Button>
          <Button onclick={clearAllWords} variant="outline" class="clear-button">
            Tout effacer
          </Button>
        </div>

        <div class="custom-words-list">
          {#each customWords as word}
            <div class="word-tag">
              <span>{word}</span>
              <button class="remove-word" onclick={() => removeCustomWord(word)}>
                <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
                  <path d="M2 2L10 10M2 10L10 2" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
                </svg>
              </button>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  </main>
</div>

<style>
  :global(body) {
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
    margin: 0;
    padding: 0;
  }

  .app-container {
    display: flex;
    height: 100vh;
    background: white;
  }

  /* Sidebar */
  .sidebar {
    width: 240px;
    background: #F3F3F3;
    padding: 32px 24px;
    display: flex;
    flex-direction: column;
  }

  .logo h1 {
    font-family: 'Nunito', sans-serif;
    font-weight: 900;
    font-size: 32px;
    margin: 0 0 48px 0;
    color: #000;
  }

  .nav-menu {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    background: transparent;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    font-size: 14px;
    font-weight: 500;
    color: #666;
    transition: all 0.2s;
  }

  .nav-item:hover {
    background: rgba(0, 0, 0, 0.05);
  }

  .nav-item.active {
    background: #4FB094;
    color: white;
  }

  .nav-icon {
    width: 20px;
    height: 20px;
  }

  /* Main Content */
  .main-content {
    flex: 1;
    padding: 48px;
    overflow-y: auto;
  }

  .content-section {
    max-width: 600px;
  }

  .section-title {
    font-size: 24px;
    font-weight: 500;
    margin: 0 0 32px 0;
    color: #000;
  }

  .setting-group {
    margin-bottom: 24px;
  }

  .setting-group.disabled {
    opacity: 0.5;
    pointer-events: none;
  }

  .disabled-hint {
    margin-top: 8px;
    font-size: 12px;
    color: #999;
    font-style: italic;
  }

  .setting-label {
    display: block;
    font-size: 14px;
    font-weight: 300;
    color: #666;
    margin-bottom: 8px;
  }

  .setting-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .hotkey-display {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .hotkey-display-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
  }

  .hotkey-plus {
    color: #999;
    font-size: 14px;
  }

  .hotkey-editor {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .hotkey-input {
    width: 100%;
    padding: 10px 12px;
    border: 2px solid #4FB094;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 500;
    background: white;
    color: #000;
    text-align: center;
  }

  .hotkey-input:focus {
    outline: none;
    border-color: #3A8B75;
  }

  .hotkey-editor-buttons {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
  }

  .select-input {
    width: 100%;
    padding: 10px 12px;
    border: 1px solid #E0E0E0;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 300;
    background: white;
    cursor: pointer;
  }

  .device-selector-row {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .device-selector-row .select-input {
    flex: 1;
  }

  .button-group {
    margin-top: 32px;
    display: flex;
    justify-content: flex-end;
    align-items: center;
    gap: 16px;
  }

  .save-status {
    font-size: 14px;
  }

  .save-status.success {
    color: #4FB094;
  }

  .save-status.error {
    color: #EF4444;
  }

  /* Models Tab */
  .models-list {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .model-item {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 16px;
    border: 1px solid #E0E0E0;
    border-radius: 8px;
  }

  .model-radio {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 12px;
    cursor: pointer;
  }

  .model-radio input[type="radio"] {
    width: 18px;
    height: 18px;
    cursor: pointer;
  }

  .model-info {
    flex: 1;
  }

  .model-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
  }

  .model-name {
    font-size: 16px;
    font-weight: 500;
    color: #000;
  }

  .model-size {
    font-size: 14px;
    font-weight: 300;
    color: #666;
  }

  .model-metrics {
    display: flex;
    gap: 24px;
  }

  .metric {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .metric-label {
    font-size: 12px;
    font-weight: 300;
    color: #666;
  }

  .metric-dots {
    display: flex;
    gap: 4px;
  }

  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #E0E0E0;
  }

  .dot.filled {
    background: #4FB094;
  }

  .model-actions {
    display: flex;
    gap: 8px;
  }

  .icon-button {
    padding: 8px;
    background: transparent;
    border: 1px solid #E0E0E0;
    border-radius: 6px;
    cursor: pointer;
    color: #666;
    transition: all 0.2s;
  }

  .icon-button:hover {
    background: #F3F3F3;
  }

  .icon-button.delete {
    color: #EF4444;
  }

  /* Vocabulary Tab */
  .vocabulary-input-group {
    display: flex;
    gap: 12px;
    margin-bottom: 24px;
  }

  .vocab-input {
    flex: 1;
    padding: 10px 12px;
    border: 1px solid #E0E0E0;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 300;
  }

  .vocab-input::placeholder {
    color: #999;
  }

  .custom-words-list {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
  }

  .word-tag {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    background: #F3F3F3;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 300;
  }

  .remove-word {
    padding: 2px;
    background: transparent;
    border: none;
    cursor: pointer;
    color: #999;
    display: flex;
    align-items: center;
    transition: color 0.2s;
  }

  .remove-word:hover {
    color: #EF4444;
  }
</style>
