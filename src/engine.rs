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

        let (compiled, cache) = if refresh {
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

        evidence_packet(&compiled, query, cache).map_err(EngineError::Search)
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
    pub end_ms: u64,
    pub timestamp_url: String,
    pub excerpt: String,
    pub evidence_kind: &'static str,
    pub language: String,
    pub caption_provenance: CaptionProvenance,
    pub score: u32,
    pub matched_terms: Vec<String>,
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
            end_ms: result.evidence.end_ms,
            timestamp_url: format!(
                "{}&t={}s",
                compiled.source.canonical_url,
                result.evidence.start_ms / 1_000
            ),
            excerpt: result.evidence.text.clone(),
            evidence_kind: "transcript",
            language: result.evidence.transcript.language.clone(),
            caption_provenance: result.evidence.transcript.captions.clone(),
            score: result.score,
            matched_terms: result.matched_terms,
        })
        .collect();
    let mut warnings = Vec::new();
    if !compiled.coverage.transcript_complete {
        warnings.push("transcript_incomplete");
    }
    if !compiled.coverage.visuals_processed {
        warnings.push("visuals_not_processed");
    }

    Ok(EvidencePacket {
        source: compiled.source.clone(),
        source_version: compiled.source_version.clone(),
        metadata: compiled.metadata.clone(),
        coverage: compiled.coverage.clone(),
        acquisition: compiled.acquisition.clone(),
        cache,
        query: query.text.clone(),
        moments,
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::{CacheReport, CacheStatus, SourceEngine, evidence_packet};
    use crate::{
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
        assert_eq!(packet.cache.status, CacheStatus::Fixture);
        assert_eq!(packet.moments[0].evidence_kind, "transcript");
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
