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
    public let qty: Int64
    
    init(asset: CCardano.CmlAsset) {
        fingerprint = asset.fingerprint.copied()
        policy = asset.policy.copied()
        name = asset.name.copied()
        qty = asset.qty
    }
    
    /**
     * Try and decode the Asset name into UTF8, should handle CIP25 and CIP68 assets.
     * If we fail in our task we return the Hex encoding of the name.
     */
    public func utf8Name() -> String {
        guard let theName = String.init(data: name, encoding: String.Encoding.utf8) else {
            if name.count < 5 {
                return name.hex(prefix: false)
            }
            let subData = name.subdata(in: Range(4...name.count - 1))
            
            guard let theName = String.init(data: subData, encoding: String.Encoding.utf8) else {
                return name.hex(prefix: false)
            }
            
            return theName
        }
        
        return theName
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

public struct CmlAssetPair: Codable {
    public let policy: Data
    public let asset_name: Data
    public let amount: UInt64;
    
    func withCAssetPair<T>(fn: @escaping (AssetPair) throws -> T ) rethrows -> T {
        try policy.withCData { policy in
            
            try asset_name.withCData { asset_name in
                try fn(
                    AssetPair(policy: policy, asset_name: asset_name, amount: amount)
                )
            }
            
        }
    }
    
}

typealias CmlAssetPairs = Array<CmlAssetPair>

extension CCardano.CArray_AssetPair: CArray {
    typealias CElement = CCardano.AssetPair
    typealias Val = [CCardano.AssetPair]
    
    mutating func free() {
        //TODO: Impl Free for this type!
    }
}

extension CmlAssetPairs {
    func withCArray<T>(fn: @escaping (CCardano.CArray_AssetPair) throws -> T) rethrows -> T {
        try self.withCArray(with: { try $0.withCAssetPair(fn: $1)}, fn: fn)
    }
}



extension CCardano.AssetPair: CPtr {
    typealias Val = CmlAssetPair
    
    func copied() -> CmlAssetPair {
        
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
    public let cbor: Data

    init(txo: CCardano.CmlTxOutput) {
        address = txo.address.copied()
        value = txo.value.copied()
        cbor = txo.cbor.copied()
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

extension COption_CmlAssets: COption {   
    typealias Tag = COption_CmlAssets_Tag
    typealias Value = CCardano.CmlAssets
    
    func someTag() -> COption_CmlAssets_Tag {
        Some_CmlAssets
    }
    func noneTag() -> COption_CmlAssets_Tag {
        None_CmlAssets
    }
}

extension COption_CData: COption {
    typealias Tag = COption_CData_Tag
    typealias Value = CCardano.CData
    
    func someTag() -> COption_CData_Tag {
        Some_CData
    }
    func noneTag() -> COption_CData_Tag {
        None_CData
    }
}

public struct UTxO: Codable {
    public let tx_hash: Data
    public let tx_index: UInt64
    public let orig_output: Optional<TxnOutput>
    
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
    
    public init(tx_hash: String, tx_index: UInt64, address: String, lovelace: UInt64, assets: Array<CmlAssetPair>, datum: Optional<Data>) throws {
        let cmlUtxo = try tx_hash.withCString { tx_hash in
            try address.withCString { address in
                try assets.withCArray { assets in
                    try datum.withCOption(with: {try $0.withCData(fn: $1) }) { datum in
                        RustResult<CCardano.CmlUTxO>.wrap { result, error in
                            into_utxo(tx_hash, tx_index, address, lovelace, assets, datum, result, error)
                        }
                    }
                }
            }
        }.get()
        
        self.init(utxo: cmlUtxo)
    }
    
    public func getAvailableLovelace() throws -> Int64 {
        
        if let output = self.orig_output {
            let ll = try output.cbor.withCData { output in
                RustResult<UInt64>.wrap { result, error in
                    available_lovelace(output, 4310, result, error) //TODO: Coins Per UTxO byte could change. How to get this value from chain?
                }
            }.get()
            
            return Int64(ll)
        }
        
        return 0
    }
}

public func tx_merge_in_witness_set(transaction: Data, witness_set: Data) throws -> Data {
    let result = try transaction.withCData { txn in
        witness_set.withCData { ws in
            RustResult<Data>.wrap { result, error in
                cml_tx_add_signers(txn, ws, result, error)
            }
        }
    }.get()
    
    return result.copied()
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
    public let mints: [CmlAsset]?
    
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
        mints = txDetails.mints.get()?.copied().map{ $0.copied() }
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

public struct SwiftMempoolUtxos: Codable {
    public let spent_inputs: Array<UTxO>
    public let created_utxos: Array<UTxO>
    
    init(spent_inputs: Array<UTxO>, created_utxos: Array<UTxO>) {
        self.spent_inputs = spent_inputs
        self.created_utxos = created_utxos
    }
    
    init(core: CCardano.MempoolUtxos) {
        created_utxos = core.created_utxos.copied().map { $0.copied() }
        spent_inputs = core.spent_inputs.copied().map { $0.copied() }
    }
    
    public init(tx: Data) throws {
        let txnData = try tx.withCData { bytes in
            RustResult<CCardano.MempoolUtxos>.wrap { result, error in
                cml_tx_utxo_result(bytes, result, error)
            }
        }.get()
        
        self = Self.init(core: txnData)
    }
}

extension CCardano.MempoolUtxos: CPtr {
    typealias Val = SwiftMempoolUtxos
    
    func copied() -> SwiftMempoolUtxos {
        SwiftMempoolUtxos.init(core: self)
    }
    mutating func free() {
        
    }
}

