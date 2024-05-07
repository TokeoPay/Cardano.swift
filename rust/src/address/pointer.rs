use crate::error::CError;
use crate::panic::Result;
use crate::transaction_input::TransactionIndex;
use std::convert::{TryInto, TryFrom};
use crate::stake_credential::StakeCredential;

use cardano_serialization_lib::address::{
  Pointer as RPointer,
  StakeCredential as RStakeCredential,
  PointerAddress as RPointerAddress
};

pub type Slot = u32;
pub type SlotBigNum = u64;
pub type CertificateIndex = u32;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Pointer {
  slot: Slot,
  tx_index: TransactionIndex,
  cert_index: CertificateIndex,
}

impl From<RPointer> for Pointer {
  fn from(ptr: RPointer) -> Self {
    Self {
      slot: ptr.slot().unwrap(), tx_index: ptr.tx_index().unwrap(),
      cert_index: ptr.cert_index().unwrap()
    }
  }
}

impl From<Pointer> for RPointer {
  fn from(ptr: Pointer) -> Self {
    Self::new(ptr.slot, ptr.tx_index, ptr.cert_index)
  }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PointerAddress {
  network: u8,
  payment: StakeCredential,
  stake: Pointer,
}

struct MPAddress {
  network: u8,
  payment: RStakeCredential,
  stake: RPointer,
}

impl TryFrom<RPointerAddress> for PointerAddress {
  type Error = CError;

  fn try_from(address: RPointerAddress) -> Result<Self> {
    let maddress: MPAddress = unsafe { std::mem::transmute(address) };
    let payment = maddress.payment.try_into()?;
    Ok(Self { network: maddress.network, payment: payment, stake: maddress.stake.into() })
  }
}

impl From<PointerAddress> for RPointerAddress {
  fn from(address: PointerAddress) -> Self {
    Self::new(
      address.network, &address.payment.into(), &address.stake.into()
    )
  }
}