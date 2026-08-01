# Oriel

Oriel is an early local-first source-intelligence engine. It is being built to turn long-form video into compact, timestamp-grounded evidence that humans and agents can reuse.

The current vertical slice is deliberately narrow: it canonicalises YouTube URLs and retrieves evidence from a validated synthetic caption fixture. It does not yet fetch live captions, persist an index, answer questions itself or inspect visuals.

## Run the current slice

Rust 1.97.1 is pinned in `rust-toolchain.toml`.

```sh
cargo run -- resolve 'https://youtu.be/Ori3lDemo01?si=tracking'
```

```sh
cargo run -- search \
  --fixture fixtures/synthetic/captioned-explainer-v1.tsv \
  --query 'Why does retrieval begin with lexical search?'
```

The search command returns a compact JSON evidence packet. It includes the canonical source, coverage, ranked moments, timestamps, transcript provenance and warnings.

## Verify

```sh
cargo fmt --check
cargo clippy --all-targets --all-features --offline -- -D warnings
cargo test --offline
```

Future TypeScript product surfaces use Bun. No pnpm project is present.

The founding product brief is in [`plans/spec1.md`](plans/spec1.md). Current implementation state is in [`status.md`](status.md).
