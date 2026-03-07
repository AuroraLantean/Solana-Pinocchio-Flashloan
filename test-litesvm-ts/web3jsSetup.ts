import { Keypair, PublicKey } from "@solana/web3.js";

const ll = console.log;
ll("\n------== web3jsSetup");
export const ownerKp = new Keypair();
export const adminKp = new Keypair();
export const user1Kp = new Keypair();
export const user2Kp = new Keypair();
export const user3Kp = new Keypair();
export const hackerKp = new Keypair();
export const dgcAuthorityKp = new Keypair();
export const dragonCoinKp = new Keypair();
export const owner = ownerKp.publicKey;
export const admin = adminKp.publicKey;
export const user1 = user1Kp.publicKey;
export const user2 = user2Kp.publicKey;
export const user3 = user3Kp.publicKey;
export const hacker = hackerKp.publicKey;
export const dgcAuthority = dgcAuthorityKp.publicKey;
export const dragonCoin = dragonCoinKp.publicKey;

export const flashloanProgAddr = new PublicKey(
	"FcLwqf7L3VyxWuMKKzLA7vJqBo8bj9i3zHkxLe65Z1Ad",
);
ll("flashloanProgAddr:", flashloanProgAddr.toBase58());
export const funcCaller = new PublicKey(
	"8ZEf7xqzjxELMywQbCEeYVgw9zDCijA3saDCHZwgbe7i",
);
ll("funcCaller:", funcCaller.toBase58());
export const futureOptionAddr = new PublicKey(
	"CgZEcSRPh1Ay1EYR4VJPTJRYcRkTDjjZhBAjZ5M8keGp",
);
ll("futureOptionAddr:", futureOptionAddr.toBase58());

export const SYSTEM_PROGRAM = new PublicKey("11111111111111111111111111111111"); //default
export const RentSysvar = new PublicKey(
	"SysvarRent111111111111111111111111111111111",
); //RENT_ID
export const usdcMint = new PublicKey(
	"EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
); //decimals = 6
export const usdtMint = new PublicKey(
	"Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
); //decimals = 6
export const pyusdMint = new PublicKey(
	"2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo",
); //token2022, decimals = 6... https://docs.paxos.com/guides/stablecoin/pyusd/mainnet
export const usdgMint = new PublicKey(
	"2u1tszSeqZ3qBWF3uNGPFc8TzMk2tdiwknnRMWGWjGWH",
); //token2022, decimals = 6... https://docs.paxos.com/guides/stablecoin/usdg/mainnet
export const ATokenGPvbd = new PublicKey(
	"ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL",
);
export const decUsdx = 6;
