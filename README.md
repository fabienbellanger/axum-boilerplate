# Axum boilerplate

[![Build status](https://github.com/fabienbellanger/axum-boilerplate/actions/workflows/CI.yml/badge.svg?branch=main)](https://github.com/fabienbellanger/axum-boilerplate/actions/workflows/CI.yml)

[Axum Repository](https://github.com/tokio-rs/axum) framework boilerplate

[Axum Doc](https://docs.rs/axum/latest/axum)

## Cargo watch

cargo-watch repository: [Github](https://github.com/passcod/cargo-watch)

```bash
cargo watch -x 'run --bin axum-boilerplate-bin'
```

## Cargo audit

cargo-audit repository: [Github](https://github.com/RustSec/rustsec/tree/main/cargo-audit)

Installation:

```bash
cargo install cargo-audit --features=fix
```

Usage:

```bash
cargo audit
cargo audit fix
```

## Unit tests

```bash
cargo test -- --test-threads=1
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

Revert migrations:

```bash
sqlx migrate revert
```

### Offline mode

Used for Github Actions or Docker

/!\ Be careful, `sqlx` and `sqlx-cli` must be in the same version!

```bash
cargo sqlx prepare -- --bin <app name in Cargo.toml>
cargo sqlx prepare -- --lib
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

### OpenAPI

OpenAPI documentation using [Rapidoc](https://rapidocweb.com/)

YAML file: `assets/doc/doc_api_v1.yml`

URL: `<baseURL>/doc/api-v1.html`

## Docker

Run the server:

```bash
make docker
```

To create the admin user:

```bash
make docker-cli-register
```

## TODO:

- [ ] Improve global documentation
- [ ] Improve README.md to explain the boilerplate
- [ ] Add scopes (currently roles) to routes
- [ ] Add password scorer [passwords](https://docs.rs/passwords/latest/passwords/) (parameter in .env?)
- [ ] Add more .env parameters in `SharedState`?
- [ ] Replace config file .env by config.toml or add config.toml?
- [x] Add WebSocket examples
  - [x] Simple WebSocket example
  - [x] Chat WebSocket example
- [ ] Rate limiter middleware
  - [ ] Add documentation
  - [ ] Optimize code
  - [x] Add white list from `.env`
- [x] Add Docker support
  - [rust-web-server-template](https://github.com/nullren/rust-web-server-template)
  - [axum-demo](https://github.com/linux-china/axum-demo)
  - [x] Create a first user to use API
  - [x] Add Prometheus metrics ([Example](https://github.com/tokio-rs/axum/blob/main/examples/prometheus-metrics/src/main.rs)) or ([Example](https://github.com/stefanprodan/dockprom))
