import XCTest
@testable import AppLogService

final class LogEntryTests: XCTestCase {
    func testJSONEncodingMatchesServerFormat() throws {
        let date = ISO8601DateFormatter().date(from: "2024-01-15T10:30:00Z")!
        let entry = LogEntry(
            id: "test-123",
            timestamp: date,
            level: .info,
            message: "Test message",
            deviceId: "device-uuid-123",
            source: "ios",
            metadata: ["key": "value"],
            tags: ["network"],
            file: "main.swift",
            function: "main()",
            line: 42
        )

        let data = try JSONEncoder().encode(entry)
        let json = try JSONSerialization.jsonObject(with: data) as! [String: Any]

        XCTAssertEqual(json["id"] as? String, "test-123")
        XCTAssertEqual(json["level"] as? String, "info")
        XCTAssertEqual(json["message"] as? String, "Test message")
        XCTAssertEqual(json["deviceId"] as? String, "device-uuid-123")
        XCTAssertEqual(json["source"] as? String, "ios")
        XCTAssertEqual(json["file"] as? String, "main.swift")
        XCTAssertEqual(json["function"] as? String, "main()")
        XCTAssertEqual(json["line"] as? Int, 42)
        XCTAssertEqual(json["tags"] as? [String], ["network"])

        let metadata = json["metadata"] as? [String: String]
        XCTAssertEqual(metadata?["key"], "value")

        // Verify timestamp is an ISO8601 string
        let timestamp = json["timestamp"] as? String
        XCTAssertNotNil(timestamp)
        XCTAssertTrue(timestamp!.contains("2024-01-15"))
    }

    func testJSONEncodingOmitsEmptyMetadata() throws {
        let entry = LogEntry(
            level: .debug,
            message: "No metadata",
            deviceId: "device-1",
            source: "ios"
        )

        let data = try JSONEncoder().encode(entry)
        let json = try JSONSerialization.jsonObject(with: data) as! [String: Any]

        // metadata should be omitted when empty
        XCTAssertNil(json["metadata"])
    }

    func testJSONEncodingUseCamelCaseKeys() throws {
        let entry = LogEntry(
            level: .warning,
            message: "Key check",
            deviceId: "dev-1",
            source: "macos"
        )

        let data = try JSONEncoder().encode(entry)
        let jsonString = String(data: data, encoding: .utf8)!

        XCTAssertTrue(jsonString.contains("\"deviceId\""))
        XCTAssertFalse(jsonString.contains("\"device_id\""))
    }
}
