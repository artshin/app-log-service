import Foundation

/// A log entry matching the Rust server's expected JSON format.
public struct LogEntry: Encodable, Sendable {
    public let id: String
    public let timestamp: Date
    public let level: LogLevel
    public let message: String
    public let deviceId: String
    public let source: String
    public let metadata: [String: String]
    public let tags: [String]
    public let file: String
    public let function: String
    public let line: UInt

    public init(
        id: String = UUID().uuidString,
        timestamp: Date = Date(),
        level: LogLevel,
        message: String,
        deviceId: String,
        source: String,
        metadata: [String: String] = [:],
        tags: [String] = [],
        file: String = "",
        function: String = "",
        line: UInt = 0
    ) {
        self.id = id
        self.timestamp = timestamp
        self.level = level
        self.message = message
        self.deviceId = deviceId
        self.source = source
        self.metadata = metadata
        self.tags = tags
        self.file = file
        self.function = function
        self.line = line
    }

    enum CodingKeys: String, CodingKey {
        case id, timestamp, level, message, deviceId, source, metadata, tags, file, function, line
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        try container.encode(id, forKey: .id)
        try container.encode(Self.iso8601Formatter.string(from: timestamp), forKey: .timestamp)
        try container.encode(level, forKey: .level)
        try container.encode(message, forKey: .message)
        try container.encode(deviceId, forKey: .deviceId)
        try container.encode(source, forKey: .source)
        if !metadata.isEmpty {
            try container.encode(metadata, forKey: .metadata)
        }
        try container.encode(tags, forKey: .tags)
        try container.encode(file, forKey: .file)
        try container.encode(function, forKey: .function)
        try container.encode(line, forKey: .line)
    }

    private static let iso8601Formatter: ISO8601DateFormatter = {
        let formatter = ISO8601DateFormatter()
        formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
        return formatter
    }()
}
