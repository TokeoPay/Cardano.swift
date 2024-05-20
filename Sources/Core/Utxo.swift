//
//  File.swift
//  
//
//  Created by Gavin Harris on 8/5/2024.
//

import Foundation
import CCardano

public typealias SwiftUtxo = CCardano.SwiftUTXO
public typealias SwiftUtxos = Array<CCardano.SwiftUTXO>

extension SwiftUtxo: CType {}

extension CCardano.SwiftUTXOs: CArray {
    typealias CElement = CCardano.SwiftUTXO
    typealias Val = [CCardano.SwiftUTXO]
    
    mutating func free() {
        //TODO: Impl Free for this type!
    }
}

public struct Utxo {
    public let transaction_hash: String
    public let transaction_index: UInt64
    public let tx_out_bytes: Data
    
    public init(transaction_hash: String, transaction_index: UInt64, tx_out_bytes: Data) {
        self.transaction_hash = transaction_hash
        self.transaction_index = transaction_index
        self.tx_out_bytes = tx_out_bytes
    }
    
    func withCSwiftUtxo<T>(fn: @escaping (SwiftUTXO) throws -> T ) rethrows -> T {
        try transaction_hash.withCString { tx_id in
            try tx_out_bytes.withCData { cdata in
                try fn(
                    SwiftUTXO(
                        transaction_hash: tx_id,
                        transaction_index: transaction_index,
                        tx_out_bytes: cdata
                    )
                )
            }
        }
    }
}

public typealias Utxos = Array<Utxo>

extension Utxos {
    func withCArray<T>(fn: @escaping (CCardano.SwiftUTXOs) throws -> T) rethrows -> T {
        try self.withCArray(with: { try $0.withCSwiftUtxo(fn: $1)}, fn: fn)
    }
}

extension SwiftUTXO: CPtr {
    typealias Val = Utxo
    
    func copied() -> Utxo {
        Utxo(
            transaction_hash: self.transaction_hash.copied(),
            transaction_index: self.transaction_index,
            tx_out_bytes: self.tx_out_bytes.copied()
        )
    }
    
    mutating func free() {
        cardano_swift_utxo_free(&self)
    }
}

public struct CoinSelectionResult {
    let selected: Utxos
    let other: Utxos
}

extension CCardano.CoinSelectionResult: CPtr {
    mutating func free() {
        //TODO: Add free to Rust code
    }
    
    typealias Val = CoinSelectionResult
    
    func copied() -> CoinSelectionResult {
        return CoinSelectionResult(
            selected: selected.copied().map {
                Utxo(
                    transaction_hash: $0.transaction_hash.copied(),
                    transaction_index: $0.transaction_index,
                    tx_out_bytes: $0.tx_out_bytes.copied()
                )
            },  other: other.copied().map {
                Utxo(
                    transaction_hash: $0.transaction_hash.copied(),
                    transaction_index: $0.transaction_index,
                    tx_out_bytes: $0.tx_out_bytes.copied()
                )
            })
    }
}

public func coin_selection(utxos: Utxos, amount: Value) throws -> CoinSelectionResult  {
        
    let result = try utxos.withCArray { swiftUtxos in
        amount.withCValue { cValue in
            RustResult<CCardano.CoinSelectionResult>.wrap { result, error in
                cardano_transaction_unspent_outputs_coin_selection(swiftUtxos, cValue, result, error)
            }
        }}.get()
    

    return result.copied()
}
