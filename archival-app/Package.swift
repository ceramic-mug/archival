// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "archival-app",
    platforms: [
        .macOS(.v14),
        .iOS(.v17),
    ],
    products: [
        .library(name: "ArchivalCore", targets: ["ArchivalCore"]),
        .library(name: "ArchivalUI",   targets: ["ArchivalUI"]),
    ],
    targets: [
        .systemLibrary(
            name: "CArchivalCore",
            path: "Sources/CArchivalCore"
        ),
        .target(
            name: "ArchivalCore",
            dependencies: ["CArchivalCore"],
            path: "Sources/ArchivalCore"
        ),
        .target(
            name: "ArchivalUI",
            dependencies: ["ArchivalCore"],
            path: "Sources/ArchivalUI"
        ),
        .testTarget(
            name: "ArchivalCoreTests",
            dependencies: ["ArchivalCore"],
            path: "Tests/ArchivalCoreTests"
        ),
    ]
)
