//
// Integration Test
// global state and referral state initialization
// RUSTFLAGS=-Awarnings cargo test test_state_initialization --manifest-path programs/marinade-referral/tests/Cargo.toml
//
use crate::{initialize::InitializeInputWithSeeds, integration_test::*};
use std::sync::Arc;

use rand::SeedableRng;
use rand_chacha::ChaChaRng;

use marinade_referral::constant::{
    DEFAULT_BASE_FEE_POINTS, DEFAULT_MAX_FEE_POINTS, DEFAULT_MAX_NET_STAKE,
    DEFAULT_OPERATION_FEE_POINTS, MAX_OPERATION_FEE_POINTS,
};
use solana_sdk::{
    native_token::LAMPORTS_PER_SOL,
    signature::{Keypair, Signer},
};
use test_env_log::test;

async fn init_test() -> anyhow::Result<(IntegrationTest, MarinadeReferralTestGlobals)> {
    let mut random = ChaChaRng::from_seed([
        248, 3, 94, 241, 228, 239, 32, 168, 219, 67, 27, 194, 26, 155, 140, 136, 154, 4, 40, 175,
        132, 80, 60, 31, 135, 250, 230, 19, 172, 106, 254, 120,
    ]);

    let input = InitializeInputWithSeeds::random(&mut random);
    let mut test = IntegrationTest::start(&input).await?;
    let marinade_referrals = init_marinade_referral_test_globals(&mut test).await;
    Ok((test, marinade_referrals))
}

#[test(tokio::test)]
async fn test_init_global_state() -> anyhow::Result<()> {
    let (mut test, marinade_referrals) = init_test().await?;

    let global_state: marinade_referral::states::GlobalState =
        get_account(&mut test, marinade_referrals.global_state_pubkey).await;
    let referral_state: marinade_referral::states::ReferralState =
        get_account(&mut test, marinade_referrals.partner_referral_state_pubkey).await;

    assert_eq!(
        marinade_referrals.admin_key.pubkey(),
        global_state.admin_account,
        "Global state 'admin key' does not match"
    );
    assert_eq!(
        test.state.msol_mint, global_state.msol_mint_account,
        "Global state 'treasury token account' key does not match"
    );

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
        DEFAULT_OPERATION_FEE_POINTS, referral_state.operation_deposit_sol_fee.basis_points,
        "Operation 'deposit sol fee' should be init at init value",
    );
    assert_eq!(
        DEFAULT_OPERATION_FEE_POINTS,
        referral_state
            .operation_deposit_stake_account_fee
            .basis_points,
        "Operation 'deposit stake account fee' should be init value",
    );
    assert_eq!(
        DEFAULT_OPERATION_FEE_POINTS, referral_state.operation_liquid_unstake_fee.basis_points,
        "Operation 'liquid unstake fee' should be init value",
    );
    assert_eq!(
        DEFAULT_OPERATION_FEE_POINTS, referral_state.operation_delayed_unstake_fee.basis_points,
        "Operation 'delayed unstake fee' should be init value",
    );

    Ok(())
}

#[test(tokio::test)]
async fn test_change_authority() -> anyhow::Result<()> {
    let (mut test, marinade_referrals) = init_test().await?;

    // changing authority to the same as it was before
    change_authority_execute(
        &mut test,
        marinade_referrals.global_state_pubkey,
        marinade_referrals.admin_key.pubkey(),
        marinade_referrals.admin_key.pubkey(),
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

    // changing authority to a new admin account
    let new_admin = Arc::new(Keypair::new());
    change_authority_execute(
        &mut test,
        marinade_referrals.global_state_pubkey,
        marinade_referrals.admin_key.pubkey(),
        new_admin.pubkey(),
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

    // changing authority to admin account that's not referred in the global state
    // (correctly signed but admin account does not match the saved value)
    let another_new_admin = Arc::new(Keypair::new());
    let txn_result = change_authority_execute(
        &mut test,
        marinade_referrals.global_state_pubkey,
        another_new_admin.pubkey(),
        marinade_referrals.admin_key.pubkey(),
        &another_new_admin,
    )
    .await;
    match txn_result {
        // https://github.com/coral-xyz/anchor/blob/v0.14.0/lang/src/error.rs
        Err(error_number) => assert_eq!(141, error_number, "A constraint should be violated"),
        _ => panic!("Expected the transaction fails with the contraint violation."),
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
    let (mut test, marinade_referrals) = init_test().await?;

    // updating only with the required values
    update_referral_execute(
        &mut test,
        marinade_referrals.global_state_pubkey,
        &marinade_referrals.admin_key,
        marinade_referrals.partner_referral_state_pubkey,
        marinade_referrals.partner.keypair.pubkey(),
        marinade_referrals.msol_partner_token_pubkey,
        true,
        None,
        None,
        None,
        None,
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
        Some(31),
        Some(32),
        Some(33),
        Some(MAX_OPERATION_FEE_POINTS as u8),
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
    assert_eq!(
        31, referral_state.operation_deposit_sol_fee.basis_points,
        "Referral state update 'partner account' should be changed",
    );
    assert_eq!(
        32,
        referral_state
            .operation_deposit_stake_account_fee
            .basis_points,
        "Referral state update 'partner account' should be changed",
    );
    assert_eq!(
        33, referral_state.operation_liquid_unstake_fee.basis_points,
        "Referral state update 'partner account' should be changed",
    );
    assert_eq!(
        MAX_OPERATION_FEE_POINTS, referral_state.operation_delayed_unstake_fee.basis_points,
        "Referral state update 'partner account' should be changed",
    );

    // updating with fee is above the permitted value
    let txn_result = update_referral_execute(
        &mut test,
        marinade_referrals.global_state_pubkey,
        &marinade_referrals.admin_key,
        marinade_referrals.partner_referral_state_pubkey,
        marinade_referrals.partner.keypair.pubkey(),
        marinade_referrals.msol_partner_token_pubkey,
        false,
        None,
        Some(MAX_OPERATION_FEE_POINTS as u8 + 1),
        None,
        None,
    )
    .await;
    match txn_result {
        // https://github.com/coral-xyz/anchor/blob/v0.14.0/lang/src/error.rs
        Err(error_number) => assert_eq!(
            307, error_number,
            "Contraint fee over max should be violated, expected error number 307"
        ),
        _ => panic!("Expected the transaction fails with the contraint violation."),
    }
    let referral_state: marinade_referral::states::ReferralState =
        get_account(&mut test, marinade_referrals.partner_referral_state_pubkey).await;
    assert_eq!(
        31, referral_state.operation_deposit_sol_fee.basis_points,
        "Referral state update 'partner account' should be changed",
    );

    Ok(())
}
