use std::{env, error::Error, fmt, fs, path::PathBuf, process::ExitCode};

use oriel::{
    evidence::{AcquisitionProvenance, CaptionProvenance, Coverage, SourceMetadata},
    fixture::compile_fixture,
    provider::ytdlp::YtDlpProvider,
    search::{RetrievedEvidence, SearchQuery, search},
    source::{CanonicalSource, canonicalise_source},
    store::FileSourceStore,
};
use serde::Serialize;

const USAGE: &str = "Usage:\n  oriel resolve <youtube-url>\n  oriel search (--fixture <path> | --source <youtube-url>) --query <text> [--language <tag>] [--cache-dir <path>] [--refresh] [--limit <count>] [--start-ms <ms>] [--end-ms <ms>]";

#[derive(Debug)]
enum CliError {
    Usage(String),
    InvalidNumber {
        flag: String,
        value: String,
    },
    ReadFixture {
        path: PathBuf,
        error: std::io::Error,
    },
    Engine(Box<dyn Error>),
    Serialise(serde_json::Error),
}

impl fmt::Display for CliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(message) => write!(formatter, "{message}\n\n{USAGE}"),
            Self::InvalidNumber { flag, value } => {
                write!(formatter, "'{value}' is not a valid number for {flag}")
            }
            Self::ReadFixture { path, error } => {
                write!(
                    formatter,
                    "could not read fixture '{}': {error}",
                    path.display()
                )
            }
            Self::Engine(error) => error.fmt(formatter),
            Self::Serialise(error) => {
                write!(formatter, "could not serialise engine output: {error}")
            }
        }
    }
}

impl Error for CliError {}

enum CliCommand {
    Resolve(String),
    Search {
        input: SearchInput,
        query: SearchQuery,
    },
}

enum SearchInput {
    Fixture(PathBuf),
    LiveSource {
        url: String,
        language: Option<String>,
        cache_dir: Option<PathBuf>,
        refresh: bool,
    },
}

#[derive(Serialize)]
struct EvidencePacket<'a> {
    source: &'a CanonicalSource,
    source_version: &'a str,
    metadata: &'a SourceMetadata,
    coverage: &'a Coverage,
    acquisition: &'a AcquisitionProvenance,
    cache: CacheReport,
    query: &'a str,
    moments: Vec<EvidenceMoment<'a>>,
    warnings: Vec<&'static str>,
}

#[derive(Serialize)]
struct CacheReport {
    status: &'static str,
    source_changed: Option<bool>,
}

#[derive(Serialize)]
struct EvidenceMoment<'a> {
    id: &'a str,
    start_ms: u64,
    end_ms: u64,
    timestamp_url: String,
    excerpt: &'a str,
    evidence_kind: &'static str,
    language: &'a str,
    caption_provenance: &'a CaptionProvenance,
    score: u32,
    matched_terms: &'a [String],
}

fn main() -> ExitCode {
    match run(env::args().skip(1)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: impl Iterator<Item = String>) -> Result<(), CliError> {
    match parse_command(args)? {
        CliCommand::Resolve(input) => {
            let source = canonicalise_source(&input).map_err(engine_error)?;
            print_json(&source)?;
        }
        CliCommand::Search { input, query } => {
            let (compiled, cache) = match input {
                SearchInput::Fixture(path) => {
                    let fixture = fs::read_to_string(&path)
                        .map_err(|error| CliError::ReadFixture { path, error })?;
                    (
                        compile_fixture(&fixture).map_err(engine_error)?,
                        CacheReport {
                            status: "fixture",
                            source_changed: None,
                        },
                    )
                }
                SearchInput::LiveSource {
                    url,
                    language,
                    cache_dir,
                    refresh,
                } => {
                    let source = canonicalise_source(&url).map_err(engine_error)?;
                    let store = cache_dir.map(FileSourceStore::new);
                    let previous = if let Some(store) = &store {
                        store
                            .load_latest(&source, language.as_deref())
                            .map_err(engine_error)?
                    } else {
                        None
                    };
                    if !refresh && previous.is_some() {
                        let cached = previous.ok_or_else(|| {
                            CliError::Usage("cache state changed unexpectedly".to_owned())
                        })?;
                        (
                            cached,
                            CacheReport {
                                status: "hit",
                                source_changed: None,
                            },
                        )
                    } else {
                        let acquired = YtDlpProvider::default()
                            .ingest(&url, language.as_deref())
                            .map_err(engine_error)?;
                        if let Some(store) = &store {
                            store
                                .save(&acquired, language.is_none())
                                .map_err(engine_error)?;
                        }
                        let cache = if store.is_none() {
                            CacheReport {
                                status: "disabled",
                                source_changed: None,
                            }
                        } else if refresh {
                            CacheReport {
                                status: "refreshed",
                                source_changed: previous
                                    .as_ref()
                                    .map(|cached| cached.source_version != acquired.source_version),
                            }
                        } else {
                            CacheReport {
                                status: "miss",
                                source_changed: None,
                            }
                        };
                        (acquired, cache)
                    }
                }
            };
            let results = search(&compiled, &query).map_err(engine_error)?;
            let packet = evidence_packet(&compiled, &query, &results, cache);
            print_json(&packet)?;
        }
    }
    Ok(())
}

fn print_json(value: &impl Serialize) -> Result<(), CliError> {
    let output = serde_json::to_string(value).map_err(CliError::Serialise)?;
    println!("{output}");
    Ok(())
}

fn engine_error(error: impl Error + 'static) -> CliError {
    CliError::Engine(Box::new(error))
}

fn parse_command(mut args: impl Iterator<Item = String>) -> Result<CliCommand, CliError> {
    match args.next().as_deref() {
        Some("resolve") => parse_resolve(args),
        Some("search") => parse_search(args),
        Some(command) => Err(CliError::Usage(format!("unknown command '{command}'"))),
        None => Err(CliError::Usage("a command is required".to_owned())),
    }
}

fn parse_resolve(mut args: impl Iterator<Item = String>) -> Result<CliCommand, CliError> {
    let source = args
        .next()
        .ok_or_else(|| CliError::Usage("resolve requires a YouTube URL".to_owned()))?;
    if args.next().is_some() {
        return Err(CliError::Usage(
            "resolve accepts exactly one YouTube URL".to_owned(),
        ));
    }
    Ok(CliCommand::Resolve(source))
}

fn parse_search(mut args: impl Iterator<Item = String>) -> Result<CliCommand, CliError> {
    let mut fixture = None;
    let mut source = None;
    let mut language = None;
    let mut cache_dir = None;
    let mut refresh = false;
    let mut query = None;
    let mut limit = None;
    let mut start_ms = None;
    let mut end_ms = None;

    while let Some(flag) = args.next() {
        if flag == "--refresh" {
            if refresh {
                return Err(CliError::Usage(
                    "--refresh may only be supplied once".to_owned(),
                ));
            }
            refresh = true;
            continue;
        }
        let value = args
            .next()
            .ok_or_else(|| CliError::Usage(format!("{flag} requires a value")))?;
        match flag.as_str() {
            "--fixture" => set_once(&mut fixture, PathBuf::from(value), &flag)?,
            "--source" => set_once(&mut source, value, &flag)?,
            "--language" => set_once(&mut language, value, &flag)?,
            "--cache-dir" => set_once(&mut cache_dir, PathBuf::from(value), &flag)?,
            "--query" => set_once(&mut query, value, &flag)?,
            "--limit" => {
                let parsed = parse_number(&flag, &value)?;
                set_once(&mut limit, parsed, &flag)?;
            }
            "--start-ms" => {
                let parsed = parse_number(&flag, &value)?;
                set_once(&mut start_ms, parsed, &flag)?;
            }
            "--end-ms" => {
                let parsed = parse_number(&flag, &value)?;
                set_once(&mut end_ms, parsed, &flag)?;
            }
            _ => return Err(CliError::Usage(format!("unknown search flag '{flag}'"))),
        }
    }

    let input = match (fixture, source) {
        (Some(path), None) => {
            if language.is_some() || cache_dir.is_some() || refresh {
                return Err(CliError::Usage(
                    "--language, --cache-dir and --refresh are only valid with --source".to_owned(),
                ));
            }
            SearchInput::Fixture(path)
        }
        (None, Some(url)) => {
            if refresh && cache_dir.is_none() {
                return Err(CliError::Usage("--refresh requires --cache-dir".to_owned()));
            }
            SearchInput::LiveSource {
                url,
                language,
                cache_dir,
                refresh,
            }
        }
        (Some(_), Some(_)) => {
            return Err(CliError::Usage(
                "search accepts either --fixture or --source, not both".to_owned(),
            ));
        }
        (None, None) => {
            return Err(CliError::Usage(
                "search requires either --fixture or --source".to_owned(),
            ));
        }
    };
    let query_text = query.ok_or_else(|| CliError::Usage("search requires --query".to_owned()))?;
    let mut query = SearchQuery::new(query_text);
    if let Some(limit) = limit {
        query.limit = limit;
    }
    query.start_ms = start_ms;
    query.end_ms = end_ms;

    Ok(CliCommand::Search { input, query })
}

fn set_once<T>(target: &mut Option<T>, value: T, flag: &str) -> Result<(), CliError> {
    if target.is_some() {
        return Err(CliError::Usage(format!("{flag} may only be supplied once")));
    }
    *target = Some(value);
    Ok(())
}

fn parse_number<T>(flag: &str, value: &str) -> Result<T, CliError>
where
    T: std::str::FromStr,
{
    value.parse().map_err(|_| CliError::InvalidNumber {
        flag: flag.to_owned(),
        value: value.to_owned(),
    })
}

fn evidence_packet<'a>(
    compiled: &'a oriel::evidence::CompiledSource,
    query: &'a SearchQuery,
    results: &'a [RetrievedEvidence<'a>],
    cache: CacheReport,
) -> EvidencePacket<'a> {
    let moments = results
        .iter()
        .map(|result| EvidenceMoment {
            id: &result.evidence.id,
            start_ms: result.evidence.start_ms,
            end_ms: result.evidence.end_ms,
            timestamp_url: format!(
                "{}&t={}s",
                compiled.source.canonical_url,
                result.evidence.start_ms / 1_000
            ),
            excerpt: &result.evidence.text,
            evidence_kind: "transcript",
            language: &result.evidence.transcript.language,
            caption_provenance: &result.evidence.transcript.captions,
            score: result.score,
            matched_terms: &result.matched_terms,
        })
        .collect();
    let mut warnings = Vec::new();
    if !compiled.coverage.transcript_complete {
        warnings.push("transcript_incomplete");
    }
    if !compiled.coverage.visuals_processed {
        warnings.push("visuals_not_processed");
    }

    EvidencePacket {
        source: &compiled.source,
        source_version: &compiled.source_version,
        metadata: &compiled.metadata,
        coverage: &compiled.coverage,
        acquisition: &compiled.acquisition,
        cache,
        query: &query.text,
        moments,
        warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::{CliCommand, SearchInput, parse_command};

    #[test]
    fn parses_fixture_search_arguments_in_any_order() {
        let args = [
            "search",
            "--query",
            "cache invalidation",
            "--fixture",
            "source.tsv",
            "--limit",
            "3",
        ]
        .into_iter()
        .map(str::to_owned);

        let CliCommand::Search { input, query } =
            parse_command(args).expect("arguments should parse")
        else {
            panic!("expected search command");
        };
        let SearchInput::Fixture(path) = input else {
            panic!("expected fixture input");
        };
        assert_eq!(path.to_string_lossy(), "source.tsv");
        assert_eq!(query.text, "cache invalidation");
        assert_eq!(query.limit, 3);
    }

    #[test]
    fn parses_live_source_and_language() {
        let args = [
            "search",
            "--source",
            "https://youtu.be/dQw4w9WgXcQ",
            "--language",
            "en",
            "--query",
            "evidence",
        ]
        .into_iter()
        .map(str::to_owned);

        let CliCommand::Search { input, .. } = parse_command(args).expect("arguments should parse")
        else {
            panic!("expected search command");
        };
        let SearchInput::LiveSource {
            url,
            language,
            cache_dir,
            refresh,
        } = input
        else {
            panic!("expected live source input");
        };
        assert_eq!(url, "https://youtu.be/dQw4w9WgXcQ");
        assert_eq!(language.as_deref(), Some("en"));
        assert!(cache_dir.is_none());
        assert!(!refresh);
    }
}
