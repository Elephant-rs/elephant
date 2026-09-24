# Elephant Architecture

Elephant is a PostgreSQL-first, type-safe query builder for Rust.

Its primary goal is to provide an idiomatic Rust API for building and executing PostgreSQL queries without hiding SQL or PostgreSQL behind ORM abstractions.

This document defines the architectural boundaries of Elephant.

---

# 1. Documentation Responsibilities

Elephant's documentation is divided into three levels.

## Architecture

`docs/architecture.md`

Defines:

* architectural boundaries
* component responsibilities
* dependency direction
* data flow
* architectural invariants
* product boundaries

This document answers:

> How is Elephant architected?

## Development Guidelines

`docs/development.md`

Defines:

* development practices
* SOLID principles
* code organization
* file responsibilities
* testing standards
* naming
* error handling
* dependency rules
* code quality standards

This document answers:

> How should Elephant code be written?

## Milestones

`docs/milestones/`

Defines:

* current feature scope
* deliverables
* acceptance criteria
* Definition of Done
* implementation sequence

These documents answer:

> What should be implemented now?

The precedence order is:

```text
Architecture
     ↓
Development Guidelines
     ↓
Active Milestone
```

A milestone must not violate architectural constraints or development standards in order to satisfy its acceptance criteria.

If a milestone reveals that an architectural rule must change, the architecture must be explicitly reviewed and updated rather than silently bypassed.

---

# 2. Product Definition

Elephant is:

> A PostgreSQL-first, type-safe query builder for Rust.

Elephant provides a type-safe, SQL-oriented Rust API for constructing and executing PostgreSQL queries while embracing PostgreSQL rather than abstracting it away.

Elephant is not:

> An ORM that happens to support PostgreSQL.

Elephant is also not intended to become a generic database abstraction.

PostgreSQL is part of the product identity.

---

# 3. Core Architectural Principles

Elephant follows these principles:

* PostgreSQL-first
* Query Builder first
* type-safe whenever practical
* SQL-oriented API
* explicit behavior
* predictable SQL generation
* no Active Record
* no database dialect abstraction
* no hidden queries
* no implicit database access
* async-first for I/O
* query construction independent from execution
* minimal runtime overhead
* PostgreSQL capabilities exposed rather than hidden

A developer familiar with PostgreSQL should be able to inspect Elephant code and reasonably predict the SQL that will be generated.

Example:

```rust
db.select((
    User::id,
    User::name,
))
.from(User::table)
.where_(
    User::active
        .eq(true)
        .and(User::age.gte(18))
)
.order_by(User::created_at.desc())
.limit(20)
```

should correspond naturally to PostgreSQL equivalent to:

```sql
SELECT "users"."id", "users"."name"
FROM "users"
WHERE "users"."active" = $1
  AND "users"."age" >= $2
ORDER BY "users"."created_at" DESC
LIMIT 20
```

with bindings:

```text
$1 = true
$2 = 18
```

---

# 4. Architectural Layers

Elephant is organized conceptually into the following layers:

```text
Schema
   │
   ▼
Query Builder
   │
   ▼
AST
   │
   ▼
PostgreSQL Compiler
   │
   ▼
CompiledQuery
   │
   ▼
Executor
   │
   ▼
Client
   │
   ▼
Protocol
   │
   ▼
Transport
   │
   ▼
PostgreSQL
```

The response path follows the opposite direction:

```text
PostgreSQL
    │
    ▼
Transport
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

Each layer owns a specific responsibility.

| Layer               | Responsibility                                            |
| ------------------- | --------------------------------------------------------- |
| Schema              | Tables, columns, constraints and PostgreSQL type metadata |
| Query Builder       | Public typed API for constructing queries                 |
| AST                 | Internal semantic representation of SQL queries           |
| PostgreSQL Compiler | Converts AST into PostgreSQL SQL and bindings             |
| CompiledQuery       | Boundary between query construction and execution         |
| Executor            | Executes compiled queries                                 |
| Client              | Manages PostgreSQL sessions and query communication       |
| Protocol            | Encodes and decodes PostgreSQL wire protocol messages     |
| Transport           | TCP/TLS communication                                     |
| Decoder             | Converts PostgreSQL values into Rust values               |

These boundaries should remain visible in the implementation.

---

# 5. Dependency Direction

Dependencies flow toward lower-level abstractions.

The primary query path is:

```text
Schema
   ↓
Query Builder
   ↓
AST
   ↓
Compiler
```

The execution path is:

```text
Executor
   ↓
Client
   ↓
Protocol
   ↓
Transport
```

The compiler connects query construction with execution through:

```text
AST
 ↓
Compiler
 ↓
CompiledQuery
 ↓
Executor
```

Dependencies must not arbitrarily flow in both directions.

---

# 6. Forbidden Dependencies

The following dependencies are architecturally forbidden unless this document is explicitly revised.

```text
Schema
    ✗ must not depend on Query Builder

AST
    ✗ must not depend on Query Builder

AST
    ✗ must not depend on Compiler

Compiler
    ✗ must not depend on Executor

Query Builder
    ✗ must not depend on Executor

Query Builder
    ✗ must not depend on Client

Query Builder
    ✗ must not depend on Protocol

Protocol
    ✗ must not depend on Query Builder

Protocol
    ✗ must not depend on AST

Transport
    ✗ must not depend on Query Builder

Transport
    ✗ must not depend on AST
```

For example:

```text
Query Builder → AST
```

is valid.

But:

```text
AST → Query Builder
```

is not.

Likewise:

```text
Compiler → AST
```

is valid.

But:

```text
AST → Compiler
```

is not.

These rules exist to preserve independent reasoning and testing of each layer.

---

# 7. Schema Layer

The Schema layer represents PostgreSQL database structures in Rust.

Its responsibilities include:

* tables
* columns
* PostgreSQL type metadata
* primary keys
* foreign keys
* nullability
* constraints
* generated values
* defaults
* relation metadata

Conceptually:

```text
Table
Column<T>
PrimaryKey<T>
ForeignKey<T>
```

Example:

```rust
User::table
User::id
User::name
User::email
User::active
User::created_at
```

Columns carry compile-time type information.

Conceptually:

```text
users.id          → Column<Uuid>
users.name        → Column<String>
users.active      → Column<bool>
users.created_at  → Column<Timestamp>
```

The Schema layer provides metadata to the Query Builder.

It does not construct complete queries.

It does not compile SQL.

It does not perform I/O.

---

# 8. Type Safety Boundary

Type safety is primarily enforced at Elephant's public API boundary.

The Query Builder and Schema layers should use Rust's type system to prevent invalid operations whenever doing so provides meaningful safety.

For example:

```rust
User::age.eq(18)
```

should compile.

While:

```rust
User::age.eq("eighteen")
```

should fail at compile time.

Likewise, operations should only exist where semantically appropriate.

For example:

```rust
User::name.ilike("%john%")
```

is valid for a compatible textual column.

An equivalent operation on an incompatible numeric column should not be accepted by the typed API.

However, compile-time information should not automatically propagate through every internal layer if doing so creates excessive generic complexity.

The architectural rule is:

> Type safety belongs primarily at the API boundary; the internal AST should prioritize a stable, understandable semantic representation.

The Query Builder may use generics extensively to validate operations and then lower those operations into a simpler internal AST.

Avoid architectures where every AST node becomes deeply parameterized solely to preserve information that has already been validated.

For example, avoid unnecessary internal structures resembling:

```text
SelectQuery<
    Projection<
        Tuple3<
            Column<User, Uuid>,
            Column<User, String>,
            Aggregate<
                Count<
                    Column<Post, Uuid>
                >
            >
        >
    >,
    From<User>,
    ...
>
```

unless such complexity provides a concrete and necessary capability.

Type safety is a product feature.

Generic complexity is not.

---

# 9. Query Builder

The Query Builder is Elephant's primary public interface.

Example:

```rust
db.select((
    User::id,
    User::name,
))
.from(User::table)
.where_(
    User::active
        .eq(true)
        .and(User::age.gte(18))
)
.order_by(User::created_at.desc())
.limit(20)
```

Its responsibilities are:

* provide an ergonomic Rust API
* enforce type constraints
* validate query construction where practical
* transform typed operations into AST nodes
* preserve SQL-oriented semantics

The Query Builder must not:

* perform database I/O
* manage connections
* manage connection pools
* encode PostgreSQL protocol messages
* decode PostgreSQL rows
* directly generate SQL strings

The Query Builder constructs semantic query representations.

It does not execute them.

---

# 10. Query Construction Performs No I/O

Query construction must always remain independent from PostgreSQL.

This:

```rust
let query = db
    .select(...)
    .from(...)
    .where_(...);
```

must not:

* acquire a connection
* open a socket
* communicate with PostgreSQL
* execute SQL
* inspect database state

This architectural property allows queries to be:

* tested without PostgreSQL
* inspected
* compiled independently
* used by tooling
* used by the future Elephant Console

---

# 11. Expression System

Expressions represent SQL semantics.

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

Expressions describe semantics.

They do not contain pre-rendered SQL.

---

# 12. AST

The AST is Elephant's internal semantic representation of a query.

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

The AST represents what the query means.

It does not represent how PostgreSQL communicates over the network.

The AST must not know:

* connection state
* connection pools
* sockets
* TLS
* PostgreSQL wire messages
* row decoding
* query execution

---

# 13. AST Is Not the Public API

The AST is an internal representation.

Users should construct queries through Elephant's Query Builder.

The normal direction is:

```text
Public Query API
       ↓
Typed validation
       ↓
AST
```

Users should not need to manually construct internal AST nodes.

This separation allows Elephant to evolve its internal query representation without unnecessarily breaking its public API.

A low-level AST API may exist in the future if there is a concrete use case, but it must be deliberate rather than accidental exposure of internals.

---

# 14. AST Does Not Render SQL

AST nodes must not contain PostgreSQL rendering logic.

Avoid designs conceptually equivalent to:

```rust
expression.to_postgres_sql()
```

when the expression itself is responsible for rendering PostgreSQL.

Instead:

```text
AST
 ↓
PostgreSQL Compiler
 ↓
SQL
```

This preserves separation between semantic representation and SQL rendering.

---

# 15. PostgreSQL Compiler

The PostgreSQL Compiler transforms AST into PostgreSQL SQL.

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

Bindings:

```text
$1 = true
$2 = 18
```

The compiler owns PostgreSQL-specific rendering behavior.

Its responsibilities include:

* PostgreSQL syntax
* identifier quoting
* parameter numbering
* aliases
* operators
* expressions
* PostgreSQL-specific query syntax

The compiler must be deterministic.

Given the same AST, it must produce equivalent SQL and binding order.

---

# 16. CompiledQuery

Compilation produces a structure conceptually equivalent to:

```rust
pub struct CompiledQuery {
    pub sql: String,
    pub parameters: Vec<Parameter>,
}
```

`CompiledQuery` is an important architectural boundary.

Before this boundary:

```text
Schema
Query Builder
AST
Compiler
```

are concerned with query description and compilation.

After this boundary:

```text
Executor
Client
Protocol
Transport
```

are concerned with execution and communication.

The execution layer should not need to understand the original Query Builder.

---

# 17. Identifiers, Parameters and Raw SQL

Elephant must distinguish between:

```text
Identifier
Parameter
RawSql
```

These concepts are fundamentally different.

## Identifier

Represents database identifiers such as:

```text
users
email
public
user_id
```

Identifiers require PostgreSQL identifier quoting rules.

## Parameter

Represents data values.

Given:

```rust
User::email.eq("john@example.com")
```

Elephant should generate:

```sql
"users"."email" = $1
```

with:

```text
$1 = "john@example.com"
```

## RawSql

Represents explicitly untyped/raw SQL supplied through an escape hatch.

Raw SQL must never be confused internally with identifiers or values.

---

# 18. Parameter Safety

User values should become PostgreSQL parameters whenever possible.

Elephant should generate:

```sql
WHERE "email" = $1
```

rather than interpolating:

```sql
WHERE "email" = 'john@example.com'
```

Parameter numbering must remain deterministic.

Nested expressions and subqueries must preserve correct binding order.

---

# 19. PostgreSQL-Specific Architecture

Elephant intentionally embraces PostgreSQL.

The architecture must support PostgreSQL-specific features naturally.

Examples include:

```text
RETURNING
ON CONFLICT
DISTINCT ON
ILIKE
ANY
ALL
ARRAY
JSON
JSONB
CTE
WITH RECURSIVE
LATERAL
window functions
FILTER
FOR UPDATE
FOR SHARE
SKIP LOCKED
advisory locks
full-text search
COPY
LISTEN
NOTIFY
```

PostgreSQL-specific features are not architectural leaks.

They are part of Elephant's purpose.

Do not weaken the architecture to accommodate hypothetical support for other databases.

---

# 20. PostgreSQL Extensions

The architecture should allow optional PostgreSQL extensions to integrate without contaminating unrelated core components.

Examples include:

```text
PostGIS
pgvector
```

Extension-specific behavior should remain isolated where practical.

The core should provide appropriate extension points only when concrete requirements emerge.

Do not design a generic plugin system prematurely.

---

# 21. Raw SQL Escape Hatch

Elephant must never prevent developers from using PostgreSQL directly.

A future API may resemble:

```rust
db.raw("SELECT * FROM users WHERE id = $1")
    .bind(id)
    .all()
    .await?
```

Raw SQL should remain explicit.

It should continue to support parameter binding.

The existence of raw SQL is intentional:

> Elephant should make common PostgreSQL operations safer and more ergonomic without restricting advanced PostgreSQL usage.

---

# 22. Executor

The Executor receives a `CompiledQuery`.

Conceptually:

```text
CompiledQuery
     │
     ▼
  Executor
```

Its responsibilities include:

* obtaining an appropriate connection
* sending the compiled query
* sending parameter values
* receiving results
* propagating database errors
* coordinating row decoding

The Executor must not reconstruct the original AST.

It should not need to know how the Query Builder produced the query.

---

# 23. Executor Capability

Eventually different execution contexts may need to execute the same compiled queries.

For example:

```text
Database
Transaction
Dedicated Connection
```

The architecture should allow shared execution behavior without duplicating query-building logic.

Do not introduce an execution trait until concrete execution implementations require such abstraction.

---

# 24. PostgreSQL Client

The PostgreSQL Client manages a PostgreSQL session.

Conceptually:

```text
Executor
   │
   ▼
Client
   │
   ▼
Protocol
   │
   ▼
Transport
```

Responsibilities may include:

* startup
* authentication
* session state
* prepared statements
* statement lifecycle
* portals
* query execution
* transaction commands
* PostgreSQL responses

The client should not know how high-level queries were constructed.

It receives commands or compiled representations appropriate to its layer.

---

# 25. PostgreSQL Protocol

The Protocol layer is responsible for PostgreSQL wire protocol semantics.

Its responsibilities include:

* frontend message encoding
* backend message decoding
* message types
* protocol framing
* authentication messages
* row descriptions
* data rows
* error responses
* ready-for-query state

The Protocol layer must not depend on:

* Query Builder
* Schema
* AST
* application models

It should be testable independently.

---

# 26. Transport

Transport is the lowest communication layer.

Its responsibilities include:

```text
TCP
TLS
byte transport
connection lifecycle at transport level
```

Transport should not understand SQL.

Transport should not understand Query Builder concepts.

Transport should not understand AST nodes.

Protocol semantics belong above the transport layer.

---

# 27. Foundational Dependencies

Elephant may use established Rust infrastructure for concerns that are not part of its product differentiation.

Examples include:

* async runtime
* TCP primitives
* TLS
* cryptography
* UUID
* date/time

Elephant should not reimplement these systems merely to claim zero dependencies.

However, core Elephant capabilities should remain owned by Elephant where they define the product.

These include:

```text
Query AST
Query Builder
PostgreSQL compiler
Schema type system
```

Long-term, PostgreSQL-specific execution components may also become Elephant-owned.

---

# 28. Connection Pool

Connection pooling belongs below query construction.

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

Responsibilities may include:

* connection creation
* acquisition
* release
* minimum connections
* maximum connections
* acquisition timeout
* idle timeout
* connection lifetime
* health checks
* broken connection detection
* graceful shutdown

Query ASTs must not know whether execution uses:

* a pool
* a transaction
* a dedicated connection

---

# 29. Transactions

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

Query construction should remain reusable inside and outside transactions.

Future transaction functionality may include:

* isolation levels
* read-only transactions
* deferrable transactions
* savepoints

Transactions must not alter Query Builder semantics.

---

# 30. Row Decoding

PostgreSQL responses must be decoded independently from query construction.

Conceptually:

```text
DataRow
   │
   ▼
PostgreSQL Type Information
   │
   ▼
Decoder
   │
   ▼
Rust Values
```

Row decoding is responsible for converting PostgreSQL representations into Rust values.

It should not know how the original query was constructed.

---

# 31. Typed Query Results

Elephant should eventually preserve enough information from typed projections to provide typed results.

For example:

```rust
db.select((
    User::id,
    User::name,
))
.from(User::table)
.all()
.await?
```

should ideally produce values corresponding to:

```text
(Uuid, String)
```

without requiring users to manually decode rows.

The design must balance:

* type safety
* compiler diagnostics
* compile time
* API ergonomics
* internal complexity

Do not preserve generic information through every internal layer merely because it exists at the Query Builder boundary.

Typed result information may be carried separately from the semantic AST when that produces a cleaner design.

---

# 32. Table and Struct Mapping

Elephant may provide procedural macros to reduce schema boilerplate.

A future API may resemble:

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

Macros may generate:

* table metadata
* typed columns
* mapping metadata
* repetitive trait implementations

Macros should not contain Elephant's core query semantics.

The Query Builder, AST and compiler should remain ordinary Rust implementations whenever practical.

---

# 33. Relations

Relations belong primarily to schema metadata.

Elephant may know that:

```text
posts.user_id → users.id
```

but relations must not imply hidden queries or automatic lazy loading.

Joins remain explicit.

For example:

```rust
db.select(...)
    .from(User::table)
    .left_join(
        Post::table,
        Post::user_id.eq(User::id),
    )
```

Relation metadata may improve ergonomics later, but must not turn Elephant into Active Record.

---

# 34. No Active Record

Elephant must not introduce model-instance persistence APIs such as:

```rust
user.save()
user.destroy()
user.update()
```

or class/model-style APIs such as:

```rust
User::find()
User::create()
```

Database operations originate from Elephant's query API.

Models represent data and schema information.

They do not own persistence behavior.

---

# 35. No Hidden Queries

Elephant must never perform unexpected database queries as a side effect of accessing data.

Avoid:

```text
lazy loading
automatic relationship fetching
implicit persistence
automatic reload
implicit database validation
```

Database I/O should always be visible through an execution operation.

---

# 36. Introspection

Elephant should eventually support PostgreSQL schema introspection.

Sources include:

```text
pg_catalog
information_schema
```

Introspection may discover:

* schemas
* tables
* columns
* PostgreSQL types
* primary keys
* foreign keys
* indexes
* unique constraints
* check constraints
* enums
* sequences
* extensions

Introspection belongs to tooling/schema infrastructure.

It should not be required for ordinary query construction at runtime.

---

# 37. Migrations

Migration infrastructure may use schema metadata and PostgreSQL introspection.

Conceptually:

```text
Desired Schema
      │
      ├──────────────┐
      ▼              ▼
Schema Metadata   PostgreSQL Introspection
      │              │
      └──────┬───────┘
             ▼
         Schema Diff
             │
             ▼
          Migration
```

Migration logic should remain separate from runtime query construction.

Applications should not need migration infrastructure merely to execute queries.

---

# 38. CLI

The future Elephant CLI is a tooling layer.

Potential commands include:

```text
elephant console
elephant migrate
elephant migrate:status
elephant schema:pull
elephant schema:diff
elephant generate
```

The CLI may orchestrate lower-level Elephant components.

Lower-level components must not depend on the CLI.

---

# 39. Elephant Console

Elephant should eventually provide an interactive PostgreSQL/query-builder console.

Conceptually:

```text
CLI
 │
 ▼
Console
 │
 ├── Query Builder
 ├── Compiler
 ├── Executor
 ├── Schema
 └── Introspection
```

The console is a consumer of Elephant's architecture.

Elephant's core must not depend on the console.

Target usage:

```text
$ elephant console

Elephant 0.x
Database: development
PostgreSQL 18

elephant>
```

Queries may be inspected:

```text
elephant> query.to_sql()
```

or eventually executed:

```text
elephant> db.select(...).all().await?
```

Potential console commands include:

```text
.sql
.explain
.explain_analyze
.tables
.schema
.indexes
.describe
```

Architectural decisions in lower layers should keep this use case possible without coupling those layers to interactive tooling.

---

# 40. Observability

Observability is a cross-cutting concern but must not control architecture.

Elephant should eventually integrate naturally with Rust's tracing ecosystem.

Useful boundaries include:

```text
query compilation
connection acquisition
query execution
protocol communication
row decoding
```

Core functionality must not depend on logging being configured.

Sensitive parameter values must not be automatically exposed.

Observability should inspect architectural boundaries rather than bypass them.

---

# 41. Error Boundaries

Errors should correspond to the layer that owns the failure.

Conceptually:

```text
Query Builder
    → BuildError

Compiler
    → CompileError

Connection
    → ConnectionError

Protocol
    → ProtocolError

PostgreSQL
    → DatabaseError

Decoder
    → DecodeError

Pool
    → PoolError
```

PostgreSQL errors should preserve useful server metadata when available:

```text
SQLSTATE
message
detail
hint
schema
table
column
constraint
```

Errors may later be composed into a convenient public error API.

Internal error ownership should remain clear.

---

# 42. Initial Module Boundaries

Elephant should initially remain a single crate.

A reasonable internal direction is:

```text
src/
├── lib.rs
│
├── schema/
│   ├── mod.rs
│   ├── table.rs
│   ├── column.rs
│   └── identifier.rs
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
│   ├── query.rs
│   ├── select.rs
│   ├── insert.rs
│   ├── update.rs
│   └── delete.rs
│
├── postgres/
│   ├── mod.rs
│   ├── compiler.rs
│   ├── parameter.rs
│   └── types.rs
│
└── error.rs
```

This is a direction, not a requirement to create all files immediately.

Files and modules should only be introduced when their responsibility exists.

Code organization standards are defined in `docs/development.md`.

---

# 43. Future Module Boundaries

As execution capabilities are introduced, additional internal areas may emerge:

```text
src/
├── client/
├── protocol/
├── transport/
├── pool/
├── decode/
├── migration/
└── introspection/
```

These should be created only when required by active milestones.

Do not create empty architecture for future work.

---

# 44. Future Crate Boundaries

If Elephant becomes sufficiently large, components may eventually become independent crates.

Possible workspace:

```text
elephant/
├── elephant
├── elephant-query
├── elephant-macros
├── elephant-postgres
├── elephant-protocol
├── elephant-pool
└── elephant-cli
```

This split is not an initial architectural requirement.

A crate boundary should exist only when there is a concrete reason such as:

* independent compilation
* optional dependency boundaries
* procedural macro requirements
* clearly independent public APIs
* significant build-time benefits
* independent testing/reuse

Until then, prefer a single cohesive crate.

---

# 45. Procedural Macro Boundary

Rust procedural macros require a separate crate when introduced.

When schema derive macros become necessary, a structure such as:

```text
elephant
elephant-macros
```

may therefore be justified.

The macro crate should generate metadata and implementations consumed by Elephant.

It must not become the owner of query semantics.

---

# 46. Query Lifecycle

The complete intended query lifecycle is:

```text
Schema Metadata
      │
      ▼
Typed Query Builder
      │
      ▼
Validation through Rust Types
      │
      ▼
Semantic AST
      │
      ▼
PostgreSQL Compiler
      │
      ▼
CompiledQuery
 ┌──────────────┐
 │ SQL          │
 │ Bindings     │
 └──────┬───────┘
        │
        ▼
Executor
        │
        ▼
Connection / Client
        │
        ▼
PostgreSQL Protocol
        │
        ▼
Transport
        │
        ▼
PostgreSQL
        │
        ▼
Transport
        │
        ▼
Protocol Decoder
        │
        ▼
Rows
        │
        ▼
Type Decoder
        │
        ▼
Typed Rust Result
```

Every major component should fit clearly somewhere in this lifecycle.

If a new component cannot be placed clearly, its responsibility should be reconsidered.

---

# 47. Architectural Invariants

The following rules are invariants of Elephant.

## 47.1 Query Construction Does Not Perform I/O

Building a query never communicates with PostgreSQL.

## 47.2 AST Contains Semantics, Not Rendered SQL

The AST describes the query.

The compiler renders it.

## 47.3 User Values Become Parameters

Values should use PostgreSQL bindings whenever possible.

## 47.4 Identifiers Are Not Values

Identifiers and parameters have separate representations.

## 47.5 Query Builder Is Not the AST

The public fluent API lowers into an internal representation.

## 47.6 AST Does Not Know About Execution

AST nodes do not manage connections or execute themselves.

## 47.7 Compiler Does Not Execute Queries

Compilation produces a `CompiledQuery`.

Nothing more.

## 47.8 Executor Does Not Rebuild Queries

The Executor consumes compiled queries.

## 47.9 Protocol Does Not Know Query Builder Concepts

Wire protocol code remains independent from high-level query construction.

## 47.10 PostgreSQL Is Not Abstracted Away

PostgreSQL-specific capabilities are first-class features.

## 47.11 No Active Record

Persistence does not belong to model instances.

## 47.12 No Hidden Queries

Database I/O is explicit.

## 47.13 Query Compilation Is Deterministic

Equivalent AST input produces deterministic SQL and binding ordering.

## 47.14 Type Safety Must Provide Semantic Value

Complex type machinery must prevent meaningful invalid states or improve useful inference.

## 47.15 Escape Hatches Remain Available

Advanced PostgreSQL usage must remain possible.

## 47.16 Tooling Depends on Core, Never the Reverse

CLI, console, migrations and development tooling may consume core Elephant functionality.

Core query infrastructure must not depend on those tools.

---

# 48. Architectural Decision Rules

When choosing between an ORM-like abstraction and PostgreSQL semantics:

> Prefer PostgreSQL semantics.

When choosing between implicit and explicit behavior:

> Prefer explicit behavior.

When choosing between multi-database compatibility and better PostgreSQL support:

> Prefer PostgreSQL.

When choosing between preserving compile-time information everywhere and keeping internal architecture understandable:

> Preserve compile-time information where it provides concrete value; otherwise prefer a simpler internal representation.

When choosing between clever generic machinery and understandable Rust:

> Prefer understandable Rust.

When choosing between convenience and meaningful compile-time safety:

> Prefer compile-time safety unless the resulting API becomes impractical.

When choosing between speculative flexibility and solving a current requirement:

> Solve the current requirement.

---

# 49. Architectural Change Policy

This document is expected to evolve more slowly than implementation code.

Architecture should not be changed merely because a local implementation becomes inconvenient.

An architectural change is justified when:

* a current milestone exposes a real architectural limitation;
* an invariant prevents a legitimate required capability;
* a dependency boundary is demonstrably incorrect;
* the current design creates unavoidable coupling;
* implementation experience provides evidence for a better boundary.

When an architectural change is required:

1. identify the concrete problem;
2. determine which architectural rule is affected;
3. update this document intentionally;
4. update affected development or milestone documentation;
5. only then implement the new direction.

Do not silently violate architecture and leave documentation outdated.

For significant decisions with meaningful alternatives or long-term consequences, create an Architecture Decision Record.

A future structure may be:

```text
docs/
├── architecture.md
├── development.md
├── adr/
│   ├── 0001-query-ast-representation.md
│   ├── 0002-postgres-driver-strategy.md
│   └── ...
└── milestones/
```

ADR files should explain:

```text
Context
Decision
Alternatives
Consequences
```

They should be introduced when real architectural decisions require them, not preemptively.

---

# 50. Architectural Debugging Model

The architecture should make failures traceable through clear boundaries.

For query construction:

```text
Public API
    │
    ▼
Was the typed operation interpreted correctly?
    │
    ▼
Is the AST correct?
    │
    ▼
Did the compiler render the AST correctly?
    │
    ▼
Are SQL and bindings correct?
```

For future execution:

```text
CompiledQuery
      │
      ▼
Was the connection acquired?
      │
      ▼
Was the query encoded correctly?
      │
      ▼
What did PostgreSQL return?
      │
      ▼
Was the response decoded correctly?
      │
      ▼
Was the Rust result mapped correctly?
```

A defect should be fixed in the layer that owns it.

Do not bypass architectural boundaries to patch symptoms in another layer.

---

# 51. Final Architectural Principle

Elephant should remain understandable as it grows.

Its architecture should make it possible to answer clearly:

```text
Where is a query constructed?

Where is its semantic representation?

Where does PostgreSQL-specific SQL generation happen?

Where are parameters created?

Where does execution begin?

Where is a connection managed?

Where is the PostgreSQL protocol implemented?

Where are rows decoded?

Where does schema metadata live?
```

Each question should point to a clear architectural layer.

The goal is not maximum abstraction.

The goal is a PostgreSQL library whose boundaries remain obvious, whose behavior remains predictable, and whose internals can evolve without turning the project into a tightly coupled system.

Elephant should remain:

> PostgreSQL-first, type-safe, explicit and understandable.
