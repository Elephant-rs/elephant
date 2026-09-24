use crate::schema::{Identifier, Table};
use crate::value::Value;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ColumnRef {
    pub(crate) table: Table,
    pub(crate) name: Identifier,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Comparison {
    Eq,
    Ne,
    Gt,
    Gte,
    Lt,
    Lte,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Logical {
    And,
    Or,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Asc,
    Desc,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Expression(pub(crate) ExpressionNode);

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ExpressionNode {
    Column(ColumnRef),
    Parameter(Value),
    Comparison {
        operator: Comparison,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Logical {
        operator: Logical,
        left: Box<Expression>,
        right: Box<Expression>,
    },
}

impl Expression {
    pub fn and(self, other: Self) -> Self {
        Self(ExpressionNode::Logical {
            operator: Logical::And,
            left: Box::new(self),
            right: Box::new(other),
        })
    }

    pub fn or(self, other: Self) -> Self {
        Self(ExpressionNode::Logical {
            operator: Logical::Or,
            left: Box::new(self),
            right: Box::new(other),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ordering {
    pub(crate) column: ColumnRef,
    pub(crate) direction: Direction,
}
