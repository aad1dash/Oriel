use std::{hint::black_box, time::Instant};

use oriel::{
    compile::{AcquiredCue, AcquiredSource, compile_source},
    evidence::{
        AcquisitionProvenance, CaptionProvenance, Coverage, SourceMetadata, TranscriptProvenance,
    },
    search::{SearchQuery, search},
    source::canonicalise_source,
};

const CUE_COUNT: usize = 10_000;
const MEASURED_RUNS: usize = 500;
const WARMUP_RUNS: usize = 50;
const CUE_DURATION_MS: u64 = 5_000;

fn main() {
    let source = long_source();
    let query = SearchQuery::new("cache invalidation evidence");

    for _ in 0..WARMUP_RUNS {
        black_box(search(black_box(&source), black_box(&query)).expect("search should succeed"));
    }

    let mut samples = Vec::with_capacity(MEASURED_RUNS);
    for _ in 0..MEASURED_RUNS {
        let started = Instant::now();
        black_box(search(black_box(&source), black_box(&query)).expect("search should succeed"));
        samples.push(started.elapsed());
    }
    samples.sort_unstable();

    println!(
        "corpus_cues={CUE_COUNT} runs={MEASURED_RUNS} p50_us={} p95_us={} p99_us={}",
        percentile_micros(&samples, 50),
        percentile_micros(&samples, 95),
        percentile_micros(&samples, 99),
    );
}

fn long_source() -> oriel::evidence::CompiledSource {
    let cues = (0..CUE_COUNT)
        .map(|index| {
            let start_ms = u64::try_from(index).unwrap_or(u64::MAX) * CUE_DURATION_MS;
            let text = if index % 97 == 0 {
                format!(
                    "Section {index} demonstrates cache invalidation evidence with a concrete example."
                )
            } else {
                format!(
                    "Section {index} discusses source provenance, retrieval context and careful interpretation."
                )
            };
            AcquiredCue {
                start_ms,
                end_ms: start_ms + CUE_DURATION_MS,
                text,
            }
        })
        .collect();
    let duration_ms = u64::try_from(CUE_COUNT).unwrap_or(u64::MAX) * CUE_DURATION_MS;

    compile_source(AcquiredSource {
        source: canonicalise_source("https://youtu.be/Ori3lDemo01")
            .expect("benchmark source should be valid"),
        metadata: SourceMetadata {
            title: "Synthetic long-source latency corpus".to_owned(),
            creator: "Oriel benchmark".to_owned(),
            duration_ms,
        },
        transcript: TranscriptProvenance {
            language: "en".to_owned(),
            captions: CaptionProvenance::Manual,
        },
        cues,
        coverage: Coverage {
            metadata: true,
            transcript_start_ms: 0,
            transcript_end_ms: duration_ms,
            transcript_complete: true,
            visuals_processed: false,
        },
        acquisition: AcquisitionProvenance {
            adapter: "benchmark".to_owned(),
            source_format: "synthetic_latency_v1".to_owned(),
            tool: None,
        },
    })
    .expect("benchmark source should compile")
}

fn percentile_micros(samples: &[std::time::Duration], percentile: usize) -> u128 {
    let index = (samples.len() - 1) * percentile / 100;
    samples[index].as_micros()
}
