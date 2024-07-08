use std::sync::Arc;
use bincode::{Decode, Encode};
use crate::storage::StorageSystem;

#[derive(Encode, Decode, Debug)]
pub struct Currency {
    pub id: u64,
    pub symbol: String,
}

#[derive(Encode, Decode, Debug)]
pub struct Stock {
    pub id: u64,
    pub symbol: String,
}

pub struct AssetSystem {
    pub last_currency_id: u64,
    pub last_stock_id: u64,
    pub storage_system: Arc<StorageSystem>,
}

impl AssetSystem {
    pub fn new(storage_system: Arc<StorageSystem>) -> AssetSystem {
        let mut last_currency_id: u64 =0;
        match storage_system.get_last_currency() {
            None => {}
            Some(currency) => {
                last_currency_id = currency.id;
            }
        }
        let mut last_stock_id: u64 = 0;
        match storage_system.get_last_stock() {
            None => {}
            Some(stock) => {
                last_stock_id = stock.id;
            }
        }

        AssetSystem {
            last_currency_id,
            last_stock_id,
            storage_system,
        }
    }

    pub fn create_currency(&mut self, mut currency: Currency) -> u64 {
        self.last_currency_id += 1;
        currency.id = self.last_currency_id;
        self.storage_system.add_currency(&currency);
        self.last_currency_id
    }

    pub fn create_stock(&mut self, mut stock: Stock) -> u64 {
        self.last_stock_id += 1;
        stock.id = self.last_stock_id;
        self.storage_system.add_stock(&stock);
        self.last_stock_id
    }

    pub fn get_currencies(&self) -> Vec<Currency> {
        let mut currencies:Vec<Currency> = self.storage_system.load_currencies();
        currencies.sort_by(|a, b| a.id.cmp(&b.id));
        currencies
    }

    pub fn get_stocks(&self) -> Vec<Stock> {
        let mut stocks:Vec<Stock> = self.storage_system.load_stocks();
        stocks.sort_by(|a, b| a.id.cmp(&b.id));
        stocks
    }

}