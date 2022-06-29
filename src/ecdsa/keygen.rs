use anyhow::{anyhow, Result};
use floating_duration::TimeFormat;
use serde_json;
use std::time::Instant;

use curv::cryptographic_primitives::twoparty::dh_key_exchange_variant_with_pok_comm::*;
use curv::elliptic::curves::secp256_k1::GE;

use kms::chain_code::two_party as chain_code;
use kms::ecdsa::two_party::*;
use multi_party_ecdsa::protocols::two_party_ecdsa::lindell_2017::*;
use zk_paillier::zkproofs::SALT_STRING;

use crate::dto::ecdsa::PrivateShare;
use crate::utilities::err_handling::{error_to_c_string, ErrorFFIKind};
use crate::utilities::ffi::ffi_utils::get_client_shim_from_raw;
use crate::utilities::requests::ClientShim;

use super::super::utilities::requests;

use std::ffi::CString;
use std::os::raw::c_char;

const KG_PATH_PRE: &str = "ecdsa/keygen";

pub fn get_private_share(client_shim: &ClientShim) -> Result<PrivateShare> {
    let start = Instant::now();
    // Receive ECDH key exchange message from P1
    let (id, kg_party_one_first_message): (String, party_one::KeyGenFirstMsg) =
        match requests::post(client_shim, &format!("{}/first", KG_PATH_PRE))? {
            Some(s) => s,
            None => return Err(anyhow!("keygen first message request failed")),
        };

    let (kg_party_two_first_message, kg_ec_key_pair_party2) = MasterKey2::key_gen_first_message();

    let body = &kg_party_two_first_message.d_log_proof;

    // Send ECDH key exchange message to P1 & receive the Paillier pubkey from P1
    let kg_party_one_second_message: party1::KeyGenParty1Message2 =
        match requests::postb(client_shim, &format!("{}/{}/second", KG_PATH_PRE, id), body)? {
            Some(s) => s,
            None => return Err(anyhow!("keygen second message request failed")),
        };

    let (_, party_two_paillier) = match MasterKey2::key_gen_second_message(
        &kg_party_one_first_message,
        &kg_party_one_second_message,
        SALT_STRING,
    ) {
        Ok(s) => s,
        Err(_) => return Err(anyhow!("calculate paillier public failed")),
    };

    // Receive non-interactive zk proof from P1
    let cc_party_one_first_message: Party1FirstMessage = match requests::post(
        client_shim,
        &format!("{}/{}/chaincode/first", KG_PATH_PRE, id),
    )? {
        Some(s) => s,
        None => return Err(anyhow!("chaincode first message request failed")),
    };

    let (cc_party_two_first_message, cc_ec_key_pair2) =
        chain_code::party2::ChainCode2::chain_code_first_message();

    let body = &cc_party_two_first_message.d_log_proof;

    // Initiate 2-round zk proof with P1 & receive the decom proof from P1
    let cc_party_one_second_message: Party1SecondMessage<GE> = match requests::postb(
        client_shim,
        &format!("{}/{}/chaincode/second", KG_PATH_PRE, id),
        body,
    )? {
        Some(s) => s,
        None => return Err(anyhow!("chaincode second message request failed")),
    };

    let cc_party_two_second_message = chain_code::party2::ChainCode2::chain_code_second_message(
        &cc_party_one_first_message,
        &cc_party_one_second_message,
    );

    assert!(cc_party_two_second_message.is_ok());

    let party2_cc = chain_code::party2::ChainCode2::compute_chain_code(
        &cc_ec_key_pair2,
        &cc_party_one_second_message.comm_witness.public_share,
    )
    .chain_code;

    // Verify zk proof, generate c_key & paillier pubkey of P1
    let master_key = MasterKey2::set_master_key(
        &party2_cc,
        &kg_ec_key_pair_party2,
        &kg_party_one_second_message
            .ecdh_second_message
            .comm_witness
            .public_share,
        &party_two_paillier,
    );

    println!("(id: {}) Took: {}", id, TimeFormat(start.elapsed()));

    Ok(PrivateShare { id, master_key })
}

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn get_client_master_key(
    c_endpoint: *const c_char,
    c_auth_token: *const c_char,
    c_user_id: *const c_char,
) -> *mut c_char {
    let client_shim = match get_client_shim_from_raw(c_endpoint, c_auth_token, c_user_id) {
        Ok(s) => s,
        Err(e) => return error_to_c_string(e),
    };

    let private_share: PrivateShare = match get_private_share(&client_shim) {
        Ok(s) => s,
        Err(e) => {
            return error_to_c_string(ErrorFFIKind::E103 {
                msg: "private_share".to_owned(),
                e: e.to_string(),
            })
        }
    };

    let private_share_json = match serde_json::to_string(&private_share) {
        Ok(share) => share,
        Err(e) => {
            return error_to_c_string(ErrorFFIKind::E102 {
                msg: "private_share".to_owned(),
                e: e.to_string(),
            })
        }
    };

    match CString::new(private_share_json) {
        Ok(s) => s.into_raw(),
        Err(e) => error_to_c_string(ErrorFFIKind::E101 {
            msg: "private_share".to_owned(),
            e: e.to_string(),
        }),
    }
}
