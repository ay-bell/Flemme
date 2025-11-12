// TranscriptionEngine - DEPRECATED: Use WhisperEngine directly
// This file is kept for backward compatibility with existing code in lib.rs

use super::whisper::WhisperEngine;

/// DEPRECATED: Legacy wrapper around WhisperEngine for backward compatibility
/// New code should use WhisperEngine directly via the TranscriptionEngine trait
pub struct TranscriptionEngine {
    inner: WhisperEngine,
}

impl TranscriptionEngine {
    /// Create a new transcription engine with the specified model
    pub fn new(model_path: &str) -> Result<Self, String> {
        use super::TranscriptionEngine as TranscriptionTrait;

        let mut inner = WhisperEngine::new();
        inner.load_model(model_path)?;

        Ok(Self { inner })
    }

    /// Transcribe audio samples to text
    /// Audio must be mono 16kHz f32 samples
    /// language: ISO 639-1 code (e.g., "fr", "en", "es") or None for auto-detect
    /// custom_words: Optional list of custom words/phrases for contextual biasing via initial prompt
    pub fn transcribe(&mut self, audio_data: &[f32], language: Option<&str>, custom_words: Option<&[String]>) -> Result<String, String> {
        use super::TranscriptionEngine as TranscriptionTrait;

        let result = self.inner.transcribe_with_prompt(
            audio_data,
            language.map(|s| s.to_string()),
            custom_words
        )?;

        Ok(result.text)
    }

    /// Get sample rate required by whisper (always 16kHz)
    pub fn sample_rate() -> u32 {
        16000
    }
}