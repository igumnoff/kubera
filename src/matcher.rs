use std::collections::HashMap;
use std::time::{SystemTime};
use crate::accounts::AccountSystem;
use crate::orders::{Order, PriceType};


pub struct MatcherSystem {
    pub stock_id: u64,
    pub currency_id: u64,
    pub orders: Vec<Order>,
    pub orders_hash_map: HashMap<u64,u64>, // order.id -> index in orders
    pub timestamp: SystemTime,
}

pub struct OrderMatch {
    pub buy_order_id: u64,
    pub sell_order_id: u64,
    pub quantity: u64,
    pub price: f64,
    pub timestamp: SystemTime,
}

impl MatcherSystem {
    pub fn new(stock_id: u64, currency_id: u64) -> MatcherSystem {
        MatcherSystem {
                stock_id,
                currency_id,
                orders: vec![],
                orders_hash_map: HashMap::new(),
                timestamp: SystemTime::now(),
        }
    }

    pub fn add_order(&mut self, order: Order) {
        assert!(order.stock_id == self.stock_id);
        assert!(order.currency_id == self.currency_id);
        self.orders_hash_map.insert(order.id, self.orders.len() as u64);
        self.orders.push(order);
    }

    pub fn match_orders(&mut self, _accounts_system: &mut AccountSystem) -> Vec<OrderMatch> {
        let mut order_for_deletion:Vec<u64> = vec![];
        let mut order_for_decrease:Vec<(u64,u64)> = vec![]; // order_id, quantity
        let mut order_matches = vec![];
        let mut i = 0;
        while i < self.orders.len() {
            let order = &self.orders[i];
            if order.trade_type == crate::orders::TradeType::Buy {
                let mut j = 0;
                while j < self.orders.len() {
                    let order2 = &self.orders[j];
                    if order2.trade_type == crate::orders::TradeType::Sell {
                        if order.stock_id == order2.stock_id && order.currency_id == order2.currency_id {
                            if order.price_type == crate::orders::PriceType::Market {
                                match order2.price_type {
                                    PriceType::Market => {}
                                    PriceType::Limit(price) => {
                                        let quantity = std::cmp::min(order.quantity, order2.quantity);
                                        let order_match = OrderMatch {
                                            buy_order_id: order.id,
                                            sell_order_id: order2.id,
                                            quantity,
                                            price,
                                            timestamp: self.timestamp,
                                        };
                                        order_matches.push(order_match);
                                        // TODO: check if the account has enough currency and stock
                                        // accounts_system.add_currency_to_account(order.account_id, order.currency_id, -(quantity as f64 * price));
                                        // accounts_system.add_stock_to_account(order.account_id, order.stock_id, quantity as i64);
                                        // accounts_system.add_currency_to_account(order2.account_id, order2.currency_id, quantity as f64 * price);
                                        // accounts_system.add_stock_to_account(order2.account_id, order2.stock_id, -(quantity as i64));
                                        if order.quantity == quantity {
                                            order_for_deletion.push(order.id);
                                        } else {
                                            order_for_decrease.push((order.id, quantity));
                                        }
                                        if order2.quantity == quantity {
                                            order_for_deletion.push(order2.id);
                                        } else {
                                            order_for_decrease.push((order2.id, quantity));
                                        }
                                    }
                                }

                            }
                        }
                    }
                    j += 1;
                }
            }
            i += 1;
        }
        order_for_decrease.iter().for_each(|(order_id, quantity)| {
            let index = self.orders_hash_map.get(order_id).unwrap();
            self.orders[*index as usize].quantity -= quantity;
        });

        order_for_deletion.iter().for_each(|order_id| {
            let index = self.orders_hash_map.get(order_id).unwrap();
            self.orders.remove(*index as usize);
            self.orders_hash_map.remove(order_id);
        });

        order_matches
    }
}

