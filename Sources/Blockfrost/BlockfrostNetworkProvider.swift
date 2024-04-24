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
    private let transactionsApi: CardanoTransactionsAPI
    private let blocksApi: CardanoBlocksAPI
    
    public init(config: BlockfrostConfig) {
        self.config = config
        addressesApi = CardanoAddressesAPI(config: config)
        transactionsApi = CardanoTransactionsAPI(config: config)
        blocksApi = CardanoBlocksAPI(config: config)
    }
    
    private func handleError<R>(error: Error,
                                expectedStatus: Int,
                                response: R,
                                _ cb: @escaping (Result<R, Error>) -> Void) {
        guard let errorResponse = error as? ErrorResponse else {
            self.config.apiResponseQueue.async {
                cb(.failure(error))
            }
            return
        }
        switch errorResponse {
        case .errorStatus(let int, _, _, _):
            guard int == expectedStatus else {
                self.config.apiResponseQueue.async {
                    cb(.failure(errorResponse))
                }
                return
            }
            cb(.success(response))
        default:
            self.config.apiResponseQueue.async {
                cb(.failure(errorResponse))
            }
        }
    }
    
    public func getSlotNumber(_ cb: @escaping (Result<Int?, Error>) -> Void) {
        let _ = blocksApi.getLatestBlock() { res in
            cb(res.map { block in
                block.slot
            })
        }
    }

    public func getBlockProcessingTime(_ cb: @escaping (Result<TimeInterval?, Error>) -> Void) {
        let _ = blocksApi.getLatestBlock { res in
            switch res {
            case .success(let latestBlock):
                guard let previousBlockHash = latestBlock.previousBlock else {
                    cb(.success(nil))
                    return
                }
                let _ = blocksApi.getBlock(hashOrNumber: previousBlockHash) { res in
                    switch res {
                    case .success(let previousBlock):
                        let difference = TimeInterval(latestBlock.time - previousBlock.time)
                        cb(.success(difference))
                    case .failure(let error):
                        cb(.failure(error))
                    }
                }
            case .failure(let error):
                cb(.failure(error))
            }
        }
    }

    public func getBlockConfirmations(for hash: String, _ cb: @escaping (Result<Int, Error>) -> Void) {
        let _ = blocksApi.getBlock(hashOrNumber: hash) { res in
            switch res {
            case .success(let block):
                cb(.success(block.confirmations))
            case .failure(let error):
                cb(.failure(error))
            }
        }
    }

    public func getBalance(for address: Address,
                           _ cb: @escaping (Result<UInt64, Error>) -> Void) {
        do {
            let _ = addressesApi.getAddress(address: try address.bech32()) { res in
                switch res {
                case .success(let address):
                    cb(Result {
                        try Value(blockfrost: address.amount.map {
                            (unit: $0.unit, quantity: $0.quantity)
                        }).coin
                    })
                case .failure(let error):
                    handleError(error: error, expectedStatus: 404, response: 0, cb)
                }
            }
        } catch {
            self.config.apiResponseQueue.async {
                cb(.failure(error))
            }
        }
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
                switch res {
                case .success(let details):
                    cb(.success(details.txCount))
                case .failure(let error):
                    handleError(error: error, expectedStatus: 404, response: 0, cb)
                }
            }
        } catch {
            self.config.apiResponseQueue.async {
                cb(.failure(error))
            }
        }
    }
    
    public func getTransaction(hash: TransactionHash,
                               _ cb: @escaping (Result<ChainTransaction?, Error>) -> Void) {
        let _ = transactionsApi.getTransaction(hash: hash.hex) { res in
            switch res {
            case .success(let transactionContent):
                cb(.success(ChainTransaction(blockfrost: transactionContent)))
            case .failure(let error):
                handleError(error: error, expectedStatus: 404, response: nil, cb)
            }
        }
    }
    
    public func getUtxos(for addresses: [Address],
                         page: Int,
                         _ cb: @escaping (Result<[TransactionUnspentOutput], Error>) -> Void) {
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
                switch res {
                case .success(let utxos):
                    mapped(Result { try utxos.map {
                        try TransactionUnspentOutput(address: address, blockfrost: $0)
                    } })
                case .failure(let error):
                    handleError(error: error, expectedStatus: 404, response: [], mapped)
                }
            }
        }.exec { (res: Result<[[TransactionUnspentOutput]], Error>) in
            cb(res.map { utxo in utxo.flatMap { $0 } })
        }
    }
    
    public func getUtxos(for transaction: TransactionHash,
                         _ cb: @escaping (Result<[TransactionUnspentOutput], Error>) -> Void) {
        do {
            let _ = transactionsApi.getTransactionUtxos(hash: try transaction.bytes().hex()) { res in
                cb(res.flatMap { txContentUtxo in
                    Result { try txContentUtxo.inputs.map {
                        try TransactionUnspentOutput(blockfrost: $0)
                    } }
                })
            }
        } catch {
            self.config.apiResponseQueue.async {
                cb(.failure(error))
            }
        }
    }
    
    public func submit(tx: Transaction,
                       _ cb: @escaping (Result<TransactionHash, Error>) -> Void) {
    
        let bytes: Data
        do {
            bytes = try tx.bytes()
        } catch {
            self.config.apiResponseQueue.async {
                cb(.failure(error))
            }
            return
        }

        return self.submit(tx: bytes, cb);
    }
    
    public func submit(tx: Data,
                       _ cb: @escaping (Result<TransactionHash, Error>) -> Void) {
        let _ = transactionsApi.submitTransaction(transaction: tx) { res in
            let mapped = res.flatMap { hash in
                Result { try TransactionHash(hex: hash.trimmingCharacters(in: ["\""])) }
            }
            cb(mapped)
        }
    }
}
