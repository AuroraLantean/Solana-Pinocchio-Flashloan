/** biome-ignore-all lint/style/noNonNullAssertion: <> */
import { expect, test } from "bun:test";
//Tutorial: <https://litesvm.github.io/litesvm/tutorial.html>
import type { Keypair, PublicKey } from "@solana/web3.js";
import {
	acctExists,
	acctIsNull,
	ataBalCk,
	atasBalc,
	findVaultV1,
	flashloan,
	flashloanArgs,
	getAta,
	getRawAcctData,
	initSolBalc,
	type PdaOut,
	setAtaCheck,
	setMint,
	svm,
	tokLgcDeposit,
	vaultAtaInit,
	vaultInit,
	//vault1,
	//vaultAta1,
	//vaultO,
} from "./litesvm-utils";
import { as6zBn, bigintAmt, ll } from "./utils";
import {
	admin,
	adminKp,
	hacker,
	usdcMint,
	user1,
	user1Kp,
	user2,
	user3,
} from "./web3jsSetup";

let signerKp: Keypair;
let signer: PublicKey;
let mint: PublicKey;
let vaultOut: PdaOut;
let vault: PublicKey;
let vaultAta: PublicKey;
let toAta: PublicKey;
let fromAta: PublicKey;
let _tokenProgram: PublicKey;
let _userAta: PublicKey;
let vaultBump: number;
let decimals: number;
let fee: number;
let fees: number[];
let amt: bigint;
let debts: bigint[];
let balcs: bigint[];

let _balcBf: bigint;
let _balcAf: bigint;
const initUsdcBalc = bigintAmt(10000, 6);
//co_balcAfultRent = 1002240n; //from Rust

const balcAdmin = svm.getBalance(admin);
ll("admin SOL:", balcAdmin);
expect(balcAdmin).toStrictEqual(initSolBalc);

test("initial conditions", () => {
	//acctIsNull(vaultAta1);
});

//------------------==
test("Set USDC Mint and ATAs", () => {
	ll("\n------== Set USDC Mint and ATAs");
	setMint(usdcMint);
	acctExists(usdcMint);

	setAtaCheck(usdcMint, admin, initUsdcBalc, "Admin USDC");
	setAtaCheck(usdcMint, user1, initUsdcBalc, "User1 USDC");
	setAtaCheck(usdcMint, user2, initUsdcBalc, "User2 USDC");
	setAtaCheck(usdcMint, user3, initUsdcBalc, "User3 USDC");
	setAtaCheck(usdcMint, hacker, initUsdcBalc, "Hacker USDC");
});
//jj tts 1
test("Init Vault", () => {
	ll("\n----------== Init Vault");
	signerKp = user1Kp;
	fee = 500;
	vaultOut = findVaultV1("Vault", fee);
	vault = vaultOut.pda;
	vaultBump = vaultOut.bump;

	acctIsNull(vault);
	vaultInit(signerKp, vault, fee, vaultBump);
	acctExists(vault);
	const rawAcctData = getRawAcctData(vault);
	expect(rawAcctData[0]).toEqual(vaultBump);
});
test.skip("Init Vault ATA", () => {
	ll("\n---------== Init Vault ATA");
	signerKp = user1Kp;
	fee = 500;
	vaultOut = findVaultV1("Vault", fee);
	vault = vaultOut.pda;
	mint = usdcMint;
	vaultAta = getAta(mint, vault);

	acctIsNull(vaultAta);
	vaultAtaInit(signerKp, vault, vaultAta, mint, fee);
	acctExists(vaultAta);
});
test("Deposit Legacy Tokens", () => {
	ll("\n----------== Deposit Legacy Tokens");
	signerKp = adminKp;
	mint = usdcMint;
	decimals = 6;
	amt = as6zBn(3700);

	signer = signerKp.publicKey;
	fromAta = getAta(mint, signer);
	fee = 500;
	vaultOut = findVaultV1("Vault", fee);
	toAta = getAta(mint, vaultOut.pda);

	tokLgcDeposit(
		signerKp,
		fromAta,
		toAta,
		vaultOut.pda, //vault as to_wallet
		mint,
		//configPDA,
		decimals,
		amt,
	);
	ataBalCk(toAta, as6zBn(3700), "vault1");
	ataBalCk(fromAta, as6zBn(6300), "admin ");
});
test("Flashloan", () => {
	ll("\n----------== Flashloan");
	signerKp = user1Kp;
	mint = usdcMint;
	decimals = 6;

	debts = [100n];
	fees = [500]; //u16, to be divided by 10_000
	const { repayAmts, vaultBumps, txnAccts, loansPdaOut } = flashloanArgs(
		debts,
		fees,
		mint,
		signerKp.publicKey,
	);
	balcs = atasBalc(txnAccts, debts.length);
	const balcBf = balcs[0];
	flashloan(
		signerKp,
		//vaults[0]!,
		loansPdaOut.pda,
		mint,
		txnAccts,
		decimals,
		loansPdaOut.bump,
		vaultBumps[0]!,
		fees[0]!,
		debts,
		repayAmts[0]!,
	);
	ataBalCk(txnAccts[1]!, balcBf + repayAmts[0]! - BigInt(debts[0]!), "vault");
	//ataBalCk(fromAta, as6zBn(424), "user1 ");
});
