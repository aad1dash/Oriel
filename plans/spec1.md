# Oriel

## Founding Product and Engineering Specification

**Status:** Authoritative founding brief
**Working name:** Oriel
**One-line description:** A fast, open-source source-intelligence engine that lets humans and agents understand, interrogate and learn from long-form video.

---

## 1. Mission

Build the missing source-intelligence layer between long-form media and intelligent work.

A user should be able to give an agent a YouTube video, lecture, stream or other supported source and ask it to:

* answer a precise question;
* locate the relevant moment;
* understand what was said and shown;
* extract the source’s consequential ideas;
* teach those ideas;
* compare them with an active repository or task;
* apply relevant lessons while preserving evidence and provenance.

Today, the friction involved often means this does not happen at all. Oriel should make giving an agent a video feel nearly as easy as pasting text into a conversation.

---

## 2. Product thesis

Oriel is not primarily:

* a transcript downloader;
* a generic video summariser;
* another “chat with YouTube” interface;
* a personal note-taking application;
* a compulsory knowledge-management system;
* an LLM wrapper that repeatedly sends entire transcripts to a model.

Oriel is:

> **A reusable source engine that compiles long-form media into fast, timestamp-grounded context that any capable agent can use.**

The expensive source work should happen once. Subsequent retrieval should be fast, deterministic where possible and inexpensive.

The engine supplies accurate source context. The calling agent supplies general reasoning, teaching, repository inspection and implementation intelligence.

---

## 3. The missing capability

The important interaction is not merely:

> Summarise this video.

It is:

> Study this source for the task I am currently doing. Understand its argument and demonstrations. Find what is relevant to this project. Distinguish what the creator showed from what they merely asserted. Then use the supported lessons appropriately.

Oriel must therefore preserve four distinct layers:

1. **Source evidence**
   What was actually said or shown, with timestamps.

2. **Source interpretation**
   Claims, techniques, concepts, demonstrations, assumptions and caveats derived from the evidence.

3. **Contextual judgement**
   Whether and how those ideas matter to the user’s current question, project or learning goal.

4. **Consequences**
   The explanation, exercise, recommendation, implementation or experiment produced from that judgement.

These layers must not silently collapse into one another.

---

## 4. Primary user situations

### 4.1 Ask

The user opens a fresh chat or the web application and asks a question about one video.

Examples:

* “What is his actual argument about Rust?”
* “Why did she reject the first architecture?”
* “Explain the idea introduced at around 42 minutes.”
* “Does this lecture support the claim made in the title?”

The result should be direct, concise and supported by timestamped evidence.

### 4.2 Find

The user wants a moment rather than an essay.

Examples:

* “Find where he changes the invalidation logic.”
* “Show me the diagram explaining the pipeline.”
* “Where is the objection to vector search discussed?”
* “Find the example involving organisational memory.”

The result should prioritise precise moments, excerpts and frames.

### 4.3 Learn

The user wants the source converted into useful understanding.

Examples:

* “Teach me this lecture.”
* “What prerequisites am I missing?”
* “Turn this into exercises.”
* “Compare this explanation with what I already know.”
* “Extract the parts that matter for my curriculum.”

The output should adapt to the learner and goal rather than reproduce a generic summary.

### 4.4 Apply

The user is inside a repository or active project.

Examples:

* “Study this video and apply anything relevant to our caching layer.”
* “Compare this creator’s agent setup against this repository.”
* “Learn from this interface teardown and improve our current flow.”
* “Use this lecture as evidence when evaluating the design in this branch.”

Oriel supplies source intelligence. The local agent inspects private project context, exercises judgement, proposes or implements changes and verifies the result.

The remote source engine should not require access to the user’s private repository.

---

## 5. Product experience contract

### 5.1 Friction

The default web interaction should require no more than:

1. Paste a link.
2. Ask a question.
3. Receive an answer with evidence.

Do not force the user to create a library, project, notebook, collection or account before asking a question.

### 5.2 Immediate response

As soon as a link is accepted, show whatever can be resolved cheaply:

* title;
* creator or channel;
* duration;
* thumbnail;
* caption availability;
* processing coverage;
* cache status.

The user should be allowed to type a question immediately.

### 5.3 Progressive usefulness

The system must not wait for the most expensive possible analysis before becoming useful.

For example:

* metadata may be ready first;
* caption-backed questions may work before visual analysis;
* indexed transcript sections may become searchable while local transcription continues;
* vision may run only when the question requires it.

### 5.4 Evidence-first answers

The default answer should contain:

* a direct response;
* the strongest supporting moments;
* clickable or copyable timestamps;
* a clear indication of whether evidence came from transcript, metadata or visuals;
* uncertainty where coverage is incomplete.

### 5.5 Keyboard-first design

The interface should feel closer to a search utility than a conventional AI dashboard.

Core actions must be fast from the keyboard:

* focus the source field;
* ask a question;
* restore the previous question;
* open the transcript;
* jump to evidence;
* copy source context for an agent.

Keyboard shortcuts should remain discoverable and should not compromise ordinary accessibility.

---

## 6. Source-intelligence model

### 6.1 Canonical source

Every supported source resolves to a canonical identity and source version.

For YouTube, this will usually begin with the video ID and include hashes or version information for retrieved caption or transcript material.

The engine must avoid reprocessing an unchanged source.

### 6.2 Immutable evidence

Store evidence in a form that preserves:

* original ordering;
* start and end timestamps;
* source language;
* caption provenance;
* transcript confidence where available;
* visual timestamp;
* source version.

Normalisation may create a derived representation, but it must not destroy the original evidence required for audit or debugging.

### 6.3 Derived intelligence

The engine may derive:

* coherent segments;
* chapters;
* concepts;
* claims;
* demonstrations;
* procedures;
* entities;
* assumptions;
* caveats;
* contradictions;
* visual moments;
* source packets.

Every consequential derived item must retain links to the evidence supporting it.

### 6.4 Coverage

The engine must state what it actually processed.

Example coverage dimensions:

* metadata;
* manual captions;
* generated captions;
* local transcription;
* slides;
* code or interface frames;
* general visual analysis;
* complete or partial duration.

A polished answer must never imply that the entire video was visually understood when only captions were processed.

---

## 7. System shape

Oriel should have one source engine and several thin surfaces.

```text
                         Oriel Engine
                               │
             ┌─────────────────┼──────────────────┐
             │                 │                  │
            MCP              HTTP               CLI
             │                 │                  │
       Agent skill          Web app        Direct automation
```

### 7.1 Engine

The engine owns deterministic and source-specific work:

* source resolution;
* provider orchestration;
* transcript normalisation;
* timestamp-preserving segmentation;
* caching;
* indexing;
* retrieval;
* evidence packaging;
* status and coverage;
* source artefacts;
* local or remote serving.

### 7.2 Agent skill

The skill teaches a capable agent when and how to use Oriel.

It owns:

* interpreting the user’s goal;
* selecting Ask, Find, Learn or Apply behaviour;
* requesting appropriate source evidence;
* inspecting a local repository when applicable;
* comparing source ideas against project reality;
* teaching or implementing;
* verifying consequences;
* reporting provenance.

It should not reproduce the engine’s source-processing logic.

### 7.3 Web application

The web application should be a thin, excellent interface over the same engine.

It must not contain a separate intelligence pipeline that behaves differently from the MCP or CLI surfaces.

### 7.4 CLI

The CLI provides:

* local use;
* scripting;
* debugging;
* evaluation;
* installation verification;
* direct access without an agent interface.

---

## 8. Engineering doctrine

### 8.1 Rust where it earns its place

Prefer Rust for the latency-sensitive, deterministic core:

* ingestion coordination;
* normalisation;
* caching;
* search and retrieval;
* local persistence;
* HTTP and MCP serving;
* CLI behaviour;
* concurrency;
* performance-critical transformations.

Rust is a means to reliability, portability and performance, not an aesthetic requirement.

A simpler implementation is preferable when measurements show that Rust would not improve the relevant path.

### 8.2 TypeScript for the product surface

Prefer strict TypeScript for the web application and other browser-facing work.

The web client should be:

* thin;
* typed;
* accessible;
* fast;
* free of duplicated domain logic;
* uncomplicated unless demonstrated requirements justify additional machinery.

### 8.3 Existing tools over heroic rewrites

Do not reimplement mature source acquisition, media conversion or transcription infrastructure merely to keep the implementation in one language.

Evaluate existing tools behind explicit provider boundaries.

Likely candidates to investigate include:

* current subtitle and metadata acquisition tools;
* local transcription engines;
* media extraction tools;
* Rust full-text search libraries;
* the official Rust MCP SDK.

These are candidates, not frozen dependencies.

### 8.4 Local-first baseline

The required product path must work locally without:

* a hosted database;
* a cloud queue;
* a paid model API;
* a proprietary vector database;
* an Oriel account;
* Oriel-operated infrastructure.

Optional hosted services may later improve convenience, speed or cross-device access without weakening the local product.

### 8.5 Calling-agent intelligence

Do not require Oriel to pay for a second general-purpose model invocation when the calling agent can reason over well-structured evidence.

The engine should expose compact, useful source context rather than dumping an entire transcript or attempting to own every final answer.

Optional model-assisted enrichment may exist behind a replaceable interface. It must not become necessary for basic operation.

### 8.6 Measure before optimising

Performance work must begin with instrumentation and representative benchmarks.

Do not introduce complexity because it sounds fast.

Optimise:

* the paths users feel;
* warm retrieval;
* time to first useful evidence;
* source-cache reuse;
* installation and invocation overhead;
* context size returned to agents.

---

## 9. Research mandate

Before fixing consequential architectural choices, inspect the current state of the relevant ecosystem.

At minimum, investigate:

* Pronsh’s product interaction and public search implementation;
* existing open-source YouTube transcript and video-intelligence tools;
* current caption and metadata access methods;
* local transcription options on Apple Silicon and ordinary developer hardware;
* Rust full-text and hybrid retrieval options;
* current MCP transports, SDKs and client compatibility;
* source-platform constraints;
* open-source licensing implications;
* security requirements for local and remotely exposed MCP servers.

Research is not an excuse for a broad landscape report.

Produce a concise decision record containing:

1. the decision being made;
2. the options seriously considered;
3. the evidence or benchmark;
4. the selected option;
5. why the rejected complexity is unnecessary;
6. what evidence would justify revisiting the choice.

Prefer direct experiments over speculative comparison.

---

## 10. Required capabilities

### 10.1 Source resolution

The system must:

* accept common YouTube URL forms;
* canonicalise them;
* reject malformed or unsupported input clearly;
* retrieve basic metadata;
* distinguish unavailable, private, age-restricted and otherwise inaccessible sources;
* expose provider and processing errors without pretending the video does not exist.

### 10.2 Caption-backed ingestion

For a captioned source, the system must:

* discover available tracks;
* choose a sensible default while preserving alternatives;
* retain language and track provenance;
* store timestamped evidence;
* normalise text without losing traceability;
* avoid duplicate processing;
* support deterministic test fixtures without live network access.

### 10.3 Retrieval

The system must support:

* lexical retrieval;
* phrases;
* natural-language questions;
* timestamp filtering;
* chapter or section constraints;
* evidence ranking;
* compact context assembly.

Begin with the simplest retrieval system capable of meeting the evaluation target.

Add semantic retrieval or reranking only when measured failure cases justify it.

### 10.4 Evidence context

For a question, the engine must return a structured evidence packet containing:

* the canonical source;
* relevant moments;
* timestamp ranges;
* excerpts;
* retrieval scores or ranking information useful for debugging;
* processing coverage;
* warnings;
* suggested adjacent moments where useful.

The calling agent should be able to answer well without receiving the entire transcript.

### 10.5 Source packet

The engine must support a richer learning representation containing, where justified:

* source summary;
* claims;
* principles;
* techniques;
* demonstrations;
* caveats;
* assumptions;
* contradictions;
* high-value timestamps;
* visual dependencies;
* unresolved questions.

A source packet must distinguish explicit source content from inferred interpretation.

### 10.6 Visual access

The architecture must permit:

* retrieving a frame at a known timestamp;
* selecting candidate visual moments;
* analysing code, diagrams, slides or interfaces when transcript evidence is insufficient.

Full-video vision analysis is not required for the first vertical slice.

Visual processing should remain query-directed or evidence-directed until broader processing is proven valuable.

### 10.7 Local transcription fallback

The architecture must permit local transcription when captions are missing or inadequate.

Raw media should be temporary by default.

The interface must expose processing progress and partial coverage honestly.

### 10.8 MCP

Expose a minimal, coherent MCP surface.

Likely conceptual operations include:

* resolve a source;
* inspect source status;
* search moments;
* obtain question context;
* obtain a source packet;
* retrieve a frame.

Final tool names and schemas should be determined through use with real agents.

Support local `stdio` first. Design the server so authenticated remote transport can be added without duplicating domain logic.

### 10.9 Web application

The first application must include:

* source input;
* resolved source information;
* processing status;
* question input;
* streaming or progressive results;
* timestamp evidence;
* transcript inspection;
* useful keyboard interaction;
* clear empty and error states.

Avoid accounts and personal libraries in the initial product.

---

## 11. Project-application behaviour

When Oriel is used from inside a repository, the agent should:

1. understand the user’s intended outcome;
2. retrieve the relevant source packet and evidence;
3. inspect the actual repository;
4. identify current behaviour before proposing change;
5. classify source ideas as:

   * already present;
   * directly applicable;
   * applicable with adaptation;
   * worth a bounded experiment;
   * unsupported;
   * irrelevant;
   * harmful in this context;
6. make only justified changes;
7. verify them with tests, measurements or observable behaviour;
8. report which consequences came from which source moments.

Watching a persuasive video must not silently rewrite project doctrine.

Project adoption and source interpretation are separate records.

---

## 12. Performance objectives

Performance targets are product objectives, not excuses to falsify measurements.

### 12.1 Warm source

For a cached, indexed source on ordinary modern developer hardware:

* metadata lookup should feel immediate;
* local retrieval should target a p95 below 100 milliseconds;
* the engine should return the first useful evidence without loading the complete transcript into the calling agent;
* repeated queries must not repeat ingestion or transcription.

### 12.2 New captioned source

For a source with accessible captions:

* begin showing metadata immediately;
* begin useful processing as soon as captions are available;
* make the transcript searchable without waiting for optional enrichment;
* expose each stage rather than hiding all work behind a generic spinner.

### 12.3 New uncaptioned source

For a source requiring transcription:

* report that local transcription is required;
* process incrementally where practical;
* make completed coverage usable;
* avoid persistent raw media by default;
* allow cancellation and safe resumption where justified.

### 12.4 Agent context efficiency

Measure:

* evidence tokens returned per question;
* proportion of returned evidence actually used;
* retrieval recall;
* duplicate context;
* time to first useful answer.

Do not optimise server milliseconds while returning tens of thousands of unnecessary tokens.

---

## 13. Evaluation

Create evaluation infrastructure alongside the product.

### 13.1 Corpus

Build a versioned test corpus containing representative material:

* a short captioned explainer;
* a long lecture;
* a podcast;
* a coding demonstration;
* a slide-led presentation;
* a source with poor generated captions;
* a source without captions;
* a multilingual source;
* a source where the title overstates the content;
* a source containing corrections or contradictions.

Where redistribution is inappropriate, store expected metadata and authorised fixtures rather than copied source media.

### 13.2 Retrieval questions

For each source, define:

* direct lexical questions;
* paraphrased semantic questions;
* “find the moment” tasks;
* negative questions whose answer is absent;
* questions requiring context across adjacent segments;
* questions where visuals are required;
* questions where the source is ambiguous.

### 13.3 Core measures

Track at least:

* relevant-span recall at a small result count;
* timestamp accuracy;
* source-attribution correctness;
* unsupported-answer rate;
* false-positive retrieval;
* warm retrieval latency;
* cold caption-ingestion latency;
* cache reuse;
* returned context size;
* failure clarity.

### 13.4 Product success threshold

The first serious release should demonstrate:

* at least 90% relevant-span recall within the top five returned moments on the agreed evaluation set;
* 100% preservation of timestamp provenance for returned excerpts;
* no unsupported claims presented as explicit source statements in the evaluation suite;
* warm local retrieval p95 below 100 milliseconds on the documented reference machine;
* complete local operation for caption-backed use without paid credentials;
* successful use from the web interface, CLI and at least one MCP-compatible agent;
* successful project-application trials in at least three real repositories or tasks;
* evidence that Aadi begins using the system naturally rather than avoiding it because of friction.

The last criterion is the most important product test.

---

## 14. Security and privacy

### 14.1 Local server

A local MCP or HTTP server should:

* bind to localhost by default;
* avoid exposing unauthenticated network access;
* validate untrusted input;
* separate protocol output from logs;
* limit filesystem access;
* provide explicit storage locations;
* avoid executing source-derived commands.

### 14.2 Remote service

Any remote transport must add:

* authentication;
* origin and host validation;
* rate limits;
* request-size limits;
* safe session handling;
* abuse controls;
* clear retention rules.

Do not expose a local server through a public tunnel without deliberate authentication and origin controls.

### 14.3 Repository privacy

Private repository contents should remain with the local calling agent unless the user explicitly authorises otherwise.

The source engine should return source evidence. It does not need private code to perform that role.

### 14.4 Source handling

By default:

* process only user-requested sources;
* retain transcripts and derived artefacts deliberately;
* make raw media temporary;
* avoid bulk crawling;
* make deletion understandable;
* document provider and platform constraints.

---

## 15. Open-source doctrine

Oriel should be built with the intention of becoming genuinely open source.

The open-source version should contain the real engine rather than a decorative shell around a proprietary service.

A user should be able to:

* install it locally;
* process captioned sources;
* search and retrieve evidence;
* connect an agent;
* inspect stored artefacts;
* run the evaluation suite;
* contribute providers and improvements.

A future hosted version may provide:

* no-install use;
* shared warm caching;
* managed transcription;
* cross-device access;
* managed updates;
* abuse-resistant public endpoints.

The hosted version should sell convenience and infrastructure rather than withholding the basic product.

Select a permissive licence before public release after checking compatibility with direct dependencies and bundled components. MIT, Apache-2.0 or a dual licence should be considered.

---

## 16. Explicit non-goals for the initial product

Do not add without new evidence:

* social features;
* public profiles;
* collaborative workspaces;
* a general note-taking system;
* compulsory accounts;
* a recommendation feed;
* video generation;
* arbitrary web crawling;
* a proprietary agent runtime;
* autonomous source discovery;
* full playlist or course management;
* a graph database;
* distributed search;
* Kubernetes;
* microservices;
* a cloud queue;
* a hosted vector database;
* a large framework for agent orchestration;
* automatic edits to global agent instructions;
* speculative multi-agent panels.

These may become relevant later. They are not required to prove the product.

---

## 17. Decision rights and agent autonomy

This document freezes the product contract, not every implementation choice.

The implementation agent is expected to:

* research current options;
* inspect actual constraints;
* build experiments;
* benchmark alternatives;
* discover simpler approaches;
* challenge assumptions with evidence;
* select the smallest architecture capable of meeting the success criteria.

The agent may change a proposed technical choice when:

1. the product contract remains intact;
2. the alternative is simpler or measurably better;
3. the decision is documented;
4. the change does not quietly broaden scope;
5. the relevant tests or benchmarks support it.

The agent must not:

* weaken provenance to make implementation easier;
* introduce a paid dependency into the required path;
* hide incomplete coverage;
* redesign the project into a general knowledge platform;
* treat a candidate technology as doctrine;
* spend weeks polishing architecture before demonstrating the core interaction.

---

## 18. Delivery stages

### Stage 0 — Grounding

Produce:

* concise prior-art findings;
* source-access experiments;
* a system boundary diagram;
* initial data contracts;
* an evaluation corpus plan;
* one architecture decision record covering the first vertical slice.

Exit when the main uncertainties have executable experiments rather than speculative answers.

### Stage 1 — Evidence engine

Deliver one complete local path:

```text
YouTube URL
→ metadata
→ captions
→ timestamped evidence
→ index
→ relevant moments
```

Include:

* CLI;
* cache;
* deterministic fixtures;
* retrieval evaluation;
* measured latency.

Exit when it is already genuinely useful for finding and answering questions about captioned videos.

### Stage 2 — Agent-native use

Deliver:

* local MCP server;
* source-context tools;
* initial global agent skill;
* Ask, Find and Learn behaviours;
* one repository Apply workflow;
* provenance from source to recommendation or change.

Exit when an agent can use Oriel naturally from an unrelated repository.

### Stage 3 — Product surface

Deliver:

* minimal web application;
* keyboard-first interaction;
* progressive processing status;
* timestamp evidence;
* transcript inspection;
* polished error and empty states.

Exit when a new user can paste a link and understand the product without documentation.

### Stage 4 — Hard sources

Add only after the earlier experience is strong:

* local transcription fallback;
* on-demand frames;
* visual evidence;
* robust partial processing;
* cancellation and resumption.

Exit when coding demonstrations, lectures and uncaptioned sources are handled honestly and usefully.

### Stage 5 — Open-source release

Before publishing:

* simplify the repository;
* remove private assumptions and secrets;
* document installation;
* add architecture and contribution guidance;
* publish evaluation results;
* test a clean installation;
* select the licence;
* provide representative examples;
* establish a clear security policy.

---

## 19. Initial repository shape

The final repository structure is not frozen, but a coherent starting shape is:

```text
oriel/
├── AGENTS.md
├── SPEC.md
├── Cargo.toml
├── crates/
│   ├── domain/
│   ├── source/
│   ├── index/
│   ├── store/
│   ├── server/
│   └── cli/
├── apps/
│   └── web/
├── skills/
│   └── oriel/
├── evals/
├── fixtures/
├── docs/
│   └── decisions/
└── README.md
```

Create boundaries only when they clarify ownership or permit independent testing. Do not create empty crates to imitate an architecture diagram.

---

## 20. Definition of the first successful product

Oriel is successful when Aadi can encounter a useful YouTube video, paste its link into any relevant working context and say:

> “Learn from this for what we are doing.”

The agent can then:

* access the source without manual transcript work;
* retrieve the right moments quickly;
* distinguish source evidence from inference;
* understand what matters for the current goal;
* answer, teach or apply it;
* show where the resulting judgement came from.

The machinery should recede.

The source should become usable intelligence.
