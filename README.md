# AppLogService

A logging service for Apple platform apps. Contains a **Rust HTTP server** for receiving and displaying logs, and a **Swift Package** client SDK for sending them.

## Repository Structure

```
├── server/       # Rust log server
└── swift/        # Swift client SDK (SPM package)
```

## Server

The Rust log server receives, stores, and displays log entries with a web dashboard and terminal output.

```bash
cd server
make run        # Build and run
make dev        # Run with hot reload
make test       # Run tests
```

### API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/` | HTML dashboard |
| POST | `/logs` | Submit a log entry |
| GET | `/logs` | Retrieve all logs (JSON) |
| DELETE | `/logs` | Clear all logs |
| GET | `/stream` | SSE real-time log stream |

### Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `PORT` | 9006 | Server port |
| `CAPACITY` | 1000 | Buffer capacity |
| `VERBOSE` | false | Show metadata in terminal |

### Docker

```bash
cd server
make docker-build
make docker-run
```

## Swift SDK

The client SDK lives in `swift/`. Add it as a local Swift Package dependency or point SPM at the `swift/` directory.

### Usage

```swift
import AppLogService

// Configure once at app startup
Logger.shared.configure(LoggerConfiguration(
    serverURL: URL(string: "http://localhost:9006")!,
    source: "ios",
    defaultTags: ["myapp"],
    minimumLevel: .debug
))

// Log at any level
Logger.shared.info("User signed in", metadata: ["userId": "123"])
Logger.shared.error("Network request failed", tags: ["network"])
Logger.shared.debug("Cache hit for key")
```

File, function, and line number are captured automatically. Logs are batched and sent in the background.

### Log Levels

`trace` | `debug` | `info` | `notice` | `warning` | `error` | `critical`
