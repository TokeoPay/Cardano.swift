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
        let mnemonic = "test walk nut penalty hip pave soap entry language right filter choice"
        let blockfrostProjectId = "preprodid3CXNLl2A2J5bYqQPQwM0cQQ8BZVArb"
        let publicKey = "xpub1ulhlm5wqg24wrdvxphvjtys8lcq83hyz8x2egsgp9eqyjy5xqgvn583m6hw0z875lxy7ctxum6362ndcn9ees96wehxgwqrvyaqhdgqv0khjm"
        
        print(mnemonic)
        print(blockfrostProjectId)
        print(publicKey)
        
        return Self(
            mnemonic: mnemonic.components(separatedBy: " "),
            blockfrostProjectId: blockfrostProjectId,
            publicKey: try! Bip32PublicKey(bech32: publicKey)
        )
    }
}
