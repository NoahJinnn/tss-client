use std::collections::HashMap;
use std::str::FromStr;

use crate::dto::ecdsa::MKPosDto;
use crate::dto::ecdsa::PrivateShare;
use crate::dto::eth::{EthSendTxReqBody, EthSendTxResp, EthTxParamsReqBody, EthTxParamsResp};
use crate::ecdsa::sign::sign;
use crate::eth::transaction::Transaction;
use crate::eth::utils::pubkey_to_eth_address;
use crate::utilities::err_handling::{error_to_c_string, ErrorFFIKind};
use crate::utilities::ffi::ffi_utils::{
    get_addresses_derivation_map_from_raw, get_client_shim_from_raw, get_private_share_from_raw,
};
use crate::utilities::requests::{self, ClientShim};

use anyhow::{anyhow, Result};
use curv::arithmetic::traits::Converter;
use curv::BigInt;
use hex;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use web3::types::{Address, H256};
use web3::{self, signing::Signature};

pub fn sign_and_send(
    from: &str,
    to: &str,
    eth_value: f64,
    client_shim: &ClientShim,
    private_share: &PrivateShare,
    addresses_derivation_map: &HashMap<String, MKPosDto>,
) -> Result<H256> {
    let pos_mk = match addresses_derivation_map.get(from.to_lowercase().as_str()) {
        Some(pos_mk) => pos_mk,
        None => {
            return Err(anyhow!(
                "from address not found in addresses_derivation_map"
            ))
        }
    };
    let mk = &pos_mk.mk;
    let pos = pos_mk.pos;

    let from_address = pubkey_to_eth_address(mk);
    let to_address = Address::from_str(to)?;

    let tx_params_body = EthTxParamsReqBody {
        from_address,
        to_address,
        eth_value,
    };

    let tx_params: EthTxParamsResp =
        match requests::postb(client_shim, "eth/tx/params", tx_params_body)? {
            Some(s) => s,
            None => return Err(anyhow!("get ETH tx params request failed")),
        };

    let tx = Transaction {
        to: tx_params.to,
        nonce: tx_params.nonce,
        gas: tx_params.gas,
        gas_price: tx_params.gas_price,
        value: tx_params.value,
        data: tx_params.data,
        transaction_type: tx_params.transaction_type,
        access_list: tx_params.access_list,
        max_priority_fee_per_gas: tx_params.max_priority_fee_per_gas,
    };
    let chain_id = tx_params.chain_id;
    let msg = tx.get_hash(chain_id);

    let sig = sign(
        client_shim,
        BigInt::from_hex(&hex::encode(&msg[..])).unwrap(),
        mk,
        BigInt::from(0),
        BigInt::from(pos),
        &private_share.id,
    )?;

    let r = H256::from_slice(&BigInt::to_bytes(&sig.r));
    let s = H256::from_slice(&BigInt::to_bytes(&sig.s));
    let v = sig.recid as u64 + 35 + chain_id * 2;
    let signature = Signature { r, s, v };
    let signed = tx.sign(signature, chain_id);

    let tx_send_body = EthSendTxReqBody {
        raw_tx: signed.raw_transaction,
    };

    let transaction_result: EthSendTxResp =
        match requests::postb(client_shim, "eth/tx/send", tx_send_body)? {
            Some(s) => s,
            None => return Err(anyhow!("send ETH tx request failed")),
        };

    Ok(transaction_result.tx_hash)
}

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn send_eth_tx(
    c_endpoint: *const c_char,
    c_auth_token: *const c_char,
    c_user_id: *const c_char,
    c_from_address: *const c_char,
    c_to_address: *const c_char,
    c_amount_eth: f64,
    c_private_share_json: *const c_char,
    c_addresses_derivation_map: *const c_char,
) -> *mut c_char {
    let raw_from_address = unsafe { CStr::from_ptr(c_from_address) };
    let from_address = match raw_from_address.to_str() {
        Ok(s) => s,
        Err(e) => {
            return error_to_c_string(ErrorFFIKind::E100 {
                msg: "from_address".to_owned(),
                e: e.to_string(),
            })
        }
    };

    let raw_to_address = unsafe { CStr::from_ptr(c_to_address) };
    let to_address = match raw_to_address.to_str() {
        Ok(s) => s,
        Err(e) => {
            return error_to_c_string(ErrorFFIKind::E100 {
                msg: "to_address".to_owned(),
                e: e.to_string(),
            })
        }
    };

    let client_shim = match get_client_shim_from_raw(c_endpoint, c_auth_token, c_user_id) {
        Ok(s) => s,
        Err(e) => {
            return error_to_c_string(ErrorFFIKind::E100 {
                msg: "client_shim".to_owned(),
                e: e.to_string(),
            })
        }
    };

    let private_share = match get_private_share_from_raw(c_private_share_json) {
        Ok(s) => s,
        Err(e) => return error_to_c_string(e),
    };

    let addresses_derivation_map =
        match get_addresses_derivation_map_from_raw(c_addresses_derivation_map) {
            Ok(s) => s,
            Err(e) => return error_to_c_string(e),
        };

    let tx_hash = match sign_and_send(
        from_address,
        to_address,
        c_amount_eth,
        &client_shim,
        &private_share,
        &addresses_derivation_map,
    ) {
        Ok(s) => s,
        Err(e) => {
            return error_to_c_string(ErrorFFIKind::E103 {
                msg: "tx_hash".to_owned(),
                e: e.to_string(),
            })
        }
    };

    let tx_hash_json = match serde_json::to_string(&tx_hash) {
        Ok(tx_resp) => tx_resp,
        Err(e) => {
            return error_to_c_string(ErrorFFIKind::E102 {
                msg: "tx_hash".to_owned(),
                e: e.to_string(),
            })
        }
    };

    match CString::new(tx_hash_json) {
        Ok(s) => s.into_raw(),
        Err(e) => error_to_c_string(ErrorFFIKind::E101 {
            msg: "tx_hash".to_owned(),
            e: e.to_string(),
        }),
    }
}
