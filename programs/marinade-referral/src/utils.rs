#![allow(unused_imports)]

use anchor_lang::prelude::*;

use crate::fees::Fee;

//-----------------------------------------------------
pub fn get_shared_amount(fee: Fee, accumulated_amount: u64) -> u64 {
    (accumulated_amount * fee.basis_points as u64 / 100) as u64
}
