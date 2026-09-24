# Elephant Development Guidelines

This document defines the development standards for Elephant.

Its purpose is to keep the codebase:

* simple to understand
* easy to navigate
* easy to debug
* easy to test
* predictable
* modular
* maintainable
* resistant to accidental coupling

These rules apply to all new code and refactoring.

`docs/architecture.md` defines how Elephant is architected.

This document defines how Elephant should be developed.

---

# 1. Core Development Philosophy

Elephant should prefer:

```text
clarity > cleverness
explicit > implicit
simple > generic
composition > inheritance-like designs
small modules > large modules
small interfaces > broad interfaces
domain types > primitive obsession
compile-time guarantees > runtime checks
local reasoning > hidden behavior
```

Code should optimize primarily for humans reading and maintaining it.

Performance matters, but complexity should not be introduced without evidence that it is necessary.

A developer unfamiliar with a specific module should be able to understand its responsibility without reading the entire project.

---

# 2. Single Responsibility

Every module, file, struct, enum, trait and function should have a clear responsibility.

A useful question is:

> What is this responsible for?

The answer should normally be short.

Good:

```text
column.rs
    → represents typed database columns

expression.rs
    → represents SQL expressions

select.rs
    → represents SELECT queries

compiler.rs
    → converts AST into PostgreSQL SQL

identifier.rs
    → represents PostgreSQL identifiers

parameter.rs
    → represents query parameters
```

Bad:

```text
query.rs
```

containing:

```text
Table
Column
Expression
Select
Insert
Update
Delete
Compiler
Parameters
PostgreSQL types
```

A file becoming difficult to describe usually indicates that it has too many responsibilities.

---

# 3. File Organization

Files should be organized around concepts and responsibilities.

Prefer:

```text
src/
├── ast/
│   ├── mod.rs
│   ├── expression.rs
│   ├── select.rs
│   ├── insert.rs
│   ├── update.rs
│   └── delete.rs
│
├── query/
│   ├── mod.rs
│   ├── select.rs
│   ├── insert.rs
│   ├── update.rs
│   └── delete.rs
│
├── schema/
│   ├── mod.rs
│   ├── table.rs
│   ├── column.rs
│   └── identifier.rs
│
├── postgres/
│   ├── mod.rs
│   ├── compiler.rs
│   ├── parameter.rs
│   └── types.rs
│
├── error.rs
└── lib.rs
```

Do not create files merely to make the tree look architecturally sophisticated.

A file should exist because a distinct responsibility exists.

---

# 4. File Size

There is no strict maximum number of lines per file.

Line count is a signal, not a rule.

A 400-line file with one cohesive responsibility may be better than five artificial 80-line files.

However, when a file grows, evaluate whether it contains multiple concepts.

Signs that a file should be split:

* unrelated structs
* unrelated enums
* multiple independent workflows
* many private helpers serving different purposes
* multiple reasons for the file to change
* difficulty finding code
* tests covering unrelated behaviors
* vague filenames such as `utils.rs`, `helpers.rs` or `common.rs`

Split by responsibility, not by arbitrary line count.

---

# 5. Avoid Generic Dumping Grounds

Avoid files or modules named:

```text
utils.rs
helpers.rs
common.rs
misc.rs
shared.rs
```

unless the contained responsibility is genuinely cohesive.

Instead of:

```text
utils.rs
├── quote_identifier()
├── next_parameter()
├── parse_table()
└── format_error()
```

prefer placing behavior near its domain:

```text
identifier.rs
parameter.rs
table.rs
error.rs
```

Code should live near the concept it represents.

---

# 6. Cohesion

Things that change together should generally live together.

Things that change for unrelated reasons should generally be separated.

For example:

```text
Column<T>
ColumnType
Column operations
```

are closely related.

But:

```text
Column<T>
TCP connection handling
```

are not.

Favor high cohesion inside modules and low coupling between modules.

---

# 7. SOLID

Elephant should follow SOLID principles where they improve the design.

SOLID should not be interpreted as requiring Java-style classes, service objects or interfaces everywhere.

Rust provides different tools:

```text
modules
traits
enums
structs
generics
ownership
composition
type states
newtypes
```

Use the Rust mechanism that best expresses the principle.

---

# 8. Single Responsibility Principle

A component should have one primary reason to change.

For example:

```text
SelectBuilder
```

changes because SELECT construction changes.

```text
PostgresCompiler
```

changes because PostgreSQL SQL compilation changes.

```text
Connection
```

changes because PostgreSQL communication changes.

Do not create:

```text
SelectBuilder
```

that also:

* opens connections
* serializes protocol messages
* executes queries
* decodes rows

Keep these concerns separated.

---

# 9. Open/Closed Principle

Core systems should allow reasonable extension without requiring unrelated code to change.

For example, adding a new expression:

```text
ILIKE
```

should ideally require changes only in the expression/compiler-related layers.

It should not require modifications to:

```text
connection pool
row decoder
table representation
CLI
```

However, do not build elaborate extension systems before extension is actually required.

Prefer simple extensibility over speculative abstraction.

---

# 10. Liskov Substitution Principle

Traits should represent genuine behavioral contracts.

If multiple implementations satisfy a trait, callers should not need to know which implementation they received.

Do not create traits solely because there is a struct.

Bad:

```text
TableTrait
ColumnTrait
ExpressionTrait
ParameterTrait
```

without multiple meaningful behaviors or implementations.

Traits must represent capabilities or contracts.

---

# 11. Interface Segregation Principle

Prefer small focused traits.

Bad conceptual interface:

```text
Database
├── connect
├── execute
├── compile
├── migrate
├── introspect
├── listen
├── copy
└── pool
```

Prefer focused capabilities when abstraction is actually required:

```text
Executor
Compiler
Decoder
```

A component should depend only on the behavior it needs.

---

# 12. Dependency Inversion Principle

High-level query behavior should not depend directly on low-level implementation details.

The architecture should preserve dependency direction.

```text
Query Builder
     │
     ▼
    AST
     │
     ▼
Compiler
```

And separately:

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

Do not allow low-level modules to import higher-level query APIs.

Dependency direction must remain obvious.

---

# 13. Prefer Concrete Types Until Abstraction Is Needed

Do not create a trait before there is a reason for polymorphism.

Prefer initially:

```rust
pub struct PostgresCompiler;
```

over prematurely creating:

```rust
pub trait Compiler {
    ...
}
```

when PostgreSQL is the only compiler Elephant intends to support.

An abstraction should solve an existing design problem.

It should not exist because another implementation might theoretically exist someday.

---

# 14. Avoid Overengineering

Do not introduce:

* factories without need
* repositories around internal data structures
* managers
* providers
* service locators
* dependency injection containers
* unnecessary trait hierarchies
* generic database dialect systems
* abstract base layers
* unnecessary wrapper types
* unnecessary indirection

Every abstraction has a maintenance cost.

Before introducing one, ask:

> What concrete problem does this abstraction solve today?

If the answer is unclear, do not introduce it.

---

# 15. Prefer Composition

Build complex behavior by composing small components.

For example:

```text
Column
   ↓
Expression
   ↓
Select AST
   ↓
Compiler
```

rather than creating large objects responsible for every stage.

Each stage should be understandable independently.

---

# 16. Local Reasoning

Code should be understandable locally.

Reading:

```rust
users.active.eq(true)
```

should not require understanding:

* global state
* runtime registration
* hidden callbacks
* connection state
* thread-local variables

Avoid behavior that depends on distant mutable state.

A developer should be able to follow data through explicit function calls.

---

# 17. Explicit Data Flow

Prefer visible transformations:

```text
Builder
   ↓
AST
   ↓
Compiler
   ↓
CompiledQuery
```

over hidden side effects.

Inputs and outputs should be clear.

Functions should preferably receive what they need through parameters rather than retrieving dependencies implicitly.

---

# 18. Pure Functions

Use pure functions whenever practical, especially for:

* AST transformations
* SQL compilation
* identifier formatting
* expression construction
* parameter processing

Given the same input, these functions should produce the same output.

Pure code is easier to:

* test
* reason about
* debug
* benchmark
* reuse

I/O should remain at explicit architectural boundaries.

---

# 19. Side Effects

Side effects should be isolated.

Examples include:

```text
network access
database access
filesystem access
logging
environment variables
time
randomness
```

Do not mix these concerns into query construction.

For example:

```rust
let query = users.active.eq(true);
```

must have no side effects.

---

# 20. Function Design

Functions should generally:

* perform one logical operation
* have descriptive names
* have few parameters
* avoid hidden state
* return meaningful types
* avoid boolean flags controlling unrelated behaviors

Be cautious with APIs such as:

```rust
compile(query, true, false, true)
```

Prefer types or explicit methods representing the intended behavior.

---

# 21. Function Size

There is no arbitrary line limit.

However, a function should normally fit one conceptual level of abstraction.

If a function:

* parses AST
* generates SQL
* manages parameters
* formats errors
* performs I/O

it likely has too many responsibilities.

Extract meaningful operations when doing so improves comprehension.

Do not extract tiny functions merely to reduce line count.

---

# 22. Naming

Names should describe domain concepts.

Prefer:

```text
CompiledQuery
Parameter
Identifier
SelectQuery
Expression
Column
Table
PostgresCompiler
```

Avoid vague names:

```text
Data
Info
Manager
Processor
Handler
Helper
Util
Thing
Object
```

A good name reduces the need for comments.

---

# 23. Domain Language

Use PostgreSQL and SQL terminology whenever the concept directly corresponds to PostgreSQL.

Prefer:

```text
Select
Projection
From
Join
Predicate
Expression
Parameter
Returning
Conflict
Transaction
Savepoint
```

instead of inventing Elephant-specific terminology unnecessarily.

A PostgreSQL developer should recognize the concepts.

---

# 24. Comments

Code should primarily explain itself through:

* names
* types
* module boundaries
* explicit data flow

Avoid comments that merely repeat code.

Do not add comments by default.

Comments are appropriate when explaining:

* non-obvious PostgreSQL behavior
* safety invariants
* protocol requirements
* performance tradeoffs
* architectural constraints
* why an unusual implementation exists

Comments should explain **why**, not narrate **what** the code visibly does.

Public documentation through rustdoc is separate from implementation comments and should be added where appropriate.

---

# 25. Error Handling

Errors must be explicit.

Use:

```text
Result<T, E>
Option<T>
```

appropriately.

Do not use:

```text
unwrap()
expect()
panic!()
```

for recoverable runtime situations.

A library should not terminate the caller's application because PostgreSQL returned an error.

Errors should preserve useful context.

Avoid reducing structured errors to strings prematurely.

---

# 26. Error Ownership

Errors should belong to the layer where they originate.

Examples:

```text
query construction
    → BuildError

SQL compilation
    → CompileError

connection
    → ConnectionError

protocol
    → ProtocolError

PostgreSQL response
    → DatabaseError

decoding
    → DecodeError
```

Do not create one enormous error type containing unrelated implementation details unless exposed through a carefully designed top-level error API.

---

# 27. Type-Driven Design

Use Rust's type system to make invalid states difficult or impossible to represent.

Prefer:

```rust
Column<bool>
Column<i32>
Column<String>
```

over:

```rust
Column {
    type_name: String
}
```

when compile-time information can provide real safety.

Use:

* enums for finite states
* newtypes for distinct concepts
* generics for meaningful type relationships
* traits for capabilities
* `Option<T>` for genuine optionality

Avoid encoding domain state using arbitrary strings.

---

# 28. Newtypes

Use newtypes when primitive values represent conceptually different things and confusing them would create bugs.

Potential examples:

```text
Identifier
ParameterIndex
Oid
StatementName
PortalName
```

Do not wrap every primitive automatically.

Use newtypes when they add semantic or type safety value.

---

# 29. Enums

Prefer enums over combinations of booleans or magic strings.

Instead of:

```text
ascending = true
nulls_first = false
```

prefer representations such as:

```text
OrderDirection::Asc
NullOrdering::Last
```

when these concepts become necessary.

This makes states explicit and improves pattern matching.

---

# 30. Ownership

Design ownership intentionally.

Do not solve ownership problems automatically with `.clone()`.

Before cloning, determine:

* whether borrowing is sufficient;
* whether ownership should move;
* whether the data structure itself should change;
* whether shared ownership is actually required.

Likewise, do not create complex lifetime hierarchies merely to avoid cloning a small value.

Balance performance and maintainability.

---

# 31. `Arc`, `Rc`, `RefCell` and Mutexes

Do not use shared ownership or interior mutability as default solutions.

Their presence should have a clear reason.

Particularly avoid spreading:

```text
Arc<Mutex<...>>
```

through the query-building layers.

Query construction should favor immutable or locally owned data.

Concurrency primitives belong where concurrency actually exists.

---

# 32. Unsafe Rust

Avoid `unsafe`.

If `unsafe` becomes necessary:

1. demonstrate why safe Rust is insufficient;
2. isolate it into the smallest possible module;
3. document its safety invariants;
4. add dedicated tests;
5. benchmark if performance is the justification.

No `unsafe` should be introduced casually.

---

# 33. Dependencies

Dependencies should be treated as architectural decisions.

Before adding a crate, evaluate:

* what problem it solves;
* whether the problem belongs to Elephant's core competency;
* maintenance status;
* API stability;
* dependency tree;
* compile-time impact;
* runtime impact;
* security implications.

Do not add a dependency for trivial functionality that can be implemented clearly with the standard library.

Do not reimplement complex security-sensitive infrastructure unnecessarily.

---

# 34. Module Visibility

Default to private.

Expose something publicly only when it belongs to Elephant's public API.

Prefer:

```rust
pub(crate)
```

for cross-module internal APIs when appropriate.

Do not make fields public merely to simplify implementation.

Public API creates long-term compatibility obligations.

---

# 35. `lib.rs`

`lib.rs` should remain small.

Its primary responsibilities are:

* declaring top-level modules
* exposing the intentional public API
* providing crate-level documentation when appropriate

Do not implement substantial business logic directly in `lib.rs`.

Use intentional re-exports so users do not need to understand Elephant's internal directory structure.

---

# 36. `mod.rs`

`mod.rs` should primarily define module boundaries and intentional re-exports.

Avoid turning it into the implementation file for an entire subsystem.

Prefer:

```text
schema/
├── mod.rs
├── table.rs
├── column.rs
└── identifier.rs
```

with `mod.rs` acting as the module facade.

---

# 37. Public API vs Internal API

Maintain a clear distinction.

Users might see:

```rust
elephant::Table
elephant::Column
elephant::select
```

while internal implementation may live under:

```text
schema::table
schema::column
query::select
ast::expression
```

Internal organization should be free to evolve without unnecessarily breaking consumers.

---

# 38. Dependency Boundaries

Avoid circular conceptual dependencies.

For example:

```text
schema
   ↓
expression
   ↓
query
   ↓
compiler
```

should not accidentally become:

```text
schema
 ↕
expression
 ↕
query
 ↕
compiler
```

If two modules constantly need each other's internal details, reconsider their responsibilities.

---

# 39. Debuggability

Debuggability is a design requirement.

Important intermediate structures should be inspectable.

For example:

```text
Expression
Select AST
CompiledQuery
DatabaseError
```

should provide useful debugging representations when appropriate.

A developer investigating a failed query should eventually be able to inspect:

```text
query structure
      ↓
generated SQL
      ↓
bindings
      ↓
database response
```

without stepping through unrelated layers.

---

# 40. Deterministic Behavior

Query compilation must be deterministic.

Given the same AST, Elephant should produce the same:

* SQL
* parameter ordering
* aliases
* bindings

Determinism makes:

* debugging easier
* tests reliable
* logs comparable
* regressions easier to identify

Avoid hidden counters or global state influencing compilation.

---

# 41. Observability Boundaries

Prepare the architecture for future integration with Rust's `tracing` ecosystem.

Observability should eventually make it possible to identify:

```text
query compilation
connection acquisition
query execution
database response
```

without coupling core query types to a logging implementation.

Never require logging infrastructure for Elephant to function.

Never automatically expose sensitive parameter values.

---

# 42. Tests Follow Responsibilities

Tests should reflect module responsibilities.

Examples:

```text
identifier tests
    → quoting and escaping

expression tests
    → expression semantics

compiler tests
    → AST → PostgreSQL SQL

query builder tests
    → public API → expected AST/SQL

protocol tests
    → PostgreSQL message encoding/decoding
```

Avoid enormous integration tests as the only form of verification.

Failures should point toward the responsible subsystem.

---

# 43. Unit Tests

Unit tests should remain close to the behavior they validate when appropriate.

Tests should be:

* deterministic
* focused
* fast
* independent

A failing test should make the broken behavior obvious.

---

# 44. Integration Tests

Use integration tests for behavior spanning architectural boundaries.

Examples:

```text
public Query Builder
       ↓
AST
       ↓
Compiler
       ↓
expected SQL
```

Later:

```text
Elephant
   ↓
PostgreSQL
   ↓
typed result
```

Do not use integration tests where a smaller unit test provides clearer failure information.

---

# 45. Regression Tests

Every fixed bug should receive a regression test when practical.

The test should reproduce the original failure and prevent it from returning.

For example, if nested expressions incorrectly generate:

```sql
a = $1 AND b = $2 OR c = $3
```

instead of the required grouping, add a test covering that exact semantic case.

---

# 46. Compile-Time Tests

Because type safety is a product feature, compile behavior must be tested.

Test both:

```text
valid code compiles
```

and:

```text
invalid code does not compile
```

where relevant.

Examples include:

```text
valid:
Column<i32>.eq(10)

invalid:
Column<i32>.eq("ten")
```

Compiler guarantees are part of Elephant's behavior.

---

# 47. Avoid Test Coupling

Tests should validate behavior rather than internal implementation details whenever possible.

A compiler test should care that:

```text
AST → expected SQL + bindings
```

works.

It should not fail merely because an internal private enum was reorganized without changing behavior.

---

# 48. Refactoring

Refactoring must preserve behavior unless behavior change is intentional.

Before significant refactoring:

1. ensure relevant tests exist;
2. identify the responsibility being improved;
3. make the smallest coherent change;
4. run tests;
5. run Clippy;
6. validate formatting.

Avoid mixing major refactoring with unrelated feature work.

---

# 49. Boy Scout Rule

When modifying an area, small improvements are acceptable if they directly improve the touched code.

Examples:

* better name
* removing obvious duplication
* reducing unnecessary visibility
* simplifying a function

Do not use this principle as justification for rewriting unrelated modules.

---

# 50. Duplication

Do not abstract at the first sign of duplicated code.

Small duplication can be preferable to a bad abstraction.

Consider abstraction when duplicated code represents the same concept and changes for the same reason.

The rule is:

> Prefer a little duplication over the wrong abstraction.

---

# 51. Premature Generalization

Do not design for hypothetical requirements.

Examples:

Do not introduce:

```text
DatabaseDialect
```

because Elephant might support MySQL someday.

Do not introduce:

```text
SyncExecutor
AsyncExecutor
DistributedExecutor
```

without requirements for them.

Solve the current problem while leaving reasonable room for evolution.

---

# 52. Feature Development Workflow

Every feature should follow approximately:

```text
Understand requirement
        ↓
Identify responsible module
        ↓
Define types/contracts
        ↓
Implement smallest solution
        ↓
Unit tests
        ↓
Integration tests if necessary
        ↓
cargo fmt
        ↓
cargo clippy
        ↓
cargo test
        ↓
Review architecture
```

Do not start implementation by creating a large hierarchy of files and abstractions.

---

# 53. Before Writing Code

Before implementing a feature, answer:

1. What responsibility does this feature belong to?
2. Which existing module owns that responsibility?
3. Does a new module actually need to exist?
4. What are the inputs?
5. What are the outputs?
6. What invalid states can Rust prevent?
7. What side effects exist?
8. Can those side effects remain isolated?
9. How will this behavior be tested?
10. Does this change respect `docs/architecture.md`?

Only then implement.

---

# 54. Before Creating a New File

Ask:

> Does this represent a distinct responsibility or concept?

If yes, create the file.

If the reason is merely:

> The existing file is getting long.

first determine whether there is actually a responsibility to extract.

File boundaries should follow conceptual boundaries.

---

# 55. Before Creating a Trait

Ask:

1. What behavioral contract does this represent?
2. Are there multiple meaningful implementations?
3. Does generic code actually need this abstraction?
4. Does it improve testing or architecture?
5. Could a concrete type solve the problem more clearly?

If there is no strong answer, use a concrete type.

---

# 56. Before Adding a Generic

Ask:

> What invalid state or duplication does this generic prevent?

Generics should carry semantic value.

Do not create deeply nested generic types purely to demonstrate compile-time sophistication.

Type safety that makes the library unusable is not a successful design.

---

# 57. Before Adding a Dependency

Ask:

1. Can the standard library solve this clearly?
2. Is this outside Elephant's core competency?
3. Is the crate maintained?
4. What does it add transitively?
5. Does it affect compile time significantly?
6. Would replacing it later be difficult?

Dependencies should remain intentional.

---

# 58. Code Review Checklist

Every meaningful change should be reviewed against the following questions.

## Responsibility

* Does every new file have a clear responsibility?
* Does every new type have a clear purpose?
* Does any component now have multiple unrelated reasons to change?

## Architecture

* Does dependency direction remain correct?
* Are architectural boundaries preserved?
* Is query construction still independent from execution?
* Is PostgreSQL-specific behavior in an appropriate layer?

## API

* Is new public API actually necessary?
* Is the naming consistent with SQL/PostgreSQL terminology?
* Can invalid states be prevented through types?
* Is the API predictable?

## Complexity

* Was unnecessary abstraction introduced?
* Was a trait introduced without a real contract?
* Was generic complexity introduced without meaningful safety?
* Is there unnecessary indirection?
* Could the implementation be simpler?

## Ownership

* Are clones justified?
* Is shared ownership actually required?
* Are lifetimes reasonably understandable?

## Errors

* Are recoverable errors represented through `Result`?
* Are errors owned by the appropriate layer?
* Is useful context preserved?

## Testing

* Is new behavior tested?
* Are edge cases covered?
* Does a regression test exist for fixed bugs?
* Are compile-time guarantees tested where relevant?

## Quality

* Does `cargo fmt --check` pass?
* Does Clippy pass with warnings denied?
* Does `cargo test` pass?

---

# 59. Required Quality Gates

Before considering a task complete, run:

```bash
cargo fmt --check
```

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

```bash
cargo test
```

All must pass.

When appropriate, also run documentation and compile-time tests.

A milestone is not complete merely because the implementation appears to work.

---

# 60. Agent Development Rules

When an AI coding agent modifies Elephant, it must follow these additional rules.

Before coding:

1. read `docs/architecture.md`;
2. read this document;
3. read the active milestone completely;
4. inspect existing implementation before proposing new abstractions.

During implementation:

* work incrementally;
* preserve existing architectural boundaries;
* keep each file focused;
* avoid speculative abstractions;
* avoid unrelated refactoring;
* prefer existing patterns when they remain appropriate;
* do not duplicate existing concepts;
* keep public API minimal;
* do not add comments unless they explain a non-obvious technical reason;
* do not add dependencies without concrete justification;
* do not implement future milestone functionality.

When encountering architectural uncertainty:

> Stop expanding scope and choose the smallest design consistent with the current architecture and milestone.

Do not attempt to make the architecture future-proof through speculative complexity.

---

# 61. Agent Refactoring Rules

An agent must not rewrite working components merely because it prefers another style.

Refactoring is justified when:

* current code violates architecture;
* current design prevents the active requirement;
* duplication represents a real shared concept;
* complexity materially harms maintainability;
* tests expose a design problem.

When refactoring is required, keep it focused and explain the architectural reason in the final report.

---

# 62. Debugging Principle

When a failure occurs, the architecture should make it possible to identify which stage is responsible.

For query generation:

```text
Was the public API interpreted correctly?
              ↓
Is the AST correct?
              ↓
Did the compiler render the AST correctly?
              ↓
Are bindings correct?
```

Later, for execution:

```text
Is CompiledQuery correct?
          ↓
Was the connection acquired?
          ↓
Was the protocol message correct?
          ↓
What did PostgreSQL return?
          ↓
Was the result decoded correctly?
```

Do not bypass these boundaries when fixing bugs.

Fix the layer that owns the defect.

---

# 63. Final Rule

The goal is not to produce the most architecturally sophisticated Rust library.

The goal is to produce a library whose code can be opened months later and understood quickly.

A good Elephant implementation should make it easy to answer:

```text
Where is this behavior implemented?

Why does this module exist?

What does this type guarantee?

Where does this data come from?

Where does this data go?

Which layer is responsible for this error?

How can I test this behavior independently?
```

If answering these questions requires understanding most of the codebase, the design is too coupled.

Prefer boring, explicit, well-separated code over clever architecture.
