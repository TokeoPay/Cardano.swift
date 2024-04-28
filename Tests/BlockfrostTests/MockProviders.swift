//
//  MockProviders.swift
//  
//
//  Created by Yehor Popovych on 25.11.2021.
//

import Foundation
import Cardano

struct SignatureProviderMock: SignatureProvider {
    
    
    var accountsMock: ((_ cb: @escaping (Result<[Account], Error>) -> Void) -> Void)?
    var signMock: ((ExtendedTransaction, _ cb: @escaping (Result<Transaction, Error>) -> Void) -> Void)?
    var signDataMock: ((String, _ cb: @escaping (Result<Data, Error>) -> Void) -> Void)?
    var signMessageMock: ((_ data: Data, _ extended_address: ExtendedAddress, _ cb: @escaping (Result<CardanoCore.Cip30DataSignature, Error>) -> Void) -> Void)?
    
    func accounts(_ cb: @escaping (Result<[Account], Error>) -> Void) {
        accountsMock!(cb)
    }
    
    func sign(tx: ExtendedTransaction,
              _ cb: @escaping (Result<Transaction, Error>) -> Void) {
        signMock!(tx, cb)
    }
    func sign(txHash: String, addresses: [CardanoCore.ExtendedAddress], _ cb: @escaping (Result<Data, any Error>) -> Void) {
        signDataMock!(txHash, cb)
    }
    func signData(data: Data, extended_address: ExtendedAddress, _ cb: @escaping (Result<CardanoCore.Cip30DataSignature, any Error>) -> Void) {
        signMessageMock!(data, extended_address, cb)
    }
}
