# Status

**2026-08-02 — WORKING PROOF: fast evidence engine, real-use behaviour measured**

## Current position

Oriel can turn a public, captioned YouTube video into complete timestamped evidence and
hand it to a calling agent in milliseconds from a warm local cache. Three isolated Codex
readers used it to answer thirty realistic questions across three videos. This proves the
source-to-work behaviour; it does not yet prove ordinary MCP use in a fresh Codex task.

The product boundary remains deliberately small:

```text
YouTube URL → captions → timestamped local evidence → calling agent judgement
```

Oriel does not run an internal model, retain interpretations or build a transcript wiki.

## What works

- Common YouTube URLs resolve to one validated identity.
- `yt-dlp` metadata and manual or generated JSON3 captions compile through the same
  `SourceEngine` used by fixtures, CLI and MCP.
- Evidence retains source version, language, caption provenance, coverage, provider
  information, millisecond timestamps, human-readable labels and clickable URLs.
- `read_source` returns the whole source as ordered passages. `search_source` locates
  bounded moments. Both reuse an immutable, inspectable local cache.
- Search and whole-read packets now both warn when captions are machine-generated and
  when visuals were not processed.
- Provider deadlines and caller cancellation terminate and reap child processes.
- Root and command-level `--help` and `-h` now succeed.
- A repository-packaged `study-with-oriel` skill encodes four calling behaviours:
  Scout, Learn, Apply and Find. It prefers a complete read for ordinary videos, uses
  temporary readers for long or multiple sources and finishes with a citation audit.

## Evidence

### Real use

The frozen use-case set is in `evals/usecase-v1/questions.tsv`; the report and exact
method are in `evals/usecase-v1/results.md`.

- **30/30 questions answered** across Kakeya, GPT-5.6 and graph-engineering videos.
- **222/222 timestamp links valid** against cached passage starts.
- **29/30 answer sections included a source timestamp.** One project-fit section omitted
  a video receipt even though the engine returned it; the skill now requires a final
  citation audit.
- **3/3 triage answers reached a specific watch-or-skip judgement.**
- **3/3 negative controls rejected unsupported extrapolation.**
- **3/3 sources carried the missing-visuals caveat.**
- Two fresh skill trials chose complete reading, used search only to verify a decisive
  moment, separated source evidence from project judgement and proposed bounded tests.
  The second trial also resisted transferring Kakeya mathematics into AI doctrine.

The three readers used the installed CLI against the same engine and cache because this
already-running Codex task could not dynamically acquire the newly registered MCP. MCP
transport has separate deterministic wire-level tests; a fresh ordinary Codex MCP task
is still the next adoption proof.

### Speed

Live cold reads used an empty temporary cache. Warm figures are fifty release-binary
process invocations per source on arm64 macOS 26.6.

| Source | Duration | Cold whole read | Warm p50 | Warm p95 |
|---|---:|---:|---:|---:|
| GPT-5.6 | 26 min | 2.61 s | 1.973 ms | 2.215 ms |
| Graph engineering | 8 min | 3.87 s | 1.892 ms | 1.951 ms |
| Kakeya | 15 min | 2.78 s | 1.878 ms | 1.950 ms |

A dense `timestamp|text` packet was 20.7–26.0% smaller in bytes, but was not shipped:
bytes are not model tokens, and citation reliability must be measured before changing
the packet contract.

### Earlier engine proof

- The initial independent trial selected correctly between search and whole read on
  **9/9 questions**; **47/47 cited links** landed inside real passages.
- Questions written from titles alone produced **20/31 lexical retrieval hits (65%)**.
  Complete reads recovered conceptual vocabulary gaps and let the caller reject sponsor
  passages, so whole-source reading is the current default for 8–26 minute sources.
- Roughly thirty lexical configurations failed to improve absent-topic precision without
  a disproportionate recall loss. No unmeasured ranking change shipped.
- Offline tests cover URL identity, acquisition, provenance, cache behaviour, refresh,
  retrieval, cancellation, cleanup, CLI and MCP schemas and wire behaviour.
- Formatting, strict Clippy and **53 offline tests** pass on Rust 1.97.1.

## Product decisions

- **Keep MCP as the capability.** It works without Bash and gives agents typed,
  cancellable tools.
- **Use a skill for judgement.** The skill explains when to Scout, Learn, Apply or Find;
  it does not duplicate the engine.
- **Hand ordinary sources over whole.** At the tested sizes, completeness is cheaper and
  more reliable than asking lexical search to understand the question.
- **Do not create a mini wiki.** Reuse compiled source evidence, but keep interpretation
  ephemeral in the calling task.
- **Do not add an internal LLM, embeddings or a graph framework yet.** None is required by
  the measured use cases.

## Known limitations

- Only public YouTube captions available as JSON3 through `yt-dlp` are supported.
- Generated captions can badly corrupt proper nouns; no silent correction is safe yet.
- Visuals, local transcription, private sources and non-YouTube sources are unsupported.
- Warm-cache change detection is explicit through `--refresh`.
- Lexical search cannot reliably answer conceptual or absent-topic questions.
- Sources longer than 26 minutes and multi-source synthesis have not been measured.
- The real-use answers used GPT-5.6 Sol; smaller-model calling behaviour is untested.
- The repository skill and newly fixed release binary are not installed into the
  founder's global Codex configuration by this change.

## Next

1. **Install the updated release binary and repository skill only with explicit founder
   approval, then run one fresh Codex task with no repository attached.** This is the
   smallest test of Oriel as an everyday YouTube summariser.
2. **Use it naturally for a week.** Capture failures from videos the founder already
   wants to understand rather than inventing features in advance.
3. **Measure one long or multi-source case.** Compare whole transcripts, temporary
   source-bounded readers and any compact packet using actual model tokens, time,
   citation coverage and answer quality.
4. **Run one restricted proper-noun experiment.** Seed a verified title term only in an
   evaluation copy and retain it only if it repairs the frozen failures without changing
   the other outcomes.

No web interface or Vercel deployment is justified yet. The useful next evidence comes
from ordinary Codex use, not from hosting the same engine behind another surface.
