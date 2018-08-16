use sodiumoxide::crypto::hash::sha256::Digest;
use std::time::{SystemTime, UNIX_EPOCH};
use std::ops::Deref;
use std::io::Cursor;

use crypto::{KeyPair, double_sha256};
use serializer::*;
use crypto;
use tx::*;

#[derive(Debug, Clone)]
pub struct Block {
    pub previous_hash: Digest,
    pub height: u64,
    pub timestamp: u32,
    pub tx_merkle_root: Digest,
    pub transactions: Vec<TxVariant>
}

impl Block {
    pub fn sign(self, key_pair: &KeyPair) -> SignedBlock {
        let mut buf = Vec::new();
        self.encode(&mut buf);
        SignedBlock {
            base: self,
            sig_pair: key_pair.sign(&buf)
        }
    }

    pub fn encode_with_tx(&self, vec: &mut Vec<u8>) {
        self.encode(vec);

        vec.push_u32(self.transactions.len() as u32);
        for tx in &self.transactions { tx.encode_with_sigs(vec) };
    }

    pub fn decode_with_tx(cur: &mut Cursor<&[u8]>) -> Option<Self> {
        let previous_hash = Digest::from_slice(&cur.take_bytes()?)?;
        let height = cur.take_u64()?;
        let timestamp = cur.take_u32()?;
        let tx_merkle_root = Digest::from_slice(&cur.take_bytes()?)?;

        let len = cur.take_u32()?;
        let mut transactions = Vec::<TxVariant>::with_capacity(len as usize);
        for _ in 0..len {
            transactions.push(TxVariant::decode_with_sigs(cur)?);
        }

        Some(Self {
            previous_hash,
            height,
            timestamp,
            tx_merkle_root,
            transactions
        })
    }

    fn encode(&self, vec: &mut Vec<u8>) {
        vec.push_bytes(self.previous_hash.as_ref());
        vec.push_u64(self.height);
        vec.push_u32(self.timestamp);
        vec.push_bytes(self.tx_merkle_root.as_ref());
    }
}

#[derive(Debug, Clone)]
pub struct SignedBlock {
    pub base: Block,
    pub sig_pair: crypto::SigPair
}

impl SignedBlock {
    pub fn new_child(&self, prev_block: &Self, txs: Vec<TxVariant>) -> Block {
        let previous_hash = {
            let mut buf = Vec::new();
            prev_block.base.encode(&mut buf);
            double_sha256(&buf)
        };
        let tx_merkle_root = {
            let mut buf = Vec::new();
            for tx in &txs { tx.encode_with_sigs(&mut buf) };
            double_sha256(&buf)
        };
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        Block {
            previous_hash,
            height: prev_block.height + 1,
            timestamp: timestamp as u32,
            tx_merkle_root,
            transactions: txs
        }
    }

    pub fn encode_with_tx(&self, vec: &mut Vec<u8>) {
        self.base.encode_with_tx(vec);
        vec.push_sig_pair(&self.sig_pair);
    }

    pub fn decode_with_tx(cur: &mut Cursor<&[u8]>) -> Option<Self> {
        let block = Block::decode_with_tx(cur)?;
        let sig_pair = cur.take_sig_pair()?;
        Some(Self {
            base: block,
            sig_pair
        })
    }
}

impl Deref for SignedBlock {
    type Target = Block;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crypto::KeyPair;
    use asset::Asset;
    use super::*;

    #[test]
    fn test_serialize_block() {
        let keys = KeyPair::gen_keypair();
        let transactions = {
            let mut vec = Vec::new();
            vec.push(TxVariant::RewardTx(RewardTx {
                base: Tx {
                    tx_type: TxType::REWARD,
                    fee: Asset::from_str("0 GOLD").unwrap(),
                    timestamp: 1234567890,
                    signature_pairs: Vec::new()
                },
                to: keys.0.clone(),
                rewards: Vec::new()
            }));
            vec
        };
        let tx_merkle_root = {
            let mut buf = Vec::new();
            for tx in &transactions { tx.encode_with_sigs(&mut buf) };
            double_sha256(&buf)
        };
        let block = (Block {
            previous_hash: Digest::from_slice(&[0u8; 32]).unwrap(),
            height: 123,
            timestamp: 1532992800,
            tx_merkle_root,
            transactions
        }).sign(&keys);

        let mut buf = Vec::new();
        block.encode_with_tx(&mut buf);

        let mut cur = Cursor::<&[u8]>::new(&buf);
        let dec = SignedBlock::decode_with_tx(&mut cur).unwrap();

        assert_eq!(block.previous_hash, dec.previous_hash);
        assert_eq!(block.height, dec.height);
        assert_eq!(block.timestamp, dec.timestamp);
        assert_eq!(block.tx_merkle_root, dec.tx_merkle_root);
        assert_eq!(block.transactions.len(), dec.transactions.len());
        assert_eq!(block.sig_pair, dec.sig_pair);
    }
}