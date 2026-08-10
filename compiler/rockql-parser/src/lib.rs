use rockql_ast::{Query, SortItem, Span, SpannedTransform, Transform};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub message: String,
    pub span: Span,
}

impl Diagnostic {
    fn new(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span,
        }
    }
}

impl Display for Diagnostic {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}:{}: {}",
            self.span.line, self.span.column, self.message
        )
    }
}

#[derive(Debug)]
struct Segment {
    text: String,
    span: Span,
}

pub fn parse(source: &str) -> Result<Query, Vec<Diagnostic>> {
    let segments = split_segments(source);

    if segments.is_empty() {
        return Err(vec![Diagnostic::new(
            "expected at least one RockQL transformation",
            Span::new(1, 1),
        )]);
    }

    let mut transforms = Vec::with_capacity(segments.len());
    let mut diagnostics = Vec::new();

    for segment in segments {
        match parse_transform(&segment.text, segment.span) {
            Ok(transform) => transforms.push(SpannedTransform::new(segment.span, transform)),
            Err(diagnostic) => diagnostics.push(diagnostic),
        }
    }

    if diagnostics.is_empty() {
        Ok(Query::new(transforms))
    } else {
        Err(diagnostics)
    }
}

pub fn format_source(source: &str) -> Result<String, Vec<Diagnostic>> {
    parse(source).map(|query| format!("{query}\n"))
}

fn split_segments(source: &str) -> Vec<Segment> {
    let mut segments = Vec::new();

    for (line_index, line) in source.lines().enumerate() {
        let mut start = 0;

        for (byte_index, character) in line.char_indices() {
            if character == '|' {
                push_segment(
                    &mut segments,
                    &line[start..byte_index],
                    line_index + 1,
                    start,
                );
                start = byte_index + character.len_utf8();
            }
        }

        push_segment(&mut segments, &line[start..], line_index + 1, start);
    }

    segments
}

fn push_segment(segments: &mut Vec<Segment>, raw: &str, line: usize, byte_start: usize) {
    let text = raw.trim();
    if text.is_empty() {
        return;
    }

    let leading_bytes = raw.find(text).unwrap_or(0);
    segments.push(Segment {
        text: text.to_owned(),
        span: Span::new(line, byte_start + leading_bytes + 1),
    });
}

fn parse_transform(text: &str, span: Span) -> Result<Transform, Diagnostic> {
    let keyword_end = text
        .char_indices()
        .find_map(|(index, character)| character.is_whitespace().then_some(index))
        .unwrap_or(text.len());

    let keyword = &text[..keyword_end];
    let rest = text[keyword_end..].trim();

    match keyword {
        "from" => parse_from(rest, span),
        "filter" => parse_filter(rest, span),
        "select" => parse_select(rest, span),
        "derive" => parse_derive(rest, span),
        "sort" => parse_sort(rest, span),
        "take" => parse_take(rest, span),
        _ => Err(Diagnostic::new(
            format!("unknown transformation `{keyword}`"),
            span,
        )),
    }
}

fn parse_from(rest: &str, span: Span) -> Result<Transform, Diagnostic> {
    require_value(rest, "expected a table or source after `from`", span)?;

    Ok(Transform::From {
        source: rest.to_owned(),
    })
}

fn parse_filter(rest: &str, span: Span) -> Result<Transform, Diagnostic> {
    require_value(rest, "expected an expression after `filter`", span)?;

    Ok(Transform::Filter {
        expression: rest.to_owned(),
    })
}

fn parse_select(rest: &str, span: Span) -> Result<Transform, Diagnostic> {
    require_value(rest, "expected one or more columns after `select`", span)?;

    let columns = rest
        .split(',')
        .map(str::trim)
        .filter(|column| !column.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();

    if columns.is_empty() {
        return Err(Diagnostic::new(
            "expected one or more columns after `select`",
            span,
        ));
    }

    Ok(Transform::Select { columns })
}

fn parse_derive(rest: &str, span: Span) -> Result<Transform, Diagnostic> {
    let Some((name, expression)) = rest.split_once('=') else {
        return Err(Diagnostic::new("expected `derive name = expression`", span));
    };

    let name = name.trim();
    let expression = expression.trim();

    require_value(name, "expected a derived column name", span)?;
    require_value(expression, "expected a derived column expression", span)?;

    Ok(Transform::Derive {
        name: name.to_owned(),
        expression: expression.to_owned(),
    })
}

fn parse_sort(rest: &str, span: Span) -> Result<Transform, Diagnostic> {
    require_value(rest, "expected one or more sort expressions", span)?;

    let inner = match (rest.strip_prefix('{'), rest.strip_suffix('}')) {
        (Some(without_open), Some(_)) => &without_open[..without_open.len().saturating_sub(1)],
        (None, None) => rest,
        _ => return Err(Diagnostic::new("sort braces must be balanced", span)),
    };

    let items = inner
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(|item| {
            if let Some(expression) = item.strip_prefix('-') {
                SortItem::descending(expression.trim())
            } else if let Some(expression) = item.strip_prefix('+') {
                SortItem::ascending(expression.trim())
            } else {
                SortItem::ascending(item)
            }
        })
        .collect::<Vec<_>>();

    if items.is_empty() || items.iter().any(|item| item.expression.is_empty()) {
        return Err(Diagnostic::new(
            "expected one or more sort expressions",
            span,
        ));
    }

    Ok(Transform::Sort { items })
}

fn parse_take(rest: &str, span: Span) -> Result<Transform, Diagnostic> {
    require_value(rest, "expected a row count after `take`", span)?;

    // ⚡ Bolt Optimization: Manually parse the integer without allocating an intermediate
    // String via `.replace('_', "")`. This directly iterates over bytes, ignoring `_`.
    let mut count: u64 = 0;
    let mut has_digits = false;
    for &byte in rest.as_bytes() {
        if byte == b'_' {
            continue;
        } else if byte.is_ascii_digit() {
            has_digits = true;
            let digit = (byte - b'0') as u64;
            count = count
                .checked_mul(10)
                .and_then(|c| c.checked_add(digit))
                .ok_or_else(|| {
                    Diagnostic::new("`take` requires a non-negative integer row count", span)
                })?;
        } else {
            return Err(Diagnostic::new(
                "`take` requires a non-negative integer row count",
                span,
            ));
        }
    }

    if !has_digits {
        return Err(Diagnostic::new(
            "`take` requires a non-negative integer row count",
            span,
        ));
    }

    Ok(Transform::Take { count })
}

fn require_value(value: &str, message: &str, span: Span) -> Result<(), Diagnostic> {
    if value.trim().is_empty() {
        Err(Diagnostic::new(message, span))
    } else {
        Ok(())
    }
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

        assert_eq!(query.transforms.len(), 5);
        assert_eq!(query.transforms[0].span, Span::new(1, 1));
        assert!(matches!(
            query.transforms[4].node,
            Transform::Take { count: 10 }
        ));
    }

    #[test]
    fn parses_pipe_separated_pipeline() {
        let query =
            parse("from users | filter active == true | take 10").expect("query should parse");

        assert_eq!(query.transforms.len(), 3);
        assert_eq!(query.transforms[1].span, Span::new(1, 14));
    }

    #[test]
    fn reports_line_and_column() {
        let diagnostics =
            parse("from users\n  unknown value").expect_err("unknown transformation should fail");

        assert_eq!(diagnostics[0].span, Span::new(2, 3));
        assert!(diagnostics[0].message.contains("unknown transformation"));
    }

    #[test]
    fn formats_to_canonical_multiline_source() {
        let formatted =
            format_source("from users | sort {-created_at} | take 5").expect("query should format");

        assert_eq!(formatted, "from users\nsort {-created_at}\ntake 5\n");
    }
}
