/*
相对强弱指数 (RSI - Relative Strength Index)
RSI是一种动量振荡指标，用于衡量价格变动的速度和变化。RSI的取值范围在0到100之间，
通常RSI高于70被认为是超买状态，低于30被认为是超卖状态。该策略在RSI处于超卖状态时
买入，在超买状态时卖出。

例如：当RSI跌破30并回升时买入，当RSI突破70并回落时卖出。

参考文献：
"New Concepts in Technical Trading Systems" by J. Welles Wilder：这本书首次
介绍了RSI指标及其应用。(第3章详细讨论了RSI的计算和使用)
*/

use std::collections::VecDeque;
use crate::models::orders::Side;
use crate::models::child_orders::ChildOrder;
use crate::models::parent_orders::ParentOrder;
use crate::strategies::common_strategies::OrderSplitStrategy;

pub struct RSIStrategy {
    period: usize,
    prices: VecDeque<f64>,
    gains: VecDeque<f64>,
    losses: VecDeque<f64>,
    overbought_threshold: f64,
    oversold_threshold: f64,
}

impl RSIStrategy {
    pub fn new(period: usize, overbought_threshold: f64, oversold_threshold: f64) -> Self {
        RSIStrategy {
            period,
            prices: VecDeque::with_capacity(period + 1),
            gains: VecDeque::with_capacity(period),
            losses: VecDeque::with_capacity(period),
            overbought_threshold,
            oversold_threshold,
        }
    }
    
    pub fn add_price(&mut self, price: f64) {
        if !self.prices.is_empty() {
            let prev_price = *self.prices.back().unwrap();
            let change = price - prev_price;
            
            if change > 0.0 {
                self.gains.push_back(change);
                self.losses.push_back(0.0);
            } else {
                self.gains.push_back(0.0);
                self.losses.push_back(-change);
            }
            
            if self.gains.len() > self.period {
                self.gains.pop_front();
                self.losses.pop_front();
            }
        }
        
        self.prices.push_back(price);
        if self.prices.len() > self.period + 1 {
            self.prices.pop_front();
        }
    }
    
    pub fn calculate_rsi(&self) -> Option<f64> {
        if self.gains.len() < self.period {
            return None;
        }
        
        let avg_gain: f64 = self.gains.iter().sum::<f64>() / self.period as f64;
        let avg_loss: f64 = self.losses.iter().sum::<f64>() / self.period as f64;
        
        if avg_loss == 0.0 {
            return Some(100.0);
        }
        
        let rs = avg_gain / avg_loss;
        let rsi = 100.0 - (100.0 / (1.0 + rs));
        
        Some(rsi)
    }
    
    pub fn get_signal(&self) -> Option<Side> {
        let rsi = self.calculate_rsi()?;
        
        if rsi < self.oversold_threshold {
            Some(Side::Buy)
        } else if rsi > self.overbought_threshold {
            Some(Side::Sell)
        } else {
            None
        }
    }
}

impl OrderSplitStrategy for RSIStrategy {
    fn split(&self, parent_order: &ParentOrder) -> Vec<ChildOrder> {
        let signal = self.get_signal();
        if signal.is_none() {
            return Vec::new();
        }
        
        match (signal.unwrap(), &parent_order.order_common.side) {
            (Side::Buy, Side::Buy) | (Side::Sell, Side::Sell) => {
                vec![ChildOrder {
                    order_common: parent_order.order_common.clone(),
                    strategy_id: parent_order.strategy_id.clone(),
                    parent_id: parent_order.order_common.id.clone(),
                    insert_at: Some(parent_order.order_common.timestamp),
                }]
            },
            _ => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::orders::{Order, OrderType, ProductType, Side, TimeInForce};
    use crate::models::parent_orders::ParentOrder;

    #[test]
    fn test_rsi_strategy_initialization() {
        let strategy = RSIStrategy::new(14, 70.0, 30.0);
        assert_eq!(strategy.period, 14);
        assert_eq!(strategy.overbought_threshold, 70.0);
        assert_eq!(strategy.oversold_threshold, 30.0);
        assert_eq!(strategy.prices.len(), 0);
        assert_eq!(strategy.gains.len(), 0);
        assert_eq!(strategy.losses.len(), 0);
    }

    #[test]
    fn test_add_price() {
        let mut strategy = RSIStrategy::new(5, 70.0, 30.0);
        
        // 添加第一个价格
        strategy.add_price(100.0);
        assert_eq!(strategy.prices.len(), 1);
        assert_eq!(strategy.gains.len(), 0);
        assert_eq!(strategy.losses.len(), 0);
        
        // 添加上升价格
        strategy.add_price(110.0);
        assert_eq!(strategy.prices.len(), 2);
        assert_eq!(strategy.gains.len(), 1);
        assert_eq!(strategy.losses.len(), 1);
        assert_eq!(strategy.gains[0], 10.0);
        assert_eq!(strategy.losses[0], 0.0);
        
        // 添加下降价格
        strategy.add_price(100.0);
        assert_eq!(strategy.prices.len(), 3);
        assert_eq!(strategy.gains.len(), 2);
        assert_eq!(strategy.losses.len(), 2);
        assert_eq!(strategy.gains[1], 0.0);
        assert_eq!(strategy.losses[1], 10.0);
    }

    #[test]
    fn test_calculate_rsi() {
        let mut strategy = RSIStrategy::new(5, 70.0, 30.0);
        
        // 添加价格
        strategy.add_price(100.0);
        assert!(strategy.calculate_rsi().is_none());
        
        // 添加连续上涨的价格
        strategy.add_price(110.0);
        strategy.add_price(120.0);
        strategy.add_price(130.0);
        strategy.add_price(140.0);
        strategy.add_price(150.0);
        
        // 现在应该能计算RSI
        let rsi = strategy.calculate_rsi();
        assert!(rsi.is_some());
        assert_eq!(rsi.unwrap(), 100.0); // 全是上涨，RSI = 100
        
        // 添加连续下跌的价格
        strategy.add_price(140.0);
        strategy.add_price(130.0);
        strategy.add_price(120.0);
        strategy.add_price(110.0);
        strategy.add_price(100.0);
        
        // 现在RSI应该很低
        let rsi = strategy.calculate_rsi();
        assert!(rsi.is_some());
        assert!(rsi.unwrap() < 50.0);
    }

    #[test]
    fn test_buy_signal_generation() {
        let mut strategy = RSIStrategy::new(5, 70.0, 30.0);
        
        // 添加价格使RSI低于30
        strategy.add_price(100.0);
        strategy.add_price(95.0);
        strategy.add_price(90.0);
        strategy.add_price(85.0);
        strategy.add_price(80.0);
        strategy.add_price(75.0);
        
        // 此时应该有买入信号
        let signal = strategy.get_signal();
        assert!(signal.is_some());
        assert_eq!(signal.unwrap(), Side::Buy);
    }

    #[test]
    fn test_sell_signal_generation() {
        let mut strategy = RSIStrategy::new(5, 70.0, 30.0);
        
        // 添加价格使RSI高于70
        strategy.add_price(100.0);
        strategy.add_price(110.0);
        strategy.add_price(120.0);
        strategy.add_price(130.0);
        strategy.add_price(140.0);
        strategy.add_price(150.0);
        
        // 此时应该有卖出信号
        let signal = strategy.get_signal();
        assert!(signal.is_some());
        assert_eq!(signal.unwrap(), Side::Sell);
    }

    #[test]
    fn test_order_split_with_matching_signal() {
        let mut strategy = RSIStrategy::new(5, 70.0, 30.0);
        
        // 设置产生买入信号
        strategy.add_price(100.0);
        strategy.add_price(95.0);
        strategy.add_price(90.0);
        strategy.add_price(85.0);
        strategy.add_price(80.0);
        strategy.add_price(75.0);
        
        // 创建买入父订单
        let parent_order = ParentOrder {
            order_common: Order::new(
                "test_id".to_string(),
                100,
                ProductType::Spot,
                OrderType::Market,
                None,
                1234567890,
                None,
                "BTC/USD".to_string(),
                Side::Buy,
                "USD".to_string(),
                Some("Binance".to_string()),
                Some(TimeInForce::GTC),
                None,
                None,
                None,
                None,
                None,
                None,
            ),
            strategy_id: "rsi_strategy".to_string(),
        };
        
        // 分割订单
        let child_orders = strategy.split(&parent_order);
        
        // 验证生成了子订单
        assert_eq!(child_orders.len(), 1);
        assert_eq!(child_orders[0].parent_id, "test_id");
        assert_eq!(child_orders[0].strategy_id, "rsi_strategy");
    }

    #[test]
    fn test_order_split_with_non_matching_signal() {
        let mut strategy = RSIStrategy::new(5, 70.0, 30.0);
        
        // 设置产生买入信号
        strategy.add_price(100.0);
        strategy.add_price(95.0);
        strategy.add_price(90.0);
        strategy.add_price(85.0);
        strategy.add_price(80.0);
        strategy.add_price(75.0);
        
        // 创建卖出父订单（与信号不匹配）
        let parent_order = ParentOrder {
            order_common: Order::new(
                "test_id".to_string(),
                100,
                ProductType::Spot,
                OrderType::Market,
                None,
                1234567890,
                None,
                "BTC/USD".to_string(),
                Side::Sell,
                "USD".to_string(),
                Some("Binance".to_string()),
                Some(TimeInForce::GTC),
                None,
                None,
                None,
                None,
                None,
                None,
            ),
            strategy_id: "rsi_strategy".to_string(),
        };
        
        // 分割订单
        let child_orders = strategy.split(&parent_order);
        
        // 验证没有生成子订单
        assert_eq!(child_orders.len(), 0);
    }
}