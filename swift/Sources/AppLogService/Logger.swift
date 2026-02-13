import Foundation

/// Configuration for the Logger.
public struct LoggerConfiguration: Sendable {
    /// Server URL for the log server.
    public var serverURL: URL
    /// Source identifier (e.g. "ios", "macos").
    public var source: String
    /// Default tags applied to every log entry.
    public var defaultTags: [String]
    /// Minimum log level. Entries below this level are discarded.
    public var minimumLevel: LogLevel
    /// Maximum number of entries to batch before flushing.
    public var batchSize: Int
    /// Maximum time interval (seconds) before flushing a partial batch.
    public var flushInterval: TimeInterval

    public init(
        serverURL: URL,
        source: String = "ios",
        defaultTags: [String] = [],
        minimumLevel: LogLevel = .trace,
        batchSize: Int = 10,
        flushInterval: TimeInterval = 5.0
    ) {
        self.serverURL = serverURL
        self.source = source
        self.defaultTags = defaultTags
        self.minimumLevel = minimumLevel
        self.batchSize = batchSize
        self.flushInterval = flushInterval
    }
}

/// Main logging interface. Queues log entries and sends them in batches.
public final class Logger: @unchecked Sendable {
    /// Shared singleton instance. Must be configured before use.
    public static let shared = Logger()

    private var transport: LogTransport?
    private var configuration: LoggerConfiguration?
    private let queue = DispatchQueue(label: "AppLogService.Logger", attributes: .concurrent)
    private var buffer: [LogEntry] = []
    private var flushTimer: DispatchSourceTimer?

    private init() {}

    /// Configure the shared logger.
    public func configure(_ configuration: LoggerConfiguration) {
        queue.async(flags: .barrier) { [self] in
            self.configuration = configuration
            self.transport = HTTPLogTransport(serverURL: configuration.serverURL)
            self.startFlushTimer()
        }
    }

    /// Configure with a custom transport (useful for testing).
    public func configure(_ configuration: LoggerConfiguration, transport: LogTransport) {
        queue.async(flags: .barrier) { [self] in
            self.configuration = configuration
            self.transport = transport
            self.startFlushTimer()
        }
    }

    // MARK: - Convenience Methods

    public func trace(
        _ message: String,
        metadata: [String: String] = [:],
        tags: [String] = [],
        file: String = #file,
        function: String = #function,
        line: UInt = #line
    ) {
        log(.trace, message, metadata: metadata, tags: tags, file: file, function: function, line: line)
    }

    public func debug(
        _ message: String,
        metadata: [String: String] = [:],
        tags: [String] = [],
        file: String = #file,
        function: String = #function,
        line: UInt = #line
    ) {
        log(.debug, message, metadata: metadata, tags: tags, file: file, function: function, line: line)
    }

    public func info(
        _ message: String,
        metadata: [String: String] = [:],
        tags: [String] = [],
        file: String = #file,
        function: String = #function,
        line: UInt = #line
    ) {
        log(.info, message, metadata: metadata, tags: tags, file: file, function: function, line: line)
    }

    public func notice(
        _ message: String,
        metadata: [String: String] = [:],
        tags: [String] = [],
        file: String = #file,
        function: String = #function,
        line: UInt = #line
    ) {
        log(.notice, message, metadata: metadata, tags: tags, file: file, function: function, line: line)
    }

    public func warning(
        _ message: String,
        metadata: [String: String] = [:],
        tags: [String] = [],
        file: String = #file,
        function: String = #function,
        line: UInt = #line
    ) {
        log(.warning, message, metadata: metadata, tags: tags, file: file, function: function, line: line)
    }

    public func error(
        _ message: String,
        metadata: [String: String] = [:],
        tags: [String] = [],
        file: String = #file,
        function: String = #function,
        line: UInt = #line
    ) {
        log(.error, message, metadata: metadata, tags: tags, file: file, function: function, line: line)
    }

    public func critical(
        _ message: String,
        metadata: [String: String] = [:],
        tags: [String] = [],
        file: String = #file,
        function: String = #function,
        line: UInt = #line
    ) {
        log(.critical, message, metadata: metadata, tags: tags, file: file, function: function, line: line)
    }

    // MARK: - Core

    /// Log a message at the specified level.
    public func log(
        _ level: LogLevel,
        _ message: String,
        metadata: [String: String] = [:],
        tags: [String] = [],
        file: String = #file,
        function: String = #function,
        line: UInt = #line
    ) {
        queue.async(flags: .barrier) { [self] in
            guard let config = self.configuration else { return }
            guard level >= config.minimumLevel else { return }

            let entry = LogEntry(
                level: level,
                message: message,
                deviceId: DeviceIdentifier.id,
                source: config.source,
                metadata: metadata,
                tags: config.defaultTags + tags,
                file: file,
                function: function,
                line: line
            )

            self.buffer.append(entry)

            if self.buffer.count >= config.batchSize {
                self.flushBuffer()
            }
        }
    }

    /// Immediately flush any buffered log entries.
    public func flush() {
        queue.async(flags: .barrier) { [self] in
            self.flushBuffer()
        }
    }

    // MARK: - Private

    private func flushBuffer() {
        // Must be called on the barrier queue
        guard !buffer.isEmpty, let transport else { return }
        let entries = buffer
        buffer.removeAll()

        Task.detached {
            do {
                try await transport.send(entries)
            } catch {
                // Fire-and-forget: log delivery failure is non-fatal
            }
        }
    }

    private func startFlushTimer() {
        flushTimer?.cancel()
        guard let config = configuration else { return }
        let timer = DispatchSource.makeTimerSource(queue: queue)
        timer.schedule(
            deadline: .now() + config.flushInterval,
            repeating: config.flushInterval
        )
        timer.setEventHandler { [weak self] in
            self?.flushBuffer()
        }
        timer.resume()
        flushTimer = timer
    }
}
