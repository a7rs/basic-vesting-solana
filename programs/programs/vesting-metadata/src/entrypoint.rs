use solana_program::{
    account_info::AccountInfo, declare_id, entrypoint, entrypoint::ProgramResult, msg,
    program_error::PrintProgramError, pubkey::Pubkey,
};

use crate::{error::ErrorCode, processor::Processor};

declare_id!("SCYGyVRR45ytWfuQGJXkY1RtkXTX1GDA6SaxuyW5ZKG");
entrypoint!(metadata_entrypoint);

#[cfg(not(feature = "no-entrypoint"))]
pub fn metadata_entrypoint(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    msg!("Entrypoint: Vesting Metadata");

    if let Err(e) = Processor::process(program_id, accounts, data) {
        e.print::<ErrorCode>();
        return Err(e);
    }

    Ok(())
}
