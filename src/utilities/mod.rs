pub mod a_requests;
pub mod err_handling;
pub mod ffi;
pub mod requests;

use crate::dto::ecdsa::PrivateShare;
use curv::BigInt;
use kms::ecdsa::two_party::MasterKey2;

pub fn derive_new_key(private_share: &PrivateShare, pos: u32) -> (u32, MasterKey2) {
    let last_pos: u32 = pos + 1;

    let last_child_master_key = private_share
        .master_key
        .get_child(vec![BigInt::from(0), BigInt::from(last_pos)]);

    (last_pos, last_child_master_key)
}
