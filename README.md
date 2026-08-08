# Oriel

Oriel is a fast, local-first engine that turns long-form video into timestamp-grounded evidence for humans and agents.

Important ideas are often discussed in YouTube videos. I kept running into the same problem: I could talk about a video, but there was no easy way to give its argument to an agent quickly, efficiently and with receipts for what was actually said. Copying transcripts by hand was slow, and asking an agent to work from a title or summary lost the source. I couldn't find anything that could effortlessly and with great performance just work with my workflow so here it is. 

I learnt enough Rust to engineer the small tool I needed (Yes Codex was used). Oriel compiles a captioned video once, keeps the evidence locally, and lets an agent read the source or find a precise moment in milliseconds from a warm cache. As I find real shortcomings and receive feedback, I will keep improving the engine.

Oriel is early, useful and deliberately narrow. It retrieves evidence; the calling agent does the interpretation.

```text
YouTube URL -> captions -> timestamped local evidence -> calling agent judgement
```

## What works today

- `read_source` returns a complete caption track as ordered, individually timestamped passages.
- `search_source` retrieves a small set of timestamped moments for an exact question.
- CLI and MCP use the same Rust engine, cache and provenance model.
- The first request compiles a source; later requests reuse an inspectable, content-addressed local cache.
- Every result reports source identity, caption provenance, coverage, cache state and warnings.
- Oriel makes no internal model call and does not retain an agent's interpretations.

The current code is best described as a working v0.1 evidence engine, not a complete video-intelligence platform. See [Current limitations](#current-limitations) before relying on it.

## Install from source

### Prerequisites

- Git.
- Rust through [`rustup`](https://rustup.rs/). The repository pins Rust 1.97.1 and the required `rustfmt` and Clippy components.
- A current [`yt-dlp`](https://github.com/yt-dlp/yt-dlp#installation) installation that includes its EJS component, plus a supported JavaScript runtime. `yt-dlp` recommends Deno and enables it by default. Its [EJS guide](https://github.com/yt-dlp/yt-dlp/wiki/EJS) explains the supported installation combinations.

On macOS, Homebrew's `yt-dlp` formula currently includes Deno:

```sh
brew install yt-dlp
```

Then build and install Oriel:

```sh
git clone https://github.com/aad1dash/Oriel.git
cd Oriel
cargo install --path . --locked
```

`cargo install` places the binary in `~/.cargo/bin` by default. Ensure that directory is on your `PATH`, then check the CLI:

```sh
oriel --help
```

Oriel's measured live-source and performance results currently come from arm64 macOS. Its deterministic tests are platform-independent, but live acquisition on Linux and Windows has not yet been verified by the project.

## Try the CLI

Read a whole captioned video:

```sh
oriel read \
  --source 'https://www.youtube.com/watch?v=VIDEO_ID' \
  --language en \
  --cache-dir "$HOME/.local/share/oriel/cache"
```

Find a specific moment:

```sh
oriel search \
  --source 'https://www.youtube.com/watch?v=VIDEO_ID' \
  --language en \
  --cache-dir "$HOME/.local/share/oriel/cache" \
  --query 'Where does the speaker explain why the first approach failed?'
```

Both commands return structured JSON. Use `read` when you want the argument or lesson from an ordinary video. Use `search` when you genuinely need to locate a moment. Add `--refresh` to reacquire a source instead of using its latest cached version.

The repository also contains `./ask`, a small human-facing retrieval harness, and a deterministic fixture path for offline development:

```sh
cargo run --locked -- search \
  --fixture fixtures/synthetic/captioned-explainer-v1.tsv \
  --query 'Why does retrieval begin with lexical search?'
```

## Use Oriel from Codex

Register the installed binary once in Codex's user-level MCP configuration:

```sh
codex mcp add oriel -- "$HOME/.cargo/bin/oriel" mcp \
  --cache-dir "$HOME/.local/share/oriel/cache"
codex mcp get oriel
```

Restart the Codex desktop app after first registering it. Oriel will then be available from an ordinary task, including one with no repository attached. For example:

```text
Summarise this video clearly and cite the important moments: <YouTube URL>
```

Codex can call `read_source` for the whole argument or `search_source` for a precise moment. Oriel supplies the timestamped source evidence; Codex performs the summarisation and judgement.

The optional [`skills/oriel`](skills/oriel) skill adds a careful workflow for scouting, learning from, applying and finding moments in a source. It keeps source evidence distinct from agent interpretation and audits timestamp citations before answering. The skill is provided for inspection and is not installed automatically.

## Use Oriel from another MCP client

Configure any local MCP client that supports `stdio` to launch the release binary with an explicit cache directory:

```json
{
  "mcpServers": {
    "oriel": {
      "command": "/absolute/path/to/oriel",
      "args": ["mcp", "--cache-dir", "/absolute/path/to/oriel-cache"]
    }
  }
}
```

The server exposes two tools:

- `read_source`: a source URL plus optional language and refresh flag;
- `search_source`: a source URL, query, and optional language, timestamp bounds, result limit and refresh flag.

MCP runs locally over standard input/output. Protocol output contains only MCP messages, and caller cancellation propagates to live acquisition.

## How it is shaped

Oriel is one engine with thin interfaces:

```text
                 +-> CLI
YouTube -> engine+-> MCP -> agent
                 +-> future local interfaces
                    |
                    +-> one local evidence cache
```

The engine keeps these concerns separate: source identity, provider I/O, caption normalisation, timestamped evidence, persistence, retrieval and transport. `yt-dlp` is invoked as a constrained external provider; source URLs are passed as process arguments rather than concatenated into shell commands.

Evidence remains distinct from interpretation. Oriel records what the source said, where it was said, how the captions were produced and what coverage is missing. An agent's conclusions remain in the calling task rather than being written back as transcript truth.

The cache contains schema-versioned compiled evidence. Oriel does not keep raw media, signed caption URLs or raw provider metadata. It does not bundle or redistribute transcripts or media. People using Oriel remain responsible for their access to a source and for how they use the resulting evidence.

The short architecture decisions in [`docs/decisions`](docs/decisions) explain why the first version uses one Rust package, a constrained `yt-dlp` adapter, local MCP and both whole-source reading and lexical search.

## Verify a checkout

The normal checks are deterministic and do not contact YouTube:

```sh
cargo fmt --check
cargo clippy --locked --all-targets --all-features --offline -- -D warnings
cargo test --locked --offline
```

Run the release-mode retrieval benchmark separately when changing performance-sensitive code:

```sh
cargo bench --locked --bench retrieval_latency --offline
```

Live-provider checks are intentionally separate because YouTube and `yt-dlp` are external, changing systems.

## Current limitations

- Live acquisition supports public YouTube videos with manual or generated captions available to `yt-dlp` as JSON3. Private, authenticated, live and upcoming sources are unsupported.
- Oriel does not process visuals. Claims shown only in diagrams, code, demonstrations or on-screen text will be missing.
- There is no local transcription fallback, so a source without usable captions cannot be read.
- Only YouTube is supported. Multi-source synthesis and sources longer than 26 minutes remain unmeasured.
- Generated captions can badly misspell names and technical terms. Oriel reports that the wording was machine-generated rather than silently repairing it.
- Search is lexical. On the frozen evaluation it found the correct moment for 20 of 31 real questions (65%), below the 90% target. It also returned results for three of four deliberately absent subjects, so a result is not proof that a source answers the question.
- Whole-source reading is therefore the default for the evaluated 8–26 minute sources. In a historical founder-run three-source evaluation, the calling agent answered 30 of 30 questions and 222 of 222 timestamp links matched cached passage starts. The raw answers remain private because one run used private local context, so these reviewed aggregates are not independently reproducible from the public checkout. The evaluation used GPT-5.6; smaller calling models are untested.
- Warm-cache change detection is explicit. A normal cache hit does not contact YouTube; use `--refresh` to check the current source.
- The live provider depends on external YouTube and `yt-dlp` behaviour and can break independently of Oriel.
- There is no web application, hosted service, account system or remote MCP transport.

The full measurements and method are in [`evals/usecase-v1/results.md`](evals/usecase-v1/results.md). [`status.md`](status.md) is the compact current state, while [`plans/spec1.md`](plans/spec1.md) preserves the founding product and engineering specification.

## Project direction

The next versions will be driven by real use and measured failures. Better retrieval, longer or multiple sources, transcription, visual evidence and a thin web interface are all plausible directions; none is promised before evidence shows that its added complexity earns its place.

The durable goal is simple: make it fast and easy for a human or agent to work from what a source actually said, without losing the timestamp, provenance or limitations along the way.

## Contributing and community

Oriel is an early project and thoughtful contributions are welcome. Please read:

- [`CONTRIBUTING.md`](CONTRIBUTING.md) for development and contribution guidance;
- [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md) for the community standard;
- [`SECURITY.md`](SECURITY.md) for private vulnerability reporting.

## Licence

Oriel is available under either the [Apache License, Version 2.0](LICENSE-APACHE) or the [MIT licence](LICENSE-MIT), at your option.
