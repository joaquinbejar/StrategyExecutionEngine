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

use crate::models::child_orders::ChildOrder;
use crate::models::parent_orders::ParentOrder;
use crate::models::orders::Side as OrderSide;
use crate::strategies::common_strategies::OrderSplitStrategy;
use std::collections::VecDeque;

/// Represents a candlestick with OHLC values
#[derive(Debug, Clone, Copy)]
pub struct Candle {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

/// Heikin-Ashi strategy implementation
pub struct HeikinAshiStrategy {
    /// Window size for the strategy
    window_size: usize,
    /// Historical candles
    candles: VecDeque<Candle>,
    /// Historical Heikin-Ashi candles
    ha_candles: VecDeque<Candle>,
}

impl HeikinAshiStrategy {
    /// Creates a new Heikin-Ashi strategy with the specified window size
    pub fn new(window_size: usize) -> Self {
        HeikinAshiStrategy {
            window_size,
            candles: VecDeque::with_capacity(window_size),
            ha_candles: VecDeque::with_capacity(window_size),
        }
    }

    /// Adds a new candle to the strategy and calculates the corresponding Heikin-Ashi candle
    pub fn add_candle(&mut self, candle: Candle) {
        // Add the new candle to the history
        self.candles.push_back(candle);
        
        // Calculate the Heikin-Ashi candle
        let ha_candle = if self.ha_candles.is_empty() {
            // First Heikin-Ashi candle is the same as the regular candle
            candle
        } else {
            let prev_ha = self.ha_candles.back().unwrap();
            
            // Heikin-Ashi formulas
            let ha_open = (prev_ha.open + prev_ha.close) / 2.0;
            let ha_close = (candle.open + candle.high + candle.low + candle.close) / 4.0;
            let ha_high = candle.high.max(ha_open).max(ha_close);
            let ha_low = candle.low.min(ha_open).min(ha_close);
            
            Candle {
                open: ha_open,
                high: ha_high,
                low: ha_low,
                close: ha_close,
                volume: candle.volume,
            }
        };
        
        // Add the Heikin-Ashi candle to the history
        self.ha_candles.push_back(ha_candle);
        
        // Maintain the window size
        if self.candles.len() > self.window_size {
            self.candles.pop_front();
        }
        
        if self.ha_candles.len() > self.window_size {
            self.ha_candles.pop_front();
        }
    }

    /// Determines the trading signal based on Heikin-Ashi patterns
    pub fn get_signal(&self) -> Option<OrderSide> {
        if self.ha_candles.len() < 3 {
            return None; // Need at least 3 candles to generate a signal
        }
        
        let candles: Vec<&Candle> = self.ha_candles.iter().collect();
        let len = candles.len();
        
        let current = candles[len - 1];
        let prev = candles[len - 2];
        let prev_prev = candles[len - 3];
        
        // Bullish signal: Three consecutive green candles with no lower shadows
        let bullish_signal = 
            current.close > current.open && 
            prev.close > prev.open && 
            prev_prev.close > prev_prev.open;
        
        // Bearish signal: Three consecutive red candles with no upper shadows
        let bearish_signal = 
            current.close < current.open && 
            prev.close < prev.open && 
            prev_prev.close < prev_prev.open;
        
        if bullish_signal {
            Some(OrderSide::Buy)
        } else if bearish_signal {
            Some(OrderSide::Sell)
        } else {
            None
        }
    }
}

impl OrderSplitStrategy for HeikinAshiStrategy {
    fn split(&self, parent_order: &ParentOrder) -> Vec<ChildOrder> {
        // Get the current signal
        let signal = self.get_signal();
        
        // If there's no signal, return empty
        if signal.is_none() {
            return Vec::new();
        }
        
        // 使用模式匹配而不是 != 运算符
        match (signal.unwrap(), &parent_order.order_common.side) {
            (OrderSide::Buy, OrderSide::Buy) | (OrderSide::Sell, OrderSide::Sell) => {
                // Signal matches parent order side, create a child order
                vec![ChildOrder {
                    order_common: parent_order.order_common.clone(),
                    strategy_id: parent_order.strategy_id.clone(),
                    parent_id: parent_order.order_common.id.clone(),
                    insert_at: Some(parent_order.order_common.timestamp),
                }]
            },
            _ => {
                // Signal doesn't match parent order side
                Vec::new()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heikin_ashi_calculation() {
        let mut strategy = HeikinAshiStrategy::new(5);
        
        // Add some test candles
        strategy.add_candle(Candle {
            open: 100.0,
            high: 110.0,
            low: 95.0,
            close: 105.0,
            volume: 1000.0,
        });
        
        // First HA candle should be the same as the regular candle
        let first_ha = strategy.ha_candles.back().unwrap();
        assert_eq!(first_ha.open, 100.0);
        assert_eq!(first_ha.high, 110.0);
        assert_eq!(first_ha.low, 95.0);
        assert_eq!(first_ha.close, 105.0);
        
        // Add another candle
        strategy.add_candle(Candle {
            open: 105.0,
            high: 115.0,
            low: 100.0,
            close: 110.0,
            volume: 1200.0,
        });
        
        // Second HA candle should be calculated using the formula
        let second_ha = strategy.ha_candles.back().unwrap();
        let expected_open = (100.0 + 105.0) / 2.0;
        let expected_close = (105.0 + 115.0 + 100.0 + 110.0) / 4.0;
        
        assert_eq!(second_ha.open, expected_open);
        assert_eq!(second_ha.close, expected_close);
    }

    #[test]
    fn test_signal_generation() {
        let mut strategy = HeikinAshiStrategy::new(5);
        
        // Not enough candles for a signal
        assert!(strategy.get_signal().is_none());
        
        // 添加第一根蜡烛线
        strategy.add_candle(Candle {
            open: 100.0,
            high: 110.0,
            low: 90.0,
            close: 110.0,
            volume: 1000.0,
        });
        
        // 添加第二根蜡烛线
        strategy.add_candle(Candle {
            open: 110.0,
            high: 120.0,
            low: 100.0,
            close: 120.0,
            volume: 1000.0,
        });
        
        // 添加第三根蜡烛线
        strategy.add_candle(Candle {
            open: 120.0,
            high: 130.0,
            low: 110.0,
            close: 130.0,
            volume: 1000.0,
        });
        
        // 现在应该有买入信号
        let signal = strategy.get_signal();
        assert!(signal.is_some());
        match signal {
            Some(OrderSide::Buy) => {}, // Expected
            _ => panic!("Expected Buy signal, got {:?}", signal),
        }
        
        // 重置策略
        let mut strategy = HeikinAshiStrategy::new(5);
        
        // 添加第一根蜡烛线
        strategy.add_candle(Candle {
            open: 100.0,
            high: 110.0,
            low: 90.0,
            close: 90.0,
            volume: 1000.0,
        });
        
        // 添加第二根蜡烛线
        strategy.add_candle(Candle {
            open: 90.0,
            high: 100.0,
            low: 80.0,
            close: 80.0,
            volume: 1000.0,
        });
        
        // 添加第三根蜡烛线
        strategy.add_candle(Candle {
            open: 80.0,
            high: 90.0,
            low: 70.0,
            close: 70.0,
            volume: 1000.0,
        });
        
        // 现在应该有卖出信号
        let signal = strategy.get_signal();
        assert!(signal.is_some());
        match signal {
            Some(OrderSide::Sell) => {}, // Expected
            _ => panic!("Expected Sell signal, got {:?}", signal),
        }
    }
}
