// HotkeyListener - manages global keyboard shortcuts
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};

pub struct HotkeyListener;

impl HotkeyListener {
    /// Create a new hotkey listener
    pub fn new() -> Result<Self, String> {
        Ok(Self)
    }

    /// Get the hardcoded Ctrl+Alt shortcut for the POC
    /// This shortcut triggers: Press → Start Recording, Release → Stop Recording → Transcribe → Auto-paste
    pub fn get_record_shortcut() -> Shortcut {
        // Ctrl+Alt combination
        // Using a key that's easy to press - we'll use 'R' for Record
        Shortcut::new(
            Some(Modifiers::CONTROL | Modifiers::ALT),
            Code::KeyR
        )
    }
}
