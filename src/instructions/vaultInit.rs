use core::convert::TryFrom;
use pinocchio::{
  cpi::{Seed, Signer},
  error::ProgramError,
  sysvars::rent::Rent,
  AccountView, ProgramResult,
};
use pinocchio_log::log;

use crate::{
  check_data_len, check_rent_sysvar, check_sysprog, instructions::check_signer, none_zero_u8,
  parse_u16, writable, Ee, Vault, PROG_ADDR,
};

/// Vault Init
pub struct VaultInit<'a> {
  pub signer: &'a AccountView,
  pub vault: &'a AccountView,
  //pub config_pda: &'a AccountView,
  pub system_program: &'a AccountView,
  pub rent_sysvar: &'a AccountView,
  pub fee: u16,
  pub vault_bump: u8,
}
impl<'a> VaultInit<'a> {
  pub const DISCRIMINATOR: &'a u8 = &0;

  pub fn process(self) -> ProgramResult {
    let VaultInit {
      signer,
      vault,
      //config_pda,
      system_program: _,
      rent_sysvar,
      fee,
      vault_bump,
    } = self;
    log!("---------== process()");
    //config_pda.check_borrow_mut()?;
    //let _config: &mut Config = Config::from_account_view(&config_pda)?;

    if vault.is_data_empty() {
      log!("Make Vault PDA 1");
      let rent = Rent::from_account_view(rent_sysvar)?;
      let lamports = rent.try_minimum_balance(Vault::LEN)?;

      log!("Make Vault PDA 2");
      let fee_seed = fee.to_le_bytes();
      let seeds = [
        Seed::from(Vault::SEED),
        //Seed::from(signer.address().as_ref()),
        Seed::from(&fee_seed),
        Seed::from(core::slice::from_ref(&vault_bump)),
      ];
      let seed_signer = Signer::from(&seeds);

      pinocchio_system::instructions::CreateAccount {
        from: signer,
        to: vault,
        lamports,
        space: Vault::LEN as u64,
        owner: &PROG_ADDR,
      }
      .invoke_signed(&[seed_signer])?;
    } else {
      return Ee::VaultExists.e();
    }
    log!("Vault is made");

    vault.check_borrow_mut()?;
    let vault: &mut Vault = Vault::from_account_view(&vault)?;
    vault.set_bump(vault_bump)?;
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

    let [signer, vault, system_program, rent_sysvar] = accounts else {
      return Err(ProgramError::NotEnoughAccountKeys);
    };
    check_signer(signer)?;
    check_sysprog(system_program)?;
    check_rent_sysvar(rent_sysvar)?;
    log!("VaultInit try_from 3");

    writable(vault)?;
    //writable(config_pda)?;
    log!("VaultInit try_from 4");

    let vault_bump = data[0];
    log!("vault_bump: {}", vault_bump);
    none_zero_u8(vault_bump)?;

    let fee = parse_u16(&data[1..3])?;
    log!("fee: {}", fee);
    log!("VaultInit try_from 5");

    Ok(Self {
      signer,
      vault,
      //config_pda,
      system_program,
      rent_sysvar,
      fee,
      vault_bump,
    })
  }
}
