use crate::fees::Fee;

//-----------------------------------------------------
pub fn get_shared_amount(fee: Fee, accumulated_amount: u64) -> u64 {
    fee.apply(accumulated_amount)
}
