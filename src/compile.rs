use std::{error::Error, fmt};

use sha2::{Digest, Sha256};

use crate::{
    evidence::{
        AcquisitionProvenance, CompiledSource, Coverage, Evidence, SourceMetadata,
        TranscriptProvenance,
    },
    source::{CanonicalSource, SourceProvider},
};

const SOURCE_VERSION_DOMAIN: &str = "oriel.compiled-source.v1";
const MAX_CUE_COUNT: usize = 1_000_000;
const MAX_CUE_TEXT_BYTES: usize = 1_000_000;
const HEX: &[u8; 16] = b"0123456789abcdef";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcquiredCue {
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcquiredSource {
    pub source: CanonicalSource,
    pub metadata: SourceMetadata,
    pub transcript: TranscriptProvenance,
    pub cues: Vec<AcquiredCue>,
    pub coverage: Coverage,
    pub acquisition: AcquisitionProvenance,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompileError {
    EmptyMetadata(&'static str),
    InvalidDuration,
    InvalidCoverage,
    NoEvidence,
    TooManyCues,
    InvalidCue { index: usize, reason: &'static str },
    InconsistentCompiledSource,
}

impl fmt::Display for CompileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyMetadata(field) => write!(formatter, "source metadata '{field}' is empty"),
            Self::InvalidDuration => {
                formatter.write_str("source duration must be greater than zero")
            }
            Self::InvalidCoverage => formatter.write_str("source coverage is invalid"),
            Self::NoEvidence => formatter.write_str("source contains no transcript evidence"),
            Self::TooManyCues => formatter.write_str("source contains too many transcript cues"),
            Self::InvalidCue { index, reason } => {
                write!(formatter, "source cue {index} {reason}")
            }
            Self::InconsistentCompiledSource => {
                formatter.write_str("compiled source fields are internally inconsistent")
            }
        }
    }
}

impl Error for CompileError {}

/// Validates acquired evidence and compiles it into Oriel's shared source model.
///
/// # Errors
///
/// Returns [`CompileError`] when metadata, coverage or any evidence cue violates
/// the engine contract.
pub fn compile_source(acquired: AcquiredSource) -> Result<CompiledSource, CompileError> {
    validate_metadata(&acquired)?;
    validate_coverage(&acquired)?;
    validate_cues(&acquired.cues)?;

    let source_version = source_version(&acquired);
    let evidence = acquired
        .cues
        .into_iter()
        .enumerate()
        .map(|(index, cue)| Evidence {
            id: format!("{}:{index}", acquired.source.source_id),
            start_ms: cue.start_ms,
            end_ms: cue.end_ms,
            text: cue.text,
            transcript: acquired.transcript.clone(),
        })
        .collect();

    Ok(CompiledSource {
        source: acquired.source,
        source_version,
        metadata: acquired.metadata,
        evidence,
        coverage: acquired.coverage,
        acquisition: acquired.acquisition,
    })
}

/// Revalidates a compiled source loaded from an untrusted storage boundary.
///
/// # Errors
///
/// Returns [`CompileError`] when evidence identifiers, transcript provenance,
/// validation rules or the semantic source version do not match the stored data.
pub fn verify_compiled_source(compiled: &CompiledSource) -> Result<(), CompileError> {
    let transcript = compiled
        .evidence
        .first()
        .map(|evidence| evidence.transcript.clone())
        .ok_or(CompileError::NoEvidence)?;
    let mut cues = Vec::with_capacity(compiled.evidence.len());
    for (index, evidence) in compiled.evidence.iter().enumerate() {
        if evidence.id != format!("{}:{index}", compiled.source.source_id)
            || evidence.transcript != transcript
        {
            return Err(CompileError::InconsistentCompiledSource);
        }
        cues.push(AcquiredCue {
            start_ms: evidence.start_ms,
            end_ms: evidence.end_ms,
            text: evidence.text.clone(),
        });
    }

    let rebuilt = compile_source(AcquiredSource {
        source: compiled.source.clone(),
        metadata: compiled.metadata.clone(),
        transcript,
        cues,
        coverage: compiled.coverage.clone(),
        acquisition: compiled.acquisition.clone(),
    })?;
    if rebuilt == *compiled {
        Ok(())
    } else {
        Err(CompileError::InconsistentCompiledSource)
    }
}

fn validate_metadata(acquired: &AcquiredSource) -> Result<(), CompileError> {
    for (field, value) in [
        ("title", acquired.metadata.title.as_str()),
        ("creator", acquired.metadata.creator.as_str()),
        ("language", acquired.transcript.language.as_str()),
        ("adapter", acquired.acquisition.adapter.as_str()),
        ("source_format", acquired.acquisition.source_format.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(CompileError::EmptyMetadata(field));
        }
    }
    if acquired.metadata.duration_ms == 0 {
        return Err(CompileError::InvalidDuration);
    }
    Ok(())
}

fn validate_coverage(acquired: &AcquiredSource) -> Result<(), CompileError> {
    let coverage = &acquired.coverage;
    if !coverage.metadata
        || coverage.transcript_start_ms > coverage.transcript_end_ms
        || coverage.transcript_end_ms > acquired.metadata.duration_ms
    {
        return Err(CompileError::InvalidCoverage);
    }
    Ok(())
}

fn validate_cues(cues: &[AcquiredCue]) -> Result<(), CompileError> {
    if cues.is_empty() {
        return Err(CompileError::NoEvidence);
    }
    if cues.len() > MAX_CUE_COUNT {
        return Err(CompileError::TooManyCues);
    }

    let mut previous_start = None;
    for (index, cue) in cues.iter().enumerate() {
        let reason = if cue.text.trim().is_empty() {
            Some("has no text")
        } else if cue.text.len() > MAX_CUE_TEXT_BYTES {
            Some("contains too much text")
        } else if cue.start_ms >= cue.end_ms {
            Some("must end after it starts")
        } else if previous_start.is_some_and(|start| cue.start_ms < start) {
            Some("is out of source order")
        } else {
            None
        };
        if let Some(reason) = reason {
            return Err(CompileError::InvalidCue { index, reason });
        }
        previous_start = Some(cue.start_ms);
    }
    Ok(())
}

fn source_version(acquired: &AcquiredSource) -> String {
    let mut hash = Sha256::new();
    hash_text(&mut hash, SOURCE_VERSION_DOMAIN);
    hash_text(
        &mut hash,
        match acquired.source.provider {
            SourceProvider::YouTube => "youtube",
        },
    );
    hash_text(&mut hash, &acquired.source.source_id);
    hash_text(&mut hash, &acquired.metadata.title);
    hash_text(&mut hash, &acquired.metadata.creator);
    hash.update(acquired.metadata.duration_ms.to_be_bytes());
    hash_text(&mut hash, &acquired.transcript.language);
    hash_text(
        &mut hash,
        match acquired.transcript.captions {
            crate::evidence::CaptionProvenance::Manual => "manual",
            crate::evidence::CaptionProvenance::Generated => "generated",
            crate::evidence::CaptionProvenance::LocalTranscription => "local_transcription",
        },
    );
    hash.update(acquired.coverage.transcript_start_ms.to_be_bytes());
    hash.update(acquired.coverage.transcript_end_ms.to_be_bytes());
    hash.update([u8::from(acquired.coverage.transcript_complete)]);
    hash.update([u8::from(acquired.coverage.visuals_processed)]);
    for cue in &acquired.cues {
        hash.update(cue.start_ms.to_be_bytes());
        hash.update(cue.end_ms.to_be_bytes());
        hash_text(&mut hash, &cue.text);
    }

    let digest = hash.finalize();
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    format!("source-v1:sha256:{encoded}")
}

fn hash_text(hash: &mut Sha256, value: &str) {
    hash.update(u64::try_from(value.len()).unwrap_or(u64::MAX).to_be_bytes());
    hash.update(value.as_bytes());
}

#[cfg(test)]
mod tests {
    use crate::{
        evidence::{
            AcquisitionProvenance, CaptionProvenance, Coverage, SourceMetadata,
            TranscriptProvenance,
        },
        source::canonicalise_source,
    };

    use super::{
        AcquiredCue, AcquiredSource, CompileError, compile_source, verify_compiled_source,
    };

    fn acquired_source() -> AcquiredSource {
        AcquiredSource {
            source: canonicalise_source("https://youtu.be/dQw4w9WgXcQ")
                .expect("source should be valid"),
            metadata: SourceMetadata {
                title: "Evidence source".to_owned(),
                creator: "Oriel".to_owned(),
                duration_ms: 10_000,
            },
            transcript: TranscriptProvenance {
                language: "en".to_owned(),
                captions: CaptionProvenance::Manual,
            },
            cues: vec![AcquiredCue {
                start_ms: 0,
                end_ms: 10_000,
                text: "A timestamped cue".to_owned(),
            }],
            coverage: Coverage {
                metadata: true,
                transcript_start_ms: 0,
                transcript_end_ms: 10_000,
                transcript_complete: true,
                visuals_processed: false,
            },
            acquisition: AcquisitionProvenance {
                adapter: "fixture".to_owned(),
                source_format: "fixture_v1".to_owned(),
                tool: None,
            },
        }
    }

    #[test]
    fn version_is_stable_and_semantically_framed() {
        let first = compile_source(acquired_source()).expect("source should compile");
        let second = compile_source(acquired_source()).expect("source should compile");
        assert_eq!(first.source_version, second.source_version);
        assert!(first.source_version.starts_with("source-v1:sha256:"));
        assert_eq!(first.source_version.len(), 81);

        let mut changed = acquired_source();
        changed.cues[0].text.push('!');
        let changed = compile_source(changed).expect("changed source should compile");
        assert_ne!(first.source_version, changed.source_version);
    }

    #[test]
    fn acquisition_tool_version_does_not_invalidate_evidence() {
        let first = compile_source(acquired_source()).expect("source should compile");
        let mut changed = acquired_source();
        changed.acquisition.tool = Some(crate::evidence::ToolProvenance {
            name: "yt-dlp".to_owned(),
            version: "future".to_owned(),
        });
        let changed = compile_source(changed).expect("source should compile");
        assert_eq!(first.source_version, changed.source_version);
    }

    #[test]
    fn rejects_invalid_coverage_and_cues() {
        let mut invalid = acquired_source();
        invalid.coverage.transcript_end_ms = 10_001;
        assert_eq!(compile_source(invalid), Err(CompileError::InvalidCoverage));

        let mut invalid = acquired_source();
        invalid.cues[0].end_ms = 0;
        assert!(matches!(
            compile_source(invalid),
            Err(CompileError::InvalidCue { .. })
        ));
    }

    #[test]
    fn rejects_tampered_compiled_sources() {
        let compiled = compile_source(acquired_source()).expect("source should compile");
        verify_compiled_source(&compiled).expect("compiled source should verify");

        let mut tampered = compiled;
        tampered.evidence[0].text.push_str(" changed");
        assert_eq!(
            verify_compiled_source(&tampered),
            Err(CompileError::InconsistentCompiledSource)
        );
    }
}
