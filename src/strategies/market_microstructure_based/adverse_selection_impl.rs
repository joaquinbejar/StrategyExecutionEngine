/*******************************************************************************
Copyright (c) 2024.

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
THE SOFTWARE.
******************************************************************************/

/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/5/24
******************************************************************************/

use std::time::SystemTime;
use rand::Rng;
use crate::models::orders::Side;
use crate::models::{ChildOrder, ParentOrder};
use crate::strategies::OrderSplitStrategy;

/// Market state enum for adverse selection strategy
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarketState {
    Normal,
    BuyerInformed,
    SellerInformed,
    HighVolatility,
}

/// Configuration for adverse selection strategy
#[derive(Debug, Clone)]
pub struct AdverseSelectionConfig {
    pub max_splits: usize,
    pub min_split_interval_ms: u64,
    pub max_split_interval_ms: u64,
    pub size_variation_pct: f64,
}

impl Default for AdverseSelectionConfig {
    fn default() -> Self {
        Self {
            max_splits: 5,
            min_split_interval_ms: 1000,
            max_split_interval_ms: 10000,
            size_variation_pct: 0.2,
        }
    }
}

/// Adverse selection strategy implementation
#[derive(Debug, Clone)]
pub struct AdverseSelectionStrategy {
    pub config: AdverseSelectionConfig,
    pub market_state: MarketState,
}

impl AdverseSelectionStrategy {
    pub fn new(config: Option<AdverseSelectionConfig>) -> Self {
        Self {
            config: config.unwrap_or_default(),
            market_state: MarketState::Normal,
        }
    }
    
    pub fn update_market_state(&mut self, state: MarketState) {
        self.market_state = state;
    }
}

/// Implement order splitting strategy
impl OrderSplitStrategy for AdverseSelectionStrategy {
    fn split(&self, parent_order: &ParentOrder) -> Vec<ChildOrder> {
        let mut child_orders = Vec::new();
        let mut rng = rand::thread_rng();
        
        // Determine number of splits based on market state
        let num_splits = match self.market_state {
            MarketState::Normal => self.config.max_splits / 2,
            MarketState::BuyerInformed => {
                if parent_order.order_common.side == Side::Buy {
                    self.config.max_splits
                } else {
                    self.config.max_splits / 3
                }
            },
            MarketState::SellerInformed => {
                if parent_order.order_common.side == Side::Sell {
                    self.config.max_splits
                } else {
                    self.config.max_splits / 3
                }
            },
            MarketState::HighVolatility => self.config.max_splits,
        };
        
        // Calculate base size for each child order
        let base_quantity = parent_order.order_common.quantity / num_splits as u32;
        let mut remaining_quantity = parent_order.order_common.quantity;
        
        // Create child orders
        for i in 0..num_splits {
            // Determine quantity for this child order
            let quantity = if i < num_splits - 1 {
                let variation = (rng.gen::<f64>() * 0.2 - 0.1) * base_quantity as f64;
                let qty = (base_quantity as f64 + variation).max(1.0) as u32;
                qty.min(remaining_quantity)
            } else {
                // Last order uses all remaining quantity
                remaining_quantity
            };
            
            // Update remaining quantity
            remaining_quantity = remaining_quantity.saturating_sub(quantity);
            
            // Calculate execution time
            let interval_ms = if i == 0 {
                0 // First order executes immediately
            } else {
                let base_interval = match self.market_state {
                    MarketState::Normal => 5000, // 5 seconds
                    MarketState::BuyerInformed | MarketState::SellerInformed => 8000, // 8 seconds
                    MarketState::HighVolatility => 3000, // 3 seconds
                };
                
                let variation = (rng.gen::<f64>() * 0.4 - 0.2) * base_interval as f64;
                (base_interval as f64 * i as f64 + variation) as u64
            };
            
            // Get current time in milliseconds
            let now = SystemTime::now();
            let execution_time = match now.duration_since(SystemTime::UNIX_EPOCH) {
                Ok(duration) => duration.as_millis() as u64 + interval_ms,
                Err(_) => interval_ms,
            };
            
            // Create a new order based on parent order
            let mut order = parent_order.order_common.clone();
            order.id = format!("{}-{}", parent_order.order_common.id, i);
            order.quantity = quantity;
            
            // Create child order
            let child_order = ChildOrder {
                order_common: order,
                strategy_id: parent_order.strategy_id.clone(),
                parent_id: parent_order.order_common.id.clone(),
                insert_at: Some(execution_time),
            };
            
            child_orders.push(child_order);
        }
        
        child_orders
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::orders::{Order, ProductType, OrderType, TimeInForce};
    use std::time::UNIX_EPOCH;
    
    #[test]
    fn test_split_normal_market() {
        let strategy = AdverseSelectionStrategy::new(None);
        
        let order = Order::new(
            "parent-1".to_string(),
            1000,
            ProductType::Spot,
            OrderType::Limit,
            Some(100.0),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64,
            None,
            "BTC/USD".to_string(),
            Side::Buy,
            "USD".to_string(),
            Some("BINANCE".to_string()),
            Some(TimeInForce::GTC),
            None,
            None,
            None,
            None,
            None,
            None,
        );
        
        let parent_order = ParentOrder {
            order_common: order,
            strategy_id: "test-strategy".to_string(),
        };
        
        let child_orders = strategy.split(&parent_order);
        
        // In normal market state, we should have max_splits/2 child orders
        assert_eq!(child_orders.len(), strategy.config.max_splits / 2);
        
        // Total quantity should match parent order
        let total_quantity: u32 = child_orders.iter().map(|o| o.order_common.quantity).sum();
        assert_eq!(total_quantity, parent_order.order_common.quantity);
        
        // First child order should execute immediately
        assert!(child_orders[0].insert_at.unwrap() < child_orders[1].insert_at.unwrap());
        println!("child_orders: {:?}", child_orders);
    }
    
    #[test]
    fn test_split_buyer_informed() {
        let strategy = AdverseSelectionStrategy {
            config: AdverseSelectionConfig::default(),
            market_state: MarketState::BuyerInformed,
        };
        
        // Test buy parent
        let buy_order = Order::new(
            "test-order-buy".to_string(),
            1000,
            ProductType::Spot,
            OrderType::Market,
            Some(50000.0),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64,
            None,
            "BTC/USD".to_string(),
            Side::Buy,
            "USD".to_string(),
            Some("BINANCE".to_string()),
            Some(TimeInForce::GTC),
            None, None, None, None, None, None
        );
        
        let buy_parent = ParentOrder {
            order_common: buy_order,
            strategy_id: "TWAP".to_string(),
        };
        
        let buy_children = strategy.split(&buy_parent);
        assert_eq!(buy_children.len(), strategy.config.max_splits);
        println!("buy_children: {:?}", buy_children);
        // Test sell parent
        let sell_order = Order::new(
            "test-order-sell".to_string(),
            1000,
            ProductType::Spot,
            OrderType::Market,
            Some(50000.0),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64,
            None,
            "BTC/USD".to_string(),
            Side::Sell,
            "USD".to_string(),
            Some("BINANCE".to_string()),
            Some(TimeInForce::GTC),
            None, None, None, None, None, None
        );
        
        let sell_parent = ParentOrder {
            order_common: sell_order,
            strategy_id: "TWAP".to_string(),
        };
        
        let sell_children = strategy.split(&sell_parent);
        assert_eq!(sell_children.len(), strategy.config.max_splits / 3);
        println!("sell_children: {:?}", sell_children);
    }
} 