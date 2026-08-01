use crate::source::CanonicalSource;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptionProvenance {
    Manual,
    Generated,
    LocalTranscription,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SourceMetadata {
    pub title: String,
    pub creator: String,
    pub duration_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TranscriptProvenance {
    pub language: String,
    pub captions: CaptionProvenance,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Evidence {
    pub id: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
    pub transcript: TranscriptProvenance,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Coverage {
    pub metadata: bool,
    pub transcript_start_ms: u64,
    pub transcript_end_ms: u64,
    pub transcript_complete: bool,
    pub visuals_processed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ToolProvenance {
    pub name: String,
    pub version: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AcquisitionProvenance {
    pub adapter: String,
    pub source_format: String,
    pub tool: Option<ToolProvenance>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CompiledSource {
    pub source: CanonicalSource,
    pub source_version: String,
    pub metadata: SourceMetadata,
    pub evidence: Vec<Evidence>,
    pub coverage: Coverage,
    pub acquisition: AcquisitionProvenance,
}
