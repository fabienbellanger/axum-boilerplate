# Axum boilerplate

[![Build status](https://github.com/fabienbellanger/axum-boilerplate/actions/workflows/CI.yml/badge.svg?branch=main)](https://github.com/fabienbellanger/axum-boilerplate/actions/workflows/CI.yml)

[Axum Repository](https://github.com/tokio-rs/axum) framework boilerplate

[Axum Doc](https://docs.rs/axum/latest/axum)

## Features

- `ws`: Enable WebSocket support

## Cargo watch

cargo-watch repository: [Github](https://github.com/passcod/cargo-watch)
Usage:

```bash
cargo watch -x 'run --bin api'
```

With all features:

```bash
cargo watch -x 'run --all-features --bin api'
```

## Benchmark

Use [Drill](https://github.com/fcsonline/drill)

```bash
drill --benchmark drill.yml --stats --quiet
```

## SQLx

sqlx repository: [Github](https://github.com/launchbadge/sqlx)

### sqlx-cli

sqlx-cli repository: [Github](https://github.com/launchbadge/sqlx/tree/master/sqlx-cli)

### Migrations

To create a migration:

```bash
sqlx migrate add -r <name>
sqlx migrate add -r create_users_table
```

Run migrations:

```bash
sqlx migrate run
```

Revet migrations:

```bash
sqlx migrate revert
```

### Offline mode

Used for Github Actions or Docker

```bash
cargo sqlx prepare -- --bin <app name in Cargo.toml>
```

Then set env variable `SQLX_OFFLINE` to `true`.

For example:

```bash
SQLX_OFFLINE=true cargo build
```

## Documentation

Run:

```bash
cargo doc --open --no-deps
```

Run with private items:

```bash
cargo doc --open --no-deps --document-private-items
```

## TODO:

- [x] Custom errors
- [x] Add CLI
- [x] Add JWT ([Example](https://github.com/tokio-rs/axum/blob/main/examples/jwt/src/main.rs))
- [x] Add Sqlx / MySQL ([Example](https://github.com/tokio-rs/axum/blob/main/examples/sqlx-postgres/src/main.rs))
- [x] Add multiple writers to logger
- [x] Add WebSocket examples
  - [x] Simple WebSocket example
  - [x] Chat WebSocket example
- [ ] Add Askama (WIP)
- [ ] Add Prometheus metrics ([Example](https://github.com/tokio-rs/axum/blob/main/examples/prometheus-metrics/src/main.rs))
- [ ] Add Basic Auth
- [ ] Rate limiter middleware
  - [ ] Add documentation
  - [x] Add white list from `.env`
