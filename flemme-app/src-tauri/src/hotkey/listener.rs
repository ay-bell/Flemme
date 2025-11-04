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

    /// Parse a hotkey string (e.g., "Ctrl+Alt+R") into a Shortcut
    /// Format: [Modifier+]...[Modifier+]Key
    /// Supported modifiers: Ctrl, Alt, Shift, Super/Meta/Cmd
    pub fn parse_hotkey_string(hotkey: &str) -> Result<Shortcut, String> {
        let parts: Vec<&str> = hotkey.split('+').map(|s| s.trim()).collect();

        if parts.is_empty() {
            return Err("Hotkey string is empty".to_string());
        }

        let mut modifiers = Modifiers::empty();
        let mut key_str = None;

        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                // Last part is the key
                key_str = Some(*part);
            } else {
                // Parse modifiers
                match part.to_lowercase().as_str() {
                    "ctrl" | "control" => modifiers |= Modifiers::CONTROL,
                    "alt" | "option" => modifiers |= Modifiers::ALT,
                    "shift" => modifiers |= Modifiers::SHIFT,
                    "super" | "meta" | "cmd" | "command" => modifiers |= Modifiers::SUPER,
                    _ => return Err(format!("Unknown modifier: {}", part)),
                }
            }
        }

        let key_str = key_str.ok_or("No key specified in hotkey")?;
        let code = Self::parse_key_code(key_str)?;

        let modifier_opt = if modifiers.is_empty() {
            None
        } else {
            Some(modifiers)
        };

        Ok(Shortcut::new(modifier_opt, code))
    }

    /// Parse a key string into a Code enum
    fn parse_key_code(key: &str) -> Result<Code, String> {
        match key.to_uppercase().as_str() {
            // Letters
            "A" => Ok(Code::KeyA),
            "B" => Ok(Code::KeyB),
            "C" => Ok(Code::KeyC),
            "D" => Ok(Code::KeyD),
            "E" => Ok(Code::KeyE),
            "F" => Ok(Code::KeyF),
            "G" => Ok(Code::KeyG),
            "H" => Ok(Code::KeyH),
            "I" => Ok(Code::KeyI),
            "J" => Ok(Code::KeyJ),
            "K" => Ok(Code::KeyK),
            "L" => Ok(Code::KeyL),
            "M" => Ok(Code::KeyM),
            "N" => Ok(Code::KeyN),
            "O" => Ok(Code::KeyO),
            "P" => Ok(Code::KeyP),
            "Q" => Ok(Code::KeyQ),
            "R" => Ok(Code::KeyR),
            "S" => Ok(Code::KeyS),
            "T" => Ok(Code::KeyT),
            "U" => Ok(Code::KeyU),
            "V" => Ok(Code::KeyV),
            "W" => Ok(Code::KeyW),
            "X" => Ok(Code::KeyX),
            "Y" => Ok(Code::KeyY),
            "Z" => Ok(Code::KeyZ),

            // Numbers
            "0" => Ok(Code::Digit0),
            "1" => Ok(Code::Digit1),
            "2" => Ok(Code::Digit2),
            "3" => Ok(Code::Digit3),
            "4" => Ok(Code::Digit4),
            "5" => Ok(Code::Digit5),
            "6" => Ok(Code::Digit6),
            "7" => Ok(Code::Digit7),
            "8" => Ok(Code::Digit8),
            "9" => Ok(Code::Digit9),

            // Function keys
            "F1" => Ok(Code::F1),
            "F2" => Ok(Code::F2),
            "F3" => Ok(Code::F3),
            "F4" => Ok(Code::F4),
            "F5" => Ok(Code::F5),
            "F6" => Ok(Code::F6),
            "F7" => Ok(Code::F7),
            "F8" => Ok(Code::F8),
            "F9" => Ok(Code::F9),
            "F10" => Ok(Code::F10),
            "F11" => Ok(Code::F11),
            "F12" => Ok(Code::F12),

            // Special keys
            "SPACE" => Ok(Code::Space),
            "ENTER" | "RETURN" => Ok(Code::Enter),
            "TAB" => Ok(Code::Tab),
            "ESCAPE" | "ESC" => Ok(Code::Escape),
            "BACKSPACE" => Ok(Code::Backspace),
            "DELETE" | "DEL" => Ok(Code::Delete),
            "INSERT" | "INS" => Ok(Code::Insert),
            "HOME" => Ok(Code::Home),
            "END" => Ok(Code::End),
            "PAGEUP" | "PGUP" => Ok(Code::PageUp),
            "PAGEDOWN" | "PGDN" => Ok(Code::PageDown),
            "ARROWUP" | "UP" => Ok(Code::ArrowUp),
            "ARROWDOWN" | "DOWN" => Ok(Code::ArrowDown),
            "ARROWLEFT" | "LEFT" => Ok(Code::ArrowLeft),
            "ARROWRIGHT" | "RIGHT" => Ok(Code::ArrowRight),

            _ => Err(format!("Unknown key: {}", key)),
        }
    }
}
