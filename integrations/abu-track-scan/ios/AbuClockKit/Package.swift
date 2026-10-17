// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "AbuClockKit",
    platforms: [
        .iOS(.v16),
        .macOS(.v13)
    ],
    products: [
        .library(
            name: "AbuClockKit",
            targets: ["AbuClockKit"]
        )
    ],
    targets: [
        .target(
            name: "AbuClockKit",
            path: "Sources/AbuClockKit"
        )
    ]
)
