import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { MarinadeReferral } from "../target/types/marinade_referral";

describe("marinade-referral", () => {
    // Local cluster provider.
    const provider = anchor.Provider.env();

    // Configure the client to use the local cluster.
    anchor.setProvider(provider);

    const program = anchor.workspace
        .MarinadeReferral as Program<MarinadeReferral>;

    it("Is initialized!", async () => {
        await program.rpc.initialize({});
    });
});
