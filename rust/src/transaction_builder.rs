use crate::address::address::Address;
use crate::address::byron::ByronAddress;
use crate::address::pointer::SlotBigNum;
use crate::array::*;
use crate::certificate::Certificates;
use crate::data::CData;
use crate::error::CError;
use crate::linear_fee::{Coin, LinearFee};
use crate::option::COption;
use crate::panic::*;
use crate::ptr::*;
use crate::stake_credential::{Ed25519KeyHash, Ed25519KeyHashes, ScriptHash};
use crate::string::IntoCString;
use crate::transaction_body::{Mint, ScriptDataHash, TransactionBody};
use crate::transaction_input::TransactionInput;
use crate::transaction_metadata::{AuxiliaryData, NativeScripts};
use crate::transaction_output::{TransactionOutput, TransactionOutputs};
use crate::transaction_unspent_output::TransactionUnspentOutputs;
use crate::value::Value;
use crate::withdrawals::Withdrawals;
// use cardano_serialization_lib::Ed25519KeyHashes;
use cardano_serialization_lib::{
    address::{Address as RAddress, ByronAddress as RByronAddress},
    crypto::{Ed25519KeyHash as REd25519KeyHash, ScriptHash as RScriptHash},
    fees::LinearFee as RLinearFee,
    metadata::AuxiliaryData as RAuxiliaryData,
    tx_builder::tx_inputs_builder::TxInputsBuilder,
    tx_builder::{
        CoinSelectionStrategyCIP2 as RCoinSelectionStrategyCIP2,
        TransactionBuilder as RTransactionBuilder,
        TransactionBuilderConfig as RTransactionBuilderConfig,
    },
    utils::{
        from_bignum, to_bignum, BigNum as RBigNum, Coin as RCoin,
        TransactionUnspentOutputs as RTransactionUnspentOutputs, Value as RValue,
    },
    Certificates as RCertificates, Ed25519KeyHashes as REd25519KeyHashes, Mint as RMint,
    NativeScripts as RNativeScripts, TransactionInput as RTransactionInput,
    TransactionOutput as RTransactionOutput, TransactionOutputs as RTransactionOutputs,
    Withdrawals as RWithdrawals,
};

use std::collections::BTreeSet;
use std::convert::{TryFrom, TryInto};

pub type BigNum = u64;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct MockWitnessSet {
    vkeys: CArray<Ed25519KeyHash>,
    scripts: CArray<ScriptHash>,
    bootstraps: CArray<CData>,
}

impl Free for MockWitnessSet {
    unsafe fn free(&mut self) {
        self.vkeys.free();
        self.scripts.free();
        self.bootstraps.free();
    }
}

// for transmute
struct TMockWitnessSet {
    vkeys: BTreeSet<REd25519KeyHash>,
    scripts: BTreeSet<RScriptHash>,
    bootstraps: BTreeSet<Vec<u8>>,
}

impl TryFrom<MockWitnessSet> for TMockWitnessSet {
    type Error = CError;

    fn try_from(mock_witness_set: MockWitnessSet) -> Result<Self> {
        let vkeys = unsafe { mock_witness_set.vkeys.unowned()? };
        let scripts = unsafe { mock_witness_set.scripts.unowned()? };
        let bootstraps = unsafe { mock_witness_set.bootstraps.unowned()? };
        let vkeys = vkeys.to_vec().into_iter().map(|vkey| vkey.into()).collect();
        let scripts = scripts.to_vec().into_iter().map(|s| s.into()).collect();
        let bootstraps = bootstraps
            .to_vec()
            .into_iter()
            .map(|bootstrap| {
                let bootstrap = unsafe { bootstrap.unowned()? };
                Ok(bootstrap.to_vec())
            })
            .collect::<Result<BTreeSet<Vec<u8>>>>()?;
        Ok(Self {
            vkeys,
            scripts,
            bootstraps,
        })
    }
}

impl TryFrom<TMockWitnessSet> for MockWitnessSet {
    type Error = CError;

    fn try_from(mock_witness_set: TMockWitnessSet) -> Result<Self> {
        mock_witness_set
            .vkeys
            .into_iter()
            .map(|vkey| vkey.try_into())
            .collect::<Result<Vec<Ed25519KeyHash>>>()
            .zip(
                mock_witness_set
                    .scripts
                    .into_iter()
                    .map(|script| script.try_into())
                    .collect::<Result<Vec<ScriptHash>>>(),
            )
            .zip(Ok(mock_witness_set
                .bootstraps
                .into_iter()
                .map(|bootstrap| bootstrap.into())
                .collect::<Vec<CData>>()))
            .map(|((vkeys, scripts), bootstraps)| MockWitnessSet {
                vkeys: vkeys.into(),
                scripts: scripts.into(),
                bootstraps: bootstraps.into(),
            })
    }
}

#[no_mangle]
pub unsafe extern "C" fn cardano_mock_witness_set_clone(
    mock_witness_set: MockWitnessSet,
    result: &mut MockWitnessSet,
    error: &mut CError,
) -> bool {
    handle_exception(|| mock_witness_set.clone()).response(result, error)
}

#[no_mangle]
pub unsafe extern "C" fn cardano_mock_witness_set_free(mock_witness_set: &mut MockWitnessSet) {
    mock_witness_set.free()
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct SubCoin {
    numerator: BigNum,
    denominator: BigNum,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct ExUnitPrices {
    mem_price: SubCoin,
    step_price: SubCoin,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct TransactionBuilderConfig {
    fee_algo: LinearFee,
    pool_deposit: BigNum,
    key_deposit: BigNum,
    max_value_size: u32,
    max_tx_size: u32,
    coins_per_utxo_word: Coin,
    ex_unit_prices: COption<ExUnitPrices>, // protocol parameter
    prefer_pure_change: bool,
}

// for transmute
pub struct TTransactionBuilderConfig {
    fee_algo: RLinearFee,
    pool_deposit: RBigNum,
    key_deposit: RBigNum,
    max_value_size: u32,
    max_tx_size: u32,
    coins_per_utxo_word: RCoin,
    ex_unit_prices: COption<ExUnitPrices>, // protocol parameter
    prefer_pure_change: bool,
}

impl From<TransactionBuilderConfig> for TTransactionBuilderConfig {
    fn from(transaction_builder_config: TransactionBuilderConfig) -> Self {
        Self {
            fee_algo: transaction_builder_config.fee_algo.into(),
            pool_deposit: to_bignum(transaction_builder_config.pool_deposit),
            key_deposit: to_bignum(transaction_builder_config.key_deposit),
            max_value_size: transaction_builder_config.max_value_size,
            max_tx_size: transaction_builder_config.max_tx_size,
            coins_per_utxo_word: to_bignum(transaction_builder_config.coins_per_utxo_word),
            ex_unit_prices: transaction_builder_config.ex_unit_prices,
            prefer_pure_change: transaction_builder_config.prefer_pure_change,
        }
    }
}

impl From<TTransactionBuilderConfig> for TransactionBuilderConfig {
    fn from(transaction_builder_config: TTransactionBuilderConfig) -> Self {
        Self {
            fee_algo: transaction_builder_config.fee_algo.into(),
            pool_deposit: from_bignum(&transaction_builder_config.pool_deposit),
            key_deposit: from_bignum(&transaction_builder_config.key_deposit),
            max_value_size: transaction_builder_config.max_value_size,
            max_tx_size: transaction_builder_config.max_tx_size,
            coins_per_utxo_word: from_bignum(&transaction_builder_config.coins_per_utxo_word),
            ex_unit_prices: transaction_builder_config.ex_unit_prices.into(),
            prefer_pure_change: transaction_builder_config.prefer_pure_change,
        }
    }
}

impl From<TransactionBuilderConfig> for RTransactionBuilderConfig {
    fn from(transaction_builder_config: TransactionBuilderConfig) -> Self {
        let tbc: TTransactionBuilderConfig = transaction_builder_config.into();
        unsafe { std::mem::transmute(tbc) }
    }
}

impl From<RTransactionBuilderConfig> for TransactionBuilderConfig {
    fn from(transaction_builder_config: RTransactionBuilderConfig) -> Self {
        let tbc: TTransactionBuilderConfig =
            unsafe { std::mem::transmute(transaction_builder_config) };
        tbc.into()
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct TxBuilderInput {
    input: TransactionInput,
    amount: Value,
}

impl Free for TxBuilderInput {
    unsafe fn free(&mut self) {
        self.amount.free();
    }
}

// for transmute
struct TTxBuilderInput {
    input: RTransactionInput,
    amount: RValue,
}

impl Clone for TTxBuilderInput {
    fn clone(&self) -> Self {
        Self {
            input: self.input.clone(),
            amount: self.amount.clone(),
        }
    }
}

impl TryFrom<TxBuilderInput> for TTxBuilderInput {
    type Error = CError;

    fn try_from(tx_builder_input: TxBuilderInput) -> Result<Self> {
        tx_builder_input.amount.try_into().map(|amount| Self {
            input: tx_builder_input.input.into(),
            amount,
        })
    }
}

impl TryFrom<TTxBuilderInput> for TxBuilderInput {
    type Error = CError;

    fn try_from(tx_builder_input: TTxBuilderInput) -> Result<Self> {
        tx_builder_input
            .input
            .try_into()
            .zip(tx_builder_input.amount.try_into())
            .map(|(input, amount)| Self { input, amount })
    }
}

#[no_mangle]
pub unsafe extern "C" fn cardano_tx_builder_input_clone(
    tx_builder_input: TxBuilderInput,
    result: &mut TxBuilderInput,
    error: &mut CError,
) -> bool {
    handle_exception(|| tx_builder_input.clone()).response(result, error)
}

#[no_mangle]
pub unsafe extern "C" fn cardano_tx_builder_input_free(tx_builder_input: &mut TxBuilderInput) {
    tx_builder_input.free()
}

#[repr(C)]
#[derive(Copy, Clone)]
pub enum CoinSelectionStrategyCIP2 {
    LargestFirst,
    RandomImprove,
    LargestFirstMultiAsset,
    RandomImproveMultiAsset,
}

impl From<CoinSelectionStrategyCIP2> for RCoinSelectionStrategyCIP2 {
    fn from(coin_selection_strategy_cip2: CoinSelectionStrategyCIP2) -> Self {
        match coin_selection_strategy_cip2 {
            CoinSelectionStrategyCIP2::LargestFirst => Self::LargestFirst,
            CoinSelectionStrategyCIP2::RandomImprove => Self::RandomImprove,
            CoinSelectionStrategyCIP2::LargestFirstMultiAsset => Self::LargestFirstMultiAsset,
            CoinSelectionStrategyCIP2::RandomImproveMultiAsset => Self::RandomImproveMultiAsset,
        }
    }
}

impl From<RCoinSelectionStrategyCIP2> for CoinSelectionStrategyCIP2 {
    fn from(coin_selection_strategy_cip2: RCoinSelectionStrategyCIP2) -> Self {
        match coin_selection_strategy_cip2 {
            RCoinSelectionStrategyCIP2::LargestFirst => Self::LargestFirst,
            RCoinSelectionStrategyCIP2::RandomImprove => Self::RandomImprove,
            RCoinSelectionStrategyCIP2::LargestFirstMultiAsset => Self::LargestFirstMultiAsset,
            RCoinSelectionStrategyCIP2::RandomImproveMultiAsset => Self::RandomImproveMultiAsset,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct TransactionBuilder {
    config: TransactionBuilderConfig,
    inputs: CArray<TxBuilderInput>,
    collateral: CArray<TxBuilderInput>,
    outputs: TransactionOutputs,
    fee: COption<Coin>,
    ttl: COption<SlotBigNum>,
    certs: COption<Certificates>,
    withdrawals: COption<Withdrawals>,
    auxiliary_data: COption<AuxiliaryData>,
    validity_start_interval: COption<SlotBigNum>,
    mint: COption<Mint>,
    mint_scripts: COption<NativeScripts>,
    script_data_hash: COption<ScriptDataHash>,
    required_signers: Ed25519KeyHashes,
}

impl Free for TransactionBuilder {
    unsafe fn free(&mut self) {
        self.inputs.free();
        self.outputs.free();
        self.certs.free();
        self.withdrawals.free();
        self.auxiliary_data.free();
        self.mint_scripts.free();
    }
}

// for transmute
pub struct TTransactionBuilder {
    config: RTransactionBuilderConfig,
    inputs: Vec<TTxBuilderInput>,
    #[allow(dead_code)]
    collateral: Vec<TTxBuilderInput>,
    outputs: RTransactionOutputs,
    fee: Option<RCoin>,
    ttl: Option<SlotBigNum>,
    certs: Option<RCertificates>,
    withdrawals: Option<RWithdrawals>,
    auxiliary_data: Option<RAuxiliaryData>,
    validity_start_interval: Option<SlotBigNum>,
    // input_types: TMockWitnessSet,
    mint: Option<RMint>,
    mint_scripts: Option<RNativeScripts>,
    #[allow(dead_code)]
    script_data_hash: COption<ScriptDataHash>,
    #[allow(dead_code)]
    required_signers: Ed25519KeyHashes,
}

impl TryFrom<TransactionBuilder> for TTransactionBuilder {
    type Error = CError;

    fn try_from(tb: TransactionBuilder) -> Result<Self> {
        let inputs = unsafe { tb.inputs.unowned()? };
        inputs
            .to_vec()
            .into_iter()
            .map(|input| input.try_into())
            .collect::<Result<Vec<TTxBuilderInput>>>()
            .zip(tb.outputs.try_into())
            .zip({
                let certs: Option<Certificates> = tb.certs.into();
                certs.map(|certs| certs.try_into()).transpose()
            })
            .zip({
                let withdrawals: Option<Withdrawals> = tb.withdrawals.into();
                withdrawals.map(|wls| wls.try_into()).transpose()
            })
            .zip({
                let auxiliary_data: Option<AuxiliaryData> = tb.auxiliary_data.into();
                auxiliary_data
                    .map(|auxiliary_data| auxiliary_data.try_into())
                    .transpose()
            })
            .zip({
                let mint: Option<Mint> = tb.mint.into();
                mint.map(|mint| mint.try_into()).transpose()
            })
            .zip({
                let mint_scripts: Option<NativeScripts> = tb.mint_scripts.into();
                mint_scripts
                    .map(|mint_scripts| mint_scripts.try_into())
                    .transpose()
            })
            .zip(TryInto::<Ed25519KeyHashes>::try_into(
                REd25519KeyHashes::new(),
            ))
            .map(
                |(
                    (
                        (((((inputs, outputs), certs), withdrawals), auxiliary_data), mint),
                        mint_scripts,
                    ),
                    key_hashes,
                )| {
                    let fee: Option<Coin> = tb.fee.into();
                    Self {
                        config: tb.config.into(),
                        inputs: inputs.clone(),
                        outputs,
                        collateral: inputs, //FIXME: This is wrong!
                        fee: fee.map(|fee| to_bignum(fee)),
                        ttl: tb.ttl.into(),
                        certs: certs.into(),
                        withdrawals: withdrawals.into(),
                        auxiliary_data: auxiliary_data.into(),
                        validity_start_interval: tb.validity_start_interval.into(),
                        // input_types,
                        mint: mint.into(),
                        mint_scripts: mint_scripts.into(),
                        script_data_hash: COption::None,
                        required_signers: key_hashes,
                    }
                },
            )
    }
}

impl TryFrom<TTransactionBuilder> for TransactionBuilder {
    type Error = CError;

    fn try_from(tb: TTransactionBuilder) -> Result<Self> {
        let config = tb.config.into();
        let fee = tb.fee.map(|fee| from_bignum(&fee)).into();
        let ttl = tb.ttl.into();
        let validity_start_interval = tb.validity_start_interval.into();
        tb.inputs
            .into_iter()
            .map(|input| input.try_into())
            .collect::<Result<Vec<TxBuilderInput>>>()
            .zip(tb.outputs.try_into())
            .zip(tb.certs.map(|certs| certs.try_into()).transpose())
            .zip(tb.withdrawals.map(|wls| wls.try_into()).transpose())
            .zip(
                tb.auxiliary_data
                    .map(|auxiliary_data| auxiliary_data.try_into())
                    .transpose(),
            )
            .zip(tb.mint.map(|mint| mint.try_into()).transpose())
            .zip(
                tb.mint_scripts
                    .map(|mint_scripts| mint_scripts.try_into())
                    .transpose(),
            )
            .zip(TryInto::<Ed25519KeyHashes>::try_into(
                REd25519KeyHashes::new(),
            ))
            .map(
                |(
                    (
                        (((((inputs, outputs), certs), withdrawals), auxiliary_data), mint),
                        mint_scripts,
                    ),
                    key_hashes,
                )| Self {
                    config,
                    inputs: inputs.clone().into(),
                    collateral: inputs.into(), //FIXME: This is wrong!
                    outputs,
                    fee,
                    ttl,
                    certs: certs.into(),
                    withdrawals: withdrawals.into(),
                    auxiliary_data: auxiliary_data.into(),
                    validity_start_interval,
                    mint: mint.into(),
                    mint_scripts: mint_scripts.into(),
                    required_signers: key_hashes,
                    script_data_hash: COption::None,
                },
            )
    }
}

impl TryFrom<TransactionBuilder> for RTransactionBuilder {
    type Error = CError;

    fn try_from(transaction_builder: TransactionBuilder) -> Result<Self> {
        // transaction_builder
        //   .try_into()
        //   .map(|transaction_builder: TTransactionBuilder| unsafe {
        //     std::mem::transmute(transaction_builder)
        //   });

        let r = TryInto::<RTransactionBuilderConfig>::try_into(transaction_builder.config)
            .and_then(|cfg| {
                let tx_builder = Self::new(&cfg);

                //FIXME: Populate!!!

                Ok(tx_builder)
            });

        r.map_err(|_e: std::convert::Infallible| CError::Error(_e.to_string().into_cstr()))
    }
}

impl TryFrom<RTransactionBuilder> for TransactionBuilder {
    type Error = CError;

    fn try_from(_transaction_builder: RTransactionBuilder) -> Result<Self> {
        // let transaction_builder: TTransactionBuilder =
        //   unsafe { std::mem::transmute(transaction_builder) };
        // transaction_builder.try_into()
        todo!()
        //FIXME: Populate!!!
        // Ok(Self {
        //   auxiliary_data: todo!(),
        //   config: todo!(),
        //   inputs: todo!(),
        //   collateral: todo!(),
        //   outputs: todo!(),
        //   fee: todo!(),
        //   ttl: todo!(),
        //   certs: todo!(),
        //   withdrawals: todo!(),
        //   validity_start_interval: todo!(),
        //   mint: todo!(),
        //   mint_scripts: todo!(),
        //   script_data_hash: todo!(),
        //   required_signers: todo!(),
        // })
    }
}

#[no_mangle]
pub unsafe extern "C" fn cardano_transaction_builder_add_inputs_from(
    tb: TransactionBuilder,
    inputs: TransactionUnspentOutputs,
    strategy: CoinSelectionStrategyCIP2,
    result: &mut TransactionBuilder,
    error: &mut CError,
) -> bool {
    handle_exception_result(|| {
        tb.try_into()
            .zip(inputs.try_into())
            .and_then(
                |(mut tb, inputs): (RTransactionBuilder, RTransactionUnspentOutputs)| {
                    tb.add_inputs_from(&inputs, strategy.into())
                        .into_result()
                        .zip(tb.try_into())
                },
            )
            .map(|(_, tb)| tb)
    })
    .response(result, error)
}

#[no_mangle]
pub unsafe extern "C" fn cardano_transaction_builder_add_key_input(
    tb: TransactionBuilder,
    hash: Ed25519KeyHash,
    input: TransactionInput,
    amount: Value,
    result: &mut TransactionBuilder,
    error: &mut CError,
) -> bool {
    handle_exception_result(|| {
        tb.try_into()
            .zip(amount.try_into())
            .map(|(mut tb, amount): (RTransactionBuilder, RValue)| {
                let mut tx_builder = TxInputsBuilder::new();
                tx_builder.add_key_input(&hash.into(), &input.into(), &amount);

                tb.set_inputs(&tx_builder);

                // tb.add_key_input(&hash.into(), &input.into(), &amount);
                tb
            })
            .and_then(|tb| tb.try_into())
    })
    .response(result, error)
}

#[no_mangle]
pub unsafe extern "C" fn cardano_transaction_builder_add_script_input(
    tb: TransactionBuilder,
    hash: ScriptHash,
    input: TransactionInput,
    amount: Value,
    result: &mut TransactionBuilder,
    error: &mut CError,
) -> bool {
    handle_exception_result(|| {
        tb.try_into()
            .zip(amount.try_into())
            .map(|(mut tb, amount): (RTransactionBuilder, RValue)| {
                let mut tx_builder = TxInputsBuilder::new();
                tx_builder.add_script_input(&hash.into(), &input.into(), &amount);

                tb.set_inputs(&tx_builder);
                tb
            })
            .and_then(|tb| tb.try_into())
    })
    .response(result, error)
}

#[no_mangle]
pub unsafe extern "C" fn cardano_transaction_builder_add_bootstrap_input(
    tb: TransactionBuilder,
    hash: ByronAddress,
    input: TransactionInput,
    amount: Value,
    result: &mut TransactionBuilder,
    error: &mut CError,
) -> bool {
    handle_exception_result(|| {
        tb.try_into()
            .zip(hash.try_into())
            .zip(amount.try_into())
            .map(
                |((mut tb, hash), amount): ((RTransactionBuilder, RByronAddress), RValue)| {
                    let mut tx_builder = TxInputsBuilder::new();
                    tx_builder.add_bootstrap_input(&hash.into(), &input.into(), &amount);

                    tb.set_inputs(&tx_builder);
                    tb
                },
            )
            .and_then(|tb| tb.try_into())
    })
    .response(result, error)
}

#[no_mangle]
pub unsafe extern "C" fn cardano_transaction_builder_add_input(
    tb: TransactionBuilder,
    address: Address,
    input: TransactionInput,
    amount: Value,
    result: &mut TransactionBuilder,
    error: &mut CError,
) -> bool {
    handle_exception_result(|| {
        tb.try_into()
            .zip(address.try_into())
            .zip(amount.try_into())
            .map(
                |((mut tb, address), amount): ((RTransactionBuilder, RAddress), RValue)| {
                    let mut tx_builder = TxInputsBuilder::new();
                    tx_builder.add_input(&address, &input.into(), &amount);

                    tb.set_inputs(&tx_builder);
                    tb
                },
            )
            .and_then(|tb| tb.try_into())
    })
    .response(result, error)
}

#[no_mangle]
pub unsafe extern "C" fn cardano_transaction_builder_fee_for_input(
    tb: TransactionBuilder,
    address: Address,
    input: TransactionInput,
    amount: Value,
    result: &mut Coin,
    error: &mut CError,
) -> bool {
    handle_exception_result(|| {
        tb.try_into()
            .zip(address.try_into())
            .zip(amount.try_into())
            .and_then(
                |((tb, address), amount): ((RTransactionBuilder, RAddress), RValue)| {
                    tb.fee_for_input(&address, &input.into(), &amount)
                        .into_result()
                },
            )
            .map(|fee| from_bignum(&fee))
    })
    .response(result, error)
}

#[no_mangle]
pub unsafe extern "C" fn cardano_transaction_builder_add_output(
    tb: TransactionBuilder,
    output: TransactionOutput,
    result: &mut TransactionBuilder,
    error: &mut CError,
) -> bool {
    handle_exception_result(|| {
        tb.try_into()
            .zip(output.try_into())
            .and_then(
                |(mut tb, output): (RTransactionBuilder, RTransactionOutput)| {
                    tb.add_output(&output).into_result().map(|_| tb)
                },
            )
            .and_then(|tb| tb.try_into())
    })
    .response(result, error)
}

#[no_mangle]
pub unsafe extern "C" fn cardano_transaction_builder_fee_for_output(
    tb: TransactionBuilder,
    output: TransactionOutput,
    result: &mut Coin,
    error: &mut CError,
) -> bool {
    handle_exception_result(|| {
        tb.try_into()
            .zip(output.try_into())
            .and_then(|(tb, output): (RTransactionBuilder, RTransactionOutput)| {
                tb.fee_for_output(&output).into_result()
            })
            .map(|fee| from_bignum(&fee))
    })
    .response(result, error)
}

#[no_mangle]
pub unsafe extern "C" fn cardano_transaction_builder_set_certs(
    tb: TransactionBuilder,
    certs: Certificates,
    result: &mut TransactionBuilder,
    error: &mut CError,
) -> bool {
    handle_exception_result(|| {
        tb.try_into()
            .zip(certs.try_into())
            .map(|(mut tb, certs): (RTransactionBuilder, RCertificates)| {
                tb.set_certs(&certs);
                tb
            })
            .and_then(|tb| tb.try_into())
    })
    .response(result, error)
}

#[no_mangle]
pub unsafe extern "C" fn cardano_transaction_builder_set_withdrawals(
    tb: TransactionBuilder,
    withdrawals: Withdrawals,
    result: &mut TransactionBuilder,
    error: &mut CError,
) -> bool {
    handle_exception_result(|| {
        tb.try_into()
            .zip(withdrawals.try_into())
            .map(
                |(mut tb, withdrawals): (RTransactionBuilder, RWithdrawals)| {
                    tb.set_withdrawals(&withdrawals);
                    tb
                },
            )
            .and_then(|tb| tb.try_into())
    })
    .response(result, error)
}

#[no_mangle]
pub unsafe extern "C" fn cardano_transaction_builder_new(
    config: TransactionBuilderConfig,
    result: &mut TransactionBuilder,
    error: &mut CError,
) -> bool {
    handle_exception_result(|| RTransactionBuilder::new(&config.into()).try_into())
        .response(result, error)
}

#[no_mangle]
pub unsafe extern "C" fn cardano_transaction_builder_get_explicit_input(
    tb: TransactionBuilder,
    result: &mut Value,
    error: &mut CError,
) -> bool {
    handle_exception_result(|| {
        tb.try_into()
            .and_then(|tb: RTransactionBuilder| tb.get_explicit_input().into_result())
            .and_then(|input| input.try_into())
    })
    .response(result, error)
}

#[no_mangle]
pub unsafe extern "C" fn cardano_transaction_builder_get_implicit_input(
    tb: TransactionBuilder,
    result: &mut Value,
    error: &mut CError,
) -> bool {
    handle_exception_result(|| {
        tb.try_into()
            .and_then(|tb: RTransactionBuilder| tb.get_implicit_input().into_result())
            .and_then(|input| input.try_into())
    })
    .response(result, error)
}

#[no_mangle]
pub unsafe extern "C" fn cardano_transaction_builder_get_explicit_output(
    tb: TransactionBuilder,
    result: &mut Value,
    error: &mut CError,
) -> bool {
    handle_exception_result(|| {
        tb.try_into()
            .and_then(|tb: RTransactionBuilder| tb.get_explicit_output().into_result())
            .and_then(|output| output.try_into())
    })
    .response(result, error)
}

#[no_mangle]
pub unsafe extern "C" fn cardano_transaction_builder_get_deposit(
    tb: TransactionBuilder,
    result: &mut Coin,
    error: &mut CError,
) -> bool {
    handle_exception_result(|| {
        tb.try_into()
            .and_then(|tb: RTransactionBuilder| tb.get_deposit().into_result())
            .map(|deposit| from_bignum(&deposit))
    })
    .response(result, error)
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct TransactionBuilderBool(TransactionBuilder, bool);

#[no_mangle]
pub unsafe extern "C" fn cardano_transaction_builder_add_change_if_needed(
    tb: TransactionBuilder,
    address: Address,
    result: &mut TransactionBuilderBool,
    error: &mut CError,
) -> bool {
    handle_exception_result(|| {
        tb.try_into().zip(address.try_into()).and_then(
            |(mut tb, address): (RTransactionBuilder, RAddress)| {
                tb.add_change_if_needed(&address)
                    .into_result()
                    .and_then(|result| tb.try_into().map(|tb| TransactionBuilderBool(tb, result)))
            },
        )
    })
    .response(result, error)
}

#[no_mangle]
pub unsafe extern "C" fn cardano_transaction_builder_build(
    tb: TransactionBuilder,
    result: &mut TransactionBody,
    error: &mut CError,
) -> bool {
    handle_exception_result(|| {
        tb.try_into()
            .and_then(|tb: RTransactionBuilder| tb.build().into_result())
            .and_then(|transaction_body| transaction_body.try_into())
    })
    .response(result, error)
}

#[no_mangle]
pub unsafe extern "C" fn cardano_transaction_builder_min_fee(
    tb: TransactionBuilder,
    result: &mut Coin,
    error: &mut CError,
) -> bool {
    handle_exception_result(|| {
        tb.try_into()
            .and_then(|tb: RTransactionBuilder| tb.min_fee().into_result())
            .map(|fee| from_bignum(&fee))
    })
    .response(result, error)
}

#[no_mangle]
pub unsafe extern "C" fn cardano_transaction_builder_clone(
    transaction_builder: TransactionBuilder,
    result: &mut TransactionBuilder,
    error: &mut CError,
) -> bool {
    handle_exception(|| transaction_builder.clone()).response(result, error)
}

#[no_mangle]
pub unsafe extern "C" fn cardano_transaction_builder_free(
    transaction_builder: &mut TransactionBuilder,
) {
    transaction_builder.free()
}
