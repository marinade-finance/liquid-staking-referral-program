#![allow(unused_imports)]

use anchor_lang::prelude::*;

use crate::{constant::*, error::*, fees::Fee, instructions::*, states::*};

pub fn process_update(ctx: Context<Update>, transfer_duration: u32, pause: bool) -> ProgramResult {
    ctx.accounts.state.transfer_duration = transfer_duration;
    ctx.accounts.state.pause = pause;

    Ok(())
}
