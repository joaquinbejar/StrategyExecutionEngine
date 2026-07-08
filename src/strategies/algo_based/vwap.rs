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

/*
VWAP (Volume Weighted Average Price)
The VWAP strategy executes orders to achieve a price close to the volume
weighted average price. VWAP is calculated by taking the total dollar amount
traded for every transaction (price times volume) and dividing it by the
total shares traded for the day. This strategy aims to minimize the market
impact by spreading the order throughout the trading day, aligning with the
natural volume distribution.

Example: If a trader wants to buy 10,000 shares, the VWAP algorithm will
distribute the order to buy shares at times when the trading volume is high,
thus achieving an average price close to the VWAP.

Reference for Further Reading:
"The Market Maker's Edge" by Joshua Lukeman: This book provides insights
into various trading tactics, including VWAP strategies. (Chapter 5 discusses
volume-weighted strategies in detail).
*/

use crate::models::{ChildOrder, ParentOrder};
use crate::strategies::common_strategies::OrderSplitStrategy;

pub struct VWAPStrategy;

impl OrderSplitStrategy for VWAPStrategy {
    fn split(&self, _parent_order: &ParentOrder) -> Vec<ChildOrder> {
        // Implement the VWAP splitting logic here
        vec![]
    }
}
