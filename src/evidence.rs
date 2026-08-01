use crate::source::CanonicalSource;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CaptionProvenance {
    Manual,
    Generated,
    LocalTranscription,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceMetadata {
    pub title: String,
    pub creator: String,
    pub duration_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TranscriptProvenance {
    pub language: String,
    pub captions: CaptionProvenance,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Evidence {
    pub id: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
    pub transcript: TranscriptProvenance,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Coverage {
    pub metadata: bool,
    pub transcript_start_ms: u64,
    pub transcript_end_ms: u64,
    pub transcript_complete: bool,
    pub visuals_processed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompiledSource {
    pub source: CanonicalSource,
    pub source_version: String,
    pub metadata: SourceMetadata,
    pub evidence: Vec<Evidence>,
    pub coverage: Coverage,
}
