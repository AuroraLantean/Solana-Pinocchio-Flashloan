import { expect } from "bun:test";
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
	checkBigint,
	checkDecimals,
	makeIxKeyArray,
	numToBytes,
	zero,
} from "./utils";
import {
	ATokenGPvbd,
	admin,
	flashloanProgAddr,
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

export const findLoanRecordsV1 = (
	fee: number,
	pdaName: string,
	seedStr = "moon_pool",
	progAddr = flashloanProgAddr,
): PdaOut => {
	const [pda, bump] = PublicKey.findProgramAddressSync(
		[Buffer.from(seedStr), Buffer.from(numToBytes(fee, 16))],
		progAddr,
	);
	ll(`${pdaName} pda: ${pda.toBase58()}, bump: ${bump}`);
	return { pda, bump };
};
//-------------== Program Methods
export const vaultInit = (
	userSigner: Keypair,
	vaultPda: PublicKey,
	//configPda: PublicKey,
	fee: number,
	vaultBump: number,
) => {
	const disc = 0;
	if (vaultBump > 255) throw new Error("vault_bump > 255");
	const argData = [vaultBump, ...numToBytes(fee, 16)];
	const blockhash = svm.latestBlockhash();
	const ix = new TransactionInstruction({
		keys: [
			{ pubkey: userSigner.publicKey, isSigner: true, isWritable: true },
			{ pubkey: vaultPda, isSigner: false, isWritable: true }, // true
			{ pubkey: SYSTEM_PROGRAM, isSigner: false, isWritable: false },
			{ pubkey: RentSysvar, isSigner: false, isWritable: false },
		],
		programId: flashloanProgAddr,
		data: Buffer.from([disc, ...argData]),
	});
	sendTxns(svm, blockhash, [ix], [userSigner]);
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

export const tokLgcDeposit = (
	userSigner: Keypair,
	fromAta: PublicKey,
	toAta: PublicKey,
	vault: PublicKey,
	mint: PublicKey,
	//configPda: PublicKey,
	decimals: number,
	amount: bigint,
	tokenProg = TOKEN_PROGRAM_ID,
	atokenProg = ATokenGPvbd,
) => {
	const disc = 2;
	checkDecimals(decimals);
	checkBigint(amount, "amount");
	const argData = [decimals, ...numToBytes(amount)];
	const blockhash = svm.latestBlockhash();
	const ix = new TransactionInstruction({
		keys: [
			{ pubkey: userSigner.publicKey, isSigner: true, isWritable: true },
			{ pubkey: fromAta, isSigner: false, isWritable: true },
			{ pubkey: toAta, isSigner: false, isWritable: true },
			{ pubkey: vault, isSigner: false, isWritable: true }, // true
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

export const flashloan = (
	userSigner: Keypair,
	lenderPda: PublicKey,
	loanRecordsPda: PublicKey,
	//lenderAta: PublicKey,
	//userAta: PublicKey,
	mint: PublicKey,
	tokenProgram: PublicKey,
	//configPda: PublicKey,
	tokenAccounts: PublicKey[],
	decimals: number,
	bump: number,
	fee: number,
	amounts: bigint[],
) => {
	const borrow_disc = 3;
	const _repay_disc = 4;
	acctIsNull(loanRecordsPda);

	if (decimals < 0 || decimals > 18) throw new Error("decimal out of range");
	if (bump < 1 || bump > 255) throw new Error("bump out of range");
	if (fee < 1 || fee > 65535) throw new Error("fee out of range");

	const { u64bytes, ixKeyArray } = makeIxKeyArray(tokenAccounts, amounts);
	const argData = [decimals, bump, ...numToBytes(fee, 16), ...u64bytes];

	const blockhash = svm.latestBlockhash();
	const ix = new TransactionInstruction({
		keys: [
			{ pubkey: userSigner.publicKey, isSigner: true, isWritable: true },
			{ pubkey: lenderPda, isSigner: false, isWritable: false },
			{ pubkey: loanRecordsPda, isSigner: false, isWritable: true },
			{ pubkey: mint, isSigner: false, isWritable: false },
			{
				pubkey: tokenProgram,
				isSigner: false,
				isWritable: false,
			},
			{
				pubkey: RentSysvar,
				isSigner: false,
				isWritable: false,
			},
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
	sendTxns(svm, blockhash, [ix], [userSigner], "", flashloanProgAddr);
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
	expect(raw).toBeNull();
};
export const acctExists = (account: PublicKey) => {
	const raw = svm.getAccount(account);
	expect(raw).not.toBeNull();
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
export const ataBalc = (
	ata: PublicKey,
	name = "token balc",
	isVerbose = true,
) => {
	const raw = svm.getAccount(ata);
	if (!raw) {
		if (isVerbose) ll(name, ": ata is null");
		return zero;
	}
	const rawAcctData = raw?.data;
	const decoded = AccountLayout.decode(rawAcctData);
	if (isVerbose) ll(name, ":", decoded.amount);
	return decoded.amount;
};
export const ataBalCk = (
	ata: PublicKey,
	expectedAmount: bigint,
	name: string,
	decimals = 6,
) => {
	const amount = ataBalc(ata, name, false);
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
export const deployFlashloanProgram = (computeMaxUnits?: bigint) => {
	ll("load deployFlashloanProgram...");
	if (computeMaxUnits) {
		const computeBudget = new ComputeBudget();
		computeBudget.computeUnitLimit = computeMaxUnits;
		svm = svm.withComputeBudget(computeBudget);
	}
	const programPath = "target/deploy/pinocchio_flashloan.so";
	//# Dump a program from mainnet
	//solana program dump progAddr pyth.so --url mainnet-beta

	svm.addProgramFromFile(flashloanProgAddr, programPath);
	//return [programId];
};
deployFlashloanProgram();
ll("deployFlashloanProgram() is successful");
acctExists(flashloanProgAddr);

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
