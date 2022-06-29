use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
};

use crate::{
    dto::ecdsa::{MKPosAddressDto, PrivateShare},
    utilities::{
        derive_new_key,
        err_handling::{error_to_c_string, ErrorFFIKind},
    },
};

use super::utils::pubkey_to_eth_address;

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn get_eth_addrs(
    c_private_share_json: *const c_char,
    c_last_derived_pos: u32,
) -> *mut c_char {
    let raw_private_share_json = unsafe { CStr::from_ptr(c_private_share_json) };
    let private_share_json = match raw_private_share_json.to_str() {
        Ok(s) => s,
        Err(_) => panic!("Error while decoding raw private share"),
    };
    let private_share: PrivateShare = serde_json::from_str(private_share_json).unwrap();

    let (pos, mk) = derive_new_key(&private_share, c_last_derived_pos);
    let address = pubkey_to_eth_address(&mk);

    let mk_pos_address = MKPosAddressDto {
        address: format!("{:?}", address),
        pos,
        mk,
    };

    let mk_pos_address_json = match serde_json::to_string(&mk_pos_address) {
        Ok(addrs_resp) => addrs_resp,
        Err(e) => {
            return error_to_c_string(ErrorFFIKind::E102 {
                msg: "mk_pos_address".to_owned(),
                e: e.to_string(),
            })
        }
    };

    match CString::new(mk_pos_address_json) {
        Ok(s) => s.into_raw(),
        Err(e) => error_to_c_string(ErrorFFIKind::E101 {
            msg: "mk_pos_address".to_owned(),
            e: e.to_string(),
        }),
    }
}
