extern crate serde_json;

use super::{CmlAsset, CmlAssets, CmlTxOutput, CmlUTxO, CmlValue, VecUtxo};
use crate::{error::CError, string::IntoCString};
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};

pub fn get_tx_input_details(tx_inputs: &[CmlUTxO]) -> Result<VecUtxo, CError> {
    let client = reqwest::blocking::Client::new();

    let body = tx_inputs
        .iter()
        .map(|input| input.get_tx_id())
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;

    // let body =
    //     serde_json::to_string(&body).map_err(|e| CError::Error(e.to_string().into_cstr()))?;

    let response = client
        .post("https://api.tokeopay.io/api/v1/tx/tx_input_to_utxo") // "https://api.tokeopay.io/api/tx/tx_input_to_utxo")
        .json(&body)
        .header("Content-Type", "application/json")
        .send()
        .map_err(|e| {
            println!("HTTP Error {:?}", e);

            CError::Error(e.to_string().into_cstr())
        })?;

    // println!("REsposne {:?}", response);
    // Print the entire response object for debugging

    let r = response.json::<Vec<UTxOResponse>>().map_err(|e| {
        println!("Mapping Error: {:?}", e);
        CError::Error(e.to_string().into_cstr())
    })?;

    Ok(r.try_into()?)
}

#[derive(Serialize, Deserialize)]
struct AssetResponse {
    fingerprint: String,
    name: String,
    policy: String,
    qty: String,
}
impl TryFrom<AssetResponse> for CmlAsset {
    type Error = CError;

    fn try_from(value: AssetResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            fingerprint: value.fingerprint.into_cstr(),
            name: value.name.as_bytes().into(),
            policy: hex::decode(value.policy)
                .map_err(|e| {
                    print!("{:?}", e);
                    CError::DeserializeError("Unable to decode Policy".into_cstr())
                })?
                .into(),
            qty: value
                .qty
                .parse()
                .or_else(|_| Err(CError::Error("Qry Parse".into_cstr())))?,
        })
    }
}

impl TryFrom<Vec<AssetResponse>> for CmlAssets {
    type Error = CError;

    fn try_from(value: Vec<AssetResponse>) -> Result<Self, Self::Error> {
        Ok(value
            .into_iter()
            .map(|asset| TryInto::<CmlAsset>::try_into(asset))
            .collect::<Result<Vec<_>, CError>>()?
            .into())
    }
}

#[derive(Serialize, Deserialize)]
struct ValueResponse {
    lovelace: String,
    assets: Vec<AssetResponse>,
}

impl TryFrom<ValueResponse> for CmlValue {
    type Error = CError;

    fn try_from(value: ValueResponse) -> Result<Self, Self::Error> {
        let assets = value.assets.try_into()?;

        let lovelace = value.lovelace.parse::<u64>().map_err(|e| {
            print!("{:?}", e);
            CError::Error(e.to_string().into_cstr())
        })?;

        Ok(Self { lovelace, assets })
    }
}

#[derive(Serialize, Deserialize)]
struct TxOutputResponse {
    address: String,
    value: ValueResponse,
    cbor: String,
}

impl TryFrom<TxOutputResponse> for CmlTxOutput {
    type Error = CError;

    fn try_from(value: TxOutputResponse) -> Result<Self, Self::Error> {
        let v: CmlValue = value.value.try_into()?;
        let cbor =
            hex::decode(value.cbor).map_err(|err| CError::Error(err.to_string().into_cstr()))?;

        Ok(Self {
            address: value.address.into_cstr(),
            value: v,
            cbor: cbor.into(),
        })
    }
}

#[derive(Serialize, Deserialize)]
struct UTxOResponse {
    #[serde(rename(deserialize = "txHash"))]
    tx_hash: String,
    index: u32,
    utxo: TxOutputResponse,
}

impl TryFrom<UTxOResponse> for CmlUTxO {
    type Error = CError;

    fn try_from(value: UTxOResponse) -> Result<Self, Self::Error> {
        let tx_hash =
            hex::decode(value.tx_hash).map_err(|e| CError::Error(e.to_string().into_cstr()))?;

        let utxo: CmlTxOutput = value.utxo.try_into()?;

        Ok(Self {
            tx_hash: tx_hash.into(),
            tx_index: value.index.into(),
            orig_output: crate::option::COption::Some(utxo),
        })
    }
}

impl TryFrom<Vec<UTxOResponse>> for VecUtxo {
    type Error = CError;

    fn try_from(value: Vec<UTxOResponse>) -> Result<Self, Self::Error> {
        value
            .into_iter()
            .map(|x| TryInto::<CmlUTxO>::try_into(x))
            .collect::<Result<Vec<_>, _>>()
            .map(|r| VecUtxo(r))
    }
}
