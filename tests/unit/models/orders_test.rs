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
mod orders_tests {
    use strategy_execution_engine::models::orders::{
        Futures, OptionType, Options, Order, OrderType, ProductType, Side, Swap, TimeInForce, CFD,
    };
    use strategy_execution_engine::Validate;

    #[test]
    fn test_create_product_type() {
        let spot = ProductType::Spot;
        let futures = ProductType::Futures;
        let options = ProductType::Options;
        let swap = ProductType::Swap;
        let cfd = ProductType::CFD;

        assert_eq!(format!("{:?}", spot), "Spot");
        assert_eq!(format!("{:?}", futures), "Futures");
        assert_eq!(format!("{:?}", options), "Options");
        assert_eq!(format!("{:?}", swap), "Swap");
        assert_eq!(format!("{:?}", cfd), "CFD");
    }

    #[test]
    fn test_create_order_type() {
        let market = OrderType::Market;
        let limit = OrderType::Limit;

        assert_eq!(format!("{:?}", market), "Market");
        assert_eq!(format!("{:?}", limit), "Limit");
    }

    #[test]
    fn test_create_side() {
        let buy = Side::Buy;
        let sell = Side::Sell;

        assert_eq!(format!("{:?}", buy), "Buy");
        assert_eq!(format!("{:?}", sell), "Sell");
    }

    #[test]
    fn test_create_option_type() {
        let call = OptionType::Call;
        let put = OptionType::Put;

        assert_eq!(format!("{:?}", call), "Call");
        assert_eq!(format!("{:?}", put), "Put");
    }

    #[test]
    fn test_create_time_in_force() {
        let gtc = TimeInForce::GTC;
        let ioc = TimeInForce::IOC;
        let fok = TimeInForce::FOK;

        assert_eq!(format!("{:?}", gtc), "GTC");
        assert_eq!(format!("{:?}", ioc), "IOC");
        assert_eq!(format!("{:?}", fok), "FOK");
    }

    #[test]
    fn test_create_futures() {
        let futures = Futures {
            delivery_date: Some(20240101),
            contract_size: Some(100.0),
            margin: Some(0.05),
            commission: Some(0.02),
            overnight_fee: Some(0.01),
        };

        assert_eq!(futures.delivery_date, Some(20240101));
        assert_eq!(futures.contract_size, Some(100.0));
        assert_eq!(futures.margin, Some(0.05));
        assert_eq!(futures.commission, Some(0.02));
        assert_eq!(futures.overnight_fee, Some(0.01));
    }

    #[test]
    fn test_create_options() {
        let options = Options {
            strike_price: 3000.0,
            option_type: OptionType::Call,
            expiry_date: 20241231,
        };

        assert_eq!(options.strike_price, 3000.0);
        assert_eq!(format!("{:?}", options.option_type), "Call");
        assert_eq!(options.expiry_date, 20241231);
    }

    #[test]
    fn test_create_swap() {
        let swap = Swap {
            fixed_rate: 1.5,
            floating_rate_index: String::from("LIBOR"),
            notional_amount: 1000000.0,
        };

        assert_eq!(swap.fixed_rate, 1.5);
        assert_eq!(swap.floating_rate_index, "LIBOR");
        assert_eq!(swap.notional_amount, 1000000.0);
    }

    #[test]
    fn test_create_cfd() {
        let cfd = CFD {
            leverage: Some(10),
            margin: Some(0.05),
            commission: Some(0.02),
            overnight_fee: Some(0.01),
            dividend_adjustment: Some(0.005),
            contract_size: Some(100.0),
        };

        assert_eq!(cfd.leverage, Some(10));
        assert_eq!(cfd.margin, Some(0.05));
        assert_eq!(cfd.commission, Some(0.02));
        assert_eq!(cfd.overnight_fee, Some(0.01));
        assert_eq!(cfd.dividend_adjustment, Some(0.005));
        assert_eq!(cfd.contract_size, Some(100.0));
    }

    #[test]
    fn test_create_order() {
        let order = Order::new(
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
        );

        assert_eq!(order.id, "order1");
        assert_eq!(order.quantity, 100);
        assert_eq!(format!("{:?}", order.product_type), "Spot");
        assert_eq!(format!("{:?}", order.order_type), "Market");
        assert_eq!(order.price, Some(3000.0));
        assert_eq!(order.timestamp, 1622512800);
        assert_eq!(order.expiry_date, Some(1625114800));
        assert_eq!(order.symbol, "AAPL");
        assert_eq!(format!("{:?}", order.side), "Buy");
        assert_eq!(order.currency, "USD");
        assert_eq!(order.exchange, Some(String::from("NASDAQ")));
        assert_eq!(format!("{:?}", order.timeinforce), "Some(GTC)");
        assert!(order.futures_opt.is_none());
        assert!(order.options_opt.is_none());
        assert!(order.swap_opt.is_none());
        assert!(order.cfd_opt.is_none());
    }

    #[test]
    fn test_order_with_futures() {
        let futures = Futures {
            delivery_date: Some(20240101),
            contract_size: Some(100.0),
            margin: Some(0.05),
            commission: Some(0.02),
            overnight_fee: Some(0.01),
        };

        let order = Order::new(
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
            None,
            None,
        );

        assert_eq!(order.id, "order2");
        assert_eq!(order.quantity, 50);
        assert_eq!(format!("{:?}", order.product_type), "Futures");
        assert_eq!(format!("{:?}", order.order_type), "Limit");
        assert_eq!(order.price, Some(3200.0));
        assert_eq!(order.timestamp, 1622512800);
        assert_eq!(order.expiry_date, None);
        assert_eq!(order.symbol, "ES");
        assert_eq!(format!("{:?}", order.side), "Sell");
        assert_eq!(order.currency, "USD");
        assert_eq!(order.exchange, Some(String::from("CME")));
        assert_eq!(format!("{:?}", order.timeinforce), "Some(GTC)");
        assert!(order.futures_opt.is_some());
        assert!(order.options_opt.is_none());
        assert!(order.swap_opt.is_none());
        assert!(order.cfd_opt.is_none());
    }

    #[test]
    fn test_order_with_options() {
        let options = Options {
            strike_price: 3000.0,
            option_type: OptionType::Call,
            expiry_date: 20241231,
        };

        let order = Order::new(
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
            Some(13.3),
            Some(123456),
        );

        assert_eq!(order.id, "order3");
        assert_eq!(order.quantity, 75);
        assert_eq!(format!("{:?}", order.product_type), "Options");
        assert_eq!(format!("{:?}", order.order_type), "Market");
        assert_eq!(order.price, Some(3000.0));
        assert_eq!(order.timestamp, 1622512800);
        assert_eq!(order.expiry_date, Some(1625114800));
        assert_eq!(order.symbol, "AAPL");
        assert_eq!(format!("{:?}", order.side), "Buy");
        assert_eq!(order.currency, "USD");
        assert_eq!(order.exchange, Some(String::from("NASDAQ")));
        assert_eq!(format!("{:?}", order.timeinforce), "Some(GTC)");
        assert!(order.futures_opt.is_none());
        assert!(order.options_opt.is_some());
        assert!(order.swap_opt.is_none());
        assert!(order.cfd_opt.is_none());
        assert_eq!(order.notional, Some(13.3));
        assert_eq!(order.nonce, Some(123456));
    }

    #[test]
    fn test_order_validation() {
        let order = Order::new(
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
        );
        assert!(order.validate().is_ok());
    }

    #[test]
    fn test_order_validation_failure() {
        let order = Order::new(
            String::new(),
            0,
            ProductType::Spot,
            OrderType::Market,
            Some(3000.0),
            1622512800,
            Some(1625114800),
            String::new(),
            Side::Buy,
            String::new(),
            Some(String::from("NASDAQ")),
            Some(TimeInForce::GTC),
            None,
            None,
            None,
            None,
            None,
            None,
        );
        assert!(order.validate().is_err());
    }
}
