use super::MempoolUtxos;
#[allow(unused_imports)]
use crate::panic::{handle_exception_result, CResponse, Result};
#[allow(unused_imports)]
use crate::ptr::Ptr;
use crate::string::IntoCString;
use crate::{data::CData, error::CError};
use cml_chain::transaction::Transaction;
#[allow(unused_imports)]
use cml_core::serialization::FromBytes;

#[no_mangle]
pub unsafe extern "C" fn cml_tx_utxo_result(
    transaction: CData,
    result: &mut MempoolUtxos,
    error: &mut CError,
) -> bool {
    handle_exception_result(|| {
        let x = transaction
            .unowned()
            .and_then(|tx_bytes| {
                Transaction::from_bytes(tx_bytes.to_vec())
                    .map_err(|_| CError::Error("Tx Build Error".into_cstr()))
            })
            .and_then(|txn| Ok(MempoolUtxos::from_tx(&txn)));

        x
    })
    .response(result, error)
}
