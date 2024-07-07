use std::collections::HashMap;
use bincode::{Decode, Encode};
use crate::storage::StorageSystem;

#[derive(Encode, Decode)]
pub struct Currency {
    pub id: u64,
    pub symbol: String,
}

#[derive(Encode, Decode)]
pub struct Stock {
    pub id: u64,
    pub symbol: String,
}

pub struct AssetSystem {
    pub currencies: HashMap<u64,Currency>,
    pub stocks: HashMap<u64,Stock>,
}

impl AssetSystem {
    pub fn new(storage_system: &StorageSystem) -> AssetSystem {
        let currencies_vec = storage_system.load_currencies();
        let mut currencies = HashMap::new();
        for currency in currencies_vec {
            currencies.insert(currency.id, currency);
        }
        let stocks_vec = storage_system.load_stocks();
        let mut stocks = HashMap::new();
        for stock in stocks_vec {
            stocks.insert(stock.id, stock);
        }
        AssetSystem {
            currencies,
            stocks,
        }
    }
}