use std::{path::PathBuf, process::Command};

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/synthetic/captioned-explainer-v1.tsv")
}

#[test]
fn resolves_a_youtube_url_to_one_identity() {
    let output = Command::new(env!("CARGO_BIN_EXE_oriel"))
        .args(["resolve", "https://youtu.be/Ori3lDemo01?si=tracking-token"])
        .output()
        .expect("CLI should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert_eq!(
        stdout.trim(),
        r#"{"provider":"youtube","source_id":"Ori3lDemo01","canonical_url":"https://www.youtube.com/watch?v=Ori3lDemo01"}"#
    );
}

#[test]
fn returns_a_compact_timestamped_evidence_packet() {
    let output = Command::new(env!("CARGO_BIN_EXE_oriel"))
        .args(["search", "--fixture"])
        .arg(fixture_path())
        .args(["--query", "Why does retrieval begin with lexical search?"])
        .output()
        .expect("CLI should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains(r#""start_ms":20000"#));
    assert!(stdout.contains(r#""timestamp_label":"0:20""#));
    assert!(stdout.contains(r#""caption_provenance":"manual""#));
    assert!(stdout.contains(r#""warnings":["visuals_not_processed"]"#));
    assert!(!stdout.contains("full transcript"));
}

#[test]
fn reports_invalid_fixture_input_without_panicking() {
    let output = Command::new(env!("CARGO_BIN_EXE_oriel"))
        .args(["search", "--fixture", "missing.tsv", "--query", "evidence"])
        .output()
        .expect("CLI should run");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(stderr.contains("could not read fixture 'missing.tsv'"));
}

#[test]
fn help_is_a_success_at_the_root_and_for_each_command() {
    for arguments in [
        vec!["--help"],
        vec!["resolve", "--help"],
        vec!["search", "--help"],
        vec!["read", "--help"],
        vec!["mcp", "--help"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_oriel"))
            .args(arguments)
            .output()
            .expect("CLI should run");

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
        assert!(stdout.starts_with("Usage:\n"));
        assert!(output.stderr.is_empty());
    }
}
