use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};

use num_derive::FromPrimitive;
use num_traits::FromPrimitive as FromPrimitiveTrait;
use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum ErrorCode {
    #[error("Invalid Instruction")]
    InvalidInstruction,
    #[error("Vesting end must be greater than the current unix timestamp.")]
    InvalidTimestamp,
    #[error("The number of vesting periods must be greater than zero.")]
    InvalidPeriod,
    #[error("The vesting deposit amount must be greater than zero.")]
    InvalidDepositAmount,
    #[error("Invalid program address. Did you provide the correct nonce?")]
    InvalidProgramAddress,
    #[error("Invalid vault owner.")]
    InvalidVaultOwner,
    #[error("Vault amount must be zero.")]
    InvalidVaultAmount,
    #[error("Insufficient withdrawal balance.")]
    InsufficientWithdrawalBalance,
    #[error("Whitelist is full")]
    WhitelistFull,
    #[error("Whitelist entry already exists")]
    WhitelistEntryAlreadyExists,
    #[error("You do not have sufficient permissions to perform this action.")]
    Unauthorized,
    #[error("You are unable to realize projected rewards until unstaking.")]
    UnableToWithdrawWhileStaked,
    #[error("You have not realized this vesting account.")]
    UnrealizedVesting,
    #[error("Invalid vesting schedule given.")]
    InvalidSchedule,
}

impl From<ErrorCode> for ProgramError {
    fn from(e: ErrorCode) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for ErrorCode {
    fn type_of() -> &'static str {
        "SCY Lockup Error"
    }
}

impl PrintProgramError for ErrorCode {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitiveTrait,
    {
        match self {
            ErrorCode::InvalidInstruction => msg!("Invalid Instruction"),
            ErrorCode::InvalidTimestamp => {
                msg!("Vesting end must be greater than the current unix timestamp.")
            }
            ErrorCode::InvalidPeriod => {
                msg!("The number of vesting periods must be greater than zero.")
            }
            ErrorCode::InvalidDepositAmount => {
                msg!("The vesting deposit amount must be greater than zero.")
            }
            ErrorCode::InvalidWhitelistEntry => {
                msg!("The Whitelist entry is not a valid program address".)
            }
            ErrorCode::InvalidProgramAddress => {
                msg!("Invalid program address. Did you provide the correct nonce?")
            }
            ErrorCode::InvalidVaultOwner => msg!("Invalid vault owner."),
            ErrorCode::InvalidVaultAmount => msg!("Invalid vault owner."),
            ErrorCode::InsufficientWithdrawalBalance => msg!("Insufficient withdrawal balance."),
            ErrorCode::Unauthorized => {
                msg!("You do not have sufficient permissions to perform this action.")
            }
            ErrorCode::UnableToWithdrawWhileStaked => {
                msg!("You are unable to realize project rewards until unstaking.")
            }
            ErrorCode::UnrealizedVesting => msg!("You have not realized this vesting account."),
            ErrorCode::InvalidSchedule => msg!("The provided vesting schedule is invalid."),
        }
    }
}
