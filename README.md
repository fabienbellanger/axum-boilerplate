# Axum boilerplate

[Axum Repository](https://github.com/tokio-rs/axum) framework boilerplate

[Axum Doc](https://docs.rs/axum/latest/axum)

## Cargo watch

cargo-watch repository: [Github](https://github.com/passcod/cargo-watch)
Usage:

```bash
cargo watch -x 'run --bin api'
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

## TODO:

- [x] Custom errors
- [x] Add CLI
- [x] Add JWT ([Example](https://github.com/tokio-rs/axum/blob/main/examples/jwt/src/main.rs))
- [x] Add Sqlx / MySQL ([Example](https://github.com/tokio-rs/axum/blob/main/examples/sqlx-postgres/src/main.rs))
- [ ] Add Askama
- [ ] Add Basic Auth
- [ ] Add Prometheus metrics ([Example](https://github.com/tokio-rs/axum/blob/main/examples/prometheus-metrics/src/main.rs))
- [ ] Add multiple writers to logger