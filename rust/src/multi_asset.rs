use crate::option::COption;
use crate::string::IntoCString;

use super::array::*;
use super::assets::Assets;
use super::error::CError;
use super::panic::*;
use super::ptr::Free;
use super::stake_credential::ScriptHash;
use cardano_serialization_lib::MultiAsset as RMultiAsset;
use cml_chain::assets::AssetName as CML_AssetName;
use cml_chain::OrderedHashMap;
use std::convert::{TryFrom, TryInto};
use std::iter::FromIterator;

use cml_chain::assets::AssetBundle;

pub type PolicyID = ScriptHash;
pub type MultiAssetKeyValue = CKeyValue<PolicyID, Assets>;
pub type MultiAsset = CArray<MultiAssetKeyValue>;

impl Free for PolicyID {
    unsafe fn free(&mut self) {}
}

impl TryFrom<COption<MultiAsset>> for AssetBundle<u64> {
  type Error = CError;
  
  fn try_from(value: COption<MultiAsset>) -> std::prelude::v1::Result<Self, Self::Error> {
        todo!()
    }
}

impl TryFrom<MultiAsset> for AssetBundle<u64> {
    type Error = CError;

    fn try_from(value: MultiAsset) -> std::prelude::v1::Result<Self, Self::Error> {
        let map = unsafe { value.as_btree_map()? };

        let mut multi_assets = Self::new();

        map.into_iter().map(|(script_hash, assets)| {
            TryInto::<cml_crypto::ScriptHash>::try_into(script_hash).map(|policy_id| {
                let assets_map = unsafe { assets.as_btree_map() }.into_result();

                let cml_assets: Result<Vec<(CML_AssetName, u64)>> = assets_map.and_then(|x| {
                    let res = (x.iter()
                        .map(|(asset_name, value)| {
                            
                            let bbb: Result<(CML_AssetName, u64)> = CML_AssetName::new(asset_name.bytes())
                                .map_err(|e| CError::Error("".into_cstr()))
                                .and_then(|cml_asset_name| Ok((cml_asset_name, value.clone())));
                            return bbb;
                            
                        })
                        .collect::<Result<Vec<_>>>()
                      ).unwrap_or(Vec::new());
                    
                    Ok(res)
                }); //;

                multi_assets.insert(policy_id, OrderedHashMap::from_iter(cml_assets.unwrap_or(Vec::new())));
            })
        });

        return Ok(multi_assets);
    }
}

impl TryFrom<MultiAsset> for RMultiAsset {
    type Error = CError;

    fn try_from(multi_asset: MultiAsset) -> Result<Self> {
        let map = unsafe { multi_asset.as_btree_map()? };
        let mut multi_asset = RMultiAsset::new();
        for (pid, assets) in map {
            let assets = assets.try_into()?;
            multi_asset.insert(&pid.into(), &assets);
        }
        Ok(multi_asset)
    }
}

impl TryFrom<RMultiAsset> for MultiAsset {
    type Error = CError;

    fn try_from(multi_asset: RMultiAsset) -> Result<Self> {
        Ok(multi_asset.keys()).and_then(|pids| {
            (0..pids.len())
                .map(|index| pids.get(index))
                .map(|pid| {
                    multi_asset
                        .get(&pid)
                        .ok_or("Cannot get Assets by PolicyID".into())
                        .and_then(|assets| assets.try_into())
                        .zip(pid.try_into())
                        .map(|(assets, pid)| (pid, assets).into())
                })
                .collect::<Result<Vec<MultiAssetKeyValue>>>()
                .map(|multi_asset| multi_asset.into())
        })
    }
}

#[no_mangle]
pub unsafe extern "C" fn cardano_multi_asset_sub(
    multi_asset: MultiAsset,
    rhs_ma: MultiAsset,
    result: &mut MultiAsset,
    error: &mut CError,
) -> bool {
    handle_exception_result(|| {
        multi_asset
            .try_into()
            .zip(rhs_ma.try_into())
            .map(|(multi_asset, rhs_ma): (RMultiAsset, RMultiAsset)| multi_asset.sub(&rhs_ma))
            .and_then(|multi_asset| multi_asset.try_into())
    })
    .response(result, error)
}

#[no_mangle]
pub unsafe extern "C" fn cardano_multi_asset_free(multi_asset: &mut MultiAsset) {
    multi_asset.free()
}
