//! Abstract syntax tree for the RockQL language.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub line: usize,
    pub column: usize,
}

impl Span {
    #[must_use]
    pub const fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Query {
    pub source: Source,
    pub transforms: Vec<Transform>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expr {
    pub text: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Transform {
    Filter {
        expression: Expr,
        span: Span,
    },
    Select {
        columns: Vec<Expr>,
        span: Span,
    },
    Derive {
        name: String,
        expression: Expr,
        span: Span,
    },
    Sort {
        items: Vec<SortItem>,
        span: Span,
    },
    Take {
        count: u64,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortItem {
    pub expression: Expr,
    pub direction: SortDirection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}
