use anyhow::Result;
use bitcoin::{self};
use curv::elliptic::curves::secp256_k1::GE;
use curv::elliptic::curves::traits::ECPoint;
use curv::BigInt;
use kms::ecdsa::two_party::MasterKey2;
use kms::ecdsa::two_party::*;
use serde_json::{self, Value};
use std::fs;
use web3::types::H256;

use centipede::juggling::proof_system::{Helgamalsegmented, Proof};
use centipede::juggling::segmentation::Msegmentation;

use crate::btc::raw_tx::select_tx_in;
use crate::btc::utils::{get_bitcoin_network, to_bitcoin_address, to_bitcoin_public_key};
use crate::dto::btc::BlockCypherRawTx;
use crate::dto::ecdsa::{MKPosDto, PrivateShare};
use crate::ecdsa::recover::{backup_client_mk, verify_client_backup};
use crate::eth;
use crate::eth::raw_tx::sign_and_send;
use crate::eth::utils::pubkey_to_eth_address;
use crate::tests::common::RINKEBY_TEST_API;
use crate::utilities::derive_new_key;
use crate::utilities::requests::ClientShim;

use super::btc;

use super::ecdsa;
use super::escrow;
use super::utilities::requests;
use std::collections::HashMap;

// TODO: move that to a config file and double check electrum server addresses
pub const WALLET_FILENAME: &str = "wallet/wallet.json";
const BACKUP_FILENAME: &str = "wallet/backup.data";
const BLOCK_CYPHER_HOST: &str = "https://api.blockcypher.com/v1/btc/test3";
#[derive(Serialize, Deserialize)]
pub struct Wallet {
    pub id: String,
    pub coin_type: String,
    pub network: String,
    pub private_share: PrivateShare,
    pub last_derived_pos: u32,
    pub addresses_derivation_map: HashMap<String, MKPosDto>,
}

impl Wallet {
    pub fn new(client_shim: &ClientShim, net: &str, c_type: &str) -> Wallet {
        // let id = Uuid::new_v4().to_string();
        let private_share = match ecdsa::get_private_share(client_shim) {
            Ok(p) => p,
            Err(e) => panic!("{}", e),
        };

        let last_derived_pos = 0;
        let addresses_derivation_map = HashMap::new();

        Wallet {
            id: private_share.id.clone(),
            coin_type: c_type.to_owned(),
            network: net.to_owned(),
            private_share,
            last_derived_pos,
            addresses_derivation_map,
        }
    }

    pub fn rotate(self, client_shim: &ClientShim, filepath: &str) {
        let rotated_private_share =
            ecdsa::rotate_private_share(self.private_share, client_shim).unwrap();
        let addresses_derivation_map = HashMap::new();
        let mut wallet_after_rotate = Wallet {
            id: self.id.clone(),
            coin_type: self.coin_type.clone(),
            network: self.network.clone(),
            private_share: rotated_private_share,
            last_derived_pos: self.last_derived_pos,
            addresses_derivation_map,
        };
        wallet_after_rotate.derived().unwrap();

        wallet_after_rotate.save_to(filepath);
    }

    pub fn backup(&self) {
        let client_backup_json = backup_client_mk(&self.private_share).unwrap();
        fs::write(BACKUP_FILENAME, client_backup_json).expect("Unable to save client backup!");

        debug!("(wallet id: {}) Backup wallet with escrow", &self.id);
    }

    pub fn verify_backup(&self, escrow_service: escrow::Escrow) {
        let y = escrow_service.get_public_key();
        let data = fs::read_to_string(BACKUP_FILENAME).expect("Unable to load client backup!");
        match verify_client_backup(y, &data) {
            Ok(_x) => println!("backup verified 🍻"),
            Err(_e) => println!("Backup was not verified correctly 😲"),
        }
    }

    pub fn recover_and_save_share(
        escrow_service: escrow::Escrow,
        net: &str,
        client_shim: &ClientShim,
    ) -> Wallet {
        let g: GE = ECPoint::generator();
        let y_priv = escrow_service.get_private_key();

        let data = fs::read_to_string(BACKUP_FILENAME).expect("Unable to load client backup!");

        let (encryptions, _proof, public_data, chain_code2, key_id): (
            Helgamalsegmented,
            Proof,
            Party2Public,
            BigInt,
            String,
        ) = serde_json::from_str(&data).unwrap();

        let sk = Msegmentation::decrypt(&encryptions, &g, &y_priv, &escrow::SEGMENT_SIZE);

        let client_master_key_recovered =
            MasterKey2::recover_master_key(sk.unwrap(), public_data, chain_code2);
        let pos_old: u32 = requests::post(client_shim, &format!("ecdsa/{}/recover", key_id))
            .unwrap()
            .unwrap();

        let pos_old = if pos_old < 10 { 10 } else { pos_old };
        //TODO: temporary, server will keep updated pos, to do so we need to send update to server for every get_new_address

        // let id = Uuid::new_v4().to_string();
        let addresses_derivation_map = HashMap::new(); //TODO: add a fucntion to recreate

        let new_wallet = Wallet {
            id: key_id.clone(),
            coin_type: "btc".to_owned(),
            network: net.to_owned(),
            private_share: PrivateShare {
                master_key: client_master_key_recovered,
                id: key_id,
            },
            last_derived_pos: pos_old,
            addresses_derivation_map,
        };

        new_wallet.save();
        println!("Recovery Completed Successfully ❤️");

        new_wallet
    }

    pub fn save_to(&self, filepath: &str) {
        let wallet_json = serde_json::to_string(self).unwrap();

        fs::write(filepath, wallet_json).expect("Unable to save wallet!");

        debug!("(wallet id: {}) Saved wallet to disk", self.id);
    }

    pub fn save(&self) {
        self.save_to(WALLET_FILENAME)
    }

    pub fn load_from(filepath: &str) -> Wallet {
        let data = fs::read_to_string(filepath).expect("Unable to load wallet!");

        let wallet: Wallet = serde_json::from_str(&data).unwrap();

        debug!("(wallet id: {}) Loaded wallet to memory", wallet.id);

        wallet
    }

    pub fn load() -> Wallet {
        Wallet::load_from(WALLET_FILENAME)
    }

    pub fn send(
        &mut self,
        from_address: &str,
        to_address: &str,
        amount: f64,
        client_shim: &ClientShim,
    ) -> String {
        let coin_type = &self.coin_type;
        if coin_type == "btc" {
            let raw_tx_opt = btc::raw_tx::create_raw_tx(
                to_address,
                amount,
                client_shim,
                self.last_derived_pos,
                &self.private_share,
                &self.addresses_derivation_map,
            );

            let raw_tx_opt = match raw_tx_opt {
                Ok(tx) => tx,
                Err(e) => {
                    panic!("Unable to create raw transaction {}", e);
                }
            };

            let raw_tx = raw_tx_opt.unwrap();
            let change_address_payload = raw_tx.change_address_payload;

            let _ = &self.addresses_derivation_map.insert(
                change_address_payload.address,
                MKPosDto {
                    mk: change_address_payload.mk,
                    pos: change_address_payload.pos,
                },
            );
            self.last_derived_pos = &self.last_derived_pos + 1;

            let raw_tx_url = BLOCK_CYPHER_HOST.to_owned() + "/txs/push";
            let raw_tx = BlockCypherRawTx {
                tx: raw_tx.raw_tx_hex,
            };
            let tx_resp_str = reqwest::blocking::Client::new()
                .post(raw_tx_url)
                .json(&raw_tx)
                .send()
                .unwrap()
                .text()
                .unwrap();
            println!(
                "Network: [{}], Sent {} BTC to address {}. Transaction State: {}",
                &self.network, amount, &to_address, tx_resp_str
            );
            let tx_obj: Value = match serde_json::from_str(&tx_resp_str) {
                Ok(tx) => tx,
                Err(e) => {
                    println!("Unable to parse tx response {}", e);
                    return "".to_owned();
                }
            };
            let tx_hash = match &tx_obj["tx"]["hash"] {
                Value::String(s) => s,
                _ => {
                    println!("Unable to get tx hash");
                    return "".to_owned();
                }
            };
            return tx_hash.to_owned();
        } else if coin_type == "eth" {
            let tx_hash = send_eth(
                amount,
                client_shim,
                from_address,
                to_address,
                &self.private_share,
                &self.addresses_derivation_map,
            )
            .unwrap();

            println!(
                "Sent {} ETH to address {}. Transaction State: {:?}",
                amount, &to_address, tx_hash
            );
            return tx_hash.to_string();
        }
        "".to_owned()
    }

    pub fn get_crypto_address(&mut self) -> String {
        let (pos, mk) = derive_new_key(&self.private_share, self.last_derived_pos);
        let coin_type = &self.coin_type;
        if coin_type == "btc" {
            let address = to_bitcoin_address(&self.network, &mk).unwrap();
            self.addresses_derivation_map
                .insert(address.to_string(), MKPosDto { mk, pos });
            self.last_derived_pos = pos;

            println!("BTC Network: [{}], Address: [{}]", &self.network, address);
            return address.to_string();
        } else if coin_type == "eth" {
            let address = pubkey_to_eth_address(&mk);
            self.addresses_derivation_map
                .insert(format!("{:?}", address), MKPosDto { mk, pos });
            self.last_derived_pos = pos;

            println!("ETH address: {:?}", address);
            return address.to_string();
        }
        "".to_owned()
    }

    pub fn derived(&mut self) -> Result<()> {
        if self.coin_type == "btc" {
            for i in 0..self.last_derived_pos {
                let (pos, mk) = derive_new_key(&self.private_share, i);

                let address = bitcoin::Address::p2wpkh(
                    &to_bitcoin_public_key(mk.public.q.get_element()),
                    get_bitcoin_network(&self.network)?,
                )
                .expect(
                    "Cannot panic because `to_bitcoin_public_key` creates a compressed address",
                );

                self.addresses_derivation_map
                    .insert(address.to_string(), MKPosDto { mk, pos });
            }
        } else if self.coin_type == "eth" {
            for i in 0..self.last_derived_pos {
                let (pos, mk) = derive_new_key(&self.private_share, i);

                let address = pubkey_to_eth_address(&mk);

                self.addresses_derivation_map
                    .insert(format!("{:?}", address), MKPosDto { mk, pos });
            }
        }
        Ok(())
    }

    pub fn get_balance(&mut self) -> usize {
        let coin_type = &self.coin_type;
        if coin_type == "btc" {
            let mut total = 0;
            for b in select_tx_in(self.last_derived_pos, &self.private_share).unwrap() {
                total += b.value;
            }
            println!(
                "Network: [{}], Balance: [balance: {}]",
                &self.network, total
            );
            return total;
        } else if coin_type == "eth" {
            let total: f64 = get_eth_balance(self.last_derived_pos, &self.private_share).unwrap();
            println!("ETH Balance: [{}]", total);
            return (total * 1000.0) as usize; // multiply 1000 to get value that is greater than 0
        }
        0
    }
}

#[tokio::main]
async fn get_eth_balance(last_derived_pos: u32, private_share: &PrivateShare) -> Result<f64> {
    let balance_l =
        eth::utils::get_all_addresses_balance(RINKEBY_TEST_API, last_derived_pos, private_share)
            .await?;

    let mut total = 0.0;
    for b in balance_l {
        total += b
    }

    Ok(total)
}

fn send_eth(
    eth_value: f64,
    client_shim: &ClientShim,
    from: &str,
    to: &str,
    private_share: &PrivateShare,
    addresses_derivation_map: &HashMap<String, MKPosDto>,
) -> Result<H256> {
    let result = sign_and_send(
        from,
        to,
        eth_value,
        client_shim,
        private_share,
        addresses_derivation_map,
    )?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use bitcoin::hashes::hex::ToHex;
    use bitcoin::hashes::sha256d;
    use bitcoin::hashes::Hash;
    use curv::arithmetic::traits::Converter;
    use curv::BigInt;

    #[test]
    fn test_message_conv() {
        let message: [u8; 32] = [
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ];

        // 14abf5ed107ff58bf844ee7f447bec317c276b00905c09a45434f8848599597e
        let hash = sha256d::Hash::from_slice(&message).unwrap();

        // 7e59998584f83454a4095c90006b277c31ec7b447fee44f88bf57f10edf5ab14
        let ser = hash.to_hex();

        // 57149727877124134702546803488322951680010683936655914236113461592936003513108
        let b: BigInt = BigInt::from_hex(&ser).unwrap();

        println!("({},{},{})", hash, ser, b.to_hex());
    }
}
