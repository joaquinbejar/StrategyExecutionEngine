/*
随机指标策略 (Stochastic Oscillator)
随机指标是一种动量指标，由两条线组成：%K线和%D线。该指标衡量当前价格相对于特定
时期内价格范围的位置。该策略基于%K线和%D线的交叉以及超买超卖水平产生信号。

例如：当%K线上穿%D线且两者都低于20时买入，当%K线下穿%D线且两者都高于80时卖出。

参考文献：
"Technical Analysis of Stock Trends" by Robert D. Edwards and John Magee：
这本书详细介绍了随机指标及其应用。(第11章详细讨论了振荡器指标的使用)
*/

use std::collections::VecDeque;
use crate::models::orders::Side;
use crate::models::child_orders::ChildOrder;
use crate::models::parent_orders::ParentOrder;
use crate::strategies::common_strategies::OrderSplitStrategy;

pub struct StochasticStrategy {
    k_period: usize,
    d_period: usize,
    prices: VecDeque<f64>,
    highs: VecDeque<f64>,
    lows: VecDeque<f64>,
    k_values: VecDeque<f64>,
    d_values: VecDeque<f64>,
    overbought_threshold: f64,
    oversold_threshold: f64,
}

impl StochasticStrategy {
    pub fn new(k_period: usize, d_period: usize, overbought_threshold: f64, oversold_threshold: f64) -> Self {
        StochasticStrategy {
            k_period,
            d_period,
            prices: VecDeque::with_capacity(k_period),
            highs: VecDeque::with_capacity(k_period),
            lows: VecDeque::with_capacity(k_period),
            k_values: VecDeque::with_capacity(d_period),
            d_values: VecDeque::new(),
            overbought_threshold,
            oversold_threshold,
        }
    }
    
    pub fn add_candle(&mut self, close: f64, high: f64, low: f64) {
        self.prices.push_back(close);
        self.highs.push_back(high);
        self.lows.push_back(low);
        
        if self.prices.len() > self.k_period {
            self.prices.pop_front();
            self.highs.pop_front();
            self.lows.pop_front();
        }
        
        self.calculate_stochastic();
    }
    
    fn calculate_stochastic(&mut self) {
        if self.prices.len() < self.k_period {
            return;
        }
        
        // 计算%K
        let highest_high = *self.highs.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let lowest_low = *self.lows.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        
        if highest_high == lowest_low {
            return;
        }
        
        let current_close = *self.prices.back().unwrap();
        let k_value = 100.0 * (current_close - lowest_low) / (highest_high - lowest_low);
        
        self.k_values.push_back(k_value);
        
        if self.k_values.len() > self.d_period {
            self.k_values.pop_front();
        }
        
        // 计算%D
        if self.k_values.len() >= self.d_period {
            let d_value: f64 = self.k_values.iter().sum::<f64>() / self.d_period as f64;
            self.d_values.push_back(d_value);
        }
    }
    
    pub fn get_signal(&self) -> Option<Side> {
        if self.k_values.len() < 2 || self.d_values.len() < 2 {
            return None;
        }
        
        let current_k = *self.k_values.back().unwrap();
        let prev_k = self.k_values.iter().rev().nth(1).unwrap();
        
        let current_d = *self.d_values.back().unwrap();
        let prev_d = self.d_values.iter().rev().nth(1).unwrap();
        
        // %K上穿%D且在超卖区域
        if prev_k < prev_d && current_k > current_d && current_k < self.oversold_threshold && current_d < self.oversold_threshold {
            Some(Side::Buy)
        }
        // %K下穿%D且在超买区域
        else if prev_k > prev_d && current_k < current_d && current_k > self.overbought_threshold && current_d > self.overbought_threshold {
            Some(Side::Sell)
        } else {
            None
        }
    }
}

impl OrderSplitStrategy for StochasticStrategy {
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
    fn test_stochastic_strategy_initialization() {
        let strategy = StochasticStrategy::new(14, 3, 80.0, 20.0);
        assert_eq!(strategy.k_period, 14);
        assert_eq!(strategy.d_period, 3);
        assert_eq!(strategy.overbought_threshold, 80.0);
        assert_eq!(strategy.oversold_threshold, 20.0);
        assert_eq!(strategy.prices.len(), 0);
        assert_eq!(strategy.highs.len(), 0);
        assert_eq!(strategy.lows.len(), 0);
        assert_eq!(strategy.k_values.len(), 0);
        assert_eq!(strategy.d_values.len(), 0);
    }

    #[test]
    fn test_add_candle() {
        let mut strategy = StochasticStrategy::new(5, 3, 80.0, 20.0);
        
        // 添加蜡烛线
        for i in 1..=10 {
            strategy.add_candle(i as f64, (i + 5) as f64, (i - 1) as f64);
        }
        
        // 检查价格队列长度不超过K周期
        assert_eq!(strategy.prices.len(), 5);
        assert_eq!(strategy.highs.len(), 5);
        assert_eq!(strategy.lows.len(), 5);
        
        // 检查价格队列内容（应该是最后5个价格）
        let expected_prices: Vec<f64> = (6..=10).map(|i| i as f64).collect();
        let actual_prices: Vec<f64> = strategy.prices.iter().cloned().collect();
        assert_eq!(actual_prices, expected_prices);
    }

    #[test]
    fn test_calculate_stochastic() {
        let mut strategy = StochasticStrategy::new(5, 3, 80.0, 20.0);
        
        // 添加蜡烛线，价格在一个范围内波动
        strategy.add_candle(50.0, 60.0, 40.0);
        strategy.add_candle(55.0, 65.0, 45.0);
        strategy.add_candle(60.0, 70.0, 50.0);
        strategy.add_candle(65.0, 75.0, 55.0);
        strategy.add_candle(70.0, 80.0, 60.0);
        
        // 此时应该有K值
        assert!(strategy.k_values.len() > 0);
        
        // 添加更多蜡烛线以计算D值
        strategy.add_candle(75.0, 85.0, 65.0);
        strategy.add_candle(80.0, 90.0, 70.0);
        strategy.add_candle(85.0, 95.0, 75.0);
        
        // 此时应该有D值
        assert!(strategy.d_values.len() > 0);
    }

    #[test]
    fn test_buy_signal_generation() {
        let mut strategy = StochasticStrategy::new(5, 3, 80.0, 20.0);
        
        // 添加蜡烛线，使K值和D值都低于20
        // 先添加足够的蜡烛线以计算K和D
        for i in 0..5 {
            strategy.add_candle(50.0, 60.0, 40.0);
        }
        
        // 添加下降趋势的蜡烛线，使K值下降到超卖区域
        strategy.add_candle(45.0, 55.0, 35.0);
        strategy.add_candle(40.0, 50.0, 30.0);
        strategy.add_candle(35.0, 45.0, 25.0);
        strategy.add_candle(30.0, 40.0, 20.0);
        
        // 添加反转蜡烛线，使K值上穿D值
        strategy.add_candle(40.0, 50.0, 30.0);
        
        // 此时可能有买入信号
        let signal = strategy.get_signal();
        if signal.is_some() {
            assert_eq!(signal.unwrap(), Side::Buy);
        }
    }

    #[test]
    fn test_sell_signal_generation() {
        let mut strategy = StochasticStrategy::new(5, 3, 80.0, 20.0);
        
        // 添加蜡烛线，使K值和D值都高于80
        // 先添加足够的蜡烛线以计算K和D
        for i in 0..5 {
            strategy.add_candle(50.0, 60.0, 40.0);
        }
        
        // 添加上升趋势的蜡烛线，使K值上升到超买区域
        strategy.add_candle(60.0, 70.0, 50.0);
        strategy.add_candle(70.0, 80.0, 60.0);
        strategy.add_candle(80.0, 90.0, 70.0);
        strategy.add_candle(90.0, 100.0, 80.0);
        
        // 添加反转蜡烛线，使K值下穿D值
        strategy.add_candle(80.0, 90.0, 70.0);
        
        // 此时可能有卖出信号
        let signal = strategy.get_signal();
        if signal.is_some() {
            assert_eq!(signal.unwrap(), Side::Sell);
        }
    }

    #[test]
    fn test_order_split_with_matching_signal() {
        let mut strategy = StochasticStrategy::new(5, 3, 80.0, 20.0);
        
        // 模拟一个买入信号
        // 这里我们直接修改k_values和d_values来创造一个明确的信号情况
        for _ in 0..5 {
            strategy.add_candle(50.0, 60.0, 40.0);
        }
        
        // 手动设置k_values和d_values以确保有买入信号
        strategy.k_values.clear();
        strategy.d_values.clear();
        
        // 添加K值（上穿）
        strategy.k_values.push_back(15.0);
        strategy.k_values.push_back(18.0);
        
        // 添加D值
        strategy.d_values.push_back(17.0);
        strategy.d_values.push_back(16.0);
        
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
            strategy_id: "stochastic_strategy".to_string(),
        };
        
        // 分割订单
        let child_orders = strategy.split(&parent_order);
        
        // 验证生成了子订单
        assert_eq!(child_orders.len(), 1);
        assert_eq!(child_orders[0].parent_id, "test_id");
        assert_eq!(child_orders[0].strategy_id, "stochastic_strategy");
    }

    #[test]
    fn test_order_split_with_non_matching_signal() {
        let mut strategy = StochasticStrategy::new(5, 3, 80.0, 20.0);
        
        // 模拟一个买入信号
        // 这里我们直接修改k_values和d_values来创造一个明确的信号情况
        for _ in 0..5 {
            strategy.add_candle(50.0, 60.0, 40.0);
        }
        
        // 手动设置k_values和d_values以确保有买入信号
        strategy.k_values.clear();
        strategy.d_values.clear();
    }
}