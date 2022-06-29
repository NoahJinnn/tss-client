use web3::types::{AccessList, Address, Bytes, H256, U256, U64};

#[derive(Serialize, Deserialize)]
pub struct EthTxParamsReqBody {
    pub from_address: Address,
    pub to_address: Address,
    pub eth_value: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EthTxParamsResp {
    pub to: Option<Address>,
    pub nonce: U256,
    pub gas: U256,
    pub gas_price: U256,
    pub value: U256,
    pub data: Vec<u8>,
    pub transaction_type: Option<U64>,
    pub access_list: AccessList,
    pub max_priority_fee_per_gas: U256,
    pub chain_id: u64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct EthSendTxResp {
    pub tx_hash: H256,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct EthSendTxReqBody {
    pub raw_tx: Bytes,
}
