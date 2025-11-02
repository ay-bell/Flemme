// Module declarations
pub mod audio;
pub mod transcription;
pub mod hotkey;
pub mod clipboard;
pub mod config;

use audio::AudioRecorder;
use transcription::TranscriptionEngine;
use clipboard::ClipboardManager;
use hotkey::HotkeyListener;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use tauri::{AppHandle, State};

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
        reply: Sender<Result<String, String>>,
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
                        match AudioRecorder::new() {
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
                Ok(TranscriptionCommand::Transcribe { audio, reply }) => {
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
                    let result = if let Some(ref engine) = self.engine {
                        engine.transcribe(&audio)
                    } else {
                        Err("Transcription engine not initialized".to_string())
                    };

                    match &result {
                        Ok(text) => println!("TranscriptionWorker: Transcription successful: '{}'", text),
                        Err(e) => eprintln!("TranscriptionWorker: Transcription failed: {}", e),
                    }

                    let _ = reply.send(result);
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
    let (reply_tx, reply_rx) = mpsc::channel();
    state
        .transcription_tx
        .send(TranscriptionCommand::Transcribe {
            audio,
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

/// Handle the complete workflow when recording finishes
/// Stop recording → Transcribe → Auto-paste
fn handle_recording_complete(
    audio_tx: Sender<AudioCommand>,
    transcription_tx: Sender<TranscriptionCommand>,
    _app_handle: AppHandle
) {
    thread::spawn(move || {
        // Stop recording and get audio data
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

        println!("Recording stopped, got {} samples", audio_data.len());

        // Check if we have audio data
        if audio_data.is_empty() {
            eprintln!("No audio data recorded!");
            return;
        }

        println!("Sending audio to transcription engine...");

        // Transcribe the audio
        let (reply_tx, reply_rx) = mpsc::channel();
        if let Err(e) = transcription_tx.send(TranscriptionCommand::Transcribe {
            audio: audio_data,
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

        println!("Transcription completed: {}", transcription);

        // Auto-paste the transcribed text
        if !transcription.is_empty() {
            match ClipboardManager::new() {
                Ok(clipboard) => {
                    if let Err(e) = clipboard.auto_paste(&transcription) {
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
            println!("No transcription to paste (empty result)");
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Determine model path
    // Default: look for model in user's AppData/Roaming/Flemme/models/
    let model_path = std::env::var("FLEMME_MODEL_PATH")
        .unwrap_or_else(|_| {
            let mut path = dirs::data_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
            path.push("Flemme");
            path.push("models");
            path.push("ggml-small.bin");
            path.to_string_lossy().to_string()
        });

    println!("Using whisper model at: {}", model_path);

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

                let shortcut = HotkeyListener::get_record_shortcut();
                let audio_tx_clone = audio_tx.clone();
                let transcription_tx_clone = transcription_tx.clone();
                let app_handle = app.handle().clone();

                // Register the global shortcut plugin with handler
                app.handle().plugin(
                    tauri_plugin_global_shortcut::Builder::new()
                        .with_handler(move |_app, _shortcut, event| {
                            match event.state() {
                                ShortcutState::Pressed => {
                                    println!("Hotkey pressed - starting recording");
                                    let _ = audio_tx_clone.send(AudioCommand::StartRecording);
                                }
                                ShortcutState::Released => {
                                    println!("Hotkey released - stopping recording and transcribing");
                                    handle_recording_complete(
                                        audio_tx_clone.clone(),
                                        transcription_tx_clone.clone(),
                                        app_handle.clone()
                                    );
                                }
                            }
                        })
                        .build()
                )?;

                // Register the shortcut
                app.global_shortcut().register(shortcut)?;
                println!("Global shortcut registered: Ctrl+Alt+R");
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
            copy_to_clipboard
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
