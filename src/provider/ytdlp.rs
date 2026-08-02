use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    ffi::OsString,
    fmt,
    fs::{self, File},
    io,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use tempfile::Builder;

use crate::{
    compile::{AcquiredCue, AcquiredSource, CompileError, compile_source, segment_cues},
    evidence::{
        AcquisitionProvenance, CaptionProvenance, CompiledSource, Coverage, SourceMetadata,
        ToolProvenance, TranscriptProvenance,
    },
    source::{CanonicalSource, SourceError, canonicalise_source},
};

const MAX_METADATA_BYTES: u64 = 10 * 1024 * 1024;
const MAX_CAPTION_BYTES: u64 = 50 * 1024 * 1024;
const MAX_STDERR_BYTES: u64 = 64 * 1024;
const MAX_LANGUAGE_LENGTH: usize = 64;
const DEFAULT_STAGE_TIMEOUT: Duration = Duration::from_secs(90);
const PROCESS_POLL_INTERVAL: Duration = Duration::from_millis(20);
const PASSAGE_TARGET_MS: u64 = 30_000;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProviderFailureKind {
    SourceUnavailable,
    SourcePrivate,
    AuthenticationRequired,
    ProviderFailure,
}

#[derive(Debug)]
pub enum ProviderError {
    InvalidSource(SourceError),
    InvalidLanguage(String),
    ToolUnavailable,
    TimedOut {
        stage: &'static str,
        timeout: Duration,
    },
    Cancelled(&'static str),
    ProcessFailed {
        stage: &'static str,
        kind: ProviderFailureKind,
    },
    ArtifactMissing(&'static str),
    UnexpectedArtifact(&'static str),
    OutputTooLarge(&'static str),
    InvalidMetadata(&'static str),
    IdentityMismatch,
    LiveSourceUnsupported,
    CaptionsUnavailable {
        requested: Option<String>,
        available: Vec<String>,
    },
    CaptionFormatUnavailable(String),
    InvalidCaptions(&'static str),
    Io {
        stage: &'static str,
        error: io::Error,
    },
    Compile(CompileError),
    Cleanup(io::Error),
}

impl fmt::Display for ProviderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSource(error) => error.fmt(formatter),
            Self::InvalidLanguage(language) => {
                write!(formatter, "caption language '{language}' is invalid")
            }
            Self::ToolUnavailable => formatter.write_str(
                "yt-dlp is not installed or is not available on the configured executable path",
            ),
            Self::TimedOut { stage, timeout } => {
                write!(
                    formatter,
                    "{stage} exceeded its {} second time limit",
                    timeout.as_secs()
                )
            }
            Self::Cancelled(stage) => write!(formatter, "{stage} was cancelled"),
            Self::ProcessFailed { stage, kind } => {
                let reason = match kind {
                    ProviderFailureKind::SourceUnavailable => "the source is unavailable",
                    ProviderFailureKind::SourcePrivate => "the source is private",
                    ProviderFailureKind::AuthenticationRequired => {
                        "the source requires authentication"
                    }
                    ProviderFailureKind::ProviderFailure => "the source provider failed",
                };
                write!(formatter, "{stage} failed: {reason}")
            }
            Self::ArtifactMissing(artifact) => {
                write!(formatter, "yt-dlp did not produce the expected {artifact}")
            }
            Self::UnexpectedArtifact(artifact) => {
                write!(formatter, "yt-dlp produced an unexpected {artifact}")
            }
            Self::OutputTooLarge(artifact) => {
                write!(formatter, "yt-dlp {artifact} exceeded the safety limit")
            }
            Self::InvalidMetadata(reason) => write!(formatter, "yt-dlp metadata {reason}"),
            Self::IdentityMismatch => {
                formatter.write_str("yt-dlp returned metadata for a different source")
            }
            Self::LiveSourceUnsupported => {
                formatter.write_str("live and upcoming sources are not supported yet")
            }
            Self::CaptionsUnavailable {
                requested,
                available,
            } => {
                if let Some(requested) = requested {
                    write!(
                        formatter,
                        "captions in '{requested}' are unavailable; available languages: {}",
                        available.join(", ")
                    )
                } else {
                    formatter.write_str("the source has no supported captions")
                }
            }
            Self::CaptionFormatUnavailable(language) => {
                write!(
                    formatter,
                    "captions in '{language}' are available but not in the required JSON3 format"
                )
            }
            Self::InvalidCaptions(reason) => write!(formatter, "yt-dlp captions {reason}"),
            Self::Io { stage, error } => write!(formatter, "{stage} failed: {error}"),
            Self::Compile(error) => error.fmt(formatter),
            Self::Cleanup(error) => write!(formatter, "temporary source cleanup failed: {error}"),
        }
    }
}

impl Error for ProviderError {}

#[derive(Clone, Debug)]
pub struct YtDlpProvider {
    executable: PathBuf,
}

#[derive(Clone, Debug, Default)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }
}

#[derive(Clone, Debug)]
pub struct AcquisitionControl {
    stage_timeout: Duration,
    cancellation: CancellationToken,
}

impl Default for AcquisitionControl {
    fn default() -> Self {
        Self {
            stage_timeout: DEFAULT_STAGE_TIMEOUT,
            cancellation: CancellationToken::new(),
        }
    }
}

impl AcquisitionControl {
    #[must_use]
    pub fn with_stage_timeout(stage_timeout: Duration) -> Self {
        Self {
            stage_timeout,
            cancellation: CancellationToken::new(),
        }
    }

    #[must_use]
    pub fn cancellation_token(&self) -> CancellationToken {
        self.cancellation.clone()
    }
}

impl Default for YtDlpProvider {
    fn default() -> Self {
        Self::new("yt-dlp")
    }
}

impl YtDlpProvider {
    #[must_use]
    pub fn new(executable: impl Into<PathBuf>) -> Self {
        Self {
            executable: executable.into(),
        }
    }

    /// Acquires and compiles one caption track from a supported live source.
    ///
    /// # Errors
    ///
    /// Returns [`ProviderError`] when the source, requested language, provider
    /// process, metadata, caption track or temporary cleanup fails validation.
    pub fn ingest(
        &self,
        input: &str,
        preferred_language: Option<&str>,
    ) -> Result<CompiledSource, ProviderError> {
        self.ingest_with_control(input, preferred_language, &AcquisitionControl::default())
    }

    /// Acquires one caption track with a bounded provider process and caller cancellation.
    ///
    /// # Errors
    ///
    /// Returns [`ProviderError`] under the same conditions as [`Self::ingest`],
    /// and when either provider stage times out or is cancelled.
    pub fn ingest_with_control(
        &self,
        input: &str,
        preferred_language: Option<&str>,
        control: &AcquisitionControl,
    ) -> Result<CompiledSource, ProviderError> {
        let source = canonicalise_source(input).map_err(ProviderError::InvalidSource)?;
        let preferred_language = preferred_language
            .map(validate_language)
            .transpose()?
            .map(str::to_owned);
        let workspace = Builder::new()
            .prefix("oriel-ytdlp-")
            .tempdir()
            .map_err(|error| ProviderError::Io {
                stage: "creating the temporary provider directory",
                error,
            })?;

        let result = self.ingest_in(
            &source,
            preferred_language.as_deref(),
            workspace.path(),
            control,
        );
        let cleanup = workspace.close();
        match (result, cleanup) {
            (Ok(compiled), Ok(())) => Ok(compiled),
            (Err(error), _) => Err(error),
            (Ok(_), Err(error)) => Err(ProviderError::Cleanup(error)),
        }
    }

    fn ingest_in(
        &self,
        source: &CanonicalSource,
        preferred_language: Option<&str>,
        workspace: &Path,
        control: &AcquisitionControl,
    ) -> Result<CompiledSource, ProviderError> {
        let metadata_path = workspace.join("metadata.json");
        let metadata_stderr = workspace.join("metadata.stderr");
        let metadata_args = metadata_arguments(&source.canonical_url);
        self.run_stage(
            "metadata acquisition",
            &metadata_args,
            &metadata_path,
            &metadata_stderr,
            control,
        )?;
        let metadata_bytes = read_limited(&metadata_path, MAX_METADATA_BYTES, "metadata output")?;
        let metadata = parse_metadata(&metadata_bytes, source)?;
        let selected = select_caption(&metadata, preferred_language)?;

        let caption_stdout = workspace.join("caption.stdout");
        let caption_stderr = workspace.join("caption.stderr");
        let caption_args = caption_arguments(source, &selected, workspace);
        self.run_stage(
            "caption acquisition",
            &caption_args,
            &caption_stdout,
            &caption_stderr,
            control,
        )?;
        let caption_path = find_caption_artifact(workspace)?;
        let caption_bytes = read_limited(&caption_path, MAX_CAPTION_BYTES, "caption output")?;
        let cues = parse_json3_passages(&caption_bytes)?;
        let duration_ms = duration_ms(metadata.duration)?;
        let caption_provenance = selected.provenance();
        let creator = metadata
            .channel
            .or(metadata.uploader)
            .filter(|value| !value.trim().is_empty())
            .ok_or(ProviderError::InvalidMetadata("does not contain a creator"))?;

        compile_source(AcquiredSource {
            source: source.clone(),
            metadata: SourceMetadata {
                title: metadata.title,
                creator,
                duration_ms,
            },
            transcript: TranscriptProvenance {
                language: selected.language,
                captions: caption_provenance,
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
                adapter: "yt_dlp".to_owned(),
                source_format: "youtube_json3".to_owned(),
                tool: metadata.version.map(|version| ToolProvenance {
                    name: "yt-dlp".to_owned(),
                    version: version.version,
                }),
            },
        })
        .map_err(ProviderError::Compile)
    }

    fn run_stage(
        &self,
        stage: &'static str,
        arguments: &[OsString],
        stdout_path: &Path,
        stderr_path: &Path,
        control: &AcquisitionControl,
    ) -> Result<(), ProviderError> {
        let stdout = File::create(stdout_path).map_err(|error| ProviderError::Io {
            stage: "creating provider output",
            error,
        })?;
        let stderr = File::create(stderr_path).map_err(|error| ProviderError::Io {
            stage: "creating provider diagnostics",
            error,
        })?;
        if control.cancellation.is_cancelled() {
            return Err(ProviderError::Cancelled(stage));
        }
        let mut child = Command::new(&self.executable)
            .args(arguments)
            .stdin(Stdio::null())
            .stdout(Stdio::from(stdout))
            .stderr(Stdio::from(stderr))
            .spawn()
            .map_err(|error| {
                if error.kind() == io::ErrorKind::NotFound {
                    ProviderError::ToolUnavailable
                } else {
                    ProviderError::Io {
                        stage: "starting yt-dlp",
                        error,
                    }
                }
            })?;

        let started = Instant::now();
        let status = loop {
            if control.cancellation.is_cancelled() {
                terminate_child(&mut child, stage)?;
                return Err(ProviderError::Cancelled(stage));
            }
            if started.elapsed() >= control.stage_timeout {
                terminate_child(&mut child, stage)?;
                return Err(ProviderError::TimedOut {
                    stage,
                    timeout: control.stage_timeout,
                });
            }
            if let Some(status) = child.try_wait().map_err(|error| ProviderError::Io {
                stage: "waiting for yt-dlp",
                error,
            })? {
                break status;
            }
            thread::sleep(PROCESS_POLL_INTERVAL);
        };

        if status.success() {
            return Ok(());
        }
        let diagnostics =
            read_limited(stderr_path, MAX_STDERR_BYTES, "diagnostics").unwrap_or_default();
        Err(ProviderError::ProcessFailed {
            stage,
            kind: classify_failure(&diagnostics),
        })
    }
}

fn terminate_child(
    child: &mut std::process::Child,
    stage: &'static str,
) -> Result<(), ProviderError> {
    let already_exited = child.try_wait().map_err(|error| ProviderError::Io {
        stage: "checking yt-dlp before termination",
        error,
    })?;
    if already_exited.is_none() {
        child.kill().map_err(|error| ProviderError::Io {
            stage: "terminating yt-dlp",
            error,
        })?;
    }
    child
        .wait()
        .map_err(|error| ProviderError::Io { stage, error })?;
    Ok(())
}

#[derive(Deserialize, Serialize)]
struct YtDlpMetadata {
    id: String,
    title: String,
    uploader: Option<String>,
    channel: Option<String>,
    duration: Option<f64>,
    webpage_url: String,
    availability: Option<String>,
    live_status: Option<String>,
    #[serde(default)]
    subtitles: BTreeMap<String, Vec<YtDlpTrack>>,
    #[serde(default)]
    automatic_captions: BTreeMap<String, Vec<YtDlpTrack>>,
    #[serde(rename = "_version")]
    version: Option<YtDlpVersion>,
}

#[derive(Deserialize, Serialize)]
struct YtDlpTrack {
    ext: String,
}

#[derive(Deserialize, Serialize)]
struct YtDlpVersion {
    version: String,
}

struct SelectedCaption {
    language: String,
    kind: SelectedCaptionKind,
}

#[derive(Clone, Copy)]
enum SelectedCaptionKind {
    Manual,
    Generated,
}

impl SelectedCaption {
    fn provenance(&self) -> CaptionProvenance {
        match self.kind {
            SelectedCaptionKind::Manual => CaptionProvenance::Manual,
            SelectedCaptionKind::Generated => CaptionProvenance::Generated,
        }
    }
}

#[derive(Deserialize)]
struct Json3Caption {
    #[serde(default)]
    events: Vec<Json3Event>,
}

#[derive(Deserialize)]
struct Json3Event {
    #[serde(rename = "tStartMs")]
    start_ms: Option<u64>,
    #[serde(rename = "dDurationMs")]
    duration_ms: Option<u64>,
    #[serde(rename = "aAppend")]
    append: Option<u8>,
    segs: Option<Vec<Json3Segment>>,
}

#[derive(Deserialize)]
struct Json3Segment {
    utf8: String,
}

fn metadata_arguments(canonical_url: &str) -> Vec<OsString> {
    let mut arguments = base_arguments();
    arguments.extend([
        OsString::from("--dump-single-json"),
        OsString::from("--"),
        OsString::from(canonical_url),
    ]);
    arguments
}

fn caption_arguments(
    source: &CanonicalSource,
    selected: &SelectedCaption,
    workspace: &Path,
) -> Vec<OsString> {
    let mut arguments = base_arguments();
    arguments.extend([
        OsString::from("--no-progress"),
        OsString::from("--no-overwrites"),
        OsString::from(match selected.kind {
            SelectedCaptionKind::Manual => "--write-subs",
            SelectedCaptionKind::Generated => "--write-auto-subs",
        }),
        OsString::from("--sub-langs"),
        OsString::from(format!("^{}$", selected.language)),
        OsString::from("--sub-format"),
        OsString::from("json3"),
        OsString::from("--output"),
        OsString::from(format!("subtitle:{}/caption.%(ext)s", workspace.display())),
        OsString::from("--"),
        OsString::from(&source.canonical_url),
    ]);
    arguments
}

fn base_arguments() -> Vec<OsString> {
    [
        "--ignore-config",
        "--no-plugin-dirs",
        "--no-remote-components",
        "--no-cache-dir",
        "--no-cookies",
        "--no-cookies-from-browser",
        "--color",
        "never",
        "--no-playlist",
        "--abort-on-error",
        "--skip-download",
    ]
    .into_iter()
    .map(OsString::from)
    .collect()
}

fn parse_metadata(
    input: &[u8],
    expected: &CanonicalSource,
) -> Result<YtDlpMetadata, ProviderError> {
    let metadata: YtDlpMetadata = serde_json::from_slice(input)
        .map_err(|_| ProviderError::InvalidMetadata("is malformed"))?;
    let returned = canonicalise_source(&metadata.webpage_url)
        .map_err(|_| ProviderError::InvalidMetadata("contains an invalid canonical URL"))?;
    if metadata.id != expected.source_id || returned != *expected {
        return Err(ProviderError::IdentityMismatch);
    }
    if metadata.title.trim().is_empty() {
        return Err(ProviderError::InvalidMetadata("does not contain a title"));
    }
    match metadata.availability.as_deref() {
        Some("private") => {
            return Err(ProviderError::ProcessFailed {
                stage: "metadata acquisition",
                kind: ProviderFailureKind::SourcePrivate,
            });
        }
        Some("needs_auth" | "premium_only" | "subscriber_only") => {
            return Err(ProviderError::ProcessFailed {
                stage: "metadata acquisition",
                kind: ProviderFailureKind::AuthenticationRequired,
            });
        }
        _ => {}
    }
    if !matches!(metadata.live_status.as_deref(), None | Some("not_live")) {
        return Err(ProviderError::LiveSourceUnsupported);
    }
    Ok(metadata)
}

fn select_caption(
    metadata: &YtDlpMetadata,
    preferred_language: Option<&str>,
) -> Result<SelectedCaption, ProviderError> {
    if let Some(language) = preferred_language {
        if let Some(selected) = select_exact(metadata, language)? {
            return Ok(selected);
        }
        return Err(ProviderError::CaptionsUnavailable {
            requested: Some(language.to_owned()),
            available: available_languages(metadata),
        });
    }

    if let Some(selected) = select_exact(metadata, "en")? {
        return Ok(selected);
    }
    for language in metadata.subtitles.keys() {
        if let Some(selected) = select_exact(metadata, language)? {
            return Ok(selected);
        }
    }
    for language in metadata.automatic_captions.keys() {
        if let Some(selected) = select_exact(metadata, language)? {
            return Ok(selected);
        }
    }
    Err(ProviderError::CaptionsUnavailable {
        requested: None,
        available: Vec::new(),
    })
}

fn select_exact(
    metadata: &YtDlpMetadata,
    language: &str,
) -> Result<Option<SelectedCaption>, ProviderError> {
    if let Some(tracks) = metadata.subtitles.get(language) {
        require_json3(tracks, language)?;
        return Ok(Some(SelectedCaption {
            language: language.to_owned(),
            kind: SelectedCaptionKind::Manual,
        }));
    }
    if let Some(tracks) = metadata.automatic_captions.get(language) {
        require_json3(tracks, language)?;
        return Ok(Some(SelectedCaption {
            language: language.to_owned(),
            kind: SelectedCaptionKind::Generated,
        }));
    }
    Ok(None)
}

fn require_json3(tracks: &[YtDlpTrack], language: &str) -> Result<(), ProviderError> {
    if tracks.iter().any(|track| track.ext == "json3") {
        Ok(())
    } else {
        Err(ProviderError::CaptionFormatUnavailable(language.to_owned()))
    }
}

fn available_languages(metadata: &YtDlpMetadata) -> Vec<String> {
    metadata
        .subtitles
        .keys()
        .chain(metadata.automatic_captions.keys())
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn duration_ms(duration: Option<f64>) -> Result<u64, ProviderError> {
    let duration = duration.ok_or(ProviderError::InvalidMetadata("has no duration"))?;
    let duration = Duration::try_from_secs_f64(duration)
        .map_err(|_| ProviderError::InvalidMetadata("has an invalid duration"))?;
    u64::try_from(duration.as_millis())
        .map_err(|_| ProviderError::InvalidMetadata("duration is too large"))
}

/// Parses JSON3 captions and normalises rolling caption fragments into passages.
fn parse_json3_passages(input: &[u8]) -> Result<Vec<AcquiredCue>, ProviderError> {
    Ok(segment_cues(parse_json3(input)?, PASSAGE_TARGET_MS))
}

fn parse_json3(input: &[u8]) -> Result<Vec<AcquiredCue>, ProviderError> {
    let caption: Json3Caption = serde_json::from_slice(input)
        .map_err(|_| ProviderError::InvalidCaptions("are malformed"))?;
    let mut cues: Vec<AcquiredCue> = Vec::new();

    for event in caption.events {
        let Some(segments) = event.segs else {
            continue;
        };
        let text = segments
            .into_iter()
            .map(|segment| segment.utf8)
            .collect::<String>();
        if text.trim().is_empty() {
            continue;
        }
        let start_ms = event.start_ms.ok_or(ProviderError::InvalidCaptions(
            "contain an untimed text event",
        ))?;
        let duration_ms = event.duration_ms.filter(|duration| *duration > 0).ok_or(
            ProviderError::InvalidCaptions("contain a text event without duration"),
        )?;
        let end_ms = start_ms
            .checked_add(duration_ms)
            .ok_or(ProviderError::InvalidCaptions(
                "contain a timestamp overflow",
            ))?;

        if event.append == Some(1) {
            let previous = cues.last_mut().ok_or(ProviderError::InvalidCaptions(
                "begin with an append-only text event",
            ))?;
            previous.text.push_str(&text);
            previous.end_ms = previous.end_ms.max(end_ms);
        } else {
            cues.push(AcquiredCue {
                start_ms,
                end_ms,
                text,
            });
        }
    }

    if cues.is_empty() {
        return Err(ProviderError::InvalidCaptions("contain no text evidence"));
    }
    for cue in &mut cues {
        cue.text = cue.text.trim().to_owned();
    }
    Ok(cues)
}

fn validate_language(language: &str) -> Result<&str, ProviderError> {
    if language.is_empty()
        || language.len() > MAX_LANGUAGE_LENGTH
        || !language
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err(ProviderError::InvalidLanguage(language.to_owned()));
    }
    Ok(language)
}

fn find_caption_artifact(workspace: &Path) -> Result<PathBuf, ProviderError> {
    let mut matches = Vec::new();
    let entries = fs::read_dir(workspace).map_err(|error| ProviderError::Io {
        stage: "inspecting caption output",
        error,
    })?;
    for entry in entries {
        let entry = entry.map_err(|error| ProviderError::Io {
            stage: "inspecting caption output",
            error,
        })?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with("caption.") && name.ends_with(".json3") {
            let metadata =
                fs::symlink_metadata(entry.path()).map_err(|error| ProviderError::Io {
                    stage: "inspecting caption output",
                    error,
                })?;
            if metadata.file_type().is_file() && !metadata.file_type().is_symlink() {
                matches.push(entry.path());
            }
        }
    }
    match matches.len() {
        0 => Err(ProviderError::ArtifactMissing("JSON3 caption file")),
        1 => Ok(matches.remove(0)),
        _ => Err(ProviderError::UnexpectedArtifact(
            "set of JSON3 caption files",
        )),
    }
}

fn read_limited(path: &Path, limit: u64, artifact: &'static str) -> Result<Vec<u8>, ProviderError> {
    let metadata = fs::symlink_metadata(path).map_err(|error| ProviderError::Io {
        stage: "reading provider output",
        error,
    })?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err(ProviderError::UnexpectedArtifact(artifact));
    }
    if metadata.len() > limit {
        return Err(ProviderError::OutputTooLarge(artifact));
    }
    fs::read(path).map_err(|error| ProviderError::Io {
        stage: "reading provider output",
        error,
    })
}

fn classify_failure(stderr: &[u8]) -> ProviderFailureKind {
    let diagnostics = String::from_utf8_lossy(stderr).to_ascii_lowercase();
    if diagnostics.contains("private video") || diagnostics.contains("video is private") {
        ProviderFailureKind::SourcePrivate
    } else if diagnostics.contains("sign in")
        || diagnostics.contains("authentication")
        || diagnostics.contains("login")
    {
        ProviderFailureKind::AuthenticationRequired
    } else if diagnostics.contains("video unavailable")
        || diagnostics.contains("source is unavailable")
    {
        ProviderFailureKind::SourceUnavailable
    } else {
        ProviderFailureKind::ProviderFailure
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeMap,
        ffi::OsString,
        path::Path,
        thread,
        time::{Duration, Instant},
    };

    use crate::{
        evidence::CaptionProvenance,
        source::{CanonicalSource, canonicalise_source},
    };

    use super::{
        AcquisitionControl, ProviderError, ProviderFailureKind, SelectedCaption,
        SelectedCaptionKind, YtDlpMetadata, YtDlpProvider, YtDlpTrack, caption_arguments,
        classify_failure, metadata_arguments, parse_json3, parse_json3_passages, parse_metadata,
        select_caption, validate_language,
    };

    fn source() -> CanonicalSource {
        canonicalise_source("https://youtu.be/dQw4w9WgXcQ").expect("source should be valid")
    }

    fn metadata() -> YtDlpMetadata {
        serde_json::from_str(
            r#"{
                "id":"dQw4w9WgXcQ",
                "title":"Evidence source",
                "uploader":"Oriel",
                "channel":"Oriel channel",
                "duration":10.5,
                "webpage_url":"https://www.youtube.com/watch?v=dQw4w9WgXcQ",
                "availability":"public",
                "live_status":"not_live",
                "subtitles":{"en":[{"ext":"json3"}]},
                "automatic_captions":{"en":[{"ext":"json3"}],"fr":[{"ext":"json3"}]},
                "_version":{"version":"2026.07.04"}
            }"#,
        )
        .expect("metadata should parse")
    }

    #[test]
    fn metadata_identity_is_verified() {
        let serialised = serde_json::to_vec(&metadata()).expect("metadata should serialise");
        let parsed = parse_metadata(&serialised, &source()).expect("metadata should validate");
        assert_eq!(parsed.id, "dQw4w9WgXcQ");

        let wrong = canonicalise_source("https://youtu.be/Ori3lDemo01")
            .expect("other source should be valid");
        assert!(matches!(
            parse_metadata(&serialised, &wrong),
            Err(ProviderError::IdentityMismatch)
        ));
    }

    #[test]
    fn exact_manual_caption_wins_over_generated() {
        let selected = select_caption(&metadata(), Some("en")).expect("caption should resolve");
        assert_eq!(selected.language, "en");
        assert_eq!(selected.provenance(), CaptionProvenance::Manual);
    }

    #[test]
    fn unavailable_and_non_json_tracks_are_distinct() {
        assert!(matches!(
            select_caption(&metadata(), Some("de")),
            Err(ProviderError::CaptionsUnavailable { .. })
        ));

        let mut metadata = metadata();
        metadata.subtitles = BTreeMap::from([(
            "de".to_owned(),
            vec![YtDlpTrack {
                ext: "vtt".to_owned(),
            }],
        )]);
        assert!(matches!(
            select_caption(&metadata, Some("de")),
            Err(ProviderError::CaptionFormatUnavailable(_))
        ));
    }

    #[test]
    fn json3_preserves_timing_and_concatenates_segments() {
        let cues = parse_json3(
            br#"{"events":[
                {"tStartMs":1000,"dDurationMs":2000,"segs":[{"utf8":"hello "},{"utf8":"world"}]},
                {"tStartMs":3000,"dDurationMs":1000},
                {"tStartMs":4000,"dDurationMs":2000,"segs":[{"utf8":"next"}]}
            ]}"#,
        )
        .expect("captions should parse");
        assert_eq!(cues.len(), 2);
        assert_eq!(cues[0].start_ms, 1_000);
        assert_eq!(cues[0].end_ms, 3_000);
        assert_eq!(cues[0].text, "hello world");
    }

    #[test]
    fn rolling_caption_fragments_become_readable_passages() {
        let passages = parse_json3_passages(
            br#"{"events":[
                {"tStartMs":0,"dDurationMs":12000,"segs":[{"utf8":"There's a deceptively simple problem"}]},
                {"tStartMs":10000,"dDurationMs":12000,"segs":[{"utf8":"that's tormented mathematicians for 50"}]},
                {"tStartMs":20000,"dDurationMs":12000,"segs":[{"utf8":"years. Suppose you have a needle."}]},
                {"tStartMs":30000,"dDurationMs":12000,"segs":[{"utf8":"What's the smallest area you can sweep?"}]}
            ]}"#,
        )
        .expect("captions should parse");

        assert_eq!(passages.len(), 2);
        assert_eq!(passages[0].start_ms, 0);
        assert_eq!(passages[0].end_ms, 32_000);
        assert_eq!(
            passages[0].text,
            "There's a deceptively simple problem that's tormented mathematicians for 50 \
years. Suppose you have a needle."
        );
        assert_eq!(passages[1].start_ms, 30_000);
        assert_eq!(passages[1].text, "What's the smallest area you can sweep?");
    }

    #[test]
    fn json3_append_events_extend_the_previous_cue() {
        let cues = parse_json3(
            br#"{"events":[
                {"tStartMs":1000,"dDurationMs":1000,"segs":[{"utf8":"hello "}]},
                {"tStartMs":2000,"dDurationMs":1000,"aAppend":1,"segs":[{"utf8":"world"}]}
            ]}"#,
        )
        .expect("captions should parse");
        assert_eq!(cues.len(), 1);
        assert_eq!(cues[0].text, "hello world");
        assert_eq!(cues[0].end_ms, 3_000);
    }

    #[test]
    fn arguments_disable_ambient_state_and_end_options() {
        let metadata = metadata_arguments(&source().canonical_url);
        assert!(metadata.contains(&"--ignore-config".into()));
        assert!(metadata.contains(&"--no-plugin-dirs".into()));
        assert!(metadata.contains(&"--no-cookies".into()));
        assert_eq!(metadata[metadata.len() - 2], "--");

        let selected = SelectedCaption {
            language: "en".to_owned(),
            kind: SelectedCaptionKind::Manual,
        };
        let captions = caption_arguments(&source(), &selected, Path::new("/tmp/oriel-test"));
        assert!(captions.contains(&"--write-subs".into()));
        assert!(captions.contains(&"^en$".into()));
        assert_eq!(captions[captions.len() - 2], "--");
    }

    #[test]
    fn language_and_failure_classification_are_bounded() {
        assert!(validate_language("zh-Hans").is_ok());
        assert!(validate_language("en.*").is_err());
        assert_eq!(
            classify_failure(b"ERROR: Private video. Sign in"),
            ProviderFailureKind::SourcePrivate
        );
        assert_eq!(
            classify_failure(b"ERROR: Video unavailable"),
            ProviderFailureKind::SourceUnavailable
        );
    }

    #[cfg(unix)]
    #[test]
    fn provider_process_is_terminated_at_the_stage_timeout() {
        let workspace = tempfile::tempdir().expect("workspace should be created");
        let provider = YtDlpProvider::new("/bin/sleep");
        let control = AcquisitionControl::with_stage_timeout(Duration::from_millis(40));
        let started = Instant::now();

        let error = provider
            .run_stage(
                "test acquisition",
                &[OsString::from("5")],
                &workspace.path().join("stdout"),
                &workspace.path().join("stderr"),
                &control,
            )
            .expect_err("the provider should time out");

        assert!(matches!(error, ProviderError::TimedOut { .. }));
        assert!(started.elapsed() < Duration::from_secs(1));
    }

    #[cfg(unix)]
    #[test]
    fn provider_process_can_be_cancelled_and_reaped() {
        let workspace = tempfile::tempdir().expect("workspace should be created");
        let provider = YtDlpProvider::new("/bin/sleep");
        let control = AcquisitionControl::default();
        let cancellation = control.cancellation_token();
        let canceller = thread::spawn(move || {
            thread::sleep(Duration::from_millis(40));
            cancellation.cancel();
        });

        let error = provider
            .run_stage(
                "test acquisition",
                &[OsString::from("5")],
                &workspace.path().join("stdout"),
                &workspace.path().join("stderr"),
                &control,
            )
            .expect_err("the provider should be cancelled");
        canceller.join().expect("canceller should finish");

        assert!(matches!(
            error,
            ProviderError::Cancelled("test acquisition")
        ));
    }
}
