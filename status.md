# Status

**2026-08-02 — IN PROGRESS: evidence engine, measured retrieval, whole-source reading**

## Works

- Common YouTube URL forms resolve to one validated source identity.
- Live `yt-dlp` metadata and manual or generated JSON3 captions compile through the same evidence engine as fixtures.
- Compiled evidence retains language, caption and acquisition provenance, SHA-256 source version and honest coverage.
- Deterministic lexical retrieval supports natural questions, timestamp filters and absent answers.
- The CLI accepts a live URL or fixture and returns compact evidence packets with clickable timestamps.
- An explicit local cache stores immutable, inspectable JSON versions and serves repeated questions without provider access.
- `--refresh` reacquires a cached source and reports whether compiled evidence changed.
- Every provider stage has a 90-second deadline and a caller can cancel acquisition; either path terminates and reaps the child process before cleanup.
- A local MCP `stdio` server exposes two bounded tools, `search_source` and `read_source`, over the same engine and structured packets as the CLI.
- Rolling caption fragments are merged into passages of at least 30 seconds, so an excerpt is a readable thought rather than four words.
- A question asked in full sentences retrieves what its keyword form retrieves.
- `./ask` drives one video from the terminal and records each question and verdict to `evals/session-log.tsv`.
- `oriel read` and the MCP `read_source` tool return a whole source as ordered passages, each keeping its own timestamp, for questions about what a source argues rather than about locating a moment in it.
- A whole-source read warns when the wording was machine-heard, which a search result can check against its matched terms but a transcript cannot.

## Evidence

- Rust 1.97.1; Bun 1.3.14 reserved for future TypeScript surfaces; `yt-dlp 2026.07.04` locally verified.
- Formatting and strict Clippy pass.
- 50 tests pass offline; live access is excluded from defaults.
- Deterministic timeout and cancellation tests terminate a five-second child in under one second.
- A wire-level MCP test initialises the server, discovers both tools, retrieves the correct cached moment at 10 seconds and reads the same source back whole as ordered, citable passages — all without network access.
- Synthetic retrieval corpus: 5/5 expected outcomes in the top five, including one negative case. This is infrastructure proof, not the product recall claim.
- Bounded live smoke: URL → manual English captions → expected evidence at 22.64 seconds; no media downloaded.
- Cache smoke: first request `miss`; identical network-restricted request `hit`; private directories were mode `0700`.
- Refresh smoke: unchanged reacquisition reported `source_changed: false` against the prior semantic version.
- Release benchmark, arm64 macOS 26.6, 10,000 synthetic cues, 500 warm runs: p50 2.908 ms, p95 3.098 ms, p99 3.172 ms. This is latency evidence, not retrieval-quality evidence.
- Two retrieval defects were found by asking the engine ordinary questions rather than by testing it. Both are fixed and covered:
  - Generated captions arrived as ~5-second overlapping fragments, so a top result read `"of mathematics. We're talking about the"`. A live 14-minute source now compiles to 31 passages instead of 406 fragments.
  - The match threshold scaled with question length, so a question needing 30 words demanded 10 of them inside one fragment and returned nothing. Phrasing a question fully now retrieves at least what its keyword form retrieves.
- The synthetic 5/5 corpus result is not recall evidence. Its questions reuse the fixture's own vocabulary, because both were written together.
- First measured recall, `evals/session-log.tsv`: **20 of 31 questions (65%)** across three real sources, against a 90% release target. Questions were written from titles alone, before any transcript was read, and graded afterwards against the full transcript. Per source: Kakeya 8/11, gpt-5.6 6/10, graph engineering 6/10.
- The 11 failures group into three causes, none of which stemming would fix:
  - **Vocabulary gap (6).** The source says `Königsberg`, the question says `concrete example`. The source says `isn't magically perfect`, the question says `downsides`. These are conceptual, not morphological.
  - **False positives on absent topics (3 of 4).** Three of four deliberately absent subjects returned evidence anyway. For `does he talk about vector embeddings or RAG`, the top result was the video's sponsor read.
  - **Mistranscribed proper nouns (2).** In 14 minutes about the Kakeya conjecture, generated captions never spell `Kakeya` once; it appears as `CA` (16), `CAA` (11), `CAT` (4) and `Kaya` (5). Any question naming the subject fails.
- Full transcripts are small: the three cached sources read whole as 31, 55 and 18 passages, or 3,299, 8,013 and 2,072 tokens, for videos of 14, 26 and 8 minutes. At this length an agent can read the whole source more cheaply and more accurately than retrieving 65% of it.
- `where does he give a concrete example` returns nothing from retrieval. The whole transcript carries the answer at 85 seconds and spells `Königsberg` correctly, which the captions elsewhere do not manage for `Kakeya`.
- **The false positives were attacked and the attack failed on measurement.** Roughly thirty lexical configurations were swept against the corpus: inverse-document-frequency term weighting, an absolute distinctiveness floor, a question-coverage ratio, a 104-word closed-class stopword list, and a two-words-or-one-strong-word rule. The frontier was 20/31 with 2 of 4 false positives, or 17/31 with 1 of 4 — roughly one and a half recall points per false positive removed. The one configuration that held recall did so by dropping the answer to a source's own title question. Nothing shipped; `src/search.rs` is unchanged. The finding is that lexical retrieval is at its ceiling here, not that the defect is acceptable.

## Not yet supported

- Authenticated, private, live or age-restricted sources.
- Representative authorised retrieval corpus and the 90% recall claim.
- Progressive acquisition status, web, local transcription or visual evidence.
- Broader MCP status/context operations and a real unrelated-repository agent trial.
- Automatic change checks on warm hits; refresh is explicit.

## Next

1. **Trial both tools from an unrelated real repository.** Two tools now need distinguishing descriptions and nothing yet shows agents choose correctly between them. This is the cheapest way to learn whether `read_source` actually changes what an agent can do, and it was already the stated next boundary before this session.

2. **Decide what to do about absent subjects, knowing retrieval cannot fix it.** Returning a sponsor read as evidence for `does he talk about vector embeddings or RAG` is still wrong. The measurement says the answer is not in ranking. It may be in the calling agent reading the source instead, in which case the honest move is to say so in the tool description rather than to keep tuning.

3. **Semantic retrieval remains justified but not urgent.** Six of eleven failures are vocabulary gaps that no lexical refinement closes. It earns its cost when sources arrive that are too long to read whole. It must stay local.

4. **Mistranscribed proper nouns are unsolved and now known to damage both tools.** Worth measuring separately before choosing between fuzzy matching and seeding vocabulary from title and description. The title is already correct in the packet, which is the cheapest place to start.

Passage length is a fixed 30 seconds chosen by inspection, not measurement. Nothing in this session isolated it as a cause of failure.

The working tree is clean and `main` is published to `origin`. Reproduce the recall number with `python3 evals/replay.py`, which reads the cache and contacts nothing.

No interface design has been selected. Web design requires founder consultation, external Claude Opus work and served HTML directions before implementation.
