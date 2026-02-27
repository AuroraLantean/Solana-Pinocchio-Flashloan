//use pinocchio::{error::ProgramError, AccountView, Address, ProgramResult};
//use crate::{none_zero_u64, Ee, PROG_ADDR};

//temporarily store loan record
#[derive(Clone, Debug)]
#[repr(C, packed)]
pub struct LoanRecord {
  pub lender_token_acct: [u8; 32],
  pub balance_with_fee: u64,
}
