import { homedir } from "os";
import { readFileSync } from "fs";

import { web3 } from "@project-serum/anchor";

export async function sleep(milliseconds): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
}

export function nodejsLocalWalletKeyPair(): web3.Keypair {
  return web3.Keypair.fromSecretKey(
    Buffer.from(
      JSON.parse(
        readFileSync(homedir() + "/.config/solana/id.json", {
          encoding: "utf-8",
        })
      )
    )
  );
}
