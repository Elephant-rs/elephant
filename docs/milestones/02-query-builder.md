# Milestone 02 — Query Builder API

## Status

Planned

## Target Version

`0.0.3`

## Goal

Transform Elephant's Query Core into an ergonomic, composable, PostgreSQL-oriented, type-safe public Query Builder API.

Milestone 01 established:

```text
Schema Primitives
      ↓
Expressions
      ↓
SELECT AST
      ↓
PostgreSQL Compiler
      ↓
CompiledQuery
```

Milestone 02 builds the primary developer-facing API on top of those foundations.

The focus of this milestone is:

```text
ergonomics
+
composition
+
type safety
+
PostgreSQL semantics
```

Database execution remains outside the scope.

---

# 1. Required Reading

Before implementation, follow `AGENTS.md`.

Read:

1. `docs/architecture.md`
2. `docs/development.md`
3. `docs/milestones/01-query-core.md`
4. this milestone

Inspect the completed Milestone 01 implementation before changing it.

Do not redesign Query Core unless a concrete Milestone 02 requirement exposes a real limitation.

---

# 2. Scope

This milestone includes:

* public Query Builder entry point;
* fluent SELECT API;
* typed projections;
* SELECT-all;
* WHERE ergonomics;
* NULL predicates;
* LIKE;
* ILIKE;
* IN;
* ORDER BY;
* LIMIT;
* OFFSET;
* DISTINCT;
* DISTINCT ON;
* aliases;
* JOINs;
* column-to-column comparisons;
* GROUP BY;
* aggregate functions;
* HAVING;
* expression aliases;
* basic SELECT subqueries;
* query composition;
* query inspection;
* compile-time type guarantees.

This milestone remains entirely independent from database execution.

---

# 3. Target API

Elephant should support an API conceptually equivalent to:

```rust
let query = db
    .select((
        users.id,
        users.name,
        users.email,
    ))
    .from(users)
    .where_(
        users.active
            .eq(true)
            .and(users.age.gte(18))
    )
    .order_by(users.created_at.desc())
    .limit(20);

let compiled = query.to_sql()?;
```

`db` in this milestone is a query-building entry point.

It does not represent a live database connection.

No I/O may occur.

---

# 4. Query Builder Entry Point

Introduce a lightweight public entry point for constructing queries.

A possible API is:

```rust
let db = Elephant::new();
```

The exact name may differ if implementation experience produces a clearer public API.

The entry point must not:

* connect to PostgreSQL;
* contain a connection pool;
* perform I/O;
* require async runtime state.

Its purpose is to provide a coherent public query-building API.

---

# 5. Fluent SELECT

Support fluent SELECT construction.

Example:

```rust
db.select((
    users.id,
    users.name,
))
.from(users)
```

The fluent API must lower into the semantic AST established by Milestone 01.

The fluent builder must not generate SQL directly.

---

# 6. Typed Projection

Projection must preserve meaningful type information.

Support at least:

```rust
db.select(users.id)
```

and heterogeneous tuples:

```rust
db.select((
    users.id,
    users.name,
    users.active,
))
```

Projection typing should provide a foundation for future typed query results.

Do not use an untyped heterogeneous collection as the primary public projection API when Rust's type system can preserve projection structure.

Support a reasonable tuple arity.

Do not introduce excessive tuple machinery beyond practical usage.

---

# 7. SELECT All

Provide an explicit way to select all columns.

Possible APIs include:

```rust
db.select_all().from(users)
```

or:

```rust
db.select(users.all()).from(users)
```

Choose the design that best fits the implemented schema model.

SELECT-all behavior must be explicit.

Do not make an empty projection silently mean `*` unless intentionally documented.

---

# 8. FROM

The fluent API must support:

```rust
.from(users)
```

and preserve schema-qualified tables from Milestone 01.

---

# 9. WHERE

WHERE must accept typed expressions.

Example:

```rust
.where_(users.active.eq(true))
```

Arbitrary strings must not be accepted as normal WHERE predicates.

The API must preserve expression composition from Query Core.

---

# 10. Comparison Ergonomics

The public API must expose typed operations including:

```rust
.eq(...)
.ne(...)
.gt(...)
.gte(...)
.lt(...)
.lte(...)
```

where semantically valid.

The type system must prevent incompatible value comparisons where practical.

---

# 11. Column-to-Column Comparisons

Support comparisons between compatible columns.

Example:

```rust
posts.user_id.eq(users.id)
```

This must compile to SQL equivalent to:

```sql
"posts"."user_id" = "users"."id"
```

It must not create a PostgreSQL binding.

Column-to-column comparison must remain type-safe where practical.

Incompatible column types should not be accepted by the typed API.

---

# 12. Logical Composition

Preserve ergonomic:

```rust
.and(...)
.or(...)
```

Example:

```rust
users.active
    .eq(true)
    .and(
        users.age
            .gte(18)
            .or(users.admin.eq(true))
    )
```

Compilation must preserve semantic grouping.

---

# 13. NULL Semantics

Introduce explicit NULL predicates:

```rust
users.deleted_at.is_null()
users.deleted_at.is_not_null()
```

Equivalent SQL:

```sql
"users"."deleted_at" IS NULL
```

and:

```sql
"users"."deleted_at" IS NOT NULL
```

Do not compile NULL comparisons as:

```sql
= NULL
```

or:

```sql
!= NULL
```

The public API should guide users toward correct SQL NULL semantics.

---

# 14. LIKE

Support PostgreSQL LIKE for compatible textual expressions.

Example:

```rust
users.name.like("John%")
```

Equivalent SQL:

```sql
"users"."name" LIKE $1
```

The pattern must remain a parameter.

LIKE must not be available for clearly incompatible types through the typed API.

---

# 15. ILIKE

Support PostgreSQL-specific ILIKE.

Example:

```rust
users.name.ilike("%john%")
```

Equivalent SQL:

```sql
"users"."name" ILIKE $1
```

The pattern must remain a parameter.

An operation conceptually equivalent to:

```rust
users.age.ilike("%18%")
```

must not be accepted by the typed API.

ILIKE is intentionally PostgreSQL-specific.

Do not abstract it into generic SQL terminology.

---

# 16. IN

Support typed IN predicates.

Example:

```rust
users.id.in_([id1, id2, id3])
```

Equivalent SQL:

```sql
"users"."id" IN ($1, $2, $3)
```

Bindings must preserve input ordering.

Values must be compatible with the column type.

---

# 17. Empty IN

Empty collections must be handled deterministically.

Elephant must never generate invalid SQL such as:

```sql
IN ()
```

Choose a semantic representation that safely produces an always-false predicate or another explicitly documented equivalent.

The behavior must be tested.

Do not silently drop the WHERE predicate.

---

# 18. ORDER BY

Provide fluent ordering.

Examples:

```rust
.order_by(users.created_at.desc())
```

and multiple expressions:

```rust
.order_by((
    users.created_at.desc(),
    users.name.asc(),
))
```

Equivalent SQL:

```sql
ORDER BY
    "users"."created_at" DESC,
    "users"."name" ASC
```

The exact tuple/list API may vary while preserving typed expressions.

---

# 19. LIMIT and OFFSET

Expose the Query Core LIMIT and OFFSET functionality through the fluent API.

Example:

```rust
.limit(20)
.offset(40)
```

Invalid negative values must remain impossible to represent.

---

# 20. DISTINCT

Support:

```rust
db.select(...)
    .distinct()
    .from(users)
```

Equivalent SQL:

```sql
SELECT DISTINCT ...
```

DISTINCT should be represented semantically in the AST.

It must not be implemented through string manipulation after compilation.

---

# 21. DISTINCT ON

Support PostgreSQL-specific `DISTINCT ON`.

Conceptually:

```rust
db.select(...)
    .distinct_on(users.email)
    .from(users)
```

Equivalent SQL:

```sql
SELECT DISTINCT ON ("users"."email") ...
```

Multiple DISTINCT ON expressions should be representable.

This feature should remain explicitly PostgreSQL-specific.

---

# 22. Table Aliases

Support table aliases.

Example:

```rust
let u = users.alias("u");
```

Then:

```rust
db.select((
    u.id,
    u.name,
))
.from(u)
```

should compile using the alias:

```sql
SELECT "u"."id", "u"."name"
FROM "users" AS "u"
```

Aliased columns must preserve their type information.

Alias identifiers must use PostgreSQL identifier escaping.

---

# 23. Expression Aliases

Support expression aliases where appropriate.

Example:

```rust
count(posts.id).alias("post_count")
```

Equivalent SQL:

```sql
COUNT("posts"."id") AS "post_count"
```

Aliases must remain identifiers, not parameters.

---

# 24. JOINs

Support:

* INNER JOIN;
* LEFT JOIN;
* RIGHT JOIN;
* FULL JOIN.

Example:

```rust
db.select(...)
    .from(users)
    .left_join(
        posts,
        posts.user_id.eq(users.id),
    )
```

Equivalent SQL:

```sql
FROM "users"
LEFT JOIN "posts"
    ON "posts"."user_id" = "users"."id"
```

JOIN predicates must use semantic expressions.

Do not accept arbitrary strings as normal JOIN conditions.

---

# 25. JOIN Type Safety

Column comparisons used in JOIN conditions should preserve type compatibility.

For example:

```rust
posts.user_id.eq(users.id)
```

should work when both columns represent compatible types.

Clearly incompatible columns should be rejected by the typed API.

Do not introduce a generic relation system merely to implement JOIN.

Relations belong to later schema functionality.

---

# 26. GROUP BY

Support GROUP BY.

Example:

```rust
.group_by((
    users.id,
    users.name,
))
```

Equivalent SQL:

```sql
GROUP BY "users"."id", "users"."name"
```

GROUP BY should accept appropriate semantic expressions.

---

# 27. Aggregate Functions

Support an initial set of aggregate functions:

```text
COUNT
SUM
AVG
MIN
MAX
```

Examples:

```rust
count(posts.id)
sum(orders.total)
avg(orders.total)
min(orders.total)
max(orders.total)
```

Aggregate expressions must integrate with:

* projections;
* aliases;
* HAVING;
* ordering where appropriate.

Do not design the AST as an enum containing every PostgreSQL function that might ever exist.

The function representation should allow future PostgreSQL functions without requiring a redesign of the entire AST.

---

# 28. COUNT

COUNT deserves explicit validation because it is commonly used.

Support at least:

```rust
count(posts.id)
```

If `COUNT(*)` is introduced, it must have an explicit semantic representation.

Do not represent `*` as an arbitrary string.

---

# 29. HAVING

Support HAVING.

Example:

```rust
.having(count(posts.id).gt(2))
```

Equivalent SQL:

```sql
HAVING COUNT("posts"."id") > $1
```

HAVING uses semantic expressions and normal binding rules.

---

# 30. Subqueries

Support basic SELECT subqueries.

A SELECT query should be representable as an expression/source where PostgreSQL semantics permit it.

At minimum, the architecture must support a nested SELECT in a meaningful tested scenario.

Nested query compilation must preserve global parameter numbering.

For example, if the outer query uses `$1`, bindings inside a subsequent subquery must continue with `$2`, `$3`, and so on.

A nested compiler invocation must not accidentally reset parameter numbering.

---

# 31. Query Composition

Queries and expressions should be composable without requiring users to rebuild them from strings.

Example conceptual behavior:

```rust
let active = users.active.eq(true);
let adult = users.age.gte(18);

let filter = active.and(adult);
```

The ownership model should make common composition practical.

Do not solve every composition issue by requiring `.clone()` everywhere.

Do not create excessive lifetime complexity merely to eliminate small clones.

Follow the ownership principles in `docs/development.md`.

---

# 32. Query Inspection

Queries must be compilable without execution.

Provide:

```rust
let compiled = query.to_sql()?;
```

or an equivalent intentional public API.

The result must expose enough information to inspect:

```text
SQL
bindings
```

No PostgreSQL server may be required.

This capability is foundational for:

* debugging;
* tests;
* future CLI tooling;
* future Elephant Console.

---

# 33. Debug Representation

Important query structures should provide useful `Debug` representations where appropriate.

Debugging should allow developers to inspect the query-building pipeline without exposing implementation internals as stable public API.

Do not automatically log parameter values.

Debug support and logging policy are separate concerns.

---

# 34. Function Representation

Do not hardcode every possible PostgreSQL function as a dedicated AST variant.

Prefer a representation that allows known typed functions to lower into a general semantic function-call node where appropriate.

For example, the public API may provide:

```rust
count(...)
avg(...)
```

while the AST represents a semantic function invocation.

The design must still distinguish functions from arbitrary raw SQL.

---

# 35. Public API Design

Public API quality is a primary deliverable of this milestone.

Evaluate APIs for:

* readability;
* discoverability;
* consistency;
* PostgreSQL familiarity;
* compiler diagnostics;
* type safety;
* composability.

Prefer:

```rust
users.name.ilike("%john%")
```

over APIs requiring users to manually construct AST nodes.

Avoid exposing implementation details merely to reduce internal code.

---

# 36. Compiler Diagnostics

Type safety is useful only if failures remain understandable.

Avoid generic designs that cause simple mistakes to produce incomprehensible compiler errors.

When choosing between equivalent type-safe designs, prefer the one producing clearer diagnostics.

Compile-fail tests should help evaluate this.

---

# 37. Internal Refactoring

Milestone 02 may expose limitations in Milestone 01.

Focused refactoring is allowed when necessary to support:

* fluent API;
* typed projection;
* column-to-column comparisons;
* aliases;
* JOINs;
* aggregates;
* subqueries;
* expression composition.

Do not rewrite Query Core merely because another implementation style is preferred.

Any significant refactoring must preserve architectural boundaries.

---

# 38. No Execution API

Do not introduce methods implying database execution.

Do not implement:

```text
.all()
.one()
.first()
.execute()
```

if those methods imply actual database I/O.

Query inspection through:

```text
.to_sql()
```

is allowed and required.

Execution belongs to a later milestone.

---

# 39. Suggested Target Query

The completed Query Builder should support code conceptually equivalent to:

```rust
let query = db
    .select((
        users.id,
        users.name,
        count(posts.id).alias("post_count"),
    ))
    .from(users)
    .left_join(
        posts,
        posts.user_id.eq(users.id),
    )
    .where_(
        users.active
            .eq(true)
            .and(users.age.gte(18))
    )
    .group_by((
        users.id,
        users.name,
    ))
    .having(count(posts.id).gt(2))
    .order_by(users.created_at.desc())
    .limit(20);

let compiled = query.to_sql()?;
```

Equivalent PostgreSQL:

```sql
SELECT
    "users"."id",
    "users"."name",
    COUNT("posts"."id") AS "post_count"
FROM "users"
LEFT JOIN "posts"
    ON "posts"."user_id" = "users"."id"
WHERE "users"."active" = $1
  AND "users"."age" >= $2
GROUP BY
    "users"."id",
    "users"."name"
HAVING COUNT("posts"."id") > $3
ORDER BY "users"."created_at" DESC
LIMIT 20
```

with bindings:

```text
$1 = true
$2 = 18
$3 = 2
```

---

# 40. Testing Requirements

Tests must cover at least:

* Query Builder entry point;
* single-column projection;
* tuple projection;
* SELECT-all;
* FROM;
* WHERE;
* all comparison operators inherited from Query Core;
* AND;
* OR;
* nested expressions;
* NULL predicates;
* LIKE;
* ILIKE;
* incompatible ILIKE compile failure;
* IN with one value;
* IN with multiple values;
* empty IN;
* IN binding order;
* ORDER BY;
* multiple ORDER BY expressions;
* LIMIT;
* OFFSET;
* DISTINCT;
* DISTINCT ON;
* table aliases;
* alias identifier escaping;
* expression aliases;
* INNER JOIN;
* LEFT JOIN;
* RIGHT JOIN;
* FULL JOIN;
* column-to-column comparisons;
* GROUP BY;
* COUNT;
* SUM;
* AVG;
* MIN;
* MAX;
* HAVING;
* basic SELECT subquery;
* nested parameter numbering;
* query composition;
* deterministic compilation;
* query inspection.

---

# 41. Compile-Time Tests

Compile-time tests must cover meaningful type guarantees.

At minimum verify appropriate behavior for:

```text
Column<i32>.eq(i32)
Column<String>.eq(&str)
compatible Column<T>.eq(Column<T>)
textual_column.like(pattern)
textual_column.ilike(pattern)
```

and rejection of inappropriate operations such as:

```text
Column<i32>.eq(&str)
Column<i32>.ilike(pattern)
incompatible_column.eq(other_column)
```

The exact compile-test mechanism is an implementation decision.

Compiler diagnostics should remain reasonably understandable.

---

# 42. Security Expectations

All ordinary values introduced through the Query Builder must remain PostgreSQL parameters.

This includes:

* comparison values;
* LIKE patterns;
* ILIKE patterns;
* IN values;
* HAVING values.

Aliases and identifiers must use identifier quoting.

JOIN column comparisons must render as identifiers, not parameters.

No API introduced by this milestone should accidentally interpolate user values into generated SQL.

---

# 43. Determinism

All features introduced in this milestone must preserve deterministic compilation.

Particularly verify:

* IN parameter ordering;
* nested subquery numbering;
* JOIN compilation;
* aggregate bindings;
* aliases;
* composed expressions.

Equivalent query structures must produce equivalent SQL and binding order.

---

# 44. Out of Scope

Do not implement:

* PostgreSQL connections;
* TCP;
* TLS;
* PostgreSQL wire protocol;
* connection pools;
* async execution;
* row decoding;
* typed runtime result mapping;
* INSERT;
* UPDATE;
* DELETE;
* RETURNING;
* ON CONFLICT;
* transactions;
* migrations;
* schema introspection;
* procedural schema macros;
* relation loading;
* Active Record;
* lazy loading;
* CLI;
* Elephant Console.

Do not begin mutation support as part of this milestone.

---

# 45. Acceptance Criteria

## Query Builder

* [ ] A public query-building entry point exists.
* [ ] It requires no database connection.
* [ ] Fluent SELECT construction is supported.
* [ ] Query Builder lowers into AST rather than rendering SQL directly.
* [ ] Query inspection is available through `to_sql()` or equivalent.

## Projection

* [ ] Single-column projection is supported.
* [ ] Typed tuple projections are supported.
* [ ] SELECT-all is explicit.
* [ ] Projection information remains useful for future typed results.

## Expressions

* [ ] Typed value comparisons remain supported.
* [ ] Column-to-column comparisons are supported.
* [ ] Column-to-column comparisons do not create bindings.
* [ ] AND is supported.
* [ ] OR is supported.
* [ ] NULL predicates are supported.
* [ ] LIKE is supported for compatible types.
* [ ] ILIKE is supported for compatible types.
* [ ] ILIKE is rejected for incompatible types.
* [ ] IN is supported.
* [ ] Empty IN never generates `IN ()`.

## Query Features

* [ ] ORDER BY is supported.
* [ ] Multiple ORDER BY expressions are supported.
* [ ] LIMIT is supported.
* [ ] OFFSET is supported.
* [ ] DISTINCT is supported.
* [ ] DISTINCT ON is supported.
* [ ] Table aliases are supported.
* [ ] Expression aliases are supported.

## JOINs

* [ ] INNER JOIN is supported.
* [ ] LEFT JOIN is supported.
* [ ] RIGHT JOIN is supported.
* [ ] FULL JOIN is supported.
* [ ] JOIN conditions use semantic expressions.
* [ ] Compatible column types can be compared.
* [ ] Clearly incompatible column comparisons are rejected.

## Aggregation

* [ ] GROUP BY is supported.
* [ ] COUNT is supported.
* [ ] SUM is supported.
* [ ] AVG is supported.
* [ ] MIN is supported.
* [ ] MAX is supported.
* [ ] Aggregate aliases are supported.
* [ ] HAVING is supported.
* [ ] Function representation remains extensible.

## Subqueries

* [ ] Basic SELECT subqueries are supported.
* [ ] Nested bindings use correct global parameter numbering.
* [ ] Subquery compilation does not independently reset parameter numbering.

## Composition

* [ ] Expressions can be reused and composed.
* [ ] Common composition does not require excessive cloning.
* [ ] Ownership remains understandable.
* [ ] Query compilation remains deterministic.

## Architecture

* [ ] Query construction performs no I/O.
* [ ] Query Builder does not render SQL directly.
* [ ] AST remains independent from Query Builder.
* [ ] AST remains independent from PostgreSQL Compiler.
* [ ] Compiler remains independent from execution.
* [ ] No execution infrastructure was introduced.
* [ ] No generic database dialect abstraction was introduced.
* [ ] AST internals were not unnecessarily exposed publicly.

## Testing

* [ ] Required behavior tests exist.
* [ ] Compile-time type guarantees are tested.
* [ ] Incompatible operations are compile-tested.
* [ ] Empty IN behavior is tested.
* [ ] JOIN behavior is tested.
* [ ] Aggregate behavior is tested.
* [ ] Subquery parameter numbering is tested.
* [ ] Deterministic compilation is tested.

## Quality

* [ ] `cargo fmt --check` passes.
* [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes.
* [ ] `cargo test` passes.

---

# 46. Definition of Done

Milestone 02 is complete when Elephant can express a query equivalent to:

```rust
let query = db
    .select((
        users.id,
        users.name,
        count(posts.id).alias("post_count"),
    ))
    .from(users)
    .left_join(
        posts,
        posts.user_id.eq(users.id),
    )
    .where_(
        users.active
            .eq(true)
            .and(users.age.gte(18))
    )
    .group_by((
        users.id,
        users.name,
    ))
    .having(count(posts.id).gt(2))
    .order_by(users.created_at.desc())
    .limit(20);

let compiled = query.to_sql()?;
```

without PostgreSQL running.

The resulting SQL must be equivalent to:

```sql
SELECT
    "users"."id",
    "users"."name",
    COUNT("posts"."id") AS "post_count"
FROM "users"
LEFT JOIN "posts"
    ON "posts"."user_id" = "users"."id"
WHERE "users"."active" = $1
  AND "users"."age" >= $2
GROUP BY
    "users"."id",
    "users"."name"
HAVING COUNT("posts"."id") > $3
ORDER BY "users"."created_at" DESC
LIMIT 20
```

with ordered bindings equivalent to:

```text
true
18
2
```

The implementation must also demonstrate:

* typed projection;
* typed comparisons;
* column-to-column comparison;
* NULL semantics;
* LIKE and ILIKE;
* IN;
* DISTINCT;
* DISTINCT ON;
* aliases;
* all required JOIN types;
* aggregation;
* HAVING;
* basic subqueries;
* deterministic nested parameter numbering;
* offline query inspection.

The following must pass:

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

No PostgreSQL server may be required.

Do not begin Milestone 03 automatically.
