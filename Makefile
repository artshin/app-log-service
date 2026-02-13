.PHONY: help build build-release run test check lint fmt clean \
        docker-build docker-run docker-stop docker-clean \
        test-log test-logs test-logs-burst clear-logs get-logs

BINARY_NAME = log-server
DOCKER_IMAGE = log-server-rust:latest

# Default target
help:
	@echo "Log Server (Rust)"
	@echo ""
	@echo "Development:"
	@echo "  make run              - Build and run server"
	@echo "  make run-verbose      - Run with verbose mode enabled"
	@echo "  make dev              - Run with cargo-watch (hot reload)"
	@echo ""
	@echo "Build:"
	@echo "  make build            - Build in debug mode (includes CSS)"
	@echo "  make build-release    - Build in release mode (optimized, includes CSS)"
	@echo "  make build-css        - Build Tailwind CSS only"
	@echo "  make watch-css        - Watch CSS for changes (development)"
	@echo "  make clean            - Clean build artifacts (including CSS)"
	@echo "  make clean-css        - Clean CSS artifacts only"
	@echo ""
	@echo "Code Quality:"
	@echo "  make test             - Run tests"
	@echo "  make check            - Run cargo check"
	@echo "  make lint             - Run clippy linter"
	@echo "  make fmt              - Format code with rustfmt"
	@echo "  make fmt-check        - Check formatting without modifying"
	@echo ""
	@echo "Docker:"
	@echo "  make docker-build     - Build Docker image"
	@echo "  make docker-run       - Run Docker container"
	@echo "  make docker-stop      - Stop running container"
	@echo "  make docker-clean     - Remove Docker image"
	@echo ""
	@echo "Testing (send logs to running server):"
	@echo "  make test-log         - Send a single test log entry"
	@echo "  make test-logs        - Send logs at all levels"
	@echo "  make test-logs-burst  - Send 50 logs for stress testing"
	@echo "  make clear-logs       - Clear all logs from server"
	@echo "  make get-logs         - Get all logs as JSON"
	@echo ""
	@echo "Environment Variables:"
	@echo "  PORT                  - Server port (default: 8081)"
	@echo "  CAPACITY              - Buffer capacity (default: 1000)"
	@echo "  VERBOSE               - Show metadata (1/true to enable)"
	@echo "  LOG_SERVER_URL        - Target server for test commands (default: http://localhost:9006)"

# ============================================================================
# Development
# ============================================================================

# Build and run
run: build
	@echo "Running $(BINARY_NAME)..."
	@./target/debug/$(BINARY_NAME)

# Run with verbose mode
run-verbose: build
	@echo "Running $(BINARY_NAME) with verbose mode..."
	@VERBOSE=1 ./target/debug/$(BINARY_NAME)

# Run with hot reload (requires cargo-watch)
dev:
	@if command -v cargo-watch > /dev/null 2>&1; then \
		echo "Running with hot reload..."; \
		cargo watch -x run; \
	else \
		echo "cargo-watch not found. Install with: cargo install cargo-watch"; \
		echo "Running without hot reload..."; \
		cargo run; \
	fi

# ============================================================================
# Build
# ============================================================================

# Build in debug mode
build: build-css
	@echo "Building in debug mode..."
	@cargo build

# Build in release mode
build-release: build-css
	@echo "Building in release mode..."
	@cargo build --release
	@echo ""
	@echo "Binary: target/release/$(BINARY_NAME)"
	@ls -lh target/release/$(BINARY_NAME)

# Build CSS with Tailwind
build-css:
	@if command -v npm > /dev/null 2>&1; then \
		echo "Building Tailwind CSS..."; \
		npm run build:css 2>/dev/null || (npm install && npm run build:css); \
	else \
		echo "npm not found. Skipping CSS build (using pre-built CSS if available)"; \
	fi

# Watch CSS for changes (development)
watch-css:
	@if command -v npm > /dev/null 2>&1; then \
		echo "Watching Tailwind CSS for changes..."; \
		npm install 2>/dev/null || true; \
		npm run watch:css; \
	else \
		echo "npm not found. Install Node.js to use CSS watch mode"; \
		exit 1; \
	fi

# Clean build artifacts
clean: clean-css
	@echo "Cleaning build artifacts..."
	@cargo clean

# Clean CSS build artifacts
clean-css:
	@echo "Cleaning CSS artifacts..."
	@rm -rf node_modules static/css/output.css

# ============================================================================
# Code Quality
# ============================================================================

# Run tests
test:
	@echo "Running tests..."
	@cargo test

# Run tests with output
test-verbose:
	@echo "Running tests (verbose)..."
	@cargo test -- --nocapture

# Run cargo check
check:
	@echo "Running cargo check..."
	@cargo check

# Run clippy linter
lint:
	@echo "Running clippy..."
	@cargo clippy -- -D warnings

# Format code
fmt:
	@echo "Formatting code..."
	@cargo fmt

# Check formatting
fmt-check:
	@echo "Checking formatting..."
	@cargo fmt -- --check

# Run all quality checks
quality: fmt-check lint test
	@echo ""
	@echo "All quality checks passed!"

# ============================================================================
# Docker
# ============================================================================

# Build Docker image
docker-build:
	@echo "Building Docker image..."
	@docker build -t $(DOCKER_IMAGE) .

# Run Docker container
docker-run:
	@echo "Running $(DOCKER_IMAGE)..."
	@docker run -it --rm -p 9006:9006 $(DOCKER_IMAGE)

# Run Docker container with verbose mode
docker-run-verbose:
	@echo "Running $(DOCKER_IMAGE) with verbose mode..."
	@docker run -it --rm -p 9006:9006 -e VERBOSE=1 $(DOCKER_IMAGE)

# Stop running Docker container
docker-stop:
	@echo "Stopping $(DOCKER_IMAGE) container..."
	@-docker stop $$(docker ps -q --filter ancestor=$(DOCKER_IMAGE)) 2>/dev/null || true

# Remove Docker image
docker-clean:
	@echo "Removing Docker image $(DOCKER_IMAGE)..."
	@-docker rmi $(DOCKER_IMAGE) 2>/dev/null || true

# ============================================================================
# Testing
# ============================================================================

LOG_SERVER_URL ?= http://localhost:9006

# Send a single test log entry
test-log:
	@echo "Sending test log to $(LOG_SERVER_URL)..."
	@curl -s -X POST $(LOG_SERVER_URL)/logs \
		-H "Content-Type: application/json" \
		-d '{"id":"test-'$$(date +%s)'","timestamp":"'$$(date -u +%Y-%m-%dT%H:%M:%SZ)'","level":"info","message":"Test log from Makefile","source":"makefile","file":"Makefile","function":"test-log","line":1}' \
		&& echo "Log sent successfully" || echo "Failed to send log"

# Send multiple test logs at different levels
test-logs:
	@echo "Sending test logs to $(LOG_SERVER_URL)..."
	@for level in trace debug info notice warning error critical; do \
		curl -s -X POST $(LOG_SERVER_URL)/logs \
			-H "Content-Type: application/json" \
			-d "{\"id\":\"test-$$level-$$(date +%s%N)\",\"timestamp\":\"$$(date -u +%Y-%m-%dT%H:%M:%SZ)\",\"level\":\"$$level\",\"message\":\"Test $$level message from Makefile\",\"source\":\"makefile\",\"file\":\"Makefile\",\"function\":\"test-logs\",\"line\":1}" \
			&& echo "  ✓ Sent $$level log" || echo "  ✗ Failed to send $$level log"; \
		sleep 0.1; \
	done
	@echo "Done!"

# Send a burst of logs for stress testing
test-logs-burst:
	@echo "Sending burst of 50 logs to $(LOG_SERVER_URL)..."
	@for i in $$(seq 1 50); do \
		level=$$(echo "trace debug info notice warning error critical" | tr ' ' '\n' | shuf -n1); \
		curl -s -X POST $(LOG_SERVER_URL)/logs \
			-H "Content-Type: application/json" \
			-d "{\"id\":\"burst-$$i-$$(date +%s%N)\",\"timestamp\":\"$$(date -u +%Y-%m-%dT%H:%M:%SZ)\",\"level\":\"$$level\",\"message\":\"Burst test message #$$i ($$level)\",\"source\":\"makefile\",\"file\":\"Makefile\",\"function\":\"test-logs-burst\",\"line\":$$i}" \
			>/dev/null; \
	done
	@echo "Sent 50 log entries"

# Clear all logs from the server
clear-logs:
	@echo "Clearing logs from $(LOG_SERVER_URL)..."
	@curl -s -X DELETE $(LOG_SERVER_URL)/logs && echo "Logs cleared" || echo "Failed to clear logs"

# Get all logs from the server
get-logs:
	@curl -s $(LOG_SERVER_URL)/logs | jq . 2>/dev/null || curl -s $(LOG_SERVER_URL)/logs
