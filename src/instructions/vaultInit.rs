use core::convert::TryFrom;
use pinocchio::{
  cpi::{Seed, Signer},
  error::ProgramError,
  sysvars::rent::Rent,
  AccountView, ProgramResult,
};
use pinocchio_log::log;

use crate::{
  check_data_len, check_rent_sysvar, check_sysprog, instructions::check_signer, none_zero_u16,
  none_zero_u8, writable, Ee, Vault, PROG_ADDR,
};

/// Vault Init
pub struct VaultInit<'a> {
  pub signer: &'a AccountView,
  pub vaults: &'a [AccountView],
  //pub config_pda: &'a AccountView,
  pub system_program: &'a AccountView,
  pub rent_sysvar: &'a AccountView,
  pub fees: &'a [u16],
  pub vault_bumps: &'a [u8],
}
impl<'a> VaultInit<'a> {
  pub const DISCRIMINATOR: &'a u8 = &0;

  pub fn process(self) -> ProgramResult {
    let VaultInit {
      signer,
      vaults,
      //config_pda,
      system_program: _,
      rent_sysvar,
      fees,
      vault_bumps,
    } = self;
    log!("---------== process()");
    //config_pda.check_borrow_mut()?;
    //let _config: &mut Config = Config::from_account_view(&config_pda)?;

    for (i, vault) in vaults.iter().enumerate() {
      log!("tryFrom loop : i = {}", i);
      let fee = &fees[i];
      let vault_bump = &vault_bumps[i];

      log!("Make Vault PDA 1");
      let rent = Rent::from_account_view(rent_sysvar)?;
      let lamports = rent.try_minimum_balance(Vault::LEN)?;

      log!("Make Vault PDA 2");
      let fee_bytes = fee.to_le_bytes();
      let seeds = [
        Seed::from(Vault::SEED),
        //signer.address().as_ref(),
        Seed::from(&fee_bytes),
        Seed::from(core::slice::from_ref(vault_bump)),
      ];
      let signer_seeds = &[Signer::from(&seeds)];

      pinocchio_system::instructions::CreateAccount {
        from: signer,
        to: vault,
        lamports,
        space: Vault::LEN as u64,
        owner: &PROG_ADDR,
      }
      .invoke_signed(signer_seeds)?;
      log!("Vault is made");

      vault.check_borrow_mut()?;
      let vault: &mut Vault = Vault::from_account_view(&vault)?;
      vault.set_bump(*vault_bump)?;
    }
    Ok(())
  }
}
impl<'a> TryFrom<(&'a [u8], &'a [AccountView])> for VaultInit<'a> {
  type Error = ProgramError;

  fn try_from(value: (&'a [u8], &'a [AccountView])) -> Result<Self, Self::Error> {
    log!("VaultInit try_from");
    let (data, accounts) = value;
    log!("accounts len: {}, data len: {}", accounts.len(), data.len());
    let data_len = 3;
    check_data_len(data, data_len)?;

    let [signer, system_program, rent_sysvar, vaults @ ..] = accounts else {
      return Err(ProgramError::NotEnoughAccountKeys);
    };
    check_signer(signer)?;
    check_sysprog(system_program)?;
    check_rent_sysvar(rent_sysvar)?;
    log!("VaultInit try_from 3");
    //writable(config_pda)?;

    // Each txn_acct requires a vault, vault_ata
    let txn_len = vaults.len();
    log!("txn_len: {}", txn_len);
    if txn_len > 8 || txn_len == 0 {
      return Err(Ee::TxnLenInvalid.into());
    }
    log!("VaultInit try_from 4");

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
    log!("VaultInit try_from 5");

    Ok(Self {
      signer,
      vaults,
      //config_pda,
      system_program,
      rent_sysvar,
      fees,
      vault_bumps,
    })
  }
}
