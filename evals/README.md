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

The whole-source runner is intentionally not distributed. Two frozen GPT questions
depended on the founder's private task history, and a general-purpose calling agent's
read-only mode does not confine which local files it may inspect. The committed questions
and reviewed report preserve the historical method without publishing a script that could
misrepresent prompt instructions as a filesystem privacy boundary.

The retained tool-choice runner deliberately lets an external agent read the repository
passed through `--repo`. Run it only against a disposable, non-sensitive checkout. Source
text and agent behaviour are untrusted; read-only access prevents writes, not disclosure.

Raw agent streams, answers, summaries, caches and diagnostics can include private
repository or task context. They stay ignored by Git. Do not force-add them. Public
descriptions should retain the exact scope and limits stated in the committed reports;
unpublished raw output is not a distributable proof artefact.
