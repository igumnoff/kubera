# Kubera

![Kubera](logo.png)

Trade-Exchange Engine Library for Cryptocurrencies market written in Rust.

## Features
- AccountSystem: accounts, currencies, cryptocurrencies, history
- AssetSystem: currencies, cryptocurrencies
- OrderSystem: orders, history
- MatcherSystem: matching orders full or partial (only for buy Market and Sell limit)
- StorageSystem: [redb](https://github.com/cberner/redb) high-performance, ACID, embedded key-value store

## TODO
- MatcherSystem: matching orders full or partial (for all types of orders)
- StorageSystem: sharding, distributed transactions, distributed storage

# How to run example
```bash
cargo run --example trade-engine
```

# Usage
Add this to your `Cargo.toml`:
```toml
[dependencies]
kubera = "0.0.2"
```

# Example
```rust
fn main() {
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
```

## Output
```
2024-07-08T14:55:28.291721Z  INFO ThreadId(01) examples\trade-engine.rs:81: AccountId: 1 Name: Alice Timestamp: 2024-07-08 19:55:28
2024-07-08T14:55:28.291953Z  INFO ThreadId(01) examples\trade-engine.rs:86: CurrencyId: 1 Symbol: USD Balance: 100000.00
2024-07-08T14:55:28.292068Z  INFO ThreadId(01) examples\trade-engine.rs:92: CurrencyHistoryId: 1 Balance: 100000.00 Timestamp: 2024-07-08 19:55:28
2024-07-08T14:55:28.292168Z  INFO ThreadId(01) examples\trade-engine.rs:81: AccountId: 2 Name: Bob Timestamp: 2024-07-08 19:55:28
2024-07-08T14:55:28.292261Z  INFO ThreadId(01) examples\trade-engine.rs:86: CurrencyId: 2 Symbol: USD Balance: 0.00
2024-07-08T14:55:28.292380Z  INFO ThreadId(01) examples\trade-engine.rs:98: CryptoCurrencyId: 1 BTC Amount: 1
2024-07-08T14:55:28.292476Z  INFO ThreadId(01) examples\trade-engine.rs:104: CryptoCurrencyHistoryId: 1 Quantity: 1 Timestamp: 2024-07-08 19:55:28
2024-07-08T14:55:29.288109Z  INFO ThreadId(02) src\matcher.rs:157: Before Matching
2024-07-08T14:55:29.288289Z  INFO ThreadId(02) src\matcher.rs:160: Buy Order: Order { id: 1, account_id: 1, trade_type: Buy, price_type: Market, execution_type: Full, crypto_currency_id: 1, currency_id: 1, quantity: 0.5, timestamp: SystemTime { intervals: 133649241282877806 }, status: Open }
2024-07-08T14:55:29.288362Z  INFO ThreadId(02) src\matcher.rs:163: Sell Order: Order { id: 2, account_id: 2, trade_type: Sell, price_type: Limit(50000.0), execution_type: Partial, crypto_currency_id: 1, currency_id: 1, quantity: 1.0, timestamp: SystemTime { intervals: 133649241282892932 }, status: Open }
2024-07-08T14:55:29.288436Z  INFO ThreadId(02) src\matcher.rs:157: After Matching
2024-07-08T14:55:29.288492Z  INFO ThreadId(02) src\matcher.rs:163: Sell Order: Order { id: 2, account_id: 2, trade_type: Sell, price_type: Limit(50000.0), execution_type: Partial, crypto_currency_id: 1, currency_id: 1, quantity: 0.5, timestamp: SystemTime { intervals: 133649241282892932 }, status: Open }
2024-07-08T14:55:29.292973Z  INFO ThreadId(01) examples\trade-engine.rs:64: OrderMatch: Buy Order Id: 1 Sell Order Id: 2 Quantity: 0.5 Price: 50000
2024-07-08T14:55:29.312581Z  INFO ThreadId(01) examples\trade-engine.rs:81: AccountId: 1 Name: Alice Timestamp: 2024-07-08 19:55:28
2024-07-08T14:55:29.312779Z  INFO ThreadId(01) examples\trade-engine.rs:86: CurrencyId: 1 Symbol: USD Balance: 75000.00
2024-07-08T14:55:29.312920Z  INFO ThreadId(01) examples\trade-engine.rs:92: CurrencyHistoryId: 1 Balance: 100000.00 Timestamp: 2024-07-08 19:55:28
2024-07-08T14:55:29.313002Z  INFO ThreadId(01) examples\trade-engine.rs:92: CurrencyHistoryId: 2 Balance: 75000.00 Timestamp: 2024-07-08 19:55:29
2024-07-08T14:55:29.313127Z  INFO ThreadId(01) examples\trade-engine.rs:98: CryptoCurrencyId: 2 BTC Amount: 0.5
2024-07-08T14:55:29.313255Z  INFO ThreadId(01) examples\trade-engine.rs:104: CryptoCurrencyHistoryId: 2 Quantity: 0.5 Timestamp: 2024-07-08 19:55:29
2024-07-08T14:55:29.313335Z  INFO ThreadId(01) examples\trade-engine.rs:81: AccountId: 2 Name: Bob Timestamp: 2024-07-08 19:55:28
2024-07-08T14:55:29.313454Z  INFO ThreadId(01) examples\trade-engine.rs:86: CurrencyId: 2 Symbol: USD Balance: 25000.00
2024-07-08T14:55:29.313579Z  INFO ThreadId(01) examples\trade-engine.rs:92: CurrencyHistoryId: 3 Balance: 25000.00 Timestamp: 2024-07-08 19:55:29
2024-07-08T14:55:29.313707Z  INFO ThreadId(01) examples\trade-engine.rs:98: CryptoCurrencyId: 1 BTC Amount: 0.5
2024-07-08T14:55:29.313814Z  INFO ThreadId(01) examples\trade-engine.rs:104: CryptoCurrencyHistoryId: 1 Quantity: 1 Timestamp: 2024-07-08 19:55:28
2024-07-08T14:55:29.313881Z  INFO ThreadId(01) examples\trade-engine.rs:104: CryptoCurrencyHistoryId: 3 Quantity: 0.5 Timestamp: 2024-07-08 19:55:29
2024-07-08T14:55:30.288721Z  INFO ThreadId(02) src\matcher.rs:157: Before Matching
2024-07-08T14:55:30.288946Z  INFO ThreadId(02) src\matcher.rs:160: Buy Order: Order { id: 3, account_id: 1, trade_type: Buy, price_type: Market, execution_type: Full, crypto_currency_id: 1, currency_id: 1, quantity: 0.5, timestamp: SystemTime { intervals: 133649241293139637 }, status: Open }
2024-07-08T14:55:30.289023Z  INFO ThreadId(02) src\matcher.rs:163: Sell Order: Order { id: 2, account_id: 2, trade_type: Sell, price_type: Limit(50000.0), execution_type: Partial, crypto_currency_id: 1, currency_id: 1, quantity: 0.5, timestamp: SystemTime { intervals: 133649241282892932 }, status: Open }
2024-07-08T14:55:30.315944Z  INFO ThreadId(01) examples\trade-engine.rs:64: OrderMatch: Buy Order Id: 3 Sell Order Id: 2 Quantity: 0.5 Price: 50000
2024-07-08T14:55:30.334073Z  INFO ThreadId(01) examples\trade-engine.rs:81: AccountId: 1 Name: Alice Timestamp: 2024-07-08 19:55:28
2024-07-08T14:55:30.334267Z  INFO ThreadId(01) examples\trade-engine.rs:86: CurrencyId: 1 Symbol: USD Balance: 50000.00
2024-07-08T14:55:30.334448Z  INFO ThreadId(01) examples\trade-engine.rs:92: CurrencyHistoryId: 1 Balance: 100000.00 Timestamp: 2024-07-08 19:55:28
2024-07-08T14:55:30.334571Z  INFO ThreadId(01) examples\trade-engine.rs:92: CurrencyHistoryId: 2 Balance: 75000.00 Timestamp: 2024-07-08 19:55:29
2024-07-08T14:55:30.334686Z  INFO ThreadId(01) examples\trade-engine.rs:92: CurrencyHistoryId: 4 Balance: 50000.00 Timestamp: 2024-07-08 19:55:30
2024-07-08T14:55:30.334863Z  INFO ThreadId(01) examples\trade-engine.rs:98: CryptoCurrencyId: 2 BTC Amount: 1
2024-07-08T14:55:30.335030Z  INFO ThreadId(01) examples\trade-engine.rs:104: CryptoCurrencyHistoryId: 2 Quantity: 0.5 Timestamp: 2024-07-08 19:55:29
2024-07-08T14:55:30.335130Z  INFO ThreadId(01) examples\trade-engine.rs:104: CryptoCurrencyHistoryId: 4 Quantity: 1 Timestamp: 2024-07-08 19:55:30
2024-07-08T14:55:30.335217Z  INFO ThreadId(01) examples\trade-engine.rs:81: AccountId: 2 Name: Bob Timestamp: 2024-07-08 19:55:28
2024-07-08T14:55:30.335340Z  INFO ThreadId(01) examples\trade-engine.rs:86: CurrencyId: 2 Symbol: USD Balance: 50000.00
2024-07-08T14:55:30.335495Z  INFO ThreadId(01) examples\trade-engine.rs:92: CurrencyHistoryId: 3 Balance: 25000.00 Timestamp: 2024-07-08 19:55:29
2024-07-08T14:55:30.335588Z  INFO ThreadId(01) examples\trade-engine.rs:92: CurrencyHistoryId: 5 Balance: 50000.00 Timestamp: 2024-07-08 19:55:30
2024-07-08T14:55:30.335720Z  INFO ThreadId(01) examples\trade-engine.rs:98: CryptoCurrencyId: 1 BTC Amount: 0
2024-07-08T14:55:30.335846Z  INFO ThreadId(01) examples\trade-engine.rs:104: CryptoCurrencyHistoryId: 1 Quantity: 1 Timestamp: 2024-07-08 19:55:28
2024-07-08T14:55:30.335923Z  INFO ThreadId(01) examples\trade-engine.rs:104: CryptoCurrencyHistoryId: 3 Quantity: 0.5 Timestamp: 2024-07-08 19:55:29
2024-07-08T14:55:30.335993Z  INFO ThreadId(01) examples\trade-engine.rs:104: CryptoCurrencyHistoryId: 5 Quantity: 0 Timestamp: 2024-07-08 19:55:30
2024-07-08T14:55:31.289822Z  INFO ThreadId(02) src\matcher.rs:157: Before Matching
2024-07-08T14:55:31.290319Z  INFO ThreadId(02) src\matcher.rs:160: Buy Order: Order { id: 4, account_id: 1, trade_type: Buy, price_type: Market, execution_type: Full, crypto_currency_id: 1, currency_id: 1, quantity: 0.5, timestamp: SystemTime { intervals: 133649241303360887 }, status: Open }
2024-07-08T14:55:31.290590Z  INFO ThreadId(02) src\matcher.rs:157: After Matching
```

## Contributing
I would love to see contributions from the community. If you experience bugs, feel free to open an issue. If you would like to implement a new feature or bug fix, please follow the steps:
1. Read "[Contributor License Agreement (CLA)](https://github.com/igumnoff/kubera/blob/main/CLA)"
2. Contact with me via telegram @ievkz or discord @igumnovnsk
3. Confirm e-mail invitation in repository
4. Do "git clone" (You don't need to fork!)
5. Create branch with your assigned issue
6. Create pull request to main branch
