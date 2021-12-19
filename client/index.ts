import { mainModule } from "process";
import { REFERRAL_TEST_PUBKEY } from "./constants";
import { delete_global_state, setup_global_state } from "./setup_global_state";
import { setup_referral_state } from "./setup_referral_state";
import { show_global_state, show_referral_info } from "./show_global_state";

async function recreateGlobalState() {
    await delete_global_state();
    await setup_global_state();
    await show_global_state();
}
async function main() {
    // await recreateGlobalState();
    // setup default admin & referral PDA
    // setup_global_state();
    // setup_referral_state();
    await show_global_state();
    await show_referral_info(REFERRAL_TEST_PUBKEY);
}

main();
