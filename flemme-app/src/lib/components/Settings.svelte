<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { Switch } from "$lib/components/ui/switch";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import UpdateChecker from "$lib/components/UpdateChecker.svelte";

  // Type definitions
  interface AppSettings {
    hotkey: string;
    language: string;
    auto_paste: boolean;
    model_name: string;
    push_to_talk: boolean;
    cancel_key: string;
    device_name: string | null;
    custom_words: string[];
  }

  interface LlmModel {
    id: string;
    name: string;
    api_url: string;
    model_name: string;
  }

  interface ExecutionMode {
    id: string;
    name: string;
    llm_model_id: string | null;
    system_prompt: string;
  }

  interface AudioDevice {
    name: string;
    is_default: boolean;
  }

  interface ModelInfo {
    name: string;
    size_mb: number;
    is_downloaded: boolean;
    download_url: string;
  }

  interface DownloadProgress {
    model_name: string;
    downloaded_bytes: number;
    total_bytes: number;
    percentage: number;
  }

  // Settings state
  let activeTab = $state("parametres");
  let modelsSubTab = $state("vocaux"); // Sub-tab for "IA et Modèles" section
  let hotkey = $state("Ctrl+Alt+R");
  let cancelKey = $state("Escape");
  let language = $state("fr");
  let autoPaste = $state(true);
  let pushToTalk = $state(false);
  let selectedModel = $state("ggml-small-q5_1.bin");
  let selectedDevice = $state<string | null>(null);
  let audioDevices = $state<AudioDevice[]>([]);
  let loading = $state(true);
  let saveStatus = $state("");
  let isInitialLoad = $state(true);
  let customWords = $state<string[]>(["Aymeric Bellavoine", "PPAT", "Harmonie Mutuelle"]);
  let newWord = $state("");
  let isEditingHotkey = $state(false);
  let isEditingCancelKey = $state(false);
  let capturedKeys = $state<string[]>([]);
  let capturedCancelKeys = $state<string[]>([]);

  // Model management state
  let availableModels = $state<ModelInfo[]>([]);
  let downloadingModel = $state<string | null>(null);
  let downloadProgress = $state<Record<string, number>>({});

  // LLM management state
  let llmModels = $state<LlmModel[]>([]);
  let editingLlm = $state<LlmModel | null>(null);
  let newLlmName = $state("");
  let newLlmApiUrl = $state("");
  let newLlmModelName = $state("");
  let newLlmApiKey = $state("");
  let newLlmServiceType = $state("openrouter");
  let showAddLlmForm = $state(false);

  // Local LLM providers state
  let lmStudioModels = $state<any[]>([]);
  let ollamaModels = $state<any[]>([]);
  let detectingModels = $state(false);
  let serviceStatus = $state<Record<string, boolean>>({});

  // Execution modes state
  let executionModes = $state<ExecutionMode[]>([]);
  let activeMode = $state("standard");
  let editingMode = $state<ExecutionMode | null>(null);
  let newModeName = $state("");
  let newModeSystemPrompt = $state("");
  let newModeLlmId = $state<string | null>(null);
  let showAddModeForm = $state(false);
  let showWelcomeModal = $state(false);

  const languages = [
    { value: "fr", label: "Français" },
    { value: "en", label: "English" },
    { value: "es", label: "Español" },
    { value: "de", label: "Deutsch" }
  ];

  // Load settings on mount
  onMount(() => {
    // Listen to download progress events
    let unlisten: (() => void) | undefined;

    listen<DownloadProgress>("download-progress", (event) => {
      const progress = event.payload;
      console.log("Download progress event received:", progress);
      downloadProgress = {
        ...downloadProgress,
        [progress.model_name]: progress.percentage
      };
    }).then((unlistenFn) => {
      unlisten = unlistenFn;
      console.log("Download progress listener registered");
    });

    (async () => {
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

      // Load custom words
      try {
        const words = await invoke<string[]>("get_custom_words");
        customWords = words;
        console.log("Custom words loaded:", customWords);
      } catch (error) {
        console.error("Failed to load custom words:", error);
      }

      try {
        const models = await invoke<ModelInfo[]>("list_available_models");
        availableModels = models;
        console.log("Available models loaded:", availableModels);

        // Check if any model is downloaded
        const hasDownloadedModel = models.some(m => m.is_downloaded);
        if (!hasDownloadedModel) {
          console.log("No models downloaded, showing welcome modal");
          showWelcomeModal = true;
          activeTab = 'modeles';
          modelsSubTab = 'vocaux';
        }
      } catch (error) {
        console.error("Failed to load available models:", error);
      }

      // Load LLM models
      try {
        const models = await invoke<LlmModel[]>("get_llm_models");
        llmModels = models;
        console.log("LLM models loaded:", llmModels);
      } catch (error) {
        console.error("Failed to load LLM models:", error);
      }

      // Load execution modes
      try {
        const modes = await invoke<ExecutionMode[]>("get_execution_modes");
        executionModes = modes;
        console.log("Execution modes loaded:", executionModes);
      } catch (error) {
        console.error("Failed to load execution modes:", error);
      }

      // Load active mode
      try {
        const mode = await invoke<string>("get_active_mode");
        activeMode = mode;
        console.log("Active mode loaded:", activeMode);
      } catch (error) {
        console.error("Failed to load active mode:", error);
      }
      } catch (error) {
        console.error("Failed to load settings:", error);
      } finally {
        loading = false;
        // Mark initial load as complete after a short delay to ensure all reactive updates are done
        setTimeout(() => {
          isInitialLoad = false;
        }, 100);
      }
    })();

    // Cleanup listener on component unmount
    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  });

  // Auto-save when language, autoPaste, pushToTalk, or selectedDevice changes
  $effect(() => {
    if (loading || isInitialLoad) return; // Don't save during initial load

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
            device_name: selectedDevice,
            custom_words: customWords,
            llm_models: llmModels,
            execution_modes: executionModes,
            active_mode: activeMode
          }
        });
        console.log("Settings auto-saved");
      } catch (error) {
        console.error("Failed to auto-save settings:", error);
      }
    })();
  });

  // Track previous model to detect changes
  let previousModel = $state<string | null>(null);

  // Auto-reload model when selectedModel changes
  $effect(() => {
    if (loading || isInitialLoad) return; // Don't reload during initial load

    // Skip if this is the first time we're setting the model
    if (previousModel === null) {
      previousModel = selectedModel;
      return;
    }

    // If model changed, reload it
    if (previousModel !== selectedModel) {
      console.log(`Model changed from ${previousModel} to ${selectedModel}, reloading...`);

      (async () => {
        try {
          await invoke("reload_model", { modelName: selectedModel });
          saveStatus = `Modèle ${getModelLabel(selectedModel)} chargé avec succès!`;
          setTimeout(() => saveStatus = "", 3000);
          previousModel = selectedModel; // Update tracking
        } catch (error) {
          console.error("Failed to reload model:", error);
          saveStatus = `Erreur lors du chargement du modèle: ${error}`;
          setTimeout(() => saveStatus = "", 3000);
        }
      })();
    }
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
          cancel_key: cancelKey,
          language,
          auto_paste: autoPaste,
          model_name: selectedModel,
          push_to_talk: pushToTalk,
          device_name: selectedDevice,
          custom_words: customWords,
          llm_models: llmModels,
          execution_modes: executionModes,
          active_mode: activeMode
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

  async function addCustomWord() {
    const trimmedWord = newWord.trim();
    if (trimmedWord && !customWords.includes(trimmedWord)) {
      try {
        await invoke("add_custom_word", { word: trimmedWord });
        customWords = [...customWords, trimmedWord];
        newWord = "";
        console.log("Custom word added:", trimmedWord);
      } catch (error) {
        console.error("Failed to add custom word:", error);
        saveStatus = "Erreur lors de l'ajout du mot";
        setTimeout(() => saveStatus = "", 2000);
      }
    }
  }

  async function removeCustomWord(word: string) {
    try {
      await invoke("remove_custom_word", { word });
      customWords = customWords.filter(w => w !== word);
      console.log("Custom word removed:", word);
    } catch (error) {
      console.error("Failed to remove custom word:", error);
      saveStatus = "Erreur lors de la suppression";
      setTimeout(() => saveStatus = "", 2000);
    }
  }

  async function clearAllWords() {
    try {
      await invoke("clear_custom_words");
      customWords = [];
      console.log("All custom words cleared");
    } catch (error) {
      console.error("Failed to clear custom words:", error);
      saveStatus = "Erreur lors de la suppression";
      setTimeout(() => saveStatus = "", 2000);
    }
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

  async function downloadModel(modelName: string, downloadUrl: string) {
    try {
      downloadingModel = modelName;
      downloadProgress = { ...downloadProgress, [modelName]: 0 };

      await invoke("download_model", {
        modelName,
        downloadUrl
      });

      // Refresh the models list to update download status
      const models = await invoke<ModelInfo[]>("list_available_models");
      availableModels = models;

      downloadingModel = null;
      const { [modelName]: _, ...rest } = downloadProgress;
      downloadProgress = rest;

      saveStatus = `Modèle ${modelName} téléchargé avec succès!`;
      setTimeout(() => saveStatus = "", 3000);
    } catch (error) {
      console.error("Failed to download model:", error);
      downloadingModel = null;
      const { [modelName]: _, ...rest } = downloadProgress;
      downloadProgress = rest;
      saveStatus = "Erreur lors du téléchargement: " + error;
      setTimeout(() => saveStatus = "", 3000);
    }
  }

  async function deleteModel(modelName: string) {
    if (!confirm(`Êtes-vous sûr de vouloir supprimer le modèle ${modelName} ?`)) {
      return;
    }

    try {
      await invoke("delete_model", { modelName });

      // Refresh the models list to update download status
      const models = await invoke<ModelInfo[]>("list_available_models");
      availableModels = models;

      saveStatus = `Modèle ${modelName} supprimé avec succès!`;
      setTimeout(() => saveStatus = "", 3000);
    } catch (error) {
      console.error("Failed to delete model:", error);
      saveStatus = "Erreur lors de la suppression: " + error;
      setTimeout(() => saveStatus = "", 3000);
    }
  }

  function getModelLabel(modelName: string): string {
    // Check for large-v3-turbo first (more specific)
    if (modelName.includes("ggml-large-v3-turbo")) return "Whisper Large V3 Turbo";
    // Then check for large-v2
    if (modelName.includes("ggml-large-v2")) return "Whisper Large V2";
    // Then check for medium, small, base
    if (modelName.includes("ggml-medium")) return "Whisper Medium";
    if (modelName.includes("ggml-small")) return "Whisper Small";
    if (modelName.includes("ggml-base")) return "Whisper Base";
    // Fallback to original name
    return modelName;
  }

  function formatFileSize(sizeMb: number): string {
    if (sizeMb >= 1000) {
      return `${(sizeMb / 1000).toFixed(1)} GB`;
    }
    return `${Math.round(sizeMb)} MB`;
  }

  // LLM Management functions
  async function addLlmModel() {
    // Validation: API key only required for non-local services
    const requiresApiKey = !["lmstudio", "ollama"].includes(newLlmServiceType);
    if (!newLlmName.trim() || !newLlmApiUrl.trim() || !newLlmModelName.trim() || (requiresApiKey && !newLlmApiKey.trim())) {
      saveStatus = "Tous les champs requis doivent être remplis";
      setTimeout(() => saveStatus = "", 2000);
      return;
    }

    try {
      const id = await invoke<string>("add_llm_model", {
        name: newLlmName.trim(),
        apiUrl: newLlmApiUrl.trim(),
        modelName: newLlmModelName.trim(),
        apiKey: newLlmApiKey.trim() || "",
        serviceType: newLlmServiceType
      });

      llmModels = [...llmModels, {
        id,
        name: newLlmName.trim(),
        api_url: newLlmApiUrl.trim(),
        model_name: newLlmModelName.trim(),
        service_type: newLlmServiceType
      }];

      newLlmName = "";
      newLlmApiUrl = "";
      newLlmModelName = "";
      newLlmApiKey = "";
      newLlmServiceType = "openrouter";
      showAddLlmForm = false;

      saveStatus = "Modèle LLM ajouté avec succès!";
      setTimeout(() => saveStatus = "", 3000);
    } catch (error) {
      console.error("Failed to add LLM model:", error);
      saveStatus = "Erreur: " + error;
      setTimeout(() => saveStatus = "", 3000);
    }
  }

  async function updateLlmModel() {
    if (!editingLlm) return;

    try {
      const updatedModel = { ...editingLlm };

      await invoke("update_llm_model", {
        id: updatedModel.id,
        name: updatedModel.name,
        apiUrl: updatedModel.api_url,
        modelName: updatedModel.model_name,
        apiKey: newLlmApiKey.trim() || null
      });

      llmModels = llmModels.map(m => m.id === updatedModel.id ? updatedModel : m);
      editingLlm = null;
      newLlmApiKey = "";

      saveStatus = "Modèle LLM mis à jour!";
      setTimeout(() => saveStatus = "", 3000);
    } catch (error) {
      console.error("Failed to update LLM model:", error);
      saveStatus = "Erreur: " + error;
      setTimeout(() => saveStatus = "", 3000);
    }
  }

  async function deleteLlmModel(id: string) {
    if (!confirm("Êtes-vous sûr de vouloir supprimer ce modèle LLM ?")) return;

    try {
      await invoke("delete_llm_model", { id });
      llmModels = llmModels.filter(m => m.id !== id);

      saveStatus = "Modèle LLM supprimé!";
      setTimeout(() => saveStatus = "", 3000);
    } catch (error) {
      console.error("Failed to delete LLM model:", error);
      saveStatus = "Erreur: " + error;
      setTimeout(() => saveStatus = "", 3000);
    }
  }

  function startEditingLlm(model: LlmModel) {
    editingLlm = { ...model };
    newLlmApiKey = "";
  }

  function cancelEditingLlm() {
    editingLlm = null;
    newLlmApiKey = "";
  }

  // Local LLM Detection functions
  async function detectLMStudioModels() {
    detectingModels = true;
    try {
      lmStudioModels = await invoke("detect_lm_studio_models", { port: null });
      serviceStatus["lmstudio"] = true;
      saveStatus = `Détecté ${lmStudioModels.length} modèles LM Studio`;
      setTimeout(() => saveStatus = "", 3000);
    } catch (error) {
      console.error("Failed to detect LM Studio models:", error);
      serviceStatus["lmstudio"] = false;
      saveStatus = "LM Studio n'est pas démarré";
      setTimeout(() => saveStatus = "", 3000);
    } finally {
      detectingModels = false;
    }
  }

  async function detectOllamaModels() {
    detectingModels = true;
    try {
      ollamaModels = await invoke("detect_ollama_models", { port: null });
      serviceStatus["ollama"] = true;
      saveStatus = `Détecté ${ollamaModels.length} modèles Ollama`;
      setTimeout(() => saveStatus = "", 3000);
    } catch (error) {
      console.error("Failed to detect Ollama models:", error);
      serviceStatus["ollama"] = false;
      saveStatus = "Ollama n'est pas démarré";
      setTimeout(() => saveStatus = "", 3000);
    } finally {
      detectingModels = false;
    }
  }

  function handleServiceTypeChange() {
    // Auto-fill URL and clear model name based on service type
    switch (newLlmServiceType) {
      case "lmstudio":
        newLlmApiUrl = "http://localhost:1234/v1/chat/completions";
        newLlmModelName = "";
        newLlmApiKey = "";
        detectLMStudioModels();
        break;
      case "ollama":
        newLlmApiUrl = "http://localhost:11434/v1/chat/completions";
        newLlmModelName = "";
        newLlmApiKey = "";
        detectOllamaModels();
        break;
      case "openrouter":
        newLlmApiUrl = "https://openrouter.ai/api/v1/chat/completions";
        newLlmModelName = "";
        break;
      case "openai":
        newLlmApiUrl = "https://api.openai.com/v1/chat/completions";
        newLlmModelName = "";
        break;
      case "gemini":
        newLlmApiUrl = "https://generativelanguage.googleapis.com/v1beta/models/";
        newLlmModelName = "";
        break;
    }
  }

  function selectDetectedModel(model: any) {
    if (newLlmServiceType === "lmstudio") {
      newLlmModelName = model.id;
      newLlmName = `LM Studio - ${model.id}`;
    } else if (newLlmServiceType === "ollama") {
      newLlmModelName = model.name;
      newLlmName = `Ollama - ${model.name}`;
    }
  }

  // Execution Mode Management functions
  async function addExecutionMode() {
    if (!newModeName.trim()) {
      saveStatus = "Le nom du mode est requis";
      setTimeout(() => saveStatus = "", 2000);
      return;
    }

    try {
      const id = await invoke<string>("add_execution_mode", {
        name: newModeName.trim(),
        llmModelId: newModeLlmId,
        systemPrompt: newModeSystemPrompt.trim()
      });

      executionModes = [...executionModes, {
        id,
        name: newModeName.trim(),
        llm_model_id: newModeLlmId,
        system_prompt: newModeSystemPrompt.trim()
      }];

      newModeName = "";
      newModeSystemPrompt = "";
      newModeLlmId = null;
      showAddModeForm = false;

      saveStatus = "Mode d'exécution ajouté!";
      setTimeout(() => saveStatus = "", 3000);
    } catch (error) {
      console.error("Failed to add execution mode:", error);
      saveStatus = "Erreur: " + error;
      setTimeout(() => saveStatus = "", 3000);
    }
  }

  async function updateExecutionMode() {
    if (!editingMode) return;

    try {
      const updatedMode = { ...editingMode };

      await invoke("update_execution_mode", {
        id: updatedMode.id,
        name: updatedMode.name,
        llmModelId: updatedMode.llm_model_id,
        systemPrompt: updatedMode.system_prompt
      });

      executionModes = executionModes.map(m => m.id === updatedMode.id ? updatedMode : m);
      editingMode = null;

      saveStatus = "Mode d'exécution mis à jour!";
      setTimeout(() => saveStatus = "", 3000);
    } catch (error) {
      console.error("Failed to update execution mode:", error);
      saveStatus = "Erreur: " + error;
      setTimeout(() => saveStatus = "", 3000);
    }
  }

  async function deleteExecutionMode(id: string) {
    if (!confirm("Êtes-vous sûr de vouloir supprimer ce mode ?")) return;

    try {
      await invoke("delete_execution_mode", { id });
      executionModes = executionModes.filter(m => m.id !== id);

      saveStatus = "Mode d'exécution supprimé!";
      setTimeout(() => saveStatus = "", 3000);
    } catch (error) {
      console.error("Failed to delete execution mode:", error);
      saveStatus = "Erreur: " + error;
      setTimeout(() => saveStatus = "", 3000);
    }
  }

  async function setActiveModeHandler(modeId: string) {
    try {
      await invoke("set_active_mode", { modeId });
      activeMode = modeId;

      saveStatus = "Mode actif changé!";
      setTimeout(() => saveStatus = "", 3000);
    } catch (error) {
      console.error("Failed to set active mode:", error);
      saveStatus = "Erreur: " + error;
      setTimeout(() => saveStatus = "", 3000);
    }
  }

  function startEditingMode(mode: ExecutionMode) {
    editingMode = { ...mode };
  }

  function cancelEditingMode() {
    editingMode = null;
  }

  function getLlmModelName(llmId: string | null): string {
    if (!llmId) return "Aucun";
    const model = llmModels.find(m => m.id === llmId);
    return model ? model.name : "Inconnu";
  }
</script>

<div class="app-container">
  <!-- Left Sidebar -->
  <aside class="sidebar">
    <div class="logo">
      <img src="/LogoFlemme.png" alt="Flemme" class="logo-image" />
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
        <span>IA et Modèles</span>
      </button>

      <button
        class="nav-item {activeTab === 'vocabulaire' ? 'active' : ''}"
        onclick={() => activeTab = 'vocabulaire'}
      >
        <img src="/icons/Vocabulaire.svg" alt="" class="nav-icon" />
        <span>Vocabulaire</span>
      </button>

      <button
        class="nav-item {activeTab === 'modes' ? 'active' : ''}"
        onclick={() => activeTab = 'modes'}
      >
        <img src="/icons/Mode.svg" alt="" class="nav-icon" />
        <span>Modes</span>
      </button>

      <button
        class="nav-item {activeTab === 'about' ? 'active' : ''}"
        onclick={() => activeTab = 'about'}
      >
        <svg class="nav-icon" width="20" height="20" viewBox="0 0 20 20" fill="none">
          <circle cx="10" cy="10" r="8" stroke="currentColor" stroke-width="1.5"/>
          <path d="M10 6V6.01M10 9V14" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
        </svg>
        <span>À propos</span>
      </button>
    </nav>

    <!-- Active Mode Indicator -->
    <div class="active-mode-indicator">
      <div class="mode-status-row">
        <div class="status-dot"></div>
        <div class="mode-label">Mode actif</div>
      </div>
      <Badge variant="default" class="mode-badge">
        {executionModes.find(m => m.id === activeMode)?.name || "Standard"}
      </Badge>
    </div>
  </aside>

  <!-- Main Content -->
  <main class="main-content">
    {#if activeTab === 'parametres'}
      <div class="content-section">
        <h2 class="section-title">Gestion de l'enregistrement</h2>

        <div class="setting-group setting-with-margin">
          <span class="setting-label">Démarrer un enregistrement</span>
          {#if isEditingHotkey}
            <div class="hotkey-editor">
              <input
                id="hotkey-input"
                type="text"
                class="hotkey-input"
                placeholder="Appuyez sur les touches..."
                readonly
                value={capturedKeys.length > 0 ? capturedKeys.join(" + ") : ""}
                onkeydown={handleHotkeyCapture}
              />
              <div class="hotkey-editor-buttons">
                <Button onclick={saveHotkey} size="sm">Valider</Button>
                <Button onclick={cancelEditingHotkey} variant="outline" size="sm">Annuler</Button>
              </div>
            </div>
          {:else}
            <div class="hotkey-display-row">
              <div class="hotkey-display">
                {hotkey.split("+").map(k => k.trim().toLowerCase()).join(" + ")}
              </div>
              <button class="modify-button" onclick={startEditingHotkey}>Modifier</button>
            </div>
          {/if}
        </div>

        <div class="setting-group setting-with-margin {pushToTalk ? 'disabled' : ''}">
          <span class="setting-label">Annuler un enregistrement</span>
          {#if isEditingCancelKey}
            <div class="hotkey-editor">
              <input
                id="cancel-key-input"
                type="text"
                class="hotkey-input"
                placeholder="Appuyez sur les touches..."
                readonly
                value={capturedCancelKeys.length > 0 ? capturedCancelKeys.join(" + ") : ""}
                onkeydown={handleCancelKeyCapture}
              />
              <div class="hotkey-editor-buttons">
                <Button onclick={saveCancelKey} size="sm">Valider</Button>
                <Button onclick={cancelEditingCancelKey} variant="outline" size="sm">Annuler</Button>
              </div>
            </div>
          {:else}
            <div class="hotkey-display-row">
              <div class="hotkey-display">
                {cancelKey.split("+").map(k => k.trim().toLowerCase()).join(" + ")}
              </div>
              <button class="modify-button" onclick={startEditingCancelKey} disabled={pushToTalk}>Modifier</button>
            </div>
          {/if}
          {#if pushToTalk}
            <p class="disabled-hint">Non disponible en mode Push-to-Talk</p>
          {/if}
        </div>

        <div class="setting-group setting-with-margin">
          <div class="setting-row">
            <div class="label-with-tooltip">
              <label class="setting-label" for="auto-paste-switch">Collage automatique</label>
              <div class="tooltip-wrapper">
                <span class="tooltip-icon">?</span>
                <div class="tooltip-content">Coller automatiquement le résultat après l'exécution. Si désactivé, ctrl+v pour coller le texte</div>
              </div>
            </div>
            <Switch id="auto-paste-switch" bind:checked={autoPaste} />
          </div>
        </div>

        <div class="setting-group setting-with-margin">
          <div class="setting-row">
            <div class="label-with-tooltip">
              <label class="setting-label" for="push-to-talk-switch">Push To Talk</label>
              <div class="tooltip-wrapper">
                <span class="tooltip-icon">?</span>
                <div class="tooltip-content">En push-to-talk, maintenir le raccourci clavier pour enregistrer. Si désactivé, veuillez réappuyer sur le raccourci clavier pour arrêter l'enregistrement.</div>
              </div>
            </div>
            <Switch id="push-to-talk-switch" bind:checked={pushToTalk} />
          </div>
        </div>

        <h2 class="section-title">Autres options</h2>

        <div class="setting-group setting-with-margin">
          <label class="setting-label" for="language-select">Langue</label>
          <select id="language-select" class="select-input" bind:value={language}>
            {#each languages as lang}
              <option value={lang.value}>{lang.label}</option>
            {/each}
          </select>
        </div>

        <div class="setting-group setting-with-margin">
          <label class="setting-label" for="device-select">Matériel</label>
          <div class="device-selector-row">
            <select id="device-select" class="select-input" bind:value={selectedDevice}>
              <option value={null}>Microphone par défaut</option>
              {#each audioDevices as device}
                <option value={device.name}>
                  {device.name}{device.is_default ? " (par défaut)" : ""}
                </option>
              {/each}
            </select>
            <button class="refresh-icon-button" onclick={refreshAudioDevices} aria-label="Rafraîchir les périphériques">
              <img src="/icons/refresh.svg" alt="Rafraîchir" width="20" height="20" />
            </button>
          </div>
        </div>
      </div>
    {:else if activeTab === 'modeles'}
      <div class="content-section">
        <div class="section-header">
          <h2 class="section-title">IA et Modèles</h2>
          <div class="tabs-container">
            <button
              class="tab-button {modelsSubTab === 'vocaux' ? 'active' : ''}"
              onclick={() => modelsSubTab = 'vocaux'}
            >
              Modèles vocaux
            </button>
            <button
              class="tab-button {modelsSubTab === 'llm' ? 'active' : ''}"
              onclick={() => modelsSubTab = 'llm'}
            >
              Modèles LLM
            </button>
          </div>
        </div>

        {#if modelsSubTab === 'vocaux'}
        <div class="models-list">
          {#each availableModels as model}
            {@const modelLabel = getModelLabel(model.name)}
            {@const modelSize = formatFileSize(model.size_mb)}
            {@const isDownloading = downloadingModel === model.name}
            {@const progress = downloadProgress[model.name] || 0}
            {@const isActive = selectedModel === model.name}
            {@const precisionMap: Record<string, number> = {"Whisper Base": 2, "Whisper Small": 3, "Whisper Medium": 4, "Whisper Large V2": 5, "Whisper Large V3 Turbo": 5}}
            {@const speedMap: Record<string, number> = {"Whisper Base": 5, "Whisper Small": 4, "Whisper Medium": 3, "Whisper Large V2": 2, "Whisper Large V3 Turbo": 3}}
            {@const precision = precisionMap[modelLabel] || 3}
            {@const speed = speedMap[modelLabel] || 3}

            <div class="model-item {isActive ? 'active' : ''}" onclick={(e) => {
              if (!e.target.closest('.model-actions') && model.is_downloaded && !isDownloading) {
                selectedModel = model.name;
              }
            }}>
              <div class="model-info">
                <div class="model-header">
                  <div class="model-name-status">
                    <span class="model-name">{modelLabel}</span>
                    {#if isActive && model.is_downloaded}
                      <Badge variant="default">Actif</Badge>
                    {:else if isDownloading}
                      <Badge variant="secondary">Téléchargement...</Badge>
                    {/if}
                  </div>
                  <span class="model-size">{modelSize}</span>
                </div>

                {#if isDownloading}
                  <div class="progress-bar">
                    <div class="progress-fill" style="width: {progress}%"></div>
                    <span class="progress-text">{Math.round(progress)}%</span>
                  </div>
                {:else}
                  <div class="model-metrics">
                    <div class="metric">
                      <span class="metric-label">Précision</span>
                      <div class="metric-dots">
                        {#each Array(5) as _, i}
                          <span class="dot {i < precision ? 'filled' : ''}"></span>
                        {/each}
                      </div>
                    </div>

                    <div class="metric">
                      <span class="metric-label">Rapidité</span>
                      <div class="metric-dots">
                        {#each Array(5) as _, i}
                          <span class="dot {i < speed ? 'filled' : ''}"></span>
                        {/each}
                      </div>
                    </div>
                  </div>
                {/if}
              </div>

              <div class="model-actions">
                {#if !model.is_downloaded && !isDownloading}
                  <button
                    class="icon-button"
                    aria-label="Télécharger le modèle {modelLabel}"
                    onclick={() => downloadModel(model.name, model.download_url)}
                  >
                    <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
                      <path d="M8 2V12M8 12L5 9M8 12L11 9M2 14H14" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
                    </svg>
                  </button>
                {:else if model.is_downloaded}
                  <button
                    class="icon-button delete"
                    aria-label="Supprimer le modèle {modelLabel}"
                    onclick={() => deleteModel(model.name)}
                    disabled={isActive}
                  >
                    <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
                      <path d="M2 4H14M6 4V2H10V4M3 4V14H13V4" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
                    </svg>
                  </button>
                {/if}
              </div>
            </div>
          {/each}
        </div>

        {#if saveStatus && modelsSubTab === 'vocaux'}
          <p class="save-status {saveStatus.includes('succès') ? 'success' : 'error'}">
            {saveStatus}
          </p>
        {/if}
        {:else if modelsSubTab === 'llm'}
        <!-- LLM Models Content -->

        <!-- Add LLM Header with Toggle Button -->
        <div class="add-mode-header">
          <h3 class="form-subtitle">Ajouter un modèle LLM</h3>
          <button class="toggle-form-button" onclick={() => showAddLlmForm = !showAddLlmForm} aria-label="Afficher/Masquer le formulaire">
            <svg width="20" height="20" viewBox="0 0 20 20" fill="none" style="transform: rotate({showAddLlmForm ? '45deg' : '0deg'}); transition: transform 0.2s;">
              <path d="M10 4V16M4 10H16" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
            </svg>
          </button>
        </div>

        {#if showAddLlmForm}
          <div class="llm-form">
            <div class="form-grid">
              <div class="form-field">
                <label for="llm-service-type">Type de provider</label>
                <select
                  id="llm-service-type"
                  bind:value={newLlmServiceType}
                  onchange={handleServiceTypeChange}
                  class="text-input"
                >
                  <option value="openrouter">OpenRouter</option>
                  <option value="openai">OpenAI</option>
                  <option value="gemini">Google Gemini</option>
                  <option value="lmstudio">LM Studio (Local)</option>
                  <option value="ollama">Ollama (Local)</option>
                </select>
              </div>

              <div class="form-field">
                <label for="llm-name">Nom</label>
                <input
                  id="llm-name"
                  type="text"
                  bind:value={newLlmName}
                  placeholder="Ex: Gemini Flash"
                  class="text-input"
                />
              </div>

              <div class="form-field full-width">
                <label for="llm-api-url">URL de l'API</label>
                <input
                  id="llm-api-url"
                  type="url"
                  bind:value={newLlmApiUrl}
                  placeholder="Ex: https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash-exp:generateContent"
                  class="text-input"
                />
              </div>

              {#if newLlmServiceType === "lmstudio"}
                <div class="form-field full-width">
                  <div style="display: flex; justify-content: space-between; align-items: center;">
                    <label>Modèles LM Studio</label>
                    <button class="icon-button" onclick={detectLMStudioModels} disabled={detectingModels} title="Détecter les modèles">
                      <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
                        <path d="M13 2L3 12M3 2L13 12" stroke="currentColor" stroke-width="2"/>
                      </svg>
                    </button>
                  </div>
                  {#if serviceStatus.lmstudio === false}
                    <p style="color: var(--color-error); font-size: 12px; margin-top: 4px;">LM Studio n'est pas démarré</p>
                  {:else if lmStudioModels.length > 0}
                    <div class="detected-models">
                      {#each lmStudioModels as model}
                        <button
                          class="detected-model-item {newLlmModelName === model.id ? 'selected' : ''}"
                          onclick={() => selectDetectedModel(model)}
                        >
                          <span>{model.id}</span>
                          {#if model.state === "loaded"}
                            <span class="badge-loaded">Chargé</span>
                          {/if}
                        </button>
                      {/each}
                    </div>
                  {/if}
                </div>
              {:else if newLlmServiceType === "ollama"}
                <div class="form-field full-width">
                  <div style="display: flex; justify-content: space-between; align-items: center;">
                    <label>Modèles Ollama</label>
                    <button class="icon-button" onclick={detectOllamaModels} disabled={detectingModels} title="Détecter les modèles">
                      <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
                        <path d="M13 2L3 12M3 2L13 12" stroke="currentColor" stroke-width="2"/>
                      </svg>
                    </button>
                  </div>
                  {#if serviceStatus.ollama === false}
                    <p style="color: var(--color-error); font-size: 12px; margin-top: 4px;">Ollama n'est pas démarré</p>
                  {:else if ollamaModels.length > 0}
                    <div class="detected-models">
                      {#each ollamaModels as model}
                        <button
                          class="detected-model-item {newLlmModelName === model.name ? 'selected' : ''}"
                          onclick={() => selectDetectedModel(model)}
                        >
                          <span>{model.name}</span>
                          <span class="model-size-small">{(model.size / 1000000000).toFixed(1)} GB</span>
                        </button>
                      {/each}
                    </div>
                  {/if}
                </div>
              {/if}

              <div class="form-field">
                <label for="llm-model-name">Nom du modèle</label>
                <input
                  id="llm-model-name"
                  type="text"
                  bind:value={newLlmModelName}
                  placeholder="Ex: gemini-2.0-flash-exp"
                  class="text-input"
                />
              </div>

              {#if !["lmstudio", "ollama"].includes(newLlmServiceType)}
                <div class="form-field full-width">
                  <label for="llm-api-key">Clé API</label>
                  <input
                    id="llm-api-key"
                    type="password"
                    bind:value={newLlmApiKey}
                    placeholder="Votre clé API"
                    class="text-input"
                  />
                </div>
              {/if}
            </div>
            <button class="modify-button" onclick={addLlmModel}>Ajouter</button>
          </div>
        {/if}

        <!-- LLM Models List -->
        <div class="llm-list">
          <h3 class="form-subtitle">Modèles configurés</h3>
          {#if llmModels.length === 0}
            <p class="empty-state">Aucun modèle LLM configuré</p>
          {:else}
            {#each llmModels as model}
              {#if editingLlm && editingLlm.id === model.id}
                <div class="llm-item editing">
                  <div class="form-grid">
                    <div class="form-field">
                      <label>Nom</label>
                      <input type="text" bind:value={editingLlm.name} class="text-input" />
                    </div>
                    <div class="form-field">
                      <label>Nom du modèle</label>
                      <input type="text" bind:value={editingLlm.model_name} class="text-input" />
                    </div>
                    <div class="form-field full-width">
                      <label>URL de l'API</label>
                      <input type="url" bind:value={editingLlm.api_url} class="text-input" />
                    </div>
                    <div class="form-field full-width">
                      <label>Nouvelle clé API (laisser vide pour conserver)</label>
                      <input type="password" bind:value={newLlmApiKey} class="text-input" placeholder="Nouvelle clé API (optionnel)" />
                    </div>
                  </div>
                  <div class="button-row">
                    <Button onclick={updateLlmModel} size="sm">Enregistrer</Button>
                    <Button onclick={cancelEditingLlm} variant="outline" size="sm">Annuler</Button>
                  </div>
                </div>
              {:else}
                <div class="llm-item">
                  <div class="llm-info">
                    <div class="llm-name">{model.name}</div>
                    <div class="llm-details">
                      <span class="detail-label">Modèle:</span> {model.model_name}
                    </div>
                  </div>
                  <div class="llm-actions">
                    <button class="icon-button" onclick={() => startEditingLlm(model)} aria-label="Modifier">
                      <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M12 2L14 4L6 12H4V10L12 2Z"/>
                      </svg>
                    </button>
                    <button class="icon-button delete" onclick={() => deleteLlmModel(model.id)} aria-label="Supprimer">
                      <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
                        <path d="M2 4H14M6 4V2H10V4M3 4V14H13V4" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
                      </svg>
                    </button>
                  </div>
                </div>
              {/if}
            {/each}
          {/if}
        </div>

        {#if saveStatus && modelsSubTab === 'llm'}
          <p class="save-status {saveStatus.includes('succès') ? 'success' : 'error'}">
            {saveStatus}
          </p>
        {/if}
        {/if}
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
          <button class="modify-button" onclick={addCustomWord}>Ajouter</button>
          <button class="secondary-button" onclick={clearAllWords}>Tout effacer</button>
        </div>

        <div class="custom-words-list">
          {#each customWords as word}
            <div class="word-tag">
              <span>{word}</span>
              <button class="remove-word" onclick={() => removeCustomWord(word)} aria-label="Supprimer {word}">
                <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
                  <path d="M2 2L10 10M2 10L10 2" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
                </svg>
              </button>
            </div>
          {/each}
        </div>
      </div>
    {:else if activeTab === 'modes'}
      <div class="content-section">
        <h2 class="section-title">Modes d'exécution</h2>

        <!-- Add Mode Header with Toggle Button -->
        <div class="add-mode-header">
          <h3 class="form-subtitle">Ajouter un mode</h3>
          <button class="toggle-form-button" onclick={() => showAddModeForm = !showAddModeForm} aria-label="Afficher/Masquer le formulaire">
            <svg width="20" height="20" viewBox="0 0 20 20" fill="none" style="transform: rotate({showAddModeForm ? '45deg' : '0deg'}); transition: transform 0.2s;">
              <path d="M10 4V16M4 10H16" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
            </svg>
          </button>
        </div>

        <!-- Add Mode Form -->
        {#if showAddModeForm}
          <div class="mode-form">
            <div class="form-grid">
              <div class="form-field">
                <label for="mode-name">Nom du mode</label>
                <input
                  id="mode-name"
                  type="text"
                  bind:value={newModeName}
                  placeholder="Ex: Correction orthographique"
                  class="text-input"
                />
              </div>
              <div class="form-field">
                <label for="mode-llm">Modèle LLM</label>
                <select id="mode-llm" bind:value={newModeLlmId} class="select-input">
                  <option value={null}>Aucun (mode standard)</option>
                  {#each llmModels as model}
                    <option value={model.id}>{model.name}</option>
                  {/each}
                </select>
              </div>
              <div class="form-field full-width">
                <label for="mode-prompt">Prompt système</label>
                <textarea
                  id="mode-prompt"
                  bind:value={newModeSystemPrompt}
                  placeholder="Ex: Corrige l'orthographe et la grammaire du texte suivant."
                  class="textarea-input"
                  rows="4"
                ></textarea>
              </div>
            </div>
            <button class="modify-button" onclick={addExecutionMode}>Ajouter</button>
          </div>
        {/if}

        <!-- Modes List -->
        <div class="modes-list">
          <h3 class="form-subtitle">Modes configurés</h3>
          {#each executionModes as mode}
            {#if editingMode && editingMode.id === mode.id}
              <div class="mode-item editing">
                <div class="form-grid">
                  <div class="form-field">
                    <label>Nom du mode</label>
                    <input type="text" bind:value={editingMode.name} class="text-input" disabled={mode.id === 'standard'} />
                  </div>
                  <div class="form-field">
                    <label>Modèle LLM</label>
                    <select bind:value={editingMode.llm_model_id} class="select-input" disabled={mode.id === 'standard'}>
                      <option value={null}>Aucun (mode standard)</option>
                      {#each llmModels as llmModel}
                        <option value={llmModel.id}>{llmModel.name}</option>
                      {/each}
                    </select>
                  </div>
                  <div class="form-field full-width">
                    <label>Prompt système</label>
                    <textarea bind:value={editingMode.system_prompt} class="textarea-input" rows="4" disabled={mode.id === 'standard'}></textarea>
                  </div>
                </div>
                <div class="button-row">
                  <Button onclick={updateExecutionMode} size="sm" disabled={mode.id === 'standard'}>Enregistrer</Button>
                  <Button onclick={cancelEditingMode} variant="outline" size="sm">Annuler</Button>
                </div>
              </div>
            {:else}
              <div class="mode-item {mode.id === activeMode ? 'active' : ''}" onclick={(e) => {
                if (!e.target.closest('.mode-actions')) {
                  setActiveModeHandler(mode.id);
                }
              }}>
                <div class="mode-info">
                  <div class="mode-header">
                    <div class="mode-name">{mode.name}</div>
                    {#if mode.id === activeMode}
                      <Badge variant="default">Actif</Badge>
                    {/if}
                  </div>
                  <div class="mode-details">
                    <span class="detail-label">Modèle LLM:</span> {getLlmModelName(mode.llm_model_id)}
                  </div>
                </div>
                <div class="mode-actions">
                  {#if mode.id !== 'standard'}
                    <button class="icon-button" onclick={() => startEditingMode(mode)} aria-label="Modifier">
                      <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M12 2L14 4L6 12H4V10L12 2Z"/>
                      </svg>
                    </button>
                    <button class="icon-button delete" onclick={() => deleteExecutionMode(mode.id)} aria-label="Supprimer">
                      <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
                        <path d="M2 4H14M6 4V2H10V4M3 4V14H13V4" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
                      </svg>
                    </button>
                  {/if}
                </div>
              </div>
            {/if}
          {/each}
        </div>

        {#if saveStatus}
          <p class="save-status {saveStatus.includes('succès') ? 'success' : 'error'}">
            {saveStatus}
          </p>
        {/if}
      </div>
    {:else if activeTab === 'about'}
      <div class="content-section">
        <h2 class="section-title">À propos</h2>

        <div class="about-content">
          <div class="app-header">
            <h3 class="app-name">Flemme</h3>
            <p class="app-description">
              Application de transcription vocale avec traitement IA
            </p>
          </div>

          <UpdateChecker />
        </div>
      </div>
    {/if}
  </main>

  {#if showWelcomeModal}
    <div class="modal-overlay">
      <div class="modal-content">
        <div class="modal-header">
          <h2 class="modal-title">Bienvenue sur Flemme !</h2>
          <p class="modal-subtitle">Pour commencer, veuillez télécharger un modèle de reconnaissance vocale.</p>
        </div>
        
        <div class="modal-body">
          <div class="models-list compact">
            {#each availableModels as model}
              {@const modelLabel = getModelLabel(model.name)}
              {@const modelSize = formatFileSize(model.size_mb)}
              {@const isDownloading = downloadingModel === model.name}
              {@const progress = downloadProgress[model.name] || 0}
              {@const isActive = selectedModel === model.name}
              
              <div class="model-item {isActive ? 'active' : ''}">
                <div class="model-info">
                  <div class="model-header">
                    <div class="model-name-status">
                      <span class="model-name">{modelLabel}</span>
                      {#if model.is_downloaded}
                        <Badge variant="default">Installé</Badge>
                      {:else if isDownloading}
                        <Badge variant="secondary">Téléchargement...</Badge>
                      {/if}
                    </div>
                    <span class="model-size">{modelSize}</span>
                  </div>

                  {#if isDownloading}
                    <div class="progress-bar">
                      <div class="progress-fill" style="width: {progress}%"></div>
                      <span class="progress-text">{Math.round(progress)}%</span>
                    </div>
                  {/if}
                </div>

                <div class="model-actions">
                  {#if !model.is_downloaded && !isDownloading}
                    <button 
                      class="icon-button" 
                      aria-label="Télécharger le modèle {modelLabel}"
                      onclick={() => downloadModel(model.name, model.download_url)}
                    >
                      <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
                        <path d="M8 2V12M8 12L5 9M8 12L11 9M2 14H14" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
                      </svg>
                    </button>
                  {:else if model.is_downloaded}
                    <div class="check-icon">
                      <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
                        <path d="M16.6666 5L7.49992 14.1667L3.33325 10" stroke="#4FB094" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
                      </svg>
                    </div>
                  {/if}
                </div>
              </div>
            {/each}
          </div>
        </div>

        <div class="modal-footer">
          {#if availableModels.some(m => m.is_downloaded)}
            <Button onclick={() => showWelcomeModal = false} class="w-full">Commencer à utiliser Flemme</Button>
          {:else}
            <p class="modal-hint">Veuillez télécharger au moins un modèle pour continuer.</p>
          {/if}
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  :global(body) {
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
    margin: 0;
    padding: 0;
  }

  /* Switch colors and size for dark theme */
  :global([data-slot="switch"][data-state="checked"]) {
    background-color: #4FB094 !important;
  }

  :global([data-slot="switch"][data-state="unchecked"]) {
    background-color: #969191 !important;
  }

  :global([data-slot="switch"]) {
    height: 1.5rem !important;
    width: 2.75rem !important;
  }

  :global([data-slot="switch-thumb"]) {
    width: 1.25rem !important;
    height: 1.25rem !important;
  }

  :global([data-slot="switch-thumb"][data-state="checked"]) {
    transform: translateX(0.25rem) !important;
  }

  .app-container {
    display: flex;
    height: 100vh;
    background: #000000;
  }

  /* Sidebar */
  .sidebar {
    width: 240px;
    background: #202020;
    padding: 32px 0;
    display: flex;
    flex-direction: column;
  }

  .logo {
    padding: 0 24px;
  }

  .logo-image {
    width: 100%;
    height: auto;
    margin-bottom: 48px;
    max-width: 192px;
    display: block;
  }

  .nav-menu {
    display: flex;
    flex-direction: column;
    gap: 0;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 24px;
    background: transparent;
    border: none;
    border-radius: 0;
    cursor: pointer;
    font-size: 20px;
    font-weight: 500;
    color: #BDBDBD;
    transition: all 0.2s;
    width: 100%;
    text-align: left;
  }

  .nav-item:hover {
    background: rgba(255, 255, 255, 0.05);
  }

  .nav-item.active {
    background: #000000;
    color: #BDBDBD;
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
    color: #BDBDBD;
  }

  .setting-group {
    margin-bottom: 24px;
  }

  .setting-with-margin {
    margin-left: 24px;
    margin-right: 24px;
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
    color: #BDBDBD;
  }

  .setting-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .label-with-tooltip {
    display: flex;
    gap: 8px;
  }

  .tooltip-wrapper {
    position: relative;
    display: inline-flex;
  }

  .tooltip-icon {
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: #4FB094;
    color: #FFFFFF;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 10px;
    font-weight: 600;
    cursor: help;
    vertical-align: super;
  }

  .tooltip-content {
    position: absolute;
    left: 50%;
    bottom: calc(100% + 8px);
    transform: translateX(-50%);
    background: #4FB094;
    color: #FFFFFF;
    padding: 8px 12px;
    border-radius: 6px;
    font-size: 12px;
    font-weight: 400;
    line-height: 1.4;
    white-space: normal;
    width: 250px;
    opacity: 0;
    pointer-events: none;
    transition: opacity 0.2s;
    z-index: 1000;
  }

  .tooltip-content::after {
    content: '';
    position: absolute;
    top: 100%;
    left: 50%;
    transform: translateX(-50%);
    border: 6px solid transparent;
    border-top-color: #4FB094;
  }

  .tooltip-wrapper:hover .tooltip-content {
    opacity: 1;
  }

  .hotkey-display {
    padding: 8px 16px;
    background: #202020;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 500;
    color: #BDBDBD;
    display: inline-block;
  }

  .hotkey-display-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    margin-top: 8px;
  }

  .modify-button {
    width: 100px;
    padding: 8px 16px;
    background: #4FB094;
    border: none;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 500;
    color: #FFFFFF;
    cursor: pointer;
    transition: all 0.2s;
  }

  .modify-button:hover:not(:disabled) {
    background: #3D8A76;
  }

  .modify-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .secondary-button {
    padding: 8px 16px;
    background: #202020;
    border: 1px solid #4FB094;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 500;
    color: #BDBDBD;
    cursor: pointer;
    transition: all 0.2s;
  }

  .secondary-button:hover:not(:disabled) {
    background: #2A2A2A;
    border-color: #3D8A76;
  }

  .secondary-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
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
    background: #202020;
    color: #BDBDBD;
    text-align: center;
  }

  .hotkey-input:focus {
    outline: none;
    border-color: #4FB094;
  }

  .hotkey-editor-buttons {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
  }

  .select-input {
    width: 100%;
    padding: 10px 12px;
    border: 1px solid #333333;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 300;
    background: #202020;
    color: #BDBDBD;
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

  .refresh-icon-button {
    padding: 8px;
    background: transparent;
    border: 1px solid #333333;
    border-radius: 6px;
    cursor: pointer;
    color: #4FB094;
    transition: all 0.2s;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .refresh-icon-button:hover {
    background: #202020;
    border-color: #4FB094;
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
    border: 1px solid #333333;
    border-radius: 8px;
    background: #000000;
    cursor: pointer;
    transition: background 0.2s, border-color 0.2s;
  }

  .model-item:hover:not(.active) {
    background: #0A0A0A;
    border-color: #4FB094;
  }

  .model-item.active {
    border-color: #4FB094;
    background: #1A2F2A;
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

  .model-name-status {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .model-name {
    font-size: 16px;
    font-weight: 500;
    color: #BDBDBD;
  }

  .model-size {
    font-size: 14px;
    font-weight: 300;
    color: #808080;
  }

  .progress-bar {
    position: relative;
    width: 100%;
    height: 24px;
    background: #202020;
    border-radius: 4px;
    overflow: hidden;
  }

  .progress-fill {
    position: absolute;
    top: 0;
    left: 0;
    height: 100%;
    background: linear-gradient(90deg, #4FB094 0%, #3A8B75 100%);
    transition: width 0.3s ease;
  }

  .progress-text {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    font-size: 12px;
    font-weight: 500;
    color: #BDBDBD;
    z-index: 1;
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
    color: #808080;
  }

  .metric-dots {
    display: flex;
    gap: 4px;
  }

  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #333333;
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
    border: 1px solid #333333;
    border-radius: 6px;
    cursor: pointer;
    color: #4FB094;
    transition: all 0.2s;
  }

  .icon-button:hover {
    background: #202020;
    border-color: #4FB094;
  }

  .icon-button.delete {
    color: #EF4444;
    border-color: #EF4444;
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
    border: 1px solid #333333;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 300;
    background: #202020;
    color: #BDBDBD;
  }

  .vocab-input::placeholder {
    color: #666666;
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
    background: #202020;
    border: 1px solid #333333;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 300;
    color: #BDBDBD;
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

  /* Active Mode Indicator */
  .active-mode-indicator {
    margin-top: auto;
    padding: 16px 24px;
    border-top: 1px solid #333333;
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  .mode-status-row {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 8px;
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #4FB094;
  }

  .mode-label {
    font-size: 14px;
    font-weight: 300;
    color: #808080;
  }

  .mode-badge {
    display: inline-block;
    font-size: 14px;
  }

  /* LLM and Modes Tabs */
  .form-subtitle {
    font-size: 18px;
    font-weight: 500;
    color: #BDBDBD;
    margin: 32px 0 16px 0;
  }

  .add-mode-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin: 0 0 16px 0;
  }

  .add-mode-header .form-subtitle {
    margin: 0;
  }

  .toggle-form-button {
    width: 32px;
    height: 32px;
    padding: 0;
    background: transparent;
    border: 1px solid #4FB094;
    border-radius: 6px;
    cursor: pointer;
    color: #4FB094;
    transition: all 0.2s;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .toggle-form-button:hover {
    background: #4FB094;
    color: #FFFFFF;
  }

  .form-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
    margin-bottom: 16px;
  }

  .form-field {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .form-field.full-width {
    grid-column: 1 / -1;
  }

  .form-field label {
    font-size: 14px;
    font-weight: 300;
    color: #BDBDBD;
  }

  .text-input {
    width: 100%;
    padding: 10px 12px;
    border: 1px solid #333333;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 300;
    background: #202020;
    color: #BDBDBD;
  }

  .text-input:focus {
    outline: none;
    border-color: #4FB094;
  }

  .textarea-input {
    width: 100%;
    padding: 10px 12px;
    border: 1px solid #333333;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 300;
    background: #202020;
    color: #BDBDBD;
    font-family: inherit;
    resize: vertical;
  }

  .textarea-input:focus {
    outline: none;
    border-color: #4FB094;
  }

  .button-row {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
  }

  .empty-state {
    padding: 32px;
    text-align: center;
    color: #999;
    font-size: 14px;
  }

  /* LLM Items */
  .llm-list,
  .modes-list {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .llm-item,
  .mode-item {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 16px;
    border: 1px solid #333333;
    border-radius: 8px;
    background: #000000;
  }

  .mode-item:not(.editing) {
    cursor: pointer;
    transition: background 0.2s, border-color 0.2s;
  }

  .mode-item:not(.editing):hover:not(.active) {
    background: #0A0A0A;
    border-color: #4FB094;
  }

  .llm-item.editing,
  .mode-item.editing {
    flex-direction: column;
    align-items: stretch;
  }

  .mode-item.active {
    border-color: #4FB094;
    background: #1A2F2A;
  }

  .llm-info,
  .mode-info {
    flex: 1;
  }

  .llm-name,
  .mode-name {
    font-size: 16px;
    font-weight: 500;
    color: #BDBDBD;
    margin-bottom: 8px;
  }

  .mode-header {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 8px;
  }

  .llm-details,
  .mode-details {
    font-size: 14px;
    font-weight: 300;
    color: #808080;
    margin-bottom: 4px;
  }

  .detail-label {
    font-weight: 500;
    color: #BDBDBD;
  }

  .mode-prompt {
    margin-top: 8px;
    padding: 12px;
    background: #202020;
    border-radius: 6px;
    font-size: 13px;
    font-weight: 300;
    color: #808080;
    line-height: 1.5;
  }

  .llm-actions,
  .mode-actions {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .llm-form,
  .mode-form {
    padding: 24px;
    background: #202020;
    border-radius: 8px;
    margin-bottom: 32px;
  }

  /* Section Header with Tabs */
  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 32px;
  }

  .section-header .section-title {
    margin: 0;
  }

  .tabs-container {
    display: flex;
    gap: 4px;
    background: #202020;
    padding: 4px;
    border-radius: 8px;
  }

  .tab-button {
    padding: 8px 16px;
    background: transparent;
    border: none;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 500;
    color: #808080;
    cursor: pointer;
    transition: all 0.2s;
  }

  .tab-button:hover {
    background: rgba(255, 255, 255, 0.05);
  }

  .tab-button.active {
    background: #333333;
    color: #4FB094;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
  }

  /* Local LLM Detection Styles */
  .detected-models {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-top: 8px;
    max-height: 200px;
    overflow-y: auto;
  }

  .detected-model-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px 14px;
    background: #202020;
    border: 1px solid #333333;
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.2s;
    text-align: left;
    width: 100%;
    font-size: 13px;
    color: #BDBDBD;
  }

  .detected-model-item:hover {
    background: #252525;
    border-color: #4FB094;
  }

  .detected-model-item.selected {
    background: #1a3d32;
    border-color: #4FB094;
    color: #4FB094;
  }

  .badge-loaded {
    background: #4FB094;
    color: #141414;
    padding: 2px 8px;
    border-radius: 4px;
    font-size: 11px;
    font-weight: 600;
  }

  .model-size-small {
    font-size: 11px;
    color: #808080;
  }

  /* About Section */
  .about-content {
    display: flex;
    flex-direction: column;
    gap: 32px;
  }

  .app-header {
    text-align: center;
    padding: 24px;
    background: linear-gradient(135deg, rgba(79, 176, 148, 0.1), rgba(79, 176, 148, 0.05));
    border: 1px solid rgba(79, 176, 148, 0.2);
    border-radius: 12px;
  }

  .app-name {
    font-size: 32px;
    font-weight: 700;
    color: #4FB094;
    margin: 0 0 8px 0;
  }

  .app-description {
    font-size: 14px;
    color: #8E8E93;
    margin: 0;
  }
  /* Modal Styles */
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background: rgba(0, 0, 0, 0.8);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 1000;
    backdrop-filter: blur(4px);
  }

  .modal-content {
    background: #202020;
    border-radius: 12px;
    width: 90%;
    max-width: 600px;
    max-height: 90vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04);
    border: 1px solid #333;
  }

  .modal-header {
    padding: 24px;
    border-bottom: 1px solid #333;
    text-align: center;
  }

  .modal-title {
    font-size: 24px;
    font-weight: 600;
    color: #fff;
    margin: 0 0 8px 0;
  }

  .modal-subtitle {
    color: #888;
    font-size: 14px;
    margin: 0;
  }

  .modal-body {
    padding: 24px;
    overflow-y: auto;
  }

  .modal-footer {
    padding: 24px;
    border-top: 1px solid #333;
    display: flex;
    justify-content: center;
  }

  .modal-hint {
    color: #888;
    font-size: 12px;
    font-style: italic;
  }

  .models-list.compact {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .models-list.compact .model-item {
    padding: 12px;
  }

  .check-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
  }
</style>
