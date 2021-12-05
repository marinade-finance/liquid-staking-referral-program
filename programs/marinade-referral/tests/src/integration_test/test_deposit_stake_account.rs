#![allow(unused_imports)]
use crate::{
    initialize::InitializeInputWithSeeds,
    integration_test::{init_marinade_referral_test_globals, IntegrationTest, TestUser},
};

use marinade_finance_offchain_sdk::anchor_lang::InstructionData;
use marinade_finance_offchain_sdk::marinade_finance::state::StateHelpers;
use marinade_finance_offchain_sdk::anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;
use marinade_finance_offchain_sdk::marinade_finance;
use marinade_finance_offchain_sdk::marinade_finance::stake_wrapper::StakeWrapper;
use marinade_finance_offchain_sdk::anchor_lang::ToAccountMetas;
use marinade_finance_offchain_sdk::spl_associated_token_account::{
    create_associated_token_account, get_associated_token_address,
};
use marinade_finance_offchain_sdk::{
    instruction_helpers::InstructionHelpers,
    marinade_finance::{ticket_account::TicketAccountData, State, validator_system::ValidatorRecord},
    spl_token,
};
use rand::SeedableRng;
use rand_chacha::ChaChaRng;
use solana_sdk::stake;
use solana_sdk::stake::instruction::LockupArgs;
use solana_sdk::stake::state::Lockup;
use solana_sdk::{
    instruction::Instruction,
    native_token::sol_to_lamports,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction, system_program,
    transaction::Transaction,
    sysvar::{clock, epoch_schedule, rent, stake_history},
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

    // init referral-program
    let marinade_referral_test_globals = init_marinade_referral_test_globals(&mut test).await;

    let simple_stake = test
        .create_activated_stake_account(&vote.pubkey(), 10 * LAMPORTS_PER_SOL)
        .await;
    let user_msol = test.builder.create_associated_token_account(
        &test.fee_payer(),
        &test.state.msol_mint,
        "mSOL",
    )?;
    let simple_stake_state: StakeWrapper = test.get_account_data(&simple_stake.pubkey()).await;

    let tx = referral_deposit_stake_account_txn(
        simple_stake.pubkey(),
        test.fee_payer(),
        user_msol,
        0,
        vote.pubkey(),
        &mut test,
        marinade_referral_test_globals.referral_key,
    );

    // marinade-referral execution
    println!("marinade-referral deposit-stake-account");
    test.execute_txn(tx, vec![test.fee_payer_signer()])
        .await;

    // -----------------------
    // MARINADE-FINANCE direct call, commented
    // -----------------------
    // test.builder.deposit_stake_account(
    //     &test.state,
    //     simple_stake.pubkey(),
    //     test.fee_payer_signer(),
    //     user_msol,
    //     0,
    //     vote.pubkey(),
    //     test.fee_payer_signer(),
    // );
    // test.execute().await;

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

pub fn referral_deposit_stake_account_txn(
    stake_account: Pubkey,
    stake_authority: Pubkey,
    user_msol_account: Pubkey,
    validator_index: u32,
    validator_vote: Pubkey,
    test: &mut IntegrationTest,
    referral_key: Pubkey,
) -> Transaction {
    // -----------------------------------------
    // Create a referral DepositStakeAccount instruction.
    // -----------------------------------------
    let transfer_from = user_msol_account;
    let mint_to = user_msol_account;
    let state_key = test.state.key;

    let accounts = marinade_referral::accounts::DepositStakeAccount {
        state: test.state.key,
        validator_list: *test.state.validator_system.validator_list_address(),
        stake_list: *test.state.stake_system.stake_list_address(),
        stake_account,
        stake_authority,
        duplication_flag: ValidatorRecord::find_duplication_flag(&test.state.key, &validator_vote).0,
        rent_payer: test.fee_payer(),
        msol_mint: test.state.msol_mint,
        mint_to,
        msol_mint_authority: State::find_msol_mint_authority(&test.state.key).0,
        clock: clock::id(),
        rent: rent::id(),
        system_program: system_program::ID,
        token_program: spl_token::ID,
        stake_program: stake::program::ID,
        //----
        marinade_finance_program: marinade_finance::ID,
        referral_state: referral_key,
    }
    .to_account_metas(None);

    let ix_data = marinade_referral::instruction::DepositStakeAccount { validator_index };
    let deposit_stake_acc_instruction = Instruction {
        program_id: marinade_referral::marinade_referral::ID,
        accounts,
        data: ix_data.data(),
    };

    return Transaction::new_with_payer(&[deposit_stake_acc_instruction], Some(&test.fee_payer()));
}
