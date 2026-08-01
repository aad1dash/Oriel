use std::{error::Error, fmt};

use crate::{
    evidence::{
        CaptionProvenance, CompiledSource, Coverage, Evidence, SourceMetadata, TranscriptProvenance,
    },
    source::{SourceError, canonicalise_source},
};

const FIXTURE_SCHEMA_VERSION: &str = "1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FixtureError {
    InvalidLine { line: usize },
    UnknownField { line: usize, field: String },
    DuplicateField { line: usize, field: String },
    MissingField(&'static str),
    UnsupportedSchema(String),
    InvalidNumber { line: usize, field: &'static str },
    InvalidCaptionProvenance(String),
    InvalidSource(SourceError),
    InvalidCue { line: usize, reason: &'static str },
    NoEvidence,
}

impl fmt::Display for FixtureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLine { line } => write!(formatter, "fixture line {line} is malformed"),
            Self::UnknownField { line, field } => {
                write!(formatter, "fixture line {line} has unknown field '{field}'")
            }
            Self::DuplicateField { line, field } => {
                write!(formatter, "fixture line {line} repeats field '{field}'")
            }
            Self::MissingField(field) => write!(formatter, "fixture is missing '{field}'"),
            Self::UnsupportedSchema(version) => {
                write!(
                    formatter,
                    "fixture schema version '{version}' is unsupported"
                )
            }
            Self::InvalidNumber { line, field } => {
                write!(formatter, "fixture line {line} has an invalid {field}")
            }
            Self::InvalidCaptionProvenance(value) => {
                write!(formatter, "caption provenance '{value}' is unsupported")
            }
            Self::InvalidSource(error) => write!(formatter, "fixture source is invalid: {error}"),
            Self::InvalidCue { line, reason } => {
                write!(formatter, "fixture cue on line {line} {reason}")
            }
            Self::NoEvidence => formatter.write_str("fixture contains no caption evidence"),
        }
    }
}

impl Error for FixtureError {}

#[derive(Debug)]
struct RawCue {
    line: usize,
    start_ms: u64,
    end_ms: u64,
    text: String,
}

#[derive(Default)]
struct RawFixture {
    schema_version: Option<String>,
    source_url: Option<String>,
    title: Option<String>,
    creator: Option<String>,
    duration_ms: Option<u64>,
    language: Option<String>,
    caption_provenance: Option<String>,
    cues: Vec<RawCue>,
}

/// Compiles a deterministic tab-separated caption fixture into source evidence.
///
/// # Errors
///
/// Returns [`FixtureError`] when a required field is absent or any external
/// value fails schema, source or timestamp validation.
pub fn compile_fixture(input: &str) -> Result<CompiledSource, FixtureError> {
    let mut raw = RawFixture::default();

    for (line_index, line) in input.lines().enumerate() {
        let line_number = line_index + 1;
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let mut fields = line.splitn(4, '\t');
        let key = fields.next().unwrap_or_default();
        match key {
            "schema_version" => {
                set_text_field(&mut raw.schema_version, fields.next(), line_number, key)?;
            }
            "source_url" => {
                set_text_field(&mut raw.source_url, fields.next(), line_number, key)?;
            }
            "title" => set_text_field(&mut raw.title, fields.next(), line_number, key)?,
            "creator" => set_text_field(&mut raw.creator, fields.next(), line_number, key)?,
            "duration_ms" => {
                set_number_field(&mut raw.duration_ms, fields.next(), line_number, "duration")?;
            }
            "language" => set_text_field(&mut raw.language, fields.next(), line_number, key)?,
            "caption_provenance" => {
                set_text_field(&mut raw.caption_provenance, fields.next(), line_number, key)?;
            }
            "cue" => raw.cues.push(parse_cue(fields, line_number)?),
            _ => {
                return Err(FixtureError::UnknownField {
                    line: line_number,
                    field: key.to_owned(),
                });
            }
        }
    }

    build_fixture(raw)
}

fn set_text_field(
    target: &mut Option<String>,
    value: Option<&str>,
    line: usize,
    field: &str,
) -> Result<(), FixtureError> {
    if target.is_some() {
        return Err(FixtureError::DuplicateField {
            line,
            field: field.to_owned(),
        });
    }
    let value = value.ok_or(FixtureError::InvalidLine { line })?;
    if value.is_empty() {
        return Err(FixtureError::InvalidLine { line });
    }
    *target = Some(value.to_owned());
    Ok(())
}

fn set_number_field(
    target: &mut Option<u64>,
    value: Option<&str>,
    line: usize,
    field: &'static str,
) -> Result<(), FixtureError> {
    if target.is_some() {
        return Err(FixtureError::DuplicateField {
            line,
            field: field.to_owned(),
        });
    }
    let value = value
        .ok_or(FixtureError::InvalidLine { line })?
        .parse()
        .map_err(|_| FixtureError::InvalidNumber { line, field })?;
    *target = Some(value);
    Ok(())
}

fn parse_cue<'a>(
    mut fields: impl Iterator<Item = &'a str>,
    line: usize,
) -> Result<RawCue, FixtureError> {
    let start_ms = fields
        .next()
        .ok_or(FixtureError::InvalidLine { line })?
        .parse()
        .map_err(|_| FixtureError::InvalidNumber {
            line,
            field: "cue start timestamp",
        })?;
    let end_ms = fields
        .next()
        .ok_or(FixtureError::InvalidLine { line })?
        .parse()
        .map_err(|_| FixtureError::InvalidNumber {
            line,
            field: "cue end timestamp",
        })?;
    let text = fields
        .next()
        .filter(|value| !value.is_empty())
        .ok_or(FixtureError::InvalidCue {
            line,
            reason: "has no text",
        })?
        .to_owned();

    Ok(RawCue {
        line,
        start_ms,
        end_ms,
        text,
    })
}

fn build_fixture(raw: RawFixture) -> Result<CompiledSource, FixtureError> {
    let schema_version = required(raw.schema_version, "schema_version")?;
    if schema_version != FIXTURE_SCHEMA_VERSION {
        return Err(FixtureError::UnsupportedSchema(schema_version));
    }
    let source_url = required(raw.source_url, "source_url")?;
    let source = canonicalise_source(&source_url).map_err(FixtureError::InvalidSource)?;
    let title = required(raw.title, "title")?;
    let creator = required(raw.creator, "creator")?;
    let duration_ms = raw
        .duration_ms
        .ok_or(FixtureError::MissingField("duration_ms"))?;
    if duration_ms == 0 {
        return Err(FixtureError::InvalidNumber {
            line: 0,
            field: "duration",
        });
    }
    let language = required(raw.language, "language")?;
    let caption_value = required(raw.caption_provenance, "caption_provenance")?;
    let captions = parse_caption_provenance(&caption_value)?;
    if raw.cues.is_empty() {
        return Err(FixtureError::NoEvidence);
    }

    let transcript = TranscriptProvenance { language, captions };
    let mut previous_start = None;
    let mut evidence = Vec::with_capacity(raw.cues.len());
    for (index, cue) in raw.cues.into_iter().enumerate() {
        validate_cue(&cue, previous_start, duration_ms)?;
        previous_start = Some(cue.start_ms);
        evidence.push(Evidence {
            id: format!("{}:{index}", source.source_id),
            start_ms: cue.start_ms,
            end_ms: cue.end_ms,
            text: cue.text,
            transcript: transcript.clone(),
        });
    }

    let transcript_start_ms = evidence.first().map_or(0, |cue| cue.start_ms);
    let transcript_end_ms = evidence.last().map_or(0, |cue| cue.end_ms);
    let source_version = fixture_version(&source.source_id, &evidence);

    Ok(CompiledSource {
        source,
        source_version,
        metadata: SourceMetadata {
            title,
            creator,
            duration_ms,
        },
        evidence,
        coverage: Coverage {
            metadata: true,
            transcript_start_ms,
            transcript_end_ms,
            transcript_complete: transcript_start_ms == 0 && transcript_end_ms == duration_ms,
            visuals_processed: false,
        },
    })
}

fn required(value: Option<String>, field: &'static str) -> Result<String, FixtureError> {
    value.ok_or(FixtureError::MissingField(field))
}

fn parse_caption_provenance(value: &str) -> Result<CaptionProvenance, FixtureError> {
    match value {
        "manual" => Ok(CaptionProvenance::Manual),
        "generated" => Ok(CaptionProvenance::Generated),
        "local_transcription" => Ok(CaptionProvenance::LocalTranscription),
        _ => Err(FixtureError::InvalidCaptionProvenance(value.to_owned())),
    }
}

fn validate_cue(
    cue: &RawCue,
    previous_start: Option<u64>,
    duration_ms: u64,
) -> Result<(), FixtureError> {
    if cue.start_ms >= cue.end_ms {
        return Err(FixtureError::InvalidCue {
            line: cue.line,
            reason: "must end after it starts",
        });
    }
    if cue.end_ms > duration_ms {
        return Err(FixtureError::InvalidCue {
            line: cue.line,
            reason: "ends after the source duration",
        });
    }
    if previous_start.is_some_and(|start| cue.start_ms < start) {
        return Err(FixtureError::InvalidCue {
            line: cue.line,
            reason: "is out of source order",
        });
    }
    Ok(())
}

fn fixture_version(source_id: &str, evidence: &[Evidence]) -> String {
    // FNV-1a is sufficient for deterministic fixtures, but not the future live-source cache.
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in source_id.bytes().chain(evidence.iter().flat_map(|cue| {
        cue.start_ms
            .to_le_bytes()
            .into_iter()
            .chain(cue.end_ms.to_le_bytes())
            .chain(cue.text.bytes())
    })) {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("fixture-fnv1a64:{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::{FixtureError, compile_fixture};
    use crate::evidence::CaptionProvenance;

    const VALID_FIXTURE: &str = "schema_version\t1\n\
source_url\thttps://youtu.be/dQw4w9WgXcQ\n\
title\tA useful source\n\
creator\tOriel fixture\n\
duration_ms\t10000\n\
language\ten\n\
caption_provenance\tmanual\n\
cue\t0\t5000\tEvidence must keep its timestamp.\n\
cue\t5000\t10000\tInterpretation must remain separate.\n";

    #[test]
    fn compiles_timestamped_evidence_and_coverage() {
        let compiled = compile_fixture(VALID_FIXTURE).expect("fixture should compile");

        assert_eq!(compiled.source.source_id, "dQw4w9WgXcQ");
        assert_eq!(compiled.evidence.len(), 2);
        assert_eq!(compiled.evidence[0].start_ms, 0);
        assert_eq!(compiled.evidence[0].end_ms, 5_000);
        assert_eq!(
            compiled.evidence[0].transcript.captions,
            CaptionProvenance::Manual
        );
        assert!(compiled.coverage.transcript_complete);
        assert!(!compiled.coverage.visuals_processed);
        assert!(compiled.source_version.starts_with("fixture-fnv1a64:"));
    }

    #[test]
    fn rejects_out_of_order_or_out_of_range_cues() {
        let out_of_order = VALID_FIXTURE
            .replace("cue\t0\t5000", "cue\t4000\t5000")
            .replace("cue\t5000\t10000", "cue\t3000\t10000");
        assert!(matches!(
            compile_fixture(&out_of_order),
            Err(FixtureError::InvalidCue {
                reason: "is out of source order",
                ..
            })
        ));

        let out_of_range = VALID_FIXTURE.replace("cue\t5000\t10000", "cue\t5000\t10001");
        assert!(matches!(
            compile_fixture(&out_of_range),
            Err(FixtureError::InvalidCue {
                reason: "ends after the source duration",
                ..
            })
        ));
    }

    #[test]
    fn rejects_unknown_fields_and_empty_evidence() {
        let unknown = VALID_FIXTURE.replace("creator\t", "channel\t");
        assert!(matches!(
            compile_fixture(&unknown),
            Err(FixtureError::UnknownField { .. })
        ));

        let empty = VALID_FIXTURE
            .lines()
            .filter(|line| !line.starts_with("cue\t"))
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(compile_fixture(&empty), Err(FixtureError::NoEvidence));
    }
}
