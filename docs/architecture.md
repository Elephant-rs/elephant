# Elephant Architecture

Elephant is a PostgreSQL-first, type-safe query builder for Rust.

Its primary goal is to provide an idiomatic Rust API for building and executing PostgreSQL queries without hiding SQL or PostgreSQL behind ORM abstractions.

## Core Principles

Elephant follows these architectural principles:

* PostgreSQL-first
* Query Builder first
* Type-safe whenever possible
* SQL-oriented API
* Explicit behavior over implicit behavior
* No Active Record
* No database dialect abstraction
* No hidden queries
* Async-first for database operations
* Minimal runtime overhead
* Compile-time validation whenever practical
* PostgreSQL features should be exposed rather than abstracted away

A developer familiar with PostgreSQL should be able to look at Elephant code and reasonably predict the generated SQL.

## High-Level Architecture

The main query pipeline is:

```text
Rust Query API
      │
      ▼
Typed Query Builder
      │
      ▼
AST
      │
      ▼
PostgreSQL Compiler
      │
      ▼
CompiledQuery
 ┌────┴─────┐
 ▼          ▼
SQL      Bindings
 │          │
 └────┬─────┘
      ▼
Executor
      │
      ▼
Connection / Pool
      │
      ▼
PostgreSQL
```

The reverse path is:

```text
PostgreSQL
    │
    ▼
Protocol
    │
    ▼
Rows
    │
    ▼
Decoder
    │
    ▼
Typed Rust Values
```

Each layer must have a clear responsibility.

## Query Builder

The Query Builder is the primary public interface of Elephant.

Example:

```rust
db.select((User::id, User::name))
    .from(User::table)
    .where_(
        User::active
            .eq(true)
            .and(User::age.gte(18))
    )
    .order_by(User::created_at.desc())
    .limit(20)
```

The Query Builder is responsible for:

* providing an ergonomic Rust API
* enforcing type constraints where possible
* constructing the AST
* preventing invalid combinations where practical

It must not:

* execute queries
* manage connections
* generate SQL directly
* contain PostgreSQL protocol logic

## Schema

The schema layer represents PostgreSQL structures in Rust.

Conceptually:

```rust
Table
Column<T>
PrimaryKey<T>
ForeignKey<T>
```

Example:

```rust
User::table
User::id
User::email
User::created_at
```

Columns carry their Rust/PostgreSQL type information.

Example:

```text
users.id          → Column<Uuid>
users.name        → Column<Text>
users.active      → Column<Bool>
users.created_at  → Column<Timestamp>
```

The schema layer enables compile-time validation of expressions.

For example:

```rust
User::age.eq(30)
```

should compile.

An incompatible comparison should fail at compile time whenever reasonably possible.

## Expressions

Expressions represent SQL expressions independently of SQL rendering.

Examples:

```rust
User::id.eq(id)

User::age.gte(18)

User::name.ilike("%john%")

User::deleted_at.is_null()
```

Expressions must be composable:

```rust
User::active
    .eq(true)
    .and(User::age.gte(18))
```

Conceptually:

```text
AND
├── EQ
│   ├── users.active
│   └── Parameter(true)
└── GTE
    ├── users.age
    └── Parameter(18)
```

Expressions must not contain rendered SQL.

They describe SQL semantics.

## AST

The AST is the internal representation of a query.

Example:

```text
Select
├── Projection
│   ├── users.id
│   └── users.name
├── From
│   └── users
├── Where
│   └── And
│       ├── Eq
│       │   ├── users.active
│       │   └── Parameter(true)
│       └── Gte
│           ├── users.age
│           └── Parameter(18)
├── OrderBy
│   └── users.created_at DESC
└── Limit
    └── 20
```

The AST must not know:

* how PostgreSQL connections work
* how queries are executed
* how connection pools work
* how PostgreSQL wire messages are encoded

It only represents the query.

## PostgreSQL Compiler

The PostgreSQL Compiler converts an AST into PostgreSQL SQL.

```text
AST
 │
 ▼
PostgresCompiler
 │
 ▼
CompiledQuery
```

Example output:

```sql
SELECT "users"."id", "users"."name"
FROM "users"
WHERE "users"."active" = $1
  AND "users"."age" >= $2
ORDER BY "users"."created_at" DESC
LIMIT 20
```

With bindings:

```text
$1 = true
$2 = 18
```

The compiler is responsible for:

* PostgreSQL syntax
* identifier quoting
* aliases
* parameter numbering
* PostgreSQL-specific operators
* generating valid SQL

The compiler must never interpolate user values directly into SQL.

## CompiledQuery

Compilation produces a structure conceptually equivalent to:

```rust
pub struct CompiledQuery {
    pub sql: String,
    pub parameters: Vec<Parameter>,
}
```

`CompiledQuery` is the boundary between query construction and query execution.

Everything before this structure is concerned with describing a query.

Everything after it is concerned with communicating with PostgreSQL.

This boundary must remain explicit.

## Parameters

Values and identifiers are fundamentally different concepts.

Elephant must internally distinguish:

```text
Identifier
Parameter
RawSql
```

Given:

```rust
User::email.eq("john@example.com")
```

Elephant generates:

```sql
"users"."email" = $1
```

and stores:

```text
$1 = "john@example.com"
```

It must never generate:

```sql
"users"."email" = 'john@example.com'
```

through direct string interpolation.

## Execution

The execution layer receives a `CompiledQuery`.

Conceptually:

```rust
executor.execute(query).await
```

Its responsibilities are:

* acquire a connection
* send the query
* send parameters
* receive PostgreSQL responses
* propagate database errors
* return rows/results

The executor must not reconstruct or modify the query AST.

## PostgreSQL Client

Long term, Elephant should provide its own PostgreSQL client layer.

Conceptually:

```text
Executor
   │
   ▼
Client
   │
   ▼
PostgreSQL Protocol
   │
   ▼
TCP/TLS
```

The client is responsible for:

* connection startup
* authentication
* prepared statements
* query execution
* transactions
* PostgreSQL messages
* error responses
* row descriptions
* data rows

Elephant may use foundational libraries for:

* async runtime
* networking
* TLS
* cryptography

It should not reimplement these components.

## Connection Pool

Pooling must remain separate from query construction.

Conceptually:

```text
Database
   │
   ▼
Pool
   │
   ├── Connection
   ├── Connection
   ├── Connection
   └── Connection
```

The pool will eventually manage:

* minimum connections
* maximum connections
* acquisition
* release
* acquisition timeout
* idle timeout
* connection lifetime
* health checks
* broken connections
* graceful shutdown

Queries must not know whether they are executed through a pooled or dedicated connection.

## Row Decoding

PostgreSQL responses must be decoded independently of query construction.

Conceptually:

```text
DataRow
   │
   ▼
PostgreSQL Types
   │
   ▼
Decoder
   │
   ▼
Rust Values
```

Eventually Elephant should support typed projections.

Example:

```rust
db.select((
    User::id,
    User::name,
))
.from(User::table)
.all()
.await?
```

should infer a result corresponding to:

```text
(Uuid, String)
```

where practical.

## Table Mapping

Elephant may provide procedural macros for schema declaration.

Example target API:

```rust
#[derive(Table)]
#[table(name = "users")]
struct User {
    #[primary_key]
    id: Uuid,

    name: String,
    email: String,
    active: bool,
}
```

Macros should generate metadata and repetitive implementations.

They must not contain the core query-building logic.

Core behavior should remain implemented using ordinary Rust types, traits and functions.

## PostgreSQL-specific Features

Elephant intentionally embraces PostgreSQL.

Features such as the following belong naturally in the architecture:

```text
RETURNING
ON CONFLICT
DISTINCT ON
ILIKE
ANY
ALL
ARRAY
JSONB
CTE
WITH RECURSIVE
LATERAL
window functions
FILTER
FOR UPDATE
FOR SHARE
SKIP LOCKED
full-text search
advisory locks
COPY
LISTEN
NOTIFY
```

The architecture must not restrict these capabilities in order to maintain compatibility with another database.

## Raw SQL

Raw SQL is a required escape hatch.

Example target API:

```rust
db.raw("SELECT * FROM users WHERE id = $1")
    .bind(id)
    .all()
    .await?
```

Raw SQL must still encourage parameter binding.

Raw SQL should be explicit in the API so that it cannot be confused with typed query-builder expressions.

## Transactions

Transactions belong to the execution layer.

Target API:

```rust
let mut tx = db.transaction().await?;

tx.insert(...)
    .execute()
    .await?;

tx.update(...)
    .execute()
    .await?;

tx.commit().await?;
```

Query builders should work with both a database executor and a transaction executor without duplicating query-building logic.

## Error Boundaries

Errors should reflect architectural layers.

Conceptually:

```text
BuildError
CompileError

ConnectionError
ProtocolError
DatabaseError
DecodeError
PoolError
```

PostgreSQL database errors should preserve useful server information such as:

```text
SQLSTATE
message
detail
hint
table
column
constraint
```

when available.

## Module Boundaries

Initially Elephant should remain a single crate.

Suggested internal organization:

```text
src/
├── lib.rs
│
├── query/
│   ├── mod.rs
│   ├── select.rs
│   ├── insert.rs
│   ├── update.rs
│   └── delete.rs
│
├── ast/
│   ├── mod.rs
│   ├── expression.rs
│   └── query.rs
│
├── schema/
│   ├── mod.rs
│   ├── table.rs
│   └── column.rs
│
├── postgres/
│   ├── mod.rs
│   ├── compiler.rs
│   └── types.rs
│
└── error.rs
```

This is a direction, not a requirement to create empty files prematurely.

Modules should only be introduced when implementation requires them.

## Future Crate Boundaries

If the project becomes large enough, components may eventually become independent crates:

```text
elephant
│
├── elephant-query
├── elephant-macros
├── elephant-postgres
├── elephant-protocol
├── elephant-pool
└── elephant-cli
```

This split must happen only when there is a concrete architectural reason.

The initial implementation should favor simplicity.

## Dependency Direction

Dependencies should flow downward:

```text
Public Query API
       │
       ▼
      AST
       │
       ▼
PostgreSQL Compiler

Execution API
       │
       ▼
     Client
       │
       ▼
 PostgreSQL Protocol
       │
       ▼
    TCP / TLS
```

Higher-level components may depend on lower-level components.

Lower-level components must not depend on higher-level APIs.

For example:

```text
AST → Query Builder
```

is forbidden.

The correct direction is:

```text
Query Builder → AST
```

Likewise:

```text
PostgreSQL Protocol → Query Builder
```

is forbidden.

## Architectural Invariants

The following rules should be treated as invariants.

### 1. Query construction does not perform I/O

Building this:

```rust
let query = db
    .select(...)
    .from(...)
    .where_(...);
```

must not communicate with PostgreSQL.

### 2. AST does not contain rendered SQL

The AST represents semantics.

The compiler produces SQL.

### 3. User values are bindings

User-provided values should become PostgreSQL parameters whenever possible.

### 4. PostgreSQL is not abstracted away

Do not introduce abstractions whose only purpose is compatibility with other databases.

### 5. No implicit queries

A method that appears to manipulate an in-memory object must not unexpectedly execute SQL.

### 6. No Active Record

Database operations originate from the query/execution API, not persistent model instances.

### 7. Query Builder remains independently usable

The query-building and compilation layers must not require a live database.

This must always be possible:

```rust
let compiled = query.to_sql();
```

without PostgreSQL running.

### 8. Type safety must have semantic value

Do not introduce complex generic machinery unless it prevents real classes of invalid queries or substantially improves result inference.

### 9. Escape hatches remain available

Elephant should make common operations safe and ergonomic without preventing advanced PostgreSQL usage.

### 10. Public API stability matters

Internal representations may evolve.

Public APIs should remain intentionally small and carefully designed.

## Architectural Decision Rule

When choosing between an ORM-like abstraction and PostgreSQL semantics:

> Prefer PostgreSQL semantics.

When choosing between implicit and explicit behavior:

> Prefer explicit behavior.

When choosing between multi-database compatibility and better PostgreSQL support:

> Prefer PostgreSQL.

When choosing between clever abstractions and understandable Rust:

> Prefer understandable Rust.

When choosing between convenience and compile-time safety where both cannot reasonably coexist:

> Prefer compile-time safety unless the ergonomics become impractical.

## Product Boundary

Elephant is:

> A PostgreSQL-first, type-safe query builder for Rust.

Elephant is not:

> An ORM that happens to support PostgreSQL.

The architecture must preserve this distinction as the project evolves.
