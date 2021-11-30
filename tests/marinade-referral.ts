import assert from "assert";
import * as anchor from "@project-serum/anchor";
import { Program, web3 } from "@project-serum/anchor";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  Token,
} from "@solana/spl-token";
// import { MarinadeReferral } from "../target/types/marinade_referral";

const { Keypair, SystemProgram, PublicKey, SYSVAR_RENT_PUBKEY } = web3;

describe("marinade-referral", () => {
  // Local cluster provider.
  const provider = anchor.Provider.env();

  // Configure the client to use the local cluster.
  anchor.setProvider(provider);

  // Instance to referral program
  const program = anchor.workspace.MarinadeReferral as Program;
  // const program = anchor.workspace
  //   .MarinadeReferral as Program<MarinadeReferral>;

  // mSOL token mint
  let msolMint: Token;

  // benefinciary associated token address
  let beneficiaryPda: InstanceType<typeof PublicKey>;

  // mSOL mint authority
  const MSOL_MINT_AUTHORITY_ID = new PublicKey("3JLPCS1qM2zRw3Dp6V4hZnYHd4toMNPkNesXdX9tg6KM");
  // partner name - length should be 10
  const PARTNER_NAME = "abcde12345";
  // admin account
  const ADMIN = Keypair.generate();
  // partner account
  const PARTNER = Keypair.generate();
  // referral state account address
  const REFERRAL = Keypair.generate();

  before(async () => {
    // Airdrop SOLs to the admin.
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(ADMIN.publicKey, 1e10),
      "confirmed"
    );

    // create mSOL token mint
    msolMint = await Token.createMint(
      provider.connection,
      ADMIN,
      MSOL_MINT_AUTHORITY_ID,
      null,
      0,
      TOKEN_PROGRAM_ID
    );

    // beneficiary - mSOL ATA for partner
    beneficiaryPda = (
      await PublicKey.findProgramAddress(
        [
          PARTNER.publicKey.toBuffer(),
          TOKEN_PROGRAM_ID.toBuffer(),
          msolMint.publicKey.toBuffer(),
        ],
        ASSOCIATED_TOKEN_PROGRAM_ID
      )
    )[0];
  });

  it("should initialize referral state", async () => {
    // initialize referral account
    await program.rpc.initialize([...Buffer.from(PARTNER_NAME)], {
      accounts: {
        msolMint: msolMint.publicKey,
        partnerAccount: PARTNER.publicKey,
        beneficiaryAccount: beneficiaryPda,
        adminAccount: ADMIN.publicKey,
        state: REFERRAL.publicKey,
        systemProgram: SystemProgram.programId,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
      },
      instructions: [
        await program.account.referralState.createInstruction(REFERRAL),
      ],
      signers: [REFERRAL, ADMIN],
    });

    // get referral account
    const referralState = await program.account.referralState.fetch(
      REFERRAL.publicKey
    );
    // check if admin address matches what we expect
    assert.ok(referralState.adminAccount.equals(ADMIN.publicKey));
    // check if partner mSOL ATA matches what we expect
    assert.ok(referralState.beneficiaryAccount.equals(beneficiaryPda));
    // check if partner name matches what we expect
    assert.ok(
      String.fromCharCode(...referralState.partnerName) === PARTNER_NAME
    );
  });

  it("should change authority", async () => {
    const NEW_ADMIN = Keypair.generate();

    // update authority
    await program.rpc.changeAuthority({
      accounts: {
        newAdminAccount: NEW_ADMIN.publicKey,
        adminAccount: ADMIN.publicKey,
        state: REFERRAL.publicKey,
      },
      signers: [ADMIN],
    });

    // old admin no longer has permission to change authority
    await assert.rejects(
      async () => {
        await program.rpc.changeAuthority({
          accounts: {
            newAdminAccount: NEW_ADMIN.publicKey,
            adminAccount: ADMIN.publicKey,
            state: REFERRAL.publicKey,
          },
          signers: [PARTNER],
        });
      }
    );

    // update authority back to previous admin
    await program.rpc.changeAuthority({
      accounts: {
        newAdminAccount: ADMIN.publicKey,
        adminAccount: NEW_ADMIN.publicKey,
        state: REFERRAL.publicKey,
      },
      signers: [NEW_ADMIN],
    });
  });

  it("should update referral state", async () => {
    const NEW_TRANSFER_DURATION = 2_592_000 * 2;

    // update referral state
    await program.rpc.update(NEW_TRANSFER_DURATION, true, {
      accounts: {
        adminAccount: ADMIN.publicKey,
        state: REFERRAL.publicKey,
      },
      signers: [ADMIN],
    });

    // get referral state
    const referralState = await program.account.referralState.fetch(
      REFERRAL.publicKey
    );
    // check if transfer period is updated
    assert.ok(referralState.transferDuration === NEW_TRANSFER_DURATION);
    // check if pause is updated
    assert.ok(referralState.pause);
  });
});
