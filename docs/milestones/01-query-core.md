# Milestone 01 — Query Core

## Status

Planned

## Target Version

`0.0.2`

## Goal

Implement the foundational query representation and PostgreSQL compilation pipeline for Elephant.

At the end of this milestone, Elephant must be capable of representing a typed `SELECT` query and compiling it into deterministic PostgreSQL SQL with parameter bindings.

No PostgreSQL server, connection, driver, pool, protocol implementation, or database I/O is required.

The milestone establishes:

```text
Schema Primitives
      ↓
Typed Expressions
      ↓
SELECT AST
      ↓
PostgreSQL Compiler
      ↓
CompiledQuery
```

This milestone is intentionally focused on the minimum foundation required by future query-building APIs.

---

# 1. Required Reading

Before implementation, follow `AGENTS.md`.

The authoritative documents are:

1. `docs/architecture.md`
2. `docs/development.md`
3. this milestone

Do not duplicate architectural decisions inside the implementation.

---

# 2. Scope

This milestone includes:

* table representation;
* schema-qualified tables;
* typed columns;
* identifiers;
* parameter values;
* basic typed expressions;
* logical expressions;
* SELECT AST;
* projection;
* FROM;
* WHERE;
* ORDER BY;
* LIMIT;
* OFFSET;
* PostgreSQL SQL compilation;
* deterministic parameter numbering;
* `CompiledQuery`;
* focused tests.

This milestone does not include database execution.

---

# 3. Target Capability

The implementation must support a query conceptually equivalent to:

```rust
select((users.id, users.name))
    .from(users)
    .where_(
        users.active
            .eq(true)
            .and(users.age.gte(18))
    )
    .order_by(users.created_at.desc())
    .limit(20)
```

and compile it to PostgreSQL equivalent to:

```sql
SELECT "users"."id", "users"."name"
FROM "users"
WHERE "users"."active" = $1
  AND "users"."age" >= $2
ORDER BY "users"."created_at" DESC
LIMIT 20
```

with bindings equivalent to:

```text
$1 = true
$2 = 18
```

The exact internal representation is intentionally not prescribed by this milestone.

It must respect the architectural boundaries defined by `docs/architecture.md`.

---

# 4. Table Representation

Implement a representation for PostgreSQL tables.

Required capabilities:

```rust
let users = Table::new("users");
```

and schema-qualified tables:

```rust
let users = Table::with_schema("public", "users");
```

Equivalent SQL should compile as:

```sql
"users"
```

and:

```sql
"public"."users"
```

respectively.

Tables represent identifiers.

They are not parameter values.

---

# 5. Identifiers

Introduce an explicit representation for PostgreSQL identifiers.

Identifiers include concepts such as:

```text
users
public
email
created_at
```

Identifier rendering must follow PostgreSQL identifier quoting rules.

For example:

```text
users
```

must render as:

```sql
"users"
```

Embedded quotes must be escaped correctly.

For example, an identifier conceptually containing:

```text
some"name
```

must render equivalently to:

```sql
"some""name"
```

Identifier handling must remain distinct from parameter handling.

---

# 6. Typed Columns

Implement typed columns.

Conceptually:

```rust
Column<T>
```

Example usage:

```rust
let id = Column::<Uuid>::new(users, "id");
let name = Column::<String>::new(users, "name");
let age = Column::<i32>::new(users, "age");
let active = Column::<bool>::new(users, "active");
```

The exact ownership or borrowing model may differ if a cleaner implementation is found.

A column must retain:

* table identity;
* column identity;
* Rust type information required for typed operations.

Columns must compile to qualified identifiers equivalent to:

```sql
"users"."id"
"users"."name"
```

---

# 7. Values

Introduce a representation for values that can become PostgreSQL parameters.

Initial support must cover at least:

* boolean values;
* signed integers required by tests;
* floating-point values required by tests;
* owned strings;
* borrowed string inputs where ergonomic;
* null representation where required by the internal model.

The representation must be designed so additional PostgreSQL types can be introduced later without coupling query construction to protocol implementation.

Do not implement PostgreSQL wire encoding in this milestone.

---

# 8. Parameters

User values used in expressions must become PostgreSQL parameters.

Example:

```rust
name.eq("John")
```

must compile equivalently to:

```sql
"users"."name" = $1
```

with `"John"` stored as a binding.

Do not generate:

```sql
"users"."name" = 'John'
```

through direct interpolation.

SQL-looking strings must remain data.

For example:

```rust
name.eq("' OR 1 = 1 --")
```

must still compile using a parameter.

---

# 9. Expression Representation

Implement an internal semantic representation for expressions.

The initial expression system must support:

```text
Column
Parameter

Eq
Ne
Gt
Gte
Lt
Lte

And
Or
```

The exact enum or type organization is an implementation decision.

Expressions must contain semantics, not rendered SQL.

Do not store fragments such as:

```text
"age >= $1"
```

inside AST nodes.

---

# 10. Typed Comparisons

Column comparisons with values must be type-safe where practical.

Required examples:

```rust
age.eq(18)
age.ne(18)
age.gt(18)
age.gte(18)
age.lt(18)
age.lte(18)

name.eq("John")

active.eq(true)
```

An incompatible comparison such as:

```rust
age.eq("eighteen")
```

must not be accepted by the typed API.

The type system should prevent this before SQL compilation.

Do not introduce excessive generic complexity beyond what is required to provide meaningful safety.

---

# 11. Logical Expressions

Expressions must support composition using:

```text
AND
OR
```

Example:

```rust
active
    .eq(true)
    .and(age.gte(18))
```

Nested expressions must preserve SQL semantics.

For example:

```text
A AND (B OR C)
```

must not accidentally compile as:

```text
(A AND B) OR C
```

The compiler must emit parentheses whenever necessary to preserve AST semantics.

---

# 12. SELECT AST

Implement the semantic representation required for SELECT queries.

The AST must support at least:

* projection;
* FROM;
* WHERE;
* ORDER BY;
* LIMIT;
* OFFSET.

Conceptually:

```text
Select
├── Projection
├── From
├── Where
├── OrderBy
├── Limit
└── Offset
```

The AST must not contain rendered SQL.

The AST must not perform I/O.

The AST must not depend on the PostgreSQL compiler.

---

# 13. Projection

Support explicit column projection.

Example:

```rust
select((users.id, users.name))
```

Equivalent SQL:

```sql
SELECT "users"."id", "users"."name"
```

The implementation must also have a representation capable of expressing SELECT-all semantics.

The exact public syntax for SELECT-all may remain minimal in this milestone because ergonomic public Query Builder design belongs primarily to Milestone 02.

---

# 14. FROM

Support a table source.

Example:

```rust
.from(users)
```

Equivalent SQL:

```sql
FROM "users"
```

Schema-qualified tables must preserve qualification.

---

# 15. WHERE

Support an optional WHERE expression.

Example:

```rust
.where_(users.active.eq(true))
```

Equivalent SQL:

```sql
WHERE "users"."active" = $1
```

WHERE accepts semantic expressions.

It must not accept arbitrary SQL strings as ordinary predicates.

Raw SQL support is outside this milestone.

---

# 16. ORDER BY

Support ascending and descending ordering.

Examples:

```rust
users.created_at.asc()
users.created_at.desc()
```

Equivalent SQL:

```sql
ORDER BY "users"."created_at" ASC
```

and:

```sql
ORDER BY "users"."created_at" DESC
```

The AST must be capable of representing multiple ordering expressions even if the initial convenience API remains minimal.

---

# 17. LIMIT

Support PostgreSQL LIMIT.

Example:

```rust
.limit(20)
```

Equivalent SQL:

```sql
LIMIT 20
```

The representation must make invalid negative limits impossible.

Do not use signed values if doing so permits meaningless negative limits.

LIMIT does not need to be represented as a PostgreSQL binding in this milestone.

---

# 18. OFFSET

Support PostgreSQL OFFSET.

Example:

```rust
.offset(40)
```

Equivalent SQL:

```sql
OFFSET 40
```

The representation must make invalid negative offsets impossible.

OFFSET does not need to be represented as a PostgreSQL binding in this milestone.

---

# 19. PostgreSQL Compiler

Implement a dedicated PostgreSQL compiler.

The compiler consumes AST.

The compiler produces:

```text
CompiledQuery
```

The compiler owns:

* PostgreSQL syntax;
* identifier rendering;
* expression rendering;
* operator rendering;
* parameter numbering;
* projection rendering;
* FROM rendering;
* WHERE rendering;
* ORDER BY rendering;
* LIMIT rendering;
* OFFSET rendering.

AST nodes must not render themselves into PostgreSQL SQL.

---

# 20. CompiledQuery

Compilation must produce a structure conceptually equivalent to:

```rust
pub struct CompiledQuery {
    pub sql: String,
    pub parameters: Vec<Parameter>,
}
```

The exact parameter type is an implementation decision.

`CompiledQuery` represents the boundary between query construction and future execution.

It contains:

* generated PostgreSQL SQL;
* ordered parameter values.

It must not require a PostgreSQL connection.

---

# 21. Parameter Numbering

PostgreSQL parameters must be numbered deterministically.

For:

```rust
active
    .eq(true)
    .and(age.gte(18))
```

the compiler should generate equivalent SQL:

```sql
"users"."active" = $1 AND "users"."age" >= $2
```

with bindings ordered:

```text
true
18
```

Nested expressions must preserve traversal order consistently.

Compilation of the same AST must always produce the same parameter ordering.

---

# 22. Compiler State

Parameter numbering may require temporary compiler state.

That state must belong to the compilation operation.

Do not use:

* mutable global counters;
* static mutable state;
* process-wide parameter numbering;
* thread-local hidden counters.

Independent compilations must not affect one another.

---

# 23. Public Surface

The public API in this milestone may remain intentionally small.

The purpose of Milestone 01 is to establish correct foundations.

Do not prematurely optimize the fluent API.

Milestone 02 owns the ergonomic Query Builder API.

Only expose types publicly when required by the milestone or clearly necessary for the future public boundary.

Prefer internal visibility otherwise.

---

# 24. Suggested Internal Organization

A possible organization is:

```text
src/
├── lib.rs
├── ast/
│   ├── mod.rs
│   ├── expression.rs
│   └── select.rs
├── schema/
│   ├── mod.rs
│   ├── table.rs
│   ├── column.rs
│   └── identifier.rs
├── postgres/
│   ├── mod.rs
│   ├── compiler.rs
│   └── parameter.rs
├── value.rs
└── error.rs
```

This is not a requirement to create every listed file.

Follow `docs/development.md`.

Create files based on actual responsibilities discovered during implementation.

Do not create empty placeholders.

---

# 25. Error Handling

Compilation and construction errors must use explicit error types where runtime failure is possible.

Do not use:

```rust
unwrap()
expect()
panic!()
```

for recoverable errors.

If a state can be prevented through the type system, prefer preventing it instead of introducing a runtime error.

Do not create a large generic error hierarchy without concrete errors to represent.

---

# 26. Testing Requirements

Tests must cover the behavior introduced by this milestone.

At minimum, test:

* simple SELECT;
* explicit projection;
* SELECT-all representation;
* FROM;
* WHERE;
* equality;
* inequality;
* greater-than;
* greater-than-or-equal;
* less-than;
* less-than-or-equal;
* AND;
* OR;
* nested AND/OR;
* expression precedence;
* parameter numbering;
* multiple bindings;
* ORDER BY ASC;
* ORDER BY DESC;
* multiple order representation;
* LIMIT;
* OFFSET;
* identifier quoting;
* embedded identifier quote escaping;
* schema-qualified tables;
* qualified columns;
* deterministic compilation;
* independent compilation parameter numbering;
* SQL-looking strings remaining bindings.

Tests should verify both:

```text
SQL
bindings
```

when parameters are involved.

---

# 27. Compile-Time Tests

Add compile-time verification where practical for type guarantees.

At minimum, the test strategy must verify that compatible comparisons are accepted.

The project should also verify that incompatible comparisons such as:

```rust
Column::<i32>::eq("eighteen")
```

are rejected by the compiler.

The exact compile-test mechanism may be chosen during implementation.

Do not add a dependency solely for compile tests without evaluating whether it is justified under `docs/development.md`.

---

# 28. Security Expectations

User values must never be directly interpolated into generated SQL.

Identifiers must use identifier quoting rather than parameter binding.

Tests must include a malicious-looking value equivalent to:

```text
' OR 1 = 1 --
```

and verify that the value remains a binding.

This milestone does not claim to provide a complete security audit.

It establishes the required separation between SQL structure and data values.

---

# 29. Determinism

Compilation must be deterministic.

Compiling the same query multiple times must produce equivalent:

* SQL;
* parameter numbering;
* parameter ordering.

Tests must cover this behavior.

---

# 30. Out of Scope

Do not implement:

* PostgreSQL connections;
* TCP;
* TLS;
* PostgreSQL wire protocol;
* connection pooling;
* async query execution;
* row decoding;
* transactions;
* INSERT;
* UPDATE;
* DELETE;
* RETURNING;
* ON CONFLICT;
* JOIN;
* GROUP BY;
* HAVING;
* aggregates;
* subqueries;
* CTEs;
* DISTINCT ON;
* migrations;
* schema introspection;
* procedural schema macros;
* relation loading;
* ORM behavior;
* Active Record;
* CLI;
* Elephant Console.

Do not implement these features merely because the current architecture could support them.

---

# 31. Acceptance Criteria

The milestone is complete only when all applicable criteria are verified.

## Schema

* [ ] Tables can be represented.
* [ ] Schema-qualified tables can be represented.
* [ ] Typed columns can be represented.
* [ ] Columns preserve meaningful Rust type information.
* [ ] Identifiers are distinct from parameter values.
* [ ] PostgreSQL identifier quoting is implemented.
* [ ] Embedded identifier quotes are escaped correctly.

## Expressions

* [ ] Equality is supported.
* [ ] Inequality is supported.
* [ ] Greater-than is supported.
* [ ] Greater-than-or-equal is supported.
* [ ] Less-than is supported.
* [ ] Less-than-or-equal is supported.
* [ ] AND is supported.
* [ ] OR is supported.
* [ ] Nested expressions preserve semantics.
* [ ] Compatible comparisons are type-safe.
* [ ] Incompatible comparisons are rejected by the typed API.

## SELECT

* [ ] Explicit projections are supported.
* [ ] SELECT-all can be represented.
* [ ] FROM is supported.
* [ ] WHERE is supported.
* [ ] ORDER BY ASC is supported.
* [ ] ORDER BY DESC is supported.
* [ ] Multiple ordering expressions can be represented.
* [ ] LIMIT is supported.
* [ ] OFFSET is supported.
* [ ] Negative LIMIT values cannot be represented.
* [ ] Negative OFFSET values cannot be represented.

## Compiler

* [ ] SELECT AST compiles to PostgreSQL SQL.
* [ ] User values become bindings.
* [ ] Parameters use PostgreSQL `$N` syntax.
* [ ] Parameter numbering is deterministic.
* [ ] Nested expressions preserve binding order.
* [ ] Schema-qualified identifiers compile correctly.
* [ ] Compilation requires no database connection.
* [ ] Independent compilations do not share parameter state.
* [ ] `CompiledQuery` contains SQL and ordered parameters.

## Architecture

* [ ] Query construction performs no I/O.
* [ ] AST contains no rendered SQL.
* [ ] AST does not depend on Query Builder.
* [ ] AST does not depend on PostgreSQL Compiler.
* [ ] Compiler does not perform execution.
* [ ] No future execution infrastructure was introduced.
* [ ] Public API exposure is intentional and minimal.

## Tests

* [ ] Required unit tests exist.
* [ ] Parameter ordering is tested.
* [ ] Expression precedence is tested.
* [ ] Identifier escaping is tested.
* [ ] SQL-looking values remain parameters.
* [ ] Deterministic compilation is tested.
* [ ] Type compatibility is tested.
* [ ] Type incompatibility is compile-tested where practical.

## Quality

* [ ] `cargo fmt --check` passes.
* [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes.
* [ ] `cargo test` passes.

---

# 32. Definition of Done

The milestone is complete when Elephant can construct a typed query equivalent to:

```rust
select((users.id, users.name))
    .from(users)
    .where_(
        users.active
            .eq(true)
            .and(users.age.gte(18))
    )
    .order_by(users.created_at.desc())
    .limit(20)
```

and compile it without PostgreSQL running into SQL equivalent to:

```sql
SELECT "users"."id", "users"."name"
FROM "users"
WHERE "users"."active" = $1
  AND "users"."age" >= $2
ORDER BY "users"."created_at" DESC
LIMIT 20
```

with ordered bindings equivalent to:

```text
true
18
```

The implementation must satisfy:

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

No database connection may be required.

Do not begin Milestone 02 automatically.
