/*
移动平均线交叉策略 (Moving Average Crossover)
该策略使用短期和长期移动平均线的交叉来生成交易信号。当短期移动平均线从下方穿过长期
移动平均线时产生买入信号，反之则产生卖出信号。这是一种趋势跟踪策略，旨在捕捉价格
趋势的变化。

例如：使用50日和200日移动平均线，当50日线上穿200日线时（黄金交叉）买入，
当50日线下穿200日线时（死亡交叉）卖出。

参考文献：
"Technical Analysis of the Financial Markets" by John J. Murphy：这本书详细
介绍了移动平均线及其交叉策略的应用。(第5章详细讨论了移动平均线策略)
*/

use std::collections::VecDeque;
use crate::models::orders::Side;
use crate::models::child_orders::ChildOrder;
use crate::models::parent_orders::ParentOrder;
use crate::strategies::common_strategies::OrderSplitStrategy;

pub struct MAStrategy {
    short_period: usize,
    long_period: usize,
    prices: VecDeque<f64>,
}

impl MAStrategy {
    pub fn new(short_period: usize, long_period: usize) -> Self {
        MAStrategy {
            short_period,
            long_period,
            prices: VecDeque::with_capacity(long_period + 1),
        }
    }
    
    pub fn add_price(&mut self, price: f64) {
        self.prices.push_back(price);
        if self.prices.len() > self.long_period {
            self.prices.pop_front();
        }
    }
    
    pub fn get_signal(&self) -> Option<Side> {
        // 确保有足够的数据
        if self.prices.len() < self.long_period {
            return None;
        }
        
        // 将价格转换为向量以便于处理
        let prices_vec: Vec<f64> = self.prices.iter().cloned().collect();
        
        // 计算当前的短期MA和长期MA
        let short_ma = prices_vec.iter().rev().take(self.short_period).sum::<f64>() / self.short_period as f64;
        let long_ma = prices_vec.iter().sum::<f64>() / prices_vec.len() as f64;
        
        // 如果只有一个价格点，无法计算交叉
        if prices_vec.len() <= 1 {
            return None;
        }
        
        // 创建前一个时间点的价格列表（移除最新价格）
        let prev_prices: Vec<f64> = prices_vec.iter().take(prices_vec.len() - 1).cloned().collect();
        
        // 计算前一个时间点的短期MA和长期MA
        let prev_short_ma = prev_prices.iter().rev().take(self.short_period).sum::<f64>() / self.short_period as f64;
        let prev_long_ma = prev_prices.iter().sum::<f64>() / prev_prices.len() as f64;
        
        // 打印调试信息
        println!("Current prices: {:?}", prices_vec);
        println!("Previous prices: {:?}", prev_prices);
        println!("Current short MA: {}, long MA: {}", short_ma, long_ma);
        println!("Previous short MA: {}, long MA: {}", prev_short_ma, prev_long_ma);
        
        // 检测黄金交叉（短期MA从下方穿过长期MA）
        // 修改条件：前一时刻短期MA <= 前一时刻长期MA，当前短期MA > 当前长期MA
        if prev_short_ma <= prev_long_ma && short_ma > long_ma {
            println!("Golden Cross detected - BUY signal");
            return Some(Side::Buy);
        }
        
        // 检测死亡交叉（短期MA从上方穿过长期MA）
        // 修改条件：前一时刻短期MA >= 前一时刻长期MA，当前短期MA < 当前长期MA
        if prev_short_ma >= prev_long_ma && short_ma < long_ma {
            println!("Death Cross detected - SELL signal");
            return Some(Side::Sell);
        }
        
        // 没有交叉
        println!("No cross detected - NO signal");
        None
    }
}

impl OrderSplitStrategy for MAStrategy {
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
    fn test_ma_strategy_initialization() {
        let strategy = MAStrategy::new(10, 30);
        assert_eq!(strategy.short_period, 10);
        assert_eq!(strategy.long_period, 30);
        assert_eq!(strategy.prices.len(), 0);
    }

    #[test]
    fn test_add_price() {
        let mut strategy = MAStrategy::new(5, 10);
        
        // 添加价格
        for i in 1..=15 {
            strategy.add_price(i as f64);
        }
        
        // 检查价格队列长度不超过长周期
        assert_eq!(strategy.prices.len(), 10);
        
        // 检查价格队列内容（应该是最后10个价格）
        let expected_prices: Vec<f64> = (6..=15).map(|i| i as f64).collect();
        let actual_prices: Vec<f64> = strategy.prices.iter().cloned().collect();
        assert_eq!(actual_prices, expected_prices);
    }

    #[test]
    fn test_buy_signal_generation() {
        let mut strategy = MAStrategy::new(2, 3);
        
        // 添加初始价格，使短期MA < 长期MA
        strategy.add_price(5.0);
        strategy.add_price(5.0);
        strategy.add_price(5.0);
        
        // 此时短期MA = 长期MA = 5.0，没有信号
        assert!(strategy.get_signal().is_none());
        
        // 添加一个高价格，使短期MA > 长期MA
        strategy.add_price(15.0);  // 现在价格是 [5.0, 5.0, 15.0]
                                  // 短期MA = (5+15)/2 = 10.0
                                  // 长期MA = (5+5+15)/3 = 8.33
                                  // 前一时刻短期MA = (5+5)/2 = 5.0
                                  // 前一时刻长期MA = (5+5+5)/3 = 5.0
                                  // 短期MA从5.0上升到10.0，长期MA从5.0上升到8.33
                                  // 短期MA从等于长期MA变为大于长期MA，产生买入信号
        
        // 此时短期MA > 长期MA，产生买入信号
        let signal = strategy.get_signal();
        assert!(signal.is_some(), "Expected a signal, got None");
        assert_eq!(signal.unwrap(), Side::Buy, "Expected Buy signal");
    }

    #[test]
    fn test_sell_signal_generation() {
        let mut strategy = MAStrategy::new(2, 3);
        
        // 添加初始价格，使短期MA > 长期MA
        strategy.add_price(5.0);
        strategy.add_price(5.0);
        strategy.add_price(15.0);  // 现在价格是 [5.0, 5.0, 15.0]
                                // 短期MA = (5+15)/2 = 10.0
                                // 长期MA = (5+5+15)/3 = 8.33
        
        // 此时短期MA > 长期MA，没有卖出信号
        assert!(strategy.get_signal() != Some(Side::Sell));
        
        // 添加一个低价格，使短期MA < 长期MA
        // 但是我们需要确保前一时刻的短期MA > 前一时刻的长期MA
        // 所以先添加一个中间价格，确保短期MA仍然 > 长期MA
        strategy.add_price(10.0);  // 现在价格是 [5.0, 15.0, 10.0]
                                // 短期MA = (15+10)/2 = 12.5
                                // 长期MA = (5+15+10)/3 = 10.0
                                // 前一时刻短期MA = (5+15)/2 = 10.0
                                // 前一时刻长期MA = (5+5+15)/3 = 8.33
        
        // 现在添加一个低价格，使短期MA < 长期MA
        strategy.add_price(1.0);   // 现在价格是 [15.0, 10.0, 1.0]
                                // 短期MA = (10+1)/2 = 5.5
                                // 长期MA = (15+10+1)/3 = 8.67
                                // 前一时刻短期MA = (15+10)/2 = 12.5
                                // 前一时刻长期MA = (5+15+10)/3 = 10.0
                                // 短期MA从12.5下降到5.5，长期MA从10.0下降到8.67
                                // 短期MA从大于长期MA变为小于长期MA，产生卖出信号
        
        // 此时短期MA < 长期MA，产生卖出信号
        let signal = strategy.get_signal();
        assert!(signal.is_some(), "Expected a signal, got None");
        assert_eq!(signal.unwrap(), Side::Sell, "Expected Sell signal");
    }
        
    #[test]
    fn test_order_split_with_matching_signal() {
        let mut strategy = MAStrategy::new(2, 3);
        
        // 添加初始价格，使短期MA < 长期MA
        strategy.add_price(5.0);
        strategy.add_price(5.0);
        strategy.add_price(5.0);
        
        // 添加一个高价格，使短期MA > 长期MA
        strategy.add_price(15.0);  // 产生买入信号
        
        // 确认有买入信号
        let signal = strategy.get_signal();
        assert_eq!(signal, Some(Side::Buy), "Expected Buy signal, got {:?}", signal);
        
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
            strategy_id: "ma_strategy".to_string(),
        };
        
        // 分割订单
        let child_orders = strategy.split(&parent_order);
        
        // 验证生成了子订单
        assert_eq!(child_orders.len(), 1, "Expected 1 child order, got {}", child_orders.len());
        assert_eq!(child_orders[0].parent_id, "test_id");
        assert_eq!(child_orders[0].strategy_id, "ma_strategy");
    }
    #[test]
    fn test_order_split_with_non_matching_signal() {
        let mut strategy = MAStrategy::new(2, 3);
        
        // 添加初始价格，使短期MA < 长期MA
        strategy.add_price(10.0);
        strategy.add_price(10.0);
        strategy.add_price(10.0);
        
        // 添加一个高价格，使短期MA > 长期MA
        strategy.add_price(20.0);  // 产生买入信号
        
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
            strategy_id: "ma_strategy".to_string(),
        };
        
        // 分割订单
        let child_orders = strategy.split(&parent_order);
        
        // 验证没有生成子订单
        assert_eq!(child_orders.len(), 0);
    }
}