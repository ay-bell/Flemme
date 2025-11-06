// HotkeyListener - manages global keyboard shortcuts
use tauri_plugin_global_shortcut::Shortcut;
use std::str::FromStr;
use crate::config;

pub struct HotkeyListener;

impl HotkeyListener {
    /// Create a new hotkey listener
    pub fn new() -> Result<Self, String> {
        Ok(Self)
    }

    /// Get the record shortcut from settings
    /// This shortcut triggers: Press → Start Recording, Release → Stop Recording → Transcribe → Auto-paste
    pub fn get_record_shortcut() -> Shortcut {
        // Load settings and get the hotkey
        let settings = config::AppSettings::load().unwrap_or_default();

        // Try to parse the hotkey from settings, fallback to default if parsing fails
        Shortcut::from_str(&settings.hotkey)
            .unwrap_or_else(|_| {
                eprintln!("Failed to parse hotkey '{}', using default Ctrl+Alt+R", settings.hotkey);
                Shortcut::from_str("Ctrl+Alt+R").unwrap()
            })
    }
}
