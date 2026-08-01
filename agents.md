
## Mission

You are working on Oriel: a fast, open-source source-intelligence engine for humans and agents.

Oriel turns long-form video into reusable, timestamp-grounded context. It should let an agent answer questions, find moments, learn from a source and apply relevant ideas within an active project.

Read `status.md` before consequential work. And update after any work is done make it very lean, clear and effective.

The specification freezes the mission, required qualities, success criteria and scope. It does not freeze implementation choices. Use current evidence and measurements to find the simplest effective architecture.

---

## Founder context

Aadi is strongest at asking consequential questions and thinking in systems. He values ambitious work, but dislikes complexity that exists only to signal sophistication.

Communicate so that he retains his bearings:

* explain the product consequence before low-level detail;
* use plain language and Feynman-style explanations for difficult concepts;
* distinguish facts, assumptions, options and recommendations;
* state what changed and why it matters;
* do not bury the central decision in implementation commentary;
* use British English in documentation, interfaces, comments and user-facing copy.

Do not assume that technical obscurity is rigour.

The goal is to make complex systems as simple as they can responsibly be.

---

## Product doctrine

Keep the following principles intact.

### Oriel is an engine with thin surfaces

There should be one coherent source engine used by:

* the CLI;
* MCP;
* the web application;
* agent skills;
* future integrations.

Do not create separate intelligence pipelines for each interface.

### Evidence before interpretation

Preserve the distinction between:

1. what a source explicitly said or showed;
2. what Oriel derived from it;
3. what an agent inferred for a user’s context;
4. what was subsequently changed or produced.

Every consequential source-derived item must retain provenance.

### Compile once, retrieve quickly

Avoid repeating expensive source work.

Canonicalise source identity, detect unchanged artefacts and reuse existing processing.

Optimise the warm path and time to first useful evidence.

### Progressive usefulness

Do not block transcript-backed use on optional enrichment, complete transcription or full visual analysis.

Expose what is ready and state coverage honestly.

### Local-first and free by default

The required path must not depend on:

* paid model APIs;
* hosted databases;
* proprietary search services;
* Oriel-operated infrastructure;
* user accounts.

Optional services may be added later without weakening local use.

### Calling agents provide general intelligence

Oriel should retrieve, structure and expose source evidence efficiently.

Do not add an internal LLM call merely because the calling agent could reason over the returned context itself.

### No silent doctrine changes

A compelling video is evidence, not authority.

When applying a source within a repository, distinguish:

* already implemented;
* directly applicable;
* applicable with adaptation;
* experimental;
* unsupported;
* irrelevant;
* harmful in this context.

Do not silently promote one source’s advice into project-wide or global agent rules.

---

## Questions are read-only

A question is a request for an answer, not permission to change files.

When a request asks:

* “Should we…?”
* “How hard would it be…?”
* “What are your thoughts…?”
* “Why does…?”
* “Is it possible…?”
* “Can X do Y…?”

answer the question first. Do not edit the repository unless the user also clearly instructs you to do so.

Even when a change appears trivial, do not reinterpret a question as an implementation request.

---

## How to approach work

### Begin from the outcome

Before implementation, establish:

* the user-visible outcome;
* the current behaviour;
* the smallest complete path that proves the outcome;
* the acceptance criteria;
* the principal uncertainties.

Do not begin by scaffolding every component in the architecture diagram.

### Inspect before deciding

Before modifying an existing area:

* inspect its current implementation;
* identify existing conventions;
* find relevant tests and fixtures;
* understand data and control flow;
* check whether the repository already solves part of the problem.

Do not replace working code with a preferred pattern merely because it is unfamiliar.

### Research uncertain current decisions

For consequential choices involving current tools, protocols, libraries, source access or platform constraints:

* inspect primary documentation and source repositories;
* verify current versions and compatibility;
* run a focused experiment where practical;
* record the decision concisely.

Do not rely on remembered library behaviour when it may have changed.

Research should resolve a decision. Avoid producing a broad landscape report unless explicitly asked.

### Prefer vertical slices

Deliver end-to-end usefulness early.

A narrow complete path is more valuable than several elegant but disconnected layers.

For example:

```text
URL → captions → timestamped storage → retrieval → CLI result
```

is preferable to separately scaffolding provider, graph, queue, web, MCP and model packages without a working user path.

### Measure before optimising

Do not claim performance without measurements.

Before an optimisation:

1. establish a representative benchmark;
2. identify the dominant cost;
3. make the smallest plausible improvement;
4. measure again;
5. retain the complexity only when the gain matters.

Optimise user-perceived paths, not theoretical microbenchmarks detached from the product.

### Keep decisions reversible

Use provider and storage boundaries where genuine volatility exists.

Do not abstract every local function behind an interface.

A useful abstraction:

* isolates unstable source acquisition;
* preserves domain logic;
* permits deterministic tests;
* allows a realistic alternative.

An unnecessary abstraction:

* has only one implementation;
* hides simple data flow;
* exists in anticipation of unspecified scale;
* increases navigation cost without protecting a product boundary.

---

## Architecture guardrails

### Core boundaries

Keep these concerns separable even when they share a crate initially:

* source identity and metadata;
* evidence and timestamps;
* provider I/O;
* normalisation and segmentation;
* persistence;
* retrieval and ranking;
* context assembly;
* transport;
* user interface;
* agent orchestration.

Domain logic should not depend directly on a CLI, web framework or MCP transport.

### Rust

Prefer stable Rust for deterministic and latency-sensitive engine work.

Use:

* precise domain types;
* `serde` at serialisation boundaries;
* `thiserror` or similarly explicit domain errors;
* `tracing` for structured diagnostics;
* async I/O only where concurrency or external I/O justifies it;
* bounded concurrency;
* explicit ownership;
* small, testable functions.

Avoid:

* `unwrap()` and `expect()` in normal runtime paths;
* broad `Box<dyn Error>` propagation across domain boundaries;
* stringly typed states;
* unbounded task spawning;
* unnecessary `Arc<Mutex<...>>`;
* hidden global mutable state;
* blocking work on asynchronous executors;
* macros that obscure ordinary control flow;
* premature unsafe code;
* bespoke parsers where a mature parser already exists.

At boundaries, treat external data as untrusted.

Validate:

* URLs;
* provider output;
* timestamps;
* file paths;
* persisted schemas;
* MCP inputs;
* HTTP requests;
* process output.

Use `cargo fmt`, Clippy and focused tests as routine quality gates.

Warnings introduced by the change should be treated as failures unless a documented exception exists.

### TypeScript

Use strict TypeScript.

Prefer:

* inference and precise domain types;
* discriminated unions for processing states;
* explicit `unknown` at untrusted boundaries;
* Zod or the established validator at runtime boundaries;
* React, Vite, Tailwind and pnpm for the initial web application;
* direct platform APIs where they are adequate;
* small presentational components;
* accessible semantic HTML;
* keyboard interaction that supplements rather than replaces ordinary controls.

Avoid:

* `any`;
* broad casts;
* duplicated Rust domain rules;
* client-side state frameworks before demonstrated need;
* SSR before demonstrated need;
* authentication before the product requires identity;
* wrapper components with no semantic purpose;
* ornamental animation that delays interaction;
* generic AI dashboard patterns.

Use Vitest for focused TypeScript tests and Playwright for consequential browser paths.

### External processes and providers

Source acquisition, media conversion and local transcription may use mature external tools.

Wrap each volatile integration behind an explicit provider boundary.

A provider must:

* expose typed input and output;
* report capabilities;
* preserve source provenance;
* classify failure modes;
* support cancellation where relevant;
* avoid leaking process-specific output into the domain model;
* be replaceable without rewriting retrieval or storage.

Never construct shell commands by concatenating untrusted input.

Use argument arrays and explicit executable invocation.

### Storage

Start with the simplest local persistence capable of meeting the product and evaluation requirements.

Requirements include:

* deterministic source identity;
* source-version detection;
* schema versioning;
* timestamp preservation;
* cache invalidation;
* deletion;
* test isolation;
* inspectability.

Do not add a remote database to solve a local product problem.

Do not add a graph database merely because the product discusses knowledge.

### Search and retrieval

Begin with the simplest measured retrieval approach capable of meeting the evaluation target.

Lexical full-text retrieval should be taken seriously rather than treated as an obsolete baseline.

Add embeddings, vector search, hybrid fusion or model reranking only when:

* the evaluation set demonstrates a meaningful failure;
* the improvement is measured;
* context size and latency remain acceptable;
* local operation remains credible;
* the additional dependency is justified.

Every retrieval result must preserve its source timestamps.

### MCP

Keep MCP transport thin.

The server should expose domain capabilities rather than internal implementation details.

Support local `stdio` before remote transport.

For Streamable HTTP:

* use the current protocol and official SDK where appropriate;
* validate origin and host information;
* require authentication for remote access;
* bind locally by default;
* keep protocol output free from ordinary logs;
* enforce request and response limits;
* avoid leaking local filesystem paths.

Tools should return compact structured data designed for agent use.

Do not return the full transcript when a small evidence packet is sufficient.

### Web application

The web application is a client of the engine.

It should not:

* perform independent source interpretation;
* reproduce indexing logic;
* require its own data model;
* block on optional processing;
* conceal errors behind generic loading states.

The initial experience should remain:

```text
Paste a source → ask a question → inspect evidence
```

---

## Source integrity

### Preserve evidence

For every excerpt, preserve:

* canonical source;
* start timestamp;
* end timestamp where available;
* transcript or visual provenance;
* source language;
* relevant provider information;
* source version.

### Separate evidence and derivation

Use explicit types for:

* source evidence;
* normalised evidence;
* retrieved evidence;
* derived claims;
* source packets;
* agent conclusions.

Do not store inferred claims as though they were transcript text.

### State coverage

Every result should be able to state:

* which source duration was processed;
* whether captions were manual or generated;
* whether local transcription was used;
* whether visuals were inspected;
* whether processing remains incomplete;
* whether the source changed after indexing.

### No invented timestamps

Never generate a timestamp from approximate model memory or unsupported inference.

When a precise moment is unavailable, say so.

### Ephemeral raw media

Raw video or audio should be temporary by default.

Tests should verify cleanup for normal completion, cancellation and failure where practical.

---

## Evaluation and testing

### Build evaluations with the feature

Do not postpone evaluation until the product appears complete.

Every retrieval or intelligence change should be tested against representative fixtures or the versioned evaluation set.

### Default tests must be deterministic

Normal unit and integration tests must not require:

* YouTube availability;
* network access;
* paid credentials;
* a specific user account;
* mutable third-party output.

Use captured, authorised and minimal fixtures.

Live-provider checks should be explicit, separately invoked and tolerant of external instability.

### Test important boundaries

Cover at least:

* URL canonicalisation;
* malformed sources;
* provider selection;
* caption provenance;
* timestamp preservation;
* source-version and cache behaviour;
* schema migrations;
* retrieval ordering;
* missing answers;
* context-size limits;
* partial processing;
* cancellation;
* cleanup;
* MCP schemas;
* web-to-engine compatibility.

### Test failure quality

A failed request should distinguish meaningful causes such as:

* unsupported URL;
* unavailable source;
* captions unavailable;
* provider changed;
* transcription required;
* processing cancelled;
* index unavailable;
* source version changed;
* invalid request;
* internal defect.

Do not collapse unrelated failures into “Something went wrong”.

### Performance claims

Record:

* machine and operating system;
* fixture or corpus;
* build mode;
* cold or warm state;
* number of runs;
* p50 and p95 where meaningful;
* context size;
* relevant configuration.

Never present a debug-build timing as a product benchmark.

---

## Scope control

Do not add any of the following unless `SPEC.md` changes or measured product use demands it:

* accounts;
* teams;
* social features;
* recommendations;
* public profiles;
* mandatory libraries;
* playlists or courses;
* autonomous web crawling;
* hosted vector databases;
* graph databases;
* distributed systems;
* cloud queues;
* microservices;
* Kubernetes;
* general agent frameworks;
* a separate internal chat model;
* automatic global-memory mutation.

When an appealing adjacent feature appears, record it briefly and continue the current mission.

---

## Communication during implementation

For substantial work, provide concise updates organised around outcomes:

* what has been established;
* what uncertainty was resolved;
* what now works;
* what remains blocked;
* what decision is being made.

Do not narrate every command.

Surface important findings early, especially when they challenge the proposed direction.

When reporting completion, include:

* the user-visible result;
* consequential architecture choices;
* tests and evaluations run;
* measured performance;
* known limitations;
* source-access assumptions;
* files or interfaces changed.

Do not claim complete support when only the happy path exists.

---

## Delegation

Match ceremony to the task.

Do not spawn subagents for work a single agent can complete well in one pass.

Use parallel agents when the work genuinely benefits from independent breadth, such as:

* prior-art inspection;
* security review;
* retrieval evaluation;
* adversarial product review;
* platform-specific experiments.

Before parallel work, assign non-overlapping ownership.

The primary agent remains responsible for integrating conclusions and resolving contradictions.

Do not paste multiple unfiltered agent reports to the founder.

---

## Blast radius

Never touch without explicit instruction:

* production services;
* live databases;
* public DNS;
* public package registries;
* paid APIs;
* public repositories;
* the founder’s daily-driver agent configuration;
* globally installed tools;
* persistent tunnels;
* secrets or credential stores.

When work is adjacent to one of these, state what would be touched before acting.

Use temporary directories and isolated test data by default.

---

## Documentation

Keep documentation useful and current.

### `SPEC.md`

Authoritative product and engineering contract.

Do not casually rewrite it to match implementation convenience.

### Architecture decisions

Create a short decision record for consequential and difficult-to-reverse choices.

A decision record should contain:

* context;
* decision;
* evidence;
* alternatives;
* consequences;
* revisit condition.

Do not create an ADR for an ordinary local implementation detail.

### README

The README should eventually explain:

* what Oriel does;
* the shortest useful setup;
* one local example;
* one agent example;
* current limitations;
* project status.

Do not turn it into an internal design archive.

---

## Git and pull requests

Preserve existing repository conventions.

Use clear, restrained commit and PR titles.

Where conventional commits are established, use them consistently, for example:

```text
feat(index): retrieve timestamped transcript passages
fix(source): preserve generated-caption provenance
perf(search): reuse warm Tantivy readers
```

Before opening a PR:

* rebase onto current `main`;
* run relevant formatting, linting, tests and evaluations;
* inspect the final diff;
* remove accidental files and debug output;
* ensure documentation reflects consequential behaviour.

Open a real PR rather than a draft when review-bot coverage depends on it.

A PR description should begin with:

1. the problem;
2. the implemented solution;
3. verification;
4. meaningful limitations.

When asked to monitor a PR:

* inspect only checks and comments newer than the latest relevant push;
* verify automated findings against source;
* fix real issues;
* explain dismissed false positives;
* distinguish product failures from infrastructure flakes;
* remain quiet when nothing changed;
* stop when the requested disposition is satisfied.

Merge only when explicitly instructed or when the request provides a clear merge-when-green disposition.

Use the `file-pr` skill when available and appropriate.

---

## Working quality gates

Until repository-specific commands supersede them, expect the relevant equivalents of:

### Rust

```text
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

Run release-mode benchmarks separately when making performance claims.

### TypeScript

```text
pnpm lint
pnpm typecheck
pnpm test
pnpm build
```

Run Playwright for consequential browser paths.

Do not run every expensive check after every tiny edit. Run focused checks while iterating and the complete relevant gate before completion.

---

## Definition of done

A task is complete when:

* the requested user outcome works;
* source provenance remains intact;
* relevant tests pass;
* evaluation changes are understood;
* performance claims are measured;
* errors are clear;
* temporary artefacts are cleaned up;
* documentation is updated where behaviour changed;
* unnecessary complexity was not added;
* the final report is honest about remaining limitations.

Passing tests is necessary but not sufficient.

The product should become simpler to use, understand or trust.
