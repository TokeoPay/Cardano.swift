//
//  AddressManagerTests.swift
//  
//
//  Created by Ostap Danylovych on 06.11.2021.
//

import Foundation
import XCTest
@testable import Cardano
import BlockfrostSwiftSDK
import Bip39
#if !COCOAPODS
@testable import CardanoBlockfrost
#endif


final class AddressManagerTests: XCTestCase {
    private let signatureProvider = SignatureProviderMock()
    
    func testFetchOnTestnet() throws {
        let fetchSuccessful = expectation(description: "Fetch successful")
        let cardano = try Cardano(
            blockfrost: TestEnvironment.instance.blockfrostProjectId,
            info: .preprod,
            signer: signatureProvider
        )
        let account = Account(publicKey: TestEnvironment.instance.publicKey, index: 0)
        var testAddresses = (0..<20).map {
            try! account.baseAddress(index: $0,
                                     change: false,
                                     networkID: cardano.info.networkID).address
        }
        let changeAddresses = (0..<20).map {
            try! account.baseAddress(index: $0,
                                     change: true,
                                     networkID: cardano.info.networkID).address
        }
        let enterpriseAddress = try! account.paymentAddress(networkID: cardano.info.networkID)
        testAddresses.append(contentsOf: changeAddresses)
        testAddresses.append(enterpriseAddress)
        cardano.addresses.fetch(for: [account]) { res in
            try! res.get()
            let addresses = try! cardano.addresses.get(cached: account)

            print("BF Addresses", addresses)
            // As this test is retrieving real information from the preprod Cardano
            // environment we can only rely on that certain addresses are
            // available in the moment of the test creation
            XCTAssertTrue(addresses.contains(where: {
                try! $0.bech32() == "addr_test1vr25g4r29lqz2aw6tcxe86p9r60ppsgd0wd206cuv7qdq8sv57qmk"
            }))
//            XCTAssertTrue(addresses.contains(where: {
//                try! $0.bech32() == "addr_test1qq4es8z3wf5f8tpv0623xk856e9uxeqdyv35pe5qjwhc6nz7wc409zgwcydgfjt06d2cmls6xz3gggq66yamyjd56mzqscqer4"
//            }))
//            XCTAssertTrue(addresses.contains(where: {
//                try! $0.bech32() == "addr_test1qq2l32uf7g34e7akuzy0l3fmczjww9vx3humnwzeuzrlw927wc409zgwcydgfjt06d2cmls6xz3gggq66yamyjd56mzq3td97a"
//            }))
            fetchSuccessful.fulfill()
        }
        wait(for: [fetchSuccessful], timeout: 100)
    }
}
