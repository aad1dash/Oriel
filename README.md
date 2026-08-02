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

Read a whole source instead of searching it, when the question is about what the source argues rather than about locating one moment:

```sh
cargo run -- read \
  --source 'https://www.youtube.com/watch?v=VIDEO_ID' \
  --language en \
  --cache-dir .oriel-cache
```

This returns every passage in order, each keeping its own timestamp, so an answer drawn from the whole argument can still cite where it was said. It takes no query or timestamp bounds. For sources of this length it is usually the better tool: an 8 to 26 minute video reads whole in 2,000 to 8,000 tokens, and measured retrieval returns 65% of what is there. See `docs/decisions/0004-read-a-whole-source.md`.

The cache stores content-addressed, schema-versioned JSON. It does not retain raw metadata, signed caption URLs, audio or video.

Each provider stage has a 90-second deadline. The engine also exposes a cancellation token for future long-running surfaces; either condition terminates and reaps the provider process before temporary files are removed.

Generated captions arrive as short overlapping fragments. The YouTube adapter merges them into passages of at least 30 seconds so that a returned excerpt is readable on its own.

## Run a retrieval session

`./ask` drives one source without retyping the full command, prints timestamps and excerpts instead of JSON, and records what you asked and whether it worked.

```sh
./ask --video 'https://www.youtube.com/watch?v=VIDEO_ID'
./ask 'where does she explain why the first approach failed?'
./ask --hit          # the right moment came back
./ask --miss 12:34   # it did not; the answer is at 12:34
```

Ask in ordinary words rather than in keywords you already know appear in the source. Every question and verdict is appended to `evals/session-log.tsv`, which is intended to become the first retrieval corpus whose questions were not written alongside the transcript.

The deterministic fixture path remains available offline:

```sh
cargo run -- search \
  --fixture fixtures/synthetic/captioned-explainer-v1.tsv \
  --query 'Why does retrieval begin with lexical search?'
```

The search command returns a compact JSON evidence packet containing source identity, coverage, acquisition and cache provenance, ranked moments, timestamps and warnings.

## Use from Codex anywhere

Oriel does not need to be added to each repository. Install the binary once and
register it in Codex's user-level MCP configuration:

```sh
cargo install --path . --root "$HOME/.local" --offline --force
codex mcp add oriel -- "$HOME/.local/bin/oriel" mcp \
  --cache-dir "$HOME/.local/share/oriel/cache"
```

Restart the Codex desktop app after registration. Oriel is then available in a
normal Codex task, whether or not that task is attached to a repository. For
example:

```text
Summarise this video clearly and cite the important moments: <YouTube URL>
```

Codex should use `read_source` to take in the argument and do the summarisation
itself. Oriel supplies ordered, timestamped source evidence; it does not add a
second model call. For a precise question such as “where does she explain the
failure?”, Codex can use `search_source` first.

### Add the study workflow

The repository also includes a candidate Codex skill at
`skills/oriel`. MCP remains the capability; the skill supplies the judgement
that does not fit in a short tool description:

- **Scout:** decide whether a video deserves the user's time;
- **Learn:** explain what it establishes, assumes and leaves unresolved;
- **Apply:** compare supported lessons with an active project and propose the smallest
  reversible experiment;
- **Find:** locate the exact moment where something was said.

The skill reads an ordinary source whole before reasoning, keeps source evidence separate
from agent interpretation and audits timestamp receipts before answering. It does not
create a second cache, wiki or retained brief. It is packaged here for inspection and is
not installed into the user's global Codex configuration automatically.

## Use from another MCP-compatible agent

Build Oriel, then configure an MCP client to launch the binary over local `stdio` with an explicit cache directory:

```json
{
  "mcpServers": {
    "oriel": {
      "command": "/absolute/path/to/oriel/target/release/oriel",
      "args": ["mcp", "--cache-dir", "/absolute/path/to/oriel-cache"]
    }
  }
}
```

Two tools are exposed. `search_source` accepts a source URL, query and optional language, timestamp bounds, result limit and refresh flag, and returns the same structured evidence packet as the CLI. `read_source` accepts a source URL and optional language and refresh flag, and returns the whole source as ordered, individually timestamped passages. Search moments and transcript passages include canonical human-readable labels such as `3:21` or `1:03:08` alongside their millisecond values and clickable URLs.

Prefer `read_source` when the question is about what the source argues, recommends or is worth taking from; prefer `search_source` when the question is genuinely about locating a moment, or when the source is too long to read whole. Both share one acquisition, cache and provenance path, and MCP cancellation propagates to live provider acquisition.

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
- Retrieval finds the right moment for 20 of 31 real questions (65%) against a 90% release target, measured in `evals/session-log.tsv`. Reading a source whole sidesteps this; searching it does not.
- Retrieval matches words literally. It has no notion that `vector search` and `semantic search` mean the same thing, and six of eleven measured failures are exactly that gap.
- Retrieval returns evidence for subjects a source never discusses: three of four planted absent subjects came back with results. About thirty lexical configurations were swept against the corpus and none improved this without costing more recall than it saved.
- Generated captions mishear proper nouns. In fourteen minutes about the Kakeya conjecture the captions never spell `Kakeya` once. This damages both tools, though `read_source` at least carries the correctly spelled title.
- Passage length is a fixed 30 seconds chosen by inspection rather than by measurement.
- A cache written before passage segmentation is rejected as an unsupported schema; delete the cache directory to reacquire.
- The MCP surface contains only source search and whole-source reading. One nine-session trial from an unrelated repository selected correctly between the tools every time. A newer three-source Codex use-case evaluation answered 30/30 questions and validated 222/222 timestamp links, but used the equivalent installed CLI because this already-running task could not dynamically acquire the MCP.
- Sources longer than 26 minutes, multi-source synthesis and smaller calling models remain unmeasured.
- The web application, transcription and visual evidence are not implemented.

The founding product brief is in [`plans/spec1.md`](plans/spec1.md). Current implementation state is in [`status.md`](status.md), and the latest real-use report is in [`evals/usecase-v1/results.md`](evals/usecase-v1/results.md).
