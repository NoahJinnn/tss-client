#[derive(Debug, Deserialize, Clone)]
pub struct UtxoAggregator {
    pub height: isize,
    pub tx_hash: String,
    pub tx_pos: usize,
    pub value: usize,
    pub address: String,
}

// BLOCKCYPHER DTO
#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct BlockCypherAddress {
    pub address: String,
    pub total_received: u64,
    pub total_sent: u64,
    pub balance: u64,
    pub unconfirmed_balance: i64,
    pub final_balance: u64,
    pub n_tx: u64,
    pub unconfirmed_n_tx: u64,
    pub final_n_tx: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub txrefs: Option<Vec<BlockCypherTxRef>>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct BlockCypherTxRef {
    pub tx_hash: String,
    pub block_height: isize,
    pub tx_input_n: isize,
    pub tx_output_n: usize,
    pub value: usize,
    pub ref_balance: u64,
    pub spent: bool,
    pub confirmations: u64,
    pub confirmed: String,
}

#[derive(Serialize, Deserialize)]
pub struct BlockCypherRawTx {
    pub tx: String,
}
