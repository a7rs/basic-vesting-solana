use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, msg,
    program_error::PrintProgramError, pubkey::Pubkey,
};

use crate::{error::ErrorCode, processor::Processor::process};

declare_id!("");
entrypoint!(entrypoint);

const PK_LEN: usize = 32;

const CREATE: u8 = 0;
const READ: u8 = 1;
const UPDATE: u8 = 2;
const DELETE: u8 = 3;

const IX_AUTH: usize = 0;
const IX_VAULT: usize = PK_LEN;
const IX_POOL: usize = IX_VAULT + PK_LEN;
const IX_DURATION: usize = IX_POOL + PK_LEN;
const IX_APR: usize = IX_DURATION + 8;

pub fn entrypoint(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    msg!("Entrypoint: Vesting Metadata");

    if let Err(e) = process(program_id, accounts, data) {
        error.print::<ErrorCode>();
        return Err(e);
    }
    Ok(())
}

pub struct IxCtx {
    /// Authority must match the authority for the provided pool and vault
    authority: Pubkey,
    /// The token account for staked tokens
    vault: Pubkey,
    /// Reward token account
    pool: Pubkey,
    /// the amount of time required to elapse before rewards are fully realised
    duration: u64,
    /// the percentage of interest generated over 12 months
    apr: u64,
}

impl IxCtx {
    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::new(size_of::<Self>());
        buf.extend_from_slice(self.authority.as_ref());
        buf.extend_from_slice(self.vault.as_ref());
        buf.extend_from_slice(self.pool.as_ref());
        buf.extend_from_slice(self.duration.to_le_bytes());
        buf.extend_from_slice(self.apr.to_le_bytes());
        buf
    }

    pub fn unpack(data: &[u8]) -> Result<Self, ProgramError> {
        let authority = data
            .get(IX_AUTH..PK_LEN)
            .and_then(|s| s.try_into().ok())
            .map(Pubkey::from_bytes)
            .ok_or(InvalidInstruction)?;

        let vault = data
            .get(IX_VAULT..IX_POOL)
            .and_then(|s| s.try_into().ok())
            .map(Pubkey::from_bytes)
            .ok_or(InvalidInstruction)?;

        let pool = data
            .get(IX_POOL..IX_DURA)
            .and_then(|s| s.try_into().ok())
            .map(Pubkey::from_bytes)
            .ok_or(InvalidInstruction)?;

        let duration = data
            .get(IX_DURA..IX_APR)
            .and_then(|s| s.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(InvalidInstruction)?;

        let apr = data
            .get(IX_APR..)
            .and_then(|s| s.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(InvalidInstruction)?;

        Ok(Self {
            authority,
            vault,
            pool,
            duration,
            apr,
        })
    }
}

pub enum MetadataInstruction {
    /// Accounts expected for all instructions:
    /// `[s]` Authority
    /// `[w]` Metadata account
    Create(IxCtx),
    Read(IxCtx),
    Update(IxCtx),
    Delete(IxCtx),
}

impl MetadataInstruction {
    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::new(size_of::<Self>());

        match &self {
            Create(ixctx) => {
                buf.push(CREATE);
                buf.extend_from_slice(ixctx.pack());
            }
            Read(ixctx) => {
                buf.push(READ);
                buf.extend_from_slice(ixctx.pack());
            }
            Update(ixctx) => {
                buf.push(UPDATE);
                buf.extend_from_slice(ixctx.pack());
            }
            Delete(ixctx) => {
                buf.push(DELETE);
                buf.extend_from_slice(ixctx.pack());
            }
            _ => unreachable!(),
        }

        buf
    }

    pub fn unpack(data: &[u8]) -> Self {
        let (tag, rest) = data.split_first().ok_or(InvalidInstruction)?;

        match tag {
            CREATE => Self::Create(IxCtx::unpack(&rest)?),
            READ => Self::Read(IxCtx::unpack(&rest)?),
            UPDATE => Self::Update(IxCtx::unpack(&rest)?),
            DELETE => Self::Delete(IxCtx::unpack(&rest)?),
            _ => unreachable!(),
        }
    }
}

pub struct EndpointCtx {
    program_id: &Pubkey,
    tx_auth: &Pubkey,
    metadata: &Pubkey,
    authority: &Pubkey,
    vault: &Pubkey,
    pool: &Pubkey,
    duration: u64,
    apr: u64,
}

/// Endpoints
pub fn create(ctx: EndpointCtx) -> Result<Instruction, ProgramError> {
    msg!("Vesting Metadata: Create");

    let accounts = vec![
        AccountMeta::new(ctx.tx_auth, true),
        AccountMeta::new(ctx.metadata, false),
    ];

    let data = MetadataInstruction::Create {
        authority: ctx.authority,
        vault: ctx.vault,
        pool: ctx.pool,
        duration: ctx.duration,
        apr: ctx.apr,
    }
    .pack();

    Ok(Instruction {
        program_id,
        accounts,
        data,
    })
}

pub fn read(ctx: EndpointCtx) -> Result<Instruction, ProgramError> {
    msg!("Vesting Metadata: Read");
    let accounts = vec![
        AccountMeta::new(ctx.tx_auth, true),
        AccountMeta::new(ctx.metadata, false),
    ];

    let data = MetadataInstruction::Read {
        authority: ctx.authority,
        vault: ctx.vault,
        pool: ctx.pool,
        duration: ctx.duration,
        apr: ctx.apr,
    }
    .pack();

    Ok(Instruction {
        program_id,
        accounts,
        data,
    })
}

pub fn update(ctx: EndpointCtx) -> Result<Instruction, ProgramError> {
    msg!("Vesting Metadata: Update");

    let accounts = vec![
        AccountMeta::new(ctx.tx_auth, true),
        AccountMeta::new(ctx.metadata, false),
    ];

    let data = MetadataInstruction::Update {
        authority: ctx.authority,
        vault: ctx.vault,
        pool: ctx.pool,
        duration: ctx.duration,
        apr: ctx.apr,
    }
    .pack();

    Ok(Instruction {
        program_id,
        accounts,
        data,
    })
}

pub fn delete(ctx: EndpointCtx) -> Result<Instruction, ProgramError> {
    msg!("Vesting Metadata: Delete");

    let accounts = vec![
        AccountMeta::new(ctx.tx_auth, true),
        AccountMeta::new(ctx.metadata, false),
    ];

    let data = MetadataInstruction::Read {
        authority: ctx.authority,
        vault: ctx.vault,
        pool: ctx.pool,
        duration: ctx.duration,
        apr: ctx.apr,
    }
    .pack();

    Ok(Instruction {
        program_id,
        accounts,
        data,
    })
}
