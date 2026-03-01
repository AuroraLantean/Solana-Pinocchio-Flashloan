//use pinocchio::{error::ProgramError, AccountView, Address, ProgramResult};
//use crate::{none_zero_u64, Ee, PROG_ADDR};

//------------==
#[derive(Clone, Debug)]
#[repr(C)]
pub struct Vault {
  //mint: Address, //32
  //id: [u8; 8],   //8
  //decimal: u8,   //1
  bump: u8, //1
}
impl Vault {
  pub const LEN: usize = core::mem::size_of::<Vault>();
  //pub const LEN: usize = 32 + 8 + 1+1;

  pub const SEED: &[u8] = b"vault";

  /*pub fn mint(&self) -> &Address {
    &self.mint
  }
  pub fn id(&self) -> u64 {
    u64::from_le_bytes(self.id)
  }
  pub fn decimal(&self) -> u8 {
    self.decimal
  }*/
  pub fn bump(&self) -> u8 {
    self.bump
  }
}
//temporarily store loan record
#[derive(Clone, Debug)]
#[repr(C, packed)]
pub struct LoanRecord {
  pub lender_token_acct: [u8; 32],
  pub balance_with_fee: u64,
}
