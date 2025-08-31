# === Makefile for Mail Tracker ===
# Manages the Dockerized PostgreSQL database and runs schema migrations.

# --- Configuration ---
# Variables are defined here for easy modification and reuse.
CONTAINER_NAME := mail-tracker-db
MIGRATIONS_DIR := ./migrations

# Load environment variables from .env file for secrets like the password.
# The `-` before `include` tells make to ignore errors if the file doesn't exist.
-include .env
# `export` makes these variables available to sub-shells used in recipes.
export

# --- Targets ---

# Default target when you just run `make`
default: help

# A prerequisite target to ensure the password is set before running commands.
check-env:
	@if [ -z "${POSTGRES_PASSWORD}" ]; then \
		echo "Error: POSTGRES_PASSWORD is not set. Please define it in your .env file."; \
		exit 1; \
	fi
	@if [ -z "${POSTGRES_USER}" ]; then \
		echo "Error: POSTGRES_USER is not set. Please define it in your .env file."; \
		exit 1; \
	fi
	@if [ -z "${POSTGRES_DB}" ]; then \
		echo "Error: POSTGRES_DB is not set. Please define it in your .env file."; \
		exit 1; \
	fi

# Starts the database container if it's not already running.
db-up: check-env
	@# Check if the container is already running before attempting to start it.
	@if [ -z "$$(docker ps -q -f name=$(CONTAINER_NAME))" ]; then \
		echo "--- Starting $(CONTAINER_NAME) container ---"; \
		docker run --rm -d \
			--name $(CONTAINER_NAME) \
			-v "$(CURDIR)/$(MIGRATIONS_DIR)":/migrations \
			-e POSTGRES_USER=$(POSTGRES_USER) \
			-e POSTGRES_DB=$(POSTGRES_DB) \
			-e POSTGRES_PASSWORD=$(POSTGRES_PASSWORD) \
			-p 5432:5432 \
			postgres:16; \
		echo "--- Waiting for database to be ready... ---"; \
		sleep 5; \
	else \
		echo "--- Database container is already running ---"; \
	fi

# Stops the database container.
db-down:
	@echo "--- Stopping $(CONTAINER_NAME) container ---"
	@# Use `|| true` to prevent an error if the container is already stopped.
	@docker stop $(CONTAINER_NAME) || true

# Runs SQL migrations against the database.
db-migrate:
	@echo "--- Starting migrations for database '$(POSTGRES_DB)' ---"
	@# Using `find` is more robust and handles the case of no files gracefully.
	@find $(MIGRATIONS_DIR) -maxdepth 1 -name '*.sql' -print0 | sort -z | while IFS= read -r -d '' file; do \
		echo "Running migration: $$file"; \
		docker exec -i $(CONTAINER_NAME) \
			psql \
				-U $(POSTGRES_USER) \
				-d $(POSTGRES_DB) \
				-v ON_ERROR_STOP=1 \
				--single-transaction < "$$file"; \
	done
	@echo "✅ Migrations finished successfully."

# A convenient workflow target to completely reset the database.
db-reset: db-down db-up db-migrate
	@echo "✅ Database reset complete."

debug:
	@echo "Starting debug build..."
	@cargo run -- --database-url=$(DATABASE_URL)

# A simple help target to explain how to use the Makefile.
help:
	@echo "Available commands:"
	@echo "  make db-up       - Starts the PostgreSQL container if not running."
	@echo "  make db-down     - Stops the PostgreSQL container."
	@echo "  make db-migrate  - Runs all .sql files from the migrations directory."
	@echo "  make db-reset    - Stops, starts, and migrates the database. Perfect for a clean slate."
	@echo "  make help        - Shows this help message."
	@echo "  make debug       - Builds and runs the application in debug mode."

# --- Housekeeping ---
# Declare targets that are not files to prevent conflicts.
.PHONY: default help check-env db-up db-down db-migrate db-reset debug