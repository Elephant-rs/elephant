//! PostgreSQL-first, type-safe query construction and compilation.

mod ast;
mod postgres;
mod query;
mod schema;
mod value;

pub use ast::{Expression, Ordering, Projection};
pub use postgres::{CompiledQuery, Parameter};
pub use query::{IntoOrderings, IntoProjection, SelectQuery, select, select_all};
pub use schema::{Column, Identifier, Table};
pub use value::{ColumnValue, Value};

/// The original crate marker. Query construction does not require an instance
/// of this type; it remains available for compatibility with the initial API.
#[derive(Debug, Default, Clone, Copy)]
pub struct Elephant;

impl Elephant {
    pub const fn new() -> Self {
        Self
    }
}
