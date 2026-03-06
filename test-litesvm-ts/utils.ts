import type { Lamports } from "@solana/kit";
import {
	type Address,
	address,
	getAddressEncoder,
	getLamportsDecoder,
	getLamportsEncoder,
	getProgramDerivedAddress,
	getU8Encoder,
	getU16Decoder,
	getU16Encoder,
	getU32Encoder,
	getU64Decoder,
	getU64Encoder,
	getUtf8Encoder,
	lamports,
} from "@solana/kit";
import type { PublicKey } from "@solana/web3.js";
import chalk from "chalk";
import * as flashloan from "../clients/js/src/generated/index";
import { acctExists, ataBalc } from "./litesvm-utils";
export const ll = console.log;
//-----------== General Config
export const network = "mainnet-beta"; //devnet
export const PROJECT_DIRECTORY = ""; // Leave empty if using default anchor project

export const USDC_DECIMALS = 6;
export const USDT_DECIMALS = 6;
export const LAMPORTS_PER_SOL = 1000000000;

export const MINIMUM_SLOT = 100;
export const USDC_BALANCE = 100_000_000_000; // 100k USDC
export const Transaction_Fee = 5000n;
export const day = 86400;
export const week = 604800;

export const zero = BigInt(0);
export const ten = BigInt(10);
export const decimalsSOL = BigInt(9);
export const baseSOL = BigInt(10) ** decimalsSOL;

//-----------==
export const bigintAmt = (amount: number, decimals = 6) =>
	BigInt(amount) * 10n ** BigInt(decimals);

export const as6zBn = (amt: number) => BigInt(amt * 10 ** 6);
export const as9zBn = (amt: number) => {
	if (Number.isInteger(amt)) {
		return BigInt(amt) * baseSOL;
	}
	return BigInt(amt * 10 ** 9);
};
export const fromLam = (amt: number) => BigInt(amt) / baseSOL;
export const checkDecimals = (decimals: number, decimalName = "decimals") => {
	if (decimals > 12 || decimals < 0) throw new Error(`${decimalName} invalid`);
};
export const checkBump = (value: number) => {
	if (value > 255 || value < 1) throw new Error(`bump ${value} is invalid`);
};
export const checkFee = (value: number, decimalName = "fee_u16 value") => {
	if (value > 65535 || value < 1) throw new Error(`${decimalName} invalid`);
};
export const checkBigint = (bint: bigint, bigintName = "bigint") => {
	if (bint <= 0) throw new Error(`${bigintName} invalid`);
};
//-----------== SolanaKit setup
export const flashloanProgAddr = flashloan.PINOCCHIO_FLASHLOAN_PROGRAM_ADDRESS;
ll("flashloanProgAddr:", flashloanProgAddr);
export const ATokenGPvbd = address(
	"ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL",
);
export const usdcMint = address("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
//decimals = 6
export const usdtMint = address("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"); //decimals = 6

//-----------==
export const findPdaV2 = async (
	addr: Address<string>,
	seedStr: string,
	pdaName: string,
	progAddr = flashloanProgAddr,
) => {
	const seedSigner = getAddressEncoder().encode(addr);
	const seedTag = getUtf8Encoder().encode(seedStr);

	const [pda, bump] = await getProgramDerivedAddress({
		programAddress: progAddr,
		seeds: [seedTag, seedSigner],
	});
	ll(`${pdaName} pda: ${pda}, bump: ${bump}`);
	return { pda, bump };
};

export type SolanaAccount = {
	account: {
		data: string[];
		executable: boolean;
		lamports: number;
		owner: string;
		rentEpoch: number;
		space: number;
	};
	pubkey: string;
};
//--------------==
export const bigIntSum = (bigintArray: bigint[]) => {
	let sum = 0n;
	for (const item of bigintArray) {
		sum = sum + item;
	}
	return sum;
};
export type IxKeyArray = {
	pubkey: PublicKey;
	isSigner: boolean;
	isWritable: boolean;
};
/// Maximum amountsLen is 8, minimum is 1
export const checkTxnAccts = (
	txnAcctsLen: number,
	amountsLen: number,
	numInGroup: number,
) => {
	if (txnAcctsLen < numInGroup) throw new Error(`txnAcctsLen < ${numInGroup}`);
	if (amountsLen > 8) throw new Error("amounts length should be <= 8");
	if (txnAcctsLen % numInGroup !== 0)
		throw new Error(`txnAccts length should be a multiple of ${numInGroup}`);
	if (txnAcctsLen / numInGroup !== amountsLen)
		throw new Error(`amounts length should match txnAcctsLen/${numInGroup}`);
};
export const makeFlashloanIxKeys = (
	txnAccts: PublicKey[],
	amounts: bigint[],
	decimals: number,
) => {
	const txnAcctsLen = txnAccts.length;
	const amountsLen = amounts.length;
	checkTxnAccts(txnAcctsLen, amountsLen, 3);

	ll("loop over amountsLen index...");
	const u64bytes: number[] = [];
	const ixKeyArray: IxKeyArray[] = [];
	let vaultTokBalc = 0n;
	for (const [i, amount] of amounts.entries()) {
		ll("amount: ", amount);
		if (amount === undefined) throw new Error("amounts[i] undefined");
		if (amount === 0n) throw new Error(`amount at {i} is zero`);

		u64bytes.push(...numToBytes(amount, 64));
		const vaultPda = txnAccts[i * 3];
		const vaultAta = txnAccts[i * 3 + 1];
		const userAta = txnAccts[i * 3 + 2];
		if (vaultPda === undefined) throw new Error("vaultPda undefined");
		if (vaultAta === undefined) throw new Error("vaultAta undefined");
		if (userAta === undefined) throw new Error("userAta undefined");

		acctExists(vaultPda);
		ll("vaultPda exists");
		acctExists(vaultAta);
		ll("vaultAta exists");
		acctExists(userAta);
		ll("userAta exists");

		vaultTokBalc = ataBalc(vaultAta, `vaultAta ${i}`, decimals, true);
		if (vaultTokBalc === 0n) throw new Error("vaultTokBalc is zero");
		if (amount > vaultTokBalc)
			throw new Error("borrowed amount > vaultTokBalc");

		ixKeyArray.push({
			pubkey: vaultPda,
			isSigner: false,
			isWritable: true,
		});
		ixKeyArray.push({
			pubkey: vaultAta,
			isSigner: false,
			isWritable: true,
		});
		ixKeyArray.push({
			pubkey: userAta,
			isSigner: false,
			isWritable: true,
		});
	}
	ll("makeFlashloanIxKeys successful");
	return { u64bytes, ixKeyArray };
};
export const makeDepositIxKeys = (txnAccts: PublicKey[], amounts: bigint[]) => {
	ll("------== makeDepositIxKeys");
	const txnAcctsLen = txnAccts.length;
	const amountsLen = amounts.length;
	checkTxnAccts(txnAcctsLen, amountsLen, 2);

	ll("loop over amountsLen index...");
	const u64bytes: number[] = [];
	const ixKeyArray: IxKeyArray[] = [];
	for (const [i, amount] of amounts.entries()) {
		ll("amount: ", amount);
		if (amount === undefined) throw new Error("amounts[i] undefined");
		if (amount === 0n) throw new Error(`amount at {i} is zero`);

		u64bytes.push(...numToBytes(amount, 64));
		const vaultPda = txnAccts[i * 2];
		const vaultAta = txnAccts[i * 2 + 1];
		if (vaultPda === undefined) throw new Error("vaultPda undefined");
		if (vaultAta === undefined) throw new Error("vaultAta undefined");

		acctExists(vaultPda);
		ll("vaultPda exists");
		//acctExists(vaultAta);
		//ll("vaultAta exists");

		ixKeyArray.push({
			pubkey: vaultPda,
			isSigner: false,
			isWritable: true,
		});
		ixKeyArray.push({
			pubkey: vaultAta,
			isSigner: false,
			isWritable: true,
		});
	}
	ll("makeDepositIxKeys successful");
	return { u64bytes, ixKeyArray };
};
export const makeVaultInitIxKeys = (
	vaults: PublicKey[],
	fees: number[],
	vaultBumps: number[],
) => {
	ll("------== makeVaultInitIxKeys");
	const vaultsLen = vaults.length;
	if (vaultsLen > 8) throw new Error("vaults length should be <= 8");
	if (vaultsLen !== fees.length || vaultsLen !== vaultBumps.length)
		throw new Error("vaults length != fees length or vaultBumps length");

	ll("loop over vaultsLen index...");
	const ixKeyArray: IxKeyArray[] = [];
	let bump: number | undefined;
	let fee: number | undefined;
	const feesU8: number[] = [];
	for (const [i, vault] of vaults.entries()) {
		fee = fees[i];
		if (fee === undefined) throw new Error(`fees[i] is undefined`);
		checkFee(fee);
		feesU8.push(...numToBytes(fee, 16));

		bump = vaultBumps[i];
		if (bump === undefined) throw new Error(`bump ${bump} invalid`);
		checkBump(bump);

		ixKeyArray.push({
			pubkey: vault,
			isSigner: false,
			isWritable: true,
		});
	}
	ll("makeDepositIxKeys successful");
	return { ixKeyArray, feesU8 };
};
//--------------== Bytes
export const u16Bytes = [0, 0];
export const u32Bytes = [0, 0, 0, 0];
export const u32x4Bytes = [...u32Bytes, ...u32Bytes, ...u32Bytes, ...u32Bytes];
export const u64Bytes = [0, 0, 0, 0, 0, 0, 0, 0];
export const u64x4Bytes = [...u64Bytes, ...u64Bytes, ...u64Bytes, ...u64Bytes];
export const u32Max = 4294967295n;
export const u8Max = 255n;

export const numToBytes = (input: bigint | number, bit = 64) => {
	let amtBigint = 0n;
	if (typeof input === "number") {
		if (input < 0) throw new Error("input < 0");
		amtBigint = BigInt(input);
	} else {
		if (input < 0n) throw new Error("input < 0");
		amtBigint = input;
	}
	const amtLam = lamports(amtBigint);
	// biome-ignore lint/suspicious/noExplicitAny: <>
	let lamportsEncoder: any;
	if (bit === 64) {
		lamportsEncoder = getLamportsEncoder(getU64Encoder());
	} else if (bit === 32) {
		lamportsEncoder = getLamportsEncoder(getU32Encoder());
	} else if (bit === 16) {
		lamportsEncoder = getLamportsEncoder(getU16Encoder());
	} else if (bit === 8) {
		lamportsEncoder = getLamportsEncoder(getU8Encoder());
	} else {
		throw new Error("bit unknown");
		//lamportsEncoder = getDefaultLamportsEncoder()
	}
	const u8Bytes: Uint8Array = lamportsEncoder.encode(amtLam);
	ll("u8Bytes", u8Bytes);
	return u8Bytes;
};

export const bytesToBigint = (bytes: Uint8Array) => {
	let bigint: Lamports = lamports(0n);
	const length = bytes.length;
	// bytes = decoder.decode(new Uint8Array([0x2a, 0x00, 0x00, 0x00]));
	if (length === 8) {
		//u64. Returns a decoder that you can use to decode a byte array representing a 64-bit little endian number to a {@link Lamports} value.
		const lamportsDecoder = getLamportsDecoder(getU64Decoder()); //getDefaultLamportsDecoder()
		bigint = lamportsDecoder.decode(bytes);
	} else if (length === 4) {
		//u32
		const newBytes = new Uint8Array([...bytes, 0, 0, 0, 0]);
		const lamportsDecoder = getLamportsDecoder(getU64Decoder());
		bigint = lamportsDecoder.decode(newBytes);
		/*const _decoder = getU32Decoder();
		const _lamportsDecoder = getLamportsEncoder(decoder);*/
	} else if (length === 2) {
		//u16
		const lamportsDecoder = getLamportsDecoder(getU16Decoder());
		bigint = lamportsDecoder.decode(bytes);
	} else if (length === 1) {
		//u8
		const newBytes = new Uint8Array([...bytes, 0, 0, 0, 0, 0, 0, 0]);
		const lamportsDecoder = getLamportsDecoder(getU64Decoder());
		bigint = lamportsDecoder.decode(newBytes);
		/*const _decoder = getU8Decoder();
		const _lamportsDecoder = getLamportsEncoder(decoder);*/
	} else {
		throw new Error("bit unknown");
		//lamportsEncoder = getDefaultLamportsCodec()
	}
	ll("bytesToBigint:", bigint);
	return bigint;
};

//--------------==
export const llBl = (txt: string) => {
	ll(chalk.blue(txt));
};
export const llGn = (txt: string) => {
	ll(chalk.green(txt));
};
export const llRd = (txt: string) => {
	ll(chalk.red(txt));
};
export const llYl = (txt: string) => {
	ll(chalk.yellow(txt));
};
export const llbalc = (name: string, amt: string) => {
	ll(`${chalk.bgBlue(name)} balc: ${chalk.yellow(amt)}`);
};

export const getTime = () => {
	const time = Math.floor(Date.now() / 1000);
	ll("JS time:", time);
	return time;
};
export const getTimeBig = () => {
	const time = getTime();
	return BigInt(time);
};
