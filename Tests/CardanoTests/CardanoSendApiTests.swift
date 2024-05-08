//
//  CardanoSendApiTests.swift
//
//
//  Created by Ostap Danylovych on 02.11.2021.
//

import XCTest
@testable import Cardano
import Bip39

//final class CardanoSendApiTests: XCTestCase {
//    private let networkProvider = NetworkProviderMock(getSlotNumberMock: { cb in
//        cb(.success(50000000))
//    }, submitMock: { tx, cb in
//        guard try! tx.bytes() == testTransaction.bytes() else {
//            cb(.failure(ApiTestError.error(from: "submit")))
//            return
//        }
//        cb(.success(testTransactionHash))
//    })
//    
//    private let signatureProvider = SignatureProviderMock(signMock: { tx, cb in
//        cb(.success(testTransaction))
//    })
//    
//    private let addressManager = AddressManagerMock(newMock: { account, change in
//        guard account == testAccount, change else {
//            throw ApiTestError.error(from: "new")
//        }
//        return testChangeAddress
//    }, getCachedMock: { account in
//        guard account == testAccount else {
//            throw ApiTestError.error(from: "get cached")
//        }
//        return [testExtendedAddress.address]
//    }, extendedMock: { addresses in
//        [testExtendedAddress]
//    })
//    
//    private let utxoProvider = UtxoProviderMock(utxoIteratorNextMock: { cb in
//        cb(.success([testUtxo]), nil)
//    })
//    
//    private static let testMnemonic = try! Mnemonic()
//    
//    private static var testAccount: Account {
//        let keychain = try! Keychain(mnemonic: testMnemonic.mnemonic(), password: Data())
//        return try! keychain.addAccount(index: 0)
//    }
//    
//    private static var testToAddress: Address {
//        let keychain = try! Keychain(mnemonic: testMnemonic.mnemonic(), password: Data())
//        let account = try! keychain.addAccount(index: 1)
//        return try! account.baseAddress(
//            index: 0,
//            change: false,
//            networkID: NetworkInfo.testnet.network_id
//        ).address
//    }
//    
//    private static var testExtendedAddress: ExtendedAddress {
//        try! testAccount.baseAddress(
//            index: 0,
//            change: false,
//            networkID: NetworkInfo.testnet.network_id
//        )
//    }
//    
//    private static var testChangeAddress: Address {
//        try! testAccount.baseAddress(
//            index: 1,
//            change: true,
//            networkID: NetworkInfo.testnet.network_id
//        ).address
//    }
//    
//    private static let testUtxo: TransactionUnspentOutput = {
//        var value = Value(coin: 10000000)
//        let policyID = try! PolicyID(bytes: testAssetID.policyIDData!)
//        let assetName = try! AssetName(name: testAssetID.assetNameData!)
//        value.multiasset = MultiAsset(
//            dictionaryLiteral: (
//                policyID,
//                Assets(dictionaryLiteral: (assetName, 10000000))
//            )
//        )
//        return TransactionUnspentOutput(
//            input: TransactionInput(
//                transaction_id: TransactionHash(),
//                index: 1
//            ),
//            output: TransactionOutput(
//                address: testExtendedAddress.address,
//                amount: value
//            )
//        )
//    }()
//
//    private static let testTransactionHash = try! TransactionHash(bytes: Data(repeating: 0, count: 32))
//    
//    private static let testTransaction = Transaction(
//        body: TransactionBody(inputs: [], outputs: [], fee: 0, ttl: nil),
//        witnessSet: TransactionWitnessSet(),
//        auxiliaryData: nil
//    )
//
//    private static let testAssetID = "f6f49b186751e61f1fb8c64e7504e771f968cea9f4d11f5222b169e374574d54"
//
//    func testSendAdaFromAccount() throws {
//        let success = expectation(description: "success")
//        let cardano = try Cardano(
//            info: .testnet,
//            signer: signatureProvider,
//            network: networkProvider,
//            addresses: addressManager,
//            utxos: utxoProvider
//        )
//        cardano.send.ada(to: Self.testToAddress, lovelace: 1000000, from: Self.testAccount) { res in
//            let transactionHash = try! res.get()
//            XCTAssertEqual(transactionHash, Self.testTransactionHash)
//            success.fulfill()
//        }
//        wait(for: [success], timeout: 10)
//    }
//    
//    func testSendAdaFromAddresses() throws {
//        let success = expectation(description: "success")
//        let cardano = try Cardano(
//            info: .testnet,
//            signer: signatureProvider,
//            network: networkProvider,
//            addresses: addressManager,
//            utxos: utxoProvider
//        )
//        cardano.send.ada(to: Self.testToAddress,
//                         lovelace: 1000000,
//                         from: [Self.testExtendedAddress.address],
//                         change: Self.testChangeAddress) { res in
//            let transactionHash = try! res.get()
//            XCTAssertEqual(transactionHash, Self.testTransactionHash)
//            success.fulfill()
//        }
//        wait(for: [success], timeout: 10)
//    }
//
//    func testSendNativeTokenFromAccount() throws {
//        let success = expectation(description: "success")
//        let cardano = try Cardano(
//            info: .testnet,
//            signer: signatureProvider,
//            network: networkProvider,
//            addresses: addressManager,
//            utxos: utxoProvider
//        )
//        cardano.send.token(assetID: Self.testAssetID,
//                           to: Self.testToAddress,
//                           lovelace: 1000000,
//                           from: Self.testAccount) { res in
//            let transactionHash = try! res.get()
//            XCTAssertEqual(transactionHash, Self.testTransactionHash)
//            success.fulfill()
//        }
//        wait(for: [success], timeout: 10)
//    }
//
//    func testSendNativeTokenFromAddresses() throws {
//        let success = expectation(description: "success")
//        let cardano = try Cardano(
//            info: .testnet,
//            signer: signatureProvider,
//            network: networkProvider,
//            addresses: addressManager,
//            utxos: utxoProvider
//        )
//        cardano.send.token(assetID: Self.testAssetID,
//                           to: Self.testToAddress,
//                           lovelace: 1000000,
//                           from: [Self.testExtendedAddress.address],
//                           change: Self.testChangeAddress) { res in
//            let transactionHash = try! res.get()
//            XCTAssertEqual(transactionHash, Self.testTransactionHash)
//            success.fulfill()
//        }
//        wait(for: [success], timeout: 10)
//    }
//}
