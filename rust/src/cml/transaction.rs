use std::collections::HashSet;
use std::iter::FromIterator;

use super::tx_input_details::get_tx_input_details;
use super::{
    AssetName, CmlAsset, CmlAssets, CmlTxOutput, CmlTxSummarised, CmlUTxO, CmlUTxOs, CmlValue,
    TxDetails, VecUtxo,
};
use crate::error::CError;
use crate::panic::{handle_exception_result, CResponse};

#[allow(unused_imports)]
use crate::ptr::Ptr;
use crate::string::CharPtr;
use crate::{data::CData, option::COption, string::IntoCString};
#[allow(unused_imports)]
use blake2::{Blake2b, Digest};
use cml_chain::address::{Address, RewardAddress};
use cml_chain::assets::{Mint, MultiAsset, Value as CML_Value};
use cml_chain::builders::tx_builder::TransactionUnspentOutput;
use cml_chain::builders::witness_builder::TransactionWitnessSetBuilder;
use cml_chain::crypto::utils::make_vkey_witness;
use cml_chain::min_ada::min_ada_required;
use cml_chain::transaction::{
    Transaction, TransactionInput, TransactionOutput, TransactionWitnessSet,
};
use cml_chain::{Deserialize, NonemptySetTransactionInput};
#[allow(unused_imports)]
use cml_core::serialization::FromBytes;
use cml_core::serialization::Serialize;
use cml_crypto::chain_crypto::bech32::Bech32;
use cml_crypto::{PrivateKey, RawBytesEncoding, TransactionHash};

/* End of Imports */

impl CmlUTxO {
    pub fn get_tx_id(&self) -> Result<String, CError> {
        let tx_hash = unsafe { self.tx_hash.unowned() }?;
        let tx_hash = hex::encode(tx_hash);
        let index = self.tx_index;
        Ok(format!("{tx_hash:}.{index:}"))
    }
}

impl From<TransactionInput> for CmlUTxO {
    fn from(value: TransactionInput) -> Self {
        Self {
            tx_hash: value.transaction_id.to_raw_bytes().into(),
            tx_index: value.index,
            orig_output: Option::None.into(),
        }
    }
}

impl From<MultiAsset> for CmlAssets {
    fn from(ma: MultiAsset) -> Self {
        let mut my_assets: Vec<CmlAsset> = Vec::new();

        ma.iter().for_each(|(policy_id, assets)| {
            assets.iter().for_each(|(asset_name, amount)| {
                let a = AssetName {
                    policy_id: &policy_id.to_raw_bytes().to_vec(),
                    asset_name: asset_name.get(),
                };

                my_assets.push(CmlAsset {
                    fingerprint: a.to_bech32_str().into_cstr(),
                    name: asset_name.get().clone().into(),
                    policy: policy_id.to_raw_bytes().into(),
                    qty: amount.clone() as i64,
                });
            });
        });

        Into::<CmlAssets>::into(my_assets)
    }
}

impl From<CML_Value> for CmlValue {
    fn from(value: CML_Value) -> Self {
        Self {
            lovelace: value.coin,
            assets: value.multiasset.into(),
        }
    }
}

impl From<TransactionOutput> for CmlTxOutput {
    fn from(value: TransactionOutput) -> Self {
        Self {
            address: value
                .address()
                .to_bech32(Option::None)
                .unwrap_or("<Address Error>".to_ascii_lowercase())
                .into_cstr(),
            value: Into::<CmlValue>::into(value.amount().clone()),
            cbor: value.to_cbor_bytes().into(),
        }
    }
}

impl From<Option<NonemptySetTransactionInput>> for COption<CmlUTxOs> {
    fn from(value: Option<NonemptySetTransactionInput>) -> Self {
        value.into()
    }
}

impl From<NonemptySetTransactionInput> for CmlUTxOs {
    fn from(value: NonemptySetTransactionInput) -> Self {
        value
            .iter()
            .map(|tx_input| tx_input.clone().into())
            .collect::<Vec<CmlUTxO>>()
            .into()
    }
}

impl From<TransactionUnspentOutput> for CmlUTxO {
    fn from(value: TransactionUnspentOutput) -> Self {
        Self {
            tx_hash: value.input.transaction_id.to_raw_bytes().into(),
            tx_index: value.input.index,
            orig_output: COption::Some(value.output.into()),
        }
    }
}

impl From<Transaction> for TxDetails {
    fn from(value: Transaction) -> Self {
        let body = value.body;
        let mints = body.mint.clone();
        let inputs = body.inputs.clone();
        let fee = body.fee;
        let collateral = body.collateral_inputs.clone();
        let collateral_return = body.collateral_return.clone();
        let outputs = body.outputs.clone();
        let hash = body.hash();
        let certs = body.certs.clone();
        let required_signers = body.required_signers.clone();

        println!("Convert UTxOs");
        let utxos: VecUtxo = Into::<VecUtxo>::into(inputs);
        println!("Convert UTxOs after");
        let tx_input_utxos: VecUtxo = get_tx_input_details(&utxos).ok().unwrap_or(utxos);

        let c_utxos: Option<VecUtxo> = collateral.map(Into::<VecUtxo>::into);
        let tx_c_input_utxos = c_utxos
            .clone()
            .map(|c| get_tx_input_details(&c).ok())
            .unwrap_or(c_utxos);

        // let tx_collateral: Option<UTxOs> = match collateral {
        //     Some(c_inputs) => {
        //         let utxos: VecUtxo = Into::<VecUtxo>::into(c_inputs);
        //         let c_utxos = get_tx_input_details(&utxos).ok();

        //         let y: Option<UTxOs> = c_utxos.map(|u| u.into());

        //         Option::Some(y.unwrap_or(utxos.into()))
        //     }
        //     None => Option::None,
        // };

        //         inputs.into_iter().map(|input| {
        //            let address = input ;
        //        });

        // let x: Vec<TxSummarised> = outputs

        let sum_outputs = outputs
            .to_vec()
            .iter()
            .map(|o| {
                (
                    o,
                    match o.address().staking_cred() {
                        Some(staking_cred) => RewardAddress::new(0, staking_cred.clone())
                            .to_address()
                            .to_bech32(Option::None)
                            .map_err(|e| CError::Error(e.to_string().into_cstr())),
                        None => o
                            .address()
                            .to_bech32(Option::None)
                            .map_err(|e| CError::Error(e.to_string().into_cstr())),
                    },
                )
            })
            .map(|(output, addr)| {
                let a = addr.unwrap_or("<BAD ADDRESS>".to_string());

                return CmlTxSummarised {
                    stake_address: a.into_cstr(),
                    value: Into::<CmlValue>::into(output.amount().clone()),
                };
            })
            .collect::<Vec<CmlTxSummarised>>();

        // fn get_reward_address(address: &Address) -> &str {
        //     match address.staking_cred() {
        //         Some(staking_cred) => {
        //             let cred = staking_cred.clone();
        //             RewardAddress::new(0, cred)
        //                 .to_address()
        //                 .to_bech32(Option::None)
        //                 .ok()
        //                 .as_str()
        //         }
        //         None => address_str,
        //     }
        // }

        let sum_inputs = tx_input_utxos
            .iter()
            .map(|utxo| match utxo.orig_output {
                COption::Some(o) => {
                    let address_str = unsafe { o.address.unowned() }.ok()?;
                    let address = Address::from_bech32(address_str).ok()?;

                    let address_str = match address.staking_cred() {
                        Some(staking_cred) => {
                            let cred = staking_cred.clone();
                            let reward_addr = RewardAddress::new(0, cred);
                            let addr = reward_addr.to_address();

                            addr.to_bech32(Option::None)
                                .ok()
                                .unwrap_or(address_str.into())
                        }
                        None => address_str.into(),
                    };

                    Option::Some(CmlTxSummarised {
                        stake_address: address_str.into_cstr(),
                        value: Into::<CmlValue>::into(o.value),
                    })
                }
                COption::None => Option::None,
            })
            .collect::<Option<Vec<CmlTxSummarised>>>()
            .unwrap_or(Vec::new());

        let c_signers: Vec<String> = tx_c_input_utxos
            .as_ref()
            .map(|x| {
                x.iter()
                    .map(|utxo| match utxo.get_signing_hash() {
                        Some(hash) => Some(hex::encode(hash)),
                        None => None,
                    })
                    .collect::<Option<Vec<_>>>()
                    .unwrap_or(Vec::new())
            })
            .unwrap_or(Vec::new());

        let mut signers: Vec<String> = tx_input_utxos // This needs to be replaced with `tx_input_utxos` and `tx_collateral`
            .iter()
            .map(|utxo| match utxo.get_signing_hash() {
                Some(hash) => Some(hex::encode(hash)),
                None => None,
            })
            .collect::<Option<Vec<String>>>()
            .unwrap_or(Vec::new());

        let cert_signers = certs.and_then(|certs| {
            certs
                .iter()
                .map(|cert| match cert {
                    cml_chain::certs::Certificate::StakeRegistration(s) => {
                        Some(hex::encode(s.stake_credential.to_raw_bytes().to_owned()))
                    }
                    cml_chain::certs::Certificate::StakeDeregistration(s) => {
                        Some(hex::encode(s.stake_credential.to_raw_bytes().to_owned()))
                    }
                    cml_chain::certs::Certificate::StakeDelegation(s) => {
                        Some(hex::encode(s.stake_credential.to_raw_bytes().to_owned()))
                    }
                    cml_chain::certs::Certificate::PoolRegistration(_) => None,
                    cml_chain::certs::Certificate::PoolRetirement(_) => None,
                    cml_chain::certs::Certificate::RegCert(_) => None,
                    cml_chain::certs::Certificate::UnregCert(_) => None,
                    cml_chain::certs::Certificate::VoteDelegCert(_) => None,
                    cml_chain::certs::Certificate::StakeVoteDelegCert(s) => {
                        Some(hex::encode(s.stake_credential.to_raw_bytes()))
                    }
                    cml_chain::certs::Certificate::StakeRegDelegCert(s) => {
                        Some(hex::encode(s.stake_credential.to_raw_bytes()))
                    }
                    cml_chain::certs::Certificate::VoteRegDelegCert(_) => None,
                    cml_chain::certs::Certificate::StakeVoteRegDelegCert(s) => {
                        Some(hex::encode(s.stake_credential.to_raw_bytes()))
                    }
                    cml_chain::certs::Certificate::AuthCommitteeHotCert(_) => None,
                    cml_chain::certs::Certificate::ResignCommitteeColdCert(_) => None,
                    cml_chain::certs::Certificate::RegDrepCert(_) => None,
                    cml_chain::certs::Certificate::UnregDrepCert(_) => None,
                    cml_chain::certs::Certificate::UpdateDrepCert(_) => None,
                })
                .collect::<Option<Vec<_>>>()
        });

        if let Some(required_signers) = required_signers {
            signers.extend(
                required_signers
                    .iter()
                    .map(|s| s.to_raw_hex())
                    .collect::<Vec<String>>(),
            );
        }

        if let Some(cert_signers) = cert_signers {
            signers.extend(cert_signers)
        } else {
            print!("No Cert Signers")
        };

        signers.extend(c_signers);

        let tmp_hash: HashSet<String, _> = HashSet::<String>::from_iter(signers);
        let signers: Vec<Vec<u8>> = Vec::from_iter(tmp_hash.iter())
            .iter()
            .map(|s| hex::decode(s).unwrap_or_default())
            .collect();

        let mut my_mints: COption<CmlAssets> = COption::None;

        if let Some(mints) = mints {
            let a: CmlAssets = mints
                .iter()
                .flat_map(|(policy_id, assets)| {
                    assets
                        .iter()
                        .map(|(asset_name, amount)| {
                            let a = AssetName {
                                policy_id: &policy_id.to_raw_bytes().to_vec(),
                                asset_name: asset_name.get(),
                            };

                            CmlAsset {
                                fingerprint: a.to_bech32_str().into_cstr(),
                                name: asset_name.get().clone().into(),
                                policy: policy_id.to_raw_bytes().into(),
                                qty: amount.clone(),
                            }
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
                .into();

            my_mints = COption::Some(a);
        }

        // let mints: Option<CmlAssets> =

        Self {
            inputs: tx_input_utxos.clone().into(),
            fee,
            collateral: match tx_c_input_utxos {
                Some(c_inputs) => COption::Some(c_inputs.into()),
                None => COption::None,
            },
            collateral_output: collateral_return
                .map_or_else(|| COption::None, |o| COption::Some(o.into())),
            signers: signers.into(),
            outputs: outputs.into(),
            hash: hash.to_raw_bytes().into(),
            sum_inputs: sum_inputs.into(),
            sum_outputs: sum_outputs.into(),
            mints: my_mints,
        }
    }
}

//impl<'a> AssetName<'a> {
//        fn to_bech32_str(&self) -> String {
//        println!(
//            "Policy {} asset {}",
//            hex::encode(self.policy_id.as_slice()),
//            hex::encode(self.asset_name.as_slice())
//        );
//
//        let data = blake2b224(
//            self.policy_id
//                .iter()
//                .chain(self.asset_name.iter())
//                .collect::<Vec<_>>()
//                .iter()
//                .map(|&&x| x)
//                .collect::<Vec<u8>>()
//                .as_slice(),
//        );
//
//        println!("Hash: {}", hex::encode(data));
//
//        let b32 = bech32::encode("asset", data).unwrap();
//
//        b32
//    }
//}

#[no_mangle]
pub unsafe extern "C" fn available_lovelace(
    tx_out_cbor: CData,
    coins_per_utxo_byte: u64,
    result: &mut u64,
    error: &mut CError,
) -> bool {
    handle_exception_result(|| {
        let tx_out_cbor = unsafe { tx_out_cbor.unowned() }?;
        let tx_output = Deserialize::from_cbor_bytes(tx_out_cbor)
            .map_err(|e| CError::DeserializeError(e.to_string().into_cstr()))?;

        let min_lovelace = min_ada_required(&tx_output, coins_per_utxo_byte)
            .map_err(|err| CError::Error(err.to_string().into_cstr()))?;
        let lovelace_on_output = tx_output.amount().coin;

        Ok(lovelace_on_output - min_lovelace)
    })
    .response(result, error)
}

#[no_mangle]
pub unsafe extern "C" fn utxo_from_parts(
    tx_hash: CharPtr,
    index: u64,
    tx_out_cbor: CData,
    result: &mut CmlUTxO,
    error: &mut CError,
) -> bool {
    handle_exception_result(|| {
        let tx_hash = unsafe { tx_hash.unowned() }?;
        let tx_hash = TransactionHash::from_hex(tx_hash)
            .map_err(|e| CError::DeserializeError(e.to_string().into_cstr()))?;

        let tx_out_cbor = unsafe { tx_out_cbor.unowned() }?;

        let tx_input = TransactionInput::new(tx_hash, index);

        let tx_output = Deserialize::from_cbor_bytes(tx_out_cbor)
            .map_err(|e| CError::DeserializeError(e.to_string().into_cstr()))?;

        Ok(Into::<CmlUTxO>::into(TransactionUnspentOutput::new(
            tx_input, tx_output,
        )))
    })
    .response(result, error)
}

#[no_mangle]
pub unsafe extern "C" fn cml_tx_details(
    transaction: CData,
    result: &mut TxDetails,
    error: &mut CError,
) -> bool {
    handle_exception_result(|| {
        let x = transaction
            .unowned()
            .and_then(|tx_bytes| {
                Transaction::from_bytes(tx_bytes.to_vec())
                    .map_err(|_| CError::Error("Tx Build Error".into_cstr()))
            })
            .and_then(|txn| Ok(Into::<TxDetails>::into(txn)));

        x
    })
    .response(result, error)
}

#[no_mangle]
pub unsafe extern "C" fn cml_tx_add_signers(
    transaction: CData,
    tx_witness_set: CData,
    result: &mut CData,
    error: &mut CError,
) -> bool {
    handle_exception_result(|| {
        let tx_bytes = transaction.unowned()?;
        let tx_witness_set_bytes = tx_witness_set.unowned()?;
        let tx = Transaction::from_bytes(tx_bytes.to_vec())
            .map_err(|_| CError::Error("Tx Build Error".into_cstr()))?;

        let mut tx_witness_set = tx.witness_set.clone();

        let tx_witness_to_add: TransactionWitnessSet =
            Deserialize::from_cbor_bytes(tx_witness_set_bytes).map_err(|err| {
                println!("{:?}", err);
                CError::Error("Tx Build Error - Bad Witness Set".into_cstr())
            })?;

        tx_witness_set.add_all_witnesses(tx_witness_to_add);

        let new_tx = Transaction::new(tx.body, tx_witness_set, true, tx.auxiliary_data);

        let res = Serialize::to_cbor_bytes(&new_tx);

        Ok(res.into())
    })
    .response(result, error)
}

#[no_mangle]
pub unsafe extern "C" fn cml_tx_sign(
    transaction: CData,
    private_key: CData,
    result: &mut CData,
    error: &mut CError,
) -> bool {
    handle_exception_result(|| {
        let tx_bytes = transaction.unowned()?;
        let pk_bytes = private_key.unowned()?;
        let pk = PrivateKey::from_raw_bytes(pk_bytes)
            .map_err(|_| CError::Error("Tx Build Error - Bad Private Key".into_cstr()))?;
        let tx = Transaction::from_bytes(tx_bytes.to_vec())
            .map_err(|_| CError::Error("Tx Build Error".into_cstr()))?;

        let vkey_witness = make_vkey_witness(&tx.body.hash(), &pk);
        let mut tx_witness_set = TransactionWitnessSetBuilder::new();

        tx_witness_set.add_vkey(vkey_witness);

        let res = Serialize::to_cbor_bytes(&tx_witness_set.build());

        Ok(res.into())
    })
    .response(result, error)
}

#[cfg(test)]
mod transaction_tests {
    use super::*;

    fn test_asset_fp(asset_name: &str, policy_id: &str, expected: &str) {
        let asset_name = AssetName {
            asset_name: &hex::decode(asset_name).unwrap(),
            policy_id: &hex::decode(policy_id).unwrap(),
        };

        assert_eq!(asset_name.to_bech32_str(), expected.trim());
    }

    #[test]
    fn asset_name_to_fingerprint() {
        test_asset_fp(
            "",
            "7eae28af2208be856f7a119668ae52a49b73725e326dc16579dcc373",
            "asset1rjklcrnsdzqp65wjgrg55sy9723kw09mlgvlc3",
        );
        test_asset_fp(
            "",
            "7eae28af2208be856f7a119668ae52a49b73725e326dc16579dcc37e",
            "asset1nl0puwxmhas8fawxp8nx4e2q3wekg969n2auw3",
        );
        test_asset_fp(
            "",
            "1e349c9bdea19fd6c147626a5260bc44b71635f398b67c59881df209",
            "asset1uyuxku60yqe57nusqzjx38aan3f2wq6s93f6ea",
        );
        test_asset_fp(
            "504154415445",
            "7eae28af2208be856f7a119668ae52a49b73725e326dc16579dcc373",
            "asset13n25uv0yaf5kus35fm2k86cqy60z58d9xmde92",
        );
        test_asset_fp(
            "504154415445",
            "1e349c9bdea19fd6c147626a5260bc44b71635f398b67c59881df209",
            "asset1hv4p5tv2a837mzqrst04d0dcptdjmluqvdx9k3",
        );
        test_asset_fp(
            "7eae28af2208be856f7a119668ae52a49b73725e326dc16579dcc373",
            "1e349c9bdea19fd6c147626a5260bc44b71635f398b67c59881df209",
            "asset1aqrdypg669jgazruv5ah07nuyqe0wxjhe2el6f",
        );
        test_asset_fp(
            "1e349c9bdea19fd6c147626a5260bc44b71635f398b67c59881df209",
            "7eae28af2208be856f7a119668ae52a49b73725e326dc16579dcc373",
            "asset17jd78wukhtrnmjh3fngzasxm8rck0l2r4hhyyt",
        );
        test_asset_fp(
            "0000000000000000000000000000000000000000000000000000000000000000",
            "7eae28af2208be856f7a119668ae52a49b73725e326dc16579dcc373",
            "asset1pkpwyknlvul7az0xx8czhl60pyel45rpje4z8w",
        );
    }

    #[test]
    fn test_tx_details() {
        env_logger::init();
        let tx = "84a9008382582048be06cebaaed874f6a27ee61d7f2a18cd7efbbcc0c0708c73566825890700e60082582088dc1916f3579e97f41e5e0f4ba9c245e6179608f19f7e680d04ca8e8000a8970182582017ff24307d62c71cf06e7d9b7fe26e1eabf1af190204b819bbf714f03b30c9af0101868258390170e60f3b5ea7153e0acc7a803e4401d44b8ed1bae1c7baaad1a62a721e78aae7c90cc36d624f7b3bb6d86b52696dc84e490f343eba89005f1a000f4240825839019737419ac8cf4a69ac64440a1a734c2c50b18d423e499f289a5267d4bf1aacd8be15b54f3cb0837e92302e2040c89718973b5d8f763a03931a000f424082583901d2f63bb93a46252714598085271f2368e8669f8aaf8f5fab4e4fc2f5d6a3bf692f79c73710e92f431adf0bcb4aa17ecb0b3b126bb1defa5d1a002dc6c082583901a298fd66dc9060b11f0a8679cd8516ef14c31236c2dfb1a2ae933788131f44c4a43b3c6ea5b2aaf660d167f894fa133ae33d5cdc62a3d9f0821a002df852a7581c279c909f348e533da5808898f87f9a14bb2c3dfbbacccd631d927a3fa144534e454b1930ad581c693c3defceb1b7d27d1bf91f52b65b59b11b66b066d10c8d6d461f4ca154000de140515244414e4f5345525645523033323201581c77999d5a1e09f9bdc16393cab713f26345dc0827a9e5134cf0f9da37a24c4d756c67614b6f6e67333731014d4d756c67614b6f6e673338313901581c95a427e384527065f2f8946f5e86320d0117839a5e98ea2c0b55fb00a14448554e541a06052340581ca5b2464b242dcbc97c1f65e85753b04ba979645f693a806a394a4931a1494d69644b6e696768741a867e44fc581ca7bf4ce10dca4f5f99b081c4ea84e0e3f919775b953324e09edea852a14d536865446576696c733131363601581cba92e5f4665a026f7d5f2f223d398d2d8b649e147b5163b759bd61a0ad4a54696765727a32373739014a54696765727a33383630014a54696765727a33383631014a54696765727a33383632014a54696765727a33383633014a54696765727a33383634014a54696765727a33383635014a54696765727a33383636014a54696765727a33383637014a54696765727a33383638014a54696765727a33383639014a54696765727a33383730014a54696765727a333837310182583901a298fd66dc9060b11f0a8679cd8516ef14c31236c2dfb1a2ae933788131f44c4a43b3c6ea5b2aaf660d167f894fa133ae33d5cdc62a3d9f0821a001898a4a3581cba92e5f4665a026f7d5f2f223d398d2d8b649e147b5163b759bd61a0a24a54696765727a33383732014a54696765727a3532303301581cbb143df7e6472b158014023d8a1c592d38be8771ce4c01f4fcd65c63a148323630313235363301581cc72d0438330ed1346f4437fcc1c263ea38e933c1124c8d0f2abc6312a2484b5749433034303501484b574943313230340182583901a298fd66dc9060b11f0a8679cd8516ef14c31236c2dfb1a2ae933788131f44c4a43b3c6ea5b2aaf660d167f894fa133ae33d5cdc62a3d9f01a032e7f60021a0007465d031a074b3efa0b58204bcd043768b51437bc8087ffe6b740ada67f7a899d12c1445d28a00808ea79d80d8182582017ff24307d62c71cf06e7d9b7fe26e1eabf1af190204b819bbf714f03b30c9af011082583901a298fd66dc9060b11f0a8679cd8516ef14c31236c2dfb1a2ae933788131f44c4a43b3c6ea5b2aaf660d167f894fa133ae33d5cdc62a3d9f01a036900e0111a000ae98c12818258209a32459bd4ef6bbafdeb8cf3b909d0e3e2ec806e4cc6268529280b0fc1d06f5b00a2049fd8799f581cd2f63bb93a46252714598085271f2368e8669f8aaf8f5fab4e4fc2f59fd8799fd8799fd8799f581c70e60f3b5ea7153e0acc7a803e4401d44b8ed1bae1c7baaad1a62a72ffd8799fd8799fd8799f581c1e78aae7c90cc36d624f7b3bb6d86b52696dc84e490f343eba89005fffffffffa140d8799f00a1401a000f4240ffffd8799fd8799fd8799f581c9737419ac8cf4a69ac64440a1a734c2c50b18d423e499f289a5267d4ffd8799fd8799fd8799f581cbf1aacd8be15b54f3cb0837e92302e2040c89718973b5d8f763a0393ffffffffa140d8799f00a1401a000f4240ffffd8799fd8799fd8799f581cd2f63bb93a46252714598085271f2368e8669f8aaf8f5fab4e4fc2f5ffd8799fd8799fd8799f581cd6a3bf692f79c73710e92f431adf0bcb4aa17ecb0b3b126bb1defa5dffffffffa140d8799f00a1401a002dc6c0ffffffffff0581840001d87a80821a002f5dee1a34ddb004f5f6";

        let txn = Transaction::from_bytes(hex::decode(tx).unwrap()).unwrap();

        let tx_details: TxDetails = Into::<TxDetails>::into(txn);

        let hash = unsafe { tx_details.hash.unowned() }.unwrap();

        println!("Hash: {}", hex::encode(hash));

        assert_eq!(
            hash,
            hex::decode("a941d0f926a2da2848faa36864699dd019dfb149420b62413d9b71e21c5e4c46")
                .unwrap()
        );
        assert_eq!(tx_details.fee, 476765);
    }
}
