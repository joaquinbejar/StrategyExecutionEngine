# Define variables
CARGO = cargo
DOCKER = docker

# Default target
.PHONY: all
all: build

# Build the project
.PHONY: build
build:
	$(CARGO) build

# Run the project
.PHONY: run
run:
	$(CARGO) run

# Test the project
.PHONY: test
test:
	$(CARGO) test

# Clean the project
.PHONY: clean
clean:
	$(CARGO) clean

# Format the code
.PHONY: fmt
fmt:
	$(CARGO) fmt

# Lint the code
.PHONY: clippy
clippy:
	$(CARGO) clippy

# Generate documentation
.PHONY: doc
doc:
	$(CARGO) doc --open

# Publish the crate to crates.io
.PHONY: publish
publish:
	$(CARGO) publish

# Build the Docker image
.PHONY: docker-build
docker-build:
	$(DOCKER) build -t strategy_execution_engine .

# Run the Docker container
.PHONY: docker-run
docker-run:
	$(DOCKER) run --rm -it strategy_execution_engine

# Stop the Docker container
.PHONY: docker-stop
docker-stop:
	$(DOCKER) stop strategy_execution_engine

# Push the Docker image to a registry (assuming you have logged in)
.PHONY: docker-push
docker-push:
	$(DOCKER) push strategy_execution_engine

# Check the status of the project (build, test, clippy, fmt)
.PHONY: check
check: build test clippy fmt