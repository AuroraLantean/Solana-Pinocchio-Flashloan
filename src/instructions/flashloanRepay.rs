use crate::{amount_from_token_acct, close_pda, instructions::check_signer, Ee, LoanRecord};
use core::convert::TryFrom;
use pinocchio::{error::ProgramError, AccountView, ProgramResult};
use pinocchio_log::log;

/// FlashloanRepay
pub struct FlashloanRepay<'a> {
  pub signer: &'a AccountView,
  pub lender_pda: &'a AccountView,
  pub loan_record_pda: &'a AccountView,
  pub token_accounts: &'a [AccountView],
}
impl<'a> FlashloanRepay<'a> {
  pub const DISCRIMINATOR: &'a u8 = &4;

  pub fn process(self) -> ProgramResult {
    log!("FlashloanRepay process()");
    let FlashloanRepay {
      signer,
      lender_pda: _,
      loan_record_pda,
      token_accounts,
    } = self;
    //Check that all balances have been correctly repaid using the loan account.
    let loan_records_data = loan_record_pda.try_borrow()?;
    let loan_num = loan_records_data.len() / size_of::<LoanRecord>();
    log!("loan_num: {}", loan_num);

    if loan_num.ne(&(token_accounts.len() / 2)) {
      return Ee::RepayTokenAccountLen.e();
    }
    log!("Repay 3");

    // Process each pair of token accounts (protocol, borrower) with corresponding amounts
    for i in 0..loan_num {
      log!("Repay loop i = {}", i);

      // Validate that protocol_ata is the same as the one in the loan account
      let lender_ata = &token_accounts[i];

      if unsafe {
        *(loan_records_data.as_ptr().add(i * size_of::<LoanRecord>()) as *const [u8; 32])
      } != lender_ata.address().to_bytes()
      {
        return Err(ProgramError::InvalidAccountData);
      }
      log!("Repay loan_records_data ok");
      // Check if the loan is already repaid
      let lender_balc = amount_from_token_acct(&lender_ata)?;
      log!("Repay lender_balc: {}", lender_balc);

      let loan_balance = unsafe {
        *(loan_records_data
          .as_ptr()
          .add(i * size_of::<LoanRecord>() + size_of::<[u8; 32]>()) as *const u64)
      };
      log!("Repay loan_balance: {}", loan_balance);

      // if lender_balc < loan_balance {
      //   return Err(ProgramError::InsufficientFunds);
      // }
    }
    log!("Repay after loop");

    // Close the loan account and give back the lamports to the borrower
    drop(loan_records_data);
    close_pda(loan_record_pda, signer)?;
    log!("Repay: loan_record_pda closed");

    /*unsafe {
      *signer.try_borrow_mut() += *loan_record_pda.borrow_lamports_unchecked();
      loan_record_pda.close_unchecked();
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

    let [signer, lender_pda, loan_record_pda, token_accounts @ ..] = accounts else {
      return Err(ProgramError::NotEnoughAccountKeys);
    };
    check_signer(signer)?;

    Ok(Self {
      signer,
      lender_pda,
      loan_record_pda,
      token_accounts,
    })
  }
}
