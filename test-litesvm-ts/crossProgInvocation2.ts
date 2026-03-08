/** biome-ignore-all lint/style/noNonNullAssertion: <> */

import { expect, test } from "bun:test";
//Tutorial: <https://litesvm.github.io/litesvm/tutorial.html>
import type { Keypair, PublicKey } from "@solana/web3.js";

import {
	acctExists,
	checkVaultData,
	initAnchorPdaCaller,
	initSolBalc,
	type PdaOut,
	setAtaCheck,
	setMint,
	svm,
	vaultInitArgs,
	vaultInitCaller,
	//vault1,
	//vaultAta1,
	//vaultO,
} from "./litesvm-utils";
import { bigintAmt, ll } from "./utils";
import {
	admin,
	adminKp,
	flashloanProgAddr,
	futureOptionAddr,
	futureOptionAnchorPda,
	futureOptionAnchorPdaBump,
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
	const targetProgDisc = [0]; //vaultInit
	const { vaultBumps, vaults } = vaultInitArgs(fees);
	vaultInitCaller(
		signerKp,
		flashloanProgAddr,
		vaults,
		targetProgDisc,
		fees,
		vaultBumps,
	);
	ll("signer:", signerKp.publicKey.toBase58());
	checkVaultData(vaults, fees, vaultBumps);
});

test("Init AnchorPda", () => {
	ll("\n----------== Init AnchorPda");
	signerKp = adminKp;
	const targetProgDisc = [200, 48, 123, 186, 217, 204, 239, 70]; //copied from Anchor IDL
	const pdaAddr = futureOptionAnchorPda;
	const bump = futureOptionAnchorPdaBump;
	const tokenBalc = 73200n;
	initAnchorPdaCaller(
		signerKp,
		futureOptionAddr,
		targetProgDisc,
		pdaAddr,
		tokenBalc,
		bump,
	);
	ll("signer:", signerKp.publicKey.toBase58());
	//checkVaultData(vaults, fees, vaultBumps);
});
//TODO: Pinocchio calls Pinocchio with seeds
//TODO: Pinocchio calls Anchor program via seeds
//TODO: in Rust backend: init account with Pkey, Transfer SOL/Tokens, call programs
