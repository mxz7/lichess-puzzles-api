# Design rules

- In production code, call store functions through `stores::`, for example `stores::puzzles::find_by_id`. Store types may be imported directly. Tests may also import store functions directly.
- Keep stores as functions that accept a SQLx executor. The caller owns the pool and any transaction.
- Use SQLx's `query!`, `query_as!`, or `query_scalar!` macros for fixed SQL that can be checked against the project schema. Update the checked-in `.sqlx/` metadata when the SQL or its schema changes. Use runtime SQL for dynamic statements, multi-statement scripts, and queries against legacy schemas.
- Every query that returns a SQL JSON aggregate must have a `#[sqlx::test]` that executes the query and deserializes a populated result into the expected Rust type. Also assert that the empty case returns exactly `[]`. This catches misspelled keys, shape drift, and accidental `[null]` results.
