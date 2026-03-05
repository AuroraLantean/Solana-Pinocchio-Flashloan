use crate::{amount_from_token_acct, close_pda, instructions::check_signer, writable, Ee, Loan};
use core::convert::TryFrom;
use pinocchio::{error::ProgramError, AccountView, ProgramResult};
use pinocchio_log::log;

/// FlashloanRepay
pub struct FlashloanRepay<'a> {
  pub signer: &'a AccountView,
  pub loans_pda: &'a AccountView,
  pub txn_accts: &'a [AccountView],
}
impl<'a> FlashloanRepay<'a> {
  pub const DISCRIMINATOR: &'a u8 = &4;

  pub fn process(self) -> ProgramResult {
    log!("FlashloanRepay process()");
    let FlashloanRepay {
      signer,
      loans_pda,
      txn_accts,
    } = self;

    let loans_data = loans_pda.try_borrow()?;
    let loan_num = loans_data.len() / size_of::<Loan>();
    log!("loan_num: {}", loan_num);

    if loan_num.ne(&(txn_accts.len() / 3)) {
      return Ee::RepayTxnAcctsLen.e();
    }
    log!("Repay 3");

    // Process each pair of token atas with its amounts
    for i in 0..loan_num {
      log!("Repay loop i = {}", i);

      // Validate that vault_ata is the same as the one in the loan account
      let vault_ata = &txn_accts[i * 3 + 1];

      if unsafe { *(loans_data.as_ptr().add(i * size_of::<Loan>()) as *const [u8; 32]) }
        != vault_ata.address().to_bytes()
      {
        return Ee::RepayVaultAta.e();
      }

      // Check if the loan is already repaid
      let vault_balc = amount_from_token_acct(&vault_ata)?;
      log!("Repay vault_balc         : {}", vault_balc);

      let vault_balc_expected = unsafe {
        *(loans_data
          .as_ptr()
          .add(i * size_of::<Loan>() + size_of::<[u8; 32]>()) as *const u64)
      };
      log!("Repay vault_balc_expected: {}", vault_balc_expected);

      if vault_balc < vault_balc_expected {
        return Ee::RepayVaultBalcNotExpected.e();
      }
    }
    log!("Repay vault_balc_expected Ok");

    // Close the loan account and give back the lamports to the debtor
    drop(loans_data);
    close_pda(loans_pda, signer)?;
    log!("Repay: loans_pda closed");

    /*unsafe {
      *signer.try_borrow_mut() += *loans_pda.borrow_lamports_unchecked();
      loans_pda.close_unchecked();
    }*/
    //As you can see, for optimization purposes and by design, the repayment doesn't happen in this instruction. This is because the debtor can choose to repay the token account in another instruction, such as when performing a swap or executing a series of CPIs from their arbitrage program.
    Ok(())
  }
}
impl<'a> TryFrom<(&'a [u8], &'a [AccountView])> for FlashloanRepay<'a> {
  type Error = ProgramError;

  fn try_from(value: (&'a [u8], &'a [AccountView])) -> Result<Self, Self::Error> {
    log!("FlashloanRepay try_from");
    let (data, accounts) = value;
    log!("accounts len: {}, data len: {}", accounts.len(), data.len());

    let [signer, loans_pda, txn_accts @ ..] = accounts else {
      return Err(ProgramError::NotEnoughAccountKeys);
    };
    check_signer(signer)?;
    writable(loans_pda)?;

    Ok(Self {
      signer,
      loans_pda,
      txn_accts,
    })
  }
}
