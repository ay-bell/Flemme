// TranscriptionEngine - handles speech-to-text conversion using whisper.cpp
use whisper_rs::{WhisperContext, WhisperContextParameters, FullParams, SamplingStrategy};
use std::path::Path;

pub struct TranscriptionEngine {
    ctx: WhisperContext,
}

impl TranscriptionEngine {
    /// Create a new transcription engine with the specified model
    pub fn new(model_path: &str) -> Result<Self, String> {
        let path = Path::new(model_path);

        if !path.exists() {
            return Err(format!("Model file not found: {}", model_path));
        }

        println!("=== WHISPER ENGINE INITIALIZATION ===");
        println!("Model path: {}", model_path);
        println!("Build mode: {}", if cfg!(debug_assertions) { "DEBUG" } else { "RELEASE" });
        println!("whisper-rs version: {}", env!("CARGO_PKG_VERSION"));

        // Use default parameters (CPU-only, no GPU)
        // GPU support requires OpenCL/Metal which may not be available
        let ctx = WhisperContext::new_with_params(
            model_path,
            WhisperContextParameters::default(),
        )
        .map_err(|e| format!("Failed to load whisper model: {:?}", e))?;

        Ok(Self { ctx })
    }

    /// Transcribe audio samples to text
    /// Audio must be mono 16kHz f32 samples
    pub fn transcribe(&self, audio_data: &[f32]) -> Result<String, String> {
        // Validate input
        if audio_data.is_empty() {
            return Err("Audio data is empty".to_string());
        }

        let duration_secs = audio_data.len() as f32 / 16000.0;
        println!("Transcribing {} samples ({:.2}s)...", audio_data.len(), duration_secs);

        // Validate and normalize audio data
        // Check for invalid values (NaN, Inf) and normalize to [-1.0, 1.0]
        let mut normalized_audio = Vec::with_capacity(audio_data.len());
        let mut max_abs = 0.0f32;

        for &sample in audio_data {
            if !sample.is_finite() {
                return Err("Audio contains invalid values (NaN or Inf)".to_string());
            }
            max_abs = max_abs.max(sample.abs());
        }

        // Normalize if needed (but keep silence as silence)
        if max_abs > 1.0 {
            println!("Normalizing audio (max value: {:.2})", max_abs);
            for &sample in audio_data {
                normalized_audio.push(sample / max_abs);
            }
        } else if max_abs < 0.001 {
            println!("Warning: Audio is very quiet (max value: {:.6})", max_abs);
            normalized_audio.extend_from_slice(audio_data);
        } else {
            normalized_audio.extend_from_slice(audio_data);
        }

        let audio_ref = &normalized_audio[..];

        // Create a state for this transcription
        let mut state = self.ctx.create_state()
            .map_err(|e| format!("Failed to create whisper state: {:?}", e))?;

        // Configure transcription parameters for maximum performance
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        // Set language (fr for French, en for English, or None for auto-detect)
        // Using "fr" as default, can be made configurable later
        params.set_language(Some("fr"));
        params.set_translate(false);
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        // Performance optimizations
        let n_threads = num_cpus::get() as i32;
        params.set_n_threads(n_threads); // Use all CPU cores
        // NOTE: set_speed_up() method removed in v0.15
        // NOTE: set_single_segment(true) causes 3x slowdown in release mode!

        println!("=== TRANSCRIPTION PARAMETERS ===");
        println!("Threads: {}", n_threads);
        println!("Language: fr");
        println!("Strategy: Greedy {{ best_of: 1 }}");
        println!("Audio duration: {:.2}s ({} samples)", duration_secs, audio_data.len());
        println!("Running Whisper inference...");

        // Run transcription with normalized audio
        state.full(params, audio_ref)
            .map_err(|e| format!("Transcription failed: {:?}", e))?;

        println!("Whisper inference completed, extracting text...");

        // Extract transcribed text using iterator (whisper-rs 0.15+ API)
        let mut result = String::new();
        for segment in state.as_iter() {
            // segment implements Display which gives us the text
            result.push_str(&format!("{}", segment));
        }

        Ok(result.trim().to_string())
    }

    /// Get sample rate required by whisper (always 16kHz)
    pub fn sample_rate() -> u32 {
        16000
    }
}