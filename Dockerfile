# TODO:
# - secure Dockerfile

FROM rust:latest as builder

LABEL maintainer="Fabien Bellanger <valentil@gmail.com>"

RUN apt-get update \
    && apt-get -y install ca-certificates cmake libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy
# ----
COPY ./assets assets
COPY ./migrations migrations
COPY ./src src
COPY ./templates templates
COPY ./tests tests
COPY ./.sqlx .sqlx
COPY ./.env.docker .env
COPY ./Cargo.toml Cargo.toml

# sqlx
# ----
ENV SQLX_OFFLINE=true

# Build
# -----
ENV PKG_CONFIG_ALLOW_CROSS=1

# RUN cargo build
RUN cargo build --release

# =============================================================================

FROM gcr.io/distroless/cc AS runtime

WORKDIR /app

COPY --from=builder /app/.env .
COPY --from=builder /app/assets assets
COPY --from=builder /app/templates templates
COPY --from=builder /app/target/release/axum-boilerplate-bin .

EXPOSE 8087
ENTRYPOINT ["./axum-boilerplate-bin"]
CMD ["serve"]
