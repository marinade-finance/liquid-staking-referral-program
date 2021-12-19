import { delete_global_state, setup_global_state } from "./setup_global_state";
import { setup_referral_state } from "./setup_referral_state";

// setup default admin & referral PDA
// setup_global_state();
// setup_referral_state();
delete_global_state();
setup_global_state();
