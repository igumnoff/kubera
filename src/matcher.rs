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
    pub buy_orders: HashMap<u64,Order>,
    pub sell_orders: HashMap<u64,Order>,
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
            buy_orders: Default::default(),
            sell_orders: Default::default(),
        }
    }

    pub fn add_order(&mut self, order: Order) {
        assert!(order.crypto_currency_id == self.crypto_currency_id);
        assert!(order.currency_id == self.currency_id);
        if order.trade_type == crate::orders::TradeType::Buy {
            self.buy_orders.insert(order.id, order);
        } else {
            self.sell_orders.insert(order.id, order);
        }
    }

    pub fn match_orders(&mut self) -> Vec<OrderMatch> {
        self.print_orders("Before Matching");

        let mut matches = Vec::new();

        let mut buy_orders: Vec<_> = self.buy_orders.drain().map(|(_, o)| o).collect();
        let mut sell_orders: Vec<_> = self.sell_orders.drain().map(|(_, o)| o).collect();

        // Matching logic
        let mut i = 0;
        let mut j = 0;

        while i < buy_orders.len() && j < sell_orders.len() {
            let buy = &buy_orders[i];
            let sell = &sell_orders[j];

            // Check for match
            match (buy.price_type, sell.price_type) {
                (PriceType::Market, PriceType::Limit(sell_price))   => {
                    // Calculate matched quantity
                    let matched_quantity = buy.quantity.min(sell.quantity);

                    // Create match record
                    matches.push(OrderMatch {
                        buy_order_id: buy.id,
                        sell_order_id: sell.id,
                        quantity: matched_quantity,
                        price: sell_price, // Execute at sell price
                        timestamp: SystemTime::now(),
                    });

                    // Update quantities
                    buy_orders[i].quantity -= matched_quantity;
                    sell_orders[j].quantity -= matched_quantity;
                    j += 1; // move to next sell order
                    i += 1; // move to next buy order
            },
                _ => i += 1, // No match, move to next buy order
            }

        }

        // Any remaining unmatched orders are put back into the HashMap
        for buy in buy_orders {
            if buy.quantity > 0.0 {
                self.buy_orders.insert(buy.id, buy);
            }
        }
        for sell in sell_orders {
            if sell.quantity > 0.0 {
                self.sell_orders.insert(sell.id, sell);
            }
        }

        self.print_orders("After Matching");
        matches
    }

    fn print_orders(&self, title: &str) {
        if !self.buy_orders.is_empty() || !self.sell_orders.is_empty() {
            tracing::info!("{title}");
        }
        for order in self.buy_orders.values() {
            tracing::info!("Buy Order: {:?}", order);
        }
        for order in self.sell_orders.values() {
            tracing::info!("Sell Order: {:?}", order);
        }
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