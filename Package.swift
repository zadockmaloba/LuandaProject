// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "LuandaEditor",
    platforms: [
        .macOS(.v14)
    ],
    products: [
        .executable(
            name: "LuandaEditor",
            targets: ["LuandaEditor"]
        )
    ],
    dependencies: [
        // Add your Swift package dependencies here
    ],
    targets: [
        .target(
            name: "LuandaBridge",
            path: "src/editor/swiftui",
            publicHeadersPath: "include"
        ),
        .executableTarget(
            name: "LuandaEditor",
            dependencies: ["LuandaBridge"],
            path: "src/editor/swiftui",
            resources: [
                // Add resources here if needed
            ],
            swiftSettings: [
                .unsafeFlags(["-Xlinker", "-debug_dylib"])
            ],
            linkerSettings: [
                .linkedFramework("Foundation"),
                .linkedFramework("SwiftUI"),
                .linkedFramework("Metal"),
                .linkedFramework("Symbols")
            ]
        )
    ]
)
