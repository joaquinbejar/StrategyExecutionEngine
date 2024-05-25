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

use super::common::{OrderCommon, OrderTrait};
use serde::{Deserialize, Serialize};

/// Structure representing a parent order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentOrder {
    pub common: OrderCommon,
    // Other specific fields for ParentOrder
}

impl OrderTrait for ParentOrder {
    fn new(id: String, quantity: u32) -> Self {
        ParentOrder {
            common: OrderCommon::new(id, quantity),
            // Initialize other specific fields
        }
    }

    fn get_id(&self) -> &String {
        self.common.get_id()
    }

    fn get_quantity(&self) -> u32 {
        self.common.get_quantity()
    }
}