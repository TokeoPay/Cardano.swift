//
//  UtxoProvider.swift
//  
//
//  Created by Ostap Danylovych on 27.10.2021.
//

import Foundation

public protocol UtxoProvider {
    func get(for addresses: [Address],
             asset: (PolicyID, AssetName)?) -> UtxoProviderAsyncIterator
    
    func get(id: (tx: TransactionHash, index: TransactionIndex),
             _ cb: @escaping (Result<[UTXO], Error>) -> Void)
}

public protocol UtxoProviderAsyncIterator {
    func next(_ cb: @escaping (Result<[UTXO], Error>, Self?) -> Void)
    func next(limit: Int, _ cb: @escaping (Result<[UTXO], Error>, Self?) -> Void)
}