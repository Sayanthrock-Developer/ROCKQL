# Contributing to RockQL

Thank you for helping build RockQL.

## Development

1. Install the stable Rust toolchain.
2. Run `cargo fmt --all -- --check`.
3. Run `cargo clippy --workspace --all-targets -- -D warnings`.
4. Run `cargo test --workspace`.

Keep language changes small and include parser and SQL snapshot-style tests. User-facing syntax changes must update `docs/reference/mvp-language.md`.

## Pull requests

Describe the syntax or compiler behaviour changed, include representative RockQL input and generated SQL, and avoid mixing unrelated refactors with language changes.
