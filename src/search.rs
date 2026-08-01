use std::{cmp::Reverse, collections::BTreeSet, error::Error, fmt};

use crate::evidence::{CompiledSource, Evidence};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchQuery {
    pub text: String,
    pub limit: usize,
    pub start_ms: Option<u64>,
    pub end_ms: Option<u64>,
}

impl SearchQuery {
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            limit: 5,
            start_ms: None,
            end_ms: None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetrievedEvidence<'a> {
    pub evidence: &'a Evidence,
    pub score: u32,
    pub matched_terms: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SearchError {
    EmptyQuery,
    ZeroLimit,
    InvalidTimestampRange,
}

impl fmt::Display for SearchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::EmptyQuery => "the search query contains no searchable terms",
            Self::ZeroLimit => "the search result limit must be greater than zero",
            Self::InvalidTimestampRange => "the search timestamp range is invalid",
        };
        formatter.write_str(message)
    }
}

impl Error for SearchError {}

/// Retrieves timestamped transcript evidence using deterministic lexical ranking.
///
/// # Errors
///
/// Returns [`SearchError`] when the query, limit or timestamp constraints are invalid.
pub fn search<'a>(
    source: &'a CompiledSource,
    query: &SearchQuery,
) -> Result<Vec<RetrievedEvidence<'a>>, SearchError> {
    if query.limit == 0 {
        return Err(SearchError::ZeroLimit);
    }
    if matches!((query.start_ms, query.end_ms), (Some(start), Some(end)) if start >= end) {
        return Err(SearchError::InvalidTimestampRange);
    }

    let terms = searchable_terms(&query.text);
    if terms.is_empty() {
        return Err(SearchError::EmptyQuery);
    }
    let normalised_query = normalise_phrase(&query.text);

    let mut results = source
        .evidence
        .iter()
        .filter(|evidence| is_within_range(evidence, query.start_ms, query.end_ms))
        .filter_map(|evidence| rank_evidence(evidence, &terms, &normalised_query))
        .collect::<Vec<_>>();

    results.sort_by_key(|result| (Reverse(result.score), result.evidence.start_ms));
    results.truncate(query.limit);
    Ok(results)
}

fn rank_evidence<'a>(
    evidence: &'a Evidence,
    query_terms: &[String],
    normalised_query: &str,
) -> Option<RetrievedEvidence<'a>> {
    let evidence_terms = tokenise(&evidence.text);
    let mut matched_terms = Vec::new();
    let mut score = 0_u32;

    for query_term in query_terms {
        let frequency = evidence_terms
            .iter()
            .filter(|term| *term == query_term)
            .count();
        if frequency > 0 {
            matched_terms.push(query_term.clone());
            score += 100 + u32::try_from(frequency.min(3)).unwrap_or(3) * 10;
        }
    }

    let minimum_matches = query_terms.len().div_ceil(3).max(1);
    if matched_terms.len() < minimum_matches {
        return None;
    }
    if matched_terms.len() == query_terms.len() {
        score += 75;
    }
    if normalised_query.split_whitespace().count() > 1
        && normalise_phrase(&evidence.text).contains(normalised_query)
    {
        score += 200;
    }

    Some(RetrievedEvidence {
        evidence,
        score,
        matched_terms,
    })
}

fn is_within_range(evidence: &Evidence, start_ms: Option<u64>, end_ms: Option<u64>) -> bool {
    start_ms.is_none_or(|start| evidence.end_ms > start)
        && end_ms.is_none_or(|end| evidence.start_ms < end)
}

fn searchable_terms(input: &str) -> Vec<String> {
    tokenise(input)
        .into_iter()
        .filter(|term| !is_stopword(term))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn tokenise(input: &str) -> Vec<String> {
    input
        .split(|character: char| !character.is_alphanumeric())
        .filter(|term| !term.is_empty())
        .map(str::to_lowercase)
        .collect()
}

fn normalise_phrase(input: &str) -> String {
    tokenise(input).join(" ")
}

fn is_stopword(term: &str) -> bool {
    matches!(
        term,
        "a" | "an"
            | "and"
            | "are"
            | "did"
            | "do"
            | "does"
            | "for"
            | "from"
            | "how"
            | "i"
            | "in"
            | "is"
            | "it"
            | "of"
            | "on"
            | "the"
            | "this"
            | "to"
            | "was"
            | "what"
            | "when"
            | "where"
            | "why"
            | "with"
    )
}

#[cfg(test)]
mod tests {
    use crate::{fixture::compile_fixture, search::SearchQuery};

    use super::{SearchError, search};

    const FIXTURE: &str = "schema_version\t1\n\
source_url\thttps://youtu.be/dQw4w9WgXcQ\n\
title\tRetrieval fixture\n\
creator\tOriel\n\
duration_ms\t30000\n\
language\ten\n\
caption_provenance\tgenerated\n\
cue\t0\t10000\tThe source states a claim without demonstrating it.\n\
cue\t10000\t20000\tA diagram demonstrates the cache invalidation pipeline.\n\
cue\t20000\t30000\tThe speaker corrects the earlier cache claim.\n";

    #[test]
    fn ranks_complete_and_phrase_matches_first() {
        let source = compile_fixture(FIXTURE).expect("fixture should compile");
        let results = search(&source, &SearchQuery::new("cache invalidation pipeline"))
            .expect("search should work");

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].evidence.start_ms, 10_000);
        assert_eq!(
            results[0].matched_terms,
            ["cache", "invalidation", "pipeline"]
        );
        assert!(results[0].score > results[1].score);
    }

    #[test]
    fn supports_natural_questions_and_timestamp_filters() {
        let source = compile_fixture(FIXTURE).expect("fixture should compile");
        let mut query = SearchQuery::new("Where does the speaker correct the cache claim?");
        query.start_ms = Some(20_000);
        query.end_ms = Some(30_000);

        let results = search(&source, &query).expect("search should work");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].evidence.start_ms, 20_000);
    }

    #[test]
    fn returns_no_evidence_for_an_absent_answer() {
        let source = compile_fixture(FIXTURE).expect("fixture should compile");
        let results =
            search(&source, &SearchQuery::new("anthropology")).expect("search should work");
        assert!(results.is_empty());
    }

    #[test]
    fn rejects_empty_queries_and_ranges() {
        let source = compile_fixture(FIXTURE).expect("fixture should compile");
        assert_eq!(
            search(&source, &SearchQuery::new("what is this?")),
            Err(SearchError::EmptyQuery)
        );

        let mut query = SearchQuery::new("cache");
        query.start_ms = Some(20_000);
        query.end_ms = Some(10_000);
        assert_eq!(
            search(&source, &query),
            Err(SearchError::InvalidTimestampRange)
        );
    }
}
