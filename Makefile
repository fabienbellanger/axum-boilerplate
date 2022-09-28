.PHONY: help \
	serve \
	check \
	test \
	clean \
	sqlx-prepare \
	doc \
	doc-deps \
	docker \
	docker-up \
	docker-up-no-daemon \
	docker-down \
	docker-down-rm

.DEFAULT_GOAL=help

# Parameters
APP_NAME="Axum Boilerplate"
CURRENT_PATH=$(shell pwd)
DOCKER_COMPOSE=docker-compose
CARGO=cargo
CARGO_BIN_NAME=axum-boilerplate-bin

## serve: Start web server
serve:
	$(CARGO) run -- serve

## check: Run clippy, rustfmt and audit
check:
	$(CARGO) clippy && $(CARGO) fmt && $(CARGO) audit

## test: Launch unit tests in a single thread
test:
	$(CARGO) test -- --test-threads=1

## clean: Remove target directory
clean:
	$(CARGO) clean

## sqlx-prepare: Prepare for sqlx offline mode
sqlx-prepare:
	$(CARGO) sqlx prepare -- --bin $(CARGO_BIN_NAME)

## doc: Open Rust documentation without dependencies
doc:
	$(CARGO) doc --open --no-deps --document-private-items

## doc: Open Rust documentation with dependencies
doc-deps:
	$(CARGO) doc --open --document-private-items

## docker: Stop running containers, build docker-compose.yml file and run containers
docker: docker-down sqlx-prepare docker-up

## docker-up: Build docker-compose.yml file and run containers
docker-up:
	$(DOCKER_COMPOSE) up --build --force-recreate -d

## docker-up-no-daemon: Build docker-compose.yml file and run containers in non daemon mode
docker-up-no-daemon:
	$(DOCKER_COMPOSE) up --build --force-recreate

## docker-down: Stop running containers
docker-down:
	$(DOCKER_COMPOSE) down --remove-orphans

## docker-down-rm: Stop running containers and remove linked volumes
docker-down-rm:
	$(DOCKER_COMPOSE) down --remove-orphans --volumes

help: Makefile
	@echo
	@echo "Choose a command run in "$(APP_NAME)":"
	@echo
	@sed -n 's/^##//p' $< | column -t -s ':' | sed -e 's/^/ /'
	@echo