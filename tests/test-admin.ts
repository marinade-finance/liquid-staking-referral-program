import assert from "assert";
import * as anchor from "@project-serum/anchor";
import { Program, web3 } from "@project-serum/anchor";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  Token,
} from "@solana/spl-token";
import {
  getAssociatedTokenAccountAddress,
  getTokenAccountInfo,
  SYSTEM_PROGRAM_ID,
} from "@marinade.finance/marinade-ts-sdk/dist/util";
import { isUint8Array } from "util/types";
import { expect } from "chai";
// import { MarinadeReferral } from "../target/types/marinade_referral";

const { Keypair, SystemProgram, PublicKey, SYSVAR_RENT_PUBKEY } = web3;

describe("marinade-referral test-admin-instructions", () => {
  // Local cluster provider.
  if (!process.env.ANCHOR_PROVIDER_URL) {
    process.env.ANCHOR_PROVIDER_URL = "http://localhost:8899";
  }
  const provider = anchor.Provider.env();

  // Configure the client to use the local cluster.
  anchor.setProvider(provider);

  // anchor-client Instance to referral program
  const marinadeReferral = anchor.workspace.MarinadeReferral as Program;

  // let blockhash: Blockhash;
  // let feeCalculator: FeeCalculator;

  // mSOL token mint
  let msolMint: Token;

  // beneficiary associated token address
  let partnerTokenAccount: InstanceType<typeof PublicKey>;

  // treasury associated token address
  let treasuryAccount: InstanceType<typeof PublicKey>;
  let treasuryAuthPda: InstanceType<typeof PublicKey>;
  let treasuryAuthBump = 0;

  const globalStateKeyPair: web3.Keypair = web3.Keypair.fromSecretKey(
    new Uint8Array([
      134, 187, 164, 119, 110, 122, 23, 81, 124, 160, 171, 39, 43, 21, 99, 70,
      76, 134, 197, 224, 143, 215, 219, 77, 113, 249, 46, 150, 129, 186, 236, 4,
      11, 97, 116, 100, 244, 31, 228, 117, 219, 46, 34, 185, 59, 70, 45, 64, 93,
      139, 190, 44, 110, 167, 44, 91, 202, 253, 222, 122, 43, 255, 45, 163,
    ])
  );

  // partner name - length should be 10
  const PARTNER_NAME = "abcde12345";
  // mSOL mint authority
  const MSOL_MINT_AUTHORITY = Keypair.generate();
  // admin account
  const ADMIN_KEYPAIR = Keypair.generate();
  // partner account
  const PARTNER_KEYPAIR = Keypair.generate();
  // treasury holder account
  const REFERRAL_KEYPAIR = Keypair.generate();

  const TREASURY_AUTH_SEED = "mr_treasury";

  before(async () => {
    // Airdrop SOLs to the admin.
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(ADMIN_KEYPAIR.publicKey, 1e10),
      "confirmed"
    );

    // // Airdrop SOLs to the treasury holder.
    // await provider.connection.confirmTransaction(
    //   await provider.connection.requestAirdrop(TREASURY.publicKey, 1e9),
    //   "confirmed"
    // );

    // create mSOL token mint
    msolMint = await Token.createMint(
      provider.connection,
      ADMIN_KEYPAIR,
      MSOL_MINT_AUTHORITY.publicKey,
      null,
      0,
      TOKEN_PROGRAM_ID
    );

    // mSOL ATA for partner
    partnerTokenAccount = await getAssociatedTokenAccountAddress(
      msolMint.publicKey,
      PARTNER_KEYPAIR.publicKey
    );
    // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // @ts-ignore // createAssociatedTokenAccountInternal exists, it's just not in the d.ts file
    await msolMint.createAssociatedTokenAccountInternal(
      PARTNER_KEYPAIR.publicKey,
      partnerTokenAccount
    );

    // treasury Auth PDA & bump
    [treasuryAuthPda, treasuryAuthBump] = await PublicKey.findProgramAddress(
      [Buffer.from(anchor.utils.bytes.utf8.encode(TREASURY_AUTH_SEED))],
      marinadeReferral.programId
    );

    // treasury - mSOL ATA for treasury
    treasuryAccount = await getAssociatedTokenAccountAddress(
      msolMint.publicKey,
      treasuryAuthPda
    );
    // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // @ts-ignore // createAssociatedTokenAccountInternal exists, it's just not in the d.ts file
    await msolMint.createAssociatedTokenAccountInternal(
      treasuryAuthPda,
      treasuryAccount
    );
    // mint mSOL to treasury_msol_account
    await msolMint.mintTo(
      treasuryAccount,
      MSOL_MINT_AUTHORITY.publicKey,
      [MSOL_MINT_AUTHORITY],
      1_000
    );

    // -----------------------
    // initialize global state
    // -----------------------
    expect(marinadeReferral.account.globalState.size).to.eq(73);
    {
      const tx = new web3.Transaction();
      tx.add(
        web3.SystemProgram.createAccount({
          fromPubkey: ADMIN_KEYPAIR.publicKey,
          newAccountPubkey: globalStateKeyPair.publicKey,
          lamports: 1e7,
          space: marinadeReferral.account.globalState.size,
          programId: marinadeReferral.programId,
        })
      );
      tx.add(
        marinadeReferral.instruction.initialize(treasuryAuthBump, {
          accounts: {
            adminAccount: ADMIN_KEYPAIR.publicKey,
            globalState: globalStateKeyPair.publicKey,
            treasuryMsolAccount: treasuryAccount,
          },
        })
      );
      const confirmation = await web3.sendAndConfirmTransaction(
        provider.connection,
        tx,
        [ADMIN_KEYPAIR, globalStateKeyPair]
      );
      console.log("create & init global state confirmation:", confirmation);
    }

    // get global account
    const globalState: Record<string, any> =
      await marinadeReferral.account.globalState.fetch(
        globalStateKeyPair.publicKey
      );
    // check if admin address matches what we expect
    assert.ok(globalState.adminAccount.equals(ADMIN_KEYPAIR.publicKey));

    {
      expect(marinadeReferral.account.referralState.size).to.eq(182);
      const tx = new web3.Transaction();
      tx.add(
        web3.SystemProgram.createAccount({
          fromPubkey: ADMIN_KEYPAIR.publicKey,
          newAccountPubkey: REFERRAL_KEYPAIR.publicKey,
          lamports: 1e7,
          space: marinadeReferral.account.referralState.size + 24, // add space for the string
          programId: marinadeReferral.programId,
        })
      );
      tx.add(
        marinadeReferral.instruction.initReferralAccount(PARTNER_NAME, {
          accounts: {
            globalState: globalStateKeyPair.publicKey,
            adminAccount: ADMIN_KEYPAIR.publicKey,
            treasuryMsolAccount: treasuryAccount,
            referralState: REFERRAL_KEYPAIR.publicKey,
            partnerAccount: PARTNER_KEYPAIR.publicKey,
            tokenPartnerAccount: partnerTokenAccount,
          },
        })
      );
      const confirmation = await web3.sendAndConfirmTransaction(
        provider.connection,
        tx,
        [ADMIN_KEYPAIR, REFERRAL_KEYPAIR]
      );
      console.log("create & init referral state confirmation:", confirmation);
    }
  });


  it("should change authority", async () => {
    const NEW_ADMIN = Keypair.generate();

    // update authority
    await marinadeReferral.rpc.changeAuthority({
      accounts: {
        newAdminAccount: NEW_ADMIN.publicKey,
        adminAccount: ADMIN_KEYPAIR.publicKey,
        globalState: globalStateKeyPair.publicKey,
      },
      signers: [ADMIN_KEYPAIR],
    });

    // prev admin no longer has permission to change authority
    await assert.rejects(async () => {
      await marinadeReferral.rpc.changeAuthority({
        accounts: {
          newAdminAccount: NEW_ADMIN.publicKey,
          adminAccount: ADMIN_KEYPAIR.publicKey,
          globalState: globalStateKeyPair.publicKey,
        },
        signers: [ADMIN_KEYPAIR],
      });
    });
    console.log("error 0x8d expected");

    // update authority back to prev admin
    await marinadeReferral.rpc.changeAuthority({
      accounts: {
        newAdminAccount: ADMIN_KEYPAIR.publicKey,
        adminAccount: NEW_ADMIN.publicKey,
        globalState: globalStateKeyPair.publicKey,
      },
      signers: [NEW_ADMIN],
    });
  });


  it("should update referral state", async () => {
    const TRANSFER_DURATION = 2_592_000;
    const NEW_TRANSFER_DURATION = TRANSFER_DURATION * 2;

    // update referral state
    await marinadeReferral.rpc.updateReferral(NEW_TRANSFER_DURATION, true, {
      accounts: {
        adminAccount: ADMIN_KEYPAIR.publicKey,
        referralState: REFERRAL_KEYPAIR.publicKey,
        globalState: globalStateKeyPair.publicKey,
      },
      signers: [ADMIN_KEYPAIR],
    });

    // get referral state
    const referralState: Record<string, any> =
      await marinadeReferral.account.referralState.fetch(
        REFERRAL_KEYPAIR.publicKey
      );
    // check if transfer period is updated
    assert.ok(referralState.transferDuration === NEW_TRANSFER_DURATION);
    // check if pause is updated
    assert.ok(referralState.pause);

    // reset settings to default
    await marinadeReferral.rpc.updateReferral(TRANSFER_DURATION, false, {
      accounts: {
        adminAccount: ADMIN_KEYPAIR.publicKey,
        referralState: REFERRAL_KEYPAIR.publicKey,
        globalState: globalStateKeyPair.publicKey,
      },
      signers: [ADMIN_KEYPAIR],
    });
  });

  /*
  it("should transferToPartner", async () => {
    // transfer mSOL
    await marinadeReferral.rpc.transferToPartner({
      accounts: {
        globalState: globalStateKeyPair.publicKey,
        adminAccount: ADMIN_KEYPAIR.publicKey,
        tokenPartnerAccount: partnerTokenAccount,
        treasuryMsolAccount: treasuryAccount,
        treasuryMsolAuth: treasuryAuthPda,
        treasuryAccount: treasuryAccount,
        referralState: REFERRAL_KEYPAIR.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
      signers: [ADMIN_KEYPAIR],
    });
  });
*/
});
