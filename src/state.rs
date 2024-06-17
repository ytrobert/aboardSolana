use solana_program::{
    //msg,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};
use std::collections::BTreeMap;
use borsh::{BorshDeserialize, BorshSerialize};
//yt: program state objects, (de)serializing data arrays of u8
use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};

const WITHDRAWIDMAP_BYTES: usize = 100;
const ACCOUNT_BYTES: usize = 1 + 32 + 1 + WITHDRAWIDMAP_BYTES;
const TOKENMAP_BYTES: usize = 2000;
const PERPETUAL_BYTES: usize = 1 + 64 + 32 + 32 + 1 + 4 + TOKENMAP_BYTES;

//can be: pub const fn from_le_bytes(bytes: [u8; 4]) -> u32
fn count_from_le(array: &[u8]) -> usize {
    (array[0] as usize)
        | (array[1] as usize) << 8
        | (array[2] as usize) << 16
        | (array[3] as usize) << 24
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Hash, Ord, Eq, PartialEq, PartialOrd)]
pub struct TypeSymbol {
    pub account_type: u8, //from 0
    pub symbol: String,   //USDC, SOL, ...
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Copy, Clone)]
pub struct MintProgram {
    pub mint: Pubkey, 
    pub program_token_account: Pubkey,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Account {
    pub is_initialized: bool,
    pub user: Pubkey,
    pub withdraw_id: BTreeMap<u8, u64>,
}

impl Sealed for Account {} //trait in program_pack size

impl IsInitialized for Account {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for Account {
    const LEN: usize = ACCOUNT_BYTES;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, Account::LEN]; //get references to sections of a slice
        let (
            is_initialized,
            user,
            withdrawid_len,
            withdraw_id,
        ) = array_refs![src, 1, 32, 1, WITHDRAWIDMAP_BYTES];
        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };
        let user = Pubkey::new_from_array(*user);
        //let withdraw_id = u16::from_le_bytes(*withdraw_id);
        //withdrawid map
        let withdrawid_len = u8::from_le_bytes(*withdrawid_len) as usize; //count_from_le(position_len);
        let withdraw_id = 
            if withdrawid_len == 0 {BTreeMap::<u8, u64>::new()}
            else                   {BTreeMap::<u8, u64>::try_from_slice(&withdraw_id[0..withdrawid_len]).unwrap()};
        //return
        Ok(Account {
            is_initialized,
            user,
            withdraw_id,
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, Account::LEN];
        let (
            is_initialized_dst,
            user_dst,
            withdrawid_len,
            withdraw_id_dst,
        ) = mut_array_refs![dst, 1, 32, 1, WITHDRAWIDMAP_BYTES];

        let Account {
            is_initialized,
            user,
            withdraw_id,
        } = self;

        is_initialized_dst[0] = *is_initialized as u8;
        user_dst.copy_from_slice(user.as_ref());
        //withdrawid map
        let data_ser = withdraw_id.try_to_vec().unwrap();
        //msg!("withdraw_id bytes len:{}",data_ser.len());
        *withdrawid_len = (data_ser.len() as u8).to_le_bytes();
        withdraw_id_dst[..data_ser.len()].copy_from_slice(&data_ser);
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Perpetual {
    //del or add more checks
    pub is_initialized: bool,
    pub secp256k1_pubkey: [u8; 64],
    pub gateway: Pubkey,
    pub admin: Pubkey,
    //pub program_token_account: Pubkey,
    //pub pda: Pubkey,
    pub bump_seed: u8,
    pub token_map: BTreeMap<TypeSymbol, MintProgram>,
    //pub user_map: BTreeMap<Pubkey, bool>,
}

impl Sealed for Perpetual {} //trait in program_pack

impl IsInitialized for Perpetual {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for Perpetual {
    const LEN: usize = PERPETUAL_BYTES;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, Perpetual::LEN]; //get references to sections of a slice
        let (
            is_initialized,
            secp256k1_pubkey,
            gateway,
            admin,
            bump_seed,
            token_map_len,
            token_map,
        ) = array_refs![src, 1, 64, 32, 32, 1, 4, TOKENMAP_BYTES];
        //every data from &[u8; _]
        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };
        let secp256k1_pubkey = *secp256k1_pubkey;
        let gateway = Pubkey::new_from_array(*gateway);
        let admin = Pubkey::new_from_array(*admin);
        let bump_seed = bump_seed[0];
        //token map
        let token_map_len = count_from_le(token_map_len);
        let token_map = 
            if token_map_len == 0 {BTreeMap::<TypeSymbol, MintProgram>::new()}
            else                  {BTreeMap::<TypeSymbol, MintProgram>::try_from_slice(&token_map[0..token_map_len]).unwrap()};
        //return
        Ok(Perpetual {
            is_initialized,
            secp256k1_pubkey,
            gateway,
            admin,
            bump_seed,
            token_map,
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, Perpetual::LEN];
        let (
            is_initialized_dst,
            secp256k1_pubkey_dst,
            gateway_dst,
            admin_dst,
            bump_seed_dst,
            token_map_len,
            token_map_dst,
        ) = mut_array_refs![dst, 1, 64, 32, 32, 1, 4, TOKENMAP_BYTES];

        let Perpetual {
            is_initialized,
            secp256k1_pubkey,
            gateway,
            admin,
            bump_seed,
            token_map,
        } = self;

        is_initialized_dst[0] = *is_initialized as u8;
        //*signer_eth_pubkey_dst = *signer_eth_pubkey;
        secp256k1_pubkey_dst.copy_from_slice(secp256k1_pubkey);
        gateway_dst.copy_from_slice(gateway.as_ref());
        admin_dst.copy_from_slice(admin.as_ref());
        bump_seed_dst[0] = *bump_seed;
        //token_map
        let data_ser = token_map.try_to_vec().unwrap();
        //msg!("token_map bytes len:{}",data_ser.len());
        *token_map_len = (data_ser.len() as u32).to_le_bytes();
        token_map_dst[..data_ser.len()].copy_from_slice(&data_ser);
    }
}
