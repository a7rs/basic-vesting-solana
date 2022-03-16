use solana_program::{
    account_info::AccountInfo, declare_id, entrypoint, entrypoint::ProgramResult, msg,
    program_error::PrintProgramError, pubkey::Pubkey,
};

use crate::{error::ErrorCode, processor::Processor::process};

declare_id!("");
entrypoint!(entrypoint);

pub fn entrypoint(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    msg!("Entrypoint: Vesting Metadata");

    if let Err(e) = process(program_id, accounts, data) {
        e.print::<ErrorCode>();
        return Err(e);
    }

    Ok(())
}
