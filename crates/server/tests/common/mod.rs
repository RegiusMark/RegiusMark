use godcoin::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

pub mod minter;
pub use minter::*;

pub fn get_balance(gold: &str, silver: &str) -> Balance {
    let gold = gold.parse().unwrap();
    let silver = silver.parse().unwrap();
    Balance::from(gold, silver).unwrap()
}

pub fn create_tx(tx_type: TxType, fee: &str) -> Tx {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    Tx {
        tx_type,
        timestamp,
        fee: fee.parse().unwrap(),
        signature_pairs: Vec::with_capacity(8),
    }
}
