//! Parser for the RockQL MVP syntax.

use std::error::Error;
use std::fmt::{self, Display, Formatter};

use rockql_ast::{Expr, Query, SortDirection, SortItem, Source, Span, Transform};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
    pub hint: Option<String>,
}

impl ParseError {
    fn new(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span,
            hint: None,
        }
    }

    fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }
}

impl Display for ParseError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at line {}, column {}",
            self.message, self.span.line, self.span.column
        )?;
        if let Some(hint) = &self.hint {
            write!(formatter, "\nhelp: {hint}")?;
        }
        Ok(())
    }
}

impl Error for ParseError {}

#[derive(Debug, Clone)]
struct Segment {
    text: String,
    span: Span,
}

/// Parse RockQL source into an AST.
///
/// Both newline-separated pipelines and `|`-separated pipelines are accepted.
pub fn parse(source: &str) -> Result<Query, ParseError> {
    let segments = split_segments(source);
    let first = segments.first().ok_or_else(|| {
        ParseError::new("query is empty", Span::new(1, 1))
            .with_hint("start a query with `from <table>`")
    })?;

    let source_name = payload(first, "from").ok_or_else(|| {
        ParseError::new("a query must start with `from`", first.span)
            .with_hint("example: `from employees`")
    })?;
    validate_identifier_path(source_name, first.span, "source")?;

    let mut transforms = Vec::new();
    for segment in segments.iter().skip(1) {
        transforms.push(parse_transform(segment)?);
    }

    Ok(Query {
        source: Source {
            name: source_name.to_owned(),
            span: first.span,
        },
        transforms,
    })
}

fn parse_transform(segment: &Segment) -> Result<Transform, ParseError> {
    if payload(segment, "from").is_some() {
        return Err(ParseError::new(
            "`from` may only appear at the start of a query",
            segment.span,
        ));
    }

    if let Some(text) = payload(segment, "filter") {
        require_non_empty(text, segment.span, "filter expression")?;
        return Ok(Transform::Filter {
            expression: Expr {
                text: text.to_owned(),
                span: segment.span,
            },
            span: segment.span,
        });
    }

    if let Some(text) = payload(segment, "select") {
        let columns = parse_expression_list(text, segment.span, "select")?;
        return Ok(Transform::Select {
            columns,
            span: segment.span,
        });
    }

    if let Some(text) = payload(segment, "derive") {
        let (name, expression) = split_assignment(text).ok_or_else(|| {
            ParseError::new("invalid `derive` transformation", segment.span)
                .with_hint("use `derive column_name = expression`")
        })?;
        validate_identifier(name, segment.span, "derived column")?;
        require_non_empty(expression, segment.span, "derive expression")?;
        return Ok(Transform::Derive {
            name: name.to_owned(),
            expression: Expr {
                text: expression.to_owned(),
                span: segment.span,
            },
            span: segment.span,
        });
    }

    if let Some(text) = payload(segment, "sort") {
        let body = strip_optional_braces(text, segment.span, "sort")?;
        let mut items = Vec::new();
        for item in split_commas(body) {
            let item = item.trim();
            require_non_empty(item, segment.span, "sort expression")?;
            let (direction, expression) = match item.as_bytes().first() {
                Some(b'-') => (SortDirection::Descending, item[1..].trim()),
                Some(b'+') => (SortDirection::Ascending, item[1..].trim()),
                _ => (SortDirection::Ascending, item),
            };
            require_non_empty(expression, segment.span, "sort expression")?;
            items.push(SortItem {
                expression: Expr {
                    text: expression.to_owned(),
                    span: segment.span,
                },
                direction,
            });
        }
        return Ok(Transform::Sort {
            items,
            span: segment.span,
        });
    }

    if let Some(text) = payload(segment, "take") {
        let normalized = text.replace('_', "");
        let count = normalized.parse::<u64>().map_err(|_| {
            ParseError::new("`take` requires a non-negative integer", segment.span)
                .with_hint("example: `take 10`")
        })?;
        return Ok(Transform::Take {
            count,
            span: segment.span,
        });
    }

    let keyword = segment.text.split_whitespace().next().unwrap_or_default();
    Err(
        ParseError::new(format!("unknown transformation `{keyword}`"), segment.span)
            .with_hint("MVP transformations: filter, select, derive, sort, take"),
    )
}

fn parse_expression_list(text: &str, span: Span, transform: &str) -> Result<Vec<Expr>, ParseError> {
    let body = strip_optional_braces(text, span, transform)?;
    let mut expressions = Vec::new();
    for item in split_commas(body) {
        let item = item.trim();
        require_non_empty(item, span, "expression")?;
        expressions.push(Expr {
            text: item.to_owned(),
            span,
        });
    }
    if expressions.is_empty() {
        return Err(ParseError::new(
            format!("`{transform}` requires at least one expression"),
            span,
        ));
    }
    Ok(expressions)
}

fn payload<'a>(segment: &'a Segment, keyword: &str) -> Option<&'a str> {
    let text = segment.text.trim();
    if text == keyword {
        return Some("");
    }
    let rest = text.strip_prefix(keyword)?;
    if rest.chars().next().is_some_and(char::is_whitespace) {
        Some(rest.trim())
    } else {
        None
    }
}

fn require_non_empty(text: &str, span: Span, label: &str) -> Result<(), ParseError> {
    if text.trim().is_empty() {
        Err(ParseError::new(format!("missing {label}"), span))
    } else {
        Ok(())
    }
}

fn strip_optional_braces<'a>(
    text: &'a str,
    span: Span,
    transform: &str,
) -> Result<&'a str, ParseError> {
    let text = text.trim();
    match (text.starts_with('{'), text.ends_with('}')) {
        (true, true) => Ok(text[1..text.len() - 1].trim()),
        (false, false) => Ok(text),
        _ => Err(ParseError::new(
            format!("unbalanced braces in `{transform}`"),
            span,
        )),
    }
}

fn split_assignment(text: &str) -> Option<(&str, &str)> {
    let bytes = text.as_bytes();
    let mut quote = None;
    let mut escaped = false;
    for (index, character) in text.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if character == '\\' && quote.is_some() {
            escaped = true;
            continue;
        }
        if matches!(character, '\'' | '"') {
            if quote == Some(character) {
                quote = None;
            } else if quote.is_none() {
                quote = Some(character);
            }
            continue;
        }
        if character == '=' && quote.is_none() {
            let previous = index
                .checked_sub(1)
                .and_then(|value| bytes.get(value))
                .copied();
            let next = bytes.get(index + 1).copied();
            if previous.is_some_and(|value| matches!(value, b'=' | b'!' | b'<' | b'>'))
                || next == Some(b'=')
            {
                continue;
            }
            return Some((text[..index].trim(), text[index + 1..].trim()));
        }
    }
    None
}

fn split_commas(text: &str) -> Vec<&str> {
    let mut values = Vec::new();
    let mut start = 0;
    let mut quote = None;
    let mut depth = 0_u32;
    let mut escaped = false;
    for (index, character) in text.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if character == '\\' && quote.is_some() {
            escaped = true;
            continue;
        }
        if matches!(character, '\'' | '"') {
            if quote == Some(character) {
                quote = None;
            } else if quote.is_none() {
                quote = Some(character);
            }
            continue;
        }
        if quote.is_some() {
            continue;
        }
        match character {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                values.push(&text[start..index]);
                start = index + 1;
            }
            _ => {}
        }
    }
    values.push(&text[start..]);
    values
}

fn validate_identifier_path(text: &str, span: Span, label: &str) -> Result<(), ParseError> {
    if text.split('.').all(is_identifier) {
        Ok(())
    } else {
        Err(ParseError::new(format!("invalid {label} `{text}`"), span).with_hint(
            "identifiers must start with a letter or underscore and contain only letters, numbers, or underscores",
        ))
    }
}

fn validate_identifier(text: &str, span: Span, label: &str) -> Result<(), ParseError> {
    if is_identifier(text) {
        Ok(())
    } else {
        Err(ParseError::new(format!("invalid {label} `{text}`"), span))
    }
}

fn is_identifier(text: &str) -> bool {
    let mut characters = text.chars();
    matches!(characters.next(), Some(first) if first == '_' || first.is_ascii_alphabetic())
        && characters.all(|character| character == '_' || character.is_ascii_alphanumeric())
}

fn split_segments(source: &str) -> Vec<Segment> {
    let mut segments = Vec::new();
    for (line_index, raw_line) in source.lines().enumerate() {
        let line = raw_line.trim_end_matches('\r');
        let mut start = 0;
        let mut quote = None;
        let mut escaped = false;
        for (index, character) in line.char_indices() {
            if escaped {
                escaped = false;
                continue;
            }
            if character == '\\' && quote.is_some() {
                escaped = true;
                continue;
            }
            if matches!(character, '\'' | '"') {
                if quote == Some(character) {
                    quote = None;
                } else if quote.is_none() {
                    quote = Some(character);
                }
                continue;
            }
            if character == '|' && quote.is_none() {
                push_segment(
                    &mut segments,
                    &line[start..index],
                    line_index + 1,
                    start + 1,
                );
                start = index + 1;
            }
        }
        push_segment(&mut segments, &line[start..], line_index + 1, start + 1);
    }
    segments
}

fn push_segment(segments: &mut Vec<Segment>, text: &str, line: usize, base_column: usize) {
    let trimmed_start = text.trim_start();
    if trimmed_start.is_empty() || trimmed_start.starts_with('#') {
        return;
    }
    let leading = text.len() - trimmed_start.len();
    segments.push(Segment {
        text: trimmed_start.trim_end().to_owned(),
        span: Span::new(line, base_column + leading),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_multiline_pipeline() {
        let query = parse(
            "from employees\nfilter salary > 50_000\nderive yearly_salary = salary * 12\nsort {-yearly_salary}\ntake 10",
        )
        .expect("query should parse");

        assert_eq!(query.source.name, "employees");
        assert_eq!(query.transforms.len(), 4);
    }

    #[test]
    fn parses_pipe_pipeline() {
        let query =
            parse("from users | filter active == true | take 10").expect("query should parse");
        assert_eq!(query.transforms.len(), 2);
    }

    #[test]
    fn reports_unknown_transformation_position() {
        let error = parse("from users\nexplode profile").expect_err("query should fail");
        assert_eq!(error.span, Span::new(2, 1));
        assert!(error.message.contains("unknown transformation"));
    }
}
