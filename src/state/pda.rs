//use pinocchio::{error::ProgramError, AccountView, Address, ProgramResult};
//use crate::{none_zero_u64, Ee, PROG_ADDR};

use pinocchio::{error::ProgramError, AccountView, ProgramResult};

use crate::{none_zero_u8, Ee, PROG_ADDR};

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
  //pub const LEN: usize = 1;

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
  pub const SEED: &[u8] = b"loan_array";
}
