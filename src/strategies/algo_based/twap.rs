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
TWAP (Time Weighted Average Price)
The TWAP strategy divides a large order into equal parts to execute over a
defined period. TWAP is calculated by taking the average price of the security
over a specific time period. This strategy is designed to execute a large order
evenly throughout the trading day, thereby reducing the risk of market impact
and price fluctuations.

Example: If a trader needs to buy 10,000 shares over 5 hours, the TWAP
algorithm will execute 2,000 shares each hour, ensuring an even distribution
and minimizing the impact on the market price.

Reference for Further Reading:
"The Market Maker's Edge" by Joshua Lukeman: This book explains the
importance of timing in trade execution and how traders can optimize their
strategies using TWAP. (Chapter 6 covers time-weighted strategies in detail).
*/

use crate::models::{ChildOrder, ParentOrder};
use crate::strategies::common_strategies::OrderSplitStrategy;
use std::vec::Vec;

pub struct TWAPStrategy;

impl OrderSplitStrategy for TWAPStrategy {
    fn split(&self, _parent_order: &ParentOrder) -> Vec<ChildOrder> {
        // Implement the TWAP splitting logic here
        vec![]
    }
}
