use solana_program::{
        account_info::{next_account_info, AccountInfo},
        declare_id, 
        entrypoint, 
        entrypoint::ProgramResult, 
        instruction::{AccountMeta, Instruction},
        msg, 
        program_error::PrintProgramError,
        pubkey::Pubkey,
};

thiserror::Error,

declare_id!("");
entrypoint!(entrypoint);

pub(crate) fn get_associated_vesting_address_and_bump_seed(
    wallet_address: &Pubkey,
    mint: &Pubkey,
    program_id: &Pubkey,
    vesting_program: &Pubkey,
) -> (Pubkey, u8) {
    get_associated_vesting_address_and_bump_seed_internal(
        wallet_address,
        mint,
        program_id,
        vesting_program,
    )
}

pub fn get_associated_vesting_address(
    wallet_address: &Pubkey,
    mint: &Pubkey,
) -> Pubkey {
    get_associated_vesting_addresss_with_program_id(
        wallet_address,
        mint,
        vesting_program,
    )
}

pub fn get_associated_vesting_address_and_bump_seed_internal(
    wallet_address: &Pubkey,
    mint: &Pubkey,
    program_id: &Pubkey,
    vesting_program: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            wallet_address.as_ref(),
            vesting_program.as_ref(),
            mint.as_ref(),
        ]
        program_id,
    )
}

pub fn entrypoint(program_id: &Pubkey, account_info: &[AccountInfo], data: &[u8]) -> ProgramResult {
    msg!("Entrypoint: Associated Vesting Account");

    if let Err(e) = process(program_id, accounts, data) {
        error.print::<ErrorCode>();
        return Err(e);
    }

    Ok(())
}

pub enum AssociatedVestingIx {
    Create,
}

impl AssociatedVestingIx;


pub struct Processor;

impl Processor {
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
        let ix = if input.is_empty() {
            AssociatedVestingIx::Create
        } else {
            AssociatedVestingIx::try_from_slice(input).map_err(|_| InvalidInstruction)?
        };

        match ix {
            AssociatedVestingIx::Create => {
                process_created_associated_vesting_accound(program_id, accounts)
            }
        }

        Ok(())
    }
}
