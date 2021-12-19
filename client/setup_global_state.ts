import * as anchor from "@project-serum/anchor";
import { web3 } from "@project-serum/anchor";
import { Token, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { MarinadeUtils } from "@marinade.finance/marinade-ts-sdk";
import { exit } from "process";

import {
  ADMIN_KEYPAIR,
  ADMIN_PUBKEY,
  GLOBAL_STATE_KEYPAIR,
  GLOBAL_STATE_PUBKEY,
  MSOL_MINT_PUBKEY,
  PARTNER_ID,
  program,
  provider,
} from "./constants";
import { sleep } from "./util";
import { homedir } from "os";
import { readFileSync } from "fs";

const { Keypair, PublicKey } = web3;

export async function delete_global_state() {
  // check if globalStateAccount exists
  const globalAccountBalance = await provider.connection.getBalance(
    GLOBAL_STATE_PUBKEY
  );
  // create globalState account
  console.log("globalAccountBalance", globalAccountBalance);
  if (globalAccountBalance !== 0) {
    // delete global state
    console.log("DELETING GLOBAL STATE");
    program.rpc.deleteProgramAccount({
      accounts: {
        accountToDelete: GLOBAL_STATE_PUBKEY,
        beneficiary: provider.wallet.publicKey,
      },
    });
    await sleep(15000);
  }
}

export async function setup_global_state() {
  const tx = new web3.Transaction();

  if (
    program.programId.toBase58() !==
    "mRefx8ypXNxE59NhoBqwqb3vTvjgf8MYECp4kgJWiDY"
  ) {
    console.log(
      'program.programId.toBase58()!="mRefx8ypXNxE59NhoBqwqb3vTvjgf8MYECp4kgJWiDY"'
    );
    exit(1);
  }

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
    // need to create
    console.log("CREATING GLOBAL STATE");

    const [referralMsolTreasuryAuth, treasuryMsolAuthBump] =
      await PublicKey.findProgramAddress(
        [Buffer.from("mr_treasury")],
        program.programId
      );
    console.log(
      "referralMsolTreasuryAuth ",
      referralMsolTreasuryAuth.toBase58(),
      "bump",
      treasuryMsolAuthBump
    );

    // treasury token account
    const newTreasuryAccount = await createNewTokenAccount(
      provider,
      MSOL_MINT_PUBKEY,
      referralMsolTreasuryAuth
    );

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
      "GlobalStateInit: program.instruction.initialize ",
      GLOBAL_STATE_PUBKEY.toBase58()
    );

    tx.add(
      program.instruction.initialize(treasuryMsolAuthBump, {
        accounts: {
          adminAccount: ADMIN_PUBKEY,
          globalState: GLOBAL_STATE_PUBKEY,
          treasuryMsolAccount: newTreasuryAccount,
        },
        signers: [ADMIN_KEYPAIR],
      })
    );
    // simulate the tx
    provider.simulate(tx, [GLOBAL_STATE_KEYPAIR, ADMIN_KEYPAIR]);
    // send the tx
    provider.send(tx, [GLOBAL_STATE_KEYPAIR, ADMIN_KEYPAIR]);
  }
}

async function createNewTokenAccount(
  anchorProvider: anchor.Provider,
  mintAddress: web3.PublicKey,
  ownerAddress: web3.PublicKey
): Promise<web3.PublicKey> {
  const payer = Keypair.fromSecretKey(
    Buffer.from(
      JSON.parse(
        readFileSync(homedir() + "/.config/solana/id.json", {
          encoding: "utf-8",
        })
      )
    )
  );

  const mintClient = new Token(
    anchorProvider.connection,
    mintAddress,
    TOKEN_PROGRAM_ID,
    payer
  );
  // const mintClient = getMintClient(anchorProvider, mintAddress);
  return mintClient.createAccount(ownerAddress);
}
