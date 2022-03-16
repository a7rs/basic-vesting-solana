use solana_program::{
    account_info::AccountInfo, declare_id, entrypoint, entrypoint::ProgramResult, msg,
    program_error::PrintProgramError, pubkey::Pubkey,
};

use crate::{error::ErrorCode, processor::Processor};

declare_id!("");
entrypoint!(vesting_entrypoint);

pub fn vesting_entrypoint(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    msg!("Entrypoint: Vesting");

    if let Err(e) = Processor::process(program_id, accounts, data) {
        e.print::<ErrorCode>();
        return Err(e);
    }
    Ok(())
}
