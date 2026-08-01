# 0001: Keep the first evidence engine in one Rust package

**Status:** Accepted, 2026-08-01

## Context

Oriel begins from an authoritative product specification and an otherwise empty repository. The first implementation must prove source identity, timestamped evidence and useful retrieval without creating empty architectural layers or depending on live YouTube behaviour in deterministic tests.

The current environment has Bun 1.3.14 for future TypeScript surfaces. Rust 1.97.1 is the current stable patch release and is pinned for the engine. The official Rust MCP SDK supports both stdio and Streamable HTTP, but MCP is a later surface over the same engine. `yt-dlp` can expose metadata and manual or generated subtitles, but live provider behaviour must remain outside default tests.

## Decision

Start with one Rust package containing explicit modules for:

- source identity;
- timestamped evidence;
- deterministic fixture ingestion;
- lexical retrieval;
- the CLI transport.

The first path uses no third-party Rust dependency. It compiles a strict, synthetic, tab-separated caption fixture and returns compact JSON evidence packets. Retrieval is a transparent weighted lexical baseline. Every moment keeps its source, timestamp range, language, caption provenance and source version. Coverage states that visuals were not processed.

The fixture hash is explicitly labelled `fixture-fnv1a64`. It is deterministic test identity, not the future live-source cache key.

## Evidence

- The runnable fixture path proves the evidence contract before provider volatility enters the engine.
- The versioned synthetic corpus covers direct lexical, natural-question, find-moment, provenance and absent-answer cases.
- The entire core builds and tests offline.
- Rust Clippy runs with warnings denied.
- The [official yt-dlp documentation](https://github.com/yt-dlp/yt-dlp/blob/master/README.md) exposes metadata and separate manual/generated subtitle operations suitable for a process provider.
- The [official Rust MCP SDK](https://github.com/modelcontextprotocol/rust-sdk) supports the transports required by later stages.

## Alternatives

### Multiple crates immediately

Rejected because the boundaries are still small modules with one release cadence. Split only when independent ownership or compilation becomes useful.

### SQLite FTS5 or Tantivy immediately

Deferred because a six-cue fixture cannot justify an index. Establish a representative long-source benchmark first, then keep the simplest option that meets warm p95 and recall targets.

### Embeddings or an internal model

Rejected for the baseline. They weaken local-first operation and are not yet supported by evaluation failures.

### TypeScript engine

Rejected for the deterministic core. Bun remains the required package manager for the future web surface, while Rust owns the source engine.

## Consequences

- The current path is useful for deterministic engineering but does not acquire a live source or persist a reusable index.
- JSON serialisation and fixture parsing are deliberately narrow. Before expanding their schemas, adopt mature serialisation behind an explicitly approved private-repository dependency fetch.
- Live acquisition must run through an argument-array process boundary and classify provider failures.
- No web or interface design has been selected.

## Revisit when

- a representative long transcript establishes that linear retrieval misses latency or recall targets;
- live source versioning requires a collision-resistant content hash;
- MCP work benefits from an independent server crate;
- a second engine consumer makes a module boundary insufficient.
