//---------------== Module Declaration
//declare your new function mod here to be included into this project, then export it via "pub use"
//#[allow(non_snake_case)]
//pub mod flashloanBorrow;
//#[allow(non_snake_case)]
//pub mod flashloanRepay;

pub mod utils;

//file names start with a lower case + Camel cases, but struct names start with Upper case + Camel cases!
//pub use flashloanBorrow::*;
//pub use flashloanRepay::*;
pub use utils::*;

use shank::ShankInstruction;

//---------------== Shank IDL Definition
/// Shank IDL enum describes all program instructions and their required accounts.
/// Manually write this below, then run IDL generation; This below does not affect runtime behavior.
/// TODO: when is signer writable?
/// writable(to be modified):, name= signer, ata, pda
/// non writable: program, system_program, mint
#[derive(ShankInstruction)]
pub enum ProgramIx {
  //---------------== Flashloan
  /// 0 FlashloanBorrow
  #[account(0, signer, writable, name = "signer", desc = "signer")]
  #[account(1, writable, name = "config_pda", desc = "Config PDA")]
  #[account(2, name = "vendor_prog", desc = "Vendor Program")]
  #[account(3, name = "token_mint", desc = "Token Mint")]
  #[account(4, name = "token_prog", desc = "Token Program")]
  #[account(5, writable, name = "from_ata", desc = "From ATA")]
  FlashloanBorrow { flashloan_vendor: u8, amount: u64 },

  /// 1 FlashloanBorrow
  #[account(0, signer, writable, name = "signer", desc = "signer")]
  #[account(1, writable, name = "config_pda", desc = "Config PDA")]
  #[account(2, name = "vendor_prog", desc = "Vendor Program")]
  #[account(3, name = "token_mint", desc = "Token Mint")]
  #[account(4, name = "token_prog", desc = "Token Program")]
  #[account(5, writable, name = "from_ata", desc = "From ATA")]
  FlashloanRepay { flashloan_vendor: u8, amount: u64 },
  //---------------== Admin PDA
  //---------------== User PDA
  //---------------== Action PDA
} //update here and lib.rs for new functions
