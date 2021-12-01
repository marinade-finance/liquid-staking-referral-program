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
  if (!process.env.ANCHOR_PROVIDER_URL) {
    process.env.ANCHOR_PROVIDER_URL = "http://localhost:8899";
  }
  const provider = anchor.Provider.env();

  // Configure the client to use the local cluster.
  anchor.setProvider(provider);

  // Instance to referral program
  const program = anchor.workspace.MarinadeReferral as Program;
  // const program = anchor.workspace
  //   .MarinadeReferral as Program<MarinadeReferral>;

  // mSOL token mint
  let msolMint: Token;

  // beneficiary associated token address
  let beneficiaryPda: InstanceType<typeof PublicKey>;

  // treasury associated token address
  let treasuryPda: InstanceType<typeof PublicKey>;

  // global state PDA
  let globalStatePda: InstanceType<typeof PublicKey>;
  let globalStateBump: number;

  // referral state PDA
  let referralStatePda: InstanceType<typeof PublicKey>;
  let referralStateBump: number;

  // partner name - length should be 10
  const PARTNER_NAME = "abcde12345";
  // mSOL mint authority
  const MSOL_MINT_AUTHORITY = Keypair.generate();
  // admin account
  const ADMIN = Keypair.generate();
  // partner account
  const PARTNER = Keypair.generate();
  // treasury holder account
  const TREASURY = Keypair.generate();

  const GLOBAL_STATE_SEED = "mrp_initialize";
  const REFERRAL_STATE_SEED = "mrp_create_referral";

  before(async () => {
    // Airdrop SOLs to the admin.
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(ADMIN.publicKey, 1e10),
      "confirmed"
    );

    // Airdrop SOLs to the treasury holder.
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(TREASURY.publicKey, 1e9),
      "confirmed"
    );

    // create mSOL token mint
    msolMint = await Token.createMint(
      provider.connection,
      ADMIN,
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

    // treasury - mSOL ATA for treasury
    treasuryPda = await msolMint.createAssociatedTokenAccount(
      TREASURY.publicKey
    );
    // Airdrop mSOL to trasury_msol_account
    await msolMint.mintTo(
      treasuryPda,
      MSOL_MINT_AUTHORITY.publicKey,
      [MSOL_MINT_AUTHORITY],
      1_000
    );

    console.log(
      msolMint.publicKey.toString(),
      beneficiaryPda.toString(),
      treasuryPda.toString()
    );

    // global state PDA & bump
    const [_global_state_pda, _global_state_bump] =
      await PublicKey.findProgramAddress(
        [Buffer.from(anchor.utils.bytes.utf8.encode(GLOBAL_STATE_SEED))],
        program.programId
      );
    globalStatePda = _global_state_pda;
    globalStateBump = _global_state_bump;

    // referral state PDA & bump
    const [_referral_state_pda, _referral_state_bump] =
      await PublicKey.findProgramAddress(
        [
          PARTNER.publicKey.toBuffer(),
          Buffer.from(anchor.utils.bytes.utf8.encode(REFERRAL_STATE_SEED)),
        ],
        program.programId
      );
    referralStatePda = _referral_state_pda;
    referralStateBump = _referral_state_bump;
  });

  it("should initialize global state", async () => {
    // initialize referral account
    await program.rpc.initialize(globalStateBump, {
      accounts: {
        adminAccount: ADMIN.publicKey,
        globalState: globalStatePda,
        systemProgram: SystemProgram.programId,
      },
      signers: [ADMIN],
    });

    // get global account
    const globalState = await program.account.globalState.fetch(globalStatePda);
    // check if admin address matches what we expect
    assert.ok(globalState.adminAccount.equals(ADMIN.publicKey));
  });

  it("should change authority", async () => {
    const NEW_ADMIN = Keypair.generate();

    // update authority
    await program.rpc.changeAuthority({
      accounts: {
        newAdminAccount: NEW_ADMIN.publicKey,
        adminAccount: ADMIN.publicKey,
        globalState: globalStatePda,
      },
      signers: [ADMIN],
    });

    // prev admin no longer has permission to change authority
    await assert.rejects(async () => {
      await program.rpc.changeAuthority({
        accounts: {
          newAdminAccount: NEW_ADMIN.publicKey,
          adminAccount: ADMIN.publicKey,
          globalState: globalStatePda,
        },
        signers: [ADMIN],
      });
    });

    // update authority back to prev admin
    await program.rpc.changeAuthority({
      accounts: {
        newAdminAccount: ADMIN.publicKey,
        adminAccount: NEW_ADMIN.publicKey,
        globalState: globalStatePda,
      },
      signers: [NEW_ADMIN],
    });
  });

  it("should create referral PDA", async () => {
    // create referral account
    await program.rpc.createReferralPda(
      referralStateBump,
      [...Buffer.from(PARTNER_NAME)],
      {
        accounts: {
          msolMint: msolMint.publicKey,
          partnerAccount: PARTNER.publicKey,
          beneficiaryAccount: beneficiaryPda,
          adminAccount: ADMIN.publicKey,
          referralState: referralStatePda,
          globalState: globalStatePda,
          systemProgram: SystemProgram.programId,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          tokenProgram: TOKEN_PROGRAM_ID,
          rent: SYSVAR_RENT_PUBKEY,
        },
        signers: [ADMIN],
      }
    );

    // get referral account
    const referralState = await program.account.referralState.fetch(
      referralStatePda
    );
    // check if partner mSOL ATA matches what we expect
    assert.ok(referralState.beneficiaryAccount.equals(beneficiaryPda));
    // check if partner name matches what we expect
    assert.ok(
      String.fromCharCode(...referralState.partnerName) === PARTNER_NAME
    );
  });

  it("should update referral state", async () => {
    const TRANSFER_DURATION = 2_592_000;
    const NEW_TRANSFER_DURATION = TRANSFER_DURATION * 2;

    // update referral state
    await program.rpc.updateReferral(NEW_TRANSFER_DURATION, true, {
      accounts: {
        adminAccount: ADMIN.publicKey,
        referralState: referralStatePda,
        globalState: globalStatePda,
      },
      signers: [ADMIN],
    });

    // get referral state
    const referralState = await program.account.referralState.fetch(
      referralStatePda
    );
    // check if transfer period is updated
    assert.ok(referralState.transferDuration === NEW_TRANSFER_DURATION);
    // check if pause is updated
    assert.ok(referralState.pause);

    // reset settings to default
    await program.rpc.updateReferral(TRANSFER_DURATION, false, {
      accounts: {
        adminAccount: ADMIN.publicKey,
        referralState: referralStatePda,
        globalState: globalStatePda,
      },
      signers: [ADMIN],
    });
  });

  it("should transfer mSOL from treasury to beneficiary", async () => {
    // transfer mSOL
    await program.rpc.transferLiqShares({
      accounts: {
        msolMint: msolMint.publicKey,
        beneficiaryAccount: beneficiaryPda,
        treasuryMsolAccount: treasuryPda,
        treasuryAccount: TREASURY.publicKey,
        referralState: referralStatePda,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
      signers: [TREASURY],
    });
  });
});
