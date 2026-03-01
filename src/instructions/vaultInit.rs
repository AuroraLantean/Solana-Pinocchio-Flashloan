use core::convert::TryFrom;
use pinocchio::{
  cpi::{Seed, Signer},
  error::ProgramError,
  sysvars::rent::Rent,
  AccountView, Address, ProgramResult,
};
use pinocchio_log::log;

use crate::{
  check_data_len, check_rent_sysvar, instructions::check_signer, writable, Ee, Vault, ID, PROG_ADDR,
};

/// Vault Init
pub struct VaultInit<'a> {
  pub signer: &'a AccountView,
  pub vault: &'a AccountView,
  //pub config_pda: &'a AccountView,
  pub system_program: &'a AccountView,
  pub rent_sysvar: &'a AccountView,
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
      vault_bump,
    } = self;
    log!("---------== process()");
    //config_pda.check_borrow_mut()?;
    //let _config: &mut Config = Config::from_account_view(&config_pda)?;

    let str_seed = "moon_pool".as_bytes(); //&[u8], Escrow::SEED
    let seed = [str_seed];
    let seeds = &seed[..];
    let (expected_pda, bump) = Address::find_program_address(seeds, &ID.into()); //TODO: may incur unknown cost
    if expected_pda.ne(vault.address()) {
      return Ee::NotMapped.e();
    }
    if bump != vault_bump {
      return Ee::InputDataBump.e();
    }

    if vault.is_data_empty() {
      log!("Make Vault PDA 1");
      let rent = Rent::from_account_view(rent_sysvar)?;
      let lamports = rent.try_minimum_balance(Vault::LEN)?;

      log!("Make Vault PDA 2");
      let seeds = [
        Seed::from(Vault::SEED),
        //Seed::from(signer.address().as_ref()),
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

    /*if vault_tokacct.is_data_empty() {
      log!("Make vault_tokacct");
      pinocchio_associated_token_account::instructions::Create {
        funding_account: signer,
        account: vault_tokacct,
        wallet: vault,
        mint: mint,
        system_program,
        token_program,
      }
      .invoke()?;
      //Please upgrade to SPL Token 2022 for immutable owner support
    } else {
      log!("vault_tokacct has data");
      check_ata_vault(vault_tokacct, vault, mint)?;
      rent_exempt_tokacct(vault_tokacct, rent_sysvar)?;
    }
    log!("Vault ATA is found/verified");*/

    Ok(())
  }
}
impl<'a> TryFrom<(&'a [u8], &'a [AccountView])> for VaultInit<'a> {
  type Error = ProgramError;

  fn try_from(value: (&'a [u8], &'a [AccountView])) -> Result<Self, Self::Error> {
    log!("VaultInit try_from");
    let (data, accounts) = value;
    log!("accounts len: {}, data len: {}", accounts.len(), data.len());
    let data_len = 26;
    //2x u8 takes 2 + 2x u64 takes 16 bytes
    check_data_len(data, data_len)?;

    let [signer, vault, system_program, rent_sysvar] = accounts else {
      return Err(ProgramError::NotEnoughAccountKeys);
    };
    check_signer(signer)?;
    //check_sysprog(system_program)?;
    check_rent_sysvar(rent_sysvar)?;
    log!("VaultInit try_from 3");

    writable(vault)?;
    //writable(config_pda)?;
    log!("VaultInit try_from 4");

    let vault_bump = data[0];
    let decimal = data[1];
    //let amount = parse_u64(&data[1..9])?;
    log!("vault_bump: {}, decimal: {}", vault_bump, decimal);
    //none_zero_u64(amount)?;

    log!("VaultInit try_from 5");

    Ok(Self {
      signer,
      vault,
      //config_pda,
      system_program,
      rent_sysvar,
      vault_bump,
    })
  }
}
