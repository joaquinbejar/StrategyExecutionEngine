/*
布林带策略 (Bollinger Bands)
布林带由三条线组成：中轨（通常是20日移动平均线）、上轨（中轨加上两倍标准差）和
下轨（中轨减去两倍标准差）。该策略基于价格回归均值的原理，当价格触及下轨时买入，
触及上轨时卖出。

例如：当价格触及下轨并开始回升时买入，当价格触及上轨并开始回落时卖出。

参考文献：
"Bollinger on Bollinger Bands" by John Bollinger：这本书由布林带的创始人撰写，
详细介绍了布林带的使用方法和策略。(第4章详细讨论了布林带交易策略)
*/

use std::collections::VecDeque;
use crate::models::orders::Side;
use crate::models::child_orders::ChildOrder;
use crate::models::parent_orders::ParentOrder;
use crate::strategies::common_strategies::OrderSplitStrategy;

pub struct BollingerBandsStrategy {
    period: usize,
    std_dev_multiplier: f64,
    prices: VecDeque<f64>,
}

impl BollingerBandsStrategy {
    pub fn new(period: usize, std_dev_multiplier: f64) -> Self {
        BollingerBandsStrategy {
            period,
            std_dev_multiplier,
            prices: VecDeque::with_capacity(period),
        }
    }
    
    pub fn add_price(&mut self, price: f64) {
        self.prices.push_back(price);
        if self.prices.len() > self.period {
            self.prices.pop_front();
        }
    }
    
    pub fn calculate_bands(&self) -> Option<(f64, f64, f64)> {
        if self.prices.len() < self.period {
            return None;
        }
        
        // 计算中轨（SMA）
        let middle_band: f64 = self.prices.iter().sum::<f64>() / self.period as f64;
        
        // 计算标准差
        let variance: f64 = self.prices.iter()
            .map(|&price| (price - middle_band).powi(2))
            .sum::<f64>() / self.period as f64;
        let std_dev = variance.sqrt();
        
        // 计算上下轨
        let upper_band = middle_band + (self.std_dev_multiplier * std_dev);
        let lower_band = middle_band - (self.std_dev_multiplier * std_dev);
        
        Some((lower_band, middle_band, upper_band))
    }
    
    pub fn get_signal(&self) -> Option<Side> {
        if self.prices.is_empty() {
            return None;
        }
        
        let current_price = *self.prices.back().unwrap();
        let bands = self.calculate_bands()?;
        
        if current_price <= bands.0 {  // 价格触及下轨
            Some(Side::Buy)
        } else if current_price >= bands.2 {  // 价格触及上轨
            Some(Side::Sell)
        } else {
            None
        }
    }
}

impl OrderSplitStrategy for BollingerBandsStrategy {
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
    fn test_bollinger_bands_initialization() {
        let strategy = BollingerBandsStrategy::new(20, 2.0);
        assert_eq!(strategy.period, 20);
        assert_eq!(strategy.std_dev_multiplier, 2.0);
        assert_eq!(strategy.prices.len(), 0);
    }

    #[test]
    fn test_add_price() {
        let mut strategy = BollingerBandsStrategy::new(5, 2.0);
        
        // 添加价格
        for i in 1..=10 {
            strategy.add_price(i as f64);
        }
        
        // 检查价格队列长度不超过周期
        assert_eq!(strategy.prices.len(), 5);
        
        // 检查价格队列内容（应该是最后5个价格）
        let expected_prices: Vec<f64> = (6..=10).map(|i| i as f64).collect();
        let actual_prices: Vec<f64> = strategy.prices.iter().cloned().collect();
        assert_eq!(actual_prices, expected_prices);
    }

    #[test]
    fn test_calculate_bands() {
        let mut strategy = BollingerBandsStrategy::new(5, 2.0);
        
        // 添加相同的价格
        for _ in 0..5 {
            strategy.add_price(100.0);
        }
        
        // 计算布林带
        let bands = strategy.calculate_bands();
        assert!(bands.is_some());
        let (lower, middle, upper) = bands.unwrap();
        
        // 所有价格相同时，标准差为0，上下轨等于中轨
        assert_eq!(lower, 100.0);
        assert_eq!(middle, 100.0);
        assert_eq!(upper, 100.0);
        
        // 添加不同的价格
        strategy.add_price(110.0);
        strategy.add_price(90.0);
        strategy.add_price(120.0);
        strategy.add_price(80.0);
        strategy.add_price(100.0);
        
        // 重新计算布林带
        let bands = strategy.calculate_bands();
        assert!(bands.is_some());
        let (lower, middle, upper) = bands.unwrap();
        
        // 验证中轨是平均值
        assert_eq!(middle, 100.0);
        
        // 验证上下轨与中轨的距离
        assert!(upper > middle);
        assert!(lower < middle);
        assert!((upper - middle).abs() > 0.0);
        assert!((middle - lower).abs() > 0.0);
    }

    #[test]
    fn test_buy_signal_generation() {
        let mut strategy = BollingerBandsStrategy::new(5, 2.0);
        
        // 添加稳定价格
        for _ in 0..5 {
            strategy.add_price(100.0);
        }
        
        // 添加低于下轨的价格
        strategy.add_price(80.0);
        
        // 此时应该有买入信号
        let signal = strategy.get_signal();
        assert!(signal.is_some());
        assert_eq!(signal.unwrap(), Side::Buy);
    }

    #[test]
    fn test_sell_signal_generation() {
        let mut strategy = BollingerBandsStrategy::new(5, 2.0);
        
        // 添加稳定价格
        for _ in 0..5 {
            strategy.add_price(100.0);
        }
        
        // 添加高于上轨的价格
        strategy.add_price(120.0);
        
        // 此时应该有卖出信号
        let signal = strategy.get_signal();
        assert!(signal.is_some());
        assert_eq!(signal.unwrap(), Side::Sell);
    }

    #[test]
    fn test_order_split_with_matching_signal() {
        let mut strategy = BollingerBandsStrategy::new(5, 2.0);
        
        // 设置产生买入信号
        for _ in 0..5 {
            strategy.add_price(100.0);
        }
        strategy.add_price(80.0);
        
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
            strategy_id: "bollinger_strategy".to_string(),
        };
        
        // 分割订单
        let child_orders = strategy.split(&parent_order);
        
        // 验证生成了子订单
        assert_eq!(child_orders.len(), 1);
        assert_eq!(child_orders[0].parent_id, "test_id");
        assert_eq!(child_orders[0].strategy_id, "bollinger_strategy");
    }

    #[test]
    fn test_order_split_with_non_matching_signal() {
        let mut strategy = BollingerBandsStrategy::new(5, 2.0);
        
        // 设置产生买入信号
        for _ in 0..5 {
            strategy.add_price(100.0);
        }
        strategy.add_price(80.0);
        
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
            strategy_id: "bollinger_strategy".to_string(),
        };
        
        // 分割订单
        let child_orders = strategy.split(&parent_order);
        
        // 验证没有生成子订单
        assert_eq!(child_orders.len(), 0);
    }
}