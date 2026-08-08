# Use-case evaluation 01 — whole sources inside real work

**2026-08-02. Historical founder-run evaluation: three sources, thirty questions and three isolated Codex readers.**

## Decision

Whole-source reading is the correct default for all three tested videos. Oriel should
remain a lean evidence engine. The useful intelligence belongs in a small calling-agent
skill, not in a retained wiki, vector database or internal model pipeline.

## Setup

Ten questions per source were frozen before the readers saw any transcript. They cover
triage, learning, depth, unsupported premises, exact moments, project application and
bounded experiments. The GPT-5.6 source additionally required a narrow audit of local
Codex task logs and real before-and-after workflows.

Each source was read once in a separate, read-only Codex subagent. The registered MCP
was not dynamically exposed inside this already-running task, so the agents used the
installed CLI against the same `SourceEngine` and cache. This evaluates the source-to-
work behaviour, not MCP transport; MCP already has separate wire-level evidence.

Raw answers and task streams stay local under ignored `answers/` and `runs/` directories
because one run cites private local task logs. The checked totals below are a reviewed
historical aggregate, not independently reproducible proof from the public checkout.

## Result

| Measure | Result |
|---|---:|
| Questions answered | 30 / 30 |
| Triage decisions that reached a non-generic verdict | 3 / 3 |
| Negative-control answers that rejected unsupported extrapolation | 3 / 3 |
| Answers that surfaced missing visual coverage | 3 / 3 sources |
| Timestamp links checked against cached passage starts | 222 / 222 valid |
| Question sections carrying at least one source timestamp | 29 / 30 |

`GRF-05`, a project-fit answer, omitted a video timestamp even though all of its project
citations were valid. The engine returned correct receipts; the calling agent failed to
carry one into that section. This is why the new skill ends with an explicit citation
audit.

Two fresh forward checks then used the repository skill on unseen prompts. Both chose a
complete read, used search only to verify a decisive passage, preserved caption and
visual caveats and proposed one bounded experiment rather than a new feature. The first
check exposed a presentation error: it displayed a timestamp range on a link targeting
only the range start. The skill now requires Oriel's exact label and separate endpoint
links when a range matters. The second check followed that corrected rule and resisted
turning the Kakeya proof method into AI doctrine.

## What the sources actually yielded

### Kakeya

Worth watching for its research-method arc, not for independently understanding the
proof. The transferable pattern is to change representation when the current measure
hides the phenomenon, then ask what structure a genuine counterexample would be forced
to possess. The agent kept this as an analogy to research design rather than silently
promoting geometry into project doctrine.

### GPT-5.6

Mostly a personal build log, with four durable lessons: give Sol substantial outcomes,
provide durable environmental context, let agents observe and verify their environment,
and use bounded subagents for review or prioritisation. The video does not support a
reasoning-effort, pricing or fast-mode recipe.

A private local-work audit supported the broader finding that large tool output and long
tasks create avoidable latency and attention pressure. Project names and exact private
work telemetry are intentionally omitted. The highest-confidence improvements were:

1. filter large tool output before it reaches Sol;
2. start fresh tasks at verified engineering boundaries;
3. route routine work away from Sol/high;
4. begin operational work with outcome, allowed mutations, forbidden actions,
   acceptance checks and stop conditions.

### Graph engineering

Worth eight minutes as a design lens, but not established as a new discipline. Its useful
content is ordinary workflow and distributed-systems thinking applied to capable agent
nodes: explicit decomposition, parallelism, context isolation and joins. The source does
not establish that 75-agent voting improves truth or that Oriel needs a graph framework.

The justified experiment is source-bounded fan-out for several long videos: one temporary
reader per source, one synthesis step, then a citation audit. Keep it only if elapsed time
or answer quality improves enough to justify the additional tokens.

## Performance

The installed release binary was tested live with an empty temporary cache on all three
sources, then fifty warm process invocations per source.

| Source | Cold whole read | Warm p50 | Warm p95 | Agent JSON bytes |
|---|---:|---:|---:|---:|
| GPT-5.6, 26 min | 2.61 s | 1.973 ms | 2.215 ms | 39,582 |
| Graph engineering, 8 min | 3.87 s | 1.892 ms | 1.951 ms | 12,211 |
| Kakeya, 15 min | 2.78 s | 1.878 ms | 1.950 ms | 19,475 |

A dense `timestamp|text` representation was 20.7–26.0% smaller in bytes without removing
transcript words. It was not shipped: bytes are not model tokens, and a smaller wrapper
only matters if citation reliability and comprehension remain intact.

## Open-source comparison

Current source inspection found three useful reference points:

- [`ytranscript`](https://github.com/nadimtuhin/ytranscript) is the closest minimal
  competitor. It uses direct Innertube/JSON3 acquisition and may beat Oriel cold, but its
  ordinary single-video path has no persistent offline cache and omits Oriel's source
  version, completeness, acquisition and visual-coverage contract.
- [`summarize`](https://github.com/steipete/summarize) is the strongest broad product,
  with transcript caching, transcription fallbacks, slides, OCR and agent backends. It is
  deliberately much larger than Oriel's intended engine.
- [`youtube-transcript-mcp`](https://github.com/hancengiz/youtube-transcript-mcp) is a
  representative transcript MCP returning one formatted text blob without Oriel's cache
  or source-integrity contract.

Competitor packages were inspected, not executed. No fair cross-tool cold-speed claim is
made until pinned tools are run against the same caption tracks and network conditions.

## Product consequences

Implemented in this repository slice:

- search packets now warn when captions were machine-generated, matching whole reads;
- ordinary root and command `--help` invocations now succeed;
- a repository-packaged `oriel` skill encodes Scout, Learn, Apply and Find,
  whole-source-first reading, temporary readers for long or multiple sources, no retained
  interpretation and a final citation audit.

Not shipped:

- a transcript wiki, embeddings or persistent derived briefs;
- an internal LLM;
- compact agent output before token measurement;
- automatic proper-noun correction;
- visual processing;
- a graph framework.

The skill is intentionally not installed into the user's global daily-driver configuration
by this repository change.
