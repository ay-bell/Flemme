// Module declarations
pub mod audio;
pub mod transcription;
pub mod hotkey;
pub mod clipboard;
pub mod config;
pub mod llm;

use audio::{AudioRecorder, VoiceActivityDetector};
use transcription::engine::TranscriptionEngine;
use clipboard::ClipboardManager;
use hotkey::HotkeyListener;
use config::settings::{LlmModel, ExecutionMode};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use tauri::{AppHandle, Emitter, State};

// Audio command messages
pub enum AudioCommand {
    StartRecording,
    StopRecording { reply: Sender<Result<Vec<f32>, String>> },
    IsRecording { reply: Sender<bool> },
    Shutdown,
}

// Transcription command messages
pub enum TranscriptionCommand {
    Transcribe {
        audio: Vec<f32>,
        language: Option<String>,
        reply: Sender<Result<String, String>>,
    },
    ReloadModel {
        model_path: String,
        reply: Sender<Result<(), String>>,
    },
    Shutdown,
}

// Audio worker that runs in dedicated thread
struct AudioWorker {
    recorder: Option<AudioRecorder>,
    rx: Receiver<AudioCommand>,
}

impl AudioWorker {
    fn new(rx: Receiver<AudioCommand>) -> Self {
        Self {
            recorder: None,
            rx,
        }
    }

    fn run(mut self) {
        loop {
            match self.rx.recv() {
                Ok(AudioCommand::StartRecording) => {
                    if self.recorder.is_none() {
                        // Load settings to check if a specific device is configured
                        let settings = config::AppSettings::load().unwrap_or_default();

                        let recorder_result = if let Some(device_name) = settings.device_name {
                            println!("Using configured audio device: {}", device_name);
                            AudioRecorder::new_with_device(&device_name)
                        } else {
                            println!("Using default audio device");
                            AudioRecorder::new()
                        };

                        match recorder_result {
                            Ok(rec) => self.recorder = Some(rec),
                            Err(e) => eprintln!("Failed to create recorder: {}", e),
                        }
                    }

                    if let Some(ref mut recorder) = self.recorder {
                        if let Err(e) = recorder.start_recording() {
                            eprintln!("Failed to start recording: {}", e);
                        }
                    }
                }
                Ok(AudioCommand::StopRecording { reply }) => {
                    let result = if let Some(ref mut recorder) = self.recorder {
                        recorder.stop_recording()
                    } else {
                        Err("No recorder initialized".to_string())
                    };
                    let _ = reply.send(result);
                }
                Ok(AudioCommand::IsRecording { reply }) => {
                    let is_recording = self
                        .recorder
                        .as_ref()
                        .map_or(false, |r| r.is_recording());
                    let _ = reply.send(is_recording);
                }
                Ok(AudioCommand::Shutdown) | Err(_) => {
                    break;
                }
            }
        }
    }
}

// Transcription worker that runs in dedicated thread
struct TranscriptionWorker {
    engine: Option<TranscriptionEngine>,
    model_path: String,
    rx: Receiver<TranscriptionCommand>,
}

impl TranscriptionWorker {
    fn new(model_path: String, rx: Receiver<TranscriptionCommand>) -> Self {
        Self {
            engine: None,
            model_path,
            rx,
        }
    }

    fn run(mut self) {
        loop {
            match self.rx.recv() {
                Ok(TranscriptionCommand::Transcribe { audio, language, reply }) => {
                    println!("TranscriptionWorker: Received transcribe request with {} samples", audio.len());

                    // Lazy load the engine on first use
                    if self.engine.is_none() {
                        println!("TranscriptionWorker: Loading Whisper model from {}", self.model_path);
                        match TranscriptionEngine::new(&self.model_path) {
                            Ok(engine) => {
                                println!("TranscriptionWorker: Model loaded successfully");
                                self.engine = Some(engine);
                            }
                            Err(e) => {
                                eprintln!("Failed to load transcription engine: {}", e);
                                let _ = reply.send(Err(e));
                                continue;
                            }
                        }
                    }

                    println!("TranscriptionWorker: Starting transcription...");

                    // Load custom words from settings for contextual biasing
                    let settings = config::AppSettings::load().unwrap_or_default();
                    let custom_words = if !settings.custom_words.is_empty() {
                        Some(settings.custom_words.as_slice())
                    } else {
                        None
                    };

                    if let Some(words) = custom_words {
                        println!("Loaded {} custom words for contextual biasing", words.len());
                    }

                    let result = if let Some(ref mut engine) = self.engine {
                        engine.transcribe(&audio, language.as_deref(), custom_words)
                    } else {
                        Err("Transcription engine not initialized".to_string())
                    };

                    match &result {
                        Ok(text) => println!("TranscriptionWorker: Transcription successful: '{}'", text),
                        Err(e) => eprintln!("TranscriptionWorker: Transcription failed: {}", e),
                    }

                    let _ = reply.send(result);
                }
                Ok(TranscriptionCommand::ReloadModel { model_path, reply }) => {
                    println!("TranscriptionWorker: Reloading model from {}", model_path);

                    // Drop the old engine (unloads the model)
                    self.engine = None;
                    self.model_path = model_path.clone();

                    // Load the new model
                    match TranscriptionEngine::new(&model_path) {
                        Ok(engine) => {
                            println!("TranscriptionWorker: New model loaded successfully");
                            self.engine = Some(engine);
                            let _ = reply.send(Ok(()));
                        }
                        Err(e) => {
                            eprintln!("Failed to reload transcription engine: {}", e);
                            let _ = reply.send(Err(e));
                        }
                    }
                }
                Ok(TranscriptionCommand::Shutdown) | Err(_) => {
                    break;
                }
            }
        }
    }
}

// Application state (Send + Sync)
pub struct AppState {
    audio_tx: Sender<AudioCommand>,
    transcription_tx: Sender<TranscriptionCommand>,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn start_recording(state: State<'_, AppState>) -> Result<String, String> {
    state
        .audio_tx
        .send(AudioCommand::StartRecording)
        .map_err(|e| format!("Failed to send command: {}", e))?;
    Ok("Recording started".to_string())
}

#[tauri::command]
fn stop_recording(state: State<'_, AppState>) -> Result<Vec<f32>, String> {
    let (reply_tx, reply_rx) = mpsc::channel();
    state
        .audio_tx
        .send(AudioCommand::StopRecording { reply: reply_tx })
        .map_err(|e| format!("Failed to send command: {}", e))?;

    reply_rx
        .recv()
        .map_err(|e| format!("Failed to receive reply: {}", e))?
}

#[tauri::command]
fn is_recording(state: State<'_, AppState>) -> Result<bool, String> {
    let (reply_tx, reply_rx) = mpsc::channel();
    state
        .audio_tx
        .send(AudioCommand::IsRecording { reply: reply_tx })
        .map_err(|e| format!("Failed to send command: {}", e))?;

    reply_rx
        .recv()
        .map_err(|e| format!("Failed to receive reply: {}", e))
}

#[tauri::command]
fn transcribe(state: State<'_, AppState>, audio: Vec<f32>) -> Result<String, String> {
    // Load settings to get language preference
    let settings = config::AppSettings::load().unwrap_or_default();
    let language = Some(settings.language);

    let (reply_tx, reply_rx) = mpsc::channel();
    state
        .transcription_tx
        .send(TranscriptionCommand::Transcribe {
            audio,
            language,
            reply: reply_tx,
        })
        .map_err(|e| format!("Failed to send command: {}", e))?;

    reply_rx
        .recv()
        .map_err(|e| format!("Failed to receive reply: {}", e))?
}

#[tauri::command]
fn auto_paste(text: String) -> Result<(), String> {
    let clipboard = ClipboardManager::new()?;
    clipboard.auto_paste(&text)
}

#[tauri::command]
fn copy_to_clipboard(text: String) -> Result<(), String> {
    let clipboard = ClipboardManager::new()?;
    clipboard.copy_text(&text)
}

#[tauri::command]
fn get_settings() -> Result<config::AppSettings, String> {
    config::AppSettings::load()
}

#[tauri::command]
fn save_settings(settings: config::AppSettings) -> Result<(), String> {
    settings.save()
}

#[tauri::command]
fn update_hotkey(app: AppHandle, new_hotkey: String) -> Result<(), String> {
    use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
    use std::str::FromStr;

    // Load current settings
    let mut settings = config::AppSettings::load()?;
    let old_hotkey = settings.hotkey.clone();

    // Parse the new hotkey
    let shortcut = Shortcut::from_str(&new_hotkey)
        .map_err(|e| format!("Invalid hotkey format: {}", e))?;

    // Unregister the old hotkey
    if let Ok(old_shortcut) = Shortcut::from_str(&old_hotkey) {
        let _ = app.global_shortcut().unregister(old_shortcut);
        println!("Unregistered old hotkey: {}", old_hotkey);
    }

    // Register the new hotkey
    app.global_shortcut().register(shortcut)
        .map_err(|e| format!("Failed to register hotkey: {}", e))?;

    println!("Registered new hotkey: {}", new_hotkey);

    // Save the new hotkey to settings
    settings.hotkey = new_hotkey;
    settings.save()?;

    Ok(())
}

#[tauri::command]
fn get_audio_devices() -> Result<Vec<(String, bool)>, String> {
    audio::AudioRecorder::list_devices()
}

#[tauri::command]
fn update_cancel_key(app: AppHandle, new_cancel_key: String) -> Result<(), String> {
    use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
    use std::str::FromStr;

    // Load current settings
    let mut settings = config::AppSettings::load()?;
    let old_cancel_key = settings.cancel_key.clone();

    // Parse the new cancel key
    let shortcut = Shortcut::from_str(&new_cancel_key)
        .map_err(|e| format!("Invalid cancel key format: {}", e))?;

    // Unregister the old cancel key
    if let Ok(old_shortcut) = Shortcut::from_str(&old_cancel_key) {
        let _ = app.global_shortcut().unregister(old_shortcut);
        println!("Unregistered old cancel key: {}", old_cancel_key);
    }

    // Register the new cancel key
    app.global_shortcut().register(shortcut)
        .map_err(|e| format!("Failed to register cancel key: {}", e))?;

    println!("Registered new cancel key: {}", new_cancel_key);

    // Save the new cancel key to settings
    settings.cancel_key = new_cancel_key;
    settings.save()?;

    Ok(())
}

#[tauri::command]
fn reload_model(state: State<'_, AppState>, model_name: String) -> Result<(), String> {
    // Construct the full model path
    let model_path = std::env::var("FLEMME_MODEL_PATH")
        .ok()
        .and_then(|path| {
            // If FLEMME_MODEL_PATH is set, use its directory with the new model name
            let mut p = std::path::PathBuf::from(path);
            p.pop(); // Remove the filename
            p.push(&model_name);
            Some(p.to_string_lossy().to_string())
        })
        .unwrap_or_else(|| {
            // Otherwise use default location
            let mut path = dirs::data_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
            path.push("Flemme");
            path.push("models");
            path.push(&model_name);
            path.to_string_lossy().to_string()
        });

    println!("Reloading model: {}", model_path);

    // Send reload command to transcription worker
    let (reply_tx, reply_rx) = mpsc::channel();
    state
        .transcription_tx
        .send(TranscriptionCommand::ReloadModel {
            model_path,
            reply: reply_tx,
        })
        .map_err(|e| format!("Failed to send reload command: {}", e))?;

    reply_rx
        .recv()
        .map_err(|e| format!("Failed to receive reply: {}", e))?
}

#[tauri::command]
fn add_custom_word(word: String) -> Result<(), String> {
    let mut settings = config::AppSettings::load()?;

    // Avoid duplicates
    if !settings.custom_words.contains(&word) {
        settings.custom_words.push(word);
        settings.save()?;
        println!("Added custom word, total: {}", settings.custom_words.len());
    }

    Ok(())
}

#[tauri::command]
fn remove_custom_word(word: String) -> Result<(), String> {
    let mut settings = config::AppSettings::load()?;
    settings.custom_words.retain(|w| w != &word);
    settings.save()?;
    println!("Removed custom word, remaining: {}", settings.custom_words.len());
    Ok(())
}

#[tauri::command]
fn clear_custom_words() -> Result<(), String> {
    let mut settings = config::AppSettings::load()?;
    settings.custom_words.clear();
    settings.save()?;
    println!("Cleared all custom words");
    Ok(())
}

#[tauri::command]
fn get_custom_words() -> Result<Vec<String>, String> {
    let settings = config::AppSettings::load()?;
    Ok(settings.custom_words)
}

#[derive(serde::Serialize, Clone)]
struct ModelInfo {
    name: String,
    size_mb: f64,
    is_downloaded: bool,
    download_url: String,
}

#[tauri::command]
fn list_available_models() -> Result<Vec<ModelInfo>, String> {
    let models_dir = dirs::data_dir()
        .ok_or_else(|| "Failed to get data directory".to_string())?
        .join("Flemme")
        .join("models");

    // Create models directory if it doesn't exist
    if !models_dir.exists() {
        std::fs::create_dir_all(&models_dir)
            .map_err(|e| format!("Failed to create models directory: {}", e))?;
    }

    // List of available Whisper models with download URLs (Q5 quantized for better performance)
    let available_models = vec![
        ("ggml-base-q5_1.bin", 59.7, "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base-q5_1.bin"),
        ("ggml-small-q5_1.bin", 192.0, "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small-q5_1.bin"),
        ("ggml-medium-q5_0.bin", 940.0, "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium-q5_0.bin"),
        ("ggml-large-v2-q5_0.bin", 1820.0, "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v2-q5_0.bin"),
        ("ggml-large-v3-turbo-q5_0.bin", 950.0, "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo-q5_0.bin"),
    ];

    let mut result = Vec::new();
    for (name, size_mb, url) in available_models {
        let model_path = models_dir.join(name);
        let is_downloaded = model_path.exists();

        result.push(ModelInfo {
            name: name.to_string(),
            size_mb,
            is_downloaded,
            download_url: url.to_string(),
        });
    }

    Ok(result)
}

#[tauri::command]
fn delete_model(model_name: String) -> Result<(), String> {
    let models_dir = dirs::data_dir()
        .ok_or_else(|| "Failed to get data directory".to_string())?
        .join("Flemme")
        .join("models");

    let model_path = models_dir.join(&model_name);

    if !model_path.exists() {
        return Err(format!("Model '{}' does not exist", model_name));
    }

    std::fs::remove_file(&model_path)
        .map_err(|e| format!("Failed to delete model '{}': {}", model_name, e))?;

    println!("Model '{}' deleted successfully", model_name);
    Ok(())
}

#[derive(Clone, serde::Serialize)]
struct DownloadProgress {
    model_name: String,
    downloaded_bytes: u64,
    total_bytes: u64,
    percentage: f64,
}

#[tauri::command]
async fn download_model(
    app: AppHandle,
    model_name: String,
    download_url: String,
) -> Result<(), String> {
    use std::io::Write;

    let models_dir = dirs::data_dir()
        .ok_or_else(|| "Failed to get data directory".to_string())?
        .join("Flemme")
        .join("models");

    // Create models directory if it doesn't exist
    if !models_dir.exists() {
        std::fs::create_dir_all(&models_dir)
            .map_err(|e| format!("Failed to create models directory: {}", e))?;
    }

    let model_path = models_dir.join(&model_name);

    println!("Downloading model '{}' from {}", model_name, download_url);

    // Download with progress tracking
    let client = reqwest::Client::new();
    let response = client
        .get(&download_url)
        .send()
        .await
        .map_err(|e| format!("Failed to download: {}", e))?;

    let total_bytes = response.content_length().unwrap_or(0);
    let mut downloaded_bytes = 0u64;
    let mut last_emitted_percentage = 0.0;

    let mut file = std::fs::File::create(&model_path)
        .map_err(|e| format!("Failed to create file: {}", e))?;

    let mut stream = response.bytes_stream();
    use futures_util::StreamExt;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Download error: {}", e))?;
        file.write_all(&chunk)
            .map_err(|e| format!("Failed to write to file: {}", e))?;

        downloaded_bytes += chunk.len() as u64;
        let percentage = if total_bytes > 0 {
            (downloaded_bytes as f64 / total_bytes as f64) * 100.0
        } else {
            0.0
        };

        // Emit progress event only if percentage changed by at least 1%
        if (percentage - last_emitted_percentage).abs() >= 1.0 || downloaded_bytes == total_bytes {
            println!("Download progress: {:.1}% ({}/{} bytes)", percentage, downloaded_bytes, total_bytes);
            let _ = app.emit(
                "download-progress",
                DownloadProgress {
                    model_name: model_name.clone(),
                    downloaded_bytes,
                    total_bytes,
                    percentage,
                },
            );
            last_emitted_percentage = percentage;
        }
    }

    println!("Model '{}' downloaded successfully", model_name);
    Ok(())
}

/// Handle the complete workflow when recording finishes
/// Stop recording → Transcribe → Auto-paste
fn handle_recording_complete(
    audio_tx: Sender<AudioCommand>,
    transcription_tx: Sender<TranscriptionCommand>,
    _app_handle: AppHandle
) {
    thread::spawn(move || {
        let pipeline_start = std::time::Instant::now();

        // Stop recording and get audio data
        let stop_start = std::time::Instant::now();
        let (reply_tx, reply_rx) = mpsc::channel();
        if let Err(e) = audio_tx.send(AudioCommand::StopRecording { reply: reply_tx }) {
            eprintln!("Failed to send stop recording command: {}", e);
            return;
        }

        let audio_data = match reply_rx.recv() {
            Ok(Ok(data)) => data,
            Ok(Err(e)) => {
                eprintln!("Failed to stop recording: {}", e);
                return;
            }
            Err(e) => {
                eprintln!("Failed to receive audio data: {}", e);
                return;
            }
        };

        println!("[TIMING] stop_recording (with resampling): {:.0}ms", stop_start.elapsed().as_millis());
        println!("Recording stopped, got {} samples", audio_data.len());

        // Check if we have audio data
        if audio_data.is_empty() {
            eprintln!("No audio data recorded!");
            return;
        }

        // Apply Voice Activity Detection to filter silence
        println!("Applying VAD to filter silence...");

        // Add padding at the beginning to prevent VAD from cutting the start of speech
        // 150ms of silence gives VAD time to "warm up" and detect speech properly
        let padding_samples = (16000.0 * 0.15) as usize; // 150ms at 16kHz
        let mut padded_audio = vec![0.0; padding_samples];
        padded_audio.extend_from_slice(&audio_data);
        println!("Added {}ms padding before VAD (from {} to {} samples)",
                 padding_samples as f32 / 16.0, audio_data.len(), padded_audio.len());

        // Store original audio length before moving audio_data
        let original_audio_len = padded_audio.len();

        // Get VAD model path
        let vad_model_path = dirs::data_dir()
            .ok_or_else(|| "Failed to get data directory".to_string())
            .map(|d| d.join("Flemme").join("models").join("silero_vad.onnx"));

        let filtered_audio = match vad_model_path {
            Ok(model_path) if model_path.exists() => {
                match VoiceActivityDetector::new_default(&model_path) {
            Ok(mut vad) => {
                // Use 512 samples per chunk (32ms at 16kHz) for VAD analysis
                let chunk_size = 512;
                let filtered = vad.filter_silence(&padded_audio, chunk_size);

                let original_duration = padded_audio.len() as f32 / 16000.0;
                let filtered_duration = filtered.len() as f32 / 16000.0;
                let silence_removed = original_duration - filtered_duration;

                println!("VAD: Original={:.2}s, Filtered={:.2}s, Silence removed={:.2}s",
                         original_duration, filtered_duration, silence_removed);

                // If VAD filtered out everything, it means no speech was detected
                // Return empty audio to skip transcription
                filtered
            }
                    Err(e) => {
                        eprintln!("Failed to initialize VAD: {}. Using padded audio.", e);
                        padded_audio
                    }
                }
            }
            Ok(model_path) => {
                eprintln!("VAD model not found at {:?}. Using padded audio. Please download the model first.", model_path);
                padded_audio
            }
            Err(e) => {
                eprintln!("Failed to get VAD model path: {}. Using padded audio.", e);
                padded_audio
            }
        };

        // Check if we still have audio after VAD filtering
        // If VAD removed everything or most of the audio (>95%), it might be too aggressive
        let audio_to_transcribe = if filtered_audio.is_empty() {
            println!("Warning: VAD filtered out all audio. This might be a very short recording or pure silence.");
            println!("Skipping transcription.");
            return;
        } else if filtered_audio.len() < (original_audio_len / 20) {
            // Less than 5% remains - likely too aggressive, but still try to transcribe
            println!("Warning: VAD removed >95% of audio ({:.2}s -> {:.2}s). This might be a very short utterance.",
                     original_audio_len as f32 / 16000.0, filtered_audio.len() as f32 / 16000.0);
            filtered_audio
        } else {
            filtered_audio
        };

        println!("Sending audio to transcription engine...");

        // Load settings to get language preference
        let settings = config::AppSettings::load().unwrap_or_default();
        let language = Some(settings.language);

        // Transcribe the audio
        let transcribe_start = std::time::Instant::now();
        let (reply_tx, reply_rx) = mpsc::channel();
        if let Err(e) = transcription_tx.send(TranscriptionCommand::Transcribe {
            audio: audio_to_transcribe,
            language,
            reply: reply_tx,
        }) {
            eprintln!("Failed to send transcription command: {}", e);
            return;
        }

        let transcription = match reply_rx.recv() {
            Ok(Ok(text)) => text,
            Ok(Err(e)) => {
                eprintln!("Transcription failed: {}", e);
                return;
            }
            Err(e) => {
                eprintln!("Failed to receive transcription: {}", e);
                return;
            }
        };

        println!("[TIMING] Transcription worker (includes queue wait): {:.0}ms", transcribe_start.elapsed().as_millis());
        println!("Transcription completed: {}", transcription);

        // Process through LLM if using a custom execution mode
        let settings = config::AppSettings::load().unwrap_or_default();
        let final_text = if !transcription.is_empty() && settings.active_mode != "standard" {
            // Find the active execution mode
            let mode = settings.execution_modes.iter()
                .find(|m| m.id == settings.active_mode);

            if let Some(mode) = mode {
                if let Some(ref llm_model_id) = mode.llm_model_id {
                    // Find the LLM model
                    let llm_model = settings.llm_models.iter()
                        .find(|m| m.id == *llm_model_id);

                    if let Some(llm_model) = llm_model {
                        println!("=== EXECUTING MODE: {} ===", mode.name);
                        println!("Using LLM: {}", llm_model.name);

                        // Get API key from keyring
                        match llm::keyring_manager::get_api_key(llm_model_id) {
                            Ok(Some(api_key)) => {
                                // Call the LLM asynchronously
                                let llm_model_clone = llm_model.clone();
                                let system_prompt = mode.system_prompt.clone();
                                let transcription_clone = transcription.clone();

                                println!("Calling LLM API...");
                                let runtime = tokio::runtime::Runtime::new().unwrap();
                                match runtime.block_on(llm::call_llm(
                                    &llm_model_clone,
                                    &api_key,
                                    &system_prompt,
                                    &transcription_clone
                                )) {
                                    Ok(llm_response) => {
                                        println!("LLM processing successful");
                                        llm_response
                                    }
                                    Err(e) => {
                                        eprintln!("LLM call failed: {}", e);
                                        eprintln!("Falling back to raw transcription");
                                        transcription
                                    }
                                }
                            }
                            Ok(None) => {
                                eprintln!("No API key found for LLM: {}", llm_model_id);
                                eprintln!("Falling back to raw transcription");
                                transcription
                            }
                            Err(e) => {
                                eprintln!("Failed to retrieve API key: {}", e);
                                eprintln!("Falling back to raw transcription");
                                transcription
                            }
                        }
                    } else {
                        eprintln!("LLM model not found: {}", llm_model_id);
                        eprintln!("Falling back to raw transcription");
                        transcription
                    }
                } else {
                    // Mode has no LLM configured, use raw transcription
                    transcription
                }
            } else {
                eprintln!("Active mode not found: {}", settings.active_mode);
                eprintln!("Falling back to raw transcription");
                transcription
            }
        } else {
            // Standard mode or empty transcription
            transcription
        };

        println!("[TIMING] ==========================================");
        println!("[TIMING] TOTAL PIPELINE: {:.0}ms", pipeline_start.elapsed().as_millis());
        println!("[TIMING] ==========================================");

        // Auto-paste the final text if enabled in settings
        if !final_text.is_empty() {

            if settings.auto_paste {
                match ClipboardManager::new() {
                    Ok(clipboard) => {
                        if let Err(e) = clipboard.auto_paste(&final_text) {
                            eprintln!("Failed to auto-paste: {}", e);
                        } else {
                            println!("Text auto-pasted successfully");
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to create clipboard manager: {}", e);
                    }
                }
            } else {
                println!("Auto-paste disabled, copying to clipboard only");
                match ClipboardManager::new() {
                    Ok(clipboard) => {
                        if let Err(e) = clipboard.copy_text(&final_text) {
                            eprintln!("Failed to copy to clipboard: {}", e);
                        } else {
                            println!("Text copied to clipboard");
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to create clipboard manager: {}", e);
                    }
                }
            }
        } else {
            println!("No text to paste (empty result)");
        }
    });
}

// ============================================================================
// LLM Model Management Commands
// ============================================================================

/// Get all configured LLM models
#[tauri::command]
fn get_llm_models() -> Result<Vec<LlmModel>, String> {
    let settings = config::AppSettings::load()
        .map_err(|e| format!("Failed to load settings: {}", e))?;
    Ok(settings.llm_models)
}

/// Add a new LLM model
#[tauri::command]
fn add_llm_model(
    name: String,
    api_url: String,
    model_name: String,
    api_key: String,
) -> Result<String, String> {
    let mut settings = config::AppSettings::load()
        .map_err(|e| format!("Failed to load settings: {}", e))?;

    // Generate a unique ID
    let id = format!("llm_{}", uuid::Uuid::new_v4().to_string());

    // Store API key in keyring
    llm::keyring_manager::store_api_key(&id, &api_key)?;

    // Add model to settings
    settings.llm_models.push(LlmModel {
        id: id.clone(),
        name,
        api_url,
        model_name,
    });

    settings.save()
        .map_err(|e| format!("Failed to save settings: {}", e))?;

    println!("LLM model added with ID: {}", id);
    Ok(id)
}

/// Update an existing LLM model
#[tauri::command]
fn update_llm_model(
    id: String,
    name: String,
    api_url: String,
    model_name: String,
    api_key: Option<String>,
) -> Result<(), String> {
    let mut settings = config::AppSettings::load()
        .map_err(|e| format!("Failed to load settings: {}", e))?;

    // Find the model
    let model = settings.llm_models.iter_mut()
        .find(|m| m.id == id)
        .ok_or_else(|| format!("LLM model not found: {}", id))?;

    // Update model fields
    model.name = name;
    model.api_url = api_url;
    model.model_name = model_name;

    // Update API key if provided
    if let Some(key) = api_key {
        llm::keyring_manager::store_api_key(&id, &key)?;
    }

    settings.save()
        .map_err(|e| format!("Failed to save settings: {}", e))?;

    println!("LLM model updated: {}", id);
    Ok(())
}

/// Delete an LLM model
#[tauri::command]
fn delete_llm_model(id: String) -> Result<(), String> {
    let mut settings = config::AppSettings::load()
        .map_err(|e| format!("Failed to load settings: {}", e))?;

    // Check if any execution mode uses this model
    let in_use = settings.execution_modes.iter()
        .any(|mode| mode.llm_model_id.as_ref() == Some(&id));

    if in_use {
        return Err(format!("Cannot delete LLM model '{}': it is used by one or more execution modes", id));
    }

    // Remove from settings
    let initial_len = settings.llm_models.len();
    settings.llm_models.retain(|m| m.id != id);

    if settings.llm_models.len() == initial_len {
        return Err(format!("LLM model not found: {}", id));
    }

    // Delete API key from keyring
    llm::keyring_manager::delete_api_key(&id)?;

    settings.save()
        .map_err(|e| format!("Failed to save settings: {}", e))?;

    println!("LLM model deleted: {}", id);
    Ok(())
}

// ============================================================================
// Execution Mode Management Commands
// ============================================================================

/// Get all execution modes
#[tauri::command]
fn get_execution_modes() -> Result<Vec<ExecutionMode>, String> {
    let settings = config::AppSettings::load()
        .map_err(|e| format!("Failed to load settings: {}", e))?;
    Ok(settings.execution_modes)
}

/// Get the currently active execution mode ID
#[tauri::command]
fn get_active_mode() -> Result<String, String> {
    let settings = config::AppSettings::load()
        .map_err(|e| format!("Failed to load settings: {}", e))?;
    Ok(settings.active_mode)
}

/// Set the active execution mode
#[tauri::command]
fn set_active_mode(mode_id: String) -> Result<(), String> {
    let mut settings = config::AppSettings::load()
        .map_err(|e| format!("Failed to load settings: {}", e))?;

    // Verify the mode exists
    if !settings.execution_modes.iter().any(|m| m.id == mode_id) {
        return Err(format!("Execution mode not found: {}", mode_id));
    }

    settings.active_mode = mode_id.clone();
    settings.save()
        .map_err(|e| format!("Failed to save settings: {}", e))?;

    println!("Active execution mode set to: {}", mode_id);
    Ok(())
}

/// Add a new execution mode
#[tauri::command]
fn add_execution_mode(
    name: String,
    llm_model_id: Option<String>,
    system_prompt: String,
) -> Result<String, String> {
    let mut settings = config::AppSettings::load()
        .map_err(|e| format!("Failed to load settings: {}", e))?;

    // If an LLM model is specified, verify it exists
    if let Some(ref model_id) = llm_model_id {
        if !settings.llm_models.iter().any(|m| m.id == *model_id) {
            return Err(format!("LLM model not found: {}", model_id));
        }
    }

    // Generate a unique ID
    let id = format!("mode_{}", uuid::Uuid::new_v4().to_string());

    // Add mode to settings
    settings.execution_modes.push(ExecutionMode {
        id: id.clone(),
        name,
        llm_model_id,
        system_prompt,
    });

    settings.save()
        .map_err(|e| format!("Failed to save settings: {}", e))?;

    println!("Execution mode added with ID: {}", id);
    Ok(id)
}

/// Update an existing execution mode
#[tauri::command]
fn update_execution_mode(
    id: String,
    name: String,
    llm_model_id: Option<String>,
    system_prompt: String,
) -> Result<(), String> {
    // Prevent modifying the standard mode
    if id == "standard" {
        return Err("Cannot modify the built-in 'standard' mode".to_string());
    }

    let mut settings = config::AppSettings::load()
        .map_err(|e| format!("Failed to load settings: {}", e))?;

    // If an LLM model is specified, verify it exists
    if let Some(ref model_id) = llm_model_id {
        if !settings.llm_models.iter().any(|m| m.id == *model_id) {
            return Err(format!("LLM model not found: {}", model_id));
        }
    }

    // Find the mode
    let mode = settings.execution_modes.iter_mut()
        .find(|m| m.id == id)
        .ok_or_else(|| format!("Execution mode not found: {}", id))?;

    // Update mode fields
    mode.name = name;
    mode.llm_model_id = llm_model_id;
    mode.system_prompt = system_prompt;

    settings.save()
        .map_err(|e| format!("Failed to save settings: {}", e))?;

    println!("Execution mode updated: {}", id);
    Ok(())
}

/// Delete an execution mode
#[tauri::command]
fn delete_execution_mode(id: String) -> Result<(), String> {
    // Prevent deleting the standard mode
    if id == "standard" {
        return Err("Cannot delete the built-in 'standard' mode".to_string());
    }

    let mut settings = config::AppSettings::load()
        .map_err(|e| format!("Failed to load settings: {}", e))?;

    // If this is the active mode, switch to standard
    if settings.active_mode == id {
        settings.active_mode = String::from("standard");
        println!("Switched active mode to 'standard' because deleted mode was active");
    }

    // Remove from settings
    let initial_len = settings.execution_modes.len();
    settings.execution_modes.retain(|m| m.id != id);

    if settings.execution_modes.len() == initial_len {
        return Err(format!("Execution mode not found: {}", id));
    }

    settings.save()
        .map_err(|e| format!("Failed to save settings: {}", e))?;

    println!("Execution mode deleted: {}", id);
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Load settings to get the configured model
    let settings = config::AppSettings::load().unwrap_or_default();

    // Determine model path
    // Default: look for model in user's AppData/Roaming/Flemme/models/
    let model_path = std::env::var("FLEMME_MODEL_PATH")
        .unwrap_or_else(|_| {
            let mut path = dirs::data_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
            path.push("Flemme");
            path.push("models");
            path.push(&settings.model_name); // Use the model from settings
            path.to_string_lossy().to_string()
        });

    println!("Using whisper model at: {} (from settings: {})", model_path, settings.model_name);

    // Create channel for audio commands
    let (audio_tx, audio_rx) = mpsc::channel();

    // Create channel for transcription commands
    let (transcription_tx, transcription_rx) = mpsc::channel();

    // Spawn audio worker thread
    thread::spawn(move || {
        let worker = AudioWorker::new(audio_rx);
        worker.run();
    });

    // Spawn transcription worker thread
    thread::spawn(move || {
        let worker = TranscriptionWorker::new(model_path, transcription_rx);
        worker.run();
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            audio_tx: audio_tx.clone(),
            transcription_tx: transcription_tx.clone(),
        })
        .setup(move |app| {
            #[cfg(desktop)]
            {
                use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
                use tauri::menu::{MenuBuilder, MenuItemBuilder};
                use tauri::tray::{TrayIconBuilder, TrayIconEvent};
                use tauri::Manager;
                use tauri::{WebviewUrl, WebviewWindowBuilder};

                // Create overlay window for recording indicator
                let indicator_window = WebviewWindowBuilder::new(
                    app,
                    "indicator",
                    WebviewUrl::App("indicator".into())
                )
                .title("Recording Indicator")
                .inner_size(350.0, 80.0)
                .decorations(false)
                .transparent(false)  // Fond blanc opaque
                .always_on_top(true)
                .skip_taskbar(true)
                .resizable(false)
                .visible(false)  // Hidden by default
                .build()
                .expect("Failed to create indicator window");

                // Position window at bottom center of screen
                if let Ok(monitor) = indicator_window.current_monitor() {
                    if let Some(monitor) = monitor {
                        let screen_size = monitor.size();
                        let window_size = indicator_window.outer_size().unwrap();

                        // Center horizontally, 100px from bottom
                        let x = (screen_size.width as i32 - window_size.width as i32) / 2;
                        let y = screen_size.height as i32 - window_size.height as i32 - 100;

                        let _ = indicator_window.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }));
                    }
                }

                let shortcut = HotkeyListener::get_record_shortcut();
                let audio_tx_clone = audio_tx.clone();
                let transcription_tx_clone = transcription_tx.clone();
                let app_handle = app.handle().clone();

                // Track recording state for toggle mode
                use std::sync::{Arc, Mutex};
                let is_recording = Arc::new(Mutex::new(false));
                let is_recording_clone = is_recording.clone();
                let is_recording_for_cancel = is_recording.clone();

                // Clone for cancel handler
                let audio_tx_for_cancel = audio_tx.clone();

                // Load cancel key
                let settings = config::AppSettings::load().unwrap_or_default();
                let cancel_key = settings.cancel_key.clone();

                // Register the global shortcut plugin with handler for both main and cancel keys
                app.handle().plugin(
                    tauri_plugin_global_shortcut::Builder::new()
                        .with_handler(move |_app, shortcut, event| {
                            // Load settings to check push_to_talk mode
                            let settings = config::AppSettings::load().unwrap_or_default();
                            let shortcut_str = shortcut.to_string();

                            // Check if this is the cancel key
                            if shortcut_str == cancel_key {
                                // Only handle in toggle mode when recording is active
                                if let ShortcutState::Pressed = event.state() {
                                    // Cancel key only works in toggle mode
                                    if !settings.push_to_talk {
                                        let mut recording = is_recording_for_cancel.lock().unwrap();

                                        if *recording {
                                            println!("Cancel key pressed - stopping recording without transcription");
                                            *recording = false;
                                            let _ = _app.emit("recording-stopped", ());

                                            // Hide indicator window
                                            if let Some(window) = _app.get_webview_window("indicator") {
                                                let _ = window.hide();
                                            }

                                            // Stop recording but don't transcribe
                                            let (reply_tx, _reply_rx) = mpsc::channel();
                                            let _ = audio_tx_for_cancel.send(AudioCommand::StopRecording { reply: reply_tx });
                                        }
                                    }
                                }
                                return;
                            }

                            // Otherwise, handle the main recording hotkey
                            if settings.push_to_talk {
                                // Push-to-Talk mode: press to start, release to stop
                                match event.state() {
                                    ShortcutState::Pressed => {
                                        println!("Hotkey pressed (push-to-talk) - starting recording");
                                        let _ = audio_tx_clone.send(AudioCommand::StartRecording);
                                        let _ = app_handle.emit("recording-started", ());

                                        // Show indicator window
                                        if let Some(window) = _app.get_webview_window("indicator") {
                                            let _ = window.show();
                                            let _ = window.set_focus();
                                        }
                                    }
                                    ShortcutState::Released => {
                                        println!("Hotkey released (push-to-talk) - stopping recording and transcribing");
                                        let _ = app_handle.emit("recording-stopped", ());

                                        // Hide indicator window
                                        if let Some(window) = _app.get_webview_window("indicator") {
                                            let _ = window.hide();
                                        }

                                        handle_recording_complete(
                                            audio_tx_clone.clone(),
                                            transcription_tx_clone.clone(),
                                            app_handle.clone()
                                        );
                                    }
                                }
                            } else {
                                // Toggle mode: press once to start, press again to stop
                                if let ShortcutState::Pressed = event.state() {
                                    let mut recording = is_recording_clone.lock().unwrap();

                                    if *recording {
                                        // Already recording, stop it
                                        println!("Hotkey pressed (toggle) - stopping recording and transcribing");
                                        *recording = false;
                                        let _ = app_handle.emit("recording-stopped", ());

                                        // Hide indicator window
                                        if let Some(window) = _app.get_webview_window("indicator") {
                                            let _ = window.hide();
                                        }

                                        handle_recording_complete(
                                            audio_tx_clone.clone(),
                                            transcription_tx_clone.clone(),
                                            app_handle.clone()
                                        );
                                    } else {
                                        // Not recording, start it
                                        println!("Hotkey pressed (toggle) - starting recording");
                                        *recording = true;
                                        let _ = app_handle.emit("recording-started", ());
                                        let _ = audio_tx_clone.send(AudioCommand::StartRecording);

                                        // Show indicator window
                                        if let Some(window) = _app.get_webview_window("indicator") {
                                            let _ = window.show();
                                            let _ = window.set_focus();
                                        }
                                    }
                                }
                            }
                        })
                        .build()
                )?;

                // Register both shortcuts
                app.global_shortcut().register(shortcut)?;
                println!("Global shortcut registered: Ctrl+Alt+R");

                // Register cancel shortcut
                use std::str::FromStr;
                use tauri_plugin_global_shortcut::Shortcut;
                if let Ok(cancel_shortcut) = Shortcut::from_str(&settings.cancel_key) {
                    app.global_shortcut().register(cancel_shortcut)?;
                    println!("Cancel shortcut registered: {}", settings.cancel_key);
                }

                // Clone for menu event handler
                let audio_tx_for_quit = audio_tx.clone();
                let transcription_tx_for_quit = transcription_tx.clone();

                // Create system tray menu
                let settings_item = MenuItemBuilder::with_id("settings", "Paramètres").build(app)?;

                // Load execution modes and create Modes submenu
                use tauri::menu::SubmenuBuilder;
                let app_settings = config::AppSettings::load().unwrap_or_default();
                let mut modes_submenu = SubmenuBuilder::new(app, "Modes");

                for mode in &app_settings.execution_modes {
                    let mode_id = format!("mode_{}", mode.id);
                    let mode_label = if mode.id == app_settings.active_mode {
                        format!("✓ {}", mode.name)
                    } else {
                        mode.name.clone()
                    };
                    let mode_item = MenuItemBuilder::with_id(&mode_id, mode_label).build(app)?;
                    modes_submenu = modes_submenu.item(&mode_item);
                }

                let modes_menu = modes_submenu.build()?;
                let quit = MenuItemBuilder::with_id("quit", "Quitter").build(app)?;

                let menu = MenuBuilder::new(app)
                    .items(&[&settings_item, &modes_menu, &quit])
                    .build()?;

                // Build the tray icon
                let _tray = TrayIconBuilder::new()
                    .icon(app.default_window_icon().unwrap().clone())
                    .menu(&menu)
                    .on_menu_event(move |app, event| {
                        let event_id = event.id().as_ref();

                        match event_id {
                            "settings" => {
                                // Show main window when settings is clicked
                                if let Some(window) = app.get_webview_window("main") {
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                }
                            }
                            "quit" => {
                                // Send shutdown commands to worker threads for graceful shutdown
                                println!("Shutting down worker threads...");
                                let _ = audio_tx_for_quit.send(AudioCommand::Shutdown);
                                let _ = transcription_tx_for_quit.send(TranscriptionCommand::Shutdown);

                                // Close all windows first (fixes Chrome_WidgetWin_0 error)
                                let windows: Vec<_> = app.webview_windows().into_values().collect();
                                for window in windows {
                                    let _ = window.close();
                                }

                                // Give threads and webview time to cleanup
                                std::thread::sleep(std::time::Duration::from_millis(200));
                                println!("Worker threads shutdown complete");
                                app.exit(0);
                            }
                            id if id.starts_with("mode_") => {
                                // Extract the mode ID by removing "mode_" prefix
                                let mode_id = id.strip_prefix("mode_").unwrap();

                                // Set the active mode
                                if let Ok(mut settings) = config::AppSettings::load() {
                                    if settings.execution_modes.iter().any(|m| m.id == mode_id) {
                                        settings.active_mode = mode_id.to_string();
                                        if let Err(e) = settings.save() {
                                            eprintln!("Failed to save active mode: {}", e);
                                        } else {
                                            println!("Active mode changed to: {}", mode_id);
                                            // TODO: Rebuild tray menu to update checkmarks
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    })
                    .on_tray_icon_event(|tray, event| {
                        if let TrayIconEvent::DoubleClick { .. } = event {
                            // Double-click to show window (Windows only)
                            if let Some(app) = tray.app_handle().get_webview_window("main") {
                                let _ = app.show();
                                let _ = app.set_focus();
                            }
                        }
                    })
                    .build(app)?;
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            start_recording,
            stop_recording,
            is_recording,
            transcribe,
            auto_paste,
            copy_to_clipboard,
            get_settings,
            save_settings,
            update_hotkey,
            update_cancel_key,
            reload_model,
            get_audio_devices,
            add_custom_word,
            remove_custom_word,
            clear_custom_words,
            get_custom_words,
            list_available_models,
            download_model,
            delete_model,
            get_llm_models,
            add_llm_model,
            update_llm_model,
            delete_llm_model,
            get_execution_modes,
            get_active_mode,
            set_active_mode,
            add_execution_mode,
            update_execution_mode,
            delete_execution_mode
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
