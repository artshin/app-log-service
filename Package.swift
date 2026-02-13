// swift-tools-version: 5.9

import PackageDescription

let package = Package(
    name: "AppLogService",
    platforms: [
        .iOS(.v16),
        .macOS(.v13),
    ],
    products: [
        .library(
            name: "AppLogService",
            targets: ["AppLogService"]
        ),
    ],
    targets: [
        .target(
            name: "AppLogService",
            path: "swift/Sources/AppLogService"
        ),
        .testTarget(
            name: "AppLogServiceTests",
            dependencies: ["AppLogService"],
            path: "swift/Tests/AppLogServiceTests"
        ),
    ]
)
