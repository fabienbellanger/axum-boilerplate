.PHONY: help \
	serve \
	watch \
	lint \
	lint-audit \
	test \
	clean \
	sqlx-prepare \
	doc \
	doc-deps \
	docker \
	docker-up \
	docker-up-no-daemon \
	docker-down \
	docker-down-rm \
	docker-cli-build \
	docker-cli-register

.DEFAULT_GOAL=help

# Parameters
APP_NAME="Axum Boilerplate"
CURRENT_PATH=$(shell pwd)
DOCKER_COMPOSE=docker-compose
DOCKER=docker
CARGO=cargo
CARGO_BIN_NAME=axum-boilerplate-bin

## serve: Start web server
serve:
	$(CARGO) run -- serve

## watch: Start web server with hot reload
watch:
	$(CARGO) watch -x "run -- serve"

## lint: Run clippy and rustfmt
lint:
	$(CARGO) clippy && $(CARGO) fmt

## lint-audit: Run clippy, rustfmt and audit
lint-audit: lint
	$(CARGO) audit

## test: Launch unit tests in a single thread
test:
	$(CARGO) test -- --test-threads=1 --nocapture

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

## docker-cli-build: Build project for CLI
docker-cli-build:
	$(DOCKER) build -f Dockerfile.cli -t axum-boilerplate-cli .

## docker-cli-register: Run CLI container to register an admin user
docker-cli-register: docker-cli-build
	$(DOCKER) run -i --rm --net axum-boilerplate_backend --link axum_boilerplate_mariadb axum-boilerplate-cli register -l Admin -f Admin -u admin@gmail.com -p 'K-qy,Kg{<AB*XX;V3}_/x19u>1BBl!d'

help: Makefile
	@echo
	@echo "Choose a command run in "$(APP_NAME)":"
	@echo
	@sed -n 's/^##//p' $< | column -t -s ':' | sed -e 's/^/ /'
	@echo
