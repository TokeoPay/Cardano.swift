use super::array::*;
use super::asset_name::AssetName;
use super::error::CError;
use super::panic::*;
use super::ptr::*;

use cml_chain::assets::AssetName as CML_AssetName;
use cml_chain::OrderedHashMap;

use cardano_serialization_lib::{
    utils::{from_bignum, to_bignum},
    Assets as RAssets,
};
use std::convert::{TryFrom, TryInto};

pub type AssetNames = CArray<AssetName>;

#[no_mangle]
pub unsafe extern "C" fn cardano_asset_names_free(asset_names: &mut AssetNames) {
    asset_names.free();
}

pub type AssetsKeyValue = CKeyValue<AssetName, u64>;
pub type Assets = CArray<AssetsKeyValue>;

impl TryFrom<OrderedHashMap<CML_AssetName, u64>> for Assets {
    type Error = CError;
    fn try_from(value: OrderedHashMap<CML_AssetName, u64>) -> Result<Self> {
        let x = value
            .iter()
            .map(|(asset_name, amt)| {
                TryInto::<AssetName>::try_into(asset_name.clone())
                    .map(|asset_name| (asset_name, amt.clone()))
            })
            .collect::<Result<Vec<_>>>()
            .map(|res| Into::<Assets>::into(res));

        x.into()
    }
}

impl TryFrom<Assets> for RAssets {
    type Error = CError;

    fn try_from(assets: Assets) -> Result<Self> {
        let map = unsafe { assets.as_btree_map()? };
        let mut assets = RAssets::new();
        for (name, bn) in map {
            let name = name.try_into()?;
            assets.insert(&name, &to_bignum(bn));
        }
        Ok(assets)
    }
}

impl TryFrom<RAssets> for Assets {
    type Error = CError;

    fn try_from(assets: RAssets) -> Result<Self> {
        Ok(assets.keys()).and_then(|names| {
            (0..names.len())
                .map(|index| names.get(index))
                .map(|name| {
                    assets
                        .get(&name)
                        .ok_or("Cannot get BigNum by AssetName".into())
                        .zip(name.try_into())
                        .map(|(bn, name)| (name, from_bignum(&bn)).into())
                })
                .collect::<Result<Vec<AssetsKeyValue>>>()
                .map(|assets| assets.into())
        })
    }
}

#[no_mangle]
pub unsafe extern "C" fn cardano_assets_free(assets: &mut Assets) {
    assets.free();
}
