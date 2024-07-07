use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime};
use bincode::{Decode, Encode};
use crate::assets::AssetSystem;
use crate::storage::StorageSystem;

#[derive(Encode, Decode,  Debug)]
pub struct Account {
    pub id: u64,
    pub name: String,
    pub timestamp: SystemTime,
}


#[derive(Encode, Decode)]
pub struct AccountCurrency {
    pub id: u64,
    pub account_id: u64,
    pub currency_id: u64,
    pub balance: f64,
}
#[derive(Encode, Decode)]
pub struct AccountCurrencyHistory {
    pub id: u64,
    pub account_id: u64,
    pub currency_id: u64,
    pub balance: f64,
    pub timestamp: SystemTime,
}

#[derive(Encode, Decode)]
pub struct AccountStock {
    pub id: u64,
    pub account_id: u64,
    pub stock_id: u64,
    pub quantity: i64,
}

#[derive(Encode, Decode)]
pub struct AccountStockHistory {
    pub account_id: u64,
    pub stock_id: u64,
    pub quantity: i64,
    pub timestamp: SystemTime,
}


pub struct AccountSystem {
    pub accounts_hash_map: HashMap<u64,u64>, // account.id -> index in accounts
    pub accounts: Vec<Account>,
    pub account_currencies: HashMap<u64,Vec<AccountCurrency>>,
    pub account_currency_histories: HashMap<u64,Vec<AccountCurrencyHistory>>,
    pub account_stocks: HashMap<u64,Vec<AccountStock>>,
    pub account_stock_histories: HashMap<u64,Vec<AccountStockHistory>>,
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
        let mut account_currencies_last_id = 0;
        let account_currency_histories_last_id = 0;
        let account_stocks_last_id = 0;
        let account_stock_histories_last_id = 0;

        let mut accounts_hash_map = HashMap::new();
        let accounts = storage_system.load_accounts();
        let mut i = 0;
        for account in &accounts {
            if account.id > account_last_id {
                account_last_id = account.id;
            }
            accounts_hash_map.insert(account.id, i);
            i += 1;
        }

        let mut account_currencies = HashMap::new();

        for account in &accounts {
            let account_currencies_vec = storage_system.load_account_currencies(account.id);
            for account_currency in &account_currencies_vec {
                if account_currency.id > account_currencies_last_id {
                    account_currencies_last_id = account_currency.id;
                }
            }
            account_currencies.insert(account.id, account_currencies_vec);
        }

        let account_currency_histories = HashMap::new();
        let account_stocks = HashMap::new();
        let account_stock_histories = HashMap::new();

        AccountSystem {
            accounts_hash_map,
            accounts,
            account_currencies,
            account_currency_histories,
            account_stocks,
            account_stock_histories,
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
        self.accounts.push(account);
        self.accounts_hash_map.insert(self.account_last_id, self.accounts.len() as u64 - 1);
        self.storage_system.save_accounts(&self.accounts);

        // self.asset_system.currencies.iter().for_each(|(currency_id, _)| {
        //     self.create_account_currency(self.account_last_id, *currency_id);
        // });

        let currencies: Vec<u64> = self.asset_system.currencies.iter()
            .map(|(currency_id, _)| *currency_id)
            .collect();

        for currency_id in currencies {
            self.create_account_currency(self.account_last_id, currency_id);
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
        if !self.account_currencies.contains_key(&account_id) {
            self.account_currencies.insert(account_id, vec![]);
        }
        self.account_currencies.get_mut(&account_id).unwrap().push(account_currency);
        self.storage_system.save_account_currencies( account_id, self.account_currencies.get(&account_id).unwrap());
        self.account_currencies_last_id
    }


    pub fn add_currency_to_account(&mut self, account_id: u64, currency_id: u64, balance: f64) {
        {
            let account_currency = self.account_currencies.get_mut(&account_id).unwrap().iter_mut().find(|x| x.currency_id == currency_id).unwrap();
            account_currency.balance += balance;
            let balance = account_currency.balance;
            self.add_account_currency_history(account_id, currency_id, balance);
        }
        self.storage_system.save_account_currencies(account_id, &self.account_currencies.get(&account_id).unwrap());
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
        self.account_currency_histories.entry(account_id).or_insert(vec![]).push(account_currency_history);
    }

    pub fn add_stock_to_account(&mut self, account_id: u64, stock_id: u64, quantity: i64) {
        self.account_stocks.entry(account_id).or_insert(vec![]);
        match self.account_stocks.get_mut(&account_id).unwrap().iter_mut().find(|x| x.stock_id == stock_id) {
            None => {
                self.account_stocks_last_id += 1;
                let account_stock = AccountStock {
                    id: self.account_stocks_last_id,
                    account_id,
                    stock_id,
                    quantity,
                };
                self.account_stocks.get_mut(&account_id).unwrap().push(account_stock);
                self.add_account_stock_history(account_id, stock_id, quantity);
            }
            Some(account_stock) => {
                account_stock.quantity += quantity;
                let quantity = account_stock.quantity;
                self.add_account_stock_history(account_id, stock_id, quantity);
            }
        }
    }

    pub fn add_account_stock_history(&mut self, account_id: u64, stock_id: u64, quantity: i64) {
        self.account_stock_histories_last_id += 1;
        let account_stock_history = AccountStockHistory {
            account_id,
            stock_id,
            quantity,
            timestamp: SystemTime::now(),
        };
        self.account_stock_histories.entry(account_id).or_insert(vec![]).push(account_stock_history);
    }

}