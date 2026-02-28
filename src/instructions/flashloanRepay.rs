use crate::{amount_from_token_acct, close_pda, instructions::check_signer, Ee, LoanRecord};
use core::convert::TryFrom;
use pinocchio::{error::ProgramError, AccountView, ProgramResult};
use pinocchio_log::log;

/// FlashloanRepay
pub struct FlashloanRepay<'a> {
  pub signer: &'a AccountView,
  pub loan_records: &'a AccountView,
  pub token_accounts: &'a [AccountView],
}
impl<'a> FlashloanRepay<'a> {
  pub const DISCRIMINATOR: &'a u8 = &1;

  pub fn process(self) -> ProgramResult {
    log!("FlashloanRepay process()");
    let FlashloanRepay {
      signer,
      loan_records,
      token_accounts,
    } = self;
    //Check that all balances have been correctly repaid using the loan account.
    let loan_records_data = loan_records.try_borrow()?;
    let loan_num = loan_records_data.len() / size_of::<LoanRecord>();

    if loan_num.ne(&token_accounts.len()) {
      return Ee::RepayTokenAccountLen.e();
    }

    // Process each pair of token accounts (protocol, borrower) with corresponding amounts
    for i in 0..loan_num {
      // Validate that protocol_ata is the same as the one in the loan account
      let lender_token_acct = &token_accounts[i];

      if unsafe {
        *(loan_records_data.as_ptr().add(i * size_of::<LoanRecord>()) as *const [u8; 32])
      } != lender_token_acct.address().to_bytes()
      {
        return Err(ProgramError::InvalidAccountData);
      }

      // Check if the loan is already repaid
      let balance = amount_from_token_acct(&lender_token_acct)?;

      let loan_balance = unsafe {
        *(loan_records_data
          .as_ptr()
          .add(i * size_of::<LoanRecord>() + size_of::<[u8; 32]>()) as *const u64)
      };

      if balance < loan_balance {
        return Err(ProgramError::InvalidAccountData);
      }
    }

    // Close the loan account and give back the lamports to the borrower
    drop(loan_records_data);
    close_pda(loan_records, signer)?;

    /*unsafe {
      *signer.try_borrow_mut() += *loan_records.borrow_lamports_unchecked();
      loan_records.close_unchecked();
    }*/
    //As you can see, for optimization purposes and by design, the repayment doesn't happen in this instruction. This is because the borrower can choose to repay the token account in another instruction, such as when performing a swap or executing a series of CPIs from their arbitrage program.
    Ok(())
  }
}
impl<'a> TryFrom<(&'a [u8], &'a [AccountView])> for FlashloanRepay<'a> {
  type Error = ProgramError;

  fn try_from(value: (&'a [u8], &'a [AccountView])) -> Result<Self, Self::Error> {
    log!("FlashloanRepay try_from");
    let (data, accounts) = value;
    log!("accounts len: {}, data len: {}", accounts.len(), data.len());

    let [signer, loan_records, token_accounts @ ..] = accounts else {
      return Err(ProgramError::NotEnoughAccountKeys);
    };
    check_signer(signer)?;

    Ok(Self {
      signer,
      loan_records,
      token_accounts,
    })
  }
}
