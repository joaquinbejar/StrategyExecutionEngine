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
   Date: 29/5/24
******************************************************************************/

use crate::{ChildOrder, Futures, Options, Order, ParentOrder, Spot, Swap, CFD};
use serde::Serialize;
use std::fmt::{Formatter, Result as FmtResult};

fn format_helper<T: Serialize>(
    serializable: &T,
    f: &mut Formatter<'_>,
    is_pretty: bool,
) -> FmtResult {
    let result = if is_pretty {
        serde_json::to_string_pretty(serializable)
    } else {
        serde_json::to_string(serializable)
    };
    match result {
        Ok(json_str) => write!(f, "{}", json_str),
        Err(_) => Err(std::fmt::Error),
    }
}

// This macro will generate code to implement the traits for a given type.
macro_rules! impl_fmt {
    ($T:ty) => {
        impl std::fmt::Debug for $T {
            fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
                format_helper(self, f, true)
            }
        }
        impl std::fmt::Display for $T {
            fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
                format_helper(self, f, false)
            }
        }
    };
}

// Call the macro for each type.
impl_fmt!(CFD);
impl_fmt!(Order);
impl_fmt!(Spot);
impl_fmt!(Futures);
impl_fmt!(Options);
impl_fmt!(Swap);
impl_fmt!(ParentOrder);
impl_fmt!(ChildOrder);
