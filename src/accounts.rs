use std::sync::Arc;
use std::time::{SystemTime};
use bincode::{Decode, Encode};
use tracing::{Level, span};
use crate::assets::AssetSystem;
use crate::storage::StorageSystem;

#[derive(Encode, Decode,  Debug)]
pub struct Account {
    pub id: u64,
    pub name: String,
    pub timestamp: SystemTime,
}


#[derive(Encode, Decode, Debug)]
pub struct AccountCurrency {
    pub id: u64,
    pub account_id: u64,
    pub currency_id: u64,
    pub balance: f64,
}
#[derive(Encode, Decode, Debug)]
pub struct AccountCurrencyHistory {
    pub id: u64,
    pub account_id: u64,
    pub currency_id: u64,
    pub balance: f64,
    pub timestamp: SystemTime,
}

#[derive(Encode, Decode, Debug)]
pub struct AccountStock {
    pub id: u64,
    pub account_id: u64,
    pub stock_id: u64,
    pub quantity: i64,
}

#[derive(Encode, Decode, Debug)]
pub struct AccountStockHistory {
    pub id: u64,
    pub account_id: u64,
    pub stock_id: u64,
    pub quantity: i64,
    pub timestamp: SystemTime,
}


pub struct AccountSystem {
    pub account_last_id: u64,
    pub account_currencies_last_id: u64,
    pub account_currency_histories_last_id: u64,
    pub account_stocks_last_id: u64,
    pub account_stock_histories_last_id: u64,
    pub storage_system: Arc<StorageSystem>,
    pub asset_system: Arc<AssetSystem>,
}


impl AccountSystem {
    pub fn new(storage_system: Arc<StorageSystem>, asset_system: Arc<AssetSystem>) -> AccountSystem {
        let mut account_last_id = 0;
        match storage_system.get_last_account() {
            None => {}
            Some(account) => {
                account_last_id = account.id;
            }
        }
        let mut account_currencies_last_id = 0;
        match storage_system.get_last_account_currency() {
            None => {}
            Some(account_currency) => {
                account_currencies_last_id = account_currency.id;
            }
        }
        let mut account_currency_histories_last_id = 0;
        match storage_system.get_last_account_currency_history() {
            None => {}
            Some(account_currency_history) => {
                account_currency_histories_last_id = account_currency_history.id;
            }
        }
        let mut account_stocks_last_id = 0;
        match storage_system.get_last_account_stock() {
            None => {}
            Some(account_stock) => {
                account_stocks_last_id = account_stock.id;
            }
        }
        let mut account_stock_histories_last_id = 0;
        match storage_system.get_last_account_stock_history() {
            None => {}
            Some(account_stock_history) => {
                account_stock_histories_last_id = account_stock_history.id;
            }
        }

        AccountSystem {
            account_last_id,
            account_currencies_last_id,
            account_currency_histories_last_id,
            account_stocks_last_id,
            account_stock_histories_last_id,
            storage_system,
            asset_system,
        }


    }

    pub fn create_account(&mut self, mut account: Account) -> u64 {
        self.account_last_id += 1;
        account.id = self.account_last_id;

        let add_account = span!(Level::TRACE, "add_account");
        let _ = add_account.enter();
        self.storage_system.add_account(&account);
        drop(add_account);
        let currencies = self.asset_system.get_currencies();
        for currency in currencies {
            self.create_account_currency(self.account_last_id, currency.id);
        }
        self.account_last_id
    }



    pub fn create_account_currency(&mut self, account_id: u64, currency_id: u64) -> u64 {
        self.account_currencies_last_id += 1;
        let account_currency = AccountCurrency {
            id: self.account_currencies_last_id,
            account_id,
            currency_id,
            balance: 0.0,
        };
        self.storage_system.add_account_currency(&account_currency);
        self.account_currencies_last_id
    }


    pub fn add_currency_to_account(&mut self, account_id: u64, currency_id: u64, balance: f64) {
        let mut account_currency = self.storage_system.get_account_currency(account_id, currency_id).unwrap();
        account_currency.balance += balance;
        let balance = account_currency.balance;
        self.storage_system.update_account_currency(account_currency);
        self.add_account_currency_history(account_id, currency_id, balance);
    }

    pub fn add_account_currency_history(&mut self, account_id: u64, currency_id: u64, balance: f64) {
        self.account_currency_histories_last_id += 1;
        let account_currency_history = AccountCurrencyHistory {
            id: self.account_currency_histories_last_id,
            account_id,
            currency_id,
            balance,
            timestamp: SystemTime::now(),
        };
        self.storage_system.add_account_currency_history(&account_currency_history);
    }

    pub fn create_account_stock(&mut self, account_id: u64, stock_id: u64) -> u64 {
        self.account_stocks_last_id += 1;
        let account_stock = AccountStock {
            id: self.account_stocks_last_id,
            account_id,
            stock_id,
            quantity: 0,
        };
        self.storage_system.add_account_stock(&account_stock);
        self.account_stocks_last_id
    }
    pub fn add_stock_to_account(&mut self, account_id: u64, stock_id: u64, quantity: i64) {
        let account_stock_opt = self.storage_system.get_account_stock(account_id, stock_id);
        let mut account_stock  = match account_stock_opt {
            None => {
                let account_stock_id = self.create_account_stock(account_id, stock_id);
                self.storage_system.get_account_stock_by_id(account_stock_id).unwrap()
            }
            Some(account_stock) => {
                account_stock
            }
        };
        account_stock.quantity += quantity;
        let quantity = account_stock.quantity;
        self.storage_system.update_account_stock(account_stock);
        self.add_account_stock_history(account_id, stock_id, quantity);
    }

    pub fn add_account_stock_history(&mut self, account_id: u64, stock_id: u64, quantity: i64) {
        self.account_stock_histories_last_id += 1;
        let account_stock_history = AccountStockHistory {
            id: self.account_stock_histories_last_id,
            account_id,
            stock_id,
            quantity,
            timestamp: SystemTime::now(),
        };
        self.storage_system.add_account_stock_history(&account_stock_history);
    }

}