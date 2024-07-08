use std::sync::Arc;
use bincode::{Decode, Encode};
use crate::storage::StorageSystem;

#[derive(Encode, Decode, Debug)]
pub struct Currency {
    pub id: u64,
    pub symbol: String,
}

#[derive(Encode, Decode, Debug)]
pub struct CryptoCurrency {
    pub id: u64,
    pub symbol: String,
}

pub struct AssetSystem {
    pub last_currency_id: u64,
    pub last_crypto_currency_id: u64,
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
        let mut last_crypto_currency_id: u64 = 0;
        match storage_system.get_last_crypto_currency() {
            None => {}
            Some(crypto_currency) => {
                last_crypto_currency_id = crypto_currency.id;
            }
        }

        AssetSystem {
            last_currency_id,
            last_crypto_currency_id,
            storage_system,
        }
    }

    pub fn create_currency(&mut self, mut currency: Currency) -> u64 {
        self.last_currency_id += 1;
        currency.id = self.last_currency_id;
        self.storage_system.add_currency(&currency);
        self.last_currency_id
    }

    pub fn create_crypto_currency(&mut self, mut crypto_currency: CryptoCurrency) -> u64 {
        self.last_crypto_currency_id += 1;
        crypto_currency.id = self.last_crypto_currency_id;
        self.storage_system.add_crypto_currency(&crypto_currency);
        self.last_crypto_currency_id
    }

    pub fn get_currencies(&self) -> Vec<Currency> {
        let mut currencies:Vec<Currency> = self.storage_system.load_currencies();
        currencies.sort_by(|a, b| a.id.cmp(&b.id));
        currencies
    }

    pub fn get_crypto_currencies(&self) -> Vec<CryptoCurrency> {
        let mut crypto_currencies:Vec<CryptoCurrency> = self.storage_system.load_crypto_currencies();
        crypto_currencies.sort_by(|a, b| a.id.cmp(&b.id));
        crypto_currencies
    }

}