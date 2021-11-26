#![allow(unused_imports)]

use anchor_lang::prelude::*;

use crate::{constant::*, error::*, fees::Fee, instructions::*, states::*};

pub fn process_pause(ctx: Context<Pause>, pause: bool) -> ProgramResult {
    ctx.accounts.state.pause = pause;

    Ok(())
}
