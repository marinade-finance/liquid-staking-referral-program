#![allow(unused_imports)]
use crate::{initialize::InitializeInputWithSeeds, integration_test::IntegrationTest};

use marinade_finance_offchain_sdk::anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;
use marinade_finance_offchain_sdk::marinade_finance;
use marinade_finance_offchain_sdk::marinade_finance::stake_wrapper::StakeWrapper;
use marinade_finance_offchain_sdk::spl_associated_token_account::{
    create_associated_token_account, get_associated_token_address,
};
use marinade_finance_offchain_sdk::{
    instruction_helpers::InstructionHelpers,
    marinade_finance::{ticket_account::TicketAccountData, State},
};
use rand::SeedableRng;
use rand_chacha::ChaChaRng;
use solana_sdk::stake;
use solana_sdk::stake::instruction::LockupArgs;
use solana_sdk::stake::state::Lockup;
use solana_sdk::{
    native_token::sol_to_lamports,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};
use std::sync::Arc;
use test_env_log::test;

use log::*;

#[test(tokio::test)]
async fn test_deposit_stake_account() -> anyhow::Result<()> {
    let mut rng = ChaChaRng::from_seed([
        251, 27, 213, 24, 63, 4, 35, 210, 233, 49, 94, 40, 61, 129, 65, 172, 150, 130, 12, 111, 5,
        240, 205, 45, 216, 97, 86, 180, 9, 102, 96, 212,
    ]);
    let input = InitializeInputWithSeeds::random(&mut rng);
    let mut test = IntegrationTest::start(&input).await?;

    let validator = Arc::new(Keypair::generate(&mut rng));
    let vote = Arc::new(Keypair::generate(&mut rng));

    test.add_validator(validator, vote.clone(), 0x100);

    let simple_stake = test
        .create_activated_stake_account(&vote.pubkey(), 10 * LAMPORTS_PER_SOL)
        .await;
    let user_msol = test.builder.create_associated_token_account(
        &test.fee_payer(),
        &test.state.msol_mint,
        "mSOL",
    )?;
    let simple_stake_state: StakeWrapper = test.get_account_data(&simple_stake.pubkey()).await;

    test.builder.deposit_stake_account(
        &test.state,
        simple_stake.pubkey(),
        test.fee_payer_signer(),
        user_msol,
        0,
        vote.pubkey(),
        test.fee_payer_signer(),
    );
    test.execute().await;
    assert_eq!(
        test.get_token_balance_or_zero(&user_msol).await,
        simple_stake_state.delegation().unwrap().stake
    );
    let stake_with_extra_lamports = test
        .create_activated_stake_account(&vote.pubkey(), 10 * LAMPORTS_PER_SOL)
        .await;
    test.builder.transfer_lamports(
        test.fee_payer_signer(),
        &stake_with_extra_lamports.pubkey(),
        1,
        "user",
        "stake",
    )?;
    test.builder.deposit_stake_account(
        &test.state,
        stake_with_extra_lamports.pubkey(),
        test.fee_payer_signer(),
        user_msol,
        0,
        vote.pubkey(),
        test.fee_payer_signer(),
    );
    test.try_execute()
        .await
        .expect_err("Must not accept extra lamports on balance");

    let stake_with_lockup = test
        .create_activated_stake_account(&vote.pubkey(), 10 * LAMPORTS_PER_SOL)
        .await;
    let lockup_epoch = test.get_clock().await.epoch + 1;
    test.builder.add_instruction(
        stake::instruction::set_lockup(
            &stake_with_lockup.pubkey(),
            &LockupArgs {
                unix_timestamp: Some(0),
                epoch: Some(lockup_epoch),
                custodian: Some(Pubkey::new_unique()),
            },
            &test.fee_payer(),
        ),
        format!("Set lockup"),
    )?;
    test.builder.deposit_stake_account(
        &test.state,
        stake_with_lockup.pubkey(),
        test.fee_payer_signer(),
        user_msol,
        0,
        vote.pubkey(),
        test.fee_payer_signer(),
    );
    test.try_execute()
        .await
        .expect_err("Must not accept locked up stake");
    test.move_to_next_epoch().await;

    test.builder.deposit_stake_account(
        &test.state,
        stake_with_lockup.pubkey(),
        test.fee_payer_signer(),
        user_msol,
        0,
        vote.pubkey(),
        test.fee_payer_signer(),
    );
    test.execute().await;

    let stake_with_lockup: StakeWrapper = test.get_account_data(&simple_stake.pubkey()).await;
    assert_eq!(stake_with_lockup.lockup().unwrap(), Lockup::default());

    Ok(())
}
