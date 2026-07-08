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

/*
Adverse Selection
According to "Empirical Market Microstructure" by Joel Hasbrouck, adverse
selection strategies involve monitoring and adjusting order executions to avoid
being exploited by traders with superior information. This involves being aware
of the information asymmetry in the market and making execution decisions
that mitigate the risk of trading against better-informed participants.

Example: A trader might adjust their execution strategy when they detect
unusual trading patterns that suggest informed trading, thus reducing the
potential losses from adverse selection.

Reference for Further Reading:
"Empirical Market Microstructure" by Joel Hasbrouck: This book covers
various aspects of adverse selection and its impact on trading strategies.
(Chapter 4 provides insights into adverse selection and methods to manage it).
*/

use std::collections::VecDeque;
use std::time::{SystemTime, Duration};
use serde::{Deserialize, Serialize};
use rand::Rng;
use crate::models::{ChildOrder, ParentOrder};
use crate::strategies::OrderSplitStrategy;
use crate::models::orders::Side;

// 导入项目中已有的模块
use crate::models::orders::{Order, OrderType as ModelOrderType, ProductType, TimeInForce};
use crate::models::child_orders::ChildOrder as ModelChildOrder;
use crate::models::parent_orders::ParentOrder as ModelParentOrder;
use crate::strategies::common_strategies::OrderSplitStrategy as CommonOrderSplitStrategy;

/// Strategy trait and related types
pub trait Strategy {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn state(&self) -> &StrategyState;
    fn set_state(&mut self, state: StrategyState);
    fn on_market_data(&mut self, data: &MarketData) -> Option<StrategySignal>;
    fn on_order_executed(&mut self, order: &Order);
    fn on_order_cancelled(&mut self, order: &Order);
    fn reset(&mut self);
}

/// Strategy state
#[derive(Debug, Clone, PartialEq)]
pub enum StrategyState {
    Idle,
    Running,
    Paused,
    Error,
}

/// Strategy signal
#[derive(Debug, Clone)]
pub enum StrategySignal {
    Buy {
        price: f64,
        size: f64,
        order_type: OrderType,
        reason: String,
    },
    Sell {
        price: f64,
        size: f64,
        order_type: OrderType,
        reason: String,
    },
}

/// Market data types
#[derive(Debug, Clone)]
pub enum MarketData {
    Trade(Trade),
    OrderBook(OrderBook),
    Ticker(Ticker),
}

/// Trade data
#[derive(Debug, Clone)]
pub struct Trade {
    pub timestamp: SystemTime,
    pub price: f64,
    pub size: f64,
    pub side: Side,
}

/// Order book data
#[derive(Debug, Clone, Default)]
pub struct OrderBook {
    pub bids: Vec<(f64, f64)>, // (price, size)
    pub asks: Vec<(f64, f64)>, // (price, size)
}

/// Ticker data
#[derive(Debug, Clone)]
pub struct Ticker {
    pub timestamp: SystemTime,
    pub bid: f64,
    pub ask: f64,
    pub last: f64,
}

/// Candle data
#[derive(Debug, Clone)]
pub struct Candle {
    pub timestamp: SystemTime,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

/// Order data (internal representation for the strategy)

/// Position data
#[derive(Debug, Clone, Default)]
pub struct Position {
    pub size: f64,
    pub avg_price: f64,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
}

/// Configuration parameters for the Adverse Selection strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdverseSelectionConfig {
    /// Threshold for order imbalance to detect potential informed trading
    pub imbalance_threshold: f64,
    /// Window size for calculating order flow imbalance
    pub window_size: usize,
    /// Threshold for price impact to detect adverse selection
    pub price_impact_threshold: f64,
    /// Threshold for trade size anomaly detection
    pub trade_size_threshold: f64,
    /// Cooldown period after detecting adverse selection (in seconds)
    pub cooldown_period: u64,
    /// Maximum position size allowed
    pub max_position_size: f64,
    /// Stop loss percentage
    pub stop_loss_pct: f64,
    /// Take profit percentage
    pub take_profit_pct: f64,
    /// Maximum splits
    pub max_splits: usize,
    /// Minimum split interval (milliseconds)
    pub min_split_interval_ms: u64,
    /// Maximum split interval (milliseconds)
    pub max_split_interval_ms: u64,
    /// Size variation percentage
    pub size_variation_pct: f64,
}

impl Default for AdverseSelectionConfig {
    fn default() -> Self {
        Self {
            imbalance_threshold: 0.7,
            window_size: 20,
            price_impact_threshold: 0.001,
            trade_size_threshold: 2.0,
            cooldown_period: 300,
            max_position_size: 1.0,
            stop_loss_pct: 0.01,
            take_profit_pct: 0.02,
            max_splits: 5,
            min_split_interval_ms: 500,
            max_split_interval_ms: 3000,
            size_variation_pct: 0.2,
        }
    }
}

/// Order side enum
#[derive(Debug, Clone, PartialEq)]
pub enum OrderSide {
    Buy,
    Sell,
}

/// Order type enum
#[derive(Debug, Clone, PartialEq)]
pub enum OrderType {
    Market,
    Limit,
    StopLoss,
    TakeProfit,
}

/// Adverse Selection strategy implementation
pub struct AdverseSelectionStrategy {
    /// Configuration parameters
    config: AdverseSelectionConfig,
    /// Current state of the strategy
    state: StrategyState,
    /// Recent trades for analysis
    recent_trades: VecDeque<Trade>,
    /// Recent order book snapshots
    recent_order_books: VecDeque<OrderBook>,
    /// Last time adverse selection was detected
    last_adverse_detection: Option<SystemTime>,
    /// Current position
    position: Position,
    /// Reference price for position management
    reference_price: Option<f64>,
    /// Current market state
    market_state: MarketState,
}

/// Market state evaluation
#[derive(Debug, Clone, PartialEq)]
enum MarketState {
    /// Normal market state
    Normal,
    /// Buyer informed state
    BuyerInformed,
    /// Seller informed state
    SellerInformed,
    /// High volatility state
    HighVolatility,
}

impl AdverseSelectionStrategy {
    /// Create a new instance of the Adverse Selection strategy
    pub fn new(config: AdverseSelectionConfig) -> Self {
        Self {
            config,
            state: StrategyState::Idle,
            recent_trades: VecDeque::with_capacity(100),
            recent_order_books: VecDeque::with_capacity(20),
            last_adverse_detection: None,
            position: Position::default(),
            reference_price: None,
            market_state: MarketState::Normal,
        }
    }

    /// Calculate order flow imbalance from recent order book data
    fn calculate_order_imbalance(&self) -> f64 {
        if self.recent_order_books.len() < 2 {
            return 0.0;
        }

        let current_book = self.recent_order_books.back().unwrap();
        let previous_book = &self.recent_order_books[self.recent_order_books.len() - 2];

        // Calculate bid and ask volume changes
        let bid_volume_current: f64 = current_book.bids.iter().map(|(_, size)| size).sum();
        let ask_volume_current: f64 = current_book.asks.iter().map(|(_, size)| size).sum();
        let bid_volume_previous: f64 = previous_book.bids.iter().map(|(_, size)| size).sum();
        let ask_volume_previous: f64 = previous_book.asks.iter().map(|(_, size)| size).sum();

        let bid_change = bid_volume_current - bid_volume_previous;
        let ask_change = ask_volume_current - ask_volume_previous;
        
        // Calculate imbalance ratio
        let total_change = bid_change.abs() + ask_change.abs();
        if total_change == 0.0 {
            return 0.0;
        }
        
        (bid_change - ask_change) / total_change
    }

    /// Detect abnormal trade sizes that might indicate informed trading
    fn detect_abnormal_trade_size(&self) -> bool {
        if self.recent_trades.len() < 10 {
            return false;
        }

        // Calculate average trade size
        let avg_size: f64 = self.recent_trades.iter().map(|t| t.size).sum::<f64>() / self.recent_trades.len() as f64;
        
        // Check if the most recent trade is significantly larger than average
        let latest_trade = self.recent_trades.back().unwrap();
        latest_trade.size > avg_size * self.config.trade_size_threshold
    }

    /// Calculate price impact from recent trades
    fn calculate_price_impact(&self) -> f64 {
        if self.recent_trades.len() < 2 {
            return 0.0;
        }

        let latest_trade = self.recent_trades.back().unwrap();
        let previous_trade = &self.recent_trades[self.recent_trades.len() - 2];
        
        (latest_trade.price - previous_trade.price).abs() / previous_trade.price
    }

    /// Detect adverse selection conditions
    fn detect_adverse_selection(&mut self) -> bool {
        // Check if we're in cooldown period
        if let Some(last_detection) = self.last_adverse_detection {
            if let Ok(elapsed) = SystemTime::now().duration_since(last_detection) {
                if elapsed.as_secs() < self.config.cooldown_period {
                    return false;
                }
            }
        }

        // Check order imbalance
        let imbalance = self.calculate_order_imbalance();
        let abnormal_size = self.detect_abnormal_trade_size();
        let price_impact = self.calculate_price_impact();

        // Update market state
        if imbalance.abs() > self.config.imbalance_threshold {
            if imbalance > 0.0 {
                self.market_state = MarketState::BuyerInformed;
            } else {
                self.market_state = MarketState::SellerInformed;
            }
        } else if price_impact > self.config.price_impact_threshold * 2.0 {
            self.market_state = MarketState::HighVolatility;
        } else {
            self.market_state = MarketState::Normal;
        }

        // Detect adverse selection if multiple conditions are met
        let is_adverse = (imbalance.abs() > self.config.imbalance_threshold && 
                         price_impact > self.config.price_impact_threshold) || 
                        (abnormal_size && price_impact > self.config.price_impact_threshold);
        
        if is_adverse {
            self.last_adverse_detection = Some(SystemTime::now());
        }
        
        is_adverse
    }

    /// Generate a trading signal based on adverse selection detection
    fn generate_signal(&mut self) -> Option<StrategySignal> {
        if self.recent_trades.is_empty() || self.recent_order_books.is_empty() {
            return None;
        }

        let current_price = self.recent_trades.back().unwrap().price;
        
        // Check for position management (stop loss/take profit)
        if let Some(ref_price) = self.reference_price {
            if self.position.size > 0.0 {
                // Long position management
                let pnl_pct = (current_price - ref_price) / ref_price;
                
                if pnl_pct <= -self.config.stop_loss_pct {
                    println!("Stop loss triggered for long position");
                    return Some(StrategySignal::Sell { 
                        price: current_price,
                        size: self.position.size,
                        order_type: OrderType::Market,
                        reason: "Stop loss".to_string()
                    });
                } else if pnl_pct >= self.config.take_profit_pct {
                    println!("Take profit triggered for long position");
                    return Some(StrategySignal::Sell { 
                        price: current_price,
                        size: self.position.size,
                        order_type: OrderType::Market,
                        reason: "Take profit".to_string()
                    });
                }
            } else if self.position.size < 0.0 {
                // Short position management
                let pnl_pct = (ref_price - current_price) / ref_price;
                
                if pnl_pct <= -self.config.stop_loss_pct {
                    println!("Stop loss triggered for short position");
                    return Some(StrategySignal::Buy { 
                        price: current_price,
                        size: -self.position.size,
                        order_type: OrderType::Market,
                        reason: "Stop loss".to_string()
                    });
                } else if pnl_pct >= self.config.take_profit_pct {
                    println!("Take profit triggered for short position");
                    return Some(StrategySignal::Buy { 
                        price: current_price,
                        size: -self.position.size,
                        order_type: OrderType::Market,
                        reason: "Take profit".to_string()
                    });
                }
            }
        }

        // Check for adverse selection
        if self.detect_adverse_selection() {
            self.last_adverse_detection = Some(SystemTime::now());
            
            // Calculate order imbalance to determine direction
            let imbalance = self.calculate_order_imbalance();
            
            if imbalance > 0.0 {
                // Positive imbalance suggests buying pressure, potentially from informed traders
                if self.position.size > 0.0 {
                    // If we have a long position, close it to avoid adverse selection
                    println!("Adverse selection detected: Closing long position due to potential informed buying");
                    return Some(StrategySignal::Sell { 
                        price: current_price,
                        size: self.position.size,
                        order_type: OrderType::Market,
                        reason: "Adverse selection protection".to_string()
                    });
                } else if self.position.size == 0.0 {
                    // If no position, consider following the informed traders
                    println!("Adverse selection detected: Following potential informed buying");
                    return Some(StrategySignal::Buy { 
                        price: current_price,
                        size: self.config.max_position_size,
                        order_type: OrderType::Limit,
                        reason: "Following informed flow".to_string()
                    });
                }
            } else {
                // Negative imbalance suggests selling pressure, potentially from informed traders
                if self.position.size < 0.0 {
                    // If we have a short position, close it to avoid adverse selection
                    println!("Adverse selection detected: Closing short position due to potential informed selling");
                    return Some(StrategySignal::Buy { 
                        price: current_price,
                        size: -self.position.size,
                        order_type: OrderType::Market,
                        reason: "Adverse selection protection".to_string()
                    });
                } else if self.position.size == 0.0 {
                    // If no position, consider following the informed traders
                    println!("Adverse selection detected: Following potential informed selling");
                    return Some(StrategySignal::Sell { 
                        price: current_price,
                        size: self.config.max_position_size,
                        order_type: OrderType::Limit,
                        reason: "Following informed flow".to_string()
                    });
                }
            }
        }
        
        None
    }

    /// Update position based on executed order
    fn update_position(&mut self, order: &Order) {
        let current_price = order.price.unwrap_or(0.0);
        let quantity = order.quantity as f64;
        
        match order.side {
            Side::Buy => {
                // Update position for buy order
                if self.position.size < 0.0 {
                    // Reducing short position
                    self.position.size += quantity;
                    if self.position.size >= 0.0 {
                        // Position flipped from short to long or flat
                        self.reference_price = Some(current_price);
                    }
                } else {
                    // Increasing long position or opening new long
                    self.position.size += quantity;
                    // Update reference price with weighted average
                    let old_value = self.position.size - quantity;
                    let new_value = quantity;
                    let total = self.position.size;
                    
                    self.reference_price = Some(match self.reference_price {
                        Some(ref_price) => (ref_price * old_value + current_price * new_value) / total,
                        None => current_price
                    });
                }
            },
            Side::Sell => {
                // Update position for sell order
                if self.position.size > 0.0 {
                    // Reducing long position
                    self.position.size -= quantity;
                    if self.position.size <= 0.0 {
                        // Position flipped from long to short or flat
                        self.reference_price = Some(current_price);
                    }
                } else {
                    // Increasing short position or opening new short
                    self.position.size -= quantity;
                    // Update reference price with weighted average
                    let old_value = -self.position.size - quantity;
                    let new_value = quantity;
                    let total = -self.position.size;
                    
                    self.reference_price = Some(match self.reference_price {
                        Some(ref_price) => (ref_price * old_value + current_price * new_value) / total,
                        None => current_price
                    });
                }
            }
        }
        
        println!("Position updated: size={}, reference_price={:?}", 
               self.position.size, self.reference_price);
    }

    /// Get current market state
    pub fn get_market_state(&self) -> &MarketState {
        &self.market_state
    }
}

impl Strategy for AdverseSelectionStrategy {
    fn name(&self) -> &str {
        "Adverse Selection Strategy"
    }

    fn description(&self) -> &str {
        "A strategy that monitors and adjusts order executions to avoid being exploited by traders with superior information"
    }

    fn state(&self) -> &StrategyState {
        &self.state
    }

    fn set_state(&mut self, state: StrategyState) {
        self.state = state;
    }

    fn on_market_data(&mut self, data: &MarketData) -> Option<StrategySignal> {
        match data {
            MarketData::Trade(trade) => {
                // Add trade to recent trades queue
                self.recent_trades.push_back(trade.clone());
                if self.recent_trades.len() > 100 {
                    self.recent_trades.pop_front();
                }
            },
            MarketData::OrderBook(order_book) => {
                // Add order book to recent order books queue
                self.recent_order_books.push_back(order_book.clone());
                if self.recent_order_books.len() > self.config.window_size {
                    self.recent_order_books.pop_front();
                }
            },
            _ => {}
        }

        // Generate signal based on updated data
        self.generate_signal()
    }

    fn on_order_executed(&mut self, order: &Order) {
        println!("Order executed: {:?}", order);
        self.update_position(order);
    }

    fn on_order_cancelled(&mut self, order: &Order) {
        println!("Order cancelled: {:?}", order);
    }

    fn reset(&mut self) {
        self.state = StrategyState::Idle;
        self.recent_trades.clear();
        self.recent_order_books.clear();
        self.last_adverse_detection = None;
        self.position = Position::default();
        self.reference_price = None;
        self.market_state = MarketState::Normal;
    }
}

/// Implement order splitting strategy
impl OrderSplitStrategy for AdverseSelectionStrategy {
    fn split(&self, parent_order: &ParentOrder) -> Vec<ChildOrder> {
        let mut child_orders = Vec::new();
        let mut rng = rand::thread_rng();
        
        // Determine split strategy based on market state
        let (num_splits, base_interval_ms) = match self.market_state {
            MarketState::Normal => {
                // Normal market state, use medium split count and interval
                (self.config.max_splits / 2, 
                 (self.config.min_split_interval_ms + self.config.max_split_interval_ms) / 2)
            },
            MarketState::BuyerInformed => {
                if parent_order.order_common.side == Side::Buy {
                    // Buyer informed, buy orders use more splits and longer interval
                    (self.config.max_splits, self.config.max_split_interval_ms)
                } else {
                    // Sell orders use fewer splits and shorter interval
                    (self.config.max_splits / 3, self.config.min_split_interval_ms)
                }
            },
            MarketState::SellerInformed => {
                if parent_order.order_common.side == Side::Sell {
                    // Seller informed, sell orders use more splits and longer interval
                    (self.config.max_splits, self.config.max_split_interval_ms)
                } else {
                    // Buy orders use fewer splits and shorter interval
                    (self.config.max_splits / 3, self.config.min_split_interval_ms)
                }
            },
            MarketState::HighVolatility => {
                // High volatility state, use maximum splits and shortest interval
                (self.config.max_splits, self.config.min_split_interval_ms)
            }
        };
        
        // Calculate base size for each child order
        let base_quantity = parent_order.order_common.quantity / num_splits as u32;
        let mut remaining_quantity = parent_order.order_common.quantity;
        
        // Create child orders
        for i in 0..num_splits {
            // Add some variation to child order size, except for the last order
            let quantity = if i < num_splits - 1 {
                // Use random variation based on configuration
                let variation_factor = 1.0 + self.config.size_variation_pct * (rng.gen::<f64>() * 2.0 - 1.0);
                let quantity = (base_quantity as f64 * variation_factor).max(1.0) as u32;
                quantity.min(remaining_quantity) // Ensure does not exceed remaining quantity
            } else {
                // Last order uses all remaining quantity
                remaining_quantity
            };
            
            // Update remaining quantity
            remaining_quantity = remaining_quantity.saturating_sub(quantity);
            
            // Calculate execution time for child order
            let interval_variation = (rng.gen::<f64>() * 0.4 - 0.2) * base_interval_ms as f64;
            let interval_ms = (base_interval_ms as f64 * (1.0 + i as f64 * 0.2) + interval_variation) as u64;
            
            // Get current time in milliseconds since UNIX epoch
            let now = SystemTime::now();
            let execution_time_millis = match now.duration_since(SystemTime::UNIX_EPOCH) {
                Ok(duration) => duration.as_millis() as u64 + interval_ms,
                Err(_) => interval_ms, // Fallback if system time is before UNIX epoch
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
                insert_at: Some(execution_time_millis),
            };
            
            child_orders.push(child_order);
        }
        
        child_orders
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::orders::{Order, ProductType, OrderType as ModelOrderType, Side, TimeInForce};
    use std::time::UNIX_EPOCH;

    #[test]
    fn test_strategy_initialization() {
        let config = AdverseSelectionConfig::default();
        let strategy = AdverseSelectionStrategy::new(config);
        
        assert_eq!(strategy.state, StrategyState::Idle);
        assert_eq!(strategy.recent_trades.len(), 0);
        assert_eq!(strategy.recent_order_books.len(), 0);
        assert_eq!(strategy.position.size, 0.0);
        assert!(strategy.reference_price.is_none());
    }

    #[test]
    fn test_order_imbalance_calculation() {
        let config = AdverseSelectionConfig::default();
        let mut strategy = AdverseSelectionStrategy::new(config);
        
        // Create two order books with imbalance
        let mut order_book1 = OrderBook::default();
        order_book1.bids.push((100.0, 10.0));
        order_book1.asks.push((101.0, 10.0));
        
        let mut order_book2 = OrderBook::default();
        order_book2.bids.push((100.0, 15.0)); // Increased bid volume
        order_book2.asks.push((101.0, 8.0));  // Decreased ask volume
        
        strategy.recent_order_books.push_back(order_book1);
        strategy.recent_order_books.push_back(order_book2);
        
        let imbalance = strategy.calculate_order_imbalance();
        assert!(imbalance > 0.0); // Should be positive due to increased buying pressure
    }

    #[test]
    fn test_abnormal_trade_size_detection() {
        let config = AdverseSelectionConfig::default();
        let mut strategy = AdverseSelectionStrategy::new(config);
        
        // Add 10 normal-sized trades
        for i in 0..10 {
            let trade = Trade {
                timestamp: SystemTime::now(),
                price: 100.0 + (i as f64 * 0.1),
                size: 1.0,
                side: if i % 2 == 0 { Side::Buy } else { Side::Sell },
            };
            strategy.recent_trades.push_back(trade);
        }
        
        // No abnormal trade yet
        assert!(!strategy.detect_abnormal_trade_size());
        
        // Add an abnormally large trade
        let large_trade = Trade {
            timestamp: SystemTime::now(),
            price: 101.0,
            size: 5.0, // 5x the average size
            side: Side::Buy, // Use a fixed side instead of random for testing
        };
        strategy.recent_trades.push_back(large_trade);
        
        // Should detect the abnormal trade
        assert!(strategy.detect_abnormal_trade_size());
    }

    #[test]
    fn test_position_update() {
        let config = AdverseSelectionConfig::default();
        let mut strategy = AdverseSelectionStrategy::new(config);
        
        // Test buy order
        let buy_order = Order::new(
            "order1".to_string(),
            100, // quantity
            ProductType::Spot,
            ModelOrderType::Market,
            Some(100.0), // price
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64, // timestamp
            None, // expiry_date
            "BTC/USD".to_string(), // symbol
            Side::Buy, // side
            "USD".to_string(), // currency
            Some("BINANCE".to_string()), // exchange
            Some(TimeInForce::GTC), // timeinforce
            None, None, None, None, None, None // optional fields
        );
        
        strategy.update_position(&buy_order);
        assert_eq!(strategy.position.size, 100.0);
        assert!(strategy.reference_price.is_some());
        
        // Test sell order that reduces position
        let sell_order = Order::new(
            "order2".to_string(),
            50, // quantity
            ProductType::Spot,
            ModelOrderType::Market,
            Some(110.0), // price
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64, // timestamp
            None, // expiry_date
            "BTC/USD".to_string(), // symbol
            Side::Sell, // side
            "USD".to_string(), // currency
            Some("BINANCE".to_string()), // exchange
            Some(TimeInForce::GTC), // timeinforce
            None, None, None, None, None, None // optional fields
        );
        
        strategy.update_position(&sell_order);
        assert_eq!(strategy.position.size, 50.0);
        
        // Test sell order that flips position to short
        let sell_order2 = Order::new(
            "order3".to_string(),
            100, // quantity
            ProductType::Spot,
            ModelOrderType::Market,
            Some(105.0), // price
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64, // timestamp
            None, // expiry_date
            "BTC/USD".to_string(), // symbol
            Side::Sell, // side
            "USD".to_string(), // currency
            Some("BINANCE".to_string()), // exchange
            Some(TimeInForce::GTC), // timeinforce
            None, None, None, None, None, None // optional fields
        );
        
        strategy.update_position(&sell_order2);
        assert_eq!(strategy.position.size, -50.0);
        assert!(strategy.reference_price.is_some());
    }

    #[test]
    fn test_split_order_normal_market() {
        let config = AdverseSelectionConfig::default();
        let mut strategy = AdverseSelectionStrategy::new(config.clone());
        
        // Set normal market state
        strategy.market_state = MarketState::Normal;
        
        // Create parent order with the correct structure
        let order = Order::new(
            "test-order-123".to_string(),
            1000, // quantity
            ProductType::Spot,
            ModelOrderType::Market,
            Some(50000.0), // price
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64, // timestamp
            None, // expiry_date
            "BTC/USD".to_string(), // symbol
            Side::Buy, // side
            "USD".to_string(), // currency
            Some("BINANCE".to_string()), // exchange
            Some(TimeInForce::GTC), // timeinforce
            None, None, None, None, None, None // optional fields
        );
        
        let parent_order = ParentOrder {
            order_common: order,
            strategy_id: "TWAP".to_string(),
        };
        
        // Split order
        let child_orders = strategy.split(&parent_order);
        
        // Verify split count
        assert_eq!(child_orders.len(), config.max_splits / 2);
        
        // Verify total quantity
        let total_quantity: u32 = child_orders.iter().map(|o| o.order_common.quantity).sum();
        assert_eq!(total_quantity, parent_order.order_common.quantity);
    }
}
