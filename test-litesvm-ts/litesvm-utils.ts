import { expect } from "bun:test";
import { getAddressDecoder } from "@solana/kit";
import {
	ACCOUNT_SIZE,
	AccountLayout,
	ASSOCIATED_TOKEN_PROGRAM_ID,
	getAssociatedTokenAddressSync,
	MINT_SIZE,
	MintLayout,
	TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import {
	type Keypair,
	LAMPORTS_PER_SOL,
	PublicKey,
	SYSVAR_INSTRUCTIONS_PUBKEY,
	SystemProgram,
	Transaction,
	TransactionInstruction,
} from "@solana/web3.js";
import {
	ComputeBudget,
	type FailedTransactionMetadata,
	LiteSVM,
	type SimulatedTransactionInfo,
	TransactionMetadata,
} from "litesvm";
import {
	bigIntSum,
	bytesToBigint,
	checkBump,
	checkDecimals,
	checkFee,
	checkTxnAccts,
	makeDepositIxKeys,
	makeFlashloanIxKeys,
	makeVaultInitIxKeys,
	numToBytes,
	zero,
} from "./utils";
import {
	ATokenGPvbd,
	admin,
	flashloanProgAddr,
	funcCallerProgAddr,
	hacker,
	owner,
	RentSysvar,
	SYSTEM_PROGRAM,
	user1,
	user2,
	user3,
} from "./web3jsSetup";

const ll = console.log;
ll("\n------== litesvm-utils");
export let svm = new LiteSVM();
export const initSolBalc = BigInt(LAMPORTS_PER_SOL) * BigInt(10);
ll("initialize accounts by airdropping SOLs");
svm.airdrop(owner, initSolBalc);
svm.airdrop(admin, initSolBalc);
svm.airdrop(user1, initSolBalc);
svm.airdrop(user2, initSolBalc);
svm.airdrop(user3, initSolBalc);
svm.airdrop(hacker, initSolBalc);

export type PdaOut = {
	pda: PublicKey;
	bump: number;
};
export const findVaultV1 = (
	pdaName: string,
	fee: number,
	seedStr = "vault",
	progAddr = flashloanProgAddr,
): PdaOut => {
	const [pda, bump] = PublicKey.findProgramAddressSync(
		[Buffer.from(seedStr), Buffer.from(numToBytes(fee, 16))], //Buffer.copyBytesFrom(numToBytes(idBigInt)),
		progAddr,
	); // addr.toBuffer()
	ll(`${pdaName} pda: ${pda.toBase58()}, bump: ${bump}`);
	return { pda, bump };
};

export const findLoansPdaV1 = (user: PublicKey): PdaOut => {
	const [pda, bump] = PublicKey.findProgramAddressSync(
		[Buffer.from("loans"), user.toBuffer()],
		flashloanProgAddr,
	);
	ll(`Loans pda: ${pda.toBase58()}, bump: ${bump}`);
	return { pda, bump };
};
//-------------== Program Methods
export const vaultInitArgs = (feesX100: number[]) => {
	ll("------== vaultInitArgs");
	const amountsLen = feesX100.length;
	let vaultOut: PdaOut;
	const vaults: PublicKey[] = [];
	const vaultBumps: number[] = [];
	for (const [idx, fee] of feesX100.entries()) {
		ll("idx:", idx);
		vaultOut = findVaultV1("Vault", fee);
		vaultBumps.push(vaultOut.bump);
		vaults.push(vaultOut.pda);
	}
	ll("vaultInitArgs successful");
	return {
		vaultBumps,
		vaults,
		amountsLen,
	};
};
export const vaultInit = (
	userSigner: Keypair,
	//configPda: PublicKey,
	vaults: PublicKey[],
	fees: number[],
	vaultBumps: number[],
) => {
	const disc = 0;
	const { ixKeyArray, feesU8 } = makeVaultInitIxKeys(vaults, fees, vaultBumps);
	const argData = [...vaultBumps, ...feesU8];

	const blockhash = svm.latestBlockhash();
	const ix = new TransactionInstruction({
		keys: [
			{ pubkey: userSigner.publicKey, isSigner: true, isWritable: true },
			{ pubkey: SYSTEM_PROGRAM, isSigner: false, isWritable: false },
			{ pubkey: RentSysvar, isSigner: false, isWritable: false },
			...ixKeyArray,
		],
		programId: flashloanProgAddr,
		data: Buffer.from([disc, ...argData]),
	});
	sendTxns(svm, blockhash, [ix], [userSigner]);
};
export const checkVaultData = (
	vaults: PublicKey[],
	fees: number[],
	vaultBumps: number[],
) => {
	ll("------== checkVaultData");
	const vaultsLen = vaults.length;
	if (vaultsLen > 8) throw new Error("vaults length should be <= 8");
	if (vaultsLen !== vaultBumps.length)
		throw new Error("vaults length != vaultBumps length");

	const addressDecoder = getAddressDecoder();
	ll("loop over vaultsLen index...");
	let tokenBalc: bigint;
	let fee: number | undefined;
	let bump: number | undefined;
	let addrStr = "";
	let addrBytes: Uint8Array<ArrayBufferLike>;
	let rawAcctData: Uint8Array<ArrayBufferLike>;
	for (const [i, vault] of vaults.entries()) {
		fee = fees[i];
		bump = vaultBumps[i];
		if (bump === undefined || fee === undefined)
			throw new Error(`bump ${bump} invalid`);
		acctExists(vault);
		rawAcctData = getRawAcctData(vault);
		addrBytes = rawAcctData.slice(0, 32);
		addrStr = addressDecoder.decode(addrBytes);
		ll("addrStr:", addrStr);
		tokenBalc = bytesToBigint(rawAcctData.slice(32, 40));
		ll("tokenBalc:", tokenBalc, ", bump:", bump);
		expect(tokenBalc).toEqual(BigInt(fee));
		expect(rawAcctData[40]).toEqual(bump);
	}
	ll("checkVaultData successful");
};
export const vaultInitCaller = (
	userSigner: Keypair,
	target_prog: PublicKey,
	//configPda: PublicKey,
	vaults: PublicKey[],
	disc0: Uint8Array<ArrayBuffer>,
	disc1: Uint8Array<ArrayBuffer>,
	fees: number[],
	vaultBumps: number[],
) => {
	const { ixKeyArray, feesU8 } = makeVaultInitIxKeys(vaults, fees, vaultBumps);
	const ixData = [...disc1, ...vaultBumps, ...feesU8];
	//const ix_data_size = argData1.length;

	const keys = [
		{ pubkey: userSigner.publicKey, isSigner: true, isWritable: true },
		{ pubkey: target_prog, isSigner: false, isWritable: false },
		{ pubkey: SYSTEM_PROGRAM, isSigner: false, isWritable: false },
		{ pubkey: RentSysvar, isSigner: false, isWritable: false },
		...ixKeyArray,
	];
	const argData = [keys.length - 1, ...ixData]; // -1 to exclude target_prog

	const blockhash = svm.latestBlockhash();
	const ix = new TransactionInstruction({
		keys,
		programId: funcCallerProgAddr,
		data: Buffer.from([...disc0, ...argData]),
	});
	sendTxns(svm, blockhash, [ix], [userSigner], "", funcCallerProgAddr);
};
export const vaultAtaInit = (
	userSigner: Keypair,
	vaultPda: PublicKey,
	vaultTokAcct: PublicKey,
	mint: PublicKey,
	//configPda: PublicKey,
	//decimals: number,
	//vaultTokAcctBump: number,
	fee: number,
	tokenProg = TOKEN_PROGRAM_ID,
	atokenProg = ATokenGPvbd,
) => {
	const disc = 1;
	//checkDecimals(decimals);
	const argData = [...numToBytes(fee, 16)];
	const blockhash = svm.latestBlockhash();
	const ix = new TransactionInstruction({
		keys: [
			{ pubkey: userSigner.publicKey, isSigner: true, isWritable: true },
			{ pubkey: vaultPda, isSigner: false, isWritable: true }, // true
			{ pubkey: vaultTokAcct, isSigner: false, isWritable: true },
			{ pubkey: mint, isSigner: false, isWritable: false },
			//{ pubkey: configPda, isSigner: false, isWritable: true },
			{ pubkey: tokenProg, isSigner: false, isWritable: false },
			{ pubkey: SYSTEM_PROGRAM, isSigner: false, isWritable: false },
			{ pubkey: atokenProg, isSigner: false, isWritable: false },
			{ pubkey: RentSysvar, isSigner: false, isWritable: false },
		],
		programId: flashloanProgAddr,
		data: Buffer.from([disc, ...argData]),
	});
	sendTxns(svm, blockhash, [ix], [userSigner]);
};

export const tokLgcDepositArgs = (
	depositAmtsLen: number,
	feesX100: number[],
	mint: PublicKey,
	signer: PublicKey,
) => {
	ll("------== tokLgcDepositArgs");
	if (depositAmtsLen !== feesX100.length)
		throw new Error("depositAmts length should be the same as feesX100");
	const userAta = getAta(mint, signer);
	acctExists(userAta);

	let vaultOut: PdaOut;
	const depAccts: PublicKey[] = [];
	const vaultBumps: number[] = [];
	for (const [idx, fee] of feesX100.entries()) {
		ll("idx:", idx);
		if (fee === undefined) throw new Error(`feesX100[${idx}] undefined`);
		checkFee(fee);
		vaultOut = findVaultV1("Vault", fee);
		acctExists(vaultOut.pda);

		checkBump(vaultOut.bump);
		vaultBumps.push(vaultOut.bump);
		depAccts.push(vaultOut.pda);
		depAccts.push(getAta(mint, vaultOut.pda));
	}
	ll("tokLgcDepositArgs successful");
	return {
		vaultBumps,
		depAccts,
		userAta,
	};
};
export const tokLgcDepositIx = (
	userSigner: Keypair,
	fromAta: PublicKey,
	mint: PublicKey,
	//configPda: PublicKey,
	decimals: number,
	depositAccts: PublicKey[], //[vault, vaultAta,...]
	depositAmts: bigint[],
	tokenProg = TOKEN_PROGRAM_ID,
	atokenProg = ATokenGPvbd,
) => {
	const disc = 2;
	checkDecimals(decimals);
	const amountsSum = bigIntSum(depositAmts);

	const userTokBalc = ataBalc(fromAta, `userAta`, decimals, true);
	if (userTokBalc === 0n) throw new Error("userTokBalc is zero");
	if (amountsSum > userTokBalc) throw new Error("amount > userTokBalc");

	const { u64bytes, ixKeyArray } = makeDepositIxKeys(depositAccts, depositAmts);
	const argData = [decimals, ...u64bytes];
	const ix = new TransactionInstruction({
		keys: [
			{ pubkey: userSigner.publicKey, isSigner: true, isWritable: true },
			{ pubkey: fromAta, isSigner: false, isWritable: true },
			{ pubkey: mint, isSigner: false, isWritable: false },
			//{ pubkey: configPda, isSigner: false, isWritable: true },
			{ pubkey: tokenProg, isSigner: false, isWritable: false },
			{ pubkey: SYSTEM_PROGRAM, isSigner: false, isWritable: false },
			{ pubkey: atokenProg, isSigner: false, isWritable: false },
			{ pubkey: RentSysvar, isSigner: false, isWritable: false },
			...ixKeyArray,
		],
		programId: flashloanProgAddr,
		data: Buffer.from([disc, ...argData]),
	});
	return ix;
};
export const tokLgcDeposit = (
	userSigner: Keypair,
	fromAta: PublicKey,
	mint: PublicKey,
	//configPda: PublicKey,
	decimals: number,
	depositAccts: PublicKey[], //[vault, vaultAta,...]
	depositAmts: bigint[],
	tokenProg = TOKEN_PROGRAM_ID,
	atokenProg = ATokenGPvbd,
) => {
	ll("------== tokLgcDeposit");
	const ix = tokLgcDepositIx(
		userSigner,
		fromAta,
		mint,
		//configPda,
		decimals,
		depositAccts,
		depositAmts,
		tokenProg,
		atokenProg,
	);
	const blockhash = svm.latestBlockhash();
	sendTxns(svm, blockhash, [ix], [userSigner]);
};

export const flashloanArgs = (
	amounts: bigint[],
	feesX100: number[],
	mint: PublicKey,
	signer: PublicKey,
) => {
	ll("------== flashloanArgs");
	const amountsLen = amounts.length;
	if (amountsLen !== feesX100.length)
		throw new Error("amounts length should be the same as feesX100");
	const userAta = getAta(mint, signer);
	acctExists(userAta);
	const loansPdaOut = findLoansPdaV1(signer);
	acctIsNull(loansPdaOut.pda);

	let repayAmt = 0n;
	let vaultOut: PdaOut;
	const repayAmts: bigint[] = [];
	const txnAccts: PublicKey[] = [];
	const vaultBumps: number[] = [];
	for (const [idx, debt] of amounts.entries()) {
		ll("idx:", idx);
		const fee = feesX100[idx];
		if (fee === undefined) throw new Error(`feesX100[${idx}] undefined`);
		repayAmt = (debt * BigInt(fee)) / 10_000n + debt;
		ll("repayAmt:", repayAmt);
		repayAmts.push(repayAmt);
		vaultOut = findVaultV1("Vault", fee);
		ll("check if vault exists");
		acctExists(vaultOut.pda);

		vaultBumps.push(vaultOut.bump);
		txnAccts.push(vaultOut.pda);
		txnAccts.push(getAta(mint, vaultOut.pda));
		txnAccts.push(userAta);
	}
	ll("flashloanArgs successful");
	return {
		repayAmts,
		vaultBumps,
		txnAccts,
		loansPdaOut,
		amountsLen,
	};
};
export const flashloan = (
	userSigner: Keypair,
	loansPda: PublicKey,
	mint: PublicKey,
	//configPda: PublicKey,
	decimals: number,
	loansBump: number,
	vaultBumps: number[],
	txnAccts: PublicKey[], //[vaultAta, userAta, ...]
	fees: number[],
	amounts: bigint[],
	repayAmts: bigint[],
	tokenProg = TOKEN_PROGRAM_ID,
	atokenProg = ATokenGPvbd,
) => {
	ll("------== flashloan() to invoke Rust");
	const borrow_disc = 3;
	const repay_disc = 4;
	acctIsNull(loansPda);
	checkDecimals(decimals);
	for (const vaultBump of vaultBumps) {
		checkBump(vaultBump);
	}
	const feesU8: number[] = [];
	for (const fee of fees) {
		checkFee(fee, "fee");
		feesU8.push(...numToBytes(fee, 16));
	}

	const { u64bytes, ixKeyArray } = makeFlashloanIxKeys(
		txnAccts,
		amounts,
		decimals,
	);
	const argData = [decimals, loansBump, ...vaultBumps, ...feesU8, ...u64bytes];
	const blockhash = svm.latestBlockhash();

	//--------== Deposit
	//const rapayAmts = [as6zBn(100000), as6zBn(700000)];
	const arrLen = amounts.length;
	const { depAccts, userAta } = tokLgcDepositArgs(
		arrLen,
		fees,
		mint,
		userSigner.publicKey,
	);
	const ixDeposit = tokLgcDepositIx(
		userSigner,
		userAta,
		mint,
		//configPda,
		decimals,
		depAccts,
		repayAmts,
		tokenProg,
		atokenProg,
	);
	//--------== FlashloanBorrow
	const ix0 = new TransactionInstruction({
		keys: [
			{ pubkey: userSigner.publicKey, isSigner: true, isWritable: true },
			{ pubkey: loansPda, isSigner: false, isWritable: true },
			{ pubkey: mint, isSigner: false, isWritable: false },
			{ pubkey: tokenProg, isSigner: false, isWritable: false },
			{ pubkey: SYSTEM_PROGRAM, isSigner: false, isWritable: false },
			{ pubkey: RentSysvar, isSigner: false, isWritable: false },
			{
				pubkey: SYSVAR_INSTRUCTIONS_PUBKEY,
				isSigner: false,
				isWritable: false,
			},
			...ixKeyArray,
		],
		programId: flashloanProgAddr,
		data: Buffer.from([borrow_disc, ...argData]),
	});
	//--------== FlashloanRepay
	const ixLast = new TransactionInstruction({
		keys: [
			{ pubkey: userSigner.publicKey, isSigner: true, isWritable: true },
			{ pubkey: loansPda, isSigner: false, isWritable: true },
			...ixKeyArray,
		],
		programId: flashloanProgAddr,
		data: Buffer.from([repay_disc]),
	});
	sendTxns(
		svm,
		blockhash,
		[ix0, ixDeposit, ixLast],
		[userSigner],
		"",
		flashloanProgAddr,
	);
};
//-------------== LiteSVM System Methods
export const sendSol = (signer: Keypair, addrTo: PublicKey, amount: bigint) => {
	const blockhash = svm.latestBlockhash();
	const ixs = [
		SystemProgram.transfer({
			fromPubkey: signer.publicKey,
			toPubkey: addrTo,
			lamports: amount,
		}),
	];
	sendTxns(svm, blockhash, ixs, [signer], "", SYSTEM_PROGRAM);
};

export const makeAccount = (
	signer: Keypair,
	newAccount: PublicKey,
	programId: PublicKey,
) => {
	const blockhash = svm.latestBlockhash();
	const ixs = [
		SystemProgram.createAccount({
			fromPubkey: signer.publicKey,
			newAccountPubkey: newAccount,
			lamports: Number(svm.minimumBalanceForRentExemption(BigInt(4))),
			space: 4,
			programId: programId,
		}),
	];
	sendTxns(svm, blockhash, ixs, [signer], "", SYSTEM_PROGRAM);
};
//const rawAccount = svm.getAccount(address);

//When you want to make Mint without the Mint Keypair. E.g. UsdtMintKp;
//https://solana.com/docs/tokens/basics/create-mint
export const setMint = (
	mint: PublicKey,
	decimals = 6,
	supply = 9_000_000_000_000n,
	mintAuthority = owner,
	freezeAuthority = owner,
	programId = TOKEN_PROGRAM_ID,
) => {
	const rawMintAcctData = Buffer.alloc(MINT_SIZE);
	MintLayout.encode(
		{
			mintAuthorityOption: 1, //0,
			mintAuthority: mintAuthority, // PublicKey.default,
			supply: supply, // 0n
			decimals: decimals, //0
			isInitialized: true, //false,
			freezeAuthorityOption: 1, //0,
			freezeAuthority: freezeAuthority, // PublicKey.default,
		},
		rawMintAcctData,
	);
	svm.setAccount(mint, {
		lamports: 1_000_000_000,
		data: rawMintAcctData,
		owner: programId,
		executable: false,
	});
};

//-------------== USDC or USDT
export const acctIsNull = (account: PublicKey) => {
	const raw = svm.getAccount(account);
	if (raw !== null) throw new Error("account should be null");
};
export const acctExists = (account: PublicKey) => {
	const raw = svm.getAccount(account);
	if (raw === null) throw new Error("account should exist");
	//expect(raw).not.toBeNull();
};
export const getAta = (
	mint: PublicKey,
	owner: PublicKey,
	allowOwnerOffCurve = true,
	programId = TOKEN_PROGRAM_ID,
	associatedTokenProgramId = ASSOCIATED_TOKEN_PROGRAM_ID,
) => {
	const ata = getAssociatedTokenAddressSync(
		mint,
		owner,
		allowOwnerOffCurve,
		programId,
		associatedTokenProgramId,
	);
	return ata;
};

//Test with arbitrary accounts: https://litesvm.github.io/litesvm/tutorial.html#time-travel
export const setAta = (
	mint: PublicKey,
	owner: PublicKey,
	tokenAmount: bigint,
	allowOwnerOffCurve = true,
	programId = TOKEN_PROGRAM_ID,
	associatedTokenProgramId = ASSOCIATED_TOKEN_PROGRAM_ID,
) => {
	const ata = getAssociatedTokenAddressSync(
		mint,
		owner,
		allowOwnerOffCurve,
		programId,
		associatedTokenProgramId,
	);

	/* Set account via knowing its layout
  export interface RawAccount {
    mint: PublicKey;
    owner: PublicKey;
    amount: bigint;
    delegateOption: 1 | 0;
    delegate: PublicKey;
    state: AccountState;
    isNativeOption: 1 | 0;
    isNative: bigint;
    delegatedAmount: bigint;
    closeAuthorityOption: 1 | 0;
    closeAuthority: PublicKey;
}

// Buffer layout for de/serializing a token account
export const AccountLayout = struct<RawAccount>([
    publicKey('mint'),
    publicKey('owner'),
    u64('amount'),
    u32('delegateOption'),
    publicKey('delegate'),
    u8('state'),
    u32('isNativeOption'),
    u64('isNative'),
    u64('delegatedAmount'),
    u32('closeAuthorityOption'),
    publicKey('closeAuthority'),
]);

// Byte length of a token account 
export const ACCOUNT_SIZE = AccountLayout.span; */
	const rawTokenAcctData = Buffer.alloc(ACCOUNT_SIZE);
	AccountLayout.encode(
		{
			mint,
			owner,
			amount: tokenAmount,
			delegateOption: 0,
			delegate: PublicKey.default,
			delegatedAmount: 0n,
			state: 1,
			isNativeOption: 0,
			isNative: 0n,
			closeAuthorityOption: 0,
			closeAuthority: PublicKey.default,
		},
		rawTokenAcctData,
	);
	svm.setAccount(ata, {
		lamports: 1_000_000_000,
		data: rawTokenAcctData,
		owner: programId,
		executable: false,
	});
	const raw = svm.getAccount(ata);
	return { raw, ata };
};

export const getRawAcctData = (account: PublicKey) => {
	const raw = svm.getAccount(account);
	if (!raw) throw new Error("account is null");
	const rawAcctData = raw?.data;
	ll("rawAcctData:", rawAcctData);
	return rawAcctData;
};
export const tokBalc = (
	mint: PublicKey,
	owner: PublicKey,
	programId = TOKEN_PROGRAM_ID,
	associatedTokenProgramId = ASSOCIATED_TOKEN_PROGRAM_ID,
) => {
	const ata = getAssociatedTokenAddressSync(
		mint,
		owner,
		true, //allowOwnerOffCurve?
		programId,
		associatedTokenProgramId,
	);
	const rawAcctData = getRawAcctData(ata);
	const decoded = AccountLayout.decode(rawAcctData);
	return decoded.amount;
};

export const ataArrayBalc = (
	txnAccts: PublicKey[],
	amountsLen: number,
	decimals: number,
	numInGroup: number,
	isVerbose = false,
) => {
	checkTxnAccts(txnAccts.length, amountsLen, numInGroup);
	let vaultAta: PublicKey | undefined;
	let balc: bigint;
	const balcArray: bigint[] = [];
	for (let i = 0; i < amountsLen; i++) {
		vaultAta = txnAccts[i * numInGroup + 1];
		if (vaultAta === undefined) throw new Error("vaultAta is undefined");
		if (isVerbose) ll(`index ${i}: ${vaultAta.toBase58()}`);
		balc = ataBalc(vaultAta, `vault ${i}`, decimals, true);
		balcArray.push(balc);
	}
	return balcArray;
};
export const ataArrayBalCk = (
	txnAccts: PublicKey[],
	prevBalcs: bigint[],
	repayAmts: bigint[],
	debts: bigint[],
	decimals: number,
	numInGroup: number,
) => {
	const arraylen = debts.length;
	if (arraylen !== prevBalcs.length || arraylen !== repayAmts.length)
		throw new Error("one/more array length invalid");

	const balcs = ataArrayBalc(txnAccts, arraylen, decimals, numInGroup);
	let prevBalc: bigint | undefined;
	let repayAmt: bigint | undefined;
	let debt: bigint | undefined;
	for (const [i, balc] of balcs.entries()) {
		prevBalc = prevBalcs[i];
		if (prevBalc === undefined) {
			ll("index i = ", i);
			throw new Error("prevBalcs[i] undefined");
		}
		repayAmt = repayAmts[i];
		if (repayAmt === undefined) {
			ll("index i = ", i);
			throw new Error("repayAmt[i] undefined");
		}
		debt = debts[i];
		if (debt === undefined) {
			ll("index i = ", i);
			throw new Error("debts[i] undefined");
		}
		expect(balc).toStrictEqual(prevBalc + repayAmt - debt);
	}
};
export const ataBalc = (
	ata: PublicKey,
	name = "token balc",
	decimals: number,
	isVerbose = true,
	stopIfNull = true,
) => {
	const raw = svm.getAccount(ata);
	if (!raw) {
		if (isVerbose) ll(name, ": ata is null");
		if (stopIfNull) throw new Error("ata is null");
		return zero;
	}
	const rawAcctData = raw?.data;
	const decoded = AccountLayout.decode(rawAcctData);
	if (isVerbose)
		ll(
			name,
			"tok balc:",
			decoded.amount,
			decoded.amount / BigInt(10 ** decimals),
		);
	return decoded.amount;
};
export const ataBalCk = (
	ata: PublicKey,
	expectedAmount: bigint,
	name: string,
	decimals = 6,
	isVerbose = true,
) => {
	const amount = ataBalc(ata, name, decimals, isVerbose);
	ll(name, "token:", amount, amount / BigInt(10 ** decimals));
	expect(amount).toStrictEqual(expectedAmount);
};

export const setAtaCheck = (
	mint: PublicKey,
	user: PublicKey,
	amt: bigint,
	user_and_mint: string,
) => {
	const { raw: rawData, ata } = setAta(mint, user, amt);
	ll(user_and_mint, "ata:", ata.toBase58());
	expect(rawData).not.toBeNull();
	const rawAcctData = rawData?.data;
	if (rawAcctData) {
		const decoded = AccountLayout.decode(rawAcctData);
		ll(user_and_mint, "balc:", decoded.amount);
		expect(decoded.amount).toStrictEqual(amt);
	} else {
		ll(user_and_mint, "rawAcctData is undefined");
	}
};
//---------------== Deployment
export const deployFlashloanProgram = (
	addr: PublicKey,
	programPath = "target/deploy/pinocchio_flashloan.so",
	computeMaxUnits?: bigint,
) => {
	ll("load deployFlashloanProgram...");
	if (computeMaxUnits) {
		const computeBudget = new ComputeBudget();
		computeBudget.computeUnitLimit = computeMaxUnits;
		svm = svm.withComputeBudget(computeBudget);
	}
	//# Dump a program from mainnet
	//solana program dump progAddr pyth.so --url mainnet-beta

	svm.addProgramFromFile(addr, programPath);
	//return [programId];
};
deployFlashloanProgram(flashloanProgAddr);
acctExists(flashloanProgAddr);
ll("deployFlashloanProgram() is successful");
deployFlashloanProgram(
	funcCallerProgAddr,
	"program_bytes/pinocchio_flashloan.so",
); //which is compiled with different declare_id!(new_addr)
acctExists(funcCallerProgAddr);

//---------------== Run Test
export const sendTxns = (
	svm: LiteSVM,
	blockhash: string,
	ixs: TransactionInstruction[],
	signerKps: Keypair[],
	expectedError = "",
	programId = flashloanProgAddr,
) => {
	const tx = new Transaction();
	tx.recentBlockhash = blockhash;
	tx.add(...ixs);
	tx.sign(...signerKps); //first signature is considered "primary" and is used identify and confirm transactions.
	const simRes = svm.simulateTransaction(tx);
	const sendRes = svm.sendTransaction(tx);
	checkLogs(simRes, sendRes, programId, expectedError);
};
export const checkLogs = (
	simRes: FailedTransactionMetadata | SimulatedTransactionInfo,
	sendRes: TransactionMetadata | FailedTransactionMetadata,
	programId: PublicKey,
	expectedError = "",
	isVerbose = false,
) => {
	ll("\nsimRes meta prettylogs:", simRes.meta().prettyLogs());
	if (isVerbose) {
		ll("\nsimRes.meta().logs():", simRes.meta().logs());
	}
	/** simRes.meta():
      computeUnitsConsumed: [class computeUnitsConsumed],
      innerInstructions: [class innerInstructions],
      logs: [class logs],
      prettyLogs: [class prettyLogs],
      returnData: [class returnData],
      signature: [class signature],
      toString: [class toString], */
	if (sendRes instanceof TransactionMetadata) {
		expect(simRes.meta().logs()).toStrictEqual(sendRes.logs());

		const logLength = simRes.meta().logs().length;
		//ll("logLength:", logLength);
		//ll("sendRes.logs()[logIndex]:", sendRes.logs()[logIndex]);
		expect(sendRes.logs()[logLength - 1]).toStrictEqual(
			`Program ${programId} success`,
		);
	} else {
		ll("sendRes.err():", sendRes.err());
		ll("sendRes.meta():", sendRes.meta());
		const errStr = sendRes.toString();
		ll("sendRes.toString():", errStr);
		const pos = errStr.search("custom program error: 0x");
		ll("pos:", pos);
		if (pos > -1) {
			let errCode = errStr.substring(pos + 22, pos + 26);
			if (errCode.slice(-1) === '"') {
				//ll("last char:", errCode.slice(-1));
				errCode = errCode.slice(0, -1);
			}
			ll("error code:", errCode, Number(errCode));
		}
		ll(
			"find error here: https://docs.rs/solana-sdk/latest/solana_sdk/transaction/enum.TransactionError.html",
		);
		if (expectedError) {
			const foundErrorMesg = sendRes
				.toString()
				.includes(`custom program error: ${expectedError}`);
			ll("found error?:", foundErrorMesg);
			expect(foundErrorMesg).toEqual(true);
		} else {
			throw new Error("This error is unexpected");
		}
	}
};
