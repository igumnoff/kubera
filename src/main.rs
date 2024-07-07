use std::sync::Arc;
use std::time::{SystemTime};
use chrono::{DateTime, Local};
use kubera::accounts::{Account, AccountSystem};
use kubera::orders::{ExecutionType, Order, OrderStatus, OrderSystem, PriceType, TradeType};
use kubera::assets::AssetSystem;
use kubera::matcher::MatcherSystem;
use kubera::storage::StorageSystem;

fn main() {
    let storage_system = Arc::new(StorageSystem::new());
    let assets_system =  Arc::new(AssetSystem::new(&storage_system));
    let mut accounts_system = AccountSystem::new(storage_system.clone(), assets_system.clone());

    if accounts_system.accounts_hash_map.len() == 0 {
        accounts_system.create_account(Account { id: 0, name: "Alice".to_string(), timestamp: SystemTime::now()});
        accounts_system.create_account(Account { id: 0, name: "Bob".to_string(), timestamp: SystemTime::now()});
    } else {
        println!("Accounts already exist");
    }
    let account1_id = accounts_system.accounts[0].id;
    let account2_id = accounts_system.accounts[1].id;
    let mut currencies = assets_system.currencies.keys().collect::<Vec<&u64>>();
    currencies.sort();
    let currency_id = *currencies[0];
    let stock_id = *assets_system.stocks.keys().next().unwrap();
    accounts_system.add_currency_to_account(account1_id, currency_id, 1000.0);
    accounts_system.add_stock_to_account(account2_id, stock_id, 50);

    print_accounts(&accounts_system, &assets_system);

    let mut order_system = OrderSystem::new(storage_system.clone(), assets_system.clone());
    let order1 = order_system.create_order(Order { id: 0, account_id: account1_id, trade_type: TradeType::Buy, price_type: PriceType::Market, execution_type: ExecutionType::Full, stock_id, currency_id, quantity: 10,  status: OrderStatus::Open, timestamp: SystemTime::now()});
    let order2 = order_system.create_order(Order { id: 0, account_id: account2_id, trade_type: TradeType::Sell, price_type: PriceType::Limit(100.00), execution_type: ExecutionType::Partial, stock_id, currency_id, quantity: 50,  status: OrderStatus::Open, timestamp: SystemTime::now()});
    let mut matcher_system = MatcherSystem::new(stock_id, currency_id);
    matcher_system.add_order(order1);
    matcher_system.add_order(order2);

    let thread_handle = std::thread::spawn(move || {
        loop {

            let order_matches = matcher_system.match_orders(&mut accounts_system);

            for order_match in &order_matches {
                println!("Buy Order Id: {} Sell Order Id: {} Quantity: {} Price: {}", order_match.buy_order_id, order_match.sell_order_id, order_match.quantity, order_match.price);
            }

            for order_match in &order_matches {
                order_system.create_order_history(order_match, &mut accounts_system);
            }
            std::thread::sleep(std::time::Duration::from_secs(1));

            println!("After matching orders");
            print_accounts(&accounts_system, &assets_system);

        }

    });
    thread_handle.join().unwrap();
}

fn print_accounts(accounts_system: &AccountSystem, assets_system: &AssetSystem) {
    for account in &accounts_system.accounts {
        print!("Id: {} Account: {} ",account.id,  account.name);
        let datetime: DateTime<Local> = account.timestamp.into();
        println!("Timestamp: {}", datetime.format("%Y-%m-%d %H:%M:%S").to_string());
        for account_currency in accounts_system.account_currencies.get(&account.id).unwrap() {
            println!("Id: {} Currency: {} Amount: {:.2}", account_currency.id, assets_system.currencies.get(&account_currency.currency_id).unwrap().symbol, account_currency.balance);
        }
        for account_stock in accounts_system.account_stocks.get(&account.id).or(Some(&vec![])).unwrap() {
            println!("Id: {} Stock: {} Amount: {}", account_stock.id, assets_system.stocks.get(&account_stock.stock_id).unwrap().symbol, account_stock.quantity);
        }
    }

}
