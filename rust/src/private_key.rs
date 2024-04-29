use crate::string::IntoCString;
use super::data::CData;
use super::ed25519_signature::Ed25519Signature;
use super::error::CError;
use super::panic::*;
use super::ptr::*;
use super::public_key::PublicKey;
use super::address::address::Address;
use cardano_message_signing as ms;
use cardano_serialization_lib::{
  address::Address as RAddress,
    crypto::PrivateKey as RPrivateKey, impl_mockchain::key::EitherEd25519SecretKey,
};
use ms::cbor::CBORValue;
use ms::utils::ToBytes;
use std::convert::{TryFrom, TryInto};

pub const EXTENDED_PRIVATE_KEY_LENGTH: usize = 64;
pub const NORMAL_PRIVATE_KEY_LENGTH: usize = 32;

#[repr(C)]
#[derive(Copy, Clone)]
pub enum PrivateKey {
    Extended([u8; EXTENDED_PRIVATE_KEY_LENGTH]),
    Normal([u8; NORMAL_PRIVATE_KEY_LENGTH]),
}

impl TryFrom<PrivateKey> for RPrivateKey {
    type Error = CError;

    fn try_from(private_key: PrivateKey) -> Result<Self> {
        match private_key {
            PrivateKey::Extended(bytes) => RPrivateKey::from_extended_bytes(&bytes).into_result(),
            PrivateKey::Normal(bytes) => RPrivateKey::from_normal_bytes(&bytes).into_result(),
        }
    }
}

// For transmutation
struct TPrivateKey(EitherEd25519SecretKey);

impl From<RPrivateKey> for PrivateKey {
    fn from(private_key: RPrivateKey) -> Self {
        let bytes = private_key.as_bytes();
        let tpkey: TPrivateKey = unsafe { std::mem::transmute(private_key) };
        match tpkey.0 {
            EitherEd25519SecretKey::Extended(_) => PrivateKey::Extended(bytes.try_into().unwrap()),
            EitherEd25519SecretKey::Normal(_) => PrivateKey::Normal(bytes.try_into().unwrap()),
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn cardano_private_key_to_public(
    private_key: PrivateKey,
    result: &mut PublicKey,
    error: &mut CError,
) -> bool {
    handle_exception_result(|| {
        private_key
            .try_into()
            .map(|private_key: RPrivateKey| private_key.to_public().into())
    })
    .response(result, error)
}

#[no_mangle]
pub unsafe extern "C" fn cardano_private_key_as_bytes(
    private_key: PrivateKey,
    result: &mut CData,
    error: &mut CError,
) -> bool {
    handle_exception_result(|| {
        private_key
            .try_into()
            .map(|private_key: RPrivateKey| private_key.as_bytes().into())
    })
    .response(result, error)
}

#[no_mangle]
pub unsafe extern "C" fn cardano_private_key_from_extended_bytes(
    data: CData,
    result: &mut PrivateKey,
    error: &mut CError,
) -> bool {
    handle_exception_result(|| {
        data.unowned()
            .and_then(|bytes| RPrivateKey::from_extended_bytes(bytes).into_result())
            .map(|private_key| private_key.into())
    })
    .response(result, error)
}

#[no_mangle]
pub unsafe extern "C" fn cardano_private_key_from_normal_bytes(
    data: CData,
    result: &mut PrivateKey,
    error: &mut CError,
) -> bool {
    handle_exception_result(|| {
        data.unowned()
            .and_then(|bytes| RPrivateKey::from_normal_bytes(bytes).into_result())
            .map(|private_key| private_key.into())
    })
    .response(result, error)
}

#[no_mangle]
pub unsafe extern "C" fn cardano_private_key_sign(
    private_key: PrivateKey,
    message: CData,
    result: &mut Ed25519Signature,
    error: &mut CError,
) -> bool {
    handle_exception_result(|| {
        private_key
            .try_into()
            .zip(message.unowned())
            .map(|(private_key, message): (RPrivateKey, &[u8])| private_key.sign(message))
            .map(|ed25519_signature| ed25519_signature.into())
    })
    .response(result, error)
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct DataSignature {
    signature: CData,
    key: CData,
}

#[no_mangle]
pub unsafe extern "C" fn cardano_private_key_sign_data(
    address: Address,
    private_key: PrivateKey,
    message: CData,
    result: &mut DataSignature,
    error: &mut CError,
) -> bool {
    handle_exception_result(|| {
        let res: std::prelude::v1::Result<std::prelude::v1::Result<DataSignature, CError>, CError> =

            TryInto::<RAddress>::try_into(address).zip(
              TryInto::<RPrivateKey>::try_into(private_key)
            ).map(|(addr, sk)| {
                let payload: &[u8] = message.unowned().unwrap();

                let mut key = ms::COSEKey::new(&ms::Label::new_int(&ms::utils::Int::new_i32(0)));

                key.set_algorithm_id(&ms::Label::from_algorithm_id(
                    ms::builders::AlgorithmId::EdDSA,
                ));

                match key
                    .set_header(
                        &ms::Label::new_int(&ms::utils::Int::new_i32(1)),
                        &CBORValue::new_int(&ms::utils::Int::new_i32(1)),
                    )
                    .map_err(|js_err| {
                        CError::Error(
                            js_err
                                .as_string()
                                .unwrap_or("Error".to_string())
                                .into_cstr(),
                        )
                    }) {
                    Ok(_) => (),
                    Err(e) => return Err(e),
                };
                match key
                    .set_header(
                        &ms::Label::new_int(&ms::utils::Int::new_i32(3)),
                        &CBORValue::new_int(&ms::utils::Int::new_i32(-8)),
                    )
                    .map_err(|js_err| {
                        CError::Error(
                            js_err
                                .as_string()
                                .unwrap_or("Error".to_string())
                                .into_cstr(),
                        )
                    }) {
                    Ok(_) => (),
                    Err(e) => return Err(e),
                };
                match key
                    .set_header(
                        &ms::Label::new_int(&ms::utils::Int::new_i32(-1)),
                        &CBORValue::new_int(&ms::utils::Int::new_i32(6)),
                    )
                    .map_err(|js_err| {
                        CError::Error(
                            js_err
                                .as_string()
                                .unwrap_or("Error".to_string())
                                .into_cstr(),
                        )
                    }) {
                    Ok(_) => (),
                    Err(e) => return Err(e),
                };
                match key
                    .set_header(
                        &ms::Label::new_int(&ms::utils::Int::new_i32(-2)),
                        &CBORValue::new_bytes(sk.to_public().as_bytes()),
                    )
                    .map_err(|js_err| {
                        CError::Error(
                            js_err
                                .as_string()
                                .unwrap_or("Error".to_string())
                                .into_cstr(),
                        )
                    }) {
                    Ok(_) => (),
                    Err(e) => return Err(e),
                };

                let mut protected_headers = ms::HeaderMap::new();

                protected_headers.set_algorithm_id(&ms::Label::from_algorithm_id(
                    ms::builders::AlgorithmId::EdDSA,
                ));
                protected_headers.set_key_id(sk.to_public().as_bytes());

                match protected_headers
                    .set_header(
                        &ms::Label::new_text("address".to_string()),
                        &CBORValue::new_bytes(addr.to_bytes()),
                    )
                    .map_err(|js_err| {
                        CError::Error(
                            js_err
                                .as_string()
                                .unwrap_or("Error".to_string())
                                .into_cstr(),
                        )
                    }) {
                    Ok(_) => (),
                    Err(e) => return Err(e),
                }

                let protected_serialized = ms::ProtectedHeaderMap::new(&protected_headers);
                let unprotected = ms::HeaderMap::new();
                let headers = ms::Headers::new(&protected_serialized, &unprotected);

                let builder =
                    ms::builders::COSESign1Builder::new(&headers, payload.to_vec(), false);
                // builder.set_external_aad(external_aad.clone());

                let to_sign = builder.make_data_to_sign().to_bytes();
                let signed_sig_struct = sk.sign(&to_sign).to_bytes();
                let cose_sign1 = builder.build(signed_sig_struct);
                Ok(DataSignature {
                    signature: cose_sign1.to_bytes().into(),
                    key: key.to_bytes().into(),
                })
            });

        match res {
            Ok(x) => x,
            Err(e) => Err(e),
        }
    })
    .response(result, error)
}
