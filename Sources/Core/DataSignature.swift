//
//  DataSignature.swift
//
//
//  Created by Gavin Harris on 27/4/2024.
//

import Foundation
import CCardano

public struct Cip30DataSignature {
    let signature: String
    let key: String
    
    init(ds: DataSignature) {
        self.key = ds.key.copied().hex(prefix: false)
        self.signature = ds.signature.copied().hex(prefix: false)
    }
    
    public func toJson() -> String {
        return String(format: "{\"key\": \"{0}\", \"signature\": \"{1}\"}", self.key, self.signature)
    }
}

public typealias DataSignature = CCardano.DataSignature

extension DataSignature: CType {}

extension DataSignature {
    public init(signature: CData, key: CData) {
        self.init()
        self.key = key
        self.signature = signature
    }
    
    public func verify() throws -> Bool {
        //TODO: This needs to be implemented!
        true
    }
    
    public func toCip30DataSignature() -> Cip30DataSignature {
        return Cip30DataSignature(ds: self)
    }
}

