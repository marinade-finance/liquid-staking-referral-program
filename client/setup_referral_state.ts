import * as anchor from "@project-serum/anchor";
import { Program, web3 } from "@project-serum/anchor";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  Token,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { MarinadeUtils } from "@marinade.finance/marinade-ts-sdk";
import { exit } from "process";
import { getMintClient } from "@marinade.finance/marinade-ts-sdk/dist/util";
import {
  ADMIN_KEYPAIR,
  ADMIN_PUBKEY,
  GLOBAL_STATE_PUBKEY,
  MSOL_MINT_PUBKEY,
  PARTNER_ID,
  PARTNER_NAME,
  program,
  provider,
  REFERRAL_TEST_KEYPAIR,
  REFERRAL_TEST_PUBKEY,
} from "./constants";
import { sleep } from "./util";

const { Keypair, SystemProgram, PublicKey, SYSVAR_RENT_PUBKEY } = web3;

export async function delete_referral_state() {
  // check if referralAccount exists
  const referralAccountBalance = await provider.connection.getBalance(
    REFERRAL_TEST_PUBKEY
  );
  console.log("referralAccountBalance", referralAccountBalance);
  if (referralAccountBalance !== 0) {
    // delete referral state
    program.rpc.deleteProgramAccount({
      accounts: {
        accountToDelete: REFERRAL_TEST_PUBKEY,
        beneficiary: provider.wallet.publicKey,
      },
    });
    await sleep(5000);
  }
}

export async function setup_referral_state() {
  const tx = new web3.Transaction();
  // check if referralAccount exists
  const referralAccountBalance = await provider.connection.getBalance(
    REFERRAL_TEST_PUBKEY
  );
  console.log("referralAccountBalance", referralAccountBalance);
  if (referralAccountBalance === 0) {
    // need to create

    // beneficiary - mSOL ATA for partner
    const {
      associatedTokenAccountAddress: beneficiaryATA,
      createAssociateTokenInstruction,
    } = await MarinadeUtils.getOrCreateAssociatedTokenAccount(
      provider,
      MSOL_MINT_PUBKEY,
      PARTNER_ID
    );
    console.log("associatedMSolTokenAccountAddress", beneficiaryATA.toBase58());
    if (createAssociateTokenInstruction) {
      tx.add(createAssociateTokenInstruction);
    }

    // create referralState account
    const referralAccSize = program.account.referralState.size + 20; // add some space for the string
    tx.add(
      web3.SystemProgram.createAccount({
        fromPubkey: provider.wallet.publicKey,
        /** Public key of the created account */
        newAccountPubkey: REFERRAL_TEST_PUBKEY,
        /** Amount of lamports to transfer to the created account */
        lamports: await provider.connection.getMinimumBalanceForRentExemption(
          referralAccSize
        ),
        /** Amount of space in bytes to allocate to the created account */
        space: referralAccSize, // add some space for the string
        /** Public key of the program to assign as the owner of the created account */
        programId: program.programId,
      })
    );
    // init referral account
    console.log(
      "program.instruction.initReferralAccount ",
      REFERRAL_TEST_PUBKEY.toBase58()
    );
    tx.add(
      program.instruction.initReferralAccount(PARTNER_NAME, {
        accounts: {
          globalState: GLOBAL_STATE_PUBKEY,
          adminAccount: ADMIN_PUBKEY,
          partnerAccount: PARTNER_ID,
          tokenPartnerAccount: beneficiaryATA,
          referralState: REFERRAL_TEST_PUBKEY,
        },
        signers: [ADMIN_KEYPAIR],
      })
    );
    // simulate the tx
    provider.simulate(tx, [REFERRAL_TEST_KEYPAIR, ADMIN_KEYPAIR]);
    // send the tx
    provider.send(tx, [REFERRAL_TEST_KEYPAIR, ADMIN_KEYPAIR]);
  }
}
