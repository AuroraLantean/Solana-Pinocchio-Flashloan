use crate::{
  amount_from_token_acct, check_rent_sysvar, executable, get_rent_exempt,
  instructions::check_signer, writable, Ee, FlashloanRepay, LoanRecord, PROG_ADDR,
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
  pub bump_array: [u8; 1],
  pub fee: u16,
  pub amounts: &'a [u64],
} /*Flashloan{
  lender_pda, lender_ata,
  user_ata, mint, user(signer),
  config, sysvar_instructions,
  token_program, system_program }*/
impl<'a> FlashloanBorrow<'a> {
  pub const DISCRIMINATOR: &'a u8 = &3;

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
      bump_array,
      fee,
      amounts,
    } = self;

    //-----------== Introspecting the Repay instruction
    let instruction_sysvar =
      unsafe { Instructions::new_unchecked(instruction_sysvar.try_borrow()?) };

    let num_instructions = instruction_sysvar.num_instructions();

    let repay_ix = instruction_sysvar.load_instruction_at(num_instructions as usize - 1)?;

    if repay_ix.get_program_id().to_bytes().ne(&crate::ID) {
      return Ee::RepayProgId.e();
    }

    if unsafe { *(repay_ix.get_instruction_data().as_ptr()) } != *FlashloanRepay::DISCRIMINATOR {
      return Ee::RepayDiscriminator.e();
    }

    if unsafe {
      !address_eq(
        &repay_ix.get_instruction_account_at_unchecked(1).key,
        lender_pda.address(),
      )
    } {
      return Ee::RepayIxLenderPda.e();
    }

    //-----------== Send tokens
    let fee_bytes = fee.to_le_bytes();
    let signer_seeds = [
      Seed::from("moon_pool".as_bytes()),
      Seed::from(&fee_bytes),
      Seed::from(&bump_array),
    ];
    let signer_seeds = &[Signer::from(&signer_seeds)];

    // Open the LoanRecord account and create a mutable slice to push the Loan struct to it
    let size = size_of::<LoanRecord>() * amounts.len();
    let lamports = get_rent_exempt(loan_records, rent_sysvar, size)?;

    pinocchio_system::instructions::CreateAccount {
      from: signer,
      to: loan_records,
      lamports,
      space: size as u64,
      owner: &PROG_ADDR,
    }
    .invoke()?;

    //Make a mutable slice from the loan account's data. We will populate this slice in a for loop as we process each loan and its corresponding transfer:
    let mut loan_records = loan_records.try_borrow_mut()?;
    let loan_records_slice = unsafe {
      core::slice::from_raw_parts_mut(loan_records.as_mut_ptr() as *mut LoanRecord, amounts.len())
    };

    //loop through all the loans. In each iteration, we get the lender_token_acct and borrower_token_acct, calculate the balance due to the protocol, save this data in the loanRecord account, and transfer the tokens.
    for (i, amount) in amounts.iter().enumerate() {
      if *amount == 0 {
        return Ee::BorrowedAmountIsZero.e();
      }
      let lender_token_acct = &token_accounts[i * 2];
      let borrower_token_acct = &token_accounts[i * 2 + 1];

      // Get the balance of the lender's token account and add the fee to it so we can save it to the loan account
      let balance = amount_from_token_acct(lender_token_acct)?;
      if balance == 0 {
        return Ee::LenderPdaBalanceIsZero.e();
      }
      if *amount > balance {
        return Ee::BorrowAmountTooBig.e();
      }

      let balance_with_fee = balance
        .checked_add(
          amount
            .checked_mul(fee as u64)
            .and_then(|x| x.checked_div(10_000))
            .ok_or_else(|| Ee::MultDivNone)?,
        )
        .ok_or_else(|| Ee::AddToOverflow)?;

      // Push the Loan struct to the loan account
      loan_records_slice[i] = LoanRecord {
        lender_token_acct: lender_token_acct.address().to_bytes(),
        balance_with_fee,
      };

      // Transfer the tokens from the lenderPda to the borrower
      pinocchio_token::instructions::TransferChecked {
        from: lender_token_acct,
        mint,
        to: borrower_token_acct,
        authority: lender_pda,
        amount: *amount,
        decimals,
      }
      .invoke_signed(signer_seeds)?;
    }

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

    let [signer, lender_pda, loan_records, mint, token_program, rent_sysvar, instruction_sysvar, token_accounts @ ..] =
      accounts
    else {
      return Err(ProgramError::NotEnoughAccountKeys);
    }; //lender_ata, user_ata
    check_signer(signer)?;
    writable(loan_records)?;
    executable(token_program)?;
    check_rent_sysvar(rent_sysvar)?;
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
      return Err(Ee::LoanRecordAcctHasData.into());
    }

    //-------== parse variadic data
    let (decimals, data) = data.split_first().ok_or_else(|| Ee::ByteSizeForU8)?;
    log!("decimals: {}", *decimals);

    let (bump, data) = data.split_first().ok_or_else(|| Ee::ByteSizeForU8)?;
    log!("bump: {}", *bump);

    let (fee, data) = data
      .split_at_checked(size_of::<u16>())
      .ok_or_else(|| Ee::ByteSizeForU16)?;
    let fee = u16::from_le_bytes(fee.try_into().map_err(|_| Ee::ByteSizeForU16)?);
    log!("fee: {}", fee);

    //Deriving the protocol PDA with the fee creates isolated liquidity pools for each fee tier, eliminating the need to store fee data in accounts. This design is both safe and optimal since each PDA with a specific fee owns only the liquidity associated with that fee rate. If someone passes an invalid fee, the corresponding token account for that fee bracket will be empty, automatically causing the transfer to fail with insufficient funds.

    if data.len() % size_of::<u64>() != 0 {
      return Err(Ee::DataArgLenForU64.into());
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
      bump_array: [*bump],
      fee,
      amounts,
    })
  }
}
