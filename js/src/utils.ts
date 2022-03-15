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
};

import { TOKEN_PROGRAM_ID } from '@solana/spl-token';

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

interface Accounts {
	pubkey: PublicKey,
	isSigner: boolean,
	isWritable: boolean,
}

const unpackMetadataKeys = (
	ctx: MetadataEndpoint,
): Array<Accounts> => {
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
): Array<Accounts> => {
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
		programId,
		data: Buffer.from([]),
	});
};

export const readVestingMetadata = async (
	ctx: MetadataEndpoint,
): Promise<TransactionInstruction> => {
	const keys = unpackMetadataKeys(ctx);

	return new TransactionInstruction({
		keys,
		programId,
		data: Buffer.from([]),
	});
};

export const updateVestingMetadata = async (
	ctx: MetadataEndpoint,
): Promise<TransactionInstruction> => {
	const keys = unpackMetadataKeys(ctx);

	return new TransactionInstruction({
		keys,
		programId,
		data: Buffer.from([]),
	});
};

export const deleteVestingMetadata = async (
	ctx: MetadataEndpoint,
): Promise<TransactionInstruction> => {
	const keys = unpackMetadataKeys(ctx);

	return new TransactionInstruction({
		keys,
		programId,
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
		programId,
		data: Buffer.from([]),
	});
};

export const withdrawTokens = async (
	ctx: VestingEndpoint,
): Promise<TransactionInstruction> => {
	const keys = unpackVestingKeys(ctx);

	return new TransactionInstruction({
		keys,
		programId,
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
	tx.add(..txInstructions);

	return await connection.sendTransaction(tx, signers, {
		preflightCommitment: 'single',
	});
};
