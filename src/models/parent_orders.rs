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

use super::orders::{Futures, Options, Order, OrderType, ProductType, Side, Swap, TimeInForce};
use crate::{Validate, CFD};
use serde::{Deserialize, Serialize};

/// Structure representing a parent order.
#[derive(Clone, Serialize, Deserialize)]
pub struct ParentOrder {
    #[serde(flatten)]
    pub order_common: Order,
    pub strategy_id: String,
}

impl ParentOrder {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        quantity: u32,
        product_type: ProductType,
        order_type: OrderType,
        price: Option<f64>,
        timestamp: u64,
        expiry_date: Option<u64>,
        symbol: String,
        side: Side,
        currency: String,
        exchange: Option<String>,
        timeinforce: Option<TimeInForce>,
        futures_opt: Option<Futures>,
        options_opt: Option<Options>,
        swap_opt: Option<Swap>,
        cfd_opt: Option<CFD>,
        notional: Option<f64>,
        nonce: Option<u64>,
        strategy_id: String,
    ) -> Self {
        ParentOrder {
            order_common: Order::new(
                id,
                quantity,
                product_type,
                order_type,
                price,
                timestamp,
                expiry_date,
                symbol,
                side,
                currency,
                exchange,
                timeinforce,
                futures_opt,
                options_opt,
                swap_opt,
                cfd_opt,
                notional,
                nonce,
            ),
            strategy_id,
        }
    }
}

impl Validate for ParentOrder {
    fn validate(&self) -> Result<(), String> {
        if self.strategy_id.is_empty() {
            return Err("Strategy ID cannot be empty".to_string());
        }
        self.order_common.validate()
    }
}
