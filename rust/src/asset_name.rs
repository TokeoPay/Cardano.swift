use crate::string::IntoCString;

use super::data::CData;
use super::error::CError;
use super::panic::*;
use super::ptr::*;
use cardano_serialization_lib::AssetName as RAssetName;
use cml_chain::assets::AssetName as CML_AssetName;
use std::convert::{TryFrom, TryInto};
use cml_core::serialization::Serialize;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AssetName {
    bytes: [u8; 32],
    len: u8,
}

impl AssetName {
    pub fn bytes(&self) -> Vec<u8> {
        self.bytes.into()
    }
}

impl Free for AssetName {
    unsafe fn free(&mut self) {}
}

impl TryFrom<RAssetName> for AssetName {
    type Error = CError;

    fn try_from(asset: RAssetName) -> Result<Self> {
        let mut name = asset.name();
        let len = name.len();
        if len < 32 {
            name.append(&mut vec![0; 32 - len]);
        }
        let bytes: [u8; 32] = name.try_into().map_err(|_| CError::DataLengthMismatch)?;
        Ok(Self {
            bytes,
            len: len as u8,
        })
    }
}

impl TryFrom<AssetName> for RAssetName {
    type Error = CError;

    fn try_from(asset: AssetName) -> Result<Self> {
        let mut bytes = Vec::from(asset.bytes);
        bytes.truncate(asset.len.into());
        RAssetName::new(bytes).into_result()
    }
}

impl TryFrom<CML_AssetName> for AssetName {
    type Error = CError;

    fn try_from(value: CML_AssetName) -> Result<Self> {
        let mut name = value.to_cbor_bytes().clone();
        let len = name.len();
        if len < 32 {
            name.append(&mut vec![0; 32 - len]);
        }
        let bytes: [u8; 32] = name.try_into().map_err(|_| CError::DataLengthMismatch)?;

        Ok(Self {
            bytes: bytes.clone(),
            len: len as u8,
        })
    }
}

// impl TryFrom<CML_AssetName> for AssetName {
//     type Error = CError;

//     fn try_from(asset: CML_AssetName) -> std::prelude::v1::Result<Self, Self::Error> {
//         let mut name = asset.inner;
//         let len = name.len();
//         if len < 32 {
//             name.append(&mut vec![0; 32 - len]);
//         }
//         let bytes: [u8; 32] = name.try_into().map_err(|_| CError::DataLengthMismatch)?;
//         Ok(Self {
//             bytes,
//             len: len as u8,
//         })
//     }
// }

impl TryFrom<AssetName> for CML_AssetName {
    type Error = CError;

    fn try_from(asset: AssetName) -> Result<Self> {
        Self::new(asset.bytes.to_vec()).map_err(|e| CError::Error(e.to_string().into_cstr()))
    }
}

#[no_mangle]
pub unsafe extern "C" fn cardano_asset_name_to_bytes(
    asset_name: AssetName,
    result: &mut CData,
    error: &mut CError,
) -> bool {
    handle_exception_result(|| {
        asset_name
            .try_into()
            .map(|name: RAssetName| name.to_bytes().into())
    })
    .response(result, error)
}

#[no_mangle]
pub unsafe extern "C" fn cardano_asset_name_from_bytes(
    data: CData,
    result: &mut AssetName,
    error: &mut CError,
) -> bool {
    handle_exception_result(|| {
        data.unowned()
            .and_then(|bytes| RAssetName::from_bytes(bytes.into()).into_result())
            .and_then(|asset| asset.try_into())
    })
    .response(result, error)
}

#[no_mangle]
pub unsafe extern "C" fn cardano_asset_name_new(
    data: CData,
    result: &mut AssetName,
    error: &mut CError,
) -> bool {
    handle_exception_result(|| {
        data.unowned()
            .and_then(|bytes| RAssetName::new(bytes.into()).into_result())
            .and_then(|asset| asset.try_into())
    })
    .response(result, error)
}

#[no_mangle]
pub unsafe extern "C" fn cardano_asset_name_get_name(
    asset_name: AssetName,
    result: &mut CData,
    error: &mut CError,
) -> bool {
    handle_exception_result(|| {
        asset_name
            .try_into()
            .map(|name: RAssetName| name.name().into())
    })
    .response(result, error)
}
