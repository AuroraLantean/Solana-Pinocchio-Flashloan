use core::convert::TryFrom;
use pinocchio::{
  cpi::invoke_signed,
  error::ProgramError,
  instruction::{InstructionAccount, InstructionView},
  AccountView, ProgramResult,
};
use pinocchio_log::log;

use crate::{
  check_rent_sysvar, check_sysprog, instructions::check_signer, none_zero_u16, none_zero_u8,
  writable, Ee,
};

/// FuncCaller
pub struct FuncCaller<'a> {
  pub signer: &'a AccountView,
  pub target_prog: &'a AccountView,
  pub vaults: &'a [AccountView],
  //pub config_pda: &'a AccountView,
  pub system_program: &'a AccountView,
  pub rent_sysvar: &'a AccountView,
  pub fees: &'a [u16],
  pub vault_bumps: &'a [u8],
}
impl<'a> FuncCaller<'a> {
  pub const DISCRIMINATOR: &'a u8 = &5;

  pub fn process(self) -> ProgramResult {
    let FuncCaller {
      signer,
      target_prog,
      vaults,
      //config_pda,
      system_program,
      rent_sysvar,
      fees,
      vault_bumps,
    } = self;
    log!("---------== process()");
    if vaults.len() != 2 {
      return Ee::TxnAcctsLength.e();
    }
    log!("FuncCaller 1");
    let instruction_accounts: [InstructionAccount; 5] = [
      InstructionAccount::writable_signer(signer.address()),
      InstructionAccount::readonly(system_program.address()),
      InstructionAccount::readonly(rent_sysvar.address()),
      InstructionAccount::writable((vaults[0]).address()),
      InstructionAccount::writable((vaults[1]).address()),
    ];
    log!("FuncCaller 2");
    let account_views = &[signer, system_program, rent_sysvar, &vaults[0], &vaults[1]];

    // Instruction data layout:
    // - [0 ]: Pinocchio func discriminator
    // - [1..3 ]: vault_bumps
    // - [3..7 ]: feess
    const LEN: usize = 7;
    let mut instruction_data = [0u8; LEN];

    log!("FuncCaller 4");
    // Set discriminator 0 as u8 at index 0
    instruction_data[0] = 0;
    instruction_data[1..1 + vault_bumps.len()].copy_from_slice(vault_bumps);

    log!(
      "FuncCaller 5. instruction_data: {}, len(): {}",
      &instruction_data,
      instruction_data.len()
    );
    //[3..7] vault_bumps (2x1 bytes, u8)
    for (idx, _bump) in vault_bumps.iter().enumerate() {
      log!("index: {}, instruction_data: {}", idx, &instruction_data);
      //instruction_data[idx + 1] = *bump;
      let fee = fees[idx].to_le_bytes();
      instruction_data[idx * 2 + 3..idx * 2 + 5].copy_from_slice(&fee);
    }
    // Set amount as u64 at offset [1..9]
    //write_bytes(&mut instruction_data[1..9], &self.amount.to_le_bytes());

    log!("FuncCaller 6");
    let instruction = InstructionView {
      program_id: target_prog.address(),
      accounts: &instruction_accounts,
      data: &instruction_data, //unsafe { from_raw_parts(instruction_data.as_ptr() as _, LEN) },
    };
    log!("FuncCaller 7");
    invoke_signed(&instruction, account_views, &[])?;
    Ok(())
  }
}
impl<'a> TryFrom<(&'a [u8], &'a [AccountView])> for FuncCaller<'a> {
  type Error = ProgramError;

  fn try_from(value: (&'a [u8], &'a [AccountView])) -> Result<Self, Self::Error> {
    log!("FuncCaller try_from");
    let (data, accounts) = value;
    log!("accounts len: {}, data len: {}", accounts.len(), data.len());
    //let data_len = 3;
    //check_data_len(data, data_len)?;

    let [signer, target_prog, system_program, rent_sysvar, vaults @ ..] = accounts else {
      return Err(ProgramError::NotEnoughAccountKeys);
    };
    check_signer(signer)?;
    check_sysprog(system_program)?;
    check_rent_sysvar(rent_sysvar)?;
    log!("FuncCaller try_from 3");
    //writable(config_pda)?;

    // Each txn_acct requires a vault, vault_ata
    let txn_len = vaults.len();
    log!("txn_len: {}", txn_len);
    if txn_len > 8 || txn_len == 0 {
      return Err(Ee::TxnLenInvalid.into());
    }
    log!("FuncCaller try_from 4");

    //-------== parse variadic data
    let (vault_bumps, data) = data
      .split_at_checked(txn_len)
      .ok_or_else(|| Ee::ByteSizeVaultBumps)?;
    log!("vault_bumps: {}", vault_bumps);

    let (fees_slice, data) = data
      .split_at_checked(size_of::<u16>() * txn_len)
      .ok_or_else(|| Ee::ByteSizeFees)?;

    let fees: &[u16] = unsafe {
      core::slice::from_raw_parts(
        fees_slice.as_ptr() as *const u16,
        fees_slice.len() / size_of::<u16>(),
      )
    };
    log!("fees: {}", fees);
    if data.len() > 0 {
      return Err(Ee::InputDataLen.into());
    }

    for (i, vault) in vaults.iter().enumerate() {
      log!("tryFrom loop : i = {}", i);
      writable(vault)?;
      if !vault.is_data_empty() {
        return Err(Ee::VaultExists.into());
      }
      none_zero_u8(vault_bumps[i])?;
      none_zero_u16(fees[i])?;
    }

    Ok(Self {
      signer,
      target_prog,
      vaults,
      //config_pda,
      system_program,
      rent_sysvar,
      fees,
      vault_bumps,
    })
  }
}
