use curv::elliptic::curves::secp256_k1::{FE, GE};
use curv::elliptic::curves::traits::ECPoint;
use curv::elliptic::curves::traits::ECScalar;

use serde_json;
use std::fs;

const ESCROW_SK_FILENAME: &str = "escrow/escrow-sk.json";

pub const SEGMENT_SIZE: usize = 8;
pub const NUM_SEGMENTS: usize = 32;

#[derive(Serialize, Deserialize, Debug)]
pub struct Escrow {
    pub secret: FE,
    pub public: GE,
}

impl Escrow {
    pub fn new() -> Escrow {
        let secret: FE = ECScalar::new_random();
        let g: GE = ECPoint::generator();
        let public: GE = g * secret;
        fs::write(
            ESCROW_SK_FILENAME,
            serde_json::to_string(&(secret, public)).unwrap(),
        )
        .expect("Unable to save escrow secret!");

        Escrow { secret, public }
    }

    pub fn load() -> Escrow {
        let sec_data = fs::read_to_string(ESCROW_SK_FILENAME).expect("Unable to load wallet!");
        let (secret, public): (FE, GE) = serde_json::from_str(&sec_data).unwrap();
        Escrow { secret, public }
    }

    pub fn get_public_key(&self) -> GE {
        self.public
    }

    pub fn get_private_key(&self) -> FE {
        self.secret
    }
}

impl Default for Escrow {
    fn default() -> Self {
        Escrow::new()
    }
}
