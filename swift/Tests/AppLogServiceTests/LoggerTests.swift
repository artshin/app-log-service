import XCTest
@testable import AppLogService

/// A mock transport that captures sent entries for verification.
final class MockTransport: LogTransport, @unchecked Sendable {
    var sentEntries: [LogEntry] = []
    private let lock = NSLock()
    var onSend: (([LogEntry]) -> Void)?

    func send(_ entries: [LogEntry]) async throws {
        lock.lock()
        sentEntries.append(contentsOf: entries)
        let callback = onSend
        lock.unlock()
        callback?(entries)
    }

    var entries: [LogEntry] {
        lock.lock()
        defer { lock.unlock() }
        return sentEntries
    }
}

final class LoggerTests: XCTestCase {
    func testLogLevelFiltering() throws {
        let transport = MockTransport()
        let expectation = expectation(description: "logs flushed")

        transport.onSend = { _ in
            expectation.fulfill()
        }

        let logger = Logger.shared
        let config = LoggerConfiguration(
            serverURL: URL(string: "http://localhost:9006")!,
            source: "test",
            minimumLevel: .warning,
            batchSize: 1
        )
        logger.configure(config, transport: transport)

        // This should be filtered out (below warning)
        logger.info("Should be filtered")

        // This should pass through
        logger.error("Should pass")

        waitForExpectations(timeout: 2)

        let entries = transport.entries
        XCTAssertEqual(entries.count, 1)
        XCTAssertEqual(entries.first?.level, .error)
        XCTAssertEqual(entries.first?.message, "Should pass")
    }

    func testLogLevelComparison() {
        XCTAssertTrue(LogLevel.trace < LogLevel.debug)
        XCTAssertTrue(LogLevel.debug < LogLevel.info)
        XCTAssertTrue(LogLevel.info < LogLevel.notice)
        XCTAssertTrue(LogLevel.notice < LogLevel.warning)
        XCTAssertTrue(LogLevel.warning < LogLevel.error)
        XCTAssertTrue(LogLevel.error < LogLevel.critical)
    }

    func testDefaultTagsAreApplied() {
        let transport = MockTransport()
        let expectation = expectation(description: "logs flushed")

        transport.onSend = { _ in
            expectation.fulfill()
        }

        let logger = Logger.shared
        let config = LoggerConfiguration(
            serverURL: URL(string: "http://localhost:9006")!,
            source: "test",
            defaultTags: ["app", "v1"],
            minimumLevel: .trace,
            batchSize: 1
        )
        logger.configure(config, transport: transport)

        logger.info("Tagged message", tags: ["extra"])

        waitForExpectations(timeout: 2)

        let entries = transport.entries
        guard let entry = entries.last else {
            XCTFail("No entries sent")
            return
        }
        XCTAssertTrue(entry.tags.contains("app"))
        XCTAssertTrue(entry.tags.contains("v1"))
        XCTAssertTrue(entry.tags.contains("extra"))
    }

    func testConvenienceMethodsSendCorrectLevel() {
        let transport = MockTransport()
        let expectation = expectation(description: "all logs flushed")
        expectation.expectedFulfillmentCount = 7

        transport.onSend = { _ in
            expectation.fulfill()
        }

        let logger = Logger.shared
        let config = LoggerConfiguration(
            serverURL: URL(string: "http://localhost:9006")!,
            source: "test",
            minimumLevel: .trace,
            batchSize: 1
        )
        logger.configure(config, transport: transport)

        logger.trace("t")
        logger.debug("d")
        logger.info("i")
        logger.notice("n")
        logger.warning("w")
        logger.error("e")
        logger.critical("c")

        waitForExpectations(timeout: 3)

        let levels = transport.entries.suffix(7).map(\.level)
        XCTAssertTrue(levels.contains(.trace))
        XCTAssertTrue(levels.contains(.debug))
        XCTAssertTrue(levels.contains(.info))
        XCTAssertTrue(levels.contains(.notice))
        XCTAssertTrue(levels.contains(.warning))
        XCTAssertTrue(levels.contains(.error))
        XCTAssertTrue(levels.contains(.critical))
    }
}
