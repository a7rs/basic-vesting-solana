use solana_program::{
    instruction::{AccountMeta, Instruction},
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{error::ErrorCode::InvalidInstruction, state::PK_LEN};

use std::convert::TryInto;
use std::mem::size_of;

const IX_INIT: u8 = 0;
const IX_CREATE: u8 = 1;
const IX_WITHDRAW: u8 = 2;
const IX_SETBENEFICIARY: u8 = 3;

const S_TS: usize = BENEFICIARY + PK_LEN;
const E_TS: usize = S_TS + 8;
const N: usize = E_TS + 8;
const NONCE: usize = N + 8;
const AMOUNT: usize = NONCE + 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VestingInstruction {
    /// Convenience function for creating vesting account
    ///
    /// Accounts expected:
    /// `[s,w]` Authority
    /// `[w]` Vesting Account
    /// `[]` System Program
    Init,

    /// Accounts expected:
    ///
    /// `[s,w]` Authority
    /// `[w]` Token Account
    /// `[w]` Vesting Account
    /// `[w]` Vault
    /// `[]` Metadata Account
    /// `[]` Token Program
    CreateVesting {
        beneficiary: Pubkey,
        start_ts: u64,
        end_ts: u64,
        period_count: u64,
        nonce: u8,
        amount: u64,
    },

    /// Accounts expected:
    ///
    /// `[s,w]` Authority
    /// `[w]` Token Account
    /// `[w]` Vesting Account
    /// `[w]` Vault
    /// `[]` Metadata Account
    /// `[]` Token Program
    Withdraw { amount: u64 },

    /// Accounts Expected:
    ///
    /// `[w,s]` Authority
    /// `[w]` New Beneficiary Authority
    /// `[w]` New Beneficiary Vesting Account
    /// `[w]` New Beneficiary Token Account
    /// `[]` Token Program
    SetBeneficiary { new_beneficiary: Pubkey },
}

impl VestingInstruction {
    fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());

        match self {
            Self::Init => buf.push(IX_INIT),
            Self::CreateVesting {
                beneficiary,
                start_ts,
                end_ts,
                period_count,
                nonce,
                amount,
            } => {
                buf.push(IX_CREATE);
                buf.extend_from_slice(beneficiary.as_ref());
                buf.extend_from_slice(&start_ts.to_le_bytes());
                buf.extend_from_slice(&end_ts.to_le_bytes());
                buf.extend_from_slice(&period_count.to_le_bytes());
                buf.extend_from_slice(&nonce.to_le_bytes());
                buf.extend_from_slice(&amount.to_le_bytes());
            }
            Self::Withdraw { amount } => {
                buf.push(IX_WITHDRAW);
                buf.extend_from_slice(&amount.to_le_bytes());
            }
            Self::SetBeneficiary {
                new_beneficiary: Pubkey,
            } => {
                buf.push(IX_SETBENEFICIARY);
                buf.extend_from_slice(new_beneficiary.as_ref());
            }
        }
        buf
    }

    fn unpack(data: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = data.split_first().ok_or(InvalidInstruction)?;
        Ok(match tag {
            IX_INIT => Self::Init,
            IX_CREATE => {
                let beneficiary = rest
                    .get(..S_TS)
                    .and_then(|s| s.try_into().ok())
                    .map(Pubkey::new_from_array)
                    .ok_or(InvalidInstruction)?;
                let start_ts = rest
                    .get(S_TS..E_TS)
                    .and_then(|s| s.try_into().ok())
                    .map(u64::from_le_bytes)
                    .ok_or(InvalidInstruction)?;
                let end_ts = rest
                    .get(E_TS..N)
                    .and_then(|s| s.try_into().ok())
                    .map(u64::from_le_bytes)
                    .ok_or(InvalidInstruction)?;
                let period_count = rest
                    .get(N..NONCE)
                    .and_then(|s| s.try_into().ok())
                    .map(u64::from_le_bytes)
                    .ok_or(InvalidInstruction)?;
                let nonce = rest
                    .get(NONCE..AMOUNT)
                    .and_then(|s| s.try_into().ok())
                    .map(u8::from_le_bytes)
                    .ok_or(InvalidInstruction)?;
                let amount = rest
                    .get(AMOUNT..)
                    .and_then(|s| s.try_into().ok())
                    .map(u64::from_le_bytes)
                    .ok_or(InvalidInstruction)?;
                Self::CreateVesting {
                    beneficiary,
                    start_ts,
                    end_ts,
                    period_count,
                    nonce,
                    amount,
                }
            }
            IX_WITHDRAW => {
                let amount = rest
                    .get(..)
                    .and_then(|s| s.try_into().ok())
                    .map(u64::from_le_bytes)
                    .ok_or(InvalidInstruction)?;
                Self::Withdraw { amount }
            }
            IX_SETBENEFICIARY => {
                let new_beneficiary = rest
                    .get(..)
                    .and_then(|s| s.try_into().ok())
                    .map(Pubkey::new_from_array)
                    .ok_or(InvalidInstruction)?;
                Self::SetBeneficiary { new_beneficiary }
            }
            _ => return Err(ProgramError::InvalidArgument),
        })
    }
}

pub fn init(
    program_id: &Pubkey,
    payer: &Pubkey,
    vesting: &Pubkey,
    system_program: &Pubkey,
) -> Result<Instruction, ProgramError> {
    msg!("Vesting: Init");

    let accounts = vec![
        AccountMeta::new(*payer, true),
        AccountMeta::new(*vesting, false),
        AccountMeta::new_readonly(*system_program, false),
    ];

    let data = VestingIntruction::Init.pack();

    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}

pub fn create_vesting(
    program_id: &Pubkey,
    vesting: &Pubkey,
    vault: &Pubkey,
    authority: &Pubkey,
    token_account: &Pubkey,
    metadata: &Pubkey,
    token_program: &Pubkey,
    start_ts: u64,
    end_ts: u64,
    period_count: u64,
    nonce: u8,
    amount: u64,
) -> Result<Instruction, ProgramErro> {
    msg!("Vesting: Create");

    let accounts = vec![
        AccountMeta::new(*vesting, false),
        AccountMeta::new(*vault, false),
        AccountMeta::new(*authority, true),
        AccountMeta::new(*token_account, false),
        AccountMeta::new_readonly(*metadata, false),
        AccountMeta::new_readonly(*token_program, false),
    ];

    let data = VestingInstruction::CreateVesting {
        beneficiary,
        start_ts,
        end_ts,       // should be calculated utlizing metadata
        period_count, // should pull from metadata
        nonce,
        amount,
    }
    .pack();

    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}

pub fn withdraw(
    program_id: &Pubkey,
    vesting: &Pubkey,
    vault: &Pubkey,
    authority: &Pubkey,
    token_account: &Pubkey,
    metadata: &Pubkey,
    token_program: &Pubkey,
    amount: u64,
) -> Result<Instruction, ProgramError> {
    msg!("Vesting: Withdraw");

    let accounts = vec![
        AccountMeta::new(*vesting, false),
        AccountMeta::new(*vault, false),
        AccountMeta::new(*authority, true),
        AccountMeta::new(*token_account, false),
        AccountMeta::new_readonly(*metadata, false),
        AccountMeta::new_readonly(*token_program, false),
    ];

    let data = VestingInstruction::Withdraw { amount: u64 }.pack();

    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}

pub fn set_beneficiary(
    program_id: &Pubkey,
    authority: &Pubkey,
    new_beneficiary: &Pubkey,
    new_beneficiary_vesting_address: &Pubkey,
    new_beneficiary_token_address: &Pubkey,
    token_program: &Pubkey,
) -> Result<Instruction, ProgramError> {
    msg!("Vesting: Set Beneficiary");

    let accounts = vec![
        AccountMeta::new(*authority, true),
        AccountMeta::new(*new_beneficiary, false),
        AccountMeta::new(*new_beneficiary_vesting_address, false),
        AccountMeta::new(*new_beneficiary_token_address, false),
        AccountMeta::new_readonly(*token_program, false),
    ];

    let data = VestingInstruction::SetBeneficiary {
        new_beneficiary: *new_beneficiary,
    }
    .pack();

    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}
