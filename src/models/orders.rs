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
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProductType {
    Spot,
    Futures,
    Options,
    Swap,
    CFD,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptionType {
    Call,
    Put,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeInForce {
    GTC, // Good-Til-Canceled
    IOC, // Immediate-Or-Cancel
    GTD, // Good-Til-Date
    FOK, // Fill-Or-Kill
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssetClass {
    Stock,
    Bond,
    Commodity,
    Currency,
    Crypto,
    ETF,
    MutualFund,
    Index,
    Equity,
    Derivative,
    Insurance,
    Loan,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Spot {
    pub asset_class: Option<AssetClass>,
}

impl Validate for Spot {
    fn validate(&self) -> Result<(), String> {
        // TODO: Implement validation logic
        Ok(())
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Futures {
    pub delivery_date: Option<u64>,
    pub contract_size: Option<f64>,
    pub margin: Option<f64>,
    pub commission: Option<f64>,
    pub overnight_fee: Option<f64>,
}

impl Futures {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(contract_size) = self.contract_size {
            if contract_size <= 0.0 {
                return Err("Contract size must be greater than zero".to_string());
            }
        }
        if let Some(margin) = self.margin {
            if margin < 0.0 {
                return Err("Margin cannot be negative".to_string());
            }
        }
        if let Some(commission) = self.commission {
            if commission < 0.0 {
                return Err("Commission cannot be negative".to_string());
            }
        }
        if let Some(overnight_fee) = self.overnight_fee {
            if overnight_fee < 0.0 {
                return Err("Overnight fee cannot be negative".to_string());
            }
        }
        Ok(())
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Options {
    pub strike_price: f64,
    pub option_type: OptionType,
    pub expiry_date: u64,
}

impl Options {
    pub fn validate(&self) -> Result<(), String> {
        if self.strike_price <= 0.0 {
            return Err("Strike price must be greater than zero".to_string());
        }
        Ok(())
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Swap {
    pub fixed_rate: f64,
    pub floating_rate_index: String,
    pub notional_amount: f64,
}

impl Swap {
    pub fn validate(&self) -> Result<(), String> {
        if self.fixed_rate < 0.0 {
            return Err("Fixed rate cannot be negative".to_string());
        }
        if self.notional_amount <= 0.0 {
            return Err("Notional amount must be greater than zero".to_string());
        }
        if self.floating_rate_index.is_empty() {
            return Err("Floating rate index cannot be empty".to_string());
        }
        Ok(())
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CFD {
    pub leverage: Option<u32>,
    pub margin: Option<f64>,
    pub commission: Option<f64>,
    pub overnight_fee: Option<f64>,
    pub dividend_adjustment: Option<f64>,
    pub contract_size: Option<f64>,
}

impl CFD {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(leverage) = self.leverage {
            if leverage == 0 {
                return Err("Leverage must be greater than zero".to_string());
            }
        }
        if let Some(margin) = self.margin {
            if margin < 0.0 {
                return Err("Margin cannot be negative".to_string());
            }
        }
        if let Some(commission) = self.commission {
            if commission < 0.0 {
                return Err("Commission cannot be negative".to_string());
            }
        }
        if let Some(overnight_fee) = self.overnight_fee {
            if overnight_fee < 0.0 {
                return Err("Overnight fee cannot be negative".to_string());
            }
        }
        Ok(())
    }
}

/// Common structure for orders.
#[derive(Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    pub quantity: u32,
    pub product_type: ProductType,
    pub order_type: OrderType,
    pub price: Option<f64>,
    pub timestamp: u64,
    pub expiry_date: Option<u64>,
    pub symbol: String,
    pub side: Side,
    pub currency: String,
    pub exchange: Option<String>,
    pub timeinforce: Option<TimeInForce>,

    // Futures specific fields
    pub futures_opt: Option<Futures>,

    // Options specific fields
    pub options_opt: Option<Options>,

    // Swaps specific fields
    pub swap_opt: Option<Swap>,

    // CFDs specific fields
    pub cfd_opt: Option<CFD>,

    pub notional: Option<f64>,
    pub nonce: Option<u64>,
}

impl Order {
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
    ) -> Self {
        Order {
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
        }
    }
}

pub trait Validate {
    fn validate(&self) -> Result<(), String>;
}

impl Validate for Order {
    fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("ID cannot be empty".to_string());
        }
        if self.quantity == 0 {
            return Err("Quantity must be greater than zero".to_string());
        }
        if self.symbol.is_empty() {
            return Err("Symbol cannot be empty".to_string());
        }
        if self.currency.is_empty() {
            return Err("Currency cannot be empty".to_string());
        }
        if let Some(notional) = self.notional {
            if notional <= 0.0 {
                return Err("Notional must be greater than zero".to_string());
            }
        }
        if let Some(futures) = &self.futures_opt {
            futures.validate()?;
        }
        if let Some(options) = &self.options_opt {
            options.validate()?;
        }
        if let Some(swap) = &self.swap_opt {
            swap.validate()?;
        }
        if let Some(cfd) = &self.cfd_opt {
            cfd.validate()?;
        }
        Ok(())
    }
}
