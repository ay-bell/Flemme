// AudioRecorder - handles audio recording from microphone
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig};
use std::sync::{Arc, Mutex};

pub struct AudioRecorder {
    device: Device,
    config: StreamConfig,
    stream: Option<Stream>,
    buffer: Arc<Mutex<Vec<f32>>>,
    sample_rate: u32,
}

impl AudioRecorder {
    /// Create a new recorder with the default microphone
    pub fn new() -> Result<Self, String> {
        let host = cpal::default_host();

        let device = host
            .default_input_device()
            .ok_or("No microphone found")?;

        let config = device
            .default_input_config()
            .map_err(|e| format!("Config error: {}", e))?;

        Ok(Self {
            device,
            config: config.into(),
            stream: None,
            buffer: Arc::new(Mutex::new(Vec::new())),
            sample_rate: 16000, // Whisper requires 16kHz
        })
    }

    /// Start recording audio
    pub fn start_recording(&mut self) -> Result<(), String> {
        let buffer = Arc::clone(&self.buffer);

        // Clear the buffer
        buffer.lock().unwrap().clear();

        let stream = self.device
            .build_input_stream(
                &self.config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    // Callback called for each audio chunk
                    let mut buf = buffer.lock().unwrap();
                    buf.extend_from_slice(data);
                },
                |err| eprintln!("Stream error: {}", err),
                None,
            )
            .map_err(|e| format!("Failed to create stream: {}", e))?;

        stream.play().map_err(|e| format!("Failed to play stream: {}", e))?;

        self.stream = Some(stream);
        Ok(())
    }

    /// Stop recording and return the audio samples
    pub fn stop_recording(&mut self) -> Result<Vec<f32>, String> {
        if let Some(stream) = self.stream.take() {
            drop(stream); // Stop the stream
        }

        let buffer = self.buffer.lock().unwrap();
        let audio = buffer.clone();

        Ok(audio)
    }

    /// Check if currently recording
    pub fn is_recording(&self) -> bool {
        self.stream.is_some()
    }

    /// Get the sample rate (16kHz for Whisper)
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}