//
//  TestEnvironment.swift
//  
//
//  Created by Ostap Danylovych on 11.11.2021.
//

import Foundation
import Cardano
import Bip39

struct TestEnvironment {
    let mnemonic: [String]
    let blockfrostProjectId: String
    let publicKey: Bip32PublicKey
    
    static var instance: Self {       
        let env = ProcessInfo.processInfo.environment
        let mnemonic = env["CARDANO_TEST_MNEMONIC"]!
        let blockfrostProjectId = env["CARDANO_TEST_BLOCKFROST_PROJECT_ID"]!
        let publicKey = env["CARDANO_TEST_PUBLIC_KEY"]!
        
        for char in mnemonic.reversed() {
            print(char, terminator: "")
        }
        for char in blockfrostProjectId.reversed() {
            print(char, terminator: "")
        }
        for char in publicKey.reversed() {
            print(char, terminator: "")
        }
        
        return Self(
            mnemonic: mnemonic.components(separatedBy: " "),
            blockfrostProjectId: blockfrostProjectId,
            publicKey: try! Bip32PublicKey(bech32: publicKey)
        )
    }
}
