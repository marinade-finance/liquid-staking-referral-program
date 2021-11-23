use anchor_lang::prelude::*;

use crate::{constant::*, error::*, instructions::*, states::*};

pub fn process_initialize(ctx: Context<Initialize>) -> ProgramResult {
    Ok(())
}
