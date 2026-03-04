use crate::{
  amount_from_token_acct, check_pda, close_pda, instructions::check_signer, writable, Ee, Loan,
};
use core::convert::TryFrom;
use pinocchio::{error::ProgramError, AccountView, ProgramResult};
use pinocchio_log::log;

/// FlashloanRepay
pub struct FlashloanRepay<'a> {
  pub signer: &'a AccountView,
  pub vault: &'a AccountView,
  pub loan_array_pda: &'a AccountView,
  pub ata_array: &'a [AccountView],
}
impl<'a> FlashloanRepay<'a> {
  pub const DISCRIMINATOR: &'a u8 = &4;

  pub fn process(self) -> ProgramResult {
    log!("FlashloanRepay process()");
    let FlashloanRepay {
      signer,
      vault: _,
      loan_array_pda,
      ata_array,
    } = self;

    let loan_array_data = loan_array_pda.try_borrow()?;
    let loan_num = loan_array_data.len() / size_of::<Loan>();
    log!("loan_num: {}", loan_num);

    if loan_num.ne(&(ata_array.len() / 2)) {
      return Ee::RepayAtaArrayLen.e();
    }
    log!("Repay 3");

    // Process each pair of token atas with its amounts
    for i in 0..loan_num {
      log!("Repay loop i = {}", i);

      // Validate that vault_ata is the same as the one in the loan account
      let vault_ata = &ata_array[i * 2];

      if unsafe { *(loan_array_data.as_ptr().add(i * size_of::<Loan>()) as *const [u8; 32]) }
        != vault_ata.address().to_bytes()
      {
        return Ee::RepayVaultAta.e();
      }
      log!("Repay loan_array_data ok");
      // Check if the loan is already repaid
      let vault_balc = amount_from_token_acct(&vault_ata)?;
      log!("Repay vault_balc: {}", vault_balc);

      let vault_balc_expected = unsafe {
        *(loan_array_data
          .as_ptr()
          .add(i * size_of::<Loan>() + size_of::<[u8; 32]>()) as *const u64)
      };
      log!("Repay vault_balc_expected: {}", vault_balc_expected);

      if vault_balc < vault_balc_expected {
        return Ee::RepayVaultBalcNotExpected.e();
      }
    }
    log!("Repay after loop");

    // Close the loan account and give back the lamports to the debtor
    drop(loan_array_data);
    close_pda(loan_array_pda, signer)?;
    log!("Repay: loan_array_pda closed");

    /*unsafe {
      *signer.try_borrow_mut() += *loan_array_pda.borrow_lamports_unchecked();
      loan_array_pda.close_unchecked();
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

    let [signer, vault, loan_array_pda, ata_array @ ..] = accounts else {
      return Err(ProgramError::NotEnoughAccountKeys);
    };
    check_signer(signer)?;
    writable(vault)?;
    check_pda(vault)?;
    writable(loan_array_pda)?;

    Ok(Self {
      signer,
      vault,
      loan_array_pda,
      ata_array,
    })
  }
}
