use bincode::{config, decode_from_slice};
use crate::accounts::{Account, AccountCurrency, AccountCurrencyHistory, AccountCryptoCurrency, AccountCryptoCurrencyHistory};
use crate::assets::{Currency, CryptoCurrency};

use std::any::type_name;
use redb::{Database, Key, Range, ReadableTable, TableDefinition, TypeName, Value};
use std::cmp::Ordering;
use std::fmt::Debug;
use crate::orders::{Order, OrderHistory};

pub struct StorageSystem {
    pub accounts_db: Database,
}

const DATABASE_FOLDER_NAME: &str = "database";
const ACCOUNTS_DB_NAME: &str = "accounts.redb";
const ACCOUNTS_TABLE: TableDefinition<u64, Bincode<Account>> = TableDefinition::new("accounts");
const CURRENCIES_TABLE: TableDefinition<u64, Bincode<Currency>> = TableDefinition::new("currencies");
const CRYPTO_CURRENCIES_TABLE: TableDefinition<u64, Bincode<CryptoCurrency>> = TableDefinition::new("crypto_currencies");
const ACCOUNT_CURRENCIES_TABLE: TableDefinition<u64, Bincode<AccountCurrency>> = TableDefinition::new("account_currencies");
const ACCOUNT_CRYPTO_CURRENCIES_TABLE: TableDefinition<u64, Bincode<AccountCryptoCurrency>> = TableDefinition::new("account_crypto_currencies");
const ACCOUNT_CURRENCY_HISTORIES_TABLE: TableDefinition<u64, Bincode<AccountCurrencyHistory>> = TableDefinition::new("account_currency_histories");
const ACCOUNT_CRYPTO_CURRENCY_HISTORIES_TABLE: TableDefinition<u64, Bincode<AccountCryptoCurrencyHistory>> = TableDefinition::new("account_crypto_currencies_histories");
const ORDERS_TABLE: TableDefinition<u64, Bincode<Order>> = TableDefinition::new("orders");
const ORDER_HISTORIES_TABLE: TableDefinition<u64, Bincode<OrderHistory>> = TableDefinition::new("order_histories");


impl StorageSystem {
    pub fn new() -> StorageSystem {
        if !std::path::Path::new(DATABASE_FOLDER_NAME).exists() {
            std::fs::create_dir(DATABASE_FOLDER_NAME).unwrap();
        }

        let db = Database::create(format!("{DATABASE_FOLDER_NAME}/{ACCOUNTS_DB_NAME}")).unwrap();

        StorageSystem {
            accounts_db: db,
        }
    }

    pub fn load_accounts(&self) -> Vec<Account> {

        let read_txn = self.accounts_db.begin_read().unwrap();
        let table_opt = read_txn.open_table(ACCOUNTS_TABLE);
        match table_opt {
            Ok(table) => {
                let iter: Range<u64, Bincode<Account>> = table.iter().unwrap();
                let mut accounts = vec![];
                for acc in iter {
                    accounts.push(acc.unwrap().1.value());
                }
                accounts
            }
            Err(_) => {
                vec![]
            }
        }

    }

    pub fn get_last_account(&self) -> Option<Account> {
        let read_txn = self.accounts_db.begin_read().unwrap();
        let table_opt = read_txn.open_table(ACCOUNTS_TABLE);
        match table_opt {
            Ok(table) => {
                let last_opt = table.last().unwrap();
                match last_opt {
                    None => {
                        None
                    },
                    Some(i) => {
                        Some(i.1.value())
                    }
                }
            }
            Err(_) => {
                None
            }
        }
    }


    pub fn add_account(&self, account: &Account) {
        let write_txn = self.accounts_db.begin_write().unwrap();
        {
            let mut table = write_txn.open_table(ACCOUNTS_TABLE).unwrap();
            table.insert(&account.id, account).unwrap();
        }
        write_txn.commit().unwrap();
    }

    pub fn get_last_currency(&self) -> Option<Currency> {
        let read_txn = self.accounts_db.begin_read().unwrap();
        let table_opt = read_txn.open_table(CURRENCIES_TABLE);
        match table_opt {
            Ok(table) => {
                let last_opt = table.last().unwrap();
                match last_opt {
                    None => {
                        None
                    },
                    Some(i) => {
                        Some(i.1.value())
                    }
                }
            }
            Err(_) => {
                None
            }
        }
    }

    pub fn get_last_crypto_currency(&self) -> Option<CryptoCurrency> {
        let read_txn = self.accounts_db.begin_read().unwrap();
        let table_opt = read_txn.open_table(CRYPTO_CURRENCIES_TABLE);
        match table_opt {
            Ok(table) => {
                let last_opt = table.last().unwrap();
                match last_opt {
                    None => {
                        None
                    },
                    Some(i) => {
                        Some(i.1.value())
                    }
                }
            }
            Err(_) => {
                None
            }
        }
    }

    pub fn add_currency(&self, currency: &Currency) {
        let write_txn = self.accounts_db.begin_write().unwrap();
        {
            let mut table = write_txn.open_table(CURRENCIES_TABLE).unwrap();
            table.insert(&currency.id, currency).unwrap();
        }
        write_txn.commit().unwrap();
    }


    pub fn add_crypto_currency(&self, crypto_currency: &CryptoCurrency) {
        let write_txn = self.accounts_db.begin_write().unwrap();
        {
            let mut table = write_txn.open_table(CRYPTO_CURRENCIES_TABLE).unwrap();
            table.insert(&crypto_currency.id, crypto_currency).unwrap();
        }
        write_txn.commit().unwrap();
    }

    pub fn get_currency(&self, currency_id: u64) -> Option<Currency> {
        let read_txn = self.accounts_db.begin_read().unwrap();
        let table_opt = read_txn.open_table(CURRENCIES_TABLE);
        match table_opt {
            Ok(table) => {
                let currency_opt = table.get(&currency_id).unwrap();
                match currency_opt {
                    Some(currency) => {
                        Some(currency.value())
                    }
                    None => {
                        None
                    }
                }
            }
            Err(_) => {
                None
            }
        }
    }

    pub fn get_crypto_currency(&self, crypto_currency_id: u64) -> Option<CryptoCurrency> {
        let read_txn = self.accounts_db.begin_read().unwrap();
        let table_opt = read_txn.open_table(CRYPTO_CURRENCIES_TABLE);
        match table_opt {
            Ok(table) => {
                let crypto_currency_opt = table.get(&crypto_currency_id).unwrap();
                match crypto_currency_opt {
                    Some(crypto_currency) => {
                        Some(crypto_currency.value())
                    }
                    None => {
                        None
                    }
                }
            }
            Err(_) => {
                None
            }
        }
    }

    pub fn load_currencies(&self) -> Vec<Currency> {
        let read_txn = self.accounts_db.begin_read().unwrap();
        let table_opt = read_txn.open_table(CURRENCIES_TABLE);
        match table_opt {
            Ok(table) => {
                let iter: Range<u64, Bincode<Currency>> = table.iter().unwrap();
                let mut currencies = vec![];
                for acc in iter {
                    currencies.push(acc.unwrap().1.value());
                }
                currencies
            }
            Err(_) => {
                vec![]
            }
        }
    }

    pub fn load_crypto_currencies(&self) -> Vec<CryptoCurrency> {
        let read_txn = self.accounts_db.begin_read().unwrap();
        let table_opt = read_txn.open_table(CRYPTO_CURRENCIES_TABLE);
        match table_opt {
            Ok(table) => {
                let iter: Range<u64, Bincode<CryptoCurrency>> = table.iter().unwrap();
                let mut crypto_currencies = vec![];
                for acc in iter {
                    crypto_currencies.push(acc.unwrap().1.value());
                }
                crypto_currencies
            }
            Err(_) => {
                vec![]
            }
        }
    }

    pub fn get_last_account_currency(&self) -> Option<AccountCurrency> {
        let read_txn = self.accounts_db.begin_read().unwrap();
        let table_opt = read_txn.open_table(ACCOUNT_CURRENCIES_TABLE);
        match table_opt {
            Ok(table) => {
                let last_opt = table.last().unwrap();
                match last_opt {
                    None => {
                        None
                    },
                    Some(i) => {
                        Some(i.1.value())
                    }
                }
            }
            Err(_) => {
                None
            }
        }
    }

    pub fn get_last_account_currency_history(&self) -> Option<AccountCurrencyHistory> {
        let read_txn = self.accounts_db.begin_read().unwrap();
        let table_opt = read_txn.open_table(ACCOUNT_CURRENCY_HISTORIES_TABLE);
        match table_opt {
            Ok(table) => {
                let last_opt = table.last().unwrap();
                match last_opt {
                    None => {
                        None
                    },
                    Some(i) => {
                        Some(i.1.value())
                    }
                }
            }
            Err(_) => {
                None
            }
        }
    }

    pub fn get_last_account_crypto_currency(&self) -> Option<AccountCryptoCurrency> {
        let read_txn = self.accounts_db.begin_read().unwrap();
        let table_opt = read_txn.open_table(ACCOUNT_CRYPTO_CURRENCIES_TABLE);
        match table_opt {
            Ok(table) => {
                let last_opt = table.last().unwrap();
                match last_opt {
                    None => {
                        None
                    },
                    Some(i) => {
                        Some(i.1.value())
                    }
                }
            }
            Err(_) => {
                None
            }
        }
    }

    pub fn get_last_account_crypto_currency_history(&self) -> Option<AccountCryptoCurrencyHistory> {
        let read_txn = self.accounts_db.begin_read().unwrap();
        let table_opt = read_txn.open_table(ACCOUNT_CRYPTO_CURRENCY_HISTORIES_TABLE);
        match table_opt {
            Ok(table) => {
                let last_opt = table.last().unwrap();
                match last_opt {
                    None => {
                        None
                    },
                    Some(i) => {
                        Some(i.1.value())
                    }
                }
            }
            Err(_) => {
                None
            }
        }
    }

    pub fn add_account_currency(&self, account_currency: &AccountCurrency) {
        let write_txn = self.accounts_db.begin_write().unwrap();
        {
            let mut table = write_txn.open_table(ACCOUNT_CURRENCIES_TABLE).unwrap();
            table.insert(&account_currency.id, account_currency).unwrap();
        }
        write_txn.commit().unwrap();
    }

    pub fn load_account_currencies(&self) -> Vec<AccountCurrency> {
        let read_txn = self.accounts_db.begin_read().unwrap();
        let table_opt = read_txn.open_table(ACCOUNT_CURRENCIES_TABLE);
        match table_opt {
            Ok(table) => {
                let iter: Range<u64, Bincode<AccountCurrency>> = table.iter().unwrap();
                let mut account_currencies = vec![];
                for acc in iter {
                    account_currencies.push(acc.unwrap().1.value());
                }
                account_currencies
            }
            Err(_) => {
                vec![]
            }
        }
    }
    pub fn get_account_currency(&self, account_id: u64, currency_id: u64) -> Option<AccountCurrency> {
        let account_currencies:Vec<AccountCurrency> = self.load_account_currencies();
        for acc in account_currencies {
            if acc.account_id == account_id && acc.currency_id == currency_id {
                return Some(acc);
            }
        }
        None
    }

    pub fn get_account_currency_by_account_id(&self, account_id: u64) -> Vec<AccountCurrency> {
        let account_currencies:Vec<AccountCurrency> = self.load_account_currencies();
        let mut account_currencies_by_account_id = vec![];
        for acc in account_currencies {
            if acc.account_id == account_id {
                account_currencies_by_account_id.push(acc);
            }
        }
        account_currencies_by_account_id
    }

    pub fn add_account_currency_history(&self, account_currency_history: &AccountCurrencyHistory) {
        let write_txn = self.accounts_db.begin_write().unwrap();
        {
            let mut table = write_txn.open_table(ACCOUNT_CURRENCY_HISTORIES_TABLE).unwrap();
            table.insert(&account_currency_history.id, account_currency_history).unwrap();
        }
        write_txn.commit().unwrap();
    }

    pub fn update_account_currency(&self, account_currency: &AccountCurrency) {
        let write_txn = self.accounts_db.begin_write().unwrap();
        {
            let mut table = write_txn.open_table(ACCOUNT_CURRENCIES_TABLE).unwrap();
            table.insert(&account_currency.id, account_currency).unwrap();
        }
        write_txn.commit().unwrap();
    }

    pub fn add_account_crypto_currency(&self, account_crypto_currency: &AccountCryptoCurrency) {
        let write_txn = self.accounts_db.begin_write().unwrap();
        {
            let mut table = write_txn.open_table(ACCOUNT_CRYPTO_CURRENCIES_TABLE).unwrap();
            table.insert(&account_crypto_currency.id, account_crypto_currency).unwrap();
        }
        write_txn.commit().unwrap();
    }

    pub fn load_account_crypto_currencies(&self) -> Vec<AccountCryptoCurrency> {
        let read_txn = self.accounts_db.begin_read().unwrap();
        let table_opt = read_txn.open_table(ACCOUNT_CRYPTO_CURRENCIES_TABLE);
        match table_opt {
            Ok(table) => {
                let iter: Range<u64, Bincode<AccountCryptoCurrency>> = table.iter().unwrap();
                let mut account_crypto_currencies = vec![];
                for acc in iter {
                    account_crypto_currencies.push(acc.unwrap().1.value());
                }
                account_crypto_currencies
            }
            Err(_) => {
                vec![]
            }
        }
    }

    pub fn get_account_crypto_currency(&self, account_id: u64, crypto_currency_id: u64) -> Option<AccountCryptoCurrency> {
        let account_crypto_currencies:Vec<AccountCryptoCurrency> = self.load_account_crypto_currencies();
        for acc in account_crypto_currencies {
            if acc.account_id == account_id && acc.crypto_currency_id == crypto_currency_id {
                return Some(acc);
            }
        }
        None
    }

    pub fn get_account_crypto_currencies_by_account_id(&self, account_id: u64) -> Vec<AccountCryptoCurrency> {
        let account_crypto_currencies:Vec<AccountCryptoCurrency> = self.load_account_crypto_currencies();
        let mut account_crypto_currencies_by_account_id = vec![];
        for acc in account_crypto_currencies {
            if acc.account_id == account_id {
                account_crypto_currencies_by_account_id.push(acc);
            }
        }
        account_crypto_currencies_by_account_id
    }

    pub fn get_account_crypto_currency_by_id(&self, account_crypto_currency_id: u64) -> Option<AccountCryptoCurrency> {
        let read_txn = self.accounts_db.begin_read().unwrap();
        let table_opt = read_txn.open_table(ACCOUNT_CRYPTO_CURRENCIES_TABLE);
        match table_opt {
            Ok(table) => {
                let account_crypto_currency_opt = table.get(&account_crypto_currency_id).unwrap();
                match account_crypto_currency_opt {
                    Some(account_crypto_currency) => {
                        Some(account_crypto_currency.value())
                    }
                    None => {
                        None
                    }
                }
            }
            Err(_) => {
                None
            }
        }
    }

    pub fn update_account_crypto_currency(&self, account_crypto_currency: AccountCryptoCurrency) {
        self.add_account_crypto_currency(&account_crypto_currency);
    }

    pub fn add_account_crypto_currency_history(&self, account_crypto_currency_history: &AccountCryptoCurrencyHistory) {
        let write_txn = self.accounts_db.begin_write().unwrap();
        {
            let mut table = write_txn.open_table(ACCOUNT_CRYPTO_CURRENCY_HISTORIES_TABLE).unwrap();
            table.insert(&account_crypto_currency_history.id, account_crypto_currency_history).unwrap();
        }
        write_txn.commit().unwrap();
    }

    pub fn get_last_order(&self) -> Option<Order> {
        let read_txn = self.accounts_db.begin_read().unwrap();
        let table_opt = read_txn.open_table(ORDERS_TABLE);
        match table_opt {
            Ok(table) => {
                let last_opt = table.last().unwrap();
                match last_opt {
                    None => {
                        None
                    },
                    Some(i) => {
                        Some(i.1.value())
                    }
                }
            }
            Err(_) => {
                None
            }
        }
    }

    pub fn get_last_order_history(&self) -> Option<OrderHistory> {
        let read_txn = self.accounts_db.begin_read().unwrap();
        let table_opt = read_txn.open_table(ORDER_HISTORIES_TABLE);
        match table_opt {
            Ok(table) => {
                let last_opt = table.last().unwrap();
                match last_opt {
                    None => {
                        None
                    },
                    Some(i) => {
                        Some(i.1.value())
                    }
                }
            }
            Err(_) => {
                None
            }
        }
    }

    pub fn add_order(&self, order: &Order) {
        let write_txn = self.accounts_db.begin_write().unwrap();
        {
            let mut table = write_txn.open_table(ORDERS_TABLE).unwrap();
            table.insert(&order.id, order).unwrap();
        }
        write_txn.commit().unwrap();
    }

    pub fn add_order_history(&self, order_history: &OrderHistory) {
        let write_txn = self.accounts_db.begin_write().unwrap();
        {
            let mut table = write_txn.open_table(ORDER_HISTORIES_TABLE).unwrap();
            table.insert(&order_history.id, order_history).unwrap();
        }
        write_txn.commit().unwrap();
    }

    pub fn get_order(&self, order_id: u64) -> Option<Order> {
        let read_txn = self.accounts_db.begin_read().unwrap();
        let table_opt = read_txn.open_table(ORDERS_TABLE);
        match table_opt {
            Ok(table) => {
                let order_opt = table.get(&order_id).unwrap();
                match order_opt {
                    Some(order) => {
                        Some(order.value())
                    }
                    None => {
                        None
                    }
                }
            }
            Err(_) => {
                None
            }
        }
    }
    pub fn get_order_histories_by_order_id(&self, order_id: u64) -> Vec<OrderHistory> {
        let order_histories:Vec<OrderHistory> = self.load_order_histories();
        let mut order_histories_by_order_id = vec![];
        for order_history in order_histories {
            if order_history.order_id == order_id {
                order_histories_by_order_id.push(order_history);
            }
        }
        order_histories_by_order_id
    }

    pub fn load_order_histories(&self) -> Vec<OrderHistory> {
        let read_txn = self.accounts_db.begin_read().unwrap();
        let table_opt = read_txn.open_table(ORDER_HISTORIES_TABLE);
        match table_opt {
            Ok(table) => {
                let iter: Range<u64, Bincode<OrderHistory>> = table.iter().unwrap();
                let mut order_histories = vec![];
                for acc in iter {
                    order_histories.push(acc.unwrap().1.value());
                }
                order_histories
            }
            Err(_) => {
                vec![]
            }
        }
    }

    pub fn load_account_currency_histories(&self) -> Vec<AccountCurrencyHistory> {
        let read_txn = self.accounts_db.begin_read().unwrap();
        let table_opt = read_txn.open_table(ACCOUNT_CURRENCY_HISTORIES_TABLE);
        match table_opt {
            Ok(table) => {
                let iter: Range<u64, Bincode<AccountCurrencyHistory>> = table.iter().unwrap();
                let mut account_currency_histories = vec![];
                for acc in iter {
                    account_currency_histories.push(acc.unwrap().1.value());
                }
                account_currency_histories
            }
            Err(_) => {
                vec![]
            }
        }
    }

    pub fn get_currency_history_by_account_id_account_currency_id(&self, account_id: u64, account_currency_id: u64) -> Vec<AccountCurrencyHistory> {
        let account_currency_histories:Vec<AccountCurrencyHistory> = self.load_account_currency_histories();
        let mut account_currency_histories_by_account_id = vec![];
        for acc in account_currency_histories {
            if acc.account_id == account_id && acc.account_currency_id == account_currency_id {
                account_currency_histories_by_account_id.push(acc);
            }
        }
        account_currency_histories_by_account_id
    }


    pub fn load_account_crypto_currency_histories(&self) -> Vec<AccountCryptoCurrencyHistory> {
        let read_txn = self.accounts_db.begin_read().unwrap();
        let table_opt = read_txn.open_table(ACCOUNT_CRYPTO_CURRENCY_HISTORIES_TABLE);
        match table_opt {
            Ok(table) => {
                let iter: Range<u64, Bincode<AccountCryptoCurrencyHistory>> = table.iter().unwrap();
                let mut account_crypto_currency_histories = vec![];
                for acc in iter {
                    account_crypto_currency_histories.push(acc.unwrap().1.value());
                }
                account_crypto_currency_histories
            }
            Err(_) => {
                vec![]
            }
        }
    }
    pub fn get_crypto_currency_history_by_account_id_crypto_currency_id(&self, account_id: u64, stock_id: u64) -> Vec<AccountCryptoCurrencyHistory> {
        let account_crypto_currency_histories:Vec<AccountCryptoCurrencyHistory> = self.load_account_crypto_currency_histories();
        let mut account_crypto_currency_histories_by_account_id = vec![];
        for acc in account_crypto_currency_histories {
            if acc.account_id == account_id && acc.crypto_currency_id == stock_id {
                account_crypto_currency_histories_by_account_id.push(acc);
            }
        }
        account_crypto_currency_histories_by_account_id
    }

}




#[derive(Debug)]
pub struct Bincode<T>(pub T);

impl<T> Value for Bincode<T>
    where
        T: Debug + bincode::Decode + bincode::Encode,
{
    type SelfType<'a> = T
        where
            Self: 'a;

    type AsBytes<'a> = Vec<u8>
        where
            Self: 'a;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
        where
            Self: 'a,
    {
        let config = config::standard();
        let (decoded, _) = decode_from_slice(data, config).unwrap();
        decoded
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
        where
            Self: 'a,
            Self: 'b,
    {
        let config = config::standard();
        let encoded: Vec<u8> = bincode::encode_to_vec(value, config).unwrap();
        encoded
    }

    fn type_name() -> TypeName {
        TypeName::new(&format!("Bincode<{}>", type_name::<T>()))
    }
}

impl<T> Key for Bincode<T>
    where
        T: Debug +  Ord + bincode::Decode + bincode::Encode,
{
    fn compare(data1: &[u8], data2: &[u8]) -> Ordering {
        Self::from_bytes(data1).cmp(&Self::from_bytes(data2))
    }
}