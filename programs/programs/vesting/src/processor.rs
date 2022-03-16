use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};
pub struct Processor;

impl Processor {
    pub fn process_instruction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        data: &[u8],
    ) -> ProgramResult {
        let instruction = VestingInstruction::unpack(data)?;

        match instruction {
            VestingInstruction::Init {
                seed,
                release_count,
            } => {
                Self::process_init(program_id, accounts, seed, release_count)?;
            }
            VestingInstruction::Create {
                seed,
                mint,
                recipient,
                releases,
            } => {
                Self::process_create_vc(program_id, accounts, seed, mint, recipient, releases)?;
            }
            VestingInstruction::Unlock { seed } => {
                Self::process_unlock_tokens(program_id, accounts, seed)?;
            }
        }
        Ok(())
    }

    fn process_init(program_id: &Pubkey, accounts: &[AccountInfo]) -> Result<(), ProgramError> {
        msg!("Instruction: Initialize Accounts");
        let accounts_iter = &mut accounts.iter();

        let authority = next_account_info(accounts_iter)?;
        let vesting = next_account_info(accounts_iter)?;
        let system_program = next_account_info(accounts_iter)?;

        let rent = Rent::get()?;
        let vesting_account_pda = Pubkey::create_program_address(&[&seed], &program_id).unwrap();
        if vesting_account_pda != *vesting_account.key {
            msg!("Incorrect vesting account address");
            return Err(ProgramError::InvalidArgument);
        }

        let state_size = (release_count as usize) * VestingInfo::LEN + VestingHeader::LEN;

        let initialize_vesting_account_ix = create_account(
            &token_authority.key,
            &vesting_account_pda,
            rent.minimum_balance(state_size),
            state_size as u64,
            &program_id,
        );

        invoke_signed(
            &initialize_vesting_account_ix,
            &[
                system_program.clone(),
                token_authority.clone(),
                vesting_account.clone(),
            ],
            &[&[&seed]],
        )?;

        Ok(())
    }

    fn process_create_vesting(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        seed: [u8; 32],
        mint: Pubkey,
        recipient: Pubkey,
        releases: Vec<VestingInfo>,
    ) -> Result<(), ProgramError> {
        msg!("Instruction: Create Vesting Contract");
        let accounts_iter = &mut accounts.iter();

        let authority = next_account_info(accounts_iter)?;
        let token_account = next_account_info(accounts_iter)?;
        let vesting_account = next_account_info(accounts_iter)?;
        let vesting_vault = next_account_info(accounts_iter)?;
        let token_program = next_account_info(accounts_iter)?;

        if !authority.is_signer {
            msg!("Authority must be a signer");
            return Err(ProgramError::InvalidArgument);
        }

        let vesting_account_pda = Pubkey::create_program_address(&[&seed], &program_id)?;
        if vesting_account_pda != *vesting_account.key {
            msg!("Incorrect vesting account address");
            return Err(ProgramError::InvalidArgument);
        }
        if *vesting_account.owner != *program_id {
            msg!("Vesting program must own the vesting account");
            return Err(ProgramError::InvalidArgument);
        }

        let is_initialized = vesting_account.try_borrow_data()?[0] == 1;

        if is_initialized {
            msg!("Vesting contract with this seed already exists");
            return Err(ProgramError::InvalidArgument);
        }
        let vesting_vault_data = Account::unpack(&vesting_vault.data.borrow())?;

        if vesting_vault_data.owner != vesting_account_pda {
            msg!("Vesting vault is not owned by the vesting program");
            return Err(ProgramError::InvalidArgument);
        }

        if vesting_vault_data.delegate.is_some() {
            msg!("Vesting vault should not have a delegate authority");
            return Err(ProgramError::InvalidArgument);
        }

        if vesting_vault_data.close_authority.is_some() {
            msg!("Vesting vault should not have a close authority");
            return Err(ProgramError::InvalidArgument);
        }

        let header = VestingHeader {
            is_initialized: true,
            mint,
            recipient,
        };

        let mut data = vesting_account.data.borrow_mut();
        let mut offset = VestingHeader::LEN;
        let mut total_tokens: u64 = 0;
        header.pack_into_slice(&mut data);

        for release in releases {
            let increment = total_tokens.checked_add(release.quantity);
            match increment {
                Some(n) => total_tokens = n,
                None => return Err(ProgramError::InvalidInstructionData),
            };
            msg!("{}", total_tokens);
            release.pack_into_slice(&mut data[offset..]);
            offset += VestingInfo::LEN;
        }

        if Account::unpack(&token_account.data.borrow())?.amount < total_tokens {
            msg!("Token vault has insufficient funds.");
            return Err(ProgramError::InsufficientFunds);
        }

        let transfer_tokens_ix = transfer(
            spl_token_program.key,
            token_account.key,
            vesting_vault.key,
            token_authority.key,
            &[],
            total_tokens,
        )?;

        invoke(
            &transfer_tokens_ix,
            &[
                token_account.clone(),
                vesting_vault.clone(),
                token_authority.clone(),
            ],
        )?;

        Ok(())
    }

    fn process_unlock_tokens(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        seed: [u8; 32],
    ) -> Result<(), ProgramError> {
        msg!("Instruction: Unlock Tokens");
        let accounts_iter = &mut accounts.iter();

        let vesting_account = next_account_info(accounts_iter)?;
        let vesting_vault = next_account_info(accounts_iter)?;
        let recipient_account = next_account_info(accounts_iter)?;
        let recipient_token_account = next_account_info(accounts_iter)?;
        let spl_token_program = next_account_info(accounts_iter)?;

        let clock = Clock::get()?;

        let vesting_account_pda = Pubkey::create_program_address(&[&seed], &program_id)?;
        if vesting_account_pda != *vesting_account.key {
            msg!("Incorrect vesting account address");
            return Err(ProgramError::InvalidArgument);
        }
        if *vesting_account.owner != *program_id {
            msg!("Synchrony vesting program must own the vesting account");
            return Err(ProgramError::InvalidArgument);
        }

        if *spl_token_program.key != spl_token::id() {
            msg!("Incorrect spl-token ID");
            return Err(ProgramError::IncorrectProgramId);
        }

        let header = VestingHeader::unpack(&vesting_account.data.borrow()[..VestingHeader::LEN])?;
        if header.recipient != *recipient_account.key {
            msg!("recipient address does not match");
            return Err(ProgramError::InvalidArgument);
        }

        let vesting_vault_data = Account::unpack(&vesting_vault.data.borrow())?;
        if vesting_vault_data.owner != vesting_account_pda {
            msg!("Vault is not owned by the vesting account");
            return Err(ProgramError::InvalidArgument);
        }

        let mut offset = VestingHeader::LEN;
        let mut release_quantity = 0;
        let mut releases = unpack_releases(&vesting_account.data.borrow()[offset..])?;

        for release in releases.iter_mut() {
            if clock.unix_timestamp as u32 >= release.timestamp {
                release_quantity += release.quantity;
                release.quantity = 0;
            }
        }

        if release_quantity == 0 {
            msg!("No vesting periods have elapsed...");
            return Err(ProgramError::InvalidArgument);
        }

        let release_tokens_ix = transfer(
            spl_token_program.key,
            vesting_vault.key,
            recipient_token_account.key,
            &vesting_account_pda,
            &[],
            release_quantity,
        )?;

        invoke_signed(
            &release_tokens_ix,
            &[
                vesting_vault.clone(),
                recipient_token_account.clone(),
                vesting_account.clone(),
            ],
            &[&[&seed]],
        )?;

        let mut data = vesting_account.data.borrow_mut();
        offset = VestingHeader::LEN;
        for release in releases {
            release.pack_into_slice(&mut data[offset..offset + VestingInfo::LEN]);
            offset += VestingInfo::LEN;
        }

        Ok(())
    }
}
