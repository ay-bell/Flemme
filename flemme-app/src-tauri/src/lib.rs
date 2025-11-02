// Module declarations
pub mod audio;
pub mod transcription;
pub mod hotkey;
pub mod clipboard;
pub mod config;

use audio::AudioRecorder;
use transcription::TranscriptionEngine;
use clipboard::ClipboardManager;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use tauri::State;

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
                    // Lazy load the engine on first use
                    if self.engine.is_none() {
                        match TranscriptionEngine::new(&self.model_path) {
                            Ok(engine) => self.engine = Some(engine),
                            Err(e) => {
                                eprintln!("Failed to load transcription engine: {}", e);
                                let _ = reply.send(Err(e));
                                continue;
                            }
                        }
                    }

                    let result = if let Some(ref engine) = self.engine {
                        engine.transcribe(&audio)
                    } else {
                        Err("Transcription engine not initialized".to_string())
                    };
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
            audio_tx,
            transcription_tx,
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
