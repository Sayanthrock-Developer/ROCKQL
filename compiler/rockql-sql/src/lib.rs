use rockql_ast::{Query, SortItem, Transform};
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dialect {
    Generic,
    Sqlite,
    Postgres,
}

impl Display for Dialect {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Generic => write!(formatter, "generic"),
            Self::Sqlite => write!(formatter, "sqlite"),
            Self::Postgres => write!(formatter, "postgres"),
        }
    }
}

impl FromStr for Dialect {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        // ⚡ Bolt Optimization: Use `eq_ignore_ascii_case` to avoid intermediate String allocation from `to_ascii_lowercase()`
        if value.eq_ignore_ascii_case("generic") || value.eq_ignore_ascii_case("sql") {
            Ok(Self::Generic)
        } else if value.eq_ignore_ascii_case("sqlite") || value.eq_ignore_ascii_case("sqlite3") {
            Ok(Self::Sqlite)
        } else if value.eq_ignore_ascii_case("postgres") || value.eq_ignore_ascii_case("postgresql")
        {
            Ok(Self::Postgres)
        } else {
            Err(format!(
                "unsupported SQL target `{value}`; expected generic, sqlite, or postgres"
            ))
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SqlError {
    #[error("query must start with a `from` transformation")]
    MissingFrom,
    #[error(
        "only one `from` transformation is currently supported (line {line}, column {column})"
    )]
    MultipleFrom { line: usize, column: usize },
}

pub fn compile(query: &Query, dialect: Dialect) -> Result<String, SqlError> {
    let Some(first) = query.transforms.first() else {
        return Err(SqlError::MissingFrom);
    };

    let Transform::From { source } = &first.node else {
        return Err(SqlError::MissingFrom);
    };

    let mut selected_columns: Option<Vec<String>> = None;
    let mut derived_columns = Vec::new();
    let mut filters = Vec::new();
    let mut sort_items = Vec::new();
    let mut limit = None;

    for transform in query.transforms.iter().skip(1) {
        match &transform.node {
            Transform::From { .. } => {
                return Err(SqlError::MultipleFrom {
                    line: transform.span.line,
                    column: transform.span.column,
                })
            }
            Transform::Filter { expression } => {
                filters.push(normalize_expression(expression, dialect));
            }
            Transform::Select { columns } => {
                selected_columns = Some(
                    columns
                        .iter()
                        .map(|column| normalize_expression(column, dialect))
                        .collect(),
                );
            }
            Transform::Derive { name, expression } => {
                derived_columns.push(format!(
                    "{} AS {}",
                    normalize_expression(expression, dialect),
                    name
                ));
            }
            Transform::Sort { items } => {
                sort_items.extend(items.iter().map(|item| compile_sort_item(item, dialect)));
            }
            Transform::Take { count } => limit = Some(*count),
        }
    }

    let mut select_items = selected_columns.unwrap_or_else(|| vec!["*".to_owned()]);
    select_items.extend(derived_columns);

    let mut sql = String::new();
    if select_items.len() == 1 {
        sql.push_str("SELECT ");
        sql.push_str(&select_items[0]);
        sql.push('\n');
    } else {
        sql.push_str("SELECT\n");
        for (index, item) in select_items.iter().enumerate() {
            sql.push_str("    ");
            sql.push_str(item);
            if index + 1 < select_items.len() {
                sql.push(',');
            }
            sql.push('\n');
        }
    }

    sql.push_str("FROM ");
    sql.push_str(source);
    sql.push('\n');

    if !filters.is_empty() {
        sql.push_str("WHERE ");
        sql.push_str(&filters.join("\n  AND "));
        sql.push('\n');
    }

    if !sort_items.is_empty() {
        sql.push_str("ORDER BY ");
        sql.push_str(&sort_items.join(", "));
        sql.push('\n');
    }

    if let Some(count) = limit {
        sql.push_str("LIMIT ");
        sql.push_str(&count.to_string());
        sql.push('\n');
    }

    if sql.ends_with('\n') {
        sql.pop();
    }
    sql.push(';');

    Ok(sql)
}

fn compile_sort_item(item: &SortItem, dialect: Dialect) -> String {
    let direction = if item.descending { "DESC" } else { "ASC" };
    format!(
        "{} {direction}",
        normalize_expression(&item.expression, dialect)
    )
}

fn normalize_expression(expression: &str, _dialect: Dialect) -> String {
    // ⚡ Bolt Optimization: Single-pass expression normalization.
    // Avoids 3 intermediate String allocations (`remove_numeric_separators`,
    // two `.replace()` calls) and consolidates string scanning.
    let mut output = String::with_capacity(expression.len());
    let mut word_start = None;
    let mut quote = None;

    let flush_word = |output: &mut String, start: usize| {
        let word = &output[start..];
        if word.eq_ignore_ascii_case("true") {
            output.replace_range(start.., "TRUE");
        } else if word.eq_ignore_ascii_case("false") {
            output.replace_range(start.., "FALSE");
        } else if word.eq_ignore_ascii_case("null") {
            output.replace_range(start.., "NULL");
        } else if word.eq_ignore_ascii_case("and") {
            output.replace_range(start.., "AND");
        } else if word.eq_ignore_ascii_case("or") {
            output.replace_range(start.., "OR");
        } else if word.eq_ignore_ascii_case("not") {
            output.replace_range(start.., "NOT");
        }
    };

    let bytes = expression.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        let b = bytes[i];

        let character = if b.is_ascii() {
            i += 1;
            b as char
        } else {
            let ch = expression[i..].chars().next().unwrap();
            i += ch.len_utf8();
            ch
        };

        if let Some(active_quote) = quote {
            output.push(character);
            if character == active_quote {
                quote = None;
            }
            continue;
        }

        if character == '\'' || character == '"' {
            if let Some(start) = word_start {
                flush_word(&mut output, start);
                word_start = None;
            }
            output.push(character);
            quote = Some(character);
        } else if character.is_ascii_alphanumeric() || character == '_' {
            // Handle numeric separators
            let current_i = if character.is_ascii() {
                i - 1
            } else {
                i - character.len_utf8()
            };
            if character == '_'
                && current_i > 0
                && current_i + 1 < bytes.len()
                && bytes[current_i - 1].is_ascii_digit()
                && bytes[current_i + 1].is_ascii_digit()
            {
                continue; // Skip the underscore
            }

            if word_start.is_none() {
                word_start = Some(output.len());
            }
            output.push(character);
        } else {
            if let Some(start) = word_start {
                flush_word(&mut output, start);
                word_start = None;
            }

            // Handle operators != and ==
            if character == '!' {
                if i < bytes.len() && bytes[i] == b'=' {
                    output.push_str("<>");
                    i += 1; // Consume '='
                } else {
                    output.push(character);
                }
            } else if character == '=' {
                if i < bytes.len() && bytes[i] == b'=' {
                    output.push('=');
                    i += 1; // Consume second '='
                } else {
                    output.push(character);
                }
            } else {
                output.push(character);
            }
        }
    }

    if let Some(start) = word_start {
        flush_word(&mut output, start);
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use rockql_parser::parse;

    #[test]
    fn compiles_the_readme_example() {
        let query = parse(
            "from employees\nfilter salary > 50_000\nderive yearly_salary = salary * 12\nsort {-yearly_salary}\ntake 10",
        )
        .expect("query should parse");

        let sql = compile(&query, Dialect::Postgres).expect("query should compile");

        assert_eq!(
            sql,
            "SELECT\n    *,\n    salary * 12 AS yearly_salary\nFROM employees\nWHERE salary > 50000\nORDER BY yearly_salary DESC\nLIMIT 10;"
        );
    }

    #[test]
    fn compiles_boolean_filter_for_sqlite() {
        let query =
            parse("from users | filter active == true | take 10").expect("query should parse");

        let sql = compile(&query, Dialect::Sqlite).expect("query should compile");

        assert_eq!(sql, "SELECT *\nFROM users\nWHERE active = TRUE\nLIMIT 10;");
    }

    #[test]
    fn preserves_keywords_inside_strings() {
        let query =
            parse("from events | filter label == \"true and false\"").expect("query should parse");

        let sql = compile(&query, Dialect::Generic).expect("query should compile");

        assert!(sql.contains("label = \"true and false\""));
    }
}
