use core::{convert::TryFrom, mem::MaybeUninit, slice::from_raw_parts};
use pinocchio::{
  cpi::invoke_signed,
  error::ProgramError,
  instruction::{InstructionAccount, InstructionView},
  AccountView, ProgramResult,
};
use pinocchio_log::log;

use crate::{check_sysprog, instructions::check_signer, Ee};

/// InitAnchorPdaCaller
pub struct InitAnchorPdaCaller<'a> {
  pub signer: &'a AccountView,
  pub target_prog: &'a AccountView,
  pub vaults: &'a [AccountView],
  //pub config_pda: &'a AccountView,
  pub system_program: &'a AccountView,
  //pub rent_sysvar: &'a AccountView,
  pub accts_len: &'a u8,
  pub ix_data: &'a [u8],
}
impl<'a> InitAnchorPdaCaller<'a> {
  pub const DISCRIMINATOR: &'a u8 = &6;

  pub fn process(self) -> ProgramResult {
    let InitAnchorPdaCaller {
      signer,
      target_prog,
      vaults,
      //config_pda,
      system_program,
      //rent_sysvar,
      accts_len,
      ix_data,
    } = self;
    log!("---------== process()");
    if vaults.len() != 1 {
      return Ee::TxnAcctsLength.e();
    }
    log!("InitAnchorPdaCaller 1");
    const MAX_ACCT_LEN: usize = 15;
    let mut instruction_accounts =
      [const { MaybeUninit::<InstructionAccount>::uninit() }; MAX_ACCT_LEN];

    instruction_accounts[0].write(InstructionAccount::writable((vaults[0]).address()));
    instruction_accounts[1].write(InstructionAccount::writable_signer(signer.address()));
    instruction_accounts[2].write(InstructionAccount::readonly(system_program.address()));
    //instruction_accounts[3].write(InstructionAccount::readonly(rent_sysvar.address()));
    //instruction_accounts[4].write(InstructionAccount::writable((vaults[1]).address()));

    log!("InitAnchorPdaCaller 2");
    let account_views = &[&vaults[0], signer, system_program]; //rent_sysvar

    //let ix_data_size = 7;
    // write_bytes(&mut ix_data[1..9], &self.amount.to_le_bytes());

    log!("InitAnchorPdaCaller 6");
    let instruction = InstructionView {
      program_id: target_prog.address(),
      accounts: unsafe { from_raw_parts(instruction_accounts.as_ptr() as _, *accts_len as usize) }, //&instruction_accounts,
      data: ix_data, //unsafe { from_raw_parts(ix_data.as_ptr() as _, *ix_data_size as usize) },
    };
    log!("InitAnchorPdaCaller 7");
    invoke_signed(&instruction, account_views, &[])?;
    Ok(())
  }
}
impl<'a> TryFrom<(&'a [u8], &'a [AccountView])> for InitAnchorPdaCaller<'a> {
  type Error = ProgramError;

  fn try_from(value: (&'a [u8], &'a [AccountView])) -> Result<Self, Self::Error> {
    log!("InitAnchorPdaCaller try_from");
    let (data, accounts) = value;
    log!("accounts len: {}, data len: {}", accounts.len(), data.len());
    //let data_len = 3;
    //check_data_len(data, data_len)?;

    let [signer, target_prog, system_program, vaults @ ..] = accounts else {
      return Err(ProgramError::NotEnoughAccountKeys);
    }; //rent_sysvar
    check_signer(signer)?;
    check_sysprog(system_program)?;
    //check_rent_sysvar(rent_sysvar)?;
    log!("InitAnchorPdaCaller try_from 3");
    //writable(config_pda)?;

    // Each txn_acct requires a vault, vault_ata
    let txn_len = vaults.len();
    log!("txn_len: {}", txn_len);
    if txn_len > 8 || txn_len == 0 {
      return Err(Ee::TxnLenInvalid.into());
    }

    //-------== parse variadic data
    let (accts_len, ix_data) = data.split_first().ok_or_else(|| Ee::ByteSizeForU8)?;
    log!("accts_len: {}", *accts_len);
    log!("ix_data: {}", ix_data);

    Ok(Self {
      signer,
      target_prog,
      vaults,
      //config_pda,
      system_program,
      //rent_sysvar,
      accts_len,
      ix_data,
    })
  }
}
