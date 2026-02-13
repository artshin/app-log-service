import Foundation

/// Log severity levels, mirroring the Rust server's LogLevel enum.
public enum LogLevel: String, Codable, Comparable, Sendable {
    case trace
    case debug
    case info
    case notice
    case warning
    case error
    case critical

    private var severity: Int {
        switch self {
        case .trace: return 0
        case .debug: return 1
        case .info: return 2
        case .notice: return 3
        case .warning: return 4
        case .error: return 5
        case .critical: return 6
        }
    }

    public static func < (lhs: LogLevel, rhs: LogLevel) -> Bool {
        lhs.severity < rhs.severity
    }
}
