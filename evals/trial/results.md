# Trial 01 — both tools, from a repository that did not write them

**2026-08-02.** Nine questions, three sources, one unrelated repository.

## Setup

The agent ran in `/path/to/an/unrelated/repository`, a Python research
harness for running psychology-style experiments on language-model agents. It shares
no code, no vocabulary and no author-intent with Oriel. It has its own `README.md`,
`STATUS.md` and `docs/`, which the agent read.

Each question ran in its own headless `claude -p` session with no memory of the others,
so a tool choice is a first impression rather than a habit. Model: `claude-opus-5`.

The server was registered with `--mcp-config evals/trial/mcp.json --strict-mcp-config`
rather than by writing `.mcp.json` into AgentPsych. The agent sees identical tool
descriptions either way, and nothing was written into that repository.

Tools were restricted to `Read`, `Grep`, `Glob` and Oriel's two. `Edit`, `Write` and
`Bash` were withheld to keep the trial read-only in someone else's repo. `WebFetch` and
`WebSearch` were withheld for a second reason: with a web door open, `search_source` and
`read_source` would not be competing for the same slot, and tool choice is the thing
being measured.

Questions were written before any run, in `questions.tsv`. Three archetypes, one per
source. The `locate` questions are the control: without them, "the agent always chose
`read_source`" could not be told apart from "the agent never chooses `search_source`".

Total cost: **$3.27** for nine questions. Every source was warm in cache; no network.

## Result 1 — tool choice: 9 of 9

| question | archetype | expected | first call | all Oriel calls |
|---|---|---|---|---|
| adopt-graph | adoption | `read_source` | `read_source` | read |
| explain-graph | explain | `read_source` | `read_source` | read |
| locate-graph | locate | `search_source` | `search_source` | search → read |
| adopt-gpt | adoption | `read_source` | `read_source` | read |
| explain-gpt | explain | `read_source` | `read_source` | read |
| locate-gpt | locate | `search_source` | `search_source` | search ×3 → read |
| adopt-kakeya | adoption | `read_source` | `read_source` | read |
| explain-kakeya | explain | `read_source` | `read_source` | read |
| locate-kakeya | locate | `search_source` | `search_source` | search ×2 |

The tool descriptions do not need changing. That was the cheap fix on offer and the
measurement says it is not needed.

## Result 2 — the tools are not competing, they compose

The premise going in was that two tools contend for one slot. They do not.

Three of three `locate` questions began with `search_source`. Two of those three then
called `read_source`, and in both cases the escalation is what made the answer correct:

- **locate-graph** asked whether he names a specific graph database. Search returned
  candidates. Answering *no, he never names one* requires having seen the whole source,
  which is what the agent went and did.
- **locate-gpt** asked where he compares the model to Claude. Search was called three
  times, found nothing satisfying, and the agent fell through to reading the whole
  source — where it found the answer.

Search narrows. Read confirms, and confirms absence. A ranked list cannot support the
sentence "he never says this," and three of the four false positives in the retrieval
corpus were exactly that kind of question.

## Result 3 — the calling agent removed the false positive that thirty configurations could not

`evals/session-log.tsv` records `does he talk about vector embeddings or RAG` returning
the video's sponsor read as its top result. Decision 0004 records that roughly thirty
lexical configurations failed to remove that class of false positive without paying
recall for it.

In **locate-graph**, the sponsor read at 343960 ms was returned again — and the agent
labelled it `Zoe — sponsor read, unrelated` and excluded it from its answer. It could
do that because it had the surrounding passages and could see what the segment was.

The defect is real and lexical retrieval still cannot fix it. A caller that reads the
source can.

## Result 4 — read_source recovered a passage the corpus recorded as a miss

`session-log.tsv` grades `what are the downsides or the things he complains about` as a
**miss**, with the correct answer at 1361000 ms.

In **locate-gpt**, `read_source` returned that passage and the agent quoted it:

> "…its UI was incredibly ugly and I had to rebuild what it did there with Opus 4.8.
> Spoiler, this model isn't magically perfect at front end…"

Verified against cache at 1361440 ms. This is one of the six vocabulary-gap failures —
the source says `isn't magically perfect`, the question says `downsides` — and reading
whole closed it without any retrieval change.

The same run also shows the corpus was generous in the other direction. It grades
`how does he compare it to claude` a **hit** at 1449000 ms. The agent read that passage
and judged it *not* a comparison, which is correct: 1449600 is about building "ClaudeX"
to run Codex auth inside Claude Code. The agent's answer is better than the graded one.

## Result 5 — citations

Every cited moment was checked against the cache.

- **47 of 47** `t=<seconds>` links land inside a real passage of the source they cite.
- Quoted wording was hand-checked. It is substantially faithful, with two slips below.
- **7 of 9** answers carried a source timestamp. The two that did not are `adopt-graph`
  and `adopt-kakeya` — both adoption verdicts. `adopt-graph` grounded itself in the
  *repository* instead (`docs/OPERATING_SYSTEM.md:71`), and `adopt-kakeya` was a "no".

### Three defects worth naming, none of them in retrieval

1. **The packet has no human-readable timestamp, so agents compute one and drift.**
   `explain-graph` wrote `[3:29]` over a link to `t=201s`, which is 3:21. The link is
   right; the label a human reads is wrong. The packet supplies `start_ms` and
   `timestamp_url` but nothing preformatted. A `timestamp_label` field would end this.

2. **A quote spanning two passages was cited at the passage where the claim starts, not
   where the words are.** `adopt-gpt` cited `t=925s` for wording that lives at 954760 ms.
   Both passages are about the same thing, so the answer is not wrong, but the citation
   does not land on the sentence it quotes.

3. **Agents silently repair mistranscribed proper nouns.** The captions say `Opus 48`;
   the agent wrote `Opus 4.8`. The captions say `CAA` and `Bezakovich`; `explain-kakeya`
   wrote `Kakeya` and `Besicovitch`. Here it flagged the repair up front. In `adopt-gpt`
   it did not. This is helpful when the agent knows the name and invisible when it is
   wrong, and it is the status.md open question about proper nouns showing up in the
   trial rather than in a benchmark.

## What this does not show

Nine questions, one repository, one model, three sources, all short and all warm in
cache. It shows the descriptions work at this scale. It does not show they hold for a
source too long to read whole, which is the case where choosing wrong actually costs
something — and that case does not exist yet in this project.

## Reproduce

    python3 evals/trial/run.py

Requires the three sources cached and the `claude` CLI on PATH. Raw stream-json
transcripts are in `evals/trial/runs/`.
