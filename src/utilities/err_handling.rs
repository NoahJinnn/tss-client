use std::ffi::CString;
use std::os::raw::c_char;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ErrorFFIKind {
    #[error("E100: Decode C string pointer to Rust &str error: {msg}\n {e}")]
    E100 { msg: String, e: String },
    #[error("E101: Encode &str to C string pointer error: {msg}\n {e}")]
    E101 { msg: String, e: String },
    #[error("E102: From struct to JSON parsing error: {msg}\n {e}")]
    E102 { msg: String, e: String },
    #[error("E103: TSS communication process error: {msg}\n {e}")]
    E103 { msg: String, e: String },
    #[error("E104: From JSON to struct parsing error: {msg}\n {e}")]
    E104 { msg: String, e: String },
}

pub fn error_to_c_string(e: ErrorFFIKind) -> *mut c_char {
    CString::new(format!("{}", e)).unwrap().into_raw()
}
