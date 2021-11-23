import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { MarinadeReferrals } from "../target/types/marinade_referrals";

describe("marinade-referrals", () => {
    // Local cluster provider.
    const provider = anchor.Provider.env();

    // Configure the client to use the local cluster.
    anchor.setProvider(provider);

    const program = anchor.workspace
        .MarinadeReferrals as Program<MarinadeReferrals>;

    it("Is initialized!", async () => {
        await program.rpc.initialize({});
    });
});
