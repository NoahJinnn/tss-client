use anyhow::{anyhow, Result};
use bitcoin::{self, Network};
use curv::elliptic::curves::secp256_k1::PK;
use curv::elliptic::curves::traits::ECPoint;
use curv::BigInt;
use kms::ecdsa::two_party::MasterKey2;

use crate::dto::btc::{BlockCypherAddress, UtxoAggregator};
use crate::dto::ecdsa::PrivateShare;
use crate::utilities::derive_new_key;

pub const BTC_TESTNET: &str = "testnet";
pub const BLOCK_CYPHER_HOST: &str = "https://api.blockcypher.com/v1/btc/test3"; // TODO: Centralize the config constants

pub fn list_unspent_for_addresss(address: String) -> Result<Vec<UtxoAggregator>> {
    let unspent_tx_url = BLOCK_CYPHER_HOST.to_owned() + "/addrs/" + &address + "?unspentOnly=true";
    let res = reqwest::blocking::get(unspent_tx_url)?.text()?;
    let address_balance_with_tx_refs: BlockCypherAddress = serde_json::from_str(res.as_str())?;
    if let Some(tx_refs) = address_balance_with_tx_refs.txrefs {
        Ok(tx_refs
            .iter()
            .map(|u| UtxoAggregator {
                value: u.value,
                height: u.block_height,
                tx_hash: u.tx_hash.clone(),
                tx_pos: u.tx_output_n,
                address: address.clone(),
            })
            .collect())
    } else {
        Ok(Vec::new())
    }
}

pub fn get_all_addresses(
    last_derived_pos: u32,
    private_share: &PrivateShare,
) -> Result<Vec<bitcoin::Address>> {
    let init = 0;
    let last_pos = last_derived_pos;

    let mut response: Vec<bitcoin::Address> = Vec::new();

    for n in init..=last_pos {
        let mk = private_share
            .master_key
            .get_child(vec![BigInt::from(0), BigInt::from(n)]);

        let bitcoin_address = to_bitcoin_address(BTC_TESTNET, &mk)?;

        response.push(bitcoin_address);
    }

    Ok(response)
}

pub fn get_new_address(
    private_share: &PrivateShare,
    last_derived_pos: u32,
) -> Result<bitcoin::Address> {
    let (_pos, mk) = derive_new_key(private_share, last_derived_pos);
    to_bitcoin_address(BTC_TESTNET, &mk)
}

pub fn to_bitcoin_address(network: &str, mk: &MasterKey2) -> Result<bitcoin::Address> {
    let pk = mk.public.q.get_element();
    match bitcoin::Address::p2wpkh(&to_bitcoin_public_key(pk), get_bitcoin_network(network)?) {
        Ok(address) => Ok(address),
        Err(e) => Err(anyhow!("Error while creating bitcoin address: {}", e)),
    }
}

pub fn to_bitcoin_public_key(pk: PK) -> bitcoin::util::key::PublicKey {
    bitcoin::util::key::PublicKey {
        compressed: true,
        key: pk,
    }
}

pub fn get_bitcoin_network(nw: &str) -> Result<Network> {
    let btc_nw = nw.to_owned().parse::<Network>()?;
    Ok(btc_nw)
}
