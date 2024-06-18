use std::ops::Deref;

use crate::ptr::Ptr;
use crate::{array::CArray, data::CData, option::COption, ptr::Free, string::CharPtr};

use blake2::digest::{Update, VariableOutput};
use blake2::Blake2bVar;
use cml_chain::address::Address;
use cml_chain::transaction::Transaction;
use cml_chain::NonemptySetTransactionInput;
use cml_crypto::chain_crypto::bech32::{to_bech32_from_bytes, Bech32};
use cml_crypto::RawBytesEncoding;
pub mod chain_helper;
pub mod transaction;
pub mod tx_input_details;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CmlAsset {
    fingerprint: CharPtr,
    policy: CData,
    name: CData,
    #[allow(dead_code)]
    qty: u64,
}

pub type CmlAssets = CArray<CmlAsset>;

impl Free for CmlAsset {
    unsafe fn free(&mut self) {
        self.policy.free();
        self.name.free();
        self.fingerprint.free();
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CmlValue {
    #[allow(dead_code)]
    lovelace: u64,
    assets: CmlAssets,
}

impl Free for CmlValue {
    unsafe fn free(&mut self) {
        self.assets.free();
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CmlTxOutput {
    address: CharPtr,
    value: CmlValue,
    cbor: CData,
}

impl Free for CmlTxOutput {
    unsafe fn free(&mut self) {
        self.address.free();
        self.value.free();
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CmlUTxO {
    tx_hash: CData,
    #[allow(dead_code)]
    tx_index: u64,
    orig_output: COption<CmlTxOutput>, // We may have this or we may not!
}

impl CmlUTxO {
    // fn get_value(&self) -> Option<Value> {
    //     match self.orig_output {
    //         COption::Some(output) => Option::Some(output.value),
    //         COption::None => Option::None,
    //     }
    // }

    fn get_signing_hash(&self) -> Option<Vec<u8>> {
        match self.orig_output {
            COption::Some(output) => {
                let address = unsafe { output.address.unowned() }.ok()?;

                let address = Address::from_bech32(&address).ok()?;

                let x = match address {
                    Address::Reward(_) => address.staking_cred()?,
                    _ => address.payment_cred()?,
                };

                Option::Some(x.to_raw_bytes().to_owned())
            }
            COption::None => Option::None,
        }
    }
}

impl Free for CmlUTxO {
    unsafe fn free(&mut self) {
        self.tx_hash.free();
        self.orig_output.free();
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CmlTxSummarised {
    stake_address: CharPtr,
    value: CmlValue,
}

impl Free for CmlTxSummarised {
    unsafe fn free(&mut self) {
        self.stake_address.free();
        self.value.free();
    }
}

pub type CmlTxSummaries = CArray<CmlTxSummarised>;

pub type CmlUTxOs = CArray<CmlUTxO>;
pub type CmlTxOutputs = CArray<CmlTxOutput>;
#[allow(dead_code)]
pub type CmlSigners = CArray<CData>;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct TxDetails {
    #[allow(dead_code)]
    fee: u64,
    hash: CData,
    inputs: CmlUTxOs,
    collateral: COption<CmlUTxOs>,
    collateral_output: COption<CmlTxOutput>,
    signers: CmlSigners,
    outputs: CmlTxOutputs,
    //
    sum_outputs: CmlTxSummaries,
    sum_inputs: CmlTxSummaries,
}

impl Free for TxDetails {
    unsafe fn free(&mut self) {
        self.hash.free();
        self.inputs.free();
        self.outputs.free();
        self.collateral.free();
        self.collateral_output.free();
        self.signers.free();
    }
}

pub struct AssetName<'a> {
    policy_id: &'a Vec<u8>,
    asset_name: &'a Vec<u8>,
}

impl<'a> Bech32 for AssetName<'a> {
    const BECH32_HRP: &'static str = "asset";

    fn to_bech32_str(&self) -> String {
        let mut hasher = Blake2bVar::new(20).unwrap();

        hasher.update(
            self.policy_id
                .iter()
                .chain(self.asset_name.iter())
                .collect::<Vec<_>>()
                .iter()
                .map(|&&x| x)
                .collect::<Vec<u8>>()
                .as_slice(),
        );

        let mut buf = [0u8; 20];

        hasher.finalize_variable(&mut buf).unwrap();

        let b32: String = to_bech32_from_bytes::<Self>(&buf);

        b32
    }

    fn try_from_bech32_str(_: &str) -> cml_crypto::chain_crypto::bech32::Result<Self> {
        todo!()
    }
}

#[derive(Clone)]
pub struct VecUtxo(Vec<CmlUTxO>);

impl Deref for VecUtxo {
    type Target = Vec<CmlUTxO>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<NonemptySetTransactionInput> for VecUtxo {
    fn from(value: NonemptySetTransactionInput) -> Self {
        VecUtxo(
            value
                .iter()
                .map(|tx_input| tx_input.clone().into())
                .collect::<Vec<CmlUTxO>>(),
        )
    }
}

impl From<VecUtxo> for CmlUTxOs {
    fn from(value: VecUtxo) -> Self {
        Into::<CArray<CmlUTxO>>::into(value.iter().map(|utxo| utxo.clone()).collect::<Vec<_>>())
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct MempoolUtxos {
    spent_inputs: CArray<CmlUTxO>,
    created_utxos: CArray<CmlUTxO>,
}

impl MempoolUtxos {
    fn from_tx(tx: &Transaction) -> Self {
        let body = tx.body.clone();
        let tx_hash: CData = body.hash().to_raw_bytes().into();
        let inputs = body.inputs;
        let outputs = body.outputs;

        Self {
            spent_inputs: inputs
                .iter()
                .map(|input| CmlUTxO {
                    tx_hash: input.transaction_id.to_raw_bytes().into(),
                    tx_index: input.index,
                    orig_output: COption::None,
                })
                .collect::<Vec<_>>()
                .into(),

            created_utxos: outputs
                .iter()
                .enumerate()
                .map(|(idx, output)| CmlUTxO {
                    tx_hash,
                    tx_index: idx as u64,
                    orig_output: Option::Some(output.clone().into()).into(),
                })
                .collect::<Vec<_>>()
                .into(),
        }
    }
}
