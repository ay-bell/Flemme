// Whisper-rs transcription engine
use super::{TranscriptionEngine, TranscriptionResult};
use std::path::Path;
use whisper_rs::{WhisperContext, WhisperContextParameters, FullParams, SamplingStrategy};

/// Whisper-rs based transcription engine
pub struct WhisperEngine {
    context: Option<WhisperContext>,
    model_loaded: bool,
}

impl WhisperEngine {
    /// Create a new Whisper engine instance
    pub fn new() -> Self {
        Self {
            context: None,
            model_loaded: false,
        }
    }
}

impl TranscriptionEngine for WhisperEngine {
    fn load_model<P: AsRef<Path>>(&mut self, model_path: P) -> Result<(), String> {
        let path = model_path.as_ref();

        if !path.exists() {
            return Err(format!("Model file not found: {:?}", path));
        }

        println!("=== WHISPER ENGINE INITIALIZATION ===");
        println!("Model path: {:?}", path);
        println!("Build mode: {}", if cfg!(debug_assertions) { "DEBUG" } else { "RELEASE" });

        // GPU acceleration detection
        #[cfg(feature = "cuda")]
        println!("GPU Acceleration: CUDA enabled (will auto-detect NVIDIA GPU at runtime)");
        #[cfg(not(feature = "cuda"))]
        println!("GPU Acceleration: Disabled (CPU only)");

        // Load the Whisper model
        let ctx = WhisperContext::new_with_params(
            path.to_str().ok_or_else(|| "Invalid path encoding".to_string())?,
            WhisperContextParameters::default()
        )
        .map_err(|e| format!("Failed to load Whisper model: {:?}", e))?;

        self.context = Some(ctx);
        self.model_loaded = true;

        println!("Whisper model loaded successfully");
        Ok(())
    }

    fn is_loaded(&self) -> bool {
        self.model_loaded
    }

    fn transcribe_with_prompt(&mut self, audio_data: &[f32], language: Option<String>, custom_words: Option<&[String]>) -> Result<TranscriptionResult, String> {
        let total_start = std::time::Instant::now();

        // Validate input
        if audio_data.is_empty() {
            return Err("Audio data is empty".to_string());
        }

        let duration_secs = audio_data.len() as f32 / 16000.0;
        println!("Transcribing {} samples ({:.2}s) with Whisper...", audio_data.len(), duration_secs);

        let context = self.context.as_ref()
            .ok_or_else(|| "Whisper model not loaded".to_string())?;

        // Create transcription parameters
        let params_start = std::time::Instant::now();
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        // Optimize thread count based on available CPU cores
        let num_cores = num_cpus::get();
        let optimal_threads = match num_cores {
            1..=4 => num_cores as i32,              // Use all on small CPUs
            5..=8 => ((num_cores * 3) / 4) as i32,  // 75% on medium CPUs
            _ => (num_cores / 2).max(8) as i32,     // 50% on large CPUs, min 8
        };
        params.set_n_threads(optimal_threads);
        println!("Using {} threads for transcription (CPU cores: {})", optimal_threads, num_cores);

        // Set language if provided
        if let Some(ref lang) = language {
            params.set_language(Some(lang.as_str()));
            params.set_translate(false);
        }

        // Set initial prompt with custom words for contextual biasing
        if let Some(words) = custom_words {
            if !words.is_empty() {
                let prompt = words.join(", ");
                params.set_initial_prompt(&prompt);
                println!("Using contextual biasing with {} custom words: {}", words.len(), prompt);
            }
        }

        // Disable printing and other output
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        println!("[TIMING] Whisper - params setup: {:.0}ms", params_start.elapsed().as_millis());

        // Create a new state for transcription
        let state_start = std::time::Instant::now();
        let mut state = context.create_state()
            .map_err(|e| format!("Failed to create Whisper state: {:?}", e))?;
        println!("[TIMING] Whisper - create state: {:.0}ms", state_start.elapsed().as_millis());

        // Run transcription
        let inference_start = std::time::Instant::now();
        state.full(params, audio_data)
            .map_err(|e| format!("Whisper transcription failed: {:?}", e))?;
        println!("[TIMING] Whisper - inference (state.full): {:.0}ms", inference_start.elapsed().as_millis());

        // Extract transcribed text
        let extraction_start = std::time::Instant::now();
        let num_segments = state.full_n_segments();

        let mut text = String::new();
        for i in 0..num_segments {
            if let Some(segment) = state.get_segment(i) {
                if let Ok(segment_text) = segment.to_str_lossy() {
                    text.push_str(&segment_text);
                }
            }
        }
        println!("[TIMING] Whisper - text extraction: {:.0}ms", extraction_start.elapsed().as_millis());

        println!("[TIMING] Whisper TOTAL: {:.0}ms", total_start.elapsed().as_millis());
        println!("Whisper transcription completed");

        Ok(TranscriptionResult {
            text,
            language,
        })
    }

    fn engine_name(&self) -> &str {
        "Whisper-rs"
    }
}

impl Default for WhisperEngine {
    fn default() -> Self {
        Self::new()
    }
}
