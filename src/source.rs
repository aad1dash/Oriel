use std::{error::Error, fmt};

const YOUTUBE_ID_LENGTH: usize = 11;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SourceProvider {
    YouTube,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalSource {
    pub provider: SourceProvider,
    pub source_id: String,
    pub canonical_url: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SourceError {
    MalformedUrl,
    UnsupportedSource,
    MissingVideoId,
}

impl fmt::Display for SourceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::MalformedUrl => "the source URL is malformed",
            Self::UnsupportedSource => "the source is not a supported YouTube URL",
            Self::MissingVideoId => "the YouTube URL does not contain a valid video ID",
        };
        formatter.write_str(message)
    }
}

impl Error for SourceError {}

/// Resolves a supported source URL to its stable identity.
///
/// # Errors
///
/// Returns [`SourceError`] when the input is malformed, uses an unsupported
/// source or does not contain a valid video identifier.
pub fn canonicalise_source(input: &str) -> Result<CanonicalSource, SourceError> {
    let input = input.trim();
    let (scheme, remainder) = input.split_once("://").ok_or(SourceError::MalformedUrl)?;
    if scheme != "https" && scheme != "http" {
        return Err(SourceError::UnsupportedSource);
    }

    let authority_end = remainder.find(['/', '?', '#']).unwrap_or(remainder.len());
    let host = remainder[..authority_end].to_ascii_lowercase();
    if host.is_empty() || host.contains('@') || host.contains(':') {
        return Err(SourceError::UnsupportedSource);
    }
    let location = &remainder[authority_end..];

    let source_id = match host.as_str() {
        "youtu.be" | "www.youtu.be" => first_path_segment(location).map(str::to_owned),
        "youtube.com" | "www.youtube.com" | "m.youtube.com" | "music.youtube.com" => {
            youtube_com_id(location)
        }
        _ => return Err(SourceError::UnsupportedSource),
    }
    .filter(|candidate| is_valid_youtube_id(candidate))
    .ok_or(SourceError::MissingVideoId)?;

    Ok(CanonicalSource {
        provider: SourceProvider::YouTube,
        canonical_url: format!("https://www.youtube.com/watch?v={source_id}"),
        source_id,
    })
}

fn youtube_com_id(location: &str) -> Option<String> {
    match first_path_segment(location) {
        Some("watch") => query_parameter(location, "v").map(str::to_owned),
        Some("shorts" | "embed" | "live") => path_segments(location).nth(1).map(str::to_owned),
        _ => None,
    }
}

fn first_path_segment(location: &str) -> Option<&str> {
    path_segments(location).next()
}

fn path_segments(location: &str) -> impl Iterator<Item = &str> {
    location
        .split(['?', '#'])
        .next()
        .unwrap_or_default()
        .split('/')
        .filter(|segment| !segment.is_empty())
}

fn query_parameter<'a>(location: &'a str, key: &str) -> Option<&'a str> {
    let query = location.split_once('?')?.1.split('#').next()?;
    query.split('&').find_map(|pair| {
        let (candidate, value) = pair.split_once('=')?;
        (candidate == key).then_some(value)
    })
}

fn is_valid_youtube_id(candidate: &str) -> bool {
    candidate.len() == YOUTUBE_ID_LENGTH
        && candidate
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

#[cfg(test)]
mod tests {
    use super::{CanonicalSource, SourceError, SourceProvider, canonicalise_source};

    const VIDEO_ID: &str = "dQw4w9WgXcQ";

    #[test]
    fn canonicalises_common_youtube_urls() {
        let inputs = [
            format!("https://www.youtube.com/watch?v={VIDEO_ID}&t=42s"),
            format!("https://youtu.be/{VIDEO_ID}?si=tracking"),
            format!("https://m.youtube.com/shorts/{VIDEO_ID}"),
            format!("https://www.youtube.com/embed/{VIDEO_ID}"),
            format!("https://youtube.com/live/{VIDEO_ID}?feature=share"),
        ];

        let expected = CanonicalSource {
            provider: SourceProvider::YouTube,
            source_id: VIDEO_ID.to_owned(),
            canonical_url: format!("https://www.youtube.com/watch?v={VIDEO_ID}"),
        };

        for input in inputs {
            assert_eq!(canonicalise_source(&input), Ok(expected.clone()));
        }
    }

    #[test]
    fn rejects_unsupported_and_malformed_sources() {
        assert_eq!(
            canonicalise_source("not a url"),
            Err(SourceError::MalformedUrl)
        );
        assert_eq!(
            canonicalise_source("https://example.com/watch?v=dQw4w9WgXcQ"),
            Err(SourceError::UnsupportedSource)
        );
        assert_eq!(
            canonicalise_source("file:///tmp/video"),
            Err(SourceError::UnsupportedSource)
        );
    }

    #[test]
    fn rejects_missing_or_invalid_video_ids() {
        let inputs = [
            "https://www.youtube.com/watch?list=playlist",
            "https://youtu.be/too-short",
            "https://www.youtube.com/shorts/dQw4w9WgXc!",
        ];

        for input in inputs {
            assert_eq!(canonicalise_source(input), Err(SourceError::MissingVideoId));
        }
    }
}
