use curv::BigInt;
use kms::ecdsa::two_party::MasterKey2;
use kms::ecdsa::two_party::*;

#[derive(Serialize, Deserialize)]
pub struct PrivateShare {
    pub id: String,
    pub master_key: MasterKey2,
}

impl PrivateShare {
    pub fn get_child(&self, path: Vec<BigInt>) -> PrivateShare {
        let child_key = self.master_key.get_child(path);
        PrivateShare {
            id: self.id.clone(),
            master_key: child_key,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SignSecondMsgRequest {
    pub message: BigInt,
    pub party_two_sign_message: party2::SignMessage,
    pub x_pos_child_key: BigInt,
    pub y_pos_child_key: BigInt,
}

#[derive(Serialize, Deserialize)]
pub struct MKPosDto {
    pub pos: u32,
    pub mk: MasterKey2,
}

#[derive(Serialize, Deserialize)]
pub struct MKPosAddressDto {
    pub address: String,
    pub pos: u32,
    pub mk: MasterKey2,
}
