//use num_derive::FromPrimitive;
use pinocchio::{
  error::{ProgramError, ToStr},
  sysvars::rent::{Rent, RENT_ID},
  AccountView, Address, ProgramResult,
};
use pinocchio_log::log; //logger::log_message
use pinocchio_token::state::{Mint, TokenAccount};
//use pinocchio_token_2022::state::{Mint as Mint22, TokenAccount as TokenAccount22};
//use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;
use thiserror::Error;

//TODO: put errors in error.rs ... https://learn.blueshift.gg/en/courses/pinocchio-for-dummies/pinocchio-errors
#[derive(Clone, Debug, Eq, Error, PartialEq)] //FromPrimitive
pub enum Ee {
  #[error("MethodDiscriminator")]
  MethodDiscriminator,
  #[error("Xyz001")]
  Xyz001,
  #[error("OnlyProgOwner")]
  OnlyProgOwner,
  #[error("OnlyAdmin")]
  OnlyAdmin,
  #[error("OnlyUser")]
  OnlyUser,
  #[error("Xyz005")]
  Xyz005,
  #[error("Xyz006")]
  Xyz006,
  #[error("NotWritable")]
  NotWritable,
  #[error("NotExecutable")]
  NotExecutable,
  #[error("TokenProgram")]
  TokenProgram,
  #[error("AtokenGPvbd")]
  AtokenGPvbd,
  #[error("SystemProgram")]
  SystemProgram,
  #[error("RentSysvar")]
  RentSysvar,
  #[error("Xyz013")]
  Xyz013,
  #[error("Xyz014")]
  Xyz014,
  //Bytes for Numerical
  #[error("ZeroU128")]
  ZeroU128,
  #[error("ZeroU64")]
  ZeroU64,
  #[error("ZeroU32")]
  ZeroU32,
  #[error("ZeroU16")]
  ZeroU16,
  #[error("ZeroU8")]
  ZeroU8,
  //Bytes Sizes
  #[error("ByteSizeForU128")]
  ByteSizeForU128,
  #[error("ByteSizeForU64")]
  ByteSizeForU64,
  #[error("ByteSizeForU32")]
  ByteSizeForU32,
  #[error("ByteSizeForU16")]
  ByteSizeForU16,
  #[error("ByteSizeForU8")]
  ByteSizeForU8,
  //Byte Slice Sizes
  #[error("ByteSliceSize128")]
  ByteSliceSize128,
  #[error("ByteSliceSize64")]
  ByteSliceSize64,
  #[error("ByteSliceSize32")]
  ByteSliceSize32,
  #[error("ByteSliceSize10")]
  ByteSliceSize10,
  #[error("ByteSliceSize6")]
  ByteSliceSize6,

  //Inputs
  #[error("InputDataLen")]
  InputDataLen,
  #[error("InputDataBump")]
  InputDataBump,

  //PDA
  #[error("PdaNoLamport")]
  PdaNoLamport,
  #[error("ForeignPDA")]
  ForeignPDA,
  #[error("VaultDataLengh")]
  VaultDataLengh,
  #[error("VaultIsForeign")]
  VaultIsForeign,

  //Mint Account
  #[error("DecimalsValue")]
  DecimalsValue,
  #[error("MintDataLen")]
  MintDataLen,
  #[error("MintOrMintAuthority")]
  MintOrMintAuthority,
  #[error("MintOrTokenProgram")]
  MintOrTokenProgram,

  //ATA
  #[error("AtaDataLen")]
  AtaDataLen,
  #[error("Ata22DataLen")]
  Ata22DataLen,
  #[error("AtaOrOwner")]
  AtaOrOwner,
  #[error("AtaOrMint")]
  AtaOrMint,
  #[error("AtaCheckX1")]
  AtaCheckX1,
  #[error("ForeignAta")]
  ForeignAta,
  #[error("AtaHasNoData")]
  AtaHasNoData,
  #[error("TokenAcctOwner")]
  TokenAcctOwner,
  #[error("TokenAcctDataLen")]
  TokenAcctDataLen,
  #[error("Xyz049")]
  Xyz049,

  #[error("NoRentExemptTokAcct")]
  NoRentExemptTokAcct,
  #[error("NoRentExemptTokAcct22")]
  NoRentExemptTokAcct22,
  #[error("NoRentExemptMint22")]
  NoRentExemptMint22,
  #[error("NoRentExemptMint")]
  NoRentExemptMint,
  #[error("Xyz054")]
  Xyz054,
  #[error("Xyz055")]
  Xyz055,
  #[error("Xyz056")]
  Xyz056,
  #[error("Xyz057")]
  Xyz057,
  #[error("Xyz058")]
  Xyz058,
  #[error("Xyz059")]
  Xyz059,

  //Math
  #[error("AddToOverflow")]
  AddToOverflow,
  #[error("MultDivNone")]
  MultDivNone,
  #[error("MultiplyOverflow")]
  MultiplyOverflow,
  #[error("DividedByZero")]
  DividedByZero,
  #[error("Remainder")]
  Remainder,
  //Misc...
  #[error("EmptyData")]
  EmptyData,
  #[error("ClockGet")]
  ClockGet,
  #[error("VaultExists")]
  VaultExists,
  #[error("Xyz068")]
  Xyz068,
  #[error("Xyz069")]
  Xyz069,

  //Flashloan
  #[error("TokenAcctsLength")]
  TokenAcctsLength,
  #[error("LoanRecordAcctHasData")]
  LoanRecordAcctHasData,
  #[error("DataArgLenForU64")]
  DataArgLenForU64,
  #[error("AmountsLenVsTokenAcctLen")]
  AmountsLenVsTokenAcctLen,
  #[error("BorrowAmountTooBig")]
  BorrowAmountTooBig,
  #[error("BorrowedAmountIsZero")]
  BorrowedAmountIsZero,
  #[error("LenderPdaBalanceIsZero")]
  LenderPdaBalanceIsZero,
  #[error("NumOfInstructions")]
  NumOfInstructions,
  #[error("RepayProgId")]
  RepayProgId,
  #[error("RepayDiscriminator")]
  RepayDiscriminator,
  #[error("RepayIxLenderPda")]
  RepayIxLenderPda,
  #[error("RepayTokenAccountLen")]
  RepayTokenAccountLen,
  //Final variant
  #[error("NotMapped")]
  NotMapped,
  //ProgramResult: AccountBorrowFailed
}
impl Ee {
  pub fn e(self) -> ProgramResult {
    Err(ProgramError::Custom(self as u32))
  }
}
impl From<Ee> for ProgramError {
  fn from(e: Ee) -> Self {
    ProgramError::Custom(e as u32)
  }
}
//Deserialize Errors from Raw Values
impl TryFrom<u32> for Ee {
  type Error = ProgramError;
  fn try_from(error: u32) -> Result<Self, Self::Error> {
    match error {
      0 => Ok(Ee::MethodDiscriminator),
      1 => Ok(Ee::Xyz001),
      2 => Ok(Ee::OnlyProgOwner),
      3 => Ok(Ee::OnlyAdmin),
      4 => Ok(Ee::OnlyUser),
      5 => Ok(Ee::Xyz005),
      6 => Ok(Ee::Xyz006),
      7 => Ok(Ee::NotWritable),
      8 => Ok(Ee::NotExecutable),
      9 => Ok(Ee::TokenProgram),
      10 => Ok(Ee::AtokenGPvbd),
      11 => Ok(Ee::SystemProgram),
      12 => Ok(Ee::RentSysvar),
      13 => Ok(Ee::Xyz013),
      14 => Ok(Ee::Xyz014),
      15 => Ok(Ee::ZeroU128),
      16 => Ok(Ee::ZeroU64),
      17 => Ok(Ee::ZeroU32),
      18 => Ok(Ee::ZeroU16),
      19 => Ok(Ee::ZeroU8),
      20 => Ok(Ee::ByteSizeForU128),
      21 => Ok(Ee::ByteSizeForU64),
      22 => Ok(Ee::ByteSizeForU32),
      23 => Ok(Ee::ByteSizeForU16),
      24 => Ok(Ee::ByteSizeForU8),
      25 => Ok(Ee::ByteSliceSize128),
      26 => Ok(Ee::ByteSliceSize64),
      27 => Ok(Ee::ByteSliceSize32),
      28 => Ok(Ee::ByteSliceSize10),
      29 => Ok(Ee::ByteSliceSize6),

      30 => Ok(Ee::InputDataLen),
      31 => Ok(Ee::InputDataBump),
      32 => Ok(Ee::PdaNoLamport),
      33 => Ok(Ee::ForeignPDA),
      34 => Ok(Ee::VaultIsForeign),
      35 => Ok(Ee::VaultDataLengh),
      36 => Ok(Ee::DecimalsValue),
      37 => Ok(Ee::MintDataLen),
      38 => Ok(Ee::MintOrMintAuthority),
      39 => Ok(Ee::MintOrTokenProgram),

      40 => Ok(Ee::AtaDataLen),
      41 => Ok(Ee::Ata22DataLen),
      42 => Ok(Ee::AtaOrOwner),
      43 => Ok(Ee::AtaOrMint),
      44 => Ok(Ee::AtaCheckX1),
      45 => Ok(Ee::ForeignAta),
      46 => Ok(Ee::AtaHasNoData),
      47 => Ok(Ee::TokenAcctOwner),
      48 => Ok(Ee::TokenAcctDataLen),
      49 => Ok(Ee::Xyz049),

      50 => Ok(Ee::NoRentExemptTokAcct),
      51 => Ok(Ee::NoRentExemptTokAcct22),
      52 => Ok(Ee::NoRentExemptMint22),
      53 => Ok(Ee::NoRentExemptMint),
      54 => Ok(Ee::Xyz054),
      55 => Ok(Ee::Xyz055),
      56 => Ok(Ee::Xyz056),
      57 => Ok(Ee::Xyz057),
      58 => Ok(Ee::Xyz058),
      59 => Ok(Ee::Xyz059),

      60 => Ok(Ee::AddToOverflow),
      61 => Ok(Ee::MultDivNone),
      62 => Ok(Ee::MultiplyOverflow),
      63 => Ok(Ee::DividedByZero),
      64 => Ok(Ee::Remainder),
      65 => Ok(Ee::EmptyData),
      66 => Ok(Ee::ClockGet),
      67 => Ok(Ee::VaultExists),
      68 => Ok(Ee::Xyz068),
      69 => Ok(Ee::Xyz069),

      70 => Ok(Ee::TokenAcctsLength),
      71 => Ok(Ee::LoanRecordAcctHasData),
      72 => Ok(Ee::DataArgLenForU64),
      73 => Ok(Ee::AmountsLenVsTokenAcctLen),
      74 => Ok(Ee::BorrowAmountTooBig),
      75 => Ok(Ee::BorrowedAmountIsZero),
      76 => Ok(Ee::LenderPdaBalanceIsZero),
      77 => Ok(Ee::NumOfInstructions),
      78 => Ok(Ee::RepayProgId),
      79 => Ok(Ee::RepayDiscriminator),
      80 => Ok(Ee::RepayIxLenderPda),
      81 => Ok(Ee::RepayTokenAccountLen),
      _ => Err(Ee::NotMapped.into()),
    }
  }
}
//Human Readable Errors; TODO: arrange below
impl ToStr for Ee {
  fn to_str(&self) -> &'static str {
    match self {
      Ee::MethodDiscriminator => "MethodDiscriminator",
      Ee::Xyz001 => "Xyz001",
      Ee::OnlyProgOwner => "OnlyProgOwner",
      Ee::OnlyAdmin => "OnlyAdmin",
      Ee::OnlyUser => "OnlyUser",
      Ee::Xyz005 => "Xyz005",
      Ee::Xyz006 => "Xyz006",

      Ee::NotWritable => "NotWritable",
      Ee::NotExecutable => "NotExecutable",
      Ee::TokenProgram => "TokenProgram",
      Ee::AtokenGPvbd => "AtokenGPvbd",
      Ee::SystemProgram => "SystemProgram",
      Ee::RentSysvar => "RentSysvar",
      Ee::Xyz013 => "Xyz013",
      Ee::Xyz014 => "Xyz014",

      Ee::ZeroU128 => "ZeroU128",
      Ee::ZeroU64 => "ZeroU64",
      Ee::ZeroU32 => "ZeroU32",
      Ee::ZeroU16 => "ZeroU16",
      Ee::ZeroU8 => "ZeroU8",
      Ee::ByteSizeForU128 => "ByteSizeForU128",
      Ee::ByteSizeForU64 => "ByteSizeForU64",
      Ee::ByteSizeForU32 => "ByteSizeForU32",
      Ee::ByteSizeForU16 => "ByteSizeForU16",
      Ee::ByteSizeForU8 => "ByteSizeForU8",
      Ee::ByteSliceSize128 => "ByteSliceSize128",
      Ee::ByteSliceSize64 => "ByteSliceSize64",
      Ee::ByteSliceSize32 => "ByteSliceSize32",
      Ee::ByteSliceSize10 => "ByteSliceSize10",
      Ee::ByteSliceSize6 => "ByteSliceSize6",

      Ee::InputDataLen => "InputDataLen",
      Ee::InputDataBump => "InputDataBump",
      Ee::PdaNoLamport => "PdaNoLamport",
      Ee::ForeignPDA => "ForeignPDA",
      Ee::VaultDataLengh => "VaultDataLengh",
      Ee::VaultIsForeign => "VaultIsForeign",
      Ee::DecimalsValue => "DecimalsValue",
      Ee::MintDataLen => "MintDataLen",
      Ee::MintOrMintAuthority => "MintOrMintAuthority",
      Ee::MintOrTokenProgram => "MintOrTokenProgram",

      Ee::AtaDataLen => "AtaDataLen",
      Ee::Ata22DataLen => "Ata22DataLen",
      Ee::AtaOrOwner => "AtaOrOwner",
      Ee::AtaOrMint => "AtaOrMint",
      Ee::AtaCheckX1 => "AtaCheckX1",
      Ee::ForeignAta => "ForeignAta",
      Ee::AtaHasNoData => "AtaHasNoData",
      Ee::TokenAcctOwner => "TokenAcctOwner",
      Ee::TokenAcctDataLen => "TokenAcctDataLen",
      Ee::Xyz049 => "Xyz049",

      Ee::NoRentExemptTokAcct => "NoRentExemptTokAcct",
      Ee::NoRentExemptTokAcct22 => "NoRentExemptTokAcct22",
      Ee::NoRentExemptMint22 => "NoRentExemptMint22",
      Ee::NoRentExemptMint => "NoRentExemptMint",
      Ee::Xyz054 => "Xyz054",
      Ee::Xyz055 => "Xyz055",
      Ee::Xyz056 => "Xyz056",
      Ee::Xyz057 => "Xyz057",
      Ee::Xyz058 => "Xyz058",
      Ee::Xyz059 => "Xyz059",

      Ee::AddToOverflow => "AddToOverflow",
      Ee::MultDivNone => "MultDivNone",
      Ee::MultiplyOverflow => "MultiplyOverflow",
      Ee::DividedByZero => "DividedByZero",
      Ee::Remainder => "Remainder",
      //Misc...
      Ee::EmptyData => "EmptyData",
      Ee::ClockGet => "ClockGet",
      Ee::VaultExists => "VaultExists",
      Ee::Xyz068 => "Xyz068",
      Ee::Xyz069 => "Xyz069",

      //Flashloan
      Ee::TokenAcctsLength => "TokenAcctsLength",
      Ee::LoanRecordAcctHasData => "LoanRecordAcctHasData",
      Ee::DataArgLenForU64 => "DataArgLenForU64",
      Ee::AmountsLenVsTokenAcctLen => "AmountsLenVsTokenAcctLen",
      Ee::BorrowAmountTooBig => "BorrowAmountTooBig",
      Ee::BorrowedAmountIsZero => "BorrowedAmountIsZero",
      Ee::LenderPdaBalanceIsZero => "LenderPdaBalanceIsZero",
      Ee::NumOfInstructions => "NumOfInstructions",
      Ee::RepayProgId => "RepayProgId",
      Ee::RepayDiscriminator => "RepayDiscriminator",
      Ee::RepayIxLenderPda => "RepayIxLenderPda",
      Ee::RepayTokenAccountLen => "RepayTokenAccountLen",
      //Final Variant
      Ee::NotMapped => "NotMapped",
    }
  }
}

//----------------==
pub fn check_data_len(data: &[u8], expected: usize) -> ProgramResult {
  if data.len() != expected {
    return Ee::InputDataLen.e();
  }
  Ok(())
}

//----------------== Account Verification
pub fn check_signer(account: &AccountView) -> ProgramResult {
  if !account.is_signer() {
    return Err(ProgramError::MissingRequiredSignature);
  }
  Ok(())
}
//----------------== TokenAccount & ATA
pub fn amount_from_token_acct(account: &AccountView) -> Result<u64, ProgramError> {
  if !account.owned_by(&pinocchio_token::ID) {
    return Err(Ee::TokenAcctOwner.into());
  }
  if account
    .data_len()
    .ne(&pinocchio_token::state::TokenAccount::LEN)
  {
    return Err(Ee::TokenAcctDataLen.into());
  }
  let data = account.try_borrow()?;
  /*Token Account Data has the struct:
  https://solana.com/docs/tokens
  pub struct Account {
      pub mint: Pubkey,// 32 bytes
      pub owner: Pubkey,// 32 bytes
      pub amount: u64, // 8 bytes
      ...
  }*/
  let balance = parse_u64(&data[64..72])?;
  Ok(balance)
}

pub fn ata_balc(from_ata: &AccountView, amount: u64) -> ProgramResult {
  let from_ata_info = TokenAccount::from_account_view(from_ata)?;
  if from_ata_info.amount() < amount {
    return Err(ProgramError::InsufficientFunds);
  }
  Ok(())
}

//----------------== PDAs and Other Accounts
pub fn check_ata(ata: &AccountView, owner: &AccountView, mint: &AccountView) -> ProgramResult {
  let ata_len = ata.data_len();
  if ata_len == 0 {
    return Ee::AtaHasNoData.e();
  }
  if ata_len.ne(&pinocchio_token::state::TokenAccount::LEN) {
    return Ee::AtaDataLen.e();
  }
  let ata_info = pinocchio_token::state::TokenAccount::from_account_view(ata)?;
  if ata_info.owner().ne(owner.address()) {
    return Ee::AtaOrOwner.e();
  }
  if ata_info.mint().ne(mint.address()) {
    return Ee::AtaOrMint.e();
  }
  Ok(())
}
pub fn rent_exempt_tokacct(account: &AccountView, rent_sysvar: &AccountView) -> ProgramResult {
  let rent = Rent::from_account_view(rent_sysvar)?;
  if !rent.is_exempt(account.lamports(), TokenAccount::LEN) {
    return Ee::NoRentExemptTokAcct.e();
  }
  Ok(())
}
//--------------==
pub fn check_sysprog(account: &AccountView) -> ProgramResult {
  if account.address().ne(&pinocchio_system::ID) {
    return Ee::SystemProgram.e();
  }
  Ok(())
}
pub const ATOKENGPVBD: Address = pinocchio_associated_token_account::ID;
pub fn check_atoken_gpvbd(account: &AccountView) -> ProgramResult {
  if account.address().ne(&ATOKENGPVBD) {
    return Ee::AtokenGPvbd.e();
  }
  Ok(())
}
pub fn check_rent_sysvar(account: &AccountView) -> ProgramResult {
  if account.address().ne(&RENT_ID) {
    return Ee::RentSysvar.e();
  }
  Ok(())
}

pub fn get_rent_exempt(
  account: &AccountView,
  rent_sysvar: &AccountView,
  data_len: usize,
) -> Result<u64, ProgramError> {
  if account.lamports() == 0 {
    return Err(ProgramError::UninitializedAccount);
  }
  let rent = Rent::from_account_view(rent_sysvar)?;
  let min_lam = rent.try_minimum_balance(data_len)?;
  log!("rent_exempt: {}", min_lam);
  Ok(min_lam)
}
//TODO: Mint and ATA from TokenLgc works. For mint and ATA from Token2022?
/// acc_type: 0 Mint, 1 TokenAccount
pub fn rent_exempt_mint(account: &AccountView, rent_sysvar: &AccountView) -> ProgramResult {
  let rent = Rent::from_account_view(rent_sysvar)?;
  if !rent.is_exempt(account.lamports(), Mint::LEN) {
    return Ee::NoRentExemptMint.e();
  }
  Ok(())
}
pub fn check_mint0a(mint: &AccountView, token_program: &AccountView) -> ProgramResult {
  //if !mint.owned_by(mint_authority)
  if mint.data_len() != pinocchio_token::state::Mint::LEN {
    return Ee::MintDataLen.e();
  }
  if token_program.address().ne(&pinocchio_token::ID) {
    return Ee::TokenProgram.e();
  }
  unsafe {
    if mint.owner().ne(&pinocchio_token::ID) {
      return Ee::MintOrTokenProgram.e();
    }
  }
  Ok(())
}

//pub const SYSTEMPROGRAM: pinocchio_pubkey::reexport::Pubkey = solana_system_interface::program::ID;

pub fn close_pda(pda: &AccountView, dest: &AccountView) -> ProgramResult {
  log!("Close pda 1");
  {
    let mut data = pda.try_borrow_mut()?;
    data[0] = 0xff;
  }
  log!("Close pda 2");
  let sum_lam = dest
    .lamports()
    .checked_add(pda.lamports())
    .ok_or_else(|| ProgramError::ArithmeticOverflow)?;
  dest.set_lamports(sum_lam);
  pda.set_lamports(0);

  pda.resize(1)?;
  pda.close()?;
  Ok(())
}

//----------------== Check Account Properties
pub fn writable(account: &AccountView) -> ProgramResult {
  if !account.is_writable() {
    return Ee::NotWritable.e();
  }
  Ok(())
}
pub fn executable(account: &AccountView) -> ProgramResult {
  if !account.executable() {
    return Ee::NotExecutable.e();
  }
  Ok(())
}

pub fn not_initialized(account: &AccountView) -> ProgramResult {
  if account.lamports() > 0 {
    return Err(ProgramError::AccountAlreadyInitialized);
  }
  Ok(())
}
pub fn initialized(account: &AccountView) -> ProgramResult {
  if account.lamports() == 0 {
    return Err(ProgramError::UninitializedAccount);
  }
  Ok(())
}
pub fn empty_data(account: &AccountView) -> ProgramResult {
  if account.data_len() == 0 {
    return Ok(());
  }
  Ee::EmptyData.e()
}

pub fn none_zero_u64(uint: u64) -> ProgramResult {
  if uint == 0u64 {
    return Ee::ZeroU64.e();
  }
  Ok(())
}
pub fn none_zero_u8(uint: u8) -> ProgramResult {
  if uint == 0u8 {
    return Ee::ZeroU8.e();
  }
  Ok(())
}
//----------------== Check Input Values

pub fn check_decimals(mint: &AccountView, decimals: u8) -> ProgramResult {
  let mint_info = pinocchio_token::state::Mint::from_account_view(mint)?;
  if decimals != mint_info.decimals() {
    return Ee::DecimalsValue.e();
  }
  Ok(())
}

//----------------== Parse Functions
/// Parse a u64 from u8 array
pub fn parse_u64(data: &[u8]) -> Result<u64, ProgramError> {
  let bytes: [u8; 8] = data.try_into().or_else(|_e| Err(Ee::ByteSizeForU64))?;

  let amt = u64::from_le_bytes(bytes);
  // let amount = u64::from_le_bytes([data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7]]);
  Ok(amt)
}
pub fn parse_u16(data: &[u8]) -> Result<u16, ProgramError> {
  let bytes: [u8; 2] = data.try_into().or_else(|_e| Err(Ee::ByteSizeForU16))?;

  let amt = u16::from_le_bytes(bytes);
  // let amount = u64::from_le_bytes([data[0], data[1], data[2], data[3]]);
  Ok(amt)
}
