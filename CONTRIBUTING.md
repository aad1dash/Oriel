# Contributing to Oriel

Thank you for taking Oriel seriously enough to help improve it.

Oriel is still early. The most valuable contributions usually begin with a real source, a concrete failure and a small change that makes the engine easier to use or trust. An ambitious rewrite without evidence is less useful than a precise bug report with a reproducible example.

## Before you begin

- Search the existing issues and architecture decisions.
- Open an issue before a substantial feature, new dependency or architectural change so the problem and smallest useful outcome can be agreed first.
- Keep source material lawful to share. Prefer minimal, authorised fixtures; do not commit downloaded media, private transcripts, signed caption URLs, credentials or personal data.
- Read the [current status](status.md) and the relevant decision in [`docs/decisions`](docs/decisions) before changing an established boundary.

Small bug fixes, documentation corrections and focused tests do not need an extensive proposal.

## Design principles

Contributions should preserve a few important distinctions:

1. Source evidence is not agent interpretation.
2. Every consequential excerpt keeps its source identity, provenance and timestamp.
3. CLI, MCP and future interfaces share one engine rather than growing separate intelligence pipelines.
4. Local use remains free and useful without a paid model, hosted database or Oriel account.
5. Expensive source work is compiled once and reused.
6. Retrieval complexity is earned by a measured failure, not added in anticipation of one.

The founding contract is in [`plans/spec1.md`](plans/spec1.md). Please discuss any proposal that would change its mission, required qualities or scope.

## Set up a development checkout

Install Rust through [`rustup`](https://rustup.rs/). The repository pins its toolchain in `rust-toolchain.toml`.

Deterministic tests use captured fixtures and do not require YouTube. Live acquisition additionally needs a current [`yt-dlp`](https://github.com/yt-dlp/yt-dlp#installation) installation with its EJS component and a supported JavaScript runtime such as Deno.

```sh
git clone https://github.com/aad1dash/Oriel.git
cd Oriel
cargo test --locked --offline
```

If Cargo has not fetched the locked dependencies on this machine yet, run `cargo fetch --locked` once while online, then repeat the offline test command.

## Make a focused change

- Follow the existing Rust style and use precise domain types.
- Treat URLs, provider output, timestamps, paths, stored data and MCP inputs as untrusted boundaries.
- Do not use `unwrap()` or `expect()` in normal runtime paths.
- Keep provider processes bounded and cancellable, and pass untrusted values as argument-array entries rather than shell text.
- Add or update deterministic tests with the behaviour.
- Use British English in documentation, comments and user-facing text.
- Update documentation when public behaviour or a consequential limitation changes.

Do not add a dependency when the standard library or current stack already solves the problem clearly.

## Check your work

Run the complete local gate before opening a pull request:

```sh
cargo fmt --check
cargo clippy --locked --all-targets --all-features --offline -- -D warnings
cargo test --locked --offline
```

If the change makes a performance claim, include a before-and-after release benchmark with the machine, operating system, corpus, cold or warm state, number of runs and relevant configuration. If it changes retrieval, report the result against the versioned evaluation set rather than only adding a favourable example.

Live-provider checks should be explicit and separate. They must not become a requirement for the deterministic test suite.

## Open a pull request

Keep the pull request small enough to review as one coherent concern. Explain:

1. the concrete problem;
2. the implemented solution;
3. how it was verified;
4. meaningful limitations or follow-up work.

Please follow the repository's restrained conventional commit style, for example:

```text
fix(source): preserve generated-caption provenance
feat(index): retrieve timestamped transcript passages
perf(search): reuse warm readers
```

By intentionally submitting a contribution to Oriel, you agree that it may be licensed under either the Apache License, Version 2.0 or the MIT licence, at the recipient's option, without additional terms or conditions.

Please also follow the [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md). Report security concerns privately as described in [`SECURITY.md`](SECURITY.md), not in a public issue.
