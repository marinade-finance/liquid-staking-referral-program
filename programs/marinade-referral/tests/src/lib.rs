#![cfg_attr(not(debug_assertions), deny(warnings))]

use marinade_finance_offchain_sdk::marinade_finance;
use marinade_finance_offchain_sdk::solana_sdk::pubkey::Pubkey;
use solana_program_test::{processor, ProgramTest};

mod integration_test;

pub mod initialize;

pub fn program_test() -> ProgramTest {
    ProgramTest::new(
        "marinade_finance",
        marinade_finance::ID,
        processor!(marinade_finance::test_entry),
    )
}

pub fn find_value<T, F: FnMut() -> Option<T>>(mut gen: F) -> T {
    loop {
        if let Some(update) = gen() {
            break update;
        }
    }
}

pub fn change_value<T: Eq, F: FnMut() -> T>(old: T, mut gen: F) -> T {
    find_value(|| {
        let update = gen();
        if update != old {
            Some(update)
        } else {
            None
        }
    })
}
