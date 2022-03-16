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
