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
export const checkBump = (value: number, decimalName = "u8 value") => {
	if (value > 255 || value < 1) throw new Error(`${decimalName} invalid`);
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
export type IxKeyArray = {
	pubkey: PublicKey;
	isSigner: boolean;
	isWritable: boolean;
};

export const makeIxKeyArray = (tokenAccts: PublicKey[], amounts: bigint[]) => {
	const tokAcctsLen = tokenAccts.length;
	const amountsLen = amounts.length;
	if (tokAcctsLen === 0) throw new Error("tokAcctsLen is zero");
	if (tokAcctsLen % 2 !== 0)
		throw new Error("tokenAccts length should be an even number");
	if (tokAcctsLen / 2 !== amountsLen)
		throw new Error("amounts length should match tokAcctLen/2");

	ll("loop over amountsLen index...");
	const u64bytes: number[] = [];
	const ixKeyArray: IxKeyArray[] = [];
	let amount = 0n;
	let lenderTokBalc = 0n;
	for (let i = 0; i < amountsLen; i++) {
		ll("index = ", i);
		if (amounts[i] === undefined) throw new Error("amounts[i] undefined");
		amount = amounts[i] ?? 0n;
		if (amount === 0n) throw new Error(`amount at {i} is zero`);

		u64bytes.push(...numToBytes(amount, 64));
		const lenderTokAcct = tokenAccts[i * 2];
		const borrowerTokAcct = tokenAccts[i * 2 + 1];
		if (lenderTokAcct === undefined) throw new Error("lenderTokAcct undefined");
		if (borrowerTokAcct === undefined)
			throw new Error("borrowerTokAcct undefined");

		acctExists(lenderTokAcct);
		ll("lenderTokAcct exists");
		acctExists(borrowerTokAcct);
		ll("borrowerTokAcct exists");

		lenderTokBalc = ataBalc(lenderTokAcct, "lenderTokAcct", true);
		if (lenderTokBalc === 0n) throw new Error("lenderTokBalc is zero");
		if (amount > lenderTokBalc)
			throw new Error("borrowed amount > lenderTokBalc");

		ixKeyArray.push({
			pubkey: lenderTokAcct,
			isSigner: false,
			isWritable: true,
		});
		ixKeyArray.push({
			pubkey: borrowerTokAcct,
			isSigner: false,
			isWritable: true,
		});
	}
	return { u64bytes, ixKeyArray };
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
