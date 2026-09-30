use elephant::{Column, Parameter, Table, Value, select, select_all};

#[test]
fn compiles_typed_select_with_bindings() {
    let users = Table::new("users");
    let id = users.column::<i32>("id");
    let name = users.column::<String>("name");
    let active = users.column::<bool>("active");
    let age = users.column::<i32>("age");
    let created_at = users.column::<i64>("created_at");

    let query = select((&id, &name))
        .from(users)
        .where_(active.eq(true).and(age.gte(18)))
        .order_by(created_at.desc())
        .limit(20)
        .offset(2)
        .compile();

    assert_eq!(
        query.sql,
        "SELECT \"users\".\"id\", \"users\".\"name\" FROM \"users\" WHERE \"users\".\"active\" = $1 AND \"users\".\"age\" >= $2 ORDER BY \"users\".\"created_at\" DESC LIMIT 20 OFFSET 2"
    );
    assert_eq!(query.parameters, vec![Value::Bool(true), Value::Int(18)]);
}

#[test]
fn supports_all_projection_and_multiple_orderings() {
    let users = Table::new("users");
    let name = users.column::<String>("name");
    let age = users.column::<i32>("age");

    let query = select_all()
        .from(users)
        .order_by((name.asc(), age.desc()))
        .compile();

    assert_eq!(
        query.sql,
        "SELECT * FROM \"users\" ORDER BY \"users\".\"name\" ASC, \"users\".\"age\" DESC"
    );
    assert!(query.parameters.is_empty());
}

#[test]
fn supports_all_comparison_operators() {
    let users = Table::new("users");
    let age = users.column::<i32>("age");

    let queries = [
        (age.eq(1), " = $1"),
        (age.ne(1), " <> $1"),
        (age.gt(1), " > $1"),
        (age.gte(1), " >= $1"),
        (age.lt(1), " < $1"),
        (age.lte(1), " <= $1"),
    ];

    for (expression, operator) in queries {
        let query = select_all()
            .from(users.clone())
            .where_(expression)
            .compile();
        assert_eq!(
            query.sql,
            format!("SELECT * FROM \"users\" WHERE \"users\".\"age\"{operator}")
        );
        assert_eq!(query.parameters, vec![Value::Int(1)]);
    }
}

#[test]
fn supports_owned_text_and_float_values() {
    let products = Table::new("products");
    let name = products.column::<String>("name");
    let price = products.column::<f32>("price");

    let query = select_all()
        .from(products)
        .where_(name.eq(String::from("coffee")).and(price.gt(1.25_f32)))
        .compile();

    assert_eq!(
        query.parameters,
        vec![Value::Text(String::from("coffee")), Value::Float(1.25)]
    );
}

#[test]
fn logical_precedence_and_parameter_order_are_preserved() {
    let users = Table::new("users");
    let active = users.column::<bool>("active");
    let age = users.column::<i32>("age");
    let score = users.column::<f64>("score");

    let query = select_all()
        .from(users)
        .where_(active.eq(true).and(age.gt(18).or(score.lt(2.5))))
        .compile();

    assert_eq!(
        query.sql,
        "SELECT * FROM \"users\" WHERE \"users\".\"active\" = $1 AND (\"users\".\"age\" > $2 OR \"users\".\"score\" < $3)"
    );
    assert_eq!(
        query.parameters,
        vec![Value::Bool(true), Value::Int(18), Value::Float(2.5)]
    );
}

#[test]
fn reverse_logical_precedence_is_preserved() {
    let users = Table::new("users");
    let active = users.column::<bool>("active");
    let age = users.column::<i32>("age");
    let score = users.column::<f64>("score");

    let query = select_all()
        .from(users)
        .where_(active.eq(true).or(age.gt(18).and(score.lt(2.5))))
        .compile();

    assert_eq!(
        query.sql,
        "SELECT * FROM \"users\" WHERE \"users\".\"active\" = $1 OR \"users\".\"age\" > $2 AND \"users\".\"score\" < $3"
    );
    assert_eq!(
        query.parameters,
        vec![Value::Bool(true), Value::Int(18), Value::Float(2.5)]
    );
}

#[test]
fn schema_qualification_and_identifier_escaping_are_distinct_from_values() {
    let table = Table::with_schema("public", "some\"table");
    let column = Column::<String>::new(table.clone(), "some\"column");
    let query = select(&column)
        .from(table)
        .where_(column.eq("' OR 1 = 1 --"))
        .compile();

    assert_eq!(
        query.sql,
        "SELECT \"public\".\"some\"\"table\".\"some\"\"column\" FROM \"public\".\"some\"\"table\" WHERE \"public\".\"some\"\"table\".\"some\"\"column\" = $1"
    );
    assert_eq!(
        query.parameters,
        vec![Parameter::Text("' OR 1 = 1 --".to_owned())]
    );
    assert!(!query.sql.contains("OR 1 = 1"));
}

#[test]
fn independent_compilations_start_parameter_numbering_at_one() {
    let users = Table::new("users");
    let age = users.column::<i32>("age");
    let query = select_all().from(users).where_(age.eq(7));

    let first = query.compile();
    let second = query.compile();
    assert_eq!(first, second);
    assert!(first.sql.ends_with(" = $1"));
}
