use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

pub(crate) const PK_LEN: usize = 32;

const IS_INIT: usize = 0;
const AUTH: usize = 1;
const VAULT: usize = AUTH + PK_LEN;
const DURA: usize = VAULT + PK_LEN;
const APR: usize = DURA + 8;
const WTL: usize = APR + 8;
const FEE: usize = WTL + 8;
const LIFE: usize = FEE + 8;

pub struct MetadataState {
    pub is_initialized: bool,
    pub authority: Pubkey,
    pub vault: Pubkey,
    pub duration: u64,
    pub apr: u64,
    pub withdrawal_timelock: u64,
    pub early_withdrawal_fee: u64,
    pub lifetime: u64,
}

impl IsInitialized for MetadataState {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Sealed for MetadataState {}

impl Pack for MetadataState {
    const LEN: usize = 111;

    fn pack_into_slice(&self, dst: &mut [u8]) {
        dst[IS_INIT] = self.is_initialized as u8;
        dst[AUTH..VAULT].copy_from_slice(self.authority.as_ref());
        dst[VAULT..DURA].copy_from_slice(self.vault.as_ref());
        dst[DURA..APR].copy_from_slice(&self.duration.to_le_bytes());
        dst[APR..WTL].copy_from_slice(&self.apr.to_le_bytes());
        dst[WTL..FEE].copy_from_slice(&self.withdrawal_timelock.to_le_bytes());
        dst[FEE..LIFE].copy_from_slice(&self.early_withdrawal_fee.to_le_bytes());
        dst[LIFE..].copy_from_slice(&self.lifetime.to_le_bytes());
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let is_initialized = match src[IS_INIT] {
            0 => false,
            1 => true,
            _ => unreachable!(),
        };

        let authority = Pubkey::new_from_array(src[AUTH..VAULT].try_into().unwrap());
        let vault = Pubkey::new_from_array(src[VAULT..DURA].try_into().unwrap());
        let duration = u64::from_le_bytes(src[DURA..APR].try_into().unwrap());
        let apr = u64::from_le_bytes(src[APR..WTL].try_into().unwrap());
        let withdrawal_timelock = u64::from_le_bytes(src[WTL..FEE].try_into().unwrap());
        let early_withdrawal_fee = u64::from_le_bytes(src[FEE..LIFE].try_into().unwrap());
        let lifetime = u64::from_le_bytes(src[LIFE..].try_into().unwrap());

        Ok(Self {
            is_initialized,
            authority,
            vault,
            duration,
            apr,
            withdrawal_timelock,
            early_withdrawal_fee,
            lifetime,
        })
    }
}
