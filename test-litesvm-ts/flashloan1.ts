/** biome-ignore-all lint/style/noNonNullAssertion: <> */
import { expect, test } from "bun:test";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
//Tutorial: <https://litesvm.github.io/litesvm/tutorial.html>
import type { Keypair, PublicKey } from "@solana/web3.js";
import {
	acctExists,
	acctIsNull,
	findLoanRecordsV1,
	findVaultV1,
	flashloan,
	getAta,
	getRawAcctData,
	initSolBalc,
	type PdaOut,
	setAtaCheck,
	setMint,
	svm,
	vaultInit,
	vaultTokAcctInit,
	//vault1,
	//vaultAta1,
	//vaultO,
} from "./litesvm-utils";
import { bigintAmt, ll } from "./utils";
import {
	admin,
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
let loanRecordsOut: PdaOut;
let vaultOut: PdaOut;
let vault: PublicKey;
let vaultTokAcct: PublicKey;
let tokenProgram: PublicKey;
let userAta: PublicKey;
let tokenAccts: PublicKey[];
let vaultBump: number;
let decimals: number;
let bump: number;
let fee: number;
let amounts: bigint[];

let balcBf: bigint | null;
let _balcAf: bigint | null;
const initUsdcBalc = bigintAmt(1000, 6);
//co_balcAfultRent = 1002240n; //from Rust

balcBf = svm.getBalance(admin);
ll("admin SOL:", balcBf);
expect(balcBf).toStrictEqual(initSolBalc);

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
	ll("\n------== Init Vault");
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
test("Init Vault ATA", () => {
	ll("\n------== Init Vault ATA");
	signerKp = user1Kp;
	fee = 500;
	vaultOut = findVaultV1("Vault", fee);
	vault = vaultOut.pda;
	mint = usdcMint;
	vaultTokAcct = getAta(mint, vault);
	//decimals = 6;

	acctIsNull(vaultTokAcct);
	vaultTokAcctInit(signerKp, vault, vaultTokAcct, mint, fee);
	acctExists(vaultTokAcct);
});
test.skip("Flashloan", () => {
	ll("\n------== Flashloan");
	signerKp = user1Kp;
	vault = user1;
	loanRecordsOut = findLoanRecordsV1(fee, "moon_pool");
	mint = usdcMint;
	tokenProgram = TOKEN_PROGRAM_ID;
	decimals = 6;
	bump = 255;
	fee = 500;
	amounts = [100n, 100n];
	userAta = getAta(mint, signer);
	tokenAccts = [vaultTokAcct, userAta];

	signer = signerKp.publicKey;

	flashloan(
		signerKp,
		vault,
		loanRecordsOut.pda,
		mint,
		tokenProgram,
		tokenAccts,
		decimals,
		bump,
		fee,
		amounts,
	);
	//ataBalCk(toAta, amt, "vaultO");
	//ataBalCk(fromAta, as6zBn(424), "user1 ");
});
