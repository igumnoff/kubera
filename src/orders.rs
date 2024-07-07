use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use crate::accounts::AccountSystem;
use crate::assets::AssetSystem;
use crate::matcher::OrderMatch;
use crate::storage::StorageSystem;

#[derive(Debug, Clone, Copy)]
pub struct Order {
    pub id: u64,
    pub account_id: u64,
    pub trade_type: TradeType,
    pub price_type: PriceType,
    pub execution_type: ExecutionType,
    pub stock_id: u64,
    pub currency_id: u64,
    pub quantity: u64,
    pub timestamp: SystemTime,
    pub status: OrderStatus,
}

pub struct OrderHistory {
    pub id: u64,
    pub order_id: u64,
    pub quantity: u64,
    pub timestamp: SystemTime,
    pub status: OrderStatus,
}

#[derive(Debug, Clone, Copy)]
pub enum OrderStatus {
    Open,
    Closed,
    PartiallyFilled,
    Cancelled,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TradeType {
    Buy,
    Sell,
}

#[derive(Debug, PartialEq,Clone, Copy)]
pub enum PriceType {
    Market,
    Limit(f64),
}

#[derive(Debug, Clone, Copy)]
pub enum ExecutionType {
    Full,
    Partial,
}

pub struct OrderSystem {
    pub orders: HashMap<u64,Vec<Order>>, // account_id -> orders
    pub orders_hash_map: HashMap<u64,(u64,u64)>, // order.id -> (account_id, index in orders)
    pub order_histories: HashMap<u64,Vec<OrderHistory>>, // account_id -> order_histories
    pub order_last_id: u64,
    pub order_history_id: u64,
    pub storage_system: Arc<StorageSystem>,
    pub assets_system: Arc<AssetSystem>,
}

impl OrderSystem {
    pub fn new(storage_system: Arc<StorageSystem>, assets_system: Arc<AssetSystem>) -> OrderSystem {
        OrderSystem {
            orders: HashMap::new(),
            orders_hash_map: HashMap::new(),
            order_histories: HashMap::new(),
            order_last_id: 0,
            order_history_id: 0,
            storage_system,
            assets_system,
        }
    }

    pub fn create_order(&mut self, mut order: Order) -> Order {
        self.order_last_id += 1;
        order.id = self.order_last_id;
        let account_id = order.account_id;
        self.orders.entry(account_id).or_insert(vec![]).push(order);
        self.orders_hash_map.insert(order.id, (account_id, self.orders.get(&account_id).unwrap().len() as u64 - 1));
        order
    }

    pub fn create_order_history(&mut self, order_match: &OrderMatch,  accounts_system: &mut AccountSystem) {
        {
            let (account_id, order_index) = self.orders_hash_map.get(&order_match.buy_order_id).unwrap();
            let buy_order: &mut Order = self.orders.get_mut(&account_id).unwrap().get_mut(*order_index as usize).unwrap();
            let quantity = self.order_histories.get(&account_id).or(Some(&vec![])).unwrap().iter().fold(0, |acc, x| acc + x.quantity);

            let status = if (quantity + order_match.quantity) == buy_order.quantity {
                OrderStatus::Closed
            } else {
                OrderStatus::PartiallyFilled
            };
            buy_order.status = status;

            self.order_history_id += 1;
            let order_history = OrderHistory {
                id: self.order_history_id,
                order_id: order_match.buy_order_id,
                quantity: order_match.quantity,
                timestamp: order_match.timestamp,
                status: status,
            };

            self.order_histories.entry(*account_id).or_insert(vec![]).push(order_history);
            accounts_system.add_currency_to_account(*account_id, buy_order.currency_id, -(order_match.quantity as f64 * order_match.price));
            accounts_system.add_stock_to_account(*account_id, buy_order.stock_id, order_match.quantity as i64);
        }


        let (account_id, order_index) = self.orders_hash_map.get(&order_match.sell_order_id).unwrap();

        let sell_order: &mut Order = self.orders.get_mut(&account_id).unwrap().get_mut(*order_index as usize).unwrap();
        let quantity = self.order_histories.get(&account_id).or(Some(&vec![])).unwrap().iter().fold(0, |acc, x| acc + x.quantity);

        let status = if (quantity + order_match.quantity) == sell_order.quantity {
            OrderStatus::Closed
        } else {
            OrderStatus::PartiallyFilled
        };
        sell_order.status = status;

        self.order_history_id += 1;
        let order_history = OrderHistory {
            id: self.order_history_id,
            order_id: order_match.sell_order_id,
            quantity: order_match.quantity,
            timestamp: order_match.timestamp,
            status: status,
        };
        self.order_histories.entry(*account_id).or_insert(vec![]).push(order_history);
        accounts_system.add_currency_to_account(*account_id, sell_order.currency_id, order_match.quantity as f64 * order_match.price);
        accounts_system.add_stock_to_account(*account_id, sell_order.stock_id, -(order_match.quantity as i64));
    }

}