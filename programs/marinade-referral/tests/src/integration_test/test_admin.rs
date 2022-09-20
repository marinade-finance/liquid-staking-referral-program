//
// Integration Test
// global state and referral state initialization
// RUSTFLAGS=-Awarnings cargo test test_state_initialization --manifest-path programs/marinade-referral/tests/Cargo.toml
//
use crate::{initialize::InitializeInputWithSeeds, integration_test::*};
use std::sync::Arc;

use rand::SeedableRng;
use rand_chacha::ChaChaRng;

use marinade_finance_offchain_sdk::spl_token::solana_program;
use marinade_referral::constant::{
    DEFAULT_BASE_FEE_POINTS, DEFAULT_MAX_FEE_POINTS, DEFAULT_MAX_NET_STAKE,
    DEFAULT_OPERATION_FEE_POINTS, MAX_OPERATION_FEE_POINTS,
};
use solana_sdk::{
    native_token::LAMPORTS_PER_SOL,
    signature::{Keypair, Signer},
};
use test_env_log::test;

#[test(tokio::test)]
async fn test_init_global_state() -> anyhow::Result<()> {
    let (mut test, marinade_referrals, _) = IntegrationTest::init_test().await?;

    let global_state: marinade_referral::states::GlobalState =
        get_account(&mut test, marinade_referrals.global_state_pubkey).await;
    let referral_state: marinade_referral::states::ReferralState =
        get_account(&mut test, marinade_referrals.partner_referral_state_pubkey).await;

    // GLOBAL STATE
    assert_eq!(
        marinade_referrals.admin_key.pubkey(),
        global_state.admin_account,
        "Global state 'admin key' does not match"
    );
    assert_eq!(
        solana_program::system_program::ID,
        global_state.foreman_1,
        "Global state 'foreman_1' pubkey does not match"
    );
    assert_eq!(
        solana_program::system_program::ID,
        global_state.foreman_2,
        "Global state 'foreman_2' pubkey does not match"
    );
    assert_eq!(
        test.state.msol_mint, global_state.msol_mint_account,
        "Global state 'treasury token account' key does not match"
    );
    // REFERRAL STATE
    assert_eq!(
        "TEST_PART", referral_state.partner_name,
        "Refferal state 'partner name' does not match"
    );
    assert_eq!(
        marinade_referrals.partner.keypair.pubkey(),
        referral_state.partner_account,
        "Referral state 'partner account' does not match",
    );
    assert_eq!(
        marinade_referrals.msol_partner_token_pubkey, referral_state.msol_token_partner_account,
        "Referral state 'partner token account' does not match",
    );
    assert!(
        !referral_state.pause,
        "Account init value of the 'pause' should be false",
    );
    assert_eq!(
        DEFAULT_MAX_NET_STAKE, referral_state.max_net_stake,
        "Referral state 'max net stake' init value does not correspond to default value",
    );
    assert_eq!(
        DEFAULT_BASE_FEE_POINTS, referral_state.base_fee,
        "Referral state 'base fee points' init value does not correspond to default value",
    );
    assert_eq!(
        DEFAULT_MAX_FEE_POINTS, referral_state.max_fee,
        "Referral state 'max fee points' init value does not correspond to default value",
    );
    assert_eq!(
        DEFAULT_OPERATION_FEE_POINTS, referral_state.operation_deposit_sol_fee,
        "Operation 'deposit sol fee' should be init at init value",
    );
    assert_eq!(
        DEFAULT_OPERATION_FEE_POINTS, referral_state.operation_deposit_stake_account_fee,
        "Operation 'deposit stake account fee' should be init value",
    );
    assert_eq!(
        DEFAULT_OPERATION_FEE_POINTS, referral_state.operation_liquid_unstake_fee,
        "Operation 'liquid unstake fee' should be init value",
    );
    assert_eq!(
        DEFAULT_OPERATION_FEE_POINTS, referral_state.operation_delayed_unstake_fee,
        "Operation 'delayed unstake fee' should be init value",
    );
    assert_eq!(
        0, referral_state.accum_deposit_sol_fees,
        "Accumulator 'deposit sol fee' should be init at 0",
    );
    assert_eq!(
        0, referral_state.accum_deposit_stake_account_fee,
        "Accumulator 'deposit stake account fee' should be init at 0",
    );
    assert_eq!(
        0, referral_state.accum_liquid_unstake_fee,
        "Accumulator 'liquid unstake fee' should be init at 0",
    );
    assert_eq!(
        0, referral_state.accum_delayed_unstake_fee,
        "Accumulator 'delayed unstake fee' should be init at 0",
    );

    Ok(())
}

#[test(tokio::test)]
async fn test_init_referral_state_wrong_msol_account() -> anyhow::Result<()> {
    let mut rng = ChaChaRng::from_seed(rand::random());
    let input = InitializeInputWithSeeds::random(&mut rng);
    let mut test = IntegrationTest::start(&input).await?;

    let (global_state_pubkey, admin) = create_global_state_account(&mut test, None, None).await;
    // creating MSOL account that is not owned by partner account
    let msol_account = admin
        .get_or_create_msol_account_instruction(&mut test)
        .await;
    test.execute().await;

    let partner = test
        .create_test_user("partner", 200 * LAMPORTS_PER_SOL)
        .await;
    let txn_result = create_referral_state_account(
        &mut test,
        &partner,
        global_state_pubkey,
        &admin.keypair,
        msol_account.pubkey,
    )
    .await;
    match txn_result {
        Err(error_number) => assert_eq!(
            303, error_number,
            "Constraint of invalid partner account as msol is not owned to partner was expected"
        ),
        _ => panic!("Expected the transaction fails with the constraint violation."),
    }
    Ok(())
}

#[test(tokio::test)]
async fn test_double_referral_state_initialization() -> anyhow::Result<()> {
    let (mut test, marinade_referrals, _) = IntegrationTest::init_test().await?;
    // referral_state exists
    let referral_state: marinade_referral::states::ReferralState =
        get_account(&mut test, marinade_referrals.partner_referral_state_pubkey).await;
    assert!(!referral_state.pause, "Referral state account should exist and not paused");


    let accounts = marinade_referral::accounts::InitReferralAccount {
        global_state: marinade_referrals.global_state_pubkey,
        signer: marinade_referrals.admin_key.pubkey(),
        referral_state: marinade_referrals.partner_referral_state_pubkey,
        partner_account: marinade_referrals.partner.keypair.pubkey(),
        msol_token_partner_account: marinade_referrals.msol_partner_token_pubkey,
    };
    let ix_data = marinade_referral::instruction::InitReferralAccount {
        partner_name: "FAILING".into(),
    };
    let instruction = Instruction {
        program_id: marinade_referral::marinade_referral::ID,
        accounts: accounts.to_account_metas(None),
        data: ix_data.data(),
    };
    let txn_result = test.try_execute_instruction(instruction, vec![test.fee_payer_signer(), marinade_referrals.admin_key.clone()])
        .await;
    match txn_result {
        // https://github.com/coral-xyz/anchor/blob/v0.14.0/lang/src/error.rs
        Err(error_number) => assert_eq!(
            153, error_number, "Expected zero account discriminant"
        ),
        _ => panic!("Expected the transaction fails with the constraint violation."),
    }
    Ok(())
}

async fn init_test_with_foreman() -> anyhow::Result<(IntegrationTest, Pubkey, TestUser, TestUser, TestUser)> {
    let mut rng = ChaChaRng::from_seed(rand::random());
    let input = InitializeInputWithSeeds::random(&mut rng);
    let mut test = IntegrationTest::start(&input).await?;

    let foreman_1 = test.create_test_user("foreman1", 1_000).await;
    let foreman_2 = test.create_test_user("foreman2", 2_000).await;
    let (global_state_pubkey, admin) = create_global_state_account(
        &mut test,
        Some(foreman_1.keypair.pubkey()),
        Some(foreman_2.keypair.pubkey()),
    )
    .await;
    let global_state: marinade_referral::states::GlobalState =
        get_account(&mut test, global_state_pubkey).await;
    assert_eq!(
        foreman_1.keypair.pubkey(),
        global_state.foreman_1,
        "Global state 'foreman_1' pubkey does not match"
    );
    assert_eq!(
        foreman_2.keypair.pubkey(),
        global_state.foreman_2,
        "Global state 'foreman_2' pubkey does not match"
    );
    Ok((test, global_state_pubkey, admin, foreman_1, foreman_2))
}

#[test(tokio::test)]
async fn test_init_referral_state_with_foreman() -> anyhow::Result<()> {
    let (mut test, global_state_pubkey, admin, foreman_1, foreman_2) = init_test_with_foreman().await?;

    // may the admin and foremen to create the referral accounts?
    let lamports = 200 * LAMPORTS_PER_SOL;
    let partner_1 = test.create_test_user("partner_1", lamports).await;
    let partner_1_msol_acc = partner_1
        .get_or_create_msol_account_instruction(&mut test)
        .await;
    test.execute().await;
    create_referral_state_account(
        &mut test,
        &partner_1,
        global_state_pubkey,
        &foreman_1.keypair,
        partner_1_msol_acc.pubkey,
    )
    .await
    .unwrap();
    let partner_2 = test.create_test_user("partner_2", lamports).await;
    let partner_2_msol_acc = partner_2
        .get_or_create_msol_account_instruction(&mut test)
        .await;
    test.execute().await;
    create_referral_state_account(
        &mut test,
        &partner_2,
        global_state_pubkey,
        &foreman_2.keypair,
        partner_2_msol_acc.pubkey,
    )
    .await
    .unwrap();
    let partner_3 = test.create_test_user("partner_3", lamports).await;
    let partner_3_msol_acc = partner_3
        .get_or_create_msol_account_instruction(&mut test)
        .await;
    test.execute().await;
    create_referral_state_account(
        &mut test,
        &partner_3,
        global_state_pubkey,
        &admin.keypair,
        partner_3_msol_acc.pubkey,
    )
    .await
    .unwrap();
    // any other account should be not permitted to execute (verification based on error code)
    let saboteur = test.create_test_user("saboteur", 1).await;
    let txn_result = create_referral_state_account(
        &mut test,
        &partner_3,
        global_state_pubkey,
        &saboteur.keypair,
        partner_3_msol_acc.pubkey,
    )
    .await;
    match txn_result {
        // https://github.com/coral-xyz/anchor/blob/v0.14.0/lang/src/error.rs
        Err(error_number) => assert_eq!(
            143, error_number, "Expected signer constraint violated"
        ),
        _ => panic!("Expected the transaction fails with the constraint violation."),
    }

    Ok(())
}

#[test(tokio::test)]
async fn test_change_authority() -> anyhow::Result<()> {
    let (mut test, marinade_referrals, _) = IntegrationTest::init_test().await?;

    // changing authority to the same as it was before
    change_authority_execute(
        &mut test,
        marinade_referrals.global_state_pubkey,
        marinade_referrals.admin_key.pubkey(),
        marinade_referrals.admin_key.pubkey(),
        solana_program::system_program::ID,
        solana_program::sysvar::clock::ID,
        &marinade_referrals.admin_key,
    )
    .await
    .unwrap();
    let global_state: marinade_referral::states::GlobalState =
        get_account(&mut test, marinade_referrals.global_state_pubkey).await;
    assert_eq!(
        marinade_referrals.admin_key.pubkey(),
        global_state.admin_account,
        "Global state admin key does not match after change authority"
    );
    assert_eq!(
        solana_program::system_program::ID,
        global_state.foreman_1,
        "Global state foreman_1 does not match after authority 'no change'"
    );
    assert_eq!(
        solana_program::sysvar::clock::ID,
        global_state.foreman_2,
        "Global state foreman does not match after authority 'no change'"
    );

    // changing authority to a new admin account
    let new_admin = Arc::new(Keypair::new());
    let new_foreman_1 = Pubkey::new_unique();
    let new_foreman_2 = Pubkey::new_unique();
    change_authority_execute(
        &mut test,
        marinade_referrals.global_state_pubkey,
        marinade_referrals.admin_key.pubkey(),
        new_admin.pubkey(),
        new_foreman_1,
        new_foreman_2,
        &marinade_referrals.admin_key,
    )
    .await
    .unwrap();
    let global_state: marinade_referral::states::GlobalState =
        get_account(&mut test, marinade_referrals.global_state_pubkey).await;
    assert_eq!(
        new_admin.pubkey(),
        global_state.admin_account,
        "Global state admin key does not match to new admin after changing authority"
    );
    assert_eq!(
        new_foreman_1,
        global_state.foreman_1,
        "Global state foreman_1 does not match after changing new admin'"
    );
    assert_eq!(
        new_foreman_2,
        global_state.foreman_2,
        "Global state foreman_2 does not match after changing new admin'"
    );

    // changing authority to admin account that's not referred in the global state
    // (correctly signed but admin account does not match the saved value)
    let another_new_admin = Arc::new(Keypair::new());
    let txn_result = change_authority_execute(
        &mut test,
        marinade_referrals.global_state_pubkey,
        another_new_admin.pubkey(),
        marinade_referrals.admin_key.pubkey(),
        solana_program::system_program::ID,
        solana_program::system_program::ID,
        &another_new_admin,
    )
    .await;
    match txn_result {
        // https://github.com/coral-xyz/anchor/blob/v0.14.0/lang/src/error.rs
        Err(error_number) => assert_eq!(141, error_number, "A constraint should be violated"),
        _ => panic!("Expected the transaction fails with the constraint violation."),
    }

    let global_state: marinade_referral::states::GlobalState =
        get_account(&mut test, marinade_referrals.global_state_pubkey).await;
    assert_eq!(
        new_admin.pubkey(),
        global_state.admin_account,
        "Global state admin key does not match after the changing authority failed"
    );

    Ok(())
}

#[test(tokio::test)]
async fn test_update_referral() -> anyhow::Result<()> {
    let (mut test, marinade_referrals, _) = IntegrationTest::init_test().await?;

    // updating only with the required values
    update_referral_execute(
        &mut test,
        marinade_referrals.global_state_pubkey,
        &marinade_referrals.admin_key,
        marinade_referrals.partner_referral_state_pubkey,
        marinade_referrals.partner.keypair.pubkey(),
        marinade_referrals.msol_partner_token_pubkey,
        true,
    )
    .await
    .unwrap();
    let referral_state: marinade_referral::states::ReferralState =
        get_account(&mut test, marinade_referrals.partner_referral_state_pubkey).await;
    assert!(
        referral_state.pause,
        "Referral state update 'pause' value should be true",
    );

    // updating with optional values
    let new_partner = test
        .create_test_user("test_referral_partner", LAMPORTS_PER_SOL)
        .await;
    // partner token account
    let new_token_partner_account = new_partner
        .get_or_create_msol_account_instruction(&mut test)
        .await;
    test.execute().await; // execute if the ATA needed to be created
    update_referral_execute(
        &mut test,
        marinade_referrals.global_state_pubkey,
        &marinade_referrals.admin_key,
        marinade_referrals.partner_referral_state_pubkey,
        new_partner.keypair.pubkey(),
        new_token_partner_account.pubkey,
        false,

    )
    .await
    .unwrap();
    let referral_state: marinade_referral::states::ReferralState =
        get_account(&mut test, marinade_referrals.partner_referral_state_pubkey).await;
    assert!(
        !referral_state.pause,
        "Referral state update 'pause' value should be false",
    );
    assert_eq!(
        new_partner.keypair.pubkey(),
        referral_state.partner_account,
        "Referral state update 'partner account' should be changed",
    );
    assert_eq!(
        new_token_partner_account.pubkey, referral_state.msol_token_partner_account,
        "Referral state update 'msol token partner account' should be changed",
    );

    Ok(())
}

#[test(tokio::test)]
async fn test_update_operation_fees() -> anyhow::Result<()> {
    let (mut test, marinade_referrals, _) = IntegrationTest::init_test().await?;

    update_operation_fees(
        &mut test,
        marinade_referrals.global_state_pubkey,
        &marinade_referrals.admin_key,
        marinade_referrals.partner_referral_state_pubkey,
        Some(31),
        Some(32),
        Some(33),
        Some(MAX_OPERATION_FEE_POINTS as u8),
    )
    .await
    .unwrap();
    let referral_state: marinade_referral::states::ReferralState =
        get_account(&mut test, marinade_referrals.partner_referral_state_pubkey).await;

    assert_eq!(
        31, referral_state.operation_deposit_sol_fee,
        "Referral state update 'partner account' should be changed",
    );
    assert_eq!(
        32, referral_state.operation_deposit_stake_account_fee,
        "Referral state update 'partner account' should be changed",
    );
    assert_eq!(
        33, referral_state.operation_liquid_unstake_fee,
        "Referral state update 'partner account' should be changed",
    );
    assert_eq!(
        MAX_OPERATION_FEE_POINTS, referral_state.operation_delayed_unstake_fee,
        "Referral state update 'partner account' should be changed",
    );

    let txn_result = update_operation_fees(
        &mut test,
        marinade_referrals.global_state_pubkey,
        &marinade_referrals.admin_key,
        marinade_referrals.partner_referral_state_pubkey,
        None,
        Some(MAX_OPERATION_FEE_POINTS as u8 + 1),
        None,
        None,
    )
    .await;

    match txn_result {
        Err(error_number) => assert_eq!(
            307, error_number,
            "Constraint fee over max should be violated"
        ),
        _ => panic!("Expected the transaction fails with the constraint violation."),
    }
    let referral_state: marinade_referral::states::ReferralState =
        get_account(&mut test, marinade_referrals.partner_referral_state_pubkey).await;
    assert_eq!(
        31+32+33,
        referral_state.operation_deposit_sol_fee +
        referral_state.operation_deposit_stake_account_fee +
        referral_state.operation_liquid_unstake_fee,
        "Check that referral state fees have not changed after wrong update failed",
    );

    Ok(())
}

#[test(tokio::test)]
async fn test_update_operation_fee_with_foreman() -> anyhow::Result<()> {
    let (mut test, global_state_pubkey, admin, foreman_1, foreman_2) = init_test_with_foreman().await?;
    let partner = test.create_test_user("partner", 200 * LAMPORTS_PER_SOL).await;
    let partner_msol_acc = partner
        .get_or_create_msol_account_instruction(&mut test)
        .await;
    test.execute().await;
    let partner_referral_state_pubkey = create_referral_state_account(
        &mut test,
        &partner,
        global_state_pubkey,
        &admin.keypair,
        partner_msol_acc.pubkey,
    )
    .await
    .unwrap();

    // may the foreman change the operation fees?
    update_operation_fees(
        &mut test,
        global_state_pubkey,
        &foreman_1.keypair,
        partner_referral_state_pubkey,
        Some(1),
        Some(2),
        Some(3),
        Some(4),
    )
    .await.unwrap();
    let referral_state: marinade_referral::states::ReferralState =
        get_account(&mut test, partner_referral_state_pubkey).await;
    assert_eq!(
        1+2+3+4,
        referral_state.operation_deposit_sol_fee +
        referral_state.operation_deposit_stake_account_fee +
        referral_state.operation_liquid_unstake_fee +
        referral_state.operation_delayed_unstake_fee,
        "Check that referral state fees changed with foreman_1 keypair",
    );
    update_operation_fees(
        &mut test,
        global_state_pubkey,
        &foreman_2.keypair,
        partner_referral_state_pubkey,
        Some(11),
        Some(12),
        Some(13),
        Some(14),
    )
    .await.unwrap();
    let referral_state: marinade_referral::states::ReferralState =
        get_account(&mut test, partner_referral_state_pubkey).await;
    assert_eq!(
        11+12+13+14,
        referral_state.operation_deposit_sol_fee +
        referral_state.operation_deposit_stake_account_fee +
        referral_state.operation_liquid_unstake_fee +
        referral_state.operation_delayed_unstake_fee,
        "Check that referral state fees changed with foreman_2 keypair",
    );
    // any other account should be not permitted to change fees (error code verification)
    let saboteur = test.create_test_user("saboteur", 1).await;
    let txn_result = update_operation_fees(
        &mut test,
        global_state_pubkey,
        &saboteur.keypair,
        partner_referral_state_pubkey,
        Some(1),
        Some(0),
        Some(0),
        Some(0),
    )
    .await;
    match txn_result {
        // https://github.com/coral-xyz/anchor/blob/v0.14.0/lang/src/error.rs
        Err(error_number) => assert_eq!(
            143, error_number, "Expected signer constraint violated"
        ),
        _ => panic!("Expected the transaction fails with the constraint violation."),
    }

    Ok(())
}
