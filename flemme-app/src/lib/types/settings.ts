/**
 * Application settings types
 * These types match the Rust AppSettings struct in src-tauri/src/config/settings.rs
 */

export interface AppSettings {
  hotkey: string;
  language: string;
  auto_paste: boolean;
  model_name: string;
}

export const DEFAULT_SETTINGS: AppSettings = {
  hotkey: "Ctrl+Alt+R",
  language: "fr",
  auto_paste: true,
  model_name: "ggml-small.bin"
};

export interface LanguageOption {
  value: string;
  label: string;
}

export interface ModelOption {
  value: string;
  label: string;
}
