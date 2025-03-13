use crate::address::byron::ByronAddress;
use crate::array::CArray;
use crate::bip32_private_key::Bip32PrivateKey;
use crate::data::CData;
use crate::ed25519_signature::Ed25519Signature;
use crate::error::CError;
use crate::panic::*;
use crate::ptr::*;
use crate::transaction_hash::TransactionHash;
use crate::vkey::Vkey;
use cardano_serialization_lib::{
    crypto::{BootstrapWitness as RBootstrapWitness, BootstrapWitnesses as RBootstrapWitnesses},
    utils::make_icarus_bootstrap_witness,
};
use std::convert::{TryFrom, TryInto};

#[repr(C)]
#[derive(Copy)]
pub struct BootstrapWitness {
    vkey: Vkey,
    signature: Ed25519Signature,
    chain_code: CData,
    attributes: CData,
}

impl Clone for BootstrapWitness {
    fn clone(&self) -> Self {
        let chain_code = unsafe { self.chain_code.unowned().expect("Bad bytes pointer").into() };
        let attributes = unsafe { self.attributes.unowned().expect("Bad bytes pointer").into() };
        BootstrapWitness {
            vkey: self.vkey.clone(),
            signature: self.signature.clone(),
            chain_code,
            attributes,
        }
    }
}

impl Free for BootstrapWitness {
    unsafe fn free(&mut self) {
        self.chain_code.free();
        self.attributes.free();
    }
}

impl TryFrom<BootstrapWitness> for RBootstrapWitness {
    type Error = CError;

    fn try_from(bootstrap_witness: BootstrapWitness) -> Result<Self> {
        let chain_code = unsafe { bootstrap_witness.chain_code.unowned()? };
        let attributes = unsafe { bootstrap_witness.attributes.unowned()? };
        bootstrap_witness
            .vkey
            .try_into()
            .zip(bootstrap_witness.signature.try_into())
            .map(|(vkey, signature)| {
                RBootstrapWitness::new(&vkey, &signature, chain_code.to_vec(), attributes.to_vec())
            })
    }
}

impl From<RBootstrapWitness> for BootstrapWitness {
    fn from(bootstrap_witness: RBootstrapWitness) -> Self {
        Self {
            vkey: bootstrap_witness.vkey().into(),
            signature: bootstrap_witness.signature().into(),
            chain_code: bootstrap_witness.chain_code().into(),
            attributes: bootstrap_witness.attributes().into(),
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn cardano_bootstrap_witness_make_icarus_bootstrap_witness(
    tx_body_hash: TransactionHash,
    addr: ByronAddress,
    key: Bip32PrivateKey,
    result: &mut BootstrapWitness,
    error: &mut CError,
) -> bool {
    handle_exception_result(|| {
        addr.try_into()
            .zip(key.try_into())
            .map(|(addr, key)| make_icarus_bootstrap_witness(&tx_body_hash.into(), &addr, &key))
            .map(|bootstrap_witness| bootstrap_witness.into())
    })
    .response(result, error)
}

#[no_mangle]
pub unsafe extern "C" fn cardano_bootstrap_witness_clone(
    bootstrap_witness: BootstrapWitness,
    result: &mut BootstrapWitness,
    error: &mut CError,
) -> bool {
    handle_exception(|| bootstrap_witness.clone()).response(result, error)
}

#[no_mangle]
pub unsafe extern "C" fn cardano_bootstrap_witness_free(bootstrap_witness: &mut BootstrapWitness) {
    bootstrap_witness.free()
}

pub type BootstrapWitnesses = CArray<BootstrapWitness>;

impl From<RBootstrapWitnesses> for BootstrapWitnesses {
    fn from(bootstrap_witnesses: RBootstrapWitnesses) -> Self {
        (0..bootstrap_witnesses.len())
            .map(|index| bootstrap_witnesses.get(index))
            .map(|bootstrap_witness| bootstrap_witness.into())
            .collect::<Vec<BootstrapWitness>>()
            .into()
    }
}

impl TryFrom<BootstrapWitnesses> for RBootstrapWitnesses {
    type Error = CError;

    fn try_from(bootstrap_witnesses: BootstrapWitnesses) -> Result<Self> {
        let vec = unsafe { bootstrap_witnesses.unowned()? };
        let mut bootstrap_witnesses = RBootstrapWitnesses::new();
        for bootstrap_witness in vec.to_vec() {
            let bootstrap_witness = bootstrap_witness.try_into()?;
            bootstrap_witnesses.add(&bootstrap_witness);
        }
        Ok(bootstrap_witnesses)
    }
}

#[no_mangle]
pub unsafe extern "C" fn cardano_bootstrap_witnesses_free(
    bootstrap_witnesses: &mut BootstrapWitnesses,
) {
    bootstrap_witnesses.free();
}
