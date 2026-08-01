use std::{env, error::Error, fmt, fs, path::PathBuf, process::ExitCode};

use oriel::{
    engine::{CacheReport, CacheStatus, SourceEngine, evidence_packet},
    fixture::compile_fixture,
    provider::ytdlp::AcquisitionControl,
    search::SearchQuery,
    source::canonicalise_source,
};
use serde::Serialize;

const USAGE: &str = "Usage:\n  oriel resolve <youtube-url>\n  oriel search (--fixture <path> | --source <youtube-url>) --query <text> [--language <tag>] [--cache-dir <path>] [--refresh] [--limit <count>] [--start-ms <ms>] [--end-ms <ms>]\n  oriel mcp --cache-dir <path>";

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
    Mcp(String),
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
            Self::Mcp(message) => formatter.write_str(message),
            Self::Serialise(error) => {
                write!(formatter, "could not serialise engine output: {error}")
            }
        }
    }
}

impl Error for CliError {}

enum CliCommand {
    Resolve(String),
    Mcp {
        cache_dir: PathBuf,
    },
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
        CliCommand::Mcp { cache_dir } => {
            let runtime = tokio::runtime::Runtime::new().map_err(|error| {
                CliError::Mcp(format!("starting the MCP runtime failed: {error}"))
            })?;
            runtime
                .block_on(oriel::mcp::serve_stdio(cache_dir))
                .map_err(CliError::Mcp)?;
        }
        CliCommand::Search { input, query } => {
            let packet = match input {
                SearchInput::Fixture(path) => {
                    let fixture = fs::read_to_string(&path)
                        .map_err(|error| CliError::ReadFixture { path, error })?;
                    let compiled = compile_fixture(&fixture).map_err(engine_error)?;
                    evidence_packet(
                        &compiled,
                        &query,
                        CacheReport {
                            status: CacheStatus::Fixture,
                            source_changed: None,
                        },
                    )
                    .map_err(engine_error)?
                }
                SearchInput::LiveSource {
                    url,
                    language,
                    cache_dir,
                    refresh,
                } => SourceEngine::new(cache_dir)
                    .search_source(
                        &url,
                        language.as_deref(),
                        refresh,
                        &query,
                        &AcquisitionControl::default(),
                    )
                    .map_err(engine_error)?,
            };
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
        Some("mcp") => parse_mcp(args),
        Some(command) => Err(CliError::Usage(format!("unknown command '{command}'"))),
        None => Err(CliError::Usage("a command is required".to_owned())),
    }
}

fn parse_mcp(mut args: impl Iterator<Item = String>) -> Result<CliCommand, CliError> {
    if args.next().as_deref() != Some("--cache-dir") {
        return Err(CliError::Usage(
            "mcp requires --cache-dir followed by an explicit storage path".to_owned(),
        ));
    }
    let cache_dir = args
        .next()
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .ok_or_else(|| CliError::Usage("--cache-dir requires a path".to_owned()))?;
    if args.next().is_some() {
        return Err(CliError::Usage(
            "mcp accepts only one --cache-dir path".to_owned(),
        ));
    }
    Ok(CliCommand::Mcp { cache_dir })
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

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
            "--cache-dir",
            ".oriel-cache",
            "--refresh",
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
        assert_eq!(cache_dir, Some(PathBuf::from(".oriel-cache")));
        assert!(refresh);
    }

    #[test]
    fn mcp_requires_an_explicit_cache_directory() {
        let args = ["mcp", "--cache-dir", ".oriel-cache"]
            .into_iter()
            .map(str::to_owned);
        let CliCommand::Mcp { cache_dir } =
            parse_command(args).expect("MCP arguments should parse")
        else {
            panic!("expected MCP command");
        };
        assert_eq!(cache_dir, PathBuf::from(".oriel-cache"));

        let missing = ["mcp"].into_iter().map(str::to_owned);
        assert!(parse_command(missing).is_err());
    }
}
