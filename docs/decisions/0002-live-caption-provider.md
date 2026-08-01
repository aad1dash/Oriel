# 0002: Acquire YouTube captions through a constrained yt-dlp adapter

**Status:** Accepted, 2026-08-01

## Context

The deterministic fixture path proved Oriel's evidence contract but could not make a real source useful. The next boundary had to acquire current YouTube metadata and captions without downloading media, inheriting user credentials or allowing provider-specific data to leak into retrieval.

YouTube extraction changes frequently and currently requires JavaScript challenge support. Reimplementing that access layer would create a large, brittle platform client unrelated to Oriel's source-intelligence advantage.

## Decision

Use the locally installed `yt-dlp` executable behind one synchronous process adapter.

The adapter:

- accepts only Oriel-canonicalised YouTube URLs;
- invokes an argument array and terminates options with `--`;
- disables ambient configuration, plugins, remote components, cookies and global cache;
- retrieves metadata before selecting an exact caption language;
- prefers a manual track over a generated track for the same language;
- requires JSON3 and verifies the output format after acquisition;
- bounds metadata, diagnostics and caption sizes;
- verifies returned source identity;
- minimises raw metadata immediately and never persists signed caption URLs;
- uses a private temporary directory and reports cleanup failure;
- records tool, format, language and caption provenance separately from source evidence.

Both fixtures and live acquisition now enter the same pure compiler. The compiler validates evidence and produces a framed SHA-256 semantic version. Tool version, temporary paths and signed URLs do not invalidate equivalent evidence.

Persist compiled sources as schema-versioned, content-addressed JSON. Version files are immutable, language pointers advance atomically and every cache read revalidates identifiers, timestamps, provenance and the semantic hash. Cache refresh explicitly reports whether the source evidence changed.

## Evidence

- `yt-dlp 2026.07.04` acquired metadata and one manual English JSON3 caption track from a public source without audio or video.
- The production CLI completed URL → metadata → captions → compiled evidence → lexical retrieval and returned the expected moment at 22.64 seconds.
- A first cached request reported `miss`; an identical second request succeeded inside the network-restricted sandbox and reported `hit`.
- An explicit reacquisition of unchanged evidence reported `refreshed` with `source_changed: false`.
- Cache version and pointer directories were mode `0700` in the smoke run.
- Strict Clippy and all default offline tests pass; live access is not part of the default test suite.
- The [official yt-dlp options](https://github.com/yt-dlp/yt-dlp/blob/master/README.md) distinguish manual and generated captions and support related-file acquisition without media.

## Alternatives

### Direct subtitle URL download

Rejected for now. It would require persisting or immediately consuming signed capability URLs and adding an HTTP client while still depending on yt-dlp for extraction.

### WebVTT

Deferred. JSON3 already uses the required JSON parser and exposes integer millisecond timing. Available Rust WebVTT parsers do not yet justify another dependency or a bespoke partial parser.

### Internal YouTube extractor

Rejected. It would reproduce volatile signature, client and challenge behaviour with worse maintenance and no source-intelligence benefit.

### SQLite or a search index

Not required for current latency. An optimised synthetic benchmark over 10,000 cues and 500 warm queries measured p50 2.908 ms, p95 3.098 ms and p99 3.172 ms on arm64 macOS 26.6. This establishes latency only, not recall.

## Consequences

- `yt-dlp` and a supported JavaScript runtime are required for live YouTube use but not for cached or fixture-backed retrieval.
- JSON3 is provider input, not an engine-wide format contract.
- Refresh reacquires the source; ordinary cache hits deliberately make no network request.
- The initial process adapter has bounded files but no timeout or cancellation yet.
- Transcript-neutral video edits cannot be detected until visual evidence exists.
- Source and caption rights remain separate from the yt-dlp software licence.

## Revisit when

- provider timeouts or cancellation become necessary for real long-running sources;
- JSON3 becomes unavailable or demonstrates evidence-loss cases;
- an authorised evaluation corpus shows lexical recall failures;
- measured warm retrieval p95 approaches 100 milliseconds;
- another source provider needs the acquisition boundary.
