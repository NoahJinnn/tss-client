#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate log;

pub mod btc;
pub mod dto;
pub mod ecdsa;
pub mod escrow;
pub mod eth;
pub mod utilities;
pub mod wallet;

// pub mod eddsa;
// pub mod schnorr;

pub mod tests;

// pub use multi_party_eddsa::protocols::aggsig::*;
pub use curv::{arithmetic::traits::Converter, BigInt};
