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
- Both tools have been used by an agent that did not write them, from an unrelated repository, and chose correctly between them on every question.

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
- **First trial from an unrelated repository: tool choice 9 of 9**, recorded in `evals/trial/results.md` and reproducible with `python3 evals/trial/run.py`. Nine questions written before any run, three archetypes across the three cached sources, each in its own headless agent session started in `AgentPsych` with web access withheld so the two tools competed for the same slot. Cost $3.27, all warm, no network.
  - The two tools **compose rather than compete**. Two of three locate questions escalated `search_source` → `read_source`, and in both the escalation made the answer correct. A ranked list cannot support the sentence "he never says this."
  - **The caller removed the sponsor-read false positive that thirty lexical configurations could not.** `search_source` returned the sponsor passage at 343960 ms as it always does; the agent labelled it a sponsor read and dropped it, because it could see the surrounding source.
  - **`read_source` recovered a graded miss.** `what are the downsides or the things he complains about` is a miss in the corpus at 1361000 ms. A whole-source read surfaced and quoted it, verified against cache at 1361440 ms — one of the six vocabulary gaps, closed with no retrieval change.
  - The corpus is also generous in the other direction: it grades `how does he compare it to claude` a hit at 1449000 ms, and the agent read that passage and correctly judged it not a comparison.
  - 47 of 47 cited timestamp links land inside a real passage. 7 of 9 answers carried a source timestamp; the two without were adoption verdicts, one grounded in the calling repository instead and one a "no".
- Three defects surfaced by the trial, none of them in retrieval: packets carry no human-readable timestamp so agents compute `mm:ss` and drift (`[3:29]` over a correct link to `t=201s`); a quote spanning two passages was cited at the passage where the claim begins rather than where the words are; and agents silently repair mistranscribed proper nouns from their own knowledge (`Opus 48` → `Opus 4.8`, `CAA` → `Kakeya`), flagging the repair in one run and not in another.
- **The false positives were attacked and the attack failed on measurement.** Roughly thirty lexical configurations were swept against the corpus: inverse-document-frequency term weighting, an absolute distinctiveness floor, a question-coverage ratio, a 104-word closed-class stopword list, and a two-words-or-one-strong-word rule. The frontier was 20/31 with 2 of 4 false positives, or 17/31 with 1 of 4 — roughly one and a half recall points per false positive removed. The one configuration that held recall did so by dropping the answer to a source's own title question. Nothing shipped; `src/search.rs` is unchanged. The finding is that lexical retrieval is at its ceiling here, not that the defect is acceptable.

## Not yet supported

- Authenticated, private, live or age-restricted sources.
- Representative authorised retrieval corpus and the 90% recall claim.
- Progressive acquisition status, web, local transcription or visual evidence.
- Broader MCP status/context operations.
- A trial on any model other than `claude-opus-5`, or on a source too long to read whole.
- Automatic change checks on warm hits; refresh is explicit.

## Next

1. **Give every passage a human-readable timestamp label.** The cheapest and best-evidenced change in the repository. Packets carry `start_ms` and `timestamp_url` but nothing preformatted, so agents compute `mm:ss` themselves and drift — one trial answer printed `[3:29]` over a correct link to `t=201s`. The link is right and the thing a human reads is wrong, which is the one failure this project cannot afford. Decision 0005 deferred it only to keep the trial's scope honest.

2. **Mistranscribed proper nouns are worse than measured.** The trial shows calling agents silently repairing them from their own knowledge — `Opus 48` → `Opus 4.8`, `CAA` → `Kakeya` — flagging it in one run and not in another. A loud failure would be safer than an invisible correction that is right most of the time. Measure this separately before choosing between fuzzy matching and seeding vocabulary from the title, which is already correct in the packet.

3. **Semantic retrieval remains justified but not urgent, and the trial weakened the case further.** Six of eleven failures are vocabulary gaps; the trial showed a caller closing one of them by reading whole. It earns its cost when sources arrive too long to read whole. It must stay local.

4. **Repeat the trial on a smaller model.** 9/9 was measured on `claude-opus-5` alone, and following a tool description is a model capability. A trial that only holds for the strongest model has not established that the descriptions are good.

Absent subjects are no longer open. The measurement said the answer was not in ranking; the trial showed where it is. A caller holding the whole source labelled the sponsor read as a sponsor read and dropped it, which is what thirty lexical configurations could not do.

Passage length is a fixed 30 seconds chosen by inspection, not measurement. Nothing in this session isolated it as a cause of failure.

The working tree is clean and `main` is published to `origin`. Reproduce the recall number with `python3 evals/replay.py`, which reads the cache and contacts nothing.

No interface design has been selected. Web design requires founder consultation, external Claude Opus work and served HTML directions before implementation.
