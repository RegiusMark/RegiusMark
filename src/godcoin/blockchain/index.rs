use rocksdb::{DB, ColumnFamilyDescriptor, Options};
use std::path::Path;
use serializer::*;

const CF_BLOCK_BYTE_POS: &str = "block_byte_pos";

const KEY_CHAIN_HEIGHT: &[u8] = b"chain_height";

pub struct Indexer {
    db: DB
}

impl Indexer {
    pub fn new(path: &Path) -> Indexer {
        let mut db_opts = Options::default();
        db_opts.create_missing_column_families(true);
        db_opts.create_if_missing(true);

        let col_families = vec![
            ColumnFamilyDescriptor::new(CF_BLOCK_BYTE_POS, Options::default())
        ];
        let db = DB::open_cf_descriptors(&db_opts, path, col_families).unwrap();
        Indexer { db }
    }

    pub fn get_block_byte_pos(&self, height: u64) -> Option<u64> {
        let cf = self.db.cf_handle(CF_BLOCK_BYTE_POS).unwrap();
        let buf = self.db.get_cf(cf, &{
            let mut key = Vec::with_capacity(8);
            key.push_u64(height);
            key
        }).unwrap()?;

        Some(u64_from_buf!(buf))
    }

    pub fn set_block_byte_pos(&self, height: u64, pos: u64) {
        let mut key = Vec::with_capacity(8);
        key.push_u64(height);
        let mut val = Vec::with_capacity(8);
        val.push_u64(pos);

        let cf = self.db.cf_handle(CF_BLOCK_BYTE_POS).unwrap();
        self.db.put_cf(cf, &key, &val).unwrap();
    }

    pub fn get_chain_height(&self) -> u64 {
        match self.db.get(KEY_CHAIN_HEIGHT).unwrap() {
            Some(val) => u64_from_buf!(val),
            None => 0
        }
    }

    pub fn set_chain_height(&self, height: u64) {
        let mut val = Vec::with_capacity(8);
        val.push_u64(height);
        self.db.put(KEY_CHAIN_HEIGHT, &val).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use rand::{thread_rng, Rng, distributions::Alphanumeric};
    use std::{env, fs, panic};
    use super::*;

    #[test]
    fn test_get_block_pos() {
        run_test(|indexer| {
            indexer.set_block_byte_pos(1, 327);
            assert!(indexer.get_block_byte_pos(0).is_none());
            assert_eq!(indexer.get_block_byte_pos(1).unwrap(), 327);
        });
    }

    #[test]
    fn test_get_chain_height() {
        run_test(|indexer| {
            assert_eq!(indexer.get_chain_height(), 0);
            indexer.set_chain_height(42);
            assert_eq!(indexer.get_chain_height(), 42);
        });
    }

    fn run_test<F>(func: F)
            where F: FnOnce(Indexer) -> () + panic::UnwindSafe {
        let mut tmp_dir = env::temp_dir();
        let mut s = String::from("godcoin_test_");
        s.push_str(&thread_rng().sample_iter(&Alphanumeric).take(4).collect::<String>());
        tmp_dir.push(s);
        fs::create_dir(&tmp_dir).expect(&format!("Could not create temp dir {:?}", &tmp_dir));

        let result = panic::catch_unwind(|| {
            let indexer = Indexer::new(&tmp_dir);
            func(indexer);
        });

        fs::remove_dir_all(&tmp_dir).expect("Failed to rm dir");
        assert!(result.is_ok());
    }
}