use crate::ast::{ColumnRef, Expression, Ordering};
use crate::schema::Table;

/// A nonnegative integer used by LIMIT and OFFSET.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct NonNegative(u64);

impl NonNegative {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Projection(pub(crate) ProjectionNode);

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ProjectionNode {
    All,
    Columns(Vec<ColumnRef>),
}

/// Semantic representation of a SELECT query. It contains no SQL text.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SelectAst {
    pub(crate) projection: Projection,
    pub(crate) from: Option<Table>,
    pub(crate) selection: Option<Expression>,
    pub(crate) order_by: Vec<Ordering>,
    pub(crate) limit: Option<NonNegative>,
    pub(crate) offset: Option<NonNegative>,
}

impl SelectAst {
    pub(crate) fn new(projection: Projection) -> Self {
        Self {
            projection,
            from: None,
            selection: None,
            order_by: Vec::new(),
            limit: None,
            offset: None,
        }
    }
}

impl Projection {
    pub(crate) fn all() -> Self {
        Self(ProjectionNode::All)
    }

    pub(crate) fn columns(columns: Vec<ColumnRef>) -> Self {
        Self(ProjectionNode::Columns(columns))
    }
}
