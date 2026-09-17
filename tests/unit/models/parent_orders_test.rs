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
   Date: 26/5/24
******************************************************************************/

#[cfg(test)]
mod parent_orders_tests {
    use strategy_execution_engine::models::orders::{
        Futures, OptionType, Options, OrderType, ProductType, Side, Swap, TimeInForce, CFD,
    };
    use strategy_execution_engine::models::parent_orders::ParentOrder;
    use strategy_execution_engine::Validate;

    #[test]
    fn test_create_parent_order() {
        let parent_order = ParentOrder::new(
            String::from("order1"),
            100,
            ProductType::Spot,
            OrderType::Market,
            Some(3000.0),
            1622512800,
            Some(1625114800),
            String::from("AAPL"),
            Side::Buy,
            String::from("USD"),
            Some(String::from("NASDAQ")),
            Some(TimeInForce::GTC),
            None,
            None,
            None,
            None,
            None,
            None,
            String::from("strategy1"),
        );

        assert_eq!(parent_order.order_common.id, "order1");
        assert_eq!(parent_order.order_common.quantity, 100);
        assert_eq!(
            format!("{:?}", parent_order.order_common.product_type),
            "Spot"
        );
        assert_eq!(
            format!("{:?}", parent_order.order_common.order_type),
            "Market"
        );
        assert_eq!(parent_order.order_common.price, Some(3000.0));
        assert_eq!(parent_order.order_common.timestamp, 1622512800);
        assert_eq!(parent_order.order_common.expiry_date, Some(1625114800));
        assert_eq!(parent_order.order_common.symbol, "AAPL");
        assert_eq!(format!("{:?}", parent_order.order_common.side), "Buy");
        assert_eq!(parent_order.order_common.currency, "USD");
        assert_eq!(
            parent_order.order_common.exchange,
            Some(String::from("NASDAQ"))
        );
        assert_eq!(
            format!("{:?}", parent_order.order_common.timeinforce),
            "Some(GTC)"
        );
        assert!(parent_order.order_common.futures_opt.is_none());
        assert!(parent_order.order_common.options_opt.is_none());
        assert!(parent_order.order_common.swap_opt.is_none());
        assert!(parent_order.order_common.cfd_opt.is_none());
        assert_eq!(parent_order.strategy_id, "strategy1");
    }

    #[test]
    fn test_parent_order_with_futures() {
        let futures = Futures {
            delivery_date: Some(20240101),
            contract_size: Some(100.0),
            margin: Some(0.05),
            commission: Some(0.02),
            overnight_fee: Some(0.01),
        };

        let parent_order = ParentOrder::new(
            String::from("order2"),
            50,
            ProductType::Futures,
            OrderType::Limit,
            Some(3200.0),
            1622512800,
            None,
            String::from("ES"),
            Side::Sell,
            String::from("USD"),
            Some(String::from("CME")),
            Some(TimeInForce::GTC),
            Some(futures),
            None,
            None,
            None,
            Some(100.0),
            Some(1234567890),
            String::from("strategy2"),
        );

        assert_eq!(parent_order.order_common.id, "order2");
        assert_eq!(parent_order.order_common.quantity, 50);
        assert_eq!(
            format!("{:?}", parent_order.order_common.product_type),
            "Futures"
        );
        assert_eq!(
            format!("{:?}", parent_order.order_common.order_type),
            "Limit"
        );
        assert_eq!(parent_order.order_common.price, Some(3200.0));
        assert_eq!(parent_order.order_common.timestamp, 1622512800);
        assert_eq!(parent_order.order_common.expiry_date, None);
        assert_eq!(parent_order.order_common.symbol, "ES");
        assert_eq!(format!("{:?}", parent_order.order_common.side), "Sell");
        assert_eq!(parent_order.order_common.currency, "USD");
        assert_eq!(
            parent_order.order_common.exchange,
            Some(String::from("CME"))
        );
        assert_eq!(
            format!("{:?}", parent_order.order_common.timeinforce),
            "Some(GTC)"
        );
        assert!(parent_order.order_common.futures_opt.is_some());
        assert!(parent_order.order_common.options_opt.is_none());
        assert!(parent_order.order_common.swap_opt.is_none());
        assert!(parent_order.order_common.cfd_opt.is_none());
        assert!(parent_order.order_common.notional.is_some());
        assert!(parent_order.order_common.nonce.is_some());
        assert_eq!(parent_order.strategy_id, "strategy2");
    }

    #[test]
    fn test_parent_order_with_options() {
        let options = Options {
            strike_price: 3000.0,
            option_type: OptionType::Call,
            expiry_date: 20241231,
        };

        let parent_order = ParentOrder::new(
            String::from("order3"),
            75,
            ProductType::Options,
            OrderType::Market,
            Some(3000.0),
            1622512800,
            Some(1625114800),
            String::from("AAPL"),
            Side::Buy,
            String::from("USD"),
            Some(String::from("NASDAQ")),
            Some(TimeInForce::GTC),
            None,
            Some(options),
            None,
            None,
            None,
            None,
            String::from("strategy3"),
        );

        assert_eq!(parent_order.order_common.id, "order3");
        assert_eq!(parent_order.order_common.quantity, 75);
        assert_eq!(
            format!("{:?}", parent_order.order_common.product_type),
            "Options"
        );
        assert_eq!(
            format!("{:?}", parent_order.order_common.order_type),
            "Market"
        );
        assert_eq!(parent_order.order_common.price, Some(3000.0));
        assert_eq!(parent_order.order_common.timestamp, 1622512800);
        assert_eq!(parent_order.order_common.expiry_date, Some(1625114800));
        assert_eq!(parent_order.order_common.symbol, "AAPL");
        assert_eq!(format!("{:?}", parent_order.order_common.side), "Buy");
        assert_eq!(parent_order.order_common.currency, "USD");
        assert_eq!(
            parent_order.order_common.exchange,
            Some(String::from("NASDAQ"))
        );
        assert_eq!(
            format!("{:?}", parent_order.order_common.timeinforce),
            "Some(GTC)"
        );
        assert!(parent_order.order_common.futures_opt.is_none());
        assert!(parent_order.order_common.options_opt.is_some());
        assert!(parent_order.order_common.swap_opt.is_none());
        assert!(parent_order.order_common.cfd_opt.is_none());
        assert_eq!(parent_order.strategy_id, "strategy3");
    }

    #[test]
    fn test_parent_order_with_swap() {
        let swap = Swap {
            fixed_rate: 1.5,
            floating_rate_index: String::from("LIBOR"),
            notional_amount: 1000000.0,
        };

        let parent_order = ParentOrder::new(
            String::from("order4"),
            25,
            ProductType::Swap,
            OrderType::Market,
            None,
            1622512800,
            None,
            String::from("SWAP1"),
            Side::Buy,
            String::from("USD"),
            Some(String::from("SWAPEX")),
            Some(TimeInForce::GTC),
            None,
            None,
            Some(swap),
            None,
            None,
            None,
            String::from("strategy4"),
        );

        assert_eq!(parent_order.order_common.id, "order4");
        assert_eq!(parent_order.order_common.quantity, 25);
        assert_eq!(
            format!("{:?}", parent_order.order_common.product_type),
            "Swap"
        );
        assert_eq!(
            format!("{:?}", parent_order.order_common.order_type),
            "Market"
        );
        assert_eq!(parent_order.order_common.price, None);
        assert_eq!(parent_order.order_common.timestamp, 1622512800);
        assert_eq!(parent_order.order_common.expiry_date, None);
        assert_eq!(parent_order.order_common.symbol, "SWAP1");
        assert_eq!(format!("{:?}", parent_order.order_common.side), "Buy");
        assert_eq!(parent_order.order_common.currency, "USD");
        assert_eq!(
            parent_order.order_common.exchange,
            Some(String::from("SWAPEX"))
        );
        assert_eq!(
            format!("{:?}", parent_order.order_common.timeinforce),
            "Some(GTC)"
        );
        assert!(parent_order.order_common.futures_opt.is_none());
        assert!(parent_order.order_common.options_opt.is_none());
        assert!(parent_order.order_common.swap_opt.is_some());
        assert!(parent_order.order_common.cfd_opt.is_none());
        assert_eq!(parent_order.strategy_id, "strategy4");
    }

    #[test]
    fn test_parent_order_with_cfd() {
        let cfd = CFD {
            leverage: Some(10),
            margin: Some(0.05),
            commission: Some(0.02),
            overnight_fee: Some(0.01),
            dividend_adjustment: Some(0.005),
            contract_size: Some(100.0),
        };

        let parent_order = ParentOrder::new(
            String::from("order5"),
            30,
            ProductType::CFD,
            OrderType::Limit,
            Some(2500.0),
            1622512800,
            None,
            String::from("CFD1"),
            Side::Sell,
            String::from("USD"),
            Some(String::from("CFDEX")),
            Some(TimeInForce::GTC),
            None,
            None,
            None,
            Some(cfd),
            None,
            None,
            String::from("strategy5"),
        );

        assert_eq!(parent_order.order_common.id, "order5");
        assert_eq!(parent_order.order_common.quantity, 30);
        assert_eq!(
            format!("{:?}", parent_order.order_common.product_type),
            "CFD"
        );
        assert_eq!(
            format!("{:?}", parent_order.order_common.order_type),
            "Limit"
        );
        assert_eq!(parent_order.order_common.price, Some(2500.0));
        assert_eq!(parent_order.order_common.timestamp, 1622512800);
        assert_eq!(parent_order.order_common.expiry_date, None);
        assert_eq!(parent_order.order_common.symbol, "CFD1");
        assert_eq!(format!("{:?}", parent_order.order_common.side), "Sell");
        assert_eq!(parent_order.order_common.currency, "USD");
        assert_eq!(
            parent_order.order_common.exchange,
            Some(String::from("CFDEX"))
        );
        assert_eq!(
            format!("{:?}", parent_order.order_common.timeinforce),
            "Some(GTC)"
        );
        assert!(parent_order.order_common.futures_opt.is_none());
        assert!(parent_order.order_common.options_opt.is_none());
        assert!(parent_order.order_common.swap_opt.is_none());
        assert!(parent_order.order_common.cfd_opt.is_some());
        assert_eq!(parent_order.strategy_id, "strategy5");
    }

    #[test]
    fn test_serialize_parent_order() {
        let parent_order = ParentOrder::new(
            String::from("order1"),
            100,
            ProductType::Spot,
            OrderType::Market,
            Some(3000.0),
            1622512800,
            Some(1625114800),
            String::from("AAPL"),
            Side::Buy,
            String::from("USD"),
            Some(String::from("NASDAQ")),
            Some(TimeInForce::GTC),
            None,
            None,
            None,
            None,
            None,
            None,
            String::from("strategy1"),
        );

        let serialized = serde_json::to_string(&parent_order).unwrap();
        assert!(serialized.contains("\"id\":\"order1\""));
        assert!(serialized.contains("\"quantity\":100"));
        assert!(serialized.contains("\"product_type\":\"Spot\""));
        assert!(serialized.contains("\"order_type\":\"Market\""));
        assert!(serialized.contains("\"price\":3000.0"));
        assert!(serialized.contains("\"timestamp\":1622512800"));
        assert!(serialized.contains("\"expiry_date\":1625114800"));
        assert!(serialized.contains("\"symbol\":\"AAPL\""));
        assert!(serialized.contains("\"side\":\"Buy\""));
        assert!(serialized.contains("\"currency\":\"USD\""));
        assert!(serialized.contains("\"exchange\":\"NASDAQ\""));
        assert!(serialized.contains("\"timeinforce\":\"GTC\""));
        assert!(serialized.contains("\"strategy_id\":\"strategy1\""));
    }

    #[test]
    fn test_deserialize_parent_order() {
        let data = r#"
        {
            "id": "order1",
            "quantity": 100,
            "product_type": "Spot",
            "order_type": "Market",
            "price": 3000.0,
            "timestamp": 1622512800,
            "expiry_date": 1625114800,
            "symbol": "AAPL",
            "side": "Buy",
            "currency": "USD",
            "exchange": "NASDAQ",
            "timeinforce": "GTC",
            "strategy_id": "strategy1"
        }
        "#;

        let deserialized: ParentOrder = serde_json::from_str(data).unwrap();
        assert_eq!(deserialized.order_common.id, "order1");
        assert_eq!(deserialized.order_common.quantity, 100);
        assert_eq!(
            format!("{:?}", deserialized.order_common.product_type),
            "Spot"
        );
        assert_eq!(
            format!("{:?}", deserialized.order_common.order_type),
            "Market"
        );
        assert_eq!(deserialized.order_common.price, Some(3000.0));
        assert_eq!(deserialized.order_common.timestamp, 1622512800);
        assert_eq!(deserialized.order_common.expiry_date, Some(1625114800));
        assert_eq!(deserialized.order_common.symbol, "AAPL");
        assert_eq!(format!("{:?}", deserialized.order_common.side), "Buy");
        assert_eq!(deserialized.order_common.currency, "USD");
        assert_eq!(
            deserialized.order_common.exchange,
            Some(String::from("NASDAQ"))
        );
        assert_eq!(
            format!("{:?}", deserialized.order_common.timeinforce),
            "Some(GTC)"
        );
        assert_eq!(deserialized.strategy_id, "strategy1");
    }

    #[test]
    fn test_parent_order_validation() {
        let parent_order = ParentOrder::new(
            String::from("order1"),
            100,
            ProductType::Spot,
            OrderType::Market,
            Some(3000.0),
            1622512800,
            Some(1625114800),
            String::from("AAPL"),
            Side::Buy,
            String::from("USD"),
            Some(String::from("NASDAQ")),
            Some(TimeInForce::GTC),
            None,
            None,
            None,
            None,
            None,
            None,
            String::from("strategy1"),
        );
        assert!(parent_order.validate().is_ok());
    }

    #[test]
    fn test_parent_order_validation_failure() {
        let parent_order = ParentOrder::new(
            String::from("order1"),
            100,
            ProductType::Spot,
            OrderType::Market,
            Some(3000.0),
            1622512800,
            Some(1625114800),
            String::from("AAPL"),
            Side::Buy,
            String::from("USD"),
            Some(String::from("NASDAQ")),
            Some(TimeInForce::GTC),
            None,
            None,
            None,
            None,
            None,
            None,
            String::new(),
        );
        assert!(parent_order.validate().is_err());
    }
}
