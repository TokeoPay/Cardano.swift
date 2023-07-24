import BigInt
import XCTest
@testable import Cardano

final class StringCardanoExtensionTests: XCTestCase {
    func testPolicyIDFromTokenAddress() throws {
        let tokenAddress = "f6f49b186751e61f1fb8c64e7504e771f968cea9f4d11f5222b169e374574d54"
        let policyIDData = tokenAddress.policyIDData
        let expectedData = Data(hex: "f6f49b186751e61f1fb8c64e7504e771f968cea9f4d11f5222b169e3")
        XCTAssertEqual(policyIDData, expectedData)
    }

    func testAssetNameFromTokenAddress() throws {
        let tokenAddress = "f6f49b186751e61f1fb8c64e7504e771f968cea9f4d11f5222b169e374574d54"
        let assetNameData = tokenAddress.assetNameData
        let expectedData = Data(hex: "74574d54")
        XCTAssertEqual(assetNameData, expectedData)

        let name = String(data: assetNameData!, encoding: .utf8)
        XCTAssertEqual(name, "tWMT")
    }
}
