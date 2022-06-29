use anyhow::Result;
use curv::{elliptic::curves::traits::ECPoint, BigInt};
use futures::future::try_join_all;
use kms::ecdsa::two_party::MasterKey2;
use std::str::FromStr;
use web3::{
    self,
    signing::keccak256,
    transports::{self, WebSocket},
    types::{Address, U256},
    Web3,
};

use crate::dto::ecdsa::PrivateShare;

pub async fn get_all_addresses_balance(
    web3_connection_url: &str,
    last_derived_pos: u32,
    private_share: &PrivateShare,
) -> Result<Vec<f64>> {
    let web3_connection = establish_web3_connection(web3_connection_url).await?;
    let addresses = get_all_addresses(last_derived_pos, private_share).unwrap();
    let result: Vec<f64> = try_join_all(
        addresses
            .iter()
            .map(|a| get_balance_in_eth(format!("{:?}", a), &web3_connection)),
    )
    .await?;
    Ok(result)
}

pub fn get_all_addresses(
    last_derived_pos: u32,
    private_share: &PrivateShare,
) -> Result<Vec<Address>> {
    let init = 0;
    let last_pos = last_derived_pos;

    let mut response: Vec<Address> = Vec::new();

    for n in init..=last_pos {
        let mk = private_share
            .master_key
            .get_child(vec![BigInt::from(0), BigInt::from(n)]);

        let eth_address = pubkey_to_eth_address(&mk);
        response.push(eth_address);
    }

    Ok(response)
}

pub fn pubkey_to_eth_address(mk: &MasterKey2) -> Address {
    let pub_k = mk.public.q.get_element().serialize_uncompressed();
    let hash = keccak256(&pub_k[1..]);
    Address::from_slice(&hash[12..])
}

pub async fn get_balance_in_eth(
    public_address: String,
    web3_connection: &Web3<transports::WebSocket>,
) -> Result<f64> {
    let wei_balance = get_balance(public_address, web3_connection).await?;
    Ok(wei_to_eth(wei_balance))
}

async fn get_balance(public_address: String, web3_connection: &Web3<WebSocket>) -> Result<U256> {
    let wallet_address = Address::from_str(public_address.as_str())?;
    let balance = web3_connection.eth().balance(wallet_address, None).await?;
    Ok(balance)
}

pub fn wei_to_eth(wei_val: U256) -> f64 {
    let res = wei_val.as_u128() as f64;
    res / 1_000_000_000_000_000_000.0
}

pub async fn establish_web3_connection(url: &str) -> Result<Web3<transports::WebSocket>> {
    let transport = transports::WebSocket::new(url).await?;
    Ok(Web3::new(transport))
}
