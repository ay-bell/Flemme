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
        // Create a state for this transcription
        let mut state = self.ctx.create_state()
            .map_err(|e| format!("Failed to create whisper state: {:?}", e))?;

        // Configure transcription parameters
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        // Set language to auto-detect or specify
        params.set_language(Some("auto"));
        params.set_translate(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        // Run transcription
        state.full(params, audio_data)
            .map_err(|e| format!("Transcription failed: {:?}", e))?;

        // Extract transcribed text
        let num_segments = state.full_n_segments()
            .map_err(|e| format!("Failed to get segments: {:?}", e))?;

        let mut result = String::new();
        for i in 0..num_segments {
            let segment = state.full_get_segment_text(i)
                .map_err(|e| format!("Failed to get segment text: {:?}", e))?;
            result.push_str(&segment);
        }

        Ok(result.trim().to_string())
    }

    /// Get sample rate required by whisper (always 16kHz)
    pub fn sample_rate() -> u32 {
        16000
    }
}