use super::super::array::CArray;
use super::super::stake_credential::{Ed25519KeyHash, StakeCredential};
use super::addr_type::AddrType;
use super::base::BaseAddress;
use super::byron::ByronAddress;
use super::enterprise::EnterpriseAddress;
use super::pointer::PointerAddress;
use super::reward::RewardAddress;
use crate::data::CData;
use crate::error::CError;
use crate::panic::*;
use crate::ptr::*;
use crate::string::*;
use cardano_serialization_lib::address::Address as RAddress;
use std::convert::{TryFrom, TryInto};

#[repr(C)]
#[derive(Copy, Clone)]
pub enum Address {
    Base(BaseAddress),
    Ptr(PointerAddress),
    Enterprise(EnterpriseAddress),
    Reward(RewardAddress),
    Byron(ByronAddress),
}

impl Free for Address {
    unsafe fn free(&mut self) {
        match self {
            &mut Address::Byron(mut byron) => byron.free(),
            _ => return,
        }
    }
}

impl TryFrom<RAddress> for Address {
    type Error = CError;

    fn try_from(address: RAddress) -> Result<Self> {
        let t: AddrType = address.into();
        match t {
            AddrType::Base(base) => base.try_into().map(Address::Base),
            AddrType::Byron(byron) => Ok(Address::Byron(byron.into())),
            AddrType::Ptr(ptr) => ptr.try_into().map(Address::Ptr),
            AddrType::Enterprise(ent) => ent.try_into().map(Address::Enterprise),
            AddrType::Reward(rew) => rew.try_into().map(Address::Reward),
        }
    }
}

impl TryFrom<Address> for RAddress {
    type Error = CError;

    fn try_from(address: Address) -> Result<Self> {
        match address {
            Address::Base(base) => Ok(AddrType::Base(base.into()).into()),
            Address::Byron(byron) => byron.try_into().map(AddrType::Byron).map(|t| t.into()),
            Address::Enterprise(ent) => Ok(AddrType::Enterprise(ent.into()).into()),
            Address::Ptr(ptr) => Ok(AddrType::Ptr(ptr.into()).into()),
            Address::Reward(rew) => Ok(AddrType::Reward(rew.into()).into()),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub enum AddressMatch {
    PaymentKeyMatch(),
    StakeKeyMatch(),
    NoMatch(),
}

impl Free for AddressMatch {
  unsafe fn free(&mut self) {
  }
}

#[no_mangle]
pub unsafe extern "C" fn cardano_address_match_pkh(
    address: Address,
    pkhs: CArray<Ed25519KeyHash>,
    is_match: &mut AddressMatch,
    error: &mut CError,
) -> bool {
    handle_exception_result(|| {
        let pkh_arr = unsafe { pkhs.unowned()? };

        let res = match address {
            Address::Base(a) => Ok((a.payment_cred(), "PAYMENT")),
            Address::Ptr(_) => Err(CError::Error("Bad address type".into_cstr())),
            Address::Enterprise(a) => Ok((a.payment_cred(), "PAYMENT")),
            Address::Reward(a) => Ok((a.payment_cred(), "REWARD")),
            Address::Byron(_) => Err(CError::Error("Bad address type - Byron".into_cstr())),
        }
        .map(|(cred, key_type)| {
            if pkh_arr
                .into_iter()
                .any(|pkh| StakeCredential::Key(pkh.clone()).eq(&cred))
            {
              return match key_type {
                    "REWARD" => Ok(AddressMatch::StakeKeyMatch()),
                    _ => Ok(AddressMatch::PaymentKeyMatch()),
                }
            }

            return Ok(AddressMatch::NoMatch());
        });

        match res {
            Ok(r) => r,
            Err(e) => Err(e),
        }
    })
    .response(is_match, error)

}

#[no_mangle]
pub unsafe extern "C" fn cardano_address_to_bytes(
    address: Address,
    bytes: &mut CData,
    error: &mut CError,
) -> bool {
    handle_exception_result(|| {
        address
            .try_into()
            .map(|addr: RAddress| addr.to_bytes().into())
    })
    .response(bytes, error)
}

#[no_mangle]
pub unsafe extern "C" fn cardano_address_from_bytes(
    bytes: CData,
    address: &mut Address,
    error: &mut CError,
) -> bool {
    handle_exception_result(|| {
        bytes
            .unowned()
            .and_then(|bytes| RAddress::from_bytes(bytes.into()).into_result())
            .and_then(|addr| addr.try_into())
    })
    .response(address, error)
}

#[no_mangle]
pub unsafe extern "C" fn cardano_address_to_bech32(
    address: Address,
    prefix: CharPtr,
    bech32: &mut CharPtr,
    error: &mut CError,
) -> bool {
    handle_exception_result(|| {
        let opt_prefix = if prefix == std::ptr::null() {
            None
        } else {
            Some(prefix)
        };
        address
            .try_into()
            .zip(opt_prefix.map_or(Ok(None), |p| p.unowned().map(|s| s.to_string().into())))
            .and_then(|(addr, prefix): (RAddress, Option<String>)| {
                addr.to_bech32(prefix).into_result()
            })
            .map(|addr_str| addr_str.into_cstr())
    })
    .response(bech32, error)
}

#[no_mangle]
pub unsafe extern "C" fn cardano_address_from_bech32(
    bech32: CharPtr,
    result: &mut Address,
    error: &mut CError,
) -> bool {
    handle_exception_result(|| {
        bech32
            .unowned()
            .and_then(|b32| RAddress::from_bech32(b32).into_result())
            .and_then(|a| a.try_into())
    })
    .response(result, error)
}

#[no_mangle]
pub unsafe extern "C" fn cardano_address_network_id(
    address: Address,
    result: &mut u8,
    error: &mut CError,
) -> bool {
    handle_exception_result(|| {
        address
            .try_into()
            .and_then(|addr: RAddress| addr.network_id().into_result())
    })
    .response(result, error)
}

#[no_mangle]
pub unsafe extern "C" fn cardano_address_clone(
    address: Address,
    result: &mut Address,
    error: &mut CError,
) -> bool {
    handle_exception(|| address.clone()).response(result, error)
}

#[no_mangle]
pub unsafe extern "C" fn cardano_address_free(address: &mut Address) {
    address.free();
}
