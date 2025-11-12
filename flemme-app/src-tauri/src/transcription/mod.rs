// Transcription module - handles speech-to-text with Whisper
pub mod whisper;
pub mod engine; // Old engine.rs for backward compatibility

pub use whisper::WhisperEngine;

use std::path::Path;

/// Result of a transcription operation
#[derive(Debug, Clone)]
pub struct TranscriptionResult {
    pub text: String,
    pub language: Option<String>,
}

/// Trait for speech-to-text transcription engines
pub trait TranscriptionEngine {
    /// Load the model from the given path
    fn load_model<P: AsRef<Path>>(&mut self, model_path: P) -> Result<(), String>;

    /// Check if a model is currently loaded
    fn is_loaded(&self) -> bool;

    /// Transcribe audio data (16kHz mono f32 samples, normalized -1.0 to 1.0)
    /// custom_words: Optional list of custom words/phrases for contextual biasing via initial prompt
    fn transcribe(&mut self, audio_data: &[f32], language: Option<String>) -> Result<TranscriptionResult, String> {
        self.transcribe_with_prompt(audio_data, language, None)
    }

    /// Transcribe with custom words for contextual biasing
    fn transcribe_with_prompt(&mut self, audio_data: &[f32], language: Option<String>, custom_words: Option<&[String]>) -> Result<TranscriptionResult, String>;

    /// Get the name of this engine
    fn engine_name(&self) -> &str;
}
