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

  // mSOL mint authority, maybe Marinade main program id
  const MSOL_MINT_AUTHORITY = Keypair.generate();
  // partner name - length should be 10
  const PARTNER_NAME = "keisukew53";
  // partner account
  const PARTNER = Keypair.generate();
  // referral state account address
  const REFERRAL = Keypair.generate();

  before(async () => {
    // Airdrop SOLs to the PARTNER.
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(PARTNER.publicKey, 1e10),
      "confirmed"
    );

    // create mSOL token mint
    msolMint = await Token.createMint(
      provider.connection,
      PARTNER,
      MSOL_MINT_AUTHORITY.publicKey,
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
        beneficiaryAccount: beneficiaryPda,
        partnerAccount: PARTNER.publicKey,
        state: REFERRAL.publicKey,
        systemProgram: SystemProgram.programId,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
      },
      instructions: [
        await program.account.referralState.createInstruction(REFERRAL),
      ],
      signers: [REFERRAL, PARTNER],
    });

    // get referral account
    const referralState = await program.account.referralState.fetch(
      REFERRAL.publicKey
    );
    // check if partner mSOL ATA matches what we expect
    assert.ok(referralState.beneficiaryAccount.equals(beneficiaryPda));
    // check if partner address matches what we expect
    assert.ok(referralState.partnerAccount.equals(PARTNER.publicKey));
    // check if partner name matches what we expect
    assert.ok(
      String.fromCharCode(...referralState.partnerName) === PARTNER_NAME
    );
  });

  it("should update authority", async () => {
    const NEW_PARTNER = Keypair.generate();
    // beneficiary - mSOL ATA for partner
    const [_new_beneficiary_pda] = await PublicKey.findProgramAddress(
      [
        NEW_PARTNER.publicKey.toBuffer(),
        TOKEN_PROGRAM_ID.toBuffer(),
        msolMint.publicKey.toBuffer(),
      ],
      ASSOCIATED_TOKEN_PROGRAM_ID
    );

    // update authority
    await program.rpc.updateAuthority({
      accounts: {
        msolMint: msolMint.publicKey,
        newBeneficiaryAccount: _new_beneficiary_pda,
        newPartnerAccount: NEW_PARTNER.publicKey,
        partnerAccount: PARTNER.publicKey,
        state: REFERRAL.publicKey,
        systemProgram: SystemProgram.programId,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
      },
      signers: [PARTNER],
    });

    // old partner no longer has permission to update authority
    await assert.rejects(
      async () => {
        await program.rpc.updateAuthority({
          accounts: {
            msolMint: msolMint.publicKey,
            newBeneficiaryAccount: _new_beneficiary_pda,
            newPartnerAccount: NEW_PARTNER.publicKey,
            partnerAccount: PARTNER.publicKey,
            state: REFERRAL.publicKey,
            systemProgram: SystemProgram.programId,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            tokenProgram: TOKEN_PROGRAM_ID,
            rent: SYSVAR_RENT_PUBKEY,
          },
          signers: [PARTNER],
        });
      }
      // {
      //   message: "300: Access denied",
      // }
    );

    // update authority back to previous partner
    await program.rpc.updateAuthority({
      accounts: {
        msolMint: msolMint.publicKey,
        newBeneficiaryAccount: beneficiaryPda,
        newPartnerAccount: PARTNER.publicKey,
        partnerAccount: NEW_PARTNER.publicKey,
        state: REFERRAL.publicKey,
        systemProgram: SystemProgram.programId,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
      },
      signers: [NEW_PARTNER],
    });
  });

  it("should update referral state", async () => {
    const NEW_TRANSFER_DURATION = 2_592_000 * 2;

    // update referral state
    await program.rpc.update(NEW_TRANSFER_DURATION, true, {
      accounts: {
        partnerAccount: PARTNER.publicKey,
        state: REFERRAL.publicKey,
      },
      signers: [PARTNER],
    });

    // get referral state
    const referralState = await program.account.referralState.fetch(
      REFERRAL.publicKey
    );
    // check if transfer period is updated
    assert.ok(referralState.transferDuration === NEW_TRANSFER_DURATION);
    // check if pause is updated
    assert.ok(referralState.pause === true);
  });
});
