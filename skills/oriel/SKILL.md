---
name: oriel
description: Use Oriel to study timestamped YouTube evidence inside the user's active work. Trigger when the user provides one or more YouTube URLs and asks whether a source is worth their attention, what it means, what can be learned or adopted, how it applies to a project, or where something was said. Use the Oriel MCP rather than asking the user to fetch a transcript manually.
---

# Oriel

Turn a video into grounded understanding or action without making Oriel a second brain.

## Choose the behaviour

- **Scout:** Decide whether the source is worth the user's time. Separate depth, novelty and evidence from framing, repetition and buzzwords.
- **Learn:** Explain the argument in language suited to the user. Identify what is demonstrated, asserted, assumed, unresolved or dependent on visuals.
- **Apply:** Compare supported lessons with the active project. Inspect the project's canonical files first and classify each idea as already present, directly applicable, needing adaptation, worth a bounded experiment, unsupported, irrelevant or harmful.
- **Find:** Locate the smallest passage answering an exact-moment question.

Combine behaviours when the request requires it. Do not force every request into a fixed report.

## Read the source

1. Infer the user's goal from the request and active task. Ask only when a missing choice would materially change the answer.
2. For an ordinary video, call `read_source` once and read the complete transcript. Prefer completeness over premature retrieval.
3. Use `search_source` for exact-moment questions or to verify the strongest citation. If search cannot establish meaning or absence, read the source.
4. For multiple or unusually long sources, give each source to a temporary isolated reader when that capability exists, then synthesise their evidence. Do not fill the main task with several raw transcripts.
5. Reuse Oriel's warm source cache. Do not create a separate transcript store, wiki, vector database or retained interpretation.

If Oriel is unavailable, state that the MCP capability must be installed or registered. Do not silently substitute an unrelated transcript service when the user explicitly asked to use Oriel.

## Preserve the evidence boundary

Keep these layers distinct in the answer:

1. what the source explicitly said or demonstrated;
2. interpretation of the source;
3. judgement for the user's context;
4. a proposed consequence, exercise or experiment.

Treat generated captions as machine-heard wording. Flag any repaired proper noun or technical term instead of silently rewriting the evidence. State when visuals were not processed, especially when the creator points to code, diagrams, interfaces or benchmarks on screen.

Do not turn a creator's preference into project doctrine. A persuasive source can inspire an experiment; it cannot waive existing authority, safety or verification rules.

## Finish with receipts

Before answering:

- give every consequential source-derived claim an exact clickable timestamp returned by Oriel;
- use Oriel's `timestamp_label` verbatim as the link label and ensure it agrees with the linked second;
- do not display a range on a link that only targets its start; link both endpoints separately when a range matters;
- distinguish local project citations from source timestamps;
- say when the transcript does not support the user's premise;
- remove claims whose evidence cannot be traced;
- keep the answer focused on the user's decision, learning goal or next action.

For a Scout request, lead with the watch-or-skip decision. For Apply, lead with what should change, what should not change and the smallest reversible test.
