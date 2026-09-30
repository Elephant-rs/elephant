use crate::ast::ProjectionNode;
use crate::ast::{
    ColumnRef, Comparison, Direction, Expression, ExpressionNode, Logical, SelectAst,
};
use crate::schema::{Identifier, Table};
use crate::value::Value;

pub type Parameter = Value;

/// SQL and ordered values produced by PostgreSQL compilation.
#[derive(Clone, Debug, PartialEq)]
pub struct CompiledQuery {
    pub sql: String,
    pub parameters: Vec<Parameter>,
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct PostgresCompiler;

impl PostgresCompiler {
    pub(crate) const fn new() -> Self {
        Self
    }

    pub(crate) fn compile(&self, query: &SelectAst) -> CompiledQuery {
        let mut state = Compilation {
            sql: String::from("SELECT "),
            parameters: Vec::new(),
        };

        match &query.projection.0 {
            ProjectionNode::All => state.sql.push('*'),
            ProjectionNode::Columns(columns) => {
                for (index, column) in columns.iter().enumerate() {
                    if index > 0 {
                        state.sql.push_str(", ");
                    }
                    state.column(column);
                }
            }
        }

        if let Some(table) = &query.from {
            state.sql.push_str(" FROM ");
            state.table(table);
        }

        if let Some(selection) = &query.selection {
            state.sql.push_str(" WHERE ");
            state.expression(selection, 0);
        }

        if !query.order_by.is_empty() {
            state.sql.push_str(" ORDER BY ");
            for (index, ordering) in query.order_by.iter().enumerate() {
                if index > 0 {
                    state.sql.push_str(", ");
                }
                state.column(&ordering.column);
                state.sql.push_str(match ordering.direction {
                    Direction::Asc => " ASC",
                    Direction::Desc => " DESC",
                });
            }
        }

        if let Some(limit) = query.limit {
            state.sql.push_str(" LIMIT ");
            state.sql.push_str(&limit.get().to_string());
        }

        if let Some(offset) = query.offset {
            state.sql.push_str(" OFFSET ");
            state.sql.push_str(&offset.get().to_string());
        }

        CompiledQuery {
            sql: state.sql,
            parameters: state.parameters,
        }
    }
}

struct Compilation {
    sql: String,
    parameters: Vec<Parameter>,
}

impl Compilation {
    fn table(&mut self, table: &Table) {
        if let Some(schema) = table.schema() {
            self.identifier(schema);
            self.sql.push('.');
        }
        self.identifier(table.name());
    }

    fn column(&mut self, column: &ColumnRef) {
        self.table(&column.table);
        self.sql.push('.');
        self.identifier(&column.name);
    }

    fn identifier(&mut self, identifier: &Identifier) {
        self.sql.push('"');
        for character in identifier.as_str().chars() {
            if character == '"' {
                self.sql.push_str("\"\"");
            } else {
                self.sql.push(character);
            }
        }
        self.sql.push('"');
    }

    fn expression(&mut self, expression: &Expression, parent_precedence: u8) {
        let precedence = expression_precedence(expression);
        let parenthesized = precedence < parent_precedence;
        if parenthesized {
            self.sql.push('(');
        }

        match &expression.0 {
            ExpressionNode::Column(column) => self.column(column),
            ExpressionNode::Parameter(value) => {
                self.parameters.push(value.clone());
                self.sql.push('$');
                self.sql.push_str(&self.parameters.len().to_string());
            }
            ExpressionNode::Comparison {
                operator,
                left,
                right,
            } => {
                self.expression(left, precedence);
                self.sql.push_str(match operator {
                    Comparison::Eq => " = ",
                    Comparison::Ne => " <> ",
                    Comparison::Gt => " > ",
                    Comparison::Gte => " >= ",
                    Comparison::Lt => " < ",
                    Comparison::Lte => " <= ",
                });
                self.expression(right, precedence);
            }
            ExpressionNode::Logical {
                operator,
                left,
                right,
            } => {
                self.expression(left, precedence);
                self.sql.push_str(match operator {
                    Logical::And => " AND ",
                    Logical::Or => " OR ",
                });
                self.expression(right, precedence + 1);
            }
        }

        if parenthesized {
            self.sql.push(')');
        }
    }
}

fn expression_precedence(expression: &Expression) -> u8 {
    match &expression.0 {
        ExpressionNode::Logical {
            operator: Logical::Or,
            ..
        } => 1,
        ExpressionNode::Logical {
            operator: Logical::And,
            ..
        } => 2,
        ExpressionNode::Comparison { .. }
        | ExpressionNode::Column(_)
        | ExpressionNode::Parameter(_) => 3,
    }
}

#[cfg(test)]
mod tests {
    use crate::{Table, select_all};

    #[test]
    fn compiler_quotes_embedded_identifier_quotes() {
        let table = Table::new("some\"table");
        let query = select_all().from(table).compile();

        assert_eq!(query.sql, "SELECT * FROM \"some\"\"table\"");
    }

    #[test]
    fn compiler_state_is_local_to_each_operation() {
        let table = Table::new("users");
        let query = select_all().from(table);

        assert_eq!(query.compile(), query.compile());
    }
}
