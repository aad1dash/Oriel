# Status

**2026-08-08 — PUBLIC v0.1.0: open-source release**

## Current position

Oriel is a working v0.1 local-first evidence engine:

```text
YouTube URL → captions → timestamped local evidence → calling agent judgement
```

It is publicly available with that narrow claim. The final audit findings are addressed,
the complete local and hosted Linux gates pass, private vulnerability reporting is enabled,
and the documented source install succeeds from a clean public clone.

## What is proven

- CLI and local `stdio` MCP share one Rust `SourceEngine`, cache and provenance model.
- `read_source` returns ordered, citable passages; `search_source` returns bounded moments.
- Evidence retains source identity, version, language, caption provenance, coverage,
  millisecond timestamps, human-readable labels and clickable URLs.
- Live coverage is derived from validated caption timing. Measured edge drift is bounded,
  partial tracks remain explicit and timestamps outside the source are rejected.
- Provider deadlines and cancellation terminate and reap child processes. Raw media,
  signed caption URLs and provider metadata are not retained.
- A later ordinary Codex task used the registered MCP to read and search a new source
  end to end. This closes the initial adoption check; it is an operational observation,
  not a new benchmark.
- A historical founder-run calling-agent evaluation answered 30/30 questions and matched
  222/222 timestamp links to cached passage starts. Its raw answers remain private because
  one run used private local context. One of 30 answer sections omitted a source receipt;
  the skill now requires a citation audit.
- Lexical retrieval found 20/31 frozen questions and returned evidence for three of four
  absent subjects. Whole-source reading remains the default for the measured 8–26 minute
  sources.
- On arm64 macOS 26.6, three live cold reads took 2.61–3.87 seconds. Fifty warm release
  invocations per source measured 1.878–1.973 ms p50 and 1.950–2.215 ms p95.
- Formatting, strict Clippy and all 60 offline tests pass on pinned Rust 1.97.1.

## Public repository preparation

- Dual licensing is explicit: MIT or Apache-2.0, at the recipient's option.
- The README now explains the founder's real problem, the current product, installation,
  CLI and MCP use, architecture, privacy boundary, evidence and limitations.
- Contribution, conduct and security policies are present.
- GitHub CI has read-only permissions, fetches the locked dependency set once, then runs
  formatting, strict Clippy and tests offline.
- Public evaluation scripts no longer contain machine-specific paths. The runner that
  previously depended on founder task logs is not distributed; raw streams, answers and
  diagnostics remain ignored.
- The interactive `./ask` harness writes new questions to an ignored local path rather
  than changing the frozen public evaluation corpus. Private project names and exact
  work telemetry have been removed from the public evaluation report.
- The current RustSec audit reports zero vulnerabilities or warnings. All 92 locked
  packages declare licences with a permissive path.
- Public `main` preserves the development sequence while excluding the machine-specific
  paths and private evaluation material that appeared in private history. Every reachable
  commit uses Aadi's GitHub noreply identity.
- `Cargo.toml` remains `publish = false`. Opening the source repository does not imply a
  crates.io release.

## Known limitations

- Live acquisition supports public YouTube captions available as JSON3 through `yt-dlp`.
- Visuals, local transcription, private sources, authenticated sources and non-YouTube
  sources are unsupported.
- Generated captions can corrupt proper nouns; Oriel reports their provenance but does
  not silently repair them.
- Search is lexical and cannot reliably establish conceptual matches or absence.
- Warm-cache source change detection requires explicit `--refresh`.
- Sources longer than 26 minutes, multi-source synthesis, smaller calling models, Linux
  live acquisition and Windows live acquisition remain unmeasured.
- There is no web application, hosted service, account system or remote MCP transport.

## Release state

- GitHub `main` contains only the reviewed, sanitised history.
- Hosted Linux CI passes on the public release commit.
- The repository description and topics describe the narrow engine honestly.
- Private vulnerability reporting is enabled.
- An unauthenticated HTTPS clone installs successfully with `cargo install --path . --locked`.
- The `v0.1.0` tag identifies the first public release.

Further retrieval, provider, visual or interface work is follow-up development. None is
required to publish Oriel honestly as the early local-first engine described above.
