// swift-tools-version:5.4
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let useLocalBinary = false

var package = Package(
    name: "Cardano",
    platforms: [.iOS(.v13), .macOS(.v10_15)],
    products: [
        .library(
            name: "Cardano",
            targets: ["Cardano"]),
        .library(
            name: "CardanoCore",
            targets: ["CardanoCore"])
    ],
    dependencies: [
        .package(url: "https://github.com/attaswift/BigInt.git", from: "5.2.1"),
        .package(name: "Bip39", url: "https://github.com/tesseract-one/Bip39.swift.git", from: "0.1.1"),
        .package(url: "https://github.com/apple/swift-collections", from: "1.0.2")
    ],
    targets: [
        .target(
            name: "Cardano",
            dependencies: ["CardanoCore", "Bip39"]),
        .target(
            name: "CardanoCore",
            dependencies: [
                "CCardano",
                "BigInt",
                .product(name: "OrderedCollections", package: "swift-collections")
            ],
            path: "Sources/Core",
            swiftSettings: [.unsafeFlags(["-Onone"])]),
        .testTarget(
            name: "CoreTests",
            dependencies: ["CardanoCore"]
        ),
        .testTarget(
            name: "CardanoTests",
            dependencies: ["Cardano"])
    ]
)

#if os(Linux)
package.targets.append(
    .systemLibrary(name: "CCardano")
)
#else
let ccardano: Target = useLocalBinary ?
    .binaryTarget(
        name: "CCardano",
        path: "rust/binaries/CCardano.xcframework") :
    .binaryTarget(
        name: "CCardano",
        url: "https://pub-5314ba2a19a94f41912a726140440b24.r2.dev/CCardano.binaries.9a5a684.zip",
        checksum: "9a5a684a07d79b9db1791aa07b10c98b05669992cf3e88abe21a5afbec914974")
package.targets.append(contentsOf: [
    ccardano,
    .target(
        name: "CardanoBlockfrost",
        dependencies: ["Cardano", "BlockfrostSwiftSDK"],
        path: "Sources/Blockfrost"),
    .testTarget(
        name: "BlockfrostTests",
        dependencies: ["CardanoBlockfrost"])
])
package.products.append(
    .library(
        name: "CardanoBlockfrost",
        targets: ["CardanoBlockfrost"])
)
package.dependencies.append(
    .package(name: "BlockfrostSwiftSDK", url: "https://github.com/blockfrost/blockfrost-swift.git", from: "1.0.1")
)
#endif
