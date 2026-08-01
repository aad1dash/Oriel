use std::{env, error::Error, fmt, fs, path::PathBuf, process::ExitCode};

use oriel::{
    evidence::{CaptionProvenance, CompiledSource},
    fixture::compile_fixture,
    search::{SearchQuery, search},
    source::{CanonicalSource, SourceProvider, canonicalise_source},
};

const USAGE: &str = "Usage:\n  oriel resolve <youtube-url>\n  oriel search --fixture <path> --query <text> [--limit <count>] [--start-ms <ms>] [--end-ms <ms>]";

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
        }
    }
}

impl Error for CliError {}

enum CliCommand {
    Resolve(String),
    Search {
        fixture: PathBuf,
        query: SearchQuery,
    },
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
            println!("{}", source_json(&source));
        }
        CliCommand::Search { fixture, query } => {
            let input = fs::read_to_string(&fixture).map_err(|error| CliError::ReadFixture {
                path: fixture,
                error,
            })?;
            let compiled = compile_fixture(&input).map_err(engine_error)?;
            let results = search(&compiled, &query).map_err(engine_error)?;
            println!("{}", evidence_packet_json(&compiled, &query, &results));
        }
    }
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
    let mut query = None;
    let mut limit = None;
    let mut start_ms = None;
    let mut end_ms = None;

    while let Some(flag) = args.next() {
        let value = args
            .next()
            .ok_or_else(|| CliError::Usage(format!("{flag} requires a value")))?;
        match flag.as_str() {
            "--fixture" => set_once(&mut fixture, PathBuf::from(value), &flag)?,
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

    let fixture = fixture.ok_or_else(|| CliError::Usage("search requires --fixture".to_owned()))?;
    let query_text = query.ok_or_else(|| CliError::Usage("search requires --query".to_owned()))?;
    let mut query = SearchQuery::new(query_text);
    if let Some(limit) = limit {
        query.limit = limit;
    }
    query.start_ms = start_ms;
    query.end_ms = end_ms;

    Ok(CliCommand::Search { fixture, query })
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

fn source_json(source: &CanonicalSource) -> String {
    format!(
        "{{\"provider\":\"{}\",\"source_id\":\"{}\",\"canonical_url\":\"{}\"}}",
        provider_name(&source.provider),
        json_string(&source.source_id),
        json_string(&source.canonical_url)
    )
}

fn evidence_packet_json(
    compiled: &CompiledSource,
    query: &SearchQuery,
    results: &[oriel::search::RetrievedEvidence<'_>],
) -> String {
    let moments = results
        .iter()
        .map(|result| {
            let evidence = result.evidence;
            let matched_terms = result
                .matched_terms
                .iter()
                .map(|term| format!("\"{}\"", json_string(term)))
                .collect::<Vec<_>>()
                .join(",");
            let timestamp_url = format!(
                "{}&t={}s",
                compiled.source.canonical_url,
                evidence.start_ms / 1_000
            );
            format!(
                concat!(
                    "{{\"id\":\"{}\",\"start_ms\":{},\"end_ms\":{},",
                    "\"timestamp_url\":\"{}\",\"excerpt\":\"{}\",",
                    "\"evidence_kind\":\"transcript\",\"language\":\"{}\",",
                    "\"caption_provenance\":\"{}\",\"score\":{},\"matched_terms\":[{}]}}"
                ),
                json_string(&evidence.id),
                evidence.start_ms,
                evidence.end_ms,
                json_string(&timestamp_url),
                json_string(&evidence.text),
                json_string(&evidence.transcript.language),
                caption_provenance_name(&evidence.transcript.captions),
                result.score,
                matched_terms
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let warning = if compiled.coverage.visuals_processed {
        "[]"
    } else {
        "[\"visuals_not_processed\"]"
    };

    format!(
        concat!(
            "{{\"source\":{},\"source_version\":\"{}\",",
            "\"metadata\":{{\"title\":\"{}\",\"creator\":\"{}\",\"duration_ms\":{}}},",
            "\"coverage\":{{\"metadata\":{},\"transcript_start_ms\":{},",
            "\"transcript_end_ms\":{},\"transcript_complete\":{},\"visuals_processed\":{}}},",
            "\"query\":\"{}\",\"moments\":[{}],\"warnings\":{}}}"
        ),
        source_json(&compiled.source),
        json_string(&compiled.source_version),
        json_string(&compiled.metadata.title),
        json_string(&compiled.metadata.creator),
        compiled.metadata.duration_ms,
        compiled.coverage.metadata,
        compiled.coverage.transcript_start_ms,
        compiled.coverage.transcript_end_ms,
        compiled.coverage.transcript_complete,
        compiled.coverage.visuals_processed,
        json_string(&query.text),
        moments,
        warning
    )
}

fn provider_name(provider: &SourceProvider) -> &'static str {
    match provider {
        SourceProvider::YouTube => "youtube",
    }
}

fn caption_provenance_name(provenance: &CaptionProvenance) -> &'static str {
    match provenance {
        CaptionProvenance::Manual => "manual",
        CaptionProvenance::Generated => "generated",
        CaptionProvenance::LocalTranscription => "local_transcription",
    }
}

fn json_string(input: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut escaped = String::with_capacity(input.len());
    for character in input.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\u{08}' => escaped.push_str("\\b"),
            '\u{0c}' => escaped.push_str("\\f"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            control if control <= '\u{1f}' => {
                let value = control as usize;
                escaped.push_str("\\u00");
                escaped.push(char::from(HEX[value >> 4]));
                escaped.push(char::from(HEX[value & 0x0f]));
            }
            other => escaped.push(other),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::{CliCommand, json_string, parse_command};

    #[test]
    fn parses_search_arguments_in_any_order() {
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

        let CliCommand::Search { fixture, query } =
            parse_command(args).expect("arguments should parse")
        else {
            panic!("expected search command");
        };
        assert_eq!(fixture.to_string_lossy(), "source.tsv");
        assert_eq!(query.text, "cache invalidation");
        assert_eq!(query.limit, 3);
    }

    #[test]
    fn escapes_json_control_characters() {
        assert_eq!(json_string("a\n\"b\\c\u{01}"), "a\\n\\\"b\\\\c\\u0001");
    }
}
