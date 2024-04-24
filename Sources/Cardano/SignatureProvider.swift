//
//  SignatureProvider.swift
//  
//
//  Created by Yehor Popovych on 27.10.2021.
//

import Foundation
#if !COCOAPODS
import CardanoCore
#endif

public protocol SignatureProvider {
    func accounts(_ cb: @escaping (Result<[Account], Error>) -> Void)
    
    func sign(tx: ExtendedTransaction,
              _ cb: @escaping (Result<Transaction, Error>) -> Void)

    func sign(txHash: String, addresses: [ExtendedAddress],
              _ cb: @escaping (Result<Data, Error>) -> Void)
}
