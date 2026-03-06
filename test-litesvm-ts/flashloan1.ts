/** biome-ignore-all lint/style/noNonNullAssertion: <> */
import { expect, test } from "bun:test";
//Tutorial: <https://litesvm.github.io/litesvm/tutorial.html>
import type { Keypair, PublicKey } from "@solana/web3.js";
import {
	acctExists,
	acctIsNull,
	ataArrayBalCk,
	ataArrayBalc,
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
	tokLgcDepositArgs,
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
let _toAta: PublicKey;
let _toAtaAta: PublicKey;
let _tokenProgram: PublicKey;
let _userAta: PublicKey;
let rawAcctData: Uint8Array<ArrayBufferLike>;
let vaultBump: number;
let decimals: number;
let fee: number;
let fees: number[];
let _amt: bigint;
let amounts: bigint[];
let debts: bigint[];
let balcs: bigint[];

let _balcBf: bigint;
let _balcAf: bigint;
const initBalc = bigintAmt(9000000, 6);
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

	setAtaCheck(usdcMint, admin, initBalc, "Admin USDC");
	setAtaCheck(usdcMint, user1, initBalc, "User1 USDC");
	setAtaCheck(usdcMint, user2, initBalc, "User2 USDC");
	setAtaCheck(usdcMint, user3, initBalc, "User3 USDC");
	setAtaCheck(usdcMint, hacker, initBalc, "Hacker USDC");
});
//jj tts 1
test("Init Vault", () => {
	ll("\n----------== Init Vault");
	signerKp = user1Kp;
	fees = [500, 700]; //u16, to be divided by 10_000
	vaultOut = findVaultV1("Vault", fee);
	vault = vaultOut.pda;
	vaultBump = vaultOut.bump;
	vaultInit(signerKp, vault, fee, vaultBump);
	acctExists(vault);
	rawAcctData = getRawAcctData(vault);
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
	signer = signerKp.publicKey;

	amounts = [as6zBn(100000), as6zBn(700000)];
	fees = [500, 700]; //u16, to be divided by 10_000
	debts = [0n, 0n];
	const { txnAccts, userAta } = tokLgcDepositArgs(amounts, fees, mint, signer);
	balcs = [0n, 0n]; //to replace null balcs

	tokLgcDeposit(signerKp, userAta, mint, decimals, txnAccts, amounts);

	ataArrayBalCk(txnAccts, balcs, amounts, debts, decimals, 2);
	//ataBalCk(toAta, amt, "vault1");
	//ataBalCk(userAta, initBalc - amt, "admin ");
});
test.skip("Flashloan", () => {
	ll("\n----------== Flashloan");
	signerKp = user1Kp;
	mint = usdcMint;
	decimals = 6;
	//TODO: Make tokLgcDepositBatch to deposit to many vaults
	amounts = [1000n, 2000n];
	fees = [500, 700]; //u16, to be divided by 10_000
	const {
		repayAmts,
		vaultBumps,
		txnAccts,
		loansPdaOut,
		amountsLen,
		rapayAmtsSum,
	} = flashloanArgs(amounts, fees, mint, signerKp.publicKey);
	balcs = ataArrayBalc(txnAccts, amountsLen, decimals, 3);

	flashloan(
		signerKp,
		loansPdaOut.pda,
		mint,
		decimals,
		loansPdaOut.bump,
		vaultBumps,
		txnAccts,
		fees,
		amounts,
		rapayAmtsSum,
	);
	ataArrayBalCk(txnAccts, balcs, repayAmts, amounts, decimals, 3);
});
