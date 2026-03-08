/*lib.rs serves as your program’s entrypoint
- takes in the program ID, accounts, and instruction data, then reads the first byte as a discriminator to determine which method to call*/
#![no_std]
#![allow(unexpected_cfgs)]
use pinocchio::{entrypoint, error::ProgramError, AccountView, Address, ProgramResult};
use pinocchio_log::log;
use pinocchio_pubkey::declare_id;

//#[cfg(not(feature = "no-entrypoint"))]
entrypoint!(process_instruction);

pub mod instructions;
pub use instructions::*;
pub mod state;
pub use state::*;

//#[cfg(test)]
//mod litesvm1;
//#[cfg(test)]
//mod litesvm_helpers;

declare_id!("FcLwqf7L3VyxWuMKKzLA7vJqBo8bj9i3zHkxLe65Z1Ad");
//declare_id!("8ZEf7xqzjxELMywQbCEeYVgw9zDCijA3saDCHZwgbe7i");
//crate::ID
//change declare_id!() to 8ZEf7
// jj build2;
//change declare_id!() back to FcLwq
// jjb ; jj tts 2
pub const PROG_ADDR: Address = Address::new_from_array(ID);
pub const TOKEN_LGC_ADDR: Address =
  Address::from_str_const("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
pub const TOKEN_2022_ADDR: Address =
  Address::from_str_const("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");

fn process_instruction(
  program_id: &Address,
  accounts: &[AccountView],
  instruction_data: &[u8],
) -> ProgramResult {
  if program_id.ne(&PROG_ADDR) {
    return Err(ProgramError::IncorrectProgramId);
  }
  // `split_first` separates the first byte (discriminator) from the rest (payload).
  let (discriminator, data) = instruction_data
    .split_first()
    .ok_or_else(|| ProgramError::InvalidInstructionData)?;
  //log!("discriminator: {}", *discriminator);

  //reads the first byte as a discriminator to determine which method to call (here: 0 = DepositSol, 1 = WithdrawSol).
  match discriminator {
    VaultInit::DISCRIMINATOR => VaultInit::try_from((data, accounts))?.process(),
    VaultAtaInit::DISCRIMINATOR => VaultAtaInit::try_from((data, accounts))?.process(),
    TokLgcDeposit::DISCRIMINATOR => TokLgcDeposit::try_from((data, accounts))?.process(),
    FlashloanBorrow::DISCRIMINATOR => FlashloanBorrow::try_from((data, accounts))?.process(),
    FlashloanRepay::DISCRIMINATOR => FlashloanRepay::try_from((data, accounts))?.process(),
    VaultInitCaller::DISCRIMINATOR => VaultInitCaller::try_from((data, accounts))?.process(),
    InitAnchorPdaCaller::DISCRIMINATOR => {
      InitAnchorPdaCaller::try_from((data, accounts))?.process()
    }
    _ => Err(Ee::MethodDiscriminator.into()),
  } //file names start with a lower case + Camel cases, but struct names start with Upper case + Camel cases!
}
