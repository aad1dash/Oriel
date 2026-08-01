# Status

**2026-08-01 — IN PROGRESS: Stage 0/1 foundation**

## Works

- Common YouTube URL forms resolve to one validated source identity.
- Synthetic captions compile into immutable timestamped evidence with language, provenance, version and coverage.
- Deterministic lexical retrieval supports natural questions, timestamp filters and absent answers.
- The CLI returns compact JSON evidence packets with clickable timestamps and honest visual-coverage warnings.

## Evidence

- Rust 1.97.1; standard-library-only engine.
- Formatting and strict Clippy pass.
- 16 tests pass offline.
- Synthetic retrieval corpus: 5/5 expected outcomes in the top five, including one negative case. This is infrastructure proof, not the product recall claim.

## Not yet supported

- Live metadata or caption acquisition, cache persistence, a long-source benchmark, MCP, web, transcription or visuals.
- The fixture hash is deterministic test identity, not a collision-resistant live cache key.

## Next

Run a bounded live-source experiment behind a `yt-dlp` provider, then add collision-resistant source versioning and an inspectable cache before broadening retrieval.

No interface design has been selected. Web design requires founder consultation, external Claude Opus work and served HTML directions before implementation.
