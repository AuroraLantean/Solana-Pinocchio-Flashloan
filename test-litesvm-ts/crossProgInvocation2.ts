/** biome-ignore-all lint/style/noNonNullAssertion: <> */

import { expect, test } from "bun:test";
//Tutorial: <https://litesvm.github.io/litesvm/tutorial.html>
import type { Keypair, PublicKey } from "@solana/web3.js";
import {
	acctExists,
	checkVaultData,
	funcCaller,
	initSolBalc,
	type PdaOut,
	setAtaCheck,
	setMint,
	svm,
	vaultInitArgs,
	//vault1,
	//vaultAta1,
	//vaultO,
} from "./litesvm-utils";
import { bigintAmt, ll } from "./utils";
import {
	admin,
	adminKp,
	flashloanProgAddr,
	hacker,
	usdcMint,
	user1,
	user2,
	user3,
} from "./web3jsSetup";

let signerKp: Keypair;
let _signer: PublicKey;
let _signerut: PdaOut;
let _vaultAta: PublicKey;
let _toAtaAta: PublicKey;
let _tokenProgram: PublicKey;
let _userAta: PublicKey;
let _arrLen: number;
let _feeunts: bigint[];
let _debts: bigint[];
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
//change declare_id!() to 8ZEf7
// jj build2;
//change declare_id!() back to FcLwq
// jjb ; jj tts 2
const fees = [500, 700]; //u16, to be divided by 10_000
const _mint = usdcMint;
const _mintmals = 6;
test("Init Vault", () => {
	ll("\n----------== Init Vault");
	signerKp = adminKp;
	const { vaultBumps, vaults } = vaultInitArgs(fees);
	funcCaller(signerKp, flashloanProgAddr, vaults, fees, vaultBumps);
	ll("signer:", signerKp.publicKey.toBase58());
	checkVaultData(vaults, fees, vaultBumps);
});
//TODO: Pinocchio calls Pinocchio with seeds
//TODO: Pinocchio calls Anchor program via signer
//TODO: Pinocchio calls Anchor program via seeds
//TODO: in Rust backend: init account with Pkey, Transfer SOL/Tokens, call programs
