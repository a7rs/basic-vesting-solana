use solana_program::{
    instruction::{AccountMeta, Instruction},
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::error::ErrorCode::InvalidInstruction;
use std::{convert::TryInto, mem::size_of};

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
    /// `[w]` Vesting Account
    /// `[w]` Vesting Token Vault
    /// `[w]` SCY Staking Token Vault
    /// `[]` Metadata Account
    /// `[]` Token Program
    CreateVesting {
        amount: u64,
    },

    /// Accounts expected:
    ///
    /// `[s,w]` Authority
    /// `[w]` Vesting Account
    /// `[w]` Vesting Token Vault
    /// `[w]` SCY Staking Token Vault
    /// `[]` Metadata Account
    /// `[]` Token Program
    Withdraw {
        amount: u64,
    }

    /// Accounts Expected:
    ///
    /// `[w,s]` Authority
    ChangeBeneficiary {
        new_beneficiary: Pubkey,
    },

    /// Accounts expected:
    ///
    /// `[s,w]` Authority
    ChangeAuthority {
        new_authority: Pubkey,
    },
}

impl LockupInstruction {
    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match self {
            Self::New => {
                buf.push(0);
            },
            Self::WhitelistAdd => {
                buf.push(1);
                buf.extend_from_slice(entry::Pubkey);
            },
            Self::WhitelistDelete => {
                buf.push(2);
                buf.extend_from_slice(entry::Pubkey);
            },
            Self::WhitelistAuthority => {
                buf.push(3);
                buf.extend_from_slice(entry::Pubkey);
            },
        }
        buf
    }
     
    /// Convenience function for unpacking entries
    fn unpack_entry(data: &[u8]) -> Result<Pubkey, ProgramError> {
        Ok(data.get(..32)
            .and_then(|slice| slice.try_into().ok())
            .map(Pubkey::from_bytes)
            .ok_or(InvalidInstruction)?)
    }

    pub fn unpack(data: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = data.split_first().ok_or(InvalidInstruction)?;
        Ok(match tag {
            0 => Self::New;
            1 => Self::WhitelistAdd { unpack_entry(rest)? }
            2 => Self::WhitelistDelete { unpack_entry(rest)? }
            3 => Self::WhitelistAuthority { unpack_entry(rest)? }
            _ => return Err(ProgramError::InvalidArgument)
        })
    }
}

impl VestingInstruction {
    fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());

        match self {
            Self::CreateVesting {
                beneficiary,
                deposit_amount,
                nonce,
                start_ts,
                end_ts,
                period_count,
                //realizor<Realizor>,
            } => {
                buf.push(0);
                buf.extend_from_slice(beneficiary.to_bytes());
                buf.extend_from_slice(deposit_amount.to_le_bytes());
                buf.extend_from_slice(nonce.to_le_bytes());
                buf.extend_from_slice(period_count.to_le_bytes());
                //buf.extend_from_slice(realizor.____());
            Self::Withdraw { amount } => {
                buf.push(1);
                buf.extend_from_slice(amount.to_le_bytes());
            },
            Self::WhitelistWithdraw { amount } => {
                buf.push(2);
                buf.extend_from_slice(amount.to_le_bytes());
            },
            Self::WhitelistDeposit { amount } => {
                buf.push(3);
                buf.extend_from_slice(amount.to_le_bytes());
            }
            Self::AvailableForWithdrawal => buf.push(4);
            }
        }
        buf
    }

    /// Convenience function for unpacking u64 amount at the start of a buffer
    fn unpack_amount(data: &[u8]) -> Result<u64, ProgramError> {
        Ok(data.get(..8)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(InvalidInstruction)?)
    }
        
    pub fn unpack(data: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = data.split_first().ok_or(InvalidInstruction)?;
        Ok(match tag {
            0 => {
                let beneficiary = rest.get(..32)
                    .and_then(|slice| slice.try_into().ok())
                    .map(Pubkey::from_bytes)
                    .ok_or(InvalidInstruction)?;
                let deposit_amount = rest.get(32..40)
                    .and_then(|slice| slice.try_into().ok())
                    .map(u64::from_le_bytes)
                    .ok_or(InvalidInstruction)?;
                let nonce = rest.get(41)
                    .and_then(|slice| slice.try_into().ok())
                    .map(u8::from_le_bytes)
                    .ok_or(InvalidInstruction)?;
                let start_ts = rest.get(41..49)
                    .and_then(|slice| slice.try_into().ok())
                    .map(u64::from_le_bytes)
                    .ok_or(InvalidInstruction)?;
                let end_ts = rest.get(49..57)
                    .and_then(|slice| slice.try_into().ok())
                    .map(u64::from_le_bytes)
                    .ok_or(InvalidInstruction)?;
                let period_count = rest.get(57..65)
                    .and_then(|slice| slice.try_into().ok())
                    .map(u64::from_le_bytes)
                    .ok_or(InvalidInstruction)?;
                // let realizor =
                Self::CreateVesting {
                    beneficiary,
                    deposit_amount,
                    nonce,
                    start_ts,
                    end_ts,
                    period_count,
                    //realizor
                }
            },
            1 => Self::Withdraw { amount: unpack_amount(rest)? },
            2 => Self::WhitelistWithdraw { amount: unpack_amount(rest)? },
            3 => Self::WhitelistDeposit { amount: unpack_amount(rest)? },
            4 => Self::AvailableForWithdrawal,
            _ => return Err(ProgramError::InvalidArgument),
        })
    }
}

/// RPC APIs
pub fn create_vesting(
    program_id: Pubkey,
    vesting: Pubkey,
    vault: Pubkey,
    depositor: Pubkey,
    depositor_authority: Pubkey,
    token_program: Pubkey,
    rent: Pubkey,
    clock: Pubkey,
    beneficiary: Pubkey, //do not pass as account
    deposit_amount: u64,
    nonce: u8,
    start_ts: u64,
    end_ts: u64,
    period_count: u64,
) -> Result<Instruction, ProgramError> {
    msg!("SCY Lockup Call: CreateVesting");
    let accounts = vec![
        AccountMeta::new(vesting, false),
        AccountMeta::new(vault, false),
        AccountMeta::new(depositor, false),
        AccountMeta::new(depositor_authority, true),
        AccountMeta::new_readonly(token_program, false),
        AccountMeta::new_readonly(rent, false),
        AccountMeta::new_readonly(clock, false),
    ];

    let data = VestingInstruction::CreateVesting {
        beneficiary,
        deposit_amount,
        nonce,
        start_ts,
        end_ts,
        period_count,
    }.pack();

    Ok(Instruction {
        program_id,
        accounts,
        data,
    })
}

pub fn withdraw(
    program_id: Pubkey,
    vesting: Pubkey,
    beneficiary: Pubkey,
    vault: Pubkey,
    vesting_signer: Pubkey,
    receiving_token_account: Pubkey,
    token_program: Pubkey,
    clock: Pubkey,
    amount: u64,
) -> Result<Instruction, ProgramError> {
    msg!("SCY Lockup Call: Withdraw");
    let accounts = vec![
        AccountMeta::new(vesting, false),
        AccountMeta::new(beneficiary, false),
        AccountMeta::new(vault, false),
        AccountMeta::new(vesting_signer, true),
        AccountMeta::new(receiving_token_account, false),
        AccountMeta::new_readonly(token_program, false),
        AccountMeta::new_readonly(clock, false),
    ];

    let data = VestingInstruction::Withdraw { amount }.pack();
    Ok(Instruction {
        program_id,
        accounts,
        data,
    })
}
   

pub fn available_for_withdrawal(
    program_id: Pubkey,
    vesting: Pubkey,
    clock: Pubkey,
) -> Result<Instruction, ProgramError> {
    msg!("SCY Lockup Call: AvailableForWithdrawal");
    let accounts = vec![
        AccountMeta::new_readonly(vesting, false),
        AccountMeta::new_readonly(clock, false),
    ];
    let data = VestingInstruction::AvailableForWithdrawal.pack();

    Ok(Instruction {
        program_id,
        accounts,
        data,
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use solana_program::{instruction::AccountMeta, pubkey::Pubkey};

    #[test]
    fn test_pack_unpack_vesting_instructions() {
        let create = VestingInstruction::CreateVesting {
            beneficiary: Pubkey::new_unique(),
            deposit_amount: 69_420_000,
            nonce: 69,
            start_ts: 69_696_696,
            end_ts: 420_420_420,
            period_count: 420,
        };
        let withdraw = VestingInstruction::Withdraw { amount: 420 };
        let whitelist_withdraw = VestingInstruction::WhitelistWithdraw { amount };
        let whitelist_deposit = VestingInstruction::WhitelistDeposit { amount };
        let available = VestingInstruction::Available;
        
        let packed_create = create.pack();
        let packed_withdraw = withdraw.pack();
        let packed_whitelist_withdraw = whitelist_withdraw.pack();
        let packed_whitelist_deposit = whitelist_deposit.pack();
        let packed_available = available.pack();

        let unpacked_create = VestingInstruction::unpack(&packed_create).unwrap();
        let unpacked_withdraw = VestingInstruction::unpack(&packed_withdraw).unwrap();
        let unpacked_whitelist_withdraw = VestingInstruction::unpack(&packed_whitelist_withdraw).unwrap();
        let unpacked_whitelist_deposit = VestingInstruction::unpack(&packed_whitelist_deposit).unwrap();
        let unpacked_available = VestingInstruction::unpack(&packed_available).unwrap();

        assert_eq!(create, unpacked_create);
        assert_eq!(withdraw, unpacked_withdraw);
        assert_eq!(whitelist_withdraw, unpacked_whitelist_withdraw);
        assert_eq!(whitelist_deposit, unpacked_whitelist_deposit);
        assert_eq!(available, unpacked_available);
    }

    #[test]
    fn test_create_api() {
        let program_id = crate::entrypoint::id();
        let vesting = Pubkey::new_unique();
        let vault = Pubkey::new_unique();
        let depositor = Pubkey::new_unique();
        let depositor_authority = Pubkey::new_unique();
        let token_program = Pubkey::new_unique();
        let rent = Pubkey::new_unique();
        let clock = Pubkey::new_unique();
        let beneficiary = Pubkey::new_unique();
        let deposit_amount = 69_420_000;
        let nonce = 42;
        let start_ts = 69_000_000;
        let end_ts = 420_000_000;
        let period_count = 690;

        let create = crate::instruction::create_vesting(
            program_id,
            vesting,
            vault,
            depositor,
            depositor_authority,
            token_program,
            rent,
            clock,
            beneficiary,
            deposit_amount,
            nonce,
            start_ts,
            end_ts,
            period_count,
        ).unwrap();

        let accounts = vec![
            AccountMeta::new(vesting, false),
            AccountMeta::new(vault, false),
            AccountMeta::new(depositor, false),
            AccountMeta::new(depositor_authority, true),
            AccountMeta::new_readonly(token_program, false),
            AccountMeta::new_readonly(rent, false),
            AccountMeta::new_readonly(clock, false),
        ];

        let data = VestingInstruction::CreateVesting {
            beneficiary,
            deposit_amount,
            nonce,
            start_ts,
            end_ts,
            period_count,
        }.pack();

        assert_eq!(create.accounts, accounts);
        assert_eq!(create.data, data);
    }

    #[test]
    fn test_withdraw_api() {
        let program_id = crate::entrypoint::id();
        let vesting = Pubkey::new_unique();
        let beneficiary = Pubkey::new_unique();
        let vault = Pubkey::new_unique();
        let vesting_signer = Pubkey::new_unique();
        let receiving_token_account = Pubkey::new_unique();
        let token_program = Pubkey::new_unique();
        let clock = Pubkey::new_unique();
        let amount = u64;
        
        let withdraw = crate::instruction::withdraw(
            program_id,
            vesting,
            beneficiary,
            vault,
            vesting_signer,
            receiving_token_account,
            token_program,
            clock,
            amount,
        ).unwrap();

        let data = VestingInstruction::Withdraw { amount }.pack();

        assert_eq!(withdraw.accounts, accounts);
        assert_eq!(withdraw.data, data);
    }

    #[test]
    fn test_whitelist_withdraw_api() {
        let program_id = crate::entrypoint::id();
        let vesting = Pubkey::new_unique();
        let vault = Pubkey::new_unique();
        let vesting_signer = Pubkey::new_unique();
        let token_program = Pubkey::new_unique();
        let whitelisted_program_vault = Pubkey::new_unique();
        let whitelisted_program_vault_authority = Pubkey::new_unique();
        let amount = 420;

        let ww = crate::instruction::whitelist_withdraw(
            program_id,
            vesting,
            vault,
            vesting_signer,
            token_program,
            whitelisted_program_vault,
            whitelisted_program_vault_authority,
        ).unwrap();
        
        let accounts = vec![
            AccountMeta::new(vesting, false),
            AccountMeta::new(vault, false),
            AccountMeta::new(vesting_signer, true),
            AccountMeta::new_readonly(token_program, false),
            AccountMeta::new(whitelisted_program_vault, false),
            AccountMeta::new(whitelisted_program_vault_authority, true),
        ];

        let data = VestingInstruction::WhitelistWithdraw { amount }.pack();

        assert_eq!(ww.accounts, accounts);
        assert_eq!(ww.data, data);
    }

    #[test]
    fn available_for_withdrawal() {
        let program_id = crate::entrypoint::id();
        let vesting = Pubkey::new_unique();
        let clock = Pubkey::new_unique();

        let available = crate::instruction::available_for_withdrawal(
            program_id,
            vesting,
            clock,
        ).unwrap();

        let accounts = vec![
            AccountMeta::new_readonly(vesting, false),
            AccountMeta::new_readonly(clock, false),
        ];

        let data = VestingInstruction::AvailableForWithdrawal.pack();

        assert_eq!(available.accounts, accounts);
        assert_eq!(available.data, data);
    }
}
