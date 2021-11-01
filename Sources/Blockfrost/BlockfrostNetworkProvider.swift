//
//  BlockfrostNetworkProvider.swift
//  
//
//  Created by Ostap Danylovych on 28.10.2021.
//

import Foundation
import BlockfrostSwiftSDK
#if !COCOAPODS
import Cardano
#endif

public struct BlockfrostNetworkProvider: NetworkProvider {
    private let config: BlockfrostConfig
    private let addressesApi: CardanoAddressesAPI
    
    public init(config: BlockfrostConfig) {
        self.config = config
        addressesApi = CardanoAddressesAPI(config: config)
    }
    
    public func getTransactions(for address: Address,
                                _ cb: @escaping (Result<[AddressTransaction], Error>) -> Void) {
        do {
            let _ = addressesApi.getAddressTransactionsAll(address: try address.bech32()) { res in
                cb(res.map { transactions in
                    transactions.map { AddressTransaction(blockfrost: $0) }
                })
            }
        } catch {
            self.config.apiResponseQueue.async {
                cb(.failure(error))
            }
        }
    }
    
    public func getTransactionCount(for address: Address,
                                    _ cb: @escaping (Result<Int, Error>) -> Void) {
        do {
            let _ = addressesApi.getAddressDetails(address: try address.bech32()) { res in
                cb(res.map { $0.txCount })
            }
        } catch {
            self.config.apiResponseQueue.async {
                cb(.failure(error))
            }
        }
    }
    
    public func getTransaction(hash: String, _ cb: @escaping (Result<Any, Error>) -> Void) {
        fatalError("Not implemented")
    }
    
    public func getUtxos(for addresses: [Address],
                         page: Int,
                         _ cb: @escaping (Result<[UTXO], Error>) -> Void) {
        let b32Addresses: Array<(Address, String)>
        do {
            b32Addresses = try addresses.map { try ($0, $0.bech32()) }
        } catch {
            self.config.apiResponseQueue.async {
                cb(.failure(error))
            }
            return
        }
        b32Addresses.asyncMap { (addrAndB32, mapped) in
            let (address, b32) = addrAndB32
            let _ = self.addressesApi.getAddressUtxos(
                address: b32,
                page: page
            ) { res in
                let result = res.flatMap { res in
                    Result { try res.map { try UTXO(address: address, blockfrost: $0) } }
                }
                mapped(result)
            }
        }.exec { (res: Result<[[UTXO]], Error>) in
            cb(res.map { utxo in utxo.flatMap { $0 } })
        }
    }
    
    public func submit(tx: TransactionBody,
                       metadata: TransactionMetadata?,
                       _ cb: @escaping (Result<Transaction, Error>) -> Void) {
        fatalError("Not implemented")
    }
}