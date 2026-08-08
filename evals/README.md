# Evaluations

Oriel keeps two kinds of evidence deliberately separate.

## Deterministic fixture

`retrieval-v1.tsv` is the synthetic fixture used by the offline test suite. It needs no
network, account or private artefact.

## Frozen live-source retrieval corpus

`session-log.tsv` is a frozen set of natural questions and human verdicts. It can be
replayed against an existing local cache with:

```sh
cargo build --release
python3 evals/replay.py
```

Replay never fetches a source. It exits if a required cached version is absent.

## Historical founder evaluations

`trial/` and `usecase-v1/` record dated evaluations performed during Oriel's initial
development. Their committed questions and reviewed Markdown reports are the public
record. They are not universal benchmarks: they used particular videos, calling agents
and founder context, all described in the reports.

The runners are retained so their method can be inspected and adapted. They invoke
external agent CLIs, may incur provider cost and require the videos to be warm in a local
Oriel cache. Running them is not part of Oriel's build or test process.

The tool-choice trial also requires an unrelated repository that the agent may inspect:

```sh
python3 evals/trial/run.py --repo /path/to/an/unrelated/repository
```

The whole-source runner can repeat the source-only questions without private context:

```sh
python3 evals/usecase-v1/run_codex.py
```

Two frozen GPT questions refer to the founder's own task history. Without
`--private-context-dir`, the runner tells the agent to mark those questions as not run.
Supplying that option is an explicit decision to expose exactly that directory to the
read-only calling agent; it is never necessary for public use or public claims.

Raw agent streams, answers, summaries, caches and diagnostics can include private
repository or task context. They stay ignored by Git. Do not force-add them. Public
descriptions should retain the exact scope and limits stated in the committed reports;
unpublished raw output is not a distributable proof artefact.
