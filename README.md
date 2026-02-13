# Log Server (Rust)

A development logging service for the Calories Tracker application. Receives log entries from Swift clients (CLI and iOS), stores them in a circular buffer, and displays them in the terminal with ANSI color coding.

## Features

- HTTP REST API for log submission and retrieval
- Thread-safe circular buffer (1000 entries by default)
- ANSI color-coded terminal output
- Graceful shutdown on SIGINT/SIGTERM
- Verbose mode for metadata display

## Quick Start

```bash
# Build and run
make run

# Run with verbose mode (shows file/line info)
make run-verbose

# Run with hot reload (development)
make dev
```

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/` | Welcome page with documentation |
| POST | `/logs` | Submit a log entry |
| GET | `/logs` | Retrieve all logs (JSON) |
| DELETE | `/logs` | Clear all logs |

## Log Entry Format

```json
{
  "id": "unique-id",
  "timestamp": "2024-01-15T10:30:00Z",
  "level": "info",
  "message": "Log message here",
  "source": "cli",
  "metadata": {"key": "value"},
  "file": "main.swift",
  "function": "main()",
  "line": 42
}
```

## Log Levels

| Level | Color |
|-------|-------|
| trace | gray |
| debug | gray |
| info | green |
| notice | blue |
| warning | yellow |
| error | red |
| critical | magenta |

## Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `PORT` | 8081 | Server port |
| `CAPACITY` | 1000 | Buffer capacity |
| `VERBOSE` | false | Show metadata |

## Docker

```bash
# Build image
make docker-build

# Run container
make docker-run

# Run with verbose mode
make docker-run-verbose
```

## Development

```bash
# Run tests
make test

# Check code
make check

# Lint with clippy
make lint

# Format code
make fmt

# Run all quality checks
make quality
```

## Architecture

- `main.rs` - Entry point, server setup, graceful shutdown
- `config.rs` - Environment variable configuration
- `models.rs` - LogEntry and LogLevel types
- `buffer.rs` - Thread-safe circular buffer
- `handlers.rs` - Axum HTTP handlers
- `display.rs` - Terminal output with colors
