// AppSettings - application configuration and settings

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Configuration for an LLM model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmModel {
    pub id: String,
    pub name: String,
    pub api_url: String,
    pub model_name: String,
}

/// Configuration for an execution mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMode {
    pub id: String,
    pub name: String,
    pub llm_model_id: Option<String>, // None for "Standard" mode
    pub system_prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub hotkey: String,
    pub language: String,
    pub auto_paste: bool,
    pub model_name: String,
    pub push_to_talk: bool,
    pub cancel_key: String,
    pub device_name: Option<String>,
    #[serde(default)]
    pub custom_words: Vec<String>,
    #[serde(default)]
    pub llm_models: Vec<LlmModel>,
    #[serde(default)]
    pub execution_modes: Vec<ExecutionMode>,
    #[serde(default = "default_active_mode")]
    pub active_mode: String,
}

fn default_active_mode() -> String {
    String::from("standard")
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            hotkey: String::from("Ctrl+Shift+R"),
            language: String::from("fr"),
            auto_paste: true,
            model_name: String::from("ggml-small-q5_1.bin"),
            push_to_talk: false, // Default to toggle mode
            cancel_key: String::from("Escape"),
            device_name: None, // None means use default device
            custom_words: vec![
                String::from("Aymeric Bellavoine"),
                String::from("PPAT"),
                String::from("Harmonie Mutuelle"),
            ],
            llm_models: vec![],
            execution_modes: vec![ExecutionMode {
                id: String::from("standard"),
                name: String::from("Standard"),
                llm_model_id: None,
                system_prompt: String::new(),
            }],
            active_mode: String::from("standard"),
        }
    }
}

impl AppSettings {
    /// Get the path to the settings file
    fn settings_path() -> Result<PathBuf, String> {
        let mut path = dirs::data_dir()
            .ok_or_else(|| "Failed to get data directory".to_string())?;
        path.push("Flemme");

        // Create directory if it doesn't exist
        if !path.exists() {
            fs::create_dir_all(&path)
                .map_err(|e| format!("Failed to create settings directory: {}", e))?;
        }

        path.push("settings.json");
        Ok(path)
    }

    /// Load settings from disk, or return default if file doesn't exist
    pub fn load() -> Result<Self, String> {
        let path = Self::settings_path()?;

        if !path.exists() {
            println!("Settings file not found, using defaults");
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read settings file: {}", e))?;

        let mut settings: AppSettings = serde_json::from_str(&contents)
            .map_err(|e| format!("Failed to parse settings: {}", e))?;

        // If custom_words is empty and this is a fresh migration, initialize with defaults
        if settings.custom_words.is_empty() {
            let defaults = Self::default();
            settings.custom_words = defaults.custom_words;
            println!("Initialized custom_words with defaults");
            // Save the updated settings
            let _ = settings.save();
        }

        // Ensure "standard" mode exists
        if !settings.execution_modes.iter().any(|m| m.id == "standard") {
            settings.execution_modes.insert(0, ExecutionMode {
                id: String::from("standard"),
                name: String::from("Standard"),
                llm_model_id: None,
                system_prompt: String::new(),
            });
            println!("Initialized standard execution mode");
            let _ = settings.save();
        }

        println!("Settings loaded from: {:?}", path);
        Ok(settings)
    }

    /// Save settings to disk
    pub fn save(&self) -> Result<(), String> {
        let path = Self::settings_path()?;

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;

        fs::write(&path, json)
            .map_err(|e| format!("Failed to write settings file: {}", e))?;

        println!("Settings saved to: {:?}", path);
        Ok(())
    }
}
