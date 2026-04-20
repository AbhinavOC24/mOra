import { PublicKey } from "@solana/web3.js";

export const PROGRAM_ID = new PublicKey(
  "E7TZEgFboKppM4V8yix6UEQrBh2WL2rDqbbn2JYxPnat"
);
// Replace with actual MORA mint after token deployment
// Using SystemProgram ID as a no-op placeholder for dev
export const MORA_MINT = new PublicKey(
  "11111111111111111111111111111111"
);
export const NETWORK = "devnet";
export const RPC_URL = "https://api.devnet.solana.com";

export const MAX_LOCK_DURATION = 63_072_000; // 2 years in seconds
export const SCALE = 1_000_000_000_000; // 10^12

export function getGlobalPda(programId: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync([Buffer.from("global")], programId)[0];
}

export function getLockingConfigPda(programId: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync([Buffer.from("locking_config")], programId)[0];
}

export function getLockPositionPda(owner: PublicKey, programId: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("lock_position"), owner.toBuffer()],
    programId
  )[0];
}

export function getAssertionPda(assertionId: bigint, programId: PublicKey): PublicKey {
  const buf = Buffer.alloc(8);
  buf.writeBigUInt64LE(assertionId);
  return PublicKey.findProgramAddressSync([Buffer.from("assertion"), buf], programId)[0];
}

export function getVlaRoundPda(assertionId: bigint, programId: PublicKey): PublicKey {
  const buf = Buffer.alloc(8);
  buf.writeBigUInt64LE(assertionId);
  return PublicKey.findProgramAddressSync([Buffer.from("vla_round"), buf], programId)[0];
}

export function getVoteRecordPda(assertionId: bigint, voter: PublicKey, programId: PublicKey): PublicKey {
  const buf = Buffer.alloc(8);
  buf.writeBigUInt64LE(assertionId);
  return PublicKey.findProgramAddressSync(
    [Buffer.from("vote_record"), buf, voter.toBuffer()],
    programId
  )[0];
}

export function formatTimestamp(ts: number): string {
  return new Date(ts * 1000).toLocaleString();
}

export function formatMora(lamports: bigint): string {
  return (Number(lamports) / 1e9).toFixed(2);
}

export function calcVotingPower(amountLocked: bigint, lockEnd: bigint, now: bigint, maxDuration: bigint): bigint {
  if (lockEnd <= now || amountLocked === 0n) return 0n;
  const remaining = lockEnd - now;
  const scale = 1_000_000_000_000n;
  const slope = (amountLocked * scale) / maxDuration;
  return (slope * remaining) / scale;
}
