use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    system_instruction,
};

use crate::{
    error::ErrorCode,
    state::MetadataState,
    instruction::{
        MetadataInstruction,
        IxCtx,
    },
};

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
        };

        metadata.pack_into_slice(&mut metadata_data);
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
        );

        if metadata_account.lamports != 0 {
            return Err(ErrorCode::RemainingBalance);
        } else {
            Ok(())
        }
    }
}
