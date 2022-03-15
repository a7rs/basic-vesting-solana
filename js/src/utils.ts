import BN from 'bn.js';
import assert from 'assert'
import nacl from 'tweetnacl';
import * as bip32 from 'bip32';
import {
	Keypair,
	Account,
	Connection,
	Transaction,
	TransactionInstruction,
	PublicKey,
} from '@solana/web3.js';

import { TOKEN_PROGRAM_ID } from '@solana/spl-token';

const U64SIZE: number = 8;

export class Numberu64 extends BN {
	toBuffer(): Buffer {
		const a = super.toArray().reverse();
		const b = Buffer.from(a);
		if (b.length === U64SIZE) {
			return b;
		}
		assert(b.length < U64SIZE, 'Numberu64 is limited to 8 bytes'); 
		const zeroPad = Buffer.alloc(U64SIZE);
		b.copy(zeroPad);
		return zeroPad;
	}

	static fromBuffer(buffer: any): any {
		assert(buffer.length === U64SIZE, `Invalid buffer length: ${buffer.length}`);
		return new BN(
			[...buffer]
				.reverse()
				.map(i => `00${i.toString(16)}`.slice(-2))
				.join(''),
			16s,
		);
	}
}

export class Numberu32 extends BN {
  toBuffer(): Buffer {
    const a = super.toArray().reverse();
    const b = Buffer.from(a);
    if (b.length === 4) {
      return b;
    }
    assert(b.length < 4, 'Numberu32 too large');

    const zeroPad = Buffer.alloc(4);
    b.copy(zeroPad);
    return zeroPad;
  }

  static fromBuffer(buffer: any): any {
    assert(buffer.length === 4, `Invalid buffer length: ${buffer.length}`);
    return new BN(
      [...buffer]
        .reverse()
        .map(i => `00${i.toString(16)}`.slice(-2))
        .join(''),
      16,
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

interface AccountMeta {
	pubkey: PublicKey,
	isSigner: boolean,
	isWritable: boolean,
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
		data: Buffer.from([]).
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