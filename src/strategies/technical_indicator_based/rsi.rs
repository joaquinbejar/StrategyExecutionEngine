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

use crate::models::child_orders::ChildOrder;
use crate::models::orders::Side;
use crate::models::parent_orders::ParentOrder;
use crate::strategies::common_strategies::OrderSplitStrategy;
#[cfg(feature = "wilder-rsi")]
use fin_primitives::signals::{indicators::Rsi, BarInput, Signal};
#[cfg(feature = "wilder-rsi")]
use rust_decimal::prelude::{Decimal, FromPrimitive, ToPrimitive};
use std::collections::VecDeque;

/// RSI reported when neither gains nor losses have occurred (a flat market).
/// 50 is the neutral midpoint, so no Buy or Sell fires on a market that never moved.
const FLAT_RSI: f64 = 50.0;

fn rsi_signal(rsi: f64, overbought_threshold: f64, oversold_threshold: f64) -> Option<Side> {
    if rsi < oversold_threshold {
        Some(Side::Buy)
    } else if rsi > overbought_threshold {
        Some(Side::Sell)
    } else {
        None
    }
}

fn split_on_signal(signal: Option<Side>, parent_order: &ParentOrder) -> Vec<ChildOrder> {
    match (signal, &parent_order.order_common.side) {
        (Some(Side::Buy), Side::Buy) | (Some(Side::Sell), Side::Sell) => {
            vec![ChildOrder {
                order_common: parent_order.order_common.clone(),
                strategy_id: parent_order.strategy_id.clone(),
                parent_id: parent_order.order_common.id.clone(),
                insert_at: Some(parent_order.order_common.timestamp),
            }]
        }
        _ => Vec::new(),
    }
}

/// RSI over a rolling simple average of the last `period` gains and losses.
pub struct RSIStrategy {
    period: usize,
    prices: VecDeque<f64>,
    gains: VecDeque<f64>,
    losses: VecDeque<f64>,
    overbought_threshold: f64,
    oversold_threshold: f64,
    rejected_ticks: u64,
}

impl RSIStrategy {
    /// # Panics
    /// Panics if `period` is 0.
    pub fn new(period: usize, overbought_threshold: f64, oversold_threshold: f64) -> Self {
        assert!(period > 0, "RSI period must be greater than 0");
        RSIStrategy {
            period,
            prices: VecDeque::with_capacity(period + 1),
            gains: VecDeque::with_capacity(period),
            losses: VecDeque::with_capacity(period),
            overbought_threshold,
            oversold_threshold,
            rejected_ticks: 0,
        }
    }

    /// Adds a close. NaN and infinite prices are skipped and counted in `rejected_ticks`.
    pub fn add_price(&mut self, price: f64) {
        if !price.is_finite() {
            self.rejected_ticks += 1;
            return;
        }

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

    /// Number of prices skipped by `add_price` because they were not finite.
    pub fn rejected_ticks(&self) -> u64 {
        self.rejected_ticks
    }

    pub fn calculate_rsi(&self) -> Option<f64> {
        if self.gains.len() < self.period {
            return None;
        }

        let avg_gain: f64 = self.gains.iter().sum::<f64>() / self.period as f64;
        let avg_loss: f64 = self.losses.iter().sum::<f64>() / self.period as f64;

        if avg_loss == 0.0 {
            if avg_gain == 0.0 {
                return Some(FLAT_RSI);
            }
            return Some(100.0);
        }

        let rs = avg_gain / avg_loss;
        let rsi = 100.0 - (100.0 / (1.0 + rs));

        Some(rsi)
    }

    pub fn get_signal(&self) -> Option<Side> {
        let rsi = self.calculate_rsi()?;
        rsi_signal(rsi, self.overbought_threshold, self.oversold_threshold)
    }
}

impl OrderSplitStrategy for RSIStrategy {
    fn split(&self, parent_order: &ParentOrder) -> Vec<ChildOrder> {
        split_on_signal(self.get_signal(), parent_order)
    }
}

/// Largest absolute price `WilderRSIStrategy` accepts. Keeping inputs this far below
/// `Decimal::MAX` (about 7.9e28) means the gain/loss sums and the Wilder products
/// cannot overflow for any `u32` period, so `Rsi::update` never fails on accepted input.
#[cfg(feature = "wilder-rsi")]
const WILDER_MAX_PRICE: f64 = 1e15;

/// RSI with Wilder smoothing (`avg = (avg * (period - 1) + x) / period` after an SMA
/// seed), computed in `Decimal` by `fin_primitives::signals::indicators::Rsi`.
/// Matches Wilder's book and TA-Lib. Same warm-up and signal rules as `RSIStrategy`.
#[cfg(feature = "wilder-rsi")]
pub struct WilderRSIStrategy {
    rsi: Rsi,
    prev_close: Option<Decimal>,
    // Set once any close differs from the one before it. Until then the market is flat.
    moved: bool,
    value: Option<f64>,
    overbought_threshold: f64,
    oversold_threshold: f64,
    rejected_ticks: u64,
}

#[cfg(feature = "wilder-rsi")]
impl WilderRSIStrategy {
    /// # Panics
    /// Panics if `period` is 0 or does not fit in a `u32`.
    pub fn new(period: usize, overbought_threshold: f64, oversold_threshold: f64) -> Self {
        assert!(period > 0, "RSI period must be greater than 0");
        assert!(
            u32::try_from(period).is_ok(),
            "RSI period must fit in a u32"
        );
        WilderRSIStrategy {
            rsi: Rsi::new("rsi", period).expect("period checked above"),
            prev_close: None,
            moved: false,
            value: None,
            overbought_threshold,
            oversold_threshold,
            rejected_ticks: 0,
        }
    }

    /// Adds a close. A price that is NaN, infinite, above `1e15` in absolute value, or
    /// that `Rsi::update` rejects is skipped and counted in `rejected_ticks`; the
    /// Wilder state carries on from the last good close.
    pub fn add_price(&mut self, price: f64) {
        let close = match Decimal::from_f64(price) {
            Some(close) if price.is_finite() && price.abs() <= WILDER_MAX_PRICE => close,
            _ => {
                self.rejected_ticks += 1;
                return;
            }
        };
        let unchanged = self.prev_close == Some(close);
        if self.prev_close.is_some() && !unchanged {
            self.moved = true;
        }

        let result = self.rsi.update(&BarInput::from_close(close));
        // `Rsi::update` records the close before any step that can fail.
        self.prev_close = Some(close);
        let value = match result {
            Ok(value) => value.as_decimal().and_then(|v| v.to_f64()),
            Err(_) => {
                self.rejected_ticks += 1;
                return;
            }
        };
        let Some(value) = value else {
            return;
        };

        self.value = if !self.moved {
            // Both averages are zero: report neutral instead of `Rsi`'s 100.
            Some(FLAT_RSI)
        } else if unchanged && self.value.is_some() {
            // A zero change scales both averages by (period - 1) / period, so the RSI
            // is exactly the previous one. Holding it also stops a long flat run from
            // rounding both Decimal averages down to the same smallest unit (RSI 50).
            self.value
        } else {
            Some(value)
        };
    }

    /// Number of prices skipped by `add_price`.
    pub fn rejected_ticks(&self) -> u64 {
        self.rejected_ticks
    }

    pub fn calculate_rsi(&self) -> Option<f64> {
        self.value
    }

    pub fn get_signal(&self) -> Option<Side> {
        let rsi = self.calculate_rsi()?;
        rsi_signal(rsi, self.overbought_threshold, self.oversold_threshold)
    }
}

#[cfg(feature = "wilder-rsi")]
impl OrderSplitStrategy for WilderRSIStrategy {
    fn split(&self, parent_order: &ParentOrder) -> Vec<ChildOrder> {
        split_on_signal(self.get_signal(), parent_order)
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

    #[test]
    fn test_flat_market_is_neutral() {
        let mut strategy = RSIStrategy::new(5, 70.0, 30.0);
        for _ in 0..10 {
            strategy.add_price(100.0);
        }
        assert_eq!(strategy.calculate_rsi(), Some(50.0));
        assert!(strategy.get_signal().is_none());
    }

    #[test]
    fn test_non_finite_prices_are_skipped() {
        let mut strategy = RSIStrategy::new(5, 70.0, 30.0);
        let mut clean = RSIStrategy::new(5, 70.0, 30.0);
        for price in [100.0, 95.0, 90.0, 85.0, 80.0, 75.0] {
            strategy.add_price(price);
            clean.add_price(price);
            strategy.add_price(f64::NAN);
            strategy.add_price(f64::INFINITY);
        }
        assert_eq!(strategy.rejected_ticks(), 12);
        assert_eq!(strategy.prices, clean.prices);
        assert_eq!(strategy.calculate_rsi(), clean.calculate_rsi());
        assert_eq!(strategy.get_signal(), Some(Side::Buy));
    }

    #[test]
    #[should_panic(expected = "RSI period must be greater than 0")]
    fn test_zero_period_panics() {
        RSIStrategy::new(0, 70.0, 30.0);
    }

    // Wilder's 14-period example closes.
    #[cfg(feature = "wilder-rsi")]
    const WILDER_CLOSES: [f64; 20] = [
        44.34, 44.09, 44.15, 43.61, 44.33, 44.83, 45.10, 45.42, 45.84, 46.08, 45.89, 46.03, 45.61,
        46.28, 46.28, 46.00, 46.03, 46.41, 46.22, 45.64,
    ];

    #[cfg(feature = "wilder-rsi")]
    #[test]
    fn test_wilder_rsi_reference_values() {
        // After the seed the values are smoothed, so the last RSI is 57.92
        // (a simple average of the last 14 changes gives 59.81).
        let mut strategy = WilderRSIStrategy::new(14, 70.0, 30.0);
        for (i, close) in WILDER_CLOSES.iter().enumerate() {
            strategy.add_price(*close);
            if i < 14 {
                assert!(strategy.calculate_rsi().is_none());
            }
            if i == 14 {
                assert!((strategy.calculate_rsi().unwrap() - 70.46).abs() < 0.01);
            }
        }
        assert!((strategy.calculate_rsi().unwrap() - 57.92).abs() < 0.01);
    }

    #[cfg(feature = "wilder-rsi")]
    #[test]
    fn test_feature_does_not_change_rsi_strategy() {
        let mut strategy = RSIStrategy::new(14, 70.0, 30.0);
        for close in WILDER_CLOSES {
            strategy.add_price(close);
        }
        assert!((strategy.calculate_rsi().unwrap() - 59.81).abs() < 0.01);
    }

    #[cfg(feature = "wilder-rsi")]
    #[test]
    fn test_wilder_rsi_skips_bad_ticks() {
        let mut strategy = WilderRSIStrategy::new(14, 70.0, 30.0);
        for (i, close) in WILDER_CLOSES.iter().enumerate() {
            strategy.add_price(*close);
            if i == 16 {
                strategy.add_price(f64::NAN);
                strategy.add_price(f64::NEG_INFINITY);
                strategy.add_price(1e30);
                strategy.add_price(-1e16);
            }
        }
        assert_eq!(strategy.rejected_ticks(), 4);
        assert!((strategy.calculate_rsi().unwrap() - 57.92).abs() < 0.01);
    }

    #[cfg(feature = "wilder-rsi")]
    #[test]
    fn test_wilder_rsi_flat_market_is_neutral() {
        let mut strategy = WilderRSIStrategy::new(5, 70.0, 30.0);
        for _ in 0..10 {
            strategy.add_price(100.0);
        }
        assert_eq!(strategy.calculate_rsi(), Some(50.0));
        assert!(strategy.get_signal().is_none());
    }

    #[cfg(feature = "wilder-rsi")]
    #[test]
    fn test_wilder_rsi_holds_through_long_flat_run() {
        let mut strategy = WilderRSIStrategy::new(14, 70.0, 30.0);
        for close in WILDER_CLOSES {
            strategy.add_price(close);
        }
        let before = strategy.calculate_rsi().unwrap();
        // Long enough for both Decimal averages to round down to their smallest unit.
        for _ in 0..2000 {
            strategy.add_price(45.64);
        }
        assert_eq!(strategy.calculate_rsi(), Some(before));
    }

    #[cfg(feature = "wilder-rsi")]
    #[test]
    #[should_panic(expected = "RSI period must be greater than 0")]
    fn test_wilder_rsi_zero_period_panics() {
        WilderRSIStrategy::new(0, 70.0, 30.0);
    }
}
