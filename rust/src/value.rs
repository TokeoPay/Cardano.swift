use crate::data::CData;
use crate::error::CError;
use crate::linear_fee::Coin;
use crate::multi_asset::MultiAsset;
use crate::option::COption;
use crate::panic::*;
use crate::ptr::*;
use crate::string::IntoCString;
use cardano_serialization_lib::utils::min_ada_required;
use cardano_serialization_lib::utils::{from_bignum, to_bignum, Value as RValue};

use cml_chain::assets::AssetBundle;

use std::cmp::Ordering;
use std::convert::{TryFrom, TryInto};

use cml_chain::assets::Value as CML_Value;

#[repr(C)]
#[derive(Copy, Clone)]
pub enum OrderingKind {
    Less,
    Equal,
    Greater,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct COrdering(OrderingKind);

impl From<COrdering> for Ordering {
    fn from(ordering: COrdering) -> Self {
        match ordering.0 {
            OrderingKind::Less => Self::Less,
            OrderingKind::Equal => Self::Equal,
            OrderingKind::Greater => Self::Greater,
        }
    }
}

impl From<Ordering> for COrdering {
    fn from(ordering: Ordering) -> Self {
        match ordering {
            Ordering::Less => Self(OrderingKind::Less),
            Ordering::Equal => Self(OrderingKind::Equal),
            Ordering::Greater => Self(OrderingKind::Greater),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Value {
    coin: Coin,
    multiasset: COption<MultiAsset>,
}

impl TryFrom<Value> for CML_Value {
    type Error = CError;

    fn try_from(value: Value) -> Result<Self> {

      let r = match value.multiasset {
        COption::Some(ma) => Some(
          TryInto::<AssetBundle<u64>>::try_into(ma)
            .unwrap_or(AssetBundle::default())
          ),
        COption::None => None,
      }.unwrap_or(AssetBundle::default());

      Ok(Self::new(value.coin, r))
    }
}

impl Free for Value {
    unsafe fn free(&mut self) {
        self.multiasset.free()
    }
}

impl TryFrom<Value> for RValue {
    type Error = CError;

    fn try_from(value: Value) -> Result<Self> {
        let multiasset: Option<MultiAsset> = value.multiasset.into();
        let mut value = Self::new(&to_bignum(value.coin));
        if let Some(multiasset) = multiasset {
            let multiasset = multiasset.try_into()?;
            value.set_multiasset(&multiasset);
        }
        Ok(value)
    }
}

impl TryFrom<RValue> for Value {
    type Error = CError;

    fn try_from(value: RValue) -> Result<Self> {
        value
            .multiasset()
            .map(|multiasset| multiasset.try_into())
            .transpose()
            .map(|option| Self {
                coin: from_bignum(&value.coin()),
                multiasset: option.into(),
            })
    }
}

#[no_mangle]
pub unsafe extern "C" fn cardano_value_from_bytes(
    bytes: CData,
    result: &mut Value,
    error: &mut CError) -> bool {

        handle_exception_result(|| {
            let bytes = unsafe { bytes.unowned() }?;
            let core_value = RValue::from_bytes(bytes.into()).map_err(|e| { CError::DeserializeError(e.to_string().into_cstr()) })?;
            TryInto::<Value>::try_into(core_value)
        }).response(result, error)
    }


#[no_mangle]
pub unsafe extern "C" fn cardano_value_checked_add(
    value: Value,
    rhs: Value,
    result: &mut Value,
    error: &mut CError,
) -> bool {
    handle_exception_result(|| {
        value
            .try_into()
            .zip(rhs.try_into())
            .and_then(|(value, rhs): (RValue, RValue)| value.checked_add(&rhs).into_result())
            .and_then(|value| value.try_into())
    })
    .response(result, error)
}

#[no_mangle]
pub unsafe extern "C" fn cardano_value_checked_sub(
    value: Value,
    rhs: Value,
    result: &mut Value,
    error: &mut CError,
) -> bool {
    handle_exception_result(|| {
        value
            .try_into()
            .zip(rhs.try_into())
            .and_then(|(value, rhs): (RValue, RValue)| value.checked_sub(&rhs).into_result())
            .and_then(|value| value.try_into())
    })
    .response(result, error)
}

#[no_mangle]
pub unsafe extern "C" fn cardano_value_clamped_sub(
    value: Value,
    rhs: Value,
    result: &mut Value,
    error: &mut CError,
) -> bool {
    handle_exception_result(|| {
        value
            .try_into()
            .zip(rhs.try_into())
            .map(|(value, rhs): (RValue, RValue)| value.clamped_sub(&rhs))
            .and_then(|value| value.try_into())
    })
    .response(result, error)
}

#[no_mangle]
pub unsafe extern "C" fn cardano_value_compare(
    value: Value,
    rhs: Value,
    result: &mut *mut i8,
    error: &mut CError,
) -> bool {
    handle_exception_result(|| {
        value
            .try_into()
            .zip(rhs.try_into())
            .map(|(value, rhs): (RValue, RValue)| value.compare(&rhs))
    })
    .response(result, error)
}

#[no_mangle]
pub unsafe extern "C" fn cardano_value_partial_cmp(
    value: Value,
    other: Value,
    result: &mut *mut COrdering,
    error: &mut CError,
) -> bool {
    handle_exception_result(|| {
        value
            .try_into()
            .zip(other.try_into())
            .map(|(value, other): (RValue, RValue)| value.partial_cmp(&other))
            .map(|ordering| ordering.map(|ordering| ordering.into()))
    })
    .response(result, error)
}

#[no_mangle]
pub unsafe extern "C" fn cardano_value_min_ada_required(
    assets: Value,
    has_data_hash: bool,
    coins_per_utxo_word: u64,
    result: &mut u64,
    error: &mut CError,
) -> bool {
    handle_exception_result(|| {
        assets
            .try_into()
            .and_then(|assets| {
                min_ada_required(&assets, has_data_hash, &to_bignum(coins_per_utxo_word))
                    .into_result()
            })
            .map(|big_num| from_bignum(&big_num))
    })
    .response(result, error)
}

#[no_mangle]
pub unsafe extern "C" fn cardano_value_clone(
    value: Value,
    result: &mut Value,
    error: &mut CError,
) -> bool {
    handle_exception(|| value.clone()).response(result, error)
}

#[no_mangle]
pub unsafe extern "C" fn cardano_value_free(value: &mut Value) {
    value.free();
}


