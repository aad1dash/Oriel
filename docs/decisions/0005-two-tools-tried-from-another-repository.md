# 0005: Keep both tools and both descriptions unchanged; the tools compose rather than compete

**Status:** Accepted, 2026-08-02

## Context

Decision 0004 added `read_source` beside `search_source` and closed with an admission:
"Two tools now need distinguishing descriptions, and whether agents choose correctly
between them is unproven." Neither tool had ever been used by an agent that did not
write it. Nothing inside this repository could settle it.

The trial is recorded in full in `evals/trial/results.md`. Nine questions were written
before any run, three archetypes across the three cached sources: *is there anything
here worth adopting*, *explain this plainly*, and *where exactly does he say X*. Each
ran in its own headless agent session started in `AgentPsych`, an unrelated Python
research repository, with web access withheld so the two tools competed for the same
slot. The `locate` questions are the control; without them a uniform preference for
`read_source` would be unreadable.

## Decision

**Change nothing in `src/mcp.rs`.** Both tool descriptions stay as written in 0004.

The cheap fix that was on offer — rewriting the descriptions so agents pick correctly —
is not needed. First tool choice matched expectation **9 of 9**. All three `locate`
questions opened with `search_source`; all six whole-argument questions opened with
`read_source`.

**Record that the two tools compose rather than compete.** The premise of the trial was
that they contend for one slot. They do not. Two of three `locate` questions escalated
from `search_source` to `read_source`, and in both the escalation is what made the
answer correct. Search narrows; read confirms, and in particular confirms *absence*.
A ranked list cannot support the sentence "he never says this."

## Evidence

- First tool choice matched expectation 9/9. Nine fresh sessions, `claude-opus-5`, $3.27
  total, all sources warm in cache and no network.
- 47 of 47 cited `t=<seconds>` links land inside a real passage of the source they cite.
- 7 of 9 answers carried a source timestamp. The two without are adoption verdicts; one
  grounded itself in the calling repository instead, one was a "no".
- **The sponsor-read false positive was removed by the caller.** `search_source` returned
  the sponsor passage at 343960 ms, as it does in the corpus. The agent labelled it a
  sponsor read and dropped it. Roughly thirty lexical configurations could not do this
  without paying recall; a caller holding the whole source did it for free.
- **`read_source` recovered a graded miss.** `session-log.tsv` records `what are the
  downsides or the things he complains about` as a miss at 1361000 ms. A whole-source
  read surfaced it, quoted verbatim and verified against cache at 1361440 ms. That is one
  of the six vocabulary-gap failures, closed with no retrieval change.
- **The corpus was also generous in the other direction.** It grades `how does he compare
  it to claude` a hit at 1449000 ms. The agent read that passage and correctly judged it
  not a comparison. On this question the agent's answer is better than the graded one.

## Alternatives

### Rewrite the tool descriptions

Rejected on measurement. This was the anticipated outcome and the pre-committed fix.
At 9/9 there is nothing to correct, and editing a description that measures perfectly
would trade a known state for an unknown one.

### Collapse to one tool that decides internally

Rejected. The escalation pattern is only visible because the caller chooses. An agent
that called `search_source`, found nothing satisfying, and then read the whole source
made a judgment Oriel has no basis to make for it — Oriel cannot tell a thin result from
a correct absence, and the caller can.

### Fix the citation and label defects found in the trial

Deferred, deliberately, and named here so they are not lost:

1. Packets carry `start_ms` and `timestamp_url` but no human-readable label, so agents
   compute `mm:ss` themselves and drift — one answer wrote `[3:29]` over a correct link
   to `t=201s`. A `timestamp_label` field would end it.
2. A quote spanning two passages was cited at the passage where the claim begins rather
   than where the words are (`t=925s` for wording at 954760 ms).
3. Agents silently repair mistranscribed proper nouns from their own knowledge — captions
   say `Opus 48` and `CAA`, answers say `Opus 4.8` and `Kakeya`. One run flagged the
   repair; another did not. Helpful when the agent knows the name, invisible when wrong.

None of these is a retrieval defect and none justified widening this trial's scope. (1)
is the cheapest and best-evidenced next change in the repository.

## Consequences

- The open question from 0004 is closed. Both surfaces have now been used by an agent
  that did not write them, and the answers were usable and traceable.
- The 65% recall figure is further demoted. It measures one tool, and the trial shows
  the caller routing around its two most expensive failure modes — sponsor-read false
  positives and vocabulary gaps — by reading the source.
- Retrieval quality remains un-binding *at these lengths*. Nothing here argues against
  semantic retrieval; it argues that the case for it still has not arrived.
- Proper-noun mistranscription is now observed damaging both tools in the field, and
  observed being silently patched over by the calling model. That is worse than it
  failing loudly, because it is invisible.

## Revisit when

- a source arrives that is too long to read whole, which is the first situation in which
  choosing the wrong tool costs anything;
- the trial is repeated on a smaller or non-Anthropic model, since 9/9 was measured on
  `claude-opus-5` alone and description-following is a model capability;
- an agent is observed choosing `read_source` for a source large enough to hurt, which is
  the failure this decision would have caught if it existed.
