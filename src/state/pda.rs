//use pinocchio::{error::ProgramError, AccountView, Address, ProgramResult};
//use crate::{none_zero_u64, Ee, PROG_ADDR};

use pinocchio::{error::ProgramError, AccountView, Address, ProgramResult};

use crate::{none_zero_u64, none_zero_u8, Ee, PROG_ADDR};

//------------==
#[derive(Clone, Debug)]
#[repr(C)]
pub struct Vault {
  admin: Address,      //32
  token_balc: [u8; 8], //8
  bump: u8,            //1
}
impl Vault {
  pub const LEN: usize = core::mem::size_of::<Vault>();
  //pub const LEN: usize = 1;

  pub const SEED: &[u8] = b"vault";

  pub fn admin(&self) -> &Address {
    &self.admin
  }
  pub fn token_balc(&self) -> u64 {
    u64::from_le_bytes(self.token_balc)
  }
  pub fn bump(&self) -> u8 {
    self.bump
  }
  pub fn set_admin(&mut self, addr: &Address) {
    self.admin = addr.clone();
  }
  pub fn set_token_balc(&mut self, amt: u64) -> ProgramResult {
    none_zero_u64(amt)?;
    self.token_balc = amt.to_le_bytes();
    Ok(())
  }
  pub fn set_bump(&mut self, bump: u8) -> ProgramResult {
    none_zero_u8(bump)?;
    self.bump = bump;
    Ok(())
  }
  pub fn check(pda: &AccountView) -> ProgramResult {
    if pda.data_len() != Self::LEN {
      return Ee::VaultDataLengh.e();
    }
    unsafe {
      if pda.owner().ne(&PROG_ADDR) {
        return Ee::VaultIsForeign.e();
      }
    }
    Ok(())
  }
  pub fn from_account_view(pda: &AccountView) -> Result<&mut Self, ProgramError> {
    Self::check(pda)?;
    unsafe { Ok(&mut *(pda.borrow_unchecked_mut().as_ptr() as *mut Self)) }
  }
}
//This is NOT a PDA, but a struct to be saved inside Loans PDA
#[derive(Clone, Debug)]
#[repr(C, packed)]
pub struct Loan {
  pub vault_ata: [u8; 32],
  pub balc_plus_fee: u64,
}
impl Loan {
  pub const LEN: usize = core::mem::size_of::<Loan>();
  //pub const LEN: usize = 32 + 8;
}
#[derive(Clone, Debug)]
#[repr(C)]
pub struct Loans {} //array of Loan. because array length is unknow, we do not specify it here
impl Loans {
  pub const SEED: &[u8] = b"loans";
}
