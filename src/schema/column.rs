use std::marker::PhantomData;

use crate::ast::{ColumnRef, Direction, Expression, Ordering};
use crate::value::{ColumnValue, Value};

use super::{Identifier, Table};

/// A typed column belonging to a table.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Column<T> {
    pub(crate) table: Table,
    pub(crate) name: Identifier,
    marker: PhantomData<fn() -> T>,
}

impl<T> Clone for Column<T> {
    fn clone(&self) -> Self {
        Self {
            table: self.table.clone(),
            name: self.name.clone(),
            marker: PhantomData,
        }
    }
}

impl<T> Column<T> {
    pub fn new(table: Table, name: impl Into<String>) -> Self {
        Self {
            table,
            name: Identifier::new(name),
            marker: PhantomData,
        }
    }

    pub fn table(&self) -> &Table {
        &self.table
    }

    pub fn name(&self) -> &Identifier {
        &self.name
    }

    pub(crate) fn reference(&self) -> ColumnRef {
        ColumnRef {
            table: self.table.clone(),
            name: self.name.clone(),
        }
    }

    /// ```compile_fail
    /// use elephant::{Column, Table};
    /// let age = Column::<i32>::new(Table::new("users"), "age");
    /// let _invalid = age.eq("eighteen");
    /// ```
    pub fn eq<V>(&self, value: V) -> Expression
    where
        V: ColumnValue<T>,
    {
        self.compare(crate::ast::Comparison::Eq, value.into_value())
    }

    pub fn ne<V>(&self, value: V) -> Expression
    where
        V: ColumnValue<T>,
    {
        self.compare(crate::ast::Comparison::Ne, value.into_value())
    }

    pub fn gt<V>(&self, value: V) -> Expression
    where
        V: ColumnValue<T>,
    {
        self.compare(crate::ast::Comparison::Gt, value.into_value())
    }

    pub fn gte<V>(&self, value: V) -> Expression
    where
        V: ColumnValue<T>,
    {
        self.compare(crate::ast::Comparison::Gte, value.into_value())
    }

    pub fn lt<V>(&self, value: V) -> Expression
    where
        V: ColumnValue<T>,
    {
        self.compare(crate::ast::Comparison::Lt, value.into_value())
    }

    pub fn lte<V>(&self, value: V) -> Expression
    where
        V: ColumnValue<T>,
    {
        self.compare(crate::ast::Comparison::Lte, value.into_value())
    }

    pub fn asc(&self) -> Ordering {
        Ordering {
            column: self.reference(),
            direction: Direction::Asc,
        }
    }

    pub fn desc(&self) -> Ordering {
        Ordering {
            column: self.reference(),
            direction: Direction::Desc,
        }
    }

    fn compare(&self, comparison: crate::ast::Comparison, value: Value) -> Expression {
        Expression(crate::ast::ExpressionNode::Comparison {
            operator: comparison,
            left: Box::new(Expression(crate::ast::ExpressionNode::Column(
                self.reference(),
            ))),
            right: Box::new(Expression(crate::ast::ExpressionNode::Parameter(value))),
        })
    }
}
