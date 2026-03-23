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
        .executableTarget(
            name: "LuandaEditor",
            //dependencies: ["SwiftUI"],
            path: "src/editor/swiftui",
            resources: [
                // Add resources here if needed
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
