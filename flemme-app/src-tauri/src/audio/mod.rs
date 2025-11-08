// Audio module - handles audio recording and processing
pub mod recorder;
pub mod vad;

pub use recorder::AudioRecorder;
pub use vad::{VoiceActivityDetector, SpeechSegment};
