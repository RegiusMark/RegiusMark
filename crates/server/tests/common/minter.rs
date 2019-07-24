use super::create_tx_header;
use regius_mark::{
    blockchain::{GenesisBlockInfo, ReindexOpts},
    prelude::*,
};
use godcoin_server::{index, prelude::*, ServerData};
use sodiumoxide::randombytes;
use std::{
    env, fs,
    io::Cursor,
    path::{Path, PathBuf},
    sync::Arc,
};
use warp::Filter;

type Indexed = bool;

pub struct TestMinter(ServerData, GenesisBlockInfo, PathBuf, Indexed);

impl TestMinter {
    pub fn new() -> Self {
        regius_mark::init().unwrap();
        let tmp_dir = {
            let mut tmp_dir = env::temp_dir();
            let mut num: [u8; 8] = [0; 8];
            randombytes::randombytes_into(&mut num);
            tmp_dir.push(&format!("godcoin_test_{}", u64::from_be_bytes(num)));
            tmp_dir
        };
        fs::create_dir(&tmp_dir).expect(&format!("Could not create temp dir {:?}", &tmp_dir));

        let blocklog_loc = &Path::join(&tmp_dir, "blklog");
        let index_loc = &Path::join(&tmp_dir, "index");
        let chain = Arc::new(Blockchain::new(blocklog_loc, index_loc));
        let minter_key = KeyPair::gen();
        let info = chain.create_genesis_block(minter_key.clone());

        {
            let txs = {
                let mut txs = Vec::with_capacity(1);

                let mut tx = TxVariant::V0(TxVariantV0::MintTx(MintTx {
                    base: create_tx_header("0.00000 GRAEL"),
                    to: (&info.script).into(),
                    amount: "1000.00000 GRAEL".parse().unwrap(),
                    attachment: vec![1, 2, 3],
                    attachment_name: "".to_owned(),
                    script: info.script.clone(),
                }));

                tx.append_sign(&info.wallet_keys[1]);
                tx.append_sign(&info.wallet_keys[0]);
                txs.push(tx);

                txs.push(TxVariant::V0(TxVariantV0::RewardTx(RewardTx {
                    base: Tx {
                        fee: "0.00000 GRAEL".parse().unwrap(),
                        timestamp: 0,
                        signature_pairs: Vec::new(),
                    },
                    to: (&info.script).into(),
                    rewards: Asset::default(),
                })));
                txs
            };

            let head = chain.get_chain_head();
            let child = match head.as_ref() {
                SignedBlock::V0(block) => block.new_child(txs).sign(&info.minter_key),
            };
            chain.insert_block(child).unwrap();
        }

        let minter = Minter::new(Arc::clone(&chain), minter_key);
        let data = ServerData { chain, minter };
        Self(data, info, tmp_dir, true)
    }

    pub fn unindexed(&mut self) {
        let unindexed_path = {
            let mut unindexed_path = self.2.clone();
            let mut num: [u8; 8] = [0; 8];
            randombytes::randombytes_into(&mut num);
            unindexed_path.push(&format!("unindexed_{}", u64::from_be_bytes(num)));
            unindexed_path
        };
        fs::create_dir(&unindexed_path)
            .expect(&format!("Could not create temp dir {:?}", &unindexed_path));
        fs::copy(self.2.join("blklog"), unindexed_path.join("blklog"))
            .expect("Could not copy block log");

        let blocklog_loc = &Path::join(&unindexed_path, "blklog");
        let index_loc = &Path::join(&unindexed_path, "index");
        self.0.chain = Arc::new(Blockchain::new(blocklog_loc, index_loc));
        self.3 = false;
    }

    pub fn reindex(&mut self) {
        let chain = Arc::clone(&self.0.chain);
        assert_eq!(chain.index_status(), IndexStatus::None);
        chain.reindex(ReindexOpts { auto_trim: true });
        self.0.minter = Minter::new(chain, self.1.minter_key.clone());
        self.3 = true;
    }

    pub fn chain(&self) -> &Blockchain {
        &self.0.chain
    }

    pub fn genesis_info(&self) -> &GenesisBlockInfo {
        &self.1
    }

    pub fn produce_block(&self) -> Result<(), verify::BlockErr> {
        self.0.minter.force_produce_block()
    }

    pub fn request(&self, req: MsgRequest) -> MsgResponse {
        self.send_request(net::RequestType::Single(req))
            .unwrap_single()
    }

    pub fn batch_request(&self, reqs: Vec<MsgRequest>) -> Vec<MsgResponse> {
        self.send_request(net::RequestType::Batch(reqs))
            .unwrap_batch()
    }

    pub fn send_request(&self, req: net::RequestType) -> net::ResponseType {
        let mut buf = Vec::with_capacity(1_048_576);
        req.serialize(&mut buf);
        self.raw_request(&buf)
    }

    pub fn raw_request(&self, body: &[u8]) -> net::ResponseType {
        assert!(
            self.3,
            "attempting to send a request to an unindexed minter"
        );

        let data = Arc::new(self.0.clone());
        let filter = godcoin_server::app_filter!(data);
        let res = warp::test::request()
            .method("POST")
            .header("content-length", body.len())
            .body(body)
            .reply(&filter);
        let body = res.into_body();
        let mut cur = Cursor::<&[u8]>::new(&body);
        net::ResponseType::deserialize(&mut cur).unwrap()
    }
}

impl Drop for TestMinter {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.2).expect("Failed to rm dir");
    }
}
