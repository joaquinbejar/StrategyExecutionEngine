# Define variables
CARGO = cargo
DOCKER = docker

# Load environment variables from .env file
include .env
export $(shell sed 's/=.*//' .env)

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
	$(CARGO) test --all-features

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


# Start Kafka and dependencies using Docker Compose
.PHONY: kafka-up
kafka-up:
	@echo "Starting Kafka with KAFKA_HOST=$(KAFKA_HOST)"
	@KAFKA_HOST=$(KAFKA_HOST) docker-compose -f Docker/kafka.yml up -d

# Stop Kafka and dependencies using Docker Compose
.PHONY: kafka-down
kafka-down:
	@KAFKA_HOST=$(KAFKA_HOST) docker-compose -f Docker/kafka.yml down

# Start Redis and dependencies using Docker Compose
.PHONY: redis-up
redis-up:
	@echo "Starting Redis with REDISPASS=$(REDISPASS)"
	@REDISPASS=$(REDISPASS) docker-compose -f Docker/redis.yml up -d

# Stop Redis and dependencies using Docker Compose
.PHONY: redis-down
redis-down:
	@REDISPASS=$(REDISPASS) docker-compose -f Docker/redis.yml down

# Start NATS and dependencies using Docker Compose
.PHONY: nats-up
nats-up:
	@docker-compose -f Docker/nats.yml up -d

# Stop NATS and dependencies using Docker Compose
.PHONY: nats-down
nats-down:
	@docker-compose -f Docker/nats.yml down

# Start RabbitMQ and dependencies using Docker Compose
.PHONY: rabbitmq-up
rabbitmq-up-docker:
	@echo $(RABBITMQ_ERLANG_COOKIE) > Docker/.erlang.cookie
	@chmod 600 Docker/.erlang.cookie
	@docker-compose -f Docker/rabbitmq.yml up -d

# Stop NATS and dependencies using Docker Compose
.PHONY: rabbitmq-down
rabbitmq-down:
	@docker-compose -f Docker/rabbitmq.yml down -v
	@rm -f Docker/.erlang.cookie

# Initialize the RabbitMQ cluster
rabbitmq-init-cluster:
	@echo "Waiting for RabbitMQ containers to be ready..."
	@sleep 30
	docker exec -it rabbitmq1 bash -c "rabbitmqctl stop_app && rabbitmqctl reset && rabbitmqctl start_app"
	docker exec -it rabbitmq2 bash -c "rabbitmqctl stop_app && rabbitmqctl reset && rabbitmqctl join_cluster rabbit@rabbitmq1 && rabbitmqctl start_app"
	docker exec -it rabbitmq3 bash -c "rabbitmqctl stop_app && rabbitmqctl reset && rabbitmqctl join_cluster rabbit@rabbitmq1 && rabbitmqctl start_app"

# Check the status of the RabbitMQ cluster
rabbitmq-status:
	docker exec -it rabbitmq1 rabbitmqctl cluster_status

# Start the RabbitMQ cluster
.PHONY: rabbitmq-up
rabbitmq-up: rabbitmq-up-docker rabbitmq-init-cluster rabbitmq-status