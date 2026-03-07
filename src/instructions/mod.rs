//---------------== Module Declaration
//declare your new function mod here to be included into this project, then export it via "pub use"
#[allow(non_snake_case)]
pub mod flashloanBorrow;
#[allow(non_snake_case)]
pub mod flashloanRepay;
#[allow(non_snake_case)]
pub mod funcCaller;
#[allow(non_snake_case)]
pub mod tokLgcDeposit;
#[allow(non_snake_case)]
pub mod vaultAtaInit;
#[allow(non_snake_case)]
pub mod vaultInit;

pub mod utils;

//file names start with a lower case + Camel cases, but struct names start with Upper case + Camel cases!
pub use flashloanBorrow::*;
pub use flashloanRepay::*;
pub use funcCaller::*;
pub use tokLgcDeposit::*;
pub use utils::*;
pub use vaultAtaInit::*;
pub use vaultInit::*;

use shank::ShankInstruction;

//---------------== Shank IDL Definition
/// Shank IDL enum describes all program instructions and their required accounts.
/// Manually write this below, then run IDL generation; This below does not affect runtime behavior.
/// TODO: when is signer writable?
/// writable(to be modified):, name= signer, ata, pda
/// non writable: program, system_program, mint
#[derive(ShankInstruction)]
pub enum ProgramIx {
  //---------------== Vault PDA Init
  ///0 Vault PDA Init
  #[account(0, signer, writable, name = "signer", desc = "signer")]
  //#[account(5, writable, name = "config_pda", desc = "Config PDA")]
  #[account(1, name = "system_program", desc = "System Program")]
  #[account(2, name = "rent_sysvar", desc = "Rent Sysvar")]
  #[account(3, writable, name = "vault", desc = "Vault PDA")]
  VaultInit { vault_bump: u8, fee: u16 },

  //---------------== Vault Ata Init
  /// 1 Vault Ata Init
  #[account(0, signer, writable, name = "signer", desc = "signer")]
  #[account(1, writable, name = "vault_pda", desc = "Vault PDA")]
  #[account(2, writable, name = "vault_ata", desc = "Vault ATA")]
  #[account(3, name = "mint", desc = "Mint")]
  //#[account(5, writable, name = "config_pda", desc = "Config PDA")]
  #[account(4, name = "token_program", desc = "Token Program")]
  #[account(5, name = "system_program", desc = "System Program")]
  #[account(6, name = "atoken_program", desc = "Associated Token Program")]
  #[account(7, name = "rent_sysvar", desc = "Rent Sysvar")]
  VaultTokAcctInit { decimal: u8 },

  /// 2 Token Legacy Deposit to Vault
  #[account(0, signer, writable, name = "signer", desc = "signer")]
  #[account(1, writable, name = "signer_ata", desc = "Signer ATA")]
  #[account(2, name = "mint", desc = "Mint")]
  //#[account(5, writable, name = "config_pda", desc = "Config PDA")]
  #[account(3, name = "token_program", desc = "Token Program")]
  #[account(4, name = "system_program", desc = "System Program")]
  #[account(5, name = "atoken_program", desc = "Associated Token Program")]
  #[account(6, name = "rent_sysvar", desc = "Rent Sysvar")]
  #[account(7, writable, name = "vault_pda", desc = "Vault PDA")]
  #[account(8, writable, name = "vault_ata", desc = "Vault ATA")]
  TokLgcDeposit { decimal: u8, amount: u64 },

  //---------------== Flashloan
  /// 3 FlashloanBorrow
  #[account(0, signer, writable, name = "signer", desc = "signer")]
  #[account(1, writable, name = "loans_pda", desc = "Loans PDA")]
  #[account(2, name = "mint", desc = "Mint")]
  #[account(3, name = "token_prog", desc = "Token Program")]
  #[account(4, name = "system_program", desc = "System Program")]
  #[account(5, name = "rent_sysvar", desc = "RentSysvar")]
  #[account(6, name = "instruction_sysvar", desc = "Instruction Sysvar")]
  #[account(7, writable, name = "vault", desc = "Vault PDA")]
  #[account(8, writable, name = "vault_ata", desc = "Vault ATA")]
  #[account(9, writable, name = "debtor_ata", desc = "Debtor ATA")]
  FlashloanBorrow {
    decimals: u8,
    loans_bump: u8,
    vault_bumps: [u8; 8],
    fees: [u16; 8],
  },

  /// 4 FlashloanRepay
  #[account(0, signer, writable, name = "signer", desc = "signer")]
  //#[account(1, writable, name = "config_pda", desc = "Config PDA")]
  #[account(1, writable, name = "loans_pda", desc = "Loans PDA")]
  #[account(2, writable, name = "vault", desc = "Vault PDA")]
  #[account(3, writable, name = "vault_ata", desc = "Vault ATA")]
  #[account(4, writable, name = "debtor_ata", desc = "Debtor ATA")]
  FlashloanRepay {},

  /// 5 FuncCaller
  #[account(0, signer, writable, name = "signer", desc = "signer")]
  //#[account(5, writable, name = "config_pda", desc = "Config PDA")]
  #[account(1, name = "target_prog", desc = "Target Program")]
  #[account(2, name = "system_program", desc = "System Program")]
  #[account(3, name = "rent_sysvar", desc = "Rent Sysvar")]
  #[account(4, writable, name = "vault", desc = "Vault PDA")]
  FuncCaller { vault_bump: u8, fee: u16 },
  //---------------== Admin PDA
  //---------------== User PDA
  //---------------== Action PDA
} //update here and lib.rs for new functions
