//
//  DataSignature.swift
//
//
//  Created by Gavin Harris on 27/4/2024.
//

import Foundation
import CCardano

class Cip30DataSignatureError: Error {
    public let message: String
    
    init(message: String) {
        self.message = message
    }
}

public struct Cip30DataSignature: Codable {
    public let signature: String
    public let key: String
    
    init(ds: DataSignature) {
        self.key = ds.key.copied().hex(prefix: false)
        self.signature = ds.signature.copied().hex(prefix: false)
    }
    
    public func toJson() -> Result<String, Error> {
        let encoder = JSONEncoder()
        do {
            let data = try encoder.encode(self)
            
            if let str = String(data: data, encoding: .utf8) {
                return Result.success(str)
            }
            
            return Result.failure(Cip30DataSignatureError(message: "Cannot parse to JSON"));

        } catch (let err) {
            return Result.failure(err)
        }
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

