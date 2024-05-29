//
//  CmlTransaction.swift
//  
//
//  Created by Gavin Harris on 16/5/2024.
//

import Foundation
import CCardano


/* MARK: TxDetails */

public struct CmlAsset: Codable {
    public let fingerprint: String
    public let policy: Data
    public let name: Data
    public let qty: UInt64
    
    init(asset: CCardano.CmlAsset) {
        fingerprint = asset.fingerprint.copied()
        policy = asset.policy.copied()
        name = asset.name.copied()
        qty = asset.qty
    }
    
}
extension CCardano.CmlAsset: CPtr {
    typealias Val = CmlAsset
    
    func copied() -> CmlAsset {
        CmlAsset(asset: self)
    }
    
    mutating func free() {
    }
}

public typealias CmlAssets = Array<CmlAsset>

extension CCardano.CmlAssets: CArray {
    typealias CElement = CCardano.CmlAsset
    typealias Val = [CCardano.CmlAsset]
    
    mutating func free() {
        
    }
}

public struct CmlValue: Codable {
    public let lovelace: UInt64
    public let assets: [CmlAsset]
    
    init(val: CCardano.CmlValue) {
        lovelace = val.lovelace
        assets = val.assets.copied().map { $0.copied() }
    }
}

extension CCardano.CmlValue: CPtr {
    typealias Val = CmlValue
    
    func copied() -> CmlValue {
        CmlValue(val: self)
    }
    
    mutating func free() {
    }
}

public struct TxnOutput: Codable {
    public let address: String
    public let value: CmlValue

    init(txo: CCardano.CmlTxOutput) {
        address = txo.address.copied()
        value = txo.value.copied()
    }
}

extension CCardano.CmlTxOutput: CPtr {
    typealias Val = TxnOutput
    
    func copied() -> TxnOutput {
        TxnOutput(txo: self)
    }
    
    mutating func free() {
    }
}

extension COption_CmlTxOutput: COption {
    typealias Tag = COption_CmlTxOutput_Tag
    typealias Value = CmlTxOutput
    
    func someTag() -> Tag {
        Some_CmlTxOutput
    }
    
    func noneTag() -> Tag {
        None_CmlTxOutput
    }
}

public struct UTxO: Codable {
    public let tx_hash: Data
    public let tx_index: UInt64
    public let orig_output: Optional<TxnOutput>
    let core_output: Data
    
    init(utxo: CCardano.CmlUTxO) {
        tx_hash = utxo.tx_hash.copied()
        tx_index = utxo.tx_index
        orig_output = utxo.orig_output.get()?.copied()
    }
    
    public init(tx_hash: String, tx_index: UInt64, output: Data) throws {
        let cmlUtxo = try output.withCData { output in
            tx_hash.withCString { tx_hash in
                RustResult<CCardano.CmlUTxO>.wrap { result, error in
                    utxo_from_parts(tx_hash, tx_index, output, result, error)
                }
            }
        }.get()
                
        self.init(utxo: cmlUtxo)
    }
    
    public func getMinAdaForUtxo() throws -> Int64 {
        let ll = try self.core_output.withCData { output in
            RustResult<UInt64>.wrap { result, error in
                available_lovelace(output, 4310, result, error) //TODO: Coins Per UTxO byte could change. How to get this value from chain?
            }
        }.get()
        
        return Int64(ll)
    }
}

extension CCardano.CmlUTxO: CPtr {
    typealias Val = UTxO
    
    func copied() -> UTxO {
        UTxO(utxo: self)
    }
    
    mutating func free() {}
}


extension CCardano.CmlUTxOs: CArray {
    typealias CElement = CCardano.CmlUTxO
    typealias Val = [CCardano.CmlUTxO]
    
    mutating func free() {
        //TODO: Need to implement!
    }
}

extension COption_CmlUTxOs: COption {
    typealias Tag = COption_CmlUTxOs_Tag
    typealias Value = CmlUTxOs
    
    func someTag() -> COption_CmlUTxOs_Tag {
        Some_CmlUTxOs
    }
    func noneTag() -> COption_CmlUTxOs_Tag {
        None_CmlUTxOs
    }
}

extension CmlTxOutputs: CArray {
    typealias CElement = CmlTxOutput
    typealias Val = [CmlTxOutput]
    
    mutating func free() {
    }
}

public struct TxSummaries: Codable {
    let stake_address: String
    let value: CmlValue
    
    init(sum: CCardano.CmlTxSummarised) {
        stake_address = sum.stake_address.copied()
        value = sum.value.copied()
    }
}

extension CmlTxSummarised: CPtr {
    func copied() -> TxSummaries {
        TxSummaries(sum: self)
    }
    
    mutating func free() {
    }
    
    typealias Val = TxSummaries
}

extension CmlTxSummaries: CArray {
    typealias CElement = CmlTxSummarised
    typealias Val = [CmlTxSummarised]
    
    mutating func free() {
    }
}

public struct SwiftTxDetails: Codable {
    public let fee: Coin
    public let hash: Data
    public let inputs: Array<UTxO>
    public let collateral: Array<UTxO>?
    public let collateral_output: TxnOutput?
    public let signers: [Data]
    public let outputs: [TxnOutput]
               //
    public let sum_outputs: [TxSummaries]
    public let sum_inputs: [TxSummaries]
    
    init(txDetails: CCardano.TxDetails) {
        fee = txDetails.fee
        hash = txDetails.hash.copied()
        inputs = txDetails.inputs.copied().map{ $0.copied() }
        collateral = txDetails.collateral.get()?.copied().map{ $0.copied() }
        collateral_output = txDetails.collateral_output.get()?.copied()
        signers = txDetails.signers.copied().map{ $0.copied() }
        outputs = txDetails.outputs.copied().map { $0.copied() }
        sum_outputs = txDetails.sum_outputs.copied().map { $0.copied() }
        sum_inputs = txDetails.sum_inputs.copied().map { $0.copied() }
    }
    
    public init(transaction: Data) throws {
        let txDetails = try CCardano.TxDetails.init(transaction: transaction)
        self.init(txDetails: txDetails)
    }
}

extension CCardano.TxDetails: CPtr {
    typealias Val = SwiftTxDetails
    
    func copied() -> SwiftTxDetails {
        SwiftTxDetails(txDetails: self)
    }
    
    mutating func free() {
        
    }
}

extension CCardano.TxDetails {
    public init(transaction: Data) throws {
        self = try transaction.withCData{ bytes in
            RustResult<CCardano.TxDetails>.wrap { result, error in
                cml_tx_details(bytes, result, error)
            }
        }.get()
    }
}


