use core::convert::TryFrom;
use pinocchio::{error::ProgramError, AccountView, ProgramResult};
use pinocchio_log::log;

use crate::{
  ata_balc, check_ata, check_atoken_gpvbd, check_decimals, check_mint0a, check_pda,
  check_rent_sysvar, check_sysprog, executable, instructions::check_signer, none_zero_u64,
  rent_exempt_mint, rent_exempt_tokacct, writable, Ee,
};

/// TokLgcDeposit: investors to deposit tokens for lending. //TODO: make receipts for depositing
pub struct TokLgcDeposit<'a> {
  pub user: &'a AccountView, //signer
  pub from_ata: &'a AccountView,
  pub mint: &'a AccountView,
  //pub config_pda: &'a AccountView,
  pub token_program: &'a AccountView,
  pub system_program: &'a AccountView,
  pub atoken_program: &'a AccountView,
  pub rent_sysvar: &'a AccountView,
  pub txn_accts: &'a [AccountView],
  pub decimals: &'a u8,
  pub amounts: &'a [u64],
}
impl<'a> TokLgcDeposit<'a> {
  pub const DISCRIMINATOR: &'a u8 = &2;

  pub fn process(self) -> ProgramResult {
    let TokLgcDeposit {
      user,
      from_ata,
      mint,
      //config_pda: _,
      token_program,
      system_program,
      atoken_program: _,
      rent_sysvar,
      txn_accts,
      decimals,
      amounts,
    } = self;
    log!("TokLgcDeposit process()");

    for (i, amount) in amounts.iter().enumerate() {
      log!("tryFrom loop : i = {}", i);
      let vault = &txn_accts[i * 2];
      let vault_ata = &txn_accts[i * 2 + 1];
      none_zero_u64(*amount)?;

      if vault_ata.is_data_empty() {
        log!("Make vault_ata");
        pinocchio_associated_token_account::instructions::Create {
          funding_account: user,
          account: vault_ata,
          wallet: vault,
          mint,
          system_program,
          token_program,
        }
        .invoke()?;
        //Please upgrade to SPL Token 2022 for immutable owner support
      } else {
        log!("vault_ata has data");
        check_ata(vault_ata, vault, mint)?;
        rent_exempt_tokacct(vault_ata, rent_sysvar)?;
      }
      log!("Vault ATA is found/verified");

      pinocchio_token::instructions::TransferChecked {
        from: from_ata,
        mint,
        to: vault_ata,
        authority: user,
        amount: *amount,
        decimals: *decimals,
      }
      .invoke()?;
    }
    Ok(())
  }
}
impl<'a> TryFrom<(&'a [u8], &'a [AccountView])> for TokLgcDeposit<'a> {
  type Error = ProgramError;

  fn try_from(value: (&'a [u8], &'a [AccountView])) -> Result<Self, Self::Error> {
    log!("TokLgcDeposit try_from");
    let (data, accounts) = value;
    log!("accounts len: {}, data len: {}", accounts.len(), data.len());

    let [user, from_ata, mint, token_program, system_program, atoken_program, rent_sysvar, txn_accts @ ..] =
      accounts
    else {
      return Err(ProgramError::NotEnoughAccountKeys);
    }; //config_pda; txn_accts: [vault, vault_ata]
    check_signer(user)?;
    executable(token_program)?;
    check_sysprog(system_program)?;
    check_atoken_gpvbd(atoken_program)?;
    check_rent_sysvar(rent_sysvar)?;

    rent_exempt_mint(mint, rent_sysvar)?;
    check_mint0a(mint, token_program)?;
    //writable(config_pda)?;

    // Each txn_acct requires a vault, vault_ata
    let txn_len = txn_accts.len() / 2;
    log!("txn_len: {}", txn_len);
    if txn_len > 8 || txn_len == 0 {
      return Err(Ee::TxnLenInvalid.into());
    }
    if (txn_accts.len() % 2).ne(&0) {
      return Err(Ee::TxnAcctsLength.into());
    }
    for i in 0..txn_len {
      log!("tryFrom loop : i = {}", i);
      let vault = &txn_accts[i * 2];
      let vault_ata = &txn_accts[i * 2 + 1];
      writable(vault)?;
      writable(vault_ata)?;
      check_pda(vault)?;
    }
    log!("TokLgcDeposit try_from 5");

    //-------== parse variadic data
    //1+8: u8 takes 1, u64 takes 8 bytes
    //check_data_len(data, 9)?;
    let (decimals, data) = data.split_first().ok_or_else(|| Ee::ByteSizeForU8)?;
    log!("decimals: {}", *decimals);
    check_decimals(mint, *decimals)?;

    if data.len() % size_of::<u64>() != 0 {
      return Err(Ee::DataArgLenForU64.into());
    }
    let amounts: &[u64] = unsafe {
      core::slice::from_raw_parts(data.as_ptr() as *const u64, data.len() / size_of::<u64>())
    };
    //let amount = parse_u64(&data[1..])?;
    log!("amounts: {}", amounts);
    if amounts.len() != txn_len {
      return Err(Ee::AmountsLenVsTxnAcctsLen.into());
    }
    let sum: u64 = amounts.iter().sum();
    ata_balc(from_ata, sum)?;

    log!("TokLgcDeposit try_from 7");
    //config_pda.check_borrow_mut()?;
    //let config: &mut Config = Config::from_account_view(&config_pda)?;
    //check_vault(vault, config.vault())?;

    Ok(Self {
      user,
      from_ata,
      mint,
      //config_pda,
      token_program,
      system_program,
      atoken_program,
      rent_sysvar,
      txn_accts,
      decimals,
      amounts,
    })
  }
}
