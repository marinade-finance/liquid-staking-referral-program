import * as anchor from "@project-serum/anchor";
import { Program, web3 } from "@project-serum/anchor";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";

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

// beneficiary associated token address
let beneficiaryPda: InstanceType<typeof PublicKey>;

// global state PDA
let globalStatePda: InstanceType<typeof PublicKey>;
let globalStateBump: number;

// referral state PDA
let referralStatePda: InstanceType<typeof PublicKey>;
let referralStateBump: number;

// partner name - length should be 10
const PARTNER_NAME = "abcde12345";

// admin account
const ADMIN = Keypair.fromSecretKey(
  new Uint8Array([
    136, 60, 191, 232, 11, 20, 1, 82, 147, 185, 119, 92, 226, 212, 217, 227, 38,
    100, 72, 135, 189, 121, 32, 38, 93, 10, 41, 104, 38, 158, 171, 38, 138, 239,
    196, 48, 200, 45, 19, 235, 223, 73, 101, 62, 195, 45, 48, 246, 226, 240,
    177, 172, 213, 0, 184, 113, 158, 176, 17, 177, 2, 215, 168, 135,
  ])
); // AMMK9YLj8PRRG4K9DUsTNPZAZXeVbHiQJxakuVuvSKrn

// mSOL token mint
const MSOL_MINT_ID = new PublicKey(
  "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So"
);

const GLOBAL_STATE_SEED = "mrp_initialize";
const REFERRAL_STATE_SEED = "mrp_create_referral";

const setup = async () => {
  // beneficiary - mSOL ATA for partner
  [beneficiaryPda] = await PublicKey.findProgramAddress(
    [
      ADMIN.publicKey.toBuffer(),
      TOKEN_PROGRAM_ID.toBuffer(),
      MSOL_MINT_ID.toBuffer(),
    ],
    ASSOCIATED_TOKEN_PROGRAM_ID
  );

  // global state PDA & bump
  [globalStatePda, globalStateBump] = await PublicKey.findProgramAddress(
    [Buffer.from(anchor.utils.bytes.utf8.encode(GLOBAL_STATE_SEED))],
    program.programId
  );
  // 9R2ExJFV6xcSBpPGSkRLQnV8wqpN1BqUb1g2g7rxPvEy
  console.log("Referral program state address: ", globalStatePda.toString());

  // referral state PDA & bump
  [referralStatePda, referralStateBump] = await PublicKey.findProgramAddress(
    [
      ADMIN.publicKey.toBuffer(),
      Buffer.from(anchor.utils.bytes.utf8.encode(REFERRAL_STATE_SEED)),
    ],
    program.programId
  );
  // G38vdEMpKne9kHByCg6G4AwJXxAyG3i6Hc96DUB1DKQA
  console.log("Referral state address: ", referralStatePda.toString());

  // initialize admin
  await program.rpc.initialize(globalStateBump, {
    accounts: {
      adminAccount: ADMIN.publicKey,
      globalState: globalStatePda,
      systemProgram: SystemProgram.programId,
    },
    signers: [ADMIN],
  });

  // create a referral pda
  await program.rpc.createReferralPda(
    referralStateBump,
    [...Buffer.from(PARTNER_NAME)],
    {
      accounts: {
        msolMint: MSOL_MINT_ID,
        partnerAccount: ADMIN.publicKey,
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
};

export default {
  setup,
};
