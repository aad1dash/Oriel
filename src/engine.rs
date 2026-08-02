use std::{error::Error, fmt, path::PathBuf};

use serde::Serialize;

use crate::{
    evidence::{
        AcquisitionProvenance, CaptionProvenance, CompiledSource, Coverage, SourceMetadata,
    },
    provider::ytdlp::{AcquisitionControl, ProviderError, YtDlpProvider},
    search::{SearchError, SearchQuery, search},
    source::{CanonicalSource, SourceError, canonicalise_source},
    store::{FileSourceStore, StoreError},
};

#[derive(Debug)]
pub enum EngineError {
    Source(SourceError),
    Provider(ProviderError),
    Store(StoreError),
    Search(SearchError),
    RefreshWithoutCache,
}

impl fmt::Display for EngineError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Source(error) => error.fmt(formatter),
            Self::Provider(error) => error.fmt(formatter),
            Self::Store(error) => error.fmt(formatter),
            Self::Search(error) => error.fmt(formatter),
            Self::RefreshWithoutCache => {
                formatter.write_str("source refresh requires a configured cache")
            }
        }
    }
}

impl Error for EngineError {}

#[derive(Clone, Debug)]
pub struct SourceEngine {
    provider: YtDlpProvider,
    store: Option<FileSourceStore>,
}

impl SourceEngine {
    #[must_use]
    pub fn new(cache_dir: Option<PathBuf>) -> Self {
        Self {
            provider: YtDlpProvider::default(),
            store: cache_dir.map(FileSourceStore::new),
        }
    }

    #[cfg(test)]
    fn with_provider(cache_dir: Option<PathBuf>, provider: YtDlpProvider) -> Self {
        Self {
            provider,
            store: cache_dir.map(FileSourceStore::new),
        }
    }

    /// Finds source evidence using the same acquisition, cache and retrieval path for all surfaces.
    ///
    /// # Errors
    ///
    /// Returns [`EngineError`] when source validation, cache access, acquisition,
    /// evidence compilation or retrieval fails.
    pub fn search_source(
        &self,
        input: &str,
        language: Option<&str>,
        refresh: bool,
        query: &SearchQuery,
        control: &AcquisitionControl,
    ) -> Result<EvidencePacket, EngineError> {
        let (compiled, cache) = self.resolve(input, language, refresh, control)?;
        evidence_packet(&compiled, query, cache).map_err(EngineError::Search)
    }

    /// Reads a whole source, for questions that are answered by the argument rather
    /// than by one moment in it.
    ///
    /// # Errors
    ///
    /// Returns [`EngineError`] when source validation, cache access, acquisition or
    /// evidence compilation fails.
    pub fn read_source(
        &self,
        input: &str,
        language: Option<&str>,
        refresh: bool,
        control: &AcquisitionControl,
    ) -> Result<TranscriptPacket, EngineError> {
        let (compiled, cache) = self.resolve(input, language, refresh, control)?;
        Ok(transcript_packet(&compiled, cache))
    }

    /// Returns compiled evidence for a source, from the cache when it is there and
    /// from the provider when it is not.
    fn resolve(
        &self,
        input: &str,
        language: Option<&str>,
        refresh: bool,
        control: &AcquisitionControl,
    ) -> Result<(CompiledSource, CacheReport), EngineError> {
        if refresh && self.store.is_none() {
            return Err(EngineError::RefreshWithoutCache);
        }
        let source = canonicalise_source(input).map_err(EngineError::Source)?;
        let previous = self
            .store
            .as_ref()
            .map(|store| store.load_latest(&source, language))
            .transpose()
            .map_err(EngineError::Store)?
            .flatten();

        let resolved = if refresh {
            let acquired = self.acquire(input, language, control)?;
            self.save(&acquired, language)?;
            let source_changed = previous
                .as_ref()
                .map(|cached| cached.source_version != acquired.source_version);
            (
                acquired,
                CacheReport {
                    status: CacheStatus::Refreshed,
                    source_changed,
                },
            )
        } else if let Some(cached) = previous {
            (
                cached,
                CacheReport {
                    status: CacheStatus::Hit,
                    source_changed: None,
                },
            )
        } else {
            let acquired = self.acquire(input, language, control)?;
            let status = if self.store.is_some() {
                CacheStatus::Miss
            } else {
                CacheStatus::Disabled
            };
            self.save(&acquired, language)?;
            (
                acquired,
                CacheReport {
                    status,
                    source_changed: None,
                },
            )
        };

        Ok(resolved)
    }

    fn acquire(
        &self,
        input: &str,
        language: Option<&str>,
        control: &AcquisitionControl,
    ) -> Result<CompiledSource, EngineError> {
        self.provider
            .ingest_with_control(input, language, control)
            .map_err(EngineError::Provider)
    }

    fn save(&self, compiled: &CompiledSource, language: Option<&str>) -> Result<(), EngineError> {
        if let Some(store) = &self.store {
            store
                .save(compiled, language.is_none())
                .map_err(EngineError::Store)?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CacheStatus {
    Disabled,
    Hit,
    Miss,
    Refreshed,
    Fixture,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CacheReport {
    pub status: CacheStatus,
    pub source_changed: Option<bool>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct EvidencePacket {
    pub source: CanonicalSource,
    pub source_version: String,
    pub metadata: SourceMetadata,
    pub coverage: Coverage,
    pub acquisition: AcquisitionProvenance,
    pub cache: CacheReport,
    pub query: String,
    pub moments: Vec<EvidenceMoment>,
    pub warnings: Vec<&'static str>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct EvidenceMoment {
    pub id: String,
    pub start_ms: u64,
    pub timestamp_label: String,
    pub end_ms: u64,
    pub timestamp_url: String,
    pub excerpt: String,
    pub evidence_kind: &'static str,
    pub language: String,
    pub caption_provenance: CaptionProvenance,
    pub score: u32,
    pub matched_terms: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TranscriptPacket {
    pub source: CanonicalSource,
    pub source_version: String,
    pub metadata: SourceMetadata,
    pub coverage: Coverage,
    pub acquisition: AcquisitionProvenance,
    pub cache: CacheReport,
    pub language: String,
    pub caption_provenance: CaptionProvenance,
    pub passage_count: usize,
    pub passages: Vec<TranscriptPassage>,
    pub warnings: Vec<&'static str>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TranscriptPassage {
    pub start_ms: u64,
    pub timestamp_label: String,
    pub end_ms: u64,
    pub timestamp_url: String,
    pub text: String,
}

/// Packages a whole compiled source so a reader can quote any part of it.
///
/// Retrieval answers "where does this happen". Some questions are not about a
/// moment at all, and for a source of this length reading it whole is both cheaper
/// and more faithful than ranking it. Every passage keeps its own timestamp so an
/// answer drawn from the argument as a whole can still be traced to where it was
/// said.
#[must_use]
pub fn transcript_packet(compiled: &CompiledSource, cache: CacheReport) -> TranscriptPacket {
    let passages = compiled
        .evidence
        .iter()
        .map(|evidence| TranscriptPassage {
            start_ms: evidence.start_ms,
            timestamp_label: timestamp_label(evidence.start_ms),
            end_ms: evidence.end_ms,
            timestamp_url: timestamp_url(compiled, evidence.start_ms),
            text: evidence.text.clone(),
        })
        .collect::<Vec<_>>();

    // Compilation rejects a source with no evidence, so the first passage carries
    // the transcript's provenance. Treat the unreachable case as machine-heard,
    // because warning about wording that was in fact verbatim costs a reader little
    // and trusting wording that was in fact guessed costs it a great deal.
    let transcript = compiled
        .evidence
        .first()
        .map(|evidence| &evidence.transcript);
    let caption_provenance = transcript.map_or(CaptionProvenance::Generated, |transcript| {
        transcript.captions.clone()
    });

    let warnings = source_warnings(compiled);

    TranscriptPacket {
        source: compiled.source.clone(),
        source_version: compiled.source_version.clone(),
        metadata: compiled.metadata.clone(),
        coverage: compiled.coverage.clone(),
        acquisition: compiled.acquisition.clone(),
        cache,
        language: transcript
            .map(|transcript| transcript.language.clone())
            .unwrap_or_default(),
        caption_provenance,
        passage_count: passages.len(),
        passages,
        warnings,
    }
}

fn timestamp_url(compiled: &CompiledSource, start_ms: u64) -> String {
    format!("{}&t={}s", compiled.source.canonical_url, start_ms / 1_000)
}

fn timestamp_label(start_ms: u64) -> String {
    let total_seconds = start_ms / 1_000;
    let seconds = total_seconds % 60;
    let total_minutes = total_seconds / 60;
    let minutes = total_minutes % 60;
    let hours = total_minutes / 60;

    if hours == 0 {
        format!("{minutes}:{seconds:02}")
    } else {
        format!("{hours}:{minutes:02}:{seconds:02}")
    }
}

fn coverage_warnings(compiled: &CompiledSource) -> Vec<&'static str> {
    let mut warnings = Vec::new();
    if !compiled.coverage.transcript_complete {
        warnings.push("transcript_incomplete");
    }
    if !compiled.coverage.visuals_processed {
        warnings.push("visuals_not_processed");
    }
    warnings
}

fn source_warnings(compiled: &CompiledSource) -> Vec<&'static str> {
    let mut warnings = coverage_warnings(compiled);
    if compiled
        .evidence
        .first()
        .is_some_and(|evidence| evidence.transcript.captions != CaptionProvenance::Manual)
    {
        warnings.push("captions_machine_generated");
    }
    warnings
}

/// Packages already compiled evidence for a transport without changing retrieval behaviour.
///
/// # Errors
///
/// Returns [`SearchError`] when the query constraints are invalid.
pub fn evidence_packet(
    compiled: &CompiledSource,
    query: &SearchQuery,
    cache: CacheReport,
) -> Result<EvidencePacket, SearchError> {
    let moments = search(compiled, query)?
        .into_iter()
        .map(|result| EvidenceMoment {
            id: result.evidence.id.clone(),
            start_ms: result.evidence.start_ms,
            timestamp_label: timestamp_label(result.evidence.start_ms),
            end_ms: result.evidence.end_ms,
            timestamp_url: timestamp_url(compiled, result.evidence.start_ms),
            excerpt: result.evidence.text.clone(),
            evidence_kind: "transcript",
            language: result.evidence.transcript.language.clone(),
            caption_provenance: result.evidence.transcript.captions.clone(),
            score: result.score,
            matched_terms: result.matched_terms,
        })
        .collect();

    Ok(EvidencePacket {
        source: compiled.source.clone(),
        source_version: compiled.source_version.clone(),
        metadata: compiled.metadata.clone(),
        coverage: compiled.coverage.clone(),
        acquisition: compiled.acquisition.clone(),
        cache,
        query: query.text.clone(),
        moments,
        warnings: source_warnings(compiled),
    })
}

#[cfg(test)]
mod tests {
    use super::{
        CacheReport, CacheStatus, SourceEngine, evidence_packet, timestamp_label, transcript_packet,
    };
    use crate::{
        evidence::CaptionProvenance,
        fixture::compile_fixture,
        provider::ytdlp::{AcquisitionControl, YtDlpProvider},
        search::SearchQuery,
    };

    const FIXTURE: &str = "schema_version\t1\n\
source_url\thttps://youtu.be/dQw4w9WgXcQ\n\
title\tEngine fixture\n\
creator\tOriel\n\
duration_ms\t10000\n\
language\ten\n\
caption_provenance\tmanual\n\
cue\t0\t10000\tThe cache keeps timestamped evidence reusable.\n";

    const GENERATED: &str = "schema_version\t1\n\
source_url\thttps://youtu.be/dQw4w9WgXcQ\n\
title\tTranscript fixture\n\
creator\tOriel\n\
duration_ms\t30000\n\
language\ten\n\
caption_provenance\tgenerated\n\
cue\t0\t10000\tThe question is how small a set can be.\n\
cue\t10000\t20000\tA needle rotates inside a shrinking area.\n\
cue\t20000\t30000\tThe answer turned out to depend on dimension.\n";

    fn fixture_cache() -> CacheReport {
        CacheReport {
            status: CacheStatus::Fixture,
            source_changed: None,
        }
    }

    #[test]
    fn a_transcript_carries_every_passage_in_order_with_citable_timestamps() {
        let compiled = compile_fixture(GENERATED).expect("fixture should compile");
        let packet = transcript_packet(&compiled, fixture_cache());

        assert_eq!(
            packet
                .passages
                .iter()
                .map(|passage| passage.start_ms)
                .collect::<Vec<_>>(),
            [0, 10_000, 20_000]
        );
        assert_eq!(
            packet.passages[1].text,
            "A needle rotates inside a shrinking area."
        );
        assert!(packet.passages[1].timestamp_url.ends_with("&t=10s"));
        assert_eq!(packet.passages[1].timestamp_label, "0:10");
    }

    /// Generated captions mishear proper nouns — in a source about the Kakeya
    /// conjecture they never spell it once. An agent reading the whole transcript
    /// has no ranking signal to warn it, so the packet has to say so itself.
    #[test]
    fn a_generated_transcript_warns_that_its_wording_was_machine_heard() {
        let compiled = compile_fixture(GENERATED).expect("fixture should compile");
        let packet = transcript_packet(&compiled, fixture_cache());

        assert_eq!(packet.caption_provenance, CaptionProvenance::Generated);
        assert!(packet.warnings.contains(&"captions_machine_generated"));
    }

    #[test]
    fn a_manual_transcript_carries_no_mishearing_warning() {
        let compiled = compile_fixture(FIXTURE).expect("fixture should compile");
        let packet = transcript_packet(&compiled, fixture_cache());

        assert_eq!(packet.caption_provenance, CaptionProvenance::Manual);
        assert!(!packet.warnings.contains(&"captions_machine_generated"));
    }

    #[test]
    fn generated_search_evidence_warns_that_its_wording_was_machine_heard() {
        let compiled = compile_fixture(GENERATED).expect("fixture should compile");
        let packet = evidence_packet(
            &compiled,
            &SearchQuery::new("needle rotates"),
            fixture_cache(),
        )
        .expect("search should work");

        assert_eq!(
            packet.moments[0].caption_provenance,
            CaptionProvenance::Generated
        );
        assert!(packet.warnings.contains(&"captions_machine_generated"));
    }

    #[test]
    fn packet_preserves_timestamp_and_cache_provenance() {
        let compiled = compile_fixture(FIXTURE).expect("fixture should compile");
        let packet = evidence_packet(
            &compiled,
            &SearchQuery::new("timestamped evidence"),
            CacheReport {
                status: CacheStatus::Fixture,
                source_changed: None,
            },
        )
        .expect("search should work");

        assert_eq!(packet.moments[0].start_ms, 0);
        assert_eq!(packet.moments[0].timestamp_label, "0:00");
        assert_eq!(packet.cache.status, CacheStatus::Fixture);
        assert_eq!(packet.moments[0].evidence_kind, "transcript");
    }

    #[test]
    fn timestamp_labels_are_canonical_for_short_and_long_sources() {
        assert_eq!(timestamp_label(201_999), "3:21");
        assert_eq!(timestamp_label(3_661_000), "1:01:01");
    }

    #[test]
    fn refresh_without_a_cache_fails_before_provider_access() {
        let engine = SourceEngine::with_provider(None, YtDlpProvider::new("missing-yt-dlp"));
        let error = engine
            .search_source(
                "https://youtu.be/dQw4w9WgXcQ",
                Some("en"),
                true,
                &SearchQuery::new("evidence"),
                &AcquisitionControl::default(),
            )
            .expect_err("refresh should require a cache");

        assert_eq!(
            error.to_string(),
            "source refresh requires a configured cache"
        );
    }
}
