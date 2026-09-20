<div align="center">

# RockQL

### Readable data pipelines for every database.

Write queries as a clear top-to-bottom flow. Compile them into standard SQL.

<br/>

[![Rust](https://img.shields.io/badge/Rust-1.97-black?logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)
[![CI](https://github.com/Sayanthrock-Developer/ROCKQL/actions/workflows/ci.yml/badge.svg)](https://github.com/Sayanthrock-Developer/ROCKQL/actions)
[![Repository](https://img.shields.io/badge/GitHub-ROCKQL-181717?logo=github)](https://github.com/Sayanthrock-Developer/ROCKQL)

</div>

---

## What is RockQL?

**RockQL** is an open-source query language and compiler for expressing database queries as readable pipelines and generating standard SQL.

Instead of starting with SQL clauses, you describe the data flow:

```rockql
from employees
filter salary > 50_000
derive yearly_salary = salary * 12
sort {-yearly_salary}
take 10
```

RockQL can compile that pipeline into SQL:

```sql
SELECT
    *,
    salary * 12 AS yearly_salary
FROM employees
WHERE salary > 50000
ORDER BY yearly_salary DESC
LIMIT 10;
```

> **Write the flow. Generate the SQL.**

---

## ✦ Why RockQL?

- **Readable** — queries follow a natural top-to-bottom data flow.
- **Composable** — build a query from small transformations.
- **Portable** — generate SQL for supported database targets.
- **Diagnosable** — parser errors include line and column information.
- **Tool-friendly** — the compiler foundation is built in Rust with a serialisable AST.
- **Format-aware** — source can be formatted consistently through the CLI.

---

## ⚡ Quick start

### Build from source

```bash
git clone https://github.com/Sayanthrock-Developer/ROCKQL.git
cd ROCKQL
cargo install --path compiler/rockql-cli
```

### Compile a query

Create `query.rockql`:

```rockql
from orders
filter status == "completed"
select customer_id, amount
derive tax = amount * 0.18
sort {-amount}
take 20
```

Then:

```bash
rockql compile query.rockql --target postgres
```

### Pipe a query through stdin

```bash
echo "from users | filter active == true | take 10" \
  | rockql compile --target sqlite
```

---

## 🧰 CLI

| Command | Purpose |
| --- | --- |
| `rockql compile` | Compile RockQL into SQL |
| `rockql check` | Validate RockQL syntax |
| `rockql ast` | Print the parsed AST |
| `rockql format` | Format RockQL source |

Examples:

```bash
rockql check query.rockql
rockql ast query.rockql
rockql format query.rockql
rockql format query.rockql --write
```

---

## 🧩 Language foundation

The initial language supports:

```text
from
filter
select
derive
sort
take
```

Both styles are accepted:

### Multiline

```rockql
from users
filter active == true
select id, name
take 10
```

### Pipe-separated

```rockql
from users | filter active == true | select id, name | take 10
```

---

## 🏗️ Compiler architecture

**Legend:** `● built` · `○ planned`

```text
● RockQL source
        ↓
● Lexer / Parser
        ↓
● Abstract Syntax Tree
        ↓
○ Name / Type Resolver
        ↓
○ Relational IR
        ↓
○ Query Optimiser
        ↓
● SQL Dialect Generator
        ↓
● Formatted SQL
```

The current repository implements the parser/AST and SQL-generation foundation. Resolver, relational IR, optimisation, and additional tooling are planned layers rather than implemented components.

---

## 🔎 RockQL compared with adjacent tools

RockQL is an independent Rust implementation with its own pipeline syntax, AST, parser, and SQL generators. It occupies a similar problem space to tools such as PRQL and dataframe-style query systems, but this repository does not reuse their implementation.

| Concern | RockQL | PRQL | SQL |
| --- | --- | --- | --- |
| Pipeline-oriented source | Yes | Yes | No |
| Compiles to SQL | Yes | Yes | N/A |
| Rust compiler workspace | Yes | Yes | N/A |
| Structured AST in this repository | Yes | Yes | N/A |
| Current maturity | v0.1 foundation | Established project | Standard |

This is a factual feature comparison, not a compatibility claim. SQL output and supported operations depend on the current RockQL implementation.

---

## 🧪 Real examples

The repository keeps examples intentionally within the implemented v0.1 language:

- `examples/employees.rockql` — filtering, deriving, sorting, and limiting rows.
- `select` pipelines — choose columns before generating SQL.
- Pipe syntax — compose the same supported transformations on one line.

Joins, grouping, aggregation, variables, and functions are **not implemented yet**; they remain v0.3 design targets.

---

## 🌐 Web playground status

A browser playground is planned for **v0.2**. The current GitHub Pages site is documentation and project presentation only; it does not pretend to compile RockQL in the browser.

The intended v0.2 direction is a real WebAssembly compiler path with diagnostics and shareable queries, backed by the same compiler crates rather than a separate mock implementation.

---

## 📦 Workspace

```text
ROCKQL/
└── compiler/
    ├── rockql-ast/       Shared syntax model
    ├── rockql-parser/    Parser and diagnostics
    ├── rockql-sql/       SQL dialect generation
    └── rockql-cli/       Cross-platform CLI
```

---

## 🗺️ Roadmap

| Version | Direction |
| --- | --- |
| **v0.1** | Compiler foundation, SQLite generator, CLI, tests |
| **v0.2** | WebAssembly compiler, Monaco playground, diagnostics, query sharing |
| **v0.3** | Joins, grouping, aggregation, variables, functions, PostgreSQL and MySQL |
| **v0.4** | Formatter, language server, VS Code extension, Tree-sitter grammar |
| **v0.5** | Visual pipeline editor, schema browser, DuckDB, CSV/JSON/Parquet |
| **v1.0** | Stable syntax, compatibility policy, benchmarks, signed releases, Android app, JavaScript and Python bindings |

The roadmap is directional; features may change as the language and compiler architecture evolve.

---

## 🔬 Project status

RockQL is currently at the **compiler-foundation stage**.

The current implementation includes:

- Rust workspace architecture
- Serialisable AST
- Parser diagnostics with line and column information
- Core pipeline operations
- Generic SQL, SQLite, and PostgreSQL output
- `compile`, `check`, `ast`, and `format` commands
- Unit tests
- GitHub Actions CI

The language and APIs are experimental until the v1.0 compatibility policy is published.

---

## 🎯 Scope

RockQL focuses on **querying and transforming data**.

Early versions intentionally do not focus on:

- Database administration
- Database migrations
- Write operations
- Cloud credential storage
- Team collaboration
- Paid AI services
- Complex optimisation

Keeping these boundaries clear helps the compiler and language evolve around a focused core.

---

## 🧭 Design principles

RockQL is designed around a few simple ideas:

**Readable first**  
A query should communicate its data flow clearly.

**Compiler-native**  
The language should have a structured syntax model rather than being a string-rewriting layer.

**Portable output**  
SQL generation should be separated from the language itself so dialects can evolve independently.

**Useful diagnostics**  
Errors should point developers toward the actual source location and problem.

**Small core, expandable tooling**  
The compiler foundation stays focused while playgrounds, editors, bindings, and visual tools can grow around it.

---

## 🤝 Contributing

Issues, ideas, documentation improvements, tests, compiler work, and tooling contributions are welcome.

Before proposing a language change, please consider:

1. Is the syntax readable?
2. Does it preserve the pipeline model?
3. Can it be represented cleanly in the AST?
4. Can SQL generation remain dialect-aware?
5. Can the behaviour be tested?

---

## 📄 Licence

RockQL is licensed under the [Apache License 2.0](LICENSE).

RockQL is an independent implementation. Third-party source code must retain its original licence, copyright notices, and required attribution.

<div align="center">

**RockQL · Readable pipelines → SQL**

[GitHub](https://github.com/Sayanthrock-Developer/ROCKQL)

</div>
