use anyhow::{anyhow, Result};
use curv::BigInt;
use kms::ecdsa::two_party::party2;
use kms::ecdsa::two_party::MasterKey2;
use multi_party_ecdsa::protocols::two_party_ecdsa::lindell_2017::party_one;
use multi_party_ecdsa::protocols::two_party_ecdsa::lindell_2017::party_two;

use super::super::utilities::requests;
use crate::dto::ecdsa::SignSecondMsgRequest;
use crate::utilities::err_handling::error_to_c_string;
use crate::utilities::err_handling::ErrorFFIKind;
use crate::utilities::ffi::ffi_utils::get_client_shim_from_raw;
use crate::utilities::ffi::ffi_utils::get_str_from_c_char;
use crate::utilities::requests::ClientShim;

// iOS bindings
use std::ffi::CString;
use std::os::raw::c_char;

pub fn sign(
    client_shim: &ClientShim,
    message: BigInt,
    mk: &MasterKey2,
    x_pos: BigInt,
    y_pos: BigInt,
    id: &str,
) -> Result<party_one::SignatureRecid> {
    // Choose ephemeral key
    let (eph_key_gen_first_message_party_two, eph_comm_witness, eph_ec_key_pair_party2) =
        MasterKey2::sign_first_message();

    let request: party_two::EphKeyGenFirstMsg = eph_key_gen_first_message_party_two;

    // Repeat Key Generation protocol for ephemeral key to obtain random point on curve that will be used in generating signature
    let sign_party_one_first_message: party_one::EphKeyGenFirstMsg =
        match requests::postb(client_shim, &format!("/ecdsa/sign/{}/first", id), &request)? {
            Some(s) => s,
            None => return Err(anyhow!("party1 sign first message request failed")),
        };

    // Generate encryption of derivative of the signature, called c3
    let party_two_sign_message = mk.sign_second_message(
        &eph_ec_key_pair_party2,
        eph_comm_witness,
        &sign_party_one_first_message,
        &message,
    );

    // Send c3 to P1 to verify and get valid signature
    let signature = match get_signature(
        client_shim,
        message,
        party_two_sign_message,
        x_pos,
        y_pos,
        id,
    ) {
        Ok(s) => s,
        Err(e) => return Err(anyhow!("ecdsa::get_signature failed failed: {}", e)),
    };

    Ok(signature)
}

fn get_signature(
    client_shim: &ClientShim,
    message: BigInt,
    party_two_sign_message: party2::SignMessage,
    x_pos_child_key: BigInt,
    y_pos_child_key: BigInt,
    id: &str,
) -> Result<party_one::SignatureRecid> {
    let request: SignSecondMsgRequest = SignSecondMsgRequest {
        message,
        party_two_sign_message,
        x_pos_child_key,
        y_pos_child_key,
    };

    let signature: party_one::SignatureRecid =
        match requests::postb(client_shim, &format!("/ecdsa/sign/{}/second", id), &request)? {
            Some(s) => s,
            None => return Err(anyhow!("party1 sign second message request failed",)),
        };

    Ok(signature)
}

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn sign_message(
    c_endpoint: *const c_char,
    c_auth_token: *const c_char,
    c_user_id: *const c_char,
    c_message_le_hex: *const c_char,
    c_master_key_json: *const c_char,
    c_x_pos: i32,
    c_y_pos: i32,
    c_id: *const c_char,
) -> *mut c_char {
    let client_shim = match get_client_shim_from_raw(c_endpoint, c_auth_token, c_user_id) {
        Ok(s) => s,
        Err(e) => return error_to_c_string(e),
    };

    let message_hex = match get_str_from_c_char(c_message_le_hex, "message_hex") {
        Ok(s) => s,
        Err(e) => return error_to_c_string(e),
    };

    let master_key_json = match get_str_from_c_char(c_master_key_json, "master_key_json") {
        Ok(s) => s,
        Err(e) => return error_to_c_string(e),
    };

    let id = match get_str_from_c_char(c_id, "id") {
        Ok(s) => s,
        Err(e) => return error_to_c_string(e),
    };

    let x: BigInt = BigInt::from(c_x_pos);
    let y: BigInt = BigInt::from(c_y_pos);

    let mk: MasterKey2 = match serde_json::from_str(&master_key_json) {
        Ok(s) => s,
        Err(e) => {
            return error_to_c_string(ErrorFFIKind::E104 {
                msg: "sign_mk".to_owned(),
                e: e.to_string(),
            })
        }
    };

    let mk_child: MasterKey2 = mk.get_child(vec![x.clone(), y.clone()]);
    let message: BigInt = match serde_json::from_str(&message_hex) {
        Ok(s) => s,
        Err(e) => {
            return error_to_c_string(ErrorFFIKind::E104 {
                msg: "sign_message".to_owned(),
                e: e.to_string(),
            })
        }
    };

    let sig = match sign(&client_shim, message, &mk_child, x, y, &id) {
        Ok(s) => s,
        Err(e) => {
            return error_to_c_string(ErrorFFIKind::E103 {
                msg: "sig".to_owned(),
                e: e.to_string(),
            })
        }
    };

    let signature_json = match serde_json::to_string(&sig) {
        Ok(share) => share,
        Err(e) => {
            return error_to_c_string(ErrorFFIKind::E103 {
                msg: "signature_json".to_owned(),
                e: e.to_string(),
            })
        }
    };

    CString::new(signature_json).unwrap().into_raw()
}
