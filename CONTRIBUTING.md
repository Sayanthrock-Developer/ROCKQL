# Contributing to RockQL

RockQL is early-stage software. Small, focused pull requests with tests are preferred over large changes that mix language design, compiler internals, and UI work.

## Development setup

Requirements:

- Rust 1.80.1 or a compatible newer toolchain.
- Git.

```bash
git clone https://github.com/Sayanthrock-Developer/ROCKQL.git
cd ROCKQL
cargo test --workspace
```

## Quality checks

Run the same checks used by CI before opening a pull request:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo build --release --package rockql-cli
```

## Language changes

A language change should include:

1. A clear syntax example.
2. Parser and AST updates.
3. SQL output examples for every affected dialect.
4. Positive and negative tests.
5. Documentation updates.
6. A compatibility note when existing syntax changes.

Do not silently reinterpret valid syntax. Diagnostics should identify the exact line and column whenever possible.

## Compiler boundaries

Keep the stages separate:

- Parsing creates syntax, not SQL.
- Resolution validates names and types.
- Relational IR represents database operations.
- Optimisation rewrites IR, not source text.
- Dialect generators own target-specific SQL.

The initial compiler intentionally excludes database administration, migrations, write operations, credential storage, collaboration, and paid AI services.

## Pull requests

Use a descriptive title and explain:

- The problem being solved.
- The implementation approach.
- Tests performed.
- Known limitations.

By contributing, you agree that your contribution is licensed under Apache License 2.0.