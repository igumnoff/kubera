use std::sync::Arc;
use std::time::SystemTime;
use bincode::{Decode, Encode};
use crate::accounts::AccountSystem;
use crate::assets::AssetSystem;
use crate::matcher::OrderMatch;
use crate::storage::StorageSystem;

#[derive(Debug, Clone, Copy, Encode, Decode)]
pub struct Order {
    pub id: u64,
    pub account_id: u64,
    pub trade_type: TradeType,
    pub price_type: PriceType,
    pub execution_type: ExecutionType,
    pub crypto_currency_id: u64,
    pub currency_id: u64,
    pub quantity: u64,
    pub timestamp: SystemTime,
    pub status: OrderStatus,
}

#[derive(Debug, Clone, Copy, Encode, Decode)]
pub struct OrderHistory {
    pub id: u64,
    pub order_id: u64,
    pub quantity: u64,
    pub timestamp: SystemTime,
    pub status: OrderStatus,
}

#[derive(Debug, Clone, Copy, Encode, Decode)]
pub enum OrderStatus {
    Open,
    Closed,
    PartiallyFilled,
    Cancelled,
}

#[derive(Debug, PartialEq, Clone, Copy, Encode, Decode)]
pub enum TradeType {
    Buy,
    Sell,
}

#[derive(Debug, PartialEq,Clone, Copy, Encode, Decode)]
pub enum PriceType {
    Market,
    Limit(f64),
}

#[derive(Debug, Clone, Copy, Encode, Decode)]
pub enum ExecutionType {
    Full,
    Partial,
}

pub struct OrderSystem {
    pub order_last_id: u64,
    pub order_history_id: u64,
    pub storage_system: Arc<StorageSystem>,
    pub assets_system: Arc<AssetSystem>,
}

impl OrderSystem {
    pub fn new(storage_system: Arc<StorageSystem>, assets_system: Arc<AssetSystem>) -> OrderSystem {
        let mut order_last_id = 0;
        match storage_system.get_last_order() {
            None => {}
            Some(order) => {
                order_last_id = order.id;
            }
        }
        let mut order_history_id = 0;
        match storage_system.get_last_order_history() {
            None => {}
            Some(order_history) => {
                order_history_id = order_history.id;
            }
        }


        OrderSystem {
            order_last_id,
            order_history_id,
            storage_system,
            assets_system,
        }
    }

    pub fn create_order(&mut self, mut order: Order) -> Order {
        self.order_last_id += 1;
        order.id = self.order_last_id;
        self.storage_system.add_order(&order);
        order
    }

    pub fn create_order_history(&mut self, order_match: &OrderMatch,  accounts_system: &mut AccountSystem) {
        {

            let mut buy_order: Order = self.storage_system.get_order(order_match.buy_order_id).unwrap();
            let quantity = self.storage_system.get_order_histories_by_order_id(order_match.buy_order_id).iter().fold(0, |acc, x| acc + x.quantity);
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
            self.storage_system.add_order_history(&order_history);
            accounts_system.add_currency_to_account(buy_order.account_id, buy_order.currency_id, -(order_match.quantity as f64 * order_match.price));
            accounts_system.add_crypto_currency_to_account(buy_order.account_id, buy_order.crypto_currency_id, order_match.quantity as i64);
        }

        let mut sell_order: Order = self.storage_system.get_order(order_match.sell_order_id).unwrap();
        let quantity = self.storage_system.get_order_histories_by_order_id(order_match.sell_order_id).iter().fold(0, |acc, x| acc + x.quantity);
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
        self.storage_system.add_order_history(&order_history);
        accounts_system.add_currency_to_account(sell_order.account_id, sell_order.currency_id, order_match.quantity as f64 * order_match.price);
        accounts_system.add_crypto_currency_to_account(sell_order.account_id, sell_order.crypto_currency_id, -(order_match.quantity as i64));

    }

}