//! Canonical formatting for RockQL source.

use rockql_ast::{Query, SortDirection, Transform};

#[must_use]
pub fn format(query: &Query) -> String {
    let mut output = format!("from {}", query.source.name);
    for transform in &query.transforms {
        output.push('\n');
        match transform {
            Transform::Filter { expression, .. } => {
                output.push_str("filter ");
                output.push_str(expression.text.trim());
            }
            Transform::Select { columns, .. } => {
                output.push_str("select {");
                output.push_str(
                    &columns
                        .iter()
                        .map(|column| column.text.trim())
                        .collect::<Vec<_>>()
                        .join(", "),
                );
                output.push('}');
            }
            Transform::Derive {
                name, expression, ..
            } => {
                output.push_str("derive ");
                output.push_str(name);
                output.push_str(" = ");
                output.push_str(expression.text.trim());
            }
            Transform::Sort { items, .. } => {
                output.push_str("sort {");
                output.push_str(
                    &items
                        .iter()
                        .map(|item| {
                            let prefix = match item.direction {
                                SortDirection::Ascending => "+",
                                SortDirection::Descending => "-",
                            };
                            format!("{prefix}{}", item.expression.text.trim())
                        })
                        .collect::<Vec<_>>()
                        .join(", "),
                );
                output.push('}');
            }
            Transform::Take { count, .. } => {
                output.push_str("take ");
                output.push_str(&count.to_string());
            }
        }
    }
    output.push('\n');
    output
}
