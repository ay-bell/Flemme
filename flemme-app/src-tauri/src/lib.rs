// Module declarations
pub mod audio;
pub mod transcription;
pub mod hotkey;
pub mod clipboard;
pub mod config;

use audio::AudioRecorder;
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

// Application state (Send + Sync)
pub struct AppState {
    audio_tx: Sender<AudioCommand>,
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Create channel for audio commands
    let (audio_tx, audio_rx) = mpsc::channel();

    // Spawn audio worker thread
    thread::spawn(move || {
        let worker = AudioWorker::new(audio_rx);
        worker.run();
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState { audio_tx })
        .invoke_handler(tauri::generate_handler![
            greet,
            start_recording,
            stop_recording,
            is_recording
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
