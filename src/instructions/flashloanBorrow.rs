use crate::{
  amount_from_token_acct, check_rent_sysvar, get_rent_exempt, instructions::check_signer, writable,
  Ee, FlashloanRepay, LoanRecord, PROG_ADDR,
};
use core::convert::TryFrom;
use pinocchio::{
  address::address_eq,
  cpi::{Seed, Signer},
  error::ProgramError,
  sysvars::instructions::{Instructions, INSTRUCTIONS_ID},
  AccountView, ProgramResult,
};
use pinocchio_log::log;

/// FlashloanBorrow
pub struct FlashloanBorrow<'a> {
  pub signer: &'a AccountView,
  pub lender_pda: &'a AccountView,
  pub loan_records: &'a AccountView,
  pub mint: &'a AccountView,
  pub instruction_sysvar: &'a AccountView,
  //pub token_program: &'a AccountView,
  //pub system_program: &'a AccountView,
  //pub config_pda: &'a AccountView,
  pub rent_sysvar: &'a AccountView,
  pub token_accounts: &'a [AccountView],
  //pub lender_ata: &'a AccountView,
  //pub user_ata: &'a AccountView,
  pub decimals: u8,
  pub bump: [u8; 1],
  pub fee: u16,
  pub amounts: &'a [u64],
} /*Flashloan{
  lender_pda, lender_ata,
  user_ata, mint, user(signer),
  config, sysvar_instructions,
  token_program, system_program }*/
impl<'a> FlashloanBorrow<'a> {
  pub const DISCRIMINATOR: &'a u8 = &22;

  pub fn process(self) -> ProgramResult {
    log!("FlashloanBorrow process()");
    let FlashloanBorrow {
      signer,
      lender_pda,
      loan_records,
      mint,
      instruction_sysvar,
      //token_program: _,
      //system_program: _,
      //config_pda: _,
      rent_sysvar,
      token_accounts,
      decimals,
      bump: _,
      fee,
      amounts,
    } = self;

    Ok(())
  }
}
impl<'a> TryFrom<(&'a [u8], &'a [AccountView])> for FlashloanBorrow<'a> {
  type Error = ProgramError;

  fn try_from(value: (&'a [u8], &'a [AccountView])) -> Result<Self, Self::Error> {
    log!("FlashloanBorrow try_from");
    let (data, accounts) = value;
    log!("accounts len: {}, data len: {}", accounts.len(), data.len());
    //let instruction_data = LoanInstructionData::try_from(data)?;

    let [signer, lender_pda, loan_records, mint, instruction_sysvar, rent_sysvar, token_accounts @ ..] =
      accounts
    else {
      return Err(ProgramError::NotEnoughAccountKeys);
    }; //lender_ata, user_ata
    check_signer(signer)?;
    writable(loan_records)?;
    check_rent_sysvar(rent_sysvar)?;
    //executable(token_program)?;
    //writable(config_pda)?;
    //check_pda(config_pda)?;
    //check_mint0a(mint, token_program)?;

    if instruction_sysvar.address().ne(&INSTRUCTIONS_ID) {
      return Err(ProgramError::UnsupportedSysvar);
    }
    // Each loan requires a lender_token_acct and a borrower_token_acct
    if (token_accounts.len() % 2).ne(&0) || token_accounts.len().eq(&0) {
      return Err(Ee::TokenAcctsLength.into());
    }
    if loan_records.try_borrow()?.len().ne(&0) {
      return Err(Ee::LoanRecordAcct.into());
    }

    //-------== parse variadic data
    let (decimals, data) = data.split_first().ok_or_else(|| Ee::ByteSizeForU8)?;

    let (bump, data) = data.split_first().ok_or_else(|| Ee::ByteSizeForU8)?;

    let (fee, data) = data
      .split_at_checked(size_of::<u16>())
      .ok_or_else(|| Ee::ByteSizeForU16)?;
    let fee = u16::from_le_bytes(fee.try_into().map_err(|_| Ee::ByteSizeForU16)?);
    log!("fee: {}", fee);

    //Deriving the protocol PDA with the fee creates isolated liquidity pools for each fee tier, eliminating the need to store fee data in accounts. This design is both safe and optimal since each PDA with a specific fee owns only the liquidity associated with that fee rate. If someone passes an invalid fee, the corresponding token account for that fee bracket will be empty, automatically causing the transfer to fail with insufficient funds.

    if data.len() % size_of::<u64>() != 0 {
      return Err(Ee::ByteSizeForU64.into());
    }
    // Get the amount slice
    let amounts: &[u64] = unsafe {
      core::slice::from_raw_parts(data.as_ptr() as *const u64, data.len() / size_of::<u64>())
    };
    log!("amounts: {}", amounts);
    if amounts.len() != token_accounts.len() / 2 {
      return Err(Ee::AmountsLenVsTokenAcctLen.into());
    }
    Ok(Self {
      signer,
      lender_pda,
      loan_records,
      mint,
      instruction_sysvar,
      //config_pda,
      //token_program,
      //system_program,
      rent_sysvar,
      token_accounts,
      //lender_ata, user_ata,
      decimals: *decimals,
      bump: [*bump],
      fee,
      amounts,
    })
  }
}
