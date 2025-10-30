// AppSettings - application configuration and settings
// TODO: Implement settings management

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub hotkey: String,
    pub model_name: String,
    pub auto_paste: bool,
    // TODO: Add more settings fields
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            hotkey: String::from("Ctrl+Shift+Space"),
            model_name: String::from("base"),
            auto_paste: true,
        }
    }
}

impl AppSettings {
    pub fn load() -> Result<Self, String> {
        todo!("Implement settings loading")
    }

    pub fn save(&self) -> Result<(), String> {
        todo!("Implement settings saving")
    }
}
