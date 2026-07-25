//! SQL generation for RockQL queries.

use std::error::Error;
use std::fmt::{self, Display, Formatter};

use rockql_ast::{Query, SortDirection, Transform};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dialect {
    Generic,
    Sqlite,
    Postgres,
}

impl Dialect {
    pub fn from_name(value: &str) -> Result<Self, SqlError> {
        match value.to_ascii_lowercase().as_str() {
            "generic" | "sql" => Ok(Self::Generic),
            "sqlite" => Ok(Self::Sqlite),
            "postgres" | "postgresql" => Ok(Self::Postgres),
            _ => Err(SqlError(format!(
                "unsupported SQL target `{value}`; expected generic, sqlite, or postgres"
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlError(pub String);

impl Display for SqlError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for SqlError {}

/// Compile a RockQL AST into formatted SQL.
pub fn compile(query: &Query, dialect: Dialect) -> Result<String, SqlError> {
    if query.source.name.trim().is_empty() {
        return Err(SqlError("query source cannot be empty".to_owned()));
    }

    let mut projections = vec!["*".to_owned()];
    let mut filters = Vec::new();
    let mut sorting = Vec::new();
    let mut limit = None;

    for transform in &query.transforms {
        match transform {
            Transform::Filter { expression, .. } => {
                filters.push(normalize_expression(&expression.text, dialect));
            }
            Transform::Select { columns, .. } => {
                projections = columns
                    .iter()
                    .map(|column| normalize_expression(&column.text, dialect))
                    .collect();
            }
            Transform::Derive {
                name, expression, ..
            } => {
                let expression = normalize_expression(&expression.text, dialect);
                projections.push(format!("{expression} AS {name}"));
            }
            Transform::Sort { items, .. } => {
                sorting = items
                    .iter()
                    .map(|item| {
                        let expression = normalize_expression(&item.expression.text, dialect);
                        let direction = match item.direction {
                            SortDirection::Ascending => "ASC",
                            SortDirection::Descending => "DESC",
                        };
                        format!("{expression} {direction}")
                    })
                    .collect();
            }
            Transform::Take { count, .. } => limit = Some(*count),
        }
    }

    let mut sql = String::from("SELECT\n");
    for (index, projection) in projections.iter().enumerate() {
        sql.push_str("    ");
        sql.push_str(projection);
        if index + 1 != projections.len() {
            sql.push(',');
        }
        sql.push('\n');
    }
    sql.push_str("FROM ");
    sql.push_str(&query.source.name);

    if !filters.is_empty() {
        sql.push_str("\nWHERE ");
        for (index, filter) in filters.iter().enumerate() {
            if index > 0 {
                sql.push_str("\n    AND ");
            }
            sql.push_str(filter);
        }
    }

    if !sorting.is_empty() {
        sql.push_str("\nORDER BY ");
        sql.push_str(&sorting.join(", "));
    }

    if let Some(count) = limit {
        sql.push_str("\nLIMIT ");
        sql.push_str(&count.to_string());
    }

    sql.push(';');
    Ok(sql)
}

fn normalize_expression(expression: &str, dialect: Dialect) -> String {
    if let Some((left, right)) = split_top_level_coalesce(expression) {
        return format!(
            "COALESCE({}, {})",
            normalize_expression(left, dialect),
            normalize_expression(right, dialect)
        );
    }

    let mut output = String::with_capacity(expression.len());
    let characters: Vec<char> = expression.chars().collect();
    let mut index = 0;
    let mut quote = None;

    while index < characters.len() {
        let character = characters[index];
        if let Some(active_quote) = quote {
            output.push(character);
            if character == active_quote {
                quote = None;
            } else if character == '\\' && index + 1 < characters.len() {
                index += 1;
                output.push(characters[index]);
            }
            index += 1;
            continue;
        }

        if matches!(character, '\'' | '"') {
            quote = Some(character);
            output.push(character);
            index += 1;
            continue;
        }

        if character == '=' && characters.get(index + 1) == Some(&'=') {
            output.push('=');
            index += 2;
            continue;
        }

        if character.is_ascii_digit() {
            while index < characters.len()
                && (characters[index].is_ascii_digit()
                    || characters[index] == '_'
                    || characters[index] == '.')
            {
                if characters[index] != '_' {
                    output.push(characters[index]);
                }
                index += 1;
            }
            continue;
        }

        if character == '_' || character.is_ascii_alphabetic() {
            let start = index;
            index += 1;
            while index < characters.len()
                && (characters[index] == '_' || characters[index].is_ascii_alphanumeric())
            {
                index += 1;
            }
            let word: String = characters[start..index].iter().collect();
            let replacement = match word.to_ascii_lowercase().as_str() {
                "and" => "AND".to_owned(),
                "or" => "OR".to_owned(),
                "not" => "NOT".to_owned(),
                "null" => "NULL".to_owned(),
                "true" => boolean_literal(true, dialect).to_owned(),
                "false" => boolean_literal(false, dialect).to_owned(),
                _ => word,
            };
            output.push_str(&replacement);
            continue;
        }

        output.push(character);
        index += 1;
    }

    output
}

fn boolean_literal(value: bool, _dialect: Dialect) -> &'static str {
    if value { "TRUE" } else { "FALSE" }
}

fn split_top_level_coalesce(expression: &str) -> Option<(&str, &str)> {
    let bytes = expression.as_bytes();
    let mut index = 0;
    let mut quote = None;
    let mut depth = 0_u32;
    while index + 1 < bytes.len() {
        let byte = bytes[index];
        if let Some(active_quote) = quote {
            if byte == b'\\' {
                index += 2;
                continue;
            }
            if byte == active_quote {
                quote = None;
            }
            index += 1;
            continue;
        }
        match byte {
            b'\'' | b'"' => quote = Some(byte),
            b'(' => depth += 1,
            b')' => depth = depth.saturating_sub(1),
            b'?' if depth == 0 && bytes[index + 1] == b'?' => {
                return Some((expression[..index].trim(), expression[index + 2..].trim()));
            }
            _ => {}
        }
        index += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use rockql_parser::parse;

    use super::*;

    #[test]
    fn compiles_core_example() {
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
    fn normalizes_boolean_and_equality() {
        let query = parse("from users | filter active == true | take 10")
            .expect("query should parse");
        let sql = compile(&query, Dialect::Sqlite).expect("query should compile");
        assert!(sql.contains("WHERE active = TRUE"));
    }
}
