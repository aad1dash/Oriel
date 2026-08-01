# 0003: Expose one source-search tool over local MCP stdio

**Status:** Accepted, 2026-08-01

## Context

The evidence engine already worked through the CLI, but an agent could not invoke it without shell-specific knowledge. The first agent surface needed to prove protocol compatibility without creating another acquisition or retrieval pipeline, exposing unauthenticated network access or prematurely freezing a broad tool taxonomy.

## Decision

Use the official Rust MCP SDK's local `stdio` transport and expose one tool: `search_source`.

The transport:

- accepts a source URL, query, optional exact language, timestamp bounds, result limit and explicit refresh flag;
- invokes the shared `SourceEngine` used by the CLI;
- requires the server process to receive an explicit cache directory;
- returns compact structured evidence with timestamps, coverage, acquisition and cache provenance;
- limits source and query sizes and returns at most twenty moments;
- performs blocking provider work away from the asynchronous protocol executor;
- propagates MCP request cancellation into provider-process cancellation;
- writes no ordinary logs to protocol stdout.

## Evidence

- `rmcp 3.1.0` is the current published official Rust SDK and provides the standard server-side `stdio` transport.
- A deterministic wire-level test launches `oriel mcp`, completes MCP initialisation, discovers exactly one tool, calls it and retrieves the correct cached moment at 10 seconds.
- The tool returns `structuredContent`, reports a cache `hit` and preserves source provenance without provider or network access.
- Formatting, strict Clippy and all offline tests pass.
- The [official Rust MCP SDK](https://github.com/modelcontextprotocol/rust-sdk) documents `stdio` as the standard local child-process transport.

## Alternatives

### Separate tools for resolve, status, search and context

Deferred. The specification identifies these as likely concepts, but says final schemas should be determined through real agent use. One complete search tool is enough to expose current product value and reveal what agents actually lack.

### Streamable HTTP

Deferred. Local `stdio` avoids authentication, host and origin concerns. Remote transport must not be added by duplicating the domain engine.

### A bespoke JSON-RPC implementation

Rejected. Protocol negotiation, cancellation, schemas and compatibility are volatile boundaries already handled by the official SDK.

## Consequences

- CLI and MCP now share acquisition, caching, retrieval and evidence-packet assembly.
- MCP adds the Tokio-based official SDK and its transitive dependency tree to the Rust binary.
- The single tool is deliberately provisional; real agent trials should decide whether status or context operations earn separate tools.
- No network listener, authentication surface or remote filesystem input was added.

## Revisit when

- real agent tasks repeatedly require a missing domain operation;
- evidence packets are too large or too small for useful agent reasoning;
- a representative agent cannot negotiate the supported MCP protocol;
- authenticated remote use becomes an explicit product requirement.
