#![allow(unused_imports)]

use anchor_lang::prelude::*;

use crate::{constant::*, error::*, instructions::*, states::*};

pub fn process_initialize(_: Context<Initialize>) -> ProgramResult {
    Ok(())
}
