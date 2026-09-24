use super::Identifier;
use super::column::Column;

/// A PostgreSQL table name, optionally qualified by a schema.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Table {
    pub(crate) schema: Option<Identifier>,
    pub(crate) name: Identifier,
}

impl Table {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            schema: None,
            name: Identifier::new(name),
        }
    }

    pub fn with_schema(schema: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            schema: Some(Identifier::new(schema)),
            name: Identifier::new(name),
        }
    }

    pub fn name(&self) -> &Identifier {
        &self.name
    }

    pub fn schema(&self) -> Option<&Identifier> {
        self.schema.as_ref()
    }

    pub fn column<T>(&self, name: impl Into<String>) -> Column<T> {
        Column::new(self.clone(), name)
    }
}
