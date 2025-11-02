// ClipboardManager - manages clipboard operations
use arboard::Clipboard;
use enigo::{Enigo, Key, Keyboard, Settings};

pub struct ClipboardManager;

impl ClipboardManager {
    /// Create a new clipboard manager
    pub fn new() -> Result<Self, String> {
        Ok(Self)
    }

    /// Copy text to clipboard
    pub fn copy_text(&self, text: &str) -> Result<(), String> {
        let mut clipboard = Clipboard::new()
            .map_err(|e| format!("Failed to access clipboard: {}", e))?;

        clipboard
            .set_text(text)
            .map_err(|e| format!("Failed to copy text: {}", e))?;

        Ok(())
    }

    /// Get text from clipboard
    pub fn paste_text(&self) -> Result<String, String> {
        let mut clipboard = Clipboard::new()
            .map_err(|e| format!("Failed to access clipboard: {}", e))?;

        clipboard
            .get_text()
            .map_err(|e| format!("Failed to get clipboard text: {}", e))
    }

    /// Auto-paste: Copy text to clipboard and simulate Ctrl+V
    pub fn auto_paste(&self, text: &str) -> Result<(), String> {
        // First, copy the text to clipboard
        self.copy_text(text)?;

        // Small delay to ensure clipboard is updated
        std::thread::sleep(std::time::Duration::from_millis(50));

        // Simulate Ctrl+V to paste
        let mut enigo = Enigo::new(&Settings::default())
            .map_err(|e| format!("Failed to create enigo: {:?}", e))?;

        enigo.key(Key::Control, enigo::Direction::Press)
            .map_err(|e| format!("Failed to press Ctrl: {:?}", e))?;
        enigo.key(Key::Unicode('v'), enigo::Direction::Click)
            .map_err(|e| format!("Failed to click V: {:?}", e))?;
        enigo.key(Key::Control, enigo::Direction::Release)
            .map_err(|e| format!("Failed to release Ctrl: {:?}", e))?;

        Ok(())
    }
}