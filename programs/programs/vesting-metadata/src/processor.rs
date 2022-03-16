use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    system_instruction,
};

use crate::{
    error::ErrorCode,
    instruction::{IxCtx, MetadataInstruction},
    state::MetadataState,
};

pub struct Processor;

impl Processor {
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
        let instruction = MetadataInstruction::unpack(data)?;

        match instruction {
            MetadataInstruction::Create(ix_ctx) => {
                Self::process_create(program_id, accounts, ix_ctx)?
            }
            MetadataInstruction::Update(ix_ctx) => {
                Self::process_update(program_id, accounts, ix_ctx)?
            }
            MetadataInstruction::Delete(ix_ctx) => {
                Self::process_delete(program_id, accounts, ix_ctx)?
            }
        }

        Ok(())
    }

    fn process_create(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        ix_ctx: IxCtx,
    ) -> Result<(), ProgramError> {
        let accounts_iter = &mut accounts.iter();

        let authority = next_account_info(accounts_iter)?;
        let metadata_account = next_account_info(accounts_iter)?;

        let mut metadata_data = metadata_account.data.borrow_mut();

        let metadata = MetadataState {
            is_initialized: true,
            authority: *authority.key,
            vault: ix_ctx.vault,
            duration: ix_ctx.duration,
            apr: ix_ctx.apr,
            withdrawal_timelock: ix_ctx.withdrawal_timelock,
            early_withdrawal_fee: ix_ctx.early_withdrawal_fee,
            lifetime: ix_ctx.lifetime,
        };

        metadata.pack_into_slice(&mut metadata_data);
        Ok(())
    }

    fn process_update(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        ix_ctx: IxCtx,
    ) -> Result<(), ProgramError> {
        let accounts_iter = &mut accounts.iter();

        let authority = next_account_info(accounts_iter)?;
        let metadata_account = next_account_info(accounts_iter)?;

        let mut metadata_data = metadata_account.data.borrow_mut();

        let metadata = MetadataState {
            is_initialized: true,
            authority: *authority.key,
            vault: ix_ctx.vault,
            duration: ix_ctx.duration,
            apr: ix_ctx.apr,
            withdrawal_timelock: ix_ctx.withdrawal_timelock,
            early_withdrawal_fee: ix_ctx.early_withdrawal_fee,
            lifetime: ix_ctx.lifetime,
        };

        metadata.pack_into_slice(&mut metadata_data);
        Ok(())
    }

    fn process_delete(
        _program_id: &Pubkey,
        _accounts: &[AccountInfo],
        _ix_ctx: IxCtx,
    ) -> Result<(), ProgramError> {
        msg!("Ok");
        /*
        let accounts_iter = &mut accounts.iter();

        let authority = next_account_info(accounts_iter)?;
        let metadata_account = next_account_info(accounts_iter)?;

        let metadata = metadata_account.data.borrow_mut();
            &[0u8; MetadataState::LEN];

        system_instruction::transfer(
            from_pubkey: metadata_account,
            to_pubkey: authority,
            lamports: metadata_account.lamports,
        );

        if metadata_account.lamports != 0 {
            return Err();
        } else {
            Ok(())
        }
        */
        Ok(())
    }
}
