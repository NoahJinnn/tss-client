#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub mod ffi_utils {
    use anyhow::Result;
    use std::{
        collections::HashMap,
        ffi::{CStr, CString},
        os::raw::c_char,
    };

    use crate::{
        dto::ecdsa::{MKPosDto, PrivateShare},
        utilities::{err_handling::ErrorFFIKind, requests::ClientShim},
    };

    pub fn get_str_from_c_char(c: *const c_char, err_msg: &str) -> Result<String, ErrorFFIKind> {
        let raw = unsafe { CStr::from_ptr(c) };
        let s = match raw.to_str() {
            Ok(s) => s,
            Err(e) => {
                return Err(ErrorFFIKind::E100 {
                    msg: err_msg.to_owned(),
                    e: e.to_string(),
                })
            }
        };

        Ok(s.to_string())
    }

    pub fn get_client_shim_from_raw(
        c_endpoint: *const c_char,
        c_auth_token: *const c_char,
        c_user_id: *const c_char,
    ) -> Result<ClientShim, ErrorFFIKind> {
        let endpoint = get_str_from_c_char(c_endpoint, "endpoint")?;
        let auth_token = get_str_from_c_char(c_auth_token, "auth_token")?;
        let user_id = get_str_from_c_char(c_user_id, "user_id")?;

        Ok(ClientShim::new(endpoint, Some(auth_token), user_id))
    }

    pub fn get_private_share_from_raw(
        c_private_share_json: *const c_char,
    ) -> Result<PrivateShare, ErrorFFIKind> {
        let private_share_json = get_str_from_c_char(c_private_share_json, "private_share_json")?;

        let private_share: PrivateShare = match serde_json::from_str(&private_share_json) {
            Ok(s) => s,
            Err(e) => {
                return Err(ErrorFFIKind::E104 {
                    msg: "private_share".to_owned(),
                    e: e.to_string(),
                })
            }
        };

        Ok(private_share)
    }

    pub fn get_addresses_derivation_map_from_raw(
        c_addresses_derivation_map: *const c_char,
    ) -> Result<HashMap<String, MKPosDto>, ErrorFFIKind> {
        let addresses_derivation_map_json =
            get_str_from_c_char(c_addresses_derivation_map, "addresses_derivation_map_json")?;

        let addresses_derivation_map: HashMap<String, MKPosDto> =
            match serde_json::from_str(&addresses_derivation_map_json) {
                Ok(s) => s,
                Err(e) => {
                    return Err(ErrorFFIKind::E104 {
                        msg: "addresses_derivation_map_json".to_owned(),
                        e: e.to_string(),
                    })
                }
            };

        Ok(addresses_derivation_map)
    }

    #[no_mangle]
    pub extern "C" fn cstring_free(cstring: *mut c_char) {
        if cstring.is_null() {
            return;
        }
        unsafe { CString::from_raw(cstring) };
    }
}
