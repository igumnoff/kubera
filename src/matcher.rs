use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime};
use core_affinity::CoreId;
use crossbeam_queue::ArrayQueue;
use tracing::{Level, span};
use crate::orders::{Order, PriceType};


#[derive(Debug)]
pub struct OrderMatcher {
    pub crypto_currency_id: u64,
    pub currency_id: u64,
    pub orders: Vec<Order>,
    pub orders_hash_map: HashMap<u64,u64>, // order.id -> index in orders
    pub timestamp: SystemTime,
}

pub struct OrderMatch {
    pub buy_order_id: u64,
    pub sell_order_id: u64,
    pub quantity: f64,
    pub price: f64,
    pub timestamp: SystemTime,
}

impl OrderMatcher {
    pub fn new(crypto_currency_id: u64, currency_id: u64) -> OrderMatcher {
        OrderMatcher {
            crypto_currency_id,
                currency_id,
                orders: vec![],
                orders_hash_map: HashMap::new(),
                timestamp: SystemTime::now(),
        }
    }

    pub fn add_order(&mut self, order: Order) {
        assert!(order.crypto_currency_id == self.crypto_currency_id);
        assert!(order.currency_id == self.currency_id);
        self.orders_hash_map.insert(order.id, self.orders.len() as u64);
        self.orders.push(order);
    }

    // #[tracing::instrument(level = "info")]
    pub fn match_orders(&mut self) -> Vec<OrderMatch> {
        let mut order_for_deletion:Vec<u64> = vec![];
        let mut order_for_decrease:Vec<(u64,f64)> = vec![]; // order_id, quantity
        let mut order_matches = vec![];
        let mut i = 0;
        while i < self.orders.len() {
            let order = &self.orders[i];
            if order.trade_type == crate::orders::TradeType::Buy {
                let mut j = 0;
                while j < self.orders.len() {
                    let order2 = &self.orders[j];
                    if order2.trade_type == crate::orders::TradeType::Sell {
                        if order.crypto_currency_id == order2.crypto_currency_id && order.currency_id == order2.currency_id {
                            if order.price_type == crate::orders::PriceType::Market {
                                match order2.price_type {
                                    PriceType::Market => {}
                                    PriceType::Limit(price) => {
                                        let quantity = if order.quantity < order2.quantity {
                                            order.quantity
                                        } else {
                                            order2.quantity
                                        };
                                        let order_match = OrderMatch {
                                            buy_order_id: order.id,
                                            sell_order_id: order2.id,
                                            quantity,
                                            price,
                                            timestamp: self.timestamp,
                                        };
                                        order_matches.push(order_match);
                                        // TODO: check if the account has enough currency and crypto currency
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

pub struct MatcherSystem {
    order_queue:Arc<ArrayQueue<Order>>,
    order_match_queue:Arc<ArrayQueue<OrderMatch>>,
}

impl MatcherSystem {
    pub fn start(crypto_currency_id: u64, currency_id: u64, core_id: CoreId) -> MatcherSystem {
        let order_queue:Arc<ArrayQueue<Order>> = Arc::new(ArrayQueue::new(100));
        let order_match_queue:Arc<ArrayQueue<OrderMatch>> = Arc::new(ArrayQueue::new(100));
        let order_queue_clone = order_queue.clone();
        let order_match_queue_clone = order_match_queue.clone();
        let _match_system_thread_handle = std::thread::spawn(move || {
            let ok = core_affinity::set_for_current(core_id);
            if ok {
                let mut matcher_system = OrderMatcher::new(crypto_currency_id, currency_id);
                loop {
                    while let Some(order) = order_queue_clone.pop() {
                        matcher_system.add_order(order);
                    };
                    let match_orders = span!(Level::TRACE, "match_orders");
                    let _ = match_orders.enter();
                    let order_matches = matcher_system.match_orders();
                    drop(match_orders);
                    for order_match in &order_matches {
                        tracing::info!("OrderMatch: Buy Order Id: {} Sell Order Id: {} Quantity: {} Price: {}", order_match.buy_order_id, order_match.sell_order_id, order_match.quantity, order_match.price);
                    }

                    for order_match in order_matches {
                        let _ = order_match_queue_clone.push(order_match);
                    }
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
            } else {
                panic!("Failed to set core affinity");
            }
        });


        MatcherSystem {
            order_queue,
            order_match_queue,
        }
    }

    pub fn add_order(&self, order: Order) {
        let _ = self.order_queue.push(order);
    }

    pub fn get_order_match(&self) -> Option<OrderMatch> {
        self.order_match_queue.pop()
    }


}