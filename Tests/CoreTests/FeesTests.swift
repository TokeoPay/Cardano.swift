//
//  FeesTests.swift
//  
//
//  Created by Ostap Danylovych on 29.07.2021.
//

import Foundation
import XCTest
#if !COCOAPODS
@testable import CardanoCore
#else
@testable import Cardano
#endif

final class FeesTests: XCTestCase {
    let initialize: Void = _initialize
    
    func testTxSimpleUtxo() throws {
        let inputs = [
            TransactionInput(
                transaction_id: try TransactionHash(
                    bytes: Data(hex: "3b40265111d8bb3c3c608d95b3a0bf83461ace32d79336579a1939b3aad1c0b7")!
                ),
                index: 0
            )
        ]
        let outputs = [
            TransactionOutput(
                address: try Address(
                    bytes: Data(hex: "611c616f1acb460668a9b2f123c80372c2adad3583b9c6cd2b1deeed1c")!
                ),
                amount: Value(coin: 1)
            )
        ]
        
        let body = TransactionBody(inputs: inputs, outputs: outputs, fee: 94002, ttl: nil)
        var w = TransactionWitnessSet()
        let vkw = [
            try Vkeywitness(
                txBodyHash: try TransactionHash(txBody: body),
                sk: try PrivateKey(
                    normalBytes: Data(hex: "c660e50315d76a53d80732efda7630cae8885dfb85c46378684b3c6103e1284a")!
                )
            )
        ]
        w.vkeys = vkw
        let signedTx = Transaction(
            body: body,
            witnessSet: w,
            auxiliaryData: nil
        )
        let linearFee = LinearFee(constant: 2, coefficient: 500)
        XCTAssertEqual(
            try signedTx.bytes().hex(prefix: false),
            "84a300818258203b40265111d8bb3c3c608d95b3a0bf83461ace32d79336579a1939b3aad1c0b700018182581d611c616f1acb460668a9b2f123c80372c2adad3583b9c6cd2b1deeed1c01021a00016f32a10081825820f9aa3fccb7fe539e471188ccc9ee65514c5961c070b06ca185962484a4813bee58404f0250f89e6efb7a0aa7a28ad4f43371fe67ff6741e2c0f9ac04506e62b1bcb09a45c7d62884aed7c30ccbcf3b45e510cb1735ca0cd07f3978fc144776e1ee09f5f6"
        )
        XCTAssertEqual(try signedTx.minFee(linearFee: linearFee), 93502)
    }
    
    func testTxSimpleByronUtxo() throws {
        let inputs = [
            TransactionInput(
                transaction_id: try TransactionHash(
                    bytes: Data(hex: "3b40265111d8bb3c3c608d95b3a0bf83461ace32d79336579a1939b3aad1c0b7")!
                ),
                index: 0
            )
        ]
        let outputs = [
            TransactionOutput(
                address: try Address(
                    bytes: Data(hex: "611c616f1acb460668a9b2f123c80372c2adad3583b9c6cd2b1deeed1c")!
                ),
                amount: Value(coin: 1)
            )
        ]
        let body = TransactionBody(inputs: inputs, outputs: outputs, fee: 112002, ttl: nil)
        var w = TransactionWitnessSet()
        let bootstrapWits = [
            try BootstrapWitness(
                txBodyHash: try TransactionHash(txBody: body),
                addr: ByronAddress(base58: "Ae2tdPwUPEZ6r6zbg4ibhFrNnyKHg7SYuPSfDpjKxgvwFX9LquRep7gj7FQ"),
                key: try Bip32PrivateKey(
                    bytes: Data(hex: "d84c65426109a36edda5375ea67f1b738e1dacf8629f2bb5a2b0b20f3cd5075873bf5cdfa7e533482677219ac7d639e30a38e2e645ea9140855f44ff09e60c52c8b95d0d35fe75a70f9f5633a3e2439b2994b9e2bc851c49e9f91d1a5dcbb1a3")!
                )
            )
        ]
        w.bootstraps = bootstrapWits
        let signedTx = Transaction(
            body: body,
            witnessSet: w,
            auxiliaryData: nil
        )
        let linearFee = LinearFee(constant: 2, coefficient: 500)
        XCTAssertEqual(
            try signedTx.bytes().hex(prefix: false),
            "84a300818258203b40265111d8bb3c3c608d95b3a0bf83461ace32d79336579a1939b3aad1c0b700018182581d611c616f1acb460668a9b2f123c80372c2adad3583b9c6cd2b1deeed1c01021a0001b582a10281845820473811afd4d939b337c9be1a2ceeb2cb2c75108bddf224c5c21c51592a7b204a584068bb9b063617324463f44afcddd64483da7230b052b9df14ba7c6f1a8be39873321c40a544a7079cc6afff5285e44d1a0c2c2f722abde393c88a308194aa63075820c8b95d0d35fe75a70f9f5633a3e2439b2994b9e2bc851c49e9f91d1a5dcbb1a341a0f5f6"
        )
        XCTAssertEqual(try signedTx.minFee(linearFee: linearFee), 111502)
    }

    func testTxMultiUtxo() throws {
        let inputs = [
            TransactionInput(
                transaction_id: try TransactionHash(
                    bytes: Data(hex: "3b40265111d8bb3c3c608d95b3a0bf83461ace32d79336579a1939b3aad1c0b7")!
                ),
                index: 42
            ),
            TransactionInput(
                transaction_id: try TransactionHash(
                    bytes: Data(hex: "82839f8200d81858248258203b40265111d8bb3c3c608d95b3a0bf83461ace32")!
                ),
                index: 7
            )
        ]
        let outputs = [
            TransactionOutput(
                address: try Address(
                    bytes: Data(hex: "611c616f1acb460668a9b2f123c80372c2adad3583b9c6cd2b1deeed1c")!
                ),
                amount: Value(coin: 289)
            ),
            TransactionOutput(
                address: try Address(
                    bytes: Data(hex: "61bcd18fcffa797c16c007014e2b8553b8b9b1e94c507688726243d611")!
                ),
                amount: Value(coin: 874551452)
            )
        ]
        let body = TransactionBody(inputs: inputs, outputs: outputs, fee: 183502, ttl: 999)
        var w = TransactionWitnessSet()
        let vkw = [
            try Vkeywitness(
                txBodyHash: try TransactionHash(txBody: body),
                sk: try PrivateKey(
                    normalBytes: Data(hex: "c660e50315d76a53d80732efda7630cae8885dfb85c46378684b3c6103e1284a")!
                )
            ),
            try Vkeywitness(
                txBodyHash: try TransactionHash(txBody: body),
                sk: try PrivateKey(
                    normalBytes: Data(hex: "13fe79205e16c09536acb6f0524d04069f380329d13949698c5f22c65c989eb4")!
                )
            )
        ]
        w.vkeys = vkw
        let signedTx = Transaction(
            body: body,
            witnessSet: w,
            auxiliaryData: nil
        )
        let linearFee = LinearFee(constant: 2, coefficient: 500)
        XCTAssertEqual(
            try signedTx.bytes().hex(prefix: false),
            "84a300828258203b40265111d8bb3c3c608d95b3a0bf83461ace32d79336579a1939b3aad1c0b7182a82582082839f8200d81858248258203b40265111d8bb3c3c608d95b3a0bf83461ace3207018282581d611c616f1acb460668a9b2f123c80372c2adad3583b9c6cd2b1deeed1c19012182581d61bcd18fcffa797c16c007014e2b8553b8b9b1e94c507688726243d6111a3420989c021a0002cccea10082825820f9aa3fccb7fe539e471188ccc9ee65514c5961c070b06ca185962484a4813bee5840ac41078712556f323db6adc42ecef80c743caeecf4ad28c4dce0f0f704b7f78b01ee7d7c45d012f29e16d2753d1fd856c744f3de5dbbca906413473c4150440f8258206872b0a874acfe1cace12b20ea348559a7ecc912f2fc7f674f43481df973d92c5840b5a4f1be7174e4b9dd7f36639b144be03ebfa2b6d1080b8985edb94c5ca93eb8750d55b70ff54e80e3f0c9f98fbfd1065b9b4492dbd286529f323402d8093207f5f6"
        )
        XCTAssertEqual(try signedTx.minFee(linearFee: linearFee), 182002)
    }
    
    func testTxRegisterStake() throws {
        let network: UInt8 = 1
        let inputs = [
            TransactionInput(
                transaction_id: try TransactionHash(
                    bytes: Data(hex: "3b40265111d8bb3c3c608d95b3a0bf83461ace32d79336579a1939b3aad1c0b7")!
                ),
                index: 0
            )
        ]
        let outputs = [
            TransactionOutput(
                address: try Address(
                    bytes: Data(hex: "611c616f1acb460668a9b2f123c80372c2adad3583b9c6cd2b1deeed1c")!
                ),
                amount: Value(coin: 1)
            )
        ]
        var body = TransactionBody(inputs: inputs, outputs: outputs, fee: 266002, ttl: 10)
        let poolOwners = [
            try PublicKey(bytes: Data(hex: "54d1a9c5ad69586ceeb839c438400c376c0bd34825fb4c17cc2f58c54e1437f3")!).hash()
        ]
        let registrationCert = PoolRegistration(
            poolParams: PoolParams(
                operator: try PublicKey(
                    bytes: Data(hex: "b24c040e65994bd5b0621a060166d32d356ef4be3cc1f848426a4cf386887089")!
                ).hash(),
                vrfKeyhash: try VRFKeyHash(
                    bytes: Data(hex: "bd0000f498ccacdc917c28274cba51c415f3f21931ff41ca8dc1197499f8e124")!
                ),
                pledge: 1000000,
                cost: 1000000,
                margin: UnitInterval(numerator: 3, denominator: 100),
                rewardAccount: RewardAddress(
                    network: network,
                    payment: StakeCredential.keyHash(
                        try PublicKey(
                            bytes: Data(hex: "54d1a9c5ad69586ceeb839c438400c376c0bd34825fb4c17cc2f58c54e1437f3")!
                        ).hash()
                    )
                ),
                poolOwners: poolOwners,
                relays: [],
                poolMetadata: nil
            )
        )
        let certs = [Certificate.poolRegistration(registrationCert)]
        body.certs = certs
        var w = TransactionWitnessSet()
        let vkw = [
            try Vkeywitness(
                txBodyHash: try TransactionHash(txBody: body),
                sk: try PrivateKey(
                    normalBytes: Data(hex: "c660e50315d76a53d80732efda7630cae8885dfb85c46378684b3c6103e1284a")!
                )
            ),
            try Vkeywitness(
                txBodyHash: try TransactionHash(txBody: body),
                sk: try PrivateKey(
                    normalBytes: Data(hex: "2363f3660b9f3b41685665bf10632272e2d03c258e8a5323436f0f3406293505")!
                )
            ),
            try Vkeywitness(
                txBodyHash: try TransactionHash(txBody: body),
                sk: try PrivateKey(
                    normalBytes: Data(hex: "5ada7f4d92bce1ee1707c0a0e211eb7941287356e6ed0e76843806e307b07c8d")!
                )
            )
        ]
        w.vkeys = vkw
        let signedTx = Transaction(
            body: body,
            witnessSet: w,
            auxiliaryData: nil
        )
        let linearFee = LinearFee(constant: 2, coefficient: 500)
        XCTAssertEqual(
            try signedTx.bytes().hex(prefix: false),
            "84a400818258203b40265111d8bb3c3c608d95b3a0bf83461ace32d79336579a1939b3aad1c0b700018182581d611c616f1acb460668a9b2f123c80372c2adad3583b9c6cd2b1deeed1c01021a00040f1204818a03581c1c13374874c68016df54b1339b6cacdd801098431e7659b24928efc15820bd0000f498ccacdc917c28274cba51c415f3f21931ff41ca8dc1197499f8e1241a000f42401a000f4240d81e82031864581de151df9ba1b74a1c9608a487e114184556801e927d31d96425cb80af7081581c51df9ba1b74a1c9608a487e114184556801e927d31d96425cb80af7080f6a10083825820f9aa3fccb7fe539e471188ccc9ee65514c5961c070b06ca185962484a4813bee584065766ccc2a2f01ef59e94d5da694faf6340962f536d8025261c82e5823612849e4924282ef9b603b9ef9ebe2cb4b089a3614fe3a87d6308b7b1c4d4d5619af0f825820b24c040e65994bd5b0621a060166d32d356ef4be3cc1f848426a4cf386887089584017ff22c9ca2737e0a5e904f2b20db93b8586732b5db13856763ee842b10a66ef261c8f8e43ba0bb8cc6812e5417f44e4919040e3de0109eb2bd54216ce1e3d0282582054d1a9c5ad69586ceeb839c438400c376c0bd34825fb4c17cc2f58c54e1437f35840c7a0b0a572766a9380dfd4c5eb40ca300fe3004033aa47a40a8d68d36bebc64d82bf1cc47ac24bed6aa7f7b2e7190ae036ae5f2b790161ab37723d1dc53fee04f5f6"
        )
        XCTAssertEqual(try signedTx.minFee(linearFee: linearFee), 268502)
    }
    
    func testTxWithdrawal() throws {
        let inputs = [
            TransactionInput(
                transaction_id: try TransactionHash(
                    bytes: Data(hex: "3b40265111d8bb3c3c608d95b3a0bf83461ace32d79336579a1939b3aad1c0b7")!
                ),
                index: 0
            )
        ]
        let outputs = [
            TransactionOutput(
                address: try Address(
                    bytes: Data(hex: "611c616f1acb460668a9b2f123c80372c2adad3583b9c6cd2b1deeed1c")!
                ),
                amount: Value(coin: 1)
            )
        ]
        var body = TransactionBody(inputs: inputs, outputs: outputs, fee: 162502, ttl: 10)
        let withdrawals = [
            try Address(bytes: Data(hex: "e151df9ba1b74a1c9608a487e114184556801e927d31d96425cb80af70")!).reward!: UInt64(1337)
        ]
        body.withdrawals = withdrawals
        var w = TransactionWitnessSet()
        let vkw = [
            try Vkeywitness(
                txBodyHash: try TransactionHash(txBody: body),
                sk: try PrivateKey(
                    normalBytes: Data(hex: "c660e50315d76a53d80732efda7630cae8885dfb85c46378684b3c6103e1284a")!
                )
            ),
            try Vkeywitness(
                txBodyHash: try TransactionHash(txBody: body),
                sk: try PrivateKey(
                    normalBytes: Data(hex: "5ada7f4d92bce1ee1707c0a0e211eb7941287356e6ed0e76843806e307b07c8d")!
                )
            )
        ]
        w.vkeys = vkw
        let signedTx = Transaction(
            body: body,
            witnessSet: w,
            auxiliaryData: nil
        )
        let linearFee = LinearFee(constant: 2, coefficient: 500)
        XCTAssertEqual(
            try signedTx.bytes().hex(prefix: false),
            "84a400818258203b40265111d8bb3c3c608d95b3a0bf83461ace32d79336579a1939b3aad1c0b700018182581d611c616f1acb460668a9b2f123c80372c2adad3583b9c6cd2b1deeed1c01021a00027ac605a1581de151df9ba1b74a1c9608a487e114184556801e927d31d96425cb80af70190539a10082825820f9aa3fccb7fe539e471188ccc9ee65514c5961c070b06ca185962484a4813bee5840bd955e767ec601c7e9f1908d7c7cf0ad7975f8689e6b7c1a8a8ed1fa0a0ac96f156f2f24790fb66b86c35c51865dc63af80bded7ee47125a377c723fd144570d82582054d1a9c5ad69586ceeb839c438400c376c0bd34825fb4c17cc2f58c54e1437f35840685ccbeb79d1b032634ecc57f4f67f5f7f37a4f7730e38b02f0e693befc0c3a9a6907b2d0127ea56521da3fc2c6f1081a12ea06cf4cfa95cf68db13c10f0e802f5f6"
        )
        XCTAssertEqual(try signedTx.minFee(linearFee: linearFee), 162002)
    }
}
