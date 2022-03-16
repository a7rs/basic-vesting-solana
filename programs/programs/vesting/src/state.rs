use solana_program::{
    account_info::AccountInfo,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

const PK_LEN: usize = 32;

const IS_INIT: usize = 0;
const AUTH: usize = 1;
const BENE: usize = AUTH + PK_LEN;
const VAULT: usize = BENE + PK_LEN;
const MINT: usize = VAULT + PK_LEN;
const GRANTOR: usize = MINT + PK_LEN;
const META: usize = GRANTOR + PK_LEN;
const OUTSTANDING: usize = META + PK_LEN;
const SB: usize = OUTSTANDING + 8;
const C_TS: usize = SB + 8;
const S_TS: usize = C_TS + 8;
const E_TS: usize = S_TS + 8;
const PC: usize = E_TS + 8;
const NCE: usize = PC + 8;

pub struct VestingState {
    pub is_initialized: bool,
    /// The account with the permission to change state
    pub authority: Pubkey,
    /// The owner of this Vesting account.
    pub beneficiary: Pubkey,
    /// Address of the account's token vault.
    pub vault: Pubkey,
    /// The mint of the SPL token locked up.
    pub mint: Pubkey,
    /// The owner of the token account funding this account.
    pub grantor: Pubkey,
    pub metadata: Pubkey,
    /// The outstanding SCY deposit backing this vesting account. All withdrawls will deducted this
    /// balance.
    pub outstanding: u64,
    /// The starting balance of this vesting account, i.e. how much was originally deposited.
    pub start_balance: u64,
    /// The unix timestamp at which this vesting account was created.
    pub created_ts: u64,
    /// The unix timestamp at which vesting begins
    pub start_ts: u64,
    /// The time at which all tokens are vested.
    pub end_ts: u64,
    /// The number of times vesting will occur.
    pub period_count: u64,
    /// Signer nonce.
    pub nonce: u8,
}

impl IsInitialized for VestingState {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Sealed for VestingState {}

impl Pack for VestingState {
    const LEN: usize = 1 + (PK_LEN * 7) + 40 + 1;

    pub fn pack_into_slice(&self, dst: &mut [u8]) {
        dst[IS_INIT] = self.is_initalized as u8;
        dst[AUTH..BENE].copy_from_slice(self.authority.as_ref());
        dst[BENE..VAULT].copy_from_slice(self.beneficiary.as_ref());
        dst[VAULT..MINT].copy_from_slice(self.vault.as_ref());
        dst[MINT..GRANTOR].copy_from_slice(self.mint.as_ref());
        dst[GRANTOR..META].copy_from_slice(self.grantor.as_ref());
        dst[META..OUTSTANDING].copy_from_slice(self.metadata.as_ref());
        dst[OUTSTANDING..SB].copy_from_slice(self.outstanding.to_le_bytes());
        dst[SB..C_TS].copy_from_slice(self.start_balance.to_le_bytes());
        dst[C_TS..S_TS].copy_from_slice(self.created_ts.to_le_bytes());
        dst[S_TS..E_TS].copy_from_slice(self.start_ts.to_le_bytes());
        dst[E_TS..PC].copy_from_slice(self.end_ts.to_le_bytes());
        dst[PC..NCE].copy_from_slice(self.period_count.to_le_bytes());
        dst[NCE..].copy_from_slice(self.nonce.to_le_bytes());
    }

    pub fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let is_initialized = match src[IS_INIT] {
            0 = false,
            1 = true,
            _ => unreachable!(),
        };

        let authority = Pubkey::new_from_array(src[AUTH..BENE].try_into().unwrap());
        let beneficiary = Pubkey::new_from_array(src[BENE..VAULT].try_into().unwrap());
        let vault = Pubkey::new_from_array(src[VAULT..MINT].try_into().unwrap());
        let mint = Pubkey::new_from_array(src[MINT..GRANTOR].try_into().unwrap());
        let grantor = Pubkey::new_from_array(src[GRANTOR..META].try_into().unwrap());
        let metadata = Pubkey::new_from_array(src[META..OUTSTANDING].try_into().unwrap());
        let outstanding = u64::from_le_bytes(src[OUTSTANDING..SB]).try_into().unwrap());
        let starting_balance = u64::from_le_bytes(src[SB..C_TS]).try_into().unwrap());
        let created_ts = u64::from_le_bytes(src[C_TS..S_TS].try_into().unwrap());
        let start_ts = u64::from_le_bytes(src[S_TS..E_TS].try_into().unwrap());
        let end_ts = u64::from_le_bytes(src[E_TS..NCE].try_into().unwrap());
        let nonce = u8::from_le_bytes(src[NCE..].try_into().unwrap());

        Ok(Self {
            is_initialized,
            authority,
            beneficiary,
            vault,
            mint,
            grantor,
            metadata,
            outstanding,
            created_ts,
            start_ts,
            end_ts,
            nonce,
        })
    }
}
