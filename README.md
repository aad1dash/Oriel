# Oriel

Oriel is an early local-first source-intelligence engine. It is being built to turn long-form video into compact, timestamp-grounded evidence that humans and agents can reuse.

The current vertical slice accepts a YouTube URL, acquires one caption track, compiles timestamped evidence, reuses an inspectable local cache and returns ranked moments. Oriel retrieves evidence; it does not add an internal model call to answer the question.

## Run the current slice

Rust 1.97.1 is pinned in `rust-toolchain.toml`. Live acquisition also requires `yt-dlp`; Homebrew's package includes the JavaScript runtime currently needed by YouTube extraction.

```sh
brew install yt-dlp
```

```sh
cargo run -- resolve 'https://youtu.be/Ori3lDemo01?si=tracking'
```

Search a live captioned source and keep its compiled evidence locally:

```sh
cargo run -- search \
  --source 'https://www.youtube.com/watch?v=VIDEO_ID' \
  --language en \
  --cache-dir .oriel-cache \
  --query 'Where does the speaker change the invalidation logic?'
```

The first request reports a cache `miss`; a repeated request loads the immutable source version without contacting the provider and reports `hit`. To reacquire the source and determine whether the compiled evidence changed:

```sh
cargo run -- search \
  --source 'https://www.youtube.com/watch?v=VIDEO_ID' \
  --language en \
  --cache-dir .oriel-cache \
  --refresh \
  --query 'What changed?'
```

The cache stores content-addressed, schema-versioned JSON. It does not retain raw metadata, signed caption URLs, audio or video.

Each provider stage has a 90-second deadline. The engine also exposes a cancellation token for future long-running surfaces; either condition terminates and reaps the provider process before temporary files are removed.

The deterministic fixture path remains available offline:

```sh
cargo run -- search \
  --fixture fixtures/synthetic/captioned-explainer-v1.tsv \
  --query 'Why does retrieval begin with lexical search?'
```

The search command returns a compact JSON evidence packet containing source identity, coverage, acquisition and cache provenance, ranked moments, timestamps and warnings.

## Verify

```sh
cargo fmt --check
cargo clippy --all-targets --all-features --offline -- -D warnings
cargo test --offline
cargo bench --bench retrieval_latency --offline
```

Future TypeScript product surfaces use Bun. No pnpm project is present.

## Current limitations

- Live acquisition currently supports YouTube captions available as JSON3 through `yt-dlp`.
- Refresh is explicit; a warm cache hit does not contact YouTube to detect changes.
- Retrieval evaluation is still synthetic and does not establish the release recall target.
- MCP, the web application, transcription and visual evidence are not implemented.

The founding product brief is in [`plans/spec1.md`](plans/spec1.md). Current implementation state is in [`status.md`](status.md).
