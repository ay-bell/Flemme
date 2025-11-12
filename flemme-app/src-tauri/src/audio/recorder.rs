// AudioRecorder - handles audio recording from microphone
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig};
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use rubato::{FftFixedIn, Resampler};

pub struct AudioRecorder {
    device: Device,
    config: StreamConfig,
    stream: Option<Stream>,
    buffer: Arc<Mutex<Vec<f32>>>,
    sample_rate: u32,
    resampler: Option<RefCell<FftFixedIn<f32>>>,
}

impl AudioRecorder {
    /// List all available input devices
    pub fn list_devices() -> Result<Vec<(String, bool)>, String> {
        let host = cpal::default_host();
        let default_device = host.default_input_device();
        let default_name = default_device
            .as_ref()
            .and_then(|d| d.name().ok());

        let devices = host
            .input_devices()
            .map_err(|e| format!("Failed to get input devices: {}", e))?;

        let mut result = Vec::new();
        for device in devices {
            if let Ok(name) = device.name() {
                let is_default = Some(&name) == default_name.as_ref();
                result.push((name, is_default));
            }
        }

        if result.is_empty() {
            return Err("No input devices found".to_string());
        }

        Ok(result)
    }

    /// Create a new recorder with the default microphone
    pub fn new() -> Result<Self, String> {
        let host = cpal::default_host();

        let device = host
            .default_input_device()
            .ok_or("No microphone found")?;

        let config = device
            .default_input_config()
            .map_err(|e| format!("Config error: {}", e))?;

        let stream_config: StreamConfig = config.into();

        // Create resampler if device sample rate differs from 16kHz
        let resampler = if stream_config.sample_rate.0 != 16000 {
            let chunk_size = 1024; // Process in 1024-sample chunks for good quality/performance balance
            let sub_chunks = 2; // Number of subchunks for FFT processing
            match FftFixedIn::<f32>::new(
                stream_config.sample_rate.0 as usize,
                16000,
                chunk_size,
                sub_chunks,
                1, // mono (number of channels)
            ) {
                Ok(r) => {
                    println!("Created high-quality FFT resampler: {} Hz -> 16000 Hz", stream_config.sample_rate.0);
                    Some(RefCell::new(r))
                }
                Err(e) => {
                    eprintln!("Failed to create resampler: {}, will use fallback", e);
                    None
                }
            }
        } else {
            None
        };

        Ok(Self {
            device,
            config: stream_config,
            stream: None,
            buffer: Arc::new(Mutex::new(Vec::new())),
            sample_rate: 16000, // Whisper requires 16kHz
            resampler,
        })
    }

    /// Create a new recorder with a specific device by name
    pub fn new_with_device(device_name: &str) -> Result<Self, String> {
        let host = cpal::default_host();

        // Find device by name
        let devices = host
            .input_devices()
            .map_err(|e| format!("Failed to get input devices: {}", e))?;

        let device = devices
            .filter_map(|d| {
                d.name().ok().and_then(|name| {
                    if name == device_name {
                        Some(d)
                    } else {
                        None
                    }
                })
            })
            .next()
            .ok_or_else(|| format!("Device '{}' not found", device_name))?;

        let config = device
            .default_input_config()
            .map_err(|e| format!("Config error: {}", e))?;

        let stream_config: StreamConfig = config.into();

        // Create resampler if device sample rate differs from 16kHz
        let resampler = if stream_config.sample_rate.0 != 16000 {
            let chunk_size = 1024; // Process in 1024-sample chunks for good quality/performance balance
            let sub_chunks = 2; // Number of subchunks for FFT processing
            match FftFixedIn::<f32>::new(
                stream_config.sample_rate.0 as usize,
                16000,
                chunk_size,
                sub_chunks,
                1, // mono (number of channels)
            ) {
                Ok(r) => {
                    println!("Created high-quality FFT resampler: {} Hz -> 16000 Hz", stream_config.sample_rate.0);
                    Some(RefCell::new(r))
                }
                Err(e) => {
                    eprintln!("Failed to create resampler: {}, will use fallback", e);
                    None
                }
            }
        } else {
            None
        };

        Ok(Self {
            device,
            config: stream_config,
            stream: None,
            buffer: Arc::new(Mutex::new(Vec::new())),
            sample_rate: 16000, // Whisper requires 16kHz
            resampler,
        })
    }

    /// Start recording audio
    pub fn start_recording(&mut self) -> Result<(), String> {
        let buffer = Arc::clone(&self.buffer);
        let channels = self.config.channels;

        println!("Recording with {} channels at {} Hz", channels, self.config.sample_rate.0);

        // Clear the buffer
        buffer.lock().unwrap().clear();

        let stream = self.device
            .build_input_stream(
                &self.config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    // Callback called for each audio chunk
                    let mut buf = buffer.lock().unwrap();

                    // Convert to mono if stereo (average both channels for maximum info preservation)
                    if channels == 2 {
                        for i in (0..data.len()).step_by(2) {
                            let left = data[i];
                            let right = data[i + 1];
                            buf.push((left + right) / 2.0);
                        }
                    } else {
                        buf.extend_from_slice(data);
                    }
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

        println!("Audio recorded: {} samples at {} Hz", audio.len(), self.config.sample_rate.0);

        // Remove DC offset (improves VAD quality)
        let mut audio = audio;
        Self::remove_dc_offset(&mut audio);

        // Normalize audio to full dynamic range (critical for consistent VAD performance)
        Self::normalize_peak(&mut audio);

        // Resample to 16kHz if needed using high-quality FFT resampler
        if self.config.sample_rate.0 != 16000 {
            println!("Resampling from {} Hz to 16000 Hz using FFT resampler", self.config.sample_rate.0);
            let resampled = self.resample_with_rubato(&audio)?;
            println!("Resampled to {} samples", resampled.len());
            Ok(resampled)
        } else {
            Ok(audio)
        }
    }

    /// Remove DC offset from audio signal
    fn remove_dc_offset(samples: &mut [f32]) {
        if samples.is_empty() {
            return;
        }
        let mean: f32 = samples.iter().sum::<f32>() / samples.len() as f32;
        for sample in samples.iter_mut() {
            *sample -= mean;
        }
    }

    /// Normalize audio to use full dynamic range (-1.0 to 1.0)
    /// Uses peak normalization to ensure consistent amplitude for VAD
    fn normalize_peak(samples: &mut [f32]) {
        if samples.is_empty() {
            return;
        }

        // Find the peak (maximum absolute value)
        let peak = samples
            .iter()
            .map(|s| s.abs())
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(1.0);

        // Only normalize if peak is significant (avoid amplifying pure noise)
        if peak > 0.001 {
            // Normalize to 95% of full scale to avoid potential clipping
            let factor = 0.95 / peak;
            for sample in samples.iter_mut() {
                *sample *= factor;
            }
            println!("Audio normalized: peak={:.4} -> normalized with factor={:.2}", peak, factor);
        } else {
            println!("Audio peak too low ({:.6}), skipping normalization (likely silence)", peak);
        }
    }

    /// High-quality resampling using Rubato FFT
    fn resample_with_rubato(&self, input: &[f32]) -> Result<Vec<f32>, String> {
        if let Some(ref resampler_cell) = self.resampler {
            let mut resampler = resampler_cell.borrow_mut();
            let chunk_size = resampler.input_frames_next();
            let mut output = Vec::new();

            // Process audio in chunks
            let mut pos = 0;
            while pos < input.len() {
                let end = (pos + chunk_size).min(input.len());
                let chunk = &input[pos..end];

                // Rubato requires Vec<Vec<f32>> format (one vec per channel)
                let mut chunk_vec = vec![chunk.to_vec()];

                // Handle partial chunks by padding with zeros if needed
                if chunk.len() < chunk_size {
                    chunk_vec[0].resize(chunk_size, 0.0);
                }

                let resampled = resampler
                    .process(&chunk_vec, None)
                    .map_err(|e| format!("Resample error: {}", e))?;

                output.extend_from_slice(&resampled[0]);
                pos = end;
            }

            Ok(output)
        } else {
            // Fallback to simple copy if no resampler
            Ok(input.to_vec())
        }
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