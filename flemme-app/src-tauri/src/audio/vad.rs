// VoiceActivityDetector - detects voice activity using Silero VAD ONNX model
use ndarray::{Array0, Array2, Array3};
use ort::session::builder::GraphOptimizationLevel;
use ort::session::Session;
use ort::value::TensorRef;
use std::path::Path;

pub struct VoiceActivityDetector {
    session: Session,
    threshold: f32,
    // Internal state for the model (batch=2, 1, hidden=128)
    state: Array3<f32>,
    // Sample rate (must be 16000 for Whisper compatibility) - scalar int64
    sr: i64,
}

impl VoiceActivityDetector {
    /// Create a new VAD instance from ONNX model file
    ///
    /// # Arguments
    /// * `model_path` - Path to silero_vad.onnx model
    /// * `threshold` - Detection threshold (0.0 to 1.0, recommended: 0.5)
    pub fn new<P: AsRef<Path>>(model_path: P, threshold: f32) -> Result<Self, String> {
        // Validate threshold
        if !(0.0..=1.0).contains(&threshold) {
            return Err(format!(
                "Invalid threshold: {}. Must be between 0.0 and 1.0",
                threshold
            ));
        }

        // Load ONNX model with ort 2.0 API
        let session = Session::builder()
            .map_err(|e| format!("Failed to create session builder: {}", e))?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| format!("Failed to set optimization level: {}", e))?
            .commit_from_file(model_path)
            .map_err(|e| format!("Failed to load ONNX model: {}", e))?;

        // Initialize internal state (required by Silero VAD model)
        // state is the combined LSTM state (batch=2, 1, hidden=128)
        let state = Array3::<f32>::zeros((2, 1, 128));
        let sr = 16000i64; // Sample rate as scalar int64

        println!("Silero VAD initialized: threshold={}", threshold);

        Ok(Self {
            session,
            threshold,
            state,
            sr,
        })
    }

    /// Create with default settings (16kHz, 0.3 threshold)
    pub fn new_default<P: AsRef<Path>>(model_path: P) -> Result<Self, String> {
        Self::new(model_path, 0.3)
    }

    /// Detect voice activity in an audio chunk
    ///
    /// # Arguments
    /// * `audio_data` - Audio samples as f32 (mono, normalized to -1.0 to 1.0)
    ///   Must be 512 or 1024 or 1536 samples (32ms, 64ms, or 96ms at 16kHz)
    ///
    /// # Returns
    /// * `f32` - Probability of speech (0.0 to 1.0)
    pub fn detect(&mut self, audio_data: &[f32]) -> Result<f32, String> {
        // Validate input size
        let valid_sizes = [512, 1024, 1536];
        if !valid_sizes.contains(&audio_data.len()) {
            return Err(format!(
                "Invalid audio chunk size: {}. Must be 512, 1024, or 1536 samples",
                audio_data.len()
            ));
        }

        // Prepare input tensor: (1, num_samples) for ort 2.0
        let input = Array2::from_shape_vec((1, audio_data.len()), audio_data.to_vec())
            .map_err(|e| format!("Failed to create input array: {}", e))?;

        // Create TensorRef from array views for ort 2.0
        let input_tensor = TensorRef::from_array_view(input.view())
            .map_err(|e| format!("Failed to create input tensor: {}", e))?;
        let state_tensor = TensorRef::from_array_view(self.state.view())
            .map_err(|e| format!("Failed to create state tensor: {}", e))?;

        // Create scalar tensor for sr (int64)
        let sr_scalar = Array0::from_elem((), self.sr);
        let sr_tensor = TensorRef::from_array_view(sr_scalar.view())
            .map_err(|e| format!("Failed to create sr tensor: {}", e))?;

        // Run inference - ort v2.0 API (Silero VAD expects: input, state, sr)
        let outputs = self
            .session
            .run(ort::inputs![input_tensor, state_tensor, sr_tensor])
            .map_err(|e| format!("Failed to run inference: {}", e))?;

        // Extract output probability (ort 2.0 API)
        let output = outputs[0]
            .try_extract_tensor::<f32>()
            .map_err(|e| format!("Failed to extract output: {}", e))?;
        let probability = output.1[0];

        // Update state for next prediction (ort 2.0 API)
        let new_state = outputs[1]
            .try_extract_tensor::<f32>()
            .map_err(|e| format!("Failed to extract new state: {}", e))?;

        // Convert back to fixed dimensions (2, 1, 128)
        self.state = Array3::from_shape_vec((2, 1, 128), new_state.1.to_vec())
            .map_err(|e| format!("Failed to convert state dimensions: {}", e))?;

        Ok(probability)
    }

    /// Check if audio contains speech above threshold
    pub fn is_speech(&mut self, audio_data: &[f32]) -> bool {
        match self.detect(audio_data) {
            Ok(probability) => probability > self.threshold,
            Err(e) => {
                eprintln!("VAD detection error: {}", e);
                false
            }
        }
    }

    /// Process audio and return only segments with speech
    ///
    /// # Arguments
    /// * `audio_data` - Full audio buffer
    /// * `chunk_size` - Size of chunks to analyze (512, 1024, or 1536 samples)
    ///
    /// # Returns
    /// * `Vec<f32>` - Audio with silence removed
    pub fn filter_silence(&mut self, audio_data: &[f32], chunk_size: usize) -> Vec<f32> {
        let mut filtered = Vec::new();

        // Reset state before processing new audio
        self.reset();

        // Process audio in chunks
        for chunk in audio_data.chunks(chunk_size) {
            // Handle incomplete chunks at the end by padding with zeros
            let chunk_to_process = if chunk.len() != chunk_size {
                let mut padded = vec![0.0f32; chunk_size];
                padded[..chunk.len()].copy_from_slice(chunk);
                padded
            } else {
                chunk.to_vec()
            };

            if self.is_speech(&chunk_to_process) {
                // Only add the original chunk length (without padding)
                filtered.extend_from_slice(chunk);
            }
        }

        filtered
    }

    /// Analyze full audio and return speech segments with timestamps
    ///
    /// # Arguments
    /// * `audio_data` - Full audio buffer
    /// * `chunk_size` - Size of chunks to analyze (512, 1024, or 1536)
    ///
    /// # Returns
    /// * `Vec<SpeechSegment>` - List of speech segments with start/end indices
    pub fn get_speech_segments(
        &mut self,
        audio_data: &[f32],
        chunk_size: usize,
    ) -> Vec<SpeechSegment> {
        let mut segments = Vec::new();
        let mut current_segment: Option<SpeechSegment> = None;

        // Reset state before processing
        self.reset();

        for (chunk_idx, chunk) in audio_data.chunks(chunk_size).enumerate() {
            // Skip incomplete chunks
            if chunk.len() != chunk_size {
                break;
            }

            let start_sample = chunk_idx * chunk_size;
            let is_speech = self.is_speech(chunk);

            match (&mut current_segment, is_speech) {
                // Start new segment
                (None, true) => {
                    current_segment = Some(SpeechSegment {
                        start: start_sample,
                        end: start_sample + chunk.len(),
                    });
                }
                // Continue segment
                (Some(seg), true) => {
                    seg.end = start_sample + chunk.len();
                }
                // End segment
                (Some(_), false) => {
                    if let Some(seg) = current_segment.take() {
                        segments.push(seg);
                    }
                }
                // No speech, no segment
                (None, false) => {}
            }
        }

        // Add last segment if exists
        if let Some(seg) = current_segment {
            segments.push(seg);
        }

        segments
    }

    /// Reset VAD internal state (call between different recordings)
    pub fn reset(&mut self) {
        self.state = Array3::<f32>::zeros((2, 1, 128));
    }

    /// Get current threshold
    pub fn threshold(&self) -> f32 {
        self.threshold
    }

    /// Set new threshold
    pub fn set_threshold(&mut self, threshold: f32) -> Result<(), String> {
        if !(0.0..=1.0).contains(&threshold) {
            return Err(format!("Invalid threshold: {}", threshold));
        }
        self.threshold = threshold;
        Ok(())
    }
}

/// Represents a segment of speech in an audio buffer
#[derive(Debug, Clone)]
pub struct SpeechSegment {
    pub start: usize, // Start sample index
    pub end: usize,   // End sample index
}

impl SpeechSegment {
    /// Get duration in seconds
    pub fn duration_seconds(&self, sample_rate: u32) -> f32 {
        (self.end - self.start) as f32 / sample_rate as f32
    }

    /// Extract audio data for this segment
    pub fn extract<'a>(&self, audio_data: &'a [f32]) -> &'a [f32] {
        let start = self.start.min(audio_data.len());
        let end = self.end.min(audio_data.len());
        &audio_data[start..end]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_speech_segment_duration() {
        let segment = SpeechSegment {
            start: 0,
            end: 16000,
        };
        assert_eq!(segment.duration_seconds(16000), 1.0);
    }

    #[test]
    fn test_invalid_threshold() {
        let result = VoiceActivityDetector::new("dummy.onnx", 1.5);
        assert!(result.is_err());
    }
}