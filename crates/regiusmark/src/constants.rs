use crate::asset::*;

pub const MARK_FEE_MIN: Asset = Asset::new(25);

pub const MARK_FEE_MULT: Asset = Asset::new(200_000);
pub const MARK_FEE_NET_MULT: Asset = Asset::new(101_500);

pub const NETWORK_FEE_AVG_WINDOW: u64 = 10;
pub const FEE_RESET_WINDOW: usize = 4;

pub const TX_EXPIRY_TIME: u64 = 30000;
pub const BLOCK_PROD_TIME: u64 = 3000;

pub const MAX_MEMO_BYTE_SIZE: usize = 1024;
pub const MAX_SCRIPT_BYTE_SIZE: usize = 2048;
pub const MAX_TX_SIGNATURES: usize = 8;

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn constants() {
        assert_eq!(MARK_FEE_MIN.to_string(), "0.00025 MARK");
        assert_eq!(MARK_FEE_MULT.to_string(), "2.00000 MARK");
        assert_eq!(MARK_FEE_NET_MULT.to_string(), "1.01500 MARK");
    }
}
