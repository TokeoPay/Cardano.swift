import Foundation

extension String {
    var policyIDData: Data? {
        let policyIDEndIndex = index(startIndex, offsetBy: 56)
        let rawPolicyID = String(self[startIndex..<policyIDEndIndex])
        let policyID = Data(hex: rawPolicyID)
        return policyID
    }

    var assetNameData: Data? {
        let assetNameStartIndex = index(startIndex, offsetBy: 56)
        let rawAssetName = String(self[assetNameStartIndex..<endIndex])
        let assetName = Data(hex: rawAssetName)
        return assetName
    }
}
