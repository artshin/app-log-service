import Foundation

/// Protocol for delivering log entries to a backend.
public protocol LogTransport: Sendable {
    func send(_ entries: [LogEntry]) async throws
}

/// HTTP transport that POSTs log entries to the server's `/logs` endpoint.
public final class HTTPLogTransport: LogTransport, @unchecked Sendable {
    private let serverURL: URL
    private let session: URLSession

    public init(serverURL: URL, session: URLSession = .shared) {
        self.serverURL = serverURL
        self.session = session
    }

    public func send(_ entries: [LogEntry]) async throws {
        let endpoint = serverURL.appendingPathComponent("logs")
        let encoder = JSONEncoder()

        for entry in entries {
            var request = URLRequest(url: endpoint)
            request.httpMethod = "POST"
            request.setValue("application/json", forHTTPHeaderField: "Content-Type")
            request.httpBody = try encoder.encode(entry)
            let (_, response) = try await session.data(for: request)
            guard let http = response as? HTTPURLResponse,
                  (200...299).contains(http.statusCode) else {
                let code = (response as? HTTPURLResponse)?.statusCode ?? -1
                throw TransportError.serverError(statusCode: code)
            }
        }
    }
}

/// Errors from log transport.
public enum TransportError: Error, Sendable {
    case serverError(statusCode: Int)
}
