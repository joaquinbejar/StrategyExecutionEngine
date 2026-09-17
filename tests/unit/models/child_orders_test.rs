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
   Date: 28/5/24
******************************************************************************/

#[cfg(test)]
mod child_orders_tests {
    use strategy_execution_engine::models::child_orders::ChildOrder;
    use strategy_execution_engine::models::orders::{
        Order, OrderType, ProductType, Side, TimeInForce,
    };
    use strategy_execution_engine::Validate;

    fn create_valid_order() -> Order {
        Order::new(
            "test_id".to_string(),
            100,
            ProductType::Futures,
            OrderType::Limit,
            Some(100.0),
            1234567890,
            Some(1234567890),
            "AAPL".to_string(),
            Side::Buy,
            "USD".to_string(),
            Some("NYSE".to_string()),
            Some(TimeInForce::GTC),
            None,
            None,
            None,
            None,
            Some(1000.0),
            Some(1),
        )
    }

    #[test]
    fn test_child_order_creation() {
        let order = ChildOrder::new(
            "test_id".to_string(),
            100,
            ProductType::Futures,
            OrderType::Limit,
            Some(100.0),
            1234567890,
            Some(1234567890),
            "AAPL".to_string(),
            Side::Buy,
            "USD".to_string(),
            Some("NYSE".to_string()),
            Some(TimeInForce::GTC),
            None,
            None,
            None,
            None,
            Some(1000.0),
            Some(1),
            "strategy_1".to_string(),
            "parent_1".to_string(),
            Some(1234567890),
        );

        assert_eq!(order.strategy_id, "strategy_1");
        assert_eq!(order.parent_id, "parent_1");
        assert!(order.insert_at.is_some());
    }

    #[test]
    fn test_child_order_validate_success() {
        let order = ChildOrder::new(
            "test_id".to_string(),
            100,
            ProductType::Futures,
            OrderType::Limit,
            Some(100.0),
            1234567890,
            Some(1234567890),
            "AAPL".to_string(),
            Side::Buy,
            "USD".to_string(),
            Some("NYSE".to_string()),
            Some(TimeInForce::GTC),
            None,
            None,
            None,
            None,
            Some(1000.0),
            Some(1),
            "strategy_1".to_string(),
            "parent_1".to_string(),
            Some(1234567890),
        );

        assert!(order.validate().is_ok());
    }

    #[test]
    fn test_child_order_validate_empty_strategy_id() {
        let order = ChildOrder::new(
            "test_id".to_string(),
            100,
            ProductType::Futures,
            OrderType::Limit,
            Some(100.0),
            1234567890,
            Some(1234567890),
            "AAPL".to_string(),
            Side::Buy,
            "USD".to_string(),
            Some("NYSE".to_string()),
            Some(TimeInForce::GTC),
            None,
            None,
            None,
            None,
            Some(1000.0),
            Some(1),
            "".to_string(),
            "parent_1".to_string(),
            Some(1234567890),
        );

        assert!(order.validate().is_err());
    }

    #[test]
    fn test_child_order_validate_empty_parent_id() {
        let order = ChildOrder::new(
            "test_id".to_string(),
            100,
            ProductType::Futures,
            OrderType::Limit,
            Some(100.0),
            1234567890,
            Some(1234567890),
            "AAPL".to_string(),
            Side::Buy,
            "USD".to_string(),
            Some("NYSE".to_string()),
            Some(TimeInForce::GTC),
            None,
            None,
            None,
            None,
            Some(1000.0),
            Some(1),
            "strategy_1".to_string(),
            "".to_string(),
            Some(1234567890),
        );

        assert!(order.validate().is_err());
    }

    #[test]
    fn test_child_order_common_order_validation() {
        let mut order = create_valid_order();
        order.quantity = 0; // Invalid order since quantity is 0

        let child_order = ChildOrder {
            order_common: order,
            strategy_id: "strategy_1".to_string(),
            parent_id: "parent_1".to_string(),
            insert_at: Some(1234567890),
        };

        assert!(child_order.validate().is_err());
    }
}
