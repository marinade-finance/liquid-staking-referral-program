import { GLOBAL_STATE_PUBKEY, program, provider } from "./constants";
import { BN, web3 } from "@project-serum/anchor";
import * as util from "util";
import { lamportsToSol } from "@marinade.finance/marinade-ts-sdk/dist/util";

export async function show_global_state() {
  console.log("-----------------------");
  console.log("globalAccount:", GLOBAL_STATE_PUBKEY.toBase58());
  // check if globalStateAccount exists
  const globalAccountBalance = await provider.connection.getBalance(
    GLOBAL_STATE_PUBKEY
  );
  // create globalState account
  console.log("globalAccount lamports Balance", globalAccountBalance);

  const globalAccountData: Record<string, any> =
    await program.account.globalState.fetch(GLOBAL_STATE_PUBKEY);
  console.log("-- adminAccount:", globalAccountData.adminAccount.toBase58());
  console.log(
    "-- treasuryMsolAccount:",
    globalAccountData.treasuryMsolAccount.toBase58()
  );
  console.log(
    "-- treasuryMsolAuthBump:",
    globalAccountData.treasuryMsolAuthBump
  );
}

export async function show_referral_info(referralPubkey: web3.PublicKey) {
  console.log("-----------------------");
  console.log("referralPubkey:", referralPubkey.toBase58());
  // check if globalStateAccount exists
  const referralAccountBalance = await provider.connection.getBalance(
    referralPubkey
  );
  // create referralState account
  console.log("referralAccount lamports Balance", referralAccountBalance);

  const data: Record<string, any> = await program.account.referralState.fetch(
    referralPubkey
  );

  console.log("-- referralAccount:", referralPubkey.toBase58());
  console.log("-- partnerName:", data.partnerName);
  console.log("-- partnerAccount:", data.partnerAccount.toBase58());
  console.log("-- tokenPartnerAccount:", data.tokenPartnerAccount.toBase58());
  console.log("-- transferDuration:", data.transferDuration);
  console.log("-- lastTransferTime:", data.lastTransferTime.toNumber());
  console.log("-- depositSolAmount:", lamportsToSol(data.depositSolAmount));
  console.log("-- depositSolOperations:", data.depositSolOperations.toNumber());
  console.log(
    "-- depositStakeAccountAmount:",
    lamportsToSol(data.depositStakeAccountAmount)
  );
  console.log(
    "-- depositStakeAccountOperations:",
    data.depositStakeAccountOperations.toNumber()
  );
  console.log("-- liqUnstakeMsolFees:", lamportsToSol(data.liqUnstakeMsolFees));
  console.log(
    "-- liqUnstakeSolAmount:",
    lamportsToSol(data.liqUnstakeSolAmount)
  );
  console.log("-- liqUnstakeOperations:", data.liqUnstakeOperations.toNumber());
  console.log(
    "-- delayedUnstakeAmount:",
    lamportsToSol(data.delayedUnstakeAmount)
  );
  console.log(
    "-- delayedUnstakeOperations:",
    data.delayedUnstakeOperations.toNumber()
  );
}
