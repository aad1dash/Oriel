# 0004: Let an agent read a whole source, not only search it

**Status:** Accepted, 2026-08-02

## Context

Retrieval was measured for the first time against real sources rather than a fixture whose questions were written alongside it. Thirty-one questions were written from video titles before any transcript was read, then graded against the full transcript. The result was **20 of 31 (65%)** against a 90% release target, recorded in `evals/session-log.tsv` and replayable with `evals/replay.py`.

Two things about that measurement mattered more than the number.

The first is what the failures were made of. Six of eleven were vocabulary gaps: the source says `Königsberg`, the question says `concrete example`; the source says `isn't magically perfect`, the question says `downsides`. Three were false positives on subjects the source never discusses. Two were proper nouns the captions never spell correctly. None of these is a ranking problem.

The second is the size of the sources. Full transcripts run 2,072 to 8,013 tokens for videos of 8 to 26 minutes. An agent given the whole transcript has all of the source. Retrieval was giving it 65%.

The questions people actually ask of a source reinforce this. They are overwhelmingly of the form *is there anything here worth adopting*, *is this worth my attention given what I work on*, *explain this to me plainly* — questions answered by an argument, not by a moment.

## Decision

Add `read_source` as a second tool on both surfaces, alongside `search_source`:

- `oriel read (--fixture <path> | --source <url>) [--language <tag>] [--cache-dir <path>] [--refresh]`;
- an MCP tool of the same name.

It returns every passage in order, each keeping its own `timestamp_url`, plus the source identity, coverage, acquisition and cache provenance already carried by evidence packets. It takes no query, limit or timestamp bounds, because narrowing flags on a whole-source read would be a lie.

The packet warns `captions_machine_generated` whenever the wording was not written by a human. A search result can be sanity-checked against its matched terms; a whole transcript cannot, and generated captions mishear exactly the proper nouns a technical source turns on.

Both tools resolve their source through one shared path, so caching, refresh semantics, deadlines and cancellation behave identically.

## Evidence

- The three cached sources read whole as 31, 55 and 18 passages — 3,299, 8,013 and 2,072 tokens — all served from cache with no network access.
- `where does he give a concrete example` returns nothing from retrieval. The whole transcript carries the answer at 85 seconds, and spells `Königsberg` correctly.
- A wire-level test initialises the MCP server, discovers both tools, and reads a cached source back as ordered passages with citable timestamps.
- Formatting, strict Clippy and 48 offline tests pass.

## Alternatives

### Fix the false positives inside lexical retrieval

Attempted and rejected on measurement. Roughly thirty configurations were swept against the corpus: inverse-document-frequency term weighting, an absolute distinctiveness floor, a question-coverage ratio, a 104-word closed-class stopword list, and a two-words-or-one-strong-word rule. The Pareto frontier was 20/31 with 2 of 4 false positives, or 17/31 with 1 of 4 — about one and a half recall points spent per false positive removed. The one configuration that held recall did so by dropping the answer to a video's own title question. No variant improved both, so none shipped, and `src/search.rs` is unchanged.

### Semantic retrieval

Deferred. Six of eleven failures are vocabulary gaps that no lexical refinement closes, so this is the fix if retrieval is the right instrument. At these lengths it is not yet clear that it is, and the machinery is large. It must stay local when it comes.

### Return the transcript as one string

Rejected. The project's promise is that a claim can be traced to when it was said. Passages keep timestamps so an answer synthesised from the whole argument can still cite it.

## Consequences

- The measured 65% is no longer the ceiling on what an agent can get from a source; it is the ceiling on what *retrieval* gets, and there is now a door around it.
- Retrieval keeps its place for sources too long to read whole and for questions that really are about locating a moment. Nothing about `search_source` changed.
- Two tools now need distinguishing descriptions, and whether agents choose correctly between them is unproven.
- A caller that reads several long sources at once will feel the token cost that retrieval exists to avoid.

## Revisit when

- a real agent trial shows agents picking the wrong tool of the two;
- sources arrive that are too long to read whole, making retrieval quality the binding constraint again;
- mistranscribed proper nouns are measured separately, since they damage both tools;
- the corpus grows enough that a lexical change could be shown to help rather than merely to trade.
