use asset::*;

pub const GOLD_FEE_MIN: Asset = Asset {
    amount: 100,
    decimals: 8,
    symbol: AssetSymbol::GOLD
};

pub const SILVER_FEE_MIN: Asset = Asset {
    amount: 1000,
    decimals: 8,
    symbol: AssetSymbol::SILVER
};

pub const GOLD_FEE_MULT: Asset = Asset {
    amount: 200000000,
    decimals: 8,
    symbol: AssetSymbol::GOLD
};

pub const SILVER_FEE_MULT: Asset = Asset {
    amount: 200000000,
    decimals: 8,
    symbol: AssetSymbol::SILVER
};

pub const BOND_FEE: Asset = Asset {
    amount: 500000000,
    decimals: 8,
    symbol: AssetSymbol::GOLD
};

pub const NETWORK_FEE_AVG_WINDOW: u64 = 10;
pub const FEE_RESET_WINDOW: u64 = 4;

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(GOLD_FEE_MIN.to_string(), "0.00000100 GOLD");
        assert_eq!(SILVER_FEE_MIN.to_string(), "0.00001000 SILVER");

        assert_eq!(BOND_FEE.to_string(), "5.00000000 GOLD");

        assert_eq!(GOLD_FEE_MULT.to_string(), "2.00000000 GOLD");
        assert_eq!(SILVER_FEE_MULT.to_string(), "2.00000000 SILVER");
    }
}