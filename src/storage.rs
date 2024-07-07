use bincode::{config, decode_from_slice};
use crate::accounts::{Account, AccountCurrency};
use crate::assets::{Currency, Stock};

use std::any::type_name;
use redb::{Database, Key, Range, ReadableTable, TableDefinition, TypeName, Value};
use std::cmp::Ordering;
use std::fmt::Debug;

pub struct StorageSystem {
    pub accounts_db: Database,
}

const DATABASE_FOLDER_NAME: &str = "database";
const CURRENCIES_FILE_NAME: &str = "currencies.bin";
const STOCKS_FILE_NAME: &str = "stocks.bin";
const ACCOUNT_FOLDER_PREFIX: &str = "account_";
const ACCOUNTS_DB_NAME: &str = "accounts.redb";
const ACCOUNTS_TABLE: TableDefinition<u64, Bincode<Account>> = TableDefinition::new("accounts");

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
    pub fn load_currencies(&self) -> Vec<Currency> {
        if !std::path::Path::new(&format!("{DATABASE_FOLDER_NAME}/{CURRENCIES_FILE_NAME}")).exists() {
            let currencies =  self.init_currencies();
            self.save_currencies(&currencies);
            return currencies;
        }
        let encoded = std::fs::read(&format!("{DATABASE_FOLDER_NAME}/{CURRENCIES_FILE_NAME}")).unwrap();
        let config = config::standard();
        let (decoded, _): (Vec<Currency>, usize) = decode_from_slice(&encoded, config).unwrap();
        decoded
    }

    pub fn save_currencies(&self, currencies: &Vec<Currency>) {
        let config = config::standard();
        let encoded: Vec<u8> = bincode::encode_to_vec(currencies, config).unwrap();
        std::fs::write(&format!("{DATABASE_FOLDER_NAME}/{CURRENCIES_FILE_NAME}"), encoded).unwrap();
    }

    fn init_currencies(&self) -> Vec<Currency> {
        let currencies = vec![
            Currency { id: 1, symbol: "USD".to_string() },
            Currency { id: 2, symbol: "EUR".to_string() },
        ];
        self.save_currencies(&currencies);
        currencies
    }

    pub fn load_stocks(&self) -> Vec<Stock> {
        if !std::path::Path::new(&format!("{DATABASE_FOLDER_NAME}/{STOCKS_FILE_NAME}")).exists() {
            let stocks =  self.init_stocks();
            self.save_stocks(&stocks);
            return stocks;
        }
        let encoded = std::fs::read(&format!("{DATABASE_FOLDER_NAME}/{STOCKS_FILE_NAME}")).unwrap();
        let config = config::standard();
        let (decoded, _): (Vec<Stock>, usize) = decode_from_slice(&encoded, config).unwrap();
        decoded
    }

    pub fn save_stocks(&self, stocks: &Vec<Stock>) {
        let config = config::standard();
        let encoded: Vec<u8> = bincode::encode_to_vec(stocks, config).unwrap();
        std::fs::write(&format!("{DATABASE_FOLDER_NAME}/{STOCKS_FILE_NAME}"), encoded).unwrap();
    }

    fn init_stocks(&self) -> Vec<Stock> {
        let stocks = vec![
            Stock { id: 1, symbol: "AAPL".to_string() },
            Stock { id: 2, symbol: "GOOGL".to_string() },
        ];
        self.save_stocks(&stocks);
        stocks
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

    pub fn save_accounts(&self, accounts: &Vec<Account>) {
        let write_txn = self.accounts_db.begin_write().unwrap();
        {
            let mut table = write_txn.open_table(ACCOUNTS_TABLE).unwrap();
            for account in accounts {
                table.insert(&account.id, account).unwrap();
            }
        }
        write_txn.commit().unwrap();
    }

    pub fn add_account(&self, account: &Account) {
        let write_txn = self.accounts_db.begin_write().unwrap();
        {
            let mut table = write_txn.open_table(ACCOUNTS_TABLE).unwrap();
            table.insert(&account.id, account).unwrap();
        }
        write_txn.commit().unwrap();
    }

    fn create_account_folder(&self, account_id: u64) {
        if !std::path::Path::new(&format!("{DATABASE_FOLDER_NAME}/{ACCOUNT_FOLDER_PREFIX}{account_id}")).exists() {
            std::fs::create_dir(&format!("{DATABASE_FOLDER_NAME}/{ACCOUNT_FOLDER_PREFIX}{account_id}")).unwrap();
        }
    }

    pub fn save_account_currencies(&self, account_id: u64, account_currencies: &Vec<AccountCurrency>) {
        let config = config::standard();
        let encoded: Vec<u8> = bincode::encode_to_vec(account_currencies, config).unwrap();
        self.create_account_folder(account_id);
        std::fs::write(&format!("{DATABASE_FOLDER_NAME}/{ACCOUNT_FOLDER_PREFIX}{account_id}/{CURRENCIES_FILE_NAME}"), encoded).unwrap();
    }

    pub fn load_account_currencies(&self, account_id: u64) -> Vec<AccountCurrency> {
        if !std::path::Path::new(&format!("{DATABASE_FOLDER_NAME}/{ACCOUNT_FOLDER_PREFIX}{account_id}/{CURRENCIES_FILE_NAME}")).exists() {
            return vec![];
        }
        let encoded = std::fs::read(&format!("{DATABASE_FOLDER_NAME}/{ACCOUNT_FOLDER_PREFIX}{account_id}/{CURRENCIES_FILE_NAME}")).unwrap();
        let config = config::standard();
        let (decoded, _): (Vec<AccountCurrency>, usize) = decode_from_slice(&encoded, config).unwrap();
        decoded
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