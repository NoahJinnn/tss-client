# Organization
## Test assets
Folder `test-assets` includes JSON files that represent **test wallet** data.

## Test arrange utilities
`src/tests/common.rs`

## Test suites
- ecdsa
- btc
- eth
# Steps to test

## 1. Fill test wallet funds with faucets in case we run out of money
- BTC: https://bitcoinfaucet.uo1.net/send.php, https://testnet-faucet.mempool.co/, https://coinfaucet.eu/en/btc-testnet/
- ETH: https://rinkebyfaucet.com/, https://rinkeby-faucet.com/

## 2. Run test
```bash
cargo test --verbose # all test suites
cargo test --verbose -- --nocapture # all test suites with logs
cargo test --verbose <mod name> -- --nocapture # specific test suite
```

## 3. Check address balance on explorers
to address: where we send money to
from address: where we keep money

BTC: https://live.blockcypher.com/btc-testnet/ 
- the `output addresses` of the transactions must include: 
    - the to (lowest pos) address: `tb1qz4lma0u0xyepgkzlsegxfxw7e65ue7azhkck5m`
    - the change (highest pos) address which will become the from address later on 

ETH: https://rinkeby.etherscan.io/ 
- the to address: `0x70045eea879fb025026e59efa099dbf99b2657db`
- the from address: `0xb3d0a620d31d064542b88b9d699e5fe7cc52565c`

# NOTES:
- Not have test coverage yet
- Not have negative cases yet (Use `should_panic`)
- Not have error-checking cases yet (Assert returned error)
- Not have bench/bounded cases yet (min/max/0/ranging/repeatability)
