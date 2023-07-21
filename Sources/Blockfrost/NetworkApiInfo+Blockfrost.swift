//
//  NetworkApiInfo+Blockfrost.swift
//  
//
//  Created by Yehor Popovych on 17.11.2021.
//

import Foundation
import BlockfrostSwiftSDK
#if !COCOAPODS
import Cardano
#endif

public enum BlockfrostNetworkInfoError: Error {
    case unknownNetworkID(UInt8)
}

public extension NetworkApiInfo {
    func blockfrostConfig() throws -> BlockfrostConfig {
        switch self {
        case .mainnet:
            return BlockfrostConfig.mainnetDefault().clone()
        case .testnet:
            return BlockfrostConfig.testnetDefault().clone()
        case .preprod:
            return BlockfrostConfig(
                basePath: "https://cardano-preprod.blockfrost.io/api/v0",
                projectId: BlockfrostConfig.getEnvProjectId() ?? BlockfrostConfig.getEnvProjectIdTestnet()
            )
        default:
            throw BlockfrostNetworkInfoError.unknownNetworkID(self.networkID)
        }
    }
}
