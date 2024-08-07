use std::sync::Arc;
use std::time::{SystemTime};
use chrono::{DateTime, Local};
use tracing_subscriber::fmt::format::FmtSpan;
use kubera::accounts::{Account, AccountSystem};
use kubera::orders::{ExecutionType, Order, OrderStatus, OrderSystem, PriceType, TradeType};
use kubera::assets::{AssetSystem, Currency, CryptoCurrency};
use kubera::matcher::{MatcherSystem};
use kubera::storage::StorageSystem;
fn main() {

    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(false)
        .with_span_events(FmtSpan::CLOSE)
        .with_max_level(tracing::Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber).unwrap();

    let _ = std::fs::remove_dir_all("database");
    let storage_system = Arc::new(StorageSystem::new());
    let mut assets_system =  AssetSystem::new(storage_system.clone());
    if assets_system.get_currencies().len() == 0 {
        let _ = assets_system.create_currency(Currency { id: 0, symbol: "USD".to_string() });
    }
    if assets_system.get_crypto_currencies().len() == 0 {
        let _ = assets_system.create_crypto_currency(CryptoCurrency { id: 0, symbol: "BTC".to_string() });
    }

    let assets_system = Arc::new(assets_system);
    let mut accounts_system = AccountSystem::new(storage_system.clone(), assets_system.clone());

    let currency_id = assets_system.get_currencies()[0].id;
    let crypto_currency_id = assets_system.get_crypto_currencies()[0].id;


    if storage_system.load_accounts().len() == 0 {
        let account1_id = accounts_system.create_account(Account { id: 0, name: "Alice".to_string(), timestamp: SystemTime::now() });
        let account2_id = accounts_system.create_account(Account { id: 0, name: "Bob".to_string(), timestamp: SystemTime::now() });
        accounts_system.add_currency_to_account(account1_id, currency_id, 100000.0);
        accounts_system.add_crypto_currency_to_account(account2_id, crypto_currency_id, 1.0);
    }
    let accounts = storage_system.load_accounts();
    let account1_id = accounts[0].id;
    let account2_id = accounts[1].id;

    let mut order_system = OrderSystem::new(storage_system.clone(), assets_system.clone());

    let core_ids = core_affinity::get_core_ids().unwrap();
    let core_id = core_ids[0];

    let matcher_system = MatcherSystem::start(crypto_currency_id, currency_id, core_id);
    let order1 = order_system.create_order(Order { id: 0, account_id: account1_id, trade_type: TradeType::Buy, price_type: PriceType::Market, execution_type: ExecutionType::Full, crypto_currency_id: crypto_currency_id, currency_id, quantity: 0.5,  status: OrderStatus::Open, timestamp: SystemTime::now()});
    let order2 = order_system.create_order(Order { id: 0, account_id: account2_id, trade_type: TradeType::Sell, price_type: PriceType::Limit(50000.00), execution_type: ExecutionType::Partial, crypto_currency_id: crypto_currency_id, currency_id, quantity: 1.0,  status: OrderStatus::Open, timestamp: SystemTime::now()});
    let _ = matcher_system.add_order(order1);
    let _ = matcher_system.add_order(order2);
    print_accounts(storage_system.clone());
    loop {
        while let Some(order_match) = matcher_system.get_order_match() {
           tracing::info!("OrderMatch: Buy Order Id: {} Sell Order Id: {} Quantity: {} Price: {}", order_match.buy_order_id, order_match.sell_order_id, order_match.quantity, order_match.price);
           order_system.create_order_history(&order_match, &mut accounts_system);
           print_accounts(storage_system.clone());
           if storage_system.get_account_currency(account1_id, currency_id).unwrap().balance > 0.0 {
               let order = order_system.create_order(Order { id: 0, account_id: account1_id, trade_type: TradeType::Buy, price_type: PriceType::Market, execution_type: ExecutionType::Full, crypto_currency_id: crypto_currency_id, currency_id, quantity: 0.5,  status: OrderStatus::Open, timestamp: SystemTime::now()});
               matcher_system.add_order(order);
           }
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

}

fn print_accounts(storage_system: Arc<StorageSystem>) {
    for account in storage_system.load_accounts() {

        let datetime: DateTime<Local> = account.timestamp.into();
        tracing::info! {
            "AccountId: {} Name: {} Timestamp: {}",account.id,  account.name, datetime.format("%Y-%m-%d %H:%M:%S").to_string()
        };

        for account_currency in storage_system.get_account_currency_by_account_id(account.id) {
            tracing::info! {
                "CurrencyId: {} Symbol: {} Balance: {:.2}", account_currency.id, storage_system.get_currency(account_currency.currency_id).unwrap().symbol, account_currency.balance
            };

            for account_currency_history in storage_system.get_currency_history_by_account_id_account_currency_id(account.id, account_currency.id) {
                let datetime: DateTime<Local> = account_currency_history.timestamp.into();
                tracing::info! {
                    "CurrencyHistoryId: {} Balance: {:.2} Timestamp: {}", account_currency_history.id,  account_currency_history.balance, datetime.format("%Y-%m-%d %H:%M:%S").to_string()
                };
            }
        }
        for account_crypto_currency in storage_system.get_account_crypto_currencies_by_account_id(account.id) {
            tracing::info! {
                "CryptoCurrencyId: {} {} Amount: {}", account_crypto_currency.id, storage_system.get_crypto_currency(account_crypto_currency.crypto_currency_id).unwrap().symbol, account_crypto_currency.quantity
            };

            for account_crypto_currency_history in storage_system.get_crypto_currency_history_by_account_id_crypto_currency_id(account.id, account_crypto_currency.crypto_currency_id) {
                let datetime: DateTime<Local> = account_crypto_currency_history.timestamp.into();
                tracing::info! {
                    "CryptoCurrencyHistoryId: {} Quantity: {} Timestamp: {}", account_crypto_currency_history.id,  account_crypto_currency_history.quantity, datetime.format("%Y-%m-%d %H:%M:%S").to_string()
                };
            }
        }
    }

}
