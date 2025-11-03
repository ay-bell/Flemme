// AppSettings - application configuration and settings

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub hotkey: String,
    pub language: String,
    pub auto_paste: bool,
    pub model_name: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            hotkey: String::from("Ctrl+Alt+R"),
            language: String::from("fr"),
            auto_paste: true,
            model_name: String::from("ggml-small.bin"),
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

        let settings: AppSettings = serde_json::from_str(&contents)
            .map_err(|e| format!("Failed to parse settings: {}", e))?;

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
