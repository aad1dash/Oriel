use std::{fs, path::PathBuf};

use oriel::{fixture::compile_fixture, search::SearchQuery, search::search};

fn repository_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

#[test]
fn synthetic_retrieval_corpus_finds_expected_spans() {
    let fixture = fs::read_to_string(repository_path(
        "fixtures/synthetic/captioned-explainer-v1.tsv",
    ))
    .expect("fixture should be readable");
    let source = compile_fixture(&fixture).expect("fixture should compile");
    let evaluation = fs::read_to_string(repository_path("evals/retrieval-v1.tsv"))
        .expect("evaluation should be readable");

    let mut evaluated = 0;
    for (index, line) in evaluation.lines().enumerate() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let [kind, question, expected] = line
            .split('\t')
            .collect::<Vec<_>>()
            .try_into()
            .unwrap_or_else(|_| panic!("evaluation line {} is malformed", index + 1));
        let results = search(&source, &SearchQuery::new(question)).unwrap_or_else(|error| {
            panic!("evaluation '{kind}' could not run: {error}");
        });

        if expected == "absent" {
            assert!(results.is_empty(), "evaluation '{kind}' should be absent");
        } else {
            let expected_start = expected
                .parse::<u64>()
                .unwrap_or_else(|_| panic!("evaluation '{kind}' has an invalid timestamp"));
            assert!(
                results
                    .iter()
                    .take(5)
                    .any(|result| result.evidence.start_ms == expected_start),
                "evaluation '{kind}' missed {expected_start} ms in its top five"
            );
        }
        evaluated += 1;
    }

    assert_eq!(evaluated, 5, "the expected evaluation corpus did not run");
}
