import * as anchor from "@project-serum/anchor";
import { Program, web3 } from "@project-serum/anchor";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { MarinadeUtils } from "@marinade.finance/marinade-ts-sdk";
import { exit } from "process";
import { timeStamp } from "console";

const { Keypair, SystemProgram, PublicKey, SYSVAR_RENT_PUBKEY } = web3;

// Local cluster provider.
if (!process.env.ANCHOR_PROVIDER_URL) {
  process.env.ANCHOR_PROVIDER_URL = "https://api.devnet.solana.com";
  // process.env.ANCHOR_PROVIDER_URL = "http://127.0.0.1:8899";
}
const provider = anchor.Provider.env();

// Configure the client to use the local cluster.
anchor.setProvider(provider);

// Instance to referral program
const program = anchor.workspace.MarinadeReferral as Program;

// partner name - length should be 10
const PARTNER_NAME = "REF_TEST";

// admin account
const ADMIN_KEYPAIR = Keypair.fromSecretKey(
  new Uint8Array([
    136, 60, 191, 232, 11, 20, 1, 82, 147, 185, 119, 92, 226, 212, 217, 227, 38,
    100, 72, 135, 189, 121, 32, 38, 93, 10, 41, 104, 38, 158, 171, 38, 138, 239,
    196, 48, 200, 45, 19, 235, 223, 73, 101, 62, 195, 45, 48, 246, 226, 240,
    177, 172, 213, 0, 184, 113, 158, 176, 17, 177, 2, 215, 168, 135,
  ])
); // AMMK9YLj8PRRG4K9DUsTNPZAZXeVbHiQJxakuVuvSKrn
const ADMIN_PUBKEY = ADMIN_KEYPAIR.publicKey;

// partner address
const PARTNER_ID = new PublicKey(
  "4yMfRHP8T5c54sm8NFT2euvNpir2TsSukS5GK8Y9h7wg"
);

// mSOL token mint
const MSOL_MINT_PUBKEY = new PublicKey(
  "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So"
);

const GLOBAL_STATE_KEYPAIR = Keypair.fromSecretKey(
  new Uint8Array([
    134, 187, 164, 119, 110, 122, 23, 81, 124, 160, 171, 39, 43, 21, 99, 70, 76,
    134, 197, 224, 143, 215, 219, 77, 113, 249, 46, 150, 129, 186, 236, 4, 11,
    97, 116, 100, 244, 31, 228, 117, 219, 46, 34, 185, 59, 70, 45, 64, 93, 139,
    190, 44, 110, 167, 44, 91, 202, 253, 222, 122, 43, 255, 45, 163,
  ])
);
// mRg6bDsAd5uwERAdNTynoUeRbqQsLa7yzuK2kkCUPGW
const GLOBAL_STATE_PUBKEY = GLOBAL_STATE_KEYPAIR.publicKey;

const REFERRAL_TEST_KEYPAIR = Keypair.fromSecretKey(
  new Uint8Array([
    67, 83, 104, 162, 169, 27, 86, 141, 190, 216, 224, 160, 123, 200, 93, 50,
    184, 177, 175, 90, 146, 83, 181, 236, 126, 227, 239, 163, 220, 95, 218, 15,
    11, 97, 179, 181, 178, 93, 218, 91, 81, 205, 66, 88, 52, 111, 248, 190, 63,
    85, 95, 203, 181, 148, 137, 36, 248, 111, 225, 23, 145, 55, 104, 21,
  ])
);
// mRtnRH2M3rMLP4BBcrxkk4WBKsSi3JvoyUEog7gf3qE
const REFERRAL_TEST_PUBKEY = REFERRAL_TEST_KEYPAIR.publicKey;

export async function setup() {
  let tx = new web3.Transaction();

  if (
    program.programId.toBase58() !==
    "mRefx8ypXNxE59NhoBqwqb3vTvjgf8MYECp4kgJWiDY"
  ) {
    console.log(
      'program.programId.toBase58()!="mRefx8ypXNxE59NhoBqwqb3vTvjgf8MYECp4kgJWiDY"'
    );
    exit(1);
  }

  // --- ASSIGN AN ACCOUNT to our program, as long as you've the priv-key and owner-program is System-Program
  // tx.add(
  //   web3.SystemProgram.assign({
  //     accountPubkey: REFERRAL_TEST_PUBKEY,
  //     programId: program.programId,
  //   })
  // );
  /*
  // --- DELETE AN ACCOUNT with space/data as long as you've the priv-key and our program is the owner-program
  tx.add(
    program.instruction.deleteAccount({
      accounts: {
        toDelete: GLOBAL_STATE_PUBKEY,
        beneficiary: new PublicKey(
          "3Pb4Q6XcZCCgz7Gvd229YzFoU1DpQ4myUQFx8Z9AauQ6"
        ),
      },
      //signers: [GLOBAL_STATE_KEYPAIR],
    })
  );
  tx.add(
    program.instruction.deleteAccount({
      accounts: {
        toDelete: REFERRAL_TEST_PUBKEY,
        beneficiary: new PublicKey(
          "3Pb4Q6XcZCCgz7Gvd229YzFoU1DpQ4myUQFx8Z9AauQ6"
        ),
      },
      //signers: [REFERRAL_TEST_KEYPAIR],
    })
  );
  // simulate the tx
  provider.simulate(tx, [GLOBAL_STATE_KEYPAIR, REFERRAL_TEST_KEYPAIR]);
  // send the tx
  provider.send(tx, [GLOBAL_STATE_KEYPAIR, REFERRAL_TEST_KEYPAIR]);
  return;
  */

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

  // check if globalStateAccount exists
  const globalAccountBalance = await provider.connection.getBalance(
    GLOBAL_STATE_PUBKEY
  );
  // create globalState account
  console.log("globalAccountBalance", globalAccountBalance);
  if (globalAccountBalance === 0) {
    const globalAccSize = program.account.globalState.size;
    tx.add(
      web3.SystemProgram.createAccount({
        fromPubkey: provider.wallet.publicKey,
        /** Public key of the created account */
        newAccountPubkey: GLOBAL_STATE_PUBKEY,
        /** Amount of lamports to transfer to the created account */
        lamports: await provider.connection.getMinimumBalanceForRentExemption(
          globalAccSize
        ),
        /** Amount of space in bytes to allocate to the created account */
        space: globalAccSize,
        /** Public key of the program to assign as the owner of the created account */
        programId: program.programId,
      })
    );
    // initialize global state
    console.log(
      "program.instruction.initialize ",
      GLOBAL_STATE_PUBKEY.toBase58()
    );
    tx.add(
      program.instruction.initialize({
        accounts: {
          adminAccount: ADMIN_PUBKEY,
          globalState: GLOBAL_STATE_PUBKEY,
          paymentMint: MSOL_MINT_PUBKEY,
          systemProgram: SystemProgram.programId,
        },
        signers: [ADMIN_KEYPAIR],
      })
    );
    // simulate the tx
    provider.simulate(tx, [GLOBAL_STATE_KEYPAIR, ADMIN_KEYPAIR]);
    // send the tx
    provider.send(tx, [GLOBAL_STATE_KEYPAIR, ADMIN_KEYPAIR]);
  }

  tx = new web3.Transaction();
  // check if referralAccount exists
  const referralAccountBalance = await provider.connection.getBalance(
    REFERRAL_TEST_PUBKEY
  );
  console.log("referralAccountBalance", referralAccountBalance);
  if (referralAccountBalance === 0) {
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
      "program.rpc.initReferralAccount ",
      REFERRAL_TEST_PUBKEY.toBase58()
    );
    tx.add(
      program.instruction.initReferralAccount(PARTNER_NAME, {
        accounts: {
          partnerAccount: PARTNER_ID,
          msolMint: MSOL_MINT_PUBKEY,
          paymentMint: MSOL_MINT_PUBKEY,
          tokenPartnerAccount: beneficiaryATA,
          adminAccount: ADMIN_PUBKEY,
          referralState: REFERRAL_TEST_PUBKEY,
          globalState: GLOBAL_STATE_PUBKEY,
          systemProgram: SystemProgram.programId,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          tokenProgram: TOKEN_PROGRAM_ID,
          rent: SYSVAR_RENT_PUBKEY,
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
