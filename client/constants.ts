import * as anchor from "@project-serum/anchor";
import { Program, web3 } from "@project-serum/anchor";
const { Keypair, PublicKey } = web3;

// Local cluster provider.
if (!process.env.ANCHOR_PROVIDER_URL) {
  process.env.ANCHOR_PROVIDER_URL = "https://api.devnet.solana.com";
  // process.env.ANCHOR_PROVIDER_URL = "http://127.0.0.1:8899";
}
export const provider = anchor.Provider.env();

// Configure the client to use the local cluster.
anchor.setProvider(provider);

// Instance to referral program
export const program = anchor.workspace.MarinadeReferral as Program;

// partner name - length should be 10
export const PARTNER_NAME = "REF_TEST";

// admin account
export const ADMIN_KEYPAIR = Keypair.fromSecretKey(
  new Uint8Array([
    136, 60, 191, 232, 11, 20, 1, 82, 147, 185, 119, 92, 226, 212, 217, 227, 38,
    100, 72, 135, 189, 121, 32, 38, 93, 10, 41, 104, 38, 158, 171, 38, 138, 239,
    196, 48, 200, 45, 19, 235, 223, 73, 101, 62, 195, 45, 48, 246, 226, 240,
    177, 172, 213, 0, 184, 113, 158, 176, 17, 177, 2, 215, 168, 135,
  ])
); // AMMK9YLj8PRRG4K9DUsTNPZAZXeVbHiQJxakuVuvSKrn
export const ADMIN_PUBKEY = ADMIN_KEYPAIR.publicKey;

// partner address
export const PARTNER_ID = new PublicKey(
  "4yMfRHP8T5c54sm8NFT2euvNpir2TsSukS5GK8Y9h7wg"
);

// mSOL token mint
export const MSOL_MINT_PUBKEY = new PublicKey(
  "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So"
);

export const GLOBAL_STATE_KEYPAIR = Keypair.fromSecretKey(
  new Uint8Array([
    134, 187, 164, 119, 110, 122, 23, 81, 124, 160, 171, 39, 43, 21, 99, 70, 76,
    134, 197, 224, 143, 215, 219, 77, 113, 249, 46, 150, 129, 186, 236, 4, 11,
    97, 116, 100, 244, 31, 228, 117, 219, 46, 34, 185, 59, 70, 45, 64, 93, 139,
    190, 44, 110, 167, 44, 91, 202, 253, 222, 122, 43, 255, 45, 163,
  ])
);
// mRg6bDsAd5uwERAdNTynoUeRbqQsLa7yzuK2kkCUPGW
export const GLOBAL_STATE_PUBKEY = GLOBAL_STATE_KEYPAIR.publicKey;

export const REFERRAL_TEST_KEYPAIR = Keypair.fromSecretKey(
  new Uint8Array([
    67, 83, 104, 162, 169, 27, 86, 141, 190, 216, 224, 160, 123, 200, 93, 50,
    184, 177, 175, 90, 146, 83, 181, 236, 126, 227, 239, 163, 220, 95, 218, 15,
    11, 97, 179, 181, 178, 93, 218, 91, 81, 205, 66, 88, 52, 111, 248, 190, 63,
    85, 95, 203, 181, 148, 137, 36, 248, 111, 225, 23, 145, 55, 104, 21,
  ])
);
// mRtnRH2M3rMLP4BBcrxkk4WBKsSi3JvoyUEog7gf3qE
export const REFERRAL_TEST_PUBKEY = REFERRAL_TEST_KEYPAIR.publicKey;
