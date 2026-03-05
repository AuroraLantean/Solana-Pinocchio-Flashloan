use crate::{
  amount_from_token_acct, check_ata, check_instruction_sysvar, check_pda, check_rent_sysvar,
  check_sysprog, executable, instructions::check_signer, writable, Ee, FlashloanRepay, Loan, Loans,
  Vault, PROG_ADDR,
};
use core::convert::TryFrom;
use pinocchio::{
  address::address_eq,
  cpi::{Seed, Signer},
  error::ProgramError,
  sysvars::{instructions::Instructions, rent::Rent},
  AccountView, ProgramResult,
};
use pinocchio_log::log;

/// FlashloanBorrow
pub struct FlashloanBorrow<'a> {
  pub signer: &'a AccountView,
  pub loans_pda: &'a AccountView,
  pub mint: &'a AccountView,
  pub instruction_sysvar: &'a AccountView,
  //pub config_pda: &'a AccountView,
  //pub token_program: &'a AccountView,
  pub system_program: &'a AccountView,
  pub rent_sysvar: &'a AccountView,
  pub txn_accts: &'a [AccountView],
  pub decimals: u8,
  pub loans_bump_a: [u8; 1],
  pub vault_bump_a: [u8; 1],
  pub fee: u16,
  pub amounts: &'a [u64],
}
impl<'a> FlashloanBorrow<'a> {
  pub const DISCRIMINATOR: &'a u8 = &3;

  pub fn process(self) -> ProgramResult {
    log!("FlashloanBorrow process()");
    let FlashloanBorrow {
      signer,
      loans_pda,
      mint,
      instruction_sysvar,
      //config_pda: _,
      //token_program: _,
      system_program: _,
      rent_sysvar,
      txn_accts,
      decimals,
      loans_bump_a,
      vault_bump_a,
      fee,
      amounts,
    } = self;

    //-----------== Introspecting the Repay instruction
    let instruction_sysvar =
      unsafe { Instructions::new_unchecked(instruction_sysvar.try_borrow()?) };
    log!("Borrow 1");

    let num_instructions = instruction_sysvar.num_instructions();
    log!("num_instructions: {}", num_instructions);
    if num_instructions < 2 {
      return Ee::NumOfInstructions.e();
    }

    let repay_ix = instruction_sysvar.load_instruction_at(num_instructions as usize - 1)?;

    if repay_ix.get_program_id().to_bytes().ne(&crate::ID) {
      return Ee::RepayProgId.e();
    }
    log!("Borrow 4: Repay ProgId Ok");

    if unsafe { *(repay_ix.get_instruction_data().as_ptr()) } != *FlashloanRepay::DISCRIMINATOR {
      return Ee::RepayDiscriminator.e();
    }
    log!("Borrow 5: Repay Disc Ok");

    if unsafe {
      !address_eq(
        &repay_ix.get_instruction_account_at_unchecked(0).key,
        signer.address(),
      )
    } {
      return Ee::RepayIxSigner.e();
    }

    if unsafe {
      !address_eq(
        &repay_ix.get_instruction_account_at_unchecked(1).key,
        loans_pda.address(),
      )
    } {
      return Ee::RepayIxLoansPda.e();
    }

    for (i, amount) in amounts.iter().enumerate() {
      log!("Borrow loop checking: i = {}", i);
      if *amount == 0 {
        return Ee::BorrowedAmountIsZero.e();
      }
      //check vault
      if unsafe {
        !address_eq(
          &repay_ix.get_instruction_account_at_unchecked(i * 3 + 2).key,
          txn_accts[i * 3].address(),
        )
      } {
        return Ee::RepayIxVaultPda.e();
      }
      if unsafe {
        !address_eq(
          &repay_ix.get_instruction_account_at_unchecked(i * 3 + 3).key,
          txn_accts[i * 3 + 1].address(),
        )
      } {
        return Ee::RepayIxVaultAta.e();
      }
      if unsafe {
        !address_eq(
          &repay_ix.get_instruction_account_at_unchecked(i * 3 + 4).key,
          txn_accts[i * 3 + 2].address(),
        )
      } {
        return Ee::RepayIxDebtorAta.e();
      }
    }
    log!("Borrow 10: all txn_accts Ok");

    //-----------== send_tokens
    //Loans is derived from the seed string and debtor.
    let seeds = [
      Seed::from(Loans::SEED),
      Seed::from(signer.address().as_ref()),
      Seed::from(&loans_bump_a),
    ];
    let loans_signer_seeds = &[Signer::from(&seeds)];
    log!("Borrow 7a");

    //Each vault is derived from the seed string and fee. Thus each PDA owns only the liquidity associated with that fee rate.
    let fee_bytes = fee.to_le_bytes();
    let vault_seeds = [
      Seed::from(Vault::SEED),
      Seed::from(&fee_bytes),
      Seed::from(&vault_bump_a),
    ];
    let vault_signer_seeds = &[Signer::from(&vault_seeds)];
    log!("Borrow 7b");

    // Make a mutable slice to save the Loan structs
    let loans_size = size_of::<Loan>() * amounts.len(); //40 = 32 + 8
    log!("Borrow 8. loans_size: {}", loans_size);

    let rent = Rent::from_account_view(rent_sysvar)?;
    let lamports = rent.try_minimum_balance(loans_size)?;
    log!("Borrow 9");

    pinocchio_system::instructions::CreateAccount {
      from: signer,
      to: loans_pda,
      lamports,
      space: loans_size as u64,
      owner: &PROG_ADDR,
    }
    .invoke_signed(loans_signer_seeds)?;
    log!("Borrow 10: Loans initialized");

    //Make a mutable slice from the Loans data. in a loop, add Loan to this slice and transfer tokens:
    let mut loans_data = loans_pda.try_borrow_mut()?;
    let loans = unsafe {
      core::slice::from_raw_parts_mut(loans_data.as_mut_ptr() as *mut Loan, amounts.len())
    };
    log!("Borrow 11");

    //loop through all the loans. In each iteration, we get the vault_ata and debtor_ata, calculate the balance due to the protocol, save this data in the loansPDA, and transfer the tokens.
    for (i, amount) in amounts.iter().enumerate() {
      log!("Borrow loop token sending: i = {}", i);
      let vault = &txn_accts[i * 3];
      let vault_ata = &txn_accts[i * 3 + 1];
      let debtor_ata = &txn_accts[i * 3 + 2];

      // Get the vault_ata_balc and add the fee to it so we can save it to the loan account
      let vault_ata_balc = amount_from_token_acct(vault_ata)?;
      log!("vault_ata balc: {}", vault_ata_balc);
      if vault_ata_balc == 0 {
        return Ee::VaultAtaBalcZero.e();
      }
      if *amount > vault_ata_balc {
        return Ee::BorrowAmountTooBig.e();
      }

      let balc_plus_fee = vault_ata_balc
        .checked_add(
          amount
            .checked_mul(fee as u64)
            .and_then(|x| x.checked_div(10_000))
            .ok_or_else(|| Ee::MultDivNone)?,
        )
        .ok_or_else(|| Ee::AddToOverflow)?;
      log!("balc_plus_fee: {}", balc_plus_fee);

      // Save the Loan to Loans
      loans[i] = Loan {
        vault_ata: vault_ata.address().to_bytes(),
        balc_plus_fee,
      };
      log!("to transfer tokens");

      // Transfer the tokens from vault to the debtor
      pinocchio_token::instructions::TransferChecked {
        from: vault_ata,
        mint,
        to: debtor_ata,
        authority: vault,
        amount: *amount,
        decimals,
      }
      .invoke_signed(vault_signer_seeds)?;
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

    let [signer, loans_pda, mint, token_program, system_program, rent_sysvar, instruction_sysvar, txn_accts @ ..] =
      accounts
    else {
      return Err(ProgramError::NotEnoughAccountKeys);
    };
    check_signer(signer)?;
    writable(loans_pda)?;
    executable(token_program)?;
    check_sysprog(system_program)?;
    check_rent_sysvar(rent_sysvar)?;
    //writable(config_pda)?;
    //check_pda(config_pda)?;
    //check_mint0a(mint, token_program)?;
    check_instruction_sysvar(instruction_sysvar)?;

    // Each loan requires a vault, vault_ata, and a debtor_ata
    if (txn_accts.len() % 3).ne(&0) || txn_accts.len().eq(&0) {
      return Err(Ee::TxnAcctsLength.into());
    }
    if loans_pda.try_borrow()?.len().ne(&0) {
      return Err(Ee::LoansPdaHasData.into());
    }

    //-------== parse variadic data
    let (decimals, data) = data.split_first().ok_or_else(|| Ee::ByteSizeForU8)?;
    log!("decimals: {}", *decimals);

    let (loans_bump, data) = data.split_first().ok_or_else(|| Ee::ByteSizeForU8)?;
    log!("loans_bump: {}", *loans_bump);

    let (vault_bump, data) = data.split_first().ok_or_else(|| Ee::ByteSizeForU8)?;
    log!("vault_bump: {}", *vault_bump);

    let (fee, data) = data
      .split_at_checked(size_of::<u16>())
      .ok_or_else(|| Ee::ByteSizeForU16)?;
    let fee = u16::from_le_bytes(fee.try_into().map_err(|_| Ee::ByteSizeForU16)?);
    log!("fee: {}", fee);

    if data.len() % size_of::<u64>() != 0 {
      return Err(Ee::DataArgLenForU64.into());
    }
    // Get the amount slice
    let amounts: &[u64] = unsafe {
      core::slice::from_raw_parts(data.as_ptr() as *const u64, data.len() / size_of::<u64>())
    };
    log!("amounts: {}", amounts);
    if amounts.len() != txn_accts.len() / 3 {
      return Err(Ee::AmountsLenVsTxnAcctsLen.into());
    }

    for (i, _) in amounts.iter().enumerate() {
      log!("tryFrom loop : i = {}", i);
      let vault = &txn_accts[i * 3];
      let vault_ata = &txn_accts[i * 3 + 1];
      let debtor_ata = &txn_accts[i * 3 + 2];

      writable(vault)?;
      check_pda(vault)?;
      check_ata(vault_ata, vault, mint)?;
      check_ata(debtor_ata, signer, mint)?;
    }
    Ok(Self {
      signer,
      loans_pda,
      mint,
      instruction_sysvar,
      //config_pda,
      //token_program,
      system_program,
      rent_sysvar,
      txn_accts,
      decimals: *decimals,
      loans_bump_a: [*loans_bump],
      vault_bump_a: [*vault_bump],
      fee,
      amounts,
    })
  }
}
