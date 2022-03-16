use solana_program::{
    instruction::{AccountMeta, Instruction},
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use std::convert::TryInto;
use std::mem::size_of;

use crate::{error::ErrorCode::InvalidInstruction, state::PK_LEN};

const CREATE: u8 = 0;
const UPDATE: u8 = 1;
const DELETE: u8 = 2;

const IX_AUTH: usize = 0;
const IX_VAULT: usize = PK_LEN;
const IX_DURA: usize = IX_VAULT + PK_LEN;
const IX_APR: usize = IX_DURA + 8;
const IX_WTL: usize = IX_APR + 8;
const IX_FEE: usize = IX_WTL + 8;
const IX_LIFE: usize = IX_FEE + 8;

pub struct IxCtx {
    /// Authority must match the authority for the provided pool and vault
    pub authority: Pubkey,
    /// The token account for staked tokens
    pub vault: Pubkey,
    /// the amount of time required to elapse before rewards are fully realised
    pub duration: u64,
    /// the percentage of interest generated over 12 months
    pub apr: u64,
    ///
    pub withdrawal_timelock: u64,
    ///
    pub early_withdrawal_fee: u64,
    ///
    pub lifetime: u64,
}

impl IxCtx {
    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        buf.extend_from_slice(self.authority.as_ref());
        buf.extend_from_slice(self.vault.as_ref());
        buf.extend_from_slice(&self.duration.to_le_bytes());
        buf.extend_from_slice(&self.apr.to_le_bytes());
        buf.extend_from_slice(&self.withdrawal_timelock.to_le_bytes());
        buf.extend_from_slice(&self.early_withdrawal_fee.to_le_bytes());
        buf.extend_from_slice(&self.lifetime.to_le_bytes());
        buf
    }

    pub fn unpack(data: &[u8]) -> Result<Self, ProgramError> {
        let authority = data
            .get(IX_AUTH..PK_LEN)
            .and_then(|s| s.try_into().ok())
            .map(Pubkey::new_from_array)
            .ok_or(InvalidInstruction)?;

        let vault = data
            .get(IX_VAULT..IX_DURA)
            .and_then(|s| s.try_into().ok())
            .map(Pubkey::new_from_array)
            .ok_or(InvalidInstruction)?;

        let duration = data
            .get(IX_DURA..IX_APR)
            .and_then(|s| s.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(InvalidInstruction)?;

        let apr = data
            .get(IX_APR..IX_WTL)
            .and_then(|s| s.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(InvalidInstruction)?;

        let withdrawal_timelock = data
            .get(IX_WTL..IX_FEE)
            .and_then(|s| s.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(InvalidInstruction)?;

        let early_withdrawal_fee = data
            .get(IX_FEE..IX_LIFE)
            .and_then(|s| s.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(InvalidInstruction)?;

        let lifetime = data
            .get(IX_LIFE..)
            .and_then(|s| s.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(InvalidInstruction)?;

        Ok(Self {
            authority,
            vault,
            duration,
            apr,
            withdrawal_timelock,
            early_withdrawal_fee,
            lifetime,
        })
    }
}

pub enum MetadataInstruction {
    /// Accounts expected for all instructions:
    /// `[s]` Authority
    /// `[w]` Metadata account
    Create(IxCtx),
    Update(IxCtx),
    Delete(IxCtx),
}

impl MetadataInstruction {
    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());

        match &self {
            Self::Create(ixctx) => {
                buf.push(CREATE);
                buf.extend_from_slice(&ixctx.pack());
            }
            Self::Update(ixctx) => {
                buf.push(UPDATE);
                buf.extend_from_slice(&ixctx.pack());
            }
            Self::Delete(ixctx) => {
                buf.push(DELETE);
                buf.extend_from_slice(&ixctx.pack());
            }
            _ => unreachable!(),
        }

        buf
    }

    pub fn unpack(data: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = data.split_first().ok_or(InvalidInstruction)?;

        Ok(match *tag {
            CREATE => Self::Create(IxCtx::unpack(&rest)?),
            UPDATE => Self::Update(IxCtx::unpack(&rest)?),
            DELETE => Self::Delete(IxCtx::unpack(&rest)?),
            _ => unreachable!(),
        })
    }
}

pub struct EndpointCtx<'a> {
    program_id: &'a Pubkey,
    tx_auth: &'a Pubkey,
    metadata: &'a Pubkey,
    authority: &'a Pubkey,
    vault: &'a Pubkey,
    duration: u64,
    apr: u64,
    withdrawal_timelock: u64,
    early_withdrawal_fee: u64,
    lifetime: u64,
}

/// Endpoints
pub fn create(ctx: EndpointCtx) -> Result<Instruction, ProgramError> {
    msg!("Vesting Metadata: Create");

    let accounts = vec![
        AccountMeta::new(*ctx.tx_auth, true),
        AccountMeta::new(*ctx.metadata, false),
    ];

    let data = MetadataInstruction::Create(IxCtx {
        authority: *ctx.authority,
        vault: *ctx.vault,
        duration: ctx.duration,
        apr: ctx.apr,
        withdrawal_timelock: ctx.withdrawal_timelock,
        early_withdrawal_fee: ctx.early_withdrawal_fee,
        lifetime: ctx.lifetime,
    })
    .pack();

    Ok(Instruction {
        program_id: *ctx.program_id,
        accounts,
        data,
    })
}

pub fn update(ctx: EndpointCtx) -> Result<Instruction, ProgramError> {
    msg!("Vesting Metadata: Update");

    let accounts = vec![
        AccountMeta::new(*ctx.tx_auth, true),
        AccountMeta::new(*ctx.metadata, false),
    ];

    let data = MetadataInstruction::Update(IxCtx {
        authority: *ctx.authority,
        vault: *ctx.vault,
        duration: ctx.duration,
        apr: ctx.apr,
        withdrawal_timelock: ctx.withdrawal_timelock,
        early_withdrawal_fee: ctx.early_withdrawal_fee,
        lifetime: ctx.lifetime,
    })
    .pack();

    Ok(Instruction {
        program_id: *ctx.program_id,
        accounts,
        data,
    })
}

pub fn delete(ctx: EndpointCtx) -> Result<Instruction, ProgramError> {
    msg!("Vesting Metadata: Delete");

    let accounts = vec![
        AccountMeta::new(*ctx.tx_auth, true),
        AccountMeta::new(*ctx.metadata, false),
    ];

    let data = MetadataInstruction::Delete(IxCtx {
        authority: *ctx.authority,
        vault: *ctx.vault,
        duration: ctx.duration,
        apr: ctx.apr,
        withdrawal_timelock: ctx.withdrawal_timelock,
        early_withdrawal_fee: ctx.early_withdrawal_fee,
        lifetime: ctx.lifetime,
    })
    .pack();

    Ok(Instruction {
        program_id: *ctx.program_id,
        accounts,
        data,
    })
}
