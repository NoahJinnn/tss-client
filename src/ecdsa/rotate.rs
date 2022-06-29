use crate::dto::ecdsa::PrivateShare;
use crate::utilities::err_handling::{error_to_c_string, ErrorFFIKind};
use crate::utilities::ffi::ffi_utils::{get_client_shim_from_raw, get_private_share_from_raw};
use crate::utilities::requests::ClientShim;

use super::super::utilities::requests;
use curv::cryptographic_primitives::twoparty::coin_flip_optimal_rounds;
use curv::elliptic::curves::secp256_k1::GE;

use anyhow::{anyhow, Result};
use kms::ecdsa::two_party::*;
use kms::rotation::two_party::party2::Rotation2;
use zk_paillier::zkproofs::SALT_STRING;

use std::ffi::CString;
use std::os::raw::c_char;

const ROT_PATH_PRE: &str = "ecdsa/rotate";

pub fn rotate_private_share(
    private_share: PrivateShare,
    client_shim: &ClientShim,
) -> Result<PrivateShare> {
    let id = &private_share.id.clone();
    let coin_flip_party1_first_message: coin_flip_optimal_rounds::Party1FirstMessage<GE> =
        match requests::post(client_shim, &format!("{}/{}/first", ROT_PATH_PRE, id)) {
            Ok(s) => match s {
                Some(s) => s,
                None => {
                    return Err(anyhow!("coin flip p1 first msg return None"));
                }
            },
            Err(e) => {
                return Err(anyhow!(
                    "coin flip p1 first msg request for rotating failed:\n {}",
                    e
                ));
            }
        };

    let coin_flip_party2_first_message =
        Rotation2::key_rotate_first_message(&coin_flip_party1_first_message);

    let body = &coin_flip_party2_first_message;

    let (coin_flip_party1_second_message, rotation_party1_first_message): (
        coin_flip_optimal_rounds::Party1SecondMessage<GE>,
        party1::RotationParty1Message1,
    ) = match requests::postb(
        client_shim,
        &format!("{}/{}/second", ROT_PATH_PRE, id.clone()),
        body,
    ) {
        Ok(s) => match s {
            Some(s) => s,
            None => {
                return Err(anyhow!("coin flip p1 second msg return None"));
            }
        },
        Err(e) => {
            return Err(anyhow!(
                "coin flip p1 second msg request for rotating failed:\n {}",
                e
            ));
        }
    };

    let random2 = Rotation2::key_rotate_second_message(
        &coin_flip_party1_second_message,
        &coin_flip_party2_first_message,
        &coin_flip_party1_first_message,
    );

    let party_two_master_key_rotated = match private_share.master_key.rotate_first_message(
        &random2,
        &rotation_party1_first_message,
        SALT_STRING,
    ) {
        Ok(s) => s,
        Err(_) => return Err(anyhow!("get rotated master key of p2 failed")),
    };

    let private_share = PrivateShare {
        id: private_share.id,
        master_key: party_two_master_key_rotated,
    };
    Ok(private_share)
}

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn get_rotated_client_master_key(
    c_endpoint: *const c_char,
    c_auth_token: *const c_char,
    c_user_id: *const c_char,
    c_private_share_json: *const c_char,
) -> *mut c_char {
    let client_shim = match get_client_shim_from_raw(c_endpoint, c_auth_token, c_user_id) {
        Ok(s) => s,
        Err(e) => return error_to_c_string(e),
    };

    let private_share: PrivateShare = match get_private_share_from_raw(c_private_share_json) {
        Ok(s) => s,
        Err(e) => return error_to_c_string(e),
    };

    let rotated_private_share: PrivateShare =
        match rotate_private_share(private_share, &client_shim) {
            Ok(s) => s,
            Err(e) => {
                return error_to_c_string(ErrorFFIKind::E103 {
                    msg: "rotated_private_share".to_owned(),
                    e: e.to_string(),
                })
            }
        };

    let private_share_json = match serde_json::to_string(&rotated_private_share) {
        Ok(share) => share,
        Err(e) => {
            return error_to_c_string(ErrorFFIKind::E102 {
                msg: "rotated_private_share".to_owned(),
                e: e.to_string(),
            })
        }
    };

    match CString::new(private_share_json) {
        Ok(s) => s.into_raw(),
        Err(e) => error_to_c_string(ErrorFFIKind::E101 {
            msg: "rotated_private_share".to_owned(),
            e: e.to_string(),
        }),
    }
}
