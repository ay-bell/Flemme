// Transcription models - defines model types and metadata
// TODO: Implement model management

#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub name: String,
    pub size: u64,
    pub url: String,
}

#[derive(Debug, Clone)]
pub enum TranscriptionModel {
    Tiny,
    Base,
    Small,
    Medium,
    Large,
}

impl TranscriptionModel {
    pub fn get_info(&self) -> ModelInfo {
        todo!("Implement model info retrieval")
    }
}
