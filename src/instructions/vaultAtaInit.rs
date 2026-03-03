use core::convert::TryFrom;
use pinocchio::{error::ProgramError, AccountView, ProgramResult};
use pinocchio_log::log;

use crate::{
  check_ata, check_atoken_gpvbd, check_data_len, check_mint0a, check_rent_sysvar, check_sysprog,
  executable, instructions::check_signer, parse_u16, rent_exempt_mint, rent_exempt_tokacct,
  writable,
};

/// Vault ATA Init
pub struct VaultAtaInit<'a> {
  pub signer: &'a AccountView,
  pub mint: &'a AccountView,
  pub vault: &'a AccountView,
  pub vault_tokacct: &'a AccountView,
  //pub config_pda: &'a AccountView,
  pub token_program: &'a AccountView,
  pub system_program: &'a AccountView,
  pub atoken_program: &'a AccountView,
  pub rent_sysvar: &'a AccountView,
}
impl<'a> VaultAtaInit<'a> {
  pub const DISCRIMINATOR: &'a u8 = &1;

  pub fn process(self) -> ProgramResult {
    let VaultAtaInit {
      signer,
      mint,
      vault,
      vault_tokacct,
      //config_pda,
      token_program,
      system_program,
      atoken_program: _,
      rent_sysvar,
    } = self;
    log!("---------== process()");
    //config_pda.check_borrow_mut()?;
    //let _config: &mut Config = Config::from_account_view(&config_pda)?;

    if vault_tokacct.is_data_empty() {
      log!("Make vault_tokacct");
      pinocchio_associated_token_account::instructions::Create {
        funding_account: signer,
        account: vault_tokacct,
        wallet: vault,
        mint,
        system_program,
        token_program,
      }
      .invoke()?;
      //Please upgrade to SPL Token 2022 for immutable owner support
    } else {
      log!("vault_tokacct has data");
      check_ata(vault_tokacct, vault, mint)?;
      rent_exempt_tokacct(vault_tokacct, rent_sysvar)?;
    }
    log!("Vault token account is found/verified");

    Ok(())
  }
}
impl<'a> TryFrom<(&'a [u8], &'a [AccountView])> for VaultAtaInit<'a> {
  type Error = ProgramError;

  fn try_from(value: (&'a [u8], &'a [AccountView])) -> Result<Self, Self::Error> {
    log!("VaultAtaInit try_from");
    let (data, accounts) = value;
    log!("accounts len: {}, data len: {}", accounts.len(), data.len());
    let data_len = 2;
    check_data_len(data, data_len)?;

    let [signer, vault, vault_tokacct, mint, token_program, system_program, atoken_program, rent_sysvar] =
      accounts
    else {
      return Err(ProgramError::NotEnoughAccountKeys);
    }; //config_pda
    check_signer(signer)?;
    executable(token_program)?;
    check_sysprog(system_program)?;
    check_atoken_gpvbd(atoken_program)?;
    check_rent_sysvar(rent_sysvar)?;
    log!("VaultAtaInit try_from 3");

    writable(vault_tokacct)?;
    writable(vault)?;
    //writable(config_pda)?;
    log!("VaultAtaInit try_from 4");

    let fee = parse_u16(&data[0..2])?;
    log!("fee: {}", fee);

    rent_exempt_mint(mint, rent_sysvar)?;

    log!("VaultAtaInit try_from 6");
    check_mint0a(mint, token_program)?;

    Ok(Self {
      signer,
      mint,
      vault,
      vault_tokacct,
      //config_pda,
      token_program,
      system_program,
      atoken_program,
      rent_sysvar,
    })
  }
}
