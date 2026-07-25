# RockQL

**Readable data pipelines for every database.**

RockQL is an open-source query language and compiler that turns readable, top-to-bottom data pipelines into standard SQL.

> **Project status:** early v0.1 compiler foundation. The syntax is experimental and may change before 1.0.

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

## Why RockQL?

- Read complex queries in execution order.
- Generate SQL without replacing existing databases.
- Target multiple SQL dialects from one language.
- Receive compiler errors with exact line and column positions.
- Keep a future visual pipeline and written code synchronised.
- Run local data privately in future RockQL Studio applications.

**Tagline:** *Write the flow. Generate the SQL.*

## Current MVP

The repository currently implements:

- Rust workspace and compiler architecture.
- AST and parser for `from`, `filter`, `select`, `derive`, `sort`, and `take`.
- Newline and pipe-separated query syntax.
- Generic SQL, SQLite, and PostgreSQL targets.
- Expression normalisation for common RockQL operators and literals.
- Canonical formatter.
- CLI commands: `compile`, `check`, `format`, and `ast`.
- Parser and SQL generation tests.
- Eight GitHub Actions entry points for CI, Rust tests, web, Android, WebAssembly, releases, security, and documentation.

## CLI

```bash
cargo run -p rockql-cli -- compile examples/employees.rockql --target postgres
cargo run -p rockql-cli -- check examples/employees.rockql
cargo run -p rockql-cli -- format examples/employees.rockql
cargo run -p rockql-cli -- ast examples/employees.rockql
```

Standard input is supported:

```bash
echo "from users | filter active == true | take 10" \
  | cargo run -q -p rockql-cli -- compile --target sqlite
```

Expected output:

```sql
SELECT
    *
FROM users
WHERE active = TRUE
LIMIT 10;
```

## Architecture

```text
RockQL source
    ↓
Segment scanner
    ↓
Parser
    ↓
Abstract syntax tree
    ↓
SQL generator
    ↓
Formatted SQL
```

```text
compiler/
├── rockql-ast/
├── rockql-parser/
├── rockql-sql/
├── rockql-formatter/
└── rockql-cli/
```

The resolver, relational IR, optimiser, WebAssembly binding, playground, language server, visual studio, Android app, and additional SQL targets will be added incrementally after the compiler core is stable.

## Roadmap

- **v0.1:** lexer/parser foundation, core transforms, SQLite, CLI, tests.
- **v0.2:** WebAssembly compiler and browser playground.
- **v0.3:** joins, grouping, aggregation, variables, functions, PostgreSQL and MySQL depth.
- **v0.4:** formatter maturity, language server, VS Code extension, Tree-sitter grammar.
- **v0.5:** visual pipeline, schema browser, DuckDB execution, local files and history.
- **v1.0:** stable specification, compatibility policy, signed releases, Android app, JavaScript and Python bindings.

## Project boundaries

Early RockQL versions focus on querying and transforming data. Database administration, migrations, writes (`INSERT`, `UPDATE`, `DELETE`), cloud credential storage, team collaboration, and paid AI services are intentionally out of scope.

## Relationship to PRQL

PRQL is an important prior project in the pipeline-query-language space. RockQL is being developed as a distinct implementation with its own name, branding, syntax decisions, product direction, and source code. Any future reuse of third-party source must preserve the original licence, copyright notices, and attribution requirements.

## Licence

Licensed under the Apache License 2.0. See [LICENSE](LICENSE).
