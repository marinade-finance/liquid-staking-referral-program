#![allow(unused_imports)]
use crate::integration_test::{
    get_account, init_marinade_referral_test_globals, update_referral_execute, IntegrationTest,
    MarinadeReferralTestGlobals, TestUser,
};

use marinade_finance_offchain_sdk::anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;
use marinade_finance_offchain_sdk::anchor_lang::InstructionData;
use marinade_finance_offchain_sdk::anchor_lang::ToAccountMetas;
use marinade_finance_offchain_sdk::marinade_finance;
use marinade_finance_offchain_sdk::marinade_finance::stake_wrapper::StakeWrapper;
use marinade_finance_offchain_sdk::marinade_finance::state::StateHelpers;
use marinade_finance_offchain_sdk::spl_associated_token_account::{
    create_associated_token_account, get_associated_token_address,
};
use marinade_finance_offchain_sdk::{
    instruction_helpers::InstructionHelpers,
    marinade_finance::{
        ticket_account::TicketAccountData, validator_system::ValidatorRecord, State,
    },
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
    sysvar::{clock, epoch_schedule, rent, stake_history},
    transaction::Transaction,
};
use std::sync::Arc;
use test_env_log::test;

use log::*;

async fn create_staked_validator(
    test: &mut IntegrationTest,
    rng: &mut ChaChaRng,
) -> anyhow::Result<(Arc<Keypair>, Arc<Keypair>, Pubkey)> {
    let validator = Arc::new(Keypair::generate(rng));
    let vote = Arc::new(Keypair::generate(rng));

    test.add_validator(validator, vote.clone(), 0x100);

    let simple_stake = test
        .create_activated_stake_account(&vote.pubkey(), 10 * LAMPORTS_PER_SOL)
        .await;
    let user_msol = test.builder.create_associated_token_account(
        &test.fee_payer(),
        &test.state.msol_mint,
        "mSOL",
    )?;
    // (create stake accounts, etc)
    test.execute().await;
    Ok((vote, simple_stake, user_msol))
}

#[test(tokio::test)]
async fn test_deposit_stake_account_with_fees() -> anyhow::Result<()> {
    let (mut test, marinade_referral_test_globals, mut rng) = IntegrationTest::init_test().await?;

    update_referral_execute(
        &mut test,
        marinade_referral_test_globals.global_state_pubkey,
        &marinade_referral_test_globals.admin_key,
        marinade_referral_test_globals.partner_referral_state_pubkey,
        marinade_referral_test_globals.partner.keypair.pubkey(),
        marinade_referral_test_globals.msol_partner_token_pubkey,
        false,
        Some(0),  // deposit sol
        Some(22), // deposit stake account
        Some(0),  // unstake liquid
        Some(0),  // unstake delayed
    )
    .await
    .unwrap();
    let referral_state: marinade_referral::states::ReferralState = get_account(
        &mut test,
        marinade_referral_test_globals.partner_referral_state_pubkey,
    )
    .await;
    assert!(
        referral_state
            .operation_deposit_stake_account_fee
            .basis_points
            > 0,
        "Expected fee for deposit stake account operation should be bigger than 0",
    );
    let partner_msol_balance_before = test
        .get_token_balance(&marinade_referral_test_globals.msol_partner_token_pubkey)
        .await;

    let (vote, simple_stake, user_msol) = create_staked_validator(&mut test, &mut rng).await?;
    let simple_stake_state: StakeWrapper = test.get_account_data(&simple_stake.pubkey()).await;

    let tx = referral_deposit_stake_account_txn(
        simple_stake.pubkey(),
        test.fee_payer(),
        user_msol,
        0,
        vote.pubkey(),
        &mut test,
        marinade_referral_test_globals.partner_referral_state_pubkey,
        marinade_referral_test_globals.msol_partner_token_pubkey,
    );
    println!("marinade-referral deposit-stake-account execution");
    test.execute_txn(tx, vec![test.fee_payer_signer()]).await;

    let operation_fee_in_lamports = simple_stake_state.delegation().unwrap().stake
        * referral_state
            .operation_deposit_stake_account_fee
            .basis_points as u64
        / 10_000;
    assert_eq!(
        test.get_token_balance_or_zero(&user_msol).await,
        simple_stake_state.delegation().unwrap().stake - operation_fee_in_lamports
    );
    assert_eq!(
        test.get_token_balance(&marinade_referral_test_globals.msol_partner_token_pubkey)
            .await,
        partner_msol_balance_before + operation_fee_in_lamports
    );

    Ok(())
}

#[test(tokio::test)]
async fn test_deposit_stake_account_wrong_referral() -> anyhow::Result<()> {
    let (mut test, marinade_referral_test_globals, mut rng) = IntegrationTest::init_test().await?;
    let (vote, simple_stake, user_msol) = create_staked_validator(&mut test, &mut rng).await?;

    let tx = referral_deposit_stake_account_txn(
        simple_stake.pubkey(),
        test.fee_payer(),
        user_msol,
        0,
        vote.pubkey(),
        &mut test,
        marinade_referral_test_globals.partner_referral_state_pubkey,
        user_msol,
    );
    println!("marinade-referral deposit-stake-account execution");
    let deposit_stake_account_result = test
        .try_execute_txn(tx, vec![test.fee_payer_signer()])
        .await;
    match deposit_stake_account_result {
        Ok(_) => panic!("Expected error happens when user want to be a refferal"),
        Err(number) => {
            // anchor 0.14.0 : error 152 : ConstraintAddress
            assert_eq!(
                152, number,
                "Expected anchor error 'An address constraint was violated'"
            );
        }
    }
    Ok(())
}

#[test(tokio::test)]
async fn test_deposit_stake_account() -> anyhow::Result<()> {
    let (mut test, marinade_referral_test_globals, mut rng) = IntegrationTest::init_test().await?;
    // no fees for the referral operations
    marinade_referral_test_globals
        .set_no_operation_fees(&mut test)
        .await;

    let (vote, simple_stake, user_msol) = create_staked_validator(&mut test, &mut rng).await?;
    let simple_stake_state: StakeWrapper = test.get_account_data(&simple_stake.pubkey()).await;

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
    //

    // -----------------------
    // REFERRAL-call
    // -----------------------
    let tx = referral_deposit_stake_account_txn(
        simple_stake.pubkey(),
        test.fee_payer(),
        user_msol,
        0,
        vote.pubkey(),
        &mut test,
        marinade_referral_test_globals.partner_referral_state_pubkey,
        marinade_referral_test_globals.msol_partner_token_pubkey,
    );

    // marinade-referral execution
    println!("marinade-referral deposit-stake-account");
    test.execute_txn(tx, vec![test.fee_payer_signer()]).await;

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
    // execute prepared transactions
    // (transfer_lamports)
    test.execute().await;

    // DIRECT-CALL, commented
    // test.builder.deposit_stake_account(
    //     &test.state,
    //     stake_with_extra_lamports.pubkey(),
    //     test.fee_payer_signer(),
    //     user_msol,
    //     0,
    //     vote.pubkey(),
    //     test.fee_payer_signer(),
    // );

    // -----------------------
    // REFERRAL-call
    // -----------------------
    let tx = referral_deposit_stake_account_txn(
        stake_with_extra_lamports.pubkey(),
        test.fee_payer(),
        user_msol,
        0,
        vote.pubkey(),
        &mut test,
        marinade_referral_test_globals.partner_referral_state_pubkey,
        marinade_referral_test_globals.msol_partner_token_pubkey,
    );

    // marinade-referral execution
    println!("marinade-referral deposit-stake-account");
    test.try_execute_txn(tx, vec![test.fee_payer_signer()])
        .await
        .expect_err("Must not accept extra lamports on balance");

    // DIRECT call, commented
    // test.try_execute()
    //     .await
    //     .expect_err("Must not accept extra lamports on balance");

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
    // execute prepared transactions
    // (transfer_lamports)
    test.execute().await;

    // DIRECT-CALL, commented
    // test.builder.deposit_stake_account(
    //     &test.state,
    //     stake_with_lockup.pubkey(),
    //     test.fee_payer_signer(),
    //     user_msol,
    //     0,
    //     vote.pubkey(),
    //     test.fee_payer_signer(),
    // );
    // test.try_execute()
    //     .await
    //     .expect_err("Must not accept locked up stake");

    // -----------------------
    // REFERRAL-call
    // -----------------------
    let tx = referral_deposit_stake_account_txn(
        stake_with_lockup.pubkey(),
        test.fee_payer(),
        user_msol,
        0,
        vote.pubkey(),
        &mut test,
        marinade_referral_test_globals.partner_referral_state_pubkey,
        marinade_referral_test_globals.msol_partner_token_pubkey,
    );

    // marinade-referral execution
    println!("marinade-referral deposit-stake-account");
    test.try_execute_txn(tx, vec![test.fee_payer_signer()])
        .await
        .expect_err("Must not accept locked up stake");

    test.move_to_next_epoch().await;
    // execute prepared transactions
    test.execute().await;

    // DIRECT-CALL, commented
    // test.builder.deposit_stake_account(
    //     &test.state,
    //     stake_with_lockup.pubkey(),
    //     test.fee_payer_signer(),
    //     user_msol,
    //     0,
    //     vote.pubkey(),
    //     test.fee_payer_signer(),
    // );
    // test.execute().await;

    // -----------------------
    // REFERRAL-call
    // -----------------------
    let tx = referral_deposit_stake_account_txn(
        stake_with_lockup.pubkey(),
        test.fee_payer(),
        user_msol,
        0,
        vote.pubkey(),
        &mut test,
        marinade_referral_test_globals.partner_referral_state_pubkey,
        marinade_referral_test_globals.msol_partner_token_pubkey,
    );
    // marinade-referral execution
    println!("marinade-referral deposit-stake-account");
    test.execute_txn(tx, vec![test.fee_payer_signer()]).await;

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
    msol_token_partner_account: Pubkey,
) -> Transaction {
    // -----------------------------------------
    // Create a referral DepositStakeAccount instruction.
    // -----------------------------------------

    let accounts = marinade_referral::accounts::DepositStakeAccount {
        state: test.state.key,
        validator_list: *test.state.validator_system.validator_list_address(),
        stake_list: *test.state.stake_system.stake_list_address(),
        stake_account,
        stake_authority,
        duplication_flag: ValidatorRecord::find_duplication_flag(&test.state.key, &validator_vote)
            .0,
        rent_payer: test.fee_payer(),
        msol_mint: test.state.msol_mint,
        mint_to: user_msol_account,
        msol_mint_authority: State::find_msol_mint_authority(&test.state.key).0,
        clock: clock::id(),
        rent: rent::id(),
        system_program: system_program::ID,
        token_program: spl_token::ID,
        stake_program: stake::program::ID,
        //----
        marinade_finance_program: marinade_finance::ID,
        referral_state: referral_key,
        msol_token_partner_account,
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
