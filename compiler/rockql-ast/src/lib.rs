use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub const fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Query {
    pub transforms: Vec<SpannedTransform>,
}

impl Query {
    pub fn new(transforms: Vec<SpannedTransform>) -> Self {
        Self { transforms }
    }
}

impl Display for Query {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        for (index, transform) in self.transforms.iter().enumerate() {
            if index > 0 {
                writeln!(formatter)?;
            }
            write!(formatter, "{}", transform.node)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpannedTransform {
    pub span: Span,
    pub node: Transform,
}

impl SpannedTransform {
    pub const fn new(span: Span, node: Transform) -> Self {
        Self { span, node }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Transform {
    From { source: String },
    Filter { expression: String },
    Select { columns: Vec<String> },
    Derive { name: String, expression: String },
    Sort { items: Vec<SortItem> },
    Take { count: u64 },
}

impl Display for Transform {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::From { source } => write!(formatter, "from {source}"),
            Self::Filter { expression } => write!(formatter, "filter {expression}"),
            Self::Select { columns } => {
                // ⚡ Bolt Optimization: Prevent intermediate String and Vec allocation
                // when formatting selected columns.
                write!(formatter, "select ")?;
                for (index, column) in columns.iter().enumerate() {
                    if index > 0 {
                        write!(formatter, ", ")?;
                    }
                    write!(formatter, "{}", column)?;
                }
                Ok(())
            }
            Self::Derive { name, expression } => {
                write!(formatter, "derive {name} = {expression}")
            }
            Self::Sort { items } => {
                // ⚡ Bolt Optimization: Prevent temporary vectors and string allocations
                // by directly writing sort items to the formatter.
                write!(formatter, "sort {{")?;
                for (index, item) in items.iter().enumerate() {
                    if index > 0 {
                        write!(formatter, ", ")?;
                    }
                    write!(formatter, "{}", item)?;
                }
                write!(formatter, "}}")
            }
            Self::Take { count } => write!(formatter, "take {count}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SortItem {
    pub expression: String,
    pub descending: bool,
}

impl SortItem {
    pub fn ascending(expression: impl Into<String>) -> Self {
        Self {
            expression: expression.into(),
            descending: false,
        }
    }

    pub fn descending(expression: impl Into<String>) -> Self {
        Self {
            expression: expression.into(),
            descending: true,
        }
    }
}

impl Display for SortItem {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        if self.descending {
            write!(formatter, "-{}", self.expression)
        } else {
            write!(formatter, "{}", self.expression)
        }
    }
}
