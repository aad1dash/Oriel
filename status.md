# Status

**2026-08-01 — IN PROGRESS: Stage 1 evidence engine**

## Works

- Common YouTube URL forms resolve to one validated source identity.
- Live `yt-dlp` metadata and manual or generated JSON3 captions compile through the same evidence engine as fixtures.
- Compiled evidence retains language, caption and acquisition provenance, SHA-256 source version and honest coverage.
- Deterministic lexical retrieval supports natural questions, timestamp filters and absent answers.
- The CLI accepts a live URL or fixture and returns compact evidence packets with clickable timestamps.
- An explicit local cache stores immutable, inspectable JSON versions and serves repeated questions without provider access.
- `--refresh` reacquires a cached source and reports whether compiled evidence changed.

## Evidence

- Rust 1.97.1; Bun 1.3.14 reserved for future TypeScript surfaces; `yt-dlp 2026.07.04` locally verified.
- Formatting and strict Clippy pass.
- 30 tests pass offline; live access is excluded from defaults.
- Synthetic retrieval corpus: 5/5 expected outcomes in the top five, including one negative case. This is infrastructure proof, not the product recall claim.
- Bounded live smoke: URL → manual English captions → expected evidence at 22.64 seconds; no media downloaded.
- Cache smoke: first request `miss`; identical network-restricted request `hit`; private directories were mode `0700`.
- Refresh smoke: unchanged reacquisition reported `source_changed: false` against the prior semantic version.
- Release benchmark, arm64 macOS 26.6, 10,000 synthetic cues, 500 warm runs: p50 2.908 ms, p95 3.098 ms, p99 3.172 ms. This is latency evidence, not retrieval-quality evidence.

## Not yet supported

- Provider timeout and cancellation; authenticated, private, live or age-restricted sources.
- Representative authorised retrieval corpus and the 90% recall claim.
- Progressive acquisition status, MCP, web, local transcription or visual evidence.
- Automatic change checks on warm hits; refresh is explicit.

## Next

Build a representative authorised retrieval corpus and test segmentation/recall before adding an index. In parallel, add provider timeout and cancellation before exposing ingestion through a long-running server.

No interface design has been selected. Web design requires founder consultation, external Claude Opus work and served HTML directions before implementation.
