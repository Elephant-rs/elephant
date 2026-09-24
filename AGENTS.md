# Elephant Agent Instructions

This file defines how coding agents must work inside the Elephant repository.

It is an operational entrypoint, not a replacement for the project's architecture, development guidelines, or milestone specifications.

---

# 1. Required Reading

Before modifying any code, read:

1. `docs/architecture.md`
2. `docs/development.md`
3. `docs/milestones/README.md`
4. the active milestone under `docs/milestones/`

Do not begin implementation before understanding these documents.

For architectural changes, also inspect relevant files under `docs/adr/` if that directory exists.

---

# 2. Source of Truth

The documentation hierarchy is:

```text
Architecture
     ↓
Development Guidelines
     ↓
Active Milestone
     ↓
Existing Implementation
```

Where:

```text
docs/architecture.md
```

defines architectural boundaries and invariants.

```text
docs/development.md
```

defines implementation and code-quality standards.

```text
docs/milestones/
```

defines current implementation scope and acceptance criteria.

The existing implementation must respect all three.

If implementation and documentation disagree, do not silently preserve the inconsistency.

Determine whether the implementation is incorrect or whether an explicit documentation change is required.

---

# 3. Scope

Implement only the active milestone or explicitly requested task.

Do not:

* implement future milestone functionality;
* perform unrelated refactoring;
* redesign unrelated modules;
* introduce speculative abstractions;
* create empty modules for future features;
* add infrastructure that is not currently required;
* expand the task because another design appears more interesting.

If future functionality affects a design decision, choose the smallest design that satisfies the current requirement without intentionally blocking known future requirements.

Do not implement the future functionality itself.

---

# 4. Before Coding

Before changing code:

1. inspect the repository structure;
2. inspect `Cargo.toml`;
3. inspect the relevant existing modules;
4. inspect existing tests;
5. identify the architectural layer responsible for the change;
6. identify which existing module owns that responsibility;
7. determine whether a new module or abstraction is actually necessary;
8. determine how the behavior will be tested.

Do not begin implementation by creating abstractions.

Begin by understanding the problem.

---

# 5. Architecture

All changes must preserve `docs/architecture.md`.

The primary query lifecycle is:

```text
Schema
   ↓
Query Builder
   ↓
AST
   ↓
PostgreSQL Compiler
   ↓
CompiledQuery
   ↓
Executor
   ↓
Client
   ↓
Protocol
   ↓
Transport
   ↓
PostgreSQL
```

Not every layer exists yet.

Do not create missing future layers unless required by the active milestone.

---

# 6. Architectural Boundaries

The following boundaries are mandatory.

## Query Construction

Query construction performs no I/O.

The Query Builder must not:

* acquire connections;
* access the network;
* execute queries;
* inspect live database state.

## Query Builder

The Query Builder:

* provides the public typed API;
* validates operations through Rust types where practical;
* produces semantic query representations.

It must not directly generate PostgreSQL SQL.

## AST

The AST:

* represents query semantics;
* is an internal representation;
* does not execute queries;
* does not manage connections;
* does not contain rendered SQL;
* does not depend on the Query Builder;
* does not depend on the compiler.

## PostgreSQL Compiler

The compiler:

* consumes AST;
* owns PostgreSQL SQL rendering;
* owns deterministic parameter numbering;
* produces `CompiledQuery`.

It must not execute queries.

## CompiledQuery

`CompiledQuery` is the boundary between query construction and execution.

Conceptually:

```rust
pub struct CompiledQuery {
    pub sql: String,
    pub parameters: Vec<Parameter>,
}
```

The exact implementation may evolve while preserving this responsibility.

## Execution

Execution consumes compiled queries.

It must not reconstruct high-level Query Builder state.

## Protocol

Protocol code must remain independent from:

* Query Builder;
* Schema;
* AST;
* application models.

---

# 7. PostgreSQL First

Elephant is PostgreSQL-first.

Do not introduce generic database abstractions for hypothetical support of:

* MySQL;
* SQLite;
* SQL Server;
* other SQL dialects.

PostgreSQL-specific functionality is intentional.

Prefer accurate PostgreSQL semantics over generic SQL abstractions.

---

# 8. Type Safety

Type safety is a product feature.

Use Rust's type system when it prevents meaningful invalid states.

Examples include:

* typed columns;
* compatible comparisons;
* valid expression operations;
* typed projections;
* typed result information where practical.

However:

> Type safety belongs primarily at the public API boundary.

Do not propagate generic complexity through every internal layer without concrete benefit.

The AST should prioritize a clear and stable semantic representation.

Avoid type-level complexity that significantly harms:

* readability;
* compiler diagnostics;
* compile time;
* maintainability;
* debugging.

Type safety must provide semantic value.

Generic complexity alone is not a goal.

---

# 9. SQL Semantics

SQL and PostgreSQL concepts should remain visible.

Prefer PostgreSQL terminology such as:

* SELECT;
* projection;
* expression;
* predicate;
* JOIN;
* GROUP BY;
* HAVING;
* RETURNING;
* ON CONFLICT;
* transaction;
* savepoint.

Do not invent Elephant-specific terminology for concepts PostgreSQL already names clearly.

A developer familiar with PostgreSQL should be able to reasonably predict generated SQL from Elephant code.

---

# 10. Identifiers and Values

Identifiers, parameters, and raw SQL are different concepts.

They must remain distinct internally.

```text
Identifier
Parameter
RawSql
```

Never treat user values as identifiers.

Never treat identifiers as ordinary parameter values.

User-provided data should become PostgreSQL bindings whenever possible.

Example:

```rust
users.email.eq("john@example.com")
```

should produce SQL equivalent to:

```sql
"users"."email" = $1
```

with the email represented as a binding.

Do not interpolate user values directly into SQL.

---

# 11. Determinism

Query compilation must be deterministic.

Given the same semantic query, compilation should produce deterministic:

* SQL;
* parameter ordering;
* aliases;
* bindings.

Do not introduce hidden global counters or mutable global state into query compilation.

---

# 12. Development Principles

Follow `docs/development.md`.

Prefer:

```text
clarity > cleverness
explicit > implicit
simple > generic
composition > unnecessary abstraction
small cohesive modules > large mixed-responsibility modules
domain types > ambiguous primitives
local reasoning > hidden behavior
```

Optimize primarily for maintainability and understanding.

---

# 13. Single Responsibility

Every:

* module;
* file;
* struct;
* enum;
* trait;
* function

should have a clear responsibility.

A file should exist because a distinct responsibility exists.

Do not split files merely to reduce line count.

Do not combine unrelated concepts merely to reduce the number of files.

---

# 14. File Organization

Place code near the concept it represents.

Avoid generic dumping grounds such as:

```text
utils.rs
helpers.rs
misc.rs
common.rs
```

when the contained behavior belongs to a specific domain concept.

Prefer:

```text
identifier.rs
parameter.rs
expression.rs
column.rs
```

when those are the actual responsibilities.

---

# 15. SOLID

Apply SOLID principles where they improve the design.

Do not translate SOLID into Java-style architecture.

Use idiomatic Rust mechanisms:

* modules;
* structs;
* enums;
* traits;
* generics;
* newtypes;
* composition;
* ownership.

In particular:

* keep responsibilities focused;
* keep interfaces small;
* preserve dependency direction;
* depend on contracts only when an abstraction is actually necessary;
* avoid components with unrelated reasons to change.

---

# 16. Traits

Do not create a trait merely because a struct exists.

Before introducing a trait, determine:

1. what behavioral contract it represents;
2. whether multiple meaningful implementations exist or are concretely required;
3. whether generic code needs the abstraction;
4. whether the trait improves architecture or testing.

Prefer concrete types until polymorphism is required.

---

# 17. Generics

Use generics when they provide meaningful:

* type safety;
* reusable behavior;
* relationships between types.

Do not use generics solely to make an implementation appear more abstract.

Before introducing a generic parameter, ask:

> What invalid state or meaningful duplication does this prevent?

If there is no clear answer, prefer a simpler design.

---

# 18. Abstractions

Every abstraction has a maintenance cost.

Do not introduce abstractions for hypothetical requirements.

Avoid unnecessary:

* factories;
* managers;
* providers;
* repositories;
* service locators;
* dependency injection systems;
* trait hierarchies;
* dialect abstractions;
* wrapper layers.

Before introducing an abstraction, identify the concrete current problem it solves.

---

# 19. Ownership

Design ownership intentionally.

Do not automatically solve ownership problems with `.clone()`.

Consider:

1. borrowing;
2. moving ownership;
3. changing the data structure;
4. cloning when appropriate.

Likewise, do not create difficult lifetime architectures merely to avoid cloning small values.

Prefer the simplest ownership model that is both correct and reasonably efficient.

---

# 20. Shared Mutable State

Do not introduce:

```text
Arc<Mutex<...>>
Rc<RefCell<...>>
```

as default architectural tools.

Shared ownership and interior mutability require a concrete reason.

Query construction should generally favor immutable or locally owned state.

---

# 21. Error Handling

Recoverable failures must use explicit error handling.

Prefer:

```text
Result<T, E>
Option<T>
```

where appropriate.

Do not use:

```text
unwrap()
expect()
panic!()
```

for recoverable situations.

Errors should originate from the responsible architectural layer.

Examples:

```text
BuildError
CompileError
ConnectionError
ProtocolError
DatabaseError
DecodeError
PoolError
```

Preserve useful structured error information.

Do not reduce errors to strings prematurely.

---

# 22. Unsafe Rust

Do not introduce `unsafe` unless there is a concrete technical requirement that cannot reasonably be solved with safe Rust.

If `unsafe` becomes necessary:

1. justify why safe Rust is insufficient;
2. isolate it;
3. document the safety invariants;
4. add dedicated tests;
5. validate the justification.

Do not introduce `unsafe` as a convenience or speculative optimization.

---

# 23. Comments

Do not add implementation comments by default.

Code should communicate through:

* names;
* types;
* module boundaries;
* explicit data flow.

Comments are appropriate when explaining why something non-obvious exists, particularly:

* PostgreSQL behavior;
* protocol requirements;
* safety invariants;
* performance tradeoffs;
* architectural constraints.

Do not add comments that simply narrate the code.

Rustdoc for intentional public API documentation is separate from ordinary implementation comments.

---

# 24. Public API

Treat Elephant's public API as a product.

Public API creates long-term compatibility obligations.

Default to private implementation.

Use `pub(crate)` when internal cross-module access is necessary.

Expose something publicly only when Elephant users need it.

Do not expose internal AST structures merely because doing so simplifies implementation.

Keep `lib.rs` focused on:

* module declarations;
* intentional re-exports;
* crate-level public documentation.

Do not place substantial implementation logic directly in `lib.rs`.

---

# 25. Dependencies

Do not add a dependency without a concrete requirement.

Before adding a crate, evaluate:

1. what problem it solves;
2. whether the responsibility belongs to Elephant's core;
3. whether the standard library is sufficient;
4. maintenance status;
5. API stability;
6. transitive dependencies;
7. compile-time impact;
8. runtime impact;
9. security implications.

Do not outsource Elephant's core differentiators without explicit architectural justification.

These include:

* Query AST;
* Query Builder;
* PostgreSQL compiler;
* Schema type system.

Using established infrastructure for concerns such as async runtime, TLS, cryptography, UUID, and date/time is acceptable when required.

---

# 26. Side Effects

Keep side effects at explicit architectural boundaries.

Side effects include:

* database access;
* network access;
* filesystem access;
* environment access;
* logging;
* time;
* randomness.

Query construction and compilation should remain deterministic and independently testable.

---

# 27. Testing

Every new behavior must be tested at the smallest useful level.

Use:

* unit tests for focused behavior;
* integration tests for architectural boundaries;
* compile-time tests for type guarantees;
* regression tests for fixed bugs.

Tests should be:

* deterministic;
* focused;
* fast where practical;
* independent;
* clear about what failed.

Do not rely exclusively on large integration tests.

---

# 28. Compile-Time Behavior

Because type safety is part of Elephant's product, compile-time behavior is part of the test surface.

When appropriate, verify that valid code compiles and invalid code does not.

For example:

```text
valid:

Column<i32>.eq(10)
```

and:

```text
invalid:

Column<i32>.eq("ten")
```

Compile-time guarantees should not exist only by assumption.

---

# 29. Regression Tests

When fixing a bug, add a regression test when practical.

The test should reproduce the incorrect behavior before the fix and verify the corrected behavior afterward.

Fix the responsible layer rather than patching symptoms elsewhere.

---

# 30. Refactoring

Do not rewrite working code merely because another design is preferred.

Refactoring is justified when:

* existing code violates architecture;
* existing design blocks the active requirement;
* duplication represents a real shared concept;
* complexity materially harms maintainability;
* a bug reveals an incorrect responsibility boundary.

Keep refactoring focused.

Do not combine large unrelated refactors with feature implementation.

---

# 31. Existing Patterns

Existing code is evidence, not absolute authority.

Follow existing patterns when they remain consistent with:

* architecture;
* development guidelines;
* active milestone.

Do not reproduce an existing pattern if it violates current project rules.

Do not replace a valid existing pattern solely because another style is preferred.

---

# 32. Debuggability

Debuggability is a design requirement.

Architectural stages should remain inspectable.

For query generation, failures should be traceable through:

```text
Public API
    ↓
Typed operation
    ↓
AST
    ↓
Compiler
    ↓
SQL + Bindings
```

Later execution failures should be traceable through:

```text
CompiledQuery
    ↓
Executor
    ↓
Client
    ↓
Protocol
    ↓
PostgreSQL response
    ↓
Decoder
```

Do not bypass boundaries to make debugging temporarily easier.

Improve observability at the responsible boundary instead.

---

# 33. Feature Development Workflow

For each feature:

```text
Understand requirement
        ↓
Read relevant documentation
        ↓
Inspect existing implementation
        ↓
Identify responsible layer
        ↓
Identify responsible module
        ↓
Define types and contracts
        ↓
Implement smallest correct solution
        ↓
Add focused tests
        ↓
Add integration/compile tests if needed
        ↓
Validate formatting
        ↓
Run Clippy
        ↓
Run tests
        ↓
Review against architecture
        ↓
Review milestone acceptance criteria
```

Work incrementally.

Do not implement an entire subsystem before validating its foundational pieces.

---

# 34. Before Creating a File

Ask:

> Does this file represent a distinct responsibility or concept?

If not, do not create it.

Do not create files merely because they appear in future architectural diagrams.

---

# 35. Before Creating a Trait

Ask:

> What real behavioral contract requires this abstraction now?

If there is no concrete answer, use a concrete type.

---

# 36. Before Adding a Generic

Ask:

> What meaningful type relationship or invalid state does this generic represent?

If there is no concrete answer, prefer a simpler type.

---

# 37. Before Adding a Dependency

Ask:

> Why should Elephant depend on this crate?

The answer must identify a concrete current requirement.

---

# 38. Before Making Something Public

Ask:

> Does an Elephant user need this?

If only internal modules need it, keep it internal.

---

# 39. Before Refactoring

Ask:

> What concrete problem does this refactor solve for the active task?

If the answer is unrelated to the active task, do not perform it.

---

# 40. Architectural Uncertainty

When uncertain about a design decision:

1. identify the smallest current requirement;
2. identify the responsible architectural layer;
3. preserve existing invariants;
4. choose the simplest implementation that satisfies the requirement;
5. avoid speculative extensibility.

If satisfying the requirement appears to require violating `docs/architecture.md`, do not silently violate it.

Surface the architectural conflict.

For significant architectural changes, use an ADR if the project has adopted `docs/adr/`.

---

# 41. Quality Gates

Before considering any implementation complete, run:

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

If formatting fails only because files need formatting, run:

```bash
cargo fmt
```

and validate again.

Run additional compile-time, integration, documentation, or PostgreSQL-backed tests when required by the active milestone.

---

# 42. Milestone Completion

A milestone is complete only when its documented acceptance criteria and Definition of Done are satisfied.

Before marking a milestone complete:

1. read the milestone again;
2. verify every acceptance criterion individually;
3. verify its Definition of Done;
4. run all required quality gates;
5. update only checkboxes corresponding to actually verified work;
6. leave incomplete items unchecked;
7. report incomplete requirements explicitly.

Do not mark a criterion complete based solely on implementation intent.

Verify it.

---

# 43. Do Not Start the Next Milestone

Completing one milestone does not authorize beginning another.

After completing the active milestone:

* stop implementation;
* validate the result;
* report completion;
* wait for the next explicit task.

Do not implement future milestone functionality preemptively.

---

# 44. Documentation Changes

Update documentation when implementation changes a documented public behavior or intentionally changes architecture.

Do not rewrite documentation unnecessarily during unrelated tasks.

If code and architecture disagree, resolve the disagreement explicitly.

Do not leave architectural decisions represented only in code.

---

# 45. Final Review

Before finishing a task, review the diff as a whole.

Verify:

## Scope

* Is every change required by the task?
* Was future functionality accidentally introduced?
* Is there unrelated refactoring?

## Architecture

* Does every component belong to the correct layer?
* Are dependency directions preserved?
* Did query construction remain free of I/O?
* Did SQL generation remain in the compiler?

## Responsibilities

* Does each new file have a clear responsibility?
* Does each type have a clear purpose?
* Was unrelated behavior combined?

## Complexity

* Are all abstractions necessary?
* Are traits justified?
* Are generics providing meaningful value?
* Could the implementation be simpler?

## Public API

* Is every public item intentionally public?
* Are AST internals still internal?
* Is the API predictable and PostgreSQL-oriented?

## Safety

* Are values parameterized?
* Are identifiers handled separately?
* Are recoverable errors explicit?
* Was unnecessary `unsafe` avoided?

## Testing

* Is every new behavior tested?
* Are edge cases covered?
* Are type guarantees tested where relevant?
* Are bug fixes protected by regression tests?

## Quality

* Does `cargo fmt --check` pass?
* Does Clippy pass with warnings denied?
* Does `cargo test` pass?

---

# 46. Final Report

After implementation, report:

1. what was implemented;
2. relevant architectural decisions;
3. files created or modified;
4. tests added or changed;
5. result of `cargo test`;
6. result of `cargo clippy --all-targets --all-features -- -D warnings`;
7. result of `cargo fmt --check`;
8. incomplete acceptance criteria, if any.

Keep the report factual.

Do not claim a requirement is complete unless it was implemented and verified.

---

# 47. Final Rule

Elephant should remain easy to understand, debug, test and evolve.

When choosing between two valid implementations, prefer the one that makes it easier to answer:

```text
Where does this behavior belong?

What responsibility does this type have?

What does this type guarantee?

Where does this data come from?

Where does this data go?

Which layer owns this failure?

How can this behavior be tested independently?
```

Prefer boring, explicit, cohesive Rust over clever architecture.

Preserve Elephant's core identity:

> PostgreSQL-first, type-safe, explicit and understandable.
