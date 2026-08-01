# Status

**2026-08-01 — IN PROGRESS: evidence engine and first agent surface**

## Works

- Common YouTube URL forms resolve to one validated source identity.
- Live `yt-dlp` metadata and manual or generated JSON3 captions compile through the same evidence engine as fixtures.
- Compiled evidence retains language, caption and acquisition provenance, SHA-256 source version and honest coverage.
- Deterministic lexical retrieval supports natural questions, timestamp filters and absent answers.
- The CLI accepts a live URL or fixture and returns compact evidence packets with clickable timestamps.
- An explicit local cache stores immutable, inspectable JSON versions and serves repeated questions without provider access.
- `--refresh` reacquires a cached source and reports whether compiled evidence changed.
- Every provider stage has a 90-second deadline and a caller can cancel acquisition; either path terminates and reaps the child process before cleanup.
- A local MCP `stdio` server exposes one bounded `search_source` tool over the same engine and structured evidence packet as the CLI.

## Evidence

- Rust 1.97.1; Bun 1.3.14 reserved for future TypeScript surfaces; `yt-dlp 2026.07.04` locally verified.
- Formatting and strict Clippy pass.
- 38 tests pass offline; live access is excluded from defaults.
- Deterministic timeout and cancellation tests terminate a five-second child in under one second.
- A wire-level MCP test initialises the server, discovers one tool and retrieves the correct cached moment at 10 seconds without network access.
- Synthetic retrieval corpus: 5/5 expected outcomes in the top five, including one negative case. This is infrastructure proof, not the product recall claim.
- Bounded live smoke: URL → manual English captions → expected evidence at 22.64 seconds; no media downloaded.
- Cache smoke: first request `miss`; identical network-restricted request `hit`; private directories were mode `0700`.
- Refresh smoke: unchanged reacquisition reported `source_changed: false` against the prior semantic version.
- Release benchmark, arm64 macOS 26.6, 10,000 synthetic cues, 500 warm runs: p50 2.908 ms, p95 3.098 ms, p99 3.172 ms. This is latency evidence, not retrieval-quality evidence.

## Not yet supported

- Authenticated, private, live or age-restricted sources.
- Representative authorised retrieval corpus and the 90% recall claim.
- Progressive acquisition status, web, local transcription or visual evidence.
- Broader MCP status/context operations and a real unrelated-repository agent trial.
- Automatic change checks on warm hits; refresh is explicit.

## Next

Build a representative authorised retrieval corpus and test segmentation/recall before adding an index. In parallel, trial the one-tool MCP surface from an unrelated repository before adding more tools.

No interface design has been selected. Web design requires founder consultation, external Claude Opus work and served HTML directions before implementation.
