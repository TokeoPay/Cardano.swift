use std::{
    cmp::Ordering,
    collections::HashSet,
    convert::{TryFrom, TryInto},
    hash::Hash,
    ops::Deref,
    panic::catch_unwind,
};

use cardano_serialization_lib::utils::{
        TransactionUnspentOutput as RTransactionUnspentOutput,
        TransactionUnspentOutputs as RTransactionUnspentOutputs
    };
use cml_chain::assets::Value as XCML_Value;
use cml_chain::{
    assets::AssetName as CML_AssetName,
    transaction::{TransactionInput as CML_TxIn, TransactionOutput as CML_TxOut},
};
use cml_chain::{
    builders::tx_builder::TransactionUnspentOutput as XCML_TransactionUnspentOutput, Deserialize,
};
use cml_core::serialization::Serialize;
use cml_crypto::ScriptHash as CML_ScriptHash;
use cml_crypto::TransactionHash as CML_TxHash;

use crate::{
    array::CArray,
    data::CData,
    error::CError,
    panic::*,
    ptr::{Free, Ptr},
    string::{CharPtr, IntoCString},
    transaction_input::TransactionInput,
    transaction_output::TransactionOutput,
    value::Value,
};

type CMLValue = XCML_Value;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct TransactionUnspentOutput {
    input: TransactionInput,
    output: TransactionOutput,
}

impl Free for TransactionUnspentOutput {
    unsafe fn free(&mut self) {
        self.output.free()
    }
}

impl TryFrom<TransactionUnspentOutput> for RTransactionUnspentOutput {
    type Error = CError;

    fn try_from(transaction_unspent_output: TransactionUnspentOutput) -> Result<Self> {
        transaction_unspent_output
            .output
            .try_into()
            .map(|output| Self::new(&transaction_unspent_output.input.into(), &output))
    }
}

impl TryFrom<RTransactionUnspentOutput> for TransactionUnspentOutput {
    type Error = CError;

    fn try_from(transaction_unspent_output: RTransactionUnspentOutput) -> Result<Self> {
        transaction_unspent_output
            .input()
            .try_into()
            .zip(transaction_unspent_output.output().try_into())
            .map(|(input, output)| Self { input, output })
    }
}

#[no_mangle]
pub unsafe extern "C" fn cardano_transaction_unspent_output_clone(
    transaction_unspent_output: TransactionUnspentOutput,
    result: &mut TransactionUnspentOutput,
    error: &mut CError,
) -> bool {
    handle_exception(|| transaction_unspent_output.clone()).response(result, error)
}

#[no_mangle]
pub unsafe extern "C" fn cardano_transaction_unspent_output_free(
    transaction_unspent_output: &mut TransactionUnspentOutput,
) {
    transaction_unspent_output.free();
}

pub type TransactionUnspentOutputs = CArray<TransactionUnspentOutput>;

impl TryFrom<TransactionUnspentOutputs> for RTransactionUnspentOutputs {
    type Error = CError;

    fn try_from(transaction_unspent_outputs: TransactionUnspentOutputs) -> Result<Self> {
        let vec = unsafe { transaction_unspent_outputs.unowned()? };
        let mut transaction_unspent_outputs = Self::new();
        for transaction_unspent_output in vec.to_vec() {
            let transaction_unspent_output = transaction_unspent_output.try_into()?;
            transaction_unspent_outputs.add(&transaction_unspent_output);
        }
        Ok(transaction_unspent_outputs)
    }
}

impl TryFrom<RTransactionUnspentOutputs> for TransactionUnspentOutputs {
    type Error = CError;

    fn try_from(transaction_unspent_outputs: RTransactionUnspentOutputs) -> Result<Self> {
        (0..transaction_unspent_outputs.len())
            .map(|index| transaction_unspent_outputs.get(index))
            .map(|transaction_unspent_output| transaction_unspent_output.try_into())
            .collect::<Result<Vec<TransactionUnspentOutput>>>()
            .map(|transaction_unspent_outputs| transaction_unspent_outputs.into())
    }
}


#[repr(C)]
#[derive(Clone, Copy)]
pub struct SwiftUTXO {
    transaction_hash: CharPtr,
    transaction_index: u64,
    tx_out_bytes: CData,
    // utxo_bytes: COption<CData>,
}

impl Free for SwiftUTXO {
    unsafe fn free(&mut self) {
        self.transaction_hash.free();
        self.tx_out_bytes.free();
    }
}

impl TryInto<CMLTransactionUnspentOutput> for SwiftUTXO {
    type Error = CError;

    fn try_into(self) -> std::prelude::v1::Result<CMLTransactionUnspentOutput, Self::Error> {
        return catch_unwind(|| unsafe { self.tx_out_bytes.unowned() })
            .into_result()
            .and_then(|r| r)
            .zip(
                catch_unwind(|| unsafe { self.transaction_hash.unowned() })
                    .into_result()
                    .and_then(|r| r)
                    .and_then(|tx_hash| {
                        CML_TxHash::from_hex(tx_hash)
                            .map_err(|err| CError::Error(err.to_string().into_cstr()))
                    }),
            )
            .and_then(|(tx_out_bytes, tx_hash)| {
                let tx_input = CML_TxIn::new(tx_hash, self.transaction_index);
                CML_TxOut::from_cbor_bytes(tx_out_bytes)
                    .and_then(|tx_out| {
                        Ok(CMLTransactionUnspentOutput(
                            XCML_TransactionUnspentOutput::new(tx_input, tx_out),
                        ))
                    })
                    .map_err(|err| CError::Error(err.to_string().into_cstr()))
            });
    }
}

impl Into<SwiftUTXO> for CMLTransactionUnspentOutput {
    fn into(self) -> SwiftUTXO {
        let output_bytes = Serialize::to_cbor_bytes(&self.output);
        // let utxo_bytes = Serialize::to_cbor_bytes(&self);

        SwiftUTXO {
            transaction_hash: self.input.transaction_id.to_hex().into_cstr(),
            transaction_index: self.input.index,
            tx_out_bytes: output_bytes.into(),
            // utxo_bytes: COption::Some(self.to_bytes())
        }
    }
}

pub type SwiftUTXOs = CArray<SwiftUTXO>;

#[derive(Clone)]
struct CMLTransactionUnspentOutput(XCML_TransactionUnspentOutput);
type CMLTransactionUnspentOutputs = Vec<CMLTransactionUnspentOutput>;

impl TryFrom<SwiftUTXOs> for CMLTransactionUnspentOutputs {
    type Error = CError;

    fn try_from(swift_utxos: SwiftUTXOs) -> Result<Self> {
        let vec = unsafe { swift_utxos.unowned()? };

        let mut utxos = Self::new();

        for swift_utxo in vec.to_vec() {
            let utxo = swift_utxo.try_into()?;
            utxos.push(utxo);
        }

        return Ok(utxos);
    }
}

impl Deref for CMLTransactionUnspentOutput {
    type Target = XCML_TransactionUnspentOutput;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Eq for CMLTransactionUnspentOutput {}
impl PartialEq for CMLTransactionUnspentOutput {
    fn eq(&self, other: &Self) -> bool {
        self.input.eq(&other.input)
    }
}
impl Hash for CMLTransactionUnspentOutput {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.input.hash(state);
    }
}

fn available_lovelace(utxo: &CMLTransactionUnspentOutput, coins_per_byte: &u64) -> u64 {
    let min_ll: u64 =
        Serialize::to_canonical_cbor_bytes(&utxo.output).len() as u64 * coins_per_byte;

    return utxo.output.amount().coin - min_ll;
}

fn have_enough(
    selected_utxos: &HashSet<CMLTransactionUnspentOutput>,
    want: &u64,
    policy_id: &CML_ScriptHash,
    asset_name: &CML_AssetName,
) -> bool {
    let have = selected_utxos.iter().fold(0 as u64, |acc, utxo| {
        return acc
            + utxo
                .output
                .amount()
                .multiasset
                .get(policy_id, asset_name)
                .unwrap_or(0);
    });

    have.ge(want)
}


#[repr(C)]
#[derive(Copy, Clone)]
pub struct CoinSelectionResult {
    selected: SwiftUTXOs,
    other: SwiftUTXOs
}

#[no_mangle]
pub unsafe extern "C" fn cardano_transaction_unspent_outputs_coin_selection(
    transaction_unspent_outputs: CArray<SwiftUTXO>,
    value: Value,
    result: &mut CoinSelectionResult,
    error: &mut CError,
) -> bool {
    let coins_per_byte = 4310 as u64; //FIXME: Make this a parameter!

    handle_exception_result(|| {
        TryInto::<Vec<CMLTransactionUnspentOutput>>::try_into(transaction_unspent_outputs)
            .zip(TryInto::<CMLValue>::try_into(value))
            .and_then(|(utxos, target)| {
                let mut selected_utxos: HashSet<CMLTransactionUnspentOutput> = HashSet::new(); //: Vec<RTransactionUnspentOutput> = Vec::with_capacity(utxos.len());
                let mut ordered_utxos: Vec<CMLTransactionUnspentOutput> = utxos.clone();

                let assets_covered = target.multiasset.iter().all(|(policy_id, asset_map)| {
                    asset_map.iter().all(|(asset_name, want)| {
                        ordered_utxos.sort_by(|a, b| {
                            let a_asset = a.output.amount().multiasset.get(&policy_id, &asset_name);
                            let b_asset = b.output.amount().multiasset.get(&policy_id, &asset_name);

                            match (a_asset, b_asset) {
                                (None, None) => Ordering::Equal,
                                (None, Some(_)) => Ordering::Less,
                                (Some(_), None) => Ordering::Greater,
                                (Some(aa), Some(bb)) => aa.cmp(&bb),
                            }
                        });

                        ordered_utxos
                            .as_slice()
                            .iter()
                            .filter(|utxo| {
                                utxo.output
                                    .amount()
                                    .multiasset
                                    .get(&policy_id, &asset_name)
                                    .unwrap_or(0)
                                    > 0
                            })
                            .any(|utxo| {
                                selected_utxos.insert(utxo.clone());
                                return have_enough(
                                    &selected_utxos,
                                    &want,
                                    &policy_id,
                                    &asset_name,
                                );
                            })
                    })
                });

                match assets_covered {
                    false => {
                        return Err(CError::CoinSelectionInsufficientFunds(
                            "Inputs exhausted - Assets".into_cstr(),
                        ))
                    }
                    true => (),
                }

                let lovelace_selected = selected_utxos.iter().fold(0 as u64, |acc, utxo| {
                    return acc + utxo.output.amount().coin;
                });

                if target.coin.gt(&lovelace_selected) {
                    // We need some ADA!
                    let mut ordered_utxos: Vec<CMLTransactionUnspentOutput> =
                        Vec::with_capacity(utxos.len());
                    utxos
                        .iter()
                        .filter(|utxo| selected_utxos.contains(&utxo))
                        .for_each(|utxo| {
                            ordered_utxos.push(utxo.clone());
                        });

                    ordered_utxos.sort_by(|a, b| {
                        available_lovelace(a, &coins_per_byte)
                            .cmp(&available_lovelace(b, &coins_per_byte))
                    });

                    ordered_utxos.as_slice().iter().any(|utxo| {
                        selected_utxos.insert(utxo.clone());
                        let lovelace_selected =
                            selected_utxos.iter().fold(0 as u64, |acc, utxo| {
                                return acc + utxo.output.amount().coin;
                            });

                        target.coin.le(&lovelace_selected)
                    });

                    let lovelace_selected = selected_utxos.iter().fold(0 as u64, |acc, utxo| {
                        return acc + utxo.output.amount().coin;
                    });

                    if target.coin.gt(&lovelace_selected) {
                        return Err(CError::CoinSelectionInsufficientFunds(
                            "Not enough Lovelace in UTxO set".into_cstr(),
                        ));
                    }
                }

                let mut result_selected_utxos: Vec<CMLTransactionUnspentOutput> = Vec::new();
                let mut result_remaining_utxos: Vec<CMLTransactionUnspentOutput> = Vec::new();
                selected_utxos
                    .iter()
                    .for_each(|x| result_selected_utxos.push(x.clone()));
                ordered_utxos
                    .as_slice()
                    .iter()
                    .filter(|utxo| !selected_utxos.contains(utxo))
                    .for_each(|utxo| {
                        result_remaining_utxos.push(utxo.clone());
                    });

                let res = TryInto::<SwiftUTXOs>::try_into(result_selected_utxos)
                    .map_err(|_| CError::Error("".into_cstr()));
                let rem = TryInto::<SwiftUTXOs>::try_into(result_remaining_utxos)
                    .map_err(|_| CError::Error("".into_cstr()));

                res.zip(rem).and_then(|(selected, other)| {
                    Ok(CoinSelectionResult {
                        selected,
                        other
                    })
                })
            })
    })
    .response(result, error)
}

#[no_mangle]
pub unsafe extern "C" fn cardano_swift_utxo_free(
  swift_utxo: &mut SwiftUTXO,
) {
    swift_utxo.free();
}


#[no_mangle]
pub unsafe extern "C" fn cardano_transaction_unspent_outputs_free(
    transaction_unspent_outputs: &mut TransactionUnspentOutputs,
) {
    transaction_unspent_outputs.free();
}
