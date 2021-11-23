use anchor_lang::prelude::*;

use crate::{constant::*, error::*, processor::*, states::*};

// define instructions here

#[derive(Accounts)]
pub struct Initialize {}
