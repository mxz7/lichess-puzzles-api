# Lichess Puzzles

A Rust HTTP API backed by SQLite. SQLx runs queries and applies migrations. Atlas generates and validates migrations from the declared schema.

Route handlers live in [`src/api/http/`](src/api/http/), SQLx queries in [`src/stores/`](src/stores/), and database setup in [`src/db/`](src/db/).

## Run locally

Install stable Rust through [Rustup](https://rustup.rs/). Set the database filename in `.env` or export it in your shell.

```sh
export DATABASE_URL='file:./dev.db'
cargo run --locked
```

The server listens on port 3000. `GET /` returns its status. Use `GET /puzzles/random` for an unfiltered puzzle or `GET /puzzles/random/<difficulty>` with `beginner`, `easy`, `medium`, `hard`, `expert`, or `grandmaster`. Difficulty matching is case-insensitive.

The legacy puzzle ID route returns 404 for a concrete ID and 500 for the literal `/puzzles/{id}` path.

## SQLite migrations and puzzle imports

The executable includes the versioned SQL migrations and applies them when opening the database. On the first run against an existing Prisma database, SQLx records the migrations Prisma already applied. To apply migrations without starting the server, run `cargo run --locked -- migrate`.

For schema changes, install the [Atlas CLI](https://atlasgo.io/getting-started/) and update [the SQL schema](src/db/schema.sql). Generate and validate the migration from the repository root, where Atlas loads [its project configuration](atlas.hcl):

```sh
atlas migrate diff --env local add_puzzle_index
atlas migrate validate --env local
```

Atlas writes versioned SQL and the checksum to [`src/db/migrations/`](src/db/migrations/). CI checks that replaying those migrations produces the declared schema.

After changing a checked SQLx query or its schema, regenerate the committed `.sqlx/` metadata with the [SQLx CLI](https://github.com/transact-rs/sqlx/blob/main/sqlx-cli/README.md) against a scratch database:

```sh
sqlx_tmp=$(mktemp -d)
sqlite3 "$sqlx_tmp/schema.db" < src/db/schema.sql
cargo sqlx prepare --database-url "sqlite://$sqlx_tmp/schema.db" -- --all-targets
```

Startup imports the CSV when the puzzle table is empty. To replace its contents manually, run:

```sh
cargo run --locked -- import-puzzles
```

The default input is `lichess_db_puzzle.csv` in the working directory. Set `PUZZLE_DB_PATH` to use another file and `IMPORT_MAX_ROWS` to limit the import to a positive number of puzzles. The import resets the integer IDs and commits the replacement in one SQLite transaction.

The Docker image runs the executable directly. Set `DATABASE_URL` to a path on a writable volume and mount the CSV if the database needs an initial import.

## Request logs

Requests produce Pino-compatible JSON lines on stdout by default. Set `LOKI_HOST`, `LOKI_USERNAME`, and `LOKI_PASSWORD` to send those logs to Loki with the existing `service_name=lichess-puzzles-api` label and `X-Scope-OrgID: max` tenant header.
