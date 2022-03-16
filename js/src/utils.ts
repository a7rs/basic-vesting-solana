import BN from 'bn.js';
import assert from 'assert'
import nacl from 'tweetnacl';
import * as bip32 from 'bip32';
import {
	Keypair,
	Account,
	AccountMeta,
	Connection,
	Transaction,
	TransactionInstruction,
	PublicKey,
} from '@solana/web3.js';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';

const U64SIZE: number = 8;
const U32SIZE: number = 4;
const DISCRIMINANT: number = 16;
const SLICE: number = -2;
const ENDPOINTS = {
	mainnet: "https://api.mainnet-beta.solana.com",
	devnet: "https://api.devnet.solana.com",
	testnet: "https://api.testnet.solana.com",
	localhost: "127.0.1.1",
};

const connection = new Connection(ENDPOINTS.mainnet);

export class Numberu64 extends BN {
	toBuffer(): Buffer {
		const a = super.toArray().reverse();
		const b = Buffer.from(a);
		if (b.length === U64SIZE) {
			return b;
		}
		assert(b.length < U64SIZE, `Numberu64 is limited to ${U64SIZE} bytes`); 
		const zeroPad = Buffer.alloc(U64SIZE);
		b.copy(zeroPad);
		return zeroPad;
	}

	static fromBuffer(buffer: any): any {
		assert(buffer.length === U64SIZE, `Invalid buffer length: ${buffer.length}`);
		return new BN(
			[...buffer]
				.reverse()
				.map(i => `00${i.toString(DISCRIMINANT)}`.slice(SLICE))
				.join(''),
			DISCRIMINANT,
		);
	}
}

export class Numberu32 extends BN {
  toBuffer(): Buffer {
    const a = super.toArray().reverse();
    const b = Buffer.from(a);
    if (b.length === U32SIZE) {
      return b;
    }
    assert(b.length < U32SIZE, `Numberu32 is limited to ${U32SIZE} bytes`);

    const zeroPad = Buffer.alloc(U32SIZE);
    b.copy(zeroPad);
    return zeroPad;
  }

  static fromBuffer(buffer: any): any {
    assert(buffer.length === U32SIZE, `Invalid buffer length: ${buffer.length}`);
    return new BN(
      [...buffer]
        .reverse()
        .map(i => `00${i.toString(DISCRIMINANT)}`.slice(SLICE))
        .join(''),
      DISCRIMINANT,
    );
  }
}

interface MetadataEndpoint {
	programId: PublicKey,
	authority: PublicKey,
	metadata: PublicKey,
	vault: PublicKey,
	duration: Numberu64,
	apr: Numberu64,
	withdrawalTimelock: Numberu64,
	earlyWithdrawalFee: Numberu64,
}

interface VestingEndpoint {
	programId: PublicKey,
	authority: PublicKey,
	vestingAccount: PublicKey,
	vault: PublicKey,
	metadata: PublicKey,
	tokenProgram: PublicKey,
	amount: Numberu64,
}

const unpackMetadataKeys = (
	ctx: MetadataEndpoint,
): Array<AccountMeta> => {
	const keys = [
		{
			pubkey: ctx.authority,
			isSigner: true,
			isWritable: true,
		},
		{
			pubkey: ctx.metadata,
			isSigner: false,
			isWritable: true,
		},
		{
			pubkey: ctx.vault,
			isSigner: false,
			isWritable: true,
		},
	];

	return keys;
};

const unpackVestingKeys = (
	ctx: VestingEndpoint,
): Array<AccountMeta> => {
	const keys = [
		{
			pubkey: ctx.authority,
			isSigner: true,
			isWritable: true,
		},
		{
			pubkey: ctx.vestingAccount,
			isSigner: false,
			isWritable: true,
		},
		{
			pubkey: ctx.vault,
			isSigner: false,
			isWritable: true,
		},
		{
			pubkey: ctx.metadata,
			isSigner: false,
			isWritable: false,
		},
		{
			pubkey: ctx.tokenProgram,
			isSigner: false,
			isWritable: false,
		},
	];

	return keys;
};

export const createVestingMetadata = async (
	ctx: MetadataEndpoint,
): Promise<TransactionInstruction> => {
	const keys = unpackMetadataKeys(ctx);

	return new TransactionInstruction({
		keys,
		programId: ctx.programId,
		data: Buffer.from([]),
	});
};

export const readVestingMetadata = async (
	ctx: MetadataEndpoint,
): Promise<TransactionInstruction> => {
	const keys = unpackMetadataKeys(ctx);

	return new TransactionInstruction({
		keys,
		programId: ctx.programId,
		data: Buffer.from([]),
	});
};

export const updateVestingMetadata = async (
	ctx: MetadataEndpoint,
): Promise<TransactionInstruction> => {
	const keys = unpackMetadataKeys(ctx);

	return new TransactionInstruction({
		keys,
		programId: ctx.programId,
		data: Buffer.from([]),
	});
};

export const deleteVestingMetadata = async (
	ctx: MetadataEndpoint,
): Promise<TransactionInstruction> => {
	const keys = unpackMetadataKeys(ctx);

	return new TransactionInstruction({
		keys,
		programId: ctx.programId,
		data: Buffer.from([]),
	});
};

export const initVesting = async (
	programId: PublicKey,
	authority: PublicKey,
	vesting: PublicKey,
	systemProgramId: PublicKey,
): Promise<TransactionInstruction> => {
	const keys = [
		{
			pubkey: authority,
			isSigner: true,
			isWritable: true,
		},
		{
			pubkey: vesting,
			isSigner: false,
			isWritable: true,
		},
		{
			pubkey: systemProgramId,
			isSigner: false,
			isWritable: false,
		},
	];

	return new TransactionInstruction({
		keys,
		programId,
		data: Buffer.from([]),
	});
};

export const createNewVesting = async (
	ctx: VestingEndpoint,
): Promise<TransactionInstruction> => {
	const keys = unpackVestingKeys(ctx);

	return new TransactionInstruction({
		keys,
		programId: ctx.programId,
		data: Buffer.from([]),
	});
};

export const withdrawTokens = async (
	ctx: VestingEndpoint,
): Promise<TransactionInstruction> => {
	const keys = unpackVestingKeys(ctx);

	return new TransactionInstruction({
		keys,
		programId: ctx.programId,
		data: Buffer.from([]),
	});
};

export const setAuthority = async (
	programId: PublicKey,
	authority: PublicKey,
	newAuthority: PublicKey,
): Promise<TransactionInstruction> => {
	const keys = [
		{
			pubkey: authority,
			isSigner: true,
			isWritable: true,
		},
	];

	return new TransactionInstruction({
		keys,
		programId,
		data: Buffer.from([]),
	});
};

export const setBeneficiary = async (
	programId: PublicKey,
	authority: PublicKey,
	newBeneficiary: PublicKey,
): Promise<TransactionInstruction> => {
	let keys = [
		{
			pubkey: authority,
			isSigner: true,
			isWritable: false,
		}
	];

	return new TransactionInstruction({
		keys,
		programId,
		data: Buffer.from([]),
	});
};

export const signTransactionInstruction = async (
	connection: Connection,
	signers: Array<Keypair>,
	feePayer: PublicKey,
	txInstructions: Array<TransactionInstruction>
): Promise<string> => {
	const tx = new Transaction();
	tx.feePayer = feePayer;
	tx.add(...txInstructions);

	return await connection.sendTransaction(tx, signers, {
		preflightCommitment: 'single',
	});
};
