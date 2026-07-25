# RockQL

> **Readable data pipelines for every database.**

RockQL is an open-source query language and compiler for writing database queries as readable top-to-bottom pipelines and compiling them into standard SQL.

```rockql
from employees
filter salary > 50_000
derive yearly_salary = salary * 12
sort {-yearly_salary}
take 10
```

```sql
SELECT
    *,
    salary * 12 AS yearly_salary
FROM employees
WHERE salary > 50000
ORDER BY yearly_salary DESC
LIMIT 10;
```

**Write the flow. Generate the SQL.**

## Status

RockQL is at the compiler-foundation stage. The first implementation includes:

- A Rust workspace.
- A serialisable abstract syntax tree.
- A parser with line and column diagnostics.
- `from`, `filter`, `select`, `derive`, `sort`, and `take`.
- Generic SQL, SQLite, and PostgreSQL output.
- `compile`, `check`, `ast`, and `format` CLI commands.
- Unit tests and GitHub Actions CI.

The language and APIs are experimental until the v1.0 compatibility policy is published.

## Install from source

```bash
git clone https://github.com/Sayanthrock-Developer/ROCKQL.git
cd ROCKQL
cargo install --path compiler/rockql-cli
```

## CLI

Compile a file:

```bash
rockql compile query.rockql --target postgres
```

Compile stdin:

```bash
echo "from users | filter active == true | take 10" \
  | rockql compile --target sqlite
```

Validate syntax:

```bash
rockql check query.rockql
```

Print the AST:

```bash
rockql ast query.rockql
```

Format source:

```bash
rockql format query.rockql
rockql format query.rockql --write
```

## Initial syntax

```rockql
from orders
filter status == "completed"
select customer_id, amount
derive tax = amount * 0.18
sort {-amount}
take 20
```

Supported transformations:

- `from`
- `filter`
- `select`
- `derive`
- `sort`
- `take`

Both newline pipelines and `|`-separated pipelines are accepted.

## Workspace

```text
compiler/
├── rockql-ast/       Shared syntax model
├── rockql-parser/    Source parser and diagnostics
├── rockql-sql/       SQL dialect generator
└── rockql-cli/       Cross-platform command-line interface
```

Planned components include the resolver, relational IR, optimiser, formatter crate, WebAssembly bindings, browser playground, Android app, desktop app, language server, Tree-sitter grammar, and VS Code extension.

## Compiler direction

```text
RockQL source
    ↓
Lexer / parser
    ↓
Abstract Syntax Tree
    ↓
Name and type resolver
    ↓
Relational intermediate representation
    ↓
Query optimiser
    ↓
SQL dialect generator
    ↓
Formatted SQL
```

## Roadmap

- **v0.1:** compiler foundation, SQLite generator, CLI, tests.
- **v0.2:** WebAssembly compiler, Monaco playground, diagnostics, query sharing.
- **v0.3:** joins, grouping, aggregation, variables, functions, PostgreSQL and MySQL.
- **v0.4:** formatter, language server, VS Code extension, Tree-sitter grammar.
- **v0.5:** visual pipeline editor, schema browser, DuckDB, CSV/JSON/Parquet.
- **v1.0:** stable syntax, compatibility policy, benchmarks, signed releases, Android app, JavaScript and Python bindings.

## Project boundaries

Early RockQL versions focus on querying and transforming data. Database administration, migrations, write operations, cloud credential storage, team collaboration, paid AI services, and complex optimisation are intentionally out of scope.

## Design direction

RockQL interfaces use charcoal and near-black surfaces, white primary text, muted supporting text, a restrained electric-blue or violet accent, rounded surfaces, clean monospaced code areas, subtle glass layers, and strong contrast without dashboard clutter.

## Licence

Licensed under the [Apache License 2.0](LICENSE).

RockQL is an independent implementation. Third-party source code must retain its original licence, copyright notices, and required attribution.