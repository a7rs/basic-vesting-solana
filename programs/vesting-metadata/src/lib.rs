use solana_program::{
    account_info::AccountInfo, entrypoint, decode_error::DecodeError, entrypoint::ProgramResult, msg,
    program_error::{PrintProgramError, ProgramError}, pubkey::Pubkey,
};

use num_derive::FromPrimitive;
use num_traits::FromPrimitive as FromPrimitiveTrait;
use thiserror::Error;

declare_id!("");
entrypoint!(entrypoint);

const PK_LEN: usize = 32;

const CREATE: u8 = 0;
const UPDATE: u8 = 1;
const DELETE: u8 = 2;

const IX_AUTH: usize = 0;
const IX_VAULT: usize = PK_LEN;
const IX_DURATION: usize = IX_POOL + PK_LEN;
const IX_APR: usize = IX_DURATION + 8;
const IX_WTL: usize = IX_APR + 8;
const IX_FEE: usize = IX_WTL + 8;

/// State Constants
const IS_INIT: usize = 0;
const AUTH: usize = 1;
const VAULT: usize = AUTH + PK_LEN;
const DURA: usize = VAULT + PK_LEN;
const APR: usize = DURA + 8;
const WTL: usize = APR + 8;
const FEE: usize = WTL + 8;
const LT: usize = FEE + 8;

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
    /// the amount of time required to elapse before rewards are fully realised
    duration: u64,
    /// the percentage of interest generated over 12 months
    apr: u64,
    ///
    withdrawal_timelock: u64,
    ///
    early_withdrawal_fee: u64,
    ///
    lifetime: u64,
}

impl IxCtx {
    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::new(size_of::<Self>());
        buf.extend_from_slice(self.authority.as_ref());
        buf.extend_from_slice(self.vault.as_ref());
        buf.extend_from_slice(self.duration.to_le_bytes());
        buf.extend_from_slice(self.apr.to_le_bytes());
        buf.extend_from_slice(self.withdrawal_timelock.to_le_bytes());
        buf.extend_from_slice(self.early_withdrawal_fee.to_le_bytes());
        buf.extend_from_slice(self.lifetime.to_le_bytes());
        buf
    }

    pub fn unpack(data: &[u8]) -> Result<Self, ProgramError> {
        let authority = data
            .get(IX_AUTH..PK_LEN)
            .and_then(|s| s.try_into().ok())
            .map(Pubkey::from_bytes)
            .ok_or(InvalidInstruction)?;

        let vault = data
            .get(IX_VAULT..IX_DURA)
            .and_then(|s| s.try_into().ok())
            .map(Pubkey::from_bytes)
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
        AccountMeta::new(ctx.tx_auth, true),
        AccountMeta::new(ctx.metadata, false),
    ];

    let data = MetadataInstruction::Create {
        authority: ctx.authority,
        vault: ctx.vault,
        duration: ctx.duration,
        apr: ctx.apr,
        withdrawal_timelock: ctx.withdrawal_timelock,
        early_withdrawal_fee: ctx.early_withdrawal_fee,
        lifetime: ctx.lifetime,
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
        duration: ctx.duration,
        apr: ctx.apr,
        withdrawal_timelock: ctx.withdrawal_timelock,
        early_withdrawal_fee: ctx.early_withdrawal_fee,
        lifetime: ctx.lifetime,
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
        duration: ctx.duration,
        apr: ctx.apr,
        withdrawal_timelock: ctx.withdrawal_timelock,
        early_withdrawal_fee: ctx.early_withdrawal_fee,
        lifetime: ctx.lifetime,
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
        duration: ctx.duration,
        apr: ctx.apr,
        withdrawal_timelock: ctx.withdrawal_timelock,
        early_withdrawal_fee: ctx.early_withdrawal_fee,
        lifetime: ctx.lifetime,
    }
    .pack();

    Ok(Instruction {
        program_id,
        accounts,
        data,
    })
}

pub struct MetadataState {
    pub is_initialized: bool,
    pub authority: Pubkey,
    pub vault: Pubkey,
    pub duration: u64,
    pub apr: u64,
    pub withdrawal_timelock: u64,
    pub early_withdrawal_fee: u64,
    pub lifetime: u64,
}

impl IsInitialized for MetadataState {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Sealed for MetadataState;

impl Pack for MetadataState {
    const LEN: usize = 111;

    fn pack_into_slice(&self, dst: &mut [u8]) {
        dst[IS_INIT] = self.is_initialized as u8;
        dst[AUTH..VAULT].copy_from_slice(self.authority.as_ref());
        dst[VAULT..DURA].copy_from_slice(self.vault.as_ref());
        dst[DURA..APR].copy_from_slice(self.duration.to_le_bytes());
        dst[APR..WTL].copy_from_slice(self.apr.to_le_bytes());
        dst[WTL..FEE].copy_from_slice(self.withdrawal_timelock.to_le_bytes());
        dst[FEE..LT].copy_from_slice(self.early_withdrawal_fee.to_le_bytes());
        dst[LT..].copy_from_slice(self.liftime.to_le_bytes());
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let is_initialized = match src[IS_INIT] {
            0 => false,
            1 => true,
            _ => unreachable!(),
        };

        let authority = Pubkey::new_from_array(src[AUTH..VAULT].try_into().unwrap());
        let vault = Pubkey::new_from_array(src[VAULT..DURA].try_into().unwrap());
        let duration = u64::from_le_bytes(src[DURA..APR].try_into().unwrap());
        let apr = u64::from_le_bytes(src[APR..WTL]).try_into().unwrap());
        let withdrawal_timelock = u64::from_le_bytes(src[WTL..FEE].try_into().unwrap());
        let early_withdrawal_fee = u64::from_le_bytes(src[FEE..LT].try_into().unwrap());
        let lifetime = u64::from_le_bytes(src[LT..].try_into().unwrap());

        Ok(Self {
            is_initialized,
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

pub struct Processor;

impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        data: &[u8],
    ) -> ProgramResult {
        let instruction = MetadataInstruction::unpack(data)?;

        match instruction {
            MetadataInstruction::Create(IxCtx) => Self::process_create(
                program_id,
                accounts,
                ix_ctx,
            ),
            MetadataInstruction::Update(IxCtx) => Self::process_update(
                program_id,
                accounts,
                ix_ctx,
            ),
            MetadataInstruction::Delete(IxCtx) => Self::process_delete(
                program_id,
                accounts,
                ix_ctx,
            ),
            _ => unreachable!(),
        }

        Ok(())
    }

    fn process_create(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        ix_ctx: IxCtx,
    ) -> Result<(), ProgramError> {
        let accounts_iter = &mut accounts.iter();

        let authority = next_account_info(accounts_iter)?;
        let metadata_account = next_account_info(accounts_iter)?;

        let metadata_data = metadata_account.data.borrow_mut();

        let metadata = MetadataState {
            is_initialized: true,
            authority: ix_ctx.authority,
            vault: ix_ctx.vault,
            pool: ix_ctx.pool,
            duration: ix_ctx.duration,
            apr: ix_ctx.apr,
            withdrawal_timelock: ix_ctx.withdrawal_timelock,
            early_withdrawal_fee: ix_ctx.early_withdrawal_fee,
            lifetime: ix_ctx.lifetime,
        }

        metadata.pack_into_slice(metadata_data);
        Ok(())
    }

    fn process_update(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        ix_ctx: IxCtx,
    ) -> Result<(), ProgramError> {
        let accounts_iter = &mut accounts.iter();

        let authority = next_account_info(accounts_iter)?;
        let metadata_account = next_account_info(accounts_iter)?;

        let metadata_data = metadata_account.data.borrow_mut();

        let metadata = MetadataState {
            is_initialized: true,
            authority: ix_ctx.authority,
            vault: ix_ctx.vault,
            duration: ix_ctx.duration,
            apr: ix_ctx.apr,
            withdrawal_timelock: ix_ctx.withdrawal_timelock,
            early_withdrawal_fee: ix_ctx.early_withdrawal_fee,
            lifetime: ix_ctx.lifetime,
        }

        metadata.pack_into_slice(metadata_data);
        Ok(())
    }

    fn process_delete(
        program_id:&Pubkey,
        accounts: &[AccountInfo],
        ix_ctx: IxCtx,
    ) -> Result<(), ProgramError> {
        let accounts_iter = &mut accounts.iter();

        let authority = next_account_info(accounts_iter)?;
        let metadata_account = next_account_info(accounts_iter)?;

        metadata_account.data.borrow_mut() = &[0u8; MetadataState::LEN];

        system_instruction::transfer(
            from_pubkey: metadata_account,
            to_pubkey: authority,
            lamports: metadata_account.lamports,
        )?;

        if metadata_account.lamports != 0 {
            return Err(ErrorCode::RemainingBalance);
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum ErrorCode {
    #[error("Invalid Instruction")]
    InvalidInstruction,
    #[error("Remaining Balance")]
    RemainingBalance,
}

impl From<ErrorCode> for ProgramError {
    fn from(e: ErrorCode) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for ErrorCode {
    fn type_of() -> &'static str {
        "Metadata Error"
    }
}

impl PrintProgramError for ErrorCode {
    fn print<E>(&self)
        where
            E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitiveTrait,
    {
        match self {
            ErrorCode::InvalidInstruction => msg!("Invalid Instruction."),
            ErrorCode::RemainingBalance => msg!("Account has remaining balance."),
        }
    }
}
