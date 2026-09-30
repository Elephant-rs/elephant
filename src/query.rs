use crate::Expression;
use crate::ast::{NonNegative, Ordering, Projection, SelectAst};
use crate::postgres::{CompiledQuery, PostgresCompiler};
use crate::schema::{Column, Table};

/// Converts one or more typed columns into a SELECT projection.
mod sealed {
    pub trait Projection {}
    pub trait Orderings {}
}

pub trait IntoProjection: sealed::Projection {
    fn into_projection(self) -> Projection;
}

impl<T> sealed::Projection for Column<T> {}

impl<T> IntoProjection for Column<T> {
    fn into_projection(self) -> Projection {
        Projection::columns(vec![self.reference()])
    }
}

impl<T> sealed::Projection for &Column<T> {}

impl<T> IntoProjection for &Column<T> {
    fn into_projection(self) -> Projection {
        Projection::columns(vec![self.reference()])
    }
}

impl<A, B> sealed::Projection for (Column<A>, Column<B>) {}

impl<A, B> IntoProjection for (Column<A>, Column<B>) {
    fn into_projection(self) -> Projection {
        Projection::columns(vec![self.0.reference(), self.1.reference()])
    }
}

impl<'columns, A, B> sealed::Projection for (&'columns Column<A>, &'columns Column<B>) {}

impl<'columns, A, B> IntoProjection for (&'columns Column<A>, &'columns Column<B>) {
    fn into_projection(self) -> Projection {
        Projection::columns(vec![self.0.reference(), self.1.reference()])
    }
}

impl<A, B, C> sealed::Projection for (Column<A>, Column<B>, Column<C>) {}

impl<A, B, C> IntoProjection for (Column<A>, Column<B>, Column<C>) {
    fn into_projection(self) -> Projection {
        Projection::columns(vec![
            self.0.reference(),
            self.1.reference(),
            self.2.reference(),
        ])
    }
}

impl<'columns, A, B, C> sealed::Projection
    for (
        &'columns Column<A>,
        &'columns Column<B>,
        &'columns Column<C>,
    )
{
}

impl<'columns, A, B, C> IntoProjection
    for (
        &'columns Column<A>,
        &'columns Column<B>,
        &'columns Column<C>,
    )
{
    fn into_projection(self) -> Projection {
        Projection::columns(vec![
            self.0.reference(),
            self.1.reference(),
            self.2.reference(),
        ])
    }
}

impl<A, B, C, D> sealed::Projection for (Column<A>, Column<B>, Column<C>, Column<D>) {}

impl<A, B, C, D> IntoProjection for (Column<A>, Column<B>, Column<C>, Column<D>) {
    fn into_projection(self) -> Projection {
        Projection::columns(vec![
            self.0.reference(),
            self.1.reference(),
            self.2.reference(),
            self.3.reference(),
        ])
    }
}

impl<'columns, A, B, C, D> sealed::Projection
    for (
        &'columns Column<A>,
        &'columns Column<B>,
        &'columns Column<C>,
        &'columns Column<D>,
    )
{
}

impl<'columns, A, B, C, D> IntoProjection
    for (
        &'columns Column<A>,
        &'columns Column<B>,
        &'columns Column<C>,
        &'columns Column<D>,
    )
{
    fn into_projection(self) -> Projection {
        Projection::columns(vec![
            self.0.reference(),
            self.1.reference(),
            self.2.reference(),
            self.3.reference(),
        ])
    }
}

/// Converts one or more orderings into a list for ORDER BY.
pub trait IntoOrderings: sealed::Orderings {
    fn into_orderings(self) -> Vec<Ordering>;
}

impl sealed::Orderings for Ordering {}

impl IntoOrderings for Ordering {
    fn into_orderings(self) -> Vec<Ordering> {
        vec![self]
    }
}

impl sealed::Orderings for (Ordering, Ordering) {}

impl IntoOrderings for (Ordering, Ordering) {
    fn into_orderings(self) -> Vec<Ordering> {
        vec![self.0, self.1]
    }
}

impl sealed::Orderings for (Ordering, Ordering, Ordering) {}

impl IntoOrderings for (Ordering, Ordering, Ordering) {
    fn into_orderings(self) -> Vec<Ordering> {
        vec![self.0, self.1, self.2]
    }
}

impl sealed::Orderings for Vec<Ordering> {}

impl IntoOrderings for Vec<Ordering> {
    fn into_orderings(self) -> Vec<Ordering> {
        self
    }
}

/// A semantic SELECT query under construction.
#[derive(Clone, Debug, PartialEq)]
pub struct SelectQuery {
    pub(crate) ast: SelectAst,
}

impl SelectQuery {
    pub fn from(mut self, table: Table) -> Self {
        self.ast.from = Some(table);
        self
    }

    pub fn where_(mut self, expression: Expression) -> Self {
        self.ast.selection = Some(expression);
        self
    }

    pub fn order_by<O>(mut self, ordering: O) -> Self
    where
        O: IntoOrderings,
    {
        self.ast.order_by = ordering.into_orderings();
        self
    }

    pub fn limit(mut self, value: u64) -> Self {
        self.ast.limit = Some(NonNegative::new(value));
        self
    }

    pub fn offset(mut self, value: u64) -> Self {
        self.ast.offset = Some(NonNegative::new(value));
        self
    }

    pub fn compile(&self) -> CompiledQuery {
        PostgresCompiler::new().compile(&self.ast)
    }
}

pub fn select<P>(projection: P) -> SelectQuery
where
    P: IntoProjection,
{
    SelectQuery {
        ast: SelectAst::new(projection.into_projection()),
    }
}

pub fn select_all() -> SelectQuery {
    SelectQuery {
        ast: SelectAst::new(Projection::all()),
    }
}
